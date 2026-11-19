# shared/grid

AG Grid setup for every entry grid: right-to-left and left-to-right, the keyboard rules of
PRD 7.2 (Enter stops on every editable cell, skips calculated ones; a column can say it is not an
Enter stop, or decide per row with `enterStop: (row) => boolean`, as the item lines of plan 35
do), and column types for money,
quantity, account and date with fixed widths (Code Standards F17, F18, F27). Arrives in plan 04
with the `entry-grid` skill. A grid never holds unsaved data on its own (F39).

## ListGrid (read-only lists)

`ListGrid` is the one grid for reading a list. Besides the plain text, number, date and status
columns, it has (plan 20, for the report tables): `unit`, a short word after a figure, which stays in
the middle of the column (the side of a balance; column type `balance`); `wrap`, a long text that
wraps and makes its row taller (the other cells of such a list keep their words in the middle of the
row); and `isGroup`, a row that heads a group. The totals row is pinned under the last row and
carries the backend's figures; its word "Total" stands in the first text column (plan 39 R6).

Alignment (plan 39, NFR-USE-19 v1.6): a number column (money, quantity, date, code, days), its heading
and its total are centred in both languages (`cell-num`, `header-num`); a text column starts at the start
edge; a small column (a choice, a tick) is centred (`cell-center`, `header-center`). Every kind of column
has one width (`SHARED_COLUMN_WIDTHS`, the same numbers as the `--col-*-w` tokens). The proof is
`tests/e2e/alignment/` (see its README).

## Status chips and quiet zeros (plan 75, items 4 and 8)

- `ListColumn.chip(row, context)` may return `{ tone, icon }` (`StatusChipLook`, `shared/ui/status-looks`
  has the shared ones: `STATUS_SAVED`, `STATUS_CANCELLED`, `STATUS_OPEN`, `STATUS_CLOSED`): the word of
  `value` is drawn as a `StatusChip`, centred under a centred heading (`cell-center cell-chip`,
  `header-center`). The word is always in the chip and the icon goes with it, so a state is never told by
  colour alone. A row with no word draws an empty cell. A column without `chip` is drawn as before.
- A read-only list (`.list-grid`) has a lighter heading (`--color-surface-readonly`) than an entry grid
  and tints the row under the pointer. A money column gives a body cell whose text is "0" the class
  `cell-zero` (the quiet ink); the total row never gets it. Colour only: the 0 stays where it is.

## Dense column types (plan 20, step F3)

`count` (100 px, a quantity or a count of days), `figure` (120 px, an amount in a table with many
money columns, up to 9 digits) and `short` (140 px, a phone number or a short name). A report picks
them by the kind of the backend's column (`features/reports/columns.ts`). The older `number` and
`money` types are unchanged.

## Tone and number runs (plan 25)

- `ListColumn.tone(row, context)` may return `'due'`: the cell is drawn in the due colour
  (`cell-tone-due`). The totals row asks for it under the key `<field>:tone`. It is never the only
  signal: a signed figure also has its sign, and its word in the unit slot (a loss of a trip).
- A text cell with digits joined by `-` or `/` (a date, a range) is drawn with each run in a
  left-to-right isolate (`NumberRuns`), so the date of a trip heading reads "10-10-2026" and not
  "2026-10-10" in Urdu (F25, NFR-URD-13). Only a cell that holds such a run gets the renderer; the
  text is the same.

## Lists with page buttons (plan 17b, plan 38 R1, PRD FR-19.11, Code Standards F34)

A list asks the backend for one page at a time (25, 50 or 100 rows) and shows only the rows in view.
Every page comes with the count and the totals of the whole filter; the screen shows those, never a
sum of the rows it holds.

- `shared/hooks/usePagedQuery` is the one hook for it: `{ screen, queryKey, fetchPage(offset, limit),
rowsIn, totalOf }`. `data.pages` holds the one page on screen, and `pager` says where the list is
  (`page`, `pageCount`, `from`, `to`, `total`, `goTo`, `setPageSize`). `screen` names the list
  (`cashbook.takes`): the rows per page is kept for it through `prefs_screen_set` as `pager.<screen>`
  (PRD FR-7.45), a new filter (a changed `queryKey`) goes back to page 1, and a list opened again
  from a record it showed comes back to the same page.
- `Pager` is the line under the list: "26 to 50 of 340", rows per page, "Page 2 of 14", and First,
  Previous, Next, Last (the arrows turn round in right-to-left). A filter that fits one page of 25
  shows only the count. `PagedList` has it built in; a screen that draws its own `ListGrid` adds
  `<Pager pager={query.pager} />` and `paging={useListPaging(query.pager)}`.
- Page Up and Page Down in a `ListGrid` with `paging` turn the page and put focus on the first row of
  the new page (PRD FR-7.17). A language switch rebuilds the grid; the page lives in the hook, so
  nothing moves.
- `PagedList` is the list with its four states (loading, error with a retry, empty with a helpful
  message, rows) and the backend's totals row. `onOpen`, `onFocusRow` and `focusRowId` are for a list
  whose rows open something or whose focused row the screen acts on.
- `useClientPager(screen, filterKey, rows)` does the same for a short list that is already in memory
  (the master lists, whose text filter is matched on screen). `useAllPagesQuery` and `useLoadRest`
  read every page, for a choice list or a screen that needs all rows (the opening balances).
- Used by: the cash book, partner takes, ادھار رقم, الگ کھاتہ, capital and the partner statement,
  آج کی بکری, باقی مال (the lots, then the firm's own goods, counted together), the voucher lists,
  the rules, the accounts, the trips and gardens (and the history inside
  one), the audit log, the backup list, the learned words, the reports and the analytics views.
- In tests, `tests/setup.ts` forgets the pages after each test (`resetPagerMemory`).

A mouse click on an editable picker cell starts its editor with the full list open (plan 32, R7),
like a click on the field of a form. A key never does: arriving on the cell by Enter or an arrow
opens nothing, and `Alt+↓` is the key for the list (PRD 7.2, FR-7.16).

## The remove button of a row (plan 43, PRD FR-7.3, FR-7.4, NFR-USE-8)

Every `EntryGrid` has a last column with a small trash button for each row (`removeRow.tsx`), so a line
can be taken away with the mouse; `Ctrl+Delete` stays as its shortcut and is shown in the tooltip. A
screen writes no code for it.

- One path for the key and the button (`useRemoveRow`): a grid with `fixedRows` has no column; a row
  that `canRemoveRow` refuses has no button and the key does nothing; after the removal the focus goes
  to the same column of the row that took its place (or the one above when it was the last).
- A screen can say what is under a row about to go with `removeQuestion(row)`, which returns the
  Fluent sentence (`{ messageId, titleId?, vars? }`) or null. The sale book's lots say "This lot and its
  2 sale lines will be removed." The key asks too.
- The column is not one of the grid's own columns (`REMOVE_COLUMN_ID`), so Enter never stops on it and
  the store never keeps it as the focused cell. Tab reaches its cell; Enter, Space and `Ctrl+Delete` on
  that cell press the button (`gridKeys.ts`).
- The arrows reach it too (Umar, plan 71 follow-up), in the reading direction of the grid: the arrow that
  reads on from the last cell of a row (← in Urdu, → in English) lands on the trash cell of that row, when
  the row has a button (`canRemoveRow`, not a grid with `fixedRows`); the other arrow goes back to the last
  cell; ↑ and ↓ go to the trash cell of the row above or below. ↓ on the trash cell of the LAST row goes to
  the "+" button under the grid (`addLink`, below); without a "+" it adds a row as ↓ does on any cell. ↓ from
  any other cell of the last row still adds a row (PRD FR-7.14), and Enter never stops on the trash cell.

A screen that has a "+" under its grid makes one link with `useAddRowLink()` and gives it to both:
`<EntryGrid addLink={link} />` and `<AddRowButton link={link} />` (`addLink.ts`). The grid puts the focus on
the button (a dimmed button, at the limit of rows, takes none), and the button's ↑ comes back to the last
row's trash cell.

## The "+" under a grid (plan 71 follow-up, Umar; PRD FR-7.3, NFR-USE-8, NFR-SCL-3)

`AddRowButton` is a round button on a row of its own under an entry grid, at the end side (left in Urdu,
right in English, with no physical side written). A screen gives it a Fluent message that is its name and
its tip (`labelId`, for example "New lot"), the `onAdd` that adds the row, and, at a limit, a `limit`
(`{ messageId, vars }`): the button is then dimmed and the words stand beside it, linked to it.

- `onAdd` must add through `addRowAndFocus(store, field)`, the one new-row path: `↓` on the last row,
  `Enter` on the last cell and the button all call it, so the rows, the cursor and the keyboard rules are
  the same. The sale book's lots go to the new lot's seller, its lines to the new line's buyer; in the
  whole-sale view the line goes to the lot of the line that had the cursor, else to the first lot.
- The limits are the screen's: 50 lots and 1,000 lines in a بکری (`features/bikri/limits.ts`, the same
  numbers as the backend). The button takes no room from the grid's rows and adds no width.

## The select cell (plan 58, PRD FR-12.42)

A `select` column holds one of a few fixed words, or nothing (the quality of a sale line: اول, دوم,
سوم). It is the shared way to have an optional fixed list in a grid.

- The column lists its words in `choices` (the one with the value `''` is the empty choice). The
  words are Fluent messages. It is usually `enterStop: false`, so Enter does not stop on it.
- The cell shows the word and a small arrow. A click, Space or Alt+↓ opens a list under the cell
  (`SelectCellEditor`). ↑ ↓ and Home and End move in the list, Enter or Space choose and the focus stays on
  the cell, Esc closes the list and keeps the old word, ← → and Tab leave the cell like in a picker
  cell. A letter or digit does nothing. Delete and Backspace clear a cell that is not being typed in, in every
  grid (below).
- What is chosen goes to the row store at once (`updateTyped`), so a language switch loses nothing.
- Test: `tests/unit/entry-grid-select.test.tsx`.

## Backspace and Delete (PRD FR-7.91, D8 Bug 2)

They only delete. In a text field, and in a cell being typed in, they delete text. On an editable cell that
is not being typed in they clear it (a chosen record of a picker cell is taken away), like a spreadsheet; a
calculated cell does nothing. The grid answers the key itself (`gridKeys.ts`), and the one listener of the
shell (`shared/keyboard/backspace.ts`, mounted with the Esc dispatcher) cancels any other Backspace that is
not in a text field. Without it WebKit, the web view of the Mac app, took Backspace on a cell, a button or the
page as "go back", and the leave guard asked "Save changes?".

## The end of an edit (`editEnd.ts`)

The store is the one truth of what was typed. When an edit ends with text, `syncEditedText` makes the store
hold that text, even if AG Grid decided the value had not changed. A cancelled edit has no text and is left
alone.
