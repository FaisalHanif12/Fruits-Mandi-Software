//! The چکتا buyers (plan 64, PRD FR-7.29, FR-7.69, Section 13): the list the buyer picker of a بکری
//! line offers beside ordinary buyers, and the add on the spot.
//!
//! A چکتا buyer is not an account and has no ledger (PRD FR-13.6). Searching only reads
//! (Code Standards B29). Adding one is one transaction with its audit row (S7); a person who is
//! already in the list (the same Urdu name and phone) is returned, not made twice, so his name is
//! spelt the same way next time.

use mandi_db::chukta as repo;
use mandi_domain::core::chukta::{
    self, ChuktaAddInput, ChuktaBuyer, ChuktaSearchInput, SEARCH_LIMIT,
};
use mandi_domain::core::error::ErrorCode;
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::search;

/// The most characters of the typed text; longer text is refused before it becomes keys.
const MAX_TEXT_CHARS: usize = search::MAX_QUERY_CHARS;

/// The چکتا buyers whose names or phone hold the typed text, newest first (by the day of his last
/// sale line), at most 10 (PRD FR-7.29). Empty text lists the newest ten. Only reads.
///
/// # Errors
/// `ERR_TEXT_TOO_LONG` on `text`; `ServiceError::Db`.
pub fn search(ctx: &Context, input: &ChuktaSearchInput) -> Result<Vec<ChuktaBuyer>, ServiceError> {
    if input
        .text
        .chars()
        .take(MAX_TEXT_CHARS.saturating_add(1))
        .count()
        > MAX_TEXT_CHARS
    {
        return Err(ServiceError::reject_with(
            ErrorCode::ErrTextTooLong,
            Some("text"),
            "max",
            i64::try_from(MAX_TEXT_CHARS).unwrap_or(i64::MAX),
        ));
    }
    let everyone = ctx
        .db
        .read(|conn| repo::all_newest_first(conn).map_err(ServiceError::from))?;
    Ok(everyone
        .into_iter()
        .filter(|b| chukta::matches(&input.text, &b.name_ur, &b.name_en, b.phone.as_deref()))
        .take(SEARCH_LIMIT)
        .collect())
}

/// Adds a چکتا buyer from the short form of the picker (PRD FR-7.69, FR-13.3): both names, each in
/// its own script, and an optional phone. A person with the same key is returned as he is. The
/// search index learns him after the commit (Code Standards B27).
///
/// # Errors
/// `ERR_CHUKTA_NAME_REQUIRED`, `ERR_NAME_UR_SCRIPT`, `ERR_NAME_EN_SCRIPT` or `ERR_PHONE_INVALID` on
/// `name_ur`, `name_en` or `phone`, all together; `ServiceError::Db`.
pub fn add(
    ctx: &Context,
    actor: &Actor,
    input: &ChuktaAddInput,
) -> Result<ChuktaBuyer, ServiceError> {
    let clean = chukta::clean(input)?;
    let now = ctx.now_text();
    let (buyer, added) = ctx.db.write(|tx| {
        let (id, added) = repo::ensure(tx, &clean, actor.user_id, &now)?;
        if added {
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "create", "chukta_buyer")
                    .entity_id(id)
                    .new_value(json!({
                        "name_ur": clean.name_ur,
                        "name_en": clean.name_en,
                        "has_phone": clean.phone.is_some(),
                    }))
                    .into_entry(),
            )?;
        }
        let buyer = repo::get(tx, id)?
            .ok_or_else(|| ServiceError::Data("a چکتا buyer just made is missing".to_owned()))?;
        Ok::<_, ServiceError>((buyer, added))
    })?;
    if added {
        search::refresh_chukta(ctx, buyer.id);
    }
    Ok(buyer)
}

/// Makes sure a چکتا buyer named on a sale line is in the list, inside the transaction of that
/// save (PRD FR-13.2): a line that carries his names and phone leaves him in the list for the next
/// time. Returns the row id and whether he was added. No audit row of its own: the بکری save writes
/// its own, and the list holds no money.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn ensure_in_save(
    tx: &mandi_db::rusqlite::Connection,
    person: &chukta::CleanChukta,
    user_id: i64,
    now: &str,
) -> Result<(i64, bool), ServiceError> {
    Ok(repo::ensure(tx, person, user_id, now)?)
}
