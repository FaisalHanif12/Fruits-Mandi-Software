// Test helper: the answer of the print job of a report or a view (`reports_print_report`,
// `analytics_print_view`, 17e): a printer job answers with nothing in `pdf_paths`, a PDF job with the
// file it wrote (`report-<id>-<YYYYMMDD>-<lang>.pdf`). A refusal is a rejected promise, the way
// `invoke` fails (off Windows: `ERR_PRINT_WINDOWS_ONLY`).

import type { PrintOutcome } from '../../src/bindings/PrintOutcome';
import type { PrintTo } from '../../src/bindings/PrintTo';
import type { UiError } from '../../src/bindings/UiError';

/** The folder the PDF files of a test are written to. */
const PDF_FOLDER = 'C:\\Users\\Owner\\Documents\\Mandi';

/** What a job of this kind answers when it works. */
export function printOutcomeFor(to: PrintTo, file: string): PrintOutcome {
  return { printed: [], pdf_paths: to === 'pdf' ? [`${PDF_FOLDER}\\${file}`] : null };
}

/** The refusal of a print off Windows (PRD plan 14 contract 8): the preview stays the way to look. */
export const PRINT_WINDOWS_ONLY: UiError = {
  ok: false,
  errors: [{ code: 'ERR_PRINT_WINDOWS_ONLY', field: null, values: {} }],
  ref: 'E-20261010-0007',
};
