// Plan 95 prints (PRD FR-19.15 v1.57, FR-12.52): the whole sale print's buyers table shows the goods value, the
// commission and the لاگا before the bill, each with a total, and the seller's cash paid reads نقد ادائیگی; all from the
// backend's REAL answer of the worked day. The lot advance (پیشگی) on the paper is in lot-advance-prints-real.test.tsx.
// The pages lay out what they are given and add up nothing (CLAUDE.md rule 1). In the Urdu and in the English app:
// every id has its words, no digit is bare, and the number columns are centred.

import { render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { SalePrintDocument } from '../../src/features/reports/print/SalePrintDocument';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { bareDigits } from '../helpers/bare-digits';
import { salePrintData } from '../helpers/day-slips';
import { message, visible } from '../helpers/render';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.style.removeProperty(FOOTER_PROPERTY);
});

const textOf = (element: Element | null | undefined) => visible(element?.textContent);
const cellTexts = (row: Element | null | undefined) =>
  [...(row?.querySelectorAll('th, td') ?? [])].map((cell) => textOf(cell));

describe.each(LANGUAGES)('the whole sale print with the bill parts (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);

  function draw() {
    useLanguageStore.setState({ language });
    return render(<SalePrintDocument data={salePrintData(language)} />).container;
  }

  it('the buyers table reads buyer, goods, value, commission, لاگا, bill, cash, credit, with a totals row', () => {
    const page = draw();
    const buyers = [...page.querySelectorAll('table')][1];
    const headings = [...(buyers?.querySelectorAll('thead th') ?? [])].map(textOf);
    expect(headings).toEqual(
      [
        'print-sale-col-buyer',
        'print-sale-col-bought',
        'print-sale-col-value',
        'print-sale-col-commission',
        'print-sale-col-laga',
        'print-sale-col-bill',
        'print-sale-col-cash',
        'print-sale-col-credit',
      ].map(m),
    );
    const rows = [...(buyers?.querySelectorAll('tbody tr') ?? [])].map(cellTexts);
    // Akram: 12,000 + 720 + 160 = 12,880, nothing paid on the spot.
    const akram = rows.find((row) => row[0]?.includes(language === 'ur' ? 'اکرم' : 'Akram'));
    expect(akram?.slice(2)).toEqual(['12,000', '720', '160', '12,880', '0', '12,880']);
    // The totals row is the backend's, a total under each of the new columns.
    expect(cellTexts(buyers?.querySelector('tfoot tr')).slice(2)).toEqual([
      '41,400',
      '2,484',
      '480',
      '44,364',
      '12,800',
      '31,564',
    ]);
  });

  it('the new columns are number columns: centred with their headings, and no message id or bare digit shows', () => {
    const page = draw();
    const buyers = [...page.querySelectorAll('table')][1];
    const heads = [...(buyers?.querySelectorAll('thead th') ?? [])];
    // value, commission, لاگا, bill, cash and credit are all numbers.
    expect(heads.slice(2).map((head) => head.getAttribute('data-align'))).toEqual(
      Array.from({ length: 6 }, () => 'num'),
    );
    const firstRow = buyers?.querySelector('tbody tr');
    expect(
      [...(firstRow?.querySelectorAll('td') ?? [])]
        .slice(2)
        .map((cell) => cell.getAttribute('data-align')),
    ).toEqual(Array.from({ length: 6 }, () => 'num'));
    expect(textOf(page)).not.toMatch(/\b(report|print|common)-[a-z0-9-]+/);
    expect(bareDigits(page)).toEqual([]);
  });

  it('the sellers table of a day without a lot advance has no advance column (the real data of the worked day)', () => {
    useLanguageStore.setState({ language });
    const page = render(<SalePrintDocument data={salePrintData(language)} />).container;
    const heads = [...(page.querySelectorAll('table')[0]?.querySelectorAll('thead th') ?? [])];
    expect(heads).toHaveLength(15);
    expect(heads.map(textOf)).not.toContain(m('print-sale-col-advance'));
  });
});

describe.each(LANGUAGES)('the seller’s cash paid on the prints (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);

  it('reads نقد ادائیگی (Cash Paid) on the R6 column, the seller bill and the seller slip, and the buyer’s cash keeps its own words', () => {
    const term = m('glossary-naqd-adaigi');
    expect(term).toBe(language === 'ur' ? 'نقد ادائیگی' : 'Cash Paid');
    // R6 and the whole sale print's sellers table (the R6 columns), the seller bill line and the seller slip line.
    for (const id of [
      'report-col-cash-paid',
      'print-cash-paid',
      'report-r35-paid-today-seller',
      'report-col-paid-today-seller',
    ]) {
      expect(m(id)).toBe(term);
    }
    // The seller statement's total of the cash given with his lots (PRD FR-12.52, FR-12.47).
    expect(m('print-statement-total-cash')).toContain(term);
    // The day summary's seller side (R34): the line reads the term after the sellers' word; the buyers' line is not changed.
    expect(m('report-r34-sellers-paid')).toContain(term);
    expect(m('report-r34-buyers-received')).not.toBe(m('report-r34-sellers-paid'));
    // The buyer's side is not changed: the money he paid on the spot and the cash he paid in.
    expect(m('report-r35-paid-today')).not.toBe(term);
    expect(m('print-sale-col-cash')).not.toBe(term);
    expect(m('print-paid-on-spot')).not.toBe(term);
  });
});
