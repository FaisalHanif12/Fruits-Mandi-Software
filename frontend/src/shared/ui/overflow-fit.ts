// How many items of a row fit its width, with a "More" button for the rest (plan 93). One piece of arithmetic for every
// row that is always ONE line: the tabs of a section (`SectionTabs`) and the quick actions of the dashboard
// (`OverflowRow`). The widths are measured by the browser; here they are numbers.

/**
 * How many items fit `width`. `widths` are the measured widths of the items, `moreWidth` the width of the "More"
 * button and `gap` the space between two items. Every item fits when the total is within the width; otherwise the
 * "More" button takes its room first. At least one item always stays. Before the row has a width (a test, a window being
 * built, `width` 0) every item is shown.
 */
export function itemsThatFit(
  width: number,
  widths: readonly number[],
  moreWidth: number,
  gap: number,
): number {
  if (width <= 0 || widths.length === 0) return widths.length;
  const all = widths.reduce((sum, w) => sum + w, 0) + gap * (widths.length - 1);
  if (all <= width) return widths.length;
  let used = moreWidth;
  let count = 0;
  for (const w of widths) {
    if (used + gap + w > width) break;
    used += gap + w;
    count += 1;
  }
  return Math.max(count, 1);
}
