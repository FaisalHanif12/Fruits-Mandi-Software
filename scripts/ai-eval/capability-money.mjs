// The PERIOD AXIS of the money and party subjects (lead, 06-11-2026): receipts (R23), payments (R25), the cash book
// (R12), the shop and personal expenses by head (R29), the balance lists (R20, R24, R26, R32), one buyer, one seller,
// a collector's buyers, the partners (R30), capital, gardens (R36), trips (R33) and fruit taken (R37), each crossed
// with the 14 periods of the item axis and with four kinds of question:
//
//   F  figure   "pichle hafte ki wasooli", "akram ne is mahine kitna diya", "bijli ka kharcha March mein"
//   L  list     how many entries
//   C  compare  against the period before
//   T  top      the biggest party, head or partner
//
// Same rules as the item axis (capability-items.mjs): every cell ends as A (one question, one wording, the language
// rotating Roman Urdu, Urdu, English and two kinds of mixed wording, a quarter misspelled), X (no meaning, or nothing
// in the books for it), N (not by design, needs approval) or G (a gap, with an owner). The facts are the oracle's
// `mx.*` facts (oracle.rs `money_axis_facts`: the reports and views of the app), never the assistant's tools. A
// balance list (R20, R24, R26, R32) has no range: it is asked "as of" the last day of the period.

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { PERIODS } from './capability-items.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const KEYS = new Set(
  existsSync(join(HERE, 'oracle-keys.txt'))
    ? readFileSync(join(HERE, 'oracle-keys.txt'), 'utf8').split('\n').filter(Boolean)
    : [],
);

// ---- the people and things --------------------------------------------------------------------------------

const AKRAM = {
  key: 'akram_toll_tax',
  r: 'Akram Toll Tax',
  u: 'اکرم ٹول ٹیکس',
  e: 'Akram Toll Tax',
  x: { r: 'Akrm Toll Tax', u: 'اکرام ٹول ٹیکس', e: 'Akram Tol Tax' },
};
const FAISAL = {
  key: 'faisal',
  r: 'Faisal',
  u: 'فیصل',
  e: 'Faisal',
  x: { r: 'Faisl', u: 'فیسل', e: 'Faysal' },
};
const RAHIM = {
  key: 'rahim',
  r: 'Rahim',
  u: 'رحیم',
  e: 'Rahim',
  x: { r: 'Raheem', u: 'رحیمم', e: 'Raheem' },
};
const KARIM = {
  key: 'karim',
  r: 'Karim',
  u: 'کریم',
  e: 'Karim',
  x: { r: 'Kareem', u: 'کرم', e: 'Kareem' },
};
const RASHEED = {
  key: 'rasheed',
  r: 'Rasheed',
  u: 'رشید',
  e: 'Rasheed',
  x: { r: 'Rasheeed', u: 'رشیدد', e: 'Rasheeed' },
};
const PARTNER_A = {
  key: 'a',
  r: 'Alif',
  u: 'الف',
  e: 'Partner A',
  x: { r: 'Alf', u: 'الفف', e: 'Partner-A' },
};
const PARTNER_B = {
  key: 'b',
  r: 'Be',
  u: 'ب',
  e: 'Partner B',
  x: { r: 'Bay', u: 'بب', e: 'Partner-B' },
};
const HEADS = [
  {
    key: 'electricity',
    r: 'bijli',
    u: 'بجلی',
    e: 'electricity',
    x: { r: 'bijly', u: 'بجلے', e: 'electrisity' },
  },
  {
    key: 'shop_rent',
    r: 'dukan ka kiraya',
    u: 'دکان کا کرایہ',
    e: 'shop rent',
    x: { r: 'dukan ka kiraaya', u: 'دکان کا کراِیہ', e: 'shop rant' },
  },
  { key: 'tea', r: 'chai', u: 'چائے', e: 'tea', x: { r: 'chaye', u: 'چاۓ', e: 'te' } },
  {
    key: 'salaries',
    r: 'tankhwah',
    u: 'تنخواہ',
    e: 'salaries',
    x: { r: 'tankhah', u: 'تنخوا', e: 'salarys' },
  },
];

/** A noun with its misspelled forms (a subject that has no person in it). */
const noun = (r, u, e, xr, xu, xe) => ({ r, u, e, x: { r: xr, u: xu, e: xe } });

// ---- the subjects ----------------------------------------------------------------------------------------------
// types: F figure, L list, C compare, T top. A type holds the fact key(s) for a period `p`, the figure label and the
// wordings (r, u, e; mr and mu are the mixed ones, taken from the English noun when `{w}` is the noun). `{w}` is the
// thing (the noun, the person or the head), `{p}` the period, `{v}` the period before.

export const MONEY_SUBJECTS = [
  // ---- registers of the period ----
  {
    id: 'receipts',
    section: 'cash',
    thing: [noun('wasooli', 'وصولی', 'receipts', 'wasuli', 'وصولئ', 'reciepts')],
    F: {
      key: (p) => `mx.${p}.r23.total.amount`,
      label: 'amt',
      r: '{p} {w} kitni hui',
      u: '{p} {w} کتنی ہوئی',
      e: 'how much were the {w} {p}',
    },
    L: {
      key: (p) => `mx.${p}.r23.count`,
      label: 'count',
      r: '{p} {w} ki kitni entries darj hui',
      u: '{p} {w} کی کتنی اندراجات درج ہوئیں',
      e: 'how many entries of {w} were made {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.receipts`,
      label: 'diff',
      r: '{p} {w} {v} se kitni zyada ya kam rahi',
      u: '{p} {w} {v} سے کتنی زیادہ یا کم رہی',
      e: 'how do the {w} {p} compare with {v}',
    },
    T: {
      key: (p) => `mx.${p}.r23.top_party`,
      label: 'top!',
      r: '{p} sab se bari {w} kis se aayi',
      u: '{p} سب سے بڑی {w} کس سے آئی',
      e: 'who paid the biggest of the {w} {p}',
    },
  },
  {
    id: 'payments',
    section: 'cash',
    thing: [noun('adaigi', 'ادائیگی', 'payments', 'adaiggi', 'ادائیگئ', 'paymnts')],
    F: {
      key: (p) => `mx.${p}.r25.total.amount`,
      label: 'amt',
      r: '{p} {w} kitni hui',
      u: '{p} {w} کتنی ہوئی',
      e: 'how much were the {w} {p}',
    },
    L: {
      key: (p) => `mx.${p}.r25.count`,
      label: 'count',
      r: '{p} {w} ki kitni entries darj hui',
      u: '{p} {w} کی کتنی اندراجات درج ہوئیں',
      e: 'how many entries of {w} were made {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.payments`,
      label: 'diff',
      r: '{p} {w} {v} se kitni zyada ya kam rahi',
      u: '{p} {w} {v} سے کتنی زیادہ یا کم رہی',
      e: 'how do the {w} {p} compare with {v}',
    },
    T: {
      key: (p) => `mx.${p}.r25.top_party`,
      label: 'top!',
      r: '{p} sab se bari {w} kis ko hui',
      u: '{p} سب سے بڑی {w} کس کو ہوئی',
      e: 'who got the biggest of the {w} {p}',
    },
  },
  {
    id: 'cash_in',
    section: 'cash',
    thing: [
      noun(
        'roakar ki aamad',
        'روکڑ کی آمد',
        'cash in',
        'rokar ki amad',
        'روکڑ کی آمدد',
        'cash inn',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r12.total.cash_in`,
      label: 'amt',
      r: '{p} roakar mein kitni raqam aayi',
      u: '{p} روکڑ میں کتنی رقم آئی',
      e: 'how much cash came in {p}',
      mr: '{p} roakar mein kitna cash aaya',
      mu: '{p} روکڑ میں کتنا cash آیا',
    },
    C: {
      key: (p) => `mx.${p}.vs.cash_in`,
      label: 'diff',
      r: '{p} roakar ki aamad {v} se kitni zyada ya kam rahi',
      u: '{p} روکڑ کی آمد {v} سے کتنی زیادہ یا کم رہی',
      e: 'cash in {p} against {v}',
    },
  },
  {
    id: 'cash_out',
    section: 'cash',
    thing: [
      noun(
        'roakar se nikli raqam',
        'روکڑ سے نکلی رقم',
        'cash out',
        'rokar se nikli rakam',
        'روکڑ سے نکلی رکم',
        'cash outt',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r12.total.cash_out`,
      label: 'amt',
      r: '{p} roakar se kitni raqam gayi',
      u: '{p} روکڑ سے کتنی رقم گئی',
      e: 'how much cash went out {p}',
      mr: '{p} roakar se kitna cash gaya',
      mu: '{p} روکڑ سے کتنا cash گیا',
    },
    C: {
      key: (p) => `mx.${p}.vs.cash_out`,
      label: 'diff',
      r: '{p} roakar se nikli raqam {v} se kitni zyada ya kam rahi',
      u: '{p} روکڑ سے نکلی رقم {v} سے کتنی زیادہ یا کم رہی',
      e: 'cash out {p} against {v}',
    },
  },
  {
    id: 'cash_close',
    section: 'cash',
    thing: [noun('roakar', 'روکڑ', 'cash in hand', 'rokar', 'روکڑڑ', 'cash in hnd')],
    F: {
      key: (p) => `mx.${p}.r12.sum.report_closing_balance`,
      label: 'amt',
      r: '{p} ke aakhir mein {w} kitni thi',
      u: '{p} کے آخر میں {w} کتنی تھی',
      e: 'what was the {w} at the end of {p}',
      mr: '{p} ke aakhir mein {w} kitna tha',
      mu: '{p} کے آخر میں {w} کتنا تھا',
    },
  },
  // ---- expenses ----
  {
    id: 'expense',
    section: 'expense',
    thing: [
      noun(
        'dukan ka kharcha',
        'دکان کا خرچہ',
        'shop expense',
        'dukan ka kharch',
        'دکان کا خرچ',
        'shop expence',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r29.total.expense`,
      label: 'amt',
      r: '{p} {w} kitna hua',
      u: '{p} {w} کتنا ہوا',
      e: 'how much was the {w} {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.expense`,
      label: 'diff',
      r: '{p} {w} {v} se kitna zyada ya kam raha',
      u: '{p} {w} {v} سے کتنا زیادہ یا کم رہا',
      e: 'the {w} {p} against {v}',
    },
    T: {
      key: (p) => `mx.${p}.r29.top_head`,
      label: 'top!',
      r: '{p} {w} sab se zyada kis cheez par hua',
      u: '{p} {w} سب سے زیادہ کس چیز پر ہوا',
      e: 'which head had the biggest {w} {p}',
    },
    L: ['N', 'R29 counts the heads and the partners together in one count: not one figure'],
  },
  {
    id: 'expense_head',
    section: 'expense',
    thing: HEADS,
    headKey: true,
    F: {
      key: (p, w) => `mx.${p}.r29.${w.key}.expense`,
      label: 'amt',
      r: '{w} ka kharcha {p} kitna hua',
      u: '{w} کا خرچہ {p} کتنا ہوا',
      e: 'how much was the {w} expense {p}',
    },
    C: {
      key: (p, w) => `mx.${p}.vs.${w.key}`,
      label: 'diff',
      r: '{w} ka kharcha {p} {v} se kitna zyada ya kam raha',
      u: '{w} کا خرچہ {p} {v} سے کتنا زیادہ یا کم رہا',
      e: 'the {w} expense {p} against {v}',
    },
    L: ['X', 'a head is one row'],
    T: ['X', 'a head is one row'],
  },
  {
    id: 'personal_taken',
    section: 'expense',
    thing: [
      noun(
        'zaati kharcha',
        'ذاتی خرچہ',
        'personal drawings',
        'zati kharcha',
        'ذاتی خرچ',
        'personal drawing',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r29.total.taken`,
      label: 'amt',
      r: '{p} shirakat daaron ne {w} ke liye kitna liya',
      u: '{p} شراکت داروں نے {w} کے لیے کتنا لیا',
      e: 'how much did the partners take for {w} {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.taken`,
      label: 'diff',
      r: '{p} {w} {v} se kitna zyada ya kam raha',
      u: '{p} {w} {v} سے کتنا زیادہ یا کم رہا',
      e: 'the {w} {p} against {v}',
    },
  },
  {
    id: 'personal_returned',
    section: 'expense',
    thing: [noun('wapsi', 'واپسی', 'returned', 'wapsee', 'واپسے', 'retuned')],
    F: {
      key: (p) => `mx.${p}.r29.total.returned`,
      label: 'amt',
      r: '{p} shirakat daaron ne zaati kharche ki kitni raqam wapas ki',
      u: '{p} شراکت داروں نے ذاتی خرچے کی کتنی رقم واپس کی',
      e: 'how much of the personal drawings did the partners give back {p}',
    },
  },
  // ---- balance lists as of the last day of the period (no range, FR-19.10) ----
  ...[
    [
      'advances',
      'dues',
      'r26',
      'total.open',
      'peshgi',
      'پیشگی',
      'advances',
      'peshgee',
      'پیشگے',
      'advanses',
      'open',
    ],
    [
      'money_lent',
      'dues',
      'r32',
      'total.open',
      'udhar raqam',
      'ادھار رقم',
      'money lent',
      'udhaar raqam',
      'ادہار رقم',
      'money lnt',
      'open',
    ],
    [
      'dues',
      'dues',
      'r20',
      'total.balance',
      'gahakon ka bakaya',
      'گاہکوں کا بقایا',
      'buyers dues',
      'gahkon ka bakaya',
      'گاہکوں کا باقایا',
      'buyers dus',
      'balance',
    ],
    [
      'payables',
      'dues',
      'r24',
      'total.payable',
      'beparion ki raqam',
      'بیوپاریوں کی رقم',
      'sellers payable',
      'bepariyon ki rakam',
      'بیوپاریوں کی رقم',
      'sellers payble',
      'payable',
    ],
  ].map(([id, section, report, total, r, u, e, xr, xu, xe]) => ({
    id,
    section,
    asof: true,
    thing: [noun(r, u, e, xr, xu, xe)],
    F: {
      key: (p) => `mx.asof.${p}.${report}.${total}`,
      label: 'amt',
      r: '{p} {w} kitna baqi tha',
      u: '{p} {w} کتنا باقی تھا',
      e: 'how much were the {w} {p}',
      mr: '{p} {w} ka baqi total kitna tha',
      mu: '{p} {w} کا باقی total کتنا تھا',
    },
    L: {
      key: (p) => `mx.asof.${p}.${report}.count`,
      label: 'count',
      r: '{p} {w} mein kitne khaate the',
      u: '{p} {w} میں کتنے کھاتے تھے',
      e: 'how many accounts were in the {w} {p}',
    },
    T: {
      key: (p) => `mx.asof.${p}.${report}.top_party`,
      label: 'top!',
      r: '{p} {w} sab se zyada kis ki thi',
      u: '{p} {w} سب سے زیادہ کس کی تھی',
      e: 'who had the biggest of the {w} {p}',
    },
    C: ['N', 'a balance list has no second period; two as-of days are two reads'],
  })),
  // ---- one buyer, one seller, one collector ----
  {
    id: 'buyer_paid',
    section: 'buyer',
    thing: [AKRAM, FAISAL],
    F: {
      key: (p, w) => `mx.${p}.r23.${w.key}.total.amount`,
      label: 'amt',
      r: '{w} ne {p} kitna diya',
      u: '{w} نے {p} کتنا دیا',
      e: 'how much did {w} pay {p}',
      mr: '{w} se {p} kitni receipt wasool hui',
      mu: '{w} سے {p} کتنی receipt وصول ہوئی',
    },
    L: {
      key: (p, w) => `mx.${p}.r23.${w.key}.count`,
      label: 'count',
      r: '{w} ne {p} kitni baar paise diye',
      u: '{w} نے {p} کتنی بار پیسے دیے',
      e: 'how many payments did {w} make {p}',
    },
    C: {
      key: (p, w) => `mx.${p}.vs.paid_${w.key}`,
      label: 'diff',
      r: '{w} se {p} ki wasooli {v} se kitni zyada ya kam rahi',
      u: '{w} سے {p} کی وصولی {v} سے کتنی زیادہ یا کم رہی',
      e: 'payments received from {w} {p} against {v}',
    },
    T: ['X', 'a top of one buyer is his top item (the item axis)'],
  },
  {
    id: 'buyer_bought',
    section: 'buyer',
    thing: [AKRAM, FAISAL],
    F: {
      key: (p, w) => [
        `mx.${p}.c.${w.key}.total.purchase_value`,
        `mx.${p}.c.${w.key}.bought_with_charges`,
      ],
      label: 'amt',
      r: '{w} ne {p} kitna maal khareeda',
      u: '{w} نے {p} کتنا مال خریدا',
      e: 'how much did {w} buy {p}',
      mr: '{w} ne {p} kitni purchase ki',
      mu: '{w} نے {p} کتنی purchase کی',
    },
    C: {
      key: (p, w) => `mx.${p}.vs.bought_${w.key}`,
      label: 'diff',
      r: '{w} ki khareedari {p} {v} se kitni zyada ya kam rahi',
      u: '{w} کی خریداری {p} {v} سے کتنی زیادہ یا کم رہی',
      e: "{w}'s purchases {p} against {v}",
    },
    L: ['X', 'the lines of one buyer are the item axis'],
    T: ['X', 'a top of one buyer is his top item (the item axis)'],
  },
  {
    id: 'seller_paid',
    section: 'seller',
    thing: [RAHIM, KARIM],
    F: {
      key: (p, w) => `mx.${p}.r25.${w.key}.total.amount`,
      label: 'amt',
      r: '{w} ko {p} kitni adaigi ki',
      u: '{w} کو {p} کتنی ادائیگی کی',
      e: 'how much did we pay {w} {p}',
      mr: '{w} ko {p} kitna payment kiya',
      mu: '{w} کو {p} کتنا payment کیا',
    },
    L: {
      key: (p, w) => `mx.${p}.r25.${w.key}.count`,
      label: 'count',
      r: '{w} ko {p} kitni baar adaigi ki',
      u: '{w} کو {p} کتنی بار ادائیگی کی',
      e: 'how many payments did we make to {w} {p}',
    },
    C: {
      key: (p, w) => `mx.${p}.vs.paid_to_${w.key}`,
      label: 'diff',
      r: '{w} ko adaigi {p} {v} se kitni zyada ya kam rahi',
      u: '{w} کو ادائیگی {p} {v} سے کتنی زیادہ یا کم رہی',
      e: 'payments to {w} {p} against {v}',
    },
    T: ['X', 'a top of one seller is his top item (the item axis)'],
  },
  {
    id: 'seller_sold',
    section: 'seller',
    thing: [RAHIM, KARIM],
    F: {
      key: (p, w) => `mx.${p}.b.${w.key}.total.gross`,
      label: 'amt',
      r: '{w} ka {p} kitna maal bika',
      u: '{w} کا {p} کتنا مال بکا',
      e: 'how much of the goods of {w} sold {p}',
      mr: '{w} ki {p} kitni sale hui',
      mu: '{w} کی {p} کتنی sale ہوئی',
    },
    C: {
      key: (p, w) => `mx.${p}.vs.sold_${w.key}`,
      label: 'diff',
      r: '{w} ki bikri {p} {v} se kitni zyada ya kam rahi',
      u: '{w} کی بکری {p} {v} سے کتنی زیادہ یا کم رہی',
      e: 'the sales of {w} {p} against {v}',
    },
    L: ['X', 'the lines of one seller are the item axis'],
    T: ['X', 'a top of one seller is his top item (the item axis)'],
  },
  {
    id: 'collector',
    section: 'dues',
    thing: [RASHEED],
    F: {
      key: (p, w) => `mx.${p}.r23.${w.key}.total.amount`,
      label: 'amt',
      r: '{w} ne {p} kitni ugrahi ki',
      u: '{w} نے {p} کتنی اگراہی کی',
      e: 'how much did {w} collect {p}',
      mr: '{w} ne {p} kitna collection kiya',
      mu: '{w} نے {p} کتنا collection کیا',
    },
    L: {
      key: (p, w) => `mx.${p}.r23.${w.key}.count`,
      label: 'count',
      r: '{w} ne {p} kitni wasooliyan kin',
      u: '{w} نے {p} کتنی وصولیاں کیں',
      e: 'how many receipts did {w} bring {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.collected`,
      label: 'diff',
      r: '{w} ki ugrahi {p} {v} se kitni zyada ya kam rahi',
      u: '{w} کی اگراہی {p} {v} سے کتنی زیادہ یا کم رہی',
      e: "{w}'s collection {p} against {v}",
    },
    T: ['X', 'a top of one collector is his top buyer; a collector has the receipts list'],
  },
  // ---- partners, capital, gardens, trips, fruit taken ----
  {
    id: 'partner_profit',
    section: 'partner',
    thing: [PARTNER_A, PARTNER_B],
    F: {
      key: (p, w) => `mx.${p}.r30.partner_${w.key}.profit_share`,
      label: 'amt',
      r: '{w} ka {p} ka munafa hissa kitna bana',
      u: '{w} کا {p} کا منافع حصہ کتنا بنا',
      e: "what was {w}'s profit share {p}",
      mr: '{w} ka {p} ka profit share kitna bana',
      mu: '{w} کا {p} کا profit share کتنا بنا',
    },
    C: {
      key: (p, w) => `mx.${p}.vs.profit_${w.key}`,
      label: 'diff',
      r: '{w} ka munafa hissa {p} {v} se kitna zyada ya kam raha',
      u: '{w} کا منافع حصہ {p} {v} سے کتنا زیادہ یا کم رہا',
      e: "{w}'s profit share {p} against {v}",
    },
    T: {
      key: (p) => `mx.${p}.r30.top_partner`,
      label: 'top!',
      r: '{p} sab se zyada munafa kis shirakat daar ka bana',
      u: '{p} سب سے زیادہ منافع کس شراکت دار کا بنا',
      e: 'which partner had the biggest profit share {p}',
    },
    L: ['X', 'a partner has one row'],
  },
  {
    id: 'partner_taken',
    section: 'partner',
    thing: [PARTNER_A],
    F: {
      key: (p, w) => `mx.${p}.r30.partner_${w.key}.taken`,
      label: 'amt',
      r: '{w} ne {p} zaati kharche ke liye kitna liya',
      u: '{w} نے {p} ذاتی خرچے کے لیے کتنا لیا',
      e: 'how much did {w} take for personal use {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.taken_a`,
      label: 'diff',
      r: '{w} ka liya hua {p} {v} se kitna zyada ya kam raha',
      u: '{w} کا لیا ہوا {p} {v} سے کتنا زیادہ یا کم رہا',
      e: 'what {w} took {p} against {v}',
    },
  },
  {
    id: 'capital_in',
    section: 'capital',
    thing: [noun('sarmaya', 'سرمایہ', 'capital invested', 'sarmaaya', 'سرمائہ', 'capitl invested')],
    F: {
      key: (p) => `mx.${p}.capital.in`,
      label: 'amt',
      r: '{p} shirakat daaron ne kitna {w} lagaya',
      u: '{p} شراکت داروں نے کتنا {w} لگایا',
      e: 'how much {w} did the partners put in {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.capital_in`,
      label: 'diff',
      r: '{p} lagaya hua {w} {v} se kitna zyada ya kam raha',
      u: '{p} لگایا ہوا {w} {v} سے کتنا زیادہ یا کم رہا',
      e: '{w} {p} against {v}',
    },
    L: ['X', 'capital is one figure'],
    T: ['X', 'capital is one figure'],
  },
  {
    id: 'capital_out',
    section: 'capital',
    thing: [
      noun(
        'sarmaya wapsi',
        'سرمایہ واپسی',
        'capital taken back',
        'sarmaya wapsee',
        'سرمایہ واپسے',
        'capital taken bak',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.capital.out`,
      label: 'amt',
      r: '{p} shirakat daaron ne kitna sarmaya wapas liya',
      u: '{p} شراکت داروں نے کتنا سرمایہ واپس لیا',
      e: 'how much capital did the partners take back {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.capital_out`,
      label: 'diff',
      r: '{p} {w} {v} se kitni zyada ya kam rahi',
      u: '{p} {w} {v} سے کتنی زیادہ یا کم رہی',
      e: '{w} {p} against {v}',
    },
    L: ['X', 'capital is one figure'],
    T: ['X', 'capital is one figure'],
  },
  {
    id: 'capital_b',
    section: 'capital',
    thing: [PARTNER_B],
    F: {
      key: (p) => `mx.${p}.capital.b.in`,
      label: 'amt',
      r: '{w} ne {p} kitna sarmaya lagaya',
      u: '{w} نے {p} کتنا سرمایہ لگایا',
      e: 'how much capital did {w} put in {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.capital_b_in`,
      label: 'diff',
      r: '{w} ka sarmaya {p} {v} se kitna zyada ya kam raha',
      u: '{w} کا سرمایہ {p} {v} سے کتنا زیادہ یا کم رہا',
      e: "{w}'s capital {p} against {v}",
    },
    L: ['X', 'capital is one figure'],
    T: ['X', 'capital is one figure'],
  },
  {
    id: 'gardens_money_in',
    section: 'garden',
    thing: [
      noun(
        'baghat ki aamad',
        'باغات کی آمد',
        'garden sales money',
        'baghaat ki amad',
        'باغات کی آمدد',
        'gardn sales money',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r36.total.money_in`,
      label: 'amt',
      r: '{p} baghat se kitni raqam aayi',
      u: '{p} باغات سے کتنی رقم آئی',
      e: 'how much money came in from the gardens {p}',
      mr: '{p} gardens se kitni raqam aayi',
      mu: '{p} gardens سے کتنی رقم آئی',
    },
    L: {
      key: (p) => `mx.${p}.r36.count`,
      label: 'count',
      r: '{p} kitne baghat mein lein dein hua',
      u: '{p} کتنے باغات میں لین دین ہوا',
      e: 'how many gardens had entries {p}',
    },
    C: [
      'N',
      'the gardens report has no second-period fact yet (view I is the screen for a garden comparison)',
    ],
    T: ['X', 'one garden in the books'],
  },
  {
    id: 'gardens_installments',
    section: 'garden',
    thing: [
      noun(
        'baghat ki qist',
        'باغات کی قسط',
        'garden installments',
        'baghaat ki qist',
        'باغات کی قسطط',
        'gardn installments',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r36.total.installments`,
      label: 'amt',
      r: '{p} baghat ki kitni qist di gayi',
      u: '{p} باغات کی کتنی قسط دی گئی',
      e: 'how much was paid in garden installments {p}',
      mr: '{p} gardens ki kitni installment di gayi',
      mu: '{p} gardens کی کتنی installment دی گئی',
    },
    L: ['X', 'covered by the gardens count'],
    C: ['N', 'see the gardens money in'],
    T: ['X', 'one garden in the books'],
  },
  {
    id: 'trips_bought',
    section: 'trip',
    thing: [
      noun(
        'market safar ki khareedari',
        'مارکیٹ سفر کی خریداری',
        'trip purchases',
        'market safar ki khareedaree',
        'مارکیٹ سفر کی خریداریی',
        'trip purchases',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r33.total.bought_cost`,
      label: 'amt',
      r: '{p} market safar mein kitna maal khareeda gaya',
      u: '{p} مارکیٹ سفر میں کتنا مال خریدا گیا',
      e: 'how much was bought on market trips {p}',
      mr: '{p} market trips mein kitna maal khareeda gaya',
      mu: '{p} market trips میں کتنا مال خریدا گیا',
    },
    L: {
      key: (p) => `mx.${p}.r33.count`,
      label: 'count',
      r: '{p} kitne market safar hue',
      u: '{p} کتنے مارکیٹ سفر ہوئے',
      e: 'how many market trips were there {p}',
    },
    C: [
      'N',
      'the trips report has no second-period fact yet (view J is the screen for a trip comparison)',
    ],
    T: ['X', 'two trips in the books'],
  },
  {
    id: 'trips_sales',
    section: 'trip',
    thing: [
      noun(
        'market safar ki bikri',
        'مارکیٹ سفر کی بکری',
        'trip sales money',
        'market safar ki bikree',
        'مارکیٹ سفر کی بکریی',
        'trip sales mony',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r33.total.sales_money`,
      label: 'amt',
      r: '{p} market safar ke maal ki kitni bikri hui',
      u: '{p} مارکیٹ سفر کے مال کی کتنی بکری ہوئی',
      e: 'how much did the market trip goods sell for {p}',
      mr: '{p} market trips ke maal ki kitni sale hui',
      mu: '{p} market trips کے مال کی کتنی sale ہوئی',
    },
    L: ['X', 'covered by the trips count'],
    C: ['N', 'see the trips purchases'],
    T: ['X', 'two trips in the books'],
  },
  {
    id: 'trips_result',
    section: 'trip',
    thing: [
      noun(
        'market safar ka nafa nuqsan',
        'مارکیٹ سفر کا نفع نقصان',
        'trip profit or loss',
        'market safar ka nafa nuksan',
        'مارکیٹ سفر کا نفع نقسان',
        'trip profit or los',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r33.total.result`,
      label: 'amt',
      r: '{p} market safar ka kitna nafa ya nuqsan raha',
      u: '{p} مارکیٹ سفر کا کتنا نفع یا نقصان رہا',
      e: 'what was the market trip profit or loss {p}',
      mr: '{p} market trips ka kitna profit ya loss raha',
      mu: '{p} market trips کا کتنا profit یا loss رہا',
    },
    L: ['X', 'covered by the trips count'],
    C: ['N', 'see the trips purchases'],
    T: ['X', 'two trips in the books'],
  },
  {
    id: 'fruit_value',
    section: 'fruit',
    thing: [noun('phal', 'پھل', 'fruit taken', 'fal', 'پھلل', 'frut taken')],
    F: {
      key: (p) => `mx.${p}.r37.total.value`,
      label: 'amt',
      r: '{p} {w} ki kitni raqam bani',
      u: '{p} {w} کی کتنی رقم بنی',
      e: 'what was the value of the {w} {p}',
    },
    L: {
      key: (p) => `mx.${p}.r37.count`,
      label: 'count',
      r: '{p} kitni baar {w} gaya',
      u: '{p} کتنی بار {w} گیا',
      e: 'how many times was {w} entered {p}',
    },
    C: {
      key: (p) => `mx.${p}.vs.fruit`,
      label: 'diff',
      r: '{p} {w} {v} se kitna zyada ya kam raha',
      u: '{p} {w} {v} سے کتنا زیادہ یا کم رہا',
      e: 'the {w} {p} against {v}',
    },
    T: ['X', 'the top partner of fruit taken is a row of view K, read by the model'],
  },
  {
    id: 'fruit_qty',
    section: 'fruit',
    thing: [
      noun(
        'phal ka nag',
        'پھل کا نگ',
        'fruit taken quantity',
        'fal ka nug',
        'پھل کا ننگ',
        'frut taken quantty',
      ),
    ],
    F: {
      key: (p) => `mx.${p}.r37.total.qty`,
      label: 'qty',
      r: '{p} kitne nag {w} gaya',
      u: '{p} کتنے نگ {w} گیا',
      e: 'how many nag of fruit were taken {p}',
    },
    L: ['X', 'covered by the fruit count'],
    C: ['N', 'see the fruit value'],
    T: ['X', 'see the fruit value'],
  },
];

// ---- the periods of an as-of question: the last day of the period --------------------------------------------

const ASOF = {
  today: { r: 'abhi', u: 'ابھی', e: 'now' },
  yesterday: { r: 'kal tak', u: 'کل تک', e: 'as of yesterday' },
  d2026_11_03: { r: '3 November tak', u: '3 نومبر تک', e: 'as of 3 November' },
  d2026_10_14: { r: '14 October tak', u: '14 اکتوبر تک', e: 'as of 14 October' },
  last_week: {
    r: 'pichle hafte ke aakhir tak',
    u: 'پچھلے ہفتے کے آخر تک',
    e: 'at the end of last week',
  },
  last_month: {
    r: 'pichle mahine ke aakhir tak',
    u: 'پچھلے مہینے کے آخر تک',
    e: 'at the end of last month',
  },
  m2026_09: { r: 'September ke aakhir tak', u: 'ستمبر کے آخر تک', e: 'at the end of September' },
  m2026_08: { r: 'August ke aakhir tak', u: 'اگست کے آخر تک', e: 'at the end of August' },
  last_year: {
    r: 'pichle saal ke aakhir tak',
    u: 'پچھلے سال کے آخر تک',
    e: 'at the end of last year',
  },
  r2026_10_01_2026_10_15: { r: '15 October tak', u: '15 اکتوبر تک', e: 'as of 15 October' },
};

// ---- the cells ---------------------------------------------------------------------------------------------------

const KINDS = ['r', 'u', 'e', 'mr', 'mu'];
const LANG = { r: 'roman', u: 'urdu', e: 'en', mr: 'roman', mu: 'urdu' };
const QTYPE = { F: 'figure', L: 'list', C: 'compare', T: 'top' };

const scriptOf = (kind) =>
  kind === 'r' || kind === 'mr' ? 'r' : kind === 'u' || kind === 'mu' ? 'u' : 'e';

function wordOf(thing, kind, mis) {
  const script = scriptOf(kind);
  return mis ? thing.x[script] : thing[script];
}

/** One wording: [lang, text, flag?]. A mixed kind uses the template's own mixed text, or the English noun. */
function wording(type, thing, period, subject, n, mis) {
  let kind = KINDS[n % KINDS.length];
  const mixed = kind === 'mr' || kind === 'mu';
  const own = type[kind];
  const isNoun = !thing.key;
  if (mixed && !own && !isNoun) kind = kind === 'mr' ? 'r' : 'u';
  const script = scriptOf(kind);
  let text = type[kind] ?? type[script];
  let w = wordOf(thing, kind, mis);
  if (mixed && !own && isNoun) w = mis ? thing.x.e : thing.e;
  const phrase = subject.asof ? ASOF[period.id][script] : period[script];
  const filled = text
    .replace(/\{w\}/g, w)
    .replace(/\{p\}/g, phrase)
    .replace(/\{v\}/g, period[`p${script}`] ?? '');
  return [LANG[kind], filled.replace(/\s+/g, ' ').trim(), mis ? 'mis' : undefined].filter(Boolean);
}

/** Builds every cell of the money axis: { id, subject, qtype, period, status, owner, note, entry }. */
export function moneyCells() {
  const cells = [];
  const counters = new Map();
  MONEY_SUBJECTS.forEach((subject, subjectIndex) => {
    for (const qt of ['F', 'L', 'C', 'T']) {
      const type = subject[qt];
      const periods = PERIODS.filter((p) => qt !== 'C' || p.prev);
      if (Array.isArray(type) || !type) {
        const [status, note] = Array.isArray(type)
          ? type
          : ['X', 'this question has no meaning for this subject'];
        cells.push({
          id: `MX-${subject.id}-${qt}-*`,
          subject: subject.id,
          qtype: qt,
          period: '*',
          status,
          owner: '',
          note,
        });
        continue;
      }
      periods.forEach((period, periodIndex) => {
        const id = `MX-${subject.id}-${qt}-${period.id}`;
        const base = { id, subject: subject.id, qtype: qt, period: period.id };
        if (subject.asof && !ASOF[period.id]) {
          cells.push({
            ...base,
            status: 'X',
            owner: '',
            note: 'the as-of day of this period is today: the same cell as "today"',
          });
          return;
        }
        // The thing: the noun, a person or a head. A head that has no row in the period is skipped.
        const things = subject.thing;
        const start = (periodIndex + subjectIndex) % things.length;
        let thing = null;
        let keys = null;
        for (let k = 0; k < things.length; k += 1) {
          const candidate = things[(start + k) % things.length];
          const raw = type.key(period.id, candidate);
          const list = Array.isArray(raw) ? raw : [raw];
          if (list.every((key) => KEYS.has(key))) {
            thing = candidate;
            keys = list;
            break;
          }
        }
        if (!thing) {
          const first = Array.isArray(type.key(period.id, things[0]))
            ? type.key(period.id, things[0])[0]
            : type.key(period.id, things[0]);
          // A figure that should always be there is a gap of the oracle; a top or a head with no row is nothing.
          const quiet = qt === 'T' || subject.headKey || subject.asof;
          cells.push({
            ...base,
            status: quiet ? 'X' : 'G',
            owner: quiet ? '' : 'robustness',
            note: quiet
              ? 'nothing in the books for this period (no row, so no fact)'
              : `the oracle has no fact ${first}`,
          });
          return;
        }
        const counterKey = `${subject.id}-${qt}`;
        const n = counters.get(counterKey) ?? 0;
        counters.set(counterKey, n + 1);
        const mis = n % 4 === 3;
        const text = wording(type, thing, period, subject, n + subjectIndex, mis);
        const figures = `${type.label}=${keys.map((k) => `@${k}`).join('|')}`;
        cells.push({
          ...base,
          status: 'A',
          owner: '',
          note: '',
          lang: text[0],
          thing: thing.r ?? '',
          entry: {
            g: id,
            cat: 'A',
            section: subject.section,
            kind: qt === 'F' ? 'period' : qt === 'L' ? 'list' : qt === 'C' ? 'compare' : 'top',
            world: 'year',
            outcome: 'answered',
            tools: '',
            filters: `facet=MONEY;subject=${subject.id};qtype=${QTYPE[qt]};when=${period.id}`,
            figures,
            source: 'capability-money.tsv',
            q: [text],
          },
        });
      });
    }
  });
  return cells;
}

/** The bank entries of the answerable cells. */
export function moneyEntries() {
  return moneyCells()
    .filter((c) => c.status === 'A')
    .map((c) => c.entry);
}
