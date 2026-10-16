// The بکری بک screen (plan 12, PRD 12.1 to 12.5, E2E-26), in the Urdu and in the English app, by
// keyboard, against a fake backend that answers with the numbers of Event 1 (PRD 4.4). The screen
// works out no money: every number on it is a number the fake backend sent.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  cellOf,
  combo,
  focusedIn,
  lotsGrid,
  named,
  names,
  openBook,
  salesGrid,
  textOf,
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
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
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

  it('opens with the truck number ready to type and nothing asked of the backend', async () => {
    const { backend } = await openBook(language);
    expect(await combo(language, 'bikri-header-truck')).toHaveFocus();
    expect(backend.previews).toHaveLength(0);
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
    // The empty state says what to do, with a button.
    expect(await screen.findByText(message(language, 'bikri-empty-title'))).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: message(language, 'bikri-empty-action') }),
    ).toBeInTheDocument();
  });

  it('a known truck number fills the adda, the zamindar and the driver, and the lot starts from them', async () => {
    const { backend, user } = await openBook(language);
    await user.keyboard('LES-1234{Enter}');
    await waitFor(() => {
      expect(
        screen.getByRole('combobox', { name: named(message(language, 'glossary-adda')) }),
      ).toHaveValue(names(language).swat);
    });
    expect(
      screen.getByRole('combobox', { name: named(message(language, 'glossary-zamindar')) }),
    ).toHaveValue(names(language).rahim);
    // The note says where the values came from, so nothing is filled without being seen.
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'bikri-header-filled-from', { number: 1, date: '09-10-2026' }),
      );
    });
    // His item and packing, and the seller of the lot, are in the cells as Enter stops.
    await user.keyboard('{Enter}{Enter}{Enter}');
    const lots = lotsGrid(language);
    await waitFor(() => {
      expect(textOf(lots, 0, 'seller')).toBe(names(language).rahim);
      expect(textOf(lots, 0, 'item')).toBe(names(language).banana);
      expect(textOf(lots, 0, 'packing')).toBe(names(language).box);
    });
    await waitFor(() => {
      expect(backend.lastDraft().header).toMatchObject({
        truck_no: 'LES-1234',
        zamindar_account_id: 17,
        adda_id: 1,
      });
    });
  });

  it('a truck with a sale on this day shows the notice with its two buttons; "continue" takes it away', async () => {
    const { user } = await openBook(language);
    await user.keyboard('LES-1234');
    const sentence = message(language, 'bikri-truck-same-day', { number: 1, date: '10-10-2026' });
    const notice = async () =>
      screen
        .findAllByRole('status')
        .then((all) => all.find((box) => visible(box.textContent).includes(sentence)));
    await waitFor(async () => {
      expect(await notice()).toBeDefined();
    });
    const box = await notice();
    if (box === undefined) throw new Error('no notice');
    // Opening the saved sale works since plan 13 (bikri-save.test.tsx follows it to the end).
    expect(
      within(box).getByRole('button', { name: named(message(language, 'bikri-truck-open')) }),
    ).toBeEnabled();
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'bikri-truck-continue')),
      }),
    );
    await waitFor(() => {
      expect(pageText()).not.toContain(sentence);
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
        { buyer_account_id: 19, qty: '20', seller_rate: '600' },
        { buyer_account_id: 20, qty: '30', seller_rate: '580' },
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
      expect(textOf(sales, 1, 'buyerComm')).toBe('1,044');
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
    for (const field of ['amount', 'sellerComm', 'buyerComm', 'buyerTotal']) {
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
      await user.keyboard('1{Enter}20{Enter}600{Enter}');
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('1:buyer');
      });
      await user.keyboard('2{Enter}');
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('1:qty');
      });
      // The quantity is what the lot has left, from the backend: 50 less the 20 sold above. It
      // arrives with the answer for the line that now has a buyer.
      await waitFor(() => {
        expect(textOf(sales, 1, 'qty')).toBe('30');
      });
      // The rate stays empty. The last rate is grey text in the cell and never part of the draft.
      const rate = cellOf(sales, 1, 'rate');
      await waitFor(() => {
        expect(rate).toHaveClass('cell-hint');
      });
      expect(visible(rate.textContent).trim()).toBe('600');
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
      await user.keyboard('1{Enter}20{Enter}600{Enter}2{Enter}{Enter}');
      await waitFor(() => {
        expect(backend.lastDraft().lots[0]?.sales[1]).toMatchObject({
          buyer_account_id: 20,
          qty: '30',
          seller_rate: '',
        });
      });
    });

    it('a sum such as 20+5 is sent as typed text', async () => {
      const { backend, user } = await openBook(language);
      await typeHeaderAndLot(user, language);
      await user.keyboard('1{Enter}20+5{Enter}');
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
    await user.keyboard('{ArrowUp}');
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).not.toBeNull();
    });
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    expect(await screen.findAllByRole('option')).not.toHaveLength(0);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('option')).not.toBeInTheDocument();
    });
  });

  it('a click or Space changes ادھاری to نقدی and back, and the backend is told each time', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const sales = salesGrid(language);
    // The backend says first: the buyer has credit, so the line is ادھاری.
    await waitFor(() => {
      expect(textOf(sales, 0, 'payment')).toBe(message(language, 'glossary-udhari'));
    });
    // A click on the cell moves to the next word.
    await user.click(cellOf(sales, 0, 'payment'));
    await waitFor(() => {
      expect(textOf(sales, 0, 'payment')).toBe(message(language, 'glossary-naqdi'));
    });
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.payment_type).toBe('cash');
    });
    // Space moves on again, back to ادھاری.
    await user.keyboard(' ');
    await waitFor(() => {
      expect(textOf(sales, 0, 'payment')).toBe(message(language, 'glossary-udhari'));
    });
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.payment_type).toBe('credit');
    });
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
