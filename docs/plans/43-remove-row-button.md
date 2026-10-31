# Plan 43: a visible button to remove a line from any entry grid

Version 1.0, 31-10-2026. Lead: tech-lead. Owner: `frontend` only. Found by Umar on 31-10-2026: four sale lines "Faisal F" were added by mistake and he saw no way to remove them.

## 1. Problem

Entry grids remove a row only with `Ctrl+Delete` (shared/grid/gridKeys.ts). There is no button. This breaks PRD FR-7.3 ("Keys are a shortcut, never the only way") and NFR-USE-8 (every task with the mouse only). On a Mac the key is Control + fn + Delete, which nobody will find.

## 2. Requirements

PRD FR-7.3, FR-7.4, Section 7.3 (`Ctrl+Delete`); NFR-USE-7, NFR-USE-8, NFR-USE-10; Code Standards F7, F12, F27, F28, F29. No requirement change.

## 3. What to build

- R1. In the shared `EntryGrid`, every editable row gets a small remove button at the row's end edge (trash icon from Lucide, 16 px), with the tooltip and accessible name "یہ لائن ہٹائیں" / "Remove this line" and the key `Ctrl+Delete` shown (FR-7.4). Every entry grid gets it with no screen code: بکری lots and sale lines, voucher lists, جنرل lines, purchase goods lines, and any other user of `EntryGrid`.
- R2. The button follows the same `canRemoveRow` rule as the key. Where a row cannot be removed (a saved lot of an edit), the button is not shown.
- R3. Removing a lot that has sale lines asks first, in one sentence, what will go. Removing a sale line or an empty row does not ask.
- R4. After removal, focus goes to the same column of the next row (or the previous one if it was the last). The live summary refreshes. A language switch after removal keeps the rest (F39).
- R5. The button is reachable by `Tab` and works with `Enter` and `Space`. It is never a stop for `Enter` moving between cells (FR-7.14).
- R6. Alignment follows plan 39 R3 (small column, centred). No sideways scroll at 1024 x 600.

## 4. Tests

Mouse removal on the sale lines, lots, voucher list and جنرل grids; `canRemoveRow` hides it; the lot question; focus after removal; Enter still skips it; ur and en screenshots at 1366 and 1024 in `frontend/tests/e2e/output/screenshots/plan43/`. Targeted runs only; `pnpm typecheck`, frontend lint and `pnpm i18n:check` clean.

## 5. Out of scope

Removing a line from a saved entry outside its edit (that is edit or cancel by reversal, PRD 12.9).
