//! Questions about the money a buyer paid at the time of his sale, and what he left on account (D8
//! Feature 10, H1, H2, H5, H6; PRD FR-12.44). The buyer bills R5 (ادھاری تفصیل) has one row for each buyer who
//! bought in the period, with his lines, his value, what he paid on the spot and what is left, and its totals
//! row holds the sums over the whole filter (the totals of R3 and R4 added over their lines), so one read of it
//! answers them. Its filters are the dates, the buyer, the seller, the item.
//!
//! The app gives NO instant line for such a question: "kitna jama karaya aur kitna baqi chhora" is two
//! figures that no plan line writes, and the plain balance of the buyer would be a real number for the
//! wrong thing. The plan only reads, for the model's first call (`plan_pre_reads`), and the model writes
//! the answer from what was read, with every figure a reference (PRD FR-7.75).
//!
//! The reads, by who the question names:
//! - one buyer (and a period): R5 for him, and view A for him when the question lists what he bought
//!   ("kya kya khareeda");
//! - one seller, with an item or not ("ehsan ke 200 peti aam"): R5 for the seller (and the item), a row for
//!   each buyer with his `paid` and `left`;
//! - nobody: R5 for the period, whose totals are `paid` and `left` ("naqd aur udhar bikri mein farq").
//!
//! Example: "is mahine anas ne mauqe par kitna jama karaya aur kitna baqi chhora": the `period` this
//! month, a `find` of Anas as a buyer, and R5 for him with those dates.

use super::{An, Plan, PlannedRead, Rep, analytics, dates, names_found, period_read, report};
use crate::understand::lexicon::Cue;
use crate::understand::scan::Scan;
use crate::understand::{Doubt, FigureKind, Intent, Outcome, PartyRole, Understanding};

/// The words that mean cash beside a verb of paying in.
const CASH_WORDS: [&str; 6] = ["naqdi", "naqd", "naqad", "cash", "نقدی", "نقد"];

/// The most rows of R5 the read carries (the totals row is over every buyer anyway).
const LINES: u32 = 100;

/// True when the question asks what a buyer paid on the spot or what he left on account, or asks the cash
/// and the credit sales together ("naqd aur udhar bikri mein kitna farq").
///
/// Example: "mauqe par kitna jama karaya" is true; "akram ne kitna jama karaya" (a receipt) is false.
pub(in crate::understand) fn paid_ask(scan: &Scan) -> bool {
    if scan.has_any(&[Cue::PaidSpot, Cue::LeftOn]) {
        return true;
    }
    let cash_beside_verb = scan.has(Cue::PaidVerb)
        && (scan.has(Cue::PayCash)
            || scan
                .words
                .iter()
                .any(|w| CASH_WORDS.iter().any(|c| super::key(c) == *w)));
    cash_beside_verb || super::totals::both_pay_kinds(scan)
}

/// Doubts that stop the reads: a name or a period the app is not sure of.
const NOT_CARRIED: [Doubt; 10] = [
    Doubt::SeveralMatches,
    Doubt::NameOrItem,
    Doubt::LookupFailed,
    Doubt::Cut,
    Doubt::TenseUnclear,
    Doubt::TimeWord,
    Doubt::MixedIntent,
    Doubt::NothingToRead,
    Doubt::SeveralPeriods,
    Doubt::PartnerLocked,
];

/// The reads to make before the first model call for a question about paid and left, or `None`.
pub(in crate::understand) fn pre_reads(u: &Understanding) -> Option<Vec<PlannedRead>> {
    if !u.paid_ask
        || !matches!(u.intent.value, Intent::Figure | Intent::List)
        || u.doubts.iter().any(|d| NOT_CARRIED.contains(d))
        || u.periods.len() > 1
        || u.periods.iter().any(|p| p.future)
        || u.parties.len() > 1
        || u.items.len() > 1
        || u.records.len() > 1
        || !names_found(u)
    {
        return None;
    }
    let period = u.periods.first();
    let (from, to) = dates(period);
    let item = match u.items.first().map(|i| &i.outcome) {
        Some(Outcome::Found(goods)) => Some(goods),
        Some(_) => return None,
        None => None,
    };
    let mut plan = Plan::default();
    let mut register = Rep {
        from: from.clone(),
        to: to.clone(),
        item: item.map(|goods| goods.key.clone()),
        rows: Some(LINES),
        ..Rep::default()
    };
    match u.parties.first().map(|p| (p, &p.outcome)) {
        Some((party, Outcome::Found(hit))) if hit.role == PartyRole::Buyer => {
            plan.find(&party.text, "buyer");
            register.party = Some(hit.key.clone());
            // What he bought, item by item: view A with him (a day is part of the same register).
            if u.intent.value == Intent::List || u.figures.value.contains(&FigureKind::Purchases) {
                plan.read(analytics(
                    "A",
                    An {
                        from,
                        to,
                        buyer: Some(hit.key.clone()),
                        ..An::default()
                    },
                ));
            }
        }
        Some((party, Outcome::Found(hit))) if hit.role == PartyRole::Seller => {
            plan.find(&party.text, "seller");
            register.seller = Some(hit.key.clone());
        }
        Some(_) => return None,
        None => {}
    }
    if let Some(goods) = item {
        plan.find(&goods.name_en, "item");
    }
    plan.read(report("R5", register));
    plan.dated = period.is_some();
    let mut all = Vec::new();
    if plan.dated
        && let Some(p) = period
    {
        all.push(period_read(p)?);
    }
    all.extend(plan.finds);
    all.extend(plan.reads);
    Some(all)
}
