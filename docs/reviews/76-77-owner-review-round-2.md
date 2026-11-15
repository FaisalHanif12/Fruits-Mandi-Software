# Review: plans 76 and 77 (owner review round 2, commission filter, kharch page, analytics)

Goal run, branch feature/d8-demo-feedback, 14 and 15 November 2026. One review and one robustness pass for both plans (tech-lead.md, Goal runs).

## Verdicts

| Plan | reviewer | robustness |
|---|---|---|
| 76 (11 owner points) | PASS WITH MINOR, all findings fixed and rechecked | PASS (after the speed fixes) |
| 77 (commission party and period, views L and M, kharch page, assistant party commission) | PASS (after the grouped E/L fix and the grouping on A, B, C, E, L) | PASS WITH MINOR (one row 5% over the 4 s budget of NFR v1.19) |

## reviewer findings and what happened

Plan 76 (first report, PASS WITH MINOR):
1. MINOR: the overall sale print for one بکری listed the whole day of each party. Fixed: only that بکری's lines, with the day's balances; a mixed save prints the new بکری and the carried lots' old بکری (`bikri_ids`, `print_bikri_ids`).
2. MINOR: the R5 sheet total could fall below R20 when a buyer starts the day in credit. Fixed: buyers ending in جمع are listed apart in a second total row (`credit_totals`); the main total equals R20.

Plan 77:
1. MAJOR: views A, B, C, E, L had no day, month or year grouping (FR-18.9). Fixed: row grouping on every view except H.
2. MAJOR: grouped E and L could drop an amount when more than 24 items had sales. Fixed (`every_item_column`, checked fold), test with 25 items over two months.
3. MINOR: no test that paged grouped rows equal the full read. The test existed and was strengthened (three بکری across two months).

Speed-fix recheck (plan 76): PASS WITH MINOR. Fixed: buyer dues last-change days read in the same snapshot; lent_dues people list paged (no cut at 200). Result cache and paging (plan 77): PASS, no stale or cross-file answer found.

## robustness findings and what happened

Plan 76 first report: FAIL on NFR-SPD-24 at 10 years (collection sheet 46 s, buyer dues 7 to 41 s, commission range 7.8 s, partner statement 1.8 to 2.2 s, windows_bounds 1.3 to 1.5 s). All fixed. Final, quiet machine: collection sheet 42 ms (a day 5 years back 640 ms), buyer dues 107 ms, partner statement 48 to 75 ms, windows_bounds under 10 ms, commission range 240 ms; next pages under 50 ms. Also fixed: windows_bounds for partner sources behind the code; lent_dues hides the partner of a link while locked; commission_day refuses a future date; khata link with a stray partner refused; Past Sales focus after an edit (fuzz 41 of 41); per-day labels on Past Sales day grids.

Plan 77 first report: FAIL on speed (kharch 47 s, grouped views up to 164 s, plain views 3 to 13 s). Fixed: view M driven from the head-05 accounts (kharch 10 years 185 ms quiet); analytics pages counted per bucket and read only the page's buckets; a safe in-memory result cache for next pages (key: filter, max ids of the append-only tables, data_version, a restore generation and a per-connection token; money agent B1 fixed); migration 0024 adds three covering indexes. Umar decided (15-11-2026, NFR v1.19): the first page of analytics views over more than one year may take up to 4 s at the NFR-SCL-1 volume. Final, idle machine: every first page under 4 s except view L by month over 10 years, 4.2 s (MINOR, follow-up); every next page 75 ms or less.

Faults, inputs and security: mixed save and undo with injected write failures leave nothing behind and burn no number; hostile inputs refused with codes; every new command refused signed out and locked; partner figures behind the code including drill lines, print and Excel. Money oracles on the 10-year file all match plain SQL.

## Money reviews (money-calculations agent)

- Plan 76 final: PASS WITH CHANGES (all taken). Second review of 10a, 10b and the R5/slip changes: PASS WITH CHANGES (all taken). Speed changes: one BLOCKER (index name), fixed.
- Plan 77: no blockers on the commission read, views L and M; result cache B1 BLOCKER fixed (per-connection token).

## Decisions recorded

Plan 76 Section 14 and plan 77 Section 4 hold every decision of this run. Umar's decisions: A10 to A19 (plan 76), the commission filter and period, the simple kharch page (FR-9.5), and the 4 s budget (NFR v1.19).

## Follow-ups (MINOR, not blocking)

1. View L grouped by month over 10 years: first page 4.2 s against 4 s at idle (NFR-SCL-1 volume).
2. NFR-SCL-4: the 10-year test file is 2.07 GB (1.93 GiB) with 1M synthetic audit rows and the new 0024 indexes; watch the limit.
3. A mixed save's carried-lot print list lives in memory for the session; after a restart the whole-day print covers it.
4. New Urdu texts are marked "review: pending" for the Urdu reviewer (NFR-URD-4).
5. Windows checks LOCAL-ONLY: Urdu shaping of the fruit picture and chart SVG in prints, printing on WebView2.
