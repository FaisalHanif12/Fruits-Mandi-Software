// Test helper: sets the answer of `reports_print_header` (PRD FR-19.13) for the tests. The answer
// itself is in print-frame-real.ts. The command wrapper is mocked in the test file with
// vi.mock('.../features/reports/api').

import { vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { printFrameFor } from './print-frame-real';

export * from './print-frame-real';

/**
 * Sets `reports_print_header` to answer for any print. A test file that previews a print mocks the
 * reports api (`vi.mock('../../src/features/reports/api')`); in one that does not, nothing is set.
 */
export function fakePrintFrame(): void {
  if (!vi.isMockFunction(reportsApi.reportsPrintHeader)) return;
  vi.mocked(reportsApi.reportsPrintHeader).mockImplementation((input) =>
    Promise.resolve(printFrameFor(input)),
  );
}
