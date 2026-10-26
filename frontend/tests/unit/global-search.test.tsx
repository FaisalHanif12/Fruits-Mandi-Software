// The search box of the top bar (plan 08, PRD FR-5.4, FR-7.35): Ctrl+K from anywhere, accounts by
// code or by name in either script, a بکری by its number or the start of its truck number (plan
// 13), ↑ ↓ and Enter to open one, Esc to clear. In the Urdu and in the English app, against a fake
// of the search command. It never converts what is typed.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { GlobalSearch } from '../../src/app/layout/GlobalSearch';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import type { GlobalHit, VoucherSearchHit } from '../../src/shared/accounts';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import * as urduApi from '../../src/shared/urdu/api';
import { fail } from '../helpers/app';
import { paisaForTest } from '../helpers/paisa';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM_REF = { id: 21, code: '0020', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' };

const HITS: GlobalHit[] = [
  {
    kind: 'account',
    id: 21,
    code: '0020',
    name_ur: 'اکرم ٹول ٹیکس',
    name_en: 'Akram Toll Tax',
    extra_ur: 'کوٹ رادھا کشن',
    extra_en: 'Kot Radha Kishan',
  },
  {
    kind: 'account',
    id: 22,
    code: '0021',
    name_ur: 'ارشد ملتانی',
    name_en: 'Arshad Multani',
    extra_ur: '',
    extra_en: '',
  },
  // The same number as an account id: the two kinds are different rows (plan 13).
  {
    kind: 'bikri',
    id: 22,
    number: 5,
    entry_date: '2026-10-10',
    truck_no: 'LES-1234',
    status: 'saved',
    matched: 'truck',
  },
  {
    kind: 'bikri',
    id: 30,
    number: 7,
    entry_date: '2026-10-09',
    truck_no: 'LES-9999',
    status: 'cancelled',
    matched: 'truck',
  },
  // Vouchers (plan 15): found by their number, in each series.
  {
    kind: 'voucher',
    id: 501,
    voucher_type: 'receipt',
    number: 12,
    entry_date: '2026-10-10',
    account: AKRAM_REF,
    amount_paisa: paisaForTest(1_000_000),
    status: 'saved',
  },
  {
    kind: 'voucher',
    id: 502,
    voucher_type: 'payment',
    number: 12,
    entry_date: '2026-10-09',
    account: { id: 31, code: '0010', name_ur: 'رحیم', name_en: 'Rahim' },
    amount_paisa: paisaForTest(431_600),
    status: 'cancelled',
  },
];

/** What the fake search command finds: accounts by code or name, a بکری by number or truck start. */
function found(hit: GlobalHit, text: string): boolean {
  const needle = text.toLowerCase();
  if (hit.kind === 'voucher') return String(hit.number) === text;
  return hit.kind === 'bikri'
    ? String(hit.number) === text || (hit.truck_no ?? '').toLowerCase().startsWith(needle)
    : [hit.code, hit.name_ur, hit.name_en].some((part) => part.toLowerCase().includes(needle));
}

/** The shell mounts the Esc dispatcher once; the box asks it for the Esc order (F27). */
function Shell({
  onOpenAccount,
  onOpenBikri,
  onOpenVoucher,
}: {
  onOpenAccount: (id: number) => void;
  onOpenBikri: (id: number) => void;
  onOpenVoucher: (hit: VoucherSearchHit) => void;
}) {
  useEscapeDispatcher();
  return (
    <GlobalSearch
      onOpenAccount={onOpenAccount}
      onOpenBikri={onOpenBikri}
      onOpenVoucher={onOpenVoucher}
    />
  );
}

function setup(language: Language) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const onOpenAccount = vi.fn();
  const onOpenBikri = vi.fn();
  const onOpenVoucher = vi.fn();
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Shell
        onOpenAccount={onOpenAccount}
        onOpenBikri={onOpenBikri}
        onOpenVoucher={onOpenVoucher}
      />
    </QueryClientProvider>,
  );
  const box = screen.getByRole<HTMLInputElement>('combobox');
  return { box, onOpenAccount, onOpenBikri, onOpenVoucher, user: userEvent.setup() };
}

const options = () => screen.queryAllByRole('option');

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(accountApi.searchGlobal).mockImplementation((text) =>
    Promise.resolve(HITS.filter((hit) => found(hit, text))),
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
    expect(row).toContain('0020');
    expect(row).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(row).toContain(language === 'ur' ? 'Akram Toll Tax' : 'اکرم ٹول ٹیکس');
    expect(row).toContain(language === 'ur' ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan');
    await user.clear(box);
    await user.keyboard('ارشد');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.clear(box);
    await user.keyboard('002');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
  });

  it('↓ then Enter opens the highlighted account and clears the box', async () => {
    const { box, user, onOpenAccount } = setup(language);
    await user.click(box);
    await user.keyboard('002');
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

  it('finds a بکری by its number and by the start of its truck number, and opens it by its id (FR-5.4)', async () => {
    const { box, user, onOpenAccount, onOpenBikri } = setup(language);
    await user.click(box);
    await user.keyboard('les');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    const rows = options().map((row) => visible(row.textContent));
    expect(rows[0]).toContain(message(language, 'bikri-hit-title', { number: 5 }));
    expect(rows[0]).toContain('LES-1234');
    expect(rows[0]).toContain('10-10-2026');
    // A cancelled بکری says so in words, not only in colour (NFR-USE-12).
    expect(rows[1]).toContain(message(language, 'bikri-status-cancelled'));
    await user.keyboard('{ArrowDown}{Enter}');
    // The بکری has the same id as an account (22): only the بکری screen is opened.
    expect(onOpenBikri).toHaveBeenCalledWith(30);
    expect(onOpenAccount).not.toHaveBeenCalled();
    expect(box.value).toBe('');
  });

  it('a click on a بکری result opens that بکری, not the account with the same id', async () => {
    const { box, user, onOpenAccount, onOpenBikri } = setup(language);
    await user.click(box);
    await user.keyboard('5');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const first = options()[0];
    if (first === undefined) throw new Error('no result');
    await user.click(first);
    expect(onOpenBikri).toHaveBeenCalledWith(22);
    expect(onOpenAccount).not.toHaveBeenCalled();
  });

  it('finds a voucher by its number: its kind and number in its series, day, account and amount (FR-5.4)', async () => {
    const { box, user } = setup(language);
    await user.click(box);
    await user.keyboard('12');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    const rows = options().map((row) => visible(row.textContent));
    expect(rows[0]).toContain(message(language, 'vouchers-type-receipt'));
    expect(rows[0]).toContain(message(language, 'vouchers-number-receipt', { number: 12 }));
    expect(rows[0]).toContain('10-10-2026');
    expect(rows[0]).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(rows[0]).toContain('10,000');
    // The same number in the other series is another row, and a cancelled one says so in words.
    expect(rows[1]).toContain(message(language, 'vouchers-number-payment', { number: 12 }));
    expect(rows[1]).toContain(message(language, 'vouchers-hit-cancelled'));
  });

  it('Enter on a voucher result opens that voucher, not an account or a بکری with the same id', async () => {
    const { box, user, onOpenAccount, onOpenBikri, onOpenVoucher } = setup(language);
    await user.click(box);
    await user.keyboard('12');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    await user.keyboard('{ArrowDown}{Enter}');
    expect(onOpenVoucher).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'voucher', id: 502, voucher_type: 'payment' }),
    );
    expect(onOpenAccount).not.toHaveBeenCalled();
    expect(onOpenBikri).not.toHaveBeenCalled();
    expect(box.value).toBe('');
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
