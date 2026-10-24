// Robustness: the same save sent twice quickly gives exactly one posting (NFR-REL-1, the double
// click and the double Enter of the robustness list). Owner: `robustness`.
//
// The backend has no request number on a voucher (see robustness_fault.rs, the test about the same
// save twice), so two identical saves make two vouchers. The only guard is on the screen. This
// test presses the save key again and again while the first save is still on its way, and counts
// the calls that reach the backend. One call is a pass.
//
// A slow fake backend holds the first save open for 400 ms, as a busy disk would.

import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import { message, visible } from '../helpers/render';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { fakeAdvances } from '../helpers/vouchers';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const RAHIM = account(31, 'seller', '030110001', 'رحیم', 'Rahim', { balance: balance(4316, 'cr') });

const named = (text: string) => (name: string) => visible(name) === text;
const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  fakeAccounts([RAHIM]);
  fakeAccountSearch([RAHIM]);
  fakeMasters({});
  resetClientState('ur');
  signedIn(sessionUser({ language: 'ur' }));
});

afterEach(() => {
  leaveAppWindow();
});

describe('the advance screen when save is pressed again and again', () => {
  async function fillOneVoucher() {
    const store = fakeAdvances();
    // The first call takes 400 ms; every call is counted by the mock.
    const real = vi.mocked(vouchersApi.vouchersSave).getMockImplementation();
    vi.mocked(vouchersApi.vouchersSave).mockImplementation(async (draft) => {
      await wait(400);
      if (real === undefined) throw new Error('no fake save');
      return real(draft);
    });
    await renderApp('/bikri/advance');
    await screen.findByRole('heading', {
      level: 1,
      name: message('ur', 'vouchers-advance-title'),
    });
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', { name: named(message('ur', 'vouchers-field-seller')) }),
    );
    await user.keyboard('rahim');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    await user.click(
      await screen.findByLabelText<HTMLInputElement>(named(message('ur', 'vouchers-field-amount'))),
    );
    await user.keyboard('5000');
    return { store, user };
  }

  it('sends one save to the backend when F10 is pressed three times in a row', async () => {
    const { store } = await fillOneVoucher();
    press('F10');
    press('F10');
    press('F10');
    await wait(900);
    expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    expect(store.saved).toHaveLength(1);
  });

  it('sends one save when F10 and Ctrl+S are mixed, and when the button is clicked meanwhile', async () => {
    const { store, user } = await fillOneVoucher();
    press('F10');
    press('s', { ctrlKey: true });
    const button = screen.getByRole('button', {
      name: new RegExp(message('ur', 'layout-save-save')),
    });
    await user.click(button);
    press('F10');
    await wait(900);
    expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    expect(store.saved).toHaveLength(1);
  });

  it('sends one save when the key events arrive in the same instant, before the screen has redrawn', async () => {
    const { store } = await fillOneVoucher();
    // Three key events inside one act(): React cannot redraw between them, so a guard that only
    // reads the screen's state sees "not saving" three times. A hardware key repeat or a very
    // fast double Enter can do the same.
    act(() => {
      for (let n = 0; n < 3; n += 1) {
        fireEvent.keyDown(document.activeElement ?? document.body, { code: 'F10', key: 'F10' });
      }
    });
    await wait(900);
    expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    expect(store.saved).toHaveLength(1);
  });
});
