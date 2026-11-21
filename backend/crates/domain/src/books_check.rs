//! The money tables compared with the posted journal (NFR-SEC-6 v1.21, plan 80 Q-T3-3), as pure
//! functions. No database, no clock. The daily books check (`mandi-services::health`) reads the
//! sums with SQL and gives them to these functions, which work the money out with checked
//! `Paisa` operations and say what differs.
//!
//! Why: the journal and the audit log are in keyed hash chains, but the sale lines, the lot totals,
//! the partner share rows, the stock moves and the analytics facts are not. NFR-SEC-6 says every
//! money table can be rebuilt and compared from the journal, so a consistent edit of one of them
//! outside the app is found and warned about like a broken chain.
//!
//! Three comparisons live here, because they work money out:
//!
//! 1. [`compare_postings`]: the sale lines and lots of each بکری posting against that entry's
//!    journal lines, account by account. The net of an account in an entry is
//!    `Σ debit − Σ credit` of its lines.
//! 2. [`lot_difference`]: each lot's stored gross, deductions and net against its live sale lines and
//!    its charges (PRD C6, C7).
//! 3. [`share_rows_difference`]: the partner share rows of an entry against the split of its income and
//!    shop expense lines by the shares of its date, through [`split_posting`], the very function
//!    the save uses (PRD C9 to C11, FR-6.5).
//!
//! A reason never holds an amount (NFR-OBS-2): it names the table, the record and the kind of
//! difference.
//!
//! Worked numbers, PRD 4.4 (Event 1, one posting): buyer accounts 12,880 and 18,684 debit;
//! the seller رحیم has credit 29,400 − 1,764 − (3,000 + 250 + 50 + 20) = 24,316 and 5,000 cash
//! debit, so his net is −19,316; commissions 1,764 and 1,764, لاگا 400, فون 20, مزدوری 250, کرایہ
//! 3,000, منشیانہ 50 are credits; روکڑ has net −5,000. Event 2 (a چکتا buyer paid in full): account
//! 0030 net 12,800 − 12,800 = 0 and روکڑ net +12,800.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use mandi_core::ids::{AccountId, EntryId, PartnerId};
use mandi_core::money::{Bps, Grams, MoneyError, Paisa, weight_value};

use crate::partners::{ShareBases, ShareKind, ShareRowDraft, split_posting};

/// A difference between a table and the journal. The text names the table, the record and what
/// differs. It holds no amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference {
    /// The table that does not match the journal.
    pub table: &'static str,
    /// The text of the difference, for example `journal entry 5: account 12 differs`.
    pub what: String,
}

impl Difference {
    /// A difference in `table`, described by `what`.
    pub fn new(table: &'static str, what: impl Into<String>) -> Difference {
        Difference {
            table,
            what: what.into(),
        }
    }
}

impl std::fmt::Display for Difference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.table, self.what)
    }
}

/// The system accounts a بکری posting writes to (PRD FR-10.4, FR-12.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemAccounts {
    /// روکڑ 01010001.
    pub cash: AccountId,
    /// کمیشن آمدن (بیوپاری) 04010001.
    pub seller_commission: AccountId,
    /// کمیشن آمدن (گاہک) 04010002.
    pub buyer_commission: AccountId,
    /// لاگا آمدن 04010003.
    pub laga: AccountId,
    /// فون آمدن 04010004.
    pub phone: AccountId,
    /// ریٹ فرق 04010005.
    pub rate_diff: AccountId,
    /// مزدوری کھاتہ 02010001.
    pub labour: AccountId,
    /// کرایہ کھاتہ 02010002.
    pub freight: AccountId,
    /// کرایہ سٹور کھاتہ 02010003.
    pub store_rent: AccountId,
    /// منشیانہ کھاتہ 02010004.
    pub munshiana: AccountId,
}

/// A بکری posting entry (`bikri_first` or `bikri_later`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostingEntry {
    /// The journal entry.
    pub entry_id: EntryId,
    /// True when no reversal of it exists. A live posting is compared on every account. A reversed
    /// one is compared on its sale-level accounts only, because the lot rows are edited in place by
    /// later edits and no longer say what the old posting charged.
    pub live: bool,
}

/// The sums of the sale lines of one posting that fall on one buyer account (a چکتا buyer is the
/// account 0030, all his persons together).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuyerSums {
    /// The posting.
    pub entry_id: EntryId,
    /// The buyer's account.
    pub account_id: AccountId,
    /// Σ of what the buyers pay (value + commission + لاگا).
    pub buyer_total: Paisa,
    /// Σ of what they paid on the spot (PRD FR-12.44).
    pub paid: Paisa,
}

/// The sums of the sale lines of one posting, for the income accounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaleSums {
    /// The posting.
    pub entry_id: EntryId,
    /// Σ seller commission.
    pub seller_comm: Paisa,
    /// Σ buyer commission.
    pub buyer_comm: Paisa,
    /// Σ لاگا.
    pub laga: Paisa,
    /// Σ rate difference (signed).
    pub rate_diff: Paisa,
}

/// One lot of one LIVE posting: the sums of its sale lines in that posting and the charges stored
/// on the lot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LotPosting {
    /// The posting.
    pub entry_id: EntryId,
    /// The lot (a record number for the reasons).
    pub lot_id: i64,
    /// The lot's seller account.
    pub seller_account_id: AccountId,
    /// True for a trip lot (the seller is اپنا مال): its seller credit is posted to the trip
    /// partners' accounts instead (PRD FR-12.26, C25).
    pub trip: bool,
    /// True when this posting is the lot's first, which carries its charges (PRD FR-12.17).
    pub first: bool,
    /// Σ seller amount of the lot's lines in this posting.
    pub seller_amount: Paisa,
    /// Σ seller commission of those lines.
    pub seller_comm: Paisa,
    /// Freight stored on the lot.
    pub freight: Paisa,
    /// True when the firm paid the freight (only then it is deducted and posted, PRD E6).
    pub freight_by_firm: bool,
    /// مزدوری stored on the lot.
    pub labour: Paisa,
    /// منشیانہ stored on the lot.
    pub munshiana: Paisa,
    /// کرایہ سٹور stored on the lot.
    pub store_rent: Paisa,
    /// فون stored on the lot.
    pub phone: Paisa,
    /// نقد stored on the lot.
    pub cash_paid: Paisa,
}

/// One trip partner's part of a trip lot's seller credit in one posting (`lot_credit_parts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TripPart {
    /// The posting.
    pub entry_id: EntryId,
    /// The lot.
    pub lot_id: i64,
    /// The partner's garden, trip and fruit account (0604).
    pub account_id: AccountId,
    /// His part of the seller credit (can be negative).
    pub amount: Paisa,
}

/// Σ debit and Σ credit of the lines of one entry on one account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSums {
    /// The entry.
    pub entry_id: EntryId,
    /// The account.
    pub account_id: AccountId,
    /// Σ debit.
    pub debit: Paisa,
    /// Σ credit.
    pub credit: Paisa,
}

/// Everything [`compare_postings`] needs, read from the tables.
#[derive(Debug, Clone, Copy)]
pub struct PostingInputs<'a> {
    /// Every posting entry.
    pub entries: &'a [PostingEntry],
    /// Sale sums per posting and buyer account.
    pub buyers: &'a [BuyerSums],
    /// Sale sums per posting.
    pub sales: &'a [SaleSums],
    /// Lots of the live postings.
    pub lots: &'a [LotPosting],
    /// Trip partners' parts of the live postings.
    pub parts: &'a [TripPart],
    /// The system accounts.
    pub system: SystemAccounts,
    /// The lines of every posting entry, per account.
    pub lines: &'a [LineSums],
    /// The accounts of the linked money mirror (PRD FR-16.6, FR-16.13): each partner's 0605 account
    /// and the contra 03070001. A بکری of a lot whose seller is a الگ کھاتہ linked to the shop or a
    /// partner gets mirror lines on them, worked out from the link of that day, so they are not
    /// compared line by line. They must add up to nothing in each entry.
    pub mirror: &'a [AccountId],
}

type Nets = BTreeMap<(EntryId, AccountId), Paisa>;

fn add_to(
    nets: &mut Nets,
    entry: EntryId,
    account: AccountId,
    by: Paisa,
) -> Result<(), MoneyError> {
    let slot = nets.entry((entry, account)).or_insert(Paisa::ZERO);
    *slot = slot.checked_add(by)?;
    Ok(())
}

/// What the posting must credit the seller of one lot (PRD C8): the seller amounts of this posting
/// less the seller commission of this posting less the lot charges, which only the first posting
/// carries. نقد is not in it: it is a separate debit line.
///
/// Example (PRD 4.4): 29,400 − 1,764 − (3,000 + 250 + 50 + 0 + 20) = 24,316.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn lot_credit(lot: &LotPosting) -> Result<Paisa, MoneyError> {
    let mut credit = lot.seller_amount.checked_sub(lot.seller_comm)?;
    if lot.first {
        let freight = if lot.freight_by_firm {
            lot.freight
        } else {
            Paisa::ZERO
        };
        for charge in [
            freight,
            lot.labour,
            lot.munshiana,
            lot.store_rent,
            lot.phone,
        ] {
            credit = credit.checked_sub(charge)?;
        }
    }
    Ok(credit)
}

/// The net (debit less credit) each account must have in each posting, from the sale lines and lots.
/// Every posting has the four sale-level income accounts, even at zero, so a line that should not
/// be there is found.
fn expected_nets(
    inputs: &PostingInputs<'_>,
    live_set: &HashSet<EntryId>,
) -> Result<Nets, MoneyError> {
    let system = &inputs.system;
    let mut nets = Nets::new();
    for entry in inputs.entries {
        for account in [
            system.seller_commission,
            system.buyer_commission,
            system.laga,
            system.rate_diff,
        ] {
            add_to(&mut nets, entry.entry_id, account, Paisa::ZERO)?;
        }
    }
    for buyer in inputs.buyers {
        // The bill is a debit, what was paid on the spot is a credit of the same account.
        let left = buyer.buyer_total.checked_sub(buyer.paid)?;
        add_to(&mut nets, buyer.entry_id, buyer.account_id, left)?;
    }
    for sale in inputs.sales {
        for (account, amount) in [
            (system.seller_commission, sale.seller_comm),
            (system.buyer_commission, sale.buyer_comm),
            (system.laga, sale.laga),
            (system.rate_diff, sale.rate_diff),
        ] {
            add_to(&mut nets, sale.entry_id, account, amount.checked_neg()?)?;
        }
    }
    // The money buyers paid on the spot comes into cash, and نقد leaves it. Lots only exist for
    // the live postings, so the cash of a reversed posting is not compared.
    for buyer in inputs.buyers {
        if live_set.contains(&buyer.entry_id) {
            add_to(&mut nets, buyer.entry_id, system.cash, buyer.paid)?;
        }
    }
    for lot in inputs.lots {
        let credit = lot_credit(lot)?;
        if lot.trip {
            // Posted to the trip partners instead of a seller (PRD FR-12.26).
        } else {
            add_to(
                &mut nets,
                lot.entry_id,
                lot.seller_account_id,
                credit.checked_neg()?,
            )?;
        }
        if lot.first {
            add_to(
                &mut nets,
                lot.entry_id,
                lot.seller_account_id,
                lot.cash_paid,
            )?;
            add_to(
                &mut nets,
                lot.entry_id,
                system.cash,
                lot.cash_paid.checked_neg()?,
            )?;
            let freight = if lot.freight_by_firm {
                lot.freight
            } else {
                Paisa::ZERO
            };
            for (account, amount) in [
                (system.phone, lot.phone),
                (system.labour, lot.labour),
                (system.freight, freight),
                (system.store_rent, lot.store_rent),
                (system.munshiana, lot.munshiana),
            ] {
                add_to(&mut nets, lot.entry_id, account, amount.checked_neg()?)?;
            }
        }
    }
    for part in inputs.parts {
        add_to(
            &mut nets,
            part.entry_id,
            part.account_id,
            part.amount.checked_neg()?,
        )?;
    }
    Ok(nets)
}

/// Compares the sale lines and lots of every بکری posting with the journal lines of that posting
/// (PRD FR-12.9). Returns one difference for each account of each posting that does not match, and
/// one for a trip lot whose partners' parts do not add up to its seller credit (PRD C25).
///
/// A live posting is compared on every account it has lines on, so an extra, missing or changed
/// line is found. A reversed posting is compared on its buyer accounts and the four sale-level
/// income accounts only.
///
/// # Errors
/// `MoneyError::Overflow` if a sum does not fit.
pub fn compare_postings(inputs: &PostingInputs<'_>) -> Result<Vec<Difference>, MoneyError> {
    const TABLE: &str = "bikri_sales";
    let live_set: HashSet<EntryId> = inputs
        .entries
        .iter()
        .filter(|e| e.live)
        .map(|e| e.entry_id)
        .collect();
    let expected = expected_nets(inputs, &live_set)?;
    let mirror: HashSet<AccountId> = inputs.mirror.iter().copied().collect();
    let mut actual = Nets::new();
    let mut mirror_nets: BTreeMap<EntryId, Paisa> = BTreeMap::new();
    for line in inputs.lines {
        let net = line.debit.checked_sub(line.credit)?;
        if mirror.contains(&line.account_id) {
            let slot = mirror_nets.entry(line.entry_id).or_insert(Paisa::ZERO);
            *slot = slot.checked_add(net)?;
        } else {
            add_to(&mut actual, line.entry_id, line.account_id, net)?;
        }
    }
    let mut differences = Vec::new();
    // A live posting: every account of either side. A reversed one: the accounts the sale lines
    // name.
    let mut keys: BTreeSet<(EntryId, AccountId)> = expected.keys().copied().collect();
    for key in actual.keys() {
        if live_set.contains(&key.0) {
            keys.insert(*key);
        }
    }
    for key in keys {
        let (entry_id, account_id) = key;
        let want = expected.get(&key).copied().unwrap_or(Paisa::ZERO);
        let have = actual.get(&key).copied().unwrap_or(Paisa::ZERO);
        if want != have {
            differences.push(Difference::new(
                TABLE,
                format!(
                    "journal entry {}: account {} does not match the sale lines and lots",
                    entry_id.get(),
                    account_id.get()
                ),
            ));
        }
    }
    // The mirror lines of a linked khata (FR-16.6) are a pair of equal and opposite amounts.
    for (entry_id, net) in mirror_nets {
        if net != Paisa::ZERO {
            differences.push(Difference::new(
                TABLE,
                format!(
                    "journal entry {}: the linked money lines do not add up to nothing",
                    entry_id.get()
                ),
            ));
        }
    }
    // A trip lot's seller credit is split between the trip partners (C25): the parts add up to it.
    let mut part_totals: BTreeMap<(EntryId, i64), Paisa> = BTreeMap::new();
    for part in inputs.parts {
        let slot = part_totals
            .entry((part.entry_id, part.lot_id))
            .or_insert(Paisa::ZERO);
        *slot = slot.checked_add(part.amount)?;
    }
    for lot in inputs.lots.iter().filter(|l| l.trip) {
        let have = part_totals
            .get(&(lot.entry_id, lot.lot_id))
            .copied()
            .unwrap_or(Paisa::ZERO);
        if have != lot_credit(lot)? {
            differences.push(Difference::new(
                "lot_credit_parts",
                format!(
                    "lot {} in journal entry {}: the partners' parts do not add up to the seller credit",
                    lot.lot_id,
                    lot.entry_id.get()
                ),
            ));
        }
    }
    Ok(differences)
}

/// The sale lines of one posting on the چکتا کھاتہ, by person: what the line bills and what was
/// paid on the spot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonSale {
    /// The posting.
    pub entry_id: EntryId,
    /// The person key (empty when a line has none).
    pub person: String,
    /// Σ what the buyer pays.
    pub buyer_total: Paisa,
    /// Σ paid on the spot.
    pub paid: Paisa,
}

/// The journal lines of one posting on the چکتا کھاتہ, by person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonLines {
    /// The posting.
    pub entry_id: EntryId,
    /// The person key (empty when a line has none).
    pub person: String,
    /// Σ debit.
    pub debit: Paisa,
    /// Σ credit.
    pub credit: Paisa,
}

/// Compares, for each posting and each person of the چکتا کھاتہ, what his sale lines bill with the
/// debit of his journal lines, and what he paid on the spot with their credit. The account's total is
/// compared by [`compare_postings`]; this finds a line moved from one person to another, whose
/// account total does not change (PRD FR-13.5).
///
/// The debit and the credit are compared one by one, not their net: a person who paid the whole
/// bill on the spot has a net of 0 whoever he is, so a net would never show that his lines were
/// moved to another person (the check of NFR-SEC-6 must find it).
///
/// Example: شاہد buys 12,800 and pays 12,800: his lines are 12,800 debit and 12,800 credit. Moved to
/// ناصر, the sale line says ناصر while the lines still say شاہد: two differences.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn compare_persons(
    sales: &[PersonSale],
    lines: &[PersonLines],
) -> Result<Vec<Difference>, MoneyError> {
    let mut want: BTreeMap<(EntryId, &str), (Paisa, Paisa)> = BTreeMap::new();
    for sale in sales {
        let slot = want
            .entry((sale.entry_id, sale.person.as_str()))
            .or_insert((Paisa::ZERO, Paisa::ZERO));
        *slot = (
            slot.0.checked_add(sale.buyer_total)?,
            slot.1.checked_add(sale.paid)?,
        );
    }
    let mut have: BTreeMap<(EntryId, &str), (Paisa, Paisa)> = BTreeMap::new();
    for line in lines {
        let slot = have
            .entry((line.entry_id, line.person.as_str()))
            .or_insert((Paisa::ZERO, Paisa::ZERO));
        *slot = (
            slot.0.checked_add(line.debit)?,
            slot.1.checked_add(line.credit)?,
        );
    }
    let keys: BTreeSet<(EntryId, &str)> = want.keys().chain(have.keys()).copied().collect();
    let mut differences = Vec::new();
    for key in keys {
        let a = want
            .get(&key)
            .copied()
            .unwrap_or((Paisa::ZERO, Paisa::ZERO));
        let b = have
            .get(&key)
            .copied()
            .unwrap_or((Paisa::ZERO, Paisa::ZERO));
        if a != b {
            differences.push(Difference::new(
                "bikri_sales",
                format!(
                    "journal entry {}: the lines of a چکتا buyer do not match his sale lines",
                    key.0.get()
                ),
            ));
        }
    }
    Ok(differences)
}

/// The amounts of a sale line sold by weight (PRD C1): the value is the weight in kilograms times
/// the rate per kilogram, rounded once to the rupee. Nothing else ties the two together, because
/// the table can only check `qty × rate` for a line sold by نگ. Returns what differs.
///
/// Example: 12.345 kg at 100 a kilogram is 1,234.5, so 1,235.
///
/// # Errors
/// `MoneyError`: a weight of 0 or less, or an overflow.
pub fn weight_line_difference(
    grams: i64,
    seller_rate: Paisa,
    seller_amount: Paisa,
    buyer_rate: Paisa,
    buyer_value: Paisa,
) -> Result<Option<&'static str>, MoneyError> {
    let weight = Grams::new(grams)?;
    if weight_value(weight, seller_rate)? != seller_amount {
        return Ok(Some("seller amount"));
    }
    if weight_value(weight, buyer_rate)? != buyer_value {
        return Ok(Some("buyer value"));
    }
    Ok(None)
}

/// A lot as stored, with the sums of its live sale lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LotRow {
    /// True when the lot's بکری was cancelled. A cancel ends every line and reverses its moves but
    /// keeps the lot's columns as the record of what was cancelled (plan 16), so only the live
    /// lines are compared: there must be none.
    pub cancelled: bool,
    /// Σ qty of the live lines.
    pub lines_qty: i64,
    /// Σ seller amount of the live lines.
    pub lines_gross: Paisa,
    /// Σ seller commission of the live lines.
    pub lines_comm: Paisa,
    /// `qty_sold` as stored.
    pub qty_sold: i64,
    /// `gross_paisa` as stored.
    pub gross: Paisa,
    /// `seller_comm_paisa` as stored.
    pub seller_comm: Paisa,
    /// `deductions_paisa` as stored.
    pub deductions: Paisa,
    /// `net_paisa` as stored.
    pub net: Paisa,
    /// Freight, مزدوری, منشیانہ, نقد, کرایہ سٹور and فون as stored.
    pub freight: Paisa,
    /// True when the firm paid the freight.
    pub freight_by_firm: bool,
    /// مزدوری.
    pub labour: Paisa,
    /// منشیانہ.
    pub munshiana: Paisa,
    /// نقد.
    pub cash_paid: Paisa,
    /// کرایہ سٹور.
    pub store_rent: Paisa,
    /// فون.
    pub phone: Paisa,
}

/// Compares a lot's stored totals with its live sale lines and charges (PRD C6, C7): `qty_sold`,
/// gross and seller commission are the sums of the live lines; deductions are the firm's freight,
/// the commission, مزدوری, منشیانہ, نقد, کرایہ سٹور and فون; net is gross less deductions. Returns the
/// name of the first thing that differs. A cancelled lot keeps its columns, so only its live lines
/// are checked (there are none).
///
/// Example (PRD 4.4): gross 29,400, deductions 10,084, net 19,316.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn lot_difference(lot: &LotRow) -> Result<Option<&'static str>, MoneyError> {
    if lot.cancelled {
        let none =
            lot.lines_qty == 0 && lot.lines_gross == Paisa::ZERO && lot.lines_comm == Paisa::ZERO;
        return Ok((!none).then_some("live sale lines of a cancelled بکری"));
    }
    if lot.qty_sold != lot.lines_qty {
        return Ok(Some("qty_sold"));
    }
    if lot.gross != lot.lines_gross {
        return Ok(Some("gross"));
    }
    if lot.seller_comm != lot.lines_comm {
        return Ok(Some("seller commission"));
    }
    let mut deductions = lot.seller_comm;
    let freight = if lot.freight_by_firm {
        lot.freight
    } else {
        Paisa::ZERO
    };
    for charge in [
        freight,
        lot.labour,
        lot.munshiana,
        lot.cash_paid,
        lot.store_rent,
        lot.phone,
    ] {
        deductions = deductions.checked_add(charge)?;
    }
    if lot.deductions != deductions {
        return Ok(Some("deductions"));
    }
    if lot.net != lot.gross.checked_sub(deductions)? {
        return Ok(Some("net"));
    }
    Ok(None)
}

/// One partner's share on a day, from `partner_shares`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareHistory {
    /// The row id (the row saved last wins on the same day).
    pub id: i64,
    /// The partner.
    pub partner_id: PartnerId,
    /// His share.
    pub share_bps: Bps,
    /// The first day it applies, `YYYY-MM-DD`.
    pub effective_from: String,
    /// When the row was saved. The rows of one `set_shares` call share it.
    pub created_at: String,
}

/// The states of everyone's shares that an entry dated `date` may have been split with (PRD
/// FR-17.2). `partner_shares` only grows, with rising ids, so every split was made with a prefix of
/// it: all the rows with an id up to some `k`. The states are `shares_on(date)` (each partner's
/// latest row on or before `date`, the larger id on the same day) taken over the rows up to each
/// `k`, newest state first. This holds whatever the computer's clock did: a change that takes
/// effect on the day of a posting may be saved before or after it (NFR-REL-11). Two prefixes that
/// give the same shares are one state.
///
/// Example: shares 40 / 30 / 30 from 01-09-26, changed to 50 / 25 / 25 on 10-10-26. For 09-10-26
/// there is one state (40 / 30 / 30); for 10-10-26 there are two, the new one first.
pub fn share_states(history: &[ShareHistory], date: &str) -> Vec<Vec<(PartnerId, Bps)>> {
    let mut applying: Vec<&ShareHistory> = history
        .iter()
        .filter(|h| h.effective_from.as_str() <= date)
        .collect();
    applying.sort_by_key(|h| h.id);
    let state_of = |rows: &[&ShareHistory]| -> Vec<(PartnerId, Bps)> {
        let mut best: BTreeMap<PartnerId, &ShareHistory> = BTreeMap::new();
        for row in rows {
            let better = best.get(&row.partner_id).is_none_or(|old| {
                (row.effective_from.as_str(), row.id) > (old.effective_from.as_str(), old.id)
            });
            if better {
                best.insert(row.partner_id, row);
            }
        }
        best.into_iter()
            .map(|(p, row)| (p, row.share_bps))
            .collect()
    };
    let mut states: Vec<Vec<(PartnerId, Bps)>> = Vec::new();
    // The state after each prefix, then newest first, without a state twice.
    for end in (1..=applying.len()).rev() {
        let state = applying.get(..end).map(state_of).unwrap_or_default();
        if !states.contains(&state) {
            states.push(state);
        }
    }
    states
}

/// A stored partner share row (`partner_share_rows`), as the comparison reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredShare {
    /// The partner.
    pub partner_id: PartnerId,
    /// Income or expense.
    pub kind: ShareKind,
    /// The signed base that was split.
    pub base: Paisa,
    /// His share on the day.
    pub share_bps: Bps,
    /// His signed part.
    pub amount: Paisa,
}

fn sorted_key(row: &StoredShare) -> (u8, i64) {
    let kind = match row.kind {
        ShareKind::Income => 0,
        ShareKind::Expense => 1,
    };
    (kind, row.partner_id.get())
}

fn as_stored(row: &ShareRowDraft) -> StoredShare {
    StoredShare {
        partner_id: row.partner_id,
        kind: row.kind,
        base: row.base,
        share_bps: row.share_bps,
        amount: row.amount,
    }
}

fn same_rows(left: &[StoredShare], right: &[StoredShare]) -> bool {
    let mut left = left.to_vec();
    let mut right = right.to_vec();
    left.sort_by_key(sorted_key);
    right.sort_by_key(sorted_key);
    left == right
}

/// True when the stored share rows of a NEW posting are what the split of its income and shop
/// expense lines gives, under any of the states of its date (see [`share_states`]). A posting with
/// no income and no expense line has no rows. The split is `split_posting`, the function of the
/// save (Code Standards B5).
///
/// Example: shared income 3,948 at 40 / 30 / 30 gives rows 1,580 / 1,184 / 1,184.
pub fn share_rows_match(
    states: &[Vec<(PartnerId, Bps)>],
    bases: ShareBases,
    stored: &[StoredShare],
) -> bool {
    if bases.is_zero() {
        return stored.is_empty();
    }
    states.iter().any(|shares| {
        split_posting(shares, bases)
            .map(|rows| {
                let rows: Vec<StoredShare> = rows.iter().map(as_stored).collect();
                same_rows(&rows, stored)
            })
            .unwrap_or(false)
    })
}

/// True when the stored rows of a REVERSAL are the stored rows of the entry it reverses, with base
/// and amount negated. A reversal copies and never splits again (PRD FR-17.4), so it cancels the
/// original exactly even if the shares changed since.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn reversal_rows_match(
    original: &[StoredShare],
    stored: &[StoredShare],
) -> Result<bool, MoneyError> {
    let mut mirrored = Vec::with_capacity(original.len());
    for row in original {
        mirrored.push(StoredShare {
            base: row.base.checked_neg()?,
            amount: row.amount.checked_neg()?,
            ..*row
        });
    }
    Ok(same_rows(&mirrored, stored))
}

/// Σ `credit − debit` over a list of lines on income accounts and Σ `debit − credit` over shop
/// expense lines: the two bases of PRD C9 and C11, from sums per entry. `income` and `expense` are
/// the (debit, credit) sums of the entry's head 04 and head 05 lines.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn bases_of(income: (Paisa, Paisa), expense: (Paisa, Paisa)) -> Result<ShareBases, MoneyError> {
    Ok(ShareBases {
        income: income.1.checked_sub(income.0)?,
        expense: expense.0.checked_sub(expense.1)?,
    })
}

/// A cache of the states of each date, so the entries of one day share one reading of the history.
#[derive(Debug, Default)]
pub struct ShareStates {
    by_date: HashMap<String, Vec<Vec<(PartnerId, Bps)>>>,
}

impl ShareStates {
    /// The states of `date`, worked out from `history` the first time.
    pub fn of<'a>(
        &'a mut self,
        history: &[ShareHistory],
        date: &str,
    ) -> &'a [Vec<(PartnerId, Bps)>] {
        self.by_date
            .entry(date.to_owned())
            .or_insert_with(|| share_states(history, date))
    }
}

/// The `Paisa` of a stored value, or a text that names the field that is not a whole rupee. Used by
/// the callers that read the tables.
///
/// # Errors
/// The name of the field, when the value is not a whole rupee.
pub fn stored_paisa(value: i64, field: &'static str) -> Result<Paisa, &'static str> {
    Paisa::try_from_paisa(value).map_err(|_| field)
}

#[cfg(test)]
mod tests;
