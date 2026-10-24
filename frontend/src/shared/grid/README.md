# shared/grid

AG Grid setup for every entry grid: right-to-left and left-to-right, the keyboard rules of
PRD 7.2 (Enter stops on every editable cell, skips calculated ones), and column types for money,
quantity, account and date with fixed widths (Code Standards F17, F18, F27). Arrives in plan 04
with the `entry-grid` skill. A grid never holds unsaved data on its own (F39).

## Lists that load page by page (plan 17b, PRD FR-19.11, Code Standards F34)

A list asks the backend for one page at a time (100 rows) and shows only the rows in view. The first
page comes with the count and the totals of the whole filter; the screen shows those, never a sum
of the rows it holds.

- `shared/hooks/usePagedQuery` is the one hook for it: `{ queryKey, fetchPage(offset, limit),
rowsIn, totalOf }`. `useLoadRest` reads the remaining pages while a screen needs the whole list for
  a moment (a typed filter matched against the rows, a record that may be on a later page).
- `PagedList` is the list with its four states (loading, error with a retry, empty with a helpful
  message, rows), the backend's totals row and the "Showing x of y / Show more" line. `onOpen`,
  `onFocusRow` and `focusRowId` are for a list whose rows open something or whose focused row the
  screen acts on. `PagedFooter` is the last line alone, for a screen that draws its rows itself.
- Used by: the cash book, partner takes, ادھار رقم, الگ کھاتہ, capital and the partner statement,
  آج کی بکری, باقی مال (the lots, then the firm's own goods, counted together), the voucher lists,
  the پختہ کھاتہ register, the masters, the rules, the accounts and the learned words.
- A language switch rebuilds the grid; the loaded pages live in the query, so nothing is lost.
