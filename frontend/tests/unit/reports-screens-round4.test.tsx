// The Sales, Collections and Business reports on the ONE report screen (plan 20, step F3, PRD 19.2,
// FR-13.9, FR-19.1 to FR-19.5, FR-19.11, FR-19.13): R2, R3, R4, R6, R7, R8, R9, R20, R21, R22, R24,
// R26, R27, R28 and R30, in the Urdu and in the English app, against a fake backend that answers
// with the backend's own fixtures. Each report asks for exactly what the filters hold and shows the
// backend's totals; the headings of R21, R22 and R24, the month names of R28, the minus sign of
// R30, the person box of R9 (one-time people, FR-13.9), the fold of the filter boxes, the print
// preview, a language switch in the middle of a filter and the keyboard-only path are each proved.
// The screen adds up nothing: every figure is one the backend sent. The layout step in a test with
// no layout is the middle one (normal), so the columns of the wide step are not drawn here; the
// screenshots in both languages and at three window sizes show those.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportId } from '../../src/bindings/ReportId';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
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
import { message, visible } from '../helpers/render';
import {
  CATALOG,
  PEOPLE_SHAHID,
  PRINT_R28_MONTH_EN,
  R2_ALL,
  R2_ALL_REQUEST,
  R20_DUES_REQUEST,
  R21_BY_CITY_REQUEST,
  R22_BY_COLLECTOR_REQUEST,
  R24_PAYABLES_REQUEST,
  R26_ADVANCES_REQUEST,
  R27_SERIES_REQUEST,
  R28_BY_DAY_REQUEST,
  R28_BY_MONTH_REQUEST,
  R30_PROFIT_REQUEST,
  R3_CASH_REQUEST,
  R4_CREDIT_REQUEST,
  R6_LOTS_REQUEST,
  R7_ITEMS_REQUEST,
  R8_ADDA_REQUEST,
  R9_ALL_REQUEST,
  R9_DUE_REQUEST,
  fakeReports,
} from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
// The window of a test has no layout, so the step is the middle one; a test that needs the compact
// step (the fold of the filter boxes) sets it.
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

/**
 * The filters of a request the backend wrote, as the screen sends them: the two newer keys are in
 * the backend's requests as null when unset and are left out by the screen (same meaning).
 */
function sentLike(request: ReportRunInput) {
  const {
    group_by: groupBy,
    status,
    partner_id: partnerId,
    garden_id: gardenId,
    bikri_id: bikriId,
    ...rest
  } = request.filters;
  return {
    ...rest,
    // The requests the backend wrote before round 7 have no `partner_id`; the screen always sends it.
    partner_id: partnerId ?? null,
    // The backend's requests leave `garden_id` out when it is unset; the screen always sends it.
    garden_id: gardenId ?? null,
    bikri_id: bikriId ?? null,
    ...(groupBy === undefined || (groupBy as unknown) === null ? {} : { group_by: groupBy }),
    ...(status === undefined || (status as unknown) === null ? {} : { status }),
  };
}

function lastRun() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];
}

describe.each(LANGUAGES)('the round 4 reports (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);

  async function openReport(id: string, options: Parameters<typeof fakeReports>[0] = {}) {
    const entry = CATALOG.find((candidate) => candidate.id === id);
    const fake = fakeReports(options);
    const view = await renderApp(`/reports/${id}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(entry?.title_id ?? 'report-r2-title')),
    });
    return { ...view, fake, user: userEvent.setup() };
  }

  describe('every report asks for what the filters hold and shows the backend’s totals', () => {
    // [id, the backend's request, the column and the text of the total it must show]
    const cases: [ReportId, ReportRunInput, string, string][] = [
      ['R2', R2_ALL_REQUEST, 'buyer_total', '44,364'],
      ['R3', R3_CASH_REQUEST, 'buyer_total', '12,800'],
      ['R4', R4_CREDIT_REQUEST, 'buyer_total', '31,564'],
      ['R6', R6_LOTS_REQUEST, 'net', '29,476'],
      ['R7', R7_ITEMS_REQUEST, 'total_comm', '5,448'],
      ['R8', R8_ADDA_REQUEST, 'gross', '41,400'],
      ['R9', R9_ALL_REQUEST, 'amount', '12,800'],
      ['R20', R20_DUES_REQUEST, 'balance', '21,564'],
      ['R21', R21_BY_CITY_REQUEST, 'balance', '21,564'],
      ['R22', R22_BY_COLLECTOR_REQUEST, 'balance', '21,564'],
      ['R24', R24_PAYABLES_REQUEST, 'payable', '10,160'],
      ['R26', R26_ADVANCES_REQUEST, 'given', '15,000'],
      ['R27', R27_SERIES_REQUEST, 'net', '29,476'],
      ['R28', R28_BY_MONTH_REQUEST, 'amount', '5,488'], // and the side word, below
      ['R30', R30_PROFIT_REQUEST, 'net', '488'],
    ];

    it.each(cases)('%s', async (id, request, column, total) => {
      // The fixture of a report that was asked with filters (R3 for today, R4 for one item, R7 for
      // one day) is answered the same however the screen asks; this proves the first request is
      // empty filters, which every one of them accepts, and the totals row is the backend's.
      await openReport(id);
      await waitFor(() => {
        // A balance carries its side as a word after the figure (R24's and R28's totals).
        expect(totalsCell(column)).toMatch(new RegExp(`^${total}(${m('report-side-cr')})?$`));
      }, SLOW_MS);
      expect(lastRun()).toMatchObject({ id, offset: 0, limit: 100 });
      expect(lastRun()?.filters).toEqual(
        sentLike({
          ...request,
          filters: {
            ...request.filters,
            shortcut: null,
            from: '',
            to: '',
            as_of: '',
            item_id: null,
            party: null,
            seller_id: null,
          },
        }),
      );
    });
  });

  describe('R2 the sales register', () => {
    it('draws the sale lines of the day with the goods, and the bikri behind a row', async () => {
      const { router, user } = await openReport('R2');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
      expect(cell('0', 'buyer')).toBe(`03040001 ${names('اکرم ٹول ٹیکس', 'Akram Toll Tax')}`);
      expect(cell('0', 'item')).toBe(names('کیلا', 'Banana'));
      expect(cell('0', 'qty')).toBe('20');
      expect(cell('0', 'rate')).toBe('600');
      expect(cell('0', 'value')).toBe('12,000');
      expect(cell('0', 'buyer_total')).toBe('12,880');
      expect(cell('0', 'doc')).toBe(
        m('report-doc-number', { kind: m('report-doc-bikri-first'), number: '1' }),
      );
      const buyerCell = document.querySelector('.ag-row[row-id="0"] [col-id="buyer"]');
      if (buyerCell === null) throw new Error('no buyer cell');
      await user.click(buyerCell);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      }, SLOW_MS);
    });

    it('shows the first page of 100 and says how many rows the filter has', async () => {
      await openReport('R2', { pages: { R2: { ...R2_ALL, total_count: 2 } } });
      expect(
        await screen.findByText(
          named(m('report-count', { shown: 2, total: 2 })),
          undefined,
          SLOW_MS,
        ),
      ).toBeInTheDocument();
    });

    it('shows every filter box from the normal width, with no “More filters” button', async () => {
      await openReport('R2');
      for (const id of ['seller', 'zamindar', 'item', 'city', 'adda']) {
        expect(
          screen.getByRole('combobox', { name: m(`report-filter-${id}`) }),
        ).toBeInTheDocument();
      }
      expect(screen.queryByRole('button', { name: named(m('report-filters-more')) })).toBeNull();
    });

    it('folds the pickers of records behind “More filters” in a compact window, and a click opens them', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('compact');
      const { user } = await openReport('R2');
      const more = await screen.findByRole('button', { name: named(m('report-filters-more')) });
      // A folded box is not drawn at all, so the arrow keys never try to enter it.
      expect(screen.queryByRole('combobox', { name: m('report-filter-seller') })).toBeNull();
      expect(screen.getByRole('combobox', { name: m('report-filter-party') })).toBeInTheDocument();
      await user.click(more);
      expect(screen.getByRole('combobox', { name: m('report-filter-seller') })).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: named(m('report-filters-fewer')) }),
      ).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(m('report-filters-fewer')) }));
      expect(screen.queryByRole('combobox', { name: m('report-filter-seller') })).toBeNull();
    });
  });

  describe('R6 lots', () => {
    it('keeps the seller box in view in a compact window (it has no party box) and folds the rest', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('compact');
      await openReport('R6');
      expect(screen.getByRole('combobox', { name: m('report-filter-seller') })).toBeInTheDocument();
      expect(screen.queryByRole('combobox', { name: m('report-filter-adda') })).toBeNull();
      expect(
        screen.getByRole('button', { name: named(m('report-filters-more')) }),
      ).toBeInTheDocument();
    });

    it('draws the gross sale, the deductions and the net sale of E2E-1', async () => {
      await openReport('R6');
      await waitFor(() => {
        expect(cell('0', 'net')).toBe('19,316');
      }, SLOW_MS);
      expect(cell('0', 'gross')).toBe('29,400');
      expect(cell('0', 'deductions')).toBe('10,084');
      expect(cell('0', 'commission')).toBe('1,764');
    });

    it('leaves the item and the store rent to the print, and the charges to the wide step', async () => {
      await openReport('R6');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      // The test window has no layout, so it is the normal step: the quantity and the commission
      // are drawn, the charges of the wide step are not.
      expect(drawnColumns()).toEqual([
        'date',
        'doc',
        'seller',
        'qty',
        'gross',
        'commission',
        'deductions',
        'net',
      ]);
    });
  });

  describe('R8 the اڈا report', () => {
    it('names the sales that have no اڈا', async () => {
      await openReport('R8');
      await waitFor(() => {
        expect(cell('1', 'adda')).toBe(m('report-adda-none'));
      }, SLOW_MS);
      expect(cell('0', 'adda')).toBe(names('سوات اڈا', 'Swat Adda'));
    });
  });

  describe('R9 the register of one-time people', () => {
    it('has a person box and a paid-or-due box, and no account box', async () => {
      await openReport('R9');
      expect(screen.getByLabelText(m('report-filter-person'))).toBeInTheDocument();
      expect(screen.getByRole('combobox', { name: m('report-filter-status') })).toBeInTheDocument();
      expect(screen.queryByLabelText(m('report-filter-party'))).toBeNull();
    });

    it('draws the line of شاہد as paid', async () => {
      await openReport('R9');
      await waitFor(() => {
        expect(cell('0', 'status')).toBe(m('report-status-paid'));
      }, SLOW_MS);
      expect(cell('0', 'amount')).toBe('12,800');
      // The side word, the phone and the commission wait for the wide step (the print has them).
      expect(drawnColumns()).toEqual([
        'date',
        'doc',
        'person',
        'item',
        'qty',
        'amount',
        'status',
        'open',
      ]);
    });

    it('asks only for the due lines when the box says so, and says so when there are none', async () => {
      const { user } = await openReport('R9');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-status') }));
      await user.click(await screen.findByRole('option', { name: m('report-status-due') }));
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(sentLike(R9_DUE_REQUEST));
      }, SLOW_MS);
      // The backend has no due line: the empty state offers to clear the filter.
      expect(await screen.findByText(named(m('report-empty')), undefined, SLOW_MS)).toBeVisible();
      await user.click(screen.getByRole('button', { name: named(m('report-clear-filters')) }));
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(sentLike(R9_ALL_REQUEST));
      }, SLOW_MS);
    });

    it('finds a person by what is typed, as typed, and asks for his lines with his key', async () => {
      const { user } = await openReport('R9');
      const box = screen.getByLabelText(m('report-filter-person'));
      await user.type(box, 'shahid');
      await waitFor(() => {
        expect(reportsApi.reportsFindPeople).toHaveBeenCalledWith({ text: 'shahid', limit: 10 });
      }, SLOW_MS);
      // The list shows his name in this script, his phone and his last day.
      const option = await screen.findByRole('option', undefined, SLOW_MS);
      expect(visible(option.textContent)).toContain(names('شاہد', 'Shahid'));
      expect(visible(option.textContent)).toContain('03000000000');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(lastRun()?.filters.party).toEqual({
          account_id: 9,
          person_key: 'شاہد|03000000000',
        });
      }, SLOW_MS);
      expect(box).toHaveValue(names('شاہد', 'Shahid'));
    });

    it('never converts the English letters typed in the person box to Urdu', async () => {
      const { user } = await openReport('R9');
      const box = screen.getByLabelText(m('report-filter-person'));
      await user.type(box, 'shahid');
      expect(box).toHaveValue('shahid');
    });

    it('shows the person in the other language after a switch, and asks nothing again', async () => {
      const { user } = await openReport('R9');
      const box = screen.getByLabelText(m('report-filter-person'));
      await user.type(box, 'shahid');
      await screen.findByRole('option', undefined, SLOW_MS);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(lastRun()?.filters.party?.person_key).toBe('شاہد|03000000000');
      }, SLOW_MS);
      const before = vi.mocked(reportsApi.reportsRun).mock.calls.length;
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(screen.getByLabelText(message(other, 'report-filter-person'))).toHaveValue(
          other === 'ur' ? 'شاہد' : 'Shahid',
        );
      }, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsRun).mock.calls.length).toBe(before);
    });

    it('opens on the person of the address, whole', async () => {
      const hit = PEOPLE_SHAHID[0];
      if (hit === undefined) throw new Error('no person');
      const person = {
        accountId: 9,
        personKey: hit.person_key,
        name_ur: hit.name_ur,
        name_en: hit.name_en,
        phone: hit.phone,
        last_date: hit.last_date,
      };
      fakeReports();
      await renderApp(`/reports/R9?person=${encodeURIComponent(JSON.stringify(person))}`);
      await screen.findByRole('heading', { level: 1, name: named(m('report-r9-title')) });
      await waitFor(() => {
        expect(lastRun()?.filters.party).toEqual({
          account_id: 9,
          person_key: 'شاہد|03000000000',
        });
      }, SLOW_MS);
      expect(screen.getByLabelText(m('report-filter-person'))).toHaveValue(names('شاہد', 'Shahid'));
    });
  });

  describe('a report on a one-time account narrows to one person', () => {
    it('shows the person box once the party is a پختہ account, and only his lines', async () => {
      const party = {
        account_id: 9,
        code: '03049999',
        short_code: null,
        name_ur: 'پختہ کھاتہ (خریدار)',
        name_en: 'Pukhta Khata (Buyer)',
        tag: null,
        person_key: null,
      };
      const page = { ...R2_ALL, filters: { ...R2_ALL.filters, party } };
      fakeReports({ pages: { R2: page } });
      const { user } = {
        ...(await renderApp('/reports/R2?party=9')),
        user: userEvent.setup(),
      };
      await screen.findByRole('heading', { level: 1, name: named(m('report-r2-title')) });
      const box = await screen.findByLabelText(m('report-filter-person'), undefined, SLOW_MS);
      await user.type(box, 'shah');
      await screen.findByRole('option', undefined, SLOW_MS);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(lastRun()?.filters.party).toEqual({
          account_id: 9,
          person_key: 'شاہد|03000000000',
        });
      }, SLOW_MS);
    });

    it('lists nobody for an account the person has no lines on', async () => {
      const party = {
        account_id: 30,
        code: '03049999',
        short_code: null,
        name_ur: 'پختہ کھاتہ (خریدار)',
        name_en: 'Pukhta Khata (Buyer)',
        tag: null,
        person_key: null,
      };
      const page = { ...R2_ALL, filters: { ...R2_ALL.filters, party } };
      fakeReports({ pages: { R2: page } });
      const { user } = {
        ...(await renderApp('/reports/R2?party=30')),
        user: userEvent.setup(),
      };
      await screen.findByRole('heading', { level: 1, name: named(m('report-r2-title')) });
      const box = await screen.findByLabelText(m('report-filter-person'), undefined, SLOW_MS);
      await user.type(box, 'shah');
      await waitFor(() => {
        expect(reportsApi.reportsFindPeople).toHaveBeenCalled();
      }, SLOW_MS);
      expect(screen.queryByRole('option')).toBeNull();
    });
  });

  describe('R20, R21 and R22 the dues', () => {
    it('R20 shows the balance and the age of each buyer’s oldest bill', async () => {
      await openReport('R20');
      await waitFor(() => {
        expect(cell('0', 'balance')).toBe('2,880');
      }, SLOW_MS);
      expect(cell('0', 'oldest_date')).toBe('10-10-2026');
      expect(cell('0', 'days')).toBe('0');
      expect(cell('0', 'age_0_7')).toBe('2,880');
      // The normal step has the two younger bands; the city and the phone wait for the wide step.
      expect(drawnColumns()).toEqual([
        'buyer',
        'balance',
        'oldest_date',
        'days',
        'age_0_7',
        'age_8_30',
        'age_31_90',
        'age_91',
      ]);
    });

    it('R21 draws a heading for each city with its balance, and names the buyers with no city', async () => {
      await openReport('R21');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1', '2', '3']);
      }, SLOW_MS);
      expect(cell('0', 'group')).toBe(names('کوٹ رادھا کشن', 'Kot Radha Kishan'));
      expect(cell('0', 'balance')).toBe('2,880');
      // The buyer under it has an empty first cell.
      expect(cell('1', 'group')).toBe('');
      expect(cell('1', 'buyer')).toBe(`03040001 ${names('اکرم ٹول ٹیکس', 'Akram Toll Tax')}`);
      // The heading of the buyers with no city has no name from the backend: the screen says so.
      expect(cell('2', 'group')).toBe(m('report-group-no-city'));
      expect(cell('2', 'balance')).toBe('18,684');
      // A heading is a group row, so it reads stronger than a buyer.
      expect(document.querySelector('.ag-row[row-id="0"]')?.className).toContain('row-group');
    });

    it('R22 names the buyers with no collector', async () => {
      await openReport('R22');
      await waitFor(() => {
        expect(cell('2', 'group')).toBe(m('report-group-no-collector'));
      }, SLOW_MS);
      expect(cell('0', 'group')).toBe(names('اسلم', 'Aslam'));
    });

    it('the shop’s total is the same figure in the three reports', async () => {
      for (const id of ['R20', 'R21', 'R22'] as const) {
        const view = await openReport(id);
        await waitFor(() => {
          expect(totalsCell('balance')).toBe('21,564');
        }, SLOW_MS);
        view.unmount();
      }
    });
  });

  describe('R24 seller payables', () => {
    it('heads the list of payables, and shows the balance with its side as a word', async () => {
      await openReport('R24');
      await waitFor(() => {
        expect(cell('0', 'section')).toBe(m('report-section-payable'));
      }, SLOW_MS);
      expect(cell('0', 'payable')).toBe('10,160');
      expect(cell('1', 'seller')).toBe(`030110002 ${names('کریم', 'Karim')}`);
      expect(cell('1', 'balance')).toContain('10,160');
      expect(cell('1', 'balance')).toContain(m('report-side-cr'));
    });
  });

  describe('R26 advances', () => {
    it('shows E2E-7: given 15,000, adjusted 15,000, open 0', async () => {
      await openReport('R26');
      await waitFor(() => {
        expect(cell('0', 'given')).toBe('15,000');
      }, SLOW_MS);
      expect(cell('0', 'adjusted')).toBe('15,000');
      expect(cell('0', 'open')).toBe('0');
      expect(cell('0', 'last_date')).toBe('01-09-2026');
    });
  });

  describe('R28 the income report', () => {
    it('names the month from Fluent, and groups by month by default', async () => {
      await openReport('R28');
      await waitFor(() => {
        expect(cell('0', 'period')).toBe(`${m('report-month-10')} 2026`);
      }, SLOW_MS);
      expect(lastRun()?.filters).toEqual(sentLike(R28_BY_MONTH_REQUEST));
      expect(totalsCell('amount')).toContain('5,488');
      expect(totalsCell('amount')).toContain(m('report-side-cr'));
    });

    it('asks for the days when the group box says so, and shows each day', async () => {
      const { user } = await openReport('R28');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-group-by') }));
      await user.click(await screen.findByRole('option', { name: m('report-group-by-day') }));
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(sentLike(R28_BY_DAY_REQUEST));
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell('0', 'period')).toBe('10-10-2026');
      }, SLOW_MS);
    });

    it('keeps the group box through a language switch and does not ask again', async () => {
      const { user } = await openReport('R28');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-group-by') }));
      await user.click(await screen.findByRole('option', { name: m('report-group-by-day') }));
      await waitFor(() => {
        expect(cell('0', 'period')).toBe('10-10-2026');
      }, SLOW_MS);
      const before = vi.mocked(reportsApi.reportsRun).mock.calls.length;
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(
          screen.getByRole('combobox', { name: message(other, 'report-filter-group-by') }),
        ).toHaveTextContent(message(other, 'report-group-by-day'));
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell('0', 'period')).toBe('10-10-2026');
      }, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsRun).mock.calls.length).toBe(before);
    });
  });

  describe('R30 profit and the partner split', () => {
    it('shows a partner’s negative net with its minus sign, and the lines above the table', async () => {
      await openReport('R30');
      await waitFor(() => {
        expect(cell('0', 'net')).toBe('-1,004');
      }, SLOW_MS);
      expect(cell('0', 'profit_share')).toBe('996');
      expect(cell('0', 'taken')).toBe('2,000');
      expect(totalsCell('income_share')).toBe('5,488');
      expect(totalsCell('expense_share')).toBe('3,000');
      // The three lines the backend sends above the table.
      for (const id of ['report-r30-income', 'report-r30-expenses', 'report-r30-profit']) {
        expect(screen.getByText(named(m(id)))).toBeInTheDocument();
      }
    });
  });

  describe('R27 the bill series', () => {
    it('shows the bills in number order with both dates at the normal step', async () => {
      await openReport('R27');
      await waitFor(() => {
        expect(cell('1', 'net')).toBe('10,160');
      }, SLOW_MS);
      expect(cell('0', 'bill_date')).toBe('10-10-2026');
      expect(cell('0', 'deductions')).toBe('10,084');
    });
  });

  describe('the four states', () => {
    it('shows the loading state while the backend answers', async () => {
      fakeReports();
      vi.mocked(reportsApi.reportsRun).mockImplementation(() => new Promise(() => undefined));
      await renderApp('/reports/R20');
      await screen.findByRole('heading', { level: 1, name: named(m('report-r20-title')) });
      expect(await screen.findByRole('status')).toBeInTheDocument();
    });

    it('says what happened and tries again when the backend fails', async () => {
      let failing = true;
      const { user } = await openReport('R24', {
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
        expect(cell('0', 'section')).toBe(m('report-section-payable'));
      }, SLOW_MS);
    });

    it('says there is nothing, and what to do, when the filter has no rows', async () => {
      await openReport('R26', {
        pages: { R26: { ...R2_ALL, id: 'R26', rows: [], total_count: 0 } },
      });
      expect(await screen.findByText(named(m('report-empty')), undefined, SLOW_MS)).toBeVisible();
      expect(screen.getByText(named(m('report-empty-help')))).toBeVisible();
    });
  });

  describe('the print preview of these reports', () => {
    it('asks for the print data of R28 and draws the month by its name', async () => {
      const { user } = await openReport('R28', { print: PRINT_R28_MONTH_EN });
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await waitFor(() => {
        expect(reportsApi.reportsPrintData).toHaveBeenCalled();
      }, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
        id: 'R28',
        language,
        paper: 'a4',
      });
      // The print data is the English one; the month is named by the print language's words.
      await screen.findByText(PRINT_R28_MONTH_EN.firm.name, undefined, SLOW_MS);
      const body = visible(document.querySelector('.print-report tbody')?.textContent);
      expect(body).toContain(`${message('en', 'report-month-10')} 2026`);
    });
  });

  describe('keyboard only', () => {
    it('the person box, the list, Enter, the table and a row open, without the mouse', async () => {
      const { router, user } = await openReport('R9');
      // The first filter box has focus; ← → ↑ ↓ and Tab move between the boxes. Go to the person box.
      const box = screen.getByLabelText(m('report-filter-person'));
      await act(async () => {
        box.focus();
        await Promise.resolve();
      });
      await user.keyboard('shahid');
      await screen.findByRole('option', undefined, SLOW_MS);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(lastRun()?.filters.party?.person_key).toBe('شاہد|03000000000');
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      // The table is reached with Tab, a row with ↓, and Enter opens the بکری behind it.
      for (let tabs = 0; tabs < 8; tabs += 1) {
        if (document.activeElement?.closest('.ag-root-wrapper') != null) break;
        await user.tab();
      }
      expect(document.activeElement?.closest('.ag-root-wrapper')).not.toBeNull();
      await user.keyboard('{ArrowDown}{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      }, SLOW_MS);
    });

    it('opens R20 from the list with ↓ and Enter, types the day, and Esc leaves it', async () => {
      fakeReports();
      const { router } = await renderApp('/reports');
      await screen.findByRole('heading', { level: 1, name: named(m('reports-title')) });
      const user = userEvent.setup();
      // R2 R3 R4 R6 R7 R8 R9 R10 R11 R12 R13 R14 R15 R16 R17 R18 R19 are in front of R20: seventeen
      // steps down.
      await waitFor(() => {
        expect(document.activeElement?.getAttribute('href')).toMatch(/R2$/);
      }, SLOW_MS);
      await user.keyboard(`${'{ArrowDown}'.repeat(17)}{Enter}`);
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports/R20');
      }, SLOW_MS);
      const box = await screen.findByLabelText(m('report-filter-as-of'));
      await waitFor(() => {
        expect(document.activeElement).toBe(box);
      }, SLOW_MS);
      await user.keyboard('10-10-2026');
      await waitFor(() => {
        expect(lastRun()?.filters.as_of).toBe('10-10-2026');
      }, SLOW_MS);
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      }, SLOW_MS);
    });
  });

  describe('the catalogue says what each of these reports shows', () => {
    it('has a title and a help line in this language for each of the 15', () => {
      for (const entry of CATALOG.filter((candidate) =>
        ['sales', 'collections', 'business'].includes(candidate.group),
      )) {
        expect(m(entry.title_id)).not.toBe('');
        expect(m(entry.title_id.replace('-title', '-help'))).not.toBe('');
      }
    });
  });
});
