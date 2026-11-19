// The Excel export of the table pages of plan 76 (PRD FR-19.3, FR-19.11): the اگراہی sheet, the buyer dues, the
// commission month and a partner's month statement. One command (`tables_export_excel`) writes every row of the
// filter, made from the same table as the print, so the two agree. `source` is the screen's filter as it is NOW
// and the language of the sheet is chosen by the button or the print language box. The file is never opened or
// read back; nothing here works out any figure (CLAUDE.md rule 1).

import type { Language } from '../../bindings/Language';
import type { TableExportSource } from '../../bindings/TableExportSource';
import { exportFileName, useExcelExport, type ExcelExporter } from '../../shared/export';
import { tablesExportExcel } from './api';

/** The exporter of a table page: `stem` starts the file name, `source` is the page's filter in a language. */
export function useTableExcel(
  stem: string,
  source: (language: Language) => TableExportSource,
): ExcelExporter {
  return useExcelExport({
    defaultName: () => exportFileName(stem, new Date()),
    run: ({ path, jobId, language }) =>
      tablesExportExcel({ job_id: jobId, path, source: source(language) }),
  });
}
