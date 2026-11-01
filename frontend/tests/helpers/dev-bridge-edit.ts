// The stand-in answers for plan 44 (the edit kit, reopen and fix, the partner code), for the
// development pages only. They use the backend's own example files (never edited), so a screen
// photographed in a browser shows the real shapes. Flags in the hash of the page choose the story:
//   ?edited        the saved بکری has been edited (the stamp and the history)
//   ?closedday     the saved بکری or voucher is of a closed day (Edit reads "reopen the day and fix")
//   ?locked        the partner code is set and the section is closed
//   ?notset        no partner code is made yet
//   ?waiting       five wrong codes: the wait
// The reviews of the edits of the trip, the garden and the purchase are the backend's own files.
// Never part of a build: only the development bridge imports it.

import type { EditHistory } from '../../src/bindings/EditHistory';
import type { PartnerCodeStatus } from '../../src/bindings/PartnerCodeStatus';
import { fakeEditReview } from './bikri-plan16';
import { HEADS } from './cashbook-books';
import { voucherRow } from './voucher-books';
import { paisaForTest } from './paisa';
import { editable } from './bikri-plan16-data';
import {
  AUDIT_EDIT_HISTORY_BIKRI,
  AUDIT_EDIT_STAMP_BIKRI,
  BIKRI_EDIT_REVIEW_AKRAM_610,
  DAY_FIX_CLOSE_E2E_45,
  DAY_FIX_OPEN_E2E_45,
  FRUIT_EDIT_PREVIEW_HOME_3,
  GARDENS_ARRIVAL_EDIT_PREVIEW_120,
  GARDENS_ENTRY_EDIT_PREVIEW_32000,
  GARDENS_HEADER_EDIT_PREVIEW_RENT_60000,
  PARTNER_CODE_STATUS_LOCKED,
  PARTNER_CODE_STATUS_NOT_SET,
  PARTNER_CODE_STATUS_UNLOCKED,
  PARTNER_CODE_STATUS_WAITING,
  PURCHASES_EDIT_PREVIEW_RATE_520,
  TRIPS_COST_EDIT_PREVIEW_4400,
  TRIPS_HEADER_EDIT_PREVIEW_MARKET,
  TRIPS_OTHER_SALE_EDIT_PREVIEW_9000,
  VOUCHERS_EDIT_PREVIEW_RECEIPT_4000,
} from './edit-real';

type Input = Record<string, unknown>;
type Handler = (input: Input) => unknown;

/** The partner code as the stand-in keeps it for one page load. */
let code: PartnerCodeStatus = devFlag('notset')
  ? PARTNER_CODE_STATUS_NOT_SET
  : devFlag('waiting')
    ? PARTNER_CODE_STATUS_WAITING
    : devFlag('locked')
      ? PARTNER_CODE_STATUS_LOCKED
      : PARTNER_CODE_STATUS_UNLOCKED;
let tries = 5;
const RIGHT_CODE = '4826';

/** The flags in the hash of the development page, for example `#/dev/bikri?edited`. */
function devFlag(name: string): boolean {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '').has(name);
}

/** A refusal the way a backend command fails (development only). */
function refused(codeName: string, field: string, values: Record<string, string | number> = {}) {
  return Object.assign(new Error(codeName), {
    ok: false,
    errors: [{ code: codeName, field, values }],
    ref: 'E-DEV',
  });
}

/** The history of the shop expense of the dev page: its amount and its note were changed once. */
const EXPENSE_HISTORY: EditHistory = {
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
      reason: 'wrong bill',
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
      ],
      changes_truncated: false,
    },
  ],
};

export const editHandlers: Record<string, Handler> = {
  partners_code_status: () => code,
  partners_code_unlock: (input) => {
    if (input['code'] === RIGHT_CODE) {
      code = PARTNER_CODE_STATUS_UNLOCKED;
      tries = 5;
      return code;
    }
    tries -= 1;
    if (tries <= 0) {
      code = PARTNER_CODE_STATUS_WAITING;
      throw refused('ERR_LOCKED_WAIT', 'code', { seconds: 60 });
    }
    throw refused('ERR_PARTNER_CODE_WRONG', 'code', { tries_left: tries });
  },
  partners_code_lock: () => {
    code = PARTNER_CODE_STATUS_LOCKED;
    return code;
  },
  partners_code_set: (input) => {
    if (input['code'] !== input['code_again']) {
      throw refused('ERR_PARTNER_CODES_DIFFER', 'code_again');
    }
    code = PARTNER_CODE_STATUS_UNLOCKED;
    return code;
  },
  partners_code_change: () => code,
  partners_code_reset: () => {
    code = PARTNER_CODE_STATUS_UNLOCKED;
    return code;
  },
  bikri_edit_review: (input) => {
    const draft = input['draft'] as Parameters<typeof fakeEditReview>[1];
    const found = fakeEditReview(editable(), draft);
    // A change shows the backend's own example of اکرم's 600 to 610: the changes, the balances and
    // the partners (E2E-13). No change shows the empty review.
    return found.review.changes.length > 0 ? BIKRI_EDIT_REVIEW_AKRAM_610 : found;
  },
  vouchers_edit_preview: () => VOUCHERS_EDIT_PREVIEW_RECEIPT_4000,
  // A shop expense of the worked day (electricity 3,000), for the edit screen of the روکڑ area.
  vouchers_open_for_edit: () => ({
    row: voucherRow({
      id: 77,
      number: 3,
      voucher_type: 'shop_expense',
      account: null,
      narration: 'October bill',
      amount_paisa: paisaForTest(300_000),
      ...(devFlag('closedday') ? { needs_reopen: true } : {}),
      // An expense that was edited before (`?edited`): it carries its stamp and the marks of its history.
      ...(devFlag('edited') ? { edited: AUDIT_EDIT_STAMP_BIKRI[0]?.stamp ?? null } : {}),
    }),
    entry: {
      type: 'shop_expense',
      date: '10-10-2026',
      expense_head_id: HEADS[0]?.id ?? 1,
      amount: '3000',
      paid_to: 'Wapda',
      narration: 'October bill',
      confirm_cash_below_zero: false,
    },
  }),
  trips_cost_edit_preview: () => TRIPS_COST_EDIT_PREVIEW_4400,
  trips_other_sale_edit_preview: () => TRIPS_OTHER_SALE_EDIT_PREVIEW_9000,
  trips_header_edit_preview: () => TRIPS_HEADER_EDIT_PREVIEW_MARKET,
  purchases_edit_preview: () => PURCHASES_EDIT_PREVIEW_RATE_520,
  gardens_entry_edit_preview: () => GARDENS_ENTRY_EDIT_PREVIEW_32000,
  gardens_arrival_edit_preview: () => GARDENS_ARRIVAL_EDIT_PREVIEW_120,
  gardens_header_edit_preview: () => GARDENS_HEADER_EDIT_PREVIEW_RENT_60000,
  fruit_edit_preview: () => FRUIT_EDIT_PREVIEW_HOME_3,
  day_fix_open: () => DAY_FIX_OPEN_E2E_45,
  day_fix_close: () => DAY_FIX_CLOSE_E2E_45,
  audit_edit_stamp: () => AUDIT_EDIT_STAMP_BIKRI,
  audit_edit_history: (input) =>
    input['entity'] === 'voucher' ? EXPENSE_HISTORY : AUDIT_EDIT_HISTORY_BIKRI,
};
