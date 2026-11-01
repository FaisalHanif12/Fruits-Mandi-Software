// A پختہ party's own account in the pickers, the short codes, the party card and the quick add (plan
// 47, PRD FR-13.11, FR-7.38, FR-7.39, FR-7.40, FR-7.69): the picker row of his side marks him with his
// kind; his running number in the پختہ series (0032 is 2) finds him in a buyer box only if he is a
// buyer, and in a seller box only if he is a seller; when an ordinary buyer has the same number the
// answer is "several" and both are listed with their kind; the card names his kind; the quick add
// offers the field's side and the same side as پختہ, and makes the account with the mark and with
// credit off. Urdu and English, by keyboard, against a fake of the search and account commands.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import type { AccountScope } from '../../src/shared/accounts';
import * as accountApi from '../../src/shared/accounts/api';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import {
  AccountPicker,
  AddRecordContext,
  Field,
  type AddRecordRequest,
  type PickerValue,
} from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const TARIQ = account(23, 'buyer', '0022', 'طارق محمود', 'Tariq Mehmood');
const RAHIM = account(31, 'seller', '0010', 'رحیم', 'Rahim');
const NAZIR = account(42, 'buyer', '0032', 'نذیر', 'Nazir', {
  is_pukhta_party: true,
  credit_allowed: false,
  balance: balance(500, 'dr'),
});
const KAMAL = account(43, 'seller', '0033', 'کمال', 'Kamal', { is_pukhta_party: true });
const SHARED_BUYER = account(9, 'buyer', '0030', 'پختہ کھاتہ (خریدار)', 'Pukhta Khata (Buyer)', {
  is_system: true,
  is_pukhta_khata: true,
});
const ROWS = [AKRAM, TARIQ, RAHIM, NAZIR, KAMAL, SHARED_BUYER];

const named = (text: string) => (name: string) => visible(name) === text;
const options = () => screen.queryAllByRole('option');
const kindLabel = (language: Language, kind: string) => message(language, `accounts-type-${kind}`);

function Harness({
  scope,
  onValue,
}: {
  scope: AccountScope;
  onValue?: (value: PickerValue | null) => void;
}) {
  const [value, setValue] = useState<PickerValue | null>(null);
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
    </form>
  );
}

function setup(
  language: Language,
  scope: AccountScope,
  props: {
    onValue?: (value: PickerValue | null) => void;
    onAdd?: (r: AddRecordRequest) => void;
  } = {},
) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <AddRecordContext.Provider value={props.onAdd ?? null}>
        <Harness
          scope={scope}
          {...(props.onValue === undefined ? {} : { onValue: props.onValue })}
        />
      </AddRecordContext.Provider>
    </QueryClientProvider>,
  );
  return { input: screen.getByRole<HTMLInputElement>('combobox'), user: userEvent.setup() };
}

beforeEach(() => {
  vi.clearAllMocks();
  fakeAccounts(ROWS);
  fakeAccountSearch(ROWS);
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        {
          toUrdu: { tariq: ['طارق'], zafar: ['ظفر'], nazir: ['نذیر'] },
          toEnglish: { طارق: 'Tariq', ظفر: 'Zafar', نذیر: 'Nazir' },
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

describe.each(LANGUAGES)('picker rows of a پختہ party (%s)', (language) => {
  it('in a buyer field his row names his kind; an ordinary buyer row does not', async () => {
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('nazir');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const row = visible(options()[0]?.textContent);
    expect(row).toContain('0032');
    // The mark next to his code is the short word پختہ کھاتہ; his side is the field's own.
    expect(row).toContain(kindLabel(language, 'pukhta'));
  });

  it('in a buyer field an ordinary buyer row has no kind mark', async () => {
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('akram');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const ordinary = visible(options()[0]?.textContent);
    expect(ordinary).toContain('0020');
    expect(ordinary).not.toContain(kindLabel(language, 'pukhta'));
  });

  it('a buyer field does not list a پختہ بیوپاری, and a seller field does not list a پختہ گاہک', async () => {
    const { input, user } = setup(language, 'buyer');
    await user.click(input);
    await user.keyboard('kamal');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
  });

  it('a field that takes any account lists each with his kind', async () => {
    const { input, user } = setup(language, 'any');
    await user.click(input);
    await user.keyboard('a');
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(3);
    });
    const rows = options().map((option) => visible(option.textContent));
    const nazir = rows.find((row) => row.includes('0032')) ?? '';
    const kamal = rows.find((row) => row.includes('0033')) ?? '';
    const akram = rows.find((row) => row.includes('0020')) ?? '';
    // Where the field takes any account, a row says his side too, besides the mark.
    expect(nazir).toContain(kindLabel(language, 'pukhta'));
    expect(nazir).toContain(kindLabel(language, 'buyer'));
    expect(kamal).toContain(kindLabel(language, 'pukhta'));
    expect(kamal).toContain(kindLabel(language, 'seller'));
    expect(akram).not.toContain(kindLabel(language, 'pukhta'));
    expect(akram).toContain(kindLabel(language, 'buyer'));
  });
});

describe.each(LANGUAGES)('short codes of a پختہ party, by side (%s)', (language) => {
  it('buyer box, `2`: an ordinary buyer and a پختہ گاہک share it, so both are listed with their kind', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('2');
    await user.tab();
    expect(
      await screen.findByText(named(message(language, 'accountpicker-several'))),
    ).toBeInTheDocument();
    const buttons = screen
      .getAllByRole('button')
      .filter((button) => /0022|0032/.test(visible(button.textContent)));
    expect(buttons.map((button) => visible(button.textContent))).toEqual([
      expect.stringContaining(kindLabel(language, 'buyer')),
      expect.stringContaining(kindLabel(language, 'pukhta_buyer')),
    ]);
    expect(visible(buttons[0]?.textContent)).toContain('0022');
    expect(visible(buttons[1]?.textContent)).toContain('0032');
    // Picking the second takes the پختہ گاہک and shows his card with his kind.
    const second = buttons[1];
    if (second === undefined) throw new Error('the second candidate is missing');
    await user.click(second);
    expect(onValue).toHaveBeenLastCalledWith(expect.objectContaining({ id: 42, code: '0032' }));
    await waitFor(() => {
      expect(
        screen.queryByText(named(message(language, 'accountpicker-several'))),
      ).not.toBeInTheDocument();
    });
  });

  it('buyer box, `2`, when only the پختہ گاہک has that number: he is found and his card names his kind', async () => {
    const without = ROWS.filter((row) => row.id !== TARIQ.id);
    fakeAccountSearch(without);
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('2{Enter}');
    await waitFor(() => {
      expect(onValue).toHaveBeenLastCalledWith(expect.objectContaining({ id: 42, code: '0032' }));
    });
    // His card appears under the field and names his kind.
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(kindLabel(language, 'pukhta_buyer'));
    });
  });

  it('seller box, `2`: no seller has that number, so it is unknown (the پختہ گاہک is not a seller)', async () => {
    const { input, user } = setup(language, 'seller');
    await user.click(input);
    await user.keyboard('2');
    await user.tab();
    expect(
      await screen.findByText(named(message(language, 'accountpicker-unknown-seller'))),
    ).toBeInTheDocument();
  });

  it('seller box, `3`: the پختہ بیوپاری is found by his running number', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'seller', { onValue });
    await user.click(input);
    await user.keyboard('3{Enter}');
    await waitFor(() => {
      expect(onValue).toHaveBeenLastCalledWith(expect.objectContaining({ id: 43, code: '0033' }));
    });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(kindLabel(language, 'pukhta_seller'));
    });
  });

  it('buyer box, the full code 0032 finds him whatever the other buyers are', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, 'buyer', { onValue });
    await user.click(input);
    await user.keyboard('0032{Enter}');
    await waitFor(() => {
      expect(onValue).toHaveBeenLastCalledWith(expect.objectContaining({ id: 42, code: '0032' }));
    });
  });
});

describe.each(LANGUAGES)('quick add of a پختہ party (%s)', (language) => {
  async function openQuickAdd(scope: AccountScope, typed: string, props = {}) {
    const { input, user } = setup(language, scope, props);
    await user.click(input);
    await user.keyboard(typed);
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    act(() => {
      input.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
    });
    const dialog = await screen.findByRole('dialog');
    return { input, user, dialog };
  }

  async function kindsOffered(user: ReturnType<typeof userEvent.setup>, dialog: HTMLElement) {
    await user.click(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'accounts-field-type')),
      }),
    );
    const shown = (await screen.findAllByRole('option')).map((option) =>
      visible(option.textContent),
    );
    await user.keyboard('{Escape}');
    return shown;
  }

  it('a buyer field offers گاہک and پختہ کھاتہ گاہک; a seller field the same for sellers', async () => {
    const buyer = await openQuickAdd('buyer', 'zzz');
    expect(await kindsOffered(buyer.user, buyer.dialog)).toEqual([
      kindLabel(language, 'buyer'),
      kindLabel(language, 'pukhta_buyer'),
    ]);
  });

  it('a seller field offers بیوپاری and پختہ کھاتہ بیوپاری', async () => {
    const seller = await openQuickAdd('seller', 'zzz');
    expect(await kindsOffered(seller.user, seller.dialog)).toEqual([
      kindLabel(language, 'seller'),
      kindLabel(language, 'pukhta_seller'),
    ]);
  });

  it('a field that takes any account offers all four', async () => {
    const any = await openQuickAdd('any', 'zzz');
    expect(await kindsOffered(any.user, any.dialog)).toEqual([
      kindLabel(language, 'seller'),
      kindLabel(language, 'buyer'),
      kindLabel(language, 'pukhta_buyer'),
      kindLabel(language, 'pukhta_seller'),
    ]);
  });

  it('a پختہ گاہک made on the spot gets the next پختہ code, is saved with the mark and credit off, and is put in the field', async () => {
    const onValue = vi.fn();
    const { input, user, dialog } = await openQuickAdd('buyer', 'zafar', { onValue });
    await user.click(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'accounts-field-type')),
      }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(kindLabel(language, 'pukhta_buyer')) }),
    );
    // NAZIR has 0032, so the next of the پختہ series is 0034 (KAMAL has 0033).
    await waitFor(() => {
      expect(
        within(dialog).getByLabelText<HTMLInputElement>(
          named(message(language, 'accounts-field-code')),
        ).value,
      ).toBe('0034');
    });
    expect(accountApi.accountsNextCode).toHaveBeenLastCalledWith('buyer', null, true);
    act(() => {
      document.activeElement?.dispatchEvent(
        new KeyboardEvent('keydown', { code: 'F10', key: 'F10', bubbles: true }),
      );
    });
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      account_type: 'buyer',
      is_pukhta_party: true,
      code: '0034',
      credit_allowed: false,
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ code: '0034' }));
    expect(input.value).not.toBe('');
  });

  it('an ordinary buyer made on the spot is saved without the mark, with credit on, as before', async () => {
    const { user, dialog } = await openQuickAdd('buyer', 'zafar');
    await waitFor(() => {
      expect(
        within(dialog).getByLabelText<HTMLInputElement>(
          named(message(language, 'accounts-field-code')),
        ).value,
      ).toBe('0023');
    });
    expect(user).toBeDefined();
    act(() => {
      document.activeElement?.dispatchEvent(
        new KeyboardEvent('keydown', { code: 'F10', key: 'F10', bubbles: true }),
      );
    });
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    const draft = vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0];
    expect(draft).toMatchObject({ account_type: 'buyer', code: '0023', credit_allowed: true });
    expect(draft?.is_pukhta_party).not.toBe(true);
  });

  it('the full form link carries the kind chosen in the short form', async () => {
    const onAdd = vi.fn<(request: AddRecordRequest) => void>();
    const { user, dialog } = await openQuickAdd('seller', 'zafar', { onAdd });
    await user.click(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'accounts-field-type')),
      }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(kindLabel(language, 'pukhta_seller')) }),
    );
    await user.click(
      within(dialog).getByRole('button', {
        name: named(message(language, 'search-quick-full-form')),
      }),
    );
    expect(onAdd).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'seller', form: 'full', accountKind: 'pukhta_seller' }),
    );
  });
});
