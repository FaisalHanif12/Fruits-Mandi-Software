// The tick "show returned too" of the lists under the ادھار رقم and ذاتی خرچہ forms (plan 38 G6): a
// loan or a take that is fully returned is left out by the backend until the tick is on, and the
// month rows and the sort come from the backend too. Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES } from '../../src/i18n';
import { named, pickAccount, prepareCashbook } from '../helpers/cashbook-screen';
import { ASLAM_ROW } from '../helpers/cashbook';
import { leaveAppWindow, renderApp } from '../helpers/app';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const lastLent = () => vi.mocked(cashbookApi.lentList).mock.calls.at(-1)?.[0];

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the tick "show returned too" (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language, [ASLAM_ROW]);
  });

  it('the loans of a person leave out the ones fully returned until the tick is on', async () => {
    await renderApp('/rokar/lent?type=money_lent');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-lent-screen-title'),
    });
    const user = userEvent.setup();
    await pickAccount(
      user,
      language,
      'cashbook-field-person',
      'aslam',
      language === 'ur' ? 'اسلم' : 'Aslam',
    );
    await waitFor(() => {
      expect(lastLent()).toMatchObject({ show_finished: false });
    });
    const tick = await screen.findByRole('checkbox', {
      name: named(message(language, 'cashbook-show-returned')),
    });
    expect(tick).not.toBeChecked();
    await user.click(tick);
    await waitFor(() => {
      expect(lastLent()).toMatchObject({ show_finished: true });
    });
  });

  it('the money taken by a partner leaves out the takes fully returned until the tick is on', async () => {
    await renderApp('/rokar/expense?type=personal_expense');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-expense-title'),
    });
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'cashbook-field-partner')),
      }),
    );
    await user.click((await screen.findAllByRole('option'))[0] ?? document.body);
    const tick = await screen.findByRole('checkbox', {
      name: named(message(language, 'cashbook-show-returned')),
    });
    await waitFor(() => {
      expect(vi.mocked(cashbookApi.partnerTakes).mock.calls.at(-1)?.[0]).toMatchObject({
        show_finished: false,
      });
    });
    await user.click(tick);
    await waitFor(() => {
      expect(vi.mocked(cashbookApi.partnerTakes).mock.calls.at(-1)?.[0]).toMatchObject({
        show_finished: true,
      });
    });
  });
});
