//! The reads the app does BEFORE the first model call, planned from what it understood (plan 54 S3,
//! plan 55 Section 2).
//!
//! The model's first call used to be spent choosing a tool and its filters; the second wrote the
//! answer. When the question is understood completely, [`plan_reads`] gives the exact tool calls the
//! model would have sent, in the same shape (`name` and the input JSON with every field of its
//! schema), so the caller can run them before call 1 and put the results in it. The model then writes
//! the answer at once. The reads are the same read-only tools with the same filters, so every figure
//! is still checked (PRD FR-7.76). Nothing is added, summed or rounded here (Code Standards Q5).
//!
//! The order is the order to run them in:
//! 1. `period`, when a read has dates (it gives the label of the period for the answer);
//! 2. a `find` for every record a later read names (a party, an item, a partner, a city, an اڈا);
//! 3. the reads of the figure.
//!
//! A later read names a record by its KEY (`a18`), which the tools accept because the `find` before
//! it has shown that key (`State::knows_key`). A read whose tool refuses its input is dropped by the
//! caller; the model carries on from what was read.
//!
//! | The question | The reads |
//! |---|---|
//! | a buyer's or seller's balance, dues, payable, advance | `find`, `party_card` |
//! | his khata or statement, what a buyer bought | `find`, R13, R14, R2 (R35 for one day) |
//! | his commission, sale or count over a period | `find`, view B (seller) or C (buyer) |
//! | an item's commission, sale or rate; the item with a party | `find`, view A (with `seller` or `buyer`) |
//! | the shop's sale or commission over a period | the tiles for today, else view A (B for the count) |
//! | cash, dues, payables, advances, dead amount, money lent, now | the tiles |
//! | goods left now | R10, oldest first |
//! | cash, expenses, day summary, day book, receipts over a period | R12, R29, R34, R11, R23 |
//! | the trial balance | R18 |
//! | a list of buyers' dues or sellers' payables | R20 or R24, sorted |
//! | the top buyers, sellers or items by commission | view H (A, B, C sorted for the lowest or for sales) |
//! | the partners' totals, one partner's total | R31 |
//! | dues of a city; sales of an اڈا; expenses (head) | R21; R8; R29 with the period |
//!
//! `None` when the question is not completely understood, compares things, has two figures, names
//! more than one party, item or period, names a garden, a trip or own goods, or is not covered: the model
//! decides then. A ranking or period the app assumed (`Understanding::assumed`) is read as assumed;
//! the caller states it in the answer.
//!
//! Example (today 10-10-2026): "kele ka commission pichle mahine" gives `period` last month, `find`
//! banana as an item, and view A with `item` the key and 2026-09-01 to 2026-09-30.

use serde_json::{Value, json};

use super::lexicon::{Cue, key};
use super::scan::Scan;
use super::{
    Doubt, FigureKind, Intent, ListOver, Outcome, PartyHit, PartyRole, PeriodKind, PeriodRef, Rank,
    RecordHit, RecordKind, Understanding,
};

/// One read to do before the first call: a tool and its input, as the model would send it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedRead {
    /// The tool: `period`, `find`, `party_card`, `report`, `analytics` or `tiles`.
    pub tool: &'static str,
    /// Its input, every field of its schema (`null` for the ones not set).
    pub input: Value,
}

/// Doubts that are not blocking for an answer but that mean the app is not certain what was meant.
const NOT_CARRIED: [Doubt; 6] = [
    Doubt::UnknownWords,
    Doubt::CloseSpelling,
    Doubt::OtherRecord,
    Doubt::Comparison,
    Doubt::PartnerLocked,
    Doubt::NoPeriod,
];

/// Words that scope a figure to a garden, a trip, own goods or the guests. The reads have no filter
/// for them, so a question with one is the model's.
const SCOPE_WORDS: [&str; 17] = [
    "bagh",
    "bag",
    "baag",
    "baagh",
    "garden",
    "gardens",
    "باغ",
    "باغات",
    "safar",
    "trip",
    "trips",
    "سفر",
    "mehman",
    "مہمان",
    "guest",
    "guests",
    "goods",
];

/// Phrases that scope a figure to own goods.
const SCOPE_PHRASES: [&str; 3] = ["apna maal", "apna mal", "اپنا مال"];

/// True when the question names a garden, a trip, own goods or the guests, by word.
pub(super) fn scoped(scan: &Scan) -> bool {
    let joined = format!(" {} ", scan.words.join(" "));
    SCOPE_PHRASES
        .iter()
        .any(|p| joined.contains(&format!(" {} ", key(p))))
        || scan
            .words
            .iter()
            .any(|w| SCOPE_WORDS.iter().any(|s| key(s) == *w))
        || scan.has_exact(Cue::RoleGarden)
        || scan.has_exact(Cue::RoleTrip)
}

// ---- the shapes of the inputs --------------------------------------------------------------------------

fn find(text: &str, role: &str) -> PlannedRead {
    PlannedRead {
        tool: "find",
        input: json!({"text": text, "role": role}),
    }
}

fn party_card(key: &str) -> PlannedRead {
    PlannedRead {
        tool: "party_card",
        input: json!({"party": key}),
    }
}

fn tiles() -> PlannedRead {
    PlannedRead {
        tool: "tiles",
        input: json!({}),
    }
}

/// The filters of a `report` call. Every field of the schema is written, `null` if not set.
#[derive(Default)]
struct Rep {
    from: Option<String>,
    to: Option<String>,
    as_of: Option<String>,
    party: Option<String>,
    item: Option<String>,
    city: Option<String>,
    adda: Option<String>,
    partner: Option<String>,
    sort: Option<Value>,
    rows: Option<u32>,
}

fn report(id: &str, r: Rep) -> PlannedRead {
    PlannedRead {
        tool: "report",
        input: json!({
            "id": id, "from": r.from, "to": r.to, "as_of": r.as_of, "party": r.party,
            "item": r.item, "city": r.city, "adda": r.adda, "collector": null,
            "partner": r.partner, "trip": null, "garden": null, "bikri": null,
            "group_by": null, "where": null, "sort": r.sort, "rows": r.rows
        }),
    }
}

/// The filters of an `analytics` call. Every field of the schema is written, `null` if not set.
#[derive(Default)]
struct An {
    from: Option<String>,
    to: Option<String>,
    item: Option<String>,
    seller: Option<String>,
    buyer: Option<String>,
    sort: Option<Value>,
    rows: Option<u32>,
}

fn analytics(view: &str, a: An) -> PlannedRead {
    PlannedRead {
        tool: "analytics",
        input: json!({
            "view": view, "from": a.from, "to": a.to, "group_by": null, "item": a.item,
            "seller": a.seller, "buyer": a.buyer, "city": null, "adda": null, "garden": null,
            "trip": null, "goods": null, "partner": null, "side": null, "compare": null,
            "sort": a.sort, "rows": a.rows
        }),
    }
}

fn sorted(column: &str, order: &str) -> Value {
    json!([{"column": column, "order": order}])
}

fn iso(day: chrono::NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

/// The `period` read of a period: the word when the tool has one, else the dates as a range.
fn period_read(p: &PeriodRef) -> Option<PlannedRead> {
    let word = match p.kind {
        PeriodKind::Today => Some("today"),
        PeriodKind::Yesterday => Some("yesterday"),
        PeriodKind::ThisWeek => Some("this_week"),
        PeriodKind::LastWeek => Some("last_week"),
        PeriodKind::ThisMonth => Some("this_month"),
        PeriodKind::LastMonth => Some("last_month"),
        PeriodKind::ThisYear => Some("this_year"),
        PeriodKind::LastYear => Some("last_year"),
        PeriodKind::UpToNow => Some("up_to_now"),
        _ => None,
    };
    let (kind, from, to) = match word {
        Some(w) => (w, Value::Null, Value::Null),
        None => ("range", json!(iso(p.from?)), json!(iso(p.to))),
    };
    Some(PlannedRead {
        tool: "period",
        input: json!({
            "kind": kind, "month": null, "day": null, "from": from, "to": to, "n": null
        }),
    })
}

// ---- the plan ------------------------------------------------------------------------------------------

/// What a plan collects: the finds its reads need, and the reads.
#[derive(Default)]
struct Plan {
    finds: Vec<PlannedRead>,
    reads: Vec<PlannedRead>,
    /// True when a read has the dates of the period (the `period` read is then added).
    dated: bool,
}

impl Plan {
    fn find(&mut self, text: &str, role: &str) {
        let one = find(text, role);
        if !self.finds.contains(&one) {
            self.finds.push(one);
        }
    }

    fn read(&mut self, read: PlannedRead) {
        self.reads.push(read);
    }
}

/// The dates of the question's period as `from` and `to`; an empty `from` stays open.
fn dates(period: Option<&PeriodRef>) -> (Option<String>, Option<String>) {
    match period {
        Some(p) => (p.from.map(iso), Some(iso(p.to))),
        None => (None, None),
    }
}

fn is_one_day(period: Option<&PeriodRef>) -> bool {
    period.is_some_and(|p| p.from == Some(p.to))
}

/// The tool calls to make before the first model call, or `None` when the model is to choose.
pub fn plan_reads(u: &Understanding) -> Option<Vec<PlannedRead>> {
    if !u.sure
        || u.compare
        || u.two_figures
        || u.scoped
        || u.periods.len() > 1
        || u.parties.len() > 1
        || u.items.len() > 1
        || u.records.len() > 1
        || u.doubts.iter().any(|d| NOT_CARRIED.contains(d))
    {
        return None;
    }
    let period = u.periods.first();
    // A period after today, or one the question did not say plainly, is the model's.
    if period.is_some_and(|p| p.future) {
        return None;
    }
    let mut plan = Plan::default();
    match u.intent.value {
        Intent::Figure => figure(u, period, &mut plan)?,
        // "arshad ne kya kya khareeda": a list of his lines, which is his figure.
        Intent::List if !u.parties.is_empty() => figure(u, period, &mut plan)?,
        Intent::List => list(u, period, &mut plan)?,
        _ => return None,
    }
    let mut out = Vec::new();
    if plan.dated
        && let Some(p) = period
    {
        out.push(period_read(p)?);
    }
    out.extend(plan.finds);
    out.extend(plan.reads);
    (!out.is_empty()).then_some(out)
}

/// The party the question names, found as asked: a buyer or a seller (or a partner), with the
/// words as typed.
fn party_of(u: &Understanding) -> Option<(&str, &PartyHit)> {
    match u.parties.as_slice() {
        [one] => match &one.outcome {
            Outcome::Found(hit) => Some((one.text.as_str(), hit)),
            _ => None,
        },
        _ => None,
    }
}

/// True when every name of the question was found as asked.
fn names_found(u: &Understanding) -> bool {
    u.parties
        .iter()
        .all(|p| matches!(p.outcome, Outcome::Found(_)))
        && u.items
            .iter()
            .all(|i| matches!(i.outcome, Outcome::Found(_)))
        && u.records
            .iter()
            .all(|r| matches!(r.outcome, Outcome::Found(_)))
}

fn role_word(role: PartyRole) -> Option<&'static str> {
    match role {
        PartyRole::Buyer => Some("buyer"),
        PartyRole::Seller => Some("seller"),
        PartyRole::Partner => Some("partner"),
        _ => None,
    }
}

// ---- a figure ------------------------------------------------------------------------------------------

fn figure(u: &Understanding, period: Option<&PeriodRef>, plan: &mut Plan) -> Option<()> {
    if !names_found(u) {
        return None;
    }
    let [kind] = u.figures.value.as_slice() else {
        return None;
    };
    let kind = *kind;
    let party = party_of(u);
    let item = u.items.first().and_then(|i| match &i.outcome {
        Outcome::Found(hit) => Some((i.text.as_str(), hit)),
        _ => None,
    });
    let record = u.records.first().and_then(|r| match &r.outcome {
        Outcome::Found(hit) => Some((r.text.as_str(), hit)),
        _ => None,
    });
    match (party, item, record) {
        (Some((text, hit)), None, None) if hit.role == PartyRole::Partner => {
            partner_figure(kind, text, hit, period, plan)
        }
        (Some((text, hit)), None, None) => party_figure(kind, text, hit, period, plan),
        (None, Some((text, hit)), None) => item_figure(kind, text, &hit.key, period, plan),
        (Some((ptext, phit)), Some((itext, ihit)), None) => {
            party_item_figure(kind, (ptext, phit), (itext, &ihit.key), period, plan)
        }
        (None, None, Some((text, hit))) => record_figure(kind, text, hit, period, plan),
        (None, None, None) => shop_figure(kind, period, plan),
        _ => None,
    }
}

/// A buyer's or a seller's figure.
fn party_figure(
    kind: FigureKind,
    text: &str,
    hit: &PartyHit,
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    let role = hit.role;
    let word = role_word(role).filter(|_| role != PartyRole::Partner)?;
    let (from, to) = dates(period);
    let key = hit.key.clone();
    match kind {
        // The card: balance with its side, advance, payable, oldest open bill. Now, not for a period.
        FigureKind::Balance | FigureKind::Dues | FigureKind::Payable | FigureKind::Advance => {
            if period.is_some() {
                return None;
            }
            plan.find(text, word);
            plan.read(party_card(&key));
        }
        // His day: the party day sheet (R35, PRD FR-19.12).
        FigureKind::Ledger | FigureKind::Statement | FigureKind::Purchases
            if is_one_day(period) =>
        {
            plan.find(text, word);
            plan.read(report(
                "R35",
                Rep {
                    party: Some(key),
                    as_of: to,
                    ..Rep::default()
                },
            ));
        }
        FigureKind::Ledger | FigureKind::Statement => {
            plan.find(text, word);
            let id = if kind == FigureKind::Ledger {
                "R13"
            } else {
                "R14"
            };
            plan.read(report(
                id,
                Rep {
                    party: Some(key),
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
        }
        // What a buyer bought: the sale lines with items and values.
        FigureKind::Purchases if role == PartyRole::Buyer => {
            plan.find(text, word);
            plan.read(report(
                "R2",
                Rep {
                    party: Some(key),
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
        }
        // His commission and sales: a seller in view B, a buyer in view C.
        FigureKind::Commission | FigureKind::Sale | FigureKind::SaleCount | FigureKind::Laga => {
            let view = match (role, kind) {
                (PartyRole::Seller, FigureKind::Laga) => return None,
                (PartyRole::Seller, _) => "B",
                (PartyRole::Buyer, FigureKind::SaleCount) => return None,
                (PartyRole::Buyer, _) => "C",
                _ => return None,
            };
            plan.find(text, word);
            let mut a = An {
                from,
                to,
                ..An::default()
            };
            if role == PartyRole::Seller {
                a.seller = Some(key);
            } else {
                a.buyer = Some(key);
            }
            plan.read(analytics(view, a));
            plan.dated = period.is_some();
        }
        _ => return None,
    }
    Some(())
}

/// A partner's figure: his total (R31).
fn partner_figure(
    kind: FigureKind,
    text: &str,
    hit: &PartyHit,
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    if !matches!(kind, FigureKind::PartnerTotal | FigureKind::Capital) {
        return None;
    }
    let (from, to) = dates(period);
    plan.find(text, "partner");
    plan.read(report(
        "R31",
        Rep {
            partner: Some(hit.key.clone()),
            from,
            to,
            ..Rep::default()
        },
    ));
    plan.dated = period.is_some();
    Some(())
}

/// An item's figure: view A with the item.
fn item_figure(
    kind: FigureKind,
    text: &str,
    key: &str,
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    let (from, to) = dates(period);
    match kind {
        FigureKind::Commission | FigureKind::Laga | FigureKind::Sale | FigureKind::Rate => {
            plan.find(text, "item");
            plan.read(analytics(
                "A",
                An {
                    from,
                    to,
                    item: Some(key.to_owned()),
                    ..An::default()
                },
            ));
            plan.dated = period.is_some();
        }
        // The goods of the item left (R10 has no dates to read for a stock now).
        FigureKind::Stock if period.is_none() => {
            plan.find(text, "item");
            plan.read(report(
                "R10",
                Rep {
                    item: Some(key.to_owned()),
                    ..Rep::default()
                },
            ));
        }
        _ => return None,
    }
    Some(())
}

/// A party's figure of one item: view A with the item and the party.
fn party_item_figure(
    kind: FigureKind,
    (ptext, phit): (&str, &PartyHit),
    (itext, ikey): (&str, &str),
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    if !matches!(
        kind,
        FigureKind::Commission | FigureKind::Laga | FigureKind::Sale
    ) {
        return None;
    }
    let word = role_word(phit.role).filter(|_| phit.role != PartyRole::Partner)?;
    let (from, to) = dates(period);
    plan.find(ptext, word);
    plan.find(itext, "item");
    let mut a = An {
        from,
        to,
        item: Some(ikey.to_owned()),
        ..An::default()
    };
    if phit.role == PartyRole::Seller {
        a.seller = Some(phit.key.clone());
    } else {
        a.buyer = Some(phit.key.clone());
    }
    plan.read(analytics("A", a));
    plan.dated = period.is_some();
    Some(())
}

/// A city's dues, an اڈا's sales, an expense head's expenses.
fn record_figure(
    kind: FigureKind,
    text: &str,
    hit: &RecordHit,
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    let (from, to) = dates(period);
    match (hit.kind, kind) {
        (RecordKind::City, FigureKind::Dues | FigureKind::Balance) if period.is_none() => {
            plan.find(text, "city");
            plan.read(report(
                "R21",
                Rep {
                    city: Some(hit.key.clone()),
                    ..Rep::default()
                },
            ));
        }
        (RecordKind::Adda, FigureKind::Sale | FigureKind::SaleCount) => {
            plan.find(text, "adda");
            plan.read(report(
                "R8",
                Rep {
                    adda: Some(hit.key.clone()),
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
        }
        // The report has no expense-head filter: the period is read and the row is found in it.
        (RecordKind::ExpenseHead, FigureKind::Expense) if period.is_some() => {
            plan.read(report(
                "R29",
                Rep {
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = true;
        }
        _ => return None,
    }
    Some(())
}

/// A figure of the books as a whole.
fn shop_figure(kind: FigureKind, period: Option<&PeriodRef>, plan: &mut Plan) -> Option<()> {
    let (from, to) = dates(period);
    let today = period.is_some_and(|p| p.kind == PeriodKind::Today);
    match kind {
        // Sales and commission: the tiles for today, else view A (view B for the number of بکری).
        FigureKind::Sale | FigureKind::SaleCount | FigureKind::Commission | FigureKind::Laga => {
            period?;
            if today && kind != FigureKind::Laga {
                plan.read(tiles());
            } else {
                let view = if kind == FigureKind::SaleCount {
                    "B"
                } else {
                    "A"
                };
                plan.read(analytics(
                    view,
                    An {
                        from,
                        to,
                        ..An::default()
                    },
                ));
                plan.dated = true;
            }
        }
        FigureKind::Cash if period.is_none() || today => plan.read(tiles()),
        FigureKind::Cash => {
            plan.read(report(
                "R12",
                Rep {
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = true;
        }
        // Dues, payables, advances, the dead amount, money lent and goods left, now.
        FigureKind::Dues
        | FigureKind::Payable
        | FigureKind::Advance
        | FigureKind::DeadAmount
        | FigureKind::MoneyLent
            if period.is_none() =>
        {
            plan.read(tiles());
        }
        // The goods left: R10 holds the lots and the totals over all of them, oldest first.
        FigureKind::Stock if period.is_none() => {
            plan.read(report(
                "R10",
                Rep {
                    sort: Some(sorted("days", "desc")),
                    ..Rep::default()
                },
            ));
        }
        FigureKind::Receipts => {
            period?;
            plan.read(report(
                "R23",
                Rep {
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = true;
        }
        FigureKind::Expense => {
            period?;
            plan.read(report(
                "R29",
                Rep {
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = true;
        }
        FigureKind::DaySummary => {
            period?;
            plan.read(report(
                "R34",
                Rep {
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = true;
        }
        FigureKind::DayBook if is_one_day(period) => {
            plan.read(report(
                "R11",
                Rep {
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = true;
        }
        FigureKind::TrialBalance => {
            plan.read(report(
                "R18",
                Rep {
                    as_of: to,
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
        }
        // All the partners' totals (the app asks for the code while the section is locked).
        FigureKind::PartnerTotal | FigureKind::Capital => {
            plan.read(report(
                "R31",
                Rep {
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
        }
        _ => return None,
    }
    Some(())
}

// ---- a list --------------------------------------------------------------------------------------------

fn list(u: &Understanding, period: Option<&PeriodRef>, plan: &mut Plan) -> Option<()> {
    if !u.parties.is_empty() || !u.items.is_empty() || !u.records.is_empty() {
        return None;
    }
    let ask = u.list?;
    let measure = u.figures.value.first().copied();
    if u.figures.value.len() > 1 {
        return None;
    }
    // "kis kis ke dues hain": the figure says whose (dues are buyers', payables sellers', goods
    // items'). A balance alone could be either, so it is not guessed.
    let over = ask
        .over
        .filter(|o| *o != ListOver::Parties)
        .or(match measure {
            Some(FigureKind::Dues) => Some(ListOver::Buyers),
            Some(FigureKind::Payable) => Some(ListOver::Sellers),
            Some(FigureKind::Stock) => Some(ListOver::Items),
            _ => None,
        })?;
    let (from, to) = dates(period);
    let order = |least: bool| if least { "asc" } else { "desc" };
    let least = ask.rank == Some(Rank::Least);
    match (over, measure) {
        // Who owes, and who is owed (R20, R24 are as of now).
        (ListOver::Buyers, Some(FigureKind::Dues | FigureKind::Balance)) if period.is_none() => {
            plan.read(report(
                "R20",
                Rep {
                    sort: Some(sorted("balance", order(least))),
                    rows: ask.n,
                    ..Rep::default()
                },
            ));
        }
        (ListOver::Sellers, Some(FigureKind::Payable | FigureKind::Balance))
            if period.is_none() =>
        {
            plan.read(report(
                "R24",
                Rep {
                    sort: Some(sorted("payable", order(least))),
                    rows: ask.n,
                    ..Rep::default()
                },
            ));
        }
        // The goods left, oldest first.
        (ListOver::Items, Some(FigureKind::Stock)) if period.is_none() => {
            plan.read(report(
                "R10",
                Rep {
                    sort: Some(sorted("days", "desc")),
                    rows: ask.n,
                    ..Rep::default()
                },
            ));
        }
        // The top ten by total commission: view H holds the items, the sellers and the buyers.
        (ListOver::Items | ListOver::Sellers | ListOver::Buyers, Some(FigureKind::Commission))
            if ask.rank == Some(Rank::Most) =>
        {
            plan.read(analytics(
                "H",
                An {
                    from,
                    to,
                    ..An::default()
                },
            ));
            plan.dated = period.is_some();
        }
        // The lowest, or the most by sales or commission: the view of the kind, sorted.
        (
            ListOver::Items | ListOver::Sellers | ListOver::Buyers,
            Some(FigureKind::Commission | FigureKind::Sale),
        ) if ask.rank.is_some() => {
            let commission = measure == Some(FigureKind::Commission);
            let (view, column) = match (over, commission) {
                (ListOver::Items, true) => ("A", "total_comm"),
                (ListOver::Items, false) => ("A", "gross"),
                (ListOver::Sellers, true) => ("B", "total_comm"),
                (ListOver::Sellers, false) => ("B", "gross"),
                (_, true) => ("C", "buyer_comm"),
                (_, false) => ("C", "purchase_value"),
            };
            plan.read(analytics(
                view,
                An {
                    from,
                    to,
                    sort: Some(sorted(column, order(least))),
                    rows: ask.n,
                    ..An::default()
                },
            ));
            plan.dated = period.is_some();
        }
        _ => return None,
    }
    Some(())
}
