// What the real backend does that the screen must follow (plan 15, T4): an inactive account is
// refused at the account field and the screen says what to do about it, making a one-time person a
// regular account reads every list it changes again, and the paths of the errors of a batch.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { rowPathOf } from '../../src/features/vouchers/collection';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import { useMakeRegular } from '../../src/features/vouchers/hooks';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { AKRAM, ARSHAD, KARIM, PUKHTA_BUYER, RAHIM, fakeVouchers } from '../helpers/vouchers';
import { EMPTY_PAIRS } from '../helpers/paged';

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

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  const rows = [AKRAM, ARSHAD, PUKHTA_BUYER, RAHIM, KARIM];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({});
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the voucher screen against the real rules (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeVouchers();
  });

  it('an inactive buyer: the refusal is at the account field and a note says to make him active first', async () => {
    const original = vi.mocked(vouchersApi.vouchersPreview).getMockImplementation();
    vi.mocked(vouchersApi.vouchersPreview).mockImplementation(async (entry, replacing) => {
      const real = await (original?.(entry, replacing) ?? Promise.reject(new Error('no fake')));
      return {
        ...real,
        errors: [{ code: 'ERR_ACCOUNT_INACTIVE', field: 'account_id', values: {} }],
      };
    });
    await renderApp('/bikri/voucher?type=receipt');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'vouchers-receipt-title'),
    });
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'vouchers-field-buyer')),
      }),
    );
    await user.keyboard('03040001');
    await screen.findByRole('option');
    await user.keyboard('{Enter}');
    await user.click(
      await screen.findByLabelText(named(message(language, 'vouchers-field-amount'))),
    );
    await user.keyboard('1000');
    // The message of the backend's code, in words, and the hint under the form.
    expect(
      (
        await screen.findAllByText(
          (content) => visible(content) === message(language, 'err-account-inactive'),
        )
      ).length,
    ).toBeGreaterThan(0);
    expect(
      await screen.findByText((content) =>
        visible(content).includes(message(language, 'vouchers-inactive-hint')),
      ),
    ).toBeInTheDocument();
    expect(document.body.textContent).not.toContain('ERR_');
  });
});

describe.each(LANGUAGES)(
  'a long list of open bills or payables is capped by the backend (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakeVouchers();
    });

    async function pickAccount(type: 'receipt' | 'payment', fieldId: string, code: string) {
      await renderApp(`/bikri/voucher?type=${type}`);
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, `vouchers-${type}-title`),
      });
      const user = userEvent.setup();
      await user.click(
        await screen.findByRole('combobox', { name: named(message(language, fieldId)) }),
      );
      await user.keyboard(code);
      await screen.findByRole('option');
      await user.keyboard('{Enter}');
      return user;
    }

    const note = (shown: string, total: string) =>
      message(language, 'vouchers-truncated-note', { shown, total });

    it('open bills: a one-line note says the oldest 200 of the total are shown, and "full balance" still fills the whole amount', async () => {
      const original = vi.mocked(vouchersApi.assistOpenBills).getMockImplementation();
      vi.mocked(vouchersApi.assistOpenBills).mockImplementation(async (input) => {
        const real = await (original?.(input) ?? Promise.reject(new Error('no fake')));
        const first = real.bills[0];
        if (first === undefined) return real;
        // 200 bills, as the backend caps them; the totals are for all 250.
        const bills = Array.from({ length: 200 }, (_, index) => ({
          ...first,
          line_id: first.line_id + index,
        }));
        return { ...real, bills, total_count: 250, truncated: true };
      });
      const user = await pickAccount('receipt', 'vouchers-field-buyer', '03040001');
      expect(
        await screen.findByText((content) => visible(content) === note('200', '250')),
      ).toBeInTheDocument();
      // The amount "full balance" fills is the open amount of ALL the bills, as the backend sent it.
      await user.click(
        await screen.findByRole('button', {
          name: (name) => visible(name).startsWith(message(language, 'vouchers-fill-full')),
        }),
      );
      expect(
        (
          await screen.findByLabelText<HTMLInputElement>(
            named(message(language, 'vouchers-field-amount')),
          )
        ).value,
      ).toBe('12,880');
    });

    it('open payables: the same note, and no note when the list is whole', async () => {
      const original = vi.mocked(vouchersApi.assistOpenPayables).getMockImplementation();
      vi.mocked(vouchersApi.assistOpenPayables).mockImplementation(async (input) => {
        const real = await (original?.(input) ?? Promise.reject(new Error('no fake')));
        const first = real.postings[0];
        if (first === undefined) return real;
        return {
          ...real,
          postings: Array.from({ length: 200 }, () => ({ ...first })),
          total_count: 312,
          truncated: true,
        };
      });
      await pickAccount('payment', 'vouchers-field-paid-to', '030110001');
      expect(
        await screen.findByText((content) => visible(content) === note('200', '312')),
      ).toBeInTheDocument();
    });

    it('a whole list has no note', async () => {
      await pickAccount('receipt', 'vouchers-field-buyer', '03040001');
      await screen.findByText(message(language, 'vouchers-bills-title'));
      await waitFor(() => {
        expect(document.body.textContent).toContain('12,880');
      });
      // The fake sends one bill and total_count 1: a whole list, so the note is not drawn.
      expect(visible(document.body.textContent)).not.toContain(note('1', '1'));
    });
  },
);

describe('making a one-time person a regular account reads the lists again', () => {
  it('every list that shows accounts, balances, bills or the one-time people is invalidated', async () => {
    const client = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
    const spy = vi.spyOn(client, 'invalidateQueries');
    vi.mocked(pukhtaApi.pukhtaMakeRegular).mockResolvedValue({
      account: { id: 24, code: '03040003', name_ur: 'شاہد', name_en: 'Shahid' },
      moved: { amount_paisa: 0 as never, side: 'zero' },
      entry_id: null,
    });
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useMakeRegular(), { wrapper });
    await act(async () => {
      await result.current.mutateAsync({
        person_key: 'shahid|03000000000',
        side: 'buyer',
        // The backend refuses any opening here; the form sends none.
        account_input: {
          opening_dr: '',
          opening_cr: '',
          opening_is_advance: false,
        } as never,
      });
    });
    await waitFor(() => {
      const keys = spy.mock.calls.map((call) => JSON.stringify(call[0]?.queryKey));
      for (const key of ['vouchers', 'accounts', 'coa', 'search', 'pukhta', 'bikri']) {
        expect(keys).toContain(JSON.stringify([key]));
      }
    });
  });
});

describe('the paths of the errors of a batch of receipts', () => {
  it('reads rows[i].field and rows[i] alone', () => {
    expect(rowPathOf('rows[2].amount')).toEqual({ index: 2, field: 'amount' });
    expect(rowPathOf('rows[10].account_id')).toEqual({ index: 10, field: 'account_id' });
    // An error of the whole row has no field: the backend sends the row alone.
    expect(rowPathOf('rows[3]')).toEqual({ index: 3, field: '' });
    expect(rowPathOf('amount')).toBeNull();
    expect(rowPathOf('rows[x].amount')).toBeNull();
  });
});
