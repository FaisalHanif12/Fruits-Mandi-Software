// The real answers and requests of the backend for plan 20 (the report engine and the Accounts
// reports). The backend wrote them in its own test (backend/crates/services/tests/
// fixtures); they are copied here, never edited, and read as the generated types. Money in them is
// paisa, as on the wire. A reader takes `unknown`, because the money in a binding is a branded type
// that a plain number is not. The ids in the requests are those of the worked day in a fresh test
// database (اکرم is account 20).

import type { FindPeopleInput } from '../../src/bindings/FindPeopleInput';
import type { PukhtaPersonHit } from '../../src/bindings/PukhtaPersonHit';
import type { ReportCatalogEntry } from '../../src/bindings/ReportCatalogEntry';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportPrintInput } from '../../src/bindings/ReportPrintInput';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import type { UiError } from '../../src/bindings/UiError';
import catalog from '../fixtures/reports_catalog.json';
import printR13AkramUr from '../fixtures/reports_print_data_r13_akram_ur.json';
import printR13AkramUrRequest from '../fixtures/reports_print_data_r13_akram_ur_request.json';
import printR18En from '../fixtures/reports_print_data_r18_en.json';
import printR18EnRequest from '../fixtures/reports_print_data_r18_en_request.json';
import r11BadDates from '../fixtures/reports_run_r11_bad_dates.json';
import r11BadDatesRequest from '../fixtures/reports_run_r11_bad_dates_request.json';
import r11Today from '../fixtures/reports_run_r11_today.json';
import r11TodayRequest from '../fixtures/reports_run_r11_today_request.json';
import r13Akram from '../fixtures/reports_run_r13_akram.json';
import r13AkramRequest from '../fixtures/reports_run_r13_akram_request.json';
import r13NoParty from '../fixtures/reports_run_r13_no_party.json';
import r13NoPartyRequest from '../fixtures/reports_run_r13_no_party_request.json';
import r13Rahim from '../fixtures/reports_run_r13_rahim_from_10_10.json';
import r13RahimRequest from '../fixtures/reports_run_r13_rahim_from_10_10_request.json';
import r14Balance from '../fixtures/reports_run_r14_akram_balance_only.json';
import r14BalanceRequest from '../fixtures/reports_run_r14_akram_balance_only_request.json';
import r14Detail from '../fixtures/reports_run_r14_akram_detail.json';
import r14DetailRequest from '../fixtures/reports_run_r14_akram_detail_request.json';
import r16Page1 from '../fixtures/reports_run_r16_page1.json';
import r16Page1Request from '../fixtures/reports_run_r16_page1_request.json';
import r17 from '../fixtures/reports_run_r17.json';
import r17Request from '../fixtures/reports_run_r17_request.json';
import r18AsOf from '../fixtures/reports_run_r18_as_of_09_10.json';
import r18AsOfRequest from '../fixtures/reports_run_r18_as_of_09_10_request.json';
import r18NotTaken from '../fixtures/reports_run_r18_filter_not_taken.json';
import r18NotTakenRequest from '../fixtures/reports_run_r18_filter_not_taken_request.json';
import r18Page1 from '../fixtures/reports_run_r18_page1.json';
import r18Page1Request from '../fixtures/reports_run_r18_page1_request.json';
import r18Page2 from '../fixtures/reports_run_r18_page2.json';
import r18Page2Request from '../fixtures/reports_run_r18_page2_request.json';
import r19 from '../fixtures/reports_run_r19.json';
import r19Request from '../fixtures/reports_run_r19_request.json';
import r2All from '../fixtures/reports_run_r2_all.json';
import r2AllRequest from '../fixtures/reports_run_r2_all_request.json';
import r2Page2 from '../fixtures/reports_run_r2_page2.json';
import r2Page2Request from '../fixtures/reports_run_r2_page2_request.json';
import r2Akram from '../fixtures/reports_run_r2_akram.json';
import r2AkramRequest from '../fixtures/reports_run_r2_akram_request.json';
import r3Cash from '../fixtures/reports_run_r3_cash.json';
import r3CashRequest from '../fixtures/reports_run_r3_cash_request.json';
import r4Credit from '../fixtures/reports_run_r4_credit.json';
import r4CreditRequest from '../fixtures/reports_run_r4_credit_request.json';
import r6Lots from '../fixtures/reports_run_r6_lots.json';
import r6LotsRequest from '../fixtures/reports_run_r6_lots_request.json';
import r7Items from '../fixtures/reports_run_r7_items.json';
import r7ItemsRequest from '../fixtures/reports_run_r7_items_request.json';
import r8Adda from '../fixtures/reports_run_r8_adda.json';
import r8AddaRequest from '../fixtures/reports_run_r8_adda_request.json';
import r9All from '../fixtures/reports_run_r9_all.json';
import r9AllRequest from '../fixtures/reports_run_r9_all_request.json';
import r9Due from '../fixtures/reports_run_r9_due.json';
import r9DueRequest from '../fixtures/reports_run_r9_due_request.json';
import r27Series from '../fixtures/reports_run_r27_series.json';
import r27SeriesRequest from '../fixtures/reports_run_r27_series_request.json';
import r20Dues from '../fixtures/reports_run_r20_dues.json';
import r20DuesRequest from '../fixtures/reports_run_r20_dues_request.json';
import r20AsOf1010 from '../fixtures/reports_run_r20_as_of_10_10.json';
import r20AsOf1010Request from '../fixtures/reports_run_r20_as_of_10_10_request.json';
import r21ByCity from '../fixtures/reports_run_r21_by_city.json';
import r21ByCityRequest from '../fixtures/reports_run_r21_by_city_request.json';
import r22ByCollector from '../fixtures/reports_run_r22_by_collector.json';
import r22ByCollectorRequest from '../fixtures/reports_run_r22_by_collector_request.json';
import r24Payables from '../fixtures/reports_run_r24_payables.json';
import r24PayablesRequest from '../fixtures/reports_run_r24_payables_request.json';
import r26Advances from '../fixtures/reports_run_r26_advances.json';
import r26AdvancesRequest from '../fixtures/reports_run_r26_advances_request.json';
import r28ByDay from '../fixtures/reports_run_r28_by_day.json';
import r28ByDayRequest from '../fixtures/reports_run_r28_by_day_request.json';
import r28ByMonth from '../fixtures/reports_run_r28_by_month.json';
import r28ByMonthRequest from '../fixtures/reports_run_r28_by_month_request.json';
import r30Profit from '../fixtures/reports_run_r30_profit.json';
import r30ProfitRequest from '../fixtures/reports_run_r30_profit_request.json';
import r9NotAOneTimeAccount from '../fixtures/reports_run_r9_not_a_one_time_account.json';
import r9NotAOneTimeAccountRequest from '../fixtures/reports_run_r9_not_a_one_time_account_request.json';
import r2UnknownSeller from '../fixtures/reports_run_r2_unknown_seller.json';
import r2UnknownSellerRequest from '../fixtures/reports_run_r2_unknown_seller_request.json';
import printR7En from '../fixtures/reports_print_data_r7_en.json';
import printR7EnRequest from '../fixtures/reports_print_data_r7_en_request.json';
import printR20Ur from '../fixtures/reports_print_data_r20_ur.json';
import printR20UrRequest from '../fixtures/reports_print_data_r20_ur_request.json';
import printR28MonthEn from '../fixtures/reports_print_data_r28_month_en.json';
import printR28MonthEnRequest from '../fixtures/reports_print_data_r28_month_en_request.json';
import printR9DueEn from '../fixtures/reports_print_data_r9_due_en.json';
import printR9DueEnRequest from '../fixtures/reports_print_data_r9_due_en_request.json';
import peopleShahid from '../fixtures/reports_find_people_shahid.json';
import peopleShahidRequest from '../fixtures/reports_find_people_shahid_request.json';
import peopleNone from '../fixtures/reports_find_people_none.json';
import peopleNoneRequest from '../fixtures/reports_find_people_none_request.json';
import peopleLimitTooBig from '../fixtures/reports_find_people_limit_too_big.json';
import peopleLimitTooBigRequest from '../fixtures/reports_find_people_limit_too_big_request.json';

function readCatalog(value: unknown): ReportCatalogEntry[] {
  return value as ReportCatalogEntry[];
}
function readPage(value: unknown): ReportPage {
  return value as ReportPage;
}
function readRunInput(value: unknown): ReportRunInput {
  return value as ReportRunInput;
}
function readPrint(value: unknown): ReportPrintData {
  return value as ReportPrintData;
}
function readPrintInput(value: unknown): ReportPrintInput {
  return value as ReportPrintInput;
}
function readRefusal(value: unknown): UiError {
  return value as UiError;
}
function readPeople(value: unknown): PukhtaPersonHit[] {
  return value as PukhtaPersonHit[];
}
function readFindInput(value: unknown): FindPeopleInput {
  return value as FindPeopleInput;
}

/**
 * Reports the backend already lists (and has fixtures for) that this build has no screen or words
 * for yet: round 8 (R10 stock, R15 vouchers, R23 receipts, R25 payments, R33 trips, R34 day summary).
 * The tests use the reports this build has built, so a report without words never shows its raw
 * message ID on the reports home. When a round lands, its ids are removed from this list, and the
 * tests that guard it (the catalogue and the words) start to cover it.
 */
export const ROUND_8_PENDING: ReadonlySet<string> = new Set([
  'R10',
  'R15',
  'R23',
  'R25',
  'R33',
  'R34',
]);

/** The catalogue as the backend sent it, with only the reports this build has built. */
export const CATALOG = readCatalog(catalog).filter((entry) => !ROUND_8_PENDING.has(entry.id));

/** The whole catalogue the backend sent, built or not (for the guard that nothing slips in). */
export const BACKEND_CATALOG = readCatalog(catalog);

export const R11_TODAY = readPage(r11Today);
export const R11_TODAY_REQUEST = readRunInput(r11TodayRequest);
export const R11_BAD_DATES = readRefusal(r11BadDates);
export const R11_BAD_DATES_REQUEST = readRunInput(r11BadDatesRequest);

export const R13_AKRAM = readPage(r13Akram);
export const R13_AKRAM_REQUEST = readRunInput(r13AkramRequest);
export const R13_RAHIM = readPage(r13Rahim);
export const R13_RAHIM_REQUEST = readRunInput(r13RahimRequest);
export const R13_NO_PARTY = readRefusal(r13NoParty);
export const R13_NO_PARTY_REQUEST = readRunInput(r13NoPartyRequest);

export const R14_BALANCE = readPage(r14Balance);
export const R14_BALANCE_REQUEST = readRunInput(r14BalanceRequest);
export const R14_DETAIL = readPage(r14Detail);
export const R14_DETAIL_REQUEST = readRunInput(r14DetailRequest);

export const R16_PAGE1 = readPage(r16Page1);
export const R16_PAGE1_REQUEST = readRunInput(r16Page1Request);
export const R17 = readPage(r17);
export const R17_REQUEST = readRunInput(r17Request);

export const R18_PAGE1 = readPage(r18Page1);
export const R18_PAGE1_REQUEST = readRunInput(r18Page1Request);
export const R18_PAGE2 = readPage(r18Page2);
export const R18_PAGE2_REQUEST = readRunInput(r18Page2Request);
export const R18_AS_OF = readPage(r18AsOf);
export const R18_AS_OF_REQUEST = readRunInput(r18AsOfRequest);
export const R18_NOT_TAKEN = readRefusal(r18NotTaken);
export const R18_NOT_TAKEN_REQUEST = readRunInput(r18NotTakenRequest);

export const R19 = readPage(r19);
export const R19_REQUEST = readRunInput(r19Request);

export const PRINT_R18_EN = readPrint(printR18En);
export const PRINT_R18_EN_REQUEST = readPrintInput(printR18EnRequest);
export const PRINT_R13_AKRAM_UR = readPrint(printR13AkramUr);
export const PRINT_R13_AKRAM_UR_REQUEST = readPrintInput(printR13AkramUrRequest);

/** The whole of R18 in one page: the two pages of the backend's fixtures joined (17 rows). */
export const R18_FULL: ReportPage = {
  ...R18_PAGE1,
  rows: [...R18_PAGE1.rows, ...R18_PAGE2.rows],
  offset: 0,
  limit: 0,
};

// ---- Round 4 (step F3): the Sales, Collections and Business reports, and the people search. ----
export const R2_ALL = readPage(r2All);
export const R2_ALL_REQUEST = readRunInput(r2AllRequest);
export const R2_PAGE2 = readPage(r2Page2);
export const R2_PAGE2_REQUEST = readRunInput(r2Page2Request);
export const R2_AKRAM = readPage(r2Akram);
export const R2_AKRAM_REQUEST = readRunInput(r2AkramRequest);
export const R3_CASH = readPage(r3Cash);
export const R3_CASH_REQUEST = readRunInput(r3CashRequest);
export const R4_CREDIT = readPage(r4Credit);
export const R4_CREDIT_REQUEST = readRunInput(r4CreditRequest);
export const R6_LOTS = readPage(r6Lots);
export const R6_LOTS_REQUEST = readRunInput(r6LotsRequest);
export const R7_ITEMS = readPage(r7Items);
export const R7_ITEMS_REQUEST = readRunInput(r7ItemsRequest);
export const R8_ADDA = readPage(r8Adda);
export const R8_ADDA_REQUEST = readRunInput(r8AddaRequest);
export const R9_ALL = readPage(r9All);
export const R9_ALL_REQUEST = readRunInput(r9AllRequest);
export const R9_DUE = readPage(r9Due);
export const R9_DUE_REQUEST = readRunInput(r9DueRequest);
export const R27_SERIES = readPage(r27Series);
export const R27_SERIES_REQUEST = readRunInput(r27SeriesRequest);
export const R20_DUES = readPage(r20Dues);
export const R20_DUES_REQUEST = readRunInput(r20DuesRequest);
export const R20_AS_OF_10_10 = readPage(r20AsOf1010);
export const R20_AS_OF_10_10_REQUEST = readRunInput(r20AsOf1010Request);
export const R21_BY_CITY = readPage(r21ByCity);
export const R21_BY_CITY_REQUEST = readRunInput(r21ByCityRequest);
export const R22_BY_COLLECTOR = readPage(r22ByCollector);
export const R22_BY_COLLECTOR_REQUEST = readRunInput(r22ByCollectorRequest);
export const R24_PAYABLES = readPage(r24Payables);
export const R24_PAYABLES_REQUEST = readRunInput(r24PayablesRequest);
export const R26_ADVANCES = readPage(r26Advances);
export const R26_ADVANCES_REQUEST = readRunInput(r26AdvancesRequest);
export const R28_BY_DAY = readPage(r28ByDay);
export const R28_BY_DAY_REQUEST = readRunInput(r28ByDayRequest);
export const R28_BY_MONTH = readPage(r28ByMonth);
export const R28_BY_MONTH_REQUEST = readRunInput(r28ByMonthRequest);
export const R30_PROFIT = readPage(r30Profit);
export const R30_PROFIT_REQUEST = readRunInput(r30ProfitRequest);
export const R9_NOT_A_ONE_TIME_ACCOUNT = readRefusal(r9NotAOneTimeAccount);
export const R9_NOT_A_ONE_TIME_ACCOUNT_REQUEST = readRunInput(r9NotAOneTimeAccountRequest);
export const R2_UNKNOWN_SELLER = readRefusal(r2UnknownSeller);
export const R2_UNKNOWN_SELLER_REQUEST = readRunInput(r2UnknownSellerRequest);
export const PRINT_R7_EN = readPrint(printR7En);
export const PRINT_R7_EN_REQUEST = readPrintInput(printR7EnRequest);
export const PRINT_R20_UR = readPrint(printR20Ur);
export const PRINT_R20_UR_REQUEST = readPrintInput(printR20UrRequest);
export const PRINT_R28_MONTH_EN = readPrint(printR28MonthEn);
export const PRINT_R28_MONTH_EN_REQUEST = readPrintInput(printR28MonthEnRequest);
export const PRINT_R9_DUE_EN = readPrint(printR9DueEn);
export const PRINT_R9_DUE_EN_REQUEST = readPrintInput(printR9DueEnRequest);
export const PEOPLE_SHAHID = readPeople(peopleShahid);
export const PEOPLE_SHAHID_REQUEST = readFindInput(peopleShahidRequest);
export const PEOPLE_NONE = readPeople(peopleNone);
export const PEOPLE_NONE_REQUEST = readFindInput(peopleNoneRequest);
export const PEOPLE_LIMIT_TOO_BIG = readRefusal(peopleLimitTooBig);
export const PEOPLE_LIMIT_TOO_BIG_REQUEST = readFindInput(peopleLimitTooBigRequest);
