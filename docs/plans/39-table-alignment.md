# Plan 39: every table lined up, heading over its content

Version 1.1, 30-10-2026 (1.1: Umar chose "centred under the heading" for numbers; documents raised to PRD 1.9, NFR 1.6, Tech Stack 1.7, Code Standards 1.10). Lead: tech-lead. Asked by Umar on 30-10-2026 while testing: "all the table headings along with the table content must be aligned, so it does not stay cluttered or disorganised ... our whole app displays data as tables, so go through each page, then make the UI perfectly aligned." Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`. Owner: `frontend` only. No backend, money or text change.

## 1. Goal

On every screen and every print, each column heading stands exactly over its own figures or words, every column of one kind looks the same everywhere, and the totals sit under their columns. A munshi reads any table at a glance, in Urdu and in English.

## 2. Requirements (already in the documents, now applied everywhere)

NFR-USE-19 (v1.6), NFR-USE-28; Code Standards F17 (v1.10), F18, F19, F15, F24, F37; PRD FR-19.5, FR-19.13.

The alignment rules (R1 to R8). The shared grid (`shared/grid/grid.css`, `columnTypes.ts`) already has most of them; the work is to find every table that does not use them and bring it in line.

| # | Rule | Example |
|---|---|---|
| R1 | **Number columns** (money, qty, rate, weight, code, short code, date, number of a record, days): the figures, the heading and the total all **centred** in the column, in both languages. Equal-width digits, written left to right. (Umar 30-10-2026, PRD 1.9, NFR-USE-19 v1.6, F17 v1.10.) | 17 and 250 each sit under the middle of "تعداد / Quantity", and the total 272 under them. |
| R2 | **Text columns** (names, item, narration, reason, city): the words and the heading both on the start edge (right in Urdu, left in English). | "گاہک" and "اکرم ٹول ٹیکس" start at the same right edge. |
| R3 | **Small columns** (a tick, a status chip, an icon or a button): the content and the heading centred. | The "ادھاری / نقدی" chip is centred under its heading. |
| R4 | **Headings**: never cut off and never broken in the middle of a word. A long heading wraps to at most 2 lines; all headings of one table have the same height and sit in the middle of the heading row. | "Total commission" on two lines, "Qty" on one, both centred top to bottom. |
| R5 | **Same kind, same width** everywhere (F18): a money column, a qty column, a date column, a code column each have one width token, used by every table and grid. A name column takes the room left. | The date column is the same width in today's sales, the ledger and the cash book. |
| R6 | **Totals row** (F19): each total under its own column with that column's alignment (a number total centred); the word "میزان / Total" in the first text column; a month heading row (plan 38) runs across the table from the start edge. | میزان 31,564 centred under the amounts. |
| R7 | **Rows**: one row height per table, contents in the middle of the row, padding from tokens. No cell text touching a border. | |
| R8 | **Label and value cards** (summaries, party card, preview cards, the شراکت دار overview): labels on the start edge, values in one column on the physical right edge, like a two-column table. | Sale summary: "خام بکری ...... 29,400" with every figure ending at the same edge. |

All rules hold in Urdu and in English, at 1366 x 768 and 1024 x 600, and in print (A4, A5, 80 mm). The client layouts of R5 and R35 keep their columns and order (FR-19.12); only their alignment follows R1 to R8.

## 3. Tasks

| # | Task | Owner |
|---|---|---|
| A1 | **Audit.** Every screen with a table, grid, list, report, ledger, register, analytics view, summary card or print template. Take shots in ur and en at 1366 and 1024 (`frontend/tests/e2e/output/screenshots/plan39/before/`). Write an automatic check that measures, for each column, the edge of the heading against the edge of its cells and of its total (allowed difference 2 px), and that headings are not cut. Write `FINDINGS.md`: per screen, which rule fails and where. | frontend |
| A2 | **Fix in the shared parts first** (`shared/grid`, `shared/ui/Table`, `shared/print`, card components, width tokens in `styles/tokens.css`), then the screens that build their own columns or tables. No one-off styling (F7, F9). | frontend |
| A3 | **Prove it.** The check of A1 passes on every screen and print in both languages and both sizes; after shots in `plan39/after/` with side-by-side sheets; the component and screenshot tests of the touched parts pass (targeted runs only, Umar's rule). `pnpm lint`, `pnpm typecheck`, `pnpm i18n:check` clean. | frontend |

Fix directly after the audit; no stop for approval (Umar asked for the fix). Anything that would change a column's content, order or a word is not in this plan: list it in FINDINGS.md as a question.

## 4. Acceptance

- The alignment check passes on every screen and print, ur and en, 1366 and 1024. It stays in the test suite so a new table cannot break the rule.
- Before and after sheets for every screen.
- No change to any number, word, column order, keyboard behaviour or saved data. Golden tests untouched.

## 5. Out of scope

New columns, new screens, changed words, the 1024 x 600 layout misses from plan 38 that are not about table alignment.
