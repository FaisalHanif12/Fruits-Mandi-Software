// Saving a بکری (plan 13, PRD FR-12.8, FR-6.7, FR-7.9, FR-7.57), in the Urdu and the English app, by
// keyboard, against a fake backend that answers with the numbers of Event 1 (PRD 4.4). The screen
// sends the draft as typed and shows what comes back; it works out no money.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import * as bikriApi from '../../src/features/bikri/api';
import { registerPrintAfterSave } from '../../src/features/bikri/after-save';
import { LANGUAGES } from '../../src/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { money } from '../helpers/accounts';
import { previewFor } from '../helpers/bikri-data';
import {
  combo,
  focusedIn,
  lotsGrid,
  named,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
  waitForLineFill,
} from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  // Reset, not only clear: a rejection queued with mockRejectedValueOnce by a test that failed early
  // must not reach the next test.
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

const pageText = () => visible(document.body.textContent);

/** The message after a save, with the number the backend gave (PRD FR-7.7). */
async function savedToast(language: 'ur' | 'en') {
  await waitFor(() => {
    expect(pageText()).toContain(message(language, 'bikri-saved-toast', { number: 1 }));
  });
}

/** The answer of the preview with the cash going below zero (PRD FR-6.7: 20,000 − 25,000). */
function cashBelowZero(draft: BikriDraft): BikriPreview {
  const preview = previewFor(draft);
  return {
    ...preview,
    cash: {
      cash_before_paisa: money(20_000),
      change_paisa: money(-25_000),
      cash_after_paisa: money(-5_000),
      below_zero: true,
      lowers_cash: true,
      needs_confirm: true,
    },
  };
}

describe.each(LANGUAGES)('saving a بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('F10 sends the draft as typed, shows the message, and opens a new empty بکری on the truck field', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    // The second line's quantity was filled by the screen (30 left): wait for it, then save.
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    });
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    const sent = backend.saves[0];
    // The confirmation is always sent: false when the entry does not take cash below zero.
    expect(sent?.confirm).toBe(false);
    // What is saved is the text typed, line by line (the backend works every number out again).
    expect(sent?.draft.lots[0]?.sales.slice(0, 2)).toMatchObject([
      { buyer_account_id: 19, qty: '20', seller_rate: '600' },
      { buyer_account_id: 20, qty: '30', seller_rate: '580' },
    ]);
    // The message, with the number the backend gave, and then a new empty بکری is ready.
    await savedToast(language);
    const truck = await combo(language, 'bikri-header-truck');
    await waitFor(() => {
      expect(truck).toHaveFocus();
    });
    expect(truck).toHaveValue('');
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
    // The saved id is kept: the way back to it is on the screen.
    expect(pageText()).toContain(message(language, 'bikri-last-saved', { number: 1 }));
    // The draft the backend removed is thrown away once more, in case one was on its way.
    expect(draftsApi.draftsDiscard).toHaveBeenCalledWith('bikri');
  });

  it('Ctrl+S saves too, and a second F10 while the first is on its way saves nothing twice', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    let release: () => void = () => undefined;
    vi.mocked(bikriApi.bikriSave).mockImplementation(
      (draft, confirm) =>
        new Promise((resolve) => {
          backend.saves.push({ draft, confirm });
          release = () => {
            resolve({ bikri_id: 41, number: 1, entry_id: 7, preview: previewFor(draft) });
          };
        }),
    );
    await user.keyboard('{Control>}s{/Control}');
    press('F10');
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    release();
    await savedToast(language);
    expect(backend.saves).toHaveLength(1);
  });

  it('Ctrl+Enter saves and opens a new بکری, and calls the printing that plan 14 registers', async () => {
    const printed = vi.fn();
    const unregister = registerPrintAfterSave(printed);
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await user.keyboard('{Control>}{Enter}{/Control}');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(printed).toHaveBeenCalledTimes(1);
    });
    expect(printed.mock.calls[0]?.[0]).toMatchObject({ bikri_id: 41, number: 1 });
    expect(await combo(language, 'bikri-header-truck')).toHaveValue('');
    unregister();
  });

  it('F10 does not call the printing: only Ctrl+Enter does', async () => {
    const printed = vi.fn();
    const unregister = registerPrintAfterSave(printed);
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    await savedToast(language);
    expect(printed).not.toHaveBeenCalled();
    unregister();
  });

  it('the Save button has its key on it and saves', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const button = screen.getByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'bikri-save')),
    });
    expect(visible(button.textContent)).toContain('F10');
    await user.click(button);
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
  });
});

describe.each(LANGUAGES)('cash below zero (%s, PRD FR-6.7)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const dialog = () => screen.findByRole('alertdialog');

  it('F10 asks first: focus is on No, Esc is No, nothing is saved, and the نقد cell is selected', async () => {
    const { backend, user } = await openBook(language, cashBelowZero);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-confirm-title'));
    });
    press('F10');
    const box = await dialog();
    expect(visible(box.textContent)).toContain('-5,000');
    const no = within(box).getByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'common-cash-confirm-no')),
    });
    expect(no).toHaveFocus();
    expect(backend.saves).toHaveLength(0);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
    expect(backend.saves).toHaveLength(0);
    // The نقد cell of the lot has focus again.
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('0:cash');
    });
  });

  it('Yes saves with the confirmation', async () => {
    const { backend, user } = await openBook(language, cashBelowZero);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-confirm-title'));
    });
    press('F10');
    const box = await dialog();
    await user.click(
      within(box).getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'common-cash-confirm-yes')),
      }),
    );
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    expect(backend.saves[0]?.confirm).toBe(true);
    await savedToast(language);
  });

  it('when the backend refuses for the cash, the box opens with the cash as it is now', async () => {
    // The answer on screen says nothing about cash; the backend then refuses the save.
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    // The screen filled the second line's quantity (30 left): wait for it before saving.
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    });
    vi.mocked(bikriApi.bikriSave).mockRejectedValueOnce(
      fail({
        code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED',
        field: null,
        values: { cash_after_paisa: -500_000 },
      }),
    );
    vi.mocked(bikriApi.bikriPreview).mockImplementation((draft) =>
      Promise.resolve(cashBelowZero(draft)),
    );
    press('F10');
    const box = await dialog();
    expect(visible(box.textContent)).toContain('-5,000');
    await user.click(
      within(box).getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'common-cash-confirm-yes')),
      }),
    );
    await waitFor(() => {
      expect(backend.saves.at(-1)?.confirm).toBe(true);
    });
  });
});

describe.each(LANGUAGES)('a refused save (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the backend’s problem at its cell and in words, keeps everything typed, and clears it on the next edit', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    // The screen filled the second line's quantity (30 left): wait for it before saving.
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    });
    vi.mocked(bikriApi.bikriSave).mockRejectedValueOnce(
      fail({
        code: 'ERR_QTY_EXCEEDS_LOT',
        field: 'lots[0].sales[1].qty',
        values: { available: 5 },
      }),
    );
    press('F10');
    const words = message(language, 'err-qty-exceeds-lot', { available: 5 });
    await waitFor(() => {
      expect(pageText()).toContain(words);
    });
    expect(backend.saves).toHaveLength(0);
    // The cell is marked and the line is still there.
    expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    expect(pageText()).not.toContain(message(language, 'bikri-saved-toast', { number: 1 }));
    // Typing something changes the draft: the refusal belongs to the old text and goes.
    await user.click(await combo(language, 'bikri-header-truck'));
    await user.keyboard('X');
    await waitFor(() => {
      expect(pageText()).not.toContain(words);
    });
  });

  it('a language switch after a refusal keeps every value and the refusal, now in the other language (F39)', async () => {
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    // The screen filled the second line's quantity (30 left): wait for it before saving.
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    });
    vi.mocked(bikriApi.bikriSave).mockRejectedValueOnce(
      fail({
        code: 'ERR_QTY_EXCEEDS_LOT',
        field: 'lots[0].sales[1].qty',
        values: { available: 5 },
      }),
    );
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-qty-exceeds-lot', { available: 5 }));
    });
    await user.keyboard('{Alt>}l{/Alt}');
    const other = language === 'ur' ? 'en' : 'ur';
    await waitFor(() => {
      expect(pageText()).toContain(message(other, 'err-qty-exceeds-lot', { available: 5 }));
    });
    // The same text typed, in the grids rebuilt for the other direction.
    expect(textOf(salesGrid(other), 0, 'qty')).toBe('20');
    expect(textOf(salesGrid(other), 1, 'qty')).toBe('30');
    expect(textOf(salesGrid(other), 1, 'buyerTotal')).toBe('18,684');
    expect(textOf(lotsGrid(other), 0, 'qty')).toBe('50');
    // And the save, tried again, goes through with the text that was typed.
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    expect(backend.saves[0]?.draft.lots[0]?.qty_received).toBe('50');
  });

  it('a sale with no lines says so only when he tries to save (PRD FR-12.8)', async () => {
    const { backend, user } = await openBook(language);
    await user.keyboard('LES-1234');
    // Nothing is said while he types: a missing line is not an error yet.
    expect(pageText()).not.toContain(message(language, 'err-no-sale-lines'));
    // The screen fills the blank line's qty from the lot (PRD FR-7.52) a moment after the preview.
    // A refusal belongs to the draft it was for, so the fill must land before F10, not after it.
    await waitForLineFill(language);
    vi.mocked(bikriApi.bikriSave).mockRejectedValueOnce(
      fail({ code: 'ERR_NO_SALE_LINES', field: 'lots', values: {} }),
    );
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-no-sale-lines'));
    });
    expect(backend.saves).toHaveLength(0);
  });

  it('a problem with no field is told in words near the buttons, with its reference', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    vi.mocked(bikriApi.bikriSave).mockRejectedValueOnce(
      fail({ code: 'ERR_DB_BUSY', field: null, values: {} }),
    );
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-db-busy'));
    });
    // He can press save again: the screen kept the draft.
    press('F10');
    await savedToast(language);
  });
});

describe.each(LANGUAGES)('leaving a بکری with typed work (%s, PRD FR-7.9)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const prompt = () => screen.findByRole('alertdialog');
  const buttonOf = (box: HTMLElement, id: string) =>
    within(box).getByRole('button', {
      name: (name) => visible(name).startsWith(message(language, id)),
    });

  it('an untouched screen leaves without asking', async () => {
    const { router, user } = await openBook(language);
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'layout-nav-dashboard')) }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });

  it('typed work asks with three buttons: back stays, leave goes and throws the draft away', async () => {
    const { router, user } = await openBook(language);
    await user.keyboard('LES-1234');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'layout-nav-dashboard')) }),
    );
    const box = await prompt();
    expect(router.state.location.pathname).toBe('/bikri/book');
    // Back: still on the screen, with the truck number.
    await user.click(buttonOf(box, 'common-back'));
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
    expect(await combo(language, 'bikri-header-truck')).toHaveValue('LES-1234');
    // Leave: goes, and the draft on the backend is thrown away.
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'layout-nav-dashboard')) }),
    );
    await user.click(buttonOf(await prompt(), 'layout-save-leave'));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
    expect(draftsApi.draftsDiscard).toHaveBeenCalledWith('bikri');
  });

  it('Save in the prompt saves the بکری and then leaves', async () => {
    const { backend, router, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'layout-nav-dashboard')) }),
    );
    await user.click(buttonOf(await prompt(), 'layout-save-save'));
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });

  it('a refused save in the prompt keeps him on the screen with the problems shown', async () => {
    const { backend, router, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    // The screen filled the second line's quantity (30 left): wait for it before saving.
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    });
    vi.mocked(bikriApi.bikriSave).mockRejectedValueOnce(
      fail({
        code: 'ERR_QTY_EXCEEDS_LOT',
        field: 'lots[0].sales[1].qty',
        values: { available: 5 },
      }),
    );
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'layout-nav-dashboard')) }),
    );
    await user.click(buttonOf(await prompt(), 'layout-save-save'));
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-qty-exceeds-lot', { available: 5 }));
    });
    expect(router.state.location.pathname).toBe('/bikri/book');
    expect(backend.saves).toHaveLength(0);
  });

  it('the same-truck notice’s open button asks which بکری it is, then leaving asks first', async () => {
    const { user, router } = await openBook(language);
    await user.keyboard('LES-1234');
    const sentence = message(language, 'bikri-truck-same-day', { number: 1, date: '10-10-2026' });
    const box = await waitFor(async () => {
      const found = (await screen.findAllByRole('status')).find((candidate) =>
        visible(candidate.textContent).includes(sentence),
      );
      if (found === undefined) throw new Error('no notice');
      return found;
    });
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'bikri-truck-open')) }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriFind).toHaveBeenCalledWith({
        number: 1,
        truck_no: null,
        seller_id: null,
        date: null,
        item_id: null,
      });
    });
    // He had typed a truck number, so the saved بکری opens after the prompt.
    await user.click(buttonOf(await prompt(), 'layout-save-leave'));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ id: 41 });
    });
    expect(draftsApi.draftsDiscard).toHaveBeenCalled();
  });
});
