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

- **Criterion benchmarks.** Criterion takes samples; each sample runs the code a number of times. The harness divides each sample's time by its run count and takes the 95th percentile of those values (nearest rank). That is the "p95" in the table. For a benchmark of one user action (milliseconds or more), set `group.sampling_mode(SamplingMode::Flat)` and a `sample_size` of at least 20, so each sample is a few single calls and the p95 is close to the p95 of one action. Each benchmark runs on a database built with the NFR-SCL-1 volume (1,200,000 sale lines, 3,500,000 journal lines, 20,000 accounts) from the generator in `scripts/perf/` (comes with the first benchmark that needs data, plan 03 or later).
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
| NFR-SPD-4  | Key press to character on screen                         | < 50 ms                                       | 50 ms                                                              | E2E timing: key event to DOM change, p95 of 100 keys, in a field and a grid cell                                   | 04, 11 (grid)                           | Pending |
| NFR-SPD-5  | Arrow move between cells or fields                       | < 30 ms                                       | 30 ms                                                              | E2E timing: key event to focus change, p95                                                                         | 04, 11                                  | Pending |
| NFR-SPD-6  | Picker search, 20,000 accounts, typos, both names        | < 50 ms                                       | 50 ms                                                              | Criterion: `services::search` on 20,000 generated accounts, p95; plus E2E                                          | D2 (search)                             | Pending |
| NFR-SPD-7  | Live summary after a sale line change (incl. 50 ms wait) | < 150 ms                                      | 150 ms                                                             | Criterion: `bikri_preview` with 30 lines, p95 must be under 100 ms (150 minus the 50 ms wait); plus E2E end to end | D3                                      | Pending |
| NFR-SPD-8  | Save a بکری with 30 sale lines                           | < 300 ms                                      | 300 ms                                                             | Criterion: `services::bikri::save` on the NFR-SCL-1 database, p95                                                  | D3                                      | Pending |
| NFR-SPD-9  | Save any voucher                                         | < 150 ms                                      | 150 ms                                                             | Criterion: each voucher save on the NFR-SCL-1 database, p95                                                        | D2 (پیشگی), D4                          | Pending |
| NFR-SPD-10 | Any report for one day                                   | < 1 s                                         | 1 s                                                                | Criterion: each report query for one busy day (5,000 sale lines), p95                                              | D5                                      | Pending |
| NFR-SPD-11 | Party ledger for 10 years                                | < 2 s                                         | 2 s                                                                | Criterion: busiest party, 10 years, p95                                                                            | D5                                      | Pending |
| NFR-SPD-12 | Any analytics view over 5 years                          | < 1 s SSD, < 3 s HDD                          | 1 s (SSD only)                                                     | Criterion: each view A to H over 5 years, p95. HDD on reference hardware only                                      | D5                                      | Pending |
| NFR-SPD-13 | Trial balance for 10 years                               | < 2 s                                         | 2 s                                                                | Criterion, p95                                                                                                     | D5                                      | Pending |
| NFR-SPD-14 | Print preview of a 15-page report                        | < 2 s                                         | 2 s                                                                | E2E timing: preview command to preview shown, p95                                                                  | D3 (first print), D5                    | Pending |
| NFR-SPD-15 | Local backup snapshot, 300 MB                            | < 5 s                                         | 5 s                                                                | Criterion or timed test in `mandi-backup` on a 300 MB database, p95 of 10                                          | D5                                      | Pending |
| NFR-SPD-16 | Memory in normal use                                     | < 400 MB                                      | 400 MB                                                             | Long run: working set sampled every minute                                                                         | 01 (D1 attack list, 30-minute loop), D5 | Pending |
| NFR-SPD-17 | Long tasks show progress, never freeze                   | Rule: progress shown; screen keeps responding | Same rule; key press during the task still under 50 ms (NFR-SPD-4) | E2E: start an export or restore, type during it                                                                    | D5                                      | Pending |
| NFR-SPD-18 | Lists with thousands of rows scroll smoothly             | Rule: only visible rows are drawn             | Same rule: rendered row count stays under 100 with 10,000 rows     | E2E: count rendered rows while scrolling                                                                           | D3 (first big grid), D5                 | Pending |
| NFR-SPD-19 | Language switch, incl. a بکری with 30 unsaved lines      | < 500 ms                                      | 500 ms                                                             | E2E timing: `Alt+L` to `dir` changed and text redrawn, p95                                                         | 01 (first screen), 04, D3 (بکری)        | Pending |
| NFR-SPD-20 | English-to-Urdu suggestions per key                      | < 50 ms                                       | 50 ms                                                              | Criterion: `translit_suggest` for the 1,000-name list, p95                                                         | D2                                      | Pending |
| NFR-SPD-21 | Party card, 10 years of data                             | < 100 ms                                      | 100 ms                                                             | Criterion: `assist_party_card` on the NFR-SCL-1 database, p95                                                      | D2                                      | Pending |
| NFR-SPD-22 | Attention list count after a save                        | within 1 s                                    | 1 s                                                                | Integration timing: save to updated count, p95                                                                     | D5                                      | Pending |
| NFR-SPD-23 | Rate hint, seller and truck memory                       | < 100 ms                                      | 100 ms                                                             | Criterion: each `assist_*` query on the NFR-SCL-1 database, p95                                                    | D3                                      | Pending |

Other measured targets:

| ID         | Action                                     | NFR target             | Runner threshold | How measured                                                                                           | Plan       | Status                       |
| ---------- | ------------------------------------------ | ---------------------- | ---------------- | ------------------------------------------------------------------------------------------------------ | ---------- | ---------------------------- |
| NFR-INS-1  | Install with no internet needed            | < 2 minutes            | 2 minutes        | This harness on Windows (silent NSIS install). The no-internet install is on Umar's go-live checklist. | 01         | Measured on each Windows run |
| NFR-SCL-6  | Find the winning rule, 10,000 rules        | < 1 ms                 | 1 ms             | Criterion: rule resolution, p95                                                                        | D2 (rules) | Pending                      |
| NFR-REL-12 | 12 hours with no slowdown or memory growth | No slowdown, no growth | Same             | Long run, simulated with many operations                                                               | D5         | Pending                      |

When a measurement is added, its plan file adds an entry to `thresholds.json` (`kind: "nfr"`, `nfr`, `action`, `stat: "p95"`, `target_ms`) and changes its Status here to "Measured".
