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
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::masters::MasterKind;
use mandi_domain::core::search::{SearchHit, SearchScope};
use serde_json::{Value, json};

pub use index::{Index, MAX_FULL_LIST, MAX_HITS, RECENT_KEPT, Usage};

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
/// `ServiceError::Db`.
pub fn search(
    ctx: &Context,
    actor: &Actor,
    scope: SearchScope,
    text: &str,
    limit: u32,
) -> Result<Vec<SearchHit>, ServiceError> {
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
