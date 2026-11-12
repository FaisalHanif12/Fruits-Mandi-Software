// Reopen the day and fix, on the voucher screens (PRD FR-21.4): the date the box asks the backend to reopen.
// `day_fix_open` reads the date as typed, DD-MM-YYYY, and refused the other with ERR_DATE_INVALID (Umar,
// 13-11-2026). Two screens start the flow for a voucher of a closed day: the voucher opened by its address
// (VoucherPage, ClosedEntryActions) and a row of the list under the form (VoucherEditor). Both ask the shared
// flow with the day as the backend sent it, YYYY-MM-DD, and the shared flow writes it as typed.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
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
import { backendReopens, confirmFixDay } from '../helpers/fix-day';
import { fakeMasters } from '../helpers/masters';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import {
  AKRAM,
  ARSHAD,
  CHUKTA_KHATA,
  KARIM,
  RAHIM,
  fakeVouchers,
  voucherRow,
} from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
  const rows = [AKRAM, ARSHAD, CHUKTA_KHATA, RAHIM, KARIM];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({});
  backendReopens();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('a voucher of a closed day, reopen and fix (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    const store = fakeVouchers();
    // One of an earlier day, opened by its address; one of today (the list shows today's), closed too.
    store.rows.push(
      voucherRow({ id: 504, number: 4, needs_reopen: true, entry_date: '2026-10-05' }),
      voucherRow({ id: 505, number: 5, needs_reopen: true, entry_date: '2026-10-10' }),
    );
  });

  const fixButton = () =>
    screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'edit-action-fix-day')),
    });

  it('the voucher opened by its address: the box reopens its day as typed, DD-MM-YYYY', async () => {
    await renderApp('/bikri/voucher?type=receipt&id=504');
    const user = userEvent.setup();
    await user.click(await fixButton());
    await confirmFixDay(language, user, '05-10-2026');
  });

  it('a row of the list under the form: the box reopens its day as typed, DD-MM-YYYY', async () => {
    await renderApp('/bikri/voucher?type=receipt');
    const user = userEvent.setup();
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="505"]')).not.toBeNull();
    });
    const number = document.querySelector<HTMLElement>('.ag-row[row-id="505"] [col-id="number"]');
    if (number === null) throw new Error('the row has no number cell');
    await user.click(number);
    await user.click(await fixButton());
    await confirmFixDay(language, user, '10-10-2026');
  });
});
