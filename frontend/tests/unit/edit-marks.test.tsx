// The "Before ..." marks inside an edited entry (plan 44, PRD FR-7.71), in the Urdu and the English
// app: each field that an earlier edit changed carries a small mark with what it held before, and
// its tip has the old against the new value with the time and name of that edit. The marks are read
// from the entry's history (the audit log). A field that was never changed has none, and an entry
// that was never edited asks for no history.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as editApi from '../../src/shared/edit/api';
import { LANGUAGES } from '../../src/i18n';
import type { EditHistory } from '../../src/bindings/EditHistory';
import { named, prepareCashbook } from '../helpers/cashbook-screen';
import { leaveAppWindow, renderApp } from '../helpers/app';
import { AUDIT_EDIT_STAMP_BIKRI, noChangeReview } from '../helpers/edit-real';
import { paisaForTest } from '../helpers/paisa';
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

const STAMP = AUDIT_EDIT_STAMP_BIKRI[0]?.stamp ?? null;

/** A shop expense of 3,000 that an earlier edit changed from 2,500 and from the note "Oct". */
function editedExpense(edited = STAMP) {
  return {
    row: voucherRow({
      id: 77,
      number: 3,
      voucher_type: 'shop_expense',
      account: null,
      amount_paisa: paisaForTest(300_000),
      edited,
    }),
    entry: {
      type: 'shop_expense' as const,
      date: '10-10-2026',
      expense_head_id: 1,
      amount: '3000',
      paid_to: null,
      narration: 'October bill',
      confirm_cash_below_zero: false,
    },
  };
}

const HISTORY: EditHistory = {
  entity: 'voucher',
  id: 77,
  versions: [
    {
      audit_id: 5,
      at: '2026-10-11T11:35:00Z',
      local_date: '2026-10-11',
      local_time: '16:35',
      by: 'Owner',
      action: 'edit',
      reason: 'typo',
      changes: [
        {
          path: 'amount_paisa',
          old: { kind: 'money', paisa: paisaForTest(250_000) },
          new: { kind: 'money', paisa: paisaForTest(300_000) },
        },
        {
          path: 'narration',
          old: { kind: 'text', text: 'Oct' },
          new: { kind: 'text', text: 'October bill' },
        },
        // The head was another one: its name is read, not its number.
        {
          path: 'expense_head_id',
          old: { kind: 'number', value: 2 },
          new: { kind: 'number', value: 1 },
        },
      ],
      changes_truncated: false,
      // An edit of an open day has no sign (PRD FR-7.71).
      sign: null,
    },
  ],
};

describe.each(LANGUAGES)('the marks of an edited expense (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(noChangeReview());
    vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
  });

  const marks = () => Array.from(document.querySelectorAll<HTMLElement>('[data-field-mark]'));

  it('a changed field says what it held before, and its tip has the old and the new with the edit', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(editedExpense());
    vi.mocked(editApi.auditEditHistory).mockResolvedValue(HISTORY);
    await renderApp('/rokar/expense?type=shop_expense&id=77');
    await screen.findByLabelText(named(message(language, 'cashbook-field-amount')));
    await waitFor(() => {
      expect(marks().length).toBeGreaterThanOrEqual(3);
    });
    const texts = marks().map((mark) => visible(mark.textContent));
    expect(texts).toContain(message(language, 'edit-mark-before', { before: '2,500' }));
    expect(texts).toContain(message(language, 'edit-mark-before', { before: 'Oct' }));
    const amountMark = marks().find(
      (mark) =>
        visible(mark.textContent) === message(language, 'edit-mark-before', { before: '2,500' }),
    );
    expect(visible(amountMark?.title)).toContain(
      message(language, 'edit-field-mark', { before: '2,500', after: '3,000' }),
    );
    expect(visible(amountMark?.title)).toContain('11-10-2026');
    expect(visible(amountMark?.title)).toContain('Owner');
    expect(editApi.auditEditHistory).toHaveBeenCalledWith('voucher', 77);
  });

  it('a field nobody changed has no mark, and an entry never edited asks for no history', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(editedExpense(null));
    vi.mocked(editApi.auditEditHistory).mockResolvedValue(HISTORY);
    await renderApp('/rokar/expense?type=shop_expense&id=77');
    await screen.findByLabelText(named(message(language, 'cashbook-field-amount')));
    expect(marks()).toHaveLength(0);
    expect(editApi.auditEditHistory).not.toHaveBeenCalled();
  });
});
