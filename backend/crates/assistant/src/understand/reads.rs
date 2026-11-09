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
//! | cash, advances, dead amount, money lent, now | the tiles |
//! | the shop's dues, now | the tiles, and R20 (the real list, with the total over the whole filter) |
//! | the shop's payables, now | R24 |
//! | goods left now | R10, oldest first |
//! | cash, expenses, day summary, day book, receipts over a period | R12, R29, R34, R11, R23 |
//! | the trial balance | R18 |
//! | a list of buyers' dues or sellers' payables | R20 or R24, sorted |
//! | the top buyers, sellers or items by commission | view H (A, B, C sorted for the lowest or for sales) |
//! | the partners' totals, one partner's total (kul raqam, not his capital) | R31 for every partner (the answer cites his row: only that read carries the checked total) |
//! | dues of a city; sales of an اڈا; expenses (head) | R21; R8; R29 with the period |
//!
//! `None` when the question is not completely understood, compares things, has two figures, names
//! more than one party, item or period, names a garden, a trip or own goods, asks what was given,
//! paid or received (an event, not what is open now), asks a rate with no period, or is not covered:
//! the model decides then. A ranking or period the app assumed (`Understanding::assumed`) is read as assumed;
//! the caller states it in the answer.
//!
//! A question that names two or three parties, or two or three items, with one figure and one period,
//! gets one `find` and one read for each ("akram aur arshad ka bakaya").
//!
//! [`plan`] also says which figures an instant answer gives from the reads (PRD FR-7.90): the lines
//! `compose_figure` writes. A read that is not for an instant answer (a khata, a list) has none.
//!
//! Example (today 10-10-2026): "kele ka commission pichle mahine" gives `period` last month, `find`
//! banana as an item (by its own name, not by the words typed), and view A with `item` the key and
//! 2026-09-01 to 2026-09-30.

mod totals;

pub(super) use totals::{pay_asked, pay_kind, qty_asked, value_asked};

use mandi_reports::analytics::{AnalyticsCompare, compare_window};
use serde_json::{Value, json};

use super::lexicon::{Cue, key};
use super::scan::Scan;
use super::who::Names;
use super::{
    CapitalFlow, CashDir, Doubt, FigureKind, GradeWord, GroupWord, Intent, ItemHit, ListOver,
    Measure, Outcome, PartyHit, PartyRole, PayKind, PeriodKind, PeriodRef, Rank, RecordHit,
    RecordKind, TrialSide, Understanding,
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
const NOT_CARRIED: [Doubt; 7] = [
    Doubt::UnknownWords,
    Doubt::TimeWord,
    Doubt::CloseSpelling,
    Doubt::OtherRecord,
    Doubt::Comparison,
    Doubt::PartnerLocked,
    Doubt::NoPeriod,
];

/// Words that scope a figure to a garden, a trip, own goods or the guests. The reads have no filter
/// for them, so a question with one is the model's.
const SCOPE_WORDS: [&str; 20] = [
    "baghat",
    "baghaat",
    "baghon",
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

/// Words that say "my own" or "ours" before a word for goods: "apna maal", "apne maal", "hamara mal".
const OWN_WORDS: [&str; 12] = [
    "apna",
    "apne",
    "apni",
    "hamara",
    "hamare",
    "hamari",
    "own",
    "our",
    "اپنا",
    "اپنے",
    "اپنی",
    "ہمارا",
];

/// Words for goods.
const GOODS_WORDS: [&str; 7] = ["maal", "mal", "goods", "saman", "saaman", "مال", "سامان"];

/// True when the words say "own goods": an own-word at most two words before a word for goods
/// ("apna maal", "apne maal", "hamara apna mal").
fn own_goods(scan: &Scan) -> bool {
    let is = |list: &[&str], word: &str| list.iter().any(|x| key(x) == word);
    scan.words.iter().enumerate().any(|(at, word)| {
        is(&OWN_WORDS, word)
            && (1..=2).any(|gap| {
                scan.word(at.saturating_add(gap))
                    .is_some_and(|next| is(&GOODS_WORDS, next))
            })
    })
}

/// True when the question names a garden, a trip, own goods, the guests or the walk-in buyers (چکتا
/// کھاتہ), by word. The guests are no scope for the fruit taken: the fruit register has a read for them
/// (R37, partner `guests`). The walk-in buyers have no read of their own for a figure: they have no
/// balance and are no part of the shop's dues ("chukta khata ka kul baqaya", PRD FR-13.6).
pub(super) fn scoped(scan: &Scan, kinds: &[FigureKind]) -> bool {
    let guests_fruit = kinds.contains(&FigureKind::FruitTaken);
    own_goods(scan)
        || scan.has_exact(Cue::Chukta)
        || scan.words.iter().any(|w| {
            SCOPE_WORDS
                .iter()
                .any(|s| key(s) == *w && !(guests_fruit && is_guest_word(s)))
        })
        || scan.has_exact(Cue::RoleGarden)
        || scan.has_exact(Cue::RoleTrip)
}

/// The words for the shop's guests in [`SCOPE_WORDS`].
fn is_guest_word(word: &str) -> bool {
    ["mehman", "مہمان", "guest", "guests"].contains(&word)
}

/// True when a print or an open-the-report request names a scope its link cannot carry: own goods or
/// the guests. A garden or a trip word with no record named is the gardens or the trips report of the
/// days ([`garden_word`], [`trip_word`]); a garden or a trip that was found has its own report (R36,
/// R33). The bare word "goods" is not a scope here: "unsold goods print" is the stock.
///
/// Example: "apne maal ki is mahine ki bikri ka print": true (the report would be every sale).
pub(super) fn print_scope_missing(scan: &Scan, kinds: &[FigureKind]) -> bool {
    // The guests' fruit has a register of its own (R37 with guests only): no scope is missing.
    own_goods(scan) || (scan.has_exact(Cue::Guests) && !kinds.contains(&FigureKind::FruitTaken))
}

/// The بکری number the question names: "bikri number 5", "bikri no 5", "bikri 5", "بکری نمبر 5",
/// "seller bill 5". A plain number straight after the word, or after "number" / "no" / "نمبر".
///
/// Example: "bikri number 5 ka print" is 5; "5 peti bikri" is none.
pub(super) fn bikri_number(scan: &Scan) -> Option<i64> {
    scan.hits.iter().find_map(|h| {
        if h.cue != Cue::Sale {
            return None;
        }
        let mut at = h.end;
        // The word for "number" between.
        if scan.word(at).is_some_and(|w| {
            ["number", "no", "nmbr", "نمبر", "num"]
                .iter()
                .any(|x| key(x) == w)
        }) {
            at = at.saturating_add(1);
        }
        scan.digits_at(at).filter(|n| *n > 0)
    })
}

/// True when the question names a document by its number: the word "number" / "no" / "نمبر" straight
/// before a plain number ("wasooli number 3", "voucher no 12"). No figure read has a filter for a
/// voucher or a بکری number, so such a question is the model's (D7 R3, R7).
///
/// Example: "aaj wasooli number 3 kitni thi" is true; "aaj 3 peti bikri" is false.
pub(super) fn document_number(scan: &Scan) -> bool {
    // A document word straight before a small plain number, with no "number" between them: "wasooli 3",
    // "voucher 3", "و-3" (money review 10). A bigger number is an amount ("wasooli 50000 se zyada").
    // Taking it for a number is the safe way: the model is asked, no figure is read ahead.
    const SMALL: i64 = 100_000;
    const DOCUMENT_WORDS: &[&str] = &[
        "voucher",
        "vouchers",
        "receipt",
        "receipts",
        "payment",
        "payments",
        "advance",
        "raseed",
        "adaigi",
        "adayegi",
        "peshgi",
        "journal",
        "واؤچر",
        "رسید",
        "ادائیگی",
        "پیشگی",
        "جنرل",
        "و",
        "ا",
        "پ",
    ];
    let after_document_word = |at: usize| {
        let before = at.checked_sub(1);
        before.is_some_and(|b| {
            scan.word(b)
                .is_some_and(|w| DOCUMENT_WORDS.iter().any(|x| key(x) == w))
                || scan
                    .hits
                    .iter()
                    .any(|h| h.cue == Cue::Collection && h.end == at)
        })
    };
    scan.words.iter().enumerate().any(|(at, w)| {
        let named = ["number", "no", "nmbr", "نمبر", "num", "nambar"]
            .iter()
            .any(|x| key(x) == *w)
            && scan.digits_at(at.saturating_add(1)).is_some_and(|n| n > 0);
        named || (scan.digits_at(at).is_some_and(|n| n > 0 && n < SMALL) && after_document_word(at))
    })
}

/// True when a packing word stands as a FILTER of a flow: "kele ki peti ki bikri" (the sales of banana
/// in the peti packing). A unit word with "ka / ki / ke" after it and a sale, commission or Laga word
/// next, and no "kitne" / "kitni" before it ("kitni peti bikri" asks a quantity). No view or report has a
/// packing filter, so the plain figure would be the figure of all the packings (D7 R6).
///
/// Example: "aaj kele ki peti ki bikri kitni hui" is true; "aaj kitni peti bikri hui" is false.
pub(super) fn packing_filter(scan: &Scan) -> bool {
    let is = |w: &str, list: &[&str]| list.iter().any(|x| key(x) == w);
    scan.hits.iter().filter(|h| h.cue == Cue::Unit).any(|unit| {
        let genitive = scan
            .word(unit.end)
            .is_some_and(|w| is(w, &["ka", "ki", "ke", "کا", "کی", "کے"]));
        let flow_next = scan.hits.iter().any(|h| {
            matches!(
                h.cue,
                Cue::Sale | Cue::SalePlural | Cue::Commission | Cue::Laga | Cue::Rate
            ) && h.start >= unit.end
                && h.start <= unit.end.saturating_add(2)
        });
        let counted = unit
            .start
            .checked_sub(1)
            .and_then(|b| scan.word(b))
            .is_some_and(|w| is(w, &["kitne", "kitni", "kitnay", "کتنے", "کتنی"]));
        genitive && flow_next && !counted
    })
}

/// True when the question says "garden" and no garden was found for it.
pub(super) fn garden_word(scan: &Scan, names: &Names) -> bool {
    let found = names
        .parties
        .iter()
        .any(|p| p.outcome.hit().is_some_and(|h| h.role == PartyRole::Garden));
    !found
        && (scan.has_exact(Cue::RoleGarden)
            || scan.words.iter().any(|w| {
                [
                    "bagh",
                    "bag",
                    "baag",
                    "baagh",
                    "garden",
                    "gardens",
                    "باغ",
                    "باغات",
                ]
                .iter()
                .any(|s| key(s) == *w)
            }))
}

/// True when the question says "trip" and no trip was found for it.
pub(super) fn trip_word(scan: &Scan, names: &Names) -> bool {
    let found = names
        .parties
        .iter()
        .any(|p| p.outcome.hit().is_some_and(|h| h.role == PartyRole::Trip));
    !found
        && (scan.has_exact(Cue::RoleTrip)
            || scan.words.iter().any(|w| {
                ["safar", "trip", "trips", "سفر"]
                    .iter()
                    .any(|s| key(s) == *w)
            }))
}

/// True when the question asks what was GIVEN, PAID or RECEIVED (an event of a period), and not what
/// is open now: "rahim ko kitni peshgi di", "rahim ko kitni adaigi hui", "aaj kitna cash aya". The
/// card of a party and the tiles hold what is open now, so such a question is the model's (money
/// review R-e). A balance is asked with a "give" verb only ("akram ko kitne paise diye"); "kitna
/// hua" is a way to ask a balance.
pub(super) fn happened(scan: &Scan, kinds: &[FigureKind]) -> bool {
    // A STATE, not an event: "tahir ne kitni peshgi li hui hai" (the advance still open), "kitna ho gaya hai"
    // (what it has come to), "have to pay", "needs to pay", "is left", "is open", "bachi hui hai",
    // "کھلی ہوئی", "put into the business" (D7, plan 59 F3b). These words say what is open now, even
    // though they hold a past participle. Only a word that gives something BACK keeps it an event.
    {
        let text = format!(" {} ", scan.words.join(" "));
        const STATE: [&str; 29] = [
            "li hui hai",
            "li hui he",
            "liya hua hai",
            "liya hua he",
            "li hui hain",
            "bachi hui",
            "bacha hua",
            "bache hue",
            "khuli hui",
            "khula hua",
            "lagi hui",
            "laga hua",
            "lagaya hua",
            "lagayi hui",
            "have to pay",
            "has to pay",
            "need to pay",
            "needs to pay",
            "due to pay",
            "is left",
            "are left",
            "is open",
            "are open",
            "is pending",
            "is still",
            "لی ہوئی ہے",
            "لیا ہوا ہے",
            "بچی ہوئی",
            "کھلی ہوئی",
        ];
        // "ho gaya hai" ("kitna ho gaya hai": what it has come to) is a state only beside a word for what is
        // still owed. Beside a collection or a payment ("wasooli ho gayi", "adaigi ho gayi") it is an event.
        const HO_GAYA: [&str; 8] = [
            "ho gaya",
            "ho gayi",
            "ho gaye",
            "ho gya",
            "ہو گیا",
            "ہو گئی",
            "ہو گئے",
            "ہو گی",
        ];
        const OWED: [&str; 12] = [
            "baqi",
            "baaqi",
            "bakaya",
            "bkaya",
            "bqaya",
            "udhar",
            "udhaar",
            "peshgi",
            "بقایا",
            "باقی",
            "ادھار",
            "پیشگی",
        ];
        const GIVES_BACK: [&str; 8] = [
            "wapas", "wapsi", "wapis", "lautaya", "nikala", "withdrew", "returned", "واپس",
        ];
        let gives_back = scan
            .words
            .iter()
            .any(|w| GIVES_BACK.iter().any(|x| key(x) == *w));
        let state_kinds = kinds.iter().all(|k| {
            matches!(
                k,
                FigureKind::Advance
                    | FigureKind::Balance
                    | FigureKind::Dues
                    | FigureKind::Payable
                    | FigureKind::MoneyLent
                    | FigureKind::Capital
            )
        });
        let says_phrase =
            |list: &[&str]| list.iter().any(|p| text.contains(&format!(" {} ", key(p))));
        let ho_gaya_state =
            says_phrase(&HO_GAYA) && scan.words.iter().any(|w| OWED.iter().any(|x| key(x) == *w));
        if state_kinds && !kinds.is_empty() && !gives_back && (says_phrase(&STATE) || ho_gaya_state)
        {
            return false;
        }
    }
    // Words that give or take: never a way to ask what is open now.
    const FIRM: &[&str] = &[
        "di",
        "dee",
        "diya",
        "diye",
        "dia",
        "دی",
        "دیا",
        "دیے",
        "li",
        "lia",
        "liya",
        "liye",
        "لی",
        "لیا",
        "لیے",
        "kiya",
        "kia",
        "kiye",
        "gayi",
        "gaya",
        "gaye",
        "chuki",
        "chuka",
        "chuke",
        "paid",
        "pay",
        "given",
        "give",
        "gave",
        "take",
        "took",
        "taken",
        "got",
        "گئی",
        "گیا",
        "کیا",
        // Return: "peshgi wapsi kitni hai" is what was given back, not what is open.
        "wapsi",
        "wapis",
        "wapas",
        "واپسی",
        "واپس",
        "lautaya",
        "lautai",
        "lautaye",
        // Sent: "peshgi bheji".
        "bheja",
        "bheji",
        "bheje",
        "sent",
        "بھیجا",
        "بھیجی",
        "بھیجے",
        "gai",
        "gae",
    ];
    // Words that say it happened but are also a way to ask a balance ("kitna hua").
    const SOFT: [&str; 11] = [
        "hui", "hue", "hua", "mili", "mila", "ہوئی", "ہوئے", "ہوا", "ملی", "ملا", "received",
    ];
    // "adaigi ki hai", "peshgi kitni ki thi": the genitive "ki" alone is "of" ("rahim ki adaigi"),
    // but "ki hai" and "ki thi" are the past of "to do".
    const DONE: [[&str; 2]; 6] = [
        ["ki", "hai"],
        ["ki", "he"],
        ["ki", "thi"],
        ["کی", "ہے"],
        ["کی", "تھی"],
        ["ki", "gayi"],
    ];
    let has = |list: &[&str]| scan.words.iter().any(|w| list.iter().any(|x| key(x) == *w));
    let done = scan.words.windows(2).any(|pair| {
        DONE.iter().any(|d| {
            pair.first().is_some_and(|a| key(d[0]) == *a)
                && pair.get(1).is_some_and(|b| key(d[1]) == *b)
        })
    }) || scan.words.last().is_some_and(|w| w == "ki" || w == "کی")
        && scan.words.len() > 2;
    let open_figure = kinds.iter().any(|k| {
        matches!(
            k,
            FigureKind::Advance | FigureKind::Payable | FigureKind::MoneyLent
        )
    });
    let balance_figure = kinds
        .iter()
        .any(|k| matches!(k, FigureKind::Balance | FigureKind::Dues));
    // "sarmaya lagaya", "sarmaya wapas liya": what was put in or taken out, not the capital now.
    const CAPITAL_EVENT: &[&str] = &[
        "lagaya",
        "lagayi",
        "lagaye",
        "dala",
        "dali",
        "daale",
        "wapas",
        "nikala",
        "nikalwaya",
        "لگایا",
        "invested",
        "invest",
        "put",
        "invests",
    ];
    let capital_event = kinds.contains(&FigureKind::Capital)
        && (has(CAPITAL_EVENT) || has(FIRM) || has(&["واپس", "ڈالا", "لگائی"]));
    // The cash in hand now is not the cash that came in today.
    let cash_figure = kinds.contains(&FigureKind::Cash);
    // "cash in", "cash out": what came in or went out, not the cash in hand ("cash in hand" is one
    // phrase, so its "in" is inside the hit).
    let cash_flow = scan.hits.iter().any(|h| {
        h.cue == Cue::Cash
            && scan.word(h.end).is_some_and(|w| {
                ["in", "out", "aamad", "kharch", "آمد", "خرچ"]
                    .iter()
                    .any(|x| key(x) == w)
            })
    });
    // A verb that says money or goods MOVED ("nikala", "came", "paid", "chukaya"): a state figure is not what
    // was asked beside one (D7, plan 59 F3b, money review).
    let moved = scan
        .words
        .iter()
        .any(|w| super::function_words::is_event_word(w));
    let state_figure =
        open_figure || balance_figure || cash_figure || kinds.contains(&FigureKind::Capital);
    capital_event
        || (state_figure && moved)
        || ((open_figure || balance_figure) && (has(FIRM) || done))
        || (open_figure && (has(&SOFT) || scan.has(Cue::Received)))
        || (cash_figure && (has(FIRM) || has(&SOFT) || scan.has(Cue::Received) || cash_flow))
}

/// True when a word of the question names another measure than the one the reads hold: the plain
/// figure would be a real number for the wrong thing (money review, 05-11-2026).
/// - A flow with a nag, a unit, a count or "goods" word asks the quantity ("kitne nag bike"), not the
///   amount.
/// - "safi", "net", "naqdi", "udhari", "percent", "average" name another sale or another number.
/// - A role word beside a flow with no name ("buyers ka commission") asks a list, not the shop's sum.
/// - "khalis" and "net" are a partner's خالص (C20), and "total raqam" or "ٹوٹل رقم" is not his کل رقم.
/// - "udhar" without "raqam" is credit, not the money lent.
///
/// Example: "aaj kitne nag bike" is true; "aaj ki bikri kitni hai" is false.
pub(super) fn other_measure(
    scan: &Scan,
    kinds: &[FigureKind],
    roles: &[PartyRole],
    has_item: bool,
    ranking: bool,
) -> Measure {
    const MEASURE: [&str; 26] = [
        "safi",
        "saafi",
        "net",
        "udhari",
        "udhaari",
        "udhri",
        "percent",
        "percentage",
        "fisad",
        "feesad",
        "average",
        "avg",
        "ausat",
        "tadad",
        "taadad",
        "tadaad",
        "qty",
        "quantity",
        "weight",
        "wazan",
        "صافی",
        "ادھاری",
        "فیصد",
        "اوسط",
        "تعداد",
        "وزن",
    ];
    const PARTNER_OTHER: [&str; 5] = ["khalis", "khalees", "khaalis", "net", "خالص"];
    const LENT: [&str; 11] = [
        "raqam", "rakam", "رقم", "lent", "loan", "loans", "qarz", "karz", "قرض", "قرضہ", "lend",
    ];
    let has = |list: &[&str]| scan.words.iter().any(|w| list.iter().any(|x| key(x) == *w));
    let text = scan.words.join(" ");
    let says = |phrase: &str| text.contains(&key(phrase));
    let flow = kinds.iter().any(|k| {
        matches!(
            k,
            FigureKind::Sale | FigureKind::SaleCount | FigureKind::Commission | FigureKind::Laga
        )
    });
    if flow {
        // "naqdi mandi" is a buyer's name; "naqdi bikri" is the cash sales.
        let cash_sales = has(&["naqdi", "nakdi", "نقدی"]) && !has(&["mandi", "منڈی"]);
        // "commission bikri ka kitne fisad": one flow as a percent of another. Both are read, and the
        // model divides them; there is no instant figure for a ratio.
        const PERCENT: [&str; 5] = ["percent", "percentage", "fisad", "feesad", "فیصد"];
        let flows = kinds
            .iter()
            .filter(|k| {
                matches!(
                    k,
                    FigureKind::Sale | FigureKind::Commission | FigureKind::Laga
                )
            })
            .count();
        // "سبجی کی quantity کل کتنی تھی": the quantity of one item or group is the nag sold, read from the
        // same views as the amount; the answer is the model's (no instant figure).
        const COUNT: [&str; 7] = [
            "qty",
            "quantity",
            "quantities",
            "tadad",
            "taadad",
            "tadaad",
            "تعداد",
        ];
        let group_word = scan.has_exact(Cue::VegetableWord) || scan.has_exact(Cue::FruitWord);
        if (has_item || group_word)
            && has(&COUNT)
            && !cash_sales
            && !side_asked(scan)
            && !MEASURE.iter().any(|w| !COUNT.contains(w) && has(&[w]))
        {
            return Measure::Quantity;
        }
        if flows >= 2
            && has(&PERCENT)
            && !cash_sales
            && !side_asked(scan)
            && !MEASURE.iter().any(|w| !PERCENT.contains(w) && has(&[w]))
        {
            return Measure::Quantity;
        }
        // "seller commission on the banana Faisal bought": one side of the commission of a named party or
        // item. View A holds the columns of each side, so the reads are made and the model picks the
        // column; there is no instant figure (the views add both sides).
        if side_asked(scan) && !has(&MEASURE) && !cash_sales && (has_item || !roles.is_empty()) {
            return Measure::Quantity;
        }
        if has(&MEASURE) || cash_sales || side_asked(scan) {
            return Measure::Other;
        }
        // "aaj kele ki kitni bikri hui": the nag and the amount of one item are both asked, and the
        // instant figure holds only the amount; the model gives both from view A (live run 2, Q667).
        if has_item
            && roles.is_empty()
            && kinds == [FigureKind::Sale]
            && has(&["kitni", "kitna", "kitne", "کتنی", "کتنا", "کتنے"])
            && has(&["hui", "hua", "hue", "hoi", "ہوئی", "ہوا", "ہوئے"])
        {
            return Measure::Quantity;
        }
        // "kitne nag bike": the quantity is in the same views as the amount, so the reads are the
        // same; the answer is the model's.
        // "raheem ke mal ka commission": the goods of a seller are the owner of the figure, not a
        // quantity asked ("kitna mal bika" is).
        let goods_of_someone = scan.hits.iter().filter(|h| h.cue == Cue::Goods).all(|h| {
            scan.word(h.end)
                .is_some_and(|w| matches!(w, "ka" | "ki" | "ke" | "kay" | "کا" | "کی" | "کے"))
        });
        if scan.has(Cue::Unit) || (scan.has(Cue::Goods) && !goods_of_someone) {
            return Measure::Quantity;
        }
        // "kis kis din kitna commission diya": the lines of the days, not one total.
        if scan.has(Cue::Many) && !ranking {
            return Measure::Breakdown;
        }
        // "buyers ka commission", "commission from buyers on banana": a role word with no party of
        // that role named asks the buyers' side or one line for each of them, not the sum of both.
        // A role word that names the side of a commission ("بیوپاری کا کمیشن") is no list of that role.
        let role_word = |many: Cue, role: Cue| {
            scan.has(many)
                || scan
                    .hits
                    .iter()
                    .any(|h| h.cue == role && !super::who::is_side_word(scan, h))
        };
        let buyer_word =
            role_word(Cue::ManyBuyers, Cue::RoleBuyer) && !roles.contains(&PartyRole::Buyer);
        let seller_word =
            role_word(Cue::ManySellers, Cue::RoleSeller) && !roles.contains(&PartyRole::Seller);
        let item_word = scan.has_any(&[Cue::ManyItems, Cue::RoleItem]) && !has_item;
        if !ranking && (buyer_word || seller_word || item_word) {
            return Measure::Other;
        }
    }
    // The partner's total is asked with its own words ("kul raqam", "partner total"). "khalis" and
    // "net" are his خالص (C20); "total raqam" and "ٹوٹل رقم" are the words of a total due.
    if kinds.contains(&FigureKind::PartnerTotal)
        && ((has(&PARTNER_OTHER) && !says("net worth")) || says("total raqam") || says("ٹوٹل رقم"))
    {
        return Measure::Other;
    }
    // "udhar" alone is credit; the money lent is "udhar raqam".
    if kinds.contains(&FigureKind::MoneyLent) && !has(&LENT) {
        return Measure::Other;
    }
    Measure::Same
}

/// True when a commission or a laga is asked for one side: "seller commission", "buyer laga". The
/// views hold the commission of both sides added together, so the sum would be the wrong figure.
fn side_asked(scan: &Scan) -> bool {
    scan.hits
        .iter()
        .any(|side| super::who::is_side_word(scan, side))
}

/// Words that ask for the whole of a figure: "total", "kul", "dono", "both", "combined", "overall".
/// Left out on purpose (money review): "mila" and "ملا" (they mean "received": "Faisal se kitna commission
/// mila" asks the buyer side), "کل" in Urdu letters (it is also "yesterday") and "sab" (as in "sab se
/// zyada"). "mila kar" ("added together") is the phrase `WHOLE_PHRASES`.
const WHOLE_WORDS: [&str; 8] = [
    "total",
    "kul",
    "ٹوٹل",
    "dono",
    "دونوں",
    "both",
    "combined",
    "overall",
];

/// Phrases that ask for the sum of the parts.
const WHOLE_PHRASES: [&str; 2] = ["mila kar", "ملا کر"];

/// True when the question says the rate is the buyer's: a buyer word right before a rate word, with at
/// most "ka / ki / ke" between ("gahak ka rate", "buyer rate", "گاہک کا ریٹ").
///
/// Example: "gahak ka aakhri rate" is true; "kele ka last rate" is false.
pub(super) fn buyer_rate(scan: &Scan) -> bool {
    const RATE: [&str; 9] = [
        "rate", "rates", "bhao", "bhaw", "price", "qeemat", "ریٹ", "بھاؤ", "قیمت",
    ];
    // Words that may stand between the buyer word and the rate: "gahak ka aakhri rate".
    const BETWEEN: [&str; 14] = [
        "ka",
        "ki",
        "ke",
        "کا",
        "کی",
        "کے",
        "last",
        "latest",
        "aakhri",
        "akhri",
        "aakhir",
        "آخری",
        "pichla",
        "پچھلا",
    ];
    let is = |w: &str, list: &[&str]| list.iter().any(|x| key(x) == w);
    scan.hits
        .iter()
        .filter(|h| matches!(h.cue, Cue::RoleBuyer | Cue::ManyBuyers))
        .any(|hit| {
            let mut at = hit.end;
            // At most two words in between.
            for _ in 0..3 {
                match scan.word(at) {
                    Some(w) if is(w, &RATE) => return true,
                    Some(w) if is(w, &BETWEEN) => at = at.saturating_add(1),
                    _ => return false,
                }
            }
            false
        })
}

/// The side of the commission a question asks (PRD 18.2), when it says one, so that the answer is that
/// side's column and never the total of both sides and Laga (D7, plan 59 F1, Q2062).
///
/// - A buyer or seller word standing right before the commission ("buyer commission", "گاہک کا کمیشن")
///   says the side.
/// - "gahak se commission", "commission from buyers": the one the commission is taken from.
/// - One buyer found by name, with the commission asked and no word for the whole ("Faisal se kele par
///   kitna commission liya"): the buyer side, as the instant answer of a buyer's commission already is
///   (`figure.rs`). A seller found by name is not a side (his commission is shown as two labelled
///   lines, Umar 05-11-2026).
///
/// `None` when the question asks no commission, asks a Laga too, names both sides, or names no side.
///
/// Example: "pichle mahine Faisal se kele par kitna commission liya" with Faisal a buyer is `Buyer`;
/// "kele ka commission pichle mahine" is `None`.
pub(super) fn commission_side(
    scan: &Scan,
    kinds: &[FigureKind],
    parties: &[super::PartyRef],
) -> Option<super::CommissionSide> {
    use super::CommissionSide as Side;
    if !kinds.contains(&FigureKind::Commission) {
        return None;
    }
    // "Faisal se commission aur laga": the buyer's commission and his Laga are two figures; the side is
    // still the buyer's, so the total of both sides is never one of them (money review).
    let laga = kinds.contains(&FigureKind::Laga);
    let of_cue = |cue: Cue| match cue {
        Cue::RoleBuyer | Cue::ManyBuyers => Some(Side::Buyer),
        Cue::RoleSeller | Cue::ManySellers => Some(Side::Seller),
        _ => None,
    };
    // A role word right before the commission names the side.
    let mut sides: Vec<Side> = scan
        .hits
        .iter()
        .filter(|h| super::who::is_side_word(scan, h))
        .filter_map(|h| of_cue(h.cue))
        .collect();
    // "gahak se commission", "commission from buyers": the role word stands next to "se" or "from".
    let from_words = ["se", "سے"];
    for hit in scan.hits.iter().filter(|h| of_cue(h.cue).is_some()) {
        let after = scan.word(hit.end).is_some_and(|w| from_words.contains(&w));
        let before = hit
            .start
            .checked_sub(1)
            .and_then(|at| scan.word(at))
            .is_some_and(|w| w == "from");
        if (after || before)
            && let Some(side) = of_cue(hit.cue)
        {
            sides.push(side);
        }
    }
    sides.dedup();
    match sides.as_slice() {
        // A seller side and a Laga (which only a buyer pays) do not go together.
        [Side::Seller] if laga => return None,
        [only] => return Some(*only),
        [] => {}
        _ => return None,
    }
    // "kitna kamishan kata", "commission deducted": taken off a seller, so the seller side (D7, plan 59 F3).
    if scan.has_exact(Cue::Deducted) {
        let found_sellers = parties
            .iter()
            .filter(|p| matches!(&p.outcome, Outcome::Found(hit) if hit.role == PartyRole::Seller))
            .count();
        if found_sellers >= 1 && !laga {
            return Some(Side::Seller);
        }
    }
    // One buyer found by name, and the question does not ask for the whole.
    let text = scan.words.join(" ");
    let whole = scan
        .words
        .iter()
        .any(|w| WHOLE_WORDS.iter().any(|x| key(x) == *w))
        || WHOLE_PHRASES.iter().any(|p| text.contains(&key(p)));
    let found: Vec<PartyRole> = parties
        .iter()
        .filter_map(|p| match &p.outcome {
            Outcome::Found(hit) => Some(hit.role),
            _ => None,
        })
        .collect();
    (found == [PartyRole::Buyer] && !whole).then_some(Side::Buyer)
}

// ---- the shapes of the inputs --------------------------------------------------------------------------

fn find(text: &str, role: &str) -> PlannedRead {
    PlannedRead {
        tool: "find",
        input: json!({"text": text, "role": role}),
    }
}

/// A planned read with one more field of its input set (a filter the plan structs do not carry yet).
fn with_field(mut read: PlannedRead, name: &str, value: Value) -> PlannedRead {
    if let Some(map) = read.input.as_object_mut() {
        map.insert(name.to_owned(), value);
    }
    read
}

fn party_card(key: &str) -> PlannedRead {
    PlannedRead {
        tool: "party_card",
        input: json!({"party": key}),
    }
}

/// An `app_state` read (plan 55 §11.1): the rule that wins for a party or an item, the attention
/// list, or the closed days. Every field of the tool's input is written.
fn app_state(what: &str, party: Option<String>, item: Option<String>) -> PlannedRead {
    app_state_of(what, party, item, None)
}

fn app_state_of(
    what: &str,
    party: Option<String>,
    item: Option<String>,
    bikri: Option<i64>,
) -> PlannedRead {
    PlannedRead {
        tool: "app_state",
        input: json!({"what": what, "party": party, "item": item, "bikri": bikri}),
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
    /// A collector's key, from a `find` of role collector (R22, R23, R35).
    collector: Option<String>,
    /// A seller (R2 to R7, R10, R26, R27) and a زمیندار (R2 to R6, R27): fields of the tool since
    /// D6 v1.5.
    seller: Option<String>,
    zamindar: Option<String>,
    /// `paid` or `due` (R38).
    status: Option<&'static str>,
    /// One kind of voucher, in its stored word (R15).
    voucher_type: Option<&'static str>,
    /// How the report lays out its rows (R14 `detail`, R33 `trip` `item` `day` `month` `partner`,
    /// R36 `partner` `garden`).
    layout: Option<&'static str>,
    /// All the items of a group, `fruit` or `vegetable` (R2 to R4, R7, R10, R27); written only when
    /// set (a field of the tool when the schema has it).
    item_group: Option<&'static str>,
    /// The quality of the lines (R2 to R4), `first`, `second` or `third`; written only when set.
    grade: Option<&'static str>,
}

fn report(id: &str, r: Rep) -> PlannedRead {
    let mut input = json!({
        "id": id, "from": r.from, "to": r.to, "as_of": r.as_of, "party": r.party,
        "item": r.item, "city": r.city, "adda": r.adda, "collector": r.collector,
        "partner": r.partner, "trip": null, "garden": null, "bikri": null,
        "seller": r.seller, "zamindar": r.zamindar, "status": r.status,
        "voucher_type": r.voucher_type, "layout": r.layout,
        "group_by": null, "where": null, "sort": r.sort, "rows": r.rows,
        "item_group": r.item_group
    });
    if let Some(grade) = r.grade
        && let Some(map) = input.as_object_mut()
    {
        map.insert("grade".to_owned(), json!(grade));
    }
    PlannedRead {
        tool: "report",
        input,
    }
}

/// The filters of an `analytics` call. Every field of the schema is written, `null` if not set.
#[derive(Default)]
struct An {
    from: Option<String>,
    to: Option<String>,
    item: Option<String>,
    /// All the items of a group, `fruit` or `vegetable`; written only when set (a field of the tool
    /// when the schema has it).
    item_group: Option<&'static str>,
    seller: Option<String>,
    buyer: Option<String>,
    sort: Option<Value>,
    rows: Option<u32>,
    /// The quality of the lines, `first`, `second` or `third`; written (null if not set) only when the
    /// tools' schema has the `grade` filter.
    grade: Option<&'static str>,
    /// The kind of goods: `consigned`, `trip` or `garden`.
    goods: Option<&'static str>,
    /// The commission side: `buyer` or `seller`; written only when set.
    side: Option<&'static str>,
}

fn analytics(view: &str, a: An) -> PlannedRead {
    let mut input = json!({
        "view": view, "from": a.from, "to": a.to, "group_by": null, "item": a.item,
        "item_group": a.item_group, "seller": a.seller, "buyer": a.buyer, "city": null,
        "adda": null, "garden": null, "trip": null, "goods": a.goods, "partner": null,
        "side": a.side, "compare": null, "sort": a.sort, "rows": a.rows
    });
    if grade_ready()
        && let Some(map) = input.as_object_mut()
    {
        map.insert("grade".to_owned(), json!(a.grade));
    }
    PlannedRead {
        tool: "analytics",
        input,
    }
}

/// True when the tools' schema (the prompt's `tools.json`) has the quality filter `grade` (on analytics,
/// view A, only): the reads of a question about one quality then carry it. Until then such a question
/// reads the sale lines.
fn grade_ready() -> bool {
    include_str!("../../prompt/tools.json").contains("\"grade\"")
}

/// The quality a question names: awwal, doyam, soyam, اول دوم سوم, first, second or third quality, A, B or
/// C grade, and their usual misspellings. The English ordinals count only beside a quality word or a
/// named item or group ("second apple rate"); "doyam", "soyam" and their spellings count anywhere.
///
/// Example: "doyam kele ka aaj ka rate" is the second quality.
pub(super) fn grade_of(scan: &Scan, with_goods: bool) -> Option<GradeWord> {
    const FIRST: [&str; 10] = [
        "awwal", "awal", "aval", "avval", "awl", "avl", "اول", "1st", "frist", "fisrt",
    ];
    const SECOND: [&str; 9] = [
        "doyam", "doem", "doum", "doyum", "dom", "dovam", "دوم", "2nd", "dooyam",
    ];
    const THIRD: [&str; 8] = [
        "soyam", "soem", "soum", "soyum", "soym", "som", "سوم", "3rd",
    ];
    const FIRST_EN: [&str; 3] = ["first", "frst", "firt"];
    const SECOND_EN: [&str; 4] = ["second", "secnd", "sekond", "seccond"];
    const THIRD_EN: [&str; 3] = ["third", "thrd", "thired"];
    let has = |list: &[&str]| scan.words.iter().any(|w| list.iter().any(|x| key(x) == *w));
    let quality_word = has(&[
        "quality",
        "qualities",
        "grade",
        "grades",
        "darja",
        "darje",
        "darjay",
        "درجہ",
        "درجے",
        "کوالٹی",
        "گریڈ",
    ]);
    let letter = |l: &str| {
        let words: Vec<&str> = scan.words.iter().map(String::as_str).collect();
        words.windows(2).any(|p| {
            matches!(p, [a, b] if (*a == l && matches!(*b, "grade" | "quality" | "class")) || (*b == l && matches!(*a, "grade" | "quality" | "class")))
        })
    };
    let english = quality_word || with_goods;
    let mut found: Vec<GradeWord> = Vec::new();
    if has(&FIRST) || (english && has(&FIRST_EN)) || letter("a") {
        found.push(GradeWord::First);
    }
    if has(&SECOND) || (english && has(&SECOND_EN)) || letter("b") {
        found.push(GradeWord::Second);
    }
    if has(&THIRD) || (english && has(&THIRD_EN)) || letter("c") {
        found.push(GradeWord::Third);
    }
    match found.as_slice() {
        [one] => Some(*one),
        _ => None,
    }
}

/// True when the tools' schema (the prompt's `tools.json`) has the `item_group` filter: the reads of a
/// group of items are planned only then, and not before the tool can take them.
fn group_filter_ready() -> bool {
    include_str!("../../prompt/tools.json").contains("\"item_group\"")
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

/// Who a figure of the answer is about. A key is the one `find` gives (`a18`, `i1`, `s2`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Who {
    /// The whole shop.
    Shop,
    /// An item (view A with the item, or its goods left).
    Item(String),
    /// A seller (view B).
    Seller(String),
    /// A buyer (view C).
    Buyer(String),
    /// A buyer and an item: the row of the item in view A read for him (buyer key, item key).
    BuyerItem(String, String),
    /// A seller and an item: the row of the item in view A read for him (seller key, item key).
    SellerItem(String, String),
    /// A city, by its key (`ct1`): the row of that city in R21.
    City(String),
    /// A buyer or a seller whose party card is read.
    Party(String),
    /// A partner's row of R31; `None` for every partner's row.
    Partner(Option<String>),
    /// A shop expense head, by its English name (the row of R29 that has it).
    Head(String),
    /// A collector (staff), by his key (`st5`): R23 read for him.
    Collector(String),
    /// The shop's guests (the fruit register for them).
    Guests,
    /// A group of items, by the word the tools take (`fruit` or `vegetable`): view A with the group.
    Group(String),
}

/// Which figure a line of the answer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum What {
    /// A party's balance (party card).
    Balance,
    /// What the shop owes a seller (party card).
    Payable,
    /// A seller's open advance (party card).
    Advance,
    /// Cash in hand (tiles).
    Cash,
    /// The buyers' dues (tiles, and R20 for the button).
    BuyersDues,
    /// The total dues (tiles).
    TotalDues,
    /// What the shop owes sellers (R24).
    ShopPayable,
    /// Advances open (tiles).
    AdvancesOpen,
    /// The dead amount (tiles).
    DeadAmount,
    /// Money lent, open (tiles).
    MoneyLent,
    /// Sales.
    Sale,
    /// How many sales.
    SaleCount,
    /// Commission.
    Commission,
    /// Laga.
    Laga,
    /// The rate of the last sale line of an item (R2, newest first, one row), with its date.
    LastRate,
    /// Goods left (R10).
    Stock,
    /// A partner's kul raqam (R31).
    PartnerTotal,
    /// A partner's capital now (R31 for the partner, the `capital` part).
    PartnerCapital,
    /// How many نگ were sold or bought: the `nag` column of view A, B or C (D7, plan 59 S-A7).
    Qty,
    /// An item's average rate: the `avg_rate` column of its row of view A, named as the average (S-A1).
    AvgRate,
    /// Shop expenses over a period (R29): the whole of them, or one head's row (S-A5).
    Expense,
    /// Money received over a period (R23): all of it, or one collector's (S-A5).
    Receipts,
    /// The cash book's closing balance at the end of the period (R12, S-A5).
    CashClosing,
    /// Sales paid in cash over a period (R3 total), S-A8.
    CashSales,
    /// Sales paid on credit over a period (R4 total), S-A8.
    CreditSales,
    /// A seller's net sale (view B `net`), S-A8.
    NetSale,
    /// The value of the fruit taken (R37 total), S-A5.
    FruitValue,
    /// Capital put into the shop over a period (R15 `capital_in` total), S-A5.
    CapitalIn,
    /// Capital taken back over a period (R15 `capital_out` total), S-A5.
    CapitalOut,
    /// The cash received over a period: the `cash_in` total of the cash book (R12).
    CashIn,
    /// The cash paid out over a period: the `cash_out` total of the cash book (R12).
    CashOut,
    /// The total debit of the trial balance (R18 `totals.debit`).
    TrialDebit,
    /// The total credit of the trial balance (R18 `totals.credit`).
    TrialCredit,
    /// What the buyers of one city owe: the city's row of R21 (`balance`).
    CityDues,
}

/// One figure the answer gives: about whom, and which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Line {
    pub who: Who,
    pub what: What,
}

/// The reads of a question, in the order to run them, and the figures the answer gives from them.
pub(super) struct Planned {
    pub reads: Vec<PlannedRead>,
    /// The figures of an instant answer (PRD FR-7.90). Empty when the reads are not for one (a
    /// khata, a list, a day sheet): `compose_figure` then gives nothing.
    pub lines: Vec<Line>,
}

/// The most things (parties or items) one question may name and still be read by the app.
const MAX_THINGS: usize = 3;
/// The newest ledger rows an answer to "his hisab" shows with his balance.
pub(super) const LEDGER_ROWS: u32 = 7;
/// The rows of a ranking asked with no number ("sab se zyada maal kis cheez ka para hai"): the top few.
const RANKED_ROWS: u32 = 5;

/// What a plan collects: the finds its reads need, and the reads.
#[derive(Default)]
struct Plan {
    finds: Vec<PlannedRead>,
    reads: Vec<PlannedRead>,
    lines: Vec<Line>,
    /// True when a read has the dates of the period (the `period` read is then added).
    dated: bool,
    /// True when the question asks the quantity (نگ), so a sale figure is read as its `nag` column.
    qty: bool,
    /// True when the question asks the average rate.
    average: bool,
    /// True when a buyers or sellers word stands in the question.
    role_word: bool,
    /// True when a SELLERS word stands in the question.
    seller_word: bool,
}

impl Plan {
    fn find(&mut self, text: &str, role: &str) {
        let one = find(text, role);
        if !self.finds.contains(&one) {
            self.finds.push(one);
        }
    }

    fn read(&mut self, read: PlannedRead) {
        if !self.reads.contains(&read) {
            self.reads.push(read);
        }
    }

    fn line(&mut self, who: Who, what: What) {
        self.lines.push(Line { who, what });
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
    plan(u).map(|p| p.reads)
}

/// The reads to make before the first model call: the reads of the plan when the app has one, else the reads
/// of each figure it can plan for a question it understood but cannot answer whole ("today's sales, the number
/// of bills and the commission": three figures that need two reads). They are only reads: the app gives no
/// instant line from them and the scope guard does not use them, so a read that turns out useless costs a
/// few milliseconds and never a wrong figure. The model then writes the answer in ONE call instead of
/// choosing the tools first (D7, plan 59 D).
pub fn plan_pre_reads(u: &Understanding) -> Option<Vec<PlannedRead>> {
    plan_reads(u).or_else(|| pre_reads(u))
}

/// The union of the reads of each figure of the question; see [`plan_pre_reads`].
fn pre_reads(u: &Understanding) -> Option<Vec<PlannedRead>> {
    let kinds = u.figures.value.as_slice();
    if !u.sure
        || u.intent.value != Intent::Figure
        || kinds.is_empty()
        || kinds.len() > MAX_PRE_FIGURES
        || kinds.iter().any(|k| matches!(k, FigureKind::Ask(_)))
        || u.periods.len() > 1
        || u.periods.iter().any(|p| p.future)
        || u.parties.len() > MAX_THINGS
        || u.items.len() > MAX_THINGS
        || (u.parties.len() > 1 && !u.items.is_empty())
        || (u.items.len() > 1 && !u.parties.is_empty())
        || u.records.len() > 1
        || u.packing_filter
        || u.quality
        || u.truck.is_some()
        || (u.bikri_number.is_some() && u.sale_number.is_none())
        || (u.document_number && u.sale_number.is_none())
        || (u.item_group.is_some()
            && u.pay_kind.is_none()
            && !u
                .figures
                .value
                .iter()
                .any(|k| matches!(k, FigureKind::Report("R3" | "R4"))))
        || !names_found(u)
    {
        return None;
    }
    let period = u.periods.first();
    let period = period.filter(|p| !(is_bare_kal(p) && asks_now_or_trial(u)));
    let period = period.filter(|p| !(p.kind == PeriodKind::UpToNow && asks_now(u)));
    let mut finds: Vec<PlannedRead> = Vec::new();
    let mut reads: Vec<PlannedRead> = Vec::new();
    let mut dated = false;
    // One named person, garden, trip, khata or partner: the report that holds its figures (a day sheet, a
    // garden's or a trip's report, a khata's ledger, a partner's row of view F).
    if let Some(extra) = pre_reads_of_one_record(u, period)
        .or_else(|| pre_reads_of_sales(u, period))
        .or_else(|| pre_reads_of_chukta(u, period))
    {
        dated = extra.dated;
        finds = extra.finds;
        reads = extra.reads;
    }
    // The figure of each kind is read from its own tile, view or report only for a question that names no
    // other measure: a buyer-side commission, a quantity, a number of a بکری or a role word has its own
    // guard, and a tile of the wrong measure must not be put in front of the model (money review).
    // It stops wherever the plan stops: a word nothing explains, a doubt the plan does not carry, an event, a
    // capital moved, a group, a payment type. The scope guard does not see these reads, so a read of the whole
    // shop for a question with a filter the read lacks would put another question's figure in front of the model
    // (money review of 08-11-2026, second pass).
    let generic = u.sale_number.is_none()
        && u.commission_side.is_none()
        && u.other_measure == Measure::Same
        && !u.role_word
        && !u.scoped
        && !u.compare
        && !u.last_one
        && !u.oldest
        && !u.months_asked
        && !u.guests
        && !u.amount_named
        && u.list.is_none()
        && !u.unexplained
        && !u.happened
        && u.capital_flow.is_none()
        && u.item_group.is_none()
        && u.pay_kind.is_none()
        && !u.doubts.iter().any(|d| NOT_CARRIED.contains(d));
    for kind in kinds.iter().filter(|_| generic) {
        let mut one = Plan {
            qty: u.qty_asked,
            average: u.average_asked,
            role_word: u.role_word,
            seller_word: u.seller_word,
            ..Plan::default()
        };
        if figure_of_kind(u, *kind, period, &mut one).is_none() {
            continue;
        }
        dated |= one.dated;
        for f in one.finds {
            if !finds.contains(&f) {
                finds.push(f);
            }
        }
        for r in one.reads {
            if !reads.contains(&r) {
                reads.push(r);
            }
        }
    }
    if reads.is_empty() {
        return None;
    }
    let mut all = Vec::new();
    if dated && let Some(p) = period {
        all.push(period_read(p)?);
    }
    all.extend(finds);
    all.extend(reads);
    Some(all)
}

/// The most figures of one question the pre-reads are made for.
const MAX_PRE_FIGURES: usize = 4;

/// The reads of a question about the walk-in buyers together (D8 H3), for the model's first call only, both with
/// the buyer filter `chukta` (the account چکتا کھاتہ, no person), from the same facts as the screens:
/// - view A (by item): the commission (seller, buyer, لاگا and the total) and the quantity;
/// - view C (by buyer): what each walk-in buyer BOUGHT, `purchase_value`, which is the buyer's value (qty x the
///   buyer's rate). View A's `gross` is the seller's amount, the same only while a line has one rate (FR-12.3),
///   so "kitne ka maal liya" is read from view C (money review of plan 70).
///
/// The app gives no instant line from them: "kitne ka maal liya" and "commission kitna bana" are two figures and
/// the model writes the answer.
///
/// Example: "aaj chukta gahakon ne kitne ka maal liya aur commission kitna bana": views A and C, today, `chukta`.
fn pre_reads_of_chukta(u: &Understanding, period: Option<&PeriodRef>) -> Option<Plan> {
    if !u.chukta_group {
        return None;
    }
    let (from, to) = dates(period);
    let mut plan = Plan::default();
    for view in ["A", "C"] {
        plan.read(analytics(
            view,
            An {
                from: from.clone(),
                to: to.clone(),
                buyer: Some(crate::tools::records::CHUKTA_ALL.to_owned()),
                ..An::default()
            },
        ));
    }
    plan.dated = period.is_some();
    Some(plan)
}

/// The reads of a question about one بکری by its number (R1 of it) or about the sales of one payment type
/// (the cash sales R3, the credit sales R4) with an item or a group, for the model's first call only.
fn pre_reads_of_sales(u: &Understanding, period: Option<&PeriodRef>) -> Option<Plan> {
    let mut plan = Plan::default();
    if let Some(number) = u.sale_number {
        plan.read(with_field(
            report("R1", Rep::default()),
            "bikri",
            json!(number),
        ));
        return Some(plan);
    }
    // The register the question names ("naqdi bikri", "udhari bikri": R3, R4) or the payment type it says.
    let named = u.figures.value.iter().find_map(|k| match k {
        FigureKind::Report(id) if matches!(*id, "R3" | "R4") => Some(*id),
        _ => None,
    });
    let register = match (named, u.pay_kind) {
        (Some(id), _) => id,
        (None, Some(PayKind::Cash)) => "R3",
        (None, Some(PayKind::Credit)) => "R4",
        (None, None) => return None,
    };
    let (from, to) = dates(period);
    let item = match u.items.as_slice() {
        [item] => match &item.outcome {
            Outcome::Found(goods) => {
                plan.find(&goods.name_en, "item");
                Some(goods.key.clone())
            }
            _ => return None,
        },
        [] => None,
        _ => return None,
    };
    plan.read(report(
        register,
        Rep {
            from,
            to,
            item,
            item_group: u.item_group.map(GroupWord::tool_word),
            ..Rep::default()
        },
    ));
    plan.dated = period.is_some();
    Some(plan)
}

/// The reads that hold the figures of ONE named record, for the model's first call only (D7, plan 59 D):
/// - a buyer or seller and one day: his day sheet (R35), which has his items, rates, cash, credit and balances;
/// - a garden: its report (R36); a trip: its report (R33); a khata (or a person who borrowed): his ledger (R13);
/// - a partner and a period: his row of view F (income share, expense share, profit, gardens, trips, fruit).
fn pre_reads_of_one_record(u: &Understanding, period: Option<&PeriodRef>) -> Option<Plan> {
    let [party] = u.parties.as_slice() else {
        return None;
    };
    let Outcome::Found(hit) = &party.outcome else {
        return None;
    };
    let text = party.text.as_str();
    let mut plan = Plan::default();
    match hit.role {
        PartyRole::Buyer | PartyRole::Seller if is_one_day(period) => {
            let (_, to) = dates(period);
            plan.find(text, role_word(hit.role)?);
            plan.read(report(
                "R35",
                Rep {
                    party: Some(hit.key.clone()),
                    as_of: to,
                    ..Rep::default()
                },
            ));
        }
        PartyRole::Garden => {
            plan.find(text, "garden");
            plan.read(with_field(
                report("R36", Rep::default()),
                "garden",
                json!(hit.key),
            ));
        }
        PartyRole::Trip => {
            plan.find(text, "trip");
            plan.read(with_field(
                report("R33", Rep::default()),
                "trip",
                json!(hit.key),
            ));
        }
        PartyRole::Khata | PartyRole::Staff => {
            plan.find(
                text,
                if hit.role == PartyRole::Khata {
                    "khata"
                } else {
                    "staff"
                },
            );
            plan.read(report(
                "R13",
                Rep {
                    party: Some(hit.key.clone()),
                    sort: Some(json!([{"column": "date", "order": "desc"}])),
                    rows: Some(LEDGER_ROWS),
                    ..Rep::default()
                },
            ));
        }
        PartyRole::Partner => {
            let (from, to) = dates(period);
            plan.find(text, "partner");
            plan.read(with_field(
                analytics(
                    "F",
                    An {
                        from,
                        to,
                        ..An::default()
                    },
                ),
                "partner",
                json!(hit.key),
            ));
            plan.dated = period.is_some();
        }
        _ => return None,
    }
    Some(plan)
}

/// The reads of a question and the figures an instant answer gives from them.
pub(super) fn plan(u: &Understanding) -> Option<Planned> {
    // A بکری number named ("kal bikri number 5 ka commission kitna tha"): no read of a figure has a filter
    // for it, so the plain figure would be the whole shop's for the day. Only its own audit is read here
    // (`Ask`); anything else is the model's, which reads R1 for that بکری (D7, plan 59 R3).
    // With or without an item, or with a group ("sabzi ki bori ki bikri"): no read has a packing filter.
    if u.packing_filter {
        return None;
    }
    if (u.bikri_number.is_some() || u.document_number)
        && !u
            .figures
            .value
            .iter()
            .any(|k| matches!(k, FigureKind::Ask(_)))
    {
        return None;
    }
    // "aaj udhaar me kitni bikri hui", "naqdi bikri": the sales of one payment type (R3 or R4).
    if let Some(planned) = totals::pay_sale_plan(u) {
        return Some(planned);
    }
    // A payment-type word that no plan above took ("sold for cash and on credit"): the plain figures are
    // another thing. The model answers (D7, plan 59 F3, D159).
    // ("credit" beside the trial balance is its side, not a payment type.)
    if u.pay_asked && u.trial_sides.is_empty() {
        return None;
    }
    // "bashir ki khalis raqam" for a seller: his net sale (view B), not a partner's total.
    if let Some(planned) = totals::net_sale_plan(u) {
        return Some(planned);
    }
    // "اس سال شراکت داروں نے کتنا سرمایہ واپس لیا": the capital that came in or went back, the vouchers of
    // that type (R15), never the drawings.
    if let Some(planned) = capital_flow_plan(u) {
        return Some(planned);
    }
    // "baghat ki ab tak apni dukan se kitni bikri": the gardens' goods sold in our shop (view A, goods
    // garden), never the whole shop.
    if let Some(planned) = garden_goods_plan(u) {
        return Some(planned);
    }
    // "total agrahi collector wise batao": a balance list asked by its title, with a figure word beside
    // it. The title wins (as in a print): the list as of today.
    if let Some(id) = titled_balance_list(u) {
        return Some(Planned {
            reads: vec![report(id, Rep::default())],
            lines: Vec::new(),
        });
    }
    // "pichle mahine se zyada", or two periods named with no comparing word ("is mahine aur pichle
    // mahine ka commission"): both periods are read, and the model writes the difference.
    if u.compare
        || (u.intent.value == Intent::Figure
            && u.periods.len() >= 2
            && u.doubts.contains(&Doubt::SeveralPeriods))
    {
        return comparison_plan(u);
    }
    // "kis se pehle wasooli karon": the buyers' dues, oldest first.
    if u.oldest {
        return oldest_dues_plan(u);
    }
    // "mooli kitne mahine biki hai": the item's sales by month; no period, so it is read before the
    // guards that want a sure period.
    if u.months_asked {
        return months_plan(u);
    }
    // "how much cash came in yesterday": the cash book names its parts, which are events of the period.
    if !u.cash_dirs.is_empty() {
        return cash_flow_plan(u);
    }
    let forecast = u.intent.value == Intent::Forecast;
    if !u.sure
        || u.compare
        || (u.two_figures && !(flow_figures(u) || (u.multi_measure && combinable_figures(u))))
        || u.scoped
        || u.happened
        || u.other_measure == Measure::Other
        || (u.periods.len() > 1 && !forecast)
        || u.parties.len() > MAX_THINGS
        || u.items.len() > MAX_THINGS
        || (u.parties.len() > 1 && !u.items.is_empty())
        || (u.items.len() > 1 && !u.parties.is_empty())
        || u.records.len() > 1
        || u.doubts.iter().any(|d| NOT_CARRIED.contains(d))
    {
        return None;
    }
    // "akram ko kela aakhri kis rate pe diya": the newest sales of the item to the buyer (R2).
    if u.last_one && u.figures.value.as_slice() == [FigureKind::Rate] {
        return last_rate_plan(u);
    }
    // The quality of a sale line (awwal, doyam, A grade): the lines of the sales, which carry it.
    if u.quality {
        return quality_plan(u);
    }
    // A group of items ("sabzi ki bikri", "phal ka commission"): the shop's view A for the group, when
    // the tool has the filter; else the model's. The rules of a group are read with `app_state`.
    if let Some(group) = u.item_group
        && !matches!(u.figures.value.as_slice(), [FigureKind::Ask(_)])
        && u.intent.value != Intent::List
    {
        return group_plan(u, group);
    }
    // A truck number: the model reads its sales (R1 by the بکری number the `find` shows).
    if let Some(truck) = &u.truck
        && matches!(u.intent.value, Intent::Figure | Intent::List)
    {
        let mut plan = Plan::default();
        plan.find(truck, "truck");
        return Some(Planned {
            reads: plan.finds,
            lines: Vec::new(),
        });
    }
    if forecast {
        return forecast_plan(u);
    }
    let period = u.periods.first();
    // A period after today, or one the question did not say plainly, is the model's.
    if period.is_some_and(|p| p.future) {
        return None;
    }
    // "گاہکوں کے ذمے کل کتنا باقی ہے": a figure of NOW asked with a bare "کل" or "kal" that no verb makes
    // yesterday is "total" (kul), not a day (D7, plan 59 F3). A past verb keeps it yesterday.
    let period = period.filter(|p| !(is_bare_kal(p) && asks_now_or_trial(u)));
    // "ab tak" beside what is true now is the same question ("akram ka bakaya ab tak", "partners ki
    // kul raqam abhi tak").
    let period = period.filter(|p| !(p.kind == PeriodKind::UpToNow && asks_now(u)));
    let mut plan = Plan {
        qty: u.qty_asked,
        average: u.average_asked,
        role_word: u.role_word,
        seller_word: u.seller_word,
        ..Plan::default()
    };
    match u.intent.value {
        Intent::Figure => figure(u, period, &mut plan)?,
        // "what is each partner's total": every partner's row of R31, which is the figure.
        Intent::List if all_partners_total(u) => figure(u, period, &mut plan)?,
        // "arshad ne kya kya khareeda": a list of his lines, which is his figure.
        Intent::List if !u.parties.is_empty() => figure(u, period, &mut plan)?,
        Intent::List => list(u, period, &mut plan)?,
        _ => return None,
    }
    let mut reads = Vec::new();
    if plan.dated
        && let Some(p) = period
    {
        reads.push(period_read(p)?);
    }
    reads.extend(plan.finds);
    reads.extend(plan.reads);
    // A quantity is read from the same views, but the figure of the amount is no answer to it. A question
    // that asks the quantity itself ("kitne nag bika") has its own line, the `nag` column (S-A7).
    let lines = if u.qty_asked {
        totals::qty_lines(plan.lines)
    } else if matches!(u.other_measure, Measure::Quantity | Measure::Breakdown) {
        Vec::new()
    } else {
        plan.lines
    };
    let asked = lines.len();
    let lines = totals::keep_lines(u, lines);
    // The gate: with several measures and the gate OFF the reads stay and no line is given.
    let lines = if u.figures.value.len() > 1 && !u.multi_measure {
        Vec::new()
    } else {
        lines
    };
    let asked = if u.multi_measure { asked } else { 0 };
    // Several figures asked and a line dropped (a total that a doubt keeps back, a fruit value nobody asked):
    // the answer would leave a part of the question out, so the model answers (D7, plan 59 C).
    if u.figures.value.len() > 1 && lines.len() < asked {
        return None;
    }
    (!reads.is_empty()).then_some(Planned { reads, lines })
}

/// Capital in or out of the shop ("kitna sarmaya wapas liya", "how much capital did partners put in"):
/// the capital vouchers of the period, `capital_out` or `capital_in` (R15), whose totals are the amount.
/// The direction must be one of the two; capital taken back is not the partners' drawings (R29).
///
/// Example: "partners ne is saal kitna sarmaya wapas liya" is R15 `capital_out` for the year.
fn capital_flow_plan(u: &Understanding) -> Option<Planned> {
    if !u.sure
        || !matches!(u.intent.value, Intent::Figure | Intent::List)
        || u.figures.value.as_slice() != [FigureKind::Capital]
        || !u.parties.is_empty()
        || !u.items.is_empty()
        || !u.records.is_empty()
        || u.periods.len() > 1
        || u.periods.iter().any(|p| p.future)
    {
        return None;
    }
    let voucher_type = match u.capital_flow? {
        CapitalFlow::Out => "capital_out",
        CapitalFlow::In => "capital_in",
    };
    let period = u.periods.first();
    let (from, to) = dates(period);
    let mut reads = Vec::new();
    if let Some(p) = period {
        reads.push(period_read(p)?);
    }
    reads.push(report(
        "R15",
        Rep {
            from,
            to,
            voucher_type: Some(voucher_type),
            ..Rep::default()
        },
    ));
    let what = match u.capital_flow? {
        CapitalFlow::Out => What::CapitalOut,
        CapitalFlow::In => What::CapitalIn,
    };
    Some(Planned {
        reads,
        lines: totals::keep_lines(
            u,
            vec![Line {
                who: Who::Shop,
                what,
            }],
        ),
    })
}

/// The gardens' goods sold in our shop, for a period ("baghat ki ab tak apni dukan se kitni bikri"): view A
/// with `goods` garden, with no garden named.
fn garden_goods_plan(u: &Understanding) -> Option<Planned> {
    if !u.sure
        || !u.garden_goods
        || u.intent.value != Intent::Figure
        || u.figures.value.as_slice() != [FigureKind::Sale]
        || !u.parties.is_empty()
        || !u.items.is_empty()
        || !u.records.is_empty()
        || u.item_group.is_some()
        || u.periods.len() != 1
        || u.periods.iter().any(|p| p.future)
    {
        return None;
    }
    let period = u.periods.first()?;
    let (from, to) = dates(Some(period));
    Some(Planned {
        reads: vec![
            period_read(period)?,
            analytics(
                "A",
                An {
                    from,
                    to,
                    goods: Some("garden"),
                    ..An::default()
                },
            ),
        ],
        lines: Vec::new(),
    })
}

/// The balance list a question names by its title ("collector wise", R22) with no party, item or period:
/// the list as of today holds the total and a row for each. Its figure words ("total agrahi") only say
/// what it is about.
///
/// Example: "total agrahi collector wise batao" is R22.
fn titled_balance_list(u: &Understanding) -> Option<&'static str> {
    if !u.sure
        || !matches!(u.intent.value, Intent::Figure | Intent::List)
        || !u.parties.is_empty()
        || !u.items.is_empty()
        || !u.records.is_empty()
        || u.item_group.is_some()
        || !u.periods.is_empty()
        || u.happened
        || u.compare
    {
        return None;
    }
    u.figures.value.iter().find_map(|k| match k {
        FigureKind::Report(id) if matches!(*id, "R16" | "R17" | "R22" | "R26" | "R32") => Some(*id),
        _ => None,
    })
}

/// True when the only doubt of a question is that it names several periods or compares them, and the
/// app can read each period (`comparison_plan`): the question is then sure of what it asks.
pub(super) fn periods_are_planned(u: &Understanding) -> bool {
    u.doubts
        .iter()
        .all(|d| matches!(d, Doubt::Comparison | Doubt::SeveralPeriods))
        && !u.doubts.is_empty()
        && comparison_plan(u).is_some()
}

/// The reads of one figure over two to four periods ("is mahine ka commission pichle mahine se kitna
/// zyada hai", "august ke muqable september ka commission", "how did onion sales go in August,
/// September and October"): every period is read before the model starts, and the model writes the
/// difference or the month by month figures. The figure is a flow (a sale, a commission, a laga), for
/// the whole shop, one or two items, a group or one party; the periods are closed and separate. For
/// two periods, when the earlier is exactly the one the analytics `compare` works out for the later,
/// and the figure is the commission (the one figure `compare` compares), it is one view A read with
/// `compare`; else it is the reads of each period. No instant answer.
///
/// Example: October and September, both whole months, is view A for October with compare `previous`.
fn comparison_plan(u: &Understanding) -> Option<Planned> {
    const LEFT: [Doubt; 2] = [Doubt::Comparison, Doubt::SeveralPeriods];
    const MOST_PERIODS: usize = 4;
    let kind = match u.figures.value.as_slice() {
        [k @ (FigureKind::Sale | FigureKind::Commission | FigureKind::Laga)] => *k,
        _ => return None,
    };
    if u.intent.value != Intent::Figure
        || !(2..=MOST_PERIODS).contains(&u.periods.len())
        || u.two_figures
        || u.scoped
        || u.happened
        || u.quality
        || u.amount_named
        || u.sum_asked
        || u.sum_strong
        || u.guests
        || u.truck.is_some()
        || u.bikri_number.is_some()
        || u.other_measure != Measure::Same
        || u.doubts.iter().any(|d| !LEFT.contains(d))
        || u.periods.iter().any(|p| p.future || p.from.is_none())
    {
        return None;
    }
    // The latest first. Separate periods: each one ends before the next later one starts.
    let mut ordered: Vec<&PeriodRef> = u.periods.iter().collect();
    ordered.sort_by_key(|p| std::cmp::Reverse(p.to));
    for pair in ordered.windows(2) {
        if let [later, earlier] = pair
            && later.from.is_none_or(|from| earlier.to >= from)
        {
            return None;
        }
    }
    // Each period is planned as the single-period question it is.
    let sub = |p: &PeriodRef| {
        let mut one = u.clone();
        one.periods = vec![p.clone()];
        one.compare = false;
        one.sure = true;
        one.doubts.clear();
        plan(&one)
    };
    let plans: Vec<Planned> = ordered.iter().map(|p| sub(p)).collect::<Option<_>>()?;
    let others = |p: &Planned| {
        p.reads
            .iter()
            .filter(|r| r.tool != "period" && r.tool != "find")
            .cloned()
            .collect::<Vec<_>>()
    };
    let mut reads: Vec<PlannedRead> = Vec::new();
    for tool in ["period", "find"] {
        for read in plans
            .iter()
            .flat_map(|p| &p.reads)
            .filter(|r| r.tool == tool)
        {
            if !reads.contains(read) {
                reads.push(read.clone());
            }
        }
    }
    // Two periods, the earlier the one `compare` works out, the commission: one read with `compare`.
    let one_read = match (ordered.as_slice(), plans.as_slice()) {
        ([later, earlier], [now, _]) if kind == FigureKind::Commission => {
            let (from, to) = (later.from.map(iso), iso(later.to));
            let earlier_dates = (earlier.from.map(iso).unwrap_or_default(), iso(earlier.to));
            let word = from.as_deref().and_then(|from| {
                [
                    (AnalyticsCompare::Previous, "previous"),
                    (AnalyticsCompare::LastYear, "last_year"),
                ]
                .into_iter()
                .find(|(how, _)| compare_window(*how, from, &to) == Some(earlier_dates.clone()))
                .map(|(_, word)| word)
            });
            match (word, others(now).as_slice()) {
                (Some(word), [read])
                    if read.tool == "analytics"
                        && read.input.get("view").is_some_and(|v| v == "A") =>
                {
                    let mut read = read.clone();
                    if let Some(input) = read.input.as_object_mut() {
                        input.insert("compare".to_owned(), json!(word));
                    }
                    Some(read)
                }
                _ => None,
            }
        }
        _ => None,
    };
    match one_read {
        Some(read) => reads.push(read),
        None => {
            for read in plans.iter().flat_map(others) {
                if !reads.contains(&read) {
                    reads.push(read);
                }
            }
        }
    }
    Some(Planned {
        reads,
        lines: Vec::new(),
    })
}

/// True when the question asks whom to collect from first: an "oldest" or "first" word and a "who" word
/// ("kis se pehle wasooli karon", "who should I collect from first").
pub(super) fn oldest(scan: &Scan) -> bool {
    let has = |list: &[&str]| scan.words.iter().any(|w| list.iter().any(|x| key(x) == *w));
    has(&[
        "pehle",
        "pahle",
        "pehly",
        "pahly",
        "purana",
        "purane",
        "purani",
        "oldest",
        "first",
        "پہلے",
        "پرانا",
        "پرانے",
        "پرانی",
    ]) && has(&[
        "kis", "kisse", "kiski", "kisko", "kon", "kaun", "who", "which", "whom", "کس", "کون",
    ])
}

/// The buyers who owe, oldest first (R20 sorted by days, 5 rows), for "kis se pehle wasooli karon". The
/// first row is the one to collect from first, with its days. No instant answer: the shop's total
/// dues is no answer to "whom first" (live run 2, H4).
///
/// Example: Arshad 18,684 (36 days) comes before Akram 2,880.
fn oldest_dues_plan(u: &Understanding) -> Option<Planned> {
    if !matches!(
        u.intent.value,
        Intent::Figure | Intent::List | Intent::Advice
    ) || !u.parties.is_empty()
        || !u.items.is_empty()
        || !u.records.is_empty()
        || !u.periods.is_empty()
        || u.item_group.is_some()
        || !matches!(
            u.figures.value.as_slice(),
            [FigureKind::Dues | FigureKind::Receipts | FigureKind::Balance]
        )
    {
        return None;
    }
    Some(Planned {
        reads: vec![report(
            "R20",
            Rep {
                sort: Some(sorted("days", "desc")),
                rows: Some(5),
                ..Rep::default()
            },
        )],
        lines: Vec::new(),
    })
}

/// The month by month sales of one item, for "in how many months did it sell" ("mooli kitne mahine
/// biki hai"): view D (item by month) for the item from the first day of the books to today, with the
/// item found first. The model counts the months with a figure. No period named (the question asks all
/// the books) and no instant answer.
///
/// Example: Radish is view D with the item, `group_by` month, from the first day to today.
fn months_plan(u: &Understanding) -> Option<Planned> {
    if u.intent.value != Intent::Figure
        || !u.periods.is_empty()
        || !u.parties.is_empty()
        || !u.records.is_empty()
        || u.figures.value.as_slice() != [FigureKind::Sale]
    {
        return None;
    }
    // All the books: the first day to today. Without them the tool takes its own default period and
    // the count of months is wrong (live run, "mooli kitne mahine biki hai": 1 for 6).
    let (from, to) = u.months_range.clone()?;
    let [item] = u.items.as_slice() else {
        return None;
    };
    let Outcome::Found(goods) = &item.outcome else {
        return None;
    };
    let mut plan = Plan::default();
    plan.find(&goods.name_en, "item");
    plan.read(PlannedRead {
        tool: "analytics",
        input: json!({
            "view": "D", "from": from, "to": to, "group_by": "month", "item": goods.key,
            "item_group": null, "seller": null, "buyer": null, "city": null, "adda": null,
            "garden": null, "trip": null, "goods": null, "partner": null, "side": null,
            "compare": null, "sort": null, "rows": null
        }),
    });
    let mut reads = plan.finds;
    reads.extend(plan.reads);
    Some(Planned {
        reads,
        lines: Vec::new(),
    })
}

/// Words that say a part of the cash book: "kitna cash aya" (in), "cash gaya" (out), "kitna bacha" (left).
/// "in" and "out" count only as the words after the cash word, and not in "cash in hand", "cash in the
/// drawer".
const CASH_IN_WORDS: [&str; 10] = [
    "aya", "aaya", "ayi", "aayi", "aamad", "came", "received", "آیا", "آئی", "آمد",
];
const CASH_OUT_WORDS: [&str; 11] = [
    "gaya",
    "gya",
    "gayi",
    "nikla",
    "nikli",
    "went",
    "گیا",
    "گئی",
    "نکلا",
    "nikala",
    "نکالا",
];
const CASH_LEFT_WORDS: [&str; 6] = ["left", "bacha", "bachi", "closing", "بچا", "بچی"];
const CASH_OPEN_WORDS: [&str; 5] = ["shuru", "opening", "started", "شروع", "starting"];

/// True for a word of the direction sets above, or "in" and "out".
pub(super) fn is_cash_direction_word(word: &str) -> bool {
    let is = |list: &[&str]| list.iter().any(|x| key(x) == word);
    is(&CASH_IN_WORDS)
        || is(&CASH_OUT_WORDS)
        || is(&CASH_LEFT_WORDS)
        || is(&CASH_OPEN_WORDS)
        || matches!(word, "in" | "out")
}

/// True when the question names a buyer, a seller, a group of them, a partner, the staff, the guests, a
/// garden, a trip or an item: the figure is then theirs, not the whole book's or the whole shop's.
fn has_entity_cue(scan: &Scan) -> bool {
    const ENTITY: [Cue; 10] = [
        Cue::RoleBuyer,
        Cue::RoleSeller,
        Cue::RolePartner,
        Cue::RoleGarden,
        Cue::RoleTrip,
        Cue::RoleStaff,
        Cue::Guests,
        Cue::ManyBuyers,
        Cue::ManySellers,
        Cue::RoleItem,
    ];
    scan.hits.iter().any(|h| ENTITY.contains(&h.cue))
}

/// The parts of the cash book a question names, in the order it names them.
///
/// Each direction word must stand right after the cash word or a "how much" word ("kitna cash aya",
/// "cash gaya", "kitna gaya"): "aa gaya" and "ho gaya" are not "went out". A question that names a party, a
/// group of parties, a partner, the staff, the guests, a garden or a trip, or has an action verb besides the
/// direction word ("diya", "le", "liya"), is about the cash of THAT, not the whole book: no direction (D7,
/// plan 59 F3b, money review of the cash book).
///
/// Example: "yesterday how much cash came in, how much went out and how much was left" is In, Out, Closing;
/// "kal byopariyon ko kitna cash gaya" is none.
pub(super) fn cash_directions(scan: &Scan) -> Vec<CashDir> {
    // A cash word must stand in the question.
    if !scan.has(Cue::Cash) {
        return Vec::new();
    }
    const ENTITY: [Cue; 10] = [
        Cue::RoleBuyer,
        Cue::RoleSeller,
        Cue::RolePartner,
        Cue::RoleGarden,
        Cue::RoleTrip,
        Cue::RoleStaff,
        Cue::Guests,
        Cue::ManyBuyers,
        Cue::ManySellers,
        Cue::RoleItem,
    ];
    if scan.hits.iter().any(|h| ENTITY.contains(&h.cue)) {
        return Vec::new();
    }
    let direction_word = |w: &str| is_cash_direction_word(w);
    if scan.words.iter().any(|w| {
        !direction_word(w)
            && (super::function_words::is_event_word(w) || super::function_words::is_action_verb(w))
    }) {
        return Vec::new();
    }
    // "in", "out" and the drawer: only straight after the cash word, and followed by nothing or a time word
    // ("cash in hand", "cash in the drawer", "cash in gala" are the cash in hand).
    let after_cash = |at: usize| scan.hits.iter().any(|h| h.cue == Cue::Cash && h.end == at);
    // The words just before a direction verb that make it a verb of the cash: the cash word itself, or a
    // "how much" word.
    let how_much = |w: &str| {
        matches!(
            w,
            "kitna" | "kitni" | "kitne" | "کتنا" | "کتنی" | "کتنے" | "much"
        )
    };
    let mut out = Vec::new();
    for (at, w) in scan.words.iter().enumerate() {
        let next = scan.word(at.saturating_add(1));
        let before = at.checked_sub(1).and_then(|b| scan.word(b));
        let is = |list: &[&str]| list.iter().any(|x| key(x) == w.as_str());
        let verb_ok = after_cash(at) || before.is_some_and(how_much);
        let tail_ok = next.is_none_or(|x| super::when::is_time_word(x) || how_much(x));
        let dir = if is(&CASH_IN_WORDS) && (after_cash(at) || verb_ok) {
            Some(CashDir::In)
        } else if is(&CASH_OUT_WORDS) && verb_ok {
            Some(CashDir::Out)
        } else if is(&CASH_LEFT_WORDS) {
            Some(CashDir::Closing)
        } else if is(&CASH_OPEN_WORDS) {
            Some(CashDir::Opening)
        } else if w == "in" && after_cash(at) && tail_ok {
            Some(CashDir::In)
        } else if w == "out" && after_cash(at) && tail_ok {
            Some(CashDir::Out)
        } else {
            None
        };
        if let Some(d) = dir
            && !out.contains(&d)
        {
            out.push(d);
        }
    }
    out
}

/// The most rows of the cash book the app reads to see whether the period has a reversal (the most the tool
/// gives, PRD FR-19.11).
const CASH_ROWS_READ: u32 = 100;

/// The cash book of a period for a question that names its parts ("cash came in yesterday", "cash went out
/// and what was left"): the `cash_in` and `cash_out` totals and the closing balance of R12, each its own
/// line, in the order asked. The opening is not answered here.
fn cash_flow_plan(u: &Understanding) -> Option<Planned> {
    if u.cash_dirs.is_empty()
        || u.cash_dirs.contains(&CashDir::Opening)
        || !u.sure
        || u.intent.value != Intent::Figure
        || u.figures.value.as_slice() != [FigureKind::Cash]
        || u.periods.len() != 1
        || !u.parties.is_empty()
        || !u.items.is_empty()
        || !u.records.is_empty()
        || u.item_group.is_some()
        || u.compare
        || u.scoped
        || u.sum_asked
        || u.other_measure != Measure::Same
        || u.doubts.iter().any(|d| NOT_CARRIED.contains(d))
    {
        return None;
    }
    let period = u.periods.first()?;
    if period.future {
        return None;
    }
    let (from, to) = dates(Some(period));
    let mut plan = Plan::default();
    // The cash that came in or went out is answered only when the period has no reversal entry (the totals
    // of the cash book hold a cancelled or edited entry twice: its posting and its reversal), which the rows
    // tell: so they are read, up to the most the tool gives. A period with more entries than that, or with
    // a reversal, is the model's (lead ruling of 08-11-2026).
    let flows = u
        .cash_dirs
        .iter()
        .any(|d| matches!(d, CashDir::In | CashDir::Out));
    plan.read(report(
        "R12",
        Rep {
            from,
            to,
            rows: flows.then_some(CASH_ROWS_READ),
            ..Rep::default()
        },
    ));
    plan.dated = true;
    for dir in &u.cash_dirs {
        plan.line(
            Who::Shop,
            match dir {
                CashDir::In => What::CashIn,
                CashDir::Out => What::CashOut,
                _ => What::CashClosing,
            },
        );
    }
    let mut reads = Vec::new();
    reads.push(period_read(period)?);
    reads.extend(plan.finds);
    reads.extend(plan.reads);
    Some(Planned {
        reads,
        lines: plan.lines,
    })
}

/// The sides of the trial balance a question names, in the order it names them: بنام / debit, جمع / credit. Only
/// beside the trial balance itself ("jama" alone is a sum: "akram aur arshad ka jama").
///
/// Example: "ٹرائل بیلنس کا بنام کل کتنا ہے" is Debit; "trial balance mein banaam aur jama" is Debit, Credit.
/// The side words of the trial balance: بنام / debit, جمع / credit, and their spellings.
const TRIAL_DEBIT_WORDS: [&str; 7] = ["banaam", "benaam", "bnaam", "banam", "بنام", "debit", "dr"];
const TRIAL_CREDIT_WORDS: [&str; 6] = ["jama", "jamaa", "جمع", "credit", "cr", "jam"];

/// True for a side word of the trial balance.
pub(super) fn is_trial_side_word(word: &str) -> bool {
    TRIAL_DEBIT_WORDS
        .iter()
        .chain(TRIAL_CREDIT_WORDS.iter())
        .any(|x| key(x) == word)
}

pub(super) fn trial_sides(scan: &Scan) -> Vec<TrialSide> {
    if !scan.has(Cue::TrialBalance) {
        return Vec::new();
    }
    // The trial balance asked is the one of now. "At the start" and "opening" are R19; a past verb ("kitna
    // tha") names a time the app does not read: neither is answered here, the model is.
    const OPENING: [&str; 10] = [
        "shuru",
        "shuruaat",
        "opening",
        "start",
        "started",
        "initial",
        "ابتدائی",
        "شروع",
        "pehle",
        "beginning",
    ];
    // A buyers, sellers or partner word, a garden or a trip: the debit of THEM is another figure than the whole
    // trial balance's. And a "kal" before the trial balance names yesterday's (or is "kul" mistyped): the
    // model's. ("بنام کل" after the side word is "total": it stands after the cue and is fine.)
    if has_entity_cue(scan) {
        return Vec::new();
    }
    let trial_at = scan
        .hits
        .iter()
        .find(|h| h.cue == Cue::TrialBalance)
        .map_or(0, |h| h.start);
    if scan
        .words
        .iter()
        .take(trial_at)
        .any(|w| matches!(w.as_str(), "kal" | "کل"))
    {
        return Vec::new();
    }
    // (Not the shared tense check: it reads the English article "the" as the Roman Urdu "the", were.)
    const PAST: [&str; 12] = [
        "tha", "thi", "thay", "was", "were", "did", "had", "kiya", "kia", "تھا", "تھی", "تھے",
    ];
    if scan.words.iter().any(|w| {
        OPENING
            .iter()
            .chain(PAST.iter())
            .any(|x| key(x) == w.as_str())
    }) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for w in &scan.words {
        let is = |list: &[&str]| list.iter().any(|x| key(x) == w.as_str());
        let side = if is(&TRIAL_DEBIT_WORDS) {
            Some(TrialSide::Debit)
        } else if is(&TRIAL_CREDIT_WORDS) {
            Some(TrialSide::Credit)
        } else {
            None
        };
        if let Some(side) = side
            && !out.contains(&side)
        {
            out.push(side);
        }
    }
    out
}

/// The direction of a question about capital: taken back ("wapas", "واپس", "withdrew", "taken back") or
/// put in ("lagaya", "dala", "invested", "put in"); `None` when it says neither or both.
pub(super) fn capital_flow(scan: &Scan) -> Option<CapitalFlow> {
    const OUT: [&str; 12] = [
        "wapas",
        "wapis",
        "wapsi",
        "واپس",
        "withdraw",
        "withdrew",
        "withdrawn",
        "nikala",
        "nikalwaya",
        "نکالا",
        "back",
        "nikal",
    ];
    const IN: [&str; 13] = [
        "dala",
        "daala",
        "dali",
        "lagaya",
        "lagayi",
        "invested",
        "invest",
        "deposit",
        "deposited",
        "ڈالا",
        "لگایا",
        "لگائی",
        "put",
    ];
    let has = |list: &[&str]| scan.words.iter().any(|w| list.iter().any(|x| key(x) == *w));
    match (has(&OUT), has(&IN)) {
        (true, false) => Some(CapitalFlow::Out),
        (false, true) => Some(CapitalFlow::In),
        _ => None,
    }
}

/// True when the question names the gardens' goods and no garden or trip was named: a garden word
/// ("baghat", "gardens", "باغات") and none of the words of another scope (a trip, the guests, own goods).
pub(super) fn garden_goods(scan: &Scan, named_a_garden_or_trip: &bool) -> bool {
    // The plural only: "bagh ka commission" may be one garden's, and then the guards and the model decide.
    const GARDEN: [&str; 5] = ["baghat", "baghaat", "baghon", "gardens", "باغات"];
    const OTHER: [&str; 9] = [
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
    let has = |list: &[&str]| scan.words.iter().any(|w| list.iter().any(|x| key(x) == *w));
    !*named_a_garden_or_trip && has(&GARDEN) && !has(&OTHER) && !own_goods(scan)
}

/// True when the question asks for the last one: "aakhri" (always), or "last", "latest", "pichla" before a
/// word for a rate or a sale ("last rate", "pichli bikri"). "Last week" and "pichle mahine" are periods,
/// never the last one.
pub(super) fn last_one(scan: &Scan) -> bool {
    const ALWAYS: [&str; 4] = ["aakhri", "akhri", "aakhir", "آخری"];
    const HEAD: [&str; 6] = ["last", "latest", "pichla", "pichli", "پچھلا", "پچھلی"];
    const RATEISH: [&str; 16] = [
        "rate", "rates", "price", "bhao", "bhaw", "sale", "bikri", "sold", "time", "bar", "baar",
        "dafa", "entry", "ریٹ", "بھاؤ", "قیمت",
    ];
    let is = |w: &str, list: &[&str]| list.iter().any(|x| key(x) == w);
    // "last buyer rate": the buyer word between the head and the rate.
    let buyer_word = |w: &str| {
        scan.hits
            .iter()
            .any(|h| h.cue == Cue::RoleBuyer && scan.words.get(h.start).is_some_and(|x| x == w))
    };
    scan.words.iter().enumerate().any(|(at, w)| {
        is(w, &ALWAYS)
            || (is(w, &HEAD)
                && scan.word(at.saturating_add(1)).is_some_and(|next| {
                    is(next, &RATEISH)
                        || (buyer_word(next)
                            && scan
                                .word(at.saturating_add(2))
                                .is_some_and(|after| is(after, &RATEISH)))
                }))
    })
}

/// The last rate of an item ("akram ko kela aakhri kis rate pe diya", "last rate of banana to akram",
/// "what was the last rate of banana today", "آلو کا اب تک کا آخری ریٹ"): the register of the sales (R2)
/// for the item, the period if one is named and the buyer if one is, newest first by date and then by
/// entry, ONE row. That row is the last sale and its `rate` the answer: never the average of the period
/// (live run 2, C8). No instant answer.
///
/// Example: Banana today is R2 for the item and today, sorted by date and entry, 1 row.
fn last_rate_plan(u: &Understanding) -> Option<Planned> {
    if u.intent.value != Intent::Figure
        || !u.records.is_empty()
        || u.item_group.is_some()
        || u.periods.len() > 1
        || u.periods.iter().any(|p| p.future)
    {
        return None;
    }
    let [item] = u.items.as_slice() else {
        return None;
    };
    let Outcome::Found(goods) = &item.outcome else {
        return None;
    };
    let buyer = match u.parties.as_slice() {
        [] => None,
        [party] => match &party.outcome {
            Outcome::Found(hit) if hit.role == PartyRole::Buyer => {
                Some((party.text.clone(), hit.key.clone()))
            }
            _ => return None,
        },
        _ => return None,
    };
    // A quality named ("awwal seb ka aakhri rate"): the lines of that quality only; every quality asked
    // ("kis darje mein"), or a filter the tools do not take yet, is the model's (D7, plan 59 R4).
    let grade = match (u.quality, u.grade) {
        (false, _) => None,
        (true, Some(g)) if grade_ready() => Some(g.tool_word()),
        (true, _) => return None,
    };
    let period = u.periods.first();
    let (from, to) = dates(period);
    let mut plan = Plan::default();
    if let Some((text, _)) = &buyer {
        plan.find(text, "buyer");
    }
    plan.find(&goods.name_en, "item");
    let buyer_key = buyer.map(|(_, key)| key);
    plan.read(report(
        "R2",
        Rep {
            from,
            to,
            party: buyer_key.clone(),
            item: Some(goods.key.clone()),
            // Newest first: the register breaks the tie of a day by بکری and line, newest first.
            sort: Some(json!([{"column": "date", "order": "desc"}])),
            rows: Some(1),
            grade,
            ..Rep::default()
        },
    ));
    // No sale in the period ("aaj angoor ka rate" on a day with none): the newest sale BEFORE the period is
    // read too, so the answer can say there was no sale and give the last rate with its date (Umar,
    // 08-11-2026). It is read for every period that has a first day; the sentence uses it only when the
    // period's own read has no row.
    if let Some(first) = period.and_then(|p| p.from)
        && let Some(before) = first.pred_opt()
    {
        plan.read(report(
            "R2",
            Rep {
                to: Some(iso(before)),
                party: buyer_key.clone(),
                item: Some(goods.key.clone()),
                sort: Some(json!([{"column": "date", "order": "desc"}])),
                rows: Some(1),
                grade,
                ..Rep::default()
            },
        ));
    }
    // The instant line (D7, plan 59 S-A2): the rate and the date of the one row. With a quality named the
    // sentence would not say which, and the rate of an item sold by weight is per kg, which the sentence
    // does not say either (lead ruling): those questions keep the model.
    if grade.is_none() && !goods.sold_by_weight {
        let who = match buyer_key {
            Some(key) => Who::BuyerItem(key, goods.key.clone()),
            None => Who::Item(goods.key.clone()),
        };
        plan.line(who, What::LastRate);
    }
    let mut reads = Vec::new();
    if let Some(p) = period {
        reads.push(period_read(p)?);
    }
    reads.extend(plan.finds);
    reads.extend(plan.reads);
    Some(Planned {
        reads,
        lines: plan.lines,
    })
}

/// The reads of a question about the quality of sale lines ("awwal seb aaj kitne ka bika", "doyam
/// kele ka aaj ka rate", "ehsan ke seb kis kis darje mein bike"): the register of the sales (R2) for
/// the period, with the item, the group, the buyer or the seller named. Every line carries its quality,
/// so the model picks the lines of the quality and sums them with `calculate`. No instant answer: the
/// figure of an item or a party is the sum of all its qualities, a wrong number for one of them. No
/// view or hint per quality (plan 58). The register is read to its last page, 100 rows at most.
///
/// Example: "awwal seb aaj kitne ka bika" is R2 for today with the item Apple.
fn quality_plan(u: &Understanding) -> Option<Planned> {
    if !matches!(u.intent.value, Intent::Figure | Intent::List) || !u.records.is_empty() {
        return None;
    }
    // Only a figure of the SALES carries a quality (what a buyer bought is the sales to him):
    // "doyam kele kitne bache hain" asks the stock, which has none, and the sales read for it would be
    // the wrong figure (D7 R11).
    if !u.figures.value.iter().all(|f| {
        matches!(
            f,
            FigureKind::Sale
                | FigureKind::SaleCount
                | FigureKind::Commission
                | FigureKind::Laga
                | FigureKind::Rate
                | FigureKind::Purchases
        )
    }) {
        return None;
    }
    let period = u.periods.first();
    if period.is_some_and(|p| p.future) || u.periods.len() > 1 {
        return None;
    }
    let item = match u.items.as_slice() {
        [] => None,
        [one] => match &one.outcome {
            Outcome::Found(hit) => Some((hit.name_en.clone(), hit.key.clone())),
            _ => return None,
        },
        _ => return None,
    };
    let party = match u.parties.as_slice() {
        [] => None,
        [one] => match &one.outcome {
            Outcome::Found(hit) if matches!(hit.role, PartyRole::Buyer | PartyRole::Seller) => {
                Some((one.text.clone(), hit.role, hit.key.clone()))
            }
            _ => return None,
        },
        _ => return None,
    };
    let group = u.item_group.map(GroupWord::tool_word);
    if group.is_some() && !group_filter_ready() {
        return None;
    }
    let mut plan = Plan::default();
    let (from, to) = dates(period);
    let mut rep = Rep {
        from,
        to,
        rows: Some(100),
        item_group: group,
        ..Rep::default()
    };
    if let Some((text, key)) = item {
        plan.find(&text, "item");
        rep.item = Some(key);
    }
    if let Some((text, role, key)) = party {
        plan.find(&text, role_word(role)?);
        if role == PartyRole::Seller {
            rep.seller = Some(key);
        } else {
            rep.party = Some(key);
        }
    }
    match u.grade {
        // One quality named, and the tools take the filter: the figures of that quality (view A with the
        // item or group, the period and the grade); never the figure of all the qualities.
        Some(grade) if grade_ready() => {
            let mut a = An {
                from: rep.from.clone(),
                to: rep.to.clone(),
                item: rep.item.clone(),
                item_group: rep.item_group,
                grade: Some(grade.tool_word()),
                ..An::default()
            };
            if rep.seller.is_some() {
                a.seller = rep.seller.clone();
            }
            if rep.party.is_some() {
                a.buyer = rep.party.clone();
            }
            plan.read(analytics("A", a));
        }
        // No quality named ("kis darje mein bike"), or no filter yet: the lines, each with its quality.
        _ => plan.read(report("R2", rep)),
    }
    let mut reads = Vec::new();
    if let Some(p) = period {
        reads.push(period_read(p)?);
    }
    reads.extend(plan.finds);
    reads.extend(plan.reads);
    Some(Planned {
        reads,
        lines: Vec::new(),
    })
}

/// The reads of a figure of a whole group of items (all the vegetables, all the fruit): a sale, a
/// commission or a laga in a named period (view A with the group), or the goods left of each of them
/// (R10 with the group, every lot with its nag). No instant answer (no line): the model writes it from
/// the rows. `None` while the tools have no group filter, or for anything else about a group.
fn group_plan(u: &Understanding, group: GroupWord) -> Option<Planned> {
    if !group_filter_ready() || !u.parties.is_empty() || !u.records.is_empty() {
        return None;
    }
    // The views have no truck filter, and a quantity that was not asked as a quantity ("kitna maal bika")
    // is no sale figure (money review of F3b).
    if u.truck.is_some()
        || (matches!(u.other_measure, Measure::Quantity | Measure::Breakdown) && !u.qty_asked)
    {
        return None;
    }
    let word = Some(group.tool_word());
    match (u.figures.value.as_slice(), u.periods.as_slice()) {
        ([FigureKind::Stock], []) => Some(Planned {
            reads: vec![report(
                "R10",
                Rep {
                    layout: Some("item"),
                    item_group: word,
                    ..Rep::default()
                },
            )],
            lines: Vec::new(),
        }),
        ([FigureKind::Sale | FigureKind::Commission | FigureKind::Laga], [period]) => {
            if period.future {
                return None;
            }
            let (from, to) = dates(Some(period));
            // The figure of the whole group over the period: the sale, the commission with Laga, the Laga, or
            // the quantity asked (D7, plan 59 S-A3). Each is a cell of the totals of view A for the group.
            let what = match (u.figures.value.as_slice(), u.qty_asked) {
                ([FigureKind::Sale], true) => Some(What::Qty),
                (kinds, false) => kinds.first().and_then(|k| what_of(*k)),
                _ => None,
            };
            let lines = what
                .map(|what| {
                    totals::keep_lines(
                        u,
                        vec![Line {
                            who: Who::Group(group.tool_word().to_owned()),
                            what,
                        }],
                    )
                })
                .unwrap_or_default();
            Some(Planned {
                reads: vec![
                    period_read(period)?,
                    analytics(
                        "A",
                        An {
                            from,
                            to,
                            item_group: word,
                            ..An::default()
                        },
                    ),
                ],
                lines,
            })
        }
        _ => None,
    }
}

/// The reads of a forecast, and of the real figure beside it ("pichle mahine bijli ka kharcha, aur
/// agle 3 mahine ka estimate"): the real read for the past period, if the question names one, and the
/// `estimate` read for the months ahead. The model then writes both from reads that are already in,
/// instead of finding its way to them with 3 or 4 calls. There is no instant answer for it.
///
/// Only one figure, one future period and at most one other period; a target, a what-if (another
/// intent), a comparison or a name that was not found is the model's.
///
/// "agle mahine" (one month, no number) is planned with `ahead` 1: the tool's own default for a question
/// with no number is 6 months, and a 6-month total is no figure for one month.
///
/// Example: "agle 3 mahine ka commission" is `estimate` with `commission_all` and `ahead` 3.
fn forecast_plan(u: &Understanding) -> Option<Planned> {
    if u.amount_named || !names_found(u) {
        return None;
    }
    let future: Vec<&PeriodRef> = u.periods.iter().filter(|p| p.future).collect();
    let [ahead_period] = future.as_slice() else {
        return None;
    };
    let past: Vec<&PeriodRef> = u.periods.iter().filter(|p| !p.future).collect();
    if past.len() > 1 {
        return None;
    }
    let [kind] = u.figures.value.as_slice() else {
        return None;
    };
    let (ahead, until) = match ahead_period.kind {
        PeriodKind::NextMonths(n) if (1..=12).contains(&n) => (Some(i64::from(n)), None),
        // "agle mahine" with no number is one month: sent as a number, like "agle 3 mahine", so the tool
        // does not take its own 6 months and the model never cites a 6-month total for one month.
        PeriodKind::NextMonth => (Some(1), None),
        PeriodKind::YearEnd => (None, Some("year_end")),
        _ => return None,
    };
    let measure = match kind {
        FigureKind::Sale => "sales",
        FigureKind::Commission => "commission_all",
        FigureKind::Laga => "laga",
        FigureKind::Expense => "shop_expenses",
        FigureKind::Profit => "profit",
        FigureKind::Receipts => "receipts",
        // A balance now and the months ahead: cash in hand, the buyers' dues, or one party's balance.
        FigureKind::Cash => "cash_in_hand",
        FigureKind::Dues => "buyers_dues",
        FigureKind::Balance | FigureKind::Ledger | FigureKind::Statement => "party_balance",
        _ => return None,
    };
    let balance_now = matches!(measure, "cash_in_hand" | "buyers_dues" | "party_balance");
    let mut plan = Plan::default();
    // The real part: the same read as the question would have without the forecast.
    if let Some(real) = past.first() {
        figure_of_kind(u, *kind, Some(real), &mut plan)?;
    } else if balance_now {
        // What it is now: the card of the party, or the tile of the shop.
        let now = match measure {
            "party_balance" => FigureKind::Balance,
            other => {
                if other == "cash_in_hand" {
                    FigureKind::Cash
                } else {
                    FigureKind::Dues
                }
            }
        };
        figure_of_kind(u, now, None, &mut plan)?;
    }
    // Who the estimate is for: one item, one seller or buyer, one expense head, one partner.
    let (mut item, mut seller, mut buyer, mut head, mut partner) = (None, None, None, None, None);
    let mut party = None;
    for p in &u.parties {
        let Outcome::Found(hit) = &p.outcome else {
            continue;
        };
        let key = Some(hit.key.clone());
        match (hit.role, measure) {
            (PartyRole::Seller, "sales") => {
                plan.find(&p.text, "seller");
                seller = key;
            }
            (PartyRole::Buyer, "sales") => {
                plan.find(&p.text, "buyer");
                buyer = key;
            }
            (PartyRole::Partner, "profit") => {
                plan.find(&p.text, "partner");
                partner = key;
            }
            // One party's balance: his key goes in `party`.
            (PartyRole::Seller | PartyRole::Buyer, "party_balance") => {
                plan.find(&p.text, role_word(hit.role)?);
                party = key;
            }
            _ => return None,
        }
    }
    for i in &u.items {
        let Outcome::Found(hit) = &i.outcome else {
            continue;
        };
        if !matches!(measure, "sales" | "commission_all" | "laga") {
            return None;
        }
        plan.find(&hit.name_en, "item");
        item = Some(hit.key.clone());
    }
    for r in &u.records {
        let Outcome::Found(hit) = &r.outcome else {
            continue;
        };
        if hit.kind != RecordKind::ExpenseHead || measure != "shop_expenses" {
            return None;
        }
        plan.find(&hit.name_en, "expense_head");
        head = Some(hit.key.clone());
    }
    plan.read(PlannedRead {
        tool: "estimate",
        input: json!({
            "measure": measure, "item": item, "seller": seller, "buyer": buyer,
            "garden": null, "trip": null, "head": head, "partner": partner, "party": party,
            "ahead": ahead, "until": until, "what_if": null, "percent": null, "target": null
        }),
    });
    let mut reads = Vec::new();
    if plan.dated
        && let Some(p) = past.first()
    {
        reads.push(period_read(p)?);
    }
    reads.extend(plan.finds);
    reads.extend(plan.reads);
    Some(Planned {
        reads,
        lines: Vec::new(),
    })
}

/// True when the one figure asked is what is true now, not a total of a period.
fn asks_now(u: &Understanding) -> bool {
    matches!(
        u.figures.value.as_slice(),
        [FigureKind::Balance
            | FigureKind::Dues
            | FigureKind::Payable
            | FigureKind::Advance
            | FigureKind::Cash
            | FigureKind::Stock
            | FigureKind::PartnerTotal
            | FigureKind::DeadAmount
            | FigureKind::MoneyLent]
    )
}

/// A "now" figure or the trial balance: a bare "kal" beside one is "total" (see `plan`).
fn asks_now_or_trial(u: &Understanding) -> bool {
    asks_now(u) || u.figures.value.as_slice() == [FigureKind::TrialBalance]
}

/// Drops a "kal" that no verb settles when the question asks a figure of NOW: Urdu "کل" before a figure noun
/// ("کل ادائیگی کتنی ہے", "ادھار رقم میں کل کتنی واپس آنی ہے") is "total" (kul), and "total" is no period.
/// A past verb keeps it yesterday (it is then of High confidence and stays), a time word or "ka / ki / ke"
/// makes it a day, and with neither clear the question keeps its doubt and goes to the model (D7, plan 59
/// F3b, guard 6).
pub(super) fn drop_total_kal(
    periods: Vec<PeriodRef>,
    kinds: &[FigureKind],
    no_tense_word: bool,
) -> Vec<PeriodRef> {
    let now = matches!(
        kinds,
        [FigureKind::Balance
            | FigureKind::Dues
            | FigureKind::Payable
            | FigureKind::Advance
            | FigureKind::Cash
            | FigureKind::Stock
            | FigureKind::PartnerTotal
            | FigureKind::DeadAmount
            | FigureKind::MoneyLent
            | FigureKind::Capital
            | FigureKind::TrialBalance]
    );
    // A past or a future word settles "کل" as a day ("کل روکڑ کتنی تھی" is yesterday's): the period and its
    // doubt stay, and the model answers.
    if !now || !no_tense_word {
        return periods;
    }
    periods.into_iter().filter(|p| !is_bare_kal(p)).collect()
}

/// A period that is only the bare word "کل" or "kal" taken for yesterday with no past verb to settle it.
fn is_bare_kal(p: &PeriodRef) -> bool {
    p.kind == PeriodKind::Yesterday
        && matches!(p.words.as_str(), "کل" | "kal")
        && p.confidence != super::Confidence::High
}

/// "sab partners ki kul raqam", "each partner's total": a list question whose one figure is the
/// partners' totals, with no name.
fn all_partners_total(u: &Understanding) -> bool {
    u.figures.value == [FigureKind::PartnerTotal]
        && u.parties.is_empty()
        && u.items.is_empty()
        && u.records.is_empty()
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
    // One figure, or up to three flows of the same record or of the shop ("kal ki bikri aur
    // commission"): each is read and each is one line of the answer.
    let kinds = u.figures.value.as_slice();
    // A question about the app's own state names its read ("bikri 1 mein kisne kya badla": the
    // audit of بکری 1); the figure words beside it only say what it is about.
    if let Some(ask) = kinds.iter().find(|k| matches!(k, FigureKind::Ask(_))) {
        return figure_of_kind(u, *ask, period, plan);
    }
    // Several different measures in one question are never an instant answer until Umar says otherwise (lead
    // decision D333: PRD FR-7.90 sends two-part questions to the model). The reads of the flows of one scope are
    // still planned, as before item C, so that the model's first call has them; the new combination of other
    // figures, its safety checks and every line of an answer exist only behind the gate.
    if kinds.len() > 1 && !(flow_figures(u) || (u.multi_measure && combinable_figures(u))) {
        return None;
    }
    if kinds.len() > 1 && u.multi_measure && !several_figures_are_safe(u, period) {
        return None;
    }
    if kinds.is_empty() {
        return None;
    }
    for kind in kinds {
        let before = plan.lines.len();
        figure_of_kind(u, *kind, period, plan)?;
        // Several figures asked: every one must give a line of its own, or the answer would leave a part of
        // the question out.
        if kinds.len() > 1 && plan.lines.len() == before {
            return None;
        }
    }
    // The same line twice (a city's dues asked as dues and as balance) would show one figure as two.
    if kinds.len() > 1
        && plan
            .lines
            .iter()
            .enumerate()
            .any(|(at, l)| plan.lines.iter().skip(at.saturating_add(1)).any(|m| m == l))
    {
        return None;
    }
    Some(())
}

/// The checks that several figures asked in one question need beyond the plan of each (money review of
/// 08-11-2026, D7 plan 59 C): true when the figures may be answered together.
/// - "fruit so far and today's sales": the "so far" was folded into the day, and which period belongs to
///   which figure is not known;
/// - a period that the plan dropped (a bare "kal" or a "so far" beside now-figures) is no period for all of them;
/// - a figure of NOW (cash in hand) beside a period other than today would be read as the closing balance of
///   that period: "pichle hafte ki bikri aur cash kitna hai" asks the cash now;
/// - a word for the shop or a total ("kul", "dukan", "sab") beside a named party or item points at a figure of
///   the shop, and the named scope would answer it for the party;
/// - a party's balance beside his payable or advance, or the total dues beside the dead amount: the second
///   is already inside the first, and two lines would read as two sums of money.
fn several_figures_are_safe(u: &Understanding, period: Option<&PeriodRef>) -> bool {
    let kinds = u.figures.value.as_slice();
    let named = !(u.parties.is_empty() && u.items.is_empty() && u.records.is_empty());
    !(u.period_merged
        || (!u.periods.is_empty() && period.is_none())
        || (kinds.contains(&FigureKind::Cash)
            && period.is_some_and(|p| p.kind != PeriodKind::Today))
        || (named && u.shop_word)
        // "aaj ki bikri aur kele ka commission": a name between two figures belongs to the second only.
        || named_between_figures(u)
        // "kul bikri aur aaj ka commission": the period stands between the figures and may be the second's only.
        || (u.period_between
            && kinds.first().is_some_and(|k| {
                matches!(
                    k,
                    FigureKind::Sale
                        | FigureKind::SaleCount
                        | FigureKind::Commission
                        | FigureKind::Laga
                        | FigureKind::Expense
                        | FigureKind::Receipts
                        | FigureKind::FruitTaken
                )
            }))
        || (kinds.contains(&FigureKind::Balance)
            && kinds
                .iter()
                .any(|k| matches!(k, FigureKind::Payable | FigureKind::Advance)))
        || (u.total_dues && kinds.contains(&FigureKind::DeadAmount)))
}

/// True when a found name stands between the first and the last figure asked: it belongs to the figures after it
/// only, and the one scope the plan reads would be given to all of them.
fn named_between_figures(u: &Understanding) -> bool {
    match (u.figure_at.iter().min(), u.figure_at.iter().max()) {
        (Some(first), Some(last)) => u.name_spans.iter().any(|(s, _)| s > first && s < last),
        _ => false,
    }
}

/// True when two to four DIFFERENT figures were asked of the same scope (the shop, one party, one item, or
/// one party with one item) and every one is a figure that has a single-figure plan with a line of its own.
/// Each figure is then planned exactly as when asked alone, and the answer holds one line for each, in the
/// order asked (item C of D7, plan 59).
///
/// Example: "aaj ki bikri, kitne bil bane aur kamishan kitna hua" is Sale, SaleCount, Commission of the shop
/// for today: three lines. A commission and a laga together are left to the model (the commission holds the
/// laga).
fn combinable_figures(u: &Understanding) -> bool {
    let kinds = u.figures.value.as_slice();
    (2..=4).contains(&kinds.len())
        && kinds.iter().all(|k| {
            matches!(
                k,
                FigureKind::Sale
                    | FigureKind::SaleCount
                    | FigureKind::Commission
                    | FigureKind::Laga
                    | FigureKind::Balance
                    | FigureKind::Dues
                    | FigureKind::Payable
                    | FigureKind::Advance
                    | FigureKind::Stock
                    | FigureKind::Cash
                    | FigureKind::DeadAmount
                    | FigureKind::MoneyLent
                    | FigureKind::Expense
                    | FigureKind::Receipts
                    | FigureKind::FruitTaken
            )
        })
        && !(kinds.contains(&FigureKind::Commission) && kinds.contains(&FigureKind::Laga))
        && u.parties.len() <= 1
        && u.items.len() <= 1
        && u.records.len() <= 1
}

/// True when two or three figures were asked and every one is a flow that the same read holds: the
/// sale, the commission or the laga (the count of sales is another view). The commission of the views
/// already holds the laga, so a commission and a laga in one answer would read as two amounts to add:
/// that is the model's.
fn flow_figures(u: &Understanding) -> bool {
    let kinds = u.figures.value.as_slice();
    (2..=3).contains(&kinds.len())
        && kinds.iter().all(|k| {
            matches!(
                k,
                FigureKind::Sale | FigureKind::Commission | FigureKind::Laga
            )
        })
        && !(kinds.contains(&FigureKind::Commission) && kinds.contains(&FigureKind::Laga))
}

/// The reads and the line of one figure.
fn figure_of_kind(
    u: &Understanding,
    kind: FigureKind,
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    // The app's own state or lists: the model reads the answer from them.
    if let FigureKind::Ask(id) = kind {
        return ask_figure(u, id, period, plan);
    }
    // "karim ne kal kya aur kitna becha": the Owner wants the lines, item by item, not a total.
    let listing = u.intent.value == Intent::List;
    let parties: Vec<(&str, &PartyHit)> = u
        .parties
        .iter()
        .filter_map(|p| match &p.outcome {
            Outcome::Found(hit) => Some((p.text.as_str(), hit)),
            _ => None,
        })
        .collect();
    // An item or a record is found by its own English name: the words typed may be a plural or a
    // spelling ("kelon") that the understanding knows and the `find` tool does not.
    let items: Vec<(&str, &ItemHit)> = u
        .items
        .iter()
        .filter_map(|i| match &i.outcome {
            Outcome::Found(hit) => Some((hit.name_en.as_str(), hit)),
            _ => None,
        })
        .collect();
    let record = u.records.first().and_then(|r| match &r.outcome {
        Outcome::Found(hit) => Some((hit.name_en.as_str(), hit)),
        _ => None,
    });
    match (parties.len(), items.len(), record) {
        (0, 0, None) => shop_figure(
            kind,
            u.total_dues,
            u.other_measure == Measure::Quantity,
            u.guests,
            &u.trial_sides,
            period,
            plan,
        ),
        (0, 0, Some((text, hit))) => record_figure(kind, text, hit, period, plan),
        // One figure for each named party (or item), each with its own read: "akram aur arshad ka
        // bakaya", "kele aur saib ka commission".
        (_, 0, None) if !parties.is_empty() => {
            for (text, hit) in parties {
                if hit.role == PartyRole::Partner {
                    partner_figure(kind, text, hit, period, plan)?;
                } else {
                    party_figure(kind, text, hit, period, listing, plan)?;
                }
            }
            Some(())
        }
        (0, _, None) if !items.is_empty() => {
            for (text, hit) in items {
                // An average rate is rupees per نگ; an item sold by weight has its rate per kg (money review).
                if kind == FigureKind::Rate && plan.average && hit.sold_by_weight {
                    return None;
                }
                item_figure(kind, text, &hit.key, period, plan)?;
            }
            Some(())
        }
        (1, 1, None) => {
            let (ptext, phit) = parties.first().copied()?;
            let (itext, ihit) = items.first().copied()?;
            if kind == FigureKind::Rate && plan.average && ihit.sold_by_weight {
                return None;
            }
            party_item_figure(
                kind,
                (ptext, phit),
                (itext, &ihit.key),
                period,
                u.commission_side,
                plan,
            )
        }
        _ => None,
    }
}

/// The reads of a question about the app's own state or lists (no figure to give by itself): the
/// commission and laga rules that win for a party or an item, the attention list, the closed days,
/// how many staff or expense heads. The model writes the answer from them.
///
/// Example: "rahim par commission kitna lagta hai" reads `find` Rahim, then `app_state` `rules` with
/// him; "kitne munshi hain" reads `find` `munshi` among the staff.
fn ask_figure(
    u: &Understanding,
    id: &str,
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    let (from, to) = dates(period);
    match id {
        "attention" | "days" => plan.read(app_state(id, None, None)),
        "rules" => {
            let party = u.parties.iter().find_map(|p| match &p.outcome {
                Outcome::Found(hit) => role_word(hit.role)
                    .filter(|_| hit.role != PartyRole::Partner)
                    .map(|word| (p.text.as_str(), word, hit.key.clone())),
                _ => None,
            });
            let item = u.items.iter().find_map(|i| match &i.outcome {
                Outcome::Found(hit) => Some((hit.name_en.as_str(), hit.key.clone())),
                _ => None,
            });
            // A name that was not found is the model's, and so is a partner (no rule of his).
            if u.parties.len() > usize::from(party.is_some())
                || u.items.len() > usize::from(item.is_some())
            {
                return None;
            }
            // "sabzi ka laga kitna lagta hai": the rule of the group (`app_state` takes the word).
            let group = u.item_group.map(GroupWord::tool_word);
            if let Some((text, word, _)) = &party {
                plan.find(text, word);
            }
            if let Some((text, _)) = &item {
                plan.find(text, "item");
            }
            plan.read(app_state(
                "rules",
                party.map(|(_, _, key)| key),
                item.map(|(_, key)| key)
                    .or_else(|| group.map(str::to_owned)),
            ));
        }
        // Who changed what: the newest rows of the audit log, or those of one party or one بکری.
        "audit" => {
            let party = u.parties.iter().find_map(|p| match &p.outcome {
                Outcome::Found(hit) => role_word(hit.role)
                    .filter(|_| hit.role != PartyRole::Partner)
                    .map(|word| (p.text.as_str(), word, hit.key.clone())),
                _ => None,
            });
            if u.parties.len() > usize::from(party.is_some()) || !u.items.is_empty() {
                return None;
            }
            if let Some((text, word, _)) = &party {
                plan.find(text, word);
            }
            plan.read(app_state_of(
                "audit",
                party.map(|(_, _, key)| key),
                None,
                u.bikri_number,
            ));
        }
        "staff_munshi" => plan.find("munshi", "staff"),
        "staff_collector" => plan.find("collector", "staff"),
        "staff_labour" => plan.find("labour", "staff"),
        "staff_all" => plan.find("all", "staff"),
        "heads" => plan.find("all", "expense_head"),
        "packings" => plan.find("all", "packing"),
        "markas" => plan.find("all", "marka"),
        "stores" => plan.find("all", "store"),
        "narrations" => plan.find("all", "narration"),
        // The vouchers of one type only: R15 with the voucher type.
        "vouchers_receipt" | "vouchers_payment" | "vouchers_advance" => {
            let voucher_type = match id {
                "vouchers_receipt" => "receipt",
                "vouchers_payment" => "payment",
                _ => "advance",
            };
            plan.read(report(
                "R15",
                Rep {
                    from,
                    to,
                    voucher_type: Some(voucher_type),
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
        }
        _ => return None,
    }
    Some(())
}

/// A buyer's or a seller's figure.
fn party_figure(
    kind: FigureKind,
    text: &str,
    hit: &PartyHit,
    period: Option<&PeriodRef>,
    listing: bool,
    plan: &mut Plan,
) -> Option<()> {
    let role = hit.role;
    // What a buyer bought is his lines: "kya kya khareeda", "kitne ka saman khareeda".
    let kind = if listing && role == PartyRole::Buyer && kind == FigureKind::Sale {
        FigureKind::Purchases
    } else {
        kind
    };
    let word = role_word(role).filter(|_| role != PartyRole::Partner)?;
    let (from, to) = dates(period);
    let key = hit.key.clone();
    match kind {
        // The card: balance with its side, advance, payable, oldest open bill. Now, not for a period.
        FigureKind::Balance | FigureKind::Dues | FigureKind::Payable | FigureKind::Advance => {
            if period.is_some() {
                return None;
            }
            // A buyer's card holds no payable and no advance (a payment to a buyer is cash lent).
            if role == PartyRole::Buyer && matches!(kind, FigureKind::Payable | FigureKind::Advance)
            {
                return None;
            }
            plan.find(text, word);
            plan.read(party_card(&key));
            plan.line(
                Who::Party(key),
                match kind {
                    FigureKind::Payable => What::Payable,
                    FigureKind::Advance => What::Advance,
                    _ => What::Balance,
                },
            );
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
        // "naseer ka hisab kya he": his account as of now. The answer is his balance, read in two places on
        // his card (D7, plan 59 F3), and the newest rows of his ledger (R13) with its totals, newest first.
        FigureKind::Ledger if period.is_none() => {
            plan.find(text, word);
            plan.read(party_card(&key));
            plan.read(report(
                "R13",
                Rep {
                    party: Some(key.clone()),
                    sort: Some(json!([{"column": "date", "order": "desc"}])),
                    rows: Some(LEDGER_ROWS),
                    ..Rep::default()
                },
            ));
            plan.line(Who::Party(key), What::Balance);
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
        // What a buyer bought over a period: view A with the buyer holds each item with its nag and
        // amount (a day is the day sheet above).
        FigureKind::Purchases if role == PartyRole::Buyer => {
            plan.find(text, word);
            plan.read(analytics(
                "A",
                An {
                    from,
                    to,
                    buyer: Some(key),
                    ..An::default()
                },
            ));
            plan.dated = period.is_some();
        }
        // What a seller sold, line by line: the lot rows of his seller summary (R6) hold the item, the
        // qty and the amount.
        FigureKind::Sale if listing && role == PartyRole::Seller => {
            plan.find(text, word);
            plan.read(report(
                "R6",
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
            let who = if role == PartyRole::Seller {
                a.seller = Some(key.clone());
                Who::Seller(key)
            } else {
                a.buyer = Some(key.clone());
                Who::Buyer(key)
            };
            plan.read(analytics(view, a));
            plan.dated = period.is_some();
            // A list ("kis kis din kitna commission diya") asked for the breakdown: the reads are
            // made and the model writes it; a total would be one line of what was asked.
            if !listing {
                plan.line(who, what_of(kind)?);
            }
        }
        _ => return None,
    }
    Some(())
}

/// The figure of a commission, sale, count or laga question.
fn what_of(kind: FigureKind) -> Option<What> {
    Some(match kind {
        FigureKind::Commission => What::Commission,
        FigureKind::Sale => What::Sale,
        FigureKind::SaleCount => What::SaleCount,
        FigureKind::Laga => What::Laga,
        _ => return None,
    })
}

/// A partner's figure: his total (R31).
///
/// R31 runs in the partner's kul raqam (E2E-37: ب 50,746, his capital 50,000). Only that is its
/// figure; his capital or his share is the model's (money review R-f). It is read for every partner,
/// with no partner filter: only that read carries the total with its second place (the partner
/// overview), and the answer cites his row.
fn partner_figure(
    kind: FigureKind,
    text: &str,
    hit: &PartyHit,
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    let (from, to) = dates(period);
    if period.is_some() && !matches!(kind, FigureKind::FruitTaken | FigureKind::PartnerTaken) {
        return None;
    }
    match kind {
        // The money he took and gave back: the expense report for him over the days (R29 has the
        // partner filter; its taken and returned columns are the figure).
        FigureKind::PartnerTaken => {
            plan.find(text, "partner");
            plan.read(report(
                "R29",
                Rep {
                    from,
                    to,
                    partner: Some(hit.key.clone()),
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
        }
        // His fruit taken: the register for him over the days.
        FigureKind::FruitTaken => {
            plan.find(text, "partner");
            plan.read(report(
                "R37",
                Rep {
                    from,
                    to,
                    partner: Some(hit.key.clone()),
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
            plan.line(Who::Partner(Some(hit.key.clone())), What::FruitValue);
        }
        FigureKind::PartnerTotal => {
            plan.read(report("R31", Rep::default()));
            plan.line(Who::Partner(Some(hit.key.clone())), What::PartnerTotal);
        }
        // His capital now: R31 for him alone holds it, read in two places (the capital history and
        // the partner overview). Not his kul raqam, which is another number.
        FigureKind::Capital => {
            plan.find(text, "partner");
            plan.read(report(
                "R31",
                Rep {
                    partner: Some(hit.key.clone()),
                    ..Rep::default()
                },
            ));
            plan.line(Who::Partner(Some(hit.key.clone())), What::PartnerCapital);
        }
        _ => return None,
    }
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
        // The rate of view A is the average of the whole period: with no period it would be the
        // average of all time, which is no rate for today (money review R-d). The model asks.
        FigureKind::Rate if period.is_none() => return None,
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
            if let Some(what) = what_of(kind).filter(|w| *w != What::SaleCount) {
                plan.line(Who::Item(key.to_owned()), what);
            } else if kind == FigureKind::Rate && plan.average {
                // The question says "average": view A's average rate of the item, named as the average.
                plan.line(Who::Item(key.to_owned()), What::AvgRate);
            }
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
            plan.line(Who::Item(key.to_owned()), What::Stock);
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
    side: Option<super::CommissionSide>,
    plan: &mut Plan,
) -> Option<()> {
    use super::CommissionSide as Side;
    // What a buyer bought of an item ("how much tomato did Faisal buy this year"): the register of his
    // lines (R2), whose totals hold the value and the bill total over the whole filter.
    if kind == FigureKind::Purchases && phit.role == PartyRole::Buyer {
        let (from, to) = dates(period);
        plan.find(ptext, "buyer");
        plan.find(itext, "item");
        plan.read(report(
            "R2",
            Rep {
                from,
                to,
                party: Some(phit.key.clone()),
                item: Some(ikey.to_owned()),
                ..Rep::default()
            },
        ));
        plan.dated = period.is_some();
        return Some(());
    }
    let average = kind == FigureKind::Rate && plan.average && phit.role == PartyRole::Seller;
    if !matches!(
        kind,
        FigureKind::Commission | FigureKind::Laga | FigureKind::Sale
    ) && !average
    {
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
    // The instant lines of a named party's figure of one item, each from the row of the item in view A
    // (D7, plan 59 S-A1, Q2062): a buyer's commission is his own side (`buyer_comm`) and only when the
    // question asks that side, his Laga is its own column; a seller's commission is two labelled lines,
    // the one charged to him and the total on his goods (Umar, 05-11-2026). A sale has no line: the
    // view holds the seller's amount, not what the buyer paid.
    let key = phit.key.clone();
    let item = ikey.to_owned();
    if phit.role == PartyRole::Seller {
        a.seller = Some(key.clone());
        // The seller side asked ("seller commission on Rahim's banana") is the one figure charged to
        // him; no side named is the two labelled lines; the buyer side asked of a seller is the model's.
        if kind == FigureKind::Commission && side != Some(Side::Buyer) {
            plan.line(Who::SellerItem(key, item), What::Commission);
        } else if average {
            plan.line(Who::SellerItem(key, item), What::AvgRate);
        } else if kind == FigureKind::Sale && plan.qty {
            plan.line(Who::SellerItem(key, item), What::Qty);
        }
    } else {
        a.buyer = Some(key.clone());
        match kind {
            FigureKind::Commission if side == Some(Side::Buyer) => {
                a.side = Some("buyer");
                plan.line(Who::BuyerItem(key, item), What::Commission);
            }
            FigureKind::Laga => plan.line(Who::BuyerItem(key, item), What::Laga),
            FigureKind::Sale if plan.qty => plan.line(Who::BuyerItem(key, item), What::Qty),
            _ => {}
        }
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
            // R21 holds the BUYERS' dues of the city: a sellers word asks another figure.
            if plan.seller_word {
                return None;
            }
            plan.find(text, "city");
            plan.read(report(
                "R21",
                Rep {
                    city: Some(hit.key.clone()),
                    ..Rep::default()
                },
            ));
            plan.line(Who::City(hit.key.clone()), What::CityDues);
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
        // What a collector collected: the receipts register for him (R23 has the collector filter).
        (RecordKind::Staff, FigureKind::Receipts) if period.is_some() => {
            plan.find(text, "collector");
            plan.read(report(
                "R23",
                Rep {
                    from,
                    to,
                    collector: Some(hit.key.clone()),
                    ..Rep::default()
                },
            ));
            plan.dated = true;
            plan.line(Who::Collector(hit.key.clone()), What::Receipts);
        }
        // The report has no expense-head filter: the period is read and the row is found in it. Every row
        // is read (up to 100), so a head outside the first 20 is not "not found" or 0 (D7 R12).
        (RecordKind::ExpenseHead, FigureKind::Expense) if period.is_some() => {
            plan.read(report(
                "R29",
                Rep {
                    from,
                    to,
                    rows: Some(100),
                    ..Rep::default()
                },
            ));
            plan.dated = true;
            plan.line(Who::Head(hit.name_en.clone()), What::Expense);
        }
        _ => return None,
    }
    Some(())
}

/// A figure of the books as a whole.
fn shop_figure(
    kind: FigureKind,
    total_dues: bool,
    quantity: bool,
    guests: bool,
    trial_sides: &[TrialSide],
    period: Option<&PeriodRef>,
    plan: &mut Plan,
) -> Option<()> {
    let (from, to) = dates(period);
    let today = period.is_some_and(|p| p.kind == PeriodKind::Today);
    match kind {
        // Sales and commission: the tiles for today, else view A (view B for the number of بکری).
        FigureKind::Sale | FigureKind::SaleCount | FigureKind::Commission | FigureKind::Laga => {
            period?;
            // A quantity is in the view only; the tile holds the amount.
            if today && kind != FigureKind::Laga && !quantity {
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
                        from: from.clone(),
                        to: to.clone(),
                        ..An::default()
                    },
                ));
                plan.dated = true;
            }
            // One day has a second place: the day summary (R34) holds the same figures, so the
            // answer is given only when the two agree (money review, 05-11-2026). Today's sale,
            // count and commission are checked by the tiles already.
            if !quantity && is_one_day(period) && !(today && kind != FigureKind::Laga) {
                plan.read(report(
                    "R34",
                    Rep {
                        from,
                        to,
                        ..Rep::default()
                    },
                ));
            }
            plan.line(Who::Shop, what_of(kind)?);
        }
        FigureKind::Cash if period.is_none() || today => {
            plan.read(tiles());
            plan.line(Who::Shop, What::Cash);
        }
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
            // The cash at the end of the period: "kal rat ko roknad kitni thi", "cash balance this month".
            plan.line(Who::Shop, What::CashClosing);
        }
        // The shop's dues are a list: R20 holds the total over the whole filter and the real list, so
        // the button opens it (lead ruling of 04-11-2026). The tile is read too: its figure is checked
        // against R20, and the answer gives the tile's. One party's dues stay on his card.
        FigureKind::Dues if period.is_none() => {
            plan.read(tiles());
            plan.read(report(
                "R20",
                Rep {
                    sort: Some(sorted("balance", "desc")),
                    ..Rep::default()
                },
            ));
            plan.line(
                Who::Shop,
                if total_dues {
                    What::TotalDues
                } else {
                    What::BuyersDues
                },
            );
        }
        FigureKind::Payable if period.is_none() => {
            plan.read(report(
                "R24",
                Rep {
                    sort: Some(sorted("payable", "desc")),
                    ..Rep::default()
                },
            ));
            plan.line(Who::Shop, What::ShopPayable);
        }
        // Advances, the dead amount, money lent: the tiles, now.
        FigureKind::Advance | FigureKind::DeadAmount | FigureKind::MoneyLent
            if period.is_none() =>
        {
            plan.read(tiles());
            plan.line(
                Who::Shop,
                match kind {
                    FigureKind::Advance => What::AdvancesOpen,
                    FigureKind::DeadAmount => What::DeadAmount,
                    _ => What::MoneyLent,
                },
            );
        }
        // The goods left: R10 holds the lots and the totals over all of them, oldest first.
        FigureKind::Stock if period.is_none() => {
            // The tile counts the lots of our shop; R10 also lists trip purchases and garden
            // arrivals. The answer is given only when the two agree.
            plan.read(tiles());
            plan.read(report(
                "R10",
                Rep {
                    sort: Some(sorted("days", "desc")),
                    ..Rep::default()
                },
            ));
            plan.line(Who::Shop, What::Stock);
        }
        // The fruit taken: the register of the days, for the shop's guests alone when they are named.
        FigureKind::FruitTaken => {
            plan.read(report(
                "R37",
                Rep {
                    from,
                    to,
                    partner: guests.then(|| "guests".to_owned()),
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
            plan.line(
                if guests { Who::Guests } else { Who::Shop },
                What::FruitValue,
            );
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
            // R23 lists every receipt, a seller's return of advance money too (PRD FR-15.12): "gahakon se
            // wasooli" is the buyers' part only, which the register cannot split, so a role word gives no
            // instant line (money review of F3a).
            if !plan.role_word {
                plan.line(Who::Shop, What::Receipts);
            }
        }
        // What the partners took: R29 holds it by partner (the partner section decides what it shows).
        FigureKind::PartnerTaken => {
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
            plan.line(Who::Shop, What::Expense);
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
            // The side or sides of the trial balance as of now: one line each, in the order asked. With a
            // period ("at the start") it is another trial balance (R19): no line.
            if period.is_none() {
                for side in trial_sides {
                    plan.line(
                        Who::Shop,
                        match side {
                            TrialSide::Debit => What::TrialDebit,
                            TrialSide::Credit => What::TrialCredit,
                        },
                    );
                }
            }
        }
        // All the partners' totals (the app asks for the code while the section is locked).
        FigureKind::PartnerTotal => {
            plan.read(report(
                "R31",
                Rep {
                    from,
                    to,
                    ..Rep::default()
                },
            ));
            plan.dated = period.is_some();
            if period.is_none() {
                plan.line(Who::Partner(None), What::PartnerTotal);
            }
        }
        _ => return None,
    }
    Some(())
}

// ---- a list --------------------------------------------------------------------------------------------

fn list(u: &Understanding, period: Option<&PeriodRef>, plan: &mut Plan) -> Option<()> {
    if !u.parties.is_empty() || u.items.len() > 1 || !u.records.is_empty() {
        return None;
    }
    // One item named ("kele ka sab se bara gahak"): the item filter of the views of buyers and
    // sellers. A name that was not found is the model's.
    let item = match u.items.first() {
        None => None,
        Some(i) => match &i.outcome {
            Outcome::Found(hit) => Some((hit.name_en.clone(), hit.key.clone())),
            _ => return None,
        },
    };
    // A list inside a group ("kaun si sabzi sab se zyada bikti hai"): the views and the stock report
    // with the group, once the tools take it.
    let group = u.item_group.map(GroupWord::tool_word);
    if group.is_some() && !group_filter_ready() {
        return None;
    }
    let ask = u.list?;
    let measure = u.figures.value.first().copied();
    if u.figures.value.len() > 1 {
        return None;
    }
    // "پچھلے مہینے کے تمام خرچوں کی فہرست": the expenses of the period (R29); "ڈوبی رقم میں کس کس کا نام
    // ہے": the dead amount, party by party (R38). No party, item or group beside them.
    if item.is_none() && group.is_none() {
        match measure {
            Some(FigureKind::Expense) if ask.rank.is_none() => {
                let (from, to) = dates(period);
                plan.read(report(
                    "R29",
                    Rep {
                        from,
                        to,
                        ..Rep::default()
                    },
                ));
                plan.dated = period.is_some();
                return Some(());
            }
            Some(FigureKind::DeadAmount) if period.is_none() => {
                plan.read(report("R38", Rep::default()));
                return Some(());
            }
            _ => {}
        }
    }
    // "kis kis ke dues hain": the figure says whose (dues are buyers', payables sellers', goods
    // items'). A balance alone could be either, so it is not guessed.
    // A group of items ("kaun si sabzi") is a list of items, whatever "kaun si" looks like, unless the
    // question says whose ("sabzi ka sab se bara gahak": the buyers, with the group as a filter).
    let over = ask
        .over
        .filter(|o| matches!(o, ListOver::Buyers | ListOver::Sellers) && group.is_some())
        .or(group.map(|_| ListOver::Items))
        .or(ask.over.filter(|o| *o != ListOver::Parties))
        .or(match measure {
            Some(FigureKind::Dues) => Some(ListOver::Buyers),
            Some(FigureKind::Payable) => Some(ListOver::Sellers),
            Some(FigureKind::Stock) => Some(ListOver::Items),
            _ => None,
        })?;
    // A list of items for one item is no list: the item is the model's (it asks view A or R10).
    if item.is_some() && over == ListOver::Items {
        return None;
    }
    let (from, to) = dates(period);
    let order = |least: bool| if least { "asc" } else { "desc" };
    let least = ask.rank == Some(Rank::Least);
    match (over, measure) {
        // Who owes, and who is owed (R20, R24 are as of now).
        (ListOver::Buyers, Some(FigureKind::Dues | FigureKind::Balance))
            if period.is_none() && group.is_none() && item.is_none() =>
        {
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
            if period.is_none() && group.is_none() && item.is_none() =>
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
            // "sab se zyada maal kis cheez ka para hai": the item with the most goods left first.
            let ranked = ask
                .rank
                .map(|_| (sorted("balance", order(least)), ask.n.or(Some(RANKED_ROWS))));
            let (sort, rows) = ranked.unzip();
            plan.read(report(
                "R10",
                Rep {
                    layout: Some("item"),
                    sort,
                    rows: rows.unwrap_or(ask.n),
                    item_group: group,
                    ..Rep::default()
                },
            ));
        }
        // The top ten by total commission: view H holds the items, the sellers and the buyers.
        (ListOver::Items | ListOver::Sellers | ListOver::Buyers, Some(FigureKind::Commission))
            if ask.rank == Some(Rank::Most) && group.is_none() && item.is_none() =>
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
            if let Some((text, _)) = &item {
                plan.find(text, "item");
            }
            plan.read(analytics(
                view,
                An {
                    from,
                    to,
                    item: item.map(|(_, key)| key),
                    sort: Some(sorted(column, order(least))),
                    rows: ask.n,
                    item_group: group,
                    ..An::default()
                },
            ));
            plan.dated = period.is_some();
        }
        _ => return None,
    }
    Some(())
}
