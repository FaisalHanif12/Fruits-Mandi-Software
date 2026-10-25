//! الگ کھاتہ: the separate khatas and their statement (PRD 16.6, FR-16.6, FR-16.7, E54, plan 17).
//! The saves of money paid out and received are in `vouchers`.
//!
//! A khata is an account under control 0305. It is never income or expense, never split and never
//! in a partner's total (PRD C22). Its balance is signed from its side: credit less debit, so
//! plus means the shop holds money for it and minus means it has taken more than it brought. The
//! statement is read from the live lines of the account (a voucher that was cancelled or edited
//! leaves no trace), so its closing figure is the balance of the account.
//!
//! Worked numbers (PRD E2E-38, خاندانی کھاتہ): paid out 10,000 on 10-10-26 (-10,000); 4,000 back on
//! 11-10-26 (-6,000); then 10 پیٹی کیلا sold at 900 for ارشد, 9,000 to the khata: +3,000.

use mandi_db::accounts as account_repo;
use mandi_db::open_lines::{self, LiveLine};
use mandi_domain::core::accounts::{AccountRow, KHATA_CONTROL, RecordRef};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::khata::{
    KhataAccount, KhataCreateInput, KhataEntryRow, KhataRowKind, KhataStatement,
    KhataStatementInput, KhataTotals,
};
use mandi_domain::core::masters::AccountToMake;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::text;
use mandi_domain::core::translit::Kind;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::khata::signed_balance;
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::page;
use crate::search;
use crate::spelling::{self, NamePair};

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn record(row: &AccountRow) -> RecordRef {
    RecordRef {
        id: row.id,
        code: row.code.clone(),
        name_ur: row.name_ur.clone(),
        name_en: row.name_en.clone(),
        tag: row.tag.clone(),
    }
}

/// The balance of a khata, signed from its side: credit less debit.
fn balance_of(
    conn: &mandi_db::rusqlite::Connection,
    account_id: i64,
) -> Result<Paisa, ServiceError> {
    let totals = mandi_db::ledger::totals_for_accounts(conn, &[account_id])?;
    let (dr, cr) = totals
        .get(&account_id)
        .map_or((0, 0), |t| (t.dr_paisa, t.cr_paisa));
    signed_balance(paisa(cr)?, paisa(dr)?).map_err(money_error)
}

/// A khata read back: the account, its note and its balance.
fn account_of(
    conn: &mandi_db::rusqlite::Connection,
    id: i64,
) -> Result<KhataAccount, ServiceError> {
    let row = account_repo::get(conn, id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "khata_account_id"))?;
    let facts = account_repo::facts(conn, id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "khata_account_id"))?;
    if facts.control_code != KHATA_CONTROL {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrNotFound,
            "khata_account_id",
        ));
    }
    Ok(KhataAccount {
        account: record(&row),
        note: row.note.clone(),
        balance_paisa: balance_of(conn, id)?,
    })
}

/// Makes a separate khata (an account under control 0305), in one transaction (PRD 16.6). Both
/// names are required, each in its own script (PRD FR-7.28); a spelling made only by the rules
/// is confirmed first (PRD FR-7.66). The search index is refreshed after the commit.
///
/// # Errors
/// The name codes on `name_ur` and `name_en`; `ERR_SPELLING_UNCONFIRMED`; the text codes on
/// `note`; `ERR_SERIES_FULL`. Nothing is written when any of them is returned.
pub fn create(
    ctx: &Context,
    actor: &Actor,
    input: &KhataCreateInput,
) -> Result<KhataAccount, ServiceError> {
    let mut errors = FieldErrors::default();
    let name_ur = errors.take(validate::name_ur("name_ur", &input.name_ur));
    let name_en = errors.take(validate::name_en("name_en", &input.name_en));
    let note = errors
        .take(validate::free_text(
            "note",
            input.note.as_deref().unwrap_or(""),
        ))
        .unwrap_or_default();
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
    let (id, learned) = ctx.db.write(|tx| -> Result<_, ServiceError> {
        let now = ctx.now_text();
        let made = AccountToMake {
            control_code: "0305",
            search_ur: text::search_ur(&name_ur),
            search_en: text::search_en(&name_en),
            search_phonetic: text::search_phonetic(&name_en),
            name_ur: name_ur.clone(),
            name_en: name_en.clone(),
        };
        let (id, code) = mandi_db::masters::create_account(tx, &made, actor.user_id, &now)?
            .ok_or_else(|| {
                ServiceError::reject_with(ErrorCode::ErrSeriesFull, None, "control", 0)
            })?;
        if !note.is_empty() {
            account_repo::set_account_note(tx, id, &note, actor.user_id, &now)?;
        }
        let learned = checked.learn(ctx, actor, tx)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "create", "account")
                .entity_id(id)
                .new_value(json!({
                    "code": code,
                    "name_ur": name_ur,
                    "name_en": name_en,
                    "note": note,
                    "made_by": "khata",
                }))
                .into_entry(),
        )?;
        Ok((id, learned))
    })?;
    learned.after_commit(ctx);
    search::refresh_account(ctx, id);
    ctx.db.read(|conn| account_of(conn, id))
}

/// What a line of a khata is (PRD FR-16.6, FR-16.7).
fn kind_of(line: &LiveLine) -> KhataRowKind {
    match (line.voucher_type.as_deref(), line.posting_type.as_str()) {
        (Some("khata_out"), _) => KhataRowKind::PaidOut,
        (Some("khata_in"), _) => KhataRowKind::Received,
        (_, "bikri_first" | "bikri_later") => KhataRowKind::Sale,
        _ => KhataRowKind::Other,
    }
}

/// A khata's entries in date order with the signed running balance, one page (PRD FR-16.6,
/// FR-19.11). Read only. `from` and `to` limit the range; the opening is the balance at the end of
/// the day before `from`, and the totals are over the whole range.
///
/// # Errors
/// `ERR_NOT_FOUND` on `khata_account_id`; the date codes on `from` and `to`.
pub fn statement(
    ctx: &Context,
    input: &KhataStatementInput,
) -> Result<KhataStatement, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let mut bound = |field: &str, text: Option<&String>| -> Option<String> {
        let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
        errors
            .take(validate::date(field, text, today))
            .map(BusinessDate::to_iso)
    };
    let from = bound("from", input.from.as_ref());
    let to = bound("to", input.to.as_ref());
    errors.finish()?;
    ctx.db.read(|conn| {
        let khata = account_of(conn, input.khata_account_id)?;
        let lines = open_lines::live_lines(conn, &[khata.account.id])?;
        let mut opening = Paisa::ZERO;
        let mut running = Paisa::ZERO;
        let mut totals = KhataTotals {
            paid_out_paisa: Paisa::ZERO,
            received_paisa: Paisa::ZERO,
        };
        let mut rows: Vec<KhataEntryRow> = Vec::new();
        for line in &lines {
            let (dr, cr) = (paisa(line.dr_paisa)?, paisa(line.cr_paisa)?);
            let amount = signed_balance(cr, dr).map_err(money_error)?;
            running = running.checked_add(amount).map_err(money_error)?;
            if from
                .as_deref()
                .is_some_and(|f| line.entry_date.as_str() < f)
            {
                opening = running;
                continue;
            }
            if to.as_deref().is_some_and(|t| line.entry_date.as_str() > t) {
                continue;
            }
            totals.paid_out_paisa = totals.paid_out_paisa.checked_add(dr).map_err(money_error)?;
            totals.received_paisa = totals.received_paisa.checked_add(cr).map_err(money_error)?;
            let kind = kind_of(line);
            rows.push(KhataEntryRow {
                kind,
                entry_id: line.entry_id,
                voucher_id: line.voucher_id,
                number: if kind == KhataRowKind::Sale {
                    line.bikri_number
                } else {
                    line.number
                },
                entry_date: line.entry_date.clone(),
                reason: line.reason.clone(),
                narration: line.narration.clone(),
                amount_paisa: amount,
                running_paisa: running,
            });
        }
        let closing = rows.last().map_or(opening, |r| r.running_paisa);
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        Ok(KhataStatement {
            khata,
            opening_paisa: opening,
            closing_paisa: closing,
            rows: page::slice(rows, input.offset, input.limit),
            total_count,
            totals,
        })
    })
}
