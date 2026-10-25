//! R5 ادھاری تفصیل (the buyer bills) (PRD 19.2 R5, FR-19.10, FR-19.11; E2E-36).
//!
//! The client's own print of 18-10-26: one row for each buyer who bought on credit in the period:
//! the serial, the party, the details with each line written "(item qty . rate)" (for example
//! "(سیب 8 . 2600)"), the value (quantity × rate) and the value after commission and لاگا (C5).
//! "Print All" is one slip for each buyer, from the same rows. The filters are those of the sales
//! registers (dates, buyer, seller, زمیندار, item, city, اڈا).
//!
//! The lines are the credit lines in force of بکری postings dated in the period, the same lines
//! R4 lists, so the total of R5 is the total of R4 (value and buyer total). Every figure is a value
//! stored on the line when it was saved (Code Standards B20). A one-time buyer (پختہ کھاتہ) is a
//! row for each person (PRD FR-13.5). The rows are in the Urdu order of the party names; the
//! totals are over the whole filter and the page only cuts the rows (Code Standards B31).
//!
//! Example (E2E-36, 10-10-26, buyer اکرم): one row, "(کیلا 20 . 600)", value 12,000, after
//! commission and لاگا 12,880.

use std::collections::HashMap;

use mandi_core::money::Paisa;
use mandi_core::text::UrduCollator;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use super::register::conds;
use crate::error::ReportsError;
use crate::reports::common::{
    IN_CHUNK, add, col, count_of, load_parties, paisa, party_of_line, pukhta_party,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, DetailPart, Names, ReportId, ReportPage, Row, RowKind,
};

/// A buyer and, for a one-time account, his person key.
type BuyerKey = (i64, Option<String>);

/// One buyer with his sums over the whole filter.
struct Buyer {
    key: BuyerKey,
    person_ur: Option<String>,
    person_en: Option<String>,
    value: Paisa,
    total: Paisa,
}

/// The buyers who bought on credit under the filters, with their sums, from one query.
fn buyers(conn: &Connection, typed: &Typed) -> Result<Vec<Buyer>, ReportsError> {
    let c = conds(typed, Some("credit"));
    let sql = format!(
        "SELECT s.buyer_account_id, s.pukhta_person_key, MAX(s.pukhta_name_ur), MAX(s.pukhta_name_en),
                COALESCE(SUM(s.buyer_value_paisa), 0), COALESCE(SUM(s.buyer_total_paisa), 0)
         FROM journal_entries je
         JOIN bikri_sales s ON s.journal_entry_id = je.id
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN bikri_entries b ON b.id = s.bikri_id
         JOIN accounts ba ON ba.id = s.buyer_account_id
         WHERE {}
         GROUP BY s.buyer_account_id, s.pukhta_person_key",
        c.sql()
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(c.args().iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (account, person, person_ur, person_en, value, total) in rows {
        out.push(Buyer {
            key: (account, person),
            person_ur,
            person_en,
            value: paisa(value)?,
            total: paisa(total)?,
        });
    }
    Ok(out)
}

/// The lines of these buyers, as the parts of their details, in the order of the bills.
fn lines_of(
    conn: &Connection,
    typed: &Typed,
    accounts: &[i64],
) -> Result<HashMap<BuyerKey, Vec<DetailPart>>, ReportsError> {
    let mut out: HashMap<BuyerKey, Vec<DetailPart>> = HashMap::new();
    for chunk in accounts.chunks(IN_CHUNK) {
        let mut c = conds(typed, Some("credit"));
        c.in_list("s.buyer_account_id", chunk);
        let sql = format!(
            "SELECT s.buyer_account_id, s.pukhta_person_key, b.number, i.name_ur, i.name_en,
                    pk.name_ur, pk.name_en, s.qty, s.buyer_rate_paisa, s.buyer_value_paisa,
                    s.buyer_comm_paisa, s.laga_paisa
             FROM journal_entries je
             JOIN bikri_sales s ON s.journal_entry_id = je.id
             JOIN bikri_lots l ON l.id = s.lot_id
             JOIN bikri_entries b ON b.id = s.bikri_id
             JOIN items i ON i.id = l.item_id
             JOIN packings pk ON pk.id = l.packing_id
             JOIN accounts ba ON ba.id = s.buyer_account_id
             WHERE {}
             ORDER BY je.entry_date, b.number, s.id",
            c.sql()
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement
            .query_map(params_from_iter(c.args().iter()), |r| {
                Ok((
                    (r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?),
                    (
                        r.get::<_, i64>(2)?,
                        Names {
                            ur: r.get(3)?,
                            en: r.get(4)?,
                        },
                        Names {
                            ur: r.get(5)?,
                            en: r.get(6)?,
                        },
                        r.get::<_, i64>(7)?,
                        [
                            r.get::<_, i64>(8)?,
                            r.get::<_, i64>(9)?,
                            r.get::<_, i64>(10)?,
                            r.get::<_, i64>(11)?,
                        ],
                    ),
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (key, (bikri_number, item, packing, qty, money)) in rows {
            out.entry(key).or_default().push(DetailPart::Sale {
                bikri_number,
                item,
                packing,
                qty,
                rate_paisa: paisa(money[0])?,
                value_paisa: paisa(money[1])?,
                commission_paisa: paisa(money[2])?,
                laga_paisa: paisa(money[3])?,
            });
        }
    }
    Ok(out)
}

/// Builds R5.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let mut all = buyers(conn, typed)?;
    let account_ids: Vec<i64> = all.iter().map(|b| b.key.0).collect();
    let parties = load_parties(conn, &account_ids)?;

    // The totals are over every buyer of the filter; the order is the Urdu order of the names.
    let (mut value, mut total) = (Paisa::ZERO, Paisa::ZERO);
    for buyer in &all {
        add(&mut value, buyer.value)?;
        add(&mut total, buyer.total)?;
    }
    let collator = UrduCollator::new().ok();
    let sort_key = |name: &str| -> Vec<u8> {
        collator
            .as_ref()
            .map_or_else(|| name.as_bytes().to_vec(), |c| c.sort_key(name))
    };
    let mut keyed: Vec<(Vec<u8>, Buyer)> = Vec::with_capacity(all.len());
    for buyer in all.drain(..) {
        let party = pukhta_party(
            party_of_line(&parties, buyer.key.0, None)?,
            buyer.key.1.clone(),
            buyer.person_ur.as_deref(),
            buyer.person_en.as_deref(),
        );
        keyed.push((sort_key(&party.name_ur), buyer));
    }
    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.key.0.cmp(&b.1.key.0))
            .then(a.1.key.1.cmp(&b.1.key.1))
    });
    let total_count = count_of(keyed.len());

    // Only the buyers of this page need their lines.
    let skip = usize::try_from(spec.offset).unwrap_or(usize::MAX);
    let take = usize::try_from(spec.sql_limit()).unwrap_or(usize::MAX);
    let page: Vec<&(Vec<u8>, Buyer)> = keyed.iter().skip(skip).take(take).collect();
    // One entry for each account: a one-time account has a row for each person, and an account in
    // two chunks of the IN list would give its lines twice (money agent, round 12).
    let mut page_accounts: Vec<i64> = page.iter().map(|(_, b)| b.key.0).collect();
    page_accounts.sort_unstable();
    page_accounts.dedup();
    let mut lines = lines_of(conn, typed, &page_accounts)?;

    let mut rows = Vec::with_capacity(page.len());
    for (index, (_, buyer)) in page.iter().enumerate() {
        let party = pukhta_party(
            party_of_line(&parties, buyer.key.0, None)?,
            buyer.key.1.clone(),
            buyer.person_ur.as_deref(),
            buyer.person_en.as_deref(),
        );
        let serial = spec
            .offset
            .checked_add(i64::try_from(index).unwrap_or(i64::MAX))
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| ReportsError::Data("the serial does not fit".to_owned()))?;
        rows.push(Row {
            cells: vec![
                Cell::Number { value: serial },
                Cell::Party { party },
                Cell::Detail {
                    parts: lines.remove(&buyer.key).unwrap_or_default(),
                },
                Cell::Money { paisa: buyer.value },
                Cell::Money { paisa: buyer.total },
            ],
            source: None,
            kind: RowKind::Normal,
        });
    }

    Ok(ReportPage {
        id: ReportId::R5,
        title_id: ids::title_id("R5"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("serial", ids::COL_SERIAL, ColumnKind::Number),
            col("buyer", ids::COL_BUYER, ColumnKind::Party),
            col("detail", ids::COL_DETAIL, ColumnKind::Detail),
            col("value", ids::COL_VALUE, ColumnKind::Money),
            col("buyer_total", ids::COL_BUYER_TOTAL, ColumnKind::Money),
        ],
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: value },
            Cell::Money { paisa: total },
        ],
        summary: Vec::new(),
    })
}
