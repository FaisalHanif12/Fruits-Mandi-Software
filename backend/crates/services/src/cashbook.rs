//! The روکڑ بک (PRD 16.1, FR-19.11, plan 17): every cash in and out from all modules, one page at a
//! time, with the opening and closing cash and the totals over the whole range. There is no cash
//! count (PRD FR-16.1): the book only shows what was posted.
//!
//! One row is one journal entry that moves cash. The running cash is the opening cash plus the
//! sums of the rows up to it, over the whole range, so a page far down has the right figure. The
//! sums are `SUM()` on INTEGER columns; the differences are worked out here with checked maths.

use std::collections::HashMap;

use mandi_db::accounts as account_repo;
use mandi_db::cashbook as repo;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::cashbook::{
    CapitalWhere, CapitalWhereKind, CapitalWhereLine, CapitalWhoseKind, CapitalWhoseLine, Cashbook,
    CashbookInput, CashbookKind, CashbookRow, CashbookTotals,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::paging::SortDirection;
use mandi_domain::core::row_source::RowSource;
use mandi_domain::core::validate::{self, FieldErrors};

use crate::cash::CASH_ACCOUNT_CODE;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::page;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// A date bound of the range: empty is open.
fn bound(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&str>,
    today: BusinessDate,
) -> Option<BusinessDate> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors.take(validate::date(field, text, today))
}

/// One page of the cash book with the opening and closing cash and the totals over the whole
/// range (PRD FR-19.11). Without `from` the book starts at the first entry and the opening is 0.
///
/// # Errors
/// The date codes on `from` and `to`; `ServiceError::Db`.
pub fn get(ctx: &Context, input: &CashbookInput) -> Result<Cashbook, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let from = bound(&mut errors, "from", input.from.as_deref(), today);
    let to = bound(&mut errors, "to", input.to.as_deref(), today);
    errors.finish()?;
    let (from_iso, to_iso) = (from.map(BusinessDate::to_iso), to.map(BusinessDate::to_iso));
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    let order = match &input.sort {
        None => repo::Order::Date { desc: false },
        Some(sort) => {
            let chosen = sort
                .order_by(&repo::SORT_COLUMNS)
                .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
            match sort.column.as_str() {
                "entry_date" => repo::Order::Date {
                    desc: sort.direction == SortDirection::Desc,
                },
                "in_paisa" | "out_paisa" => repo::Order::Amount {
                    out: sort.column == "out_paisa",
                    desc: sort.direction == SortDirection::Desc,
                },
                _ => repo::Order::Other(chosen),
            }
        }
    };
    let in_date_order = matches!(order, repo::Order::Date { .. });
    ctx.db.read_snapshot(|conn| {
        let cash = account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
            .ok_or_else(|| ServiceError::Data("the cash account is missing".to_owned()))?;
        let range = repo::range(conn, cash, from_iso.as_deref(), to_iso.as_deref())?;
        // The opening is the cash at the end of the day before the first day of the range: the
        // whole book less what is dated from the first day on (both are kept sums, so a page
        // after the first does not read the book again).
        let opening = match from_iso.as_deref() {
            Some(first) => {
                let whole = repo::range(conn, cash, None, None)?;
                let from_first = repo::range(conn, cash, Some(first), None)?;
                let before_in = whole
                    .in_paisa
                    .checked_sub(from_first.in_paisa)
                    .ok_or_else(|| ServiceError::Data("cash before the range".to_owned()))?;
                let before_out = whole
                    .out_paisa
                    .checked_sub(from_first.out_paisa)
                    .ok_or_else(|| ServiceError::Data("cash before the range".to_owned()))?;
                paisa(before_in)?
                    .checked_sub(paisa(before_out)?)
                    .map_err(money_error)?
            }
            None => Paisa::ZERO,
        };
        let (total_in, total_out) = (paisa(range.in_paisa)?, paisa(range.out_paisa)?);
        let closing = opening
            .checked_add(total_in)
            .and_then(|v| v.checked_sub(total_out))
            .map_err(money_error)?;
        let stored = repo::page(
            conn,
            cash,
            from_iso.as_deref(),
            to_iso.as_deref(),
            &order,
            i64::from(input.offset),
            page::limit_of(input.limit),
        )?;
        let ids: Vec<i64> = stored.iter().map(|r| r.entry_id).collect();
        let others = repo::counterparts(conn, cash, &ids)?;
        // The partner behind each capital entry of the page (plan 78 E), one query.
        let capital: HashMap<i64, repo::CapitalPartner> = repo::capital_partners(conn, &ids)?
            .into_iter()
            .map(|c| (c.entry_id, c))
            .collect();
        let mut rows = Vec::with_capacity(stored.len());
        for row in &stored {
            let running = match (row.run_in_paisa, row.run_out_paisa) {
                (Some(run_in), Some(run_out)) => Some(
                    opening
                        .checked_add(paisa(run_in)?)
                        .map_err(money_error)?
                        .checked_sub(paisa(run_out)?)
                        .map_err(money_error)?,
                ),
                _ => None,
            };
            // The other side is shown only when it is one account: the entry of a بکری has many.
            let account = counterpart(&others, row);
            let partner = capital.get(&row.entry_id).map(|c| RecordRef {
                id: c.partner_id,
                code: String::new(),
                name_ur: c.name_ur.clone(),
                name_en: c.name_en.clone(),
                tag: None,
            });
            rows.push(CashbookRow {
                entry_id: row.entry_id,
                entry_date: row.entry_date.clone(),
                posting_type: row.posting_type.clone(),
                number: row.number,
                account,
                kind: kind_of(row, cash, partner.is_some()),
                partner,
                narration: if row.narration.is_empty() {
                    row.reason.clone()
                } else {
                    row.narration.clone()
                },
                in_paisa: paisa(row.in_paisa)?,
                out_paisa: paisa(row.out_paisa)?,
                running_paisa: running,
                source: Some(source_of(row)),
            });
        }
        // Month headings with their subtotals, for the months this page touches (plan 38 B2).
        let months = match (
            input.group_by_month && in_date_order,
            stored.first(),
            stored.last(),
        ) {
            (true, Some(first), Some(last)) => {
                let newest_first = matches!(order, repo::Order::Date { desc: true });
                page::month_subtotals(
                    repo::months_between(
                        conn,
                        cash,
                        (from_iso.as_deref(), to_iso.as_deref()),
                        (&first.entry_date, &last.entry_date),
                    )?,
                    &[("in_paisa", true), ("out_paisa", true)],
                    newest_first,
                )
            }
            _ => Vec::new(),
        };
        Ok(Cashbook {
            opening_paisa: opening,
            closing_paisa: closing,
            first_entry_date: repo::first_entry_date(conn, cash)?,
            rows,
            total_count: range.count,
            totals: CashbookTotals {
                in_paisa: total_in,
                out_paisa: total_out,
            },
            months,
        })
    })
}

/// What a row is in the words of the Owner (plan 78 E, PRD FR-16.14): the opening of the cash account,
/// a partner's opening capital, or capital in or out; any other row is `other`.
fn kind_of(row: &repo::CashRow, cash: i64, has_partner: bool) -> CashbookKind {
    match row.posting_type.as_str() {
        "opening" if row.source_type == "account" && row.source_id == cash => {
            CashbookKind::OpeningCash
        }
        "opening" if has_partner => CashbookKind::OpeningCapital,
        "capital_in" => CashbookKind::CapitalIn,
        "capital_out" => CashbookKind::CapitalOut,
        _ => CashbookKind::Other,
    }
}

/// The record a row opens (PRD FR-19.4). An entry of a trip or a garden opens the trip or the
/// garden itself: its stored source id is the id of one of its entries.
fn source_of(row: &repo::CashRow) -> RowSource {
    RowSource::of_stored(
        &row.source_type,
        row.source_id,
        row.entry_id,
        row.venture_kind.as_deref(),
        row.venture_id,
    )
}

/// The one account on the other side of an entry that only takes cash in, or only gives it out.
fn counterpart(
    others: &HashMap<i64, Vec<repo::Counterpart>>,
    row: &repo::CashRow,
) -> Option<RecordRef> {
    // An entry that does both (cash in on one line and out on another) has no single other side.
    if row.in_paisa > 0 && row.out_paisa > 0 {
        return None;
    }
    // Cash received (debit) is matched by credits; cash paid out (credit) by debits.
    let cash_received = row.in_paisa > 0;
    let mut found: Option<&repo::Counterpart> = None;
    for line in others.get(&row.entry_id)?.iter() {
        if line.is_debit == cash_received {
            continue;
        }
        match found {
            Some(first) if first.account_id != line.account_id => return None,
            Some(_) => {}
            None => found = Some(line),
        }
    }
    found.map(|line| RecordRef {
        id: line.account_id,
        code: line.code.clone(),
        name_ur: line.name_ur.clone(),
        name_en: line.name_en.clone(),
        tag: line.tag.clone(),
    })
}

/// Where the partners' money is now and whose it is (PRD FR-16.14 v1.43, plan 78 E). Every figure is
/// from the posted records, in one read snapshot, with checked maths. The caller checks the partner
/// code (`ERR_PARTNER_LOCKED`).
///
/// The left side is the chart's heads 01 to 03 (what the shop has, is owed and owes), the right side
/// heads 04 to 06 (capital, income, expenses). The books balance, so the two totals are equal. A seller
/// is split in the three lines of PRD FR-16.14 from his ledger net `n` and his open advance `a`:
/// advances `a`, beyond `max(0, n − a)`, payable `max(0, a − n)`, so `a + beyond − payable = n`. A
/// difference no named line explains is an `other` line, shown only when it is not 0.
///
/// Example (after Events 1 to 5): 58,484 + 21,564 − 10,160 − 4,400 = 65,488 = −1,004 + 746 + 746 +
/// 65,000.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data`; `ERR_OVERFLOW`.
pub fn capital_where(ctx: &Context) -> Result<CapitalWhere, ServiceError> {
    ctx.db.read_snapshot(|conn| {
        let nets = repo::account_nets(conn)?;
        let open_adv: HashMap<i64, i64> =
            repo::open_advances_by_seller(conn)?.into_iter().collect();
        let net_of = |dr: i64, cr: i64| -> Result<Paisa, ServiceError> {
            paisa(dr)?.checked_sub(paisa(cr)?).map_err(money_error)
        };
        let add = |sum: &mut Paisa, part: Paisa| -> Result<(), ServiceError> {
            *sum = sum.checked_add(part).map_err(money_error)?;
            Ok(())
        };
        let (mut cash, mut buyers, mut lent, mut charges) =
            (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        let (mut khatas, mut gardens, mut contra) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        let (mut advances, mut beyond, mut payable) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
        let (mut left_all, mut right_all) = (Paisa::ZERO, Paisa::ZERO);
        let (mut opening_capital, mut difference) = (Paisa::ZERO, Paisa::ZERO);
        for a in &nets {
            let net = net_of(a.dr_paisa, a.cr_paisa)?;
            match a.head_code.as_str() {
                "01" | "02" | "03" => add(&mut left_all, net)?,
                _ => add(&mut right_all, net)?,
            }
            if a.code == CASH_ACCOUNT_CODE {
                add(&mut cash, net)?;
            } else if a.code == "06010001" {
                add(&mut opening_capital, net)?;
            } else if a.code == mandi_domain::core::accounts::OPENING_DIFFERENCE_CODE {
                add(&mut difference, net)?;
            } else if a.head_code == "02" {
                add(&mut charges, net)?;
            } else {
                match a.control_code.as_str() {
                    "0304" => add(&mut buyers, net)?,
                    "0103" => add(&mut lent, net)?,
                    "0305" => add(&mut khatas, net)?,
                    "0306" => add(&mut gardens, net)?,
                    "0307" => add(&mut contra, net)?,
                    "03011" => {
                        let open = paisa(open_adv.get(&a.id).copied().unwrap_or(0))?;
                        add(&mut advances, open)?;
                        let over = net.checked_sub(open).map_err(money_error)?;
                        if over.is_negative() {
                            add(&mut payable, over.checked_neg().map_err(money_error)?)?;
                        } else {
                            add(&mut beyond, over)?;
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut where_lines = vec![
            (CapitalWhereKind::Cash, cash),
            (CapitalWhereKind::BuyersOwe, buyers),
            (CapitalWhereKind::SellerAdvances, advances),
            (CapitalWhereKind::SellersOweBeyond, beyond),
            (
                CapitalWhereKind::SellersPayable,
                payable.checked_neg().map_err(money_error)?,
            ),
            (CapitalWhereKind::MoneyLent, lent),
            (CapitalWhereKind::OwedForCharges, charges),
            (CapitalWhereKind::SeparateKhatas, khatas),
            (CapitalWhereKind::GardensHeld, gardens),
            (CapitalWhereKind::LinkedInPartnerTotals, contra),
        ];
        let mut listed = Paisa::ZERO;
        for (_, amount) in &where_lines {
            add(&mut listed, *amount)?;
        }
        let other = left_all.checked_sub(listed).map_err(money_error)?;
        if other != Paisa::ZERO {
            where_lines.push((CapitalWhereKind::Other, other));
        }
        // Whose it is: heads 04 to 06 are credit-side, so the right total is their net with the sign
        // turned.
        let right_total = right_all.checked_neg().map_err(money_error)?;
        let mut whose_lines: Vec<CapitalWhoseLine> = Vec::new();
        let mut listed = Paisa::ZERO;
        // The advances to sellers are read and replayed once: for every partner's total and for
        // the advances still open (R1-4).
        let advances = super::partners::money::read_advances(conn)?;
        for partner in mandi_db::partners::list(conn)? {
            let total = super::partners::money::total_with(conn, &partner, None, &advances)?;
            add(&mut listed, total)?;
            whose_lines.push(CapitalWhoseLine {
                kind: CapitalWhoseKind::PartnerTotal,
                partner: Some(RecordRef {
                    id: partner.id,
                    code: String::new(),
                    name_ur: partner.name_ur,
                    name_en: partner.name_en,
                    tag: None,
                }),
                amount_paisa: total,
            });
        }
        let opening = opening_capital.checked_neg().map_err(money_error)?;
        add(&mut listed, opening)?;
        whose_lines.push(CapitalWhoseLine {
            kind: CapitalWhoseKind::OpeningCapital,
            partner: None,
            amount_paisa: opening,
        });
        let open_parts = advances.open_parts_total(None).map_err(money_error)?;
        add(&mut listed, open_parts)?;
        whose_lines.push(CapitalWhoseLine {
            kind: CapitalWhoseKind::AdvancesOpen,
            partner: None,
            amount_paisa: open_parts,
        });
        let unrecorded = difference.checked_neg().map_err(money_error)?;
        if unrecorded != Paisa::ZERO {
            add(&mut listed, unrecorded)?;
            whose_lines.push(CapitalWhoseLine {
                kind: CapitalWhoseKind::UnrecordedWhere,
                partner: None,
                amount_paisa: unrecorded,
            });
        }
        let rest = right_total.checked_sub(listed).map_err(money_error)?;
        if rest != Paisa::ZERO {
            whose_lines.push(CapitalWhoseLine {
                kind: CapitalWhoseKind::Other,
                partner: None,
                amount_paisa: rest,
            });
        }
        Ok(CapitalWhere {
            where_lines: where_lines
                .into_iter()
                .map(|(kind, amount_paisa)| CapitalWhereLine { kind, amount_paisa })
                .collect(),
            where_total_paisa: left_all,
            whose_lines,
            whose_total_paisa: right_total,
        })
    })
}
