//! The `find` role `lot`: a lot by its name (plan 71 L9, PRD FR-12.45, FR-7.77).
//!
//! A lot's name is its seller, its item and a counter: احسان، آم-2. The Owner may write any of its
//! parts in any order, in Roman Urdu, Urdu letters or English:
//!
//! | Typed | Found |
//! |---|---|
//! | `ehsan aam-2`, `ehsan-2 ke aam`, `احسان آم-2` | exactly that lot |
//! | `ehsan ke aam`, `ehsan ke 200 peti aam` | every mango lot of Ehsan: one match answers, several ask "کون سا؟" |
//! | `ehsan` | his lots, newest first |
//!
//! The words are split into the counter (a number joined to a word by a hyphen: `aam-2`), the words
//! that only fill the sentence (`ke`, `peti`, a quantity), the seller's words and the item's words.
//! A word is the seller's when a seller's name holds it (or it was found through its Urdu spelling),
//! else the item's when an item has it. The lots are then read through `services::bikri::find_lots`.
//! Each match has its key `l` and the lot, its name (seller, item and counter), its day and the
//! quantity received: the buttons of "کون سا؟" show them (PRD FR-7.77). Nothing but names, dates and
//! counts is sent (PRD FR-7.83).
//!
//! Example (E2E-70 data, Ehsan with آم-1 and آم-2): `find{text:"ehsan aam-2", role:"lot"}` gives one
//! match, `l7`, "احسان، آم-2"; `find{text:"ehsan ke aam", role:"lot"}` gives two matches, newest first.

use mandi_services::bikri;
use mandi_services::core::accounts::RecordRef;
use mandi_services::core::bikri::{LotFindInput, LotFound, lot_label};

use super::find::{Exact, Role, search_role};
use super::find_lists::list_node;
use super::typed::add_note;
use super::{Env, Stop, service_stop};
use crate::limits::MAX_OPTIONS;
use crate::results::{NameVal, Node, Val};

/// What the model is told with a lot find: a lot is read through its seller and item.
const LOT_NOTE: &str = "a lot is one seller's goods of one item; to read its sales call report \
    with seller = the match's seller and item = the match's item (their keys), and the report is \
    narrowed to the lot the Owner chose";

/// Words that only fill the sentence, in Roman Urdu, Urdu letters and English.
const FILLER: [&str; 22] = [
    "ke", "ka", "ki", "ko", "se", "me", "mein", "wala", "wale", "wali", "of", "the", "lot", "lots",
    "peti", "petiyan", "nag", "box", "boxes", "کے", "کا", "کی",
];

/// Splits a typed word that carries a counter: `aam-2` is the word `aam` and the counter 2. A word
/// with nothing before the hyphen (`-2`) is only the counter. `None` for any other word.
fn with_counter(word: &str) -> Option<(&str, i64)> {
    let (left, right) = word.rsplit_once('-')?;
    if right.is_empty() || !right.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((left, right.parse().ok()?))
}

/// The words of the typed text that name a seller or an item, and the counter if one was written.
///
/// Example: "ehsan ke 200 peti aam-2" is the words `ehsan` and `aam`, and the counter 2.
fn words_of(query: &str) -> (Vec<String>, Option<i64>) {
    let mut counter = None;
    let mut words = Vec::new();
    for raw in query.split_whitespace() {
        let word = match with_counter(raw) {
            Some((left, n)) => {
                counter = Some(n);
                left
            }
            None => raw,
        };
        let lower = word.to_lowercase();
        let is_quantity = !lower.is_empty() && lower.bytes().all(|b| b.is_ascii_digit());
        if lower.is_empty() || is_quantity || FILLER.contains(&lower.as_str()) {
            continue;
        }
        words.push(word.to_owned());
    }
    (words, counter)
}

/// The record id inside a match key such as `a20` or `i4`.
fn id_of(key: &str, prefix: char) -> Option<i64> {
    key.strip_prefix(prefix)?.parse().ok()
}

/// What a word names.
enum Named {
    Seller(Vec<i64>),
    Item(Vec<i64>),
    Nothing,
}

/// How close a word must be to a name to name it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reading {
    /// The words of a lot's name and nothing else ("ehsan ke aam"): a name may be started ("ehsan"
    /// for "ehsan ali") and an item may be a close spelling.
    Name,
    /// A sentence with many other words ("ehsan ke 200 peti aam kis kis ne liye"): a word names a
    /// seller or an item only when it is a whole word of that name (or its Urdu spelling is).
    Sentence,
}

/// True when the record `m` found for `word` is named by it, as `reading` asks.
fn names(m: &Node, exact: &Exact, via: Option<&Exact>, reading: Reading) -> bool {
    let Some(Node::Val(Val::Name(n))) = m.get(&["name"]) else {
        return false;
    };
    let tag = n.tag.as_deref();
    match reading {
        Reading::Name => exact.holds(&n.name_ur, &n.name_en, tag),
        Reading::Sentence => {
            exact.whole_word(&n.name_ur, &n.name_en, tag)
                || via.is_some_and(|v| v.whole_word(&n.name_ur, &n.name_en, tag))
        }
    }
}

/// Decides whether one word names a seller or an item, and which records.
fn classify(env: &Env<'_>, word: &str, reading: Reading) -> Result<Named, Stop> {
    let exact = Exact::of(word);
    let sellers = search_role(env, Role::Seller, word)?;
    let seller_via = sellers.via.as_ref().map(|(_, text)| Exact::of(text));
    let strict: Vec<i64> = sellers
        .matches
        .iter()
        .filter(|m| {
            // Found through its Urdu spelling, or its name holds the typed word.
            (reading == Reading::Name && sellers.via.is_some())
                || names(m, &exact, seller_via.as_ref(), reading)
        })
        .filter_map(|m| match m.get(&["key"]) {
            Some(Node::Str(key)) => id_of(key, 'a'),
            _ => None,
        })
        .collect();
    if !strict.is_empty() {
        return Ok(Named::Seller(strict));
    }
    let items = search_role(env, Role::Item, word)?;
    let item_via = items.via.as_ref().map(|(_, text)| Exact::of(text));
    let item_ids: Vec<i64> = items
        .matches
        .iter()
        .filter(|m| reading == Reading::Name || names(m, &exact, item_via.as_ref(), reading))
        .filter_map(|m| match m.get(&["key"]) {
            Some(Node::Str(key)) => id_of(key, 'i'),
            _ => None,
        })
        .collect();
    Ok(if item_ids.is_empty() {
        Named::Nothing
    } else {
        Named::Item(item_ids)
    })
}

/// A record inside a lot's match: its key and its names.
fn record_node(prefix: char, id: &i64, record: &RecordRef) -> Node {
    Node::obj(vec![
        ("key", Node::Str(format!("{prefix}{id}"))),
        (
            "name",
            Node::Val(Val::Name(NameVal {
                name_ur: record.name_ur.clone(),
                name_en: record.name_en.clone(),
                tag: record.tag.clone(),
                code: Some(record.code.clone()).filter(|c| !c.is_empty()),
            })),
        ),
    ])
}

/// One lot as a match.
fn lot_match(found: &LotFound) -> Node {
    let name = &found.name;
    let (seller_ur, seller_en) = (&name.seller.name_ur, &name.seller.name_en);
    Node::obj(vec![
        ("key", Node::Str(format!("l{}", name.lot_id))),
        ("code", Node::Str(String::new())),
        ("type", Node::Str("lot".to_owned())),
        // The lot's name: "احسان، آم-2" and "Ehsan, Mango-2". The screen draws it as a name.
        (
            "name",
            Node::Val(Val::Name(NameVal {
                name_ur: format!(
                    "{seller_ur}، {}",
                    lot_label(&name.item.name_ur, name.counter)
                ),
                name_en: format!(
                    "{seller_en}, {}",
                    lot_label(&name.item.name_en, name.counter)
                ),
                tag: name.seller.tag.clone(),
                code: None,
            })),
        ),
        ("date", Node::Val(Val::Date(found.entry_date.clone()))),
        ("qty_received", Node::Val(Val::Count(found.qty_received))),
        ("bikri_number", Node::Val(Val::Count(found.bikri_number))),
        // The seller and the item of the lot as records with their keys, so a report call can take
        // them (`seller`, `item`) like the match of a find of their own.
        ("seller", record_node('a', &name.seller.id, &name.seller)),
        ("item", record_node('i', &name.item.id, &name.item)),
        ("active", Node::Bool(true)),
    ])
}

/// The lots a typed name finds, newest first, with how many there are in all.
///
/// # Errors
/// A stop for the model: the services it reads.
pub(super) fn lots(env: &Env<'_>, query: &str) -> Result<Node, Stop> {
    let (words, counter) = words_of(query);
    let mut seller_ids: Vec<i64> = Vec::new();
    let mut item_ids: Vec<i64> = Vec::new();
    for word in &words {
        match classify(env, word, Reading::Name)? {
            Named::Seller(ids) => seller_ids.extend(ids),
            Named::Item(ids) => item_ids.extend(ids),
            Named::Nothing => {}
        }
    }
    seller_ids.sort_unstable();
    seller_ids.dedup();
    item_ids.sort_unstable();
    item_ids.dedup();
    // Nothing named, or a counter alone: no lot is guessed.
    if seller_ids.is_empty() && item_ids.is_empty() {
        return Ok(list_node("lot", query, Vec::new(), 0, Vec::new()));
    }
    let limit = i64::try_from(MAX_OPTIONS.saturating_add(1)).unwrap_or(6);
    let (rows, total) = bikri::find_lots(
        env.ctx,
        &LotFindInput {
            seller_ids,
            item_ids,
            counter,
            lot_id: None,
            limit,
        },
    )
    .map_err(service_stop)?;
    let total = usize::try_from(total).unwrap_or(usize::MAX);
    let matches: Vec<Node> = rows.iter().take(MAX_OPTIONS).map(lot_match).collect();
    let mut node = list_node("lot", query, matches, total, Vec::new());
    if total > 0 {
        add_note(&mut node, LOT_NOTE);
    }
    Ok(node)
}

/// True when the text of a lot find ends with a counter ("ehsan aam-2").
pub fn has_counter(text: &str) -> bool {
    text.rsplit_once('-')
        .is_some_and(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// The seller and the item a question names inside a sentence, as the text of a lot find ("ehsan
/// aam", or "ehsan aam-2" when it names the counter), or `None` when it does not name both (plan 71
/// L9, PRD FR-7.77). A word counts only when it is a whole word of a seller's or an item's name, so
/// the many other words of a question ("kis", "aur", "liye") name nothing.
///
/// Example: "ehsan ke 200 peti aam kis kis ne liye aur kis ne kitna naqd diya" is "ehsan aam";
/// "aam ki bikri" is `None` (no seller).
///
/// # Errors
/// A stop for the model: the searches it reads.
pub fn named_in(env: &Env<'_>, question: &str) -> Result<Option<String>, Stop> {
    let (words, counter) = words_of(question);
    let mut sellers: Vec<String> = Vec::new();
    let mut items: Vec<String> = Vec::new();
    for word in words {
        match classify(env, &word, Reading::Sentence)? {
            Named::Seller(_) if !sellers.contains(&word) => sellers.push(word),
            Named::Item(_) if !items.contains(&word) => items.push(word),
            _ => {}
        }
    }
    if sellers.is_empty() || items.is_empty() {
        return Ok(None);
    }
    let mut text = sellers
        .into_iter()
        .chain(items)
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(counter) = counter {
        text = format!("{text}-{counter}");
    }
    Ok(Some(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_counter_joined_to_a_word_is_read_from_either_word() {
        assert_eq!(with_counter("aam-2"), Some(("aam", 2)));
        assert_eq!(with_counter("ehsan-12"), Some(("ehsan", 12)));
        assert_eq!(with_counter("-2"), Some(("", 2)));
        assert_eq!(with_counter("aam"), None);
        assert_eq!(with_counter("aam-x"), None);
        assert_eq!(with_counter("aam-"), None);
    }

    #[test]
    fn the_words_of_a_lot_question_leave_out_fillers_and_quantities() {
        assert_eq!(
            words_of("ehsan ke 200 peti aam-2"),
            (vec!["ehsan".to_owned(), "aam".to_owned()], Some(2))
        );
        assert_eq!(
            words_of("ehsan-2 ke aam"),
            (vec!["ehsan".to_owned(), "aam".to_owned()], Some(2))
        );
        assert_eq!(
            words_of("ehsan ke aam"),
            (vec!["ehsan".to_owned(), "aam".to_owned()], None)
        );
        assert_eq!(words_of("lot-2"), (Vec::new(), Some(2)));
        assert_eq!(
            words_of("احسان کے آم"),
            (vec!["احسان".to_owned(), "آم".to_owned()], None)
        );
    }

    #[test]
    fn a_match_key_holds_the_record_id() {
        assert_eq!(id_of("a20", 'a'), Some(20));
        assert_eq!(id_of("i4", 'i'), Some(4));
        assert_eq!(id_of("a20", 'i'), None);
        assert_eq!(id_of("ax", 'a'), None);
    }
}
