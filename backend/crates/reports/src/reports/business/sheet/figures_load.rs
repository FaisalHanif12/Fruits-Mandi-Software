//! The queries behind the day figures of a party (plan 42): the ledger by day and by kind of
//! posting, the sale lines of buyers and of sellers, and the lot charges of the first postings.
//!
//! Every money value is read on its own and checked with [`paisa`]; the sums are made in Rust
//! with checked operations (agent Section 9). Each query takes a range of days and an optional
//! party, so a slip (one party, one day) and the day total sheet (every party) use the same code
//! and the same rows (agent rule 6: one value, one source). The fragments are constants of this
//! file; a value is only ever bound as a parameter (Code Standards B14).

use super::activity::SAME_DAY_PAIR;
use super::*;
use mandi_db::rusqlite::params_from_iter;
use mandi_db::rusqlite::types::Value;

/// The party a query is for. `person` is only for a پختہ کھاتہ account.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Filter<'a> {
    pub(super) account: Option<i64>,
    pub(super) person: Option<&'a str>,
}

impl Filter<'_> {
    /// True when no party is asked for: the whole book.
    pub(super) fn whole_book(&self) -> bool {
        self.account.is_none() && self.person.is_none()
    }

    /// The ` AND ...` fragment for the party, and its values pushed after `args` (which already
    /// holds the dates). The placeholders are numbered after the ones already used.
    fn sql(&self, column_account: &str, column_person: &str, args: &mut Vec<Value>) -> String {
        let mut sql = String::new();
        if let Some(account) = self.account {
            args.push(Value::Integer(account));
            sql.push_str(&format!(" AND {column_account} = ?{}", args.len()));
        }
        if let Some(person) = self.person {
            args.push(Value::Text(person.to_owned()));
            sql.push_str(&format!(" AND {column_person} = ?{}", args.len()));
        }
        sql
    }
}

/// One day of one party on the books: all posted lines (for the running balance), and the lines
/// of the day without the two halves of a same-day cancel or edit, by kind of posting.
#[derive(Debug, Clone)]
pub(super) struct LedgerRow {
    pub(super) account_id: i64,
    pub(super) person: String,
    pub(super) buyer: bool,
    pub(super) date: String,
    pub(super) all_dr: Paisa,
    pub(super) all_cr: Paisa,
    pub(super) dr: Paisa,
    pub(super) cr: Paisa,
    /// Debits and credits of بکری postings (the sale, and the cash part).
    pub(super) bikri_dr: Paisa,
    pub(super) bikri_cr: Paisa,
    /// Credits of receipts (a buyer paid).
    pub(super) receipts_cr: Paisa,
    /// Debits of payments (a seller was paid).
    pub(super) payments_dr: Paisa,
}

/// The balance of a party before the first day.
#[derive(Debug, Clone)]
pub(super) struct BaseRow {
    pub(super) account_id: i64,
    pub(super) person: String,
    pub(super) buyer: bool,
    pub(super) dr: Paisa,
    pub(super) cr: Paisa,
}

/// The tables of a query for the lines of the buyers and the sellers on some days. For the whole
/// book the entries of the days are read first and their lines after (plan 48 P48-S3): the app's
/// file has no table statistics, and without them SQLite starts from the buyer and seller accounts
/// and reads every line they ever had (2 seconds for one day on 3.5 million lines). For one party
/// his own lines come first, as before. The rows are the same either way.
fn lines_of_days(entries_first: bool) -> &'static str {
    if !entries_first {
        "FROM journal_lines l
           JOIN journal_entries e ON e.id = l.entry_id
           JOIN accounts a ON a.id = l.account_id
           JOIN account_controls k ON k.id = a.control_id"
    } else {
        "FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
           CROSS JOIN journal_lines l ON l.entry_id = e.id
           CROSS JOIN accounts a ON a.id = l.account_id
           CROSS JOIN account_controls k ON k.id = a.control_id"
    }
}

/// The ledger by party and day for the days from `from` to `to`.
pub(super) fn ledger_rows(
    conn: &Connection,
    from: &str,
    to: &str,
    who: Filter<'_>,
) -> Result<Vec<LedgerRow>, ReportsError> {
    ledger_rows_in(conn, from, to, who, who.whole_book())
}

/// [`ledger_rows`] in a chosen form: the entries of the days first, or the lines of the accounts
/// first. The same rows either way; a test reads both.
pub(super) fn ledger_rows_in(
    conn: &Connection,
    from: &str,
    to: &str,
    who: Filter<'_>,
    entries_first: bool,
) -> Result<Vec<LedgerRow>, ReportsError> {
    let mut args = vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())];
    let filter = who.sql("l.account_id", "l.pukhta_person_key", &mut args);
    let from_lines = lines_of_days(entries_first);
    // The pair test is worked out once for each line, in the inner query.
    let sql = format!(
        "SELECT account_id, person, code, entry_date, SUM(dr), SUM(cr),
                SUM(CASE WHEN paired THEN 0 ELSE dr END), SUM(CASE WHEN paired THEN 0 ELSE cr END),
                SUM(CASE WHEN paired OR NOT is_bikri THEN 0 ELSE dr END),
                SUM(CASE WHEN paired OR NOT is_bikri THEN 0 ELSE cr END),
                SUM(CASE WHEN paired OR posting_type <> 'receipt' THEN 0 ELSE cr END),
                SUM(CASE WHEN paired OR posting_type <> 'payment' THEN 0 ELSE dr END)
         FROM (
           SELECT l.account_id AS account_id, COALESCE(l.pukhta_person_key, '') AS person,
                  k.code AS code, e.entry_date AS entry_date, l.dr_paisa AS dr, l.cr_paisa AS cr,
                  ({SAME_DAY_PAIR}) AS paired, e.posting_type AS posting_type,
                  (e.source_type = 'bikri' AND e.posting_type IN ('bikri_first', 'bikri_later'))
                      AS is_bikri
           {from_lines}
           WHERE k.code IN ('{BUYER_CONTROL}', '{SELLER_CONTROL}') AND e.status = 'posted'
             AND e.entry_date >= ?1 AND e.entry_date <= ?2{filter})
         GROUP BY account_id, person, code, entry_date
         ORDER BY account_id, person, entry_date"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                [
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                    r.get::<_, i64>(8)?,
                    r.get::<_, i64>(9)?,
                    r.get::<_, i64>(10)?,
                    r.get::<_, i64>(11)?,
                ],
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (account_id, person, code, date, v) in rows {
        let [
            all_dr,
            all_cr,
            dr,
            cr,
            bikri_dr,
            bikri_cr,
            receipts_cr,
            payments_dr,
        ] = v;
        out.push(LedgerRow {
            account_id,
            person,
            buyer: code == BUYER_CONTROL,
            date,
            all_dr: paisa(all_dr)?,
            all_cr: paisa(all_cr)?,
            dr: paisa(dr)?,
            cr: paisa(cr)?,
            bikri_dr: paisa(bikri_dr)?,
            bikri_cr: paisa(bikri_cr)?,
            receipts_cr: paisa(receipts_cr)?,
            payments_dr: paisa(payments_dr)?,
        });
    }
    Ok(out)
}

/// What each party had before `from`: every posted line dated earlier. Read by
/// `mandi_db::party_reads::totals_before` (plan 48 P48-S3): the whole book is found from the
/// covering index less the few lines of the entries on or after `from`, a slip for one party line by
/// line. The rows and the sums are the ones the line-by-line SQL gave (a test compares them).
pub(super) fn base_rows(
    conn: &Connection,
    from: &str,
    who: Filter<'_>,
) -> Result<Vec<BaseRow>, ReportsError> {
    let scope = mandi_db::party_reads::PartyScope {
        account_id: who.account,
        person_key: who.person,
    };
    let rows =
        mandi_db::party_reads::totals_before(conn, &[BUYER_CONTROL, SELLER_CONTROL], from, scope)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(BaseRow {
            account_id: row.account_id,
            person: row.person_key,
            buyer: row.control_code == BUYER_CONTROL,
            dr: paisa(row.dr_paisa)?,
            cr: paisa(row.cr_paisa)?,
        });
    }
    Ok(out)
}

/// The names of the item and the packing of a line, in both scripts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Goods {
    pub(super) item_id: i64,
    pub(super) packing_id: i64,
    pub(super) item: Names,
    pub(super) packing: Names,
}

/// One sale line as a buyer was charged for it (PRD C5), as stored.
#[derive(Debug, Clone)]
pub(super) struct BuyerLine {
    pub(super) day: String,
    pub(super) account_id: i64,
    pub(super) person: String,
    pub(super) bikri_id: i64,
    pub(super) goods: Goods,
    pub(super) qty: i64,
    /// The weight in grams of a line of an item sold by weight; `None` for any other line.
    pub(super) weight_grams: Option<i64>,
    pub(super) rate: Paisa,
    pub(super) value: Paisa,
    pub(super) commission: Paisa,
    pub(super) laga: Paisa,
    pub(super) total: Paisa,
    pub(super) cash: bool,
}

/// One sale line as the seller of its lot was credited for it, as stored.
#[derive(Debug, Clone)]
pub(super) struct SellerLine {
    pub(super) day: String,
    pub(super) account_id: i64,
    pub(super) person: String,
    pub(super) bikri_id: i64,
    pub(super) goods: Goods,
    pub(super) qty: i64,
    /// The weight in grams of a line of an item sold by weight; `None` for any other line.
    pub(super) weight_grams: Option<i64>,
    pub(super) rate: Paisa,
    pub(super) amount: Paisa,
    pub(super) commission: Paisa,
}

fn names(ur: String, en: String) -> Names {
    Names { ur, en }
}

/// The active sale lines dated from `from` to `to`, by buyer.
pub(super) fn buyer_lines(
    conn: &Connection,
    from: &str,
    to: &str,
    who: Filter<'_>,
) -> Result<Vec<BuyerLine>, ReportsError> {
    let mut args = vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())];
    let filter = who.sql("s.buyer_account_id", "s.pukhta_person_key", &mut args);
    let sql = format!(
        "SELECT e.entry_date, s.buyer_account_id, COALESCE(s.pukhta_person_key, ''), s.bikri_id,
                l.item_id, l.packing_id, i.name_ur, i.name_en, pk.name_ur, pk.name_en,
                s.qty, s.buyer_rate_paisa, s.buyer_value_paisa, s.buyer_comm_paisa,
                s.laga_paisa, s.buyer_total_paisa, s.payment_type = 'cash', s.weight_grams
         FROM bikri_sales s
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN items i ON i.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         JOIN journal_entries e ON e.id = s.journal_entry_id
         WHERE s.is_active = 1 AND e.entry_date >= ?1 AND e.entry_date <= ?2{filter}
         ORDER BY e.entry_date, s.bikri_id, s.line_no"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                [
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(8)?,
                    r.get::<_, String>(9)?,
                ],
                [
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, i64>(10)?,
                    r.get::<_, i64>(11)?,
                    r.get::<_, i64>(12)?,
                    r.get::<_, i64>(13)?,
                    r.get::<_, i64>(14)?,
                    r.get::<_, i64>(15)?,
                ],
                r.get::<_, bool>(16)?,
                r.get::<_, Option<i64>>(17)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (text, n, cash, weight_grams) in rows {
        let [day, person, item_ur, item_en, packing_ur, packing_en] = text;
        let [
            account_id,
            bikri_id,
            item_id,
            packing_id,
            qty,
            rate,
            value,
            commission,
            laga,
            total,
        ] = n;
        out.push(BuyerLine {
            day,
            account_id,
            person,
            bikri_id,
            goods: Goods {
                item_id,
                packing_id,
                item: names(item_ur, item_en),
                packing: names(packing_ur, packing_en),
            },
            qty,
            weight_grams,
            rate: paisa(rate)?,
            value: paisa(value)?,
            commission: paisa(commission)?,
            laga: paisa(laga)?,
            total: paisa(total)?,
            cash,
        });
    }
    Ok(out)
}

/// The sale lines and their lots on some days. For the whole book the entries of the days come
/// first (see [`lines_of_days`]): without table statistics SQLite would read every lot with no
/// purchase and every sale line of it. For one party his lots come first, as before.
fn sales_of_days(entries_first: bool) -> &'static str {
    if !entries_first {
        "FROM bikri_sales s
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN journal_entries e ON e.id = s.journal_entry_id"
    } else {
        "FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
         CROSS JOIN bikri_sales s ON s.journal_entry_id = e.id
         CROSS JOIN bikri_lots l ON l.id = s.lot_id"
    }
}

/// The active sale lines dated from `from` to `to`, by the seller of the lot. Lots of the trips'
/// own goods (`purchase_id` set) are left out: their credit goes to the partners, not to a seller
/// (PRD FR-12.26). Gardens and الگ کھاتے are left out by the control of the seller account.
pub(super) fn seller_lines(
    conn: &Connection,
    from: &str,
    to: &str,
    who: Filter<'_>,
) -> Result<Vec<SellerLine>, ReportsError> {
    seller_lines_in(conn, from, to, who, who.whole_book())
}

/// [`seller_lines`] in a chosen form (see [`ledger_rows_in`]).
pub(super) fn seller_lines_in(
    conn: &Connection,
    from: &str,
    to: &str,
    who: Filter<'_>,
    entries_first: bool,
) -> Result<Vec<SellerLine>, ReportsError> {
    let mut args = vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())];
    let filter = who.sql("l.seller_account_id", "l.pukhta_person_key", &mut args);
    let from_sales = sales_of_days(entries_first);
    let sql = format!(
        "SELECT e.entry_date, l.seller_account_id, COALESCE(l.pukhta_person_key, ''), s.bikri_id,
                l.item_id, l.packing_id, i.name_ur, i.name_en, pk.name_ur, pk.name_en,
                s.qty, s.seller_rate_paisa, s.seller_amount_paisa, s.seller_comm_paisa,
                s.weight_grams
         {from_sales}
         JOIN items i ON i.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         JOIN accounts a ON a.id = l.seller_account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE s.is_active = 1 AND k.code = '{SELLER_CONTROL}' AND l.purchase_id IS NULL
           AND e.entry_date >= ?1 AND e.entry_date <= ?2{filter}
         ORDER BY e.entry_date, s.bikri_id, s.lot_id, s.line_no, s.id"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                [
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(8)?,
                    r.get::<_, String>(9)?,
                ],
                [
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, i64>(10)?,
                    r.get::<_, i64>(11)?,
                    r.get::<_, i64>(12)?,
                    r.get::<_, i64>(13)?,
                ],
                r.get::<_, Option<i64>>(14)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (text, n, weight_grams) in rows {
        let [day, person, item_ur, item_en, packing_ur, packing_en] = text;
        let [
            account_id,
            bikri_id,
            item_id,
            packing_id,
            qty,
            rate,
            amount,
            commission,
        ] = n;
        out.push(SellerLine {
            day,
            account_id,
            person,
            bikri_id,
            goods: Goods {
                item_id,
                packing_id,
                item: names(item_ur, item_en),
                packing: names(packing_ur, packing_en),
            },
            qty,
            weight_grams,
            rate: paisa(rate)?,
            amount: paisa(amount)?,
            commission: paisa(commission)?,
        });
    }
    Ok(out)
}

/// The charges of one lot, charged once, in its first posting (PRD FR-11.9, C8), and the cash
/// given to the seller with it.
#[derive(Debug, Clone)]
pub(super) struct LotCharges {
    pub(super) day: String,
    pub(super) account_id: i64,
    pub(super) person: String,
    pub(super) freight: Paisa,
    pub(super) labour: Paisa,
    pub(super) munshiana: Paisa,
    pub(super) phone: Paisa,
    pub(super) store_rent: Paisa,
    pub(super) cash: Paisa,
}

/// The lots whose first posting is dated from `from` to `to`, a posting cancelled or edited the
/// same day left out. Read from the lots alone, with no join to the sale lines: a lot with nothing
/// sold in its first posting still has its charges, and they debit the seller (PRD FR-12.7).
pub(super) fn lot_charges(
    conn: &Connection,
    from: &str,
    to: &str,
    who: Filter<'_>,
) -> Result<Vec<LotCharges>, ReportsError> {
    lot_charges_in(conn, from, to, who, who.whole_book())
}

/// [`lot_charges`] in a chosen form (see [`ledger_rows_in`]).
pub(super) fn lot_charges_in(
    conn: &Connection,
    from: &str,
    to: &str,
    who: Filter<'_>,
    entries_first: bool,
) -> Result<Vec<LotCharges>, ReportsError> {
    let mut args = vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())];
    let filter = who.sql("l.seller_account_id", "l.pukhta_person_key", &mut args);
    let from_lots = if !entries_first {
        "FROM bikri_lots l
         JOIN journal_entries e ON e.id = l.first_entry_id"
    } else {
        // The entries of the days first (see [`lines_of_days`]).
        "FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
         CROSS JOIN bikri_lots l ON l.first_entry_id = e.id"
    };
    let sql = format!(
        "SELECT e.entry_date, l.seller_account_id, COALESCE(l.pukhta_person_key, ''),
                l.freight_paisa, l.freight_paid_by = 'firm', l.labour_paisa, l.munshiana_paisa,
                l.phone_paisa, l.store_rent_paisa, l.cash_paid_paisa
         {from_lots}
         JOIN accounts a ON a.id = l.seller_account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code = '{SELLER_CONTROL}' AND l.purchase_id IS NULL AND e.status = 'posted'
           AND NOT ({SAME_DAY_PAIR})
           AND e.entry_date >= ?1 AND e.entry_date <= ?2{filter}
         ORDER BY e.entry_date, l.id"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, bool>(4)?,
                [
                    r.get::<_, i64>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                    r.get::<_, i64>(8)?,
                    r.get::<_, i64>(9)?,
                ],
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (day, account_id, person, freight, firm_paid, [labour, munshiana, phone, rent, cash]) in
        rows
    {
        out.push(LotCharges {
            day,
            account_id,
            person,
            // Freight the seller paid the driver himself is shown on the lot but never deducted.
            freight: if firm_paid {
                paisa(freight)?
            } else {
                Paisa::ZERO
            },
            labour: paisa(labour)?,
            munshiana: paisa(munshiana)?,
            phone: paisa(phone)?,
            store_rent: paisa(rent)?,
            cash: paisa(cash)?,
        });
    }
    Ok(out)
}
