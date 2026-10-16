// Canned print data for the three documents of the PRD worked day (plan 14, PRD 4.4 and 4.5), in the
// exact shape of the generated binding types (`PrintData`). They stand in for the backend's answers
// until its real files are used (T4): the numbers are the PRD's, and the templates only show them.
// The development page /dev/print uses them too, so it must not import Vitest.

import type { Language } from '../../src/bindings/Language';
import type { Paper } from '../../src/bindings/Paper';
import type { PrintAmount } from '../../src/bindings/PrintAmount';
import type { PrintData } from '../../src/bindings/PrintData';
import type { PrintDoc } from '../../src/bindings/PrintDoc';
import type { PrintFact } from '../../src/bindings/PrintFact';
import type { PrintLine } from '../../src/bindings/PrintLine';
import type { PrintPage } from '../../src/bindings/PrintPage';

export interface CannedOptions {
  language?: Language;
  paper?: Paper;
  /** Urdu digits on an Urdu print: every digit of every value is the Urdu one. */
  urduDigits?: boolean;
  cancelled?: boolean;
}

/** `0` to `9` as the Urdu digits `۰` to `۹` (what the backend's number formatter does). */
export function toUrduDigits(text: string): string {
  return text.replace(/\d/g, (digit) => String.fromCodePoint(0x06f0 + Number(digit)));
}

interface Words {
  firm: string;
  mandi: string;
  rahim: string;
  akram: string;
  arshad: string;
  shahid: string;
  banana: string;
  apple: string;
  box: string;
}

const NAMES: Record<Language, Words> = {
  ur: {
    firm: 'ماہر عدنان اینڈ برادرز',
    mandi: 'نیو سبزی منڈی، کوٹ رادھا کشن',
    rahim: 'رحیم',
    akram: 'اکرم ٹول ٹیکس',
    arshad: 'ارشد ملتانی',
    shahid: 'شاہد',
    banana: 'کیلا',
    apple: 'سیب',
    box: 'پیٹی',
  },
  en: {
    firm: 'Maher Adnan and Brothers',
    mandi: 'New Sabzi Mandi, Kot Radha Kishan',
    rahim: 'Rahim',
    akram: 'Akram Toll Tax',
    arshad: 'Arshad Multani',
    shahid: 'Shahid',
    banana: 'Banana',
    apple: 'Apple',
    box: 'Box',
  },
};

const WORDS: Record<Language, Record<string, string>> = {
  ur: {
    '19316': 'انیس ہزار تین سو سولہ روپے',
    '12880': 'بارہ ہزار آٹھ سو اسی روپے',
    '18684': 'اٹھارہ ہزار چھ سو چوراسی روپے',
    '12800': 'بارہ ہزار آٹھ سو روپے',
  },
  en: {
    '19316': 'Nineteen thousand three hundred sixteen rupees only',
    '12880': 'Twelve thousand eight hundred eighty rupees only',
    '18684': 'Eighteen thousand six hundred eighty-four rupees only',
    '12800': 'Twelve thousand eight hundred rupees only',
  },
};

function amount(label_id: string, value: string, role: PrintAmount['role'] = 'line'): PrintAmount {
  return { label_id, amount: value, role };
}

function sellerPage(language: Language): PrintPage {
  const n = NAMES[language];
  const line = (name: string, qty: string, rate: string, total: string): PrintLine => ({
    date: '10-10-2026',
    name,
    item: n.banana,
    packing: n.box,
    qty,
    weight_kg: null,
    rate,
    amount: total,
    note: null,
  });
  return {
    party: { role_id: 'print-party-seller', code: '030110001', name: n.rahim, phone: null },
    lines: [line(n.akram, '20', '600', '12,000'), line(n.arshad, '30', '580', '17,400')],
    amounts: [
      amount('print-gross-sale', '29,400', 'subtotal'),
      amount('print-freight', '3,000'),
      amount('print-commission', '1,764'),
      amount('print-labour', '250'),
      amount('print-munshiana', '50'),
      amount('print-cash-paid', '5,000'),
      amount('print-store-rent', '0'),
      amount('print-phone-charge', '20'),
      amount('print-total-deductions', '10,084', 'subtotal'),
      amount('print-net-sale', '19,316', 'total'),
    ],
    payment_id: null,
    words: { label_id: 'print-in-words', text: WORDS[language]['19316'] ?? '' },
  };
}

interface SlipSpec {
  role: string;
  code: string | null;
  name: string;
  phone: string | null;
  item: string;
  qty: string;
  rate: string;
  value: string;
  comm: string;
  laga: string;
  total: string;
  totalKey: string;
  payment: string;
}

function slipPage(language: Language, spec: SlipSpec): PrintPage {
  const n = NAMES[language];
  return {
    party: { role_id: spec.role, code: spec.code, name: spec.name, phone: spec.phone },
    lines: [
      {
        date: '10-10-2026',
        name: null,
        item: spec.item,
        packing: n.box,
        qty: spec.qty,
        weight_kg: null,
        rate: spec.rate,
        amount: spec.value,
        note: null,
      },
    ],
    amounts: [
      amount('print-items-value', spec.value, 'subtotal'),
      amount('print-buyer-commission', spec.comm),
      amount('print-laga', spec.laga),
      amount('print-buyer-pays', spec.total, 'total'),
    ],
    payment_id: spec.payment,
    words: { label_id: 'print-in-words', text: WORDS[language][spec.totalKey] ?? '' },
  };
}

function slipPages(language: Language): PrintPage[] {
  const n = NAMES[language];
  return [
    slipPage(language, {
      role: 'print-party-buyer',
      code: '03040001',
      name: n.akram,
      phone: null,
      item: n.banana,
      qty: '20',
      rate: '600',
      value: '12,000',
      comm: '720',
      laga: '160',
      total: '12,880',
      totalKey: '12880',
      payment: 'print-payment-credit',
    }),
    slipPage(language, {
      role: 'print-party-buyer',
      code: '03040002',
      name: n.arshad,
      phone: null,
      item: n.banana,
      qty: '30',
      rate: '580',
      value: '17,400',
      comm: '1,044',
      laga: '240',
      total: '18,684',
      totalKey: '18684',
      payment: 'print-payment-credit',
    }),
  ];
}

function pukhtaPages(language: Language): PrintPage[] {
  const n = NAMES[language];
  return [
    slipPage(language, {
      role: 'print-party-onetime-buyer',
      code: null,
      name: n.shahid,
      phone: '03000000000',
      item: n.apple,
      qty: '10',
      rate: '1,200',
      value: '12,000',
      comm: '720',
      laga: '80',
      total: '12,800',
      totalKey: '12800',
      payment: 'print-payment-cash',
    }),
  ];
}

const TITLES: Record<PrintDoc, string> = {
  seller_bill: 'print-seller-bill-title',
  buyer_slips: 'print-buyer-slip-title',
  pukhta_slip: 'print-pukhta-slip-title',
};

/** The paper each document has before the Owner changes it (PRD A21). */
export const DEFAULT_PAPER: Record<PrintDoc, Paper> = {
  seller_bill: 'a4',
  buyer_slips: 'a5',
  pukhta_slip: 'a5',
};

const FACTS: PrintFact[] = [
  { label_id: 'print-fact-number', value: '1' },
  { label_id: 'print-fact-date', value: '10-10-2026' },
  { label_id: 'print-fact-truck', value: 'LES-1234' },
];

/** بکری 2 (PRD 4.5): the one-time buyer's sale, from the truck LHR-0077. */
const FACTS_BIKRI_2: PrintFact[] = [
  { label_id: 'print-fact-number', value: '2' },
  { label_id: 'print-fact-date', value: '10-10-2026' },
  { label_id: 'print-fact-truck', value: 'LHR-0077' },
];

/** The print data of one document of بکری 1 (or of بکری 2 for the پختہ slip). */
export function printDataFor(doc: PrintDoc, options: CannedOptions = {}): PrintData {
  const language = options.language ?? 'ur';
  const n = NAMES[language];
  const pages =
    doc === 'seller_bill'
      ? [sellerPage(language)]
      : doc === 'buyer_slips'
        ? slipPages(language)
        : pukhtaPages(language);
  const data: PrintData = {
    doc,
    language,
    dir: language === 'ur' ? 'rtl' : 'ltr',
    paper: options.paper ?? DEFAULT_PAPER[doc],
    urdu_digits: options.urduDigits === true,
    title_id: TITLES[doc],
    firm: {
      name: n.firm,
      mandi: n.mandi,
      address: 'Kot Radha Kishan',
      phone: '0300-1112233',
    },
    number: doc === 'pukhta_slip' ? '2' : '1',
    facts: doc === 'pukhta_slip' ? FACTS_BIKRI_2 : FACTS,
    printed_at: '10-10-2026 17:40',
    cancelled: options.cancelled === true,
    pages,
  };
  return options.urduDigits === true ? withUrduDigits(data) : data;
}

/** Every number of a print in Urdu digits, the way the backend writes them (NFR-URD-14). */
function withUrduDigits(data: PrintData): PrintData {
  const d = toUrduDigits;
  return {
    ...data,
    number: d(data.number),
    printed_at: d(data.printed_at),
    facts: data.facts.map((fact) => ({ ...fact, value: d(fact.value) })),
    pages: data.pages.map((page) => ({
      ...page,
      party: {
        ...page.party,
        code: page.party.code === null ? null : d(page.party.code),
        phone: page.party.phone === null ? null : d(page.party.phone),
      },
      lines: page.lines.map((line) => ({
        ...line,
        date: d(line.date),
        qty: d(line.qty),
        rate: d(line.rate),
        amount: d(line.amount),
      })),
      amounts: page.amounts.map((item) => ({ ...item, amount: d(item.amount) })),
    })),
  };
}

/**
 * A long document for the preview speed check (NFR-SPD-14): `buyers` slips, each with `lines`
 * lines of goods, so the layout has about that many pages. The numbers are filler; the check is
 * how fast the pages are drawn.
 */
export function longPrintData(
  buyers: number,
  lines: number,
  options: CannedOptions = {},
): PrintData {
  const base = printDataFor('buyer_slips', options);
  const template = base.pages[0];
  if (template === undefined) throw new Error('the canned slip has no page');
  const line = template.lines[0];
  if (line === undefined) throw new Error('the canned slip has no line');
  const pages = Array.from({ length: buyers }, (_, index) => ({
    ...template,
    party: { ...template.party, name: `${template.party.name} ${String(index + 1)}` },
    lines: Array.from({ length: lines }, () => ({ ...line })),
  }));
  return { ...base, pages };
}

/** One document whose first page has `lines` lines of goods, to see a table run over many sheets. */
export function manyLinesData(
  doc: PrintDoc,
  lines: number,
  options: CannedOptions = {},
): PrintData {
  const base = printDataFor(doc, options);
  const first = base.pages[0];
  const line = first?.lines[0];
  if (first === undefined || line === undefined) throw new Error('the canned document has no page');
  return {
    ...base,
    pages: [
      { ...first, lines: Array.from({ length: lines }, () => ({ ...line })) },
      ...base.pages.slice(1),
    ],
  };
}
