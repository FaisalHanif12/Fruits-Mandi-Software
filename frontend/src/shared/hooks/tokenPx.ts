// A size in pixels from a design token (tokens.css), for the few places that need the number:
// a windowed list works out which rows to draw from the height of a row. The tokens are whole
// pixels (`44px`), so the digits are read with parseInt. This is layout, never money.

/** A size in pixels from a design token, or `fallback` when it is not there (tests have no CSS). */
export function tokenPx(name: string, fallback: number): number {
  const size = Number.parseInt(
    getComputedStyle(document.documentElement).getPropertyValue(name),
    10,
  );
  return Number.isFinite(size) && size > 0 ? size : fallback;
}
