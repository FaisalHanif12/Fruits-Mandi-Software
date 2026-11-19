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
use mandi_services::core::chukta::ChuktaSearchInput;
use mandi_services::core::partners::PartnerRow;
use mandi_services::core::search::SearchScope;
use mandi_services::core::text;
use mandi_services::core::translit::Direction;
use mandi_services::{chukta, coa, partners, search, translit};

use super::lexicon::{Cue, Hit, table};
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
    // The same word with a double letter written single ("tol" for "toll", "mohamad" for
    // "mohammad"): a spelling of the name, not another word, even for a short word.
    if !is_urdu(typed) && undouble(typed) == undouble(name) && undouble(typed).chars().count() >= 3
    {
        return 2;
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

/// A word with every doubled letter written once.
fn undouble(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for c in word.chars() {
        if !out.ends_with(c) {
            out.push(c);
        }
    }
    out
}

/// True when two words differ only in their vowels and doubled letters ("akram", "akrm", "akrum";
/// "muhammad", "mohammed"). Such a spelling is the same name written another way. One consonant
/// changed ("nasir", "nasim"; "nazir", "nasir") is another name: it is a close word, not a clear one.
pub(super) fn sounds_alike(typed: &str, name: &str) -> bool {
    if is_urdu(typed) || is_urdu(name) {
        return is_urdu(typed)
            && is_urdu(name)
            && urdu_skeleton(typed).chars().count() >= 2
            && urdu_skeleton(typed) == urdu_skeleton(name);
    }
    if undouble(typed) == undouble(name) && undouble(typed).chars().count() >= 3 {
        return true;
    }
    let (p, q) = (text::search_phonetic(typed), text::search_phonetic(name));
    p.chars().count() >= 3 && p == q
}

/// An Urdu word without its vowel letters and doubled letters, its first letter always kept.
fn urdu_skeleton(word: &str) -> String {
    let mut out = String::new();
    for (at, c) in word.chars().enumerate() {
        let vowel = matches!(c, 'ا' | 'و' | 'ی' | 'ے' | 'آ' | 'ء' | 'ي' | 'ى');
        if (at == 0 || !vowel) && !out.ends_with(c) {
            out.push(c);
        }
    }
    out
}

/// True when a word of the question is near a word of a name: one letter off (two from seven
/// letters). A name that is near is not "missing from the books", whatever the matcher decided
/// (PRD FR-7.40). It runs over every account name, so the cheap tests come first: the lengths, then
/// the ends (one edit cannot change both the first and the last letter of a word of three or more).
fn near_word(typed: &str, name: &str) -> bool {
    if typed == name {
        return true;
    }
    let (a, b) = (typed.chars().count(), name.chars().count());
    if a < 3 || b < 3 {
        return false;
    }
    let cap = if a >= 7 { 2 } else { 1 };
    if a.abs_diff(b) > cap {
        return false;
    }
    if cap == 1
        && typed.chars().next() != name.chars().next()
        && typed.chars().next_back() != name.chars().next_back()
    {
        return false;
    }
    edit_distance(typed, name, cap) <= cap
}

/// How the typed words fit the words of a name.
pub(super) struct Fit {
    /// The sum of the grades.
    pub total: u32,
    /// Every typed word was the same as a word of the name.
    pub exact: bool,
    /// Every typed word was the same, or the same name written another way ([`sounds_alike`]). A
    /// word that is only one letter off is not clear: it may be another person.
    pub clear: bool,
}

/// How the typed words fit the words of a name. `None` when a typed word fits no word of the name.
pub(super) fn grade_fit(typed: &[String], name: &[String]) -> Option<Fit> {
    let mut free: Vec<&String> = name.iter().collect();
    let mut total = 0u32;
    let mut exact = true;
    let mut clear = true;
    for word in typed {
        let (at, grade, same_sound) = free
            .iter()
            .enumerate()
            .map(|(at, n)| (at, word_grade(word, n), sounds_alike(word, n)))
            .max_by_key(|(_, g, c)| (*g, *c))?;
        if grade == 0 {
            return None;
        }
        exact &= grade == 3;
        clear &= grade == 3 || same_sound;
        total = total.saturating_add(u32::from(grade));
        free.remove(at);
    }
    Some(Fit {
        total,
        exact,
        clear,
    })
}

/// How the typed words fit the words of a name: the sum of the grades and whether every one was the
/// same. `None` when a typed word fits no word of the name.
pub(super) fn grade_words(typed: &[String], name: &[String]) -> Option<(u32, bool)> {
    grade_fit(typed, name).map(|fit| (fit.total, fit.exact))
}

/// The score used to choose among hits: the grade first, then a small bonus for a hit whose whole name
/// was typed. Records and items use the bonus ("kela" is Kela, not Kela Desi). ACCOUNTS do not: they
/// pass a length that is never whole, so a typed name that is the whole name of one account and the
/// first word of another ("faisal" with Faisal and Faisal Traders) ties and the app asks which one,
/// as PRD FR-7.77 says for two to five matches (D7 R5). Two hits that both have more words than were
/// typed are alike ("akram" is Akram Toll Tax and Akram Baloch).
pub(super) fn score(total: u32, typed_len: usize, name_len: usize) -> u32 {
    let whole = u32::from(name_len <= typed_len).saturating_mul(4);
    total.saturating_mul(8).saturating_add(whole)
}

/// A group of the best-fitting hits.
pub(super) struct Best<T> {
    pub hits: Vec<T>,
    pub exact: bool,
    /// True when every hit fitted clearly: the same words, or the same name in another spelling.
    pub clear: bool,
    /// True when other records fitted too, only less well.
    pub others: bool,
}

/// Keeps the hits with the best score. For items and records, whose close spellings are taken as
/// they come: every hit counts as clear.
pub(super) fn best_of<T>(scored: Vec<(T, u32, bool)>) -> Option<Best<T>> {
    best_of_clear(
        scored
            .into_iter()
            .map(|(h, s, e)| (h, s, e, true))
            .collect(),
    )
}

/// Keeps the hits with the best score, with whether each fitted clearly.
pub(super) fn best_of_clear<T>(scored: Vec<(T, u32, bool, bool)>) -> Option<Best<T>> {
    let top = scored.iter().map(|(_, s, _, _)| *s).max()?;
    let others = scored.iter().any(|(_, s, _, _)| *s < top);
    let kept: Vec<(T, u32, bool, bool)> = scored
        .into_iter()
        .filter(|(_, s, _, _)| *s == top)
        .collect();
    let exact = kept.iter().all(|(_, _, e, _)| *e);
    let clear = kept.iter().all(|(_, _, _, c)| *c);
    Some(Best {
        hits: kept.into_iter().map(|(h, _, _, _)| h).collect(),
        exact,
        clear,
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
            with_chukta: None,
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
        best_of_clear(scored)
    }

    fn score_accounts(
        &self,
        hits: &[AccountHit],
        typed: &[String],
        urdu_only: bool,
    ) -> Vec<(PartyHit, u32, bool, bool)> {
        let use_urdu = urdu_only || typed.iter().any(|w| is_urdu(w));
        hits.iter()
            .filter_map(|hit| {
                let role = self.role_of(hit)?;
                let mut name = if use_urdu {
                    split_words(&text::search_ur(&hit.name_ur))
                } else {
                    split_words(&text::search_en(&hit.name_en))
                };
                // The tag tells two accounts of one name apart in both scripts: "اکرم" is اکرم ٹول ٹیکس and
                // اکرم K, so the app asks which one, as it does for "akram" (PRD FR-7.77, D7 R5).
                if let Some(tag) = &hit.tag {
                    name.push(text::search_en(tag));
                }
                let fit = grade_fit(typed, &name)?;
                // A spelling made from Roman letters is a guess: only the same word counts.
                if urdu_only && !fit.exact {
                    return None;
                }
                Some((
                    self.party_hit(hit, role),
                    // No account gets the whole-name bonus: a typed name that is the whole name of one
                    // account and the first word of another ("faisal" with Faisal and Faisal Traders), or
                    // that two accounts share and only the tag tells apart (اسلم and اسلم BA), is not
                    // taken for the shorter one. The accounts tie, so the app asks which one (PRD
                    // FR-7.77, FR-10.7, E38; D7 R5, lead ruling). The bonus only broke ties between equal
                    // grades, so a better fit still wins.
                    score(fit.total, typed.len(), typed.len().saturating_add(1)),
                    fit.exact,
                    fit.clear,
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
        let scored: Vec<(PartyHit, u32, bool, bool)> = hits
            .into_iter()
            .filter_map(|hit| {
                let name = if use_urdu {
                    split_words(&text::search_ur(&hit.name_ur))
                } else {
                    split_words(&text::search_en(&hit.name_en))
                };
                let fit = if numeric {
                    Fit {
                        total: 3,
                        exact: true,
                        clear: true,
                    }
                } else {
                    grade_fit(typed, &name)?
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
                    },
                    score(fit.total, typed.len(), name.len()),
                    fit.exact,
                    fit.clear,
                ))
            })
            .collect();
        best_of_clear(scored)
    }

    /// True when the typed words are the name of an account the search does not list (an inactive
    /// one), or of a walk-in buyer of the چکتا کھاتہ (who is no account at all and so is never an
    /// account hit, PRD FR-13.6). Such a name is not missing from the books, so the question is not
    /// answered as "no such party" without the model (PRD FR-13.2, FR-10.10). A failed read counts
    /// as known: the safe side.
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
        // A word that is only near a word of a name counts too ("اکرام" for اکرم): such a name is not
        // missing from the books, the model looks further (PRD FR-7.40).
        if !typed.is_empty()
            && names
                .iter()
                .any(|words| typed.iter().all(|t| words.iter().any(|w| near_word(t, w))))
        {
            return true;
        }
        let input = ChuktaSearchInput {
            text: typed.join(" "),
        };
        chukta::search(self.reader.ctx, &input).map_or_else(
            |_| {
                self.failed.set(true);
                true
            },
            |people| !people.is_empty(),
        )
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

/// True when a seller or buyer word stands right before a commission or a laga word, with at most "ka /
/// ki / ke" between: it says which side's charge is asked, not who the name is.
pub(super) fn is_side_word(scan: &Scan, role: &Hit) -> bool {
    if !matches!(role.cue, Cue::RoleSeller | Cue::RoleBuyer) {
        return false;
    }
    scan.hits.iter().any(|charge| {
        matches!(charge.cue, Cue::Commission | Cue::Laga)
            && (charge.start == role.end
                || (charge.start == role.end.saturating_add(1)
                    && scan
                        .word(role.end)
                        .is_some_and(|w| matches!(w, "ka" | "ki" | "ke" | "کا" | "کی" | "کے"))))
    })
}

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
    // One role noun in the whole question. A role word that names the side of a commission or a laga
    // ("seller commission", "بیوپاری کا کمیشن") is no role of the name ("seller commission on the banana
    // Faisal bought": Faisal is a buyer).
    let mut nouns: Vec<PartyRole> = scan
        .hits
        .iter()
        .filter(|h| !h.fuzzy && ROLE_CUES.contains(&h.cue) && !is_side_word(scan, h))
        .filter_map(|h| role_of_cue(h.cue))
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
    // "Alif ne zaati kharche ke liye kitna liya": "kitna liya" is "how much did he take", not a
    // purchase. The money a partner took is asked of a partner.
    let taken = scan.has(Cue::PartnerTaken);
    match (
        scan.has_exact(Cue::Sold) && !taken,
        scan.has_exact(Cue::Bought) && !taken,
    ) {
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
    if scan.has_any(&[
        Cue::Capital,
        Cue::PartnerTotal,
        Cue::PartnerShare,
        Cue::PartnerTaken,
    ]) {
        return soft(PartyRole::Partner);
    }
    if scan.has(Cue::MoneyLent) {
        return soft(PartyRole::Staff);
    }
    None
}

// ---- turning a match into a PartyRef -----------------------------------------------------------------

/// True when every hit has the very same Urdu and English name (the tag aside).
fn same_name(hits: &[PartyHit]) -> bool {
    hits.windows(2).all(|pair| match pair {
        [a, b] => a.name_ur == b.name_ur && a.name_en == b.name_en,
        _ => true,
    })
}

fn party_ref(
    scan: &Scan,
    span: (usize, usize),
    wanted: Option<Wanted>,
    best: Best<PartyHit>,
) -> PartyRef {
    let asked = wanted.filter(|w| w.hard).map(|w| w.role);
    // A name that is only one letter off a name of the books may be another person ("nasir" for
    // Nasim): the app is not sure of him (money review R-a).
    let sure = if !best.clear {
        Confidence::Low
    } else if best.exact && !best.others {
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
        // Only when every tied account has the same name (a seller and a buyer of one person, E10):
        // a tie of different names ("faisal" with a seller Faisal and a buyer Faisal Traders) is not
        // settled by a role word, the app asks (PRD FR-7.77; money review 15, M2).
        (_, Some(w)) if of_role.len() == 1 && w.hard && same_name(&best.hits) => {
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
        Cue::Define,
        Cue::Total,
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
/// (an expense head to R29, a city to R21); the others are only detected, an اڈا too (R8 is retired in
/// v1.34).
const OTHER_LISTS: [(SearchScope, Option<(RecordKind, &str)>); 6] = [
    (
        SearchScope::ExpenseHead,
        Some((RecordKind::ExpenseHead, "eh")),
    ),
    (SearchScope::City, Some((RecordKind::City, "ct"))),
    (SearchScope::Adda, None),
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

/// The staff member the unknown words of a question name, if they name one: a collector, a munshi or
/// a labourer who has no account of his own. The words are tried from the longest run to a single
/// word. A word that is also one letter off a common word must be the same as the name, not close.
///
/// Example: "Rasheed ne kitna collection kiya" names the collector Rasheed (`st3`); "kitna" names
/// no one.
fn staff_record(look: &Lookup<'_>, scan: &Scan, span: (usize, usize)) -> Option<RecordRef> {
    let width = span.1.saturating_sub(span.0);
    if width == 0 || width > 3 {
        return None;
    }
    for len in (1..=width).rev() {
        for start in span.0..=span.1.saturating_sub(len) {
            let end = start.saturating_add(len);
            let words = scan.words.get(start..end)?;
            let letters: usize = words.iter().map(|w| w.chars().count()).sum();
            if letters < 3 || words.iter().all(|w| table().is_known(w)) {
                continue;
            }
            let exact_only = words.iter().any(|w| table().is_near_known(w));
            let Some(best) = record_in(
                look,
                SearchScope::Staff,
                (RecordKind::Staff, "st"),
                words,
                exact_only,
            ) else {
                continue;
            };
            let (outcome, confidence) = match best.hits.as_slice() {
                [only] => (
                    Outcome::Found(only.clone()),
                    if best.exact && !best.others {
                        Confidence::High
                    } else {
                        Confidence::Medium
                    },
                ),
                many => (
                    Outcome::Several(many.iter().take(KEEP).cloned().collect()),
                    Confidence::Low,
                ),
            };
            return Some(RecordRef {
                text: scan.phrase(start, end),
                words: (start, end),
                outcome,
                confidence,
            });
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
        // A staff member with no account of his own (a collector: "Rasheed ne kitna collection kiya")
        // is a real record. It is looked for before the common words are trimmed off, because a name
        // can be one letter off a common word (Rasheed and raseed, a receipt).
        if let Some(record) = staff_record(&look, scan, span) {
            names.spans.push(record.words);
            names.records.push(record);
            continue;
        }
        // A common word spelt wrong ("kitny", "septmber", "gaahkon") is never a name that is missing
        // from the books: the words around it are what is looked at (money review of run 1).
        let vocabulary = |at: usize| {
            scan.word(at)
                // A word the table knows exactly (a place, a part of a firm's name) is looked up as a
                // record; only a spelling one letter off a known word is dropped.
                .is_some_and(|w| {
                    !table().is_known(w)
                        && (table().is_near_known(w) || super::when::is_near_time_word(w))
                })
        };
        let mut span = span;
        while span.0 < span.1 && vocabulary(span.0) {
            span.0 = span.0.saturating_add(1);
        }
        while span.0 < span.1 && vocabulary(span.1.saturating_sub(1)) {
            span.1 = span.1.saturating_sub(1);
        }
        if span.0 >= span.1 {
            continue;
        }
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
        // (inactive ones too), is no known چکتا buyer, and the partner section is open: while it
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
            None if typed.iter().enumerate().all(|(i, w)| {
                // A word inside a phrase of the table ("market safar", "adda report") is known too.
                let at = span.0.saturating_add(i);
                table().is_known(w) || scan.hits.iter().any(|h| h.start <= at && at < h.end)
            }) => {}
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
    let mut unclear = false;
    let party = party_ok
        .then(|| look.match_accounts(typed))
        .flatten()
        .map(|best| {
            unclear = !best.clear;
            party_ref(scan, span, wanted, best)
        });
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
            if unclear {
                names.doubts.push(Doubt::CloseSpelling);
            }
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
                if !best.clear {
                    names.doubts.push(Doubt::CloseSpelling);
                }
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
            })
            .collect();
        let start = number.map_or(hit.start, |(at, _)| at.min(hit.start));
        let end = number.map_or(hit.end, |(at, _)| at.saturating_add(1).max(hit.end));
        // A trip number or day that no trip has ("safar 99"): no such trip, said as it is (live run 2,
        // G9), never the guide or the screen of all trips.
        if found.is_empty() {
            names.spans.push((start, end));
            names.parties.push(PartyRef {
                text: scan.phrase(start, end),
                words: (start, end),
                asked_role: Some(PartyRole::Trip),
                outcome: Outcome::NotFound,
                confidence: Confidence::High,
            });
            continue;
        }
        let best = Best {
            hits: found,
            exact: !days.is_empty(),
            clear: true,
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
        clear: true,
        others: false,
    })
}
