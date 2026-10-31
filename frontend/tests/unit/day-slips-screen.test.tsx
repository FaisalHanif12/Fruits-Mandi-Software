// The buyer's and the seller's day slip and the day total sheet on the ONE report screen (plan 42,
// PRD v1.12, FR-19.12, R34), in the Urdu and in the English app, against a fake backend that answers
// with the backend's own fixtures of the worked day. The slip has its goods in a list (the row after an
// item with more than one line is an item total, set apart) and its labelled lines as a card beside
// them: every figure the backend sent, a balance with its side word, a payment left empty as a blank
// to write on. The day total sheet has the buyers' and the sellers' short tables under its lines. The
// screen adds up nothing.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportPage } from '../../src/bindings/ReportPage';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  DAY_SHEET,
  SLIP_BUYER_AKRAM,
  SLIP_BUYER_WEIGHED,
  SLIP_SELLER_RAHIM,
} from '../helpers/day-slips-real';
import { fakeGardens } from '../helpers/gardens';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
// The window of a test has no layout, so the step is the middle one.
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  fakePartners([]);
  fakeGardens();
});

afterEach(() => {
  leaveAppWindow();
});

function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('missing element');
  return found;
}

/**
 * The text of an element as the reader sees it: a figure and, after it, the side word of a balance (in
 * its own slot, so the two are joined by a space here).
 */
function readable(element: Element | null | undefined): string {
  if (element === null || element === undefined) return '';
  const full = visible(element.textContent);
  const word = visible(element.querySelector('.cell-unit, .slip-side')?.textContent);
  return word !== '' && full.endsWith(word)
    ? `${full.slice(0, -word.length).trim()} ${word}`
    : full;
}

/** The labelled lines of the slip card: [label, figure and side word]. */
function cardLines(): [string, string][] {
  const dl = document.querySelector('section dl.flex-col');
  return Array.from(dl?.children ?? []).map((line) => [
    visible(line.querySelector('dt')?.textContent),
    readable(line.querySelector('dd')),
  ]);
}

const SIDE = {
  en: { dr: 'Debit', cr: 'Credit' },
  ur: { dr: 'بنام', cr: 'جمع' },
} as const;

describe.each(LANGUAGES)('the day slip on the report screen (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const side = SIDE[language];

  async function open(page: ReportPage) {
    fakeReports({ pages: { R35: page } });
    await renderApp('/reports/R35');
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name).startsWith(m('report-r35-title')),
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="0"]')).not.toBeNull();
    }, SLOW_MS);
  }

  it('shows the buyer’s goods in a list with five columns and the lines as a card beside them', async () => {
    await open(SLIP_BUYER_AKRAM);
    const heads = Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map((head) =>
      head.getAttribute('col-id'),
    );
    expect(heads).toEqual(['item', 'packing', 'qty', 'rate', 'amount']);
    const row = must(document.querySelector('.ag-row[row-id="0"]'));
    expect(visible(row.querySelector('[col-id="qty"]')?.textContent)).toBe('20');
    expect(visible(row.querySelector('[col-id="rate"]')?.textContent)).toBe('600');
    expect(visible(row.querySelector('[col-id="amount"]')?.textContent)).toBe('12,000');
    expect(cardLines()).toEqual([
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
    // The card is the one place the slip's lines are: no totals row under the list.
    expect(document.querySelector('.ag-row-pinned')).toBeNull();
  });

  it('shows the kilos of a weighed line in a column of their own, with the stored amount (PRD A73)', async () => {
    await open(SLIP_BUYER_WEIGHED);
    const heads = Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map((head) =>
      head.getAttribute('col-id'),
    );
    expect(heads).toEqual(['item', 'packing', 'qty', 'rate', 'amount', 'weight']);
    const row = must(document.querySelector('.ag-row[row-id="0"]'));
    const cellOfRow = (col: string) => visible(row.querySelector(`[col-id="${col}"]`)?.textContent);
    // 2 نگ, 12.345 kg at 100 a kilo: the amount is the stored 1,235, never qty times rate.
    expect(cellOfRow('qty')).toBe('2');
    expect(cellOfRow('rate')).toBe('100');
    expect(cellOfRow('amount')).toBe('1,235');
    expect(cellOfRow('weight')).toBe('12.345');
    expect(cardLines().slice(0, 4)).toEqual([
      [m('report-r35-goods'), '1,235'],
      [m('report-r35-commission'), '74'],
      [m('report-r35-laga'), '16'],
      [m('report-r35-today-total'), '1,325'],
    ]);
  });

  it('shows a payment left empty as a line to write on, not as a zero', async () => {
    await open({
      ...SLIP_BUYER_AKRAM,
      summary: SLIP_BUYER_AKRAM.summary.map((line) =>
        line.label_id === 'report-r35-paid-today' ? { ...line, cell: { type: 'empty' } } : line,
      ),
    });
    const paid = must(
      Array.from(document.querySelectorAll('section dl.flex-col > div')).find(
        (line) => visible(line.querySelector('dt')?.textContent) === m('report-r35-paid-today'),
      ),
    );
    expect(paid.querySelector('.slip-write-line')).not.toBeNull();
    expect(visible(paid.querySelector('dd')?.textContent)).toBe('');
  });

  it('shows the seller’s two lots, the item total row set apart, and his lines with their sides', async () => {
    await open(SLIP_SELLER_RAHIM);
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="2"]')).not.toBeNull();
    }, SLOW_MS);
    const total = must(document.querySelector('.ag-row[row-id="2"]'));
    expect(total.className).toContain('row-group');
    expect(visible(total.querySelector('[col-id="item"]')?.textContent)).toBe(
      language === 'ur' ? 'کیلا کل' : 'Banana total',
    );
    expect(visible(total.querySelector('[col-id="qty"]')?.textContent)).toBe('50');
    expect(visible(total.querySelector('[col-id="amount"]')?.textContent)).toBe('29,400');
    // The two lines of the item are not set apart.
    expect(document.querySelector('.ag-row[row-id="0"]')?.className).not.toContain('row-group');
    expect(cardLines()).toEqual([
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
});

describe.each(LANGUAGES)('the day total sheet R34 on the report screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const side = SIDE[language];

  async function open() {
    fakeReports({ pages: { R34: DAY_SHEET } });
    await renderApp('/reports/R34');
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name).startsWith(m('report-r34-title')),
    });
    // Both short tables are drawn with their head and their totals row.
    for (const id of ['report-r34-table-buyers', 'report-r34-table-sellers']) {
      const table = await screen.findByRole('group', { name: m(id) }, SLOW_MS);
      await waitFor(() => {
        expect(table.querySelector('.ag-header-cell[col-id]')).not.toBeNull();
        expect(table.querySelector('.ag-row-pinned')).not.toBeNull();
      }, SLOW_MS);
    }
  }

  it('has the buyers’ and the sellers’ short table under the lines of the day, each with its title', async () => {
    await open();
    const buyers = screen.getByRole('group', { name: m('report-r34-table-buyers') });
    const sellers = screen.getByRole('group', { name: m('report-r34-table-sellers') });
    expect(
      Array.from(buyers.querySelectorAll('.ag-header-cell[col-id]')).map((head) =>
        head.getAttribute('col-id'),
      ),
    ).toEqual(['code', 'buyer', 'previous', 'today_total', 'paid', 'new_balance']);
    expect(
      Array.from(sellers.querySelectorAll('.ag-header-cell[col-id]')).map((head) =>
        head.getAttribute('col-id'),
      ),
    ).toEqual(['code', 'seller', 'previous', 'net_sale', 'paid', 'new_balance']);
    // A single day: the titles do not carry a date.
    expect(
      screen.getByRole('heading', { level: 2, name: m('report-r34-table-buyers') }),
    ).toBeVisible();
  });

  it('shows each buyer’s day and the totals row 0, 44,364, 22,800, 21,564', async () => {
    await open();
    const buyers = screen.getByRole('group', { name: m('report-r34-table-buyers') });
    const keys = Array.from(buyers.querySelectorAll('.ag-row:not(.ag-row-pinned)')).map((row) =>
      row.getAttribute('row-id'),
    );
    expect(keys).toHaveLength(3);
    // The buyers are in the order of their Urdu names, as the backend sent them: ارشد, اکرم, شاہد.
    const rowOf = (index: number, col: string) =>
      readable(buyers.querySelector(`.ag-row[row-id="${String(keys[index])}"] [col-id="${col}"]`));
    expect([0, 1, 2].map((index) => rowOf(index, 'today_total'))).toEqual([
      '18,684',
      '12,880',
      '12,800',
    ]);
    expect([0, 1, 2].map((index) => rowOf(index, 'paid'))).toEqual(['0', '10,000', '12,800']);
    expect([0, 1, 2].map((index) => rowOf(index, 'new_balance'))).toEqual([
      `18,684 ${side.dr}`,
      `2,880 ${side.dr}`,
      '0',
    ]);
    const total = must(buyers.querySelector('.ag-row-pinned'));
    expect(readable(total.querySelector('[col-id="today_total"]'))).toBe('44,364');
    expect(readable(total.querySelector('[col-id="paid"]'))).toBe('22,800');
    expect(readable(total.querySelector('[col-id="new_balance"]'))).toBe(`21,564 ${side.dr}`);
    // The word for the total stands under the party, not under the narrow code.
    expect(visible(total.querySelector('[col-id="buyer"]')?.textContent)).toBe(m('report-total'));
  });

  it('shows each seller’s net sale of the day with his side, and the totals row', async () => {
    await open();
    const sellers = screen.getByRole('group', { name: m('report-r34-table-sellers') });
    const total = must(sellers.querySelector('.ag-row-pinned'));
    expect(readable(total.querySelector('[col-id="previous"]'))).toBe(`15,000 ${side.dr}`);
    expect(readable(total.querySelector('[col-id="net_sale"]'))).toBe(`34,476 ${side.cr}`);
    expect(readable(total.querySelector('[col-id="paid"]'))).toBe('9,316');
    expect(readable(total.querySelector('[col-id="new_balance"]'))).toBe(`10,160 ${side.cr}`);
  });

  it('keeps the commission on its own lines in the main list, the total after the three', async () => {
    await open();
    const labels = Array.from(document.querySelectorAll('.ag-row [col-id="item"]'))
      .map((cell) => visible(cell.textContent))
      .filter((text) => text !== '');
    expect(labels.slice(0, 6)).toEqual([
      m('report-r34-bikri'),
      m('report-r34-qty'),
      m('report-r34-commission-seller'),
      m('report-r34-commission-buyer'),
      m('report-r34-commission-laga'),
      m('report-r34-commission'),
    ]);
  });
});
