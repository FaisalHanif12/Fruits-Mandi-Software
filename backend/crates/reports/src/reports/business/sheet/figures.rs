//! The figures of one party on one day, for the day slips (R35) and the day total sheet (R34)
//! (plan 42, PRD FR-19.12 v1.12, R34).
//!
//! One sign rule runs through the code: a signed amount is the ledger net, debit minus credit, so
//! a debit (بنام, he owes the shop) is positive and a credit (جمع, the shop owes him) is negative.
//! The side word is chosen only when a cell is built.
//!
//! A buyer's day, from the stored sale lines and the journal by kind of posting:
//!
//! * today's total = Σ (value + commission + لاگا) of his active sale lines (C5), each line checked;
//! * paid today = the cash part of his نقدی lines + his receipts;
//! * other entries = what the journal holds on him that day besides the sale and its cash part and
//!   the receipts (cash lent, a جنرل line, an opening), debits less credits;
//! * today's left = today's total + other − paid; new balance = previous + today's left.
//!
//! A seller's day:
//!
//! * `sale` = (commission + lot charges) − gross, the negative of the net sale (C8) so that it
//!   follows the one sign rule; the lot charges are those of the lots whose first posting is dated
//!   the day;
//! * paid today = the نقد given with his lots + his payments (ادائیگی);
//! * other entries as for a buyer; today's left = sale + other + paid (a payment is a debit).
//!
//! Before anything is returned, the sale lines are matched to the journal (the debits of the
//! بکری postings on a buyer must be his total, the cash part his نقدی lines; on a seller the
//! بکری postings must give `net sale − نقد`), the other entries must not be negative on either
//! side, and the new balance must be the ledger balance at the end of the day. A mismatch is an
//! error, never a printed wrong slip (plan 42, agent rule 3).

use std::collections::{BTreeMap, HashMap};

use super::figures_load::{
    BaseRow, BuyerLine, Filter, LedgerRow, LotCharges, SellerLine, base_rows, buyer_lines,
    ledger_rows, lot_charges, seller_lines,
};
use super::*;

/// The party of a figure: account and person (empty for a plain account).
pub(super) type PartyKey = (i64, String);

/// A buyer's day.
#[derive(Debug, Clone)]
pub(super) struct BuyerDay {
    pub(super) account_id: i64,
    pub(super) person: String,
    /// The ledger balance at the start of the day, signed.
    pub(super) previous: Paisa,
    pub(super) lines: Vec<BuyerLine>,
    pub(super) goods: Paisa,
    pub(super) commission: Paisa,
    pub(super) laga: Paisa,
    pub(super) total: Paisa,
    pub(super) qty: i64,
    pub(super) paid: Paisa,
    /// Other entries, debits less credits.
    pub(super) other: Paisa,
    /// True when there is a debit or a credit among the other entries, even if they cancel.
    pub(super) has_other: bool,
    pub(super) left: Paisa,
    pub(super) new: Paisa,
    /// True when the party has a line on the books or a sale line that day.
    pub(super) has_entry: bool,
}

/// A seller's day.
#[derive(Debug, Clone)]
pub(super) struct SellerDay {
    pub(super) account_id: i64,
    pub(super) person: String,
    pub(super) previous: Paisa,
    pub(super) lines: Vec<SellerLine>,
    pub(super) gross: Paisa,
    pub(super) commission: Paisa,
    pub(super) freight: Paisa,
    pub(super) labour: Paisa,
    pub(super) munshiana: Paisa,
    pub(super) phone: Paisa,
    pub(super) store_rent: Paisa,
    /// The negative of the net sale: (commission + charges) − gross. A credit of the day is a
    /// negative number here, as everywhere.
    pub(super) sale: Paisa,
    pub(super) paid: Paisa,
    pub(super) other: Paisa,
    pub(super) has_other: bool,
    pub(super) left: Paisa,
    pub(super) new: Paisa,
    pub(super) has_entry: bool,
}

/// The days asked for, with the figures of every party that has a day in them.
#[derive(Debug, Default)]
pub(super) struct Figures {
    pub(super) buyers: BTreeMap<(String, i64, String), BuyerDay>,
    pub(super) sellers: BTreeMap<(String, i64, String), SellerDay>,
}

/// One party asked for by name, for a slip.
#[derive(Debug, Clone)]
pub(super) struct Who {
    pub(super) account: i64,
    pub(super) person: Option<String>,
    pub(super) buyer: bool,
}

/// The lines of each party, by day.
type DayMap<T> = HashMap<PartyKey, BTreeMap<String, Vec<T>>>;

/// What a party has on the books: the start balance and the days in the range.
#[derive(Default)]
struct KeyData {
    buyer: bool,
    base_dr: Paisa,
    base_cr: Paisa,
    rows: Vec<LedgerRow>,
}

fn data_error(text: &str) -> ReportsError {
    ReportsError::Data(text.to_owned())
}

fn count_add(total: i64, qty: i64) -> Result<i64, ReportsError> {
    total
        .checked_add(qty)
        .ok_or_else(|| data_error("the quantities do not fit"))
}

/// The figures of the `days` (sorted, oldest first). With `who`, only that party, and it has a day
/// even when nothing happened (a slip is then empty). Without it, every party that has an entry on
/// a day.
///
/// # Errors
/// A database error, `ReportsError::Money` if a sum does not fit, and `ReportsError::Data` when the
/// sale lines do not agree with the journal or the day does not add up to the ledger.
pub(super) fn load(
    conn: &Connection,
    days: &[String],
    who: Option<&Who>,
) -> Result<Figures, ReportsError> {
    let (Some(from), Some(to)) = (days.first(), days.last()) else {
        return Ok(Figures::default());
    };
    let filter = Filter {
        account: who.map(|w| w.account),
        person: who.and_then(|w| w.person.as_deref()),
    };
    let mut data: HashMap<PartyKey, KeyData> = HashMap::new();
    for BaseRow {
        account_id,
        person,
        buyer,
        dr,
        cr,
    } in base_rows(conn, from, filter)?
    {
        let entry = data.entry((account_id, person)).or_default();
        entry.buyer = buyer;
        entry.base_dr = dr;
        entry.base_cr = cr;
    }
    for row in ledger_rows(conn, from, to, filter)? {
        let entry = data
            .entry((row.account_id, row.person.clone()))
            .or_default();
        entry.buyer = row.buyer;
        entry.rows.push(row);
    }
    if let Some(w) = who {
        let entry = data
            .entry((w.account, w.person.clone().unwrap_or_default()))
            .or_default();
        entry.buyer = w.buyer;
    }

    // The lines of each party, by day. Keyed by the party first, so the loop below looks a party
    // up by the key it already holds and copies nothing.
    let mut buyer_by: DayMap<BuyerLine> = HashMap::new();
    for line in buyer_lines(conn, from, to, filter)? {
        buyer_by
            .entry((line.account_id, line.person.clone()))
            .or_default()
            .entry(line.day.clone())
            .or_default()
            .push(line);
    }
    let mut seller_by: DayMap<SellerLine> = HashMap::new();
    for line in seller_lines(conn, from, to, filter)? {
        seller_by
            .entry((line.account_id, line.person.clone()))
            .or_default()
            .entry(line.day.clone())
            .or_default()
            .push(line);
    }
    let mut charges_by: DayMap<LotCharges> = HashMap::new();
    for charges in lot_charges(conn, from, to, filter)? {
        charges_by
            .entry((charges.account_id, charges.person.clone()))
            .or_default()
            .entry(charges.day.clone())
            .or_default()
            .push(charges);
    }
    // A party that has sale lines or lot charges is in the figures even when the books hold no
    // line for him (a lot whose net is 0 posts nothing): no line is ever lost.
    for key in buyer_by.keys() {
        data.entry(key.clone()).or_insert_with(|| KeyData {
            buyer: true,
            ..KeyData::default()
        });
    }
    for key in seller_by.keys().chain(charges_by.keys()) {
        data.entry(key.clone()).or_default();
    }

    let asked = who.is_some();
    let mut out = Figures::default();
    for (party_key, key) in &data {
        // A party with nothing on the books in the range and no line is not worked out.
        if !asked
            && key.rows.is_empty()
            && !buyer_by.contains_key(party_key)
            && !seller_by.contains_key(party_key)
            && !charges_by.contains_key(party_key)
        {
            continue;
        }
        let (account_id, person) = party_key;
        let mut run_dr = key.base_dr;
        let mut run_cr = key.base_cr;
        let mut next = 0_usize;
        for day in days {
            while let Some(row) = key.rows.get(next).filter(|r| r.date < *day) {
                add(&mut run_dr, row.all_dr)?;
                add(&mut run_cr, row.all_cr)?;
                next = next.saturating_add(1);
            }
            let previous = run_dr.checked_sub(run_cr)?;
            let today = key.rows.get(next).filter(|r| r.date == *day);
            let end = match today {
                Some(row) => run_dr
                    .checked_add(row.all_dr)?
                    .checked_sub(run_cr.checked_add(row.all_cr)?)?,
                None => previous,
            };
            let has_lines = [
                buyer_by.get(party_key).is_some_and(|m| m.contains_key(day)),
                seller_by
                    .get(party_key)
                    .is_some_and(|m| m.contains_key(day)),
                charges_by
                    .get(party_key)
                    .is_some_and(|m| m.contains_key(day)),
            ]
            .contains(&true);
            // A party with nothing on the books and no line that day has no figures.
            if !asked && today.is_none() && !has_lines {
                continue;
            }
            let day_row = today.cloned().unwrap_or_else(|| empty_row(day, key.buyer));
            let map_key = (day.clone(), *account_id, person.clone());
            if key.buyer {
                let lines = buyer_by
                    .get_mut(party_key)
                    .and_then(|m| m.remove(day))
                    .unwrap_or_default();
                let figure = buyer_day(*account_id, person, previous, end, &day_row, lines)?;
                if asked || figure.has_entry {
                    out.buyers.insert(map_key, figure);
                }
            } else {
                let lines = seller_by
                    .get_mut(party_key)
                    .and_then(|m| m.remove(day))
                    .unwrap_or_default();
                let charges = charges_by
                    .get_mut(party_key)
                    .and_then(|m| m.remove(day))
                    .unwrap_or_default();
                let figure = seller_day(
                    *account_id,
                    person,
                    previous,
                    end,
                    &day_row,
                    lines,
                    &charges,
                )?;
                if asked || figure.has_entry {
                    out.sellers.insert(map_key, figure);
                }
            }
        }
    }
    Ok(out)
}

fn empty_row(day: &str, buyer: bool) -> LedgerRow {
    LedgerRow {
        account_id: 0,
        person: String::new(),
        buyer,
        date: day.to_owned(),
        all_dr: Paisa::ZERO,
        all_cr: Paisa::ZERO,
        dr: Paisa::ZERO,
        cr: Paisa::ZERO,
        bikri_dr: Paisa::ZERO,
        bikri_cr: Paisa::ZERO,
        receipts_cr: Paisa::ZERO,
        payments_dr: Paisa::ZERO,
    }
}

/// A day on the books of a buyer.
fn buyer_day(
    account_id: i64,
    person: &str,
    previous: Paisa,
    end: Paisa,
    today: &LedgerRow,
    lines: Vec<BuyerLine>,
) -> Result<BuyerDay, ReportsError> {
    let (mut goods, mut commission, mut laga, mut total, mut on_spot) = (
        Paisa::ZERO,
        Paisa::ZERO,
        Paisa::ZERO,
        Paisa::ZERO,
        Paisa::ZERO,
    );
    let mut qty = 0_i64;
    for line in &lines {
        // C5: the total is the value plus the commission plus لاگا, on every line.
        let parts = line
            .value
            .checked_add(line.commission)?
            .checked_add(line.laga)?;
        if parts != line.total {
            return Err(data_error(
                "a sale line's total is not its value + commission + laga",
            ));
        }
        add(&mut goods, line.value)?;
        add(&mut commission, line.commission)?;
        add(&mut laga, line.laga)?;
        add(&mut total, line.total)?;
        if line.cash {
            add(&mut on_spot, line.total)?;
        }
        qty = count_add(qty, line.qty)?;
    }
    // The sale lines must be what the بکری postings put on the buyer.
    if today.bikri_dr != total || today.bikri_cr != on_spot {
        return Err(data_error(
            "the sale lines of the day do not match the ledger of the buyer",
        ));
    }
    let other_dr = today.dr.checked_sub(today.bikri_dr)?;
    let other_cr = today
        .cr
        .checked_sub(today.bikri_cr)?
        .checked_sub(today.receipts_cr)?;
    if other_dr.is_negative() || other_cr.is_negative() {
        return Err(data_error(
            "the ledger of the buyer holds less than his sale lines and receipts",
        ));
    }
    let other = other_dr.checked_sub(other_cr)?;
    let paid = on_spot.checked_add(today.receipts_cr)?;
    let left = total.checked_add(other)?.checked_sub(paid)?;
    let new = previous.checked_add(left)?;
    if new != end {
        return Err(data_error(
            "the day of a buyer does not add up to his ledger balance",
        ));
    }
    let has_entry = today.dr != Paisa::ZERO || today.cr != Paisa::ZERO || !lines.is_empty();
    Ok(BuyerDay {
        account_id,
        person: person.to_owned(),
        previous,
        lines,
        goods,
        commission,
        laga,
        total,
        qty,
        paid,
        other,
        has_other: other_dr != Paisa::ZERO || other_cr != Paisa::ZERO,
        left,
        new,
        has_entry,
    })
}

/// A day on the books of a seller.
#[allow(clippy::too_many_arguments)]
fn seller_day(
    account_id: i64,
    person: &str,
    previous: Paisa,
    end: Paisa,
    today: &LedgerRow,
    lines: Vec<SellerLine>,
    charges: &[LotCharges],
) -> Result<SellerDay, ReportsError> {
    let (mut gross, mut commission) = (Paisa::ZERO, Paisa::ZERO);
    for line in &lines {
        add(&mut gross, line.amount)?;
        add(&mut commission, line.commission)?;
    }
    let (mut freight, mut labour, mut munshiana, mut phone) =
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    let (mut store_rent, mut cash_given) = (Paisa::ZERO, Paisa::ZERO);
    for c in charges {
        add(&mut freight, c.freight)?;
        add(&mut labour, c.labour)?;
        add(&mut munshiana, c.munshiana)?;
        add(&mut phone, c.phone)?;
        add(&mut store_rent, c.store_rent)?;
        add(&mut cash_given, c.cash)?;
    }
    let mut deductions = commission;
    for part in [freight, labour, munshiana, phone, store_rent] {
        add(&mut deductions, part)?;
    }
    // C8 with the sign of the ledger: a credit of the day is negative.
    let sale = deductions.checked_sub(gross)?;
    // The بکری postings give the seller: credits for the lots with a net, debits for the نقد and
    // for a lot whose net is negative. Together: debit − credit = نقد + sale.
    let bikri_net = today.bikri_dr.checked_sub(today.bikri_cr)?;
    if bikri_net != cash_given.checked_add(sale)? {
        return Err(data_error(
            "the sale lines of the day do not match the ledger of the seller",
        ));
    }
    let other_dr = today
        .dr
        .checked_sub(today.bikri_dr)?
        .checked_sub(today.payments_dr)?;
    let other_cr = today.cr.checked_sub(today.bikri_cr)?;
    if other_dr.is_negative() || other_cr.is_negative() {
        return Err(data_error(
            "the ledger of the seller holds less than his sales and payments",
        ));
    }
    let other = other_dr.checked_sub(other_cr)?;
    let paid = cash_given.checked_add(today.payments_dr)?;
    let left = sale.checked_add(other)?.checked_add(paid)?;
    let new = previous.checked_add(left)?;
    if new != end {
        return Err(data_error(
            "the day of a seller does not add up to his ledger balance",
        ));
    }
    let has_entry = today.dr != Paisa::ZERO
        || today.cr != Paisa::ZERO
        || !lines.is_empty()
        || !charges.is_empty();
    Ok(SellerDay {
        account_id,
        person: person.to_owned(),
        previous,
        lines,
        gross,
        commission,
        freight,
        labour,
        munshiana,
        phone,
        store_rent,
        sale,
        paid,
        other,
        has_other: other_dr != Paisa::ZERO || other_cr != Paisa::ZERO,
        left,
        new,
        has_entry,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::model::Names;
    use proptest::prelude::*;

    fn rupees(n: i64) -> Paisa {
        Paisa::from_rupees(n).unwrap()
    }

    fn goods() -> super::super::figures_load::Goods {
        super::super::figures_load::Goods {
            item_id: 1,
            packing_id: 1,
            item: Names {
                ur: "کیلا".to_owned(),
                en: "Banana".to_owned(),
            },
            packing: Names {
                ur: "پیٹی".to_owned(),
                en: "Box".to_owned(),
            },
        }
    }

    fn row(dr: Paisa, cr: Paisa) -> LedgerRow {
        LedgerRow {
            dr,
            cr,
            all_dr: dr,
            all_cr: cr,
            ..empty_row("2026-10-10", true)
        }
    }

    /// (value, commission, laga, cash) of a buyer's line, in whole rupees.
    type Line = (i64, i64, i64, bool);

    fn buyer_lines_of(lines: &[Line]) -> Vec<BuyerLine> {
        lines
            .iter()
            .map(|&(value, commission, laga, cash)| BuyerLine {
                day: "2026-10-10".to_owned(),
                account_id: 1,
                person: String::new(),
                bikri_id: 1,
                goods: goods(),
                qty: 1,
                weight_grams: None,
                rate: rupees(value),
                value: rupees(value),
                commission: rupees(commission),
                laga: rupees(laga),
                total: rupees(value + commission + laga),
                cash,
            })
            .collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        // A buyer's day that the books explain: whatever the sale lines, receipts and other
        // entries are, left is the day's debit less its credit, the new balance is the ledger
        // balance, and paid is the cash part plus the receipts.
        #[test]
        fn a_buyers_day_that_the_books_explain_adds_up(
            lines in proptest::collection::vec(
                (1_i64..=1_000_000, 0_i64..=60_000, 0_i64..=20_000, any::<bool>()), 0..12),
            receipts in 0_i64..=2_000_000,
            other_dr in 0_i64..=500_000,
            other_cr in 0_i64..=500_000,
            previous in -3_000_000_i64..=3_000_000,
        ) {
            let built = buyer_lines_of(&lines);
            let total: i64 = lines.iter().map(|l| l.0 + l.1 + l.2).sum();
            let on_spot: i64 = lines.iter().filter(|l| l.3).map(|l| l.0 + l.1 + l.2).sum();
            let dr = total + other_dr;
            let cr = on_spot + receipts + other_cr;
            let mut today = row(rupees(dr), rupees(cr));
            today.bikri_dr = rupees(total);
            today.bikri_cr = rupees(on_spot);
            today.receipts_cr = rupees(receipts);
            let end = rupees(previous + dr - cr);
            let day = buyer_day(1, "", rupees(previous), end, &today, built).unwrap();
            prop_assert_eq!(day.left, rupees(dr - cr));
            prop_assert_eq!(day.new, end);
            prop_assert_eq!(day.paid, rupees(on_spot + receipts));
            prop_assert_eq!(day.other, rupees(other_dr - other_cr));
            prop_assert_eq!(day.total, rupees(total));
            // today's left = total + other − paid.
            prop_assert_eq!(
                day.left,
                rupees(total + (other_dr - other_cr) - (on_spot + receipts))
            );
            prop_assert_eq!(day.has_other, other_dr != 0 || other_cr != 0);
            // The same day, with the sale debit off by a rupee, is refused: never a wrong slip.
            let mut bad = today.clone();
            bad.bikri_dr = rupees(total + 1);
            prop_assert!(buyer_day(1, "", rupees(previous), end, &bad, buyer_lines_of(&lines)).is_err());
            // And a wrong ledger balance at the end is refused too.
            prop_assert!(buyer_day(1, "", rupees(previous), rupees(previous + dr - cr + 1), &today, buyer_lines_of(&lines)).is_err());
        }

        // A seller's day: whatever the lines, the charges, the cash, the payments and the other
        // entries, left is the day's debit less its credit.
        #[test]
        fn a_sellers_day_that_the_books_explain_adds_up(
            lines in proptest::collection::vec((1_i64..=1_000_000, 0_i64..=60_000), 0..8),
            charges in proptest::collection::vec(
                (0_i64..=5_000, 0_i64..=5_000, 0_i64..=500, 0_i64..=500, 0_i64..=3_000, 0_i64..=300_000), 0..4),
            payments in 0_i64..=1_000_000,
            other_dr in 0_i64..=500_000,
            other_cr in 0_i64..=500_000,
            previous in -3_000_000_i64..=3_000_000,
        ) {
            let built: Vec<SellerLine> = lines
                .iter()
                .map(|&(amount, commission)| SellerLine {
                    day: "2026-10-10".to_owned(),
                    account_id: 2,
                    person: String::new(),
                    bikri_id: 1,
                    goods: goods(),
                    qty: 1,
                    weight_grams: None,
                    rate: rupees(amount),
                    amount: rupees(amount),
                    commission: rupees(commission),
                })
                .collect();
            let lots: Vec<LotCharges> = charges
                .iter()
                .map(|&(freight, labour, munshiana, phone, rent, cash)| LotCharges {
                    day: "2026-10-10".to_owned(),
                    account_id: 2,
                    person: String::new(),
                    freight: rupees(freight),
                    labour: rupees(labour),
                    munshiana: rupees(munshiana),
                    phone: rupees(phone),
                    store_rent: rupees(rent),
                    cash: rupees(cash),
                })
                .collect();
            let gross: i64 = lines.iter().map(|l| l.0).sum();
            let commission: i64 = lines.iter().map(|l| l.1).sum();
            let deductions: i64 = charges.iter().map(|c| c.0 + c.1 + c.2 + c.3 + c.4).sum();
            let cash: i64 = charges.iter().map(|c| c.5).sum();
            // Debit-positive: the sale is the deductions less the gross.
            let sale = commission + deductions - gross;
            // The بکری postings: debits are the cash and a negative net, credits a positive net.
            let bikri_dr = cash + sale.max(0);
            let bikri_cr = (-sale).max(0);
            let dr = bikri_dr + payments + other_dr;
            let cr = bikri_cr + other_cr;
            let mut today = row(rupees(dr), rupees(cr));
            today.buyer = false;
            today.bikri_dr = rupees(bikri_dr);
            today.bikri_cr = rupees(bikri_cr);
            today.payments_dr = rupees(payments);
            let end = rupees(previous + dr - cr);
            let day = seller_day(2, "", rupees(previous), end, &today, built.clone(), &lots).unwrap();
            prop_assert_eq!(day.sale, rupees(sale));
            prop_assert_eq!(day.left, rupees(dr - cr));
            prop_assert_eq!(day.new, end);
            prop_assert_eq!(day.paid, rupees(cash + payments));
            // today's left = sale + other + paid.
            prop_assert_eq!(
                day.left,
                rupees(sale + (other_dr - other_cr) + cash + payments)
            );
            // The same day with the cash debit off by a rupee is refused.
            let mut bad = today.clone();
            bad.bikri_dr = rupees(bikri_dr + 1);
            prop_assert!(seller_day(2, "", rupees(previous), end, &bad, built, &lots).is_err());
        }
    }

    #[test]
    fn a_line_whose_total_is_not_its_parts_is_refused() {
        let mut lines = buyer_lines_of(&[(12_000, 720, 160, false)]);
        lines[0].total = rupees(12_000);
        let mut today = row(rupees(12_000), Paisa::ZERO);
        today.bikri_dr = rupees(12_000);
        let result = buyer_day(1, "", Paisa::ZERO, rupees(12_000), &today, lines);
        assert!(result.is_err());
    }
}
