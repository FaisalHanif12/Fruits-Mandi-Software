//! R20 وصولی / اگراہی رپورٹ (buyer dues), R21 اگراہی بحساب شہر (by city) and R22 اگراہی بحساب
//! اگراہی والا (by collector) (PRD 19.2, FR-15.6, FR-19.9, FR-19.10).
//!
//! One row for each buyer who owes the firm (and each چکتا buyer inside the چکتا کھاتہ
//! account, PRD FR-13.5): his balance, his phone, the date and age of his oldest open bill and
//! his open bills in four age bands (0 to 7, 8 to 30, 31 to 90, 91 and more days).
//!
//! - **Balance.** The posted `Σ بنام − Σ جمع` of the buyer up to the "as of" day, from the same
//!   grouped totals as the trial balance (`mandi_db::ledger::account_totals`). Only a debit
//!   balance is listed: a buyer who has paid in advance owes nothing. The total row is the
//!   shop's total اگراہی and equals the dashboard tile (PRD FR-19.9).
//! - **Open bills.** The debit lines of the buyer that the active rows of `bill_allocations` do
//!   not settle in full (PRD FR-15.4). Each open amount is the line less what is matched, worked
//!   out here with checked maths. The matching rows are kept as they stand now and are not dated,
//!   so the bills and their ages can only be given for a report as of today or later. For an
//!   earlier day only the balance is given and the age cells are empty (nothing is guessed).
//! - **Grouping.** R21 groups the rows under the buyer's city and R22 under his collector (a
//!   staff member). Each group has a heading row with the group's total; the heading rows are
//!   counted in `total_count` and are part of the paging, like the heads of R17.
//!
//! Example (the worked day, after Events 1 to 5): اکرم 2,880 (oldest bill 10-10-26, 0 days, band
//! 0 to 7: 2,880) and ارشد 18,684; the total row 21,564. After E2E-32 (3,000 cash lent to اکرم)
//! the total row is 24,564.

use std::collections::{BTreeMap, HashMap};

use mandi_core::accounts::BUYER_CONTROL;
use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_core::text::phone_display;
use mandi_db::ledger::{Totals, account_totals_in_controls, totals_of};
use mandi_db::rusqlite::{Connection, params};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{
    add, amount_or_empty, col, count_of, date_cell, load_parties, paisa, text_cell,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, Names, PartyRef, ReportId, ReportPage, Row, RowKind, RowSource,
    SourceKind,
};

/// How the rows are arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layout {
    /// R20: one flat list.
    Flat,
    /// R21: grouped under the city.
    ByCity,
    /// R22: grouped under the collector.
    ByCollector,
}

impl Layout {
    fn id(self) -> ReportId {
        match self {
            Layout::Flat => ReportId::R20,
            Layout::ByCity => ReportId::R21,
            Layout::ByCollector => ReportId::R22,
        }
    }
}

/// The age bands of the open bills, in days (PRD R20): 0 to 7, 8 to 30, 31 to 90, 91 and more.
const BAND_ENDS: [i64; 3] = [7, 30, 90];

/// One buyer (or one چکتا buyer) who owes.
pub(crate) struct Buyer {
    pub(crate) party: PartyRef,
    pub(crate) city_id: Option<i64>,
    collector_id: Option<i64>,
    phone: Option<String>,
    pub(crate) balance: Paisa,
    /// The date of the oldest open bill and the open amounts in the four bands. `None` when the
    /// report is for an earlier day.
    pub(crate) ages: Option<Ages>,
}

/// The ages of a buyer's open bills.
pub(crate) struct Ages {
    /// The day of the oldest open bill and its age in days.
    pub(crate) oldest: Option<(String, i64)>,
    /// The open amounts in the four age bands (0 to 7, 8 to 30, 31 to 90, 91 and more days).
    pub(crate) bands: [Paisa; 4],
}

/// The buyer accounts. Parameters: city, account, collector (R22 only: the buyers he collects
/// from, by the collector on the account, plan 55 §11.1).
fn buyers_sql() -> String {
    format!(
        "SELECT a.id, a.city_id, a.collector_staff_id, a.phone1
    FROM accounts a JOIN account_controls k ON k.id = a.control_id
    -- چکتا کھاتہ has no ledger and no dues (PRD FR-13.6); an old open amount on it stays in the
    -- books and does not show here (PRD FR-13.9).
    WHERE k.code = '0304' AND a.is_chukta_khata = 0 AND (?1 IS NULL OR a.city_id = ?1) AND (?2 IS NULL OR a.id = ?2)
        AND (?3 IS NULL OR a.collector_staff_id = ?3)
    ORDER BY {}",
        mandi_db::codes::order_by_code("a.code")
    )
}

struct Account {
    id: i64,
    city_id: Option<i64>,
    collector_id: Option<i64>,
    phone: Option<String>,
}

fn totals_pair(totals: Totals) -> Result<(Paisa, Paisa), ReportsError> {
    Ok((paisa(totals.dr_paisa)?, paisa(totals.cr_paisa)?))
}

/// The buyers with a debit balance, in code order, چکتا buyers after their account.
fn owing(
    conn: &Connection,
    typed: &Typed,
    parties: &HashMap<i64, PartyRef>,
    accounts: &[Account],
) -> Result<Vec<Buyer>, ReportsError> {
    let as_of = typed.as_of.as_deref();
    // People in the dead amount are not in the dues: they are in R38 and the tile ڈوبی رقم (PRD
    // FR-10.10). For a past day, only those moved by that day are left out.
    let dead = mandi_db::dead::dead_set(conn, as_of)?;
    // The buyers only (plan 48 P48-S3): a third of the journal, not all of it.
    let all: HashMap<i64, Totals> = if typed.party_id.is_some() {
        HashMap::new()
    } else {
        account_totals_in_controls(conn, &[BUYER_CONTROL], as_of)?
            .into_iter()
            .collect()
    };
    let mut found = Vec::new();
    for account in accounts {
        let party = parties.get(&account.id).cloned().ok_or_else(|| {
            ReportsError::Data(format!("buyer account {} is missing", account.id))
        })?;
        if dead.has_account(account.id) {
            continue;
        }
        let totals = if typed.party_id.is_some() {
            totals_of(conn, account.id, None, as_of)?
        } else {
            all.get(&account.id).copied().unwrap_or(Totals {
                dr_paisa: 0,
                cr_paisa: 0,
            })
        };
        let (dr, cr) = totals_pair(totals)?;
        let balance = dr.checked_sub(cr)?;
        if balance <= Paisa::ZERO {
            continue;
        }
        found.push(Buyer {
            party,
            city_id: account.city_id,
            collector_id: account.collector_id,
            phone: account.phone.clone(),
            balance,
            ages: None,
        });
    }
    Ok(found)
}

/// The band of a bill that is `days` old: 0 for 0 to 7 days up to 3 for 91 and more.
fn band_of(days: i64) -> usize {
    BAND_ENDS
        .iter()
        .position(|end| days <= *end)
        .unwrap_or(BAND_ENDS.len())
}

/// Puts the ages of the open bills on the buyers.
///
/// The open bills come from `mandi_db::party_reads::open_buyer_bills` (plan 48 P48-S3). The report
/// uses two things of a buyer's bills: the open amount in each age band, and the day of the oldest
/// one. Both are worked out here in one pass, whatever the order of the bills: the bands are sums
/// and the oldest bill is the one with the smallest day.
fn put_ages(
    conn: &Connection,
    today: BusinessDate,
    buyers: &mut [Buyer],
) -> Result<(), ReportsError> {
    let ids: Vec<i64> = buyers.iter().map(|b| b.party.account_id).collect();
    let mut unique = ids.clone();
    unique.sort_unstable();
    unique.dedup();
    let bills = mandi_db::party_reads::open_buyer_bills(conn, &unique)?;
    let mut by_key: HashMap<(i64, Option<String>), Ages> = HashMap::new();
    for bill in &bills {
        let day = BusinessDate::parse_iso(&bill.entry_date)
            .ok_or_else(|| ReportsError::Data(format!("entry date {}", bill.entry_date)))?;
        let days = today.days_after(day).max(0);
        let open = paisa(bill.open_paisa)?;
        let ages = by_key
            .entry((bill.account_id, bill.person_key.clone()))
            .or_insert_with(|| Ages {
                oldest: None,
                bands: [Paisa::ZERO; 4],
            });
        if let Some(slot) = ages.bands.get_mut(band_of(days)) {
            add(slot, open)?;
        }
        if ages
            .oldest
            .as_ref()
            .is_none_or(|(oldest, _)| bill.entry_date < *oldest)
        {
            ages.oldest = Some((bill.entry_date.clone(), days));
        }
    }
    for buyer in buyers.iter_mut() {
        let key = (buyer.party.account_id, buyer.party.person_key.clone());
        buyer.ages = Some(by_key.remove(&key).unwrap_or(Ages {
            oldest: None,
            bands: [Paisa::ZERO; 4],
        }));
    }
    Ok(())
}

fn band_cells(ages: Option<&Ages>) -> Vec<Cell> {
    match ages {
        Some(ages) => ages.bands.iter().map(|b| amount_or_empty(*b)).collect(),
        None => vec![Cell::Empty; 4],
    }
}

/// The cells of one buyer: the party, optionally his city, his phone, the balance, the oldest
/// bill and the four bands.
fn buyer_cells(buyer: &Buyer, city: Option<&Cell>) -> Vec<Cell> {
    let mut cells = vec![Cell::Party {
        party: buyer.party.clone(),
    }];
    if let Some(city) = city {
        cells.push(city.clone());
    }
    cells.push(text_cell(buyer.phone.as_deref().map(phone_display)));
    cells.push(Cell::Money {
        paisa: buyer.balance,
    });
    let (date, days) = match buyer.ages.as_ref().and_then(|a| a.oldest.as_ref()) {
        Some((date, days)) => (date_cell(Some(date.clone())), Cell::Number { value: *days }),
        None => (Cell::Empty, Cell::Empty),
    };
    cells.push(date);
    cells.push(days);
    cells.extend(band_cells(buyer.ages.as_ref()));
    cells
}

fn age_columns() -> Vec<Column> {
    vec![
        col("oldest_date", ids::COL_OLDEST_DATE, ColumnKind::Date),
        col("days", ids::COL_DAYS, ColumnKind::Number),
        col("age_0_7", ids::COL_AGE_0_7, ColumnKind::Money),
        col("age_8_30", ids::COL_AGE_8_30, ColumnKind::Money),
        col("age_31_90", ids::COL_AGE_31_90, ColumnKind::Money),
        col("age_91", ids::COL_AGE_91, ColumnKind::Money),
    ]
}

/// The columns of R20, R21 or R22: the leading columns of the layout, then the phone, the balance
/// and the age columns. The one definition: the run and the catalogue both use it (plan 75 T1).
pub(crate) fn columns(layout: Layout) -> Vec<Column> {
    let mut columns = match layout {
        Layout::Flat => vec![
            col("buyer", ids::COL_BUYER, ColumnKind::Party),
            col("city", ids::COL_CITY, ColumnKind::Names),
        ],
        Layout::ByCity => vec![
            col("group", ids::COL_CITY, ColumnKind::Names),
            col("buyer", ids::COL_BUYER, ColumnKind::Party),
        ],
        Layout::ByCollector => vec![
            col("group", ids::COL_COLLECTOR, ColumnKind::Names),
            col("buyer", ids::COL_BUYER, ColumnKind::Party),
            col("city", ids::COL_CITY, ColumnKind::Names),
        ],
    };
    columns.push(col("phone", ids::COL_PHONE, ColumnKind::Text));
    columns.push(col("balance", ids::COL_BALANCE, ColumnKind::Money));
    columns.extend(age_columns());
    columns
}

/// A master row (city or staff): id, code and names, to order and name the groups.
struct Group {
    code: String,
    names: Names,
}

fn cities(conn: &Connection) -> Result<BTreeMap<i64, Group>, ReportsError> {
    let mut statement = conn.prepare("SELECT id, code, name_ur, name_en FROM cities")?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            Group {
                code: r.get(1)?,
                names: Names {
                    ur: r.get(2)?,
                    en: r.get(3)?,
                },
            },
        ))
    })?;
    Ok(rows.collect::<Result<BTreeMap<_, _>, _>>()?)
}

fn collectors(conn: &Connection) -> Result<BTreeMap<i64, Group>, ReportsError> {
    let mut statement = conn.prepare("SELECT id, name_ur, name_en FROM staff")?;
    let rows = statement.query_map([], |r| {
        let id: i64 = r.get(0)?;
        Ok((
            id,
            Group {
                // A staff member has no code; his id keeps the groups in the order he was made.
                code: format!("{id:012}"),
                names: Names {
                    ur: r.get(1)?,
                    en: r.get(2)?,
                },
            },
        ))
    })?;
    Ok(rows.collect::<Result<BTreeMap<_, _>, _>>()?)
}

/// Builds R20, R21 or R22.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total
/// does not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    layout: Layout,
) -> Result<ReportPage, ReportsError> {
    let (buyers, as_of_is_now) = load_buyers(conn, typed)?;
    page_of(conn, typed, spec, layout, buyers, as_of_is_now)
}

/// The buyers who owe the firm today, each with the ages of his open bills: exactly the rows of
/// R20 for no filter. The dashboard reads its tile, its list of the oldest dues and the attention
/// item for old dues from these rows, so they can never differ from R20 (PRD FR-19.9).
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn buyers_owing_now(
    conn: &Connection,
    today: BusinessDate,
) -> Result<Vec<Buyer>, ReportsError> {
    let typed = Typed {
        today: Some(today),
        ..Typed::default()
    };
    Ok(load_buyers(conn, &typed)?.0)
}

/// The buyers who owe the firm today WITHOUT the ages of their open bills (plan 48 P48-S1): the
/// first half of [`load_buyers`], so the set and every balance are exactly the ones of R20 (the
/// ages never change a balance or drop a buyer). Reading the ages takes 3.6 seconds on the
/// NFR-SCL-1 volume and the balances 0.3 seconds. The dashboard's narrow tiles and the second
/// place of the assistant's buyers' dues read these rows. The `ages` of every buyer are `None`,
/// so these rows must never be given to what draws ages (the old-dues list, the bell).
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn buyers_balances_now(
    conn: &Connection,
    today: BusinessDate,
) -> Result<Vec<Buyer>, ReportsError> {
    let typed = Typed {
        today: Some(today),
        ..Typed::default()
    };
    balances_owing(conn, &typed)
}

/// The buyers who owe under these filters, with their balances only: the accounts of the buyer
/// control, their parties, and the debit balances (`owing`).
fn balances_owing(conn: &Connection, typed: &Typed) -> Result<Vec<Buyer>, ReportsError> {
    let accounts = {
        let mut statement = conn.prepare(&buyers_sql())?;
        statement
            .query_map(
                params![typed.city_id, typed.party_id, typed.collector_id],
                |r| {
                    Ok(Account {
                        id: r.get(0)?,
                        city_id: r.get(1)?,
                        collector_id: r.get(2)?,
                        phone: r.get(3)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?
    };
    let account_ids: Vec<i64> = accounts.iter().map(|a| a.id).collect();
    let parties = load_parties(conn, &account_ids)?;
    owing(conn, typed, &parties, &accounts)
}

/// R20, R21 or R22 with the ages left out (plan 48 P48-S1): the same buyers, the same balances and
/// the same total row, the age cells empty. The page is built by the very `page_of` of the full
/// report, so its balance total is the report's own.
///
/// # Errors
/// As [`run`].
pub(crate) fn run_without_ages(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    layout: Layout,
) -> Result<ReportPage, ReportsError> {
    let buyers = balances_owing(conn, typed)?;
    page_of(conn, typed, spec, layout, buyers, false)
}

/// The buyers who owe under these filters, with their ages when the day is today or later, and
/// whether the ages were worked out.
fn load_buyers(conn: &Connection, typed: &Typed) -> Result<(Vec<Buyer>, bool), ReportsError> {
    let mut buyers = balances_owing(conn, typed)?;

    // The ages are known only for today or later: the matching rows are not dated.
    let today = typed
        .today
        .ok_or_else(|| ReportsError::Data("the report has no today".to_owned()))?;
    let as_of_is_now = typed
        .as_of
        .as_deref()
        .is_none_or(|as_of| as_of >= today.to_iso().as_str());
    if as_of_is_now {
        put_ages(conn, today, &mut buyers)?;
    }
    Ok((buyers, as_of_is_now))
}

/// The page of R20, R21 or R22 from the buyers who owe.
fn page_of(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    layout: Layout,
    buyers: Vec<Buyer>,
    as_of_is_now: bool,
) -> Result<ReportPage, ReportsError> {
    // Totals over the whole filter.
    let mut total_balance = Paisa::ZERO;
    let mut total_bands = [Paisa::ZERO; 4];
    for buyer in &buyers {
        add(&mut total_balance, buyer.balance)?;
        if let Some(ages) = &buyer.ages {
            for (slot, band) in total_bands.iter_mut().zip(ages.bands) {
                add(slot, band)?;
            }
        }
    }

    let city_names = cities(conn)?;
    let all_rows = match layout {
        Layout::Flat => flat_rows(&buyers, &city_names)?,
        Layout::ByCity => grouped_rows(&buyers, &city_names, |b| b.city_id, false, &city_names)?,
        Layout::ByCollector => {
            let staff = collectors(conn)?;
            grouped_rows(&buyers, &staff, |b| b.collector_id, true, &city_names)?
        }
    };

    let columns = columns(layout);
    let leading = columns.len().saturating_sub(7);
    let mut totals = vec![Cell::Empty; leading];
    totals.push(Cell::Money {
        paisa: total_balance,
    });
    totals.push(Cell::Empty);
    totals.push(Cell::Empty);
    if as_of_is_now {
        totals.extend(total_bands.iter().map(|b| amount_or_empty(*b)));
    } else {
        totals.extend(vec![Cell::Empty; 4]);
    }

    let id = layout.id();
    Ok(ReportPage {
        group_above: None,
        tables: Vec::new(),
        id,
        title_id: ids::title_id(id.code()),
        filters: typed.resolved.clone(),
        columns,
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals,
        summary: Vec::new(),
    })
}

fn source_of(buyer: &Buyer) -> Option<RowSource> {
    Some(RowSource {
        kind: SourceKind::Account,
        id: buyer.party.account_id,
    })
}

fn city_cell(buyer: &Buyer, cities: &BTreeMap<i64, Group>) -> Cell {
    buyer
        .city_id
        .and_then(|id| cities.get(&id))
        .map_or(Cell::Empty, |g| Cell::Names {
            names: g.names.clone(),
        })
}

fn flat_rows(buyers: &[Buyer], cities: &BTreeMap<i64, Group>) -> Result<Vec<Row>, ReportsError> {
    Ok(buyers
        .iter()
        .map(|buyer| Row {
            cells: buyer_cells(buyer, Some(&city_cell(buyer, cities))),
            source: source_of(buyer),
            kind: RowKind::Normal,
        })
        .collect())
}

/// The rows under group headings. A buyer with no group goes last, under an empty heading. The
/// heading row holds the group's total balance and bands.
fn grouped_rows(
    buyers: &[Buyer],
    groups: &BTreeMap<i64, Group>,
    key_of: impl Fn(&Buyer) -> Option<i64>,
    with_city: bool,
    cities: &BTreeMap<i64, Group>,
) -> Result<Vec<Row>, ReportsError> {
    let mut by_group: BTreeMap<(bool, String, i64), Vec<&Buyer>> = BTreeMap::new();
    for buyer in buyers {
        let key = match key_of(buyer) {
            Some(id) => match groups.get(&id) {
                Some(g) => (false, g.code.clone(), id),
                None => (true, String::new(), 0),
            },
            None => (true, String::new(), 0),
        };
        by_group.entry(key).or_default().push(buyer);
    }
    let mut rows = Vec::new();
    for ((_, _, id), members) in by_group {
        let mut balance = Paisa::ZERO;
        let mut bands = [Paisa::ZERO; 4];
        let mut has_ages = false;
        for buyer in &members {
            add(&mut balance, buyer.balance)?;
            if let Some(ages) = &buyer.ages {
                has_ages = true;
                for (slot, band) in bands.iter_mut().zip(ages.bands) {
                    add(slot, band)?;
                }
            }
        }
        let heading = groups.get(&id).map_or(Cell::Empty, |g| Cell::Names {
            names: g.names.clone(),
        });
        // Heading: group, buyer, (city,) phone, balance, oldest, days, bands.
        let mut cells = vec![heading, Cell::Empty];
        if with_city {
            cells.push(Cell::Empty);
        }
        cells.push(Cell::Empty);
        cells.push(Cell::Money { paisa: balance });
        cells.push(Cell::Empty);
        cells.push(Cell::Empty);
        if has_ages {
            cells.extend(bands.iter().map(|b| amount_or_empty(*b)));
        } else {
            cells.extend(vec![Cell::Empty; 4]);
        }
        rows.push(Row {
            cells,
            source: None,
            kind: RowKind::Group,
        });
        for buyer in members {
            let city = with_city.then(|| city_cell(buyer, cities));
            let mut cells = vec![Cell::Empty];
            cells.extend(buyer_cells(buyer, city.as_ref()));
            rows.push(Row {
                cells,
                source: source_of(buyer),
                kind: RowKind::Normal,
            });
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_age_bands_are_0_to_7_8_to_30_31_to_90_and_91_up() {
        assert_eq!(band_of(0), 0);
        assert_eq!(band_of(7), 0);
        assert_eq!(band_of(8), 1);
        assert_eq!(band_of(30), 1);
        assert_eq!(band_of(31), 2);
        assert_eq!(band_of(90), 2);
        assert_eq!(band_of(91), 3);
        assert_eq!(band_of(4000), 3);
    }
}
