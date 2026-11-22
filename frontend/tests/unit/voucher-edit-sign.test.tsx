// Plan 92 (Umar's report of 22-11-2026, PRD FR-15.3 v1.55, FR-7.71, FR-21.4): saving a changed وصولی, ادائیگی or
// پیشگی asks the reason and the Owner's sign (دستخط) in a box, on any day. Both are required and told under their
// fields; a wrong sign is told under the sign and saves nothing; the right sign saves with the reason and the
// sign; Esc closes the box with nothing saved and the change kept; the cash question still comes, after the box;
// and a refusal is never silent (the reason of the change was refused at the field `reason`, which the screen
// listed as shown although a payment has no such field, so F10 did nothing). Keyboard only path included. In the
// Urdu and in the English app, against a fake backend that refuses like the real one.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as reportsApi from '../../src/features/reports/api';
import { isCashLent, newDraft, shownFieldPaths } from '../../src/features/vouchers/draft';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
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
import { noteOf, openSignBox, saveChange, signBoxFields } from '../helpers/edit-sign-box';
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
  type FakeVouchers,
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

const named = (text: string) => (name: string) => visible(name) === text;
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

/** The payment of ادائیگی 1: 10,000 to Rahim, a seller. Its id is 601. */
function paymentRow() {
  return voucherRow({
    id: 601,
    voucher_type: 'payment',
    number: 1,
    account: { id: RAHIM.id, code: RAHIM.code, name_ur: RAHIM.name_ur, name_en: RAHIM.name_en },
    account_type: 'seller',
    // Not today: this is the voucher of Umar's screenshots, the one that did not save.
    entry_date: '2026-10-05',
  });
}

let store: FakeVouchers;

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
});

afterEach(() => {
  leaveAppWindow();
});

describe('the fields the form really draws', () => {
  it('lists the reason only for cash lent: a payment to a seller has no reason field, so its refusal must show in the notice', () => {
    const payment = newDraft('payment', '2026-10-10');
    expect(shownFieldPaths(payment)).not.toContain('reason');
    expect(shownFieldPaths({ ...payment, accountType: 'buyer' })).toContain('reason');
    expect(isCashLent({ ...payment, accountType: 'buyer' })).toBe(true);
    expect(shownFieldPaths(newDraft('receipt', '2026-10-10'))).toContain('collector_staff_id');
    expect(shownFieldPaths(newDraft('advance', '2026-10-10'))).toContain('seller_account_id');
  });
});

describe.each(LANGUAGES)(
  'saving a changed voucher asks the reason and the sign (%s)',
  (language) => {
    const m = (id: string, vars: Record<string, string | number> = {}) =>
      message(language, id, vars);

    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      store = fakeVouchers();
      store.rows.push(paymentRow());
    });

    /** Opens ادائیگی 1 for a change and puts 9,000 in the amount: the edit is ready to save. */
    async function openChanged(amount = '9000') {
      await renderApp('/bikri/voucher?type=payment&id=601');
      const field = await labelled(language, 'vouchers-field-amount');
      const user = userEvent.setup();
      await user.click(field);
      await user.keyboard(`{Control>}a{/Control}${amount}`);
      return { user, field };
    }

    const saveButton = () =>
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(m('layout-save-save')),
      });

    it('F10 opens a box with the reason and the sign, the cursor in the reason, and sends nothing yet', async () => {
      await openChanged();
      const dialog = await openSignBox(language);
      expect(visible(dialog.textContent)).toContain(m('vouchers-edit-sign-title'));
      expect(visible(dialog.textContent)).toContain(m('vouchers-edit-sign-hint'));
      // The inline field of the form is gone: the reason is asked here.
      expect(screen.queryAllByLabelText(named(m('vouchers-field-edit-reason')))).toHaveLength(1);
      expect(store.editTries).toHaveLength(0);
    });

    it('the Save button asks the same box', async () => {
      const { user } = await openChanged();
      await user.click(saveButton());
      expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
      expect(store.editTries).toHaveLength(0);
    });

    it('an empty reason and an empty sign are told in the box, and nothing is sent', async () => {
      const { user } = await openChanged();
      const dialog = await openSignBox(language);
      await user.click(
        within(dialog).getByRole('button', {
          // The button shows its key (F10) after the words.
          name: (name) => visible(name).startsWith(m('edit-reason-confirm')),
        }),
      );
      expect(within(dialog).getAllByText(named(m('err-required')))).toHaveLength(2);
      expect(store.editTries).toHaveLength(0);
    });

    it('a reason with no sign: only the sign is told, and nothing is sent', async () => {
      const { user } = await openChanged();
      const dialog = await openSignBox(language);
      await user.keyboard('wrong amount{Enter}{Enter}');
      expect(await within(dialog).findAllByText(named(m('err-required')))).toHaveLength(1);
      expect(store.editTries).toHaveLength(0);
    });

    it('a wrong sign (munshi) is told under the sign, nothing is saved, the box stays and keeps what was typed', async () => {
      await openChanged();
      const user = userEvent.setup();
      const dialog = await saveChange(user, language, 'wrong amount', 'munshi');
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(m('err-sign-wrong'));
      });
      // Under the sign field itself, not only in a notice.
      const { reason, sign } = signBoxFields(language, dialog);
      expect(noteOf(sign)).toContain(m('err-sign-wrong'));
      expect(store.editTries).toHaveLength(1);
      expect(store.edits).toHaveLength(0);
      expect(screen.getByRole('alertdialog')).toBeInTheDocument();
      expect((reason as HTMLInputElement).value).toBe('wrong amount');
      expect((sign as HTMLInputElement).value).toBe('munshi');
      // The voucher still holds the change that was typed.
      expect((await labelled(language, 'vouchers-field-amount')).value).toBe('9,000');
    });

    it('after a wrong sign the right one saves: the edit carries the reason and the sign, and the box closes', async () => {
      await openChanged();
      const user = userEvent.setup();
      const dialog = await saveChange(user, language, 'wrong amount', 'munshi');
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(m('err-sign-wrong'));
      });
      const { sign } = signBoxFields(language, dialog);
      await user.clear(sign);
      await user.type(sign, 'adnan{Enter}');
      await waitFor(() => {
        expect(store.edits).toHaveLength(1);
      });
      expect(store.edits[0]).toMatchObject({
        id: 601,
        version: 1,
        reason: 'wrong amount',
        sign: 'adnan',
        entry: { type: 'payment', amount: '9000' },
      });
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      const toast = m('vouchers-edited', {
        kind: 'payment',
        number: m('vouchers-number-payment', { number: 1 }),
      });
      expect(
        (await screen.findAllByText((content) => visible(content) === toast)).length,
      ).toBeGreaterThan(0);
      // The entry shows its stamp with the reason and the sign (PRD FR-7.71).
      await waitFor(() => {
        expect(document.querySelector('[data-edit-stamp]')).not.toBeNull();
      });
      const stamp = visible(document.querySelector('[data-edit-stamp]')?.textContent);
      expect(stamp).toContain('wrong amount');
      expect(stamp).toContain(`${m('glossary-dastakhat')}: adnan`);
    });

    it('keyboard only: F10, the reason, Enter to the sign, the sign, then F10 saves', async () => {
      await openChanged();
      const dialog = await openSignBox(language);
      const user = userEvent.setup();
      await user.keyboard('wrong amount{Enter}');
      await waitFor(() => {
        expect(signBoxFields(language, dialog).sign).toHaveFocus();
      });
      await user.keyboard('ADNAN ');
      press('F10');
      await waitFor(() => {
        expect(store.edits).toHaveLength(1);
      });
      // The sign goes as typed, trimmed: the backend judges it (case and end spaces do not matter there).
      expect(store.edits[0]).toMatchObject({ reason: 'wrong amount', sign: 'ADNAN' });
    });

    it('Esc closes the box with nothing saved, and the change stays in the form', async () => {
      await openChanged();
      const user = userEvent.setup();
      await openSignBox(language);
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(store.editTries).toHaveLength(0);
      expect((await labelled(language, 'vouchers-field-amount')).value).toBe('9,000');
    });

    it('a refusal at the field `reason` is shown under the reason in the box: it is never silent', async () => {
      vi.mocked(vouchersApi.vouchersEdit).mockRejectedValueOnce(
        fail({ code: 'ERR_REASON_REQUIRED', field: 'reason' }),
      );
      await openChanged();
      const user = userEvent.setup();
      const dialog = await saveChange(user, language, 'wrong amount');
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(m('err-reason-required'));
      });
      const { reason } = signBoxFields(language, dialog);
      expect(noteOf(reason)).toContain(m('err-reason-required'));
      expect(screen.getByRole('alertdialog')).toBeInTheDocument();
    });

    it('any other refusal is shown in the form after the box closes: F10 never does nothing', async () => {
      vi.mocked(vouchersApi.vouchersEdit).mockRejectedValueOnce(
        fail({ code: 'ERR_VERSION_CHANGED', field: 'version' }),
      );
      await openChanged();
      const user = userEvent.setup();
      await saveChange(user, language, 'wrong amount');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(
        (
          await screen.findAllByText((content) =>
            visible(content).includes(m('err-version-changed')),
          )
        ).length,
      ).toBeGreaterThan(0);
    });

    it('cash below zero: the box of the reason and the sign comes first, then the cash question, then one save with all of it', async () => {
      store = fakeVouchers({ cash: 20000 });
      store.rows.push(paymentRow());
      await openChanged('60000');
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain(
          m('common-cash-after', { amount: '-40,000' }),
        );
      });
      const user = userEvent.setup();
      await saveChange(user, language, 'wrong amount');
      // The reason and the sign are given: now the cash question, and still nothing is sent.
      const yes = await screen.findByRole('button', { name: m('common-cash-confirm-yes') });
      expect(store.editTries).toHaveLength(0);
      await user.click(yes);
      await waitFor(() => {
        expect(store.edits).toHaveLength(1);
      });
      expect(store.edits[0]).toMatchObject({
        reason: 'wrong amount',
        sign: 'adnan',
        confirm: true,
        entry: { type: 'payment', amount: '60000' },
      });
    });
  },
);
