//! The one-time people and the register of the پختہ کھاتہ accounts (PRD FR-7.29, FR-13.5,
//! FR-13.6, plan 13). Read only (Code Standards B29).
//!
//! # The register
//!
//! One row for each line of a one-time buyer and each lot of a one-time seller. Paid or due:
//!
//! - A buyer line: the bill's open amount is its debit line less the active rows of
//!   `bill_allocations` (PRD FR-15.4). A نقدی line is pinned to its own cash, so it is paid at
//!   once.
//! - A seller lot: what is still to pay him is worked out when shown (PRD FR-7.59, nothing is
//!   stored): each lot's صافی بکری when it is above 0 (the نقد is already in it), then what is set
//!   against his lots, oldest first, with the same `match_bills` as everything else: his later
//!   payments (a debit line of any entry that is not a بکری posting) and the debit of a lot whose
//!   net is below 0 (he owes the firm what that lot paid him too much). So his open amounts add
//!   up to his balance whenever something is open.
//!
//! Example (PRD 4.5): شاہد buys 10 × 1,200 for cash. His line is 12,800, paid, nothing open.

use std::collections::{BTreeMap, HashMap};

use mandi_db::bikri::saved::{self as repo, BillLine, RegisterLine};
use mandi_db::matching as matching_repo;
use mandi_domain::core::bikri::{
    PaidState, PukhtaPeopleInput, PukhtaPerson, PukhtaRegister, PukhtaRegisterInput,
    PukhtaRegisterRow, PukhtaRegisterTotals, PukhtaSide,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::paging::{MonthSubtotal, SortDirection};
use mandi_domain::core::text;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::matching::{Open, match_bills};

use super::shared::data;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};

/// The most people the merged list shows.
const PEOPLE_LIMIT: usize = 10;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// The digits of a typed text (a phone search ignores `-` and spaces).
fn digits(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

/// True if the typed text is part of the person's names or phone (PRD FR-7.40 style: both
/// scripts, letter variants unified, close English spellings by the phonetic key). Empty text
/// matches everybody.
fn matches(text_typed: &str, name_ur: &str, name_en: &str, phone: &str) -> bool {
    let typed = text_typed.trim();
    if typed.is_empty() {
        return true;
    }
    let ur = text::search_ur(typed);
    if !ur.is_empty() && text::search_ur(name_ur).contains(&ur) {
        return true;
    }
    let en = text::search_en(typed);
    if !en.is_empty() && text::search_en(name_en).contains(&en) {
        return true;
    }
    let phonetic = text::search_phonetic(typed);
    if !phonetic.is_empty() && text::search_phonetic(name_en).contains(&phonetic) {
        return true;
    }
    let typed_digits = digits(typed);
    !typed_digits.is_empty() && phone.contains(&typed_digits)
}

/// One-time people already recorded on a side, newest first, at most 10 (PRD FR-7.29). The
/// names and phone are as he was last typed; picking him reuses them, so his lines keep one
/// person key.
///
/// # Errors
/// `ServiceError::Db`.
pub fn search_people(
    ctx: &Context,
    input: &PukhtaPeopleInput,
) -> Result<Vec<PukhtaPerson>, ServiceError> {
    let include_dead = input.include_dead.unwrap_or(false);
    let everyone = ctx.db.read(|conn| {
        let mut people = repo::pukhta_people(conn, input.side)?;
        // A person in the dead amount is marked by his key on the account of the side (PRD
        // FR-10.10); without `include_dead` he is not listed at all.
        let code = match input.side {
            PukhtaSide::Buyer => mandi_domain::core::codes::PUKHTA_BUYER_CODE,
            PukhtaSide::Seller => mandi_domain::core::codes::PUKHTA_SELLER_CODE,
        };
        if let Some(account) = mandi_db::accounts::id_by_code(conn, code)? {
            let dead = mandi_db::dead::dead_set(conn, None)?;
            if !dead.is_empty() {
                for person in &mut people {
                    person.is_dead = dead.has(account, Some(&person.person_key));
                }
            }
        }
        Ok::<_, ServiceError>(people)
    })?;
    Ok(everyone
        .into_iter()
        .filter(|p| include_dead || !p.is_dead)
        .filter(|p| matches(&input.text, &p.name_ur, &p.name_en, &p.phone))
        .take(PEOPLE_LIMIT)
        .collect())
}

/// A payment against a one-time seller's lots, with where it stands in the order of the key.
struct Payment {
    entry_date: String,
    entry_id: i64,
    open: Open,
}

/// What is open on each seller lot (see the module text), by lot id.
fn seller_open(
    conn: &mandi_db::rusqlite::Connection,
    lines: &[RegisterLine],
) -> Result<HashMap<i64, Paisa>, ServiceError> {
    let mut accounts: Vec<i64> = lines.iter().map(|l| l.account_id).collect();
    accounts.sort_unstable();
    accounts.dedup();
    let mut open_of: HashMap<i64, Paisa> = HashMap::new();
    for account_id in accounts {
        let live = matching_repo::live_lines(conn, account_id)?;
        // Each person's lots, and what is set against them: his later payments and the debit
        // of a lot whose net is below 0 (he owes the firm what the lot paid him too much).
        let mut people: BTreeMap<&str, (Vec<Open>, Vec<Payment>)> = BTreeMap::new();
        for line in lines.iter().filter(|l| l.account_id == account_id) {
            let net = paisa(line.seller_net_paisa)?;
            let entry = people.entry(line.person_key.as_str()).or_default();
            if line.cancelled {
                entry.0.push(Open {
                    line_id: line.row_id,
                    open: Paisa::ZERO,
                });
            } else if net.is_negative() {
                entry.0.push(Open {
                    line_id: line.row_id,
                    open: Paisa::ZERO,
                });
                entry.1.push(Payment {
                    entry_date: line.entry_date.clone(),
                    entry_id: line.journal_entry_id,
                    open: Open {
                        line_id: line.row_id,
                        open: net.checked_neg().map_err(money_error)?,
                    },
                });
            } else {
                entry.0.push(Open {
                    line_id: line.row_id,
                    open: net,
                });
            }
        }
        for line in &live {
            let Some(key) = line.person_key.as_deref() else {
                continue;
            };
            let paid_later = line.dr_paisa > 0
                && !line.is_advance
                && !matches!(line.posting_type.as_str(), "bikri_first" | "bikri_later");
            if paid_later && let Some(entry) = people.get_mut(key) {
                entry.1.push(Payment {
                    entry_date: line.entry_date.clone(),
                    entry_id: line.entry_id,
                    open: Open {
                        line_id: line.line_id,
                        open: paisa(line.dr_paisa)?,
                    },
                });
            }
        }
        for (lots, payments) in people.values_mut() {
            payments.sort_by(|a, b| {
                (a.entry_date.as_str(), a.entry_id, a.open.line_id).cmp(&(
                    b.entry_date.as_str(),
                    b.entry_id,
                    b.open.line_id,
                ))
            });
            let ordered: Vec<Open> = payments.iter().map(|p| p.open).collect();
            let rows = match_bills(lots, &ordered).map_err(money_error)?;
            for lot in lots.iter() {
                let used = rows
                    .iter()
                    .filter(|r| r.bill_line_id == lot.line_id)
                    .try_fold(Paisa::ZERO, |sum, r| sum.checked_add(r.amount))
                    .map_err(money_error)?;
                open_of.insert(
                    lot.line_id,
                    lot.open.checked_sub(used).map_err(money_error)?,
                );
            }
        }
    }
    Ok(open_of)
}

/// What is open on each buyer line, by sale id: the debit line less its active matching rows.
fn buyer_open(
    bills: &[BillLine],
    lines: &[RegisterLine],
) -> Result<HashMap<i64, Paisa>, ServiceError> {
    // The bill lines of one posting and person, in line order, line up with its sale lines in
    // order of saving (the builder writes the buyer debits in lot and line order).
    let mut by_group: BTreeMap<(i64, i64, &str), Vec<&BillLine>> = BTreeMap::new();
    for bill in bills {
        by_group
            .entry((bill.entry_id, bill.account_id, bill.person_key.as_str()))
            .or_default()
            .push(bill);
    }
    let mut sales_by_group: BTreeMap<(i64, i64, &str), Vec<&RegisterLine>> = BTreeMap::new();
    for line in lines {
        sales_by_group
            .entry((
                line.journal_entry_id,
                line.account_id,
                line.person_key.as_str(),
            ))
            .or_default()
            .push(line);
    }
    let mut out = HashMap::new();
    for (group, sales) in sales_by_group {
        let bills = by_group.get(&group).map_or(&[][..], Vec::as_slice);
        if bills.len() != sales.len() {
            return Err(data(
                "the sale lines of a one-time buyer and his bill lines do not line up",
            ));
        }
        for (sale, bill) in sales.iter().zip(bills) {
            let open = paisa(bill.amount_paisa)?
                .checked_sub(paisa(bill.matched_paisa)?)
                .map_err(money_error)?;
            out.insert(sale.row_id, open);
        }
    }
    Ok(out)
}

/// The register of the پختہ کھاتہ lines (PRD FR-13.6, R9), oldest first.
///
/// # Errors
/// The date codes on `from` and `to`; `ServiceError::Db`.
pub fn register(
    ctx: &Context,
    input: &PukhtaRegisterInput,
) -> Result<PukhtaRegister, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let bound = |errors: &mut FieldErrors, field: &str, text: &str| {
        if text.trim().is_empty() {
            None
        } else {
            errors
                .take(validate::date(field, text, today))
                .map(BusinessDate::to_iso)
        }
    };
    let from = bound(&mut errors, "from", &input.from);
    let to = bound(&mut errors, "to", &input.to);
    errors.finish()?;
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    if let Some(sort) = &input.sort {
        sort.check(&SORT_COLUMNS)
            .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    }
    let want = |side: PukhtaSide| input.side.is_none_or(|s| s == side);
    ctx.db.read(|conn| {
        let mut rows: Vec<(RegisterLine, Paisa)> = Vec::new();
        if want(PukhtaSide::Buyer) {
            let lines = repo::register_buyer_lines(conn, from.as_deref(), to.as_deref())?;
            let bills = repo::pukhta_bill_lines(conn)?;
            let open = buyer_open(&bills, &lines)?;
            for line in lines {
                let left = open.get(&line.row_id).copied().unwrap_or(Paisa::ZERO);
                rows.push((line, left));
            }
        }
        if want(PukhtaSide::Seller) {
            let lines = repo::register_seller_lines(conn, from.as_deref(), to.as_deref())?;
            // The open amounts count every lot of the person, not only those in the range.
            let all = repo::register_seller_lines(conn, None, None)?;
            let open = seller_open(conn, &all)?;
            for line in lines {
                let left = open.get(&line.row_id).copied().unwrap_or(Paisa::ZERO);
                rows.push((line, left));
            }
        }
        rows.sort_by(|a, b| {
            (a.0.entry_date.as_str(), a.0.bikri_number, a.0.row_id).cmp(&(
                b.0.entry_date.as_str(),
                b.0.bikri_number,
                b.0.row_id,
            ))
        });
        let name = input
            .name
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty());
        let phone = input.phone.as_deref().map(digits).filter(|p| !p.is_empty());
        let search = input
            .search
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty());
        let mut out = Vec::with_capacity(rows.len());
        for (line, open) in rows {
            if name.is_some_and(|n| !matches(n, &line.name_ur, &line.name_en, "")) {
                continue;
            }
            if phone
                .as_ref()
                .is_some_and(|p| !line.phone.contains(p.as_str()))
            {
                continue;
            }
            if search.is_some_and(|t| {
                !(matches(t, &line.name_ur, &line.name_en, &line.phone)
                    || matches(t, &line.item.name_ur, &line.item.name_en, "")
                    || line.item.code.to_lowercase().starts_with(&t.to_lowercase())
                    || t.parse::<i64>().is_ok_and(|n| n == line.bikri_number))
            }) {
                continue;
            }
            // Lines of cancelled بکری step aside unless asked for (plan 38 R5).
            if line.cancelled && !input.show_finished {
                continue;
            }
            let open = if line.cancelled { Paisa::ZERO } else { open };
            let paid = !line.cancelled && open == Paisa::ZERO;
            let keep = match input.state {
                None => true,
                Some(PaidState::Paid) => paid,
                Some(PaidState::Due) => !line.cancelled && !paid,
            };
            if !keep {
                continue;
            }
            out.push(PukhtaRegisterRow {
                entry_date: line.entry_date,
                bikri_id: line.bikri_id,
                bikri_number: line.bikri_number,
                side: line.side,
                name_ur: line.name_ur,
                name_en: line.name_en,
                phone: line.phone,
                person_key: line.person_key,
                item: line.item,
                qty: line.qty,
                amount: line.amount,
                commission: line.commission,
                paid,
                open,
                cancelled: line.cancelled,
            });
        }
        // The totals are over every matching line, cancelled ones left out; the page is cut
        // after (PRD FR-19.11, B31). With no side chosen the lines of buyers and sellers are
        // mixed, so there is no total (see `PukhtaRegisterTotals`).
        let mut qty = 0_i64;
        let (mut amount, mut commission, mut open) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        for row in out.iter().filter(|row| !row.cancelled) {
            qty = qty
                .checked_add(row.qty)
                .ok_or_else(|| ServiceError::reject(ErrorCode::ErrOverflow))?;
            amount = amount.checked_add(row.amount).map_err(money_error)?;
            commission = commission
                .checked_add(row.commission)
                .map_err(money_error)?;
            open = open.checked_add(row.open).map_err(money_error)?;
        }
        let one_side = input.side.is_some();
        let totals = PukhtaRegisterTotals {
            qty: one_side.then_some(qty),
            amount: one_side.then_some(amount),
            commission: one_side.then_some(commission),
            open: one_side.then_some(open),
        };
        let total_count =
            i64::try_from(out.len()).map_err(|_| ServiceError::Data("too many rows".to_owned()))?;
        // The sort is of the whole filter, before the page is cut. The sort is stable, so equal
        // rows keep the order by day and number.
        if let Some(sort) = &input.sort {
            sort_rows(&mut out, &sort.column, sort.direction);
        }
        // Month headings with their subtotals, over the whole filter (plan 38 R4).
        let in_date_order = input.sort.as_ref().is_none_or(|s| s.column == "entry_date");
        let newest_first = input
            .sort
            .as_ref()
            .is_some_and(|s| s.column == "entry_date" && s.direction == SortDirection::Desc);
        let months = if input.group_by_month && in_date_order {
            months_of(&out, one_side, newest_first)?
        } else {
            Vec::new()
        };
        Ok(PukhtaRegister {
            rows: crate::page::slice(out, input.offset, input.limit),
            total_count,
            totals,
            months,
        })
    })
}

/// The columns the register can be sorted by: the names of the fields of the row.
const SORT_COLUMNS: [&str; 12] = [
    "entry_date",
    "bikri_number",
    "side",
    "name_ur",
    "name_en",
    "phone",
    "item",
    "qty",
    "amount",
    "commission",
    "paid",
    "open",
];

/// Sorts the whole register by a column (checked by [`SORT_COLUMNS`]). A stable sort.
fn sort_rows(rows: &mut [PukhtaRegisterRow], column: &str, direction: SortDirection) {
    use std::cmp::Ordering;
    let cmp: fn(&PukhtaRegisterRow, &PukhtaRegisterRow) -> Ordering = match column {
        "entry_date" => |a, b| {
            (a.entry_date.as_str(), a.bikri_number).cmp(&(b.entry_date.as_str(), b.bikri_number))
        },
        "bikri_number" => |a, b| a.bikri_number.cmp(&b.bikri_number),
        "side" => |a, b| (a.side as u8).cmp(&(b.side as u8)),
        "name_ur" => |a, b| crate::page::urdu_compare(&a.name_ur, &b.name_ur),
        "name_en" => |a, b| a.name_en.to_lowercase().cmp(&b.name_en.to_lowercase()),
        "phone" => |a, b| a.phone.cmp(&b.phone),
        "item" => |a, b| a.item.code.cmp(&b.item.code),
        "qty" => |a, b| a.qty.cmp(&b.qty),
        "amount" => |a, b| a.amount.cmp(&b.amount),
        "commission" => |a, b| a.commission.cmp(&b.commission),
        "paid" => |a, b| a.paid.cmp(&b.paid),
        "open" => |a, b| a.open.cmp(&b.open),
        _ => |_, _| Ordering::Equal,
    };
    rows.sort_by(|a, b| direction.apply(cmp(a, b)));
}

/// The count and subtotals of each month of the register rows (the cancelled lines count in the
/// month and add no money). The money figures are given only when one side is chosen, like the
/// totals.
fn months_of(
    rows: &[PukhtaRegisterRow],
    one_side: bool,
    newest_first: bool,
) -> Result<Vec<MonthSubtotal>, ServiceError> {
    #[derive(Default)]
    struct Sums {
        count: i64,
        qty: i64,
        amount: Paisa,
        commission: Paisa,
        open: Paisa,
    }
    let overflow = || ServiceError::reject(ErrorCode::ErrOverflow);
    let mut by_month: BTreeMap<&str, Sums> = BTreeMap::new();
    for row in rows {
        let month = row.entry_date.get(..7).unwrap_or(row.entry_date.as_str());
        let sums = by_month.entry(month).or_default();
        sums.count = sums.count.checked_add(1).ok_or_else(overflow)?;
        if !row.cancelled {
            sums.qty = sums.qty.checked_add(row.qty).ok_or_else(overflow)?;
            sums.amount = sums.amount.checked_add(row.amount).map_err(money_error)?;
            sums.commission = sums
                .commission
                .checked_add(row.commission)
                .map_err(money_error)?;
            sums.open = sums.open.checked_add(row.open).map_err(money_error)?;
        }
    }
    let mut months: Vec<MonthSubtotal> = by_month
        .into_iter()
        .map(|(month, sums)| MonthSubtotal {
            month: month.to_owned(),
            count: sums.count,
            figures: if one_side {
                vec![
                    MonthSubtotal::count_of("qty", sums.qty),
                    MonthSubtotal::money("amount", sums.amount.as_paisa()),
                    MonthSubtotal::money("commission", sums.commission.as_paisa()),
                    MonthSubtotal::money("open", sums.open.as_paisa()),
                ]
            } else {
                Vec::new()
            },
        })
        .collect();
    if newest_first {
        months.reverse();
    }
    Ok(months)
}
