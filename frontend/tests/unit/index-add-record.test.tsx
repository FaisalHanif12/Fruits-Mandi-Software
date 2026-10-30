// Plan 41, PRD FR-7.69, FR-10.8: the short form of a buyer that is not there has a link "Full form"
// that opens the Index form of the account above the screen, with the typed name in the name box.
// (Before, plan 32 R8, a button of the empty list opened it at once.) Save or Esc returns to the same
// screen, and everything typed there is still there. A saved account is chosen in the field. The
// short form itself is in bikri-add-record.test.tsx and account-picker.test.tsx.
//
// Run on the voucher screen (a form) in both languages, against a fake backend. The بکری screen
// (a form and two grids) has its own run in bikri-add-record.test.tsx.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';
import { AKRAM, ARSHAD, fakeVouchers, type FakeVouchers } from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

// The Index form is a lazy chunk; load it once so the first test is not waiting for the import.
beforeAll(async () => {
  await import('../../src/features/masters/components/IndexAddDialog');
});

let store: FakeVouchers;
/** The accounts the search sees; a saved account is added here, as the real backend would. */
let searchable: ReturnType<typeof account>[] = [];

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  searchable = [AKRAM, ARSHAD];
  fakeAccounts(searchable);
  fakeAccountSearch(searchable);
  fakeMasters();
  // A saved account is found by the search from then on.
  const saveAccount = vi.mocked(accountApi.accountsSave).getMockImplementation();
  vi.mocked(accountApi.accountsSave).mockImplementation(async (draft, confirmed) => {
    const saved = await saveAccount?.(draft, confirmed);
    if (saved === undefined) throw new Error('no fake save');
    searchable.push(saved);
    return saved;
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)(
  'the full Index form from a picker, on the voucher screen (%s)',
  (language) => {
    // A buyer who is not there, typed in the script of the screen.
    const newBuyer = language === 'ur' ? 'زید' : 'xyz';
    const nameField = language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en';
    const quickForm = () => ({ name: named(message(language, 'accountpicker-add-title')) });
    const indexForm = () => ({ name: named(message(language, 'accounts-new-title')) });
    const addRowName = (name: string) =>
      visible(name).startsWith(
        message(language, 'search-add-row-named', { noun: 'buyer', text: newBuyer }),
      );

    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      store = fakeVouchers();
    });

    async function open() {
      const app = await renderApp('/bikri/voucher?type=receipt');
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-receipt-title'),
      });
      return { ...app, user: userEvent.setup() };
    }

    /** Types the amount and the narration, then asks for a buyer that does not exist. */
    async function askForNewBuyer(user: ReturnType<typeof userEvent.setup>) {
      await user.type(await labelled(language, 'vouchers-field-amount'), '5000');
      await user.type(await labelled(language, 'vouchers-field-narration'), 'note-one');
      const buyer = await screen.findByRole<HTMLInputElement>('combobox', {
        name: named(message(language, 'vouchers-field-buyer')),
      });
      await user.click(buyer);
      await user.keyboard(newBuyer);
      await user.click(await screen.findByRole('button', { name: addRowName }));
      return buyer;
    }

    /** The short form is open: its link opens the full Index form. */
    async function goToFullForm(user: ReturnType<typeof userEvent.setup>) {
      const quick = await screen.findByRole('dialog', quickForm());
      await user.click(
        within(quick).getByRole('button', {
          name: named(message(language, 'search-quick-full-form')),
        }),
      );
      return screen.findByRole('dialog', indexForm());
    }

    it('a click on the buyer field lists the buyers, and an unknown name shows the add row', async () => {
      const { user } = await open();
      const buyer = await screen.findByRole('combobox', {
        name: named(message(language, 'vouchers-field-buyer')),
      });
      await user.click(buyer);
      await waitFor(() => {
        expect(screen.getAllByRole('option')).toHaveLength(2);
      });
      await user.keyboard(newBuyer);
      expect(await screen.findByText(message(language, 'search-no-match'))).toBeInTheDocument();
      expect(await screen.findByRole('button', { name: addRowName })).toBeInTheDocument();
    });

    it('"Full form" looks like the Index page: the main menu says انڈکس while it is open, and the cursor is in the first empty box', async () => {
      const { user } = await open();
      const current = () =>
        Array.from(document.querySelectorAll('[aria-current="page"]')).map((item) =>
          visible(item.textContent),
        );
      expect(current().join(' ')).toContain(message(language, 'layout-nav-bikri'));
      await askForNewBuyer(user);
      const dialog = await goToFullForm(user);
      // The short form is gone; the full form is above the screen with the typed text as the name.
      // (Both forms are titled "New account"; the short form is the one with the "Full form" link.)
      await waitFor(() => {
        expect(
          screen.queryByRole('button', {
            name: named(message(language, 'search-quick-full-form')),
          }),
        ).not.toBeInTheDocument();
      });
      expect(screen.getAllByRole('dialog')).toHaveLength(1);
      expect(
        within(dialog).getByRole('heading', {
          level: 1,
          name: named(message(language, 'accounts-new-title')),
        }),
      ).toBeInTheDocument();
      expect(
        (
          await within(dialog).findByLabelText<HTMLInputElement>(
            named(message(language, nameField)),
          )
        ).value,
      ).toBe(newBuyer);
      await waitFor(() => {
        expect(current().join(' ')).toContain(message(language, 'layout-nav-index'));
      });
      await waitFor(() => {
        const active = document.activeElement;
        if (!(active instanceof HTMLInputElement) || !dialog.contains(active)) {
          throw new Error('the cursor is not in the form');
        }
        expect(active.value).toBe('');
      });
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('dialog', indexForm())).not.toBeInTheDocument();
      });
      expect(current().join(' ')).toContain(message(language, 'layout-nav-bikri'));
    });

    it('Save in the full form returns with nothing lost and the new buyer chosen in the field', async () => {
      const { user } = await open();
      const buyer = await askForNewBuyer(user);
      const dialog = await goToFullForm(user);
      expect(
        within(dialog).getByRole('combobox', {
          name: named(message(language, 'accounts-field-type')),
        }),
      ).toHaveTextContent(message(language, 'accounts-type-buyer'));

      // F10 inside the form saves the account, and does not also save the voucher under it.
      press('F10');
      await waitFor(() => {
        expect(vi.mocked(accountApi.accountsSave)).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
        account_type: 'buyer',
        names: language === 'ur' ? { urdu: newBuyer } : { english: newBuyer },
      });
      await waitFor(() => {
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      });
      expect(store.saves).toHaveLength(0);

      // Back on the same screen: the amount and the narration are still typed.
      expect((await labelled(language, 'vouchers-field-amount')).value).toBe('5,000');
      expect((await labelled(language, 'vouchers-field-narration')).value).toBe('note-one');

      // The new buyer is in the field, chosen, and the cursor is there for Enter to carry on.
      await waitFor(() => {
        expect(buyer.value).toBe(newBuyer);
      });
      expect(buyer).toHaveFocus();
      expect(buyer).toHaveAttribute('aria-expanded', 'false');
      expect(vi.mocked(accountApi.searchAccountsTouch)).toHaveBeenCalledTimes(1);
    });

    it('Esc closes the full form without saving, and the field is as it was', async () => {
      const { user } = await open();
      const buyer = await askForNewBuyer(user);
      await goToFullForm(user);
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      });
      expect(vi.mocked(accountApi.accountsSave)).not.toHaveBeenCalled();
      expect((await labelled(language, 'vouchers-field-amount')).value).toBe('5,000');
      expect((await labelled(language, 'vouchers-field-narration')).value).toBe('note-one');
      await waitFor(() => {
        expect(buyer).toHaveFocus();
      });
      expect(buyer.value).toBe(newBuyer);
      // Nothing was added, so the list still ends with the same add row.
      expect(await screen.findByRole('button', { name: addRowName })).toBeInTheDocument();
    });

    it('the other name the app fills on open is not a change of his: Esc then closes without asking', async () => {
      vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
        Promise.resolve(
          suggestionsFor({ toUrdu: { xyz: ['زیڈ'] }, toEnglish: { زید: 'Zaid' } }, text, direction),
        ),
      );
      const { user } = await open();
      await askForNewBuyer(user);
      const dialog = await goToFullForm(user);
      const other = await within(dialog).findByLabelText<HTMLInputElement>(
        named(
          message(
            language,
            language === 'ur' ? 'accounts-field-name-en' : 'accounts-field-name-ur',
          ),
        ),
      );
      await waitFor(() => {
        expect(other.value).not.toBe('');
      });
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('dialog', indexForm())).not.toBeInTheDocument();
      });
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
      expect(vi.mocked(accountApi.accountsSave)).not.toHaveBeenCalled();
    });

    it('Esc after typing in the full form asks "save changes?" first', async () => {
      const { user } = await open();
      await askForNewBuyer(user);
      const dialog = await goToFullForm(user);
      const name = await within(dialog).findByLabelText<HTMLInputElement>(
        named(message(language, nameField)),
      );
      await user.type(name, 'a');
      await user.keyboard('{Escape}');
      expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
      // The form stays open under the question.
      expect(document.querySelector('[role="dialog"]')).not.toBeNull();
      await user.click(
        screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-leave')) }),
      );
      await waitFor(() => {
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      });
      expect(vi.mocked(accountApi.accountsSave)).not.toHaveBeenCalled();
      expect((await labelled(language, 'vouchers-field-amount')).value).toBe('5,000');
    });
  },
);
