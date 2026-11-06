//! The day slip of one party (R35): a buyer's or a seller's (plan 42, PRD FR-19.12 v1.12).
//!
//! The goods table lists every sale line of the day with its item, packing, qty, rate and amount;
//! lines of one item stay together and an item with more than one line has a total row. Below it
//! the labelled lines of the PRD, in the PRD order. Every figure comes from
//! [`figures`](super::figures); this file only turns them into cells. A line that can be negative
//! is a balance with its side; the identity `new = previous + left`, and `left` from its parts, is
//! checked once more on the cells that are printed (money agent, round 1, point 2f).

use super::activity::net_cell;
use super::figures::{BuyerDay, SellerDay, Who, load};
use super::figures_load::Goods;
use super::*;
use crate::reports::model::{ArgValue, Column, MessageArg};
use mandi_core::bikri::SaleGrade;

/// The columns of a goods table: item, packing, qty, rate and amount, and after them, only when
/// some line is of an item sold by weight, the weight in kilos (plan 42, PRD A73), and only when
/// some line has a quality, the quality (plan 58, PRD FR-12.42). The rate of a
/// weighed line is for one kilo and its amount is the stored value (weight × rate, C1), so the
/// screen writes "12.345 kg @ 100 = 1,235" from the weight, rate and amount cells. The quality
/// column is last, so no other column moves; the screen writes its word after the item. A slip
/// with no quality has exactly the columns it had before.
pub(super) fn slip_columns(with_weight: bool, with_grade: bool) -> Vec<Column> {
    let mut columns = vec![
        col("item", ids::COL_ITEM, ColumnKind::Names),
        col("packing", ids::COL_PACKING, ColumnKind::Names),
        col("qty", ids::COL_QTY, ColumnKind::Number),
        col("rate", ids::COL_RATE, ColumnKind::Money),
        col("amount", ids::COL_AMOUNT, ColumnKind::Money),
    ];
    if with_weight {
        columns.push(col("weight", ids::COL_WEIGHT, ColumnKind::Weight));
    }
    if with_grade {
        columns.push(col("grade", ids::COL_GRADE, ColumnKind::Label));
    }
    columns
}

/// One line of a goods table, from a buyer's sale line or a seller's.
struct GoodsLine<'a> {
    bikri_id: i64,
    goods: &'a Goods,
    qty: i64,
    weight_grams: Option<i64>,
    grade: Option<SaleGrade>,
    rate: Paisa,
    amount: Paisa,
}

/// The cell of the quality column: its word, or nothing.
fn grade_cell(grade: Option<SaleGrade>) -> Cell {
    grade.map_or(Cell::Empty, |grade| Cell::Label {
        id: ids::grade_id(grade).to_owned(),
    })
}

fn weight_cell(grams: Option<i64>) -> Cell {
    grams.map_or(Cell::Empty, |grams| Cell::Weight { grams })
}

/// The rows of a goods table: lines grouped by item (in the order the items first appear), inside
/// an item by packing (in the order they first appear), inside a packing in the order they were
/// saved. After an item that has more than one line, a total row with its qty and amount (and
/// its weight, when it has weighed lines). With `with_weight` each row has the weight cell, and
/// with `with_grade` the quality cell after it, last.
fn goods_rows(
    lines: &[GoodsLine<'_>],
    with_weight: bool,
    with_grade: bool,
) -> Result<Vec<Row>, ReportsError> {
    let mut items: Vec<i64> = Vec::new();
    for line in lines {
        if !items.contains(&line.goods.item_id) {
            items.push(line.goods.item_id);
        }
    }
    let mut rows = Vec::with_capacity(lines.len());
    for item_id in items {
        let of_item: Vec<&GoodsLine<'_>> = lines
            .iter()
            .filter(|l| l.goods.item_id == item_id)
            .collect();
        let mut packings: Vec<i64> = Vec::new();
        for line in &of_item {
            if !packings.contains(&line.goods.packing_id) {
                packings.push(line.goods.packing_id);
            }
        }
        let (mut qty, mut amount) = (0_i64, Paisa::ZERO);
        let mut grams: Option<i64> = None;
        let mut count = 0_usize;
        let mut item_names: Option<Names> = None;
        for packing_id in packings {
            for line in of_item.iter().filter(|l| l.goods.packing_id == packing_id) {
                qty = qty
                    .checked_add(line.qty)
                    .ok_or_else(|| ReportsError::Data("the quantities do not fit".to_owned()))?;
                add(&mut amount, line.amount)?;
                if let Some(g) = line.weight_grams {
                    grams =
                        Some(grams.unwrap_or(0).checked_add(g).ok_or_else(|| {
                            ReportsError::Data("the weights do not fit".to_owned())
                        })?);
                }
                count = count.saturating_add(1);
                item_names.get_or_insert_with(|| line.goods.item.clone());
                let mut cells = vec![
                    Cell::Names {
                        names: line.goods.item.clone(),
                    },
                    Cell::Names {
                        names: line.goods.packing.clone(),
                    },
                    Cell::Number { value: line.qty },
                    Cell::Money { paisa: line.rate },
                    Cell::Money { paisa: line.amount },
                ];
                if with_weight {
                    cells.push(weight_cell(line.weight_grams));
                }
                if with_grade {
                    cells.push(grade_cell(line.grade));
                }
                rows.push(Row {
                    cells,
                    source: Some(RowSource {
                        kind: SourceKind::Bikri,
                        id: line.bikri_id,
                    }),
                    kind: RowKind::Normal,
                });
            }
        }
        if count > 1
            && let Some(item) = item_names
        {
            let mut cells = vec![
                Cell::Message {
                    id: ids::R35_ITEM_TOTAL.to_owned(),
                    args: vec![MessageArg {
                        name: "item".to_owned(),
                        value: ArgValue::Names { names: item },
                    }],
                },
                Cell::Empty,
                Cell::Number { value: qty },
                Cell::Empty,
                Cell::Money { paisa: amount },
            ];
            if with_weight {
                cells.push(weight_cell(grams));
            }
            if with_grade {
                cells.push(Cell::Empty);
            }
            rows.push(Row {
                cells,
                source: None,
                kind: RowKind::ItemTotal,
            });
        }
    }
    Ok(rows)
}

fn money(paisa: Paisa) -> Cell {
    Cell::Money { paisa }
}

/// The signed value of a printed cell: a debit is positive, a credit negative. Used for the
/// identity check on what is printed.
pub(super) fn signed(cell: &Cell) -> Result<Paisa, ReportsError> {
    use mandi_core::accounts::BalanceSide;
    match cell {
        Cell::Empty => Ok(Paisa::ZERO),
        Cell::Money { paisa } => Ok(*paisa),
        Cell::Balance { paisa, side } => match side {
            BalanceSide::Dr => Ok(*paisa),
            BalanceSide::Cr => Ok(Paisa::ZERO.checked_sub(*paisa)?),
            BalanceSide::Zero => Ok(Paisa::ZERO),
        },
        _ => Err(ReportsError::Data(
            "a slip line is not an amount".to_owned(),
        )),
    }
}

fn lacking() -> ReportsError {
    ReportsError::Data("a slip is missing one of its lines".to_owned())
}

/// The buyer's lines, in the PRD order.
fn buyer_summary(d: &BuyerDay) -> Result<Vec<SummaryLine>, ReportsError> {
    let previous = net_cell(d.previous)?;
    let other = net_cell(d.other)?;
    // Left empty when 0: the collector writes what he is paid.
    let paid = blank_if_zero(d.paid);
    let left = net_cell(d.left)?;
    let new = net_cell(d.new)?;
    // The identity, on the cells that are printed.
    let want_left = d
        .total
        .checked_add(signed(&other)?)?
        .checked_sub(signed(&paid)?)?;
    let want_new = signed(&previous)?.checked_add(signed(&left)?)?;
    // C5 on what is printed: goods + commission + لاگا is today's total.
    let want_total = d.goods.checked_add(d.commission)?.checked_add(d.laga)?;
    if signed(&left)? != want_left || signed(&new)? != want_new || want_total != d.total {
        return Err(ReportsError::Data(
            "the lines of the buyer's slip do not add up".to_owned(),
        ));
    }
    let mut out = vec![
        line(ids::R35_GOODS, money(d.goods)),
        line(ids::R35_COMMISSION, money(d.commission)),
        line(ids::R35_LAGA, money(d.laga)),
        line(ids::R35_TODAY_TOTAL, money(d.total)),
    ];
    if d.has_other {
        out.push(line(ids::R35_OTHER, other));
    }
    out.push(line(ids::R35_PAID_TODAY, paid));
    out.push(line(ids::R35_TODAY_LEFT, left));
    out.push(line(ids::R35_PREVIOUS, previous));
    out.push(line(ids::R35_NEW_BALANCE, new));
    out.push(line(ids::R35_TOTAL_QTY, Cell::Number { value: d.qty }));
    Ok(out)
}

/// The seller's lines, in the PRD order. A deduction is shown only when it is not 0.
fn seller_summary(d: &SellerDay) -> Result<Vec<SummaryLine>, ReportsError> {
    let previous = net_cell(d.previous)?;
    let sale = net_cell(d.sale)?;
    let other = net_cell(d.other)?;
    let paid = blank_if_zero(d.paid);
    let left = net_cell(d.left)?;
    let new = net_cell(d.new)?;
    let want_left = signed(&sale)?
        .checked_add(signed(&other)?)?
        .checked_add(signed(&paid)?)?;
    let want_new = signed(&previous)?.checked_add(signed(&left)?)?;
    // The lines above the net sale must give it: every deduction shown, less the gross.
    let deduction_lines = [
        (ids::R35_COMMISSION, d.commission),
        (ids::R35_FREIGHT, d.freight),
        (ids::R35_LABOUR, d.labour),
        (ids::R35_MUNSHIANA, d.munshiana),
        (ids::R35_PHONE, d.phone),
        (ids::R35_STORE_RENT, d.store_rent),
    ];
    let mut deductions = Paisa::ZERO;
    for (_, amount) in &deduction_lines {
        add(&mut deductions, *amount)?;
    }
    let want_sale = deductions.checked_sub(d.gross)?;
    if signed(&left)? != want_left || signed(&new)? != want_new || signed(&sale)? != want_sale {
        return Err(ReportsError::Data(
            "the lines of the seller's slip do not add up".to_owned(),
        ));
    }
    let mut out = vec![line(ids::R35_GROSS, money(d.gross))];
    for (label, amount) in deduction_lines {
        if amount != Paisa::ZERO {
            out.push(line(label, money(amount)));
        }
    }
    out.push(line(ids::R35_NET_SALE, sale));
    if d.has_other {
        out.push(line(ids::R35_OTHER, other));
    }
    out.push(line(ids::R35_PAID_TODAY, paid));
    out.push(line(ids::R35_TODAY_LEFT, left));
    out.push(line(ids::R35_PREVIOUS, previous));
    out.push(line(ids::R35_NEW_BALANCE, new));
    Ok(out)
}

/// The slip of one party.
pub(super) fn slip(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    day: &str,
    account: i64,
) -> Result<ReportPage, ReportsError> {
    let control: Option<(String, String, bool)> = conn
        .query_row(
            "SELECT k.code, a.code, a.is_pukhta_khata FROM accounts a
             JOIN account_controls k ON k.id = a.control_id WHERE a.id = ?1",
            [account],
            |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(2)? == 1)),
        )
        .ok();
    let Some((control, code, pukhta)) = control else {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrNotFound,
            "filters.party.account_id",
        )]));
    };
    let buyer = control == BUYER_CONTROL;
    // The own-goods account never holds a balance: the credit of its lots goes to the trip's
    // partners (PRD FR-12.26), so it has no slip.
    if (!buyer && control != SELLER_CONTROL) || code == mandi_core::accounts::OWN_GOODS_CODE {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.party.account_id",
        )]));
    }
    let key = typed.person_key.as_deref();
    if !pukhta && key.is_some() {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.party.person_key",
        )]));
    }
    if pukhta && key.is_none() {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrRequired,
            "filters.party.person_key",
        )]));
    }

    let who = Who {
        account,
        person: key.map(str::to_owned),
        buyer,
    };
    let figures = load(conn, &[day.to_owned()], Some(&who))?;
    let map_key = (day.to_owned(), account, key.unwrap_or_default().to_owned());
    // The weight column is there only when some line of the slip was weighed.
    let weighed = if buyer {
        figures
            .buyers
            .get(&map_key)
            .is_some_and(|d| d.lines.iter().any(|l| l.weight_grams.is_some()))
    } else {
        figures
            .sellers
            .get(&map_key)
            .is_some_and(|d| d.lines.iter().any(|l| l.weight_grams.is_some()))
    };
    // The quality column is there only when some line of the slip has a quality.
    let graded = if buyer {
        figures
            .buyers
            .get(&map_key)
            .is_some_and(|d| d.lines.iter().any(|l| l.grade.is_some()))
    } else {
        figures
            .sellers
            .get(&map_key)
            .is_some_and(|d| d.lines.iter().any(|l| l.grade.is_some()))
    };
    let (rows, summary) = if buyer {
        let d = figures.buyers.get(&map_key).ok_or_else(lacking)?;
        let lines: Vec<GoodsLine<'_>> = d
            .lines
            .iter()
            .map(|l| GoodsLine {
                bikri_id: l.bikri_id,
                goods: &l.goods,
                qty: l.qty,
                weight_grams: l.weight_grams,
                grade: l.grade,
                rate: l.rate,
                amount: l.value,
            })
            .collect();
        (goods_rows(&lines, weighed, graded)?, buyer_summary(d)?)
    } else {
        let d = figures.sellers.get(&map_key).ok_or_else(lacking)?;
        let lines: Vec<GoodsLine<'_>> = d
            .lines
            .iter()
            .map(|l| GoodsLine {
                bikri_id: l.bikri_id,
                goods: &l.goods,
                qty: l.qty,
                weight_grams: l.weight_grams,
                grade: l.grade,
                rate: l.rate,
                amount: l.amount,
            })
            .collect();
        (goods_rows(&lines, weighed, graded)?, seller_summary(d)?)
    };

    let mut resolved = typed.resolved.clone();
    resolved.as_of = Some(day.to_owned());
    Ok(ReportPage {
        group_above: None,
        tables: Vec::new(),
        id: ReportId::R35,
        title_id: ids::title_id("R35"),
        filters: resolved,
        columns: slip_columns(weighed, graded),
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        // The summary lines hold the day's figures: the slip has no totals row (PRD FR-19.12), so
        // nothing is said twice.
        totals: vec![Cell::Empty; 5 + usize::from(weighed) + usize::from(graded)],
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn goods(item_id: i64, item: &str, packing_id: i64, packing: &str) -> Goods {
        Goods {
            item_id,
            packing_id,
            item: Names {
                ur: item.to_owned(),
                en: item.to_owned(),
            },
            packing: Names {
                ur: packing.to_owned(),
                en: packing.to_owned(),
            },
        }
    }

    fn rupees(n: i64) -> Paisa {
        Paisa::from_rupees(n).unwrap()
    }

    #[test]
    fn lines_of_an_item_stay_together_and_an_item_with_two_lines_has_a_total() {
        let banana_box = goods(1, "Banana", 10, "Box");
        let banana_crate = goods(1, "Banana", 11, "Crate");
        let apple_box = goods(2, "Apple", 10, "Box");
        // Saved order: banana box, apple box, banana crate, banana box.
        let lines = [
            GoodsLine {
                bikri_id: 1,
                goods: &banana_box,
                qty: 20,
                weight_grams: None,
                grade: None,
                rate: rupees(600),
                amount: rupees(12_000),
            },
            GoodsLine {
                bikri_id: 1,
                goods: &apple_box,
                qty: 5,
                weight_grams: None,
                grade: None,
                rate: rupees(1_000),
                amount: rupees(5_000),
            },
            GoodsLine {
                bikri_id: 2,
                goods: &banana_crate,
                qty: 4,
                weight_grams: None,
                grade: None,
                rate: rupees(900),
                amount: rupees(3_600),
            },
            GoodsLine {
                bikri_id: 2,
                goods: &banana_box,
                qty: 30,
                weight_grams: None,
                grade: None,
                rate: rupees(580),
                amount: rupees(17_400),
            },
        ];
        let rows = goods_rows(&lines, false, false).unwrap();
        // Banana: the two box lines, then the crate line, then the banana total; then apple alone
        // (one line, no total).
        let qty = |row: &Row| match &row.cells[2] {
            Cell::Number { value } => *value,
            other => panic!("{other:?}"),
        };
        let kinds: Vec<RowKind> = rows.iter().map(|r| r.kind).collect();
        assert_eq!(
            kinds,
            vec![
                RowKind::Normal,
                RowKind::Normal,
                RowKind::Normal,
                RowKind::ItemTotal,
                RowKind::Normal
            ]
        );
        assert_eq!(
            rows.iter().map(qty).collect::<Vec<_>>(),
            vec![20, 30, 4, 54, 5]
        );
        // The total row holds the amount of the item: 12,000 + 17,400 + 3,600.
        assert_eq!(
            rows[3].cells[4],
            Cell::Money {
                paisa: rupees(33_000)
            }
        );
        // A weight item: the stored amount is shown as it is, never qty × rate.
        let weight = [GoodsLine {
            bikri_id: 3,
            goods: &apple_box,
            qty: 20,
            weight_grams: Some(12_345),
            grade: None,
            rate: rupees(100),
            amount: rupees(1_235),
        }];
        let rows = goods_rows(&weight, false, false).unwrap();
        assert_eq!(
            rows[0].cells[4],
            Cell::Money {
                paisa: rupees(1_235)
            }
        );
    }

    #[test]
    fn a_balance_cell_is_read_with_its_side() {
        use mandi_core::accounts::BalanceSide;
        let cr = Cell::Balance {
            paisa: rupees(500),
            side: BalanceSide::Cr,
        };
        assert_eq!(
            signed(&cr).unwrap(),
            Paisa::ZERO.checked_sub(rupees(500)).unwrap()
        );
        assert_eq!(signed(&Cell::Empty).unwrap(), Paisa::ZERO);
        assert!(signed(&Cell::Number { value: 1 }).is_err());
    }
}
