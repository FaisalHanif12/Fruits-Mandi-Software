// The backend's answers for plan 44 (the edit review, the stamp, the history, reopen and fix, the
// partner code), copied from its own contract test (backend/crates/services/tests/fixtures) and read
// as the generated types. Never edited. T1 fixtures are examples built from PRD numbers
// (E2E-13/45, FR-15.3); T2 gives real answers of the same shape.

import type { DayFixClosed } from '../../src/bindings/DayFixClosed';
import type { DayFixOpened } from '../../src/bindings/DayFixOpened';
import type { EditHistory } from '../../src/bindings/EditHistory';
import type { EditPreview } from '../../src/bindings/EditPreview';
import type { EditStampRow } from '../../src/bindings/EditStampRow';
import type { EntryEdited } from '../../src/bindings/EntryEdited';
import type { EntryFlags } from '../../src/bindings/EntryFlags';
import type { PartnerCodeStatus } from '../../src/bindings/PartnerCodeStatus';
import type { PartnersList } from '../../src/bindings/PartnersList';
import type { UiError } from '../../src/bindings/UiError';

import auditEditHistoryBikri from '../fixtures/audit_edit_history_bikri.json';
import auditEditStampBikri from '../fixtures/audit_edit_stamp_bikri.json';
import bikriEditReviewAkram610 from '../fixtures/bikri_edit_review_akram_610.json';
import dayFixCloseE2e45 from '../fixtures/day_fix_close_e2e_45.json';
import dayFixCloseStayedOpen from '../fixtures/day_fix_close_stayed_open.json';
import dayFixCloseTokenInvalid from '../fixtures/day_fix_close_token_invalid.json';
import dayFixOpenE2e45 from '../fixtures/day_fix_open_e2e_45.json';
import dayFixOpenE2e45Request from '../fixtures/day_fix_open_e2e_45_request.json';
import dayFixOpenSignRequired from '../fixtures/day_fix_open_sign_required.json';
import dayFixOpenSignRequiredRequest from '../fixtures/day_fix_open_sign_required_request.json';
import dayFixOpenSignWrong from '../fixtures/day_fix_open_sign_wrong.json';
import dayFixOpenSignWrongRequest from '../fixtures/day_fix_open_sign_wrong_request.json';
import entryFlagsExamples from '../fixtures/entry_flags_examples.json';
import fruitEditPreviewHome3 from '../fixtures/fruit_edit_preview_home_3.json';
import gardensArrivalEditPreview120 from '../fixtures/gardens_arrival_edit_preview_120.json';
import gardensEntryEditPreview32000 from '../fixtures/gardens_entry_edit_preview_32000.json';
import gardensEntryEditSave32000 from '../fixtures/gardens_entry_edit_save_32000.json';
import gardensHeaderEditPreviewRent60000 from '../fixtures/gardens_header_edit_preview_rent_60000.json';
import partnersCodeSet from '../fixtures/partners_code_set.json';
import partnersCodeStatusLocked from '../fixtures/partners_code_status_locked.json';
import partnersCodeStatusNotSet from '../fixtures/partners_code_status_not_set.json';
import partnersCodeStatusUnlocked from '../fixtures/partners_code_status_unlocked.json';
import partnersCodeStatusWaiting from '../fixtures/partners_code_status_waiting.json';
import partnersCodeUnlockWrong from '../fixtures/partners_code_unlock_wrong.json';
import partnersListLocked from '../fixtures/partners_list_locked.json';
import partnersListOpen from '../fixtures/partners_list_open.json';
import purchasesEditPreviewDateFixed from '../fixtures/purchases_edit_preview_date_fixed.json';
import purchasesEditPreviewGoodsMoved from '../fixtures/purchases_edit_preview_goods_moved.json';
import purchasesEditPreviewRate520 from '../fixtures/purchases_edit_preview_rate_520.json';
import purchasesEditSaveRate520 from '../fixtures/purchases_edit_save_rate_520.json';
import tripsCostEditPreview4400 from '../fixtures/trips_cost_edit_preview_4400.json';
import tripsCostEditSave4400 from '../fixtures/trips_cost_edit_save_4400.json';
import tripsHeaderEditPreviewMarket from '../fixtures/trips_header_edit_preview_market.json';
import tripsOtherSaleEditPreview9000 from '../fixtures/trips_other_sale_edit_preview_9000.json';
import vouchersEditPreviewReceipt4000 from '../fixtures/vouchers_edit_preview_receipt_4000.json';

/** E2E-13: اکرم's rate 600 to 610; اکرم +212, رحیم +188 (credit), the partners 1,590 / 1,191 / 1,191. */
export const BIKRI_EDIT_REVIEW_AKRAM_610 = bikriEditReviewAkram610 as unknown as EditPreview;
/** FR-15.3: a receipt of 10,000 changed to 4,000; the cash goes from 58,484 to 52,484. */
export const VOUCHERS_EDIT_PREVIEW_RECEIPT_4000 =
  vouchersEditPreviewReceipt4000 as unknown as EditPreview;
/** E2E-42: home fruit 2 changed to 3 پیٹی; the value 1,160 becomes 1,740. */
export const FRUIT_EDIT_PREVIEW_HOME_3 = fruitEditPreviewHome3 as unknown as EditPreview;
/** A review with a mistake at the date field and a reason that is missing. */
export const PURCHASES_EDIT_PREVIEW_DATE_FIXED =
  purchasesEditPreviewDateFixed as unknown as EditPreview;
/** The whole edit refused: the goods of the purchase have moved on. */
export const PURCHASES_EDIT_PREVIEW_GOODS_MOVED =
  purchasesEditPreviewGoodsMoved as unknown as UiError;

export const AUDIT_EDIT_STAMP_BIKRI = auditEditStampBikri as unknown as EditStampRow[];
export const AUDIT_EDIT_HISTORY_BIKRI = auditEditHistoryBikri as unknown as EditHistory;

/** E2E-45: 10-10, 11-10 and 12-10 reopen; then they close again. */
export const DAY_FIX_OPEN_E2E_45 = dayFixOpenE2e45 as unknown as DayFixOpened;
/** What the screen sends to reopen with the right sign, and the two refusals of plan 73 with what was sent. */
export const DAY_FIX_OPEN_E2E_45_REQUEST = dayFixOpenE2e45Request;
export const DAY_FIX_OPEN_SIGN_WRONG = dayFixOpenSignWrong as unknown as UiError;
export const DAY_FIX_OPEN_SIGN_WRONG_REQUEST = dayFixOpenSignWrongRequest;
export const DAY_FIX_OPEN_SIGN_REQUIRED = dayFixOpenSignRequired as unknown as UiError;
export const DAY_FIX_OPEN_SIGN_REQUIRED_REQUEST = dayFixOpenSignRequiredRequest;
export const DAY_FIX_CLOSE_E2E_45 = dayFixCloseE2e45 as unknown as DayFixClosed;
export const DAY_FIX_CLOSE_STAYED_OPEN = dayFixCloseStayedOpen as unknown as DayFixClosed;
export const DAY_FIX_CLOSE_TOKEN_INVALID = dayFixCloseTokenInvalid as unknown as UiError;

/** The four examples of the flags: open, closed day, edited, goods moved. */
export const ENTRY_FLAGS = entryFlagsExamples as unknown as {
  open: EntryFlags;
  closed_day: EntryFlags;
  edited: EntryFlags;
  goods_moved: EntryFlags;
};

export const PARTNER_CODE_STATUS_NOT_SET: PartnerCodeStatus = partnersCodeStatusNotSet;
export const PARTNER_CODE_STATUS_LOCKED: PartnerCodeStatus = partnersCodeStatusLocked;
export const PARTNER_CODE_STATUS_UNLOCKED: PartnerCodeStatus = partnersCodeStatusUnlocked;
export const PARTNER_CODE_STATUS_WAITING: PartnerCodeStatus = partnersCodeStatusWaiting;
export const PARTNER_CODE_SET: PartnerCodeStatus = partnersCodeSet;
export const PARTNER_CODE_UNLOCK_WRONG = partnersCodeUnlockWrong as unknown as UiError;
export const PARTNERS_LIST_OPEN: PartnersList = partnersListOpen;
export const PARTNERS_LIST_LOCKED: PartnersList = partnersListLocked;

/** A review that finds no change: the entry as loaded, nothing to list, nothing moves. */
export function noChangeReview(entryDate = '2026-10-10', reasonRequired = false): EditPreview {
  return {
    errors: [],
    review: {
      entry_date: entryDate,
      changes: [],
      effect: { parties: [], cash: null, partners: [], partners_locked: false },
      reason_required: reasonRequired,
      needs_reopen: false,
    },
  };
}

// ---- The edits of the trip, the garden and the purchase (plan 44, T2a part 2): the backend's own
// answers, read as the generated types. ----
export const TRIPS_COST_EDIT_PREVIEW_4400 = tripsCostEditPreview4400 as unknown as EditPreview;
export const TRIPS_COST_EDIT_SAVE_4400 = tripsCostEditSave4400 as unknown as EntryEdited;
export const TRIPS_OTHER_SALE_EDIT_PREVIEW_9000 =
  tripsOtherSaleEditPreview9000 as unknown as EditPreview;
export const TRIPS_HEADER_EDIT_PREVIEW_MARKET =
  tripsHeaderEditPreviewMarket as unknown as EditPreview;
export const PURCHASES_EDIT_PREVIEW_RATE_520 =
  purchasesEditPreviewRate520 as unknown as EditPreview;
export const PURCHASES_EDIT_SAVE_RATE_520 = purchasesEditSaveRate520 as unknown as EntryEdited;
export const GARDENS_ENTRY_EDIT_PREVIEW_32000 =
  gardensEntryEditPreview32000 as unknown as EditPreview;
export const GARDENS_ENTRY_EDIT_SAVE_32000 = gardensEntryEditSave32000 as unknown as EntryEdited;
export const GARDENS_ARRIVAL_EDIT_PREVIEW_120 =
  gardensArrivalEditPreview120 as unknown as EditPreview;
export const GARDENS_HEADER_EDIT_PREVIEW_RENT_60000 =
  gardensHeaderEditPreviewRent60000 as unknown as EditPreview;
