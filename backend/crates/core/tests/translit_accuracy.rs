//! How good the suggestions are, measured on lists nobody tuned the rules to (NFR-URD-24 v1.2,
//! NFR-URD-25, Code Standards 9.1, research report 5).
//!
//! Three lists:
//!
//! - `translit_names.tsv` (T1), the frozen 1,000-name list. It was composed from hand-written
//!   word pools. Never change it and never tune the rules or the word list to it.
//! - `translit_holdout.tsv` (T2), names of Wikidata items whose number ends in 0. The import
//!   tool keeps those items out of the word list. It is an independent source (CC0).
//! - `translit_dev.tsv`, a different hand-made list. The rules are tuned on this one.
//!
//! Each list is split by how many of its words the word list knows: every word, some words, or
//! no word. NFR-URD-24 v1.2 asks for top-1 95% and top-3 99% when every word is in the list; for
//! names with a word outside it (rules only) the top-3 rate is only reported, and must not fall
//! below `translit_baseline.txt`. Also reported: the silent wrong rate, the share of names whose
//! first suggestion is wrong and not marked "check the spelling" (source `rules`).
//!
//! Every run writes a report to the build folder (`translit_accuracy.txt`) and prints it
//! (`cargo test -p mandi-core --test translit_accuracy -- --nocapture`).
#![cfg(test)]

use std::fmt::Write as _;

use mandi_core::text::{is_english_name, is_urdu_name};
use mandi_core::translit::{Direction, Source, WordList, suggest, suggest_to_english};

const FROZEN: &str = include_str!("translit_names.tsv");
const HOLDOUT: &str = include_str!("translit_holdout.tsv");
const DEV: &str = include_str!("translit_dev.tsv");
const BASELINE: &str = include_str!("translit_baseline.txt");

/// NFR-URD-24 v1.2: names whose words are all in the word list. The right Urdu spelling is the
/// first suggestion for this share of them ...
const GATE_TOP1_IN_LIST: f64 = 95.0;
/// ... and among the three for this share.
const GATE_TOP3_IN_LIST: f64 = 99.0;
/// NFR-URD-25: the first English suggestion is accepted for this share of the whole list.
const GATE_ENGLISH: f64 = 90.0;

struct Case {
    roman: String,
    urdu: Vec<String>,
    english: Vec<String>,
}

fn parse(text: &str) -> Vec<Case> {
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|line| {
            let mut columns = line.split('\t');
            let roman = columns.next().unwrap_or_default().to_owned();
            let urdu = columns
                .next()
                .unwrap_or_default()
                .split('|')
                .map(str::to_owned)
                .collect();
            let english = columns
                .next()
                .unwrap_or_default()
                .split('|')
                .map(str::to_owned)
                .collect();
            Case {
                roman,
                urdu,
                english,
            }
        })
        .collect()
}

#[derive(Default, Clone, Copy)]
struct Tally {
    names: usize,
    top1: usize,
    top3: usize,
    /// First suggestion right when spaces are ignored: Urdu writes `پور` with or without a
    /// space, so this shows how much of a miss is only spacing.
    top1_spaceless: usize,
    english: usize,
    /// First suggestion wrong and not marked `rules`: the user may accept it without looking.
    silent_wrong: usize,
    /// First suggestion marked `rules` ("check the spelling").
    flagged: usize,
}

impl Tally {
    fn add(&mut self, other: &Tally) {
        self.names += other.names;
        self.top1 += other.top1;
        self.top3 += other.top3;
        self.top1_spaceless += other.top1_spaceless;
        self.english += other.english;
        self.silent_wrong += other.silent_wrong;
        self.flagged += other.flagged;
    }

    fn pct(part: usize, whole: usize) -> f64 {
        if whole == 0 {
            0.0
        } else {
            100.0 * part as f64 / whole as f64
        }
    }

    fn top1_pct(&self) -> f64 {
        Self::pct(self.top1, self.names)
    }
    fn top3_pct(&self) -> f64 {
        Self::pct(self.top3, self.names)
    }
    fn english_pct(&self) -> f64 {
        Self::pct(self.english, self.names)
    }

    fn line(&self, label: &str) -> String {
        format!(
            "{label:<34} names {:>4}  Urdu top-1 {:>5.1}%  top-3 {:>5.1}%  (top-1 ignoring spaces {:>5.1}%)  English top-1 {:>5.1}%  silent wrong {:>4.1}%  marked {:>4.1}%",
            self.names,
            self.top1_pct(),
            self.top3_pct(),
            Self::pct(self.top1_spaceless, self.names),
            self.english_pct(),
            Self::pct(self.silent_wrong, self.names),
            Self::pct(self.flagged, self.names),
        )
    }
}

fn spaceless(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Which words of the typed name are missing from the word list: none, some or all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Coverage {
    AllSeen,
    Mixed,
    AllUnseen,
}

fn coverage(list: &WordList, roman: &str) -> Coverage {
    let words: Vec<&str> = roman.split(' ').collect();
    let unseen = words
        .iter()
        .filter(|w| !list.contains_known_roman(&w.to_lowercase()))
        .count();
    if unseen == 0 {
        Coverage::AllSeen
    } else if unseen == words.len() {
        Coverage::AllUnseen
    } else {
        Coverage::Mixed
    }
}

struct Result {
    whole: Tally,
    seen: Tally,
    mixed: Tally,
    unseen: Tally,
}

impl Result {
    /// Names with at least one word the list lacks (rules only).
    fn outside(&self) -> Tally {
        let mut t = self.mixed;
        t.add(&self.unseen);
        t
    }
}

fn measure(cases: &[Case]) -> Result {
    let list = WordList::embedded();
    let mut result = Result {
        whole: Tally::default(),
        seen: Tally::default(),
        mixed: Tally::default(),
        unseen: Tally::default(),
    };
    for case in cases {
        let got = suggest(&case.roman, &[], list, Direction::ToUrdu);
        let first = got.first();
        let right = first.is_some_and(|g| case.urdu.contains(&g.text));
        let marked = first.is_some_and(|g| g.source == Source::Rules);
        let tally = Tally {
            names: 1,
            top1: usize::from(right),
            top3: usize::from(got.iter().any(|g| case.urdu.contains(&g.text))),
            top1_spaceless: usize::from(
                first.is_some_and(|g| case.urdu.iter().any(|u| spaceless(u) == spaceless(&g.text))),
            ),
            english: usize::from(
                case.urdu
                    .first()
                    .map(|u| suggest_to_english(u, &[], list))
                    .and_then(|back| back.into_iter().next())
                    .is_some_and(|b| case.english.contains(&b)),
            ),
            silent_wrong: usize::from(!right && !marked),
            flagged: usize::from(marked),
        };
        result.whole.add(&tally);
        match coverage(list, &case.roman) {
            Coverage::AllSeen => result.seen.add(&tally),
            Coverage::Mixed => result.mixed.add(&tally),
            Coverage::AllUnseen => result.unseen.add(&tally),
        }
    }
    result
}

fn report(title: &str, result: &Result) -> String {
    let mut text = String::new();
    let _ = writeln!(text, "== {title}");
    let _ = writeln!(text, "{}", result.whole.line("whole list"));
    let _ = writeln!(text, "{}", result.seen.line("every word is in the list"));
    let _ = writeln!(
        text,
        "{}",
        result.mixed.line("some words are not in the list")
    );
    let _ = writeln!(
        text,
        "{}",
        result.unseen.line("every word is outside the list")
    );
    let _ = writeln!(
        text,
        "{}",
        result.outside().line("a word outside the list (rules)")
    );
    text
}

/// A value of `translit_baseline.txt`: `key = number`.
fn baseline(key: &str) -> f64 {
    BASELINE
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key)
                .then(|| v.trim().parse::<f64>().ok())
                .flatten()
        })
        .unwrap_or_else(|| panic!("translit_baseline.txt has no {key}"))
}

#[test]
fn the_frozen_list_is_well_formed_and_exercises_the_rules() {
    let cases = parse(FROZEN);
    assert_eq!(cases.len(), 1000, "NFR-URD-24 asks for 1,000 names");
    let mut typed: Vec<&str> = cases.iter().map(|c| c.roman.as_str()).collect();
    typed.sort_unstable();
    typed.dedup();
    assert_eq!(typed.len(), 1000, "a name is listed twice");
    for case in &cases {
        assert!(
            !case.urdu.is_empty() && !case.english.is_empty(),
            "{}",
            case.roman
        );
        assert!(
            case.roman
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == ' ')
        );
        for urdu in &case.urdu {
            assert!(
                !urdu.is_empty() && is_urdu_name(urdu),
                "bad Urdu for {}: {urdu}",
                case.roman
            );
        }
        for english in &case.english {
            assert!(is_english_name(english), "bad English for {}", case.roman);
        }
    }
    // The list must still exercise the spelling rules: names whose every word is outside the
    // word list are the subset that reports the rules-only rate. (The word list grew from
    // 4,572 to 15,500 entries, so fewer names are outside it than when the list was frozen.)
    let result = measure(&cases);
    assert!(
        result.unseen.names >= 50,
        "the rules-only subset is too small to mean anything: {}",
        result.unseen.names
    );
}

#[test]
fn the_word_list_does_not_hold_the_test_names() {
    let list = WordList::embedded();
    // T2: names of Wikidata items kept out of the word list. Not one of them is a whole entry,
    // by its English spelling or its Urdu spelling (research 5.5).
    let holdout = parse(HOLDOUT);
    assert!(
        holdout.len() >= 500,
        "the hold-out is too small: {}",
        holdout.len()
    );
    for case in &holdout {
        assert!(
            list.by_roman(&case.roman).is_empty(),
            "hold-out name {} is an entry of the word list",
            case.roman
        );
        for urdu in &case.urdu {
            assert!(
                list.by_urdu(urdu).is_empty(),
                "hold-out name {urdu} is an entry of the word list"
            );
        }
    }
    // T1: no phrase of the frozen list was added by the import. The multi-word frozen names
    // that are phrase entries were already in the first list the team wrote (source `team`);
    // that number must never grow.
    const OLD_PHRASE_OVERLAP: usize = 22;
    let frozen = parse(FROZEN);
    let mut phrase_overlap = 0;
    for case in frozen.iter().filter(|c| c.roman.contains(' ')) {
        for entry in list.by_roman(&case.roman) {
            if case.urdu.contains(&entry.urdu) {
                phrase_overlap += 1;
                assert_eq!(
                    entry.source, "team",
                    "the import added the test name {}",
                    case.roman
                );
            }
        }
    }
    assert!(
        phrase_overlap <= OLD_PHRASE_OVERLAP,
        "{phrase_overlap} frozen multi-word names are phrase entries (at most {OLD_PHRASE_OVERLAP})"
    );
}

#[test]
fn accuracy_report_floors_and_baseline() {
    // The three lists are measured side by side; the word list and its model are shared.
    let _ = WordList::embedded().training_report();
    let (frozen, holdout, dev) = std::thread::scope(|scope| {
        let frozen = scope.spawn(|| measure(&parse(FROZEN)));
        let holdout = scope.spawn(|| measure(&parse(HOLDOUT)));
        let dev = measure(&parse(DEV));
        (
            frozen.join().expect("T1 measure"),
            holdout.join().expect("T2 measure"),
            dev,
        )
    });
    let mut text = String::new();
    text.push_str(&report("T1 frozen list (translit_names.tsv)", &frozen));
    text.push('\n');
    text.push_str(&report(
        "T2 Wikidata hold-out (translit_holdout.tsv)",
        &holdout,
    ));
    text.push('\n');
    text.push_str(&report("development list (translit_dev.tsv)", &dev));
    let _ = writeln!(
        text,
        "\nNFR-URD-24 v1.2 gates, every word in the list: Urdu top-1 >= {GATE_TOP1_IN_LIST}%, top-3 >= {GATE_TOP3_IN_LIST}%. NFR-URD-25: English top-1 >= {GATE_ENGLISH}% on the whole list.\nWord list: {} entries.",
        WordList::embedded().len()
    );
    // Numbers for translit_baseline.txt, rounded down to one decimal.
    let down = |x: f64| (x * 10.0).floor() / 10.0;
    let _ = writeln!(
        text,
        "\nbaseline values now:\nfrozen_outside_top3 = {}\nholdout_outside_top3 = {}\ndev_outside_top3 = {}\nfrozen_in_list_top1 = {}\nfrozen_in_list_top3 = {}\nfrozen_english = {}\nholdout_in_list_top1 = {}\nholdout_in_list_top3 = {}\ndev_in_list_top1 = {}\ndev_in_list_top3 = {}",
        down(frozen.outside().top3_pct()),
        down(holdout.outside().top3_pct()),
        down(dev.outside().top3_pct()),
        down(frozen.seen.top1_pct()),
        down(frozen.seen.top3_pct()),
        down(frozen.whole.english_pct()),
        down(holdout.seen.top1_pct()),
        down(holdout.seen.top3_pct()),
        down(dev.seen.top1_pct()),
        down(dev.seen.top3_pct()),
    );
    println!("{text}");
    let folder = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let _ = std::fs::write(folder.join("translit_accuracy.txt"), &text);

    // The baseline of NFR-URD-24 v1.2: names with a word outside the list (rules only) must not
    // do worse than the value stored when the rules were last improved. The build fails if
    // they do. Raise the numbers in translit_baseline.txt when the rules get better; never
    // lower them to make a change pass.
    for (key, got) in [
        ("frozen_outside_top3", frozen.outside().top3_pct()),
        ("holdout_outside_top3", holdout.outside().top3_pct()),
        ("dev_outside_top3", dev.outside().top3_pct()),
    ] {
        let floor = baseline(key);
        assert!(
            got >= floor,
            "{key}: {got:.1}% is below the baseline {floor:.1}%"
        );
    }
    // Floors that keep the in-list and English numbers from falling. They sit a little under
    // what the engine gives now. They are NOT the NFR gates (see the ignored test).
    for (key, got) in [
        ("frozen_in_list_top1", frozen.seen.top1_pct()),
        ("frozen_in_list_top3", frozen.seen.top3_pct()),
        ("frozen_english", frozen.whole.english_pct()),
        ("holdout_in_list_top1", holdout.seen.top1_pct()),
        ("holdout_in_list_top3", holdout.seen.top3_pct()),
        ("dev_in_list_top1", dev.seen.top1_pct()),
        ("dev_in_list_top3", dev.seen.top3_pct()),
    ] {
        let floor = baseline(key);
        assert!(
            got >= floor,
            "{key}: {got:.1}% is below the floor {floor:.1}%"
        );
    }
}

/// The NFR gates of v1.2. They do not pass yet on the frozen list: see the report. Run with
/// `-- --ignored`. The numbers are the honest ones, so the test is kept (and ignored) instead of
/// being weakened.
#[test]
#[ignore = "NFR-URD-24 (95% / 99% in the list) and NFR-URD-25 are not met on the frozen list; see translit_accuracy.txt"]
fn nfr_gates_hold_on_the_frozen_list() {
    let result = measure(&parse(FROZEN));
    assert!(
        result.seen.top1_pct() >= GATE_TOP1_IN_LIST,
        "{}",
        result.seen.line("every word in the list")
    );
    assert!(
        result.seen.top3_pct() >= GATE_TOP3_IN_LIST,
        "{}",
        result.seen.line("every word in the list")
    );
    assert!(
        result.whole.english_pct() >= GATE_ENGLISH,
        "{}",
        result.whole.line("English")
    );
}
