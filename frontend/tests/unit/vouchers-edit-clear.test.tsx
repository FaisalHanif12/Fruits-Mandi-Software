// Plan 89 (Umar's report of 21-11-2026): changing a saved وصولی, ادائیگی or پیشگی is clear. The amber band says
// what CAN be changed (the amount, the account and the narration; only the day cannot), not only that the day is
// fixed. The amount and the reason for the change stand in the first rows of the form and the cursor starts in
// the amount; the party card is drawn beside the fields, never between them, so a tall card cannot push the amount
// out of view. After the save the voucher stays on the screen with its stamp (time, Owner, the sign when the day was
// reopened, the reason), the mark "Before 10,000" on the amount with the old and the new figure in its tip, and its
// row in the list reads "Edited" (PRD FR-7.70, FR-7.71, FR-15.3, FR-21.4). The save is proved end to end for the
// three types and for a voucher of a closed day (reopen with the sign, change, save, the day closes again). In the
// Urdu and in the English app, against a fake backend. The screen works nothing out: every figure is the backend's.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { EditHistory } from '../../src/bindings/EditHistory';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
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
import { DAY_FIX_CLOSE_E2E_45, DAY_FIX_OPEN_E2E_45 } from '../helpers/edit-real';
import { fakeMasters } from '../helpers/masters';
import { EMPTY_PAIRS } from '../helpers/paged';
import { paisaForTest } from '../helpers/paisa';
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
const pageText = () => visible(document.body.textContent);
const rupees = (value: number) => paisaForTest(value * 100);

/** The voucher of a type, as the list and the edit read it: 10,000 with Akram (a buyer) or Rahim (a seller). */
function rowOf(type: 'receipt' | 'payment' | 'advance') {
  const rahim = { id: RAHIM.id, code: RAHIM.code, name_ur: RAHIM.name_ur, name_en: RAHIM.name_en };
  return type === 'receipt'
    ? voucherRow({ id: 501, number: 1 })
    : voucherRow({
        id: type === 'payment' ? 601 : 701,
        voucher_type: type,
        number: 1,
        account: rahim,
        account_type: 'seller',
      });
}

/** What the audit log holds after the edit: the amount went from 10,000 to 9,000, with the reason. */
function historyOf(row: { id: number }, sign: string | null): EditHistory {
  return {
    entity: 'voucher',
    id: row.id,
    versions: [
      {
        audit_id: 9,
        at: '2026-10-10T11:35:00Z',
        local_date: '2026-10-10',
        local_time: '16:35',
        by: 'Owner',
        action: 'edit',
        reason: 'wrong amount',
        changes: [
          {
            path: 'amount_paisa',
            old: { kind: 'money', paisa: rupees(10_000) },
            new: { kind: 'money', paisa: rupees(9_000) },
          },
        ],
        changes_truncated: false,
        sign,
      },
    ],
  };
}

let store: FakeVouchers;

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  const rows = [AKRAM, ARSHAD, CHUKTA_KHATA, RAHIM, KARIM];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({});
  vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('editing a voucher is clear (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    store = fakeVouchers();
    store.rows.push(rowOf('receipt'), rowOf('payment'), rowOf('advance'));
  });

  /** Opens a saved voucher by its address and waits until the form shows it. */
  async function openSaved(type: 'receipt' | 'payment' | 'advance') {
    const id = { receipt: 501, payment: 601, advance: 701 }[type];
    const app = await renderApp(`/bikri/voucher?type=${type}&id=${String(id)}`);
    const amount = await labelled(language, 'vouchers-field-amount');
    return { ...app, amount, user: userEvent.setup() };
  }

  it.each(['receipt', 'payment', 'advance'] as const)(
    'the band of a %s says what can be changed, and that only the day cannot',
    async (type) => {
      await openSaved(type);
      expect(pageText()).toContain(m('edit-band-can-voucher'));
      // The old line told only that the day cannot change, which read as "cancel it".
      expect(pageText()).not.toContain(m('edit-band-date-fixed'));
      expect(pageText()).toContain(
        m('edit-band', {
          what: m(
            {
              receipt: 'glossary-wasooli',
              payment: 'glossary-adaigi',
              advance: 'glossary-peshgi-khata',
            }[type],
          ),
          number: '1',
          date: '10-10-2026',
        }),
      );
    },
  );

  it('the amount and the reason for the change come before the other fields, and the party card comes after all of them', async () => {
    const { amount } = await openSaved('receipt');
    const reason = await labelled(language, 'vouchers-field-edit-reason');
    const narration = await labelled(language, 'vouchers-field-narration');
    // The card is the one with the buyer's name inside the form (the list under it has his name too).
    const form = document.querySelector('form');
    if (form === null) throw new Error('no form');
    const card = await within(form).findByText(
      (text) => visible(text) === (language === 'ur' ? AKRAM.name_ur : AKRAM.name_en),
    );
    const before = (a: Node, b: Node) =>
      Boolean(a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING);
    // Amount, then the reason for the change, then the narration, as in the first rows of the form.
    expect(before(amount, reason)).toBe(true);
    expect(before(reason, narration)).toBe(true);
    // The card is beside the fields, so it follows every one of them: it can never sit between them and push
    // the amount down (Umar's screenshot).
    expect(before(narration, card)).toBe(true);
  });

  it('the cursor starts in the amount', async () => {
    const { amount } = await openSaved('receipt');
    await waitFor(() => {
      expect(amount).toHaveFocus();
    });
  });

  it.each(['receipt', 'payment', 'advance'] as const)(
    'a %s: change the amount, give the reason, F10: it is saved, and the voucher stays with its stamp, the mark on the amount and "Edited" in its row',
    async (type) => {
      vi.mocked(editApi.auditEditHistory).mockImplementation((_entity, id) =>
        Promise.resolve(historyOf({ id }, null)),
      );
      const { user, router, amount } = await openSaved(type);
      await user.click(amount);
      await user.keyboard('{Control>}a{/Control}9000');
      await user.click(await labelled(language, 'vouchers-field-edit-reason'));
      await user.keyboard('wrong amount');
      press('F10');
      await waitFor(() => {
        expect(store.edits).toHaveLength(1);
      });
      expect(store.edits[0]).toMatchObject({
        version: 1,
        reason: 'wrong amount',
        entry: { type, amount: '9000' },
      });
      // The voucher is still on the screen (not a blank form), with the amount that was saved.
      await waitFor(() => {
        expect(router.state.location.search).toHaveProperty('id');
      });
      await waitFor(() => {
        expect(document.querySelector('[data-edit-stamp]')).not.toBeNull();
      });
      const stamp = visible(document.querySelector('[data-edit-stamp]')?.textContent);
      expect(stamp).toContain('Owner');
      expect(stamp).toContain('10-10-2026');
      expect(stamp).toContain('wrong amount');
      expect((await labelled(language, 'vouchers-field-amount')).value).toBe('9,000');
      // The mark on the amount: what it held before, and the old against the new in its tip.
      const mark = await waitFor(() => {
        const found = document.querySelector<HTMLElement>('[data-field-mark]');
        if (found === null) throw new Error('no mark yet');
        return found;
      });
      expect(visible(mark.textContent)).toContain('10,000');
      expect(visible(mark.title)).toContain('9,000');
      // The row of the list says "Edited" too.
      await waitFor(() => {
        const id = { receipt: 501, payment: 601, advance: 701 }[type];
        const status = document.querySelector(`.ag-row[row-id="${String(id)}"] [col-id="status"]`);
        expect(visible(status?.textContent)).toContain(m('edit-mark'));
      });
    },
  );

  it('a voucher of a closed day: reopen with the sign, change the amount, save, the day closes again, and the read-only voucher shows the stamp with the sign and the mark', async () => {
    const row = store.rows.find((one) => one.id === 501);
    if (row === undefined) throw new Error('no row');
    row.entry_date = '2026-10-05';
    row.needs_reopen = true;
    store.editSign = 'adnan';
    vi.mocked(editApi.dayFixOpen).mockImplementation(() => {
      row.needs_reopen = false;
      return Promise.resolve(DAY_FIX_OPEN_E2E_45);
    });
    vi.mocked(editApi.dayFixClose).mockImplementation(() => {
      row.needs_reopen = true;
      return Promise.resolve(DAY_FIX_CLOSE_E2E_45);
    });
    vi.mocked(editApi.auditEditHistory).mockImplementation((_entity, id) =>
      Promise.resolve(historyOf({ id }, 'adnan')),
    );
    await renderApp('/bikri/voucher?type=receipt&id=501');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(m('edit-action-fix-day')),
      }),
    );
    const box = await screen.findByRole('alertdialog');
    await user.type(
      within(box).getByRole('combobox', { name: named(m('edit-fix-label')) }),
      'wrong amount{Enter}',
    );
    await user.keyboard('adnan{Enter}');
    // The days are open now: the form comes with the cursor in the amount.
    const amount = await labelled(language, 'vouchers-field-amount');
    await waitFor(() => {
      expect(amount).toHaveFocus();
    });
    await user.keyboard('{Control>}a{/Control}9000');
    await user.click(await labelled(language, 'vouchers-field-edit-reason'));
    await user.keyboard('wrong amount');
    press('F10');
    await waitFor(() => {
      expect(store.edits).toHaveLength(1);
    });
    await waitFor(() => {
      expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
    });
    // The day is closed again: the voucher is read only, with its stamp, the sign and the mark.
    await waitFor(() => {
      expect(document.querySelector('[data-edit-stamp]')).not.toBeNull();
    });
    const stamp = visible(document.querySelector('[data-edit-stamp]')?.textContent);
    expect(stamp).toContain('adnan');
    expect(stamp).toContain('wrong amount');
    expect(screen.queryByLabelText(named(m('vouchers-field-amount')))).toBeNull();
    const mark = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('[data-field-mark]');
      if (found === null) throw new Error('no mark yet');
      return found;
    });
    expect(visible(mark.textContent)).toContain('10,000');
    expect(visible(mark.title)).toContain('9,000');
  });

  it('nothing in the band, the form or the stamp is a raw message id', async () => {
    await openSaved('receipt');
    expect(pageText()).not.toMatch(/\bedit-[a-z-]+\b/);
    expect(pageText()).not.toMatch(/\bvouchers-[a-z-]+\b/);
  });
});

// The messages exist in both languages with the same meaning (NFR-URD-29): the band of a voucher names the three
// things that can be changed and the one that cannot.
describe('the words of the band', () => {
  it('Urdu and English both say amount, account, narration, and that only the day is fixed', () => {
    const ur = message('ur', 'edit-band-can-voucher');
    const en = message('en', 'edit-band-can-voucher');
    for (const word of ['رقم', 'کھاتہ', 'تفصیل', 'دن']) expect(ur).toContain(word);
    for (const word of ['amount', 'account', 'narration', 'day']) expect(en).toContain(word);
  });
});
