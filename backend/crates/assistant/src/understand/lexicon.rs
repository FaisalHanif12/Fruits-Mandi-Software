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
            "navigate", "screen par", "screen pe", "page par", "page pe", "screen kholo", "کھولو",
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
            "kar do", "kardo", "change karo", "change kar do", "update karo", "update kar do",
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
            "reduce", "zyada kaise", "kam kaise", "بڑھائیں", "بڑھانے", "بڑھاؤں", "بڑھاو", "بڑھاؤ",
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
            "kis se wasooli", "kisko wasooli", "pehle kis", "کس سے پہلے", "کسے پہلے", "کس کو پہلے",
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
            "kitna", "kitni", "kitne", "kitnay", "kitnaa", "kitnee", "kitnaah", "kitnon", "kitana",
            "kitani", "kitane", "kitnay", "kitno", "how much", "how many", "kitna hai", "کتنا",
            "کتنی", "کتنے", "کتنوں", "کتنیں",
        ],
    ),
    (
        Cue::Many,
        &[
            "kis ka", "kis ki", "kis ke", "kya kya", "kaun kaun", "kon kon", "kaun se", "kon se", "konse", "kaunse", "kaun si",
            "konsi", "kaunsi", "kis kis", "kisne kisne", "list", "lists", "fehrist", "fihrist",
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
            "سب سے زیادہ", "سب سے بڑا", "سب سے بڑی", "زیادہ سے زیادہ", "ٹاپ",
        ],
    ),
    (
        Cue::Least,
        &[
            "sab se kam", "sabse kam", "sab say kam", "kam se kam", "lowest", "smallest", "least",
            "minimum", "min", "worst", "sab se chota", "sab se choti", "سب سے کم", "کم سے کم",
            "سب سے چھوٹا",
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
            "teeno", "mila kar", "milakar", "mila ke", "دونوں", "تینوں", "ملا کر",
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
            "شراکت دار", "شریک", "پارٹنر",
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
            "trip", "trips", "safar", "market safar", "market trip", "سفر", "مارکیٹ سفر",
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
        Cue::Guests,
        &[
            "mehman", "mehmano", "mehmanon", "mehmaan", "guest", "guests", "مہمان", "مہمانوں",
        ],
    ),
    (
        Cue::RoleItem,
        &[
            "item", "qisam", "kisam", "qism", "maal ki qisam", "قسم", "قسم مال", "آئٹم",
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
            "beecha", "beeche", "bika", "bike", "biki", "bikta", "sold", "sell", "sells", "selling",
            "بیچا", "بیچے", "بیچی", "بیچنا", "بیچتا", "بیچتے", "بکا", "بکے", "بکی",
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
            "bakaye", "بقایا", "بکایا", "لینا", "دینا", "کتنا دے گا", "دے گا",
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
        &["dues", "receivable", "receivables", "ادھاری بقایا"],
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
            "سٹاک", "بچا مال", "بچا ہوا مال",
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
            "سرمایہ", "سرمایہ کاری",
        ],
    ),
    (
        Cue::PartnerTotal,
        &[
            "kul raqam", "kul rakam", "total raqam", "partner total", "khalis", "khalees",
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
            "کمائی",
        ],
    ),
    (
        Cue::Expense,
        &[
            "kharcha", "kharch", "kharche", "kharchay", "kharcho", "expense", "expenses",
            "spending", "خرچہ", "خرچ", "خرچے", "اخراجات",
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
        &["phal", "phalon", "fruit", "fruits", "پھل", "پھلوں"],
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
        &["roznamcha", "roznamchah", "roznama", "day book", "daybook", "روزنامچہ", "روزنامہ"],
    ),
    (
        Cue::TrialBalance,
        &["trial balance", "trial", "ٹرائل بیلنس", "ٹرائل"],
    ),
    (
        Cue::ByCity,
        &[
            "city wise", "shehar wise", "shahar wise", "sheher wise", "bahisab shehar",
            "shehar ke hisab se", "city ke hisab se", "بحساب شہر", "شہر وار", "شہر کے حساب سے",
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
        &["result", "natija", "nateeja", "nataij", "نتیجہ", "نتائج"],
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
    "ziyada", "kam", "thora", "thoda", "kitni", "kitne", "ka", "ke", "ko", "اور", "کا", "کی", "کے",
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
        for table in [INTENT, ROLES, FIGURES, OUTSIDE] {
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
