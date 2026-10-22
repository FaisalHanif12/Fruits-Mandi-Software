# shared/grid

AG Grid setup for every entry grid: right-to-left and left-to-right, the keyboard rules of
PRD 7.2 (Enter stops on every editable cell, skips calculated ones), and column types for money,
quantity, account and date with fixed widths (Code Standards F17, F18, F27). Arrives in plan 04
with the `entry-grid` skill. A grid never holds unsaved data on its own (F39).

## ListGrid (read-only lists)

`ListGrid` is the one grid for reading a list. Besides the plain text, number, date and status
columns, it has (plan 20, for the report tables): `unit`, a short word after a figure in a slot of
fixed width so the figures keep one right edge (the side of a balance; column type `balance`);
`wrap`, a long text that wraps and makes its row taller; and `isGroup`, a row that heads a group.
The totals row is pinned under the last row and carries the backend's figures.
