// A test double of the statement print data (plan 74 T3, PRD FR-12.47): the example of the PRD, a buyer from
// 01-11-2026 to 06-11-2026 (bought 51,360, paid on the spot 21,360, closing 30,000 بنام), and a 30-day statement
// of a buyer with several lines of goods, quality words, commission and لاگا on one line, receipts and a part
// paid on the spot. The figures are written out in `statement-doubles.json` by a script that works them out
// once (6% commission rounded to the rupee, 8 for each نگ, balance = opening + bill - paid - received); the
// app never does this arithmetic. The shape is the contract of plan 74; when the backend's real files are
// copied the tests read those instead (the double stays for the development page).

import type { Language } from '../../src/i18n';
import type { PrintFrame } from '../../src/bindings/PrintFrame';
import type { StatementDay } from '../../src/bindings/StatementDay';
import type { StatementPrintData } from '../../src/bindings/StatementPrintData';
import type { StatementPrintDataInput } from '../../src/bindings/StatementPrintDataInput';
import doubles from './statement-doubles.json';
import akramEn from '../fixtures/party_ledgers_statement_print_akram.json';
import akramEnRequest from '../fixtures/party_ledgers_statement_print_akram_request.json';
import akramUr from '../fixtures/party_ledgers_statement_print_akram_ur.json';
import akramUrRequest from '../fixtures/party_ledgers_statement_print_akram_ur_request.json';
import emptyReal from '../fixtures/party_ledgers_statement_print_empty.json';
import emptyRealRequest from '../fixtures/party_ledgers_statement_print_empty_request.json';
import exampleReal from '../fixtures/party_ledgers_statement_print_example.json';
import exampleRealRequest from '../fixtures/party_ledgers_statement_print_example_request.json';
import rahimReal from '../fixtures/party_ledgers_statement_print_rahim.json';
import rahimRealRequest from '../fixtures/party_ledgers_statement_print_rahim_request.json';
import { printFrameFor } from './print-frame-real';

interface DoubleDays {
  days: StatementDay[];
  totals: StatementPrintData['totals'];
  closing: number;
  opening?: number;
}

const WORDS: Record<string, Record<Language, string>> = {
  '30000': { ur: 'تیس ہزار روپے', en: 'Thirty thousand rupees only' },
  '252216': {
    ur: 'دو لاکھ باون ہزار دو سو سولہ روپے',
    en: 'Two hundred fifty-two thousand two hundred sixteen rupees only',
  },
};

const FIRST_DAY = '14-10-2026';

function party(language: Language): StatementPrintData['party'] {
  return {
    role_id: 'print-party-buyer',
    code: '0020',
    short_code: '0',
    name: language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    phone: '0300-1234567',
  };
}

function sideOf(amount: number): 'debit' | 'credit' | 'none' {
  return amount > 0 ? 'debit' : amount < 0 ? 'credit' : 'none';
}

function grouped(amount: number): string {
  return Math.abs(amount).toLocaleString('en-US');
}

function build(
  language: Language,
  source: DoubleDays,
  period: { from: string; to: string },
  opening: number,
): StatementPrintData {
  return {
    language,
    dir: language === 'ur' ? 'rtl' : 'ltr',
    paper: 'a4',
    title_id: 'print-statement-title',
    filters: [
      {
        label_id: 'report-filter-party',
        value: `0 ${party(language).name}`,
      },
      { label_id: 'report-filter-from', value: period.from },
      { label_id: 'report-filter-to', value: period.to },
    ],
    kind: 'buyer',
    party: party(language),
    period_from: period.from,
    period_to: period.to,
    opening: { amount: grouped(opening), side: sideOf(opening) },
    days: source.days,
    totals: source.totals,
    closing: { amount: grouped(source.closing), side: sideOf(source.closing) },
    words:
      source.closing === 0
        ? null
        : {
            label_id: 'print-in-words',
            text: WORDS[String(source.closing)]?.[language] ?? '',
          },
  };
}

type Doubles = Record<'example' | 'thirty', Record<Language, DoubleDays>>;
const SOURCE = doubles as unknown as Doubles;

/** The example of PRD FR-12.47: 01-11-2026 to 06-11-2026, opening 0, closing 30,000 بنام. */
export function statementExample(language: Language): StatementPrintData {
  return build(language, SOURCE.example[language], { from: '01-11-2026', to: '06-11-2026' }, 0);
}

/** Thirty days, 14-10-2026 to 12-11-2026: 14 dates with an entry, opening 12,500 بنام, closing 252,216 بنام. */
export function statementThirtyDays(language: Language): StatementPrintData {
  return build(
    language,
    SOURCE.thirty[language],
    { from: FIRST_DAY, to: '12-11-2026' },
    SOURCE.thirty[language].opening ?? 0,
  );
}

/** A period with no entry at all. */
export function statementEmpty(language: Language): StatementPrintData {
  return {
    ...statementExample(language),
    days: [],
    totals: [],
    opening: { amount: '0', side: 'none' },
    closing: { amount: '0', side: 'none' },
    words: null,
  };
}

/**
 * The backend's own answers (backend/crates/reports/tests/fixtures, copied, never edited) with the request each
 * one answers: the FR-12.47 buyer (51,360 / 21,360 / 30,000 debit), a buyer of five dates in English and in
 * Urdu, a seller (advance, two lots, deductions, cash, a payment), and a period with no entry.
 */
export const STATEMENT_REAL = {
  example: { request: exampleRealRequest, data: exampleReal as unknown as StatementPrintData },
  akramEn: { request: akramEnRequest, data: akramEn as unknown as StatementPrintData },
  akramUr: { request: akramUrRequest, data: akramUr as unknown as StatementPrintData },
  rahim: { request: rahimRealRequest, data: rahimReal as unknown as StatementPrintData },
  empty: { request: emptyRealRequest, data: emptyReal as unknown as StatementPrintData },
} as const;

/** The real answer to this request, when the backend wrote one for it. */
function realAnswer(input: StatementPrintDataInput): StatementPrintData | null {
  for (const { request, data } of Object.values(STATEMENT_REAL)) {
    if (
      request.account_id === input.account_id &&
      request.from === input.from &&
      request.to === input.to &&
      request.language === input.language
    ) {
      return data;
    }
  }
  return null;
}

/** What the backend answers for a request: its own answer when it wrote one, else the double of thirty days. */
export function statementAnswer(input: StatementPrintDataInput): StatementPrintData {
  const real = realAnswer(input);
  if (real !== null) return real;
  const data =
    input.from === '01-11-2026' && input.to === '06-11-2026'
      ? statementExample(input.language)
      : input.from === '01-01-2000'
        ? statementEmpty(input.language)
        : statementThirtyDays(input.language);
  return { ...data, period_from: input.from, period_to: input.to };
}

/** The header and footer the backend builds for the print. */
export function statementFrame(data: StatementPrintData): PrintFrame {
  return printFrameFor({
    language: data.language,
    title_id: data.title_id,
    facts: data.filters,
  });
}
