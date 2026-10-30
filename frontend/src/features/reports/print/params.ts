// The address of the hidden print window for a report: `#/print?report=<id>&language=<ur|en>&paper=
// <a4|a5>[&slips=1]&filters=<percent-encoded JSON of ReportFilters>[&sort=<percent-encoded JSON of the
// screen's SortInput>][&months=1]&job=<job>` (17e contract, plan 38). The window
// is opened by the backend, so every value is read as plain text and checked here. A value that is not
// right is a failed print, never a guess. The job name is checked by the same function as every other
// print window's (one rule for the window). The filters are the backend's own JSON for the same filters
// the screen asked with; the print-data command checks them again (Code Standards V0).

import { z } from 'zod';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { ReportFilters } from '../../../bindings/ReportFilters';
import type { ReportId } from '../../../bindings/ReportId';
import type { ReportPrintInput } from '../../../bindings/ReportPrintInput';
import type { SortInput } from '../../../bindings/SortInput';
import { jobOf } from '../../../shared/print';

export interface ReportPrintParams {
  /** The job name; the events are `print-ready-<job>` and `print-error-<job>`. */
  job: string;
  /** True for the slips of all the credit buyers of R5 ("Print All"). */
  slips: boolean;
  input: ReportPrintInput;
}

/** Every report of the catalogue (PRD 19.2). A test holds the list to the backend's catalogue. */
export const REPORT_IDS: readonly ReportId[] = [
  'R1',
  'R2',
  'R3',
  'R4',
  'R5',
  'R6',
  'R7',
  'R8',
  'R9',
  'R10',
  'R11',
  'R12',
  'R13',
  'R14',
  'R15',
  'R16',
  'R17',
  'R18',
  'R19',
  'R20',
  'R21',
  'R22',
  'R23',
  'R24',
  'R25',
  'R26',
  'R27',
  'R28',
  'R29',
  'R30',
  'R31',
  'R32',
  'R33',
  'R34',
  'R35',
  'R36',
  'R37',
  // The dead amount (plan 40): printed, saved as PDF and exported like any report.
  'R38',
];

export const LANGUAGE_VALUES: readonly Language[] = ['ur', 'en'];
export const PAPER_VALUES: readonly Paper[] = ['a4', 'a5', 'mm80'];

/** The filters as the screen sent them: a JSON object, which the backend checks field by field. */
const filtersSchema = z.custom<ReportFilters>(
  (value) => typeof value === 'object' && value !== null && !Array.isArray(value),
);

/** The order the screen had, as the backend wrote it into the address. */
const sortSchema = z.object({
  column: z.string().min(1),
  direction: z.enum(['asc', 'desc']),
  language: z.enum(['ur', 'en']).nullable(),
});

/**
 * The `sort` of an address: absent means the report's own order (null); a value that is not a
 * sort is wrong (undefined), so the print fails rather than prints in another order.
 */
export function sortOf(text: string | null): SortInput | null | undefined {
  if (text === null) return null;
  return readJsonObject<SortInput>(text, sortSchema) ?? undefined;
}

/** Reads the `filters` of an address: percent-decoded JSON that holds an object, or null. */
export function readJsonObject<T>(text: string | null, schema: z.ZodType<T>): T | null {
  if (text === null || text === '') return null;
  try {
    const parsed = schema.safeParse(JSON.parse(text));
    return parsed.success ? parsed.data : null;
  } catch {
    return null;
  }
}

/** True when the address is for a report (it names one), even if the rest is wrong. */
export function isReportAddress(searchStr: string): boolean {
  return new URLSearchParams(searchStr).has('report');
}

/** The paper of an address: absent means A4 (null); a name that is not a paper is wrong (undefined). */
export function paperOf(text: string | null): Paper | null | undefined {
  if (text === null) return null;
  return PAPER_VALUES.find((candidate) => candidate === text);
}

/** The parameters of the print window for a report, or null when the address is not valid. */
export function parseReportPrintParams(searchStr: string): ReportPrintParams | null {
  const search = new URLSearchParams(searchStr);
  const job = jobOf(searchStr);
  const id = REPORT_IDS.find((candidate) => candidate === search.get('report'));
  const language = LANGUAGE_VALUES.find((candidate) => candidate === search.get('language'));
  const paper = paperOf(search.get('paper'));
  const filters = readJsonObject(search.get('filters'), filtersSchema);
  const slipsText = search.get('slips');
  const sort = sortOf(search.get('sort'));
  const monthsText = search.get('months');
  if (job === null || id === undefined || language === undefined || paper === undefined) {
    return null;
  }
  if (filters === null || (slipsText !== null && slipsText !== '1')) return null;
  // The order and the month headings of the screen are in the address when it had them: the print
  // has every row in that order, with the month heading rows (plan 38).
  if (sort === undefined || (monthsText !== null && monthsText !== '1')) return null;
  return {
    job,
    slips: slipsText === '1',
    input: { id, filters, language, paper, sort, group_by_month: monthsText === '1' },
  };
}
