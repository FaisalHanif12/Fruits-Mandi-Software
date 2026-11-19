# D5 late track (Run B): everything of D5 that is not in the early track

Version 1.0, 23-10-2026. Lead: tech-lead. Branch `feature/d5-early` (Run B). Umar's order of 23-10-2026: build every part of v1.6 and v1.7 that is still missing (D4R in Run A, the late parts of D5 here in Run B), and only then run ONE verification gate (reviewer and robustness) over the whole of D3, D4, D4R and D5. Brief: `docs/deliverables/D5-reports-backup-golive.md` (plans 19 to 23 and acceptance D5-A1 to D5-A26) and `docs/deliverables/D5-parallel.md`. Run B file rules of `docs/plans/d5-early.md` Section 1 still apply (no migration; no edits in core, domain or `services/src/{bikri,vouchers,ledger,partners,ventures,trips,purchases}` or `day.rs`; shared files append-only; PAUSE file; nice; disk 12 GB).

D5-Q1 is answered: yes (Umar, 23-10-2026). Money goes to Excel only through `rupees_as_f64` (Code Standards M13, money agent 1.6).

## 1. Order and dependencies

Run A builds D4R in this order on `feature/refresh-v16`: 17b (one cash account, lists page by page), 17c (gardens), 17d (fruit taken), 17e (the tag, firm contacts, clear prints). Run B merges that branch after each D4R plan commit, runs `pnpm gen:types`, `golden_day` and its own tests, and adds one line to `coordination.md`.

| Round | Builds | Needs from Run A | Backend part (`backend-d5`) | Frontend part (`frontend-d5`) |
|---|---|---|---|---|
| 5 (running) | B5: settings, audit viewer, books check, updates, support info. F4: analytics screens A to E and H | nothing | B5 | F4 |
| 6 | F5: settings, audit viewer, update prompt, support button | nothing | (idle or L1) | F5 |
| 7 (L1) | Month close and its reversal (FR-17.6, FR-21.2) with the statement note line; reports R12, R29, R31, R32 | plan 17 (committed) | month close service in a NEW file (not in Run A's files), R12, R29, R31, R32 print data and fixtures; money agent before and after | month close screen, report screens for R12, R29, R31, R32 |
| 8 (L2) | Reports R33 (trip), R34 (day summary, trips, cash, receipts; garden and fruit parts wait), R15, R23, R25 (no mode column), R10 (without the fruit column), R1 and R5 as report entries | plan 16b, 17 | the reports, fixtures, tests | the screens |
| 9 (L3) | Plan 19: the dashboard (all tiles except the garden tile), partner strip, today's list, oldest dues, mini charts, attention list and count, health check result, remembered last screen, first-run guide, empty states | plan 17 (overdue lists) | tile and list reads, attention, last screen | the dashboard, bell, آج کے کام, guide, empty states |
| 10 (L4) | Excel export for every report and analytics view and the readable backup pack (FR-20.5): `rupees_as_f64`, Fluent headings, right-to-left Urdu sheets, real dates, totals row, filters above the table, background run with progress | none | export code, `rupees_as_f64` with the maximum-amount test, pack spreadsheets, fixtures | export buttons, progress |
| 11 (L5) | After D4R is merged: R35 in the client's two layouts, R36 (gardens), R37 (fruit), R10 fruit column, R34 garden and fruit parts, analytics views F, G, I, J, K and the garden, trip, goods and partner filters, the garden tile, the FR-19.13 header and footer with the tag on every report and analytics print, report print and PDF enabled through the shared print job of 17e, party names with the tag everywhere | D4R 17c, 17d, 17e committed and merged | the reports, views, print data, snapshots | the screens, print preview and the print action |
| 12 (L6) | Go-live parts that run on the Mac: version 1.0.0 in `tauri.conf.json`, `docs/release/whats-new.md` (lead), `scripts/perf` data generator and every NFR Section 2 benchmark (robustness at the gate), the Windows suite specs written, the word list gates (D5-A22) with Umar's review rows | all | as the lead assigns | as the lead assigns |

Windows-only items stay LOCAL-ONLY (installer, WebView2 shaping and printing, the Windows suite run, a real cloud upload for D5-Q2, the signing certificate for D5-Q3). They are listed in status.md under "Waiting for Windows".

## 2. Worked numbers and acceptance

Every number comes from the PRD (27.2 and 4.9) and `worked_day.py`. Acceptance subset of this track: D5-A1 to D5-A10, D5-A21 to D5-A26 and the Excel parts of D5-A5; D5-A11 to D5-A14 are done in the early track; D5-A15 to D5-A20 belong to the final gate and the Windows phase.

## 3. Rules for the builders in every round

Same as `docs/plans/d5-early.md`: money only through `core::money`; reports and analytics read posted records and write nothing; money-calculations agent before and for review of any money code; fixtures for every command; shape tests; four states; paged lists with backend totals; keyboard-only and language-switch tests; screenshots at three sizes on a private port; Windows specs numbered 40 and up. The builders run their own targeted checks and, at the end of a round, `cargo fmt`, `clippy`, the touched crates' tests, `golden_day` and the app contract tests, and for the frontend `pnpm lint`, `pnpm typecheck`, `pnpm i18n:check`. The whole-suite run, `pnpm perf` and the reviewer and robustness gate happen once, at the end of everything (Umar, 23-10-2026).

## 4. Contracts as built

(The lead appends each round's contract here, as in `d5-early.md` Section 7.)

### Round 7 (L1) contract as built: month close and reports R12, R29, R31, R32 (lead, 25-10-2026, from the `backend-d5` hand-off)

**Month close** (FR-17.6, FR-21.2): commands `month_list`, `month_close_preview`, `month_close_save`, `month_close_reverse` (types `MonthInput`, `MonthReverseInput`, `MonthLine` and its kind, `MonthPreview`, `MonthClosed`, `MonthReversed`, `MonthState`, `MonthRow`, `MonthList`). Errors: `ERR_FORMAT_INVALID`, `ERR_NOT_FOUND`, `ERR_REASON_REQUIRED` on `month` or `reason`; `ERR_DATE_IN_FUTURE`, `ERR_DAY_NOT_CLOSED` (value `closed_up_to`), `ERR_MONTH_CLOSED`, `ERR_NOTHING_TO_CLOSE`, `ERR_JOURNAL_UNBALANCED`. The close entry is dated TODAY (closed days take no postings); the reversal is a mirror dated today; neither writes share rows or changes any partner's کل رقم (C22). One entry from one plan shared by preview and close; the profit of the lines must equal the partner sums or the close is refused and nothing is written. Fixtures: `services/tests/fixtures/month_*.json`.

**Reports:** new ids R12 (cash book), R29 (expenses), R31 (partner statement, built by the app from `services::partners::statement`; the engine alone answers a Data error for R31), R32 (money lent, a copy of `services::lent::report_rows`; `open_amounts` and `days_past_return_by` are copied into `reports/workings.rs` with 10,000-case parity tests). New filter kind `partner` and `ReportFilters.partner_id` (required for R31: in the catalogue `party_required: true` means the partner). New cell and column kind `message` (Fluent id and args). R31 print and export read the statement in chunks of 10,000 joined only when every chunk agrees; otherwise repeated up to 3 times. Fixtures: `reports_run_r12_day`, `r29_day`, `r29_alif`, `r31_bay`, `r31_partner_required`, `r32_e2e_30b`, `reports_print_data_r31_bay_en`; every older `reports_run_*` fixture now carries `"partner": null`.

**Run A files edited by Run B (lead approved):** `services/src/ledger.rs` one line, `post_ordered` skips the active-account check for `PostingType::MonthClose` (found by the money review: an inactive expense head must not block a close); `db/src/cashbook.rs` reason subselect wrapped in COALESCE; registration lines in `lib.rs`, `build.rs`, capabilities and command tables.

**Follow-ups from the review:** (1) every reader by date must skip `source_type = 'month_close'` (the close entry is dated today and debits income accounts): the partner statement, overview, R28, R30, R29 and the pack lists already do; the dashboard income tiles (plan 19), analytics views F and G (round 11), R34 and the day close summary of Run A's `day.rs` must be checked when they are built or touched. (2) A partner whose month sum is 0 gets no line and no note line. (3) R12 shows the account name, not the person, for a پختہ line. (4) Garden and fruit statement lines have no Fluent sentence until D4R; R31 uses `partners-statement-detail-other` with the reason. (5) A random-sequence test comparing R32 with `lent::report` is a suggestion for the final gate.
