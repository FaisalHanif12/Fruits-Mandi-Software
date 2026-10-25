//! The reads of gardens: the list, one garden with its history and partners, the tile data, and the
//! figures of a garden (PRD FR-12.34, FR-12.37, FR-12.38, FR-19.11, plan 17c). Every figure is read
//! from stored rows; the only arithmetic is sums and differences with checked maths (Code Standards
//! B20, B31, money agent Section 9). Nothing is split again: a partner's part of the result is his
//! stored closing part less what he paid (a derived figure that adds up to the result exactly).

use std::collections::HashMap;

use mandi_db::gardens::{self as repo, AccountEvent, Figures, GardenRec};
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as vrepo, EntryRec, PartRec};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::gardens::{
    GardenDetail, GardenGetInput, GardenHistoryKind, GardenHistoryRow, GardenPartnerResult,
    GardenRow, GardenSummaryRow, GardensList, GardensListInput, GardensSummary, GardensTotals,
};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::partners::PartnerVentureCard;
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::ventures::{
    EntryPart, MoneySource, ShareAmount, VentureKind, VentureStatus,
};
use mandi_domain::ventures::{credit_working, payment_working};

use super::{DEFAULT_PAGE, data};
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ventures::{self, Venture, not_found, partner_rows, record_of};

pub(super) fn paisa(value: i64) -> Result<Paisa, ServiceError> {
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

/// A garden with its partners, as one action reads it.
#[derive(Debug, Clone)]
pub(super) struct Garden {
    /// The garden row with its account.
    pub rec: GardenRec,
    /// The shared venture side: the partners with their shares and sources.
    pub venture: Venture,
}

impl Garden {
    /// True while it takes entries.
    pub(super) fn is_open(&self) -> bool {
        self.rec.status == "open"
    }

    /// The day it was made.
    pub(super) fn date(&self) -> Result<BusinessDate, ServiceError> {
        BusinessDate::parse_iso(&self.rec.entry_date)
            .ok_or_else(|| data("the date of a garden is not a date"))
    }
}

/// Reads a garden and its partners.
///
/// # Errors
/// `ServiceError::Db`.
pub(super) fn load(conn: &Connection, id: i64) -> Result<Option<Garden>, ServiceError> {
    let Some(rec) = repo::get(conn, id)? else {
        return Ok(None);
    };
    let Some(venture) = ventures::load(conn, id)? else {
        return Ok(None);
    };
    Ok(Some(Garden { rec, venture }))
}

/// The figures of one garden as money, all checked.
#[derive(Debug, Clone)]
pub(super) struct Money {
    pub installments: Paisa,
    pub costs: Paisa,
    pub paid: Paisa,
    pub paid_shop: Paisa,
    pub paid_pocket: Paisa,
    pub other_shop: Paisa,
    pub rent_due: Paisa,
    pub money_in: Paisa,
    pub balance: Paisa,
    pub account_balance: Paisa,
    pub last_date: Option<String>,
}

impl Money {
    /// Works the figures out from the stored sums.
    ///
    /// # Errors
    /// `ServiceError::Data` if a stored sum does not fit or the parts of the payments do not add up
    /// to the payments; `ERR_OVERFLOW`.
    pub(super) fn of(rec: &GardenRec, f: &Figures) -> Result<Money, ServiceError> {
        let installments = paisa(f.installments_paisa)?;
        let costs = paisa(f.costs_paisa)?;
        let paid = installments.checked_add(costs).map_err(money_error)?;
        let paid_shop = paisa(f.paid_shop_paisa)?;
        let paid_pocket = paisa(f.paid_pocket_paisa)?;
        // The parts of every payment add up to the payment (C23), so the two sources add up to what
        // was paid.
        if paid_shop.checked_add(paid_pocket).map_err(money_error)? != paid {
            return Err(data("the parts of the payments of a garden do not add up"));
        }
        let money_in = paisa(f.credits_paisa)?
            .checked_sub(paisa(f.debits_paisa)?)
            .map_err(money_error)?;
        let balance = money_in.checked_sub(paid).map_err(money_error)?;
        let account_balance = paisa(f.account_credits_paisa)?
            .checked_sub(paisa(f.account_debits_paisa)?)
            .map_err(money_error)?;
        let rent_due = paisa(rec.rent_paisa)?
            .checked_sub(installments)
            .map_err(money_error)?;
        Ok(Money {
            installments,
            costs,
            paid,
            paid_shop,
            paid_pocket,
            other_shop: paisa(f.other_shop_paisa)?,
            rent_due,
            money_in,
            balance,
            account_balance,
            last_date: f.last_date.clone(),
        })
    }

    /// How the running balance was worked out (CalcInfo).
    pub(super) fn balance_working(&self) -> Working {
        balance_working(self.money_in, self.paid, self.balance)
    }
}

/// The working of a running balance: money in less everything paid.
pub(super) fn balance_working(money_in: Paisa, paid: Paisa, balance: Paisa) -> Working {
    Working {
        message_id: "working-garden-balance".to_owned(),
        args: vec![
            money_arg("money_in", money_in),
            money_arg("paid", paid),
            money_arg("result", balance),
        ],
        rule_id: None,
    }
}

/// One garden as the screens show it.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`.
pub(super) fn row_of(conn: &Connection, garden: &Garden) -> Result<GardenRow, ServiceError> {
    let figures = repo::figures(conn, &garden.rec)?;
    let money = Money::of(&garden.rec, &figures)?;
    row_with(conn, garden, &money)
}

fn row_with(conn: &Connection, garden: &Garden, money: &Money) -> Result<GardenRow, ServiceError> {
    let rec = &garden.rec;
    Ok(GardenRow {
        id: rec.id,
        version: rec.version,
        number: rec.number,
        name_ur: rec.name_ur.clone(),
        name_en: rec.name_en.clone(),
        account: rec.account.clone(),
        entry_date: rec.entry_date.clone(),
        item: rec.item.clone(),
        rent_paisa: paisa(rec.rent_paisa)?,
        season_end_date: rec.season_end_date.clone(),
        note: rec.note.clone(),
        status: VentureStatus::parse(&rec.status)
            .ok_or_else(|| data("a garden has a status that is not open or closed"))?,
        closed_date: rec.closed_date.clone(),
        shares_locked: mandi_db::gardens::shares_locked(conn, rec.id)?,
        partners: partner_rows(&garden.venture.partners)?,
        installments_paisa: money.installments,
        costs_paisa: money.costs,
        paid_paisa: money.paid,
        paid_shop_paisa: money.paid_shop,
        paid_pocket_paisa: money.paid_pocket,
        rent_due_paisa: money.rent_due,
        money_in_paisa: money.money_in,
        other_shop_paisa: money.other_shop,
        balance_paisa: money.balance,
        balance_working: money.balance_working(),
        account_balance_paisa: money.account_balance,
        last_date: money.last_date.clone(),
    })
}

/// One garden by id, as the screens show it.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ServiceError::Db`.
pub(super) fn row_by_id(conn: &Connection, id: i64) -> Result<GardenRow, ServiceError> {
    let garden = load(conn, id)?.ok_or_else(|| not_found("id"))?;
    row_of(conn, &garden)
}

/// One page of gardens with the totals over the whole filter.
pub(super) fn list(ctx: &Context, input: &GardensListInput) -> Result<GardensList, ServiceError> {
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
    let filter = vrepo::ListFilter {
        kind: "garden".to_owned(),
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
        // The totals are over every garden of the filter, never the page (Code Standards B31); the
        // gardens of a firm are few, so each is read once and the page is cut after.
        let ids = vrepo::ids_of(conn, &filter)?;
        let recs = repo::get_many(conn, &ids)?;
        let mut totals = GardensTotals {
            rent_paisa: Paisa::ZERO,
            paid_paisa: Paisa::ZERO,
            money_in_paisa: Paisa::ZERO,
            balance_paisa: Paisa::ZERO,
            rent_due_paisa: Paisa::ZERO,
        };
        let mut all = Vec::with_capacity(recs.len());
        for rec in recs {
            let garden = load(conn, rec.id)?.ok_or_else(|| data("a garden was lost"))?;
            let figures = repo::figures(conn, &garden.rec)?;
            let money = Money::of(&garden.rec, &figures)?;
            let add = |sum: Paisa, value: Paisa| sum.checked_add(value).map_err(money_error);
            totals.rent_paisa = add(totals.rent_paisa, paisa(garden.rec.rent_paisa)?)?;
            totals.paid_paisa = add(totals.paid_paisa, money.paid)?;
            totals.money_in_paisa = add(totals.money_in_paisa, money.money_in)?;
            totals.balance_paisa = add(totals.balance_paisa, money.balance)?;
            totals.rent_due_paisa = add(totals.rent_due_paisa, money.rent_due)?;
            all.push((garden, money));
        }
        // Newest day first, then by number.
        all.sort_by(|a, b| {
            b.0.rec
                .entry_date
                .cmp(&a.0.rec.entry_date)
                .then(b.0.rec.number.cmp(&a.0.rec.number))
        });
        let total_count =
            i64::try_from(all.len()).map_err(|_| data("the number of gardens does not fit"))?;
        let skip = usize::try_from(input.offset).unwrap_or(usize::MAX);
        let take = usize::try_from(limit).unwrap_or(usize::MAX);
        let mut rows = Vec::new();
        for (garden, money) in all.iter().skip(skip).take(take) {
            rows.push(row_with(conn, garden, money)?);
        }
        Ok(GardensList {
            rows,
            total_count,
            totals,
        })
    })
}

/// The data of the dashboard tile باغات.
pub(super) fn summary(ctx: &Context) -> Result<GardensSummary, ServiceError> {
    ctx.db.read(|conn| {
        let filter = vrepo::ListFilter {
            kind: "garden".to_owned(),
            status: Some("open".to_owned()),
            ..vrepo::ListFilter::default()
        };
        let ids = vrepo::ids_of(conn, &filter)?;
        let mut rows = Vec::with_capacity(ids.len());
        let mut balance_total = Paisa::ZERO;
        for rec in repo::get_many(conn, &ids)? {
            let figures = repo::figures(conn, &rec)?;
            let money = Money::of(&rec, &figures)?;
            balance_total = balance_total
                .checked_add(money.balance)
                .map_err(money_error)?;
            rows.push(GardenSummaryRow {
                id: rec.id,
                number: rec.number,
                name_ur: rec.name_ur.clone(),
                name_en: rec.name_en.clone(),
                balance_paisa: money.balance,
                rent_due_paisa: money.rent_due,
            });
        }
        rows.sort_by_key(|r| r.number);
        Ok(GardensSummary {
            open_gardens: i64::try_from(rows.len())
                .map_err(|_| data("the number of gardens does not fit"))?,
            balance_total_paisa: balance_total,
            rows,
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

/// A stored part of a closing as the screens show it (signed).
fn credit_view(rec: &PartRec, balance: Paisa) -> Result<ShareAmount, ServiceError> {
    let share = i32::try_from(rec.share_bps).map_err(|_| data("a stored share is out of range"))?;
    let amount = paisa(rec.amount_paisa)?;
    Ok(ShareAmount {
        partner: rec.partner.clone(),
        share_bps: share,
        amount_paisa: amount,
        working: credit_working(balance, share, amount),
    })
}

/// One thing in a garden's history before the running balance is put on it.
struct Item {
    /// The day, `YYYY-MM-DD`.
    date: String,
    /// 0 payments, 1 money in, 2 the closing: the order within a day.
    group: u8,
    /// Breaks ties: the venture entry id for a payment and the closing, the journal entry id for
    /// money in.
    order: i64,
    row: GardenHistoryRow,
}

fn history_kind_of(kind: &str) -> Result<GardenHistoryKind, ServiceError> {
    match kind {
        "installment" => Ok(GardenHistoryKind::Installment),
        "cost" => Ok(GardenHistoryKind::Cost),
        "other_sale" => Ok(GardenHistoryKind::OtherSale),
        "close" => Ok(GardenHistoryKind::Close),
        _ => Err(data("a garden entry has a kind that is not known")),
    }
}

/// The whole history of a garden, oldest first by day, each row with the running balance after it
/// (PRD FR-12.37). Payments come first within a day, then money in by journal entry, then the closing.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`; `ERR_OVERFLOW`.
pub(super) fn history(
    conn: &Connection,
    garden: &Garden,
) -> Result<Vec<GardenHistoryRow>, ServiceError> {
    let open = garden.is_open();
    let (entries, _) = vrepo::entries_page(conn, garden.rec.id, 0, i64::MAX)?;
    let ids: Vec<i64> = entries.iter().map(|e| e.id).collect();
    let parts = vrepo::parts_of(conn, &ids)?;
    let mut items: Vec<Item> = Vec::with_capacity(entries.len());
    for entry in &entries {
        items.push(entry_item(entry, &parts, open)?);
    }
    for event in repo::account_events(conn, garden.rec.account.id)? {
        if let Some(item) = event_item(&event)? {
            items.push(item);
        }
    }
    items.sort_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then(a.group.cmp(&b.group))
            .then(a.order.cmp(&b.order))
    });
    let mut running = Paisa::ZERO;
    let mut out = Vec::with_capacity(items.len());
    for mut item in items {
        running = running
            .checked_add(item.row.effect_paisa)
            .map_err(money_error)?;
        item.row.running_balance_paisa = running;
        out.push(item.row);
    }
    Ok(out)
}

/// A venture entry as a row of the history.
fn entry_item(
    entry: &EntryRec,
    parts: &[PartRec],
    garden_open: bool,
) -> Result<Item, ServiceError> {
    let kind = history_kind_of(&entry.kind)?;
    let cancelled = entry.status == "cancelled";
    let amount = paisa(entry.amount_paisa)?;
    let mine: Vec<&PartRec> = parts.iter().filter(|p| p.entry_id == entry.id).collect();
    let mut payment_parts = Vec::new();
    let mut credits = Vec::new();
    match kind {
        GardenHistoryKind::Installment | GardenHistoryKind::Cost => {
            for rec in &mine {
                payment_parts.push(part_view(rec, amount)?);
            }
        }
        GardenHistoryKind::Close => {
            // The balance moved out, signed like the parts (a loss has negative parts).
            let mut balance = Paisa::ZERO;
            for rec in &mine {
                balance = balance
                    .checked_add(paisa(rec.amount_paisa)?)
                    .map_err(money_error)?;
            }
            for rec in &mine {
                credits.push(credit_view(rec, balance)?);
            }
        }
        _ => {}
    }
    // What the row does to the running balance: a payment takes away, money in adds, the closing
    // moves the balance out to the partners and is not money in. A cancelled entry counts for nothing
    // (its posting and its reversal net to 0).
    let effect = match (kind, cancelled) {
        (_, true) | (GardenHistoryKind::Close, _) => Paisa::ZERO,
        (GardenHistoryKind::Installment | GardenHistoryKind::Cost, false) => {
            amount.checked_neg().map_err(money_error)?
        }
        (_, false) => amount,
    };
    let group = match kind {
        GardenHistoryKind::Installment | GardenHistoryKind::Cost => 0,
        GardenHistoryKind::Close => 2,
        _ => 1,
    };
    // Money in is ordered by its journal entry so it sits with the sales of the same day in the order
    // they were posted; a payment by its own entry id.
    let order = match group {
        1 => entry.journal_entry_id.unwrap_or(entry.id),
        _ => entry.id,
    };
    let can_cancel = !cancelled
        && garden_open
        && matches!(
            kind,
            GardenHistoryKind::Installment | GardenHistoryKind::Cost | GardenHistoryKind::OtherSale
        );
    Ok(Item {
        date: entry.entry_date.clone(),
        group,
        order,
        row: GardenHistoryRow {
            entry_date: entry.entry_date.clone(),
            kind,
            entry_id: Some(entry.id),
            version: Some(entry.version),
            bikri_id: None,
            bikri_number: None,
            amount_paisa: amount,
            effect_paisa: effect,
            running_balance_paisa: Paisa::ZERO,
            reason: entry.reason.clone(),
            shop_name: entry.shop_name.clone(),
            qty: entry.qty,
            journal_entry_id: entry.journal_entry_id,
            cancelled,
            cancel_reason: entry.cancel_reason.clone(),
            parts: payment_parts,
            credits,
            can_cancel,
        },
    })
}

/// A journal entry that touched the garden's account as a row of the history: the seller credit of a
/// lot sold in our shop (and its reversal), or fruit taken from its lots. The sales in other shops
/// (source `venture`) are shown by their own entry.
fn event_item(event: &AccountEvent) -> Result<Option<Item>, ServiceError> {
    let kind = match event.source_type.as_str() {
        "venture" => return Ok(None),
        "fruit" => GardenHistoryKind::Fruit,
        "bikri" => GardenHistoryKind::Sale,
        other => {
            return Err(data(&format!(
                "a line of a garden's account comes from {other}, which is not a sale or fruit"
            )));
        }
    };
    let net = paisa(event.cr_paisa)?
        .checked_sub(paisa(event.dr_paisa)?)
        .map_err(money_error)?;
    Ok(Some(Item {
        date: event.entry_date.clone(),
        group: 1,
        order: event.entry_id,
        row: GardenHistoryRow {
            entry_date: event.entry_date.clone(),
            kind,
            entry_id: None,
            version: None,
            bikri_id: (kind == GardenHistoryKind::Sale).then_some(event.source_id),
            bikri_number: None,
            amount_paisa: net,
            effect_paisa: net,
            running_balance_paisa: Paisa::ZERO,
            reason: None,
            shop_name: None,
            qty: None,
            journal_entry_id: Some(event.entry_id),
            cancelled: false,
            cancel_reason: None,
            parts: Vec::new(),
            credits: Vec::new(),
            can_cancel: false,
        },
    }))
}

/// Puts the number of each بکری on its sale rows (one statement for all of them).
fn put_bikri_numbers(conn: &Connection, rows: &mut [GardenHistoryRow]) -> Result<(), ServiceError> {
    let ids: Vec<i64> = rows.iter().filter_map(|r| r.bikri_id).collect();
    if ids.is_empty() {
        return Ok(());
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT id, number FROM bikri_entries WHERE id IN ({marks})"
    ))?;
    let numbers: HashMap<i64, i64> = statement
        .query_map(mandi_db::rusqlite::params_from_iter(ids.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<Result<_, _>>()?;
    for row in rows {
        if let Some(id) = row.bikri_id {
            row.bikri_number = numbers.get(&id).copied();
        }
    }
    Ok(())
}

/// Each partner's money in and money back on a garden (PRD FR-12.38, FR-17.10, E2E-40).
///
/// His part of the result is his stored closing part less what he paid from the shop and from his
/// pocket. It is derived from stored rows and never `split_by_bps` of the result, so the parts of all
/// partners add up to the result exactly.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`; `ERR_OVERFLOW`.
pub(super) fn partner_results(
    conn: &Connection,
    garden: &Garden,
) -> Result<Vec<GardenPartnerResult>, ServiceError> {
    let paid = repo::paid_by_partner(conn, garden.rec.id)?;
    let closing: HashMap<i64, i64> = match repo::saved_close_entry(conn, garden.rec.id)? {
        Some(entry_id) => vrepo::parts_of(conn, &[entry_id])?
            .into_iter()
            .map(|p| (p.partner.id, p.amount_paisa))
            .collect(),
        None => HashMap::new(),
    };
    garden
        .venture
        .partners
        .iter()
        .map(|p| {
            let (shop, pocket) = paid.get(&p.partner_id).copied().unwrap_or((0, 0));
            let back = closing.get(&p.partner_id).copied().unwrap_or(0);
            result_of(
                p.partner_id,
                &p.name_ur,
                &p.name_en,
                p.share_bps,
                paisa(shop)?,
                paisa(pocket)?,
                paisa(back)?,
            )
        })
        .collect()
}

/// One partner's result from his three stored figures.
///
/// # Errors
/// `ServiceError::Data` if the share is out of range; `ERR_OVERFLOW`.
pub(super) fn result_of(
    partner_id: i64,
    name_ur: &str,
    name_en: &str,
    share_bps: i64,
    paid_shop: Paisa,
    paid_pocket: Paisa,
    back: Paisa,
) -> Result<GardenPartnerResult, ServiceError> {
    let net = back
        .checked_sub(paid_shop)
        .and_then(|v| v.checked_sub(paid_pocket))
        .map_err(money_error)?;
    Ok(GardenPartnerResult {
        partner: record_of(partner_id, name_ur, name_en),
        share_bps: i32::try_from(share_bps).map_err(|_| data("a stored share is out of range"))?,
        paid_shop_paisa: paid_shop,
        paid_pocket_paisa: paid_pocket,
        money_back_paisa: back,
        net_paisa: net,
        working: Working {
            message_id: "working-trip-partner-net".to_owned(),
            args: vec![
                money_arg("in_shop", paid_shop),
                money_arg("in_pocket", paid_pocket),
                money_arg("back", back),
                money_arg("result", net),
            ],
            rule_id: None,
        },
    })
}

/// One garden with its history and each partner's money.
pub(super) fn get(ctx: &Context, input: &GardenGetInput) -> Result<GardenDetail, ServiceError> {
    let limit = if input.history_limit == 0 {
        DEFAULT_PAGE
    } else {
        input.history_limit
    };
    ctx.db.read(|conn| {
        let garden = load(conn, input.id)?.ok_or_else(|| not_found("id"))?;
        let row = row_of(conn, &garden)?;
        let all = history(conn, &garden)?;
        let history_count =
            i64::try_from(all.len()).map_err(|_| data("the history does not fit"))?;
        let mut page: Vec<GardenHistoryRow> = all
            .into_iter()
            .skip(usize::try_from(input.history_offset).unwrap_or(usize::MAX))
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .collect();
        put_bikri_numbers(conn, &mut page)?;
        let partner_results = partner_results(conn, &garden)?;
        Ok(GardenDetail {
            row,
            history: page,
            history_count,
            partner_results,
        })
    })
}

/// The open gardens a partner is in, as the cards under his name (PRD 17.3, plan 17c): his %, what he
/// paid so far from the shop and from his pocket, and the garden's running balance. A garden pays its
/// partners only when it is closed (C26), so his money back is 0 here.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`; `ERR_OVERFLOW`.
pub(crate) fn cards_of_partner(
    conn: &Connection,
    partner_id: i64,
) -> Result<Vec<PartnerVentureCard>, ServiceError> {
    let mut cards = Vec::new();
    for (garden_id, share_bps) in repo::open_gardens_of_partner(conn, partner_id)? {
        let garden = load(conn, garden_id)?.ok_or_else(|| data("a garden was lost"))?;
        let figures = repo::figures(conn, &garden.rec)?;
        let money = Money::of(&garden.rec, &figures)?;
        let paid = repo::paid_by_partner(conn, garden_id)?;
        let (shop, pocket) = paid.get(&partner_id).copied().unwrap_or((0, 0));
        cards.push(PartnerVentureCard {
            venture_id: garden.rec.id,
            kind: VentureKind::Garden,
            number: garden.rec.number,
            name_ur: garden.rec.name_ur.clone(),
            name_en: garden.rec.name_en.clone(),
            share_bps: i32::try_from(share_bps)
                .map_err(|_| data("a stored share is out of range"))?,
            paid_shop_paisa: paisa(shop)?,
            paid_pocket_paisa: paisa(pocket)?,
            money_back_paisa: Paisa::ZERO,
            balance_paisa: money.balance,
        });
    }
    Ok(cards)
}
