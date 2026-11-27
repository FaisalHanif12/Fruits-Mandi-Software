//! A garden's sale money list in SQL (PRD FR-12.53 v1.60, plan 99). No rules here: each read is one
//! query (Code Standards B22), and every figure is a stored one.

use std::collections::HashMap;

use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

/// The most ids one query takes.
const IN_CHUNK: usize = 500;

/// An id with its code and both names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    /// Row id.
    pub id: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Tag, if any.
    pub tag: Option<String>,
}

/// The lot a row of our shop is of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotRef {
    /// The lot.
    pub lot_id: i64,
    /// The بکری.
    pub bikri_id: i64,
    /// Its number.
    pub bikri_number: i64,
    /// Its day (the day of its first posting).
    pub bikri_date: String,
    /// The lot's number in the بکری.
    pub lot_no: i64,
    /// The counter of its name.
    pub lot_counter: i64,
    /// Item.
    pub item: Named,
    /// Packing.
    pub packing: Named,
}

/// One active sale line of a lot of the garden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The sale line.
    pub sale_id: i64,
    /// The posting it belongs to.
    pub entry_id: i64,
    /// Its day.
    pub date: String,
    /// Its number in the lot.
    pub line_no: i64,
    /// The lot.
    pub lot: LotRef,
    /// The buyer; a چکتا buyer by his own names.
    pub buyer: Named,
    /// نگ.
    pub qty: i64,
    /// Weight in grams.
    pub weight_grams: Option<i64>,
    /// Seller rate, paisa.
    pub rate_paisa: i64,
    /// Seller amount, paisa.
    pub value_paisa: i64,
    /// Seller commission, paisa.
    pub seller_comm_paisa: i64,
    /// Buyer total, paisa.
    pub buyer_total_paisa: i64,
    /// Paid on the spot, paisa.
    pub paid_paisa: i64,
}

/// The charges of a lot of the garden other than the commission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotCharges {
    /// The lot.
    pub lot: LotRef,
    /// Freight paid by آڑھتی, paisa.
    pub freight_paisa: i64,
    /// Labour.
    pub labour_paisa: i64,
    /// Munshiana.
    pub munshiana_paisa: i64,
    /// Phone.
    pub phone_paisa: i64,
    /// Store rent.
    pub store_rent_paisa: i64,
}

/// A saved sale in another shop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherSale {
    /// The garden entry.
    pub entry_id: i64,
    /// Its day.
    pub date: String,
    /// The money received, paisa.
    pub amount_paisa: i64,
    /// نگ.
    pub qty: i64,
    /// The shop, as typed.
    pub shop_name: Option<String>,
    /// The item of the arrival it names, if any.
    pub item: Option<Named>,
}

const LOT_COLUMNS: &str = "l.id, b.id, b.number, b.entry_date, l.lot_no, l.lot_counter,
    i.id, i.code, i.name_ur, i.name_en, pk.id, pk.code, pk.name_ur, pk.name_en";

const LOT_JOINS: &str = "JOIN bikri_entries b ON b.id = l.bikri_id AND b.status = 'saved'
    JOIN items i ON i.id = l.item_id JOIN packings pk ON pk.id = l.packing_id";

fn named_at(r: &rusqlite::Row<'_>, at: usize) -> rusqlite::Result<Named> {
    Ok(Named {
        id: r.get(at)?,
        code: r.get(at + 1)?,
        name_ur: r.get(at + 2)?,
        name_en: r.get(at + 3)?,
        tag: None,
    })
}

fn lot_at(r: &rusqlite::Row<'_>) -> rusqlite::Result<LotRef> {
    Ok(LotRef {
        lot_id: r.get(0)?,
        bikri_id: r.get(1)?,
        bikri_number: r.get(2)?,
        bikri_date: r.get(3)?,
        lot_no: r.get(4)?,
        lot_counter: r.get(5)?,
        item: named_at(r, 6)?,
        packing: named_at(r, 10)?,
    })
}

/// The lots of the account, with the arrival condition when asked: `(sql, values)`.
fn lot_filter(account_id: i64, arrival_id: Option<i64>) -> (String, Vec<Value>) {
    let mut sql = " WHERE l.seller_account_id = ? AND l.removed_at IS NULL".to_owned();
    let mut values = vec![Value::Integer(account_id)];
    if let Some(arrival) = arrival_id {
        sql.push_str(" AND l.arrival_id = ?");
        values.push(Value::Integer(arrival));
    }
    (sql, values)
}

/// Every active sale line of the lots whose seller is the account, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn lines(
    conn: &Connection,
    account_id: i64,
    arrival_id: Option<i64>,
) -> rusqlite::Result<Vec<Line>> {
    let (condition, values) = lot_filter(account_id, arrival_id);
    let mut statement = conn.prepare(&format!(
        "SELECT {LOT_COLUMNS}, s.id, s.journal_entry_id, s.entry_date, s.line_no,
                a.id, a.code, COALESCE(s.chukta_name_ur, a.name_ur), COALESCE(s.chukta_name_en, a.name_en),
                CASE WHEN s.chukta_name_ur IS NULL THEN a.tag END,
                s.qty, s.weight_grams, s.seller_rate_paisa, s.seller_amount_paisa,
                s.seller_comm_paisa, s.buyer_total_paisa, s.paid_paisa
         FROM bikri_lots l {LOT_JOINS}
         JOIN bikri_sales s ON s.lot_id = l.id AND s.is_active = 1
         JOIN accounts a ON a.id = s.buyer_account_id{condition}
         ORDER BY s.entry_date, b.number, l.lot_no, s.line_no, s.id"
    ))?;
    let rows = statement.query_map(params_from_iter(values.iter()), |r| {
        Ok(Line {
            lot: lot_at(r)?,
            sale_id: r.get(14)?,
            entry_id: r.get(15)?,
            date: r.get(16)?,
            line_no: r.get(17)?,
            buyer: Named {
                id: r.get(18)?,
                code: r.get(19)?,
                name_ur: r.get(20)?,
                name_en: r.get(21)?,
                tag: r.get(22)?,
            },
            qty: r.get(23)?,
            weight_grams: r.get(24)?,
            rate_paisa: r.get(25)?,
            value_paisa: r.get(26)?,
            seller_comm_paisa: r.get(27)?,
            buyer_total_paisa: r.get(28)?,
            paid_paisa: r.get(29)?,
        })
    })?;
    rows.collect()
}

/// The lots of the account that carry charges (their first posting), oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn lot_charges(
    conn: &Connection,
    account_id: i64,
    arrival_id: Option<i64>,
) -> rusqlite::Result<Vec<LotCharges>> {
    let (condition, values) = lot_filter(account_id, arrival_id);
    let mut statement = conn.prepare(&format!(
        "SELECT {LOT_COLUMNS},
                CASE WHEN l.freight_paid_by = 'firm' THEN l.freight_paisa ELSE 0 END,
                l.labour_paisa, l.munshiana_paisa, l.phone_paisa, l.store_rent_paisa
         FROM bikri_lots l {LOT_JOINS}{condition}
           AND ((l.freight_paid_by = 'firm' AND l.freight_paisa > 0) OR l.labour_paisa > 0
                OR l.munshiana_paisa > 0 OR l.phone_paisa > 0 OR l.store_rent_paisa > 0)
         ORDER BY b.entry_date, b.number, l.lot_no"
    ))?;
    let rows = statement.query_map(params_from_iter(values.iter()), |r| {
        Ok(LotCharges {
            lot: lot_at(r)?,
            freight_paisa: r.get(14)?,
            labour_paisa: r.get(15)?,
            munshiana_paisa: r.get(16)?,
            phone_paisa: r.get(17)?,
            store_rent_paisa: r.get(18)?,
        })
    })?;
    rows.collect()
}

/// A sale line of a posting, for finding its bill line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostedSale {
    /// The sale line.
    pub sale_id: i64,
    /// The buyer account.
    pub buyer_account_id: i64,
    /// The چکتا person, if any.
    pub person_key: Option<String>,
    /// The bill.
    pub buyer_total_paisa: i64,
}

/// Every active sale line of these postings (all sellers), per posting in lot and line order: the
/// order their bills were posted in.
///
/// # Errors
/// Any SQLite error.
pub fn posted_sales(
    conn: &Connection,
    entry_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Vec<PostedSale>>> {
    let mut out: HashMap<i64, Vec<PostedSale>> = HashMap::new();
    for chunk in entry_ids.chunks(IN_CHUNK) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&format!(
            "SELECT s.journal_entry_id, s.id, s.buyer_account_id, s.chukta_person_key, s.buyer_total_paisa
             FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id
             WHERE s.journal_entry_id IN ({marks}) AND s.is_active = 1
             ORDER BY s.journal_entry_id, l.lot_no, s.line_no, s.id"
        ))?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                PostedSale {
                    sale_id: r.get(1)?,
                    buyer_account_id: r.get(2)?,
                    person_key: r.get(3)?,
                    buyer_total_paisa: r.get(4)?,
                },
            ))
        })?;
        for row in rows {
            let (entry, sale) = row?;
            out.entry(entry).or_default().push(sale);
        }
    }
    Ok(out)
}

/// A debit line of a posting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebitLine {
    /// The journal line.
    pub line_id: i64,
    /// The account.
    pub account_id: i64,
    /// The چکتا person, if any.
    pub person_key: Option<String>,
    /// Debit, paisa.
    pub dr_paisa: i64,
}

/// The debit lines of these postings, in line order.
///
/// # Errors
/// Any SQLite error.
pub fn debit_lines(
    conn: &Connection,
    entry_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Vec<DebitLine>>> {
    let mut out: HashMap<i64, Vec<DebitLine>> = HashMap::new();
    for chunk in entry_ids.chunks(IN_CHUNK) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&format!(
            "SELECT l.entry_id, l.id, l.account_id, l.chukta_person_key, l.dr_paisa
             FROM journal_lines l WHERE l.entry_id IN ({marks}) AND l.dr_paisa > 0
             ORDER BY l.entry_id, l.line_no"
        ))?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                DebitLine {
                    line_id: r.get(1)?,
                    account_id: r.get(2)?,
                    person_key: r.get(3)?,
                    dr_paisa: r.get(4)?,
                },
            ))
        })?;
        for row in rows {
            let (entry, line) = row?;
            out.entry(entry).or_default().push(line);
        }
    }
    Ok(out)
}

/// What is matched on these bill lines by credits of OTHER entries (the credit of the same posting
/// is the money paid on the spot), paisa by bill line.
///
/// # Errors
/// Any SQLite error.
pub fn collected_since(conn: &Connection, line_ids: &[i64]) -> rusqlite::Result<HashMap<i64, i64>> {
    let mut out = HashMap::new();
    for chunk in line_ids.chunks(IN_CHUNK) {
        let marks = vec!["?"; chunk.len()].join(", ");
        let mut statement = conn.prepare(&format!(
            "SELECT m.bill_line_id, SUM(m.amount_paisa)
             FROM bill_allocations m
             JOIN journal_lines bl ON bl.id = m.bill_line_id
             JOIN journal_lines pl ON pl.id = m.payment_line_id
             WHERE m.is_active = 1 AND m.bill_line_id IN ({marks}) AND pl.entry_id <> bl.entry_id
             GROUP BY m.bill_line_id"
        ))?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (line, sum) = row?;
            out.insert(line, sum);
        }
    }
    Ok(out)
}

/// The saved sales in another shop of a garden, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn other_sales(
    conn: &Connection,
    venture_id: i64,
    arrival_id: Option<i64>,
) -> rusqlite::Result<Vec<OtherSale>> {
    let mut statement = conn.prepare(
        "SELECT e.id, e.entry_date, e.amount_paisa, COALESCE(e.qty, 0), e.shop_name,
                i.id, i.code, i.name_ur, i.name_en
         FROM venture_entries e
         LEFT JOIN garden_arrivals ga ON ga.id = e.arrival_id
         LEFT JOIN items i ON i.id = ga.item_id
         WHERE e.venture_id = ?1 AND e.kind = 'other_sale' AND e.status = 'saved'
           AND (?2 IS NULL OR e.arrival_id = ?2)
         ORDER BY e.entry_date, e.id",
    )?;
    let rows = statement.query_map(rusqlite::params![venture_id, arrival_id], |r| {
        let item = match r.get::<_, Option<i64>>(5)? {
            Some(_) => Some(named_at(r, 5)?),
            None => None,
        };
        Ok(OtherSale {
            entry_id: r.get(0)?,
            date: r.get(1)?,
            amount_paisa: r.get(2)?,
            qty: r.get(3)?,
            shop_name: r.get(4)?,
            item,
        })
    })?;
    rows.collect()
}

/// The first day with a sale of the garden's goods, in our shop or another.
///
/// # Errors
/// Any SQLite error.
pub fn first_sale_day(conn: &Connection, venture_id: i64) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT MIN(d) FROM (
             SELECT MIN(s.entry_date) AS d FROM ventures v
             JOIN bikri_lots l ON l.seller_account_id = v.account_id AND l.removed_at IS NULL
             JOIN bikri_sales s ON s.lot_id = l.id AND s.is_active = 1
             WHERE v.id = ?1
             UNION ALL
             SELECT MIN(e.entry_date) FROM venture_entries e
             WHERE e.venture_id = ?1 AND e.kind = 'other_sale' AND e.status = 'saved')",
        [venture_id],
        |r| r.get(0),
    )
}
