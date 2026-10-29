// What a screen tells the shared grid (entry-grid skill, Code Standards F27). A screen declares
// its columns and keeps its rows in a store. It never writes key handlers.

import type { AccountScope } from '../accounts';
import type { SearchScope } from '../search';

/** The column types of the shared setup. Each has one fixed width (Code Standards F18). */
export type EntryColumnType =
  | 'account'
  | 'money'
  | 'qty'
  // Kilograms with up to 3 decimals, as typed (PRD E23): a number cell that is never grouped.
  | 'weight'
  | 'rate'
  | 'date'
  | 'text'
  | 'calc'
  | 'label'
  // A record chosen from a list in the cell: an account, or an item, packing and the like.
  | 'picker'
  // One of a few fixed words (ادھاری / نقدی), changed with Space or a click.
  | 'choice';

/**
 * The types the user never types in: `calc` (a number from the backend) and `label` (words the
 * screen shows, for example the name of the account on a line of the opening balances grid).
 */
export function isReadOnlyType(type: EntryColumnType): boolean {
  return type === 'calc' || type === 'label';
}

/**
 * A record the user chose in a picker cell. Kept with both names, so a language switch shows the
 * right one (Code Standards F38) without asking the backend again.
 */
export interface PickedRecord {
  readonly id: number;
  readonly code: string;
  readonly name_ur: string;
  readonly name_en: string;
  /** The tag (نشان) after the name, shown by `displayName`. Absent when there is none. */
  readonly tag?: string | undefined;
  /**
   * An item sold by weight, as its search hit said when it was chosen. Absent for every other
   * record. The item lines of an entry stop Enter at the weight of such a row (plan 35, D3).
   */
  readonly soldByWeight?: boolean | undefined;
}

/** Which records a picker column offers. */
export type PickerSpec =
  | { readonly kind: 'account'; readonly scope: AccountScope }
  | { readonly kind: 'master'; readonly scope: SearchScope };

/** One of the words of a `choice` column. `value` is what is kept; the words come from Fluent. */
export interface ChoiceSpec {
  readonly value: string;
  readonly labelId: string;
}

/** One line the user typed. `typed` holds the text of each cell, as typed (CLAUDE.md rule 1). */
export interface EntryRow {
  /** Stable for the life of the row. Used as the grid's row ID, never the row index. */
  readonly rowId: string;
  readonly typed: Readonly<Record<string, string>>;
  /** The records chosen in the picker cells of this row, by field. */
  readonly picked?: Readonly<Record<string, PickedRecord>>;
}

export interface EntryColumn {
  /** The key of this cell in `typed`, and the ID of the column. */
  readonly field: string;
  readonly type: EntryColumnType;
  /** The Fluent message ID of the header. */
  readonly headerId: string;
  /**
   * The column takes the free width of the grid, and keeps its own type width as the least it
   * has. Used for the name column of a grid that is narrower than the room it has, so no empty
   * strip is left at the end of it.
   */
  readonly grow?: boolean;
  /**
   * Can this cell be typed in on this row? Asked per cell, because some cells are open only on
   * some rows (the buyer rate when "ریٹ مختلف" is on). Default: true. A `calc` cell never is.
   */
  readonly editable?: (row: EntryRow) => boolean;
  /**
   * Does Enter stop here? Default: yes for every typed column, no for `calc`. Cells such as a
   * payment-type toggle can be editable but not an Enter stop (entry-grid skill). A function
   * decides per row: the weight of a line stops Enter only when its item is sold by weight
   * (plan 35, PRD FR-12.41).
   */
  readonly enterStop?: boolean | ((row: EntryRow) => boolean);
  /** In a grid that must fit its room (`fit`): the least width of this column, if not the type's. */
  readonly minWidth?: number;
  /** Grey hint inside an empty cell, for example the last rate. Never saved (PRD FR-7.53). */
  readonly placeholder?: (row: EntryRow) => string | undefined;
  /** The hint is a number: it stays on the physical right edge like the figures (NFR-USE-19). */
  readonly hintIsNumber?: boolean;
  /**
   * The grey hint of an empty cell is the text the backend sent for this field (the grid's
   * `calc`), for example the last rate. It is never typed into the cell and never sent (PRD
   * FR-7.53, Code Standards F43).
   */
  readonly hintFromCalc?: boolean;
  /** The `calc` field that holds the words of the hint, shown as a tooltip when the cell is narrow. */
  readonly hintTooltipField?: string;
  /**
   * An editable cell that shows the value the backend worked out (from the grid's `calc`) while
   * nothing is typed in it, as normal text. Typing replaces it; clearing the cell brings it back.
   * The commission rate tag and the lot charges use it, so what the rule gave is visible in the
   * cell (NFR-USE-27, Code Standards F43). Nothing is sent for the shown value.
   */
  readonly showsCalc?: boolean;
  /** For a `picker` column: which records it offers. */
  readonly picker?: PickerSpec;
  /** For a `choice` column: its words. Space or a click changes to the next one. */
  readonly choices?: readonly ChoiceSpec[];
  /** A calculated cell of this column has the ⓘ with the backend's working (PRD FR-7.43). */
  readonly explain?: boolean;
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

/** What a screen can add to the keys of a grid (Code Standards F27). */
export interface GridInterceptors {
  /**
   * Enter was pressed on this cell. Return true if the screen took it (it moved focus somewhere
   * else, for example from the last lot cell into the sale lines); the grid then does nothing.
   */
  readonly onEnter?: (cell: CellRef) => boolean;
  /** ↑ was pressed on the first row. Return true if the screen took it. */
  readonly onAboveFirst?: (cell: CellRef) => boolean;
}
