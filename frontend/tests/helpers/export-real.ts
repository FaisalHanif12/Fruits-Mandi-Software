// The real answers and requests of the backend for the Excel export (D5-Q1, backend round 10). The
// backend wrote them in its own test (backend/app/tests/fixtures, export_*.json); they are copied
// here, never edited, and read as the generated types. The path in them is the stand-in
// `<chosen path>`; a test that shows a path puts a real one in.

import type { AnalyticsExportRequest } from '../../src/bindings/AnalyticsExportRequest';
import type { ExcelExported } from '../../src/bindings/ExcelExported';
import type { ReportsExportRequest } from '../../src/bindings/ReportsExportRequest';
import type { UiError } from '../../src/bindings/UiError';
import analyticsExport from '../fixtures/export_analytics_export_excel.json';
import analyticsExportRequest from '../fixtures/export_analytics_export_excel_request.json';
import reportsExport from '../fixtures/export_reports_export_excel.json';
import reportsExportRefusedFilter from '../fixtures/export_reports_export_excel_refused_filter.json';
import reportsExportRefusedFilterRequest from '../fixtures/export_reports_export_excel_refused_filter_request.json';
import reportsExportRefusedJob from '../fixtures/export_reports_export_excel_refused_job.json';
import reportsExportRefusedJobRequest from '../fixtures/export_reports_export_excel_refused_job_request.json';
import reportsExportRefusedPath from '../fixtures/export_reports_export_excel_refused_path.json';
import reportsExportRefusedPathRequest from '../fixtures/export_reports_export_excel_refused_path_request.json';
import reportsExportRequest from '../fixtures/export_reports_export_excel_request.json';
import reportsExportUrdu from '../fixtures/export_reports_export_excel_urdu.json';
import reportsExportUrduRequest from '../fixtures/export_reports_export_excel_urdu_request.json';

function readExported(value: unknown): ExcelExported {
  return value as ExcelExported;
}
function readReportsRequest(value: unknown): ReportsExportRequest {
  return value as ReportsExportRequest;
}
function readAnalyticsRequest(value: unknown): AnalyticsExportRequest {
  return value as AnalyticsExportRequest;
}
function readRefusal(value: unknown): UiError {
  return value as UiError;
}

export const REPORTS_EXPORT = readExported(reportsExport);
export const REPORTS_EXPORT_REQUEST = readReportsRequest(reportsExportRequest);
export const REPORTS_EXPORT_URDU = readExported(reportsExportUrdu);
export const REPORTS_EXPORT_URDU_REQUEST = readReportsRequest(reportsExportUrduRequest);
export const ANALYTICS_EXPORT = readExported(analyticsExport);
export const ANALYTICS_EXPORT_REQUEST = readAnalyticsRequest(analyticsExportRequest);
export const EXPORT_REFUSED_FILTER = readRefusal(reportsExportRefusedFilter);
export const EXPORT_REFUSED_FILTER_REQUEST = readReportsRequest(reportsExportRefusedFilterRequest);
export const EXPORT_REFUSED_JOB = readRefusal(reportsExportRefusedJob);
export const EXPORT_REFUSED_JOB_REQUEST = readReportsRequest(reportsExportRefusedJobRequest);
export const EXPORT_REFUSED_PATH = readRefusal(reportsExportRefusedPath);
export const EXPORT_REFUSED_PATH_REQUEST = readReportsRequest(reportsExportRefusedPathRequest);
