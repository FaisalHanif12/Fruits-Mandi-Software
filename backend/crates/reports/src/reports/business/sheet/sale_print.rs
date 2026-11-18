//! The whole sale print, پوری بکری (plan 78 A4, PRD FR-19.15, FR-12.12): everything sold in one
//! بکری, in one save, or on one day, on one or two pages.
//!
//! Two tables follow each other. There is never a page for a party:
//!
//! * **Sellers**: one row for each lot posted in the scope, as R6 shows a lot (date, بکری, seller, lot
//!   name, quantity, خام بکری, the deductions, کل اخراجات, صافی بکری), with a totals row.
//! * **Buyers**: one row for each buyer: what he bought in short, his bill, نقد (paid on the spot)
//!   and ادھار (left on his account), with a totals row. A چکتا buyer is a row by his name; a buyer
//!   with two بکری in the scope is one row.
//!
//! R6 lists a lot by the date of its بکری and gives its totals over all its postings, so a lot
//! carried from an earlier بکری and sold today (PRD FR-12.48) would be missing from it while its
//! buyers are in the second table. This print reads the postings of the scope instead: the sale lines
//! of the day (as the R35 slips do) and the charges of a lot only in its first posting (PRD FR-12.17).
//! So the sellers' gross is the sale lines of the scope, and the buyers' bills are their sale lines.
//!
//! Every figure is a stored value of a sale line or a lot; the sums are made once here with checked
//! maths (Code Standards B20, M7). The buyers table is checked before it is returned: its bills are
//! the sale lines.
//!
//! Example (E2E-1 and E2E-2, 10-10-26, بکری 1): the sellers table has one row, رحیم کیلا-1, 50 نگ,
//! gross 29,400, freight 3,000, commission 1,764, labour 250, munshiana 50, نقد 5,000, phone 20,
//! deductions 10,084, net 19,316. The buyers table has اکرم 12,880 and ارشد 18,684, total 31,564,
//! نقد 0 and ادھار 31,564.

use std::collections::{BTreeMap, HashMap};

use mandi_core::bikri::lot_label;
use mandi_core::dates::{BusinessDate, format_ddmmyyyy, parse_ddmmyyyy};
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use mandi_db::Db;
use mandi_db::rusqlite::params_from_iter;
use mandi_db::rusqlite::types::Value;
use serde::Serialize;
use ts_rs::TS;

use super::activity::{SAME_DAY_PAIR, parties_for, sort_by_party};
use super::collection_sheet::numbers_of;
use super::figures_load::{BuyerLine, Filter, buyer_lines};
use crate::error::ReportsError;
use crate::print::{PrintFact, PrintFrame, PrintFrameInput, print_frame};
use crate::reports::common::{add, in_snapshot, paisa, placeholders, today as today_of};
use crate::reports::day_slips::DaySlipsInput;
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, DetailPart, Names, ReportId, ReportPage, ReportTable,
    ResolvedFilters, Row, RowKind, RowSource, SourceKind,
};
use crate::reports::print_data::{PrintOptions, PrintReportTable, ReportPrintData, build_print};

/// Fluent message ID of the title of the print.
pub const TITLE_ID: &str = "print-sale-title";
/// The heading of the buyers table and its columns (the sellers table has the columns of R6).
const BUYERS_TITLE: &str = "print-sale-buyers-title";
const BUYERS_TOTAL: &str = "print-sale-buyers-total";
const COL_BUYER: &str = "print-sale-col-buyer";
const COL_BOUGHT: &str = "print-sale-col-bought";
const COL_BILL: &str = "print-sale-col-bill";
const COL_CASH: &str = "print-sale-col-cash";
const COL_CREDIT: &str = "print-sale-col-credit";

/// The whole sale print: the header and footer, the sellers table and the buyers table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SalePrintData {
    /// The header and the footer (PRD FR-19.13).
    pub frame: PrintFrame,
    /// The paper class.
    pub paper: Paper,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The sellers table: the columns of R6, one row for each lot posted in the scope, and a totals
    /// row. Its `tables` is empty.
    pub sellers: ReportPrintData,
    /// The buyers table: one row for each buyer, and a totals row.
    pub buyers: PrintReportTable,
}

fn reject(code: ErrorCode, field: &str) -> ReportsError {
    ReportsError::Rejected(vec![FieldError::new(code, field)])
}

/// A lot as the database gives it, before the sums of its sale lines are put on it.
struct LotRaw {
    bikri_id: i64,
    number: i64,
    lot_no: i64,
    date: String,
    seller: i64,
    person: Option<String>,
    person_ur: Option<String>,
    person_en: Option<String>,
    item: Names,
    counter: i64,
    /// Freight (when the firm paid it), labour, munshiana, نقد, store rent, phone.
    charges: [i64; 6],
    first_entry: i64,
}

/// One lot posted in the scope: the sums of its sale lines of that posting, and its charges when
/// the posting is its first.
struct LotRow {
    bikri_id: i64,
    number: i64,
    date: String,
    seller: i64,
    person: Option<String>,
    person_ur: Option<String>,
    person_en: Option<String>,
    item: Names,
    counter: i64,
    qty: i64,
    gross: Paisa,
    commission: Paisa,
    freight: Paisa,
    labour: Paisa,
    munshiana: Paisa,
    cash: Paisa,
    store_rent: Paisa,
    phone: Paisa,
}

const LOT_COLUMNS: &str = "l.id, e.id, b.id, b.number, l.lot_no, e.entry_date, l.seller_account_id,
    l.chukta_person_key, l.chukta_name_ur, l.chukta_name_en, i.name_ur, i.name_en, l.lot_counter,
    CASE WHEN l.freight_paid_by = 'firm' THEN l.freight_paisa ELSE 0 END, l.labour_paisa,
    l.munshiana_paisa, l.cash_paid_paisa, l.store_rent_paisa, l.phone_paisa, l.first_entry_id";

/// The first 20 columns of [`LOT_COLUMNS`]: the lot id, the posting (entry) id and the lot.
fn lot_of(r: &mandi_db::rusqlite::Row<'_>) -> mandi_db::rusqlite::Result<(i64, i64, LotRaw)> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        LotRaw {
            bikri_id: r.get(2)?,
            number: r.get(3)?,
            lot_no: r.get(4)?,
            date: r.get(5)?,
            seller: r.get(6)?,
            person: r.get(7)?,
            person_ur: r.get(8)?,
            person_en: r.get(9)?,
            item: Names {
                ur: r.get(10)?,
                en: r.get(11)?,
            },
            counter: r.get(12)?,
            charges: [
                r.get(13)?,
                r.get(14)?,
                r.get(15)?,
                r.get(16)?,
                r.get(17)?,
                r.get(18)?,
            ],
            first_entry: r.get(19)?,
        },
    ))
}

/// The lots posted on `iso` in the scope. A lot is a row for each posting that sold it that day;
/// a lot whose first posting is that day has its charges in that row (PRD FR-12.17), even when
/// nothing of it was sold in that posting.
fn lot_rows(
    conn: &mandi_db::rusqlite::Connection,
    iso: &str,
    only: &[i64],
) -> Result<Vec<LotRow>, ReportsError> {
    let mut args: Vec<Value> = vec![Value::Text(iso.to_owned())];
    let scope = if only.is_empty() {
        String::new()
    } else {
        args.extend(only.iter().map(|id| Value::Integer(*id)));
        format!(" AND s.bikri_id IN ({})", placeholders(only.len()))
    };
    // The sale lines of the day, by posting and lot.
    let mut statement = conn.prepare(&format!(
        "SELECT {LOT_COLUMNS}, SUM(s.qty), SUM(s.seller_amount_paisa), SUM(s.seller_comm_paisa)
         FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
         CROSS JOIN bikri_sales s ON s.journal_entry_id = e.id
         CROSS JOIN bikri_lots l ON l.id = s.lot_id
         CROSS JOIN bikri_entries b ON b.id = s.bikri_id
         CROSS JOIN items i ON i.id = l.item_id
         WHERE s.is_active = 1 AND e.status = 'posted' AND e.entry_date = ?1{scope}
         GROUP BY s.journal_entry_id, s.lot_id
         ORDER BY b.number, l.lot_no, e.id"
    ))?;
    let sold = statement
        .query_map(params_from_iter(args.iter()), |r| {
            let (lot, entry, raw) = lot_of(r)?;
            Ok((
                lot,
                entry,
                raw,
                [r.get::<_, i64>(20)?, r.get(21)?, r.get(22)?],
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    // The rows in the order of the بکری, the lot and the posting.
    let mut rows: BTreeMap<(i64, i64, i64), LotRow> = BTreeMap::new();
    let mut put = |lot: i64, entry: i64, raw: LotRaw, sums: [i64; 3]| -> Result<(), ReportsError> {
        let first = raw.first_entry == entry;
        let [freight, labour, munshiana, cash, store_rent, phone] = raw.charges;
        let charge = |v: i64| if first { paisa(v) } else { Ok(Paisa::ZERO) };
        let key = (raw.number, raw.lot_no, entry);
        let row = LotRow {
            bikri_id: raw.bikri_id,
            number: raw.number,
            date: raw.date,
            seller: raw.seller,
            person: raw.person,
            person_ur: raw.person_ur,
            person_en: raw.person_en,
            item: raw.item,
            counter: raw.counter,
            qty: sums[0],
            gross: paisa(sums[1])?,
            commission: paisa(sums[2])?,
            freight: charge(freight)?,
            labour: charge(labour)?,
            munshiana: charge(munshiana)?,
            cash: charge(cash)?,
            store_rent: charge(store_rent)?,
            phone: charge(phone)?,
        };
        let _ = lot;
        rows.insert(key, row);
        Ok(())
    };
    let mut seen: Vec<(i64, i64)> = Vec::new();
    for (lot, entry, raw, sums) in sold {
        seen.push((lot, entry));
        put(lot, entry, raw, sums)?;
    }
    // A lot whose first posting is today and sold nothing in it still has its charges.
    let mut args: Vec<Value> = vec![Value::Text(iso.to_owned())];
    let scope = if only.is_empty() {
        String::new()
    } else {
        args.extend(only.iter().map(|id| Value::Integer(*id)));
        format!(" AND l.bikri_id IN ({})", placeholders(only.len()))
    };
    let mut statement = conn.prepare(&format!(
        "SELECT {LOT_COLUMNS}
         FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
         CROSS JOIN bikri_lots l ON l.first_entry_id = e.id
         CROSS JOIN bikri_entries b ON b.id = l.bikri_id
         CROSS JOIN items i ON i.id = l.item_id
         WHERE e.status = 'posted' AND e.entry_date = ?1 AND NOT ({SAME_DAY_PAIR}){scope}
         ORDER BY b.number, l.lot_no"
    ))?;
    let firsts = statement
        .query_map(params_from_iter(args.iter()), lot_of)?
        .collect::<Result<Vec<_>, _>>()?;
    for (lot, entry, raw) in firsts {
        if !seen.contains(&(lot, entry)) {
            put(lot, entry, raw, [0, 0, 0])?;
        }
    }
    Ok(rows.into_values().collect())
}

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

/// The sale print of a scope (PRD FR-19.15). `input.sections` and `input.buyer_ids` do not apply: the
/// print always holds both tables, for the whole scope.
///
/// # Errors
/// `ERR_DATE_INVALID` and `ERR_DATE_IN_FUTURE` on `date`; a database or money error;
/// `ReportsError::Data` when a table does not add up.
pub fn sale_print(db: &Db, input: &DaySlipsInput) -> Result<SalePrintData, ReportsError> {
    let grouping = db.read(crate::print::load::load_grouping)?;
    let options = PrintOptions {
        language: input.language,
        grouping,
        paper: input.paper.unwrap_or(Paper::A4),
    };
    let (day, page) = db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let text = input.date.trim();
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
            let only: Vec<i64> = match (&input.bikri_ids, input.bikri_id) {
                (Some(ids), _) if !ids.is_empty() => ids.clone(),
                (_, Some(id)) => vec![id],
                _ => Vec::new(),
            };
            let page = page_of(conn, &day.to_iso(), &only)?;
            Ok((day, page))
        })
    })?;
    let mut sellers = build_print(&page, &options)?;
    if sellers.tables.is_empty() {
        return Err(ReportsError::Data(
            "the sale print has no buyers table".to_owned(),
        ));
    }
    let buyers = sellers.tables.remove(0);
    let frame = print_frame(
        db,
        &PrintFrameInput {
            language: input.language,
            title_id: TITLE_ID.to_owned(),
            facts: vec![PrintFact {
                label_id: "print-fact-date".to_owned(),
                value: format_ddmmyyyy(day),
            }],
        },
    )?;
    Ok(SalePrintData {
        frame,
        paper: options.paper,
        date: day.to_iso(),
        sellers,
        buyers,
    })
}

/// Σ of a stored money column of the active sale lines posted on `iso` in the scope, read apart from
/// the tables: the check that a table adds up to the sale lines (Code Standards M7).
fn stored_sum(
    conn: &mandi_db::rusqlite::Connection,
    iso: &str,
    only: &[i64],
    column: &str,
) -> Result<Paisa, ReportsError> {
    let mut args: Vec<Value> = vec![Value::Text(iso.to_owned())];
    let scope = if only.is_empty() {
        String::new()
    } else {
        args.extend(only.iter().map(|id| Value::Integer(*id)));
        format!(" AND s.bikri_id IN ({})", placeholders(only.len()))
    };
    let sum: i64 = conn.query_row(
        &format!(
            "SELECT COALESCE(SUM({column}), 0)
             FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
             CROSS JOIN bikri_sales s ON s.journal_entry_id = e.id
             WHERE s.is_active = 1 AND e.status = 'posted' AND e.entry_date = ?1{scope}"
        ),
        params_from_iter(args.iter()),
        |r| r.get(0),
    )?;
    paisa(sum)
}

/// The sellers table as a report page, with the buyers table under it.
fn page_of(
    conn: &mandi_db::rusqlite::Connection,
    iso: &str,
    only: &[i64],
) -> Result<ReportPage, ReportsError> {
    let lots = lot_rows(conn, iso, only)?;
    let keys: Vec<(i64, Option<String>)> = lots.iter().map(|l| (l.seller, None)).collect();
    let parties = parties_for(conn, &keys)?;
    let mut rows = Vec::with_capacity(lots.len());
    let mut sums = [Paisa::ZERO; 10];
    let mut qty_total = 0_i64;
    for lot in &lots {
        let base = parties
            .get(&(lot.seller, None))
            .cloned()
            .ok_or_else(|| ReportsError::Data("a lot's seller has no account".to_owned()))?;
        // The chukta seller of old data has his own names on the lot, as in R6.
        let party = crate::reports::common::chukta_party(
            base,
            lot.person.clone(),
            lot.person_ur.as_deref(),
            lot.person_en.as_deref(),
        );
        let mut deductions = lot.commission;
        for part in [
            lot.freight,
            lot.labour,
            lot.munshiana,
            lot.cash,
            lot.store_rent,
            lot.phone,
        ] {
            add(&mut deductions, part)?;
        }
        let net = lot.gross.checked_sub(deductions)?;
        let money = [
            lot.gross,
            lot.freight,
            lot.commission,
            lot.labour,
            lot.munshiana,
            lot.cash,
            lot.store_rent,
            lot.phone,
            deductions,
            net,
        ];
        for (total, amount) in sums.iter_mut().zip(money) {
            add(total, amount)?;
        }
        qty_total = qty_total
            .checked_add(lot.qty)
            .ok_or_else(|| ReportsError::Data("the quantities do not fit".to_owned()))?;
        let mut cells = vec![
            Cell::Date {
                iso: lot.date.clone(),
            },
            Cell::Doc {
                type_id: ids::doc_id("bikri_first"),
                number: Some(lot.number),
            },
            Cell::Party { party },
            Cell::Names {
                names: Names {
                    ur: lot_label(&lot.item.ur, lot.counter),
                    en: lot_label(&lot.item.en, lot.counter),
                },
            },
            Cell::Number { value: lot.qty },
        ];
        cells.extend(money.map(|paisa| Cell::Money { paisa }));
        rows.push(Row {
            cells,
            source: Some(RowSource {
                kind: SourceKind::Bikri,
                id: lot.bikri_id,
            }),
            kind: RowKind::Normal,
        });
    }
    // The sellers' gross is the seller amounts stored on the sale lines of the scope.
    if stored_sum(conn, iso, only, "s.seller_amount_paisa")? != sums[0] {
        return Err(ReportsError::Data(
            "the sellers table does not add up to the sale lines".to_owned(),
        ));
    }
    let mut totals = vec![
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Number { value: qty_total },
    ];
    totals.extend(sums.map(|paisa| Cell::Money { paisa }));
    let buyers = buyers_table(conn, iso, only)?;
    Ok(ReportPage {
        group_above: None,
        tables: vec![buyers],
        id: ReportId::R6,
        title_id: ids::title_id("R6"),
        filters: ResolvedFilters {
            from: Some(iso.to_owned()),
            to: Some(iso.to_owned()),
            ..ResolvedFilters::default()
        },
        columns: crate::reports::sales::default_columns(ReportId::R6),
        total_count: i64::try_from(rows.len()).unwrap_or(i64::MAX),
        rows,
        offset: 0,
        limit: 0,
        totals_label_id: ids::TOTAL.to_owned(),
        totals,
        summary: Vec::new(),
    })
}

/// The buyers table: one row for each buyer of the scope, by name in the Urdu order.
fn buyers_table(
    conn: &mandi_db::rusqlite::Connection,
    iso: &str,
    only: &[i64],
) -> Result<ReportTable, ReportsError> {
    let lines: Vec<BuyerLine> = buyer_lines(conn, iso, iso, Filter::default())?
        .into_iter()
        .filter(|l| only.is_empty() || only.contains(&l.bikri_id))
        .collect();
    let numbers = numbers_of(conn, &lines.iter().collect::<Vec<_>>())?;
    let mut by_party: HashMap<(i64, Option<String>), Vec<&BuyerLine>> = HashMap::new();
    for line in &lines {
        let person = (!line.person.is_empty()).then(|| line.person.clone());
        by_party
            .entry((line.account_id, person))
            .or_default()
            .push(line);
    }
    let keys: Vec<(i64, Option<String>)> = by_party.keys().cloned().collect();
    let parties = parties_for(conn, &keys)?;
    let mut rows = Vec::with_capacity(keys.len());
    let (mut bill_total, mut cash_total) = (Paisa::ZERO, Paisa::ZERO);
    for key in sort_by_party(keys, &parties, |k| k.clone()) {
        let party = parties
            .get(&key)
            .cloned()
            .ok_or_else(|| ReportsError::Data("a buyer has no account".to_owned()))?;
        let (mut bill, mut cash) = (Paisa::ZERO, Paisa::ZERO);
        let mut parts = Vec::new();
        for l in by_party.get(&key).map(Vec::as_slice).unwrap_or_default() {
            add(&mut bill, l.total)?;
            add(&mut cash, l.paid)?;
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
        add(&mut bill_total, bill)?;
        add(&mut cash_total, cash)?;
        rows.push(Row {
            cells: vec![
                Cell::Party { party },
                Cell::Detail { parts },
                Cell::Money { paisa: bill },
                Cell::Money { paisa: cash },
                Cell::Money {
                    paisa: bill.checked_sub(cash)?,
                },
            ],
            source: None,
            kind: RowKind::Normal,
        });
    }
    // The table is the sale lines of the scope, read once more apart from the rows above: its bills
    // are the sums of the stored buyer totals.
    let want_bill = stored_sum(conn, iso, only, "s.buyer_total_paisa")?;
    if want_bill != bill_total {
        return Err(ReportsError::Data(
            "the buyers table does not add up to the sale lines".to_owned(),
        ));
    }
    Ok(ReportTable {
        key: "buyers".to_owned(),
        day: iso.to_owned(),
        title_id: BUYERS_TITLE.to_owned(),
        columns: vec![
            column("party", COL_BUYER, ColumnKind::Party),
            column("detail", COL_BOUGHT, ColumnKind::Detail),
            column("bill", COL_BILL, ColumnKind::Money),
            column("cash", COL_CASH, ColumnKind::Money),
            column("credit", COL_CREDIT, ColumnKind::Money),
        ],
        rows,
        totals_label_id: BUYERS_TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: bill_total },
            Cell::Money { paisa: cash_total },
            Cell::Money {
                paisa: bill_total.checked_sub(cash_total)?,
            },
        ],
    })
}
