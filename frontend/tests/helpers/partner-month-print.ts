// Test helper: the print data of a partner's statement for one month (PRD FR-17.12, FR-17.13, plan 76
// point 11) as the backend recorded it: the real answer of `partners_statement_month_print_data` for
// partner الف on Events 1 to 5 and رحیم's advance, in each print language
// (backend/crates/services/tests/fixtures, copied byte for byte by scripts/copy-backend-fixtures.mjs).
// The data carries message IDs (the title, the ten column headings, the totals label), not text, so
// the print has to turn them into words. The figures are the backend's: the print adds nothing up
// (CLAUDE.md rule 1).

import type { TablePrintData } from '../../src/bindings/TablePrintData';
import en from '../fixtures/partners_statement_month_print_data_en.json';
import ur from '../fixtures/partners_statement_month_print_data_ur.json';

export function partnerMonthPrintData(language: 'ur' | 'en'): TablePrintData {
  return (language === 'ur' ? ur : en) as TablePrintData;
}
