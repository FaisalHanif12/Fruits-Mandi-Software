//! Saves a بکری (PRD FR-12.8, FR-12.9, plan 13): ONE user action, ONE transaction, all or nothing
//! (Code Standards B3, NFR-REL-1).
//!
//! The draft is worked out by `work_out`, the very function of the live preview, so the saved
//! numbers are the previewed ones (Code Standards B5). Then, inside the same transaction:
//!
//! 1. every error of the draft is returned together and nothing is written; then the cash
//!    confirmation (PRD FR-6.7);
//! 2. the بکری number (the biggest ever given plus one) and the `bikri_entries` row;
//! 3. each lot, with its final stored totals in the INSERT (the table checks that deductions are
//!    the sum of the charges and net is gross less deductions);
//! 4. the journal entry of PRD FR-12.9 through `ledger::post` (draft, lines, one update to
//!    posted; the partner share rows are written by the hook inside it);
//! 5. each sale line with its snapshot (method, value, rule, amount; PRD FR-11.6), stock moves
//!    and analytics facts;
//! 6. matching: the pinned pair of each نقدی line, then the new bills of each buyer key matched to
//!    his unmatched credits, then the advance adjustment of each lot whose tick is on;
//! 7. the audit row, and the draft of the screen is removed.
//!
//! A failure anywhere rolls the whole transaction back: no row is left, and the number is not
//! used.
//!
//! Worked numbers (PRD 4.4): Event 1 saves بکری number 1 with the journal of 12 lines, sale part
//! 31,564 = 31,564, whole entry 36,564 = 36,564, share rows 1,580 / 1,184 / 1,184, and (tick on)
//! 15,000 of رحیم's advance adjusted.

use mandi_db::bikri::write::{self as repo, NewEntry};
use mandi_domain::core::bikri::{BikriSaveInput, BikriSaved, prepare_draft};
use mandi_domain::core::money::Paisa;
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::json;

use super::rows::{
    LotCharges, LotShape, Posting, check_shares, insert_sales_and_facts, lot_columns, lot_person,
    lot_running_totals, match_posting, one_based, posting_sums, shape, stored_line_ids,
};
use super::shared::{Basis, data, switches, work_out};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::drafts;
use crate::error::ServiceError;
use crate::ledger::{self, PostingEvent};
use crate::rules;

/// Saves a بکری in one transaction (PRD FR-12.8). See the module text for the order.
///
/// # Errors
/// Every error and code of the preview, each at its field, all together and with nothing written
/// (`lots[i].sales[j].qty`, `ERR_NO_LOTS`, `ERR_NO_SALE_LINES` on `lots`, `ERR_DATE_CLOSED` on
/// `header.entry_date`, `ERR_SHARES_NOT_100` on `split`, ...);
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`) when the entry takes روکڑ below
/// 0 and `confirm_cash_below_zero` is false; `ServiceError::Db` or `ServiceError::Data` for a
/// failure the user cannot fix, which rolls everything back.
pub fn save(
    ctx: &Context,
    actor: &Actor,
    input: &BikriSaveInput,
) -> Result<BikriSaved, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    let prepared = prepare_draft(&input.draft, today, closed_up_to);
    // Fetched before the write: the cache reads through the read connection.
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let worked = work_out(tx, prepared, &rule_set, switches, &Basis::new_bikri())?;
        if !worked.errors.is_empty() {
            return Err(ServiceError::from(worked.errors.clone()));
        }
        cash::require_confirmed(&worked.cash, input.confirm_cash_below_zero)?;
        let (entry, layout) = worked
            .entry
            .as_ref()
            .ok_or_else(|| data("a بکری without errors has no journal entry"))?;
        let lots = shape(&worked)?;
        let header = &worked.prepared.header;
        let date = header
            .entry_date
            .ok_or_else(|| data("a بکری without errors has no date"))?;
        let date_iso = date.to_iso();
        let bill_iso = header.bill_date.unwrap_or(date).to_iso();

        // 2. The بکری.
        let number = repo::next_number(tx)?;
        let bikri_id = repo::insert_entry(
            tx,
            &NewEntry {
                number,
                book: header.book.as_deref(),
                entry_date: &date_iso,
                bill_date: &bill_iso,
                truck_no: header.truck_no.as_deref(),
                zamindar_account_id: header.zamindar_account_id,
                adda_id: header.adda_id,
                driver_name: header.driver_name.as_deref(),
                driver_phone: header.driver_phone.as_deref(),
                total_freight_paisa: header.total_freight.map(Paisa::as_paisa),
                store_id: header.store_id,
                note: header.note.as_deref(),
            },
            actor.user_id,
            &now,
        )?;

        // 3. The lots, with their final totals.
        let mut lot_ids = Vec::with_capacity(lots.len());
        for (index, shaped) in lots.iter().enumerate() {
            lot_ids.push(insert_lot(
                tx,
                actor.user_id,
                &now,
                bikri_id,
                index,
                shaped,
            )?);
        }

        // 4. The journal entry, posted through the one function.
        let posted = ledger::post(
            ctx,
            tx,
            actor.user_id,
            &PostingEvent {
                date,
                posting_type: PostingType::BikriFirst,
                source_type: SourceType::Bikri,
                source_id: bikri_id,
                narration: None,
                reverses: None,
                entry: entry.clone(),
            },
        )?;
        let line_ids = stored_line_ids(tx, posted.entry_id, entry.lines().len())?;
        for lot_id in &lot_ids {
            repo::set_lot_first_entry(tx, *lot_id, posted.entry_id, actor.user_id, &now)?;
        }

        // 5. The sale lines, stock moves and analytics facts.
        let posting = Posting {
            user_id: actor.user_id,
            now: &now,
            bikri_id,
            entry_id: posted.entry_id,
            date_iso: &date_iso,
            year: date.year(),
            month: date.month(),
            adda_id: header.adda_id,
        };
        let lines_saved: usize = insert_sales_and_facts(tx, &posting, &lots, &lot_ids, true)?
            .iter()
            .map(Vec::len)
            .sum();

        // A trip whose last goods this sold closes by itself (PRD FR-12.33).
        crate::purchases::refresh_trips_of_entry(
            tx,
            posted.entry_id,
            &date_iso,
            actor.user_id,
            &now,
        )?;

        // 6. Matching.
        match_posting(tx, &posting, &lots, (entry, layout), &line_ids)?;

        // The shares were written by the hook inside `ledger::post`; they must add up to the
        // shared income the screen showed (PRD FR-6.5).
        check_shares(tx, posted.entry_id, worked.calc.summary.shared_income)?;

        // 7. Audit and the draft.
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "create", "bikri")
                .entity_id(bikri_id)
                .new_value(json!({
                    "number": number,
                    "entry_date": date_iso,
                    "entry_id": posted.entry_id,
                    "lots": lots.len(),
                    "lines": lines_saved,
                    "gross_paisa": worked.calc.summary.gross.as_paisa(),
                    "deductions_paisa": worked.calc.summary.deductions.as_paisa(),
                    "net_paisa": worked.calc.summary.net.as_paisa(),
                    "buyer_total_paisa": worked.calc.summary.buyer_total.as_paisa(),
                    "shared_income_paisa": worked.calc.summary.shared_income.as_paisa(),
                    "cash_after_paisa": worked.cash.cash_after_paisa.as_paisa(),
                    "cash_below_zero_confirmed": worked.cash.needs_confirm,
                }))
                .into_entry(),
        )?;
        drafts::discard_in(tx, actor.user_id, drafts::BIKRI_SCREEN)?;
        Ok(BikriSaved {
            bikri_id,
            number,
            entry_id: posted.entry_id,
            preview: worked.preview(),
        })
    })
}

/// Adds one lot with the totals `calc_posting` worked out (PRD FR-11.6, B20). The running totals
/// of a lot in its first save are the sums of its own lines; the table checks the same formulas
/// (C6 to C8), and they must equal the totals of the preview.
fn insert_lot(
    tx: &mandi_db::rusqlite::Connection,
    user_id: i64,
    now: &str,
    bikri_id: i64,
    index: usize,
    shaped: &LotShape<'_>,
) -> Result<i64, ServiceError> {
    let person = lot_person(shaped)?;
    let running = lot_running_totals(&LotCharges::of_totals(shaped.totals), posting_sums(shaped)?)?;
    let totals = shaped.totals;
    if running.gross_paisa != totals.gross.as_paisa()
        || running.seller_comm_paisa != totals.seller_comm.as_paisa()
        || running.deductions_paisa != totals.deductions.as_paisa()
        || running.net_paisa != totals.net.as_paisa()
    {
        return Err(data("the stored totals of a lot differ from its preview"));
    }
    let columns = lot_columns(
        tx,
        bikri_id,
        one_based(index)?,
        shaped,
        person.as_ref(),
        &running,
    )?;
    Ok(repo::insert_lot(tx, &columns, user_id, now)?)
}
