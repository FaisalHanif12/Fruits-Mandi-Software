//! Report and analytics cells as result nodes (D6-system-prompt Section 5.1, PRD FR-7.83).
//!
//! The report engine gives typed cells; this file turns each into the typed [`Node`] the model reads
//! and cites. It also gives every row its stable key (a row is found by its record, never by its
//! place), leaves out the columns that hold what PRD FR-7.83 keeps from the model (notes, reasons,
//! narrations, phones, addresses ...), and orders cells for `sort` and `where`.
//!
//! Example: a `balance` cell of 288,000 paisa on the debit side becomes
//! `{"rs": 2880, "side": "بنام"}`; the row of a sale of بکری 4518 gets the key `je4518`.

use std::cmp::Ordering;

use mandi_reports::reports::{
    ArgValue, Cell, Column, ColumnKind, DetailPart, MessageArg, Names, PartyRef, Row, SourceKind,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::money::Paisa;

use super::Env;
use super::party::side_of;
use crate::results::{Money, NameVal, Node, Val};

/// Columns that hold what a person typed by hand or private details: never sent (PRD FR-7.83).
const LEFT_OUT_COLUMNS: [&str; 17] = [
    "narration",
    "note",
    "notes",
    "reason",
    "phone",
    "phone1",
    "phone2",
    "address",
    "father",
    "driver",
    "driver_name",
    "supplier",
    "shop",
    "market",
    "sent",
    "marka",
    "taken_by",
];

/// True for a column the model must not read. A phone NUMBER is never read (PRD FR-7.83); R6's
/// `phone_charge` is the فون charge, money the Owner asks about, and is read (lead ruling, plan 53
/// Section 9).
///
/// Example: `phone1` is left out; `phone_charge` is not.
pub fn is_left_out(column: &Column) -> bool {
    LEFT_OUT_COLUMNS.contains(&column.key.as_str())
        || (column.key.contains("phone") && column.key != PHONE_CHARGE)
        || column.key.starts_with("paid_to")
}

/// The key of the فون charge column of R6 and R1: money, not a phone number.
const PHONE_CHARGE: &str = "phone_charge";

fn money(paisa: Paisa) -> Node {
    Node::Val(Val::Money(Money::plain(paisa)))
}

fn name_of(names: &Names) -> Node {
    Node::Val(Val::Name(NameVal {
        name_ur: names.ur.clone(),
        name_en: names.en.clone(),
        tag: None,
        code: None,
    }))
}

fn party_name(party: &PartyRef) -> NameVal {
    NameVal {
        name_ur: party.name_ur.clone(),
        name_en: party.name_en.clone(),
        tag: party.tag.clone(),
        code: Some(party.code.clone()),
    }
}

fn doc(type_id: &str, number: Option<i64>) -> Node {
    Node::Val(Val::Doc {
        type_id: type_id.to_owned(),
        number,
    })
}

fn count(n: i64) -> Node {
    Node::Val(Val::Count(n))
}

fn detail_part(part: &DetailPart) -> Node {
    match part {
        DetailPart::Sale {
            bikri_number,
            item,
            packing,
            grade,
            qty,
            rate_paisa,
            value_paisa,
            commission_paisa,
            laga_paisa,
        } => {
            let mut fields = vec![
                ("kind", Node::Str("sale".to_owned())),
                ("bikri", count(*bikri_number)),
                ("item", name_of(item)),
                ("packing", name_of(packing)),
            ];
            // The quality of the line (PRD FR-12.42), as its word `first`, `second` or `third`.
            // A line with none has no `grade` field, so every older result reads as before.
            if let Some(grade) = grade {
                fields.push(("grade", Node::Str(grade.as_str().to_owned())));
            }
            fields.extend([
                ("qty", count(*qty)),
                ("rate", money(*rate_paisa)),
                ("value", money(*value_paisa)),
                ("commission", money(*commission_paisa)),
                ("laga", money(*laga_paisa)),
            ]);
            Node::obj(fields)
        }
        DetailPart::Lot {
            bikri_number,
            item,
            packing,
            qty,
            gross_paisa,
            deductions_paisa,
        } => Node::obj(vec![
            ("kind", Node::Str("lot".to_owned())),
            ("bikri", count(*bikri_number)),
            ("item", name_of(item)),
            ("packing", name_of(packing)),
            ("qty", count(*qty)),
            ("gross", money(*gross_paisa)),
            ("deductions", money(*deductions_paisa)),
        ]),
        // A reason is typed by hand: the model learns only that cash was lent (PRD FR-7.83).
        DetailPart::CashLent { .. } => Node::obj(vec![("kind", Node::Str("cash_lent".to_owned()))]),
        DetailPart::Text { .. } => Node::obj(vec![("kind", Node::Str("text".to_owned()))]),
    }
}

fn arg_node(value: &ArgValue) -> Node {
    match value {
        ArgValue::Text { text } => Node::Str(text.clone()),
        ArgValue::Number { value } => count(*value),
        ArgValue::Money { paisa } => money(*paisa),
        ArgValue::Percent { hundredths } => Node::Val(Val::Pct(*hundredths)),
        ArgValue::Names { names } => name_of(names),
        ArgValue::Message { id, args } => message_node(id, args),
    }
}

fn message_node(id: &str, args: &[MessageArg]) -> Node {
    let args = args
        .iter()
        .filter(|a| !LEFT_OUT_COLUMNS.contains(&a.name.as_str()))
        .map(|a| (a.name.clone(), arg_node(&a.value)))
        .collect();
    Node::Obj(vec![
        ("message".to_owned(), Node::Str(id.to_owned())),
        ("args".to_owned(), Node::Obj(args)),
    ])
}

/// The weight in kilos as text (a weighed line of a day slip): the model reads it but cannot cite.
fn kilos(grams: i64) -> String {
    let whole = grams.checked_div(1_000).unwrap_or(0);
    let rest = grams.checked_rem(1_000).unwrap_or(0);
    if rest == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{rest:03}")
    }
}

/// The size of a signed amount (a loss of 3,000 has the size 3,000).
pub fn magnitude(paisa: Paisa) -> Paisa {
    if paisa.is_negative() {
        paisa.checked_neg().unwrap_or(paisa)
    } else {
        paisa
    }
}

/// One cell as a node. A cell of a `difference` column (the change of a comparison) becomes a
/// difference with its direction, so the app puts زیادہ or کم next to it.
pub fn cell_node(env: &Env<'_>, cell: &Cell, as_difference: bool) -> Node {
    use mandi_services::domain::figures::Dir;
    match cell {
        Cell::Empty => Node::Null,
        Cell::Text { text } => Node::Str(text.clone()),
        Cell::Label { id } => Node::Str(env.words.text(env.language, id)),
        Cell::Number { value } => count(*value),
        Cell::Money { paisa } if as_difference => Node::Val(Val::Diff {
            size: magnitude(*paisa),
            dir: Dir::of(*paisa),
        }),
        Cell::Money { paisa } => money(*paisa),
        Cell::Balance { paisa, side } => {
            Node::Val(Val::Money(Money::balance(*paisa, side_of(*side))))
        }
        Cell::Date { iso } => Node::Val(Val::Date(iso.clone())),
        Cell::Month { year, month } => match (i32::try_from(*year), u32::try_from(*month)) {
            (Ok(year), Ok(month)) => Node::Val(Val::Month { year, month }),
            _ => Node::Null,
        },
        Cell::Percent { hundredths } => Node::Val(Val::Pct(*hundredths)),
        Cell::Weight { grams } => Node::Str(kilos(*grams)),
        Cell::Party { party } => Node::Val(Val::Name(party_name(party))),
        Cell::Names { names } => name_of(names),
        Cell::Doc { type_id, number } => doc(type_id, *number),
        Cell::Detail { parts } => Node::List(parts.iter().map(detail_part).collect()),
        Cell::Message { id, args } => message_node(id, args),
    }
}

/// The name the model reads for a column (D6-system-prompt Section 5.2): the entry of a ledger line
/// is `entry`; the first column that names the row is `name`; in analytics the quantity is `nag`.
/// Every other column keeps the engine's key.
pub fn shown_key(index: usize, column: &Column, analytics: bool) -> String {
    let names_row = matches!(column.kind, ColumnKind::Names | ColumnKind::Party);
    if index == 0 && names_row {
        "name".to_owned()
    } else if column.key == "doc" {
        "entry".to_owned()
    } else if analytics && column.key == "qty" {
        "nag".to_owned()
    } else {
        column.key.clone()
    }
}

/// The key prefix of the record a row opens.
fn source_prefix(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::Account => "a",
        SourceKind::Bikri | SourceKind::Voucher | SourceKind::Entry | SourceKind::MonthClose => {
            "je"
        }
        SourceKind::Purchase => "pu",
        SourceKind::Arrival => "ar",
        SourceKind::Venture => "v",
        SourceKind::Trip => "t",
        SourceKind::Garden => "g",
        SourceKind::Fruit => "fr",
    }
}

/// The stable key a row gets before duplicates are told apart: its record, else its first party,
/// else its month; `None` for a row that is only a place in a list.
fn record_key(row: &Row, lots: bool) -> Option<String> {
    if let Some(source) = &row.source {
        // A row of the stock report (R10) is a lot: `lt`.
        let prefix = if lots && source.kind == SourceKind::Bikri {
            "lt"
        } else {
            source_prefix(source.kind)
        };
        return Some(format!("{prefix}{}", source.id));
    }
    for cell in &row.cells {
        if let Cell::Party { party } = cell {
            return Some(format!("a{}", party.account_id));
        }
    }
    row.cells.iter().find_map(|cell| match cell {
        Cell::Month { year, month } => Some(format!("m{year}{month:02}")),
        _ => None,
    })
}

/// The key of a row, unique among `used`. Two rows of one record (a sale and its cash part) get
/// `je4518` and `je4518_2`; a row with no record is `x` and its place, a heading, month or total line `h` and
/// its place (the place is the row's place in the whole filter, not in the page).
pub fn row_key(row: &Row, place: usize, lots: bool, used: &mut Vec<String>) -> String {
    let number = place.saturating_add(1);
    let base = match row.kind {
        mandi_reports::reports::RowKind::Normal => {
            record_key(row, lots).unwrap_or_else(|| format!("x{number}"))
        }
        // A heading, a month line or an item's total line holds sums of the lines under it.
        _ => format!("h{number}"),
    };
    let mut key = base.clone();
    let mut copy: usize = 1;
    while used.contains(&key) {
        copy = copy.saturating_add(1);
        key = format!("{base}_{copy}");
    }
    used.push(key.clone());
    key
}

/// True for the key of a heading or total line (`h12`): its cells are sums of the lines under it,
/// so no sum, count or filter over the rows may take it in (money review C3).
pub fn is_total_line(key: &str) -> bool {
    key.strip_prefix('h')
        .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
}

/// True when balance cells hold both sides: their size then says nothing about "more" or "less"
/// (a buyer who owes and a buyer who is owed), so `where` and `sort` refuse such a column (money
/// review R5). A balance of 0 has no side.
pub fn mixed_sides<'a>(cells: impl IntoIterator<Item = &'a Cell>) -> bool {
    let (mut debit, mut credit) = (false, false);
    for cell in cells {
        if let Cell::Balance { paisa, side } = cell
            && paisa.as_paisa() != 0
        {
            match side {
                BalanceSide::Dr => debit = true,
                BalanceSide::Cr => credit = true,
                BalanceSide::Zero => {}
            }
        }
    }
    debit && credit
}

/// What a cell is worth when rows are sorted or filtered.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Order {
    /// No value: sorts first.
    None,
    /// A number: paisa for money (the size of a balance), the value of a count, hundredths of a
    /// percent, grams.
    Num(i128),
    /// Text: a date, a name.
    Text(String),
}

/// True for the cells that are money (their `where` value is rupees).
pub fn is_money(cell: &Cell) -> bool {
    matches!(cell, Cell::Money { .. } | Cell::Balance { .. })
}

/// The value of a cell for `sort` and `where`. A balance is its size: "the biggest dues" means the
/// biggest amount, whichever side the list is on.
pub fn order_of(cell: &Cell, language_ur: bool) -> Order {
    match cell {
        Cell::Money { paisa } | Cell::Balance { paisa, .. } => {
            Order::Num(i128::from(paisa.as_paisa()))
        }
        Cell::Number { value } => Order::Num(i128::from(*value)),
        Cell::Percent { hundredths } => Order::Num(i128::from(*hundredths)),
        Cell::Weight { grams } => Order::Num(i128::from(*grams)),
        Cell::Date { iso } => Order::Text(iso.clone()),
        Cell::Month { year, month } => Order::Num(
            i128::from(*year)
                .saturating_mul(100)
                .saturating_add(i128::from(*month)),
        ),
        Cell::Text { text } => Order::Text(text.to_lowercase()),
        // An entry (`doc`) is ordered by its number, so "newest first by date, then by entry" has a
        // second key that compares something (D7, plan 59 R1). An entry with no number sorts first.
        Cell::Doc {
            number: Some(number),
            ..
        } => Order::Num(i128::from(*number)),
        Cell::Party { party } => Order::Text(if language_ur {
            party.name_ur.clone()
        } else {
            party.name_en.to_lowercase()
        }),
        Cell::Names { names } => Order::Text(if language_ur {
            names.ur.clone()
        } else {
            names.en.to_lowercase()
        }),
        _ => Order::None,
    }
}

/// Compares two rows by a column, then (when equal) leaves them as they are.
pub fn compare_cells(a: &Cell, b: &Cell, language_ur: bool) -> Ordering {
    order_of(a, language_ur).cmp(&order_of(b, language_ur))
}

/// The signed paisa of a balance cell: بنام positive, جمع negative. Other cells have none.
pub fn signed_paisa(cell: &Cell) -> Option<i64> {
    match cell {
        Cell::Money { paisa } => Some(paisa.as_paisa()),
        Cell::Balance { paisa, side } => Some(match side {
            BalanceSide::Cr => paisa.as_paisa().checked_neg()?,
            _ => paisa.as_paisa(),
        }),
        _ => None,
    }
}
