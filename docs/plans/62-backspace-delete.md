# Plan 62: Bug 2, Backspace and Delete open the "save changes?" box

Deliverable D8, item 2. Requirements: PRD FR-7.9, FR-7.14, FR-7.15, FR-7.18, new FR-7.91 (v1.32); NFR-USE-7. Decision K1 in plan 60.

## 1. Goal

Backspace and Delete only delete. In a field they delete text. On a grid cell that is not being edited they clear an editable cell, like a spreadsheet. They never leave the screen and never open the leave box, on every entry screen, on Mac and Windows.

## 2. Lead's first reading of the cause (to be proved)

Nothing in `shared/keyboard` handles Backspace outside a text field, so with focus on a grid cell or a non-text element the web view's default runs: going back in history. `LeaveGuard` (`useBlocker`) then opens the box. `frontend` proves this with a failing test on the real route, and writes the real cause here.

## 3. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | A failing test: focus a grid cell (not editing) with unsaved work, press Backspace and Delete: the leave box opens or the screen changes. | frontend |
| T2 | One fix in the shared keyboard layer (rule 10), not per screen. Grid cells clear on Backspace and Delete, read-only cells do nothing. Register it in `shared/keyboard/shortcuts.ts` if it is a shortcut. | frontend |
| T3 | A keyboard test over every entry screen: بکری, later sale, voucher (وصولی, ادائیگی, پیشگی), خرچہ, پھل, ادھار رقم, الگ کھاتہ, سرمایہ, جنرل, trips, gardens, masters forms, account form. Backspace and Delete never open the box or change the route. | frontend |
| T4 | Check the dialogs and the picker still work as FR-7.16, RomanUrduInput's Backspace-after-conversion still works (FR-7.22). | frontend |

## 4. Acceptance (D8-A5)

The test of T3 passes, and failed before the fix on at least the بکری screen.

## 5. Real cause

To be filled.
