// The address of the hidden print window for a party's statement (plan 74 T3, PRD FR-12.47):
// `#/print?statement=<account id>&from=<DD-MM-YYYY>&to=<DD-MM-YYYY>&language=<ur|en>&paper=<a4|a5>&job=<job>`.
// The backend opens the window with this address, waits for `print-ready-<job>` and then prints it or saves
// it as a PDF. Like the other print windows it sits outside the app shell, and its capability allows the
// print-data and print-header commands only. Every value is read as plain text and checked here; a value that
// is not right is a failed print, never a guess. The dates go to the print-data command as they are: the
// backend reads them as typed (DD-MM-YYYY) and checks them again (Code Standards V0).

import { jobOf, positiveInt } from '../../../shared/print';
import { LANGUAGE_VALUES, paperOf } from '../../reports';
import type { Paper } from '../../../bindings/Paper';
import type { StatementPrintDataInput } from '../../../bindings/StatementPrintDataInput';

/** What the window reads the data with, and the paper the page is laid out for (A4 when the address has none). */
export type StatementPrintInput = StatementPrintDataInput & { paper: Paper };

export interface StatementPrintParams {
  /** The job name; the events are `print-ready-<job>` and `print-error-<job>`. */
  job: string;
  input: StatementPrintInput;
}

/** A date as typed: digits and dashes, short. The backend is the judge of whether it is a day. */
const TYPED_DATE = /^[0-9]{1,4}(-[0-9]{1,4}){0,2}$/;

/** True when the address is for this print (it names the party), even if the rest is wrong. */
export function isStatementAddress(searchStr: string): boolean {
  return new URLSearchParams(searchStr).has('statement');
}

/** The parameters of the print window for the statement, or null when the address is not valid. */
export function parseStatementPrintParams(searchStr: string): StatementPrintParams | null {
  const search = new URLSearchParams(searchStr);
  const job = jobOf(searchStr);
  const account = positiveInt(search.get('statement'));
  const from = search.get('from');
  const to = search.get('to');
  const language = LANGUAGE_VALUES.find((candidate) => candidate === search.get('language'));
  const paper = paperOf(search.get('paper'));
  if (job === null || account === null || language === undefined || paper === undefined) {
    return null;
  }
  if (from === null || to === null || !TYPED_DATE.test(from) || !TYPED_DATE.test(to)) return null;
  return { job, input: { account_id: account, from, to, language, paper: paper ?? 'a4' } };
}
