//! Sums of the sale facts (`analytics_facts`, Architecture 15.6), the one source of every sales
//! figure in analytics (PRD FR-18.6, FR-18.7, money agent Section 9).
//!
//! A fact is a stored sale line value with a sign: +1 when the line was saved and −1 on the row a
//! reversal writes. A reversal row copies the date and every dimension of the row it reverses
//! (`mandi_db::bikri::edit`), so inside any group the −1 rows can never be more than the +1 rows.
//!
//! Each column is summed twice in SQL, once over the +1 rows and once over the −1 rows, with
//! `SUM()` on the stored `INTEGER` columns and no other arithmetic in SQL. The net is worked out
//! in Rust with checked maths. A net below 0 is a bug signal and an error, never shown or hidden.
//! A group whose net quantity and amounts are all 0 (a cancelled or fully edited-away sale) has
//! no row (FR-18.7).
//!
//! Example (PRD 18.4, view A, کیلا): +1 rows qty 50, seller amount 29,400, seller commission
//! 1,764, buyer commission 1,764, لاگا 400; no −1 rows. The net is the same; the total
//! commission is 1,764 + 1,764 + 400 = 3,928.

use std::collections::HashMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::filters::Typed;
use super::model::{AnalyticsGoods, AnalyticsKey, AnalyticsSide};
use crate::error::ReportsError;
use crate::reports::common::{Conds, add, paisa};

/// The summed columns, in query order.
const COLUMNS: [&str; 6] = [
    "qty",
    "seller_amount_paisa",
    "seller_comm_paisa",
    "buyer_value_paisa",
    "buyer_comm_paisa",
    "laga_paisa",
];

/// The net figures of one group: the +1 rows less the −1 rows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Sums {
    /// نگ.
    pub(crate) qty: i64,
    /// خام بکری: the seller amounts.
    pub(crate) gross: Paisa,
    /// The seller commission.
    pub(crate) seller_comm: Paisa,
    /// The value the buyers were charged before commission and لاگا.
    pub(crate) buyer_value: Paisa,
    /// The buyer commission.
    pub(crate) buyer_comm: Paisa,
    /// لاگا.
    pub(crate) laga: Paisa,
}

fn net(plus: i64, minus: i64, what: &str) -> Result<i64, ReportsError> {
    match plus.checked_sub(minus) {
        Some(value) if value >= 0 => Ok(value),
        _ => Err(ReportsError::Data(format!(
            "the net of {what} in a fact group is below 0"
        ))),
    }
}

impl Sums {
    /// The net of the twelve sums of a group (six columns, +1 rows then −1 rows).
    fn from_raw(raw: &[i64; 12]) -> Result<Self, ReportsError> {
        let at = |index: usize| raw.get(index).copied().unwrap_or(0);
        let pair = |index: usize| -> Result<i64, ReportsError> {
            net(
                at(index),
                at(index.saturating_add(6)),
                COLUMNS.get(index).copied().unwrap_or("a column"),
            )
        };
        Ok(Self {
            qty: pair(0)?,
            gross: paisa(pair(1)?)?,
            seller_comm: paisa(pair(2)?)?,
            buyer_value: paisa(pair(3)?)?,
            buyer_comm: paisa(pair(4)?)?,
            laga: paisa(pair(5)?)?,
        })
    }

    /// True when nothing is left of the group (FR-18.7).
    pub(crate) fn is_zero(&self) -> bool {
        self.qty == 0
            && self.gross == Paisa::ZERO
            && self.seller_comm == Paisa::ZERO
            && self.buyer_value == Paisa::ZERO
            && self.buyer_comm == Paisa::ZERO
            && self.laga == Paisa::ZERO
    }

    /// Seller commission plus buyer commission plus لاگا ("total comm", PRD 18.2), or the part
    /// of it on the side asked for.
    pub(crate) fn total_comm(&self, side: AnalyticsSide) -> Result<Paisa, ReportsError> {
        let mut total = Paisa::ZERO;
        match side {
            AnalyticsSide::Both => {
                add(&mut total, self.seller_comm)?;
                add(&mut total, self.buyer_comm)?;
                add(&mut total, self.laga)?;
            }
            AnalyticsSide::Seller => add(&mut total, self.seller_comm)?,
            AnalyticsSide::Buyer => {
                add(&mut total, self.buyer_comm)?;
                add(&mut total, self.laga)?;
            }
        }
        Ok(total)
    }

    /// Adds another group to this one, with checked maths.
    pub(crate) fn add(&mut self, other: &Sums) -> Result<(), ReportsError> {
        self.qty = self
            .qty
            .checked_add(other.qty)
            .ok_or_else(|| ReportsError::Data("the quantity total does not fit".to_owned()))?;
        add(&mut self.gross, other.gross)?;
        add(&mut self.seller_comm, other.seller_comm)?;
        add(&mut self.buyer_value, other.buyer_value)?;
        add(&mut self.buyer_comm, other.buyer_comm)?;
        add(&mut self.laga, other.laga)?;
        Ok(())
    }
}

/// One group of a query: the values of its dimensions and its net figures.
#[derive(Debug, Clone)]
pub(crate) struct Group {
    /// The dimension values, in the order asked for.
    pub(crate) dims: Vec<Value>,
    /// The net figures.
    pub(crate) sums: Sums,
}

impl Group {
    /// An integer dimension.
    pub(crate) fn int(&self, index: usize) -> Option<i64> {
        match self.dims.get(index) {
            Some(Value::Integer(value)) => Some(*value),
            _ => None,
        }
    }

    /// A text dimension.
    pub(crate) fn text(&self, index: usize) -> Option<String> {
        match self.dims.get(index) {
            Some(Value::Text(value)) => Some(value.clone()),
            _ => None,
        }
    }
}

/// The condition of a goods filter on the facts, with the table named `f` (PRD 18.1 "goods"), or
/// none for all goods. The commission page uses the same text (plan 78 D4), so a goods filter means
/// the same thing in both places.
pub(crate) const fn goods_condition(goods: AnalyticsGoods) -> Option<&'static str> {
    match goods {
        AnalyticsGoods::All => None,
        // Consigned goods have no venture and are not a garden's (PRD 18.1).
        AnalyticsGoods::Consigned => Some(
            "f.venture_id IS NULL AND f.seller_account_id NOT IN
             (SELECT account_id FROM ventures WHERE kind = 'garden' AND account_id IS NOT NULL)",
        ),
        AnalyticsGoods::Trip => {
            Some("f.venture_id IN (SELECT id FROM ventures WHERE kind = 'trip')")
        }
        AnalyticsGoods::Garden => Some(
            "(f.venture_id IN (SELECT id FROM ventures WHERE kind = 'garden')
              OR f.seller_account_id IN
                 (SELECT account_id FROM ventures WHERE kind = 'garden' AND account_id IS NOT NULL))",
        ),
    }
}

/// The conditions on the facts for a run: the date range given (the run's or the compared one),
/// the picked records, and the extra narrowing of a drill-down key.
pub(crate) fn conds(
    typed: &Typed,
    range: (Option<&str>, Option<&str>),
    key: &AnalyticsKey,
) -> Conds {
    let mut c = dated(range);
    c.text("f.entry_date >= ?", key.from.as_deref())
        .text("f.entry_date <= ?", key.to.as_deref())
        .id("f.item_id = ?", typed.item_id)
        // All the items of a group (PRD v1.29; views A, B, C, D, G and H, and the lines behind them).
        .item_group("f.item_id", typed.item_group)
        // The quality of the line (PRD FR-12.42): a sale line never changes its quality, so the
        // reversal rows of an edit carry the quality of the line they cancel.
        .text(
            "f.sale_id IN (SELECT id FROM bikri_sales WHERE grade = ?)",
            typed.grade.map(|g| g.as_str()),
        )
        .id("f.item_id = ?", key.item_id)
        .not_in("f.item_id", &key.not_item_ids)
        .id("f.seller_account_id = ?", typed.seller_id)
        .id("f.seller_account_id = ?", key.seller_id)
        .id("f.buyer_account_id = ?", typed.buyer_id)
        .id("f.buyer_account_id = ?", key.buyer_id)
        .text("f.chukta_person_key = ?", typed.person_key.as_deref())
        .text("f.chukta_person_key = ?", key.person_key.as_deref())
        .id("f.city_id = ?", typed.city_id)
        .id("f.adda_id = ?", typed.adda_id)
        .id("f.venture_id = ?", typed.trip_id)
        // The goods of one garden: the lots whose seller is its account (FR-12.36).
        .id("f.seller_account_id = ?", typed.garden_account_id)
        .in_list("f.purchase_id", &key.purchase_ids);
    if let Some(sql) = goods_condition(typed.goods) {
        c.always(sql);
    }
    c
}

/// The date range view J reads its commission in: none. J picks a purchase by the day it was
/// bought, and its sales can fall on any later day (R33 reads them the same way).
pub(crate) const TRIPS_RANGE: (Option<&str>, Option<&str>) = (None, None);

/// The filters view J reads the sale facts of its purchases under, and the lines behind a row of
/// J open them under: the run's filters without the item. The item picks the purchases; the
/// commission of a purchase is of all its sales (money agent M1, M2).
pub(crate) fn for_trips(typed: &Typed) -> Typed {
    Typed {
        item_id: None,
        ..typed.clone()
    }
}

/// The conditions of a date range alone, found through the date index of the facts (Code
/// Standards B18).
pub(crate) fn dated(range: (Option<&str>, Option<&str>)) -> Conds {
    let mut c = Conds::new();
    c.text("f.entry_date >= ?", range.0)
        .text("f.entry_date <= ?", range.1);
    c
}

/// The date range of the run itself.
pub(crate) fn run_range(typed: &Typed) -> (Option<&str>, Option<&str>) {
    (typed.from.as_deref(), typed.to.as_deref())
}

/// The date range of the compared period.
pub(crate) fn compare_range(typed: &Typed) -> (Option<&str>, Option<&str>) {
    typed
        .compare
        .as_ref()
        .map_or((None, None), |(from, to)| (Some(from), Some(to)))
}

/// The query of [`groups`]: the dimension columns, then the +1 sums and the −1 sums of the six
/// columns, grouped by the dimensions.
pub(crate) fn groups_sql(conds: &Conds, dims: &[&'static str]) -> String {
    let dimension_list = dims.join(", ");
    let mut sums_sql = String::new();
    for sign in ["1", "-1"] {
        for column in COLUMNS {
            sums_sql.push_str(&format!(
                ", COALESCE(SUM(CASE WHEN f.sign = {sign} THEN f.{column} END), 0)"
            ));
        }
    }
    let group_by = if dims.is_empty() {
        String::new()
    } else {
        format!("GROUP BY {dimension_list}")
    };
    let select = if dims.is_empty() {
        sums_sql.trim_start_matches(", ").to_owned()
    } else {
        format!("{dimension_list}{sums_sql}")
    };
    format!(
        "SELECT {select} FROM analytics_facts f WHERE {} {group_by}",
        conds.sql()
    )
}

/// A fast hasher for the group keys: they are ids and short texts picked by our own SQL, never by
/// an attacker, so the default hasher (made to resist one) only costs time. A million rows are added
/// up one by one.
#[derive(Default)]
struct KeyHasher(u64);

impl std::hash::Hasher for KeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(*byte)).wrapping_mul(0x517c_c1b7_2722_0a95);
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(0x517c_c1b7_2722_0a95);
    }

    fn write_i64(&mut self, value: i64) {
        self.write_u64(u64::from_ne_bytes(value.to_ne_bytes()));
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(u64::try_from(value).unwrap_or(u64::MAX));
    }

    fn write_isize(&mut self, value: isize) {
        self.write_i64(i64::try_from(value).unwrap_or(i64::MAX));
    }
}

type KeyMap = HashMap<Vec<DimKey>, [i64; 12], std::hash::BuildHasherDefault<KeyHasher>>;

/// One value of a dimension column in a group key: NULL, a number or a text, ordered as SQLite orders
/// them in a `GROUP BY` (NULL first, then numbers, then text by its bytes), so the groups come out in
/// the order the SQL grouping gave them.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum DimKey {
    Null,
    Int(i64),
    Text(String),
}

impl DimKey {
    /// A dimension value as a key. The dimension columns hold integers, texts and NULL; any other
    /// value would make two groups one, so it is an error, never a silent merge.
    fn of(value: Value) -> Result<Self, ReportsError> {
        match value {
            Value::Null => Ok(Self::Null),
            Value::Integer(n) => Ok(Self::Int(n)),
            Value::Text(t) => Ok(Self::Text(t)),
            other => Err(ReportsError::Data(format!(
                "a fact dimension holds a value that is not a number or a text: {other:?}"
            ))),
        }
    }

    fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Int(n) => Value::Integer(n),
            Self::Text(t) => Value::Text(t),
        }
    }
}

/// The query of [`groups_multi`]: the dimension columns, the sign and the six summed columns of
/// every fact under `conds`, with no grouping and no sums in SQL.
pub(crate) fn rows_sql(conds: &Conds, dims: &[&'static str]) -> String {
    let mut select: Vec<String> = dims.iter().map(|d| (*d).to_owned()).collect();
    select.push("f.sign".to_owned());
    select.extend(COLUMNS.iter().map(|c| format!("f.{c}")));
    format!(
        "SELECT {} FROM analytics_facts f WHERE {}",
        select.join(", "),
        conds.sql()
    )
}

fn overflow() -> ReportsError {
    ReportsError::Data("a fact total does not fit".to_owned())
}

/// The groups of the facts under `conds` for several sets of dimension columns at once: the facts
/// are read ONCE (from the covering index of the views) and added up for each set in memory, with
/// checked integer maths, instead of one `GROUP BY` pass for each set. A `GROUP BY` of 1.2 million
/// facts costs 3 to 6 seconds in SQLite (a temporary tree for the groups and twelve `CASE` sums for
/// every row); the same sums in memory cost under one (plan 80 gate, views C and H).
///
/// For each set the answer is exactly what [`groups_by_sql`] gives: one group for each combination
/// of the dimension values, the +1 rows less the −1 rows, a group with nothing left left out, in the
/// order of the dimension values. Only a row whose sign is 1 or −1 counts, as in the SQL.
///
/// # Errors
/// A database error; `ReportsError::Data` for a net below 0 or a sum that does not fit;
/// `ReportsError::Money`.
pub(crate) fn groups_multi(
    conn: &Connection,
    conds: &Conds,
    sets: &[&[&'static str]],
) -> Result<Vec<Vec<Group>>, ReportsError> {
    // The distinct dimension columns, in the order they are first asked for; each set picks its own.
    let mut all: Vec<&'static str> = Vec::new();
    for set in sets {
        for dim in *set {
            if !all.contains(dim) {
                all.push(dim);
            }
        }
    }
    let picks: Vec<Vec<usize>> = sets
        .iter()
        .map(|set| {
            set.iter()
                .filter_map(|dim| all.iter().position(|d| d == dim))
                .collect()
        })
        .collect();
    let sql = rows_sql(conds, &all);
    let mut statement = conn.prepare(&sql)?;
    let width = all.len();
    let mut maps: Vec<KeyMap> = sets.iter().map(|_| KeyMap::default()).collect();
    let mut rows = statement.query(params_from_iter(conds.args().iter()))?;
    while let Some(row) = rows.next()? {
        let sign: i64 = row.get(width)?;
        let offset = match sign {
            1 => 0_usize,
            -1 => 6_usize,
            _ => continue,
        };
        let mut values = [0_i64; 6];
        for (slot, index) in values.iter_mut().zip(width.saturating_add(1)..) {
            *slot = row.get::<_, Option<i64>>(index)?.unwrap_or(0);
        }
        let mut dim_keys: Vec<DimKey> = Vec::with_capacity(width);
        for index in 0..width {
            dim_keys.push(DimKey::of(row.get::<_, Value>(index)?)?);
        }
        for (map, pick) in maps.iter_mut().zip(&picks) {
            let key: Vec<DimKey> = pick
                .iter()
                .filter_map(|i| dim_keys.get(*i).cloned())
                .collect();
            let slot = map.entry(key).or_insert([0; 12]);
            for (i, value) in values.iter().enumerate() {
                let at = offset.saturating_add(i);
                let cell = slot.get_mut(at).ok_or_else(overflow)?;
                *cell = cell.checked_add(*value).ok_or_else(overflow)?;
            }
        }
    }
    let mut out = Vec::with_capacity(sets.len());
    for map in maps {
        let mut keyed: Vec<(Vec<DimKey>, [i64; 12])> = map.into_iter().collect();
        keyed.sort_by(|a, b| a.0.cmp(&b.0));
        let mut found = Vec::with_capacity(keyed.len());
        for (key, raw) in keyed {
            let sums = Sums::from_raw(&raw)?;
            if !sums.is_zero() {
                found.push(Group {
                    dims: key.into_iter().map(DimKey::into_value).collect(),
                    sums,
                });
            }
        }
        out.push(found);
    }
    Ok(out)
}

/// The groups of the facts under `conds`, one for each combination of the dimension columns
/// (constants of this crate such as `f.item_id`). Groups with nothing left are left out. Read by
/// [`groups_multi`], once, and added up in memory.
///
/// # Errors
/// A database error; `ReportsError::Data` for a net below 0; `ReportsError::Money`.
pub(crate) fn groups(
    conn: &Connection,
    conds: &Conds,
    dims: &[&'static str],
) -> Result<Vec<Group>, ReportsError> {
    // Grouped by the day, the month or the year the covering index already gives the facts in the
    // order of the groups, so SQLite adds them as they come, with no tree for the groups: that is
    // cheaper than adding them here. Any other grouping needs the tree, and is added up here.
    if let [dim] = dims
        && dim.contains("entry_date")
    {
        return groups_by_sql(conn, conds, dims);
    }
    Ok(groups_multi(conn, conds, &[dims])?
        .into_iter()
        .next()
        .unwrap_or_default())
}

/// [`groups`] by one SQL `GROUP BY` with the twelve sums in SQL, the way every view read the facts
/// before plan 80. Still used for a grouping by day, month or year (see [`groups`]) and by the test
/// that holds [`groups`] to the rows of this query.
///
/// # Errors
/// As [`groups`].
pub(crate) fn groups_by_sql(
    conn: &Connection,
    conds: &Conds,
    dims: &[&'static str],
) -> Result<Vec<Group>, ReportsError> {
    let sql = groups_sql(conds, dims);
    let mut statement = conn.prepare(&sql)?;
    let width = dims.len();
    let raw = statement
        .query_map(params_from_iter(conds.args().iter()), |r| {
            let mut dim_values = Vec::with_capacity(width);
            for index in 0..width {
                dim_values.push(r.get::<_, Value>(index)?);
            }
            let mut sums = [0_i64; 12];
            for (slot, index) in sums.iter_mut().zip(width..) {
                *slot = r.get(index)?;
            }
            Ok((dim_values, sums))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut found = Vec::with_capacity(raw.len());
    for (dim_values, sums) in raw {
        let sums = Sums::from_raw(&sums)?;
        if !sums.is_zero() {
            found.push(Group {
                dims: dim_values,
                sums,
            });
        }
    }
    Ok(found)
}

/// The sums over all facts under `conds`, with no grouping.
///
/// # Errors
/// As [`groups`].
pub(crate) fn total(conn: &Connection, conds: &Conds) -> Result<Sums, ReportsError> {
    let mut all = groups(conn, conds, &[])?;
    Ok(all.pop().map_or_else(Sums::default, |g| g.sums))
}

#[cfg(test)]
mod tests {
    //! `groups` and `groups_multi` (one read of the facts, added up in memory) give exactly the rows
    //! of the SQL `GROUP BY` they replaced (plan 80 gate): the same groups, the same nets, the same
    //! order, the groups with nothing left left out, and the same refusal of a net below 0.

    use super::*;

    /// A table with the columns the views read, and a few thousand facts made from a fixed seed: sales
    /// with a quality of the dimensions that repeat, some with a person, and a reversal row (sign −1)
    /// for one sale in five that copies its dimensions and values, so some groups net to 0.
    fn facts_db(with_orphan_reversal: bool) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE analytics_facts (
                id INTEGER PRIMARY KEY, entry_date TEXT NOT NULL, sign INTEGER NOT NULL,
                item_id INTEGER NOT NULL, seller_account_id INTEGER NOT NULL,
                buyer_account_id INTEGER NOT NULL, chukta_person_key TEXT, purchase_id INTEGER,
                qty INTEGER NOT NULL, seller_amount_paisa INTEGER NOT NULL,
                seller_comm_paisa INTEGER NOT NULL, buyer_value_paisa INTEGER NOT NULL,
                buyer_comm_paisa INTEGER NOT NULL, laga_paisa INTEGER NOT NULL);",
        )
        .unwrap();
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = move |n: u64| -> i64 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            i64::try_from((seed >> 33) % n).unwrap_or(0)
        };
        let insert = |conn: &Connection, row: [Value; 13]| {
            conn.execute(
                "INSERT INTO analytics_facts (entry_date, sign, item_id, seller_account_id,
                     buyer_account_id, chukta_person_key, purchase_id, qty, seller_amount_paisa,
                     seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa, laga_paisa)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params_from_iter(row.iter()),
            )
            .unwrap();
        };
        for _ in 0..3_000 {
            let day = 1 + next(28);
            let month = 1 + next(12);
            let date = format!("2026-{month:02}-{day:02}");
            let person = (next(7) == 0).then(|| format!("person-{}", next(5)));
            let qty = 1 + next(50);
            let amount = (1 + next(900)) * 100;
            let comm = (next(60)) * 100;
            let row = |sign: i64, person: &Option<String>| -> [Value; 13] {
                [
                    Value::Text(date.clone()),
                    Value::Integer(sign),
                    Value::Integer(1 + qty % 6),
                    Value::Integer(1 + day % 5),
                    Value::Integer(10 + (day + month) % 9),
                    person.clone().map_or(Value::Null, Value::Text),
                    if qty % 4 == 0 {
                        Value::Integer(qty % 3)
                    } else {
                        Value::Null
                    },
                    Value::Integer(qty),
                    Value::Integer(amount),
                    Value::Integer(comm),
                    Value::Integer(amount + 100),
                    Value::Integer(comm),
                    Value::Integer(qty * 100),
                ]
            };
            insert(&conn, row(1, &person));
            if next(5) == 0 {
                insert(&conn, row(-1, &person));
            }
        }
        if with_orphan_reversal {
            // A −1 row with nothing to cancel: a net below 0.
            insert(
                &conn,
                [
                    Value::Text("2026-05-05".into()),
                    Value::Integer(-1),
                    Value::Integer(99),
                    Value::Integer(99),
                    Value::Integer(99),
                    Value::Null,
                    Value::Null,
                    Value::Integer(1),
                    Value::Integer(100),
                    Value::Integer(100),
                    Value::Integer(100),
                    Value::Integer(100),
                    Value::Integer(100),
                ],
            );
        }
        conn
    }

    fn as_text(groups: &[Group]) -> Vec<String> {
        groups.iter().map(|g| format!("{g:?}")).collect()
    }

    const SETS: [&[&str]; 6] = [
        &[],
        &["f.item_id"],
        &["f.seller_account_id"],
        &["f.item_id", "substr(f.entry_date, 1, 7)"],
        &["f.buyer_account_id", "f.chukta_person_key"],
        &["f.purchase_id"],
    ];

    #[test]
    fn the_groups_read_in_memory_are_the_groups_of_the_sql() {
        let conn = facts_db(false);
        for conds in [
            dated((None, None)),
            dated((Some("2026-03-01"), Some("2026-09-30"))),
            dated((Some("2026-12-30"), None)),
            dated((Some("2030-01-01"), None)),
        ] {
            for dims in SETS {
                let old = groups_by_sql(&conn, &conds, dims).unwrap();
                let new = groups(&conn, &conds, dims).unwrap();
                assert_eq!(as_text(&new), as_text(&old), "{dims:?} {:?}", conds.sql());
            }
            // All the sets at once give each set what it gives alone.
            let together = groups_multi(&conn, &conds, &SETS).unwrap();
            for (dims, got) in SETS.iter().zip(&together) {
                let old = groups_by_sql(&conn, &conds, dims).unwrap();
                assert_eq!(as_text(got), as_text(&old), "together {dims:?}");
            }
        }
    }

    #[test]
    fn some_group_nets_to_nothing_and_is_left_out_in_both() {
        let conn = facts_db(false);
        let conds = dated((None, None));
        let old = groups_by_sql(&conn, &conds, &["f.item_id", "f.seller_account_id"]).unwrap();
        let all_plus: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM (SELECT 1 FROM analytics_facts GROUP BY item_id, seller_account_id)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!old.is_empty());
        // The test book is made so that the reversals only take a part of a group; the empty groups
        // are the ones with every row reversed, and both reads leave them out alike.
        assert!(i64::try_from(old.len()).unwrap() <= all_plus);
    }

    #[test]
    fn a_net_below_nothing_is_the_same_error_in_both() {
        let conn = facts_db(true);
        let conds = dated((None, None));
        for dims in SETS {
            let old = groups_by_sql(&conn, &conds, dims).map(|g| as_text(&g));
            let new = groups(&conn, &conds, dims).map(|g| as_text(&g));
            match (&old, &new) {
                (Err(old), Err(new)) => {
                    assert_eq!(format!("{old:?}"), format!("{new:?}"), "{dims:?}");
                }
                // Over every fact together the orphan row is a small part of a big net: no error.
                (Ok(old), Ok(new)) => assert_eq!(old, new, "{dims:?}"),
                _ => panic!("{dims:?}: old {old:?}, new {new:?}"),
            }
        }
    }
}
