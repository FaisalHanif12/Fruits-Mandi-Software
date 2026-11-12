// One party's statement of 30 days (plan 74, PRD FR-12.47), in the Urdu and the English app: the window
// that ends today, the earlier and later buttons and the days between them, Page Up and Page Down and the
// arrow keys, the totals of the window from the backend, "go to a date" (an entry, no entry, after today),
// a day that opens under its own row and shuts again, the keys of the list, a range of typed dates, and
// the states. The numbers are the plan's worked example; every figure is one the double was given: the
// screen adds up nothing. Today is 10-10-2026.
//
// The backend names of "go to a date" are the plan's proposal (a double): they change here, and only here,
// when backend-lots sends the real ones.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as ledgersApi from '../../src/features/party_ledgers/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartyLedgers } from '../helpers/party-ledgers';
import {
  FOUR_DAY_STATEMENT,
  WINDOW_BEFORE_TOTALS,
  datesShown,
  dayRow,
  footerRow,
  isOpen,
  slipForDate,
  treegrid,
  windowTotals,
} from '../helpers/party-statement';
import { rangeText } from '../helpers/pager';
import { message, visible } from '../helpers/render';
import { R35_SLIP_AKRAM } from '../helpers/reports-party-real';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/party_ledgers/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
});

afterEach(() => {
  vi.unstubAllGlobals();
  leaveAppWindow();
});

/** The window as the three layout steps see it: media queries that match up to a step. */
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

describe.each(LANGUAGES)('one party’s statement of 30 days (%s)', (language: Language) => {
  const m = (id: string, args: Record<string, string | number> = {}) => message(language, id, args);
  const rtl = language === 'ur';
  const dr = rtl ? 'بنام' : 'Debit';
  /** The two ends of the window in the words of the language: "12-08-2026 to 10-09-2026". */
  const rangeWords = (from: string, to: string) => m('party-ledgers-range', { from, to });

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakePartyLedgers({
      statements: { 20: FOUR_DAY_STATEMENT },
      totalsOf: (input) => windowTotals(input),
      // Each date answers its own slip, so a day that read another day's date shows up as wrong figures.
      slipFor: (accountId, date) =>
        accountId === 20 ? slipForDate(R35_SLIP_AKRAM, date) : undefined,
    });
  });

  const named = (text: string) => (name: string) => visible(name).startsWith(text);
  const said = (text: string) => (content: string) => visible(content) === text;
  const cellsOf = (row: Element | null | undefined) =>
    Array.from(row?.querySelectorAll('[role="gridcell"]') ?? []).map((item) =>
      visible(item.textContent),
    );
  const statementCalls = () =>
    vi.mocked(ledgersApi.partyLedgersStatement).mock.calls.map(([input]) => input);
  const earlierButton = () =>
    screen.getByRole('button', { name: named(m('party-ledgers-back-days', { days: 30 })) });
  const laterButton = () =>
    screen.getByRole('button', { name: named(m('party-ledgers-forward-days', { days: 30 })) });
  const nav = () => screen.getByRole('group', { name: named(m('party-ledgers-nav-label')) });

  async function open(path = '/bikri/party-ledgers?side=buyer&party=20', firstDate = '2026-10-10') {
    const view = await renderApp(path);
    await waitFor(() => {
      expect(dayRow(firstDate)).not.toBeNull();
    }, SLOW_MS);
    return { ...view, user: userEvent.setup() };
  }

  describe('the window', () => {
    it('asks for the 30 days that end today, in one read, and shows its dates newest first', async () => {
      await open();
      expect(statementCalls().at(-1)).toEqual({
        account_id: 20,
        from: '11-09-2026',
        to: '10-10-2026',
        offset: 0,
        limit: 100,
      });
      // 08-20 is before the window: it is not drawn.
      expect(datesShown()).toEqual(['2026-10-10', '2026-10-05', '2026-09-20']);
      expect(cellsOf(dayRow('2026-10-05'))).toEqual([
        '05-10-2026',
        `30,000${dr}`,
        '21,360',
        '21,360',
        `30,000${dr}`,
      ]);
    });

    it('the statement print button of plan 74 T3 is at the top, next to the way back to the list', async () => {
      await open();
      expect(
        screen.getByRole('button', { name: named(m('statement-print-button')) }),
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: m('party-ledgers-back-list') }),
      ).toBeInTheDocument();
    });

    it('the footer is the backend’s for the window: its start, what it charged, what was paid, its end', async () => {
      await open();
      expect(cellsOf(footerRow())).toEqual([
        m('glossary-meezan'),
        '0',
        '56,360',
        '21,360',
        `35,000${dr}`,
      ]);
    });

    it('the days of the window are between the buttons, and the later button is off at today, with its reason', async () => {
      await open();
      expect(visible(nav().textContent)).toContain(rangeWords('11-09-2026', '10-10-2026'));
      expect(laterButton()).toBeDisabled();
      expect(screen.getByText(said(m('party-ledgers-nav-latest')))).toBeInTheDocument();
      // No way back to "the latest" while the window is the latest.
      expect(
        screen.queryByRole('button', { name: m('party-ledgers-nav-latest-button') }),
      ).not.toBeInTheDocument();
    });

    it('the earlier button moves 30 days back: new dates, the new footer, the days between, and the address remembers', async () => {
      const { user, router } = await open();
      await user.click(earlierButton());
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-08-20']);
      }, SLOW_MS);
      expect(statementCalls().at(-1)).toMatchObject({ from: '12-08-2026', to: '10-09-2026' });
      expect(cellsOf(footerRow())).toEqual([m('glossary-meezan'), '0', '10,000', '10,000', '0']);
      expect(WINDOW_BEFORE_TOTALS.closing_side).toBe('zero');
      expect(visible(nav().textContent)).toContain(rangeWords('12-08-2026', '10-09-2026'));
      expect(router.state.location.search).toMatchObject({ party: 20, end: '2026-09-10' });
      // The later button works now, and the way back to the latest window is offered.
      expect(laterButton()).toBeEnabled();
      expect(
        screen.getByRole('button', { name: m('party-ledgers-nav-latest-button') }),
      ).toBeInTheDocument();
    });

    it('the later button comes 30 days nearer and stops at today', async () => {
      const { user, router } = await open(
        '/bikri/party-ledgers?side=buyer&party=20&end=2026-09-10',
        '2026-08-20',
      );
      await user.click(laterButton());
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-10-10', '2026-10-05', '2026-09-20']);
      }, SLOW_MS);
      // 10-09 and 30 days is 10-10 at most: the window ends today, and the address says "today" by saying nothing.
      expect(statementCalls().at(-1)).toMatchObject({ from: '11-09-2026', to: '10-10-2026' });
      expect(router.state.location.search).not.toHaveProperty('end');
      expect(laterButton()).toBeDisabled();
    });

    it('"back to the days up to today" jumps to the latest window', async () => {
      const { user, router } = await open(
        '/bikri/party-ledgers?side=buyer&party=20&end=2026-09-10',
        '2026-08-20',
      );
      await user.click(screen.getByRole('button', { name: m('party-ledgers-nav-latest-button') }));
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-10-10', '2026-10-05', '2026-09-20']);
      }, SLOW_MS);
      expect(router.state.location.search).not.toHaveProperty('end');
    });

    it('Page Up and Page Down move the window; Page Down at today does nothing', async () => {
      await open();
      const calls = statementCalls().length;
      press('PageUp');
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-08-20']);
      }, SLOW_MS);
      press('PageDown');
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-10-10', '2026-10-05', '2026-09-20']);
      }, SLOW_MS);
      const settled = statementCalls().length;
      expect(settled).toBeGreaterThan(calls);
      press('PageDown');
      await new Promise((resolve) => setTimeout(resolve, 50));
      expect(statementCalls()).toHaveLength(settled);
      expect(datesShown()).toEqual(['2026-10-10', '2026-10-05', '2026-09-20']);
    });

    it('with focus on the buttons the arrow keys follow the reading direction', async () => {
      const { user } = await open();
      earlierButton().focus();
      // The arrow that points at the start edge goes to the earlier days: left in English, right in Urdu.
      await user.keyboard(rtl ? '{ArrowRight}' : '{ArrowLeft}');
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-08-20']);
      }, SLOW_MS);
      earlierButton().focus();
      await user.keyboard(rtl ? '{ArrowLeft}' : '{ArrowRight}');
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-10-10', '2026-10-05', '2026-09-20']);
      }, SLOW_MS);
    });

    it('a window with no entry says so in words, keeps its footer and its buttons', async () => {
      await renderApp('/bikri/party-ledgers?side=buyer&party=20&end=2026-07-01');
      expect(
        await screen.findByText(said(m('party-ledgers-window-empty')), {}, SLOW_MS),
      ).toBeInTheDocument();
      expect(cellsOf(footerRow())[0]).toBe(m('glossary-meezan'));
      expect(earlierButton()).toBeEnabled();
    });

    it('at the compact step the “previous” column waits for the normal step: four columns, never sideways', async () => {
      stubStep('compact');
      await open();
      expect(cellsOf(dayRow('2026-10-05'))).toEqual([
        '05-10-2026',
        '21,360',
        '21,360',
        `30,000${dr}`,
      ]);
      expect(cellsOf(footerRow())).toHaveLength(4);
    });
  });

  describe('go to a date', () => {
    const goBox = () => screen.getByRole('textbox', { name: named(m('party-ledgers-goto')) });

    it('a date with an entry: its window opens, its row is open and highlighted, and its day is read', async () => {
      const { user, router } = await open();
      await user.type(goBox(), '20-09-2026{Enter}');
      await waitFor(() => {
        expect(isOpen('2026-09-20')).toBe(true);
      }, SLOW_MS);
      expect(ledgersApi.partyLedgersGoToDate).toHaveBeenCalledWith({
        account_id: 20,
        date: '20-09-2026',
      });
      expect(dayRow('2026-09-20')?.getAttribute('aria-selected')).toBe('true');
      // The go-to answer carries the window's own page and totals: the window is not asked for again.
      expect(statementCalls().some((call) => call.to === '05-10-2026')).toBe(false);
      expect(datesShown()).toEqual(['2026-10-05', '2026-09-20']);
      expect(router.state.location.search).toMatchObject({ party: 20, end: '2026-10-05' });
      await waitFor(() => {
        expect(reportsApi.reportsRun).toHaveBeenCalledWith(expect.objectContaining({ id: 'R35' }));
      }, SLOW_MS);
      // No words of "no entry".
      expect(visible(document.body.textContent)).not.toContain(
        m('party-ledgers-goto-none', { date: '20-09-2026' }),
      );
    });

    it('a date with no entry says so and shows the window around it, with every day shut', async () => {
      const { user, router } = await open();
      await user.type(goBox(), '15-09-2026{Enter}');
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain(
          m('party-ledgers-goto-none', { date: '15-09-2026' }),
        );
      }, SLOW_MS);
      // The window the backend gives: 15 days after the date, so the 30 days that end on 30-09-2026.
      // The 20-09 entry is in it, shut.
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-09-20']);
      }, SLOW_MS);
      expect(isOpen('2026-09-20')).toBe(false);
      expect(router.state.location.search).toMatchObject({ party: 20, end: '2026-09-30' });
    });

    it('a date after today is refused in words and nothing is asked', async () => {
      const { user } = await open();
      await user.type(goBox(), '11-10-2026{Enter}');
      expect(
        await screen.findByText(said(m('err-date-in-future', { today: '10-10-2026' }))),
      ).toBeInTheDocument();
      expect(ledgersApi.partyLedgersGoToDate).not.toHaveBeenCalled();
    });

    it('a date that is not a day is refused in words', async () => {
      const { user } = await open();
      await user.type(goBox(), '31-02-2026{Enter}');
      expect(await screen.findByText(said(m('err-date-invalid')))).toBeInTheDocument();
      expect(ledgersApi.partyLedgersGoToDate).not.toHaveBeenCalled();
    });
  });

  describe('a day opened in place', () => {
    const slips = () => document.querySelectorAll('section dl');

    it('a click opens a day under its row, a second click shuts it, and several days can be open', async () => {
      const { user } = await open();
      await user.click(dayRow('2026-10-10') as Element);
      await waitFor(() => {
        expect(isOpen('2026-10-10')).toBe(true);
      }, SLOW_MS);
      // Right under its own row, and only that day was read.
      expect(
        dayRow('2026-10-10')?.nextElementSibling?.querySelector('[data-day-detail]'),
      ).not.toBeNull();
      expect(isOpen('2026-10-05')).toBe(false);
      await user.click(dayRow('2026-10-05') as Element);
      await waitFor(() => {
        expect(slips()).toHaveLength(2);
      }, SLOW_MS);
      expect(isOpen('2026-10-10')).toBe(true);
      expect(isOpen('2026-10-05')).toBe(true);
      await user.click(dayRow('2026-10-10') as Element);
      await waitFor(() => {
        expect(isOpen('2026-10-10')).toBe(false);
      }, SLOW_MS);
      expect(isOpen('2026-10-05')).toBe(true);
    });

    it('each opened day reads and shows its OWN date: its request, its heading and its figures, not another day’s', async () => {
      const { user } = await open();
      await user.click(dayRow('2026-10-10') as Element);
      await user.click(dayRow('2026-10-05') as Element);
      await waitFor(() => {
        expect(slips()).toHaveLength(2);
      }, SLOW_MS);
      // The day command was asked once for each date, with the party and that row's own date.
      const asked = vi
        .mocked(reportsApi.reportsRun)
        .mock.calls.map(([input]) => input)
        .filter((input) => input.id === 'R35');
      expect(asked.map((input) => input.filters.as_of).sort()).toEqual([
        '05-10-2026',
        '10-10-2026',
      ]);
      for (const input of asked) {
        expect(input.filters.party).toEqual({ account_id: 20, person_key: null });
      }
      const slipOf = (iso: string) => {
        const detail = dayRow(iso)?.nextElementSibling;
        if (detail === null || detail === undefined) throw new Error(`no day open under ${iso}`);
        return detail;
      };
      const figures = (iso: string) =>
        Object.fromEntries(
          Array.from(slipOf(iso).querySelectorAll('section dl > *')).map((line) => [
            visible(line.querySelector('dt')?.textContent),
            visible(line.querySelector('dd')?.textContent),
          ]),
        );
      // 10-10-26 charged 5,000 and paid nothing; 05-10-26 charged 21,360 and paid all of it.
      expect(figures('2026-10-10')).toMatchObject({
        [m('report-r35-today-total')]: '5,000',
        [m('report-r35-paid-today')]: '0',
        [m('report-r35-new-balance')]: `35,000${dr}`,
      });
      expect(figures('2026-10-05')).toMatchObject({
        [m('report-r35-today-total')]: '21,360',
        [m('report-r35-paid-today')]: '21,360',
        [m('report-r35-new-balance')]: `30,000${dr}`,
      });
      // Each day's heading says its own date, and no day shows the other's date.
      for (const [iso, shown, other] of [
        ['2026-10-10', '10-10-2026', '05-10-2026'],
        ['2026-10-05', '05-10-2026', '10-10-2026'],
      ] as const) {
        const detail = slipOf(iso);
        expect(
          within(detail as HTMLElement).getByRole('heading', {
            level: 3,
            name: (name) => visible(name) === m('party-ledgers-day-heading', { date: shown }),
          }),
        ).toBeInTheDocument();
        expect(visible(detail.textContent)).toContain(shown);
        expect(visible(detail.textContent)).not.toContain(other);
      }
    });

    it('a day that is one page has no page buttons inside it', async () => {
      const { user } = await open();
      await user.click(dayRow('2026-10-05') as Element);
      await waitFor(() => {
        expect(slips()).toHaveLength(1);
      }, SLOW_MS);
      expect(visible(document.body.textContent)).not.toContain(rangeText(language, 1, 1, 1));
    });

    it('each open day keeps its own print button, named with its date', async () => {
      const { user } = await open();
      await user.click(dayRow('2026-10-05') as Element);
      expect(
        await screen.findByRole(
          'button',
          { name: named(m('party-ledgers-day-print-name', { date: '05-10-2026' })) },
          SLOW_MS,
        ),
      ).toBeInTheDocument();
    });

    it('moving the window shuts every open day, also when it comes back', async () => {
      const { user } = await open();
      await user.click(dayRow('2026-10-10') as Element);
      await user.click(dayRow('2026-10-05') as Element);
      await waitFor(() => {
        expect(slips()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(earlierButton());
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-08-20']);
      }, SLOW_MS);
      expect(slips()).toHaveLength(0);
      await user.click(laterButton());
      await waitFor(() => {
        expect(datesShown()).toEqual(['2026-10-10', '2026-10-05', '2026-09-20']);
      }, SLOW_MS);
      expect(isOpen('2026-10-10')).toBe(false);
      expect(isOpen('2026-10-05')).toBe(false);
    });

    it('with the keyboard only: ↓ moves, Enter opens and shuts, the arrows open and shut in both directions', async () => {
      const { user } = await open();
      const list = treegrid();
      if (list === null) throw new Error('no list');
      list.focus();
      const active = () => list.getAttribute('aria-activedescendant');
      expect(active()).toBe(dayRow('2026-10-10')?.id);
      await user.keyboard('{ArrowDown}');
      expect(active()).toBe(dayRow('2026-10-05')?.id);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(isOpen('2026-10-05')).toBe(true);
      }, SLOW_MS);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(isOpen('2026-10-05')).toBe(false);
      }, SLOW_MS);
      // The arrow that points at the children opens, the one that points at the parent shuts: right and
      // left in English, left and right in Urdu (FR-7.13).
      await user.keyboard(rtl ? '{ArrowLeft}' : '{ArrowRight}');
      await waitFor(() => {
        expect(isOpen('2026-10-05')).toBe(true);
      }, SLOW_MS);
      await user.keyboard(rtl ? '{ArrowRight}' : '{ArrowLeft}');
      await waitFor(() => {
        expect(isOpen('2026-10-05')).toBe(false);
      }, SLOW_MS);
      await user.keyboard('{End}');
      expect(active()).toBe(dayRow('2026-09-20')?.id);
      await user.keyboard('{Home}');
      expect(active()).toBe(dayRow('2026-10-10')?.id);
    });
  });

  describe('typed dates make it a range', () => {
    it('the window buttons go and the dates are sent as typed; clearing them brings the window back', async () => {
      const { user } = await open();
      await user.type(
        screen.getByRole('textbox', { name: named(m('party-ledgers-from')) }),
        '01-09-2026',
      );
      await waitFor(() => {
        expect(statementCalls().at(-1)).toMatchObject({ from: '01-09-2026', to: null });
      }, SLOW_MS);
      await waitFor(() => {
        expect(
          screen.queryByRole('group', { name: named(m('party-ledgers-nav-label')) }),
        ).toBeNull();
      }, SLOW_MS);
      await user.clear(screen.getByRole('textbox', { name: named(m('party-ledgers-from')) }));
      await waitFor(() => {
        expect(nav()).toBeInTheDocument();
      }, SLOW_MS);
    });
  });

  describe('the states', () => {
    it('while the window is on its way the list is not drawn', async () => {
      vi.mocked(ledgersApi.partyLedgersStatement).mockImplementation(
        () => new Promise(() => undefined),
      );
      await renderApp('/bikri/party-ledgers?side=buyer&party=20');
      await screen.findByRole(
        'heading',
        { level: 1, name: named(m('party-ledgers-title')) },
        SLOW_MS,
      );
      expect(treegrid()).toBeNull();
      expect(screen.queryByRole('group', { name: named(m('party-ledgers-nav-label')) })).toBeNull();
    });

    it('a window that fails says so, with a retry that reads again', async () => {
      vi.mocked(ledgersApi.partyLedgersStatement).mockRejectedValueOnce(
        new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
      );
      const view = await renderApp('/bikri/party-ledgers?side=buyer&party=20');
      expect(
        await screen.findByText(said(m('party-ledgers-statement-failed')), {}, SLOW_MS),
      ).toBeInTheDocument();
      expect(treegrid()).toBeNull();
      await userEvent.setup().click(screen.getByRole('button', { name: m('common-retry') }));
      await waitFor(() => {
        expect(dayRow('2026-10-10')).not.toBeNull();
      }, SLOW_MS);
      expect(view.router.state.location.search).toMatchObject({ party: 20 });
    });
  });

  it('a language switch keeps the window, the open day and the place of the keys', async () => {
    const { user, router } = await open(
      '/bikri/party-ledgers?side=buyer&party=20&end=2026-09-10',
      '2026-08-20',
    );
    await user.click(dayRow('2026-08-20') as Element);
    await waitFor(() => {
      expect(isOpen('2026-08-20')).toBe(true);
    }, SLOW_MS);
    press('KeyL', { altKey: true });
    const other: Language = rtl ? 'en' : 'ur';
    await screen.findByRole(
      'heading',
      { level: 1, name: (name) => visible(name).startsWith(message(other, 'party-ledgers-title')) },
      SLOW_MS,
    );
    await waitFor(() => {
      expect(dayRow('2026-08-20')).not.toBeNull();
    }, SLOW_MS);
    expect(datesShown()).toEqual(['2026-08-20']);
    expect(isOpen('2026-08-20')).toBe(true);
    expect(router.state.location.search).toMatchObject({ party: 20, end: '2026-09-10' });
  });
});
