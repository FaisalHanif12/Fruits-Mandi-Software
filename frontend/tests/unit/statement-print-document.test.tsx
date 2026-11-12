// The statement of one party on paper (plan 74 T3, PRD FR-12.47, FR-19.5, FR-19.7, FR-19.13, NFR-USE-19,
// NFR-URD-14), in the Urdu and the English print. The page is laid out from a double of the plan's print data
// (the example of the PRD: a buyer, 01-11-2026 to 06-11-2026, bought 51,360, paid on the spot 21,360, closing
// 30,000 بنام): one tight block for each date with an entry, oldest first, the lines of goods as "آم 20 ×
// 1,000 = 20,000", commission and لاگا on one line, the day's bill, what was paid on the spot, each entry of
// the day, the balance at the end of the day with its side, then the totals of the period, the closing balance
// and its amount in words. Nothing is worked out on the page: the test reads what the data says.

import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StatementPrintDocument } from '../../src/features/party_ledgers/print/StatementPrintDocument';
import type { StatementPrintData } from '../../src/bindings/StatementPrintData';
import { LANGUAGES, type Language } from '../../src/i18n';
import { statementExample, statementFrame, statementThirtyDays } from '../helpers/statement-print';
import { printFirmName } from '../helpers/print-frame-real';
import { message, renderIn, visible } from '../helpers/render';

const text = (element: Element | null | undefined) => visible(element?.textContent).trim();

function draw(data: StatementPrintData) {
  return renderIn(
    data.language,
    <StatementPrintDocument data={data} frame={statementFrame(data)} />,
  );
}

/** The rows of one block of the table, as the words of their cells, and the block itself. */
function blocks(container: HTMLElement) {
  return Array.from(container.querySelectorAll('tbody.print-statement-day'));
}

describe.each(LANGUAGES)(
  'the statement on paper, the example of the PRD (%s)',
  (language: Language) => {
    const data = statementExample(language);
    const side = (id: 'glossary-banaam' | 'glossary-jama') => message(language, id);

    it('has the header of the app, the party, and the period in its facts', () => {
      const { container } = draw(data);
      const page = text(container);
      expect(page).toContain(data.party.name);
      expect(page).toContain(printFirmName(language));
      // The facts of the header: the party and the two dates of the period.
      expect(page).toContain('01-11-2026');
      expect(page).toContain('06-11-2026');
      // The party block: the role word, the short code and the phone.
      expect(page).toContain(message(language, 'print-party-buyer'));
      expect(page).toContain('0300-1234567');
    });

    it('opens with the previous balance (0, no side), then one block for each date, oldest first', () => {
      const { container } = draw(data);
      const opening = container.querySelector('tbody.print-statement-opening');
      expect(text(opening)).toContain(message(language, 'print-statement-opening'));
      expect(text(opening)).toContain('0');
      // At 0 there is no side word.
      expect(text(opening)).not.toContain(side('glossary-banaam'));
      const days = blocks(container);
      expect(days).toHaveLength(2);
      expect(text(days[0]?.querySelector('.print-statement-date'))).toBe('05-11-2026');
      expect(text(days[1]?.querySelector('.print-statement-date'))).toBe('06-11-2026');
    });

    it('writes each line of goods as "item qty × rate = amount", and commission and لاگا on one line', () => {
      const { container } = draw(data);
      const [first] = blocks(container);
      const lines = Array.from(first?.querySelectorAll('.print-statement-line') ?? []).map(text);
      const mango = language === 'ur' ? 'آم' : 'Mango';
      const apple = language === 'ur' ? 'سیب' : 'Apple';
      expect(lines[0]).toBe(`${mango} 20 × 1,000 = 20,000`);
      // The × and the = are written in the number style (Inter, the size of the digits), not in the font of the
      // words: Gulzar draws them tiny. Each is its own isolated number element, like the figures.
      const firstLine = first?.querySelector('.print-statement-line');
      expect(Array.from(firstLine?.querySelectorAll('bdi.number-run') ?? []).map(text)).toEqual([
        '20',
        '×',
        '1,000',
        '=',
        '20,000',
      ]);
      expect(lines[1]).toBe(`${apple} 20 × 400 = 8,000`);
      // One line for the two charges: each label with its amount.
      expect(lines[2]).toBe(
        `${message(language, 'print-commission')} 1,680${message(language, 'print-laga')} 320`,
      );
    });

    it('gives the day’s bill, what was paid on the spot, and the balance at the end of the day with its side', () => {
      const { container } = draw(data);
      const [first, second] = blocks(container);
      expect(text(first?.querySelector('.print-statement-bill'))).toContain('30,000');
      expect(text(first?.querySelector('.print-statement-bill'))).toContain(
        message(language, 'print-statement-bill-buyer'),
      );
      // Nothing paid on the spot the first day: the row is not there at all.
      expect(text(first)).not.toContain(message(language, 'print-statement-paid-on-spot'));
      expect(text(first?.querySelector('.print-statement-end'))).toContain('30,000');
      expect(text(first?.querySelector('.print-statement-end'))).toContain(side('glossary-banaam'));
      expect(text(second?.querySelector('.print-statement-bill'))).toContain('21,360');
      expect(text(second)).toContain(message(language, 'print-statement-paid-on-spot'));
      expect(text(second?.querySelector('.print-statement-end'))).toContain('30,000');
    });

    it('ends with the totals of the period, the closing balance with its side, and the amount in words', () => {
      const { container } = draw(data);
      const totals = text(container.querySelector('tbody.print-statement-totals'));
      expect(totals).toContain(`${message(language, 'print-statement-total-bought')}51,360`);
      expect(totals).toContain('3,360');
      expect(totals).toContain(`${message(language, 'print-statement-total-paid-on-spot')}21,360`);
      const closing = text(container.querySelector('.print-statement-closing'));
      expect(closing).toContain(message(language, 'print-statement-total-closing'));
      expect(closing).toContain('30,000');
      expect(closing).toContain(side('glossary-banaam'));
      expect(text(container.querySelector('.print-words'))).toContain(data.words?.text ?? 'x');
    });

    it('is compact: no empty row, no date without an entry, the heading row is a header that repeats', () => {
      const { container } = draw(data);
      for (const row of Array.from(container.querySelectorAll('tbody tr'))) {
        expect(text(row)).not.toBe('');
      }
      expect(blocks(container)).toHaveLength(data.days.length);
      expect(container.querySelectorAll('thead').length).toBe(1);
      // The date is one cell that covers its whole block, not a cell in each row.
      for (const block of blocks(container)) {
        expect(block.querySelectorAll('.print-statement-date')).toHaveLength(1);
        expect(Number(block.querySelector('.print-statement-date')?.getAttribute('rowspan'))).toBe(
          block.querySelectorAll('tr').length,
        );
      }
    });

    it('centres every figure under its heading, and uses the digits 0 to 9 only', () => {
      const { container } = draw(data);
      const heads = Array.from(container.querySelectorAll('thead th'));
      // The date, the amount and the balance headings are number columns.
      expect(heads.filter((head) => head.classList.contains('print-num'))).toHaveLength(3);
      for (const cell of Array.from(container.querySelectorAll('td.print-num'))) {
        expect(cell.querySelector('.type-number')).not.toBeNull();
      }
      expect(text(container)).not.toMatch(/[٠-٩۰-۹]/);
    });

    it('a seller’s day has his own words: the net sale of the day instead of the bill', () => {
      const seller: StatementPrintData = {
        ...data,
        kind: 'seller',
        days: data.days.map((day) => ({
          ...day,
          bill: day.bill === null ? null : { ...day.bill, label_id: 'print-statement-bill-seller' },
        })),
      };
      const { container } = draw(seller);
      expect(text(blocks(container)[0]?.querySelector('.print-statement-bill'))).toContain(
        message(language, 'print-statement-bill-seller'),
      );
      expect(text(container)).not.toContain(message(language, 'print-statement-bill-buyer'));
      // What he was given in cash with his lots is the نقد, not "paid on the spot": the same figure, his own word.
      const second = blocks(container)[1];
      expect(text(second)).toContain(message(language, 'print-cash-paid'));
      expect(text(second)).not.toContain(message(language, 'print-statement-paid-on-spot'));
    });

    it('a day with a receipt and no sale line has no bill row, and still its date, its entry and its balance', () => {
      const day = data.days[0];
      if (day === undefined) throw new Error('the double has a day');
      const receiptOnly: StatementPrintData = {
        ...data,
        days: [
          {
            ...day,
            items: [],
            charges: [],
            bill: null,
            paid_on_spot: null,
            entries: [{ label_id: 'print-statement-entry-receipt', amount: '10,000', note: null }],
          },
        ],
      };
      const { container } = draw(receiptOnly);
      const [block] = blocks(container);
      expect(block?.querySelector('.print-statement-bill')).toBeNull();
      expect(text(block)).toContain(message(language, 'print-statement-entry-receipt'));
      expect(text(block)).toContain('10,000');
      expect(text(block?.querySelector('.print-statement-end'))).toContain(day.closing.amount);
      expect(Number(block?.querySelector('.print-statement-date')?.getAttribute('rowspan'))).toBe(
        block?.querySelectorAll('tr').length,
      );
    });

    it('the labels the backend sends for the other entries and the seller totals are all words, never ids', () => {
      const day = data.days[0];
      if (day === undefined) throw new Error('the double has a day');
      const ids = [
        'print-statement-entry-other-debit',
        'print-statement-entry-other-credit',
        'print-statement-entry-advance',
      ];
      const full: StatementPrintData = {
        ...data,
        kind: 'seller',
        days: [
          { ...day, entries: ids.map((id) => ({ label_id: id, amount: '1,000', note: null })) },
        ],
        totals: [
          'print-statement-total-sold',
          'print-statement-total-deductions',
          'print-statement-total-cash',
          'print-statement-total-advanced',
          'print-statement-total-other-debit',
          'print-statement-total-other-credit',
        ].map((id) => ({ label_id: id, amount: '2,000', role: 'line' as const })),
      };
      const { container } = draw(full);
      const page = text(container);
      for (const id of [...ids, ...full.totals.map((total) => total.label_id)]) {
        expect(page).toContain(message(language, id));
      }
      expect(page).not.toContain('print-statement-');
    });

    it('an item sold by weight shows its kilograms, a quality word follows the item, a note follows its entry', () => {
      const day = data.days[0];
      if (day === undefined) throw new Error('the double has a day');
      const changed: StatementPrintData = {
        ...data,
        days: [
          {
            ...day,
            items: [
              {
                name: 'X',
                grade: 'first',
                qty: '10',
                weight_kg: '12.5',
                rate: '100',
                amount: '1,235',
              },
            ],
            entries: [
              {
                label_id: 'print-statement-entry-receipt',
                amount: '5,000',
                note: 'Aslam',
              },
            ],
          },
        ],
      };
      const { container } = draw(changed);
      const line = text(container.querySelector('.print-statement-line'));
      expect(line).toContain('12.5');
      expect(
        Array.from(
          container.querySelector('.print-statement-line')?.querySelectorAll('bdi.number-run') ??
            [],
        ).map(text),
      ).toEqual(['12.5', '×', '100', '=', '1,235']);
      expect(line).toContain(message(language, 'glossary-darja-awwal'));
      expect(line).toContain('1,235');
      const entry = text(
        Array.from(container.querySelectorAll('tbody.print-statement-day tr')).find((row) =>
          text(row).includes('5,000'),
        ),
      );
      expect(entry).toContain(message(language, 'print-statement-entry-receipt'));
      expect(entry).toContain('Aslam');
    });
  },
);

describe.each(LANGUAGES)('a statement of thirty days (%s)', (language: Language) => {
  const data = statementThirtyDays(language);

  it('has one block for each of its dates, oldest first, and opens and closes with the balances of the data', () => {
    const { container } = draw(data);
    const dates = blocks(container).map((block) =>
      text(block.querySelector('.print-statement-date')),
    );
    expect(dates).toEqual(data.days.map((day) => day.date));
    // Oldest first.
    const asNumbers = dates.map((date) => date.split('-').reverse().join(''));
    expect([...asNumbers].sort()).toEqual(asNumbers);
    expect(text(container.querySelector('tbody.print-statement-opening'))).toContain('12,500');
    expect(text(container.querySelector('.print-statement-closing'))).toContain('252,216');
    expect(text(container.querySelector('.print-words'))).toContain(data.words?.text ?? 'x');
    // The quality words of the data are drawn after the item.
    expect(text(container)).toContain(message(language, 'glossary-darja-awwal'));
  });

  it('draws every figure of every day as the data wrote it', () => {
    const { container } = draw(data);
    for (const day of data.days) {
      const block = blocks(container).find(
        (candidate) => text(candidate.querySelector('.print-statement-date')) === day.date,
      );
      if (day.bill !== null) {
        expect(text(block?.querySelector('.print-statement-bill'))).toContain(day.bill.amount);
      }
      expect(text(block?.querySelector('.print-statement-end'))).toContain(day.closing.amount);
      for (const entry of day.entries) expect(text(block)).toContain(entry.amount);
      if (day.paid_on_spot !== null) expect(text(block)).toContain(day.paid_on_spot);
    }
  });

  it('takes the paper of the data: A5 for a small sheet', () => {
    const { container } = renderIn(
      language,
      <StatementPrintDocument data={{ ...data, paper: 'a5' }} frame={statementFrame(data)} />,
    );
    expect(container.querySelector('.print-doc')?.getAttribute('data-paper')).toBe('a5');
    expect(screen.getAllByRole('columnheader')).toHaveLength(4);
    expect(within(container).getAllByRole('table')).toHaveLength(1);
  });
});
