// A پختہ party's own account on the account form and in the account list (plan 47, PRD FR-13.11,
// FR-10.2, FR-10.5, A79): the type list has four kinds; a پختہ گاہک or بیوپاری gets the next code of
// the پختہ series (0032 on, one counter for both sides) and is saved with the mark; a پختہ buyer
// starts with credit off (cash first); the edit form shows the kind and cannot change it; the list
// shows the kind after his name and the پختہ filter has the shared accounts and his. Urdu and English,
// against a fake backend.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import { CommandError } from '../../src/shared/api';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccounts } from '../helpers/accounts';
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
import * as real from '../helpers/pukhta-party-real';
import { message, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const RAHIM = account(31, 'seller', '0010', 'رحیم', 'Rahim');
const SHARED_BUYER = account(9, 'buyer', '0030', 'پختہ کھاتہ (خریدار)', 'Pukhta Khata (Buyer)', {
  is_system: true,
  is_pukhta_khata: true,
});
const SHARED_SELLER = account(
  7,
  'seller',
  '0031',
  'پختہ کھاتہ (بیوپاری)',
  'Pukhta Khata (Seller)',
  {
    is_system: true,
    is_pukhta_khata: true,
  },
);
const NAZIR = account(32, 'buyer', '0032', 'نذیر', 'Nazir', {
  is_pukhta_party: true,
  credit_allowed: false,
  phone1: '03001111111',
});
const KAMAL = account(33, 'seller', '0033', 'کمال', 'Kamal', { is_pukhta_party: true });

const named = (text: string) => (name: string) => visible(name) === text;
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));
const rowIds = () =>
  Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map((row) =>
    row.getAttribute('row-id'),
  );
const nameCell = (rowId: number) =>
  visible(
    document.querySelector(`.ag-row[row-id="${String(rowId)}"]`)?.querySelector('[col-id="name"]')
      ?.textContent,
  );

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeMasters();
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        {
          toUrdu: { nazir: ['نذیر'], kamal: ['کمال'] },
          toEnglish: { نذیر: 'Nazir', کمال: 'Kamal' },
        },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

/** Chooses a kind in the type list of the account form. */
async function chooseKind(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  kind: string,
) {
  await user.click(
    await screen.findByRole('combobox', { name: named(message(language, 'accounts-field-type')) }),
  );
  await user.click(
    await screen.findByRole('option', {
      name: named(message(language, `accounts-type-${kind}`)),
    }),
  );
}

/** Types a name in the language of the app: in English letters, so the Urdu name fills by itself. */
async function typeName(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  text: string,
) {
  if (language === 'ur') {
    await user.click(await labelled(language, 'accounts-field-name-ur'));
    await user.keyboard(`${text.toLowerCase()}{Enter}`);
  } else {
    await user.click(await labelled(language, 'accounts-field-name-en'));
    await user.keyboard(text);
  }
  await waitFor(() => {
    expect(
      screen.getByLabelText<HTMLInputElement>(named(message(language, 'accounts-field-name-en')))
        .value,
    ).toBe(text);
  });
}

async function openCreditPart(user: ReturnType<typeof userEvent.setup>, language: Language) {
  const heading = message(language, 'accounts-section-credit');
  await user.click(
    await screen.findByRole('button', { name: (name) => visible(name).startsWith(heading) }),
  );
}

describe.each(LANGUAGES)('the account form, a پختہ party (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('a پختہ گاہک gets 0032, is saved with the mark, and starts with credit off', async () => {
    fakeAccounts([AKRAM, RAHIM, SHARED_BUYER, SHARED_SELLER]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await chooseKind(user, language, 'pukhta_buyer');
    const code = await labelled(language, 'accounts-field-code');
    await waitFor(() => {
      expect(code.value).toBe('0032');
    });
    // The code is asked of the backend as the next one of the پختہ series, for the buyer side.
    expect(accountApi.accountsNextCode).toHaveBeenLastCalledWith('buyer', null, true);
    await typeName(user, language, 'Nazir');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      id: null,
      account_type: 'buyer',
      is_pukhta_party: true,
      code: '0032',
      names: { urdu: 'نذیر', english: 'Nazir' },
      credit_allowed: false,
    });
  });

  it('a پختہ بیوپاری after a پختہ گاہک gets 0033: one counter for both sides', async () => {
    fakeAccounts([AKRAM, RAHIM, SHARED_BUYER, SHARED_SELLER, NAZIR]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await chooseKind(user, language, 'pukhta_seller');
    const code = await labelled(language, 'accounts-field-code');
    await waitFor(() => {
      expect(code.value).toBe('0033');
    });
    expect(accountApi.accountsNextCode).toHaveBeenLastCalledWith('seller', null, true);
    await typeName(user, language, 'Kamal');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      account_type: 'seller',
      is_pukhta_party: true,
      code: '0033',
      names: { urdu: 'کمال', english: 'Kamal' },
    });
  });

  it('an ordinary buyer is saved without the mark, with credit on, as before', async () => {
    fakeAccounts([AKRAM, RAHIM, SHARED_BUYER, SHARED_SELLER]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    const code = await labelled(language, 'accounts-field-code');
    await waitFor(() => {
      expect(code.value).toBe('0021');
    });
    await typeName(user, language, 'Nazir');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    const draft = vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0];
    expect(draft).toMatchObject({ account_type: 'buyer', code: '0021', credit_allowed: true });
    expect(draft?.is_pukhta_party).not.toBe(true);
  });

  it('the credit box follows the kind: off for a پختہ گاہک, on again for a گاہک, and a choice already made stays', async () => {
    fakeAccounts([AKRAM, RAHIM, SHARED_BUYER, SHARED_SELLER]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    const credit = () =>
      screen.getByRole('checkbox', {
        name: named(message(language, 'accounts-field-credit-allowed')),
      });
    await openCreditPart(user, language);
    expect(credit()).toBeChecked();
    await chooseKind(user, language, 'pukhta_buyer');
    await waitFor(() => {
      expect(credit()).not.toBeChecked();
    });
    await chooseKind(user, language, 'buyer');
    await waitFor(() => {
      expect(credit()).toBeChecked();
    });
    // The Owner ticks credit on for a پختہ گاہک; changing nothing else keeps his choice.
    await chooseKind(user, language, 'pukhta_buyer');
    await waitFor(() => {
      expect(credit()).not.toBeChecked();
    });
    await user.click(credit());
    expect(credit()).toBeChecked();
    // A seller has no credit part, so it goes away and comes back shut; the tick is still his choice.
    await chooseKind(user, language, 'pukhta_seller');
    await chooseKind(user, language, 'pukhta_buyer');
    await openCreditPart(user, language);
    await waitFor(() => {
      expect(credit()).toBeChecked();
    });
  });

  it('a پختہ بیوپاری has no credit part: it is for buyers', async () => {
    fakeAccounts([AKRAM, RAHIM, SHARED_BUYER, SHARED_SELLER]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await chooseKind(user, language, 'pukhta_seller');
    await labelled(language, 'accounts-field-code');
    expect(
      screen.queryByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'accounts-section-credit')),
      }),
    ).not.toBeInTheDocument();
  });

  it('the edit form shows his kind, cannot change it, and saves with the mark kept', async () => {
    fakeAccounts([AKRAM, SHARED_BUYER, NAZIR]);
    await renderApp('/index/accounts/32');
    await screen.findByRole('heading', { level: 1 });
    const type = await screen.findByRole('combobox', {
      name: named(message(language, 'accounts-field-type')),
    });
    expect(type).toBeDisabled();
    expect(visible(type.textContent)).toBe(message(language, 'accounts-type-pukhta_buyer'));
    // The note about the type shows on the new form only.
    expect(
      screen.queryByText(message(language, 'accounts-field-type-hint').slice(0, 20)),
    ).not.toBeInTheDocument();
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      id: 32,
      account_type: 'buyer',
      is_pukhta_party: true,
      code: '0032',
      credit_allowed: false,
    });
  });

  it("the backend's refusal of a changed mark is shown in words, not as a code", async () => {
    fakeAccounts([AKRAM, SHARED_BUYER, NAZIR]);
    await renderApp('/index/accounts/32');
    await screen.findByRole('heading', { level: 1 });
    // The backend's own answer when a test of its sent a changed mark on an edit (plan 47, T1).
    vi.mocked(accountApi.accountsSave).mockRejectedValueOnce(new CommandError(real.SAVE_CHANGE));
    press('F10');
    expect(
      await screen.findByText((text) =>
        visible(text).includes(message(language, 'err-format-invalid')),
      ),
    ).toBeInTheDocument();
    expect(visible(document.body.textContent)).not.toContain('ERR_FORMAT_INVALID');
  });

  it('an ordinary buyer edits as before: no mark on the draft that is sent', async () => {
    fakeAccounts([AKRAM]);
    await renderApp('/index/accounts/21');
    await screen.findByRole('heading', { level: 1 });
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0].is_pukhta_party).not.toBe(true);
  });
});

describe.each(LANGUAGES)('the account list, پختہ parties (%s)', (language) => {
  const ROWS = [AKRAM, RAHIM, SHARED_BUYER, SHARED_SELLER, NAZIR, KAMAL];
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function pickFilter(kind: string) {
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'accounts-filter-type')),
      }),
    );
    await user.click(
      await screen.findByRole('option', {
        name: named(message(language, `accounts-type-${kind}`)),
      }),
    );
  }

  it('all: his name carries his kind, an ordinary party and the shared accounts do not', async () => {
    fakeAccounts(ROWS);
    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31', '9', '7', '32', '33']);
    });
    const nazir = language === 'ur' ? 'نذیر' : 'Nazir';
    const kamal = language === 'ur' ? 'کمال' : 'Kamal';
    expect(nameCell(32)).toBe(`${nazir} (${message(language, 'accounts-type-pukhta_buyer')})`);
    expect(nameCell(33)).toBe(`${kamal} (${message(language, 'accounts-type-pukhta_seller')})`);
    expect(nameCell(21)).toBe(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(nameCell(9)).toBe(language === 'ur' ? 'پختہ کھاتہ (خریدار)' : 'Pukhta Khata (Buyer)');
  });

  it('the پختہ filter lists the two shared accounts and his own, each with its side', async () => {
    fakeAccounts(ROWS);
    await renderApp('/index/accounts');
    await pickFilter('pukhta');
    await waitFor(() => {
      expect(rowIds()).toEqual(['9', '7', '32', '33']);
    });
    expect(accountApi.accountsList).toHaveBeenLastCalledWith(
      { kind: 'pukhta', text: '', include_inactive: false, has_opening: null },
      0,
      expect.any(Number),
    );
  });

  it('the Buyer and Seller filters leave the پختہ parties out', async () => {
    fakeAccounts(ROWS);
    await renderApp('/index/accounts');
    await pickFilter('buyer');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21']);
    });
    await pickFilter('seller');
    await waitFor(() => {
      expect(rowIds()).toEqual(['31']);
    });
  });

  it('the filter has no new entry: the list kinds stay All, Seller, Buyer, Pukhta', async () => {
    fakeAccounts(ROWS);
    await renderApp('/index/accounts');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'accounts-filter-type')),
      }),
    );
    const options = await screen.findAllByRole('option');
    expect(within(document.body).getAllByRole('option')).toHaveLength(options.length);
    expect(options.map((option) => visible(option.textContent))).toEqual([
      message(language, 'accounts-type-all'),
      message(language, 'accounts-type-seller'),
      message(language, 'accounts-type-buyer'),
      message(language, 'accounts-type-pukhta'),
    ]);
  });
});
