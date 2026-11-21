# Plan 86: A dropdown opens upward when there is no room below

| | |
|---|---|
| Status | Umar's request, 20-11-2026 (screenshot: the item picker in a garden's goods arrived line, near the bottom of the window; the list is cut off by the status bar and the window edge) |
| Branch | `fix/gardens-new-form` (one pull request for plans 81 to 86) |
| Rules | NFR-USE-16 (nothing overlaps or gets cut off), NFR-USE-15, FR-7.10, FR-7.16, Code Standards F7 (one shared component), F24 (logical properties) |

## Goal

Every list that drops down (the shared pickers: AccountPicker and the master pickers, the select boxes, the grid cell pickers such as the item and packing cells, the typing suggestions of RomanUrduInput, date pickers, menus) opens **below** its field when it fits, and **above** it when the room below is too small and there is more room above. All its options stay visible inside the window: the list never goes under the status bar, the assistant button or the window edge; it scrolls inside itself when it is longer than the room. It also stays inside the window sideways (RTL and LTR). It moves with its field when the page or a grid scrolls, and when the window is resized.

## How

- One shared positioning piece in `frontend/src/shared/ui` used by every dropdown. Use Radix's popper positioning (already in the stack through Radix; no new dependency) or the same logic in our own small hook: side `bottom`, flip to `top`, a collision padding that keeps clear of the status bar and the window edges, `max-height` = the room left on the chosen side, inner scroll.
- The list is drawn in a portal above the page (so a grid cell or a card with `overflow: hidden` does not cut it), with the same keyboard behaviour as today (FR-7.16: open on focus, `↑` `↓`, `Enter`, `Esc`), the same focus outline and both directions.
- In the screenshot the item cell of the goods arrived grid opens a second box under the cell before its list. Check that the grid cell picker opens its list directly from the cell, like the sale line pickers, so the list starts at the cell.

## Acceptance

- A picker in the last row near the bottom of the window (the garden goods arrived line, a بکری sale line near the bottom, a voucher list's last row) opens upward with every option visible or scrollable; near the top it opens downward.
- Tests (jsdom with mocked sizes): flips up when the room below is under the list height and more room is above; max height follows the room; stays within the window sideways in ur and en; keyboard unchanged; inside an overflow-hidden grid the list is not cut.
- Screenshots at 1366x768 and 1024x600 in Urdu and English of a bottom-row picker opening upward.
- typecheck, lint, i18n:check and the touched tests pass; the existing picker and grid tests stay green.
