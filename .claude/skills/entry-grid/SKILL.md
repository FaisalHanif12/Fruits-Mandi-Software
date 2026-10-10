---
name: entry-grid
description: Builds or changes a spreadsheet-style entry grid in the Mandi Software (بکری lots and sale lines, voucher lists, جنرل lines, opening balances) on AG Grid with the exact keyboard behaviour of PRD 7.2, both directions, live backend totals, and no lost data on a language switch. Use for any grid where the user types data, not for read-only report tables.
---

# Entry grids

Entry grids are where the munshi spends his day. A sale line must take **3 fields and 3 `Enter`s** (NFR-USE-6). The grid must feel like a spreadsheet, never lose what was typed, and never calculate money itself.

All entry grids are built from the shared setup in `frontend/src/shared/grid/`. A screen only declares its columns and its row store. **A screen never writes its own key handlers** (Code Standards F27). If the shared setup is missing something, add it there for every grid.

Code sketches for the shared setup are in [reference.md](reference.md).

## Where the data lives

```
Row store (Zustand, per screen)      <- the source of truth for what the user typed
   rows: [{ rowId, typed: { buyerCode: "1", qty: "20", rate: "600" }, ... }]
        |
        v
AG Grid (display + editing only)     <- can be destroyed and rebuilt at any time
        |
        v  (50 ms after the last key, previous call cancelled)
Backend preview command              <- returns calculated values and errors per row
   rows: [{ rowId, valuePaisa, commPaisa, ..., errors: [...] }, totals ]
```

- Typed text is saved into the store when a cell edit finishes, never only inside AG Grid. A language switch rebuilds the grid, because `enableRtl` is set only when the grid is created (CLAUDE.md, Gotchas), and nothing may be lost.
- Calculated cells show values from the last preview response. The grid never adds, multiplies or rounds money.
- Each row has a stable `rowId` (a UUID made when the row is created), used as AG Grid's `getRowId`. Never use the row index as an ID.

## Column types

Every column uses one of the shared types from `shared/grid/column_types.ts`. Never style a column by hand.

**Editable is decided per cell, not per column.** Some cells open only on some rows: the buyer rate only when that line's "ریٹ مختلف" toggle is on (FR-12.3), and the commission and لاگا rates only when the Owner allows overrides (FR-11.10). Use a function for `editable` that reads the row and the settings, and let the Enter order ask each cell.

| Type | Editable | Shows | Notes |
|---|---|---|---|
| `account` | yes | code, then name through `displayName` | `AccountPicker` editor; short codes resolve on `Enter` (PRD FR-7.39); shows the party card (FR-7.38) |
| `money` | yes | the typed text while editing, `formatPaisa` result after | Accepts simple sums like `20+25` (FR-7.48); parsed only by the backend |
| `qty` | yes | whole numbers | Same parsing rules as money |
| `rate` | yes | money | Empty with the last-rate hint as placeholder text (FR-7.53). **Never pre-filled.** |
| `date` | yes | `DD-MM-YYYY` | `+` and `−` change the day (FR-7.19) |
| `text` | yes | Urdu text uses `RomanUrduInput` | Notes, narration |
| `calc` | **no** | a value from the preview | Has the ⓘ (`CalcInfo`) that shows the backend's working (FR-7.43) |

Number cells (`money`, `qty`, `rate`, `calc`) use tabular digits and sit on the **physical right edge** of the cell in both languages (NFR-USE-19, Code Standards F17).

## Keyboard behaviour (PRD FR-7.13 to FR-7.20)

The shared keyboard setup implements this table exactly. Test every row of it.

| Key | Not editing | While editing |
|---|---|---|
| Any letter or digit | Start editing; replace the cell's content | Type |
| `+` `−` | In a date cell: next or previous day (FR-7.19). In other cells: start editing (simple sums, FR-7.48). | Type |
| `Enter` | Move to the next **Enter stop** in reading order. Skip cells that are not editable on this row, and `calc` cells. **Stop** on editable cells the app filled (FR-7.51, FR-7.52). On the last editable cell of a row, go to the first editable cell of the next row, creating it if needed. | Finish the cell, then the same as not editing |
| `↑` `↓` | Move to the cell above or below. `↓` on the last row creates a new row. | Finish, then move up or down |
| `←` `→` | Move to the neighbouring cell **as the user sees it** (in Urdu, `←` is the next cell) | Move the text cursor. At the edge of the text, finish and move to the neighbouring cell. |
| `F2` | Does **not** start editing (PRD Section 7.3: F2 is "New") | |
| `Ctrl+D` | Sale-line grids only: copy the same cell from the line above (PRD 7.3, FR-7.54) | |
| `Ctrl+I` | On a `calc` cell: open "how was this worked out" (FR-7.43) | |
| `Ctrl+Delete` | Remove the current row, before save | |
| `Alt+↓` | In an `account` cell: open the full list (FR-7.16) | |
| `Esc` | Leave the grid, following FR-7.18 order (ask before leaving unsaved work) | Cancel the typing in this cell, keep the old value |

While a picker's list is open, `↑` `↓` move through the list and `Enter` picks. While it is closed, they move between cells.

## Filled values and blank rows

- **Remaining quantity (FR-7.52).** A new sale line's `qty` starts as the lot's remaining quantity, selected, so typing replaces it. The value comes from the preview, not from frontend maths.
- **Seller memory (FR-7.51).** Item, packing and مارکہ are filled from the backend's memory command and are editable stops.
- **Blank rows.** A row whose buyer and rate are both empty is blank. It is not sent as an error, the preview ignores it, and save removes it. A filled `qty` alone does not make a row non-blank.
- Filled values look exactly like typed ones. Nothing is hidden (Code Standards F43).

## Errors on cells

- The preview and save return errors with paths like `lots[0].sales[2].qty`. The shared helper maps each path to `rowId` and column, gives the cell an error style, and shows the message from `errors.ftl` on focus.
- Errors never block typing. They block save.

## Build steps

1. List the columns in the plan's contract order, each with a shared type, an `editable` rule (a function of the row and settings where needed) and an `enterStop` flag.
   - **Enter stops** are the cells the PRD's own example walks through. For a sale line that is buyer, qty and rate (FR-7.14, NFR-USE-6).
   - Cells such as the payment-type toggle and the line note stay editable (by arrows, click or their own key), but are not Enter stops.
   - This reading of FR-7.14 is recorded as an open question for Umar. Do not change the stop set without his answer.
2. Create the row store with `addRow`, `updateTyped(rowId, field, text)`, `removeRow` and `blankRowCheck`.
3. Use `<EntryGrid columns rows onCommit />` from `shared/grid`. Mount it with `key={language}`, so a language switch rebuilds it, and restore focus from the store afterwards.
4. Connect the preview: the store changes, then after 50 ms the preview command runs (cancelling the last one), then the calculated cells and totals update.
5. Totals row: AG Grid's pinned bottom row, filled from the preview totals and aligned under the columns (Code Standards F19).

## Tests

- **Keyboard-only test** for the screen (Code Standards F30). Enter one بکری 1 line from PRD 4.4 using only the keys: buyer `1` `Enter`, qty `20` `Enter`, rate `600` `Enter`. Check that the cursor is on the next row's buyer cell. Take the Enter stops from the plan's column list, not from this example, so the test follows any later decision.
- **Per-row editability:** with "ریٹ مختلف" off, Enter skips the buyer rate; with it on, the buyer rate is a stop.
- **One test per row** of the keyboard table above, in Urdu **and** English layouts (arrow direction flips).
- **Language switch mid-entry** (Code Standards F39): type two lines, switch, check every typed value and the focused cell are the same.
- **Remaining qty and blank row:** 50 received, 20 sold, the new line shows 30. A trailing blank row does not block save.
- Run the full keyboard check on Windows with the `windows-verify` skill, because AG Grid focus behaviour must be confirmed in WebView2.

## Done when

- [ ] Only shared column types and the shared keyboard setup are used.
- [ ] Typed values live in the store. Rebuilding the grid loses nothing.
- [ ] No money is calculated in the grid. Rates are never pre-filled.
- [ ] Every row of the keyboard table works in both layouts.
- [ ] A sale line takes 3 fields and 3 `Enter`s (buyer, qty, rate).
- [ ] Editability is decided per cell; Enter skips cells that are closed on that row.
- [ ] Errors show on the right cell with a message from `errors.ftl`.
