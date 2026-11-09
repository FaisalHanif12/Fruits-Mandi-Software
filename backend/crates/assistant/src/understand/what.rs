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
use super::lexicon::{Cue, Hit, key};
use super::plan;
use super::scan::Scan;
use super::who::Names;
use super::{
    Assumed, Confidence, Doubt, FigureKind, Intent, ListAsk, ListOver, Measure, Outcome,
    OutsideKind, Part, PartyRole, PeriodRef, Rank, Reader, Understanding,
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
        lower_ask: false,
        period_merged: false,
        sum_asked: false,
        compare: false,
        list: None,
        report: None,
        topic: None,
        screen: None,
        wants_steps: false,
        scoped: false,
        guests: false,
        quality: false,
        grade: None,
        capital_flow: None,
        cash_dirs: Vec::new(),
        trial_sides: Vec::new(),
        sale_number: None,
        garden_goods: false,
        last_one: false,
        months_asked: false,
        months_range: None,
        oldest: false,
        sum_strong: false,
        amount_named: false,
        item_group: None,
        bikri_number: None,
        truck: None,
        happened: false,
        other_measure: Measure::Same,
        commission_side: None,
        buyer_rate: false,
        document_number: false,
        packing_filter: false,
        asks_count: false,
        count_asks: 0,
        how_much_cues: 0,
        qty_asked: false,
        average_asked: false,
        pay_kind: None,
        pay_asked: false,
        unexplained: false,
        role_word: false,
        seller_word: false,
        buyer_word: false,
        open_ask: false,
        figure_at: Vec::new(),
        name_spans: Vec::new(),
        period_between: false,
        shop_after_and: false,
        multi_measure: super::MULTI_MEASURE_INSTANT,
        stray_screen_word: false,
        shop_word: false,
        how_much_twice: false,
        value_asked: false,
        total_dues: false,
        chukta_group: false,
        beyond_estimate: false,
        sure: false,
        doubts: vec![doubt],
    }
}

/// Words that name the whole shop or a total ("kul", "total", "dukan", "sab", "poori", "saara", "tamam", "all"):
/// beside a named party or item they point at a figure of the shop, which the named scope would answer wrongly.
/// The words that only sum a few things ("dono", "mila kar") are not here.
const SHOP_WORDS: [&str; 38] = [
    "kul",
    "total",
    "dukan",
    "dukaan",
    "shop",
    "sab",
    "sabhi",
    "poori",
    "poora",
    "pura",
    "puri",
    "whole",
    "overall",
    "all",
    "entire",
    "saara",
    "saari",
    "sara",
    "sari",
    "saare",
    "sare",
    "saray",
    "saaray",
    "tamam",
    "tamaam",
    "mukammal",
    "کل",
    "دکان",
    "سب",
    "پوری",
    "پورا",
    "سارا",
    "ساری",
    "سارے",
    "تمام",
    "ٹوٹل",
    "مجموعی",
    "میزان",
];

/// The first word of `tokens` as a run in `words`: where a period's words stand in the question.
fn find_run(words: &[String], tokens: &[String]) -> Option<usize> {
    if tokens.is_empty() {
        return None;
    }
    (0..words.len()).find(|at| {
        tokens
            .iter()
            .enumerate()
            .all(|(i, t)| words.get(at.saturating_add(i)) == Some(t))
    })
}

/// True when a word of the question is known only as a part of a screen's name ("agrahi" of "agrahi list"): no cue
/// of a figure covers it, it is no function word, no time word, no movement verb or its particle, no word of a
/// period and no part of a name that was found. A figure answered beside it would leave it out.
fn stray_screen_word(scan: &Scan, spans: &[(usize, usize)], periods: &[PeriodRef]) -> bool {
    let table = super::lexicon::table();
    let period_words: Vec<String> = periods
        .iter()
        .flat_map(|p| p.words.split_whitespace().map(key))
        .collect();
    scan.words.iter().enumerate().any(|(at, w)| {
        table.is_screen_word_only(w)
            && !table.is_stop(w)
            && !super::when::is_time_word(w)
            // The verbs that say which way money moved are the business of `happened`, not of this rule.
            && !super::function_words::is_action_verb(w)
            && !super::function_words::is_event_word(w)
            // The particle of a phrasal verb ("come back", "paid back"): the verb is read by `happened`.
            && !at
                .checked_sub(1)
                .and_then(|b| scan.word(b))
                .is_some_and(super::function_words::is_action_verb)
            && !period_words.contains(w)
            && !spans.iter().any(|(s, e)| *s <= at && at < *e)
            && !scan
                .hits
                .iter()
                .any(|h| h.cue != Cue::ScreenName && h.start <= at && at < h.end)
    })
}

/// The "how much" and "how many" cues of the question, each place once.
fn how_much_cues(scan: &Scan) -> usize {
    let mut starts: Vec<usize> = scan
        .hits
        .iter()
        .filter(|h| h.cue == Cue::HowMuch)
        .map(|h| h.start)
        .collect();
    starts.sort_unstable();
    starts.dedup();
    starts.len()
}

/// The counts a question asks, read from the lexicon: a "how many" head (kitne, kitni, how many) with a
/// buyer, seller or item word right after it, or after one word ("kitne purane gahak"). A misspelling the
/// lexicon takes for the word counts too.
fn scan_count_asks(scan: &Scan) -> usize {
    const HEADS: [&str; 6] = ["kitne", "kitnay", "kitni", "kitny", "کتنے", "کتنی"];
    const THINGS: [Cue; 6] = [
        Cue::RoleBuyer,
        Cue::ManyBuyers,
        Cue::RoleSeller,
        Cue::ManySellers,
        Cue::ManyItems,
        Cue::RoleItem,
    ];
    let mut starts: Vec<usize> = Vec::new();
    for hit in scan.hits.iter().filter(|h| h.cue == Cue::HowMuch) {
        let word = scan.word(hit.start).unwrap_or_default();
        let many = word == "how" && scan.word(hit.start.saturating_add(1)) == Some("many");
        if !(HEADS.contains(&word) || many) {
            continue;
        }
        let after = hit.end;
        if scan.hits.iter().any(|t| {
            THINGS.contains(&t.cue) && t.start >= after && t.start <= after.saturating_add(1)
        }) {
            starts.push(hit.start);
        }
    }
    starts.sort_unstable();
    starts.dedup();
    starts.len()
}

fn part<T>(value: T, confidence: Confidence) -> Part<T> {
    Part { value, confidence }
}

// ---- the figures --------------------------------------------------------------------------------------

/// What the figures depend on besides the cue: who and what is named, and whether a day is.
struct Named<'a> {
    /// The word spans of the names that were found (a party, an item, a record).
    spans: &'a [(usize, usize)],
    roles: &'a [PartyRole],
    has_item: bool,
    has_period: bool,
    /// True when a word of the question is in no table, is no function word and is part of no name.
    has_unexplained_words: bool,
}

fn figure_of(hit: &Hit, scan: &Scan, named: &Named<'_>) -> Option<FigureKind> {
    let roles = named.roles;
    let has_party = !roles.is_empty();
    let account_party = roles
        .iter()
        .any(|r| matches!(r, PartyRole::Buyer | PartyRole::Seller));
    Some(match hit.cue {
        Cue::Balance => FigureKind::Balance,
        // "kitne bikri hue", "how many sale": how many, not how much. "kitni bikri" and "kitne ki
        // bikri" ask the amount. An amount shown for a count would be a wrong figure.
        Cue::Sale => {
            let before = hit.start.checked_sub(1).and_then(|b| scan.word(b));
            if matches!(before, Some("kitne" | "kitnay" | "کتنے" | "many")) {
                FigureKind::SaleCount
            } else {
                FigureKind::Sale
            }
        }
        Cue::SalePlural => FigureKind::SaleCount,
        Cue::Commission => FigureKind::Commission,
        // "kitna laga hua hai" is "how much is put in" (a verb), not the laga charge.
        Cue::Laga => {
            // "aaj kitna laga": how much did it cost, a verb. The charge is "laga kitna hai".
            let before = hit.start.checked_sub(1).and_then(|b| scan.word(b));
            if before.is_some_and(|w| {
                ["kitna", "kitni", "kitne", "کتنا", "کتنی", "کتنے"]
                    .iter()
                    .any(|x| key(x) == w)
            }) {
                return None;
            }
            // "kitna commission laga": the verb straight after the commission (money review 9); the charge
            // is asked with "aur", "ka" or a word of its own.
            if scan
                .hits
                .iter()
                .any(|h| h.cue == Cue::Commission && h.end == hit.start)
            {
                return None;
            }
            let next = scan.word(hit.end);
            if next.is_some_and(|w| {
                ["hua", "hui", "hue", "huwa", "ہوا", "ہوئی", "ہوئے"]
                    .iter()
                    .any(|x| key(x) == w)
            }) {
                return None;
            }
            FigureKind::Laga
        }
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
        // A payable question about a named party stays a payable question: his card has a
        // `payable` field, and for a seller with an advance that was not deducted it is not his
        // ledger balance (E2E-28: payable 19,316, ledger 4,316 Cr). Lead ruling of 04-11-2026.
        Cue::Payable => FigureKind::Payable,
        Cue::Advance => FigureKind::Advance,
        Cue::Stock => FigureKind::Stock,
        Cue::Cash => FigureKind::Cash,
        Cue::Capital => FigureKind::Capital,
        Cue::PartnerTotal => FigureKind::PartnerTotal,
        Cue::PartnerShare => FigureKind::PartnerShare,
        Cue::Profit => FigureKind::Profit,
        Cue::Expense => FigureKind::Expense,
        Cue::FruitTaken => FigureKind::FruitTaken,
        Cue::PartnerTaken => FigureKind::PartnerTaken,
        // "Partner A's fruit": the fruit he took. The bare word beside no partner is an item group.
        Cue::FruitWord if roles.contains(&PartyRole::Partner) || scan.has(Cue::Guests) => {
            FigureKind::FruitTaken
        }
        // The title of a report that no figure names: "udhari bikri", "adda report".
        // The app's own state or lists: the rules, the attention list, the closed days, the staff.
        Cue::AskName => FigureKind::Ask(super::lexicon::ask_named(
            scan.words.get(hit.start..hit.end).unwrap_or_default(),
        )?),
        Cue::ReportName => FigureKind::Report(super::lexicon::report_named(
            scan.words.get(hit.start..hit.end).unwrap_or_default(),
        )?),
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
            if scan.has_any(&[
                Cue::Goods,
                Cue::Stock,
                Cue::Unit,
                Cue::VegetableWord,
                Cue::FruitWord,
            ]) || (named.has_item && !roles.contains(&PartyRole::Buyer))
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

/// Words of buying. `Cue::Bought` also holds the bare "li" of "peshgi li hui hai", which is no buying.
const BUY_WORDS: [&str; 20] = [
    "khareed",
    "khareeda",
    "khareedi",
    "khareede",
    "khareedna",
    "khareedta",
    "kharid",
    "kharida",
    "kharidi",
    "buy",
    "bought",
    "buys",
    "buying",
    "purchase",
    "purchased",
    "purchases",
    "خرید",
    "خریدا",
    "خریدی",
    "خریدے",
];

/// "Jo" and "jis" ("aaj jo maal bika us par kitna commission bana"): a verb after one qualifies the figure.
const RELATIVE_WORDS: [&str; 6] = ["jo", "jis", "jin", "جو", "جس", "جن"];

/// True when an asked figure is dropped before the plan sees it, so that the answer would leave a part of the
/// question out (money reviews of 08-11-2026, D7 plan 59 C):
/// - a cue of tier 1 (collecting, money) names ANOTHER figure than the plainest tier found ("kharcha aur wasooli"),
///   or stands after an "and" that follows a plain figure ("bakaya kitna hai aur wasooli kitni");
/// - a tier-2 "baqi" with goods is the stock, dropped beside a plain figure ("bikri aur maal kitna baqi");
/// - the same figure is named twice (`repeated`);
/// - collected and still to collect stand together;
/// - a verb of buying or selling stands and no figure of buying or selling was found beside another one, unless the
///   verb is in a relative clause with no "and" after it.
///
/// Tier 2 words ("khata", "hisab") only qualify a figure and are no second ask.
fn dropped_ask(
    scan: &Scan,
    named: &Named<'_>,
    found: &[(usize, FigureKind)],
    best: Option<u8>,
    repeated: bool,
) -> bool {
    let kind_found = |kind: FigureKind| found.iter().any(|(_, k)| *k == kind);
    let outstanding = [FigureKind::Balance, FigureKind::Dues, FigureKind::Payable];
    let family_found = |kind: FigureKind| {
        if outstanding.contains(&kind) {
            found.iter().any(|(_, k)| outstanding.contains(k))
        } else {
            kind_found(kind)
        }
    };
    let ands: Vec<usize> = scan
        .hits
        .iter()
        .filter(|h| h.cue == Cue::And)
        .map(|h| h.start)
        .collect();
    let plain_before = |at: usize| {
        scan.hits
            .iter()
            .any(|h| tier(h.cue) == 0 && h.end <= at && figure_of(h, scan, named).is_some())
    };
    let and_after_plain = |at: usize| ands.iter().any(|a| *a < at && plain_before(*a));
    let tier1 = best == Some(0)
        && scan.hits.iter().filter(|h| tier(h.cue) == 1).any(|h| {
            match figure_of(h, scan, named) {
                // "agrahi wale ke hisab se kul baqaya": the collecting word names the outstanding balance, which is the
                // figure found (balance, dues and payable are one family here).
                Some(kind) => !family_found(kind) || and_after_plain(h.start),
                None => false,
            }
        });
    let tier2_stock = best != Some(2)
        && scan.hits.iter().filter(|h| tier(h.cue) == 2).any(|h| {
            figure_of(h, scan, named) == Some(FigureKind::Stock) && !kind_found(FigureKind::Stock)
        });
    let word_at = |list: &[&str]| {
        scan.words
            .iter()
            .position(|w| list.iter().any(|x| key(x) == *w))
    };
    let relative_at = word_at(&RELATIVE_WORDS);
    // A verb in a relative clause qualifies the figure; with an "and" after it, it is an ask of its own.
    let own_ask = |verb_at: usize| {
        !(relative_at.is_some_and(|r| r < verb_at) && !ands.iter().any(|a| *a > verb_at))
    };
    let buy_ask = !found.is_empty()
        && !(kind_found(FigureKind::Purchases) || kind_found(FigureKind::Sale))
        && word_at(&BUY_WORDS).is_some_and(own_ask);
    // "kele ka aakhri rate kis par bika": the rate it sold at is the rate asked.
    let sold_ask = !found.is_empty()
        && scan.has_exact(Cue::Sold)
        && !(kind_found(FigureKind::Sale)
            || kind_found(FigureKind::SaleCount)
            || kind_found(FigureKind::Purchases)
            || kind_found(FigureKind::Rate))
        && scan
            .hits
            .iter()
            .find(|h| h.cue == Cue::Sold)
            .is_some_and(|h| own_ask(h.start));
    tier1
        || tier2_stock
        || repeated
        || buy_ask
        || sold_ask
        || (scan.has(Cue::Collection)
            && scan.has(Cue::Received)
            && scan.has_any(&[Cue::Pending, Cue::Balance, Cue::Dues]))
}

/// The figures asked for, in the order of the question, and whether two were asked in one breath.
fn figures(scan: &Scan, named: &Named<'_>) -> (Vec<FigureKind>, Vec<usize>, bool, bool) {
    let roles = named.roles;
    let has_party = !roles.is_empty();
    let mut found: Vec<(usize, FigureKind)> = Vec::new();
    // Where each found figure's word ends, to tell a figure named again from its own phrase and word.
    let mut ends: Vec<usize> = Vec::new();
    let mut repeated = false;
    let best = scan
        .hits
        .iter()
        .filter(|h| figure_of(h, scan, named).is_some())
        .map(|h| tier(h.cue))
        .min();
    // A name found between two hits of one figure makes them two scopes ("akram ka bakaya aur arshad ka bakaya").
    let name_between = |a: usize, b: usize| named.spans.iter().any(|(s, _)| *s > a && *s < b);
    for hit in &scan.hits {
        if Some(tier(hit.cue)) != best {
            continue;
        }
        if let Some(kind) = figure_of(hit, scan, named) {
            match found.iter().position(|(_, k)| *k == kind) {
                None => {
                    found.push((hit.start, kind));
                    ends.push(hit.end);
                }
                // The same figure named twice with no name between ("akram ka bakaya aur kul bakaya", "akram ka
                // bakaya, kul bakaya"): two asks of one kind, merged into one here, so the question is no one
                // figure's.
                Some(at) => {
                    let first = found.get(at).map_or(0, |f| f.0);
                    let first_end = ends.get(at).copied().unwrap_or(0);
                    // Two words of one noun phrase are no second ask ("galle me kitna cash hai", "cash balance of the
                    // cash book"): an "and" between them, or a word for the whole shop right before the second one.
                    let and_between = scan
                        .hits
                        .iter()
                        .any(|h| h.cue == Cue::And && h.start > first && h.start < hit.start);
                    let shop_before = hit
                        .start
                        .checked_sub(1)
                        .and_then(|b| scan.word(b))
                        .is_some_and(|w| SHOP_WORDS.iter().any(|x| key(x) == w));
                    // "kal kitna cash aya aur kitna cash gaya": the cash that came in and went out are two parts of
                    // the cash book, which its own plan answers.
                    let cash_parts =
                        kind == FigureKind::Cash && super::reads::cash_directions(scan).len() >= 2;
                    if hit.start >= first_end
                        && !name_between(first, hit.start)
                        && (and_between || shop_before)
                        && !cash_parts
                    {
                        repeated = true;
                    }
                }
            }
        }
    }
    let mut lower_ask = dropped_ask(scan, named, &found, best, repeated);
    // What a buyer bought or a seller sold, from the verb, when no figure word is there.
    if found.is_empty() {
        if scan.has_exact(Cue::Bought) && !scan.has_exact(Cue::Sold) {
            found.push((0, FigureKind::Purchases));
        } else if scan.has_exact(Cue::Sold) {
            found.push((0, FigureKind::Sale));
        }
    }
    // "سبجی کی quantity کل کتنی تھی": the quantity of a group (or an item) is the nag sold.
    if found.is_empty()
        && scan.has(Cue::Unit)
        && (named.has_item || scan.has_exact(Cue::VegetableWord) || scan.has_exact(Cue::FruitWord))
    {
        found.push((0, FigureKind::Sale));
    }
    // "maal kis cheez ka para hai": goods lying (para, pada, padi) are the goods left.
    if found.is_empty()
        && scan.has(Cue::Goods)
        && scan.words.iter().any(|w| {
            matches!(
                w.as_str(),
                "para" | "pada" | "padi" | "pare" | "pade" | "paray" | "پڑا" | "پڑی" | "پڑے"
            )
        })
    {
        found.push((0, FigureKind::Stock));
    }
    // "usman shirakat daar ka kitna hai abhi": a partner and "kitna" ask for his total, when nothing
    // else is asked. A cue of another figure ("fruit"), or a word nothing explains ("home"), means
    // another question: "how much fruit did partner A take home" is not his total.
    if found.is_empty()
        && scan.has(Cue::HowMuch)
        && roles == [PartyRole::Partner]
        && scan.hits.iter().all(|h| {
            matches!(
                h.cue,
                Cue::HowMuch | Cue::RolePartner | Cue::And | Cue::ScreenName
            )
        })
        && !named.has_unexplained_words
        // A verb of action ("nikala", "liya", "took", "paid", "earn", "milta", "le gaya") asks what he TOOK, PUT or
        // EARNED, not what he has: never his total. Only the verbs of being and having leave it alone.
        && !scan
            .words
            .iter()
            .any(|w| super::function_words::is_action_verb(w))
    {
        found.push((0, FigureKind::PartnerTotal));
    }
    // "byopariyon ko kitna dena hai": what the shop owes sellers. "sab gahakon ka bakaya": the dues.
    // The direction counts: "dena" is what the shop pays, "lena" what it collects. "gahakon ko kitna
    // dena hai" and "byopariyon se kitna lena hai" are the other way round, so they keep the plain
    // balance word and, with no name, go to the model (money review, 05-11-2026).
    if !has_party {
        let says = |list: &[&str]| scan.words.iter().any(|w| list.iter().any(|x| key(x) == *w));
        let gives = says(&[
            "dena", "dene", "dega", "degi", "denge", "dengi", "deta", "dete", "دینا", "دینے", "دے",
        ]);
        let takes = says(&[
            "lena", "lene", "lenge", "lengi", "leta", "lete", "لینا", "لینے", "لے",
        ]);
        let mut rewrote = false;
        for (_, kind) in &mut found {
            if *kind == FigureKind::Balance {
                if scan.has(Cue::ManySellers) {
                    if !takes {
                        *kind = FigureKind::Payable;
                        rewrote = true;
                    }
                } else if scan.has_any(&[Cue::ManyBuyers, Cue::Total]) && !gives {
                    *kind = FigureKind::Dues;
                }
            }
        }
        // "gahakon ka bakaya aur byopariyon ki peshgi": the one word "bakaya" became the sellers' payable for both
        // sides, and the buyers' dues that were asked are gone.
        if rewrote
            && scan.has_any(&[Cue::ManyBuyers, Cue::RoleBuyer])
            && scan.has_any(&[Cue::ManySellers, Cue::RoleSeller])
        {
            lower_ask = true;
        }
    }
    // "cash balance": the cash, not a party's balance. A balance asked apart from the cash word ("roqar kitni hai aur
    // kitna lena hai") is a second ask that this would drop.
    if !has_party && found.iter().any(|(_, k)| *k == FigureKind::Cash) {
        let cash_at = found
            .iter()
            .find(|(_, k)| *k == FigureKind::Cash)
            .map_or(0, |f| f.0);
        if found
            .iter()
            .any(|(at, k)| *k == FigureKind::Balance && at.abs_diff(cash_at) > 2)
        {
            lower_ask = true;
        }
        found.retain(|(_, k)| *k != FigureKind::Balance);
    }
    found.sort_by_key(|(at, _)| *at);
    let kinds: Vec<FigureKind> = found.iter().map(|(_, k)| *k).collect();
    let positions: Vec<usize> = found.iter().map(|(at, _)| *at).collect();
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
    (kinds, positions, two, lower_ask)
}

// ---- intent -------------------------------------------------------------------------------------------

/// The signs the rules below read.
struct Signs {
    how: bool,
    how_much: bool,
    /// A screen is named so that "show me" opens it.
    screen_open: bool,
    /// The name of that screen has two words or more ("today's sales list"): a time word inside it is
    /// part of the name.
    screen_long: bool,
    party_found: bool,
    /// A garden or a trip of the books was found by name (the words of its name are no cue).
    entry_party: bool,
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
    // "what is each partner's total now": a figure asked with "what is", not the meaning of a word.
    // A word that is only explained has no party, no item, no period and no "now" beside it
    // ("what is wasooli").
    let now_words = scan.words.iter().any(|w| {
        matches!(
            w.as_str(),
            "now" | "abhi" | "current" | "currently" | "اب" | "ابھی" | "موجودہ"
        )
    });
    let defines_a_figure = scan.has(Cue::Define)
        && !signs.how
        && !scan.has_any(&[Cue::Where, Cue::Capability])
        && signs.figures
        && (signs.party_found || signs.item_found || signs.period || now_words);
    let how_like = !defines_a_figure
        && (signs.how || scan.has_any(&[Cue::Where, Cue::Define, Cue::Capability]));
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
    let weak_write = scan.has_exact(Cue::WriteWeak)
        && (signs.amount
            || percent_to_write
            || signs.entry_party
            || change::weak_write_of_an_entry(scan));
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
    // A phrase of the advice words that holds its own "how" ("ziyada kaise", "zyada kaise karen")
    // has swallowed the word "kaise" as a cue, so it needs no other sign.
    let advice_phrase = scan
        .hits
        .iter()
        .any(|h| h.cue == Cue::Advice && !h.fuzzy && h.end.saturating_sub(h.start) >= 2);
    let asks_advice = scan.has(Cue::Decision)
        || scan.has(Cue::AdviceStrong)
        || advice_phrase
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
        || (scan.has_exact(Cue::Show) && !signs.party_found && !signs.figures && !signs.how_much)
        || (scan.has_exact(Cue::Show)
            && signs.screen_open
            && !signs.party_found
            && (signs.screen_long || !signs.period)
            && !signs.how_much);
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

/// The topic of the screen a question names ("cash book", "day close", "gardens"): the longest name of
/// [`SCREEN_NAMES`] in it, when one topic only is longest.
fn screen_topic_of(scan: &Scan) -> Option<Part<&'static str>> {
    let topics = &guide().topics;
    let found: Vec<(usize, &'static str)> = super::lexicon::screen_windows(&scan.words)
        .into_iter()
        .filter(|(_, _, topic)| topics.contains_key(*topic))
        .map(|(start, end, topic)| (end.saturating_sub(start), topic))
        .collect();
    let longest = found.iter().map(|(n, _)| *n).max()?;
    let mut top: Vec<&'static str> = found
        .iter()
        .filter(|(n, _)| *n == longest)
        .map(|(_, t)| *t)
        .collect();
    top.dedup();
    match top.as_slice() {
        [one] => Some(part(*one, Confidence::High)),
        _ => None,
    }
}

/// True when a "show me" names a screen so that it opens it: a name of two words ("cash book"), or the
/// word "screen" or "page" beside a name ("wasooli screen dikhao"). A one-word figure name alone
/// ("cash dikhao") asks for the figure.
fn screen_open(scan: &Scan) -> bool {
    let windows = super::lexicon::screen_windows(&scan.words);
    let named = !windows.is_empty();
    let long = windows.iter().any(|(s, e, _)| e.saturating_sub(*s) >= 2);
    let word = scan.words.iter().any(|w| {
        matches!(
            w.as_str(),
            "screen" | "page" | "safha" | "صفحہ" | "سکرین" | "اسکرین"
        )
    });
    named && (long || word)
}

/// The entry screen name of the answer schema whose topic is this one.
pub(super) fn screen_of(topic: &str) -> Option<&'static str> {
    // A screen whose name is the topic ("accounts", the list) before the others of the same topic
    // ("account", the form).
    let own = EntryScreenName::ALL.iter().copied().find(|name| {
        *name == topic
            && serde_json::from_value::<EntryScreenName>(serde_json::json!(name))
                .is_ok_and(|screen| help_guard::topic_of(screen) == topic)
    });
    if own.is_some() {
        return own;
    }
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

/// True when a word of the question is in no table of the app, is no function word, is no number
/// and is part of no name that was read: the question says something the app did not understand.
///
/// Example: in "how much fruit did partner A take home", "home" is such a word.
fn unexplained_words(scan: &Scan, names: &Names, periods: &[PeriodRef]) -> bool {
    unexplained_except(scan, names, periods, &|_| false)
}

/// As [`unexplained_words`], not counting the words `skip` explains.
fn unexplained_except(
    scan: &Scan,
    names: &Names,
    periods: &[PeriodRef],
    skip: &dyn Fn(&str) -> bool,
) -> bool {
    let table = super::lexicon::table();
    let mut name_words: Vec<String> = Vec::new();
    // The words of a period ("so far", "ab tak") are explained by the period.
    for period in periods {
        name_words.extend(period.words.split_whitespace().map(key));
    }
    let texts = names
        .parties
        .iter()
        .map(|p| p.text.as_str())
        .chain(names.items.iter().map(|i| i.text.as_str()))
        .chain(names.records.iter().map(|r| r.text.as_str()));
    for text in texts {
        name_words.extend(text.split_whitespace().map(key));
    }
    // "after", "since" and "until" relate a figure to a period: a plain connective only when the question
    // has no period and no digit ("buyers still owe how much after paying").
    let plain_connective_ok = periods.is_empty()
        && !scan.words.iter().any(|w| {
            w.chars().any(|c| c.is_ascii_digit()) || w.starts_with(super::scan::DATE_WORD)
        });
    // A word that limits, excludes, places or gives a habit is read by no plan: never explained.
    if scan
        .words
        .iter()
        .any(|w| super::function_words::is_unread_filter(w))
    {
        return true;
    }
    scan.words.iter().enumerate().any(|(at, w)| {
        !table.is_known(w)
            && !skip(w)
            && !(plain_connective_ok && super::function_words::conditional(w))
            && !name_words.contains(w)
            && !super::when::is_time_word(w)
            && !scan.hits.iter().any(|h| h.start <= at && at < h.end)
            && !w.chars().all(|c| c.is_ascii_digit())
            && w.chars().count() > 1
    })
}

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
    mut names: Names,
    periods: Vec<PeriodRef>,
    time_word: bool,
    period_merged: bool,
) -> Understanding {
    let spans: Vec<(usize, usize)> = names.spans.clone();
    // Where each period's words stand: they hold no ask of their own ("14 tareekh ko"), and a period that stands
    // between two figures may be the second one's only.
    let period_runs: Vec<(usize, usize)> = periods
        .iter()
        .filter_map(|p| {
            let tokens: Vec<String> = p.words.split_whitespace().map(key).collect();
            find_run(&scan.words, &tokens).map(|at| (at, at.saturating_add(tokens.len())))
        })
        .collect();
    let skip: Vec<(usize, usize)> = spans.iter().chain(period_runs.iter()).copied().collect();
    let stray_screen = stray_screen_word(scan, &spans, &periods);
    let ands: Vec<usize> = scan
        .hits
        .iter()
        .filter(|h| h.cue == Cue::And)
        .map(|h| h.start)
        .collect();
    let roles: Vec<PartyRole> = names
        .parties
        .iter()
        .filter_map(|p| p.outcome.hit().map(|h| h.role))
        .collect();
    let named = Named {
        spans: &spans,
        roles: &roles,
        has_item: names.items.iter().any(|i| i.outcome.hit().is_some()),
        has_period: !periods.is_empty(),
        has_unexplained_words: unexplained_words(scan, &names, &periods),
    };
    let (kinds, figure_at, two_figures, lower_ask) = figures(scan, &named);
    // An advance and a payable belong to a seller: a name found only as a seller is the seller asked
    // for, though "ne ... li" reads like a buyer ("tahir ne kitni peshgi li hui hai", live run 2, Q610).
    if kinds
        .iter()
        .any(|k| matches!(k, FigureKind::Advance | FigureKind::Payable))
    {
        for party in &mut names.parties {
            if let Outcome::FoundAs(hit) = party.outcome.clone()
                && hit.role == PartyRole::Seller
            {
                party.outcome = Outcome::Found(hit);
                party.asked_role = None;
            }
        }
    }
    // "akram aur arshad ka jama": two named parties and "jama" (their total) are the sum of their
    // balances, read for each of them (live run 2, Q069).
    let jama = scan
        .words
        .iter()
        .any(|w| matches!(w.as_str(), "jama" | "jamaa" | "جمع"))
        && names
            .parties
            .iter()
            .filter(|p| p.outcome.hit().is_some())
            .count()
            >= 2;
    let kinds = if kinds.is_empty() && jama {
        vec![FigureKind::Balance]
    } else {
        kinds
    };
    // "partner Zafar Iqbal's total": the word "partner" and a total are a partner's total, though the
    // name stands between them.
    let kinds = if kinds.is_empty() && scan.has(Cue::Total) && scan.has_exact(Cue::RolePartner) {
        vec![FigureKind::PartnerTotal]
    } else {
        kinds
    };
    // A word for a group of items ("sabzi", "phal") with no item named is a group, not a drop.
    let item_group = if !names.items.is_empty() {
        None
    } else if group_word(scan, &names, Cue::VegetableWord) {
        Some(super::GroupWord::Vegetable)
    } else if group_word(scan, &names, Cue::FruitWord)
        && !kinds.contains(&FigureKind::FruitTaken)
        && !scan.has(Cue::Guests)
        && !roles
            .iter()
            .any(|r| matches!(r, PartyRole::Partner | PartyRole::Garden | PartyRole::Trip))
    {
        Some(super::GroupWord::Fruit)
    } else {
        None
    };
    // "کل ادائیگی کتنی ہے": a bare "kal" beside a figure of NOW is "total", not yesterday (guard 6).
    let periods = super::reads::drop_total_kal(periods, &kinds, !super::when::has_tense_word(scan));
    let signs = Signs {
        how: scan.has(Cue::How),
        how_much: scan.has(Cue::HowMuch) || how_it_went(scan, !kinds.is_empty()),
        screen_open: screen_open(scan),
        screen_long: super::lexicon::screen_windows(&scan.words)
            .iter()
            .any(|(start, end, _)| end.saturating_sub(*start) >= 2),
        party_found: names
            .parties
            .iter()
            .any(|p| p.outcome.hit().is_some() || matches!(p.outcome, Outcome::Several(_))),
        entry_party: roles
            .iter()
            .any(|r| matches!(r, PartyRole::Garden | PartyRole::Trip)),
        item_found: names.items.iter().any(|i| i.outcome.hit().is_some()),
        period: !periods.is_empty(),
        figures: !kinds.is_empty(),
        amount: has_amount(scan),
    };
    let Chosen { intent, mut doubts } = intent_of(scan, &signs, &periods);
    doubts.extend(names.doubts.iter().copied());
    // A word no plan reads (an exclusion, a place, a limit, a habit) is as good as an unknown word: the model
    // answers, whatever near spelling the lexicon finds for it.
    if scan
        .words
        .iter()
        .any(|w| super::function_words::is_unread_filter(w))
        && !doubts.contains(&Doubt::UnknownWords)
    {
        doubts.push(Doubt::UnknownWords);
    }
    // A time word that no period took ("pichli", "hafte mein"): for a figure, a list or a printout the
    // period is part of the question, so the app is not sure of it. Help and the rest read no period.
    if time_word && matches!(intent.value, Intent::Figure | Intent::List | Intent::Print) {
        doubts.push(Doubt::TimeWord);
    }
    if scan.cut {
        doubts.push(Doubt::Cut);
    }
    // A request to write that also asks to read: the refusal would drop the question.
    if intent.value == Intent::ChangeRequest && change::asks_to_read(scan, signs.amount) {
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
    let sum_asked = cues::together(&scan.raw) || jama;
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
        .then(|| {
            // To open a screen, its own name is the best sign of which one.
            screen_topic_of(scan)
                .filter(|_| intent.value == Intent::Navigate)
                .or_else(|| topic_of(scan))
        })
        .flatten();
    let screen = match intent.value {
        Intent::Navigate => topic.as_ref().and_then(|t| screen_of(t.value)),
        // The entry screen of a request to write (the assistant never writes, PRD FR-7.72).
        Intent::ChangeRequest => change::screen_of(scan, &names, topic_of(scan).map(|t| t.value)),
        _ => None,
    };
    let list = (intent.value == Intent::List).then(|| list_ask(scan));
    // A ranking ("top 5", "sab se zyada") reads its own view; a list of the lines of one party is not.
    let ranking = list.is_some_and(|l| l.rank.is_some());
    let wants_steps = intent.value == Intent::Print && scan.has(Cue::How);
    let (first_day, trip_days) = plan_days(reader, intent.value, &names, &kinds);
    // One بکری named by its number ("bikri number 5 ka print"): looked up in the books.
    let bikri_number = (intent.value == Intent::Print)
        .then(|| super::reads::bikri_number(scan))
        .flatten();
    let bikri_row = bikri_number.and_then(|number| {
        mandi_services::bikri::find(
            reader.ctx,
            &mandi_services::core::bikri::BikriFindInput {
                number: Some(number),
                ..Default::default()
            },
        )
        .ok()?
        .into_iter()
        .find(|r| r.number == number)
        .map(|r| (r.id, number))
    });
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
        unknown_words: names.doubts.contains(&Doubt::UnknownWords),
        scope_missing: super::reads::print_scope_missing(scan, &kinds),
        garden_word: super::reads::garden_word(scan, &names),
        trip_word: super::reads::trip_word(scan, &names),
        guests: scan.has_exact(Cue::Guests),
        item_group,
        bikri_number,
        bikri_row,
    });
    doubts.extend(planned.doubts.iter().copied());
    // A print has no quality filter: it would show every quality for the one asked (plan 58, FR-12.42).
    if scan.has_exact(Cue::Quality) && intent.value == Intent::Print {
        doubts.push(Doubt::NoReport);
    }

    let kinds_for_happened = kinds.clone();
    let roles_found = roles.clone();
    let has_item_found = names.items.iter().any(|i| i.outcome.hit().is_some());
    let grade = super::reads::grade_of(scan, has_item_found || item_group.is_some());
    // The side of the commission asked, read before the names move into the understanding (plan 59, F1).
    let commission_side = super::reads::commission_side(scan, &kinds_for_happened, &names.parties);
    let unexplained_flag = unexplained_words(scan, &names, &periods);
    // The same, not counting the side words of the trial balance ("debit", "credit", "banaam", "jama").
    let unexplained_but_trial_words = unexplained_except(scan, &names, &periods, &|w| {
        super::reads::is_trial_side_word(w)
    });
    // The same, not counting the direction words of the cash book ("came", "went", "out", "gaya").
    let unexplained_but_cash_words = unexplained_except(scan, &names, &periods, &|w| {
        super::reads::is_cash_direction_word(w)
    });
    // A "how many" before any word the lexicon takes for buyers, sellers or items, in any spelling it
    // accepts, asks a count; the text rule has its own list of nouns. The larger of the two counts stands
    // (D7, plan 59 F3b: "kitne gahkon ke zimme paise baqi hain" asks the number of buyers).
    let count_asks = cues::count_asks(&scan.raw).max(scan_count_asks(scan));
    let names_have_garden = roles_found
        .iter()
        .any(|r| matches!(r, PartyRole::Garden | PartyRole::Trip));
    let beyond = reader
        .today_date()
        .is_some_and(|today| beyond_estimate(&periods, today));
    let chukta_group =
        scan.has_exact(Cue::Chukta) && names.parties.is_empty() && names.items.is_empty();
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
        lower_ask,
        period_merged,
        sum_asked,
        compare,
        list,
        report: planned.report,
        topic,
        screen,
        wants_steps,
        scoped: super::reads::scoped(scan, &kinds_for_happened),
        guests: scan.has_exact(Cue::Guests),
        quality: scan.has_exact(Cue::Quality) || grade.is_some(),
        grade,
        capital_flow: super::reads::capital_flow(scan),
        cash_dirs: super::reads::cash_directions(scan),
        trial_sides: super::reads::trial_sides(scan),
        sale_number: super::reads::document_number(scan)
            .then(|| super::reads::bikri_number(scan))
            .flatten(),
        garden_goods: super::reads::garden_goods(scan, &names_have_garden),
        last_one: super::reads::last_one(scan),
        months_asked: scan.has_exact(Cue::HowManyMonths),
        months_range: scan
            .has_exact(Cue::HowManyMonths)
            .then(|| books_range(reader))
            .flatten(),
        oldest: super::reads::oldest(scan),
        item_group,
        bikri_number: super::reads::bikri_number(scan),
        truck: scan.truck.clone(),
        amount_named: signs.amount,
        sum_strong: cues::together_strong(&scan.raw),
        happened: super::reads::happened(scan, &kinds_for_happened),
        other_measure: super::reads::other_measure(
            scan,
            &kinds_for_happened,
            &roles_found,
            has_item_found,
            ranking,
        ),
        commission_side,
        buyer_rate: super::reads::buyer_rate(scan),
        asks_count: count_asks > 0,
        count_asks,
        how_much_cues: how_much_cues(scan),
        qty_asked: false,
        average_asked: cues::average_asked(&scan.raw),
        pay_kind: super::reads::pay_kind(scan),
        pay_asked: super::reads::pay_asked(scan),
        unexplained: unexplained_flag,
        role_word: scan.has_any(&[
            Cue::RoleBuyer,
            Cue::ManyBuyers,
            Cue::RoleSeller,
            Cue::ManySellers,
        ]),
        seller_word: scan.has_any(&[Cue::RoleSeller, Cue::ManySellers]),
        buyer_word: scan.has_any(&[Cue::RoleBuyer, Cue::ManyBuyers]),
        open_ask: super::function_words::asks_beyond_a_figure(&scan.words, &skip),
        multi_measure: super::MULTI_MEASURE_INSTANT,
        figure_at: figure_at.clone(),
        name_spans: spans.clone(),
        period_between: match (figure_at.iter().min(), figure_at.iter().max()) {
            (Some(first), Some(last)) => period_runs.iter().any(|(s, _)| s > first && s < last),
            _ => false,
        },
        shop_after_and: spans.iter().map(|s| s.1).max().is_some_and(|last_end| {
            scan.words.iter().enumerate().any(|(w, word)| {
                w >= last_end
                    && SHOP_WORDS.iter().any(|x| key(x) == *word)
                    && ands.iter().any(|a| *a >= last_end && *a < w)
            })
        }),
        stray_screen_word: screen.is_none() && stray_screen,
        shop_word: scan
            .words
            .iter()
            .any(|w| SHOP_WORDS.iter().any(|x| key(x) == *w)),
        // Two asks, not two hits on one word: "kitna hai" is a phrase and a word that start at the same place.
        how_much_twice: how_much_cues(scan) >= 2,
        value_asked: super::reads::value_asked(scan),
        document_number: super::reads::document_number(scan),
        packing_filter: super::reads::packing_filter(scan),
        total_dues: kinds_for_happened.contains(&FigureKind::Dues)
            && (dead_amount_included(scan) || total_dues_asked(scan))
            && !scan.has_any(&[Cue::ManyBuyers, Cue::RoleBuyer]),
        chukta_group,
        beyond_estimate: beyond,
        sure: false,
        doubts,
    };
    understanding.qty_asked = super::reads::qty_asked(
        scan,
        &understanding.figures.value,
        understanding.other_measure,
    );
    if understanding.intent.value == Intent::NotInApp {
        understanding.outside = outside_kind(scan);
    }
    // A screen named by its own name is opened whatever figure its words also are ("commission
    // rules", "سرمایہ"): no report is looked for.
    if understanding.intent.value == Intent::Navigate
        && understanding.screen.is_some()
        && screen_topic_of(scan).is_some()
    {
        understanding.doubts.retain(|d| *d != Doubt::NoReport);
        // The screen wins over the report of its figure ("wasooli screen dikhao" opens the receipt
        // screen, not the dues report), unless a report is asked by name ("dead amount report").
        let report_word = scan
            .words
            .iter()
            .any(|w| matches!(w.as_str(), "report" | "reports" | "رپورٹ" | "رپورٹس"));
        let reports_screen = understanding
            .topic
            .as_ref()
            .is_some_and(|t| t.value == "reports");
        // A report name that is the screen's own name ("accounts list") is the screen.
        let windows = super::lexicon::screen_windows(&scan.words);
        let report_named = scan.hits.iter().any(|r| {
            r.cue == Cue::ReportName
                && !windows
                    .iter()
                    .any(|(start, end, _)| *start <= r.start && r.end <= *end)
        });
        if !report_named && (!report_word || reports_screen) {
            understanding.report = None;
        }
    }
    assume_ranking(&mut understanding, reader);
    assume_comparison(&mut understanding, reader);
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
    // "how much cash came in today", "cash went out yesterday": the question names a part of the cash book,
    // and its direction words are the only words nothing else explains. They are read, so they raise no
    // doubt and the question is not an event of a state figure (D7, plan 59 F3b, cash in and out).
    if !understanding.cash_dirs.is_empty()
        && understanding.figures.value == [FigureKind::Cash]
        && !unexplained_but_cash_words
    {
        understanding.doubts.retain(|d| *d != Doubt::UnknownWords);
        understanding.happened = false;
        understanding.unexplained = false;
    }
    // "what is the total debit of the trial balance": the side words are read by the plan, so they raise no doubt.
    if !understanding.trial_sides.is_empty()
        && understanding.figures.value == [FigureKind::TrialBalance]
        && !unexplained_but_trial_words
    {
        understanding.doubts.retain(|d| *d != Doubt::UnknownWords);
        understanding.unexplained = false;
    }
    understanding.sure = is_sure(&understanding);
    // Two or more periods of one figure ("august, september and october", "this month and last month")
    // are not a doubt when every period can be read: the app plans the reads, and the run starts only
    // from a sure understanding (live run 2, H348 and H371 read nothing).
    if !understanding.sure && super::reads::periods_are_planned(&understanding) {
        understanding.sure = true;
    }
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
    // A group of items ("kaun si sabzi sab se zyada bikti hai") is a ranking of items.
    if u.item_group.is_none()
        && !matches!(
            ask.over,
            Some(ListOver::Items | ListOver::Buyers | ListOver::Sellers)
        )
    {
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

/// "pichle mahine se kitna zyada hai" names only the month it is compared with: the other side is this
/// month so far. The month is added and marked as assumed, so the two periods can be read before the
/// model starts (`reads::comparison_plan`). Only for last month and one figure of a flow.
fn assume_comparison(u: &mut Understanding, reader: &Reader<'_>) {
    let one_flow = matches!(
        u.figures.value.as_slice(),
        [FigureKind::Sale | FigureKind::Commission | FigureKind::Laga]
    );
    if !u.compare || !one_flow || u.intent.value != Intent::Figure {
        return;
    }
    let [only] = u.periods.as_slice() else {
        return;
    };
    if only.kind != super::PeriodKind::LastMonth {
        return;
    }
    if let Some(month) = reader.today_date().and_then(super::when::this_month_so_far) {
        u.periods.push(month);
        u.assumed.push(Assumed::Period);
    }
}

/// A word for a group of items: exact, or one letter off ("vegitables") when it is part of no name
/// that was read. A group word spelt wrong is still the group: dropping it would answer for the whole
/// shop (live run 2, C1: "how much was sold of vegitables last week" got the whole shop's 704,040).
fn group_word(scan: &Scan, names: &Names, cue: Cue) -> bool {
    scan.has_exact(cue)
        || scan.hits.iter().any(|h| {
            h.cue == cue
                && h.fuzzy
                && !names
                    .spans
                    .iter()
                    .any(|(start, end)| h.start < *end && *start < h.end)
        })
}

/// True when the dues asked include the dead amount: "doobi raqam samet", "sab milake jo lena hai",
/// "with the dead amount". Then it is the tile `total_dues` (کل اگراہی: the buyers' dues, what sellers owe
/// the shop and the dead amount, PRD 9.1, FR-19.9, FR-7.76), as it is for "kul ugrahi" and "total dues"
/// ([`total_dues_asked`], D7: PRD v1.31 overrules the ruling of plan 55 that those two are the buyers'
/// dues of R20).
/// True when the question says "kul ugrahi", "total dues", "کل اگراہی": the glossary's کل اگراہی (Total
/// Dues, PRD Section 3), which is the tile of everything people owe the shop. Not when the dues are asked by
/// collector or city (R21, R22 hold the buyers' dues: the figure of that list is the buyers' tile).
///
/// Example: "what is the total dues of the shop" is true; "total dues by collector" is false.
fn total_dues_asked(scan: &Scan) -> bool {
    let is = |w: &str, list: &[&str]| list.iter().any(|x| key(x) == w);
    const WHOLE: [&str; 4] = ["kul", "total", "ٹوٹل", "کل"];
    const DUES: [&str; 8] = [
        "ugrahi",
        "ugrai",
        "ugrahee",
        "agrahi",
        "agrai",
        "dues",
        "اگراہی",
        "اگرائی",
    ];
    const BY: [&str; 12] = [
        "collector",
        "collectors",
        "wise",
        "city",
        "cities",
        "shehar",
        "wale",
        "wala",
        "wali",
        "والا",
        "والے",
        "شہر",
    ];
    if scan.words.iter().any(|w| is(w, &BY)) || scan.has(Cue::ByCity) {
        return false;
    }
    scan.words
        .windows(2)
        .any(|pair| matches!(pair, [a, b] if is(a, &WHOLE) && is(b, &DUES)))
}

fn dead_amount_included(scan: &Scan) -> bool {
    let text = format!(" {} ", scan.words.join(" "));
    let says = |phrase: &str| {
        let keyed: Vec<String> = phrase.split(' ').map(key).collect();
        text.contains(&format!(" {} ", keyed.join(" ")))
    };
    [
        "sab milake",
        "sab mila kar",
        "sab milakar",
        "sab mila ke",
        "sabhi milake",
        "jo lena hai",
        "doobi raqam samet",
        "doobi raqam ke saath",
        "doobi raqam shamil",
        "dubi raqam samet",
        "dead amount included",
        "including dead amount",
        "including the dead amount",
        "with dead amount",
        "with the dead amount",
        "سب ملا کر",
        "سب ملا کے",
        "ڈوبی رقم سمیت",
        "ڈوبی رقم کے ساتھ",
    ]
    .iter()
    .any(|p| says(p))
}

/// The first day of the books and today, as ISO dates: all the books, for a question that names no period
/// and asks for all of them ("kitne mahine biki hai").
fn books_range(reader: &Reader<'_>) -> Option<(String, String)> {
    let first = mandi_services::day::first_book_day(reader.ctx)
        .ok()
        .flatten()?;
    Some((first.to_iso(), reader.today.to_iso()))
}

/// "how did onion sales go in August", "how were sales this month": a figure asked in the past tense,
/// which is no how-to. Needs a figure word beside it, and no "you" or "calculate" (how it was made).
///
/// Example: "how did onion sales go in October" is true; "how do I print a bill" is false.
fn how_it_went(scan: &Scan, has_figure: bool) -> bool {
    let has = |list: &[&str]| scan.words.iter().any(|w| list.contains(&w.as_str()));
    has_figure
        && has(&["how"])
        && has(&["did", "was", "were", "has", "have", "had"])
        && !has(&[
            "you",
            "your",
            "calculate",
            "calculated",
            "work",
            "worked",
            "make",
            "made",
            "come",
            "came",
            "get",
            "got",
        ])
}

/// Doubts that always send the question to the model.
const BLOCKING: [Doubt; 12] = [
    Doubt::NothingToRead,
    Doubt::SeveralMatches,
    Doubt::NameOrItem,
    Doubt::MixedIntent,
    Doubt::LookupFailed,
    Doubt::Cut,
    Doubt::TenseUnclear,
    Doubt::TimeWord,
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
        // A how-to that stands only on "what is" (Medium) and has words the app does not know in it is
        // not sure: "what is Partner As totel in the shop" asks a figure, and the guide of the partner
        // screen is no answer to it (D7 O2).
        Intent::HowTo => {
            u.topic
                .as_ref()
                .is_some_and(|t| t.confidence >= Confidence::Medium)
                && !(u.intent.confidence == Confidence::Medium
                    && u.doubts.contains(&Doubt::UnknownWords))
        }
        Intent::Forecast | Intent::WhatIf => !party_locked,
        Intent::Advice | Intent::ChangeRequest | Intent::NotInApp => true,
    }
}

/// True when the question names ONE month that is after every month an estimate can reach: the
/// estimator goes at most `MAX_MONTHS_AHEAD` months ahead and the current month is the first of them,
/// so a month that many or more months after this one is out of reach (D8 H9).
///
/// Example: today 10-10-2026, "december 2027" is 14 months after October 2026: true. "december 2026"
/// is 2 months after: false (the model sets up an estimate).
fn beyond_estimate(periods: &[PeriodRef], today: NaiveDate) -> bool {
    use chrono::Datelike;
    let [only] = periods else {
        return false;
    };
    let super::PeriodKind::Month { year, month } = only.kind else {
        return false;
    };
    // Whole months since year 0, with checked maths (this crate does no plain arithmetic).
    let months = |y: i32, m: u32| i64::from(y).checked_mul(12)?.checked_add(i64::from(m));
    let (Some(target), Some(now)) = (months(year, month), months(today.year(), today.month()))
    else {
        return false;
    };
    only.future
        && target
            .checked_sub(now)
            .is_some_and(|ahead| ahead >= i64::from(crate::limits::MAX_MONTHS_AHEAD))
}
