//! Edit of one posting of a بکری (PRD FR-12.18 to FR-12.21, FR-11.9, FR-11.6, plan 16).
//!
//! An edit is a reversal of the posting plus a new posting on the SAME day, in one transaction
//! (PRD FR-12.19):
//!
//! 1. the draft is checked against the stored rows and worked out by the same function as the
//!    preview (`work_out`, Code Standards B5), with the stored lines and lot charges, so a line the
//!    user did not change keeps its stored values exactly (PRD FR-12.20);
//! 2. the old posting is taken back (`ledger::reverse`: the journal, and through the hook the
//!    partner share rows), its stock moves and analytics facts get opposite rows, and the matching
//!    that rested on its lines is switched off;
//! 3. the new posting is posted with the order key of the old one, so it keeps its place in the day
//!    for matching; the partner split is the one of the posting date (PRD FR-12.21);
//! 4. new sale lines are written, the old ones end (`replaced` or `removed`), the lots are written
//!    in ONE update each, and `first_entry_id` points at the new first posting;
//! 5. every buyer key and every seller the two postings touch is matched again from the start,
//!    never by the fast paths (Architecture 12); the pinned pair of each نقدی line is written first;
//! 6. the audit row holds the old and the new lines and the reason.
//!
//! A posting of a cancelled بکری cannot be edited. A closed day needs the Owner to reopen it first
//! (PRD FR-12.18).
//!
//! Example (PRD E2E-13): Event 1, اکرم's rate 600 → 610. His line is 20 × 610 = 12,200, buyer
//! commission 732, he pays 13,092; خام 29,600, seller commission 1,776, صافی 19,504; the shared
//! income is 3,972, split 1,590 / 1,191 / 1,191.

use std::collections::{BTreeSet, HashSet};

use mandi_db::bikri::edit::{self as edit_repo, Ended, PostingRef};
use mandi_db::bikri::write::{self as repo};
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::{PostingKind, PostingTarget};
use mandi_domain::core::bikri::{
    BikriDetail, BikriDraft, BikriEdited, BikriPreview, BikriStatus, EditOpenInput,
    EditPreviewInput, EditSaveInput, EditableBikri, PreparedDraft, SavedLot, prepare_draft,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::{Value, json};

use super::load::{
    detail_of, draft_of, later_header, later_lot_input, posting_lots, stored_posting,
};
use super::rows::{
    LotCharges, LotShape, Posting, check_shares, insert_sales_and_facts, lot_columns, lot_person,
    lot_running_totals, pin_cash_sales, posting_sums, shape, stored_line_ids,
};
use super::shared::{Basis, Replaced, Worked, data, switches, work_out};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::ServiceError;
use crate::ledger::{self, PostingEvent};
use crate::rules;
use crate::vouchers::{rematch_party, role_of_account, stored_cash_change};

/// Everything an edit works from, read once.
struct EditPlan {
    detail: BikriDetail,
    posting: PostingRef,
    kind: PostingKind,
    date: BusinessDate,
    prepared: PreparedDraft,
    basis: Basis,
    /// Mistakes of the draft against the stored rows (a lot or line that is not the posting's).
    extra: Vec<FieldError>,
}

fn not_found(field: &str) -> ServiceError {
    ServiceError::reject_field(ErrorCode::ErrNotFound, field)
}

/// An edit the rules do not allow (NFR-USE-13). `rule` is a short fixed name the screen turns
/// into words: `day_changed`, `lot_removed`, `lot_not_in_posting`, `lot_order_changed`,
/// `lot_seller_locked`, `lot_item_locked`, `lot_purchase_locked`, `line_not_in_posting`, `posting_lots_locked`.
fn not_allowed(field: &str, rule: &str) -> FieldError {
    FieldError::new(ErrorCode::ErrEditNotAllowed, field)
        .with("rule", ErrorValue::Text(rule.to_owned()))
}

fn kind_of(posting: &PostingRef) -> PostingKind {
    if posting.posting_type == "bikri_first" {
        PostingKind::First
    } else {
        PostingKind::Later
    }
}

/// The بکری and the posting an edit is about, read and checked.
fn load_posting(
    conn: &Connection,
    entry_id: i64,
) -> Result<(BikriDetail, PostingRef), ServiceError> {
    let bikri_id =
        edit_repo::bikri_of_entry(conn, entry_id)?.ok_or_else(|| not_found("entry_id"))?;
    let detail = detail_of(conn, bikri_id, "entry_id")?;
    if detail.status == BikriStatus::Cancelled {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrBikriCancelled,
            "entry_id",
        ));
    }
    // PRD FR-12.38: a بکری that sold the goods of a closed garden is not edited until the closing is
    // undone. Changing a lot's seller here would take the money back from the garden's account
    // after it was paid out to the partners.
    if mandi_db::gardens::bikri_has_closed_garden_lot(conn, bikri_id)? {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrGardenClosed,
            "entry_id",
        ));
    }
    let posting = edit_repo::posting_in_force(conn, bikri_id, entry_id)?
        .ok_or_else(|| not_found("entry_id"))?;
    Ok((detail, posting))
}

/// Loads one posting as a draft with its stored values (PRD FR-12.18). Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `bikri_id` or `entry_id`; `ERR_BIKRI_CANCELLED` on `entry_id`.
pub fn open_for_edit(ctx: &Context, input: &EditOpenInput) -> Result<EditableBikri, ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let (detail, posting) = load_posting(conn, input.entry_id)?;
        if detail.id != input.bikri_id {
            return Err(not_found("entry_id"));
        }
        let kind = kind_of(&posting);
        let date = BusinessDate::parse_iso(&posting.entry_date)
            .ok_or_else(|| data("the date of a posting is not a date"))?;
        let draft = draft_of(&detail, posting.entry_id, date, kind, true)?;
        let stored = posting_lots(&detail, posting.entry_id, kind);
        let mut shown = detail.clone();
        super::read::mark_editable(&mut shown, closed_up_to);
        let editable = shown
            .postings
            .iter()
            .any(|p| p.entry_id == posting.entry_id && p.editable);
        Ok(EditableBikri {
            bikri_id: detail.id,
            number: detail.number,
            version: detail.version,
            entry_id: posting.entry_id,
            posting_type: posting.posting_type,
            entry_date: posting.entry_date,
            editable,
            draft,
            stored,
            postings: shown.postings,
        })
    })
}

/// Checks the draft of the FIRST posting against the stored rows: the day, the lots, their order
/// and the lines (PRD FR-12.18, FR-12.20). Every mistake comes back with its field.
fn check_first(
    detail: &BikriDetail,
    entry_id: i64,
    date: BusinessDate,
    prepared: &PreparedDraft,
) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if prepared.header.entry_date.is_some_and(|d| d != date) {
        // An edit keeps the day of the posting (PRD FR-12.19).
        errors.push(not_allowed("header.entry_date", "day_changed"));
    }
    let stored: Vec<&SavedLot> = detail.lots.iter().collect();
    let mut seen_lots: HashSet<i64> = HashSet::new();
    let mut seen_sales: HashSet<i64> = HashSet::new();
    let mut last_lot_no = 0_i64;
    let mut new_lot_started = false;
    for (index, lot) in prepared.lots.iter().enumerate() {
        match lot.lot_id {
            None => new_lot_started = true,
            Some(id) => {
                let Some(found) = stored.iter().find(|l| l.id == id) else {
                    errors.push(not_allowed(
                        &format!("lots[{index}].lot_id"),
                        "lot_not_in_posting",
                    ));
                    continue;
                };
                // The saved lots keep their order, and new lots come after them: lines of the
                // journal are paired with lots by position.
                if !seen_lots.insert(id) {
                    errors.push(not_allowed(
                        &format!("lots[{index}].lot_id"),
                        "lot_not_in_posting",
                    ));
                } else if new_lot_started || found.lot_no < last_lot_no {
                    errors.push(not_allowed(
                        &format!("lots[{index}].lot_id"),
                        "lot_order_changed",
                    ));
                }
                last_lot_no = found.lot_no;
                // A lot that other postings or write-offs rest on keeps its seller and item.
                let others = found.qty_written_off > 0
                    || found.qty_taken > 0
                    || found
                        .sales
                        .iter()
                        .any(|s| s.is_active && s.journal_entry_id != entry_id);
                if others && lot.seller_account_id != Some(found.seller.id) {
                    errors.push(not_allowed(
                        &format!("lots[{index}].seller_account_id"),
                        "lot_seller_locked",
                    ));
                }
                // The purchase of an own lot never changes: its stock and cost rest on it.
                if lot.purchase_id != found.purchase.as_ref().map(|p| p.id) {
                    errors.push(not_allowed(
                        &format!("lots[{index}].purchase_id"),
                        "lot_purchase_locked",
                    ));
                }
                if others && lot.item_id != Some(found.item.id) {
                    errors.push(not_allowed(
                        &format!("lots[{index}].item_id"),
                        "lot_item_locked",
                    ));
                }
                for (j, sale) in lot.sales.iter().enumerate() {
                    if let Some(sale_id) = sale.sale_id {
                        let ours = found.sales.iter().any(|s| {
                            s.id == sale_id && s.is_active && s.journal_entry_id == entry_id
                        });
                        if !ours || !seen_sales.insert(sale_id) {
                            errors.push(not_allowed(
                                &format!("lots[{index}].sales[{j}].sale_id"),
                                "line_not_in_posting",
                            ));
                        }
                    }
                }
            }
        }
        if lot.lot_id.is_none() {
            for (j, sale) in lot.sales.iter().enumerate() {
                if sale.sale_id.is_some() {
                    errors.push(not_allowed(
                        &format!("lots[{index}].sales[{j}].sale_id"),
                        "line_not_in_posting",
                    ));
                }
            }
        }
    }
    // No lot can be removed in an edit: its postings and stock rest on it.
    if stored.iter().any(|l| !seen_lots.contains(&l.id)) {
        errors.push(not_allowed("lots", "lot_removed"));
    }
    errors
}

/// Reads and checks everything an edit works from. Nothing is written.
fn plan_edit(
    conn: &Connection,
    entry_id: i64,
    draft: &BikriDraft,
    window: (Option<BusinessDate>, BusinessDate),
) -> Result<EditPlan, ServiceError> {
    let (detail, posting) = load_posting(conn, entry_id)?;
    let kind = kind_of(&posting);
    let date = BusinessDate::parse_iso(&posting.entry_date)
        .ok_or_else(|| data("the date of a posting is not a date"))?;
    let (closed_up_to, today) = window;
    let mut extra = Vec::new();
    let effective = match kind {
        PostingKind::First => draft.clone(),
        PostingKind::Later => {
            // Only the lines of a later posting are the user's; the lot comes from the database.
            let lots = posting_lots(&detail, entry_id, PostingKind::Later);
            let lot = lots
                .first()
                .ok_or_else(|| data("a later posting has no lot"))?;
            let sent = match draft.lots.as_slice() {
                [only] if only.lot_id == Some(lot.id) => only,
                _ => {
                    extra.push(not_allowed("lots", "posting_lots_locked"));
                    draft.lots.first().ok_or_else(|| not_found("lots"))?
                }
            };
            let valid: HashSet<i64> = lot
                .sales
                .iter()
                .filter(|s| s.is_active && s.journal_entry_id == entry_id)
                .map(|s| s.id)
                .collect();
            let mut seen = HashSet::new();
            for (j, sale) in sent.sales.iter().enumerate() {
                if let Some(id) = sale.sale_id
                    && (!valid.contains(&id) || !seen.insert(id))
                {
                    extra.push(not_allowed(
                        &format!("lots[0].sales[{j}].sale_id"),
                        "line_not_in_posting",
                    ));
                }
            }
            let state = edit_repo::lot_state(conn, lot.id)?.ok_or_else(|| not_found("lots"))?;
            let mut header = later_header(date);
            // The day cannot change in an edit (PRD FR-12.19).
            if validate::date("header.entry_date", &draft.header.entry_date, today)
                .is_ok_and(|d| d != date)
            {
                extra.push(not_allowed("header.entry_date", "day_changed"));
            }
            header.entry_date = mandi_domain::core::dates::format_ddmmyyyy(date);
            BikriDraft {
                header,
                lots: vec![later_lot_input(&state, sent.sales.clone())],
            }
        }
    };
    let prepared = prepare_draft(&effective, today, closed_up_to);
    if kind == PostingKind::First {
        extra.extend(check_first(&detail, entry_id, date, &prepared));
    }
    let old_entry = ledger_repo::get_entry(conn, entry_id)?
        .ok_or_else(|| data("the posting of a بکری cannot be read"))?;
    let stored = stored_posting(conn, &detail, entry_id, kind)?;
    let basis = Basis {
        target: PostingTarget::Edit { entry_id },
        stored,
        replaces: Some(Replaced {
            entry_id,
            cash_change: stored_cash_change(conn, &old_entry)?,
        }),
    };
    Ok(EditPlan {
        detail,
        posting,
        kind,
        date,
        prepared,
        basis,
        extra,
    })
}

/// Works out an edited posting for the live summary (PRD FR-12.20). Nothing is saved.
///
/// # Errors
/// `ERR_NOT_FOUND` on `entry_id`; `ERR_BIKRI_CANCELLED` on `entry_id`. The mistakes in the draft
/// come back inside the preview (`errors`).
pub fn edit_preview(ctx: &Context, input: &EditPreviewInput) -> Result<BikriPreview, ServiceError> {
    let window = day::window(ctx)?;
    // Fetched before the read below: the read connection is not re-entrant.
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    ctx.db.read(|conn| {
        let plan = plan_edit(conn, input.entry_id, &input.draft, window)?;
        let mut worked = work_out(conn, plan.prepared, &rule_set, switches, &plan.basis)?;
        worked.errors.extend(plan.extra);
        Ok(worked.preview())
    })
}

/// The lines of a posting for the audit log: what the user can compare before and after.
fn lines_json(lots: &[SavedLot]) -> Value {
    Value::Array(
        lots.iter()
            .flat_map(|lot| {
                lot.sales.iter().map(move |s| {
                    json!({
                        "lot_id": lot.id,
                        "sale_id": s.id,
                        "buyer_account_id": s.buyer.id,
                        "qty": s.qty,
                        "seller_rate_paisa": s.seller_rate.as_paisa(),
                        "buyer_rate_paisa": s.buyer_rate.as_paisa(),
                        "seller_comm_paisa": s.seller_comm.amount.as_paisa(),
                        "buyer_comm_paisa": s.buyer_comm.amount.as_paisa(),
                        "laga_paisa": s.laga.amount.as_paisa(),
                        "buyer_total_paisa": s.buyer_total.as_paisa(),
                        "payment_type": s.payment_type.as_str(),
                    })
                })
            })
            .collect(),
    )
}

fn shaped_lines_json(lots: &[LotShape<'_>]) -> Value {
    Value::Array(
        lots.iter()
            .flat_map(|lot| {
                lot.rows.iter().map(|r| {
                    json!({
                        "buyer_account_id": r.journal.buyer_account_id.get(),
                        "qty": r.calc.qty,
                        "seller_rate_paisa": r.calc.seller_rate.as_paisa(),
                        "buyer_rate_paisa": r.calc.buyer_rate.as_paisa(),
                        "seller_comm_paisa": r.calc.seller_comm.amount.as_paisa(),
                        "buyer_comm_paisa": r.calc.buyer_comm.amount.as_paisa(),
                        "laga_paisa": r.calc.laga.amount.as_paisa(),
                        "buyer_total_paisa": r.calc.buyer_total.as_paisa(),
                        "payment_type": r.calc.payment_type.as_str(),
                    })
                })
            })
            .collect(),
    )
}

/// The buyer keys and sellers that two postings touch: each is matched again from the start.
fn touched_keys(
    old: &ledger_repo::StoredEntry,
    new: &ledger_repo::StoredEntry,
) -> BTreeSet<(i64, Option<String>)> {
    old.lines
        .iter()
        .chain(&new.lines)
        .map(|l| (l.account_id, l.person_key.clone()))
        .collect()
}

/// Saves an edit (PRD FR-12.19): see the module text.
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_BIKRI_CANCELLED` on `entry_id`; `ERR_VERSION_CHANGED` on `version`;
/// every error of the book screen at its field, `ERR_FORMAT_INVALID` for a lot or line that is not
/// the posting's; `ERR_DATE_CLOSED`; `ERR_QTY_BELOW_SOLD`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`
/// (value `cash_after_paisa`). Nothing is written when any of them is returned.
#[allow(clippy::too_many_lines)]
pub fn edit_save(
    ctx: &Context,
    actor: &Actor,
    input: &EditSaveInput,
) -> Result<BikriEdited, ServiceError> {
    let window = day::window(ctx)?;
    let reason = match input
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty())
    {
        Some(text) => Some(validate::free_text("reason", text)?),
        None => None,
    };
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    ctx.db.write(|tx| {
        let plan = plan_edit(tx, input.entry_id, &input.draft, window)?;
        if plan.detail.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        let EditPlan {
            detail,
            posting,
            kind,
            date,
            prepared,
            basis,
            extra,
        } = plan;
        let worked = work_out(tx, prepared, &rule_set, switches, &basis)?;
        let mut errors = worked.errors.clone();
        errors.extend(extra);
        if !errors.is_empty() {
            return Err(ServiceError::from(errors));
        }
        cash::require_confirmed(&worked.cash, input.confirm_cash_below_zero)?;
        write_edit(
            ctx,
            tx,
            actor,
            &Edit {
                detail: &detail,
                posting: &posting,
                kind,
                date,
                worked: &worked,
                reason: reason.as_deref(),
            },
        )
    })
}

/// What `write_edit` is told.
struct Edit<'a> {
    detail: &'a BikriDetail,
    posting: &'a PostingRef,
    kind: PostingKind,
    date: BusinessDate,
    worked: &'a Worked,
    reason: Option<&'a str>,
}

#[allow(clippy::too_many_lines)]
fn write_edit(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    edit: &Edit<'_>,
) -> Result<BikriEdited, ServiceError> {
    let Edit {
        detail,
        posting: old_posting,
        kind,
        date,
        worked,
        reason,
    } = *edit;
    let now = ctx.now_text();
    let user = actor.user_id;
    let old_entry_id = old_posting.entry_id;
    let (entry, layout) = worked
        .entry
        .as_ref()
        .ok_or_else(|| data("an edit without errors has no journal entry"))?;
    let lots = shape(worked)?;
    let first = kind == PostingKind::First;

    // 1. Take the old posting back, on its own day (ERR_DATE_CLOSED if it is closed).
    let old_key = ledger::order_key_of(tx, old_entry_id)?;
    let old_entry = ledger_repo::get_entry(tx, old_entry_id)?
        .ok_or_else(|| data("the posting of a بکری cannot be read"))?;
    let reversal = ledger::reverse(ctx, tx, user, old_entry_id, reason.unwrap_or(""))?;
    edit_repo::reverse_moves(tx, old_entry_id, reversal.entry_id, user, &now)?;
    edit_repo::reverse_facts(tx, old_entry_id, reversal.entry_id, user, &now)?;
    // The trip partners' parts of the seller credit are taken back (PRD FR-12.26, C25).
    edit_repo::reverse_credit_parts(tx, old_entry_id, reversal.entry_id, user, &now)?;
    // The own goods the old lines took go back to their purchases, exactly (PRD FR-12.20).
    crate::purchases::give_back_entry(tx, old_entry_id, user, &now)?;

    // 2. The new posting, with the place of the old one.
    let posted = ledger::post_ordered(
        ctx,
        tx,
        user,
        &PostingEvent {
            date,
            posting_type: if first {
                PostingType::BikriFirst
            } else {
                PostingType::BikriLater
            },
            source_type: SourceType::Bikri,
            source_id: detail.id,
            narration: None,
            reverses: None,
            entry: entry.clone(),
        },
        Some(old_key),
    )?;
    let line_ids = stored_line_ids(tx, posted.entry_id, entry.lines().len())?;

    // 3. New lots (first posting only), then the new sale lines of every lot.
    let mut lot_ids = Vec::with_capacity(lots.len());
    let mut next_lot_no = detail
        .lots
        .iter()
        .map(|l| l.lot_no)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| data("too many lots"))?;
    for shaped in &lots {
        match shaped.lot.lot_id {
            Some(id) => lot_ids.push(id),
            None => {
                let person = lot_person(shaped)?;
                let running = lot_running_totals(
                    &LotCharges::of_totals(shaped.totals),
                    posting_sums(shaped)?,
                )?;
                let columns = lot_columns(
                    tx,
                    detail.id,
                    next_lot_no,
                    shaped,
                    person.as_ref(),
                    &running,
                )?;
                lot_ids.push(repo::insert_lot(tx, &columns, user, &now)?);
                next_lot_no = next_lot_no
                    .checked_add(1)
                    .ok_or_else(|| data("too many lots"))?;
            }
        }
    }
    let new_entry_id = posted.entry_id;
    let post = Posting {
        user_id: user,
        now: &now,
        bikri_id: detail.id,
        entry_id: new_entry_id,
        date_iso: &old_posting.entry_date,
        year: date.year(),
        month: date.month(),
        // A later posting carries the truck stand of the بکری.
        adda_id: worked
            .prepared
            .header
            .adda_id
            .or(detail.adda_id.filter(|_| !first)),
    };
    let new_ids = insert_sales_and_facts(tx, &post, &lots, &lot_ids, first)?;

    // 4. The old lines end: replaced by the new line that carries their id, or removed.
    let mut replacements = std::collections::HashMap::new();
    for (shaped, ids) in lots.iter().zip(&new_ids) {
        for (row, new_id) in shaped.rows.iter().zip(ids) {
            if let Some(old_id) = row.row.sale_id {
                replacements.insert(old_id, *new_id);
            }
        }
    }
    let old_sales = edit_repo::active_sales_of_entry(tx, old_entry_id)?;
    for old in &old_sales {
        let (ended, by) = match replacements.get(&old.id) {
            Some(new_id) => (Ended::Replaced, Some(*new_id)),
            None => (Ended::Removed, None),
        };
        edit_repo::end_sale(tx, old.id, ended, by, user, &now)?;
    }

    // 5. The lots: ONE update each, with the first posting pointing at the new entry.
    for (shaped, lot_id) in lots.iter().zip(&lot_ids) {
        let sums = edit_repo::lot_sums(tx, *lot_id, None)?;
        // The rows of this posting in the database are the rows the calculation worked out.
        if edit_repo::lot_sums_of_entry(tx, *lot_id, new_entry_id)? != posting_sums(shaped)? {
            return Err(data("the saved lines of a lot differ from the preview"));
        }
        if first {
            let running = lot_running_totals(&LotCharges::of_totals(shaped.totals), sums)?;
            if shaped.lot.lot_id.is_some() {
                let person = lot_person(shaped)?;
                let lot_no = detail
                    .lots
                    .iter()
                    .find(|l| Some(l.id) == shaped.lot.lot_id)
                    .map(|l| l.lot_no)
                    .ok_or_else(|| data("a lot of an edit is not in its بکری"))?;
                let columns =
                    lot_columns(tx, detail.id, lot_no, shaped, person.as_ref(), &running)?;
                edit_repo::update_lot(tx, *lot_id, &columns, user, &now)?;
            }
            repo::set_lot_first_entry(tx, *lot_id, new_entry_id, user, &now)?;
        } else {
            let state = edit_repo::lot_state(tx, *lot_id)?
                .ok_or_else(|| data("the lot of a later posting is missing"))?;
            let running = lot_running_totals(&super::later::lot_charges_of(&state)?, sums)?;
            edit_repo::update_lot_totals(tx, *lot_id, &running, user, &now)?;
        }
    }

    // 6. The header, and the version.
    if first {
        let header = &worked.prepared.header;
        let bill_iso = header.bill_date.unwrap_or(date).to_iso();
        edit_repo::update_entry(
            tx,
            detail.id,
            &edit_repo::EntryUpdate {
                book: header.book.as_deref(),
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
            user,
            &now,
        )?;
    } else {
        edit_repo::bump_version(tx, detail.id, user, &now)?;
    }

    // 7. Matching, from the start for every key the two postings touch (Architecture 12).
    pin_cash_sales(tx, &post, &lots, layout, &line_ids)?;
    let new_stored = ledger_repo::get_entry(tx, new_entry_id)?
        .ok_or_else(|| data("the new posting cannot be read back"))?;
    for (account_id, person_key) in touched_keys(&old_entry, &new_stored) {
        if let Some((role, is_pukhta)) = role_of_account(tx, account_id)? {
            rematch_party(
                tx,
                user,
                &now,
                account_id,
                person_key.as_deref(),
                role,
                is_pukhta,
            )?;
        }
    }
    check_shares(tx, new_entry_id, worked.calc.summary.shared_income)?;

    // A trip closes when this took its last goods and opens again when the old posting had (PRD
    // FR-12.33). The old and the new posting may name different trips.
    for entry in [old_entry_id, new_entry_id] {
        crate::purchases::refresh_trips_of_entry(tx, entry, &old_posting.entry_date, user, &now)?;
    }

    // 8. Audit: the old and the new lines, and why.
    let mut event = Event::new(Some(user), "edit", "bikri")
        .entity_id(detail.id)
        .old(json!({
            "number": detail.number,
            "entry_id": old_entry_id,
            "entry_date": old_posting.entry_date,
            "lines": lines_json(&posting_lots(detail, old_entry_id, kind)),
        }))
        .new_value(json!({
            "number": detail.number,
            "entry_id": new_entry_id,
            "reversal_entry_id": reversal.entry_id,
            "entry_date": old_posting.entry_date,
            "lines": shaped_lines_json(&lots),
            "gross_paisa": worked.calc.summary.gross.as_paisa(),
            "deductions_paisa": worked.calc.summary.deductions.as_paisa(),
            "net_paisa": worked.calc.summary.net.as_paisa(),
            "buyer_total_paisa": worked.calc.summary.buyer_total.as_paisa(),
            "shared_income_paisa": worked.calc.summary.shared_income.as_paisa(),
            "cash_after_paisa": worked.cash.cash_after_paisa.as_paisa(),
            "cash_below_zero_confirmed": worked.cash.needs_confirm,
        }));
    if let Some(reason) = reason {
        event = event.reason(reason);
    }
    ctx.db.auditor().write(tx, &event.into_entry())?;

    let version = edit_repo::entry_state(tx, detail.id)?
        .map(|(_, _, version)| version)
        .ok_or_else(|| data("the بکری is missing after its edit"))?;
    Ok(BikriEdited {
        bikri_id: detail.id,
        number: detail.number,
        version,
        old_entry_id,
        reversal_entry_id: reversal.entry_id,
        entry_id: new_entry_id,
        preview: worked.preview(),
    })
}
