//! View K, fruit taken, پھل (PRD 18.2 K, 16.7, FR-16.10 to FR-16.12, FR-18.2, FR-18.8, C16, C27;
//! plan d5-late, after D4R 17d).
//!
//! One row for each partner (or the shop's guests) and item: the quantity (نگ), the value, and the
//! value by where the fruit came from (our shop, another shop). The period, the item, a partner,
//! a garden (the fruit taken from its lots) and the kind of goods narrow it. The shop's guests are
//! one row for each item whatever the partners' parts are (C27); each partner's own part of the
//! guests' fruit is in his column of view F.
//!
//! Fruit writes no sale facts and no share rows, so this view reads the saved fruit entries
//! (`fruit_takes`) and their stored quantity and value, the same rows as the fruit register R37.
//! A cancelled entry is not here. The totals row adds the rows (checked), and the lines behind a
//! row are the entries of that row, which add up to it (FR-18.2).
//!
//! Example (E2E-42, 10-10-26): الف, کیلا, 2 نگ, 1,160 (our shop 1,160, another shop 0); the guests,
//! سیب, 3 نگ, 3,000 (our shop 0, another shop 3,000). Totals 5 نگ and 4,160.

use mandi_core::money::Paisa;
use mandi_core::text::UrduCollator;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::filters::Typed;
use super::ids;
use super::model::{
    AnalyticsGoods, AnalyticsKey, AnalyticsLinesInput, AnalyticsLinesPage, AnalyticsRow,
};
use super::table::{Table, col, count_of};
use crate::error::ReportsError;
use crate::reports::business::fruit::{LotKind, Scope, conds, read, totals_cells};
use crate::reports::common::{add, paisa};
use crate::reports::{Cell, ColumnKind, Names, PageSpec, RowKind};

/// Which lots the goods filter lets the fruit come from. Trip goods are refused before this
/// (`resolve`): they are not offered as fruit.
fn lots_of(goods: AnalyticsGoods) -> LotKind {
    match goods {
        AnalyticsGoods::All | AnalyticsGoods::Trip => LotKind::Any,
        AnalyticsGoods::Consigned => LotKind::Consigned,
        AnalyticsGoods::Garden => LotKind::Garden,
    }
}

/// The scope of the run: its period, partner, item, garden and goods.
fn scope_of(typed: &Typed) -> Scope<'_> {
    Scope {
        from: typed.from.as_deref(),
        to: typed.to.as_deref(),
        partner_id: typed.partner_id,
        guests_only: false,
        item_id: typed.item_id,
        garden_id: typed.garden_id,
        lots: lots_of(typed.goods),
    }
}

/// One group read from the database.
struct Group {
    partner: Option<(i64, Names)>,
    item_id: i64,
    item: Names,
    qty: i64,
    value: i64,
    own: i64,
    other: i64,
}

/// Builds view K.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does not
/// fit; `ReportsError::Data` if the two sources of a row do not add up to its value.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let c = conds(&scope_of(typed));
    let sql = format!(
        "SELECT ft.partner_id, p.name_ur, p.name_en, ft.item_id, i.name_ur, i.name_en,
                COALESCE(SUM(ft.qty), 0), COALESCE(SUM(ft.value_paisa), 0),
                COALESCE(SUM(CASE WHEN ft.source = 'shop' THEN ft.value_paisa END), 0),
                COALESCE(SUM(CASE WHEN ft.source = 'other_shop' THEN ft.value_paisa END), 0)
         FROM fruit_takes ft
         JOIN items i ON i.id = ft.item_id
         LEFT JOIN partners p ON p.id = ft.partner_id
         WHERE {} GROUP BY ft.partner_id, ft.item_id",
        c.sql()
    );
    let mut groups = conn
        .prepare(&sql)?
        .query_map(params_from_iter(c.args().iter()), |r| {
            let partner_id: Option<i64> = r.get(0)?;
            let partner_ur: Option<String> = r.get(1)?;
            let partner_en: Option<String> = r.get(2)?;
            Ok(Group {
                partner: partner_id
                    .zip(partner_ur.zip(partner_en))
                    .map(|(id, (ur, en))| (id, Names { ur, en })),
                item_id: r.get(3)?,
                item: Names {
                    ur: r.get(4)?,
                    en: r.get(5)?,
                },
                qty: r.get(6)?,
                value: r.get(7)?,
                own: r.get(8)?,
                other: r.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // The partners in the order they were made, the shop's guests last; an item by its Urdu name.
    let collator = UrduCollator::new().ok();
    let sort_key = |name: &str| -> Vec<u8> {
        collator
            .as_ref()
            .map_or_else(|| name.as_bytes().to_vec(), |c| c.sort_key(name))
    };
    groups.sort_by(|a, b| {
        let partner_order = |g: &Group| g.partner.as_ref().map_or(i64::MAX, |(id, _)| *id);
        partner_order(a)
            .cmp(&partner_order(b))
            .then_with(|| sort_key(&a.item.ur).cmp(&sort_key(&b.item.ur)))
            .then(a.item_id.cmp(&b.item_id))
    });

    let (mut qty_total, mut value_total, mut own_total, mut other_total) =
        (0_i64, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    let mut all_rows: Vec<AnalyticsRow> = Vec::with_capacity(groups.len());
    for group in groups {
        let (value, own, other) = (paisa(group.value)?, paisa(group.own)?, paisa(group.other)?);
        // The two sources are the only ones (a CHECK of the table), so they add up to the value.
        let mut sources = own;
        add(&mut sources, other)?;
        if sources != value {
            return Err(ReportsError::Data(
                "the sources of the fruit taken do not add up to its value".to_owned(),
            ));
        }
        qty_total = qty_total
            .checked_add(group.qty)
            .ok_or_else(|| ReportsError::Data("the fruit quantities do not fit".to_owned()))?;
        add(&mut value_total, value)?;
        add(&mut own_total, own)?;
        add(&mut other_total, other)?;
        let (for_whom, key_for) = match &group.partner {
            Some((id, names)) => (
                Cell::Names {
                    names: names.clone(),
                },
                (Some(*id), None),
            ),
            None => (
                Cell::Label {
                    id: ids::FRUIT_GUESTS.to_owned(),
                },
                (None, Some(true)),
            ),
        };
        all_rows.push(AnalyticsRow {
            cells: vec![
                for_whom,
                Cell::Names { names: group.item },
                Cell::Number { value: group.qty },
                Cell::Money { paisa: value },
                Cell::Money { paisa: own },
                Cell::Money { paisa: other },
            ],
            key: Some(AnalyticsKey {
                partner_id: key_for.0,
                guests: key_for.1,
                item_id: Some(group.item_id),
                ..AnalyticsKey::default()
            }),
            kind: RowKind::Normal,
        });
    }

    Ok(Table {
        columns: vec![
            col("for", ids::COL_FRUIT_FOR, ColumnKind::Names),
            col("item", ids::COL_ITEM, ColumnKind::Names),
            col("qty", ids::COL_QTY, ColumnKind::Number),
            col("value", ids::COL_VALUE, ColumnKind::Money),
            col("own_shop", ids::COL_FRUIT_OWN, ColumnKind::Money),
            col("other_shops", ids::COL_FRUIT_OTHER, ColumnKind::Money),
        ],
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Number { value: qty_total },
            Cell::Money { paisa: value_total },
            Cell::Money { paisa: own_total },
            Cell::Money { paisa: other_total },
        ],
    })
}

/// The lines behind a number of view K (PRD FR-18.2): the saved fruit entries of the row (or of the
/// whole filter), each opening its entry. They are the rows of the fruit register under the same
/// filters, so their sums are the number that was opened.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn lines(
    conn: &Connection,
    typed: &Typed,
    key: &AnalyticsKey,
    input: &AnalyticsLinesInput,
    spec: PageSpec,
) -> Result<AnalyticsLinesPage, ReportsError> {
    // `YYYY-MM-DD` sorts as a date: the later start and the earlier end.
    let from = [typed.from.as_ref(), key.from.as_ref()]
        .into_iter()
        .flatten()
        .max()
        .cloned();
    let to = [typed.to.as_ref(), key.to.as_ref()]
        .into_iter()
        .flatten()
        .min()
        .cloned();
    let guests_only = key.guests == Some(true);
    let partner_id = key.partner_id.or(typed.partner_id);
    // A partner of the filter and a row of the guests cannot meet: nothing is left, and the query
    // (`partner_id = ? AND partner_id IS NULL`) says so.
    let c = conds(&Scope {
        from: from.as_deref(),
        to: to.as_deref(),
        partner_id,
        guests_only,
        item_id: key.item_id.or(typed.item_id),
        garden_id: typed.garden_id,
        lots: lots_of(typed.goods),
    });
    let entries = read(conn, &c, spec)?;
    Ok(AnalyticsLinesPage {
        view: input.view,
        title_id: ids::LINES_TITLE.to_owned(),
        filters: typed.resolved.clone(),
        columns: crate::reports::business::fruit::columns(),
        totals: totals_cells(&entries),
        rows: entries.rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: entries.total_count,
        totals_label_id: ids::TOTAL.to_owned(),
    })
}
