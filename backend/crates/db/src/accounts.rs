//! Accounts in SQL (plan 07, PRD 10.2, FR-10.3, FR-10.5, FR-10.9). Plain statements, no rules:
//! the checks are in `mandi_core::accounts` and `mandi_services::accounts`. Nothing is deleted
//! (PRD FR-10.6). Balances are not here: they come from the journal (`crate::ledger`).

use mandi_core::accounts::{
    AccountFilter, AccountRow, AccountType, Balance, CoaControl, CoaHead, RecordRef,
    SimilarAccount, can_take_opening,
};
use mandi_core::masters::ColValue;
use mandi_core::money::Paisa;
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

use crate::masters::SearchSource;

/// A control with the head it is under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlRow {
    /// Row id.
    pub id: i64,
    /// The head.
    pub head_id: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `seller`, `buyer` or `other`.
    pub party_type: String,
    /// The code of the head (`01` to `06`).
    pub head_code: String,
}

/// What a rule needs to know about an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountFacts {
    /// Row id.
    pub id: i64,
    /// Version.
    pub version: i64,
    /// Full code.
    pub code: String,
    /// Control code.
    pub control_code: String,
    /// Head code.
    pub head_code: String,
    /// A system account.
    pub is_system: bool,
    /// The پختہ کھاتہ account.
    pub is_pukhta_khata: bool,
    /// Active.
    pub is_active: bool,
    /// The opening entry now in force.
    pub opening_entry_id: Option<i64>,
    /// The opening date now in force.
    pub opening_date: Option<String>,
    /// The opening debit now in force, paisa.
    pub opening_dr_paisa: i64,
    /// The opening credit now in force, paisa.
    pub opening_cr_paisa: i64,
    /// The advance flag now in force.
    pub opening_is_advance: bool,
}

fn control_from(r: &Row<'_>) -> rusqlite::Result<ControlRow> {
    Ok(ControlRow {
        id: r.get(0)?,
        head_id: r.get(1)?,
        code: r.get(2)?,
        name_ur: r.get(3)?,
        name_en: r.get(4)?,
        party_type: r.get(5)?,
        head_code: r.get(6)?,
    })
}

const CONTROL_SELECT: &str =
    "SELECT c.id, c.head_id, c.code, c.name_ur, c.name_en, c.party_type, h.code
     FROM account_controls c JOIN account_heads h ON h.id = c.head_id";

/// A control by its code.
///
/// # Errors
/// Any SQLite error.
pub fn control_by_code(conn: &Connection, code: &str) -> rusqlite::Result<Option<ControlRow>> {
    conn.query_row(
        &format!("{CONTROL_SELECT} WHERE c.code = ?1"),
        [code],
        control_from,
    )
    .optional()
}

/// A control by its id.
///
/// # Errors
/// Any SQLite error.
pub fn control_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<ControlRow>> {
    conn.query_row(
        &format!("{CONTROL_SELECT} WHERE c.id = ?1"),
        [id],
        control_from,
    )
    .optional()
}

/// Which of these accounts are inactive, hidden from pickers (PRD FR-10.2), in id order. One
/// query for the whole list; the `?` marks are only as many as there are ids.
///
/// # Errors
/// Any SQLite error.
pub fn inactive_among(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<i64>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT id FROM accounts WHERE is_active = 0 AND id IN ({marks}) ORDER BY id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(ids), |r| r.get(0))?;
    rows.collect()
}

/// The facts a rule needs about one account.
///
/// # Errors
/// Any SQLite error.
pub fn facts(conn: &Connection, id: i64) -> rusqlite::Result<Option<AccountFacts>> {
    conn.query_row(
        "SELECT a.id, a.version, a.code, c.code, h.code, a.is_system, a.is_pukhta_khata, a.is_active,
                a.opening_entry_id, a.opening_date, a.opening_dr_paisa, a.opening_cr_paisa, a.opening_is_advance
         FROM accounts a JOIN account_controls c ON c.id = a.control_id JOIN account_heads h ON h.id = c.head_id
         WHERE a.id = ?1",
        [id],
        |r| {
            Ok(AccountFacts {
                id: r.get(0)?,
                version: r.get(1)?,
                code: r.get(2)?,
                control_code: r.get(3)?,
                head_code: r.get(4)?,
                is_system: r.get(5)?,
                is_pukhta_khata: r.get(6)?,
                is_active: r.get(7)?,
                opening_entry_id: r.get(8)?,
                opening_date: r.get(9)?,
                opening_dr_paisa: r.get(10)?,
                opening_cr_paisa: r.get(11)?,
                opening_is_advance: r.get(12)?,
            })
        },
    )
    .optional()
}

/// The id of a system account by code, for example `06019999` (ابتدائی فرق).
///
/// # Errors
/// Any SQLite error.
pub fn id_by_code(conn: &Connection, code: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
        r.get(0)
    })
    .optional()
}

/// True if another account (not `except_id`) already has this code.
///
/// # Errors
/// Any SQLite error.
pub fn code_taken(conn: &Connection, code: &str, except_id: Option<i64>) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM accounts WHERE code = ?1 AND id IS NOT ?2",
            params![code, except_id],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

/// The next free serial under a control: the largest normal serial (1 to 8999) plus one. The
/// reserved serials are ignored (PRD FR-10.3). `None` when 8999 is used.
///
/// # Errors
/// Any SQLite error.
pub fn next_serial(conn: &Connection, control_id: i64) -> rusqlite::Result<Option<u32>> {
    let largest: i64 = conn.query_row(
        "SELECT COALESCE(MAX(serial), 0) FROM accounts WHERE control_id = ?1 AND serial BETWEEN 1 AND 8999",
        [control_id],
        |r| r.get(0),
    )?;
    let next = largest.saturating_add(1);
    Ok(if next <= 8999 {
        u32::try_from(next).ok()
    } else {
        None
    })
}

/// SQL that gives the kind of account from its control code.
const TYPE_SQL: &str = "CASE c.code WHEN '03011' THEN 'seller' WHEN '0304' THEN 'buyer' WHEN '0103' THEN 'money_lent' ELSE 'other' END";

const ROW_SELECT: &str = "SELECT a.id, a.version, a.code, a.serial,
        c.code, c.id, c.name_ur, c.name_en,
        a.name_ur, a.name_en, a.father_or_firm_ur, a.father_or_firm_en,
        ci.id, ci.code, ci.name_ur, ci.name_en,
        ad.id, ad.code, ad.name_ur, ad.name_en,
        a.phone1, a.phone2, a.address,
        l.id, l.code, l.name_ur, l.name_en,
        a.opening_date, a.opening_dr_paisa, a.opening_cr_paisa, a.opening_is_advance,
        a.credit_allowed, a.credit_limit_paisa,
        mk.id, mk.code, mk.mark_text,
        st.id, st.name_ur, st.name_en,
        a.collection_weekday, a.is_system, a.is_pukhta_khata, a.is_active, a.note, h.code
     FROM accounts a
     JOIN account_controls c ON c.id = a.control_id
     JOIN account_heads h ON h.id = c.head_id
     LEFT JOIN cities ci ON ci.id = a.city_id
     LEFT JOIN addas ad ON ad.id = a.adda_id
     LEFT JOIN accounts l ON l.id = a.linked_account_id
     LEFT JOIN markas mk ON mk.id = a.default_marka_id
     LEFT JOIN staff st ON st.id = a.collector_staff_id";

fn optional_ref(
    r: &Row<'_>,
    first: usize,
    code: Option<usize>,
) -> rusqlite::Result<Option<RecordRef>> {
    let id: Option<i64> = r.get(first)?;
    let Some(id) = id else {
        return Ok(None);
    };
    // With a code column (accounts, cities...) the layout is id, code, name_ur, name_en.
    // Without one (staff) it is id, name_ur, name_en and the code is empty.
    Ok(Some(match code {
        Some(_) => RecordRef {
            id,
            code: r.get(first + 1)?,
            name_ur: r.get(first + 2)?,
            name_en: r.get(first + 3)?,
            tag: None,
        },
        None => RecordRef {
            id,
            code: String::new(),
            name_ur: r.get(first + 1)?,
            name_en: r.get(first + 2)?,
            tag: None,
        },
    }))
}

fn paisa(value: i64) -> rusqlite::Result<Paisa> {
    Paisa::try_from_paisa(value).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Integer, Box::new(e))
    })
}

fn row_from(r: &Row<'_>) -> rusqlite::Result<AccountRow> {
    let control_code: String = r.get(4)?;
    let account_type = match control_code.as_str() {
        "03011" => AccountType::Seller,
        "0304" => AccountType::Buyer,
        "0103" => AccountType::MoneyLent,
        _ => AccountType::Other,
    };
    let credit_limit: Option<i64> = r.get(32)?;
    Ok(AccountRow {
        id: r.get(0)?,
        version: r.get(1)?,
        code: r.get(2)?,
        serial: r.get(3)?,
        account_type,
        control: RecordRef {
            id: r.get(5)?,
            code: control_code,
            name_ur: r.get(6)?,
            name_en: r.get(7)?,
            tag: None,
        },
        name_ur: r.get(8)?,
        name_en: r.get(9)?,
        father_or_firm_ur: r.get::<_, Option<String>>(10)?.unwrap_or_default(),
        father_or_firm_en: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
        city: optional_ref(r, 12, Some(13))?,
        adda: optional_ref(r, 16, Some(17))?,
        phone1: r.get::<_, Option<String>>(20)?.unwrap_or_default(),
        phone2: r.get::<_, Option<String>>(21)?.unwrap_or_default(),
        address: r.get::<_, Option<String>>(22)?.unwrap_or_default(),
        linked_account: optional_ref(r, 23, Some(24))?,
        opening_date: r.get(27)?,
        opening_dr_paisa: paisa(r.get(28)?)?,
        opening_cr_paisa: paisa(r.get(29)?)?,
        opening_is_advance: r.get(30)?,
        credit_allowed: r.get(31)?,
        credit_limit_paisa: credit_limit.map(paisa).transpose()?,
        default_marka: marka_ref(r)?,
        collector: optional_ref(r, 36, None)?,
        collection_weekday: r.get(39)?,
        is_system: r.get(40)?,
        can_take_opening: can_take_opening(
            &r.get::<_, String>(44)?,
            &r.get::<_, String>(2)?,
            r.get(41)?,
        ),
        is_pukhta_khata: r.get(41)?,
        is_active: r.get(42)?,
        note: r.get::<_, Option<String>>(43)?.unwrap_or_default(),
        balance: Balance::ZERO,
        tag: None,
    })
}

/// A مارکہ shows its text as both names.
fn marka_ref(r: &Row<'_>) -> rusqlite::Result<Option<RecordRef>> {
    let id: Option<i64> = r.get(33)?;
    let Some(id) = id else {
        return Ok(None);
    };
    let text: String = r.get(35)?;
    Ok(Some(RecordRef {
        id,
        code: r.get(34)?,
        name_ur: text.clone(),
        name_en: text,
        tag: None,
    }))
}

/// One account with its names of what it points at. The balance is `Balance::ZERO`: the
/// service fills it from the journal.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<AccountRow>> {
    conn.query_row(&format!("{ROW_SELECT} WHERE a.id = ?1"), [id], row_from)
        .optional()
}

/// The WHERE part and its parameters for a filter. `text_ur` and `text_en` are search keys made
/// by `core::text`. Parameters are numbered from 1 in the order of the returned values.
fn filter_sql(filter: &AccountFilter, text_ur: &str, text_en: &str) -> (String, Vec<Value>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    if !filter.include_inactive {
        conditions.push("a.is_active = 1".to_owned());
    }
    if let Some(kind) = filter.account_type {
        values.push(Value::Text(kind.as_str().to_owned()));
        conditions.push(format!("{TYPE_SQL} = ?{}", values.len()));
    }
    if !text_ur.is_empty() || !text_en.is_empty() {
        values.push(Value::Text(text_ur.to_owned()));
        let ur = values.len();
        values.push(Value::Text(text_en.to_owned()));
        let en = values.len();
        conditions.push(format!(
            "(instr(a.code, ?{en}) > 0 OR instr(a.search_ur, ?{ur}) > 0 OR instr(a.search_en, ?{en}) > 0)"
        ));
    }
    let sql = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    (sql, values)
}

/// What the list returns besides its rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListTotals {
    /// How many accounts match.
    pub count: u32,
    /// Σ opening debits of all matching accounts, paisa.
    pub opening_dr_paisa: i64,
    /// Σ opening credits of all matching accounts, paisa.
    pub opening_cr_paisa: i64,
}

/// One page of accounts by code, and the totals of all that match (PRD FR-10.5).
///
/// # Errors
/// Any SQLite error.
pub fn list(
    conn: &Connection,
    filter: &AccountFilter,
    text_ur: &str,
    text_en: &str,
    offset: u32,
    limit: u32,
) -> rusqlite::Result<(Vec<AccountRow>, ListTotals)> {
    let (where_sql, values) = filter_sql(filter, text_ur, text_en);
    let totals = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(a.opening_dr_paisa), 0), COALESCE(SUM(a.opening_cr_paisa), 0)
             FROM accounts a JOIN account_controls c ON c.id = a.control_id{where_sql}"
        ),
        params_from_iter(values.iter()),
        |r| {
            Ok(ListTotals {
                count: r.get(0)?,
                opening_dr_paisa: r.get(1)?,
                opening_cr_paisa: r.get(2)?,
            })
        },
    )?;
    let mut page = values;
    page.push(Value::Integer(i64::from(limit)));
    let limit_mark = page.len();
    page.push(Value::Integer(i64::from(offset)));
    let offset_mark = page.len();
    let mut statement = conn.prepare(&format!(
        "{ROW_SELECT}{where_sql} ORDER BY a.code, a.id LIMIT ?{limit_mark} OFFSET ?{offset_mark}"
    ))?;
    let rows = statement
        .query_map(params_from_iter(page.iter()), row_from)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok((rows, totals))
}

/// Adds an account. `fields` are the columns of `PreparedAccount::fields`. Returns the id.
///
/// # Errors
/// Any SQLite error (a UNIQUE clash on the code, a missing foreign key).
#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    control_id: i64,
    code: &str,
    serial: u32,
    party_type: &str,
    fields: &[(&'static str, ColValue)],
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    let mut columns: Vec<&str> = vec![
        "created_at",
        "created_by",
        "updated_at",
        "updated_by",
        "control_id",
        "code",
        "serial",
        "party_type",
    ];
    let mut values: Vec<Value> = vec![
        Value::Text(now.to_owned()),
        Value::Integer(user_id),
        Value::Text(now.to_owned()),
        Value::Integer(user_id),
        Value::Integer(control_id),
        Value::Text(code.to_owned()),
        Value::Integer(i64::from(serial)),
        Value::Text(party_type.to_owned()),
    ];
    for (name, v) in fields {
        columns.push(name);
        values.push(match v {
            ColValue::Text(t) => Value::Text(t.clone()),
            ColValue::Int(i) => Value::Integer(*i),
            ColValue::Null => Value::Null,
        });
    }
    let marks: Vec<String> = (1..=columns.len()).map(|n| format!("?{n}")).collect();
    conn.execute(
        &format!(
            "INSERT INTO accounts ({}) VALUES ({})",
            columns.join(", "),
            marks.join(", ")
        ),
        params_from_iter(values),
    )?;
    Ok(conn.last_insert_rowid())
}

/// Edits an account whose version is `version`; the version goes up by one. Returns false if no
/// row has this id and version. The code, serial, control and opening columns do not change here.
///
/// # Errors
/// Any SQLite error.
pub fn update(
    conn: &Connection,
    id: i64,
    version: i64,
    fields: &[(&'static str, ColValue)],
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let mut sets: Vec<String> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    let mut push = |name: &str, v: Value| {
        values.push(v);
        sets.push(format!("{name} = ?{}", values.len()));
    };
    for (name, v) in fields {
        push(
            name,
            match v {
                ColValue::Text(t) => Value::Text(t.clone()),
                ColValue::Int(i) => Value::Integer(*i),
                ColValue::Null => Value::Null,
            },
        );
    }
    push("updated_at", Value::Text(now.to_owned()));
    push("updated_by", Value::Integer(user_id));
    push("version", Value::Integer(version.saturating_add(1)));
    values.push(Value::Integer(id));
    let id_mark = values.len();
    values.push(Value::Integer(version));
    let version_mark = values.len();
    let changed = conn.execute(
        &format!(
            "UPDATE accounts SET {} WHERE id = ?{id_mark} AND version = ?{version_mark}",
            sets.join(", ")
        ),
        params_from_iter(values),
    )?;
    Ok(changed == 1)
}

/// Records the opening now in force on the account (the date, both amounts, the advance flag and
/// the entry that posted it). The entry itself is written by the ledger.
///
/// # Errors
/// Any SQLite error (the CHECKs: only one side, an advance needs a debit).
#[allow(clippy::too_many_arguments)]
pub fn set_opening(
    conn: &Connection,
    id: i64,
    date: Option<&str>,
    dr_paisa: i64,
    cr_paisa: i64,
    is_advance: bool,
    entry_id: Option<i64>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET opening_date = ?1, opening_dr_paisa = ?2, opening_cr_paisa = ?3,
             opening_is_advance = ?4, opening_entry_id = ?5, updated_at = ?6, updated_by = ?7,
             version = version + 1 WHERE id = ?8",
        params![
            date, dr_paisa, cr_paisa, is_advance, entry_id, now, user_id, id
        ],
    )?;
    Ok(())
}

/// Sets the active flag and raises the version. Returns false if there is no such row.
///
/// # Errors
/// Any SQLite error.
pub fn set_active(
    conn: &Connection,
    id: i64,
    is_active: bool,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE accounts SET is_active = ?1, updated_at = ?2, updated_by = ?3, version = version + 1 WHERE id = ?4",
        params![is_active, now, user_id, id],
    )?;
    Ok(changed == 1)
}

/// The accounts whose phone is this number (either phone column).
///
/// # Errors
/// Any SQLite error.
pub fn ids_by_phone(
    conn: &Connection,
    phone: &str,
    except_id: Option<i64>,
) -> rusqlite::Result<Vec<i64>> {
    let mut statement = conn.prepare(
        "SELECT id FROM accounts WHERE (phone1 = ?1 OR phone2 = ?1) AND id IS NOT ?2 ORDER BY code LIMIT 20",
    )?;
    let rows = statement.query_map(params![phone, except_id], |r| r.get(0))?;
    rows.collect()
}

/// Close matches shown while a name is typed, by the ids given, in that order.
///
/// # Errors
/// Any SQLite error.
pub fn similar_by_ids(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<SimilarAccount>> {
    let mut out = Vec::new();
    for id in ids {
        if let Some(row) = get(conn, *id)? {
            out.push(SimilarAccount {
                id: row.id,
                code: row.code,
                name_ur: row.name_ur,
                name_en: row.name_en,
                father_or_firm_ur: row.father_or_firm_ur,
                father_or_firm_en: row.father_or_firm_en,
                city_ur: row
                    .city
                    .as_ref()
                    .map(|c| c.name_ur.clone())
                    .unwrap_or_default(),
                city_en: row
                    .city
                    .as_ref()
                    .map(|c| c.name_en.clone())
                    .unwrap_or_default(),
                phone1: row.phone1,
                account_type: row.account_type,
                tag: None,
            });
        }
    }
    Ok(out)
}

/// The heads in code order.
///
/// # Errors
/// Any SQLite error.
pub fn heads(conn: &Connection) -> rusqlite::Result<Vec<CoaHead>> {
    let mut statement =
        conn.prepare("SELECT id, code, name_ur, name_en, nature FROM account_heads ORDER BY code")?;
    let rows = statement.query_map([], |r| {
        Ok(CoaHead {
            id: r.get(0)?,
            code: r.get(1)?,
            name_ur: r.get(2)?,
            name_en: r.get(3)?,
            nature: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// The controls in code order, with the flag for accounts of type `other`.
///
/// # Errors
/// Any SQLite error.
pub fn controls(conn: &Connection) -> rusqlite::Result<Vec<CoaControl>> {
    let mut statement = conn.prepare(
        "SELECT id, head_id, code, name_ur, name_en, party_type FROM account_controls ORDER BY code",
    )?;
    let rows = statement.query_map([], |r| {
        let code: String = r.get(2)?;
        let party_type: String = r.get(5)?;
        // 0103 has its own type; sellers and buyers have theirs. 0501 and 0102 are filled by the
        // expense heads and bank accounts of the masters; 06019999 and the like are system rows.
        let allowed = mandi_core::accounts::ControlRule::allows_other(&code, &party_type);
        Ok(CoaControl {
            id: r.get(0)?,
            head_id: r.get(1)?,
            code,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            party_type,
            allowed_for_other: allowed,
        })
    })?;
    rows.collect()
}

/// An account in the tree, with the ids of its control, before balances are added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeAccount {
    /// Row id.
    pub id: i64,
    /// The control it is under.
    pub control_id: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Active.
    pub is_active: bool,
    /// A system account.
    pub is_system: bool,
}

/// Every account for the chart, by code.
///
/// # Errors
/// Any SQLite error.
pub fn tree_accounts(conn: &Connection) -> rusqlite::Result<Vec<TreeAccount>> {
    let mut statement = conn.prepare(
        "SELECT id, control_id, code, name_ur, name_en, is_active, is_system FROM accounts ORDER BY code, id",
    )?;
    let rows = statement.query_map([], |r| {
        Ok(TreeAccount {
            id: r.get(0)?,
            control_id: r.get(1)?,
            code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            is_active: r.get(5)?,
            is_system: r.get(6)?,
        })
    })?;
    rows.collect()
}

/// Accounts as the search index sees them (plan 07 account scopes). The columns are as in
/// `masters::search_sources`; the second line is father or firm and city, and the extra keys
/// are father or firm, city and both phones (PRD FR-7.40). `only_id` limits it to one account.
///
/// # Errors
/// Any SQLite error.
pub fn search_sources(
    conn: &Connection,
    only_id: Option<i64>,
) -> rusqlite::Result<Vec<SearchSource>> {
    let sql = format!(
        "SELECT a.id, a.code, a.name_ur, a.name_en,
                TRIM(COALESCE(a.father_or_firm_ur, '') || ' ' || COALESCE(ci.name_ur, '')),
                TRIM(COALESCE(a.father_or_firm_en, '') || ' ' || COALESCE(ci.name_en, '')),
                TRIM(COALESCE(a.father_or_firm_ur, '') || ' ' || COALESCE(a.father_or_firm_en, '') || ' ' ||
                     COALESCE(ci.name_ur, '') || ' ' || COALESCE(ci.name_en, '') || ' ' ||
                     COALESCE(a.phone1, '') || ' ' || COALESCE(a.phone2, '')),
                a.search_ur, a.search_en, a.search_phonetic,
                -- The account of a closed garden is offered in no picker and no search (PRD FR-12.38).
                (a.is_active = 1 AND NOT EXISTS (SELECT 1 FROM ventures g WHERE g.account_id = a.id
                                                 AND g.kind = 'garden' AND g.status = 'closed')),
                {TYPE_SQL}, a.control_id
         FROM accounts a
         JOIN account_controls c ON c.id = a.control_id
         LEFT JOIN cities ci ON ci.id = a.city_id
         WHERE (?1 IS NULL OR a.id = ?1) AND a.code <> ?2"
    );
    let mut statement = conn.prepare(&sql)?;
    // بینک is never offered in a picker or a search: all money is روکڑ (PRD FR-10.4, v1.6).
    let rows = statement.query_map(
        params![only_id, mandi_core::accounts::BANK_ACCOUNT_CODE],
        |r| {
            let kind: String = r.get(11)?;
            Ok(SearchSource {
                id: r.get(0)?,
                code: r.get(1)?,
                name_ur: r.get(2)?,
                name_en: r.get(3)?,
                extra_ur: r.get(4)?,
                extra_en: r.get(5)?,
                more_keys: r.get(6)?,
                search_ur: r.get(7)?,
                search_en: r.get(8)?,
                search_phonetic: r.get(9)?,
                is_active: r.get(10)?,
                account_type: Some(match kind.as_str() {
                    "seller" => AccountType::Seller,
                    "buyer" => AccountType::Buyer,
                    "money_lent" => AccountType::MoneyLent,
                    _ => AccountType::Other,
                }),
                control_id: Some(r.get(12)?),
            })
        },
    )?;
    rows.collect()
}

/// Gives an account its note (the note of a separate khata).
///
/// # Errors
/// Any SQLite error.
pub fn set_account_note(
    conn: &Connection,
    account_id: i64,
    note: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET note = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![account_id, note, now, user_id],
    )?;
    Ok(())
}
