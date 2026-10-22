//! The balance lists of the Accounts group: R16 انڈکس (account index), R17 مدوں بیلنس (head
//! balances), R18 ٹرائل بیلنس (trial balance) and R19 اوپن ٹرائل (opening trial) (PRD 19.2,
//! FR-19.10, FR-19.11, NFR-SPD-13).
//!
//! The balance of an account is the posted `Σ debit − Σ credit` of its lines, up to the "as of"
//! day. The sums come from one grouped query over the stored `INTEGER` columns
//! (`mandi_db::ledger::account_totals`, the same one the services trial balance uses); the
//! difference is worked out here with checked maths. The totals row is a checked sum of the
//! balances of the whole filter, never of the rows of one page, and the two sides are equal
//! because every entry balances (PRD FR-6.2).
//!
//! Example (E2E-10, after the worked day): 85,048 on each side, with روکڑ 58,484 بنام and
//! ابتدائی سرمایہ 65,000 جمع.

use std::collections::HashMap;

use mandi_core::accounts::Balance;
use mandi_core::money::Paisa;
use mandi_db::ledger::{Totals, account_totals};
use mandi_db::rusqlite::{Connection, OptionalExtension, params, params_from_iter};

use super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{
    IN_CHUNK, amount_or_empty, balance_cell, dr_cr_cells, load_parties, paisa, placeholders,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, Names, PartyRef, ReportId, ReportPage, Row, RowKind, RowSource,
    SourceKind, SummaryLine,
};

fn col(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

/// The account ids that pass the city filter, in code order.
fn account_ids(conn: &Connection, city_id: Option<i64>) -> Result<Vec<i64>, ReportsError> {
    let mut statement =
        conn.prepare("SELECT id FROM accounts WHERE (?1 IS NULL OR city_id = ?1) ORDER BY code")?;
    let ids = statement
        .query_map(params![city_id], |r| r.get(0))?
        .collect::<Result<Vec<i64>, _>>()?;
    Ok(ids)
}

fn totals_map(
    conn: &Connection,
    as_of: Option<&str>,
) -> Result<HashMap<i64, Totals>, ReportsError> {
    Ok(account_totals(conn, as_of)?.into_iter().collect())
}

/// The two sides of a trial: the sum of the debit balances and the sum of the credit balances.
#[derive(Debug, Clone, Copy)]
struct Sides {
    dr: Paisa,
    cr: Paisa,
}

impl Sides {
    fn new() -> Self {
        Self {
            dr: Paisa::ZERO,
            cr: Paisa::ZERO,
        }
    }

    /// Adds the balance of one account on its side.
    fn add(&mut self, dr: Paisa, cr: Paisa) -> Result<(), ReportsError> {
        let balance = Balance::from_totals(dr, cr)?;
        match balance.side {
            mandi_core::accounts::BalanceSide::Dr => {
                self.dr = self.dr.checked_add(balance.amount_paisa)?;
            }
            mandi_core::accounts::BalanceSide::Cr => {
                self.cr = self.cr.checked_add(balance.amount_paisa)?;
            }
            mandi_core::accounts::BalanceSide::Zero => {}
        }
        Ok(())
    }

    fn difference(self) -> Result<Balance, ReportsError> {
        Ok(Balance::from_totals(self.dr, self.cr)?)
    }
}

fn totals_of(map: &HashMap<i64, Totals>, id: i64) -> Result<(Paisa, Paisa), ReportsError> {
    match map.get(&id) {
        Some(t) => Ok((paisa(t.dr_paisa)?, paisa(t.cr_paisa)?)),
        None => Ok((Paisa::ZERO, Paisa::ZERO)),
    }
}

fn account_source(id: i64) -> Option<RowSource> {
    Some(RowSource {
        kind: SourceKind::Account,
        id,
    })
}

/// R18: every account with posted lines, with its balance on its side.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(crate) fn run_r18(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let map = totals_map(conn, typed.as_of.as_deref())?;
    let ordered: Vec<i64> = account_ids(conn, None)?
        .into_iter()
        .filter(|id| map.contains_key(id))
        .collect();
    let mut sides = Sides::new();
    for id in &ordered {
        let (dr, cr) = totals_of(&map, *id)?;
        sides.add(dr, cr)?;
    }
    let page_ids = spec.slice(&ordered);
    let parties = load_parties(conn, &page_ids)?;
    let mut rows = Vec::with_capacity(page_ids.len());
    for id in &page_ids {
        let (dr, cr) = totals_of(&map, *id)?;
        let (dr_cell, cr_cell, _) = dr_cr_cells(dr, cr)?;
        rows.push(Row {
            cells: vec![
                Cell::Party {
                    party: party_ref(&parties, *id)?,
                },
                dr_cell,
                cr_cell,
            ],
            source: account_source(*id),
            kind: RowKind::Normal,
        });
    }
    Ok(ReportPage {
        id: ReportId::R18,
        title_id: ids::title_id("R18"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("account", ids::COL_ACCOUNT, ColumnKind::Party),
            col("debit", ids::COL_DEBIT, ColumnKind::Money),
            col("credit", ids::COL_CREDIT, ColumnKind::Money),
        ],
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: i64::try_from(ordered.len()).unwrap_or(i64::MAX),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Money { paisa: sides.dr },
            Cell::Money { paisa: sides.cr },
        ],
        summary: vec![SummaryLine {
            label_id: ids::DIFFERENCE.to_owned(),
            cell: balance_cell(sides.difference()?),
        }],
    })
}

fn party_ref(parties: &HashMap<i64, PartyRef>, id: i64) -> Result<PartyRef, ReportsError> {
    parties
        .get(&id)
        .cloned()
        .ok_or_else(|| ReportsError::Data(format!("account {id} is missing")))
}

/// R16: every account (of the city, if one is picked) with its code, city and balance.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(crate) fn run_r16(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let map = totals_map(conn, typed.as_of.as_deref())?;
    let ordered = account_ids(conn, typed.city_id)?;
    let mut sides = Sides::new();
    for id in &ordered {
        let (dr, cr) = totals_of(&map, *id)?;
        sides.add(dr, cr)?;
    }
    let page_ids = spec.slice(&ordered);
    let parties = load_parties(conn, &page_ids)?;
    let cities = load_cities(conn, &page_ids)?;
    let mut rows = Vec::with_capacity(page_ids.len());
    for id in &page_ids {
        let (dr, cr) = totals_of(&map, *id)?;
        let (dr_cell, cr_cell, _) = dr_cr_cells(dr, cr)?;
        rows.push(Row {
            cells: vec![
                Cell::Party {
                    party: party_ref(&parties, *id)?,
                },
                cities
                    .get(id)
                    .cloned()
                    .map_or(Cell::Empty, |names| Cell::Names { names }),
                dr_cell,
                cr_cell,
            ],
            source: account_source(*id),
            kind: RowKind::Normal,
        });
    }
    Ok(ReportPage {
        id: ReportId::R16,
        title_id: ids::title_id("R16"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("account", ids::COL_ACCOUNT, ColumnKind::Party),
            col("city", ids::COL_CITY, ColumnKind::Names),
            col("debit", ids::COL_DEBIT, ColumnKind::Money),
            col("credit", ids::COL_CREDIT, ColumnKind::Money),
        ],
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: i64::try_from(ordered.len()).unwrap_or(i64::MAX),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: sides.dr },
            Cell::Money { paisa: sides.cr },
        ],
        summary: Vec::new(),
    })
}

fn load_cities(
    conn: &Connection,
    account_ids: &[i64],
) -> Result<HashMap<i64, Names>, ReportsError> {
    let mut found = HashMap::new();
    for chunk in account_ids.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT a.id, c.name_ur, c.name_en FROM accounts a JOIN cities c ON c.id = a.city_id
             WHERE a.id IN ({})",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                Names {
                    ur: r.get(1)?,
                    en: r.get(2)?,
                },
            ))
        })?;
        for row in rows {
            let (id, names) = row?;
            found.insert(id, names);
        }
    }
    Ok(found)
}

/// One control and the head it belongs to.
struct ControlRow {
    control_id: i64,
    control_code: String,
    control_names: Names,
    head_id: i64,
    head_code: String,
    head_names: Names,
}

/// R17: the balance of every head and of every control under it, as of the day.
///
/// A head or control balance is the net of all its lines (`Σ debit − Σ credit`), shown on its
/// side. The total row adds the heads only, so nothing is counted twice.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(crate) fn run_r17(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let map = totals_map(conn, typed.as_of.as_deref())?;
    let mut by_control: HashMap<i64, (Paisa, Paisa)> = HashMap::new();
    {
        let mut statement = conn.prepare("SELECT id, control_id FROM accounts")?;
        let pairs = statement
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        for (account_id, control_id) in pairs {
            if let Some(t) = map.get(&account_id) {
                let slot = by_control
                    .entry(control_id)
                    .or_insert((Paisa::ZERO, Paisa::ZERO));
                slot.0 = slot.0.checked_add(paisa(t.dr_paisa)?)?;
                slot.1 = slot.1.checked_add(paisa(t.cr_paisa)?)?;
            }
        }
    }
    let controls = {
        let mut statement = conn.prepare(
            "SELECT c.id, c.code, c.name_ur, c.name_en, h.id, h.code, h.name_ur, h.name_en
             FROM account_controls c JOIN account_heads h ON h.id = c.head_id
             ORDER BY h.code, c.code",
        )?;
        statement
            .query_map([], |r| {
                Ok(ControlRow {
                    control_id: r.get(0)?,
                    control_code: r.get(1)?,
                    control_names: Names {
                        ur: r.get(2)?,
                        en: r.get(3)?,
                    },
                    head_id: r.get(4)?,
                    head_code: r.get(5)?,
                    head_names: Names {
                        ur: r.get(6)?,
                        en: r.get(7)?,
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
    };

    let mut all_rows: Vec<Row> = Vec::new();
    let mut sides = Sides::new();
    let mut index = 0;
    while index < controls.len() {
        let Some(first) = controls.get(index) else {
            break;
        };
        let head_id = first.head_id;
        let group: Vec<&ControlRow> = controls
            .iter()
            .skip(index)
            .take_while(|c| c.head_id == head_id)
            .collect();
        index += group.len();
        let mut head_dr = Paisa::ZERO;
        let mut head_cr = Paisa::ZERO;
        let mut control_rows = Vec::new();
        for control in &group {
            let Some((dr, cr)) = by_control.get(&control.control_id).copied() else {
                continue;
            };
            head_dr = head_dr.checked_add(dr)?;
            head_cr = head_cr.checked_add(cr)?;
            let (dr_cell, cr_cell, _) = dr_cr_cells(dr, cr)?;
            control_rows.push(Row {
                cells: vec![
                    Cell::Text {
                        text: control.control_code.clone(),
                    },
                    Cell::Names {
                        names: control.control_names.clone(),
                    },
                    dr_cell,
                    cr_cell,
                ],
                source: None,
                kind: RowKind::Normal,
            });
        }
        if control_rows.is_empty() {
            continue;
        }
        sides.add(head_dr, head_cr)?;
        let (dr_cell, cr_cell, _) = dr_cr_cells(head_dr, head_cr)?;
        all_rows.push(Row {
            cells: vec![
                Cell::Text {
                    text: first.head_code.clone(),
                },
                Cell::Names {
                    names: first.head_names.clone(),
                },
                dr_cell,
                cr_cell,
            ],
            source: None,
            kind: RowKind::Group,
        });
        all_rows.extend(control_rows);
    }

    Ok(ReportPage {
        id: ReportId::R17,
        title_id: ids::title_id("R17"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("code", ids::COL_ACCOUNT, ColumnKind::Text),
            col("head", ids::COL_HEAD, ColumnKind::Names),
            col("debit", ids::COL_DEBIT, ColumnKind::Money),
            col("credit", ids::COL_CREDIT, ColumnKind::Money),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: i64::try_from(all_rows.len()).unwrap_or(i64::MAX),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: sides.dr },
            Cell::Money { paisa: sides.cr },
        ],
        summary: vec![SummaryLine {
            label_id: ids::DIFFERENCE.to_owned(),
            cell: balance_cell(sides.difference()?),
        }],
    })
}

/// R19: the opening balances only (PRD FR-10.9). Each account with an opening shows it on its
/// side; the last row is ابتدائی فرق (06019999), the other side of every opening, so both sides
/// are equal and a wrong opening shows as a difference.
///
/// # Errors
/// A database error; `ReportsError::Data` if the system account 06019999 is missing;
/// `ReportsError::Money`.
pub(crate) fn run_r19(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    struct Opening {
        id: i64,
        date: Option<String>,
        dr: i64,
        cr: i64,
    }
    let openings = {
        let mut statement = conn.prepare(
            "SELECT id, opening_date, opening_dr_paisa, opening_cr_paisa FROM accounts
             WHERE (opening_dr_paisa > 0 OR opening_cr_paisa > 0) AND code <> '06019999'
             ORDER BY code",
        )?;
        statement
            .query_map([], |r| {
                Ok(Opening {
                    id: r.get(0)?,
                    date: r.get(1)?,
                    dr: r.get(2)?,
                    cr: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
    };
    let difference_id: i64 = conn
        .query_row("SELECT id FROM accounts WHERE code = '06019999'", [], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or_else(|| ReportsError::Data("the account 06019999 is missing".to_owned()))?;

    let mut sum_dr = Paisa::ZERO;
    let mut sum_cr = Paisa::ZERO;
    for o in &openings {
        sum_dr = sum_dr.checked_add(paisa(o.dr)?)?;
        sum_cr = sum_cr.checked_add(paisa(o.cr)?)?;
    }
    // The other side of the openings. Every opening posts against ابتدائی فرق, so it stands
    // on the side that makes both equal (PRD FR-10.9).
    let net = Balance::from_totals(sum_dr, sum_cr)?;
    let (diff_dr, diff_cr) = match net.side {
        mandi_core::accounts::BalanceSide::Dr => (Paisa::ZERO, net.amount_paisa),
        mandi_core::accounts::BalanceSide::Cr => (net.amount_paisa, Paisa::ZERO),
        mandi_core::accounts::BalanceSide::Zero => (Paisa::ZERO, Paisa::ZERO),
    };
    let total_dr = sum_dr.checked_add(diff_dr)?;
    let total_cr = sum_cr.checked_add(diff_cr)?;

    let mut ids_needed: Vec<i64> = openings.iter().map(|o| o.id).collect();
    ids_needed.push(difference_id);
    let parties = load_parties(conn, &ids_needed)?;
    let mut all_rows: Vec<Row> = Vec::with_capacity(openings.len() + 1);
    for o in &openings {
        let date = o.date.clone().map_or(Cell::Empty, |iso| Cell::Date { iso });
        all_rows.push(Row {
            cells: vec![
                Cell::Party {
                    party: party_ref(&parties, o.id)?,
                },
                date,
                amount_or_empty(paisa(o.dr)?),
                amount_or_empty(paisa(o.cr)?),
            ],
            source: account_source(o.id),
            kind: RowKind::Normal,
        });
    }
    all_rows.push(Row {
        cells: vec![
            Cell::Party {
                party: party_ref(&parties, difference_id)?,
            },
            Cell::Empty,
            amount_or_empty(diff_dr),
            amount_or_empty(diff_cr),
        ],
        source: account_source(difference_id),
        kind: RowKind::Normal,
    });

    let sides = Sides {
        dr: total_dr,
        cr: total_cr,
    };
    Ok(ReportPage {
        id: ReportId::R19,
        title_id: ids::title_id("R19"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("account", ids::COL_ACCOUNT, ColumnKind::Party),
            col("opening_date", ids::COL_OPENING_DATE, ColumnKind::Date),
            col("debit", ids::COL_DEBIT, ColumnKind::Money),
            col("credit", ids::COL_CREDIT, ColumnKind::Money),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: i64::try_from(all_rows.len()).unwrap_or(i64::MAX),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: total_dr },
            Cell::Money { paisa: total_cr },
        ],
        summary: vec![SummaryLine {
            label_id: ids::DIFFERENCE.to_owned(),
            cell: balance_cell(sides.difference()?),
        }],
    })
}
