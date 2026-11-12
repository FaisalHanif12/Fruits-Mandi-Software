// پچھلی بکری / Past Sales (plan 72, PRD 12.7, FR-12.46), in the Urdu and the English app, against a fake
// backend: today's section and the window of 7 earlier days with a heading for each day (the date, the
// count and the خام بکری, all the backend's), empty days, the divider, the buttons, Page Up and Page
// Down and the arrow keys that move the window, "تاریخ پر جائیں", the tick "show every بکری", opening
// a row, the way back from an edit, and the four states. Today in these tests is 10-10-2026, so the
// first window is 03-10-2026 to 09-10-2026. The counts and totals are what the double is given; the
// screen adds up nothing.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { listRow } from '../helpers/bikri-saved';
import { cellOf, gridOf, named, salesGrid } from '../helpers/bikri-screen';
import {
  DAY,
  fail,
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
  vi.unstubAllGlobals();
  leaveAppWindow();
});

/** Two بکری on 09-10-2026 (one old with a truck, one new without), and one on 07-10-2026. */
const WINDOW_ROWS = [
  listRow({ id: 62, number: 3, entry_date: '2026-10-09', truck_no: null }),
  listRow({ id: 61, number: 2, entry_date: '2026-10-09', truck_no: 'LES-1234' }),
  listRow({ id: 71, number: 1, entry_date: '2026-10-07', truck_no: null }),
];
/** The خام بکری of those days in whole rupees, as the backend would say them. */
const WINDOW_GROSS = { '2026-10-09': 41_400, '2026-10-07': 12_000 };

/** The width of the window as the three steps see it: media queries that match up to a step. */
function stubStep(step: 'compact' | 'normal' | 'wide') {
  const matchesFor = (query: string) => {
    if (query.includes('1920')) return step === 'wide';
    if (query.includes('1366')) return step !== 'compact';
    return false;
  };
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: matchesFor(query),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  }));
  document.documentElement.style.setProperty('--breakpoint-normal', '1366px');
  document.documentElement.style.setProperty('--breakpoint-wide', '1920px');
}

/** The double answers both sections from `fake`. */
function fakePast(fake: PastFake = {}) {
  vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
    Promise.resolve(pastAnswer(input, fake)),
  );
}

/** The calls of the window (they have no `date`), oldest first. */
const windowCalls = () =>
  vi
    .mocked(bikriApi.bikriList)
    .mock.calls.map(([input]) => input)
    .filter((input) => input.date === null);
const lastWindow = () => windowCalls().at(-1);
const todayCalls = () =>
  vi
    .mocked(bikriApi.bikriList)
    .mock.calls.map(([input]) => input)
    .filter((input) => input.date !== null);

/** The element or value, or a clear failure when it is not on screen. */
function present<T>(value: T | null | undefined): T {
  if (value === null || value === undefined) throw new Error('not on screen');
  return value;
}

/** The ids of the rows and headings of a grid, in the order they stand. */
function rowIds(grid: HTMLElement): (string | null)[] {
  return Array.from(grid.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)'))
    .sort((a, b) => Number(a.getAttribute('row-index')) - Number(b.getAttribute('row-index')))
    .map((row) => row.getAttribute('row-id'));
}

describe.each(LANGUAGES)('پچھلی بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const pageText = () => visible(document.body.textContent);
  const windowGrid = () => gridOf(language, 'bikri-past-label');
  const heading = (day: string) =>
    visible(document.querySelector(`.ag-row[row-id="group:${day}"]`)?.textContent);
  /** The text of the heading of the window: its title and its chips. */
  const windowHeader = () =>
    visible(
      screen.getByRole('heading', {
        level: 2,
        name: named(message(language, 'bikri-past-window-title', { days: 7 })),
      }).parentElement?.textContent,
    );
  const button = (id: string) =>
    screen.getByRole('button', {
      name: (name) => visible(name).startsWith(message(language, id, { days: 7 })),
    });
  const open = async (path = '/bikri/today') => {
    const view = await renderApp(path);
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'layout-module-today')),
    });
    // The day is known a moment after the screen opens; the buttons of the window are drawn then.
    await screen.findByRole('group', { name: named(message(language, 'bikri-past-nav-label')) });
    return { ...view, user: userEvent.setup() };
  };
  /** A screen of the sale book, opened from the list (it has no list title). */
  const openBook = async (path: string) => {
    const view = await renderApp(path);
    return { ...view, user: userEvent.setup() };
  };
  /** The window is on screen with its day headings. */
  const windowShown = async () => {
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id^="group:"]')).not.toBeNull();
    });
  };

  describe('the two sections', () => {
    it('is named پچھلی بکری / Past Sales, and today’s بکری come first', async () => {
      await open();
      expect(
        await screen.findByRole('heading', {
          level: 2,
          name: named(message(language, 'bikri-past-today-title')),
        }),
      ).toBeInTheDocument();
      await waitFor(() => {
        expect(document.querySelector('.ag-row[row-id="41"]')).not.toBeNull();
      });
      expect(message(language, 'layout-module-today')).toBe(
        language === 'ur' ? 'پچھلی بکری' : 'Past Sales',
      );
    });

    it('asks for today in one call and for the 7 days that end yesterday in another', async () => {
      await open();
      await waitFor(() => {
        expect(lastWindow()).toBeDefined();
      });
      expect(todayCalls()[0]).toMatchObject({
        date: '10-10-2026',
        from: null,
        to: null,
        all: false,
        group_by_day: false,
      });
      expect(lastWindow()).toMatchObject({
        date: null,
        from: '03-10-2026',
        to: '09-10-2026',
        offset: 0,
        all: false,
        // Only the window asks for the days: they are its day headings.
        group_by_day: true,
      });
    });

    it('asks for no list until the day is known, however late the day status comes: every call has its days', async () => {
      // The day status is slower than the pause of the filters (150 ms): a list asked for with the days
      // it did not have yet would be a call with no day at all (every بکری of every day), and a window
      // without days is refused (plan 72, found on the development page).
      vi.mocked(appApi.dayStatus).mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => {
              resolve(DAY);
            }, 400);
          }),
      );
      await open();
      await waitFor(() => {
        expect(lastWindow()).toBeDefined();
      });
      const calls = vi.mocked(bikriApi.bikriList).mock.calls.map(([input]) => input);
      expect(calls.length).toBeGreaterThanOrEqual(2);
      for (const input of calls) {
        expect(input.date !== null || (input.from !== null && input.to !== null)).toBe(true);
      }
    });

    it('a rule with a pill between the sections says the earlier 7 days and the days they cover; the two buttons are at the end, with their keys', async () => {
      await open();
      const divider = await screen.findByRole('separator', {
        name: named(
          message(language, 'bikri-past-divider-label', { from: '03-10-2026', to: '09-10-2026' }),
        ),
      });
      expect(visible(divider.textContent)).toContain('03-10-2026');
      expect(visible(divider.textContent)).toContain('09-10-2026');
      // The buttons are in the group at the end of the page, after both lists.
      const nav = screen.getByRole('group', {
        name: named(message(language, 'bikri-past-nav-label')),
      });
      expect(nav.compareDocumentPosition(divider) & Node.DOCUMENT_POSITION_PRECEDING).toBeTruthy();
      expect(visible(button('bikri-past-back').textContent)).toContain('Page Up');
      expect(visible(button('bikri-past-forward').textContent)).toContain('Page Down');
    });
  });

  describe('the days of the window', () => {
    beforeEach(() => {
      fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
    });

    it('every day has a heading, newest first, with its rows under it; empty days say so', async () => {
      await open();
      await windowShown();
      expect(rowIds(windowGrid())).toEqual([
        'group:2026-10-09',
        '62',
        '61',
        'group:2026-10-08',
        'group:2026-10-07',
        '71',
        'group:2026-10-06',
        'group:2026-10-05',
        'group:2026-10-04',
        'group:2026-10-03',
      ]);
      for (const day of ['2026-10-08', '2026-10-06', '2026-10-05', '2026-10-04', '2026-10-03']) {
        expect(heading(day)).toContain(message(language, 'bikri-past-day-empty'));
      }
    });

    it('a day with sales is a band with its date in bold and two chips; an empty day is one quiet line', async () => {
      await open();
      await windowShown();
      const band = (day: string) =>
        document.querySelector<HTMLElement>(`.ag-row[row-id="group:${day}"] .month-heading`);
      expect(band('2026-10-09')?.classList.contains('month-heading-quiet')).toBe(false);
      expect(band('2026-10-09')?.querySelectorAll('[data-heading-chip]')).toHaveLength(2);
      expect(band('2026-10-09')?.querySelector('.month-heading-title')?.textContent).toBe(
        '09-10-2026',
      );
      expect(band('2026-10-08')?.classList.contains('month-heading-quiet')).toBe(true);
      expect(band('2026-10-08')?.querySelectorAll('[data-heading-chip]')).toHaveLength(0);
      // The figures of a chip are written left to right, as every number (F25).
      expect(
        band('2026-10-09')?.querySelector('[data-heading-chip] bdi[dir="ltr"]'),
      ).not.toBeNull();
    });

    it('a day heading has its date, its count and its خام بکری, as the backend said them', async () => {
      await open();
      await windowShown();
      const figures = (day: string) =>
        Array.from(
          document.querySelectorAll(`.ag-row[row-id="group:${day}"] [data-heading-chip]`),
        ).map((figure) => visible(figure.textContent));
      expect(heading('2026-10-09')).toContain('09-10-2026');
      // The count and the خام بکری are the backend's: two بکری on that day, 41,400 together.
      expect(figures('2026-10-09')).toEqual([
        `${message(language, 'bikri-past-day-sales')}2`,
        `${message(language, 'glossary-kham-bikri')}41,400`,
      ]);
      expect(figures('2026-10-07')).toEqual([
        `${message(language, 'bikri-past-day-sales')}1`,
        `${message(language, 'glossary-kham-bikri')}12,000`,
      ]);
      // A day with no بکری shows no figure, only its words.
      expect(figures('2026-10-08')).toEqual([]);
    });

    it('the rows of the window have the columns of the list, and the totals row is the backend’s', async () => {
      await open();
      await windowShown();
      const row = windowGrid().querySelector('.ag-row[row-id="61"]');
      expect(visible(row?.querySelector('[col-id="number"]')?.textContent)).toBe('2');
      expect(visible(row?.querySelector('[col-id="gross"]')?.textContent)).toBe('29,400');
      expect(windowGrid().querySelector('.ag-row-pinned [col-id="sellers"]')).not.toBeNull();
    });

    it('the truck of an old بکری is in its row at the wide step, and a new one has none', async () => {
      stubStep('wide');
      await open();
      await windowShown();
      const truck = (id: string) =>
        visible(
          windowGrid().querySelector(`.ag-row[row-id="${id}"] [col-id="truck"]`)?.textContent,
        );
      expect(truck('61')).toBe('LES-1234');
      expect(truck('62')).toBe('');
    });

    it('the search box sends what is typed (a number or a truck) to both lists', async () => {
      await open();
      await windowShown();
      const user = userEvent.setup();
      await user.type(
        await screen.findByRole('textbox', {
          name: named(message(language, 'common-search-label')),
        }),
        'LES',
      );
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ search: 'LES' });
      });
      expect(todayCalls().at(-1)).toMatchObject({ search: 'LES' });
    });
  });

  describe('moving the window', () => {
    beforeEach(() => {
      fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
    });

    it('the earlier button goes back 7 days, the later one comes back, and it never passes yesterday', async () => {
      const { router, user } = await open();
      await windowShown();
      // On yesterday already: the later button is off.
      expect(button('bikri-past-forward')).toBeDisabled();
      await user.click(button('bikri-past-back'));
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '26-09-2026', to: '02-10-2026' });
      });
      expect(router.state.location.search).toMatchObject({ end: '2026-10-02' });
      expect(pageText()).toContain('26-09-2026');
      await waitFor(() => {
        expect(button('bikri-past-forward')).toBeEnabled();
      });
      await user.click(button('bikri-past-forward'));
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '03-10-2026', to: '09-10-2026' });
      });
      // The window that ends yesterday is not written in the address.
      expect(router.state.location.search).not.toHaveProperty('end');
      await waitFor(() => {
        expect(button('bikri-past-forward')).toBeDisabled();
      });
    });

    it('Page Up in the window goes back 7 days and Page Down comes 7 days nearer', async () => {
      const { user } = await open();
      await windowShown();
      await user.click(cellOf(windowGrid(), 1, 'number'));
      await user.keyboard('{PageUp}');
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '26-09-2026', to: '02-10-2026' });
      });
      await waitFor(() => {
        expect(button('bikri-past-forward')).toBeEnabled();
      });
      // The new window has no بکری, so the grid is not there: the keys of the divider are tried too.
      const calls = windowCalls().length;
      await user.click(button('bikri-past-back'));
      await waitFor(() => {
        expect(windowCalls().length).toBeGreaterThan(calls);
      });
      expect(lastWindow()).toMatchObject({ from: '19-09-2026', to: '25-09-2026' });
    });

    it('with focus on the bar, the arrow that points back in reading goes back, the other comes nearer', async () => {
      const { user } = await open();
      await windowShown();
      // Urdu reads to the left, so the right arrow goes back; English reads to the right.
      const back = language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}';
      const nearer = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
      button('bikri-past-back').focus();
      await user.keyboard(back);
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '26-09-2026', to: '02-10-2026' });
      });
      await user.keyboard(nearer);
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '03-10-2026', to: '09-10-2026' });
      });
      // Already on yesterday: the same key does nothing.
      const calls = windowCalls().length;
      await user.keyboard(nearer);
      expect(windowCalls().length).toBe(calls);
    });
  });

  describe('the heading of each part, and the way back to the latest week', () => {
    it('the heading of the window has the days, the count and the خام بکری of the whole filter, as the backend sent them', async () => {
      fakePast({
        window: WINDOW_ROWS,
        gross: WINDOW_GROSS,
        // Whole rupees 53,300, as the backend's totals over the filter would say them.
        windowTotals: { gross: paisaForTest(5_330_000) },
      });
      await open();
      await windowShown();
      const header = windowHeader();
      expect(header).toContain(message(language, 'bikri-past-day-sales') + '3');
      expect(header).toContain(message(language, 'glossary-kham-bikri') + '53,300');
      expect(header).toContain('03-10-2026');
      expect(header).toContain('09-10-2026');
    });

    it('the button back to the latest week is there only when the window is an earlier one, and goes to yesterday’s window', async () => {
      fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
      const { router, user } = await open();
      await windowShown();
      const latest = () =>
        screen.queryByRole('button', {
          name: named(message(language, 'bikri-past-nav-latest-button')),
        });
      expect(latest()).toBeNull();
      await user.click(button('bikri-past-back'));
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ to: '02-10-2026' });
      });
      await user.click(
        await screen.findByRole('button', {
          name: named(message(language, 'bikri-past-nav-latest-button')),
        }),
      );
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '03-10-2026', to: '09-10-2026' });
      });
      expect(router.state.location.search).not.toHaveProperty('end');
      await waitFor(() => {
        expect(latest()).toBeNull();
      });
    });
  });

  describe('go to a date', () => {
    beforeEach(() => {
      fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
    });
    const dateBox = () =>
      screen.findByRole('textbox', { name: named(message(language, 'bikri-past-goto')) });

    it('a day typed opens the 7 days that end on it, and the box is empty again', async () => {
      const { router, user } = await open();
      await windowShown();
      const box = await dateBox();
      await user.type(box, '05-10-2026');
      await user.tab();
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '29-09-2026', to: '05-10-2026' });
      });
      expect(router.state.location.search).toMatchObject({ end: '2026-10-05' });
      expect(box).toHaveValue('');
    });

    it('keyboard only: the search box has focus on opening, Shift+Tab reaches the date box, a day and Enter go to it', async () => {
      await open();
      await windowShown();
      const search = await screen.findByRole('textbox', {
        name: named(message(language, 'common-search-label')),
      });
      // The date box is above the filters but is not the one in focus when the screen opens.
      await waitFor(() => {
        expect(search).toHaveFocus();
      });
      const box = await dateBox();
      expect(box).not.toHaveFocus();
      expect(box).toHaveAttribute('placeholder', message(language, 'bikri-past-goto-placeholder'));
      const user = userEvent.setup();
      await user.tab({ shift: true });
      await waitFor(() => {
        expect(box).toHaveFocus();
      });
      // A short form (FR-7.19): "5" is the 5th of this month.
      await user.keyboard('5{Enter}');
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '29-09-2026', to: '05-10-2026' });
      });
    });

    it('a day after today is refused in words, and the window stays', async () => {
      const { user } = await open();
      await windowShown();
      const calls = windowCalls().length;
      await user.type(await dateBox(), '11-10-2026');
      await user.tab();
      await waitFor(() => {
        expect(pageText()).toContain(
          message(language, 'err-date-in-future', { today: '10-10-2026' }),
        );
      });
      expect(windowCalls().length).toBe(calls);
    });

    it('a day that is not real is refused in words', async () => {
      const { user } = await open();
      await windowShown();
      await user.type(await dateBox(), '31-02-2026');
      await user.tab();
      await waitFor(() => {
        expect(pageText()).toContain(message(language, 'err-date-invalid'));
      });
    });

    it('today’s date shows the window that ends yesterday', async () => {
      const { router, user } = await open('/bikri/today?end=2026-10-02');
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ to: '02-10-2026' });
      });
      await user.type(await dateBox(), '10-10-2026');
      await user.tab();
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '03-10-2026', to: '09-10-2026' });
      });
      expect(router.state.location.search).not.toHaveProperty('end');
    });

    it('the + key moves the day of the box by one, as in every date field (FR-7.19)', async () => {
      const { user } = await open();
      const box = await dateBox();
      await user.click(box);
      await user.keyboard('05-10-2026');
      // The plus key moves the day while all the text is selected, as it is on arrival in the field; after
      // typing it would be a character of the date (FR-7.19, FR-7.15). It needs Shift on most keyboards.
      if (!(box instanceof HTMLInputElement)) throw new Error('the date box is not an input');
      box.select();
      press('Equal', { shiftKey: true });
      expect(box).toHaveValue('06-10-2026');
    });
  });

  describe('پوری بکری دکھائیں', () => {
    const tick = () =>
      screen.findByRole('checkbox', { name: named(message(language, 'bikri-past-show-all')) });

    it('is off when the screen opens, and sets the filters aside when it is on', async () => {
      fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
      const { router, user } = await open();
      await windowShown();
      expect(await tick()).not.toBeChecked();
      await user.type(
        await screen.findByRole('textbox', {
          name: named(message(language, 'common-search-label')),
        }),
        'LES',
      );
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ search: 'LES', all: false });
      });
      await user.click(await tick());
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ all: true, search: null, status: null });
      });
      expect(todayCalls().at(-1)).toMatchObject({ all: true, search: null });
      expect(router.state.location.search).toMatchObject({ all: true });
      expect(pageText()).toContain(message(language, 'bikri-past-show-all-note'));
      // The filters keep their values: the box still says LES, and unticking sends it again.
      expect(
        await screen.findByRole('textbox', {
          name: named(message(language, 'common-search-label')),
        }),
      ).toHaveValue('LES');
      await user.click(await tick());
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ all: false, search: 'LES' });
      });
      expect(pageText()).not.toContain(message(language, 'bikri-past-show-all-note'));
    });

    it('a cancelled بکری of the window shows with the tick, in words', async () => {
      const cancelled = listRow({
        id: 63,
        number: 4,
        entry_date: '2026-10-09',
        status: 'cancelled',
      });
      fakePast({ window: [cancelled, ...WINDOW_ROWS], gross: WINDOW_GROSS });
      const { user } = await open();
      await windowShown();
      await user.click(await tick());
      await waitFor(() => {
        expect(
          visible(
            windowGrid().querySelector('.ag-row[row-id="63"] [col-id="status"]')?.textContent,
          ),
        ).toBe(message(language, 'bikri-status-cancelled'));
      });
    });
  });

  describe('opening a row and coming back', () => {
    beforeEach(() => {
      fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
    });

    it('Enter on a row of today opens the whole بکری, and the way back is in the address', async () => {
      const { router, user } = await open();
      await waitFor(() => {
        expect(document.querySelector('.ag-row[row-id="41"]')).not.toBeNull();
      });
      await user.click(
        present(document.querySelector<HTMLElement>('.ag-row[row-id="41"] [col-id="number"]')),
      );
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      });
      expect(router.state.location.search).toMatchObject({ id: 41, past: '2026-10-09' });
      expect(router.state.location.search).not.toHaveProperty('pastAll');
    });

    it('Enter on a row of the window opens it with its own window, and the tick comes along', async () => {
      fakePast({ window: [listRow({ id: 81, number: 9, entry_date: '2026-10-04' })] });
      const { router, user } = await open('/bikri/today?end=2026-10-05&all=true');
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ to: '05-10-2026', all: true });
      });
      await waitFor(() => {
        expect(windowGrid().querySelector('.ag-row[row-id="81"]')).not.toBeNull();
      });
      await user.click(
        present(windowGrid().querySelector<HTMLElement>('.ag-row[row-id="81"] [col-id="number"]')),
      );
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      });
      expect(router.state.location.search).toMatchObject({
        id: 81,
        past: '2026-10-05',
        pastAll: true,
      });
    });

    it('a بکری opened from the list has a button back to it, and Edit keeps the way back', async () => {
      fakePlan16();
      const { router, user } = await openBook('/bikri/book?id=41&past=2026-10-02');
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'bikri-saved-title', { number: 1 })),
      });
      await user.click(
        screen.getByRole('button', { name: named(message(language, 'edit-action-edit')) }),
      );
      await waitFor(() => {
        expect(router.state.location.search).toMatchObject({ id: 41, edit: 7, past: '2026-10-02' });
      });
      await router.navigate({ to: '/bikri/book', search: { id: 41, past: '2026-10-02' } });
      // "Back to Past Sales" is on the toolbar of the saved بکری (plan 72).
      await user.click(
        await screen.findByRole('button', {
          name: named(message(language, 'bikri-past-back-to-list')),
        }),
      );
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/today');
      });
      expect(router.state.location.search).toMatchObject({ end: '2026-10-02', row: 41 });
    });

    it('Esc on a بکری opened from the list goes back to the list at the same window and row (plan 72)', async () => {
      fakePlan16();
      // As the Owner does it: from the list to the بکری (a screen opened later than the shell's own Esc).
      const { router } = await open();
      await router.navigate({ to: '/bikri/book', search: { id: 41, past: '2026-10-02' } });
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'bikri-saved-title', { number: 1 })),
      });
      press('Escape', { key: 'Escape' });
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/today');
      });
      expect(router.state.location.search).toMatchObject({ end: '2026-10-02', row: 41 });
    });

    it('after an edit is saved the Owner is back on the same window and row, and the list asks again', async () => {
      const plan = fakePlan16();
      const { router, user } = await openBook('/bikri/book?id=41&edit=7&past=2026-10-02');
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'bikri-edit-title', { number: 1 })),
      });
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).toBeInTheDocument();
      });
      await user.click(cellOf(salesGrid(language), 0, 'rate'));
      await user.keyboard('610{Enter}');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
      });
      press('F10');
      await waitFor(() => {
        expect(plan.editSaves).toHaveLength(1);
      });
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/today');
      });
      expect(router.state.location.search).toMatchObject({ end: '2026-10-02', row: 41 });
      // The list reads the backend again for that window (nothing is kept on the screen).
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '26-09-2026', to: '02-10-2026' });
      });
    });

    it('the row and the day heading show the new values when the list is opened again', async () => {
      const { router } = await open('/bikri/today?end=2026-10-09&row=61');
      await windowShown();
      expect(heading('2026-10-09')).toContain('41,400');
      await router.navigate({ to: '/bikri/book', search: { id: 61, past: '2026-10-09' } });
      // The edit changed the day's total and the row; the way back asks the backend again.
      fakePast({
        window: [
          listRow({ id: 62, number: 3, entry_date: '2026-10-09' }),
          listRow({ id: 61, number: 2, entry_date: '2026-10-09', gross: listRow().net }),
        ],
        gross: { '2026-10-09': 38_316 },
      });
      await router.navigate({ to: '/bikri/today', search: { end: '2026-10-09', row: 61 } });
      await waitFor(() => {
        expect(heading('2026-10-09')).toContain('38,316');
      });
      await waitFor(() => {
        expect(
          visible(windowGrid().querySelector('.ag-row[row-id="61"] [col-id="gross"]')?.textContent),
        ).toBe('19,316');
      });
    });

    it('the row the Owner came back to has focus', async () => {
      await open('/bikri/today?end=2026-10-09&row=61');
      await windowShown();
      await waitFor(() => {
        expect(windowGrid().querySelector('.ag-row[row-id="61"] .ag-cell-focus')).not.toBeNull();
      });
    });
  });

  describe('the four states', () => {
    it('a window with no بکری says so in words, and today’s empty section says it in one line', async () => {
      fakePast({ today: [], window: [] });
      const { router, user } = await open();
      await waitFor(() => {
        expect(pageText()).toContain(message(language, 'bikri-past-window-empty'));
      });
      expect(pageText()).toContain(message(language, 'bikri-past-window-empty-help'));
      expect(pageText()).toContain(message(language, 'bikri-past-today-empty', { key: 'Alt+B' }));
      // The divider is still there, so the Owner can look at other days.
      await user.click(button('bikri-past-back'));
      await waitFor(() => {
        expect(lastWindow()).toMatchObject({ from: '26-09-2026' });
      });
      expect(router.state.location.search).toMatchObject({ end: '2026-10-02' });
    });

    it('a window that cannot be read says so, with a retry that asks again', async () => {
      fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
      vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
        input.date === null
          ? Promise.reject(fail({ code: 'ERR_DB_BUSY', field: null, values: {} }))
          : Promise.resolve(pastAnswer(input)),
      );
      const { user } = await open();
      await waitFor(() => {
        expect(pageText()).toContain(message(language, 'bikri-past-failed'));
      });
      // Today's section is on screen beside it.
      await waitFor(() => {
        expect(document.querySelector('.ag-row[row-id="41"]')).not.toBeNull();
      });
      fakePast({ window: WINDOW_ROWS, gross: WINDOW_GROSS });
      await user.click(
        within(
          screen.getByText(message(language, 'bikri-past-failed')).closest('div') as HTMLElement,
        ).getByRole('button', { name: named(message(language, 'common-retry')) }),
      );
      await windowShown();
    });
  });
});
