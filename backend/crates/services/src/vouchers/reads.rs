//! The read-only helpers of the voucher screen (plan 15): a buyer's open bills (PRD FR-7.58), what
//! is payable to a seller (PRD FR-7.59), the advances report (PRD R26 data) and the collection
//! list (PRD FR-15.6). Nothing here writes (Code Standards B29).
//!
//! The payable postings are worked out when shown, from the live lines of the seller and the
//! advance rows that are stored (money agent 8.2a). The workings tie to the ledger: a seller's
//! balance (credit less debit) is `open − advances_open − owes_firm`.

use std::collections::{BTreeMap, HashMap};

use mandi_db::accounts as account_repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::matching as matching_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers as repo;
use mandi_domain::core::accounts::{BalanceSide, RecordRef};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::core::vouchers::{
    AdvancesReport, AdvancesReportInput, AdvancesReportRow, CollectionBuyer, CollectionGroup,
    CollectionList, CollectionListInput, LineSource, MAX_OPEN_ROWS, OpenBill, OpenBills,
    OpenBillsInput, OpenPayables, OpenPayablesInput, PayablePosting,
};
use mandi_domain::payables;

use super::common::{Accepts, Party, resolve_party};
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// A count of rows for the contract (`number`).
fn count_of(count: usize) -> Result<i64, ServiceError> {
    i64::try_from(count).map_err(|_| ServiceError::reject(ErrorCode::ErrOverflow))
}

/// Where a line came from, from the posting type and the kind of record of its entry.
pub(super) fn line_source(posting_type: &str, source_type: &str) -> LineSource {
    match (posting_type, source_type) {
        ("opening", _) => LineSource::Opening,
        ("bikri_first" | "bikri_later", _) | (_, "bikri") => LineSource::Bikri,
        ("payment", _) => LineSource::CashLent,
        ("receipt", _) => LineSource::Receipt,
        ("general", _) => LineSource::General,
        ("purchase", _) => LineSource::Purchase,
        ("fruit_taken", _) | (_, "fruit") => LineSource::Fruit,
        _ => LineSource::Other,
    }
}

/// Finds the party of a read and checks it is a buyer or a seller of the kind the read is for.
fn party_for_read(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
    field: &str,
    buyer: bool,
) -> Result<Party, ServiceError> {
    let mut errors = validate::FieldErrors::default();
    let party = resolve_party(
        conn,
        &mut errors,
        account_id,
        person_key,
        Accepts {
            buyer,
            seller: !buyer,
            payable: false,
            allow_inactive: true,
            refusal: if buyer {
                ErrorCode::ErrNotABuyer
            } else {
                ErrorCode::ErrNotASeller
            },
        },
    )?;
    // The checks name the fields of the voucher form; a read names its own field.
    let rename = |mut list: Vec<mandi_domain::core::error::FieldError>| {
        for item in &mut list {
            if item.field == "account_id" {
                item.field = field.to_owned();
            }
        }
        list
    };
    errors
        .finish()
        .map_err(|list| ServiceError::from(rename(list)))?;
    party.ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, field))
}

/// A buyer's open bills, oldest first, with their age (PRD FR-7.58, C14). Only reads.
///
/// Example (PRD 4.6, E2E-6): after Events 1 and 2 and اکرم's 10,000, his one open bill is بکری 1
/// of 10-10-26, 12,880 less 10,000 = 2,880, 0 days old.
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_NOT_A_BUYER` on `account_id`; `ERR_PERSON_REQUIRED` and `ERR_NOT_FOUND`
/// on `person_key`.
pub fn open_bills(ctx: &Context, input: &OpenBillsInput) -> Result<OpenBills, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let party = party_for_read(
            conn,
            input.account_id,
            input.person_key.as_deref(),
            "account_id",
            true,
        )?;
        let key = party.person_key.as_deref();
        let mut bills = Vec::new();
        // The oldest bills only (PRD FR-7.58); the count and the open total are of all of them.
        let found = repo::open_bill_rows_fast(conn, party.record.id, key, MAX_OPEN_ROWS)?;
        let open_total = paisa(found.open_paisa)?;
        let total_count = count_of(found.total_count)?;
        for row in found.rows {
            let amount = paisa(row.amount_paisa)?;
            let open = amount
                .checked_sub(paisa(row.matched_paisa)?)
                .map_err(money_error)?;
            let day = BusinessDate::parse_iso(&row.entry_date)
                .ok_or_else(|| ServiceError::Data(format!("entry date {}", row.entry_date)))?;
            bills.push(OpenBill {
                line_id: row.line_id,
                entry_date: row.entry_date,
                source: line_source(&row.posting_type, &row.source_type),
                number: row.number,
                amount_paisa: amount,
                open_paisa: open,
                days: today.days_after(day),
            });
        }
        // What no bill has used: the credit less the debit when that is above 0 (the rows are
        // canonical, so a credit waits only while no bill is open).
        let unmatched = paisa(matching_repo::expected_unmatched_credit(
            conn,
            party.record.id,
            key,
        )?)?;
        Ok(OpenBills {
            oldest_days: bills.first().map(|b| b.days),
            truncated: total_count > count_of(bills.len())?,
            bills,
            total_count,
            open_paisa: open_total,
            unmatched_credit_paisa: unmatched,
            balance: ledger::balance_in(conn, party.record.id, key, None)?,
            account: party.record,
        })
    })
}

/// What is payable to a seller, oldest first (PRD FR-7.59, money agent 8.2a). Only reads.
///
/// Example (PRD 4.5, E2E-27): after Event 2, کریم has one posting: بکری 2, صافی بکری 10,160,
/// advance adjusted 0, paid 0, still open 10,160. A seller's opening credit (PRD FR-10.9) is his
/// oldest posting; a جنرل credit on him is a posting too (PRD 16.4).
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_NOT_A_SELLER` on `seller_id`; `ERR_PERSON_REQUIRED` and `ERR_NOT_FOUND`
/// on `person_key`.
pub fn open_payables(
    ctx: &Context,
    input: &OpenPayablesInput,
) -> Result<OpenPayables, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let party = party_for_read(
            conn,
            input.seller_id,
            input.person_key.as_deref(),
            "seller_id",
            false,
        )?;
        let key = party.person_key.as_deref();
        if !input.include_settled {
            return open_payables_fast(conn, today, party);
        }
        let live = matching_repo::live_lines(conn, party.record.id)?;
        // The lines of this seller key, with the advance rows as they are stored.
        let mut keyed = Vec::new();
        let mut details: HashMap<i64, &matching_repo::LiveLine> = HashMap::new();
        for line in live.iter().filter(|l| l.person_key.as_deref() == key) {
            keyed.push((
                !line.is_opening_credit(),
                payables::Line {
                    line_id: line.line_id,
                    dr: paisa(line.dr_paisa)?,
                    cr: paisa(line.cr_paisa)?,
                    is_advance: line.is_advance,
                    advance_matched: paisa(line.advance_matched_paisa)?,
                },
            ));
            details.insert(line.line_id, line);
        }
        // His opening credit is his oldest payable whatever its date (PRD FR-7.59); the sort is
        // stable, so every other line keeps its place.
        keyed.sort_by_key(|(not_opening_credit, _)| *not_opening_credit);
        let lines: Vec<payables::Line> = keyed.into_iter().map(|(_, line)| line).collect();
        let work = payables::work(&lines).map_err(|error| {
            tracing::error!(
                seller = party.record.id,
                %error,
                "the advance rows are bigger than their lines"
            );
            money_error(error)
        })?;
        let mut postings = Vec::new();
        for row in &work.rows {
            if row.still_open == Paisa::ZERO && !input.include_settled {
                continue;
            }
            let Some(line) = details.get(&row.line_id) else {
                continue;
            };
            let day = BusinessDate::parse_iso(&line.entry_date)
                .ok_or_else(|| ServiceError::Data(format!("entry date {}", line.entry_date)))?;
            postings.push(PayablePosting {
                entry_date: line.entry_date.clone(),
                source: line_source(&line.posting_type, &line.source_type),
                number: line.number,
                credit_paisa: row.credit,
                advance_adjusted_paisa: row.advance_adjusted,
                paid_paisa: row.paid,
                still_open_paisa: row.still_open,
                days: today.days_after(day),
            });
        }
        // The history of a seller was asked for: all of it, not capped.
        Ok(OpenPayables {
            account: party.record.clone(),
            total_count: count_of(postings.len())?,
            truncated: false,
            postings,
            open_paisa: work.open,
            advances_open_paisa: work.advances_open,
            owes_firm_paisa: work.excess_paid,
            balance: ledger::balance_in(conn, party.record.id, key, None)?,
        })
    })
}

/// The totals of a seller key (payable, open advances, owed beyond them) from sums of his lines,
/// without reading the lines one by one. `minus_payment` is a payment taken out first (the
/// voucher being edited).
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` if more is matched than a side holds.
pub(super) fn seller_totals(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
    minus_payment: Paisa,
) -> Result<payables::Totals, ServiceError> {
    let t = repo::payable_totals(conn, account_id, person_key)?;
    payables::from_totals(
        paisa(t.credit_paisa)?,
        paisa(t.payments_paisa)?
            .checked_sub(minus_payment)
            .map_err(money_error)?,
        paisa(t.advances_given_paisa)?,
        paisa(t.adjusted_on_credits_paisa)?,
        paisa(t.adjusted_on_advances_paisa)?,
    )
    .map_err(|error| {
        tracing::error!(account_id, %error, "the advance rows are bigger than their lines");
        money_error(error)
    })
}

/// The open payables without reading the whole history (money agent review of plan 15): the
/// totals come from sums, and the open postings are the newest credits, found by walking back
/// from the newest until they add up to the payable. Gives the postings that [`payables::work`]
/// gives, for the ones with something to pay, but only the OLDEST `MAX_OPEN_ROWS` of them are read
/// with their details; the count and the open total are of all of them.
fn open_payables_fast(
    conn: &Connection,
    today: BusinessDate,
    party: Party,
) -> Result<OpenPayables, ServiceError> {
    let key = party.person_key.as_deref();
    let totals = seller_totals(conn, party.record.id, key, Paisa::ZERO)?;
    let mut postings = Vec::new();
    let mut total_count = 0_usize;
    if totals.open > Paisa::ZERO {
        let credits = repo::payable_credits_oldest_first(conn, party.record.id, key)?;
        let adjusted = repo::advance_adjusted_by_credit(conn, party.record.id)?;
        let mut walk = payables::NewestWalk::new(totals.open);
        for credit in credits.iter().rev() {
            let done = walk
                .push(
                    credit.line_id,
                    paisa(credit.credit_paisa)?,
                    paisa(adjusted.get(&credit.line_id).copied().unwrap_or(0))?,
                )
                .map_err(money_error)?;
            if done {
                break;
            }
        }
        if walk.is_short() {
            return Err(ServiceError::Data(format!(
                "account {}: the payable is more than his credits",
                party.record.id
            )));
        }
        let found = walk.finish();
        total_count = found.len();
        // The details (where each came from) are read for the kept postings only.
        for posting in found.into_iter().take(MAX_OPEN_ROWS) {
            let row = repo::credit_row(conn, posting.line_id)?;
            let day = BusinessDate::parse_iso(&row.entry_date)
                .ok_or_else(|| ServiceError::Data(format!("entry date {}", row.entry_date)))?;
            postings.push(PayablePosting {
                entry_date: row.entry_date.clone(),
                source: line_source(&row.posting_type, &row.source_type),
                number: row.number,
                credit_paisa: posting.credit,
                advance_adjusted_paisa: posting.advance_adjusted,
                paid_paisa: posting.paid,
                still_open_paisa: posting.still_open,
                days: today.days_after(day),
            });
        }
    }
    Ok(OpenPayables {
        truncated: total_count > postings.len(),
        total_count: count_of(total_count)?,
        postings,
        open_paisa: totals.open,
        advances_open_paisa: totals.advances_open,
        owes_firm_paisa: totals.excess_paid,
        balance: ledger::balance_in(conn, party.record.id, key, None)?,
        account: party.record,
    })
}

/// The advances of sellers as of a day (PRD R26 data, E2E-7, E2E-31): given, returned by a
/// وصولی from him (PRD FR-15.12), adjusted by his sales (PRD FR-15.8) and open. Only reads.
///
/// Example (E2E-31): رحیم was given 15,000, returned 10,000 and had 5,000 adjusted by بکری 1, so
/// 0 is open.
///
/// # Errors
/// The date codes on `as_of`; `ERR_NOT_FOUND`, `ERR_NOT_A_SELLER` on `seller_id`;
/// `ServiceError::Data` if more was returned and adjusted than was given (a bug signal).
pub fn advances_report(
    ctx: &Context,
    input: &AdvancesReportInput,
) -> Result<AdvancesReport, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let as_of = if input.as_of.trim().is_empty() {
        today
    } else {
        validate::date("as_of", &input.as_of, today)?
    };
    let as_of_iso = as_of.to_iso();
    ctx.db.read(|conn| {
        if let Some(id) = input.seller_id {
            let facts = account_repo::facts(conn, id)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "seller_id"))?;
            if facts.control_code != mandi_domain::core::accounts::SELLER_CONTROL
                || facts.is_pukhta_khata
            {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrNotASeller,
                    "seller_id",
                ));
            }
        }
        let mut rows = Vec::new();
        let (mut given, mut returned, mut adjusted, mut open) =
            (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        for totals in repo::advance_totals(conn, &as_of_iso, input.seller_id)? {
            let account = account_repo::get(conn, totals.account_id)?
                .map(|row| RecordRef {
                    id: row.id,
                    code: row.code,
                    name_ur: row.name_ur,
                    name_en: row.name_en,
                    tag: None,
                })
                .ok_or_else(|| ServiceError::Data("an advance names no account".to_owned()))?;
            let (g, r, a) = (
                paisa(totals.given_paisa)?,
                paisa(totals.returned_paisa)?,
                paisa(totals.adjusted_paisa)?,
            );
            let left = g
                .checked_sub(r)
                .and_then(|v| v.checked_sub(a))
                .map_err(money_error)?;
            if left.is_negative() {
                tracing::error!(
                    seller = totals.account_id,
                    "more was returned and adjusted than was given as advance"
                );
                return Err(ServiceError::Data(format!(
                    "account {}: more returned and adjusted than given as advance",
                    totals.account_id
                )));
            }
            given = given.checked_add(g).map_err(money_error)?;
            returned = returned.checked_add(r).map_err(money_error)?;
            adjusted = adjusted.checked_add(a).map_err(money_error)?;
            open = open.checked_add(left).map_err(money_error)?;
            rows.push(AdvancesReportRow {
                account,
                given_paisa: g,
                returned_paisa: r,
                adjusted_paisa: a,
                open_paisa: left,
                last_date: totals.last_date,
            });
        }
        rows.sort_by(|a, b| a.account.code.cmp(&b.account.code));
        Ok(AdvancesReport {
            as_of: as_of_iso,
            rows,
            given_paisa: given,
            returned_paisa: returned,
            adjusted_paisa: adjusted,
            open_paisa: open,
        })
    })
}

/// The اگراہی فہرست of a day (PRD FR-15.6, FR-7.61): the active buyers whose collection weekday is
/// the weekday of the day and who owe something, with their dues and the age of their oldest open
/// bill, grouped by collector and city. Only reads. Team default (plan 15): the weekday set on
/// the buyer's account decides.
///
/// # Errors
/// The date codes on `date`.
pub fn collection_list(
    ctx: &Context,
    input: &CollectionListInput,
) -> Result<CollectionList, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let date = validate::date("date", &input.date, today)?;
    let weekday = date.weekday_from_monday();
    ctx.db.read(|conn| {
        let candidates = repo::collection_candidates(
            conn,
            i64::from(weekday),
            input.collector_staff_id,
            input.city_id,
        )?;
        let ids: Vec<i64> = candidates.iter().map(|c| c.account.id).collect();
        let totals = ledger_repo::totals_for_accounts(conn, &ids)?;
        let oldest = repo::oldest_open_bill_dates(conn, &ids)?;
        // (collector id, city id) in id order; a buyer with none sorts first.
        let mut groups: BTreeMap<(Option<i64>, Option<i64>), CollectionGroup> = BTreeMap::new();
        let mut grand = Paisa::ZERO;
        for candidate in candidates {
            let balance =
                ledger::balance_of(totals.get(&candidate.account.id).copied().unwrap_or(
                    ledger_repo::Totals {
                        dr_paisa: 0,
                        cr_paisa: 0,
                    },
                ))?;
            // Only what he owes is collected.
            if balance.side != BalanceSide::Dr {
                continue;
            }
            let oldest_bill_date = oldest.get(&candidate.account.id).cloned();
            let oldest_days = match &oldest_bill_date {
                Some(text) => Some(
                    today.days_after(
                        BusinessDate::parse_iso(text)
                            .ok_or_else(|| ServiceError::Data(format!("entry date {text}")))?,
                    ),
                ),
                None => None,
            };
            grand = grand
                .checked_add(balance.amount_paisa)
                .map_err(money_error)?;
            let key = (
                candidate.collector.as_ref().map(|c| c.id),
                candidate.city.as_ref().map(|c| c.id),
            );
            let group = groups.entry(key).or_insert_with(|| CollectionGroup {
                collector: candidate.collector.clone(),
                city: candidate.city.clone(),
                buyers: Vec::new(),
                total_paisa: Paisa::ZERO,
            });
            group.total_paisa = group
                .total_paisa
                .checked_add(balance.amount_paisa)
                .map_err(money_error)?;
            group.buyers.push(CollectionBuyer {
                account: candidate.account,
                phone: candidate.phone,
                dues_paisa: balance.amount_paisa,
                oldest_bill_date,
                oldest_days,
            });
        }
        Ok(CollectionList {
            date: date.to_iso(),
            weekday,
            groups: groups.into_values().collect(),
            total_paisa: grand,
        })
    })
}
