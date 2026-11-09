// What a screen tells the shared grid (entry-grid skill, Code Standards F27). A screen declares
// its columns and keeps its rows in a store. It never writes key handlers.

import type { AccountScope, PickedChukta } from '../accounts';
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
  | 'choice'
  // One of a few fixed words that is optional, chosen from a small list in a popup under the cell
  // (the quality of a sale line: اول, دوم, سوم, or nothing). Space, Alt+↓ or a click opens it.
  | 'select';

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
  /**
   * Present only for a چکتا buyer (PRD FR-7.29, FR-13.5): the line is on the account چکتا کھاتہ, and
   * this holds his phone and his own record when the picker knew it. Names are his own.
   */
  readonly chukta?: PickedChukta | undefined;
}

/** Which records a picker column offers. */
export type PickerSpec =
  | {
      readonly kind: 'account';
      readonly scope: AccountScope;
      /**
       * The buyer box of a بکری line: it lists چکتا buyers beside ordinary buyers and ends its list
       * with the second add row, a new چکتا buyer (PRD FR-7.29, FR-7.69, FR-13.3). No other box does.
       */
      readonly withChukta?: boolean;
    }
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
  /**
   * The grey hint of an empty cell is the text the screen made from what the backend sent for this
   * field (the grid's `calc`), for example "Last 900" for the last rate. It is words in grey italic,
   * laid out as text and never as a figure, so it is not taken for a typed value. It is never typed
   * into the cell and never sent (PRD FR-7.53, Code Standards F43, D8 Bug 1).
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
  /**
   * For a `choice` column: its words. Space or a click changes to the next one. For a `select`
   * column: the words of its list, in order; the one with the value '' is the empty choice.
   */
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
  /**
   * Start typing in the cell once it has the focus, with its text selected, so the next key replaces it
   * (a screen that sends the cursor back to a cell that must be fixed, D8 Bug 1).
   */
  readonly edit?: boolean;
}

/** What the grid tells the screen about an Enter. */
export interface EnterInfo {
  /** The Enter finished a cell and changed its text (the typed text differs from what it was). */
  readonly changed: boolean;
}

/** What a screen can add to the keys of a grid (Code Standards F27). */
export interface GridInterceptors {
  /**
   * Enter was pressed on this cell. Return true if the screen took it (it moved focus somewhere
   * else, for example from the last lot cell into the sale lines); the grid then does nothing.
   */
  readonly onEnter?: (cell: CellRef, enter?: EnterInfo) => boolean;
  /** ↑ was pressed on the first row. Return true if the screen took it. */
  readonly onAboveFirst?: (cell: CellRef) => boolean;
  /**
   * The cell this row is held on, or null. While a row is held (the backend refused its quantity), the
   * cursor stays on that cell: arrows, Tab and Enter do not leave it, and no other cell of the row can
   * be typed in. Esc and Ctrl+Delete keep their meaning (PRD FR-7.18). The screen only reads what the
   * backend said; it judges nothing (D8 Bug 1, B2).
   */
  readonly heldField?: (rowId: string) => string | null;
}
