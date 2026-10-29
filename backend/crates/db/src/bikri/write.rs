//! The rows of a saved بکری (PRD FR-12.8, plan 13): the بکری, its lots and lines with their
//! stored values, stock moves and analytics facts. SQL only: every number comes from
//! `mandi_domain::bikri::calc_posting`, already worked out and checked, and the CHECKs of
//! `0001_init.sql` are the last guard (Code Standards 4.1 layer 4).
//!
//! The caller is the save of `mandi_services::bikri`, inside its one transaction. Nothing here
//! deletes a row (Code Standards D3).

use rusqlite::{Connection, params};

/// The next بکری number: the biggest ever given plus one. A بکری is never deleted (a cancelled one
/// keeps its row and its number), so a number is never used again; a save that rolls back leaves
/// no gap because the number is taken inside its transaction.
///
/// # Errors
/// Any SQLite error.
pub fn next_number(conn: &Connection) -> rusqlite::Result<i64> {
    let last: i64 = conn.query_row(
        "SELECT COALESCE(MAX(number), 0) FROM bikri_entries",
        [],
        |r| r.get(0),
    )?;
    Ok(last.saturating_add(1))
}

/// The header columns of a new بکری.
#[derive(Debug, Clone)]
pub struct NewEntry<'a> {
    /// The number, from [`next_number`].
    pub number: i64,
    /// The second number.
    pub book: Option<&'a str>,
    /// The day of the first posting, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// The bill day, `YYYY-MM-DD`.
    pub bill_date: &'a str,
    /// The truck number.
    pub truck_no: Option<&'a str>,
    /// The زمیندار.
    pub zamindar_account_id: Option<i64>,
    /// The truck stand.
    pub adda_id: Option<i64>,
    /// The driver's name.
    pub driver_name: Option<&'a str>,
    /// The driver's phone, 11 digits.
    pub driver_phone: Option<&'a str>,
    /// The header's total freight, paisa.
    pub total_freight_paisa: Option<i64>,
    /// The default store.
    pub store_id: Option<i64>,
    /// A note.
    pub note: Option<&'a str>,
}

/// Adds a بکری and returns its id.
///
/// # Errors
/// Any SQLite error (a CHECK that the checks before the save should have caught).
pub fn insert_entry(
    conn: &Connection,
    entry: &NewEntry<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO bikri_entries (created_at, created_by, updated_at, updated_by, number, book, entry_date,
             bill_date, truck_no, zamindar_account_id, adda_id, driver_name, driver_phone, total_freight_paisa,
             store_id, note, status)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'saved')",
        params![
            now,
            user_id,
            entry.number,
            entry.book,
            entry.entry_date,
            entry.bill_date,
            entry.truck_no,
            entry.zamindar_account_id,
            entry.adda_id,
            entry.driver_name,
            entry.driver_phone,
            entry.total_freight_paisa,
            entry.store_id,
            entry.note,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// A one-time person kept on a lot or a line.
#[derive(Debug, Clone, Copy)]
pub struct PukhtaColumns<'a> {
    /// Urdu name.
    pub name_ur: &'a str,
    /// English name.
    pub name_en: &'a str,
    /// Phone, 11 digits or empty.
    pub phone: &'a str,
    /// The person key.
    pub person_key: &'a str,
}

/// A lot charge as the lot stores it (مزدوری, منشیانہ, فون).
#[derive(Debug, Clone, Copy)]
pub struct LotChargeColumns<'a> {
    /// The amount charged for the lot, paisa.
    pub paisa: i64,
    /// `per_unit`, `per_lot` or `manual`.
    pub method: &'a str,
    /// What the field shows (per نگ, per lot; 0 for manual: the table keeps a manual value in the
    /// amount only).
    pub value_paisa: i64,
    /// The rule that suggested it.
    pub rule_id: Option<i64>,
}

/// The columns of a new lot. The totals are the final stored values (the table checks that the
/// deductions are the sum of the charges and that net is gross less deductions).
#[derive(Debug, Clone)]
pub struct NewLot<'a> {
    /// The بکری.
    pub bikri_id: i64,
    /// The lot number, from 1.
    pub lot_no: i64,
    /// The seller account.
    pub seller_account_id: i64,
    /// The purchase an own lot takes its goods from (PRD FR-12.26).
    pub purchase_id: Option<i64>,
    /// The garden arrival a garden lot takes its goods from (PRD FR-12.40).
    pub arrival_id: Option<i64>,
    /// The one-time seller.
    pub pukhta: Option<PukhtaColumns<'a>>,
    /// The item.
    pub item_id: i64,
    /// The packing.
    pub packing_id: i64,
    /// The مارکہ.
    pub marka_id: Option<i64>,
    /// The store.
    pub store_id: Option<i64>,
    /// Quantity received.
    pub qty_received: i64,
    /// Weight received, grams.
    pub weight_received_grams: Option<i64>,
    /// Freight typed on the lot, paisa.
    pub freight_paisa: i64,
    /// `firm` or `seller`.
    pub freight_paid_by: &'a str,
    /// مزدوری.
    pub labour: LotChargeColumns<'a>,
    /// منشیانہ.
    pub munshiana: LotChargeColumns<'a>,
    /// فون.
    pub phone: LotChargeColumns<'a>,
    /// نقد, paisa.
    pub cash_paid_paisa: i64,
    /// کرایہ سٹور, paisa.
    pub store_rent_paisa: i64,
    /// The tick "پیشگی سے کاٹیں".
    pub adjust_advance: bool,
    /// Quantity sold in this posting.
    pub qty_sold: i64,
    /// خام بکری, paisa.
    pub gross_paisa: i64,
    /// The sum of the line commissions, paisa.
    pub seller_comm_paisa: i64,
    /// کل اخراجات, paisa.
    pub deductions_paisa: i64,
    /// صافی بکری, paisa. Can be negative.
    pub net_paisa: i64,
}

/// Adds a lot and returns its id.
///
/// # Errors
/// Any SQLite error (the CHECKs of `bikri_lots`).
pub fn insert_lot(
    conn: &Connection,
    lot: &NewLot<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO bikri_lots (created_at, created_by, updated_at, updated_by, bikri_id, lot_no, seller_account_id,
             pukhta_name_ur, pukhta_name_en, pukhta_phone, pukhta_person_key, item_id, packing_id, marka_id, store_id,
             qty_received, weight_received_grams, freight_paisa, freight_paid_by,
             labour_paisa, labour_method, labour_value_paisa, labour_rule_id,
             munshiana_paisa, munshiana_method, munshiana_value_paisa, munshiana_rule_id,
             phone_paisa, phone_method, phone_value_paisa, phone_rule_id,
             cash_paid_paisa, store_rent_paisa, adjust_advance, qty_sold, gross_paisa, seller_comm_paisa,
             deductions_paisa, net_paisa, purchase_id, arrival_id)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
             ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39)",
        params![
            now,
            user_id,
            lot.bikri_id,
            lot.lot_no,
            lot.seller_account_id,
            lot.pukhta.map(|p| p.name_ur),
            lot.pukhta.map(|p| p.name_en),
            lot.pukhta.map(|p| p.phone),
            lot.pukhta.map(|p| p.person_key),
            lot.item_id,
            lot.packing_id,
            lot.marka_id,
            lot.store_id,
            lot.qty_received,
            lot.weight_received_grams,
            lot.freight_paisa,
            lot.freight_paid_by,
            lot.labour.paisa,
            lot.labour.method,
            lot.labour.value_paisa,
            lot.labour.rule_id,
            lot.munshiana.paisa,
            lot.munshiana.method,
            lot.munshiana.value_paisa,
            lot.munshiana.rule_id,
            lot.phone.paisa,
            lot.phone.method,
            lot.phone.value_paisa,
            lot.phone.rule_id,
            lot.cash_paid_paisa,
            lot.store_rent_paisa,
            lot.adjust_advance,
            lot.qty_sold,
            lot.gross_paisa,
            lot.seller_comm_paisa,
            lot.deductions_paisa,
            lot.net_paisa,
            lot.purchase_id,
            lot.arrival_id,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Names the posting of a lot's first save.
///
/// # Errors
/// Any SQLite error.
pub fn set_lot_first_entry(
    conn: &Connection,
    lot_id: i64,
    entry_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bikri_lots SET first_entry_id = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![lot_id, entry_id, now, user_id],
    )?;
    Ok(())
}

/// A commission or لاگا of a sale line as the line stores it (PRD FR-11.6).
#[derive(Debug, Clone, Copy)]
pub struct SaleChargeColumns<'a> {
    /// `percent`, `per_unit`, `per_lot` or `manual`.
    pub method: &'a str,
    /// The percent, basis points.
    pub value_bps: Option<i32>,
    /// The amount per نگ or per lot, paisa.
    pub value_paisa: Option<i64>,
    /// The rule that decided.
    pub rule_id: Option<i64>,
    /// The amount, paisa.
    pub paisa: i64,
}

/// The columns of a new sale line.
#[derive(Debug, Clone)]
pub struct NewSale<'a> {
    /// The بکری.
    pub bikri_id: i64,
    /// The lot.
    pub lot_id: i64,
    /// Position in the lot, from 1.
    pub line_no: i64,
    /// The posting this line belongs to.
    pub journal_entry_id: i64,
    /// The day of the posting, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// The buyer account.
    pub buyer_account_id: i64,
    /// The one-time buyer.
    pub pukhta: Option<PukhtaColumns<'a>>,
    /// Quantity.
    pub qty: i64,
    /// Weight, grams.
    pub weight_grams: Option<i64>,
    /// The seller's rate, paisa.
    pub seller_rate_paisa: i64,
    /// C1 with the seller rate, paisa.
    pub seller_amount_paisa: i64,
    /// The seller's commission.
    pub seller_comm: SaleChargeColumns<'a>,
    /// The buyer's rate, paisa.
    pub buyer_rate_paisa: i64,
    /// C1 with the buyer rate, paisa.
    pub buyer_value_paisa: i64,
    /// The buyer's commission.
    pub buyer_comm: SaleChargeColumns<'a>,
    /// لاگا.
    pub laga: SaleChargeColumns<'a>,
    /// C5, paisa.
    pub buyer_total_paisa: i64,
    /// C19, paisa.
    pub rate_diff_paisa: i64,
    /// `credit` or `cash`.
    pub payment_type: &'a str,
    /// A note.
    pub note: Option<&'a str>,
    /// C21: the cost of the trip goods this line sold, paisa. 0 on a line of any other lot.
    pub cost_paisa: i64,
}

/// Adds a sale line and returns its id.
///
/// # Errors
/// Any SQLite error (the CHECKs of `bikri_sales`).
pub fn insert_sale(
    conn: &Connection,
    sale: &NewSale<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO bikri_sales (created_at, created_by, updated_at, updated_by, bikri_id, lot_id, line_no,
             journal_entry_id, entry_date, buyer_account_id, pukhta_name_ur, pukhta_name_en, pukhta_phone,
             pukhta_person_key, qty, weight_grams, seller_rate_paisa, seller_amount_paisa,
             seller_comm_method, seller_comm_value_bps, seller_comm_value_paisa, seller_comm_rule_id, seller_comm_paisa,
             buyer_rate_paisa, buyer_value_paisa,
             buyer_comm_method, buyer_comm_value_bps, buyer_comm_value_paisa, buyer_comm_rule_id, buyer_comm_paisa,
             laga_method, laga_value_bps, laga_value_paisa, laga_rule_id, laga_paisa,
             buyer_total_paisa, rate_diff_paisa, payment_type, note, cost_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
             ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38)",
        params![
            now,
            user_id,
            sale.bikri_id,
            sale.lot_id,
            sale.line_no,
            sale.journal_entry_id,
            sale.entry_date,
            sale.buyer_account_id,
            sale.pukhta.map(|p| p.name_ur),
            sale.pukhta.map(|p| p.name_en),
            sale.pukhta.map(|p| p.phone),
            sale.pukhta.map(|p| p.person_key),
            sale.qty,
            sale.weight_grams,
            sale.seller_rate_paisa,
            sale.seller_amount_paisa,
            sale.seller_comm.method,
            sale.seller_comm.value_bps,
            sale.seller_comm.value_paisa,
            sale.seller_comm.rule_id,
            sale.seller_comm.paisa,
            sale.buyer_rate_paisa,
            sale.buyer_value_paisa,
            sale.buyer_comm.method,
            sale.buyer_comm.value_bps,
            sale.buyer_comm.value_paisa,
            sale.buyer_comm.rule_id,
            sale.buyer_comm.paisa,
            sale.laga.method,
            sale.laga.value_bps,
            sale.laga.value_paisa,
            sale.laga.rule_id,
            sale.laga.paisa,
            sale.buyer_total_paisa,
            sale.rate_diff_paisa,
            sale.payment_type,
            sale.note,
            sale.cost_paisa,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// What kind of stock move (`stock_moves.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StockKind {
    /// Goods received, a positive quantity.
    Received,
    /// Goods sold, a negative quantity.
    Sold,
}

impl StockKind {
    const fn as_str(self) -> &'static str {
        match self {
            StockKind::Received => "received",
            StockKind::Sold => "sold",
        }
    }
}

/// The columns of a stock move.
#[derive(Debug, Clone, Copy)]
pub struct NewStockMove<'a> {
    /// The lot.
    pub lot_id: i64,
    /// The posting.
    pub entry_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub move_date: &'a str,
    /// Received or sold.
    pub kind: StockKind,
    /// Signed quantity: received is plus, sold is minus.
    pub qty: i64,
    /// The purchase the goods came out of, for a sale of an own lot (PRD FR-12.26). The lot's own
    /// "received" move has none.
    pub purchase_id: Option<i64>,
    /// The garden arrival the goods came out of, for a sale of a lot linked to one (PRD FR-12.40).
    pub arrival_id: Option<i64>,
}

/// Adds a stock move (append-only).
///
/// # Errors
/// Any SQLite error.
pub fn insert_stock_move(
    conn: &Connection,
    stock_move: &NewStockMove<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO stock_moves (created_at, created_by, updated_at, updated_by, lot_id, journal_entry_id, move_date,
             kind, qty, purchase_id, arrival_id)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            now,
            user_id,
            stock_move.lot_id,
            stock_move.entry_id,
            stock_move.move_date,
            stock_move.kind.as_str(),
            stock_move.qty,
            stock_move.purchase_id,
            stock_move.arrival_id
        ],
    )?;
    Ok(())
}

/// The columns of an analytics fact of one sale line (Architecture 15.6). Every value is the
/// STORED value of the line (PRD FR-18.6).
#[derive(Debug, Clone)]
pub struct NewFact<'a> {
    /// The posting.
    pub journal_entry_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// The year of the day.
    pub year: i64,
    /// The month of the day, 1 to 12.
    pub month: i64,
    /// The بکری.
    pub bikri_id: i64,
    /// The lot.
    pub lot_id: i64,
    /// The sale line.
    pub sale_id: i64,
    /// The item.
    pub item_id: i64,
    /// The seller account.
    pub seller_account_id: i64,
    /// The buyer account.
    pub buyer_account_id: i64,
    /// The one-time buyer's key.
    pub pukhta_person_key: Option<&'a str>,
    /// The buyer's city.
    pub city_id: Option<i64>,
    /// The truck stand.
    pub adda_id: Option<i64>,
    /// Quantity.
    pub qty: i64,
    /// Seller amount, paisa.
    pub seller_amount_paisa: i64,
    /// Seller commission, paisa.
    pub seller_comm_paisa: i64,
    /// Buyer value, paisa.
    pub buyer_value_paisa: i64,
    /// Buyer commission, paisa.
    pub buyer_comm_paisa: i64,
    /// لاگا, paisa.
    pub laga_paisa: i64,
    /// +1, or -1 on a reversal row.
    pub sign: i64,
    /// The purchase of an own lot line (PRD FR-12.26), else `None`.
    pub purchase_id: Option<i64>,
    /// The trip whose goods this line sold (for the analytics filters), else `None`.
    pub venture_id: Option<i64>,
    /// C21: the cost of the trip goods this line sold, paisa. 0 for any other line.
    pub cost_paisa: i64,
}

/// Adds an analytics fact (append-only).
///
/// # Errors
/// Any SQLite error.
pub fn insert_fact(
    conn: &Connection,
    fact: &NewFact<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO analytics_facts (created_at, created_by, updated_at, updated_by, journal_entry_id, entry_date,
             year, month, bikri_id, lot_id, sale_id, item_id, seller_account_id, buyer_account_id,
             pukhta_person_key, city_id, adda_id, qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa,
             buyer_comm_paisa, laga_paisa, sign, purchase_id, venture_id, cost_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
             ?21, ?22, ?23, ?24, ?25)",
        params![
            now,
            user_id,
            fact.journal_entry_id,
            fact.entry_date,
            fact.year,
            fact.month,
            fact.bikri_id,
            fact.lot_id,
            fact.sale_id,
            fact.item_id,
            fact.seller_account_id,
            fact.buyer_account_id,
            fact.pukhta_person_key,
            fact.city_id,
            fact.adda_id,
            fact.qty,
            fact.seller_amount_paisa,
            fact.seller_comm_paisa,
            fact.buyer_value_paisa,
            fact.buyer_comm_paisa,
            fact.laga_paisa,
            fact.sign,
            fact.purchase_id,
            fact.venture_id,
            fact.cost_paisa,
        ],
    )?;
    Ok(())
}

/// One trip partner's part of the seller credit of a trip lot (PRD FR-12.26, C25).
#[derive(Debug, Clone, Copy)]
pub struct NewCreditPart {
    /// The lot.
    pub lot_id: i64,
    /// The posting that credits it.
    pub journal_entry_id: i64,
    /// The partner.
    pub partner_id: i64,
    /// His share of the trip, in bps.
    pub share_bps: i64,
    /// His part, signed like the credit: negative when the lot lost money.
    pub amount_paisa: i64,
}

/// Writes one trip partner's part of the seller credit of a trip lot, in the posting that credits
/// it. The journal lines carry no lot, so the part is kept here too.
///
/// # Errors
/// Any SQLite error (the pair of the posting, the lot and the partner is unique).
pub fn insert_credit_part(
    conn: &Connection,
    part: &NewCreditPart,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO lot_credit_parts (created_at, created_by, updated_at, updated_by, lot_id,
             journal_entry_id, partner_id, share_bps, amount_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            now,
            user_id,
            part.lot_id,
            part.journal_entry_id,
            part.partner_id,
            part.share_bps,
            part.amount_paisa
        ],
    )?;
    Ok(())
}

/// The city of each of these accounts (an account with no city is not in the answer), for the
/// analytics facts.
///
/// # Errors
/// Any SQLite error.
pub fn account_cities(
    conn: &Connection,
    ids: &[i64],
) -> rusqlite::Result<std::collections::HashMap<i64, i64>> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT id, city_id FROM accounts WHERE city_id IS NOT NULL AND id IN ({marks})"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(ids), |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?;
    rows.collect()
}
