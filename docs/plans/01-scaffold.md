# Plan 01: Scaffold, tooling and pipeline

Deliverable: D1 (`docs/deliverables/D1-foundation.md`, plan file 01). Branch: `feature/scaffold-repo`.

## 1. Goal

Make the empty repository a working project. The Rust workspace and the React app build. Every script in CLAUDE.md "Commands" exists and runs. The lint rules block the CLAUDE.md rules that a tool can check. The Windows workflow builds the installers, runs the first end-to-end test with screenshots, and runs the speed harness. The first merge publishes the first signed release.

There is no business feature yet. The only command is `app_info`, so the whole chain (Rust type, ts-rs binding, `api.ts`, screen) is proven once.

## 2. Requirements

- Tech Stack 2, 3, 5 and 7. Code Standards 2 and 10. `.claude/rules/Release_Pipeline.md` 3.3.
- NFR-MNT-1, NFR-MNT-2, NFR-MNT-6, NFR-SCL-7, NFR-SCL-9, NFR-INS-1, NFR-SEC-9, NFR-SEC-10, NFR-REL-9, NFR-URD-1 to NFR-URD-3, NFR-URD-22.
- Acceptance items it feeds: D1-A1, D1-A2, D1-A3, D1-A4, D1-A15 (runner factor).

Where the documents live: the requirement documents are in `.claude/rules/` (Umar's choice). When any file says `docs/PRD_Mandi_Software_v1.md`, read `.claude/rules/PRD_Mandi_Software_v1.md`. The same for NFR, Tech Stack, Code Standards, Architecture and Release_Pipeline. Plans, reviews, design and release notes are in `docs/`.

## 3. Worked numbers

No money in this plan. (Money starts in 02.)

## 4. Contract

### 4.1 Command

| Command | Role | Input | Output | Errors |
|---|---|---|---|---|
| `app_info` | none (shown before login) | none | `AppInfo { version: string, identifier: string }` | none |

- Rust type `AppInfo` in `backend/app/src/commands/app.rs`, exported with ts-rs to `frontend/src/bindings/AppInfo.ts`.
- Frontend calls it only through `frontend/src/shared/api/` (`invoke` client) and a feature `api.ts`. Components never call `invoke()` (Code Standards F4).
- The capability file allows only `app_info` and the core events the shell needs. No shell, no general file system (NFR-SEC-10, Code Standards S4).

### 4.2 Error shape (fixed now so every later command copies it)

Every command returns `Result<T, UiError>`. `UiError` serialises as:

```json
{ "ok": false, "errors": [ { "code": "ERR_NOT_IMPLEMENTED", "field": null, "values": {} } ], "ref": "E-20261010-0001" }
```

- `code` is `ERR_` + SCREAMING_SNAKE_CASE. `field` is a path like `lots[0].sales[2].qty` or null. `values` is a map of strings or numbers.
- `ref` is a reference number that later matches a log line (NFR-OBS-3, built in 03). In 01 it may be a fixed format with a counter.
- `UiError` is exported with ts-rs as `UiError` and `UiErrorItem`.

### 4.3 Root files and their owners

| Files | Owner |
|---|---|
| `package.json`, `pnpm-workspace.yaml`, `pnpm-lock.yaml`, ESLint, Prettier and Stylelint configs, `.editorconfig`, `scripts/i18n-check.*`, `scripts/gen-types.*` | `frontend` |
| `rust-toolchain.toml`, `backend/**`, `backend/deny.toml`, `scripts/seed/` (03) | `backend` |
| `.github/`, `README.md`, `docs/` | lead |
| `scripts/perf/`, `backend/crates/*/benches/`, `backend/crates/*/tests/robustness*` | `robustness` |

## 5. Tasks

### T1 backend: workspace and contract (first, small)

1. `rust-toolchain.toml`: `channel = "1.99.0"`, components `rustfmt`, `clippy`, profile minimal.
2. `backend/Cargo.toml` workspace, edition 2024, `resolver = "3"`. Members: `app`, `crates/core`, `crates/domain`, `crates/db`, `crates/services`, `crates/reports`, `crates/backup`. Crate names `mandi-app`, `mandi-core`, `mandi-domain`, `mandi-db`, `mandi-services`, `mandi-reports`, `mandi-backup`. Dependencies follow CLAUDE.md rule 6 exactly: app -> services, reports, backup; services -> domain, db; reports -> db, core; backup -> db, core; domain -> core; db -> core. A short `README.md` in each crate (NFR-MNT-6): what it holds and what it may depend on.
3. Workspace lints (`[workspace.lints]`, each crate `lints.workspace = true`):
   - rust: `unsafe_code = "forbid"`.
   - clippy: `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr` = deny. `clippy.toml` with `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`, `allow-indexing-slicing-in-tests`, `allow-print-in-tests` = true.
   - `arithmetic_side_effects` = deny inside `mandi-core` (money lives there; it is applied crate-wide in `core` and `domain`, tests excepted).
4. `backend/app`: Tauri 2 app. `tauri.conf.json` exactly as Release_Pipeline 3.3, with:
   - `productName` "Mandi Software", `version` "0.1.0", `identifier` "pk.mandisoftware.app".
   - `pubkey` = the full contents of `docs/release/updater-pubkey.txt` (one line, base64).
   - `build.beforeDevCommand` `pnpm --filter frontend dev`, `build.devUrl` `http://localhost:5173`, `build.beforeBuildCommand` `pnpm --filter frontend build`, `build.frontendDist` `../../frontend/dist` (Tech Stack 5.5).
   - Window: title "Mandi Software", min size 1024 x 600 (NFR-USE-16), default 1366 x 768, centered.
   - A strict CSP: `default-src 'self'; script-src 'self'; style-src 'self'; font-src 'self'; img-src 'self' data:; connect-src ipc: http://ipc.localhost` and nothing remote (Code Standards S5, NFR-SEC-9). If Tauri dev mode needs more, use `devCsp` only.
   - `bundle.targets` `["nsis","msi"]`, `createUpdaterArtifacts` true, `windows.webviewInstallMode` offlineInstaller (NFR-INS-1). Icons generated with `pnpm tauri icon` from a simple placeholder.
5. Plugins: only `tauri-plugin-single-instance` (registered first; a second launch focuses the first window, NFR-REL-9), `tauri-plugin-updater`, `tauri-plugin-dialog`, `tauri-plugin-window-state`, `tauri-plugin-process`. No fs, no shell.
6. WebView2 settings on Windows (Tech Stack 7) through `with_webview` and `webview2-com`, behind `#[cfg(windows)]`: `AreBrowserAcceleratorKeysEnabled = false`, `IsZoomControlEnabled = false`, `AreDefaultContextMenusEnabled = false`. The COM calls need `unsafe`; keep them in one small module with `#![allow(unsafe_code)]` scoped to that module only, with a comment naming Tech Stack 7. This is the only allowed exception. If you find a safe wrapper, use it instead.
7. `capabilities/default.json`: only `app_info` and the plugin permissions the shell needs (window-state, updater check, process relaunch, dialog). Nothing else (NFR-SEC-10).
8. `app_info` command, `UiError` type, ts-rs export. `pnpm gen:types` (script owned by frontend, coordinate with them) writes `frontend/src/bindings/`.
9. `backend/deny.toml`: licences allowed (MIT, Apache-2.0, BSD-2/3, ISC, Unicode-3.0, Zlib, MPL-2.0, CC0-1.0, BSL-1.0, OFL for fonts if any), advisories deny, bans wildcard deps, sources crates.io only (plus the git source only if a crate truly needs it). `cargo deny check` must pass.
10. Message `frontend` the command and type names as soon as the bindings exist.

### T2 backend: checks green

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check` all pass on the Mac.
- One unit test in each crate so `cargo test` covers every crate.
- A test that `tauri.conf.json` has the identifier, the pubkey matching `docs/release/updater-pubkey.txt`, the endpoint, the targets and offlineInstaller (guards Release_Pipeline 3.3).

### T3 frontend: app, tooling and scripts

1. `pnpm-workspace.yaml` with `frontend`. Root `package.json`: `"private": true`, `"packageManager": "pnpm@12.11.2"`, `"engines": { "node": ">=24" }`. Scripts, exactly these names:
   - `dev` = `tauri dev`; `build` = `tauri build`; `tauri` = `tauri` (the CLI, `@tauri-apps/cli` pinned).
   - `lint` = ESLint + Prettier check + Stylelint (frontend), then `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` in `backend/`.
   - `typecheck` = `tsc --noEmit` for the frontend.
   - `test` = Vitest run, then `cargo test --workspace` in `backend/`.
   - `gen:types` = run the ts-rs export (`cargo test` with `TS_RS_EXPORT_DIR` pointing at `frontend/src/bindings`, or the ts-rs export test name), then Prettier on the bindings, so the output is stable and CI's "no diff" check passes.
   - `i18n:check` = a Node script: every `.ftl` file exists in both `ur/` and `en/`, same message IDs, none empty, and none identical to the other language unless listed in an allow list (codes, "Mandi Software") (NFR-URD-22).
   - `seed` = runs the backend seed (prints "seed: not available until plan 03" and exits 0 until 03 adds it; backend replaces it in 03).
   - `perf` = `node scripts/perf/run.mjs` (owned by `robustness`).
   - `e2e` = WebdriverIO run of `frontend/tests/e2e/`. On a non-Windows machine it prints "e2e runs on Windows only" and exits 0.
   - Any script that needs a path with spaces must quote it (the project folder has a space).
2. `frontend/`: React + TypeScript + Vite. Latest stable versions, pinned exactly in `package.json` (no `^`). Libraries from Tech Stack 3: React, TanStack Router, TanStack Query, Zustand, React Hook Form, Zod, AG Grid Community, Radix UI, Downshift, Tailwind CSS, Lucide, Apache ECharts, `@fluent/bundle`, `@fluent/react`, Vitest, Testing Library, ESLint with typescript-eslint strict (type-aware), Prettier, `@tauri-apps/api`, the Tauri plugin JS packages for the five plugins. If the newest TypeScript is not yet supported by typescript-eslint, pin the newest TypeScript it supports and write that reason in `frontend/README.md`.
   - `tsconfig`: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes` (Code Standards F1).
   - Folder layout of Tech Stack 5.3 (empty folders get a short `README.md` or `index.ts`, not `.gitkeep` noise).
3. Fluent: `frontend/src/i18n/ur/` and `en/` with `glossary.ftl` (every row of PRD Section 3, Urdu column and English column, message IDs `glossary-<roman>` in kebab-case English, for example `glossary-bikri`, `glossary-naqdi-mandi`), `common.ftl` (app name, the starting screen text) and `errors.ftl` (`err-not-implemented` and a generic `err-internal`). Both bundles load at start. `<html dir lang>` follows the one `language` value in the store. `Alt+L` switches (FR-7.30, basic version; the full shell comes in 04).
4. ESLint rules that fail the build (D1-A4):
   - **Rule 8, physical directions.** In JSX `className` strings: Tailwind `ml- mr- pl- pr- left- right- text-left text-right float-left float-right rounded-l rounded-r border-l border-r scroll-ml` and similar. In `style` objects: `marginLeft`, `marginRight`, `paddingLeft`, `paddingRight`, `left`, `right`, `borderLeft*`, `borderRight*`, `textAlign: 'left'|'right'`. In CSS files: Stylelint with a logical-properties rule (for example `stylelint-use-logical`) so `margin-left`, `left:`, `text-align: left` fail.
   - **Rule 7, no literal text.** `react/jsx-no-literals` (or an equivalent) so text in JSX fails. Numbers and the `·` style separators may be allowed only through components.
   - **Rule 1, no money maths.** A local ESLint rule (type-aware, in `frontend/eslint-rules/`) that fails on `+ - * / %`, `+=` and similar, `Math.round`, `Math.floor`, `toFixed`, `parseFloat` when an operand has the branded type `Paisa`. The only allowed file is `frontend/src/shared/format/money.ts` (Code Standards F3, M3, M12). Define `Paisa` as a branded number type in `frontend/src/shared/format/money.ts` now (the formatter itself comes in 02).
   - typescript-eslint `no-explicit-any`, `ban-ts-comment`, and no `console.log` (G4).
5. The first screen: a calm placeholder page that shows the app name and `app_info().version` from Fluent in the current language, with `Alt+L` to switch. It uses design tokens in `frontend/src/styles/tokens.css` (a minimal first set; the full design system comes after the design step). Fonts are bundled from `frontend/public/fonts/` (Gulzar, Noto Nastaliq Urdu, Noto Naskh Arabic, Inter; all SIL OFL, licence files included). No remote fonts.
6. The Windows end-to-end suite in `frontend/tests/e2e/`: WebdriverIO + `tauri-driver`. Config starts `tauri-driver` and points `tauri:options.application` at `backend/target/release/mandi-app.exe` (the binary from `pnpm tauri build`; check the real file name). First test: the window opens, the app name shows in Urdu with `dir="rtl"`, `Alt+L` switches to English with `dir="ltr"`. Screenshots in Urdu and English at 1024 x 600, 1366 x 768 and 1920 x 1080, saved as `frontend/tests/e2e/output/screenshots/<test>-<lang>-<w>x<h>.png`. A JSON or JUnit report in `frontend/tests/e2e/output/`. No visual comparison yet (D1-Q1 is open).
7. Vitest: one component test for the first screen (renders in ur and en, `dir` changes) and the i18n check script tested.

### T4 frontend: wire `app_info` for real

When `backend` says the command works, the screen shows the real version through `api.ts`. On Mac, `pnpm dev` opens the window.

### T5 robustness: perf harness and runner factor

1. `scripts/perf/run.mjs` (the `pnpm perf` entry): runs `cargo bench --workspace` (criterion) and collects the results; on Windows, if `backend/target/release/bundle/nsis/*-setup.exe` exists, installs it silently (`/S`), times it, checks the installed exe exists, uninstalls it, and records the install time against NFR-INS-1 (under 2 minutes). Exits non-zero when a measured value is over its runner threshold.
2. One criterion benchmark now in `backend/crates/core/benches/` (a placeholder workload is fine until 02 gives real functions; replace it in 02).
3. Runner factor (README Section 6): define an honest, repeatable method, run it, and write `scripts/perf/README.md` with the method, the factor, and a table of every NFR Section 2 target beside its runner threshold. The reference machine (NFR 0.2: Core i3 7th gen, 4 GB, SSD) is not available. A sound method is a fixed CPU and SQLite workload whose time on reference-class hardware is taken from public single-thread benchmark scores, with the source written down. If the runner is faster than the reference, the factor is 1.0 (we never loosen a target because the runner is fast). The factor is recorded once; later plan files only add rows.
4. Robustness checks for this plan: the lint rules really fail on violations (rules 1, 5, 7, 8), `cargo deny` and `pnpm audit` pass, the capability file allows nothing else, the CSP has no remote source, a second app launch does not start a second instance (Windows E2E if possible, otherwise code review of the plugin order).

### T6 reviewer

Review against Section 6 below and the D1 review focus (D1 Section 5).

### T7 lead

Gate, `docs/reviews/01-scaffold.md`, `docs/release/whats-new.md`, commit, pull request, merge after both checks are green, first Release.

## 6. Acceptance checklist

1. Tree matches Tech Stack 5.1, 5.3 and 5.4 (folders present for the parts this plan creates).
2. `pnpm install`, `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm gen:types` (no diff after), `pnpm i18n:check`, `pnpm seed`, `pnpm perf`, `pnpm e2e` (skip on Mac), `pnpm dev`, `pnpm build` all run and exit 0 on the Mac (build may produce a Mac bundle; the Windows bundle comes from CI).
3. `cargo fmt`, `clippy -D warnings`, `cargo test`, `cargo deny check` pass.
4. A deliberate violation of each rule fails lint: money arithmetic on `Paisa` (rule 1), `unwrap()` in non-test Rust (rule 5), a literal word in JSX (rule 7), `ml-2` or `margin-left` (rule 8). Output pasted, then reverted.
5. `tauri.conf.json` matches Release_Pipeline 3.3, with the real public key.
6. Only the five Tech Stack plugins; single instance first; capability file lists only our command.
7. CSP has no remote source; fonts are local files with their licences.
8. `glossary.ftl` has every PRD Section 3 row in both languages; `i18n:check` fails on a missing or empty key (shown by a test).
9. On Windows CI: installers built (NSIS and MSI, offline WebView2), the first E2E test passes with 6 screenshots, `pnpm perf` reports install time under 2 minutes, both required checks green.
10. `scripts/perf/README.md` has the runner factor method and the threshold table.

## 7. Out of scope

Money, database, login, the design system (comes after the design step), every business screen.

## 8. Open questions

- D1-Q1 (screenshot comparison tool) is still open. Until Umar answers, screenshots are saved without comparison.
