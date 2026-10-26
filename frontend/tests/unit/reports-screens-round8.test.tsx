// Stock (R10), the vouchers list (R15), the receipts and payments registers (R23, R25), the trip
// profit or loss (R33) and the day summary (R34) on the ONE report screen (round 8, PRD 19.2,
// FR-12.27, FR-19.2, FR-19.10, FR-19.13, E50), in the Urdu and in the English app, against a fake
// backend that answers with the backend's own fixtures. Each report asks for exactly what the filters
// hold and shows the backend's figures. The kind box of the vouchers list and the layout box of the
// trip report, the lines above R15 (one for each kind, never added up here), a result that is a loss
// (red, with its sign and the word "Loss"), the partner layout (his share in percent, his money and
// the cost still left), the day summary with no totals row, the print, a language switch in the middle
// of a layout and the keyboard-only path are each proved. The screen adds up nothing: every figure is
// one the backend sent. The layout step in a test with no layout is the middle one (normal), so the
// columns of the wide step are not drawn here; the screenshots show those.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import * as partnersApi from '../../src/features/partners/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { paisaForTest } from '../helpers/paisa';
import { printFirmName } from '../helpers/print-frame';
import { message, visible, paragraph } from '../helpers/render';
import {
  CATALOG,
  PRINT_R33_PARTNER_EN,
  PRINT_R34_EN,
  R10_FRUIT,
  R15_ALL,
  R15_ALL_REQUEST,
  R15_PAYMENTS_REQUEST,
  R23_DAY,
  R25_DAY,
  R33_ITEM,
  R33_ITEM_REQUEST,
  R33_MONTH,
  R33_PARTNER_REQUEST,
  R33_TRIP,
  R33_TRIP_REQUEST,
  R34_DAY,
  R34_FRUIT,
  fakeReports,
} from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
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
});

afterEach(() => {
  leaveAppWindow();
});

const KEY_CHIPS = new Set(['F2', 'F9', 'F10', 'Enter', 'Esc']);

/** Matches a name that is exactly `text`, or `text` followed by the key chip of its button. */
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  if (seen === text) return true;
  return seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim());
};

function cell(rowId: string, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function cellElement(rowId: string, colId: string): Element | null {
  return document.querySelector(`.ag-row[row-id="${rowId}"] [col-id="${colId}"]`);
}

function totalsCell(colId: string): string {
  const row = document.querySelector('.ag-row-pinned');
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function drawnRows(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map(
    (row) => row.getAttribute('row-id') ?? '',
  );
}

/** The columns drawn, by key, in the order of the grid. */
function drawnColumns(): string[] {
  return Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map(
    (header) => header.getAttribute('col-id') ?? '',
  );
}

/** The filters of a request the backend wrote, as the screen sends them (nulls left out). */
function sentLike(request: ReportRunInput, change: Record<string, unknown> = {}) {
  const {
    group_by: groupBy,
    status,
    voucher_type: voucherType,
    trip_view: tripView,
    partner_id: partnerId,
    garden_id: gardenId,
    garden_view: gardenView,
    bikri_id: bikriId,
    ...rest
  } = request.filters;
  const present = (value: unknown) => value !== undefined && value !== null;
  return {
    ...rest,
    partner_id: partnerId ?? null,
    garden_id: gardenId ?? null,
    bikri_id: bikriId ?? null,
    ...(present(gardenView) ? { garden_view: gardenView } : {}),
    ...(present(groupBy) ? { group_by: groupBy } : {}),
    ...(present(status) ? { status } : {}),
    ...(present(voucherType) ? { voucher_type: voucherType } : {}),
    ...(present(tripView) ? { trip_view: tripView } : {}),
    ...change,
  };
}

function lastRun() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];
}

function runCount() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.length;
}

describe.each(LANGUAGES)('the round 8 reports (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const other: Language = language === 'ur' ? 'en' : 'ur';

  async function openReport(id: string, options: Parameters<typeof fakeReports>[0] = {}) {
    const entry = CATALOG.find((candidate) => candidate.id === id);
    const fake = fakeReports(options);
    const view = await renderApp(`/reports/${id}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(entry?.title_id ?? 'report-r10-title')),
    });
    return { ...view, fake, user: userEvent.setup() };
  }

  async function choose(user: ReturnType<typeof userEvent.setup>, box: string, option: string) {
    await user.click(screen.getByRole('combobox', { name: m(box) }));
    await user.click(await screen.findByRole('option', { name: option }));
  }

  describe('R10 stock', () => {
    it('asks for what the filters hold and draws the lot with what came, what was sold and what is left', async () => {
      await openReport('R10');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(lastRun()).toMatchObject({ id: 'R10', offset: 0, limit: 100 });
      expect(cell('0', 'seller')).toContain('0010');
      expect(cell('0', 'item')).toBe(names('کیلا', 'Banana'));
      expect(cell('0', 'received')).toBe('50');
      expect(cell('0', 'sold')).toBe('20');
      // No fruit was taken from this lot (the fruit lot of E2E-42 is another test).
      expect(cell('0', 'fruit_taken')).toBe('0');
      expect(cell('0', 'balance')).toBe('30');
      expect(cell('0', 'days')).toBe('0');
      // The backend's totals under the figures. The packing and the write-offs wait for the wide step.
      expect(totalsCell('received')).toBe('50');
      expect(totalsCell('sold')).toBe('20');
      expect(totalsCell('balance')).toBe('30');
      expect(drawnColumns()).toEqual([
        'date',
        'seller',
        'item',
        'received',
        'sold',
        'fruit_taken',
        'balance',
        'days',
      ]);
    });

    it('draws the document, the packing and the write-offs at the wide step', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('wide');
      await openReport('R10');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(drawnColumns()).toEqual([
        'date',
        'doc',
        'seller',
        'item',
        'packing',
        'received',
        'sold',
        'written_off',
        'fruit_taken',
        'balance',
        'days',
      ]);
      expect(cell('0', 'doc')).toBe(
        m('report-doc-number', { kind: m('report-doc-bikri-first'), number: '1' }),
      );
    });

    it('has the days, a seller and an item to filter by', async () => {
      await openReport('R10');
      expect(screen.getByLabelText(m('report-filter-from'))).toBeInTheDocument();
      expect(screen.getByRole('combobox', { name: m('report-filter-seller') })).toBeInTheDocument();
      expect(screen.getByRole('combobox', { name: m('report-filter-item') })).toBeInTheDocument();
    });

    it('has a fruit column between the write-offs and the balance, and writes what the backend sent', async () => {
      expect(R10_FRUIT.columns.map((column) => column.key)).toEqual([
        'date',
        'doc',
        'seller',
        'item',
        'packing',
        'received',
        'sold',
        'written_off',
        'fruit_taken',
        'balance',
        'days',
      ]);
      await openReport('R10', { pages: { R10: R10_FRUIT } });
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      // 50 came, 45 were sold, 2 were taken as fruit: 3 are left (the backend works the balance out).
      expect(cell('0', 'received')).toBe('50');
      expect(cell('0', 'sold')).toBe('45');
      expect(cell('0', 'fruit_taken')).toBe('2');
      expect(cell('0', 'balance')).toBe('3');
      expect(totalsCell('fruit_taken')).toBe('2');
      // The heading is the word of the glossary, in either language.
      expect(
        document.querySelector('.ag-header-cell[col-id="fruit_taken"]')?.textContent,
      ).toContain(m('report-col-fruit-taken'));
    });

    it('opens the بکری behind a row with Enter', async () => {
      const { router, user } = await openReport('R10');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      const target = cellElement('0', 'seller');
      if (target === null) throw new Error('no seller cell');
      await user.click(target);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      }, SLOW_MS);
    });
  });

  describe('R15 the vouchers list', () => {
    it('with no kind picked, shows the money of each kind above the table and has no totals row', async () => {
      await openReport('R15');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1', '2']);
      }, SLOW_MS);
      expect(lastRun()?.filters).toEqual(sentLike(R15_ALL_REQUEST));
      expect(lastRun()?.filters).not.toHaveProperty('voucher_type');
      // One line for each kind, in the words of the kind, with the backend's figure.
      const lines = new Map(
        Array.from(document.querySelectorAll('dt')).map((term) => [
          visible(term.textContent),
          visible(term.nextElementSibling?.textContent),
        ]),
      );
      expect(lines.get(m('report-doc-advance'))).toBe('15,000');
      expect(lines.get(m('report-doc-payment'))).toBe('4,316');
      expect(lines.get(m('report-doc-receipt'))).toBe('10,000');
      // Nothing is added up: there is no total of money of different kinds.
      expect(document.querySelector('.ag-row-pinned')).toBeNull();
      expect(cell('0', 'doc')).toBe(
        m('report-doc-number', { kind: m('report-doc-advance'), number: '1' }),
      );
      expect(cell('0', 'amount')).toBe('15,000');
      expect(drawnColumns()).toEqual(['date', 'doc', 'account', 'amount', 'status']);
    });

    it('asks for one kind when it is picked, shows one total and no lines, and asks for all again', async () => {
      const { user } = await openReport('R15');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await choose(user, 'report-filter-voucher-type', m('report-doc-payment'));
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(sentLike(R15_PAYMENTS_REQUEST));
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(totalsCell('amount')).toBe('4,316');
      expect(document.querySelector('dl dt')).toBeNull();
      await choose(user, 'report-filter-voucher-type', m('report-voucher-type-all'));
      await waitFor(() => {
        expect(lastRun()?.filters).not.toHaveProperty('voucher_type');
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
    });

    it('offers every kind of voucher by its own word', async () => {
      const { user } = await openReport('R15');
      await user.click(screen.getByRole('combobox', { name: m('report-filter-voucher-type') }));
      for (const kind of ['receipt', 'payment', 'advance', 'shop-expense', 'general', 'khata-in']) {
        expect(
          await screen.findByRole('option', { name: m(`report-doc-${kind}`) }),
        ).toBeInTheDocument();
      }
      expect(
        screen.getByRole('option', { name: m('report-voucher-type-all') }),
      ).toBeInTheDocument();
    });

    it('lists a cancelled voucher with its status in words', async () => {
      const cancelled: ReportPage = {
        ...R15_ALL,
        rows: R15_ALL.rows.map((row, index) =>
          index === 1
            ? {
                ...row,
                cells: row.cells.map((one, at) =>
                  at === 5 ? { type: 'label' as const, id: 'report-status-cancelled' } : one,
                ),
              }
            : row,
        ),
      };
      await openReport('R15', { pages: { R15: cancelled } });
      await waitFor(() => {
        expect(cell('1', 'status')).toBe(m('report-status-cancelled'));
      }, SLOW_MS);
      expect(cell('0', 'status')).toBe('');
    });

    it('opens the voucher behind a row with Enter', async () => {
      const { router, user } = await openReport('R15');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      const target = cellElement('0', 'account');
      if (target === null) throw new Error('no account cell');
      await user.click(target);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/voucher');
      }, SLOW_MS);
    });
  });

  describe('R23 the receipts register and R25 the payments register', () => {
    it('R23 draws the receipt with its buyer and the backend’s total, and a collector box column', async () => {
      await openReport('R23');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(cell('0', 'account')).toContain('0020');
      expect(cell('0', 'amount')).toBe('10,000');
      expect(totalsCell('amount')).toBe('10,000');
      expect(drawnColumns()).toEqual(['date', 'doc', 'account', 'amount', 'collector', 'status']);
      expect(R23_DAY.columns.map((column) => column.key)).toContain('narration');
    });

    it('R25 draws the payment and has no mode, bank or cheque column', async () => {
      await openReport('R25');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(cell('0', 'account')).toContain('0010');
      expect(totalsCell('amount')).toBe('4,316');
      expect(drawnColumns()).toEqual(['date', 'doc', 'account', 'amount', 'status']);
      const keys = R25_DAY.columns.map((column) => column.key).join(' ');
      expect(keys).not.toMatch(/mode|bank|cheque/);
    });

    it.each(['R23', 'R25'])('%s has the days and the party to filter by', async (id) => {
      await openReport(id);
      expect(screen.getByLabelText(m('report-filter-from'))).toBeInTheDocument();
      expect(screen.getByRole('combobox', { name: m('report-filter-party') })).toBeInTheDocument();
    });
  });

  describe('R33 trip profit or loss', () => {
    it('asks by trip at first and draws the trip as a heading with its purchase under it', async () => {
      await openReport('R33');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
      expect(lastRun()?.filters).toEqual(sentLike(R33_TRIP_REQUEST));
      // The first row is the trip heading (a group row): its name and the sums of its purchases.
      expect(document.querySelector('.ag-row[row-id="0"]')).toHaveClass('row-group');
      expect(cell('0', 'trip')).toContain(names('سفر-1', 'TRP-1'));
      // The date inside the heading keeps its digit order in Urdu: it sits in its own isolate.
      const runs = Array.from(cellElement('0', 'trip')?.querySelectorAll('bdi[dir="ltr"]') ?? []);
      expect(runs.map((run) => run.textContent)).toContain('10-10-2026');
      expect(cell('1', 'purchase')).toBe(
        m('report-doc-number', { kind: m('report-doc-purchase'), number: '1' }),
      );
      expect(cell('1', 'item')).toBe(names('مالٹا', 'Orange'));
      expect(cell('1', 'bought_cost')).toContain('10,400');
      expect(cell('1', 'sales_money')).toContain('8,460');
      expect(cell('1', 'left_qty')).toBe('0');
    });

    it('shows a loss in red with its minus sign and the word, on the row and in the totals', async () => {
      await openReport('R33');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
      for (const row of ['0', '1']) {
        expect(cell(row, 'result')).toContain('-1,940');
        expect(cell(row, 'result')).toContain(m('report-result-loss'));
        expect(cellElement(row, 'result')).toHaveClass('cell-tone-due');
      }
      expect(totalsCell('result')).toContain('-1,940');
      expect(totalsCell('result')).toContain(m('report-result-loss'));
      expect(document.querySelector('.ag-row-pinned [col-id="result"]')).toHaveClass(
        'cell-tone-due',
      );
    });

    it('shows a profit with the word “Profit” and no red', async () => {
      const profit: ReportPage = {
        ...R33_ITEM,
        rows: R33_ITEM.rows.map((row) => ({
          ...row,
          cells: row.cells.map((one) =>
            one.type === 'money' && one === row.cells.at(-1)
              ? { type: 'money' as const, paisa: paisaForTest(66_000) }
              : one,
          ),
        })),
        totals: R33_ITEM.totals.map((one) =>
          one.type === 'money' && one === R33_ITEM.totals.at(-1)
            ? { type: 'money' as const, paisa: paisaForTest(66_000) }
            : one,
        ),
      };
      const { user } = await openReport('R33', { pages: { R33: profit } });
      await choose(user, 'report-filter-trip-view', m('report-filter-trip-by-item'));
      await waitFor(() => {
        expect(cell('0', 'result')).toContain('660');
      }, SLOW_MS);
      expect(cell('0', 'result')).toContain(m('report-result-profit'));
      expect(cellElement('0', 'result')).not.toHaveClass('cell-tone-due');
    });

    it('keeps the columns that fit and leaves the rest to the wider steps and the print', async () => {
      await openReport('R33');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      expect(drawnColumns()).toEqual([
        'trip',
        'purchase',
        'item',
        'bought_cost',
        'sold_qty',
        'sales_money',
        'left_qty',
        'result',
      ]);
    });

    it('has a layout box with the five layouts', async () => {
      const { user } = await openReport('R33');
      await user.click(screen.getByRole('combobox', { name: m('report-filter-trip-view') }));
      for (const view of ['trip', 'item', 'day', 'month', 'partner']) {
        expect(
          await screen.findByRole('option', { name: m(`report-filter-trip-by-${view}`) }),
        ).toBeInTheDocument();
      }
    });

    it('asks by partner when it is chosen and draws his share, his money and the cost still left', async () => {
      const { user } = await openReport('R33');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await choose(user, 'report-filter-trip-view', m('report-filter-trip-by-partner'));
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(sentLike(R33_PARTNER_REQUEST));
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1', '2', '3']);
      }, SLOW_MS);
      expect(drawnColumns()).toEqual([
        'trip',
        'partner',
        'share',
        'money_in_shop',
        'money_back',
        'net',
        'cost_left',
      ]);
      await waitFor(() => {
        expect(cell('1', 'net')).toContain('-776');
      }, SLOW_MS);
      expect(cell('1', 'partner')).toBe(names('الف', 'Partner A'));
      // The share is a percent from the backend, written with two decimals.
      expect(cell('1', 'share')).toBe('40.00%');
      expect(cell('1', 'money_in_shop')).toBe('4,160');
      expect(cell('1', 'money_back')).toBe('3,384');
      // His part so far is a loss, and the trip heading says what is still left to release.
      expect(cell('1', 'net')).toContain(m('report-result-loss'));
      expect(cellElement('1', 'net')).toHaveClass('cell-tone-due');
      expect(cell('0', 'cost_left')).toBe('0');
      expect(totalsCell('net')).toContain('-1,940');
      // The column is named "part so far", never "result".
      const headers = Array.from(document.querySelectorAll('.ag-header-cell[col-id="net"]'));
      expect(visible(headers[0]?.textContent).trim()).toBe(m('report-col-part-of-result'));
    });

    it('asks by item and by month and shows the same result in each', async () => {
      const { user } = await openReport('R33');
      await choose(user, 'report-filter-trip-view', m('report-filter-trip-by-item'));
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(sentLike(R33_ITEM_REQUEST));
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell('0', 'item')).toBe(names('مالٹا', 'Orange'));
      }, SLOW_MS);
      expect(totalsCell('result')).toContain('-1,940');
      await choose(user, 'report-filter-trip-view', m('report-filter-trip-by-month'));
      await waitFor(() => {
        expect(cell('0', 'period')).toBe(`${m('report-month-10')} 2026`);
      }, SLOW_MS);
      expect(totalsCell('result')).toContain('-1,940');
      expect(R33_MONTH.totals.at(-1)).toEqual(R33_ITEM.totals.at(-1));
    });

    it('keeps the layout through a language switch, words the loss in the other language, and asks nothing again', async () => {
      const { user } = await openReport('R33');
      await choose(user, 'report-filter-trip-view', m('report-filter-trip-by-partner'));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      const before = runCount();
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(cell('1', 'net')).toContain(message(other, 'report-result-loss'));
      }, SLOW_MS);
      expect(
        screen.getByRole('combobox', { name: message(other, 'report-filter-trip-view') }),
      ).toHaveTextContent(message(other, 'report-filter-trip-by-partner'));
      expect(cell('1', 'partner')).toBe(other === 'ur' ? 'الف' : 'Partner A');
      expect(runCount()).toBe(before);
    });

    it('shows the print of the trip by partner with its signs and the layout in the header', async () => {
      const { user } = await openReport('R33', { print: PRINT_R33_PARTNER_EN });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByText(printFirmName(language), undefined, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
        id: 'R33',
        language,
        paper: 'a4',
      });
      const body = visible(document.querySelector('.print-report tbody')?.textContent);
      expect(body).toContain('Partner A');
      expect(body).toContain('-776');
      const facts = visible(document.querySelector('.print-facts')?.textContent);
      expect(facts).toContain(m('report-filter-trip-by-partner'));
    });
  });

  describe('R34 the day summary', () => {
    it('draws a block of lines for the day, one line each, with the backend’s figures', async () => {
      await openReport('R34');
      // A long list draws the rows in view only (the rest come as it scrolls), so the first rows are read.
      await waitFor(() => {
        expect(drawnRows().length).toBeGreaterThan(5);
      }, SLOW_MS);
      expect(cell('0', 'date')).toBe('10-10-2026');
      expect(cell('0', 'item')).toBe(m('report-r34-bikri'));
      expect(cell('0', 'count')).toBe('2');
      expect(cell('0', 'amount')).toBe('41,400');
      expect(cell('1', 'item')).toBe(m('report-r34-commission'));
      expect(cell('1', 'amount')).toBe('5,448');
      expect(cell('2', 'item')).toBe(m('report-r34-cash-opening'));
      expect(cell('2', 'amount')).toBe('50,000');
      expect(cell('5', 'item')).toBe(m('report-r34-cash-closing'));
      expect(cell('5', 'amount')).toBe('58,484');
      expect(drawnColumns()).toEqual(['date', 'item', 'count', 'amount']);
      // The page says how many lines the backend has: all 27 of the day.
      expect(
        await screen.findByText(paragraph(m('report-count', { shown: 27, total: 27 }))),
      ).toBeInTheDocument();
    });

    it('has a fruit row after the gardens: the count is نگ and the amount the value', async () => {
      // The block that holds the fruit row, drawn whole.
      const block: ReportPage = {
        ...R34_FRUIT,
        rows: R34_FRUIT.rows.slice(17, 21),
        total_count: 4,
      };
      await openReport('R34', { pages: { R34: block } });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      expect(cell('2', 'item')).toBe(m('report-r34-fruit-taken'));
      expect(cell('2', 'count')).toBe('5');
      expect(cell('2', 'amount')).toBe('4,160');
    });

    it('has a heading row for the partners’ shares and the partners under it by name', async () => {
      // The block of the partners and what follows it, as a short list that is drawn whole.
      const block: ReportPage = {
        ...R34_DAY,
        rows: R34_DAY.rows.slice(19, 25),
        total_count: 6,
      };
      await openReport('R34', { pages: { R34: block } });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(6);
      }, SLOW_MS);
      expect(cell('1', 'item')).toBe(m('report-r34-partner-shares'));
      expect(cell('1', 'count')).toBe('');
      expect(cell('1', 'amount')).toBe('');
      expect(cell('2', 'item')).toBe(names('الف', 'Partner A'));
      expect(cell('2', 'amount')).toBe('996');
      expect(cell('3', 'item')).toBe(names('ب', 'Partner B'));
      expect(cell('3', 'amount')).toBe('746');
      expect(cell('5', 'item')).toBe(m('report-r34-buyers-dues'));
      expect(cell('5', 'amount')).toBe('21,564');
    });

    it('has no totals row, because lines of different kinds do not add up', async () => {
      await openReport('R34');
      await waitFor(() => {
        expect(drawnRows().length).toBeGreaterThan(5);
      }, SLOW_MS);
      expect(document.querySelector('.ag-row-pinned')).toBeNull();
    });

    it('asks for the day typed and only the days', async () => {
      const { user } = await openReport('R34');
      await waitFor(() => {
        expect(drawnRows().length).toBeGreaterThan(5);
      }, SLOW_MS);
      await user.click(screen.getByLabelText(m('report-filter-from')));
      await user.keyboard('10-10-2026');
      await waitFor(() => {
        expect(lastRun()?.filters.from).toBe('10-10-2026');
      }, SLOW_MS);
      expect(screen.queryByRole('combobox', { name: m('report-filter-party') })).toBeNull();
    });

    it('shows the print with every line and no totals row', async () => {
      const { user } = await openReport('R34', { print: PRINT_R34_EN });
      await waitFor(() => {
        expect(drawnRows().length).toBeGreaterThan(5);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByText(printFirmName(language), undefined, SLOW_MS);
      expect(document.querySelectorAll('.print-report tbody tr')).toHaveLength(27);
      expect(document.querySelector('.print-report tfoot')).toBeNull();
      expect(visible(document.querySelector('.print-report tbody')?.textContent)).toContain(
        '58,484',
      );
    });
  });

  describe('the four states of the new reports', () => {
    it('says what happened and tries again when the backend fails', async () => {
      let failing = true;
      const { user } = await openReport('R34', {
        refuse: () => {
          if (!failing) return null;
          failing = false;
          return {
            ok: false,
            errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
            ref: 'E-1',
          } as never;
        },
      });
      await screen.findByText(named(m('report-load-failed')), undefined, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('common-retry')) }));
      await waitFor(() => {
        expect(drawnRows().length).toBeGreaterThan(5);
      }, SLOW_MS);
    });

    it('says there is nothing, and what to do, when a layout has no rows', async () => {
      await openReport('R33', { pages: { R33: { ...R33_TRIP, rows: [], total_count: 0 } } });
      expect(await screen.findByText(named(m('report-empty')), undefined, SLOW_MS)).toBeVisible();
    });
  });

  describe('keyboard only', () => {
    it('opens R33 from the list, takes the partner layout with the keys, and Esc leaves it', async () => {
      fakeReports();
      const { router } = await renderApp('/reports');
      await screen.findByRole('heading', { level: 1, name: named(m('reports-title')) });
      const user = userEvent.setup();
      // 10 Sales reports (R1 has focus), 9 Accounts, 8 Collections and R28 to R32 are in front of R33:
      // thirty-two steps down.
      await waitFor(() => {
        expect(document.activeElement?.getAttribute('href')).toMatch(/R1$/);
      }, SLOW_MS);
      await user.keyboard(`${'{ArrowDown}'.repeat(32)}{Enter}`);
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports/R33');
      }, SLOW_MS);
      // The first date has focus; Tab goes to the second date and then to the layout box.
      const from = screen.getByLabelText(m('report-filter-from'));
      await waitFor(() => {
        expect(document.activeElement).toBe(from);
      }, SLOW_MS);
      const box = screen.getByRole('combobox', { name: m('report-filter-trip-view') });
      for (let tabs = 0; tabs < 4; tabs += 1) {
        if (document.activeElement === box) break;
        await user.tab();
      }
      expect(document.activeElement).toBe(box);
      // Enter opens the list; ↓ four times goes from "by trip" to "by partner"; Enter takes it.
      await user.keyboard('{Enter}');
      await screen.findByRole('option', { name: m('report-filter-trip-by-partner') });
      await user.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}{ArrowDown}{Enter}');
      await waitFor(() => {
        expect(lastRun()?.filters.trip_view).toBe('partner');
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      }, SLOW_MS);
    });
  });

  it('the partners api is not asked by the new reports', async () => {
    await openReport('R34');
    expect(partnersApi.partnersList).not.toHaveBeenCalled();
  });
});
