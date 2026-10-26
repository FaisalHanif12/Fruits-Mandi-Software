// Every number inside a line of words in a text cell: a date ("10-10-2026"), a quantity and a rate
// ("(کیلا 30 . 580)"), a code ("0021"), the number of a trip ("سفر-1"). In a right-to-left cell the
// digits that follow Arabic letters are read as Arabic-Indic numbers, the dashes between them split a
// date into three pieces that come out back to front ("2026-10-10"), and an Urdu font may draw the
// digits in its own Eastern shapes. Each number is put in its own left-to-right isolate in Inter with
// equal-width digits (`number-run`), so it keeps its order and its shape (Code Standards F25,
// NFR-URD-13, Umar 26-10-2026). The text itself is not changed: the same characters are drawn, only
// inside `<bdi dir="ltr">`; the words around the numbers, and the spaces, stay as they are.

import type { ICellRendererParams } from 'ag-grid-community';
import { Fragment, type ReactNode } from 'react';

/**
 * A number: digits with their thousands commas and decimals (1,160 or 12.5), and the numbers joined to
 * it by `-`, `/` or `:` (a date, a range, a time). A lone number is a run of its own.
 */
const NUMBER = String.raw`\d+(?:,\d{3})*(?:\.\d+)?`;
const NUMBER_RUN = new RegExp(String.raw`${NUMBER}(?:[-/:]${NUMBER})*`, 'g');

/** True when the text holds a number, which the isolate protects. */
export function hasNumberRun(text: unknown): text is string {
  return typeof text === 'string' && new RegExp(NUMBER_RUN.source).test(text);
}

/** The text with each number run in its own isolate. */
export function NumberRuns({ text }: { text: string }): ReactNode {
  const parts: ReactNode[] = [];
  let last = 0;
  for (const match of text.matchAll(NUMBER_RUN)) {
    const at = match.index;
    if (at > last) parts.push(<Fragment key={`t${String(at)}`}>{text.slice(last, at)}</Fragment>);
    parts.push(
      <bdi key={`n${String(at)}`} dir="ltr" className="number-run">
        {match[0]}
      </bdi>,
    );
    last = at + match[0].length;
  }
  if (last < text.length) parts.push(<Fragment key="end">{text.slice(last)}</Fragment>);
  return parts;
}

/** The cell renderer AG Grid uses for a text cell that holds a number run. */
export function NumberRunsCell({ value }: ICellRendererParams) {
  return typeof value === 'string' ? <NumberRuns text={value} /> : null;
}
