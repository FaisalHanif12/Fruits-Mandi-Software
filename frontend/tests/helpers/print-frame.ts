// Test helper: the answer of `reports_print_header` (the shared header and footer of every report and
// analytics print, PRD FR-19.13). The backend's real answer for the firm, its two contact persons, the
// trade mark and the footer is kept in two fixtures (English and Urdu); the title and the facts are
// what the print asked for, as the backend echoes them. The command wrapper is mocked in the test
// file with vi.mock('.../features/reports/api').

import { vi } from 'vitest';
import type { PrintFrame } from '../../src/bindings/PrintFrame';
import type { PrintFrameInput } from '../../src/bindings/PrintFrameInput';
import * as reportsApi from '../../src/features/reports/api';
import frameEn from '../fixtures/reports_print_header_en.json';
import frameUr from '../fixtures/reports_print_header_ur.json';

function readFrame(value: unknown): PrintFrame {
  return value as PrintFrame;
}

export const PRINT_FRAME_EN = readFrame(frameEn);
export const PRINT_FRAME_UR = readFrame(frameUr);

/** The frame the backend builds for a print: the firm of the language, the title and facts asked for. */
export function printFrameFor(input: PrintFrameInput): PrintFrame {
  const base = input.language === 'ur' ? PRINT_FRAME_UR : PRINT_FRAME_EN;
  return { ...base, header: { ...base.header, title_id: input.title_id, facts: input.facts } };
}

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

/** The firm name the shared header prints in this language (the firm of the fixtures). */
export function printFirmName(language: 'ur' | 'en'): string {
  return (language === 'ur' ? PRINT_FRAME_UR : PRINT_FRAME_EN).header.firm.name;
}
