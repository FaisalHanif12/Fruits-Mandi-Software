// One party's statement with a row for each entry (plan 78 A7, B1 to B3, PRD FR-12.47, FR-19.16), in the Urdu and
// the English app: the 30-day window that ends today, the earlier and later buttons and the days between them,
// Page Up and Page Down and the arrow keys, the totals of the window from the backend, the balance brought
// forward, "go to a date" (an entry, no entry, after today), an entry that opens under its own row with its
// items in full, its narration and its edit, the slip of its day in place, the keys of the table, "show
// cancelled too", a range of typed dates, and the states. The numbers are the plan's worked example; every
// figure is one the double was given: the screen adds up nothing. Today is 10-10-2026.

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
  BUYER_STATEMENT,
  ENTRIES,
  REPLACED_ENTRY,
  SELLER_STATEMENT,
  entriesShown,
  entryRow,
  footerRow,
  isOpen,
  leadRow,
  treegrid,
  windowTotals,
} from '../helpers/party-statement-rows';
import { message, visible } from '../helpers/render';
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

/** The entries of the 30 days that end today, oldest first (08-20 is in the window before). */
const NOW = [107, 109, 112, 113];
const BEFORE = [101];

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

describe.each(LANGUAGES)(
  'one party’s statement, a row for each entry (%s)',
  (language: Language) => {
    const m = (id: string, args: Record<string, string | number> = {}) =>
      message(language, id, args);
    const rtl = language === 'ur';
    const dr = rtl ? 'بنام' : 'Debit';
    const cr = rtl ? 'جمع' : 'Credit';
    /** The two ends of the window in the words of the language: "12-08-2026 to 10-09-2026". */
    const rangeWords = (from: string, to: string) => m('party-ledgers-range', { from, to });
    const item = (ur: string, en: string) => (rtl ? ur : en);

    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakePartyLedgers({
        statements: { 20: BUYER_STATEMENT, 19: SELLER_STATEMENT },
        totalsOf: windowTotals,
      });
    });

    const named = (text: string) => (name: string) => visible(name).startsWith(text);
    const said = (text: string) => (content: string) => visible(content) === text;
    const cellsOf = (row: Element | null | undefined) =>
      Array.from(row?.querySelectorAll('[role="gridcell"]') ?? []).map((cell) =>
        visible(cell.textContent),
      );
    const headings = () =>
      Array.from(treegrid()?.querySelectorAll('[role="columnheader"]') ?? []).map((head) =>
        visible(head.textContent).replace(/\s+/g, ' ').trim(),
      );
    const statementCalls = () =>
      vi.mocked(ledgersApi.partyLedgersStatementRows).mock.calls.map(([input]) => input);
    const earlierButton = () =>
      screen.getByRole('button', { name: named(m('party-ledgers-back-days', { days: 30 })) });
    const laterButton = () =>
      screen.getByRole('button', { name: named(m('party-ledgers-forward-days', { days: 30 })) });
    const nav = () => screen.getByRole('group', { name: named(m('party-ledgers-nav-label')) });
    /** An entry's own words: its kind, then its number (the number sits in its own element). */
    const entryText = (type: string, number: number) => `${m(type)}${String(number)}`;

    async function open(path = '/bikri/party-ledgers?side=buyer&party=20', firstEntry = 107) {
      const view = await renderApp(path);
      await waitFor(() => {
        expect(entryRow(firstEntry)).not.toBeNull();
      }, SLOW_MS);
      return { ...view, user: userEvent.setup() };
    }

    describe('the window', () => {
      it('asks for the 30 days that end today, oldest first, in one read, and shows its entries', async () => {
        await open();
        expect(statementCalls().at(-1)).toEqual({
          account_id: 20,
          from: '11-09-2026',
          to: '10-10-2026',
          include_history: false,
          newest_first: false,
          offset: 0,
          limit: 100,
        });
        // 08-20 is before the window: it is not drawn. One date (10-10) holds two entries: two rows.
        expect(entriesShown()).toEqual(NOW);
        expect(cellsOf(entryRow(109))).toEqual([
          '05-10-2026',
          entryText('report-doc-bikri-first', 9),
          `${item('سیب', 'Apple')} 20`,
          '21,360',
          '21,360',
          '21,360',
          `30,000${dr}`,
        ]);
        // Two items of one بکری are one line: each with its quantity.
        expect(cellsOf(entryRow(112))[2]).toBe(
          `${item('مالٹا', 'Orange')} 3${m('party-ledgers-list-separator')}${item('کیلا', 'Banana')} 2`,
        );
        // A receipt has no goods and no money paid on the spot.
        expect(cellsOf(entryRow(113))).toEqual([
          '10-10-2026',
          entryText('report-doc-receipt', 3),
          '',
          '0',
          '2,000',
          '0',
          `33,000${dr}`,
        ]);
      });

      it('the headings of a buyer: date, entry, details, debit, credit, cash, balance', async () => {
        await open();
        expect(headings()).toEqual([
          m('party-ledgers-col-date'),
          m('party-ledgers-col-entry'),
          m('party-ledgers-col-detail'),
          m('party-ledgers-col-debit'),
          m('party-ledgers-col-credit'),
          m('party-ledgers-col-cash'),
          m('party-ledgers-col-running'),
        ]);
      });

      it('an edited entry carries the mark, and an entry that was not edited does not', async () => {
        await open();
        expect(entryRow(112)?.querySelector('[data-edited-mark]')).not.toBeNull();
        expect(entryRow(109)?.querySelector('[data-edited-mark]')).toBeNull();
      });

      it('the statement print button is at the top, next to the way back to the list', async () => {
        await open();
        expect(
          screen.getByRole('button', { name: named(m('statement-print-button')) }),
        ).toBeInTheDocument();
        expect(
          screen.getByRole('button', { name: m('party-ledgers-back-list') }),
        ).toBeInTheDocument();
      });

      it('the footer is the backend’s for the window: what it charged, what was paid, cash and its end', async () => {
        await open();
        expect(cellsOf(footerRow())).toEqual([
          m('glossary-meezan'),
          '',
          '',
          '56,360',
          '23,360',
          '21,360',
          `33,000${dr}`,
        ]);
      });

      it('a window that starts after a balance opens with it, on its own line before the first entry', async () => {
        await open('/bikri/party-ledgers?side=seller&party=19', 202);
        // رحیم: 15,000 was owed before the window (the advance of 01-09).
        expect(entriesShown()).toEqual([202, 203]);
        expect(cellsOf(leadRow())).toEqual([
          '',
          m('party-ledgers-brought-forward'),
          '',
          '',
          '',
          `15,000${dr}`,
        ]);
      });

      it('a window that starts from nothing has no line for the balance brought forward', async () => {
        await open();
        expect(leadRow()).toBeNull();
      });

      it('the headings of a seller have no column for cash paid on the spot', async () => {
        await open('/bikri/party-ledgers?side=seller&party=19', 202);
        expect(headings()).toEqual([
          m('party-ledgers-col-date'),
          m('party-ledgers-col-entry'),
          m('party-ledgers-col-detail'),
          m('party-ledgers-col-debit'),
          m('party-ledgers-col-credit'),
          m('party-ledgers-col-running'),
        ]);
        expect(cellsOf(entryRow(202))).toEqual([
          '10-10-2026',
          entryText('report-doc-bikri-first', 1),
          `${item('کیلا', 'Banana')} 50`,
          '5,000',
          '24,316',
          `4,316${cr}`,
        ]);
      });

      it('the days of the window are between the buttons, and the later button is off at today, with its reason', async () => {
        await open();
        expect(visible(nav().textContent)).toContain(rangeWords('11-09-2026', '10-10-2026'));
        expect(laterButton()).toBeDisabled();
        expect(screen.getByText(said(m('party-ledgers-nav-latest')))).toBeInTheDocument();
        expect(
          screen.queryByRole('button', { name: m('party-ledgers-nav-latest-button') }),
        ).not.toBeInTheDocument();
      });

      it('the earlier button moves 30 days back: new entries, the new footer, the days between, and the address remembers', async () => {
        const { user, router } = await open();
        await user.click(earlierButton());
        await waitFor(() => {
          expect(entriesShown()).toEqual(BEFORE);
        }, SLOW_MS);
        expect(statementCalls().at(-1)).toMatchObject({ from: '12-08-2026', to: '10-09-2026' });
        expect(cellsOf(footerRow())).toEqual([
          m('glossary-meezan'),
          '',
          '',
          '10,000',
          '10,000',
          '10,000',
          '0',
        ]);
        expect(visible(nav().textContent)).toContain(rangeWords('12-08-2026', '10-09-2026'));
        expect(router.state.location.search).toMatchObject({ party: 20, end: '2026-09-10' });
        expect(laterButton()).toBeEnabled();
        expect(
          screen.getByRole('button', { name: m('party-ledgers-nav-latest-button') }),
        ).toBeInTheDocument();
      });

      it('the later button comes 30 days nearer and stops at today', async () => {
        const { user, router } = await open(
          '/bikri/party-ledgers?side=buyer&party=20&end=2026-09-10',
          101,
        );
        await user.click(laterButton());
        await waitFor(() => {
          expect(entriesShown()).toEqual(NOW);
        }, SLOW_MS);
        // 10-09 and 30 days is 10-10 at most: the window ends today, and the address says "today" by saying nothing.
        expect(statementCalls().at(-1)).toMatchObject({ from: '11-09-2026', to: '10-10-2026' });
        expect(router.state.location.search).not.toHaveProperty('end');
        expect(laterButton()).toBeDisabled();
      });

      it('"back to the days up to today" jumps to the latest window', async () => {
        const { user, router } = await open(
          '/bikri/party-ledgers?side=buyer&party=20&end=2026-09-10',
          101,
        );
        await user.click(
          screen.getByRole('button', { name: m('party-ledgers-nav-latest-button') }),
        );
        await waitFor(() => {
          expect(entriesShown()).toEqual(NOW);
        }, SLOW_MS);
        expect(router.state.location.search).not.toHaveProperty('end');
      });

      it('Page Up and Page Down move the window; Page Down at today does nothing', async () => {
        await open();
        const calls = statementCalls().length;
        press('PageUp');
        await waitFor(() => {
          expect(entriesShown()).toEqual(BEFORE);
        }, SLOW_MS);
        press('PageDown');
        await waitFor(() => {
          expect(entriesShown()).toEqual(NOW);
        }, SLOW_MS);
        const settled = statementCalls().length;
        expect(settled).toBeGreaterThan(calls);
        press('PageDown');
        await new Promise((resolve) => setTimeout(resolve, 50));
        expect(statementCalls()).toHaveLength(settled);
        expect(entriesShown()).toEqual(NOW);
      });

      it('with focus on the buttons the arrow keys follow the reading direction', async () => {
        const { user } = await open();
        earlierButton().focus();
        // The arrow that points at the start edge goes to the earlier days: left in English, right in Urdu.
        await user.keyboard(rtl ? '{ArrowRight}' : '{ArrowLeft}');
        await waitFor(() => {
          expect(entriesShown()).toEqual(BEFORE);
        }, SLOW_MS);
        earlierButton().focus();
        await user.keyboard(rtl ? '{ArrowLeft}' : '{ArrowRight}');
        await waitFor(() => {
          expect(entriesShown()).toEqual(NOW);
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

      it('at the compact step the details and the cash columns wait for the normal step: five columns, never sideways', async () => {
        stubStep('compact');
        await open();
        expect(cellsOf(entryRow(109))).toEqual([
          '05-10-2026',
          entryText('report-doc-bikri-first', 9),
          '21,360',
          '21,360',
          `30,000${dr}`,
        ]);
        expect(cellsOf(footerRow())).toHaveLength(5);
      });
    });

    describe('go to a date', () => {
      const goBox = () => screen.getByRole('textbox', { name: named(m('party-ledgers-goto')) });

      it('a date with an entry: its window opens, its entry is open and highlighted', async () => {
        const { user, router } = await open();
        await user.type(goBox(), '20-09-2026{Enter}');
        await waitFor(() => {
          expect(isOpen(107)).toBe(true);
        }, SLOW_MS);
        expect(ledgersApi.partyLedgersGoToDate).toHaveBeenCalledWith({
          account_id: 20,
          date: '20-09-2026',
        });
        expect(entryRow(107)?.getAttribute('aria-selected')).toBe('true');
        // The window around it is read as any other window: the 30 days that end on 05-10-2026.
        expect(statementCalls().at(-1)).toMatchObject({ from: '06-09-2026', to: '05-10-2026' });
        expect(entriesShown()).toEqual([107, 109]);
        expect(router.state.location.search).toMatchObject({ party: 20, end: '2026-10-05' });
        expect(visible(document.body.textContent)).not.toContain(
          m('party-ledgers-goto-none', { date: '20-09-2026' }),
        );
      });

      it('the first key or click after it takes it over: the entry can be shut like any other', async () => {
        const { user } = await open();
        await user.type(goBox(), '20-09-2026{Enter}');
        await waitFor(() => {
          expect(isOpen(107)).toBe(true);
        }, SLOW_MS);
        await user.click(entryRow(107) as Element);
        await waitFor(() => {
          expect(isOpen(107)).toBe(false);
        }, SLOW_MS);
      });

      it('a date with no entry says so and shows the window around it, with every entry shut', async () => {
        const { user, router } = await open();
        await user.type(goBox(), '15-09-2026{Enter}');
        await waitFor(() => {
          expect(visible(document.body.textContent)).toContain(
            m('party-ledgers-goto-none', { date: '15-09-2026' }),
          );
        }, SLOW_MS);
        // The window the backend gives: 15 days after the date, so the 30 days that end on 30-09-2026.
        await waitFor(() => {
          expect(entriesShown()).toEqual([107]);
        }, SLOW_MS);
        expect(isOpen(107)).toBe(false);
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

    describe('an entry opened in place', () => {
      /** What an open entry shows under its row. */
      const detailOf = (id: number) => {
        const found = entryRow(id)?.nextElementSibling;
        if (found === null || found === undefined)
          throw new Error(`entry ${String(id)} is not open`);
        return found as HTMLElement;
      };

      it('a click opens an entry under its own row, a second click shuts it, and several can be open', async () => {
        const { user } = await open();
        await user.click(entryRow(112) as Element);
        await waitFor(() => {
          expect(isOpen(112)).toBe(true);
        }, SLOW_MS);
        expect(
          entryRow(112)?.nextElementSibling?.querySelector('[data-day-detail]'),
        ).not.toBeNull();
        expect(isOpen(109)).toBe(false);
        await user.click(entryRow(109) as Element);
        expect(isOpen(112)).toBe(true);
        expect(isOpen(109)).toBe(true);
        await user.click(entryRow(112) as Element);
        await waitFor(() => {
          expect(isOpen(112)).toBe(false);
        }, SLOW_MS);
        expect(isOpen(109)).toBe(true);
      });

      it('two entries of one date open on their own: opening one leaves the other shut', async () => {
        const { user } = await open();
        await user.click(entryRow(113) as Element);
        expect(isOpen(113)).toBe(true);
        expect(isOpen(112)).toBe(false);
      });

      it('shows the items in full, one line for each, the way the party ledger writes them', async () => {
        const { user } = await open();
        await user.click(entryRow(112) as Element);
        const lines = Array.from(detailOf(112).querySelectorAll('li')).map((line) =>
          visible(line.textContent),
        );
        const line = (name: [string, string], qty: string) =>
          m('report-detail-sale', {
            number: '12',
            item: item(name[0], name[1]),
            qty,
            packing: item('پیٹی', 'Box'),
            rate: '1,000',
            value: qty === '3' ? '3,000' : '2,000',
            commission: '0',
            laga: '0',
          });
        expect(lines).toEqual([line(['مالٹا', 'Orange'], '3'), line(['کیلا', 'Banana'], '2')]);
      });

      it('shows the narration of a receipt and the reason of an edit', async () => {
        const { user } = await open();
        await user.click(entryRow(113) as Element);
        expect(visible(detailOf(113).textContent)).toContain('cash at the shop');
        await user.click(entryRow(112) as Element);
        expect(visible(detailOf(112).textContent)).toContain(
          m('party-ledgers-edited-reason', { reason: 'rate was wrong' }),
        );
      });

      it('the button for the record opens it: a بکری in the sale book', async () => {
        const { user, router } = await open();
        await user.click(entryRow(112) as Element);
        await user.click(
          within(detailOf(112)).getByRole('button', { name: m('party-ledgers-open-record') }),
        );
        await waitFor(() => {
          expect(router.state.location.pathname).toBe('/bikri/book');
        }, SLOW_MS);
        expect(router.state.location.search).toMatchObject({ id: 12 });
      });

      it('the slip of the day shows in place: that day’s own date, its own request, a print preview button', async () => {
        const { user } = await open();
        await user.click(entryRow(112) as Element);
        expect(reportsApi.reportsRun).not.toHaveBeenCalled();
        await user.click(
          within(detailOf(112)).getByRole('button', { name: m('party-ledgers-day-slip-button') }),
        );
        await waitFor(() => {
          expect(reportsApi.reportsRun).toHaveBeenCalledWith(
            expect.objectContaining({ id: 'R35' }),
          );
        }, SLOW_MS);
        const asked = vi
          .mocked(reportsApi.reportsRun)
          .mock.calls.map(([input]) => input)
          .filter((input) => input.id === 'R35');
        expect(asked.map((input) => input.filters.as_of)).toEqual(['10-10-2026']);
        expect(asked[0]?.filters.party).toEqual({ account_id: 20, person_key: null });
        expect(
          within(detailOf(112)).getByRole('heading', {
            level: 3,
            name: (name) =>
              visible(name) === m('party-ledgers-day-heading', { date: '10-10-2026' }),
          }),
        ).toBeInTheDocument();
        expect(
          within(detailOf(112)).getByRole('button', {
            name: named(m('party-ledgers-day-print-name', { date: '10-10-2026' })),
          }),
        ).toBeInTheDocument();
        // Pressing the button again shuts the slip.
        await user.click(
          within(detailOf(112)).getByRole('button', { name: m('party-ledgers-day-slip-button') }),
        );
        expect(within(detailOf(112)).queryByRole('heading', { level: 3 })).toBeNull();
      });

      it('moving the window shuts every open entry, also when it comes back', async () => {
        const { user } = await open();
        await user.click(entryRow(112) as Element);
        await user.click(entryRow(109) as Element);
        await user.click(earlierButton());
        await waitFor(() => {
          expect(entriesShown()).toEqual(BEFORE);
        }, SLOW_MS);
        await user.click(laterButton());
        await waitFor(() => {
          expect(entriesShown()).toEqual(NOW);
        }, SLOW_MS);
        expect(isOpen(112)).toBe(false);
        expect(isOpen(109)).toBe(false);
      });

      it('with the keyboard only: ↓ moves, Enter opens and shuts, the arrows open and shut in both directions', async () => {
        const { user } = await open();
        const list = treegrid();
        if (list === null) throw new Error('no list');
        list.focus();
        const active = () => list.getAttribute('aria-activedescendant');
        expect(active()).toBe(entryRow(107)?.id);
        await user.keyboard('{ArrowDown}');
        expect(active()).toBe(entryRow(109)?.id);
        await user.keyboard('{Enter}');
        await waitFor(() => {
          expect(isOpen(109)).toBe(true);
        }, SLOW_MS);
        await user.keyboard('{Enter}');
        await waitFor(() => {
          expect(isOpen(109)).toBe(false);
        }, SLOW_MS);
        // The arrow that points at the children opens, the one that points at the parent shuts: right and
        // left in English, left and right in Urdu (FR-7.13).
        await user.keyboard(rtl ? '{ArrowLeft}' : '{ArrowRight}');
        await waitFor(() => {
          expect(isOpen(109)).toBe(true);
        }, SLOW_MS);
        await user.keyboard(rtl ? '{ArrowRight}' : '{ArrowLeft}');
        await waitFor(() => {
          expect(isOpen(109)).toBe(false);
        }, SLOW_MS);
        // Two entries of one date are two stops of the keys.
        await user.keyboard('{End}');
        expect(active()).toBe(entryRow(113)?.id);
        await user.keyboard('{ArrowUp}');
        expect(active()).toBe(entryRow(112)?.id);
        await user.keyboard('{Home}');
        expect(active()).toBe(entryRow(107)?.id);
      });
    });

    describe('show cancelled too', () => {
      const withReplaced = () => {
        const rows = [...ENTRIES.slice(0, 3), REPLACED_ENTRY, ...ENTRIES.slice(3)];
        fakePartyLedgers({
          statements: { 20: { ...BUYER_STATEMENT, rows, count: rows.length } },
          totalsOf: windowTotals,
        });
      };
      const switchBox = () =>
        screen.getByRole('checkbox', { name: named(m('party-ledgers-show-history')) });

      it('is off: an entry that an edit replaced is not on the list, and the backend is told so', async () => {
        withReplaced();
        await open();
        expect(entriesShown()).toEqual(NOW);
        expect(switchBox()).not.toBeChecked();
        expect(statementCalls().every((call) => !call.include_history)).toBe(true);
      });

      it('is on: the replaced entry shows, marked, and the backend is asked for it', async () => {
        withReplaced();
        const { user } = await open();
        await user.click(switchBox());
        await waitFor(() => {
          expect(entriesShown()).toEqual([107, 109, 111, 112, 113]);
        }, SLOW_MS);
        expect(statementCalls().at(-1)).toMatchObject({ include_history: true });
        expect(visible(entryRow(111)?.textContent)).toContain(m('party-ledgers-replaced'));
        // Off again, and it goes.
        await user.click(switchBox());
        await waitFor(() => {
          expect(entriesShown()).toEqual(NOW);
        }, SLOW_MS);
      });
    });

    describe('typed dates make it a range', () => {
      it('the window buttons go, the dates are sent as typed, the balance before them opens the table; clearing them brings the window back', async () => {
        const { user } = await open();
        await user.type(
          screen.getByRole('textbox', { name: named(m('party-ledgers-from')) }),
          '01-10-2026',
        );
        await waitFor(() => {
          expect(statementCalls().at(-1)).toMatchObject({ from: '01-10-2026' });
        }, SLOW_MS);
        await waitFor(() => {
          expect(
            screen.queryByRole('group', { name: named(m('party-ledgers-nav-label')) }),
          ).toBeNull();
        }, SLOW_MS);
        // 30,000 was owed before 01-10: the range opens with it.
        await waitFor(() => {
          expect(entriesShown()).toEqual([109, 112, 113]);
        }, SLOW_MS);
        expect(cellsOf(leadRow())[6]).toBe(`30,000${dr}`);
        await user.clear(screen.getByRole('textbox', { name: named(m('party-ledgers-from')) }));
        await waitFor(() => {
          expect(nav()).toBeInTheDocument();
        }, SLOW_MS);
      });
    });

    describe('the states', () => {
      it('while the window is on its way the list is not drawn', async () => {
        vi.mocked(ledgersApi.partyLedgersStatementRows).mockImplementation(
          () => new Promise(() => undefined),
        );
        await renderApp('/bikri/party-ledgers?side=buyer&party=20');
        await screen.findByRole(
          'heading',
          { level: 1, name: named(m('party-ledgers-title')) },
          SLOW_MS,
        );
        expect(treegrid()).toBeNull();
        expect(
          screen.queryByRole('group', { name: named(m('party-ledgers-nav-label')) }),
        ).toBeNull();
      });

      it('a window that fails says so, with a retry that reads again', async () => {
        vi.mocked(ledgersApi.partyLedgersStatementRows).mockRejectedValueOnce(
          new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
        );
        const view = await renderApp('/bikri/party-ledgers?side=buyer&party=20');
        expect(
          await screen.findByText(said(m('party-ledgers-statement-failed')), {}, SLOW_MS),
        ).toBeInTheDocument();
        expect(treegrid()).toBeNull();
        await userEvent.setup().click(screen.getByRole('button', { name: m('common-retry') }));
        await waitFor(() => {
          expect(entryRow(107)).not.toBeNull();
        }, SLOW_MS);
        expect(view.router.state.location.search).toMatchObject({ party: 20 });
      });
    });

    it('a language switch keeps the window, the open entry and the place of the keys', async () => {
      const { user, router } = await open(
        '/bikri/party-ledgers?side=buyer&party=20&end=2026-09-10',
        101,
      );
      await user.click(entryRow(101) as Element);
      await waitFor(() => {
        expect(isOpen(101)).toBe(true);
      }, SLOW_MS);
      press('KeyL', { altKey: true });
      const other: Language = rtl ? 'en' : 'ur';
      await screen.findByRole(
        'heading',
        {
          level: 1,
          name: (name) => visible(name).startsWith(message(other, 'party-ledgers-title')),
        },
        SLOW_MS,
      );
      await waitFor(() => {
        expect(entryRow(101)).not.toBeNull();
      }, SLOW_MS);
      expect(entriesShown()).toEqual(BEFORE);
      expect(isOpen(101)).toBe(true);
      expect(router.state.location.search).toMatchObject({ party: 20, end: '2026-09-10' });
      // The row says its kind in the other language now.
      expect(visible(entryRow(101)?.textContent)).toContain(
        message(other, 'report-doc-bikri-first'),
      );
    });
  },
);
