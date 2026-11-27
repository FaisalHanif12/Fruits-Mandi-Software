// Test helper: the month and the day answers of the partner statement (PRD FR-17.12, plan 76 point 11)
// made from a list of statement lines, the way the backend makes them from posted records. The sums here are
// a test double's, in whole paisa numbers; the screens only show what comes back and never add a figure
// (CLAUDE.md rule 1). The lines can be the books' own (partner-money-books.ts) or a REAL answer of the
// backend (tests/fixtures/*.json), so the old line tests keep their real numbers on the new page.

import { screen, waitFor } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import type { PartnerStatementDay } from '../../src/bindings/PartnerStatementDay';
import type { PartnerStatementMonth } from '../../src/bindings/PartnerStatementMonth';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { StatementColumns } from '../../src/bindings/StatementColumns';
import type { StatementDetail } from '../../src/bindings/StatementDetail';
import type { StatementLine } from '../../src/bindings/StatementLine';
import type { StatementMonthDay } from '../../src/bindings/StatementMonthDay';
import * as partnersApi from '../../src/features/partners/api';
import { paisaForTest, plainPaisaForTest } from './paisa';
import { visible } from './render';

type ColumnKey = Exclude<keyof StatementColumns, 'net_paisa'>;

/** The column a line lands in; the notes have no amount and land nowhere. */
const COLUMN_OF: Record<StatementDetail, ColumnKey | null> = {
  income_share: 'commission_paisa',
  expense_share: 'expense_paisa',
  capital_opening: 'capital_paisa',
  capital_in: 'capital_paisa',
  capital_out: 'capital_paisa',
  month_close: null,
  taken: 'taken_returned_paisa',
  returned: 'taken_returned_paisa',
  drawings_opening: 'taken_returned_paisa',
  trip_paid: 'gardens_trips_paisa',
  trip_credit: 'gardens_trips_paisa',
  trip_write_off: null,
  trip_closed: null,
  garden_paid: 'gardens_trips_paisa',
  garden_closed: 'gardens_trips_paisa',
  fruit_taken: 'fruit_paisa',
  khata_paid: 'taken_returned_paisa',
  khata_received: 'taken_returned_paisa',
  lent_given: 'taken_returned_paisa',
  lent_returned: 'taken_returned_paisa',
  linked_other: 'taken_returned_paisa',
  // An advance to a seller goes out of his total by his part and comes back when it is settled (plan 78 B4).
  advance_out: 'advances_paisa',
  advance_back: 'advances_paisa',
  // His part comes back while the seller's own credit covers the advance (FR-17.13 v1.61, plan 100 item 12).
  advance_covered: 'advances_paisa',
  lent_share_note: null,
};

const COLUMN_KEYS: readonly ColumnKey[] = [
  'commission_paisa',
  'expense_paisa',
  'gardens_trips_paisa',
  'fruit_paisa',
  'taken_returned_paisa',
  'capital_paisa',
  'advances_paisa',
];

type Sums = Record<ColumnKey, number>;

const emptySums = (): Sums => ({
  commission_paisa: 0,
  expense_paisa: 0,
  gardens_trips_paisa: 0,
  fruit_paisa: 0,
  taken_returned_paisa: 0,
  capital_paisa: 0,
  advances_paisa: 0,
});

function columnsOf(sums: Sums): StatementColumns {
  const net = COLUMN_KEYS.reduce((total, key) => total + sums[key], 0);
  return {
    commission_paisa: paisaForTest(sums.commission_paisa),
    expense_paisa: paisaForTest(sums.expense_paisa),
    gardens_trips_paisa: paisaForTest(sums.gardens_trips_paisa),
    fruit_paisa: paisaForTest(sums.fruit_paisa),
    taken_returned_paisa: paisaForTest(sums.taken_returned_paisa),
    capital_paisa: paisaForTest(sums.capital_paisa),
    advances_paisa: paisaForTest(sums.advances_paisa),
    net_paisa: paisaForTest(net),
  };
}

const monthOf = (date: string) => date.slice(0, 7);
const numberOf = (paisa: StatementLine['amount_paisa']) =>
  paisa === null ? 0 : plainPaisaForTest(paisa);

/** The month of the statement, `YYYY-MM`, made from the lines in date order. */
export function monthAnswerOf(
  partner: RecordRef,
  lines: readonly StatementLine[],
  asked: string | null,
  today: string,
): PartnerStatementMonth {
  const current = monthOf(today);
  const first = lines[0] === undefined ? current : monthOf(lines[0].entry_date);
  let month = asked ?? current;
  if (month > current) month = current;
  if (month < first) month = first;
  let opening = 0;
  const byDay = new Map<string, { sums: Sums; notes: boolean }>();
  const totals = emptySums();
  for (const line of lines) {
    const in_month = monthOf(line.entry_date) === month;
    const column = COLUMN_OF[line.detail];
    const amount = numberOf(line.amount_paisa);
    if (monthOf(line.entry_date) < month) opening += amount;
    if (!in_month) continue;
    const day = byDay.get(line.entry_date) ?? { sums: emptySums(), notes: false };
    if (column === null) {
      day.notes = true;
    } else {
      day.sums[column] += amount;
      totals[column] += amount;
    }
    byDay.set(line.entry_date, day);
  }
  // The running total at the end of each day, and of the month.
  let after = opening;
  const days: StatementMonthDay[] = [...byDay.entries()]
    .sort(([a], [b]) => (a < b ? -1 : 1))
    .map(([date, day]) => {
      const columns = columnsOf(day.sums);
      after += plainPaisaForTest(columns.net_paisa);
      return { date, columns, running_paisa: paisaForTest(after), has_notes: day.notes };
    });
  return {
    partner: { ...partner, code: '' },
    month,
    first_month: first,
    current_month: current,
    opening_total_paisa: paisaForTest(opening),
    days,
    totals: columnsOf(totals),
    closing_total_paisa: paisaForTest(after),
  };
}

/** One day of the statement: its lines with the totals before and after. */
export function dayAnswerOf(
  partner: RecordRef,
  lines: readonly StatementLine[],
  date: string,
): PartnerStatementDay {
  const before = lines
    .filter((line) => line.entry_date < date)
    .reduce((total, line) => total + numberOf(line.amount_paisa), 0);
  const ofDay = lines.filter((line) => line.entry_date === date);
  const after = ofDay.reduce((total, line) => total + numberOf(line.amount_paisa), before);
  return {
    partner: { ...partner, code: '' },
    date,
    opening_total_paisa: paisaForTest(before),
    lines: [...ofDay],
    after_total_paisa: paisaForTest(after),
  };
}

/**
 * Makes the month and the day commands of the partners module answer from `lines` (by partner id the
 * caller gives). `today` is the day of the app, so the current month is the one it opens on.
 */
export function fakePartnerMonth(
  partner: RecordRef,
  lines: readonly StatementLine[],
  today = '2026-12-31',
) {
  vi.mocked(partnersApi.partnersStatementMonth).mockImplementation((input) =>
    Promise.resolve(
      monthAnswerOf(
        input.partner_id === partner.id ? partner : { ...partner, id: input.partner_id },
        input.partner_id === partner.id ? lines : [],
        input.month === null || input.month === '' ? null : input.month,
        today,
      ),
    ),
  );
  vi.mocked(partnersApi.partnersStatementDay).mockImplementation((input) =>
    Promise.resolve(
      dayAnswerOf(
        input.partner_id === partner.id ? partner : { ...partner, id: input.partner_id },
        input.partner_id === partner.id ? lines : [],
        input.date,
      ),
    ),
  );
}

/** One line of an opened day, as the screen draws it: the group, the sentence and the amount. */
export interface CardLine {
  heading: string;
  detail: string;
  amount: string;
}

/** Opens the days (`YYYY-MM-DD`) of the list by a click on their rows, and gives the lines of each. */
export async function openDays(
  user: ReturnType<typeof userEvent.setup>,
  dates: readonly string[],
): Promise<Map<string, CardLine[]>> {
  const found = new Map<string, CardLine[]>();
  for (const date of dates) {
    const shown = date.split('-').reverse().join('-');
    await user.click(await screen.findByRole('row', { name: (name) => name.includes(shown) }));
    const card = await waitFor(() => {
      const one = document.querySelector(`[data-partner-day="${date}"]`);
      if (!(one instanceof HTMLElement)) throw new Error(`the day ${date} is not open`);
      if (one.querySelector('li') === null) throw new Error(`the day ${date} has no lines yet`);
      return one;
    });
    found.set(
      date,
      Array.from(card.querySelectorAll('li')).map((item) => {
        // The group chip is left out when the sentence already says it, so the sentence is the last one.
        const words = item.firstElementChild;
        const parts = Array.from(words?.children ?? []);
        return {
          heading: parts.length > 1 ? visible(parts[0]?.textContent) : '',
          detail: visible(parts.at(-1)?.textContent),
          amount: visible(item.lastElementChild?.textContent),
        };
      }),
    );
  }
  return found;
}
