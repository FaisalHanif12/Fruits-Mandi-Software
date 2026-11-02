//! The words the model reads before every question (D6-system-prompt.md, Code Standards Q11).
//!
//! Four fixed files, copied word for word from `docs/deliverables/D6-system-prompt.md` (Sections 2,
//! 3, 4 and 6) and embedded with `include_str!`. Nothing in them is built at run time, so every
//! request starts with the same bytes and OpenRouter can cache them. A test (P7) keeps the files
//! equal to the document; a change to any of them is a requirement change, made in the document
//! first and approved by Umar.
//!
//! The word lists (`words.tsv`) are read by `check.rs` and by the prompt tests only from here:
//! there is no second list in code (Code Standards Q11).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::Value;

/// The instructions: who the assistant is, the rules, the Owner's words, the sources, the
/// references, the answer, advice, estimates and 15 worked examples.
pub const SYSTEM: &str = include_str!("../prompt/system.md");
/// The 8 tools of PRD FR-7.87 as strict function definitions.
pub const TOOLS_JSON: &str = include_str!("../prompt/tools.json");
/// The strict schema of the final answer.
pub const ANSWER_SCHEMA_JSON: &str = include_str!("../prompt/answer.schema.json");
/// The word lists that `check.rs` and the prompt tests share.
pub const WORDS_TSV: &str = include_str!("../prompt/words.tsv");

/// The names of the 8 tools, in the order of `tools.json`.
pub const TOOL_NAMES: [&str; 8] = [
    "find",
    "party_card",
    "report",
    "analytics",
    "tiles",
    "period",
    "calculate",
    "estimate",
];

/// One kind of word list in `words.tsv`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WordKind {
    /// Number words, refused in the text and in the next questions.
    Number,
    /// Month names, refused in the text and in the next questions.
    Month,
    /// Words the answer must never use.
    Never,
    /// Words of "he owes the shop" (بنام), refused next to a جمع balance.
    OwesShop,
    /// Words of "the shop owes him" (جمع), refused next to a بنام balance.
    ShopOwes,
}

impl WordKind {
    fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "number" => Some(Self::Number),
            "month" => Some(Self::Month),
            "never" => Some(Self::Never),
            "owes_shop" => Some(Self::OwesShop),
            "shop_owes" => Some(Self::ShopOwes),
            _ => None,
        }
    }
}

/// The word lists, by kind and language (`ur` or `en`). Parsed once from [`WORDS_TSV`].
#[derive(Debug, Default)]
pub struct WordLists {
    lists: BTreeMap<(WordKind, String), Vec<String>>,
}

impl WordLists {
    /// Parses `words.tsv`: lines of `kind<TAB>lang<TAB>word`; `#` starts a comment line.
    pub fn parse(text: &str) -> Self {
        let mut lists: BTreeMap<(WordKind, String), Vec<String>> = BTreeMap::new();
        for line in text
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
        {
            let mut parts = line.splitn(3, '\t');
            let (Some(kind), Some(lang), Some(word)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            if let Some(kind) = WordKind::from_tag(kind) {
                lists
                    .entry((kind, lang.to_owned()))
                    .or_default()
                    .push(word.to_owned());
            }
        }
        Self { lists }
    }

    /// The words of one kind in one language.
    pub fn get(&self, kind: WordKind, lang: &str) -> &[String] {
        self.lists
            .get(&(kind, lang.to_owned()))
            .map_or(&[], Vec::as_slice)
    }

    /// The words of one kind in both languages.
    pub fn both(&self, kind: WordKind) -> impl Iterator<Item = &String> {
        ["ur", "en"]
            .into_iter()
            .flat_map(move |lang| self.get(kind, lang).iter())
    }
}

/// The word lists of the shipped `words.tsv`.
pub fn words() -> &'static WordLists {
    static LISTS: OnceLock<WordLists> = OnceLock::new();
    LISTS.get_or_init(|| WordLists::parse(WORDS_TSV))
}

/// The tool definitions of [`TOOLS_JSON`], as sent in the request.
pub fn tools() -> &'static [Value] {
    static TOOLS: OnceLock<Vec<Value>> = OnceLock::new();
    TOOLS.get_or_init(|| serde_json::from_str(TOOLS_JSON).unwrap_or_default())
}

/// The answer schema of [`ANSWER_SCHEMA_JSON`], as sent in the request.
pub fn answer_schema() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| serde_json::from_str(ANSWER_SCHEMA_JSON).unwrap_or(Value::Null))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_files_parse() {
        assert_eq!(tools().len(), TOOL_NAMES.len());
        for (tool, name) in tools().iter().zip(TOOL_NAMES) {
            assert_eq!(tool["function"]["name"], name);
            assert_eq!(tool["function"]["strict"], true);
        }
        assert_eq!(answer_schema()["type"], "object");
        assert!(SYSTEM.starts_with("# Who you are"));
    }

    #[test]
    fn the_word_lists_have_every_kind() {
        let w = words();
        assert!(w.get(WordKind::Number, "ur").iter().any(|x| x == "ایک"));
        assert!(w.get(WordKind::Number, "en").iter().any(|x| x == "lakh"));
        assert!(w.get(WordKind::Month, "ur").iter().any(|x| x == "جنوری"));
        assert!(!w.get(WordKind::Never, "ur").is_empty());
        assert!(!w.get(WordKind::OwesShop, "ur").is_empty());
        assert!(!w.get(WordKind::ShopOwes, "en").is_empty());
        // The comment line is not a word.
        assert!(w.both(WordKind::Number).all(|x| !x.starts_with('#')));
    }
}
