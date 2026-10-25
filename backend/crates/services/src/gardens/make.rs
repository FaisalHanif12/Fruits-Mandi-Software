//! Making a garden and changing its partners before its first entry (PRD 12.11, FR-12.29,
//! FR-12.34): the partners and their shares first, then the money source of each, then the details.
//! One action is one transaction with its audit row (Code Standards B3, S7). A garden gets its own
//! account under control 0306 with both names, and its first installment if one is typed.
//!
//! The preview and the save of a new garden share one checking and working out ([`plan`], Code
//! Standards B5).

use std::collections::HashMap;

use mandi_db::gardens::{self as repo, NewGarden};
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as vrepo, NewEntry, PartnerShare};
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::gardens::{
    GardenCreatePreview, GardenCreated, GardenDefaults, GardenInput, GardenRow,
    GardenSetPartnersInput,
};
use mandi_domain::core::masters::AccountToMake;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::text;
use mandi_domain::core::translit::Kind;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::ventures::{MoneySource, VenturePartnerRow};
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::json;

use super::data;
use super::read::{load, row_by_id};
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::ServiceError;
use crate::search;
use crate::spelling::{self, NamePair};
use crate::ventures::{
    self, Parsed, PaymentPlan, Settlement, not_found, partners_json, parts_view_of, plan_payment,
    read_partners, record_of, resolve_sources_of, write_partners,
};

/// An optional free text: empty is none.
fn optional_text(errors: &mut FieldErrors, field: &str, text: Option<&str>) -> Option<String> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// The form of a new garden (PRD A56): every active partner with the firm's share of today and the
/// source `shop`.
pub(super) fn defaults(ctx: &Context) -> Result<GardenDefaults, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let active = vrepo::active_partner_ids(conn)?;
        let names: HashMap<i64, (String, String)> = mandi_db::partners::list(conn)?
            .into_iter()
            .map(|p| (p.id, (p.name_ur, p.name_en)))
            .collect();
        let firm: HashMap<i64, i64> = mandi_db::partners::shares_on(conn, &today.to_iso())?
            .into_iter()
            .collect();
        let mut rows = Vec::with_capacity(active.len());
        for id in active {
            let (name_ur, name_en) = names.get(&id).cloned().unwrap_or_default();
            rows.push(VenturePartnerRow {
                partner: record_of(id, &name_ur, &name_en),
                share_bps: i32::try_from(firm.get(&id).copied().unwrap_or(0))
                    .map_err(|_| data("a stored share is out of range"))?,
                source: MoneySource::Shop,
            });
        }
        Ok(GardenDefaults { partners: rows })
    })
}

/// The first installment of a new garden, worked out.
struct First {
    amount: Paisa,
    reason: String,
    plan: PaymentPlan,
}

/// A new garden after the checks.
struct Ready {
    date: BusinessDate,
    name_ur: String,
    name_en: String,
    item_id: Option<i64>,
    rent: Paisa,
    season_end: Option<BusinessDate>,
    note: Option<String>,
    partners: Vec<Parsed>,
    shares: Vec<PartnerShare>,
    first: Option<First>,
}

/// What [`plan`] found.
struct Plan {
    errors: Vec<FieldError>,
    ready: Option<Ready>,
    cash: CashCheck,
}

/// The partners of the new garden as the shared venture helpers read them: with both names and the
/// 0604 account of each.
fn shares_of(conn: &Connection, partners: &[Parsed]) -> Result<Vec<PartnerShare>, ServiceError> {
    let records: HashMap<i64, mandi_db::partners::PartnerRecord> = mandi_db::partners::list(conn)?
        .into_iter()
        .map(|p| (p.id, p))
        .collect();
    partners
        .iter()
        .map(|p| {
            let record = records
                .get(&p.partner_id)
                .ok_or_else(|| data("a partner of a new garden was lost"))?;
            Ok(PartnerShare {
                partner_id: p.partner_id,
                name_ur: record.name_ur.clone(),
                name_en: record.name_en.clone(),
                share_bps: i64::from(p.share.get()),
                source: p.source.as_str().to_owned(),
                venture_account_id: record.venture_account_id,
            })
        })
        .collect()
}

/// Checks and works out a new garden. Nothing is written.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix. The user's mistakes
/// come back in [`Plan::errors`].
fn plan(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &GardenInput,
) -> Result<Plan, ServiceError> {
    let (closed_up_to, today) = window;
    let mut errors = FieldErrors::default();
    let name_ur = errors.take(validate::name_ur("name_ur", &input.name_ur));
    let name_en = errors.take(validate::name_en("name_en", &input.name_en));
    let date = errors.take(validate::posting_date(
        "date",
        &input.date,
        closed_up_to,
        today,
    ));
    let rent = errors
        .take(validate::money("rent", &input.rent))
        .filter(|rent| {
            let positive = *rent > Paisa::ZERO;
            if !positive {
                errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, "rent"));
            }
            positive
        });
    let season_end = match input
        .season_end
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        Some(text) => errors.take(validate::date("season_end", text, today)),
        None => None,
    };
    if let (Some(end), Some(start)) = (season_end, date)
        && end < start
    {
        errors.push(
            FieldError::new(ErrorCode::ErrDateBeforeGarden, "season_end")
                .with("garden_date", ErrorValue::Text(start.to_iso())),
        );
    }
    let note = optional_text(&mut errors, "note", input.note.as_deref());
    if let Some(item_id) = input.item_id {
        match mandi_db::bikri::items(conn, &[item_id])?.into_iter().next() {
            None => errors.push(FieldError::new(ErrorCode::ErrNotFound, "item_id")),
            Some(item) if !item.is_active => {
                errors.push(FieldError::new(ErrorCode::ErrAccountInactive, "item_id"));
            }
            Some(_) => {}
        }
    }
    let paid_now = match input
        .paid_now
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        Some(text) => errors.take(validate::money("paid_now", text)).filter(|a| {
            let positive = *a > Paisa::ZERO;
            if !positive {
                errors.push(FieldError::new(
                    ErrorCode::ErrMustBeMoreThanZero,
                    "paid_now",
                ));
            }
            positive
        }),
        None => None,
    };
    let paid_now_reason = optional_text(
        &mut errors,
        "paid_now_reason",
        input.paid_now_reason.as_deref(),
    );
    if paid_now.is_some() && paid_now_reason.is_none() {
        errors.push(FieldError::new(
            ErrorCode::ErrReasonRequired,
            "paid_now_reason",
        ));
    }
    let mut found = errors.finish().err().unwrap_or_default();
    let partners = read_partners(conn, &mut found, &input.partners)?;
    let shares = shares_of(conn, &partners)?;
    // The first installment: the parts by the garden's shares (C23), with the source of each partner
    // for this entry.
    let mut part_errors = FieldErrors::default();
    let first = match (paid_now, found.is_empty()) {
        (Some(amount), true) => {
            match resolve_sources_of(&shares, &input.parts, &mut part_errors)? {
                Some(chosen) => {
                    let payment = plan_payment(conn, amount, &chosen)?;
                    Some(First {
                        amount,
                        reason: paid_now_reason.clone().unwrap_or_default(),
                        plan: payment,
                    })
                }
                None => None,
            }
        }
        _ => None,
    };
    found.extend(part_errors.finish().err().unwrap_or_default());
    let cash = match &first {
        Some(first) => first.plan.cash,
        None => crate::cash::cash_after_entry(conn, Paisa::ZERO)?,
    };
    let (Some(date), Some(name_ur), Some(name_en), Some(rent)) = (date, name_ur, name_en, rent)
    else {
        return Ok(Plan {
            errors: found,
            ready: None,
            cash,
        });
    };
    if !found.is_empty() {
        return Ok(Plan {
            errors: found,
            ready: None,
            cash,
        });
    }
    Ok(Plan {
        errors: found,
        ready: Some(Ready {
            date,
            name_ur,
            name_en,
            item_id: input.item_id,
            rent,
            season_end,
            note,
            partners,
            shares,
            first,
        }),
        cash,
    })
}

fn preview_of(plan: &Plan) -> Result<GardenCreatePreview, ServiceError> {
    let first_installment = match plan.ready.as_ref().and_then(|r| r.first.as_ref()) {
        Some(first) => {
            let ready = plan
                .ready
                .as_ref()
                .ok_or_else(|| data("a first installment without a garden"))?;
            Some(parts_view_of(&ready.shares, &first.plan.parts)?)
        }
        None => None,
    };
    Ok(GardenCreatePreview {
        first_installment,
        cash: plan.cash,
        errors: plan.errors.clone(),
        warnings: Vec::new(),
    })
}

/// Works out a new garden's first installment for the live form. Nothing is saved.
pub(super) fn create_preview(
    ctx: &Context,
    input: &GardenInput,
) -> Result<GardenCreatePreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| preview_of(&plan(conn, window, input)?))
}

/// Makes a garden: its account under 0306, its row, its partners and, if typed, its first installment.
///
/// Example: garden 1 "شاہ پور باغ" with a rent of 50,000 gets the account `03060001`; the first
/// installment of 30,000 at 60 / 40 with الف from the shop and ب from his pocket takes 18,000 off
/// cash and records 12,000 as paid from his pocket.
pub(super) fn create(
    ctx: &Context,
    actor: &Actor,
    input: &GardenInput,
) -> Result<GardenCreated, ServiceError> {
    let window = day::window(ctx)?;
    // The checks that need no transaction come first; the spelling check needs the typed names.
    let (checked_plan, checked) = ctx.db.read(|conn| {
        let planned = plan(conn, window, input)?;
        Ok::<_, ServiceError>((planned.errors.is_empty(), planned))
    })?;
    if !checked_plan {
        return Err(ServiceError::from(checked.errors.clone()));
    }
    let names = checked
        .ready
        .as_ref()
        .map(|r| (r.name_ur.clone(), r.name_en.clone()))
        .ok_or_else(|| data("a garden without errors has no names"))?;
    let spelled = spelling::check(
        ctx,
        &NamePair {
            field_ur: "name_ur",
            name_ur: &names.0,
            field_en: "name_en",
            name_en: &names.1,
            kind: Kind::Place,
        },
        &input.confirmed_rules_words,
    )?;
    let now = ctx.now_text();
    let (created, account_id, learned) = ctx.db.write(|tx| {
        // The same checks again inside the transaction: the day or the cash may have moved.
        let planned = plan(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(ServiceError::from(planned.errors.clone()));
        }
        crate::cash::require_confirmed(&planned.cash, input.confirm_cash_below_zero)?;
        let preview = preview_of(&planned)?;
        let ready = planned
            .ready
            .ok_or_else(|| data("a garden without errors has no numbers"))?;
        let user = actor.user_id;
        let date_iso = ready.date.to_iso();
        let account = AccountToMake {
            control_code: "0306",
            search_ur: text::search_ur(&ready.name_ur),
            search_en: text::search_en(&ready.name_en),
            search_phonetic: text::search_phonetic(&ready.name_en),
            name_ur: ready.name_ur.clone(),
            name_en: ready.name_en.clone(),
        };
        let (account_id, account_code) =
            mandi_db::masters::create_account(tx, &account, user, &now)?.ok_or_else(|| {
                ServiceError::reject_with(ErrorCode::ErrSeriesFull, Some("name_ur"), "control", 0)
            })?;
        let number = vrepo::next_number(tx, "garden")?;
        let season_end = ready.season_end.map(BusinessDate::to_iso);
        let id = repo::insert(
            tx,
            &NewGarden {
                number,
                entry_date: &date_iso,
                name_ur: &ready.name_ur,
                name_en: &ready.name_en,
                search_ur: &text::search_ur(&ready.name_ur),
                search_en: &text::search_en(&ready.name_en),
                search_phonetic: &text::search_phonetic(&ready.name_en),
                item_id: ready.item_id,
                rent_paisa: ready.rent.as_paisa(),
                season_end_date: season_end.as_deref(),
                account_id,
                note: ready.note.as_deref(),
            },
            user,
            &now,
        )?;
        write_partners(tx, id, &ready.partners, user, &now)?;
        let learned = spelled.learn(ctx, actor, tx)?;
        // The first installment is the garden's first entry; the shares are locked by it.
        let (first_entry_id, journal_entry_id) = match &ready.first {
            Some(first) => {
                let entry_number = vrepo::next_entry_number(tx, "installment")?;
                let entry_id = vrepo::insert_entry(
                    tx,
                    &NewEntry {
                        venture_id: id,
                        kind: "installment",
                        entry_date: &date_iso,
                        amount_paisa: first.amount.as_paisa(),
                        reason: Some(&first.reason),
                        purchase_id: None,
                        qty: None,
                        shop_name: None,
                        cost_paisa: 0,
                    },
                    entry_number,
                    user,
                    &now,
                )?;
                ventures::write_parts(tx, user, &now, entry_id, &first.plan.parts)?;
                let journal_id = ventures::post_settlement(
                    ctx,
                    tx,
                    user,
                    &now,
                    &Settlement {
                        venture_entry_id: entry_id,
                        date: ready.date,
                        entry: first.plan.entry.as_ref(),
                        posting_type: PostingType::GardenPayment,
                        source_type: SourceType::Venture,
                        source_id: entry_id,
                    },
                )?;
                ctx.db.auditor().write(
                    tx,
                    &Event::new(Some(user), "create", "garden_installment")
                        .entity_id(entry_id)
                        .new_value(json!({
                            "garden_id": id,
                            "entry_date": date_iso,
                            "amount_paisa": first.amount.as_paisa(),
                            "journal_entry_id": journal_id,
                            "first": true,
                        }))
                        .into_entry(),
                )?;
                (Some(entry_id), journal_id)
            }
            None => (None, None),
        };
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "create", "garden")
                .entity_id(id)
                .new_value(json!({
                    "number": number,
                    "entry_date": date_iso,
                    "name_ur": ready.name_ur,
                    "name_en": ready.name_en,
                    "account_id": account_id,
                    "account_code": account_code,
                    "rent_paisa": ready.rent.as_paisa(),
                    "season_end_date": season_end,
                    "item_id": ready.item_id,
                    "partners": partners_json(&ready.partners),
                    "cash_after_paisa": planned.cash.cash_after_paisa.as_paisa(),
                    "cash_below_zero_confirmed": planned.cash.needs_confirm,
                }))
                .into_entry(),
        )?;
        let row = row_by_id(tx, id)?;
        Ok((
            GardenCreated {
                row,
                first_entry_id,
                journal_entry_id,
                preview,
            },
            account_id,
            learned,
        ))
    })?;
    learned.after_commit(ctx);
    search::refresh_account(ctx, account_id);
    Ok(created)
}

/// Changes the partners and shares of a garden that has no entry yet (PRD FR-12.29). After its first
/// entry they are fixed.
pub(super) fn set_partners(
    ctx: &Context,
    actor: &Actor,
    input: &GardenSetPartnersInput,
) -> Result<GardenRow, ServiceError> {
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let garden = load(tx, input.id)?.ok_or_else(|| not_found("id"))?;
        if garden.rec.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        // Locked by an entry of the garden or by any posted line on its account, for example the
        // seller credit of a lot sold into it (PRD E66).
        if repo::shares_locked(tx, garden.rec.id)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVentureSharesLocked,
                "partners",
            ));
        }
        let mut found = Vec::new();
        let partners = read_partners(tx, &mut found, &input.partners)?;
        if !found.is_empty() {
            return Err(ServiceError::from(found));
        }
        let old = vrepo::partners_of(tx, garden.rec.id)?;
        vrepo::delete_partners(tx, garden.rec.id)?;
        write_partners(tx, garden.rec.id, &partners, actor.user_id, &now)?;
        vrepo::touch(tx, garden.rec.id, actor.user_id, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "garden")
                .entity_id(garden.rec.id)
                .old(json!({
                    "partners": old
                        .iter()
                        .map(|p| json!({
                            "partner_id": p.partner_id,
                            "share_bps": p.share_bps,
                            "source": p.source,
                        }))
                        .collect::<Vec<_>>(),
                }))
                .new_value(json!({ "partners": partners_json(&partners) }))
                .into_entry(),
        )?;
        row_by_id(tx, garden.rec.id)
    })
}
