// Plan 32, R8 on the بکری screen, the hardest screen to leave: a header, a grid of lots and a grid of
// sale lines, all typed and unsaved. A buyer that does not exist is asked for in a sale line; the
// "add a new buyer" button opens the Index form above the screen. Save or Esc returns to the same
// screen, and the header, the lot and its numbers are exactly as they were typed.
// Both languages, against the fake backend of Event 1.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  lotsGrid,
  named,
  names,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

// The Index form is a lazy chunk; load it once so the first test is not waiting for the import.
beforeAll(async () => {
  await import('../../src/features/masters/components/IndexAddDialog');
});

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the Index form from a sale line (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** The Index form's dialog, told apart from the grid's own popup editor (also a dialog). */
  const indexForm = () => ({ name: named(message(language, 'accounts-new-title')) });

  /** The header and the lot of Event 1 are typed; a buyer that is not there is asked for. */
  async function askForNewBuyer() {
    const book = await openBook(language);
    await typeHeaderAndLot(book.user, language);
    // Two complete sale lines, so the بکری could be saved: the third line asks for a new buyer.
    await typeTwoLines(book.user, language);
    await book.user.keyboard('zzz');
    const add = await screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'search-add-buyer')),
    });
    await book.user.click(add);
    return book;
  }

  /** Everything typed before the form was asked for. */
  function expectScreenAsTyped() {
    const lots = lotsGrid(language);
    expect(textOf(lots, 0, 'seller')).toBe(names(language).rahim);
    expect(textOf(lots, 0, 'packing')).toBe(names(language).box);
    expect(textOf(lots, 0, 'qty')).toBe('50');
    expect(
      screen.getByRole('combobox', { name: named(message(language, 'glossary-zamindar')) }),
    ).toHaveValue(names(language).rahim);
  }

  it('Save returns to the same screen: the header, the lot and the sale grid are as they were', async () => {
    const { backend, user } = await askForNewBuyer();
    const dialog = await screen.findByRole('dialog', indexForm());
    const english = await within(dialog).findByLabelText<HTMLInputElement>(
      named(message(language, 'accounts-field-name-en')),
    );
    expect(english.value).toBe('zzz');
    expect(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'accounts-field-type')),
      }),
    ).toHaveTextContent(message(language, 'accounts-type-buyer'));

    press('F10');
    await waitFor(() => {
      expect(vi.mocked(accountApi.accountsSave)).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog', indexForm())).not.toBeInTheDocument();
    });
    // F10 in the Index form saved the account only: the بکری was not saved under it.
    expect(backend.saves).toHaveLength(0);
    expectScreenAsTyped();
    // The two lines are still there, and the بکری could be saved: F10 on the screen saves it, so
    // the nothing above is the dialog owning F10, not a sale that could not be saved anyway.
    expect(textOf(salesGrid(language), 0, 'qty')).toBe('20');
    expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    const cell = document.activeElement;
    if (!(cell instanceof HTMLInputElement)) throw new Error('the buyer cell is not open');
    expect(cell.value).toBe('zzz');
    await user.keyboard('{Escape}');
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
  });

  it('Esc returns to the same screen with the lot as it was, and saves nothing', async () => {
    const { backend, user } = await askForNewBuyer();
    await screen.findByRole('dialog', indexForm());
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog', indexForm())).not.toBeInTheDocument();
    });
    expect(vi.mocked(accountApi.accountsSave)).not.toHaveBeenCalled();
    expect(backend.saves).toHaveLength(0);
    expectScreenAsTyped();
    // The cell he was typing in is still open with the same text, so he can go on.
    await waitFor(() => {
      const active = document.activeElement;
      if (!(active instanceof HTMLInputElement)) throw new Error('the cell is not open');
      expect(active.value).toBe('zzz');
    });
  });
});
