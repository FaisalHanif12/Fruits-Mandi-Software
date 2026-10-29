// The بکری بک screen, the other half (plan 12): the four states, an answer that arrives late, the
// override of a commission, the cash note, the problems at their cells, the lots, the freight split
// and a language switch in the middle of an entry. Urdu and English, against a fake backend that
// answers with the numbers of Event 1. The screen works out no money.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { money } from '../helpers/accounts';
import { previewFor } from '../helpers/bikri-data';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  names,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeCodeAndEnter,
  typeTwoLines,
  waitForLineFill,
} from '../helpers/bikri-screen';
import {
  DAY,
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { IDS } from '../helpers/bikri-data';
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

const pageText = () => visible(document.body.textContent);
const other = (language: Language): Language => (language === 'ur' ? 'en' : 'ur');
const textMatching = (text: string) => (content: string) => visible(content) === text;

describe.each(LANGUAGES)('the four states (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('loading: while today is not known the screen shows the loading state and no form', async () => {
    vi.mocked(appApi.dayStatus).mockReturnValue(new Promise(() => undefined));
    await renderApp('/bikri/book');
    await waitFor(() => {
      expect(document.querySelector('[aria-busy="true"]')).not.toBeNull();
    });
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('error: today cannot be read, so it says so with a retry that opens the screen', async () => {
    vi.mocked(appApi.dayStatus).mockRejectedValue(fail({ code: 'ERR_DB_BUSY' }));
    await renderApp('/bikri/book');
    expect(
      await screen.findByText(textMatching(message(language, 'bikri-load-failed'))),
    ).toBeInTheDocument();
    expect(pageText()).not.toContain('ERR_');
    vi.mocked(appApi.dayStatus).mockResolvedValue(DAY);
    await openRetry();
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'glossary-bikri-book')),
      }),
    ).toBeInTheDocument();
  });

  it('error: the numbers cannot be worked out, so the screen says so in words with a retry', async () => {
    let failing = true;
    const { user } = await openBook(language, (draft) =>
      failing ? Promise.reject(fail({ code: 'ERR_NOT_IMPLEMENTED' })) : previewFor(draft),
    );
    await user.keyboard('LES-1234');
    expect(
      // The preview waits 50 ms after the last key and the full suite loads the machine, so this
      // waits longer than the default for the real condition: the words on screen.
      await screen.findByText(
        textMatching(message(language, 'bikri-preview-failed')),
        {},
        { timeout: 5000 },
      ),
    ).toBeInTheDocument();
    expect(pageText()).toContain(message(language, 'err-not-implemented'));
    expect(pageText()).not.toContain('ERR_');
    failing = false;
    await openRetry();
    await screen.findByRole('heading', { name: named(message(language, 'bikri-summary-title')) });
    await waitFor(() => {
      expect(pageText()).not.toContain(message(language, 'bikri-preview-failed'));
    });
  });

  async function openRetry() {
    await userEvent
      .setup()
      .click(
        await screen.findByRole('button', { name: new RegExp(message(language, 'common-retry')) }),
      );
  }
});

describe.each(LANGUAGES)('an answer that arrives late (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('an older answer never replaces the answer for what is typed now (Code Standards F35)', async () => {
    let release: (() => void) | undefined;
    const slow = new Promise<void>((resolve) => {
      release = resolve;
    });
    // Each answer says its own number for the buyer total, so a test can tell which one is shown:
    // 1 for what was typed first (qty 20, which comes late), 2,500 for what was typed last (qty 25).
    const withTotal = (draft: BikriDraft, paisa: ReturnType<typeof money>) => {
      const preview = previewFor(draft);
      const line = preview.lots[0]?.lines[0]?.calc;
      if (line !== null && line !== undefined) line.buyer_total = paisa;
      return preview;
    };
    const { backend, user } = await openBook(language, async (draft) => {
      const qty = draft.lots[0]?.sales[0]?.qty;
      if (qty === '20') {
        await slow;
        return withTotal(draft, money(1));
      }
      return qty === '25' ? withTotal(draft, money(2500)) : previewFor(draft);
    });
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    await user.keyboard('0{Enter}20{Enter}');
    await waitFor(() => {
      expect(backend.previews.some((draft) => draft.lots[0]?.sales[0]?.qty === '20')).toBe(true);
    });
    // The user changes his mind before the answer comes.
    await user.click(cellOf(sales, 0, 'qty'));
    await user.keyboard('25{Enter}');
    // Release the old answer only once the answer for what is typed now is on screen.
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('2,500');
    });
    await act(async () => {
      release?.();
      await new Promise((resolve) => setTimeout(resolve, 80));
    });
    expect(textOf(sales, 0, 'buyerTotal')).toBe('2,500');
  });
});

describe.each(LANGUAGES)('numbers while the next answer is on its way (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('the last numbers stay on screen, with no empty flash, until the new ones come', async () => {
    let holding = false;
    let release: (() => void) | undefined;
    const { user } = await openBook(language, async (draft) => {
      const preview = previewFor(draft);
      if (!holding) return preview;
      await new Promise<void>((resolve) => {
        release = resolve;
      });
      return preview;
    });
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('12,880');
    });
    holding = true;
    await user.click(cellOf(sales, 0, 'qty'));
    await user.keyboard('25{Enter}');
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 120));
    });
    // The answer for 25 has not come: the numbers of the last answer are still there.
    expect(textOf(sales, 0, 'buyerTotal')).toBe('12,880');
    expect(screen.queryByTestId('skeleton-row')).not.toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: named(message(language, 'bikri-summary-title')) }),
    ).toBeInTheDocument();
    holding = false;
    await act(async () => {
      release?.();
      await new Promise((resolve) => setTimeout(resolve, 20));
    });
    expect(textOf(sales, 0, 'buyerTotal')).toBe('12,880');
  });
});

describe.each(LANGUAGES)('a commission typed on the line (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const sellerComm = () =>
    screen.findByRole('textbox', { name: named(message(language, 'bikri-line-seller-comm')) });

  /** The first answer has come: the screen knows whether a commission may be typed. */
  const waitForNumbers = () =>
    waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('12,880');
    });

  it('is sent as typed with its method when the setting allows it, and read-only when it does not', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitForNumbers();
    await user.click(cellOf(salesGrid(language), 0, 'qty'));
    const field = await sellerComm();
    expect(field).not.toHaveAttribute('readonly');
    await user.click(field);
    await user.keyboard('4');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.seller_comm).toMatchObject({
        method: 'percent',
        value: '4',
      });
    });
    expect(pageText()).not.toContain(message(language, 'bikri-line-override-off'));
  });

  it('is read-only with a note when the Owner switched the setting off', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      allow_line_override: false,
    }));
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitForNumbers();
    await user.click(cellOf(salesGrid(language), 0, 'qty'));
    expect(await sellerComm()).toHaveAttribute('readonly');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-line-override-off'));
    });
  });
});

describe.each(LANGUAGES)('the cash note (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the cash after the entry, from the backend, when cash would go below zero', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      cash: {
        cash_before_paisa: money(2000),
        change_paisa: money(-7000),
        cash_after_paisa: money(-5000),
        below_zero: true,
        lowers_cash: true,
        needs_confirm: true,
      },
    }));
    await typeHeaderAndLot(user, language);
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-confirm-title'));
    });
    expect(pageText()).toMatch(/5,000/);
  });

  it('shows no note when cash stays above zero', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await screen.findByRole('heading', { name: named(message(language, 'bikri-summary-title')) });
    expect(pageText()).not.toContain(message(language, 'common-cash-confirm-title'));
  });
});

describe.each(LANGUAGES)('problems at their cells (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('an error on the quantity turns that cell red and is written in words; a warning turns the buyer amber', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      errors: [
        {
          code: 'ERR_QTY_EXCEEDS_LOT',
          field: 'lots[0].sales[1].qty',
          values: { available: 30 },
        },
      ],
      warnings: [
        {
          code: 'ERR_CREDIT_LIMIT_EXCEEDED',
          field: 'lots[0].sales[0].buyer_account_id',
          values: { would_owe_paisa: 1288000, limit_paisa: 1000000 },
        },
      ],
    }));
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(cellOf(sales, 1, 'qty')).toHaveClass('cell-error');
    });
    expect(cellOf(sales, 0, 'buyer')).toHaveClass('cell-warn');
    // Colour is never the only signal: both are written, with the lot and line they are about.
    const text = pageText();
    expect(text).toContain(message(language, 'bikri-where-line', { lot: 1, line: 2 }));
    expect(text).toContain(message(language, 'err-qty-exceeds-lot', { available: 30 }));
    expect(text).toContain(
      message(language, 'err-credit-limit-exceeded', { would_owe: '12,880', limit: '10,000' }),
    );
  });

  it('a missing weight opens the "more" panel of the lot and asks for it under the weight field', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      errors: [{ code: 'ERR_WEIGHT_REQUIRED', field: 'lots[0].weight_received', values: {} }],
    }));
    await typeHeaderAndLot(user, language);
    // A weight item needs the weight received (the lot) and the weight sold (each line).
    const weights = await screen.findAllByRole('textbox', {
      name: named(message(language, 'bikri-more-weight')),
    });
    expect(weights.length).toBeGreaterThan(0);
    for (const weight of weights) expect(weight).toBeVisible();
    expect(pageText()).toContain(message(language, 'err-weight-required'));
  });
});

describe.each(LANGUAGES)('lots, the one-time buyer and the freight split (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('a new lot starts with the seller of the lot before it, and Ctrl+Delete takes it away', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'bikri-lots-add')) }),
    );
    const lots = lotsGrid(language);
    await waitFor(() => {
      expect(textOf(lots, 1, 'seller')).toBe(names(language).rahim);
    });
    await user.click(cellOf(lots, 1, 'qty'));
    await user.keyboard('{Control>}{Delete}{/Control}');
    await waitFor(() => {
      expect(lots.querySelector('.ag-row[row-index="1"]')).toBeNull();
    });
  });

  it('the one-time buyer account sends Enter to the name fields of the line', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    // The quantity the screen fills in is awaited first, and the code is typed step by step (the
    // helper waits for the editor and for the list). This test used to fail on a busy computer:
    // jsdom has no layout, and AG Grid closes the popup editor of a cell whose rectangle is all
    // zeros, 200 ms after it opens. tests/setup.ts now gives every element a rectangle, so the
    // editor stays open as it does in a real window. bikri-fill-while-typing.test.tsx proves a
    // fill that lands in the middle of the code does not close it either.
    await waitForLineFill(language);
    await typeCodeAndEnter(user, '0030');
    await waitFor(() => {
      const active = document.activeElement;
      expect(active?.closest('[data-pukhta-for]')).not.toBeNull();
    });
  });

  it('"split" puts the backend’s part of the total freight into the freight cell of the lot', async () => {
    const { user } = await openBook(language);
    vi.mocked(bikriApi.bikriSplitFreight).mockImplementation((_total, lots) =>
      Promise.resolve(lots.map((lot) => ({ key: lot.key, freight_paisa: money(4000) }))),
    );
    await typeHeaderAndLot(user, language);
    // The total freight is behind "More" in the header (plan 38, sale book).
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'bikri-header-more')) }),
    );
    const total = await screen.findByRole('textbox', {
      name: named(message(language, 'bikri-header-total-freight')),
    });
    await user.click(total);
    await user.keyboard('4000');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'bikri-header-split-freight')) }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriSplitFreight).toHaveBeenCalledWith('4000', expect.any(Array));
    });
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'freight')).toMatch(/^4,?000$/);
    });
  });
});

describe.each(LANGUAGES)('a language switch in the middle of an entry (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('keeps every value, the focused cell and the numbers, redraws the names, and typing goes on', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await user.keyboard('0{Enter}20{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:rate');
    });
    // The preview waits 50 ms after the last key, so the draft the backend has seen comes a
    // moment later.
    let before = backend.previews[0];
    await waitFor(() => {
      before = backend.lastDraft();
      expect(before.lots[0]?.sales[0]?.qty).toBe('20');
    });

    const next = other(language);
    await user.keyboard('{Alt>}l{/Alt}');
    // The names are drawn again in the other language, from the same records.
    await waitFor(() => {
      expect(textOf(lotsGrid(next), 0, 'seller')).toBe(names(next).rahim);
    });
    expect(textOf(salesGrid(next), 0, 'buyer')).toBe(names(next).akram);
    expect(textOf(lotsGrid(next), 0, 'item')).toBe(names(next).banana);
    expect(
      screen.getByRole('combobox', { name: named(message(next, 'glossary-zamindar')) }),
    ).toHaveValue(names(next).rahim);
    // Nothing typed was lost, and the numbers are the same.
    expect(textOf(lotsGrid(next), 0, 'qty')).toBe('50');
    expect(textOf(salesGrid(next), 0, 'qty')).toBe('20');
    expect(textOf(salesGrid(next), 0, 'buyerTotal')).toBe('12,880');
    // The cursor is back in the cell it was in.
    await waitFor(() => {
      expect(focusedIn(salesGrid(next))).toBe('0:rate');
    });
    // And the entry goes on: the rate, then the second line.
    await user.keyboard('600{Enter}1{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales.slice(0, 2)).toMatchObject([
        { buyer_account_id: IDS.akram, qty: '20', seller_rate: '600' },
        { buyer_account_id: IDS.arshad },
      ]);
    });
    expect(backend.lastDraft().header).toEqual(before?.header);
    expect(within(salesGrid(next)).getAllByRole('row').length).toBeGreaterThan(0);
  });
});
