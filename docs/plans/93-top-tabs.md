# Plan 93: The second menu becomes a row of tabs at the top

| | |
|---|---|
| Status | Umar's request and decisions, 22-11-2026 (screenshots of بکری بک and روکڑ) |
| Branch | `fix/vouchers-edit-sign` (with plan 92; one pull request) |
| PRD | FR-5.3 (v1.56), FR-7.17 (v1.56), FR-7.92, NFR-USE-10, NFR-USE-15, NFR-USE-16, Code Standards F7, F12, F22, F24, F27 |

## Goal

The second menu of بکری (7 items) and روکڑ (its items) moves from a vertical side panel to a row of tabs at the top, so the screen gets the width the panel took. It must look modern, organized and professional.

## Design

- The tabs sit in the page's title row: icon + short label, the active one clearly marked (filled pill or underline in the design tokens, like the existing segmented switches). The active tab is the page title, so no vertical space is lost. The page's own buttons (save, print ...) stay in the same row at the end side, or on the row below if it is too tight.
- Labels: بکری بک, وصولی (not "وصولی / اگراہی"), ادائیگی, پیشگی کھاتہ, پچھلی بکری, پارٹی کھاتے, باقی مال. روکڑ: its current items with the same names. English the same way.
- At a narrow window the tabs that do not fit go into a "مزید ▾" menu, never on two lines (1024 x 600 must work).
- One shared component in `shared/ui` (or `app/layout`, replacing `ModuleMenu`), used by every section with sub-pages.
- Keyboard: `←` `→` between tabs in the reading direction, `Enter` opens, `↓` into the screen (FR-7.17 v1.56); every tab reachable by keyboard with a visible focus.
- The voucher screen keeps its in-page type switch (FR-7.67); the وصولی, ادائیگی and پیشگی tabs open it with that type, as the side menu did.
- The Back button (FR-7.92) and unsaved-work question (FR-7.9) work as before.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Frontend: the tabs component, بکری and روکڑ use it, the side panel removed, Fluent label وصولی, keyboard, both languages, tests | `frontend` |

## Acceptance (the lead checks in WebKit before Umar sees it)

- بکری بک and روکڑ at 1024x600, 1366x768, 1480x830 and 1920x1080, ur and en: the tabs on one line (or a مزید menu), no horizontal scroll, the lot grid wider than before.
- Every tab opens its page; the active one is marked; keyboard path works.
- typecheck, lint, i18n:check and the touched tests pass; screenshot tests updated.

## Small tweaks (Umar, 22-11-2026)

- T2: the logo in the main side bar (beside "منڈی سافٹ ویئر") shows the fruit picture of the app icon again (`backend/app/icons/icon.png`, bundled with the frontend, no network), as it was before the crate-only change (`git show f81ad78d^:frontend/src/shared/ui/BrandMark.tsx`). Only in the main side bar.
- T3: the sign-in page side panel shows no logo icon at all (the gold crate box goes); keep it simple: the firm name and text only.
- The installer icon itself stays as it is (Umar, 22-11-2026).

- T4: the scroll line (scroll bar) of the page and panel scroll areas is not shown, on Mac and Windows; scrolling by wheel, trackpad, keys and drag still works. One shared rule in `styles/globals.css` (for example a `no-scrollbar` utility on the main screen area and the inner panels: `scrollbar-width: none` and `::-webkit-scrollbar { display: none }`). Data grids and wide tables that scroll sideways keep their own behaviour.

| # | Task | Owner |
|---|---|---|
| T2, T3, T4 | Frontend: the two logo tweaks and hidden scroll bars, both languages, tests updated | `frontend` |
