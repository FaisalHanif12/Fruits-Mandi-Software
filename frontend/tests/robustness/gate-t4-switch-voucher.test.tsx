// Gate lane T4 (plan 80): a language switch in the middle of a voucher loses nothing (Code
// Standards F39, PRD FR-7.33, FR-7.67, E32). Owner: `robustness`.
//
// `vouchers.test.tsx` switches once and looks at the fields. This test switches the way a user
// presses `Alt+L` by mistake, again and again, and then SAVES: what the backend is sent must be the
// entry typed before the first switch (the account, the amount as typed, the narration typed in
// Urdu letters, the type), and it must be sent once. The screen works out no money.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters, record } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import {
  AKRAM,
  ARSHAD,
  CHUKTA_KHATA,
  KARIM,
  RAHIM,
  TRANSPORT,
  fakeVouchers,
} from '../helpers/vouchers';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));
const dropdown = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });
const other = (language: Language): Language => (language === 'ur' ? 'en' : 'ur');
const switchTo = (language: Language) => {
  act(() => {
    useLanguageStore.setState({ language });
  });
};

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  const rows = [AKRAM, ARSHAD, CHUKTA_KHATA, RAHIM, KARIM, TRANSPORT];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({ staff: [record('staff', 81, '0001', 'اسلم', 'Aslam')] });
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('a language switch in the middle of a voucher (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeVouchers();
  });

  async function open(type: 'receipt' | 'payment' | 'advance') {
    const app = await renderApp(`/bikri/voucher?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, `vouchers-${type}-title`),
    });
    return { ...app, user: userEvent.setup() };
  }

  async function pick(
    user: ReturnType<typeof userEvent.setup>,
    fieldId: string,
    typed: string,
    shown: string,
  ) {
    await user.click(await dropdown(language, fieldId));
    await user.keyboard(typed);
    await waitFor(() => {
      const options = screen.getAllByRole('option');
      expect(options).toHaveLength(1);
      expect(visible(options[0]?.textContent)).toContain(shown);
    });
    await user.keyboard('{Enter}');
  }

  it('four switches in a row keep the type, the account, the amount and the Urdu narration, and F10 sends them once', async () => {
    const { user } = await open('payment');
    await pick(user, 'vouchers-field-paid-to', 'rahim', '0010');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('300');
    await user.click(await labelled(language, 'vouchers-field-narration'));
    await user.keyboard('ٹھیک');
    // The Alt+L key and the store both switch: the same four times, ending where they started.
    for (let round = 0; round < 2; round += 1) {
      press('KeyL', { altKey: true });
      await waitFor(() => {
        expect(useLanguageStore.getState().language).toBe(other(language));
      });
      switchTo(language);
      await waitFor(() => {
        expect(useLanguageStore.getState().language).toBe(language);
      });
    }
    // Back in the first language, and once more to the other: both show the same typed values.
    for (const shown of [language, other(language)]) {
      switchTo(shown);
      expect((await labelled(shown, 'vouchers-field-amount')).value).toBe('300');
      expect((await labelled(shown, 'vouchers-field-narration')).value).toBe('ٹھیک');
      expect(await dropdown(shown, 'vouchers-field-paid-to')).toHaveValue(
        shown === 'ur' ? 'رحیم' : 'Rahim',
      );
    }
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0];
    expect(sent?.type).toBe('payment');
    expect(sent?.input).toMatchObject({
      account_id: RAHIM.id,
      amount: '300',
      narration: 'ٹھیک',
    });
  });

  it('one switch keeps the account, the amount as a formatted number and the date of a receipt', async () => {
    const { user } = await open('receipt');
    await pick(user, 'vouchers-field-buyer', '0', '0020');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('2500');
    switchTo(other(language));
    expect((await labelled(other(language), 'vouchers-field-amount')).value).toBe('2,500');
    expect((await labelled(other(language), 'vouchers-field-date')).value).toBe('10-10-2026');
    expect(await dropdown(other(language), 'vouchers-field-buyer')).toHaveValue(
      other(language) === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    );
  });
});
