// The real answers and requests of the backend for plan 77 (analytics views L and M, and views A, B, C, E, F and L grouped by month),
// worked day, October 2026: view L, buyer × item (اکرم ٹول ٹیکس کیلا 880, ارشد ملتانی کیلا 1,284, شاہد سیب 800,
// total 2,964: buyer commission + لاگا), view M, expenses by head (بجلی 3,000). The backend wrote them in its own
// test (backend/crates/services/tests/fixtures/analytics_*.json); they are copied here, never edited, and read as
// the generated types. Money in them is paisa, as on the wire. A reader takes `unknown`, because the money in a
// binding is a branded type that a plain number is not.

import type { AnalyticsLinesInput } from '../../src/bindings/AnalyticsLinesInput';
import type { AnalyticsLinesPage } from '../../src/bindings/AnalyticsLinesPage';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsRunInput } from '../../src/bindings/AnalyticsRunInput';
import anLinesLAkramBanana from '../fixtures/analytics_lines_l_akram_banana.json';
import anLinesLAkramBananaRequest from '../fixtures/analytics_lines_l_akram_banana_request.json';
import anLinesMPartnerAlif from '../fixtures/analytics_lines_m_partner_alif.json';
import anLinesMPartnerAlifRequest from '../fixtures/analytics_lines_m_partner_alif_request.json';
import anLinesMElectricity from '../fixtures/analytics_lines_m_electricity_october.json';
import anLinesMElectricityRequest from '../fixtures/analytics_lines_m_electricity_october_request.json';
import anRunAByMonth from '../fixtures/analytics_run_a_by_month.json';
import anRunBByMonth from '../fixtures/analytics_run_b_by_month.json';
import anRunCByMonth from '../fixtures/analytics_run_c_by_month.json';
import anRunEByMonth from '../fixtures/analytics_run_e_by_month.json';
import anRunLByMonth from '../fixtures/analytics_run_l_by_month.json';
import anRunFByMonth from '../fixtures/analytics_run_f_by_month.json';
import anRunFByMonthRequest from '../fixtures/analytics_run_f_by_month_request.json';
import anRunL from '../fixtures/analytics_run_l.json';
import anRunLRequest from '../fixtures/analytics_run_l_request.json';
import anRunM from '../fixtures/analytics_run_m.json';
import anRunMRequest from '../fixtures/analytics_run_m_request.json';
import anRunMPartners from '../fixtures/analytics_run_m_partners.json';
import anRunMPartnersRequest from '../fixtures/analytics_run_m_partners_request.json';
import anRunMByDay from '../fixtures/analytics_run_m_by_day.json';
import anRunMByDayRequest from '../fixtures/analytics_run_m_by_day_request.json';

function readPage(value: unknown): AnalyticsPage {
  return value as AnalyticsPage;
}
function readRunInput(value: unknown): AnalyticsRunInput {
  return value as AnalyticsRunInput;
}
function readLines(value: unknown): AnalyticsLinesPage {
  return value as AnalyticsLinesPage;
}
function readLinesInput(value: unknown): AnalyticsLinesInput {
  return value as AnalyticsLinesInput;
}

export const ANALYTICS_RUN_L = readPage(anRunL);
export const ANALYTICS_RUN_L_REQUEST = readRunInput(anRunLRequest);
export const ANALYTICS_RUN_M = readPage(anRunM);
export const ANALYTICS_RUN_M_REQUEST = readRunInput(anRunMRequest);
export const ANALYTICS_RUN_M_BY_DAY = readPage(anRunMByDay);
export const ANALYTICS_RUN_M_BY_DAY_REQUEST = readRunInput(anRunMByDayRequest);
/** View M, the partners' part (`expense_part` partner): one row for each partner with what he spent himself. */
export const ANALYTICS_RUN_M_PARTNERS = readPage(anRunMPartners);
export const ANALYTICS_RUN_M_PARTNERS_REQUEST = readRunInput(anRunMPartnersRequest);
/** View F grouped by month: the period is its first column, one row for each partner in each month. */
export const ANALYTICS_RUN_F_BY_MONTH = readPage(anRunFByMonth);
export const ANALYTICS_RUN_F_BY_MONTH_REQUEST = readRunInput(anRunFByMonthRequest);
export const ANALYTICS_LINES_L_AKRAM_BANANA = readLines(anLinesLAkramBanana);
export const ANALYTICS_LINES_L_AKRAM_BANANA_REQUEST = readLinesInput(anLinesLAkramBananaRequest);
export const ANALYTICS_LINES_M_ELECTRICITY = readLines(anLinesMElectricity);
export const ANALYTICS_LINES_M_ELECTRICITY_REQUEST = readLinesInput(anLinesMElectricityRequest);

/** The entries behind partner الف's own kharch of October: his personal expense of 2,000. */
export const ANALYTICS_LINES_M_PARTNER_ALIF = readLines(anLinesMPartnerAlif);
export const ANALYTICS_LINES_M_PARTNER_ALIF_REQUEST = readLinesInput(anLinesMPartnerAlifRequest);

/**
 * Views A, B, C, E and L grouped by month, October 2026 (plan 77, the reviewer's finding): the period is the first
 * column (a month cell), then the view's own columns; a row's key carries the `from` and `to` of its bucket; the
 * totals are the whole period's; there is no chart. The backend's own answers, copied, never edited.
 */
export const ANALYTICS_RUN_A_BY_MONTH = readPage(anRunAByMonth);
export const ANALYTICS_RUN_B_BY_MONTH = readPage(anRunBByMonth);
export const ANALYTICS_RUN_C_BY_MONTH = readPage(anRunCByMonth);
export const ANALYTICS_RUN_E_BY_MONTH = readPage(anRunEByMonth);
export const ANALYTICS_RUN_L_BY_MONTH = readPage(anRunLByMonth);
