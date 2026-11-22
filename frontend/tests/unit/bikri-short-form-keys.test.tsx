// Plan 95 item 4 (PRD FR-12.52, FR-7.69, Umar 23-11-2026): the short forms opened from a pickers of the بکری screen are
// used by the keyboard from start to end. Enter or ↓ goes to the next field, Enter on the last field or F10 saves and the
// cell takes the record, Tab stays inside the form, and only Esc closes it without saving. Umar lost a new چکتا buyer
// by pressing Tab and ↓ in his English name. In the Urdu and the English app.

import { act, screen, waitFor, within } from '@testing-library/react';
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
import type { ChuktaBuyer } from '../../src/bindings/ChuktaBuyer';
import {
  lotsGrid,
  cellOf,
  named,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

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

const NEW_BUYER: ChuktaBuyer = {
  id: 7,
  account_id: 9,
  name_ur: 'نیا گاہک',
  name_en: 'Naya Gahak',
  phone: '03001234567',
  last_date: null,
};

describe.each(LANGUAGES)('the short forms of the بکری screen, by keyboard (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const newBuyer = language === 'ur' ? 'نیا گاہک' : 'zzz';
  const say = (id: string) => message(language, id);
  async function typeInCell(user: Awaited<ReturnType<typeof openBook>>['user'], text: string) {
    await user.keyboard(text.slice(0, 1));
    await waitFor(() => {
      expect(document.activeElement?.tagName).toBe('INPUT');
    });
    await user.keyboard(text.slice(1));
  }
  /** The buyer cell of a third line is asked for a name nobody has; the second add row opens the چکتا form. */
  async function openChuktaForm() {
    const book = await openBook(language);
    await typeHeaderAndLot(book.user, language);
    await typeTwoLines(book.user, language);
    await typeInCell(book.user, newBuyer);
    await screen.findByText(say('search-no-match'));
    await book.user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    const dialog = await screen.findByRole('dialog', { name: named(say('chukta-add-title')) });
    return { ...book, dialog };
  }
  const nameField = (dialog: HTMLElement, id: 'ur' | 'en') =>
    within(dialog).getByLabelText<HTMLInputElement>(named(say(`accounts-field-name-${id}`)));
  const phoneField = (dialog: HTMLElement) =>
    within(dialog).getByLabelText<HTMLInputElement>(named(say('accounts-field-phone1')));
  const here = (dialog: HTMLElement) => dialog.contains(document.activeElement);

  describe('a new چکتا buyer', () => {
    it('Tab and ↓ from the name go to the next field, the form stays open, and nothing is saved (Umar lost the form here)', async () => {
      const { user, dialog } = await openChuktaForm();
      const first = document.activeElement;
      await user.keyboard('{Tab}');
      expect(screen.getByRole('dialog', { name: named(say('chukta-add-title')) })).toBe(dialog);
      expect(here(dialog)).toBe(true);
      expect(document.activeElement).not.toBe(first);
      await user.keyboard('{ArrowDown}');
      expect(dialog).toBeInTheDocument();
      expect(here(dialog)).toBe(true);
      await user.keyboard('{ArrowUp}');
      expect(dialog).toBeInTheDocument();
      expect(here(dialog)).toBe(true);
      expect(accountApi.chuktaAdd).not.toHaveBeenCalled();
    });

    it('Tab goes round inside the form and never leaves it', async () => {
      const { user, dialog } = await openChuktaForm();
      for (let step = 0; step < 8; step += 1) {
        await user.keyboard('{Tab}');
        expect(dialog).toBeInTheDocument();
        expect(here(dialog)).toBe(true);
      }
      await user.keyboard('{Shift>}{Tab}{/Shift}');
      expect(here(dialog)).toBe(true);
      expect(dialog).toBeInTheDocument();
    });

    it('Enter goes from field to field, and Enter on the phone, the last field, saves him and the cell takes him', async () => {
      vi.mocked(accountApi.chuktaAdd).mockResolvedValue(NEW_BUYER);
      const { user, dialog, backend } = await openChuktaForm();
      // Both names: the one that was typed is in its box, the other is typed here.
      const empty = nameField(dialog, language === 'ur' ? 'en' : 'ur');
      await user.click(empty);
      if (empty.value === '') await user.keyboard(language === 'ur' ? 'Naya Gahak' : 'نیا گاہک');
      const phone = phoneField(dialog);
      await user.click(phone);
      await user.keyboard('03001234567');
      expect(accountApi.chuktaAdd).not.toHaveBeenCalled();
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(accountApi.chuktaAdd).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(accountApi.chuktaAdd).mock.calls[0]?.[0]).toMatchObject({
        phone: '03001234567',
      });
      await waitFor(() => {
        expect(
          screen.queryByRole('dialog', { name: named(say('chukta-add-title')) }),
        ).not.toBeInTheDocument();
      });
      // The بکری itself was not saved by the form, and the lines are as typed.
      expect(backend.saves).toHaveLength(0);
      expect(textOf(salesGrid(language), 0, 'qty')).toBe('20');
    });

    it('Esc closes it from any field with nothing saved', async () => {
      const { user, dialog } = await openChuktaForm();
      await user.keyboard('{Tab}');
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(
          screen.queryByRole('dialog', { name: named(say('chukta-add-title')) }),
        ).not.toBeInTheDocument();
      });
      expect(accountApi.chuktaAdd).not.toHaveBeenCalled();
      expect(dialog).not.toBeInTheDocument();
    });
  });

  describe('a new seller from the lots', () => {
    it('Tab and ↓ in the short form of a seller keep it open', async () => {
      const book = await openBook(language);
      await typeHeaderAndLot(book.user, language);
      const lots = lotsGrid(language);
      // A second click on the seller cell opens its picker; a name nobody has asks for the short form.
      await book.user.click(cellOf(lots, 0, 'seller'));
      await book.user.click(cellOf(lots, 0, 'seller'));
      await waitFor(() => {
        expect(document.activeElement?.tagName).toBe('INPUT');
      });
      await book.user.keyboard(`{Control>}a{/Control}${newBuyer}`);
      await screen.findByText(say('search-no-match'));
      await book.user.keyboard('{Enter}');
      const dialog = await screen.findByRole('dialog', {
        name: named(say('accountpicker-add-title')),
      });
      await book.user.keyboard('{Tab}');
      expect(dialog).toBeInTheDocument();
      expect(here(dialog)).toBe(true);
      await book.user.keyboard('{ArrowDown}');
      expect(dialog).toBeInTheDocument();
      expect(here(dialog)).toBe(true);
      await book.user.keyboard('{Escape}');
      await waitFor(() => {
        expect(dialog).not.toBeInTheDocument();
      });
      press('Escape');
    });
  });
  describe('a new item from the lots (its last field is a list)', () => {
    const newItem = language === 'ur' ? 'آم' : 'Mango';
    async function openItemForm() {
      const book = await openBook(language);
      await typeHeaderAndLot(book.user, language);
      const lots = lotsGrid(language);
      await book.user.click(cellOf(lots, 0, 'item'));
      await book.user.click(cellOf(lots, 0, 'item'));
      await waitFor(() => {
        expect(document.activeElement?.tagName).toBe('INPUT');
      });
      await book.user.keyboard(`{Control>}a{/Control}${newItem}`);
      await screen.findByText(say('search-no-match'));
      await book.user.keyboard('{Enter}');
      const dialog = await screen.findByRole('dialog', { name: named(say('masters-item-new')) });
      return { ...book, dialog };
    }
    const listOf = (dialog: HTMLElement, id: string) =>
      within(dialog).getByRole('combobox', { name: named(say(id)) });

    it('Enter on the last field, with its list closed, saves the item; with the list open it only picks', async () => {
      const { user, dialog } = await openItemForm();
      const soldBy = listOf(dialog, 'masters-item-sold-by');
      act(() => {
        soldBy.focus();
      });
      // Esc-free way in: ↓ opens the list. Enter then picks and closes it, and saves nothing.
      await user.keyboard('{ArrowDown}');
      await waitFor(() => {
        expect(soldBy).toHaveAttribute('aria-expanded', 'true');
      });
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(soldBy).toHaveAttribute('aria-expanded', 'false');
      });
      expect(mastersApi.mastersSave).not.toHaveBeenCalled();
      expect(dialog).toBeInTheDocument();
      // The list is closed and the cursor is on the last field: Enter saves.
      act(() => {
        soldBy.focus();
      });
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
      });
    });

    it('Enter on a list that is not the last field still opens it', async () => {
      const { user, dialog } = await openItemForm();
      const group = listOf(dialog, 'masters-item-group');
      act(() => {
        group.focus();
      });
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(group).toHaveAttribute('aria-expanded', 'true');
      });
      expect(mastersApi.mastersSave).not.toHaveBeenCalled();
    });
  });
});
