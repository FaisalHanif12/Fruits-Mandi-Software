// پارٹی کھاتے, the party ledgers screen (plan 66, PRD FR-12.43, D8 Feature 6, E2E-67), in the Urdu and the
// English app, against a fake backend that answers with the backend's own fixtures of the worked day.
// Level 1 lists the buyers or the sellers with their balance under the side it is on and the plain
// total of the debit and the credit; level 2 is one party's statement, a row for each date; level 3
// is the R35 slip of one party and one date, which prints. Every figure is one the backend sent: the
// screen adds up nothing. The numbers of plan 66 Section 2 are checked here from the real fixtures.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { PartyLedgerRow } from '../../src/bindings/PartyLedgerRow';
import * as ledgersApi from '../../src/features/party_ledgers/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { LEDGER_BUYERS, STATEMENT_AKRAM, fakePartyLedgers } from '../helpers/party-ledgers';
import { PAGE_ROWS } from '../helpers/pager';
import { R35_SLIP_AKRAM_REQUEST, R35_SLIP_RAHIM_REQUEST } from '../helpers/reports-party-real';
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

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

/** The words of the side of a balance, as the screen shows them. */
const SIDE = {
  en: { dr: 'Debit', cr: 'Credit' },
  ur: { dr: 'بنام', cr: 'جمع' },
} as const;

describe.each(LANGUAGES)('party ledgers (%s)', (language: Language) => {
  const m = (id: string, args: Record<string, string | number> = {}) => message(language, id, args);
  const side = SIDE[language];
  const names = {
    akram: language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    arshad: language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
    rahim: language === 'ur' ? 'رحیم' : 'Rahim',
    karim: language === 'ur' ? 'کریم' : 'Karim',
  };

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** A name that is exactly `text`, or `text` followed by the key chip of its button. */
  const named = (text: string) => (name: string) => {
    const seen = visible(name);
    return (
      seen === text ||
      (seen.startsWith(text) && ['F9', 'Esc'].includes(seen.slice(text.length).trim()))
    );
  };

  async function open(path = '/bikri/party-ledgers') {
    const view = await renderApp(path);
    await screen.findByRole(
      'heading',
      { level: 1, name: named(m('party-ledgers-title')) },
      SLOW_MS,
    );
    return { ...view, user: userEvent.setup() };
  }

  const rowOf = (id: string | number) => document.querySelector(`.ag-row[row-id="${String(id)}"]`);
  const cell = (id: string | number, col: string) =>
    visible(rowOf(id)?.querySelector(`[col-id="${col}"]`)?.textContent);
  /** The figure of a balance cell and, apart from it, the side word set after it. */
  const figure = (id: string | number, col: string) =>
    visible(
      rowOf(id)?.querySelector(`[col-id="${col}"] .cell-with-unit > span:first-child`)?.textContent,
    );
  const unit = (id: string | number, col: string) =>
    visible(rowOf(id)?.querySelector(`[col-id="${col}"] .cell-unit`)?.textContent);
  const pinned = (col: string) =>
    visible(
      document.querySelector('.ag-row-pinned')?.querySelector(`[col-id="${col}"]`)?.textContent,
    );
  const pinnedFigure = (col: string) =>
    visible(
      document
        .querySelector('.ag-row-pinned')
        ?.querySelector(`[col-id="${col}"] .cell-with-unit > span:first-child`)?.textContent,
    );
  const pinnedUnit = (col: string) =>
    visible(
      document.querySelector('.ag-row-pinned')?.querySelector(`[col-id="${col}"] .cell-unit`)
        ?.textContent,
    );

  /** A matcher for an element whose whole text is `text`, without the isolation marks. */
  const said = (text: string) => (content: string) => visible(content) === text;

  /** Puts the focus on the first cell of a row, the way a click does. */
  async function focusRow(user: ReturnType<typeof userEvent.setup>, id: string | number) {
    const first = rowOf(id)?.querySelector<HTMLElement>('[col-id]');
    if (first === null || first === undefined) throw new Error(`no row ${String(id)}`);
    await user.click(first);
  }

  describe('level 1, the list of parties', () => {
    it('asks for the buyers, a page at a time', async () => {
      fakePartyLedgers();
      await open();
      await waitFor(() => {
        expect(ledgersApi.partyLedgersList).toHaveBeenCalledWith({
          side: 'buyer',
          text: null,
          offset: 0,
          limit: PAGE_ROWS,
        });
      });
    });

    it('shows the code, the name and the balance under its side, with the backend’s total (E2E-67)', async () => {
      fakePartyLedgers();
      await open();
      await waitFor(() => {
        expect(cell(20, 'debit')).toBe('2,880');
      }, SLOW_MS);
      expect(cell(20, 'code')).toBe('0020');
      expect(cell(20, 'name')).toBe(names.akram);
      expect(cell(20, 'credit')).toBe('');
      expect(cell(21, 'code')).toBe('0021');
      expect(cell(21, 'name')).toBe(names.arshad);
      expect(cell(21, 'debit')).toBe('18,684');
      // The footer is the plain total of the debit and of the credit, the backend's, not a sum of the rows.
      expect(pinned('name')).toBe(m('glossary-meezan'));
      expect(pinned('debit')).toBe('21,564');
      expect(pinned('credit')).toBe('0');
      const heads = Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map((head) =>
        head.getAttribute('col-id'),
      );
      expect(heads).toEqual(['code', 'name', 'debit', 'credit']);
    });

    it('the toggle shows the sellers: a credit balance is under the credit column, zero is 0', async () => {
      fakePartyLedgers();
      const { user, router } = await open();
      await waitFor(() => {
        expect(cell(20, 'debit')).toBe('2,880');
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('party-ledgers-side-seller')) }));
      await waitFor(() => {
        expect(cell(19, 'credit')).toBe('10,160');
      }, SLOW_MS);
      expect(ledgersApi.partyLedgersList).toHaveBeenLastCalledWith({
        side: 'seller',
        text: null,
        offset: 0,
        limit: PAGE_ROWS,
      });
      expect(cell(19, 'name')).toBe(names.karim);
      expect(cell(19, 'debit')).toBe('');
      expect(cell(18, 'name')).toBe(names.rahim);
      expect(cell(18, 'debit')).toBe('0');
      expect(cell(18, 'credit')).toBe('');
      expect(pinned('debit')).toBe('0');
      expect(pinned('credit')).toBe('10,160');
      expect(router.state.location.search).toMatchObject({ side: 'seller' });
    });

    it('the search box is sent as typed, in either script', async () => {
      fakePartyLedgers();
      const { user } = await open();
      await user.type(
        await screen.findByRole('textbox', { name: named(m('common-search-label')) }),
        'akram',
      );
      await waitFor(() => {
        expect(ledgersApi.partyLedgersList).toHaveBeenLastCalledWith(
          expect.objectContaining({ text: 'akram' }),
        );
      });
      await waitFor(() => {
        expect(rowOf(21)).toBeNull();
      }, SLOW_MS);
      expect(cell(20, 'name')).toBe(names.akram);
      // The footer follows the search: it is the backend's total for what was found.
      expect(pinned('debit')).toBe('2,880');
    });

    it('a person in the dead amount shows his balance, marked in words (PRD FR-10.10)', async () => {
      const dead: PartyLedgerRow[] = LEDGER_BUYERS.rows.map((row) =>
        row.account_id === 21 ? { ...row, is_dead: true, is_active: false } : row,
      );
      fakePartyLedgers({ buyers: { ...LEDGER_BUYERS, rows: dead } });
      await open();
      await waitFor(() => {
        expect(cell(21, 'debit')).toBe('18,684');
      }, SLOW_MS);
      expect(cell(21, 'name')).toBe(`${names.arshad} (${m('dead-tag')})`);
      expect(cell(20, 'name')).toBe(names.akram);
    });

    it('nothing yet: says why and offers the Index', async () => {
      fakePartyLedgers({
        buyers: {
          rows: [],
          count: 0,
          total_dr_paisa: LEDGER_BUYERS.total_cr_paisa,
          total_cr_paisa: LEDGER_BUYERS.total_cr_paisa,
        },
      });
      const { router, user } = await open();
      expect(
        await screen.findByText(said(m('party-ledgers-empty-buyer')), {}, SLOW_MS),
      ).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: m('party-ledgers-empty-action') }));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/index/accounts');
      });
    });

    it('nothing found by the search: a way out that clears it', async () => {
      fakePartyLedgers();
      const { user } = await open();
      await user.type(
        await screen.findByRole('textbox', { name: named(m('common-search-label')) }),
        'zzz',
      );
      expect(
        await screen.findByText(said(m('party-ledgers-no-match')), {}, SLOW_MS),
      ).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: m('party-ledgers-clear') }));
      await waitFor(() => {
        expect(cell(20, 'debit')).toBe('2,880');
      }, SLOW_MS);
    });

    it('a list that cannot be read says so, with a retry', async () => {
      fakePartyLedgers({ failList: refusal([{ code: 'ERR_INTERNAL' }]) });
      const { user } = await open();
      expect(
        await screen.findByText(said(m('party-ledgers-list-failed')), {}, SLOW_MS),
      ).toBeInTheDocument();
      fakePartyLedgers();
      await user.click(screen.getByRole('button', { name: m('common-retry') }));
      await waitFor(() => {
        expect(cell(20, 'debit')).toBe('2,880');
      }, SLOW_MS);
    });
  });

  describe('level 2, the statement of one party', () => {
    it('Enter on a buyer opens his statement: previous, charged, paid, balance (E2E-67)', async () => {
      fakePartyLedgers();
      const { user, router } = await open();
      await waitFor(() => {
        expect(cell(20, 'debit')).toBe('2,880');
      }, SLOW_MS);
      await focusRow(user, 20);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(cell('2026-10-10', 'charged')).toBe('12,880');
      }, SLOW_MS);
      expect(router.state.location.search).toMatchObject({ party: 20 });
      expect(ledgersApi.partyLedgersStatement).toHaveBeenLastCalledWith({
        account_id: 20,
        from: null,
        to: null,
        offset: 0,
        limit: PAGE_ROWS,
      });
      // 10-10-26: previous 0, charged 12,880, paid 10,000, balance 2,880 (Debit).
      expect(cell('2026-10-10', 'date')).toBe('10-10-2026');
      expect(cell('2026-10-10', 'previous')).toBe('0');
      expect(cell('2026-10-10', 'paid')).toBe('10,000');
      expect(figure('2026-10-10', 'balance')).toBe('2,880');
      expect(unit('2026-10-10', 'balance')).toBe(side.dr);
      // The buyer's headings, and the party's name and code on the head.
      const heads = Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map((head) =>
        visible(head.textContent).replace(/\s+/g, ' ').trim(),
      );
      expect(heads).toEqual([
        m('party-ledgers-col-date'),
        m('party-ledgers-col-previous'),
        m('party-ledgers-col-buyer-charged'),
        m('party-ledgers-col-buyer-paid'),
        m('party-ledgers-col-balance'),
      ]);
      expect(screen.getByRole('heading', { level: 2, name: names.akram })).toBeInTheDocument();
      // The footer is the backend's: opening, the totals of the dates, closing.
      expect(pinned('date')).toBe(m('glossary-meezan'));
      expect(pinnedFigure('previous')).toBe('0');
      expect(pinned('charged')).toBe('12,880');
      expect(pinned('paid')).toBe('10,000');
      expect(pinnedFigure('balance')).toBe('2,880');
      expect(pinnedUnit('balance')).toBe(side.dr);
    });

    it('a seller’s statement: the dates newest first, the advance of 01-09-26, balance 0 after the payment', async () => {
      fakePartyLedgers();
      const { user } = await open();
      await user.click(screen.getByRole('button', { name: named(m('party-ledgers-side-seller')) }));
      await waitFor(() => {
        expect(cell(18, 'name')).toBe(names.rahim);
      }, SLOW_MS);
      await focusRow(user, 18);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(cell('2026-10-10', 'charged')).toBe('24,316');
      }, SLOW_MS);
      // 10-10-26: previous 15,000 (Debit), credited 24,316, paid 9,316, balance 0.
      expect(figure('2026-10-10', 'previous')).toBe('15,000');
      expect(unit('2026-10-10', 'previous')).toBe(side.dr);
      expect(cell('2026-10-10', 'paid')).toBe('9,316');
      expect(figure('2026-10-10', 'balance')).toBe('0');
      expect(unit('2026-10-10', 'balance')).toBe('');
      // 01-09-26: the advance of 15,000 paid to him.
      expect(cell('2026-09-01', 'charged')).toBe('0');
      expect(cell('2026-09-01', 'paid')).toBe('15,000');
      expect(figure('2026-09-01', 'balance')).toBe('15,000');
      expect(unit('2026-09-01', 'balance')).toBe(side.dr);
      // A seller's headings say what his two columns are.
      const heads = Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map((head) =>
        visible(head.textContent).replace(/\s+/g, ' ').trim(),
      );
      expect(heads).toContain(m('party-ledgers-col-seller-charged'));
      expect(heads).toContain(m('party-ledgers-col-seller-paid'));
      expect(pinned('charged')).toBe('24,316');
      expect(pinned('paid')).toBe('24,316');
      expect(pinnedFigure('balance')).toBe('0');
    });

    it('the dates are sent as typed and a refused date shows its words under its box, the rows stay', async () => {
      fakePartyLedgers({
        refuseStatement: (input) =>
          input.from === 'x1' ? refusal([{ code: 'ERR_DATE_INVALID', field: 'from' }]) : null,
      });
      const { user } = await open('/bikri/party-ledgers?party=20');
      await waitFor(() => {
        expect(cell('2026-10-10', 'charged')).toBe('12,880');
      }, SLOW_MS);
      await user.type(screen.getByRole('textbox', { name: named(m('party-ledgers-from')) }), 'x1');
      expect(await screen.findByText(said(m('err-date-invalid')), {}, SLOW_MS)).toBeInTheDocument();
      // The page last shown stays on screen.
      expect(cell('2026-10-10', 'charged')).toBe('12,880');
      expect(ledgersApi.partyLedgersStatement).toHaveBeenLastCalledWith(
        expect.objectContaining({ account_id: 20, from: 'x1', to: null }),
      );
    });

    it('no entry in the dates: a way out that clears them', async () => {
      fakePartyLedgers({
        statements: {
          20: { ...STATEMENT_AKRAM, rows: [], count: 0 },
        },
      });
      const { user } = await open('/bikri/party-ledgers?party=20');
      expect(
        await screen.findByText(said(m('party-ledgers-statement-empty')), {}, SLOW_MS),
      ).toBeInTheDocument();
      await user.type(
        screen.getByRole('textbox', { name: named(m('party-ledgers-to')) }),
        '09-10-2026',
      );
      expect(
        await screen.findByText(said(m('party-ledgers-statement-no-match')), {}, SLOW_MS),
      ).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: m('party-ledgers-clear-dates') }));
      expect(
        await screen.findByText(said(m('party-ledgers-statement-empty')), {}, SLOW_MS),
      ).toBeInTheDocument();
    });

    it('a party that has no ledger is an error with a retry, never an empty page', async () => {
      fakePartyLedgers();
      await open('/bikri/party-ledgers?party=99');
      expect(
        await screen.findByText(said(m('party-ledgers-statement-failed')), {}, SLOW_MS),
      ).toBeInTheDocument();
    });
  });

  describe('level 3, the day slip', () => {
    /** Goes down the three levels with the keyboard, the way the Owner does: list, statement, slip. */
    async function downToTheSlip(user: ReturnType<typeof userEvent.setup>) {
      await waitFor(() => {
        expect(cell(20, 'debit')).toBe('2,880');
      }, SLOW_MS);
      await focusRow(user, 20);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(cell('2026-10-10', 'charged')).toBe('12,880');
      }, SLOW_MS);
      await focusRow(user, '2026-10-10');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(document.querySelector('section dl')).not.toBeNull();
      }, SLOW_MS);
    }

    it('Enter on a date opens that day’s R35 slip for the party, the request the report screen makes', async () => {
      fakePartyLedgers();
      const { user, router } = await open();
      await downToTheSlip(user);
      expect(router.state.location.search).toMatchObject({
        side: 'buyer',
        party: 20,
        date: '2026-10-10',
      });
      expect(reportsApi.reportsRun).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'R35', filters: R35_SLIP_AKRAM_REQUEST.filters }),
      );
      // The lines of the slip: the day's total 12,880, paid 10,000, the new balance 2,880 (Debit).
      const card = document.querySelector('section dl');
      const lines = Array.from(card?.children ?? []).map((line) => [
        visible(line.querySelector('dt')?.textContent),
        visible(line.querySelector('dd')?.textContent),
      ]);
      expect(lines).toContainEqual([m('report-r35-today-total'), '12,880']);
      expect(lines).toContainEqual([m('report-r35-paid-today'), '10,000']);
      expect(lines).toContainEqual([m('report-r35-new-balance'), `2,880${side.dr}`]);
      // The head says whose slip and which day.
      const comma = language === 'ur' ? '، ' : ', ';
      expect(
        screen.getByRole('heading', {
          level: 2,
          name: said(`${names.akram} 0020${comma}10-10-2026`),
        }),
      ).toBeInTheDocument();
    });

    it('a seller’s slip is asked for with his account and the same date', async () => {
      fakePartyLedgers();
      await open('/bikri/party-ledgers?side=seller&party=18&date=2026-10-10');
      await waitFor(() => {
        expect(reportsApi.reportsRun).toHaveBeenCalledWith(
          expect.objectContaining({ id: 'R35', filters: R35_SLIP_RAHIM_REQUEST.filters }),
        );
      }, SLOW_MS);
    });

    it('F9 and the print button show the print preview of that slip; Esc closes it, then goes back', async () => {
      fakePartyLedgers();
      const { user, router } = await open();
      await downToTheSlip(user);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await waitFor(() => {
        expect(reportsApi.reportsPrintData).toHaveBeenCalledWith(
          expect.objectContaining({ id: 'R35', filters: R35_SLIP_AKRAM_REQUEST.filters }),
        );
      }, SLOW_MS);
      // Esc closes the preview and stays on the slip.
      press('Escape');
      await waitFor(() => {
        expect(document.querySelector('section dl')).not.toBeNull();
      }, SLOW_MS);
      expect(router.state.location.search).toMatchObject({ date: '2026-10-10' });
      // F9 opens it again.
      press('F9');
      await waitFor(() => {
        expect(reportsApi.reportsPrintData).toHaveBeenCalledTimes(2);
      }, SLOW_MS);
    });

    it('Esc goes back one level at a time: slip, statement, list, then the dashboard', async () => {
      fakePartyLedgers();
      const { user, router } = await open();
      await downToTheSlip(user);
      press('Escape');
      await waitFor(() => {
        expect(router.state.location.href).toBe('/bikri/party-ledgers?side=buyer&party=20');
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell('2026-10-10', 'charged')).toBe('12,880');
      }, SLOW_MS);
      press('Escape');
      await waitFor(() => {
        expect(router.state.location.href).toBe('/bikri/party-ledgers?side=buyer');
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell(20, 'debit')).toBe('2,880');
      }, SLOW_MS);
      press('Escape');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/');
      }, SLOW_MS);
    });

    it('the back button of the statement goes to the list of the party’s own kind', async () => {
      fakePartyLedgers();
      // The address leaves the side out: the backend says he is a seller, so the list is the sellers.
      const { user, router } = await open('/bikri/party-ledgers?party=18');
      await waitFor(() => {
        expect(cell('2026-10-10', 'charged')).toBe('24,316');
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: m('party-ledgers-back-list') }));
      await waitFor(() => {
        expect(router.state.location.search).toEqual({ side: 'seller' });
      });
    });
  });

  it('a language switch in the middle of level 2 keeps the level and the typed dates', async () => {
    fakePartyLedgers();
    const { user, router } = await open('/bikri/party-ledgers?party=20');
    await waitFor(() => {
      expect(cell('2026-10-10', 'charged')).toBe('12,880');
    }, SLOW_MS);
    await user.type(
      screen.getByRole('textbox', { name: named(m('party-ledgers-from')) }),
      '01-10-2026',
    );
    press('KeyL', { altKey: true });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await screen.findByRole(
      'heading',
      { level: 1, name: named(message(other, 'party-ledgers-title')) },
      SLOW_MS,
    );
    await waitFor(() => {
      expect(cell('2026-10-10', 'charged')).toBe('12,880');
    }, SLOW_MS);
    expect(router.state.location.search).toMatchObject({ party: 20 });
    expect(
      screen.getByRole('textbox', { name: named(message(other, 'party-ledgers-from')) }),
    ).toHaveValue('01-10-2026');
    expect(
      screen.getByRole('heading', {
        level: 2,
        name: other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
      }),
    ).toBeInTheDocument();
  });
});
