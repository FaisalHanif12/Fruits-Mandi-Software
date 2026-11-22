// The seller's parchi shows the same lines as the summary on the screen (plan 91, PRD FR-12.23 v1.53, FR-19.12,
// FR-19.15, Umar 21-11-2026): his day slip (R35) and his bill from the print menu (the seller bill). Drawn from the
// backend's REAL print data of the worked case (previous 30,000 بنام, today's credit 20,000, paid 5,000, the rest 15,000
// credited to his account, new 15,000 بنام) and of two lots paid 15,000, in Urdu and in English. The page lays out what
// it is given: every line has its words (no message id shows raw), the lines stand in the backend's order, each balance
// has its side word, and the new line reads with Umar's wording.

import { render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { bareDigits } from '../helpers/bare-digits';
import { printFrameFor } from '../helpers/print-frame';
import { message, visible } from '../helpers/render';
import {
  SELLER_BILL_A_EN,
  SELLER_BILL_A_UR,
  SELLER_BILL_TWO_EN,
  SELLER_BILL_TWO_UR,
  SELLER_SLIP_A_EN,
  SELLER_SLIP_A_UR,
  SELLER_SLIP_TWO_EN,
  SELLER_SLIP_TWO_UR,
} from '../helpers/seller-paid-real';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.style.removeProperty(FOOTER_PROPERTY);
});

function show(data: ReportPrintData) {
  const frame = {
    ...printFrameFor({ language: data.language, title_id: data.title_id, facts: data.filters }),
    dir: data.dir,
  };
  const result = render(<ReportPrintDocument data={data} frame={frame} />);
  const root = result.container.querySelector<HTMLElement>('.print-doc');
  if (root === null) throw new Error('no print document');
  return { ...result, root };
}

const textOf = (element: Element | null | undefined) => visible(element?.textContent);

/** A cell as the reader sees it: its figure and, after it, the side word of a balance. */
function figureOf(cell: Element | null | undefined): string {
  if (cell === null || cell === undefined) return '';
  const parts = Array.from(cell.querySelectorAll('.type-number, .print-balance-side')).map(textOf);
  const words = parts.filter((part) => part !== '');
  return words.length > 0 ? words.join(' ') : textOf(cell);
}

/** The labelled lines of a slip: each label with its figure (and side word). */
function linesOf(root: HTMLElement): [string, string][] {
  return Array.from(root.querySelectorAll('.print-slip-lines tr')).map((line) => [
    textOf(line.querySelector('th')),
    figureOf(line.querySelector('td')),
  ]);
}

/** What the worked case and the two-lot case print, line by line: label id, figure, and the side of a balance. */
interface Expected {
  paid: string;
  rest: string;
  newBalance: string;
  newSide: 'dr' | 'cr';
  left: string;
}

const A: Expected = {
  paid: '5,000',
  rest: '15,000',
  left: '15,000',
  newBalance: '15,000',
  newSide: 'dr',
};
const TWO: Expected = {
  paid: '15,000',
  rest: '5,000',
  left: '5,000',
  newBalance: '25,000',
  newSide: 'dr',
};

const CASES = [
  ['day slip', 'en', SELLER_SLIP_A_EN, A],
  ['day slip', 'ur', SELLER_SLIP_A_UR, A],
  ['seller bill', 'en', SELLER_BILL_A_EN, A],
  ['seller bill', 'ur', SELLER_BILL_A_UR, A],
  ['day slip of two lots', 'en', SELLER_SLIP_TWO_EN, TWO],
  ['day slip of two lots', 'ur', SELLER_SLIP_TWO_UR, TWO],
  ['seller bill of two lots', 'en', SELLER_BILL_TWO_EN, TWO],
  ['seller bill of two lots', 'ur', SELLER_BILL_TWO_UR, TWO],
] as const;

describe.each(CASES)('the seller’s %s in %s', (_what, language, data, expected) => {
  const m = (id: string) => message(language, id);
  const side = { dr: m('print-slip-side-dr'), cr: m('print-slip-side-cr') };

  it('has the lines of the screen: today’s net, paid, the rest credited to his account, previous and new balance, each balance with its side', () => {
    const { root } = show(data);
    expect(linesOf(root)).toEqual([
      [m('report-r35-gross'), '20,000'],
      [m('report-r35-net-sale'), `20,000 ${side.cr}`],
      [m('report-r35-paid-today-seller'), expected.paid],
      [m('report-r35-sale-rest'), expected.rest],
      [m('report-r35-today-left-seller'), `${expected.left} ${side.cr}`],
      [m('report-r35-previous'), `30,000 ${side.dr}`],
      [m('report-r35-new-balance'), `${expected.newBalance} ${side[expected.newSide]}`],
    ]);
  });

  it('reads the new line with Umar’s words, the same as the screen', () => {
    const { root } = show(data);
    const rest = linesOf(root).find(([, figure]) => figure === expected.rest)?.[0];
    expect(rest).toBe(
      language === 'ur'
        ? 'بکری کی باقی رقم، کھاتے میں جمع'
        : "Rest of today's Sale, credited to his account",
    );
  });

  it('shows no message id raw and no bare digit', () => {
    const { root } = show(data);
    expect(textOf(root)).not.toMatch(/\b(report|print|common)-[a-z0-9-]+/);
    expect(bareDigits(root)).toEqual([]);
  });
});
