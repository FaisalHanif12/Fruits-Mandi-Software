//! The names of a question: parties in their real roles, found through the search index of the
//! pickers, the partner list and the garden and trip lists (plan 55 step 1, PRD FR-7.40, FR-7.77).
//!
//! A name is found as typed, or as a close spelling, or through the transliteration words
//! (`kela` is کیلا). A hit is accepted only when every typed word is the same as, or close to, a word
//! of the record's name: the search index also returns near names ("کریم" finds اکرم), and a
//! made-up name must not be turned into a real person. Only reads (Code Standards B29).
//!
//! Example: "faisal ne kya becha" with Faisal a buyer is `FoundAs`: the verb asks for a seller.

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;

use mandi_services::core::accounts::{AccountHit, AccountScope, AccountType, SearchAccountsInput};
use mandi_services::core::bikri::{PukhtaPeopleInput, PukhtaSide};
use mandi_services::core::partners::PartnerRow;
use mandi_services::core::search::SearchScope;
use mandi_services::core::text;
use mandi_services::core::translit::Direction;
use mandi_services::{bikri, coa, partners, search, translit};

use super::lexicon::{Cue, table};
use super::scan::{Scan, edit_distance};
use super::{
    Confidence, Doubt, ItemRef, Outcome, PartyHit, PartyRef, PartyRole, PeriodKind, PeriodRef,
    Reader, RecordHit, RecordKind, RecordRef,
};

/// The most words of one name that are tried.
const MAX_GRAM: usize = 4;
/// The most words of a run of possible name words.
const MAX_RUN: usize = 6;
/// The longest word that is tried as a name.
const MAX_NAME_WORD: usize = 40;
/// The most hits asked of the search.
const ASK: u32 = 20;
/// The most hits kept in a `Several`.
const KEEP: usize = 5;
/// The code of the control of the الگ کھاتے.
const KHATA_CONTROL: &str = "0305";

/// What the names of a question came to.
pub(super) struct Names {
    pub parties: Vec<PartyRef>,
    pub items: Vec<ItemRef>,
    /// Expense heads, cities and addas.
    pub records: Vec<RecordRef>,
    /// The word spans that are names.
    pub spans: Vec<(usize, usize)>,
    pub doubts: Vec<Doubt>,
}

// ---- grading a typed word against the words of a name --------------------------------------------

/// True when a word has an Urdu letter.
pub(super) fn is_urdu(word: &str) -> bool {
    word.chars().any(text::is_urdu_letter)
}

/// The words of a normalised text.
pub(super) fn split_words(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// How well a typed word fits a word of a name: 3 the same, 2 close, 0 not. Short words must be the
/// same; a longer word may be one letter off (two from seven letters), or sound the same.
pub(super) fn word_grade(typed: &str, name: &str) -> u8 {
    if typed == name {
        return 3;
    }
    let (a, b) = (typed.chars().count(), name.chars().count());
    // Urdu words are short: a letter off in a four-letter word is another word (حامد, احمد).
    let shortest = if is_urdu(typed) { 5 } else { 4 };
    if a < shortest || b < shortest {
        return 0;
    }
    let cap = if a >= 7 { 2 } else { 1 };
    if edit_distance(typed, name, cap) <= cap {
        return 2;
    }
    if !is_urdu(typed) && !is_urdu(name) {
        let (p, q) = (text::search_phonetic(typed), text::search_phonetic(name));
        if p.chars().count() >= 3 && p == q {
            return 2;
        }
    }
    0
}

/// How the typed words fit the words of a name: the sum of the grades and whether every one was the
/// same. `None` when a typed word fits no word of the name.
pub(super) fn grade_words(typed: &[String], name: &[String]) -> Option<(u32, bool)> {
    let mut free: Vec<&String> = name.iter().collect();
    let mut total = 0u32;
    let mut exact = true;
    for word in typed {
        let (at, grade) = free
            .iter()
            .enumerate()
            .map(|(at, n)| (at, word_grade(word, n)))
            .max_by_key(|(_, g)| *g)?;
        if grade == 0 {
            return None;
        }
        exact &= grade == 3;
        total = total.saturating_add(u32::from(grade));
        free.remove(at);
    }
    Some((total, exact))
}

/// The score used to choose among hits: the grade first; a hit whose whole name was typed beats a
/// hit that has more words ("faisal" is Faisal, not Faisal Traders). Two hits that both have more
/// words than were typed are alike ("akram" is Akram Toll Tax and Akram Baloch).
pub(super) fn score(total: u32, typed_len: usize, name_len: usize) -> u32 {
    let whole = u32::from(name_len <= typed_len).saturating_mul(4);
    total.saturating_mul(8).saturating_add(whole)
}

/// A group of the best-fitting hits.
pub(super) struct Best<T> {
    pub hits: Vec<T>,
    pub exact: bool,
    /// True when other records fitted too, only less well.
    pub others: bool,
}

/// Keeps the hits with the best score.
pub(super) fn best_of<T>(scored: Vec<(T, u32, bool)>) -> Option<Best<T>> {
    let top = scored.iter().map(|(_, s, _)| *s).max()?;
    let others = scored.iter().any(|(_, s, _)| *s < top);
    let kept: Vec<(T, u32, bool)> = scored.into_iter().filter(|(_, s, _)| *s == top).collect();
    let exact = kept.iter().all(|(_, _, e)| *e);
    Some(Best {
        hits: kept.into_iter().map(|(h, _, _)| h).collect(),
        exact,
        others,
    })
}

// ---- the lookups --------------------------------------------------------------------------------------

/// The reads one question makes, with what they found kept for the next.
pub(super) struct Lookup<'a> {
    pub reader: &'a Reader<'a>,
    controls: OnceCell<Vec<(i64, String)>>,
    partner_rows: OnceCell<Vec<PartnerRow>>,
    /// The words of every account name, active or not.
    all_names: OnceCell<Vec<Vec<String>>>,
    pub failed: Cell<bool>,
    suggestions: RefCell<HashMap<String, Vec<String>>>,
}

impl<'a> Lookup<'a> {
    pub(super) fn new(reader: &'a Reader<'a>) -> Self {
        Self {
            reader,
            controls: OnceCell::new(),
            partner_rows: OnceCell::new(),
            all_names: OnceCell::new(),
            failed: Cell::new(false),
            suggestions: RefCell::new(HashMap::new()),
        }
    }

    fn control_code(&self, id: i64) -> Option<String> {
        let controls = self.controls.get_or_init(|| {
            coa::controls(self.reader.ctx)
                .map(|rows| rows.into_iter().map(|c| (c.id, c.code)).collect())
                .unwrap_or_else(|_| {
                    self.failed.set(true);
                    Vec::new()
                })
        });
        controls
            .iter()
            .find(|(i, _)| *i == id)
            .map(|(_, c)| c.clone())
    }

    /// The Urdu spellings of a Roman word, from the same word list and learned pairs as the typing
    /// help (best first, at most three).
    pub(super) fn urdu_of(&self, word: &str) -> Vec<String> {
        if let Some(found) = self.suggestions.borrow().get(word) {
            return found.clone();
        }
        let found: Vec<String> = translit::suggest(self.reader.ctx, word, Direction::ToUrdu)
            .map(|s| s.into_iter().map(|x| text::search_ur(&x.text)).collect())
            .unwrap_or_default();
        self.suggestions
            .borrow_mut()
            .insert(word.to_owned(), found.clone());
        found
    }

    fn accounts(&self, typed: &str) -> Vec<AccountHit> {
        let input = SearchAccountsInput {
            scope: AccountScope::Any,
            control_id: None,
            text: typed.to_owned(),
            limit: ASK,
            offset: 0,
            include_dead: Some(true),
        };
        search::search_accounts(self.reader.ctx, self.reader.actor, &input).unwrap_or_else(|_| {
            self.failed.set(true);
            Vec::new()
        })
    }

    pub(super) fn list(
        &self,
        scope: SearchScope,
        typed: &str,
    ) -> Vec<mandi_services::core::search::SearchHit> {
        search::search(self.reader.ctx, self.reader.actor, scope, typed, ASK).unwrap_or_else(|_| {
            self.failed.set(true);
            Vec::new()
        })
    }

    fn role_of(&self, hit: &AccountHit) -> Option<PartyRole> {
        match hit.account_type {
            AccountType::Seller => Some(PartyRole::Seller),
            AccountType::Buyer => Some(PartyRole::Buyer),
            AccountType::MoneyLent => Some(PartyRole::Staff),
            AccountType::Other => (self.control_code(hit.control_id).as_deref()
                == Some(KHATA_CONTROL))
            .then_some(PartyRole::Khata),
        }
    }

    fn party_hit(&self, hit: &AccountHit, role: PartyRole) -> PartyHit {
        PartyHit {
            key: format!("a{}", hit.id),
            id: hit.id,
            role,
            code: hit.code.clone(),
            name_ur: hit.name_ur.clone(),
            name_en: hit.name_en.clone(),
            tag: hit.tag.clone(),
            pukhta: hit.is_pukhta_party,
        }
    }

    /// The accounts whose name fits the typed words, best first.
    fn match_accounts(&self, typed: &[String]) -> Option<Best<PartyHit>> {
        let joined = typed.join(" ");
        let mut scored = self.score_accounts(&self.accounts(&joined), typed, false);
        // Nothing for a Roman Urdu spelling of an Urdu name ("faisal" for فیصل when the English name
        // is spelt another way): its Urdu spellings are tried, one word at a time.
        if scored.is_empty()
            && let [one] = typed
            && !is_urdu(one)
        {
            for urdu in self.urdu_of(one).into_iter().take(3) {
                let words = split_words(&urdu);
                scored = self.score_accounts(&self.accounts(&urdu), &words, true);
                if !scored.is_empty() {
                    break;
                }
            }
        }
        best_of(scored)
    }

    fn score_accounts(
        &self,
        hits: &[AccountHit],
        typed: &[String],
        urdu_only: bool,
    ) -> Vec<(PartyHit, u32, bool)> {
        let use_urdu = urdu_only || typed.iter().any(|w| is_urdu(w));
        hits.iter()
            .filter_map(|hit| {
                let role = self.role_of(hit)?;
                let mut name = if use_urdu {
                    split_words(&text::search_ur(&hit.name_ur))
                } else {
                    split_words(&text::search_en(&hit.name_en))
                };
                if !use_urdu && let Some(tag) = &hit.tag {
                    name.push(text::search_en(tag));
                }
                let (total, exact) = grade_words(typed, &name)?;
                // A spelling made from Roman letters is a guess: only the same word counts.
                if urdu_only && !exact {
                    return None;
                }
                Some((
                    self.party_hit(hit, role),
                    score(total, typed.len(), name.len()),
                    exact,
                ))
            })
            .collect()
    }

    /// A garden or a trip whose name fits the typed words (or whose short code is the number).
    fn match_venture(
        &self,
        scope: SearchScope,
        role: PartyRole,
        typed: &[String],
    ) -> Option<Best<PartyHit>> {
        let prefix = if role == PartyRole::Garden { "g" } else { "t" };
        let joined = typed.join(" ");
        let hits = self.list(scope, &joined);
        let numeric = typed.iter().all(|w| w.chars().all(|c| c.is_ascii_digit()));
        let use_urdu = typed.iter().any(|w| is_urdu(w));
        let scored: Vec<(PartyHit, u32, bool)> = hits
            .into_iter()
            .filter_map(|hit| {
                let name = if use_urdu {
                    split_words(&text::search_ur(&hit.name_ur))
                } else {
                    split_words(&text::search_en(&hit.name_en))
                };
                let (total, exact) = if numeric {
                    (3, true)
                } else {
                    grade_words(typed, &name)?
                };
                Some((
                    PartyHit {
                        key: format!("{prefix}{}", hit.id),
                        id: hit.id,
                        role,
                        code: hit.code.clone(),
                        name_ur: hit.name_ur.clone(),
                        name_en: hit.name_en.clone(),
                        tag: None,
                        pukhta: false,
                    },
                    score(total, typed.len(), name.len()),
                    exact,
                ))
            })
            .collect();
        best_of(scored)
    }

    /// True when the typed words are the name of an account the search does not list (an inactive
    /// one), or of a one-time person of the پختہ کھاتہ (who is no account at all). Such a name is
    /// not missing from the books, so the question is not answered as "no such party" without the
    /// model (PRD FR-13.5, FR-10.10). A failed read counts as known: the safe side.
    pub(super) fn known_elsewhere(&self, typed: &[String]) -> bool {
        let names = self.all_names.get_or_init(|| {
            search::account_names(self.reader.ctx)
                .unwrap_or_else(|_| {
                    self.failed.set(true);
                    Vec::new()
                })
                .into_iter()
                .map(|(ur, en, tag)| {
                    let mut words = split_words(&text::search_ur(&ur));
                    words.extend(split_words(&text::search_en(&en)));
                    words.extend(split_words(&text::search_en(&tag.unwrap_or_default())));
                    words
                })
                .collect()
        });
        if !typed.is_empty()
            && names
                .iter()
                .any(|words| typed.iter().all(|t| words.contains(t)))
        {
            return true;
        }
        let joined = typed.join(" ");
        [PukhtaSide::Buyer, PukhtaSide::Seller]
            .into_iter()
            .any(|side| {
                let input = PukhtaPeopleInput {
                    text: joined.clone(),
                    side,
                    include_dead: Some(true),
                };
                bikri::search_people(self.reader.ctx, &input).map_or_else(
                    |_| {
                        self.failed.set(true);
                        true
                    },
                    |people| !people.is_empty(),
                )
            })
    }

    fn partner_rows(&self) -> &[PartnerRow] {
        self.partner_rows.get_or_init(|| {
            partners::list(self.reader.ctx).unwrap_or_else(|_| {
                self.failed.set(true);
                Vec::new()
            })
        })
    }

    /// A partner whose name is the typed word (his English name, a word of it, or his Urdu name).
    /// A one-letter name (الف is "A") needs the word "partner" beside it.
    fn match_partner(&self, word: &str, cued: bool) -> Option<PartyHit> {
        if self.reader.partner_locked {
            return None;
        }
        let letters = word.chars().count();
        if letters < 2 && !cued {
            return None;
        }
        let urdu: Vec<String> = if is_urdu(word) {
            vec![word.to_owned()]
        } else {
            self.urdu_of(word)
        };
        self.partner_rows()
            .iter()
            .find(|row| {
                let en = split_words(&text::search_en(&row.name_en));
                let ur = split_words(&text::search_ur(&row.name_ur));
                let generic = |w: &String| w == "partner";
                en.iter()
                    .any(|w| !generic(w) && w == word && (cued || w.chars().count() >= 3))
                    || ur
                        .iter()
                        .any(|w| urdu.iter().any(|u| u == w) && (cued || w.chars().count() >= 2))
            })
            .map(|row| PartyHit {
                key: format!("s{}", row.id),
                id: row.id,
                role: PartyRole::Partner,
                code: String::new(),
                name_ur: row.name_ur.clone(),
                name_en: row.name_en.clone(),
                tag: None,
                pukhta: false,
            })
    }
}

// ---- the role a name was asked in --------------------------------------------------------------------

/// A role and whether the words of the question said it plainly (a noun, a verb) or only hinted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Wanted {
    pub role: PartyRole,
    pub hard: bool,
}

fn role_of_cue(cue: Cue) -> Option<PartyRole> {
    Some(match cue {
        Cue::RoleBuyer => PartyRole::Buyer,
        Cue::RoleSeller => PartyRole::Seller,
        Cue::RolePartner => PartyRole::Partner,
        Cue::RoleGarden => PartyRole::Garden,
        Cue::RoleTrip => PartyRole::Trip,
        Cue::RoleStaff => PartyRole::Staff,
        Cue::RoleKhata => PartyRole::Khata,
        _ => return None,
    })
}

const ROLE_CUES: [Cue; 7] = [
    Cue::RoleBuyer,
    Cue::RoleSeller,
    Cue::RolePartner,
    Cue::RoleGarden,
    Cue::RoleTrip,
    Cue::RoleStaff,
    Cue::RoleKhata,
];

/// The role the question asks the name at `span` in.
pub(super) fn wanted_role(scan: &Scan, span: (usize, usize)) -> Option<Wanted> {
    let hard = |role| Some(Wanted { role, hard: true });
    // A role noun beside the name, or inside it ("shahpur bagh").
    for hit in scan.hits.iter().filter(|h| !h.fuzzy) {
        let Some(role) = role_of_cue(hit.cue) else {
            continue;
        };
        let beside = hit.end == span.0 || hit.start == span.1;
        let inside = span.0 <= hit.start && hit.end <= span.1;
        if beside || inside {
            return hard(role);
        }
    }
    // One role noun in the whole question.
    let mut nouns: Vec<PartyRole> = ROLE_CUES
        .iter()
        .filter(|c| scan.has_exact(**c))
        .filter_map(|c| role_of_cue(*c))
        .collect();
    nouns.dedup();
    if let [only] = nouns.as_slice() {
        return hard(*only);
    }
    // A verb of trade: the one who sold is a seller, the one who bought a buyer, unless the name is
    // the other side of the verb (sold TO him, bought FROM him).
    let next = scan.word(span.1).unwrap_or_default();
    let before = span
        .0
        .checked_sub(1)
        .and_then(|b| scan.word(b))
        .unwrap_or_default();
    let to = matches!(next, "ko" | "کو") || before == "to";
    let from = matches!(next, "se" | "say" | "سے") || before == "from";
    match (scan.has_exact(Cue::Sold), scan.has_exact(Cue::Bought)) {
        (true, false) => {
            return hard(if to {
                PartyRole::Buyer
            } else {
                PartyRole::Seller
            });
        }
        (false, true) => {
            return hard(if from {
                PartyRole::Seller
            } else {
                PartyRole::Buyer
            });
        }
        _ => {}
    }
    // What was asked fits one side best: a hint only.
    let soft = |role| Some(Wanted { role, hard: false });
    if scan.has(Cue::Payable) || scan.has(Cue::Advance) {
        return soft(PartyRole::Seller);
    }
    if scan.has(Cue::Dues) && !scan.has(Cue::Payable) {
        return soft(PartyRole::Buyer);
    }
    if scan.has_any(&[Cue::Capital, Cue::PartnerTotal, Cue::PartnerShare]) {
        return soft(PartyRole::Partner);
    }
    if scan.has(Cue::MoneyLent) {
        return soft(PartyRole::Staff);
    }
    None
}

// ---- turning a match into a PartyRef -----------------------------------------------------------------

fn party_ref(
    scan: &Scan,
    span: (usize, usize),
    wanted: Option<Wanted>,
    best: Best<PartyHit>,
) -> PartyRef {
    let asked = wanted.filter(|w| w.hard).map(|w| w.role);
    let sure = if best.exact && !best.others {
        Confidence::High
    } else {
        Confidence::Medium
    };
    let text = scan.phrase(span.0, span.1);
    let of_role: Vec<&PartyHit> = wanted
        .map(|w| best.hits.iter().filter(|h| h.role == w.role).collect())
        .unwrap_or_default();
    let (outcome, confidence) = match (best.hits.as_slice(), wanted) {
        ([only], Some(w)) if only.role != w.role && w.hard => {
            (Outcome::FoundAs(only.clone()), sure)
        }
        ([only], _) => (Outcome::Found(only.clone()), sure),
        // Two accounts with this name, a buyer's and a seller's: only a plain word of the question
        // (a noun, a verb) settles which. A soft hint (adaigi, dues) does not: a payment to a buyer
        // is cash lent (PRD FR-15.11), so the other one may be meant.
        (_, Some(w)) if of_role.len() == 1 && w.hard => {
            let kept = of_role.first().map(|h| (*h).clone());
            match kept {
                Some(hit) => (Outcome::Found(hit), sure),
                None => (Outcome::NotFound, Confidence::Low),
            }
        }
        (many, _) => (
            Outcome::Several(many.iter().take(KEEP).cloned().collect()),
            Confidence::Low,
        ),
    };
    PartyRef {
        text,
        words: span,
        asked_role: asked,
        outcome,
        confidence,
    }
}

/// True when the question asks for something (a figure, a list, a printout), so a name that is not
/// in the books is a name that was asked about.
fn asks_something(scan: &Scan) -> bool {
    scan.has_any(&[
        Cue::HowMuch,
        Cue::Many,
        Cue::Print,
        Cue::Show,
        Cue::Balance,
        Cue::Sale,
        Cue::SalePlural,
        Cue::Commission,
        Cue::Laga,
        Cue::Dues,
        Cue::Collection,
        Cue::Money,
        Cue::Payable,
        Cue::Advance,
        Cue::Stock,
        Cue::Cash,
        Cue::Ledger,
        Cue::Statement,
        Cue::Bought,
        Cue::Sold,
        Cue::Rate,
        Cue::Profit,
        Cue::Expense,
        Cue::PartnerTotal,
        Cue::PartnerShare,
        Cue::Capital,
        Cue::Pending,
        Cue::Received,
        Cue::Goods,
        Cue::Open,
    ])
}

/// Particles that follow a name: ka, ki, ke, ko, ne, se, and the English "'s".
const AFTER_NAME: [&str; 12] = [
    "ka", "ki", "ke", "kay", "ko", "ne", "nay", "se", "say", "s", "کا", "کی",
];

/// Words that come before a name in English ("balance of zubair", "how much does zubair owe").
const BEFORE_NAME: [&str; 9] = [
    "of", "for", "from", "about", "does", "did", "has", "by", "کا",
];

/// Whether a run of unknown words is a name that was asked about and is not in the books. It is,
/// when the words stand where a name stands: before ka, ki, ke, ko, ne, se; after a role noun
/// ("buyer zubair"); after "of", "for", "does"; or before a verb of the trade or a balance word
/// ("zubair ne becha", "does zubair owe"). A word that stands anywhere else is a word the app does
/// not know, not a name: it is no reason to say a party does not exist.
fn not_found_confidence(scan: &Scan, span: (usize, usize)) -> Option<Confidence> {
    let words = scan.words.get(span.0..span.1)?;
    let letters: usize = words.iter().map(|w| w.chars().count()).sum();
    if letters < 3 || words.len() > 3 || !asks_something(scan) {
        return None;
    }
    if words.iter().all(|w| table().is_known(w)) {
        return None;
    }
    let followed = scan
        .word(span.1)
        .is_some_and(|w| AFTER_NAME.contains(&w) || matches!(w, "کے" | "کو" | "نے" | "سے"));
    let after_noun = scan
        .hits
        .iter()
        .any(|h| !h.fuzzy && role_of_cue(h.cue).is_some() && h.end == span.0);
    let after_of = span
        .0
        .checked_sub(1)
        .and_then(|b| scan.word(b))
        .is_some_and(|w| BEFORE_NAME.contains(&w));
    let before_verb = scan
        .hits
        .iter()
        .any(|h| !h.fuzzy && matches!(h.cue, Cue::Sold | Cue::Bought) && h.start == span.1);
    let before_owe = scan
        .word(span.1)
        .is_some_and(|w| matches!(w, "owe" | "owes" | "owed"));
    (followed || after_noun || after_of || before_verb || before_owe).then_some(Confidence::High)
}

/// The master lists a name can belong to besides parties and items. The first three are resolved
/// (an expense head to R29, a city to R21, an اڈا to R8); the others are only detected.
const OTHER_LISTS: [(SearchScope, Option<(RecordKind, &str)>); 6] = [
    (
        SearchScope::ExpenseHead,
        Some((RecordKind::ExpenseHead, "eh")),
    ),
    (SearchScope::City, Some((RecordKind::City, "ct"))),
    (SearchScope::Adda, Some((RecordKind::Adda, "ad"))),
    (SearchScope::Packing, None),
    (SearchScope::Marka, None),
    (SearchScope::Store, None),
];

/// What a name of another list came to.
enum Other {
    /// An expense head, a city or an اڈا.
    Resolved(Outcome<RecordHit>, Confidence),
    /// A packing, a مارکہ or a store: it is a record, the app does not use it.
    Detected,
}

/// The records of one list that fit the typed words. `exact_only` keeps only the same words.
fn record_in(
    look: &Lookup<'_>,
    scope: SearchScope,
    kind: (RecordKind, &str),
    words: &[String],
    exact_only: bool,
) -> Option<Best<RecordHit>> {
    let use_urdu = words.iter().any(|w| is_urdu(w));
    let scored: Vec<(RecordHit, u32, bool)> = look
        .list(scope, &words.join(" "))
        .into_iter()
        .filter_map(|hit| {
            let name = if use_urdu {
                split_words(&text::search_ur(&hit.name_ur))
            } else {
                split_words(&text::search_en(&hit.name_en))
            };
            let (total, exact) = grade_words(words, &name)?;
            if exact_only && !exact {
                return None;
            }
            Some((
                RecordHit {
                    kind: kind.0,
                    key: format!("{}{}", kind.1, hit.id),
                    id: hit.id,
                    code: hit.code,
                    name_ur: hit.name_ur,
                    name_en: hit.name_en,
                },
                score(total, words.len(), name.len()),
                exact,
            ))
        })
        .collect();
    best_of(scored)
}

/// True when the words fit a record of the list.
fn detected_in(look: &Lookup<'_>, scope: SearchScope, words: &[String], exact_only: bool) -> bool {
    let use_urdu = words.iter().any(|w| is_urdu(w));
    look.list(scope, &words.join(" ")).iter().any(|hit| {
        let name = if use_urdu {
            split_words(&text::search_ur(&hit.name_ur))
        } else {
            split_words(&text::search_en(&hit.name_en))
        };
        grade_words(words, &name).is_some_and(|(_, exact)| exact || !exact_only)
    })
}

/// The record of another list the words name, if they name one. Such a name is a real record, so it
/// must not be answered as "no such party".
fn other_record(look: &Lookup<'_>, typed: &[String]) -> Option<Other> {
    let urdu_typed = typed.iter().any(|w| is_urdu(w));
    let attempt = |words: &[String], exact_only: bool| -> Option<Other> {
        for (scope, kind) in OTHER_LISTS {
            match kind {
                Some(k) => {
                    if let Some(best) = record_in(look, scope, k, words, exact_only) {
                        let confidence = if best.exact && !best.others {
                            Confidence::High
                        } else {
                            Confidence::Medium
                        };
                        return Some(match best.hits.as_slice() {
                            [only] => Other::Resolved(Outcome::Found(only.clone()), confidence),
                            many => Other::Resolved(
                                Outcome::Several(many.iter().take(KEEP).cloned().collect()),
                                Confidence::Low,
                            ),
                        });
                    }
                }
                None => {
                    if detected_in(look, scope, words, exact_only) {
                        return Some(Other::Detected);
                    }
                }
            }
        }
        None
    };
    if let Some(found) = attempt(typed, false) {
        return Some(found);
    }
    // A Roman Urdu spelling of an Urdu name: only the same word counts.
    if let [one] = typed
        && !urdu_typed
    {
        for urdu in look.urdu_of(one).into_iter().take(3) {
            if let Some(found) = attempt(&split_words(&urdu), true) {
                return Some(found);
            }
        }
    }
    None
}

/// The names of a question.
pub(super) fn resolve(reader: &Reader<'_>, scan: &Scan, periods: &[PeriodRef]) -> Names {
    let look = Lookup::new(reader);
    let mut names = Names {
        parties: Vec::new(),
        items: Vec::new(),
        records: Vec::new(),
        spans: Vec::new(),
        doubts: Vec::new(),
    };
    let mut unknown: Vec<(usize, usize)> = Vec::new();
    let total = scan.words.len();
    let mut at = 0usize;
    while at < total {
        if !scan.can_start_name(at) {
            at = at.saturating_add(1);
            continue;
        }
        let mut end = at.saturating_add(1);
        while end < total && end.saturating_sub(at) < MAX_RUN && scan.can_start_name(end) {
            end = end.saturating_add(1);
        }
        let mut i = at;
        let mut open: Option<usize> = None;
        while i < end {
            let longest = end.saturating_sub(i).min(MAX_GRAM);
            let found = (1..=longest)
                .rev()
                .find_map(|len| name_at(&look, scan, (i, i.saturating_add(len)), &mut names));
            match found {
                Some(next) => {
                    if let Some(start) = open.take() {
                        unknown.push((start, i));
                    }
                    i = next;
                }
                None => {
                    open.get_or_insert(i);
                    i = i.saturating_add(1);
                }
            }
        }
        if let Some(start) = open {
            unknown.push((start, end));
        }
        at = end;
    }
    venture_and_partner_names(&look, scan, &mut unknown, &mut names);
    reconcile_partners(&look, scan, &mut names);
    trip_names(&look, scan, periods, &mut names);
    for span in unknown {
        let typed = scan.words.get(span.0..span.1).unwrap_or_default();
        let few = typed.len() <= 3 && typed.iter().map(|w| w.chars().count()).sum::<usize>() >= 3;
        // A record of another list (an expense head, a city, an اڈا, a packing ...) is no missing
        // party, with or without a word beside it that says so.
        match few.then(|| other_record(&look, typed)).flatten() {
            Some(Other::Resolved(outcome, confidence)) => {
                names.spans.push(span);
                names.records.push(RecordRef {
                    text: scan.phrase(span.0, span.1),
                    words: span,
                    outcome,
                    confidence,
                });
                continue;
            }
            Some(Other::Detected) => {
                names.doubts.push(Doubt::OtherRecord);
                continue;
            }
            None => {}
        }
        // A word for a place alone (Lahore) is no name of a party.
        let place_only = typed
            .iter()
            .all(|w| table().cues_of(w).contains(&Cue::ExternalPlace));
        // "No such party" is said without the model only when the name is in no account at all
        // (inactive ones too), is no known one-time person, and the partner section is open: while it
        // is locked a partner's name cannot be looked up (PRD FR-7.82).
        let missing = !place_only && !reader.partner_locked && !look.known_elsewhere(typed);
        match not_found_confidence(scan, span).filter(|_| missing) {
            Some(confidence) => {
                names.spans.push(span);
                names.parties.push(PartyRef {
                    text: scan.phrase(span.0, span.1),
                    words: span,
                    asked_role: wanted_role(scan, span).filter(|w| w.hard).map(|w| w.role),
                    outcome: Outcome::NotFound,
                    confidence,
                });
            }
            // Words of the table that are also parts of names (cash, khata) are not unknown words.
            None if typed.iter().all(|w| table().is_known(w)) => {}
            None => names.doubts.push(Doubt::UnknownWords),
        }
    }
    names.parties.sort_by_key(|p| p.words);
    names.items.sort_by_key(|i| i.words);
    names.records.sort_by_key(|r| r.words);
    if look.failed.get() {
        names.doubts.push(Doubt::LookupFailed);
    }
    let several = names
        .parties
        .iter()
        .any(|p| matches!(p.outcome, Outcome::Several(_)))
        || names
            .items
            .iter()
            .any(|i| matches!(i.outcome, Outcome::Several(_)))
        || names
            .records
            .iter()
            .any(|r| matches!(r.outcome, Outcome::Several(_)));
    if several {
        names.doubts.push(Doubt::SeveralMatches);
    }
    names
}

/// Tries the words of `span` as a party, then as an item. On success the name is recorded and the
/// index after it is returned.
fn name_at(
    look: &Lookup<'_>,
    scan: &Scan,
    span: (usize, usize),
    names: &mut Names,
) -> Option<usize> {
    let typed = scan.words.get(span.0..span.1)?;
    let t = table();
    // Only function words and words of the table: no name (a name has at least one word of its own).
    // A word that is only the usual part of a firm's name ("mandi", "fruit") is no name alone.
    if typed.iter().all(|w| t.is_known(w) || t.is_nameish(w)) {
        return None;
    }
    let long = typed.len() > 2;
    // A person needs three letters; an item may have two (آم).
    let party_ok = typed.iter().any(|w| w.chars().count() >= 3);
    let item_ok = typed.iter().any(|w| w.chars().count() >= 2);
    if !item_ok {
        return None;
    }
    // A word of that length is no name; it would only be refused by the search.
    if typed.iter().any(|w| w.chars().count() > MAX_NAME_WORD) {
        return None;
    }
    let wanted = wanted_role(scan, span);
    let party = party_ok
        .then(|| look.match_accounts(typed))
        .flatten()
        .map(|best| party_ref(scan, span, wanted, best));
    let item = if long {
        None
    } else {
        super::items::resolve(look, scan, span)
    };
    match (party, item) {
        (Some(p), Some(i)) if !matches!(i.outcome, Outcome::NotFound) => {
            // A word that is a person and an item at once.
            names.doubts.push(Doubt::NameOrItem);
            names.spans.push(span);
            names.parties.push(PartyRef {
                confidence: Confidence::Low,
                ..p
            });
            names.items.push(ItemRef {
                confidence: Confidence::Low,
                ..i
            });
        }
        (Some(p), _) => {
            names.spans.push(span);
            names.parties.push(p);
        }
        (None, Some(i)) => {
            names.spans.push(span);
            names.items.push(i);
        }
        (None, None) => return None,
    }
    Some(span.1)
}

/// Gardens, trips and partners, from the words no account or item took.
fn venture_and_partner_names(
    look: &Lookup<'_>,
    scan: &Scan,
    unknown: &mut Vec<(usize, usize)>,
    names: &mut Names,
) {
    let partner_cue = scan.has_exact(Cue::RolePartner);
    let partner_figure = scan.has_any(&[Cue::PartnerTotal, Cue::PartnerShare, Cue::Capital]);
    if look.reader.partner_locked && (partner_cue || partner_figure) {
        names.doubts.push(Doubt::PartnerLocked);
    }
    // A partner is named after the word "partner" even when his name is one letter (A, الف).
    for hit in scan
        .hits
        .iter()
        .filter(|h| h.cue == Cue::RolePartner && !h.fuzzy)
    {
        let candidates = [Some(hit.end), hit.start.checked_sub(1)];
        for at in candidates.into_iter().flatten() {
            let Some(word) = scan.word(at) else { continue };
            if names.spans.iter().any(|(s, e)| *s <= at && at < *e)
                || (table().is_stop(word) && word.chars().count() > 1)
            {
                continue;
            }
            if let Some(found) = look.match_partner(word, true) {
                let span = (at, at.saturating_add(1));
                names.spans.push(span);
                names.parties.push(PartyRef {
                    text: scan.phrase(span.0, span.1),
                    words: span,
                    asked_role: Some(PartyRole::Partner),
                    outcome: Outcome::Found(found),
                    confidence: Confidence::High,
                });
                unknown.retain(|(s, e)| !(*s <= at && at < *e));
                break;
            }
        }
    }
    let garden_cue = scan.has_exact(Cue::RoleGarden);
    let trip_cue = scan.has_exact(Cue::RoleTrip);
    let mut rest: Vec<(usize, usize)> = Vec::new();
    for span in unknown.drain(..) {
        let typed: Vec<String> = scan
            .words
            .get(span.0..span.1)
            .map(<[String]>::to_vec)
            .unwrap_or_default();
        let plain: Vec<String> = typed
            .iter()
            .filter(|w| {
                !matches!(
                    w.as_str(),
                    "bagh" | "bag" | "garden" | "safar" | "trip" | "باغ" | "سفر"
                )
            })
            .cloned()
            .collect();
        let found = single_word_partner(look, &plain, partner_cue || partner_figure)
            .or_else(|| {
                (!plain.is_empty())
                    .then(|| look.match_venture(SearchScope::GardenAll, PartyRole::Garden, &plain))
                    .flatten()
            })
            .or_else(|| {
                (trip_cue && !plain.is_empty())
                    .then(|| look.match_venture(SearchScope::TripAll, PartyRole::Trip, &plain))
                    .flatten()
            });
        match found {
            Some(best) => {
                let wanted = wanted_role(scan, span);
                names.spans.push(span);
                let mut reference = party_ref(scan, span, wanted, best);
                if garden_cue || trip_cue {
                    reference.confidence = reference.confidence.max(Confidence::Medium);
                }
                names.parties.push(reference);
            }
            None => rest.push(span),
        }
    }
    *unknown = rest;
}

/// A market trip. The Owner knows it by its number ("safar 1") or by its day ("14 august ka
/// safar"); the books name it by its code and its day ("0070 14-08-2026"). A day finds it; a number
/// alone may fit more than one trip, and then the app is not sure.
fn trip_names(look: &Lookup<'_>, scan: &Scan, periods: &[PeriodRef], names: &mut Names) {
    for hit in scan
        .hits
        .iter()
        .filter(|h| h.cue == Cue::RoleTrip && !h.fuzzy)
    {
        let beside = [Some(hit.end), hit.start.checked_sub(1)];
        let number = beside
            .into_iter()
            .flatten()
            .find_map(|at| scan.digits_at(at).map(|n| (at, n)));
        let days: Vec<String> = periods
            .iter()
            .filter(|p| p.kind == PeriodKind::Date)
            .map(|p| p.to.format("%d-%m-%Y").to_string())
            .collect();
        if number.is_none() && days.is_empty() {
            continue;
        }
        let typed = number.map(|(_, n)| n.to_string()).unwrap_or_default();
        let found: Vec<_> = look
            .list(SearchScope::TripAll, &typed)
            .into_iter()
            .filter(|h| days.is_empty() || days.iter().any(|d| h.name_en.contains(d.as_str())))
            .map(|h| PartyHit {
                key: format!("t{}", h.id),
                id: h.id,
                role: PartyRole::Trip,
                code: h.code,
                name_ur: h.name_ur,
                name_en: h.name_en,
                tag: None,
                pukhta: false,
            })
            .collect();
        if found.is_empty() {
            continue;
        }
        let start = number.map_or(hit.start, |(at, _)| at.min(hit.start));
        let end = number.map_or(hit.end, |(at, _)| at.saturating_add(1).max(hit.end));
        let best = Best {
            hits: found,
            exact: !days.is_empty(),
            others: false,
        };
        names.spans.push((start, end));
        names.parties.push(party_ref(
            scan,
            (start, end),
            wanted_role(scan, (start, end)),
            best,
        ));
    }
}

/// A name that is a buyer's or a seller's and a partner's too ("usman": the buyer Usman Ghani and
/// the partner Usman). When the question asks for what only a partner has (his capital, his share,
/// the word "partner"), it is the partner; when it asks as a buyer or a seller, it is the account;
/// when it does not say, both are listed and the app is not sure.
fn reconcile_partners(look: &Lookup<'_>, scan: &Scan, names: &mut Names) {
    if look.reader.partner_locked {
        return;
    }
    let cued = scan.has_exact(Cue::RolePartner);
    for party in &mut names.parties {
        let (start, end) = party.words;
        if end.saturating_sub(start) != 1 {
            continue;
        }
        let Some(word) = scan.word(start) else {
            continue;
        };
        let accounts: Vec<PartyHit> = match &party.outcome {
            Outcome::Found(h) | Outcome::FoundAs(h) if h.role != PartyRole::Partner => {
                vec![h.clone()]
            }
            Outcome::Several(v) if v.iter().all(|h| h.role != PartyRole::Partner) => v.clone(),
            _ => continue,
        };
        let Some(partner) = look.match_partner(word, cued) else {
            continue;
        };
        match wanted_role(scan, party.words) {
            Some(w) if w.role == PartyRole::Partner => {
                party.outcome = Outcome::Found(partner);
                party.asked_role = w.hard.then_some(PartyRole::Partner);
                party.confidence = if w.hard {
                    Confidence::High
                } else {
                    Confidence::Medium
                };
            }
            Some(w) if w.hard => {}
            _ => {
                let mut all = accounts;
                all.push(partner);
                party.outcome = Outcome::Several(all.into_iter().take(KEEP).collect());
                party.confidence = Confidence::Low;
            }
        }
    }
}

/// A partner named by his name alone ("alif ka khalis", "الف کا حصہ").
fn single_word_partner(look: &Lookup<'_>, plain: &[String], cued: bool) -> Option<Best<PartyHit>> {
    let [word] = plain else { return None };
    let hit = look.match_partner(word, cued)?;
    Some(Best {
        hits: vec![hit],
        exact: true,
        others: false,
    })
}
