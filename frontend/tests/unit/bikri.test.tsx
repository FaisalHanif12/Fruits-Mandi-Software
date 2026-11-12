// The بکری بک screen (plan 12, PRD 12.1 to 12.5, E2E-26), in the Urdu and in the English app, by
// keyboard, against a fake backend that answers with the numbers of Event 1 (PRD 4.4). The screen
// works out no money: every number on it is a number the fake backend sent.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { IDS } from '../helpers/bikri-data';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  names,
  openBook,
  salesGrid,
  textOf,
  typeCodeAndEnter,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** The text of the whole page, without the isolation marks Fluent puts around variables. */
const pageText = () => visible(document.body.textContent);

/** The value in the summary or split row called `labelId`. */
function rowText(section: HTMLElement, language: Language, labelId: string): string {
  const wanted = message(language, labelId);
  const label = within(section).getByText((content) => visible(content) === wanted);
  return visible(label.closest('div')?.textContent);
}

describe.each(LANGUAGES)('the sale book screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const dateBox = () =>
    screen.getByRole('textbox', { name: named(message(language, 'bikri-header-date')) });

  it('opens with the date ready to type and nothing asked of the backend', async () => {
    const { backend } = await openBook(language);
    expect(dateBox()).toHaveFocus();
    expect(backend.previews).toHaveLength(0);
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
    // The empty state says what to do, with a button.
    expect(await screen.findByText(message(language, 'bikri-empty-title'))).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: named(message(language, 'bikri-empty-action')) }),
    ).toBeInTheDocument();
  });

  // Plan 71 H1, H4: the header card is gone. The date and the number are in the top row with the
  // buttons; the truck, the زمیندار, the اڈا and everything behind "More" are not on the screen.
  it('has only the date and the number in the top row, and none of the old header fields', async () => {
    await openBook(language);
    const row = dateBox().closest('form')?.parentElement?.parentElement;
    if (row == null) throw new Error('no top row');
    // The date, the number ("New" until saved) and the buttons of the row are one row.
    const inRow = (id: string) =>
      within(row).getByText((text) => visible(text) === visible(message(language, id)));
    expect(inRow('bikri-head-number')).toBeInTheDocument();
    expect(inRow('bikri-head-number-new')).toBeInTheDocument();
    expect(
      within(row).getByRole('button', {
        name: (label) => visible(label).startsWith(visible(message(language, 'bikri-save'))),
      }),
    ).toBeInTheDocument();
    // The store is a field of each lot and the note a field of each line, so they are not in this list.
    for (const id of [
      'bikri-header-truck',
      'bikri-header-bill-date',
      'bikri-header-total-freight',
      'bikri-header-book',
      'bikri-header-driver',
      'bikri-header-driver-phone',
    ]) {
      expect(
        screen.queryByRole('textbox', { name: named(message(language, id)) }),
        id,
      ).not.toBeInTheDocument();
      expect(
        screen.queryByRole('combobox', { name: named(message(language, id)) }),
        id,
      ).not.toBeInTheDocument();
    }
    for (const id of ['glossary-zamindar', 'glossary-adda']) {
      expect(
        screen.queryByRole('combobox', { name: named(message(language, id)) }),
        id,
      ).not.toBeInTheDocument();
    }
  });

  it('Enter on the date goes to the first lot, whose seller is empty: nothing is filled from a header', async () => {
    const { backend, user } = await openBook(language);
    await user.keyboard('{Enter}');
    const lots = lotsGrid(language);
    await waitFor(() => {
      expect(focusedIn(lots)).toBe('0:seller');
    });
    expect(textOf(lots, 0, 'seller')).toBe('');
    // Rahim's code is the first thing typed; his item and packing come from his last lot.
    await typeCodeAndEnter(user, '0');
    await waitFor(() => {
      expect(textOf(lots, 0, 'seller')).toBe(names(language).rahim);
      expect(textOf(lots, 0, 'item')).toBe(names(language).banana);
      expect(textOf(lots, 0, 'packing')).toBe(names(language).box);
    });
    // The header sent is the date alone: the backend takes nothing else (plan 71 H1).
    await waitFor(() => {
      expect(backend.lastDraft().header).toEqual({ entry_date: '10-10-2026' });
    });
  });

  it('the empty state button puts the cursor on the seller of the first lot', async () => {
    const { user } = await openBook(language);
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'bikri-empty-action')) }),
    );
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('0:seller');
    });
  });
});

describe.each(LANGUAGES)('Event 1 by keyboard (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('header, lot and two sale lines with the keys only; every number is the backend’s', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]).toMatchObject({
        qty_received: '50',
        freight: '3000',
        labour: { method: 'manual', value: '5' },
        munshiana: { method: 'manual', value: '50' },
        cash_paid: '5000',
      });
    });
    await typeTwoLines(user, language);
    // Three fields and three Enters a line: buyer, qty, rate (NFR-USE-6). The second line's qty
    // started at what the lot has left, and Enter took it.
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales.slice(0, 2)).toMatchObject([
        { buyer_account_id: IDS.akram, qty: '20', seller_rate: '600' },
        { buyer_account_id: IDS.arshad, qty: '30', seller_rate: '580' },
      ]);
    });
    const sales = salesGrid(language);
    // Every number comes with one answer of the backend for both lines. The answer for the first
    // line alone arrives earlier, so nothing is read until the answer for both lines is on screen.
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('12,880');
      expect(textOf(sales, 0, 'amount')).toBe('12,000');
      expect(textOf(sales, 0, 'sellerComm')).toBe('720');
      expect(textOf(sales, 1, 'amount')).toBe('17,400');
      expect(textOf(sales, 1, 'buyerTotal')).toBe('18,684');
      expect(textOf(lotsGrid(language), 0, 'net')).toBe('19,316');
      const summary = screen
        .getByRole('heading', { name: named(message(language, 'bikri-summary-title')) })
        .closest('section');
      if (summary === null) throw new Error('no summary');
      expect(rowText(summary, language, 'glossary-kham-bikri')).toContain('29,400');
      expect(rowText(summary, language, 'bikri-summary-deductions')).toContain('10,084');
      expect(rowText(summary, language, 'glossary-safi-bikri')).toContain('19,316');
      expect(rowText(summary, language, 'bikri-summary-buyer-total')).toContain('31,564');
      const split = screen
        .getByRole('heading', { name: named(message(language, 'bikri-split-title')) })
        .closest('section');
      if (split === null) throw new Error('no split');
      expect(rowText(split, language, 'bikri-split-shared')).toContain('3,948');
      const splitText = visible(split.textContent);
      expect(splitText).toContain('1,580');
      expect(splitText).toContain('1,184');
    });
  });

  it('ⓘ on the seller commission of line 1 shows 12,000 × 6 ÷ 100 = 720, by click and by Ctrl+I', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(textOf(sales, 0, 'sellerComm')).toBe('720');
    });
    const working = message(language, 'working-percent', {
      base: '12,000',
      bps: '6',
      result: '720',
    });
    await user.click(
      within(cellOf(sales, 0, 'sellerComm')).getByRole('button', {
        name: message(language, 'calc-info-label'),
      }),
    );
    await waitFor(() => {
      expect(pageText()).toContain(working);
    });
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(pageText()).not.toContain(working);
    });
    // The same working from the keyboard: the cell has focus, Ctrl+I.
    await user.click(cellOf(sales, 0, 'sellerComm'));
    await user.keyboard('{Control>}i{/Control}');
    await waitFor(() => {
      expect(pageText()).toContain(working);
    });
  });

  it('every calculated amount of a line has its working, and it is the backend’s own', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('12,880');
    });
    for (const field of ['amount', 'sellerComm', 'buyerTotal']) {
      expect(
        within(cellOf(sales, 0, field)).getByRole('button', {
          name: message(language, 'calc-info-label'),
        }),
      ).toBeInTheDocument();
    }
    await user.click(
      within(cellOf(sales, 0, 'buyerTotal')).getByRole('button', {
        name: message(language, 'calc-info-label'),
      }),
    );
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'working-buyer-total', {
          value: '12,000',
          comm: '720',
          laga: '160',
          result: '12,880',
        }),
      );
    });
  });
});

describe.each(LANGUAGES)(
  'E2E-26: what the lot has left, the rate hint and Ctrl+D (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    it('line 2 starts at 30, the rate is empty with 600 as grey text, and Ctrl+D copies the rate above', async () => {
      const { user } = await openBook(language);
      await typeHeaderAndLot(user, language);
      const sales = salesGrid(language);
      await user.keyboard('0{Enter}20{Enter}600{Enter}{Enter}');
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('1:buyer');
      });
      await user.keyboard('1{Enter}');
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('1:qty');
      });
      // The quantity is what the lot has left, from the backend: 50 less the 20 sold above. It
      // arrives with the answer for the line that now has a buyer.
      await waitFor(() => {
        expect(textOf(sales, 1, 'qty')).toBe('30');
      });
      // The rate stays empty. The last rate is grey words ("Last 600") in the cell and never part of the draft.
      const rate = cellOf(sales, 1, 'rate');
      await waitFor(() => {
        expect(rate).toHaveClass('cell-hint');
      });
      expect(visible(rate.textContent).trim()).toBe(
        message(language, 'bikri-rate-hint-cell', { rate: '600' }),
      );
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('1:rate');
      });
      await user.keyboard('{Control>}d{/Control}');
      await waitFor(() => {
        expect(rate).not.toHaveClass('cell-hint');
      });
      expect(visible(rate.textContent).trim()).toBe('600');
    });

    it('never fills the rate by itself: without a typed rate the draft has an empty rate', async () => {
      const { backend, user } = await openBook(language);
      await typeHeaderAndLot(user, language);
      await user.keyboard('0{Enter}20{Enter}600{Enter}{Enter}1{Enter}{Enter}');
      await waitFor(() => {
        expect(backend.lastDraft().lots[0]?.sales[1]).toMatchObject({
          buyer_account_id: IDS.arshad,
          qty: '30',
          seller_rate: '',
        });
      });
    });

    it('a sum such as 20+5 is sent as typed text', async () => {
      const { backend, user } = await openBook(language);
      await typeHeaderAndLot(user, language);
      await user.keyboard('0{Enter}20+5{Enter}');
      await waitFor(() => {
        expect(backend.lastDraft().lots[0]?.sales[0]?.qty).toBe('20+5');
      });
    });
  },
);

describe.each(LANGUAGES)('keys of the grids (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('↑ on the first line goes back up to the lot, and Alt+↓ opens the list of a picker cell', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    // The buyer cell's list is open (plan 46, PRD FR-7.16), and an open list keeps ↑ ↓ for choosing.
    // Esc closes it, and then ↑ goes back up to the lot, whose seller list opens in its turn.
    expect(await screen.findAllByRole('option')).not.toHaveLength(0);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('option')).not.toBeInTheDocument();
    });
    await user.keyboard('{ArrowUp}');
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).not.toBeNull();
    });
    expect(await screen.findAllByRole('option')).not.toHaveLength(0);
    // Esc closes it; Alt+↓ opens the full list again without typing; Esc closes it once more.
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('option')).not.toBeInTheDocument();
    });
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    expect(await screen.findAllByRole('option')).not.toHaveLength(0);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('option')).not.toBeInTheDocument();
    });
  });

  it('the paid cell is the last cell of the line: what is typed goes to the backend as typed, and what is left shows in the details', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    // The grid has the paid column after the buyer total, and no payment column (PRD FR-12.44).
    const ids = Array.from(
      salesGrid(language).querySelectorAll('.ag-header-cell'),
      (cell) => cell.getAttribute('col-id') ?? '',
    );
    expect(ids.at(-2)).toBe('paid');
    expect(ids).not.toContain('payment');
    // Nothing typed: the line is sent with no paid amount (the backend takes it as 0).
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.paid).toBeNull();
    });
    expect('payment_type' in (backend.lastDraft().lots[0]?.sales[0] ?? {})).toBe(false);
    // Typing in the paid cell: the text is sent as typed.
    await user.click(cellOf(salesGrid(language), 0, 'paid'));
    await user.keyboard('5000{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.paid).toBe('5000');
    });
    // The details of the line show what is left on the account: the backend's number, with its working.
    await user.click(cellOf(salesGrid(language), 0, 'rate'));
    const details = document.querySelector<HTMLElement>(
      'section[aria-labelledby^="line-details-"]',
    );
    if (details === null) throw new Error('no details of the line');
    expect(within(details).getByText(message(language, 'bikri-line-left'))).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('the summary of a lot and of the whole sale (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('the switch shows the lot or the whole sale, from the same backend numbers', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    // Both lines are answered: the whole sale says 29,400 only from then on. The first line alone
    // is answered earlier, with 12,000.
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    const summary = screen
      .getByRole('heading', { name: named(message(language, 'bikri-summary-title')) })
      .closest('section');
    if (summary === null) throw new Error('no summary');
    const whole = within(summary).getByRole('button', {
      name: named(message(language, 'bikri-summary-whole')),
    });
    expect(whole).toHaveAttribute('aria-pressed', 'false');
    await user.click(whole);
    await waitFor(() => {
      expect(whole).toHaveAttribute('aria-pressed', 'true');
      expect(visible(summary.textContent)).toContain('29,400');
    });
  });
});
