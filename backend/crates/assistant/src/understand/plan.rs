//! Which report a print or an open-the-report request means, and with which filters (plan 54
//! Sections 6.2 and 6.4, plan 55 ruling 1, PRD FR-19.10, FR-19.12).
//!
//! | Asked | One day | A period |
//! |---|---|---|
//! | A buyer's bikri or bill | R35, his day slip (`party`, `as_of`) | R5 (`party`, `from`, `to`), and R13 as the second button |
//! | A seller's bikri | R6 (`seller`, `from`, `to`, the day twice) | R6, and R13 as the second button |
//! | A whole day or period, no party | R34 (`from`, `to`) | R2 (`from`, `to`) |
//! | A party's khata or statement | R13 or R14 (`party`, `from`, `to`) | the same |
//!
//! The rows of plan 55 ruling 1: a partner's statement R31 (`partner`, `from`, `to`); cash book R12
//! and expense report R29 (`from`, `to`); dues R20, payables R24 and the trial balance R18
//! (`as_of`); stock R10 (`from`, `to`: from the first day of the books to the day); the day book R11
//! (`from` = `to` = the day); gardens R36 (`garden`, `from`, `to`); trips R33 (`from`, `to`: the trip's
//! days, with the trip layout); fruit taken R37 (`from`, `to`, and `partner` when named); dues by
//! city R21 (`city`, `as_of`); the truck stand report R8 (`adda`, `from`, `to`).
//!
//! The screens of R33 and R29 have no one-trip and no expense-head filter (lead ruling of
//! 04-11-2026), so a named trip or head is carried here only to say, in the steps, which row to
//! read. The button never claims a filter the screen does not apply.
//!
//! The party comes from the books in his real role: a buyer is never printed as a seller. The
//! filters are the ones the report links take; no new link type.
//!
//! A print follows the figure asked (money review R-g): a receipt, an advance or a payment of a
//! buyer or a seller is in his khata (R13), not in the sale reports; a figure no report of a party
//! holds (cash, stock ...) plans no report.
//!
//! A report is never built wider than the question (money review BUG-1, BUG-2): none for a question
//! that names an item (no link has an item filter), a garden, a trip, own goods or the guests that
//! no report of its own covers, or a name the app cannot place (a چکتا buyer, an inactive
//! account, a partner while the section is locked). The model answers those.
//!
//! Example (today 10-10-2026): "faisal ki aaj ki bikri ka print", Faisal a buyer: R35 with
//! `party` Faisal and `as_of` 2026-10-10.

use chrono::NaiveDate;

use super::who::Names;
use super::{
    Doubt, FigureKind, GroupWord, Intent, ItemHit, Outcome, PartyHit, PartyRole, PeriodRef,
    RecordHit, RecordKind,
};

/// The filters of a report link.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanFilters {
    /// The party (R35, R5, R13, R14, and R2 where it is given).
    pub party: Option<PartyHit>,
    /// The seller (R6).
    pub seller: Option<PartyHit>,
    /// The partner (R31, R37).
    pub partner: Option<PartyHit>,
    /// The garden (R36).
    pub garden: Option<PartyHit>,
    /// The trip (R33). Named in the steps only: the report has no one-trip filter.
    pub trip: Option<PartyHit>,
    /// The city (R21).
    pub city: Option<RecordHit>,
    /// The اڈا (R8).
    pub adda: Option<RecordHit>,
    /// The expense head (R29). Named in the steps only: the report has no expense-head filter.
    pub head: Option<RecordHit>,
    /// The item (R2 to R4, R7, R10, R27 and the analytics views A, D, E, G).
    pub item: Option<ItemHit>,
    /// All the items of a group (phal, sabzi): the same reports and views as an item.
    pub item_group: Option<GroupWord>,
    /// Only the shop's guests (R37).
    pub guests: bool,
    /// The one بکری of R1: its row id and its number.
    pub bikri: Option<(i64, i64)>,
    /// First day.
    pub from: Option<NaiveDate>,
    /// Last day.
    pub to: Option<NaiveDate>,
    /// The day of a day slip or a balance list (R35, R20, R24, R18).
    pub as_of: Option<NaiveDate>,
}

impl PlanFilters {
    /// The filters as the report link names them: a record as its id, a date as `YYYY-MM-DD`. A
    /// filter that is not set is left out.
    pub fn fields(&self) -> Vec<(&'static str, String)> {
        let iso = |d: &NaiveDate| d.format("%Y-%m-%d").to_string();
        let mut out = Vec::new();
        let parties = [
            ("party", &self.party),
            ("seller", &self.seller),
            ("partner", &self.partner),
            ("garden", &self.garden),
            ("trip", &self.trip),
        ];
        for (name, hit) in parties {
            if let Some(h) = hit {
                out.push((name, h.id.to_string()));
            }
        }
        for (name, hit) in [
            ("city", &self.city),
            ("adda", &self.adda),
            ("head", &self.head),
        ] {
            if let Some(h) = hit {
                out.push((name, h.id.to_string()));
            }
        }
        if let Some(item) = &self.item {
            out.push(("item", item.id.to_string()));
        }
        if let Some(group) = self.item_group {
            out.push(("item_group", group.tool_word().to_owned()));
        }
        if self.guests {
            out.push(("guests", "true".to_owned()));
        }
        if let Some((_, number)) = &self.bikri {
            out.push(("bikri", number.to_string()));
        }
        for (name, date) in [
            ("from", &self.from),
            ("to", &self.to),
            ("as_of", &self.as_of),
        ] {
            if let Some(d) = date {
                out.push((name, iso(d)));
            }
        }
        out
    }
}

/// A report with its filters, and the second one the plan offers as a second button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportPlan {
    /// The report, for example `R35`.
    pub id: &'static str,
    /// Its filters.
    pub filters: PlanFilters,
    /// The second report to offer, if the plan gives one.
    pub also: Option<Box<ReportPlan>>,
    /// True when the question named no day and today was taken. The caller should not call this
    /// sure.
    pub assumed_period: bool,
}

pub(super) struct Input<'a> {
    pub intent: Intent,
    pub names: &'a Names,
    pub periods: &'a [PeriodRef],
    pub kinds: &'a [FigureKind],
    pub today: String,
    pub statement: bool,
    pub ledger: bool,
    /// The question says "by city".
    pub by_city: bool,
    /// The partner section is locked and the question is about a partner.
    pub partner_locked: bool,
    /// The first day of the books, when stock (R10) is asked: stock now runs from there to the day.
    pub first_day: Option<NaiveDate>,
    /// The days of the trip that is named (R33): its day to the day it closed, or to today.
    pub trip_days: Option<(NaiveDate, NaiveDate)>,
    /// Words stand where a name should be and no record has them: a چکتا buyer, an inactive
    /// account, a partner while the section is locked. The report would be the whole shop's.
    pub unknown_words: bool,
    /// The question names a garden, a trip, own goods or the guests, and no record of it was found
    /// (or the link has no filter for it).
    pub scope_missing: bool,
    /// The question says "garden" (bagh) and no garden of the books is named: the gardens report.
    pub garden_word: bool,
    /// The question says "trip" (safar) and no trip of the books is named: the trips report.
    pub trip_word: bool,
    /// The question names the shop's guests: with the fruit taken, the register of their fruit.
    pub guests: bool,
    /// The question names a group of items (sabzi, phal), and no item.
    pub item_group: Option<GroupWord>,
    /// A بکری number the question names ("bikri number 5"), and the row found for it (id, number).
    pub bikri_number: Option<i64>,
    pub bikri_row: Option<(i64, i64)>,
}

pub(super) struct Planned {
    pub report: Option<ReportPlan>,
    pub doubts: Vec<Doubt>,
}

fn none(doubt: Doubt) -> Planned {
    Planned {
        report: None,
        doubts: vec![doubt],
    }
}

fn nothing() -> Planned {
    Planned {
        report: None,
        doubts: Vec::new(),
    }
}

fn plan_of(
    id: &'static str,
    filters: PlanFilters,
    also: Option<ReportPlan>,
    assumed: bool,
) -> ReportPlan {
    ReportPlan {
        id,
        filters,
        also: also.map(Box::new),
        assumed_period: assumed,
    }
}

/// How a report takes its days.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// R34 for one day, R2 for a range.
    Sales,
    /// `from` and `to` (R12, R29, R37).
    Range(&'static str),
    /// `as_of` (R20, R24, R18).
    AsOf(&'static str),
    /// `from` the first day of the books `to` the day (R10).
    Stock,
    /// One day, as `from` and `to` (R11).
    OneDay(&'static str),
    /// A list with no date and no party: R19 (opening trial) and R38 (the dead amount).
    NoFilters(&'static str),
    /// An analytics view A to K: the days are optional (the screen has its own), no day is assumed.
    View(&'static str),
}

/// True for the id of an analytics view (A to K) and not of a report.
pub(super) fn is_view(id: &str) -> bool {
    matches!(
        id,
        "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K"
    )
}

/// How a report named by its title takes its filters (the specs of the report engine, PRD 19.2).
fn report_shape(id: &'static str) -> Option<Shape> {
    Some(match id {
        "R3" | "R4" | "R7" | "R8" | "R15" | "R23" | "R25" | "R27" | "R28" | "R30" => {
            Shape::Range(id)
        }
        "R16" | "R17" | "R22" | "R26" | "R32" => Shape::AsOf(id),
        "R19" | "R38" => Shape::NoFilters(id),
        _ if is_view(id) => Shape::View(id),
        _ => return None,
    })
}

/// The report a figure belongs to when no person or record is named.
fn shape_of(kind: FigureKind) -> Option<Shape> {
    Some(match kind {
        FigureKind::Sale
        | FigureKind::SaleCount
        | FigureKind::Commission
        | FigureKind::Laga
        | FigureKind::DaySummary
        | FigureKind::Purchases => Shape::Sales,
        FigureKind::Cash => Shape::Range("R12"),
        FigureKind::Expense => Shape::Range("R29"),
        // The money the partners took or gave back: the expense report holds it by partner.
        FigureKind::PartnerTaken => Shape::Range("R29"),
        FigureKind::FruitTaken => Shape::Range("R37"),
        FigureKind::Dues => Shape::AsOf("R20"),
        // What was collected on the days: the receipts register, not the list of what is still due.
        FigureKind::Receipts => Shape::Range("R23"),
        FigureKind::Payable => Shape::AsOf("R24"),
        // The advances report, the profit split, the money lent and the dead amount lists: reports
        // of the whole shop, with no party named.
        FigureKind::Advance => Shape::AsOf("R26"),
        FigureKind::Profit => Shape::Range("R30"),
        FigureKind::MoneyLent => Shape::AsOf("R32"),
        FigureKind::DeadAmount => Shape::NoFilters("R38"),
        FigureKind::Report(id) => return report_shape(id),
        FigureKind::Stock => Shape::Stock,
        FigureKind::TrialBalance => Shape::AsOf("R18"),
        FigureKind::DayBook => Shape::OneDay("R11"),
        _ => return None,
    })
}

/// What a print of a buyer or a seller is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wants {
    /// His sales, purchases, commission or balance (or nothing said): the sale reports hold them.
    Sales,
    /// A receipt, an advance or a payment: his khata holds them.
    Khata,
    /// A figure no report of a party holds (cash, stock, a partner's total ...).
    Other,
}

fn wants_of(kinds: &[FigureKind]) -> Wants {
    let mut khata = false;
    for kind in kinds {
        match kind {
            FigureKind::Sale
            | FigureKind::SaleCount
            | FigureKind::Purchases
            | FigureKind::Commission
            | FigureKind::Laga
            | FigureKind::DaySummary
            | FigureKind::Balance
            | FigureKind::Dues
            | FigureKind::Ledger
            | FigureKind::Statement => {}
            FigureKind::Receipts | FigureKind::Advance | FigureKind::Payable => khata = true,
            _ => return Wants::Other,
        }
    }
    if khata { Wants::Khata } else { Wants::Sales }
}

/// The days of a plan.
struct Days {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    one_day: bool,
    named: bool,
}

/// The report of a print or an open-the-report request.
pub(super) fn build(input: &Input<'_>) -> Planned {
    if !matches!(input.intent, Intent::Print | Intent::Navigate) {
        return nothing();
    }
    let names = input.names;
    if names
        .parties
        .iter()
        .any(|p| matches!(p.outcome, Outcome::Several(_) | Outcome::NotFound))
        || names
            .records
            .iter()
            .any(|r| matches!(r.outcome, Outcome::Several(_)))
    {
        // The names decide first: the caller asks which one, or says there is no such party.
        return nothing();
    }
    let Some(today) = NaiveDate::parse_from_str(&input.today, "%Y-%m-%d").ok() else {
        return none(Doubt::LookupFailed);
    };
    let period = input.periods.first();
    if period.is_some_and(|p| p.future) {
        return none(Doubt::NoPeriod);
    }
    // Two report titles in one question are two reports.
    if input
        .kinds
        .iter()
        .filter(|k| matches!(k, FigureKind::Report(_)))
        .count()
        > 1
    {
        return none(Doubt::NoReport);
    }
    let navigate = input.intent == Intent::Navigate;
    let days = match period {
        Some(p) => Days {
            from: p.from,
            to: Some(p.to),
            one_day: p.from == Some(p.to),
            named: true,
        },
        None => Days {
            from: Some(today),
            to: Some(today),
            one_day: true,
            named: false,
        },
    };
    let assumed = !days.named && !navigate;
    let range = |filters: PlanFilters| PlanFilters {
        from: days.from,
        to: days.to,
        ..filters
    };
    // A report of a person or a thing with no day named is the whole of the books (`whole_books`);
    // a report of a day (sales, cash book ...) takes today and says so.
    let dated = |id: &'static str, filters: PlanFilters, whole_books: bool| {
        if days.named {
            plan_of(id, range(filters), None, false)
        } else if whole_books || navigate {
            plan_of(id, filters, None, false)
        } else {
            plan_of(id, range(filters), None, true)
        }
    };
    let found: Vec<&PartyHit> = names
        .parties
        .iter()
        .filter_map(|p| match &p.outcome {
            Outcome::Found(h) | Outcome::FoundAs(h) => Some(h),
            _ => None,
        })
        .collect();
    let record = |kind: RecordKind| {
        names.records.iter().find_map(|r| match &r.outcome {
            Outcome::Found(h) if h.kind == kind => Some(h.clone()),
            _ => None,
        })
    };
    // A named question is never widened to a wider report (money review BUG-1, BUG-2): a name the
    // app cannot place, an item or a scope the links cannot filter by, and the report is the model's.
    // A staff member (a collector) is one of them: no report link has his filter.
    let blocked = input.unknown_words
        || input.scope_missing
        || names
            .records
            .iter()
            .any(|r| matches!(&r.outcome, Outcome::Found(h) if h.kind == RecordKind::Staff));
    let note = |report: ReportPlan| {
        if blocked {
            return none(Doubt::NoReport);
        }
        // A period report with no day named opens on its own default, today to today, and the steps
        // say so (lead ruling of 06-11-2026): it is a sure plan with `assumed_period`, not a doubt.
        Planned {
            report: Some(report),
            doubts: Vec::new(),
        }
    };

    // One بکری by its number (R1): the row was looked up in the books.
    if let Some(number) = input.bikri_number {
        return match input.bikri_row {
            Some(row) => note(plan_of(
                "R1",
                PlanFilters {
                    bikri: Some(row),
                    ..PlanFilters::default()
                },
                None,
                false,
            )),
            None => {
                let _ = number;
                none(Doubt::NoReport)
            }
        };
    }
    // An item named, or a group of items: a link with the item filter or the group filter (R7, R10,
    // R27, the registers and the item views).
    if !names.items.is_empty() || input.item_group.is_some() {
        let items: Vec<&ItemHit> = names
            .items
            .iter()
            .filter_map(|i| match &i.outcome {
                Outcome::Found(h) => Some(h),
                _ => None,
            })
            .collect();
        if items.len() != names.items.len() || items.len() > 1 || found.len() > 1 {
            return none(Doubt::NoReport);
        }
        let mut filters = PlanFilters {
            item: items.first().map(|i| (*i).clone()),
            item_group: input.item_group,
            ..PlanFilters::default()
        };
        if let Some(hit) = found.first() {
            match hit.role {
                PartyRole::Buyer => filters.party = Some((*hit).clone()),
                PartyRole::Seller => filters.seller = Some((*hit).clone()),
                _ => return none(Doubt::NoReport),
            }
        }
        let titled = input.kinds.iter().find_map(|k| match k {
            FigureKind::Report(id) => Some(*id),
            _ => None,
        });
        let sales_like = input.kinds.iter().all(|k| {
            matches!(
                k,
                FigureKind::Sale
                    | FigureKind::SaleCount
                    | FigureKind::Commission
                    | FigureKind::Laga
                    | FigureKind::DaySummary
                    | FigureKind::Purchases
            )
        });
        return match titled {
            // Registers and views that take an item.
            Some(id @ ("R3" | "R4" | "R7" | "R27" | "A" | "D" | "E" | "G")) => {
                note(dated(id, filters, is_view(id)))
            }
            Some(_) => none(Doubt::NoReport),
            // Goods left of one item: the stock report from the first day of the books to the day.
            None if input.kinds == [FigureKind::Stock] && found.is_empty() => note(plan_of(
                "R10",
                PlanFilters {
                    from: if days.named && !days.one_day {
                        days.from
                    } else {
                        input.first_day
                    },
                    to: Some(days.to.unwrap_or(today)),
                    ..filters
                },
                None,
                false,
            )),
            // The sales of one item: the item-wise sales report.
            None if sales_like => note(dated("R7", filters, false)),
            None => none(Doubt::NoReport),
        };
    }

    match found.as_slice() {
        [hit] => {
            let party = (*hit).clone();
            let with_party = || PlanFilters {
                party: Some(party.clone()),
                ..PlanFilters::default()
            };
            let ledger_like = input.statement || input.ledger || navigate;
            // A report named by its title, for one buyer or seller: the registers that take a party,
            // and the advances report of one seller.
            if let [FigureKind::Report(id)] = input.kinds
                && matches!(hit.role, PartyRole::Buyer | PartyRole::Seller)
            {
                return match (*id, hit.role) {
                    ("R3" | "R4" | "R7" | "R15" | "R23" | "R25" | "R28", _) => {
                        note(dated(id, with_party(), false))
                    }
                    // The views of one buyer (C) and of one seller (B, E).
                    ("C", PartyRole::Buyer) => note(dated(id, with_party(), true)),
                    ("B" | "E", PartyRole::Seller) => note(dated(
                        id,
                        PlanFilters {
                            seller: Some(party.clone()),
                            ..PlanFilters::default()
                        },
                        true,
                    )),
                    _ => none(Doubt::NoReport),
                };
            }
            if let [FigureKind::Report(id)] = input.kinds {
                let own = |filters: PlanFilters| note(dated(id, filters, true));
                match (*id, hit.role) {
                    ("F" | "G" | "J", PartyRole::Partner) => {
                        return own(PlanFilters {
                            partner: Some(party.clone()),
                            ..PlanFilters::default()
                        });
                    }
                    ("I", PartyRole::Garden) => {
                        return own(PlanFilters {
                            garden: Some(party.clone()),
                            ..PlanFilters::default()
                        });
                    }
                    ("J", PartyRole::Trip) => {
                        return own(PlanFilters {
                            trip: Some(party.clone()),
                            ..PlanFilters::default()
                        });
                    }
                    _ => return none(Doubt::NoReport),
                }
            }
            let wants = wants_of(input.kinds);
            let report = match hit.role {
                PartyRole::Buyer | PartyRole::Seller if input.statement => {
                    dated("R14", with_party(), true)
                }
                PartyRole::Buyer | PartyRole::Seller if ledger_like => {
                    dated("R13", with_party(), true)
                }
                // A figure the sale reports do not hold (a receipt, an advance, a payment): only his
                // khata holds it. Any other figure has no report of a party (money review R-g).
                PartyRole::Buyer | PartyRole::Seller if wants == Wants::Other => {
                    return none(Doubt::NoReport);
                }
                PartyRole::Buyer | PartyRole::Seller if wants == Wants::Khata => {
                    dated("R13", with_party(), true)
                }
                PartyRole::Buyer if days.one_day => plan_of(
                    "R35",
                    PlanFilters {
                        party: Some(party.clone()),
                        as_of: days.to,
                        ..PlanFilters::default()
                    },
                    None,
                    assumed,
                ),
                PartyRole::Buyer => plan_of(
                    "R5",
                    range(with_party()),
                    Some(dated("R13", with_party(), true)),
                    assumed,
                ),
                PartyRole::Seller => plan_of(
                    "R6",
                    range(PlanFilters {
                        seller: Some(party.clone()),
                        ..PlanFilters::default()
                    }),
                    Some(dated("R13", with_party(), true)),
                    assumed,
                ),
                PartyRole::Staff | PartyRole::Khata => dated(
                    if input.statement { "R14" } else { "R13" },
                    with_party(),
                    true,
                ),
                PartyRole::Partner => {
                    let filters = PlanFilters {
                        partner: Some(party.clone()),
                        ..PlanFilters::default()
                    };
                    if input.kinds.contains(&FigureKind::FruitTaken) {
                        dated("R37", filters, false)
                    } else if input.kinds.contains(&FigureKind::PartnerTaken) {
                        // The money one partner took or gave back: R29 for him, today if no day.
                        dated("R29", filters, false)
                    } else {
                        dated("R31", filters, true)
                    }
                }
                PartyRole::Garden => dated(
                    "R36",
                    PlanFilters {
                        garden: Some(party.clone()),
                        ..PlanFilters::default()
                    },
                    true,
                ),
                PartyRole::Trip => {
                    let filters = PlanFilters {
                        trip: Some(party.clone()),
                        ..PlanFilters::default()
                    };
                    match (days.named, input.trip_days) {
                        // No day named: the trip's own days, so the trip is in the list.
                        (false, Some((from, to))) => plan_of(
                            "R33",
                            PlanFilters {
                                from: Some(from),
                                to: Some(to),
                                ..filters
                            },
                            None,
                            false,
                        ),
                        _ => dated("R33", filters, true),
                    }
                }
            };
            note(report)
        }
        [] => {
            // A partner while the section is locked: the report opens on the code box.
            if input.partner_locked {
                return Planned {
                    report: Some(dated("R31", PlanFilters::default(), true)),
                    doubts: vec![Doubt::PartnerLocked],
                };
            }
            // "bagh", "safar" with no garden or trip named: the gardens or the trips report of the days.
            let titled_report = input
                .kinds
                .iter()
                .any(|k| matches!(k, FigureKind::Report(_)));
            if (input.garden_word || input.trip_word) && !titled_report {
                let fits = input.kinds.iter().all(|k| {
                    matches!(
                        k,
                        FigureKind::Sale
                            | FigureKind::Commission
                            | FigureKind::Profit
                            | FigureKind::Result
                            | FigureKind::Balance
                            | FigureKind::Expense
                            | FigureKind::Ledger
                            | FigureKind::Statement
                            | FigureKind::Purchases
                            | FigureKind::DaySummary
                            | FigureKind::Stock
                    )
                });
                // Both words in one question is two reports.
                if !fits || (input.garden_word && input.trip_word) {
                    return none(Doubt::NoReport);
                }
                let id = if input.garden_word { "R36" } else { "R33" };
                return note(dated(id, PlanFilters::default(), true));
            }
            // The shop's guests and the fruit: their register (R37 with guests only).
            if input.guests && input.kinds.contains(&FigureKind::FruitTaken) {
                return note(dated(
                    "R37",
                    PlanFilters {
                        guests: true,
                        ..PlanFilters::default()
                    },
                    false,
                ));
            }
            let dues_only = input.kinds.iter().all(|k| {
                matches!(
                    k,
                    FigureKind::Dues | FigureKind::Receipts | FigureKind::Balance
                )
            });
            let report = if let Some(adda) = record(RecordKind::Adda) {
                dated(
                    "R8",
                    PlanFilters {
                        adda: Some(adda),
                        ..PlanFilters::default()
                    },
                    false,
                )
            } else if let Some(city) = record(RecordKind::City) {
                if !dues_only {
                    return none(Doubt::NoReport);
                }
                plan_of(
                    "R21",
                    PlanFilters {
                        city: Some(city),
                        as_of: Some(days.to.unwrap_or(today)),
                        ..PlanFilters::default()
                    },
                    None,
                    false,
                )
            } else if let Some(head) = record(RecordKind::ExpenseHead) {
                dated(
                    "R29",
                    PlanFilters {
                        head: Some(head),
                        ..PlanFilters::default()
                    },
                    false,
                )
            } else if input.by_city && dues_only {
                plan_of(
                    "R21",
                    PlanFilters {
                        as_of: Some(days.to.unwrap_or(today)),
                        ..PlanFilters::default()
                    },
                    None,
                    false,
                )
            } else {
                // The title of a report wins over the figures beside it: "collector wise dues" is R22.
                let titled = input.kinds.iter().find_map(|k| match k {
                    FigureKind::Report(id) => Some(*id),
                    _ => None,
                });
                let shapes: Vec<Shape> = match titled {
                    Some(id) => report_shape(id).into_iter().collect(),
                    None => input.kinds.iter().filter_map(|k| shape_of(*k)).collect(),
                };
                if titled.is_none() && shapes.len() != input.kinds.len() {
                    return none(Doubt::NoReport);
                }
                if titled.is_some() && shapes.is_empty() {
                    return none(Doubt::NoReport);
                }
                let shape = match shapes.split_first() {
                    // No figure named: the sales of the day or the period.
                    None => Shape::Sales,
                    Some((first, rest)) if rest.iter().all(|s| s == first) => *first,
                    Some(_) => return none(Doubt::NoReport),
                };
                match shape {
                    Shape::Sales => {
                        if navigate {
                            return nothing();
                        }
                        // No party and no day: today's summary, and the steps say the day was taken.
                        if !days.named {
                            return note(plan_of("R34", range(PlanFilters::default()), None, true));
                        }
                        let id = if days.one_day { "R34" } else { "R2" };
                        plan_of(id, range(PlanFilters::default()), None, false)
                    }
                    Shape::Range(id) => dated(id, PlanFilters::default(), false),
                    // Stock now is from the first day of the books to the day. A range the
                    // question names is taken as it is.
                    Shape::Stock => plan_of(
                        "R10",
                        PlanFilters {
                            from: if days.named && !days.one_day {
                                days.from
                            } else {
                                input.first_day
                            },
                            to: Some(days.to.unwrap_or(today)),
                            ..PlanFilters::default()
                        },
                        None,
                        false,
                    ),
                    Shape::AsOf(id) => plan_of(
                        id,
                        PlanFilters {
                            as_of: Some(days.to.unwrap_or(today)),
                            ..PlanFilters::default()
                        },
                        None,
                        false,
                    ),
                    Shape::OneDay(id) => {
                        if days.named && !days.one_day {
                            return none(Doubt::NoPeriod);
                        }
                        dated(id, PlanFilters::default(), false)
                    }
                    Shape::NoFilters(id) => plan_of(id, PlanFilters::default(), None, false),
                    Shape::View(id) => dated(id, PlanFilters::default(), true),
                }
            };
            note(report)
        }
        _ => none(Doubt::NoReport),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::understand::{Confidence, PartyRef, PeriodKind};

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("a date")
    }

    fn period(kind: PeriodKind, from: NaiveDate, to: NaiveDate) -> PeriodRef {
        PeriodRef {
            kind,
            from: Some(from),
            to,
            confidence: Confidence::High,
            future: false,
            words: String::new(),
        }
    }

    fn trip_names() -> Names {
        let hit = PartyHit {
            key: "t2".into(),
            id: 2,
            role: PartyRole::Trip,
            code: "0070".into(),
            name_ur: "سفر 0070".into(),
            name_en: "0070 14-08-2026".into(),
            tag: None,
        };
        Names {
            parties: vec![PartyRef {
                text: "0070".into(),
                words: (1, 2),
                asked_role: Some(PartyRole::Trip),
                outcome: Outcome::Found(hit),
                confidence: Confidence::High,
            }],
            items: Vec::new(),
            records: Vec::new(),
            spans: Vec::new(),
            doubts: Vec::new(),
        }
    }

    fn input<'a>(names: &'a Names, periods: &'a [PeriodRef], kinds: &'a [FigureKind]) -> Input<'a> {
        Input {
            intent: Intent::Print,
            names,
            periods,
            kinds,
            today: "2026-11-05".into(),
            statement: false,
            ledger: false,
            by_city: false,
            partner_locked: false,
            first_day: Some(day(2026, 8, 31)),
            trip_days: Some((day(2026, 8, 14), day(2026, 10, 22))),
            unknown_words: false,
            scope_missing: false,
            garden_word: false,
            trip_word: false,
            guests: false,
            item_group: None,
            bikri_number: None,
            bikri_row: None,
        }
    }

    fn dates(plan: &ReportPlan) -> (Option<NaiveDate>, Option<NaiveDate>, Option<NaiveDate>) {
        (plan.filters.from, plan.filters.to, plan.filters.as_of)
    }

    /// A trip named with no day: R33 over the trip's own days, so the trip is in the list. The trip
    /// is carried for the steps (find its row), the screen has no filter for it.
    #[test]
    fn a_trip_with_no_day_named_takes_its_own_days() {
        let names = trip_names();
        let planned = build(&input(&names, &[], &[]));
        let plan = planned.report.expect("a report");
        assert_eq!(plan.id, "R33");
        assert_eq!(
            dates(&plan),
            (Some(day(2026, 8, 14)), Some(day(2026, 10, 22)), None)
        );
        assert!(plan.filters.trip.is_some());
        assert!(!plan.assumed_period);
    }

    /// The days the Owner named win over the trip's days.
    #[test]
    fn a_day_the_owner_named_wins_over_the_days_of_the_trip() {
        let names = trip_names();
        let named = [period(PeriodKind::Date, day(2026, 8, 14), day(2026, 8, 14))];
        let plan = build(&input(&names, &named, &[])).report.expect("a report");
        assert_eq!(
            dates(&plan),
            (Some(day(2026, 8, 14)), Some(day(2026, 8, 14)), None)
        );
    }

    /// Stock now runs from the first day of the books to today (R10 has `from` and `to`, no
    /// `as_of`). A day named is the end; a range named is taken as it is.
    #[test]
    fn stock_runs_from_the_first_day_of_the_books() {
        let none = Names {
            parties: Vec::new(),
            items: Vec::new(),
            records: Vec::new(),
            spans: Vec::new(),
            doubts: Vec::new(),
        };
        let stock = [FigureKind::Stock];
        let now = build(&input(&none, &[], &stock)).report.expect("a report");
        assert_eq!(now.id, "R10");
        assert_eq!(
            dates(&now),
            (Some(day(2026, 8, 31)), Some(day(2026, 11, 5)), None)
        );
        let one = [period(
            PeriodKind::Date,
            day(2026, 10, 10),
            day(2026, 10, 10),
        )];
        let until = build(&input(&none, &one, &stock)).report.expect("a report");
        assert_eq!(
            dates(&until),
            (Some(day(2026, 8, 31)), Some(day(2026, 10, 10)), None)
        );
        let range = [period(
            PeriodKind::Range,
            day(2026, 10, 1),
            day(2026, 10, 15),
        )];
        let within = build(&input(&none, &range, &stock))
            .report
            .expect("a report");
        assert_eq!(
            dates(&within),
            (Some(day(2026, 10, 1)), Some(day(2026, 10, 15)), None)
        );
    }
}
