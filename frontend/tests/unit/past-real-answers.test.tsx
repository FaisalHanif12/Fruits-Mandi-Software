// پچھلی بکری drawn from the backend's own answers (plan 72 T1 and T2): the files the backend wrote in
// `bikri_past_*.json`, read as the types of the list, not canned data. Today in these files is
// 12-11-2026. What is checked: the window is asked for as the backend's request file has it (the days,
// `group_by_day`, the page); the day headings and the rows under them are the backend's days and rows, with
// the count and the خام بکری it sent; "show every بکری" brings the cancelled بکری with its day; an earlier
// window, an empty window and a refusal show in words; a window read again after an edit shows the new
// figures; the headings of a middle page are the days from its first row to its last. The screen adds up
// nothing: every figure is `formatPaisa` of a number the backend sent.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as bikriApi from '../../src/features/bikri/api';
import type { BikriList } from '../../src/bindings/BikriList';
import { headingDaysOnPage } from '../../src/features/bikri/past-window';
import { LANGUAGES } from '../../src/i18n';
import { DEFAULT_GROUPING, formatPaisa, type Paisa } from '../../src/shared/format';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { gridOf, named } from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import {
  PAST_FUTURE_REFUSED,
  PAST_TODAY,
  PAST_TODAY_ISO,
  PAST_WINDOW,
  PAST_WINDOW_AFTER_EDIT,
  PAST_WINDOW_ALL,
  PAST_WINDOW_BACK,
  PAST_WINDOW_BACK_REQUEST,
  PAST_WINDOW_EMPTY,
  PAST_WINDOW_REQUEST,
  PAST_WINDOW_SECOND_PAGE,
} from '../helpers/past-real';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  // The files were written on 12-11-2026.
  vi.mocked(appApi.dayStatus).mockResolvedValue({
    today: PAST_TODAY_ISO,
    closed_up_to: null,
    today_open: true,
  });
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** An amount of the file as the screen writes it: whole rupees with the app's grouping. */
const money = (amount: Paisa) => formatPaisa(amount, { grouping: DEFAULT_GROUPING, plain: false });

/** The value, or a clear failure when the file does not have it. */
function present<T>(value: T | undefined): T {
  if (value === undefined) throw new Error('not in the backend’s file');
  return value;
}

const windowCalls = () =>
  vi
    .mocked(bikriApi.bikriList)
    .mock.calls.map(([input]) => input)
    .filter((input) => input.date === null);

describe.each(LANGUAGES)('پچھلی بکری from the backend’s answers (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const answer = (window: BikriList, today: BikriList = PAST_TODAY) => {
    vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
      Promise.resolve(input.date === null ? window : today),
    );
  };
  const open = async (path = '/bikri/today') => {
    const view = await renderApp(path);
    await screen.findByRole('group', { name: named(m('bikri-past-nav-label')) });
    return { ...view, user: userEvent.setup() };
  };
  const windowGrid = () => gridOf(language, 'bikri-past-label');
  const shown = async () => {
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id^="group:"]')).not.toBeNull();
    });
  };
  const heading = (day: string) =>
    document.querySelector<HTMLElement>(`.ag-row[row-id="group:${day}"]`);
  /** The figures of a day heading: its words, then its figure, as drawn. */
  const figures = (day: string) =>
    Array.from(heading(day)?.querySelectorAll('[data-heading-chip]') ?? []).map((figure) =>
      visible(figure.textContent),
    );
  const rowIds = () =>
    Array.from(windowGrid().querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)'))
      .sort((a, b) => Number(a.getAttribute('row-index')) - Number(b.getAttribute('row-index')))
      .map((row) => row.getAttribute('row-id'));

  it('asks for the window as the backend’s request file has it', async () => {
    answer(PAST_WINDOW);
    await open();
    await waitFor(() => {
      expect(windowCalls()).toHaveLength(1);
    });
    // 05-11-2026 to 11-11-2026, with the days, the first page of 25.
    expect(windowCalls()[0]).toMatchObject(PAST_WINDOW_REQUEST);
  });

  it('every day of the file has its heading, with the count and the خام بکری it sent, and its rows under it', async () => {
    answer(PAST_WINDOW);
    await open();
    await shown();
    for (const day of PAST_WINDOW.days) {
      if (day.count === 0) {
        expect(visible(heading(day.date)?.textContent), day.date).toContain(
          m('bikri-past-day-empty'),
        );
      } else {
        expect(figures(day.date), day.date).toEqual([
          `${m('bikri-past-day-sales')}${String(day.count)}`,
          `${m('glossary-kham-bikri')}${money(day.gross_paisa)}`,
        ]);
      }
    }
    // Each heading is followed by the rows of its day, newest day first.
    const expected = PAST_WINDOW.days.flatMap((day) => [
      `group:${day.date}`,
      ...PAST_WINDOW.rows.filter((row) => row.entry_date === day.date).map((row) => String(row.id)),
    ]);
    // jsdom has no layout, so the grid draws the first rows only (its row buffer); the order of those
    // is what is checked, and the screenshots show every row.
    await waitFor(() => {
      expect(rowIds().length).toBeGreaterThanOrEqual(10);
    });
    expect(rowIds()).toEqual(expected.slice(0, rowIds().length));
  });

  it('the totals row is the backend’s over the whole window', async () => {
    answer(PAST_WINDOW);
    await open();
    await shown();
    expect(
      visible(windowGrid().querySelector('.ag-row-pinned [col-id="gross"]')?.textContent),
    ).toBe(money(PAST_WINDOW.totals.gross));
  });

  it('"show every بکری" brings the cancelled one, under its day, with the figures the backend sent', async () => {
    answer(PAST_WINDOW);
    const { user } = await open();
    await shown();
    answer(PAST_WINDOW_ALL);
    await user.click(
      await screen.findByRole('checkbox', { name: named(m('bikri-past-show-all')) }),
    );
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id="6"]')).not.toBeNull();
    });
    expect(
      visible(windowGrid().querySelector('.ag-row[row-id="6"] [col-id="status"]')?.textContent),
    ).toBe(m('bikri-status-cancelled'));
    // The day counts the cancelled بکری and leaves its money out, as the file says.
    const day = PAST_WINDOW_ALL.days.find((entry) => entry.date === '2026-11-08');
    expect(figures('2026-11-08')[0]).toBe(`${m('bikri-past-day-sales')}${String(day?.count)}`);
    expect(windowCalls().at(-1)).toMatchObject({ all: true, group_by_day: true });
  });

  it('an earlier window is asked for as the backend’s file has it, and its days are drawn', async () => {
    answer(PAST_WINDOW);
    const { user } = await open();
    await shown();
    answer(PAST_WINDOW_BACK);
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(m('bikri-past-back', { days: 7 })),
      }),
    );
    await waitFor(() => {
      expect(windowCalls().at(-1)).toMatchObject(PAST_WINDOW_BACK_REQUEST);
    });
    await waitFor(() => {
      expect(heading('2026-10-29')).not.toBeNull();
    });
    expect(rowIds()[0]).toBe('group:2026-11-04');
  });

  it('a window with nothing says so in words, and the divider and the buttons are still there', async () => {
    answer(PAST_WINDOW_EMPTY);
    await open();
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(m('bikri-past-window-empty'));
    });
    expect(
      screen.getByRole('group', { name: named(m('bikri-past-nav-label')) }),
    ).toBeInTheDocument();
  });

  it('a window the backend refuses says it in the backend’s words, with the day it names', async () => {
    const refusal = PAST_FUTURE_REFUSED.errors[0];
    if (refusal === undefined) throw new Error('no refusal in the file');
    vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
      input.date === null
        ? Promise.reject(fail({ code: refusal.code, field: refusal.field, values: refusal.values }))
        : Promise.resolve(PAST_TODAY),
    );
    await open();
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(m('bikri-past-failed'));
    });
    expect(visible(document.body.textContent)).toContain(
      m('err-date-in-future', { today: String(refusal.values['today']) }),
    );
  });

  it('read again after an edit, the window shows the new figures: the row, its day and the totals', async () => {
    answer(PAST_WINDOW);
    const { router } = await open('/bikri/today?row=7');
    await shown();
    const grossOf = (list: BikriList, day: string) =>
      money(present(list.days.find((entry) => entry.date === day)).gross_paisa);
    const rowGross = (list: BikriList, id: number) =>
      money(present(list.rows.find((row) => row.id === id)).gross);
    expect(figures('2026-11-09')[1]).toBe(
      `${m('glossary-kham-bikri')}${grossOf(PAST_WINDOW, '2026-11-09')}`,
    );
    // The edit changed one بکری of 09-11-2026; the way back asks the backend again.
    answer(PAST_WINDOW_AFTER_EDIT);
    await router.navigate({ to: '/bikri/book', search: { id: 7, past: '2026-11-11' } });
    await router.navigate({ to: '/bikri/today', search: { end: '2026-11-11', row: 7 } });
    await waitFor(() => {
      expect(figures('2026-11-09')[1]).toBe(
        `${m('glossary-kham-bikri')}${grossOf(PAST_WINDOW_AFTER_EDIT, '2026-11-09')}`,
      );
    });
    expect(
      visible(windowGrid().querySelector('.ag-row[row-id="7"] [col-id="gross"]')?.textContent),
    ).toBe(rowGross(PAST_WINDOW_AFTER_EDIT, 7));
    expect(
      visible(windowGrid().querySelector('.ag-row-pinned [col-id="gross"]')?.textContent),
    ).toBe(money(PAST_WINDOW_AFTER_EDIT.totals.gross));
  });
});

describe('the headings of a middle page of the backend’s window', () => {
  it('are the days from its first row to its last row, with the empty days between', () => {
    const days = PAST_WINDOW_SECOND_PAGE.days.map((day) => day.date);
    const rowDays = PAST_WINDOW_SECOND_PAGE.rows.map((row) => row.entry_date);
    // Page 2 of 3 holds the rows of 06-11-2026 and 05-11-2026: those two days, no more.
    expect(headingDaysOnPage(days, rowDays, false, false)).toEqual(['2026-11-06', '2026-11-05']);
  });
});
