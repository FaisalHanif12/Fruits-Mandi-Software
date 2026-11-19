# D5 in parallel: an early track beside the D3+D4 run

Version 1.1, 22-10-2026. Umar's decision of 22-10-2026: D5 work that does not depend on plans 16b, 17 or D4R starts now, in a second working folder, while the D3+D4 run finishes. Where this file and README Section 4 or D5 "Depends on" differ, this file wins for the early track only. Requirements, acceptance items and golden numbers do not change.

## 1. Two runs, two folders, one source of truth

| Run | Folder | Branch | Builds |
|---|---|---|---|
| Run A (now running) | `Fruit Mandi Project` | `feature/bikri` | D3+D4: plans 16b and 17, the end gate, the DONE REPORT. Then D4R on `feature/refresh-v16`. |
| Run B (new) | `Fruit Mandi Parallel/d5` (a git worktree) | `feature/d5-early`, made from `feature/bikri` | The D5 early track (Section 2) |

- `main` is not touched. In the Mac phase nothing merges to `main` (README Section 10), and `main` does not have the D3 and D4 code that D5 needs.
- `feature/bikri` is the source of truth until the D3+D4 DONE REPORT; then `feature/refresh-v16`. Run B merges the current one into its own branch after every plan commit of Run A (`git fetch`, then `git merge`; never rebase a pushed branch). After each merge it runs `pnpm gen:types`, `cargo test -p mandi-services --test golden_day` and its own tests.
- Each folder has its own `node_modules` and its own Cargo `target` folder. Run B runs `pnpm install` once.

## 2. What Run B builds now (D5 early track)

Everything here reads data that plans 05 to 16 already write, so it does not wait for 16b, 17 or D4R.

| Plan | Early part | Waits for the late part |
|---|---|---|
| 18 backup and restore | All of it, except the readable pack's spreadsheets (wait for D5-Q1) | Spreadsheets of FR-20.5 |
| 20 reports | The report screen pattern, filters, date shortcuts, paging (FR-19.11), print and PDF, open a row to its source; reports R2, R3, R4, R6, R7, R8, R9, R11, R13, R14, R16, R17, R18, R19, R20, R21, R22, R24, R26, R27, R28, R30; global search (FR-5.4); party report buttons (FR-7.65) | R10 (fruit taken column, D4R 17d), R12 and R29 (plan 17), R15, R23, R25 (no mode, D4R 17b), R31 and R32 (plan 17), R33 (16b), R34, R35, R36, R37; the R5 screen (template from 17e); Excel (D5-Q1) |
| 21 analytics | Views A, B, C, D, E, H, filters, charts, drill-down, saved views, print | Views F, G, I, J, K, and the garden, trip and goods filters (they need 16b, 17c, 17d) |
| 22 settings, audit, updates | Remaining settings (Checks, Day, number grouping), audit viewer, daily books check, updates, support info | Month close (plan 17 writes its statement note line) |
| 19 dashboard, 23 go-live | Nothing | All |

Prints of the early reports use the shared print layout of plan 14. D4R plan 17e later upgrades that shared header and footer to FR-19.13, so every report gets it with no second change.

## 3. Rules that keep the two runs from colliding

1. **No migration in Run B.** Migration 0006 belongs to Run A and stays open until D4R ends. If Run B needs a column or table, it writes the need in the coordination file (Section 4) and waits for Run A to add it to 0006.
2. **Shared files are append-only for Run B:** command registration (`backend/app/src/commands/mod.rs`, `lib.rs`), capabilities, `frontend/src/app/router.tsx`, Fluent `common.ftl` and `glossary.ftl`. Run B adds its own lines in its own block and never edits or moves Run A's lines. New modules live in their own folders (`reports`, `analytics`, `backup`, `audit`).
3. **Run B never changes** a money function, a posting, a domain rule, a golden test or a file in `backend/crates/core`, `domain` or `services/src/{bikri,vouchers,ledger,partners,ventures,trips,purchases}`. If a report needs a new read query, it goes in `mandi-reports`. If it needs a change to code Run A owns, it asks in the coordination file.
4. **Golden numbers.** After every merge from Run A, every golden test passes with no number changed. A failure caused by the merge is reported to Run A in the coordination file, not fixed by editing Run A's code.
5. **Builder count.** Many agents do not make Rust builds faster: they share one CPU and Cargo locks its `target` folder. Each run keeps at most 4 builders at a time, plus its reviewer and robustness agents at the gate. Agents run one crate's tests while they work (`cargo test -p <crate>`), and the full suite only at the end of a plan.

6. **Shared machine.** Both folders share the CPU, the app's data folder, its Keychain entries and port 5173. So Run B never runs `pnpm dev`, `pnpm seed`, the app or `pnpm perf` while Run A has a workflow step running; its tests use temporary databases only. While Run A runs its full checks, `pnpm perf` or its end gate, Run B pauses all builds, so Run A's speed numbers stay true and its gate is not slowed.
7. **Run A comes first.** Run B never delays plan 17, the D3+D4 end gate or the DONE REPORT. If the two compete for the machine, Run B waits.
8. **Separate gates.** The D3+D4 end gate reviews `feature/bikri` only. Before its handoff, Run B gets its own reviewer and robustness pass over `feature/d5-early` (README Section 4a), with the same quality bar.
9. **When unsure, Run B stops, not Run A.** Run B writes the question in the coordination file and waits for Umar. It never widens plan 17 to suit D5.
10. **Excel waits for D5-Q1.** Until Umar answers it, Run B builds print and PDF only, with no Excel export of money.
11. **Disk.** Two Cargo `target` folders need room. Run B checks free disk first and stops with a message if less than 20 GB is free.

## 4. How the two runs talk

Run A and Run B may be two sessions, or two workflows of one tech-lead session. Either way they keep one shared record outside both folders: `Fruit Mandi Parallel/coordination.md`.

- After every plan commit, each lead appends one short entry: date and time, run, branch, commit, what is now available (for example "16b: `trips_summary()` and the 0604 accounts are on feature/bikri at abc1234"), and any request to the other run.
- Before starting a plan file, each lead reads the file from the top.
- A request is answered in the same file. Example: "Run B: R10 needs `bikri_lots.qty_taken`. Run A: it is in 0006 at def5678; merge feature/bikri."

## 5. End of the early track

Run B ends with a "D5 EARLY HANDOFF" message in the coordination file and in its session: each early part with its tests and real command output, and a list of the late parts. It does not post a D5 DONE REPORT.

After the D4R DONE REPORT, the D5 goal line of README Section 3 runs on a branch made from `feature/refresh-v16` with `feature/d5-early` merged in. It builds the late parts and proves every item D5-A1 to D5-A26, as written.
