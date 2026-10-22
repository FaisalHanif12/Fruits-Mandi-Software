//! The system accounts that exist from the first start (PRD FR-10.4, plan 03 Section 4.4).
//!
//! The first migration run calls [`system_accounts`] inside its own transaction, so a new
//! data file always starts with the heads, the controls and the 16 fixed accounts. Running
//! it again changes nothing.
//!
//! Account names are exactly the Urdu and English names of the FR-10.4 table. The Urdu names
//! of the heads and controls are NOT in FR-10.4 (it gives English names only), so they are
//! provisional: built from words the PRD already uses, and waiting for the Urdu reviewer
//! (NFR-URD-4). Search keys come from `mandi_core::text` only (Code Standards B26).
//!
//! Not seeded here: the `0501xxxx` shop expense heads (masters, plan D2) and the `0602xxxx`
//! and `0603xxxx` partner accounts (partners, plan D3).

use mandi_core::text;
use rusqlite::{Connection, params};

/// A level 1 head of the chart (PRD 10.1).
struct Head {
    code: &'static str,
    name_ur: &'static str,
    name_en: &'static str,
    nature: &'static str,
}

/// A level 2 control of the chart (PRD 10.1).
struct Control {
    head_code: &'static str,
    code: &'static str,
    name_ur: &'static str,
    name_en: &'static str,
    party_type: &'static str,
}

/// A fixed system account of PRD FR-10.4. The code is the control code plus a 4-digit serial.
struct Account {
    control_code: &'static str,
    serial: u32,
    name_ur: &'static str,
    name_en: &'static str,
    party_type: &'static str,
    is_pukhta_khata: bool,
    credit_allowed: bool,
}

/// provisional, needs Urdu reviewer (NFR-URD-4): FR-10.4 gives only the English names.
const HEADS: [Head; 6] = [
    Head {
        code: "01",
        name_ur: "اثاثے",
        name_en: "Assets",
        nature: "asset",
    },
    Head {
        code: "02",
        name_ur: "واجب الادا",
        name_en: "Payables",
        nature: "payable",
    },
    Head {
        code: "03",
        name_ur: "پارٹیاں",
        name_en: "Parties",
        nature: "party",
    },
    Head {
        code: "04",
        name_ur: "آمدن",
        name_en: "Income",
        nature: "income",
    },
    Head {
        code: "05",
        name_ur: "اخراجات",
        name_en: "Expenses",
        nature: "expense",
    },
    Head {
        code: "06",
        name_ur: "سرمایہ",
        name_en: "Capital",
        nature: "capital",
    },
];

/// provisional, needs Urdu reviewer (NFR-URD-4): FR-10.4 gives only the English names.
/// 0601 "Capital" and 0602 "Partner Capital" were agreed with the lead (plan 03 Section 4.2c).
const CONTROLS: [Control; 11] = [
    Control {
        head_code: "01",
        code: "0101",
        name_ur: "روکڑ",
        name_en: "Cash",
        party_type: "other",
    },
    Control {
        head_code: "01",
        code: "0102",
        name_ur: "بینک",
        name_en: "Bank",
        party_type: "other",
    },
    // PRD FR-10.4 v1.2: one ادھار رقم account per person (staff or anyone else) under 0103.
    // The person accounts are made by the user (plan 06), never seeded.
    Control {
        head_code: "01",
        code: "0103",
        name_ur: "ادھار رقم",
        name_en: "Money Lent",
        party_type: "other",
    },
    Control {
        head_code: "02",
        code: "0201",
        name_ur: "واجب الادا",
        name_en: "Payables",
        party_type: "other",
    },
    Control {
        head_code: "03",
        code: "03011",
        name_ur: "بیوپاری",
        name_en: "Sellers",
        party_type: "seller",
    },
    Control {
        head_code: "03",
        code: "0304",
        name_ur: "گاہک",
        name_en: "Buyers",
        party_type: "buyer",
    },
    Control {
        head_code: "04",
        code: "0401",
        name_ur: "آمدن",
        name_en: "Income",
        party_type: "other",
    },
    Control {
        head_code: "05",
        code: "0501",
        name_ur: "دکان خرچہ",
        name_en: "Shop Expenses",
        party_type: "other",
    },
    Control {
        head_code: "06",
        code: "0601",
        name_ur: "سرمایہ",
        name_en: "Capital",
        party_type: "other",
    },
    Control {
        head_code: "06",
        code: "0602",
        name_ur: "شراکت دار سرمایہ",
        name_en: "Partner Capital",
        party_type: "other",
    },
    Control {
        head_code: "06",
        code: "0603",
        name_ur: "ذاتی خرچہ",
        name_en: "Drawings",
        party_type: "other",
    },
];

/// The 16 rows of the FR-10.4 table, in table order. PRD FR-10.4, FR-13.1, FR-10.9.
const ACCOUNTS: [Account; 16] = [
    Account {
        control_code: "0101",
        serial: 1,
        name_ur: "روکڑ",
        name_en: "Cash in Hand",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0102",
        serial: 1,
        name_ur: "بینک",
        name_en: "Bank",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0201",
        serial: 1,
        name_ur: "مزدوری کھاتہ",
        name_en: "Labour Payable",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0201",
        serial: 2,
        name_ur: "کرایہ کھاتہ",
        name_en: "Freight Payable",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0201",
        serial: 3,
        name_ur: "کرایہ سٹور کھاتہ",
        name_en: "Store Rent Payable",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0201",
        serial: 4,
        name_ur: "منشیانہ کھاتہ",
        name_en: "Munshiana Payable",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "03011",
        serial: 9999,
        name_ur: "پختہ کھاتہ (بیوپاری)",
        name_en: "Pukhta Khata (Seller)",
        party_type: "seller",
        is_pukhta_khata: true,
        credit_allowed: true,
    },
    // The cash buyer with no name always pays at once (PRD 12.4).
    Account {
        control_code: "0304",
        serial: 0,
        name_ur: "نقدی منڈی",
        name_en: "Naqdi Mandi (Cash Buyer)",
        party_type: "buyer",
        is_pukhta_khata: false,
        credit_allowed: false,
    },
    Account {
        control_code: "0304",
        serial: 9999,
        name_ur: "پختہ کھاتہ (خریدار)",
        name_en: "Pukhta Khata (Buyer)",
        party_type: "buyer",
        is_pukhta_khata: true,
        credit_allowed: true,
    },
    Account {
        control_code: "0401",
        serial: 1,
        name_ur: "کمیشن آمدن (بیوپاری)",
        name_en: "Commission Income (Seller)",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0401",
        serial: 2,
        name_ur: "کمیشن آمدن (گاہک)",
        name_en: "Commission Income (Buyer)",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0401",
        serial: 3,
        name_ur: "لاگا آمدن",
        name_en: "Laga Income",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0401",
        serial: 4,
        name_ur: "فون آمدن",
        name_en: "Phone Charge Income",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0401",
        serial: 5,
        name_ur: "ریٹ فرق",
        name_en: "Rate Difference",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0601",
        serial: 1,
        name_ur: "ابتدائی سرمایہ",
        name_en: "Opening Capital",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
    Account {
        control_code: "0601",
        serial: 9999,
        name_ur: "ابتدائی فرق",
        name_en: "Opening Difference",
        party_type: "other",
        is_pukhta_khata: false,
        credit_allowed: true,
    },
];

/// Inserts the heads, controls and system accounts that are not there yet, and returns how
/// many rows it added (0 on a second run). Call it inside a transaction: `Transaction`
/// derefs to `Connection`.
///
/// # Errors
/// Any SQLite error, for example a UNIQUE clash with a hand-made account of the same code.
pub fn system_accounts(conn: &Connection) -> rusqlite::Result<usize> {
    let mut added = 0;
    for head in &HEADS {
        added += conn.execute(
            "INSERT INTO account_heads (code, name_ur, name_en, nature) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(code) DO NOTHING",
            params![head.code, head.name_ur, head.name_en, head.nature],
        )?;
    }
    for control in &CONTROLS {
        added += conn.execute(
            "INSERT INTO account_controls (head_id, code, name_ur, name_en, party_type)
             SELECT id, ?2, ?3, ?4, ?5 FROM account_heads WHERE code = ?1
             ON CONFLICT(code) DO NOTHING",
            params![
                control.head_code,
                control.code,
                control.name_ur,
                control.name_en,
                control.party_type
            ],
        )?;
    }
    for account in &ACCOUNTS {
        // PRD FR-10.2: the code is the control code plus the 4-digit serial.
        let code = format!("{}{:04}", account.control_code, account.serial);
        added += conn.execute(
            "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en,
                 search_ur, search_en, search_phonetic, is_system, is_pukhta_khata, credit_allowed)
             SELECT id, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11
             FROM account_controls WHERE code = ?1
             ON CONFLICT(code) DO NOTHING",
            params![
                account.control_code,
                code,
                account.serial,
                account.party_type,
                account.name_ur,
                account.name_en,
                text::search_ur(account.name_ur),
                text::search_en(account.name_en),
                text::search_phonetic(account.name_en),
                account.is_pukhta_khata,
                account.credit_allowed,
            ],
        )?;
    }
    Ok(added)
}

/// The controls added by migration 6 (plan 16b, PRD FR-10.4 v1.6): the separate khatas (0305, party
/// type `separate`, for plan 17), the gardens (0306, party type `garden`, for D4R plan 17c) and the
/// gardens, trips and fruit accounts of the partners (0604, one account for each partner).
/// provisional, needs Urdu reviewer (NFR-URD-4): FR-10.4 gives only the English names.
const TRIP_CONTROLS: [Control; 3] = [
    Control {
        head_code: "03",
        code: "0305",
        name_ur: "الگ کھاتہ",
        name_en: "Separate Khatas",
        party_type: "separate",
    },
    Control {
        head_code: "03",
        code: "0306",
        name_ur: "باغات",
        name_en: "Gardens",
        party_type: "garden",
    },
    Control {
        head_code: "06",
        code: "0604",
        name_ur: "باغ سفر اور پھل",
        name_en: "Gardens Trips and Fruit",
        party_type: "other",
    },
];

/// The system account of migration 6 (PRD FR-10.4, FR-12.26): the seller of a trip's goods, a
/// reserved serial. It never holds a balance. The gardens' accounts and the 0604 accounts of the
/// partners are made by the garden and partner saves, not seeded.
const TRIP_ACCOUNTS: [Account; 1] = [Account {
    control_code: "03011",
    serial: 9998,
    name_ur: "اپنا مال",
    name_en: "Own Goods (Trips)",
    party_type: "seller",
    is_pukhta_khata: false,
    credit_allowed: true,
}];

/// The code of the seller of a trip's goods, اپنا مال (PRD FR-12.26).
pub const OWN_GOODS_ACCOUNT_CODE: &str = "030119998";
/// The code of the cash account, روکڑ (PRD FR-10.4).
pub const CASH_ACCOUNT_CODE: &str = "01010001";
/// The English name of the cash account from v1.6 (PRD FR-10.4).
const CASH_NAME_EN: &str = "Cash";

fn insert_controls(conn: &Connection, controls: &[Control]) -> rusqlite::Result<usize> {
    let mut added = 0;
    for control in controls {
        added += conn.execute(
            "INSERT INTO account_controls (head_id, code, name_ur, name_en, party_type)
             SELECT id, ?2, ?3, ?4, ?5 FROM account_heads WHERE code = ?1
             ON CONFLICT(code) DO NOTHING",
            params![
                control.head_code,
                control.code,
                control.name_ur,
                control.name_en,
                control.party_type
            ],
        )?;
    }
    Ok(added)
}

fn insert_accounts(conn: &Connection, accounts: &[Account]) -> rusqlite::Result<usize> {
    let mut added = 0;
    for account in accounts {
        // PRD FR-10.2: the code is the control code plus the 4-digit serial.
        let code = format!("{}{:04}", account.control_code, account.serial);
        added += conn.execute(
            "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en,
                 search_ur, search_en, search_phonetic, is_system, is_pukhta_khata, credit_allowed)
             SELECT id, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11
             FROM account_controls WHERE code = ?1
             ON CONFLICT(code) DO NOTHING",
            params![
                account.control_code,
                code,
                account.serial,
                account.party_type,
                account.name_ur,
                account.name_en,
                text::search_ur(account.name_ur),
                text::search_en(account.name_en),
                text::search_phonetic(account.name_en),
                account.is_pukhta_khata,
                account.credit_allowed,
            ],
        )?;
    }
    Ok(added)
}

/// Inserts the controls and the system account of migration 6 that are not there yet, gives the cash
/// account its v1.6 English name ("Cash") with its search keys, and returns how many rows it added
/// (0 on a second run). Migration 6 calls it after it has widened the party type checks to accept
/// `separate` and `garden`. [`system_accounts`] is the seed of migration 1 and stays as it was
/// released.
///
/// # Errors
/// Any SQLite error.
pub fn trip_accounts(conn: &Connection) -> rusqlite::Result<usize> {
    let controls = insert_controls(conn, &TRIP_CONTROLS)?;
    let accounts = insert_accounts(conn, &TRIP_ACCOUNTS)?;
    conn.execute(
        "UPDATE accounts SET name_en = ?2, search_en = ?3, search_phonetic = ?4,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), version = version + 1
         WHERE code = ?1 AND name_en <> ?2",
        params![
            CASH_ACCOUNT_CODE,
            CASH_NAME_EN,
            text::search_en(CASH_NAME_EN),
            text::search_phonetic(CASH_NAME_EN),
        ],
    )?;
    Ok(controls + accounts)
}

/// Makes the 0604 account (gardens, trips and fruit) of every partner that has none, and names it in
/// `partners.venture_account_id`. Migration 6 calls it for the partners that exist; the partner save
/// (plan 11) makes the account of each new partner with [`mandi_core::partners::venture_account`].
/// Returns how many accounts it made.
///
/// # Errors
/// Any SQLite error, or `StatementChangedRows` if the 0604 series is full.
pub fn partner_venture_accounts(conn: &Connection) -> rusqlite::Result<usize> {
    let mut statement = conn.prepare(
        "SELECT id, name_ur, name_en FROM partners WHERE venture_account_id IS NULL ORDER BY id",
    )?;
    let partners: Vec<(i64, String, String)> = statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let now: String = conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now')", [], |r| {
        r.get(0)
    })?;
    let owner: i64 = conn.query_row("SELECT COALESCE(MIN(id), 0) FROM users", [], |r| r.get(0))?;
    let mut made = 0;
    for (partner_id, name_ur, name_en) in partners {
        let account = mandi_core::partners::venture_account(&name_ur, &name_en);
        let (account_id, _) = crate::masters::create_account(conn, &account, owner, &now)?
            .ok_or(rusqlite::Error::StatementChangedRows(0))?;
        conn.execute(
            "UPDATE partners SET venture_account_id = ?2 WHERE id = ?1",
            params![partner_id, account_id],
        )?;
        made += 1;
    }
    Ok(made)
}

/// How many heads, controls and accounts a data file holds. `pnpm seed` prints it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedCounts {
    /// Rows in `account_heads`.
    pub heads: i64,
    /// Rows in `account_controls`.
    pub controls: i64,
    /// Rows in `accounts`.
    pub accounts: i64,
}

/// Counts the seeded rows.
///
/// # Errors
/// Any SQLite error.
pub fn counts(conn: &Connection) -> rusqlite::Result<SeedCounts> {
    let count = |table: &str| -> rusqlite::Result<i64> {
        // The table name is one of our three constants, never user input.
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
    };
    Ok(SeedCounts {
        heads: count("account_heads")?,
        controls: count("account_controls")?,
        accounts: count("accounts")?,
    })
}

/// The default charges of PRD 4.2 and Section 21 "Charges" (they are data, NFR-MNT-5): seller
/// commission 6%, buyer commission 6%, and لاگا 8 per نگ on fruit (an item-group rule). Nothing
/// for مزدوری, منشیانہ and فون: they are typed on the lot. They start on `1000-01-01`, the earliest date the
/// app accepts, so any date has a default in force; a change in Settings ends a default on the day
/// before and starts a new row (PRD 11.3). Idempotent: returns 0 if the chain already exists.
///
/// # Errors
/// Any SQLite error.
pub fn default_rules(conn: &Connection) -> rusqlite::Result<usize> {
    use crate::rules::{DEFAULT_RULES_FROM, has_defaults};
    if has_defaults(conn)? {
        return Ok(0);
    }
    let insert = |side: &str,
                  charge: &str,
                  group: Option<&str>,
                  method: &str,
                  bps: Option<i32>,
                  paisa: Option<i64>|
     -> rusqlite::Result<usize> {
        conn.execute(
            "INSERT INTO charge_rules (side, charge_type, item_group, method, value_bps, value_paisa, from_date, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'default (PRD 4.2)')",
            params![side, charge, group, method, bps, paisa, DEFAULT_RULES_FROM],
        )
    };
    let mut added = insert("seller", "commission", None, "percent", Some(600), None)?;
    added += insert("buyer", "commission", None, "percent", Some(600), None)?;
    // 8 rupees = 800 paisa.
    added += insert("buyer", "laga", Some("fruit"), "per_unit", None, Some(800))?;
    Ok(added)
}
