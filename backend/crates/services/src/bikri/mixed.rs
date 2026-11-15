//! A بکری that holds carried lots and new lots in one save (plan 76 point 1, PRD FR-12.17 v1.41).
//!
//! The save is ONE user action and ONE transaction. It writes separate journal entries, each one
//! by the code of its own standalone path, so every amount, share row and matching line is what
//! the standalone save would have written:
//!
//! 1. one `bikri_later` entry for each carried lot, on the lot's OLD بکری, in screen order (lot
//!    charges are not charged again, FR-12.17);
//! 2. one `bikri_first` entry for the new lots, on a NEW بکری number (none when there are no new
//!    lots).
//!
//! No entry mixes both kinds, and each rounds its own partner split, so the screen's total per
//! partner is the exact sum of what the entries write (money agent, plan 76 item A). A carried lot
//! with no line writes nothing. Every entry is written, and matched, before the next one: the
//! same as running the standalone saves one after the other in this fixed order.
//!
//! The cash question (PRD FR-6.7) is asked once over all entries together. Undo reverses the later
//! entries (one posting each, lot totals taken back from the stored lines) and cancels the new
//! بکری, in one transaction, with the same cash check.
//!
//! Worked numbers (the E2E-16 state, 11-10-26): 5 کیلا of رحیم to ارشد at 560 (later entry 3,008 =
//! 3,008, shares 152 / 112 / 112) plus کریم's 10 سیب at 1,200 to شاہد (new بکری 2, entry 25,600 =
//! 25,600, shares 616 / 462 / 462): shared income 1,916, partners 768 / 574 / 574, cash
//! 45,000 + 12,800 = 57,800.

use std::collections::{BTreeSet, HashSet};

use mandi_db::bikri::edit as edit_repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::{PostingKind, merge_splits, merge_summaries, sum_changes};
use mandi_domain::core::bikri::{
    BikriDraft, BikriPreview, BikriStatus, CarriedLotInput, CarriedSaved, LotInput, LotPreview,
    MixedPart, MixedPartKind, MixedSalePreview, MixedSaleSaveInput, MixedSaleSaved,
    MixedUndoCarried, MixedUndoDraft, MixedUndoInput, MixedUndoResult, SaleInput, SplitPart,
    SummaryTotals, UNDO_REASON_ID, is_blank_sale, prepare_draft,
};
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::money::Paisa;
use serde_json::json;

use super::cancel::{
    cancel_in, cash_of, closed_days, closed_error, live_postings, refuse_closed_garden,
    refuse_fruit_taken, reverse_posting_in,
};
use super::later::{LaterPlan, lot_charges_of, on_date, plan_later, write_later};
use super::load::{detail_of, draft_of, later_header};
use super::rows::lot_running_totals;
use super::save::write_new;
use super::shared::{Basis, Worked, data, switches, work_out};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::rules;
use crate::vouchers::{rematch_party, role_of_account};

/// A carried lot worked out: its plan facts and its entry.
struct CarriedWork {
    state: mandi_db::bikri::edit::LotState,
    date: Option<BusinessDate>,
    worked: Worked,
}

/// The new lots worked out.
struct NewWork {
    date: Option<BusinessDate>,
    worked: Worked,
}

/// The whole screen worked out, with the errors in the paths of the mixed screen.
struct Work {
    carried: Vec<CarriedWork>,
    new: Option<NewWork>,
    errors: Vec<FieldError>,
    cash: CashCheck,
    preview: MixedSalePreview,
}

/// `lots[0].x` becomes `carried[i].x`; `lots` (no lot line) becomes `carried[i].sales`.
fn carried_path(i: usize, field: &str) -> String {
    match field.strip_prefix("lots[0]") {
        Some(rest) => format!("carried[{i}]{rest}"),
        None if field == "lots" => format!("carried[{i}].sales"),
        None => field.to_owned(),
    }
}

/// `lots[k].x` becomes `new_lots[k].x`; `lots` becomes `new_lots`.
fn new_path(field: &str) -> String {
    match field.strip_prefix("lots") {
        Some(rest) if rest.is_empty() || rest.starts_with('[') => format!("new_lots{rest}"),
        _ => field.to_owned(),
    }
}

fn remap(errors: Vec<FieldError>, path: impl Fn(&str) -> String) -> Vec<FieldError> {
    errors
        .into_iter()
        .map(|mut e| {
            e.field = path(&e.field);
            e
        })
        .collect()
}

/// Adds errors once: two entries of one screen report the same closed day, or the same shares
/// problem, and the screen needs it once.
fn push_unique(into: &mut Vec<FieldError>, more: Vec<FieldError>) {
    for e in more {
        if !into.iter().any(|x| x.code == e.code && x.field == e.field) {
            into.push(e);
        }
    }
}

/// Works the whole screen out (the one path of the preview and the save, Code Standards B5).
fn plan_all(
    conn: &Connection,
    input: (&str, &[CarriedLotInput], &[LotInput]),
    window: (Option<BusinessDate>, BusinessDate),
    rule_set: &mandi_domain::rules::RuleSet,
    switches: mandi_domain::bikri::Switches,
) -> Result<Work, ServiceError> {
    let (date_text, carried_in, new_lots) = input;
    let (closed_up_to, today) = window;
    let mut errors: Vec<FieldError> = Vec::new();
    let mut warnings: Vec<FieldError> = Vec::new();
    let mut carried: Vec<CarriedWork> = Vec::new();
    let mut parts: Vec<MixedPart> = Vec::new();
    let mut lots: Vec<LotPreview> = Vec::new();
    let mut seen: HashSet<i64> = HashSet::new();

    for (i, item) in carried_in.iter().enumerate() {
        if !seen.insert(item.lot_id) {
            errors.push(FieldError::new(
                ErrorCode::ErrDuplicateLot,
                &format!("carried[{i}].lot_id"),
            ));
            continue;
        }
        // A carried lot with no line writes nothing.
        if item.sales.iter().all(is_blank_sale) {
            continue;
        }
        let plan = match plan_later(conn, item.lot_id, date_text, &item.sales, window) {
            Ok(plan) => plan,
            Err(ServiceError::Rejected(rejections)) => {
                let rejected = rejections
                    .into_iter()
                    .map(|r| FieldError {
                        code: r.code,
                        field: match r.field.as_deref() {
                            Some("lot_id") | None => format!("carried[{i}].lot_id"),
                            Some(other) => format!("carried[{i}].{other}"),
                        },
                        values: r.values,
                    })
                    .collect();
                push_unique(&mut errors, rejected);
                continue;
            }
            Err(other) => return Err(other),
        };
        let LaterPlan {
            state,
            date,
            prepared,
            basis,
            extra,
        } = plan;
        let worked = work_out(conn, prepared, rule_set, switches, &basis)?;
        let mut found = on_date(worked.errors.clone());
        found.extend(extra);
        push_unique(&mut errors, remap(found, |f| carried_path(i, f)));
        push_unique(
            &mut warnings,
            remap(on_date(worked.calc.warnings.clone()), |f| {
                carried_path(i, f)
            }),
        );
        // The lot row keeps the key the screen sent, so the screen finds its row again.
        let key = if item.key.is_empty() {
            format!("lot:{}", state.id)
        } else {
            item.key.clone()
        };
        lots.extend(worked.calc.lots.clone().into_iter().map(|mut lot| {
            lot.key.clone_from(&key);
            lot
        }));
        parts.push(MixedPart {
            key: Some(key),
            kind: MixedPartKind::Carried,
            lot_id: Some(state.id),
            old_bikri_id: Some(state.bikri_id),
            old_number: Some(state.bikri_number),
            summary: worked.calc.summary.clone(),
            split: worked.split.clone(),
        });
        carried.push(CarriedWork {
            state,
            date,
            worked,
        });
    }

    let new = if new_lots.is_empty() {
        None
    } else {
        let mut header = later_header(today);
        header.entry_date = date_text.to_owned();
        let draft = BikriDraft {
            header,
            lots: new_lots.to_vec(),
        };
        let prepared = prepare_draft(&draft, today, closed_up_to);
        let date = prepared.header.entry_date;
        let worked = work_out(conn, prepared, rule_set, switches, &Basis::new_bikri())?;
        push_unique(&mut errors, remap(on_date(worked.errors.clone()), new_path));
        push_unique(
            &mut warnings,
            remap(on_date(worked.calc.warnings.clone()), new_path),
        );
        lots.extend(worked.calc.lots.clone());
        parts.push(MixedPart {
            key: None,
            kind: MixedPartKind::New,
            lot_id: None,
            old_bikri_id: None,
            old_number: None,
            summary: worked.calc.summary.clone(),
            split: worked.split.clone(),
        });
        Some(NewWork { date, worked })
    };

    if carried.is_empty() && new.is_none() && errors.is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrNoSaleLines, "carried"));
    }

    // The screen's totals: sums of the parts, with checked maths (never rounded again).
    let lot_count = i64::try_from(lots.len()).map_err(|_| data("too many lots"))?;
    let summaries: Vec<&SummaryTotals> = parts.iter().map(|p| &p.summary).collect();
    let summary = merge_summaries(&summaries, lot_count).map_err(money_error)?;
    let splits: Vec<&[SplitPart]> = parts.iter().map(|p| p.split.as_slice()).collect();
    let split = merge_splits(&splits).map_err(money_error)?;
    // One cash question over every entry together (PRD FR-6.7).
    let changes: Vec<Paisa> = carried
        .iter()
        .map(|c| c.worked.cash.change_paisa)
        .chain(new.iter().map(|n| n.worked.cash.change_paisa))
        .collect();
    let cash = cash::cash_after_entry(conn, sum_changes(&changes).map_err(money_error)?)?;
    let preview = MixedSalePreview {
        preview: BikriPreview {
            lots,
            summary,
            split,
            cash,
            allow_line_override: switches.allow_line_override,
            errors: errors.clone(),
            warnings: warnings.clone(),
        },
        parts,
    };
    Ok(Work {
        carried,
        new,
        errors,
        cash,
        preview,
    })
}

/// Works out a whole mixed screen for the live summary. Nothing is saved.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix.
/// The mistakes of the screen come back inside the preview (`errors`).
pub fn preview(
    ctx: &Context,
    input: &mandi_domain::core::bikri::MixedSaleInput,
) -> Result<MixedSalePreview, ServiceError> {
    let window = day::window(ctx)?;
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    ctx.db.read(|conn| {
        let work = plan_all(
            conn,
            (&input.date, &input.carried, &input.new_lots),
            window,
            &rule_set,
            switches,
        )?;
        Ok(work.preview)
    })
}

/// Saves a mixed screen in one transaction: see the module text.
///
/// # Errors
/// The errors of [`preview`], each at its field and all together; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`
/// (value `cash_after_paisa`, over all entries together). Nothing is written when any is returned.
pub fn save(
    ctx: &Context,
    actor: &Actor,
    input: &MixedSaleSaveInput,
) -> Result<MixedSaleSaved, ServiceError> {
    let window = day::window(ctx)?;
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    let now = ctx.now_text();
    let saved = ctx.db.write(|tx| {
        let work = plan_all(
            tx,
            (&input.date, &input.carried, &input.new_lots),
            window,
            &rule_set,
            switches,
        )?;
        if !work.errors.is_empty() {
            return Err(ServiceError::from(work.errors.clone()));
        }
        cash::require_confirmed(&work.cash, input.confirm_cash_below_zero)?;
        let mut entry_ids = Vec::new();
        let mut carried_saved = Vec::new();
        let mut date_iso = String::new();
        for c in &work.carried {
            let date = c
                .date
                .ok_or_else(|| data("a mixed save without errors has no date"))?;
            date_iso = date.to_iso();
            let entry_id = write_later(ctx, tx, actor, &now, &c.state, date, &c.worked)?;
            entry_ids.push(entry_id);
            carried_saved.push(CarriedSaved {
                lot_id: c.state.id,
                old_bikri_id: c.state.bikri_id,
                old_number: c.state.bikri_number,
                entry_id,
            });
        }
        let (mut new_id, mut new_number) = (None, None);
        if let Some(n) = &work.new {
            let date = n
                .date
                .ok_or_else(|| data("a mixed save without errors has no date"))?;
            date_iso = date.to_iso();
            let written = write_new(ctx, tx, actor, &now, &n.worked)?;
            entry_ids.push(written.entry_id);
            new_id = Some(written.bikri_id);
            new_number = Some(written.number);
        }
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "mixed_save", "bikri")
                .new_value(json!({
                    "entry_date": date_iso,
                    "entries": entry_ids,
                    "new_bikri_id": new_id,
                    "new_number": new_number,
                    "carried_lots": carried_saved.iter().map(|c| c.lot_id).collect::<Vec<_>>(),
                    "cash_after_paisa": work.cash.cash_after_paisa.as_paisa(),
                    "cash_below_zero_confirmed": work.cash.needs_confirm,
                }))
                .into_entry(),
        )?;
        // The بکری this save sold: the new one first, then the old one of each carried lot.
        let mut print_bikri_ids: Vec<i64> = new_id.into_iter().collect();
        for carried in &carried_saved {
            if !print_bikri_ids.contains(&carried.old_bikri_id) {
                print_bikri_ids.push(carried.old_bikri_id);
            }
        }
        Ok(MixedSaleSaved {
            date: date_iso,
            new_bikri_id: new_id,
            new_bikri_number: new_number,
            carried: carried_saved,
            entry_ids,
            print_bikri_ids,
            preview: work.preview.preview,
        })
    })?;
    let chukta = input
        .carried
        .iter()
        .flat_map(|c| c.sales.iter())
        .chain(input.new_lots.iter().flat_map(|l| l.sales.iter()))
        .any(|s| s.chukta.is_some());
    if chukta {
        crate::search::refresh_chukta_all(ctx);
    }
    Ok(saved)
}

/// The lines a later entry wrote, as drafts, for the undo.
fn carried_draft(
    tx: &Connection,
    bikri_id: i64,
    entry_id: i64,
    date: BusinessDate,
    lot_id: i64,
) -> Result<MixedUndoCarried, ServiceError> {
    let detail = detail_of(tx, bikri_id, "later_entry_ids")?;
    let draft = draft_of(&detail, entry_id, date, PostingKind::Later, false);
    let sales: Vec<SaleInput> = draft.lots.into_iter().flat_map(|l| l.sales).collect();
    Ok(MixedUndoCarried { lot_id, sales })
}

/// Undoes a mixed save straight after it: see the module text.
///
/// # Errors
/// `ERR_NOT_FOUND` on `later_entry_ids` for an entry that is not a later posting in force;
/// `ERR_BIKRI_CANCELLED`, `ERR_GARDEN_CLOSED`, `ERR_BIKRI_HAS_FRUIT` on `new_bikri_id`;
/// `ERR_DATE_CLOSED` when a day is closed; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written
/// when any is returned.
#[allow(clippy::too_many_lines)]
pub fn undo(
    ctx: &Context,
    actor: &Actor,
    input: &MixedUndoInput,
) -> Result<MixedUndoResult, ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    let now = ctx.now_text();
    let user = actor.user_id;
    ctx.db.write(|tx| {
        // 1. Every later entry is a posting in force, named once.
        let mut later: Vec<(i64, i64, BusinessDate, i64)> = Vec::new(); // entry, bikri, date, lot
        let mut named: HashSet<i64> = HashSet::new();
        for (i, entry_id) in input.later_entry_ids.iter().enumerate() {
            let field = format!("later_entry_ids[{i}]");
            if !named.insert(*entry_id) {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrDuplicateLot,
                    &field,
                ));
            }
            let entry = ledger_repo::get_entry(tx, *entry_id)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, &field))?;
            if entry.posting_type != "bikri_later" || entry.source_type != "bikri" {
                return Err(ServiceError::reject_field(ErrorCode::ErrNotFound, &field));
            }
            let in_force = edit_repo::postings_in_force(tx, entry.source_id)?
                .iter()
                .any(|p| p.entry_id == *entry_id);
            if !in_force {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrBikriCancelled,
                    &field,
                ));
            }
            let date = BusinessDate::parse_iso(&entry.entry_date)
                .ok_or_else(|| data("the date of a posting is not a date"))?;
            let lot_id = edit_repo::active_sales_of_entry(tx, *entry_id)?
                .first()
                .map(|s| s.lot_id)
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, &field))?;
            // A lot of a closed garden is not changed until the closing is undone (PRD FR-12.38).
            refuse_closed_garden(tx, entry.source_id)?;
            later.push((*entry_id, entry.source_id, date, lot_id));
        }
        // 2. The new بکری, if any.
        let new_detail = match input.new_bikri_id {
            Some(id) => {
                let detail = detail_of(tx, id, "new_bikri_id")?;
                if detail.status == BikriStatus::Cancelled {
                    return Err(ServiceError::reject_field(
                        ErrorCode::ErrBikriCancelled,
                        "new_bikri_id",
                    ));
                }
                refuse_closed_garden(tx, detail.id)?;
                refuse_fruit_taken(tx, detail.id)?;
                Some(detail)
            }
            None => None,
        };
        // 3. Closed days, and the cash question over everything together (PRD FR-6.7).
        let mut live_all = Vec::new();
        for (entry_id, _, date, _) in &later {
            live_all.push(super::cancel::Live {
                entry_id: *entry_id,
                date: *date,
            });
        }
        if let Some(detail) = &new_detail {
            live_all.extend(live_postings(tx, detail.id)?);
        }
        if let Some(day) = closed_days(&live_all, closed_up_to).first() {
            return Err(closed_error(ctx, *day, "later_entry_ids"));
        }
        let change = cash_of(tx, &live_all)?.checked_neg().map_err(money_error)?;
        cash::require_confirmed(
            &cash::cash_after_entry(tx, change)?,
            input.confirm_cash_below_zero,
        )?;

        // 4. The values to give back, before anything is reversed.
        let mut draft_carried = Vec::new();
        for (entry_id, bikri_id, date, lot_id) in &later {
            draft_carried.push(carried_draft(tx, *bikri_id, *entry_id, *date, *lot_id)?);
        }
        let mut new_draft = None;
        if let Some(detail) = &new_detail {
            let first = edit_repo::postings_in_force(tx, detail.id)?
                .into_iter()
                .find(|p| p.posting_type == "bikri_first")
                .ok_or_else(|| {
                    ServiceError::reject_field(ErrorCode::ErrBikriCancelled, "new_bikri_id")
                })?;
            let date = BusinessDate::parse_iso(&first.entry_date)
                .ok_or_else(|| data("the date of a posting is not a date"))?;
            new_draft = Some(draft_of(
                detail,
                first.entry_id,
                date,
                PostingKind::First,
                false,
            ));
        }

        // 5. Reverse each later entry, then take the lot totals back from the stored lines.
        let mut touched: BTreeSet<(i64, Option<String>)> = BTreeSet::new();
        let mut reversals = Vec::new();
        let mut lots: BTreeSet<i64> = BTreeSet::new();
        let mut bikris: BTreeSet<i64> = BTreeSet::new();
        for (entry_id, bikri_id, date, lot_id) in &later {
            let (reversal_id, keys) =
                reverse_posting_in(ctx, tx, user, &now, *entry_id, *date, UNDO_REASON_ID)?;
            touched.extend(keys);
            reversals.push(reversal_id);
            lots.insert(*lot_id);
            bikris.insert(*bikri_id);
        }
        for lot_id in &lots {
            let state = edit_repo::lot_state(tx, *lot_id)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrLotNotFound, "lot_id"))?;
            let sums = edit_repo::lot_sums(tx, state.id, None)?;
            let running = lot_running_totals(&lot_charges_of(&state)?, sums)?;
            edit_repo::update_lot_totals(tx, state.id, &running, user, &now)?;
        }
        for bikri_id in &bikris {
            edit_repo::bump_version(tx, *bikri_id, user, &now)?;
        }
        // Matching of the old بکری's buyers again, oldest first (PRD FR-12.22).
        for (account_id, person_key) in touched {
            if let Some(role) = role_of_account(tx, account_id)? {
                rematch_party(tx, user, &now, account_id, person_key.as_deref(), role)?;
            }
        }
        // 6. Cancel the new بکری with the fixed undo reason; the cash question was asked above.
        if let Some(detail) = &new_detail {
            reversals.extend(cancel_in(ctx, tx, actor, detail, UNDO_REASON_ID, true)?);
        }
        let date_text = match later.first() {
            Some((_, _, d, _)) => mandi_domain::core::dates::format_ddmmyyyy(*d),
            None => new_draft
                .as_ref()
                .map(|d| d.header.entry_date.clone())
                .unwrap_or_default(),
        };
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "mixed_undo", "bikri")
                .new_value(json!({
                    "reversals": reversals,
                    "later_entries": input.later_entry_ids,
                    "new_bikri_id": input.new_bikri_id,
                    "cash_change_paisa": change.as_paisa(),
                }))
                .reason(UNDO_REASON_ID)
                .into_entry(),
        )?;
        Ok(MixedUndoResult {
            reversal_entry_ids: reversals,
            cancelled_bikri_id: input.new_bikri_id,
            draft: MixedUndoDraft {
                date: date_text,
                carried: draft_carried,
                new_draft,
            },
        })
    })
}
