// Canned print data for the three documents of the PRD worked day (plan 14, PRD 4.4 and 4.5), in the
// exact shape of the generated binding types (`PrintData`). They stand in for the backend's answers
// until its real files are used (T4): the numbers are the PRD's, and the templates only show them.
// The development page /dev/print uses them too, so it must not import Vitest.

import type { CollectionPrintData } from '../../src/bindings/CollectionPrintData';
import type { CollectionPrintRow } from '../../src/bindings/CollectionPrintRow';
import type { Language } from '../../src/bindings/Language';
import type { Paper } from '../../src/bindings/Paper';
import type { PrintAmount } from '../../src/bindings/PrintAmount';
import type { PrintContact } from '../../src/bindings/PrintContact';
import type { PrintData } from '../../src/bindings/PrintData';
import type { PrintDoc } from '../../src/bindings/PrintDoc';
import type { PrintFact } from '../../src/bindings/PrintFact';
import type { PrintFooter } from '../../src/bindings/PrintFooter';
import type { PrintHeader } from '../../src/bindings/PrintHeader';
import type { PrintLine } from '../../src/bindings/PrintLine';
import type { PrintPage } from '../../src/bindings/PrintPage';

export interface CannedOptions {
  language?: Language;
  paper?: Paper;
  /** Urdu digits on an Urdu print: every digit of every value is the Urdu one. */
  urduDigits?: boolean;
  cancelled?: boolean;
  /** The firm's contact persons (up to three) and the trade mark are set in Settings (PRD 21). */
  contacts?: boolean;
  tradeMark?: string | null;
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

/** The firm's three contact persons of the PRD (Section 21), in the print language. */
function contactsOf(language: Language): PrintContact[] {
  return language === 'ur'
    ? [
        { name: 'منشی آصف بھٹی', phone: '0300-4058860' },
        { name: 'ماہر عدنان', phone: '0300-1112233' },
        { name: 'محمد اکرم', phone: '0321-5556677' },
      ]
    : [
        { name: 'Munshi Asif Bhatti', phone: '0300-4058860' },
        { name: 'Maher Adnan', phone: '0300-1112233' },
        { name: 'Muhammad Akram', phone: '0321-5556677' },
      ];
}

/**
 * The shared header and footer of a print, in the exact shape of the generated types
 * (PRD FR-19.13): no contact and no trade mark unless `options` ask for them, because a contact or
 * a trade mark that is not set is left out and has no box.
 */
export function frameOf(
  language: Language,
  titleId: string,
  facts: PrintFact[],
  options: CannedOptions = {},
): { header: PrintHeader; footer: PrintFooter } {
  const n = NAMES[language];
  return {
    header: {
      firm: {
        name: n.firm,
        mandi: n.mandi,
        address: 'Kot Radha Kishan',
        phone: '0300-1112233',
      },
      contacts: options.contacts === true ? contactsOf(language) : [],
      trade_mark: options.tradeMark ?? null,
      title_id: titleId,
      facts,
    },
    footer: {
      printed_label_id: 'print-printed-at',
      printed_at: '10-10-2026 17:40',
      page_label_id: 'print-page-label',
      of_label_id: 'print-page-of',
    },
  };
}

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
    detail: `(${n.banana} ${qty} . ${rate})`,
    note: null,
  });
  return {
    serial: '1',
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

function slipPage(language: Language, spec: SlipSpec, serial = '1'): PrintPage {
  const n = NAMES[language];
  return {
    serial,
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
        // The client's own way of writing a line (R5): "(item qty . rate)", built by the backend.
        detail: `(${spec.item} ${spec.qty} . ${spec.rate})`,
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
    slipPage(
      language,
      {
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
      },
      '2',
    ),
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
  receipt_slip: 'print-receipt-slip-title',
  collection_list: 'print-collection-list-title',
};

/** The paper each document has before the Owner changes it (PRD A21). */
export const DEFAULT_PAPER: Record<PrintDoc, Paper> = {
  seller_bill: 'a4',
  buyer_slips: 'a5',
  pukhta_slip: 'a5',
  receipt_slip: 'a5',
  collection_list: 'a4',
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
    ...frameOf(language, TITLES[doc], doc === 'pukhta_slip' ? FACTS_BIKRI_2 : FACTS, options),
    number: doc === 'pukhta_slip' ? '2' : '1',
    cancelled: options.cancelled === true,
    pages,
  };
  return options.urduDigits === true ? withUrduDigits(data) : data;
}

/**
 * The numbers of a print in Urdu digits, the way the backend writes them with the setting on
 * (NFR-URD-14): amounts, dates, serials, counts and rates. A phone, a code and a short code are never
 * converted (backend, 26-10-2026), and neither is a truck number, which is text as typed (PRD FR-7.25).
 */
function withUrduDigits(data: PrintData): PrintData {
  const d = toUrduDigits;
  return {
    ...data,
    number: d(data.number),
    header: {
      ...data.header,
      facts: data.header.facts.map((fact) =>
        fact.label_id === 'print-fact-truck' ? fact : { ...fact, value: d(fact.value) },
      ),
    },
    footer: { ...data.footer, printed_at: d(data.footer.printed_at) },
    pages: data.pages.map((page) => ({
      ...page,
      serial: d(page.serial),
      lines: page.lines.map((line) => ({
        ...line,
        date: d(line.date),
        qty: d(line.qty),
        rate: d(line.rate),
        amount: d(line.amount),
        detail: d(line.detail),
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

/** The kinds of voucher the receipt slip prints, as the backend's title ids name them. */
export type SlipKind = 'receipt' | 'payment' | 'advance';

export interface SlipOptions extends CannedOptions {
  /** Reason, collector and narration facts on the slip. */
  extras?: boolean;
}

const SLIP_TITLE: Record<SlipKind, string> = {
  receipt: 'print-receipt-slip-title',
  payment: 'print-payment-slip-title',
  advance: 'print-advance-slip-title',
};
const SLIP_TOTAL: Record<SlipKind, string> = {
  receipt: 'print-slip-amount-received',
  payment: 'print-slip-amount-paid',
  advance: 'print-slip-amount-advance',
};

/**
 * The receipt slip of a voucher (plan 15 contract 9), in the exact shape of the generated types:
 * one page with the party, no goods, one total, the amount in words and the facts. A receipt of
 * 10,000 from Akram on 10-10-2026, number R-1 (و-1), or a payment of 4,316 to Rahim, or an advance
 * of 5,000 to Rahim. The numbers are the PRD's.
 */
export function voucherPrintData(kind: SlipKind, options: SlipOptions = {}): PrintData {
  const language = options.language ?? 'ur';
  const n = NAMES[language];
  const amount = kind === 'receipt' ? '10,000' : kind === 'payment' ? '4,316' : '5,000';
  const words =
    language === 'ur'
      ? kind === 'receipt'
        ? 'دس ہزار روپے'
        : kind === 'payment'
          ? 'چار ہزار تین سو سولہ روپے'
          : 'پانچ ہزار روپے'
      : kind === 'receipt'
        ? 'Ten thousand rupees only'
        : kind === 'payment'
          ? 'Four thousand three hundred sixteen rupees only'
          : 'Five thousand rupees only';
  const number =
    language === 'ur'
      ? `${kind === 'receipt' ? 'و' : kind === 'payment' ? 'ا' : 'پ'}-1`
      : `${kind === 'receipt' ? 'R' : kind === 'payment' ? 'P' : 'A'}-1`;
  const facts: PrintFact[] = [
    { label_id: 'print-fact-voucher-number', value: number },
    { label_id: 'print-fact-date', value: '10-10-2026' },
    ...(options.extras === true
      ? [
          { label_id: 'print-fact-collector', value: language === 'ur' ? 'اسلم' : 'Aslam' },
          {
            label_id: 'print-fact-reason',
            value: language === 'ur' ? 'گھریلو ضرورت' : 'Home need',
          },
          { label_id: 'print-fact-narration', value: 'x' },
        ]
      : []),
  ];
  const page: PrintPage = {
    serial: '1',
    party: {
      role_id: kind === 'receipt' ? 'print-party-buyer' : 'print-party-seller',
      code: kind === 'receipt' ? '03040001' : '030110001',
      name: kind === 'receipt' ? n.akram : n.rahim,
      phone: kind === 'receipt' ? '0300-1234567' : null,
    },
    lines: [],
    amounts: [{ label_id: SLIP_TOTAL[kind], amount, role: 'total' }],
    // All money is روکڑ: a slip has no line about how it was paid (PRD 15.1).
    payment_id: null,
    words: { label_id: 'print-in-words', text: words },
  };
  const data: PrintData = {
    doc: 'receipt_slip',
    language,
    dir: language === 'ur' ? 'rtl' : 'ltr',
    paper: options.paper ?? 'a5',
    urdu_digits: options.urduDigits === true,
    ...frameOf(language, SLIP_TITLE[kind], facts, options),
    number,
    cancelled: options.cancelled === true,
    pages: [page],
  };
  return options.urduDigits === true ? withUrduDigits(data) : data;
}

/**
 * The collection list of 10-10-2026 (PRD FR-15.6), in the exact shape of the generated types: two
 * groups. Aslam collects in Kot Radha Kishan from Akram (2,880) and Arshad (18,684), 21,564 in all
 * (the PRD worked day after Events 1 to 5); a buyer with no collector and no city, Nasir, owes
 * 2,000. The list total, 23,564, is the total OF THIS LIST: every number is the backend's, and the
 * tests check that the template shows them as they are and adds nothing up.
 */
export function collectionPrintData(options: CannedOptions = {}): CollectionPrintData {
  const language = options.language ?? 'ur';
  const ur = language === 'ur';
  const row = (
    serial: string,
    code: string,
    name: string,
    dues: string,
    oldest: string | null,
    days: string | null,
    tag?: string,
  ): CollectionPrintRow => ({
    serial,
    code,
    name,
    ...(tag === undefined ? {} : { tag }),
    phone: serial === '3' ? '' : `0300-00000${serial}`,
    dues,
    oldest_bill_date: oldest,
    oldest_days: days,
  });
  const data: CollectionPrintData = {
    doc: 'collection_list',
    language,
    dir: ur ? 'rtl' : 'ltr',
    paper: options.paper ?? DEFAULT_PAPER.collection_list,
    urdu_digits: options.urduDigits === true,
    ...frameOf(
      language,
      'print-collection-list-title',
      [
        { label_id: 'print-fact-date', value: '10-10-2026' },
        { label_id: 'print-fact-collector', value: ur ? 'اسلم' : 'Aslam' },
        { label_id: 'print-fact-city', value: ur ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan' },
      ],
      options,
    ),
    groups: [
      {
        collector: ur ? 'اسلم' : 'Aslam',
        city: ur ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan',
        rows: [
          row('1', '1', ur ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax', '2,880', '10-10-2026', '0', 'K'),
          row('2', '2', ur ? 'ارشد ملتانی' : 'Arshad Multani', '18,684', '10-10-2026', '0'),
        ],
        total: '21,564',
      },
      {
        collector: null,
        city: null,
        rows: [row('3', '9', ur ? 'ناصر' : 'Nasir', '2,000', null, null)],
        total: '2,000',
      },
    ],
    total: { label_id: 'print-collection-total', amount: '23,564', role: 'total' },
    buyer_count: '3',
  };
  return options.urduDigits === true ? collectionWithUrduDigits(data) : data;
}

function collectionWithUrduDigits(data: CollectionPrintData): CollectionPrintData {
  const d = toUrduDigits;
  return {
    ...data,
    header: {
      ...data.header,
      // A phone and a code are never converted, even with the setting on (backend, 26-10-2026).
      facts: data.header.facts.map((fact) => ({ ...fact, value: d(fact.value) })),
    },
    footer: { ...data.footer, printed_at: d(data.footer.printed_at) },
    groups: data.groups.map((group) => ({
      ...group,
      total: d(group.total),
      rows: group.rows.map((item) => ({
        ...item,
        serial: d(item.serial),
        dues: d(item.dues),
        oldest_bill_date: item.oldest_bill_date === null ? null : d(item.oldest_bill_date),
        oldest_days: item.oldest_days === null ? null : d(item.oldest_days),
      })),
    })),
    total: { ...data.total, amount: d(data.total.amount) },
    buyer_count: d(data.buyer_count),
  };
}
