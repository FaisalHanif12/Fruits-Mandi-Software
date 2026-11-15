//! The mirror of a linked khata or loan, applied to every new posting (plan 76 points 5 and 10, PRD
//! FR-16.4, FR-16.6, FR-17.9 v1.41; Umar 14-11-2026, U1). [`mirror_new_posting`] is called by
//! `ledger::post` for every posting that is not a reversal, so a voucher, the seller credit of a
//! lot, fruit taken and a جنرل entry on a linked khata all get the same mirror pair, built by
//! `domain::ledger::linked`. A reversal copies the stored lines, mirror lines included.
//!
//! Example: خاندانی کھاتہ linked to the shop (40 / 30 / 30) is paid out 10,000 on 10-10-26: the
//! entry holds the khata بنام and روکڑ جمع of 10,000, then الف 4,000, ب 3,000 and ج 3,000 بنام on
//! their 0605 accounts and 03070001 جمع 10,000.

use std::collections::HashMap;

use mandi_db::links as repo;
use mandi_db::partners as partner_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::ids::{AccountId, PartnerId};
use mandi_domain::core::links::{Link, LinkKind};
use mandi_domain::core::money::{Bps, Paisa};
use mandi_domain::ledger::linked::{Mirror, MirrorError, MirrorInputs, mirror_linked};
use mandi_domain::ledger::{BalancedEntry, PostingType, Side};

use mandi_db::ledger as ledger_repo;
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::ledger::PostingEvent;

/// The entry an edit replaces: its order key (the id of its first version) and the (account, person
/// key) pairs that its versions have lines on (`ledger::lines_with_order_key`).
pub(crate) type Replaced<'a> = (i64, &'a [(i64, Option<String>)]);

/// The entry of a new posting with the mirror pair of every line on a linked account added, or
/// `None` when it has none, or when it is a reversal or a month close (they never mirror).
///
/// # Errors
/// `ERR_SHARES_NOT_100` when a shop link has to split and the shares of the day do not total 100%;
/// `ServiceError::Data` when a partner has no 0605 account or the contra account is missing.
pub(crate) fn mirror_new_posting(
    tx: &Connection,
    event: &PostingEvent,
    replaced: Option<Replaced<'_>>,
) -> Result<Option<BalancedEntry>, ServiceError> {
    if event.reverses.is_some() || event.posting_type == PostingType::MonthClose {
        return Ok(None);
    }
    mirror_entry(tx, event.date, &event.entry, replaced, None)
}

/// What each partner's کل رقم would change by if this entry were posted on this day: the mirror
/// lines on his 0605 account, credit less debit. Empty when the entry touches no linked account.
/// The preview of a khata or loan shows it before the save, worked out by the same function the
/// save uses (Code Standards B5). Partners in order of their id.
///
/// # Errors
/// As [`mirror_new_posting`].
pub(crate) fn preview_parts(
    tx: &Connection,
    date: BusinessDate,
    entry: &BalancedEntry,
    replaced: Option<Replaced<'_>>,
    typed: Option<&TypedLink>,
) -> Result<Vec<(i64, Paisa)>, ServiceError> {
    let Some(mirrored) = mirror_entry(tx, date, entry, replaced, typed)? else {
        return Ok(Vec::new());
    };
    let accounts: HashMap<i64, i64> = repo::partner_accounts(tx)?
        .into_iter()
        .map(|(partner, account)| (account, partner))
        .collect();
    let mut parts: std::collections::BTreeMap<i64, Paisa> = std::collections::BTreeMap::new();
    for line in mirrored.lines() {
        let Some(partner) = accounts.get(&line.account_id.get()) else {
            continue;
        };
        let signed = match line.side {
            Side::Cr => line.amount,
            Side::Dr => line
                .amount
                .checked_neg()
                .map_err(crate::error::money_error)?,
        };
        let slot = parts.entry(*partner).or_insert(Paisa::ZERO);
        *slot = slot
            .checked_add(signed)
            .map_err(crate::error::money_error)?;
    }
    Ok(parts.into_iter().collect())
}

/// A link typed on the loan form for the person of the entry, which the save would set first (plan
/// 76 point 10): the preview works as if it were already stored (Code Standards B5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypedLink {
    /// The person's account.
    pub account_id: i64,
    /// The link as it would be stored.
    pub checked: CheckedLink,
}

fn mirror_entry(
    tx: &Connection,
    date: BusinessDate,
    entry: &BalancedEntry,
    replaced: Option<Replaced<'_>>,
    typed: Option<&TypedLink>,
) -> Result<Option<BalancedEntry>, ServiceError> {
    let mut ids: Vec<i64> = entry.lines().iter().map(|l| l.account_id.get()).collect();
    ids.sort_unstable();
    ids.dedup();
    let mut linked = repo::linked_among(tx, &ids)?;
    // The link typed for a person replaces his stored one. Only a partner link is mirrored for a
    // loan; a shop link posts as before.
    if let Some(typed) = typed {
        linked.retain(|a| a.account_id != typed.account_id);
        if typed.checked.kind == "partner" && ids.contains(&typed.account_id) {
            linked.push(repo::LinkedAccount {
                account_id: typed.account_id,
                control_code: "0103".to_owned(),
                kind: "partner".to_owned(),
                partner_id: typed.checked.partner_id,
                // The link would be set after every entry that exists now.
                since_entry_id: i64::MAX,
            });
        }
    }
    if linked.is_empty() {
        return Ok(None);
    }
    let offset = repo::offset_account(tx)?
        .map(AccountId::new)
        .ok_or_else(|| ServiceError::Data("the contra account 03070001 is missing".to_owned()))?;
    // An edit keeps what the entry it replaces was. An account that was in it, and whose link was
    // set after that entry was made, is not mirrored when the entry was not (it was made before the
    // link, and its reversal has no mirror either, so a mirror on the new version alone would move
    // the partners' totals for money that was never linked); and the edit is refused when the entry
    // was mirrored under an older link (the new version would be split by the new one). An account
    // that is new to the entry is mirrored like in any new posting.
    let had_contra = replaced
        .is_some_and(|(_, lines)| lines.iter().any(|(account, _)| *account == offset.get()));
    let mut links: HashMap<AccountId, Mirror> = HashMap::new();
    for account in &linked {
        if let Some((key, lines)) = replaced
            && lines.iter().any(|(id, _)| *id == account.account_id)
            && account.since_entry_id >= key
        {
            if had_contra {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrLinkChangedForEntry,
                    "link",
                ));
            }
            continue;
        }
        let mirror = match (account.kind.as_str(), account.partner_id) {
            ("partner", Some(partner)) => Mirror::Partner(PartnerId::new(partner)),
            ("shop", _) => Mirror::Shop,
            _ => {
                return Err(ServiceError::Data(
                    "a linked account has a link that does not name its partner".to_owned(),
                ));
            }
        };
        links.insert(AccountId::new(account.account_id), mirror);
    }
    if links.is_empty() {
        return Ok(None);
    }
    let partner_accounts: HashMap<PartnerId, AccountId> = repo::partner_accounts(tx)?
        .into_iter()
        .map(|(partner, account)| (PartnerId::new(partner), AccountId::new(account)))
        .collect();
    let needs_shares = links.values().any(|m| *m == Mirror::Shop);
    let shares: Vec<(PartnerId, Bps)> = if needs_shares {
        partner_repo::shares_on(tx, &date.to_iso())?
            .into_iter()
            .map(|(partner, bps)| {
                let bps = i32::try_from(bps)
                    .ok()
                    .and_then(|b| Bps::new(b).ok())
                    .ok_or_else(|| {
                        ServiceError::Data("a stored share is not 0 to 10,000".to_owned())
                    })?;
                Ok((PartnerId::new(partner), bps))
            })
            .collect::<Result<_, ServiceError>>()?
    } else {
        Vec::new()
    };
    let mirrored = mirror_linked(
        entry,
        &MirrorInputs {
            links: &links,
            offset,
            partner_accounts: &partner_accounts,
            shares: &shares,
        },
    )
    .map_err(|error| match error {
        MirrorError::Money(mandi_domain::core::money::MoneyError::SharesNot100) => {
            ServiceError::reject(ErrorCode::ErrSharesNot100)
        }
        MirrorError::Money(other) => crate::error::money_error(other),
        other => ServiceError::Data(other.to_string()),
    })?;
    Ok(Some(mirrored))
}

/// A link as it is stored: the kind word and the partner of a partner link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckedLink {
    /// `none`, `shop` or `partner`.
    pub kind: &'static str,
    /// The partner of a partner link; null for every other kind.
    pub partner_id: Option<i64>,
}

/// Reads a stored link back into the type of the screens.
pub(crate) fn link_from_stored(kind: &str, partner_id: Option<i64>) -> Link {
    Link {
        kind: LinkKind::parse(kind).unwrap_or(LinkKind::None),
        partner_id: partner_id.filter(|_| kind == "partner"),
    }
}

/// A shop link or no link names no partner. `partner_id` on such a link is a mistake of the caller
/// (a stale value left in a form), so it is refused and never silently dropped.
///
/// # Errors
/// `ERR_FORMAT_INVALID` on `link.partner_id`.
pub(crate) fn refuse_stray_partner(link: &Link) -> Result<(), ServiceError> {
    if link.kind != LinkKind::Partner && link.partner_id.is_some() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "link.partner_id",
        ));
    }
    Ok(())
}

/// Checks a link as typed (plan 76 points 5 and 10, PRD FR-16.6, FR-16.13). A partner link needs an
/// active partner; a shop link needs shares that total 100% today, so the first movement cannot
/// fail on them. `shares_day` is that day, `YYYY-MM-DD`.
///
/// # Errors
/// `ERR_LINK_PARTNER_REQUIRED` and `ERR_NOT_FOUND` and `ERR_PARTNER_INACTIVE` on `link.partner_id`;
/// `ERR_SHARES_NOT_100` on `link`; `ERR_FORMAT_INVALID` on `link.partner_id` for a shop link or no
/// link that names a partner.
pub(crate) fn check_link(
    tx: &Connection,
    link: &Link,
    shares_day: &str,
) -> Result<CheckedLink, ServiceError> {
    refuse_stray_partner(link)?;
    match link.kind {
        LinkKind::None => Ok(CheckedLink {
            kind: "none",
            partner_id: None,
        }),
        LinkKind::Shop => {
            let total = partner_repo::shares_on(tx, shares_day)?
                .iter()
                .try_fold(0_i64, |sum, (_, bps)| sum.checked_add(*bps));
            if total != Some(i64::from(Bps::FULL.get())) {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrSharesNot100,
                    "link",
                ));
            }
            Ok(CheckedLink {
                kind: "shop",
                partner_id: None,
            })
        }
        LinkKind::Partner => {
            let Some(id) = link.partner_id else {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrLinkPartnerRequired,
                    "link.partner_id",
                ));
            };
            let partner = partner_repo::get(tx, id)?.ok_or_else(|| {
                ServiceError::reject_field(ErrorCode::ErrNotFound, "link.partner_id")
            })?;
            if !partner.is_active {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrPartnerInactive,
                    "link.partner_id",
                ));
            }
            Ok(CheckedLink {
                kind: "partner",
                partner_id: Some(id),
            })
        }
    }
}

/// Whether setting `checked` on an account that has `stored` would change its link. A person who is
/// lent money has no `none`: a loan with no link belongs to the shop, as it always did, so for him
/// the stored `none` and the shop are the same link. One function for the save and the preview
/// (Code Standards B5).
pub(crate) fn link_would_change(
    stored: &repo::StoredLink,
    checked: &CheckedLink,
    person: bool,
) -> bool {
    let stored_partner = stored.partner_id.filter(|_| stored.kind == "partner");
    let same_kind =
        stored.kind == checked.kind || (person && stored.kind == "none" && checked.kind == "shop");
    !(same_kind && stored_partner == checked.partner_id)
}

/// Whether the account has a balance (debit or credit) that is not 0 now.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn has_balance(tx: &Connection, account_id: i64) -> Result<bool, ServiceError> {
    let totals = ledger_repo::totals_for_accounts(tx, &[account_id])?;
    Ok(totals
        .get(&account_id)
        .is_some_and(|t| t.dr_paisa != t.cr_paisa))
}

/// Writes the link of an account, in the caller's transaction, when it differs from the stored one.
/// A different link is refused with `locked` (field `link`) while the account has a balance. The
/// change is audited with the old and the new link (Code Standards S7). Returns true when it wrote.
///
/// # Errors
/// `locked`; `ServiceError::Db`.
pub(crate) fn apply_link(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    account_id: i64,
    checked: &CheckedLink,
    locked: ErrorCode,
) -> Result<bool, ServiceError> {
    let stored = repo::link_of(tx, account_id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "account_id"))?;
    let stored_partner = stored.partner_id.filter(|_| stored.kind == "partner");
    let person = locked == ErrorCode::ErrLinkFixedWhileOwed;
    if !link_would_change(&stored, checked, person) {
        return Ok(false);
    }
    if has_balance(tx, account_id)? {
        return Err(ServiceError::reject_field(locked, "link"));
    }
    repo::set_link(
        tx,
        account_id,
        checked.kind,
        checked.partner_id,
        actor.user_id,
        &ctx.now_text(),
    )?;
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), "update", "account")
            .entity_id(account_id)
            .old(json!({ "link_kind": stored.kind, "link_partner_id": stored_partner }))
            .new_value(json!({ "link_kind": checked.kind, "link_partner_id": checked.partner_id }))
            .into_entry(),
    )?;
    Ok(true)
}

/// Refuses a typed line on the contra account 03070001 or on a partner's 0605 account (plan 76
/// points 5 and 10): the app writes them itself, as the mirror of a linked line. A reversal copies the
/// stored lines and a month close only moves balances, so neither is checked.
///
/// # Errors
/// `ERR_LINKED_ACCOUNT_RESTRICTED`.
pub(crate) fn refuse_typed_lines(
    tx: &Connection,
    entry: &BalancedEntry,
) -> Result<(), ServiceError> {
    let mut restricted: Vec<i64> = repo::partner_accounts(tx)?.into_values().collect();
    restricted.extend(repo::offset_account(tx)?);
    if let Some(line) = entry
        .lines()
        .iter()
        .find(|l| restricted.contains(&l.account_id.get()))
    {
        return Err(ServiceError::Rejected(vec![crate::Rejection {
            code: ErrorCode::ErrLinkedAccountRestricted,
            field: None,
            values: std::collections::BTreeMap::from([(
                "account_id".to_owned(),
                mandi_domain::core::error::ErrorValue::Number(line.account_id.get()),
            )]),
        }]));
    }
    Ok(())
}
