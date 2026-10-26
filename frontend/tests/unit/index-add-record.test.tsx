// Plan 32, R8: a picker that finds nothing offers a button that opens the Index form of that kind
// above the screen, with the typed text in the name box. Save or Esc returns to the same screen,
// and everything typed there is still there. The new record is not filled in by itself: the field
// shows the same text with its list open, and the user picks the new record from it.
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

/** The "add" button of an empty list, by its words (a term in a message carries isolation marks). */
/** The Index form's dialog, told apart from any other dialog of the screen. */
let indexForm: () => { name: (name: string) => boolean };

const addButton = (language: Language, id: string) => ({
  name: (name: string) => visible(name).startsWith(message(language, id)),
});

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

describe.each(LANGUAGES)('the Index form from a picker, on the voucher screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    store = fakeVouchers();
    indexForm = () => ({ name: named(message(language, 'accounts-new-title')) });
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
  async function typeEntryAndAskForNewBuyer(user: ReturnType<typeof userEvent.setup>) {
    await user.type(await labelled(language, 'vouchers-field-amount'), '5000');
    await user.type(await labelled(language, 'vouchers-field-narration'), 'note-one');
    const buyer = await screen.findByRole('combobox', {
      name: named(message(language, 'vouchers-field-buyer')),
    });
    await user.click(buyer);
    await user.keyboard('xyz');
    const add = await screen.findByRole('button', addButton(language, 'search-add-buyer'));
    await user.click(add);
    return buyer as HTMLInputElement;
  }

  it('a click on the buyer field lists the buyers, and an unknown name offers "add a new buyer"', async () => {
    const { user } = await open();
    const buyer = await screen.findByRole('combobox', {
      name: named(message(language, 'vouchers-field-buyer')),
    });
    await user.click(buyer);
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(2);
    });
    await user.keyboard('xyz');
    expect(await screen.findByText(message(language, 'search-no-match'))).toBeInTheDocument();
    expect(screen.getByRole('button', addButton(language, 'search-add-buyer'))).toBeInTheDocument();
  });

  it('looks like the Index page: the main menu says انڈکس while it is open, its heading is the Index title, and the cursor is in the first empty box', async () => {
    const { user } = await open();
    const current = () =>
      Array.from(document.querySelectorAll('[aria-current="page"]')).map((item) =>
        visible(item.textContent),
      );
    expect(current().join(' ')).toContain(message(language, 'layout-nav-bikri'));
    await typeEntryAndAskForNewBuyer(user);
    const dialog = await screen.findByRole('dialog', indexForm());
    expect(
      within(dialog).getByRole('heading', {
        level: 1,
        name: named(message(language, 'accounts-new-title')),
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(current().join(' ')).toContain(message(language, 'layout-nav-index'));
    });
    // The typed text is in its name box; the cursor is in a box that is still empty.
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

  it('opens the new-account form for a buyer with xyz in the name box; Save returns with nothing lost', async () => {
    const { user } = await open();
    const buyer = await typeEntryAndAskForNewBuyer(user);

    // The Index form is above the screen: its heading, the type, and the typed text as the name.
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByText(message(language, 'accounts-new-title'))).toBeInTheDocument();
    const english = await within(dialog).findByLabelText<HTMLInputElement>(
      named(message(language, 'accounts-field-name-en')),
    );
    expect(english.value).toBe('xyz');
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
      names: { english: 'xyz' },
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(store.saves).toHaveLength(0);

    // Back on the same screen: the amount and the narration are still typed.
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('5,000');
    expect((await labelled(language, 'vouchers-field-narration')).value).toBe('note-one');

    // The field has the same text and its list is open; the new buyer is in it, not filled in.
    await waitFor(() => {
      expect(buyer).toHaveFocus();
    });

    expect(buyer.value).toBe('xyz');
    const rows = await screen.findAllByRole('option');
    expect(rows).toHaveLength(1);
    expect(visible(rows[0]?.textContent)).toContain('xyz');
    await user.keyboard('{Enter}');
    // He is picked from the list now (his Urdu name is empty in this fake, so the words are not read).
    await waitFor(() => {
      expect(vi.mocked(accountApi.searchAccountsTouch)).toHaveBeenCalledTimes(1);
    });
    expect(buyer).toHaveAttribute('aria-expanded', 'false');
  });

  it('Esc closes the form without saving, and the screen is as it was', async () => {
    const { user } = await open();
    const buyer = await typeEntryAndAskForNewBuyer(user);
    await screen.findByRole('dialog');
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
    expect(buyer.value).toBe('xyz');
    // Nothing was added, so the list is empty again with the same button.
    expect(
      await screen.findByRole('button', addButton(language, 'search-add-buyer')),
    ).toBeInTheDocument();
  });

  it('the other name the app fills on open is not a change of his: Esc then closes without asking', async () => {
    vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
      Promise.resolve(
        suggestionsFor({ toUrdu: { xyz: ['زیڈ'] }, toEnglish: { زیڈ: 'Xyz' } }, text, direction),
      ),
    );
    const { user } = await open();
    await typeEntryAndAskForNewBuyer(user);
    const dialog = await screen.findByRole('dialog', indexForm());
    const urdu = await within(dialog).findByLabelText<HTMLInputElement>(
      named(message(language, 'accounts-field-name-ur')),
    );
    await waitFor(() => {
      expect(urdu.value).toBe('زیڈ');
    });
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog', indexForm())).not.toBeInTheDocument();
    });
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    expect(vi.mocked(accountApi.accountsSave)).not.toHaveBeenCalled();
  });

  it('Esc after typing in the form asks "save changes?" first', async () => {
    const { user } = await open();
    await typeEntryAndAskForNewBuyer(user);
    const dialog = await screen.findByRole('dialog');
    const english = await within(dialog).findByLabelText<HTMLInputElement>(
      named(message(language, 'accounts-field-name-en')),
    );
    await user.type(english, 'a');
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
});
