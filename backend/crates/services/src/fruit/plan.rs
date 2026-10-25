//! Checks and works out one fruit entry (PRD FR-16.10, FR-16.11, C1, C27, plan 17d). The preview,
//! the save and the edit call the same [`plan`] (Code Standards B5). Nothing is written here.

use mandi_db::accounts as account_repo;
use mandi_db::bikri::edit as edit_repo;
use mandi_db::fruit::{self as frepo, FruitRec};
use mandi_db::gardens as garden_repo;
use mandi_db::partners::{self as prepo, PartnerRecord};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{OWN_GOODS_CODE, RecordRef, SELLER_CONTROL};
use mandi_domain::core::bikri::{RateHint, SavedPukhta};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::fruit::{
    FruitCredit, FruitInput, FruitLotBalance, FruitNumbers, FruitPart, FruitPreview, FruitSource,
};
use mandi_domain::core::ids::{AccountId, PartnerId};
use mandi_domain::core::money::{Grams, MAX_RUPEES, Paisa, Qty};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::fruit::{
    FruitPartAmount, FruitParts, fruit_entry, fruit_value, guest_parts, partner_part,
};
use mandi_domain::ledger::BalancedEntry;

use crate::assist::saved_rate_hint_except;
use crate::error::{ServiceError, money_error};
use crate::partners::money::total_of;

/// The firm's shares total 100% (PRD FR-17.1).
const FULL_BPS: i64 = 10_000;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

fn optional_text(errors: &mut FieldErrors, field: &str, text: Option<&str>) -> Option<String> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// An entry that passed every check, with the journal entry it will post.
pub(super) struct Ready {
    pub date: BusinessDate,
    pub partner_id: Option<i64>,
    pub source: FruitSource,
    pub lot_id: Option<i64>,
    pub credit_account_id: i64,
    pub person_key: Option<String>,
    pub item_id: i64,
    pub packing_id: i64,
    pub qty: i64,
    pub weight_grams: Option<i64>,
    pub rate: Paisa,
    pub value: Paisa,
    pub note: Option<String>,
    pub entry: BalancedEntry,
    pub numbers: FruitNumbers,
}

/// What [`plan`] found.
pub(super) struct Planned {
    pub errors: Vec<FieldError>,
    pub ready: Option<Ready>,
    pub rate_hint: Option<RateHint>,
}

impl Planned {
    pub fn preview(&self) -> FruitPreview {
        FruitPreview {
            numbers: self.ready.as_ref().map(|r| r.numbers.clone()),
            errors: self.errors.clone(),
            warnings: Vec::new(),
            rate_hint: self.rate_hint.clone(),
        }
    }
}

/// Where the credit goes: the account, the person of a پختہ کھاتہ lot, and the references the
/// preview shows.
struct Source {
    account: RecordRef,
    person_key: Option<String>,
    pukhta: Option<SavedPukhta>,
    item: RecordRef,
    packing: RecordRef,
    by_weight: bool,
    /// نگ the lot can give: its balance, with the quantity of the entry being edited given back.
    lot: Option<(i64, i64)>,
}

fn pukhta_of(tuple: &Option<(String, String, String, String)>) -> Option<SavedPukhta> {
    tuple
        .as_ref()
        .map(|(name_ur, name_en, phone, key)| SavedPukhta {
            name_ur: name_ur.clone(),
            name_en: name_en.clone(),
            phone: phone.clone(),
            person_key: key.clone(),
        })
}

/// The lot of a fruit entry, if it is one that fruit can be taken from.
fn lot_source(
    conn: &Connection,
    errors: &mut FieldErrors,
    lot_id: Option<i64>,
    date: Option<BusinessDate>,
    editing: Option<&FruitRec>,
) -> Result<Option<Source>, ServiceError> {
    let Some(id) = lot_id else {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "lot_id"));
        return Ok(None);
    };
    let (rec, state) = (frepo::lot_one(conn, id)?, edit_repo::lot_state(conn, id)?);
    let (Some(rec), Some(state)) = (rec, state) else {
        errors.push(FieldError::new(ErrorCode::ErrLotNotFound, "lot_id"));
        return Ok(None);
    };
    // A cancelled بکری and trip goods are not offered (PRD FR-16.11).
    if state.bikri_status != "saved" || state.purchase_id.is_some() {
        errors.push(FieldError::new(ErrorCode::ErrLotNotFound, "lot_id"));
        return Ok(None);
    }
    if garden_repo::account_is_closed(conn, state.seller_account_id)? {
        errors.push(FieldError::new(ErrorCode::ErrGardenClosed, "lot_id"));
        return Ok(None);
    }
    // The posting refuses an inactive account; the preview says so at the field too (B5).
    if account_repo::facts(conn, state.seller_account_id)?.is_some_and(|facts| !facts.is_active) {
        errors.push(FieldError::new(ErrorCode::ErrAccountInactive, "lot_id"));
        return Ok(None);
    }
    if date.is_some_and(|d| d.to_iso() < state.received_date) {
        errors.push(
            FieldError::new(ErrorCode::ErrDateBeforeReceived, "date").with(
                "received_date",
                ErrorValue::Text(state.received_date.clone()),
            ),
        );
    }
    let give_back = editing
        .filter(|old| old.status == "saved" && old.lot.map(|l| l.0) == Some(id))
        .map_or(0, |old| old.qty);
    let available = rec
        .balance
        .checked_add(give_back)
        .ok_or_else(|| data("the quantity of a lot does not fit"))?;
    let source = Source {
        account: rec.seller.clone(),
        person_key: rec.pukhta.as_ref().map(|p| p.3.clone()),
        pukhta: pukhta_of(&rec.pukhta),
        item: rec.item.clone(),
        packing: rec.packing.clone(),
        by_weight: rec.sold_by_weight,
        lot: Some((id, available)),
    };
    Ok(Some(source))
}

/// The other agent's shop: an ordinary seller account, the item and the packing typed.
fn agent_source(
    conn: &Connection,
    errors: &mut FieldErrors,
    input: &FruitInput,
) -> Result<Option<Source>, ServiceError> {
    let mut account = None;
    match input.agent_account_id {
        None => errors.push(FieldError::new(ErrorCode::ErrRequired, "agent_account_id")),
        Some(id) => match account_repo::facts(conn, id)? {
            None => errors.push(FieldError::new(ErrorCode::ErrNotFound, "agent_account_id")),
            Some(facts) => {
                // An ordinary seller: not the one-time seller, not اپنا مال, not a khata or garden.
                if facts.control_code != SELLER_CONTROL
                    || facts.code == OWN_GOODS_CODE
                    || facts.is_pukhta_khata
                {
                    errors.push(FieldError::new(
                        ErrorCode::ErrFormatInvalid,
                        "agent_account_id",
                    ));
                } else if !facts.is_active {
                    errors.push(FieldError::new(
                        ErrorCode::ErrAccountInactive,
                        "agent_account_id",
                    ));
                } else {
                    account = frepo::account_ref(conn, id)?.map(|(r, _, _)| r);
                }
            }
        },
    }
    let mut item = None;
    match input.item_id {
        None => errors.push(FieldError::new(ErrorCode::ErrRequired, "item_id")),
        Some(id) => match frepo::item_ref(conn, id)? {
            Some((record, by_weight, true)) => item = Some((record, by_weight)),
            _ => errors.push(FieldError::new(ErrorCode::ErrNotFound, "item_id")),
        },
    }
    let mut packing = None;
    match input.packing_id {
        None => errors.push(FieldError::new(ErrorCode::ErrRequired, "packing_id")),
        Some(id) => match frepo::packing_ref(conn, id)? {
            Some((record, true)) => packing = Some(record),
            _ => errors.push(FieldError::new(ErrorCode::ErrNotFound, "packing_id")),
        },
    }
    Ok(match (account, item, packing) {
        (Some(account), Some((item, by_weight)), Some(packing)) => Some(Source {
            account,
            person_key: None,
            pukhta: None,
            item,
            packing,
            by_weight,
            lot: None,
        }),
        _ => None,
    })
}

/// The partners who share fruit for the guests on a day: those with a share above 0 on that day,
/// in the order they were created (PRD C27). The shares must total 100%.
fn guest_partners<'a>(
    conn: &Connection,
    errors: &mut FieldErrors,
    all: &'a [PartnerRecord],
    date: BusinessDate,
) -> Result<Vec<&'a PartnerRecord>, ServiceError> {
    let shares = prepo::shares_on(conn, &date.to_iso())?;
    let total: i64 = shares.iter().map(|&(_, bps)| bps).sum();
    if total != FULL_BPS {
        errors.push(
            FieldError::new(ErrorCode::ErrSharesNot100, "date")
                .with("total_bps", ErrorValue::Number(total)),
        );
        return Ok(Vec::new());
    }
    let mut chosen: Vec<&PartnerRecord> = Vec::new();
    for (id, bps) in shares {
        if bps > 0
            && let Some(partner) = all.iter().find(|p| p.id == id)
        {
            chosen.push(partner);
        }
    }
    chosen.sort_by_key(|p| p.id);
    Ok(chosen)
}

fn account_of(partner: &PartnerRecord) -> Result<AccountId, ServiceError> {
    if partner.venture_account_id == 0 {
        return Err(data("a partner has no gardens, trips and fruit account"));
    }
    Ok(AccountId::new(partner.venture_account_id))
}

/// Checks and works out an entry. `editing` is the saved entry being edited, if any: its quantity
/// goes back to its lot, its part goes back to the partner, and its split is kept when for whom, the
/// day and the value did not change.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix; the
/// user's mistakes come back in [`Planned::errors`].
pub(super) fn plan(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &FruitInput,
    editing: Option<&FruitRec>,
) -> Result<Planned, ServiceError> {
    let (closed_up_to, today) = window;
    let mut errors = FieldErrors::default();
    let date = errors.take(validate::posting_date(
        "date",
        &input.date,
        closed_up_to,
        today,
    ));
    let note = optional_text(&mut errors, "note", input.note.as_deref());
    let all = prepo::list(conn)?;

    // For whom.
    let mut chosen: Option<&PartnerRecord> = None;
    if input.for_guests {
        if input.partner_id.is_some() {
            errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, "partner_id"));
        }
    } else {
        match input.partner_id {
            None => errors.push(FieldError::new(ErrorCode::ErrPartnerRequired, "partner_id")),
            // An unknown partner and an inactive one are both ERR_NOT_FOUND: the words of
            // ERR_PARTNER_NOT_LISTED are written for the share list.
            Some(id) => match all.iter().find(|p| p.id == id && p.is_active) {
                None => errors.push(FieldError::new(ErrorCode::ErrNotFound, "partner_id")),
                Some(p) => chosen = Some(p),
            },
        }
    }

    // Where it comes from.
    let source = match input.source {
        FruitSource::Shop => lot_source(conn, &mut errors, input.lot_id, date, editing)?,
        FruitSource::OtherShop => agent_source(conn, &mut errors, input)?,
    };

    // The quantity, the weight and the rate.
    let qty: Option<Qty> = errors.take(validate::qty("qty", &input.qty));
    let rate: Option<Paisa> = errors.take(validate::rate("rate", &input.rate));
    let weight_text = input
        .weight
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    let mut weight: Option<Grams> = None;
    if let Some(source) = source.as_ref() {
        match (source.by_weight, weight_text) {
            (true, None) => errors.push(FieldError::new(ErrorCode::ErrWeightRequired, "weight")),
            (true, Some(text)) => weight = errors.take(validate::weight("weight", text)),
            (false, Some(_)) => errors.push(FieldError::new(ErrorCode::ErrFormatInvalid, "weight")),
            (false, None) => {}
        }
    }
    if let (
        Some(qty),
        Some(Source {
            lot: Some((_, available)),
            ..
        }),
    ) = (qty, source.as_ref())
        && qty.get() > *available
    {
        errors.push(
            FieldError::new(ErrorCode::ErrQtyExceedsLot, "qty")
                .with("available", ErrorValue::Number(*available)),
        );
    }

    let rate_hint = match (source.as_ref(), date) {
        (Some(s), Some(d)) => saved_rate_hint_except(conn, s.item.id, s.packing.id, d, None)?,
        _ => None,
    };

    // The value (C1), then the parts (C27) and the journal entry.
    let ready = match (source, qty, rate, date) {
        (Some(source), Some(qty), Some(rate), Some(date)) if errors.is_empty() => {
            let value = fruit_value(qty, weight, rate).map_err(money_error)?;
            if value.value == Paisa::ZERO {
                errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, "weight"));
                None
            } else if value.value.rupees() > MAX_RUPEES {
                errors.push(
                    FieldError::new(ErrorCode::ErrAmountTooLarge, "rate")
                        .with("max", ErrorValue::Number(MAX_RUPEES)),
                );
                None
            } else {
                let guests = if input.for_guests {
                    guest_partners(conn, &mut errors, &all, date)?
                } else {
                    Vec::new()
                };
                if errors.is_empty() {
                    Some(finish(
                        conn,
                        &all,
                        chosen,
                        &guests,
                        input,
                        FinishArgs {
                            source,
                            qty,
                            weight,
                            rate,
                            value,
                            date,
                            note,
                            editing,
                        },
                    )?)
                } else {
                    None
                }
            }
        }
        _ => None,
    };
    let errors = match errors.finish() {
        Ok(()) => Vec::new(),
        Err(list) => list,
    };
    Ok(Planned {
        errors,
        ready,
        rate_hint,
    })
}

struct FinishArgs<'a> {
    source: Source,
    qty: Qty,
    weight: Option<Grams>,
    rate: Paisa,
    value: mandi_domain::fruit::FruitValue,
    date: BusinessDate,
    note: Option<String>,
    editing: Option<&'a FruitRec>,
}

/// The parts, the entry and the numbers, once every check has passed.
fn finish(
    conn: &Connection,
    all: &[PartnerRecord],
    chosen: Option<&PartnerRecord>,
    guests: &[&PartnerRecord],
    input: &FruitInput,
    args: FinishArgs<'_>,
) -> Result<Ready, ServiceError> {
    let FinishArgs {
        source,
        qty,
        weight,
        rate,
        value,
        date,
        note,
        editing,
    } = args;
    let old_parts: Vec<frepo::PartRec> = match editing.and_then(|o| o.current_entry_id) {
        Some(entry) => frepo::parts_of_entry(conn, entry)?,
        None => Vec::new(),
    };
    // Keep the saved split when for whom, the day and the value did not change (money agent rule 13).
    let kept = editing.filter(|old| {
        old.entry_date == date.to_iso()
            && old.value_paisa == value.value.as_paisa()
            && old.partner.as_ref().map(|p| p.id) == chosen.map(|p| p.id)
            && !old_parts.is_empty()
    });
    let pairs: Vec<(PartnerId, AccountId)> = guests
        .iter()
        .map(|p| account_of(p).map(|a| (PartnerId::new(p.id), a)))
        .collect::<Result<_, _>>()?;
    let parts: FruitParts = if kept.is_some() {
        let mut out = Vec::new();
        for part in &old_parts {
            let partner = all
                .iter()
                .find(|p| p.id == part.partner.id)
                .ok_or_else(|| data("a part of a fruit entry has no partner"))?;
            out.push(FruitPartAmount {
                partner_id: PartnerId::new(partner.id),
                account_id: account_of(partner)?,
                amount: Paisa::try_from_paisa(part.amount_paisa).map_err(money_error)?,
            });
        }
        FruitParts {
            parts: out,
            working: None,
        }
    } else if input.for_guests {
        guest_parts(value.value, &pairs).map_err(money_error)?
    } else {
        let partner = chosen.ok_or_else(|| data("a fruit entry for a partner has no partner"))?;
        partner_part(
            (PartnerId::new(partner.id), account_of(partner)?),
            value.value,
        )
    };
    let entry = fruit_entry(
        &parts.parts,
        AccountId::new(source.account.id),
        source.person_key.as_deref(),
    )
    .map_err(|e| data(&format!("the journal entry of a fruit entry: {e}")))?;
    // The credit is the sum of the parts. It must be the value that is stored with the entry.
    if entry.total() != value.value {
        return Err(data(
            "the parts of a fruit entry do not add up to its value",
        ));
    }

    // Each partner's کل رقم now and after; in an edit his old part is given back first.
    let mut shown = Vec::new();
    for part in parts.parts.iter().filter(|p| p.amount > Paisa::ZERO) {
        let partner = all
            .iter()
            .find(|p| p.id == part.partner_id.get())
            .ok_or_else(|| data("a part of a fruit entry has no partner"))?;
        let mut before = total_of(conn, partner, None)?;
        if let Some(old) = old_parts.iter().find(|o| o.partner.id == partner.id) {
            before = before
                .checked_add(Paisa::try_from_paisa(old.amount_paisa).map_err(money_error)?)
                .map_err(money_error)?;
        }
        let after = before.checked_sub(part.amount).map_err(money_error)?;
        shown.push(FruitPart {
            partner: RecordRef {
                id: partner.id,
                code: String::new(),
                name_ur: partner.name_ur.clone(),
                name_en: partner.name_en.clone(),
                tag: None,
            },
            amount_paisa: part.amount,
            total_before_paisa: before,
            total_after_paisa: after,
        });
    }
    let lot = source.lot.map(|(lot_id, available)| FruitLotBalance {
        lot_id,
        balance_before: available,
        balance_after: available.saturating_sub(qty.get()),
    });
    let numbers = FruitNumbers {
        item: source.item.clone(),
        packing: source.packing.clone(),
        qty: qty.get(),
        weight_grams: weight.map(Grams::get),
        rate_paisa: rate,
        value_paisa: value.value,
        value_working: value.working,
        parts: shown,
        parts_working: parts.working,
        credit: FruitCredit {
            account: source.account.clone(),
            pukhta: source.pukhta.clone(),
        },
        lot,
    };
    Ok(Ready {
        date,
        partner_id: chosen.map(|p| p.id),
        source: input.source,
        lot_id: source.lot.map(|(id, _)| id),
        credit_account_id: source.account.id,
        person_key: source.person_key,
        item_id: source.item.id,
        packing_id: source.packing.id,
        qty: qty.get(),
        weight_grams: weight.map(Grams::get),
        rate,
        value: value.value,
        note,
        entry,
        numbers,
    })
}
