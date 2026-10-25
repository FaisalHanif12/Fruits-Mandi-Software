// Which goods columns each document has (plan 14). Only layout choices, no sums: a column is shown
// when the lines have something for it. The weight column shows only when a line has a weight; the
// date column only when a line was sold on another day than the header's date (a later sale,
// PRD FR-12.17). The seller bill has the buyer's name on each line; a slip is for one buyer, so it
// has no buyer column.

import type { PrintColumn, PrintData, PrintPage } from '../../../shared/print';

/** The id of the header fact that holds the date of the بکری (contract 6). */
export const DATE_FACT_ID = 'print-fact-date';

function headerDate(data: PrintData): string | null {
  return data.header.facts.find((fact) => fact.label_id === DATE_FACT_ID)?.value ?? null;
}

function withDateAndWeight(
  page: PrintPage,
  data: PrintData,
  middle: readonly PrintColumn[],
): PrintColumn[] {
  const day = headerDate(data);
  const showDate = page.lines.some((line) => day === null || line.date !== day);
  const showWeight = page.lines.some((line) => line.weight_kg !== null);
  return [
    ...(showDate ? (['date'] as const) : []),
    ...middle,
    'qty',
    ...(showWeight ? (['weight'] as const) : []),
    'rate',
    'amount',
  ];
}

/** The seller bill (R1): who bought, what, how many, at what rate. */
export function sellerBillColumns(page: PrintPage, data: PrintData): PrintColumn[] {
  return withDateAndWeight(page, data, ['buyer', 'item', 'packing']);
}

/** A buyer slip (R5) and the پختہ slip: the goods of one buyer. */
export function slipColumns(page: PrintPage, data: PrintData): PrintColumn[] {
  return withDateAndWeight(page, data, ['item', 'packing']);
}
