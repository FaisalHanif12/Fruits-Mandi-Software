// The printed report (plan 20, PRD FR-7.36, FR-19.3, FR-19.5, FR-19.7, FR-19.13, NFR-URD-13): the
// print data of the backend in Urdu and in English, drawn by the shared print layout. Each value is
// shown exactly as the backend wrote it, every number is isolated left to right, the print language
// is the print's own (not the app's), a party has its short code and its name with its tag, the
// totals row is under the columns, and the amount in words is there where the backend sent it.

import { render, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { printFrameFor } from '../helpers/print-frame';
import { message, visible } from '../helpers/render';
import {
  PRINT_R13_AKRAM_UR,
  PRINT_R18_EN,
  PRINT_R20_UR,
  PRINT_R28_MONTH_EN,
  PRINT_R7_EN,
  PRINT_R9_DUE_EN,
} from '../helpers/reports-real';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.style.removeProperty(FOOTER_PROPERTY);
});

/**
 * The header and footer the backend builds for this print from its title and filters, in the print's
 * language, direction and digit style (the print data of the test says which).
 */
function frameOf(data: ReportPrintData) {
  const frame = printFrameFor({
    language: data.language,
    title_id: data.title_id,
    facts: data.filters,
  });
  return { ...frame, dir: data.dir, urdu_digits: data.urdu_digits };
}

function show(data: ReportPrintData, applyRoot = false) {
  const result = render(
    <ReportPrintDocument data={data} frame={frameOf(data)} applyRoot={applyRoot} />,
  );
  const root = result.container.querySelector<HTMLElement>('.print-doc');
  if (root === null) throw new Error('no print document');
  return { ...result, root };
}

const textOf = (element: Element | null | undefined) => visible(element?.textContent);

/** True when `value` is shown inside an element written left to right, so it never flips. */
function isolated(root: HTMLElement, value: string): boolean {
  const found = within(root)
    .getAllByText((_, element) => element?.tagName === 'BDI' && textOf(element) === value)
    .map((element) => element.closest('bdi'));
  return found.length > 0 && found.every((bdi) => bdi?.getAttribute('dir') === 'ltr');
}

describe('the trial balance in English (R18)', () => {
  const data = PRINT_R18_EN;

  it('is a left-to-right A4 page in English with Latin digits', () => {
    const { root } = show(data);
    expect(root.getAttribute('lang')).toBe('en');
    expect(root.getAttribute('dir')).toBe('ltr');
    expect(root.getAttribute('data-paper')).toBe('a4');
    expect(root.getAttribute('data-digits')).toBe('latin');
  });

  it('has the firm and the title in the header', () => {
    const { root } = show(data);
    const header = must(root.querySelector('.print-header'));
    expect(textOf(header)).toContain(frameOf(data).header.firm.name);
    expect(textOf(header)).toContain(message('en', 'report-r18-title'));
    expect(textOf(header)).toContain(frameOf(data).header.firm.phone);
  });

  it('shows the heading of every column and one row for each of the 17 accounts', () => {
    const { root } = show(data);
    const heads = Array.from(root.querySelectorAll('thead th')).map(textOf);
    expect(heads).toEqual(data.columns.map((column) => message('en', column.title_id)));
    expect(root.querySelectorAll('tbody tr')).toHaveLength(17);
  });

  it('shows an account with its code and name, and its figure on the right edge', () => {
    const { root } = show(data);
    const first = must(root.querySelector('tbody tr'));
    expect(textOf(first.querySelectorAll('td')[0])).toBe('01010001 Cash');
    const debit = must(first.querySelectorAll('td')[1]);
    expect(textOf(debit)).toBe('58,484');
    expect(debit.classList.contains('print-num')).toBe(true);
    expect(isolated(root, '58,484')).toBe(true);
  });

  it('has the totals row under the columns, 85,048 on both sides, and the difference above', () => {
    const { root } = show(data);
    const totals = must(root.querySelector('tfoot tr'));
    const cells = Array.from(totals.querySelectorAll('td')).map(textOf);
    expect(cells).toEqual([message('en', 'report-total'), '85,048', '85,048']);
    // A balance of nothing has no side word.
    const difference = must(root.querySelector('.print-summary-lines'));
    expect(textOf(difference)).toContain(message('en', 'report-difference'));
    expect(textOf(difference)).toContain('0');
    expect(textOf(difference)).not.toContain(message('en', 'report-side-zero'));
  });

  it('has no amount in words (a trial balance has none)', () => {
    const { root } = show(data);
    expect(root.querySelector('.print-words')).toBeNull();
  });
});

describe('the ledger of اکرم in Urdu (R13)', () => {
  const data = PRINT_R13_AKRAM_UR;

  it('is a right-to-left page in Urdu', () => {
    const { root } = show(data);
    expect(root.getAttribute('lang')).toBe('ur');
    expect(root.getAttribute('dir')).toBe('rtl');
  });

  it('prints the party filter with its short code and name in the header', () => {
    const { root } = show(data);
    const facts = textOf(root.querySelector('.print-facts'));
    expect(facts).toContain(message('ur', 'report-filter-party'));
    expect(facts).toContain('0 اکرم ٹول ٹیکس');
  });

  it('writes the goods of the sale line with the Urdu words of the PRD example', () => {
    const { root } = show(data);
    const detail = textOf(root.querySelector('tbody tr .print-lines'));
    expect(detail).toBe(
      message('ur', 'report-detail-sale', {
        number: '1',
        item: 'کیلا',
        qty: '20',
        packing: 'پیٹی',
        rate: '600',
        value: '12,000',
        commission: '720',
        laga: '160',
      }),
    );
  });

  it('shows the kind and number of each entry, and the balance with its side word in a slot', () => {
    const { root } = show(data);
    const rows = Array.from(root.querySelectorAll('tbody tr'));
    expect(textOf(rows[0]?.querySelectorAll('td')[1])).toBe(
      `${message('ur', 'report-doc-bikri-first')} 1`,
    );
    expect(textOf(rows[1]?.querySelectorAll('td')[1])).toBe(
      `${message('ur', 'report-doc-receipt')} 1`,
    );
    const balance = must(rows[1]?.querySelectorAll('td')[6]);
    expect(textOf(balance.querySelector('.print-balance-side'))).toBe(
      message('ur', 'report-side-dr'),
    );
    expect(textOf(balance)).toContain('2,880');
  });

  it('has the closing balance in words, as the backend wrote them', () => {
    const { root } = show(data);
    const words = must(root.querySelector('.print-words'));
    expect(textOf(words)).toContain(message('ur', 'print-in-words'));
    expect(textOf(words)).toContain('دو ہزار آٹھ سو اسی روپے');
  });

  it('keeps every figure, date and short code isolated left to right inside the Urdu text', () => {
    const { root } = show(data);
    expect(isolated(root, '12,880')).toBe(true);
    expect(isolated(root, '10,000')).toBe(true);
    expect(isolated(root, '10-10-2026')).toBe(true);
    for (const number of root.querySelectorAll('.type-number')) {
      expect(number.closest('bdi')?.getAttribute('dir')).toBe('ltr');
    }
  });

  it('has the closing balance above the table and the totals under it', () => {
    const { root } = show(data);
    const summary = textOf(root.querySelector('.print-summary-lines'));
    expect(summary).toContain(message('ur', 'report-opening-balance'));
    expect(summary).toContain(message('ur', 'report-closing-balance'));
    const totals = Array.from(must(root.querySelector('tfoot tr')).querySelectorAll('td')).map(
      textOf,
    );
    expect(totals[0]).toBe(message('ur', 'report-total'));
    expect(totals[4]).toBe('12,880');
    expect(totals[5]).toBe('10,000');
    expect(totals[6]).toContain('2,880');
  });

  it('puts the print language and the footer on the root in the hidden window only', () => {
    const preview = show(data);
    expect(document.documentElement.style.getPropertyValue(FOOTER_PROPERTY)).toBe('');
    preview.unmount();
    show(data, true);
    expect(document.documentElement.lang).toBe('ur');
    expect(document.documentElement.dir).toBe('rtl');
    expect(document.documentElement.style.getPropertyValue(FOOTER_PROPERTY)).toContain(
      'counter(page)',
    );
  });
});

describe('the reports of round 4 in print (R7, R9, R20, R28)', () => {
  it('R7 in English: the item rows and the totals row of E2E-11 (5,448 of commission)', () => {
    const { root } = show(PRINT_R7_EN);
    expect(root.getAttribute('lang')).toBe('en');
    const first = must(root.querySelector('tbody tr'));
    expect(textOf(first.querySelectorAll('td')[0])).toBe('Banana');
    const totals = Array.from(must(root.querySelector('tfoot tr')).querySelectorAll('td')).map(
      textOf,
    );
    expect(totals).toContain('5,448');
    expect(totals).toContain('41,400');
  });

  it('R20 in Urdu: every column, the print language is its own, and the total is the shop’s', () => {
    const { root } = show(PRINT_R20_UR);
    expect(root.getAttribute('lang')).toBe('ur');
    expect(root.getAttribute('dir')).toBe('rtl');
    // The print has every column the backend sent: the screen folds some, the paper does not.
    expect(root.querySelectorAll('thead th')).toHaveLength(PRINT_R20_UR.columns.length);
    expect(textOf(must(root.querySelector('tfoot tr')))).toContain('21,564');
  });

  it('R28 by month in English: the month is named from Fluent and the year is a number', () => {
    const { root } = show(PRINT_R28_MONTH_EN);
    const first = must(root.querySelector('tbody tr'));
    expect(textOf(first.querySelectorAll('td')[0])).toBe(
      `${message('en', 'report-month-10')} 2026`,
    );
    expect(isolated(root, '2026')).toBe(true);
  });

  it('R9 for the due lines only: the filter is named in the header', () => {
    const { root } = show(PRINT_R9_DUE_EN);
    expect(textOf(root.querySelector('.print-facts'))).toContain(
      message('en', 'report-filter-due'),
    );
  });

  it('names the heading of the buyers with no city, and the sales with no اڈا, in words', () => {
    const group = { ...PRINT_R20_UR, report: 'R21' as const };
    const row = group.rows[0];
    if (row === undefined) throw new Error('no row');
    const empty = {
      ...row,
      kind: 'group' as const,
      cells: [{ type: 'empty' as const }, ...row.cells.slice(1)],
    };
    const city = show({ ...group, rows: [empty] });
    expect(textOf(city.root.querySelector('tbody tr td'))).toBe(
      message('ur', 'report-group-no-city'),
    );
    city.unmount();
    const adda = show({
      ...group,
      report: 'R8' as const,
      rows: [{ ...empty, kind: 'normal' as const }],
    });
    expect(textOf(adda.root.querySelector('tbody tr td'))).toBe(message('ur', 'report-adda-none'));
  });
});

function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}
