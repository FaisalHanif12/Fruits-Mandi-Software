//! The words that name WHICH column of a sale or a commission is asked (plan 82, the wrong-column fix).
//!
//! A real figure of the books is a wrong answer when it is the wrong column. The live check of
//! 21-11-2026 answered "ab tak ki safi bikri kitni hui" with the GROSS sale 382,000 (the net sale, the
//! sum of the lot nets, was 332,780) and "safi bikri batao" with the buyers' bills 407,750. The words
//! "safi" (net), "bill" and "laga ke baghair" (without laga) were no cue of the app's own understanding:
//! each was an unknown word, or turned the question into a partner's total, so the model chose a column.
//!
//! This module reads those words, strictly, and only for a question that is otherwise a plain one
//! (a figure, the shop or one party, at most one period, no item, no other unknown word):
//!
//! | Words | Measure | Column |
//! |---|---|---|
//! | safi, saafi, صافی, "net" beside a sale | [`SaleMeasure::Net`] | R6 `net` (the shop), view B `net` (a seller) |
//! | bill, bills, بل, بلوں | [`SaleMeasure::Bills`] | R2 `buyer_total` (the shop, or one buyer) |
//! | laga ke baghair, "not counting laga", "sirf commission" | [`LagaView::Without`] | the commission alone |
//! | sirf laga, "only laga" | [`LagaView::Only`] | the Laga alone |
//! | commission aur laga alag alag | [`LagaView::Apart`] | the two, each by itself |
//!
//! A question this module does not recognise is left exactly as the rest of the understanding read it
//! (and goes to the model, whose answer the measure guard then checks against these same words).
//!
//! Example: "abhi tak ki safi bikri kitni hai" is `Sale` with measure `Net`, the shop, up to now; the answer
//! is the net total of R6 (332,780 on the dev books), never the gross.

use super::lexicon::{Cue, key};
use super::scan::Scan;
use super::who::Names;
use super::{
    Confidence, Doubt, FigureKind, Intent, LagaView, Measure, Outcome, PartyRole, PeriodRef,
    SaleMeasure, Understanding,
};

const NET_WORDS: &[&str] = &["safi", "saafi", "saaf", "safy", "صافی"];
const BILL_WORDS: &[&str] = &["bill", "bills", "bil", "bilon", "bilo", "بل", "بلوں", "بلز"];

/// Phrases that put the Laga outside the commission.
#[rustfmt::skip]
const NO_LAGA: &[&str] = &[
    "laga ke baghair", "laga ke bagair", "laga ke bghair", "laga ke bina", "laga ke bagher", "laga baghair",
    "baghair laga", "bagair laga", "bghair laga", "bina laga", "laga chor kar", "laga chhor kar",
    "laga nikal kar", "laga ke alawa", "laga ke ilawa", "without laga", "not counting laga",
    "excluding laga", "except laga", "no laga", "minus laga", "sirf commission", "commission only",
    "only commission", "just commission",
    "لاگا کے بغیر", "بغیر لاگا", "لاگا کے بنا", "لاگا چھوڑ کر", "لاگا کے علاوہ", "صرف کمیشن",
];

/// Phrases that ask the Laga alone.
#[rustfmt::skip]
const LAGA_ONLY: &[&str] = &[
    "sirf laga", "only laga", "laga only", "laga hi", "just laga", "laga alone", "sirf lagga",
    "صرف لاگا", "سرف لاگا", "لاگا ہی",
];

/// Phrases that ask the commission and the Laga each by itself.
#[rustfmt::skip]
const APART: &[&str] = &[
    "alag alag", "alag se", "separately", "separate", "الگ الگ", "الگ سے",
];

/// Words that belong to the measure and are explained by it, so they are no doubt: the phrases above and
/// the words that say the commission and the Laga are in the bill.
#[rustfmt::skip]
const MEASURE_PHRASES: &[&str] = &[
    "samet", "shamil", "mila kar", "milakar", "mila ke", "including", "included", "inclusive",
    "with commission", "ke sath", "ke saath", "سمیت", "شامل", "ملا کر", "ملا کے", "counting", "kamaya", "kamai",
    "earned", "earn", "raqm", "rakm", "raqme", "raqam", "rakam", "رقم", "total", "kul", "کل",
];

fn tokens(phrase: &str) -> Vec<String> {
    key(phrase)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Where the phrase's words stand in the question, as spans of word positions.
fn runs(scan: &Scan, phrase: &str) -> Vec<(usize, usize)> {
    let want = tokens(phrase);
    let n = want.len();
    if n == 0 || scan.words.len() < n {
        return Vec::new();
    }
    (0..=scan.words.len().saturating_sub(n))
        .filter(|&at| {
            scan.words
                .get(at..at.saturating_add(n))
                .is_some_and(|w| w == want.as_slice())
        })
        .map(|at| (at, at.saturating_add(n)))
        .collect()
}

fn says_any(scan: &Scan, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| !runs(scan, p).is_empty())
}

fn has_word(scan: &Scan, words: &[&str]) -> bool {
    words.iter().any(|w| {
        let k = key(w);
        scan.words.contains(&k)
    })
}

/// The word spans of the measure words and phrases: the words the rest of the understanding does not
/// know, and that this module explains.
pub(super) fn spans(scan: &Scan) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for list in [
        NO_LAGA,
        LAGA_ONLY,
        APART,
        MEASURE_PHRASES,
        NET_WORDS,
        BILL_WORDS,
    ] {
        for phrase in list {
            out.extend(runs(scan, phrase));
        }
    }
    out
}

/// True when a question asks how MANY bills ("kitne bill bane"), not how much they came to.
fn counts_bills(u: &Understanding) -> bool {
    u.asks_count || u.count_asks > 0
}

/// The figure kinds that are a plain sale or commission question and nothing else (a partner's total, a
/// profit and the like are the words "net", "kamaya" read as other figures).
fn plain_flow(kinds: &[FigureKind]) -> bool {
    kinds.iter().all(|k| {
        matches!(
            k,
            FigureKind::Sale
                | FigureKind::Commission
                | FigureKind::Laga
                | FigureKind::PartnerTotal
                | FigureKind::Profit
        )
    })
}

/// Makes the understanding the plain question of one figure (`kinds`), with no doubt left from the words
/// this module explained.
fn clean(u: &mut Understanding, kinds: Vec<FigureKind>) {
    u.figures.value = kinds;
    u.figures.confidence = Confidence::High;
    u.other_measure = Measure::Same;
    u.unexplained = false;
    u.two_figures = false;
    u.lower_ask = false;
    u.how_much_cues = u.how_much_cues.min(1);
    u.figure_at.truncate(1);
    u.doubts.retain(|d| {
        !matches!(
            d,
            Doubt::UnknownWords | Doubt::NoFigure | Doubt::PartnerLocked
        )
    });
    u.intent.value = Intent::Figure;
    u.intent.confidence = u.intent.confidence.max(Confidence::Medium);
    u.topic = None;
}

/// Reads the measure words of a question the rest of the understanding left plain, and sets
/// [`Understanding::sale_measure`] and [`Understanding::laga_view`].
///
/// `only_measure_words_unknown` is true when every word the app does not know is one of the measure words
/// ([`spans`]) or a filler of [`super::party_flow`].
pub(super) fn settle(scan: &Scan, u: &mut Understanding, only_measure_words_unknown: bool) {
    // A quantity, a breakdown or another measure already read from other words ("kitne nag", "mahine ke hisab
    // se") is not this module's to override.
    if matches!(u.other_measure, Measure::Quantity | Measure::Breakdown) {
        return;
    }
    if !only_measure_words_unknown
        || !u.items.is_empty()
        || !u.records.is_empty()
        || u.item_group.is_some()
        || u.periods.len() > 1
        || u.periods.first().is_some_and(|p| p.future)
        || u.compare
        || u.list.is_some()
        || u.scoped
        || u.quality
        || u.packing_filter
        || u.document_number
        || u.bikri_number.is_some()
        || u.truck.is_some()
        || u.pay_asked
        || u.guests
        || u.last_one
        || u.months_asked
        || u.oldest
        || u.happened
        || counts_bills(u)
        || !plain_flow(&u.figures.value)
        || u.doubts.iter().any(|d| {
            !matches!(
                d,
                Doubt::UnknownWords
                    | Doubt::NoFigure
                    | Doubt::NoParty
                    | Doubt::NoPeriod
                    | Doubt::PartnerLocked
            )
        })
    {
        return;
    }
    // An intent other than a figure is overridden only for a plain "how much" ("What is the total of all
    // buyer bills so far?" reads as a how-to): never for a request to write or to open a screen.
    let figure_like = match u.intent.value {
        Intent::Figure => true,
        Intent::HowTo | Intent::Unclear => {
            !u.wants_steps && (scan.has(Cue::HowMuch) || scan.has(Cue::Total))
        }
        _ => false,
    };
    if !figure_like {
        return;
    }
    // A measure word that stood where a name stands ("بلوں" of "ہارون کے بلوں کی کل رقم") is no party that
    // is missing from the books.
    let measure_spans = spans(scan);
    u.parties.retain(|p| {
        !(matches!(p.outcome, Outcome::NotFound)
            && measure_spans
                .iter()
                .any(|(s, e)| *s <= p.words.0 && p.words.1 <= *e))
    });
    // Who: the shop, or exactly one party found in his own role.
    let party = match u.parties.as_slice() {
        [] => None,
        [one] => match &one.outcome {
            Outcome::Found(hit) => Some(hit.role),
            _ => return,
        },
        _ => return,
    };
    let sale_word = scan.has(Cue::Sale);
    let net_word = has_word(scan, NET_WORDS) || (has_word(scan, &["net"]) && sale_word);
    let bill_word = has_word(scan, BILL_WORDS);
    // The net sale: of the shop, or of one seller.
    if net_word && sale_word && !bill_word && matches!(party, None | Some(PartyRole::Seller)) {
        // A "net" that is a partner's (his khalis, "net worth") has no sale word beside it.
        clean(u, vec![FigureKind::Sale]);
        u.sale_measure = Some(SaleMeasure::Net);
        return;
    }
    // The buyers' bills: of all buyers, or of one buyer.
    if bill_word && !net_word && matches!(party, None | Some(PartyRole::Buyer)) {
        clean(u, vec![FigureKind::Sale]);
        u.sale_measure = Some(SaleMeasure::Bills);
        return;
    }
    // Where the Laga stands in a commission question.
    let kinds = u.figures.value.clone();
    // A plain sale ("total bikri raqm batao"): only a word that means "amount" was unknown, so the question is
    // the plain gross sale, as "total bikri batao" is.
    if kinds == [FigureKind::Sale]
        && !net_word
        && !bill_word
        && party.is_none()
        && (u.unexplained || u.doubts.contains(&Doubt::UnknownWords))
    {
        clean(u, vec![FigureKind::Sale]);
        return;
    }
    let commission_like =
        kinds.contains(&FigureKind::Commission) || kinds.contains(&FigureKind::Laga);
    if !commission_like || net_word || bill_word {
        return;
    }
    // A party that is not a buyer pays no Laga: the shop's and the buyers' are the ones that read.
    if matches!(party, Some(PartyRole::Seller)) && !says_any(scan, NO_LAGA) {
        return;
    }
    if says_any(scan, LAGA_ONLY) {
        clean(u, vec![FigureKind::Laga]);
        u.laga_view = LagaView::Only;
    } else if says_any(scan, NO_LAGA) {
        clean(u, vec![FigureKind::Commission]);
        u.laga_view = LagaView::Without;
    } else if says_any(scan, APART)
        && kinds.contains(&FigureKind::Commission)
        && kinds.contains(&FigureKind::Laga)
    {
        clean(u, vec![FigureKind::Commission]);
        u.laga_view = LagaView::Apart;
    }
}

/// The unexplained-words check with the measure words left out: true when a word nothing explains is
/// still left (a noun that may be a filter). The names and periods explain their own words.
pub(super) fn unexplained_outside(
    scan: &Scan,
    names: &Names,
    periods: &[PeriodRef],
    skip: &dyn Fn(&str) -> bool,
) -> bool {
    let spans = spans(scan);
    super::what::unexplained_outside(scan, names, periods, skip, &spans)
}

/// What the words of a question say about the sale measure, for the guard of the model's answer
/// ([`crate::sale_guard`]): the measure, and whether the question also asks the column the guard would
/// otherwise refuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaleAsk {
    /// The net sale or the buyers' bills.
    pub measure: SaleMeasure,
    /// The gross is asked too ("khaam aur safi bikri"): another sale column is no fault.
    pub gross_too: bool,
    /// A balance or the dues are asked too ("mani ka bill aur bakaya").
    pub balance_too: bool,
    /// A commission or a Laga is asked beside the bills, not as a part of them.
    pub commission_too: bool,
}

/// The sale measure the words of a question name, read from the words alone (no books, no names), or `None`
/// for a question that names none, names both, or asks how MANY bills.
///
/// Example: "abhi tak ki safi bikri kitni hai" is `Net`; "mani ke bill ki kul raqam (commission aur laga
/// samet)" is `Bills` with no commission asked apart; "kitne bill bane" is `None`.
#[must_use]
pub fn asked_sale_measure(question: &str) -> Option<SaleAsk> {
    let scan = Scan::new(question, chrono::NaiveDate::default());
    let sale_word = scan.has(Cue::Sale);
    let net = has_word(&scan, NET_WORDS) || (has_word(&scan, &["net"]) && sale_word);
    let bill = has_word(&scan, BILL_WORDS) && !scan.has(Cue::SalePlural);
    let gross_word = has_word(&scan, &["khaam", "kham", "gross", "خام"]);
    if net && sale_word && !bill {
        return Some(SaleAsk {
            measure: SaleMeasure::Net,
            gross_too: gross_word,
            balance_too: false,
            commission_too: false,
        });
    }
    if bill && !net {
        let inclusive = says_any(
            &scan,
            &[
                "samet",
                "shamil",
                "mila kar",
                "milakar",
                "mila ke",
                "including",
                "included",
                "with",
                "ke sath",
                "ke saath",
                "سمیت",
                "شامل",
                "ملا کر",
                "ملا کے",
            ],
        );
        return Some(SaleAsk {
            measure: SaleMeasure::Bills,
            gross_too: gross_word,
            balance_too: scan.has_any(&[
                Cue::Balance,
                Cue::Dues,
                Cue::Pending,
                Cue::Payable,
                Cue::Advance,
            ]),
            commission_too: scan.has_any(&[Cue::Commission, Cue::Laga]) && !inclusive,
        });
    }
    None
}
