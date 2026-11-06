//! The words of the Owner's questions, in Roman Urdu, Urdu letters and English, with the spellings
//! people really type (plan 55 step 1, plan 54 Section 6).
//!
//! Every word or phrase belongs to one [`Cue`]. The table is normalised once (the same keys as the
//! search: lower case, one Urdu letter for each look-alike) and kept in a map, so finding the cues of
//! a question costs a map lookup for each word. A word the table does not hold but that is one or two
//! letters from a long word of it ("commision", "bakayaa") is a soft hit (`fuzzy`): the name finder
//! may still claim the word as a name first.
//!
//! No arithmetic on money or quantities happens here. PRD FR-7.76, FR-7.89.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use mandi_services::core::text;

/// What a word or phrase of a question can say. A word may carry several cues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Cue {
    // ---- what kind of question -------------------------------------------------------------
    /// print, پرنٹ, chhapo.
    Print,
    /// bill, slip, parchi: what is printed.
    PrintObject,
    /// nikalna, chahiye, banao: the verb that goes with a printed thing.
    PrintVerb,
    /// kholo, open, jao: open a screen.
    Open,
    /// dikhao: show (also the way to ask for data, so weak).
    Show,
    /// likh do, delete, save karo: an instruction to write.
    WriteStrong,
    /// jama karo, add karo, de do: an instruction to write only beside an amount.
    WriteWeak,
    /// kaise, tareeqa, how to.
    How,
    /// kahan, where.
    Where,
    /// kya hota hai, what is, matlab.
    Define,
    /// "kya hai" (weak: also asks for a figure).
    DefineWeak,
    /// kya kya dekh sakte hain, what can I.
    Capability,
    /// barhayen, behtar, jaldi: a way to get better (advice when it asks how).
    Advice,
    /// mashwara, suggest, recommend: asks for advice by itself.
    AdviceStrong,
    /// kis se pehle, kisko udhar, kya karun: a decision to be advised on.
    Decision,
    /// andaza, forecast, agle mahine: ahead in time.
    Forecast,
    /// andaza, forecast, predict: asks for an estimate by itself.
    ForecastStrong,
    /// kitna, kitni, how much.
    HowMuch,
    /// kya kya, kaun kaun, list: many things.
    Many,
    /// sab se zyada, top, highest.
    Most,
    /// sab se kam, lowest.
    Least,
    // ---- joins -----------------------------------------------------------------------------
    /// aur, and, or.
    And,
    /// kul, total, overall.
    Total,
    // ---- who -------------------------------------------------------------------------------
    /// buyer, gahak.
    RoleBuyer,
    /// seller, byopari, zamindar.
    RoleSeller,
    /// partner, shareek.
    RolePartner,
    /// garden, bagh.
    RoleGarden,
    /// trip, safar.
    RoleTrip,
    /// staff, munshi.
    RoleStaff,
    /// alag khata.
    RoleKhata,
    /// mehman, guests: the shop's guests (fruit taken for them, PRD 16.7). Not a name.
    Guests,
    /// item, qisam.
    RoleItem,
    /// buyers, gahakon (many of them).
    ManyBuyers,
    /// sellers, byopariyon.
    ManySellers,
    /// items, qismein.
    ManyItems,
    /// becha, sold, bika: the one named is a seller.
    Sold,
    /// khareeda, bought: the one named is a buyer.
    Bought,
    // ---- which figure ----------------------------------------------------------------------
    /// bakaya, balance, lena, dena.
    Balance,
    /// baqi, pending, outstanding (also remaining).
    Pending,
    /// bikri, sale, sold amount.
    Sale,
    /// A plural of bikri: bikriyan, sales (a count).
    SalePlural,
    /// commission.
    Commission,
    /// laga.
    Laga,
    /// dues, receivable: what buyers still owe.
    Dues,
    /// ugrahi, wasooli: collecting money (what is still to collect, or what was collected).
    Collection,
    /// hui, mili, received.
    Received,
    /// adaigi, payable.
    Payable,
    /// peshgi, advance.
    Advance,
    /// stock, baqi maal.
    Stock,
    /// maal, goods.
    Goods,
    /// peti, box, kg: a unit goods are counted in.
    Unit,
    /// paise, raqam, rupay: money.
    Money,
    /// pukhta khata: the one-time parties' account.
    Pukhta,
    /// cash, roqar, gala.
    Cash,
    /// sarmaya, capital.
    Capital,
    /// kul raqam, khalis, partner total.
    PartnerTotal,
    /// hissa, share.
    PartnerShare,
    /// munafa, profit, nuqsan.
    Profit,
    /// kharcha, expense.
    Expense,
    /// phal liya, fruit taken.
    FruitTaken,
    /// phal, fruit (the word only).
    FruitWord,
    /// sabzi, vegetables (the word only): the group of the vegetable items.
    VegetableWord,
    /// udhar raqam, money lent, qarz.
    MoneyLent,
    /// doobi raqam, dead amount.
    DeadAmount,
    /// rate, bhao, price.
    Rate,
    /// khulasa, din ka hisab.
    DaySummary,
    /// roznamcha, day book.
    DayBook,
    /// trial balance.
    TrialBalance,
    /// shehar wise, city wise: by city.
    ByCity,
    /// khata, hisab, ledger.
    Ledger,
    /// statement, fard.
    Statement,
    /// result, natija.
    ResultWord,
    /// li hui raqam, zaati kharcha, drawings: the money the partners took.
    PartnerTaken,
    /// The title of a report that no figure names (R3, R4, R7, R8, R9, R15 ...): see [`REPORT_NAMES`].
    ReportName,
    /// A question about the app's own state or lists (the rules, the attention list, the closed days,
    /// how many staff): see [`ASK_NAMES`].
    AskName,
    /// "kitne mahine", "how many months": the number of months something sold in.
    HowManyMonths,
    /// The name of a screen of the app (cash book, day close, gardens), for "open it": see
    /// [`SCREEN_NAMES`].
    ScreenName,
    /// The journal (جنرل) as the thing to write in: "journal", "jarnal", "general entry". Never the
    /// bare word "general", which is also part of names.
    Journal,
    /// The quality of a sale line: awwal, doyam, soyam, first quality, A grade, darja (PRD FR-12.42).
    Quality,
    /// The word "number" before a number ("bikri number 5"): see `reads::bikri_number`.
    NumberWord,
    /// A truck number written as the trucks are (LES-1234, LHR 0077): see `scan::truck_number`.
    TruckNo,
    // ---- outside the app -------------------------------------------------------------------
    /// news, cricket, a joke: things the app does not keep.
    External,
    /// weather, rain.
    ExternalWeather,
    /// GST, income tax, FBR.
    ExternalTax,
    /// a bank balance.
    ExternalBank,
    /// the dollar, gold, petrol, the stock market.
    ExternalPrices,
    /// A city or market outside the books (Lahore, Karachi).
    ExternalPlace,
    /// A market-wide price phrase: mandi ka rate, open market.
    MarketRate,
}

/// One cue found in a question: where it is, and whether it is a soft (misspelt) hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Hit {
    pub cue: Cue,
    /// Index of its first word.
    pub start: usize,
    /// Index after its last word.
    pub end: usize,
    /// True when the word was only close to the table's word.
    pub fuzzy: bool,
}

/// The table: a cue and its words and phrases (a space between the words of a phrase).
type Entry = (Cue, &'static [&'static str]);

#[rustfmt::skip]
const INTENT: &[Entry] = &[
    (
        Cue::Print,
        &[
            "print", "prints", "printout", "print out", "prnt", "prent", "printing", "پرنٹ",
            "پرنٹر", "chhapo", "chapo", "chhapna", "chapna", "chhapni", "chhap", "chap", "chhapai",
            "چھاپ", "چھاپو", "چھاپنا", "چھاپیں", "چھاپنی", "pdf",
        ],
    ),
    (
        Cue::PrintObject,
        &[
            "bill", "bills", "billing", "slip", "slips", "parchi", "parchee", "purchi", "پرچی",
            "بل", "بلز", "رسید", "raseed", "receipt slip", "voucher", "register", "registers", "رجسٹر",
        ],
    ),
    (
        Cue::PrintVerb,
        &[
            "nikalna", "nikalni", "nikalo", "nikal", "nikalein", "nikalen", "nikaal", "nikalwa",
            "nikalwana", "nikalun", "nikalu", "chahiye", "chahie", "chahiey", "chaiye", "banao",
            "banana hai", "banana he", "banana h", "bana do", "banado", "banani", "نکالنا", "نکالنی", "نکالو", "نکال", "نکالیں",
            "چاہیے", "بناؤ", "بنانا", "بنا دو", "بنانی",
        ],
    ),
    (
        Cue::Open,
        &[
            "kholo", "kholna", "kholein", "kholen", "kholiye", "khol do", "kholdo", "kholni",
            "open", "jao", "jana", "le jao", "lejao", "le chalo", "chalo", "goto", "go to",
            "navigate", "screen par", "screen pe", "page par", "page pe", "screen kholo", "take me to",
            "take me", "le chalein", "le chalen", "le chaliye", "le chalte", "لے چلیں", "لے چلیے",
            "لے چلئے", "لے چلتے", "کھولو",
            "کھولنا", "کھولیں", "کھولیے", "کھول دو", "جاؤ", "جانا", "لے جاؤ", "لے چلو", "چلو",
            "سکرین پر", "اسکرین پر",
        ],
    ),
    (
        Cue::Show,
        &[
            "dikhao", "dikha", "dikhayen", "dikhaye", "dikhaiye", "dikha do", "dikhado", "dikhana",
            "show", "show me", "display", "view", "دکھاؤ", "دکھا", "دکھائیں", "دکھائیے", "دکھا دو",
            "دکھانا",
        ],
    ),
    (
        Cue::WriteStrong,
        &[
            "likh do", "likhdo", "likho", "likh", "likh den", "likh dein", "likh dena",
            "likh dijiye", "likhna hai", "entry karo", "entry kar do", "entry kardo", "entry kar",
            "entry dal do", "entry daal do", "dal do", "daal do", "daldo", "daaldo", "save karo",
            "save kar do", "save kardo", "save kar", "delete", "cancel", "delete karo", "delete kar do",
            "remove", "hata do", "hatao", "hatado", "mita do", "mitao", "mitado", "cancel karo",
            "cancel kar do", "cancel kardo", "write", "record karo", "record kar do", "post karo",
            "enter karo", "enter kar do", "create", "banao account", "naya account banao",
            "indraaj karna hai", "indraaj karni hai", "indraj karna hai", "indraj karni hai",
            "indraaj karo", "indraj karo", "record the", "record a", "اندراج کرنی ہے",
            "اندراج کرنا ہے", "اندراج کرو", "اندراج کریں",
            "likhiye", "لکھ دو", "لکھو", "لکھ دیں", "لکھ دیجیے", "انٹری کرو", "انٹری کر دو",
            "ڈال دو", "ہٹا دو", "ہٹاؤ", "مٹا دو", "مٹاؤ", "منسوخ کرو", "منسوخ کر دو", "محفوظ کرو",
            "محفوظ کر دو",
        ],
    ),
    (
        Cue::WriteWeak,
        &[
            "jama karo", "jama kar do", "jama kardo", "jama kar", "jamaa karo", "add karo",
            "add kar do", "add kardo", "add kar", "add", "de do", "dedo", "de dein", "dijiye",
            "kar do", "kardo", "band karo", "band kijiye", "close karo", "close kar do", "بند کرو",
            "بند کر دو", "change karo", "change kar do", "update karo", "update kar do",
            "edit karo", "edit kar do", "badal do", "badlo", "change", "update", "edit", "pay",
            "transfer", "جمع کرو", "جمع کر دو", "شامل کرو", "شامل کر دو", "بدل دو", "بدلو", "دے دو",
            "کر دو",
        ],
    ),
    (
        Cue::How,
        &[
            "kaise", "kese", "kaisay", "kaisey", "kesay", "kesey", "kaisse", "kyse", "kiase",
            "tareeka", "tareeqa", "tareeqay", "tareeky", "tareeke", "tareqa", "tareqe", "tarika",
            "tariqa", "tarike", "tariqay", "tarikay", "tareeqon", "kis tarah", "kis tara", "how",
            "how to", "how do", "how can", "how should", "steps", "step by step", "procedure",
            "طریقہ", "طریقے", "طریقوں", "کیسے", "کس طرح", "کس طریقے",
        ],
    ),
    (
        Cue::Where,
        &[
            "kahan", "kahaan", "kidhar", "kidher", "kaha", "where", "کہاں", "کدھر", "کس جگہ",
        ],
    ),
    (
        Cue::Define,
        &[
            "kya hota hai", "kya hoti hai", "kya hote hain", "kya hota he", "kya hoti he",
            "kya hote hen", "kya hota", "kya hoti", "kya hote", "matlab", "mtlb", "meaning",
            "means", "mean", "what is a", "what is an", "what are", "what does", "explain",
            "samjhao", "samjhaiye", "samjha do", "samjhana", "define", "کیا ہوتا ہے", "کیا ہوتی ہے",
            "کیا ہوتے ہیں", "کیا ہوتا", "کیا ہوتی", "مطلب", "سمجھاؤ", "سمجھائیں", "سمجھا دو",
        ],
    ),
    (
        Cue::DefineWeak,
        &[
            "kya hai", "kya he", "kya hain", "kya hen", "what is", "کیا ہے", "کیا ہیں",
        ],
    ),
    (
        Cue::Capability,
        &[
            "kya kya dekh sakte", "kya kya dekh sakta", "kya kya pooch sakta", "kya pooch sakta",
            "kya pooch sakte", "what can i ask", "what can i see", "what can i do", "what can",
            "kya kya kar sakta", "kya kya kar sakte", "کیا پوچھ سکتا", "کیا پوچھ سکتے",
            "کیا کیا دیکھ سکتے", "کیا کیا دیکھ سکتا",
        ],
    ),
    (
        Cue::Advice,
        &[
            "barhayen", "barhana", "barhaon", "barhao", "barhane", "barhaye", "barhaen", "barhe",
            "barhen", "barhein", "barhta", "bherane", "bherana", "bheraon", "bherayen", "bhera",
            "badhayen", "badhana", "badhao", "badhane", "badhaon", "increase", "raise", "improve",
            "grow", "boost", "faster", "jaldi", "tez", "fast", "quickly", "behtar", "better",
            "reduce", "zyada kaise", "ziyada kaise", "ziada kaise", "kam kaise", "زیادہ کیسے", "بڑھا سکتے", "بڑھا سکتا", "barha sakte", "barha sakta", "badha sakte", "بڑھائیں", "بڑھانے", "بڑھاؤں", "بڑھاو", "بڑھاؤ",
            "بڑھے", "بہتر", "جلدی", "تیز",
        ],
    ),
    (
        Cue::AdviceStrong,
        &[
            "mashwara", "mashwaray", "mashwarah", "salah", "suggest", "suggestion", "suggestions",
            "recommend", "recommendation", "advice", "advise", "tips", "tip", "strategy",
            "plan banao", "مشورہ", "مشورے", "صلاح", "تجویز", "حکمت عملی",
        ],
    ),
    (
        Cue::Decision,
        &[
            "kis se pehle", "kisse pehle", "kisko pehle", "kis ko pehle", "kisko udhar",
            "kis ko udhar", "udhar na dun", "udhar na doon", "udhar na dein", "udhar na du",
            "udhar nahi dena", "udhar band", "kya karun", "kya karoon", "kya karna chahiye",
            "kya karen", "kya karein", "should i", "what should i", "who should", "kis par bharosa",
            "kis se wasooli", "kisko wasooli", "pehle kis", "کس سے پہلے", "کسے پہلے", "پہلے کس", "کس سے وصولی", "کس کو وصولی", "کس سے وصول", "کس کو پہلے",
            "کس کو ادھار", "ادھار نہ دوں", "کیا کروں", "کیا کرنا چاہیے",
        ],
    ),
    (
        Cue::Forecast,
        &[
            "is raftar", "is raftaar", "raftar se", "agle", "agla", "aane wale", "aane wala",
            "aanay wale", "future", "next", "target", "expected", "اس رفتار", "رفتار سے", "اگلے",
            "اگلا", "آنے والے", "آنے والا", "ہدف",
        ],
    ),
    (
        Cue::ForecastStrong,
        &[
            "andaza", "andaaza", "andazah", "forecast", "predict", "prediction", "estimate",
            "projection", "anticipate", "اندازہ", "اندازے",
        ],
    ),
    (
        Cue::HowMuch,
        &[
            "kitna", "kitni", "kitny", "kitne", "kitnay", "kitnaa", "kitnee", "kitnaah", "kitnon", "kitana",
            "kitani", "kitane", "kitnay", "kitno", "how much", "how many", "kitna hai", "کتنا",
            "کتنی", "کتنے", "کتنوں", "کتنیں",
        ],
    ),
    (
        Cue::Many,
        &[
            "kis ka", "kis ki", "kis ke", "kya kya", "kaun kaun", "kon kon", "kaun se", "kon se", "konse", "kaunse", "kaun si",
            "konsi", "kaunsi", "kis kis", "kisne kisne", "kaun kaun si", "kon kon si", "kaun kaun", "kon kon",
            "کون کون سی", "کون کون سے", "list", "lists", "fehrist", "fihrist",
            "ki list", "sab ki list", "all the", "every", "each", "har", "har ek", "saray",
            "saaray", "sare", "sab", "tamam", "تمام", "ساری", "سارے", "سب", "ہر", "ہر ایک",
            "کیا کیا", "کون کون", "کون سے", "کون سی", "کس کس", "فہرست", "لسٹ",
        ],
    ),
    (
        Cue::Most,
        &[
            "sab se zyada", "sab se ziada", "sab se ziyada", "sab say zyada", "sabse zyada",
            "sabse ziada", "zyada se zyada", "top", "highest", "biggest", "largest", "most",
            "maximum", "max", "best", "sabse purana", "sab se purana", "oldest", "sab se bara", "sab se barhi", "sab se bada", "number one",
            "sabse bara", "sabse bada", "sabse barhi", "sabse badi", "sabse bade", "sab se badi",
            "sab se bade", "sab se barhe", "sabse barhe",
            "سب سے زیادہ", "سب سے بڑا", "سب سے بڑی", "سب سے بڑے", "زیادہ سے زیادہ", "ٹاپ",
        ],
    ),
    (
        Cue::Least,
        &[
            "sab se kam", "sabse kam", "sab say kam", "kam se kam", "lowest", "smallest", "least",
            "minimum", "min", "worst", "sab se chota", "sab se choti", "sab se chote", "sabse chota",
            "sabse choti", "sabse chote", "سب سے کم", "کم سے کم", "سب سے چھوٹا", "سب سے چھوٹی",
            "سب سے چھوٹے",
        ],
    ),
    (
        Cue::And,
        &[
            "aur", "or", "and", "ya", "plus", "&", "aor", "او", "اور", "و", "یا",
        ],
    ),
    (
        Cue::Total,
        &[
            "kul", "total", "overall", "sum", "altogether", "poora", "pura", "puri", "poori", "dono", "donon",
            "teeno", "mila kar", "milakar", "mila ke", "kul mila kar", "kul milakar", "دونوں", "تینوں",
            "ملا کر", "کل ملا کر", "کل ملا کے", "کل میزان",
            "tamam", "ٹوٹل", "مجموعی", "میزان", "پورا", "پوری",
        ],
    ),
];

#[rustfmt::skip]
const ROLES: &[Entry] = &[
    (
        Cue::RoleBuyer,
        &[
            "buyer", "buyers", "gahak", "gaahak", "gahek", "gahk", "customer", "customers",
            "khareedar", "khariddar", "kharidar", "client", "گاہک", "خریدار", "کسٹمر",
        ],
    ),
    (
        Cue::RoleSeller,
        &[
            "seller", "sellers", "byopari", "byopary", "beopari", "bayopari", "vyapari", "vypari",
            "zamindar", "zameendar", "bezgar", "bazgar", "bizgar", "grower", "supplier", "farmer",
            "kisan", "بیوپاری", "زمیندار", "بزگر", "سپلائر", "کسان",
        ],
    ),
    (
        Cue::RolePartner,
        &[
            "partner", "partners", "shareek", "shareekdar", "shirakat dar", "shirkat dar", "shirakat daar",
            "shirkat daar", "shirakatdar", "shirkatdar", "sharik",
            "partnron", "partneron", "partnaron", "partnarz", "shirakat daron", "shirkat daron",
            "shirakatdaron", "shirkatdaron",
            "شراکت دار", "شراکت داروں", "شریک", "شریکوں", "پارٹنر", "پارٹنروں", "پارٹنرز",
        ],
    ),
    (
        Cue::RoleGarden,
        &[
            "garden", "gardens", "bag", "bagh", "baag", "baagh", "باغ", "باغات", "باغ کا",
        ],
    ),
    (
        Cue::RoleTrip,
        &[
            "trip", "trips", "safar", "safr", "safer", "market safar", "market safr", "market safer",
            "market trip", "market trips", "trips report", "trip report", "سفر", "مارکیٹ سفر",
        ],
    ),
    (
        Cue::RoleStaff,
        &[
            "staff", "munshi", "mulazim", "mulazam", "employee", "worker", "عملہ", "منشی", "ملازم",
        ],
    ),
    (
        Cue::RoleKhata,
        &[
            "alag khata", "alag khate", "separate khata", "الگ کھاتہ", "الگ کھاتے",
        ],
    ),
    (
        Cue::Journal,
        &[
            "journal", "jarnal", "jernal", "journal entry", "general entry", "جرنل", "جنرل اندراج",
            "جنرل انٹری",
        ],
    ),
    (
        Cue::HowManyMonths,
        &[
            "kitne mahine", "kitne mahinay", "kitne mahino", "kitne mahinon", "how many months",
            "کتنے مہینے", "کتنے مہینوں",
        ],
    ),
    (
        Cue::Quality,
        &[
            "awwal", "awal", "aval", "avval", "doyam", "doem", "doum", "doyum", "dom", "soyam",
            "soem", "soum", "soyum", "som", "first quality", "first grade", "second quality",
            "second grade", "third quality", "third grade", "a grade", "b grade", "c grade",
            "grade a", "grade b", "grade c", "a quality", "b quality", "c quality", "quality",
            "qualities", "grade", "grades", "darja", "darje", "darjay", "darjon", "darjey", "secnd",
            "sekond", "seccond", "frist", "fisrt", "thrd", "soym", "awl", "avl", "doyam", "dooyam", "dovam", "1st", "2nd", "3rd",
            "اول", "دوم", "سوم", "درجہ", "درجے", "درجات", "درجوں", "کوالٹی", "گریڈ",
        ],
    ),
    (
        Cue::Guests,
        &[
            "mehman", "mehmano", "mehmanon", "mehmaan", "guest", "guests", "مہمان", "مہمانوں",
        ],
    ),
    (
        Cue::RoleItem,
        &[
            "item", "qisam", "kisam", "qism", "maal ki qisam", "cheez", "cheezon", "chiz", "چیز",
            "چیزوں", "قسم", "قسم مال", "آئٹم",
        ],
    ),
    (
        Cue::ManyBuyers,
        &[
            "buyers", "gahakon", "gaahakon", "gahko", "gahkon", "customers", "khareedaron",
            "گاہکوں", "خریداروں", "کسٹمرز",
        ],
    ),
    (
        Cue::ManySellers,
        &[
            "sellers", "byopariyon", "byoparyon", "beopariyon", "zamindaron", "bezgaron",
            "بیوپاریوں", "زمینداروں", "بزگروں",
        ],
    ),
    (
        Cue::ManyItems,
        &[
            "items", "qismein", "qismain", "aqsam", "اقسام", "قسمیں", "آئٹمز",
        ],
    ),
    (
        Cue::Sold,
        &[
            "becha", "beche", "bechi", "bechna", "bechta", "bechte", "bechti", "bechay", "bheja", "bheje", "bheji", "bhejna", "maal bheja", "mal bheja",
            "supplied", "sent", "بھیجا", "بھیجے", "بھیجی", "مال بھیجا",
            "beecha", "beeche", "bika", "bike", "biki", "bikta", "bikti", "bikte", "biktay", "bikay", "bikega", "bikegi", "bikenge", "sold", "sell", "sells", "selling",
            "بیچا", "بیچے", "بیچی", "بیچنا", "بیچتا", "بیچتے", "بکا", "بکے", "بکی", "بکتا", "بکتی", "بکتے",
        ],
    ),
    (
        Cue::Bought,
        &[
            "khareeda", "khareede", "khareedi", "khareedna", "khareeda", "kharida", "kharide",
            "kharidi", "kharidna", "khareed", "kharid", "bought", "buy", "buys", "buying",
            "purchased", "purchase", "purchases", "maal liya", "mal liya", "maal lia", "mal lia",
            "maal le gaya", "maal le gaye", "le gaya", "le gaye", "kya liya", "kya lia", "kitna liya", "li", "lee", "khareed li", "kharid li",
            "kitne ka liya", "خریدا", "خریدے", "خریدی", "خریدنا",
            "خرید", "مال لیا", "مال لے گیا", "مال لے گئے",
        ],
    ),
];

#[rustfmt::skip]
const FIGURES: &[Entry] = &[
    (
        Cue::Balance,
        &[
            "bakaya", "baqaya", "bakaaya", "bakaayaa", "baqaya", "bakya", "bkaya", "baqiya",
            "baqia", "bakia", "balance", "balence", "balans", "outstanding", "owes", "owe", "owed",
            "lena hai", "lena he", "dena hai", "dena he", "kitna lena", "kitna dena", "kitna dega",
            "kitna dena hai", "kitna lena hai", "kitna deta", "kitna lete", "dega", "degi", "dene hain",
            "dene hai", "dene he", "dene hen", "lene hain", "lene hai", "lene he", "lene hen",
            "dena hain", "lena hain", "denge", "dengi", "lenge", "lengi", "دینے ہیں", "لینے ہیں",
            "دینا ہے", "لینا ہے",
            "bakaye", "بقایا", "بیلنس", "بکایا", "لینا", "دینا", "کتنا دے گا", "دے گا",
            "zimme", "zimmay", "zimay", "zimma", "ذمے", "ذمہ",
        ],
    ),
    (
        Cue::Pending,
        &[
            "baqi", "baki", "baqee", "bakee", "pending", "remaining", "left", "bacha", "bachi",
            "bache", "bachay", "باقی", "بچا", "بچی", "بچے", "پینڈنگ",
        ],
    ),
    (
        Cue::Sale,
        &[
            "bikri", "bikree", "bikari", "bikry", "bikre", "sale", "sales", "faroKht", "farokht",
            "khaam bikri", "kham bikri", "gross sale", "gross", "turnover", "بکری", "فروخت",
            "خام بکری", "خام",
        ],
    ),
    (
        Cue::SalePlural,
        &[
            "bikriyan", "bikriyaan", "bikriyon", "bikrian", "bikriyan", "بکریاں", "بکریوں",
            "how many sales", "number of sales", "count of sales", "sales count",
        ],
    ),
    (
        Cue::Commission,
        &[
            "commission", "commision", "comision", "commishan", "comm", "kamishan", "kamision",
            "kamishn", "kamisan", "comission", "comisson", "کمیشن", "کمشن", "کمیشں",
        ],
    ),
    (Cue::Laga, &["laga", "lagga", "لاگا"]),
    (
        Cue::Dues,
        &["dues", "receivable", "receivables", "ادھاری بقایا", "کل اگراہی"],
    ),
    (
        Cue::Collection,
        &[
            "ugrahi", "ugrai", "ugrahee", "ugrahy", "wasooli", "wasuli", "wasoli", "wasoolee",
            "wusooli", "collection", "اگراہی", "اگرائی", "وصولی",
        ],
    ),
    (
        Cue::Unit,
        &[
            "peti", "petiyan", "petiyon", "pati", "crate", "crates", "carton", "cartons", "box",
            "boxes", "bori", "boriyan", "boriyon", "nag", "dana", "kg", "kilo", "kilos", "ton",
            "پیٹی", "پیٹیاں", "پیٹیوں", "کریٹ", "کارٹن", "بوری", "بوریاں", "نگ", "دانہ", "کلو",
            "quantity", "qty", "quantities", "کوانٹیٹی",
        ],
    ),
    (
        Cue::Money,
        &[
            "paise", "paisa", "paisay", "pese", "raqam", "rakam", "rupay", "rupee", "rupees",
            "rupaye", "amount", "money", "cash amount", "پیسے", "پیسہ", "رقم", "روپے", "روپیہ",
        ],
    ),
    (
        Cue::Pukhta,
        &[
            "pukhta khata", "pakka khata", "pukhta", "pakka", "پختہ کھاتہ", "پکا کھاتہ", "پختہ",
        ],
    ),
    (
        Cue::Received,
        &[
            "mili", "mila", "mile", "aayi", "aai", "aya", "aaya", "received", "receive",
            "collected", "aaye", "aye", "آئے", "hui", "hue", "hua", "huee", "di", "dee", "diya", "diye", "dia", "ملی", "ملا",
            "آئی", "آیا", "وصول ہوئی", "وصول ہوا", "ہوئی", "ہوئے", "ہوا", "دی", "دیا", "دیے",
        ],
    ),
    (
        Cue::Payable,
        &[
            "adaigi", "adayegi", "adaiygi", "payable", "payables", "ادائیگی", "ادائیگیاں",
        ],
    ),
    (
        Cue::Advance,
        &[
            "peshgi", "peshgee", "pesgi", "peshgy", "advance", "advances", "پیشگی",
        ],
    ),
    (
        Cue::Stock,
        &[
            "stock", "stok", "baqi maal", "baki maal", "baqi mal", "baki mal", "maal bacha",
            "mal bacha", "bacha hua maal", "bacha maal", "unsold", "godown", "gudam", "باقی مال",
            "سٹاک", "بچا مال", "بچا ہوا مال", "maal para", "maal pada", "maal padi", "maal pare",
            "mal para", "mal pada", "mal padi", "مال پڑا", "مال پڑی", "مال پڑے",
        ],
    ),
    (
        Cue::Goods,
        &["maal", "mal", "goods", "saman", "saaman", "مال", "سامان"],
    ),
    (
        Cue::Cash,
        &[
            "cash", "cash book", "roqar book", "rokar book", "roqar bak", "rokar bak", "cash bak", "roqar", "rokar", "rokad", "roqad", "rokra", "rokkar", "gala", "galla", "gale",
            "cash in hand", "hand mein", "naqad", "naqd", "روکڑ", "گلہ", "گلے", "نقد",
        ],
    ),
    (
        Cue::Capital,
        &[
            "sarmaya", "sarmayah", "sarmaiya", "capital", "investment", "invest", "invested",
            "سرمایہ", "سرمائے", "سرمایہ کاری",
        ],
    ),
    (
        Cue::PartnerTotal,
        &[
            "kul raqam", "kul rakam", "total raqam", "partner total", "partner s total", "partners total", "khalis", "khalees",
            "khaalis", "khalis raqam", "net worth", "net", "کل رقم", "ٹوٹل رقم", "خالص",
        ],
    ),
    (
        Cue::PartnerShare,
        &[
            "hissa", "hissah", "hisa", "hissay", "hisse", "share", "shares", "profit share", "حصہ",
            "حصے", "شیئر",
        ],
    ),
    (
        Cue::Profit,
        &[
            "munafa", "munafah", "munafe", "nafa", "profit", "profits", "faida", "fayda", "loss",
            "nuqsan", "nuksan", "kamai", "kamayi", "earning", "earnings", "منافع", "فائدہ", "نقصان",
            "کمائی", "نفع",
        ],
    ),
    (
        Cue::Expense,
        &[
            "kharcha", "kharch", "kharche", "kharchay", "kharcho", "expense", "expenses",
            "spending", "خرچہ", "خرچ", "خرچے", "خرچوں", "خرچات", "اخراجات",
        ],
    ),
    (
        Cue::FruitTaken,
        &[
            "phal liya", "phal le gaye", "phal le gaya", "ghar ke liye phal", "fruit taken",
            "fruit liya", "phal register", "phal ka register", "phal ka print", "phal ka hisab", "phal ki list", "phal list",
            "fruit register", "پھل لیا", "پھل لے گئے",
            "گھر کے لیے پھل",
        ],
    ),
    (
        Cue::FruitWord,
        &[
            "phal", "phalon", "phalo", "phl", "phalan", "fruit", "fruits", "frut", "fruts", "fruiit",
            "پھل", "پھلوں",
        ],
    ),
    (
        Cue::VegetableWord,
        &[
            "sabzi", "sabzian", "sabziyan", "sabziyon", "sabzion", "sabzee", "sabji", "sabjiyan",
            "sabjiyon", "sabzi", "subzi", "subzian", "subziyan", "subziyon", "sabziya", "vegetable",
            "vegetables", "vegitable", "vegitables", "vegetabls", "vegtable", "vegtables", "veggies",
            "veggie", "vegies", "سبزی", "سبجی", "سبجیاں", "سبجیوں", "سبزیاں",
            "سبزیوں", "سبزیات",
        ],
    ),
    (
        Cue::MoneyLent,
        &[
            "udhar raqam", "udhar rakam", "udhaar raqam", "udhar", "udhaar", "money lent", "loan", "loans", "qarz",
            "qarza", "karz", "ادھار رقم", "قرض", "قرضہ", "ادھار",
        ],
    ),
    (
        Cue::DeadAmount,
        &[
            "dubi", "doobi", "dobi", "dubi hui", "dubi raqam", "dobi raqam", "dead amount",
            "bad debt", "ڈوبی", "ڈوبی رقم",
        ],
    ),
    (
        Cue::Rate,
        &[
            "rate", "rates", "reet", "ret", "bhao", "bhaw", "bhaao", "bhav", "daam", "dam",
            "qeemat", "keemat", "price", "prices", "ریٹ", "بھاؤ", "بھاو", "دام", "قیمت",
        ],
    ),
    (
        Cue::DaySummary,
        &[
            "khulasa", "khulasah", "summary", "din ka hisab", "roz ka hisab", "aaj ka hisab",
            "daily summary", "day summary", "خلاصہ", "دن کا حساب",
        ],
    ),
    (
        Cue::DayBook,
        &["roznamcha", "roznamchah", "roznamchay", "roznamche", "roznama", "day book", "daybook", "روزنامچہ", "روزنامچے", "روزنامہ"],
    ),
    (
        Cue::TrialBalance,
        &["trial balance", "trial", "ٹرائل بیلنس", "ٹرائل"],
    ),
    (
        Cue::ByCity,
        &[
            "city wise", "shehar wise", "shahar wise", "sheher wise", "bahisab shehar",
            "shehar ke hisab se", "city ke hisab se", "by city", "per city", "city by city",
            "by shehar", "بحساب شہر", "شہر وار", "شہر کے حساب سے", "شہر کے لحاظ سے",
        ],
    ),
    (
        Cue::Ledger,
        &[
            "khata", "khaata", "khate", "khatay", "ledger", "hisab", "hisaab", "hisabat", "account",
            "کھاتہ", "کھاتے", "کھاتا", "حساب", "لیجر",
        ],
    ),
    (
        Cue::Statement,
        &[
            "statement", "fard", "fard hisab", "fard e hisab", "فرد", "فرد حساب", "اسٹیٹمنٹ",
        ],
    ),
    (
        Cue::ResultWord,
        &["result", "natija", "nateeja", "nataij", "نتیجہ", "نتیجے", "نتائج"],
    ),
    (Cue::NumberWord, &["number", "numbar", "nmbr", "num", "نمبر"]),
    (
        Cue::PartnerTaken,
        &[
            "li hui raqam", "li hui rakam", "lee hui raqam", "liye hue paise", "partner drawings",
            "partners drawings", "drawings", "partners took", "partner took", "money taken by partners",
            "money taken by the partners", "taken by the partners", "partner has taken", "zaati kharcha", "zaati kharch", "zati kharcha",
            "zaati kharche", "ذاتی خرچہ", "ذاتی خرچ", "لی ہوئی رقم", "لیے ہوئے پیسے",
        ],
    ),
];

#[rustfmt::skip]
const OUTSIDE: &[Entry] = &[
    (
        Cue::ExternalWeather,
        &[
            "weather", "mausam", "mosam", "موسم", "barish", "baarish", "rain", "temperature",
            "garmi", "sardi", "weather forecast",
        ],
    ),
    (
        Cue::ExternalTax,
        &[
            "gst", "g s t", "sales tax", "income tax", "tax return", "fbr", "ntn", "withholding",
            "withholding tax", "ٹیکس ریٹرن", "جی ایس ٹی",
        ],
    ),
    (
        Cue::ExternalBank,
        &[
            "bank balance", "bank mein", "bank me", "bank account", "bank ka balance", "bank se",
            "banking", "بینک بیلنس", "بینک میں", "بینک اکاؤنٹ",
        ],
    ),
    (
        Cue::ExternalPrices,
        &[
            "dollar", "dollars", "usd", "euro", "ڈالر", "gold", "gold rate", "silver", "petrol",
            "diesel", "psx", "stock market", "share market", "bitcoin", "crypto", "پیٹرول", "ڈیزل",
        ],
    ),
    (
        Cue::External,
        &[
            "cricket", "match", "news", "khabar", "khabrein", "election", "politics", "joke",
            "latifa", "recipe", "film", "movie", "song", "gana", "namaz ka time", "prayer time",
            "azan", "مشہور", "خبریں", "خبر", "کرکٹ", "الیکشن",
        ],
    ),
    (
        Cue::ExternalPlace,
        &[
            "lahore", "karachi", "islamabad", "multan", "faisalabad", "peshawar", "quetta",
            "rawalpindi", "sargodha", "gujranwala", "sialkot", "hyderabad", "bahawalpur", "okara",
            "pakistan", "india", "dubai", "china", "لاہور", "کراچی", "اسلام آباد", "ملتان",
            "فیصل آباد", "پشاور", "کوئٹہ", "راولپنڈی", "گوجرانوالہ", "سیالکوٹ", "حیدرآباد",
            "پاکستان", "بھارت", "دبئی",
        ],
    ),
    (
        Cue::MarketRate,
        &[
            "mandi ka rate", "mandi ka bhao", "mandi rate", "market rate", "market ka rate",
            "market price", "open market", "mandi bhao", "mandi ka bhaw", "منڈی کا ریٹ",
            "منڈی کا بھاؤ", "مارکیٹ ریٹ",
        ],
    ),
];

/// Function words that start no name: particles, pronouns, question words and small verbs.
#[rustfmt::skip]
pub(super) const STOP: &[&str] = &[
    "sahab", "sahib", "saheb", "bhai", "bhaiya", "bhaijaan", "ji", "miyan", "mr", "mrs", "ms", "sir",
    "madam", "uncle", "chacha", "shop", "dukan", "dukaan", "business", "karobar", "firm", "صاحب",
    "بھائی", "بھیا", "جی", "میاں", "چچا", "دکان", "کاروبار",
    "bana", "bani", "bane", "banay", "بنا", "بنی", "بنے",
    "purana", "purani", "purane", "puranay", "naya", "nayi", "naye", "aakhri", "akhri", "pehla", "pehli",
    "oldest", "newest", "latest", "first", "sabse", "report", "reports", "رپورٹ", "saari", "saara", "sara", "saray", "سارا", "whole",
    "entire", "karni", "karne", "kijiye", "کرنی", "کرنے", "hazar", "hazaar", "lakh", "lac", "crore", "karor",
    "sau", "million", "thousand", "hundred", "billion", "book", "bak", "buk", "بک", "ہزار", "لاکھ",
    "کروڑ", "سو", "پرانا", "پرانی", "نیا", "نئی", "آخری", "پہلا", "پہلی",
    "ka", "ki", "ke", "kay", "ko", "se", "say", "ne", "nay", "mein", "me", "main", "men", "par",
    "pe", "per", "tak", "tk", "ho", "hai", "he", "hain", "hen", "hy", "hoon", "hun", "tha", "thi",
    "the", "thay", "hoga", "hogi", "hoge", "hote", "hota", "hoti", "hua", "hui", "hue", "huwa",
    "huay", "gaya", "gayi", "gaye", "gai", "kiya", "kia", "kiye", "kiyaa", "ki", "diya", "dia",
    "diye", "liya", "lia", "liye", "kar", "karo", "karen", "karein", "karun", "karoon", "karna",
    "kare", "karte", "karta", "karti", "kro", "do", "de", "den", "dein", "dena", "batao", "bata",
    "bataye", "bataiye", "btao", "bta", "batana", "kya", "kyun", "kyon", "kab", "kaun", "kon",
    "kisne", "kisi", "kis", "sa", "si", "kaunsa", "konsa", "koi", "kuch", "kuchh", "ye", "yeh", "wo", "woh", "us", "uska", "uski",
    "uske", "un", "unka", "unki", "unke", "is", "iska", "iski", "iske", "in", "inka", "inki",
    "inke", "mera", "meri", "mere", "mujhe", "mujhko", "mjhe", "humein", "hamein", "hamara",
    "hamari", "hamare", "hum", "ham", "main", "apna", "apni", "apne", "aap", "aapka", "aapki",
    "aapke", "please", "plz", "pls", "zara", "bhi", "hi", "to", "toh", "sirf", "phir", "abhi", "ab",
    "nahi", "nahin", "na", "agar", "magar", "lekin", "ke", "ya", "ka", "ek", "sath", "saath",
    "liye", "baad", "pehle", "pahle", "wala", "wali", "wale", "walay", "waly", "ka", "a", "an",
    "the", "of", "in", "on", "at", "by", "with", "from", "for", "to", "is", "are", "was", "were",
    "be", "been", "do", "does", "did", "i", "me", "my", "we", "our", "us", "you", "your", "he",
    "his", "she", "her", "they", "their", "it", "its", "this", "that", "these", "those", "have",
    "has", "had", "can", "could", "will", "would", "shall", "should", "may", "might", "tell",
    "give", "need", "want", "please", "s", "take", "than", "then", "what", "which", "who", "whom", "when",
    "why", "also", "just", "so", "if", "about", "all", "any", "some", "ka", "kitna", "jo", "jis",
    "jin", "ke", "kaha", "kehna", "bolo", "boliye", "suna", "suniye", "dekho", "dekhna", "dekh",
    "dekhen", "dekhein", "sakta", "sakte", "sakti", "chahta", "chahte", "chahti", "chahiye", "raha",
    "rahi", "rahe", "rha", "rhi", "rhe", "gaya", "bohat", "bahut", "bahot", "zyada", "ziada",
    "ziyada", "kam", "thora", "thoda", "kitni", "kitne", "ka", "ke", "ko", "wapsi", "wapis", "now", "far", "till", "upto", "currently", "current", "muqable", "muqabla",
    "muqabale", "mawazna", "compare", "compared", "comparison", "versus", "vs", "with", "مقابلے",
    "مقابلہ", "موازنہ", "زیادہ", "کم", "کتنا", "up", "نام", "کام", "عملے", "عملہ", "jama", "jamaa", "screen", "page", "safha", "صفحہ", "fisad", "feesad", "percent", "percentage", "فیصد", "ga", "gi", "ge", "گا", "گی", "گے", "آخر", "اختتام", "aakhir", "akhir", "end", "khatam", "ختم",
    "did", "go", "went", "going", "doing", "been", "padi", "pada", "padhi", "pade", "پڑی",
    "پڑا", "ausat", "average", "avg", "اوسط", "time", "bar", "baar", "dafa", "dafah", "martaba", "بار", "دفعہ", "اور", "کس", "کسے", "کسی", "پہلے", "پہلا", "واپسی", "واپس", "کا", "کی", "کے",
    "کو", "سے", "نے", "میں", "پر", "تک", "ہے", "ہیں", "تھا", "تھی", "تھے", "ہو", "ہوا", "ہوئی",
    "ہوئے", "ہوگا", "ہوگی", "گیا", "گئی", "گئے", "کیا", "کیے", "دیا", "دی", "دیے", "لیا", "لی",
    "لیے", "کر", "کرو", "کریں", "کروں", "کرنا", "کرے", "بتاؤ", "بتائیں", "بتا", "کیوں", "کب", "کون",
    "کوئی", "کچھ", "یہ", "وہ", "اس", "اسکا", "اسکی", "ان", "ان کا", "میرا", "میری", "مجھے", "ہمیں",
    "ہمارا", "ہماری", "ہم", "اپنا", "اپنی", "آپ", "براہ", "مہربانی", "بھی", "ہی", "تو", "صرف",
    "پھر", "ابھی", "اب", "نہیں", "نہ", "اگر", "مگر", "لیکن", "ایک", "ساتھ", "والا", "والی", "والے",
    "رہا", "رہی", "رہے", "سکتا", "سکتے", "سکتی", "چاہیے", "بہت", "تھوڑا",
];

/// Words that mean something else in a question but are also the usual parts of a firm's name
/// ("Imran Fruit", "Shahpur Garden", "Akram Toll Tax", "Nadeem Traders"). They stay in a name run;
/// if the name is found, a cue inside it is dropped.
#[rustfmt::skip]
pub(super) const NAMEISH: &[&str] = &[
    "fruit", "fruits", "garden", "bagh", "bag", "traders", "trader", "trading", "brothers", "bros",
    "tax", "toll", "store", "stores", "general", "cash", "mandi", "khata", "company", "co",
    "enterprises", "enterprise", "sons", "son", "ltd", "mart", "centre", "center", "depot",
    "agency", "supplier", "suppliers", "farm", "farms", "adda", "market", "haji", "malik", "sheikh",
    "پھل", "باغ", "ٹریڈرز", "برادرز", "ٹیکس", "ٹول", "سٹور", "جنرل", "کیش", "منڈی", "کمپنی", "اینڈ",
    "سنز", "فارم", "فارمز", "اڈا", "مارکیٹ", "حاجی", "ملک",
];

/// The titles of the reports that no figure names, by report: the words a print or an open request
/// uses for it (PRD 19.2). The words are in the table as [`Cue::ReportName`]; [`report_named`] gives
/// the report of the words of one hit.
///
/// Example: "udhari bikri ka print" is R4; "adda report" is R8.
#[rustfmt::skip]
pub(super) const REPORT_NAMES: &[(&str, &[&str])] = &[
    ("R3", &["naqdi bikri", "naqdi sales", "naqdi sale", "cash sales", "cash sale", "نقدی بکری"]),
    ("R4", &["udhari bikri", "udhari sales", "udhari sale", "udhaari bikri", "credit sales", "credit sale", "ادھاری بکری"]),
    ("R7", &["item wise bikri", "item wise sale", "item wise sales", "itemwise bikri", "qisam war bikri", "qisam wise bikri", "item wise", "قسم وار بکری", "قسم وار"]),
    ("R8", &["adda report", "adda rapat", "adda wise", "truck stand report", "truck adda report", "اڈا رپورٹ", "اڈا وار"]),
    ("R9", &["pukhta khata register", "pukhta khata report", "pukhta register", "pakka khata register", "پختہ کھاتہ رجسٹر", "پختہ کھاتہ رپورٹ"]),
    ("R15", &["voucher list", "vouchers list", "voucher register", "vouchers register", "ووچر لسٹ", "واؤچر لسٹ", "واؤچر رجسٹر"]),
    ("R16", &["account index", "accounts index", "index report", "index print", "accounts list", "list of accounts", "khaton ki list", "کھاتوں کی فہرست", "انڈکس رپورٹ"]),
    ("R17", &["head balances", "head balance", "heads balance", "madon ka balance", "mad wise balance", "مدوں بیلنس", "مدوں کا بیلنس"]),
    ("R19", &["opening trial", "open trial", "opening trial balance", "اوپن ٹرائل", "ابتدائی ٹرائل"]),
    ("R22", &["collector wise", "collector report", "ugrahi wala wise", "agrahi wala wise", "ugrahi wale wise", "اگراہی والا وار", "اگراہی والے وار", "اگراہی والا", "اگراہی والے"]),
    ("R23", &["wasooli register", "wasuli register", "receipts register", "receipt register", "receipts list", "وصولی رجسٹر"]),
    ("R25", &["adaigi register", "payments register", "payment register", "payments list", "ادائیگی رجسٹر"]),
    ("R27", &["bijak series", "bijag series", "bejak series", "baijak series", "bill series", "بیجک سیریز", "بجک سیریز"]),
    ("R28", &["amdan report", "aamdan report", "amdani report", "income report", "income ka report", "آمدن رپورٹ", "آمدنی رپورٹ"]),
    ("R30", &["munafa taqseem", "munafa o taqseem", "profit and partner split", "profit split", "partner profit split", "منافع و تقسیم", "منافع تقسیم"]),
    ("R32", &["udhar raqam report", "udhar raqam ka report", "money lent report", "loans report", "ادھار رقم رپورٹ"]),
    ("A", &["item wise tajziya", "item wise tajziyat", "item wise analytics", "qisam war tajziya", "analytics by item", "commission by item", "قسم وار تجزیہ"]),
    ("B", &["seller wise tajziya", "byopari wise tajziya", "analytics by seller", "commission by seller", "بیوپاری وار تجزیہ"]),
    ("C", &["buyer wise tajziya", "gahak wise tajziya", "analytics by buyer", "commission by buyer", "گاہک وار تجزیہ"]),
    ("D", &["item month wise", "month wise commission", "mahine wise commission", "item aur mahina wise", "ماہانہ کمیشن", "قسم مال مہینہ وار"]),
    ("E", &["seller item wise", "byopari item wise", "seller by item", "بیوپاری قسم مال وار"]),
    ("F", &["partner wise tajziya", "shirakat daar wise tajziya", "partner analytics", "شراکت دار وار تجزیہ"]),
    ("G", &["commission trend", "bikri ka rujhan", "bikri trend", "sales trend", "rujhan", "رجحان"]),
    ("H", &["top 10 tajziya", "top 10 analytics", "top 10 view", "top 10 report", "top ten report", "top ten view", "sarfehrist", "sarfarist", "sar farist", "سرفہرست"]),
    ("I", &["gardens analytics", "garden analytics", "bagh tajziya", "باغات تجزیہ", "باغ تجزیہ"]),
    ("J", &["trips analytics", "trip analytics", "safar tajziya", "سفر تجزیہ", "مارکیٹ سفر تجزیہ"]),
    ("K", &["fruit analytics", "phal tajziya", "fruit taken analytics", "پھل تجزیہ"]),
    ("R38", &["doobi raqam report", "dubi raqam report", "dead amount report", "bad debt report", "ڈوبی رقم رپورٹ"]),
];

/// The names of the screens, by the topic of the guide that describes each (PRD 5, `prompt/guide.json`):
/// what the clerk says to open one. With an opening word ("kholo", "take me to", "dikhao") the topic and
/// the screen follow from the name (live run 2, G1). A one-word name opens a screen only with a plain
/// opening word; "show me" needs a name of two words or the word "screen" or "page".
///
/// Example: "take me to day close" is `day_close`; "روکڑ بک کھولیں" is `cash_book`.
#[rustfmt::skip]
pub(super) const SCREEN_NAMES: &[(&str, &[&str])] = &[
    ("bikri_book", &["sale book", "sales book", "bikri book", "bikri bak", "bikri screen", "sale screen", "بکری بک"]),
    ("receipt", &["wasooli", "wasuli", "wasooli screen", "receipt screen", "receipts screen", "وصولی"]),
    ("payment", &["adaigi", "adaigi screen", "payment screen", "ادائیگی"]),
    ("advance", &["peshgi", "peshgi screen", "advance screen", "پیشگی"]),
    ("shop_expense", &["kharcha", "kharcha screen", "expense screen", "expenses screen", "خرچہ"]),
    ("money_lent", &["udhar raqam", "udhaar raqam", "money lent", "money lent screen", "ادھار رقم"]),
    ("partner_capital", &["partner capital", "partners capital", "sarmaya", "sarmaya screen", "capital screen", "سرمایہ"]),
    ("fruit_taken", &["fruit taken", "phal", "phal screen", "fruit screen", "پھل"]),
    ("general_journal", &["general", "general screen", "journal", "journal screen", "jarnal", "جنرل", "جرنل"]),
    ("masters", &["index", "index screen", "masters", "masters screen", "انڈکس"]),
    ("accounts", &["accounts list", "account list", "accounts screen", "اکاؤنٹس لسٹ", "کھاتوں کی فہرست"]),
    ("commission_rules", &["commission rules", "commission rule", "rules screen", "commission usool", "کمیشن اصول"]),
    ("today_sales", &["todays sales list", "today s sales list", "today s sales", "today sales list", "todays sales", "today sales", "aaj ki bikri list", "آج کی بکری فہرست"]),
    ("unsold_stock", &["unsold stock", "unsold goods", "baqi maal", "baqi maal screen", "باقی مال"]),
    ("pukhta_khata", &["pukhta khata", "pakka khata", "pukhta khata screen", "پختہ کھاتہ"]),
    ("collection_list", &["collection list", "agrahi list", "agraahi list", "ugrahi list", "اگراہی فہرست", "اگراہی لسٹ"]),
    ("cash_book", &["cash book", "cashbook", "roknad book", "rokar book", "روکڑ بک", "کیش بک"]),
    ("gardens", &["gardens", "baghat", "bagh", "باغات", "باغ"]),
    ("market_trips", &["market trips", "market trip", "market safar", "مارکیٹ سفر"]),
    ("day_close", &["day close", "dayclose", "din band", "دن بند"]),
    ("reports", &["reports", "report screen", "reports screen", "رپورٹ", "رپورٹس"]),
    ("dead_amount", &["dead amount", "doobi raqam", "dubi raqam", "ڈوبی رقم"]),
    ("backup", &["backup", "back up", "بیک اپ"]),
];

/// Other spellings and the oblique forms of the names above ("kharche wali screen"). They find the
/// screen by its words only: they are not added to the known words, because a word that is only near a
/// figure word ("kharchon", the expenses) keeps its figure.
#[rustfmt::skip]
const SCREEN_FORMS: &[(&str, &[&str])] = &[
    ("shop_expense", &["kharche", "kharchay", "kharchey", "kharchon", "خرچے"]),
    ("receipt", &["vasooli", "vasuli", "وصولیاں"]),
    ("payment", &["adaigy", "adayegi", "ادائیگیاں"]),
    ("advance", &["peshgee", "pesgi"]),
    ("partner_capital", &["sarmaye", "سرمائے"]),
];

/// Where the names of the screens stand in the words of a question: (first word, after the last word,
/// topic). Found by the words, so a name that another table also holds ("doobi raqam", "cash book") is
/// found too.
pub(super) fn screen_windows(words: &[String]) -> Vec<(usize, usize, &'static str)> {
    let mut out = Vec::new();
    for (topic, names) in SCREEN_NAMES.iter().chain(SCREEN_FORMS) {
        for name in *names {
            let phrase = words_of(name);
            let size = phrase.len();
            if size == 0 || size > words.len() {
                continue;
            }
            for start in 0..=words.len().saturating_sub(size) {
                let end = start.saturating_add(size);
                if words.get(start..end) == Some(phrase.as_slice()) {
                    out.push((start, end, *topic));
                }
            }
        }
    }
    out
}

/// The report a hit of [`Cue::ReportName`] names, from the normalised words it covers.
pub(super) fn report_named(words: &[String]) -> Option<&'static str> {
    REPORT_NAMES.iter().find_map(|(id, names)| {
        names
            .iter()
            .any(|name| words_of(name) == words)
            .then_some(*id)
    })
}

/// Questions about the app's own state, by the read they need (plan 55 §11.1, `app_state` and the
/// staff and expense-head lists of `find`): `rules` (which commission or laga rule applies),
/// `attention` (the list "آج کے کام"), `days` (the days closed), `staff_munshi`, `staff_collector`,
/// `staff_labour`, `staff_all` (how many of the staff), `heads` (how many expense heads),
/// `vouchers_receipt`, `vouchers_payment`, `vouchers_advance` (the vouchers of one type only: R15),
/// `audit` (who changed what: the newest rows of the audit log, or those of one party or one بکری).
///
/// Example: "rahim par commission kitna lagta hai" is `rules`; "kitne munshi hain" is `staff_munshi`.
#[rustfmt::skip]
pub(super) const ASK_NAMES: &[(&str, &[&str])] = &[
    ("rules", &[
        "commission kitna lagta hai", "laga kitna lagta hai", "kitna commission lagta hai", "kitna laga lagta hai",
        "commission kitna lagta", "laga kitna lagta", "commission lagta hai", "commission rate", "commission rates",
        "commission ka rate", "laga rate", "laga ka rate", "commission rule", "commission rules", "laga rule",
        "charge rule", "commission kitne percent lagta hai", "کمیشن کتنا لگتا ہے", "لاگا کتنا لگتا ہے",
        "کمیشن ریٹ", "لاگا ریٹ", "کمیشن کا ریٹ", "لاگا کا ریٹ",
    ]),
    ("attention", &[
        "aaj ke kaam", "aaj ka kaam", "aaj kya kaam hai", "kya baqi hai", "kya kya baqi hai", "kya pending hai",
        "kya zaroori hai", "attention list", "needs attention", "what needs attention", "what is pending",
        "توجہ طلب", "آج کے کام", "آج کا کام", "کیا باقی ہے",
    ]),
    ("days", &[
        "kaunse din band hain", "kaun se din band hain", "kon se din band hain", "kaunse din band",
        "din band hain", "din band hue", "mahina band hua", "mahina band ho gaya", "month closed",
        "closed days", "days closed", "which days are closed", "کون سے دن بند ہیں", "کونسے دن بند ہیں",
        "دن بند ہیں", "مہینہ بند ہوا", "مہینہ بند ہو گیا",
    ]),
    ("audit", &[
        "kisne kya badla", "kisne kya badal", "kis ne kya badla", "kisne badla", "kis ne badla",
        "aakhri changes", "aakhri tabdeeliyan", "recent changes", "latest changes", "kya changes hue",
        "who changed what", "who changed", "کس نے کیا بدلا", "آخری تبدیلیاں",
    ]),
    ("staff_munshi", &["kitne munshi", "munshi kitne", "how many munshi", "how many munshis", "کتنے منشی", "منشی کتنے"]),
    ("staff_collector", &["kitne collector", "collector kitne", "how many collectors", "کتنے اگراہی والے"]),
    ("staff_labour", &["kitne mazdoor", "mazdoor kitne", "kitne labour", "how many labourers", "کتنے مزدور"]),
    ("staff_all", &["kitne staff", "staff kitne", "kitne log kaam", "how many staff", "کتنا عملہ", "کتنے ملازم", "عملے کے نام", "سارے عملے کے نام", "عملے کی فہرست", "تمام عملہ"]),
    ("vouchers_receipt", &["sirf wasooli ki list", "sirf wasooli", "sirf wasooli ke voucher", "wasooli ke voucher", "receipt vouchers", "only receipts", "only receipt vouchers", "صرف وصولی"]),
    ("vouchers_payment", &["sirf adaigi ki list", "sirf adaigi", "sirf adaigi ke voucher", "adaigi ke voucher", "payment vouchers", "only payments", "صرف ادائیگی"]),
    ("vouchers_advance", &["sirf peshgi ki list", "sirf peshgi", "sirf peshgi ke voucher", "peshgi ke voucher", "advance vouchers", "only advances", "صرف پیشگی"]),
    ("packings", &["kitni packing", "packing kitni", "how many packings", "how many packing", "کتنی پیکنگ"]),
    ("markas", &["kitne marka", "marka kitne", "how many markas", "how many marka", "کتنے مارکہ"]),
    ("stores", &["kitne store", "store kitne", "kitne godam", "how many stores", "کتنے سٹور"]),
    ("narrations", &["kitni narration", "narration kitni", "how many narrations", "how many narration"]),
    ("heads", &["kitne expense head", "kitne expense heads", "expense heads kitne", "how many expense heads", "kharche ke kitne head", "کتنی خرچہ مد", "خرچہ مد کتنی"]),
];

/// The ask a hit of [`Cue::AskName`] names, from the normalised words it covers.
pub(super) fn ask_named(words: &[String]) -> Option<&'static str> {
    ASK_NAMES.iter().find_map(|(id, names)| {
        names
            .iter()
            .any(|name| words_of(name) == words)
            .then_some(*id)
    })
}

/// The compiled table.
pub(super) struct Table {
    singles: HashMap<String, Vec<Cue>>,
    phrases: Vec<(Vec<String>, Cue)>,
    /// Long Latin words of the table, for the soft match.
    pool: Vec<(String, Vec<Cue>)>,
    stop: HashSet<String>,
    nameish: HashSet<String>,
}

/// The key of a word or phrase: lower case, one Urdu letter for each look-alike.
pub(super) fn key(word: &str) -> String {
    text::search_ur(&text::search_en(word))
}

fn words_of(phrase: &str) -> Vec<String> {
    key(phrase)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

impl Table {
    fn build() -> Self {
        let mut singles: HashMap<String, Vec<Cue>> = HashMap::new();
        let mut phrases: Vec<(Vec<String>, Cue)> = Vec::new();
        let leak = |tables: &[(&str, &[&'static str])]| -> &'static [&'static str] {
            Box::leak(
                tables
                    .iter()
                    .flat_map(|(_, names)| names.iter().copied())
                    .collect::<Vec<&'static str>>()
                    .into_boxed_slice(),
            )
        };
        let report_entries: Vec<Entry> = vec![
            (Cue::ReportName, leak(REPORT_NAMES)),
            (Cue::AskName, leak(ASK_NAMES)),
        ];
        for table in [INTENT, ROLES, FIGURES, OUTSIDE, report_entries.as_slice()] {
            for (cue, entries) in table {
                for entry in *entries {
                    let words = words_of(entry);
                    match words.as_slice() {
                        [] => {}
                        [one] => {
                            let cues = singles.entry(one.clone()).or_default();
                            if !cues.contains(cue) {
                                cues.push(*cue);
                            }
                        }
                        _ => phrases.push((words, *cue)),
                    }
                }
            }
        }
        // The names of the screens add their words to the known words and take nothing away: a phrase
        // here would hide the cue of a word it holds ("doobi raqam", "unsold stock"). The screen is
        // found by `screen_windows`, from the words.
        for (_, names) in SCREEN_NAMES {
            for entry in *names {
                for word in words_of(entry) {
                    let cues = singles.entry(word).or_default();
                    if !cues.contains(&Cue::ScreenName) {
                        cues.push(Cue::ScreenName);
                    }
                }
            }
        }
        // Longest phrases first, so a longer phrase is looked for before a shorter one.
        phrases.sort_by_key(|(words, _)| std::cmp::Reverse(words.len()));
        let pool = singles
            .iter()
            .filter(|(word, _)| {
                word.chars().count() >= 6 && word.chars().all(|c| c.is_ascii_lowercase())
            })
            .map(|(word, cues)| (word.clone(), cues.clone()))
            .collect();
        Self {
            singles,
            phrases,
            pool,
            stop: STOP.iter().map(|w| key(w)).collect(),
            nameish: NAMEISH.iter().map(|w| key(w)).collect(),
        }
    }

    /// The cues of one word.
    pub(super) fn cues_of(&self, word: &str) -> &[Cue] {
        self.singles.get(word).map_or(&[], Vec::as_slice)
    }

    /// True for a function word.
    pub(super) fn is_stop(&self, word: &str) -> bool {
        self.stop.contains(word)
    }

    /// True for a word that may be a part of a name though it has a cue.
    pub(super) fn is_nameish(&self, word: &str) -> bool {
        // The name of a place (Lahore) can be the name of a city or an اڈا of the books.
        self.nameish.contains(word) || self.cues_of(word).contains(&Cue::ExternalPlace)
    }

    /// True when the word is in the table or a function word: it starts no name.
    pub(super) fn is_known(&self, word: &str) -> bool {
        self.singles.contains_key(word) || self.stop.contains(word)
    }

    /// True when the word is known, or one letter off a word of the table or a function word (two
    /// letters from nine): a common word spelt wrong ("kitny", "gaahkon"). Such a word is never a name
    /// that is missing from the books (money review of run 1: "aj ki bikri kitny ki hui" is not a
    /// question about a party called kitny).
    pub(super) fn is_near_known(&self, word: &str) -> bool {
        if self.is_known(word) {
            return true;
        }
        let length = word.chars().count();
        if length < 4 || !word.chars().all(|c| c.is_ascii_lowercase()) {
            return false;
        }
        let allowed = if length >= 9 { 2 } else { 1 };
        let near = |candidate: &str| {
            candidate.chars().count().abs_diff(length) <= allowed
                && super::scan::edit_distance(word, candidate, allowed) <= allowed
        };
        self.pool.iter().any(|(candidate, _)| near(candidate))
            || self.stop.iter().any(|candidate| near(candidate))
    }

    /// All the hits of a list of normalised words: phrases first, then single words, then soft hits
    /// for long words that are one or two letters from a word of the table. A hit that lies inside a
    /// longer hit is dropped, so "baqi maal" is stock and not a balance.
    pub(super) fn hits(&self, words: &[String]) -> Vec<Hit> {
        let mut hits: Vec<Hit> = Vec::new();
        for (at, _) in words.iter().enumerate() {
            for (phrase, cue) in &self.phrases {
                let end = at.saturating_add(phrase.len());
                let Some(window) = words.get(at..end) else {
                    continue;
                };
                if window == phrase.as_slice() {
                    hits.push(Hit {
                        cue: *cue,
                        start: at,
                        end,
                        fuzzy: false,
                    });
                }
            }
        }
        for (at, word) in words.iter().enumerate() {
            let end = at.saturating_add(1);
            let exact = self.cues_of(word);
            if !exact.is_empty() {
                hits.extend(exact.iter().map(|cue| Hit {
                    cue: *cue,
                    start: at,
                    end,
                    fuzzy: false,
                }));
            } else if let Some(cues) = self.nearest(word) {
                hits.extend(cues.iter().map(|cue| Hit {
                    cue: *cue,
                    start: at,
                    end,
                    fuzzy: true,
                }));
            }
        }
        let spans: Vec<(usize, usize)> = hits.iter().map(|h| (h.start, h.end)).collect();
        hits.retain(|h| {
            // "kitna" inside "kitna dena hai" still says that a figure is asked; "print" inside "phal ka
            // print" still says that a printout is asked.
            matches!(h.cue, Cue::HowMuch | Cue::Print | Cue::Show | Cue::Open)
                || !spans.iter().any(|(s, e)| {
                    (*s <= h.start && h.end <= *e)
                        && (e.saturating_sub(*s) > h.end.saturating_sub(h.start))
                })
        });
        hits.sort_by_key(|h| (h.start, h.end));
        hits.dedup();
        hits
    }

    /// The cues of the one word of the table that a long Latin word is one or two letters from.
    fn nearest(&self, word: &str) -> Option<&[Cue]> {
        let length = word.chars().count();
        if length < 6 || self.stop.contains(word) || !word.chars().all(|c| c.is_ascii_lowercase()) {
            return None;
        }
        let allowed = if length >= 9 { 2 } else { 1 };
        let mut best: Option<(&str, usize)> = None;
        for (candidate, _) in &self.pool {
            let gap = candidate.chars().count().abs_diff(length);
            if gap > allowed {
                continue;
            }
            let distance = super::scan::edit_distance(word, candidate, allowed);
            if distance <= allowed && best.is_none_or(|(_, d)| distance < d) {
                best = Some((candidate.as_str(), distance));
            }
        }
        let (found, _) = best?;
        self.singles.get(found).map(Vec::as_slice)
    }
}

/// The compiled table, built once.
pub(super) fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(Table::build)
}
