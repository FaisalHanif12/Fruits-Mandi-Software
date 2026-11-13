// پچھلی بکری / Past Sales: the days of the window are closed by default (plan 75 7.7, PRD FR-12.46 v1.39),
// in the Urdu and the English app, against a fake backend. Each earlier day is one closed line with its
// date, count and خام بکری; its arrow (a click, Enter or Space) opens its بکری under it and closes it again;
// several days can be open; a day with no بکری has no arrow; a move of the window closes every day; "تاریخ
// پر جائیں" opens and lights the day; a search or a filter, and "show every بکری", open the days with a
// match; the way back from an edit has its day open with its row selected; a language switch keeps what is
// open; opening a day asks the backend once. Today in these tests is 10-10-2026, so the first window is
// 03-10-2026 to 09-10-2026. The counts and totals are what the double is given; the screen adds up nothing.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { listRow, savedEvent1 } from '../helpers/bikri-saved';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';
import { paisaForTest } from '../helpers/paisa';
import { pastAnswer, type PastFake } from '../helpers/past';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** Two بکری on 09-10-2026, one on 07-10-2026 and one on 05-10-2026. */
const WINDOW_ROWS = [
  listRow({ id: 62, number: 3, entry_date: '2026-10-09', truck_no: null }),
  listRow({ id: 61, number: 2, entry_date: '2026-10-09', truck_no: 'LES-1234' }),
  listRow({ id: 71, number: 1, entry_date: '2026-10-07', truck_no: null }),
  listRow({ id: 72, number: 4, entry_date: '2026-10-05', truck_no: null }),
];
const WINDOW_GROSS = { '2026-10-09': 41_400, '2026-10-07': 12_000, '2026-10-05': 9_000 };

function fakePast(fake: PastFake = {}) {
  vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
    Promise.resolve(pastAnswer(input, fake)),
  );
}

/** The calls that ask for one earlier day (a `date` that is not today), oldest first. */
const dayCalls = (display: string) =>
  vi
    .mocked(bikriApi.bikriList)
    .mock.calls.map(([input]) => input)
    .filter((input) => input.date === display);
const windowCalls = () =>
  vi
    .mocked(bikriApi.bikriList)
    .mock.calls.map(([input]) => input)
    .filter((input) => input.date === null);

function present<T>(value: T | null | undefined): T {
  if (value === null || value === undefined) throw new Error('not on screen');
  return value;
}

describe.each(LANGUAGES)('پچھلی بکری, the days are closed by default (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
  });

  const dayLine = (day: string) =>
    document.querySelector<HTMLElement>(`[data-day="${day}"] [data-day-line]`);
  const dayBox = (day: string) =>
    present(document.querySelector<HTMLElement>(`[data-day="${day}"]`));
  const dayGrid = (day: string) =>
    within(dayBox(day)).getByRole('group', { name: named(message(language, 'bikri-past-label')) });
  const isOpen = (day: string) => present(dayLine(day)).getAttribute('aria-expanded') === 'true';
  const open = async (path = '/bikri/today') => {
    const view = await renderApp(path);
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'layout-module-today')),
    });
    await waitFor(() => {
      expect(document.querySelector('[data-day-line]')).not.toBeNull();
    });
    return { ...view, user: userEvent.setup() };
  };
  const waitOpen = async (day: string) => {
    await waitFor(() => {
      expect(isOpen(day)).toBe(true);
    });
  };
  const rowsOf = (day: string) =>
    Array.from(dayBox(day).querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map((row) =>
      row.getAttribute('row-id'),
    );

  it('every day with a بکری is closed when the screen opens, and no rows of the window are drawn', async () => {
    await open();
    for (const day of ['2026-10-09', '2026-10-07', '2026-10-05']) {
      expect(isOpen(day)).toBe(false);
    }
    expect(document.querySelector('.ag-row[row-id="62"]')).toBeNull();
    expect(document.querySelector('.ag-row[row-id="71"]')).toBeNull();
    // Today's section is open, as always.
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="41"]')).not.toBeNull();
    });
    // The rows of a closed day were never asked for.
    expect(dayCalls('09-10-2026')).toHaveLength(0);
  });

  it('a click on the line opens the day with its own rows, and a second click closes it', async () => {
    const { user } = await open();
    await user.click(present(dayLine('2026-10-09')));
    await waitOpen('2026-10-09');
    await waitFor(() => {
      expect(rowsOf('2026-10-09')).toEqual(['62', '61']);
    });
    await user.click(present(dayLine('2026-10-09')));
    await waitFor(() => {
      expect(isOpen('2026-10-09')).toBe(false);
    });
    expect(document.querySelector('.ag-row[row-id="62"]')).toBeNull();
  });

  it('Enter and Space on a line open and close the day', async () => {
    const { user } = await open();
    present(dayLine('2026-10-07')).focus();
    await user.keyboard('{Enter}');
    await waitOpen('2026-10-07');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(isOpen('2026-10-07')).toBe(false);
    });
    await user.keyboard(' ');
    await waitOpen('2026-10-07');
    await user.keyboard(' ');
    await waitFor(() => {
      expect(isOpen('2026-10-07')).toBe(false);
    });
  });

  it('several days can be open together, and each shows only its own rows', async () => {
    const { user } = await open();
    await user.click(present(dayLine('2026-10-09')));
    await user.click(present(dayLine('2026-10-07')));
    await waitFor(() => {
      expect(rowsOf('2026-10-09')).toEqual(['62', '61']);
      expect(rowsOf('2026-10-07')).toEqual(['71']);
    });
    expect(isOpen('2026-10-09')).toBe(true);
    expect(isOpen('2026-10-07')).toBe(true);
    expect(isOpen('2026-10-05')).toBe(false);
  });

  it('opening a day asks the backend for that day once, with the filters, and shows the backend’s totals', async () => {
    const { user } = await open();
    await user.click(present(dayLine('2026-10-09')));
    await waitFor(() => {
      expect(rowsOf('2026-10-09')).toEqual(['62', '61']);
    });
    expect(dayCalls('09-10-2026')).toHaveLength(1);
    expect(dayCalls('09-10-2026')[0]).toMatchObject({
      from: null,
      to: null,
      all: false,
      group_by_day: false,
    });
    expect(dayGrid('2026-10-09').querySelector('.ag-row-pinned [col-id="sellers"]')).not.toBeNull();
  });

  it('an open day shows the backend’s totals for that day, a cancelled بکری left out, and the window’s own totals stay as they were', async () => {
    // 07-10-2026 has a saved بکری (net 14,500) and a cancelled one (net 14,400): the backend's totals for the day
    // count the saved one only. The window's totals are the whole week's.
    fakePast({
      window: [
        ...WINDOW_ROWS,
        listRow({
          id: 73,
          number: 5,
          entry_date: '2026-10-07',
          status: 'cancelled',
          gross: paisaForTest(1_440_000),
          net: paisaForTest(1_440_000),
        }),
      ],
      gross: WINDOW_GROSS,
      windowTotals: {
        gross: paisaForTest(6_240_000),
        net: paisaForTest(5_100_000),
        buyer_total: paisaForTest(6_700_000),
      },
      dayTotals: {
        '2026-10-07': {
          qty_received: 50,
          qty_sold: 50,
          gross: paisaForTest(1_750_000),
          net: paisaForTest(1_450_000),
          buyer_total: paisaForTest(1_850_000),
        },
      },
    });
    const { user } = await open();
    await user.click(present(dayLine('2026-10-07')));
    await waitFor(() => {
      expect(rowsOf('2026-10-07')).toContain('73');
    });
    const total = (field: string) =>
      visible(
        dayGrid('2026-10-07').querySelector(`.ag-row-pinned [col-id="${field}"]`)?.textContent,
      );
    expect(total('gross')).toBe('17,500');
    expect(total('net')).toBe('14,500');
    // The cancelled بکری is in the day's rows, marked, and not in its totals.
    expect(
      visible(
        dayGrid('2026-10-07').querySelector('.ag-row[row-id="73"] [col-id="status"]')?.textContent,
      ),
    ).toBe(message(language, 'bikri-status-cancelled'));
    // The window's band is the week's: its own figures, not the day's.
    const band = visible(document.querySelector('[data-day-totals]')?.textContent);
    expect(band).toContain('62,400');
    expect(band).toContain('51,000');
    expect(band).not.toContain('14,500');
  });

  it('the arrow is the first thing on the line, before the date: it points in the reading direction when closed and down when open', async () => {
    const { user } = await open();
    const line = present(dayLine('2026-10-09'));
    const arrow = present(line.querySelector('svg'));
    // Before the date in the line, so it stands at the start edge: the right in Urdu, the left in English.
    expect(line.firstElementChild).toBe(arrow);
    expect(line.querySelector('.month-heading-title')).not.toBeNull();
    // Chevron-right, turned half a turn in Urdu so it points left; in English it points right.
    expect(arrow.classList.contains('rtl:rotate-180')).toBe(true);
    expect(arrow.classList.contains('rotate-90')).toBe(false);
    await user.click(line);
    await waitOpen('2026-10-09');
    const turned = present(present(dayLine('2026-10-09')).querySelector('svg'));
    expect(turned.classList.contains('rotate-90')).toBe(true);
    expect(turned.classList.contains('rtl:rotate-180')).toBe(false);
  });

  it('a day with no بکری is one line with no arrow, and it cannot be reached as a button', async () => {
    await open();
    for (const day of ['2026-10-08', '2026-10-06', '2026-10-04', '2026-10-03']) {
      expect(dayLine(day)).toBeNull();
      expect(visible(dayBox(day).textContent)).toContain(message(language, 'bikri-past-day-empty'));
      expect(dayBox(day).querySelector('button')).toBeNull();
    }
  });

  it('the arrow keys move between the day lines (↑ ↓), skipping a day with no arrow', async () => {
    const { user } = await open();
    present(dayLine('2026-10-09')).focus();
    await user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(dayLine('2026-10-07')).toHaveFocus();
    });
    await user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(dayLine('2026-10-05')).toHaveFocus();
    });
    await user.keyboard('{ArrowUp}');
    await waitFor(() => {
      expect(dayLine('2026-10-07')).toHaveFocus();
    });
  });

  it('a move of the window closes every day, and the next window starts closed', async () => {
    const { user } = await open();
    await user.click(present(dayLine('2026-10-09')));
    await waitOpen('2026-10-09');
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'bikri-past-back', { days: 7 })),
      }),
    );
    await waitFor(() => {
      expect(windowCalls().at(-1)).toMatchObject({ from: '26-09-2026', to: '02-10-2026' });
    });
    // Back to the first window: the day that was open is closed again.
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'bikri-past-nav-latest-button')),
      }),
    );
    await waitFor(() => {
      expect(windowCalls().at(-1)).toMatchObject({ from: '03-10-2026', to: '09-10-2026' });
    });
    await waitFor(() => {
      expect(dayLine('2026-10-09')).not.toBeNull();
    });
    expect(isOpen('2026-10-09')).toBe(false);
  });

  it('Page Down and Page Up on a day line move the window, and the days are closed after it', async () => {
    const { user } = await open();
    await user.click(present(dayLine('2026-10-07')));
    await waitOpen('2026-10-07');
    present(dayLine('2026-10-07')).focus();
    await user.keyboard('{PageUp}');
    await waitFor(() => {
      expect(windowCalls().at(-1)).toMatchObject({ from: '26-09-2026', to: '02-10-2026' });
    });
  });

  describe('go to a date', () => {
    const dateBox = () =>
      screen.findByRole('textbox', { name: named(message(language, 'bikri-past-goto')) });

    it('a day in the window on screen opens, and its line is lit', async () => {
      const { user } = await open();
      await user.type(await dateBox(), '07-10-2026');
      await user.tab();
      await waitOpen('2026-10-07');
      expect(dayLine('2026-10-07')).toHaveAttribute('data-lit');
      // The other days are as they were.
      expect(isOpen('2026-10-09')).toBe(false);
      await waitFor(() => {
        expect(rowsOf('2026-10-07')).toEqual(['71']);
      });
    });

    it('a day in another window opens that window, with that day open and lit', async () => {
      fakePast({
        window: [...WINDOW_ROWS, listRow({ id: 91, number: 7, entry_date: '2026-10-02' })],
        gross: { ...WINDOW_GROSS, '2026-10-02': 5_000 },
      });
      const { router, user } = await open();
      await user.type(await dateBox(), '02-10-2026');
      await user.tab();
      await waitFor(() => {
        expect(windowCalls().at(-1)).toMatchObject({ from: '26-09-2026', to: '02-10-2026' });
      });
      expect(router.state.location.search).toMatchObject({ end: '2026-10-02' });
      await waitFor(() => {
        expect(dayLine('2026-10-02')).toHaveAttribute('aria-expanded', 'true');
      });
      expect(dayLine('2026-10-02')).toHaveAttribute('data-lit');
    });

    it('a day with no بکری is lit but has nothing to open', async () => {
      const { user } = await open();
      await user.type(await dateBox(), '08-10-2026');
      await user.tab();
      await waitFor(() => {
        expect(dayBox('2026-10-08').className).toContain('bg-primary-soft');
      });
      expect(dayLine('2026-10-08')).toBeNull();
    });
  });

  describe('a search, a filter and "show every بکری"', () => {
    it('a search opens every day with a match, and clearing it closes them', async () => {
      const { user } = await open();
      const box = await screen.findByRole('textbox', {
        name: named(message(language, 'common-search-label')),
      });
      await user.type(box, 'LES');
      await waitOpen('2026-10-09');
      expect(isOpen('2026-10-07')).toBe(true);
      expect(isOpen('2026-10-05')).toBe(true);
      await user.clear(box);
      await waitFor(() => {
        expect(isOpen('2026-10-09')).toBe(false);
      });
    });

    it('the days that a search opens can be closed by hand', async () => {
      const { user } = await open();
      await user.type(
        await screen.findByRole('textbox', {
          name: named(message(language, 'common-search-label')),
        }),
        'LES',
      );
      await waitOpen('2026-10-09');
      await user.click(present(dayLine('2026-10-09')));
      await waitFor(() => {
        expect(isOpen('2026-10-09')).toBe(false);
      });
      expect(isOpen('2026-10-07')).toBe(true);
    });

    it('"show every بکری" opens every day, and unticking closes them again', async () => {
      const { user } = await open();
      const tick = await screen.findByRole('checkbox', {
        name: named(message(language, 'bikri-past-show-all')),
      });
      await user.click(tick);
      await waitOpen('2026-10-09');
      expect(isOpen('2026-10-07')).toBe(true);
      expect(isOpen('2026-10-05')).toBe(true);
      await user.click(tick);
      await waitFor(() => {
        expect(isOpen('2026-10-09')).toBe(false);
      });
      expect(isOpen('2026-10-07')).toBe(false);
    });
  });

  describe('the way back from an edit, and a language switch', () => {
    it('the day of the بکری the Owner came back to is open, its row has focus, and the other days stay closed', async () => {
      vi.mocked(bikriApi.bikriGet).mockResolvedValue({
        ...savedEvent1(),
        id: 61,
        entry_date: '2026-10-09',
      });
      await open('/bikri/today?end=2026-10-09&row=61');
      await waitOpen('2026-10-09');
      expect(isOpen('2026-10-07')).toBe(false);
      await waitFor(() => {
        expect(
          dayGrid('2026-10-09').querySelector('.ag-row[row-id="61"] .ag-cell-focus'),
        ).not.toBeNull();
      });
    });

    it('a language switch keeps the days that are open', async () => {
      const { user } = await open();
      await user.click(present(dayLine('2026-10-09')));
      await user.click(present(dayLine('2026-10-05')));
      await waitFor(() => {
        expect(rowsOf('2026-10-09')).toEqual(['62', '61']);
      });
      press('KeyL', { altKey: true });
      // The other language: the lines are drawn again with their words, and the same days are open.
      await waitFor(() => {
        expect(document.documentElement.lang).not.toBe(language);
      });
      await waitFor(() => {
        expect(isOpen('2026-10-09')).toBe(true);
      });
      expect(isOpen('2026-10-05')).toBe(true);
      expect(isOpen('2026-10-07')).toBe(false);
      await waitFor(() => {
        expect(rowsOf('2026-10-09')).toEqual(['62', '61']);
      });
    });
  });
});
