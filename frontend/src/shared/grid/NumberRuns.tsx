// Numbers joined by dashes or slashes inside a text cell, for example the date of a trip heading
// ("سفر-1 10-10-2026"). In a right-to-left cell the digits that follow Arabic letters are read as
// Arabic-Indic numbers, and the dashes between them split the date into three pieces that come out
// back to front ("2026-10-10"). Each such run is put in its own left-to-right isolate, so it keeps
// its order (Code Standards F25, NFR-URD-13). The text itself is not changed: the same characters
// are drawn, only inside `<bdi dir="ltr">`.

import type { ICellRendererParams } from 'ag-grid-community';
import { Fragment, type ReactNode } from 'react';

/** Digits in groups joined by `-` or `/`: a date, a range. A lone number needs no help. */
const NUMBER_RUN = /\d+(?:[-/]\d+)+/g;

/** True when the text holds a run that the isolate protects. */
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
