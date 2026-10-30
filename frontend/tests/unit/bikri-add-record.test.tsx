// Plan 41 on the بکری screen, the hardest screen to leave: a header, a grid of lots and a grid of sale
// lines, all typed and unsaved. A buyer or an item that does not exist is added from the cell where
// it is needed: Enter on the name that matches nothing opens the short form over the screen, F10
// saves the record and the cell takes it. Esc leaves the cell as it was. The header, the lot and
// the numbers are exactly as they were typed, and the بکری itself is not saved by the form's F10.
// Both languages, against the fake backend of Event 1.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import * as mastersApi from '../../src/features/masters/api';
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
  cellOf,
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

// The short forms are lazy chunks; load them once so the first test is not waiting for the import.
beforeAll(async () => {
  await import('../../src/features/masters/components/QuickAddMaster');
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

describe.each(LANGUAGES)('adding from the cells of the بکری screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  // A buyer who is not there, typed in the script of the screen.
  const newBuyer = language === 'ur' ? 'نیا گاہک' : 'zzz';
  // An item that is not there, typed in the script of the screen.
  const newItem = language === 'ur' ? 'آم' : 'Mango';
  const quickAccount = () => ({ name: named(message(language, 'accountpicker-add-title')) });
  const quickItem = () => ({ name: named(message(language, 'masters-item-new')) });

  /** Types text into the cell the cursor is on: the first key opens the editor, the rest follows. */
  async function typeInCell(user: Awaited<ReturnType<typeof openBook>>['user'], text: string) {
    await user.keyboard(text.slice(0, 1));
    await waitFor(() => {
      expect(document.activeElement?.tagName).toBe('INPUT');
    });
    await user.keyboard(text.slice(1));
  }

  /** The header and the lot of Event 1 are typed, two sale lines too; the third asks for a buyer. */
  async function askForNewBuyer() {
    const book = await openBook(language);
    await typeHeaderAndLot(book.user, language);
    await typeTwoLines(book.user, language);
    await typeInCell(book.user, newBuyer);
    await screen.findByText(message(language, 'search-no-match'));
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
    expect(textOf(salesGrid(language), 0, 'qty')).toBe('20');
    expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
  }

  it('Enter on a buyer who is not there opens the short form; F10 saves him and the cell takes him', async () => {
    const { backend, user } = await askForNewBuyer();
    // The row names the typed text; Enter on it (nothing matches, and it is not a code) adds.
    expect(
      screen.getByRole('button', {
        name: (name) =>
          visible(name).startsWith(
            message(language, 'search-add-row-named', { noun: 'buyer', text: newBuyer }),
          ),
      }),
    ).toBeInTheDocument();
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog', quickAccount());
    const typedName = await within(dialog).findByLabelText<HTMLInputElement>(
      named(
        message(language, language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en'),
      ),
    );
    expect(typedName.value).toBe(newBuyer);
    // It is the short form: no father or firm, and a link to the full form.
    expect(
      within(dialog).queryByLabelText(named(message(language, 'accounts-field-father-en'))),
    ).toBeNull();
    expect(
      within(dialog).getByRole('button', {
        name: named(message(language, 'search-quick-full-form')),
      }),
    ).toBeInTheDocument();

    press('F10');
    await waitFor(() => {
      expect(vi.mocked(accountApi.accountsSave)).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog', quickAccount())).not.toBeInTheDocument();
    });
    // F10 in the form saved the account only: the بکری was not saved under it.
    expect(backend.saves).toHaveLength(0);
    expectScreenAsTyped();
    // The cell has the new buyer, and the cursor goes on from there with Enter as usual.
    await waitFor(() => {
      const active = document.activeElement;
      if (!(active instanceof HTMLInputElement)) throw new Error('the buyer cell is not open');
      expect(active.value).toContain(newBuyer);
    });
  });

  it('Esc in the form returns to the same screen with the cell as it was, and saves nothing', async () => {
    const { backend, user } = await askForNewBuyer();
    await user.keyboard('{Enter}');
    await screen.findByRole('dialog', quickAccount());
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog', quickAccount())).not.toBeInTheDocument();
    });
    expect(vi.mocked(accountApi.accountsSave)).not.toHaveBeenCalled();
    expect(backend.saves).toHaveLength(0);
    expectScreenAsTyped();
    // The cell he was typing in is still open with the same text, so he can go on.
    await waitFor(() => {
      const active = document.activeElement;
      if (!(active instanceof HTMLInputElement)) throw new Error('the cell is not open');
      expect(active.value).toBe(newBuyer);
    });
  });

  it('an item that is not in the list is added from the lot: F10 saves it as a fruit sold by نگ and the cell takes it', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    const lots = lotsGrid(language);
    // A click selects the cell, a second click opens its picker (plan 32, R7).
    await user.click(cellOf(lots, 0, 'item'));
    await user.click(cellOf(lots, 0, 'item'));
    await waitFor(() => {
      expect(document.activeElement?.tagName).toBe('INPUT');
    });
    await user.keyboard(`{Control>}a{/Control}${newItem}`);
    await screen.findByText(message(language, 'search-no-match'));
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog', quickItem());
    expect(
      within(dialog).getByLabelText<HTMLInputElement>(
        named(
          message(language, language === 'ur' ? 'masters-field-name-ur' : 'masters-field-name-en'),
        ),
      ).value,
    ).toBe(newItem);
    // The short form of an item asks the group and how it is sold, and starts at نگ.
    const choice = (id: string) =>
      visible(
        within(dialog).getByRole('combobox', { name: named(message(language, id)) }).textContent,
      );
    expect(choice('masters-item-group')).toBe(message(language, 'masters-item-group-fruit'));
    expect(choice('masters-item-sold-by')).toBe(message(language, 'masters-sold-by-unit'));

    press('F10');
    await waitFor(() => {
      expect(vi.mocked(mastersApi.mastersSave)).toHaveBeenCalledTimes(1);
    });
    const [kind, draft] = vi.mocked(mastersApi.mastersSave).mock.calls[0] ?? [];
    expect(kind).toBe('item');
    // The code is empty: the backend gives the next one.
    expect(draft).toMatchObject({
      code: '',
      names: language === 'ur' ? { urdu: newItem } : { english: newItem },
      text: { item_group: 'fruit', sold_by: 'unit' },
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog', quickItem())).not.toBeInTheDocument();
    });
    // The cursor stays in the cell with the new item chosen; Enter carries on as usual.
    await waitFor(() => {
      const active = document.activeElement;
      if (!(active instanceof HTMLInputElement)) throw new Error('the item cell is not open');
      expect(active.value).toBe(newItem);
    });
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(textOf(lots, 0, 'item')).toBe(newItem);
    });
    // The lot is as it was typed.
    expect(textOf(lots, 0, 'qty')).toBe('50');
    expect(textOf(lots, 0, 'seller')).toBe(names(language).rahim);
  });
});
