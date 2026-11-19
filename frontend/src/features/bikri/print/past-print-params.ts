// The address of the hidden print window for پچھلی بکری (plan 72 P8): `#/print?bikrilist=1&language=<ur|en>
// &paper=<a4|a5>&list=<percent-encoded JSON of the list's filter>&job=<job>`. The backend opens the window
// with this address, waits for `print-ready-<job>` and then prints it or saves it as a PDF. Like the other
// print windows it sits outside the app shell, and its capability allows the print-data and print-header
// commands only. Every value is read as plain text and checked here; a value that is not right is a failed
// print, never a guess. The print-data command checks the filter again (Code Standards V0).

import { z } from 'zod';
import { jobOf } from '../../../shared/print';
import { LANGUAGE_VALUES, paperOf, readJsonObject } from '../../reports';
import type { BikriListPrintInput } from '../../../bindings/BikriListPrintInput';
import type { PastPrintFilter } from '../past-print-filter';

export interface PastPrintParams {
  /** The job name; the events are `print-ready-<job>` and `print-error-<job>`. */
  job: string;
  input: BikriListPrintInput;
}

/** The list's filter as the screen sent it: a JSON object, which the backend checks field by field. */
const filterSchema = z.custom<PastPrintFilter>(
  (value) => typeof value === 'object' && value !== null && !Array.isArray(value),
);

/** True when the address is for this print (it names it), even if the rest is wrong. */
export function isPastListAddress(searchStr: string): boolean {
  return new URLSearchParams(searchStr).has('bikrilist');
}

/** The parameters of the print window for the list, or null when the address is not valid. */
export function parsePastPrintParams(searchStr: string): PastPrintParams | null {
  const search = new URLSearchParams(searchStr);
  const job = jobOf(searchStr);
  const language = LANGUAGE_VALUES.find((candidate) => candidate === search.get('language'));
  const paper = paperOf(search.get('paper'));
  const filter = readJsonObject(search.get('list'), filterSchema);
  if (job === null || language === undefined || paper === undefined || filter === null) return null;
  return { job, input: { ...filter, language, paper: paper ?? 'a4' } };
}
