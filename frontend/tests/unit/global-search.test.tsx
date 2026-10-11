// The search box of the top bar (plan 08, PRD FR-5.4, FR-7.35): Ctrl+K from anywhere, accounts by
// code or by name in either script, ↑ ↓ and Enter to open one, Esc to clear. In the Urdu and in
// the English app, against a fake of the search command. It never converts what is typed.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { GlobalSearch } from '../../src/app/layout/GlobalSearch';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import type { GlobalHit } from '../../src/shared/accounts';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fail } from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/urdu/api');

const HITS: GlobalHit[] = [
  {
    kind: 'account',
    id: 21,
    code: '03040001',
    name_ur: 'اکرم ٹول ٹیکس',
    name_en: 'Akram Toll Tax',
    extra_ur: 'کوٹ رادھا کشن',
    extra_en: 'Kot Radha Kishan',
  },
  {
    kind: 'account',
    id: 22,
    code: '03040002',
    name_ur: 'ارشد ملتانی',
    name_en: 'Arshad Multani',
    extra_ur: '',
    extra_en: '',
  },
];

function setup(language: Language) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const onOpenAccount = vi.fn();
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <GlobalSearch onOpenAccount={onOpenAccount} />
    </QueryClientProvider>,
  );
  const box = screen.getByRole<HTMLInputElement>('combobox');
  return { box, onOpenAccount, user: userEvent.setup() };
}

const options = () => screen.queryAllByRole('option');

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(accountApi.searchGlobal).mockImplementation((text) =>
    Promise.resolve(
      HITS.filter((hit) =>
        [hit.code, hit.name_ur, hit.name_en].some((part) =>
          part.toLowerCase().includes(text.toLowerCase()),
        ),
      ),
    ),
  );
});

describe.each(LANGUAGES)('GlobalSearch (%s)', (language) => {
  it('Ctrl+K focuses the box from anywhere', async () => {
    const { box, user } = setup(language);
    expect(box).not.toHaveFocus();
    await user.keyboard('{Control>}k{/Control}');
    expect(box).toHaveFocus();
  });

  it('finds an account by name in either script and by code, taking the text as typed', async () => {
    const { box, user } = setup(language);
    await user.click(box);
    await user.keyboard('akram');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    expect(accountApi.searchGlobal).toHaveBeenLastCalledWith('akram');
    const row = visible(options()[0]?.textContent);
    expect(row).toContain('03040001');
    expect(row).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(row).toContain(language === 'ur' ? 'Akram Toll Tax' : 'اکرم ٹول ٹیکس');
    expect(row).toContain(language === 'ur' ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan');
    await user.clear(box);
    await user.keyboard('ارشد');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.clear(box);
    await user.keyboard('0304');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
  });

  it('↓ then Enter opens the highlighted account and clears the box', async () => {
    const { box, user, onOpenAccount } = setup(language);
    await user.click(box);
    await user.keyboard('0304');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    await user.keyboard('{ArrowDown}{Enter}');
    expect(onOpenAccount).toHaveBeenCalledWith(22);
    expect(box.value).toBe('');
  });

  it('a click on a result opens it', async () => {
    const { box, user, onOpenAccount } = setup(language);
    await user.click(box);
    await user.keyboard('akram');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const first = options()[0];
    if (first === undefined) throw new Error('no result');
    await user.click(first);
    expect(onOpenAccount).toHaveBeenCalledWith(21);
  });

  it('says so when nothing is found, and Esc clears the box', async () => {
    const { box, user } = setup(language);
    await user.click(box);
    await user.keyboard('zzz');
    expect(await screen.findByText(message(language, 'layout-search-none'))).toBeInTheDocument();
    await user.keyboard('{Escape}');
    expect(box.value).toBe('');
    expect(screen.queryByText(message(language, 'layout-search-none'))).not.toBeInTheDocument();
  });

  it('a failure says so, with the plain message', async () => {
    vi.mocked(accountApi.searchGlobal).mockRejectedValue(fail({ code: 'ERR_DB_BUSY' }));
    const { box, user } = setup(language);
    await user.click(box);
    await user.keyboard('ak');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'layout-search-failed'));
    expect(visible(alert.textContent)).toContain(message(language, 'err-db-busy'));
  });
});
