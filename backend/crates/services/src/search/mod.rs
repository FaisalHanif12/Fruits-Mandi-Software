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
    AccountBalance, AccountHit, AccountScope, Balance, GlobalHit, ResolveCodeInput, ResolveStatus,
    ResolvedCode, SearchAccountsInput, SearchGlobalInput,
};
use mandi_domain::core::accounts::{AccountsRecentInput, AccountsTouchInput};
use mandi_domain::core::codes::{self, CodeType};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::masters::MasterKind;
use mandi_domain::core::search::{SearchHit, SearchScope};
use serde_json::{Value, json};

pub use index::{Index, MAX_FULL_LIST, MAX_HITS, RECENT_KEPT, Usage, short_running};

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

/// The active records of a list that match `text`, best first (see [`Index::search_page`]). Empty
/// text gives the whole list: the records chosen last first, then the rest by code (plan 32 R7).
/// This is the first page; [`search_page`] takes an offset. Only reads.
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
    search_page(ctx, actor, scope, text, 0, limit)
}

/// One page of [`search`]: `limit` hits after skipping `offset`. A page shorter than `limit` is the
/// last one. Only reads.
///
/// # Errors
/// As [`search`].
pub fn search_page(
    ctx: &Context,
    actor: &Actor,
    scope: SearchScope,
    text: &str,
    offset: u32,
    limit: u32,
) -> Result<Vec<SearchHit>, ServiceError> {
    // A huge paste is refused before it is turned into search keys, like the account search.
    check_text(text)?;
    let index = current(ctx)?;
    let usage = usage_of(ctx, actor, scope)?;
    let limit = usize::try_from(limit).unwrap_or(MAX_HITS);
    let offset = usize::try_from(offset).unwrap_or(0);
    Ok(index.search_page(scope, text, offset, limit, &usage))
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
        (AccountScope::Seller | AccountScope::LotSeller, _) => pref_key(SearchScope::Seller),
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

/// The accounts of a field that match the typed text, best first: exact full code, running number
/// (a short code, PRD 1.8 R5), names that start with the text, accounts used before, close
/// spellings. Empty text gives the whole list of the field: the accounts chosen last first, then
/// the rest by code, one page at a time (plan 32 R7). Only reads. A short code needs no `Enter`
/// here: the list previews the top match while the user types, and the code is only taken by
/// [`resolve_code`] (PRD FR-7.38).
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
    let offset = usize::try_from(input.offset).unwrap_or(0);
    Ok(index.search_accounts_page(
        input.scope,
        input.control_id,
        &input.text,
        offset,
        limit,
        &usage,
    ))
}

/// The code type of the sellers or the buyers of a field, if the field has one series.
fn series_type(scope: AccountScope) -> Option<CodeType> {
    match scope {
        AccountScope::Seller | AccountScope::LotSeller => Some(CodeType::Seller),
        AccountScope::Buyer => Some(CodeType::Buyer),
        AccountScope::Any | AccountScope::Control => None,
    }
}

/// Takes a typed code on `Enter` or on leaving the field (PRD FR-7.38, FR-7.39, FR-7.42, PRD 1.8
/// R5).
///
/// - A full code is looked up as it is, in the field's list: `0010` or `00111` in a seller field,
///   or the code of any account in a field that takes any account.
/// - A running number (`0`, `11`: digits with no leading zero, at most 7) is a short code: the
///   account with that running number in a seller or buyer field, and the seller and the buyer with
///   it in a field that takes any account. Other accounts have no short code.
/// - Anything that is not digits is a name, not a code: `unknown`, nothing suggested.
/// - `unknown` carries `suggested_code` when the field has one series and the typed text is a short
///   code or a full code of that series (the code an `F4` add would take).
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
    // A full code first: a code of the scheme never looks like a running number (it starts with 00).
    if let Some(hit) = index.account_with_code(input.scope, input.control_id, text) {
        let status = if hit.is_active {
            ResolveStatus::Found
        } else {
            ResolveStatus::Inactive
        };
        return Ok(answer(status, Some(hit), Vec::new(), None));
    }
    let series = series_type(input.scope);
    if let Some(running) = short_running(text) {
        if input.scope == AccountScope::Control {
            return Ok(unknown(None));
        }
        let all = index.accounts_with_running(input.scope, running);
        let mut active: Vec<AccountHit> = all.iter().filter(|a| a.is_active).cloned().collect();
        return Ok(match (active.len(), all.first()) {
            (1, _) => answer(ResolveStatus::Found, active.pop(), Vec::new(), None),
            (0, None) => unknown(series.and_then(|t| codes::build_code(t, running))),
            (0, Some(first)) => answer(
                ResolveStatus::Inactive,
                Some(first.clone()),
                Vec::new(),
                None,
            ),
            _ => answer(ResolveStatus::Several, None, active, None),
        });
    }
    // Not found. In the field of one control (other accounts keep the control code plus a 4-digit
    // serial, PRD FR-10.3), a well-formed code of that control is what an add would use; a system
    // serial (0000 and 9000 to 9999) can never be added.
    if input.scope == AccountScope::Control {
        let Some(id) = input.control_id else {
            return Ok(unknown(None));
        };
        let control = ctx
            .db
            .read(|conn| mandi_db::accounts::control_by_id(conn, id))?;
        let serial = control
            .and_then(|c| text.strip_prefix(c.code.as_str()).map(str::to_owned))
            .filter(|rest| rest.len() == 4)
            .and_then(|rest| rest.parse::<u32>().ok());
        return Ok(match serial {
            Some(serial)
                if (mandi_domain::core::accounts::FIRST_SERIAL
                    ..=mandi_domain::core::accounts::LAST_SERIAL)
                    .contains(&serial) =>
            {
                unknown(Some(text.to_owned()))
            }
            Some(_) => answer(ResolveStatus::Reserved, None, Vec::new(), None),
            None => unknown(None),
        });
    }
    // A well-formed full code of the field's series is what an add would use.
    Ok(unknown(
        series
            .filter(|t| codes::running_of(text, *t).is_some())
            .map(|_| text.to_owned()),
    ))
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

/// The series letters of the voucher kinds, Urdu and English (the same letters as the Fluent
/// messages `vouchers-number-<kind>`), for the search by number.
const SERIES_LETTERS: [(&str, &str, mandi_domain::core::vouchers::VoucherType); 13] = {
    use mandi_domain::core::vouchers::VoucherType as T;
    [
        ("و", "R", T::Receipt),
        ("ا", "P", T::Payment),
        ("پ", "A", T::Advance),
        ("خ", "E", T::ShopExpense),
        ("ذ", "D", T::PersonalExpense),
        ("ذو", "DR", T::PersonalReturn),
        ("ر", "L", T::MoneyLent),
        ("رو", "LR", T::MoneyLentReturn),
        ("س", "CI", T::CapitalIn),
        ("سو", "CO", T::CapitalOut),
        ("کد", "KO", T::KhataOut),
        ("کا", "KI", T::KhataIn),
        ("ج", "G", T::General),
    ]
};

/// Reads a voucher number the way it is typed (PRD FR-5.4, plans 15 and 17): the digits alone find
/// the voucher with that number in every series; the series letters first (`R-12`, `DR-3`, `و-12`,
/// `ذو-3`, with or without the dash and spaces) find one series.
fn voucher_query(typed: &str) -> Option<(i64, Option<mandi_domain::core::vouchers::VoucherType>)> {
    let text = typed.trim();
    let first = text.chars().next()?;
    let (series, digits) = if first.is_ascii_digit() {
        (None, text)
    } else {
        let letters_end = text
            .find(|c: char| c.is_ascii_digit() || c == '-' || c.is_whitespace())
            .unwrap_or(text.len());
        let letters = text.get(..letters_end)?;
        let kind = SERIES_LETTERS
            .iter()
            .find(|(ur, en, _)| letters == *ur || letters.eq_ignore_ascii_case(en))
            .map(|(_, _, kind)| *kind)?;
        let rest = text.get(letters_end..)?;
        (Some(kind), rest.trim_start_matches(['-', ' ']).trim())
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((digits.parse::<i64>().ok()?, series))
}

/// The vouchers that the typed text names by their number (PRD FR-5.4).
fn voucher_hits(ctx: &Context, text: &str) -> Result<Vec<GlobalHit>, ServiceError> {
    let Some((number, series)) = voucher_query(text) else {
        return Ok(Vec::new());
    };
    ctx.db.read(|conn| {
        Ok(mandi_db::vouchers::hits_by_number(conn, number, series)?
            .into_iter()
            .map(|hit| GlobalHit::Voucher { hit })
            .collect())
    })
}

/// Global search (`Ctrl+K`, PRD FR-5.4): accounts, searched like a field that takes any
/// account, then the بکری whose number is the typed digits, then the newest بکری whose truck
/// number starts with the text, then the vouchers with that number. The whole answer stays
/// within `limit`. Only reads.
///
/// # Errors
/// As [`search_accounts`].
pub fn global(
    ctx: &Context,
    actor: &Actor,
    input: &SearchGlobalInput,
) -> Result<Vec<GlobalHit>, ServiceError> {
    check_text(&input.text)?;
    // Global search needs a word: empty text is not "the whole list" here (that is for pickers).
    if input.text.trim().is_empty() {
        return Ok(Vec::new());
    }
    let limit = usize::try_from(input.limit)
        .unwrap_or(MAX_HITS)
        .min(MAX_HITS);
    let mut bikri = bikri_hits(ctx, &input.text)?;
    bikri.extend(voucher_hits(ctx, &input.text)?);
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
            offset: 0,
        },
    )?;
    let mut out: Vec<GlobalHit> = hits
        .into_iter()
        .map(|hit| GlobalHit::Account { hit })
        .collect();
    out.extend(bikri);
    Ok(out)
}
