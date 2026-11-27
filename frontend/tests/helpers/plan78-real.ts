// The real answers and requests of the backend for plan 78 (C1 load buttons, D3 rent relief and refund, B4
// advances in the partner statement). The backend wrote them in its own test
// (backend/crates/services/tests/plan78_fixtures.rs); they are copied here, never edited, and read as the generated
// types.

import type { GardenEntryInput } from '../../src/bindings/GardenEntryInput';
import type { GardenHeaderEditInput } from '../../src/bindings/GardenHeaderEditInput';
import type { GoodsSources } from '../../src/bindings/GoodsSources';
import type { GoodsToLoadInput } from '../../src/bindings/GoodsToLoadInput';
import type { DaySlipsPrintData } from '../../src/bindings/DaySlipsPrintData';
import type { PartnerStatementMonth } from '../../src/bindings/PartnerStatementMonth';
import type { ReadyGoods } from '../../src/bindings/ReadyGoods';
import type { SalePrintData } from '../../src/bindings/SalePrintData';
import type { StatementRows } from '../../src/bindings/StatementRows';
import type { TablePrintData } from '../../src/bindings/TablePrintData';
import type { UiError } from '../../src/bindings/UiError';
import adhariPrintEn from '../fixtures/reports_adhari_sheet_print_data_en.json';
import adhariPrintUr from '../fixtures/reports_adhari_sheet_print_data_ur.json';
import parchiEn from '../fixtures/reports_buyer_parchi_print_data_en.json';
import parchiUr from '../fixtures/reports_buyer_parchi_print_data_ur.json';
import salePrintEn from '../fixtures/reports_sale_print_data_en.json';
import salePrintUr from '../fixtures/reports_sale_print_data_ur.json';
import sellerBillsEn from '../fixtures/reports_seller_bills_print_data_en.json';
import sellerBillsUr from '../fixtures/reports_seller_bills_print_data_ur.json';
import statementRowsFromEleventh from '../fixtures/party_ledgers_statement_rows_from_11_10.json';
import statementRowsWhole from '../fixtures/party_ledgers_statement_rows_whole.json';
import statementAdvanceSeptember from '../fixtures/partners_statement_month_advance_2026-09.json';
import statementAdvanceOctober from '../fixtures/partners_statement_month_advance_2026-10.json';
import goodsSources from '../fixtures/bikri_goods_sources.json';
import goodsToLoadGarden from '../fixtures/bikri_goods_to_load_garden.json';
import goodsToLoadGardenRequest from '../fixtures/bikri_goods_to_load_garden_request.json';
import goodsToLoadTrip from '../fixtures/bikri_goods_to_load_trip.json';
import goodsToLoadTripRequest from '../fixtures/bikri_goods_to_load_trip_request.json';
import refundRequest from '../fixtures/gardens_entry_save_refund_request.json';
import refundTooMuch from '../fixtures/gardens_entry_save_refund_too_much.json';
import headerEditReasonRequired from '../fixtures/gardens_header_edit_save_reason_required.json';
import headerEditRelief from '../fixtures/gardens_header_edit_save_relief_request.json';
import headerEditSignWrong from '../fixtures/gardens_header_edit_save_sign_wrong.json';

// A fixture file as its generated type: the JSON is what the backend wrote, and the type is what the screens read.
function readGardenEntryInput(value: unknown): GardenEntryInput {
  return value as GardenEntryInput;
}
function readGardenHeaderEditInput(value: unknown): GardenHeaderEditInput {
  return value as GardenHeaderEditInput;
}
function readGoodsSources(value: unknown): GoodsSources {
  return value as GoodsSources;
}
function readGoodsToLoadInput(value: unknown): GoodsToLoadInput {
  return value as GoodsToLoadInput;
}
function readPartnerStatementMonth(value: unknown): PartnerStatementMonth {
  return value as PartnerStatementMonth;
}
function readDaySlipsPrintData(value: unknown): DaySlipsPrintData {
  return value as DaySlipsPrintData;
}
function readSalePrintData(value: unknown): SalePrintData {
  return value as SalePrintData;
}
function readStatementRows(value: unknown): StatementRows {
  return value as StatementRows;
}
function readTablePrintData(value: unknown): TablePrintData {
  return value as TablePrintData;
}
function readReadyGoods(value: unknown): ReadyGoods {
  return value as ReadyGoods;
}
function readUiError(value: unknown): UiError {
  return value as UiError;
}

export const GOODS_SOURCES = readGoodsSources(goodsSources);
export const GOODS_TO_LOAD_TRIP = readReadyGoods(goodsToLoadTrip);
export const GOODS_TO_LOAD_TRIP_REQUEST = readGoodsToLoadInput(goodsToLoadTripRequest);
export const GOODS_TO_LOAD_GARDEN = readReadyGoods(goodsToLoadGarden);
export const GOODS_TO_LOAD_GARDEN_REQUEST = readGoodsToLoadInput(goodsToLoadGardenRequest);

/** A garden's rent changed with its reason and the Owner's sign (relief from the owner, PRD FR-12.51). */
export const HEADER_EDIT_RELIEF_REQUEST = readGardenHeaderEditInput(headerEditRelief);
export const HEADER_EDIT_REASON_REQUIRED = readUiError(headerEditReasonRequired);
export const HEADER_EDIT_SIGN_WRONG = readUiError(headerEditSignWrong);

/** The owner pays money back after a relief (رعایت واپسی). */
export const REFUND_REQUEST = readGardenEntryInput(refundRequest);
export const REFUND_TOO_MUCH = readUiError(refundTooMuch);

/** Partner B's statement with the advance to a seller: it goes out in September and part of it comes back in October. */
export const STATEMENT_ADVANCE_SEPTEMBER = readPartnerStatementMonth(statementAdvanceSeptember);
export const STATEMENT_ADVANCE_OCTOBER = readPartnerStatementMonth(statementAdvanceOctober);

/** The prints of the worked day (Events 1 to 5, 10-10-26), as the backend writes them in its own test. */
export const PARCHI_EN = readDaySlipsPrintData(parchiEn);
export const PARCHI_UR = readDaySlipsPrintData(parchiUr);
export const SELLER_BILLS_EN = readDaySlipsPrintData(sellerBillsEn);
export const SELLER_BILLS_UR = readDaySlipsPrintData(sellerBillsUr);
export const SALE_PRINT_EN = readSalePrintData(salePrintEn);
export const SALE_PRINT_UR = readSalePrintData(salePrintUr);
export const ADHARI_PRINT_EN = readTablePrintData(adhariPrintEn);
export const ADHARI_PRINT_UR = readTablePrintData(adhariPrintUr);

/** Akram's statement, from the first day, with the opening balance as its first row, and from 11-10-26 on. */
export const STATEMENT_ROWS_WHOLE = readStatementRows(statementRowsWhole);
export const STATEMENT_ROWS_FROM_11_10 = readStatementRows(statementRowsFromEleventh);
