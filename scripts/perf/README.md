# Speed harness and runner factor

Owner: `robustness`. Plan 01 T5. Requirements: NFR Section 0.2 and Section 2, NFR-INS-1, Code Standards B25, deliverables README Section 6.

## 1. What `pnpm perf` does

`pnpm perf` runs `node scripts/perf/run.mjs`. It:

1. Runs every criterion benchmark of the Rust workspace: `cargo bench --workspace --bench '*'` in `backend/`. Only `[[bench]]` targets are built. (Plain `cargo bench` would also build the Tauri app in the release profile, which takes many minutes and measures nothing.)
2. Reads each result from `backend/target/criterion/<group>/<bench>/new/` and compares it with its entry in `thresholds.json`.
3. On Windows only: if `backend/target/release/bundle/nsis/*-setup.exe` exists, installs it silently (`/S`), times the install, checks the app exe is in the install folder, closes the app if it started, and uninstalls it silently. The time is compared with NFR-INS-1 (under 2 minutes). On a Mac it prints "install timing runs on Windows only".
4. Prints one table and the machine's CPU, writes `backend/target/perf/report.json`, and exits:
   - `0` when every row is OK (or SKIPPED),
   - `1` when any row is OVER, MISSING, NO THRESHOLD or ERROR,
   - `2` when the harness itself cannot run (cargo fails, a bad `thresholds.json`).

Options: `--skip-bench` (read the last results, do not run cargo), `--skip-install`, `--require-install` (on Windows, fail if no installer was built; use this in the Windows workflow after `pnpm build`).

Rules the harness enforces:

- Every criterion benchmark must have an entry in `thresholds.json`. A benchmark with no entry fails (NO THRESHOLD). An entry whose benchmark did not run in this run fails (MISSING). Old results left in `target/` from a deleted benchmark never count.
- `runner_factor` must be at least 1.0. A target is never loosened because the runner is fast.

Example output on the Mac (10-10-2026):

```
Action                           | NFR          | Measured | Stat    | Mean     | Runner threshold | NFR target | Result
Calibration (fixed CPU workload) | factor check | 139.1 us | mean    | 139.1 us | 228.0 us         | -          | OK
Install (NSIS, silent)           | NFR-INS-1    | -        | one run | -        | 120.00 s         | 120.00 s   | SKIPPED
perf: machine Apple M1 Pro, 8 logical CPUs, 16.0 GB memory
perf: PASS
```

## 2. The runner factor

**Factor: 1.0.** Recorded once on 10-10-2026. Later plan files only add rows to Section 5 and entries to `thresholds.json`.

The reference machine of NFR 0.2 (Intel Core i3 7th generation, 4 GB, SSD) is not available. So the factor is set from public single-thread CPU scores, and a fixed calibration benchmark checks that estimate on every run (Section 3). Single-thread scores are the right measure because every NFR speed target is one user action handled on one thread (one SQLite write connection, Code Standards B21).

| Machine                                      | CPU                                  | PassMark single-thread rating | Source (read 10-10-2026)                                                                                                              |
| -------------------------------------------- | ------------------------------------ | ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Reference (NFR 0.2 minimum)                  | Intel Core i3-7100, 3.9 GHz, 7th gen | **2,268**                     | [cpubenchmark.net i3-7100](https://www.cpubenchmark.net/compare/2924vs2923/Intel-Core-i3-7100-vs-Intel-Core-i3-7300)                  |
| GitHub `windows-latest` (image windows-2025) | AMD EPYC 7763                        | **2,517**                     | [cpubenchmark.net EPYC 7763](https://cpubenchmark.net/compare/4207vs4365/AMD-EPYC-7763-vs-AMD-EPYC-7V13)                              |
| GitHub `windows-2025-vs2026` image           | AMD EPYC 9V74                        | **2,888**                     | [cpubenchmark.net EPYC 9V74](https://cpubenchmark.net/compare/6535vs5635/AMD-EPYC-9V74-vs-AMD-EPYC-9R14)                              |
| Developer Mac (calibration only)             | Apple M1 Pro, 8 cores                | **3,752**                     | [cpubenchmark.net M1 Pro 8 core](https://www.cpubenchmark.net/compare/4580vs4584/Apple-M1-Pro-10-3200-MHz-vs-Apple-M1-Pro-8-3200-MHz) |

The runner hardware: for a public repository, `windows-latest` is a Windows Server 2025 virtual machine with 4 CPUs, 16 GB memory and a 14 GB SSD ([GitHub docs: GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)). (A private repository gets 2 CPUs and 8 GB. Single-thread speed is the same CPU class, so the factor holds, but the long run of NFR-SPD-16 should be read with less memory in mind.) GitHub does not publish the CPU model. The two models above come from Geekbench uploads tagged as GitHub Actions Windows runners ([upload 1](https://browser.geekbench.com/v6/cpu/baseline/18567798), [upload 2](https://browser.geekbench.com/v6/cpu/baseline/18567802)). Those pages block automatic reading, so the model names were taken from the search result text. That is why the harness prints the CPU model on every run: the first Windows run confirms it.

The working:

- Slowest known runner CPU against the reference: 2,517 ÷ 2,268 = 1.11. The runner is about 11% faster.
- Faster runner CPU: 2,888 ÷ 2,268 = 1.27.
- Rule: if the runner is faster than the reference, the factor is 1.0. If it is slower, threshold = target × (reference time ÷ runner time).
- So the factor is **1.0** and every runner threshold equals its NFR target.

## 3. The calibration check (repeatable)

Scores from a website are an estimate. Our workload is not PassMark's, and a cloud CPU is a shared, hyper-threaded virtual CPU whose clock varies. So the harness runs a fixed workload every time and checks the estimate.

- Benchmark: `calibration/cpu_fixed_10k` in `backend/crates/core/benches/calibration.rs`. 10,000 fixed amounts and rates: multiply in `i128`, divide by 10,000, round half up, add with checked operations, sort. No database, no file, no clock. The code MUST NOT change; real benchmarks go in other files.
- Measured on the developer Mac (Apple M1 Pro, PassMark 3,752): mean **0.138 ms**, the middle of five runs (0.1377, 0.1380, 0.1384, 0.1450, 0.1482 ms). Other builds were running on the Mac at the time, so this is if anything slow, which makes the check below lenient, never strict.
- Expected on the reference i3-7100: 0.138 × 3,752 ÷ 2,268 = **0.228 ms**. This is `expected_reference_ms` in `thresholds.json`.
- Expected on the runner: EPYC 7763: 0.138 × 3,752 ÷ 2,517 = 0.206 ms. EPYC 9V74: 0.138 × 3,752 ÷ 2,888 = 0.179 ms.

On every run the harness compares the calibration mean with 0.228 ms:

- At or under: OK. The runner is at least as fast as the reference on this work, so factor 1.0 holds.
- Over: the row shows CHECK FACTOR and a warning. It does not fail the run, because a slow runner only makes the real targets harder to meet, never easier. When it happens, `robustness` takes the calibration mean of three Windows runs, and if it is still over, sets factor = mean ÷ 0.228 (rounded up to the next 0.05), records the runs here, and updates `runner_factor`.

How to repeat the Mac measurement: `node scripts/perf/run.mjs` five times on an idle Mac, take the middle calibration mean.

## 4. How each number is measured

- **Criterion benchmarks.** Criterion takes samples; each sample runs the code a number of times. The harness divides each sample's time by its run count and takes the 95th percentile of those values (nearest rank). That is the "p95" in the table. For a benchmark of one user action (milliseconds or more), set `group.sampling_mode(SamplingMode::Flat)` and a `sample_size` of at least 20, so each sample is a few single calls and the p95 is close to the p95 of one action. Each benchmark runs on a database built with the NFR-SCL-1 volume (1,200,000 sale lines, 3,500,000 journal lines, 20,000 accounts) from the generator described in Section 7.2 (`backend/crates/services/benches/support/mod.rs`).
- **E2E timings** come from the Windows end-to-end suite (`pnpm e2e`, `frontend/tests/e2e/`): the test reads `performance.now()` in the page around the action, or the time from process start to the first screen. Each action runs at least 20 times; the p95 is reported against the runner threshold.
- **Long run** (NFR-REL-12, NFR-SPD-16): the E2E suite loops through screens and saves for a fixed time and samples the app's memory (working set) every minute. Pass: under the limit and no upward trend.
- **Install** (NFR-INS-1): timed by this harness on Windows.

Two caveats:

- **WebView2 and the screen.** The runner has no GPU and runs WebView2 in a headless-like virtual display. Screen timings there are a strong guide, not proof. The real check of every target is on reference hardware before go-live (D5, plan 23).
- **Disk.** The runner's disk is a cloud SSD whose write-and-flush time differs from a laptop SSD. Save benchmarks use `synchronous = FULL` as the app does. The HDD target of NFR-SPD-12 (3 seconds) cannot be measured on the runner and is checked only on reference hardware.

## 5. Every NFR speed target and its runner threshold

Factor 1.0, so each runner threshold equals its NFR target. "Plan" is where the measurement is added. "Now" means measured from plan 01.

| ID         | Action                                                   | NFR target                                    | Runner threshold                                                   | How measured                                                                                                       | Plan                                    | Status  |
| ---------- | -------------------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | --------------------------------------- | ------- |
| NFR-SPD-1  | Cold start to login screen                               | < 2 s                                         | 2 s                                                                | E2E timing: process start to login screen shown, p95 of 20 starts                                                  | 01 (first screen), 04 (login)           | Pending |
| NFR-SPD-2  | Login to dashboard fully loaded                          | < 1 s                                         | 1 s                                                                | E2E timing, p95                                                                                                    | 04                                      | Pending |
| NFR-SPD-3  | Open any screen from the menu                            | < 300 ms                                      | 300 ms                                                             | E2E timing per screen, p95                                                                                         | 04, then each screen                    | Pending |
| NFR-SPD-4  | Key press to character on screen                         | < 50 ms                                       | 50 ms                                                              | E2E timing: key event to DOM change, p95 of 100 keys, in a field and a grid cell                                   | 04, 11 (grid)                           | jsdom guard 12-10-2026 (`frontend/tests/robustness/key-latency.test.tsx`); Windows E2E still to do |
| NFR-SPD-5  | Arrow move between cells or fields                       | < 30 ms                                       | 30 ms                                                              | E2E timing: key event to focus change, p95                                                                         | 04, 11                                  | jsdom guard 12-10-2026 (same file); Windows E2E still to do |
| NFR-SPD-6  | Picker search, 20,000 accounts, typos, both names        | < 50 ms                                       | 50 ms                                                              | Criterion: `services::search` on 20,000 generated accounts, p95; plus E2E                                          | D2 (search)                             | Measured D2 (Section 8) |
| NFR-SPD-7  | Live summary after a sale line change (incl. 50 ms wait) | < 150 ms                                      | 150 ms                                                             | Criterion: `bikri_preview` with 30 lines, p95 must be under 100 ms (150 minus the 50 ms wait); plus E2E end to end | D3                                      | Pending |
| NFR-SPD-8  | Save a بکری with 30 sale lines                           | < 300 ms                                      | 300 ms                                                             | Criterion: `services::bikri::save` on the NFR-SCL-1 database, p95                                                  | D3                                      | Pending |
| NFR-SPD-9  | Save any voucher                                         | < 150 ms                                      | 150 ms                                                             | Criterion: each voucher save on the NFR-SCL-1 database, p95                                                        | D2 (پیشگی), D4                          | Measured D2 for the advance (Section 8) |
| NFR-SPD-10 | Any report for one day                                   | < 1 s                                         | 1 s                                                                | Criterion: each report query for one busy day (5,000 sale lines), p95                                              | D5                                      | Cash book of one day measured D2 (Section 8); the other reports are D5 |
| NFR-SPD-11 | Party ledger for 10 years                                | < 2 s                                         | 2 s                                                                | Criterion: busiest party, 10 years, p95                                                                            | D5                                      | Measured D2 (Section 8) |
| NFR-SPD-12 | Any analytics view over 5 years                          | < 1 s SSD, < 3 s HDD                          | 1 s (SSD only)                                                     | Criterion: each view A to H over 5 years, p95. HDD on reference hardware only                                      | D5                                      | Pending |
| NFR-SPD-13 | Trial balance for 10 years                               | < 2 s                                         | 2 s                                                                | Criterion, p95                                                                                                     | D5                                      | Measured D2 (Section 8) |
| NFR-SPD-14 | Print preview of a 15-page report                        | < 2 s                                         | 2 s                                                                | E2E timing: preview command to preview shown, p95                                                                  | D3 (first print), D5                    | Pending |
| NFR-SPD-15 | Local backup snapshot, 300 MB                            | < 5 s                                         | 5 s                                                                | Criterion or timed test in `mandi-backup` on a 300 MB database, p95 of 10                                          | D5                                      | Pending |
| NFR-SPD-16 | Memory in normal use                                     | < 400 MB                                      | 400 MB                                                             | Long run: working set sampled every minute                                                                         | 01 (D1 attack list, 30-minute loop), D5 | Pending |
| NFR-SPD-17 | Long tasks show progress, never freeze                   | Rule: progress shown; screen keeps responding | Same rule; key press during the task still under 50 ms (NFR-SPD-4) | E2E: start an export or restore, type during it                                                                    | D5                                      | Pending |
| NFR-SPD-18 | Lists with thousands of rows scroll smoothly             | Rule: only visible rows are drawn             | Same rule: rendered row count stays under 100 with 10,000 rows     | E2E: count rendered rows while scrolling                                                                           | D3 (first big grid), D5                 | Pending |
| NFR-SPD-19 | Language switch, incl. a بکری with 30 unsaved lines      | < 500 ms                                      | 500 ms                                                             | E2E timing: `Alt+L` to `dir` changed and text redrawn, p95                                                         | 01 (first screen), 04, D3 (بکری)        | Pending |
| NFR-SPD-20 | English-to-Urdu suggestions per key                      | < 50 ms                                       | 50 ms                                                              | Criterion: `translit_suggest` for the 1,000-name list, p95                                                         | D2                                      | Measured D2 (Section 8) |
| NFR-SPD-21 | Party card, 10 years of data                             | < 100 ms                                      | 100 ms                                                             | Criterion: `assist_party_card` on the NFR-SCL-1 database, p95                                                      | D2                                      | Measured D2 (Section 8) |
| NFR-SPD-22 | Attention list count after a save                        | within 1 s                                    | 1 s                                                                | Integration timing: save to updated count, p95                                                                     | D5                                      | Pending |
| NFR-SPD-23 | Rate hint, seller and truck memory                       | < 100 ms                                      | 100 ms                                                             | Criterion: each `assist_*` query on the NFR-SCL-1 database, p95                                                    | D3                                      | Pending |

Other measured targets:

| ID         | Action                                     | NFR target             | Runner threshold | How measured                                                                                           | Plan       | Status                       |
| ---------- | ------------------------------------------ | ---------------------- | ---------------- | ------------------------------------------------------------------------------------------------------ | ---------- | ---------------------------- |
| NFR-INS-1  | Install with no internet needed            | < 2 minutes            | 2 minutes        | This harness on Windows (silent NSIS install). The no-internet install is on Umar's go-live checklist. | 01         | Measured on each Windows run |
| NFR-SCL-6  | Find the winning rule, 10,000 rules        | < 1 ms                 | 1 ms             | Criterion: rule resolution, p95                                                                        | D2 (rules) | Measured D2 (Section 8)       |
| NFR-REL-12 | 12 hours with no slowdown or memory growth | No slowdown, no growth | Same             | Long run, simulated with many operations                                                               | D5         | Pending                      |

When a measurement is added, its plan file adds an entry to `thresholds.json` (`kind: "nfr"`, `nfr`, `action`, `stat: "p95"`, `target_ms`) and changes its Status here to "Measured".

## 6. Budgets for core functions (plan 02)

The NFR sets targets for user actions, not for one function. A core function (parse, percent charge, split, search key) is a small part of an action, so it gets a **budget**: its share of the action's NFR target. Each budget is a row in `thresholds.json` with `kind: "nfr"`, the NFR it comes from marked "(budget)", and the p95 compared as for any other row. Factor 1.0 applies.

Benchmarks: `backend/crates/core/benches/money.rs` and `backend/crates/core/benches/text.rs`. Both need a `[[bench]]` entry with `harness = false` in `backend/crates/core/Cargo.toml` (owned by `backend`).

### 6.1 Money maths, from NFR-SPD-7

- The live summary must show 150 ms after the last key. 50 ms is the wait for the user to stop typing, so the preview has **100 ms** (Section 5).
- The largest بکری has 500 sale lines (NFR-SCL-3). The preview also reads SQLite, crosses the Tauri bridge and builds the journal and the working records. The pure money maths gets 10% of the 100 ms: **10 ms for 500 lines, 20 µs per line**.
- One line parses about 3 texts (qty, seller rate, buyer rate) and makes about 3 percent charges (seller commission, buyer commission, لاگا as a percent). 3 × 2 µs + 3 × 1 µs = 9 µs of the 20 µs. The rest is for `line_value`, `per_unit` and the sums.
- A partner split runs once per posting, not per line.

| Benchmark id | What it runs | Budget | Working |
|---|---|---|---|
| `money/parse_rupees` | `parse_rupees("1,23,456")` | 2 µs | per-line share, above |
| `money/apply_bps` | `apply_bps(5,630, 6%)` | 1 µs | per-line share, above |
| `money/split_by_bps_3` | split 3,948 at 40/30/30 | 5 µs | once per posting |
| `money/split_by_bps_10` | split 3,948 over 10 partners (NFR-SCL-5 maximum) | 10 µs | once per posting |
| `preview/maths_500_lines` | 500 lines: parse qty and both rates (some typed as `20+25` and `1,234`), both line values, both commissions, لاگا, all totals, one 3-partner split | 10 ms | 10% of 100 ms |

### 6.2 Search keys, from NFR-SPD-6 and NFR-SPD-1

- The in-memory search index builds the three keys (`search_ur`, `search_en`, `search_phonetic`) for every account at start-up and after a restore (Tech Stack 4.7). With 20,000 accounts (NFR-SCL-1), the build gets 10% of the 2 s cold start (NFR-SPD-1): **200 ms, so 10 µs per account**.
- On each key press in a picker, the typed text gets the same keys. NFR-SPD-6 gives the whole search 50 ms; building the query keys must take under 1% (0.5 ms). The 10 µs per name budget meets that with a wide margin.

| Benchmark id | What it runs | Budget | Working |
|---|---|---|---|
| `text/normalize_urdu_name` | `normalize_urdu` on a typical name with Arabic letter variants, a diacritic and double spaces | 5 µs | half of the 10 µs per account |
| `text/search_keys_name` | all three keys of that name | 10 µs | 200 ms ÷ 20,000 |
| `text/search_keys_name_100` | all three keys of a 96-character Urdu and 91-character English name (near the 100-character maximum, Code Standards 4.2) | 30 µs | worst case; the average stays at 10 µs |
| `index/keys_20000_accounts` | all three keys for 20,000 generated accounts | 200 ms | 10% of NFR-SPD-1 |

### 6.3 Measured on the developer Mac (10-10-2026)

Criterion mean on the Apple M1 Pro. "Reference estimate" = Mac × 3,752 ÷ 2,268 (Section 2), the expected time on the i3-7100.

| Benchmark id | Mac | Reference estimate | Budget | Headroom on the reference |
|---|---|---|---|---|
| `money/apply_bps` | 2.9 ns | 4.8 ns | 1 µs | about 200 × |
| `money/parse_rupees` | 139 ns | 230 ns | 2 µs | about 8 × |
| `money/split_by_bps_3` | 87 ns | 145 ns | 5 µs | about 34 × |
| `money/split_by_bps_10` | 251 ns | 415 ns | 10 µs | about 24 × |
| `preview/maths_500_lines` | 253 µs | 419 µs | 10 ms | about 24 × |
| `text/normalize_urdu_name` | 470 ns | 777 ns | 5 µs | about 6 × |
| `text/search_keys_name` | 1.33 µs | 2.2 µs | 10 µs | about 4.5 × |
| `text/search_keys_name_100` | 5.0 µs | 8.3 µs | 30 µs | about 3.6 × |
| `index/keys_20000_accounts` | 19.7 ms | 32.6 ms | 200 ms | about 6 × |

The budgets are hard limits taken from the NFR, not alarms set just above today's time. A slowdown that stays inside a budget passes; one that breaks it means the action's NFR target is at risk and is a finding.


## 7. The D2 benchmarks and the NFR-SCL-1 data

Added 12-10-2026 for plans 03 to 10. Owner: `robustness`.

### 7.1 What runs

`pnpm perf` runs these benchmark files (`[[bench]]` entries with `harness = false`):

| File | Group ids | NFR |
|---|---|---|
| `backend/crates/core/benches/calibration.rs` | `calibration` | runner factor |
| `backend/crates/core/benches/money.rs` | `money`, `preview` | NFR-SPD-7 budgets |
| `backend/crates/core/benches/text.rs` | `text`, `index/keys` | NFR-SPD-6 and NFR-SPD-1 budgets |
| `backend/crates/services/benches/search.rs` | `search`, `index/rebuild` | NFR-SPD-6 |
| `backend/crates/services/benches/rules.rs` | `rules` | NFR-SCL-6 |
| `backend/crates/services/benches/accounts.rs` | `accounts`, `coa`, `assist`, `ledger`, `vouchers` | NFR-SPD-3, 9, 10, 11, 13, 21 |
| `backend/crates/services/benches/translit.rs` | `translit` | NFR-SPD-20 |

Each service benchmark calls the same service function the screen calls (`search::search_accounts`, `accounts::list`, `assist::party_card`, `ledger::trial_balance`, `vouchers::save_advance`, `translit::suggest`), on the signed-in Owner's context, so the number includes the database, the checks and the audit row. It is not a number for an inner helper.

### 7.2 The data (NFR-SCL-1)

The generator is `backend/crates/services/benches/support/mod.rs`. It lives next to the Rust benchmarks, not in this folder, because it must use the real schema, indexes and triggers (every entry is first a draft and one `UPDATE` makes it posted, so the balance trigger checks all of them) and the real start-up path for the Owner and the system accounts.

It builds once and keeps the file in `backend/target/perf/nfr_scl1/` (set `MANDI_PERF_DIR` to use another folder). The next run opens it again. A marker file `volume.ok` holds the generator version and the entry count; if either changes, the folder is deleted and built again. Delete the folder by hand to force a rebuild. `MANDI_PERF_BUILD_ONLY=1 cargo bench -p mandi-services --bench search` builds the data and stops.

What is in it:

| Part | Count |
|---|---|
| Accounts | 20,000: 8,999 sellers (0010 to 0018998), 8,999 buyers (0020 to 0028998), 2,002 others (0103) |
| Masters | 2,000: 400 cities, 500 items, 100 packings, 300 addas, 100 stores, 200 expense heads, 400 staff |
| Posted journal entries | 1,030,000, dated 11-10-2016 to 10-10-2026 (10 years) |
| Journal lines | 3,503,940 |
| Data file | about 670 MB |
| Skewed accounts | the cash account has about 514,000 lines; the busiest buyer (0020) about 55,000; the busiest seller (0010) about 49,000; an ordinary buyer (0024499) about 90 |

The skew is on purpose. A shop has a few parties that trade every day and the cash account is touched by every cash sale, so the worst case of a balance, a party card and a ledger is a busy account, not an average one. Benchmarks report the busiest and an ordinary account side by side.

Not in it: the 1,200,000 sale lines of NFR-SCL-1 (`bikri_sales`, `analytics_facts`). No D1 or D2 query reads those tables. D3 adds them to the generator when the first بکری benchmark needs them; the version string is then raised so the file is built again.

### 7.3 How to read the output

```
Action | NFR | Measured | Stat | Mean | Runner threshold | NFR target | Result
```

- **Measured** is the 95th percentile (p95) of the per-sample time per call, by nearest rank (Section 4). **Mean** is criterion's mean, for comparison.
- **Runner threshold** is the NFR target times the runner factor (1.0). A row is `OK` when measured is at or under it and `OVER` when it is above.
- `MISSING` means a registered benchmark did not run. `NO THRESHOLD` means a benchmark ran that has no entry in `thresholds.json`. Both fail the run.
- The exit code is 0 (all rows OK or SKIPPED), 1 (any row not OK) or 2 (the harness itself failed).
- **The Mac is faster than the reference machine.** The developer Mac (Apple M1 Pro, PassMark single-thread 3,752) is about 1.65 times faster than the reference Core i3-7100 (2,268), and the GitHub Windows runner is about 1.11 times faster (Section 2). A row that passes on the Mac with less than 1.65 times of room may still fail on the reference computer. For a close row, multiply the Mac number by 3,752 ÷ 2,268 = 1.65 to estimate the reference time.
- The p95 of a flat sample of a few calls is close to the p95 of one call, but a slow call that happens once in a hundred (a checkpoint, a cache miss) shows in the Mean more than in the p95. When the two differ a lot (for example 642 ms mean and 1.94 s p95), the first call after a quiet moment is much slower than the next ones: the cold file cache. A user sees the cold number the first time he opens a busy party in the morning.

### 7.4 Frontend timing

`frontend/tests/robustness/key-latency.test.tsx` measures key press to character (NFR-SPD-4) and arrow key to focus (NFR-SPD-5) in jsdom, for the money and quantity fields and for the entry grid with 30 and 100 lines. It prints one line per case (`key-latency: ... p95 ... ms`). jsdom has no painting, so the numbers are the page's own code only: a guard against a slow handler or a render of too many rows, not proof of the NFR. The arrow-move guard is 100 ms (about three times the target) because AG Grid's focus code costs 13 to 17 ms per move in jsdom on a quiet machine and up to 45 ms on a busy one; it catches a slowdown of an order of magnitude. The real number comes from the Windows end-to-end suite. Typical quiet-machine values (12-10-2026): money and quantity fields p95 2 to 3 ms, grid cell typing p95 7 to 9 ms with 30 or 100 lines, arrow move p95 15 to 23 ms.


## 8. Results of the D2 speed runs

Two full runs are kept here. 8.1 is the current one, taken after backend's migration 0002 (covering index for balances, 15-10-2026). 8.2 is the first full run (12-10-2026), kept as history because it is where the eight slow rows were found.

### 8.1 Run of 15-10-2026, after migration 0002 (current)

Command: `pnpm perf` from the repository root. Exit code **0** ("perf: PASS"): no row OVER, install row SKIPPED (Windows only). Data: the NFR-SCL-1 volume of Section 7.2 (20,000 accounts, 3,503,940 journal lines), upgraded to schema version 2 before the run. Runner factor **1.0**, so every runner threshold equals its NFR target. Calibration row: 135.1 us against the expected 228 us, so the factor check is OK on this machine.

Machine: Apple M1 Pro, 8 logical CPUs, 16 GB. Load average (1 minute) 5.68 before the run and 6.15 after, on 8 CPUs, so the machine was not oversubscribed. (It reached 16.7 for about a minute while cargo built the benchmarks, before any timing started.) The reviewer had been told not to run cargo or pnpm until this run ended. No row was over target, so no row was rerun.

| Action | NFR | p95 | Mean | NFR target | Estimate on the reference computer (x 1.65) | Result |
|---|---|---|---|---|---|---|
| Calibration (fixed CPU workload) | factor check | 135.1 us | 135.1 us | - | - | OK |
| apply_bps, one percent charge | NFR-SPD-7 (budget) | 2.5 ns | 2.4 ns | 1.0 us | 4.1 ns | OK |
| parse_rupees, one money text | NFR-SPD-7 (budget) | 255.8 ns | 156.5 ns | 2.0 us | 422.1 ns | OK |
| split_by_bps, 3 partners | NFR-SPD-7 (budget) | 82.8 ns | 82.2 ns | 5.0 us | 136.6 ns | OK |
| split_by_bps, 10 partners | NFR-SPD-7 (budget) | 248.3 ns | 244.7 ns | 10.0 us | 409.7 ns | OK |
| Preview money maths, 500 lines | NFR-SPD-7 (budget) | 253.2 us | 248.5 us | 10.00 ms | 417.8 us | OK |
| normalize_urdu, one name | NFR-SPD-6 (budget) | 666.7 ns | 651.0 ns | 5.0 us | 1.1 us | OK |
| Three search keys, one name | NFR-SPD-6 (budget) | 1.3 us | 1.3 us | 10.0 us | 2.2 us | OK |
| Three search keys, 100-char name | NFR-SPD-6 (budget) | 4.9 us | 4.9 us | 30.0 us | 8.1 us | OK |
| Search keys for 20,000 accounts | NFR-SPD-1 (budget) | 22.89 ms | 22.57 ms | 200.00 ms | 37.77 ms | OK |
| Picker search, English name, exact start | NFR-SPD-6 | 843.8 us | 717.7 us | 50.00 ms | 1.39 ms | OK |
| Picker search, English name, 1 spelling mistake | NFR-SPD-6 | 3.42 ms | 2.98 ms | 50.00 ms | 5.65 ms | OK |
| Picker search, English name, 2 spelling mistakes | NFR-SPD-6 | 4.26 ms | 3.86 ms | 50.00 ms | 7.03 ms | OK |
| Picker search, Urdu name, exact start | NFR-SPD-6 | 1.10 ms | 734.1 us | 50.00 ms | 1.81 ms | OK |
| Picker search, Urdu name, 1 spelling mistake | NFR-SPD-6 | 6.54 ms | 5.78 ms | 50.00 ms | 10.80 ms | OK |
| Picker search, Urdu name, 2 spelling mistakes | NFR-SPD-6 | 6.47 ms | 5.57 ms | 50.00 ms | 10.68 ms | OK |
| Picker search, text that matches nothing | NFR-SPD-6 | 5.76 ms | 5.25 ms | 50.00 ms | 9.50 ms | OK |
| Picker search, 1 letter typed | NFR-SPD-6 | 1.51 ms | 1.07 ms | 50.00 ms | 2.49 ms | OK |
| Picker search, 3 letters typed | NFR-SPD-6 | 1.32 ms | 957.1 us | 50.00 ms | 2.18 ms | OK |
| Buyer picker search, English name, 1 mistake | NFR-SPD-6 | 4.44 ms | 2.82 ms | 50.00 ms | 7.33 ms | OK |
| Item picker search with a mistake (500 items) | NFR-SPD-6 | 19.7 us | 17.9 us | 50.00 ms | 32.6 us | OK |
| Search index rebuild, 20,000 accounts and 2,000 masters | NFR-SPD-1 (budget) | 90.09 ms | 84.52 ms | 200.00 ms | 148.66 ms | OK |
| Rule resolution, 10,000 rules spread over parties and items | NFR-SCL-6 | 255.0 ns | 245.8 ns | 1.00 ms | 420.7 ns | OK |
| Rule resolution, 10,000 rules for one party and item (worst case) | NFR-SCL-6 | 10.5 us | 9.6 us | 1.00 ms | 17.4 us | OK |
| Rule resolution, 10,000 dated versions of one default | NFR-SCL-6 | 9.9 us | 9.6 us | 1.00 ms | 16.4 us | OK |
| Rule resolution, no rule at any of the six levels | NFR-SCL-6 | 51.6 ns | 49.5 ns | 1.00 ms | 85.1 ns | OK |
| Filing 10,000 rules again after a rule change | NFR-SPD-7 (budget) | 1.69 ms | 890.2 us | 50.00 ms | 2.79 ms | OK |
| Account list, every account with its balance | NFR-SPD-3 | 283.97 ms | 268.85 ms | 300.00 ms | 468.56 ms | OK |
| Account list, first page of 200 | NFR-SPD-3 | 56.18 ms | 54.11 ms | 300.00 ms | 92.70 ms | OK |
| Account list filtered by text (about 2,000 matches) | NFR-SPD-3 | 34.24 ms | 23.10 ms | 300.00 ms | 56.50 ms | OK |
| Chart of accounts tree with every balance | NFR-SPD-3 | 254.87 ms | 237.79 ms | 300.00 ms | 420.54 ms | OK |
| Party card, the busiest buyer (about 55,000 lines) | NFR-SPD-21 | 28.47 ms | 7.65 ms | 100.00 ms | 46.97 ms | OK |
| Party card, the busiest seller (about 49,000 lines) | NFR-SPD-21 | 5.55 ms | 5.00 ms | 100.00 ms | 9.16 ms | OK |
| Party card, an ordinary buyer (about 90 lines) | NFR-SPD-21 | 120.7 us | 105.7 us | 100.00 ms | 199.1 us | OK |
| Party card, the cash account (about 514,000 lines) | NFR-SPD-21 | 44.76 ms | 43.17 ms | 100.00 ms | 73.85 ms | OK |
| Party card, 43 accounts in turn | NFR-SPD-21 | 557.6 us | 476.2 us | 100.00 ms | 920.0 us | OK |
| Party ledger of the busiest buyer, 10 years | NFR-SPD-11 | 189.84 ms | 170.44 ms | 2.00 s | 313.24 ms | OK |
| Party ledger of an ordinary buyer, 10 years | NFR-SPD-11 | 156.4 us | 151.7 us | 2.00 s | 258.1 us | OK |
| Cash book of one day (opening = all before) | NFR-SPD-10 | 386.15 ms | 256.36 ms | 1.00 s | 637.14 ms | OK |
| Trial balance, 10 years | NFR-SPD-13 | 440.48 ms | 300.68 ms | 2.00 s | 726.79 ms | OK |
| Save an advance voucher for the busiest seller | NFR-SPD-9 | 12.01 ms | 11.29 ms | 150.00 ms | 19.81 ms | OK |
| Save an advance voucher for an ordinary seller | NFR-SPD-9 | 3.78 ms | 3.43 ms | 150.00 ms | 6.23 ms | OK |
| Name suggestions per key press, English letters to Urdu | NFR-SPD-20 | 545.6 us | 353.4 us | 50.00 ms | 900.3 us | OK |
| Name suggestions for a whole typed name, English letters to Urdu | NFR-SPD-20 | 878.4 us | 616.3 us | 50.00 ms | 1.45 ms | OK |
| Name suggestions per key press, Urdu letters to English | NFR-SPD-20 | 13.09 ms | 5.92 ms | 50.00 ms | 21.61 ms | OK |
| Install (NSIS, silent) | NFR-INS-1 | - | - | 120.00 s | - | SKIPPED (Windows only) |

Reading the table:

- The last column multiplies the Mac time by 3,752 / 2,268 = 1.65 (Section 7.3). It is an estimate, not a measurement. The reference computer is the NFR 0.2 minimum (Core i3-7100).
- Nothing is OVER on this machine. Before migration 0002, eight rows were OVER (8.2). Now the party card is 0.1 to 45 ms against 100 ms, the first page of 200 accounts 56 ms against 300 ms, the trial balance 440 ms against 2 s, and saving an advance for the busiest seller 12 ms against 150 ms.
- **Two rows are at the target, not under it with room.** They need the balance of every account, which is one pass over all 3.5 million index entries (about 220 ms of CPU on this Mac).
  - Account list, every account: 284 ms against 300 ms (95% of the target). Estimate on the reference computer 469 ms. Estimate on a GitHub Windows runner (EPYC 7763, PassMark 2,517; Mac 3,752 / 2,517 = 1.49) about 423 ms. The accounts screen asks for every account with `limit 0` (`frontend/src/shared/accounts/api.ts`, `LIST_PAGE_LIMIT`), so this is the real path of the انڈکس screen at 20,000 accounts.
  - Chart of accounts tree: 255 ms against 300 ms (85%). Estimate on the reference computer 421 ms, on the runner about 380 ms.
  - Both are under their runner threshold on this Mac, but by less than the 1.49 to 1.65 that the slower machines need. The first Windows run of `pnpm perf` will show the real numbers. If either row reads OVER there, it is a real miss of NFR-SPD-3 at the NFR-SCL-1 volume.
  - At the shop's real volume (about 350,000 journal lines, one tenth), the same pass is about 22 ms.
- Run 8.1 is one quiet run. A second quiet run was not taken because the machine was shared again straight after.

### 8.2 First full run of 12-10-2026 (history, before migration 0002)


Command: `pnpm perf` from the repository root (cargo bench of the whole workspace, then the table). The data is the NFR-SCL-1 volume of Section 7.2. Runner factor 1.0, so every threshold is the NFR target. The machine is the developer Mac, which is about 1.65 times faster than the reference computer (Section 7.3): multiply a Mac time by 1.65 to estimate the reference time.

Machine: Apple M1 Pro, 8 logical CPUs, 16 GB. Runner factor 1.

| Action | NFR | p95 | Mean | NFR target | Result |
|---|---|---|---|---|---|
| Calibration (fixed CPU workload) | factor check | 235.2 us | 235.2 us | - | CHECK FACTOR |
| apply_bps, one percent charge | NFR-SPD-7 (budget) | 2.5 ns | 2.4 ns | 1.0 us | OK |
| parse_rupees, one money text | NFR-SPD-7 (budget) | 258.3 ns | 166.5 ns | 2.0 us | OK |
| split_by_bps, 3 partners | NFR-SPD-7 (budget) | 119.5 ns | 89.8 ns | 5.0 us | OK |
| split_by_bps, 10 partners | NFR-SPD-7 (budget) | 254.4 ns | 248.5 ns | 10.0 us | OK |
| Preview money maths, 500 lines | NFR-SPD-7 (budget) | 271.4 us | 254.3 us | 10.00 ms | OK |
| normalize_urdu, one name | NFR-SPD-6 (budget) | 671.2 ns | 659.8 ns | 5.0 us | OK |
| Three search keys, one name | NFR-SPD-6 (budget) | 1.3 us | 1.3 us | 10.0 us | OK |
| Three search keys, 100-char name | NFR-SPD-6 (budget) | 5.3 us | 5.0 us | 30.0 us | OK |
| Search keys for 20,000 accounts | NFR-SPD-1 (budget) | 22.78 ms | 22.57 ms | 200.00 ms | OK |
| Picker search, English name, exact start | NFR-SPD-6 | 713.4 us | 681.6 us | 50.00 ms | OK |
| Picker search, English name, 1 spelling mistake | NFR-SPD-6 | 3.30 ms | 2.92 ms | 50.00 ms | OK |
| Picker search, English name, 2 spelling mistakes | NFR-SPD-6 | 4.19 ms | 3.88 ms | 50.00 ms | OK |
| Picker search, Urdu name, exact start | NFR-SPD-6 | 559.2 us | 551.2 us | 50.00 ms | OK |
| Picker search, Urdu name, 1 spelling mistake | NFR-SPD-6 | 5.22 ms | 5.01 ms | 50.00 ms | OK |
| Picker search, Urdu name, 2 spelling mistakes | NFR-SPD-6 | 7.07 ms | 5.58 ms | 50.00 ms | OK |
| Picker search, text that matches nothing | NFR-SPD-6 | 5.11 ms | 4.95 ms | 50.00 ms | OK |
| Picker search, 1 letter typed | NFR-SPD-6 | 803.4 us | 780.5 us | 50.00 ms | OK |
| Picker search, 3 letters typed | NFR-SPD-6 | 813.2 us | 775.5 us | 50.00 ms | OK |
| Buyer picker search, English name, 1 mistake | NFR-SPD-6 | 2.70 ms | 2.63 ms | 50.00 ms | OK |
| Item picker search with a mistake (500 items) | NFR-SPD-6 | 16.0 us | 15.7 us | 50.00 ms | OK |
| Search index rebuild, 20,000 accounts and 2,000 masters | NFR-SPD-1 (budget) | 76.21 ms | 75.10 ms | 200.00 ms | OK |
| Rule resolution, 10,000 rules spread over parties and items | NFR-SCL-6 | 245.4 ns | 241.2 ns | 1.00 ms | OK |
| Rule resolution, 10,000 rules for one party and item (worst case) | NFR-SCL-6 | 10.1 us | 9.3 us | 1.00 ms | OK |
| Rule resolution, 10,000 dated versions of one default | NFR-SCL-6 | 9.6 us | 9.5 us | 1.00 ms | OK |
| Rule resolution, no rule at any of the six levels | NFR-SCL-6 | 53.7 ns | 50.5 ns | 1.00 ms | OK |
| Filing 10,000 rules again after a rule change | NFR-SPD-7 (budget) | 1.95 ms | 926.9 us | 50.00 ms | OK |
| Account list, every account with its balance | NFR-SPD-3 | 2.73 s | 1.92 s | 300.00 ms | OVER |
| Account list, first page of 200 | NFR-SPD-3 | 318.79 ms | 291.90 ms | 300.00 ms | OVER |
| Account list filtered by text (about 2,000 matches) | NFR-SPD-3 | 412.28 ms | 253.73 ms | 300.00 ms | OVER |
| Chart of accounts tree with every balance | NFR-SPD-3 | 1.81 s | 1.63 s | 300.00 ms | OVER |
| Party card, the busiest buyer (about 55,000 lines) | NFR-SPD-21 | 241.25 ms | 159.65 ms | 100.00 ms | OVER |
| Party card, the busiest seller (about 49,000 lines) | NFR-SPD-21 | 228.29 ms | 182.61 ms | 100.00 ms | OVER |
| Party card, an ordinary buyer (about 90 lines) | NFR-SPD-21 | 146.7 us | 134.6 us | 100.00 ms | OK |
| Party card, the cash account (about 514,000 lines) | NFR-SPD-21 | 525.99 ms | 448.13 ms | 100.00 ms | OVER |
| Party card, 43 accounts in turn | NFR-SPD-21 | 33.79 ms | 8.17 ms | 100.00 ms | OK |
| Party ledger of the busiest buyer, 10 years | NFR-SPD-11 | 218.25 ms | 188.28 ms | 2.00 s | OK |
| Party ledger of an ordinary buyer, 10 years | NFR-SPD-11 | 144.2 us | 139.8 us | 2.00 s | OK |
| Cash book of one day (opening = all before) | NFR-SPD-10 | 782.42 ms | 629.08 ms | 1.00 s | OK |
| Trial balance, 10 years | NFR-SPD-13 | 1.97 s | 1.64 s | 2.00 s | OK |
| Save an advance voucher for the busiest seller | NFR-SPD-9 | 201.17 ms | 174.33 ms | 150.00 ms | OVER |
| Save an advance voucher for an ordinary seller | NFR-SPD-9 | 4.94 ms | 4.48 ms | 150.00 ms | OK |
| Name suggestions per key press, English letters to Urdu | NFR-SPD-20 | 669.1 us | 390.8 us | 50.00 ms | OK |
| Name suggestions for a whole typed name, English letters to Urdu | NFR-SPD-20 | 809.8 us | 578.1 us | 50.00 ms | OK |
| Name suggestions per key press, Urdu letters to English | NFR-SPD-20 | 13.26 ms | 5.95 ms | 50.00 ms | OK |
| Install (NSIS, silent) | NFR-INS-1 | - | - | 120.00 s | SKIPPED |

`pnpm perf` exit code 1: 8 rows OVER (the first run had 9). Everything else is OK, most of it by a wide margin: picker search 0.6 to 7 ms against 50 ms, rule resolution 0.05 to 10 us against 1 ms, name suggestions 0.4 to 13 ms against 50 ms, party ledger 218 ms against 2 s, voucher save for an ordinary seller 5 ms against 150 ms.

#### 8.2.1 The rows that were OVER

All eight come from one cause. A balance is worked out when it is asked for: `SUM` over every journal line of the account, joined one line at a time to `journal_entries` to check that the entry is posted (`backend/crates/db/src/ledger.rs`, `totals_for_accounts`; `EXPLAIN QUERY PLAN`: `SEARCH l USING INDEX idx_journal_lines_account_entry (account_id=?)`, then `SEARCH e USING INTEGER PRIMARY KEY (rowid=?)` for each line). The cost grows with the number of lines of the account. Measured with the `sqlite3` tool on the same file, warm cache, CPU time: the busiest buyer (55,658 lines) 105 ms for the balance and 56 ms more for the last entry date; the cash account (513,970 lines) 360 ms for the balance. The party card does both (and a third scan for a seller's open advance), the account list and the chart tree do the balance for every account on the page, and the trial balance does it for all of them.

| Row | Run 1 p95 | Run 2 p95 | Target | On the reference computer (x 1.65, run 2) |
|---|---|---|---|---|
| Party card, busiest buyer | 1.94 s | 241 ms | 100 ms | about 400 ms |
| Party card, busiest seller | 599 ms | 228 ms | 100 ms | about 380 ms |
| Party card, cash account | 1.20 s | 526 ms | 100 ms | about 870 ms |
| Account list, first page of 200 | 442 ms | 319 ms | 300 ms | about 530 ms |
| Account list, filtered by text | 302 ms | 412 ms | 300 ms | about 680 ms |
| Account list, every account | 2.10 s | 2.73 s | 300 ms | about 4.5 s |
| Chart of accounts tree | 2.08 s | 1.81 s | 300 ms | about 3.0 s |
| Save an advance, busiest seller | 239 ms | 201 ms | 150 ms | about 330 ms |

Trial balance, 10 years: run 1 2.51 s (OVER), run 2 1.97 s (OK by 30 ms). On the reference computer it would be about 3.2 s. Treat it as OVER.

Run 1 and run 2 differ because the Mac was shared. Other builds and test runs were using it (load average 15.9 on 8 cores when checked), and run 2's calibration row also reads `CHECK FACTOR` (235 us against the expected 228 us; run 1 read 135 us). Run 1 also had a colder file cache: the first call after a quiet moment is much slower (the 642 ms mean against a 160 ms mean for the busiest buyer). The CPU-time measurement above does not depend on the load, and it agrees with run 2. The numbers on the Windows runner are the ones that count (Release_Pipeline.md Section 4).

What the NFR says: NFR-SPD-21 (party card under 100 ms with 10 years of data), NFR-SPD-3 (a screen opens under 300 ms), NFR-SPD-9 (a voucher saves under 150 ms), NFR-SPD-13 (trial balance under 2 s). NFR-SPD-2 (login to dashboard under 1 s) is not measured yet, but the dashboard's cash tile reads the balance of the cash account, which costs 360 ms to 530 ms here.

A way to fix it, for `backend` to decide: keep a running balance per account (and per پختہ کھاتہ person) in a table that the posting writes in the same transaction (Code Standards B20), so a balance, a card, a list page, the tree and the trial balance read one row each. The books stay the source of truth: the daily check (NFR-REL-8) compares the table with the sum of the lines.
