//! The اگراہی sheet (plan 76 point 2, PRD R5 screen v1.41): the total the shop still has to take
//! from its buyers, buyer by buyer, for one day.
//!
//! A row for every buyer who owed at the START of the day (his balance before it) and for every
//! buyer who bought that day, even one who paid in full. Each row:
//!
//! * **previous due**: his ledger balance at the start of the day, every posted line dated before
//!   it (so every وصولی up to the day before is already off it);
//! * **today's bill**: Σ of the buyer totals of his active sale lines of the day (C5, as stored);
//! * **paid on the spot**: Σ of the money he paid on those lines (PRD FR-12.44);
//! * **total due** = previous due + today's bill − paid on the spot.
//!
//! There is no وصولی column. A وصولی is collected later in the day, so it shows in the next
//! day's previous due, because every figure is read from the books. For the same reason anything
//! else posted on his account that day (cash lent, a جنرل line) waits for the next day's previous
//! due; the sheet never adds a figure that is not one of its four columns.
//!
//! People in the dead amount follow the R35 table rule: they are listed only on a day they bought,
//! and they are left out of the main total and have a total of their own. A چکتا buyer is listed
//! on the day he bought, by name, with 0 due, because he pays in full.
//!
//! Example (after Events 1 to 5, 10-10-26): اکرم 0 + 12,880 − 0 = 12,880; ارشد 0 + 18,684 − 0 =
//! 18,684; شاہد (چکتا) 0 + 12,800 − 12,800 = 0; the total 0 + 44,364 − 12,800 = 31,564. The next
//! day, with no sale: اکرم 2,880 and ارشد 18,684, total 21,564, the total of R20.

use std::collections::HashMap;

use mandi_core::accounts::{BUYER_CONTROL, Balance, BalanceSide};
use mandi_core::auth::Language;
use mandi_core::bikri::SaleGrade;
use mandi_core::dates::{BusinessDate, parse_ddmmyyyy};
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_core::paging::{SortDirection, SortInput};
use mandi_core::printing::Paper;
use mandi_core::text::{search_en, search_ur};
use mandi_db::Db;
use mandi_db::ledger::account_totals_in_controls;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::activity::{parties_for, sort_by_party};
use super::figures_load::{BuyerLine, Filter, buyer_lines};
use crate::error::ReportsError;
use crate::print::PrintFact;
use crate::reports::common::{add, in_snapshot, paisa, placeholders, today as today_of};
use crate::reports::model::{Cell, Column, ColumnKind, DetailPart, Names, PartyRef, Row, RowKind};
use crate::reports::table_print::{TablePrintData, TableSpec, table_print_data};

/// The rows a page holds when no limit is sent, and the most.
const PAGE: usize = 100;
/// The columns a sort may name.
const SORT_COLUMNS: [&str; 5] = ["party", "previous_due", "bill", "paid", "total_due"];

/// Input of `reports_collection_sheet`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct CollectionSheetInput {
    /// The day, `DD-MM-YYYY` or a shortcut (PRD FR-7.19), or `YYYY-MM-DD`. Empty is today. A day
    /// after today is refused (`ERR_DATE_IN_FUTURE` on `date`).
    #[serde(default)]
    pub date: String,
    /// Part of the code, the running number or a name in either script.
    #[serde(default)]
    pub search: Option<String>,
    /// `party`, `previous_due`, `bill`, `paid` or `total_due`. Null is by name, in the Urdu order.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives 100.
    #[serde(default)]
    pub limit: u32,
}

/// One sale line of the day in short: the item, the quantity and the rate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionLine {
    /// The item.
    pub item: Names,
    /// The packing.
    pub packing: Names,
    /// The quality of the goods (PRD FR-12.42).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub grade: Option<SaleGrade>,
    /// The quantity in نگ.
    #[ts(type = "number")]
    pub qty: i64,
    /// The weight in grams of a line of an item sold by weight, else null.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The buyer's rate of the line as stored: whole rupees per نگ or per kg.
    #[ts(type = "Paisa")]
    pub rate_paisa: Paisa,
}

/// One buyer of the sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionSheetRow {
    /// The buyer, a چکتا buyer by his own names.
    pub party: PartyRef,
    /// What he bought that day, in short. Empty when he bought nothing.
    pub lines: Vec<CollectionLine>,
    /// His balance at the start of the day.
    pub previous_due: Balance,
    /// What he was charged that day.
    #[ts(type = "Paisa")]
    pub bill_paisa: Paisa,
    /// What he paid on the spot on those lines.
    #[ts(type = "Paisa")]
    pub paid_paisa: Paisa,
    /// previous due + today's bill − paid on the spot.
    pub total_due: Balance,
    /// True when he is in the dead amount (PRD FR-10.10).
    pub is_dead: bool,
}

/// The four figures of the sheet added up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionSheetTotals {
    /// Σ previous due.
    pub previous_due: Balance,
    /// Σ today's bills.
    #[ts(type = "Paisa")]
    pub bill_paisa: Paisa,
    /// Σ paid on the spot.
    #[ts(type = "Paisa")]
    pub paid_paisa: Paisa,
    /// Σ total due: the whole اگراہی after today's sales.
    pub total_due: Balance,
}

/// What `reports_collection_sheet` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionSheet {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// One page of buyers.
    pub rows: Vec<CollectionSheetRow>,
    /// How many buyers match (the dead ones included).
    #[ts(type = "number")]
    pub total_count: i64,
    /// The total of every matching buyer who is not in the dead amount and whose total due is not
    /// جمع (never of the page). On a day with no وصولی it is the R20 total.
    pub totals: CollectionSheetTotals,
    /// The total of the matching people in the dead amount, listed apart; null when there is none.
    pub dead_totals: Option<CollectionSheetTotals>,
    /// The total of the matching buyers whose total due ends in جمع (the shop owes him: he started
    /// the day in credit and bought less than it), listed apart so the main total stays what the
    /// buyers owe, as R20 (PRD FR-19.9); null when there is none.
    pub credit_totals: Option<CollectionSheetTotals>,
}

#[derive(Default)]
struct Sums {
    previous: Paisa,
    bill: Paisa,
    paid: Paisa,
}

impl Sums {
    fn add_row(&mut self, previous: Paisa, bill: Paisa, paid: Paisa) -> Result<(), ReportsError> {
        self.previous = self.previous.checked_add(previous)?;
        add(&mut self.bill, bill)?;
        add(&mut self.paid, paid)?;
        Ok(())
    }

    fn totals(&self) -> Result<CollectionSheetTotals, ReportsError> {
        let due = self
            .previous
            .checked_add(self.bill)?
            .checked_sub(self.paid)?;
        Ok(CollectionSheetTotals {
            previous_due: Balance::from_net(self.previous)?,
            bill_paisa: self.bill,
            paid_paisa: self.paid,
            total_due: Balance::from_net(due)?,
        })
    }
}

fn reject(code: ErrorCode, field: &str) -> ReportsError {
    ReportsError::Rejected(vec![FieldError::new(code, field)])
}

fn day_of(text: &str, today: BusinessDate) -> Result<BusinessDate, ReportsError> {
    let text = text.trim();
    let day = if text.is_empty() {
        today
    } else {
        BusinessDate::parse_iso(text)
            .or_else(|| parse_ddmmyyyy(text, today).ok())
            .ok_or_else(|| reject(ErrorCode::ErrDateInvalid, "date"))?
    };
    if day > today {
        return Err(reject(ErrorCode::ErrDateInFuture, "date"));
    }
    Ok(day)
}

/// The bikri numbers of the lines, in one read.
fn numbers_of(
    conn: &mandi_db::rusqlite::Connection,
    lines: &[&BuyerLine],
) -> Result<HashMap<i64, i64>, ReportsError> {
    let mut ids: Vec<i64> = lines.iter().map(|l| l.bikri_id).collect();
    ids.sort_unstable();
    ids.dedup();
    let mut out = HashMap::new();
    for chunk in ids.chunks(500) {
        let sql = format!(
            "SELECT id, number FROM bikri_entries WHERE id IN ({})",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement
            .query_map(mandi_db::rusqlite::params_from_iter(chunk.iter()), |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
            })?;
        for row in rows {
            let (id, number) = row?;
            out.insert(id, number);
        }
    }
    Ok(out)
}

/// The ids of the accounts that hold چکتا buyers (the chukta account and the old closed one): they
/// have no balance of their own.
fn chukta_accounts(
    conn: &mandi_db::rusqlite::Connection,
) -> Result<std::collections::HashSet<i64>, ReportsError> {
    let mut statement = conn.prepare("SELECT id FROM accounts WHERE is_chukta_khata = 1")?;
    let ids = statement
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<Result<std::collections::HashSet<_>, _>>()?;
    Ok(ids)
}

/// A buyer of the sheet before it is cut into pages, with his lines as stored.
struct Entry {
    row: CollectionSheetRow,
    parts: Vec<DetailPart>,
}

/// The sheet as loaded: the day, every row, and the three totals.
struct Loaded {
    date: String,
    entries: Vec<Entry>,
    main: Sums,
    dead: Option<Sums>,
    credit: Option<Sums>,
}

/// Every row of the sheet, sorted, with the totals.
fn load_all(db: &Db, input: &CollectionSheetInput) -> Result<Loaded, ReportsError> {
    if let Some(sort) = &input.sort {
        sort.check(&SORT_COLUMNS)
            .map_err(|e| ReportsError::Rejected(vec![e]))?;
    }
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let day = day_of(&input.date, today)?;
            let iso = day.to_iso();
            let all_lines = buyer_lines(conn, &iso, &iso, Filter::default())?;
            let mut by_party: HashMap<(i64, Option<String>), Vec<&BuyerLine>> = HashMap::new();
            for line in &all_lines {
                let person = (!line.person.is_empty()).then(|| line.person.clone());
                by_party
                    .entry((line.account_id, person))
                    .or_default()
                    .push(line);
            }
            // The keys of the sheet: a buyer who owed at the start or who bought today. The balance
            // at the start of the day of every buyer account is one grouped read over the buyer
            // accounts, the posted lines dated up to the day before (the same read the dashboard
            // tile and R20 use), not a pass over every line of the books (NFR-SPD-24). An old open
            // amount on the چکتا account stays in the books and out of the dues lists (PRD FR-13.9,
            // as R20): a چکتا buyer is listed only on the day he bought, with a previous due of 0.
            let mut keys: Vec<(i64, Option<String>)> = Vec::new();
            let mut previous: HashMap<(i64, Option<String>), Paisa> = HashMap::new();
            let chukta = chukta_accounts(conn)?;
            if let Some(before) = day.previous_day() {
                for (account, totals) in
                    account_totals_in_controls(conn, &[BUYER_CONTROL], Some(&before.to_iso()))?
                {
                    if chukta.contains(&account) {
                        continue;
                    }
                    let prev = paisa(totals.dr_paisa)?.checked_sub(paisa(totals.cr_paisa)?)?;
                    previous.insert((account, None), prev);
                    if prev > Paisa::ZERO {
                        keys.push((account, None));
                    }
                }
            }
            for key in by_party.keys() {
                previous.entry(key.clone()).or_insert(Paisa::ZERO);
                keys.push(key.clone());
            }
            keys.sort();
            keys.dedup();
            let parties = parties_for(conn, &keys)?;
            let dead = mandi_db::dead::dead_set(conn, Some(&iso))?;
            let typed = input.search.as_deref().map_or("", str::trim);
            let (text_ur, text_en) = (search_ur(typed), search_en(typed));
            let numbers = numbers_of(conn, &all_lines.iter().collect::<Vec<_>>())?;

            let mut entries = Vec::new();
            let (mut main, mut dead_sums, mut any_dead) = (Sums::default(), Sums::default(), false);
            let (mut credit_sums, mut any_credit) = (Sums::default(), false);
            for key in keys {
                let party = parties.get(&key).cloned().ok_or_else(|| {
                    ReportsError::Data(format!("buyer {} of the sheet has no account", key.0))
                })?;
                let lines = by_party.get(&key).cloned().unwrap_or_default();
                let is_dead = dead.has_account(key.0);
                // A person in the dead amount is listed only on a day he bought (R35 rule).
                if is_dead && lines.is_empty() {
                    continue;
                }
                if !typed.is_empty()
                    && !(party.code.contains(typed)
                        || party.short_code.as_deref() == Some(typed)
                        || search_ur(&party.name_ur).contains(&text_ur)
                        || search_en(&party.name_en).contains(&text_en))
                {
                    continue;
                }
                let prev = previous.get(&key).copied().unwrap_or(Paisa::ZERO);
                let (mut bill, mut paid) = (Paisa::ZERO, Paisa::ZERO);
                let mut shown = Vec::with_capacity(lines.len());
                let mut parts = Vec::with_capacity(lines.len());
                for l in &lines {
                    add(&mut bill, l.total)?;
                    add(&mut paid, l.paid)?;
                    shown.push(CollectionLine {
                        item: l.goods.item.clone(),
                        packing: l.goods.packing.clone(),
                        grade: l.grade,
                        qty: l.qty,
                        weight_grams: l.weight_grams,
                        rate_paisa: l.rate,
                    });
                    parts.push(DetailPart::Sale {
                        bikri_number: numbers.get(&l.bikri_id).copied().unwrap_or(0),
                        item: l.goods.item.clone(),
                        packing: l.goods.packing.clone(),
                        grade: l.grade,
                        qty: l.qty,
                        rate_paisa: l.rate,
                        value_paisa: l.value,
                        commission_paisa: l.commission,
                        laga_paisa: l.laga,
                    });
                }
                let due = prev.checked_add(bill)?.checked_sub(paid)?;
                if is_dead {
                    any_dead = true;
                    dead_sums.add_row(prev, bill, paid)?;
                } else if due.is_negative() {
                    // He started the day in credit and bought less than it: he stays on the sheet
                    // with his side word, but the main total is what buyers owe (R20, FR-19.9).
                    any_credit = true;
                    credit_sums.add_row(prev, bill, paid)?;
                } else {
                    main.add_row(prev, bill, paid)?;
                }
                entries.push(Entry {
                    row: CollectionSheetRow {
                        party,
                        lines: shown,
                        previous_due: Balance::from_net(prev)?,
                        bill_paisa: bill,
                        paid_paisa: paid,
                        total_due: Balance::from_net(due)?,
                        is_dead,
                    },
                    parts,
                });
            }
            // By name in the Urdu order unless a column is asked for.
            let mut sorted = sort_by_party(entries, &parties, |e| {
                (e.row.party.account_id, e.row.party.person_key.clone())
            });
            if let Some(sort) = &input.sort
                && sort.column != "party"
            {
                let direction: SortDirection = sort.direction;
                let signed = |b: &Balance| match b.side {
                    BalanceSide::Cr => b.amount_paisa.as_paisa().checked_neg().unwrap_or(i64::MIN),
                    _ => b.amount_paisa.as_paisa(),
                };
                sorted.sort_by(|a, b| {
                    let ordering = match sort.column.as_str() {
                        "previous_due" => {
                            signed(&a.row.previous_due).cmp(&signed(&b.row.previous_due))
                        }
                        "bill" => a.row.bill_paisa.cmp(&b.row.bill_paisa),
                        "paid" => a.row.paid_paisa.cmp(&b.row.paid_paisa),
                        _ => signed(&a.row.total_due).cmp(&signed(&b.row.total_due)),
                    };
                    direction.apply(ordering)
                });
            } else if let Some(sort) = &input.sort
                && sort.direction == SortDirection::Desc
            {
                sorted.reverse();
            }
            Ok(Loaded {
                date: iso,
                entries: sorted,
                main,
                dead: any_dead.then_some(dead_sums),
                credit: any_credit.then_some(credit_sums),
            })
        })
    })
}

/// A page of the sheet (plan 76 point 2). Read only.
///
/// # Errors
/// `ERR_DATE_INVALID` and `ERR_DATE_IN_FUTURE` on `date`; `ERR_OUT_OF_RANGE` on `sort.column`; a
/// database or money error.
pub fn collection_sheet(
    db: &Db,
    input: &CollectionSheetInput,
) -> Result<CollectionSheet, ReportsError> {
    let Loaded {
        date,
        entries,
        main,
        dead,
        credit,
    } = load_all(db, input)?;
    let total_count = i64::try_from(entries.len()).unwrap_or(i64::MAX);
    let limit = match input.limit as usize {
        0 => PAGE,
        n => n.min(PAGE),
    };
    Ok(CollectionSheet {
        date,
        rows: entries
            .into_iter()
            .skip(input.offset as usize)
            .take(limit)
            .map(|e| e.row)
            .collect(),
        total_count,
        totals: main.totals()?,
        dead_totals: dead.map(|d| d.totals()).transpose()?,
        credit_totals: credit.map(|c| c.totals()).transpose()?,
    })
}

/// Input of `reports_collection_sheet_print_data`: the screen's filter and the print language.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CollectionSheetPrintInput {
    /// The day, as in [`CollectionSheetInput`].
    #[serde(default)]
    pub date: String,
    /// The search of the screen.
    #[serde(default)]
    pub search: Option<String>,
    /// The sort of the screen.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}

/// The print of the sheet: every buyer, in the print language, with the totals row, then the
/// people in the dead amount as a second total row.
///
/// # Errors
/// As [`collection_sheet`]; a stored amount that cannot be written in words.
pub fn collection_sheet_print_data(
    db: &Db,
    input: &CollectionSheetPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = collection_sheet_spec(db, input)?;
    table_print_data(db, input.language, input.paper, spec)
}

/// The table of the sheet for print and Excel (plan 76 point 2).
///
/// # Errors
/// As [`collection_sheet`]; a stored amount that cannot be written in words.
pub fn collection_sheet_spec(
    db: &Db,
    input: &CollectionSheetPrintInput,
) -> Result<TableSpec, ReportsError> {
    let Loaded {
        date,
        entries,
        main,
        dead,
        credit,
    } = load_all(
        db,
        &CollectionSheetInput {
            date: input.date.clone(),
            search: input.search.clone(),
            sort: input.sort.clone(),
            offset: 0,
            limit: 0,
        },
    )?;
    let column = |key: &str, title: &str, kind: ColumnKind| Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    };
    let columns = vec![
        column("code", "print-collection-sheet-col-code", ColumnKind::Text),
        column(
            "party",
            "print-collection-sheet-col-party",
            ColumnKind::Party,
        ),
        column(
            "detail",
            "print-collection-sheet-col-detail",
            ColumnKind::Detail,
        ),
        column(
            "previous_due",
            "print-collection-sheet-col-previous",
            ColumnKind::Balance,
        ),
        column("bill", "print-collection-sheet-col-bill", ColumnKind::Money),
        column("paid", "print-collection-sheet-col-paid", ColumnKind::Money),
        column(
            "total_due",
            "print-collection-sheet-col-due",
            ColumnKind::Balance,
        ),
    ];
    let count = i64::try_from(entries.len()).unwrap_or(i64::MAX);
    let rows = entries
        .into_iter()
        .map(|e| Row {
            cells: vec![
                Cell::Text {
                    text: e
                        .row
                        .party
                        .short_code
                        .clone()
                        .unwrap_or_else(|| e.row.party.code.clone()),
                },
                Cell::Party { party: e.row.party },
                if e.parts.is_empty() {
                    Cell::Empty
                } else {
                    Cell::Detail { parts: e.parts }
                },
                Cell::Balance {
                    paisa: e.row.previous_due.amount_paisa,
                    side: e.row.previous_due.side,
                },
                Cell::Money {
                    paisa: e.row.bill_paisa,
                },
                Cell::Money {
                    paisa: e.row.paid_paisa,
                },
                Cell::Balance {
                    paisa: e.row.total_due.amount_paisa,
                    side: e.row.total_due.side,
                },
            ],
            source: None,
            kind: RowKind::Normal,
        })
        .collect();
    let totals_cells = |t: &CollectionSheetTotals| -> Vec<Cell> {
        vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Balance {
                paisa: t.previous_due.amount_paisa,
                side: t.previous_due.side,
            },
            Cell::Money {
                paisa: t.bill_paisa,
            },
            Cell::Money {
                paisa: t.paid_paisa,
            },
            Cell::Balance {
                paisa: t.total_due.amount_paisa,
                side: t.total_due.side,
            },
        ]
    };
    let totals = main.totals()?;
    let (y, rest) = date.split_once('-').unwrap_or(("", ""));
    let (m, d) = rest.split_once('-').unwrap_or(("", ""));
    let mut facts = vec![PrintFact {
        label_id: "print-fact-date".to_owned(),
        value: format!("{d}-{m}-{y}"),
    }];
    if let Some(text) = input
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        facts.push(PrintFact {
            label_id: "print-fact-search".to_owned(),
            value: text.to_owned(),
        });
    }
    Ok(TableSpec {
        title_id: "print-collection-sheet-title".to_owned(),
        facts,
        column_names: vec![None; columns.len()],
        columns,
        rows,
        totals_label_id: "print-collection-sheet-total".to_owned(),
        totals: totals_cells(&totals),
        total_count: count,
        words_of: Some(totals.total_due.amount_paisa)
            .filter(|a| *a != Paisa::ZERO && totals.total_due.side == BalanceSide::Dr),
        plain_sale: true,
        // The people in the dead amount, then the buyers whose total due ends in جمع, each in a
        // total row of its own under the main one.
        extra_totals: [
            ("print-collection-sheet-dead-total", dead),
            ("print-collection-sheet-credit-total", credit),
        ]
        .into_iter()
        .filter_map(|(label, sums)| sums.map(|s| (label, s)))
        .map(|(label, sums)| Ok((label.to_owned(), totals_cells(&sums.totals()?))))
        .collect::<Result<Vec<_>, ReportsError>>()?,
    })
}
