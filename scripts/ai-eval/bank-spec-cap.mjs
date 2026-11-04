// Facet CAP of the team bank (plan 55 Section 11): the questions that fill the cells of the
// capability matrix (scripts/ai-eval/capability.tsv) that had no bank question. Each is a group of
// three wordings, Roman Urdu, Urdu and English, with the figure from the oracle where it has one.
//
// This file holds the screens: every one of the 30 screens the assistant can open or write to
// (EntryScreenName in backend/crates/assistant/src/model_answer.rs).
//
//   OPEN   "bikri book kholo", "open the cash book": the assistant opens the screen. It ends as an answer
//          with the guide's words and the screen route (`entry_route`), with no model call.
//   WRITE  "wasooli likh do", "write a payment": the assistant never writes (PRD FR-7.72). It ends as
//          change_refused with the entry screen that does it (plan 55 ruling).
//
// The reports and views that need oracle figures are in bank-spec-cap-reports.mjs.

const r = (text) => ['roman', text];
const u = (text) => ['urdu', text];
const e = (text) => ['en', text];

/** screen -> [guide topic, Roman Urdu, Urdu, English]. */
const SCREENS = {
  bikri: ['bikri_book', 'bikri book', 'بکری بک', 'the sale book'],
  wasooli: ['receipt', 'wasooli', 'وصولی', 'the receipt screen'],
  adaigi: ['payment', 'adaigi', 'ادائیگی', 'the payment screen'],
  peshgi: ['advance', 'peshgi', 'پیشگی', 'the advance screen'],
  kharcha: ['shop_expense', 'kharcha', 'خرچہ', 'the expense screen'],
  udhar_raqam: ['money_lent', 'udhar raqam', 'ادھار رقم', 'money lent'],
  alag_khata: ['separate_khata', 'alag khata', 'الگ کھاتہ', 'the separate khata'],
  sarmaya: ['partner_capital', 'sarmaya', 'سرمایہ', 'partner capital'],
  phal: ['fruit_taken', 'phal', 'پھل', 'fruit taken'],
  general: ['general_journal', 'general journal', 'جنرل', 'the journal'],
  account: ['accounts', 'naya khata', 'نیا کھاتہ', 'a new account'],
  dashboard: ['dashboard', 'dashboard', 'ڈیش بورڈ', 'the dashboard'],
  settings: ['settings', 'settings', 'ترتیبات', 'settings'],
  masters: ['masters', 'index', 'انڈکس', 'the index'],
  accounts: ['accounts', 'khaton ki list', 'کھاتوں کی فہرست', 'the accounts list'],
  commission_rules: ['commission_rules', 'commission rules', 'کمیشن اصول', 'commission rules'],
  today_sales: ['today_sales', 'aaj ki bikri ki list', 'آج کی بکری', "today's sales list"],
  unsold_stock: ['unsold_stock', 'baqi maal', 'باقی مال', 'unsold stock'],
  pukhta_khata: ['pukhta_khata', 'pukhta khata', 'پختہ کھاتہ', 'pukhta khata'],
  collection_list: ['collection_list', 'agrahi list', 'اگراہی فہرست', 'the collection list'],
  cash_book: ['cash_book', 'rokar book', 'روکڑ بک', 'the cash book'],
  gardens: ['gardens', 'baghat', 'باغات', 'gardens'],
  market_trips: ['market_trips', 'market safar', 'مارکیٹ سفر', 'market trips'],
  partners: ['partners', 'shirakat daar', 'شراکت دار', 'the partners screen'],
  month_close: ['month_close', 'mahina band', 'مہینہ بند', 'month close'],
  day_close: ['day_close', 'din band', 'دن بند', 'day close'],
  reports: ['reports', 'reports', 'رپورٹ', 'reports'],
  dead_amount: ['dead_amount', 'doobi raqam', 'ڈوبی رقم', 'the dead amount'],
  analytics: ['analytics', 'tajziyat', 'تجزیات', 'analytics'],
  backup: ['backup', 'backup', 'بیک اپ', 'backup'],
};

// The entry screens: a request to write is refused with the screen that does it.
const ENTRY = [
  'bikri',
  'wasooli',
  'adaigi',
  'peshgi',
  'kharcha',
  'udhar_raqam',
  'alag_khata',
  'sarmaya',
  'phal',
  'general',
  'account',
];

const OPEN_ROMAN = [
  '{x} kholo',
  '{x} screen dikhao',
  'mujhe {x} par le chalo',
  '{x} wala page kholna hai',
];
const OPEN_URDU = [
  '{x} کھولیں',
  '{x} کی اسکرین دکھائیں',
  'مجھے {x} پر لے چلیں',
  '{x} والا صفحہ کھولنا ہے',
];
const OPEN_EN = ['open {x}', 'show me {x}', 'take me to {x}', 'I want to open {x}'];

const WRITE_ROMAN = ['{x} likh do', '{x} ki entry karni hai', '{x} darj karo'];
const WRITE_URDU = ['{x} لکھ دو', '{x} کی اندراج کرنی ہے', '{x} درج کریں'];
const WRITE_EN = ['write {x}', 'enter {x}', 'record {x}'];

const fill = (template, x) => template.replace('{x}', x);

const OPEN = Object.entries(SCREENS)
  .filter(([screen]) => screen !== 'account')
  .map(([screen, [topic, roman, urdu, en]], i) => ({
    g: `CP-open-${screen}`,
    cat: 'D',
    section: 'help',
    kind: 'help',
    world: 'year',
    outcome: 'answered',
    tools: 'app_help',
    filters: `facet=CAP;topic=${topic};route=${screen}`,
    figures: '',
    source: 'capability.tsv: open a screen',
    q: [
      r(fill(OPEN_ROMAN[i % 4], roman)),
      u(fill(OPEN_URDU[(i + 1) % 4], urdu)),
      e(fill(OPEN_EN[(i + 2) % 4], en)),
    ],
  }));

const WRITE = ENTRY.map((screen, i) => {
  const [, roman, urdu, en] = SCREENS[screen];
  return {
    g: `CP-write-${screen}`,
    cat: 'E',
    section: 'change',
    kind: 'change',
    world: 'year',
    outcome: 'change_refused',
    tools: '',
    filters: `facet=CAP;route=${screen}`,
    figures: '',
    source: 'capability.tsv: a request to write',
    q: [
      r(fill(WRITE_ROMAN[i % 3], roman)),
      u(fill(WRITE_URDU[(i + 1) % 3], urdu)),
      e(fill(WRITE_EN[(i + 2) % 3], en)),
    ],
  };
});

export default [...OPEN, ...WRITE];
