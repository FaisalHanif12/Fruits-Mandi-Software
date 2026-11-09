//! Which entry screen a request to write opens (plan 55 Section 7, lead ruling of 04-11-2026).
//!
//! The assistant never writes (PRD FR-7.72, rule 11 of CLAUDE.md). A request to write is refused in
//! plain words, and the panel offers the entry screen that does it, with the party chosen as a hint.
//! This file picks that screen from the words of the question, with no model call. It picks one
//! only when the question says it plainly; a request that fits two screens, or none, is left to the
//! model path.
//!
//! The amount is never taken from the question: the entry screen opens with the amount empty and
//! the Owner writes it himself (PRD FR-7.53, rule 11).
//!
//! Example: "akram ki 5000 wasooli likh do" -> the wasooli screen, Akram chosen, no amount.
//! "faisal ko 2000 de do" -> the adaigi screen (a payment to a buyer is cash lent, PRD FR-15.11).
//! "5000 jama karo" -> no screen: it could be a receipt, capital or a journal line.

use super::lexicon::{Cue, key};
use super::scan::Scan;
use super::what::screen_of as screen_of_topic;
use super::who::Names;
use super::{PartyHit, PartyRole};

/// Cues of a question that also asks to read something ("akram ka balance batao aur 5000 jama
/// karo"). The refusal would drop the question, so it goes to the model path, which answers the
/// question and carries the route too.
const ASKS_TO_READ: [Cue; 14] = [
    Cue::Balance,
    Cue::Pending,
    Cue::Show,
    Cue::Ledger,
    Cue::Statement,
    Cue::DaySummary,
    Cue::DayBook,
    Cue::TrialBalance,
    Cue::Many,
    Cue::Most,
    Cue::Least,
    Cue::Decision,
    Cue::Forecast,
    Cue::Print,
];

/// Screens that only show things. A request to write never opens them.
const READ_ONLY: [&str; 8] = [
    "dashboard",
    "cash_book",
    "today_sales",
    "unsold_stock",
    "collection_list",
    "dead_amount",
    "reports",
    "analytics",
];

/// The nouns that name an entry screen, and the screen. A request that says exactly one of them has
/// its screen. The role words (partner, garden, trip) are not here: they say who, not what.
const ENTRY_NOUNS: [(Cue, &str); 12] = [
    (Cue::Collection, "wasooli"),
    (Cue::Dues, "wasooli"),
    (Cue::Payable, "adaigi"),
    (Cue::Advance, "peshgi"),
    (Cue::Expense, "kharcha"),
    // ذاتی خرچہ and "li hui raqam" are types of the خرچہ screen.
    (Cue::PartnerTaken, "kharcha"),
    (Cue::MoneyLent, "udhar_raqam"),
    (Cue::RoleKhata, "alag_khata"),
    (Cue::Capital, "sarmaya"),
    (Cue::FruitTaken, "phal"),
    (Cue::Sale, "bikri"),
    (Cue::Journal, "general"),
];

/// Words that say "take it away" (delete, cancel, remove), in the normalised form of `Scan`.
const REMOVE_WORDS: [&str; 12] = [
    "delete", "remove", "hata", "hatao", "hatado", "mita", "mitao", "mitado", "ہٹا", "ہٹاؤ", "مٹا",
    "مٹاؤ",
];

/// Words for an installment of a garden's rent (قسط).
const INSTALLMENT_WORDS: [&str; 8] = [
    "qist",
    "qisat",
    "qisst",
    "kist",
    "installment",
    "instalment",
    "installments",
    "قسط",
];

/// Words for buying goods on a market trip (خریداری).
const PURCHASE_WORDS: [&str; 8] = [
    "khareedari",
    "kharidari",
    "khareedna",
    "purchase",
    "purchases",
    "buy",
    "buying",
    "خریداری",
];

/// Words that close a garden (باغ بند کریں).
const CLOSE_WORDS: [&str; 6] = ["band", "close", "closed", "khatam", "بند", "ختم"];

/// Words that cancel an entry.
const CANCEL_WORDS: [&str; 7] = ["cancel", "mansookh", "mansukh", "radd", "منسوخ", "ردّ", "رد"];

/// Words that correct an entry already saved.
const EDIT_WORDS: [&str; 12] = [
    "edit",
    "badlo",
    "badal",
    "badalna",
    "tabdeel",
    "theek",
    "correct",
    "change",
    "update",
    "ایڈٹ",
    "بدلو",
    "تبدیل",
];

/// Phrases for a sale in another shop (دوسری دکان میں فروخت). Joined with single spaces.
const OTHER_SHOP_PHRASES: [&str; 9] = [
    " doosri dukan ",
    " dusri dukan ",
    " doosri dukaan ",
    " dusri dukaan ",
    " dosri dukan ",
    " other shop ",
    " another shop ",
    " دوسری دکان ",
    " دوسرے آڑھتی ",
];

/// Phrases for a later sale of goods left over (باقی مال کی بکری). Joined with single spaces.
const LATER_SALE_PHRASES: [&str; 10] = [
    " baad mein ",
    " baad main ",
    " later sale ",
    " baqi maal ",
    " baki maal ",
    " leftover ",
    " bacha hua maal ",
    " بعد میں ",
    " باقی مال ",
    " بچا ہوا مال ",
];

/// Phrases that say "give": a payment out. Joined with single spaces.
const GIVE_PHRASES: [&str; 6] = [
    " de do ",
    " dedo ",
    " de dein ",
    " dijiye ",
    " دے دو ",
    " دیجیے ",
];

/// True when the question asks to read something besides the instruction to write.
pub(super) fn asks_to_read(scan: &Scan) -> bool {
    scan.has_any(&ASKS_TO_READ)
}

/// The accounts the question names, found for sure, with their roles: the only ones a route can
/// carry. A walk-in buyer (چکتا کھاتہ) is not an account to choose, and the search of names
/// never gives him as an account hit.
fn named(names: &Names) -> Vec<&PartyHit> {
    names
        .parties
        .iter()
        .filter_map(|p| p.outcome.hit())
        .collect()
}

fn has_role(hits: &[&PartyHit], roles: &[PartyRole]) -> bool {
    hits.iter().any(|h| roles.contains(&h.role))
}

/// True when the question says one of the words, as a whole word.
fn says_any(scan: &Scan, words: &[&str]) -> bool {
    let keys: Vec<String> = words.iter().map(|w| key(w)).collect();
    scan.words.iter().any(|w| keys.iter().any(|k| k == w))
}

/// True when the question says one of the phrases.
fn says_phrase(scan: &Scan, phrases: &[&str]) -> bool {
    let joined = format!(" {} ", scan.words.join(" "));
    phrases.iter().any(|p| joined.contains(&key(p)))
}

/// The entry screen of a request to write, by its name in the answer schema, or `None` when the
/// question does not say which one.
///
/// The nouns of the entry screens come first (wasooli, peshgi, kharcha ...). Then the topic of the
/// guide (a main screen such as the commission rules). When neither names a screen, the party and
/// the verb decide, in a few plain cases only.
pub(super) fn screen_of(
    scan: &Scan,
    names: &Names,
    topic: Option<&'static str>,
) -> Option<&'static str> {
    if let Some(screen) = trade_screen(scan, names) {
        return Some(screen);
    }
    let mut nouns: Vec<&'static str> = ENTRY_NOUNS
        .iter()
        .filter(|(cue, _)| scan.has_exact(*cue))
        .map(|(_, screen)| *screen)
        .collect();
    nouns.dedup();
    match nouns.as_slice() {
        [screen] => return Some(screen),
        // Two entry screens named: the question does not say which.
        [_, _, ..] => return None,
        [] => {}
    }
    if let Some(screen) = topic.and_then(screen_of_topic) {
        return (!READ_ONLY.contains(&screen)).then_some(screen);
    }
    let hits = named(names);
    // Money told as received ("akram ne 5000 diye"): a receipt, from a buyer or a seller returning
    // an advance (PRD FR-15.12).
    if scan.has(Cue::Received) && has_role(&hits, &[PartyRole::Buyer, PartyRole::Seller]) {
        return Some("wasooli");
    }
    // "akram ke 5000 jama karo": money in from a buyer.
    if says_any(scan, &["jama", "jamaa", "جمع"])
        && hits.len() == 1
        && has_role(&hits, &[PartyRole::Buyer])
    {
        return Some("wasooli");
    }
    // "x ko 2000 de do": a payment out. To a buyer it is cash lent (PRD FR-15.11).
    if says_phrase(scan, &GIVE_PHRASES) && hits.len() == 1 {
        if has_role(&hits, &[PartyRole::Seller, PartyRole::Buyer]) {
            return Some("adaigi");
        }
        if has_role(&hits, &[PartyRole::Staff]) {
            return Some("udhar_raqam");
        }
    }
    // "delete akram": the account form, where an account is made inactive.
    if says_any(scan, &REMOVE_WORDS)
        && hits.len() == 1
        && has_role(
            &hits,
            &[
                PartyRole::Seller,
                PartyRole::Buyer,
                PartyRole::Khata,
                PartyRole::Staff,
            ],
        )
    {
        return Some("account");
    }
    None
}

/// The entry of a garden or a trip, and a change to a بکری already saved (plan 55, lead order of
/// 05-11-2026). The garden or the trip comes first, then the kind of entry:
/// - garden: `garden_installment` (قسط), `garden_close` (بند), `garden_sale` (another shop),
///   `garden_cost` (خرچہ);
/// - trip: `trip_purchase` (خریداری), `trip_sale` (another shop), `trip_cost` (خرچہ);
/// - بکری: `bikri_cancel`, `bikri_edit`, `bikri_later_sale` (goods left in باقی مال sold later).
///
/// `None` when the question does not say which kind of entry; the nouns and the model decide then.
/// The party of a route is the garden or the trip when one is named; a بکری change carries none.
///
/// Example: "shahpur bagh ki qist likh do" is `garden_installment`; "bikri 5 cancel karo" is
/// `bikri_cancel`; "bagh ka kharcha likh do" is `garden_cost`, not the shop's خرچہ screen.
fn trade_screen(scan: &Scan, names: &Names) -> Option<&'static str> {
    let hits = named(names);
    let garden = has_role(&hits, &[PartyRole::Garden]) || scan.has_exact(Cue::RoleGarden);
    let trip = has_role(&hits, &[PartyRole::Trip]) || scan.has_exact(Cue::RoleTrip);
    let other_shop = says_phrase(scan, &OTHER_SHOP_PHRASES);
    let expense = scan.has_exact(Cue::Expense);
    if garden && !trip {
        if says_any(scan, &INSTALLMENT_WORDS) {
            return Some("garden_installment");
        }
        if says_any(scan, &CLOSE_WORDS) {
            return Some("garden_close");
        }
        if other_shop {
            return Some("garden_sale");
        }
        if expense {
            return Some("garden_cost");
        }
    }
    if trip && !garden {
        if says_any(scan, &PURCHASE_WORDS) {
            return Some("trip_purchase");
        }
        if other_shop {
            return Some("trip_sale");
        }
        if expense {
            return Some("trip_cost");
        }
    }
    if garden || trip || !scan.has_exact(Cue::Sale) {
        return None;
    }
    // A بکری already saved: only the بکری word beside the verb (no other entry noun).
    let others = ENTRY_NOUNS
        .iter()
        .any(|(cue, screen)| *screen != "bikri" && scan.has_exact(*cue));
    if others {
        return None;
    }
    if says_any(scan, &CANCEL_WORDS) {
        return Some("bikri_cancel");
    }
    if says_any(scan, &EDIT_WORDS) {
        return Some("bikri_edit");
    }
    if says_phrase(scan, &LATER_SALE_PHRASES) {
        return Some("bikri_later_sale");
    }
    None
}

/// True when a weak write word ("edit", "add", "change") stands beside the name of an entry that has
/// no amount: a بکری, a garden, a trip, an installment or a purchase. Such a request is something to
/// write, though no amount is told (the screens open empty).
///
/// Example: "bikri 5 edit karo" is a request; "commission change karo" is not (no entry).
pub(super) fn weak_write_of_an_entry(scan: &Scan) -> bool {
    scan.has_exact(Cue::Sale)
        || scan.has_exact(Cue::RoleGarden)
        || scan.has_exact(Cue::RoleTrip)
        || says_any(scan, &INSTALLMENT_WORDS)
        || says_any(scan, &PURCHASE_WORDS)
}
