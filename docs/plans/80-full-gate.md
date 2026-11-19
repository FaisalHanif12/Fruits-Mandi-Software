# Plan 80: Final verification gate for the whole app

| | |
|---|---|
| Status | Started 19-11-2026 by Umar's request. No new features. |
| Branch | `feature/d8-demo-feedback`, from commit `e65a18f7` (plans up to 79 built) |
| Docs | PRD v1.44, NFR v1.20, Tech Stack v1.11, Code Standards v1.25, Architecture v1.28, money agent v1.13. They live in `.claude/rules/`, not `docs/`. |

## 1. Goal

Review and stress test the whole app, fix what is found, and prove it. Nothing new is built.

## 2. Lanes

Review lanes (`code-reviewer` copies, read-only):

| Lane | Scope |
|---|---|
| R1 | Money and books: `core::money`, `domain`, journal, partner split, matching, rounding, golden tests. Calls the `money-calculations` agent. |
| R2 | بکری, later sales, carried lots, edit, cancel, undo, reopen and fix, vouchers (وصولی, ادائیگی, پیشگی), paid on the spot. |
| R3 | Masters, accounts and codes, opening balances, rules, چکتا, dead amount, search, translit, names in two scripts. |
| R4 | Gardens, trips, fruit, capital, الگ کھاتہ, ادھار رقم and links, partner totals and the partner statement, the partner code. |
| R5 | Dashboard and every tile page, reports R1 to R38, analytics, prints, PDF, Excel, paging and backend totals. |
| R6 | The assistant: Code Standards Q1 to Q11, privacy (FR-7.83), the prompt files equal to D6, refusals, limits. |
| R7 | Backup and restore, security (NFR-SEC), the frontend shared parts: keyboard, grid, i18n, RTL and LTR, the four states. |

Robustness lanes (`robustness-tester` copies):

| Lane | Scope |
|---|---|
| T1 | Property tests: splits add up, books balance, partner totals equal equity, reversal plus entry is zero, for random sequences of every posting type. |
| T2 | Volume and speed: the NFR-SCL-1 dataset (10 years) and every NFR-SPD target, including every dashboard page and report. |
| T3 | Failure: crash during save, drafts after a kill, disk full, backup and restore round trip, migrations on old data with chain hashes unchanged. |
| T4 | Bad input: fuzz every command, Unicode and Urdu edge cases, a بکری with 1,000 lines, a language switch in the middle of an entry, every edge case of PRD Section 23. |
| T5 | Assistant attacks: injection text in names (E92), typed figures, wrong tool calls, locked partner figures, the daily limit, offline. |

At most 6 lanes run at once.

## 3. Rules for every lane

1. Work only on your lane. Others cover the rest.
2. The requirement docs are the truth. If code and a doc disagree, or two docs disagree, report it as a QUESTION for Umar. Do not choose.
3. A failing golden test means the code is wrong. Never change a golden expected number.
4. Mac phase: list every check that needs real Windows as LOCAL-ONLY (WebView2 Urdu shaping, Windows print path, installer, `pnpm e2e`, reference hardware speed).
5. Report findings in your agent file's format. Each finding has: ID (lane + number, for example `R2-3`), severity (BLOCKER, MAJOR, MINOR), file and line, the failing case (inputs and wrong result), the command that shows it, and the requirement ID.
6. Do not repeat findings already accepted by Umar in earlier reviews (`docs/reviews/*.md`, "accepted" or "for Umar" items) unless they are now worse.

### Build and test rules (8 cores, shared machine)

- Reviewers do not edit any file. Run only targeted tests (`cargo test -p <crate> --test <file>`, `pnpm --filter frontend exec vitest run <file>`). Use the shared `backend/target`. The lead runs the full suites at the end.
- Each robustness lane uses its own build folder: `export CARGO_TARGET_DIR="$PWD/backend/target-gate-<lane>" CARGO_INCREMENTAL=0` (lane in lower case, for example `target-gate-t1`). Delete nothing outside it.
- Robustness lanes write only in folders they own, and only new files named for the lane: `backend/crates/<crate>/tests/robustness_gate_<lane>*.rs`, `frontend/tests/robustness/gate-<lane>-*`, `scripts/perf/gate-<lane>-*`, `backend/crates/<crate>/benches/gate_<lane>_*.rs`. A bench needs a `[[bench]]` entry in a `Cargo.toml` that `backend` owns: ask the lead in your report instead of editing it. Every test you add must pass on correct code; a test that shows a bug is marked `#[ignore = "gate finding <ID>"]` (or `it.skip`) and named in the finding, so the suite stays green until the fix.
- No test may call the network. The assistant uses recorded transcripts.

## 4. After the lanes

1. The lead removes duplicates and ranks the findings.
2. `backend` and `frontend` fix them. Every fix has a test that fails before and passes after.
3. The affected lanes run again until clean.
4. Final checks: `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `pnpm perf`, `cargo test --workspace`, `cargo deny check`.
5. Report: `docs/reviews/2026-11-19-full-gate.md`.

## 5. Findings and fixes

(Filled by the lead as lanes report.)

### R1 (PASS WITH MINOR, 19-11-2026, with the money-calculations agent)

Golden tests all pass; the only changed golden numbers are the PRD v1.43 B4 set approved by Umar on 17-11-2026.

| ID | Sev | Owner | Finding |
|---|---|---|---|
| R1-1 | MINOR | backend | `split_by_qty` doc still names the retired C17 (`core/src/money/split.rs:1,36`; G5) |
| R1-2 | MINOR | backend | "% of all" maths copied in `reports/src/analytics/calc.rs:38` instead of `core::money::hundredths` (G6) |
| R1-3 | MINOR | backend | An advance made before any partner share exists gets no partner parts, so the FR-17.9 check misses (`core/src/advances.rs:134`). Waits for Q-R1-3 |
| R1-4 | MINOR | backend | `advance_net` replays every advance on each partner total call (`services/src/partners/money.rs:67`; B22, NFR-SPD-24). Sent to T2 to time |
| R1-5 | MINOR | lead (doc) | Code Standards 9.2 names `reports/tests/ledger_items.rs`, which never existed; E2E-70 has no row |
| R1-6 | MINOR | backend | Partner statement notes swallow a split error silently (`services/src/partners/statement.rs:391,415`; Principle 4) |

### R5 (FAIL, 19-11-2026: 1 MAJOR, 3 MINOR)

| ID | Sev | Owner | Finding |
|---|---|---|---|
| R5-1 | MAJOR | backend (+frontend) | The آج کی بکری tile counts today's postings (`reports/src/dashboard/tiles.rs:53`) but its page پچھلی بکری counts بکری by their own date with lot gross over all postings (`db/src/bikri/saved.rs:595,685`). A later sale or a carried lot today makes them differ (E2E-16b: tile 2,800, page 0). FR-9.1, FR-12.46, FR-6.1. Direction waits for Q-R5-5 |
| R5-3 | MINOR | frontend | On the کل اگراہی page a dead buyer or seller opens R13, the others open پارٹی کھاتے (`TotalDuesSections.tsx:179`; FR-19.8, NFR-USE-10) |
| R5-4 | MINOR | backend + frontend | R13 and R14 use ابتدائی بقایا both for the period-start balance and for the opening row (`reports/src/reports/accounts/ledger.rs:153`; FR-19.16, glossary) |
| R5-5 | MINOR | frontend | Retired word سابقہ رقم still in `ur/reports.ftl:1553,1750` (glossary v1.43, NFR-URD-7) |

### R3 (PASS WITH MINOR, 19-11-2026)

Translit numbers did not drop (top-1 92.0%, top-3 95.4%, English 72.2%; the NFR-URD-24 and 25 gates are go-live gates, still open). The search speed test failed once under shared-machine load and passed alone (flaky wall-clock bound, LOCAL-ONLY on reference hardware).

| ID | Sev | Owner | Finding |
|---|---|---|---|
| R3-1 | MINOR | backend + frontend | A new چکتا buyer skips the backend spelling check, and a confirmed rules-only word is never learned (`services/src/chukta.rs:43`, `QuickAddChukta.tsx:72`; FR-7.66, V0) |
| R3-2 | MINOR | backend | Renaming a city leaves accounts with that city stale in the search index until restart (`services/src/masters/mod.rs:165,341`; Tech Stack 4.7, B27) |
| R3-3 | MINOR | frontend | A short code in an item, packing or staff box is dropped on leaving the box; only Enter takes it (`Picker.tsx`, `PickerBase.tsx:371`; Code Standards 4.2) |
| R3-x | check | R4 | `db/src/ventures.rs:547` runs `DELETE FROM venture_partners`: check it runs only before the first entry (D3, FR-12.29). Sent to R4 |

### R6 (FAIL, 19-11-2026: 2 MAJOR, 2 MINOR; all assistant tests pass)

| ID | Sev | Owner | Finding |
|---|---|---|---|
| R6-1 | MAJOR | backend | R10 sends the typed other-shop name (`shop_name`) to the model: `is_left_out` matches `shop` but not `shop_name` (`assistant/src/tools/cells.rs:24-42`; FR-7.83, Q7, NFR-AI-4). No test has a typed shop name |
| R6-2 | MAJOR | lead/backend | `app_state{what:"audit"}` sends masked audit rows; `app_state` is not in FR-7.87 and FR-7.83 says the audit log never goes (`tools/state_audit.rs`). Waits for Q-R6-1 |
| R6-3 | MINOR | backend + frontend | A records difference adds no attention item (`run.rs:1418`; FR-7.76 point 2). Waits for Q-R6-3 |
| R6-4 | MINOR | lead (docs) | Time limits in code (12 s call, 25 s question, plan 55) differ from NFR-AI-3, Code Standards Q8 and Q9, Tech Stack 4.8. Waits for Q-R6-2 |
| R6-x | MAJOR (from Q-R5-3) | lead/backend | With money lent open, the model path answers "kul ugrahi" with the buyers_dues tile (prompt line 41), a checked but wrong figure (FR-7.76, NFR-AI-1). The tiles tool still sends `last_backup` and lacks آج کی نقدی and the others' dues tile |

### R4 (FAIL, 19-11-2026: 1 MAJOR, 5 MINOR)

| ID | Sev | Owner | Finding |
|---|---|---|---|
| R4-1 | MAJOR | backend + frontend | `fruit_list` gives every partner's fruit taken with totals while the partner section is locked (`app/src/commands/fruit.rs:106`, `services/src/fruit/read.rs:136`; FR-17.11, NFR-SEC-14, S8). Only its print (R37) is refused |
| R4-2 | MINOR | backend | While locked: `vouchers_list` with personal_expense, personal_return, capital_in, capital_out lists a partner's takes and capital (`commands/vouchers.rs:82`); `assist_attention_overdue` gives overdue partner takes (`services/src/lent.rs:804`); `lent_list` and `khata_statement` give the linked partner id (`lent.rs:488`, `khata.rs:87`). NFR-SEC-14 |
| R4-3 | MINOR | backend | After a relief refund, raising the rent back or cancelling an installment leaves refunds above what the owner owed, silently (`gardens/header_edit.rs:228`, `entry.rs:698`; FR-12.51) |
| R4-4 | MINOR | backend + frontend | Changing an opening that is in روکڑ is refused when cash goes below 0 instead of asking (`accounts/openings.rs:271`; FR-6.7, 17.1 v1.43) |
| R4-5 | MINOR | backend | `partner_code.rs:23` doc says `tries_left` 4; the limit is 10 since v1.35 (G5) |
| R4-6 | MINOR | backend or lead | `venture_partners` rows are DELETEd when shares are set again (`db/src/ventures.rs:547`). Safe (only before the first entry, service checks and a trigger, audited) but against D3 as written. Fix: soft rows, or a recorded D3 exception |

### T1 (PASS, 19-11-2026, no findings)

New test `backend/crates/services/tests/robustness_gate_t1/` (45 step kinds through the real services, invariants A to L after every step: balance, whole rupees, reversal = 0, share rows = income less expenses, FR-17.9 with FR-17.13, C21 to C28, matching re-run changes nothing, no half-save, audit on every save, chains verify). Fixed-seed run: 7 passed. Hunt with 150 random runs (about 3,900 steps): no invariant broke. Not covered: day close, reopen-and-fix and month close (closed-day interplay).

### T3 (PASS WITH MINOR, 19-11-2026)

New tests (all green, 3 ignored by finding name): `services/tests/robustness_gate_t3_{atomic,kill,disk,race,tamper}.rs`, `backup/tests/robustness_gate_t3_{backup,old_copy}.rs`, `db/tests/robustness_gate_t3_migrations.rs`. Proven: a failure at every write step of about 50 save kinds leaves every row unchanged; 60 SIGKILLs during saves leave whole books; disk full gives ERR_DISK_FULL with nothing changed; races give one result; backup and restore of a rich book from local, USB and cloud give every row equal, 69 damaged copies refused; an old schema-4 copy and file upgrade to 25 with every chain hash unchanged.

| ID | Sev | Owner | Finding |
|---|---|---|---|
| T3-1 | MINOR | backend | Same as R2-3: `day::fix_open` reopens in one transaction and records the fix in a second (`day.rs:774,794`). Test `fix_open_is_not_one_save` (ignored) |
| T3-2 | MINOR | backend | `day::fix_close` closes the days, then clears the fix record and writes its audit in another transaction (`day.rs:888,913`). Test `fix_close_is_not_one_save` (ignored) |

### R7 (PASS WITH MINOR, 19-11-2026)

All 345 commands check sign-in and lock (static scan plus `robustness_security`); capabilities, CSP, Argon2id, lockout, secrets, logs, network hosts, audit, backup schedule, retention and safety copy hold; `pnpm i18n:check` and frontend lint pass. The readable pack's روزنامچہ PDF is still pending (Windows phase).

| ID | Sev | Owner | Finding |
|---|---|---|---|
| R7-1 | MINOR | frontend | The five plan 79 tile pages (`/dues/sellers`, `/dues/total`, `/dues/lent`, `/dues/advances`, `/cash-sales`) have no Back fallback parent (`shared/navigation/back-parents.ts:16`; FR-7.92) |
| R7-2 | MINOR | frontend | پچھلی بکری handles its own Page Up and Page Down (`PastDays.tsx:92-115`, `PastWeekNav.tsx:52-59`); they work only with focus on a day line (F27, F29, FR-12.46) |
| R7-3 | MINOR | robustness | 1 s wall-clock bound at `backup/tests/robustness_restore_new_computer.rs:1449` fails under load |

Flaky under load, pass alone: `services/tests/search.rs:455` (R3), `cashbook-lent.test.tsx:317` (R4), `backspace-delete` timeouts (R7), R7-3.

### T4 (FAIL, 19-11-2026: 1 MAJOR, 2 MINOR)

New tests: `app/tests/robustness_gate_t4_ipc.rs` (19,115 hostile calls on 342 of 345 commands: no ERR_INTERNAL apart from T4-1, no refused call committed, every value error has a field, books balance after), `services/tests/robustness_gate_t4_{big,names,numbers,general}.rs`, `frontend/tests/robustness/gate-t4-switch-{bikri,voucher,expense}.test.tsx`. A بکری of 50 lots and 1,000 lines: figures exact, preview = save (debug preview 15.8 ms, save 383 ms). E1 to E103 coverage table in the T4 report; E10 had no test.

| ID | Sev | Owner | Finding |
|---|---|---|---|
| T4-1 | MAJOR | backend-c | Trip cancel and edit commands accept a garden's entry id: `trips_cost_cancel` cancels a cost of a CLOSED garden (bypasses ERR_GARDEN_CLOSED); `trips_other_sale_cancel` on a garden sale gives ERR_INTERNAL (`trips/cost.rs:356`, `sale.rs:446`, `cost_edit.rs:48,93`, `sale_edit.rs:55,93`; V0, FR-12.38). Test `an_entry_of_a_garden_is_not_found_by_the_trip_commands_and_the_other_way_round` (ignored) |
| T4-2 | MINOR | backend-c + frontend-2 | E10 combined balance of linked seller and buyer accounts is never shown; the link is stored only |
| T4-3 | MINOR | backend-c | Seeded system account 03070001 has ':' in both names (Code Standards 4.2) |

Lead decisions on its questions: Q-T4-1 the plain IPC refusal of a wrongly typed payload is accepted (only a hostile caller can send it; nothing is written); Q-T4-2 build E10 as stated (the party card and the ledger heading show the combined balance, read-only, from the backend); Q-T4-3 notes strip zero-width and bidi marks and invisible-only text counts as empty; Q-T4-4 rename 03070001 in migration 0026 with names that pass 4.2.

### T5 (FAIL, 19-11-2026: 2 new MAJOR, 7 MINOR or questions)

New tests: `assistant/tests/robustness_gate_t5.rs` and `tests/robustness_gate_t5/` (75 tests, no network). At e65a18f7: 59 pass, 12 ignored by finding; the R6-1, R6-x and T5-3 tests fail at baseline and pass on the fixed tree. The privacy sweep puts a marker in 49 free-text columns of 31 tables over about 1,000 tool results per run: nothing leaks except `shop_name` (R6-1, fixed). Held: injection names, braces, numeric characters, 150+ number words, about 110 wrong tool calls (never ERR_INTERNAL), limits, the key never shown, offline.

| ID | Sev | Owner | Finding |
|---|---|---|---|
| T5-1 | MAJOR | backend-a | Invisible characters (U+00AD, U+034F, U+2061-2064, U+180E, U+3164, variation selectors, tags) inside a number word, month or never-use word, and look-alike Cyrillic or Greek letters in an English answer, pass `check.rs` (:218, :224, :758). FR-7.76, NFR-AI-1, Q4. Tests `typed::t5_1_*` (ignored) |
| T5-2 | MAJOR | backend-a | "aaj ka commission": a model that reads another period and cites its total is shown as today's, checked (`scope_guard.rs:17,81`). The instant path covers the plain wording. Test `tools::t5_2_*` (ignored) |
| T5-3 | MAJOR | backend-a | `app_state` audit showed partner money while locked. Fixed by the R6-2 removal; test passes now |
| T5-4 | MINOR | backend-a | ✅ and other check-mark shapes are not refused (`check.rs:786`) |
| T5-5 | MINOR | backend-a | The connection test ignores the daily limit (`run.rs:3134`; FR-7.81) |
| T5-6 | MINOR | backend-a | A failed continuation cannot be tried again (`run.rs:3062`) |
| T5-7 | MINOR | backend-a | A conversation id starting with NUL fails only at the log write, after the paid call (`run.rs:2785`; V0) |
| T5-8 | MINOR | backend-a | A party named باقی spoils answers about other parties (`check.rs:148,188`) |
| T5-9 | MINOR | lead (D6), then backend-a | words.tsv gaps: nil, couple, pair, thrice, trillion, adha, pauna, sawa, dedh, dhai, darjan, جوڑا, دہائی, اکتبر, اپرئیل |
| T5-10 | question | Umar | `find` matches phones and father or firm names, so a model could probe them (no tool result holds them) |
| T5-11 | question | Umar | R11, R12, R15, R17, R18 show partner capital and drawings as lines while the partner section is locked |

### T2 (interim, 19-11-2026: FAIL on speed)

Data: 41,000 بکری, 1,200,000 sale lines, 5.57M journal lines (1.6x NFR-SCL-1), busiest day 500 بکری and 5,250 lines; books balance. Release build, quiet Mac, p95, at e65a18f7. Mac, not the reference hardware; a Mac miss is still a finding.

| ID | Target | Measured | Owner |
|---|---|---|---|
| T2-1 | dashboard_get, NFR-SPD-2 1 s | 2.5 s | backend-f |
| T2-2 | assist_attention, NFR-SPD-22 1 s | 1.85 s | backend-f |
| T2-3 | R35 busy day, NFR-SPD-10 and 24 | first 16-28 s, next 26-45 s | backend-f |
| T2-4 | R34 day summary, 1 s | 1.8 s busy, 3.3 s ordinary | backend-f |
| T2-5 | R20, R21, R22 as of busy day | first 1.1-1.8 s, next 1.7-3.0 s | backend-f |
| T2-6 | R2, R4 10 years; R28; R3, R7, R16, R18 next page | 2.0-2.4 s; 3.2 s; 0.53-0.8 s (R7 scans analytics_facts) | backend-f |
| T2-7 | party statement rows, busiest buyer | first about 5 s, next about 4 s | backend-f |
| T2-8 | view H over 10 years (exception 4 s) | 6.5 s | backend-f |
| T2-9 | view D over 5 years by month | refused by MAX_PERIOD_COLUMNS = 36 (not in the PRD) | backend-f |
| T2-10 | start-up quick_check 16-23 s; a save is refused after CHECK_WAIT 30 s (NFR-REL-7 says held) | | backend-f |
| T2-11 | benches `bikri` and `bikri_save` use account codes gone since plan 32, so `pnpm perf` cannot see NFR-SPD-7, 8 and SCL-3 | | t2-speed |
| T2-12 | NFR-SCL-4: 2.31 GB file, 537 MB copy at 1.6x; about 1.9 GB and 440 MB at exactly 3.5M lines: inside, close | | none |

Fine at volume: commission page, cash sales pages, lent open, buyer and seller dues pages, advances pages, R12 to R15, R6, party card, partner overview 127 ms, partner month statement 165-220 ms, R31 140 ms (R1-4 confirmed fine).

### R2 (PASS WITH MINOR, 19-11-2026)

| ID | Sev | Owner | Finding |
|---|---|---|---|
| R2-1 | MINOR | frontend | Undo of a mixed save never opens the cash confirm (`undo.tsx:119`), so an undo that takes cash below 0 is always refused (FR-6.7, FR-7.47) |
| R2-2 | MINOR | backend | `bikri_undo` and `mixed::undo` cancel any بکری with no version check and no "just saved" check (`cancel.rs:354`; V0, FR-7.47) |
| R2-3 | MINOR | backend | Reopen-and-fix writes in two transactions (`day.rs:774`, `day.rs:794`; B3, FR-21.4) |
| R2-4 | MINOR | frontend | Seller tick shows on while a new lot added in an edit sends `adjust_advance false` (`draft.ts:188`; FR-12.23). Waits for question Q-R2-3 |
| R2-5 | MINOR | backend + frontend | Trip and garden goods cards stop at 100 rows (`stock.rs:564`; FR-12.50) |
| R2-6 | MINOR | backend | Retired rate hint still queried per item pair on every preview (`shared.rs:481`; FR-7.53 retired, B22) |

### Fix log

| Finding | Owner | State | Test that failed before |
|---|---|---|---|
| R7-1 | frontend | fixed | back-trail.test.ts, back-pages.test.tsx (25 failed before) |
| R7-2 | frontend | fixed | past-sales.test.tsx, 3 new (4 failed before); past-sales-fuzz 41 pass |
| R5-3 | frontend | fixed | total-dues-page.test.tsx (4 failed before) |
| R5-5 | frontend | fixed | retired-words.test.ts (new, 3 failed before) |
| R3-3 | frontend | fixed | picker.test.tsx (12 failed before) |
| R2-1 | frontend | fixed | bikri-carried-lots.test.tsx, money-format.test.ts (4 failed before) |
| R3-1 (screen) | frontend | fixed | quick-add-chukta.test.tsx (failed before in both languages) |
| T4-2 (screen) | frontend | fixed | linked-accounts.test.tsx (card tests failed before) |
| R5-4 (words) | frontend | fixed | report-lines.test.tsx (6 failed before: id matched no message) |
| Q-R2-2 (screen) | frontend-2 | fixed | day.test.tsx (wrong sign refused at the field, nothing reopens) |
| R2-2 (screen) | frontend-2 | fixed | undo refusal ERR_VERSION_CHANGED has its own message |
| R6-3 (screen) | frontend-2 | fixed | dashboard-words, dashboard-bell tests |
| Q-R7-1 (fixtures) | frontend-2 | fixed | settings-updates-support test (46 pass) |
| R4-1, Q-R4-2, R4-4, Q-T3-3 (screens) | frontend | fixed: typecheck, full lint, i18n:check green; 26 files 888 tests pass | fruit-partners-locked, gardens-close-owes, account-cash-confirm, openings, settings-day-check tests |
| R2-4 | frontend-2 | fixed | bikri-advance-tick.test.ts (failed before) |
| Q-R5-2 | frontend-2 | fixed | adhari-sheet-page.test.tsx (new, 24 tests). Open: no Excel kind for the new sheet |
| R5-1 | backend-b | fixed | mixed_save e2e_16b and mixed save tests, paging_plans plan test (failed before); money agent PASS; 2 plan-72 tests rewritten to Umar's new rule |
| R2-3, T3-1, T3-2 | backend-b | fixed | robustness_gate_t3_atomic fix_open/fix_close (failed before, now un-ignored and passing) |
| Q-R2-2 | backend-b | fixed | owner_sign::the_plain_reopen_asks_the_owners_sign_too |
| R2-2 | backend-b | fixed | mixed_save::an_undo_is_refused_once_the_bikri_has_changed_since_its_save |
| R5-4 | backend-b | fixed | reports_accounts r14 test (failed before); 7 fixtures follow the new label id |
| T4-1 | backend-d | fixed (isolated copy) | services/tests/venture_entry_kinds.rs (2 of 3 failed before) |
| R3-1 | backend-d | fixed (isolated copy) | services/tests/chukta.rs new test (failed before) |
| R3-2 | backend-d | fixed (isolated copy) | services/tests/search.rs new test (failed before) |
| Q-T4-3 | backend-d | fixed (isolated copy) | core validate_fields + services note_invisible_text.rs (3 failed before) |
| T4-2 | backend-d | built (isolated copy) | services/tests/linked_balance.rs (6 pass; new feature). Money review pending |

## 6. Questions for Umar

### 6.1 Decisions (19-11-2026)

Umar answered Q-R5-5 ("only posted sale") and Q-R5-3 ("yes, the assistant should be corrected"), and said "the rest, choose what is best". The lead chose the rest as below. Each is recorded in the docs it touches.

| Q | Decision | Work |
|---|---|---|
| Q-R5-5 | **Umar:** آج کی بکری counts the sales posted today, including later sales and carried lots. The tile is right; پچھلی بکری's today section and its day headings follow the postings of each day. | backend-b (R5-1) |
| Q-R5-3 | **Umar:** correct the assistant. D6 prompt v1.12: kul ugrahi = total_dues; tiles list drops last_backup, adds today_cash_sales and others_dues. The live NFR-AI-2 run is for Umar with his key. | backend-a |
| Q-R6-1 | Remove the audit read from `app_state` (FR-7.83 stands: the audit log never goes). PRD FR-7.87 lists `app_state` (rules, attention, days) and `app_help`. D6 v1.12. | backend-a, lead (PRD) |
| Q-R6-2 | The code's limits (12 s a call with one retry, 25 s a question, plan 55) are right; the docs follow. | lead (NFR-AI-3, CS Q8, Q9, Tech Stack 4.8) |
| Q-R6-3 | Build it: an attention item for a question whose two places differed in the last 7 days. | backend-a, frontend |
| Q-R1-1 | Keep: a balance estimate starts next month, because today's balance already holds the current month. PRD FR-7.86 says so. | lead (PRD) |
| Q-R1-2 | Reinstated (Umar, 20-11: nothing left open for time): the note shows the money that really reached him, worked out by replaying his credits and his shop and pocket parts (no simple formula that can drift). Money agent before and after. | backend-c |
| Q-R1-3 | Withdrawn (19-11): PRD 4.3 and golden E2E-37 save رحیم's پیشگی before any share exists; the FR-17.13 fallback (the first shares ever set) covers it. R1-3 stays a recorded edge case. | none |
| Q-R1-5 | Yes, the lead corrects Code Standards 9.2. | lead |
| Q-R2-1 | Follow the PRD: a saved lot can be removed in an edit of its first posting, unless a later sale, write-off or fruit taken rests on it (as A74) or it is a carried lot; the screen says why when locked. | backend-b (money agent), frontend |
| Q-R2-2 | The plain reopen (FR-21.2) also asks the Owner's sign. | backend-b, frontend |
| Q-R2-3 | One tick covers every lot of that seller in the بکری, also a lot added in an edit (R2-4). | frontend |
| Q-R3-1 | "Saved last" means the last saved time (edited counts), then the bigger id. | backend-c |
| Q-R4-1 | Yes: goods arrived counts as an entry; shares lock after it (service check and a new trigger, migration 0026). | backend-c |
| Q-R4-2 | Warn, do not refuse: closing a garden while the owner owes asks a confirm box with the amount. | backend-c, frontend |
| Q-R4-3 | Keep: guest fruit is split over the partners with a share above 0 on the date; PRD C27 wording follows. | lead (PRD) |
| Q-R4-4 | Keep: partner figures are open until the code is first set (the Owner has not chosen to protect them yet); PRD FR-17.11 says so. | lead (PRD) |
| Q-R5-1 | R13 and R14 stay the full line ledger (the accountant's view the assistant checks against); the one-row-per-بکری view is پارٹی کھاتے, the statement print (FR-12.47) and the dues histories. PRD FR-19.8 says so. | lead (PRD) |
| Q-R5-2 | Follow the PRD: R5 in the reports menu opens the v1.43 buyers-only sheet; the guide card describes it. | frontend (+backend if a catalogue change is needed) |
| Q-R5-4 | View H (Top 10) is a ranking and is exempt from day, month and year grouping. PRD FR-18.9 says so. | lead (PRD) |
| Q-R4-6 | Keep the DELETE of `venture_partners` before the first entry as a recorded exception to D3 (setup rows, audited, trigger-guarded). | lead (Code Standards D3) |
| Q-T3-1 | Yes: the backup check also verifies both chains and marks a broken copy bad. | backend-c |
| Q-T3-2 | The screen guard is enough for one user on one computer; no backend request key now. | none |
| Q-T3-3 | Reinstated as a check (Umar, 20-11: nothing left open): the daily books check compares sale lines, lot totals, partner share rows, stock moves and analytics facts with the posted journal and warns like a broken chain (NFR-SEC-6 v1.21). The T3 test is un-ignored. | backend-e |
| Q-R7-1 | Back to 0.1.0 until go-live, as Release_Pipeline.md says; the test follows. | backend-c |
| Q-R7-2 | After the 20th wrong try, every further wrong try waits 15 minutes (password and partner code). | backend-c |

### 6.2 Questions as raised

- Q-R1-1. A balance estimate starts at next month (`domain/src/forecast.rs:1005`); the other estimates count the current month as month 1 (FR-7.86, E2E-59). Should a balance estimate also start at the current month?
- Q-R1-2. The "آپ کا" part of a trip-result note is `split_by_bps(result)`; the money that really reaches the partner (his C25 part less his C23 shop parts) can differ by 1 rupee. Which does the note mean? (The note does not move the total.)
- Q-R1-3. What happens to a پیشگی entered before any partner share exists: refuse it, split it by the first shares once set, or show it as unassigned in the FR-17.9 check?
- Q-R1-5. May the lead correct the Code Standards 9.2 table (the real files of E2E-33, E2E-36, E2E-43, and a row for E2E-70)? Doc change only.
- Q-R3-1. FR-11.5 says on the same start date "the one saved last" wins. The code uses the bigger row id (created last), and an unused rule can be edited later. Rule A (id 5) and B (id 6) start on 01-11, A is edited after B: PRD says A wins, code says B. Created last or edited last?
- Q-R4-1. FR-12.29 fixes a garden's shares after its first entry. مال آیا posts nothing and is not a venture entry, so a garden with goods arrived but nothing sold can still change its shares. Does goods arrived count as the first entry?
- Q-R4-2. A garden can be closed while "مالک کے ذمے" (FR-12.51) is still open, and then no refund can be entered. Refuse or warn on closing while the owner still owes?
- Q-R4-3. C27 splits guest fruit over "each partner active on the date". The code uses the partners with a share above 0 that day. They differ only for an active partner at 0%. Which is meant?
- Q-R4-4. With no partner code set yet, every partner figure is open until the first visit to شراکت دار. Is that intended?
- Q-T3-1. The backup check (NFR-BAK-5: data check, counts, checksum) does not verify the hash chains, so a copy of a file changed outside the app is marked good (the restore itself refuses it). Should the backup check also verify both chains and mark such a copy bad?
- Q-T3-2. Two identical بکری saves sent at the same instant are both saved; only the screen stops a double press. Is the screen guard enough, or add a request key in the backend?
- Q-T3-3. NFR-SEC-6 says every money table can be rebuilt and compared from the journal. No check does it: a consistent edit outside the app to a sale line, share row, analytics fact, stock move or lot total is not found. A promise that needs a compare check, or only "the journal is the truth"?
- Q-R7-1. `backend/app/tauri.conf.json` says version 1.0.0 (commit 5dd7d36b); Release_Pipeline.md 3.1 and 3.3 say 0.1.0 until go-live. Back to 0.1.0, or change the doc?
- Q-R7-2. FR-8.2 says after 20 wrong tries "15 minutes each time". The code waits 15 minutes at the 30th, 40th ... try; tries 21 to 29 get no wait. Each wrong try after the 20th, or each further 10?
- Q-R5-1. FR-19.8 v1.43 names R13 and R14 as one row per بکری. They are still one row per journal line (a plan 78 lead note kept R13 line-based for the assistant fixtures; Umar never approved it). Should R13 and R14 follow FR-19.8, or should the PRD say they stay line-based?
- Q-R5-2. The PRD R5 entry v1.43 says the buyers-only sheet replaces the v1.41 sheet. The reports menu R5 still opens the v1.41 اگراہی sheet; the v1.43 sheet opens only from the بکری page button. Which sheet should R5 in the reports menu show?
- Q-R5-3. D6 prompt line 98 says "kul ugrahi: tile buyers_dues"; PRD FR-7.76, E2E-51 and the glossary make کل اگراہی the total_dues tile, and since v1.44 they differ when money is lent. D6 line 131 also lists the retired tile last_backup and not the cash sales or others' dues tiles. May the D6 prompt be corrected (a requirement change, NFR-AI-2 runs again)?
- Q-R5-4. FR-18.9 says every view takes day, month and year grouping. View H (Top 10) refuses it. Is H exempt?
- Q-R5-5. Should آج کی بکری (tile and page) count today's postings (as the tile, the commission page and R34 do today) or only بکری dated today (as the page does)? Either way they will match.
- Q-R6-1. The tool `app_state` (rules, attention, days, audit) came in D6 v1.5 as a lead change. FR-7.87 lists neither it nor `app_help`, and its audit read sends masked audit rows (dates, actions, changed fields, money old and new), which FR-7.83 says never go. Add `app_state` to the PRD and allow masked audit rows, or remove the audit read?
- Q-R6-2. The code stops a model call after 12 s (one retry) and a question after 25 s (plan 55). NFR-AI-3 and Code Standards Q9 say 30 and 90; Code Standards Q8 and Tech Stack 4.8 say 15 and 45. Which are right? May the lead then update the three docs and the out-of-date request fields in Tech Stack 4.8?
- Q-R6-3. FR-7.76 point 2 asks for an attention item when two places differ. It was listed for Umar in plan 48 and never built. Build it, or drop it?
- Q-R2-1. FR-7.70 says a first posting edit can "add or remove a line or lot". The code refuses removing a saved lot (`edit.rs:265`, a plan 16 lead decision before v1.13). Which is right?
- Q-R2-2. FR-21.4 asks the Owner's sign to reopen a day to fix an entry, but the plain reopen of FR-21.2 asks only a reason, and after it any entry can be edited with no sign. Should the plain reopen also ask the sign?
- Q-R2-3. In an edit, a seller has a saved lot with the tick on and a new lot of his is added. Should the new lot take the advance too (one tick covers all) or not (today: not, while the summary shows ticked)?

## 7. LOCAL-ONLY

(Filled by the lead.)
