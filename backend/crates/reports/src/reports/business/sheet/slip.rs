//! The slip of one party (R35): a buyer's day slip or a seller's.

use super::activity::*;
use super::lines::*;
use super::*;

/// The columns of one party's slip.
pub(super) fn slip_columns() -> Vec<crate::reports::model::Column> {
    vec![
        col("qty", ids::COL_QTY, ColumnKind::Number),
        col("item", ids::COL_ITEM, ColumnKind::Names),
        col("rate", ids::COL_RATE, ColumnKind::Money),
        col("amount", ids::COL_AMOUNT, ColumnKind::Money),
    ]
}

/// The item of a line as the slip writes it: the packing and the item (کیلا in a پیٹی reads
/// "پیٹی کیلا" in Urdu), in both languages.
pub(super) fn item_cell(packing_ur: &str, item_ur: &str, packing_en: &str, item_en: &str) -> Cell {
    names_cell(
        Some(format!("{packing_ur} {item_ur}")),
        Some(format!("{packing_en} {item_en}")),
    )
}

/// The slip of one party.
pub(super) fn slip(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    day: &str,
    account: i64,
) -> Result<ReportPage, ReportsError> {
    let control: Option<(String, bool)> = conn
        .query_row(
            "SELECT k.code, a.is_pukhta_khata FROM accounts a
             JOIN account_controls k ON k.id = a.control_id WHERE a.id = ?1",
            [account],
            |r| Ok((r.get(0)?, r.get::<_, i64>(1)? == 1)),
        )
        .ok();
    let Some((control, pukhta)) = control else {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrNotFound,
            "filters.party.account_id",
        )]));
    };
    let buyer = control == BUYER_CONTROL;
    if !buyer && control != SELLER_CONTROL {
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

    // The party's sums; a party with no posted line has none.
    let found = activities(conn, day, Some(account), key)?;
    let me = found.into_iter().next().unwrap_or(Activity {
        account_id: account,
        person_key: key.map(str::to_owned),
        buyer,
        prev_dr: Paisa::ZERO,
        prev_cr: Paisa::ZERO,
        day_dr: Paisa::ZERO,
        day_cr: Paisa::ZERO,
        end_dr: Paisa::ZERO,
        end_cr: Paisa::ZERO,
    });
    let previous = net_cell(me.prev_net()?)?;
    let remaining = net_cell(me.end_net()?)?;

    let mut rows = Vec::new();
    let mut total_qty = 0_i64;
    let total_amount: Paisa;
    let mut summary = Vec::new();
    if buyer {
        let lines = sale_lines(conn, day, account, key)?;
        let (mut bought, mut on_spot) = (Paisa::ZERO, Paisa::ZERO);
        for l in &lines {
            total_qty = total_qty
                .checked_add(l.qty)
                .ok_or_else(|| ReportsError::Data("the quantities do not fit".to_owned()))?;
            add(&mut bought, l.total)?;
            if l.cash {
                add(&mut on_spot, l.total)?;
            }
            rows.push(Row {
                cells: vec![
                    Cell::Number { value: l.qty },
                    item_cell(&l.packing_ur, &l.item_ur, &l.packing_en, &l.item_en),
                    Cell::Money { paisa: l.rate },
                    Cell::Money { paisa: l.total },
                ],
                source: Some(RowSource {
                    kind: SourceKind::Bikri,
                    id: l.bikri_id,
                }),
                kind: RowKind::Normal,
            });
        }
        total_amount = bought;
        // What the ledger shows beyond the sale lines: cash lent, a جنرل debit.
        let other_debits = me.day_dr.checked_sub(bought)?;
        // "Paid that day" is the receipts (وصولی) only; any other credit (a جنرل credit, an
        // opening credit, a sale taken back) is its own line (money agent S2).
        let paid = posting_sum(conn, day, account, key, "receipt", true)?;
        let other_credits = me.day_cr.checked_sub(on_spot)?.checked_sub(paid)?;
        if other_debits.is_negative() || paid.is_negative() || other_credits.is_negative() {
            return Err(ReportsError::Data(
                "the sale lines of the day are more than the ledger of the party".to_owned(),
            ));
        }
        summary.push(line(ids::R35_TOTAL_QTY, Cell::Number { value: total_qty }));
        summary.push(line(ids::R35_PREVIOUS, previous));
        summary.push(line(ids::R35_BOUGHT, Cell::Money { paisa: bought }));
        if other_debits != Paisa::ZERO {
            summary.push(line(
                ids::R35_OTHER_DEBITS,
                Cell::Money {
                    paisa: other_debits,
                },
            ));
        }
        if on_spot != Paisa::ZERO {
            summary.push(line(ids::R35_ON_SPOT, Cell::Money { paisa: on_spot }));
        }
        summary.push(line(ids::R35_RECEIVED, blank_if_zero(paid)));
        if other_credits != Paisa::ZERO {
            summary.push(line(
                ids::R35_OTHER_CREDITS,
                Cell::Money {
                    paisa: other_credits,
                },
            ));
        }
        summary.push(line(ids::R35_REMAINING, remaining));
    } else {
        let lines = credit_lines(conn, day, account, key)?;
        let mut sold = Paisa::ZERO;
        for l in &lines {
            add(&mut sold, l.amount)?;
            if let Some(qty) = l.qty {
                total_qty = total_qty
                    .checked_add(qty)
                    .ok_or_else(|| ReportsError::Data("the quantities do not fit".to_owned()))?;
            }
            rows.push(Row {
                cells: vec![
                    l.qty.map_or(Cell::Empty, |value| Cell::Number { value }),
                    l.names
                        .as_ref()
                        .map_or(Cell::Empty, |(pu, pe, iu, ie)| item_cell(pu, iu, pe, ie)),
                    Cell::Empty,
                    Cell::Money { paisa: l.amount },
                ],
                source: Some(RowSource {
                    kind: SourceKind::Bikri,
                    id: l.bikri_id,
                }),
                kind: RowKind::Normal,
            });
        }
        let cash_given = cash_given(conn, day, account, key)?;
        let debited_by_bikri = bikri_side(conn, day, account, key, false)?;
        let credited_by_bikri = bikri_side(conn, day, account, key, true)?;
        // A lot whose net is negative debits the seller; that is not cash he got.
        let negative_nets = debited_by_bikri.checked_sub(cash_given)?;
        let other_credits = me.day_cr.checked_sub(credited_by_bikri)?;
        // "Paid that day" is the payments (ادائیگی) only; an advance given or a جنرل debit is
        // its own line (money agent S3).
        let paid = posting_sum(conn, day, account, key, "payment", false)?;
        let other_debits = me.day_dr.checked_sub(debited_by_bikri)?.checked_sub(paid)?;
        if negative_nets.is_negative()
            || other_credits.is_negative()
            || other_debits.is_negative()
            || paid.is_negative()
            || credited_by_bikri != sold
        {
            return Err(ReportsError::Data(
                "the credit lines of the day do not match the ledger of the seller".to_owned(),
            ));
        }
        if negative_nets != Paisa::ZERO {
            // One row for them, so the rows still add up to what was sold.
            rows.push(Row {
                cells: vec![
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Money {
                        paisa: Paisa::ZERO.checked_sub(negative_nets)?,
                    },
                ],
                source: None,
                kind: RowKind::Normal,
            });
        }
        let sold = sold.checked_sub(negative_nets)?;
        total_amount = sold;
        summary.push(line(ids::R35_TOTAL_QTY, Cell::Number { value: total_qty }));
        summary.push(line(ids::R35_PREVIOUS, previous));
        summary.push(line(ids::R35_SOLD, Cell::Money { paisa: sold }));
        if other_credits != Paisa::ZERO {
            summary.push(line(
                ids::R35_OTHER_CREDITS,
                Cell::Money {
                    paisa: other_credits,
                },
            ));
        }
        if cash_given != Paisa::ZERO {
            summary.push(line(ids::R35_CASH_GIVEN, Cell::Money { paisa: cash_given }));
        }
        summary.push(line(ids::R35_PAID, blank_if_zero(paid)));
        if other_debits != Paisa::ZERO {
            summary.push(line(
                ids::R35_OTHER_DEBITS,
                Cell::Money {
                    paisa: other_debits,
                },
            ));
        }
        summary.push(line(ids::R35_REMAINING, remaining));
    }

    let mut resolved = typed.resolved.clone();
    resolved.as_of = Some(day.to_owned());
    Ok(ReportPage {
        id: ReportId::R35,
        title_id: ids::title_id("R35"),
        filters: resolved,
        columns: slip_columns(),
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Number { value: total_qty },
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: total_amount,
            },
        ],
        summary,
    })
}
