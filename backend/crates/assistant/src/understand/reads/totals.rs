//! Plans of the figures that the shop's reports hold as totals (D7, plan 59 F3): the quantity of a sale,
//! the sales of one payment type, a seller's net sale. They share the shapes of `reads`.
//!
//! None of them adds, subtracts or rounds anything: each line of the answer is a cell of the report or
//! the view the plan reads (Code Standards Q5).

use super::{
    An, FigureKind, Intent, Line, Measure, NOT_CARRIED, Outcome, PartyRole, PayKind, Plan, Planned,
    Rep, Understanding, What, Who, analytics, dates, key, period_read, report,
};
use crate::understand::lexicon::Cue;
use crate::understand::scan::Scan;

/// Words that ask a quantity in نگ and name no packing: "kitne nag bika", "how many units", "qty".
const QTY_WORDS: [&str; 11] = [
    "nag",
    "نگ",
    "qty",
    "quantity",
    "quantities",
    "units",
    "unit",
    "tadad",
    "taadad",
    "تعداد",
    "کوانٹیٹی",
];

/// True when the question asks the quantity of an item, a party or the shop, in { نگ }: one sale figure,
/// the measure is a quantity, a quantity word stands in it, and no packing or weight word (peti, bori,
/// kg) is beside it, because the views count نگ of every packing.
///
/// Example: "is saal aam kitni qty bika" is true; "aloo ki peti me kitni bikri" is false (a packing).
pub(in crate::understand) fn qty_asked(
    scan: &Scan,
    kinds: &[FigureKind],
    measure: Measure,
) -> bool {
    if kinds != [FigureKind::Sale] || measure != Measure::Quantity {
        return false;
    }
    let is_qty = |w: &str| QTY_WORDS.iter().any(|x| key(x) == w);
    if !scan.words.iter().any(|w| is_qty(w)) {
        return false;
    }
    // Any other unit word (a packing, a weight) makes it another question.
    !scan
        .hits
        .iter()
        .any(|h| h.cue == Cue::Unit && !scan.words.get(h.start).is_some_and(|w| is_qty(w)))
}

/// Words of money lent: with them "udhar" is the loan, not a credit sale.
const LENT_WORDS: [&str; 9] = [
    "raqam", "rakam", "lent", "loan", "loans", "qarz", "karz", "رقم", "قرض",
];

/// The single words that mean "paid in cash" beside a sale ("sold for cash"): the cash book's own phrases
/// ("cash in hand", "cash book") are longer hits and never these.
const CASH_WORDS: [&str; 5] = ["cash", "naqad", "nakad", "naqd", "نقد"];

/// Whether the question names credit and cash payment: `(credit, cash)`.
fn pay_flags(scan: &Scan) -> (bool, bool) {
    let has = |list: &[&str]| scan.words.iter().any(|w| list.iter().any(|x| key(x) == *w));
    let lent = has(&LENT_WORDS);
    let credit = scan.has_exact(Cue::PayCredit)
        || (!lent && scan.has(Cue::Sale) && has(&["udhar", "udhaar", "ادھار"]));
    // "naqdi mandi" is a buyer's name.
    let naqdi = scan.has_exact(Cue::PayCash) && !has(&["mandi", "منڈی"]);
    // "sold for cash", "cash sales": the word cash beside a sale or a sold word, as a hit of its own.
    let cash_word = scan.hits.iter().any(|h| {
        h.cue == Cue::Cash
            && h.end.saturating_sub(h.start) == 1
            && scan
                .words
                .get(h.start)
                .is_some_and(|w| CASH_WORDS.iter().any(|x| key(x) == *w))
    }) && (scan.has(Cue::Sold) || scan.has(Cue::Sale));
    (credit, naqdi || cash_word)
}

/// The payment type of the sales a question asks, when it names ONE: "aaj udhaar me kitni bikri hui" is
/// credit, "naqdi bikri" is cash. "Naqdi Mandi" is a buyer's name, and "udhar raqam" the money lent. A
/// question that names both kinds has none.
///
/// Example: "how much was sold on credit today" is credit; "udhar raqam kitni hai" is none.
pub(in crate::understand) fn pay_kind(scan: &Scan) -> Option<PayKind> {
    match pay_flags(scan) {
        (true, false) => Some(PayKind::Credit),
        (false, true) => Some(PayKind::Cash),
        _ => None,
    }
}

/// True when a payment-type word stands in the question, whether it names one kind or both.
pub(in crate::understand) fn pay_asked(scan: &Scan) -> bool {
    let (credit, cash) = pay_flags(scan);
    credit || cash
}

/// Words for the value of a thing ("qeemat", "value", "worth"). The quantity of the fruit is another figure.
const VALUE_WORDS: [&str; 10] = [
    "value",
    "valew",
    "valu",
    "qeemat",
    "qimat",
    "keemat",
    "worth",
    "قیمت",
    "قدر",
    "کتنے کا",
];

/// True when the question asks the VALUE of the fruit taken and not its quantity: a value word stands in it,
/// and no quantity word, "how many", or "how much" right before the fruit word ("how much fruit did A take"
/// asks the quantity).
///
/// Example: "what is the total value of the fruit taken so far" is true; "how much fruit did A take and its
/// value" is false.
pub(in crate::understand) fn value_asked(scan: &Scan) -> bool {
    let is = |w: &str, list: &[&str]| list.iter().any(|x| key(x) == w);
    if !scan.words.iter().any(|w| is(w, &VALUE_WORDS)) {
        return false;
    }
    if scan.words.iter().any(|w| is(w, &QTY_WORDS)) || scan.has(Cue::Unit) {
        return false;
    }
    // "how much fruit", "kitna phal": a how-much word at most two words before a fruit word.
    let fruit_after_how_much = scan.words.iter().enumerate().any(|(at, w)| {
        is(
            w,
            &["kitna", "kitni", "kitne", "کتنا", "کتنی", "کتنے", "many"],
        ) || (w == "much" && at > 0 && scan.word(at.saturating_sub(1)) == Some("how"))
    }) && scan.hits.iter().any(|h| {
        matches!(h.cue, Cue::FruitWord | Cue::FruitTaken)
            && scan.words.iter().enumerate().any(|(at, w)| {
                (is(
                    w,
                    &[
                        "kitna", "kitni", "kitne", "کتنا", "کتنی", "کتنے", "many", "much",
                    ],
                )) && h.start > at
                    && h.start.saturating_sub(at) <= 2
            })
    });
    !fruit_after_how_much
}

/// The lines that are figures of a report's totals or a view's column (D7, plan 59 F3).
fn is_total_line(what: What) -> bool {
    matches!(
        what,
        What::Qty
            | What::AvgRate
            | What::Expense
            | What::Receipts
            | What::CashClosing
            | What::CashSales
            | What::CreditSales
            | What::NetSale
            | What::FruitValue
            | What::CapitalIn
            | What::CapitalOut
            | What::CashIn
            | What::CashOut
    )
}

/// The lines an instant answer may give. Those of a report's totals are not given when a word of the
/// question is explained by nothing ("chai" of "chai ka kharcha": the head was not found, and the shop's
/// whole expense would be the answer to another question), nor the fruit's value unless it is the value
/// that was asked.
pub(super) fn keep_lines(u: &Understanding, lines: Vec<Line>) -> Vec<Line> {
    lines
        .into_iter()
        .filter(|l| {
            !((u.unexplained
                || u.how_much_twice
                || u.compare
                || u.doubts.iter().any(|d| NOT_CARRIED.contains(d)))
                && (is_total_line(l.what) || matches!(l.who, Who::Group(_))))
        })
        .filter(|l| !(l.what == What::FruitValue && !u.value_asked))
        .collect()
}

/// The sales of one payment type over a period: the register of cash sales (R3) or of credit sales (R4),
/// whose total `value` is the amount ("aaj udhaar me kitni bikri hui").
///
/// `None` for anything else: a party, an item, a group, two periods, a comparison, two figures, a
/// doubt, or no period.
pub(super) fn pay_sale_plan(u: &Understanding) -> Option<Planned> {
    let pay = u.pay_kind?;
    // "udhar" is also the word of money lent: it is read as the payment type here, so that cue is no second
    // figure; nothing else may stand beside the sale.
    let sale_only = u
        .figures
        .value
        .iter()
        .all(|k| matches!(k, FigureKind::Sale | FigureKind::MoneyLent))
        && u.figures.value.contains(&FigureKind::Sale);
    if !u.sure
        || u.intent.value != Intent::Figure
        || !sale_only
        || !u.parties.is_empty()
        || !u.items.is_empty()
        || !u.records.is_empty()
        || u.item_group.is_some()
        || u.compare
        || u.quality
        || u.scoped
        || u.packing_filter
        || u.document_number
        || u.bikri_number.is_some()
        || u.qty_asked
        || u.asks_count
        || u.unexplained
        || u.how_much_twice
        || u.truck.is_some()
        || u.other_measure == Measure::Quantity
        || u.last_one
        || u.sum_asked
        || u.periods.len() != 1
        || u.doubts.iter().any(|d| NOT_CARRIED.contains(d))
    {
        return None;
    }
    let period = u.periods.first()?;
    if period.future {
        return None;
    }
    let (from, to) = dates(Some(period));
    let id = match pay {
        PayKind::Cash => "R3",
        PayKind::Credit => "R4",
    };
    let mut plan = Plan::default();
    plan.read(report(
        id,
        Rep {
            from,
            to,
            ..Rep::default()
        },
    ));
    let mut reads = vec![period_read(period)?];
    reads.extend(plan.reads);
    Some(Planned {
        reads,
        lines: keep_lines(
            u,
            vec![Line {
                who: Who::Shop,
                what: match pay {
                    PayKind::Cash => What::CashSales,
                    PayKind::Credit => What::CreditSales,
                },
            }],
        ),
    })
}

/// A seller's net sale ("bashir ki pichle mahine ki khalis raqam", "what was Bashir's net last month"): the
/// words that name a partner's خالص are here about a SELLER, so the figure is his net sale, the `net` column
/// of view B.
///
/// `None` unless exactly one seller was found and one period named.
pub(super) fn net_sale_plan(u: &Understanding) -> Option<Planned> {
    if !u.sure
        || u.intent.value != Intent::Figure
        || u.figures.value.as_slice() != [FigureKind::PartnerTotal]
        || u.compare
        || u.scoped
        || u.item_group.is_some()
        || u.quality
        || u.truck.is_some()
        || u.unexplained
        || u.how_much_twice
        || u.qty_asked
        || u.asks_count
        || u.packing_filter
        || u.bikri_number.is_some()
        || u.document_number
        || !u.items.is_empty()
        || !u.records.is_empty()
        || u.periods.len() != 1
        || u.doubts.iter().any(|d| NOT_CARRIED.contains(d))
    {
        return None;
    }
    let [party] = u.parties.as_slice() else {
        return None;
    };
    let Outcome::Found(hit) = &party.outcome else {
        return None;
    };
    if hit.role != PartyRole::Seller || hit.pukhta {
        return None;
    }
    let period = u.periods.first()?;
    if period.future {
        return None;
    }
    let (from, to) = dates(Some(period));
    let mut plan = Plan::default();
    plan.find(&party.text, "seller");
    plan.read(analytics(
        "B",
        An {
            from,
            to,
            seller: Some(hit.key.clone()),
            ..An::default()
        },
    ));
    let mut reads = vec![period_read(period)?];
    reads.extend(plan.finds);
    reads.extend(plan.reads);
    Some(Planned {
        reads,
        lines: keep_lines(
            u,
            vec![Line {
                who: Who::Seller(hit.key.clone()),
                what: What::NetSale,
            }],
        ),
    })
}

/// The lines of a quantity question: each sale line is read as its quantity. Every line must be one that has
/// a quantity (the shop, an item, a seller, a buyer, or a party's item); otherwise none is given and the
/// model answers.
pub(super) fn qty_lines(lines: Vec<Line>) -> Vec<Line> {
    let mut out = Vec::new();
    for line in lines {
        let ok = matches!(
            (&line.who, line.what),
            (
                Who::Shop | Who::Item(_) | Who::Seller(_) | Who::Buyer(_),
                What::Sale
            ) | (Who::SellerItem(..) | Who::BuyerItem(..), What::Qty)
        );
        if !ok {
            return Vec::new();
        }
        out.push(Line {
            who: line.who,
            what: What::Qty,
        });
    }
    out
}
