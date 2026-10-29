//! The figures of the gardens as the reports need them (PRD 12.11, FR-12.37, FR-12.38, plan 17c and
//! plan d5-late round 11): one place that reads a garden's stored rows once, so R36, the analytics
//! view I, view G, view F, the day summary and the dashboard tile all show the same numbers as the
//! garden screen.
//!
//! Nothing is split or worked out again with a rule. The figures are:
//!
//! * the sums of the garden ([`mandi_db::gardens::figures`]) and the money derived from them
//!   ([`mandi_db::gardens::Money::of`], the one function the garden screens use too);
//! * where the money in came from: sales in our shop (lines of بکری on its account), sales in other
//!   shops (the saved entries) and fruit taken from its lots, from one grouped query over all the
//!   gardens. The three add up to the money in, and a line from any other source is an error, so
//!   no money falls into a bucket silently;
//! * the commission on its goods, from the sale facts of its lots: seller commission, buyer
//!   commission and لاگا, the facts with sign +1 and with sign −1 summed apart and subtracted in
//!   Rust. It is the commission over the whole life of the garden, not narrowed by the period of a
//!   report (a garden is picked by the day it was made, and its goods are sold on later days);
//! * each partner's money: what he paid from the shop and from his pocket (the stored parts of the
//!   installments and costs) and what he got back (his stored part of the saved closing). His result
//!   is back less paid. It is never a new split of the garden's result, so the results of all the
//!   partners add up to the running balance exactly once the garden is closed (for an open garden
//!   nothing has come back and each result is minus what he paid).
//!
//! Example (E2E-40, after the closing): rent 50,000, paid 55,000 (shop 33,000, pocket 22,000), our
//! shop 56,400, other shops 10,000, money in 66,400, commission 8,000, balance 11,400; الف 60%:
//! paid 33,000 from the shop, back 39,840, result 6,840; ب 40%: paid 22,000 from his pocket, back
//! 26,560, result 4,560.

use std::collections::HashMap;

use mandi_core::money::Paisa;
use mandi_db::garden_figures::{Money, MoneyDataError, money_in_by_source};
use mandi_db::gardens::{self as repo, GardenRec};
use mandi_db::rusqlite::{Connection, params_from_iter};
use mandi_db::ventures::{self as vrepo, ListFilter, PartnerShare};

use crate::error::ReportsError;
use crate::reports::common::{IN_CHUNK, add, paisa};

/// Which gardens a report picks (PRD 18.1, 19.10). A garden is picked by the day it was made, its
/// main fruit, a partner who is in it, or by itself.
#[derive(Debug, Clone, Default)]
pub(crate) struct Pick {
    /// First day made, `YYYY-MM-DD`.
    pub(crate) from: Option<String>,
    /// Last day made, `YYYY-MM-DD`.
    pub(crate) to: Option<String>,
    /// The main fruit of the garden.
    pub(crate) item_id: Option<i64>,
    /// A partner who is in the garden.
    pub(crate) partner_id: Option<i64>,
    /// One garden.
    pub(crate) garden_id: Option<i64>,
    /// Only open or only closed gardens.
    pub(crate) status: Option<String>,
}

/// The commission on the goods of a garden, by kind, summed over its whole life with the sign of
/// the facts (a cancelled sale takes itself out).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Commission {
    pub(crate) seller: Paisa,
    pub(crate) buyer: Paisa,
    pub(crate) laga: Paisa,
}

impl Commission {
    /// The seller side only, the buyer side only (commission and لاگا), or both.
    pub(crate) fn total(&self, seller: bool, buyer: bool) -> Result<Paisa, ReportsError> {
        let mut total = Paisa::ZERO;
        if seller {
            add(&mut total, self.seller)?;
        }
        if buyer {
            add(&mut total, self.buyer)?;
            add(&mut total, self.laga)?;
        }
        Ok(total)
    }
}

/// One partner of a garden.
#[derive(Debug, Clone)]
pub(crate) struct PartnerFigure {
    pub(crate) partner_id: i64,
    pub(crate) name_ur: String,
    pub(crate) name_en: String,
    pub(crate) share_bps: i64,
    pub(crate) paid_shop: Paisa,
    pub(crate) paid_pocket: Paisa,
    /// His stored part of the closing. Negative in a loss closing, 0 while the garden is open.
    pub(crate) back: Paisa,
    /// Back less paid from the shop and from his pocket.
    pub(crate) net: Paisa,
}

/// One garden with everything a report shows.
#[derive(Debug, Clone)]
pub(crate) struct Garden {
    pub(crate) rec: GardenRec,
    pub(crate) money: Money,
    /// Sales in our shop: the net of the lines of بکری on its account.
    pub(crate) our_shop: Paisa,
    /// Money in that is neither: fruit taken from its lots.
    pub(crate) other_in: Paisa,
    pub(crate) commission: Commission,
    pub(crate) partners: Vec<PartnerFigure>,
}

fn data(message: &str) -> ReportsError {
    ReportsError::Data(message.to_owned())
}

fn money_error(error: MoneyDataError) -> ReportsError {
    match error {
        MoneyDataError::Money(error) => ReportsError::Money(error),
        MoneyDataError::PartsDoNotAdd => {
            data("the parts of the payments of a garden do not add up")
        }
    }
}

/// The ids of the gardens a pick finds, in the order they were made (the same set the garden list
/// shows for the same filter).
///
/// # Errors
/// A database error.
pub(crate) fn pick_ids(conn: &Connection, pick: &Pick) -> Result<Vec<i64>, ReportsError> {
    let filter = ListFilter {
        order_by: None,
        kind: "garden".to_owned(),
        from: pick.from.clone(),
        to: pick.to.clone(),
        status: pick.status.clone(),
        partner_id: pick.partner_id,
        search: None,
    };
    let mut ids = vrepo::ids_of(conn, &filter)?;
    if let Some(id) = pick.garden_id {
        ids.retain(|g| *g == id);
    }
    Ok(ids)
}

/// The garden records a pick finds, in the order they were made.
///
/// # Errors
/// A database error.
pub(crate) fn pick_recs(conn: &Connection, pick: &Pick) -> Result<Vec<GardenRec>, ReportsError> {
    let ids = pick_ids(conn, pick)?;
    let mut recs = repo::get_many(conn, &ids)?;
    if let Some(item) = pick.item_id {
        recs.retain(|r| r.item.as_ref().is_some_and(|i| i.id == item));
    }
    recs.sort_by_key(|r| r.id);
    Ok(recs)
}

/// Reads the gardens a pick finds.
///
/// # Errors
/// A database error; `ReportsError::Money`; `ReportsError::Data` if the parts of a payment do not
/// add up, if the sources of the money in do not add up to it, or if a closing is missing a part.
pub(crate) fn load(conn: &Connection, pick: &Pick) -> Result<Vec<Garden>, ReportsError> {
    let recs = pick_recs(conn, pick)?;
    let ids: Vec<i64> = recs.iter().map(|r| r.id).collect();
    let accounts: Vec<i64> = recs.iter().map(|r| r.account.id).collect();

    let sources = sources_of(conn, &accounts)?;
    let commissions = commissions_of(conn, &accounts)?;
    let shares = vrepo::partners_of_many(conn, &ids)?;
    let paid = paid_by_partner_many(conn, &ids)?;
    let back = closing_parts(conn, &ids)?;

    let mut out = Vec::with_capacity(recs.len());
    for rec in recs {
        let figures = repo::figures(conn, &rec)?;
        let money = Money::of(rec.rent_paisa, &figures).map_err(money_error)?;
        let (our_shop, other_in) = split_money_in(&rec, &money, sources.get(&rec.account.id))?;
        let partners = partners_of(
            shares.get(&rec.id).map(Vec::as_slice).unwrap_or_default(),
            &paid,
            &back,
            rec.id,
        )?;
        check_partners(&rec, &money, &partners)?;
        out.push(Garden {
            commission: commissions
                .get(&rec.account.id)
                .copied()
                .unwrap_or_default(),
            rec,
            money,
            our_shop,
            other_in,
            partners,
        });
    }
    Ok(out)
}

/// The partners' figures must add up to the garden's: what they paid is what was paid, and once the
/// garden is closed what came back is the money in, so the results add up to the balance (C26).
/// A part of a partner who is not in the garden's list would be dropped silently otherwise.
fn check_partners(
    rec: &GardenRec,
    money: &Money,
    partners: &[PartnerFigure],
) -> Result<(), ReportsError> {
    let (mut paid, mut back) = (Paisa::ZERO, Paisa::ZERO);
    for p in partners {
        add(&mut paid, p.paid_shop)?;
        add(&mut paid, p.paid_pocket)?;
        add(&mut back, p.back)?;
    }
    if paid != money.paid {
        return Err(data(
            "what the partners of a garden paid is not what was paid into it",
        ));
    }
    if rec.status == "closed" && back != money.money_in {
        return Err(data(
            "what the partners of a closed garden got back is not its money in",
        ));
    }
    Ok(())
}

/// Splits the money in by where it came from and checks that the parts add up to it.
fn split_money_in(
    rec: &GardenRec,
    money: &Money,
    sources: Option<&Vec<(String, i64, i64)>>,
) -> Result<(Paisa, Paisa), ReportsError> {
    let (mut our_shop, mut other_shops, mut other_in) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    for (source, credits, debits) in sources.map(Vec::as_slice).unwrap_or_default() {
        let net = paisa(*credits)?.checked_sub(paisa(*debits)?)?;
        match source.as_str() {
            "bikri" => add(&mut our_shop, net)?,
            "venture" => add(&mut other_shops, net)?,
            "fruit" => add(&mut other_in, net)?,
            _ => {
                return Err(data(&format!(
                    "the account of garden {} has a line from an unknown source",
                    rec.number
                )));
            }
        }
    }
    if other_shops != money.other_shop {
        return Err(data(
            "the money from other shops on a garden account is not the sum of its saved sales",
        ));
    }
    let mut whole = our_shop;
    add(&mut whole, other_shops)?;
    add(&mut whole, other_in)?;
    if whole != money.money_in {
        return Err(data(
            "the money in of a garden does not add up from its sources",
        ));
    }
    Ok((our_shop, other_in))
}

/// For each garden account: `(source type, Σ credits, Σ debits)`.
type Sources = HashMap<i64, Vec<(String, i64, i64)>>;

fn sources_of(conn: &Connection, accounts: &[i64]) -> Result<Sources, ReportsError> {
    let mut out: Sources = HashMap::new();
    for (account, source, credits, debits) in money_in_by_source(conn, accounts)? {
        out.entry(account)
            .or_default()
            .push((source, credits, debits));
    }
    Ok(out)
}

/// The commission on the goods of each garden account, over its whole life: one query, the facts
/// with sign +1 and with sign −1 summed apart and subtracted in Rust (Code Standards M5, B20).
fn commissions_of(
    conn: &Connection,
    accounts: &[i64],
) -> Result<HashMap<i64, Commission>, ReportsError> {
    let mut out = HashMap::new();
    for chunk in accounts.chunks(IN_CHUNK) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&format!(
            "SELECT f.seller_account_id,
                    COALESCE(SUM(CASE WHEN f.sign > 0 THEN f.seller_comm_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign < 0 THEN f.seller_comm_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign > 0 THEN f.buyer_comm_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign < 0 THEN f.buyer_comm_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign > 0 THEN f.laga_paisa END), 0),
                    COALESCE(SUM(CASE WHEN f.sign < 0 THEN f.laga_paisa END), 0)
             FROM analytics_facts f WHERE f.seller_account_id IN ({marks})
             GROUP BY f.seller_account_id"
        ))?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            let mut sums = [0_i64; 6];
            for (index, slot) in sums.iter_mut().enumerate() {
                *slot = r.get(index + 1)?;
            }
            Ok((r.get::<_, i64>(0)?, sums))
        })?;
        for row in rows {
            let (account, sums) = row?;
            let net = |plus: i64, minus: i64| -> Result<Paisa, ReportsError> {
                Ok(paisa(plus)?.checked_sub(paisa(minus)?)?)
            };
            let [sp, sm, bp, bm, lp, lm] = sums;
            out.insert(
                account,
                Commission {
                    seller: net(sp, sm)?,
                    buyer: net(bp, bm)?,
                    laga: net(lp, lm)?,
                },
            );
        }
    }
    Ok(out)
}

/// For each (garden, partner): `(paid from the shop, paid from his pocket)`.
type PaidParts = HashMap<(i64, i64), (i64, i64)>;

/// For each garden and partner: what he paid from the shop and from his pocket, from the stored
/// parts of the live installments and costs.
fn paid_by_partner_many(conn: &Connection, garden_ids: &[i64]) -> Result<PaidParts, ReportsError> {
    let mut out = HashMap::new();
    for chunk in garden_ids.chunks(IN_CHUNK) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&format!(
            "SELECT x.venture_id, p.partner_id,
                    COALESCE(SUM(CASE WHEN p.source = 'shop' THEN p.amount_paisa END), 0),
                    COALESCE(SUM(CASE WHEN p.source = 'pocket' THEN p.amount_paisa END), 0)
             FROM venture_entry_parts p JOIN venture_entries x ON x.id = p.entry_id
             WHERE x.venture_id IN ({marks}) AND x.status = 'saved'
               AND x.kind IN ('installment', 'cost')
             GROUP BY x.venture_id, p.partner_id"
        ))?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                (r.get::<_, i64>(0)?, r.get::<_, i64>(1)?),
                (r.get::<_, i64>(2)?, r.get::<_, i64>(3)?),
            ))
        })?;
        for row in rows {
            let (key, value) = row?;
            out.insert(key, value);
        }
    }
    Ok(out)
}

/// For each garden and partner: his stored part of the saved closing. Read only from the saved
/// closing row of each garden, never by kind alone, so a closing that was undone and made again
/// counts once.
fn closing_parts(
    conn: &Connection,
    garden_ids: &[i64],
) -> Result<HashMap<(i64, i64), i64>, ReportsError> {
    let mut entry_of: HashMap<i64, i64> = HashMap::new();
    for chunk in garden_ids.chunks(IN_CHUNK) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&format!(
            "SELECT venture_id, MAX(id) FROM venture_entries
             WHERE venture_id IN ({marks}) AND kind = 'close' AND status = 'saved'
             GROUP BY venture_id"
        ))?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (garden, entry) = row?;
            entry_of.insert(entry, garden);
        }
    }
    let entries: Vec<i64> = entry_of.keys().copied().collect();
    let mut out = HashMap::new();
    for part in vrepo::parts_of(conn, &entries)? {
        if let Some(garden) = entry_of.get(&part.entry_id) {
            out.insert((*garden, part.partner.id), part.amount_paisa);
        }
    }
    Ok(out)
}

fn partners_of(
    shares: &[PartnerShare],
    paid: &PaidParts,
    back: &HashMap<(i64, i64), i64>,
    garden_id: i64,
) -> Result<Vec<PartnerFigure>, ReportsError> {
    let mut out = Vec::with_capacity(shares.len());
    for share in shares {
        let (shop, pocket) = paid
            .get(&(garden_id, share.partner_id))
            .copied()
            .unwrap_or((0, 0));
        let money_back = back
            .get(&(garden_id, share.partner_id))
            .copied()
            .unwrap_or(0);
        let (shop, pocket, money_back) = (paisa(shop)?, paisa(pocket)?, paisa(money_back)?);
        let net = money_back.checked_sub(shop)?.checked_sub(pocket)?;
        out.push(PartnerFigure {
            partner_id: share.partner_id,
            name_ur: share.name_ur.clone(),
            name_en: share.name_en.clone(),
            share_bps: share.share_bps,
            paid_shop: shop,
            paid_pocket: pocket,
            back: money_back,
            net,
        });
    }
    Ok(out)
}

/// Sums of the open gardens for the dashboard tile and the list under it: the number of open
/// gardens, the total of their running balances, and for each its balance and rent due. The same
/// figures as `services::gardens::summary` (one `Money::of`).
#[derive(Debug, Clone)]
pub(crate) struct OpenGarden {
    pub(crate) id: i64,
    pub(crate) number: i64,
    pub(crate) name_ur: String,
    pub(crate) name_en: String,
    pub(crate) balance: Paisa,
    pub(crate) rent_due: Paisa,
}

/// The open gardens, in the order they were made.
///
/// # Errors
/// A database error; `ReportsError::Money`; `ReportsError::Data`.
pub(crate) fn open_gardens(conn: &Connection) -> Result<Vec<OpenGarden>, ReportsError> {
    let filter = ListFilter {
        kind: "garden".to_owned(),
        status: Some("open".to_owned()),
        ..ListFilter::default()
    };
    let ids = vrepo::ids_of(conn, &filter)?;
    let mut recs = repo::get_many(conn, &ids)?;
    recs.sort_by_key(|r| r.number);
    let mut out = Vec::with_capacity(recs.len());
    for rec in recs {
        let figures = repo::figures(conn, &rec)?;
        let money = Money::of(rec.rent_paisa, &figures).map_err(money_error)?;
        out.push(OpenGarden {
            id: rec.id,
            number: rec.number,
            name_ur: rec.name_ur,
            name_en: rec.name_en,
            balance: money.balance,
            rent_due: money.rent_due,
        });
    }
    Ok(out)
}
