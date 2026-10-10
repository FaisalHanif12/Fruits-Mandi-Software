# Plan 04: Sign-in, roles, first run and the app shell

Deliverable: D1, plan file 04. Branches: `feature/auth-login` (backend and login screens) and `feature/shell-keyboard` (shell, keyboard engine, grid, settings). Two pull requests, one purpose each.

## 1. Goal

The D1 story of `docs/deliverables/D1-foundation.md` Section 1 works end to end on Windows by keyboard only, in both languages: first-run wizard, login with lockout, the shell with the main menu on the start side, the top bar and status bar, the بکری side menu frame, the empty dashboard, Settings (firm, users, idle lock, language, text size), `Alt+L` everywhere, idle lock that keeps unsaved work, and audit rows for every login event.

## 2. Requirements

PRD 2 (FR-2.1 to FR-2.4), 5 (FR-5.1 to FR-5.3, FR-5.5, FR-5.6), 7.1 (FR-7.1 to FR-7.11), 7.2 and 7.3 (FR-7.13 to FR-7.20), 7.5 (FR-7.30 to FR-7.34, FR-7.37), 8 (FR-8.1 to FR-8.6), 21 (firm, users, idle lock, language, text size). NFR-INS-2, NFR-SEC-1 to NFR-SEC-4, NFR-SEC-12, NFR-SPD-1 to NFR-SPD-5, NFR-SPD-19, NFR-USE-7, NFR-USE-8, NFR-USE-10 to NFR-USE-24, NFR-USE-28, NFR-URD-5 to NFR-URD-7, NFR-URD-9 to NFR-URD-13, NFR-URD-21, NFR-URD-27. Architecture 4, 5, 18. Code Standards 6.

Acceptance items: D1-A12, D1-A13, D1-A14, D1-A15 (timings), D1-A16 (screens).

Design: `docs/design/design-system.md` and the boards `LoginUr/En`, `WizardUr/En`, `ShellUr/En`, `UsersUr/En`, `FormUr/En`, `GridUr/En`, `StatesUr`, `CompactUr` in `docs/design/canvas/`.

## 3. Worked numbers

Lockout (FR-8.2), per user name:

| Wrong tries in a row | Result |
|---|---|
| 1 to 4 | `ERR_WRONG_PASSWORD` with `tries_left` 4, 3, 2, 1 |
| 5 | locked 60 seconds (`ERR_LOCKED_WAIT`, `seconds` = remaining) |
| 6 to 9 (after the wait) | `ERR_WRONG_PASSWORD` |
| 10 | locked 300 seconds |
| 15, 20, 25 ... | locked 900 seconds each time |
| correct password | counter back to 0 |

Every try, right or wrong, writes an audit row (`login`, `login_failed`, `logout`, `lock`, `unlock`, `recover`). An unknown user name gives the same `ERR_WRONG_PASSWORD` (no hint that the name is unknown) and an audit row with the typed name.

## 4. Contract

All commands are `async`, run blocking work on a worker thread, check the role first, then parse, then call one service (Code Standards 5.6). Errors are `UiError` (plan 01). Types are exported with ts-rs.

| Command | Role | Input | Output | Errors (field) |
|---|---|---|---|---|
| `auth_status` | none | none | `AuthStatus { first_run, session: Option<SessionUser>, locked: bool, login_language: "ur"\|"en", firm: FirmDisplay, app_version }` | |
| `setup_first_run` | none, only while no user exists | `FirstRunInput { firm_name_ur, firm_name_en, mandi_ur, mandi_en, address, phone, owner_username, owner_display_name, password, password_repeat }` (all text) | `FirstRunDone { recovery_code }` | `ERR_ALREADY_SET_UP`, `ERR_REQUIRED` (each field), `ERR_NAME_UR_SCRIPT` (`firm_name_ur`), `ERR_NAME_EN_SCRIPT` (`firm_name_en`), `ERR_PASSWORD_TOO_SHORT` (`password`, min 6, NFR-SEC-1), `ERR_PASSWORDS_DIFFER` (`password_repeat`), `ERR_USERNAME_INVALID` (`owner_username`: 3 to 32 of `a-z 0-9 . _ -`), `ERR_PHONE_INVALID` |
| `auth_login` | none | `LoginInput { username, password }` | `SessionUser { id, username, display_name, role, language, text_size }` | `ERR_WRONG_PASSWORD { tries_left }` (`password`), `ERR_LOCKED_WAIT { seconds }`, `ERR_USER_INACTIVE` |
| `auth_logout` | any logged in | none | `()` | |
| `auth_lock` | any logged in | none | `()` | |
| `auth_unlock` | the locked user | `{ password }` | `SessionUser` | as login (same lockout counter) |
| `auth_recover` | none | `{ username, recovery_code, new_password, new_password_repeat }` | `RecoverDone { new_recovery_code }` | `ERR_RECOVERY_CODE_WRONG` (counts as a wrong try), password errors. Owner only. A used code is spent; a new one is shown to print. |
| `users_list` | Owner | none | `Vec<UserRow { id, username, display_name, role, is_active, last_login_at }>` | `ERR_NOT_ALLOWED` |
| `users_create` | Owner | `{ username, display_name, role, password, password_repeat }` | `UserRow` | `ERR_USERNAME_TAKEN` (`username`), the password and name errors |
| `users_update` | Owner | `{ id, display_name, role, is_active }` | `UserRow` | `ERR_LAST_OWNER` (cannot make the last active Owner inactive or not Owner) |
| `users_reset_password` | Owner | `{ id, password, password_repeat }` | `()` | password errors |
| `settings_get` | any logged in | none | `Settings { firm: FirmSettings, idle_lock_minutes, default_language, number_grouping }` | |
| `settings_set_firm` | Owner | `FirmSettings` (text) | `FirmSettings` | name script errors |
| `settings_set_security` | Owner | `{ idle_lock_minutes }` (1 to 120, default 15) | `()` | `ERR_OUT_OF_RANGE` |
| `settings_set_default_language` | Owner | `{ language }` | `()` | |
| `prefs_set` | any logged in | `{ key: "language"\|"text_size"\|"last_screen", value }` (text size 80 to 150 in steps of 10) | `()` | `ERR_OUT_OF_RANGE` |
| `day_status` | any logged in | none | `DayStatus { today, closed_up_to: Option<date>, today_open: bool }` | |

- Roles: `require_at_least(Role::Munshi)` lets Owner and Munshi through; `require_owner()`; Partner is view only (FR-2.4). A refused call writes nothing except an audit row `denied` (NFR-SEC-4).
- One logged-in user per app (FR-2.3). Session lives in `AppState` in memory.
- Passwords: Argon2id (default params of the `argon2` crate's recommended setting), never logged, never returned (S2). Recovery code: 16 characters from an unambiguous alphabet, shown as `XXXX-XXXX-XXXX-XXXX`, stored as an Argon2id hash.
- The clock is injected. Lock times use the wall clock stored in `users.locked_until_at` plus an in-process monotonic check, so moving the clock back during a lock never ends it early and never makes it longer than its step while the app runs (NFR-REL-11, robustness test).
- `login_language` setting is the last language used on this computer (FR-7.30).
- `today` for `day_status` is the computer's local date (Pakistan). `closed_up_to` is null until D2 adds day close.

## 5. Frontend

From the boards, with `bilingual-screen`, `urdu-english-text`, `entry-grid` skills:

1. Design system completion (tokens and base components came in 02): `LanguageSwitch`, `MainNav` (full and rail), `ModuleMenu`, `TopBar` (search box and bell are placeholders that say "coming soon" in a tooltip; `Ctrl+K` focuses the box), `StatusBar`, `Stepper`, `SaveChangesPrompt`, text size with `Ctrl +`, `Ctrl -`, `Ctrl 0` saved per user (FR-7.11, NFR-USE-18: stops where the content area would be narrower than 1024 px).
2. `shared/keyboard`: the arrow rules FR-7.13 to FR-7.18 for forms, menus and lists, in both directions; the shortcut registry with every key of PRD 7.3 in one file (F29); `Esc` order (FR-7.18); date field keys (FR-7.19); number fields with thousands separators shown as you type, value kept as text (FR-7.20).
3. `shared/grid`: AG Grid setup with RTL from the language, column types (money, qty, account, date, text) with fixed widths per type (F18), the Enter rule (skip read-only, stop on editable including app-filled), `↓` on the last row adds a row, `Ctrl+Delete` removes, `F2` does not start editing, `Esc` order. AG Grid uses its CSS-file theming (`theme: "legacy"` with the grid and theme CSS files imported through Vite and restyled from our tokens), never runtime `<style>` injection, so the CSP keeps `style-src 'self'` (Code Standards S5; lead decision 10-10-2026). Proven on `/dev/grid` (development build only) with a test of every key in both directions.
4. Screens: login (lockout message with a live countdown, wrong-password field error, show-password, exit), recovery path, first-run wizard (firm, Owner password, recovery code with print `F9` and the "I printed it" tick), shell, بکری side menu with seven items that open "coming soon" empty states, empty dashboard, Settings tabs (firm, users with add and edit dialogs, auto lock, language and text size). Idle lock: an overlay after the set minutes; typing the password unlocks; nothing under it is lost (FR-8.4).
5. Language switch on every screen, no reload, nothing typed lost (unsaved form data in stores above the screens, F39). Keys follow screen direction (FR-7.34).
6. Windows E2E (`frontend/tests/e2e/`): the D1 story by keyboard only in Urdu and in English; screenshots of every screen in both languages at the three sizes; timings for cold start, login to dashboard, screen open, arrow move, language switch written to `frontend/tests/e2e/output/timings.json`.

## 6. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Types and command stubs, `pnpm gen:types`, message frontend | backend | 03 |
| T2 | Services: users, sessions, Argon2id, lockout, recovery, settings, prefs, day status, audit rows; integration tests for every row of Section 3, role refusal for every command (D1-A13), idle lock keeps the session | backend | T1 |
| T3 | Design system completion, keyboard engine, grid, screens against stubs | frontend | T1 |
| T4 | Wire to real commands; E2E story and screenshots | frontend | T2, T3 |
| T5 | Review (README Section 4 worked example) | reviewer | T2, T4 |
| T6 | Robustness: D1 attack list items 2, 4, 5, 6 and timings vs runner thresholds | robustness | T2, T4 |
| T7 | Gate, `windows-verify`, merge | lead | T5, T6 |

## 7. Acceptance checklist

1. Section 3 lockout table passes as an integration test; audit rows for login, failed login, logout (D1-A11, D1-A12).
2. Idle lock after the set minutes; unlocking keeps unsaved form text (component test and E2E).
3. Recovery code resets the Owner password; the old code no longer works.
4. Every command refuses a lower role and writes nothing but the `denied` audit row (D1-A13).
5. First-run wizard and the D1 story pass by keyboard only, in Urdu and English, on Windows (D1-A14).
6. Timings within runner thresholds: NFR-SPD-1, 2, 3, 5, 19 (D1-A15).
7. Screens match the boards in both languages at 1024 x 600, 1366 x 768, 1920 x 1080; contrast check passes; Urdu has no broken letters (D1-A16).
8. No literal text, no physical CSS directions, no one-off components or key handlers.
9. `pnpm lint`, `pnpm test`, `pnpm i18n:check` exit 0.

## 8. Out of scope

Accounts and masters (D2), partners and the wizard partners step (D3), backup key step and update prompt (D5), global search results and the attention list (D2 and D5).

## 9. Open questions

None. Decision recorded here: a used recovery code is spent and a new one is shown, because NFR-SEC-12 says no master password exists anywhere, and a printed code that works forever would be one.
