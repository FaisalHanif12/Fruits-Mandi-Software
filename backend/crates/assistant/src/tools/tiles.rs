//! The tool `tiles`: the dashboard tiles of PRD 9.1, read now (PRD FR-7.87, Code Standards Q3).
//!
//! It calls `mandi_reports::dashboard::tiles_now`, the dashboard's tiles without the ages of the
//! buyers' bills, the attention list, the partner strip, the lists and the charts (plan 48
//! P48-S1: the full `get` takes 7 seconds on the NFR-SCL-1 volume and each tile is the very tile
//! `get` gives). A quick chip reads only its own tile with [`run_one`]. Where a tile has a second
//! place (PRD FR-7.76) the figure is read there too, in the same moment, and the two travel
//! together:
//!
//! | Tile | Second place |
//! |---|---|
//! | `today_sales` | analytics view A, today, the gross |
//! | `today_commission` | analytics view A, today, the total commission |
//! | `cash_in_hand` | R12, today, the closing balance |
//! | `buyers_dues` | R20 as of today (without the ages of the bills), its total |
//! | `sellers_dues` | R24, the "بیوپاری کے ذمے" total |
//! | `dead_amount` | R38, the amount still owed |
//! | `total_dues` | each of its three parts checked as above |
//!
//! The other tiles have no second place and carry no check mark. `last_backup` is what the backup
//! service says (passed in by the app): whether the cloud copy is on and how many hours old it is.
//!
//! Example (worked day): `today_sales.amount` is 41,400 in the tile and in view A: checked.

use mandi_reports::analytics::{AnalyticsFilters, AnalyticsRunInput, AnalyticsView};
use mandi_reports::dashboard::{self as narrow, BackupFacts, DashboardTiles, DuesParts, tiles_now};
use mandi_reports::reports::{
    Cell, ReportFilters, ReportId, ReportRunInput, run as engine_run, run_dues_without_ages,
};
use mandi_services::core::money::Paisa;
use mandi_services::domain::figures::{CalcOp, Value};
use mandi_services::figures;
use serde_json::Value as Json;

use super::cells::signed_paisa;
use super::report::reports_stop;
use super::{Draft, Env, State, Stop, ToolError, input};
use crate::answer::AnswerSource;
use crate::results::{Money, Node, Val};

/// The tool takes no input.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {}

fn plain(paisa: Paisa) -> Node {
    Node::Val(Val::Money(Money::plain(paisa)))
}

fn count(n: i64) -> Node {
    Node::Val(Val::Count(n))
}

/// A figure with its two places: checked when they are equal.
fn checked(tile: Paisa, place: &'static str, other: Option<Paisa>) -> Node {
    match other {
        Some(other) => Node::Val(Val::Money(Money {
            paisa: tile,
            side: None,
            checked: Some(tile == other),
            places: vec![("tile", tile), (place, other)],
        })),
        None => plain(tile),
    }
}

fn internal(detail: &str) -> Stop {
    Stop::Error(ToolError::new("ERR_INTERNAL", "tiles", detail))
}

/// View A for today: (gross, total commission) from its totals row.
fn view_a_today(env: &Env<'_>, today: &str) -> Result<(Option<Paisa>, Option<Paisa>), Stop> {
    let page = mandi_reports::analytics::run(
        env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::A,
            filters: AnalyticsFilters {
                from: today.to_owned(),
                to: today.to_owned(),
                ..AnalyticsFilters::default()
            },
            offset: 0,
            limit: 1,
            sort: None,
        },
    )
    .map_err(reports_stop)?;
    let at = |key: &str| {
        let index = page.columns.iter().position(|c| c.key == key)?;
        match page.totals.get(index)? {
            Cell::Money { paisa } => Some(*paisa),
            _ => None,
        }
    };
    Ok((at("gross"), at("total_comm")))
}

/// The closing balance of the cash book today, signed (below zero is credit).
fn cash_closing(env: &Env<'_>, today: &str) -> Result<Option<Paisa>, Stop> {
    let page = engine_run(
        env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R12,
            filters: ReportFilters {
                from: today.to_owned(),
                to: today.to_owned(),
                ..ReportFilters::default()
            },
            offset: 0,
            limit: 1,
            sort: None,
            group_by_month: false,
        },
    )
    .map_err(reports_stop)?;
    let line = page
        .summary
        .iter()
        .find(|l| l.label_id == "report-closing-balance");
    Ok(line
        .and_then(|l| signed_paisa(&l.cell))
        .and_then(|p| Paisa::try_from_paisa(p).ok()))
}

/// The total of one column of a report page, whether the engine gives it as money or as a balance.
fn column_total(page: &mandi_reports::reports::ReportPage, key: &str) -> Option<Paisa> {
    let index = page.columns.iter().position(|c| c.key == key)?;
    match page.totals.get(index)? {
        Cell::Money { paisa } | Cell::Balance { paisa, .. } => Some(*paisa),
        _ => None,
    }
}

fn run_balance_report(
    env: &Env<'_>,
    id: ReportId,
    filters: ReportFilters,
) -> Result<mandi_reports::reports::ReportPage, Stop> {
    engine_run(
        env.ctx.db(),
        &ReportRunInput {
            id,
            filters,
            offset: 0,
            limit: 1,
            sort: None,
            group_by_month: false,
        },
    )
    .map_err(reports_stop)
}

/// The second places of the dues tiles: R20 total, the "بیوپاری کے ذمے" total of R24, and the total
/// of R38 (PRD FR-7.76).
struct Dues {
    buyers: Option<Paisa>,
    sellers: Option<Paisa>,
    dead: Option<Paisa>,
}

/// The R20 total as of today, read without the ages of the bills (they do not change a balance,
/// and cost 3.6 seconds on the NFR-SCL-1 volume).
fn place_buyers(env: &Env<'_>, today: &str) -> Result<Option<Paisa>, Stop> {
    let page = run_dues_without_ages(
        env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R20,
            filters: ReportFilters {
                as_of: today.to_owned(),
                ..ReportFilters::default()
            },
            offset: 0,
            limit: 1,
            sort: None,
            group_by_month: false,
        },
    )
    .map_err(reports_stop)?;
    Ok(column_total(&page, "balance"))
}

fn place_sellers(env: &Env<'_>, today: &str) -> Result<Option<Paisa>, Stop> {
    let r24 = run_balance_report(
        env,
        ReportId::R24,
        ReportFilters {
            as_of: today.to_owned(),
            ..ReportFilters::default()
        },
    )?;
    Ok(column_total(&r24, "owes_firm"))
}

fn place_dead(env: &Env<'_>) -> Result<Option<Paisa>, Stop> {
    let r38 = run_balance_report(env, ReportId::R38, ReportFilters::default())?;
    Ok(column_total(&r38, "still_owed"))
}

fn dues_places(env: &Env<'_>, today: &str) -> Result<Dues, Stop> {
    Ok(Dues {
        buyers: place_buyers(env, today)?,
        sellers: place_sellers(env, today)?,
        dead: place_dead(env)?,
    })
}

/// The tile of the total dues: checked when each of its three parts is, with the sum of their
/// second places as its own.
fn total_dues_node(t: &DuesParts, d: &Dues) -> Node {
    let parts = [
        (t.buyers_paisa, d.buyers),
        (t.sellers_paisa, d.sellers),
        (t.dead_paisa, d.dead),
    ];
    let all_agree = parts.iter().all(|(tile, other)| *other == Some(*tile));
    let sum = parts
        .iter()
        .map(|(_, other)| other.map(Value::Money))
        .collect::<Option<Vec<_>>>()
        .and_then(|values| figures::calculate(CalcOp::Sum, &values).ok())
        .and_then(|calc| match calc.result {
            Value::Money(p) => Some(p),
            _ => None,
        });
    match sum {
        Some(sum) => Node::Val(Val::Money(Money {
            paisa: t.total_paisa,
            side: None,
            checked: Some(all_agree && sum == t.total_paisa),
            places: vec![("tile", t.total_paisa), ("parts", sum)],
        })),
        None => plain(t.total_paisa),
    }
}

/// The tile of the total dues as the model reads it: its amount and what it holds. It is NOT the
/// shop's total اگراہی of PRD FR-19.9 (that is R20's total, the tile `buyers_dues`): it adds the
/// sellers who owe the firm and the dead amount (live run 2: 21,751,319 = 20,324,367 + 0 + 1,426,952,
/// quoted for a question about the collectors' dues, whose answer is 20,324,367).
fn total_dues_tile(amount: Node) -> Node {
    Node::obj(vec![
        ("amount", amount),
        (
            "holds",
            Node::List(
                ["buyers_dues", "sellers_dues", "dead_amount"]
                    .iter()
                    .map(|part| Node::Str((*part).to_owned()))
                    .collect(),
            ),
        ),
        (
            "note",
            Node::Str(
                "total_dues is buyers_dues plus sellers_dues plus dead_amount. The shop's \
                 ugrahi (dues of the buyers, by collector, by city, R20 to R23) is buyers_dues."
                    .to_owned(),
            ),
        ),
    ])
}

/// The three parts of the total dues as the tiles hold them.
fn dues_parts(t: &DashboardTiles) -> DuesParts {
    DuesParts {
        buyers_paisa: t.buyers_dues_paisa,
        sellers_paisa: t.sellers_dues_paisa,
        dead_paisa: t.dead_amount_paisa,
        total_paisa: t.total_dues_paisa,
    }
}

fn tiles_node(
    t: &DashboardTiles,
    sales: (Option<Paisa>, Option<Paisa>),
    cash: Option<Paisa>,
    dues: &Dues,
) -> Node {
    Node::obj(vec![
        (
            "today_sales",
            Node::obj(vec![
                ("amount", checked(t.sales_gross_paisa, "view_A", sales.0)),
                ("count", count(t.sales_count)),
            ]),
        ),
        (
            "today_commission",
            Node::obj(vec![(
                "amount",
                checked(t.commission_paisa, "view_A", sales.1),
            )]),
        ),
        (
            "cash_in_hand",
            Node::obj(vec![("amount", checked(t.cash_in_hand_paisa, "R12", cash))]),
        ),
        (
            "buyers_dues",
            Node::obj(vec![(
                "amount",
                checked(t.buyers_dues_paisa, "R20", dues.buyers),
            )]),
        ),
        (
            "sellers_dues",
            Node::obj(vec![(
                "amount",
                checked(t.sellers_dues_paisa, "R24", dues.sellers),
            )]),
        ),
        (
            "total_dues",
            total_dues_tile(total_dues_node(&dues_parts(t), dues)),
        ),
        (
            "dead_amount",
            Node::obj(vec![
                ("amount", checked(t.dead_amount_paisa, "R38", dues.dead)),
                ("count", count(t.dead_count)),
            ]),
        ),
        (
            "sellers_payable",
            Node::obj(vec![("amount", plain(t.sellers_payable_paisa))]),
        ),
        (
            "advances_open",
            Node::obj(vec![("amount", plain(t.advances_open_paisa))]),
        ),
        (
            "stock",
            Node::obj(vec![
                ("lots", count(t.stock_lots)),
                ("qty", count(t.stock_qty)),
            ]),
        ),
        (
            "trips",
            Node::obj(vec![
                ("open", count(t.trips_open)),
                ("purchases_with_stock", count(t.trips_purchases_with_stock)),
                ("qty_left", count(t.trips_qty_left)),
                ("cost_left", plain(t.trips_cost_left_paisa)),
            ]),
        ),
        (
            "gardens",
            Node::obj(vec![
                ("open", count(t.gardens_open)),
                ("balance", plain(t.gardens_balance_paisa)),
            ]),
        ),
        (
            "money_lent_open",
            Node::obj(vec![
                ("amount", plain(t.lent_open_paisa)),
                ("overdue", plain(t.lent_overdue_paisa)),
            ]),
        ),
        (
            "today_expenses",
            Node::obj(vec![("amount", plain(t.expenses_today_paisa))]),
        ),
    ])
}

/// The tile `last_backup`: the cloud copy on or off and its age in hours (null when there is none).
fn last_backup_node(backup: BackupFacts) -> Node {
    Node::obj(vec![
        ("cloud_on", Node::Bool(backup.cloud_on)),
        (
            "age_hours",
            backup
                .cloud_age_hours
                .map_or(Node::Null, |h| Node::Val(Val::Count(h))),
        ),
    ])
}

/// The tool.
pub fn run(env: &Env<'_>, _state: &State, args: &Json) -> Result<Draft, Stop> {
    let _: Input = input(args)?;
    // The backup state is not read here (see the module doc): the facts say "no cloud copy".
    let now = tiles_now(env.ctx.db()).map_err(reports_stop)?;
    let today = now.today.clone();
    let sales = view_a_today(env, &today).map_err(|_| internal("view A"))?;
    let cash = cash_closing(env, &today).map_err(|_| internal("R12"))?;
    let dues = dues_places(env, &today).map_err(|_| internal("R20, R24 or R38"))?;
    let mut node = tiles_node(&now.tiles, sales, cash, &dues);
    if let Node::Obj(fields) = &mut node {
        fields.push(("last_backup".to_owned(), last_backup_node(env.backup)));
    }
    let mut draft = Draft::new("tiles", args.clone(), node);
    draft.source = Some(AnswerSource::Tiles {
        result_id: String::new(),
    });
    Ok(draft)
}

/// The tiles a quick chip reads, one at a time (plan 48 P48-S1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    /// آج کی بکری.
    TodaySales,
    /// آج کا کمیشن.
    TodayCommission,
    /// روکڑ بقایا.
    CashInHand,
    /// گاہکوں کے ذمے.
    BuyersDues,
    /// کل اگراہی.
    TotalDues,
    /// ڈوبی رقم.
    DeadAmount,
    /// ادائیگی باقی.
    SellersPayable,
    /// پیشگی باقی.
    AdvancesOpen,
    /// باقی مال.
    Stock,
}

/// One tile of the tool `tiles`, read with its own narrow function and its second place only, so a
/// chip does not pay for the other tiles. The day of the run is the day of the tile and of its
/// second place. The result has the shape of the tool's result with this
/// tile alone (`r1.cash_in_hand.amount`), the same figure with the same ✓.
///
/// # Errors
/// As the tool.
pub fn run_one(env: &Env<'_>, tile: Tile) -> Result<Draft, Stop> {
    let db = env.ctx.db();
    let today = env.today.to_iso();
    let fail = |detail: &'static str| move |_: Stop| internal(detail);
    let node = match tile {
        Tile::TodaySales => {
            let sales = narrow::sales_today(db, env.today).map_err(reports_stop)?;
            let place = view_a_today(env, &today).map_err(fail("view A"))?;
            Node::obj(vec![(
                "today_sales",
                Node::obj(vec![
                    ("amount", checked(sales.gross_paisa, "view_A", place.0)),
                    ("count", count(sales.count)),
                ]),
            )])
        }
        Tile::TodayCommission => {
            let sales = narrow::sales_today(db, env.today).map_err(reports_stop)?;
            let place = view_a_today(env, &today).map_err(fail("view A"))?;
            Node::obj(vec![(
                "today_commission",
                Node::obj(vec![(
                    "amount",
                    checked(sales.commission_paisa, "view_A", place.1),
                )]),
            )])
        }
        Tile::CashInHand => {
            let cash = narrow::cash_in_hand(db).map_err(reports_stop)?;
            let place = cash_closing(env, &today).map_err(fail("R12"))?;
            Node::obj(vec![(
                "cash_in_hand",
                Node::obj(vec![("amount", checked(cash, "R12", place))]),
            )])
        }
        Tile::BuyersDues => {
            let dues = narrow::buyers_dues(db, env.today).map_err(reports_stop)?;
            let place = place_buyers(env, &today).map_err(fail("R20"))?;
            Node::obj(vec![(
                "buyers_dues",
                Node::obj(vec![("amount", checked(dues, "R20", place))]),
            )])
        }
        Tile::TotalDues => {
            let parts = narrow::total_dues(db, env.today).map_err(reports_stop)?;
            let places = dues_places(env, &today).map_err(fail("R20, R24 or R38"))?;
            Node::obj(vec![(
                "total_dues",
                total_dues_tile(total_dues_node(&parts, &places)),
            )])
        }
        Tile::DeadAmount => {
            let (amount, dead_count) = narrow::dead(db).map_err(reports_stop)?;
            let place = place_dead(env).map_err(fail("R38"))?;
            Node::obj(vec![(
                "dead_amount",
                Node::obj(vec![
                    ("amount", checked(amount, "R38", place)),
                    ("count", count(dead_count)),
                ]),
            )])
        }
        Tile::SellersPayable => {
            let sellers = narrow::sellers(db, env.today).map_err(reports_stop)?;
            Node::obj(vec![(
                "sellers_payable",
                Node::obj(vec![("amount", plain(sellers.payable_paisa))]),
            )])
        }
        Tile::AdvancesOpen => {
            let sellers = narrow::sellers(db, env.today).map_err(reports_stop)?;
            Node::obj(vec![(
                "advances_open",
                Node::obj(vec![("amount", plain(sellers.advances_open_paisa))]),
            )])
        }
        Tile::Stock => {
            let (lots, qty) = narrow::stock(db, env.today).map_err(reports_stop)?;
            Node::obj(vec![(
                "stock",
                Node::obj(vec![("lots", count(lots)), ("qty", count(qty))]),
            )])
        }
    };
    let mut draft = Draft::new("tiles", Json::Object(serde_json::Map::new()), node);
    draft.source = Some(AnswerSource::Tiles {
        result_id: String::new(),
    });
    Ok(draft)
}
