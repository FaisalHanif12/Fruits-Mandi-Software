// The ITEM AXIS of the capability matrix (Umar, 06-11-2026, "thousands of question types on real-time item
// data"): every measure about an item, against every kind of subject, in every period, as a full cross product.
//
//   measures  sale (gross), quantity, seller commission, buyer commission, laga, total commission, average rate,
//             last rate, buyers (count, top), sellers (count, top), rank among the items, the top item of a group,
//             against the period before, best month and the monthly trend, stock left, forecast, print, open
//   subjects  one fruit, one vegetable, the fruit group, the vegetable group, all the items, an item for one
//             party (Faisal), an item of one quality (plan 58), an item that is NOT in the books (Capsicum), and
//             an item that IS in the books with no sale in the period
//   periods   today, yesterday, two named days, this and last week, this and last month, two named months, this
//             and last year, a range, and "ab tak" (14 periods; the period before each is the baseline of a
//             comparison)
//
// Every cell ends as one of:
//   A  answerable: one bank question (one wording, the language rotating Roman Urdu, Urdu, English, and two
//      kinds of mixed wording; about a quarter misspelled)
//   X  not a question (it has no meaning: "the top buyer of an item for one buyer")
//   N  not by design (needs the lead's approval; the reason is in the file)
//   G  a gap, with an owner
//
// The facts of every figure are the oracle's `ix.*` facts (oracle.rs `item_axis_facts`: view A, B and C and SQL
// on the books), never the assistant's tools. Which items sold in which period is the static table
// `item-axis-sold.json`, written from an oracle dump by `build-capability-items.mjs --sold-from <dump.json>`
// (a cell whose period is not in the table yet waits as G for the oracle job).

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
export const SOLD_FILE = join(HERE, 'item-axis-sold.json');

// ---- the words ---------------------------------------------------------------------------------------------

/** The items of the year books. r Roman Urdu, u Urdu, e English; x the misspelled form of each. */
export const ITEMS = [
  {
    key: 'banana',
    group: 'fruit',
    en: 'Banana',
    r: 'kele',
    u: 'کیلا',
    e: 'banana',
    x: { r: 'kaile', u: 'کیلہ', e: 'bananna' },
  },
  {
    key: 'apple',
    group: 'fruit',
    en: 'Apple',
    r: 'seb',
    u: 'سیب',
    e: 'apple',
    x: { r: 'saib', u: 'صیب', e: 'appel' },
  },
  {
    key: 'mosambi',
    group: 'fruit',
    en: 'Mosambi',
    r: 'mosambi',
    u: 'موسمی',
    e: 'mosambi',
    x: { r: 'musami', u: 'مسمی', e: 'mosambbi' },
  },
  {
    key: 'mango',
    group: 'fruit',
    en: 'Mango',
    r: 'aam',
    u: 'آم',
    e: 'mango',
    x: { r: 'am', u: 'عام', e: 'mangoo' },
  },
  {
    key: 'grapes',
    group: 'fruit',
    en: 'Grapes',
    r: 'angoor',
    u: 'انگور',
    e: 'grapes',
    x: { r: 'angor', u: 'انگر', e: 'grapse' },
  },
  {
    key: 'potato',
    group: 'vegetable',
    en: 'Potato',
    r: 'aloo',
    u: 'آلو',
    e: 'potato',
    x: { r: 'alu', u: 'الو', e: 'potatoe' },
  },
  {
    key: 'radish',
    group: 'vegetable',
    en: 'Radish',
    r: 'mooli',
    u: 'مولی',
    e: 'radish',
    x: { r: 'muli', u: 'مولي', e: 'radsih' },
  },
  {
    key: 'tomato',
    group: 'vegetable',
    en: 'Tomato',
    r: 'tamatar',
    u: 'ٹماٹر',
    e: 'tomato',
    x: { r: 'tamater', u: 'ٹماتر', e: 'tomatto' },
  },
  {
    key: 'onion',
    group: 'vegetable',
    en: 'Onion',
    r: 'piyaz',
    u: 'پیاز',
    e: 'onion',
    x: { r: 'pyaz', u: 'پیاض', e: 'onoin' },
  },
  {
    key: 'saag',
    group: 'vegetable',
    en: 'Saag',
    r: 'saag',
    u: 'ساگ',
    e: 'saag',
    x: { r: 'sag', u: 'ساک', e: 'sag' },
  },
];

/** The two groups and all the items, as a unit of the facts (`ix.<period>.<unit>.*`). */
const UNITS = {
  fruit: { r: 'phal', u: 'پھل', e: 'fruit', x: { r: 'fal', u: 'پھال', e: 'frut' } },
  vegetable: {
    r: 'sabzi',
    u: 'سبزی',
    e: 'vegetables',
    x: { r: 'sabji', u: 'سبجی', e: 'vegitables' },
  },
  all: {
    r: 'tamam maal',
    u: 'تمام مال',
    e: 'all items',
    x: { r: 'tamaam mal', u: 'تمام مل', e: 'al items' },
  },
};

/** An item that is not in the books (Capsicum, shimla mirch). */
const CAPSICUM = {
  en: 'Capsicum',
  r: 'shimla mirch',
  u: 'شملہ مرچ',
  e: 'capsicum',
  x: { r: 'shimla mirchh', u: 'شملا مرچ', e: 'capsicom' },
};

/** The party of the "an item for one party" subject, and his two items. */
const FAISAL = {
  en: 'Faisal',
  r: 'Faisal',
  u: 'فیصل',
  e: 'Faisal',
  x: { r: 'Faisl', u: 'فیسل', e: 'Faysal' },
};
export const PARTY_ITEMS = ['banana', 'potato'];

/**
 * The periods that are asked. `prev` is the baseline of "against ...", `p*` its words; `cal` the oracle's
 * calendar keys of the print dates (`none`: the plan gives no dates, the whole books).
 */
export const PERIODS = [
  {
    id: 'today',
    r: 'aaj',
    u: 'آج',
    e: 'today',
    prev: 'yesterday',
    pr: 'kal',
    pu: 'کل',
    pe: 'yesterday',
    cal: ['today', 'today'],
  },
  {
    id: 'yesterday',
    r: 'kal',
    u: 'کل',
    e: 'yesterday',
    prev: 'd2026_11_03',
    pr: 'parson',
    pu: 'پرسوں',
    pe: 'the day before',
    cal: ['yesterday', 'yesterday'],
  },
  {
    id: 'd2026_11_03',
    r: '3 November ko',
    u: '3 نومبر کو',
    e: 'on 3 November',
    cal: ['nov_03', 'nov_03'],
  },
  {
    id: 'd2026_10_14',
    r: '14 October ko',
    u: '14 اکتوبر کو',
    e: 'on 14 October',
    prev: 'd2026_10_13',
    pr: '13 October',
    pu: '13 اکتوبر',
    pe: '13 October',
    cal: ['oct_14', 'oct_14'],
  },
  {
    id: 'week',
    r: 'is hafte',
    u: 'اس ہفتے',
    e: 'this week',
    prev: 'last_week',
    pr: 'pichle hafte',
    pu: 'پچھلے ہفتے',
    pe: 'last week',
    cal: ['week_start', 'today'],
  },
  {
    id: 'last_week',
    r: 'pichle hafte',
    u: 'پچھلے ہفتے',
    e: 'last week',
    prev: 'last_week2',
    pr: 'us se pichle hafte',
    pu: 'اس سے پچھلے ہفتے',
    pe: 'the week before',
    cal: ['last_week_start', 'last_week_end'],
  },
  {
    id: 'month',
    r: 'is mahine',
    u: 'اس مہینے',
    e: 'this month',
    prev: 'last_month',
    pr: 'pichle mahine',
    pu: 'پچھلے مہینے',
    pe: 'last month',
    cal: ['month_start', 'today'],
  },
  {
    id: 'last_month',
    r: 'pichle mahine',
    u: 'پچھلے مہینے',
    e: 'last month',
    prev: 'm2026_09',
    pr: 'September',
    pu: 'ستمبر',
    pe: 'September',
    cal: ['last_month_start', 'last_month_end'],
  },
  {
    id: 'm2026_09',
    r: 'September mein',
    u: 'ستمبر میں',
    e: 'in September',
    prev: 'm2026_08',
    pr: 'August',
    pu: 'اگست',
    pe: 'August',
    cal: ['sep_start', 'sep_end'],
  },
  {
    id: 'm2026_08',
    r: 'August mein',
    u: 'اگست میں',
    e: 'in August',
    cal: ['aug_start', 'aug_end'],
  },
  {
    id: 'year',
    r: 'is saal',
    u: 'اس سال',
    e: 'this year',
    prev: 'last_year',
    pr: 'pichle saal',
    pu: 'پچھلے سال',
    pe: 'last year',
    cal: ['year_start', 'today'],
  },
  {
    id: 'last_year',
    r: 'pichle saal',
    u: 'پچھلے سال',
    e: 'last year',
    prev: 'last_year2',
    pr: 'us se pichle saal',
    pu: 'اس سے پچھلے سال',
    pe: 'the year before',
    cal: ['last_year_start', 'last_year_end'],
  },
  {
    id: 'r2026_10_01_2026_10_15',
    r: '1 se 15 October tak',
    u: '1 سے 15 اکتوبر تک',
    e: 'from 1 to 15 October',
    prev: 'r2026_09_16_2026_09_30',
    pr: '16 se 30 September',
    pu: '16 سے 30 ستمبر',
    pe: '16 to 30 September',
    cal: ['oct_01', 'oct_15'],
  },
  { id: 'up_to_now', r: 'ab tak', u: 'اب تک', e: 'so far', cal: ['first_day', 'today'] },
];
const MONTHLY = new Set(['year', 'last_year', 'up_to_now']);

// ---- the wordings ------------------------------------------------------------------------------------------
// r Roman Urdu, u Urdu, e English, mr Roman Urdu with English words, mu Urdu with English words. {i} the item or
// the group, {p} the period, {v} the period before, {pt} the party.

const T = {
  gross: {
    r: ['{p} {i} ki bikri kitni hui', '{i} kitne ka bika {p}'],
    u: ['{p} {i} کی بکری کتنی ہوئی', '{i} کتنے کا بکا {p}'],
    e: ['sales of {i} {p}', 'how much was sold of {i} {p}'],
    mr: ['{p} {i} ki sale kitni rahi'],
    mu: ['{i} کی sale {p} کتنی رہی'],
    party: {
      r: ['{p} {pt} ne {i} kitne ka khareeda'],
      u: ['{p} {pt} نے {i} کتنے کا خریدا'],
      e: ['how much {i} did {pt} buy {p}'],
      mr: ['{p} {pt} ne {i} ki purchase kitni ki'],
      mu: ['{pt} نے {p} {i} کی purchase کتنی کی'],
    },
  },
  qty: {
    r: ['{p} {i} ke kitne nag bike', '{i} ka {p} kitna nag bika'],
    u: ['{p} {i} کے کتنے نگ بکے', '{i} کا {p} کتنا نگ بکا'],
    e: ['quantity of {i} sold {p}', 'how many nag of {i} were sold {p}'],
    mr: ['{p} {i} ki quantity kitni thi'],
    mu: ['{i} کی quantity {p} کتنی تھی'],
    party: {
      r: ['{p} {pt} ne {i} ke kitne nag liye'],
      u: ['{p} {pt} نے {i} کے کتنے نگ لیے'],
      e: ['how many nag of {i} did {pt} take {p}'],
      mr: ['{p} {pt} ne {i} ki quantity kitni li'],
      mu: ['{pt} نے {p} {i} کی quantity کتنی لی'],
    },
  },
  comm_seller: {
    r: ['{p} {i} par bepari ka commission kitna bana', '{i} ka {p} seller commission kitna hai'],
    u: ['{p} {i} پر بیوپاری کا کمیشن کتنا بنا', '{i} کا {p} سیلر کمیشن کتنا ہے'],
    e: ['seller commission on {i} {p}', 'how much seller commission did {i} make {p}'],
    mr: ['{p} {i} par seller ka commission kitna bana'],
    mu: ['{i} پر {p} seller کا کمیشن کتنا بنا'],
    party: {
      r: ['{p} {pt} ke {i} par bepari ka commission kitna bana'],
      u: ['{p} {pt} کے {i} پر بیوپاری کا کمیشن کتنا بنا'],
      e: ['seller commission on the {i} {pt} bought {p}'],
      mr: ['{p} {pt} ke {i} par seller commission kitna bana'],
      mu: ['{pt} کے {i} پر {p} seller کمیشن کتنا بنا'],
    },
  },
  comm_buyer: {
    r: ['{p} {i} par gahak se kitna commission mila', '{i} ka {p} buyer commission kitna hai'],
    u: ['{p} {i} پر گاہک سے کتنا کمیشن ملا', '{i} کا {p} بائر کمیشن کتنا ہے'],
    e: ['buyer commission on {i} {p}', 'how much buyer commission did {i} make {p}'],
    mr: ['{p} {i} par buyer se kitna commission mila'],
    mu: ['{i} پر {p} buyer سے کتنا کمیشن ملا'],
    party: {
      r: ['{p} {pt} se {i} par kitna commission liya'],
      u: ['{p} {pt} سے {i} پر کتنا کمیشن لیا'],
      e: ['buyer commission charged to {pt} on {i} {p}'],
      mr: ['{p} {pt} se {i} par buyer commission kitna laga'],
      mu: ['{pt} سے {p} {i} پر buyer کمیشن کتنا لگا'],
    },
  },
  laga: {
    r: ['{p} {i} par laga kitna laga', '{i} ka {p} ka laga batao'],
    u: ['{p} {i} پر لاگا کتنا لگا', '{i} کا {p} کا لاگا بتائیں'],
    e: ['laga charged on {i} {p}', 'how much laga did {i} carry {p}'],
    mr: ['{p} {i} par laga charge kitna hua'],
    mu: ['{i} پر {p} laga charge کتنا ہوا'],
    party: {
      r: ['{p} {pt} ke {i} par laga kitna laga'],
      u: ['{p} {pt} کے {i} پر لاگا کتنا لگا'],
      e: ['laga on the {i} {pt} bought {p}'],
      mr: ['{p} {pt} ke {i} par laga charge kitna hua'],
      mu: ['{pt} کے {i} پر {p} laga charge کتنا ہوا'],
    },
  },
  comm_total: {
    r: ['{p} {i} se kul commission kitna bana', '{i} ka {p} total commission kitna hai'],
    u: ['{p} {i} سے کل کمیشن کتنا بنا', '{i} کا {p} ٹوٹل کمیشن کتنا ہے'],
    e: ['total commission from {i} {p}', 'how much commission did {i} make {p}'],
    mr: ['{p} {i} se total commission kitna bana'],
    mu: ['{i} سے {p} total کمیشن کتنا بنا'],
    party: {
      r: ['{p} {pt} ke {i} se kul commission kitna bana'],
      u: ['{p} {pt} کے {i} سے کل کمیشن کتنا بنا'],
      e: ['total commission on the {i} {pt} bought {p}'],
      mr: ['{p} {pt} ke {i} se total commission kitna bana'],
      mu: ['{pt} کے {i} سے {p} total کمیشن کتنا بنا'],
    },
  },
  avg_rate: {
    r: ['{i} ka {p} ka ausat rate kya raha', '{p} {i} ka average rate kitna tha'],
    u: ['{i} کا {p} کا اوسط ریٹ کیا رہا', '{p} {i} کا ایوریج ریٹ کتنا تھا'],
    e: ['average rate of {i} {p}', 'what was the average rate of {i} {p}'],
    mr: ['{p} {i} ka average rate kya tha'],
    mu: ['{i} کا {p} average ریٹ کیا تھا'],
    party: {
      r: ['{pt} ko {p} {i} ka ausat rate kya laga'],
      u: ['{pt} کو {p} {i} کا اوسط ریٹ کیا لگا'],
      e: ['average rate of {i} for {pt} {p}'],
      mr: ['{pt} ko {p} {i} ka average rate kya laga'],
      mu: ['{pt} کو {p} {i} کا average ریٹ کیا لگا'],
    },
  },
  last_rate: {
    r: ['{i} ka {p} ka aakhri rate kya tha', '{p} {i} ka last rate batao'],
    u: ['{i} کا {p} کا آخری ریٹ کیا تھا', '{p} {i} کا آخری ریٹ بتائیں'],
    e: ['last rate of {i} {p}', 'what was the last rate of {i} {p}'],
    mr: ['{p} {i} ka last rate kya tha'],
    mu: ['{i} کا {p} last ریٹ کیا تھا'],
    // The lead's ruling of 06-11-2026: "akram ko kela aakhri kis rate pe diya", read from R13 with the item.
    party: {
      r: ['{pt} ko {p} {i} aakhri kis rate par diya'],
      u: ['{pt} کو {p} {i} آخری کس ریٹ پر دیا'],
      e: ['at what rate did we last give {i} to {pt} {p}'],
      mr: ['{pt} ko {p} {i} ka last rate kya lagaya'],
      mu: ['{pt} کو {p} {i} کا last ریٹ کیا لگایا'],
    },
  },
  buyers_n: {
    r: ['{p} {i} kitne gahakon ne khareeda', '{i} ke {p} kitne gahak the'],
    u: ['{p} {i} کتنے گاہکوں نے خریدا', '{i} کے {p} کتنے گاہک تھے'],
    e: ['how many buyers bought {i} {p}', 'number of buyers of {i} {p}'],
    mr: ['{p} {i} kitne buyers ne khareeda'],
    mu: ['{i} {p} کتنے buyers نے خریدا'],
  },
  sellers_n: {
    r: ['{p} {i} kitne bepariyon ka aaya', '{i} ke {p} kitne bepari the'],
    u: ['{p} {i} کتنے بیوپاریوں کا آیا', '{i} کے {p} کتنے بیوپاری تھے'],
    e: ['how many sellers sent {i} {p}', 'number of sellers of {i} {p}'],
    mr: ['{p} {i} kitne sellers ka aaya'],
    mu: ['{i} {p} کتنے sellers کا آیا'],
    party: {
      r: ['{pt} ne {p} {i} kitne bepariyon se liya'],
      u: ['{pt} نے {p} {i} کتنے بیوپاریوں سے لیا'],
      e: ['how many sellers did {pt} get {i} from {p}'],
      mr: ['{pt} ne {p} {i} kitne sellers se liya'],
      mu: ['{pt} نے {p} {i} کتنے sellers سے لیا'],
    },
  },
  top_buyer: {
    r: ['{p} {i} sab se zyada kis gahak ne khareeda', 'kis gahak ne {p} sab se zyada {i} liya'],
    u: ['{p} {i} سب سے زیادہ کس گاہک نے خریدا', 'کس گاہک نے {p} سب سے زیادہ {i} لیا'],
    e: ['which buyer bought the most {i} {p}', 'top buyer of {i} {p}'],
    mr: ['{p} {i} sab se zyada kis buyer ne liya'],
    mu: ['{p} {i} سب سے زیادہ کس buyer نے لیا'],
  },
  top_seller: {
    r: ['{p} {i} sab se zyada kis bepari ka aaya', 'kis bepari ne {p} sab se zyada {i} bheja'],
    u: ['{p} {i} سب سے زیادہ کس بیوپاری کا آیا', 'کس بیوپاری نے {p} سب سے زیادہ {i} بھیجا'],
    e: ['which seller sent the most {i} {p}', 'top seller of {i} {p}'],
    mr: ['{p} {i} sab se zyada kis seller ka aaya'],
    mu: ['{p} {i} سب سے زیادہ کس seller کا آیا'],
    party: {
      r: ['{pt} ko {p} {i} sab se zyada kis bepari ne diya'],
      u: ['{pt} کو {p} {i} سب سے زیادہ کس بیوپاری نے دیا'],
      e: ['which seller gave {pt} the most {i} {p}'],
      mr: ['{pt} ko {p} {i} sab se zyada kis seller ne diya'],
      mu: ['{pt} کو {p} {i} سب سے زیادہ کس seller نے دیا'],
    },
  },
  rank: {
    r: ['{p} bikri mein {i} ka kaun sa number raha', '{p} {i} bikri mein kitne number par tha'],
    u: ['{p} بکری میں {i} کا کون سا نمبر رہا', '{p} {i} بکری میں کس نمبر پر تھا'],
    e: ['where did {i} rank among the items by sales {p}', 'what rank was {i} in sales {p}'],
    mr: ['{p} sale mein {i} ka rank kya raha'],
    mu: ['{p} sale میں {i} کا rank کیا رہا'],
  },
  top_item: {
    r: [
      '{p} {i} mein sab se zyada kaunsa maal bika',
      '{p} {i} mein sab se zyada bikne wala maal kaunsa hai',
    ],
    u: [
      '{p} {i} میں سب سے زیادہ کون سا مال بکا',
      '{p} {i} میں سب سے زیادہ بکنے والا مال کون سا ہے',
    ],
    e: ['which item sold the most in {i} {p}', 'best selling item of {i} {p}'],
    mr: ['{p} {i} mein sab se zyada kaunsa item sell hua'],
    mu: ['{p} {i} میں سب سے زیادہ کون سا item sell ہوا'],
  },
  vs: {
    r: ['{p} {i} ki bikri {v} se kitni zyada ya kam rahi'],
    u: ['{p} {i} کی بکری {v} سے کتنی زیادہ یا کم رہی'],
    e: ['sales of {i} {p} against {v}'],
    mr: ['{p} {i} ki sale {v} se kitni up ya down rahi'],
    mu: ['{p} {i} کی sale {v} سے کتنی up یا down رہی'],
    party: {
      r: ['{p} {pt} ki {i} ki khareedari {v} se kitni zyada ya kam rahi'],
      u: ['{p} {pt} کی {i} کی خریداری {v} سے کتنی زیادہ یا کم رہی'],
      e: ['{pt} purchases of {i} {p} against {v}'],
      mr: ['{p} {pt} ki {i} ki purchase {v} se kitni up ya down rahi'],
      mu: ['{p} {pt} کی {i} کی purchase {v} سے کتنی up یا down رہی'],
    },
  },
  best_month: {
    r: ['{p} kis mahine mein {i} ki sab se zyada bikri hui'],
    u: ['{p} کس مہینے میں {i} کی سب سے زیادہ بکری ہوئی'],
    e: ['which month had the most sales of {i} {p}'],
    mr: ['{p} kis month mein {i} ki sab se zyada sale hui'],
    mu: ['{p} کس month میں {i} کی سب سے زیادہ sale ہوئی'],
  },
  trend: {
    r: ['{p} {i} ki mahine ke hisab se bikri dikhao'],
    u: ['{p} {i} کی مہینے کے حساب سے بکری دکھائیں'],
    e: ['show the monthly sales of {i} {p}'],
    mr: ['{p} {i} ki monthly sale dikhao'],
    mu: ['{p} {i} کی monthly sale دکھائیں'],
  },
  stock: {
    r: ['{i} ka kitna maal baqi hai', 'baqi maal mein {i} kitna hai'],
    u: ['{i} کا کتنا مال باقی ہے', 'باقی مال میں {i} کتنا ہے'],
    e: ['how much of {i} is left unsold', 'unsold stock of {i}'],
    mr: ['{i} ka kitna stock bacha hai'],
    mu: ['{i} کا کتنا stock بچا ہے'],
  },
  forecast: {
    r: ['{i} ka agle 3 mahine ka commission kitna hoga'],
    u: ['{i} کا اگلے 3 مہینوں کا کمیشن کتنا ہو گا'],
    e: ['estimate the {i} commission for the next 3 months'],
    mr: ['{i} ka next 3 months ka commission estimate kya hai'],
    mu: ['{i} کا next 3 months کا کمیشن estimate کیا ہے'],
  },
  print: {
    r: ['{i} ki {p} ki bikri ka print', '{p} {i} ki bikri print karo'],
    u: ['{i} کی {p} کی بکری کا پرنٹ', '{p} {i} کی بکری پرنٹ کریں'],
    e: ['print the {i} sales {p}', 'print sales of {i} {p}'],
    mr: ['{i} ki {p} ki sale ka print'],
    mu: ['{i} کی {p} کی sale کا print'],
    party: {
      r: ['{pt} ki {p} ki {i} ki bikri ka print'],
      u: ['{pt} کی {p} کی {i} کی بکری کا پرنٹ'],
      e: ['print {pt} {i} purchases {p}'],
      mr: ['{pt} ki {p} ki {i} sale ka print'],
      mu: ['{pt} کی {p} کی {i} sale کا print'],
    },
    all: {
      r: ['item wise bikri {p} ka print'],
      u: ['قسم وار بکری {p} کا پرنٹ'],
      e: ['print the item wise sales {p}'],
      mr: ['item wise sale {p} ka print'],
      mu: ['item wise sale {p} کا print'],
    },
  },
  open: {
    r: ['{i} ka item wise tajziya kholo'],
    u: ['{i} کا قسم وار تجزیہ کھولیں'],
    e: ['open the item wise analytics for {i}'],
    mr: ['{i} ka item wise analytics kholo'],
    mu: ['{i} کا item wise analytics کھولیں'],
    party: {
      r: ['{pt} ke {i} ka item wise tajziya kholo'],
      u: ['{pt} کے {i} کا قسم وار تجزیہ کھولیں'],
      e: ['open the item wise analytics of {i} for {pt}'],
      mr: ['{pt} ke {i} ka item wise analytics kholo'],
      mu: ['{pt} کے {i} کا item wise analytics کھولیں'],
    },
    all: {
      r: ['item wise tajziya kholo'],
      u: ['قسم وار تجزیہ کھولیں'],
      e: ['open the analytics by item'],
      mr: ['item wise analytics kholo'],
      mu: ['item wise analytics کھولیں'],
    },
  },
};

// ---- the measures and what each subject does with them ----------------------------------------------------------

const G58 = [
  'G',
  'backend3 (plan 58: migration 0019 and the grade input), then robustness T4',
  'the quality of a sale line is not in the books yet',
];
const GROUP_PRINT = [
  'G',
  'backend2',
  'understand/plan.rs: "a group of items has no filter on a report link yet: the model answers" (waits for backend4\'s item_group on the links)',
];
/** The measures of a sale line that an item of one quality answers: what the lines hold (plan 58). */
const QUALITY_MEASURES = new Set([
  'gross',
  'qty',
  'comm_seller',
  'comm_buyer',
  'laga',
  'comm_total',
  'avg_rate',
  'last_rate',
  'buyers_n',
  'sellers_n',
  'top_buyer',
  'top_seller',
]);

/** What a measure does with a subject: 'A', or [status, note] / ['G', owner, note]. */
function ruleFor(measure, subject) {
  if (subject !== 'quality') return measure.subjects[subject];
  if (QUALITY_MEASURES.has(measure.id)) return 'A';
  if (measure.id === 'print' || measure.id === 'open')
    return [
      'N',
      'Umar (06-11-2026, "rest ignore"): the quality is shown on the prints, a print is not filtered by quality',
    ];
  return [
    'N',
    'no view groups by quality and plan 58 adds none (the lines carry it): a rank, a comparison, a month table, stock and a forecast by quality are not asked',
  ];
}

const SUBJECTS = ['fruit', 'veg', 'g_fruit', 'g_veg', 'all', 'party', 'quality', 'unknown', 'zero'];

const SALE_ROW = {
  fruit: 'A',
  veg: 'A',
  g_fruit: 'A',
  g_veg: 'A',
  all: 'A',
  party: 'A',
  quality: G58,
  unknown: 'A',
  zero: 'A',
};
const NAMED_ROW = {
  fruit: 'A',
  veg: 'A',
  g_fruit: 'A',
  g_veg: 'A',
  all: 'A',
  quality: G58,
  unknown: 'A',
  zero: 'A',
};

/**
 * period: the measure is asked in the periods; sold: it needs a sale in the period (else it is the zero
 * subject's); kind and section: the bank columns; subjects: a cell for each subject.
 */
export const MEASURES = [
  { id: 'gross', period: true, sold: false, kind: 'item', section: 'item', subjects: SALE_ROW },
  { id: 'qty', period: true, sold: false, kind: 'item', section: 'item', subjects: SALE_ROW },
  {
    id: 'comm_seller',
    period: true,
    sold: false,
    kind: 'item',
    section: 'item',
    subjects: SALE_ROW,
  },
  {
    id: 'comm_buyer',
    period: true,
    sold: false,
    kind: 'item',
    section: 'item',
    subjects: SALE_ROW,
  },
  { id: 'laga', period: true, sold: false, kind: 'laga', section: 'item', subjects: SALE_ROW },
  {
    id: 'comm_total',
    period: true,
    sold: false,
    kind: 'item',
    section: 'item',
    subjects: SALE_ROW,
  },
  {
    id: 'avg_rate',
    period: true,
    sold: true,
    kind: 'item',
    section: 'item',
    subjects: {
      fruit: 'A',
      veg: 'A',
      party: 'A',
      quality: G58,
      unknown: 'A',
      zero: 'A',
      g_fruit: [
        'N',
        'an average rate of mixed packings is not a figure of any screen (view A leaves it empty in a total row)',
      ],
      g_veg: [
        'N',
        'an average rate of mixed packings is not a figure of any screen (view A leaves it empty in a total row)',
      ],
      all: [
        'N',
        'an average rate of mixed packings is not a figure of any screen (view A leaves it empty in a total row)',
      ],
    },
  },
  {
    id: 'last_rate',
    period: true,
    sold: true,
    kind: 'item',
    section: 'item',
    subjects: {
      fruit: 'A',
      veg: 'A',
      quality: G58,
      unknown: 'A',
      zero: 'A',
      g_fruit: ['N', 'a group has no single rate'],
      g_veg: ['N', 'a group has no single rate'],
      all: ['N', 'the shop has no single rate'],
      party: 'A',
    },
  },
  {
    id: 'buyers_n',
    period: true,
    sold: true,
    kind: 'top',
    section: 'buyer',
    subjects: { ...NAMED_ROW, party: ['X', 'the buyer is the party'] },
  },
  {
    id: 'sellers_n',
    period: true,
    sold: true,
    kind: 'top',
    section: 'seller',
    subjects: { ...NAMED_ROW, party: 'A' },
  },
  {
    id: 'top_buyer',
    period: true,
    sold: true,
    kind: 'top',
    section: 'buyer',
    subjects: { ...NAMED_ROW, party: ['X', 'the buyer is the party'] },
  },
  {
    id: 'top_seller',
    period: true,
    sold: true,
    kind: 'top',
    section: 'seller',
    subjects: { ...NAMED_ROW, party: 'A' },
  },
  {
    id: 'rank',
    period: true,
    sold: true,
    kind: 'top',
    section: 'item',
    subjects: {
      fruit: 'A',
      veg: 'A',
      quality: G58,
      unknown: 'A',
      zero: 'A',
      g_fruit: ['N', 'a group is not an item (the best item of a group is top_item)'],
      g_veg: ['N', 'a group is not an item (the best item of a group is top_item)'],
      all: ['X', 'the shop has no rank among items'],
      party: ['X', 'a rank among items is of the shop, not of one buyer'],
    },
  },
  {
    id: 'top_item',
    period: true,
    sold: true,
    kind: 'top',
    section: 'item',
    subjects: {
      g_fruit: 'A',
      g_veg: 'A',
      all: 'A',
      fruit: ['X', 'one item has no top item'],
      veg: ['X', 'one item has no top item'],
      party: ['X', 'one item has no top item'],
      quality: ['X', 'one item has no top item'],
      unknown: ['X', 'a group question'],
      zero: ['X', 'a group question'],
    },
  },
  {
    id: 'vs',
    period: true,
    sold: false,
    vs: true,
    kind: 'compare',
    section: 'item',
    subjects: { ...SALE_ROW },
  },
  {
    id: 'best_month',
    period: true,
    monthly: true,
    sold: true,
    kind: 'item',
    section: 'item',
    subjects: {
      fruit: 'A',
      veg: 'A',
      g_fruit: 'A',
      g_veg: 'A',
      all: 'A',
      quality: G58,
      unknown: 'A',
      zero: 'A',
      party: ['N', 'the month table of one buyer and one item (view D) keeps no oracle fact; rare'],
    },
  },
  {
    id: 'trend',
    period: true,
    monthly: true,
    sold: true,
    kind: 'item',
    section: 'item',
    subjects: {
      fruit: 'A',
      veg: 'A',
      g_fruit: 'A',
      g_veg: 'A',
      all: 'A',
      quality: G58,
      unknown: 'A',
      zero: 'A',
      party: ['N', 'the month table of one buyer and one item (view D) keeps no oracle fact; rare'],
    },
  },
  {
    id: 'stock',
    period: false,
    kind: 'stock',
    section: 'stock',
    subjects: {
      fruit: 'A',
      veg: 'A',
      g_fruit: 'A',
      g_veg: 'A',
      all: 'A',
      unknown: 'A',
      zero: 'A',
      party: ['X', 'stock has no buyer'],
      quality: ['N', 'stock is kept by lot; the quality is on the sale line'],
    },
  },
  {
    id: 'forecast',
    period: false,
    kind: 'estimates',
    section: 'forecast',
    subjects: {
      fruit: 'A',
      veg: 'A',
      all: 'A',
      unknown: 'A',
      g_fruit: [
        'N',
        'for now (the prompt is at its token limit; the estimate tool has no item group argument): on the follow-up list',
      ],
      g_veg: [
        'N',
        'for now (the prompt is at its token limit; the estimate tool has no item group argument): on the follow-up list',
      ],
      party: ['X', 'a forecast has no buyer and item together'],
      quality: ['N', 'a forecast by quality needs a history of qualities; none yet'],
      zero: ['X', 'a forecast does not depend on a period'],
    },
  },
  {
    id: 'print',
    period: true,
    sold: false,
    kind: 'print',
    section: 'print',
    subjects: {
      fruit: 'A',
      veg: 'A',
      all: 'A',
      party: 'A',
      quality: G58,
      unknown: 'A',
      zero: 'A',
      g_fruit: GROUP_PRINT,
      g_veg: GROUP_PRINT,
    },
  },
  {
    id: 'open',
    period: false,
    kind: 'print',
    section: 'print',
    subjects: {
      fruit: 'A',
      veg: 'A',
      all: 'A',
      party: 'A',
      quality: G58,
      unknown: 'A',
      g_fruit: GROUP_PRINT,
      g_veg: GROUP_PRINT,
      zero: ['X', 'an open screen does not depend on a period'],
    },
  },
];

// ---- the sold table -----------------------------------------------------------------------------------------

/** The table of which items sold in which period, and which have no stock: null when it is not written. */
function readSold() {
  if (!existsSync(SOLD_FILE)) return { periods: {}, stock0: null };
  return JSON.parse(readFileSync(SOLD_FILE, 'utf8'));
}

// ---- the cells -------------------------------------------------------------------------------------------------

const KINDS = ['r', 'u', 'e', 'mr', 'mu'];
const LANG = { r: 'roman', u: 'urdu', e: 'en', mr: 'roman', mu: 'urdu' };

const itemOf = (key) => ITEMS.find((i) => i.key === key);
const capital = (s) => s.charAt(0).toUpperCase() + s.slice(1);

function unitOfSubject(subject) {
  return { g_fruit: 'fruit', g_veg: 'vegetable', all: 'all' }[subject];
}

/** The words a cell uses for its item, in one kind of wording; misspelled when `mis`. */
function word(thing, kind, mis) {
  const script = kind === 'r' || kind === 'mr' ? 'r' : kind === 'u' || kind === 'mu' ? 'u' : 'e';
  return mis ? thing.x[script] : thing[script];
}

/**
 * The three qualities of a sale line (plan 58, PRD FR-12.42): awwal, doyam, soyam. The quality word comes before the
 * item: "awwal seb aaj kitne ka bika", "doyam kele ka aaj ka rate", "how much first quality apple was sold today".
 */
export const QUALITIES = [
  {
    key: 'first',
    r: 'awwal',
    u: 'اول',
    e: 'first quality',
    x: { r: 'awal', u: 'اوول', e: 'frist quality' },
  },
  {
    key: 'second',
    r: 'doyam',
    u: 'دوم',
    e: 'second quality',
    x: { r: 'doyem', u: 'دوؤم', e: 'secnd quality' },
  },
  {
    key: 'third',
    r: 'soyam',
    u: 'سوم',
    e: 'third quality',
    x: { r: 'soyem', u: 'سووم', e: 'thrid quality' },
  },
];

/** The templates of a measure for an item of one quality: the same words with the quality before the item. */
function qualityTemplates(base) {
  const put = (list) => list.map((t) => t.replace('{i}', '{q} {i}'));
  return Object.fromEntries(['r', 'u', 'e', 'mr', 'mu'].map((k) => [k, put(base[k])]));
}

/** The wording of one cell: [lang, text, flag?]. */
function wording(measure, subject, ctx, n, mis) {
  const kind = KINDS[n % KINDS.length];
  const base = T[measure.id];
  const set =
    subject === 'quality'
      ? qualityTemplates(base)
      : subject === 'party' && base.party
        ? base.party
        : subject === 'all' && base.all
          ? base.all
          : base;
  const templates = set[kind];
  const text = templates[Math.floor(n / KINDS.length) % templates.length];
  const script = kind === 'r' || kind === 'mr' ? 'r' : kind === 'u' || kind === 'mu' ? 'u' : 'e';
  const period = ctx.period;
  const values = {
    i: ctx.thing ? word(ctx.thing, kind, mis) : '',
    p: period ? period[script] : '',
    v: period?.prev ? period[`p${script}`] : '',
    pt: word(FAISAL, kind, false),
    q: ctx.quality ? word(ctx.quality, kind, mis) : '',
  };
  const filled = text.replace(/\{(\w+)\}/g, (_, k) => values[k] ?? '');
  return [LANG[kind], filled.replace(/\s+/g, ' ').trim(), mis ? 'mis' : undefined].filter(Boolean);
}

/** The facts the oracle gives (oracle-keys.txt): an estimate method an item has no history for is not a fact. */
const ORACLE_KEYS = new Set(
  existsSync(join(HERE, 'oracle-keys.txt'))
    ? readFileSync(join(HERE, 'oracle-keys.txt'), 'utf8').split('\n').filter(Boolean)
    : [],
);

/** The three methods of an estimate as alternatives (any of them counts), the ones the oracle has. */
const three = (prefix) =>
  ['same_pace', 'same_growth', 'same_season']
    .map((m) => `${prefix}.${m}`)
    .filter((k) => ORACLE_KEYS.size === 0 || ORACLE_KEYS.has(k))
    .map((k) => `@${k}`)
    .join('|');

/** The expected figures of a cell. Returns { figures, outcome?, extra? } or null when the facts cannot say. */
function expectation(measure, subject, ctx) {
  const p = ctx.period?.id;
  const unit = ctx.unit; // an item (lower case English), fruit, vegetable or all
  const ix = (suffix) => `@ix.${p}.${unit}.${suffix}`;
  const faisal = (suffix) => `@ix.${p}.${unit}.faisal.${suffix}`;
  const veg = subject === 'g_veg' || ctx.itemGroup === 'vegetable';
  const party = subject === 'party';
  if (subject === 'quality') {
    // The sale lines of one item and one quality (the oracle's `ixq.*` facts, plan 58 T4).
    const q = (suffix) => `@ixq.${p}.${unit}.${ctx.quality.key}.${suffix}`;
    const figures = {
      gross: `sale=${q('gross')}`,
      qty: `qty=${q('qty')}`,
      comm_seller: `comm=${q('comm_seller')}#seller`,
      comm_buyer: `comm=${q('comm_buyer')}`,
      laga: `laga=${q('laga')}`,
      comm_total: `comm=${q('comm_total')}`,
      avg_rate: `rate=${q('avg_rate')}`,
      last_rate: `last=${q('last_rate')}`,
      buyers_n: `count=${q('buyers_n')}`,
      sellers_n: `count=${q('sellers_n')}`,
      top_buyer: `top!=${q('top_buyer')}`,
      top_seller: `top!=${q('top_seller')}`,
    }[measure.id];
    return {
      figures,
      extra: [`quality=${ctx.quality.key}`, measure.id === 'laga' && veg ? 'nolaga=1' : '']
        .filter(Boolean)
        .join(';'),
    };
  }
  const pick = (own, partyKey) => (party ? faisal(partyKey ?? own) : ix(own));
  switch (measure.id) {
    case 'gross':
      return {
        figures: party
          ? `sale=${faisal('gross')}|${faisal('with_charges')}`
          : `sale=${ix('gross')}`,
      };
    case 'qty':
      return { figures: `qty=${pick('qty')}` };
    case 'comm_seller':
      return { figures: `comm=${pick('comm_seller')}#seller` };
    case 'comm_buyer':
      return { figures: `comm=${pick('comm_buyer')}` };
    case 'laga':
      return { figures: `laga=${pick('laga')}`, extra: veg ? 'nolaga=1' : '' };
    case 'comm_total':
      return { figures: `comm=${pick('comm_total')}` };
    case 'avg_rate':
      return { figures: `rate=${pick('avg_rate')}` };
    case 'last_rate':
      return { figures: `last=${party ? faisal('last_rate') : ix('last_rate')}` };
    case 'buyers_n':
      return { figures: `count=${ix('buyers_n')}` };
    case 'sellers_n':
      return { figures: `count=${party ? faisal('sellers_n') : ix('sellers_n')}` };
    case 'top_buyer':
      return { figures: `top!=${ix('top_buyer')}` };
    case 'top_seller':
      return { figures: `top!=${party ? faisal('top_seller') : ix('top_seller')}` };
    case 'rank':
      return { figures: `rank=${ix('rank')}` };
    case 'top_item':
      return { figures: `top!=${ix('top_item')}` };
    case 'vs':
      return { figures: `diff=${party ? faisal('vs_gross') : ix('vs_gross')}` };
    case 'best_month':
      return { figures: `best=${ix('best_month_gross')}` };
    case 'trend':
      return { figures: `total=${ix('gross')}` };
    case 'stock':
      return { figures: `qty=@ix.stock.${unit}` };
    case 'forecast':
      return {
        figures: (() => {
          const alts = three(
            unit === 'all' ? 'est.commission.a3' : `est_item.${unit}.commission.a3`,
          );
          return alts ? `est=${alts}` : '';
        })(),
        outcome: 'estimate',
        cat: 'C',
      };
    case 'print':
    case 'open': {
      const dates =
        measure.id === 'print'
          ? ctx.period.cal === 'none'
            ? 'print.dates=none'
            : `print.from=@cal.${ctx.period.cal[0]};print.to=@cal.${ctx.period.cal[1]}`
          : 'print.dates=none';
      const report = measure.id === 'print' ? 'R7' : 'A';
      const parts = [`print.report=${report}`];
      if (ctx.thing && unit !== 'all') parts.push(`print.item=${ctx.thing.en}`);
      if (party) parts.push('print.party=Faisal', 'print.role=buyer');
      if (measure.id === 'open') parts.push('print.intent=navigate');
      parts.push(dates, 'print.steps=1');
      return { figures: parts.join(';') };
    }
    default:
      return null;
  }
}

/**
 * Builds every cell: { id, measure, subject, period, status, owner, note, entry }. `entry` is a bank entry for an
 * answerable cell. A cell that needs the sold table and finds its period missing waits as G for the oracle job.
 */
export function buildCells() {
  const sold = readSold();
  const soldIn = (period) => {
    const list = sold.periods[period];
    return Array.isArray(list) ? new Set(list) : null;
  };
  const cells = [];
  const counters = new Map();
  MEASURES.forEach((measure, measureIndex) => {
    const periods = measure.period
      ? PERIODS.filter((p) => !measure.monthly || MONTHLY.has(p.id)).filter(
          (p) => !measure.vs || p.prev,
        )
      : [null];
    for (const subject of SUBJECTS) {
      const rule = ruleFor(measure, subject);
      if (!rule) continue;
      periods.forEach((period, periodIndex) => {
        const base = {
          measure: measure.id,
          subject,
          period: period?.id ?? '-',
        };
        const id = `IT-${measure.id}-${subject}-${period?.id ?? 'now'}`;
        if (rule !== 'A') {
          const [status, a, b] = rule;
          cells.push({
            ...base,
            id,
            status,
            owner: status === 'G' ? a : '',
            note: status === 'G' ? b : a,
          });
          return;
        }
        // The thing the question is about.
        const sale = period ? soldIn(period.id) : null;
        const prevSale = period?.prev ? soldIn(period.prev) : null;
        const needsTable =
          subject !== 'party' &&
          subject !== 'unknown' &&
          Boolean(measure.sold || subject === 'zero' || measure.vs);
        const qualityTable = subject === 'quality' && period ? sold.quality?.[period.id] : null;
        if (subject === 'quality' && period && !qualityTable) {
          cells.push({
            ...base,
            id,
            status: 'G',
            owner: 'robustness',
            note: 'waits for the oracle job: the sold table has no qualities for this period',
          });
          return;
        }
        if (period && needsTable && (!sale || (measure.vs && !prevSale))) {
          cells.push({
            ...base,
            id,
            status: 'G',
            owner: 'robustness',
            note: 'waits for the oracle job: the sold table has no row for this period',
          });
          return;
        }
        const rotate = (list) => list[(periodIndex + measureIndex) % list.length];
        let thing = null;
        let unit = null;
        let itemGroup = null;
        let quality = null;
        let gone = null; // why there is no cell
        const anyIn = (group, set) =>
          ITEMS.some((i) => (!group || i.group === group) && set.has(i.key));
        if (subject === 'fruit' || subject === 'veg') {
          const group = subject === 'fruit' ? 'fruit' : 'vegetable';
          const all = ITEMS.filter((i) => i.group === group);
          const fit =
            measure.sold || measure.vs
              ? all.filter(
                  (i) =>
                    (sale && sale.has(i.key)) || (measure.vs && prevSale && prevSale.has(i.key)),
                )
              : sale
                ? all.filter((i) => sale.has(i.key))
                : all;
          const list = fit.length ? fit : measure.sold || measure.vs ? [] : all;
          if (list.length === 0)
            gone = `no ${group} item sold in this period (the zero subject holds it)`;
          else thing = rotate(list);
          if (thing) {
            unit = thing.key;
            itemGroup = thing.group;
          }
        } else if (subject === 'zero') {
          const none = ITEMS.filter((i) => !sale || !sale.has(i.key));
          if (!period) {
            const out = (sold.stock0 ?? []).map(itemOf).filter(Boolean);
            if (sold.stock0 === null) {
              cells.push({
                ...base,
                id,
                status: 'G',
                owner: 'robustness',
                note: 'waits for the oracle job: the sold table has no list of items with no stock',
              });
              return;
            }
            if (out.length === 0) gone = 'every item has stock';
            else thing = rotate(out);
          } else if (none.length === 0) gone = 'every item sold in this period';
          else thing = rotate(none);
          if (thing) {
            unit = thing.key;
            itemGroup = thing.group;
          }
        } else if (subject === 'quality') {
          // An item and a quality that have a sale line in this period.
          const pairs = Object.entries(qualityTable ?? {}).flatMap(([item, list]) =>
            list.map((q) => [itemOf(item), QUALITIES.find((x) => x.key === q)]),
          );
          if (pairs.length === 0) gone = 'no sale line has a quality in this period';
          else [thing, quality] = rotate(pairs);
          if (thing) {
            unit = thing.key;
            itemGroup = thing.group;
          }
        } else if (subject === 'party') {
          thing = itemOf(PARTY_ITEMS[(periodIndex + measureIndex) % PARTY_ITEMS.length]);
          unit = thing.key;
          itemGroup = thing.group;
        } else if (subject === 'unknown') {
          thing = CAPSICUM;
          unit = 'capsicum';
        } else {
          unit = unitOfSubject(subject);
          thing = UNITS[unit];
          if (measure.sold || measure.vs) {
            const group = unit === 'all' ? null : unit;
            const has = (set) => set && anyIn(group, set);
            if (!(has(sale) || (measure.vs && has(prevSale))))
              gone = `nothing of ${unit} sold in this period`;
          }
        }
        if (gone) {
          cells.push({ ...base, id, status: 'X', owner: '', note: gone });
          return;
        }
        const counterKey = measure.id;
        const n = counters.get(counterKey) ?? 0;
        counters.set(counterKey, n + 1);
        const mis = n % 4 === 3;
        const ctx = { period, thing, unit, itemGroup, quality };
        const text = wording(measure, subject, ctx, n + measureIndex, mis);
        let figures = '';
        let outcome = 'answered';
        let cat = 'A';
        let extra = '';
        const filters = [`facet=ITEM`, `measure=${measure.id}`, `subject=${subject}`];
        if (period) filters.push(`when=${period.id}`);
        let { section, kind } = measure;
        if (subject === 'unknown') {
          outcome = 'not_found';
          cat = 'E';
          section = 'unknown';
          kind = 'nosuch';
          extra = 'asked_role=item';
        } else if (
          subject === 'party' &&
          (measure.id === 'top_seller' || measure.id === 'last_rate') &&
          !(sold.party?.[period.id] ?? []).includes(thing.key)
        ) {
          // He bought none of it in the period: there is no top seller and no rate, so any safe ending.
          outcome = 'safe';
          extra = 'zero=1';
        } else if (subject === 'zero' && (measure.sold || measure.id === 'print')) {
          // No sale, so no rate, no buyer and no rank: the honest answer is any safe ending; a print is still a print.
          if (measure.id !== 'print') {
            outcome = 'safe';
          }
          if (measure.id === 'print') {
            const e = expectation(measure, subject, ctx);
            figures = e?.figures ?? '';
          }
        } else {
          const e = expectation(measure, subject, ctx);
          if (!e) throw new Error(`no expectation for ${measure.id}`);
          figures = e.figures;
          outcome = e.outcome ?? outcome;
          cat = e.cat ?? cat;
          extra = e.extra ?? '';
        }
        if (extra) filters.push(extra);
        if (!['unknown'].includes(subject) && thing?.en && unit !== 'all' && !UNITS[unit]) {
          filters.push(`thing=${thing.en}`);
        }
        const entry = {
          g: id,
          cat,
          section,
          kind,
          world: 'year',
          outcome,
          tools: '',
          filters: filters.join(';'),
          figures,
          source: 'capability-items.tsv',
          q: [text],
        };
        cells.push({
          ...base,
          id,
          status: 'A',
          owner: '',
          note: '',
          entry,
          lang: text[0],
          thing: thing?.en ?? '',
        });
      });
    }
  });
  return cells;
}

/**
 * The quality questions of plan 58 (PRD FR-12.42), each in three languages (Roman Urdu, Urdu, English) as the lead
 * asked: "awwal seb kal kitne ka bika", "doyam kele ka aaj ka rate". The figures are the oracle's `ixq.*` facts: the
 * sale lines of one item and one quality, by SQL on the books.
 */
export const QUALITY_GROUPS = [
  {
    key: 'sale',
    measure: 'gross',
    thing: 'Apple',
    quality: 'first',
    figures: 'sale=@ixq.yesterday.apple.first.gross',
    q: [
      ['roman', 'awwal seb kal kitne ka bika'],
      ['urdu', 'اول سیب کل کتنے کا بکا'],
      ['en', 'how much first quality apple was sold yesterday'],
    ],
  },
  {
    key: 'rate',
    measure: 'last_rate',
    thing: 'Banana',
    quality: 'second',
    figures: 'rate=@ixq.today.banana.second.last_rate|@ixq.today.banana.second.avg_rate',
    q: [
      ['roman', 'doyam kele ka aaj ka rate'],
      ['urdu', 'دوم کیلے کا آج کا ریٹ'],
      ['en', "today's rate of second quality banana"],
    ],
  },
  {
    key: 'qty',
    measure: 'qty',
    thing: 'Potato',
    quality: 'third',
    figures: 'qty=@ixq.week.potato.third.qty',
    q: [
      ['roman', 'is hafte soyam aloo ke kitne nag bike'],
      ['urdu', 'اس ہفتے سوم آلو کے کتنے نگ بکے'],
      ['en', 'how many nag of third quality potato were sold this week'],
    ],
  },
  {
    key: 'each',
    measure: 'gross',
    thing: 'Apple',
    quality: 'all',
    figures:
      'first=@ixq.yesterday.apple.first.gross;second=@ixq.yesterday.apple.second.gross;third=@ixq.yesterday.apple.third.gross',
    q: [
      ['roman', 'kal seb awwal doyam soyam alag alag kitne ke bike'],
      ['urdu', 'کل سیب اول دوم سوم الگ الگ کتنے کے بکے'],
      ['en', 'yesterday how much apple was sold in each quality'],
    ],
  },
  {
    key: 'higher',
    measure: 'avg_rate',
    thing: 'Banana',
    quality: 'first',
    figures: 'first=@ixq.today.banana.first.avg_rate;second=@ixq.today.banana.second.avg_rate',
    q: [
      ['roman', 'aaj kele ka awwal rate doyam se kitna zyada hai'],
      ['urdu', 'آج کیلے کا اول ریٹ دوم سے کتنا زیادہ ہے'],
      ['en', 'how much higher is first quality banana than second quality today'],
    ],
  },
].map((x) => ({
  g: `IQ-${x.key}`,
  cat: 'A',
  section: 'item',
  kind: 'item',
  world: 'year',
  outcome: 'answered',
  tools: '',
  filters: `facet=ITEM;measure=${x.measure};subject=quality;quality=${x.quality};thing=${x.thing}`,
  figures: x.figures,
  source: 'plan 58 T4: the quality of a sale line',
  q: x.q,
}));

/** The bank entries of the answerable cells. */
export function itemEntries() {
  return [
    ...buildCells()
      .filter((c) => c.status === 'A')
      .map((c) => c.entry),
    ...QUALITY_GROUPS,
  ];
}

export { capital };
