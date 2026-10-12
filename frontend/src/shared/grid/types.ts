// What a screen tells the shared grid (entry-grid skill, Code Standards F27). A screen declares
// its columns and keeps its rows in a store. It never writes key handlers.

/** The column types of the shared setup. Each has one fixed width (Code Standards F18). */
export type EntryColumnType =
  'account' | 'money' | 'qty' | 'rate' | 'date' | 'text' | 'calc' | 'label';

/**
 * The types the user never types in: `calc` (a number from the backend) and `label` (words the
 * screen shows, for example the name of the account on a line of the opening balances grid).
 */
export function isReadOnlyType(type: EntryColumnType): boolean {
  return type === 'calc' || type === 'label';
}

/** One line the user typed. `typed` holds the text of each cell, as typed (CLAUDE.md rule 1). */
export interface EntryRow {
  /** Stable for the life of the row. Used as the grid's row ID, never the row index. */
  readonly rowId: string;
  readonly typed: Readonly<Record<string, string>>;
}

export interface EntryColumn {
  /** The key of this cell in `typed`, and the ID of the column. */
  readonly field: string;
  readonly type: EntryColumnType;
  /** The Fluent message ID of the header. */
  readonly headerId: string;
  /**
   * Can this cell be typed in on this row? Asked per cell, because some cells are open only on
   * some rows (the buyer rate when "ریٹ مختلف" is on). Default: true. A `calc` cell never is.
   */
  readonly editable?: (row: EntryRow) => boolean;
  /**
   * Does Enter stop here? Default: yes for every typed column, no for `calc`. Cells such as a
   * payment-type toggle can be editable but not an Enter stop (entry-grid skill).
   */
  readonly enterStop?: boolean;
  /** Grey hint inside an empty cell, for example the last rate. Never saved (PRD FR-7.53). */
  readonly placeholder?: (row: EntryRow) => string | undefined;
}

/** A cell, by the row's ID and the column's field. */
export interface CellRef {
  readonly rowId: string;
  readonly field: string;
}

/** Where the grid should put focus once React has drawn the rows. */
export interface PendingFocus {
  /** A row index, or 'last' for the row just added. Clamped to the rows that exist. */
  readonly rowIndex: number | 'last';
  readonly field: string;
}
