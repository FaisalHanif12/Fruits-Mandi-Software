// Goods set for another shop, as a sale in another shop reads them (plan 90, PRD FR-12.41,
// FR-12.32, FR-12.36). A garden's goods arrived line and a trip's purchase line both say how many
// نگ are left for another shop (`qty_other_left`) and the name of that shop (`shop_name`). The sale
// offers only the lines with goods left for another shop, fills the shop name from the line, and
// does not take more نگ than the line has for another shop. The numbers are the backend's: this
// only compares whole quantities and picks text. No money is worked out here (CLAUDE.md rule 1).

/** What a goods arrived line and a purchase line both carry for a sale in another shop. */
export interface OtherShopLine {
  id: number;
  qty_other_left: number;
  shop_name: string | null;
}

/**
 * The lines a sale in another shop offers: the ones with goods left for another shop. The line that
 * is already chosen stays, so an entry open for edit can still show the line it was made on even
 * when its own sale took the last نگ.
 * Example: lines with 10, 0 and 4 left give the first and the third.
 */
export function offeredForOtherShop<T extends OtherShopLine>(
  lines: readonly T[],
  chosenId: string,
): T[] {
  return lines.filter((line) => line.qty_other_left > 0 || String(line.id) === chosenId);
}

/**
 * The shop name after another line is picked. A name typed by hand is never overwritten: only an
 * empty box, or a box that still holds the name of the line picked before, takes the new line's
 * name. A line with no shop name leaves a typed name alone.
 * Example: "Haji Shop" (from the old line) and a new line of "Noor Shop" gives "Noor Shop"; a typed
 * "My Shop" stays.
 */
export function shopAfterPicking(
  current: string,
  previousShop: string | null | undefined,
  nextShop: string | null | undefined,
): string {
  const isFilledByTheLine = current === '' || current === (previousShop ?? '');
  return isFilledByTheLine ? (nextShop ?? '') : current;
}

/**
 * True when the whole number typed is more than the line has for another shop, so the qty field says so before
 * the backend is asked. A sum such as `20+25` is read by the backend, whose preview and save refuse it with
 * `ERR_QTY_EXCEEDS_OTHER_SHOP` (value `remaining`, PRD FR-12.41), shown under the qty like any refusal.
 */
export function qtyAboveOther(qtyText: string, left: number): boolean {
  const plain = qtyText.replaceAll(',', '').trim();
  return /^\d+$/.test(plain) && Number(plain) > left;
}
