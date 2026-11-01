// A پختہ party's own account (plan 47, PRD FR-13.11) through the real wrappers, against the
// backend's own answers: the bridge is stood in for, so each wrapper runs as in the app. The request
// each wrapper sends is the request the backend's test sent; the answer is the backend's own JSON
// (tests/fixtures, copied, never edited). Then the screens that show the answers, in Urdu and in
// English: the picker row, the short code that two buyers share, and the party card. The story:
// نذیر 0032 (پختہ گاہک), کمال 0033 (پختہ بیوپاری), and the ordinary buyer طارق محمود 0022 whose
// running number 2 is also نذیر's.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  accountInputOf,
  accountsList,
  accountsNextCode,
  accountsResolveCode,
  accountsSave,
  assistPartyCard,
  pickedOf,
  searchAccounts,
  type AccountDraft,
  type AccountScope,
} from '../../src/shared/accounts';
import * as invoke from '../../src/shared/api/invoke';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import { AccountPicker, Field, PartyCard, type PickerValue } from '../../src/shared/ui';
import { account, cardOf } from '../helpers/accounts';
import * as real from '../helpers/pukhta-party-real';
import { message, renderIn, visible } from '../helpers/render';
import { same, sendsTheSame } from '../helpers/shapes';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

/** Answers each command by name, from the backend's own JSON; a command not listed fails. */
function backend(answers: Record<string, unknown>) {
  call.mockImplementation((command: string) => {
    if (command in answers) return Promise.resolve(answers[command] as never);
    return Promise.reject(new Error(`no answer for ${command}`));
  });
}

beforeEach(() => {
  vi.clearAllMocks();
});

/** What the form holds for a new پختہ party: his names and phone, credit off, nothing else. */
function draftOf(
  side: 'buyer' | 'seller',
  names: { urdu: string; english: string },
  phone: string,
): AccountDraft {
  return {
    id: null,
    version: null,
    account_type: side,
    is_pukhta_party: true,
    control_id: null,
    code: '',
    names,
    tag: '',
    father: { urdu: '', english: '' },
    city: null,
    adda: null,
    phone1: phone,
    phone2: '',
    address: '',
    linked_account: null,
    opening_date: '',
    opening_dr: '',
    opening_cr: '',
    opening_is_advance: false,
    credit_allowed: side === 'seller',
    credit_limit: '',
    default_marka: null,
    collector: null,
    collection_weekday: null,
    is_active: true,
    note: '',
  };
}

describe("the wrappers send the backend's requests and read its answers", () => {
  it('accounts_next_code asks for the پختہ series, for each side, and reads the code', async () => {
    for (const [type, request, answer] of [
      ['buyer', real.NEXT_CODE_BUYER_REQUEST, real.NEXT_CODE_BUYER],
      ['seller', real.NEXT_CODE_SELLER_REQUEST, real.NEXT_CODE_SELLER],
      ['buyer', real.NEXT_CODE_AFTER_TWO_REQUEST, real.NEXT_CODE_AFTER_TWO],
    ] as const) {
      backend({ accounts_next_code: answer });
      const code = await accountsNextCode(type, null, true);
      expect(call).toHaveBeenLastCalledWith('accounts_next_code', { input: request });
      expect(code).toBe(answer);
    }
    // The series starts at 0032, and the next code after two is 0034 (one counter, both sides).
    expect(real.NEXT_CODE_BUYER).toBe('0032');
    expect(real.NEXT_CODE_AFTER_TWO).toBe('0034');
  });

  it('accounts_next_code for an ordinary kind sends no mark, as before', async () => {
    backend({ accounts_next_code: '0023' });
    await accountsNextCode('buyer', null);
    expect(call).toHaveBeenLastCalledWith('accounts_next_code', {
      input: { account_type: 'buyer', control_id: null },
    });
  });

  it('accounts_save sends the mark with credit off, and reads the account with the mark', async () => {
    backend({ accounts_save: real.SAVE_BUYER });
    const draft = draftOf('buyer', { urdu: 'نذیر', english: 'Nazir' }, '0300-1111111');
    const saved = await accountsSave(draft, []);
    const [command, args] = call.mock.calls[0] ?? [];
    expect(command).toBe('accounts_save');
    expect(sendsTheSame((args as { input: object }).input, real.SAVE_BUYER_REQUEST)).toBeNull();
    expect(saved.is_pukhta_party).toBe(true);
    expect(saved.code).toBe('0032');
    expect(saved.account_type).toBe('buyer');
    expect(saved.is_pukhta_khata).toBe(false);
    expect(saved.credit_allowed).toBe(false);
  });

  it('accounts_save for a پختہ seller reads code 0033 and keeps his side', async () => {
    backend({ accounts_save: real.SAVE_SELLER });
    const draft = draftOf('seller', { urdu: 'کمال', english: 'Kamal' }, '');
    const saved = await accountsSave(draft, []);
    const [, args] = call.mock.calls[0] ?? [];
    expect(
      sendsTheSame(
        (args as { input: object }).input,
        // The backend's own request had phone empty too.
        real.SAVE_SELLER_REQUEST,
      ),
    ).toBeNull();
    expect(saved.is_pukhta_party).toBe(true);
    expect(saved.account_type).toBe('seller');
    expect(saved.code).toBe(real.SAVE_SELLER.code);
  });

  it('an ordinary account is sent without the mark, so its request is the old request', () => {
    const names = { urdu: 'طارق', english: 'Tariq' };
    const marked = draftOf('buyer', names, '');
    const unmarked: AccountDraft = { ...marked, is_pukhta_party: false };
    const missing: AccountDraft = { ...marked };
    Reflect.deleteProperty(missing, 'is_pukhta_party');
    expect(accountInputOf(unmarked, [])).not.toHaveProperty('is_pukhta_party');
    expect(accountInputOf(missing, [])).not.toHaveProperty('is_pukhta_party');
    expect(accountInputOf(marked, [])).toHaveProperty('is_pukhta_party', true);
  });

  it('the refusals name the field is_pukhta_party and the code ERR_FORMAT_INVALID', () => {
    for (const refused of [real.SAVE_WRONG_TYPE, real.SAVE_CHANGE]) {
      expect(refused.errors).toEqual([
        { code: 'ERR_FORMAT_INVALID', field: 'is_pukhta_party', values: {} },
      ]);
    }
    // The backend's own tests sent the mark on a wrong kind and a changed mark on an edit.
    expect(real.SAVE_WRONG_TYPE_REQUEST.is_pukhta_party).toBe(true);
    expect(real.SAVE_CHANGE_REQUEST.id).not.toBeNull();
  });

  it('accounts_list: the پختہ filter has the two shared accounts, then every own account', async () => {
    backend({ accounts_list: real.LIST_PUKHTA });
    const list = await accountsList(
      { kind: 'pukhta', text: '', include_inactive: false, has_opening: null },
      0,
      0,
    );
    expect(call).toHaveBeenLastCalledWith('accounts_list', { input: real.LIST_PUKHTA_REQUEST });
    expect(list.rows.map((row) => row.code)).toEqual(['0030', '0031', '0032', '0033']);
    expect(list.rows.map((row) => row.is_pukhta_khata)).toEqual([true, true, false, false]);
    expect(list.rows.map((row) => row.is_pukhta_party === true)).toEqual([
      false,
      false,
      true,
      true,
    ]);
    // His side is on the row: a buyer and a seller in one series.
    expect(list.rows.map((row) => row.account_type)).toEqual([
      'buyer',
      'seller',
      'buyer',
      'seller',
    ]);
  });

  it('search_accounts keeps the mark of a hit, and the picked record carries it', async () => {
    backend({ search_accounts: real.SEARCH_BUYER, accounts_balances: [] });
    const request = real.SEARCH_BUYER_REQUEST;
    const hits = await searchAccounts({
      scope: request.scope,
      controlId: request.control_id,
      text: request.text,
      limit: request.limit,
    });
    expect(call).toHaveBeenCalledWith('search_accounts', { input: { ...request, offset: 0 } });
    expect(hits[0]?.is_pukhta_party).toBe(true);
    const first = hits[0];
    if (first === undefined) throw new Error('no hit');
    expect(pickedOf(first).code).toBe('0032');
  });

  it('accounts_resolve_code: the running number 2 is two buyers in a buyer field and an any field, none in a seller field', async () => {
    for (const [scope, text, request, answer] of [
      ['buyer', '2', real.RESOLVE_BUYER_SHORT_2_REQUEST, real.RESOLVE_BUYER_SHORT_2],
      ['any', '2', real.RESOLVE_ANY_SHORT_2_REQUEST, real.RESOLVE_ANY_SHORT_2],
      ['seller', '2', real.RESOLVE_SELLER_SHORT_2_REQUEST, real.RESOLVE_SELLER_SHORT_2],
      ['seller', '3', real.RESOLVE_SELLER_SHORT_3_REQUEST, real.RESOLVE_SELLER_SHORT_3],
      ['buyer', '0032', real.RESOLVE_BUYER_FULL_REQUEST, real.RESOLVE_BUYER_FULL],
    ] as const) {
      backend({ accounts_resolve_code: answer });
      const resolved = await accountsResolveCode(scope, null, text);
      expect(call).toHaveBeenLastCalledWith('accounts_resolve_code', { input: request });
      expect(resolved.status).toBe(answer.status);
      expect(resolved.candidates).toHaveLength(answer.candidates.length);
    }
    expect(real.RESOLVE_BUYER_SHORT_2.status).toBe('several');
    // The ordinary buyer is listed first, then the پختہ one with his mark.
    expect(
      real.RESOLVE_BUYER_SHORT_2.candidates.map((hit) => [hit.code, hit.is_pukhta_party === true]),
    ).toEqual([
      ['0022', false],
      ['0032', true],
    ]);
    expect(real.RESOLVE_SELLER_SHORT_2.status).toBe('unknown');
    expect(real.RESOLVE_SELLER_SHORT_3.account?.code).toBe('0033');
    expect(real.RESOLVE_SELLER_SHORT_3.account?.account_type).toBe('seller');
  });

  it('assist_party_card reads the mark and the side of the card', async () => {
    backend({ assist_party_card: real.PARTY_CARD_BUYER });
    const card = await assistPartyCard(32);
    expect(call).toHaveBeenCalledWith('assist_party_card', { input: { account_id: 32 } });
    expect(card.is_pukhta_party).toBe(true);
    expect(card.account_type).toBe('buyer');
  });
});

describe('the canned data has the real shape', () => {
  const NAZIR = account(32, 'buyer', '0032', 'نذیر', 'Nazir', {
    is_pukhta_party: true,
    credit_allowed: false,
    phone1: '03001111111',
  });

  it('an account row of a پختہ party', () => {
    expect(same(real.SAVE_BUYER, NAZIR)).toBeNull();
  });

  it('his party card', () => {
    expect(same(real.PARTY_CARD_BUYER, cardOf(NAZIR))).toBeNull();
    expect(cardOf(NAZIR).is_pukhta_party).toBe(true);
  });
});

const wrapper = (children: React.ReactNode) => (
  <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
    {children}
  </QueryClientProvider>
);

function setLanguage(language: Language) {
  act(() => {
    useLanguageStore.setState({ language });
  });
}

function Harness({
  scope,
  onPick,
}: {
  scope: AccountScope;
  onPick: (v: PickerValue | null) => void;
}) {
  const [value, setValue] = useState<PickerValue | null>(null);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <AccountPicker
            {...control}
            scope={scope}
            value={value}
            onChange={(hit) => {
              const next = hit === null ? null : pickedOf(hit);
              setValue(next);
              onPick(next);
            }}
          />
        )}
      </Field>
    </form>
  );
}

describe.each(LANGUAGES)('the screens show the real answers (%s)', (language) => {
  const kind = message(language, 'accounts-type-pukhta_buyer');

  it('the picker row of a real hit names his kind, and picking him keeps his code', async () => {
    setLanguage(language);
    backend({
      search_accounts: real.SEARCH_BUYER,
      search_accounts_recent: [],
      search_accounts_touch: null,
      accounts_balances: [],
      assist_party_card: real.PARTY_CARD_BUYER,
    });
    const picked: (PickerValue | null)[] = [];
    renderIn(
      language,
      wrapper(
        <Harness
          scope="buyer"
          onPick={(value) => {
            picked.push(value);
          }}
        />,
      ),
    );
    const user = userEvent.setup();
    const input = screen.getByRole<HTMLInputElement>('combobox');
    await user.click(input);
    await user.keyboard('nazir');
    const row = await screen.findByRole('option');
    expect(visible(row.textContent)).toContain('0032');
    expect(visible(row.textContent)).toContain(message(language, 'accounts-type-pukhta'));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(picked.at(-1)).toMatchObject({ id: 32, code: '0032' });
    });
    // His card shows under the field with his kind.
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(kind);
    });
  });

  it('the short code 2 in a buyer field lists the ordinary buyer and the پختہ one with their kinds', async () => {
    setLanguage(language);
    backend({
      search_accounts: [],
      search_accounts_recent: [],
      search_accounts_touch: null,
      accounts_balances: [],
      accounts_resolve_code: real.RESOLVE_BUYER_SHORT_2,
      assist_party_card: real.PARTY_CARD_BUYER,
    });
    const picked: (PickerValue | null)[] = [];
    renderIn(
      language,
      wrapper(
        <Harness
          scope="buyer"
          onPick={(value) => {
            picked.push(value);
          }}
        />,
      ),
    );
    const user = userEvent.setup();
    await user.click(screen.getByRole('combobox'));
    await user.keyboard('2');
    await user.tab();
    expect(
      await screen.findByText(
        (text) => visible(text) === message(language, 'accountpicker-several'),
      ),
    ).toBeInTheDocument();
    const buttons = screen
      .getAllByRole('button')
      .filter((button) => /0022|0032/.test(visible(button.textContent)));
    expect(buttons).toHaveLength(2);
    expect(visible(buttons[0]?.textContent)).toContain(message(language, 'accounts-type-buyer'));
    expect(visible(buttons[0]?.textContent)).not.toContain(kind);
    expect(visible(buttons[1]?.textContent)).toContain(kind);
    expect(visible(buttons[1]?.textContent)).toContain('0032');
    const second = buttons[1];
    if (second === undefined) throw new Error('no second candidate');
    await user.click(second);
    expect(picked.at(-1)).toMatchObject({ id: 32, code: '0032' });
  });

  it('the party card names his kind, not only his side', async () => {
    setLanguage(language);
    backend({ assist_party_card: real.PARTY_CARD_BUYER });
    renderIn(language, wrapper(<PartyCard accountId={32} />));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(kind);
    });
    expect(visible(document.body.textContent)).toContain('0032');
  });
});
