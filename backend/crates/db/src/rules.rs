//! Queries for `charge_rules` (Architecture 15.5, plan 09). SQL only: the checks are in
//! `mandi_core::rules` and `mandi_services::rules`, and the choosing of the winning rule is in
//! `mandi_domain::rules`. A rule is never deleted; it is made inactive (Code Standards D3).

use mandi_core::accounts::RecordRef;
use mandi_core::money::{Bps, Paisa};
use mandi_core::rules::{
    ChargeMethod, ChargeSide, ChargeType, ItemGroup, RuleOverlap, RuleRow, RulesFilter, level_of,
    percent_text,
};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

/// The first day of the seeded defaults: early enough for any date the app accepts.
pub const DEFAULT_RULES_FROM: &str = "2000-01-01";
/// The settings key that names the three default rules.
pub const DEFAULTS_KEY: &str = "charge_defaults";

const ROW_SELECT: &str = "SELECT r.id, r.version, r.side, r.charge_type,
        r.account_id, a.code, a.name_ur, a.name_en,
        r.item_id, i.code, i.name_ur, i.name_en,
        r.item_group, r.method, r.value_bps, r.value_paisa, r.from_date, r.to_date, r.is_active, r.note
    FROM charge_rules r
    LEFT JOIN accounts a ON a.id = r.account_id
    LEFT JOIN items i ON i.id = r.item_id";

fn bad(column: usize, what: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        rusqlite::types::Type::Text,
        format!("charge_rules: {what}").into(),
    )
}

fn row_from(r: &Row<'_>) -> rusqlite::Result<RuleRow> {
    let side = ChargeSide::parse(&r.get::<_, String>(2)?).ok_or_else(|| bad(2, "side"))?;
    let charge_type =
        ChargeType::parse(&r.get::<_, String>(3)?).ok_or_else(|| bad(3, "charge_type"))?;
    let account_id: Option<i64> = r.get(4)?;
    let account = match account_id {
        Some(id) => Some(RecordRef {
            id,
            code: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
            name_ur: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
            name_en: r.get::<_, Option<String>>(7)?.unwrap_or_default(),
        }),
        None => None,
    };
    let item_id: Option<i64> = r.get(8)?;
    let item = match item_id {
        Some(id) => Some(RecordRef {
            id,
            code: r.get::<_, Option<String>>(9)?.unwrap_or_default(),
            name_ur: r.get::<_, Option<String>>(10)?.unwrap_or_default(),
            name_en: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
        }),
        None => None,
    };
    let item_group = match r.get::<_, Option<String>>(12)? {
        Some(text) => Some(ItemGroup::parse(&text).ok_or_else(|| bad(12, "item_group"))?),
        None => None,
    };
    let method = ChargeMethod::parse(&r.get::<_, String>(13)?).ok_or_else(|| bad(13, "method"))?;
    let value_bps: Option<i32> = r.get(14)?;
    let value_paisa: Option<i64> = r.get(15)?;
    let (value_paisa, value_text) = match (method, value_bps, value_paisa) {
        (ChargeMethod::Percent, Some(bps), None) => (
            None,
            percent_text(Bps::new(bps).map_err(|_| bad(14, "value_bps"))?),
        ),
        (ChargeMethod::PerUnit | ChargeMethod::PerLot, None, Some(p)) => {
            let paisa = Paisa::try_from_paisa(p).map_err(|_| bad(15, "value_paisa"))?;
            (Some(paisa), paisa.rupees().to_string())
        }
        _ => return Err(bad(13, "method and value do not match")),
    };
    Ok(RuleRow {
        id: r.get(0)?,
        version: r.get(1)?,
        side,
        charge_type,
        account,
        item,
        item_group,
        level: level_of(account_id, item_id, item_group)
            .ok_or_else(|| bad(12, "a group together with a party"))?,
        is_default: false,
        method,
        value_bps,
        value_paisa,
        value_text,
        from_date: r.get(16)?,
        to_date: r.get(17)?,
        is_active: r.get(18)?,
        note: r.get::<_, Option<String>>(19)?.unwrap_or_default(),
    })
}

/// One rule. `is_default` is left false: the services know which rows are the defaults.
///
/// # Errors
/// Any SQLite error, or a row whose columns do not fit together.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<RuleRow>> {
    conn.query_row(&format!("{ROW_SELECT} WHERE r.id = ?1"), [id], row_from)
        .optional()
}

/// The rules that match a filter: by side, charge, level (most specific first), then newest
/// first (latest start, then the biggest id).
///
/// # Errors
/// Any SQLite error.
pub fn list(conn: &Connection, filter: &RulesFilter) -> rusqlite::Result<Vec<RuleRow>> {
    let mut conditions: Vec<String> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    if !filter.include_inactive {
        conditions.push("r.is_active = 1".to_owned());
    }
    if let Some(side) = filter.side {
        values.push(Value::Text(side.as_str().to_owned()));
        conditions.push(format!("r.side = ?{}", values.len()));
    }
    if let Some(charge) = filter.charge_type {
        values.push(Value::Text(charge.as_str().to_owned()));
        conditions.push(format!("r.charge_type = ?{}", values.len()));
    }
    if let Some(id) = filter.account_id {
        values.push(Value::Integer(id));
        conditions.push(format!("r.account_id = ?{}", values.len()));
    }
    if let Some(id) = filter.item_id {
        values.push(Value::Integer(id));
        conditions.push(format!("r.item_id = ?{}", values.len()));
    }
    let wher = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "{ROW_SELECT}{wher} ORDER BY r.side, r.charge_type,
           CASE WHEN r.account_id IS NOT NULL AND r.item_id IS NOT NULL THEN 2
                WHEN r.account_id IS NOT NULL THEN 3
                WHEN r.item_id IS NOT NULL THEN 4
                WHEN r.item_group IS NOT NULL THEN 5 ELSE 6 END,
           r.from_date DESC, r.id DESC"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(values), row_from)?;
    rows.collect()
}

/// The columns of a rule that the user chooses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRule {
    /// Whose side.
    pub side: ChargeSide,
    /// Which charge.
    pub charge: ChargeType,
    /// The party, if any.
    pub account_id: Option<i64>,
    /// The item, if any.
    pub item_id: Option<i64>,
    /// The item group, if any.
    pub item_group: Option<ItemGroup>,
    /// How it is worked out.
    pub method: ChargeMethod,
    /// A percent in basis points, for method `percent`.
    pub value_bps: Option<i32>,
    /// The amount in paisa, for the other methods.
    pub value_paisa: Option<i64>,
    /// The first day, `YYYY-MM-DD`.
    pub from_date: String,
    /// The last day, `YYYY-MM-DD`, or none.
    pub to_date: Option<String>,
    /// Active.
    pub is_active: bool,
    /// The note, or none.
    pub note: Option<String>,
}

/// Adds a rule and returns its id.
///
/// # Errors
/// Any SQLite error (a CHECK that the checks above should have caught).
pub fn insert(conn: &Connection, rule: &NewRule, user_id: i64, now: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO charge_rules (created_at, created_by, updated_at, updated_by, side, charge_type,
             account_id, item_id, item_group, method, value_bps, value_paisa, from_date, to_date,
             is_active, note)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            now,
            user_id,
            rule.side.as_str(),
            rule.charge.as_str(),
            rule.account_id,
            rule.item_id,
            rule.item_group.map(ItemGroup::as_str),
            rule.method.as_str(),
            rule.value_bps,
            rule.value_paisa,
            rule.from_date,
            rule.to_date,
            rule.is_active,
            rule.note,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Edits a rule whose version is `version`; the version goes up by one. Returns false if no
/// row has this id and version.
///
/// # Errors
/// Any SQLite error.
pub fn update(
    conn: &Connection,
    id: i64,
    version: i64,
    rule: &NewRule,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE charge_rules SET updated_at = ?1, updated_by = ?2, version = version + 1,
             side = ?3, charge_type = ?4, account_id = ?5, item_id = ?6, item_group = ?7, method = ?8,
             value_bps = ?9, value_paisa = ?10, from_date = ?11, to_date = ?12, is_active = ?13, note = ?14
         WHERE id = ?15 AND version = ?16",
        params![
            now,
            user_id,
            rule.side.as_str(),
            rule.charge.as_str(),
            rule.account_id,
            rule.item_id,
            rule.item_group.map(ItemGroup::as_str),
            rule.method.as_str(),
            rule.value_bps,
            rule.value_paisa,
            rule.from_date,
            rule.to_date,
            rule.is_active,
            rule.note,
            id,
            version,
        ],
    )?;
    Ok(changed == 1)
}

/// Switches a rule on or off. Returns false if there is no such rule.
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
        "UPDATE charge_rules SET is_active = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![id, is_active, now, user_id],
    )?;
    Ok(changed == 1)
}

/// Other active rules with exactly the same scope (side, charge, party, item and group) whose
/// dates meet `from` to `to` (PRD FR-11.5). Dates are `YYYY-MM-DD`; `to` of `None` means no end.
/// The rule `except_id` is not compared with itself.
///
/// # Errors
/// Any SQLite error.
pub fn overlaps(
    conn: &Connection,
    rule: &NewRule,
    except_id: Option<i64>,
) -> rusqlite::Result<Vec<RuleOverlap>> {
    let mut statement = conn.prepare(
        "SELECT id, from_date, to_date FROM charge_rules
         WHERE is_active = 1 AND side = ?1 AND charge_type = ?2
           AND account_id IS ?3 AND item_id IS ?4 AND item_group IS ?5
           AND (?6 IS NULL OR id <> ?6)
           AND from_date <= COALESCE(?8, '9999-12-31')
           AND COALESCE(to_date, '9999-12-31') >= ?7
         ORDER BY from_date DESC, id DESC",
    )?;
    let rows = statement.query_map(
        params![
            rule.side.as_str(),
            rule.charge.as_str(),
            rule.account_id,
            rule.item_id,
            rule.item_group.map(ItemGroup::as_str),
            except_id,
            rule.from_date,
            rule.to_date,
        ],
        |r| {
            Ok(RuleOverlap {
                id: r.get(0)?,
                from_date: r.get(1)?,
                to_date: r.get(2)?,
            })
        },
    )?;
    rows.collect()
}

/// One active rule as resolution needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRecord {
    /// Row id.
    pub id: i64,
    /// Whose side.
    pub side: ChargeSide,
    /// Which charge.
    pub charge: ChargeType,
    /// The party, if any.
    pub account_id: Option<i64>,
    /// The item, if any.
    pub item_id: Option<i64>,
    /// The item group, if any.
    pub item_group: Option<ItemGroup>,
    /// How it is worked out.
    pub method: ChargeMethod,
    /// A percent in basis points.
    pub value_bps: Option<i32>,
    /// An amount in paisa.
    pub value_paisa: Option<i64>,
    /// The first day, `YYYY-MM-DD`.
    pub from_date: String,
    /// The last day, or none.
    pub to_date: Option<String>,
}

/// Every active rule (in one query, B22).
///
/// # Errors
/// Any SQLite error, or a row whose text is not one of the known words.
pub fn load_active(conn: &Connection) -> rusqlite::Result<Vec<RuleRecord>> {
    let mut statement = conn.prepare(
        "SELECT id, side, charge_type, account_id, item_id, item_group, method, value_bps,
                value_paisa, from_date, to_date
         FROM charge_rules WHERE is_active = 1 ORDER BY id",
    )?;
    let rows = statement.query_map([], |r| {
        Ok(RuleRecord {
            id: r.get(0)?,
            side: ChargeSide::parse(&r.get::<_, String>(1)?).ok_or_else(|| bad(1, "side"))?,
            charge: ChargeType::parse(&r.get::<_, String>(2)?).ok_or_else(|| bad(2, "charge"))?,
            account_id: r.get(3)?,
            item_id: r.get(4)?,
            item_group: match r.get::<_, Option<String>>(5)? {
                Some(text) => Some(ItemGroup::parse(&text).ok_or_else(|| bad(5, "item_group"))?),
                None => None,
            },
            method: ChargeMethod::parse(&r.get::<_, String>(6)?).ok_or_else(|| bad(6, "method"))?,
            value_bps: r.get(7)?,
            value_paisa: r.get(8)?,
            from_date: r.get(9)?,
            to_date: r.get(10)?,
        })
    })?;
    rows.collect()
}

/// The item group of an item (`fruit` or `vegetable`), if the item exists.
///
/// # Errors
/// Any SQLite error.
pub fn item_group_of(conn: &Connection, item_id: i64) -> rusqlite::Result<Option<ItemGroup>> {
    let text: Option<String> = conn
        .query_row(
            "SELECT item_group FROM items WHERE id = ?1",
            [item_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(text.and_then(|t| ItemGroup::parse(&t)))
}

/// The ids of the three default rules, from the settings: seller commission, buyer commission,
/// لاگا on fruit.
///
/// # Errors
/// Any SQLite error.
pub fn default_ids(conn: &Connection) -> rusqlite::Result<Option<(i64, i64, i64)>> {
    let text: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [DEFAULTS_KEY],
            |r| r.get(0),
        )
        .optional()?;
    Ok(text.and_then(|text| {
        let value: serde_json::Value = serde_json::from_str(&text).ok()?;
        Some((
            value.get("seller_commission")?.as_i64()?,
            value.get("buyer_commission")?.as_i64()?,
            value.get("laga_fruit")?.as_i64()?,
        ))
    }))
}
