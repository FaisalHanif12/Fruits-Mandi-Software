// The buyer's and the seller's day slip and the day total sheet printed (plan 42, PRD v1.12, FR-19.12,
// FR-19.13, R34, E2E-36, E2E-43): drawn from the backend's print data of the worked day (10-10-2026
// after Events 1 to 5), in Urdu and in English, on A4 and on the 80 mm roll. The page lays out what it
// is given: the worked numbers of plan 42 section 2 are read off it, the labelled lines stand in the
// backend's order, a balance has its side word, a line the backend sent empty is a blank to write on,
// the header has the owners and then the munshi, and no number is bare in the words.

import { render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { bareDigits } from '../helpers/bare-digits';
import {
  PRINT_DAY_SHEET_EN,
  PRINT_DAY_SHEET_UR,
  PRINT_SLIP_BUYER_EN,
  PRINT_SLIP_BUYER_UR,
  PRINT_SLIP_SELLER_EN,
  PRINT_SLIP_SELLER_UR,
  PRINT_SLIP_WEIGHED_EN,
  PRINT_SLIP_WEIGHED_UR,
} from '../helpers/day-slips-real';
import { printFrameFor } from '../helpers/print-frame';
import { message, visible } from '../helpers/render';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.style.removeProperty(FOOTER_PROPERTY);
});

function frameOf(data: ReportPrintData) {
  const frame = printFrameFor({
    language: data.language,
    title_id: data.title_id,
    facts: data.filters,
  });
  return { ...frame, dir: data.dir };
}

function show(data: ReportPrintData) {
  const result = render(<ReportPrintDocument data={data} frame={frameOf(data)} />);
  const root = result.container.querySelector<HTMLElement>('.print-doc');
  if (root === null) throw new Error('no print document');
  return { ...result, root };
}

function must<T extends Element>(element: T | null | undefined): T {
  if (element === null || element === undefined) throw new Error('missing element');
  return element;
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

const SIDE = {
  en: { dr: 'Debit', cr: 'Credit' },
  ur: { dr: 'بنام', cr: 'جمع' },
} as const;

const LANGS = [
  ['English', 'en', PRINT_SLIP_BUYER_EN, PRINT_SLIP_SELLER_EN, PRINT_DAY_SHEET_EN],
  ['Urdu', 'ur', PRINT_SLIP_BUYER_UR, PRINT_SLIP_SELLER_UR, PRINT_DAY_SHEET_UR],
] as const;

describe.each(LANGS)('the buyer slip of اکرم in %s', (_name, language, buyer) => {
  const m = (id: string) => message(language, id);
  const side = SIDE[language];

  it('is one page in the print language and direction, with the worked numbers of the day', () => {
    const { root } = show(buyer);
    expect(root.getAttribute('lang')).toBe(language);
    expect(root.getAttribute('dir')).toBe(language === 'ur' ? 'rtl' : 'ltr');
    expect(root.querySelectorAll('.print-sheet')).toHaveLength(1);
  });

  it('lists the goods with the five columns and no totals row', () => {
    const { root } = show(buyer);
    const heads = Array.from(root.querySelectorAll('thead th')).map(textOf);
    expect(heads).toEqual(
      [
        'report-col-item',
        'report-col-packing',
        'report-col-qty',
        'report-col-rate',
        'report-col-amount',
      ].map(m),
    );
    const cells = Array.from(root.querySelectorAll('.print-table tbody tr td')).map(textOf);
    expect(cells).toEqual([
      language === 'ur' ? 'کیلا' : 'Banana',
      language === 'ur' ? 'پیٹی' : 'Box',
      '20',
      '600',
      '12,000',
    ]);
    expect(root.querySelector('tfoot')).toBeNull();
  });

  it('has every labelled line on its own row, in the backend’s order, with the worked figures', () => {
    const { root } = show(buyer);
    expect(linesOf(root)).toEqual([
      [m('report-r35-goods'), '12,000'],
      [m('report-r35-commission'), '720'],
      [m('report-r35-laga'), '160'],
      [m('report-r35-today-total'), '12,880'],
      [m('report-r35-paid-today'), '10,000'],
      [m('report-r35-today-left'), `2,880 ${side.dr}`],
      [m('report-r35-previous'), '0'],
      [m('report-r35-new-balance'), `2,880 ${side.dr}`],
      [m('report-r35-total-qty'), '20'],
    ]);
  });

  it('sets the day’s total apart with a rule, and the new balance with a double rule', () => {
    const { root } = show(buyer);
    const rows = Array.from(root.querySelectorAll('.print-slip-lines tr'));
    expect(rows[3]?.classList.contains('print-amount-subtotal')).toBe(true);
    expect(rows[7]?.classList.contains('print-amount-total')).toBe(true);
  });

  it('draws a payment the backend left empty as a line to write on, not as a zero', () => {
    const blank: ReportPrintData = {
      ...buyer,
      summary: buyer.summary.map((line) =>
        line.label_id === 'report-r35-paid-today' ? { ...line, cell: { type: 'empty' } } : line,
      ),
    };
    const { root } = show(blank);
    const paid = must(
      Array.from(root.querySelectorAll('.print-slip-lines tr')).find(
        (line) => textOf(line.querySelector('th')) === m('report-r35-paid-today'),
      ),
    );
    expect(paid.querySelector('.print-write-line')).not.toBeNull();
    expect(textOf(paid.querySelector('td'))).toBe('');
  });

  it('puts the owners first and then the munshi in the header, each with his role word', () => {
    const { root } = show(buyer);
    const people = Array.from(root.querySelectorAll('.print-contacts li')).map(textOf);
    const role = (id: string) => m(id);
    expect(people).toHaveLength(3);
    expect(people[0]).toContain(role('print-role-owner'));
    expect(people[1]).toContain(role('print-role-owner'));
    expect(people[2]).toContain(role('print-role-munshi'));
    expect(people[2]).toContain('0300-4058860');
    expect(textOf(root.querySelector('.print-trade-mark'))).toBe('RH');
  });

  it('draws every number in its own left-to-right isolate, none bare in the words', () => {
    const { root } = show(buyer);
    expect(bareDigits(root)).toEqual([]);
  });
});

describe.each(LANGS)('the seller slip of رحیم in %s', (_name, language, _buyer, seller) => {
  const m = (id: string) => message(language, id);
  const side = SIDE[language];

  it('lists his two lots and, after them, the item total row with its quantity and amount', () => {
    const { root } = show(seller);
    const rows = Array.from(root.querySelectorAll('.print-table tbody tr'));
    expect(rows).toHaveLength(3);
    expect(rows.map((row) => textOf(row.querySelectorAll('td')[2]))).toEqual(['20', '30', '50']);
    expect(rows.map((row) => textOf(row.querySelectorAll('td')[4]))).toEqual([
      '12,000',
      '17,400',
      '29,400',
    ]);
    const total = must(root.querySelector('tr[data-row="item_total"]'));
    expect(textOf(total.querySelector('td'))).toBe(language === 'ur' ? 'کیلا کل' : 'Banana total');
    // The two lines of the item are not total rows.
    expect(root.querySelectorAll('tr[data-row="item_total"]')).toHaveLength(1);
  });

  it('lists the gross sale, each deduction, the net sale of the day and the balances, with sides', () => {
    const { root } = show(seller);
    expect(linesOf(root)).toEqual([
      [m('report-r35-gross'), '29,400'],
      [m('report-r35-commission'), '1,764'],
      [m('report-r35-freight'), '3,000'],
      [m('report-r35-labour'), '250'],
      [m('report-r35-munshiana'), '50'],
      [m('report-r35-phone'), '20'],
      [m('report-r35-net-sale'), `24,316 ${side.cr}`],
      [m('report-r35-paid-today'), '9,316'],
      [m('report-r35-today-left'), `15,000 ${side.cr}`],
      [m('report-r35-previous'), `15,000 ${side.dr}`],
      [m('report-r35-new-balance'), '0'],
    ]);
  });

  it('has no total quantity line, and a nil new balance has no side word', () => {
    const { root } = show(seller);
    const labels = linesOf(root).map(([label]) => label);
    expect(labels).not.toContain(m('report-r35-total-qty'));
    const last = must(root.querySelector('.print-slip-lines tr:last-child td'));
    expect(textOf(last.querySelector('.print-balance-side'))).toBe('');
  });

  it('draws every number in its own left-to-right isolate, none bare in the words', () => {
    const { root } = show(seller);
    expect(bareDigits(root)).toEqual([]);
  });
});

describe.each(LANGS)('the slips on the 80 mm roll in %s', (_name, language, buyer, seller) => {
  it('draws each line of goods as two rows of text, not as a table of five columns', () => {
    const { root } = show({ ...seller, paper: 'mm80' });
    expect(root.getAttribute('data-paper')).toBe('mm80');
    expect(root.querySelector('table.print-table')).toBeNull();
    const lines = Array.from(root.querySelectorAll('.print-compact-line'));
    expect(lines).toHaveLength(3);
    const first = textOf(lines[0]);
    expect(first).toContain('12,000');
    expect(first).toContain('20 @ 600');
    // The item total row is set apart and carries its quantity and amount.
    expect(lines[2]?.classList.contains('print-compact-total')).toBe(true);
    expect(textOf(lines[2])).toContain('29,400');
    expect(textOf(lines[2])).toContain(language === 'ur' ? 'کیلا کل' : 'Banana total');
  });

  it('keeps the labelled lines of the buyer slip the same on the roll', () => {
    const { root } = show({ ...buyer, paper: 'mm80' });
    expect(linesOf(root)).toHaveLength(9);
    expect(bareDigits(root)).toEqual([]);
  });
});

describe.each([
  ['English', 'en', PRINT_SLIP_WEIGHED_EN],
  ['Urdu', 'ur', PRINT_SLIP_WEIGHED_UR],
] as const)(
  'a line of an item sold by weight on a buyer slip in %s (PRD A73)',
  (_name, language, weighed) => {
    const m = (id: string) => message(language, id);

    it('has a sixth column of kilos with its heading, last, and the kilos of the weighed line', () => {
      expect(weighed.columns.at(-1)?.key).toBe('weight');
      const { root } = show(weighed);
      const heads = Array.from(root.querySelectorAll('.print-table thead th')).map(textOf);
      expect(heads).toHaveLength(6);
      expect(heads.at(-1)).toBe(m('report-col-weight'));
      const cells = Array.from(root.querySelectorAll('.print-table tbody tr td')).map(textOf);
      // 2 نگ, 12.345 kg at 100 a kilo: the stored amount is 1,235, not 2 times 100.
      expect(cells).toEqual([
        language === 'ur' ? 'انگور' : 'Grapes',
        language === 'ur' ? 'پیٹی' : 'Box',
        '2',
        '100',
        '1,235',
        '12.345',
      ]);
      expect(bareDigits(root)).toEqual([]);
    });

    it('has the worked lines of the day, and leaves what was paid today empty to write on', () => {
      const { root } = show(weighed);
      expect(linesOf(root).slice(0, 4)).toEqual([
        [m('report-r35-goods'), '1,235'],
        [m('report-r35-commission'), '74'],
        [m('report-r35-laga'), '16'],
        [m('report-r35-today-total'), '1,325'],
      ]);
      const paid = must(
        Array.from(root.querySelectorAll('.print-slip-lines tr')).find(
          (line) => textOf(line.querySelector('th')) === m('report-r35-paid-today'),
        ),
      );
      expect(paid.querySelector('.print-write-line')).not.toBeNull();
    });

    it('writes the kilos and the rate per kilo under the line on the 80 mm roll, with the amount beside it', () => {
      const { root } = show({ ...weighed, paper: 'mm80' });
      const line = must(root.querySelector('.print-compact-line'));
      expect(textOf(line)).toContain(language === 'ur' ? '12.345 کلو @ 100' : '12.345 kg @ 100');
      expect(textOf(line)).toContain('1,235');
      expect(bareDigits(root)).toEqual([]);
    });
  },
);

describe.each(LANGS)('the day total sheet R34 in %s', (_name, language, _buyer, _seller, day) => {
  const m = (id: string) => message(language, id);
  const side = SIDE[language];

  /** A row of the main table by the id of its line, as [count, amount]. */
  function lineOf(root: HTMLElement, id: string): [string, string] {
    const label = m(id);
    const row = must(
      Array.from(root.querySelectorAll('.print-table tbody tr')).find(
        (candidate) => textOf(candidate.querySelectorAll('td')[1]) === label,
      ),
    );
    const cells = row.querySelectorAll('td');
    return [figureOf(cells[2]), figureOf(cells[3])];
  }

  it('has the commission on its own lines and the total commission under them', () => {
    const { root } = show(day);
    expect(lineOf(root, 'report-r34-bikri')).toEqual(['2', '41,400']);
    expect(lineOf(root, 'report-r34-qty')).toEqual(['60', '']);
    expect(lineOf(root, 'report-r34-commission-seller')).toEqual(['', '2,484']);
    expect(lineOf(root, 'report-r34-commission-buyer')).toEqual(['', '2,484']);
    expect(lineOf(root, 'report-r34-commission-laga')).toEqual(['', '480']);
    expect(lineOf(root, 'report-r34-commission')).toEqual(['', '5,448']);
  });

  it('has the buyers’ and the sellers’ lines of the day', () => {
    const { root } = show(day);
    expect(lineOf(root, 'report-r34-buyers-billed')[1]).toBe('44,364');
    expect(lineOf(root, 'report-r34-buyers-received')[1]).toBe('22,800');
    expect(lineOf(root, 'report-r34-buyers-left')[1]).toBe(`21,564 ${side.dr}`);
    expect(lineOf(root, 'report-r34-sellers-net')[1]).toBe(`34,476 ${side.cr}`);
    expect(lineOf(root, 'report-r34-sellers-paid')[1]).toBe('9,316');
  });

  it('puts the two short tables after the sellers’ lines and before the cash lines', () => {
    const { root } = show(day);
    const tables = Array.from(root.querySelectorAll('.print-day-table'));
    expect(tables.map((table) => textOf(table.querySelector('h2')))).toEqual([
      m('report-r34-table-buyers'),
      m('report-r34-table-sellers'),
    ]);
    const paid = must(
      Array.from(root.querySelectorAll('.print-table tbody tr')).find(
        (row) => textOf(row.querySelectorAll('td')[1]) === m('report-r34-sellers-paid'),
      ),
    );
    const cash = must(
      Array.from(root.querySelectorAll('.print-table tbody tr')).find(
        (row) => textOf(row.querySelectorAll('td')[1]) === m('report-r34-cash-opening'),
      ),
    );
    expect(
      paid.compareDocumentPosition(must(tables[0])) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      must(tables[1]).compareDocumentPosition(cash) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  it('has the buyers’ table with a row for each buyer and the total row 0, 44,364, 22,800, 21,564', () => {
    const { root } = show(day);
    const table = must(root.querySelector('.print-day-table:nth-of-type(1)'));
    const heads = Array.from(table.querySelectorAll('thead th')).map(textOf);
    expect(heads).toEqual(
      [
        'report-col-code',
        'report-col-buyer',
        'report-col-previous',
        'report-col-today-total',
        'report-col-paid-today',
        'report-col-new-balance',
      ].map(m),
    );
    const rows = Array.from(table.querySelectorAll('tbody tr')).map((row) =>
      Array.from(row.querySelectorAll('td')).map(figureOf),
    );
    // The buyers are in the order of their Urdu names, as the backend sent them: ارشد, اکرم, شاہد.
    expect(rows.map((row) => [row[2], row[3], row[4], row[5]])).toEqual([
      ['0', '18,684', '0', `18,684 ${side.dr}`],
      ['0', '12,880', '10,000', `2,880 ${side.dr}`],
      ['0', '12,800', '12,800', '0'],
    ]);
    const total = Array.from(table.querySelectorAll('tfoot td')).map(figureOf);
    expect([total[1], total[2], total[3], total[4], total[5]]).toEqual([
      m('report-total'),
      '0',
      '44,364',
      '22,800',
      `21,564 ${side.dr}`,
    ]);
  });

  it('has the sellers’ table with his net sale of the day and his sides', () => {
    const { root } = show(day);
    const table = must(root.querySelectorAll('.print-day-table')[1]);
    const rows = Array.from(table.querySelectorAll('tbody tr')).map((row) =>
      Array.from(row.querySelectorAll('td')).map(figureOf),
    );
    expect(rows.map((row) => [row[2], row[3], row[4], row[5]])).toEqual([
      [`15,000 ${side.dr}`, `24,316 ${side.cr}`, '9,316', '0'],
      ['0', `10,160 ${side.cr}`, '0', `10,160 ${side.cr}`],
    ]);
    const total = Array.from(table.querySelectorAll('tfoot td')).map(figureOf);
    expect([total[2], total[3], total[4], total[5]]).toEqual([
      `15,000 ${side.dr}`,
      `34,476 ${side.cr}`,
      '9,316',
      `10,160 ${side.cr}`,
    ]);
  });

  it('puts the lines of the day in two columns on A4, none left out and in the same order, and one column on A5', () => {
    const { root } = show(day);
    const columns = Array.from(root.querySelectorAll('.print-two-columns'));
    // One pair for the lines before the short tables and one for the lines after them.
    expect(columns).toHaveLength(2);
    for (const pair of columns) expect(pair.querySelectorAll(':scope > table')).toHaveLength(2);
    // Read column after column, the 39 lines are the backend's lines in its order.
    const amounts = Array.from(root.querySelectorAll('.print-two-columns tbody tr')).map((row) =>
      textOf(row.querySelectorAll('td')[3]?.querySelector('.type-number')),
    );
    const expected = day.rows.map((row) => {
      const cell = row.cells[3];
      return cell !== undefined && 'text' in cell ? cell.text : '';
    });
    expect(amounts).toEqual(expected);
    expect(amounts).toHaveLength(39);
    // The short tables stay full width, outside the columns.
    expect(root.querySelectorAll('.print-two-columns .print-day-table')).toHaveLength(0);
    const a5 = show({ ...day, paper: 'a5' });
    expect(a5.root.querySelectorAll('.print-two-columns')).toHaveLength(0);
    expect(a5.root.querySelectorAll('.print-sheet > table.print-table tbody tr')).toHaveLength(39);
  });

  it('draws every number in its own left-to-right isolate, none bare in the words', () => {
    const { root } = show(day);
    expect(bareDigits(root)).toEqual([]);
  });
});
