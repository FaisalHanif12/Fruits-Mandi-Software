//! R20 وصولی / اگراہی رپورٹ (buyer dues), R21 اگراہی بحساب شہر (by city) and R22 اگراہی بحساب
//! اگراہی والا (by collector) (PRD 19.2, FR-15.6, FR-19.9, FR-19.10).
//!
//! One row for each buyer who owes the firm (and each one-time person inside the پختہ کھاتہ
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

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::ledger::{Totals, account_totals, totals_of};
use mandi_db::rusqlite::{Connection, params, params_from_iter};

use super::super::PageSpec;
use super::grouped::pukhta_names;
use crate::error::ReportsError;
use crate::reports::common::{
    IN_CHUNK, LIVE, add, amount_or_empty, col, count_of, date_cell, load_parties, paisa,
    placeholders, pukhta_party, text_cell,
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

/// One buyer (or one one-time person) who owes.
struct Buyer {
    party: PartyRef,
    city_id: Option<i64>,
    collector_id: Option<i64>,
    phone: Option<String>,
    balance: Paisa,
    /// The date of the oldest open bill and the open amounts in the four bands. `None` when the
    /// report is for an earlier day.
    ages: Option<Ages>,
}

struct Ages {
    oldest: Option<(String, i64)>,
    bands: [Paisa; 4],
}

/// The buyer accounts. Parameters: city, account.
const BUYERS_SQL: &str = "SELECT a.id, a.is_pukhta_khata, a.city_id, a.collector_staff_id,
        a.phone1
    FROM accounts a JOIN account_controls k ON k.id = a.control_id
    WHERE k.code = '0304' AND (?1 IS NULL OR a.city_id = ?1) AND (?2 IS NULL OR a.id = ?2)
    ORDER BY a.code";

/// The balances of the persons of a پختہ کھاتہ account. Parameters: account, as-of day.
const PERSONS_SQL: &str = "SELECT l.pukhta_person_key, COALESCE(SUM(l.dr_paisa), 0),
        COALESCE(SUM(l.cr_paisa), 0)
    FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
    WHERE l.account_id = ?1 AND e.status = 'posted' AND (?2 IS NULL OR e.entry_date <= ?2)
    GROUP BY l.pukhta_person_key ORDER BY l.pukhta_person_key";

struct Account {
    id: i64,
    pukhta: bool,
    city_id: Option<i64>,
    collector_id: Option<i64>,
    phone: Option<String>,
}

fn totals_pair(totals: Totals) -> Result<(Paisa, Paisa), ReportsError> {
    Ok((paisa(totals.dr_paisa)?, paisa(totals.cr_paisa)?))
}

/// The buyers with a debit balance, in code order, one-time persons after their account.
fn owing(
    conn: &Connection,
    typed: &Typed,
    parties: &HashMap<i64, PartyRef>,
    accounts: &[Account],
) -> Result<Vec<Buyer>, ReportsError> {
    let as_of = typed.as_of.as_deref();
    let all: HashMap<i64, Totals> = if typed.party_id.is_some() {
        HashMap::new()
    } else {
        account_totals(conn, as_of)?.into_iter().collect()
    };
    // The names of the one-time people, from one query and not one for each person (B22).
    let pukhta_ids: Vec<i64> = accounts.iter().filter(|a| a.pukhta).map(|a| a.id).collect();
    let names = pukhta_names(conn, &pukhta_ids)?;
    let mut found = Vec::new();
    for account in accounts {
        let party = parties.get(&account.id).cloned().ok_or_else(|| {
            ReportsError::Data(format!("buyer account {} is missing", account.id))
        })?;
        if account.pukhta {
            let mut statement = conn.prepare(PERSONS_SQL)?;
            let persons = statement
                .query_map(params![account.id, as_of], |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            for (key, dr, cr) in persons {
                if typed.person_key.is_some() && typed.person_key != key {
                    continue;
                }
                let balance = paisa(dr)?.checked_sub(paisa(cr)?)?;
                if balance <= Paisa::ZERO {
                    continue;
                }
                let (name_ur, name_en, phone) = match key.as_ref().and_then(|k| names.get(k)) {
                    Some(p) => (
                        Some(p.name_ur.clone()),
                        Some(p.name_en.clone()),
                        Some(p.phone.clone()),
                    ),
                    None => (None, None, None),
                };
                found.push(Buyer {
                    party: pukhta_party(party.clone(), key, name_ur.as_deref(), name_en.as_deref()),
                    city_id: None,
                    collector_id: None,
                    phone,
                    balance,
                    ages: None,
                });
            }
        } else {
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
    }
    Ok(found)
}

/// One open bill: a debit line less what the matching rows settle.
struct OpenBill {
    account_id: i64,
    person_key: Option<String>,
    date: String,
    open: Paisa,
}

/// The open bills of these accounts, oldest first. One query for each chunk of accounts (Code
/// Standards B22); the open amount is worked out here with checked maths.
fn open_bills(conn: &Connection, account_ids: &[i64]) -> Result<Vec<OpenBill>, ReportsError> {
    let mut found = Vec::new();
    for chunk in account_ids.chunks(IN_CHUNK) {
        let matched = "COALESCE((SELECT SUM(a.amount_paisa) FROM bill_allocations a
                                WHERE a.bill_line_id = l.id AND a.is_active = 1), 0)";
        let sql = format!(
            "SELECT l.account_id, l.pukhta_person_key, e.entry_date, l.dr_paisa, {matched}
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id IN ({marks}) AND l.dr_paisa > 0 AND {LIVE}
               AND l.dr_paisa > {matched}
             ORDER BY e.entry_date, e.id, l.line_no",
            marks = placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement
            .query_map(params_from_iter(chunk.iter()), |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (account_id, person_key, date, dr, matched) in rows {
            found.push(OpenBill {
                account_id,
                person_key,
                date,
                open: paisa(dr)?.checked_sub(paisa(matched)?)?,
            });
        }
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
fn put_ages(
    conn: &Connection,
    today: BusinessDate,
    buyers: &mut [Buyer],
) -> Result<(), ReportsError> {
    let ids: Vec<i64> = buyers.iter().map(|b| b.party.account_id).collect();
    let mut unique = ids.clone();
    unique.sort_unstable();
    unique.dedup();
    let bills = open_bills(conn, &unique)?;
    let mut by_key: HashMap<(i64, Option<String>), Vec<&OpenBill>> = HashMap::new();
    for bill in &bills {
        by_key
            .entry((bill.account_id, bill.person_key.clone()))
            .or_default()
            .push(bill);
    }
    for buyer in buyers.iter_mut() {
        let key = (buyer.party.account_id, buyer.party.person_key.clone());
        let mut bands = [Paisa::ZERO; 4];
        let mut oldest: Option<(String, i64)> = None;
        for bill in by_key.get(&key).map_or(&[][..], Vec::as_slice) {
            let day = BusinessDate::parse_iso(&bill.date)
                .ok_or_else(|| ReportsError::Data(format!("entry date {}", bill.date)))?;
            let days = today.days_after(day).max(0);
            if let Some(slot) = bands.get_mut(band_of(days)) {
                add(slot, bill.open)?;
            }
            if oldest.is_none() {
                oldest = Some((bill.date.clone(), days));
            }
        }
        buyer.ages = Some(Ages { oldest, bands });
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
    cells.push(text_cell(buyer.phone.clone()));
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
    let accounts = {
        let mut statement = conn.prepare(BUYERS_SQL)?;
        statement
            .query_map(params![typed.city_id, typed.party_id], |r| {
                Ok(Account {
                    id: r.get(0)?,
                    pukhta: r.get::<_, i64>(1)? == 1,
                    city_id: r.get(2)?,
                    collector_id: r.get(3)?,
                    phone: r.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
    };
    let account_ids: Vec<i64> = accounts.iter().map(|a| a.id).collect();
    let parties = load_parties(conn, &account_ids)?;
    let mut buyers = owing(conn, typed, &parties, &accounts)?;

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
