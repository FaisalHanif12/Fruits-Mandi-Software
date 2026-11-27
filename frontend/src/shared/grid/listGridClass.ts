// The class list of the box of a list grid (plan 96 T2). It is built from separate words and joined once, so two
// classes can never run together. A missing space once made `ag-theme-quartz` and `list-grid-fill` one unknown class
// word: a list of more than ten rows lost the AG Grid theme and its height, and the روکڑ بک drew no table.

/** The class of the box of every list grid, and the selector that finds it (for `useShortcut`'s `unlessFocusIn`). */
export const LIST_GRID_CLASS = 'list-grid';
export const LIST_GRID_SELECTOR = `.${LIST_GRID_CLASS}`;

/** The AG Grid theme class: without it the grid has no layout rules at all. */
export const GRID_THEME_CLASS = 'ag-theme-quartz';

/**
 * The classes of the box: the entry grid base, the list marker, the AG Grid theme, the fill height of a long list
 * (a short list is as tall as its rows) and the wrapping of text that grows.
 */
export function listGridClassName(heightMode: 'fill' | 'auto', growing: boolean): string {
  return [
    'entry-grid',
    LIST_GRID_CLASS,
    GRID_THEME_CLASS,
    heightMode === 'fill' ? 'list-grid-fill' : '',
    growing ? 'list-grid-wrap' : '',
  ]
    .filter((word) => word !== '')
    .join(' ');
}
