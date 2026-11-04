//! Items in a question: kela, saib, mausami, apples, کیلے (plan 55 step 1, plan 52 item 3).
//!
//! An item is found as typed, then as its singular (a plural or an oblique form: apples, kele, kelon,
//! کیلے), then through a short list of Roman Urdu names with their English name, then through the
//! transliteration words (`kela` is کیلا). A word that the list knows as an item but the books do
//! not hold gives "not found" for the item, so a missing item is not mistaken for a missing party.
//! Only reads.
//!
//! Example: "kele ka commission" finds Banana (کیلا) through `kele` -> `kela`.

use mandi_services::core::search::SearchScope;
use mandi_services::core::text;

use super::scan::Scan;
use super::who::{Best, Lookup, best_of, grade_words, is_urdu, score, split_words};
use super::{Confidence, ItemHit, ItemRef, Outcome};

/// Roman Urdu and English names of items, each with the usual English name. The list is the
/// assistant's own, small and fixed (the word list of the typing help is not changed for it).
#[rustfmt::skip]
const ALIASES: &[(&str, &str)] = &[
    ("kela", "banana"),
    ("keela", "banana"),
    ("kaila", "banana"),
    ("banana", "banana"),
    ("saib", "apple"),
    ("seb", "apple"),
    ("seeb", "apple"),
    ("saeb", "apple"),
    ("sayb", "apple"),
    ("apple", "apple"),
    ("mausami", "mosambi"),
    ("mosami", "mosambi"),
    ("musambi", "mosambi"),
    ("mosambi", "mosambi"),
    ("mausambi", "mosambi"),
    ("musami", "mosambi"),
    ("malta", "orange"),
    ("santra", "orange"),
    ("orange", "orange"),
    ("aam", "mango"),
    ("aamm", "mango"),
    ("mango", "mango"),
    ("angoor", "grapes"),
    ("angur", "grapes"),
    ("grapes", "grapes"),
    ("grape", "grapes"),
    ("aloo", "potato"),
    ("alu", "potato"),
    ("aaloo", "potato"),
    ("potato", "potato"),
    ("pyaz", "onion"),
    ("piyaz", "onion"),
    ("pyaaz", "onion"),
    ("onion", "onion"),
    ("tamatar", "tomato"),
    ("tamater", "tomato"),
    ("tomato", "tomato"),
    ("gajar", "carrot"),
    ("carrot", "carrot"),
    ("mooli", "radish"),
    ("muli", "radish"),
    ("radish", "radish"),
    ("gobhi", "cauliflower"),
    ("gobi", "cauliflower"),
    ("cauliflower", "cauliflower"),
    ("palak", "spinach"),
    ("spinach", "spinach"),
    ("matar", "peas"),
    ("mattar", "peas"),
    ("peas", "peas"),
    ("bhindi", "okra"),
    ("bhendi", "okra"),
    ("okra", "okra"),
    ("kheera", "cucumber"),
    ("khira", "cucumber"),
    ("cucumber", "cucumber"),
    ("nimbu", "lemon"),
    ("lemon", "lemon"),
    ("lime", "lemon"),
    ("lehsan", "garlic"),
    ("lahsan", "garlic"),
    ("garlic", "garlic"),
    ("adrak", "ginger"),
    ("ginger", "ginger"),
    ("anar", "pomegranate"),
    ("anaar", "pomegranate"),
    ("pomegranate", "pomegranate"),
    ("amrood", "guava"),
    ("amrud", "guava"),
    ("guava", "guava"),
    ("kinnu", "kinnow"),
    ("kinno", "kinnow"),
    ("kinow", "kinnow"),
    ("kinnow", "kinnow"),
    ("tarbooz", "watermelon"),
    ("tarbuz", "watermelon"),
    ("watermelon", "watermelon"),
    ("kharbooza", "melon"),
    ("kharbuza", "melon"),
    ("melon", "melon"),
    ("papita", "papaya"),
    ("papaya", "papaya"),
    ("nashpati", "pear"),
    ("pear", "pear"),
    ("aaru", "peach"),
    ("aru", "peach"),
    ("peach", "peach"),
    ("khubani", "apricot"),
    ("apricot", "apricot"),
    ("ananas", "pineapple"),
    ("pineapple", "pineapple"),
    ("khajoor", "dates"),
    ("khajur", "dates"),
    ("dates", "dates"),
    ("nariyal", "coconut"),
    ("coconut", "coconut"),
    ("chiku", "chikoo"),
    ("cheeku", "chikoo"),
    ("chikoo", "chikoo"),
    ("lichi", "lychee"),
    ("lychee", "lychee"),
    ("litchi", "lychee"),
    ("strawberry", "strawberry"),
    ("anjeer", "fig"),
    ("cherry", "cherry"),
    ("shaljam", "turnip"),
    ("kaddu", "pumpkin"),
    ("baingan", "brinjal"),
    ("brinjal", "brinjal"),
    ("karela", "bitter gourd"),
];

fn alias(word: &str) -> Option<&'static str> {
    ALIASES.iter().find(|(a, _)| *a == word).map(|(_, e)| *e)
}

/// The word as it may be written in the singular, most likely first (apples, kele, kelon, کیلے).
fn variants(word: &str) -> Vec<String> {
    let mut out = vec![word.to_owned()];
    let mut add = |s: String| {
        if s.chars().count() >= 3 && !out.contains(&s) {
            out.push(s);
        }
    };
    if is_urdu(word) {
        if let Some(stem) = word.strip_suffix('ے') {
            add(format!("{stem}ا"));
        }
        for suffix in ["وں", "یں", "ویں"] {
            if let Some(stem) = word.strip_suffix(suffix) {
                add(stem.to_owned());
                add(format!("{stem}ا"));
            }
        }
        if let Some(stem) = word.strip_suffix("یاں") {
            add(stem.to_owned());
            add(format!("{stem}ی"));
        }
        return out;
    }
    if let Some(stem) = word.strip_suffix("ies") {
        add(format!("{stem}y"));
    }
    if let Some(stem) = word.strip_suffix('s') {
        add(stem.to_owned());
    }
    for suffix in ["ay", "e"] {
        if let Some(stem) = word.strip_suffix(suffix) {
            add(format!("{stem}a"));
        }
    }
    for suffix in ["on", "oon", "ein", "yan", "iyan", "ian"] {
        if let Some(stem) = word.strip_suffix(suffix) {
            add(stem.to_owned());
            add(format!("{stem}a"));
            add(format!("{stem}i"));
        }
    }
    out
}

fn search_items(look: &Lookup<'_>, typed: &[String]) -> Option<Best<ItemHit>> {
    let joined = typed.join(" ");
    let use_urdu = typed.iter().any(|w| is_urdu(w));
    let scored: Vec<(ItemHit, u32, bool)> = look
        .list(SearchScope::Item, &joined)
        .into_iter()
        .filter_map(|hit| {
            let name = if use_urdu {
                split_words(&text::search_ur(&hit.name_ur))
            } else {
                split_words(&text::search_en(&hit.name_en))
            };
            let (total, exact) = grade_words(typed, &name)?;
            Some((
                ItemHit {
                    key: format!("i{}", hit.id),
                    id: hit.id,
                    code: hit.code,
                    name_ur: hit.name_ur,
                    name_en: hit.name_en,
                },
                score(total, typed.len(), name.len()),
                exact,
            ))
        })
        .collect();
    best_of(scored)
}

fn item_ref(scan: &Scan, span: (usize, usize), best: Best<ItemHit>) -> ItemRef {
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
            Outcome::Several(many.iter().take(5).cloned().collect()),
            Confidence::Low,
        ),
    };
    ItemRef {
        text: scan.phrase(span.0, span.1),
        words: span,
        outcome,
        confidence,
    }
}

/// The item a name (one or two words) is, if it is one: found in the books, or known as an item
/// that the books do not hold (not found).
pub(super) fn resolve(look: &Lookup<'_>, scan: &Scan, span: (usize, usize)) -> Option<ItemRef> {
    let typed = scan.words.get(span.0..span.1)?;
    let last = typed.last()?;
    let head: Vec<String> = typed
        .iter()
        .take(typed.len().saturating_sub(1))
        .cloned()
        .collect();
    let forms = variants(last);
    let with_head = |form: &str| -> Vec<String> {
        let mut words = head.clone();
        words.push(form.to_owned());
        words
    };
    let mut known: Option<&'static str> = None;
    // The list of Roman Urdu names: the English name is searched.
    if head.is_empty() {
        for form in &forms {
            if let Some(english) = alias(form) {
                known = Some(english);
                let words = vec![english.to_owned()];
                if let Some(best) = search_items(look, &words) {
                    return Some(item_ref(scan, span, best));
                }
            }
        }
    }
    // As typed, then as the singular. A singular made from the word ("banate" -> "banata") must be
    // the same word as the item's: a close spelling counts only for the word as typed.
    for (at, form) in forms.iter().enumerate() {
        let found = search_items(look, &with_head(form));
        let found = if at == 0 {
            found
        } else {
            found.filter(|b| b.exact)
        };
        if let Some(best) = found {
            return Some(item_ref(scan, span, best));
        }
    }
    // Through the transliteration words: the Urdu spelling of the Roman word.
    if head.is_empty() && !is_urdu(last) {
        for form in forms.iter().take(2) {
            for urdu in look.urdu_of(form).into_iter().take(3) {
                // A spelling made from Roman letters is a guess: only the same word counts.
                if let Some(best) = search_items(look, &split_words(&urdu)).filter(|b| b.exact) {
                    return Some(item_ref(scan, span, best));
                }
            }
        }
    }
    // Known as an item, not in the books.
    known.map(|_| ItemRef {
        text: scan.phrase(span.0, span.1),
        words: span,
        outcome: Outcome::NotFound,
        confidence: Confidence::High,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plurals_and_oblique_forms_lead_back_to_the_singular() {
        assert!(variants("apples").contains(&"apple".to_owned()));
        assert!(variants("kele").contains(&"kela".to_owned()));
        assert!(variants("kelay").contains(&"kela".to_owned()));
        assert!(variants("kelon").contains(&"kela".to_owned()));
        assert!(variants("saibon").contains(&"saib".to_owned()));
        assert!(variants("mausamiyan").contains(&"mausami".to_owned()));
        assert!(variants("کیلے").contains(&"کیلا".to_owned()));
        assert!(variants("سیبوں").contains(&"سیب".to_owned()));
        assert_eq!(alias("mausami"), Some("mosambi"));
        assert_eq!(alias("zubair"), None);
    }
}
