//! Repositories of the ten master lists of PRD 10.3 (plan 06). Plain SQL, no rules: the checks
//! are in `mandi_core::masters::prepare` and `mandi_services::masters`.
//!
//! Every statement uses parameters (Code Standards B14). The table and column names in the
//! statements come from the fixed lists in `mandi_core::masters`, never from input. Nothing is
//! ever deleted here (PRD E21, Code Standards D3).

use mandi_core::masters::{
    AccountToMake, AddaRow, CityRow, ColValue, ExpenseHeadRow, ItemRow, MarkaRow, MasterKind,
    MasterRow, NarrationRow, PackingRow, StaffRow, StoreRow, format_kg,
};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

/// What one master looks like to the search index. Search keys are `None` for the lists whose
/// tables have no search columns (marks, narrations); the services build them with `core::text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSource {
    /// Row id.
    pub id: i64,
    /// Code, or empty.
    pub code: String,
    /// Urdu name (or the mark text, or the Urdu narration).
    pub name_ur: String,
    /// English name (or the mark text, or the English narration).
    pub name_en: String,
    /// Second line, Urdu.
    pub extra_ur: String,
    /// Second line, English.
    pub extra_en: String,
    /// Extra words the English key should also find (bank name, title and number).
    pub more_keys: String,
    /// `search_ur` of the table, if it has one.
    pub search_ur: Option<String>,
    /// `search_en` of the table, if it has one.
    pub search_en: Option<String>,
    /// `search_phonetic` of the table, if it has one.
    pub search_phonetic: Option<String>,
    /// True for an active record.
    pub is_active: bool,
    /// For an account: its kind (decides which search lists it belongs to). `None` for masters.
    pub account_type: Option<mandi_core::accounts::AccountType>,
    /// For an account: the control it is under. `None` for masters.
    pub control_id: Option<i64>,
    /// For an account: the code of that control (a garden's account has the code of its garden,
    /// `0080`, so the control tells it apart). `None` for masters.
    pub control_code: Option<String>,
    /// For an account: its tag (نشان, PRD 10.2). `None` for masters and for an account with none.
    pub tag: Option<String>,
}

fn value(v: &ColValue) -> Value {
    match v {
        ColValue::Text(t) => Value::Text(t.clone()),
        ColValue::Int(i) => Value::Integer(*i),
        ColValue::Null => Value::Null,
    }
}

fn used_in(json: Option<String>) -> Vec<String> {
    // The column holds a JSON list of words written by `prepare`. Anything else reads as empty.
    json.map(|text| {
        text.trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(|part| part.trim().trim_matches('"').to_owned())
            .filter(|part| !part.is_empty())
            .collect()
    })
    .unwrap_or_default()
}

/// The SELECT of one kind: the row columns first, in the order of `row_from`. Joins bring the
/// names of the records it points at.
fn select(kind: MasterKind) -> &'static str {
    match kind {
        MasterKind::City => {
            "SELECT m.id, m.version, m.code, m.name_ur, m.name_en, m.is_active FROM cities m"
        }
        MasterKind::Store => {
            "SELECT m.id, m.version, m.code, m.name_ur, m.name_en, m.is_active FROM stores m"
        }
        MasterKind::Item => {
            "SELECT m.id, m.version, m.code, m.name_ur, m.name_en, m.item_group, m.default_packing_id,
                    p.name_ur, p.name_en, m.sold_by, m.is_active
             FROM items m LEFT JOIN packings p ON p.id = m.default_packing_id"
        }
        MasterKind::Packing => {
            "SELECT m.id, m.version, m.code, m.name_ur, m.name_en, m.approx_grams, m.is_active FROM packings m"
        }
        MasterKind::Adda => {
            "SELECT m.id, m.version, m.code, m.name_ur, m.name_en, m.city_id, c.name_ur, c.name_en, m.is_active
             FROM addas m LEFT JOIN cities c ON c.id = m.city_id"
        }
        MasterKind::Marka => {
            "SELECT m.id, m.version, m.code, m.mark_text, m.owner_account_id, a.code, a.name_ur, a.name_en, m.is_active
             FROM markas m LEFT JOIN accounts a ON a.id = m.owner_account_id"
        }
        MasterKind::Narration => {
            "SELECT m.id, m.version, m.text_ur, m.text_en, m.used_in_json, m.is_active FROM narrations m"
        }
        MasterKind::ExpenseHead => {
            "SELECT m.id, m.version, m.code, m.name_ur, m.name_en, m.account_id, a.code, m.is_active
             FROM expense_heads m JOIN accounts a ON a.id = m.account_id"
        }
        MasterKind::Staff => {
            "SELECT m.id, m.version, m.code, m.name_ur, m.name_en, m.role, m.phone, m.account_id, a.code, m.is_active
             FROM staff m LEFT JOIN accounts a ON a.id = m.account_id"
        }
    }
}

fn row_from(kind: MasterKind, r: &Row<'_>) -> rusqlite::Result<MasterRow> {
    Ok(match kind {
        MasterKind::City => MasterRow::City(CityRow {
            id: r.get(0)?,
            version: r.get(1)?,
            code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            is_active: r.get(5)?,
        }),
        MasterKind::Store => MasterRow::Store(StoreRow {
            id: r.get(0)?,
            version: r.get(1)?,
            code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            is_active: r.get(5)?,
        }),
        MasterKind::Item => MasterRow::Item(ItemRow {
            id: r.get(0)?,
            version: r.get(1)?,
            code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            item_group: r.get(5)?,
            default_packing_id: r.get(6)?,
            default_packing_name_ur: r.get(7)?,
            default_packing_name_en: r.get(8)?,
            sold_by: r.get(9)?,
            is_active: r.get(10)?,
        }),
        MasterKind::Packing => {
            let grams: Option<i64> = r.get(5)?;
            MasterRow::Packing(PackingRow {
                id: r.get(0)?,
                version: r.get(1)?,
                code: r.get(2)?,
                name_ur: r.get(3)?,
                name_en: r.get(4)?,
                approx_kg: grams.and_then(format_kg),
                is_active: r.get(6)?,
            })
        }
        MasterKind::Adda => MasterRow::Adda(AddaRow {
            id: r.get(0)?,
            version: r.get(1)?,
            code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            city_id: r.get(5)?,
            city_name_ur: r.get(6)?,
            city_name_en: r.get(7)?,
            is_active: r.get(8)?,
        }),
        MasterKind::Marka => MasterRow::Marka(MarkaRow {
            id: r.get(0)?,
            version: r.get(1)?,
            code: r.get(2)?,
            mark_text: r.get(3)?,
            owner_account_id: r.get(4)?,
            owner_account_code: r.get(5)?,
            owner_name_ur: r.get(6)?,
            owner_name_en: r.get(7)?,
            is_active: r.get(8)?,
        }),
        MasterKind::Narration => MasterRow::Narration(NarrationRow {
            id: r.get(0)?,
            version: r.get(1)?,
            text_ur: r.get(2)?,
            text_en: r.get(3)?,
            used_in: used_in(r.get(4)?),
            is_active: r.get(5)?,
        }),
        MasterKind::ExpenseHead => MasterRow::ExpenseHead(ExpenseHeadRow {
            id: r.get(0)?,
            version: r.get(1)?,
            code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            account_id: r.get(5)?,
            account_code: r.get(6)?,
            is_active: r.get(7)?,
        }),
        MasterKind::Staff => MasterRow::Staff(StaffRow {
            id: r.get(0)?,
            version: r.get(1)?,
            code: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            job: r.get(5)?,
            phone: r.get(6)?,
            account_id: r.get(7)?,
            account_code: r.get(8)?,
            is_active: r.get(9)?,
        }),
    })
}

/// Every master of one kind. Kinds with a code come in code order (a number first), the others
/// in the order they were added. Inactive ones only when asked.
///
/// # Errors
/// Any SQLite error.
pub fn list(
    conn: &Connection,
    kind: MasterKind,
    include_inactive: bool,
) -> rusqlite::Result<Vec<MasterRow>> {
    let filter = if include_inactive {
        ""
    } else {
        " WHERE m.is_active = 1"
    };
    let order = if kind.has_code() {
        " ORDER BY CAST(m.code AS INTEGER), m.code, m.id"
    } else {
        " ORDER BY m.id"
    };
    let mut statement = conn.prepare(&format!("{}{filter}{order}", select(kind)))?;
    let rows = statement.query_map([], |r| row_from(kind, r))?;
    rows.collect()
}

/// One page of the masters of a kind, as [`list`] orders them, with how many match and how many
/// of those are active, counted over the whole filter in one query (PRD FR-19.11, B31).
///
/// # Errors
/// Any SQLite error.
pub fn list_page(
    conn: &Connection,
    kind: MasterKind,
    include_inactive: bool,
    id: Option<i64>,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<(Vec<MasterRow>, i64, i64)> {
    // `:id` is the one record asked for, or null for no record in particular.
    let filter = if include_inactive {
        " WHERE (:id IS NULL OR m.id = :id)"
    } else {
        " WHERE m.is_active = 1 AND (:id IS NULL OR m.id = :id)"
    };
    // `kind.table()` is one of a fixed list of names, never input.
    let (count, active): (i64, i64) = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(m.is_active), 0) FROM {} m{filter}",
            kind.table()
        ),
        rusqlite::named_params! { ":id": id },
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let order = if kind.has_code() {
        " ORDER BY CAST(m.code AS INTEGER), m.code, m.id"
    } else {
        " ORDER BY m.id"
    };
    let mut statement = conn.prepare(&format!(
        "{}{filter}{order} LIMIT :limit OFFSET :offset",
        select(kind)
    ))?;
    let rows = statement.query_map(
        rusqlite::named_params! { ":id": id, ":limit": limit, ":offset": offset },
        |r| row_from(kind, r),
    )?;
    Ok((rows.collect::<rusqlite::Result<Vec<_>>>()?, count, active))
}

/// One master by id.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, kind: MasterKind, id: i64) -> rusqlite::Result<Option<MasterRow>> {
    conn.query_row(&format!("{} WHERE m.id = ?1", select(kind)), [id], |r| {
        row_from(kind, r)
    })
    .optional()
}

/// True if a record with this id exists in the table. `table` is one of the fixed names of
/// `mandi_core::masters::Reference`.
///
/// # Errors
/// Any SQLite error, or a table name that is not one of the allowed ones.
pub fn exists(conn: &Connection, table: &str, id: i64) -> rusqlite::Result<bool> {
    let sql = match table {
        "packings" => "SELECT 1 FROM packings WHERE id = ?1",
        "cities" => "SELECT 1 FROM cities WHERE id = ?1",
        "accounts" => "SELECT 1 FROM accounts WHERE id = ?1",
        "addas" => "SELECT 1 FROM addas WHERE id = ?1",
        "markas" => "SELECT 1 FROM markas WHERE id = ?1",
        "staff" => "SELECT 1 FROM staff WHERE id = ?1",
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(conn.query_row(sql, [id], |_| Ok(())).optional()?.is_some())
}

/// True if another record of the kind (not `except_id`) already has this code.
///
/// # Errors
/// Any SQLite error.
pub fn code_taken(
    conn: &Connection,
    kind: MasterKind,
    code: &str,
    except_id: Option<i64>,
) -> rusqlite::Result<bool> {
    if !kind.has_code() {
        return Ok(false);
    }
    let sql = format!(
        "SELECT 1 FROM {} WHERE code = ?1 AND id IS NOT ?2",
        kind.table()
    );
    Ok(conn
        .query_row(&sql, params![code, except_id], |_| Ok(()))
        .optional()?
        .is_some())
}

/// The next free code. An item, a packing or a staff member gets the type number plus the next
/// running number (`0040`, `0050`, `0060` for the first, plan 32 R2). Every other kind: the largest
/// number among the codes plus one, written as wide as the widest code so far (`07` gives `08`),
/// `1` for an empty list. Empty for kinds without a code. Only codes of digits count (the app
/// allows nothing else). `None` if the next code would be longer than the 10 digits the app accepts
/// (the list is full).
///
/// # Errors
/// Any SQLite error.
pub fn next_code(conn: &Connection, kind: MasterKind) -> rusqlite::Result<Option<String>> {
    if !kind.has_code() {
        return Ok(Some(String::new()));
    }
    if let Some(code_type) = kind.code_type() {
        let table = match kind {
            MasterKind::Item => crate::codes::CodeTable::Items,
            MasterKind::Packing => crate::codes::CodeTable::Packings,
            _ => crate::codes::CodeTable::Staff,
        };
        return Ok(crate::codes::next_code(conn, table, code_type)?.map(|(_, code)| code));
    }
    let mut statement = conn.prepare(&format!("SELECT code FROM {}", kind.table()))?;
    let codes = statement.query_map([], |r| r.get::<_, String>(0))?;
    let (mut largest, mut width) = (0_u64, 1_usize);
    for code in codes {
        let code = code?;
        // `parse` would also take a leading `+`; the app never writes one.
        if code.bytes().all(|b| b.is_ascii_digit())
            && let Ok(number) = code.parse::<u64>()
        {
            largest = largest.max(number);
            width = width.max(code.len());
        }
    }
    let Some(next) = largest.checked_add(1) else {
        return Ok(None);
    };
    let text = format!("{next:0width$}");
    Ok((text.len() <= mandi_core::masters::MAX_CODE_DIGITS).then_some(text))
}

/// Adds a master. `fields` are the columns of `Prepared::fields`; `code` and `account_id` are
/// added when the kind has them. Returns the new id.
///
/// # Errors
/// Any SQLite error (a UNIQUE clash on the code, a missing foreign key).
#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    kind: MasterKind,
    code: Option<&str>,
    account_id: Option<i64>,
    fields: &[(&'static str, ColValue)],
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    let mut columns: Vec<&str> = vec!["created_at", "created_by", "updated_at", "updated_by"];
    let mut values: Vec<Value> = vec![
        Value::Text(now.to_owned()),
        Value::Integer(user_id),
        Value::Text(now.to_owned()),
        Value::Integer(user_id),
    ];
    if let Some(code) = code {
        columns.push("code");
        values.push(Value::Text(code.to_owned()));
    }
    if let Some(account_id) = account_id {
        columns.push("account_id");
        values.push(Value::Integer(account_id));
    }
    for (name, v) in fields {
        columns.push(name);
        values.push(value(v));
    }
    let marks: Vec<String> = (1..=columns.len()).map(|n| format!("?{n}")).collect();
    conn.execute(
        &format!(
            "INSERT INTO {} ({}) VALUES ({})",
            kind.table(),
            columns.join(", "),
            marks.join(", ")
        ),
        params_from_iter(values),
    )?;
    Ok(conn.last_insert_rowid())
}

/// Edits a master whose version is `version`; the version goes up by one. Returns false if no
/// row has this id and version.
///
/// # Errors
/// Any SQLite error.
#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    kind: MasterKind,
    id: i64,
    version: i64,
    code: Option<&str>,
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
    if let Some(code) = code {
        push("code", Value::Text(code.to_owned()));
    }
    for (name, v) in fields {
        push(name, value(v));
    }
    push("updated_at", Value::Text(now.to_owned()));
    push("updated_by", Value::Integer(user_id));
    push("version", Value::Integer(version.saturating_add(1)));
    values.push(Value::Integer(id));
    values.push(Value::Integer(version));
    let id_mark = values.len().saturating_sub(1);
    let version_mark = values.len();
    let changed = conn.execute(
        &format!(
            "UPDATE {} SET {} WHERE id = ?{id_mark} AND version = ?{version_mark}",
            kind.table(),
            sets.join(", ")
        ),
        params_from_iter(values),
    )?;
    Ok(changed == 1)
}

/// Sets the active flag and raises the version. Returns false if there is no such row.
///
/// # Errors
/// Any SQLite error.
pub fn set_active(
    conn: &Connection,
    kind: MasterKind,
    id: i64,
    is_active: bool,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        &format!(
            "UPDATE {} SET is_active = ?1, updated_at = ?2, updated_by = ?3, version = version + 1 WHERE id = ?4",
            kind.table()
        ),
        params![is_active, now, user_id, id],
    )?;
    Ok(changed == 1)
}

/// The next free serial under a control: the largest normal serial (1 to 8999) plus one. The
/// reserved serials 0000 and 9000 to 9999 are ignored (PRD FR-10.3). `None` if the series is full.
///
/// # Errors
/// Any SQLite error, or a control that does not exist.
pub fn next_serial(conn: &Connection, control_code: &str) -> rusqlite::Result<Option<u32>> {
    // A control that does not exist is an error, not an empty series.
    conn.query_row(
        "SELECT id FROM account_controls WHERE code = ?1",
        [control_code],
        |r| r.get::<_, i64>(0),
    )?;
    let largest: i64 = conn.query_row(
        "SELECT COALESCE(MAX(a.serial), 0) FROM accounts a JOIN account_controls c ON c.id = a.control_id
         WHERE c.code = ?1 AND a.serial BETWEEN 1 AND 8999",
        [control_code],
        |r| r.get(0),
    )?;
    let next = largest.saturating_add(1);
    Ok(if next <= 8999 {
        u32::try_from(next).ok()
    } else {
        None
    })
}

/// Makes the account of an expense head or a bank account under its control, with the next free
/// serial. This is the one function that plan 07's account service wraps. Returns the account
/// id and its code (the control code plus the 4-digit serial). `None` if the series is full.
///
/// # Errors
/// Any SQLite error, or a control that does not exist.
pub fn create_account(
    conn: &Connection,
    account: &AccountToMake,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<Option<(i64, String)>> {
    let Some(serial) = next_serial(conn, account.control_code)? else {
        return Ok(None);
    };
    let code = format!("{}{serial:04}", account.control_code);
    let id = create_account_coded(conn, account, &code, serial, user_id, now)?;
    Ok(Some((id, code)))
}

/// Makes an account under its control with the code and the serial given. A garden's account has the
/// code of its garden (`0080`, plan 34), which is not the control code plus a serial. Returns the id.
///
/// # Errors
/// Any SQLite error (a UNIQUE clash on the code), or a control that does not exist.
pub fn create_account_coded(
    conn: &Connection,
    account: &AccountToMake,
    code: &str,
    serial: u32,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    let made = conn.execute(
        "INSERT INTO accounts (created_at, created_by, updated_at, updated_by, control_id, code, serial,
             party_type, name_ur, name_en, search_ur, search_en, search_phonetic, is_system, is_active)
         SELECT ?1, ?2, ?1, ?2, c.id, ?3, ?4, c.party_type, ?5, ?6, ?7, ?8, ?9, 0, 1
         FROM account_controls c WHERE c.code = ?10",
        params![
            now,
            user_id,
            code,
            serial,
            account.name_ur,
            account.name_en,
            account.search_ur,
            account.search_en,
            account.search_phonetic,
            account.control_code
        ],
    )?;
    // Exactly one row, or `last_insert_rowid` would give the id of some earlier insert.
    if made != 1 {
        return Err(rusqlite::Error::StatementChangedRows(made));
    }
    Ok(conn.last_insert_rowid())
}

/// Gives the account of a master its new names (and search columns).
///
/// # Errors
/// Any SQLite error.
pub fn rename_account(
    conn: &Connection,
    account_id: i64,
    account: &AccountToMake,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET name_ur = ?1, name_en = ?2, search_ur = ?3, search_en = ?4, search_phonetic = ?5,
             updated_at = ?6, updated_by = ?7, version = version + 1 WHERE id = ?8",
        params![
            account.name_ur,
            account.name_en,
            account.search_ur,
            account.search_en,
            account.search_phonetic,
            now,
            user_id,
            account_id
        ],
    )?;
    Ok(())
}

/// Makes the account of a master active or inactive with it.
///
/// # Errors
/// Any SQLite error.
pub fn set_account_active(
    conn: &Connection,
    account_id: i64,
    is_active: bool,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET is_active = ?1, updated_at = ?2, updated_by = ?3, version = version + 1 WHERE id = ?4",
        params![is_active, now, user_id, account_id],
    )?;
    Ok(())
}

/// The account a master points at, if the kind has one.
///
/// # Errors
/// Any SQLite error.
pub fn account_of(conn: &Connection, kind: MasterKind, id: i64) -> rusqlite::Result<Option<i64>> {
    let sql = match kind {
        MasterKind::ExpenseHead => "SELECT account_id FROM expense_heads WHERE id = ?1",
        _ => return Ok(None),
    };
    conn.query_row(sql, [id], |r| r.get(0)).optional()
}

fn source_from(r: &Row<'_>) -> rusqlite::Result<SearchSource> {
    Ok(SearchSource {
        id: r.get(0)?,
        code: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
        name_ur: r.get(2)?,
        name_en: r.get(3)?,
        extra_ur: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
        extra_en: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
        more_keys: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
        search_ur: r.get(7)?,
        search_en: r.get(8)?,
        search_phonetic: r.get(9)?,
        is_active: r.get(10)?,
        account_type: None,
        control_id: None,
        control_code: None,
        tag: None,
    })
}

/// The records of one kind as the search index sees them: the columns are code, both names, the
/// two second lines, extra keys, the three search columns and the active flag. `only_id` limits
/// it to one record (after a save).
///
/// # Errors
/// Any SQLite error.
pub fn search_sources(
    conn: &Connection,
    kind: MasterKind,
    only_id: Option<i64>,
) -> rusqlite::Result<Vec<SearchSource>> {
    let select = match kind {
        MasterKind::City => {
            "SELECT m.id, m.code, m.name_ur, m.name_en, NULL, NULL, NULL, m.search_ur, m.search_en, m.search_phonetic, m.is_active FROM cities m"
        }
        MasterKind::Store => {
            "SELECT m.id, m.code, m.name_ur, m.name_en, NULL, NULL, NULL, m.search_ur, m.search_en, m.search_phonetic, m.is_active FROM stores m"
        }
        MasterKind::Item => {
            "SELECT m.id, m.code, m.name_ur, m.name_en, p.name_ur, p.name_en, NULL, m.search_ur, m.search_en, m.search_phonetic, m.is_active
             FROM items m LEFT JOIN packings p ON p.id = m.default_packing_id"
        }
        MasterKind::Packing => {
            "SELECT m.id, m.code, m.name_ur, m.name_en, NULL, NULL, NULL, m.search_ur, m.search_en, m.search_phonetic, m.is_active FROM packings m"
        }
        MasterKind::Adda => {
            "SELECT m.id, m.code, m.name_ur, m.name_en, c.name_ur, c.name_en, NULL, m.search_ur, m.search_en, m.search_phonetic, m.is_active
             FROM addas m LEFT JOIN cities c ON c.id = m.city_id"
        }
        MasterKind::Marka => {
            "SELECT m.id, m.code, m.mark_text, m.mark_text, a.name_ur, a.name_en, NULL, NULL, NULL, NULL, m.is_active
             FROM markas m LEFT JOIN accounts a ON a.id = m.owner_account_id"
        }
        MasterKind::Narration => {
            "SELECT m.id, NULL, m.text_ur, m.text_en, NULL, NULL, NULL, NULL, NULL, NULL, m.is_active FROM narrations m"
        }
        MasterKind::ExpenseHead => {
            "SELECT m.id, m.code, m.name_ur, m.name_en, a.code, a.code, NULL, m.search_ur, m.search_en, m.search_phonetic, m.is_active
             FROM expense_heads m JOIN accounts a ON a.id = m.account_id"
        }
        MasterKind::Staff => {
            "SELECT m.id, m.code, m.name_ur, m.name_en, m.phone, m.phone, NULL, m.search_ur, m.search_en, m.search_phonetic, m.is_active FROM staff m"
        }
    };
    let sql = if only_id.is_some() {
        format!("{select} WHERE m.id = ?1")
    } else {
        format!("{select} WHERE ?1 IS NULL")
    };
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map([only_id], source_from)?;
    rows.collect()
}
