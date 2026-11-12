// The statement print on the backend's OWN answers (plan 74, PRD FR-12.47; backend/crates/reports/tests/
// fixtures, copied, never edited): the buyer of the PRD example (bought 51,360, paid on the spot 21,360,
// closing 30,000 بنام), a buyer of five dates in English and in Urdu, a seller (an advance, two lots, his
// deductions, cash and a payment) and a period with no entry. The page lays out what the data says: every
// label the backend sends is a word in the print language (never an id), a day with no sale line has no bill
// row, a seller's cash row says نقد, and nothing is worked out on the page.

import { describe, expect, it } from 'vitest';
import { StatementPrintDocument } from '../../src/features/party_ledgers/print/StatementPrintDocument';
import type { StatementPrintData } from '../../src/bindings/StatementPrintData';
import { hasNothingToPrint } from '../../src/features/party_ledgers/print/statement-empty';
import {
  STATEMENT_REAL,
  statementBalanceOnly,
  statementEmpty,
  statementFrame,
} from '../helpers/statement-print';
import { message, renderIn, visible } from '../helpers/render';

const text = (element: Element | null | undefined) => visible(element?.textContent).trim();

function draw(data: StatementPrintData) {
  return renderIn(
    data.language,
    <StatementPrintDocument data={data} frame={statementFrame(data)} />,
  );
}

const blocks = (container: HTMLElement) =>
  Array.from(container.querySelectorAll('tbody.print-statement-day'));

/** An id the backend sent that was drawn as it is: "print-..." with dashes, which no word has. */
const RAW_ID = /\bprint-[a-z0-9]+(-[a-z0-9]+)+/;

describe('the PRD example, the buyer of 01-11-2026 to 06-11-2026', () => {
  const { data } = STATEMENT_REAL.example;

  it('draws the two dates oldest first with the figures of the backend, and the closing in words', () => {
    const { container } = draw(data);
    const days = blocks(container);
    expect(days.map((day) => text(day.querySelector('.print-statement-date')))).toEqual([
      '05-11-2026',
      '06-11-2026',
    ]);
    const [first, second] = days;
    expect(text(first?.querySelector('.print-statement-line'))).toBe('Banana 30 × 1,000 = 30,000');
    expect(text(second?.querySelector('.print-statement-line'))).toBe('Banana 20 × 1,068 = 21,360');
    // No charges on these lines: no empty row for them.
    expect(container.querySelector('.print-statement-charges')).toBeNull();
    // Nothing paid on the spot the first day, 21,360 the second.
    expect(text(first)).not.toContain(message('en', 'print-statement-paid-on-spot'));
    expect(text(second)).toContain(`${message('en', 'print-statement-paid-on-spot')}21,360`);
    const totals = text(container.querySelector('tbody.print-statement-totals'));
    expect(totals).toContain(`${message('en', 'print-statement-total-bought')}51,360`);
    expect(totals).toContain(`${message('en', 'print-statement-total-paid-on-spot')}21,360`);
    const closing = text(container.querySelector('.print-statement-closing'));
    expect(closing).toContain('30,000');
    expect(closing).toContain(message('en', 'glossary-banaam'));
    expect(text(container.querySelector('.print-words'))).toContain('Thirty thousand rupees only');
    expect(text(container)).not.toMatch(RAW_ID);
  });
});

describe('a buyer of five dates, in both languages', () => {
  it.each([
    ['en', STATEMENT_REAL.akramEn.data],
    ['ur', STATEMENT_REAL.akramUr.data],
  ] as const)(
    '(%s) every label is a word, the digits are 0 to 9, the closing is as the data says',
    (language, data) => {
      const { container } = draw(data);
      expect(blocks(container)).toHaveLength(data.days.length);
      const page = text(container);
      expect(page).not.toMatch(RAW_ID);
      expect(page).not.toMatch(/[٠-٩۰-۹]/);
      expect(page).toContain(data.party.name);
      for (const day of data.days) {
        const block = blocks(container).find(
          (candidate) => text(candidate.querySelector('.print-statement-date')) === day.date,
        );
        expect(text(block?.querySelector('.print-statement-end'))).toContain(day.closing.amount);
        for (const entry of day.entries) {
          expect(text(block)).toContain(message(language, entry.label_id));
          expect(text(block)).toContain(entry.amount);
        }
        for (const charge of day.charges) {
          expect(text(block)).toContain(`${message(language, charge.label_id)} ${charge.amount}`);
        }
      }
      expect(text(container.querySelector('.print-statement-closing'))).toContain(
        data.closing.amount,
      );
    },
  );
});

describe('a seller: an advance, two lots with deductions, cash and a payment', () => {
  const { data } = STATEMENT_REAL.rahim;

  it('a day with only his advance has no bill and no goods, its entry and its balance', () => {
    const { container } = draw(data);
    const [advance] = blocks(container);
    expect(advance?.querySelector('.print-statement-bill')).toBeNull();
    expect(advance?.querySelector('.print-statement-line')).toBeNull();
    expect(text(advance)).toContain(`${message('en', 'print-statement-entry-advance')}15,000`);
    expect(text(advance?.querySelector('.print-statement-end'))).toContain('15,000');
    expect(text(advance?.querySelector('.print-statement-end'))).toContain(
      message('en', 'glossary-banaam'),
    );
  });

  it('the lot day: both lines, the deductions on one line, his net sale, his cash as نقد, the payment, balance 0', () => {
    const { container } = draw(data);
    const lot = blocks(container)[1];
    const lines = Array.from(lot?.querySelectorAll('.print-statement-line') ?? []).map(text);
    expect(lines[0]).toBe('Banana 20 × 600 = 12,000');
    expect(lines[1]).toBe('Banana 30 × 580 = 17,400');
    const charges: [string, string][] = [
      ['print-commission', '1,764'],
      ['print-freight', '3,000'],
      ['print-labour', '250'],
      ['print-munshiana', '50'],
      ['print-phone-charge', '20'],
    ];
    for (const [id, amount] of charges) {
      expect(lines[2]).toContain(`${message('en', id)} ${amount}`);
    }
    expect(text(lot?.querySelector('.print-statement-bill'))).toContain(
      `${message('en', 'print-statement-bill-seller')}24,316`,
    );
    // The cash given with his lots is the نقد, not "paid on the spot".
    expect(text(lot)).toContain(`${message('en', 'print-cash-paid')}5,000`);
    expect(text(lot)).not.toContain(message('en', 'print-statement-paid-on-spot'));
    expect(text(lot)).toContain(`${message('en', 'print-statement-entry-payment')}4,316`);
    // At 0 there is no side word.
    expect(text(lot?.querySelector('.print-statement-end'))).not.toContain(
      message('en', 'glossary-banaam'),
    );
  });

  it('draws his totals in the order of the data, with no amount in words for a closing of 0', () => {
    const { container } = draw(data);
    const totals = text(container.querySelector('tbody.print-statement-totals'));
    let from = 0;
    for (const total of data.totals) {
      const found = totals.indexOf(`${message('en', total.label_id)}${total.amount}`, from);
      expect(found).toBeGreaterThanOrEqual(from);
      from = found;
    }
    expect(container.querySelector('.print-words')).toBeNull();
    expect(text(container)).not.toMatch(RAW_ID);
  });
});

describe('a period with no entry, with a balance carried through it (Umar, 13-11-2026)', () => {
  const real = STATEMENT_REAL.empty.data;

  it('the backend answers days: [] with the balance as it was, and it is something to print', () => {
    expect(real.days).toEqual([]);
    // He still owes what he owed before the period: the opening and the closing are the same.
    expect(real.opening).toEqual(real.closing);
    expect(real.closing).toEqual({ amount: '1,000', side: 'debit' });
    expect(hasNothingToPrint(real)).toBe(false);
  });

  it.each([
    ['en', statementBalanceOnly('en')],
    ['ur', statementBalanceOnly('ur')],
  ] as const)(
    '(%s) draws one line: the opening balance with its side, the closing equal to it, its amount in words',
    (language, data) => {
      const { container } = draw(data);
      expect(blocks(container)).toHaveLength(0);
      // No totals of the period (they would all be 0): only the opening, the closing and the words.
      expect(
        container.querySelector('tbody.print-statement-totals')?.querySelectorAll('tr'),
      ).toHaveLength(1);
      const opening = text(container.querySelector('tbody.print-statement-opening'));
      expect(opening).toContain(message(language, 'print-statement-opening'));
      expect(opening).toContain('1,000');
      expect(opening).toContain(message(language, 'glossary-banaam'));
      const closing = text(container.querySelector('.print-statement-closing'));
      expect(closing).toContain(message(language, 'print-statement-total-closing'));
      expect(closing).toContain('1,000');
      expect(closing).toContain(message(language, 'glossary-banaam'));
      expect(text(container.querySelector('.print-words'))).toContain(data.words?.text ?? 'x');
      expect(text(container)).not.toMatch(RAW_ID);
      expect(text(container)).not.toMatch(/[٠-٩۰-۹]/);
    },
  );

  it('with no entry and no balance there is nothing to print', () => {
    expect(hasNothingToPrint(statementEmpty('en'))).toBe(true);
    expect(hasNothingToPrint(statementEmpty('ur'))).toBe(true);
  });
});
