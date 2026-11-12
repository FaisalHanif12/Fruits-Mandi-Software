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
| P13 | Bug found by Umar, 12-11-2026, added to this plan: on a sale line, changing the commission or لاگا rate from the default (for example seller commission 6% to 4%, FR-11.10) does not change the worked-out commission. The new rate must re-work the line at once, like any other typed value: 12,000 at 4% gives 480, and the line total, the lot and بکری summary and the partner split follow. Find the real cause with a failing test first. |
| P12 | Four states, both languages, every window size, keyboard only. Empty window message: "اس ہفتے کوئی بکری نہیں". |

## 3. Contract (backend proposes exact names; the lead records them here)

- `bikri_list` (or a new `bikri_past`) takes `from`, `to`, the filters, `show_finished`, `all` (P8), `offset`, `limit`, and returns the page of rows, `total_count`, the totals over the filter, and `days`: one entry per date in the range (including empty days), with `date`, `count`, `gross_paisa`.
- The row type carries the truck, lot names and the edit mark.
- No money maths: counts and sums by SQL `SUM()` on stored `*_paisa` columns.

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| T0 | PRD v1.35: rewrite 12.7, FR-5.3 menu row, any other mention; Architecture screen map (diagram 5); committed alone before the code | docs helper, lead |
| T1 | Command and query, day groups, totals, index migration if needed, EXPLAIN QUERY PLAN test, speed test on NFR-SCL-1 volume; `pnpm gen:types` | backend |
| T2 | The screen: sections, divider, arrows, keys, date jump, switch, row open, return after edit, print and Excel follow the switch; four states; both languages; keyboard; screenshots | frontend |

## 5. Done when

Every item above is proven by tests: backend for windows, filters, day totals and speed; screen for the two sections, divider, arrows, date jump, switch, open and edit, and the refreshed row after an edit, both languages, keyboard only. All existing tests pass, no golden number changes. Committed and pushed; the lead posts "PLAN 72 DONE" with the commands and real output.
