//! The reads of market trips: the form defaults, the list, one trip with its history, and the tile
//! data (PRD FR-12.25, FR-12.27, FR-19.11, plan 16b). Every figure is read from stored rows;
//! nothing is worked out again except sums, differences and the C25 split of a lot's credit (the
//! same function the posting used, on the same stored credit and the same fixed shares), all with
//! checked maths (Code Standards B20, B31).

use std::collections::HashMap;

use mandi_db::purchases as purchase_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as repo, EntryRec, PartRec, PurchaseFigures, VentureRec};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::trips::{
    CostSpreadPart, TripDefaults, TripDetail, TripEntryKind, TripEntryRow, TripGetInput,
    TripPartnerResult, TripRow, TripsList, TripsListInput, TripsSummary, TripsTotals,
};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::ventures::{
    EntryPart, MoneySource, ShareAmount, VenturePartnerRow, VentureStatus,
};
use mandi_domain::ventures::{credit_working, payment_working, spread_working};

use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::purchases;
use crate::ventures::{self, Venture, not_found, partner_rows, record_of};

/// The default page of a list (PRD FR-19.11).
const DEFAULT_PAGE: u32 = 100;
/// How many ids go into one `IN (...)` list.
const CHUNK: usize = 500;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn money_arg(name: &str, value: Paisa) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Money { paisa: value },
    }
}

/// An optional date of a filter: empty or missing is no limit.
fn optional_date(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&String>,
    today: BusinessDate,
) -> Option<String> {
    let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
    errors
        .take(validate::date(field, text, today))
        .map(BusinessDate::to_iso)
}

/// The figures of ventures, from grouped queries in chunks (Code Standards B22).
fn figures_of(
    conn: &Connection,
    ids: &[i64],
) -> Result<HashMap<i64, PurchaseFigures>, ServiceError> {
    let mut out = HashMap::with_capacity(ids.len());
    for chunk in ids.chunks(CHUNK) {
        out.extend(repo::purchase_figures(conn, chunk)?);
    }
    Ok(out)
}

/// The money figures of a trip (PRD FR-12.27) worked out from the stored sums.
struct Money {
    sales: Paisa,
    other_shop: Paisa,
    released: Paisa,
    cost_sold: Paisa,
    cost_written_off: Paisa,
    result: Paisa,
}

impl Money {
    /// Sales money plus other-shop money, less the cost released (sold and written off).
    fn of(f: &PurchaseFigures) -> Result<Self, ServiceError> {
        let sales = paisa(f.sales_paisa)?;
        let other_shop = paisa(f.other_shop_paisa)?;
        let released = paisa(f.cost_released_paisa)?;
        let cost_written_off = paisa(f.cost_written_off_paisa)?;
        let cost_sold = released
            .checked_sub(cost_written_off)
            .map_err(money_error)?;
        let result = sales
            .checked_add(other_shop)
            .and_then(|v| v.checked_sub(released))
            .map_err(money_error)?;
        Ok(Money {
            sales,
            other_shop,
            released,
            cost_sold,
            cost_written_off,
            result,
        })
    }

    fn working(&self) -> Working {
        Working {
            message_id: "working-trip-result".to_owned(),
            args: vec![
                money_arg("sales", self.sales),
                money_arg("other_shop", self.other_shop),
                money_arg("cost_sold", self.cost_sold),
                money_arg("cost_written_off", self.cost_written_off),
                money_arg("result", self.result),
            ],
            rule_id: None,
        }
    }
}

/// One trip as the lists show it.
fn row_of(
    rec: &VentureRec,
    partners: &[repo::PartnerShare],
    figures: PurchaseFigures,
    locked: bool,
) -> Result<TripRow, ServiceError> {
    let money = Money::of(&figures)?;
    let qty_left = figures
        .qty_bought
        .checked_sub(figures.qty_sold)
        .and_then(|v| v.checked_sub(figures.qty_written_off))
        .ok_or_else(|| data("the quantity of a trip does not fit"))?;
    let cost = paisa(figures.cost_paisa)?;
    let cost_left = cost.checked_sub(money.released).map_err(money_error)?;
    Ok(TripRow {
        id: rec.id,
        version: rec.version,
        number: rec.number,
        name_ur: rec.name_ur.clone(),
        name_en: rec.name_en.clone(),
        entry_date: rec.entry_date.clone(),
        market: rec.market.clone(),
        sent_person: rec.sent_person.clone(),
        note: rec.note.clone(),
        status: VentureStatus::parse(&rec.status)
            .ok_or_else(|| data("a trip has a status that is not open or closed"))?,
        closed_date: rec.closed_date.clone(),
        shares_locked: locked,
        partners: partner_rows(partners)?,
        purchase_count: figures.purchase_count,
        qty_bought: figures.qty_bought,
        qty_sold: figures.qty_sold,
        qty_written_off: figures.qty_written_off,
        qty_left,
        cost_paisa: cost,
        cost_left_paisa: cost_left,
        sales_paisa: money.sales,
        other_shop_paisa: money.other_shop,
        cost_sold_paisa: money.cost_sold,
        cost_written_off_paisa: money.cost_written_off,
        result_paisa: money.result,
        result_working: money.working(),
    })
}

/// The rows of trips, with partners and figures from grouped queries.
pub(super) fn rows_of(
    conn: &Connection,
    recs: &[VentureRec],
) -> Result<Vec<TripRow>, ServiceError> {
    let ids: Vec<i64> = recs.iter().map(|r| r.id).collect();
    let partners = repo::partners_of_many(conn, &ids)?;
    let figures = figures_of(conn, &ids)?;
    let locked = repo::with_entries(conn, &ids)?;
    recs.iter()
        .map(|rec| {
            row_of(
                rec,
                partners.get(&rec.id).map_or(&[], Vec::as_slice),
                figures.get(&rec.id).copied().unwrap_or_default(),
                locked.contains(&rec.id),
            )
        })
        .collect()
}

/// One trip as a row, read inside the caller's transaction.
pub(super) fn row_by_id(conn: &Connection, id: i64) -> Result<TripRow, ServiceError> {
    let rec = repo::get(conn, id)?.ok_or_else(|| not_found("id"))?;
    rows_of(conn, std::slice::from_ref(&rec))?
        .pop()
        .ok_or_else(|| data("a trip has no row"))
}

/// The form of a new trip (PRD A56): the partners with the % and the sources of the last trip,
/// else the active partners with the firm's shares of today and the source `shop`.
pub(super) fn defaults(ctx: &Context) -> Result<TripDefaults, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let active = repo::active_partner_ids(conn)?;
        let last = repo::partners_of_last_trip(conn)?;
        let names: HashMap<i64, (String, String)> = mandi_db::partners::list(conn)?
            .into_iter()
            .map(|p| (p.id, (p.name_ur, p.name_en)))
            .collect();
        let from_last_trip = !last.is_empty();
        let firm: HashMap<i64, i64> = mandi_db::partners::shares_on(conn, &today.to_iso())?
            .into_iter()
            .collect();
        let mut rows = Vec::with_capacity(active.len());
        for id in active {
            let (name_ur, name_en) = names.get(&id).cloned().unwrap_or_default();
            let (share_bps, source) = if from_last_trip {
                match last.iter().find(|p| p.partner_id == id) {
                    Some(p) => (p.share_bps, p.source.clone()),
                    None => (0, MoneySource::Shop.as_str().to_owned()),
                }
            } else {
                (
                    firm.get(&id).copied().unwrap_or(0),
                    MoneySource::Shop.as_str().to_owned(),
                )
            };
            rows.push(VenturePartnerRow {
                partner: record_of(id, &name_ur, &name_en),
                share_bps: i32::try_from(share_bps)
                    .map_err(|_| data("a stored share is out of range"))?,
                source: MoneySource::parse(&source)
                    .ok_or_else(|| data("a stored money source is not shop or pocket"))?,
            });
        }
        Ok(TripDefaults {
            partners: rows,
            from_last_trip,
        })
    })
}

/// One page of trips with the totals over the whole filter.
pub(super) fn list(ctx: &Context, input: &TripsListInput) -> Result<TripsList, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let from = optional_date(&mut errors, "from", input.from.as_ref(), today);
    let to = optional_date(&mut errors, "to", input.to.as_ref(), today);
    errors.finish()?;
    let search = input
        .search
        .as_ref()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty());
    let filter = repo::ListFilter {
        kind: "trip".to_owned(),
        from,
        to,
        status: input.status.map(|s| s.as_str().to_owned()),
        partner_id: input.partner_id,
        search,
    };
    let limit = if input.limit == 0 {
        DEFAULT_PAGE
    } else {
        input.limit
    };
    ctx.db.read(|conn| {
        let recs = repo::list(conn, &filter, i64::from(input.offset), i64::from(limit))?;
        let rows = rows_of(conn, &recs)?;
        let ids = repo::ids_of(conn, &filter)?;
        let figures = figures_of(conn, &ids)?;
        let mut totals = TripsTotals {
            qty_left: 0,
            cost_left_paisa: Paisa::ZERO,
            cost_paisa: Paisa::ZERO,
            sales_paisa: Paisa::ZERO,
            result_paisa: Paisa::ZERO,
        };
        for f in figures.values() {
            let money = Money::of(f)?;
            let left = f
                .qty_bought
                .checked_sub(f.qty_sold)
                .and_then(|v| v.checked_sub(f.qty_written_off))
                .ok_or_else(|| data("the quantity of a trip does not fit"))?;
            let cost = paisa(f.cost_paisa)?;
            totals.qty_left = totals
                .qty_left
                .checked_add(left)
                .ok_or_else(|| data("the quantity of the trips does not fit"))?;
            totals.cost_paisa = totals.cost_paisa.checked_add(cost).map_err(money_error)?;
            totals.cost_left_paisa = totals
                .cost_left_paisa
                .checked_add(cost.checked_sub(money.released).map_err(money_error)?)
                .map_err(money_error)?;
            totals.sales_paisa = totals
                .sales_paisa
                .checked_add(money.sales)
                .and_then(|v| v.checked_add(money.other_shop))
                .map_err(money_error)?;
            totals.result_paisa = totals
                .result_paisa
                .checked_add(money.result)
                .map_err(money_error)?;
        }
        Ok(TripsList {
            rows,
            total_count: u32::try_from(ids.len())
                .map_err(|_| data("the number of trips does not fit"))?,
            totals,
        })
    })
}

/// The data of the dashboard tile مارکیٹ سفر.
pub(super) fn summary(ctx: &Context) -> Result<TripsSummary, ServiceError> {
    ctx.db.read(|conn| {
        let (open_trips, purchases_with_stock, qty_left, cost_left) = repo::tile_totals(conn)?;
        Ok(TripsSummary {
            open_trips,
            purchases_with_stock,
            qty_left,
            cost_left_paisa: paisa(cost_left)?,
        })
    })
}

/// A stored part of a payment as the screens show it.
fn part_view(rec: &PartRec, entry_amount: Paisa) -> Result<EntryPart, ServiceError> {
    let share = i32::try_from(rec.share_bps).map_err(|_| data("a stored share is out of range"))?;
    let amount = paisa(rec.amount_paisa)?;
    Ok(EntryPart {
        partner: rec.partner.clone(),
        share_bps: share,
        source: MoneySource::parse(&rec.source)
            .ok_or_else(|| data("a stored money source is not shop or pocket"))?,
        amount_paisa: amount,
        working: payment_working(entry_amount, share, amount),
    })
}

/// A stored credit of money that came in, as the screens show it.
fn credit_view(rec: &PartRec, entry_amount: Paisa) -> Result<ShareAmount, ServiceError> {
    let share = i32::try_from(rec.share_bps).map_err(|_| data("a stored share is out of range"))?;
    let amount = paisa(rec.amount_paisa)?;
    Ok(ShareAmount {
        partner: rec.partner.clone(),
        share_bps: share,
        amount_paisa: amount,
        working: credit_working(entry_amount, share, amount),
    })
}

/// The history of a trip: one row for each entry, newest first.
fn entry_rows(
    conn: &Connection,
    recs: &[EntryRec],
    purchase_rows: &[mandi_domain::core::purchases::PurchaseRow],
) -> Result<Vec<TripEntryRow>, ServiceError> {
    let ids: Vec<i64> = recs.iter().map(|r| r.id).collect();
    let parts = repo::parts_of(conn, &ids)?;
    let cost_ids: Vec<i64> = recs
        .iter()
        .filter(|r| r.kind == "cost")
        .map(|r| r.id)
        .collect();
    let spreads = purchase_repo::cost_parts_detail(conn, &cost_ids)?;
    let mut out = Vec::with_capacity(recs.len());
    for rec in recs {
        let kind = TripEntryKind::parse(&rec.kind).ok_or_else(|| {
            data("a trip entry has a kind that is not purchase, cost or other sale")
        })?;
        let amount = paisa(rec.amount_paisa)?;
        let mine: Vec<&PartRec> = parts.iter().filter(|p| p.entry_id == rec.id).collect();
        let (payment_parts, credits) = if kind == TripEntryKind::OtherSale {
            (
                Vec::new(),
                mine.iter()
                    .map(|p| credit_view(p, amount))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        } else {
            (
                mine.iter()
                    .map(|p| part_view(p, amount))
                    .collect::<Result<Vec<_>, _>>()?,
                Vec::new(),
            )
        };
        let details: Vec<&purchase_repo::CostPartDetail> =
            spreads.iter().filter(|d| d.entry_id == rec.id).collect();
        let total_qty = details
            .iter()
            .try_fold(0_i64, |sum, d| sum.checked_add(d.qty))
            .ok_or_else(|| data("the quantity of a spread does not fit"))?;
        let spread = details
            .iter()
            .map(|d| {
                let part = paisa(d.amount_paisa)?;
                Ok(CostSpreadPart {
                    purchase_id: d.purchase_id,
                    purchase_number: d.purchase_number,
                    item: d.item.clone(),
                    qty: d.qty,
                    amount_paisa: part,
                    working: spread_working(amount, d.qty, total_qty, part),
                })
            })
            .collect::<Result<Vec<_>, ServiceError>>()?;
        let cancelled = rec.status == "cancelled";
        let can_cancel = !cancelled
            && match kind {
                TripEntryKind::Purchase => purchase_rows
                    .iter()
                    .find(|p| Some(p.id) == rec.purchase_id)
                    .is_some_and(|p| p.can_cancel),
                TripEntryKind::Cost => details
                    .iter()
                    .all(|d| d.qty_sold == 0 && d.qty_written_off == 0),
                TripEntryKind::OtherSale => true,
            };
        out.push(TripEntryRow {
            id: rec.id,
            version: rec.version,
            kind,
            number: rec.number,
            entry_date: rec.entry_date.clone(),
            amount_paisa: amount,
            reason: rec.reason.clone(),
            shop_name: rec.shop_name.clone(),
            qty: rec.qty,
            purchase_id: rec.purchase_id,
            purchase_number: rec.purchase_number,
            cost_paisa: paisa(rec.cost_paisa)?,
            journal_entry_id: rec.journal_entry_id,
            cancelled,
            cancel_reason: rec.cancel_reason.clone(),
            parts: payment_parts,
            credits,
            spread,
            can_cancel,
        });
    }
    Ok(out)
}

/// Each partner's money in and money back on a trip (PRD FR-12.27, E2E-34b).
fn partner_results(
    conn: &Connection,
    venture: &Venture,
) -> Result<Vec<TripPartnerResult>, ServiceError> {
    let money = repo::partner_money(conn, venture.rec.id)?;
    // The partners' parts of the seller credits of the trip's own lots, as stored with each posting
    // (C25, reversals counted).
    let lot_back = repo::lot_credits_by_partner(conn, venture.rec.id)?;
    venture
        .partners
        .iter()
        .map(|p| {
            let m = money.get(&p.partner_id).copied().unwrap_or_default();
            let in_shop = paisa(m.in_shop_paisa)?;
            let in_pocket = paisa(m.in_pocket_paisa)?;
            let back = paisa(m.back_other_paisa)?
                .checked_add(paisa(lot_back.get(&p.partner_id).copied().unwrap_or(0))?)
                .map_err(money_error)?;
            let net = back
                .checked_sub(in_shop)
                .and_then(|v| v.checked_sub(in_pocket))
                .map_err(money_error)?;
            Ok(TripPartnerResult {
                partner: record_of(p.partner_id, &p.name_ur, &p.name_en),
                share_bps: i32::try_from(p.share_bps)
                    .map_err(|_| data("a stored share is out of range"))?,
                money_in_shop_paisa: in_shop,
                money_in_pocket_paisa: in_pocket,
                money_back_paisa: back,
                net_paisa: net,
                working: Working {
                    message_id: "working-trip-partner-net".to_owned(),
                    args: vec![
                        money_arg("in_shop", in_shop),
                        money_arg("in_pocket", in_pocket),
                        money_arg("back", back),
                        money_arg("result", net),
                    ],
                    rule_id: None,
                },
            })
        })
        .collect()
}

/// One trip with its purchases, history and each partner's money.
pub(super) fn get(ctx: &Context, input: &TripGetInput) -> Result<TripDetail, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let limit = if input.entries_limit == 0 {
        DEFAULT_PAGE
    } else {
        input.entries_limit
    };
    ctx.db.read(|conn| {
        let venture = ventures::load(conn, input.id)?
            .filter(|v| v.rec.kind == "trip")
            .ok_or_else(|| not_found("id"))?;
        let row = row_by_id(conn, input.id)?;
        let purchases = purchases::rows_of_trip(conn, input.id, today)?;
        let (recs, entries_count) = repo::entries_page(
            conn,
            input.id,
            i64::from(input.entries_offset),
            i64::from(limit),
        )?;
        let entries = entry_rows(conn, &recs, &purchases)?;
        let partner_results = partner_results(conn, &venture)?;
        Ok(TripDetail {
            row,
            purchases,
            entries,
            entries_count,
            partner_results,
        })
    })
}

/// The open trips a partner is in, as the cards under his name (PRD 17.3): his %, what he paid so
/// far from the shop and from his pocket, what he got back, and the result of the trip so far.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`.
pub(crate) fn cards_of_partner(
    conn: &Connection,
    partner_id: i64,
) -> Result<Vec<mandi_domain::core::partners::PartnerVentureCard>, ServiceError> {
    let filter = repo::ListFilter {
        kind: "trip".to_owned(),
        status: Some("open".to_owned()),
        partner_id: Some(partner_id),
        ..repo::ListFilter::default()
    };
    let mut ids = repo::ids_of(conn, &filter)?;
    ids.sort_unstable();
    let mut cards = Vec::with_capacity(ids.len());
    for id in ids {
        let venture = ventures::load(conn, id)?.ok_or_else(|| not_found("id"))?;
        let row = row_by_id(conn, id)?;
        let mine = partner_results(conn, &venture)?
            .into_iter()
            .find(|r| r.partner.id == partner_id)
            .ok_or_else(|| data("a partner of a trip lost his row"))?;
        cards.push(mandi_domain::core::partners::PartnerVentureCard {
            venture_id: id,
            kind: mandi_domain::core::ventures::VentureKind::Trip,
            number: row.number,
            name_ur: row.name_ur.clone(),
            name_en: row.name_en.clone(),
            share_bps: mine.share_bps,
            paid_shop_paisa: mine.money_in_shop_paisa,
            paid_pocket_paisa: mine.money_in_pocket_paisa,
            money_back_paisa: mine.money_back_paisa,
            balance_paisa: row.result_paisa,
        });
    }
    Ok(cards)
}
