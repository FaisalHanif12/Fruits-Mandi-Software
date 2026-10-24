//! The mechanics trips share with gardens (PRD 12.10, 12.11, FR-12.29, FR-12.30, FR-12.33, plan
//! 16b): load a venture with its partners, read the money source of each partner for one entry, plan
//! a payment (C23), write the parts and post the shop parts, and open or close a trip by itself.
//!
//! The money is in `mandi_domain::ventures` (every split through `split_by_bps` or `split_by_qty`).
//! Nothing here rounds. A user action is one service function in one transaction: the callers
//! (`trips`, `purchases`) own the transaction and call these helpers inside it.

use std::collections::{BTreeSet, HashSet};

use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as repo, PartnerShare, VentureRec};
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::ids::{AccountId, PartnerId};
use mandi_domain::core::money::{BPS_FULL, Bps, MoneyError, Paisa};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::ventures::{
    EntryPart, EntryPartInput, EntryParts, MoneySource, ShareAmount, VenturePartnerInput,
    VenturePartnerRow,
};
use mandi_domain::ledger::{BalancedEntry, PostingType, SourceType};
use mandi_domain::ventures::{
    CreditPart, PaymentPart, VenturePartner, payment_entry, payment_parts, pocket_total, shop_total,
};

use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger::{self, PostingEvent};
use serde_json::json;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// `ERR_NOT_FOUND` on a field.
pub(crate) fn not_found(field: &str) -> ServiceError {
    ServiceError::reject_field(ErrorCode::ErrNotFound, field)
}

/// A reason is required: `ERR_REASON_REQUIRED` on `reason`, else the trimmed text.
///
/// # Errors
/// The text codes of `free_text`; `ERR_REASON_REQUIRED`.
pub(crate) fn required_reason(text: &str) -> Result<String, ServiceError> {
    let reason = validate::free_text("reason", text)?;
    if reason.is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrReasonRequired,
            "reason",
        ));
    }
    Ok(reason)
}

/// The day of a posting that is undone or written must be an open day (PRD FR-6.6).
///
/// # Errors
/// `ERR_DATE_CLOSED` (value `closed_up_to`) on `field`.
pub(crate) fn require_open_day(
    ctx: &Context,
    date: BusinessDate,
    field: &str,
) -> Result<(), ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    match closed_up_to {
        Some(closed) if date <= closed => Err(ServiceError::Rejected(vec![Rejection {
            code: ErrorCode::ErrDateClosed,
            field: Some(field.to_owned()),
            values: std::collections::BTreeMap::from([(
                "closed_up_to".to_owned(),
                ErrorValue::Text(closed.to_iso()),
            )]),
        }])),
        _ => Ok(()),
    }
}

/// A venture with its partners, as one action reads it.
#[derive(Debug, Clone)]
pub(crate) struct Venture {
    /// The row.
    pub rec: VentureRec,
    /// Its partners, in the order the partners were created.
    pub partners: Vec<PartnerShare>,
}

impl Venture {
    /// True while it takes new entries.
    pub(crate) fn is_open(&self) -> bool {
        self.rec.status == "open"
    }

    /// The day it was made.
    pub(crate) fn date(&self) -> Result<BusinessDate, ServiceError> {
        BusinessDate::parse_iso(&self.rec.entry_date)
            .ok_or_else(|| data("the date of a venture is not a date"))
    }
}

/// Reads a venture and its partners.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn load(conn: &Connection, id: i64) -> Result<Option<Venture>, ServiceError> {
    let Some(rec) = repo::get(conn, id)? else {
        return Ok(None);
    };
    let partners = repo::partners_of(conn, id)?;
    Ok(Some(Venture { rec, partners }))
}

/// A partner as the lists show him: both names, no code.
pub(crate) fn record_of(partner_id: i64, name_ur: &str, name_en: &str) -> RecordRef {
    RecordRef {
        id: partner_id,
        code: String::new(),
        name_ur: name_ur.to_owned(),
        name_en: name_en.to_owned(),
    }
}

/// The share as the domain type, from the stored bps.
fn bps_of(share_bps: i64) -> Result<Bps, ServiceError> {
    i32::try_from(share_bps)
        .map_err(|_| data("a stored share is out of range"))
        .and_then(|v| Bps::new(v).map_err(money_error))
}

/// The default source stored with a partner of the venture.
fn stored_source(text: &str) -> Result<MoneySource, ServiceError> {
    MoneySource::parse(text).ok_or_else(|| data("a stored money source is not shop or pocket"))
}

/// The partners of a venture as the rows of the screens show them.
///
/// # Errors
/// `ServiceError::Data` if a stored share or source is not valid.
pub(crate) fn partner_rows(
    partners: &[PartnerShare],
) -> Result<Vec<VenturePartnerRow>, ServiceError> {
    partners
        .iter()
        .map(|p| {
            Ok(VenturePartnerRow {
                partner: record_of(p.partner_id, &p.name_ur, &p.name_en),
                share_bps: i32::try_from(p.share_bps)
                    .map_err(|_| data("a stored share is out of range"))?,
                source: stored_source(&p.source)?,
            })
        })
        .collect()
}

/// The sources chosen for one entry (PRD FR-12.29, A54): one row for every partner of the venture,
/// or none to take the venture's default sources. Returns the partners as the money functions need
/// them, or `None` after pushing the errors: `ERR_PARTNER_NOT_LISTED` on `parts` when a partner of
/// the venture is missing, repeated or not in the venture.
///
/// # Errors
/// `ServiceError::Data` if a stored share, source or account is not valid.
pub(crate) fn resolve_sources(
    venture: &Venture,
    chosen: &[EntryPartInput],
    errors: &mut FieldErrors,
) -> Result<Option<Vec<VenturePartner>>, ServiceError> {
    resolve_sources_of(&venture.partners, chosen, errors)
}

/// As [`resolve_sources`], for the partners of a venture that is not saved yet (a new garden with
/// its first installment).
///
/// # Errors
/// `ServiceError::Data` if a stored share, source or account is not valid.
pub(crate) fn resolve_sources_of(
    partners: &[PartnerShare],
    chosen: &[EntryPartInput],
    errors: &mut FieldErrors,
) -> Result<Option<Vec<VenturePartner>>, ServiceError> {
    let mut by_id: Vec<(i64, MoneySource)> = Vec::with_capacity(partners.len());
    if chosen.is_empty() {
        for p in partners {
            by_id.push((p.partner_id, stored_source(&p.source)?));
        }
    } else {
        let mut seen = BTreeSet::new();
        let mut ok = true;
        for (index, row) in chosen.iter().enumerate() {
            let in_venture = partners.iter().any(|p| p.partner_id == row.partner_id);
            if !in_venture || !seen.insert(row.partner_id) {
                errors.push(FieldError::new(
                    ErrorCode::ErrPartnerNotListed,
                    &format!("parts[{index}].partner_id"),
                ));
                ok = false;
            }
            by_id.push((row.partner_id, row.source));
        }
        if seen.len() != partners.len() && ok {
            errors.push(FieldError::new(ErrorCode::ErrPartnerNotListed, "parts"));
            ok = false;
        }
        if !ok {
            return Ok(None);
        }
    }
    let mut out = Vec::with_capacity(partners.len());
    for p in partners {
        let source = by_id
            .iter()
            .find(|(id, _)| *id == p.partner_id)
            .map(|(_, s)| *s)
            .ok_or_else(|| data("a partner of a venture lost his source"))?;
        if p.venture_account_id == 0 {
            return Err(data("a partner has no gardens, trips and fruit account"));
        }
        out.push(VenturePartner {
            partner_id: PartnerId::new(p.partner_id),
            account_id: AccountId::new(p.venture_account_id),
            share: bps_of(p.share_bps)?,
            source,
        });
    }
    Ok(Some(out))
}

/// A partner of a new trip or garden after the checks.
pub(crate) struct Parsed {
    /// The partner.
    pub(crate) partner_id: i64,
    /// His share.
    pub(crate) share: Bps,
    /// His default source.
    pub(crate) source: MoneySource,
}

/// Reads the partners of a trip or a garden as typed (PRD FR-12.29): each share with the percent reader, rows with
/// a share of 0 skipped, every other partner active and listed once, and the shares totalling exactly
/// 100%. The mistakes go to `errors`, each at its field.
pub(crate) fn read_partners(
    conn: &Connection,
    errors: &mut Vec<FieldError>,
    rows: &[VenturePartnerInput],
) -> Result<Vec<Parsed>, ServiceError> {
    let active: HashSet<i64> = repo::active_partner_ids(conn)?.into_iter().collect();
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(rows.len());
    let mut total: i64 = 0;
    for (index, row) in rows.iter().enumerate() {
        let share = match validate::percent(&format!("partners[{index}].share"), &row.share) {
            Ok(share) => share,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        if share.get() == 0 {
            continue;
        }
        if !active.contains(&row.partner_id) || !seen.insert(row.partner_id) {
            errors.push(FieldError::new(
                ErrorCode::ErrPartnerNotListed,
                &format!("partners[{index}].partner_id"),
            ));
            continue;
        }
        total = total.saturating_add(i64::from(share.get()));
        out.push(Parsed {
            partner_id: row.partner_id,
            share,
            source: row.source,
        });
    }
    if total != i64::from(BPS_FULL) {
        errors.push(
            FieldError::new(ErrorCode::ErrSharesNot100, "partners")
                .with("total_bps", ErrorValue::Number(total)),
        );
    }
    out.sort_by_key(|p| p.partner_id);
    Ok(out)
}

pub(crate) fn write_partners(
    tx: &Connection,
    trip_id: i64,
    partners: &[Parsed],
    user_id: i64,
    now: &str,
) -> Result<(), ServiceError> {
    for p in partners {
        repo::insert_partner(
            tx,
            trip_id,
            p.partner_id,
            i64::from(p.share.get()),
            p.source.as_str(),
            user_id,
            now,
        )?;
    }
    Ok(())
}

pub(crate) fn partners_json(partners: &[Parsed]) -> serde_json::Value {
    json!(
        partners
            .iter()
            .map(|p| json!({
                "partner_id": p.partner_id,
                "share_bps": p.share.get(),
                "source": p.source.as_str(),
            }))
            .collect::<Vec<_>>()
    )
}

/// The partners as the money functions need them, with every source set to `shop`: for money that
/// comes in, where the source does not matter (PRD FR-12.32, C25).
///
/// # Errors
/// `ServiceError::Data` if a stored share or account is not valid.
pub(crate) fn credit_partners(venture: &Venture) -> Result<Vec<VenturePartner>, ServiceError> {
    venture
        .partners
        .iter()
        .map(|p| {
            if p.venture_account_id == 0 {
                return Err(data("a partner has no gardens, trips and fruit account"));
            }
            Ok(VenturePartner {
                partner_id: PartnerId::new(p.partner_id),
                account_id: AccountId::new(p.venture_account_id),
                share: bps_of(p.share_bps)?,
                source: MoneySource::Shop,
            })
        })
        .collect()
}

/// A payment worked out: the parts, the entry for the shop parts, and what it does to cash.
#[derive(Debug, Clone)]
pub(crate) struct PaymentPlan {
    /// One part for each partner.
    pub parts: Vec<PaymentPart>,
    /// The journal entry of the shop parts. `None` when every part is from a pocket.
    pub entry: Option<BalancedEntry>,
    /// What the entry does to cash in hand (PRD FR-6.7).
    pub cash: CashCheck,
}

/// Works out a payment of the venture (C23): the parts, the entry and the cash check. Reads only.
///
/// # Errors
/// `ServiceError::Data` if the shares of the venture do not total 100%; `ERR_OVERFLOW`.
pub(crate) fn plan_payment(
    conn: &Connection,
    amount: Paisa,
    partners: &[VenturePartner],
) -> Result<PaymentPlan, ServiceError> {
    let parts = payment_parts(amount, partners).map_err(|error| match error {
        MoneyError::SharesNot100 => data("the shares of a venture do not total 100%"),
        other => money_error(other),
    })?;
    let cash_id = account_repo::id_by_code(conn, cash::CASH_ACCOUNT_CODE)?
        .ok_or_else(|| data("the cash account is missing"))?;
    let entry = payment_entry(&parts, AccountId::new(cash_id)).map_err(ledger::ledger_error)?;
    let shop = shop_total(&parts).map_err(money_error)?;
    let change = Paisa::ZERO.checked_sub(shop).map_err(money_error)?;
    let cash = cash::cash_after_entry(conn, change)?;
    Ok(PaymentPlan { parts, entry, cash })
}

/// The parts of a payment as the screens show them.
///
/// # Errors
/// `ServiceError::Data` if a part has no partner row; `ERR_OVERFLOW`.
pub(crate) fn parts_view(
    venture: &Venture,
    parts: &[PaymentPart],
) -> Result<EntryParts, ServiceError> {
    parts_view_of(&venture.partners, parts)
}

/// As [`parts_view`], for the partners of a venture that is not saved yet.
///
/// # Errors
/// `ServiceError::Data` if a part has no partner row; `ERR_OVERFLOW`.
pub(crate) fn parts_view_of(
    partners: &[PartnerShare],
    parts: &[PaymentPart],
) -> Result<EntryParts, ServiceError> {
    let mut out = Vec::with_capacity(parts.len());
    for part in parts {
        let partner = partners
            .iter()
            .find(|p| p.partner_id == part.partner_id.get())
            .ok_or_else(|| data("a part names a partner who is not in the venture"))?;
        out.push(EntryPart {
            partner: record_of(partner.partner_id, &partner.name_ur, &partner.name_en),
            share_bps: part.share.get(),
            source: part.source,
            amount_paisa: part.amount,
            working: part.working.clone(),
        });
    }
    Ok(EntryParts {
        parts: out,
        shop_total_paisa: shop_total(parts).map_err(money_error)?,
        pocket_total_paisa: pocket_total(parts).map_err(money_error)?,
    })
}

/// The credits of money that comes to the venture's partners as the screens show them.
///
/// # Errors
/// `ServiceError::Data` if a part has no partner row.
pub(crate) fn credits_view(
    venture: &Venture,
    credits: &[CreditPart],
) -> Result<Vec<ShareAmount>, ServiceError> {
    credits
        .iter()
        .map(|credit| {
            let partner = venture
                .partners
                .iter()
                .find(|p| p.partner_id == credit.partner_id.get())
                .ok_or_else(|| data("a credit names a partner who is not in the venture"))?;
            Ok(ShareAmount {
                partner: record_of(partner.partner_id, &partner.name_ur, &partner.name_en),
                share_bps: credit.share.get(),
                amount_paisa: credit.amount,
                working: credit.working.clone(),
            })
        })
        .collect()
}

/// What a venture entry needs written next to its row.
pub(crate) struct Settlement<'a> {
    /// The venture entry (`venture_entries.id`).
    pub venture_entry_id: i64,
    /// The day of the posting.
    pub date: BusinessDate,
    /// The journal entry to post, or `None` when nothing posts.
    pub entry: Option<&'a BalancedEntry>,
    /// What the entry is for.
    pub posting_type: PostingType,
    /// What kind of record it belongs to.
    pub source_type: SourceType,
    /// The id of that record.
    pub source_id: i64,
}

/// Posts the journal entry of a venture entry, if there is one, and names it on the venture entry.
/// Returns the journal entry id.
///
/// # Errors
/// The errors of `ledger::post` (the date, an inactive account, the database balance guard).
pub(crate) fn post_settlement(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    now: &str,
    settlement: &Settlement<'_>,
) -> Result<Option<i64>, ServiceError> {
    let Some(entry) = settlement.entry else {
        return Ok(None);
    };
    let posted = ledger::post(
        ctx,
        tx,
        user_id,
        &PostingEvent {
            date: settlement.date,
            posting_type: settlement.posting_type,
            source_type: settlement.source_type,
            source_id: settlement.source_id,
            narration: None,
            reverses: None,
            entry: entry.clone(),
        },
    )?;
    repo::set_entry_journal(
        tx,
        settlement.venture_entry_id,
        posted.entry_id,
        user_id,
        now,
    )?;
    Ok(Some(posted.entry_id))
}

/// Writes the parts of a payment next to its venture entry.
///
/// # Errors
/// `ServiceError::Db` (the CHECKs of `venture_entry_parts`).
pub(crate) fn write_parts(
    tx: &Connection,
    user_id: i64,
    now: &str,
    venture_entry_id: i64,
    parts: &[PaymentPart],
) -> Result<(), ServiceError> {
    for part in parts {
        repo::insert_part(
            tx,
            venture_entry_id,
            part.partner_id.get(),
            i64::from(part.share.get()),
            part.source.as_str(),
            part.amount.as_paisa(),
            user_id,
            now,
        )?;
    }
    Ok(())
}

/// Writes the credits of money that comes in (C25) next to its venture entry. The source of such a
/// part is `shop`: the money comes into روکڑ (PRD A57).
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn write_credits(
    tx: &Connection,
    user_id: i64,
    now: &str,
    venture_entry_id: i64,
    credits: &[CreditPart],
) -> Result<(), ServiceError> {
    for credit in credits {
        repo::insert_part(
            tx,
            venture_entry_id,
            credit.partner_id.get(),
            i64::from(credit.share.get()),
            MoneySource::Shop.as_str(),
            credit.amount.as_paisa(),
            user_id,
            now,
        )?;
    }
    Ok(())
}

/// What `refresh_trip_status` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TripChange {
    /// The status stayed as it was.
    Same,
    /// The trip closed by itself (PRD FR-12.33).
    Closed,
    /// The trip opened again because a sale or a write-off of it was reversed.
    Reopened,
}

/// Opens or closes a trip by itself (PRD FR-12.33): it is closed when it has at least one saved
/// purchase and every saved purchase has nothing left, and open otherwise. Call it inside the
/// transaction of every action that changes the stock of a trip's purchases (a purchase, a sale in
/// our shop or another, a write-off, and the reversal of each). `date_iso` is the day of the action,
/// kept as the day the trip closed.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn refresh_trip_status(
    tx: &Connection,
    trip_id: i64,
    date_iso: &str,
    user_id: i64,
    now: &str,
) -> Result<TripChange, ServiceError> {
    let Some(rec) = repo::get(tx, trip_id)? else {
        return Ok(TripChange::Same);
    };
    if rec.kind != "trip" {
        return Ok(TripChange::Same);
    }
    let figures = repo::purchase_figures(tx, &[trip_id])?
        .remove(&trip_id)
        .unwrap_or_default();
    let left = figures
        .qty_bought
        .checked_sub(figures.qty_sold)
        .and_then(|v| v.checked_sub(figures.qty_written_off))
        .ok_or_else(|| data("the quantity of a trip does not fit"))?;
    let should_close = figures.purchase_count > 0 && left == 0;
    match (rec.status.as_str(), should_close) {
        ("open", true) => {
            repo::set_status(tx, trip_id, "closed", Some(date_iso), user_id, now)?;
            Ok(TripChange::Closed)
        }
        ("closed", false) => {
            repo::set_status(tx, trip_id, "open", None, user_id, now)?;
            Ok(TripChange::Reopened)
        }
        _ => Ok(TripChange::Same),
    }
}
