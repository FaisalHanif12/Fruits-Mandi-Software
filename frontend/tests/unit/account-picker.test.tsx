// The account picker, the party card and the quick add (plan 08, PRD FR-7.38 to FR-7.42, FR-10.8,
// FR-7.64): every row of the plan's Section 3, in the Urdu and in the English app, by keyboard,
// against a fake of the search, short-code and party-card commands.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import {
  AccountPicker,
  AddRecordContext,
  Field,
  PartyCard,
  type AddRecordRequest,
  type PickerValue,
} from '../../src/shared/ui';
import type { AccountScope } from '../../src/shared/accounts';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, cardOf, fakeAccountSearch, money, ref } from '../helpers/accounts';
import { fail } from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  city: ref(1, '0001', 'کوٹ رادھا کشن', 'Kot Radha Kishan'),
  father_or_firm_ur: 'ٹول ٹیکس',
  father_or_firm_en: 'Toll Tax',
  phone1: '03001234567',
  balance: balance(2880, 'dr'),
  credit_limit_paisa: money(50000),
});
const ARSHAD = account(22, 'buyer', '0021', 'ارشد ملتانی', 'Arshad Multani', {
  balance: balance(18684, 'dr'),
});
const RAHIM = account(31, 'seller', '0010', 'رحیم', 'Rahim', {
  balance: balance(4316, 'cr'),
});
const KARIM = account(32, 'seller', '0011', 'کریم', 'Karim');
const ROWS = [AKRAM, ARSHAD, RAHIM, KARIM];

const named = (text: string) => (name: string) => visible(name) === text;

function Harness({
  scope,
  initial = null,
  onValue,
}: {
  scope: AccountScope;
  initial?: PickerValue | null;
  onValue?: (value: PickerValue | null) => void;
}) {
  const [value, setValue] = useState<PickerValue | null>(initial);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <input data-testid="before" />
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <AccountPicker
            {...control}
            scope={scope}
            value={value}
            onChange={(hit) => {
              const next =
                hit === null
                  ? null
                  : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en };
              setValue(next);
              onValue?.(next);
            }}
          />
        )}
      </Field>
      <input data-testid="after" />
    </form>
  );
}

function setup(
  language: Language,
  scope: AccountScope,
  props: { initial?: PickerValue | null; onValue?: (value: PickerValue | null) => void } = {},
) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness scope={scope} {...props} />
    </QueryClientProvider>,
  );
  const input = screen.getByRole<HTMLInputElement>('combobox');
  return { input, user: userEvent.setup() };
}

const options = () => screen.queryAllByRole('option');
const nameIn = (language: Language, row: { name_ur: string; name_en: string }) =>
  language === 'ur' ? row.name_ur : row.name_en;

beforeEach(() => {
  vi.clearAllMocks();
  fakeAccountSearch(ROWS);
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        {
          toUrdu: { tariq: ['طارق'], mehmood: ['محمود'] },
          toEnglish: { طارق: 'Tariq', محمود: 'Mehmood' },
        },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
});

describe.each(LANGUAGES)('AccountPicker, short codes (%s)', (language) => {
  it('buyer: `0` then Enter takes 0020, and his card shows at once', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('0');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 21, code: '0020' }));
    expect(input.value).toBe(nameIn(language, AKRAM));
    expect(screen.getByTestId('after')).toHaveFocus();
    // The card: name, city, phone, balance with the side as a word, last entry, credit.
    const card = await screen.findByText(message(language, 'assist-card-balance'));
    const text = visible(card.closest('section')?.textContent);
    expect(text).toContain(nameIn(language, AKRAM));
    expect(text).toContain('0020');
    expect(text).toContain(language === 'ur' ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan');
    expect(text).toContain('03001234567');
    expect(text).toContain('2,880');
    expect(text).toContain(message(language, 'common-side-dr'));
    expect(text).toContain('10-10-2026');
    expect(text).toContain(message(language, 'assist-card-credit-limit'));
    expect(text).toContain('50,000');
  });

  it('Alt+Down opens the recent accounts of this field, and the pick is remembered in the same list', async () => {
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(0);
    });
    // Empty text is every account of the field: the last chosen first, then by code (plan 32, R7).
    expect(accountApi.searchAccounts).toHaveBeenCalledWith({
      scope: 'buyer',
      controlId: null,
      text: '',
      offset: 0,
      limit: 50,
    });
    await user.keyboard('{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(accountApi.searchAccountsTouch).toHaveBeenCalledTimes(1);
    });
    const [scope, controlId, id] = vi.mocked(accountApi.searchAccountsTouch).mock.calls[0] ?? [];
    expect([scope, controlId]).toEqual(['buyer', null]);
    expect(typeof id).toBe('number');
  });

  it('the full code `0020` is the same buyer', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('0020');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 21 }));
  });

  it('seller: `0` is 0010, and the card shows his open advance, not a credit limit', async () => {
    const { input, user } = setup(language, 'seller');
    await user.click(input);
    await user.keyboard('0{Enter}');
    const card = await screen.findByText(message(language, 'assist-card-balance'));
    const text = visible(card.closest('section')?.textContent);
    expect(text).toContain('0010');
    expect(text).toContain(message(language, 'assist-card-advance'));
    expect(text).not.toContain(message(language, 'assist-card-credit-limit'));
    expect(text).toContain(message(language, 'common-side-cr'));
  });

  it('seller: `7` does not exist, so Enter says so and offers to add the next seller', async () => {
    const { input, user } = setup(language, 'seller');
    await user.click(input);
    await user.keyboard('7');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.keyboard('{Enter}');
    const note = await screen.findByText(named(message(language, 'accountpicker-unknown-seller')));
    expect(note).toBeInTheDocument();
    expect(accountApi.accountsResolveCode).toHaveBeenCalledWith('seller', null, '7');
    expect(
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'accountpicker-add')),
      }),
    ).toBeInTheDocument();
  });

  it('any account: `0` lists the buyer and the seller with their types, and the user picks one', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'any', { onValue });
    await user.click(input);
    await user.keyboard('0');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    const rows = options().map((option) => visible(option.textContent));
    expect(rows[0]).toContain(message(language, 'accounts-type-buyer'));
    expect(rows[0]).toContain('0020');
    expect(rows[1]).toContain(message(language, 'accounts-type-seller'));
    expect(rows[1]).toContain('0010');
    await user.keyboard('{ArrowDown}{Enter}');
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 31 }));
  });

  it('the card previews the top match while typing, and nothing is taken until Enter', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('1');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'assist-card-balance'))).toBeInTheDocument();
    });
    expect(onValue).not.toHaveBeenCalled();
    expect(accountApi.accountsResolveCode).not.toHaveBeenCalled();
    // Still typing: `12` is not a buyer. Nothing is taken and the backend is not asked yet.
    await user.keyboard('2');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    expect(onValue).not.toHaveBeenCalled();
    expect(accountApi.accountsResolveCode).not.toHaveBeenCalled();
    await user.keyboard('{Enter}');
    expect(accountApi.accountsResolveCode).toHaveBeenCalledWith('buyer', null, '12');
  });

  it('Enter straight after a key never takes a row from the older text (1, then 2 and Enter at once)', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('1');
    // The list answers `1` (one buyer). The next key and Enter come before the answer to `12`.
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.keyboard('2{Enter}');
    expect(input.value).toBe('12');
    expect(onValue).not.toHaveBeenCalled();
    // The whole code `12` goes to the backend, which says there is no such buyer.
    await waitFor(() => {
      expect(accountApi.accountsResolveCode).toHaveBeenCalledWith('buyer', null, '12');
    });
    expect(onValue).not.toHaveBeenCalled();
  });

  it('Enter straight after a name key waits for the answer to the whole text, then takes its best match', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('ak');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    // `aks` matches nothing: the row of `ak` (Akram) must not be taken.
    await user.keyboard('s{Enter}');
    await waitFor(() => {
      expect(accountApi.searchAccounts).toHaveBeenCalledWith(
        expect.objectContaining({ text: 'aks' }),
      );
    });
    await waitFor(() => {
      expect(input).toHaveAttribute('aria-expanded', 'false');
    });
    expect(onValue).not.toHaveBeenCalled();
    expect(input.value).toBe('aks');
  });

  it('Enter then leaving the field resolves the code once, and shows no message under a filled field', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('1');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    expect(onValue).toHaveBeenCalledTimes(1);
    expect(accountApi.accountsResolveCode).not.toHaveBeenCalled();
    expect(screen.queryByText(message(language, 'accountpicker-several'))).not.toBeInTheDocument();
  });

  it('a پختہ کھاتہ account has no short code, but its full code works; the backend never says "reserved"', async () => {
    const pukhta = account(23, 'buyer', '0030', 'پختہ کھاتہ (خریدار)', 'Pukhta Khata (Buyer)', {
      is_system: true,
      is_pukhta_khata: true,
    });
    fakeAccountSearch([...ROWS, pukhta]);
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    // `0` is the running number of the first buyer, not the پختہ کھاتہ account.
    await user.keyboard('0');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    expect(visible(options()[0]?.textContent)).toContain('0020');
    await user.clear(input);
    await user.keyboard('0030');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 23, code: '0030' }));
  });

  it('a number no buyer has says so and offers to add one; there is no "reserved" answer', async () => {
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('9');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.keyboard('{Enter}');
    expect(
      await screen.findByText(named(message(language, 'accountpicker-unknown-buyer'))),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'accountpicker-add')),
      }),
    ).toBeInTheDocument();
  });

  it('a code of an inactive account says so, names it, and offers no add and takes nothing', async () => {
    const hidden = account(41, 'buyer', '0028', 'پرانا', 'Old One', { is_active: false });
    fakeAccountSearch([...ROWS, hidden]);
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('8');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.keyboard('{Enter}');
    expect(
      await screen.findByText(
        named(
          message(language, 'accountpicker-inactive', {
            name: language === 'ur' ? 'پرانا' : 'Old One',
            code: '0028',
          }),
        ),
      ),
    ).toBeInTheDocument();
    expect(onValue).not.toHaveBeenCalled();
    expect(
      screen.queryByRole('button', { name: new RegExp(message(language, 'accountpicker-add')) }),
    ).not.toBeInTheDocument();
  });

  it('leaving the field with a code typed takes the account, without Enter', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('1');
    await user.tab();
    await waitFor(() => {
      expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 22 }));
    });
    expect(input.value).toBe(nameIn(language, ARSHAD));
  });

  it('leaving the field with a code that fits two accounts offers the two, with their types', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'any', { onValue });
    await user.click(input);
    await user.keyboard('0');
    await user.tab();
    expect(
      await screen.findByText(named(message(language, 'accountpicker-several'))),
    ).toBeInTheDocument();
    expect(onValue).not.toHaveBeenCalled();
    await user.click(
      screen.getByRole('button', { name: (name) => visible(name).includes('0010') }),
    );
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 31 }));
  });

  it('a typed name is searched as typed and never turned into Urdu', async () => {
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('akram');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    expect(accountApi.searchAccounts).toHaveBeenLastCalledWith({
      scope: 'buyer',
      controlId: null,
      text: 'akram',
      offset: 0,
      limit: 50,
    });
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
  });

  it('the rows show code, name, city and the balance with its side', async () => {
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('akr');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const row = visible(options()[0]?.textContent);
    expect(row).toContain('0020');
    expect(row).toContain(nameIn(language, AKRAM));
    expect(row).toContain(language === 'ur' ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan');
    expect(row).toContain('2,880');
    expect(row).toContain(message(language, 'common-side-dr'));
  });

  it('a row whose balance could not be read shows no balance, never a wrong one', async () => {
    vi.mocked(accountApi.searchAccounts).mockResolvedValue([
      {
        id: AKRAM.id,
        code: AKRAM.code,
        account_type: 'buyer',
        control_id: 4,
        name_ur: AKRAM.name_ur,
        name_en: AKRAM.name_en,
        extra_ur: '',
        extra_en: '',
        is_active: true,
        is_dead: false,
        balance: null,
      },
    ]);
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('akr');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const row = visible(options()[0]?.textContent);
    expect(row).toContain('0020');
    expect(row).not.toContain(message(language, 'common-side-dr'));
    expect(row).not.toContain('2,880');
  });
});

describe.each(LANGUAGES)('AccountPicker, adding an account on the spot (%s)', (language) => {
  it('an unknown full code: leave the field, F4, fill the name, F10 saves and the account is in the field', async () => {
    const onValue = vi.fn();
    vi.mocked(accountApi.accountsNextCode).mockResolvedValue('0022');
    vi.mocked(accountApi.accountsSave).mockImplementation((draft) =>
      Promise.resolve(
        account(99, draft.account_type, draft.code, draft.names.urdu, draft.names.english),
      ),
    );
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('0025554');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.tab();
    expect(
      await screen.findByText(named(message(language, 'accountpicker-unknown-buyer'))),
    ).toBeInTheDocument();
    // F4 works from the message, after the field has been left.
    act(() => {
      document.body.dispatchEvent(
        new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }),
      );
    });
    const dialog = await screen.findByRole('dialog');
    expect(
      within(dialog).getByText(message(language, 'accountpicker-add-title')),
    ).toBeInTheDocument();
    const code = within(dialog).getByLabelText<HTMLInputElement>(
      named(message(language, 'accounts-field-code')),
    );
    // The form offers the code the backend suggested, not the digits typed (plan 32, R2).
    expect(code.value).toBe('0022');
    if (language === 'ur') {
      await user.click(
        within(dialog).getByLabelText(named(message(language, 'accounts-field-name-ur'))),
      );
      await user.keyboard('tariq mehmood{Enter}');
    } else {
      await user.click(
        within(dialog).getByLabelText(named(message(language, 'accounts-field-name-en'))),
      );
      await user.keyboard('Tariq Mehmood');
    }
    await waitFor(() => {
      expect(
        within(dialog).getByLabelText<HTMLInputElement>(
          named(message(language, 'accounts-field-name-en')),
        ).value,
      ).toBe('Tariq Mehmood');
    });
    act(() => {
      document.activeElement?.dispatchEvent(
        new KeyboardEvent('keydown', { code: 'F10', key: 'F10', bubbles: true }),
      );
    });
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      id: null,
      account_type: 'buyer',
      code: '0022',
      names: { urdu: 'طارق محمود', english: 'Tariq Mehmood' },
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 99, code: '0022' }));
    expect(input.value).toBe(language === 'ur' ? 'طارق محمود' : 'Tariq Mehmood');
    expect(
      screen.queryByText(named(message(language, 'accountpicker-unknown-buyer'))),
    ).not.toBeInTheDocument();
  });

  it('F4 in the field adds an account with the typed text as its name, and Esc closes without saving', async () => {
    vi.mocked(accountApi.accountsNextCode).mockResolvedValue('0022');
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('Tariq');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    act(() => {
      input.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
    });
    const dialog = await screen.findByRole('dialog');
    // The code is the next free one, suggested and visible; the typed letters start the name.
    await waitFor(() => {
      expect(
        within(dialog).getByLabelText<HTMLInputElement>(
          named(message(language, 'accounts-field-code')),
        ).value,
      ).toBe('0022');
    });
    expect(
      within(dialog).getByLabelText<HTMLInputElement>(
        named(message(language, 'accounts-field-name-en')),
      ).value,
    ).toBe('Tariq');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(accountApi.accountsSave).not.toHaveBeenCalled();
    // Focus goes back to the field that opened it.
    await waitFor(() => {
      expect(input).toHaveFocus();
    });
  });

  it('one Esc closes the suggestion list inside the dialog, the next Esc closes the dialog (FR-7.18)', async () => {
    vi.mocked(accountApi.accountsNextCode).mockResolvedValue('0022');
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    act(() => {
      input.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
    });
    const dialog = await screen.findByRole('dialog');
    const urduName = within(dialog).getByLabelText<HTMLInputElement>(
      named(message(language, 'accounts-field-name-ur')),
    );
    await user.click(urduName);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(within(dialog).getAllByRole('option').length).toBeGreaterThan(0);
    });
    await user.keyboard('{Escape}');
    // The list is gone, the dialog and the English letters are still there.
    await waitFor(() => {
      expect(within(dialog).queryAllByRole('option')).toHaveLength(0);
    });
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(urduName.value).toBe('tariq');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(accountApi.accountsSave).not.toHaveBeenCalled();
  });

  it('in a field that takes any account the quick add asks which type', async () => {
    const { input, user } = setup(language, 'any');
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    act(() => {
      input.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
    });
    const dialog = await screen.findByRole('dialog');
    expect(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'accounts-field-type')),
      }),
    ).toBeInTheDocument();
  });

  it('a refusal at the code sits under the code field and the dialog stays open', async () => {
    vi.mocked(accountApi.accountsNextCode).mockResolvedValue('0022');
    vi.mocked(accountApi.accountsSave).mockRejectedValue(
      fail({ code: 'ERR_CODE_TAKEN', field: 'code' }),
    );
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('Tariq');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    act(() => {
      input.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
    });
    const dialog = await screen.findByRole('dialog');
    await user.click(
      within(dialog).getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'layout-save-save')),
      }),
    );
    await waitFor(() => {
      expect(
        within(dialog).getByLabelText(named(message(language, 'accounts-field-code'))),
      ).toHaveAttribute('aria-invalid', 'true');
    });
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
});

// Plan 41, PRD FR-7.69, FR-7.42, FR-7.63, E74, E75: the add row ends the list of an account field, a name
// that matches nothing is added with Enter, a look-alike can be taken instead, and the quick add has
// a link to the full form, which the app opens.
describe.each(LANGUAGES)('AccountPicker, the add row and the quick add (%s)', (language) => {
  const typedBuyer = language === 'ur' ? 'نیا گاہک' : 'Nadeem Khan';
  const nameBox = language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en';
  const addRowName = (text?: string) => (name: string) =>
    visible(name).startsWith(
      text === undefined
        ? message(language, 'search-add-row', { noun: 'buyer' })
        : message(language, 'search-add-row-named', { noun: 'buyer', text }),
    );

  /** The picker with the app's host faked, so the full form can be asked for. */
  function setupWithHost(host: (request: AddRecordRequest) => void, onValue?: () => void) {
    act(() => {
      useLanguageStore.setState({ language });
    });
    renderIn(
      language,
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <AddRecordContext.Provider value={host}>
          <Harness scope="buyer" {...(onValue === undefined ? {} : { onValue })} />
        </AddRecordContext.Provider>
      </QueryClientProvider>,
    );
    return { input: screen.getByRole<HTMLInputElement>('combobox'), user: userEvent.setup() };
  }

  beforeEach(() => {
    vi.mocked(accountApi.accountsNextCode).mockResolvedValue('0022');
    vi.mocked(accountApi.accountsSave).mockImplementation((draft) =>
      Promise.resolve(
        account(99, draft.account_type, draft.code, draft.names.urdu, draft.names.english),
      ),
    );
  });

  it('the quick add fills the other name by itself from the name typed in the picker (FR-7.24)', async () => {
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard(language === 'ur' ? 'طارق' : 'tariq');
    await screen.findByRole('button', {
      name: addRowName(language === 'ur' ? 'طارق' : 'tariq'),
    });
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog');
    const box = (id: string) =>
      within(dialog).getByLabelText<HTMLInputElement>(named(message(language, id)));
    // Typed in Urdu letters: the English name is suggested. Typed in English letters (and the app
    // in English): the Urdu name is.
    await waitFor(() => {
      if (language === 'ur') expect(box('accounts-field-name-en').value).toBe('Tariq');
      else expect(box('accounts-field-name-ur').value).toBe('طارق');
    });
  });

  it('the open list ends with the row, and an empty list says so in words', async () => {
    fakeAccountSearch([]);
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    expect(
      await screen.findByText(
        (_content, element) =>
          element?.tagName === 'P' &&
          visible(element.textContent) ===
            message(language, 'search-empty-kind', { noun: 'buyer' }),
      ),
    ).toBeInTheDocument();
    expect(await screen.findByRole('button', { name: addRowName() })).toBeInTheDocument();
  });

  it('Enter on a name that matches nothing opens the quick add with it; F10 saves and the field takes the account', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard(typedBuyer);
    await screen.findByRole('button', { name: addRowName(typedBuyer) });
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog');
    expect(
      within(dialog).getByLabelText<HTMLInputElement>(named(message(language, nameBox))).value,
    ).toBe(typedBuyer);
    // The short form asks the tag, the phone and the city; father or firm is the full form's.
    for (const id of ['accounts-field-tag', 'accounts-field-phone1', 'accounts-field-city']) {
      expect(
        within(dialog).getAllByText((content) => visible(content) === message(language, id)).length,
      ).toBeGreaterThan(0);
    }
    expect(
      within(dialog).queryByText(
        (content) => visible(content) === message(language, 'accounts-field-father-ur'),
      ),
    ).toBeNull();
    act(() => {
      document.activeElement?.dispatchEvent(
        new KeyboardEvent('keydown', { code: 'F10', key: 'F10', bubbles: true }),
      );
    });
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 99 }));
    await waitFor(() => {
      expect(input.value).toBe(typedBuyer);
    });
    expect(input).toHaveFocus();
  });

  it('Esc in the quick add leaves the field as it was', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard(typedBuyer);
    await screen.findByRole('button', { name: addRowName(typedBuyer) });
    await user.click(screen.getByRole('button', { name: addRowName(typedBuyer) }));
    await screen.findByRole('dialog');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(accountApi.accountsSave).not.toHaveBeenCalled();
    expect(onValue).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(input).toHaveFocus();
    });
    expect(input.value).toBe(typedBuyer);
  });

  it('a buyer like the name typed shows under it, and he can be taken instead of a new one', async () => {
    vi.mocked(accountApi.accountsSimilar).mockResolvedValue([
      {
        id: AKRAM.id,
        code: AKRAM.code,
        name_ur: AKRAM.name_ur,
        name_en: AKRAM.name_en,
        father_or_firm_ur: AKRAM.father_or_firm_ur,
        father_or_firm_en: AKRAM.father_or_firm_en,
        city_ur: 'کوٹ رادھا کشن',
        city_en: 'Kot Radha Kishan',
        phone1: AKRAM.phone1,
        account_type: 'buyer',
        is_duplicate: false,
      },
      {
        id: RAHIM.id,
        code: RAHIM.code,
        name_ur: RAHIM.name_ur,
        name_en: RAHIM.name_en,
        father_or_firm_ur: '',
        father_or_firm_en: '',
        city_ur: '',
        city_en: '',
        phone1: '',
        account_type: 'seller',
        is_duplicate: false,
      },
    ]);
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard(typedBuyer);
    await screen.findByRole('button', { name: addRowName(typedBuyer) });
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog');
    const panel = (
      await within(dialog).findByRole('heading', {
        name: named(message(language, 'accounts-similar-title')),
      })
    ).closest('section');
    if (panel === null) throw new Error('no panel');
    // Both look alike, but only the buyer can be taken in a buyer field.
    const pick = within(panel).getAllByRole('button', {
      name: named(message(language, 'search-quick-pick')),
    });
    expect(pick).toHaveLength(1);
    const first = pick[0];
    if (first === undefined) throw new Error('no pick button');
    await user.click(first);
    await waitFor(() => {
      expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: AKRAM.id }));
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    // Nothing was saved: the existing account is the one chosen.
    expect(accountApi.accountsSave).not.toHaveBeenCalled();
    expect(input.value).toBe(nameIn(language, AKRAM));
  });

  it('the link "Full form" closes the quick add and asks the app for the full form with the name', async () => {
    const host = vi.fn();
    const onValue = vi.fn();
    const { input, user } = setupWithHost(host, onValue);
    await user.click(input);
    await user.keyboard(typedBuyer);
    await screen.findByRole('button', { name: addRowName(typedBuyer) });
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog');
    await user.click(
      within(dialog).getByRole('button', {
        name: named(message(language, 'search-quick-full-form')),
      }),
    );
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(host).toHaveBeenCalledTimes(1);
    const request = host.mock.calls[0]?.[0] as AddRecordRequest;
    expect(request).toMatchObject({ kind: 'buyer', text: typedBuyer, form: 'full' });
    // The full form saved the account: the picker finds it by its code and the field takes it.
    act(() => {
      request.onClosed({ id: AKRAM.id, code: AKRAM.code });
    });
    await waitFor(() => {
      expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: AKRAM.id }));
    });
    expect(input.value).toBe(nameIn(language, AKRAM));
  });
});

describe.each(LANGUAGES)('PartyCard (%s)', (language) => {
  function Card({ id }: { id: number | null }) {
    return (
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <PartyCard accountId={id} />
      </QueryClientProvider>
    );
  }

  it('shows nothing without an account', () => {
    renderIn(language, <Card id={null} />);
    expect(screen.queryByText(message(language, 'assist-card-balance'))).not.toBeInTheDocument();
    expect(accountApi.assistPartyCard).not.toHaveBeenCalled();
  });

  it('says it is opening while the card loads', () => {
    vi.mocked(accountApi.assistPartyCard).mockReturnValue(new Promise(() => undefined));
    renderIn(language, <Card id={21} />);
    expect(screen.getByText(message(language, 'assist-card-loading'))).toBeInTheDocument();
  });

  it('a failure says so, with the plain message', async () => {
    vi.mocked(accountApi.assistPartyCard).mockRejectedValue(fail({ code: 'ERR_DB_BUSY' }));
    renderIn(language, <Card id={21} />);
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'assist-card-failed'));
    expect(visible(alert.textContent)).toContain(message(language, 'err-db-busy'));
  });

  it('an account with no entry yet says so', async () => {
    vi.mocked(accountApi.assistPartyCard).mockResolvedValue(
      cardOf(KARIM, { last_entry_date: null }),
    );
    renderIn(language, <Card id={32} />);
    expect(await screen.findByText(message(language, 'assist-card-never'))).toBeInTheDocument();
  });

  it('warnings come with a word and an icon, not by colour alone', async () => {
    vi.mocked(accountApi.assistPartyCard).mockResolvedValue(
      cardOf(AKRAM, {
        warnings: [
          { kind: 'inactive', amount_paisa: null },
          { kind: 'over_credit_limit', amount_paisa: null },
          { kind: 'owes_firm', amount_paisa: null },
        ],
      }),
    );
    renderIn(language, <Card id={21} />);
    const items = await screen.findAllByRole('listitem');
    expect(items.map((item) => visible(item.textContent))).toEqual([
      message(language, 'assist-warning-inactive'),
      message(language, 'assist-warning-over-credit-limit'),
      message(language, 'assist-warning-owes-firm'),
    ]);
    for (const item of items) expect(item.querySelector('svg')).not.toBeNull();
  });

  it('a card is read again when the account changes, never reused', async () => {
    function Swap() {
      const [id, setId] = useState(21);
      return (
        <>
          <Card id={id} />
          <button
            type="button"
            data-testid="swap"
            onClick={() => {
              setId(22);
            }}
          />
        </>
      );
    }
    renderIn(language, <Swap />);
    await screen.findByText(message(language, 'assist-card-balance'));
    await userEvent.setup().click(screen.getByTestId('swap'));
    await waitFor(() => {
      expect(accountApi.assistPartyCard).toHaveBeenCalledWith(22);
    });
  });
});
