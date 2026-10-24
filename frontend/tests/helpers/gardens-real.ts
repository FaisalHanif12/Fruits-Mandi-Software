// The real answers and requests of the backend for plan 17c (gardens): the garden list, the garden
// with its history and each partner's money, the entries (installment, cost, sale in another shop),
// the closing and its undo, the refusals, and the partner overview and statement lines that come
// from a garden. The backend wrote them in its own test (backend/crates/services/tests/
// gardens_fixtures.rs and the golden day); they are copied here, never edited, and read as the
// generated types. Money in them is paisa, as on the wire. The story is the PRD worked numbers of
// E2E-40 (Shahpur Garden: الف 60% from the shop, ب 40% from his pocket, rent 50,000) and E2E-40b.

import type { GardenCloseInput } from '../../src/bindings/GardenCloseInput';
import type { GardenClosePreview } from '../../src/bindings/GardenClosePreview';
import type { GardenClosed } from '../../src/bindings/GardenClosed';
import type { GardenCreatePreview } from '../../src/bindings/GardenCreatePreview';
import type { GardenCreated } from '../../src/bindings/GardenCreated';
import type { GardenDefaults } from '../../src/bindings/GardenDefaults';
import type { GardenDetail } from '../../src/bindings/GardenDetail';
import type { GardenEntryCancelInput } from '../../src/bindings/GardenEntryCancelInput';
import type { GardenEntryCancelled } from '../../src/bindings/GardenEntryCancelled';
import type { GardenEntryInput } from '../../src/bindings/GardenEntryInput';
import type { GardenEntryPreview } from '../../src/bindings/GardenEntryPreview';
import type { GardenEntrySaved } from '../../src/bindings/GardenEntrySaved';
import type { GardenGetInput } from '../../src/bindings/GardenGetInput';
import type { GardenInput } from '../../src/bindings/GardenInput';
import type { GardenReopenInput } from '../../src/bindings/GardenReopenInput';
import type { GardenReopened } from '../../src/bindings/GardenReopened';
import type { GardenRow } from '../../src/bindings/GardenRow';
import type { GardenSetPartnersInput } from '../../src/bindings/GardenSetPartnersInput';
import type { GardensList } from '../../src/bindings/GardensList';
import type { GardensListInput } from '../../src/bindings/GardensListInput';
import type { GardensSummary } from '../../src/bindings/GardensSummary';
import type { PartnerOverviewRow } from '../../src/bindings/PartnerOverviewRow';
import type { PartnerStatement } from '../../src/bindings/PartnerStatement';
import type { PartnersStatementInput } from '../../src/bindings/PartnersStatementInput';
import type { UiError } from '../../src/bindings/UiError';

import gardensCloseBeforeLastEntry from '../fixtures/gardens_close_before_last_entry.json';
import gardensCloseBeforeLastEntryRequest from '../fixtures/gardens_close_before_last_entry_request.json';
import gardensCloseE2e40 from '../fixtures/gardens_close_e2e40.json';
import gardensCloseE2e40Request from '../fixtures/gardens_close_e2e40_request.json';
import gardensCloseLoss from '../fixtures/gardens_close_loss.json';
import gardensCloseLossRequest from '../fixtures/gardens_close_loss_request.json';
import gardensClosePreviewBeforeLastEntry from '../fixtures/gardens_close_preview_before_last_entry.json';
import gardensClosePreviewBeforeLastEntryRequest from '../fixtures/gardens_close_preview_before_last_entry_request.json';
import gardensClosePreviewE2e40 from '../fixtures/gardens_close_preview_e2e40.json';
import gardensClosePreviewE2e40Request from '../fixtures/gardens_close_preview_e2e40_request.json';
import gardensClosePreviewLoss from '../fixtures/gardens_close_preview_loss.json';
import gardensClosePreviewLossRequest from '../fixtures/gardens_close_preview_loss_request.json';
import gardensClosePreviewNothingToMove from '../fixtures/gardens_close_preview_nothing_to_move.json';
import gardensClosePreviewNothingToMoveRequest from '../fixtures/gardens_close_preview_nothing_to_move_request.json';
import gardensCreateE2e40 from '../fixtures/gardens_create_e2e40.json';
import gardensCreateE2e40Request from '../fixtures/gardens_create_e2e40_request.json';
import gardensCreateNoInstallment from '../fixtures/gardens_create_no_installment.json';
import gardensCreateNoInstallmentRequest from '../fixtures/gardens_create_no_installment_request.json';
import gardensCreatePreviewFirstInstallment from '../fixtures/gardens_create_preview_first_installment.json';
import gardensCreatePreviewFirstInstallmentRequest from '../fixtures/gardens_create_preview_first_installment_request.json';
import gardensCreateSharesNot100 from '../fixtures/gardens_create_shares_not_100.json';
import gardensCreateSharesNot100Request from '../fixtures/gardens_create_shares_not_100_request.json';
import gardensDefaultsFirst from '../fixtures/gardens_defaults_first.json';
import gardensDefaultsFromLastGarden from '../fixtures/gardens_defaults_from_last_garden.json';
import gardensEntryCancelCost from '../fixtures/gardens_entry_cancel_cost.json';
import gardensEntryCancelCostRequest from '../fixtures/gardens_entry_cancel_cost_request.json';
import gardensEntryCancelGardenClosed from '../fixtures/gardens_entry_cancel_garden_closed.json';
import gardensEntryCancelGardenClosedRequest from '../fixtures/gardens_entry_cancel_garden_closed_request.json';
import gardensEntryPreviewCashBelowZero from '../fixtures/gardens_entry_preview_cash_below_zero.json';
import gardensEntryPreviewCashBelowZeroRequest from '../fixtures/gardens_entry_preview_cash_below_zero_request.json';
import gardensEntryPreviewInstallment from '../fixtures/gardens_entry_preview_installment.json';
import gardensEntryPreviewInstallmentRequest from '../fixtures/gardens_entry_preview_installment_request.json';
import gardensEntryPreviewOtherSale from '../fixtures/gardens_entry_preview_other_sale.json';
import gardensEntryPreviewOtherSaleRequest from '../fixtures/gardens_entry_preview_other_sale_request.json';
import gardensEntrySaveCashBelowZero from '../fixtures/gardens_entry_save_cash_below_zero.json';
import gardensEntrySaveCashBelowZeroRequest from '../fixtures/gardens_entry_save_cash_below_zero_request.json';
import gardensEntrySaveCost from '../fixtures/gardens_entry_save_cost.json';
import gardensEntrySaveCostRequest from '../fixtures/gardens_entry_save_cost_request.json';
import gardensEntrySaveDateBeforeGarden from '../fixtures/gardens_entry_save_date_before_garden.json';
import gardensEntrySaveDateBeforeGardenRequest from '../fixtures/gardens_entry_save_date_before_garden_request.json';
import gardensEntrySaveGardenClosed from '../fixtures/gardens_entry_save_garden_closed.json';
import gardensEntrySaveGardenClosedRequest from '../fixtures/gardens_entry_save_garden_closed_request.json';
import gardensEntrySaveInstallment from '../fixtures/gardens_entry_save_installment.json';
import gardensEntrySaveInstallmentRequest from '../fixtures/gardens_entry_save_installment_request.json';
import gardensEntrySaveOtherSale from '../fixtures/gardens_entry_save_other_sale.json';
import gardensEntrySaveOtherSaleRequest from '../fixtures/gardens_entry_save_other_sale_request.json';
import gardensGetClosed from '../fixtures/gardens_get_closed.json';
import gardensGetClosedLoss from '../fixtures/gardens_get_closed_loss.json';
import gardensGetOpen from '../fixtures/gardens_get_open.json';
import gardensGetOpenRequest from '../fixtures/gardens_get_open_request.json';
import gardensGetReopened from '../fixtures/gardens_get_reopened.json';
import gardensListAll from '../fixtures/gardens_list_all.json';
import gardensListNoEntry from '../fixtures/gardens_list_no_entry.json';
import gardensListOpenPaged from '../fixtures/gardens_list_open_paged.json';
import gardensListOpenPagedRequest from '../fixtures/gardens_list_open_paged_request.json';
import gardensReopenE2e40 from '../fixtures/gardens_reopen_e2e40.json';
import gardensReopenE2e40Request from '../fixtures/gardens_reopen_e2e40_request.json';
import gardensReopenNotClosed from '../fixtures/gardens_reopen_not_closed.json';
import gardensReopenNotClosedRequest from '../fixtures/gardens_reopen_not_closed_request.json';
import gardensSetPartnersLocked from '../fixtures/gardens_set_partners_locked.json';
import gardensSetPartnersLockedRequest from '../fixtures/gardens_set_partners_locked_request.json';
import gardensSetPartnersNot100 from '../fixtures/gardens_set_partners_not_100.json';
import gardensSetPartnersNot100Request from '../fixtures/gardens_set_partners_not_100_request.json';
import gardensSetPartnersThree from '../fixtures/gardens_set_partners_three.json';
import gardensSetPartnersThreeRequest from '../fixtures/gardens_set_partners_three_request.json';
import gardensSummaryAfterClose from '../fixtures/gardens_summary_after_close.json';
import gardensSummaryOpen from '../fixtures/gardens_summary_open.json';
import partnersOverviewAfterGardenClose from '../fixtures/partners_overview_after_garden_close.json';
import partnersOverviewWithGardenCards from '../fixtures/partners_overview_with_garden_cards.json';
import partnersStatementGardenAlif from '../fixtures/partners_statement_garden_alif.json';
import partnersStatementGardenAlifRequest from '../fixtures/partners_statement_garden_alif_request.json';

// The fixture files are the backend's own JSON; each is read as its generated type. A reader takes
// `unknown`, because the money in a binding is a branded type that a plain number is not.
function readGardenCloseInput(value: unknown): GardenCloseInput {
  return value as GardenCloseInput;
}

function readGardenClosePreview(value: unknown): GardenClosePreview {
  return value as GardenClosePreview;
}

function readGardenClosed(value: unknown): GardenClosed {
  return value as GardenClosed;
}

function readGardenCreatePreview(value: unknown): GardenCreatePreview {
  return value as GardenCreatePreview;
}

function readGardenCreated(value: unknown): GardenCreated {
  return value as GardenCreated;
}

function readGardenDefaults(value: unknown): GardenDefaults {
  return value as GardenDefaults;
}

function readGardenDetail(value: unknown): GardenDetail {
  return value as GardenDetail;
}

function readGardenEntryCancelInput(value: unknown): GardenEntryCancelInput {
  return value as GardenEntryCancelInput;
}

function readGardenEntryCancelled(value: unknown): GardenEntryCancelled {
  return value as GardenEntryCancelled;
}

function readGardenEntryInput(value: unknown): GardenEntryInput {
  return value as GardenEntryInput;
}

function readGardenEntryPreview(value: unknown): GardenEntryPreview {
  return value as GardenEntryPreview;
}

function readGardenEntrySaved(value: unknown): GardenEntrySaved {
  return value as GardenEntrySaved;
}

function readGardenGetInput(value: unknown): GardenGetInput {
  return value as GardenGetInput;
}

function readGardenInput(value: unknown): GardenInput {
  return value as GardenInput;
}

function readGardenReopenInput(value: unknown): GardenReopenInput {
  return value as GardenReopenInput;
}

function readGardenReopened(value: unknown): GardenReopened {
  return value as GardenReopened;
}

function readGardenRow(value: unknown): GardenRow {
  return value as GardenRow;
}

function readGardenSetPartnersInput(value: unknown): GardenSetPartnersInput {
  return value as GardenSetPartnersInput;
}

function readGardensList(value: unknown): GardensList {
  return value as GardensList;
}

function readGardensListInput(value: unknown): GardensListInput {
  return value as GardensListInput;
}

function readGardensSummary(value: unknown): GardensSummary {
  return value as GardensSummary;
}

function readPartnerOverview(value: unknown): PartnerOverviewRow[] {
  return value as PartnerOverviewRow[];
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

export const GARDENS_CLOSE_BEFORE_LAST_ENTRY = readUiError(gardensCloseBeforeLastEntry);
export const GARDENS_CLOSE_BEFORE_LAST_ENTRY_REQUEST = readGardenCloseInput(
  gardensCloseBeforeLastEntryRequest,
);
export const GARDENS_CLOSE_E2E40 = readGardenClosed(gardensCloseE2e40);
export const GARDENS_CLOSE_E2E40_REQUEST = readGardenCloseInput(gardensCloseE2e40Request);
export const GARDENS_CLOSE_LOSS = readGardenClosed(gardensCloseLoss);
export const GARDENS_CLOSE_LOSS_REQUEST = readGardenCloseInput(gardensCloseLossRequest);
export const GARDENS_CLOSE_PREVIEW_BEFORE_LAST_ENTRY = readGardenClosePreview(
  gardensClosePreviewBeforeLastEntry,
);
export const GARDENS_CLOSE_PREVIEW_BEFORE_LAST_ENTRY_REQUEST = readGardenCloseInput(
  gardensClosePreviewBeforeLastEntryRequest,
);
export const GARDENS_CLOSE_PREVIEW_E2E40 = readGardenClosePreview(gardensClosePreviewE2e40);
export const GARDENS_CLOSE_PREVIEW_E2E40_REQUEST = readGardenCloseInput(
  gardensClosePreviewE2e40Request,
);
export const GARDENS_CLOSE_PREVIEW_LOSS = readGardenClosePreview(gardensClosePreviewLoss);
export const GARDENS_CLOSE_PREVIEW_LOSS_REQUEST = readGardenCloseInput(
  gardensClosePreviewLossRequest,
);
export const GARDENS_CLOSE_PREVIEW_NOTHING_TO_MOVE = readGardenClosePreview(
  gardensClosePreviewNothingToMove,
);
export const GARDENS_CLOSE_PREVIEW_NOTHING_TO_MOVE_REQUEST = readGardenCloseInput(
  gardensClosePreviewNothingToMoveRequest,
);
export const GARDENS_CREATE_E2E40 = readGardenCreated(gardensCreateE2e40);
export const GARDENS_CREATE_E2E40_REQUEST = readGardenInput(gardensCreateE2e40Request);
export const GARDENS_CREATE_NO_INSTALLMENT = readGardenCreated(gardensCreateNoInstallment);
export const GARDENS_CREATE_NO_INSTALLMENT_REQUEST = readGardenInput(
  gardensCreateNoInstallmentRequest,
);
export const GARDENS_CREATE_PREVIEW_FIRST_INSTALLMENT = readGardenCreatePreview(
  gardensCreatePreviewFirstInstallment,
);
export const GARDENS_CREATE_PREVIEW_FIRST_INSTALLMENT_REQUEST = readGardenInput(
  gardensCreatePreviewFirstInstallmentRequest,
);
export const GARDENS_CREATE_SHARES_NOT_100 = readUiError(gardensCreateSharesNot100);
export const GARDENS_CREATE_SHARES_NOT_100_REQUEST = readGardenInput(
  gardensCreateSharesNot100Request,
);
export const GARDENS_DEFAULTS_FIRST = readGardenDefaults(gardensDefaultsFirst);
export const GARDENS_DEFAULTS_FROM_LAST_GARDEN = readGardenDefaults(gardensDefaultsFromLastGarden);
export const GARDENS_ENTRY_CANCEL_COST = readGardenEntryCancelled(gardensEntryCancelCost);
export const GARDENS_ENTRY_CANCEL_COST_REQUEST = readGardenEntryCancelInput(
  gardensEntryCancelCostRequest,
);
export const GARDENS_ENTRY_CANCEL_GARDEN_CLOSED = readUiError(gardensEntryCancelGardenClosed);
export const GARDENS_ENTRY_CANCEL_GARDEN_CLOSED_REQUEST = readGardenEntryCancelInput(
  gardensEntryCancelGardenClosedRequest,
);
export const GARDENS_ENTRY_PREVIEW_CASH_BELOW_ZERO = readGardenEntryPreview(
  gardensEntryPreviewCashBelowZero,
);
export const GARDENS_ENTRY_PREVIEW_CASH_BELOW_ZERO_REQUEST = readGardenEntryInput(
  gardensEntryPreviewCashBelowZeroRequest,
);
export const GARDENS_ENTRY_PREVIEW_INSTALLMENT = readGardenEntryPreview(
  gardensEntryPreviewInstallment,
);
export const GARDENS_ENTRY_PREVIEW_INSTALLMENT_REQUEST = readGardenEntryInput(
  gardensEntryPreviewInstallmentRequest,
);
export const GARDENS_ENTRY_PREVIEW_OTHER_SALE = readGardenEntryPreview(
  gardensEntryPreviewOtherSale,
);
export const GARDENS_ENTRY_PREVIEW_OTHER_SALE_REQUEST = readGardenEntryInput(
  gardensEntryPreviewOtherSaleRequest,
);
export const GARDENS_ENTRY_SAVE_CASH_BELOW_ZERO = readUiError(gardensEntrySaveCashBelowZero);
export const GARDENS_ENTRY_SAVE_CASH_BELOW_ZERO_REQUEST = readGardenEntryInput(
  gardensEntrySaveCashBelowZeroRequest,
);
export const GARDENS_ENTRY_SAVE_COST = readGardenEntrySaved(gardensEntrySaveCost);
export const GARDENS_ENTRY_SAVE_COST_REQUEST = readGardenEntryInput(gardensEntrySaveCostRequest);
export const GARDENS_ENTRY_SAVE_DATE_BEFORE_GARDEN = readUiError(gardensEntrySaveDateBeforeGarden);
export const GARDENS_ENTRY_SAVE_DATE_BEFORE_GARDEN_REQUEST = readGardenEntryInput(
  gardensEntrySaveDateBeforeGardenRequest,
);
export const GARDENS_ENTRY_SAVE_GARDEN_CLOSED = readUiError(gardensEntrySaveGardenClosed);
export const GARDENS_ENTRY_SAVE_GARDEN_CLOSED_REQUEST = readGardenEntryInput(
  gardensEntrySaveGardenClosedRequest,
);
export const GARDENS_ENTRY_SAVE_INSTALLMENT = readGardenEntrySaved(gardensEntrySaveInstallment);
export const GARDENS_ENTRY_SAVE_INSTALLMENT_REQUEST = readGardenEntryInput(
  gardensEntrySaveInstallmentRequest,
);
export const GARDENS_ENTRY_SAVE_OTHER_SALE = readGardenEntrySaved(gardensEntrySaveOtherSale);
export const GARDENS_ENTRY_SAVE_OTHER_SALE_REQUEST = readGardenEntryInput(
  gardensEntrySaveOtherSaleRequest,
);
export const GARDENS_GET_CLOSED = readGardenDetail(gardensGetClosed);
export const GARDENS_GET_CLOSED_LOSS = readGardenDetail(gardensGetClosedLoss);
export const GARDENS_GET_OPEN = readGardenDetail(gardensGetOpen);
export const GARDENS_GET_OPEN_REQUEST = readGardenGetInput(gardensGetOpenRequest);
export const GARDENS_GET_REOPENED = readGardenDetail(gardensGetReopened);
export const GARDENS_LIST_ALL = readGardensList(gardensListAll);
export const GARDENS_LIST_NO_ENTRY = readGardensList(gardensListNoEntry);
export const GARDENS_LIST_OPEN_PAGED = readGardensList(gardensListOpenPaged);
export const GARDENS_LIST_OPEN_PAGED_REQUEST = readGardensListInput(gardensListOpenPagedRequest);
export const GARDENS_REOPEN_E2E40 = readGardenReopened(gardensReopenE2e40);
export const GARDENS_REOPEN_E2E40_REQUEST = readGardenReopenInput(gardensReopenE2e40Request);
export const GARDENS_REOPEN_NOT_CLOSED = readUiError(gardensReopenNotClosed);
export const GARDENS_REOPEN_NOT_CLOSED_REQUEST = readGardenReopenInput(
  gardensReopenNotClosedRequest,
);
export const GARDENS_SET_PARTNERS_LOCKED = readUiError(gardensSetPartnersLocked);
export const GARDENS_SET_PARTNERS_LOCKED_REQUEST = readGardenSetPartnersInput(
  gardensSetPartnersLockedRequest,
);
export const GARDENS_SET_PARTNERS_NOT_100 = readUiError(gardensSetPartnersNot100);
export const GARDENS_SET_PARTNERS_NOT_100_REQUEST = readGardenSetPartnersInput(
  gardensSetPartnersNot100Request,
);
export const GARDENS_SET_PARTNERS_THREE = readGardenRow(gardensSetPartnersThree);
export const GARDENS_SET_PARTNERS_THREE_REQUEST = readGardenSetPartnersInput(
  gardensSetPartnersThreeRequest,
);
export const GARDENS_SUMMARY_AFTER_CLOSE = readGardensSummary(gardensSummaryAfterClose);
export const GARDENS_SUMMARY_OPEN = readGardensSummary(gardensSummaryOpen);
export const PARTNERS_OVERVIEW_AFTER_GARDEN_CLOSE = readPartnerOverview(
  partnersOverviewAfterGardenClose,
);
export const PARTNERS_OVERVIEW_WITH_GARDEN_CARDS = readPartnerOverview(
  partnersOverviewWithGardenCards,
);
export const PARTNERS_STATEMENT_GARDEN_ALIF = readPartnerStatement(partnersStatementGardenAlif);
export const PARTNERS_STATEMENT_GARDEN_ALIF_REQUEST = readPartnersStatementInput(
  partnersStatementGardenAlifRequest,
);
