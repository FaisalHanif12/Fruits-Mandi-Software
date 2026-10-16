//! Search for the pickers (plan 06, Tech Stack 4.7, PRD FR-7.40, FR-7.41).
//!
//! - The index is built from the database the first time it is needed, at start-up
//!   ([`warm_up`]) and after a restore ([`rebuild`]); a save that changes a name, a code or the
//!   active flag updates it right after the commit ([`refresh`]).
//! - [`search`] and [`recent`] only read. [`touch`] writes the signed-in user's own preference
//!   `recent.<list>` and nothing else (Code Standards B29).
//! - The text is searched as typed: it is never converted to Urdu (PRD FR-7.22, F41).

mod index;

use std::sync::Arc;

use mandi_db::masters as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{
    AccountBalance, AccountHit, AccountScope, BUYER_CONTROL, Balance, GlobalHit, ResolveCodeInput,
    ResolveStatus, ResolvedCode, SELLER_CONTROL, SearchAccountsInput, SearchGlobalInput,
};
use mandi_domain::core::accounts::{AccountsRecentInput, AccountsTouchInput};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::masters::MasterKind;
use mandi_domain::core::search::{SearchHit, SearchScope};
use serde_json::{Value, json};

pub use index::{
    Index, MAX_FULL_LIST, MAX_HITS, RECENT_KEPT, SHORT_CODE_DIGITS, Usage, short_serial,
};

use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::store;

fn pref_key(scope: SearchScope) -> String {
    format!("recent.{}", scope.as_str())
}

fn build(conn: &Connection) -> Result<Index, ServiceError> {
    let mut index = Index::empty();
    for kind in MasterKind::ALL {
        index.set_scope(kind, repo::search_sources(conn, kind, None)?);
    }
    index.set_accounts(mandi_db::accounts::search_sources(conn, None)?);
    Ok(index)
}

/// The index in memory, built from the database the first time.
fn current(ctx: &Context) -> Result<Arc<Index>, ServiceError> {
    let mut slot = ctx.search_lock();
    if let Some(index) = slot.as_ref() {
        return Ok(Arc::clone(index));
    }
    let index = Arc::new(ctx.db.read(build)?);
    *slot = Some(Arc::clone(&index));
    Ok(index)
}

/// Builds the index again from the database. The restore of a backup calls this (the whole
/// data is replaced). Searches give the same answers as before a restore that changed nothing.
///
/// # Errors
/// `ServiceError::Db`.
pub fn rebuild(ctx: &Context) -> Result<(), ServiceError> {
    let index = Arc::new(ctx.db.read(build)?);
    *ctx.search_lock() = Some(index);
    Ok(())
}

/// Gets the index built, so the first key press is not the slow one (NFR-SPD-6). Failures are
/// ignored: the first real use tries again.
pub fn warm_up(ctx: &Context) {
    let _ = current(ctx);
}

/// Updates the index for one master after the save that changed it has committed (Code
/// Standards B27). If anything goes wrong the index is thrown away and built again on the next
/// use, so it can never keep a wrong answer.
pub fn refresh(ctx: &Context, kind: MasterKind, id: i64) {
    let mut slot = ctx.search_lock();
    let Some(index) = slot.as_mut() else {
        return;
    };
    let source = ctx
        .db
        .read(|conn| repo::search_sources(conn, kind, Some(id)));
    match source {
        Ok(mut found) => Arc::make_mut(index).put(kind, id, found.pop()),
        Err(_) => *slot = None,
    }
}

/// Updates the index for one account after the save that changed it has committed (plan 07).
/// Like [`refresh`], any failure throws the index away so it is built again on the next use.
pub fn refresh_account(ctx: &Context, id: i64) {
    let mut slot = ctx.search_lock();
    let Some(index) = slot.as_mut() else {
        return;
    };
    let source = ctx
        .db
        .read(|conn| mandi_db::accounts::search_sources(conn, Some(id)));
    match source {
        Ok(mut found) => Arc::make_mut(index).put_account(id, found.pop()),
        Err(_) => *slot = None,
    }
}

/// The ids of active accounts whose name looks like what was typed, in either script, with
/// mistakes forgiven (PRD FR-10.7, FR-7.63). Only reads.
///
/// # Errors
/// `ServiceError::Db`.
pub fn similar_account_ids(
    ctx: &Context,
    typed: &[&str],
    exclude: Option<i64>,
    limit: usize,
) -> Result<Vec<i64>, ServiceError> {
    let index = current(ctx)?;
    let usage = Usage::default();
    let mut ids: Vec<i64> = Vec::new();
    for text in typed.iter().filter(|t| !t.trim().is_empty()) {
        for hit in index.search(SearchScope::AnyAccount, text, limit, &usage) {
            if Some(hit.id) != exclude && !ids.contains(&hit.id) {
                ids.push(hit.id);
            }
        }
    }
    ids.truncate(limit);
    Ok(ids)
}

fn usage_of(ctx: &Context, actor: &Actor, scope: SearchScope) -> Result<Usage, ServiceError> {
    let key = pref_key(scope);
    let value = ctx.db.read(|conn| store::pref(conn, actor.user_id, &key))?;
    Ok(value.as_ref().map(parse_usage).unwrap_or_default())
}

fn parse_usage(value: &Value) -> Usage {
    let recent = value
        .get("recent")
        .and_then(Value::as_array)
        .map(|ids| ids.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default();
    let counts = value
        .get("counts")
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .filter_map(|(k, v)| {
                    Some((k.parse::<i64>().ok()?, u32::try_from(v.as_u64()?).ok()?))
                })
                .collect()
        })
        .unwrap_or_default();
    Usage { recent, counts }
}

fn usage_json(usage: &Usage) -> Value {
    let counts: serde_json::Map<String, Value> = usage
        .counts
        .iter()
        .map(|(id, n)| (id.to_string(), json!(n)))
        .collect();
    json!({ "recent": usage.recent, "counts": counts })
}

/// The active records of a list that match `text`, best first (see [`Index::search`]). Empty
/// text gives nothing. Only reads.
///
/// # Errors
/// `ERR_TEXT_TOO_LONG` (`text`, value `max`) for more than [`MAX_QUERY_CHARS`] characters;
/// `ServiceError::Db`.
pub fn search(
    ctx: &Context,
    actor: &Actor,
    scope: SearchScope,
    text: &str,
    limit: u32,
) -> Result<Vec<SearchHit>, ServiceError> {
    // A huge paste is refused before it is turned into search keys, like the account search.
    check_text(text)?;
    let index = current(ctx)?;
    let usage = usage_of(ctx, actor, scope)?;
    let limit = usize::try_from(limit).unwrap_or(MAX_HITS);
    Ok(index.search(scope, text, limit, &usage))
}

/// The list for `Alt+Down`: the last 10 chosen, then the rest (see [`Index::recent`]). Only
/// reads.
///
/// # Errors
/// `ServiceError::Db`.
pub fn recent(
    ctx: &Context,
    actor: &Actor,
    scope: SearchScope,
) -> Result<Vec<SearchHit>, ServiceError> {
    let index = current(ctx)?;
    let usage = usage_of(ctx, actor, scope)?;
    Ok(index.recent(scope, &usage))
}

/// Records that the user chose a record, so it comes first next time (PRD FR-7.41). Writes the
/// user's own preference `recent.<list>` and nothing else.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id` if there is no active record with it in the list.
pub fn touch(
    ctx: &Context,
    actor: &Actor,
    scope: SearchScope,
    id: i64,
) -> Result<(), ServiceError> {
    let index = current(ctx)?;
    if index.active_hit(scope, id).is_none() {
        return Err(ServiceError::reject_field(ErrorCode::ErrNotFound, "id"));
    }
    let key = pref_key(scope);
    ctx.db.write(|tx| {
        let mut usage = store::pref(tx, actor.user_id, &key)?
            .as_ref()
            .map(parse_usage)
            .unwrap_or_default();
        usage.recent.retain(|r| *r != id);
        usage.recent.insert(0, id);
        usage.recent.truncate(RECENT_KEPT);
        let count = usage.counts.entry(id).or_insert(0);
        *count = count.saturating_add(1);
        store::set_pref(tx, actor.user_id, &key, &usage_json(&usage))
    })
}

/// The most characters a typed search text may have. Longer text is refused, so a huge paste
/// is never turned into keys.
pub const MAX_QUERY_CHARS: usize = 200;
/// The most accounts `balances` answers for in one call.
pub const MAX_BALANCES: usize = 500;

fn check_text(text: &str) -> Result<(), ServiceError> {
    if text.chars().take(MAX_QUERY_CHARS.saturating_add(1)).count() > MAX_QUERY_CHARS {
        return Err(ServiceError::reject_with(
            ErrorCode::ErrTextTooLong,
            Some("text"),
            "max",
            i64::try_from(MAX_QUERY_CHARS).unwrap_or(i64::MAX),
        ));
    }
    Ok(())
}

fn check_scope(scope: AccountScope, control_id: Option<i64>) -> Result<(), ServiceError> {
    if scope == AccountScope::Control && control_id.is_none() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrRequired,
            "control_id",
        ));
    }
    Ok(())
}

/// The preference key of the recent list of an account field: sellers, buyers and any-account
/// fields have their own, and each control has its own (`recent.control_<id>`).
fn account_usage_key(scope: AccountScope, control_id: Option<i64>) -> String {
    match (scope, control_id) {
        (AccountScope::Seller, _) => pref_key(SearchScope::Seller),
        (AccountScope::Buyer, _) => pref_key(SearchScope::Buyer),
        (AccountScope::Control, Some(id)) => format!("recent.control_{id}"),
        (AccountScope::Any | AccountScope::Control, _) => pref_key(SearchScope::AnyAccount),
    }
}

fn usage_by_key(ctx: &Context, actor: &Actor, key: &str) -> Result<Usage, ServiceError> {
    let value = ctx.db.read(|conn| store::pref(conn, actor.user_id, key))?;
    Ok(value.as_ref().map(parse_usage).unwrap_or_default())
}

/// The last accounts this user chose in a field, then the rest (`Alt+Down`, PRD FR-7.41). Each
/// is an `AccountHit`, like a search result. Only reads.
///
/// # Errors
/// `ERR_REQUIRED` on `control_id` for scope `control`; `ServiceError::Db`.
pub fn recent_accounts(
    ctx: &Context,
    actor: &Actor,
    input: &AccountsRecentInput,
) -> Result<Vec<AccountHit>, ServiceError> {
    check_scope(input.scope, input.control_id)?;
    let index = current(ctx)?;
    let usage = usage_by_key(
        ctx,
        actor,
        &account_usage_key(input.scope, input.control_id),
    )?;
    Ok(index.recent_accounts(input.scope, input.control_id, &usage))
}

/// Records that an account was chosen in a field, so it comes first next time (PRD FR-7.41).
/// Writes the user's own preference `recent.<list>` and nothing else (Code Standards B29).
///
/// # Errors
/// `ERR_REQUIRED` on `control_id` for scope `control`; `ERR_NOT_FOUND` on `id` if there is no
/// active account with it in the field's list.
pub fn touch_account(
    ctx: &Context,
    actor: &Actor,
    input: &AccountsTouchInput,
) -> Result<(), ServiceError> {
    check_scope(input.scope, input.control_id)?;
    let index = current(ctx)?;
    if !index.has_active_account(input.scope, input.control_id, input.id) {
        return Err(ServiceError::reject_field(ErrorCode::ErrNotFound, "id"));
    }
    let key = account_usage_key(input.scope, input.control_id);
    ctx.db.write(|tx| {
        let mut usage = store::pref(tx, actor.user_id, &key)?
            .as_ref()
            .map(parse_usage)
            .unwrap_or_default();
        usage.recent.retain(|r| *r != input.id);
        usage.recent.insert(0, input.id);
        usage.recent.truncate(RECENT_KEPT);
        let count = usage.counts.entry(input.id).or_insert(0);
        *count = count.saturating_add(1);
        store::set_pref(tx, actor.user_id, &key, &usage_json(&usage))
    })
}

/// The accounts of a field that match the typed text, best first: exact full code, exact serial
/// (a short code, PRD FR-7.39), names that start with the text, accounts used before, close
/// spellings. Only reads. A short code needs no `Enter` here: the list previews the top match
/// while the user types, and the code is only taken by [`resolve_code`] (PRD FR-7.38).
///
/// # Errors
/// `ERR_REQUIRED` on `control_id` for scope `control`; `ERR_TEXT_TOO_LONG` on `text`;
/// `ServiceError::Db`.
pub fn search_accounts(
    ctx: &Context,
    actor: &Actor,
    input: &SearchAccountsInput,
) -> Result<Vec<AccountHit>, ServiceError> {
    check_scope(input.scope, input.control_id)?;
    check_text(&input.text)?;
    let index = current(ctx)?;
    let usage = usage_by_key(
        ctx,
        actor,
        &account_usage_key(input.scope, input.control_id),
    )?;
    let limit = usize::try_from(input.limit).unwrap_or(MAX_HITS);
    Ok(index.search_accounts(input.scope, input.control_id, &input.text, limit, &usage))
}

/// The full code a new account would get, if the typed digits are a well-formed code of the
/// field's series and the serial is a normal one (PRD FR-10.3).
fn well_formed(control_code: &str, digits: &str) -> Option<u32> {
    let serial = digits.strip_prefix(control_code)?;
    if serial.len() != 4 {
        return None;
    }
    serial.parse().ok()
}

/// Takes a typed code on `Enter` or on leaving the field (PRD FR-7.38, FR-7.39, FR-7.42).
///
/// - 1 to 4 digits are a short code: the serial in a seller or buyer field, the seller and the
///   buyer with that serial in a field that takes any account. Serial 0 and 9000 and up are
///   `reserved`. Other accounts have no short code.
/// - More digits are a full code, which must be in the field's series.
/// - Anything that is not digits is a name, not a code: `unknown`, nothing suggested.
///
/// # Errors
/// `ERR_REQUIRED` on `control_id` (scope `control`) or on `text` (empty); `ERR_TEXT_TOO_LONG`.
pub fn resolve_code(ctx: &Context, input: &ResolveCodeInput) -> Result<ResolvedCode, ServiceError> {
    check_scope(input.scope, input.control_id)?;
    check_text(&input.text)?;
    let text = input.text.trim();
    if text.is_empty() {
        return Err(ServiceError::reject_field(ErrorCode::ErrRequired, "text"));
    }
    let index = current(ctx)?;
    let answer = |status, account: Option<AccountHit>, candidates, suggested_code| ResolvedCode {
        status,
        account,
        candidates,
        suggested_code,
    };
    let unknown =
        |suggested: Option<String>| answer(ResolveStatus::Unknown, None, Vec::new(), suggested);
    if !text.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(unknown(None));
    }
    let series = |scope: AccountScope| match scope {
        AccountScope::Seller => Some(SELLER_CONTROL.to_owned()),
        AccountScope::Buyer => Some(BUYER_CONTROL.to_owned()),
        _ => None,
    };
    if text.len() <= SHORT_CODE_DIGITS {
        if input.scope == AccountScope::Control {
            return Ok(unknown(None));
        }
        let Some(serial) = short_serial(text) else {
            return Ok(answer(ResolveStatus::Reserved, None, Vec::new(), None));
        };
        let all = index.accounts_with_serial(input.scope, serial);
        let mut active: Vec<AccountHit> = all.iter().filter(|a| a.is_active).cloned().collect();
        return Ok(match (active.len(), all.first()) {
            (1, _) => answer(ResolveStatus::Found, active.pop(), Vec::new(), None),
            (0, None) => unknown(series(input.scope).map(|code| format!("{code}{serial:04}"))),
            (0, Some(first)) => answer(
                ResolveStatus::Inactive,
                Some(first.clone()),
                Vec::new(),
                None,
            ),
            _ => answer(ResolveStatus::Several, None, active, None),
        });
    }
    if let Some(hit) = index.account_with_code(input.scope, input.control_id, text) {
        let status = if hit.is_active {
            ResolveStatus::Found
        } else {
            ResolveStatus::Inactive
        };
        return Ok(answer(status, Some(hit), Vec::new(), None));
    }
    // Not found: if the digits are a well-formed code of the field's series, say which add
    // would make it, or that its serial is the system's.
    let control_code = match input.scope {
        AccountScope::Control => match input.control_id {
            Some(id) => ctx
                .db
                .read(|conn| mandi_db::accounts::control_by_id(conn, id))?
                .map(|c| c.code),
            None => None,
        },
        other => series(other),
    };
    let serial = control_code
        .as_deref()
        .and_then(|code| well_formed(code, text));
    Ok(match serial {
        Some(serial) if short_serial(&serial.to_string()).is_some() => {
            unknown(Some(text.to_owned()))
        }
        Some(_) => answer(ResolveStatus::Reserved, None, Vec::new(), None),
        None => unknown(None),
    })
}

/// The balances of some accounts, read from the posted journal (PRD FR-6.1). An account with no
/// posted line has the zero balance. Only reads.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `ids` for more than [`MAX_BALANCES`] ids; `ServiceError::Db`.
pub fn balances(ctx: &Context, ids: &[i64]) -> Result<Vec<AccountBalance>, ServiceError> {
    if ids.len() > MAX_BALANCES {
        return Err(ServiceError::reject_field(ErrorCode::ErrOutOfRange, "ids"));
    }
    let totals = ctx
        .db
        .read(|conn| mandi_db::ledger::totals_for_accounts(conn, ids))?;
    ids.iter()
        .map(|id| {
            let balance = match totals.get(id) {
                Some(found) => crate::ledger::balance_of(*found)?,
                None => Balance::ZERO,
            };
            Ok(AccountBalance { id: *id, balance })
        })
        .collect()
}

/// The most truck-number results of the global search (PRD FR-5.4).
const GLOBAL_TRUCK_HITS: i64 = 5;
/// A truck number is looked up from this many typed characters, so one letter does not list
/// every truck.
const GLOBAL_TRUCK_MIN_CHARS: usize = 2;

/// The بکری that the typed text names (PRD FR-5.4): the one whose number is the typed digits,
/// then the newest ones whose truck number starts with the text.
fn bikri_hits(ctx: &Context, text: &str) -> Result<Vec<GlobalHit>, ServiceError> {
    let typed = text.trim();
    if typed.is_empty() {
        return Ok(Vec::new());
    }
    ctx.db.read(|conn| {
        let mut hits: Vec<mandi_domain::core::bikri::BikriHit> = Vec::new();
        if typed.chars().all(|c| c.is_ascii_digit())
            && let Ok(number) = typed.parse::<i64>()
            && let Some(hit) = mandi_db::bikri::saved::hit_by_number(conn, number)?
        {
            hits.push(hit);
        }
        if typed.chars().count() >= GLOBAL_TRUCK_MIN_CHARS {
            for hit in mandi_db::bikri::saved::hits_by_truck(conn, typed, GLOBAL_TRUCK_HITS)? {
                if !hits.iter().any(|h| h.id == hit.id) {
                    hits.push(hit);
                }
            }
        }
        Ok(hits
            .into_iter()
            .map(|hit| GlobalHit::Bikri { hit })
            .collect())
    })
}

/// Global search (`Ctrl+K`, PRD FR-5.4): accounts, searched like a field that takes any
/// account, then the بکری whose number is the typed digits, then the newest بکری whose truck
/// number starts with the text. The whole answer stays within `limit`. Only reads.
///
/// # Errors
/// As [`search_accounts`].
pub fn global(
    ctx: &Context,
    actor: &Actor,
    input: &SearchGlobalInput,
) -> Result<Vec<GlobalHit>, ServiceError> {
    check_text(&input.text)?;
    let limit = usize::try_from(input.limit)
        .unwrap_or(MAX_HITS)
        .min(MAX_HITS);
    let mut bikri = bikri_hits(ctx, &input.text)?;
    // The accounts keep at least one place.
    bikri.truncate(limit.saturating_sub(1));
    let account_limit = limit.saturating_sub(bikri.len());
    let hits = search_accounts(
        ctx,
        actor,
        &SearchAccountsInput {
            scope: AccountScope::Any,
            control_id: None,
            text: input.text.clone(),
            limit: u32::try_from(account_limit).unwrap_or(input.limit),
        },
    )?;
    let mut out: Vec<GlobalHit> = hits
        .into_iter()
        .map(|hit| GlobalHit::Account { hit })
        .collect();
    out.extend(bikri);
    Ok(out)
}
