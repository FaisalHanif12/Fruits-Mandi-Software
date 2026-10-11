//! The master lists of PRD 10.3 (plan 06): add, edit, make inactive, list, next code.
//!
//! One user action is one transaction (CLAUDE.md rule 3): the master, the account it makes
//! (expense head, bank account), the pairs the user confirmed in the spelling check, and the
//! audit row are written together or not at all. The search index is updated after the commit
//! (Code Standards B27). Nothing is ever deleted (PRD E21).

use mandi_db::masters as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::masters::{self, ColValue, MasterInput, MasterKind, MasterRow, Prepared};
use mandi_domain::core::translit::Kind;
use serde_json::{Map, Value, json};

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::search;
use crate::spelling::{self, NamePair, SpellingCheck};

/// The kind of word the pairs learned from a master are (Architecture 15.3 `translit_pairs`).
fn learned_kind(kind: MasterKind) -> Kind {
    match kind {
        MasterKind::City | MasterKind::Adda => Kind::Place,
        MasterKind::Item | MasterKind::Packing => Kind::Item,
        MasterKind::Store | MasterKind::ExpenseHead => Kind::Word,
        MasterKind::Staff | MasterKind::BankAccount | MasterKind::Marka | MasterKind::Narration => {
            Kind::Name
        }
    }
}

/// Every master of one kind: kinds with a code in code order, the others in the order they were
/// added. Inactive ones only when asked.
///
/// # Errors
/// `ServiceError::Db`.
pub fn list(
    ctx: &Context,
    kind: MasterKind,
    include_inactive: bool,
) -> Result<Vec<MasterRow>, ServiceError> {
    ctx.db
        .read(|conn| repo::list(conn, kind, include_inactive).map_err(ServiceError::from))
}

/// The next free code of a kind, empty for kinds without a code.
///
/// # Errors
/// `ServiceError::Db`.
pub fn next_code(ctx: &Context, kind: MasterKind) -> Result<String, ServiceError> {
    ctx.db
        .read(|conn| repo::next_code(conn, kind).map_err(ServiceError::from))?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrSeriesFull, "code"))
}

/// The values of the columns written, for the audit row. The search columns are left out: they
/// follow from the names.
fn written_json(code: Option<&str>, fields: &[(&'static str, ColValue)]) -> Value {
    let mut map = Map::new();
    if let Some(code) = code {
        map.insert("code".to_owned(), json!(code));
    }
    for (name, value) in fields {
        if name.starts_with("search_") {
            continue;
        }
        map.insert(
            (*name).to_owned(),
            match value {
                ColValue::Text(t) => json!(t),
                ColValue::Int(i) => json!(i),
                ColValue::Null => Value::Null,
            },
        );
    }
    Value::Object(map)
}

fn row_json(row: &MasterRow) -> Result<Value, ServiceError> {
    serde_json::to_value(row).map_err(|e| ServiceError::Data(e.to_string()))
}

/// Adds a master (`id` null) or edits one (`id` and the `version` that was read).
///
/// # Errors
/// The field errors of `mandi_core::masters::prepare`; `ERR_SPELLING_UNCONFIRMED` (`name_ur`,
/// `name_en`); `ERR_NOT_FOUND` on `id` or on a record it points at; `ERR_CODE_TAKEN` on `code`;
/// `ERR_VERSION_CHANGED`. Nothing is written when any of them is returned.
pub fn save(ctx: &Context, actor: &Actor, input: &MasterInput) -> Result<MasterRow, ServiceError> {
    let prepared = masters::prepare(input)?;
    let kind = prepared.kind;
    // The spelling check reads only (learned pairs, word list): it can refuse before the
    // transaction starts. The pairs it teaches are written inside the transaction below.
    let checked: Option<SpellingCheck> = match &prepared.names {
        Some(names) => Some(spelling::check(
            ctx,
            &NamePair {
                field_ur: names.field_ur,
                name_ur: &names.name_ur,
                field_en: names.field_en,
                name_en: &names.name_en,
                kind: learned_kind(kind),
            },
            &prepared.confirmed_rules_words,
        )?),
        None => None,
    };
    let (row, learned) = ctx
        .db
        .write(|tx| write(ctx, actor, tx, &prepared, checked.as_ref()))?;
    if let Some(learned) = learned {
        learned.after_commit(ctx);
    }
    search::refresh(ctx, kind, row.id());
    Ok(row)
}

fn write(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    prepared: &Prepared,
    checked: Option<&SpellingCheck>,
) -> Result<(MasterRow, Option<crate::spelling::Learned>), ServiceError> {
    let kind = prepared.kind;
    let now = ctx.now_text();
    for reference in &prepared.references {
        if !repo::exists(tx, reference.table, reference.id)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNotFound,
                reference.field,
            ));
        }
    }
    let old = match prepared.id {
        Some(id) => {
            let row = repo::get(tx, kind, id)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
            if prepared.version != Some(row.version()) {
                return Err(ServiceError::reject(ErrorCode::ErrVersionChanged));
            }
            Some(row)
        }
        None => None,
    };
    // A new record with no code typed gets the next one; an edit with no code keeps its own.
    let code: Option<String> = match (&prepared.code, kind.has_code(), &old) {
        (Some(code), true, _) => Some(code.clone()),
        (None, true, None) => Some(
            repo::next_code(tx, kind)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrSeriesFull, "code"))?,
        ),
        _ => None,
    };
    if let Some(code) = &code
        && repo::code_taken(tx, kind, code, prepared.id)?
    {
        return Err(ServiceError::reject_field(ErrorCode::ErrCodeTaken, "code"));
    }

    // The account of an expense head or a bank account (same transaction).
    let account_id = match (&prepared.account, prepared.id) {
        (Some(account), None) => {
            let made =
                repo::create_account(tx, account, actor.user_id, &now)?.ok_or_else(|| {
                    ServiceError::Rejected(vec![crate::error::Rejection {
                        code: ErrorCode::ErrSeriesFull,
                        field: None,
                        values: std::collections::BTreeMap::from([(
                            "control".to_owned(),
                            mandi_domain::core::error::ErrorValue::Text(
                                account.control_code.to_owned(),
                            ),
                        )]),
                    }])
                })?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "create", "account")
                    .entity_id(made.0)
                    .new_value(json!({
                        "code": made.1,
                        "name_ur": account.name_ur,
                        "name_en": account.name_en,
                        "made_by": kind.as_str(),
                    }))
                    .into_entry(),
            )?;
            Some(made.0)
        }
        (Some(account), Some(id)) => {
            let existing = repo::account_of(tx, kind, id)?
                .ok_or_else(|| ServiceError::Data("master without its account".to_owned()))?;
            repo::rename_account(tx, existing, account, actor.user_id, &now)?;
            None
        }
        (None, _) => None,
    };

    let id = match prepared.id {
        None => repo::insert(
            tx,
            kind,
            code.as_deref(),
            account_id,
            &prepared.fields,
            actor.user_id,
            &now,
        )?,
        Some(id) => {
            let version = prepared.version.unwrap_or_default();
            let changed = repo::update(
                tx,
                kind,
                id,
                version,
                code.as_deref(),
                &prepared.fields,
                actor.user_id,
                &now,
            )?;
            if !changed {
                return Err(ServiceError::reject(ErrorCode::ErrVersionChanged));
            }
            id
        }
    };

    let learned = checked
        .map(|check| check.learn(ctx, actor, tx))
        .transpose()?;
    let row = repo::get(tx, kind, id)?
        .ok_or_else(|| ServiceError::Data("the saved master is missing".to_owned()))?;
    let written = written_json(code.as_deref(), &prepared.fields);
    let mut event = Event::new(
        Some(actor.user_id),
        if old.is_some() { "edit" } else { "create" },
        kind.as_str(),
    )
    .entity_id(id)
    .new_value(written);
    if let Some(old) = &old {
        event = event.old(row_json(old)?);
    }
    ctx.db.auditor().write(tx, &event.into_entry())?;
    Ok((row, learned))
}

/// Makes a master active or inactive (PRD E21: a used master can only be made inactive). The
/// account of an expense head or bank account follows. Asking for the state it already has
/// changes nothing and writes nothing.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn set_active(
    ctx: &Context,
    actor: &Actor,
    kind: MasterKind,
    id: i64,
    is_active: bool,
) -> Result<MasterRow, ServiceError> {
    let (row, changed) = ctx.db.write(|tx| {
        let old = repo::get(tx, kind, id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
        if old.is_active() == is_active {
            return Ok::<_, ServiceError>((old, false));
        }
        let now = ctx.now_text();
        repo::set_active(tx, kind, id, is_active, actor.user_id, &now)?;
        if let Some(account_id) = repo::account_of(tx, kind, id)? {
            repo::set_account_active(tx, account_id, is_active, actor.user_id, &now)?;
        }
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", kind.as_str())
                .entity_id(id)
                .old(json!({ "is_active": old.is_active() }))
                .new_value(json!({ "is_active": is_active }))
                .into_entry(),
        )?;
        let row = repo::get(tx, kind, id)?
            .ok_or_else(|| ServiceError::Data("the master is missing".to_owned()))?;
        Ok((row, true))
    })?;
    if changed {
        search::refresh(ctx, kind, id);
    }
    Ok(row)
}
