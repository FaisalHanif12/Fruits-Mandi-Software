// The tag (plan 17e) through the real wrappers, against the backend's own answers: the bridge is
// stood in for, so each wrapper runs as in the app. The request each wrapper sends is the request
// the backend's test sent; the answer is the backend's own JSON (tests/fixtures, copied, never
// edited). Then the screens that show the answers: the global search (Ctrl+K), the party card and the
// account picker, in Urdu and in English, with the tag after the name through `displayName` only.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { GlobalSearch } from '../../src/app/layout/GlobalSearch';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  accountsSave,
  accountsSimilar,
  assistPartyCard,
  pickedOf,
  searchAccounts,
  searchGlobal,
  type AccountDraft,
} from '../../src/shared/accounts';
import {
  reportsCollectionPrintData,
  reportsPrintCollection,
  reportsPrintHeader,
} from '../../src/features/reports/api';
import * as invoke from '../../src/shared/api/invoke';
import { displayName, useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import { AccountPicker, Field, PartyCard, type PickerValue } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';
import { sendsTheSame } from '../helpers/shapes';
import * as real from '../helpers/tag-real';

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

const DRAFT: AccountDraft = {
  id: null,
  version: null,
  account_type: 'buyer',
  control_id: null,
  code: '',
  names: { urdu: 'اکرم ٹول ٹیکس', english: 'Akram Toll Tax' },
  tag: 'k',
  father: { urdu: '', english: '' },
  city: null,
  adda: null,
  phone1: '03001234567',
  phone2: '',
  address: '',
  linked_account: null,
  opening_date: '',
  opening_dr: '',
  opening_cr: '',
  opening_is_advance: false,
  credit_allowed: true,
  credit_limit: '',
  default_marka: null,
  collector: null,
  collection_weekday: null,
  is_active: true,
  note: '',
};

describe("the wrappers send the backend's requests and read its answers", () => {
  it('accounts_save sends the tag as typed, and the saved row has it capital', async () => {
    backend({ accounts_save: real.ACCOUNTS_SAVE_TAG });
    const saved = await accountsSave(DRAFT, []);
    const [command, args] = call.mock.calls[0] ?? [];
    expect(command).toBe('accounts_save');
    const input = (args as { input: object }).input;
    // The backend's own test typed the same names, phone and tag.
    expect(sendsTheSame(input, real.ACCOUNTS_SAVE_TAG_REQUEST)).toBeNull();
    expect(saved.tag).toBe('K');
    expect(displayName(saved, 'en')).toBe('Akram Toll Tax K');
  });

  it('accounts_similar sends the tag too, and reads which account is a duplicate', async () => {
    backend({ accounts_similar: real.ACCOUNTS_SIMILAR_DUPLICATE });
    const request = real.ACCOUNTS_SIMILAR_DUPLICATE_REQUEST;
    const found = await accountsSimilar({ ...request, name_en: request.name_en });
    expect(call).toHaveBeenCalledWith('accounts_similar', { input: request });
    expect(found).toHaveLength(1);
    expect(found[0]?.is_duplicate).toBe(true);
    expect(found[0]?.tag).toBe('K');
    backend({ accounts_similar: real.ACCOUNTS_SIMILAR_OTHER });
    const other = await accountsSimilar({ ...real.ACCOUNTS_SIMILAR_OTHER_REQUEST });
    // The same name with another tag is listed, but it is not a duplicate (PRD E38).
    expect(other[0]?.is_duplicate).toBe(false);
  });

  it('search_accounts keeps the tag of a hit, and the picked record keeps it too', async () => {
    backend({ search_accounts: real.SEARCH_ACCOUNTS_TAG, accounts_balances: [] });
    const request = real.SEARCH_ACCOUNTS_TAG_REQUEST;
    const hits = await searchAccounts({
      scope: request.scope,
      controlId: request.control_id,
      text: request.text,
      limit: request.limit,
    });
    // The wrapper always sends the page it asks for; the backend's file leaves the first page out.
    expect(call).toHaveBeenCalledWith('search_accounts', { input: { ...request, offset: 0 } });
    expect(hits[0]?.tag).toBe('K');
    const first = hits[0];
    if (first === undefined) throw new Error('no hit');
    expect(pickedOf(first).tag).toBe('K');
  });

  it('search_global keeps the tag of an account hit, for a name and for a number', async () => {
    backend({ search_global: real.SEARCH_GLOBAL_TAG });
    const byName = await searchGlobal('akram k');
    const sent = call.mock.calls[0];
    expect(sent?.[0]).toBe('search_global');
    expect((sent?.[1] as { input: { text: string } }).input.text).toBe(
      real.SEARCH_GLOBAL_TAG_REQUEST.text,
    );
    const hit = byName[0];
    expect(hit?.kind).toBe('account');
    if (hit?.kind !== 'account') throw new Error('not an account');
    expect(hit.tag).toBe('K');
    backend({ search_global: real.SEARCH_GLOBAL_TAG_NUMBER });
    const byNumber = await searchGlobal('1');
    const tagged = byNumber.filter((item) => item.kind === 'account' && item.tag !== undefined);
    // Only Akram has a tag; the other hits of the number have none and show the name alone.
    expect(tagged).toHaveLength(1);
    expect(byNumber.length).toBeGreaterThan(1);
  });

  it('assist_party_card reads the tag and the balance of the card', async () => {
    backend({ assist_party_card: real.PARTY_CARD_TAG });
    const card = await assistPartyCard(20);
    expect(call).toHaveBeenCalledWith('assist_party_card', { input: { account_id: 20 } });
    expect(card.tag).toBe('K');
    expect(card.balance.side).toBe('dr');
  });
});

describe("the report print wrappers send the backend's requests and read its answers", () => {
  it('reports_print_header: the one command of every report print gives the frame', async () => {
    for (const [request, answer] of [
      [real.PRINT_HEADER_UR_REQUEST, real.PRINT_HEADER_UR],
      [real.PRINT_HEADER_EN_REQUEST, real.PRINT_HEADER_EN],
    ] as const) {
      backend({ reports_print_header: answer });
      const frame = await reportsPrintHeader(request);
      expect(call).toHaveBeenLastCalledWith('reports_print_header', { input: request });
      expect(frame.header.contacts).toHaveLength(2);
      expect(frame.header.trade_mark).toBe('RH');
      expect(frame.urdu_digits).toBe(false);
      expect(frame.language).toBe(request.language);
    }
  });

  it('reports_collection_print_data: the list of a day, in each language, and an empty day', async () => {
    for (const [request, answer] of [
      [real.COLLECTION_UR_REQUEST, real.COLLECTION_UR],
      [real.COLLECTION_EN_REQUEST, real.COLLECTION_EN],
      [real.COLLECTION_EMPTY_REQUEST, real.COLLECTION_EMPTY],
    ] as const) {
      backend({ reports_collection_print_data: answer });
      const data = await reportsCollectionPrintData(request);
      expect(call).toHaveBeenLastCalledWith('reports_collection_print_data', { input: request });
      expect(data.doc).toBe('collection_list');
      expect(data.urdu_digits).toBe(false);
    }
    expect(real.COLLECTION_EMPTY.groups).toEqual([]);
    expect(real.COLLECTION_EMPTY.total.amount).toBe('0');
  });

  it('reports_print_collection: the day, the filters, the language, the copies as text and where it goes', async () => {
    backend({ reports_print_collection: { printed: ['collection_list'], pdf_paths: null } });
    const input = {
      date: '10-10-2026',
      collector_staff_id: 81,
      city_id: null,
      search: null,
      language: 'ur' as const,
      copies: '2',
      to: 'printer' as const,
    };
    const outcome = await reportsPrintCollection(input);
    expect(call).toHaveBeenLastCalledWith('reports_print_collection', { input });
    expect(outcome.printed).toEqual(['collection_list']);
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

describe.each(LANGUAGES)('the screens show the real answers with the tag (%s)', (language) => {
  const akram = language === 'ur' ? 'اکرم ٹول ٹیکس K' : 'Akram Toll Tax K';

  it('the global search lists Akram with his tag after the name, found by "akram k"', async () => {
    setLanguage(language);
    backend({ search_global: real.SEARCH_GLOBAL_TAG });
    function Box() {
      useEscapeDispatcher();
      return (
        <GlobalSearch
          onOpenAccount={() => undefined}
          onOpenBikri={() => undefined}
          onOpenVoucher={() => undefined}
        />
      );
    }
    renderIn(language, wrapper(<Box />));
    const user = userEvent.setup();
    await user.click(screen.getByRole('combobox'));
    await user.keyboard('akram k');
    const option = await screen.findByRole('option');
    expect(visible(option.textContent)).toContain(akram);
    // The other script's name has the tag too: it tells two people with one name apart.
    expect(visible(option.textContent)).toContain(
      language === 'ur' ? 'Akram Toll Tax K' : 'اکرم ٹول ٹیکس K',
    );
    // What was typed went to the backend as typed: nothing was turned into Urdu.
    const asked = call.mock.calls.find((item) => item[0] === 'search_global');
    expect((asked?.[1] as { input: { text: string } }).input.text).toBe('akram k');
  });

  it('the party card names him with his tag, his balance and his credit used', async () => {
    setLanguage(language);
    backend({ assist_party_card: real.PARTY_CARD_TAG });
    renderIn(language, wrapper(<PartyCard accountId={20} />));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(akram);
    });
    expect(visible(document.body.textContent)).toContain('2,880');
  });

  it('the account picker row and the field show the tag of the real hit, and keep it when picked', async () => {
    setLanguage(language);
    backend({
      search_accounts: real.SEARCH_ACCOUNTS_TAG,
      search_accounts_recent: [],
      search_accounts_touch: null,
      accounts_balances: [],
      assist_party_card: real.PARTY_CARD_TAG,
    });
    const picked: (PickerValue | null)[] = [];
    function Harness() {
      const [value, setValue] = useState<PickerValue | null>(null);
      useEscapeDispatcher();
      return (
        <form {...formNavigationProps}>
          <Field label={message(language, 'common-back')}>
            {(control) => (
              <AccountPicker
                {...control}
                scope="any"
                value={value}
                onChange={(hit) => {
                  const next = hit === null ? null : pickedOf(hit);
                  setValue(next);
                  picked.push(next);
                }}
              />
            )}
          </Field>
        </form>
      );
    }
    renderIn(language, wrapper(<Harness />));
    const user = userEvent.setup();
    const input = screen.getByRole<HTMLInputElement>('combobox');
    await user.click(input);
    await user.keyboard('akram k');
    const row = await screen.findByRole('option');
    expect(visible(within(row).getByText(akram).textContent)).toBe(akram);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(picked.at(-1)).toMatchObject({ id: 20, tag: 'K' });
    });
    expect(input.value).toBe(akram);
  });
});
