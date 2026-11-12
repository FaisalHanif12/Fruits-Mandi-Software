# Plan 74: one party's statement on پارٹی کھاتے (30-day window, day in place, statement print)

| | |
|---|---|
| Asked by | Umar, 13-11-2026 (he named it plan 73; the number 73 was already taken by the Owner's sign, so this is plan 74) |
| Branch | `feature/d8-demo-feedback` (Mac phase) |
| Requirements | PRD v1.37 FR-12.47 (new), FR-12.43, FR-19.7, FR-19.8, FR-19.10, FR-19.11, FR-19.12 (R35), FR-19.13, R13, R14; NFR-SPD-14, NFR-SPD-24; Code Standards B24, B31 |
| Pattern to follow | Plan 72 (پچھلی بکری): window, arrows, Page Up/Down, go-to-date, four states |

## 1. Goal

A party's statement grows every day. The page shows 30 days at a time, opens a day's detail under its row, and prints a compact statement for any period. It stays fast with 10 years of data.

## 2. Decisions

| # | Decision |
|---|---|
| W1 | One row per date with an entry, 30-day window, newest first. Q1 default: the first window ends today. |
| W2 | "پچھلے 30 دن" and "اگلے 30 دن" under the table, with the window between them, in the style of plan 72's week buttons. Next stops at today. Page Up and Page Down do the same. Arrows follow the reading direction. |
| W3 | The میزان row is the window's: opening, charged, paid, closing, all from the backend. The newest window's closing equals the party card and R13's closing. |
| W4 | از and تا stay. A range lists every entry date in it, page by page, with its own opening and closing. |
| W5 | "تاریخ پر جائیں" (FR-7.19 shortcuts). With an entry: that row is open and highlighted inside its 30-day window. With none: "اس تاریخ کو کوئی اندراج نہیں" and the window around it. A date after today is refused. |
| W6 | A date row opens under itself on click or Enter (accordion); a second click closes it. Several can be open; a window move closes them. The day is the R35 day slip data from the same backend function, with its own print button. |
| P1 | "اسٹیٹمنٹ پرنٹ" at the top: period (آخری 10 دن, آخری 30 دن default, آخری 6 مہینے, اپنی تاریخیں), print language, preview, print, PDF. |
| P2 | Q2 default: oldest first. Q3 default: each day's items, compact. Contents and layout as PRD FR-12.47. |
| P3 | One calculation path: the R13, R14 and R35 functions. No new money maths. |
| S1 | Speed, 10 years for one party: open under 1 s; a window move and a day opening under 300 ms; a 6-month statement preview under 2 s. EXPLAIN QUERY PLAN checked; an index in a new numbered migration only if missing. |

## 3. Contract (backend proposes exact names; the lead records them here)

- The statement read takes `account_id`, `from`, `to`, `offset`, `limit`, and answers the page of date rows, `total_count`, and the window's opening, charged, paid and closing.
- Go to date: `account_id`, `date`. Answers the 30-day window that holds it (`from`, `to`), whether the date has an entry, and the page and row to show.
- The day detail is the existing R35 day read (unchanged).
- Statement print: `party_ledgers_statement_print_data { account_id, from, to, language }`, a print job and PDF the same way as the other report prints, and its row data built from the R13 and R35 functions.

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| T0 | PRD v1.37 FR-12.47 (done by the lead, committed before the code) | lead |
| T1 | Window and range reads with backend totals, go-to-date, statement print data and job, tests (the example of FR-12.47, window ends, range, go-to-date found and not found, closing equals R13), EXPLAIN test, `pnpm gen:types` | backend-lots |
| T2 | The statement page: window, arrows, keys, go-to-date, the day opened in place, four states, both languages, keyboard | frontend-past |
| T3 | The statement print box and print view (compact), preview, print and PDF; tests in both languages; a 30-day PDF as proof | frontend-lots |
| T4 | Speed on 10 years for one party (open, move, day, 6-month preview) | robustness, short run at the end |

## 5. Worked numbers

Buyer, 01-11-2026 to 06-11-2026. Opening 0. 05-11: bought 30,000, paid 0, balance 30,000 بنام. 06-11: bought 21,360, paid 21,360 on the spot, balance 30,000 بنام. Totals: bought 51,360; paid 21,360; closing 30,000 بنام.

## 6. Done when

Backend tests for windows, range, go-to-date, window totals, opening and closing, statement data with the example. Screen tests for arrows, go-to-date found and not found, a day opening and closing in place, the print box with each period, both languages, keyboard only. A 30-day statement PDF. Speed numbers on 10 years. All existing tests pass, no golden number changes. Committed and pushed; the lead posts "PLAN 74 DONE" with commands and real output.
