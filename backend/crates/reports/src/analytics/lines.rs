//! The lines behind a number (PRD FR-18.2): every fact row under the filters of the run and the
//! key of the row (and the column) that was opened, one line for each sale line, each opening its
//! بکری.
//!
//! A fact written by a reversal has the sign −1. Its amounts and quantity are shown NEGATIVE, so
//! the lines add up to exactly the number that was opened (money agent review): the original
//! line, its reversal and the new line of an edit read +12,880, −12,880, +13,092. The totals row
//! is the net of the same sums as the views use ([`super::facts::groups`]); it is never summed
//! from the page.
//!
//! Example (the worked day, view A, کیلا): two lines, اکرم 20 نگ 12,000 with 720, 720, 160 and
//! ارشد 30 نگ 17,400 with 1,044, 1,044, 240; the total row 50, 29,400, 1,764, 1,764, 400, 3,928.

use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::facts::{self, Sums, run_range};
use super::filters::Typed;
use super::ids;
use super::model::{AnalyticsKey, AnalyticsLinesInput, AnalyticsLinesPage};
use super::table::{Part, part_cell};
use crate::error::ReportsError;
use crate::reports::common::{load_parties, paisa, party_of_line};
use crate::reports::grouped_sums::pukhta_names;
use crate::reports::{
    Cell, Column, ColumnKind, Names, PartyRef, Row, RowKind, RowSource, SourceKind,
};
use crate::reports::{PageSpec, common::pukhta_party};

/// Most items a key may leave out (the "other items" column of view E has fewer than 24).
const MAX_EXCLUDED_ITEMS: usize = 50;

/// Most purchases a key may name (the row of a trip with the most purchases is far below it; the
/// limit keeps the bound values of the query below the limit of SQLite).
const MAX_PURCHASES: usize = 1_000;

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

fn columns() -> Vec<Column> {
    vec![
        column("date", ids::COL_DATE, ColumnKind::Date),
        column("doc", ids::COL_DOC, ColumnKind::Doc),
        column("seller", ids::COL_SELLER, ColumnKind::Party),
        column("buyer", ids::COL_BUYER, ColumnKind::Party),
        column("item", ids::COL_ITEM, ColumnKind::Names),
        column("qty", ids::COL_QTY, ColumnKind::Number),
        column("gross", ids::COL_GROSS, ColumnKind::Money),
        column("buyer_value", ids::COL_PURCHASE_VALUE, ColumnKind::Money),
        column("seller_comm", ids::COL_SELLER_COMM, ColumnKind::Money),
        column("buyer_comm", ids::COL_BUYER_COMM, ColumnKind::Money),
        column("laga", ids::COL_LAGA, ColumnKind::Money),
        column("total_comm", ids::COL_TOTAL_COMM, ColumnKind::Money),
    ]
}

/// Puts the key of a row and the key of a column together: every id must agree, the days are
/// narrowed to the days both allow, and the excluded items are added up.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` (`column_key`) when the two name different records; `ERR_DATE_INVALID`
/// for a day that is not a date.
pub(crate) fn merge(
    row_key: &AnalyticsKey,
    column_key: Option<&AnalyticsKey>,
) -> Result<AnalyticsKey, ReportsError> {
    let mut problems = Vec::new();
    for (key, field) in [(Some(row_key), "row_key"), (column_key, "column_key")] {
        let Some(key) = key else { continue };
        for (day, name) in [(&key.from, "from"), (&key.to, "to")] {
            if let Some(day) = day
                && BusinessDate::parse_iso(day).is_none()
            {
                problems.push(FieldError::new(
                    ErrorCode::ErrDateInvalid,
                    &format!("{field}.{name}"),
                ));
            }
        }
    }
    let other = column_key.cloned().unwrap_or_default();
    // The "other items" column names at most one column less than the widest matrix; a longer
    // list is not one the views make (and would run into the limit of bound values).
    for (key, field) in [(row_key, "row_key"), (&other, "column_key")] {
        if key.not_item_ids.len() > MAX_EXCLUDED_ITEMS {
            problems.push(FieldError::new(
                ErrorCode::ErrOutOfRange,
                &format!("{field}.not_item_ids"),
            ));
        }
    }
    for (key, field) in [(row_key, "row_key"), (&other, "column_key")] {
        if key.purchase_ids.len() > MAX_PURCHASES {
            problems.push(FieldError::new(
                ErrorCode::ErrOutOfRange,
                &format!("{field}.purchase_ids"),
            ));
        }
    }
    let mut conflicts: Vec<&str> = Vec::new();
    let item_id = merge_id(
        row_key.item_id,
        other.item_id,
        "column_key.item_id",
        &mut conflicts,
    );
    let seller_id = merge_id(
        row_key.seller_id,
        other.seller_id,
        "column_key.seller_id",
        &mut conflicts,
    );
    let buyer_id = merge_id(
        row_key.buyer_id,
        other.buyer_id,
        "column_key.buyer_id",
        &mut conflicts,
    );
    let partner_id = merge_id(
        row_key.partner_id,
        other.partner_id,
        "column_key.partner_id",
        &mut conflicts,
    );
    for field in conflicts {
        problems.push(FieldError::new(ErrorCode::ErrOutOfRange, field));
    }
    let person_key = match (&row_key.person_key, &other.person_key) {
        (Some(a), Some(b)) if a != b => {
            problems.push(FieldError::new(
                ErrorCode::ErrOutOfRange,
                "column_key.person_key",
            ));
            Some(a.clone())
        }
        (a, b) => a.clone().or_else(|| b.clone()),
    };
    if !problems.is_empty() {
        return Err(ReportsError::Rejected(problems));
    }
    // `YYYY-MM-DD` sorts as a date, so the narrowest range is the later start and earlier end.
    let from = [&row_key.from, &other.from]
        .into_iter()
        .flatten()
        .max()
        .cloned();
    let to = [&row_key.to, &other.to]
        .into_iter()
        .flatten()
        .min()
        .cloned();
    let mut not_item_ids = row_key.not_item_ids.clone();
    not_item_ids.extend(other.not_item_ids.iter().copied());
    not_item_ids.sort_unstable();
    not_item_ids.dedup();
    // Both name purchases: only those both name; one names none: the other's (money agent S6).
    let mut purchase_ids: Vec<i64> = match (
        row_key.purchase_ids.is_empty(),
        other.purchase_ids.is_empty(),
    ) {
        (false, false) => row_key
            .purchase_ids
            .iter()
            .copied()
            .filter(|id| other.purchase_ids.contains(id))
            .collect(),
        (true, _) => other.purchase_ids.clone(),
        (_, true) => row_key.purchase_ids.clone(),
    };
    purchase_ids.sort_unstable();
    purchase_ids.dedup();
    Ok(AnalyticsKey {
        item_id,
        seller_id,
        buyer_id,
        person_key,
        from,
        to,
        not_item_ids,
        partner_id,
        purchase_ids,
    })
}

/// The id both keys agree on; a conflict is noted and the row's id is kept.
fn merge_id<'a>(
    a: Option<i64>,
    b: Option<i64>,
    field: &'a str,
    conflicts: &mut Vec<&'a str>,
) -> Option<i64> {
    match (a, b) {
        (Some(x), Some(y)) if x != y => {
            conflicts.push(field);
            Some(x)
        }
        (a, b) => a.or(b),
    }
}

/// A line of the page as read from the database.
struct Raw {
    date: String,
    sign: i64,
    bikri_id: i64,
    bikri_number: i64,
    posting_type: String,
    seller: i64,
    buyer: i64,
    person_key: Option<String>,
    item: Names,
    sums: Sums,
}

/// Builds the lines behind a number.
///
/// # Errors
/// As [`merge`]; a database error; `ReportsError::Data` for a fact net below 0.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    input: &AnalyticsLinesInput,
    spec: PageSpec,
) -> Result<AnalyticsLinesPage, ReportsError> {
    let key = merge(&input.row_key, input.column_key.as_ref())?;
    // View F opens the share rows of a partner, not sale facts.
    if input.view == super::model::AnalyticsView::F {
        return super::partner_lines::run(conn, typed, &key, input, spec);
    }
    // The lines behind a row of view J are the sale facts its commission was made from: under the
    // same filters, in no date range (money agent M2), so they add up to the number opened.
    let for_j = matches!(
        input.view,
        super::model::AnalyticsView::J | super::model::AnalyticsView::I
    )
    .then(|| facts::for_trips(typed));
    let (typed, range) = match &for_j {
        Some(j) => (j, facts::TRIPS_RANGE),
        None => (typed, run_range(typed)),
    };
    let conds = facts::conds(typed, range, &key);

    let total_count: i64 = {
        let sql = format!(
            "SELECT COUNT(*) FROM analytics_facts f WHERE {}",
            conds.sql()
        );
        conn.query_row(&sql, params_from_iter(conds.args().iter()), |r| r.get(0))?
    };
    let sums = facts::total(conn, &conds)?;

    let sql = format!(
        "SELECT f.entry_date, f.sign, f.bikri_id, b.number, je.posting_type, f.seller_account_id,
                f.buyer_account_id, f.pukhta_person_key, i.name_ur, i.name_en, f.qty,
                f.seller_amount_paisa, f.seller_comm_paisa, f.buyer_value_paisa,
                f.buyer_comm_paisa, f.laga_paisa
         FROM analytics_facts f
         JOIN items i ON i.id = f.item_id
         JOIN bikri_entries b ON b.id = f.bikri_id
         JOIN journal_entries je ON je.id = f.journal_entry_id
         WHERE {} ORDER BY f.entry_date, f.id LIMIT ? OFFSET ?",
        conds.sql()
    );
    let args = conds.page_args(spec.sql_limit(), spec.offset);
    let mut statement = conn.prepare(&sql)?;
    let raws = statement
        .query_map(params_from_iter(args.iter()), |r| {
            let amount = |index: usize| -> mandi_db::rusqlite::Result<i64> { r.get(index) };
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, Option<String>>(7)?,
                Names {
                    ur: r.get(8)?,
                    en: r.get(9)?,
                },
                [
                    amount(10)?,
                    amount(11)?,
                    amount(12)?,
                    amount(13)?,
                    amount(14)?,
                    amount(15)?,
                ],
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut lines = Vec::with_capacity(raws.len());
    for (date, sign, bikri_id, bikri_number, posting_type, seller, buyer, person_key, item, a) in
        raws
    {
        lines.push(Raw {
            date,
            sign,
            bikri_id,
            bikri_number,
            posting_type,
            seller,
            buyer,
            person_key,
            item,
            sums: Sums {
                qty: a[0],
                gross: paisa(a[1])?,
                seller_comm: paisa(a[2])?,
                buyer_value: paisa(a[3])?,
                buyer_comm: paisa(a[4])?,
                laga: paisa(a[5])?,
            },
        });
    }

    let mut account_ids: Vec<i64> = lines.iter().flat_map(|l| [l.seller, l.buyer]).collect();
    account_ids.sort_unstable();
    account_ids.dedup();
    let parties = load_parties(conn, &account_ids)?;
    let pukhta_ids: Vec<i64> = lines
        .iter()
        .filter(|l| l.person_key.is_some())
        .map(|l| l.buyer)
        .collect();
    let mut pukhta_ids = pukhta_ids;
    pukhta_ids.sort_unstable();
    pukhta_ids.dedup();
    let names = pukhta_names(conn, &pukhta_ids)?;
    let buyer_party = |l: &Raw| -> Result<PartyRef, ReportsError> {
        let party = party_of_line(&parties, l.buyer, l.person_key.clone())?;
        Ok(match &l.person_key {
            Some(person) => {
                let saved = names.get(person);
                pukhta_party(
                    party,
                    Some(person.clone()),
                    saved.map(|p| p.name_ur.as_str()),
                    saved.map(|p| p.name_en.as_str()),
                )
            }
            None => party,
        })
    };

    let mut rows = Vec::with_capacity(lines.len());
    for line in &lines {
        let signed = if line.sign < 0 {
            Sign::Minus
        } else {
            Sign::Plus
        };
        let total = line.sums.total_comm(typed.side)?;
        rows.push(Row {
            cells: vec![
                Cell::Date {
                    iso: line.date.clone(),
                },
                Cell::Doc {
                    type_id: crate::reports::ids::doc_id(&line.posting_type),
                    number: Some(line.bikri_number),
                },
                Cell::Party {
                    party: party_of_line(&parties, line.seller, None)?,
                },
                Cell::Party {
                    party: buyer_party(line)?,
                },
                Cell::Names {
                    names: line.item.clone(),
                },
                Cell::Number {
                    value: signed.number(line.sums.qty)?,
                },
                Cell::Money {
                    paisa: signed.money(line.sums.gross)?,
                },
                Cell::Money {
                    paisa: signed.money(line.sums.buyer_value)?,
                },
                signed.part(typed, Part::Seller, line.sums.seller_comm)?,
                signed.part(typed, Part::Buyer, line.sums.buyer_comm)?,
                signed.part(typed, Part::Buyer, line.sums.laga)?,
                Cell::Money {
                    paisa: signed.money(total)?,
                },
            ],
            source: Some(RowSource {
                kind: SourceKind::Bikri,
                id: line.bikri_id,
            }),
            kind: RowKind::Normal,
        });
    }

    let total_comm = sums.total_comm(typed.side)?;
    let totals = vec![
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Number { value: sums.qty },
        Cell::Money { paisa: sums.gross },
        Cell::Money {
            paisa: sums.buyer_value,
        },
        part_cell(typed.side, Part::Seller, sums.seller_comm),
        part_cell(typed.side, Part::Buyer, sums.buyer_comm),
        part_cell(typed.side, Part::Buyer, sums.laga),
        Cell::Money { paisa: total_comm },
    ];
    Ok(AnalyticsLinesPage {
        view: input.view,
        title_id: ids::LINES_TITLE.to_owned(),
        filters: typed.resolved.clone(),
        columns: columns(),
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals,
    })
}

/// The sign of a fact line: a reversal row (−1) shows its amounts negative.
#[derive(Clone, Copy)]
enum Sign {
    Plus,
    Minus,
}

impl Sign {
    fn number(self, value: i64) -> Result<i64, ReportsError> {
        match self {
            Sign::Plus => Ok(value),
            Sign::Minus => value
                .checked_neg()
                .ok_or_else(|| ReportsError::Data("a quantity does not fit".to_owned())),
        }
    }

    fn money(self, amount: Paisa) -> Result<Paisa, ReportsError> {
        match self {
            Sign::Plus => Ok(amount),
            Sign::Minus => Ok(amount.checked_neg()?),
        }
    }

    fn part(self, typed: &Typed, part: Part, amount: Paisa) -> Result<Cell, ReportsError> {
        Ok(match part_cell(typed.side, part, amount) {
            Cell::Money { paisa } => Cell::Money {
                paisa: self.money(paisa)?,
            },
            other => other,
        })
    }
}
