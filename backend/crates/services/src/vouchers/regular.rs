//! Make a one-time person a regular account (PRD FR-13.8, plan 15). A person who started
//! coming often gets an account of his own, made with the rules of the account form, and his open
//! balance on the پختہ کھاتہ account moves to it with a جنرل entry, in one transaction.
//!
//! The move is one balanced entry on today's date between the person's key on the پختہ کھاتہ
//! account and the new account: the net he owes (debit less credit), or the net the firm holds for
//! him, whichever way it stands. It touches no income and no expense, so it writes no partner
//! share rows (PRD FR-6.5). His old bills are settled by that credit; the new account starts with
//! one bill dated today, so the age of the moved balance starts again (accepted limitation).

use mandi_db::accounts as account_repo;
use mandi_domain::core::accounts::{AccountType, BalanceSide, RecordRef};
use mandi_domain::core::bikri::PukhtaSide;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::vouchers::{MadeRegular, MakeRegularInput};
use mandi_domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use serde_json::json;

use super::common::{Role, rematch_party};
use crate::accounts;
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger::{self, PostingEvent};
use crate::search;

/// The code of the پختہ کھاتہ account of a side (PRD FR-13.1).
const fn pukhta_code(side: PukhtaSide) -> &'static str {
    match side {
        PukhtaSide::Buyer => mandi_domain::core::codes::PUKHTA_BUYER_CODE,
        PukhtaSide::Seller => mandi_domain::core::codes::PUKHTA_SELLER_CODE,
    }
}

/// Makes the person a regular account and moves his balance.
///
/// # Errors
/// Every error of the account form for a new account, at `account_input.<field>` (the account's
/// own field names are kept); `ERR_PERSON_REQUIRED` and `ERR_NOT_FOUND` on `person_key`;
/// `ERR_FORMAT_INVALID` on `side` when the account type is not that of the side, on
/// `account_input.id` when it is not null, and on `account_input.opening_dr`, `opening_cr` or
/// `opening_is_advance` when filled (the balance moves in by the entry, not by an opening); `ERR_DATE_CLOSED` if today is closed. Nothing is
/// written when any of them is returned.
pub fn make_regular(
    ctx: &Context,
    actor: &Actor,
    input: &MakeRegularInput,
) -> Result<MadeRegular, ServiceError> {
    let window = day::window(ctx)?;
    let today = window.1;
    let wanted_type = match input.side {
        PukhtaSide::Buyer => AccountType::Buyer,
        PukhtaSide::Seller => AccountType::Seller,
    };
    if input.account_input.account_type != wanted_type {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "side",
        ));
    }
    if input.account_input.id.is_some() || input.account_input.version.is_some() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "account_input.id",
        ));
    }
    // The new account starts with no opening of its own: the balance moves in by the entry below.
    // An opening on the form would count the same money twice.
    let form = &input.account_input;
    // A form that sends "0" for an empty opening is the same as an empty one.
    let typed = |text: &str| !text.trim().trim_matches(['0', '۰', ',']).is_empty();
    for (field, filled) in [
        ("account_input.opening_dr", typed(&form.opening_dr)),
        ("account_input.opening_cr", typed(&form.opening_cr)),
        ("account_input.opening_is_advance", form.opening_is_advance),
    ] {
        if filled {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrFormatInvalid,
                field,
            ));
        }
    }
    let key = input.person_key.trim();
    if key.is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrPersonRequired,
            "person_key",
        ));
    }
    let (made, learned, new_id) = ctx.db.write(|tx| {
        let pukhta_id = account_repo::id_by_code(tx, pukhta_code(input.side))?
            .ok_or_else(|| ServiceError::Data("the پختہ کھاتہ account is missing".to_owned()))?;
        if !mandi_db::vouchers::person_has_lines(tx, pukhta_id, key)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNotFound,
                "person_key",
            ));
        }
        // A person in the dead amount is not made regular (PRD FR-13.8, FR-10.10): his balance
        // stays where it is until it is recovered or he is made active again.
        if crate::dead::is_dead(tx, pukhta_id, Some(key))? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrPersonInDeadAmount,
                "person_key",
            ));
        }
        let (new_id, learned) = accounts::create_in(ctx, actor, tx, &input.account_input, window)?;
        let record = account_repo::get(tx, new_id)?
            .map(|row| RecordRef {
                id: row.id,
                code: row.code,
                name_ur: row.name_ur,
                name_en: row.name_en,
                tag: row.tag,
            })
            .ok_or_else(|| ServiceError::Data("the new account is missing".to_owned()))?;
        // His net on the پختہ کھاتہ account: debit less credit.
        let net = ledger::signed(ledger::balance_in(tx, pukhta_id, Some(key), None)?)?;
        let magnitude = if net.is_negative() {
            net.checked_neg().map_err(money_error)?
        } else {
            net
        };
        let mut entry_id = None;
        let moved = super::common::balance_of_net(net)?;
        if moved.side != BalanceSide::Zero {
            let mut on_pukhta = LedgerLine::new(
                AccountId::new(pukhta_id),
                if net.is_negative() {
                    Side::Dr
                } else {
                    Side::Cr
                },
                magnitude,
            );
            on_pukhta.person_key = Some(key.to_owned());
            let on_new = LedgerLine::new(
                AccountId::new(new_id),
                if net.is_negative() {
                    Side::Cr
                } else {
                    Side::Dr
                },
                magnitude,
            );
            let entry = build_entry(vec![on_new, on_pukhta]).map_err(ledger::ledger_error)?;
            let posted = ledger::post(
                ctx,
                tx,
                actor.user_id,
                &PostingEvent {
                    date: today,
                    posting_type: PostingType::General,
                    source_type: SourceType::Account,
                    source_id: new_id,
                    // The person's key names who the balance came from (it holds his name and
                    // phone), so the ledger of the new account explains its first line.
                    narration: Some(key.to_owned()),
                    reverses: None,
                    entry,
                },
            )?;
            entry_id = Some(posted.entry_id);
        }
        // The person's bills are settled by the move; the new account is matched from scratch.
        let now = ctx.now_text();
        let role = match input.side {
            PukhtaSide::Buyer => Role::Buyer,
            PukhtaSide::Seller => Role::Seller,
        };
        rematch_party(tx, actor.user_id, &now, pukhta_id, Some(key), role, true)?;
        rematch_party(tx, actor.user_id, &now, new_id, None, role, false)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "make_regular", "account")
                .entity_id(new_id)
                .new_value(json!({
                    "person_key": key,
                    "side": input.side,
                    "account_id": new_id,
                    "moved_side": moved.side,
                    "moved_paisa": moved.amount_paisa.as_paisa(),
                    "entry_id": entry_id,
                }))
                .into_entry(),
        )?;
        Ok((
            MadeRegular {
                account: record,
                moved,
                entry_id,
            },
            learned,
            new_id,
        ))
    })?;
    for learned in learned {
        learned.after_commit(ctx);
    }
    search::refresh_account(ctx, new_id);
    Ok(made)
}
