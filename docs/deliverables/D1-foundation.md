# D1. Foundation: the app installs, starts and signs in

Version 1.0, 10-10-2026. Deliverable 1 of 5. Read `README.md` in this folder first.
Depends on: nothing. Plan files: `01-scaffold`, `02-core`, `03-database`, `04-shell-and-login`.
Build order steps: CLAUDE.md MVP steps 1, 2 and 3, plus the app shell.

## Decisions

| ID | Question | Recommended | Umar's answer |
|---|---|---|---|
| D1-Q1 | Which tool takes and compares screenshots in the Windows suite? | WebdriverIO visual service (`@wdio/visual-service`), because the suite already runs on WebdriverIO | (waiting) |

Until D1-Q1 is answered, plan file 01 builds the suite without visual comparison and saves plain screenshots. Comparison is added when the answer comes.

## 1. What Umar can do at the end

He downloads the installer from the GitHub Releases page and installs it on a Windows computer with no internet. The app opens in under 2 seconds. The first-run wizard, in Urdu, asks for the firm details and the Owner password, then shows the recovery code to print. He logs in. He sees the dashboard with a friendly empty state, the main menu on the right, the top bar, and the status bar. He presses `Alt+L`: everything turns to English and mirrors to the left, instantly. He moves through every menu with the arrow keys only. Settings holds the firm details, the auto-lock time, the language and the text size. Five wrong passwords lock the login for one minute, and the audit log shows every try.

Behind that: the repository, the checks, the release pipeline, the money core with its proofs, and the full database are in place.

## 2. Plan files

### 01-scaffold: repository, tooling and pipeline

**Requirements:** Tech Stack 2, 3, 5 and 7; Code Standards 2, 10; NFR-MNT-1, NFR-MNT-2, NFR-SCL-7, NFR-SCL-9, NFR-INS-1, NFR-SEC-9, NFR-SEC-10, NFR-REL-9, NFR-URD-1 to NFR-URD-3; `docs/Release_Pipeline.md` 3.3.

**Who owns the root files.** The ownership table in CLAUDE.md covers `backend/` and `frontend/`. For the files at the root, the lead writes this into the plan file:

| Files | Owner |
|---|---|
| `package.json`, `pnpm-workspace.yaml`, ESLint and Prettier configs, `.editorconfig` | `frontend` |
| `rust-toolchain.toml`, `backend/deny.toml`, `scripts/seed` | `backend` |
| `.github/`, `README.md`, `docs/` | lead |
| `scripts/perf/` | `robustness` |

**backend builds:**

- The Cargo workspace in `backend/`, laid out so a new module is a new folder (NFR-SCL-7) and nothing blocks a later multi-computer option (NFR-SCL-9), with the crates `core`, `domain`, `db`, `services`, `reports`, `backup` and `app`, wired in the dependency direction of CLAUDE.md rule 6. Each crate has a short README (NFR-MNT-6).
- `tauri.conf.json` exactly as `docs/Release_Pipeline.md` 3.3, with the public key from `docs/release/updater-pubkey.txt` and identifier `pk.mandisoftware.app`.
- Only the Tauri plugins in Tech Stack 2. Single instance: opening the app twice brings the first window forward (NFR-REL-9).
- WebView2 settings from Tech Stack 7: browser keys off, browser zoom off, default context menu off.
- A strict Content Security Policy with no remote fonts or scripts. A capability file that allows only our listed commands (NFR-SEC-10).
- Clippy settings that deny `unwrap`, `expect`, `panic!` and `unsafe` outside tests (CLAUDE.md rule 5). `cargo deny` config.

**frontend builds:**

- React + TypeScript + Vite in `frontend/`, with the libraries from Tech Stack 3 pinned.
- Fluent set up for `ur` and `en`, with `glossary.ftl` taken from PRD Section 3 as the start of the locked glossary (NFR-URD-1, NFR-URD-2). Machine translation is never used for final text (NFR-URD-3). `pnpm i18n:check` fails on a missing or empty key (NFR-URD-22).
- ESLint rules that fail the build on: physical CSS directions (`left`, `right`, `margin-left` and the like; CLAUDE.md rule 8), literal text in JSX (rule 7), and money arithmetic in the frontend (rule 1, for example a `+` or `*` on a value typed as money).
- Every script named in CLAUDE.md "Commands", plus `tauri` and `e2e`, in the root `package.json`, with `packageManager` pinned.
- The Windows end-to-end suite: WebdriverIO + `tauri-driver`. First test: the app window opens, with screenshots in Urdu and English at 1024 × 600, 1366 × 768 and 1920 × 1080, saved to `frontend/tests/e2e/output/`. (The login screen comes in 04.)

**robustness builds:** the `pnpm perf` harness in `scripts/perf/` and `backend/crates/*/benches/` (criterion for backend functions; app timings come from the E2E suite). It measures the runner factor and writes `scripts/perf/README.md` (README Section 6). On Windows, `pnpm perf` also installs the NSIS file built earlier in the same run silently and records the install time (NFR-INS-1), so the workflows need no change.

**Lead:** checks that the three workflows in `.github/workflows/` call the script names exactly. They need no edit if the names match.

**Done when:** `pnpm lint`, `pnpm test`, `pnpm i18n:check`, `pnpm e2e` and `pnpm perf` all run. Both checks pass on the pull request. The merge publishes the first release with a signed `latest.json`.

### 02-core: money, dates, text, words and validation

**Requirements:** PRD 22 and FR-6.4; Code Standards 3, 4.2 and 9.3; the money-calculations agent; NFR-MNT-3, NFR-MNT-4; NFR-URD-14 to NFR-URD-16, NFR-URD-18, NFR-URD-19, NFR-URD-23.

This plan file is backend only, plus one small display helper in the frontend. `backend` calls the money-calculations agent before writing `core::money` and again to review it.

**backend builds in `backend/crates/core/`:**

- `money`: `Paisa(i64)`, always a multiple of 100. Parse from the text the user typed (Code Standards 4.2: no decimals, no negatives where not allowed, no empty, no too-large, the right digits). `round_to_rupee` (half up), `split_by_bps` (floor each share, leftover to the largest share), `split_by_qty`. Checked arithmetic only.
- `dates`: `DD-MM-YYYY` parse and format; the helpers for the open-day model of FR-6.6 (a date is after "closed up to" and not after today).
- `text`: one normaliser for search keys. Look-alike letters count as the same (Arabic ي and Urdu ی, Arabic ك and Urdu ک, and the others in NFR-URD-18). Urdu alphabetical sort order (NFR-URD-19).
- `urdu_words` and `english_words`: amounts in words for every whole-rupee amount in the range NFR-URD-15 and NFR-URD-23 give, with lakh and crore grouping.
- `validate`: the shared field checks of Code Standards 4.2, returning error codes with field paths.
- Unit tests, and property tests with `proptest` for money (Code Standards 9.3), at least 10,000 cases each. 100% coverage of `core::money`.

**Worked numbers the tests must contain** (from PRD 4.4 and E2E-4):

- `split_by_bps(3,948, [4000, 3000, 3000])` gives 1,580 / 1,184 / 1,184. Floors: 3,948 × 40 ÷ 100 = 1,579.2, so 1,579; 3,948 × 30 ÷ 100 = 1,184.4, so 1,184 and 1,184. Sum 3,947, leftover 1 goes to the largest share, so 1,580.
- 19,316 in words: "انیس ہزار تین سو سولہ روپے" and "Nineteen thousand three hundred sixteen rupees only".

**frontend builds:** `shared/format`: `formatPaisa` for display only (thousands separators, the Urdu-digits setting of NFR-URD-14). It never does arithmetic. While `backend` works on `core`, `frontend` also builds the design system tokens and base components from `docs/design/design-system.md` (Section 3), so 04 can start on screens at once.

**Done when:** every test above passes and the money agent's review says PASS.

### 03-database: schema, seed, audit and logs

**Requirements:** Architecture 15 and 16; Code Standards 5.3, 7, 4.3 and 8; FR-10.4; FR-21.3 (writing audit rows); NFR-REL-1, NFR-REL-6, NFR-REL-7, NFR-REL-10, NFR-REL-13; NFR-SEC-5 to NFR-SEC-7; NFR-OBS-1 to NFR-OBS-3; NFR-SCL-8.

**backend builds:**

- Migration `0001` with **every** table of Architecture Section 15, with its columns, keys, checks and indexes. Money columns are `INTEGER *_paisa`.
- The migration runner. Before any migration on an existing data file, it takes a full copy (NFR-REL-13).
- SQLite settings from Code Standards 7 (for example WAL and foreign keys on).
- The transaction wrapper used by every service: one user action, one transaction (CLAUDE.md rule 3). It checks free disk space before writing (NFR-REL-10).
- The quick data check at start-up, run in the background (NFR-REL-7).
- The seed of system accounts from FR-10.4: codes, Urdu names and English names exactly as the table.
- The audit writer: every audit row carries a keyed hash (NFR-SEC-6). The key lives in Windows Credential Manager (NFR-SEC-7).
- The error model of Architecture 16: `ERR_...` codes mapped to Fluent `err-...` messages, each with a reference number that matches a log line (NFR-OBS-3). No error ever closes the app (NFR-REL-6).
- The log file: 30 days, at most 50 MB, never passwords, keys or balances (NFR-OBS-1, NFR-OBS-2).
- `pnpm seed` creates an empty dev database with the seeded accounts. (The PRD 4.3 opening position is added to it in D2.)

**frontend builds:** the shared error helper. It maps each `ERR_...` code to its Fluent message and shows it under its field. Never the raw code (NFR-USE-14).

**Done when:** a migration test on an empty file and a re-run both pass, the seeded accounts match FR-10.4 row for row, and a tampered audit row is detected.

### 04-shell-and-login: sign-in, roles, first run and the app shell

**Requirements:** PRD 2 (FR-2.1 to FR-2.4), 5 (FR-5.1 to FR-5.3, FR-5.5, FR-5.6), 7.1 (FR-7.1 to FR-7.11), 7.2 and 7.3 (FR-7.13 to FR-7.20), 7.5 (FR-7.30 to FR-7.34, FR-7.37), 8 (FR-8.1 to FR-8.6) and the Settings parts of 21 for firm, users, idle lock and text size. NFR-INS-2 (firm, Owner, recovery code), NFR-SEC-1 to NFR-SEC-4, NFR-SEC-12, NFR-SPD-1 to NFR-SPD-5, NFR-SPD-19, NFR-USE-7, NFR-USE-8, NFR-USE-10 to NFR-USE-24, NFR-USE-28, NFR-URD-5 to NFR-URD-7, NFR-URD-9 to NFR-URD-13, NFR-URD-21, NFR-URD-27. Architecture 4, 5 and 18; Code Standards 6.

The worked example in `README.md` Section 4 shows how the team runs the login part.

**backend builds:**

- One user: the Owner, made by the first-run wizard (Umar, 11-10-2026: only the Owner uses the app, so there are no user or role screens and no add-user commands). Every command except first run, sign-in, recovery, app info and data status needs the signed-in Owner (the check is the first line of every command, NFR-SEC-4). The database keeps its `users` table and role column. One user logged in at a time (FR-2.3).
- Login: Argon2id (NFR-SEC-1), the lockout steps of FR-8.2, idle lock after 15 minutes as a setting (FR-8.4), the recovery code printed at first setup is the way back in (FR-8.5), audit rows for every login, failed login and logout (FR-8.6).
- First-run setup: firm details, Owner account, recovery code. The wizard has room for the partners step (added in D3) and the backup key step (added in D5).
- Settings commands for firm details, idle lock minutes, default language and text size.
- The "closed up to" date and today's day status, read for the status bar.

**frontend builds:**

- The rest of the design system in `shared/` (tokens and base components were started in 02): bundled fonts (Nastaliq for Urdu, Inter for English and numbers, Noto Naskh Arabic for Urdu digits; NFR-URD-9, NFR-URD-27), `Num`, `Code`, buttons that show their key (FR-7.4), success toasts (FR-7.7), the confirm dialog (FR-7.8), the "save changes?" prompt (FR-7.9), field errors (FR-7.6), empty states, and text size with `Ctrl +` and `Ctrl −` (FR-7.11, NFR-USE-18).
- `shared/keyboard` and `shared/grid`: the arrow-key rules of FR-7.13 to FR-7.20 and the keys of PRD 7.3, in both directions. The grid is proven on a development-only page (not in the shipped build), with the `entry-grid` skill. Every later grid uses it.
- The login screen with the language switch (FR-7.30), the first-run wizard, the shell (main menu on the start side, top bar with the search box and bell as placeholders, status bar), the بکری side menu frame (FR-5.3) with its items leading to "coming soon" empty states, the dashboard empty state, and Settings for firm, idle lock, language and text size.
- The language switch on every screen: instant, no reload, nothing typed is lost, keys follow the screen direction (FR-7.31 to FR-7.34).

**robustness** measures cold start, login to dashboard, screen open, arrow-key move and language switch through the E2E suite (NFR-SPD-1 to NFR-SPD-3, NFR-SPD-5, NFR-SPD-19).

**Done when:** the D1 story in Section 1 runs end to end on Windows, by keyboard only, in both languages.

## 3. Design for D1

Right after 01 is merged, the lead runs `/design` with the brief in `README.md` Section 5 for these screens, each in Urdu and English, with all four states:

1. Login, with the language switch and the lockout message.
2. First-run wizard: firm details, Owner password, recovery code.
3. App shell with the empty dashboard.
4. Settings: firm details, auto lock, language and text size.
5. A sample entry form and a sample entry grid, to fix the look of every later form and grid.

The lead writes `docs/design/design-system.md` from the canvas: colours and their one meaning each, type scale for Nastaliq and Inter, spacing grid, focus outline, button styles with their key labels, form field states, grid row heights (Nastaliq line height at least 1.8), toasts and dialogs. `frontend` builds the tokens and components from it before any screen.

## 4. Team flow for D1

- Plan files run in order: 01, 02, 03, 04 (README Section 4).
- 01 first. Nothing else starts until its pull request is merged, because every other plan file needs the scripts and checks. `robustness` is spawned in 01 for the perf harness, and `reviewer` and `robustness` give their verdicts on 01 like any plan file.
- The design step runs between 01 and 02.
- Merging in D1 happens before the `main` ruleset exists for at least the first pull request. Follow README Section 4, point 6: never `--auto` until the ruleset is active.

## 5. Review focus for D1

`reviewer` pays extra attention to:

- The rules that will be copied by every later feature: the command shape (Code Standards 5.6), the error model, the transaction wrapper, the lint rules, the keyboard and grid behaviour.
- `core::money` against PRD 22 and the money agent, line by line.
- The design system against NFR-USE-10 to NFR-USE-24 and the design canvas.
- Both languages on every screen; no hard-coded words; no physical CSS directions.

## 6. Robustness attack list for D1

1. Money core: property tests with extreme values (0, 1 rupee, `i64` limits); an overflow returns an error, never wraps or panics. Malformed text inputs from Code Standards 4.2 (decimals, `1,00,0`, Urdu digits, spaces, signs, empty).
2. Login: 1,000 wrong passwords in a row; a 10,000-character password; Urdu letters in the password; the clock moved back during a lockout (NFR-REL-11); the app killed during login.
3. Database: the app killed during a migration, then restarted (the copy is restored or the migration resumes cleanly); a full disk before a save (NFR-REL-10); a corrupted data file at start-up (NFR-REL-7 shows a clear message, never a crash).
4. Security: try to call a command that is not in the capability file from the frontend; call a command before signing in; check the log for passwords or keys; check the CSP blocks a remote script.
5. Speed on the Windows runner: NFR-SPD-1, NFR-SPD-2, NFR-SPD-3, NFR-SPD-5 and NFR-SPD-19. Report each measured value against its runner threshold and its NFR target.
6. Two copies of the app (NFR-REL-9). Running all day: open and switch screens for 30 minutes in a loop and watch memory (NFR-SPD-16).

## 7. Acceptance (D1-A1 to D1-A16)

| ID | Item | Evidence expected |
|---|---|---|
| D1-A1 | Repository layout matches Tech Stack 5; every CLAUDE.md command runs | Tree listing; each command and its exit code |
| D1-A2 | CI and Windows checks pass on `main`; Release published with a signed `latest.json` | Run IDs; the "Check the published update file" step output |
| D1-A3 | The installer installs in under 2 minutes on the Windows runner, with the offline WebView2 installer bundled (NFR-INS-1). A no-internet install is on Umar's go-live checklist. | Install time from `pnpm perf` on Windows; the bundle setting |
| D1-A4 | Lint blocks the CLAUDE.md rules 1, 5, 7 and 8 | A deliberate violation of each fails; output shown, then reverted |
| D1-A5 | Money property tests pass, at least 10,000 cases each; 100% coverage of `core::money` | Test and coverage output |
| D1-A6 | The split example gives 1,580 / 1,184 / 1,184 | Test name and result |
| D1-A7 | Amounts in words correct in Urdu and English, including 19,316, 0, 1, 1,00,000 and the top of the range | Test table output |
| D1-A8 | Look-alike letters match in search keys; Urdu sort order correct | Test output |
| D1-A9 | Migration 0001 creates every Architecture 15 table; re-run is safe; a copy is taken before migrating | Test output; table list compared with Architecture 15 |
| D1-A10 | Seeded accounts match FR-10.4 row for row | Test output |
| D1-A11 | Audit rows for login, failed login and logout, with a keyed hash; an edited row is detected | Test output |
| D1-A12 | FR-8.1 to FR-8.6 pass: lockout steps, idle lock keeps unsaved work, recovery code reset | Integration test output |
| D1-A13 | Before sign-in, every command except first run, sign-in, recovery, app info and data status refuses with `ERR_NOT_LOGGED_IN` (or `ERR_SESSION_LOCKED` while locked) and writes nothing | Test output |
| D1-A14 | First-run wizard and the D1 story of Section 1 pass by keyboard only, in Urdu and English, on Windows | Windows E2E names and results |
| D1-A15 | Runner factor recorded; NFR-SPD-1, NFR-SPD-2, NFR-SPD-3, NFR-SPD-5 and NFR-SPD-19 met at their runner thresholds (README Section 6) | `scripts/perf/README.md`; E2E timings against each threshold |
| D1-A16 | Design system written; screenshots in both languages at the three sizes match it; contrast check passes (NFR-USE-21); Urdu renders with no broken letters (NFR-URD-10) | Canvas link, screenshot file names, contrast check output, `windows-verify` report |

## 8. Not in D1

- Accounts, names in two scripts, search, party card, masters and rules: D2.
- Partners (and the wizard's partners step), بکری and printing: D3.
- Day close and reopen, and the first voucher (پیشگی): D2.
- Dashboard tiles, attention bell, reports, global search results, backup (and the wizard's backup key step), in-app update prompt: D5.
