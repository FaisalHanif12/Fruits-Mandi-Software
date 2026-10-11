//! The in-memory search index and the matching (Tech Stack 4.7, plan 06, NFR-SPD-6).
//!
//! The index is a speed aid, never a source of truth (Code Standards B27): it is built from the
//! database, updated after a save commits, and can be thrown away and built again with the same
//! result. Every key is made by `mandi_core::text` for the record and for the query, so look-alike
//! Urdu letters, diacritics and zero-width characters never matter (NFR-URD-18).

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use mandi_db::masters::SearchSource;
use mandi_domain::core::masters::MasterKind;
use mandi_domain::core::search::{SearchHit, SearchScope};
use mandi_domain::core::text::{self, UrduCollator};

/// The most hits one search returns (plan 06).
pub const MAX_HITS: usize = 50;
/// The most records the "full list" returns (`Alt+Down`, PRD FR-7.41).
pub const MAX_FULL_LIST: usize = 500;
/// How many recently used records are kept per list and user (PRD FR-7.41).
pub const RECENT_KEPT: usize = 10;
/// A word of at most this many letters may have one mistake; a longer word two (Tech Stack 4.7).
const SHORT_WORD_LETTERS: usize = 5;

/// One record in the index.
#[derive(Debug)]
pub struct Entry {
    /// Row id.
    pub id: i64,
    /// Code, lower case, or empty.
    code: String,
    /// What the picker shows.
    hit: SearchHit,
    key_ur: String,
    key_en: String,
    key_ph: String,
    words_ur: Vec<String>,
    words_en: Vec<String>,
    /// The Urdu name as a sort key: records with equal rank are ordered by comparing these bytes.
    name_key: Vec<u8>,
}

impl Entry {
    fn from_source(scope: SearchScope, source: SearchSource) -> Entry {
        // Bank accounts also find the bank name, title and number (typed in one script).
        let more = source.more_keys.trim();
        let mut key_ur = source
            .search_ur
            .unwrap_or_else(|| text::search_ur(&source.name_ur));
        let mut key_en = source
            .search_en
            .unwrap_or_else(|| text::search_en(&source.name_en));
        let key_ph = source
            .search_phonetic
            .unwrap_or_else(|| text::search_phonetic(&source.name_en));
        if !more.is_empty() {
            key_ur = format!("{key_ur} {}", text::search_ur(more));
            key_en = format!("{key_en} {}", text::search_en(more));
        }
        let words = |key: &str| -> Vec<String> {
            key.split(' ')
                .filter(|w| !w.is_empty())
                .map(str::to_owned)
                .collect()
        };
        let name_key = match collator() {
            Some(c) => c.sort_key(&source.name_ur),
            None => source.name_ur.as_bytes().to_vec(),
        };
        Entry {
            id: source.id,
            name_key,
            code: source.code.to_lowercase(),
            words_ur: words(&key_ur),
            words_en: words(&key_en),
            key_ur,
            key_en,
            key_ph,
            hit: SearchHit {
                id: source.id,
                kind: scope,
                code: source.code,
                name_ur: source.name_ur,
                name_en: source.name_en,
                extra_ur: source.extra_ur,
                extra_en: source.extra_en,
                is_active: source.is_active,
            },
        }
    }
}

/// All entries of all lists, by scope, each list in id order.
#[derive(Debug, Default, Clone)]
pub struct Index {
    scopes: BTreeMap<SearchScope, Vec<Arc<Entry>>>,
}

impl Index {
    /// An index with nothing in it.
    pub fn empty() -> Index {
        Index::default()
    }

    /// Puts the records of one list in the index, replacing the list.
    pub fn set_scope(&mut self, kind: MasterKind, sources: Vec<SearchSource>) {
        let scope = SearchScope::of(kind);
        let mut entries: Vec<Arc<Entry>> = sources
            .into_iter()
            .map(|s| Arc::new(Entry::from_source(scope, s)))
            .collect();
        entries.sort_by_key(|e| e.id);
        self.scopes.insert(scope, entries);
    }

    /// Puts one record in (new or changed), keeping the list in id order. `None` takes it out.
    pub fn put(&mut self, kind: MasterKind, id: i64, source: Option<SearchSource>) {
        let scope = SearchScope::of(kind);
        let list = self.scopes.entry(scope).or_default();
        let position = list.binary_search_by_key(&id, |e| e.id);
        match (position, source) {
            (Ok(at), Some(source)) => {
                if let Some(slot) = list.get_mut(at) {
                    *slot = Arc::new(Entry::from_source(scope, source));
                }
            }
            (Err(at), Some(source)) => {
                list.insert(at, Arc::new(Entry::from_source(scope, source)));
            }
            (Ok(at), None) => {
                list.remove(at);
            }
            (Err(_), None) => {}
        }
    }

    /// How many records a list has (active and inactive).
    pub fn len(&self, scope: SearchScope) -> usize {
        self.scopes.get(&scope).map_or(0, Vec::len)
    }

    /// True if the list has no records.
    pub fn is_empty(&self, scope: SearchScope) -> bool {
        self.len(scope) == 0
    }

    fn entries(&self, scope: SearchScope) -> &[Arc<Entry>] {
        self.scopes.get(&scope).map_or(&[], Vec::as_slice)
    }

    /// The active record with this id, if any.
    pub fn active_hit(&self, scope: SearchScope, id: i64) -> Option<&SearchHit> {
        let list = self.entries(scope);
        let at = list.binary_search_by_key(&id, |e| e.id).ok()?;
        list.get(at).map(|e| &e.hit).filter(|hit| hit.is_active)
    }
}

/// What the user typed, in the forms the keys are in.
struct Query {
    /// Lower case code text (digits and letters), for the code tiers.
    code: String,
    /// True if the text has an Urdu letter: then the Urdu keys are searched.
    urdu: bool,
    ur_words: Vec<String>,
    en_words: Vec<String>,
    ph: String,
}

impl Query {
    fn new(typed: &str) -> Query {
        let ur = text::search_ur(typed);
        let en = text::search_en(typed);
        let words = |key: &str| -> Vec<String> {
            key.split(' ')
                .filter(|w| !w.is_empty())
                .map(str::to_owned)
                .collect()
        };
        Query {
            code: typed.trim().to_lowercase(),
            urdu: typed.chars().any(text::is_urdu_letter),
            ur_words: words(&ur),
            en_words: words(&en),
            ph: text::search_phonetic(typed),
        }
    }

    fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}

/// How well an entry matches. Lower is better; compared field by field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Rank {
    tier: u8,
    distance: usize,
}

const TIER_CODE: u8 = 0;
const TIER_STARTS: u8 = 1;
const TIER_USED: u8 = 2;
const TIER_CLOSE: u8 = 3;

/// Numbers compare as numbers: `01` is `1`.
fn same_code(entry_code: &str, typed: &str) -> bool {
    if entry_code.is_empty() || typed.is_empty() {
        return false;
    }
    if entry_code == typed {
        return true;
    }
    match (entry_code.parse::<u64>(), typed.parse::<u64>()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Every typed word is the start of some word of the record.
fn all_words_start(typed: &[String], words: &[String]) -> bool {
    !typed.is_empty()
        && typed
            .iter()
            .all(|t| words.iter().any(|w| w.starts_with(t.as_str())))
}

/// Every typed word is inside some word of the record.
fn all_words_inside(typed: &[String], words: &[String]) -> bool {
    !typed.is_empty()
        && typed
            .iter()
            .all(|t| words.iter().any(|w| w.contains(t.as_str())))
}

/// The most mistakes a word of this length may have.
fn allowed_mistakes(letters: usize) -> usize {
    if letters <= SHORT_WORD_LETTERS { 1 } else { 2 }
}

/// The distance of the typed words to the record: every typed word must be within its allowed
/// mistakes of some word of the record (or, for one word, the whole key). The result is the
/// largest of the best distances; `None` if some word has no near word.
fn close_distance(typed: &[String], whole: &str, words: &[String]) -> Option<usize> {
    if typed.is_empty() {
        return None;
    }
    let mut worst = 0;
    for t in typed {
        let letters = t.chars().count();
        let limit = allowed_mistakes(letters);
        let mut best: Option<usize> = None;
        let candidates = words.iter().map(String::as_str).chain(
            // The whole key too, for a name typed without its space (`kotradha`).
            (typed.len() == 1).then_some(whole),
        );
        for word in candidates {
            // Strings whose length differs by more than the limit cannot be that close.
            if word.chars().count().abs_diff(letters) > limit {
                continue;
            }
            let d = strsim::osa_distance(t, word);
            if d <= limit {
                best = Some(best.map_or(d, |b| b.min(d)));
            }
        }
        worst = worst.max(best?);
    }
    Some(worst)
}

/// The cheap tiers: exact code, starts with, used before.
fn rank_cheap(entry: &Entry, q: &Query, used: bool) -> Option<Rank> {
    if same_code(&entry.code, &q.code) {
        return Some(Rank {
            tier: TIER_CODE,
            distance: 0,
        });
    }
    let starts = if q.urdu {
        all_words_start(&q.ur_words, &entry.words_ur)
    } else {
        all_words_start(&q.en_words, &entry.words_en)
            || (q.ph.chars().count() >= 2 && entry.key_ph.starts_with(&q.ph))
            || (!entry.code.is_empty() && entry.code.starts_with(&q.code))
    };
    if starts {
        return Some(Rank {
            tier: TIER_STARTS,
            distance: 0,
        });
    }
    if used {
        let inside = if q.urdu {
            all_words_inside(&q.ur_words, &entry.words_ur)
        } else {
            all_words_inside(&q.en_words, &entry.words_en)
        };
        if inside {
            return Some(Rank {
                tier: TIER_USED,
                distance: 0,
            });
        }
    }
    None
}

/// The last tier: close spellings (a mistake or two).
fn rank_close(entry: &Entry, q: &Query) -> Option<Rank> {
    let distance = if q.urdu {
        close_distance(&q.ur_words, &entry.key_ur, &entry.words_ur)
    } else {
        close_distance(&q.en_words, &entry.key_en, &entry.words_en)
    }?;
    Some(Rank {
        tier: TIER_CLOSE,
        distance,
    })
}

/// How often and how lately a user chose each record of a list.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Usage {
    /// The most recent first, at most [`RECENT_KEPT`].
    pub recent: Vec<i64>,
    /// How many times each was chosen.
    pub counts: BTreeMap<i64, u32>,
}

impl Usage {
    fn position(&self, id: i64) -> usize {
        self.recent
            .iter()
            .position(|r| *r == id)
            .unwrap_or(usize::MAX)
    }

    fn count(&self, id: i64) -> u32 {
        self.counts.get(&id).copied().unwrap_or(0)
    }

    fn knows(&self, id: i64) -> bool {
        self.counts.contains_key(&id) || self.recent.contains(&id)
    }
}

fn collator() -> Option<&'static UrduCollator> {
    static COLLATOR: OnceLock<Option<UrduCollator>> = OnceLock::new();
    COLLATOR.get_or_init(|| UrduCollator::new().ok()).as_ref()
}

/// Urdu alphabetical order of two records by their Urdu names, then by id.
fn by_name(a: &Entry, b: &Entry) -> Ordering {
    a.name_key.cmp(&b.name_key).then_with(|| a.id.cmp(&b.id))
}

impl Index {
    /// The active records of a list that match the typed text, best first: exact code, then
    /// records that start with the text, then records this user chose before, then close
    /// spellings. Equal ranks: recently chosen first, more often chosen first, then Urdu
    /// alphabetical order (Tech Stack 4.7). At most `limit` (and never more than [`MAX_HITS`]).
    pub fn search(
        &self,
        scope: SearchScope,
        typed: &str,
        limit: usize,
        usage: &Usage,
    ) -> Vec<SearchHit> {
        let query = Query::new(typed);
        if query.is_empty() {
            return Vec::new();
        }
        let limit = limit.clamp(1, MAX_HITS);
        let mut found: Vec<(Rank, &Arc<Entry>)> = Vec::new();
        let mut open: Vec<&Arc<Entry>> = Vec::new();
        // First the cheap tiers. The close spellings are only worked out if the cheap ones
        // did not already fill the answer.
        for entry in self.entries(scope).iter().filter(|e| e.hit.is_active) {
            match rank_cheap(entry, &query, usage.knows(entry.id)) {
                Some(r) => found.push((r, entry)),
                None => open.push(entry),
            }
        }
        if found.len() < limit {
            for entry in open {
                if let Some(r) = rank_close(entry, &query) {
                    found.push((r, entry));
                }
            }
        }
        let order = |(ra, a): &(Rank, &Arc<Entry>), (rb, b): &(Rank, &Arc<Entry>)| {
            ra.cmp(rb)
                .then_with(|| usage.position(a.id).cmp(&usage.position(b.id)))
                .then_with(|| usage.count(b.id).cmp(&usage.count(a.id)))
                .then_with(|| by_name(a, b))
        };
        // Only the best `limit` are needed: pick them, then put those in order.
        if found.len() > limit {
            found.select_nth_unstable_by(limit.saturating_sub(1), order);
            found.truncate(limit);
        }
        found.sort_by(order);
        found
            .into_iter()
            .take(limit)
            .map(|(_, e)| e.hit.clone())
            .collect()
    }

    /// The list for `Alt+Down` (PRD FR-7.41): the records this user chose last (up to 10, the
    /// latest first), then the rest in Urdu alphabetical order. Active records only, at most
    /// [`MAX_FULL_LIST`].
    pub fn recent(&self, scope: SearchScope, usage: &Usage) -> Vec<SearchHit> {
        let list = self.entries(scope);
        let mut out: Vec<SearchHit> = Vec::new();
        for id in &usage.recent {
            if let Some(hit) = self.active_hit(scope, *id) {
                out.push(hit.clone());
            }
        }
        let mut rest: Vec<&Arc<Entry>> = list
            .iter()
            .filter(|e| e.hit.is_active && !usage.recent.contains(&e.id))
            .collect();
        rest.sort_by(|a, b| {
            usage
                .count(b.id)
                .cmp(&usage.count(a.id))
                .then_with(|| by_name(a, b))
        });
        out.extend(rest.into_iter().map(|e| e.hit.clone()));
        out.truncate(MAX_FULL_LIST);
        out
    }
}
