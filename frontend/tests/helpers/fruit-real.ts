// The real answers and requests of the backend for plan 17d (fruit taken): the lots, the previews
// (home, guests, another shop, a weight item, the refusals), the saves, the edit, the cancels, the
// entry as it is read, the lists and the partner statement with its fruit lines. The backend wrote
// them in its own test (backend/crates/services/tests/fruit_fixtures.rs and the golden day);
// they are copied here, never edited, and read as the generated types. Money in them is paisa, as
// on the wire. The story is E2E-42: 5 پیٹی کیلا left in رحیم's lot; home fruit 2 × 580 = 1,160
// taken by A; guest fruit of 3,000 shared 1,000 / 1,000 / 1,000.

import type { CancelInput } from '../../src/bindings/CancelInput';
import type { FruitCancelInput } from '../../src/bindings/FruitCancelInput';
import type { FruitCancelled } from '../../src/bindings/FruitCancelled';
import type { FruitEditInput } from '../../src/bindings/FruitEditInput';
import type { FruitEdited } from '../../src/bindings/FruitEdited';
import type { FruitGetInput } from '../../src/bindings/FruitGetInput';
import type { FruitInput } from '../../src/bindings/FruitInput';
import type { FruitList } from '../../src/bindings/FruitList';
import type { FruitListInput } from '../../src/bindings/FruitListInput';
import type { FruitLots } from '../../src/bindings/FruitLots';
import type { FruitLotsInput } from '../../src/bindings/FruitLotsInput';
import type { FruitPreview } from '../../src/bindings/FruitPreview';
import type { FruitRow } from '../../src/bindings/FruitRow';
import type { FruitSaved } from '../../src/bindings/FruitSaved';
import type { PartnerStatement } from '../../src/bindings/PartnerStatement';
import type { PartnersStatementInput } from '../../src/bindings/PartnersStatementInput';
import type { UiError } from '../../src/bindings/UiError';

import bikriCancelHasFruit from '../fixtures/bikri_cancel_has_fruit.json';
import bikriCancelHasFruitRequest from '../fixtures/bikri_cancel_has_fruit_request.json';
import fruitCancelAgain from '../fixtures/fruit_cancel_again.json';
import fruitCancelAgainRequest from '../fixtures/fruit_cancel_again_request.json';
import fruitCancelHome from '../fixtures/fruit_cancel_home.json';
import fruitCancelHomeRequest from '../fixtures/fruit_cancel_home_request.json';
import fruitEditHome from '../fixtures/fruit_edit_home.json';
import fruitEditHomeRequest from '../fixtures/fruit_edit_home_request.json';
import fruitGetHome from '../fixtures/fruit_get_home.json';
import fruitGetHomeRequest from '../fixtures/fruit_get_home_request.json';
import fruitGetHomeCancelled from '../fixtures/fruit_get_home_cancelled.json';
import fruitGetHomeCancelledRequest from '../fixtures/fruit_get_home_cancelled_request.json';
import fruitGetHomeEdited from '../fixtures/fruit_get_home_edited.json';
import fruitGetHomeEditedRequest from '../fixtures/fruit_get_home_edited_request.json';
import fruitListAll from '../fixtures/fruit_list_all.json';
import fruitListAllRequest from '../fixtures/fruit_list_all_request.json';
import fruitListCancelled from '../fixtures/fruit_list_cancelled.json';
import fruitListCancelledRequest from '../fixtures/fruit_list_cancelled_request.json';
import fruitListGuestsOnly from '../fixtures/fruit_list_guests_only.json';
import fruitListGuestsOnlyRequest from '../fixtures/fruit_list_guests_only_request.json';
import fruitListPartnerPaged from '../fixtures/fruit_list_partner_paged.json';
import fruitListPartnerPagedRequest from '../fixtures/fruit_list_partner_paged_request.json';
import fruitLotsAll from '../fixtures/fruit_lots_all.json';
import fruitLotsAllRequest from '../fixtures/fruit_lots_all_request.json';
import fruitLotsSearch from '../fixtures/fruit_lots_search.json';
import fruitLotsSearchRequest from '../fixtures/fruit_lots_search_request.json';
import fruitPreviewEditHome from '../fixtures/fruit_preview_edit_home.json';
import fruitPreviewEditHomeRequest from '../fixtures/fruit_preview_edit_home_request.json';
import fruitPreviewGuests from '../fixtures/fruit_preview_guests.json';
import fruitPreviewGuestsRequest from '../fixtures/fruit_preview_guests_request.json';
import fruitPreviewGuestsOneRupee from '../fixtures/fruit_preview_guests_one_rupee.json';
import fruitPreviewGuestsOneRupeeRequest from '../fixtures/fruit_preview_guests_one_rupee_request.json';
import fruitPreviewHome from '../fixtures/fruit_preview_home.json';
import fruitPreviewHomeRequest from '../fixtures/fruit_preview_home_request.json';
import fruitPreviewQtyExceedsLot from '../fixtures/fruit_preview_qty_exceeds_lot.json';
import fruitPreviewQtyExceedsLotRequest from '../fixtures/fruit_preview_qty_exceeds_lot_request.json';
import fruitPreviewRateZero from '../fixtures/fruit_preview_rate_zero.json';
import fruitPreviewRateZeroRequest from '../fixtures/fruit_preview_rate_zero_request.json';
import fruitPreviewWeightItem from '../fixtures/fruit_preview_weight_item.json';
import fruitPreviewWeightItemRequest from '../fixtures/fruit_preview_weight_item_request.json';
import fruitPreviewWeightRequired from '../fixtures/fruit_preview_weight_required.json';
import fruitPreviewWeightRequiredRequest from '../fixtures/fruit_preview_weight_required_request.json';
import fruitSaveDateBeforeReceived from '../fixtures/fruit_save_date_before_received.json';
import fruitSaveDateBeforeReceivedRequest from '../fixtures/fruit_save_date_before_received_request.json';
import fruitSaveGuests from '../fixtures/fruit_save_guests.json';
import fruitSaveGuestsRequest from '../fixtures/fruit_save_guests_request.json';
import fruitSaveHome from '../fixtures/fruit_save_home.json';
import fruitSaveHomeRequest from '../fixtures/fruit_save_home_request.json';
import fruitSaveQtyExceedsLot from '../fixtures/fruit_save_qty_exceeds_lot.json';
import fruitSaveQtyExceedsLotRequest from '../fixtures/fruit_save_qty_exceeds_lot_request.json';
import partnersStatementFruitAlif from '../fixtures/partners_statement_fruit_alif.json';
import partnersStatementFruitAlifRequest from '../fixtures/partners_statement_fruit_alif_request.json';

// The fixture files are the backend's own JSON; each is read as its generated type. A reader takes
// `unknown`, because the money in a binding is a branded type that a plain number is not.
function readCancelInput(value: unknown): CancelInput {
  return value as CancelInput;
}

function readFruitCancelInput(value: unknown): FruitCancelInput {
  return value as FruitCancelInput;
}

function readFruitCancelled(value: unknown): FruitCancelled {
  return value as FruitCancelled;
}

function readFruitEditInput(value: unknown): FruitEditInput {
  return value as FruitEditInput;
}

function readFruitEdited(value: unknown): FruitEdited {
  return value as FruitEdited;
}

function readFruitGetInput(value: unknown): FruitGetInput {
  return value as FruitGetInput;
}

function readFruitInput(value: unknown): FruitInput {
  return value as FruitInput;
}

function readFruitList(value: unknown): FruitList {
  return value as FruitList;
}

function readFruitListInput(value: unknown): FruitListInput {
  return value as FruitListInput;
}

function readFruitLots(value: unknown): FruitLots {
  return value as FruitLots;
}

function readFruitLotsInput(value: unknown): FruitLotsInput {
  return value as FruitLotsInput;
}

function readFruitPreview(value: unknown): FruitPreview {
  return value as FruitPreview;
}

function readFruitRow(value: unknown): FruitRow {
  return value as FruitRow;
}

function readFruitSaved(value: unknown): FruitSaved {
  return value as FruitSaved;
}

function readPartnerStatement(value: unknown): PartnerStatement {
  return value as PartnerStatement;
}

function readPartnersStatementInput(value: unknown): PartnersStatementInput {
  return value as PartnersStatementInput;
}

function readUiError(value: unknown): UiError {
  return value as UiError;
}

export const BIKRI_CANCEL_HAS_FRUIT = readUiError(bikriCancelHasFruit);
export const BIKRI_CANCEL_HAS_FRUIT_REQUEST = readCancelInput(bikriCancelHasFruitRequest);
export const FRUIT_CANCEL_AGAIN = readUiError(fruitCancelAgain);
export const FRUIT_CANCEL_AGAIN_REQUEST = readFruitCancelInput(fruitCancelAgainRequest);
export const FRUIT_CANCEL_HOME = readFruitCancelled(fruitCancelHome);
export const FRUIT_CANCEL_HOME_REQUEST = readFruitCancelInput(fruitCancelHomeRequest);
export const FRUIT_EDIT_HOME = readFruitEdited(fruitEditHome);
export const FRUIT_EDIT_HOME_REQUEST = readFruitEditInput(fruitEditHomeRequest);
export const FRUIT_GET_HOME = readFruitRow(fruitGetHome);
export const FRUIT_GET_HOME_REQUEST = readFruitGetInput(fruitGetHomeRequest);
export const FRUIT_GET_HOME_CANCELLED = readFruitRow(fruitGetHomeCancelled);
export const FRUIT_GET_HOME_CANCELLED_REQUEST = readFruitGetInput(fruitGetHomeCancelledRequest);
export const FRUIT_GET_HOME_EDITED = readFruitRow(fruitGetHomeEdited);
export const FRUIT_GET_HOME_EDITED_REQUEST = readFruitGetInput(fruitGetHomeEditedRequest);
export const FRUIT_LIST_ALL = readFruitList(fruitListAll);
export const FRUIT_LIST_ALL_REQUEST = readFruitListInput(fruitListAllRequest);
export const FRUIT_LIST_CANCELLED = readFruitList(fruitListCancelled);
export const FRUIT_LIST_CANCELLED_REQUEST = readFruitListInput(fruitListCancelledRequest);
export const FRUIT_LIST_GUESTS_ONLY = readFruitList(fruitListGuestsOnly);
export const FRUIT_LIST_GUESTS_ONLY_REQUEST = readFruitListInput(fruitListGuestsOnlyRequest);
export const FRUIT_LIST_PARTNER_PAGED = readFruitList(fruitListPartnerPaged);
export const FRUIT_LIST_PARTNER_PAGED_REQUEST = readFruitListInput(fruitListPartnerPagedRequest);
export const FRUIT_LOTS_ALL = readFruitLots(fruitLotsAll);
export const FRUIT_LOTS_ALL_REQUEST = readFruitLotsInput(fruitLotsAllRequest);
export const FRUIT_LOTS_SEARCH = readFruitLots(fruitLotsSearch);
export const FRUIT_LOTS_SEARCH_REQUEST = readFruitLotsInput(fruitLotsSearchRequest);
export const FRUIT_PREVIEW_EDIT_HOME = readFruitPreview(fruitPreviewEditHome);
export const FRUIT_PREVIEW_EDIT_HOME_REQUEST = readFruitInput(fruitPreviewEditHomeRequest);
export const FRUIT_PREVIEW_GUESTS = readFruitPreview(fruitPreviewGuests);
export const FRUIT_PREVIEW_GUESTS_REQUEST = readFruitInput(fruitPreviewGuestsRequest);
export const FRUIT_PREVIEW_GUESTS_ONE_RUPEE = readFruitPreview(fruitPreviewGuestsOneRupee);
export const FRUIT_PREVIEW_GUESTS_ONE_RUPEE_REQUEST = readFruitInput(
  fruitPreviewGuestsOneRupeeRequest,
);
export const FRUIT_PREVIEW_HOME = readFruitPreview(fruitPreviewHome);
export const FRUIT_PREVIEW_HOME_REQUEST = readFruitInput(fruitPreviewHomeRequest);
export const FRUIT_PREVIEW_QTY_EXCEEDS_LOT = readFruitPreview(fruitPreviewQtyExceedsLot);
export const FRUIT_PREVIEW_QTY_EXCEEDS_LOT_REQUEST = readFruitInput(
  fruitPreviewQtyExceedsLotRequest,
);
export const FRUIT_PREVIEW_RATE_ZERO = readFruitPreview(fruitPreviewRateZero);
export const FRUIT_PREVIEW_RATE_ZERO_REQUEST = readFruitInput(fruitPreviewRateZeroRequest);
export const FRUIT_PREVIEW_WEIGHT_ITEM = readFruitPreview(fruitPreviewWeightItem);
export const FRUIT_PREVIEW_WEIGHT_ITEM_REQUEST = readFruitInput(fruitPreviewWeightItemRequest);
export const FRUIT_PREVIEW_WEIGHT_REQUIRED = readFruitPreview(fruitPreviewWeightRequired);
export const FRUIT_PREVIEW_WEIGHT_REQUIRED_REQUEST = readFruitInput(
  fruitPreviewWeightRequiredRequest,
);
export const FRUIT_SAVE_DATE_BEFORE_RECEIVED = readUiError(fruitSaveDateBeforeReceived);
export const FRUIT_SAVE_DATE_BEFORE_RECEIVED_REQUEST = readFruitInput(
  fruitSaveDateBeforeReceivedRequest,
);
export const FRUIT_SAVE_GUESTS = readFruitSaved(fruitSaveGuests);
export const FRUIT_SAVE_GUESTS_REQUEST = readFruitInput(fruitSaveGuestsRequest);
export const FRUIT_SAVE_HOME = readFruitSaved(fruitSaveHome);
export const FRUIT_SAVE_HOME_REQUEST = readFruitInput(fruitSaveHomeRequest);
export const FRUIT_SAVE_QTY_EXCEEDS_LOT = readUiError(fruitSaveQtyExceedsLot);
export const FRUIT_SAVE_QTY_EXCEEDS_LOT_REQUEST = readFruitInput(fruitSaveQtyExceedsLotRequest);
export const PARTNERS_STATEMENT_FRUIT_ALIF = readPartnerStatement(partnersStatementFruitAlif);
export const PARTNERS_STATEMENT_FRUIT_ALIF_REQUEST = readPartnersStatementInput(
  partnersStatementFruitAlifRequest,
);
