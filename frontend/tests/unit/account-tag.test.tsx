// The tag (نشان) of an account (plan 17e, PRD 10.2, FR-7.28, FR-7.40, FR-10.7, E38): English capital
// letters after a name, shown after the name in both languages through `displayName` and nowhere
// else, kept with a picked record by `pickedOf`, typed as plain text on the account form and the
// quick add (never converted to Urdu, made capital on leaving the field), and sent as typed.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { accountInputOf, pickedOf } from '../../src/shared/accounts';
import { displayName, otherScriptName, useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import { AccountPicker, Field, PartyCard, type PickerValue } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, cardOf, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const ASHIQ = account(61, 'buyer', '00260', 'عاشق ظفر', 'Ashiq Zafar', {
  tag: 'K',
  balance: balance(1000, 'dr'),
});
const ASHIQ_NO_TAG = account(62, 'buyer', '00261', 'عاشق ظفر', 'Ashiq Zafar');

const named = (text: string) => (name: string) => visible(name) === text;
function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(suggestionsFor({ toUrdu: {}, toEnglish: {} }, text, direction)),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
  useLanguageStore.setState({ language: 'ur' });
});

describe('displayName and the tag', () => {
  const record = { name_ur: 'عاشق ظفر', name_en: 'Ashiq Zafar' };

  it('puts the tag after the name in both languages, and only when there is one', () => {
    expect(displayName({ ...record, tag: 'K' }, 'ur')).toBe('عاشق ظفر K');
    expect(displayName({ ...record, tag: 'K' }, 'en')).toBe('Ashiq Zafar K');
    expect(displayName({ ...record, tag: 'D.P' }, 'en')).toBe('Ashiq Zafar D.P');
    for (const none of [undefined, null, '']) {
      expect(displayName({ ...record, tag: none }, 'ur')).toBe('عاشق ظفر');
      expect(displayName({ ...record, tag: none }, 'en')).toBe('Ashiq Zafar');
    }
    expect(displayName(record, 'en')).toBe('Ashiq Zafar');
  });

  it('the name of the other script has the tag too, so the second line tells two people apart', () => {
    expect(otherScriptName({ ...record, tag: 'K' }, 'ur')).toBe('Ashiq Zafar K');
    expect(otherScriptName({ ...record, tag: 'K' }, 'en')).toBe('عاشق ظفر K');
  });
});

describe('pickedOf', () => {
  it('keeps the id, the code, both names and the tag of a hit', () => {
    expect(
      pickedOf({ id: 61, code: '00260', name_ur: 'عاشق ظفر', name_en: 'Ashiq Zafar', tag: 'K' }),
    ).toEqual({
      id: 61,
      code: '00260',
      name_ur: 'عاشق ظفر',
      name_en: 'Ashiq Zafar',
      tag: 'K',
    });
  });

  it('leaves the tag out when there is none, so a record without one has no empty tag', () => {
    for (const tag of [undefined, null, '']) {
      const picked = pickedOf({ id: 62, code: '00261', name_ur: 'ا', name_en: 'A', tag });
      expect('tag' in picked).toBe(false);
    }
  });

  it('works for a row of the account list and for a reference, the same way', () => {
    expect(pickedOf(ASHIQ).tag).toBe('K');
    expect(pickedOf(ASHIQ_NO_TAG).tag).toBeUndefined();
  });
});

describe('what the account form sends', () => {
  it('sends the tag as typed text, and an empty text when there is none', () => {
    const draft = {
      id: null,
      version: null,
      account_type: 'buyer' as const,
      control_id: null,
      code: '00260',
      names: { urdu: 'عاشق ظفر', english: 'Ashiq Zafar' },
      tag: 'k',
      father: { urdu: '', english: '' },
      city: null,
      adda: null,
      phone1: '',
      phone2: '',
      address: '',
      linked_account: null,
      opening_date: '10-10-2026',
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
    // The frontend does not make it capital for the backend: the backend decides (PRD 10.2).
    expect(accountInputOf(draft, []).tag).toBe('k');
    expect(accountInputOf({ ...draft, tag: '' }, []).tag).toBe('');
  });
});

describe.each(LANGUAGES)('the tag on the account form (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('is a plain field right after the English name, with a hint, and it is empty at the start', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const english = await labelled(language, 'accounts-field-name-en');
    const tag = await labelled(language, 'accounts-field-tag');
    expect(tag.value).toBe('');
    expect(tag).toHaveAttribute('lang', 'en');
    expect(tag).toHaveAttribute('dir', 'ltr');
    // After the two names in the page, before the father or firm name.
    const order = [...document.querySelectorAll('input')];
    const father = await labelled(language, 'accounts-field-father-ur');
    expect(order.indexOf(tag)).toBeGreaterThan(order.indexOf(english));
    expect(order.indexOf(tag)).toBeLessThan(order.indexOf(father));
    // The hint is behind the ⓘ next to the label of the field (plan 38 G5).
    const label = screen.getByText(named(message(language, 'accounts-field-tag')));
    const info = within(label.parentElement ?? document.body).getByRole('button', {
      name: named(message(language, 'common-info-label')),
    });
    await userEvent.setup().click(info);
    expect(await screen.findByText(message(language, 'accounts-field-tag-hint'))).toBeVisible();
  });

  it('makes the letters capital when he leaves the field, and never turns them into Urdu', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    const tag = await labelled(language, 'accounts-field-tag');
    await user.click(tag);
    await user.keyboard('ba');
    // Typed letters stay English letters while typing: no suggestion list is offered.
    expect(tag.value).toBe('ba');
    expect(screen.queryAllByRole('option')).toHaveLength(0);
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
    await user.tab();
    expect(tag.value).toBe('BA');
  });

  it('F10 saves with the tag, and the list then shows the name with its tag after it', async () => {
    const store = fakeAccounts([]);
    const { router } = await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-en'));
    await user.keyboard('Ashiq Zafar');
    await user.click(await labelled(language, 'accounts-field-name-ur'));
    await user.keyboard('عاشق ظفر');
    await user.click(await labelled(language, 'accounts-field-tag'));
    await user.keyboard('k');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({ tag: 'k' });
    expect(store.rows[0]?.tag).toBe('K');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/accounts');
    });
    const cell = await screen.findByText(named(language === 'ur' ? 'عاشق ظفر K' : 'Ashiq Zafar K'));
    expect(cell).toBeInTheDocument();
  });

  it('a refused tag shows at the tag field, with words that say what is allowed', async () => {
    fakeAccounts([]);
    vi.mocked(accountApi.accountsSave).mockRejectedValue(
      fail({ code: 'ERR_TAG_INVALID', field: 'tag' }),
    );
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-en'));
    await user.keyboard('Ashiq Zafar');
    await user.click(await labelled(language, 'accounts-field-name-ur'));
    await user.keyboard('عاشق ظفر');
    const tag = await labelled(language, 'accounts-field-tag');
    await user.click(tag);
    await user.keyboard('1A');
    press('F10');
    await waitFor(() => {
      expect(tag).toHaveAttribute('aria-invalid', 'true');
    });
    expect(screen.getByText(message(language, 'err-tag-invalid'))).toBeInTheDocument();
  });

  it('opens an account with its tag in the field, and the heading names him with it', async () => {
    fakeAccounts([ASHIQ]);
    await renderApp('/index/accounts/61');
    const tag = await labelled(language, 'accounts-field-tag');
    await waitFor(() => {
      expect(tag.value).toBe('K');
    });
    const heading = await screen.findByRole('heading', { level: 1 });
    expect(visible(heading.textContent)).toContain(
      language === 'ur' ? 'عاشق ظفر K' : 'Ashiq Zafar K',
    );
  });

  it('two people with the same name: the similar list shows each with his tag, and says a tag is not a duplicate', async () => {
    fakeAccounts([ASHIQ, ASHIQ_NO_TAG]);
    vi.mocked(accountApi.accountsSimilar).mockResolvedValue([
      {
        id: 61,
        code: '00260',
        name_ur: 'عاشق ظفر',
        name_en: 'Ashiq Zafar',
        tag: 'K',
        father_or_firm_ur: '',
        father_or_firm_en: '',
        city_ur: '',
        city_en: '',
        phone1: '03001234567',
        account_type: 'buyer',
        is_duplicate: false,
      },
      {
        id: 62,
        code: '00261',
        name_ur: 'عاشق ظفر',
        name_en: 'Ashiq Zafar',
        father_or_firm_ur: '',
        father_or_firm_en: '',
        city_ur: '',
        city_en: '',
        phone1: '03001234567',
        account_type: 'buyer',
        is_duplicate: false,
      },
    ]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-en'));
    await user.keyboard('Ashiq');
    const heading = await screen.findByRole('heading', {
      name: message(language, 'accounts-similar-title'),
    });
    const panel = heading.closest('section');
    if (panel === null) throw new Error('no similar panel');
    const rows = within(panel).getAllByRole('listitem');
    const nameOf = (item: HTMLElement) =>
      visible(item.querySelector('.font-semibold')?.textContent);
    const english = language === 'en';
    expect(rows.map(nameOf)).toEqual([
      english ? 'Ashiq Zafar K' : 'عاشق ظفر K',
      english ? 'Ashiq Zafar' : 'عاشق ظفر',
    ]);
    expect(
      within(panel).getByText(message(language, 'accounts-similar-tag-help')),
    ).toBeInTheDocument();
    // Neither is a duplicate, so the panel keeps its plain title and marks no row.
    expect(
      within(panel).queryByText(message(language, 'accounts-similar-duplicate-chip')),
    ).toBeNull();
  });

  it("the tag typed goes with the question, and the backend's duplicate mark is shown in words on that row only", async () => {
    fakeAccounts([ASHIQ, ASHIQ_NO_TAG]);
    const base = {
      code: '00260',
      name_ur: 'عاشق ظفر',
      name_en: 'Ashiq Zafar',
      father_or_firm_ur: '',
      father_or_firm_en: '',
      city_ur: '',
      city_en: '',
      phone1: '03001234567',
      account_type: 'buyer' as const,
    };
    vi.mocked(accountApi.accountsSimilar).mockResolvedValue([
      { ...base, id: 61, tag: 'K', is_duplicate: true },
      { ...base, id: 62, code: '00261', is_duplicate: false },
    ]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-en'));
    await user.keyboard('Ashiq Zafar');
    await user.click(await labelled(language, 'accounts-field-tag'));
    await user.keyboard('k');
    await user.tab();
    await waitFor(() => {
      expect(accountApi.accountsSimilar).toHaveBeenLastCalledWith(
        expect.objectContaining({ name_en: 'Ashiq Zafar', tag: 'K' }),
      );
    });
    const heading = await screen.findByRole('heading', {
      name: message(language, 'accounts-similar-duplicate-title'),
    });
    const panel = must(heading.closest('section'));
    const marked = within(panel).getAllByText(message(language, 'accounts-similar-duplicate-chip'));
    expect(marked).toHaveLength(1);
    expect(must(marked[0]?.closest('li')).textContent).toContain('00260');
    // The save is never blocked by the warning: F10 still saves.
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalled();
    });
  });
});

function Harness({ onValue }: { onValue: (value: PickerValue | null) => void }) {
  const [value, setValue] = useState<PickerValue | null>(null);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <AccountPicker
            {...control}
            scope="buyer"
            value={value}
            onChange={(hit) => {
              const next = hit === null ? null : pickedOf(hit);
              setValue(next);
              onValue(next);
            }}
          />
        )}
      </Field>
    </form>
  );
}

describe.each(LANGUAGES)('the tag in the picker and the party card (%s)', (language) => {
  beforeEach(() => {
    fakeAccountSearch([ASHIQ, ASHIQ_NO_TAG]);
    vi.mocked(accountApi.assistPartyCard).mockImplementation((id) =>
      Promise.resolve(cardOf(id === 61 ? ASHIQ : ASHIQ_NO_TAG)),
    );
  });

  it('a row shows the name with its tag, the same name without one is a different row, and the field keeps the tag', async () => {
    act(() => {
      useLanguageStore.setState({ language });
    });
    const picked: (PickerValue | null)[] = [];
    renderIn(
      language,
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness onValue={(value) => picked.push(value)} />
      </QueryClientProvider>,
    );
    const input = screen.getByRole<HTMLInputElement>('combobox');
    const user = userEvent.setup();
    await user.click(input);
    await user.keyboard(language === 'ur' ? 'عاشق' : 'ashiq');
    const options = await screen.findAllByRole('option');
    const first = language === 'ur' ? 'عاشق ظفر K' : 'Ashiq Zafar K';
    const second = language === 'ur' ? 'عاشق ظفر' : 'Ashiq Zafar';
    const texts = options.map((option) => visible(option.textContent));
    expect(texts.some((text) => text.includes(first))).toBe(true);
    expect(texts.filter((text) => text.includes(first))).toHaveLength(1);
    expect(texts.some((text) => text.includes(second) && !text.includes(first))).toBe(true);
    const row = options.find((option) => visible(option.textContent).includes(first));
    if (row === undefined) throw new Error('no row with the tag');
    await user.click(row);
    expect(picked.at(-1)).toMatchObject({ id: 61, tag: 'K' });
    expect(input.value).toBe(first);
    // A language switch re-draws the name from the record: the tag stays.
    act(() => {
      useLanguageStore.setState({ language: language === 'ur' ? 'en' : 'ur' });
    });
    expect(input.value).toBe(language === 'ur' ? 'Ashiq Zafar K' : 'عاشق ظفر K');
  });

  it('the quick add has the tag after the names, and the account it makes comes back to the field with it', async () => {
    act(() => {
      useLanguageStore.setState({ language });
    });
    vi.mocked(accountApi.accountsNextCode).mockResolvedValue('00262');
    vi.mocked(accountApi.accountsSave).mockResolvedValue(
      account(63, 'buyer', '00262', 'عاشق ظفر', 'Ashiq Zafar', { tag: 'BA' }),
    );
    const picked: (PickerValue | null)[] = [];
    renderIn(
      language,
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness onValue={(value) => picked.push(value)} />
      </QueryClientProvider>,
    );
    const input = screen.getByRole<HTMLInputElement>('combobox');
    const user = userEvent.setup();
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    act(() => {
      input.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
    });
    const dialog = await screen.findByRole('dialog');
    const field = (id: string) =>
      within(dialog).getByLabelText<HTMLInputElement>(named(message(language, id)));
    const order = [...dialog.querySelectorAll('input')];
    expect(order.indexOf(field('accounts-field-tag'))).toBeGreaterThan(
      order.indexOf(field('accounts-field-name-en')),
    );
    // The short form asks the names, the tag, the phone and the city (plan 41); father or firm is the
    // full form's.
    expect(order.indexOf(field('accounts-field-tag'))).toBeLessThan(
      order.indexOf(field('accounts-field-phone1')),
    );
    expect(
      within(dialog).queryByLabelText(named(message(language, 'accounts-field-father-ur'))),
    ).toBeNull();
    await user.clear(field('accounts-field-name-en'));
    await user.type(field('accounts-field-name-en'), 'Ashiq Zafar');
    await user.clear(field('accounts-field-name-ur'));
    await user.type(field('accounts-field-name-ur'), 'عاشق ظفر');
    await user.click(field('accounts-field-tag'));
    await user.keyboard('ba');
    await user.tab();
    expect(field('accounts-field-tag').value).toBe('BA');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({ tag: 'BA' });
    await waitFor(() => {
      expect(picked.at(-1)).toMatchObject({ id: 63, tag: 'BA' });
    });
    expect(input.value).toBe(language === 'ur' ? 'عاشق ظفر BA' : 'Ashiq Zafar BA');
  });

  it('the party card names him with his tag', async () => {
    act(() => {
      useLanguageStore.setState({ language });
    });
    renderIn(
      language,
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <PartyCard accountId={61} />
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        language === 'ur' ? 'عاشق ظفر K' : 'Ashiq Zafar K',
      );
    });
  });
});
