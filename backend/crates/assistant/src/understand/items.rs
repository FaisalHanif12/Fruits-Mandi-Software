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
    // Vegetables, with the spellings that are written (the item master decides the English name).
    ("piaz", "onion"),
    ("pyaj", "onion"),
    ("pyaaj", "onion"),
    ("piyaaz", "onion"),
    ("onions", "onion"),
    ("aalu", "potato"),
    ("aluu", "potato"),
    ("allo", "potato"),
    ("potatoes", "potato"),
    ("tamatr", "tomato"),
    ("tamaatar", "tomato"),
    ("tamatar", "tomato"),
    ("tomatoes", "tomato"),
    ("gajr", "carrot"),
    ("gaajar", "carrot"),
    ("carrots", "carrot"),
    ("moli", "radish"),
    ("mooly", "radish"),
    ("gobhee", "cauliflower"),
    ("gobee", "cauliflower"),
    ("bandgobi", "cabbage"),
    ("bandh", "cabbage"),
    ("cabbage", "cabbage"),
    ("paalak", "spinach"),
    ("palk", "spinach"),
    ("saag", "saag"),
    ("sag", "saag"),
    ("saaag", "saag"),
    ("sarson", "saag"),
    ("bhindee", "okra"),
    ("bhendee", "okra"),
    ("bhindy", "okra"),
    ("ladyfinger", "okra"),
    ("mirch", "chilli"),
    ("mirchi", "chilli"),
    ("mirchy", "chilli"),
    ("mirchen", "chilli"),
    ("chilli", "chilli"),
    ("chili", "chilli"),
    ("chillies", "chilli"),
    ("shimla", "capsicum"),
    ("capsicum", "capsicum"),
    ("adrk", "ginger"),
    ("adrakh", "ginger"),
    ("lehsun", "garlic"),
    ("lasan", "garlic"),
    ("lahsun", "garlic"),
    ("lasun", "garlic"),
    ("kadu", "pumpkin"),
    ("kaddoo", "pumpkin"),
    ("kadoo", "pumpkin"),
    ("pumpkin", "pumpkin"),
    ("lauki", "bottle gourd"),
    ("ghia", "bottle gourd"),
    ("tori", "ridge gourd"),
    ("turai", "ridge gourd"),
    ("tinda", "tinda"),
    ("arvi", "taro"),
    ("dhaniya", "coriander"),
    ("dhania", "coriander"),
    ("coriander", "coriander"),
    ("pudina", "mint"),
    ("mint", "mint"),
    ("methi", "fenugreek"),
    ("chukandar", "beetroot"),
    ("beetroot", "beetroot"),
    ("shakarkandi", "sweet potato"),
    ("shaljum", "turnip"),
    ("turnip", "turnip"),
    ("beans", "beans"),
    ("cucumbers", "cucumber"),
    ("kheeray", "cucumber"),
    // Names of two words, and the synonyms of one item.
    ("shimla mirch", "capsicum"),
    ("simla mirch", "capsicum"),
    ("shimla mirchi", "capsicum"),
    ("simla mirchi", "capsicum"),
    ("shamla mirch", "capsicum"),
    ("shimlah mirch", "capsicum"),
    ("simla", "capsicum"),
    ("bell pepper", "capsicum"),
    ("green pepper", "capsicum"),
    ("hari mirch", "green chilli"),
    ("lal mirch", "red chilli"),
    ("kali mirch", "black pepper"),
    ("phool gobi", "cauliflower"),
    ("phul gobi", "cauliflower"),
    ("band gobi", "cabbage"),
    ("patta gobi", "cabbage"),
    ("hara dhaniya", "coriander"),
    ("mitha kaddu", "pumpkin"),
    ("sweet potato", "sweet potato"),
    ("bitter gourd", "bitter gourd"),
    ("bottle gourd", "bottle gourd"),
    // The same names in Urdu letters: an item the books do not hold is "no such item", not an
    // unknown word.
    ("سیب", "apple"),
    ("آم", "mango"),
    ("کیلا", "banana"),
    ("انگور", "grapes"),
    ("مالٹا", "orange"),
    ("کینو", "kinnow"),
    ("امرود", "guava"),
    ("انار", "pomegranate"),
    ("تربوز", "watermelon"),
    ("آڑو", "peach"),
    ("خوبانی", "apricot"),
    ("کھجور", "dates"),
    ("پپیتا", "papaya"),
    ("آلو", "potato"),
    ("الو", "potato"),
    ("کیلہ", "banana"),
    ("مسمی", "mosambi"),
    ("مسمبی", "mosambi"),
    ("پیاز", "onion"),
    ("ٹماٹر", "tomato"),
    ("مولی", "radish"),
    ("ساگ", "saag"),
    ("گوبھی", "cauliflower"),
    ("بند گوبھی", "cabbage"),
    ("گاجر", "carrot"),
    ("بھنڈی", "okra"),
    ("مرچ", "chilli"),
    ("ہری مرچ", "green chilli"),
    ("لال مرچ", "red chilli"),
    ("شملہ مرچ", "capsicum"),
    ("ادرک", "ginger"),
    ("لہسن", "garlic"),
    ("کدو", "pumpkin"),
    ("پالک", "spinach"),
    ("مٹر", "peas"),
    ("کھیرا", "cucumber"),
    ("بینگن", "brinjal"),
    ("کریلا", "bitter gourd"),
    ("لیموں", "lemon"),
    ("شلجم", "turnip"),
    ("دھنیا", "coriander"),
    ("پودینہ", "mint"),
    ("میتھی", "fenugreek"),
    ("چقندر", "beetroot"),
    ("لوکی", "bottle gourd"),
    ("توری", "ridge gourd"),
    ("ٹنڈا", "tinda"),
    ("اروی", "taro"),
];

/// The key of an alias as `Scan` writes the words of a question: the Urdu look-alike letters are one
/// letter each, so an alias in Urdu letters is the same as the word typed.
fn key_of(alias: &str) -> String {
    text::search_ur(&text::search_en(alias))
}

fn alias(word: &str) -> Option<&'static str> {
    ALIASES
        .iter()
        .find(|(a, _)| *a == word || (!a.is_ascii() && key_of(a) == word))
        .map(|(_, e)| *e)
}

/// A word with its vowels, `h` and `y` taken out and each doubled letter made one: "tmatar", "tamatar"
/// and "tamatr" are all "tmtr". Used only to find a misspelt name of the alias list.
fn skeleton(word: &str) -> String {
    let mut out = String::new();
    for c in word.chars().filter(char::is_ascii_alphabetic) {
        let c = c.to_ascii_lowercase();
        if matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'h') {
            continue;
        }
        if !out.ends_with(c) {
            out.push(c);
        }
    }
    out
}

/// The English name an alias of the same skeleton gives, when all aliases of that skeleton give the
/// same one and the skeleton has 4 letters or more ("tmatar" is tomato; "motor" is nothing, because
/// "mtr" is short). A guess: the caller marks it medium.
fn alias_by_skeleton(word: &str) -> Option<&'static str> {
    if !word.is_ascii() || word.chars().count() < 4 {
        return None;
    }
    let wanted = skeleton(word);
    if wanted.chars().count() < 4 {
        return None;
    }
    let mut found: Option<&'static str> = None;
    for (name, english) in ALIASES
        .iter()
        .filter(|(a, _)| a.is_ascii() && !a.contains(' '))
    {
        if skeleton(name) == wanted {
            match found {
                None => found = Some(english),
                Some(before) if before == *english => {}
                Some(_) => return None,
            }
        }
    }
    found
}

/// An Urdu word with the letters that sound alike written as one: س ص ث, ز ض ظ ذ, ت ط, ح ہ, ا ع آ.
/// "صیب" and "سیب" are both "سیب"; "پیاض" and "پیاز" are both "پیاز". Doubled letters are one.
fn confused(word: &str) -> String {
    let mut out = String::new();
    for c in word.chars() {
        let one = match c {
            'ص' | 'ث' => 'س',
            'ض' | 'ظ' | 'ذ' => 'ز',
            'ط' => 'ت',
            'ح' => 'ہ',
            'ع' | 'آ' => 'ا',
            other => other,
        };
        if !out.ends_with(one) {
            out.push(one);
        }
    }
    out
}

/// The English name of the Urdu item whose name sounds like the word typed, when every such name gives
/// the same one: "صیب" is apple, "پیاض" is onion. A guess: the caller marks it medium, or low for a
/// short word (عام for آم is also the common word "general").
fn alias_by_sound(word: &str) -> Option<&'static str> {
    if !is_urdu(word) || word.chars().count() < 2 {
        return None;
    }
    let wanted = confused(word);
    let mut found: Option<&'static str> = None;
    for (name, english) in ALIASES
        .iter()
        .filter(|(a, _)| !a.is_ascii() && !a.contains(' '))
    {
        if confused(&key_of(name)) == wanted {
            match found {
                None => found = Some(english),
                Some(before) if before == *english => {}
                Some(_) => return None,
            }
        }
    }
    found
}

/// True for a word the alias list knows as an item (so it is not a month: "mirch" is not March).
pub(super) fn is_item_word(word: &str) -> bool {
    alias(word).is_some()
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
    // The list of Roman Urdu names: the English name is searched. A name of two words ("shimla
    // mirch") is looked up whole, with its last word in its singular forms.
    for form in &forms {
        if let Some(english) = alias(&with_head(form).join(" ")) {
            known = Some(english);
            let words = split_words(english);
            if let Some(best) = search_items(look, &words) {
                return Some(item_ref(scan, span, best));
            }
        }
    }
    // A misspelling of a Roman Urdu name ("tmatar", "tmatr" for tamatar): the same consonants.
    let mut guessed = false;
    if known.is_none()
        && head.is_empty()
        && !is_urdu(last)
        && let Some(english) = alias_by_skeleton(last)
    {
        known = Some(english);
        guessed = true;
        if let Some(best) = search_items(look, &split_words(english)) {
            let mut one = item_ref(scan, span, best);
            if one.confidence > Confidence::Medium {
                one.confidence = Confidence::Medium;
            }
            return Some(one);
        }
    }
    // A misspelling of an Urdu name whose letters sound alike ("صیب" for سیب, "پیاض" for پیاز): the item
    // by its English name. Low for a word of two letters, which may be a common word (عام).
    if known.is_none()
        && head.is_empty()
        && let Some(english) = alias_by_sound(last)
    {
        known = Some(english);
        guessed = true;
        if let Some(best) = search_items(look, &split_words(english)) {
            let mut one = item_ref(scan, span, best);
            let cap = if confused(last).chars().count() >= 3 {
                Confidence::Medium
            } else {
                Confidence::Low
            };
            if one.confidence > cap {
                one.confidence = cap;
            }
            return Some(one);
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
        // A name guessed from its consonants is not "no such item" for sure.
        confidence: if guessed {
            Confidence::Medium
        } else {
            Confidence::High
        },
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
