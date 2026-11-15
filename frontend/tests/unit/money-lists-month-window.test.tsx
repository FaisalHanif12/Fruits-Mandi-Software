// The lists under the ادھار رقم, ذاتی خرچہ (partner takes) and سرمایہ forms show ONE MONTH at a time (plan 76
// point 8, PRD FR-19.11, NFR-SPD-24), in the Urdu and the English app: each list asks the backend for the first
// and the last day of the month on screen (today is 10-10-2026, so 01-10-2026 to 31-10-2026), the month
// buttons move it, and nothing before it is read. The rows, counts and totals stay the backend's.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as windowsApi from '../../src/shared/windows/api';
import { choose, pickAccount, prepareCashbook } from '../helpers/cashbook-screen';
import { ASLAM_ROW } from '../helpers/cashbook';
import { leaveAppWindow, renderApp } from '../helpers/app';
import { preparePartnerMoney } from '../helpers/partner-money';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

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
vi.mock('../../src/shared/windows/api');

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(windowsApi.windowsBounds).mockResolvedValue({
    first_date: '2026-08-15',
    last_date: '2026-10-10',
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the lists of money, one month at a time (%s)', (language) => {
  const m = (id: string) => message(language, id);
  const earlier = () =>
    screen.getByRole('button', {
      name: (name) => visible(name).includes(m('common-month-earlier')),
    });

  beforeEach(() => {
    prepareCashbook(language, [ASLAM_ROW]);
  });

  it('the loans of a person are asked for the month of today, and the earlier button moves to September', async () => {
    await renderApp('/rokar/lent?type=money_lent');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-lent-screen-title') });
    const user = userEvent.setup();
    await pickAccount(
      user,
      language,
      'cashbook-field-person',
      'aslam',
      nameOf(language, 'اسلم', 'Aslam'),
    );
    await waitFor(() => {
      expect(
        vi
          .mocked(cashbookApi.lentList)
          .mock.calls.some(([input]) => input.from === '01-10-2026' && input.to === '31-10-2026'),
      ).toBe(true);
    });
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('common-month-earlier')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(cashbookApi.lentList).mock.calls.at(-1)?.[0]).toMatchObject({
        from: '01-09-2026',
        to: '30-09-2026',
      });
    });
    // It never asks for the whole history.
    for (const [input] of vi.mocked(cashbookApi.lentList).mock.calls) {
      if (input.limit !== 1) expect(input.from).not.toBeNull();
    }
    expect(earlier()).toBeInTheDocument();
  });

  it('the money a partner took is asked for the month of today', async () => {
    await renderApp('/rokar/expense?type=personal_expense');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-expense-title') });
    const user = userEvent.setup();
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    await waitFor(() => {
      expect(vi.mocked(cashbookApi.partnerTakes).mock.calls.at(-1)?.[0]).toMatchObject({
        partner_id: 1,
        from: '01-10-2026',
        to: '31-10-2026',
      });
    });
  });

  it('the money a partner took, while the partner section is locked, is the code box and not an error', async () => {
    vi.mocked(cashbookApi.partnerTakes).mockRejectedValue(
      new CommandError(refusal([{ code: 'ERR_PARTNER_LOCKED' }])),
    );
    await renderApp('/rokar/expense?type=personal_expense');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-expense-title') });
    const user = userEvent.setup();
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    expect(await screen.findByText(m('partners-code-box-text'))).toBeInTheDocument();
    // No error with a retry, and no month window of figures that are hidden.
    expect(screen.queryByText(m('cashbook-takes-failed'))).toBeNull();
    expect(
      screen.queryByRole('button', {
        name: (name) => visible(name).includes(m('common-month-earlier')),
      }),
    ).toBeNull();
  });

  it('the capital history of a partner is asked for the month of today', async () => {
    preparePartnerMoney(language);
    await renderApp('/partners/capital?type=capital_in');
    await screen.findByRole('heading', { level: 1, name: m('partners-capital-screen-title') });
    const user = userEvent.setup();
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'ب', 'Partner B'));
    await waitFor(() => {
      expect(vi.mocked(partnersApi.capitalHistory).mock.calls.at(-1)?.[0]).toMatchObject({
        partner_id: 2,
        from: '01-10-2026',
        to: '31-10-2026',
      });
    });
  });
});
