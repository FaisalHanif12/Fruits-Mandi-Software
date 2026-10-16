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
    PaidState, PukhtaPeopleInput, PukhtaPerson, PukhtaRegisterInput, PukhtaRegisterRow, PukhtaSide,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::money::Paisa;
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
    let everyone = ctx
        .db
        .read(|conn| repo::pukhta_people(conn, input.side).map_err(ServiceError::from))?;
    Ok(everyone
        .into_iter()
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
) -> Result<Vec<PukhtaRegisterRow>, ServiceError> {
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
        Ok(out)
    })
}
