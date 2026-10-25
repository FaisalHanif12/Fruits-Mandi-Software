// A printed table that is wider than its sheet is made to fit by zooming it down (PRD FR-19.3: the
// print has every column; NFR-USE-15: nothing is cut off). The analytics views of partners and trips
// have up to eighteen columns, which no A4 portrait page holds at the usual size. The measures are
// the table's own width and the width its sheet leaves for it; the zoom is the ratio, never below
// `MIN_ZOOM`, because a print nobody can read has not fitted anything.

/** The smallest zoom: below it the figures are too small to read on paper. */
export const MIN_ZOOM = 0.55;

/**
 * The zoom that brings a table of `tableWidth` inside the `available` width of its sheet, or null when
 * it already fits (or the sizes are not real).
 * Example: a table 1000 wide on a sheet that leaves 700 needs 700 / 1000 = 0.7.
 */
export function fitZoom(tableWidth: number, available: number): number | null {
  if (!(tableWidth > 0) || !(available > 0) || tableWidth <= available) return null;
  // Two decimals, rounded down, so the table is never a hair too wide after the zoom.
  return Math.max(MIN_ZOOM, Math.floor((available / tableWidth) * 100) / 100);
}
