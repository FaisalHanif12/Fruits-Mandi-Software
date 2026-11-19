// The kharch page (plan 77, PRD FR-9.5) in the Urdu and the English app, against a fake backend with the backend's own
// answers of the analytics view M (worked day, October 2026): the shop's kharch per head (بجلی 3,000, total 3,000) and
// each partner's own kharch (الف 2,000, ب 0, ج 0, total 2,000), a period (a month by default, a year, one date), one
// partner, the partner part behind the partner code while the shop part shows, the entries behind a figure (date,
// entry, name, reason, amount) with a click on a line that opens its voucher, the print preview of one part, and the
// month buttons with the keyboard. The page and view M read the same thing (`analytics_run`, view M), so the screen
// adds nothing up: every figure is the backend's.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { UiError } from '../../src/bindings/UiError';
import * as analyticsApi from '../../src/features/analytics/api';
import { useAnalyticsStore } from '../../src/features/analytics/store';
import { LANGUAGES } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { fakeAnalytics } from '../helpers/analytics';
import { ANALYTICS_LINES_M_PARTNER_ALIF_REQUEST } from '../helpers/analytics-plan77';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners, partnerRow } from '../helpers/partners';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

const PARTNERS = [
  partnerRow(1, 'الف', 'Partner A', 4000),
  partnerRow(2, 'ب', 'Partner B', 3000),
  partnerRow(3, 'ج', 'Partner C', 3000),
];

const LOCKED: UiError = {
  ok: false,
  errors: [{ code: 'ERR_PARTNER_LOCKED', field: null, values: {} }],
  ref: 'E-20261010-0007',
};

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  window.localStorage.clear();
  healthyBackend();
  fakePartners(PARTNERS);
  act(() => {
    useAnalyticsStore.getState().reset();
  });
});

afterEach(() => {
  leaveAppWindow();
});

/** The request of one part: the shop's or the partners'. */
function readOf(part: 'shop' | 'partner') {
  return vi
    .mocked(analyticsApi.analyticsRun)
    .mock.calls.map((call) => call[0])
    .filter((input) => input.view === 'M' && input.filters.expense_part === part)
    .at(-1);
}

function section(part: 'shop' | 'partner'): Element {
  const found = document.querySelector(`[data-kharch-part="${part}"]`);
  if (found === null) throw new Error(`no ${part} part on screen`);
  return found;
}

function cell(part: 'shop' | 'partner', rowId: string, colId: string): string {
  const row = section(part).querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function totalsCell(part: 'shop' | 'partner', colId: string): string {
  const row = section(part).querySelector('.ag-row-pinned');
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

describe.each(LANGUAGES)('the kharch page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const heading = () =>
    screen.findByRole(
      'heading',
      { level: 1, name: (name) => visible(name) === m('kharch-title') },
      SLOW_MS,
    );
  const open = async (path = '/kharch') => {
    fakeAnalytics();
    const result = await renderApp(path);
    await heading();
    return { ...result, user: userEvent.setup() };
  };

  it('opens on the current month with the two parts: the shop’s kharch per head, and each partner’s own', async () => {
    await open();
    await waitFor(() => {
      expect(cell('shop', '0', 'head')).toContain(names('بجلی', 'Electricity'));
    }, SLOW_MS);
    // The shop part: بجلی 3,000 and the whole shop's total 3,000.
    expect(cell('shop', '0', 'p_2026_10')).toBe('3,000');
    expect(totalsCell('shop', 'total')).toBe('3,000');
    // The partner part: الف 2,000, ب 0, ج 0 and the total 2,000.
    await waitFor(() => {
      expect(cell('partner', '0', 'partner')).toContain(names('الف', 'Partner A'));
    }, SLOW_MS);
    expect(cell('partner', '0', 'own')).toBe('2,000');
    expect(cell('partner', '1', 'own')).toBe('0');
    expect(cell('partner', '2', 'own')).toBe('0');
    expect(totalsCell('partner', 'own')).toBe('2,000');
    // Each part has its chart, with its own words: the shop's by period, the partners' own kharch.
    expect(
      await screen.findByRole(
        'img',
        { name: (name) => visible(name).startsWith(m('analytics-chart-m-partner')) },
        SLOW_MS,
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('img', { name: (name) => visible(name).startsWith(m('analytics-chart-m')) }),
    ).toBeInTheDocument();
    // Each part has its own name as a heading, from the words the print header uses too.
    expect(visible(section('shop').textContent)).toContain(m('analytics-filter-expense-shop'));
    expect(visible(section('partner').textContent)).toContain(
      m('analytics-filter-expense-partner'),
    );
    // One read for each part, of the month so far (today is 10-10-2026), by month (the backend's default).
    expect(readOf('shop')).toMatchObject({
      view: 'M',
      filters: { from: '2026-10-01', to: '2026-10-10', group_by: null, partner_id: null },
    });
    expect(readOf('partner')).toMatchObject({
      view: 'M',
      filters: { from: '2026-10-01', to: '2026-10-10', group_by: null, partner_id: null },
    });
  });

  it('a partner chosen narrows the partner part only; the shop part never takes him', async () => {
    const { user } = await open();
    await screen.findByRole('combobox', { name: m('report-filter-partner') }, SLOW_MS);
    await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
    await user.click(await screen.findByRole('option', { name: names('الف', 'Partner A') }));
    await waitFor(() => {
      expect(readOf('partner')?.filters.partner_id).toBe(1);
    }, SLOW_MS);
    expect(readOf('shop')?.filters.partner_id).toBeNull();
  });

  it('a year reads the year so far, by month', async () => {
    await open('/kharch?period=year&year=2026');
    await waitFor(() => {
      expect(readOf('shop')).toMatchObject({
        filters: { from: '2026-01-01', to: '2026-10-10', group_by: null },
      });
    }, SLOW_MS);
  });

  it('one date reads that day, by day', async () => {
    await open('/kharch?period=day&date=2026-10-09');
    await waitFor(() => {
      expect(readOf('shop')).toMatchObject({
        filters: { from: '2026-10-09', to: '2026-10-09', group_by: 'day' },
      });
    }, SLOW_MS);
  });

  it('while the partner code is not entered the partner part asks for it and the shop part still shows', async () => {
    fakeAnalytics({
      refuse: (input) => (input.filters.expense_part === 'partner' ? LOCKED : null),
    });
    await renderApp('/kharch');
    await heading();
    await waitFor(() => {
      expect(cell('shop', '0', 'head')).toContain(names('بجلی', 'Electricity'));
    }, SLOW_MS);
    expect(visible(section('partner').textContent)).toContain(m('partners-code-box-text'));
    // No partner figure is on screen while it is locked.
    expect(visible(section('partner').textContent)).not.toContain('2,000');
  });

  it('Enter on a partner’s figure opens his entries, a line opens its voucher, and Esc returns to the two parts', async () => {
    const { user, router } = await open();
    await waitFor(() => {
      expect(cell('partner', '0', 'own')).toBe('2,000');
    }, SLOW_MS);
    const figure = section('partner').querySelector('.ag-row[row-id="0"] [col-id="own"]');
    if (figure === null) throw new Error('no figure');
    await user.click(figure);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(vi.mocked(analyticsApi.analyticsLines).mock.calls.at(-1)?.[0]).toMatchObject({
        view: 'M',
        filters: { expense_part: 'partner' },
        row_key: ANALYTICS_LINES_M_PARTNER_ALIF_REQUEST.row_key,
        column_key: ANALYTICS_LINES_M_PARTNER_ALIF_REQUEST.column_key,
      });
    }, SLOW_MS);
    // The entry: his personal expense of 2,000 with its reason.
    const lineCell = (colId: string) =>
      visible(
        document.querySelector(`[data-analytics-lines] .ag-row[row-id="0"] [col-id="${colId}"]`)
          ?.textContent,
      );
    await waitFor(() => {
      expect(lineCell('amount')).toBe('2,000');
    }, SLOW_MS);
    expect(lineCell('name')).toContain(names('الف', 'Partner A'));
    expect(lineCell('reason')).toBe('گھر کا خرچ');
    // A line opens the voucher it comes from (Enter on the line).
    const line = document.querySelector(
      '[data-analytics-lines] .ag-row[row-id="0"] [col-id="amount"]',
    );
    if (line === null) throw new Error('no line');
    await user.click(line);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/voucher');
    });
    expect(router.state.location.search).toMatchObject({ id: 5 });
  });

  it('Esc leaves the entries for the two parts', async () => {
    const { user } = await open();
    await waitFor(() => {
      expect(cell('shop', '0', 'total')).toBe('3,000');
    }, SLOW_MS);
    const figure = section('shop').querySelector('.ag-row[row-id="0"] [col-id="total"]');
    if (figure === null) throw new Error('no figure');
    await user.click(figure);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(document.querySelector('[data-analytics-lines]')).not.toBeNull();
    }, SLOW_MS);
    press('Escape');
    await waitFor(() => {
      expect(document.querySelector('[data-kharch-part="shop"]')).not.toBeNull();
    }, SLOW_MS);
  });

  it('the print preview of the partner part asks the backend for that part with the same filters', async () => {
    const { user } = await open();
    await waitFor(() => {
      expect(cell('partner', '0', 'own')).toBe('2,000');
    }, SLOW_MS);
    const buttons = Array.from(
      section('partner').querySelectorAll<HTMLButtonElement>('button'),
    ).filter((button) => visible(button.textContent).includes(m('report-preview')));
    const preview = buttons[0];
    if (preview === undefined) throw new Error('no print button in the partner part');
    await user.click(preview);
    await waitFor(() => {
      expect(vi.mocked(analyticsApi.analyticsPrintData).mock.calls.at(-1)?.[0]).toMatchObject({
        view: 'M',
        filters: { expense_part: 'partner', from: '2026-10-01', to: '2026-10-10' },
      });
    }, SLOW_MS);
  });

  it('Page Up goes to the month before with the keyboard, and keeps it in the address', async () => {
    const { router } = await open();
    await waitFor(() => {
      expect(cell('shop', '0', 'head')).toContain(names('بجلی', 'Electricity'));
    }, SLOW_MS);
    press('PageUp');
    await waitFor(() => {
      expect(readOf('shop')).toMatchObject({
        filters: { from: '2026-09-01', to: '2026-09-30' },
      });
    }, SLOW_MS);
    expect(router.state.location.search).toMatchObject({ month: '2026-09' });
  });
});
