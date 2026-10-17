// The real answers and requests of the backend for plan 15 (T4). The backend wrote them in its own
// test (backend/crates/services/tests/voucher_fixtures.rs); they are copied here, never edited, and
// read as the generated types. Money in them is paisa, as on the wire. The story is the PRD worked
// day of 10-10-2026: advance to Rahim 15,000, بکری 1, receipt 10,000 from Akram, payment 4,316 to
// Rahim, cash lent 3,000 to Akram, then the rest.

import advancesKarim from '../fixtures/assist_advances_karim.json';
import collectionSaturday from '../fixtures/assist_collection_list_saturday.json';
import collectionSaturdayRequest from '../fixtures/assist_collection_list_saturday_request.json';
import openBillsAkram from '../fixtures/assist_open_bills_akram.json';
import openBillsAkramRequest from '../fixtures/assist_open_bills_akram_request.json';
import openBillsTruncated from '../fixtures/assist_open_bills_truncated.json';
import openPayablesRahim from '../fixtures/assist_open_payables_rahim.json';
import openPayablesRahimRequest from '../fixtures/assist_open_payables_rahim_request.json';
import openPayablesTruncated from '../fixtures/assist_open_payables_truncated.json';
import makeRegular from '../fixtures/pukhta_make_regular_shahid.json';
import makeRegularRequest from '../fixtures/pukhta_make_regular_shahid_request.json';
import slipAdvanceUr from '../fixtures/reports_voucher_print_data_advance_ur.json';
import slipChequeEn from '../fixtures/reports_voucher_print_data_cheque_en.json';
import slipPaymentEn from '../fixtures/reports_voucher_print_data_payment_en.json';
import slipReceiptUr from '../fixtures/reports_voucher_print_data_receipt_ur.json';
import searchVoucherNumber from '../fixtures/search_global_voucher_number.json';
import cancelCashLent from '../fixtures/vouchers_cancel_cash_lent.json';
import cancelCashLentRequest from '../fixtures/vouchers_cancel_cash_lent_request.json';
import editReceipt from '../fixtures/vouchers_edit_receipt.json';
import editReceiptRequest from '../fixtures/vouchers_edit_receipt_request.json';
import getReceipt from '../fixtures/vouchers_get_receipt.json';
import listAll from '../fixtures/vouchers_list_all.json';
import listAllRequest from '../fixtures/vouchers_list_all_request.json';
import listPayments from '../fixtures/vouchers_list_payments.json';
import listPaymentsRequest from '../fixtures/vouchers_list_payments_request.json';
import previewAdvance from '../fixtures/vouchers_preview_advance.json';
import previewPaymentBuyer from '../fixtures/vouchers_preview_payment_buyer.json';
import previewPaymentCashBelowZero from '../fixtures/vouchers_preview_payment_cash_below_zero.json';
import previewPaymentCashBelowZeroRequest from '../fixtures/vouchers_preview_payment_cash_below_zero_request.json';
import previewPaymentNothingPayable from '../fixtures/vouchers_preview_payment_seller_nothing_payable.json';
import previewPaymentSeller from '../fixtures/vouchers_preview_payment_seller.json';
import previewReceiptBuyer from '../fixtures/vouchers_preview_receipt_buyer.json';
import previewReceiptBuyerRequest from '../fixtures/vouchers_preview_receipt_buyer_request.json';
import previewReceiptEdit from '../fixtures/vouchers_preview_receipt_edit.json';
import previewReceiptEditRequest from '../fixtures/vouchers_preview_receipt_edit_request.json';
import previewReceiptSellerReturn from '../fixtures/vouchers_preview_receipt_seller_return.json';
import saveAdvance from '../fixtures/vouchers_save_advance.json';
import saveAdvanceRequest from '../fixtures/vouchers_save_advance_request.json';
import savePayment from '../fixtures/vouchers_save_payment.json';
import savePaymentRequest from '../fixtures/vouchers_save_payment_request.json';
import savePaymentCashBelowZero from '../fixtures/vouchers_save_payment_cash_below_zero.json';
import savePaymentCashLent from '../fixtures/vouchers_save_payment_cash_lent.json';
import savePaymentCashLentRequest from '../fixtures/vouchers_save_payment_cash_lent_request.json';
import savePaymentNeedsReason from '../fixtures/vouchers_save_payment_to_buyer_needs_reason.json';
import saveReceipt from '../fixtures/vouchers_save_receipt.json';
import saveReceiptRequest from '../fixtures/vouchers_save_receipt_request.json';
import saveReceiptCheque from '../fixtures/vouchers_save_receipt_cheque.json';
import saveReceiptChequeRequest from '../fixtures/vouchers_save_receipt_cheque_request.json';
import saveReceiptReturnTooMuch from '../fixtures/vouchers_save_receipt_seller_return_too_much.json';
import saveReceiptsBatch from '../fixtures/vouchers_save_receipts_batch_round.json';
import saveReceiptsBatchRequest from '../fixtures/vouchers_save_receipts_batch_round_request.json';
import type { AdvanceInput } from '../../src/bindings/AdvanceInput';
import type { AdvancesInfo } from '../../src/bindings/AdvancesInfo';
import type { CollectionList } from '../../src/bindings/CollectionList';
import type { CollectionListInput } from '../../src/bindings/CollectionListInput';
import type { GlobalHit } from '../../src/bindings/GlobalHit';
import type { MadeRegular } from '../../src/bindings/MadeRegular';
import type { MakeRegularInput } from '../../src/bindings/MakeRegularInput';
import type { OpenBills } from '../../src/bindings/OpenBills';
import type { OpenBillsInput } from '../../src/bindings/OpenBillsInput';
import type { OpenPayables } from '../../src/bindings/OpenPayables';
import type { OpenPayablesInput } from '../../src/bindings/OpenPayablesInput';
import type { PaymentInput } from '../../src/bindings/PaymentInput';
import type { PrintData } from '../../src/bindings/PrintData';
import type { ReceiptInput } from '../../src/bindings/ReceiptInput';
import type { ReceiptsInput } from '../../src/bindings/ReceiptsInput';
import type { ReceiptsSaved } from '../../src/bindings/ReceiptsSaved';
import type { UiError } from '../../src/bindings/UiError';
import type { VoucherCancelInput } from '../../src/bindings/VoucherCancelInput';
import type { VoucherCancelled } from '../../src/bindings/VoucherCancelled';
import type { VoucherEditInput } from '../../src/bindings/VoucherEditInput';
import type { VoucherPreview } from '../../src/bindings/VoucherPreview';
import type { VoucherPreviewInput } from '../../src/bindings/VoucherPreviewInput';
import type { VoucherRow } from '../../src/bindings/VoucherRow';
import type { VoucherSaved } from '../../src/bindings/VoucherSaved';
import type { VouchersList } from '../../src/bindings/VouchersList';
import type { VouchersListInput } from '../../src/bindings/VouchersListInput';

// The fixture files are the backend's own JSON; each is read as its generated type.

function readVoucherPreview(value: unknown): VoucherPreview {
  return value as VoucherPreview;
}
function readVoucherSaved(value: unknown): VoucherSaved {
  return value as VoucherSaved;
}
function readVoucherCancelled(value: unknown): VoucherCancelled {
  return value as VoucherCancelled;
}
function readVoucherRow(value: unknown): VoucherRow {
  return value as VoucherRow;
}
function readVouchersList(value: unknown): VouchersList {
  return value as VouchersList;
}
function readOpenBills(value: unknown): OpenBills {
  return value as OpenBills;
}
function readOpenPayables(value: unknown): OpenPayables {
  return value as OpenPayables;
}
function readAdvancesInfo(value: unknown): AdvancesInfo {
  return value as AdvancesInfo;
}
function readCollectionList(value: unknown): CollectionList {
  return value as CollectionList;
}
function readReceiptsSaved(value: unknown): ReceiptsSaved {
  return value as ReceiptsSaved;
}
function readMadeRegular(value: unknown): MadeRegular {
  return value as MadeRegular;
}
function readPrintData(value: unknown): PrintData {
  return value as PrintData;
}
function readListGlobalHit(value: unknown): GlobalHit[] {
  return value as GlobalHit[];
}
function readUiError(value: unknown): UiError {
  return value as UiError;
}
function readReceiptInput(value: unknown): ReceiptInput {
  return value as ReceiptInput;
}
function readPaymentInput(value: unknown): PaymentInput {
  return value as PaymentInput;
}
function readAdvanceInput(value: unknown): AdvanceInput {
  return value as AdvanceInput;
}
function readVoucherEditInput(value: unknown): VoucherEditInput {
  return value as VoucherEditInput;
}
function readVouchersListInput(value: unknown): VouchersListInput {
  return value as VouchersListInput;
}
function readOpenBillsInput(value: unknown): OpenBillsInput {
  return value as OpenBillsInput;
}
function readReceiptsInput(value: unknown): ReceiptsInput {
  return value as ReceiptsInput;
}
function readMakeRegularInput(value: unknown): MakeRegularInput {
  return value as MakeRegularInput;
}

function readVoucherPreviewInput(value: unknown): VoucherPreviewInput {
  return value as VoucherPreviewInput;
}
function readVoucherCancelInput(value: unknown): VoucherCancelInput {
  return value as VoucherCancelInput;
}
function readOpenPayablesInput(value: unknown): OpenPayablesInput {
  return value as OpenPayablesInput;
}
function readCollectionListInput(value: unknown): CollectionListInput {
  return value as CollectionListInput;
}

// Answers
export const PREVIEW_RECEIPT_BUYER: VoucherPreview = readVoucherPreview(previewReceiptBuyer);
export const PREVIEW_PAYMENT_SELLER: VoucherPreview = readVoucherPreview(previewPaymentSeller);
export const PREVIEW_PAYMENT_BUYER: VoucherPreview = readVoucherPreview(previewPaymentBuyer);
export const PREVIEW_PAYMENT_NOTHING_PAYABLE: VoucherPreview = readVoucherPreview(
  previewPaymentNothingPayable,
);
export const PREVIEW_PAYMENT_CASH_BELOW_ZERO: VoucherPreview = readVoucherPreview(
  previewPaymentCashBelowZero,
);
export const PREVIEW_ADVANCE: VoucherPreview = readVoucherPreview(previewAdvance);
export const PREVIEW_RECEIPT_SELLER_RETURN: VoucherPreview = readVoucherPreview(
  previewReceiptSellerReturn,
);
export const PREVIEW_RECEIPT_EDIT: VoucherPreview = readVoucherPreview(previewReceiptEdit);
export const SAVED_RECEIPT: VoucherSaved = readVoucherSaved(saveReceipt);
export const SAVED_PAYMENT: VoucherSaved = readVoucherSaved(savePayment);
export const SAVED_ADVANCE: VoucherSaved = readVoucherSaved(saveAdvance);
export const SAVED_CASH_LENT: VoucherSaved = readVoucherSaved(savePaymentCashLent);
export const SAVED_CHEQUE: VoucherSaved = readVoucherSaved(saveReceiptCheque);
export const EDITED_RECEIPT: VoucherSaved = readVoucherSaved(editReceipt);
export const CANCELLED_CASH_LENT: VoucherCancelled = readVoucherCancelled(cancelCashLent);
export const GOT_RECEIPT: VoucherRow = readVoucherRow(getReceipt);
export const LIST_PAYMENTS: VouchersList = readVouchersList(listPayments);
export const LIST_ALL: VouchersList = readVouchersList(listAll);
export const OPEN_BILLS_AKRAM: OpenBills = readOpenBills(openBillsAkram);
export const OPEN_BILLS_TRUNCATED: OpenBills = readOpenBills(openBillsTruncated);
export const OPEN_PAYABLES_RAHIM: OpenPayables = readOpenPayables(openPayablesRahim);
export const OPEN_PAYABLES_TRUNCATED: OpenPayables = readOpenPayables(openPayablesTruncated);
export const ADVANCES_KARIM: AdvancesInfo = readAdvancesInfo(advancesKarim);
export const COLLECTION_SATURDAY: CollectionList = readCollectionList(collectionSaturday);
export const RECEIPTS_BATCH: ReceiptsSaved = readReceiptsSaved(saveReceiptsBatch);
export const MADE_REGULAR: MadeRegular = readMadeRegular(makeRegular);
export const SLIP_RECEIPT_UR: PrintData = readPrintData(slipReceiptUr);
export const SLIP_PAYMENT_EN: PrintData = readPrintData(slipPaymentEn);
export const SLIP_ADVANCE_UR: PrintData = readPrintData(slipAdvanceUr);
export const SLIP_CHEQUE_EN: PrintData = readPrintData(slipChequeEn);
export const GLOBAL_VOUCHER_NUMBER: GlobalHit[] = readListGlobalHit(searchVoucherNumber);

// Refusals, as `{ ok: false, errors, ref }`
export const REFUSED_NEEDS_REASON: UiError = readUiError(savePaymentNeedsReason);
export const REFUSED_RETURN_TOO_MUCH: UiError = readUiError(saveReceiptReturnTooMuch);
export const REFUSED_CASH_BELOW_ZERO: UiError = readUiError(savePaymentCashBelowZero);

// Requests: exactly the `input` each command took
export const REQUEST_SAVE_RECEIPT: ReceiptInput = readReceiptInput(saveReceiptRequest);
export const REQUEST_SAVE_RECEIPT_CHEQUE: ReceiptInput = readReceiptInput(saveReceiptChequeRequest);
export const REQUEST_SAVE_PAYMENT: PaymentInput = readPaymentInput(savePaymentRequest);
export const REQUEST_SAVE_CASH_LENT: PaymentInput = readPaymentInput(savePaymentCashLentRequest);
export const REQUEST_SAVE_ADVANCE: AdvanceInput = readAdvanceInput(saveAdvanceRequest);
export const REQUEST_PREVIEW_RECEIPT_BUYER: VoucherPreviewInput = readVoucherPreviewInput(
  previewReceiptBuyerRequest,
);
export const REQUEST_PREVIEW_RECEIPT_EDIT: VoucherPreviewInput =
  readVoucherPreviewInput(previewReceiptEditRequest);
export const REQUEST_PREVIEW_CASH_BELOW_ZERO: VoucherPreviewInput = readVoucherPreviewInput(
  previewPaymentCashBelowZeroRequest,
);
export const REQUEST_EDIT_RECEIPT: VoucherEditInput = readVoucherEditInput(editReceiptRequest);
export const REQUEST_CANCEL_CASH_LENT: VoucherCancelInput =
  readVoucherCancelInput(cancelCashLentRequest);
export const REQUEST_LIST_PAYMENTS: VouchersListInput = readVouchersListInput(listPaymentsRequest);
export const REQUEST_LIST_ALL: VouchersListInput = readVouchersListInput(listAllRequest);
export const REQUEST_OPEN_BILLS_AKRAM: OpenBillsInput = readOpenBillsInput(openBillsAkramRequest);
export const REQUEST_OPEN_PAYABLES_RAHIM: OpenPayablesInput =
  readOpenPayablesInput(openPayablesRahimRequest);
export const REQUEST_COLLECTION_SATURDAY: CollectionListInput =
  readCollectionListInput(collectionSaturdayRequest);
export const REQUEST_RECEIPTS_BATCH: ReceiptsInput = readReceiptsInput(saveReceiptsBatchRequest);
export const REQUEST_MAKE_REGULAR: MakeRegularInput = readMakeRegularInput(makeRegularRequest);
