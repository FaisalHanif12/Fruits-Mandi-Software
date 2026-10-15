# D5. Dashboard, reports, analytics, backup and go-live

Version 1.3, 15-10-2026 (PRD v1.4: E2E-39 in the full run; v1.3: print any day and any party, R33, R34, own goods; v1.2: items in ledgers, the shop's total اگراہی, R32, one user). Deliverable 5 of 5. Read `README.md` in this folder first.
Depends on: D4 done. Plan files: `18-backup-and-restore`, `19-dashboard-and-attention`, `20-reports-and-export`, `21-analytics`, `22-settings-audit-updates`, `23-go-live`.
At the end, version 1.0 is released and ready for the demo of PRD 27.1. (`tauri.conf.json` says 1.0.0; the pipeline publishes it as 1.0.N, where N is the release run number.)

This is the largest deliverable, with six plan files run in order.

## Decisions

| ID | Question | Recommended | Umar's answer |
|---|---|---|---|
| D5-Q1 | May money be written to Excel as numbers? Excel stores numbers as floating point, so this needs one approved exception to the no-float rule at the export boundary (see the `print-and-export` skill, reference.md). | Yes. One function, `rupees_as_f64`, exact below 2^53, tested at the maximum, never read back. Umar runs `/requirement-change` to record it in Code Standards and the money agent before D5 starts. | (waiting) |
| D5-Q2 | Which cloud account do the backup tests use? | Automated tests use a local stand-in for the cloud. Umar creates a Cloudflare R2 test bucket and keys for one real upload and restore check, kept separate from the client's bucket. | (waiting) |
| D5-Q3 | Buy a Windows code-signing certificate before go-live (Tech Stack 7)? | Yes, so Windows SmartScreen does not warn the client on install. Umar adds it as the two GitHub secrets in `docs/Release_Pipeline.md` 3.2. | (waiting) |

Answer D5-Q1 before starting D5. Excel export and the daily backup pack (FR-20.5) are Must items, so without the answer D5 can only end in BLOCKED. Until D5-Q2 is answered, plan file 18 runs against the local stand-in only. D5-Q3 does not block the build; without it, the release is unsigned for Windows and the DONE REPORT says so.

## 1. What Umar can do at the end

He logs in after the worked day. The dashboard shows every tile of PRD 9.1 with the PRD 4.9 numbers: today's sales 41,400 in 2 بکری, cash 58,484, buyers' dues 21,564, sellers payable 10,160, advances open 0, nothing in stock. The partner strip shows الف 996, ب 746, ج 746 for today. The bell shows what needs attention, and آج کے کام lists today's jobs.

He opens any report from R1 to R34, for one day or a date range and for one party where it applies, filters it, prints it in Urdu or English, saves a PDF, exports to Excel. The trial balance balances. Analytics view A shows کیلا and سیب with their commission share, 72.10% and 27.90%. He clicks any number and sees the lines behind it.

At closing time the cloud backup runs by itself. He plugs in a USB drive; a copy goes there too. On a second computer he installs the app, types the backup key, restores, and every balance is identical. A new version appears; the app shows what is new in Urdu, takes a backup, and updates only when he agrees.

The full demo of PRD 27.1 runs from start to end without a problem, and version 1.0 is released.

## 2. Plan files

### 18-backup-and-restore

**Requirements:** PRD 20 (FR-20.1 to FR-20.10), FR-21.1 (cloud backup at day close); NFR-BAK-1 to NFR-BAK-15, NFR-REL-3, NFR-REL-4, NFR-SEC-7, NFR-SPD-15, NFR-INS-2 (the wizard's backup key step), NFR-INS-6; Architecture 14. E2E-18.

**backend builds in `backend/crates/backup/`:**

- Local copy every 15 minutes while data changes, and on exit (FR-20.2, NFR-BAK-2), as a consistent snapshot taken while the app runs (NFR-BAK-4), under NFR-SPD-15.
- Cloud copy at closing time, at app exit and at day close (FR-20.3, FR-21.1); queued and retried every 10 minutes with no internet (FR-20.6, NFR-BAK-9). USB copy when a drive is plugged in (FR-20.4).
- Cloud and USB copies encrypted before they leave the computer, with the key shown once at setup and printed (FR-20.10, NFR-BAK-6, NFR-BAK-7). Secrets in Windows Credential Manager (NFR-SEC-7).
- Every backup opened and checked after it is made; a failure shows on the dashboard (FR-20.7, NFR-BAK-5). Retention as NFR-BAK-8.
- The readable daily pack in the cloud: روزنامچہ PDF and spreadsheets for buyers, sellers and the rest of FR-20.5 (waits for D5-Q1).
- Restore: pick a backup, see its date and totals, confirm; the current data is backed up first (FR-20.9). Test restore into a separate place (NFR-BAK-11). After a restore, the search index is rebuilt.
- `backend/crates/backup/tests/round_trip.rs`: E2E-18.

**frontend builds:** the بیک اپ screen (FR-20.8), the restore flow with its confirm, the wizard's backup key step with print, and the last good cloud backup time in the status bar, red when old (NFR-BAK-10).

**Done when:** E2E-18 passes, and a restore on a fresh install with the key brings back identical balances in under 30 minutes in total (NFR-REL-4).

### 19-dashboard-and-attention

**Requirements:** PRD 9 (9.1 to 9.4, FR-9.1, including the ادھار رقم باقی and اپنا مال tiles), FR-5.6 (the bell), FR-7.12, FR-7.44 to FR-7.46 (with overdue loans and partner money); NFR-SPD-2, NFR-SPD-22, NFR-USE-9, NFR-OBS-5. PRD 27.1 step 7.

**backend builds:** the tile values (each read from posted records, FR-6.1), the partner strip, today's بکری list, the 10 oldest buyer dues, the mini chart series, the attention list and its count (FR-7.44), the daily health check result for the Owner (NFR-OBS-5), and the remembered last screen and choices per user (FR-7.45).

**frontend builds:** the dashboard with tiles (each clickable, FR-9.1), the partner strip, lists, quick actions and mini charts; the bell and آج کے کام; helpful empty states on every list in the app that does not have one yet (FR-7.46); the 5-step first-run guide (FR-7.12).

**Done when:** after the worked day, every tile and the partner strip show the PRD 4.9 values, and the dashboard loads within NFR-SPD-2.

### 20-reports-and-export

**Requirements:** PRD 19 (FR-19.1 to FR-19.10), every report R1 to R34 in 19.2, FR-5.4 (complete global search), FR-7.65, FR-13.9; NFR-SPD-10, NFR-SPD-11, NFR-SPD-13, NFR-SPD-14, NFR-SPD-17, NFR-SPD-18, NFR-SEC-13, NFR-URD-13 to NFR-URD-16, NFR-URD-28; Tech Stack 4.5. Use the `print-and-export` skill.

R1 and R5 exist from D3. This plan file adds R2 to R4 and R6 to R34, and gives every report the same behaviour: filters with date shortcuts, totals, open a row to its source, print, PDF and Excel (FR-19.1 to FR-19.5).

**backend builds:** one query and print data per report, reading posted records only. Party reports group پختہ کھاتہ by person (FR-13.9). Excel through `rust_xlsxwriter` once D5-Q1 is answered: Fluent headings, right-to-left sheets for Urdu, real dates, a totals row, the filters above the table. Exports need the signed-in Owner (NFR-SEC-13). R13 and R14 show the items of each sale line and each seller credit (FR-19.8). R20's میزان is the shop's total اگراہی and equals the dashboard tile (FR-19.9). Long exports run in the background with progress (NFR-SPD-17). Global search finds accounts, بکری, vouchers, trucks and پختہ persons (FR-5.4).

**frontend builds:** the رپورٹ area with the catalogue grouped as in 19.2 (Sales, Accounts, Collections, Business); one report screen pattern used by all; the party report buttons on the party card (FR-7.65).

**Worked checks:** after the worked day, R18 shows the E2E-10 table (85,048 both sides), R12 closes at 58,484, R34 for 10-10-26 shows E2E-36 (2 بکری, خام 41,400, commission 5,448, cash 50,000 + 22,800 − 14,316 = 58,484), R5 filtered to اکرم and 10-10-26 shows only his slip, R20 shows اکرم 2,880 and ارشد 18,684 with میزان 21,564 equal to the dashboard tile, R13 for اکرم shows E2E-33 (کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160, then the receipt, closing 2,880 بنام), R24 shows کریم 10,160, R26 shows رحیم given 15,000, adjusted 15,000, open 0, and R30 and R31 show the PRD 4.9 partner numbers; after E2E-35, R31 for ب shows the E2E-37 lines with the running کل رقم.

**Done when:** every report opens, totals, prints and exports in both languages, and the worked checks pass.

### 21-analytics

**Requirements:** PRD 18 (FR-18.1 to FR-18.7), views A to H, 18.4, FR-13.9, C18, own goods in the views (FR-12.27); NFR-SPD-12. E2E-11.

**backend builds:** every view from the analytics facts written at save (FR-18.6), never by recalculating sales. Cancelled postings cancel themselves out (FR-18.7). Filters combine freely (FR-18.1). Drill down from any number to its lines and then to the بکری (FR-18.2). Saved views (FR-18.5). Percentages to 2 decimals as 18.2 defines.

**frontend builds:** the تجزیات area: filters, a simple chart above each table (FR-18.3), drill-down, saved views, print and Excel (FR-18.4).

**Done when:** E2E-11 passes: view A as PRD 18.4, including avg rate 588 / 1,200 and 72.10% / 27.90%, and view F as PRD 18.4.

### 22-settings-audit-updates

**Requirements:** PRD 21 (every settings group not yet built), FR-21.3 (the audit viewer), FR-17.6 and FR-21.2 (month close and its reversal); NFR-SEC-5, NFR-SEC-11, NFR-INS-3 to NFR-INS-5, NFR-REL-8, NFR-REL-13, NFR-OBS-4.

**backend builds:**

- The remaining settings: Checks, Day (closing time, auto day close), Language (number grouping), and any other row of the PRD 21 table not built in D1 to D4.
- Month close: one journal per finished month as FR-17.6 says; a month-closed month cannot be reopened until the Owner reverses its month close (FR-21.2).
- The audit viewer data: filters by user, date, screen and record; read-only (FR-21.3).
- The daily automatic books check (NFR-REL-8), feeding the health check.
- Updates: check when online, show what is new in Urdu from the release notes, install only when the Owner agrees and never during work, take a full backup first, refuse an unsigned update (NFR-INS-3, NFR-INS-4, NFR-SEC-11, NFR-REL-13). Uninstall never touches the data folder or backups (NFR-INS-5).
- Support info: collects the log, app version and health check into one file for Umar (NFR-OBS-4).

**frontend builds:** the full Settings area, month close, the audit viewer, the update prompt, and the support info button.

**Done when:** an update from one test release to the next takes a backup, keeps all data and settings, and refuses a release with a bad signature.

### 23-go-live

**Requirements:** PRD 27 (27.1 demo script, 27.2 all tests), PRD 24 (assumptions to check at the demo); NFR Section 2 (every speed target), NFR-SCL-1 to NFR-SCL-4, NFR-REL-5, NFR-REL-12, NFR-USE-15 to NFR-USE-17, NFR-USE-28, NFR-URD-4, NFR-URD-8, NFR-INS-1; Code Standards 9.2 and 9.4. E2E-1 to E2E-27.

This plan file adds no new feature. It proves the whole app.

**robustness builds:** the data generator in `scripts/perf/` for 10 years at NFR-SCL-1 volumes; every NFR Section 2 target as a benchmark in `pnpm perf`; a 12-hour run without slowing down or memory growth (NFR-REL-12, NFR-SPD-16).

**frontend builds:** the Windows end-to-end suite completed: the full keyboard run of PRD Section 4, E2E-17 and E2E-22 to E2E-27 as full screen runs, screenshots of every screen in both languages at every size and scaling of NFR-USE-15 to NFR-USE-17.

**backend builds:** any fix the runs find; version `1.0.0` in `tauri.conf.json`.

**Lead:**

- Writes `docs/demo/demo-checklist.md`: the PRD 27.1 steps with the expected numbers, the PRD 24 assumptions to ask the client about, the user measures to take with the munshi and Owner (NFR-USE-3 to NFR-USE-5), and the BitLocker recommendation for the shop laptop (NFR-SEC-8).
- Writes `docs/demo/urdu-review.md`: every screen and print with new Urdu text, for the Urdu reviewer and the munshi (NFR-URD-4, NFR-URD-8).
- Updates `docs/release/whats-new.md` for version 1.0 and gives Umar the manual Windows checklist from the `windows-verify` skill (Step 4).

**Done when:** every test passes, every target is met at its runner threshold, and the first release after the 1.0.0 setting is green. The lead adds to Umar's checklist: a no-internet install on a clean Windows computer, and a speed run on the reference hardware of NFR 0.2.

## 3. Design for D5

The lead runs `/design` inside `docs/design/design-system.md` for:

1. **The dashboard** after the worked day: tiles, partner strip, lists, quick actions, mini charts, the bell and آج کے کام. This is the first screen the Owner sees every day; it must read at a glance.
2. The report screen pattern (filters, table, totals, actions) and three printed reports: R11 روزنامچہ, R13 کھاتہ, R18 trial balance, in A4, both languages.
3. Analytics views A and F with their charts and drill-down.
4. The backup screen, the restore confirm, the backup key step with print.
5. Settings, the audit viewer, the update prompt.

## 4. Team flow for D5

- Plan files run in order: 18, 19, 20, 21, 22, 23 (README Section 4).
- 18 (backup) first: the dashboard's last-backup tile, day close and the update's backup-first step all need it.
- 21 (analytics) after 20, because it reuses the report screen pattern.
- 23 only when 18 to 22 are merged.

## 5. Review focus for D5

- Reports and the dashboard read posted records only; no report keeps its own totals (FR-6.1, FR-19.5).
- Prints: numbers never flip, Urdu inside English keeps its direction, headers repeat, rows never split (FR-19.6).
- Backup: encryption before leaving the computer; the key never in a log; a restore always backs up first.
- **The final whole-app review:** every PRD Must and Should item against the build, every NFR group, and a regression run of every earlier acceptance item. The verdict goes into `docs/reviews/final-1.0.md`.

## 6. Robustness attack list for D5

1. **Scale.** 10 years of generated data at NFR-SCL-1: every report, the ledger, the trial balance and analytics within NFR-SPD-10 to NFR-SPD-13; the database under the NFR-SCL-4 size.
2. **Backup faults.** Disk full during a backup; USB pulled out half way; internet off for a day, then on (the queue catches up); a corrupted backup file (found by the check, never restored silently); the wrong key on restore (clear message, nothing changed); restore of 10 years under 5 minutes (NFR-BAK-12).
3. **Update faults.** An update with a bad signature (refused); power cut during an update (data safe, backup taken first).
4. **Security.** Every report and export refused before sign-in and while locked; no keys or balances in logs; the backup key not readable from the data file.
5. **Long run.** 12 hours of mixed work without slowing down or memory growth.
6. **Everything at once.** The full E2E-1 to E2E-27 run, in Urdu and in English, at 1366 × 768 and 125% scaling.

## 7. Acceptance (D5-A1 to D5-A21)

| ID | Item | Evidence expected |
|---|---|---|
| D5-A1 | Every dashboard tile and the partner strip show the PRD 4.9 values; every tile opens its screen | Test output; screenshot |
| D5-A2 | Attention list and آج کے کام; count updated within NFR-SPD-22 | Test output |
| D5-A3 | Every report R1 to R34 opens, filters, totals and opens rows to their source | Test list output |
| D5-A4 | Every report prints and saves as PDF in Urdu and English; headers repeat; numbers never flip | PDFs from the Windows run |
| D5-A5 | Excel export with real numbers, dates, totals and right-to-left Urdu sheets (after D5-Q1) | Excel test output |
| D5-A6 | The plan 20 worked checks pass (R12, R13, R18, R20, R24, R26, R30, R31); E2E-33 passes in `ledger_items.rs`; after E2E-32, R20's میزان and the وصولی باقی tile both show 24,564; after E2E-28, the ادائیگی باقی tile and R24 show 19,316 | Test output |
| D5-A7 | E2E-11 passes; drill-down from a view number to the بکری | Test output |
| D5-A8 | Saved views; analytics within NFR-SPD-12 over 5 years | Test and benchmark output |
| D5-A9 | Month close and its reversal; the audit viewer filters and is read-only | Test output |
| D5-A10 | Update: backup first, data kept, unsigned update refused, never during work | Test output |
| D5-A11 | E2E-18 passes | round_trip output |
| D5-A12 | Local, USB and cloud copies made, encrypted and checked; offline queue retries | Test output |
| D5-A13 | Restore on a fresh install with the key gives identical balances; NFR-REL-4 and NFR-BAK-12 met | Test output and timings |
| D5-A14 | Day close starts the cloud backup; the status bar shows the last good backup time | Test output |
| D5-A15 | Every NFR Section 2 target measured the way the NFR names and met at its runner threshold | Table of each target, its runner threshold and the measured value |
| D5-A16 | NFR-SCL-1 to NFR-SCL-4 met with 10 years of generated data | Robustness report |
| D5-A17 | 12-hour run with no slowdown or memory growth (NFR-REL-12) | Robustness report |
| D5-A18 | E2E-1 to E2E-39 all pass: Rust golden tests and the full Windows suite, in both languages | golden_day, assist and round_trip output; Windows run ID |
| D5-A19 | Final whole-app review PASS or PASS WITH MINOR; `docs/demo/demo-checklist.md` and `docs/demo/urdu-review.md` written | `docs/reviews/final-1.0.md` |
| D5-A20 | The first release after `tauri.conf.json` is set to 1.0.0 is green (version 1.0.N); installer installs in under 2 minutes on the runner; PRD 27.1 demo steps 1 to 11 run end to end in the Windows suite (step 12 is Umar's, with the client) | Release run ID and version; install time; demo run result |
| D5-A21 | Print any day and any party (FR-19.10): every report, ledger, register and list screen has the date and party filters, and the print header shows them; E2E-36 passes; R33 shows E2E-34b (purchase −1,940) | Test output; PDFs |

## 8. Not in D5 (and not in this build)

- PRD 26 "Could" (Phase 2): importing 2018 to 2026 data from the old software, crate tracking, a goods return screen, interest on old dues, a daily rate board.
- PRD 26 "Won't": several computers, the firm's own stock trading, tax filing.
