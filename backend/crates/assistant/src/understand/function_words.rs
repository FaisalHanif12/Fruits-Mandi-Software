//! The closed classes of everyday words that no figure, filter or name hangs on (D7, plan 59 F3b).
//!
//! A word the app does not know switches off instant answers, because an unknown word could be a filter
//! (an item, an expense head, a group, a packing, a party) that the figure would then ignore. That is
//! right for a noun. It is wrong for the small words every question has: pronouns ("humne"),
//! postpositions ("andar"), light verbs ("rehta", "nikalta"), time and manner words ("still", "needs",
//! "وقت"). They are a closed set, so the set is written here once, by word class, and the lexicon takes
//! it into its function words ([`all`]). It is a rule by class, never a list of the questions of one run.
//!
//! A word that is known here is a word the app no longer doubts, so a word may be here ONLY if no figure
//! changes with it. What is therefore NOT here (money review of 08-11-2026):
//! - a noun, or a word that names a figure or a filter (item, head, group, packing, party);
//! - a word that groups, sums or excludes ("har", "sab", "dono", "each", "both", "alawa", "siwa",
//!   "bagair", "without", "except") or gives a limit or a range ("upar", "neeche", "darmiyan", "over",
//!   "under", "between", "jitna");
//! - a place that is another filter ("bahar", "outside", "inside": the shop's own sales are not another
//!   shop's), or a habit ("usually", "aksar": a typical day is not the month);
//! - a verb that says which way money or goods moved: given, taken, paid, received, came in, went out,
//!   put in, withdrawn, got, made, "mila", "nikala", "aaya", "chukaya", "wapas". Those stay unknown to the
//!   lexicon (or known only as before) and stay the business of `reads::happened` and of the capital and
//!   cash plans;
//! - "aana" and "jaana" ("kitna aana hai" is what is to be received: the direction of money);
//! - any word that can be the name of a person, a town or a firm ([`NAMES`]).
//!
//! Every verb here is still an ACTION word for one purpose: a partner's total is never inferred beside it
//! ([`is_action_verb`]): "how much did partner b earn" asks his earnings, not what he has.
//!
//! Example: "hamare gahkon ke zimme kitna baqi rehta hai": every word is a cue or a function word, so the
//! dues are answered. "hamare gahkon ke zimme kitna baqi rehta hai chai par": "chai" is a noun nothing
//! explains, so the model answers.

use std::collections::HashSet;
use std::sync::OnceLock;

/// Pronouns, demonstratives and question words that are not a measure ("humne", "usne", "kahan").
#[rustfmt::skip]
const PRONOUNS: &[&str] = &[
    // Roman Urdu
    "humne", "hamne", "tumne", "usne", "unhone", "unhon", "unhein", "unhen", "inhein", "inhen", "maine", "mene",
    "usse", "isse", "unse", "inse", "mujh", "mujhse", "tujhe", "tum", "tu", "tumhe", "tumhein", "tumhara",
    "tumhari", "tumhare", "aapko", "aapse", "aapne", "apko", "humara", "humari", "humare", "humse", "hamse",
    "kisko", "kisse", "kiska", "kiski", "kiske", "kahan", "kidhar", "idhar", "yahan", "yaha", "wahan", "waha",
    "jahan", "jab", "tab", "wahi", "yahi", "aisa", "aisi", "aise", "waisa", "waisi", "waise", "kaisa", "kaisi",
    "kaise", "khud", "apna", "apni", "apne", "kaha", "kyunke", "kyunki", "isliye", "islye", "jabke", "halanke",
    "chunke", "taake", "ya", "balke", "warna",
    // Urdu letters
    "ہمنے", "تمنے", "اسنے", "اس نے", "ہم نے", "تم", "تمہارا", "تمہاری", "تمہارے", "انہوں", "انہوں نے", "انہیں",
    "انھیں", "اسے", "آپکو", "آپ کو", "آپ نے", "کہاں", "کدھر", "ادھر", "یہاں", "وہاں", "جہاں", "جب", "تب",
    "وہی", "یہی", "ایسا", "ایسی", "ایسے", "ویسا", "ویسی", "ویسے", "کیسا", "کیسی", "کیسے", "خود", "کیونکہ",
    "اسلیے", "اس لیے", "جبکہ", "حالانکہ", "چونکہ", "تاکہ", "بلکہ", "ورنہ", "ہمارے", "ہمارا", "ہماری", "ہمیں",
    "جو", "جس", "جن", "جنہوں",
    // English
    "anything", "something", "everything", "nothing", "anyone", "someone", "everyone", "anybody", "somebody",
    "everybody", "nobody", "whatever", "whoever", "whichever", "whenever", "wherever", "however", "how",
    "here", "there", "where", "whose", "ours", "mine", "yours", "theirs", "myself", "ones", "thing", "things",
    "people", "persons", "person", "folks", "guys",
];

/// Postpositions, prepositions and adverbs of manner and time ("andar", "still", "very", "وقت"). Not the
/// ones that limit, exclude, place or give a habit: see the header.
#[rustfmt::skip]
const PLACE_AND_TIME: &[&str] = &[
    // Roman Urdu
    "andar", "waqt", "wqt", "samay", "zaroor", "shayad", "bas", "bilkul", "khaas", "khas", "wagera", "waghera",
    "fir", "tak", "se", "ko", "ka", "ki", "ke", "mein", "par", "saath", "sath", "pas", "paas", "qareeb",
    "qarib", "dwara", "zariye", "zariya", "baray", "bare", "barey", "baarey", "mutaliq", "mutalliq", "yaani",
    "yani", "matlab", "jaise", "jese", "jaisa", "jaisi", "wese", "wesa",
    // Urdu letters
    "اندر", "وقت", "ضرور", "شاید", "بس", "بالکل", "خاص", "وغیرہ", "ساتھ", "پاس", "قریب", "ذریعے", "بارے",
    "متعلق", "یعنی", "مطلب", "جیسے", "جیسا", "جیسی", "ویسے",
    // English
    "still", "already", "yet", "even", "now", "then", "afterwards", "currently", "presently", "regarding",
    "concerning", "very", "too", "quite", "rather", "really", "actually", "exactly", "approximately", "almost",
    "nearly", "maybe", "perhaps", "probably", "please", "kindly", "but", "because", "though", "although",
    "while", "whether", "unless",
];

/// Light and helping verbs in Roman Urdu are a stem and an ending ("reh" + "ta" is "rehta"). The stems are
/// the verbs of ordinary questions that carry no figure and no direction of their own: stay, do, be, go on,
/// look, tell, write, hear, understand, ask, say, want, can, sit, rise, try, keep, fall. NOT give, take,
/// come, go, get, find, send, pay, sell, buy, bring back, take out, fill: those are directions or figures.
const ROMAN_STEMS: &[&str] = &[
    "kar", "ho", "reh", "rah", "chal", "dekh", "bata", "likh", "sun", "samajh", "pad", "uth",
    "baith", "aazma", "puch", "pooch", "kah", "keh", "chah", "chaah", "sak", "rakh",
];

/// The endings of a Roman Urdu verb: habitual, perfect, future, infinitive and the plain forms. An ending
/// that starts with a vowel and is longer than one letter is NOT here ("ban" + "ana" was "banana", the
/// fruit). "wala" is not here either ("kharid" + "wala" is a buyer).
const ROMAN_ENDINGS: &[&str] = &[
    "", "a", "i", "e", "ta", "ti", "te", "na", "ni", "ne", "ya", "yi", "ye", "ga", "gi", "ge",
    "oon", "un", "en", "ein", "iye", "hua", "hui", "hue", "kar", "ke",
];

/// Roman Urdu forms the product cannot make or must not: "to pay off" ("baqi chukana hai", what is still to
/// be paid: a state) and "comes out" ("kitna nikalta hai": what it comes to). Not "chukaya" (paid) and not
/// "nikala" (took out): those say which way money moved.
#[rustfmt::skip]
const ROMAN_EXTRA: &[&str] = &[
    "chukana", "chukani", "chukane", "nikalta", "nikalti", "nikalte", "lagaya", "lagayi", "lagaye", "lagai",
];

/// Words that relate a figure to a period or a limit when a period or a number is in the question
/// ("after 10 october", "since monday"), and are plain connectives when it has none ("buyers still owe how
/// much after paying"). They are function words only in a question with no period and no digit: see
/// [`conditional`].
const CONDITIONAL: &[&str] = &["after", "afterwards", "since", "until", "till", "during"];

/// Light and helping verbs and their forms in Urdu letters. Not the ones that give, take, receive, come,
/// go, buy, sell, count, or earn: those are directions or figures.
#[rustfmt::skip]
const URDU_VERBS: &[&str] = &[
    "کرتے", "کرتا", "کرتی", "کرنا", "کرنے", "کرنی", "کیا", "کیے", "کرکے", "رہتا", "رہتی", "رہتے", "رہنا", "رہنے",
    "رہی", "رہا", "رہے", "نکلتا", "نکلتی", "نکلتے", "رکھا", "رکھی", "رکھے", "رکھتے", "رکھتا", "رکھتی", "رکھنا",
    "رکھنے", "چکانا", "چکانی", "چکانے", "لگایا", "لگائی", "لگائے", "ہوتا", "ہوتی", "ہوتے", "ہونا", "ہونی", "ہونے", "ہوگا", "ہوگی", "ہوں",
    "چلتا", "چلتی", "چلتے", "چلنا", "چلنے", "دیکھنا", "دیکھنے", "دیکھیں", "دیکھو", "بولنا", "بتانا", "بتانے",
    "بتایا", "بتائی", "بتائے", "لکھنا", "سمجھنا", "سمجھ", "پوچھنا", "پوچھ", "کہنا", "کہنے", "کہا", "کہتے", "کہتا",
    "کہتی", "چاہتا", "چاہتی", "چاہتے", "چاہیے", "چاہئیے", "سکتا", "سکتی", "سکتے", "پڑتا", "پڑتی", "پڑتے", "پڑنا",
    "پڑنی", "پڑنے", "ہیں", "ہے", "تھا", "تھی", "تھے", "گا", "گی", "گے", "ہوئی", "ہوئے", "ہوا",
];

/// English verb forms that carry no figure and no direction of their own: be, have, do, need, want, wait,
/// show, keep, earn and the small verbs of a question. Not give, take, pay, get, make, put, come, go, send,
/// receive, return, collect, owe: those are directions.
#[rustfmt::skip]
const ENGLISH_VERBS: &[&str] = &[
    "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "having", "do", "does", "did",
    "doing", "done", "need", "needs", "needed", "needing", "want", "wants", "wanted", "wanting", "wait", "waits",
    "waited", "waiting", "show", "shows", "showed", "shown", "showing", "keep", "keeps", "kept", "keeping", "stay",
    "stays", "stayed", "staying", "exist", "exists", "become", "becomes", "became", "seem", "seems", "work",
    "works", "worked", "working", "use", "uses", "used", "using", "run", "runs", "ran", "running", "know",
    "knows", "knew", "known", "say", "says", "said", "saying", "mean", "means", "meant", "call", "calls",
    "called", "calling", "let", "lets", "see", "sees", "saw", "seen", "look", "looks", "looked", "looking",
    "find", "finds", "found", "finding", "ask", "asks", "asked", "asking", "answer", "answers", "earn", "earns",
    "earned", "earning", "earnings", "can", "could", "will", "would", "shall", "should", "may", "might", "must",
    "ought", "tell", "tells", "told", "telling",
];

/// Verbs of being and having, which say nothing of an action: "partner b ka kitna hai", "how much does he
/// have". Every other verb word of the sets above is an ACTION verb ([`is_action_verb`]).
#[rustfmt::skip]
const COPULAS: &[&str] = &[
    "is", "are", "am", "was", "were", "be", "been", "being", "have", "has", "had", "having", "do", "does", "did",
    "hai", "he", "hy", "hain", "hen", "hoon", "hun", "tha", "thi", "the", "thay", "ho", "hoga", "hogi", "hoge",
    "hota", "hoti", "hote", "hua", "hui", "hue", "ہے", "ہیں", "تھا", "تھی", "تھے", "ہو", "ہوں", "ہوگا", "ہوگی",
    "ہوتا", "ہوتی", "ہوتے", "ہوا", "ہوئی", "ہوئے", "گا", "گی", "گے", "ga", "gi", "ge",
];

/// The verbs that say that money or goods MOVED (an event of a period): given, taken, paid, received, came in,
/// went out, put in, withdrawn. They are NOT in [`all`] (they stay as unknown as they were, or known only
/// through the old function words). `reads::happened` refuses a state figure (a balance, the dues, the cash
/// in hand, a capital) beside one ([`is_event_word`]), and they are action verbs for [`is_action_verb`].
#[rustfmt::skip]
const EVENT: &[&str] = &[
    // English
    "paid", "pay", "pays", "paying", "gave", "give", "gives", "giving", "given", "took", "take", "takes",
    "taking", "taken", "got", "get", "gets", "getting", "received", "receive", "receives", "receiving",
    "came", "come", "comes", "coming", "went", "go", "goes", "going", "gone", "spent", "spend", "spends",
    "spending", "put", "putting", "sent", "send", "sends", "sending", "withdrew", "withdrawn", "withdraw",
    "withdrawing", "returned", "return", "returns", "returning", "deposited", "deposit", "collected",
    "collect", "collects", "collecting", "invested", "invest", "made", "make", "makes", "making",
    // Roman Urdu
    "diya", "di", "diye", "dia", "dee", "li", "liya", "liye", "lia", "nikala", "nikali", "nikale", "nikla",
    "nikli", "nikle", "aya", "aaya", "ayi", "aayi", "aye", "aaye", "bheja", "bheji", "bheje", "daala", "dala",
    "daali", "dali", "wapas", "wapsi", "wapis", "kamaya", "kamai", "mila", "mili", "mile", "chukaya",
    "chukayi", "chukaye", "chukai", "bikwaya", "gaya", "gayi", "gaye", "kiya", "kia", "kiye", "nikalwaya",
    "lautaya", "lautai",
    // Urdu letters
    "دیا", "دی", "دیے", "لیا", "لی", "لیے", "نکالا", "نکالی", "نکالے", "نکلا", "نکلی", "نکلے", "آیا", "آئی",
    "آئے", "بھیجا", "بھیجی", "بھیجے", "ڈالا", "ڈالی", "ڈالے", "واپس", "واپسی", "کمایا", "کمائی", "ملا",
    "ملی", "ملے", "چکایا", "چکائی", "چکائے", "گیا", "گئی", "گئے",
];

/// Verbs that name a direction of money in a STATE ("kitna dena hai" is what is to be paid, "kitna milta
/// hai" what is received). Not events, so `reads::happened` leaves them alone; but a partner's total is
/// never inferred beside one ([`is_action_verb`]).
#[rustfmt::skip]
const DIRECTION: &[&str] = &[
    "milta", "milti", "milte", "kamata", "kamati", "kamate", "dena", "dene", "deni", "lena", "lene", "leni",
    "aana", "aane", "aani", "jana", "jane", "jani", "le", "de", "bhej", "nikal", "kamao", "bhar", "bhara",
    "bhari", "bhare", "owe", "owed", "owes", "lent", "lend", "borrow", "borrowed", "کماتا", "کماتی", "کماتے",
    "ملتا", "ملتی", "ملتے", "دینا", "دینے", "دینی", "لینا", "لینے", "لینی", "آنا", "آنے", "آنی", "جانا", "جانے",
    "جانی", "بھرا", "بھری", "بھرے",
];

/// Words that limit, exclude, place or give a habit: "kele ke alawa", "is mahine bahar", "usually", "10,000
/// se upar", "without banana". No plan reads one, so a question that has one is never answered by the app
/// alone, whatever else is in it (it is as good as an unknown word, which a near spelling must not hide).
#[rustfmt::skip]
const UNREAD_FILTERS: &[&str] = &[
    "bahar", "باہر", "outside", "inside", "alawa", "ilawa", "siwa", "siwae", "علاوہ", "سوا", "bagair", "baghair",
    "بغیر", "without", "except", "excluding", "besides", "upar", "neeche", "niche", "اوپر", "نیچے", "over", "under",
    "above", "below", "between", "darmiyan", "beech", "درمیان", "بیچ", "usually", "normally", "often", "always",
    "never", "aksar", "hamesha", "kabhi", "اکثر", "ہمیشہ", "کبھی", "jitna", "jitni", "jitne", "utna", "utni",
    "utne", "جتنا", "جتنی", "جتنے", "اتنا", "اتنی", "اتنے", "again", "dobara", "دوبارہ",
];

/// Made forms that are also the name of a person, a town or a firm: they are searched as names, not skipped
/// ("rakh" + "i" is "rakhi", a woman's name; "kah" + "na" is Kahna, a town; "sak" + "i" is "saki"). A unit
/// test lists the fruits, vegetables, roles, towns and names that must stay out.
const NAMES: &[&str] = &[
    "rakhi", "kahna", "saki", "rahi", "chali", "pada", "sune", "suna",
];

/// Every neutral function word of the three scripts, with the Roman verb forms made from their stems.
///
/// The result is the set the lexicon adds to its own function words: a word in it is known, so it neither
/// starts a name nor switches off a total, and the app does not doubt it. Keep it short of any word that
/// changes a figure (see the header).
pub(super) fn all() -> &'static HashSet<String> {
    static ALL: OnceLock<HashSet<String>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut set: HashSet<String> = HashSet::new();
        for list in [
            PRONOUNS,
            PLACE_AND_TIME,
            URDU_VERBS,
            ENGLISH_VERBS,
            ROMAN_EXTRA,
        ] {
            for word in list {
                for part in word.split_whitespace() {
                    set.insert(super::lexicon::key(part));
                }
            }
        }
        for form in roman_verbs() {
            set.insert(form);
        }
        set
    })
}

/// True for a word that is a plain connective only when the question has no period and no digit.
pub(super) fn conditional(word: &str) -> bool {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    SET.get_or_init(|| CONDITIONAL.iter().map(|w| super::lexicon::key(w)).collect())
        .contains(word)
}

/// The Roman Urdu verb forms: each stem with each ending.
fn roman_verbs() -> Vec<String> {
    let mut out = Vec::with_capacity(ROMAN_STEMS.len().saturating_mul(ROMAN_ENDINGS.len()));
    for stem in ROMAN_STEMS {
        for ending in ROMAN_ENDINGS {
            let form = format!("{stem}{ending}");
            if !NAMES.contains(&form.as_str()) {
                out.push(form);
            }
        }
    }
    out
}

/// True for any verb that is not a verb of being or having: the light verbs of [`all`], the English verbs
/// and every verb of [`EVENT`]. A figure of NOW inferred from a name and a "how much" (a partner's total)
/// is never inferred beside one: "partner b ne kitna nikala", "how much did partner b earn", "partner b
/// ko kitna milta hai", "partner b kitna le gaya" each ask something else than what he has.
pub(super) fn is_action_verb(word: &str) -> bool {
    static ACTIONS: OnceLock<HashSet<String>> = OnceLock::new();
    let actions = ACTIONS.get_or_init(|| {
        let copulas: HashSet<String> = COPULAS.iter().map(|w| super::lexicon::key(w)).collect();
        let mut set: HashSet<String> = HashSet::new();
        for list in [URDU_VERBS, ENGLISH_VERBS, ROMAN_EXTRA, EVENT, DIRECTION] {
            for word in list {
                for part in word.split_whitespace() {
                    set.insert(super::lexicon::key(part));
                }
            }
        }
        for form in roman_verbs() {
            set.insert(form);
        }
        set.retain(|w| !copulas.contains(w));
        set
    });
    actions.contains(word)
}

/// True for a verb that says money or goods moved (an event of a period): "paid", "nikala", "came",
/// "wapas". A balance, the dues, the cash in hand or a capital is not asked beside one.
pub(super) fn is_event_word(word: &str) -> bool {
    static EVENTS: OnceLock<HashSet<String>> = OnceLock::new();
    EVENTS
        .get_or_init(|| EVENT.iter().map(|w| super::lexicon::key(w)).collect())
        .contains(word)
}

/// The question words that ask for something other than a figure: when, who, whom, which, where, why,
/// how (the way). A "how much" or "how many" is a cue of its own and is not here. "kisi" (any, someone)
/// is not a question word.
///
/// Example: "akram ka bakaya kitna hai aur kab se" asks a date as well as a figure.
#[rustfmt::skip]
const INTERROGATIVES: &[&str] = &[
    // Roman Urdu
    "kab", "kaun", "koun", "kon", "kaunsa", "kaunsi", "kaunse", "konsa", "konsi", "konse", "kounsa", "kounsi",
    "kounse", "konsy", "kis", "kisko", "kisse", "kiska", "kiski", "kiske", "kisne", "kin", "kinhon", "kinhone",
    "kinka", "kinki", "kinke", "kinko", "kinse", "kahan", "kahaan", "kidhar", "kidher", "kyun", "kyon", "kyu",
    "kiyon", "kiyun", "kyoon", "kaise", "kaisa", "kaisi", "kese", "kesay", "kaisay", "kaisey", "kesey", "kyse",
    "kiase",
    // Urdu letters
    "کب", "کون", "کونسا", "کونسی", "کونسے", "کس", "کسے", "کسکو", "کسنے", "کن", "کنہوں", "کہاں", "کدھر", "کیوں",
    "کیسے", "کیسا", "کیسی",
    // English
    "when", "who", "whom", "whose", "which", "where", "why",
];

/// The words of a rate or a price: "kis rate par diya" asks the rate, which is the figure, so the "kis" of it is
/// no question word.
#[rustfmt::skip]
const RATE_WORDS: &[&str] = &[
    "rate", "rates", "bhao", "bhav", "bhaao", "bhaw", "price", "prices", "qeemat", "keemat", "daam", "dam",
    "ریٹ", "بھاؤ", "بھاو", "قیمت", "دام",
];

/// Words that follow "how many" and make it a duration or a count of times ("kitne din", "kitni baar").
#[rustfmt::skip]
const SPAN_WORDS: &[&str] = &[
    "din", "dino", "roz", "hafte", "haftay", "mahine", "mahinay", "saal", "baar", "bar", "dafa", "dafah",
    "martaba", "arse", "arsa", "purana", "purani", "purane", "دن", "ہفتے", "مہینے", "سال", "بار", "دفعہ",
    "مرتبہ", "پرانا", "پرانی", "پرانے", "days", "weeks", "months", "years", "times",
];

/// Negations: "aaj kitna maal nahi bika" asks what did NOT sell, and the plain figure of the day is the opposite
/// of it. No figure is read beside one.
#[rustfmt::skip]
const NEGATIONS: &[&str] = &[
    "nahi", "nahin", "nahee", "nhi", "nehi", "not", "never", "نہیں", "نہ",
];

/// The words that ask for a date ("akram ki aakhri wasooli ki tareekh"), when no period holds them.
#[rustfmt::skip]
const DATE_WORDS: &[&str] = &["tareekh", "tarikh", "tareekhein", "date", "dates", "تاریخ", "تاریخیں"];

fn key_set(list: &'static [&'static str]) -> HashSet<String> {
    list.iter().map(|w| super::lexicon::key(w)).collect()
}

/// True for a question word that asks a date, a person, a place, a reason or a way (see [`INTERROGATIVES`]):
/// a figure alone leaves that ask unanswered.
pub(super) fn is_interrogative(word: &str) -> bool {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    SET.get_or_init(|| key_set(INTERROGATIVES)).contains(word)
}

/// True when the words of a question ask something that a figure alone leaves out: a question word, a
/// duration or a count of times ("kitne din se", "kitni baar", "kitna purana", "how long", "how often",
/// "what date"), a date word, a negation ("nahi", "not"). The words inside the spans (the names that were found
/// and the periods) are not read. "kis rate par" and
/// "kis qadar" (how much) are not asks of this kind.
///
/// Example: "akram ka bakaya kitna hai aur kab se" is true; "akram ko kela aakhri kis rate pe diya" is false.
pub(super) fn asks_beyond_a_figure(words: &[String], skip: &[(usize, usize)]) -> bool {
    static RATE: OnceLock<HashSet<String>> = OnceLock::new();
    static SPAN: OnceLock<HashSet<String>> = OnceLock::new();
    static NEG: OnceLock<HashSet<String>> = OnceLock::new();
    static DATE: OnceLock<HashSet<String>> = OnceLock::new();
    let rate = RATE.get_or_init(|| key_set(RATE_WORDS));
    let span = SPAN.get_or_init(|| key_set(SPAN_WORDS));
    let negation = NEG.get_or_init(|| key_set(NEGATIONS));
    let date = DATE.get_or_init(|| key_set(DATE_WORDS));
    let how_many = |w: &str| {
        ["kitna", "kitne", "kitni", "کتنا", "کتنے", "کتنی"]
            .iter()
            .any(|x| super::lexicon::key(x) == w)
    };
    let kis = |w: &str| {
        ["kis", "which", "kaunsa", "konsa", "کس", "کونسا"]
            .iter()
            .any(|x| super::lexicon::key(x) == w)
    };
    for (at, word) in words.iter().enumerate() {
        if skip.iter().any(|(s, e)| *s <= at && at < *e) {
            continue;
        }
        let word = word.as_str();
        let next = words.get(at.saturating_add(1)).map(String::as_str);
        if negation.contains(word) || date.contains(word) {
            return true;
        }
        // "how many days", "how many times".
        if word == "many"
            && at
                .checked_sub(1)
                .and_then(|b| words.get(b))
                .is_some_and(|b| b == "how")
            && next.is_some_and(|n| span.contains(n))
        {
            return true;
        }
        if is_interrogative(word) {
            let a_rate = kis(word) && next.is_some_and(|n| rate.contains(n));
            let how_much = kis(word)
                && next.is_some_and(|n| {
                    n == super::lexicon::key("qadar") || n == super::lexicon::key("قدر")
                });
            if !(a_rate || how_much) {
                return true;
            }
            continue;
        }
        if how_many(word) && next.is_some_and(|n| span.contains(n)) {
            return true;
        }
        let pair = |a: &str, b: &[&str]| word == a && next.is_some_and(|n| b.contains(&n));
        if pair("how", &["long", "often", "old"]) || pair("what", &["date", "day", "time"]) {
            return true;
        }
    }
    false
}

/// True when a word of the question limits, excludes, places or gives a habit, which no plan reads.
pub(super) fn is_unread_filter(word: &str) -> bool {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    SET.get_or_init(|| {
        UNREAD_FILTERS
            .iter()
            .map(|w| super::lexicon::key(w))
            .collect()
    })
    .contains(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asks(question: &str) -> bool {
        let words: Vec<String> = question
            .split_whitespace()
            .map(crate::understand::lexicon::key)
            .collect();
        asks_beyond_a_figure(&words, &[])
    }

    #[test]
    fn question_words_that_ask_more_than_a_figure_are_found_in_three_scripts() {
        for q in [
            "bakaya kitna hai aur kab se",
            "dead amount kitna hai aur kin logon ka",
            "dead amount kitna hai aur kis kis ka",
            "aaj ki bikri kitni hui aur kahan se",
            "aaj ki bikri kitni hui aur kahaan se",
            "bikri kitni hui kese",
            "aaj kitni baar bikri hui",
            "akram ka bakaya kitne din se hai",
            "akram ka bakaya kitna purana hai",
            "how long has akram owed",
            "how often did akram pay",
            "what date did he pay",
            "who owes the most",
            "which buyer owes",
            "why is it low",
            "اکرم کا بقایا کتنا ہے اور کب سے",
            "ڈوبی رقم کتنی ہے اور کن کن کی",
            "aaj kitna maal nahi bika",
            "how many days has akram owed",
            "how many times did he pay",
            "akram ka bakaya aur aakhri wasooli ki tareekh",
            "آج کتنی وصولی نہیں ہوئی",
            "آج کتنی بار بکری ہوئی",
            "کس کا بقایا کتنا ہے",
        ] {
            assert!(asks(q), "{q}");
        }
        // How much, what, any, the rate or the price, "kis qadar" and the nouns of a figure are not asks.
        for q in [
            "akram ka bakaya kitna hai",
            "kya hai aaj ki bikri",
            "kisi ka bakaya",
            "what is the cash in hand",
            "how much is the dead amount",
            "how many buyers owe",
            "akram ko kela aakhri kis rate pe diya",
            "kele ka last rate kis bhao tha",
            "kis qadar bikri hui",
            "which rate did akram get",
            "کس ریٹ پر دیا",
            "کس قدر بکری ہوئی",
            "بقایا کتنا ہے",
        ] {
            assert!(!asks(q), "{q}");
        }
        // The words of a found name are not read.
        let words: Vec<String> = ["kaun", "traders", "ka", "bakaya"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        assert!(!asks_beyond_a_figure(&words, &[(0, 2)]));
    }

    #[test]
    fn ordinary_words_of_three_scripts_are_function_words() {
        let all = all();
        for word in [
            "humne",
            "andar",
            "rehta",
            "nikalta",
            "rakha",
            "chukana",
            "still",
            "needs",
            "earned",
            "waiting",
            "have",
            "anything",
            "people",
            "وقت",
            "ہمارے",
            "نکلتا",
            "اندر",
            "کرتے",
        ] {
            let key = crate::understand::lexicon::key(word);
            assert!(all.contains(&key), "{word}");
        }
        // "kartay" is not made: only the endings of the language are.
        assert!(!all.contains("kartay"));
    }

    #[test]
    fn a_word_that_relates_to_a_period_is_a_connective_only_when_asked_so() {
        // "after" is not in the set: `unexplained_words` takes it only when the question has no period.
        let all = all();
        assert!(!all.contains("after"));
        assert!(conditional("after"));
        assert!(conditional("since"));
        assert!(!conditional("still"));
    }

    #[test]
    fn nouns_filters_exclusions_places_habits_and_directions_are_not_function_words() {
        let all = all();
        for word in [
            // nouns and filters
            "chai",
            "kiraya",
            "aam",
            "kela",
            "peti",
            "bijli",
            "akram",
            "bano",
            "ghar",
            "rate",
            "bikri",
            "commission",
            "tamatar",
            "tea",
            "total",
            "together",
            "kul",
            // groups, sums, exclusions, limits
            "har",
            "sab",
            "dono",
            "each",
            "both",
            "alawa",
            "ilawa",
            "siwa",
            "bagair",
            "baghair",
            "without",
            "except",
            "upar",
            "neeche",
            "over",
            "under",
            "between",
            "darmiyan",
            "jitna",
            "utna",
            "علاوہ",
            "سوا",
            "بغیر",
            "اوپر",
            "نیچے",
            // another place, a habit
            "bahar",
            "باہر",
            "outside",
            "inside",
            "usually",
            "normally",
            "aksar",
            "often",
            "always",
            "kabhi",
            "hamesha",
            "again",
            // directions of money or goods
            "paid",
            "took",
            "came",
            "got",
            "made",
            "nikala",
            "nikali",
            "aaya",
            "chukaya",
            "wapas",
            "mila",
            "aana",
            "aani",
            "jana",
            "dena",
            "lena",
            "le",
            "de",
            "diya",
            "liya",
            "نکالا",
            "واپس",
            "آنی",
            "ملا",
            "ملتا",
            "کمایا",
        ] {
            assert!(
                !all.contains(&crate::understand::lexicon::key(word)),
                "{word} must stay out of the function words"
            );
        }
    }

    #[test]
    fn no_made_verb_form_is_the_name_of_an_item_or_a_person() {
        // The product of stems and endings must never make the word of a fruit, a vegetable, a packing, a
        // role, a town or a name the books may hold ("ban" + "ana" was "banana"): those are searched, not
        // skipped.
        let all = all();
        for word in [
            "banana",
            "kela",
            "seb",
            "apple",
            "aam",
            "mango",
            "angoor",
            "grapes",
            "aloo",
            "potato",
            "pyaz",
            "onion",
            "tamatar",
            "tomato",
            "moosami",
            "mosambi",
            "kinnu",
            "malta",
            "orange",
            "amrood",
            "anar",
            "papita",
            "gobhi",
            "mooli",
            "radish",
            "saag",
            "palak",
            "adrak",
            "lehsan",
            "nimbu",
            "kheera",
            "mirch",
            "bhindi",
            "karela",
            "tinda",
            "tori",
            "lauki",
            "matar",
            "gajar",
            "shalgam",
            "chiku",
            "falsa",
            "jamun",
            "khajoor",
            "tarbooz",
            "kharbooza",
            "makai",
            "peti",
            "boriyan",
            "crate",
            "box",
            "gahak",
            "gahakon",
            "byopari",
            "banya",
            "boli",
            "ginti",
            "bano",
            "rakhi",
            "rani",
            "hina",
            "kausar",
            "rubina",
            "shahid",
            "nadeem",
            "akram",
            "rahim",
            "karim",
            "bashir",
            "imran",
            "tariq",
            "naseer",
            "saleem",
            "usman",
            "zahid",
            "faisal",
            "bilal",
            "danish",
            "kashif",
            "naveed",
            "waqas",
            "rizwan",
            "arshad",
            "aslam",
            "sarfraz",
            "tahir",
            "jameel",
            "haji",
            "malik",
            "manga",
            "kahna",
            "deen",
            "saki",
            "rahi",
            "mandi",
            "kasur",
            "lahore",
            "okara",
            "pakpattan",
            "sahiwal",
            "chunian",
            "kot",
            "radha",
            "kishan",
            "pattoki",
            "multan",
        ] {
            assert!(
                !all.contains(&crate::understand::lexicon::key(word)),
                "{word} is made by a stem and an ending"
            );
        }
    }

    #[test]
    fn action_verbs_block_a_partner_total_and_the_verbs_of_being_do_not() {
        for word in [
            "paid",
            "took",
            "came",
            "nikala",
            "diya",
            "wapas",
            "earn",
            "earned",
            "made",
            "le",
            "milta",
            "kamata",
            "نکالا",
            "واپس",
            "کماتا",
            "ملتا",
            "rehta",
            "nikalta",
            "chukana",
        ] {
            assert!(
                is_action_verb(&crate::understand::lexicon::key(word)),
                "{word}"
            );
        }
        for word in [
            "hai", "hain", "tha", "is", "are", "have", "has", "ہے", "ہیں", "hota", "kitna",
            "partner",
        ] {
            assert!(
                !is_action_verb(&crate::understand::lexicon::key(word)),
                "{word}"
            );
        }
    }
}
