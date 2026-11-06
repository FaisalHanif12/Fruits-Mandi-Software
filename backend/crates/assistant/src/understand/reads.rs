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

use mandi_reports::analytics::{AnalyticsCompare, compare_window};
use serde_json::{Value, json};

use super::lexicon::{Cue, key};
use super::scan::Scan;
use super::who::Names;
use super::{
    CapitalFlow, Doubt, FigureKind, GradeWord, GroupWord, Intent, ItemHit, ListOver, Measure,
    Outcome, PartyHit, PartyRole, PeriodKind, PeriodRef, Rank, RecordHit, RecordKind,
    Understanding,
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

/// True when the question names a garden, a trip, own goods, the guests or the one-time people (پختہ
/// کھاتہ), by word. The guests are no scope for the fruit taken: the fruit register has a read for them
/// (R37, partner `guests`). The one-time people have no read of their own for a figure: the shop's
/// dues are not theirs ("pukhta khata ka kul baqaya").
pub(super) fn scoped(scan: &Scan, kinds: &[FigureKind]) -> bool {
    let guests_fruit = kinds.contains(&FigureKind::FruitTaken);
    own_goods(scan)
        || scan.has_exact(Cue::Pukhta)
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
    // "tahir ne kitni peshgi li hui hai": the state "has taken", which is the advance still open.
    if kinds == [FigureKind::Advance] {
        let text = format!(" {} ", scan.words.join(" "));
        const STATE: [&str; 7] = [
            "li hui hai",
            "li hui he",
            "liya hua hai",
            "liya hua he",
            "li hui hain",
            "لی ہوئی ہے",
            "لیا ہوا ہے",
        ];
        if STATE
            .iter()
            .any(|p| text.contains(&format!(" {} ", key(p))))
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
    capital_event
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
    /// `paid` or `due` (R9, R38).
    status: Option<&'static str>,
    /// One kind of voucher, in its stored word (R15).
    voucher_type: Option<&'static str>,
    /// How the report lays out its rows (R14 `detail`, R33 `trip` `item` `day` `month` `partner`,
    /// R36 `partner` `garden`).
    layout: Option<&'static str>,
    /// All the items of a group, `fruit` or `vegetable` (R2 to R4, R7, R10, R27); written only when
    /// set (a field of the tool when the schema has it).
    item_group: Option<&'static str>,
}

fn report(id: &str, r: Rep) -> PlannedRead {
    let input = json!({
        "id": id, "from": r.from, "to": r.to, "as_of": r.as_of, "party": r.party,
        "item": r.item, "city": r.city, "adda": r.adda, "collector": r.collector,
        "partner": r.partner, "trip": null, "garden": null, "bikri": null,
        "seller": r.seller, "zamindar": r.zamindar, "status": r.status,
        "voucher_type": r.voucher_type, "layout": r.layout,
        "group_by": null, "where": null, "sort": r.sort, "rows": r.rows,
        "item_group": r.item_group
    });
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
}

fn analytics(view: &str, a: An) -> PlannedRead {
    let mut input = json!({
        "view": view, "from": a.from, "to": a.to, "group_by": null, "item": a.item,
        "item_group": a.item_group, "seller": a.seller, "buyer": a.buyer, "city": null,
        "adda": null, "garden": null, "trip": null, "goods": a.goods, "partner": null,
        "side": null, "compare": null, "sort": a.sort, "rows": a.rows
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
    /// A buyer or a seller whose party card is read.
    Party(String),
    /// A partner's row of R31; `None` for every partner's row.
    Partner(Option<String>),
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
    /// Goods left (R10).
    Stock,
    /// A partner's kul raqam (R31).
    PartnerTotal,
    /// A partner's capital now (R31 for the partner, the `capital` part).
    PartnerCapital,
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

/// The reads of a question and the figures an instant answer gives from them.
pub(super) fn plan(u: &Understanding) -> Option<Planned> {
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
    let forecast = u.intent.value == Intent::Forecast;
    if !u.sure
        || u.compare
        || (u.two_figures && !flow_figures(u))
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
    // "ab tak" beside what is true now is the same question ("akram ka bakaya ab tak", "partners ki
    // kul raqam abhi tak").
    let period = period.filter(|p| !(p.kind == PeriodKind::UpToNow && asks_now(u)));
    let mut plan = Plan::default();
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
    // A quantity is read from the same views, but the figure of the amount is no answer to it.
    let lines = if matches!(u.other_measure, Measure::Quantity | Measure::Breakdown) {
        Vec::new()
    } else {
        plan.lines
    };
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
    Some(Planned {
        reads,
        lines: Vec::new(),
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
    scan.words.iter().enumerate().any(|(at, w)| {
        is(w, &ALWAYS)
            || (is(w, &HEAD)
                && scan
                    .word(at.saturating_add(1))
                    .is_some_and(|next| is(next, &RATEISH)))
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
            Outcome::Found(hit) if hit.role == PartyRole::Buyer && !hit.pukhta => {
                Some((party.text.clone(), hit.key.clone()))
            }
            _ => return None,
        },
        _ => return None,
    };
    let period = u.periods.first();
    let (from, to) = dates(period);
    let mut plan = Plan::default();
    if let Some((text, _)) = &buyer {
        plan.find(text, "buyer");
    }
    plan.find(&goods.name_en, "item");
    plan.read(report(
        "R2",
        Rep {
            from,
            to,
            party: buyer.map(|(_, key)| key),
            item: Some(goods.key.clone()),
            sort: Some(json!([
                {"column": "date", "order": "desc"},
                {"column": "entry", "order": "desc"}
            ])),
            rows: Some(1),
            ..Rep::default()
        },
    ));
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
            Outcome::Found(hit)
                if !hit.pukhta && matches!(hit.role, PartyRole::Buyer | PartyRole::Seller) =>
            {
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
                lines: Vec::new(),
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
            // One party's balance: his key goes in `party`; a pukhta person has none.
            (PartyRole::Seller | PartyRole::Buyer, "party_balance") if !hit.pukhta => {
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
    if kinds.len() > 1 && !flow_figures(u) {
        return None;
    }
    if kinds.is_empty() {
        return None;
    }
    for kind in kinds {
        figure_of_kind(u, *kind, period, plan)?;
    }
    Some(())
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
                item_figure(kind, text, &hit.key, period, plan)?;
            }
            Some(())
        }
        (1, 1, None) => {
            let (ptext, phit) = parties.first().copied()?;
            let (itext, ihit) = items.first().copied()?;
            party_item_figure(kind, (ptext, phit), (itext, &ihit.key), period, plan)
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
    plan: &mut Plan,
) -> Option<()> {
    // What a buyer bought of an item ("how much tomato did Faisal buy this year"): the register of his
    // lines (R2), whose totals hold the value and the bill total over the whole filter.
    if kind == FigureKind::Purchases && phit.role == PartyRole::Buyer && !phit.pukhta {
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
fn shop_figure(
    kind: FigureKind,
    total_dues: bool,
    quantity: bool,
    guests: bool,
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
