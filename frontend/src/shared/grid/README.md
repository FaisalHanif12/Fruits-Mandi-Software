# shared/grid

AG Grid setup for every entry grid: right-to-left and left-to-right, the keyboard rules of
PRD 7.2 (Enter stops on every editable cell, skips calculated ones; a column can say it is not an
Enter stop, or decide per row with `enterStop: (row) => boolean`, as the item lines of plan 35
do), and column types for money,
quantity, account and date with fixed widths (Code Standards F17, F18, F27). Arrives in plan 04
with the `entry-grid` skill. A grid never holds unsaved data on its own (F39).

## ListGrid (read-only lists)

`ListGrid` is the one grid for reading a list. Besides the plain text, number, date and status
columns, it has (plan 20, for the report tables): `unit`, a short word after a figure in a slot of
fixed width so the figures keep one right edge (the side of a balance; column type `balance`);
`wrap`, a long text that wraps and makes its row taller; and `isGroup`, a row that heads a group.
The totals row is pinned under the last row and carries the backend's figures.

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
  the پختہ کھاتہ register, the rules, the accounts, the trips and gardens (and the history inside
  one), the audit log, the backup list, the learned words, the reports and the analytics views.
- In tests, `tests/setup.ts` forgets the pages after each test (`resetPagerMemory`).

A mouse click on an editable picker cell starts its editor with the full list open (plan 32, R7),
like a click on the field of a form. A key never does: arriving on the cell by Enter or an arrow
opens nothing, and `Alt+↓` is the key for the list (PRD 7.2, FR-7.16).
