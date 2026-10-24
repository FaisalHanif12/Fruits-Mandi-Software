//! An installment, a running cost or a sale in another shop of a garden (PRD FR-12.35, FR-12.36,
//! C23, plan 17c). An installment or a cost is paid in as any payment of a venture: split by the
//! garden's shares, each partner's part from the shop's cash or from his pocket (FR-12.30). Money
//! from a sale in another shop comes into روکڑ and goes to the garden's own account; nothing is split
//! before the closing (C26).
//!
//! The preview and the save call the same [`plan`] (Code Standards B5). A saved entry is never
//! edited: it is cancelled by reversal while its day is open, and not at all while the garden is
//! closed (undo the closing first).

use mandi_db::accounts as account_repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as vrepo, NewEntry};
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::gardens::{
    GardenEntryCancelInput, GardenEntryCancelled, GardenEntryInput, GardenEntryNumbers,
    GardenEntryPreview, GardenEntrySaved, GardenHistoryKind, GardenOtherSaleInput,
    GardenPaymentInput,
};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::ventures::EntryParts;
use mandi_domain::ledger::{BalancedEntry, LedgerLine, PostingType, Side, SourceType, build_entry};
use serde_json::json;

use super::data;
use super::read::{Garden, Money, balance_working, load};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::ventures::{
    self, PaymentPlan, Settlement, not_found, parts_view_of, plan_payment, require_open_day,
    required_reason, resolve_sources_of,
};
use crate::vouchers::stored_cash_change;

/// An optional free text: empty is none.
fn optional_text(errors: &mut FieldErrors, field: &str, text: Option<&str>) -> Option<String> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// What the entry is, with what it needs beyond the common fields.
enum Body {
    /// An installment or a running cost, with its payment worked out.
    Payment { payment: PaymentPlan },
    /// Money from a sale in another shop, with its journal entry.
    OtherSale { entry: BalancedEntry, qty: i64 },
}

/// An entry after the checks.
struct Ready {
    garden: Garden,
    kind: GardenHistoryKind,
    date: BusinessDate,
    amount: Paisa,
    /// Why (a payment), or the note (a sale in another shop).
    reason: Option<String>,
    shop: Option<String>,
    body: Body,
    numbers: GardenEntryNumbers,
}

/// What [`plan`] found.
struct Plan {
    kind: GardenHistoryKind,
    errors: Vec<FieldError>,
    ready: Option<Ready>,
    cash: CashCheck,
}

impl Plan {
    fn preview(&self) -> GardenEntryPreview {
        GardenEntryPreview {
            kind: self.kind,
            numbers: self.ready.as_ref().map(|r| r.numbers.clone()),
            cash: self.cash,
            errors: self.errors.clone(),
            warnings: Vec::new(),
        }
    }
}

/// The common fields of an entry: the garden, which must be open, and the day, which must be an open
/// day not before the garden.
fn garden_and_date(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    garden_id: Option<i64>,
    date_text: &str,
    errors: &mut FieldErrors,
) -> Result<(Option<Garden>, Option<BusinessDate>), ServiceError> {
    let (closed_up_to, today) = window;
    let garden = match garden_id {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, "garden_id"));
            None
        }
        Some(id) => match load(conn, id)? {
            Some(g) => Some(g),
            None => {
                errors.push(FieldError::new(ErrorCode::ErrNotFound, "garden_id"));
                None
            }
        },
    };
    if garden.as_ref().is_some_and(|g| !g.is_open()) {
        errors.push(FieldError::new(ErrorCode::ErrGardenClosed, "garden_id"));
    }
    let date = errors.take(validate::posting_date(
        "date",
        date_text,
        closed_up_to,
        today,
    ));
    if let (Some(date), Some(g)) = (date, garden.as_ref()) {
        let garden_date = g.date()?;
        if date < garden_date {
            errors.push(
                FieldError::new(ErrorCode::ErrDateBeforeGarden, "date")
                    .with("garden_date", ErrorValue::Text(garden_date.to_iso())),
            );
        }
    }
    Ok((garden, date))
}

fn positive_money(errors: &mut FieldErrors, field: &str, text: &str) -> Option<Paisa> {
    errors.take(validate::money(field, text)).filter(|a| {
        let positive = *a > Paisa::ZERO;
        if !positive {
            errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, field));
        }
        positive
    })
}

/// Checks and works out an entry. Nothing is written.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix. The
/// user's mistakes come back in [`Plan::errors`].
fn plan(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &GardenEntryInput,
) -> Result<Plan, ServiceError> {
    match input {
        GardenEntryInput::Installment(payment) => {
            plan_payment_entry(conn, window, payment, GardenHistoryKind::Installment)
        }
        GardenEntryInput::Cost(payment) => {
            plan_payment_entry(conn, window, payment, GardenHistoryKind::Cost)
        }
        GardenEntryInput::OtherSale(sale) => plan_other_sale(conn, window, sale),
    }
}

fn plan_payment_entry(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &GardenPaymentInput,
    kind: GardenHistoryKind,
) -> Result<Plan, ServiceError> {
    let mut errors = FieldErrors::default();
    let (garden, date) = garden_and_date(conn, window, input.garden_id, &input.date, &mut errors)?;
    let amount = positive_money(&mut errors, "amount", &input.amount);
    let reason = errors
        .take(validate::free_text("reason", &input.reason))
        .filter(|r| {
            let given = !r.is_empty();
            if !given {
                errors.push(FieldError::new(ErrorCode::ErrReasonRequired, "reason"));
            }
            given
        });
    let partners = match &garden {
        Some(g) => resolve_sources_of(&g.venture.partners, &input.parts, &mut errors)?,
        None => None,
    };
    let found = errors.finish().err().unwrap_or_default();
    let (Some(garden), Some(date), Some(amount), Some(reason), Some(partners)) =
        (garden, date, amount, reason, partners)
    else {
        return Ok(Plan {
            kind,
            errors: found,
            ready: None,
            cash: cash::cash_after_entry(conn, Paisa::ZERO)?,
        });
    };
    if !found.is_empty() {
        return Ok(Plan {
            kind,
            errors: found,
            ready: None,
            cash: cash::cash_after_entry(conn, Paisa::ZERO)?,
        });
    }
    let payment = plan_payment(conn, amount, &partners)?;
    let money = money_of(conn, &garden)?;
    let balance_after = money.balance.checked_sub(amount).map_err(money_error)?;
    let paid_after = money.paid.checked_add(amount).map_err(money_error)?;
    let rent_due_after = if kind == GardenHistoryKind::Installment {
        money.rent_due.checked_sub(amount).map_err(money_error)?
    } else {
        money.rent_due
    };
    let numbers = GardenEntryNumbers {
        amount,
        parts: Some(parts_of(&garden, &payment)?),
        balance_before_paisa: money.balance,
        balance_after_paisa: balance_after,
        rent_due_after_paisa: rent_due_after,
        balance_working: balance_working(money.money_in, paid_after, balance_after),
    };
    let cash = payment.cash;
    Ok(Plan {
        kind,
        errors: found,
        ready: Some(Ready {
            garden,
            kind,
            date,
            amount,
            reason: Some(reason),
            shop: None,
            body: Body::Payment { payment },
            numbers,
        }),
        cash,
    })
}

fn parts_of(garden: &Garden, payment: &PaymentPlan) -> Result<EntryParts, ServiceError> {
    parts_view_of(&garden.venture.partners, &payment.parts)
}

fn plan_other_sale(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &GardenOtherSaleInput,
) -> Result<Plan, ServiceError> {
    let kind = GardenHistoryKind::OtherSale;
    let mut errors = FieldErrors::default();
    let (garden, date) = garden_and_date(conn, window, input.garden_id, &input.date, &mut errors)?;
    let shop = optional_text(&mut errors, "shop", input.shop.as_deref());
    let note = optional_text(&mut errors, "note", input.note.as_deref());
    let qty = errors.take(validate::qty("qty", &input.qty));
    let amount = positive_money(&mut errors, "amount", &input.amount);
    let found = errors.finish().err().unwrap_or_default();
    // The money comes into روکڑ: nothing here can take cash below 0.
    let cash = cash::cash_after_entry(conn, amount.unwrap_or(Paisa::ZERO))?;
    let (Some(garden), Some(date), Some(qty), Some(amount)) = (garden, date, qty, amount) else {
        return Ok(Plan {
            kind,
            errors: found,
            ready: None,
            cash,
        });
    };
    if !found.is_empty() {
        return Ok(Plan {
            kind,
            errors: found,
            ready: None,
            cash,
        });
    }
    let cash_id = account_repo::id_by_code(conn, cash::CASH_ACCOUNT_CODE)?
        .ok_or_else(|| data("the cash account is missing"))?;
    // روکڑ بنام the money, the garden's account جمع it (PRD FR-12.36).
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(cash_id), Side::Dr, amount),
        LedgerLine::new(AccountId::new(garden.rec.account.id), Side::Cr, amount),
    ])
    .map_err(ledger::ledger_error)?;
    let money = money_of(conn, &garden)?;
    let balance_after = money.balance.checked_add(amount).map_err(money_error)?;
    let money_in_after = money.money_in.checked_add(amount).map_err(money_error)?;
    let numbers = GardenEntryNumbers {
        amount,
        parts: None,
        balance_before_paisa: money.balance,
        balance_after_paisa: balance_after,
        rent_due_after_paisa: money.rent_due,
        balance_working: balance_working(money_in_after, money.paid, balance_after),
    };
    Ok(Plan {
        kind,
        errors: found,
        ready: Some(Ready {
            garden,
            kind,
            date,
            amount,
            reason: note,
            shop,
            body: Body::OtherSale {
                entry,
                qty: qty.get(),
            },
            numbers,
        }),
        cash,
    })
}

/// The figures of a garden as money.
fn money_of(conn: &Connection, garden: &Garden) -> Result<Money, ServiceError> {
    let figures = mandi_db::gardens::figures(conn, &garden.rec)?;
    Money::of(&garden.rec, &figures)
}

/// Works out an entry for the live screen. Nothing is saved.
pub(super) fn preview(
    ctx: &Context,
    input: &GardenEntryInput,
) -> Result<GardenEntryPreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| Ok(plan(conn, window, input)?.preview()))
}

/// Saves an entry in one transaction: the entry with each partner's part, the journal entry (the shop
/// parts of a payment, or the money of a sale in another shop) and the audit row.
///
/// Example (E2E-40): an installment of 30,000 at 60 / 40, الف from the shop and ب from his pocket,
/// gives the parts 18,000 / 12,000; only 18,000 posts (الف's 0604 بنام, روکڑ جمع).
pub(super) fn save(
    ctx: &Context,
    actor: &Actor,
    input: &GardenEntryInput,
) -> Result<GardenEntrySaved, ServiceError> {
    let window = day::window(ctx)?;
    let now = ctx.now_text();
    let confirmed = match input {
        GardenEntryInput::Installment(p) | GardenEntryInput::Cost(p) => p.confirm_cash_below_zero,
        GardenEntryInput::OtherSale(_) => false,
    };
    ctx.db.write(|tx| {
        let planned = plan(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(ServiceError::from(planned.errors.clone()));
        }
        let preview = planned.preview();
        let cash_check = planned.cash;
        let ready = planned
            .ready
            .ok_or_else(|| data("an entry without errors has no numbers"))?;
        cash::require_confirmed(&cash_check, confirmed)?;
        let user = actor.user_id;
        let date_iso = ready.date.to_iso();
        let kind_text = match ready.kind {
            GardenHistoryKind::Installment => "installment",
            GardenHistoryKind::Cost => "cost",
            _ => "other_sale",
        };
        let number = vrepo::next_entry_number(tx, kind_text)?;
        let qty = match &ready.body {
            Body::OtherSale { qty, .. } => Some(*qty),
            Body::Payment { .. } => None,
        };
        let id = vrepo::insert_entry(
            tx,
            &NewEntry {
                venture_id: ready.garden.rec.id,
                kind: kind_text,
                entry_date: &date_iso,
                amount_paisa: ready.amount.as_paisa(),
                reason: ready.reason.as_deref(),
                purchase_id: None,
                qty,
                shop_name: ready.shop.as_deref(),
                cost_paisa: 0,
            },
            number,
            user,
            &now,
        )?;
        let (entry, posting_type) = match &ready.body {
            Body::Payment { payment } => {
                ventures::write_parts(tx, user, &now, id, &payment.parts)?;
                (payment.entry.as_ref(), PostingType::GardenPayment)
            }
            Body::OtherSale { entry, .. } => (Some(entry), PostingType::VentureReceipt),
        };
        let journal_entry_id = ventures::post_settlement(
            ctx,
            tx,
            user,
            &now,
            &Settlement {
                venture_entry_id: id,
                date: ready.date,
                entry,
                posting_type,
                source_type: SourceType::Venture,
                source_id: id,
            },
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "create", "garden_entry")
                .entity_id(id)
                .new_value(json!({
                    "number": number,
                    "kind": kind_text,
                    "garden_id": ready.garden.rec.id,
                    "entry_date": date_iso,
                    "amount_paisa": ready.amount.as_paisa(),
                    "reason": ready.reason,
                    "shop": ready.shop,
                    "qty": qty,
                    "journal_entry_id": journal_entry_id,
                    "cash_after_paisa": cash_check.cash_after_paisa.as_paisa(),
                    "cash_below_zero_confirmed": cash_check.needs_confirm,
                }))
                .into_entry(),
        )?;
        let version = vrepo::get_entry(tx, id)?
            .map(|e| e.version)
            .ok_or_else(|| data("a garden entry is missing after its save"))?;
        Ok(GardenEntrySaved {
            id,
            number,
            version,
            journal_entry_id,
            preview,
        })
    })
}

/// Cancels an installment, a running cost or a sale in another shop by reversal while its day is
/// open. Refused while the garden is closed: the closing was worked out with the entry, so undo the
/// closing first.
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_GARDEN_ENTRY_CANCELLED` and `ERR_GARDEN_CLOSED` on `id`;
/// `ERR_VERSION_CHANGED` on `version`; `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written when any of them is returned.
pub(super) fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &GardenEntryCancelInput,
) -> Result<GardenEntryCancelled, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let entry = vrepo::get_entry(tx, input.id)?
            .filter(|e| matches!(e.kind.as_str(), "installment" | "cost" | "other_sale"))
            .ok_or_else(|| not_found("id"))?;
        let garden = load(tx, entry.venture_id)?.ok_or_else(|| not_found("id"))?;
        if entry.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrGardenEntryCancelled,
                "id",
            ));
        }
        if !garden.is_open() {
            return Err(ServiceError::reject_field(ErrorCode::ErrGardenClosed, "id"));
        }
        if entry.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        let date = BusinessDate::parse_iso(&entry.entry_date)
            .ok_or_else(|| data("the date of a garden entry is not a date"))?;
        require_open_day(ctx, date, "id")?;
        let user = actor.user_id;
        let reversal_entry_id = match entry.journal_entry_id {
            Some(journal_id) => {
                // A cancel gives back what the shop parts took from cash, or takes back the money of
                // a sale in another shop (PRD FR-6.7).
                let stored = ledger_repo::get_entry(tx, journal_id)?
                    .ok_or_else(|| data("the entry of a garden entry cannot be read"))?;
                let change = stored_cash_change(tx, &stored)?
                    .checked_neg()
                    .map_err(money_error)?;
                cash::require_confirmed(
                    &cash::cash_after_entry(tx, change)?,
                    input.confirm_cash_below_zero,
                )?;
                Some(ledger::reverse(ctx, tx, user, journal_id, &reason)?.entry_id)
            }
            None => None,
        };
        vrepo::mark_entry_cancelled(tx, entry.id, &reason, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "cancel", "garden_entry")
                .entity_id(entry.id)
                .old(json!({
                    "number": entry.number,
                    "kind": entry.kind,
                    "status": "saved",
                    "amount_paisa": entry.amount_paisa,
                }))
                .new_value(json!({
                    "status": "cancelled",
                    "reversal_entry_id": reversal_entry_id,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(GardenEntryCancelled {
            id: entry.id,
            number: entry.number,
            reversal_entry_id,
        })
    })
}
