# Plan 38: clean, professional pages and page buttons on every list

Version 1.0, 30-10-2026. Lead: tech-lead. Asked by Umar on 30-10-2026 ("the whole app is writing data and displaying it ... make it more effective, professional and perfect in displaying experience; use pagination so it is not cluttered; keep the user experience highly prioritized; go through all the pages"). His choice: page buttons. Branch `feature/d5-early`.

## 1. Requirements

R1. **One pager for every list** (PRD FR-19.11, v1.8): every list, ledger, register, report table and analytics table that loads page by page shows, under the table: rows per page (25, 50, 100; default 25, remembered per screen in `user_prefs`), "صفحہ 2 از 14" / "Page 2 of 14", the row count ("340 میں سے 26 تا 50" / "26 to 50 of 340"), and buttons First, Previous, Next, Last. Page Up / Page Down move a page when the table has focus (PRD FR-7.17). The totals row is always over the whole filter (unchanged rule). Changing a filter goes back to page 1. The selected row stays selected when the language is switched. Print, PDF and Excel still hold every row. Built once in `shared/grid`; no screen builds its own.

R2. **A clean look on every screen**, checked one by one against this list:
- one clear title and one short help line; the main action in the same place on every screen (design system);
- filters in one tidy row; rarely used filters behind "مزید فلٹر" (More filters) at 1024 and 1366 (open point from D5);
- tables show the columns used every day; rarely used columns behind "مزید کالم" (More columns), as the accounts list does;
- numbers right-aligned with equal-width digits; totals under their columns; no text cut off (Urdu ink included, plan 36 fix);
- every empty, loading and error state designed (F8);
- no sideways scroll at 1024 x 600 and 1366 x 768, in Urdu and English (NFR-USE-15, NFR-USE-28);
- spacing and sizes only from the tokens (F9 to F11); cards and sections aligned on the grid (F15).

R4. **Data over many years** (Umar, 30-10-2026: "this software will keep going for years; keep the data professional and organized, without clutter"):
- **A sensible starting period, never "everything":** day lists (آج کی بکری, وصولی and ادائیگی lists, روکڑ بک) open on today; registers and reports on this month; ledgers and statements on this year, with one click to "from the first date". The date filter offers today, this week, this month, last month, this year, last year and a custom range (FR-19.2). The last choice is remembered per screen (FR-7.45).
- **Finished records step aside:** closed trips and gardens, cancelled entries and inactive records are hidden by default, behind a tick "بند / منسوخ بھی دکھائیں" (show closed and cancelled too). Nothing is deleted.
- **Month and year headings** in long registers and ledgers (a heading row per month, with that month's subtotal); the grand total stays the whole filter's (B31). Subtotals come from the backend, never summed on screen (F34).
- **Sort by a column:** a click on a column title sorts by it, a second click reverses; each list opens in its natural order (newest first, or by code for master lists). Sorting is done by the backend over the whole filter, never only on the page.
- **Fixed header and totals row** while moving through a page.
- **A search box in every list** (code, name or number) in addition to its filters.
- **Speed with years of data:** first page with totals under 1 second, each next page under 300 ms (NFR-SPD-24), proved with the 10-year test data.

R3. Nothing changes in money, postings or what is saved. Only how pages show data.

## 2. Phases

| # | Task | Owner |
|---|---|---|
| B1 | Check every list command accepts `offset` and `limit` 25, 50 or 100 and returns the whole-filter `total_count` and totals (B31). List any that do not and fix them, with tests; `pnpm gen:types` if needed | backend |
| B2 | R4 on the backend: a `sort` (column and direction) on every list command, done in SQL over the whole filter with indexes; month subtotals for registers and ledgers (`group_by_month`); the closed and cancelled filter where missing; a speed test with the 10-year data for the first page and the next page | backend |
| P1 | The shared pager (R1) in `shared/grid`, used by every paged list; rows-per-page pref; keys; tests (paging, filter resets to page 1, totals over the whole filter, language switch, both directions) | frontend |
| P2 | R4 on the screens: default periods, the show-closed tick, month heading rows, column-title sorting, fixed header and totals, a search box; all in `shared/grid` and the shared filter bar so every list gets them | frontend |
| A1 | Audit: a screenshot of EVERY screen (ur and en, 1366 and 1024) through the dev pages (add a dev page for any screen not covered yet: Index lists, chart, rules, openings, partners setup, day close, audit, sign-in), and a findings list per screen against R2 with the fix proposed. Send it to the lead before changing screens | frontend |
| A2 | The lead reviews the findings and a sample of shots, approves the list (and asks Umar where a choice is his) | tech-lead |
| F1 | The approved fixes, screen by screen, with screenshot proof before and after | frontend |

Targeted checks only.

## 3. Acceptance

R1 on every paged list (a test per list type); the A1 shots before and after; no sideways scroll at 1024 in both languages; no change in any saved number or fixture value.
