//! Partners and their shares (PRD 17.1 to 17.3, FR-6.5, plan 11): add and edit a partner, set the
//! shares of all partners from a day, list, read, and the overview.
//!
//! One user action is one transaction (CLAUDE.md rule 3): a partner, his two accounts, his
//! opening capital, the pairs the user confirmed in the spelling check and the audit rows are
//! written together or not at all. The shares of a posting are read on the posting date, and the
//! split itself is in `domain::partners`; the one place that writes share rows is
//! [`rows`], called by `ledger::post` (PRD FR-6.5).
//!
//! Partners do not log in (PRD FR-2.4): no user is made for a partner.

use std::collections::HashMap;

use mandi_db::masters as master_repo;
use mandi_db::partners as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::OpeningRequest;
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::masters::AccountToMake;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::partners::{
    CAPITAL_CONTROL, DRAWINGS_CONTROL, MAX_PARTNERS, PartnerCreateInput, PartnerDetail,
    PartnerOverviewRow, PartnerRow, PartnerUpdateInput, PartnersCheckSharesInput,
    PartnersSetSharesInput, ShareHistoryRow, SharesCheck, check_listed, check_shares, not_100,
    read_shares, venture_account,
};
use mandi_domain::core::text;
use mandi_domain::core::translit::Kind;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::partners::ShareKind;
use serde_json::{Value, json};

pub(crate) mod money;
pub(crate) mod rows;
mod statement;

pub use money::{capital_history, takes};
pub use statement::statement;

pub use rows::split_entry;

use crate::accounts::apply_opening;
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger;
use crate::search;
use crate::spelling::{self, NamePair};

/// The words of the names of a partner's two accounts (PRD FR-10.4: "Capital, <partner name>"
/// and "Drawings, <partner name>"). The seeded controls 0602 and 0603 use the same words. A
/// hyphen joins the word and the name, because a comma is not allowed in a name (PRD FR-7.28).
const CAPITAL_WORD_UR: &str = "سرمایہ";
const CAPITAL_WORD_EN: &str = "Capital";
const DRAWINGS_WORD_UR: &str = "ذاتی خرچہ";
const DRAWINGS_WORD_EN: &str = "Drawings";

fn share_i32(bps: i64) -> Result<i32, ServiceError> {
    i32::try_from(bps).map_err(|_| ServiceError::Data("a stored share is out of range".to_owned()))
}

fn row_of(record: &repo::PartnerRecord, share_bps: i32) -> PartnerRow {
    PartnerRow {
        id: record.id,
        name_ur: record.name_ur.clone(),
        name_en: record.name_en.clone(),
        capital_account_id: record.capital_account_id,
        capital_account_code: record.capital_account_code.clone(),
        drawings_account_id: record.drawings_account_id,
        drawings_account_code: record.drawings_account_code.clone(),
        venture_account_id: record.venture_account_id,
        venture_account_code: record.venture_account_code.clone(),
        share_bps,
        is_active: record.is_active,
        version: record.version,
    }
}

/// All partners with their share on `today_iso`: active first, then by id.
fn rows_in(conn: &Connection, today_iso: &str) -> Result<Vec<PartnerRow>, ServiceError> {
    let shares: HashMap<i64, i64> = repo::shares_on(conn, today_iso)?.into_iter().collect();
    repo::list(conn)?
        .iter()
        .map(|record| {
            let bps = shares.get(&record.id).copied().unwrap_or(0);
            Ok(row_of(record, share_i32(bps)?))
        })
        .collect()
}

fn not_found(field: &str) -> ServiceError {
    ServiceError::reject_field(ErrorCode::ErrNotFound, field)
}

fn reject_value(
    code: ErrorCode,
    field: Option<&str>,
    key: &str,
    value: ErrorValue,
) -> ServiceError {
    ServiceError::Rejected(vec![Rejection {
        code,
        field: field.map(str::to_owned),
        values: std::collections::BTreeMap::from([(key.to_owned(), value)]),
    }])
}

fn too_many() -> ServiceError {
    let max = i64::try_from(MAX_PARTNERS).unwrap_or(i64::MAX);
    ServiceError::reject_with(ErrorCode::ErrTooManyPartners, None, "max", max)
}

/// Every partner, active first, then by id. `share_bps` is the share on today's date.
///
/// # Errors
/// `ServiceError::Db`.
pub fn list(ctx: &Context) -> Result<Vec<PartnerRow>, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| rows_in(conn, &today.to_iso()))
}

/// One partner with his share history, oldest first.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, id: i64) -> Result<PartnerDetail, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let record = repo::get(conn, id)?.ok_or_else(|| not_found("id"))?;
        let today_iso = today.to_iso();
        let share = repo::shares_on(conn, &today_iso)?
            .into_iter()
            .find(|&(partner, _)| partner == id)
            .map_or(0, |(_, bps)| bps);
        let shares = repo::share_history(conn, id)?
            .into_iter()
            .map(|h| {
                Ok(ShareHistoryRow {
                    share_bps: share_i32(h.share_bps)?,
                    effective_from: h.effective_from,
                })
            })
            .collect::<Result<Vec<_>, ServiceError>>()?;
        Ok(PartnerDetail {
            row: row_of(&record, share_i32(share)?),
            shares,
        })
    })
}

/// The account a partner's name makes: `<word> - <name>` in both scripts, with the search keys.
fn account_to_make(
    control_code: &'static str,
    word_ur: &str,
    word_en: &str,
    name_ur: &str,
    name_en: &str,
) -> AccountToMake {
    let ur = format!("{word_ur} - {name_ur}");
    let en = format!("{word_en} - {name_en}");
    AccountToMake {
        control_code,
        search_ur: text::search_ur(&ur),
        search_en: text::search_en(&en),
        search_phonetic: text::search_phonetic(&en),
        name_ur: ur,
        name_en: en,
    }
}

fn capital_account(name_ur: &str, name_en: &str) -> AccountToMake {
    account_to_make(
        CAPITAL_CONTROL,
        CAPITAL_WORD_UR,
        CAPITAL_WORD_EN,
        name_ur,
        name_en,
    )
}

fn drawings_account(name_ur: &str, name_en: &str) -> AccountToMake {
    account_to_make(
        DRAWINGS_CONTROL,
        DRAWINGS_WORD_UR,
        DRAWINGS_WORD_EN,
        name_ur,
        name_en,
    )
}

fn series_full(control: &str) -> ServiceError {
    reject_value(
        ErrorCode::ErrSeriesFull,
        None,
        "control",
        ErrorValue::Text(control.to_owned()),
    )
}

/// Checks the two names of a partner. Both are required, each in its own script (PRD FR-7.28).
fn names_of(name_ur: &str, name_en: &str) -> (Option<String>, Option<String>, FieldErrors) {
    let mut errors = FieldErrors::default();
    let ur = errors.take(validate::name_ur("name_ur", name_ur));
    let en = errors.take(validate::name_en("name_en", name_en));
    (ur, en, errors)
}

/// Adds a partner with his capital account (0602xxxx) and drawings account (0603xxxx), both
/// named in two scripts. He has no share (0 bps) until [`set_shares`]. An opening capital, if
/// given, is posted as the opening credit of his capital account dated today (PRD FR-10.9).
///
/// # Errors
/// The name codes (`name_ur`, `name_en`); `ERR_SPELLING_UNCONFIRMED`; the money codes on
/// `opening_capital`; `ERR_DATE_CLOSED` or `ERR_DATE_IN_FUTURE` on `opening_capital` (today is a
/// closed day); `ERR_TOO_MANY_PARTNERS` (value `max`); `ERR_SERIES_FULL`. Nothing is written
/// when any of them is returned.
pub fn create(
    ctx: &Context,
    actor: &Actor,
    input: &PartnerCreateInput,
) -> Result<PartnerRow, ServiceError> {
    let (name_ur, name_en, mut errors) = names_of(&input.name_ur, &input.name_en);
    let opening = errors
        .take(validate::money_optional(
            "opening_capital",
            input.opening_capital.as_deref().unwrap_or(""),
        ))
        .flatten()
        .filter(|amount| *amount != Paisa::ZERO);
    errors.finish()?;
    let (Some(name_ur), Some(name_en)) = (name_ur, name_en) else {
        return Err(ServiceError::Data("a name was lost".to_owned()));
    };
    // The spelling check reads only; the pairs it teaches are written in the transaction.
    let checked = spelling::check(
        ctx,
        &NamePair {
            field_ur: "name_ur",
            name_ur: &name_ur,
            field_en: "name_en",
            name_en: &name_en,
            kind: Kind::Name,
        },
        &input.confirmed_rules_words,
    )?;
    let (_, today) = day::window(ctx)?;
    let (row, account_ids, learned) = ctx.db.write(|tx| {
        if usize::try_from(repo::active_count(tx)?).unwrap_or(usize::MAX) >= MAX_PARTNERS {
            return Err(too_many());
        }
        let now = ctx.now_text();
        let capital = capital_account(&name_ur, &name_en);
        let drawings = drawings_account(&name_ur, &name_en);
        // PRD FR-10.4 v1.6: his gardens, trips and fruit account (0604) is made with him.
        let venture = venture_account(&name_ur, &name_en);
        let mut made = Vec::new();
        for account in [&capital, &drawings, &venture] {
            let (id, code) = master_repo::create_account(tx, account, actor.user_id, &now)?
                .ok_or_else(|| series_full(account.control_code))?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "create", "account")
                    .entity_id(id)
                    .new_value(json!({
                        "code": code,
                        "name_ur": account.name_ur,
                        "name_en": account.name_en,
                        "made_by": "partner",
                    }))
                    .into_entry(),
            )?;
            made.push(id);
        }
        let (capital_id, drawings_id, venture_id) = match made.as_slice() {
            [c, d, v] => (*c, *d, *v),
            _ => {
                return Err(ServiceError::Data(
                    "a partner needs three accounts".to_owned(),
                ));
            }
        };
        let id = repo::insert(
            tx,
            &repo::NewPartner {
                name_ur: &name_ur,
                name_en: &name_en,
                capital_account_id: capital_id,
                drawings_account_id: drawings_id,
                venture_account_id: venture_id,
            },
            actor.user_id,
            &now,
        )?;
        if let Some(amount) = opening {
            post_opening_capital(ctx, actor, tx, capital_id, today, amount, &now)?;
        }
        let learned = checked.learn(ctx, actor, tx)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "create", "partner")
                .entity_id(id)
                .new_value(json!({
                    "name_ur": name_ur,
                    "name_en": name_en,
                    "capital_account_id": capital_id,
                    "drawings_account_id": drawings_id,
                    "venture_account_id": venture_id,
                    "opening_capital_paisa": opening.map(Paisa::as_paisa),
                }))
                .into_entry(),
        )?;
        let record = repo::get(tx, id)?
            .ok_or_else(|| ServiceError::Data("the saved partner is missing".to_owned()))?;
        Ok((
            row_of(&record, 0),
            [capital_id, drawings_id, venture_id],
            learned,
        ))
    })?;
    learned.after_commit(ctx);
    for account_id in account_ids {
        search::refresh_account(ctx, account_id);
    }
    Ok(row)
}

/// Posts the opening capital of a new partner through the accounts opening service (PRD
/// FR-10.9): his capital account جمع, ابتدائی فرق بنام, dated today.
fn post_opening_capital(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    capital_id: i64,
    today: BusinessDate,
    amount: Paisa,
    now: &str,
) -> Result<(), ServiceError> {
    let facts = mandi_db::accounts::facts(tx, capital_id)?
        .ok_or_else(|| ServiceError::Data("the capital account is missing".to_owned()))?;
    let wanted = OpeningRequest {
        date: Some(today),
        dr: Paisa::ZERO,
        cr: amount,
        is_advance: false,
    };
    apply_opening(
        ctx,
        tx,
        actor.user_id,
        now,
        &facts,
        &wanted,
        "opening_capital",
    )
    .map_err(on_opening_capital)?;
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), "edit", "account")
            .entity_id(capital_id)
            .old(json!({ "opening": { "date": null, "dr_paisa": 0, "cr_paisa": 0, "is_advance": false } }))
            .new_value(json!({
                "opening": {
                    "date": today.to_iso(),
                    "dr_paisa": 0,
                    "cr_paisa": amount.as_paisa(),
                    "is_advance": false,
                }
            }))
            .into_entry(),
    )?;
    Ok(())
}

/// A date refusal of the ledger names the posting date field; on this form it is the opening
/// capital.
fn on_opening_capital(error: ServiceError) -> ServiceError {
    match error {
        ServiceError::Rejected(items) => ServiceError::Rejected(
            items
                .into_iter()
                .map(|mut item| {
                    if matches!(
                        item.field.as_deref(),
                        None | Some("entry_date" | "opening_date")
                    ) {
                        item.field = Some("opening_capital".to_owned());
                    }
                    item
                })
                .collect(),
        ),
        other => other,
    }
}

/// True if the partner has a share today or has one set for a later day (PRD FR-17.1).
fn has_share_from(history: &[repo::ShareRecord], today_iso: &str) -> bool {
    // The latest row on or before today is the share today; any row after today with a share
    // is a share set for later.
    let today_share = history
        .iter()
        .rfind(|h| h.effective_from.as_str() <= today_iso)
        .map_or(0, |h| h.share_bps);
    today_share > 0
        || history
            .iter()
            .any(|h| h.effective_from.as_str() > today_iso && h.share_bps > 0)
}

/// Changes a partner's names (and the names of his two accounts) or makes him active or inactive.
///
/// # Errors
/// The name codes; `ERR_SPELLING_UNCONFIRMED`; `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED`;
/// `ERR_PARTNER_HAS_SHARE` on `is_active` (he has a share today or later);
/// `ERR_PARTNER_IN_OPEN_VENTURE` on `is_active` (he is in an open garden or trip, PRD A63);
/// `ERR_TOO_MANY_PARTNERS` when making one active would pass the limit.
pub fn update(
    ctx: &Context,
    actor: &Actor,
    input: &PartnerUpdateInput,
) -> Result<PartnerRow, ServiceError> {
    let (name_ur, name_en, errors) = names_of(&input.name_ur, &input.name_en);
    errors.finish()?;
    let (Some(name_ur), Some(name_en)) = (name_ur, name_en) else {
        return Err(ServiceError::Data("a name was lost".to_owned()));
    };
    let checked = spelling::check(
        ctx,
        &NamePair {
            field_ur: "name_ur",
            name_ur: &name_ur,
            field_en: "name_en",
            name_en: &name_en,
            kind: Kind::Name,
        },
        &input.confirmed_rules_words,
    )?;
    let (_, today) = day::window(ctx)?;
    let today_iso = today.to_iso();
    let (row, learned) = ctx.db.write(|tx| {
        let old = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
        if old.version != input.version {
            return Err(ServiceError::reject(ErrorCode::ErrVersionChanged));
        }
        if old.is_active && !input.is_active {
            let history = repo::share_history(tx, old.id)?;
            if has_share_from(&history, &today_iso) {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrPartnerHasShare,
                    "is_active",
                ));
            }
        }
        if old.is_active && !input.is_active && repo::in_open_venture(tx, old.id)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrPartnerInOpenVenture,
                "is_active",
            ));
        }
        if !old.is_active
            && input.is_active
            && usize::try_from(repo::active_count(tx)?).unwrap_or(usize::MAX) >= MAX_PARTNERS
        {
            return Err(too_many());
        }
        let now = ctx.now_text();
        let changed = repo::update(
            tx,
            old.id,
            input.version,
            &name_ur,
            &name_en,
            input.is_active,
            actor.user_id,
            &now,
        )?;
        if !changed {
            return Err(ServiceError::reject(ErrorCode::ErrVersionChanged));
        }
        if old.name_ur != name_ur || old.name_en != name_en {
            master_repo::rename_account(
                tx,
                old.capital_account_id,
                &capital_account(&name_ur, &name_en),
                actor.user_id,
                &now,
            )?;
            master_repo::rename_account(
                tx,
                old.drawings_account_id,
                &drawings_account(&name_ur, &name_en),
                actor.user_id,
                &now,
            )?;
            if old.venture_account_id != 0 {
                master_repo::rename_account(
                    tx,
                    old.venture_account_id,
                    &venture_account(&name_ur, &name_en),
                    actor.user_id,
                    &now,
                )?;
            }
        }
        let learned = checked.learn(ctx, actor, tx)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "partner")
                .entity_id(old.id)
                .old(json!({
                    "name_ur": old.name_ur,
                    "name_en": old.name_en,
                    "is_active": old.is_active,
                }))
                .new_value(json!({
                    "name_ur": name_ur,
                    "name_en": name_en,
                    "is_active": input.is_active,
                }))
                .into_entry(),
        )?;
        let record = repo::get(tx, old.id)?
            .ok_or_else(|| ServiceError::Data("the saved partner is missing".to_owned()))?;
        let share = repo::shares_on(tx, &today_iso)?
            .into_iter()
            .find(|&(partner, _)| partner == old.id)
            .map_or(0, |(_, bps)| bps);
        Ok((row_of(&record, share_i32(share)?), learned))
    })?;
    learned.after_commit(ctx);
    search::refresh_account(ctx, row.capital_account_id);
    search::refresh_account(ctx, row.drawings_account_id);
    if row.venture_account_id != 0 {
        search::refresh_account(ctx, row.venture_account_id);
    }
    Ok(row)
}

fn field_errors(errors: Vec<FieldError>) -> ServiceError {
    ServiceError::from(errors)
}

/// Sets the shares of all active partners from a day (PRD FR-17.1, FR-17.2). One history row is
/// written for each listed partner, in one transaction with one audit row. Postings already
/// saved keep their split.
///
/// # Errors
/// `ERR_DATE_INVALID` and `ERR_SHARE_BACKDATED` (value `today`) on `effective_from`; the percent
/// codes on `shares[i].share`; `ERR_PARTNER_NOT_LISTED` on `shares`; `ERR_SHARES_NOT_100` on
/// `shares` (value `total_bps`). All problems that can be found together come back together.
pub fn set_shares(
    ctx: &Context,
    actor: &Actor,
    input: &PartnersSetSharesInput,
) -> Result<Vec<PartnerRow>, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors: Vec<FieldError> = Vec::new();
    let date = match validate::date("effective_from", &input.effective_from, today) {
        Ok(date) if date < today => {
            errors.push(
                FieldError::new(ErrorCode::ErrShareBackdated, "effective_from")
                    .with("today", ErrorValue::Text(format_ddmmyyyy(today))),
            );
            None
        }
        Ok(date) => Some(date),
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let read = read_shares(&input.shares);
    let rows_ok = read.errors.is_empty();
    errors.extend(read.errors);
    if rows_ok && read.total_bps != mandi_domain::core::money::BPS_FULL {
        errors.push(not_100(read.total_bps));
    }
    let today_iso = today.to_iso();
    let now_rows = ctx.db.write(|tx| {
        // The list must name every active partner once (checked inside the transaction, so a
        // partner made inactive a moment ago cannot slip in).
        let active: Vec<i64> = repo::list(tx)?
            .iter()
            .filter(|p| p.is_active)
            .map(|p| p.id)
            .collect();
        let mut errors = errors.clone();
        errors.extend(check_listed(&input.shares, &active));
        let Some(date) = date.filter(|_| errors.is_empty()) else {
            return Err(field_errors(errors));
        };
        let now = ctx.now_text();
        let effective = date.to_iso();
        let old: Vec<Value> = repo::shares_on(tx, &today_iso)?
            .into_iter()
            .map(|(partner, bps)| json!({ "partner_id": partner, "share_bps": bps }))
            .collect();
        let mut new: Vec<Value> = Vec::with_capacity(read.shares.len());
        for (partner, bps) in &read.shares {
            repo::insert_share(
                tx,
                *partner,
                i64::from(bps.get()),
                &effective,
                actor.user_id,
                &now,
            )?;
            new.push(json!({ "partner_id": partner, "share_bps": bps.get() }));
        }
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "set_shares", "partner")
                .old(json!({ "shares_today": old }))
                .new_value(json!({ "effective_from": effective, "shares": new }))
                .into_entry(),
        )?;
        rows_in(tx, &today_iso)
    })?;
    Ok(now_rows)
}

/// The running total of typed shares. Read only; the same reading as [`set_shares`].
///
/// # Errors
/// `ServiceError::Db`.
pub fn check(ctx: &Context, input: &PartnersCheckSharesInput) -> Result<SharesCheck, ServiceError> {
    let active: Vec<i64> = ctx.db.read(|conn| {
        Ok::<_, ServiceError>(
            repo::list(conn)?
                .iter()
                .filter(|p| p.is_active)
                .map(|p| p.id)
                .collect(),
        )
    })?;
    Ok(check_shares(&input.shares, Some(&active)))
}

/// A sum of the signed share rows of one kind, per partner.
fn sums_of(sums: &[repo::ShareSum], partner: i64, kind: ShareKind) -> Result<Paisa, ServiceError> {
    sums.iter()
        .filter(|s| s.partner_id == partner && s.kind == kind.as_str())
        .try_fold(Paisa::ZERO, |acc, s| {
            acc.checked_add(Paisa::try_from_paisa(s.total_paisa).map_err(money_error)?)
                .map_err(money_error)
        })
}

/// One row per partner (PRD 17.3): today, month and year income share, month expense share,
/// profit, money taken and capital balance. An inactive partner is listed only if he has a
/// number to show. "Month" and "year" are the calendar month and year up to today.
///
/// # Errors
/// `ServiceError::Db`; `ERR_OVERFLOW`.
pub fn overview(ctx: &Context) -> Result<Vec<PartnerOverviewRow>, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let today_iso = today.to_iso();
    let month_start = BusinessDate::from_ymd(today.year(), today.month(), 1)
        .ok_or_else(|| ServiceError::Data("the first day of the month".to_owned()))?
        .to_iso();
    let year_start = BusinessDate::from_ymd(today.year(), 1, 1)
        .ok_or_else(|| ServiceError::Data("the first day of the year".to_owned()))?
        .to_iso();
    ctx.db.read(|conn| {
        let partners = rows_in(conn, &today_iso)?;
        let day_sums = repo::share_sums(conn, &today_iso, &today_iso)?;
        let month_sums = repo::share_sums(conn, &month_start, &today_iso)?;
        let year_sums = repo::share_sums(conn, &year_start, &today_iso)?;
        let drawings: Vec<i64> = partners.iter().map(|p| p.drawings_account_id).collect();
        let capitals: Vec<i64> = partners.iter().map(|p| p.capital_account_id).collect();
        let taken = repo::taken_between(conn, &drawings, &month_start, &today_iso)?;
        let capital_totals = mandi_db::ledger::totals_for_accounts(conn, &capitals)?;
        // This month's capital put in and the net of his gardens, trips and fruit account, with
        // the entries of month close left out (PRD FR-17.6, FR-17.7, FR-17.10).
        let ventures: Vec<i64> = partners
            .iter()
            .map(|p| p.venture_account_id)
            .filter(|id| *id != 0)
            .collect();
        let capital_month = repo::net_between(conn, &capitals, &month_start, &today_iso)?;
        let venture_month = repo::net_between(conn, &ventures, &month_start, &today_iso)?;
        let net_of = |sums: Option<&repo::NetSums>| -> Result<Paisa, ServiceError> {
            let (dr, cr) = sums.map_or((0, 0), |s| (s.dr, s.cr));
            Paisa::try_from_paisa(cr)
                .map_err(money_error)?
                .checked_sub(Paisa::try_from_paisa(dr).map_err(money_error)?)
                .map_err(money_error)
        };
        let mut out = Vec::with_capacity(partners.len());
        for partner in &partners {
            let id = partner.id;
            let today_income = sums_of(&day_sums, id, ShareKind::Income)?;
            let month_income = sums_of(&month_sums, id, ShareKind::Income)?;
            let year_income = sums_of(&year_sums, id, ShareKind::Income)?;
            // Expense rows are negative; the screen shows the expense as a positive number.
            let month_expense = sums_of(&month_sums, id, ShareKind::Expense)?
                .checked_neg()
                .map_err(money_error)?;
            let month_profit = month_income
                .checked_sub(month_expense)
                .map_err(money_error)?;
            // Money taken: ordinary debits on his drawings account, less the takes that were
            // cancelled or edited (reversal credits), in checked Paisa maths.
            let sums = taken.get(&partner.drawings_account_id);
            let (debits, reversed) = sums.map_or((0, 0), |t| (t.debits, t.reversed_credits));
            let month_taken = Paisa::try_from_paisa(debits)
                .map_err(money_error)?
                .checked_sub(Paisa::try_from_paisa(reversed).map_err(money_error)?)
                .map_err(money_error)?;
            // Money returned (PRD FR-16.2): the credits of ordinary entries, less the returns that
            // were cancelled or edited (their reversals debit the account).
            let (credits, reversed_debits) =
                sums.map_or((0, 0), |t| (t.credits, t.reversed_debits));
            let month_returned = Paisa::try_from_paisa(credits)
                .map_err(money_error)?
                .checked_sub(Paisa::try_from_paisa(reversed_debits).map_err(money_error)?)
                .map_err(money_error)?;
            // PRD C20: خالص = profit share - (money taken - money returned).
            let month_net = month_profit
                .checked_sub(
                    month_taken
                        .checked_sub(month_returned)
                        .map_err(money_error)?,
                )
                .map_err(money_error)?;
            let month_capital_in = net_of(capital_month.get(&partner.capital_account_id))?;
            let month_ventures = net_of(venture_month.get(&partner.venture_account_id))?;
            // PRD C22, FR-17.9: everything he has in the firm, from the posted records.
            let record = repo::get(conn, id)?
                .ok_or_else(|| ServiceError::Data("a partner of the list is missing".to_owned()))?;
            let total = money::total_of(conn, &record, None)?;
            // The credit balance of his capital account; negative if it stands on the debit side.
            let capital = match capital_totals.get(&partner.capital_account_id) {
                Some(totals) => ledger::signed(ledger::balance_of(*totals)?)?
                    .checked_neg()
                    .map_err(money_error)?,
                None => Paisa::ZERO,
            };
            let numbers = [
                today_income,
                month_income,
                year_income,
                month_expense,
                month_profit,
                month_taken,
                capital,
                month_returned,
                month_capital_in,
                month_ventures,
                total,
            ];
            if !partner.is_active && numbers.iter().all(|n| *n == Paisa::ZERO) {
                continue;
            }
            out.push(PartnerOverviewRow {
                partner_id: id,
                name_ur: partner.name_ur.clone(),
                name_en: partner.name_en.clone(),
                share_bps: partner.share_bps,
                today_income_paisa: today_income,
                month_income_paisa: month_income,
                year_income_paisa: year_income,
                month_expense_paisa: month_expense,
                month_profit_paisa: month_profit,
                month_taken_paisa: month_taken,
                capital_balance_paisa: capital,
                month_returned_paisa: month_returned,
                month_net_paisa: month_net,
                month_capital_in_paisa: month_capital_in,
                month_ventures_paisa: month_ventures,
                total_paisa: total,
            });
        }
        Ok(out)
    })
}
