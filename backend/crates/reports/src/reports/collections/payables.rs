//! R24 ادائیگی باقی (seller payables) (PRD 19.2, FR-7.59, FR-15.9, FR-19.10).
//!
//! One row for each seller (and each one-time person inside the پختہ کھاتہ seller account): what
//! is still payable to him, his open advances, what he owes the firm beyond his advances, his
//! ledger balance and the date of his last بکری. The three amounts are the read-time working of
//! PRD FR-7.59 (`workings::payable_totals_from` over sums of his live lines, the same
//! numbers the voucher screen and the dashboard tile show). The ledger balance is the posted
//! `Σ جمع − Σ بنام` and equals `payable − open advance − owed beyond advances`.
//!
//! The rows are in two lists (PRD 19.2): the sellers the firm owes or has advanced to, and, below
//! them, "بیوپاری کے ذمے", the sellers who owe the firm beyond their advances (PRD FR-15.9). Each
//! list has a heading row with its totals. The total row adds both lists.
//!
//! The advance and payment rows are kept as they stand now and are not dated, so the three
//! workings can only be given for a report as of today or later. For an earlier day only the
//! ledger balance is given (one flat list, the other cells empty): nothing is guessed.
//!
//! Example (after Event 2, PRD 4.9): کریم payable 10,160, advance 0, balance 10,160 جمع; رحیم is
//! not listed (everything adjusted and paid). After E2E-28 رحیم shows payable 19,316, advance
//! 15,000 and balance 4,316 جمع.

use std::collections::{HashMap, HashSet};

use mandi_core::accounts::{Balance, BalanceSide};
use mandi_core::money::Paisa;
use mandi_db::ledger::{Totals, account_totals, totals_of};
use mandi_db::rusqlite::{Connection, params};
use mandi_db::vouchers::advance_totals;

use super::super::PageSpec;
use super::grouped::{SellerSums, payable_sums, pukhta_names};
use crate::error::ReportsError;
use crate::reports::common::{
    add, amount_or_empty, balance_cell, col, count_of, date_cell, load_parties, paisa, pukhta_party,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, PartyRef, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};
use crate::reports::workings::payable_totals_from;

/// The seller accounts. Parameter: account.
fn sellers_sql() -> String {
    format!(
        "SELECT a.id, a.is_pukhta_khata
    FROM accounts a JOIN account_controls k ON k.id = a.control_id
    WHERE k.code = '03011' AND (?1 IS NULL OR a.id = ?1)
    ORDER BY {}",
        mandi_db::codes::order_by_code("a.code")
    )
}

/// The posted totals of each person of a پختہ کھاتہ account. Parameters: account, as-of day.
const PERSONS_SQL: &str = "SELECT l.pukhta_person_key, COALESCE(SUM(l.dr_paisa), 0),
        COALESCE(SUM(l.cr_paisa), 0)
    FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
    WHERE l.account_id = ?1 AND e.status = 'posted' AND (?2 IS NULL OR e.entry_date <= ?2)
    GROUP BY l.pukhta_person_key ORDER BY l.pukhta_person_key";

/// The date of the last saved بکری of each seller key.
const LAST_DATE_SQL: &str = "SELECT l.seller_account_id, l.pukhta_person_key, MAX(b.entry_date)
    FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id
    WHERE b.status = 'saved' GROUP BY l.seller_account_id, l.pukhta_person_key";

/// What the working gives for one seller key.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Working {
    /// What is payable to him (PRD FR-7.59).
    pub(crate) open: Paisa,
    /// His open advances (PRD FR-15.8).
    pub(crate) advances_open: Paisa,
    /// What he owes the firm beyond his advances (PRD FR-15.9).
    pub(crate) owes_firm: Paisa,
}

/// One seller key (an account, or a person inside a پختہ کھاتہ account).
pub(crate) struct Seller {
    pub(crate) party: PartyRef,
    pub(crate) balance: Balance,
    pub(crate) working: Option<Working>,
    pub(crate) last_date: Option<String>,
}

impl Seller {
    /// True for a seller who owes the firm beyond his advances (PRD FR-15.9).
    fn owes_firm(&self) -> bool {
        self.working.is_some_and(|w| w.owes_firm > Paisa::ZERO)
    }
}

/// The sellers worth a row: a balance, or an advance, or something payable or owed.
pub(crate) fn candidates(
    conn: &Connection,
    typed: &Typed,
    today_iso: &str,
    current: bool,
) -> Result<(Vec<Seller>, Option<SellerSums>), ReportsError> {
    let as_of = typed.as_of.as_deref();
    let accounts = {
        let mut statement = conn.prepare(&sellers_sql())?;
        statement
            .query_map(params![typed.party_id], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? == 1))
            })?
            .collect::<Result<Vec<_>, _>>()?
    };
    let ids: Vec<i64> = accounts.iter().map(|(id, _)| *id).collect();
    let parties = load_parties(conn, &ids)?;
    // The payable sums of every seller key and the names of the one-time people, each from a few
    // grouped queries and not one set of queries for each seller (Code Standards B22).
    let sums = if current {
        Some(payable_sums(conn, typed.party_id)?)
    } else {
        None
    };
    let pukhta_ids: Vec<i64> = accounts
        .iter()
        .filter(|(_, pukhta)| *pukhta)
        .map(|(id, _)| *id)
        .collect();
    let names = pukhta_names(conn, &pukhta_ids)?;
    let totals: HashMap<i64, Totals> = if typed.party_id.is_some() {
        HashMap::new()
    } else {
        account_totals(conn, as_of)?.into_iter().collect()
    };
    let with_advance: HashSet<i64> = if current {
        advance_totals(conn, as_of.unwrap_or(today_iso), typed.party_id)?
            .into_iter()
            .map(|a| a.account_id)
            .collect()
    } else {
        HashSet::new()
    };
    let last_dates = {
        let mut statement = conn.prepare(LAST_DATE_SQL)?;
        statement
            .query_map([], |r| {
                Ok((
                    (r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?),
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<HashMap<_, _>, _>>()?
    };

    let mut found = Vec::new();
    for (id, pukhta) in accounts {
        let party = parties
            .get(&id)
            .cloned()
            .ok_or_else(|| ReportsError::Data(format!("seller account {id} is missing")))?;
        if pukhta {
            let mut statement = conn.prepare(PERSONS_SQL)?;
            let persons = statement
                .query_map(params![id, as_of], |r| {
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
                let balance = Balance::from_totals(paisa(dr)?, paisa(cr)?)?;
                let (name_ur, name_en) = match key.as_deref().and_then(|k| names.get(k)) {
                    Some(p) => (Some(p.name_ur.clone()), Some(p.name_en.clone())),
                    None => (None, None),
                };
                let working = sums
                    .as_ref()
                    .map(|s| working_of(s, id, key.as_deref()))
                    .transpose()?;
                if !worth_a_row(balance, working) {
                    continue;
                }
                found.push(Seller {
                    party: pukhta_party(
                        party.clone(),
                        key.clone(),
                        name_ur.as_deref(),
                        name_en.as_deref(),
                    ),
                    balance,
                    working,
                    last_date: last_dates.get(&(id, key)).cloned(),
                });
            }
        } else {
            let own = if typed.party_id.is_some() {
                totals_of(conn, id, None, as_of)?
            } else {
                totals.get(&id).copied().unwrap_or(Totals {
                    dr_paisa: 0,
                    cr_paisa: 0,
                })
            };
            let balance = Balance::from_totals(paisa(own.dr_paisa)?, paisa(own.cr_paisa)?)?;
            let has_advance = with_advance.contains(&id);
            if balance.side == BalanceSide::Zero && !has_advance {
                continue;
            }
            let working = sums.as_ref().map(|s| working_of(s, id, None)).transpose()?;
            if !worth_a_row(balance, working) {
                continue;
            }
            found.push(Seller {
                party,
                balance,
                working,
                last_date: last_dates.get(&(id, None)).cloned(),
            });
        }
    }
    Ok((found, sums))
}

fn worth_a_row(balance: Balance, working: Option<Working>) -> bool {
    if balance.side != BalanceSide::Zero {
        return true;
    }
    working.is_some_and(|w| {
        w.open > Paisa::ZERO || w.advances_open > Paisa::ZERO || w.owes_firm > Paisa::ZERO
    })
}

/// The payable working of one seller key from the grouped sums of his live lines (PRD FR-7.59).
fn working_of(
    sums: &SellerSums,
    account_id: i64,
    person_key: Option<&str>,
) -> Result<Working, ReportsError> {
    let t = sums.of(account_id, person_key);
    let totals = payable_totals_from(
        paisa(t.credit)?,
        paisa(t.payments)?,
        paisa(t.advances_given)?,
        paisa(t.adjusted_on_credits)?,
        paisa(t.adjusted_on_advances)?,
    )?;
    Ok(Working {
        open: totals.open,
        advances_open: totals.advances_open,
        owes_firm: totals.excess_paid,
    })
}

fn seller_row(seller: &Seller) -> Row {
    let (open, advance, owes) = match seller.working {
        Some(w) => (
            amount_or_empty(w.open),
            amount_or_empty(w.advances_open),
            amount_or_empty(w.owes_firm),
        ),
        None => (Cell::Empty, Cell::Empty, Cell::Empty),
    };
    Row {
        cells: vec![
            Cell::Empty,
            Cell::Party {
                party: seller.party.clone(),
            },
            open,
            advance,
            owes,
            balance_cell(seller.balance),
            date_cell(seller.last_date.clone()),
        ],
        source: Some(RowSource {
            kind: SourceKind::Account,
            id: seller.party.account_id,
        }),
        kind: RowKind::Normal,
    }
}

/// A heading row with the totals of a list.
fn heading(label: &str, sellers: &[&Seller]) -> Result<Row, ReportsError> {
    let (mut open, mut advance, mut owes) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    for seller in sellers {
        if let Some(w) = seller.working {
            add(&mut open, w.open)?;
            add(&mut advance, w.advances_open)?;
            add(&mut owes, w.owes_firm)?;
        }
    }
    Ok(Row {
        cells: vec![
            Cell::Label {
                id: label.to_owned(),
            },
            Cell::Empty,
            amount_or_empty(open),
            amount_or_empty(advance),
            amount_or_empty(owes),
            Cell::Empty,
            Cell::Empty,
        ],
        source: None,
        kind: RowKind::Group,
    })
}

/// Builds R24.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees, a total does
/// not fit, or more is matched than a side holds (a bug signal).
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let today = typed
        .today
        .ok_or_else(|| ReportsError::Data("the report has no today".to_owned()))?;
    let today_iso = today.to_iso();
    let current = typed
        .as_of
        .as_deref()
        .is_none_or(|as_of| as_of >= today_iso.as_str());
    let (sellers, _) = candidates(conn, typed, &today_iso, current)?;

    // Totals over the whole filter.
    let (mut open, mut advance, mut owes) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    let (mut sum_dr, mut sum_cr) = (Paisa::ZERO, Paisa::ZERO);
    for seller in &sellers {
        if let Some(w) = seller.working {
            add(&mut open, w.open)?;
            add(&mut advance, w.advances_open)?;
            add(&mut owes, w.owes_firm)?;
        }
        match seller.balance.side {
            BalanceSide::Dr => add(&mut sum_dr, seller.balance.amount_paisa)?,
            BalanceSide::Cr => add(&mut sum_cr, seller.balance.amount_paisa)?,
            BalanceSide::Zero => {}
        }
    }

    let mut all_rows = Vec::new();
    if current {
        let (owing, payable): (Vec<&Seller>, Vec<&Seller>) =
            sellers.iter().partition(|s| s.owes_firm());
        if !payable.is_empty() {
            all_rows.push(heading(ids::SECTION_PAYABLE, &payable)?);
            all_rows.extend(payable.iter().map(|s| seller_row(s)));
        }
        if !owing.is_empty() {
            all_rows.push(heading(ids::SECTION_OWES_FIRM, &owing)?);
            all_rows.extend(owing.iter().map(|s| seller_row(s)));
        }
    } else {
        all_rows.extend(sellers.iter().map(seller_row));
    }

    Ok(ReportPage {
        id: ReportId::R24,
        title_id: ids::title_id("R24"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("section", ids::COL_SECTION, ColumnKind::Label),
            col("seller", ids::COL_SELLER, ColumnKind::Party),
            col("payable", ids::COL_PAYABLE, ColumnKind::Money),
            col("advance_open", ids::COL_ADVANCE_OPEN, ColumnKind::Money),
            col("owes_firm", ids::COL_OWES_FIRM, ColumnKind::Money),
            col("balance", ids::COL_BALANCE, ColumnKind::Balance),
            col("last_date", ids::COL_LAST_DATE, ColumnKind::Date),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            if current {
                Cell::Money { paisa: open }
            } else {
                Cell::Empty
            },
            if current {
                Cell::Money { paisa: advance }
            } else {
                Cell::Empty
            },
            if current {
                Cell::Money { paisa: owes }
            } else {
                Cell::Empty
            },
            balance_cell(Balance::from_totals(sum_dr, sum_cr)?),
            Cell::Empty,
        ],
        summary: Vec::new(),
    })
}
