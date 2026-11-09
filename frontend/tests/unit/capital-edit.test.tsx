// A saved capital entry open for edit (plan 44, PRD FR-7.70, FR-17.7), in the Urdu and the English
// app, against a fake backend: the سرمایہ screen opens it with `?id=` filled as it was saved, the
// type list shut, the amber band, and the save goes to `vouchers_edit`. A closed day shows the
// entry as stored with the button that reopens the day first.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { dropdown, labelled, named, startsWith } from '../helpers/cashbook-screen';
import { leaveAppWindow, press, renderApp } from '../helpers/app';
import { noChangeReview } from '../helpers/edit-real';
import { CLOSED_DAY_ENTRY } from '../helpers/entry-flags';
import { paisaForTest } from '../helpers/paisa';
import { preparePartnerMoney } from '../helpers/partner-money';
import { message, visible } from '../helpers/render';
import { voucherRow } from '../helpers/voucher-books';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.clearAllMocks();
});
afterEach(() => {
  leaveAppWindow();
});

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

/** 50,000 put in by partner B as saved. */
function capitalIn(overrides: Parameters<typeof voucherRow>[0] = {}) {
  return {
    row: voucherRow({
      id: 95,
      number: 1,
      voucher_type: 'capital_in',
      account: null,
      amount_paisa: paisaForTest(5_000_000),
      ...overrides,
    }),
    entry: {
      type: 'capital_in' as const,
      date: '10-10-2026',
      partner_id: 2,
      amount: '50000',
      narration: 'new year',
      confirm_cash_below_zero: false,
    },
  };
}

describe.each(LANGUAGES)('a capital entry open for edit (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(noChangeReview());
  });

  async function open(id: number) {
    const app = await renderApp(`/partners/capital?id=${String(id)}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'partners-capital-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('fills the form as it was saved, shows the band and shuts the type list', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(capitalIn());
    await open(95);
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('50,000');
    expect((await labelled(language, 'cashbook-field-narration')).value).toBe('new year');
    expect(await dropdown(language, 'cashbook-field-partner')).toHaveTextContent(
      nameOf(language, 'ب', 'Partner B'),
    );
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-band', {
        what: message(language, 'glossary-sarmaya'),
        number: '1',
        date: '10-10-2026',
      }),
    );
    for (const button of screen
      .getByRole('group', { name: named(message(language, 'partners-capital-type')) })
      .querySelectorAll('button')) {
      expect(button).toBeDisabled();
    }
  });

  it('saves with vouchers_edit: the id, the version and the entry as typed', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(capitalIn());
    vi.mocked(vouchersApi.vouchersEdit).mockResolvedValue({
      voucher_id: 95,
      number: 1,
      version: 2,
      entry_id: 9,
      cash_after: paisaForTest(0),
    } as never);
    const { user, router } = await open(95);
    const amount = await labelled(language, 'cashbook-field-amount');
    await user.clear(amount);
    await user.type(amount, '60000');
    await waitFor(() => {
      expect(vouchersApi.vouchersEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersEdit).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersEdit).mock.calls[0]?.[0]).toMatchObject({
      id: 95,
      version: 1,
      reason: '',
      entry: { type: 'capital_in', amount: '60000', partner_id: 2 },
    });
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('id');
    });
  });

  it('a closed day shows the entry as stored and a button that reopens the day first', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      capitalIn({ ...CLOSED_DAY_ENTRY, entry_date: '2026-10-05' }),
    );
    await renderApp('/partners/capital?id=95');
    expect(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'edit-action-fix-day')),
      }),
    ).toBeEnabled();
    expect(screen.queryByLabelText(named(message(language, 'cashbook-field-amount')))).toBeNull();
  });

  it('an expense opened on the capital address moves to its own screen', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue({
      row: voucherRow({ id: 77, number: 3, voucher_type: 'shop_expense', account: null }),
      entry: {
        type: 'shop_expense',
        date: '10-10-2026',
        expense_head_id: 1,
        amount: '3000',
        paid_to: null,
        narration: null,
        confirm_cash_below_zero: false,
      },
    });
    const { router } = await renderApp('/partners/capital?id=77');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/expense');
    });
  });
});
