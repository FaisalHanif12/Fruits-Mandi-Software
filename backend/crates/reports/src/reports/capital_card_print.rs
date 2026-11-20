//! The print of the card "where the capital is" (PRD FR-16.14 v1.50, FR-19.10, FR-19.13, plan 87).
//!
//! The card is read once by `services::capital_card` (the same read as the screen, Code Standards B5)
//! and this module only lays it out as a table of typed cells: the have block with its subtotal, the
//! owe block with its subtotal, صافی, whose it is, and the warning of ابتدائی فرق with its breakdown.
//! No figure is worked out here: every amount, side and share is the card's. Money is written by the
//! shared print formatter, so digits are 0 to 9 and the header and footer are the shared print frame.
//!
//! The words come from Fluent ids that start with `print-capital-` (listed in `ids` below); the
//! language files are the frontend's.
//!
//! Example (after Events 1 to 5): have 58,484 + 21,564 = 80,048; owe 10,160 + 300 + 4,000 + 100 =
//! 14,560; صافی 65,488; partners −1,004 (owes the firm), 746, 746 and ابتدائی سرمایہ 65,000.

use mandi_core::accounts::BalanceSide;
use mandi_core::auth::Language;
use mandi_core::cashbook::{
    CapitalDifference, CapitalDifferenceKind, CapitalDifferenceSide, CapitalSide, CapitalWhere,
    CapitalWhereKind, CapitalWhereLine, CapitalWhoseKind, CapitalWhoseLine,
};
use mandi_core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use mandi_db::Db;

use super::{
    Cell, Column, ColumnKind, Names, Row, RowKind, TablePrintData, TableSpec, table_print_data,
};
use crate::error::ReportsError;
use crate::print::PrintFact;

/// The Fluent ids this print asks for. The frontend writes each in Urdu and in English.
pub mod ids {
    /// The title of the print.
    pub const TITLE: &str = "print-capital-title";
    /// The fact under the title: the day the card is read at (the shared label "Date").
    pub const FACT_DATE: &str = "print-fact-date";
    /// Column headings.
    pub const COL_ITEM: &str = "print-capital-col-item";
    /// The amount column.
    pub const COL_AMOUNT: &str = "print-capital-col-amount";
    /// The share column (whose it is).
    pub const COL_SHARE: &str = "print-capital-col-share";
    /// Heading of the have block (لینا ہے / پاس ہے).
    pub const HAVE_TITLE: &str = "print-capital-have-title";
    /// Subtotal of the have block.
    pub const HAVE_TOTAL: &str = "print-capital-have-total";
    /// Heading of the owe block (دینا ہے).
    pub const OWE_TITLE: &str = "print-capital-owe-title";
    /// Subtotal of the owe block.
    pub const OWE_TOTAL: &str = "print-capital-owe-total";
    /// صافی, when the shop holds more than it owes.
    pub const NET: &str = "print-capital-net";
    /// صافی, when the shop holds less than it owes (a shortfall).
    pub const NET_SHORT: &str = "print-capital-net-short";
    /// Heading of whose it is.
    pub const WHOSE_TITLE: &str = "print-capital-whose-title";
    /// The line of ابتدائی سرمایہ.
    pub const WHOSE_OPENING_CAPITAL: &str = "print-capital-whose-opening-capital";
    /// The line of the open advance parts.
    pub const WHOSE_ADVANCES_OPEN: &str = "print-capital-whose-advances-open";
    /// Any other difference of the right side (it shows a bug).
    pub const WHOSE_OTHER: &str = "print-capital-whose-other";
    /// The totals row: the partners' total capital.
    pub const TOTAL: &str = "print-capital-total";
    /// The totals row when the net is a shortfall.
    pub const TOTAL_SHORT: &str = "print-capital-total-short";
    /// Heading of the warning of ابتدائی فرق.
    pub const DIFF_TITLE: &str = "print-capital-diff-title";
    /// A group of the warning: a partner's opening capital.
    pub const DIFF_PARTNER_CAPITAL: &str = "print-capital-diff-partner-capital";
    /// A group of the warning: the openings of other accounts.
    pub const DIFF_ACCOUNT_OPENING: &str = "print-capital-diff-account-opening";
    /// A group of the warning: an entry that is not an opening.
    pub const DIFF_OTHER_ENTRY: &str = "print-capital-diff-other-entry";
}

fn line_id(kind: CapitalWhereKind, side: CapitalSide) -> &'static str {
    match (kind, side) {
        (CapitalWhereKind::Cash, CapitalSide::Have) => "print-capital-line-cash",
        // Cash below zero is a line of the owe block, worded "cash short", never a minus figure.
        (CapitalWhereKind::Cash, CapitalSide::Owe) => "print-capital-line-cash-short",
        (CapitalWhereKind::BuyersOwe, _) => "print-capital-line-buyers-owe",
        (CapitalWhereKind::BuyerCredits, _) => "print-capital-line-buyer-credits",
        (CapitalWhereKind::DeadAmount, _) => "print-capital-line-dead-amount",
        (CapitalWhereKind::SellerAdvances, _) => "print-capital-line-seller-advances",
        (CapitalWhereKind::SellersOweBeyond, _) => "print-capital-line-sellers-owe-beyond",
        (CapitalWhereKind::SellersPayable, _) => "print-capital-line-sellers-payable",
        (CapitalWhereKind::MoneyLent, _) => "print-capital-line-money-lent",
        // A charge line names its account; this id is the fallback when it has none.
        (CapitalWhereKind::OwedForCharges, _) => "print-capital-line-owed-for-charges",
        (CapitalWhereKind::SeparateKhatas, _) => "print-capital-line-separate-khatas",
        (CapitalWhereKind::GardensHeld, _) => "print-capital-line-gardens-held",
        (CapitalWhereKind::LinkedInPartnerTotals, _) => "print-capital-line-linked",
        (CapitalWhereKind::Other, _) => "print-capital-line-other",
    }
}

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

fn label(id: &str) -> Cell {
    Cell::Label { id: id.to_owned() }
}

fn names(ur: &str, en: &str) -> Cell {
    Cell::Names {
        names: Names {
            ur: ur.to_owned(),
            en: en.to_owned(),
        },
    }
}

/// A row of three cells: the item, the amount and the share.
fn row(kind: RowKind, item: Cell, amount: Cell, share: Cell) -> Row {
    Row {
        cells: vec![item, amount, share],
        source: None,
        kind,
    }
}

fn dr_cr(side: CapitalDifferenceSide) -> BalanceSide {
    match side {
        CapitalDifferenceSide::Dr => BalanceSide::Dr,
        CapitalDifferenceSide::Cr => BalanceSide::Cr,
    }
}

fn block_rows(
    rows: &mut Vec<Row>,
    (title, total_id): (&str, &str),
    lines: &[CapitalWhereLine],
    total: Paisa,
) {
    rows.push(row(RowKind::Group, label(title), Cell::Empty, Cell::Empty));
    for line in lines {
        let item = match &line.account {
            // A charge line is one account: labour, freight, the munshi, a store.
            Some(account) => names(&account.name_ur, &account.name_en),
            None => label(line_id(line.kind, line.side)),
        };
        rows.push(row(
            RowKind::Normal,
            item,
            Cell::Money {
                paisa: line.amount_paisa,
            },
            Cell::Empty,
        ));
    }
    rows.push(row(
        RowKind::Group,
        label(total_id),
        Cell::Money { paisa: total },
        Cell::Empty,
    ));
}

fn whose_row(line: &CapitalWhoseLine) -> Row {
    let item = match (line.kind, &line.partner) {
        (CapitalWhoseKind::PartnerTotal, Some(p)) => names(&p.name_ur, &p.name_en),
        (CapitalWhoseKind::PartnerTotal, None) => label(ids::WHOSE_OTHER),
        (CapitalWhoseKind::OpeningCapital, _) => label(ids::WHOSE_OPENING_CAPITAL),
        (CapitalWhoseKind::AdvancesOpen, _) => label(ids::WHOSE_ADVANCES_OPEN),
        (CapitalWhoseKind::Other, _) => label(ids::WHOSE_OTHER),
    };
    // A partner below 0 owes the firm: his figure is written as an amount with its side (بنام), not
    // as a minus sign.
    let amount = if line.amount_paisa.is_negative() {
        Cell::Balance {
            paisa: line.amount_paisa.checked_neg().unwrap_or(line.amount_paisa),
            side: BalanceSide::Dr,
        }
    } else {
        Cell::Money {
            paisa: line.amount_paisa,
        }
    };
    let share = line.share_bps.map_or(Cell::Empty, |bps| Cell::Percent {
        hundredths: i64::from(bps),
    });
    row(RowKind::Normal, item, amount, share)
}

fn difference_rows(rows: &mut Vec<Row>, difference: &CapitalDifference) {
    rows.push(row(
        RowKind::Group,
        label(ids::DIFF_TITLE),
        Cell::Balance {
            paisa: difference.amount_paisa,
            side: dr_cr(difference.side),
        },
        Cell::Empty,
    ));
    for group in &difference.groups {
        let id = match group.kind {
            CapitalDifferenceKind::PartnerCapital => ids::DIFF_PARTNER_CAPITAL,
            CapitalDifferenceKind::AccountOpening => ids::DIFF_ACCOUNT_OPENING,
            CapitalDifferenceKind::OtherEntry => ids::DIFF_OTHER_ENTRY,
        };
        rows.push(row(
            RowKind::Group,
            label(id),
            Cell::Balance {
                paisa: group.amount_paisa,
                side: dr_cr(group.side),
            },
            Cell::Empty,
        ));
        for part in &group.parts {
            let item = match (&part.partner, &part.account) {
                (Some(p), _) => names(&p.name_ur, &p.name_en),
                (None, Some(a)) => names(&a.name_ur, &a.name_en),
                (None, None) => label(id),
            };
            rows.push(row(
                RowKind::Normal,
                item,
                Cell::Balance {
                    paisa: part.amount_paisa,
                    side: dr_cr(part.side),
                },
                Cell::Empty,
            ));
        }
    }
}

/// The card laid out as a table: the rows, the totals row and the date fact.
pub fn capital_card_spec(card: &CapitalWhere) -> TableSpec {
    let columns = vec![
        column("item", ids::COL_ITEM, ColumnKind::Label),
        column("amount", ids::COL_AMOUNT, ColumnKind::Money),
        column("share", ids::COL_SHARE, ColumnKind::Percent),
    ];
    let mut rows: Vec<Row> = Vec::new();
    block_rows(
        &mut rows,
        (ids::HAVE_TITLE, ids::HAVE_TOTAL),
        &card.have_lines,
        card.have_total_paisa,
    );
    block_rows(
        &mut rows,
        (ids::OWE_TITLE, ids::OWE_TOTAL),
        &card.owe_lines,
        card.owe_total_paisa,
    );
    let short = card.net_side == CapitalSide::Owe;
    rows.push(row(
        RowKind::Group,
        label(if short { ids::NET_SHORT } else { ids::NET }),
        Cell::Money {
            paisa: card.net_paisa,
        },
        Cell::Empty,
    ));
    rows.push(row(
        RowKind::Group,
        label(ids::WHOSE_TITLE),
        Cell::Empty,
        Cell::Empty,
    ));
    rows.extend(card.whose_lines.iter().map(whose_row));
    if let Some(difference) = &card.difference {
        difference_rows(&mut rows, difference);
    }
    let date = BusinessDate::parse_iso(&card.as_of)
        .map(format_ddmmyyyy)
        .unwrap_or_else(|| card.as_of.clone());
    TableSpec {
        title_id: ids::TITLE.to_owned(),
        facts: vec![PrintFact {
            label_id: ids::FACT_DATE.to_owned(),
            value: date,
        }],
        column_names: vec![None; columns.len()],
        columns,
        total_count: i64::try_from(rows.len()).unwrap_or(i64::MAX),
        rows,
        totals_label_id: if short { ids::TOTAL_SHORT } else { ids::TOTAL }.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Money {
                paisa: card.whose_total_paisa,
            },
            Cell::Empty,
        ],
        words_of: None,
        plain_sale: false,
        extra_totals: Vec::new(),
    }
}

/// The print of the card in the print language.
///
/// # Errors
/// A database error (the print frame reads the firm settings).
pub fn capital_card_print_data(
    db: &Db,
    language: Language,
    paper: Option<Paper>,
    card: &CapitalWhere,
) -> Result<TablePrintData, ReportsError> {
    table_print_data(db, language, paper, capital_card_spec(card))
}
