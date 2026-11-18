//! The print and the sheet of the commission page (plan 76 point 7, plan 77, PRD FR-9.3,
//! FR-19.3): the same table the screen shows, with every row of the period, so the print and the
//! sheet agree with the screen.
//!
//! The header names the period and, under a party, his side and himself (PRD FR-19.10, FR-19.13).
//! The partner columns are only there for the whole shop and only while the partner section is
//! open (PRD FR-17.11).

use mandi_core::auth::Language;
use mandi_core::dates::format_ddmmyyyy;
use mandi_db::Db;

use super::model::{
    CommissionDayRow, CommissionMonth, CommissionMonthInput, CommissionPartyShown,
    CommissionPeriodKind, CommissionPrintInput, CommissionSide,
};
use super::read::commission_every_row;
use crate::analytics::AnalyticsGoods;
use crate::analytics::print::goods_fact;
use crate::error::ReportsError;
use crate::print::PrintFact;
use crate::reports::{
    Cell, Column, ColumnKind, Names, Row, RowKind, TablePrintData, TableSpec, table_print_data,
};

/// The print of a period: every day (or month) with a figure, the partner columns only when
/// `show_partners` (the partner section is open) and the period totals row.
///
/// # Errors
/// As [`super::commission_month`]; a database error.
pub fn commission_month_print_data(
    db: &Db,
    input: &CommissionPrintInput,
    show_partners: bool,
) -> Result<TablePrintData, ReportsError> {
    let spec = commission_month_spec(db, input, show_partners)?;
    table_print_data(db, input.language, input.paper, spec)
}

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

/// The party in the print language: his name with the tag, and his code unless he is a چکتا buyer.
fn party_text(shown: &CommissionPartyShown, language: Language) -> String {
    let party = &shown.party;
    let name = match language {
        Language::Ur => &party.name_ur,
        Language::En => &party.name_en,
    };
    let mut text = name.clone();
    if let Some(tag) = &party.tag {
        text.push(' ');
        text.push_str(tag);
    }
    if party.person_key.is_none() {
        text.push_str(&format!(" ({})", party.code));
    }
    text
}

/// The facts under the title: the period, and the party when there is one.
fn facts_of(month: &CommissionMonth, language: Language) -> Vec<PrintFact> {
    let fact = |id: &str, value: String| PrintFact {
        label_id: id.to_owned(),
        value,
    };
    let iso = |text: &str| {
        mandi_core::dates::BusinessDate::parse_iso(text)
            .map(format_ddmmyyyy)
            .unwrap_or_default()
    };
    let (y, m) = month.month.split_once('-').unwrap_or(("", ""));
    let mut facts = match month.period.kind {
        CommissionPeriodKind::Month => vec![fact("print-fact-month", format!("{m}-{y}"))],
        CommissionPeriodKind::Day => vec![fact("print-fact-date", iso(&month.period.from))],
        CommissionPeriodKind::Year => vec![fact("print-fact-year", month.year.to_string())],
        CommissionPeriodKind::Range => vec![
            fact("print-fact-from", iso(&month.period.from)),
            fact("print-fact-to", iso(&month.period.to)),
        ],
    };
    if let Some(shown) = &month.party {
        let label = match shown.side {
            CommissionSide::Seller => "print-fact-seller",
            CommissionSide::Buyer => "print-fact-buyer",
        };
        facts.push(fact(label, party_text(shown, language)));
    }
    // The goods say it all in their label, so their value is empty (as the analytics header).
    facts.extend(goods_fact(month.goods));
    facts
}

/// The table of a period for print and Excel: the same cells the screen shows, in the order the
/// print and the sheet have them.
///
/// # Errors
/// As [`super::commission_month`]; a database error.
pub fn commission_month_spec(
    db: &Db,
    input: &CommissionPrintInput,
    show_partners: bool,
) -> Result<TableSpec, ReportsError> {
    let mut month = commission_every_row(
        db,
        &CommissionMonthInput {
            month: input.month.clone(),
            period: input.period.clone(),
            party: input.party.clone(),
            goods: input.goods,
            offset: None,
            limit: None,
        },
    )?;
    if !show_partners {
        month.hide_partners();
    }
    let by_month = month.period.kind == CommissionPeriodKind::Year;
    let date_kind = if by_month {
        ColumnKind::Month
    } else {
        ColumnKind::Date
    };
    let mut columns = vec![column("date", "print-commission-col-date", date_kind)];
    // What each shape prints, in this order.
    enum Part {
        Sale,
        Seller,
        Buyer,
        Laga,
        Total,
        Goods,
        Shared,
    }
    let parts: &[Part] = match month.party.as_ref().map(|p| p.side) {
        // Shared income is of whole postings: not shown under a goods filter (plan 78 D4).
        None if month.goods != AnalyticsGoods::All => {
            &[Part::Seller, Part::Buyer, Part::Laga, Part::Total]
        }
        None => &[
            Part::Seller,
            Part::Buyer,
            Part::Laga,
            Part::Total,
            Part::Shared,
        ],
        Some(CommissionSide::Seller) => &[Part::Sale, Part::Seller, Part::Goods],
        Some(CommissionSide::Buyer) => &[Part::Sale, Part::Buyer, Part::Laga, Part::Total],
    };
    let is_buyer = month.party.as_ref().map(|p| p.side) == Some(CommissionSide::Buyer);
    for part in parts {
        columns.push(match part {
            Part::Sale if is_buyer => {
                column("sale", "print-commission-col-purchase", ColumnKind::Money)
            }
            Part::Sale => column("sale", "print-commission-col-sale", ColumnKind::Money),
            Part::Seller => column(
                "seller_comm",
                "print-commission-col-seller",
                ColumnKind::Money,
            ),
            Part::Buyer => column(
                "buyer_comm",
                "print-commission-col-buyer",
                ColumnKind::Money,
            ),
            Part::Laga => column("laga", "print-commission-col-laga", ColumnKind::Money),
            Part::Total => column(
                "total_comm",
                "print-commission-col-total",
                ColumnKind::Money,
            ),
            Part::Goods => column(
                "goods_comm",
                "print-commission-col-goods",
                ColumnKind::Money,
            ),
            Part::Shared => column(
                "shared_income",
                "print-commission-col-shared",
                ColumnKind::Money,
            ),
        });
    }
    let mut names: Vec<Option<Names>> = vec![None; columns.len()];
    if show_partners && month.party.is_none() {
        for p in &month.partners {
            columns.push(column(
                &format!("partner_{}", p.id),
                "print-commission-col-partner",
                ColumnKind::Money,
            ));
            names.push(Some(Names {
                ur: p.name_ur.clone(),
                en: p.name_en.clone(),
            }));
        }
    }
    let cells = |row: &CommissionDayRow, date: Cell| -> Vec<Cell> {
        let money = |paisa| Cell::Money { paisa };
        let mut cells = vec![date];
        for part in parts {
            cells.push(match part {
                Part::Sale => money(row.sale_paisa.unwrap_or_default()),
                Part::Seller => money(row.seller_comm_paisa),
                Part::Buyer => money(row.buyer_comm_paisa),
                Part::Laga => money(row.laga_paisa),
                Part::Total => money(row.total_comm_paisa),
                Part::Goods => money(row.goods_comm_paisa.unwrap_or_default()),
                Part::Shared => money(row.shared_income_paisa),
            });
        }
        if let Some(shares) = &row.partner_shares {
            cells.extend(shares.iter().map(|s| money(s.share_paisa)));
        }
        cells
    };
    let date_cell = |key: &str| -> Cell {
        if by_month {
            let (year, mm) = key.split_once('-').unwrap_or(("", ""));
            Cell::Month {
                year: year.parse().unwrap_or_default(),
                month: mm.parse().unwrap_or_default(),
            }
        } else {
            Cell::Date {
                iso: key.to_owned(),
            }
        }
    };
    let rows: Vec<Row> = month
        .rows
        .iter()
        .map(|r| Row {
            cells: cells(r, date_cell(&r.date)),
            source: None,
            kind: RowKind::Normal,
        })
        .collect();
    let count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let title = if month.period.kind == CommissionPeriodKind::Month {
        "print-commission-title"
    } else {
        "print-commission-title-period"
    };
    Ok(TableSpec {
        title_id: title.to_owned(),
        facts: facts_of(&month, input.language),
        columns,
        column_names: names,
        totals: cells(&month.totals, Cell::Empty),
        rows,
        totals_label_id: "print-commission-total".to_owned(),
        total_count: count,
        words_of: None,
        plain_sale: false,
        extra_totals: Vec::new(),
    })
}
