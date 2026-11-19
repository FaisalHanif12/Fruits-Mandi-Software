# Plan 72: پچھلی بکری (Past Sales)

| | |
|---|---|
| Asked by | Umar, 12-11-2026 |
| Branch | `feature/d8-demo-feedback` (Mac phase) |
| Requirements | PRD 12.7 (rewritten), FR-5.3, FR-7.19, FR-7.70, FR-7.71, FR-12.13, FR-19.11, FR-21.4; NFR-SPD-24; Code Standards B31, F34, F8 |
| Requirement change | PRD v1.35 (this plan is Umar's approval) |

## 1. Goal

The screen "آج کی بکری" becomes "پچھلی بکری" (Past Sales). It shows today's بکری and, below a divider, a 7-day window of earlier days, grouped by day. Arrows and Page Up / Page Down move the window by 7 days; a date field jumps to any day; a switch shows every بکری of the window without the filters. Any row opens that بکری on بکری بک, where it can be edited (FR-7.70, FR-21.4); after the save the user comes back to the same window and row, and every figure shows the new values. Years of data stay fast, because only what is on screen is loaded.

## 2. Decisions (Umar's text, and the defaults of his open points)

| # | Decision |
|---|---|
| P1 | Menu item and title: پچھلی بکری / Past Sales. Fluent ids may stay. |
| P2 | Everything on the screen today stays and works: search (بکری number or truck number), date, بیوپاری, قسم مال and حالت filters, the filter counter, کم فلٹر, بند / منسوخ بھی دکھائیں, مزید کالم, چھانٹیاں ہٹائیں. |
| P3 | The truck column and the truck search come back on this screen (they were removed in plan 71 from the today list). A new بکری has no truck, so its cell is empty; an old one shows its truck. |
| P4 | Section 1, آج کی بکری: every بکری of today. Section 2, پچھلے 7 دن: below a divider bar that shows the range, "05-11-2026 تا 11-11-2026", grouped by day, newest first; each day heading shows its date, its بکری count and its خام بکری total. |
| P5 | Q1 default: the window ends yesterday (today has its own section). Arrows move it by 7 days; it never goes past yesterday. Arrows follow the reading direction. Page Up goes back 7 days, Page Down comes 7 days nearer. Example: today 12-11-2026: first window 05-11-2026 to 11-11-2026; one step back 29-10-2026 to 04-11-2026. |
| P6 | "تاریخ پر جائیں" date field above the filters, with the shortcuts of FR-7.19. It shows the 7-day window that ends on that date. A date after today is refused. A date of today shows today's section and the window ending yesterday. |
| P7 | Q2 default: a day with no بکری shows in the window as a heading with "کوئی بکری نہیں". |
| P8 | Q3 default: "پوری بکری دکھائیں" is unticked on open. Ticked: filters are set aside (they keep their values), every بکری of today and of the window shows, including closed and cancelled. Printing and Excel follow the same choice. |
| P9 | Row: بکری number, date, time, truck, seller(s) (lot names of plan 71), items, نگ, خام بکری, صافی بکری, buyer total, status, and the edited or cancelled mark (FR-7.71). Stored values only; nothing is worked out again. |
| P10 | Click or Enter opens the whole بکری on بکری بک. Its Edit works as FR-7.70; on a closed day through FR-21.4. After a save the user comes back to پچھلی بکری at the same window and row, and the row, its day heading and the window totals show the new values at once (the screen asks the backend again; no figure is kept on the screen). |
| P11 | Large data (B31, FR-19.11, NFR-SPD-24): today's section and the window each come from the backend as one indexed query per page, with totals over the filter from the backend; day headings' counts and totals from one grouped indexed query. Moving loads only that window. EXPLAIN QUERY PLAN checked, a missing index added in a new numbered migration. Speed on 10 years of test data: open under 1 s, arrow move under 300 ms, go to date under 1 s. |
| P10a | Umar, 12-11-2026: editing from پچھلی بکری must be smooth. Each row has the shared Edit and Cancel buttons of FR-7.70 (as every other list): Edit opens the بکری straight in edit mode, a closed day through FR-21.4; Cancel asks the reason and stays on the list. بکری بک opened from the list has a "پچھلی بکری پر واپس" button. After save, cancel, undo or Esc the user is back on the same window and row with fresh figures. |
| P6a | Lead, 12-11-2026: the one date box of the screen is "تاریخ پر جائیں"; it replaces the old day filter, because a day filter and a 7-day window cannot both pick the day. A whole empty window shows "اس ہفتے کوئی بکری نہیں"; a window with some بکری shows its empty days as headings (P7). Route and file names stay `/bikri/today` and `TodayBikriPage.tsx`. |
| P4a | Umar, 12-11-2026: page order, modern and clean: (1) آج کی بکری with its section header (date, count, خام بکری); (2) a quiet divider with no buttons: a thin rule and a centred label "پچھلے 7 دن" with the range; (3) پچھلے 7 دن کی بکری with its header and modern day headings; (4) at the end, after the table and pager, the week arrows "پچھلے 7 دن" and "اگلے 7 دن", each showing the range it goes to, and "تازہ ہفتے پر واپس" when not on the latest week. Keys unchanged. |
| P14 | Umar, 13-11-2026: less focus on the AI assistant, more on core functionality. Assistant work already written is finished only if its tests pass at once; no new assistant work in this plan. Free agents move to core: پچھلی بکری, the party ledger page speed (D8 robustness MINOR 2), and a regression sweep of the core screens. |
| P15 | Umar, 13-11-2026: on بکری بک, a modern "+" button at the bottom of the lots grid and of the sale-lines grid, on the left in Urdu (the end side), adds a new lot or a new sale line and puts the focus on its first cell. It uses the grid's existing new-row path (the same as ↓ on the last row), is a shared part of shared/grid, and is off at the limit of NFR-SCL-3 (FR-7.3, NFR-USE-8). Umar, 13-11-2026: the old "لاٹ شامل کریں" button above the lots grid is removed; only the "+" stays. |
| P16 | Umar, 13-11-2026: no more time on speed or extra hardening now; finish the core functionality perfectly first. Speed and robustness extras come later. |
| P17 | Umar, 13-11-2026: the assistant's chat window header shows its name, "عدنان اسسٹنٹ" / "Adnan Assistant" (FR-7.73 "a header with the assistant's name"). The status bar button stays "سوال پوچھیں F1". |
| P18 | Umar, 13-11-2026: in the entry grids the arrows reach the delete cell (the last stop of a row toward the end; Enter or Space removes the row as a click does) and, from the last row's delete cell, ↓ reaches the grid's "+" (Enter adds a row and puts the cursor on its first cell; ↑ goes back). ↓ from any other cell of the last row still adds a row (FR-7.14). The Enter path of a line is unchanged. |
| P13 | Bug found by Umar, 12-11-2026, added to this plan: on a sale line, changing the commission or لاگا rate from the default (for example seller commission 6% to 4%, FR-11.10) does not change the worked-out commission. The new rate must re-work the line at once, like any other typed value: 12,000 at 4% gives 480, and the line total, the lot and بکری summary and the partner split follow. Find the real cause with a failing test first. |
| P12 | Four states, both languages, every window size, keyboard only. Empty window message: "اس ہفتے کوئی بکری نہیں". |

## 3. Contract (backend proposes exact names; the lead records them here)

- `bikri_list` (or a new `bikri_past`) takes `from`, `to`, the filters, `show_finished`, `all` (P8), `offset`, `limit`, and returns the page of rows, `total_count`, the totals over the filter, and `days`: one entry per date in the range (including empty days), with `date`, `count`, `gross_paisa`.
- The row type carries the truck, lot names and the edit mark.
- No money maths: counts and sums by SQL `SUM()` on stored `*_paisa` columns.
- Recorded 12-11-2026 (backend-past): no new command. `BikriListInput` keeps its fields and gains `all: boolean` (seller, item, status, search and show_finished set aside, cancelled included, dates still apply) and `group_by_day: boolean` (needs from and to, or date; else `ERR_REQUIRED`; to before from `ERR_DATE_RANGE`; more than 366 days `ERR_OUT_OF_RANGE` with max 366). `BikriList` gains `days: BikriDay[]`, `BikriDay = { date, count, gross_paisa }`: one per date, empty days included, newest first (oldest first only for sort entry_date asc); same filter as `total_count` and `totals.gross` (gross leaves cancelled out); Σ count = total_count, Σ gross = totals.gross; empty unless group_by_day. A date after today on any `bikri_list` call is `ERR_DATE_IN_FUTURE` with `today`. `BikriListRow` already carries truck, lot names, sellers, items, status and the edit flags. Today's section: from = to = today, group_by_day. Window: from = to − 6, to = yesterday; paged 25/50/100. No new index needed (EXPLAIN shows no table scan), so no migration 0023. Speed on 40,650 بکری and 1,138,200 sale lines: open 23 ms, each move 2 to 23 ms.
- Print and Excel of the list (P8, lead's decision 12-11-2026): two commands that reuse the list query, not a new report number, so the assistant's catalogue does not change.
  - `bikri_list_print_data`: input as `bikri_list` (`from`, `to`, filters, `show_finished`, `all`) plus the print language, no paging. Output: the shared print header (FR-19.13) with the filters and the switch named; `today` (its date, rows, count, خام بکری); `days` (one per date of the window, newest first, empty days included, each with `date`, `count`, `gross_paisa` and its rows); `totals` over the filter. Rows are the P9 rows.
  - `bikri_list_export_excel`: the same input; writes the same rows with today's section and the day headings through the existing Excel writer (`rupees_as_f64`, `count_as_f64` only, Code Standards M13), and answers like the other exports.
  - `bikri_list_print { list, to, copies }` → `PrintOutcome`, the same shape as `reports_print_collection`, runs Print and PDF through the hidden print window (lead, 12-11-2026). The print window capability allows `bikri_list_print_data` and `reports_print_header`. The print data names its title with the Fluent id `bikri-list-print-title`. `list_whole` reads today's section and every page of the window in one read snapshot (money review). `search` over 200 characters, and the truck and buyer text of `bikri::find`, are refused with `ERR_TEXT_TOO_LONG` (robustness finding 1).
  - Exact type and field names are confirmed by `backend-past` when the bindings are generated, and recorded here.

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| T0 | PRD v1.35: rewrite 12.7, FR-5.3 menu row, any other mention; Architecture screen map (diagram 5); committed alone before the code | docs helper, lead |
| T1 | Command and query, day groups, totals, index migration if needed, EXPLAIN QUERY PLAN test, speed test on NFR-SCL-1 volume; `pnpm gen:types` | backend |
| T2 | The screen: sections, divider, arrows, keys, date jump, switch, row open, return after edit, print and Excel follow the switch; four states; both languages; keyboard; screenshots | frontend |

## 5. Done when

Every item above is proven by tests: backend for windows, filters, day totals and speed; screen for the two sections, divider, arrows, date jump, switch, open and edit, and the refreshed row after an edit, both languages, keyboard only. All existing tests pass, no golden number changes. Committed and pushed; the lead posts "PLAN 72 DONE" with the commands and real output.

## 6. Follow-ups

- The report print header shows the lot only when the backend sends it as a print fact `report-filter-lot`; the screen and the Fluent label are ready. Parked by P14 (assistant path).
- PRD 7.3 keys table: add Page Up / Page Down for پچھلی بکری (already in FR-12.46).
- Summary cards: in Urdu the values sit at the physical left end of each row, while NFR-USE-19 and F17 say a card's values are on the physical right edge. It predates plans 71 and 72 (every card using value-col). Look at it as its own item.
- PRD FR-7.19 wording: "-" (and "+") change the day only when the date field is empty or its whole value is selected; after a digit "-" types a dash, so "10-9" and "05-10-2026" can be typed (lead decision 13-11-2026 on robustness finding 1; the PRD text says both and must be clarified at the next version).
