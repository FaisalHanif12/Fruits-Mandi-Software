// R38, the dead amount (plan 40, PRD FR-10.10, E71, E73, E2E-44): the screen the tile ڈوبی رقم opens.
// After ارشد's move (11-10-26) and his son's 4,000 (12-10-26) it shows owed then 18,684, recovered
// 4,000, still owed 14,684; the totals are the backend's. The filters, the four states, the way to
// his ledger and "make active again" (all by keyboard) are checked in the Urdu and the English app.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import * as mastersApi from '../../src/features/masters/api';
import type { DeadRow } from '../../src/bindings/DeadRow';
import { DEAD_COLUMNS } from '../../src/features/reports/dead-columns';
import { LANGUAGES, type Language } from '../../src/i18n';
import { formatPaisa, formatPaisaValue } from '../../src/shared/format';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  ARSHAD_DEAD,
  ARSHAD_ROW,
  PAGE_ARSHAD,
  PAGE_BOTH,
  SHAHID_ROW,
  deadPage,
} from '../helpers/dead';
import { PAGE_ROWS } from '../helpers/pager';
import { fakeReports } from '../helpers/reports';
import { PRINT_R18_EN } from '../helpers/reports-real';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };
const KEY_CHIPS = new Set(['F9', 'Esc']);

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(reportsApi.reportsDeadAmount).mockResolvedValue(PAGE_ARSHAD);
  vi.mocked(mastersApi.deadUndo).mockResolvedValue({ ok: true });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('R38, the dead amount (%s)', (language: Language) => {
  const m = (id: string, args: Record<string, string | number> = {}) => message(language, id, args);
  /** A name that is exactly `text`, or `text` followed by the key chip of its button (PRD FR-7.4). */
  const named = (text: string) => (name: string) => {
    const seen = visible(name);
    return (
      seen === text || (seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim()))
    );
  };

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const view = await renderApp('/reports/dead-amount');
    await screen.findByRole('heading', { level: 1, name: named(m('dead-report-title')) }, SLOW_MS);
    return view;
  }

  const rowOf = (id: number) => document.querySelector(`.ag-row[row-id="${String(id)}"]`);
  const cell = (id: number, col: string) =>
    visible(rowOf(id)?.querySelector(`[col-id="${col}"]`)?.textContent);
  const pinned = (col: string) =>
    visible(
      document.querySelector('.ag-row-pinned')?.querySelector(`[col-id="${col}"]`)?.textContent,
    );

  it('asks for the people still owing first, one page at a time', async () => {
    await open();
    await waitFor(() => {
      expect(reportsApi.reportsDeadAmount).toHaveBeenCalledWith({
        filter: 'owed',
        search: '',
        offset: 0,
        limit: PAGE_ROWS,
      });
    });
  });

  it('shows what he owed then, what was recovered and what he still owes, E2E-44', async () => {
    await open();
    await waitFor(() => {
      expect(cell(1, 'still_owed')).toBe('14,684');
    }, SLOW_MS);
    expect(cell(1, 'party')).toBe(language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani');
    expect(cell(1, 'code')).toBe('0021');
    expect(cell(1, 'date')).toBe('11-10-2026');
    expect(cell(1, 'owed_then')).toBe('18,684');
    expect(cell(1, 'recovered')).toBe('4,000');
    expect(cell(1, 'status')).toBe(m('dead-status-owing'));
    // The totals row is the backend's, over the whole filter.
    expect(pinned('owed_then')).toBe('18,684');
    expect(pinned('recovered')).toBe('4,000');
    expect(pinned('still_owed')).toBe('14,684');
  });

  it('the filter "all" also lists a person fully recovered, a one-time person by the name on his lines', async () => {
    vi.mocked(reportsApi.reportsDeadAmount).mockImplementation((input) =>
      Promise.resolve(input.filter === 'all' ? PAGE_BOTH : PAGE_ARSHAD),
    );
    await open();
    const user = userEvent.setup();
    await waitFor(() => {
      expect(cell(1, 'still_owed')).toBe('14,684');
    }, SLOW_MS);
    expect(rowOf(2)).toBeNull();
    await user.click(await screen.findByRole('combobox', { name: named(m('dead-report-filter')) }));
    await user.click(await screen.findByRole('option', { name: named(m('dead-filter-all')) }));
    await waitFor(() => {
      expect(cell(2, 'party')).toBe(language === 'ur' ? 'شاہد' : 'Shahid');
    }, SLOW_MS);
    expect(cell(2, 'status')).toBe(m('dead-status-recovered'));
    expect(cell(2, 'still_owed')).toBe('0');
    expect(reportsApi.reportsDeadAmount).toHaveBeenLastCalledWith(
      expect.objectContaining({ filter: 'all' }),
    );
  });

  it('the search box is sent as typed, in either script', async () => {
    await open();
    const user = userEvent.setup();
    await user.click(await screen.findByRole('textbox', { name: named(m('common-search-label')) }));
    await user.keyboard('arshad');
    await waitFor(() => {
      expect(reportsApi.reportsDeadAmount).toHaveBeenLastCalledWith(
        expect.objectContaining({ search: 'arshad' }),
      );
    });
  });

  it('"make active again" waits for a person to be chosen, then asks for a reason and ends the move', async () => {
    await open();
    const user = userEvent.setup();
    const button = await screen.findByRole('button', { name: named(m('dead-undo-button')) });
    expect(button).toBeDisabled();
    await waitFor(() => {
      expect(cell(1, 'still_owed')).toBe('14,684');
    }, SLOW_MS);
    // The keyboard: the first cell of the row takes focus, and the button is then on.
    const first = rowOf(1)?.querySelector<HTMLElement>('[col-id="party"]');
    if (first === null || first === undefined) throw new Error('no first cell');
    await user.click(first);
    await waitFor(() => {
      expect(button).toBeEnabled();
    });
    await user.click(button);
    const dialog = await screen.findByRole('alertdialog');
    await user.type(within(dialog).getByRole('combobox'), 'Paid in full');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(mastersApi.deadUndo).toHaveBeenCalledWith({
        mark_id: ARSHAD_DEAD.mark_id,
        reason: 'Paid in full',
      });
    });
  });

  it('Enter on a person opens his ledger, R13, with his account in the address', async () => {
    const { router } = await open();
    await waitFor(() => {
      expect(cell(1, 'still_owed')).toBe('14,684');
    }, SLOW_MS);
    const user = userEvent.setup();
    const first = rowOf(1)?.querySelector<HTMLElement>('[col-id="party"]');
    if (first === null || first === undefined) throw new Error('no first cell');
    await user.click(first);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R13');
    });
    expect(router.state.location.search).toMatchObject({ party: ARSHAD_ROW.account_id });
  });

  it('F9 shows the print preview of R38 for the people still owing, from the report engine', async () => {
    fakeReports();
    vi.mocked(reportsApi.reportsPrintData).mockResolvedValue(PRINT_R18_EN);
    await open();
    await waitFor(() => {
      expect(cell(1, 'still_owed')).toBe('14,684');
    }, SLOW_MS);
    press('F9');
    await screen.findByRole('heading', { level: 2, name: named(m('report-print-title')) }, SLOW_MS);
    await waitFor(() => {
      expect(reportsApi.reportsPrintData).toHaveBeenCalled();
    }, SLOW_MS);
    const asked = vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0];
    expect(asked).toMatchObject({ id: 'R38', language, paper: 'a4' });
    expect(asked?.filters.status).toBe('due');
  });

  it('with the filter "all" the print asks for every move: no status is sent', async () => {
    fakeReports();
    vi.mocked(reportsApi.reportsPrintData).mockResolvedValue(PRINT_R18_EN);
    vi.mocked(reportsApi.reportsDeadAmount).mockResolvedValue(PAGE_BOTH);
    await open();
    const user = userEvent.setup();
    await user.click(await screen.findByRole('combobox', { name: named(m('dead-report-filter')) }));
    await user.click(await screen.findByRole('option', { name: named(m('dead-filter-all')) }));
    await waitFor(() => {
      expect(cell(2, 'party')).not.toBe('');
    }, SLOW_MS);
    await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
    await waitFor(() => {
      expect(reportsApi.reportsPrintData).toHaveBeenCalled();
    }, SLOW_MS);
    const asked = vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0];
    expect(asked?.id).toBe('R38');
    expect(asked?.filters).not.toHaveProperty('status');
  });

  it('has the Excel button, and Esc closes the preview', async () => {
    fakeReports();
    vi.mocked(reportsApi.reportsPrintData).mockResolvedValue(PRINT_R18_EN);
    await open();
    await screen.findByRole('button', { name: named(message(language, 'export-excel')) }, SLOW_MS);
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
    const title = { level: 2 as const, name: named(m('report-print-title')) };
    await screen.findByRole('heading', title, SLOW_MS);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('heading', title)).toBeNull();
    }, SLOW_MS);
  });

  it('R38 in the catalogue, the search or a link lands on this screen', async () => {
    const { router } = await open();
    await router.navigate({ to: '/reports/$id', params: { id: 'R38' }, search: {} });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/dead-amount');
    });
  });

  it('nobody in the dead amount: says so and says what to do next', async () => {
    vi.mocked(reportsApi.reportsDeadAmount).mockResolvedValue(
      deadPage([], { owed: 0, recovered: 0, still: 0 }),
    );
    await open();
    await screen.findByText(
      (content) => visible(content) === m('dead-report-empty'),
      undefined,
      SLOW_MS,
    );
    expect(
      screen.getByText((content) => visible(content) === m('dead-report-empty-help')),
    ).toBeInTheDocument();
  });

  it('a filter that matches nobody offers to clear the filters', async () => {
    vi.mocked(reportsApi.reportsDeadAmount).mockImplementation((input) =>
      Promise.resolve(
        input.search === 'zzz' ? deadPage([], { owed: 0, recovered: 0, still: 0 }) : PAGE_ARSHAD,
      ),
    );
    await open();
    const user = userEvent.setup();
    await user.click(await screen.findByRole('textbox', { name: named(m('common-search-label')) }));
    await user.keyboard('zzz');
    await screen.findByText(
      (content) => visible(content) === m('dead-report-no-match'),
      undefined,
      SLOW_MS,
    );
    await user.click(screen.getByRole('button', { name: named(m('dead-report-clear')) }));
    await waitFor(() => {
      expect(cell(1, 'still_owed')).toBe('14,684');
    }, SLOW_MS);
  });

  it('a failure is an error with a retry, never an empty list', async () => {
    vi.mocked(reportsApi.reportsDeadAmount).mockRejectedValueOnce(fail({ code: 'ERR_NOT_FOUND' }));
    await open();
    const retry = await screen.findByRole(
      'button',
      { name: named(message(language, 'common-retry')) },
      SLOW_MS,
    );
    expect(
      screen.getByText((content) => visible(content) === m('dead-report-failed')),
    ).toBeInTheDocument();
    await userEvent.setup().click(retry);
    await waitFor(() => {
      expect(cell(1, 'still_owed')).toBe('14,684');
    }, SLOW_MS);
  });

  it('while the first page is on its way there is a loading state', async () => {
    vi.mocked(reportsApi.reportsDeadAmount).mockReturnValue(new Promise(() => undefined));
    await open();
    expect(
      screen.getAllByRole('status').some((one) => one.getAttribute('aria-busy') === 'true'),
    ).toBe(true);
  });

  it('keeps the filter and the search when the language is switched', async () => {
    await open();
    const user = userEvent.setup();
    await user.click(await screen.findByRole('textbox', { name: named(m('common-search-label')) }));
    await user.keyboard('arshad');
    await waitFor(() => {
      expect(reportsApi.reportsDeadAmount).toHaveBeenLastCalledWith(
        expect.objectContaining({ search: 'arshad' }),
      );
    });
    await user.keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      expect(document.documentElement.lang).toBe(language === 'ur' ? 'en' : 'ur');
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    const box = await screen.findByRole<HTMLInputElement>('textbox', {
      name: (name) => visible(name) === message(other, 'common-search-label'),
    });
    expect(box.value).toBe('arshad');
    await waitFor(() => {
      expect(cell(1, 'still_owed')).toBe('14,684');
    }, SLOW_MS);
  });
});

describe.each(LANGUAGES)('the columns of R38 (%s)', (language: Language) => {
  // The wide columns are not drawn at a narrow window, so their words are checked here, where the
  // grid asks for them, with the real messages and the formatting of the app.
  const context = {
    language,
    text: (id: string, vars?: Record<string, string>) => message(language, id, vars),
    money: (amount: Parameters<typeof formatPaisa>[0]) =>
      formatPaisa(amount, { grouping: 'international' }),
    signedMoney: (value: number) => formatPaisaValue(value, { grouping: 'international' }),
  };
  const value = (field: string, row: DeadRow) => {
    const column = DEAD_COLUMNS.find((one) => one.field === field);
    if (column === undefined) throw new Error(`no column ${field}`);
    return column.value(row, context);
  };

  it('says the type, the reason, the day of the last recovery and the status in words', () => {
    expect(value('type', ARSHAD_ROW)).toBe(message(language, 'dead-type-buyer'));
    expect(value('reason', ARSHAD_ROW)).toBe(message(language, 'dead-reason-died'));
    expect(value('last_recovery', ARSHAD_ROW)).toBe('12-10-2026');
    expect(value('status', ARSHAD_ROW)).toBe(message(language, 'dead-status-owing'));
    expect(value('type', SHAHID_ROW)).toBe(message(language, 'dead-type-pukhta'));
    expect(value('status', SHAHID_ROW)).toBe(message(language, 'dead-status-recovered'));
  });

  it('a one-time person is named by the name saved on his lines, not by the shared account', () => {
    expect(value('party', SHAHID_ROW)).toBe(language === 'ur' ? 'شاہد' : 'Shahid');
    expect(value('party', ARSHAD_ROW)).toBe(language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani');
  });

  it('shows a correction with its sign, and nothing for a recovery that has not happened', () => {
    expect(value('corrections', { ...ARSHAD_ROW, corrections_paisa: -50000 })).toBe('-500');
    expect(value('corrections', { ...ARSHAD_ROW, corrections_paisa: 120000 })).toBe('1,200');
    expect(value('last_recovery', { ...ARSHAD_ROW, last_recovery_date: null })).toBe('');
  });
});
