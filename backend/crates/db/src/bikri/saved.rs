//! Reads of saved بکری (PRD FR-12.7, FR-12.13, FR-13.6, FR-7.29, FR-5.4, plan 13): one بکری with
//! its stored values, the list and the find, first / previous / next / last, the one-time people
//! and the lines of the پختہ کھاتہ register. Everything here only reads (Code Standards B29), and
//! every number is a stored value: nothing is worked out again (Code Standards B20).
//!
//! A sum of money is a `SUM()` on `INTEGER` columns only (money agent Section 9).

use std::collections::HashMap;

use mandi_core::accounts::RecordRef;
use mandi_core::bikri::{
    BikriDetail, BikriHit, BikriListRow, BikriMatch, BikriStatus, FreightPaidBy, LotChargeMethod,
    NavigateTo, PaymentType, PukhtaPerson, PukhtaSide, SaleState, SavedCharge, SavedCredit,
    SavedLot, SavedLotCharge, SavedOwnLot, SavedPosting, SavedPukhta, SavedPurchaseRef, SavedSale,
    SavedSplit, SavedTotals, StoredMethod,
};
use mandi_core::money::Paisa;
use rusqlite::types::FromSql;
use rusqlite::{Connection, OptionalExtension, Row, params};

/// Reads the columns of a row one after the other, so a long SELECT has no index to count.
struct Cols<'a> {
    row: &'a Row<'a>,
    at: usize,
}

fn bad(column: usize, what: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        rusqlite::types::Type::Text,
        format!("bikri: {what}").into(),
    )
}

impl<'a> Cols<'a> {
    fn new(row: &'a Row<'a>) -> Self {
        Self { row, at: 0 }
    }

    fn get<T: FromSql>(&mut self) -> rusqlite::Result<T> {
        let value = self.row.get(self.at)?;
        self.at = self.at.saturating_add(1);
        Ok(value)
    }

    fn paisa(&mut self) -> rusqlite::Result<Paisa> {
        let at = self.at;
        let value: i64 = self.get()?;
        Paisa::try_from_paisa(value).map_err(|_| bad(at, "an amount that is not a whole rupee"))
    }

    fn opt_paisa(&mut self) -> rusqlite::Result<Option<Paisa>> {
        let at = self.at;
        let value: Option<i64> = self.get()?;
        value
            .map(|v| {
                Paisa::try_from_paisa(v).map_err(|_| bad(at, "an amount that is not a whole rupee"))
            })
            .transpose()
    }

    fn record(&mut self) -> rusqlite::Result<RecordRef> {
        Ok(RecordRef {
            id: self.get()?,
            code: self.get()?,
            name_ur: self.get()?,
            name_en: self.get()?,
        })
    }

    /// Four columns that are all NULL when a LEFT JOIN found nothing.
    fn opt_record(&mut self) -> rusqlite::Result<Option<RecordRef>> {
        let id: Option<i64> = self.get()?;
        let code: Option<String> = self.get()?;
        let name_ur: Option<String> = self.get()?;
        let name_en: Option<String> = self.get()?;
        Ok(id.map(|id| RecordRef {
            id,
            code: code.unwrap_or_default(),
            name_ur: name_ur.unwrap_or_default(),
            name_en: name_en.unwrap_or_default(),
        }))
    }

    /// The four columns of a one-time person; `None` when there is no person key.
    fn pukhta(&mut self) -> rusqlite::Result<Option<SavedPukhta>> {
        let name_ur: Option<String> = self.get()?;
        let name_en: Option<String> = self.get()?;
        let phone: Option<String> = self.get()?;
        let key: Option<String> = self.get()?;
        Ok(key.map(|person_key| SavedPukhta {
            name_ur: name_ur.unwrap_or_default(),
            name_en: name_en.unwrap_or_default(),
            phone: phone.unwrap_or_default(),
            person_key,
        }))
    }

    fn status(&mut self) -> rusqlite::Result<BikriStatus> {
        let at = self.at;
        let text: String = self.get()?;
        BikriStatus::parse(&text).ok_or_else(|| bad(at, "a status that is not saved or cancelled"))
    }
}

fn sale_charge(c: &mut Cols<'_>) -> rusqlite::Result<SavedCharge> {
    let at = c.at;
    let method: String = c.get()?;
    let value_bps: Option<i32> = c.get()?;
    let value_paisa = c.opt_paisa()?;
    let rule_id: Option<i64> = c.get()?;
    let amount = c.paisa()?;
    Ok(SavedCharge {
        method: StoredMethod::parse(&method).ok_or_else(|| bad(at, "a charge method"))?,
        value_bps,
        value_paisa,
        amount,
        rule_id,
    })
}

fn lot_charge(c: &mut Cols<'_>) -> rusqlite::Result<SavedLotCharge> {
    let amount = c.paisa()?;
    let at = c.at;
    let method: String = c.get()?;
    let value_paisa = c.paisa()?;
    let rule_id: Option<i64> = c.get()?;
    Ok(SavedLotCharge {
        method: LotChargeMethod::parse(&method).ok_or_else(|| bad(at, "a lot charge method"))?,
        value_paisa,
        amount,
        rule_id,
    })
}

fn read_sale(c: &mut Cols<'_>) -> rusqlite::Result<(i64, SavedSale)> {
    let lot_id: i64 = c.get()?;
    let id: i64 = c.get()?;
    let line_no: i64 = c.get()?;
    let journal_entry_id: i64 = c.get()?;
    let entry_date: String = c.get()?;
    let buyer = c.record()?;
    let pukhta = c.pukhta()?;
    let qty: i64 = c.get()?;
    let weight_grams: Option<i64> = c.get()?;
    let seller_rate = c.paisa()?;
    let seller_amount = c.paisa()?;
    let seller_comm = sale_charge(c)?;
    let buyer_rate = c.paisa()?;
    let buyer_value = c.paisa()?;
    let buyer_comm = sale_charge(c)?;
    let laga = sale_charge(c)?;
    let buyer_total = c.paisa()?;
    let rate_diff = c.paisa()?;
    let at = c.at;
    let payment: String = c.get()?;
    let is_active: bool = c.get()?;
    let ended_at = c.at;
    let ended: Option<String> = c.get()?;
    let note: Option<String> = c.get()?;
    let cost_paisa = c.paisa()?;
    Ok((
        lot_id,
        SavedSale {
            id,
            line_no,
            journal_entry_id,
            entry_date,
            buyer,
            pukhta,
            qty,
            weight_grams,
            seller_rate,
            seller_amount,
            seller_comm,
            buyer_rate,
            buyer_value,
            buyer_comm,
            laga,
            buyer_total,
            rate_diff,
            cost_paisa,
            payment_type: PaymentType::parse(&payment).ok_or_else(|| bad(at, "a payment type"))?,
            is_active,
            state: SaleState::parse(ended.as_deref())
                .ok_or_else(|| bad(ended_at, "a sale line state"))?,
            note,
        },
    ))
}

const SALE_COLUMNS: &str = "s.lot_id, s.id, s.line_no, s.journal_entry_id, s.entry_date,
    ba.id, ba.code, ba.name_ur, ba.name_en,
    s.pukhta_name_ur, s.pukhta_name_en, s.pukhta_phone, s.pukhta_person_key,
    s.qty, s.weight_grams, s.seller_rate_paisa, s.seller_amount_paisa,
    s.seller_comm_method, s.seller_comm_value_bps, s.seller_comm_value_paisa, s.seller_comm_rule_id, s.seller_comm_paisa,
    s.buyer_rate_paisa, s.buyer_value_paisa,
    s.buyer_comm_method, s.buyer_comm_value_bps, s.buyer_comm_value_paisa, s.buyer_comm_rule_id, s.buyer_comm_paisa,
    s.laga_method, s.laga_value_bps, s.laga_value_paisa, s.laga_rule_id, s.laga_paisa,
    s.buyer_total_paisa, s.rate_diff_paisa, s.payment_type, s.is_active, s.ended, s.note, s.cost_paisa";

fn read_lot(c: &mut Cols<'_>) -> rusqlite::Result<SavedLot> {
    let id: i64 = c.get()?;
    let lot_no: i64 = c.get()?;
    let seller = c.record()?;
    let pukhta = c.pukhta()?;
    let item = c.record()?;
    let packing = c.record()?;
    let marka_id: Option<i64> = c.get()?;
    let marka_text: Option<String> = c.get()?;
    let store_id: Option<i64> = c.get()?;
    let qty_received: i64 = c.get()?;
    let qty_sold: i64 = c.get()?;
    let qty_written_off: i64 = c.get()?;
    let weight_received_grams: Option<i64> = c.get()?;
    let freight = c.paisa()?;
    let at = c.at;
    let paid_by: String = c.get()?;
    let labour = lot_charge(c)?;
    let munshiana = lot_charge(c)?;
    let phone = lot_charge(c)?;
    let cash_paid = c.paisa()?;
    let store_rent = c.paisa()?;
    let adjust_advance: bool = c.get()?;
    let first_entry_id: Option<i64> = c.get()?;
    let gross = c.paisa()?;
    let seller_comm = c.paisa()?;
    let deductions = c.paisa()?;
    let net = c.paisa()?;
    let purchase_id: Option<i64> = c.get()?;
    let purchase_number: Option<i64> = c.get()?;
    let trip_id: Option<i64> = c.get()?;
    let trip_number: Option<i64> = c.get()?;
    Ok(SavedLot {
        id,
        lot_no,
        seller,
        purchase: match (purchase_id, purchase_number, trip_id, trip_number) {
            (Some(id), Some(number), Some(trip_id), Some(trip_number)) => Some(SavedPurchaseRef {
                id,
                number,
                trip_id,
                trip_number,
            }),
            _ => None,
        },
        // Filled by `fill_own_lots` after the lines are read (PRD FR-12.27).
        own: None,
        pukhta,
        item,
        packing,
        marka_id,
        marka_text,
        store_id,
        qty_received,
        qty_sold,
        qty_written_off,
        weight_received_grams,
        freight,
        freight_paid_by: FreightPaidBy::parse(&paid_by)
            .ok_or_else(|| bad(at, "who paid freight"))?,
        labour,
        munshiana,
        phone,
        cash_paid,
        store_rent,
        adjust_advance,
        first_entry_id,
        gross,
        seller_comm,
        deductions,
        net,
        sales: Vec::new(),
        totals: SavedTotals::default(),
    })
}

/// Fills what a trip lot shows besides its lines (PRD FR-12.26, FR-12.27): the cost of the goods
/// its lines in force sold, the seller credit that stands, the result, and each trip partner's part.
/// Every number is a stored value: the lines' `cost_paisa` and the rows of `lot_credit_parts` (a
/// reversal's rows are the opposite, so the sums are what stands now).
fn fill_own_lots(conn: &Connection, lots: &mut [SavedLot]) -> rusqlite::Result<()> {
    let total_error = |_| bad(0, "a total that does not fit");
    for lot in lots.iter_mut().filter(|l| l.purchase.is_some()) {
        let cost: i64 = conn.query_row(
            "SELECT COALESCE(SUM(cost_paisa), 0) FROM bikri_sales WHERE lot_id = ?1 AND is_active = 1",
            [lot.id],
            |r| r.get(0),
        )?;
        let mut statement = conn.prepare(
            "SELECT p.id, p.name_ur, p.name_en, p.venture_account_id, MAX(c.share_bps), SUM(c.amount_paisa)
             FROM lot_credit_parts c JOIN partners p ON p.id = c.partner_id
             WHERE c.lot_id = ?1
             GROUP BY p.id, p.name_ur, p.name_en, p.venture_account_id ORDER BY p.id",
        )?;
        let rows = statement.query_map([lot.id], |row| {
            let mut c = Cols::new(row);
            let id: i64 = c.get()?;
            let name_ur: String = c.get()?;
            let name_en: String = c.get()?;
            let account_id: Option<i64> = c.get()?;
            let share_bps: i32 = c.get()?;
            Ok(SavedCredit {
                partner: RecordRef {
                    id,
                    code: String::new(),
                    name_ur,
                    name_en,
                },
                account_id: account_id.unwrap_or_default(),
                share_bps,
                amount_paisa: c.paisa()?,
            })
        })?;
        let credits = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        let mut credit = Paisa::ZERO;
        for part in &credits {
            credit = credit.checked_add(part.amount_paisa).map_err(total_error)?;
        }
        let cost = Paisa::try_from_paisa(cost).map_err(total_error)?;
        lot.own = Some(SavedOwnLot {
            cost_paisa: cost,
            credit_paisa: credit,
            result_paisa: credit.checked_sub(cost).map_err(total_error)?,
            credits,
        });
    }
    Ok(())
}

/// One saved بکری with its lots, lines, postings and partner split, or `None`.
///
/// # Errors
/// Any SQLite error, or a stored value that is not in the form this code wrote.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<BikriDetail>> {
    let head = conn
        .query_row(
            "SELECT e.id, e.version, e.number, e.book, e.entry_date, e.bill_date, e.truck_no, e.adda_id, e.driver_name,
                    e.driver_phone, e.total_freight_paisa, e.store_id, e.note, e.status, e.cancel_reason,
                    e.print_count, e.created_at, z.id, z.code, z.name_ur, z.name_en
             FROM bikri_entries e LEFT JOIN accounts z ON z.id = e.zamindar_account_id
             WHERE e.id = ?1",
            [id],
            |row| {
                let mut c = Cols::new(row);
                Ok(BikriDetail {
                    id: c.get()?,
                    version: c.get()?,
                    number: c.get()?,
                    book: c.get()?,
                    entry_date: c.get()?,
                    bill_date: c.get()?,
                    truck_no: c.get()?,
                    adda_id: c.get()?,
                    driver_name: c.get()?,
                    driver_phone: c.get()?,
                    total_freight: c.opt_paisa()?,
                    store_id: c.get()?,
                    note: c.get()?,
                    status: c.status()?,
                    cancel_reason: c.get()?,
                    print_count: c.get()?,
                    created_at: c.get()?,
                    zamindar: c.opt_record()?,
                    lots: Vec::new(),
                    postings: Vec::new(),
                    split: Vec::new(),
                    totals: SavedTotals::default(),
                })
            },
        )
        .optional()?;
    let Some(mut detail) = head else {
        return Ok(None);
    };

    let mut statement = conn.prepare(
        "SELECT l.id, l.lot_no, sa.id, sa.code, sa.name_ur, sa.name_en,
                l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone, l.pukhta_person_key,
                it.id, it.code, it.name_ur, it.name_en, pk.id, pk.code, pk.name_ur, pk.name_en,
                l.marka_id, mk.mark_text, l.store_id, l.qty_received, l.qty_sold, l.qty_written_off,
                l.weight_received_grams, l.freight_paisa, l.freight_paid_by,
                l.labour_paisa, l.labour_method, l.labour_value_paisa, l.labour_rule_id,
                l.munshiana_paisa, l.munshiana_method, l.munshiana_value_paisa, l.munshiana_rule_id,
                l.phone_paisa, l.phone_method, l.phone_value_paisa, l.phone_rule_id,
                l.cash_paid_paisa, l.store_rent_paisa, l.adjust_advance, l.first_entry_id,
                l.gross_paisa, l.seller_comm_paisa, l.deductions_paisa, l.net_paisa,
                l.purchase_id, pu.number, pu.venture_id, tr.number
         FROM bikri_lots l
         LEFT JOIN purchases pu ON pu.id = l.purchase_id
         LEFT JOIN ventures tr ON tr.id = pu.venture_id
         JOIN accounts sa ON sa.id = l.seller_account_id
         JOIN items it ON it.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         LEFT JOIN markas mk ON mk.id = l.marka_id
         WHERE l.bikri_id = ?1 ORDER BY l.lot_no",
    )?;
    let lots = statement.query_map([id], |row| read_lot(&mut Cols::new(row)))?;
    detail.lots = lots.collect::<rusqlite::Result<Vec<_>>>()?;

    let mut statement = conn.prepare(&format!(
        "SELECT {SALE_COLUMNS} FROM bikri_sales s JOIN accounts ba ON ba.id = s.buyer_account_id
         WHERE s.bikri_id = ?1 ORDER BY s.lot_id, s.line_no, s.id"
    ))?;
    let sales = statement.query_map([id], |row| read_sale(&mut Cols::new(row)))?;
    for sale in sales {
        let (lot_id, sale) = sale?;
        if let Some(lot) = detail.lots.iter_mut().find(|l| l.id == lot_id) {
            lot.sales.push(sale);
        }
    }

    fill_own_lots(conn, &mut detail.lots)?;

    // The totals of each lot and of the whole بکری: sums of the stored values (Code Standards B20).
    let total_error = |_| bad(0, "a total that does not fit");
    let mut whole = SavedTotals::default();
    for lot in &mut detail.lots {
        lot.totals = SavedTotals::of_lot(lot).map_err(total_error)?;
        whole = whole.plus(&lot.totals).map_err(total_error)?;
    }
    detail.totals = whole;

    let mut statement = conn.prepare(
        "SELECT e.id, e.posting_type, e.entry_date, e.total_paisa,
                EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
         FROM journal_entries e
         WHERE e.source_type = 'bikri' AND e.source_id = ?1 AND e.status = 'posted' ORDER BY e.id",
    )?;
    let postings = statement.query_map([id], |row| {
        let mut c = Cols::new(row);
        Ok(SavedPosting {
            entry_id: c.get()?,
            posting_type: c.get()?,
            entry_date: c.get()?,
            total: c.paisa()?,
            reversed: c.get()?,
            // Set by the service, which knows the closed-up-to day.
            editable: false,
        })
    })?;
    detail.postings = postings.collect::<rusqlite::Result<Vec<_>>>()?;

    let mut statement = conn.prepare(
        "SELECT p.id, p.name_ur, p.name_en, SUM(r.amount_paisa)
         FROM partner_share_rows r
         JOIN journal_entries e ON e.id = r.journal_entry_id
         JOIN partners p ON p.id = r.partner_id
         WHERE e.source_type = 'bikri' AND e.source_id = ?1 AND r.kind = 'income'
         GROUP BY p.id, p.name_ur, p.name_en ORDER BY p.id",
    )?;
    let split = statement.query_map([id], |row| {
        let mut c = Cols::new(row);
        Ok(SavedSplit {
            partner_id: c.get()?,
            name_ur: c.get()?,
            name_en: c.get()?,
            income_paisa: c.paisa()?,
        })
    })?;
    detail.split = split.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(detail))
}

/// What the list and the find ask for. A field that is `None` is not a filter.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    /// First day, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// A seller of one of its lots.
    pub seller_id: Option<i64>,
    /// An item of one of its lots.
    pub item_id: Option<i64>,
    /// `saved` or `cancelled`.
    pub status: Option<BikriStatus>,
    /// The بکری number.
    pub number: Option<i64>,
    /// The start of the truck number (letter case ignored).
    pub truck_prefix: Option<String>,
    /// The most rows.
    pub limit: i64,
}

/// Escapes `%`, `_` and `\` so typed text is taken as plain characters in a `LIKE`.
fn like_prefix(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(1));
    for c in text.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('%');
    out
}

/// The rows of the list of بکری (PRD FR-12.7), newest first (day, then number). Sums are
/// `SUM()` of stored columns. The sellers and items are read with one more statement for all
/// the rows (Code Standards B22).
///
/// # Errors
/// Any SQLite error.
pub fn list(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Vec<BikriListRow>> {
    let truck = filter.truck_prefix.as_deref().map(like_prefix);
    let status = filter.status.map(BikriStatus::as_str);
    let mut statement = conn.prepare(
        "SELECT e.id, e.number, e.entry_date, e.created_at, e.truck_no, e.status,
                (SELECT COALESCE(SUM(l.qty_received), 0) FROM bikri_lots l WHERE l.bikri_id = e.id),
                (SELECT COALESCE(SUM(l.qty_sold), 0) FROM bikri_lots l WHERE l.bikri_id = e.id),
                (SELECT COALESCE(SUM(l.gross_paisa), 0) FROM bikri_lots l WHERE l.bikri_id = e.id),
                (SELECT COALESCE(SUM(l.net_paisa), 0) FROM bikri_lots l WHERE l.bikri_id = e.id),
                (SELECT COALESCE(SUM(s.buyer_total_paisa), 0) FROM bikri_sales s
                   WHERE s.bikri_id = e.id AND s.is_active = 1)
         FROM bikri_entries e
         WHERE (?1 IS NULL OR e.entry_date >= ?1) AND (?2 IS NULL OR e.entry_date <= ?2)
           AND (?3 IS NULL OR EXISTS (SELECT 1 FROM bikri_lots l WHERE l.bikri_id = e.id AND l.seller_account_id = ?3))
           AND (?4 IS NULL OR EXISTS (SELECT 1 FROM bikri_lots l WHERE l.bikri_id = e.id AND l.item_id = ?4))
           AND (?5 IS NULL OR e.status = ?5)
           AND (?6 IS NULL OR e.number = ?6)
           AND (?7 IS NULL OR e.truck_no LIKE ?7 ESCAPE '\\')
         ORDER BY e.entry_date DESC, e.number DESC
         LIMIT ?8",
    )?;
    let rows = statement.query_map(
        params![
            filter.from,
            filter.to,
            filter.seller_id,
            filter.item_id,
            status,
            filter.number,
            truck,
            filter.limit
        ],
        |row| {
            let mut c = Cols::new(row);
            Ok(BikriListRow {
                id: c.get()?,
                number: c.get()?,
                entry_date: c.get()?,
                created_at: c.get()?,
                truck_no: c.get()?,
                status: c.status()?,
                qty_received: c.get()?,
                qty_sold: c.get()?,
                gross: c.paisa()?,
                net: c.paisa()?,
                buyer_total: c.paisa()?,
                sellers: Vec::new(),
                pukhta_sellers: Vec::new(),
                items: Vec::new(),
            })
        },
    )?;
    let mut out = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    fill_sellers_and_items(conn, &mut out)?;
    Ok(out)
}

/// Adds the sellers (each once, in lot order) and the items of each row, from one statement.
fn fill_sellers_and_items(conn: &Connection, rows: &mut [BikriListRow]) -> rusqlite::Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT l.bikri_id, sa.id, sa.code, sa.name_ur, sa.name_en,
                l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone, l.pukhta_person_key,
                it.id, it.code, it.name_ur, it.name_en
         FROM bikri_lots l JOIN accounts sa ON sa.id = l.seller_account_id JOIN items it ON it.id = l.item_id
         WHERE l.bikri_id IN ({marks}) ORDER BY l.bikri_id, l.lot_no"
    ))?;
    let lots = statement.query_map(rusqlite::params_from_iter(&ids), |row| {
        let mut c = Cols::new(row);
        let bikri_id: i64 = c.get()?;
        Ok((bikri_id, c.record()?, c.pukhta()?, c.record()?))
    })?;
    let mut by_id: HashMap<i64, usize> = HashMap::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        by_id.insert(row.id, index);
    }
    for lot in lots {
        let (bikri_id, seller, pukhta, item) = lot?;
        let Some(row) = by_id.get(&bikri_id).and_then(|i| rows.get_mut(*i)) else {
            continue;
        };
        match pukhta {
            Some(person) => row.pukhta_sellers.push(person),
            None => {
                if !row.sellers.iter().any(|s| s.id == seller.id) {
                    row.sellers.push(seller);
                }
            }
        }
        if !row.items.iter().any(|i| i.id == item.id) {
            row.items.push(item);
        }
    }
    Ok(())
}

/// The number of a بکری, or `None` if there is none with this id.
///
/// # Errors
/// Any SQLite error.
pub fn number_of(conn: &Connection, id: i64) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT number FROM bikri_entries WHERE id = ?1",
        [id],
        |r| r.get(0),
    )
    .optional()
}

/// The id of the first, previous, next or last بکری by number. `current_number` is the number
/// of the بکری on the screen, if any: previous of none is the last, next of none is the first.
/// `None` when there is nothing in that direction.
///
/// # Errors
/// Any SQLite error.
pub fn neighbour(
    conn: &Connection,
    current_number: Option<i64>,
    to: NavigateTo,
) -> rusqlite::Result<Option<i64>> {
    let sql = match (to, current_number) {
        (NavigateTo::First, _) | (NavigateTo::Next, None) => {
            "SELECT id FROM bikri_entries ORDER BY number ASC LIMIT 1"
        }
        (NavigateTo::Last, _) | (NavigateTo::Previous, None) => {
            "SELECT id FROM bikri_entries ORDER BY number DESC LIMIT 1"
        }
        (NavigateTo::Previous, Some(_)) => {
            "SELECT id FROM bikri_entries WHERE number < ?1 ORDER BY number DESC LIMIT 1"
        }
        (NavigateTo::Next, Some(_)) => {
            "SELECT id FROM bikri_entries WHERE number > ?1 ORDER BY number ASC LIMIT 1"
        }
    };
    match current_number {
        Some(number) if matches!(to, NavigateTo::Previous | NavigateTo::Next) => {
            conn.query_row(sql, [number], |r| r.get(0)).optional()
        }
        _ => conn.query_row(sql, [], |r| r.get(0)).optional(),
    }
}

fn hit(row: &Row<'_>, matched: BikriMatch) -> rusqlite::Result<BikriHit> {
    let mut c = Cols::new(row);
    Ok(BikriHit {
        id: c.get()?,
        number: c.get()?,
        entry_date: c.get()?,
        truck_no: c.get()?,
        status: c.status()?,
        matched,
    })
}

/// The بکری with exactly this number, for the global search (PRD FR-5.4).
///
/// # Errors
/// Any SQLite error.
pub fn hit_by_number(conn: &Connection, number: i64) -> rusqlite::Result<Option<BikriHit>> {
    conn.query_row(
        "SELECT id, number, entry_date, truck_no, status FROM bikri_entries WHERE number = ?1",
        [number],
        |r| hit(r, BikriMatch::Number),
    )
    .optional()
}

/// The newest بکری whose truck number starts with the text (letter case ignored), for the global
/// search (PRD FR-5.4).
///
/// # Errors
/// Any SQLite error.
pub fn hits_by_truck(
    conn: &Connection,
    prefix: &str,
    limit: i64,
) -> rusqlite::Result<Vec<BikriHit>> {
    let mut statement = conn.prepare(
        "SELECT id, number, entry_date, truck_no, status FROM bikri_entries
         WHERE truck_no LIKE ?1 ESCAPE '\\'
         ORDER BY entry_date DESC, number DESC LIMIT ?2",
    )?;
    let rows = statement.query_map(params![like_prefix(prefix), limit], |r| {
        hit(r, BikriMatch::Truck)
    })?;
    rows.collect()
}

/// Every one-time person of a side, newest first, with the names and phone as he was last
/// typed (PRD FR-7.29). The services keep the ones that match the typed text.
///
/// # Errors
/// Any SQLite error.
pub fn pukhta_people(conn: &Connection, side: PukhtaSide) -> rusqlite::Result<Vec<PukhtaPerson>> {
    // The names and phone of his latest line: the row numbered 1 in each person's own order.
    let sql = match side {
        PukhtaSide::Buyer => {
            "SELECT person_key, name_ur, name_en, phone, entry_date FROM (
                 SELECT s.pukhta_person_key AS person_key, s.pukhta_name_ur AS name_ur,
                        s.pukhta_name_en AS name_en, s.pukhta_phone AS phone, s.entry_date AS entry_date,
                        s.id AS row_id,
                        ROW_NUMBER() OVER (PARTITION BY s.pukhta_person_key
                                           ORDER BY s.entry_date DESC, s.id DESC) AS position
                 FROM bikri_sales s WHERE s.pukhta_person_key IS NOT NULL)
             WHERE position = 1 ORDER BY entry_date DESC, row_id DESC"
        }
        PukhtaSide::Seller => {
            "SELECT person_key, name_ur, name_en, phone, entry_date FROM (
                 SELECT l.pukhta_person_key AS person_key, l.pukhta_name_ur AS name_ur,
                        l.pukhta_name_en AS name_en, l.pukhta_phone AS phone, e.entry_date AS entry_date,
                        l.id AS row_id,
                        ROW_NUMBER() OVER (PARTITION BY l.pukhta_person_key
                                           ORDER BY e.entry_date DESC, l.id DESC) AS position
                 FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
                 WHERE l.pukhta_person_key IS NOT NULL)
             WHERE position = 1 ORDER BY entry_date DESC, row_id DESC"
        }
    };
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map([], |row| {
        let mut c = Cols::new(row);
        let person_key: String = c.get()?;
        let name_ur: Option<String> = c.get()?;
        let name_en: Option<String> = c.get()?;
        let phone: Option<String> = c.get()?;
        let last_date: String = c.get()?;
        Ok(PukhtaPerson {
            name_ur: name_ur.unwrap_or_default(),
            name_en: name_en.unwrap_or_default(),
            phone: phone.unwrap_or_default(),
            last_date,
            person_key,
        })
    })?;
    rows.collect()
}

/// A line of the پختہ کھاتہ register before paid or due is known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterLine {
    /// Row id of the sale line (a buyer) or the lot (a seller).
    pub row_id: i64,
    /// The day of the posting, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The بکری.
    pub bikri_id: i64,
    /// Its number.
    pub bikri_number: i64,
    /// Seller or buyer.
    pub side: PukhtaSide,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Phone, 11 digits or empty.
    pub phone: String,
    /// The person key.
    pub person_key: String,
    /// The item.
    pub item: RecordRef,
    /// نگ.
    pub qty: i64,
    /// A buyer line: the buyer total. A seller lot: خام بکری.
    pub amount: Paisa,
    /// A buyer line: the buyer commission. A seller lot: the seller commission.
    pub commission: Paisa,
    /// A seller lot: its صافی بکری, which is what is left to pay him after the نقد (C7). Zero for
    /// a buyer line.
    pub seller_net_paisa: i64,
    /// The posting of this line.
    pub journal_entry_id: i64,
    /// The account of this line (the پختہ کھاتہ account).
    pub account_id: i64,
    /// True if the بکری was cancelled.
    pub cancelled: bool,
}

/// The lines of a one-time buyer, oldest first. A line replaced by an edit does not show.
///
/// # Errors
/// Any SQLite error.
pub fn register_buyer_lines(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
) -> rusqlite::Result<Vec<RegisterLine>> {
    let mut statement = conn.prepare(
        "SELECT s.id, s.entry_date, e.id, e.number, s.pukhta_name_ur, s.pukhta_name_en, s.pukhta_phone,
                s.pukhta_person_key, it.id, it.code, it.name_ur, it.name_en, s.qty, s.buyer_total_paisa,
                s.buyer_comm_paisa, s.journal_entry_id, s.buyer_account_id, e.status
         FROM bikri_sales s
         JOIN bikri_entries e ON e.id = s.bikri_id
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN items it ON it.id = l.item_id
         WHERE s.pukhta_person_key IS NOT NULL AND s.replaced_by_sale_id IS NULL
           AND (?1 IS NULL OR s.entry_date >= ?1) AND (?2 IS NULL OR s.entry_date <= ?2)
         ORDER BY s.entry_date, s.id",
    )?;
    let rows = statement.query_map(params![from, to], |row| {
        let mut c = Cols::new(row);
        let row_id: i64 = c.get()?;
        let entry_date: String = c.get()?;
        let bikri_id: i64 = c.get()?;
        let bikri_number: i64 = c.get()?;
        let name_ur: Option<String> = c.get()?;
        let name_en: Option<String> = c.get()?;
        let phone: Option<String> = c.get()?;
        let person_key: String = c.get()?;
        let item = c.record()?;
        let qty: i64 = c.get()?;
        let amount = c.paisa()?;
        let commission = c.paisa()?;
        let journal_entry_id: i64 = c.get()?;
        let account_id: i64 = c.get()?;
        let status = c.status()?;
        Ok(RegisterLine {
            row_id,
            entry_date,
            bikri_id,
            bikri_number,
            side: PukhtaSide::Buyer,
            name_ur: name_ur.unwrap_or_default(),
            name_en: name_en.unwrap_or_default(),
            phone: phone.unwrap_or_default(),
            person_key,
            item,
            qty,
            amount,
            commission,
            seller_net_paisa: 0,
            journal_entry_id,
            account_id,
            cancelled: status == BikriStatus::Cancelled,
        })
    })?;
    rows.collect()
}

/// The lots of a one-time seller, oldest first, with what each posting credits him.
///
/// # Errors
/// Any SQLite error.
pub fn register_seller_lines(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
) -> rusqlite::Result<Vec<RegisterLine>> {
    let mut statement = conn.prepare(
        "SELECT l.id, e.entry_date, e.id, e.number, l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone,
                l.pukhta_person_key, it.id, it.code, it.name_ur, it.name_en, l.qty_received, l.gross_paisa,
                l.seller_comm_paisa, l.net_paisa, l.first_entry_id, l.seller_account_id, e.status
         FROM bikri_lots l
         JOIN bikri_entries e ON e.id = l.bikri_id
         JOIN items it ON it.id = l.item_id
         WHERE l.pukhta_person_key IS NOT NULL
           AND (?1 IS NULL OR e.entry_date >= ?1) AND (?2 IS NULL OR e.entry_date <= ?2)
         ORDER BY e.entry_date, l.id",
    )?;
    let rows = statement.query_map(params![from, to], |row| {
        let mut c = Cols::new(row);
        let row_id: i64 = c.get()?;
        let entry_date: String = c.get()?;
        let bikri_id: i64 = c.get()?;
        let bikri_number: i64 = c.get()?;
        let name_ur: Option<String> = c.get()?;
        let name_en: Option<String> = c.get()?;
        let phone: Option<String> = c.get()?;
        let person_key: String = c.get()?;
        let item = c.record()?;
        let qty: i64 = c.get()?;
        let amount = c.paisa()?;
        let commission = c.paisa()?;
        let net: i64 = c.get()?;
        let first_entry_id: Option<i64> = c.get()?;
        let account_id: i64 = c.get()?;
        let status = c.status()?;
        Ok(RegisterLine {
            row_id,
            entry_date,
            bikri_id,
            bikri_number,
            side: PukhtaSide::Seller,
            name_ur: name_ur.unwrap_or_default(),
            name_en: name_en.unwrap_or_default(),
            phone: phone.unwrap_or_default(),
            person_key,
            item,
            qty,
            amount,
            commission,
            seller_net_paisa: net,
            journal_entry_id: first_entry_id.unwrap_or_default(),
            account_id,
            cancelled: status == BikriStatus::Cancelled,
        })
    })?;
    rows.collect()
}

/// A debit line of the one-time buyer account with the part of it that is settled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillLine {
    /// The journal line.
    pub line_id: i64,
    /// Its entry.
    pub entry_id: i64,
    /// The account.
    pub account_id: i64,
    /// The person key.
    pub person_key: String,
    /// Its amount, paisa.
    pub amount_paisa: i64,
    /// The sum of its active matching rows, paisa.
    pub matched_paisa: i64,
}

/// Every debit line of the one-time buyer account with a person key, in entry and line order,
/// with the part of each that the active bill matching rows settle (PRD FR-13.6).
///
/// # Errors
/// Any SQLite error.
pub fn pukhta_bill_lines(conn: &Connection) -> rusqlite::Result<Vec<BillLine>> {
    let mut statement = conn.prepare(
        "SELECT l.id, l.entry_id, l.account_id, l.pukhta_person_key, l.dr_paisa,
                COALESCE((SELECT SUM(a.amount_paisa) FROM bill_allocations a
                          WHERE a.bill_line_id = l.id AND a.is_active = 1), 0)
         FROM journal_lines l JOIN accounts acc ON acc.id = l.account_id
         WHERE acc.is_pukhta_khata = 1 AND acc.party_type = 'buyer' AND l.dr_paisa > 0
           AND l.pukhta_person_key IS NOT NULL
         ORDER BY l.entry_id, l.line_no",
    )?;
    let rows = statement.query_map([], |row| {
        let mut c = Cols::new(row);
        Ok(BillLine {
            line_id: c.get()?,
            entry_id: c.get()?,
            account_id: c.get()?,
            person_key: c.get()?,
            amount_paisa: c.get()?,
            matched_paisa: c.get()?,
        })
    })?;
    rows.collect()
}
