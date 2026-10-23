//! Queries for `vouchers` and for the day state (Architecture 15.3, 15.7, plan 10). SQL only:
//! the checks are in `mandi_core::vouchers` and `mandi_services::vouchers`. A voucher is never
//! deleted: a cancelled voucher keeps its number and its row (Code Standards D3).

use mandi_core::accounts::RecordRef;
use mandi_core::money::Paisa;
use mandi_core::vouchers::{LastAdvance, VoucherMode, VoucherRow, VoucherType};
use rusqlite::{Connection, OptionalExtension, params};

/// The place of an entry in its account, oldest first: not an opening, day, order key, entry id.
type EntryOrder = (bool, String, i64, i64);
/// The same with the line number last.
type LineOrder = (bool, String, i64, i64, i64);

/// The columns only the vouchers of plan 17 use (PRD 16.2, 16.3, 16.5): the partner, the expense
/// head, the return-by day, who took the cash, and the salary mode. All empty for a وصولی, an
/// ادائیگی and a پیشگی.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VoucherExtra {
    /// The partner of a ذاتی خرچہ and its واپسی (`partners.id`).
    pub partner_id: Option<i64>,
    /// The expense head of a دکان خرچہ, or the salary head of a salary deduction
    /// (`expense_heads.id`).
    pub expense_head_id: Option<i64>,
    /// The return-by day, `YYYY-MM-DD`, of money taken or lent.
    pub return_by_date: Option<String>,
    /// The person who took the cash, if not the partner himself.
    pub taken_by: Option<String>,
    /// True for a salary deduction: the stored mode is `salary` and no cash moves.
    pub salary: bool,
}

impl VoucherExtra {
    /// The stored mode: `salary` for a salary deduction, else the mode as given.
    fn mode_text(&self, mode: VoucherMode) -> &'static str {
        if self.salary { "salary" } else { mode.as_str() }
    }
}

/// The extra columns of a stored voucher.
///
/// # Errors
/// Any SQLite error.
pub fn extras(conn: &Connection, voucher_id: i64) -> rusqlite::Result<VoucherExtra> {
    conn.query_row(
        "SELECT partner_id, expense_head_id, return_by_date, taken_by, mode = 'salary'
         FROM vouchers WHERE id = ?1",
        [voucher_id],
        |r| {
            Ok(VoucherExtra {
                partner_id: r.get(0)?,
                expense_head_id: r.get(1)?,
                return_by_date: r.get(2)?,
                taken_by: r.get(3)?,
                salary: r.get(4)?,
            })
        },
    )
}

/// The columns of a new voucher that the user chooses or the framework gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewVoucher {
    /// The kind.
    pub voucher_type: VoucherType,
    /// The number in its series.
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The account.
    pub account_id: Option<i64>,
    /// The amount in paisa.
    pub amount_paisa: i64,
    /// How the money moves.
    pub mode: VoucherMode,
    /// The bank account row (`bank_accounts.id`), for bank and cheque.
    pub bank_account_id: Option<i64>,
    /// The cheque number, for a cheque.
    pub cheque_no: Option<String>,
    /// The cheque date, `YYYY-MM-DD`, for a cheque.
    pub cheque_date: Option<String>,
    /// The expected goods of an advance.
    pub expected_goods: Option<String>,
    /// The note.
    pub narration: Option<String>,
    /// The one-time person, for a voucher on a پختہ کھاتہ account (PRD FR-13.5).
    pub person_key: Option<String>,
    /// The collector of a وصولی, a staff member.
    pub collector_staff_id: Option<i64>,
    /// Why, for a payment to a buyer (PRD FR-15.11).
    pub reason: Option<String>,
    /// The columns of the vouchers of plan 17 (empty for the three of plan 15).
    pub extra: VoucherExtra,
}

/// The next number of a series: the biggest number ever given plus one. Cancelled vouchers keep
/// their rows, so a number is never used again.
///
/// # Errors
/// Any SQLite error.
pub fn next_number(conn: &Connection, voucher_type: VoucherType) -> rusqlite::Result<i64> {
    let last: i64 = conn.query_row(
        "SELECT COALESCE(MAX(number), 0) FROM vouchers WHERE type = ?1",
        [voucher_type.as_str()],
        |r| r.get(0),
    )?;
    Ok(last.saturating_add(1))
}

/// Adds a voucher and returns its id.
///
/// # Errors
/// Any SQLite error (a CHECK that the checks above should have caught).
pub fn insert(
    conn: &Connection,
    voucher: &NewVoucher,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO vouchers (created_at, created_by, updated_at, updated_by, type, number, entry_date,
             account_id, amount_paisa, mode, bank_account_id, cheque_no, cheque_date, expected_goods, narration,
             pukhta_person_key, collector_staff_id, reason, partner_id, expense_head_id, return_by_date,
             taken_by)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                 ?17, ?18, ?19, ?20)",
        params![
            now,
            user_id,
            voucher.voucher_type.as_str(),
            voucher.number,
            voucher.entry_date,
            voucher.account_id,
            voucher.amount_paisa,
            voucher.extra.mode_text(voucher.mode),
            voucher.bank_account_id,
            voucher.cheque_no,
            voucher.cheque_date,
            voucher.expected_goods,
            voucher.narration,
            voucher.person_key,
            voucher.collector_staff_id,
            voucher.reason,
            voucher.extra.partner_id,
            voucher.extra.expense_head_id,
            voucher.extra.return_by_date,
            voucher.extra.taken_by,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Names the journal entry that holds the voucher's posting.
///
/// # Errors
/// Any SQLite error.
pub fn set_current_entry(
    conn: &Connection,
    voucher_id: i64,
    entry_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE vouchers SET current_entry_id = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![voucher_id, entry_id, now, user_id],
    )?;
    Ok(())
}

/// A bank account row as a voucher needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BankFacts {
    /// The ledger account that holds its money (under control 0102).
    pub account_id: i64,
    /// False when hidden from pickers.
    pub is_active: bool,
}

/// A bank account (`bank_accounts.id`), if there is one.
///
/// # Errors
/// Any SQLite error.
pub fn bank_account(conn: &Connection, id: i64) -> rusqlite::Result<Option<BankFacts>> {
    conn.query_row(
        "SELECT account_id, is_active FROM bank_accounts WHERE id = ?1",
        [id],
        |r| {
            Ok(BankFacts {
                account_id: r.get(0)?,
                is_active: r.get(1)?,
            })
        },
    )
    .optional()
}

fn bad(column: usize, what: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        rusqlite::types::Type::Text,
        format!("vouchers: {what}").into(),
    )
}

/// The columns of a voucher row in the order [`read_row`] reads them.
const ROW_COLUMNS: &str =
    "v.id, v.type, v.number, v.entry_date, v.account_id, a.code, a.name_ur, a.name_en,
                v.amount_paisa, v.mode, v.bank_account_id, v.cheque_no, v.cheque_date,
                v.expected_goods, v.narration, v.status, v.version, v.pukhta_person_key,
                v.collector_staff_id, v.reason, v.cancel_reason, v.current_entry_id";

/// A voucher row and the person key it names. The person's names are filled in by
/// [`fill_people`], because they come from the بکری lines, not from the voucher.
fn read_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<(VoucherRow, Option<String>)> {
    let account_id: Option<i64> = r.get(4)?;
    let person_key: Option<String> = r.get(17)?;
    let row = VoucherRow {
        id: r.get(0)?,
        voucher_type: VoucherType::parse(&r.get::<_, String>(1)?).ok_or_else(|| bad(1, "type"))?,
        number: r.get(2)?,
        entry_date: r.get(3)?,
        account: account_id.map(|id| RecordRef {
            id,
            code: r
                .get::<_, Option<String>>(5)
                .ok()
                .flatten()
                .unwrap_or_default(),
            name_ur: r
                .get::<_, Option<String>>(6)
                .ok()
                .flatten()
                .unwrap_or_default(),
            name_en: r
                .get::<_, Option<String>>(7)
                .ok()
                .flatten()
                .unwrap_or_default(),
        }),
        amount_paisa: Paisa::try_from_paisa(r.get(8)?).map_err(|_| bad(8, "amount"))?,
        // A salary deduction (plan 17) is stored as `salary`; as a row it reads as cash because it
        // moves no money outside the books. `extras` tells it apart.
        mode: match r.get::<_, String>(9)?.as_str() {
            "salary" => VoucherMode::Cash,
            other => VoucherMode::parse(other).ok_or_else(|| bad(9, "mode"))?,
        },
        bank_account_id: r.get(10)?,
        cheque_no: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
        cheque_date: r.get(12)?,
        expected_goods: r.get::<_, Option<String>>(13)?.unwrap_or_default(),
        narration: r.get::<_, Option<String>>(14)?.unwrap_or_default(),
        status: r.get(15)?,
        version: r.get(16)?,
        person: None,
        collector_staff_id: r.get(18)?,
        reason: r.get::<_, Option<String>>(19)?.unwrap_or_default(),
        cancel_reason: r.get::<_, Option<String>>(20)?.unwrap_or_default(),
        entry_id: r.get(21)?,
        // The service decides this from the day state (the repository does not know it).
        editable: false,
    };
    Ok((row, person_key))
}

/// The names and phone of a one-time person as he was last typed, from his latest بکری line
/// (a buyer line or a seller lot). `None` if no line carries the key.
///
/// # Errors
/// Any SQLite error.
pub fn pukhta_person(
    conn: &Connection,
    person_key: &str,
) -> rusqlite::Result<Option<mandi_core::bikri::SavedPukhta>> {
    conn.query_row(
        "SELECT name_ur, name_en, phone FROM (
             SELECT s.pukhta_name_ur AS name_ur, s.pukhta_name_en AS name_en, s.pukhta_phone AS phone,
                    s.entry_date AS entry_date, s.id AS row_id
             FROM bikri_sales s WHERE s.pukhta_person_key = ?1
             UNION ALL
             SELECT l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone, e.entry_date, l.id
             FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
             WHERE l.pukhta_person_key = ?1)
         ORDER BY entry_date DESC, row_id DESC LIMIT 1",
        [person_key],
        |r| {
            Ok(mandi_core::bikri::SavedPukhta {
                name_ur: r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                name_en: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                phone: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                person_key: person_key.to_owned(),
            })
        },
    )
    .optional()
}

/// Puts the one-time person's names on every row that names one.
fn fill_people(
    conn: &Connection,
    rows: Vec<(VoucherRow, Option<String>)>,
) -> rusqlite::Result<Vec<VoucherRow>> {
    let mut known: std::collections::HashMap<String, Option<mandi_core::bikri::SavedPukhta>> =
        std::collections::HashMap::new();
    let mut out = Vec::with_capacity(rows.len());
    for (mut row, key) in rows {
        if let Some(key) = key {
            if !known.contains_key(&key) {
                known.insert(key.clone(), pukhta_person(conn, &key)?);
            }
            row.person = known.get(&key).cloned().flatten().or_else(|| {
                // A person whose lines are gone still keeps his key on the voucher.
                Some(mandi_core::bikri::SavedPukhta {
                    name_ur: String::new(),
                    name_en: String::new(),
                    phone: String::new(),
                    person_key: key.clone(),
                })
            });
        }
        out.push(row);
    }
    Ok(out)
}

/// A filter of [`list`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ListFilter<'a> {
    /// Only this kind.
    pub voucher_type: Option<VoucherType>,
    /// First day, `YYYY-MM-DD`, included.
    pub from: Option<&'a str>,
    /// Last day, `YYYY-MM-DD`, included.
    pub to: Option<&'a str>,
    /// Only this account.
    pub account_id: Option<i64>,
    /// Only this one-time person.
    pub person_key: Option<&'a str>,
}

/// The vouchers matching a filter, by day and number.
///
/// # Errors
/// Any SQLite error, or a row whose text is not one of the known words.
pub fn list(conn: &Connection, filter: &ListFilter<'_>) -> rusqlite::Result<Vec<VoucherRow>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {ROW_COLUMNS}
         FROM vouchers v LEFT JOIN accounts a ON a.id = v.account_id
         WHERE (?1 IS NULL OR v.type = ?1) AND (?2 IS NULL OR v.entry_date >= ?2)
           AND (?3 IS NULL OR v.entry_date <= ?3) AND (?4 IS NULL OR v.account_id = ?4)
           AND (?5 IS NULL OR v.pukhta_person_key = ?5)
         ORDER BY v.entry_date, v.number, v.id"
    ))?;
    let rows = statement
        .query_map(
            params![
                filter.voucher_type.map(VoucherType::as_str),
                filter.from,
                filter.to,
                filter.account_id,
                filter.person_key
            ],
            read_row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    fill_people(conn, rows)
}

/// One voucher by id.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<VoucherRow>> {
    let found = conn
        .query_row(
            &format!(
                "SELECT {ROW_COLUMNS}
                 FROM vouchers v LEFT JOIN accounts a ON a.id = v.account_id
                 WHERE v.id = ?1"
            ),
            [id],
            read_row,
        )
        .optional()?;
    Ok(fill_people(conn, found.into_iter().collect())?
        .into_iter()
        .next())
}

/// The last پیشگی voucher of a seller that is still in force.
///
/// # Errors
/// Any SQLite error.
pub fn last_advance(conn: &Connection, seller_id: i64) -> rusqlite::Result<Option<LastAdvance>> {
    conn.query_row(
        "SELECT entry_date, amount_paisa, number FROM vouchers
         WHERE type = 'advance' AND account_id = ?1 AND status = 'saved'
         ORDER BY entry_date DESC, number DESC LIMIT 1",
        [seller_id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        },
    )
    .optional()?
    .map(|(entry_date, paisa, voucher_number)| {
        Ok(LastAdvance {
            entry_date,
            amount_paisa: Paisa::try_from_paisa(paisa).map_err(|_| bad(1, "amount"))?,
            voucher_number,
        })
    })
    .transpose()
}

/// One advance line of a seller and what has been adjusted against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceLine {
    /// `voucher` or `opening` (or the other source types, as stored).
    pub source_type: String,
    /// The voucher number, for source `voucher`.
    pub voucher_number: Option<i64>,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What was given, paisa.
    pub given_paisa: i64,
    /// What has been adjusted against it, paisa.
    pub adjusted_paisa: i64,
}

/// The seller's advance lines: posted debit lines flagged as an advance, without those of an
/// entry that has been reversed, oldest first (PRD FR-15.8). Adjusted counts only active
/// matching rows whose credit line is still in force, the same rule as the card's open
/// advance.
///
/// # Errors
/// Any SQLite error.
pub fn advance_lines(conn: &Connection, seller_id: i64) -> rusqlite::Result<Vec<AdvanceLine>> {
    let mut statement = conn.prepare(
        "SELECT e.source_type, v.number, e.entry_date, l.dr_paisa,
                COALESCE((SELECT SUM(a.amount_paisa)
                          FROM advance_allocations a JOIN journal_lines cl ON cl.id = a.credit_line_id
                          WHERE a.advance_line_id = l.id AND a.is_active = 1
                            AND NOT EXISTS (SELECT 1 FROM journal_entries r
                                            WHERE r.reverses_entry_id = cl.entry_id AND r.status = 'posted')), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         WHERE l.account_id = ?1 AND l.is_advance = 1 AND e.status = 'posted'
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
         ORDER BY e.entry_date, COALESCE(e.order_key, e.id), e.id, l.line_no",
    )?;
    let rows = statement.query_map([seller_id], |r| {
        Ok(AdvanceLine {
            source_type: r.get(0)?,
            voucher_number: r.get(1)?,
            entry_date: r.get(2)?,
            given_paisa: r.get(3)?,
            adjusted_paisa: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// The values an edit writes back to a voucher row. The type and the number never change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditedVoucher {
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The account.
    pub account_id: Option<i64>,
    /// The one-time person.
    pub person_key: Option<String>,
    /// The amount in paisa.
    pub amount_paisa: i64,
    /// How the money moves.
    pub mode: VoucherMode,
    /// The bank account row.
    pub bank_account_id: Option<i64>,
    /// The cheque number.
    pub cheque_no: Option<String>,
    /// The cheque date, `YYYY-MM-DD`.
    pub cheque_date: Option<String>,
    /// The collector.
    pub collector_staff_id: Option<i64>,
    /// Why, for a payment to a buyer.
    pub reason: Option<String>,
    /// The expected goods of an advance.
    pub expected_goods: Option<String>,
    /// The note.
    pub narration: Option<String>,
    /// The columns of the vouchers of plan 17 (empty for the three of plan 15).
    pub extra: VoucherExtra,
    /// The live posting after the edit.
    pub current_entry_id: i64,
}

/// Writes an edit into the voucher row and raises its version by one. Returns false if the row
/// is not `saved` (a cancelled voucher is never edited).
///
/// # Errors
/// Any SQLite error (the table checks refuse a wrong combination).
pub fn update_for_edit(
    conn: &Connection,
    id: i64,
    edited: &EditedVoucher,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE vouchers SET entry_date = ?2, account_id = ?3, pukhta_person_key = ?4, amount_paisa = ?5,
             mode = ?6, bank_account_id = ?7, cheque_no = ?8, cheque_date = ?9, collector_staff_id = ?10,
             reason = ?11, expected_goods = ?12, narration = ?13, current_entry_id = ?14,
             updated_at = ?15, updated_by = ?16, partner_id = ?17, expense_head_id = ?18,
             return_by_date = ?19, taken_by = ?20, version = version + 1
         WHERE id = ?1 AND status = 'saved'",
        params![
            id,
            edited.entry_date,
            edited.account_id,
            edited.person_key,
            edited.amount_paisa,
            edited.extra.mode_text(edited.mode),
            edited.bank_account_id,
            edited.cheque_no,
            edited.cheque_date,
            edited.collector_staff_id,
            edited.reason,
            edited.expected_goods,
            edited.narration,
            edited.current_entry_id,
            now,
            user_id,
            edited.extra.partner_id,
            edited.extra.expense_head_id,
            edited.extra.return_by_date,
            edited.extra.taken_by,
        ],
    )?;
    Ok(changed > 0)
}

/// Marks a saved voucher cancelled, with the reason, and raises its version. The row, its number
/// and its last posting stay (Code Standards D3). Returns false if it was not `saved`.
///
/// # Errors
/// Any SQLite error.
pub fn mark_cancelled(
    conn: &Connection,
    id: i64,
    reason: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE vouchers SET status = 'cancelled', cancel_reason = ?2, updated_at = ?3, updated_by = ?4,
             version = version + 1
         WHERE id = ?1 AND status = 'saved'",
        params![id, reason, now, user_id],
    )?;
    Ok(changed > 0)
}

/// Whether a staff member exists, and if he is active. `None` if there is no such row.
///
/// # Errors
/// Any SQLite error.
pub fn staff_active(conn: &Connection, id: i64) -> rusqlite::Result<Option<bool>> {
    conn.query_row("SELECT is_active FROM staff WHERE id = ?1", [id], |r| {
        r.get(0)
    })
    .optional()
}

/// The vouchers with this number: one for each series, or only the series asked for. For the
/// global search (PRD FR-5.4).
///
/// # Errors
/// Any SQLite error, or a row whose text is not one of the known words.
pub fn hits_by_number(
    conn: &Connection,
    number: i64,
    voucher_type: Option<VoucherType>,
) -> rusqlite::Result<Vec<mandi_core::vouchers::VoucherHit>> {
    let mut statement = conn.prepare(
        "SELECT v.id, v.type, v.number, v.entry_date, v.account_id, a.code, a.name_ur, a.name_en,
                v.amount_paisa, v.status
         FROM vouchers v LEFT JOIN accounts a ON a.id = v.account_id
         WHERE v.number = ?1 AND (?2 IS NULL OR v.type = ?2)
         ORDER BY CASE v.type WHEN 'receipt' THEN 0 WHEN 'payment' THEN 1 WHEN 'advance' THEN 2
                  ELSE 3 END, v.type",
    )?;
    let rows = statement.query_map(
        params![number, voucher_type.map(VoucherType::as_str)],
        |r| {
            let account_id: Option<i64> = r.get(4)?;
            Ok(mandi_core::vouchers::VoucherHit {
                id: r.get(0)?,
                voucher_type: VoucherType::parse(&r.get::<_, String>(1)?)
                    .ok_or_else(|| bad(1, "type"))?,
                number: r.get(2)?,
                entry_date: r.get(3)?,
                account: account_id.map(|id| RecordRef {
                    id,
                    code: r
                        .get::<_, Option<String>>(5)
                        .ok()
                        .flatten()
                        .unwrap_or_default(),
                    name_ur: r
                        .get::<_, Option<String>>(6)
                        .ok()
                        .flatten()
                        .unwrap_or_default(),
                    name_en: r
                        .get::<_, Option<String>>(7)
                        .ok()
                        .flatten()
                        .unwrap_or_default(),
                }),
                amount_paisa: Paisa::try_from_paisa(r.get(8)?).map_err(|_| bad(8, "amount"))?,
                status: r.get(9)?,
            })
        },
    )?;
    rows.collect()
}

/// The bank, title and number of a bank account row, as typed (PRD FR-7.25).
///
/// # Errors
/// Any SQLite error.
pub fn bank_account_label(
    conn: &Connection,
    id: i64,
) -> rusqlite::Result<Option<(String, String, String)>> {
    conn.query_row(
        "SELECT bank_name, title, number FROM bank_accounts WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .optional()
}

/// The Urdu and English names of a staff member.
///
/// # Errors
/// Any SQLite error.
pub fn staff_names(conn: &Connection, id: i64) -> rusqlite::Result<Option<(String, String)>> {
    conn.query_row(
        "SELECT name_ur, name_en FROM staff WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

/// True if some posted line of this key exists, so the person is known on the account (PRD
/// FR-13.5).
///
/// # Errors
/// Any SQLite error.
pub fn person_has_lines(
    conn: &Connection,
    account_id: i64,
    person_key: &str,
) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                        WHERE l.account_id = ?1 AND l.pukhta_person_key = ?2 AND e.status = 'posted')",
        params![account_id, person_key],
        |r| r.get(0),
    )
}

/// An open bill of a buyer key with where it came from (PRD FR-7.58).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillRow {
    /// The journal line.
    pub line_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The posting type of its entry.
    pub posting_type: String,
    /// The kind of record of its entry (`bikri`, `voucher`, `account`, ...).
    pub source_type: String,
    /// The بکری or voucher number, where there is one.
    pub number: Option<i64>,
    /// The whole bill, paisa.
    pub amount_paisa: i64,
    /// The sum of its active matching rows, paisa.
    pub matched_paisa: i64,
}

/// The live debit lines of a buyer key that are not matched in full, oldest first, with their
/// source. Same lines and order as `matching::open_bills`.
///
/// # Errors
/// Any SQLite error.
pub fn open_bill_rows(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Vec<BillRow>> {
    let matched = "COALESCE((SELECT SUM(a.amount_paisa) FROM bill_allocations a
                             WHERE a.bill_line_id = l.id AND a.is_active = 1), 0)";
    let person = if person_key.is_some() {
        "AND l.pukhta_person_key = ?2"
    } else {
        "AND ?2 IS NULL"
    };
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, e.entry_date, e.posting_type, e.source_type,
                CASE e.source_type
                    WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = e.source_id)
                    WHEN 'voucher' THEN (SELECT v.number FROM vouchers v WHERE v.id = e.source_id)
                END,
                l.dr_paisa, {matched}
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.dr_paisa > 0 {person} AND {live}
           AND l.dr_paisa > {matched}
         ORDER BY {order}",
        live = crate::matching::LIVE,
        order = crate::matching::ORDER,
    ))?;
    let rows = statement.query_map(params![account_id, person_key], |r| {
        Ok(BillRow {
            line_id: r.get(0)?,
            entry_date: r.get(1)?,
            posting_type: r.get(2)?,
            source_type: r.get(3)?,
            number: r.get(4)?,
            amount_paisa: r.get(5)?,
            matched_paisa: r.get(6)?,
        })
    })?;
    rows.collect()
}

/// The bill lines with these ids (open or not), oldest first, with their source. The matched
/// amount is not read: the caller works out what is open.
///
/// # Errors
/// Any SQLite error.
pub fn bill_rows_by_ids(conn: &Connection, line_ids: &[i64]) -> rusqlite::Result<Vec<BillRow>> {
    if line_ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; line_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, e.entry_date, e.posting_type, e.source_type,
                CASE e.source_type
                    WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = e.source_id)
                    WHEN 'voucher' THEN (SELECT v.number FROM vouchers v WHERE v.id = e.source_id)
                END,
                l.dr_paisa
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.id IN ({marks}) AND l.dr_paisa > 0
         ORDER BY {order}",
        order = crate::matching::ORDER,
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(line_ids), |r| {
        Ok(BillRow {
            line_id: r.get(0)?,
            entry_date: r.get(1)?,
            posting_type: r.get(2)?,
            source_type: r.get(3)?,
            number: r.get(4)?,
            amount_paisa: r.get(5)?,
            matched_paisa: 0,
        })
    })?;
    rows.collect()
}

/// The voucher number of each of these entries, where the entry belongs to a voucher.
///
/// # Errors
/// Any SQLite error.
pub fn voucher_numbers_of_entries(
    conn: &Connection,
    entry_ids: &[i64],
) -> rusqlite::Result<std::collections::HashMap<i64, i64>> {
    if entry_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let marks = vec!["?"; entry_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT e.id, v.number FROM journal_entries e JOIN vouchers v ON v.id = e.source_id
         WHERE e.source_type = 'voucher' AND e.id IN ({marks})"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(entry_ids), |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?;
    rows.collect()
}

// ---- What is payable to a seller: totals and the newest credits ----

/// The totals of a seller key's live lines that the payable working needs (money agent 8.2a).
/// Sums only, no line is read one by one for an ordinary seller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayableTotals {
    /// Σ of his credit lines.
    pub credit_paisa: i64,
    /// Σ of his debit lines that are not advances (payments, نقد, جنرل debits).
    pub payments_paisa: i64,
    /// Σ of his advance lines.
    pub advances_given_paisa: i64,
    /// What active advance rows take from his credit lines.
    pub adjusted_on_credits_paisa: i64,
    /// What active advance rows take from his advance lines.
    pub adjusted_on_advances_paisa: i64,
}

/// Entries that are not live: drafts, reversal entries and the entries a posted reversal reverses.
/// The exact complement of `matching::LIVE`, written as a small set so that sums over the many
/// lines of an account can use the index on the amounts and take these few lines off.
const NOT_LIVE_ENTRIES: &str = "nonlive(id) AS (
    SELECT id FROM journal_entries WHERE reverses_entry_id IS NOT NULL
    UNION SELECT reverses_entry_id FROM journal_entries
          WHERE reverses_entry_id IS NOT NULL AND status = 'posted'
    UNION SELECT id FROM journal_entries WHERE status <> 'posted')";

/// (credit, payments, advances given) over the live lines of one key, line by line with the
/// entries. Only for a key with few lines.
fn live_sums_of_key(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<(i64, i64, i64)> {
    conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(CASE WHEN l.cr_paisa > 0 THEN l.cr_paisa END), 0),
                    COALESCE(SUM(CASE WHEN l.dr_paisa > 0 AND l.is_advance = 0 THEN l.dr_paisa END), 0),
                    COALESCE(SUM(CASE WHEN l.is_advance = 1 THEN l.dr_paisa END), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id = ?1 AND l.pukhta_person_key IS ?2 AND {live}",
            live = crate::matching::LIVE
        ),
        params![account_id, person_key],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
}

/// The same three sums for the lines of an account that carry no person key, without reading its
/// lines one by one: the sums over every line (from the covering index of the amounts and the
/// partial index of the advances), less the sums over the lines that carry a key or sit in an
/// entry that is not live. Gives exactly what [`live_sums_of_key`] gives for `None`.
fn live_sums_without_key(conn: &Connection, account_id: i64) -> rusqlite::Result<(i64, i64, i64)> {
    let (all_credit, all_debit): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(cr_paisa), 0), COALESCE(SUM(dr_paisa), 0)
         FROM journal_lines WHERE account_id = ?1",
        [account_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let all_given: i64 = conn.query_row(
        "SELECT COALESCE(SUM(dr_paisa), 0) FROM journal_lines
         WHERE account_id = ?1 AND is_advance = 1",
        [account_id],
        |r| r.get(0),
    )?;
    let (out_credit, out_payments, out_given): (i64, i64, i64) = conn.query_row(
        &format!(
            "WITH {not_live}
             SELECT COALESCE(SUM(CASE WHEN cr_paisa > 0 THEN cr_paisa END), 0),
                    COALESCE(SUM(CASE WHEN dr_paisa > 0 AND is_advance = 0 THEN dr_paisa END), 0),
                    COALESCE(SUM(CASE WHEN is_advance = 1 THEN dr_paisa END), 0)
             FROM (SELECT id, cr_paisa, dr_paisa, is_advance FROM journal_lines
                   WHERE account_id = ?1 AND pukhta_person_key IS NOT NULL
                   UNION
                   SELECT id, cr_paisa, dr_paisa, is_advance FROM journal_lines
                   WHERE account_id = ?1 AND entry_id IN (SELECT id FROM nonlive))",
            not_live = NOT_LIVE_ENTRIES
        ),
        [account_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    // Sums of whole-rupee amounts: a difference that does not fit means broken data.
    let take = crate::matching::difference;
    Ok((
        take(all_credit, out_credit)?,
        take(take(all_debit, all_given)?, out_payments)?,
        take(all_given, out_given)?,
    ))
}

/// The totals of a seller key (an ordinary seller, or one person of the پختہ کھاتہ seller
/// account). Sums are `SUM()` on `INTEGER` columns; the working in Rust uses checked maths.
///
/// # Errors
/// Any SQLite error.
pub fn payable_totals(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<PayableTotals> {
    let (credit, payments, given) = match person_key {
        // One person of the پختہ کھاتہ account: his few lines are read with their entries.
        Some(_) => live_sums_of_key(conn, account_id, person_key)?,
        // An ordinary seller: his lines are many (tens of thousands for the busiest), so the sums
        // come from the index and the few lines that are not live are taken off.
        None => live_sums_without_key(conn, account_id)?,
    };
    let adjusted = |end: &str| -> rusqlite::Result<i64> {
        conn.query_row(
            &format!(
                "SELECT COALESCE(SUM(a.amount_paisa), 0)
                 FROM advance_allocations a
                 JOIN journal_lines l ON l.id = a.{end}
                 JOIN journal_entries e ON e.id = l.entry_id
                 WHERE a.account_id = ?1 AND a.is_active = 1 AND l.pukhta_person_key IS ?2 AND {live}",
                live = crate::matching::LIVE
            ),
            params![account_id, person_key],
            |r| r.get(0),
        )
    };
    Ok(PayableTotals {
        credit_paisa: credit,
        payments_paisa: payments,
        advances_given_paisa: given,
        adjusted_on_credits_paisa: adjusted("credit_line_id")?,
        adjusted_on_advances_paisa: adjusted("advance_line_id")?,
    })
}

/// A credit line of a seller key with where it came from and what advance rows take from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditRow {
    /// The journal line.
    pub line_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The posting type of its entry.
    pub posting_type: String,
    /// The kind of record of its entry.
    pub source_type: String,
    /// The بکری or voucher number, where there is one.
    pub number: Option<i64>,
    /// The whole credit, paisa.
    pub credit_paisa: i64,
    /// What active advance rows take from it, paisa.
    pub adjusted_paisa: i64,
}

/// A credit line of a seller key, as the payable walk needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayableCredit {
    /// The journal line.
    pub line_id: i64,
    /// The whole credit, paisa.
    pub credit_paisa: i64,
}

/// The live credit lines of a seller key in the order of his payable working, OLDEST first
/// (PRD FR-7.59): his opening credit first whatever its date, then the others by day, entry and
/// line. It is not the order of the account (`matching::ORDER`), which keeps an opening first only
/// within its own day.
///
/// An ordinary seller (no line with a person key) is read from the index of the amounts and the
/// dates of the entries, not line by line: the busiest seller of the volume of NFR-SCL-1 has
/// about 37,000 credit lines. The order of lines of one entry is then read for the few entries
/// that hold more than one credit line of his. The result is exactly the one of the slow reading
/// that a one-time person's key uses. [`credit_row`] reads the details of the few lines a caller
/// keeps.
///
/// # Errors
/// Any SQLite error.
pub fn payable_credits_oldest_first(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Vec<PayableCredit>> {
    let keyed: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM journal_lines
                        WHERE account_id = ?1 AND pukhta_person_key IS NOT NULL)",
        [account_id],
        |r| r.get(0),
    )?;
    if person_key.is_some() || keyed {
        return payable_credits_of_key(conn, account_id, person_key);
    }
    let mut statement = conn.prepare(&format!(
        "WITH {not_live}
         SELECT l.id, l.cr_paisa, e.posting_type <> 'opening', e.entry_date, COALESCE(e.order_key, e.id),
                l.entry_id
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.cr_paisa > 0
           AND l.entry_id NOT IN (SELECT id FROM nonlive)",
        not_live = NOT_LIVE_ENTRIES
    ))?;
    // (not an opening, day, order key, entry), line id, credit.
    let mut rows: Vec<(EntryOrder, i64, i64)> = statement
        .query_map([account_id], |r| {
            Ok((
                (r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?),
                r.get(0)?,
                r.get(1)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    rows.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    // Lines saved in one entry stay in line order: ask the line numbers of those few only.
    let mut at = 0;
    while at < rows.len() {
        let mut end = at + 1;
        while end < rows.len() && rows.get(end).map(|r| &r.0) == rows.get(at).map(|r| &r.0) {
            end += 1;
        }
        if end - at > 1 {
            let mut group: Vec<(i64, (i64, i64))> = Vec::with_capacity(end - at);
            for (_, id, credit) in rows.get(at..end).unwrap_or_default() {
                let line_no: i64 = conn
                    .prepare_cached("SELECT line_no FROM journal_lines WHERE id = ?1")?
                    .query_row([id], |r| r.get(0))?;
                group.push((line_no, (*id, *credit)));
            }
            group.sort_by_key(|(line_no, _)| *line_no);
            for (offset, (_, (id, credit))) in group.into_iter().enumerate() {
                if let Some(row) = rows.get_mut(at + offset) {
                    row.1 = id;
                    row.2 = credit;
                }
            }
        }
        at = end;
    }
    Ok(rows
        .into_iter()
        .map(|(_, line_id, credit_paisa)| PayableCredit {
            line_id,
            credit_paisa,
        })
        .collect())
}

/// [`payable_credits_oldest_first`] for a key whose lines are read with their entries: one person
/// of the پختہ کھاتہ seller account.
fn payable_credits_of_key(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Vec<PayableCredit>> {
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.cr_paisa, e.posting_type <> 'opening', e.entry_date, COALESCE(e.order_key, e.id),
                l.entry_id, l.line_no
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.pukhta_person_key IS ?2 AND l.cr_paisa > 0 AND {live}",
        live = crate::matching::LIVE
    ))?;
    // (not an opening, day, order key, entry, line), credit line, credit.
    let mut keyed: Vec<(LineOrder, PayableCredit)> = statement
        .query_map(params![account_id, person_key], |r| {
            Ok((
                (r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?),
                PayableCredit {
                    line_id: r.get(0)?,
                    credit_paisa: r.get(1)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(keyed.into_iter().map(|(_, credit)| credit).collect())
}

/// What active advance rows take from each credit line of a seller, by credit line id. Only the
/// lines that something is taken from are in the map.
///
/// # Errors
/// Any SQLite error.
pub fn advance_adjusted_by_credit(
    conn: &Connection,
    account_id: i64,
) -> rusqlite::Result<std::collections::HashMap<i64, i64>> {
    let mut statement = conn.prepare(
        "SELECT credit_line_id, SUM(amount_paisa) FROM advance_allocations
         WHERE account_id = ?1 AND is_active = 1 GROUP BY credit_line_id",
    )?;
    statement
        .query_map([account_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect()
}

/// One credit line of a seller with where it came from and what advance rows take from it.
///
/// # Errors
/// Any SQLite error.
pub fn credit_row(conn: &Connection, line_id: i64) -> rusqlite::Result<CreditRow> {
    conn.prepare_cached(
        "SELECT l.id, e.entry_date, e.posting_type, e.source_type,
                CASE e.source_type
                    WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = e.source_id)
                    WHEN 'voucher' THEN (SELECT v.number FROM vouchers v WHERE v.id = e.source_id)
                END,
                l.cr_paisa,
                COALESCE((SELECT SUM(a.amount_paisa) FROM advance_allocations a
                          WHERE a.is_active = 1 AND a.credit_line_id = l.id), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.id = ?1",
    )?
    .query_row([line_id], |r| {
        Ok(CreditRow {
            line_id: r.get(0)?,
            entry_date: r.get(1)?,
            posting_type: r.get(2)?,
            source_type: r.get(3)?,
            number: r.get(4)?,
            credit_paisa: r.get(5)?,
            adjusted_paisa: r.get(6)?,
        })
    })
}

/// The open bills of a buyer key: the oldest ones with their source, how many there are in all,
/// and what is open on all of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenBillRows {
    /// The oldest open bills, oldest first, at most the `limit` that was asked.
    pub rows: Vec<BillRow>,
    /// All the open bills of the key.
    pub total_count: usize,
    /// Σ of what is open on all of them, paisa (not only on `rows`).
    pub open_paisa: i64,
}

/// The open bills of a buyer key, found by walking back from the newest entry (see
/// [`crate::matching::open_bills_upto`]). The matching rows must be canonical, which every writer
/// keeps true; if the walk does not end exactly on the open total it reads the whole key as
/// [`open_bill_rows`] does. Only the oldest `limit` bills are read with their source (the bill
/// numbers cost a read each); the count and the open total are of all of them.
///
/// # Errors
/// Any SQLite error.
pub fn open_bill_rows_fast(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
    limit: usize,
) -> rusqlite::Result<OpenBillRows> {
    let expected = crate::matching::expected_open_bills(conn, account_id, person_key)?;
    if expected == 0 {
        return Ok(OpenBillRows {
            rows: Vec::new(),
            total_count: 0,
            open_paisa: 0,
        });
    }
    let found = crate::matching::open_bills_upto(conn, account_id, person_key, expected)?;
    let mut open_paisa: i64 = 0;
    for line in &found {
        let open = crate::matching::difference(line.amount_paisa, line.matched_paisa)?;
        open_paisa = open_paisa
            .checked_add(open)
            .ok_or(rusqlite::Error::IntegralValueOutOfRange(0, open_paisa))?;
    }
    let kept: Vec<_> = found.iter().take(limit).collect();
    let ids: Vec<i64> = kept.iter().map(|l| l.line_id).collect();
    let mut rows = bill_rows_by_ids(conn, &ids)?;
    for row in &mut rows {
        if let Some(line) = kept.iter().find(|l| l.line_id == row.line_id) {
            row.matched_paisa = line.matched_paisa;
        }
    }
    Ok(OpenBillRows {
        rows,
        total_count: found.len(),
        open_paisa,
    })
}

// ---- The advances report (PRD R26 data) ----

/// What one seller was given and what became of it, as of a day (paisa).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceTotals {
    /// The seller.
    pub account_id: i64,
    /// Σ of his advance lines dated on or before the day.
    pub given_paisa: i64,
    /// Σ of the active rows whose credit is a وصولی from him dated on or before the day.
    pub returned_paisa: i64,
    /// Σ of the active rows whose credit is a بکری posting dated on or before the day.
    pub adjusted_paisa: i64,
    /// The day of his last advance on or before the day, `YYYY-MM-DD`.
    pub last_date: Option<String>,
}

/// The advance totals of every seller who was given an advance by `as_of` (or of one seller).
/// Live lines only (a reversed entry and its reversal are both left out).
///
/// # Errors
/// Any SQLite error.
pub fn advance_totals(
    conn: &Connection,
    as_of: &str,
    seller_id: Option<i64>,
) -> rusqlite::Result<Vec<AdvanceTotals>> {
    let live = |alias: &str| {
        format!(
            "{alias}.status = 'posted' AND {alias}.reverses_entry_id IS NULL
             AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = {alias}.id AND r.status = 'posted')"
        )
    };
    let mut statement = conn.prepare(&format!(
        "SELECT l.account_id, SUM(l.dr_paisa), MAX(e.entry_date)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.is_advance = 1 AND l.dr_paisa > 0 AND e.entry_date <= ?1 AND {live_e}
           AND (?2 IS NULL OR l.account_id = ?2)
         GROUP BY l.account_id ORDER BY l.account_id",
        live_e = live("e")
    ))?;
    let given = statement
        .query_map(params![as_of, seller_id], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut by_kind = conn.prepare(&format!(
        "SELECT a.account_id, ec.posting_type = 'receipt', SUM(a.amount_paisa)
         FROM advance_allocations a
         JOIN journal_lines cl ON cl.id = a.credit_line_id
         JOIN journal_entries ec ON ec.id = cl.entry_id
         JOIN journal_lines al ON al.id = a.advance_line_id
         JOIN journal_entries ea ON ea.id = al.entry_id
         WHERE a.is_active = 1 AND ec.entry_date <= ?1 AND {live_c} AND {live_a}
           AND (?2 IS NULL OR a.account_id = ?2)
         GROUP BY a.account_id, ec.posting_type = 'receipt'",
        live_c = live("ec"),
        live_a = live("ea")
    ))?;
    let used = by_kind
        .query_map(params![as_of, seller_id], |r| {
            Ok((
                (r.get::<_, i64>(0)?, r.get::<_, bool>(1)?),
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<std::collections::HashMap<(i64, bool), i64>>>()?;
    Ok(given
        .into_iter()
        .map(|(account_id, given_paisa, last)| {
            // The query groups by account and kind, so each pair has at most one row: nothing to add.
            let sum = |is_receipt: bool| used.get(&(account_id, is_receipt)).copied().unwrap_or(0);
            AdvanceTotals {
                account_id,
                given_paisa,
                returned_paisa: sum(true),
                adjusted_paisa: sum(false),
                last_date: Some(last),
            }
        })
        .collect())
}

// ---- The collection list (PRD FR-15.6) ----

/// A buyer planned for a collection day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionCandidate {
    /// The buyer.
    pub account: RecordRef,
    /// His phone as typed. Empty if none.
    pub phone: String,
    /// His collector (a staff member), if any.
    pub collector: Option<RecordRef>,
    /// His city, if any.
    pub city: Option<RecordRef>,
}

/// The active buyers (not the cash buyer and not the پختہ کھاتہ account) whose collection
/// weekday is `weekday`, by code, filtered by collector and city.
///
/// # Errors
/// Any SQLite error.
pub fn collection_candidates(
    conn: &Connection,
    weekday: i64,
    collector_staff_id: Option<i64>,
    city_id: Option<i64>,
) -> rusqlite::Result<Vec<CollectionCandidate>> {
    let mut statement = conn.prepare(
        "SELECT a.id, a.code, a.name_ur, a.name_en, COALESCE(a.phone1, ''),
                s.id, s.name_ur, s.name_en, c.id, c.code, c.name_ur, c.name_en
         FROM accounts a
         JOIN account_controls k ON k.id = a.control_id
         LEFT JOIN staff s ON s.id = a.collector_staff_id
         LEFT JOIN cities c ON c.id = a.city_id
         WHERE k.code = '0304' AND a.is_active = 1 AND a.is_pukhta_khata = 0 AND a.code <> '03040000'
           AND a.collection_weekday = ?1
           AND (?2 IS NULL OR a.collector_staff_id = ?2) AND (?3 IS NULL OR a.city_id = ?3)
         ORDER BY a.code",
    )?;
    let rows = statement.query_map(params![weekday, collector_staff_id, city_id], |r| {
        let collector_id: Option<i64> = r.get(5)?;
        let city_id: Option<i64> = r.get(8)?;
        Ok(CollectionCandidate {
            account: RecordRef {
                id: r.get(0)?,
                code: r.get(1)?,
                name_ur: r.get(2)?,
                name_en: r.get(3)?,
            },
            phone: r.get(4)?,
            collector: collector_id
                .map(|id| -> rusqlite::Result<RecordRef> {
                    Ok(RecordRef {
                        id,
                        code: String::new(),
                        name_ur: r.get(6)?,
                        name_en: r.get(7)?,
                    })
                })
                .transpose()?,
            city: city_id
                .map(|id| -> rusqlite::Result<RecordRef> {
                    Ok(RecordRef {
                        id,
                        code: r.get::<_, Option<String>>(9)?.unwrap_or_default(),
                        name_ur: r.get(10)?,
                        name_en: r.get(11)?,
                    })
                })
                .transpose()?,
        })
    })?;
    rows.collect()
}

/// The day of the oldest open bill of each of these accounts (ordinary buyer accounts). An
/// account with no open bill is not in the answer. One query for all of them (Code Standards
/// B22).
///
/// # Errors
/// Any SQLite error.
pub fn oldest_open_bill_dates(
    conn: &Connection,
    account_ids: &[i64],
) -> rusqlite::Result<std::collections::HashMap<i64, String>> {
    if account_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let marks = vec!["?"; account_ids.len()].join(", ");
    let matched = "COALESCE((SELECT SUM(a.amount_paisa) FROM bill_allocations a
                             WHERE a.bill_line_id = l.id AND a.is_active = 1), 0)";
    let mut statement = conn.prepare(&format!(
        "SELECT l.account_id, MIN(e.entry_date)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id IN ({marks}) AND l.dr_paisa > 0 AND {live} AND l.dr_paisa > {matched}
         GROUP BY l.account_id",
        live = crate::matching::LIVE,
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(account_ids), |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?;
    rows.collect()
}

// ---- The day state (PRD FR-6.6, FR-21.1, FR-21.2) ----

/// The date of the oldest posted entry, if there is one.
///
/// # Errors
/// Any SQLite error.
pub fn first_posting_date(conn: &Connection) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT MIN(entry_date) FROM journal_entries WHERE status = 'posted'",
        [],
        |r| r.get(0),
    )
}

/// How many journal entries are not posted. A draft never stays after a save, so any is a bug.
///
/// # Errors
/// Any SQLite error.
pub fn unposted_entries(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
        [],
        |r| r.get(0),
    )
}

/// How many posted entries are dated from `from` to `to` (both included).
///
/// # Errors
/// Any SQLite error.
pub fn entries_between(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM journal_entries WHERE status = 'posted' AND entry_date >= ?1 AND entry_date <= ?2",
        params![from, to],
        |r| r.get(0),
    )
}

/// What one account did on one day: the sums of its posted lines dated that day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movement {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The account.
    pub account_id: i64,
    /// Σ debits that day, paisa.
    pub dr_paisa: i64,
    /// Σ credits that day, paisa.
    pub cr_paisa: i64,
}

/// What each account did on each day from `from` to `to` (both included), in day order. One
/// query, so a close of many days reads the journal once (Code Standards B22).
///
/// # Errors
/// Any SQLite error.
pub fn movements_between(
    conn: &Connection,
    from: &str,
    to: &str,
) -> rusqlite::Result<Vec<Movement>> {
    let mut statement = conn.prepare(
        "SELECT e.entry_date, l.account_id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE e.status = 'posted' AND e.entry_date >= ?1 AND e.entry_date <= ?2
         GROUP BY e.entry_date, l.account_id ORDER BY e.entry_date, l.account_id",
    )?;
    let rows = statement.query_map(params![from, to], |r| {
        Ok(Movement {
            date: r.get(0)?,
            account_id: r.get(1)?,
            dr_paisa: r.get(2)?,
            cr_paisa: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// How many posted entries are dated on each day from `from` to `to`; days with none are not in
/// the answer.
///
/// # Errors
/// Any SQLite error.
pub fn entries_per_day(
    conn: &Connection,
    from: &str,
    to: &str,
) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut statement = conn.prepare(
        "SELECT entry_date, COUNT(*) FROM journal_entries
         WHERE status = 'posted' AND entry_date >= ?1 AND entry_date <= ?2
         GROUP BY entry_date ORDER BY entry_date",
    )?;
    let rows = statement.query_map(params![from, to], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// Moves the "closed up to" date (`None` clears it).
///
/// # Errors
/// Any SQLite error.
pub fn set_closed_up_to(
    conn: &Connection,
    date: Option<&str>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE day_state SET closed_up_to_date = ?1, updated_at = ?2, updated_by = ?3, version = version + 1
         WHERE id = 1",
        params![date, now, user_id],
    )?;
    Ok(())
}

/// Writes a row of the day-close history (`close` or `reopen`).
///
/// # Errors
/// Any SQLite error.
pub fn insert_day_close(
    conn: &Connection,
    entry_date: &str,
    action: &str,
    user_id: i64,
    now: &str,
    reason: Option<&str>,
    summary_json: &str,
) -> rusqlite::Result<()> {
    // Cached: a long close writes thousands of these rows one after another.
    conn.prepare_cached(
        "INSERT INTO day_closes (created_at, created_by, updated_at, updated_by, entry_date, action, by_user_id, at, reason, summary_json)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?2, ?1, ?5, ?6)",
    )?
    .execute(params![now, user_id, entry_date, action, reason, summary_json])?;
    Ok(())
}

/// True if a month-close that is still in force covers `month` (`YYYY-MM`) or a later month
/// (PRD FR-21.2: such a month cannot be reopened until its month close is reversed).
///
/// # Errors
/// Any SQLite error.
pub fn month_closed_from(conn: &Connection, month: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM month_closes WHERE is_active = 1 AND month >= ?1)",
        [month],
        |r| r.get(0),
    )
}

/// What the sales of a range of days came to, as stored (PRD FR-21.1, 4.9): the sums of
/// `analytics_facts` with `sign` applied, so a cancelled or edited بکری counts once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SalesOfDays {
    /// بکری with sales in the range, after the reversals (a بکری that nets to nothing is left out).
    pub bikri_count: i64,
    /// `SUM(sign * seller_amount_paisa)`: خام بکری.
    pub gross_paisa: i64,
    /// `SUM(sign * seller_comm_paisa)`.
    pub seller_comm_paisa: i64,
    /// `SUM(sign * buyer_comm_paisa)`.
    pub buyer_comm_paisa: i64,
    /// `SUM(sign * laga_paisa)`.
    pub laga_paisa: i64,
}

/// The sales of the days from `from` to `to`, both included (`YYYY-MM-DD`). Two indexed reads of
/// `analytics_facts`; only `SUM()` on INTEGER columns is used.
///
/// # Errors
/// Any SQLite error.
pub fn sales_of_days(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<SalesOfDays> {
    let (gross, seller, buyer, laga) = conn.query_row(
        "SELECT COALESCE(SUM(sign * seller_amount_paisa), 0), COALESCE(SUM(sign * seller_comm_paisa), 0),
                COALESCE(SUM(sign * buyer_comm_paisa), 0), COALESCE(SUM(sign * laga_paisa), 0)
         FROM analytics_facts WHERE entry_date >= ?1 AND entry_date <= ?2",
        params![from, to],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        },
    )?;
    let bikri_count = conn.query_row(
        "SELECT COUNT(*) FROM (SELECT bikri_id FROM analytics_facts
                               WHERE entry_date >= ?1 AND entry_date <= ?2
                               GROUP BY bikri_id HAVING SUM(sign) > 0)",
        params![from, to],
        |r| r.get::<_, i64>(0),
    )?;
    Ok(SalesOfDays {
        bikri_count,
        gross_paisa: gross,
        seller_comm_paisa: seller,
        buyer_comm_paisa: buyer,
        laga_paisa: laga,
    })
}

/// The lots of saved (not cancelled) بکری with goods left in باقی مال AS OF the end of the day
/// `to` (`YYYY-MM-DD`; PRD C16, FR-12.14, FR-21.1): how many lots, and their نگ in all. The
/// balance of a lot on that day is the sum of its stock moves dated on or before it (received is
/// plus; sold, written off and taken as fruit are minus; a reversal carries the date of the move
/// it takes back). So a day-close preview made after a later sale still shows what was left on
/// the day itself: 5 نگ on 10-10-26 even when the last 5 were sold on 11-10-26 (PRD E2E-16, 16b).
///
/// # Errors
/// Any SQLite error.
pub fn open_stock(conn: &Connection, to: &str) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(left_qty), 0)
         FROM (SELECT SUM(m.qty) AS left_qty
               FROM stock_moves m
               JOIN bikri_lots l ON l.id = m.lot_id
               JOIN bikri_entries e ON e.id = l.bikri_id
               WHERE m.move_date <= ?1 AND e.status = 'saved'
               GROUP BY m.lot_id
               HAVING SUM(m.qty) > 0)",
        [to],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
    )
}
