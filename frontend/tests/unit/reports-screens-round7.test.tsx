// The cash book (R12), the expense report (R29), the partner statement (R31) and the money lent
// report (R32) on the ONE report screen (round 7, PRD 16.1, 16.2, 16.5, FR-17.9, FR-17.10, FR-19.2,
// FR-19.10, FR-19.13), in the Urdu and in the English app, against a fake backend that answers with the
// backend's own fixtures. Each report asks for exactly what the filters hold and shows the backend's
// totals. The partner box (R29 takes one partner or all, R31 cannot run without one), the sentences of
// the partner lines (message cells made of Fluent words and the backend's figures), the person box
// of R32 (the people who can borrow), the print preview, a language switch in the middle of a filter
// and the keyboard-only path are each proved. The screen adds up nothing: every figure is one the
// backend sent. The layout step in a test with no layout is the middle one (normal), so the columns of
// the wide step are not drawn here; the screenshots in both languages and at three window sizes show
// those.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import * as partnersApi from '../../src/features/partners/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners, partnerRow } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import {
  CATALOG,
  PRINT_R31_BAY_EN,
  R12_DAY_REQUEST,
  R29_ALIF_REQUEST,
  R29_DAY_REQUEST,
  R31_BAY,
  R31_BAY_REQUEST,
  R32_E2E_30B_REQUEST,
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

/** The partners of the worked day (الف 40%, ب 30%, ج 30%), the ids the backend's fixtures use. */
const PARTNERS = [
  partnerRow(1, 'الف', 'Partner A', 4000),
  partnerRow(2, 'ب', 'Partner B', 3000),
  partnerRow(3, 'ج', 'Partner C', 3000),
];

/** The people who can borrow (control 0103): the party box of R32 searches this control. */
const LENT_CONTROL = {
  id: 103,
  head_id: 1,
  code: '0103',
  name_ur: 'ادھار رقم',
  name_en: 'Money Lent',
  party_type: 'other' as const,
  allowed_for_other: false,
};

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  fakePartners(PARTNERS);
  vi.mocked(accountApi.coaControlsList).mockResolvedValue([LENT_CONTROL]);
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

/** The filters of a request the backend wrote, as the screen sends them (every key is sent). */
function sentLike(request: ReportRunInput, change: Record<string, unknown> = {}) {
  const { group_by: groupBy, status, ...rest } = request.filters;
  return {
    ...rest,
    ...(groupBy === undefined || (groupBy as unknown) === null ? {} : { group_by: groupBy }),
    ...(status === undefined || (status as unknown) === null ? {} : { status }),
    ...change,
  };
}

function lastRun() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];
}

function runCount() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.length;
}

describe.each(LANGUAGES)('the round 7 reports (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const other: Language = language === 'ur' ? 'en' : 'ur';

  async function openReport(
    id: string,
    options: Parameters<typeof fakeReports>[0] = {},
    search = '',
  ) {
    const entry = CATALOG.find((candidate) => candidate.id === id);
    const fake = fakeReports(options);
    const view = await renderApp(`/reports/${id}${search}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(entry?.title_id ?? 'report-r12-title')),
    });
    return { ...view, fake, user: userEvent.setup() };
  }

  async function pickPartner(user: ReturnType<typeof userEvent.setup>, name: string) {
    await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
    await user.click(await screen.findByRole('option', { name }));
  }

  function switchLanguage() {
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
  }

  describe('R12 the cash book', () => {
    it('asks for what the filters hold and shows the backend’s totals and balances', async () => {
      await openReport('R12');
      await waitFor(() => {
        expect(totalsCell('cash_in')).toBe('22,800');
      }, SLOW_MS);
      expect(totalsCell('cash_out')).toBe('14,316');
      expect(totalsCell('balance')).toContain('58,484');
      expect(lastRun()).toMatchObject({ id: 'R12', offset: 0, limit: 100 });
      expect(lastRun()?.filters).toEqual(
        sentLike(R12_DAY_REQUEST, { shortcut: null, from: '', to: '' }),
      );
      // The opening and the closing cash are the two lines above the table.
      for (const id of ['report-opening-balance', 'report-closing-balance']) {
        expect(screen.getByText(named(m(id)))).toBeInTheDocument();
      }
      const lines = visible(document.body.textContent);
      expect(lines).toContain('50,000');
    });

    it('draws every cash in and out of the day with its kind of entry and the running balance', async () => {
      await openReport('R12');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(6);
      }, SLOW_MS);
      expect(cell('0', 'cash_out')).toBe('5,000');
      expect(cell('1', 'cash_in')).toBe('12,800');
      expect(cell('2', 'doc')).toBe(
        m('report-doc-number', { kind: m('report-doc-receipt'), number: '1' }),
      );
      expect(cell('2', 'account')).toContain('03040001');
      expect(cell('5', 'balance')).toContain('58,484');
      // The narration waits for the wide step (the print has it).
      expect(drawnColumns()).toEqual(['date', 'doc', 'account', 'cash_in', 'cash_out', 'balance']);
    });

    it('opens with the focus on the first date, so the day can be typed at once', async () => {
      await openReport('R12');
      const from = screen.getByLabelText(m('report-filter-from'));
      await waitFor(() => {
        expect(document.activeElement).toBe(from);
      }, SLOW_MS);
    });

    it('has the days and nothing else to filter by', async () => {
      await openReport('R12');
      expect(screen.getByRole('combobox', { name: m('report-filter-period') })).toBeInTheDocument();
      expect(screen.getByLabelText(m('report-filter-from'))).toBeInTheDocument();
      expect(screen.queryByLabelText(m('report-filter-party'))).toBeNull();
      expect(screen.queryByRole('combobox', { name: m('report-filter-partner') })).toBeNull();
    });

    it('opens the voucher behind a row with Enter', async () => {
      const { router, user } = await openReport('R12');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(6);
      }, SLOW_MS);
      const target = document.querySelector('.ag-row[row-id="2"] [col-id="account"]');
      if (target === null) throw new Error('no account cell');
      await user.click(target);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/voucher');
      }, SLOW_MS);
    });
  });

  describe('R29 the expense report', () => {
    it('draws the shop expenses and the partner money as two sections with the backend’s totals', async () => {
      await openReport('R29');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      expect(cell('0', 'section')).toBe(m('report-section-shop-expenses'));
      expect(cell('1', 'name')).toBe(names('بجلی', 'Electricity'));
      expect(cell('1', 'expense')).toBe('3,000');
      expect(cell('2', 'section')).toBe(m('report-section-partner-money'));
      expect(cell('3', 'name')).toBe(names('الف', 'Partner A'));
      expect(cell('3', 'taken')).toBe('2,000');
      expect(totalsCell('expense')).toBe('3,000');
      expect(totalsCell('taken')).toBe('2,000');
      expect(totalsCell('returned')).toBe('0');
      expect(drawnColumns()).toEqual(['section', 'name', 'expense', 'taken', 'returned']);
    });

    it('starts on all partners, asks with no partner, and lists the partners in this language', async () => {
      const { user } = await openReport('R29');
      const box = screen.getByRole('combobox', { name: m('report-filter-partner') });
      await waitFor(() => {
        expect(box).toHaveTextContent(m('report-partner-all'));
      }, SLOW_MS);
      expect(lastRun()?.filters).toEqual(
        sentLike(R29_DAY_REQUEST, { shortcut: null, from: '', to: '' }),
      );
      await user.click(box);
      for (const partner of PARTNERS) {
        expect(
          await screen.findByRole('option', { name: names(partner.name_ur, partner.name_en) }),
        ).toBeInTheDocument();
      }
      expect(screen.getByRole('option', { name: m('report-partner-all') })).toBeInTheDocument();
    });

    it('asks for one partner when he is chosen, and for all again when “all partners” is', async () => {
      const { user } = await openReport('R29');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      await pickPartner(user, names('الف', 'Partner A'));
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(
          sentLike(R29_ALIF_REQUEST, { shortcut: null, from: '', to: '' }),
        );
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      expect(totalsCell('taken')).toBe('2,000');
      await pickPartner(user, m('report-partner-all'));
      await waitFor(() => {
        expect(lastRun()?.filters.partner_id).toBeNull();
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
    });

    it('says in words under the box when the partners could not be opened, and still shows the report', async () => {
      await openReport('R29');
      vi.mocked(partnersApi.partnersList).mockRejectedValue(fail({ code: 'ERR_INTERNAL' }));
      // A fresh visit with the list failing.
      leaveAppWindow();
      await renderApp('/reports/R29');
      await screen.findByText(named(m('cashbook-partners-failed')), undefined, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
    });
  });

  describe('R31 the partner statement', () => {
    it('cannot run without a partner: it says to pick one, asks nothing, and offers no print', async () => {
      await openReport('R31');
      expect(await screen.findByText(named(m('report-pick-partner')))).toBeVisible();
      expect(screen.getByText(named(m('report-pick-partner-help')))).toBeVisible();
      expect(reportsApi.reportsRun).not.toHaveBeenCalled();
      expect(screen.getByRole('button', { name: named(m('report-preview')) })).toBeDisabled();
      // R31 has no “all partners” choice: that would be a statement of nobody.
      const { user } = { user: userEvent.setup() };
      await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
      await screen.findByRole('option', { name: names('الف', 'Partner A') });
      expect(screen.queryByRole('option', { name: m('report-partner-all') })).toBeNull();
    });

    it('asks for the partner chosen and draws his dated lines as sentences with the running total', async () => {
      const { user } = await openReport('R31');
      await pickPartner(user, names('ب', 'Partner B'));
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(
          sentLike(R31_BAY_REQUEST, { shortcut: null, from: '', to: '' }),
        );
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      expect(cell('0', 'date')).toBe('10-10-2026');
      expect(cell('0', 'heading')).toBe(m('partners-statement-heading-bikri'));
      expect(cell('0', 'detail')).toBe(
        names('بکری 1: کمیشن، لاگا اور فون', 'Sale 1: Commission, Laga and Phone Charge'),
      );
      expect(cell('2', 'detail')).toBe(
        names('دکان خرچہ خ-1: بجلی', 'Shop Expense E-1: Electricity'),
      );
      expect(cell('3', 'detail')).toBe(names('سرمایہ لگایا س-1', 'Capital put in CI-1'));
      // A share is plus, an expense share is minus, and the running total follows (FR-17.9).
      expect(cell('0', 'amount')).toBe('1,184');
      expect(cell('2', 'amount')).toBe('-900');
      expect(cell('3', 'running')).toBe('50,746');
      expect(totalsCell('running')).toBe('50,746');
      expect(drawnColumns()).toEqual(['date', 'heading', 'detail', 'amount', 'running']);
      // No sentence is left with a hole in it: no variable, no raw message ID.
      const body = visible(document.querySelector('.ag-center-cols-container')?.textContent);
      expect(body).not.toMatch(/[{}$]|partners-statement|vouchers-number/);
    });

    it('shows the opening, the plus, the minus and the closing total above the table', async () => {
      const { user } = await openReport('R31');
      await pickPartner(user, names('ب', 'Partner B'));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      for (const id of [
        'partners-statement-opening',
        'partners-statement-plus',
        'partners-statement-minus',
        'partners-statement-closing',
      ]) {
        expect(screen.getByText(named(m(id)))).toBeInTheDocument();
      }
      expect(visible(document.body.textContent)).toContain('51,646');
    });

    it('opens on the partner of the address, and the box shows him', async () => {
      await openReport('R31', {}, '?partner=2');
      await waitFor(() => {
        expect(lastRun()?.filters.partner_id).toBe(2);
      }, SLOW_MS);
      expect(screen.getByRole('combobox', { name: m('report-filter-partner') })).toHaveTextContent(
        names('ب', 'Partner B'),
      );
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
    });

    it('re-words the sentences and the partner’s name in the other language and asks nothing again', async () => {
      await openReport('R31', {}, '?partner=2');
      await waitFor(() => {
        expect(cell('0', 'detail')).toBe(
          names('بکری 1: کمیشن، لاگا اور فون', 'Sale 1: Commission, Laga and Phone Charge'),
        );
      }, SLOW_MS);
      const before = runCount();
      switchLanguage();
      const inOther = (ur: string, en: string) => (other === 'ur' ? ur : en);
      await waitFor(() => {
        expect(cell('0', 'detail')).toBe(
          inOther('بکری 1: کمیشن، لاگا اور فون', 'Sale 1: Commission, Laga and Phone Charge'),
        );
      }, SLOW_MS);
      expect(cell('2', 'detail')).toBe(
        inOther('دکان خرچہ خ-1: بجلی', 'Shop Expense E-1: Electricity'),
      );
      expect(
        screen.getByRole('combobox', { name: message(other, 'report-filter-partner') }),
      ).toHaveTextContent(inOther('ب', 'Partner B'));
      expect(runCount()).toBe(before);
    });

    it('shows the print with the same sentences, in the print language, and asks with the partner', async () => {
      const { user } = await openReport('R31', { print: PRINT_R31_BAY_EN }, '?partner=2');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await waitFor(() => {
        expect(reportsApi.reportsPrintData).toHaveBeenCalled();
      }, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
        id: 'R31',
        language,
        paper: 'a4',
      });
      expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0].filters.partner_id).toBe(2);
      await screen.findByText(PRINT_R31_BAY_EN.firm.name, undefined, SLOW_MS);
      const body = visible(document.querySelector('.print-report tbody')?.textContent);
      expect(body).toContain('Sale 1: Commission, Laga and Phone Charge');
      expect(body).toContain('Shop Expense E-1: Electricity');
      expect(body).toContain('Capital put in CI-1');
      expect(body).not.toMatch(/[{}$]/);
    });

    it('says there is nothing, and what to do, when the partner has no lines', async () => {
      await openReport(
        'R31',
        { pages: { R31: { ...R31_BAY, rows: [], total_count: 0, summary: [] } } },
        '?partner=2',
      );
      expect(await screen.findByText(named(m('report-empty')), undefined, SLOW_MS)).toBeVisible();
    });

    it('says what happened and tries again when the backend fails', async () => {
      let failing = true;
      const { user } = await openReport(
        'R31',
        {
          refuse: () => {
            if (!failing) return null;
            failing = false;
            return {
              ok: false,
              errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
              ref: 'E-1',
            } as never;
          },
        },
        '?partner=2',
      );
      await screen.findByText(named(m('report-load-failed')), undefined, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('common-retry')) }));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
    });
  });

  describe('R32 the money lent report', () => {
    it('asks as of the day typed and shows the backend’s lent, returned and open amounts', async () => {
      const { user } = await openReport('R32');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      const day = screen.getByLabelText(m('report-filter-as-of'));
      await user.click(day);
      await user.keyboard('16-10-2026');
      await waitFor(() => {
        expect(lastRun()?.filters).toEqual(sentLike(R32_E2E_30B_REQUEST));
      }, SLOW_MS);
      expect(cell('0', 'person')).toContain('01030001');
      expect(cell('0', 'lent')).toBe('5,000');
      expect(cell('0', 'returned_cash')).toBe('3,000');
      expect(cell('0', 'returned_salary')).toBe('0');
      expect(cell('0', 'open')).toBe('2,000');
      expect(cell('0', 'return_by')).toBe('15-10-2026');
      expect(cell('0', 'days_past')).toBe('1');
      expect(totalsCell('lent')).toBe('5,000');
      expect(totalsCell('open')).toBe('2,000');
    });

    it('draws the loans past their return-by day as a line above the table', async () => {
      await openReport('R32');
      expect(
        await screen.findByText(named(m('report-r32-overdue')), undefined, SLOW_MS),
      ).toBeVisible();
      expect(visible(document.body.textContent)).toContain('2,000');
    });

    it('keeps the job and the oldest open day for the wide step', async () => {
      await openReport('R32');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(drawnColumns()).toEqual([
        'person',
        'lent',
        'returned_cash',
        'returned_salary',
        'open',
        'return_by',
        'days_past',
      ]);
    });

    it('has a box for the people who can borrow, not for any account', async () => {
      await openReport('R32');
      expect(
        screen.getByRole('combobox', { name: m('report-filter-borrower') }),
      ).toBeInTheDocument();
      expect(screen.queryByRole('combobox', { name: m('report-filter-party') })).toBeNull();
      expect(screen.getByLabelText(m('report-filter-as-of'))).toBeInTheDocument();
    });

    it('opens the ledger of the person behind a row with Enter', async () => {
      const { router, user } = await openReport('R32');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      const target = document.querySelector('.ag-row[row-id="0"] [col-id="person"]');
      if (target === null) throw new Error('no person cell');
      await user.click(target);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports/R13');
      }, SLOW_MS);
      expect(router.state.location.search).toMatchObject({ party: 32 });
    });
  });

  describe('keyboard only', () => {
    it('opens R31 from the list with ↓ and Enter, picks a partner from the box, and Esc leaves it', async () => {
      fakeReports();
      const { router } = await renderApp('/reports');
      await screen.findByRole('heading', { level: 1, name: named(m('reports-title')) });
      const user = userEvent.setup();
      // 7 Sales reports, 8 Accounts (R11 to R19 without R15), 6 Collections and R28 to R30 are in front
      // of R31: twenty-four steps down.
      await waitFor(() => {
        expect(document.activeElement?.getAttribute('href')).toMatch(/R2$/);
      }, SLOW_MS);
      await user.keyboard(`${'{ArrowDown}'.repeat(24)}{Enter}`);
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports/R31');
      }, SLOW_MS);
      expect(await screen.findByText(named(m('report-pick-partner')))).toBeVisible();
      // The first date has focus when the report opens (the period drop-down before it is a button).
      const from = screen.getByLabelText(m('report-filter-from'));
      await waitFor(() => {
        expect(document.activeElement).toBe(from);
      }, SLOW_MS);
      // Tab to the partner box, open it with Enter, move to Partner B with ↓ and take him with Enter.
      const box = screen.getByRole('combobox', { name: m('report-filter-partner') });
      for (let tabs = 0; tabs < 4; tabs += 1) {
        if (document.activeElement === box) break;
        await user.tab();
      }
      expect(document.activeElement).toBe(box);
      await user.keyboard('{Enter}');
      await screen.findByRole('option', { name: names('الف', 'Partner A') });
      await user.keyboard('{ArrowDown}{Enter}');
      await waitFor(() => {
        expect(lastRun()?.filters.partner_id).toBe(2);
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
});
