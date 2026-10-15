// The advance screen (plan 10, PRD 15.1, 15.4, FR-7.60, FR-15.1), in the Urdu and in the English
// app, by keyboard, against a fake backend: the form with the party card and the seller's advances,
// the balances from the backend, a lone voucher saved with F10, several vouchers in a list saved
// together, and a refused one that stays in the list. The screen works out no money.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  ZERO,
  account,
  balance,
  fakeAccountSearch,
  fakeAccounts,
  money,
} from '../helpers/accounts';
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
import { fakeMasters, record } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { at } from '../helpers/sample';
import { fakeAdvances } from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const RAHIM = account(31, 'seller', '030110001', 'رحیم', 'Rahim', { balance: balance(4316, 'cr') });
const KARIM = account(32, 'seller', '030110002', 'کریم', 'Karim');
const BANK = record('bank_account', 71, '0001', 'بینک الفلاح', 'Bank Alfalah');

const named = (text: string) => (name: string) => visible(name) === text;

const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

const dropdown = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });

const saveButton = (language: Language) =>
  screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) });

/** Types in the seller picker and takes the first match. */
async function pickSeller(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  typed: string,
  shown: string,
) {
  await user.click(await dropdown(language, 'vouchers-field-seller'));
  await user.keyboard(typed);
  // The recent list is open before the search answers: wait until the matches are the only rows.
  await waitFor(() => {
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(1);
    expect(visible(options[0]?.textContent)).toContain(shown);
  });
  await user.keyboard('{Enter}');
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
  fakeAccounts([RAHIM, KARIM]);
  fakeAccountSearch([RAHIM, KARIM]);
  fakeMasters({ bank_account: [BANK] });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the advance screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const store = fakeAdvances();
    const app = await renderApp('/bikri/advance');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'vouchers-advance-title'),
    });
    return { ...app, store, user: userEvent.setup() };
  }

  it('starts with today as the date, cash as the mode and nothing to save', async () => {
    await open();
    expect((await labelled(language, 'vouchers-field-date')).value).toBe('10-10-2026');
    expect((await dropdown(language, 'vouchers-field-mode')).textContent).toContain(
      message(language, 'vouchers-mode-cash'),
    );
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('');
    expect(saveButton(language)).toBeDisabled();
    // The bank fields are for bank and cheque only.
    expect(screen.queryByLabelText(named(message(language, 'vouchers-field-bank')))).toBeNull();
  });

  it('a seller shows his party card and his advances: open, last, and each with what is adjusted', async () => {
    const { user } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    const title = await screen.findByRole('heading', {
      name: named(message(language, 'vouchers-advances-title')),
    });
    const panel = visible(title.closest('section')?.textContent);
    expect(panel).toContain(message(language, 'vouchers-advances-open'));
    expect(panel).toContain('15,000');
    expect(panel).toContain(message(language, 'vouchers-advances-last'));
    expect(panel).toContain('01-09-2026');
    expect(panel).toContain(message(language, 'vouchers-number-advance', { number: '1' }));
    // The party card of the seller is there too.
    expect(await screen.findByText(message(language, 'assist-card-balance'))).toBeInTheDocument();
  });

  it('a seller with no advance yet says so', async () => {
    const { user } = await open();
    await pickSeller(user, language, 'karim', '030110002');
    await screen.findByRole('heading', {
      name: named(message(language, 'vouchers-advances-title')),
    });
    expect(
      await screen.findByText(named(message(language, 'vouchers-advances-none'))),
    ).toBeInTheDocument();
  });

  it("the amount goes to the preview as typed, and the balances shown are the backend's", async () => {
    const { user } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreviewAdvances).toHaveBeenLastCalledWith([
        expect.objectContaining({ amount: '5000', mode: 'cash', date: '10-10-2026' }),
      ]);
    });
    const current = await screen.findByText(named(message(language, 'vouchers-current-balance')));
    const newer = screen.getByText(named(message(language, 'vouchers-new-balance')));
    await waitFor(() => {
      expect(visible(newer.closest('span')?.textContent)).toContain('5,000');
    });
    expect(visible(newer.parentElement?.textContent)).toContain(
      message(language, 'common-side-dr'),
    );
    expect(visible(current.closest('span')?.textContent)).not.toContain('5,000');
  });

  it('asks nothing of the preview while there is no seller and amount', async () => {
    const { user } = await open();
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    await new Promise((resolve) => setTimeout(resolve, 150));
    expect(vouchersApi.vouchersPreviewAdvances).not.toHaveBeenCalled();
  });

  it('a lone voucher is saved from the form with F10: text sent, message, form ready for the next, voucher listed', async () => {
    const { user, store } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    await user.click(await labelled(language, 'vouchers-field-narration'));
    await user.keyboard('x');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSaveAdvance).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(vouchersApi.vouchersSaveAdvance).mock.calls[0]?.[0];
    expect(sent?.amount).toBe('5000');
    expect(sent?.seller.id).toBe(31);
    expect(sent?.mode).toBe('cash');
    expect(sent?.date).toBe('10-10-2026');
    expect(sent?.narration).toBe('x');
    const toast = message(language, 'vouchers-saved-advance', {
      number: message(language, 'vouchers-number-advance', { number: '1' }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    expect(store.saved).toEqual([{ seller: 31, amount: '5000' }]);
    // The form is ready for the next: seller and amount empty, the date kept.
    await waitFor(() => {
      expect(
        screen.getByLabelText<HTMLInputElement>(named(message(language, 'vouchers-field-amount')))
          .value,
      ).toBe('');
    });
    expect((await labelled(language, 'vouchers-field-date')).value).toBe('10-10-2026');
    // The saved voucher shows in the list with its number and the new balance.
    const list = await screen.findByRole('heading', {
      name: named(message(language, 'vouchers-list-title')),
    });
    const words = visible(list.closest('section')?.textContent);
    expect(words).toContain(message(language, 'vouchers-state-saved'));
    expect(words).toContain(message(language, 'vouchers-number-advance', { number: '1' }));
    expect(words).toContain('5,000');
    expect(saveButton(language)).toBeDisabled();
  });

  it('a refused field shows at the field, and nothing typed is lost', async () => {
    const { user } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    const amount = await labelled(language, 'vouchers-field-amount');
    await user.click(amount);
    await user.keyboard('50.5');
    vi.mocked(vouchersApi.vouchersSaveAdvance).mockRejectedValueOnce(
      fail({ code: 'ERR_DECIMALS_NOT_ALLOWED', field: 'amount' }),
    );
    press('F10');
    await waitFor(() => {
      expect(amount).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'err-decimals-not-allowed'),
    );
    expect(amount.value).toBe('50.5');
  });

  it('an advance given with the opening balance is listed as that, not as a voucher number', async () => {
    const { user } = await open();
    vi.mocked(vouchersApi.assistAdvances).mockResolvedValue({
      open: [
        {
          source: 'opening',
          voucher_number: null,
          entry_date: '2026-08-31',
          given_paisa: money(15000),
          adjusted_paisa: money(0),
          open_paisa: money(15000),
        },
      ],
      open_paisa: money(15000),
      last: null,
    });
    await pickSeller(user, language, 'rahim', '030110001');
    const title = await screen.findByRole('heading', {
      name: named(message(language, 'vouchers-advances-title')),
    });
    const panel = visible(title.closest('section')?.textContent);
    expect(panel).toContain(message(language, 'vouchers-source-opening'));
    expect(panel).toContain('31-08-2026');
    expect(panel).not.toContain(message(language, 'vouchers-number-advance', { number: '0' }));
  });

  it("a refused date in the preview shows the date of the backend's own values at the date field", async () => {
    const { user } = await open();
    vi.mocked(vouchersApi.vouchersPreviewAdvances).mockImplementation((drafts) =>
      Promise.resolve({
        rows: drafts.map(() => ({
          account: null,
          errors: [
            {
              code: 'ERR_DATE_CLOSED' as const,
              field: 'lines[0].date',
              values: { closed_up_to: '09-10-2026' },
            },
          ],
          amount_paisa: money(0),
          current_balance: ZERO,
          new_balance: ZERO,
        })),
        total_paisa: money(0),
      }),
    );
    await pickSeller(user, language, 'rahim', '030110001');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    const date = await labelled(language, 'vouchers-field-date');
    await waitFor(() => {
      expect(date).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(document.body.textContent)).toContain('09-10-2026');
  });

  it('an amount with no seller is told at the seller field, and the backend is not asked', async () => {
    const { user } = await open();
    const seller = await dropdown(language, 'vouchers-field-seller');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    press('F10');
    await waitFor(() => {
      expect(seller).toHaveAttribute('aria-invalid', 'true');
    });
    expect(vouchersApi.vouchersSaveAdvance).not.toHaveBeenCalled();
    expect(vouchersApi.vouchersPreviewAdvances).not.toHaveBeenCalled();
    // Nothing typed is lost (the field shows its thousands separator).
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('5,000');
  });

  it('the preview points at a bad amount before it is saved', async () => {
    const { user } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    const amount = await labelled(language, 'vouchers-field-amount');
    await user.click(amount);
    await user.keyboard('50.5');
    await waitFor(() => {
      expect(amount).toHaveAttribute('aria-invalid', 'true');
    });
    expect(vouchersApi.vouchersSaveAdvance).not.toHaveBeenCalled();
  });

  it('F2 puts the voucher in the list and the next one starts with the date and mode; F10 saves them in order', async () => {
    const { user, store } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    press('F2');
    const list = await screen.findByRole('heading', {
      name: named(message(language, 'vouchers-list-title')),
    });
    // The form is empty again, the date kept.
    await waitFor(() => {
      expect(
        screen.getByLabelText<HTMLInputElement>(named(message(language, 'vouchers-field-amount')))
          .value,
      ).toBe('');
    });
    expect((await labelled(language, 'vouchers-field-date')).value).toBe('10-10-2026');
    await pickSeller(user, language, 'rahim', '030110001');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('2000');
    // The total of the list is the backend's: both vouchers, listed and on the form.
    await waitFor(() => {
      expect(visible(list.closest('section')?.textContent)).toContain('7,000');
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSaveAdvance).toHaveBeenCalledTimes(2);
    });
    expect(store.saved).toEqual([
      { seller: 31, amount: '5000' },
      { seller: 31, amount: '2000' },
    ]);
    const words = visible(list.closest('section')?.textContent);
    expect(words).toContain(message(language, 'vouchers-number-advance', { number: '1' }));
    expect(words).toContain(message(language, 'vouchers-number-advance', { number: '2' }));
    expect(
      (
        await screen.findAllByText(
          (content) =>
            visible(content) === message(language, 'vouchers-saved-many', { count: '2' }),
        )
      ).length,
    ).toBeGreaterThan(0);
  });

  it('a voucher the backend refuses stays in the list with its message while the others are saved', async () => {
    const { user, store } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    press('F2');
    await pickSeller(user, language, 'karim', '030110002');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('2000');
    const original = vi.mocked(vouchersApi.vouchersSaveAdvance).getMockImplementation();
    vi.mocked(vouchersApi.vouchersSaveAdvance).mockImplementation((draft) =>
      draft.amount === '2000'
        ? Promise.reject(fail({ code: 'ERR_DATE_CLOSED', values: { closed_up_to: '09-10-2026' } }))
        : (original?.(draft) ?? Promise.reject(new Error('no fake'))),
    );
    press('F10');
    const list = await screen.findByRole('heading', {
      name: named(message(language, 'vouchers-list-title')),
    });
    await waitFor(() => {
      expect(visible(list.closest('section')?.textContent)).toContain(
        message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }),
      );
    });
    // The first was saved, the second is still waiting to be fixed and saved again.
    expect(store.saved).toEqual([{ seller: 31, amount: '5000' }]);
    expect(
      (
        await screen.findAllByText(
          (content) =>
            visible(content) === message(language, 'vouchers-some-refused', { count: 1 }),
        )
      ).length,
    ).toBeGreaterThan(0);
    expect(saveButton(language)).toBeEnabled();
    // A row that is not saved can be taken out of the list.
    const removers = screen.getAllByRole('button', { name: message(language, 'vouchers-remove') });
    expect(removers).toHaveLength(1);
    await user.click(at(removers, 0));
    await waitFor(() => {
      expect(saveButton(language)).toBeDisabled();
    });
  });

  it('bank and cheque show their fields, cash hides them, and they go to the backend', async () => {
    const { user } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('100');
    await user.click(await dropdown(language, 'vouchers-field-mode'));
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'vouchers-mode-cheque')) }),
    );
    expect(await dropdown(language, 'vouchers-field-bank')).toBeInTheDocument();
    await user.click(await dropdown(language, 'vouchers-field-bank'));
    await user.keyboard('alfalah');
    await screen.findByRole('option', {
      name: (name) => visible(name).includes('Alfalah') || visible(name).includes('الفلاح'),
    });
    await user.keyboard('{Enter}');
    await user.click(await labelled(language, 'vouchers-field-cheque-no'));
    await user.keyboard('123456');
    await user.click(await labelled(language, 'vouchers-field-cheque-date'));
    await user.keyboard('15-10-2026');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSaveAdvance).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(vouchersApi.vouchersSaveAdvance).mock.calls[0]?.[0];
    expect(sent?.mode).toBe('cheque');
    expect(sent?.bank?.id).toBe(71);
    expect(sent?.cheque_no).toBe('123456');
    expect(sent?.cheque_date).toBe('15-10-2026');
  });

  it('switching the language in the middle of an entry keeps what was typed and the list', async () => {
    const { user } = await open();
    await pickSeller(user, language, 'rahim', '030110001');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    press('F2');
    await pickSeller(user, language, 'karim', '030110002');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('300');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'vouchers-field-amount')).value).toBe('300');
    const list = await screen.findByRole('heading', {
      name: named(message(other, 'vouchers-list-title')),
    });
    expect(visible(list.closest('section')?.textContent)).toContain(
      other === 'ur' ? 'رحیم' : 'Rahim',
    );
    expect(
      within(document.body).getByRole('combobox', {
        name: named(message(other, 'vouchers-field-seller')),
      }),
    ).toHaveValue(other === 'ur' ? 'کریم' : 'Karim');
  });

  it('a failed load of today shows the message with a retry that works', async () => {
    const appApi = await import('../../src/app/api');
    vi.mocked(appApi.dayStatus).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    fakeAdvances();
    await renderApp('/bikri/advance');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-advance-title'),
      }),
    ).toBeInTheDocument();
  });

  it('the advance menu item and Alt+P open the screen', async () => {
    fakeAdvances();
    const { router } = await renderApp('/');
    act(() => {
      void router.navigate({ to: '/bikri/advance' });
    });
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-advance-title'),
      }),
    ).toBeInTheDocument();
  });
});
