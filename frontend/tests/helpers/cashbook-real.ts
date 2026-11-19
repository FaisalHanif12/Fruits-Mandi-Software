// The real answers and requests of the backend for plan 17 (T4, cash book, خرچہ, partner takes,
// repeat last month, ادھار رقم and the overdue list). The backend wrote them in its own test
// (backend/crates/services/tests/day_money_fixtures.rs); they are copied here, never edited, and
// read as the generated types. Money in them is paisa, as on the wire. The story is the PRD worked
// day of 10-10-2026 with Aslam's loan of 5,000 (E2E-30) and Partner A's 2,000 (E2E-29).

import assistAttentionOverdue from '../fixtures/assist_attention_overdue.json';
import cashbookGetDay from '../fixtures/cashbook_get_day.json';
import cashbookGetDayRequest from '../fixtures/cashbook_get_day_request.json';
import cashbookGetSecondPage from '../fixtures/cashbook_get_second_page.json';
import cashbookGetSecondPageRequest from '../fixtures/cashbook_get_second_page_request.json';
import lentList from '../fixtures/lent_list.json';
import lentListRequest from '../fixtures/lent_list_request.json';
import lentPersonCreate from '../fixtures/lent_person_create.json';
import lentPersonCreateRequest from '../fixtures/lent_person_create_request.json';
import lentReport from '../fixtures/lent_report.json';
import lentReportRequest from '../fixtures/lent_report_request.json';
import mastersExpenseHeads from '../fixtures/masters_expense_heads.json';
import partnerTakes from '../fixtures/partner_takes.json';
import partnerTakesRequest from '../fixtures/partner_takes_request.json';
import vouchersPreviewExpensePersonal from '../fixtures/vouchers_preview_expense_personal.json';
import vouchersPreviewExpensePersonalRequest from '../fixtures/vouchers_preview_expense_personal_request.json';
import vouchersPreviewExpenseShop from '../fixtures/vouchers_preview_expense_shop.json';
import vouchersPreviewExpenseShopRequest from '../fixtures/vouchers_preview_expense_shop_request.json';
import vouchersRepeatLastMonthPreview from '../fixtures/vouchers_repeat_last_month_preview.json';
import vouchersRepeatLastMonthPreviewRequest from '../fixtures/vouchers_repeat_last_month_preview_request.json';
import vouchersSaveMoneyLent from '../fixtures/vouchers_save_money_lent.json';
import vouchersSaveMoneyLentRequest from '../fixtures/vouchers_save_money_lent_request.json';
import vouchersSaveMoneyLentReturnByBeforeEntry from '../fixtures/vouchers_save_money_lent_return_by_before_entry.json';
import vouchersSaveMoneyLentReturnByBeforeEntryRequest from '../fixtures/vouchers_save_money_lent_return_by_before_entry_request.json';
import vouchersSaveMoneyLentReturnCash from '../fixtures/vouchers_save_money_lent_return_cash.json';
import vouchersSaveMoneyLentReturnCashRequest from '../fixtures/vouchers_save_money_lent_return_cash_request.json';
import vouchersSaveMoneyLentReturnSalary from '../fixtures/vouchers_save_money_lent_return_salary.json';
import vouchersSaveMoneyLentReturnSalaryRequest from '../fixtures/vouchers_save_money_lent_return_salary_request.json';
import vouchersSavePersonalExpense from '../fixtures/vouchers_save_personal_expense.json';
import vouchersSavePersonalExpenseCashConfirm from '../fixtures/vouchers_save_personal_expense_cash_confirm.json';
import vouchersSavePersonalExpenseCashConfirmRequest from '../fixtures/vouchers_save_personal_expense_cash_confirm_request.json';
import vouchersSavePersonalExpenseRequest from '../fixtures/vouchers_save_personal_expense_request.json';
import vouchersSavePersonalReturn from '../fixtures/vouchers_save_personal_return.json';
import vouchersSavePersonalReturnExceeds from '../fixtures/vouchers_save_personal_return_exceeds.json';
import vouchersSavePersonalReturnExceedsRequest from '../fixtures/vouchers_save_personal_return_exceeds_request.json';
import vouchersSavePersonalReturnRequest from '../fixtures/vouchers_save_personal_return_request.json';
import vouchersSaveShopExpense from '../fixtures/vouchers_save_shop_expense.json';
import vouchersSaveShopExpenseMistakes from '../fixtures/vouchers_save_shop_expense_mistakes.json';
import vouchersSaveShopExpenseMistakesRequest from '../fixtures/vouchers_save_shop_expense_mistakes_request.json';
import vouchersSaveShopExpenseRequest from '../fixtures/vouchers_save_shop_expense_request.json';
import type { AttentionOverdue } from '../../src/bindings/AttentionOverdue';
import type { Cashbook } from '../../src/bindings/Cashbook';
import type { CashbookInput } from '../../src/bindings/CashbookInput';
import type { ExpenseHeads } from '../../src/bindings/ExpenseHeads';
import type { ExpensePreview } from '../../src/bindings/ExpensePreview';
import type { ExpensePreviewInput } from '../../src/bindings/ExpensePreviewInput';
import type { LentList } from '../../src/bindings/LentList';
import type { LentListInput } from '../../src/bindings/LentListInput';
import type { LentPerson } from '../../src/bindings/LentPerson';
import type { LentPersonCreateInput } from '../../src/bindings/LentPersonCreateInput';
import type { LentReport } from '../../src/bindings/LentReport';
import type { LentReportInput } from '../../src/bindings/LentReportInput';
import type { MoneyLentInput } from '../../src/bindings/MoneyLentInput';
import type { MoneyLentReturnInput } from '../../src/bindings/MoneyLentReturnInput';
import type { PartnerTakes } from '../../src/bindings/PartnerTakes';
import type { PartnerTakesInput } from '../../src/bindings/PartnerTakesInput';
import type { PersonalExpenseInput } from '../../src/bindings/PersonalExpenseInput';
import type { PersonalReturnInput } from '../../src/bindings/PersonalReturnInput';
import type { RepeatLastMonth } from '../../src/bindings/RepeatLastMonth';
import type { RepeatLastMonthInput } from '../../src/bindings/RepeatLastMonthInput';
import type { ShopExpenseInput } from '../../src/bindings/ShopExpenseInput';
import type { UiError } from '../../src/bindings/UiError';
import type { VoucherSaved } from '../../src/bindings/VoucherSaved';

// A fixture file is the backend's own JSON; each is read as its generated type.
function readAttentionOverdue(value: unknown): AttentionOverdue {
  return value as AttentionOverdue;
}
function readCashbook(value: unknown): Cashbook {
  return value as Cashbook;
}
function readCashbookInput(value: unknown): CashbookInput {
  return value as CashbookInput;
}
function readExpenseHeads(value: unknown): ExpenseHeads {
  return value as ExpenseHeads;
}
function readExpensePreview(value: unknown): ExpensePreview {
  return value as ExpensePreview;
}
function readExpensePreviewInput(value: unknown): ExpensePreviewInput {
  return value as ExpensePreviewInput;
}
function readLentList(value: unknown): LentList {
  return value as LentList;
}
function readLentListInput(value: unknown): LentListInput {
  return value as LentListInput;
}
function readLentPerson(value: unknown): LentPerson {
  return value as LentPerson;
}
function readLentPersonCreateInput(value: unknown): LentPersonCreateInput {
  return value as LentPersonCreateInput;
}
function readLentReport(value: unknown): LentReport {
  return value as LentReport;
}
function readLentReportInput(value: unknown): LentReportInput {
  return value as LentReportInput;
}
function readMoneyLentInput(value: unknown): MoneyLentInput {
  return value as MoneyLentInput;
}
function readMoneyLentReturnInput(value: unknown): MoneyLentReturnInput {
  return value as MoneyLentReturnInput;
}
function readPartnerTakes(value: unknown): PartnerTakes {
  return value as PartnerTakes;
}
function readPartnerTakesInput(value: unknown): PartnerTakesInput {
  return value as PartnerTakesInput;
}
function readPersonalExpenseInput(value: unknown): PersonalExpenseInput {
  return value as PersonalExpenseInput;
}
function readPersonalReturnInput(value: unknown): PersonalReturnInput {
  return value as PersonalReturnInput;
}
function readRepeatLastMonth(value: unknown): RepeatLastMonth {
  return value as RepeatLastMonth;
}
function readRepeatLastMonthInput(value: unknown): RepeatLastMonthInput {
  return value as RepeatLastMonthInput;
}
function readShopExpenseInput(value: unknown): ShopExpenseInput {
  return value as ShopExpenseInput;
}
function readUiError(value: unknown): UiError {
  return value as UiError;
}
function readVoucherSaved(value: unknown): VoucherSaved {
  return value as VoucherSaved;
}

// Answers
export const CASHBOOK_DAY: Cashbook = readCashbook(cashbookGetDay);
export const CASHBOOK_SECOND_PAGE: Cashbook = readCashbook(cashbookGetSecondPage);
export const PREVIEW_SHOP: ExpensePreview = readExpensePreview(vouchersPreviewExpenseShop);
export const PREVIEW_PERSONAL: ExpensePreview = readExpensePreview(vouchersPreviewExpensePersonal);
export const SAVED_SHOP: VoucherSaved = readVoucherSaved(vouchersSaveShopExpense);
export const SAVED_PERSONAL: VoucherSaved = readVoucherSaved(vouchersSavePersonalExpense);
export const SAVED_PERSONAL_RETURN: VoucherSaved = readVoucherSaved(vouchersSavePersonalReturn);
export const SAVED_LENT: VoucherSaved = readVoucherSaved(vouchersSaveMoneyLent);
export const SAVED_LENT_RETURN_CASH: VoucherSaved = readVoucherSaved(
  vouchersSaveMoneyLentReturnCash,
);
export const SAVED_LENT_RETURN_SALARY: VoucherSaved = readVoucherSaved(
  vouchersSaveMoneyLentReturnSalary,
);
export const REFUSED_SHOP_MISTAKES: UiError = readUiError(vouchersSaveShopExpenseMistakes);
export const REFUSED_PERSONAL_CASH_CONFIRM: UiError = readUiError(
  vouchersSavePersonalExpenseCashConfirm,
);
export const REFUSED_PERSONAL_RETURN_EXCEEDS: UiError = readUiError(
  vouchersSavePersonalReturnExceeds,
);
export const REFUSED_RETURN_BY_BEFORE_ENTRY: UiError = readUiError(
  vouchersSaveMoneyLentReturnByBeforeEntry,
);
export const PARTNER_TAKES: PartnerTakes = readPartnerTakes(partnerTakes);
export const REPEAT_LAST_MONTH: RepeatLastMonth = readRepeatLastMonth(
  vouchersRepeatLastMonthPreview,
);
export const EXPENSE_HEADS: ExpenseHeads = readExpenseHeads(mastersExpenseHeads);
export const ATTENTION_OVERDUE: AttentionOverdue = readAttentionOverdue(assistAttentionOverdue);
export const LENT_LIST: LentList = readLentList(lentList);
export const LENT_PERSON: LentPerson = readLentPerson(lentPersonCreate);
export const LENT_REPORT: LentReport = readLentReport(lentReport);

// Requests: exactly the `input` each command took
export const REQUEST_CASHBOOK_DAY: CashbookInput = readCashbookInput(cashbookGetDayRequest);
export const REQUEST_CASHBOOK_SECOND_PAGE: CashbookInput = readCashbookInput(
  cashbookGetSecondPageRequest,
);
export const REQUEST_PREVIEW_SHOP: ExpensePreviewInput = readExpensePreviewInput(
  vouchersPreviewExpenseShopRequest,
);
export const REQUEST_PREVIEW_PERSONAL: ExpensePreviewInput = readExpensePreviewInput(
  vouchersPreviewExpensePersonalRequest,
);
export const REQUEST_SAVE_SHOP: ShopExpenseInput = readShopExpenseInput(
  vouchersSaveShopExpenseRequest,
);
export const REQUEST_SAVE_SHOP_MISTAKES: ShopExpenseInput = readShopExpenseInput(
  vouchersSaveShopExpenseMistakesRequest,
);
export const REQUEST_SAVE_PERSONAL: PersonalExpenseInput = readPersonalExpenseInput(
  vouchersSavePersonalExpenseRequest,
);
export const REQUEST_SAVE_PERSONAL_CASH_CONFIRM: PersonalExpenseInput = readPersonalExpenseInput(
  vouchersSavePersonalExpenseCashConfirmRequest,
);
export const REQUEST_SAVE_PERSONAL_RETURN: PersonalReturnInput = readPersonalReturnInput(
  vouchersSavePersonalReturnRequest,
);
export const REQUEST_SAVE_PERSONAL_RETURN_EXCEEDS: PersonalReturnInput = readPersonalReturnInput(
  vouchersSavePersonalReturnExceedsRequest,
);
export const REQUEST_SAVE_LENT: MoneyLentInput = readMoneyLentInput(vouchersSaveMoneyLentRequest);
export const REQUEST_SAVE_LENT_BEFORE_ENTRY: MoneyLentInput = readMoneyLentInput(
  vouchersSaveMoneyLentReturnByBeforeEntryRequest,
);
export const REQUEST_SAVE_LENT_RETURN_CASH: MoneyLentReturnInput = readMoneyLentReturnInput(
  vouchersSaveMoneyLentReturnCashRequest,
);
export const REQUEST_SAVE_LENT_RETURN_SALARY: MoneyLentReturnInput = readMoneyLentReturnInput(
  vouchersSaveMoneyLentReturnSalaryRequest,
);
export const REQUEST_PARTNER_TAKES: PartnerTakesInput = readPartnerTakesInput(partnerTakesRequest);
export const REQUEST_REPEAT_LAST_MONTH: RepeatLastMonthInput = readRepeatLastMonthInput(
  vouchersRepeatLastMonthPreviewRequest,
);
export const REQUEST_LENT_LIST: LentListInput = readLentListInput(lentListRequest);
export const REQUEST_LENT_PERSON: LentPersonCreateInput =
  readLentPersonCreateInput(lentPersonCreateRequest);
export const REQUEST_LENT_REPORT: LentReportInput = readLentReportInput(lentReportRequest);
