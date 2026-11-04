//! What the question wants: its intent, the figures it asks for, and the checks that decide whether
//! the app is sure (plan 55 step 1; PRD FR-7.79, FR-7.88, FR-7.89).
//!
//! The order matters. Outside the app first, then an instruction to write, then advice, then a
//! printout, how-to, opening a screen, a forecast or a what-if, and last a list or a figure. A figure
//! question is never help: "faisal ka bakaya kitna hai" has a party and a figure word, so it is a
//! figure. An instruction to write ("likh do", "5000 jama karo") is never a figure or a printout.
//!
//! Example: "wasooli kaise likhun" is how-to (topic `receipt`); "akram ki 5000 wasooli likh do" is a
//! change request; "sale kaise barhayen" is advice.

use chrono::NaiveDate;

use super::change;
use super::lexicon::{Cue, Hit};
use super::plan;
use super::scan::Scan;
use super::who::Names;
use super::{
    Assumed, Confidence, Doubt, FigureKind, Intent, ListAsk, ListOver, Outcome, OutsideKind, Part,
    PartyRole, PeriodRef, Rank, Reader, Understanding,
};
use crate::help_guard;
use crate::model_answer::EntryScreenName;
use crate::prompt::guide;
use crate::tools::cues;

/// An understanding of a question that could not be read.
pub(super) fn nothing(doubt: Doubt) -> Understanding {
    Understanding {
        intent: Part {
            value: Intent::Unclear,
            confidence: Confidence::Low,
        },
        parties: Vec::new(),
        items: Vec::new(),
        records: Vec::new(),
        periods: Vec::new(),
        assumed: Vec::new(),
        outside: None,
        figures: Part {
            value: Vec::new(),
            confidence: Confidence::Low,
        },
        two_figures: false,
        sum_asked: false,
        compare: false,
        list: None,
        report: None,
        topic: None,
        screen: None,
        wants_steps: false,
        scoped: false,
        sure: false,
        doubts: vec![doubt],
    }
}

fn part<T>(value: T, confidence: Confidence) -> Part<T> {
    Part { value, confidence }
}

// ---- the figures --------------------------------------------------------------------------------------

/// What the figures depend on besides the cue: who and what is named, and whether a day is.
struct Named<'a> {
    roles: &'a [PartyRole],
    has_item: bool,
    has_period: bool,
}

fn figure_of(hit: &Hit, scan: &Scan, named: &Named<'_>) -> Option<FigureKind> {
    let roles = named.roles;
    let has_party = !roles.is_empty();
    let account_party = roles
        .iter()
        .any(|r| matches!(r, PartyRole::Buyer | PartyRole::Seller));
    Some(match hit.cue {
        Cue::Balance => FigureKind::Balance,
        Cue::Sale => FigureKind::Sale,
        Cue::SalePlural => FigureKind::SaleCount,
        Cue::Commission => FigureKind::Commission,
        Cue::Laga => FigureKind::Laga,
        Cue::Dues => {
            if has_party {
                FigureKind::Balance
            } else {
                FigureKind::Dues
            }
        }
        // "wasooli" / "ugrahi": what is still to collect, or what was collected. A word for
        // something outstanding says the first; "hui", "mili", "di" or a day says the second.
        Cue::Collection => {
            let outstanding = scan.has_any(&[Cue::Pending, Cue::Balance, Cue::Dues]);
            if outstanding && has_party {
                FigureKind::Balance
            } else if outstanding {
                FigureKind::Dues
            } else if scan.has(Cue::Received) || named.has_period {
                FigureKind::Receipts
            } else if account_party {
                FigureKind::Balance
            } else {
                FigureKind::Dues
            }
        }
        Cue::Payable => {
            if has_party {
                FigureKind::Balance
            } else {
                FigureKind::Payable
            }
        }
        Cue::Advance => FigureKind::Advance,
        Cue::Stock => FigureKind::Stock,
        Cue::Cash => FigureKind::Cash,
        Cue::Capital => FigureKind::Capital,
        Cue::PartnerTotal => FigureKind::PartnerTotal,
        Cue::PartnerShare => FigureKind::PartnerShare,
        Cue::Profit => FigureKind::Profit,
        Cue::Expense => FigureKind::Expense,
        Cue::FruitTaken => FigureKind::FruitTaken,
        // A person who borrowed is a staff account; a buyer or a seller "ka udhar" is his balance.
        Cue::MoneyLent => {
            if account_party && !roles.contains(&PartyRole::Staff) {
                FigureKind::Balance
            } else {
                FigureKind::MoneyLent
            }
        }
        Cue::DeadAmount => FigureKind::DeadAmount,
        Cue::Rate => FigureKind::Rate,
        // "faisal ka aaj ka hisab" is his ledger for the day, not the shop's day summary.
        Cue::DaySummary => {
            if has_party {
                FigureKind::Ledger
            } else {
                FigureKind::DaySummary
            }
        }
        Cue::Ledger => FigureKind::Ledger,
        Cue::Statement => FigureKind::Statement,
        Cue::DayBook => FigureKind::DayBook,
        Cue::TrialBalance => FigureKind::TrialBalance,
        Cue::ResultWord => FigureKind::Result,
        // "baqi" alone is a balance; beside goods, a unit or an item it is stock.
        Cue::Pending => {
            if scan.has_any(&[Cue::Goods, Cue::Stock, Cue::Unit])
                || (named.has_item && !roles.contains(&PartyRole::Buyer))
            {
                FigureKind::Stock
            } else {
                FigureKind::Balance
            }
        }
        // "paise aaye": money received. "akram ke kitne paise": his balance.
        Cue::Money => {
            if scan.has(Cue::Received) {
                FigureKind::Receipts
            } else if account_party && scan.has(Cue::HowMuch) {
                FigureKind::Balance
            } else {
                return None;
            }
        }
        _ => return None,
    })
}

/// How plainly a cue names a figure: 0 a word that names one, 1 a word that names one only when the
/// question says no more (collecting, money), 2 a word that only qualifies or points at a book
/// ("baqi", "khata"). Only the plainest tier a question has is read.
fn tier(cue: Cue) -> u8 {
    match cue {
        Cue::Collection | Cue::Money => 1,
        Cue::Pending | Cue::Ledger => 2,
        _ => 0,
    }
}

/// The figures asked for, in the order of the question, and whether two were asked in one breath.
fn figures(scan: &Scan, named: &Named<'_>) -> (Vec<FigureKind>, bool) {
    let roles = named.roles;
    let has_party = !roles.is_empty();
    let mut found: Vec<(usize, FigureKind)> = Vec::new();
    let best = scan
        .hits
        .iter()
        .filter(|h| figure_of(h, scan, named).is_some())
        .map(|h| tier(h.cue))
        .min();
    for hit in &scan.hits {
        if Some(tier(hit.cue)) != best {
            continue;
        }
        if let Some(kind) = figure_of(hit, scan, named)
            && !found.iter().any(|(_, k)| *k == kind)
        {
            found.push((hit.start, kind));
        }
    }
    // What a buyer bought or a seller sold, from the verb, when no figure word is there.
    if found.is_empty() {
        if scan.has_exact(Cue::Bought) && !scan.has_exact(Cue::Sold) {
            found.push((0, FigureKind::Purchases));
        } else if scan.has_exact(Cue::Sold) {
            found.push((0, FigureKind::Sale));
        }
    }
    // "usman shirakat daar ka kitna hai abhi": a partner and "kitna" ask for his total.
    if found.is_empty() && scan.has(Cue::HowMuch) && roles == [PartyRole::Partner] {
        found.push((0, FigureKind::PartnerTotal));
    }
    // "byopariyon ko kitna dena hai": what the shop owes sellers. "sab gahakon ka bakaya": the dues.
    if !has_party {
        for (_, kind) in &mut found {
            if *kind == FigureKind::Balance {
                if scan.has(Cue::ManySellers) {
                    *kind = FigureKind::Payable;
                } else if scan.has_any(&[Cue::ManyBuyers, Cue::Total]) {
                    *kind = FigureKind::Dues;
                }
            }
        }
    }
    found.sort_by_key(|(at, _)| *at);
    let kinds: Vec<FigureKind> = found.iter().map(|(_, k)| *k).collect();
    let two = match found.as_slice() {
        [(first, _), (second, _)] => {
            let between = |cue: Cue| {
                scan.hits
                    .iter()
                    .any(|h| h.cue == cue && h.start > *first && h.start < *second)
            };
            second.saturating_sub(*first) <= 3
                && (between(Cue::And) || second.saturating_sub(*first) == 1)
        }
        _ => false,
    };
    (kinds, two)
}

// ---- intent -------------------------------------------------------------------------------------------

/// The signs the rules below read.
struct Signs {
    how: bool,
    how_much: bool,
    party_found: bool,
    item_found: bool,
    period: bool,
    figures: bool,
    amount: bool,
}

fn has_amount(scan: &Scan) -> bool {
    use mandi_services::domain::figures::NumberUnit;
    scan.numbers
        .iter()
        .any(|n| n.unit == NumberUnit::Amount || (n.unit == NumberUnit::Plain && n.number >= 100))
}

fn percent_numbers(scan: &Scan) -> usize {
    use mandi_services::domain::figures::NumberUnit;
    scan.numbers
        .iter()
        .filter(|n| n.unit == NumberUnit::Percent)
        .count()
}

/// The words "kya" and "کیا": what.
fn says_what(scan: &Scan) -> bool {
    scan.words
        .iter()
        .any(|w| w == "kya" || w == "کیا" || w == "what")
}

/// True when the question has a word that asks (kya, kab, kaun, kitna, how, what, a question mark).
fn asks_a_question(scan: &Scan) -> bool {
    const ASKING: [&str; 21] = [
        "kya", "kab", "kaun", "kon", "kis", "kisne", "kyun", "kyon", "kahan", "kitna", "kitni",
        "kitne", "what", "when", "who", "which", "how", "why", "where", "کیا", "کب",
    ];
    scan.raw.contains('?')
        || scan.raw.contains('؟')
        || scan.words.iter().any(|w| ASKING.contains(&w.as_str()))
        || scan.has_any(&[
            Cue::HowMuch,
            Cue::Many,
            Cue::Show,
            Cue::How,
            Cue::Where,
            Cue::Define,
        ])
}

fn is_printish(scan: &Scan) -> bool {
    scan.has_exact(Cue::Print)
        || (scan.has_exact(Cue::PrintObject)
            && scan.has_exact(Cue::PrintVerb)
            && !scan.has(Cue::HowMuch))
}

/// "kis buyer", "kaun sa item", "which seller": a question about which one. Its words are a list.
fn which_one(scan: &Scan) -> bool {
    const WHICH: [&str; 8] = [
        "kis", "kaun", "kon", "konsa", "kaunsa", "which", "کس", "کون",
    ];
    scan.hits.iter().any(|h| {
        matches!(h.cue, Cue::RoleBuyer | Cue::RoleSeller | Cue::RoleItem)
            && !h.fuzzy
            && h.start
                .checked_sub(1)
                .and_then(|b| scan.word(b))
                .is_some_and(|prev| {
                    WHICH.contains(&prev)
                        || (matches!(prev, "sa" | "si" | "se" | "سا" | "سی")
                            && h.start
                                .checked_sub(2)
                                .and_then(|b| scan.word(b))
                                .is_some_and(|p| WHICH.contains(&p)))
                })
    })
}

/// What a question outside the books is about, if it is outside: the weather, a tax, a bank balance,
/// a rate in another market, the dollar or gold, or anything else the app does not keep.
fn outside_kind(scan: &Scan) -> Option<OutsideKind> {
    let kinds = [
        (Cue::ExternalWeather, OutsideKind::Weather),
        (Cue::ExternalTax, OutsideKind::Tax),
        (Cue::ExternalBank, OutsideKind::Bank),
        (Cue::ExternalPrices, OutsideKind::Prices),
    ];
    if let Some((_, kind)) = kinds.iter().find(|(cue, _)| scan.has_exact(*cue)) {
        return Some(*kind);
    }
    if scan.has_exact(Cue::MarketRate)
        || (scan.has(Cue::Rate)
            && scan.has(Cue::ExternalPlace)
            && !scan.has_any(&[Cue::Sale, Cue::Commission]))
    {
        return Some(OutsideKind::MarketRate);
    }
    scan.has_exact(Cue::External).then_some(OutsideKind::Other)
}

/// What kind of list, and in what order.
fn list_ask(scan: &Scan) -> ListAsk {
    let over = if scan.has(Cue::ManyItems) || scan.has(Cue::RoleItem) {
        Some(ListOver::Items)
    } else if scan.has(Cue::ManyBuyers) || scan.has(Cue::RoleBuyer) {
        Some(ListOver::Buyers)
    } else if scan.has(Cue::ManySellers) || scan.has(Cue::RoleSeller) {
        Some(ListOver::Sellers)
    } else if scan.has(Cue::Many) {
        Some(ListOver::Parties)
    } else {
        None
    };
    let rank = if scan.has(Cue::Most) {
        Some(Rank::Most)
    } else if scan.has(Cue::Least) {
        Some(Rank::Least)
    } else {
        None
    };
    let n = scan
        .words
        .iter()
        .enumerate()
        .filter(|(_, w)| matches!(w.as_str(), "top" | "ٹاپ"))
        .find_map(|(at, _)| scan.digits_at(at.saturating_add(1)))
        .and_then(|n| u32::try_from(n).ok());
    ListAsk { over, rank, n }
}

struct Chosen {
    intent: Part<Intent>,
    doubts: Vec<Doubt>,
}

fn chosen(intent: Intent, confidence: Confidence) -> Chosen {
    Chosen {
        intent: part(intent, confidence),
        doubts: Vec::new(),
    }
}

/// The intent, by the order of Section 2 of the module documentation.
fn intent_of(scan: &Scan, signs: &Signs, periods: &[PeriodRef]) -> Chosen {
    let how_like = signs.how || scan.has_any(&[Cue::Where, Cue::Define, Cue::Capability]);
    // Outside the app: only when no party of the books is named.
    let outside = outside_kind(scan).is_some();
    if outside && !signs.party_found {
        return chosen(Intent::NotInApp, Confidence::High);
    }
    // An instruction to write.
    let strong_write = scan.has_exact(Cue::WriteStrong);
    // A percent is also something to write ("commission 5 percent kar do"), unless it is asked
    // under a condition ("agar commission 8 percent kar den to").
    let percent_to_write = percent_numbers(scan) > 0 && !cues::conditional(&scan.raw);
    let weak_write = scan.has_exact(Cue::WriteWeak) && (signs.amount || percent_to_write);
    if (strong_write || weak_write) && !how_like && !signs.how_much && !is_printish(scan) {
        return chosen(Intent::ChangeRequest, Confidence::High);
    }
    // A payment told and not asked ("akram ne 5000 diye"): something to write, not to read.
    if signs.amount
        && signs.party_found
        && scan.has(Cue::Received)
        && !asks_a_question(scan)
        && !is_printish(scan)
    {
        return chosen(Intent::ChangeRequest, Confidence::Medium);
    }
    // Advice: a way to get better, or a decision to be advised on.
    let asks_advice = scan.has(Cue::Decision)
        || scan.has(Cue::AdviceStrong)
        || (scan.has(Cue::Advice) && (signs.how || scan.has(Cue::Capability)));
    if asks_advice && !signs.how_much {
        let mut c = chosen(Intent::Advice, Confidence::High);
        if signs.figures && (signs.party_found || signs.item_found) && scan.has(Cue::HowMuch) {
            c.doubts.push(Doubt::MixedIntent);
        }
        return c;
    }
    if asks_advice {
        // "sale kaise barhayen, aur kele ka commission kitna hai": two questions in one.
        let mut c = chosen(Intent::Advice, Confidence::Low);
        c.doubts.push(Doubt::MixedIntent);
        return c;
    }
    // A printout, with a party or a period or a figure to print. "How do I print a bill" is help.
    if is_printish(scan) {
        let thing = signs.party_found || signs.period || signs.figures;
        if thing {
            return chosen(Intent::Print, Confidence::High);
        }
        if how_like {
            return chosen(Intent::HowTo, Confidence::High);
        }
        return chosen(Intent::Print, Confidence::Low);
    }
    // How to do it, where it is, what it means. Never when a figure is asked ("kitna").
    if how_like && !signs.how_much {
        let party_with_figure = signs.party_found && signs.figures && !signs.how;
        if !party_with_figure {
            let weak_only = !signs.how
                && !scan.has_any(&[Cue::Where, Cue::Capability])
                && !scan.has(Cue::Define);
            return chosen(
                Intent::HowTo,
                if weak_only {
                    Confidence::Medium
                } else {
                    Confidence::High
                },
            );
        }
    }
    if scan.has(Cue::DefineWeak) && !signs.party_found && !signs.figures && !signs.how_much {
        return chosen(Intent::HowTo, Confidence::Medium);
    }
    // Open a screen or a report.
    let open = scan.has_exact(Cue::Open)
        || (scan.has_exact(Cue::Show) && !signs.party_found && !signs.figures && !signs.how_much);
    if open && !signs.how_much {
        return chosen(Intent::Navigate, Confidence::High);
    }
    // Ahead in time, or under a condition.
    let conditional = cues::conditional(&scan.raw);
    let scenario = cues::scenario(&scan.raw).is_some()
        || cues::percent_change(&scan.raw, percent_numbers(scan)).is_some();
    if conditional && (scenario || signs.figures) {
        return chosen(
            Intent::WhatIf,
            if scenario {
                Confidence::High
            } else {
                Confidence::Medium
            },
        );
    }
    let future_period = periods.iter().any(|p| p.future);
    // "agle hafte ka andaza": an estimate is asked, but of what is not said.
    if scan.has(Cue::ForecastStrong) && !signs.figures && !signs.how_much {
        return chosen(Intent::Forecast, Confidence::Medium);
    }
    if (scan.has_any(&[Cue::Forecast, Cue::ForecastStrong])
        || future_period
        || cues::year_end(&scan.raw)
        || cues::year_ahead(&scan.raw))
        && (signs.figures || signs.how_much)
    {
        return chosen(Intent::Forecast, Confidence::High);
    }
    // A list.
    let kya_verb = says_what(scan)
        && (scan.has_exact(Cue::Sold) || scan.has_exact(Cue::Bought))
        && !scan.has(Cue::Total);
    let ranked = scan.has(Cue::Most) || scan.has(Cue::Least);
    // "buyers ka total bakaya" is one figure, not a list of buyers.
    let many = (scan.has(Cue::Many)
        || which_one(scan)
        || scan.has(Cue::ManyBuyers)
        || scan.has(Cue::ManySellers)
        || scan.has(Cue::ManyItems))
        && !signs.how_much
        && !scan.has(Cue::Total);
    let show_ledger =
        scan.has_exact(Cue::Show) && scan.has_any(&[Cue::Ledger, Cue::Statement, Cue::Bought]);
    if ranked
        || many
        || kya_verb
        || show_ledger
        || (scan.has(Cue::Ledger) && !signs.how_much && !signs.figures_except_ledger(scan))
    {
        return chosen(
            Intent::List,
            if signs.figures || ranked || kya_verb {
                Confidence::High
            } else {
                Confidence::Medium
            },
        );
    }
    // One figure (or two).
    if signs.figures {
        let clear = signs.how_much || signs.party_found || signs.item_found || signs.period;
        return chosen(
            Intent::Figure,
            if clear {
                Confidence::High
            } else {
                Confidence::Medium
            },
        );
    }
    if signs.how_much {
        return chosen(Intent::Figure, Confidence::Low);
    }
    chosen(Intent::Unclear, Confidence::Low)
}

impl Signs {
    /// True when a figure other than the ledger itself is asked.
    fn figures_except_ledger(&self, scan: &Scan) -> bool {
        self.figures
            && scan.hits.iter().any(|h| {
                matches!(
                    h.cue,
                    Cue::Balance
                        | Cue::Sale
                        | Cue::Commission
                        | Cue::Dues
                        | Cue::Payable
                        | Cue::Stock
                        | Cue::Cash
                )
            })
    }
}

// ---- the topic of the guide ---------------------------------------------------------------------------

/// The topic of the guide the question is about: the guide's own word table first (the one the help
/// guard uses), then the figure the question names.
fn topic_of(scan: &Scan) -> Option<Part<&'static str>> {
    let topics = &guide().topics;
    // A "kaise" is added so the guard reads a "kholo" or a "kya hai" question as a question about a
    // screen too. A number would make it a figure question, so none is passed.
    let asked = format!("{} kaise", scan.raw);
    if let Some(topic) = help_guard::topic_of_question(&asked, &[])
        && topics.contains_key(topic)
    {
        return Some(part(topic, Confidence::High));
    }
    let by_figure: &[(Cue, &str)] = &[
        (Cue::Print, "printing"),
        (Cue::Stock, "unsold_stock"),
        (Cue::Cash, "cash_book"),
        (Cue::Capital, "partner_capital"),
        (Cue::MoneyLent, "money_lent"),
        (Cue::DeadAmount, "dead_amount"),
        (Cue::Advance, "advance"),
        (Cue::Payable, "payment"),
        (Cue::Dues, "receipt"),
        (Cue::Expense, "shop_expense"),
        (Cue::FruitTaken, "fruit_taken"),
        (Cue::RoleKhata, "separate_khata"),
        (Cue::RoleGarden, "gardens"),
        (Cue::RoleTrip, "market_trips"),
        (Cue::RolePartner, "partners"),
        (Cue::Sale, "bikri_book"),
    ];
    let mut named: Vec<&'static str> = by_figure
        .iter()
        .filter(|(cue, _)| scan.has_exact(*cue))
        .map(|(_, topic)| *topic)
        .filter(|topic| topics.contains_key(*topic))
        .collect();
    named.dedup();
    match named.as_slice() {
        [one] => Some(part(*one, Confidence::Medium)),
        _ => None,
    }
}

/// The entry screen name of the answer schema whose topic is this one.
pub(super) fn screen_of(topic: &str) -> Option<&'static str> {
    EntryScreenName::ALL.iter().copied().find(|name| {
        serde_json::from_value::<EntryScreenName>(serde_json::json!(name))
            .is_ok_and(|screen| help_guard::topic_of(screen) == topic)
    })
}

/// What a report plan needs from the books besides the names: the first day of the books when stock
/// is asked (stock now runs from there), and the days of a named trip (its day to the day it closed,
/// or to today). Reads only, and only for a print or an open-the-report request.
fn plan_days(
    reader: &Reader<'_>,
    intent: Intent,
    names: &Names,
    kinds: &[FigureKind],
) -> (Option<NaiveDate>, Option<(NaiveDate, NaiveDate)>) {
    if !matches!(intent, Intent::Print | Intent::Navigate) {
        return (None, None);
    }
    let iso = |text: &str| NaiveDate::parse_from_str(text, "%Y-%m-%d").ok();
    let first_day = kinds.contains(&FigureKind::Stock).then(|| {
        mandi_services::day::first_book_day(reader.ctx)
            .ok()
            .flatten()
            .and_then(|d| iso(&d.to_iso()))
    });
    let trip_days = names
        .parties
        .iter()
        .find_map(|p| p.outcome.hit().filter(|h| h.role == PartyRole::Trip))
        .and_then(|hit| {
            let input = mandi_services::core::trips::TripGetInput {
                id: hit.id,
                entries_offset: 0,
                entries_limit: 1,
            };
            let row = mandi_services::trips::get(reader.ctx, &input).ok()?.row;
            let from = iso(&row.entry_date)?;
            let to = row
                .closed_date
                .as_deref()
                .and_then(iso)
                .or_else(|| iso(&reader.today.to_iso()))?;
            Some((from, to.max(from)))
        });
    (first_day.flatten(), trip_days)
}

// ---- assembling ---------------------------------------------------------------------------------------

/// True when the figure needs a named person to be answered.
fn needs_party(kinds: &[FigureKind]) -> bool {
    kinds.iter().any(|k| {
        matches!(
            k,
            FigureKind::Balance
                | FigureKind::Purchases
                | FigureKind::Ledger
                | FigureKind::Statement
                | FigureKind::Result
        )
    })
}

/// Puts the understanding together: names, periods, figures, intent, the report or the topic, and
/// whether the app is sure.
pub(super) fn assemble(
    reader: &Reader<'_>,
    scan: &Scan,
    names: Names,
    periods: Vec<PeriodRef>,
) -> Understanding {
    let roles: Vec<PartyRole> = names
        .parties
        .iter()
        .filter_map(|p| p.outcome.hit().map(|h| h.role))
        .collect();
    let named = Named {
        roles: &roles,
        has_item: names.items.iter().any(|i| i.outcome.hit().is_some()),
        has_period: !periods.is_empty(),
    };
    let (kinds, two_figures) = figures(scan, &named);
    let signs = Signs {
        how: scan.has(Cue::How),
        how_much: scan.has(Cue::HowMuch),
        party_found: names
            .parties
            .iter()
            .any(|p| p.outcome.hit().is_some() || matches!(p.outcome, Outcome::Several(_))),
        item_found: names.items.iter().any(|i| i.outcome.hit().is_some()),
        period: !periods.is_empty(),
        figures: !kinds.is_empty(),
        amount: has_amount(scan),
    };
    let Chosen { intent, mut doubts } = intent_of(scan, &signs, &periods);
    doubts.extend(names.doubts.iter().copied());
    if scan.cut {
        doubts.push(Doubt::Cut);
    }
    // A request to write that also asks to read: the refusal would drop the question.
    if intent.value == Intent::ChangeRequest && change::asks_to_read(scan) {
        doubts.push(Doubt::MixedIntent);
    }
    let figure_confidence = if kinds.is_empty() {
        Confidence::Low
    } else if scan.hits.iter().any(|h| h.fuzzy) && !scan.hits.iter().any(|h| !h.fuzzy) {
        Confidence::Medium
    } else {
        Confidence::High
    };
    let compare = cues::comparing(&scan.raw);
    let sum_asked = cues::together(&scan.raw);
    if periods.len() > 1 && !matches!(intent.value, Intent::Forecast) {
        doubts.push(Doubt::SeveralPeriods);
    }
    if periods.iter().any(|p| p.confidence == Confidence::Low) {
        doubts.push(Doubt::TenseUnclear);
    }
    if compare {
        doubts.push(Doubt::Comparison);
    }
    let topic = matches!(intent.value, Intent::HowTo | Intent::Navigate)
        .then(|| topic_of(scan))
        .flatten();
    let screen = match intent.value {
        Intent::Navigate => topic.as_ref().and_then(|t| screen_of(t.value)),
        // The entry screen of a request to write (the assistant never writes, PRD FR-7.72).
        Intent::ChangeRequest => change::screen_of(scan, &names, topic_of(scan).map(|t| t.value)),
        _ => None,
    };
    let list = (intent.value == Intent::List).then(|| list_ask(scan));
    let wants_steps = intent.value == Intent::Print && scan.has(Cue::How);
    let (first_day, trip_days) = plan_days(reader, intent.value, &names, &kinds);
    let planned = plan::build(&plan::Input {
        intent: intent.value,
        names: &names,
        periods: &periods,
        kinds: &kinds,
        today: reader.today.to_iso(),
        statement: scan.has(Cue::Statement),
        ledger: scan.has(Cue::Ledger),
        by_city: scan.has(Cue::ByCity),
        partner_locked: reader.partner_locked
            && (scan.has_exact(Cue::RolePartner)
                || kinds.iter().any(|k| {
                    matches!(
                        k,
                        FigureKind::Capital | FigureKind::PartnerTotal | FigureKind::PartnerShare
                    )
                })),
        first_day,
        trip_days,
    });
    doubts.extend(planned.doubts.iter().copied());

    let mut understanding = Understanding {
        intent,
        parties: names.parties,
        items: names.items,
        records: names.records,
        periods,
        assumed: Vec::new(),
        outside: None,
        figures: part(kinds, figure_confidence),
        two_figures,
        sum_asked,
        compare,
        list,
        report: planned.report,
        topic,
        screen,
        wants_steps,
        scoped: super::reads::scoped(scan),
        sure: false,
        doubts,
    };
    if understanding.intent.value == Intent::NotInApp {
        understanding.outside = outside_kind(scan);
    }
    assume_ranking(&mut understanding, reader);
    // Several things and several figures at once is a table, not an answer.
    let several_things = understanding.items.len().max(understanding.parties.len()) >= 2;
    if several_things && understanding.figures.value.len() >= 2 {
        understanding.doubts.push(Doubt::TwoParts);
    }
    // A rate is the rate of an item.
    if matches!(understanding.intent.value, Intent::Figure)
        && understanding.figures.value.contains(&FigureKind::Rate)
        && understanding.items.is_empty()
    {
        understanding.doubts.push(Doubt::NoParty);
    }
    // A figure about someone needs the someone.
    if matches!(understanding.intent.value, Intent::Figure)
        && needs_party(&understanding.figures.value)
        && understanding.parties.is_empty()
    {
        understanding.doubts.push(Doubt::NoParty);
    }
    let no_measure = understanding.figures.value.is_empty()
        && match understanding.intent.value {
            Intent::Figure | Intent::Forecast => true,
            // "mera sab se bara buyer": the biggest by what?
            Intent::List => understanding
                .list
                .is_none_or(|l| l.over.is_none() || l.rank.is_some()),
            _ => false,
        };
    if no_measure {
        understanding.doubts.push(Doubt::NoFigure);
    }
    understanding.sure = is_sure(&understanding);
    // A question that is not sure never carries a button: a wrong, wider date on a button is worse
    // than no button. It goes to the model path (lead ruling of 04-11-2026).
    if !understanding.sure {
        understanding.report = None;
    }
    understanding.doubts.dedup();
    understanding
}

/// True when every name is understood well enough to answer without the model: found for sure, or
/// plainly not in the books.
fn names_ok(u: &Understanding) -> bool {
    u.parties.iter().all(|p| match p.outcome {
        Outcome::NotFound => p.confidence == Confidence::High,
        Outcome::Several(_) => false,
        _ => p.confidence >= Confidence::Medium,
    }) && u.items.iter().all(|i| match i.outcome {
        Outcome::NotFound => i.confidence == Confidence::High,
        Outcome::Several(_) => false,
        _ => i.confidence >= Confidence::Medium,
    }) && u.records.iter().all(|r| match r.outcome {
        Outcome::NotFound | Outcome::Several(_) => false,
        _ => r.confidence >= Confidence::Medium,
    })
}

/// True for a figure that is a flow over days (a sum of a period), so a ranking of it needs a period.
fn is_flow(kind: FigureKind) -> bool {
    matches!(
        kind,
        FigureKind::Sale
            | FigureKind::SaleCount
            | FigureKind::Commission
            | FigureKind::Laga
            | FigureKind::Purchases
            | FigureKind::Profit
            | FigureKind::Expense
            | FigureKind::Receipts
            | FigureKind::FruitTaken
    )
}

/// A ranking with no measure ranks by total commission (view H, PRD 18.2); with no period it takes
/// this year. The understanding says so in `assumed`, so the answer can state it (plan 55 ruling 2).
///
/// Example: "top 5 buyers" is the 5 buyers with the most commission this year.
fn assume_ranking(u: &mut Understanding, reader: &Reader<'_>) {
    let Some(ask) = u.list.filter(|l| l.rank.is_some()) else {
        return;
    };
    if !matches!(
        ask.over,
        Some(ListOver::Items | ListOver::Buyers | ListOver::Sellers)
    ) {
        return;
    }
    if u.figures.value.is_empty() {
        u.figures = part(vec![FigureKind::Commission], Confidence::Medium);
        u.assumed.push(Assumed::Measure);
    }
    let flow = u.figures.value.iter().all(|k| is_flow(*k));
    if flow
        && u.periods.is_empty()
        && let Some(year) = reader.today_date().and_then(super::when::this_year)
    {
        u.periods.push(year);
        u.assumed.push(Assumed::Period);
    }
}

/// Doubts that always send the question to the model.
const BLOCKING: [Doubt; 11] = [
    Doubt::NothingToRead,
    Doubt::SeveralMatches,
    Doubt::NameOrItem,
    Doubt::MixedIntent,
    Doubt::LookupFailed,
    Doubt::Cut,
    Doubt::TenseUnclear,
    Doubt::NoFigure,
    Doubt::NoParty,
    Doubt::TwoParts,
    Doubt::OtherRecord,
];

fn is_sure(u: &Understanding) -> bool {
    if u.doubts.iter().any(|d| BLOCKING.contains(d)) || !names_ok(u) {
        return false;
    }
    if u.intent.confidence < Confidence::Medium {
        return false;
    }
    let one_period = u.periods.len() <= 1;
    let party_locked = u.doubts.contains(&Doubt::PartnerLocked);
    match u.intent.value {
        Intent::Unclear => false,
        Intent::Figure | Intent::List => {
            one_period
                && !party_locked
                && !u.compare
                && (!u.figures.value.is_empty() || u.list.is_some_and(|l| l.over.is_some()))
        }
        Intent::Print | Intent::Navigate => {
            // A name that is not in the books is answered too: there is no such party.
            let named_ok =
                u.report.is_some() || u.screen.is_some() || u.topic.is_some() || u.has_not_found();
            // A partner's report while the section is locked opens on the code box: that is the
            // answer, so it is sure.
            let locked_ok = !party_locked || u.report.as_ref().is_some_and(|r| r.id == "R31");
            named_ok
                && one_period
                && !u.doubts.contains(&Doubt::NoPeriod)
                && !u.doubts.contains(&Doubt::NoReport)
                && locked_ok
        }
        Intent::HowTo => u
            .topic
            .as_ref()
            .is_some_and(|t| t.confidence >= Confidence::Medium),
        Intent::Forecast | Intent::WhatIf => !party_locked,
        Intent::Advice | Intent::ChangeRequest | Intent::NotInApp => true,
    }
}
