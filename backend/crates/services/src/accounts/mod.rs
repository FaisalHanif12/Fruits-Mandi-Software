//! Accounts (PRD 10.2, FR-10.3, FR-10.5, FR-10.9, plan 07): add, edit, make inactive, list, the
//! next code, the duplicate warning and the openings grid.
//!
//! One user action is one transaction (CLAUDE.md rule 3): the account, its opening entry, the
//! pairs the user confirmed in the spelling check, and the audit row are written together or
//! not at all. The search index is updated after the commit (Code Standards B27). Nothing is
//! ever deleted (PRD FR-10.6).

mod openings;

use mandi_db::accounts as repo;
use mandi_db::codes::CodeTable;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{
    AccountInput, AccountRow, AccountType, AccountsList, AccountsListInput, AccountsSimilarInput,
    ControlRule, OpeningRequest, SimilarAccount, account_code_for, can_take_opening,
    prepare_account,
};
use mandi_domain::core::codes::{build_code, check_typed_code};
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::text;
use mandi_domain::core::translit::Kind;
use mandi_domain::core::validate;
use serde_json::{Map, Value, json};

pub(crate) use openings::apply as apply_opening;
pub use openings::{openings_preview, openings_save};

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger;
use crate::search;
use crate::spelling::{self, NamePair, SpellingCheck};

/// The most rows one list returns: 2.5 times the 20,000 accounts of NFR-SCL-1, so the list is
/// never cut at the planned size (plan 07). A screen draws only the rows it can see.
pub const MAX_PAGE: u32 = 50_000;
/// The most accounts the duplicate warning shows.
const MAX_SIMILAR: usize = 8;

fn series_full(control_code: &str) -> ServiceError {
    ServiceError::Rejected(vec![Rejection {
        code: ErrorCode::ErrSeriesFull,
        field: Some("code".to_owned()),
        values: std::collections::BTreeMap::from([(
            "control".to_owned(),
            ErrorValue::Text(control_code.to_owned()),
        )]),
    }])
}

/// Fills the balance of rows from the posted journal (one query for all accounts).
/// A page of at most this many accounts reads its balances with one query that names them; a
/// bigger page reads the balance of every account in one pass over the journal, which is cheaper
/// than a very long list of ids.
const NAMED_BALANCES_UP_TO: usize = 500;

fn fill_balances(conn: &Connection, rows: &mut [AccountRow]) -> Result<(), ServiceError> {
    let totals = if rows.len() <= NAMED_BALANCES_UP_TO {
        let ids: Vec<i64> = rows.iter().map(|row| row.id).collect();
        mandi_db::ledger::totals_for_accounts(conn, &ids)?
    } else {
        ledger::balances_by_account(conn, None)?
    };
    for row in rows {
        if let Some(found) = totals.get(&row.id) {
            row.balance = ledger::balance_of(*found)?;
        }
    }
    Ok(())
}

fn one_with_balance(conn: &Connection, id: i64) -> Result<AccountRow, ServiceError> {
    let mut row = repo::get(conn, id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
    row.balance = ledger::balance_in(conn, id, None, None)?;
    Ok(row)
}

/// One account with its balance.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, id: i64) -> Result<AccountRow, ServiceError> {
    ctx.db.read(|conn| one_with_balance(conn, id))
}

/// A page of accounts by code with the columns of PRD FR-10.5, and the footer: how many match
/// and the sum of their opening debits and credits.
///
/// # Errors
/// `ServiceError::Db`.
pub fn list(ctx: &Context, input: &AccountsListInput) -> Result<AccountsList, ServiceError> {
    let limit = match input.limit {
        0 => MAX_PAGE,
        n => n.min(MAX_PAGE),
    };
    let typed = input.filter.text.trim();
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    let order_by = input
        .sort
        .as_ref()
        .map(|sort| {
            sort.order_by(&repo::SORT_COLUMNS).map(|order| {
                if sort.column == "code" {
                    mandi_db::codes::order_by_code_dir("a.code", sort.direction.sql())
                } else {
                    order
                }
            })
        })
        .transpose()
        .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    ctx.db.read(|conn| {
        let (mut rows, totals) = repo::list(
            conn,
            &input.filter,
            &text::search_ur(typed),
            &text::search_en(typed),
            order_by.as_deref(),
            input.offset,
            limit,
        )?;
        fill_balances(conn, &mut rows)?;
        Ok(AccountsList {
            rows,
            count: totals.count,
            total_dr_paisa: Paisa::try_from_paisa(totals.opening_dr_paisa).map_err(money_error)?,
            total_cr_paisa: Paisa::try_from_paisa(totals.opening_cr_paisa).map_err(money_error)?,
        })
    })
}

/// The control of a type of account. For `other` the control is chosen and must be one that
/// accounts of that type may use.
fn resolve_control(
    tx: &Connection,
    account_type: AccountType,
    control_id: Option<i64>,
) -> Result<repo::ControlRow, ServiceError> {
    match account_type.fixed_control() {
        Some(code) => repo::control_by_code(tx, code)?
            .ok_or_else(|| ServiceError::Data(format!("control {code} is missing"))),
        None => {
            let id = control_id
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrRequired, "control_id"))?;
            let control = repo::control_by_id(tx, id)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "control_id"))?;
            if ControlRule::allows_other(&control.code, &control.party_type) {
                Ok(control)
            } else {
                Err(ServiceError::reject_field(
                    ErrorCode::ErrFormatInvalid,
                    "control_id",
                ))
            }
        }
    }
}

/// The next free code for a kind of account. A seller gets `001` plus the next running number and a
/// buyer `002` plus the next running number (`0010`, `0011` ...; plan 32 R2, never a code used
/// before). Every other kind keeps the control code plus the next free serial (PRD FR-10.3).
///
/// # Errors
/// `ERR_REQUIRED` or `ERR_NOT_FOUND` on `control_id`; `ERR_SERIES_FULL` when the numbers are used up.
pub fn next_code(
    ctx: &Context,
    account_type: AccountType,
    control_id: Option<i64>,
) -> Result<String, ServiceError> {
    ctx.db.read(|conn| {
        if let Some(code_type) = account_type.code_type() {
            let (_, code) = mandi_db::codes::next_code(conn, CodeTable::Accounts, code_type)?
                .ok_or_else(|| series_full(code_type.number()))?;
            return Ok(code);
        }
        let control = resolve_control(conn, account_type, control_id)?;
        let serial =
            repo::next_serial(conn, control.id)?.ok_or_else(|| series_full(&control.code))?;
        Ok(format!("{}{serial:04}", control.code))
    })
}

/// Accounts that look like the one being typed: names in either script (mistakes forgiven) and
/// the same phone (PRD FR-10.7, FR-7.63). Only reads.
///
/// An account is marked `is_duplicate` when its name (in either script typed), its tag and its
/// phone all match what was typed (PRD FR-10.7, E38). The same name and phone with another tag,
/// or one with a tag and one without, is another person and is only listed as similar: `اسلم BA`
/// and `اسلم` are two different people. A tag or a phone that is not valid yet marks nothing.
///
/// # Errors
/// `ServiceError::Db`.
pub fn similar(
    ctx: &Context,
    input: &AccountsSimilarInput,
) -> Result<Vec<SimilarAccount>, ServiceError> {
    let mut ids = search::similar_account_ids(
        ctx,
        &[&input.name_ur, &input.name_en],
        input.exclude_id,
        MAX_SIMILAR,
    )?;
    // The same phone number, if the typed text is a valid one.
    if let Ok(Some(phone)) = validate::phone_optional("phone", &input.phone) {
        let found = ctx
            .db
            .read(|conn| repo::ids_by_phone(conn, &phone, input.exclude_id))?;
        for id in found {
            if !ids.contains(&id) {
                ids.insert(0, id);
            }
        }
    }
    ids.truncate(MAX_SIMILAR);
    let mut found = ctx
        .db
        .read(|conn| repo::similar_by_ids(conn, &ids).map_err(ServiceError::from))?;
    mark_duplicates(&mut found, input);
    // A duplicate is listed first; the rest keep their order.
    found.sort_by_key(|account| !account.is_duplicate);
    Ok(found)
}

/// Sets `is_duplicate` where the typed name, tag and phone all equal the account's (PRD FR-10.7).
/// Names are compared by their search keys, so look-alike letters and capitals do not matter.
fn mark_duplicates(found: &mut [SimilarAccount], input: &AccountsSimilarInput) {
    let (Ok(tag), Ok(phone)) = (
        validate::tag("tag", &input.tag),
        validate::phone_optional("phone", &input.phone),
    ) else {
        return;
    };
    let phone = phone.unwrap_or_default();
    let typed_ur = text::search_ur(&input.name_ur);
    let typed_en = text::search_en(&input.name_en);
    if typed_ur.is_empty() && typed_en.is_empty() {
        return;
    }
    for account in found {
        let name_matches = (!typed_ur.is_empty() && text::search_ur(&account.name_ur) == typed_ur)
            || (!typed_en.is_empty() && text::search_en(&account.name_en) == typed_en);
        account.is_duplicate = name_matches && account.tag == tag && account.phone1.trim() == phone;
    }
}

/// Why an opening cannot be put on an account (PRD FR-6.5, FR-10.9, FR-13.5, FR-12.4): income and
/// shop expense accounts would need partner share rows, ابتدائی فرق would be posted against
/// itself, a پختہ کھاتہ account has no person to hold the balance, and نقدی منڈی pays at once.
/// The documents do not say what to do about these; they are refused until Umar decides.
pub(crate) fn opening_allowed(facts: &repo::AccountFacts) -> bool {
    can_take_opening(
        &facts.head_code,
        &facts.control_code,
        &facts.code,
        facts.is_pukhta_khata,
    )
}

/// Checks the name and the father or firm name together, so both fields come back at once.
fn spelling_checks(
    ctx: &Context,
    prepared: &mandi_domain::core::accounts::PreparedAccount,
) -> Result<Vec<SpellingCheck>, ServiceError> {
    spelling::check_all(
        ctx,
        &[
            NamePair {
                field_ur: prepared.names.field_ur,
                name_ur: &prepared.names.name_ur,
                field_en: prepared.names.field_en,
                name_en: &prepared.names.name_en,
                kind: Kind::Name,
            },
            NamePair {
                field_ur: "father_or_firm_ur",
                name_ur: &prepared.father.0,
                field_en: "father_or_firm_en",
                name_en: &prepared.father.1,
                kind: Kind::Name,
            },
        ],
        &prepared.confirmed_rules_words,
    )
}

fn audit_fields(prepared: &mandi_domain::core::accounts::PreparedAccount, code: &str) -> Value {
    let mut map = Map::new();
    map.insert("code".to_owned(), json!(code));
    for (name, value) in &prepared.fields {
        if name.starts_with("search_") {
            continue;
        }
        map.insert(
            (*name).to_owned(),
            match value {
                mandi_domain::core::masters::ColValue::Text(t) => json!(t),
                mandi_domain::core::masters::ColValue::Int(i) => json!(i),
                mandi_domain::core::masters::ColValue::Null => Value::Null,
            },
        );
    }
    let o = &prepared.opening;
    map.insert(
        "opening".to_owned(),
        json!({
            "date": o.date.map(|d| d.to_iso()),
            "dr_paisa": o.dr.as_paisa(),
            "cr_paisa": o.cr.as_paisa(),
            "is_advance": o.is_advance,
        }),
    );
    Value::Object(map)
}

/// Adds an account (`id` null) or edits one (`id` and the `version` that was read).
///
/// # Errors
/// The field errors of `mandi_core::accounts::prepare_account` (and the date codes of the
/// opening date); `ERR_SPELLING_UNCONFIRMED`; `ERR_NOT_FOUND` on `id` or on a record it points
/// at; `ERR_CODE_TAKEN`, `ERR_CODE_RESERVED`, `ERR_SERIES_FULL` on `code`; `ERR_VERSION_CHANGED`;
/// `ERR_FORMAT_INVALID` on `account_type` or `code` for an edit that tries to change them;
/// `ERR_SYSTEM_ACCOUNT` on `is_active` for an edit that makes a system account inactive.
/// Nothing is written when any of them is returned.
pub fn save(
    ctx: &Context,
    actor: &Actor,
    input: &AccountInput,
) -> Result<AccountRow, ServiceError> {
    let (closed, today) = day::window(ctx)?;
    // An edit that leaves the opening as it is must not trip over a closed opening day.
    let stored = match input.id {
        Some(id) => ctx.db.read(|conn| repo::facts(conn, id))?,
        None => None,
    };
    // A system account stays active (PRD FR-10.4), whether the box is changed on the form or by
    // `set_active`. `is_system` never changes (trigger `accounts_system_fixed`), so reading it
    // before the write is safe.
    if !input.is_active && stored.as_ref().is_some_and(|facts| facts.is_system) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrSystemAccount,
            "is_active",
        ));
    }
    // A person in the dead amount is made active again only by "دوبارہ فعال کریں" with a reason
    // (PRD FR-10.10), never by the box on this form.
    if input.is_active
        && stored.as_ref().is_some_and(|facts| !facts.is_active)
        && let Some(id) = input.id
        && ctx.db.read(|conn| crate::dead::is_dead(conn, id, None))?
    {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrAlreadyDead,
            "is_active",
        ));
    }
    let in_force = stored.as_ref().map(current_opening).transpose()?;
    let prepared = prepare_account(input, closed, today, in_force.as_ref())?;
    let checks = spelling_checks(ctx, &prepared)?;
    let (id, learned) = ctx
        .db
        .write(|tx| write(ctx, actor, tx, &prepared, &checks))?;
    for learned in learned {
        learned.after_commit(ctx);
    }
    search::refresh_account(ctx, id);
    get(ctx, id)
}

/// Makes a new account inside the caller's transaction, with the same checks as [`save`] (plan
/// 15, PRD FR-13.8: a one-time person becomes a regular account in the same action that moves his
/// balance). The caller refreshes the search index and runs the returned learning steps after the
/// commit.
///
/// # Errors
/// As [`save`] for a new account.
pub(crate) fn create_in(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    input: &AccountInput,
    window: (
        Option<mandi_domain::core::dates::BusinessDate>,
        mandi_domain::core::dates::BusinessDate,
    ),
) -> Result<(i64, Vec<crate::spelling::Learned>), ServiceError> {
    let prepared = prepare_account(input, window.0, window.1, None)?;
    let checks = spelling_checks(ctx, &prepared)?;
    write(ctx, actor, tx, &prepared, &checks)
}

/// The code and the serial of a new account. A seller or a buyer: the typed code, which must be a
/// full code of its type, or the next running number; the serial is the running number (plan 32
/// R2). Any other account: the control code plus a 4-digit serial (PRD FR-10.3).
fn new_code(
    tx: &Connection,
    prepared: &mandi_domain::core::accounts::PreparedAccount,
    control: &repo::ControlRow,
) -> Result<(String, u32), ServiceError> {
    if let Some(code_type) = prepared.account_type.code_type() {
        let running = match &prepared.typed_code {
            Some(typed) => check_typed_code("code", typed, code_type)?,
            None => {
                mandi_db::codes::next_code(tx, CodeTable::Accounts, code_type)?
                    .ok_or_else(|| series_full(code_type.number()))?
                    .0
            }
        };
        let code = build_code(code_type, running)
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrSeriesFull, "code"))?;
        let serial = u32::try_from(running)
            .map_err(|_| ServiceError::reject_field(ErrorCode::ErrSeriesFull, "code"))?;
        return Ok((code, serial));
    }
    let serial = match &prepared.typed_code {
        Some(typed) => account_code_for("code", &control.code, typed)?,
        None => repo::next_serial(tx, control.id)?.ok_or_else(|| series_full(&control.code))?,
    };
    Ok((format!("{}{serial:04}", control.code), serial))
}

fn write(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    prepared: &mandi_domain::core::accounts::PreparedAccount,
    checks: &[SpellingCheck],
) -> Result<(i64, Vec<crate::spelling::Learned>), ServiceError> {
    let now = ctx.now_text();
    for reference in &prepared.references {
        if !mandi_db::masters::exists(tx, reference.table, reference.id)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNotFound,
                reference.field,
            ));
        }
    }
    let old = match prepared.id {
        Some(id) => {
            let row = repo::get(tx, id)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
            if prepared.version != Some(row.version) {
                return Err(ServiceError::reject(ErrorCode::ErrVersionChanged));
            }
            if prepared.account_type != row.account_type {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrFormatInvalid,
                    "account_type",
                ));
            }
            Some(row)
        }
        None => None,
    };

    let id = match &old {
        None => {
            let control = resolve_control(tx, prepared.account_type, prepared.control_id)?;
            let (code, serial) = new_code(tx, prepared, &control)?;
            if repo::code_taken(tx, &code, None)? {
                return Err(ServiceError::reject_field(ErrorCode::ErrCodeTaken, "code"));
            }
            repo::insert(
                tx,
                control.id,
                &code,
                serial,
                &control.party_type,
                &prepared.fields,
                actor.user_id,
                &now,
            )?
        }
        Some(row) => {
            // The code of an account never changes after it is made (its postings name it).
            if prepared.typed_code.as_ref().is_some_and(|c| *c != row.code) {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrFormatInvalid,
                    "code",
                ));
            }
            if !repo::update(
                tx,
                row.id,
                row.version,
                &prepared.fields,
                actor.user_id,
                &now,
            )? {
                return Err(ServiceError::reject(ErrorCode::ErrVersionChanged));
            }
            row.id
        }
    };

    // The opening (PRD FR-10.9): reverse the old entry and post the new one, or leave it alone.
    let facts = repo::facts(tx, id)?
        .ok_or_else(|| ServiceError::Data("the saved account is missing".to_owned()))?;
    openings::apply(
        ctx,
        tx,
        actor.user_id,
        &now,
        &facts,
        &prepared.opening,
        "opening_dr",
    )?;

    let learned = checks
        .iter()
        .map(|check| check.learn(ctx, actor, tx))
        .collect::<Result<Vec<_>, _>>()?;
    let row = repo::get(tx, id)?
        .ok_or_else(|| ServiceError::Data("the saved account is missing".to_owned()))?;
    let mut event = Event::new(
        Some(actor.user_id),
        if old.is_some() { "edit" } else { "create" },
        "account",
    )
    .entity_id(id)
    .new_value(audit_fields(prepared, &row.code));
    if let Some(old) = &old {
        event =
            event.old(serde_json::to_value(old).map_err(|e| ServiceError::Data(e.to_string()))?);
    }
    ctx.db.auditor().write(tx, &event.into_entry())?;
    Ok((id, learned))
}

/// Makes an account active or inactive (an account is never deleted, PRD FR-10.6). Asking for
/// the state it already has changes nothing.
///
/// A system account (PRD FR-10.4: روکڑ, ابتدائی فرق, the income accounts, نقدی منڈی, the two
/// پختہ کھاتہ accounts ...) cannot be made inactive, because the app posts to it by itself and a
/// picker would no longer offer it. Making one active again is allowed, so a file where the
/// state was changed some other way can be put right.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ERR_SYSTEM_ACCOUNT` on `id` when a system account is made inactive.
pub fn set_active(
    ctx: &Context,
    actor: &Actor,
    id: i64,
    is_active: bool,
) -> Result<AccountRow, ServiceError> {
    let changed = ctx.db.write(|tx| {
        let old = repo::facts(tx, id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
        if old.is_system && !is_active {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrSystemAccount,
                "id",
            ));
        }
        if old.is_active == is_active {
            return Ok::<_, ServiceError>(false);
        }
        // A person in the dead amount is made active again only by "دوبارہ فعال کریں" (PRD
        // FR-10.10).
        if is_active && crate::dead::is_dead(tx, id, None)? {
            return Err(ServiceError::reject_field(ErrorCode::ErrAlreadyDead, "id"));
        }
        repo::set_active(tx, id, is_active, actor.user_id, &ctx.now_text())?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "account")
                .entity_id(id)
                .old(json!({ "is_active": old.is_active }))
                .new_value(json!({ "is_active": is_active }))
                .into_entry(),
        )?;
        Ok(true)
    })?;
    if changed {
        search::refresh_account(ctx, id);
    }
    get(ctx, id)
}

/// Used by the grid and the form: the opening the account has now, as a request.
pub(crate) fn current_opening(facts: &repo::AccountFacts) -> Result<OpeningRequest, ServiceError> {
    let date = facts
        .opening_date
        .as_deref()
        .map(|d| {
            mandi_domain::core::dates::BusinessDate::parse_iso(d)
                .ok_or_else(|| ServiceError::Data(format!("opening date {d}")))
        })
        .transpose()?;
    Ok(OpeningRequest {
        date,
        dr: Paisa::try_from_paisa(facts.opening_dr_paisa).map_err(money_error)?,
        cr: Paisa::try_from_paisa(facts.opening_cr_paisa).map_err(money_error)?,
        is_advance: facts.opening_is_advance,
    })
}
