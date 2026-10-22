// The real answers and requests of the backend for plan 20 (the report engine and the Accounts
// reports). The backend wrote them in its own test (backend/crates/services/tests/
// fixtures); they are copied here, never edited, and read as the generated types. Money in them is
// paisa, as on the wire. A reader takes `unknown`, because the money in a binding is a branded type
// that a plain number is not. The ids in the requests are those of the worked day in a fresh test
// database (اکرم is account 20).

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

export const CATALOG = readCatalog(catalog);

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
