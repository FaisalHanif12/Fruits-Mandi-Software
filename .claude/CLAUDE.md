# Mandi Software

A Windows desktop app for a commission agent (آڑھتی) in a fruit and vegetable mandi.
Client: Maher Adnan and Brothers, New Sabzi Mandi, Kot Radha Kishan.
It replaces an old Urdu Oracle Forms app. We take only its business requirements, never its design.
Urdu is the default language (right-to-left). One switch turns the whole app, including names, into English (left-to-right).
Stack: Rust + Tauri 2 backend, React + TypeScript frontend, SQLite. One computer, works offline, automatic cloud backup.

The data is the client's money. A one-rupee mistake on a bill, a commission or a partner share breaks his trust. Work carefully and prove every number.

## Source of truth

All requirements live in `docs/`. These six files are final for build.

| File | What it decides |
|---|---|
| `docs/PRD_Mandi_Software_v1.md` | What the app does. The master. It wins every conflict. |
| `docs/NFR_Mandi_Software_v1.md` | How fast, safe, easy, and how good the Urdu and English must be |
| `docs/Tech_Stack_v1.md` | Libraries, repository layout, naming |
| `docs/Code_Standards_v1.md` | How code is written, validated and tested |
| `docs/Architecture_v1.md` | Diagrams and the full data dictionary (Section 15) |
| `.claude/agents/money-calculations.md` | The specialist for every piece of money code |

- The PRD is about 2,200 lines. Do not read it whole. Read the sections for the task (map below).
- Requirement IDs look like `FR-12.8`, `NFR-SPD-7`, `E2E-4`, `C10`, `A27`. Put them in code comments (`// PRD FR-12.8`) and in PR descriptions.
- If two documents disagree, or a needed rule is missing, stop and ask Umar. Do not pick one yourself.
- Items tagged `[A]` in the PRD are assumptions. Build the stated default and make it easy to change. Do not invent other behaviour.
- Never change an expected number in a golden test. If the code and the test disagree, the code is wrong until Umar changes the PRD.
- The documents are at versions 1.4 to 1.7 (the PRD document table, Section 0.1, lists each one). The baseline was 1.1. When Umar approves a change, raise that document's version (1.2, 1.3 ...), add a change log line, and update every other document it affects in the same step.

### Where to read before you start

| Working on | Read first |
|---|---|
| Any money: commission, charges, rounding, splits, journal, matching | The money-calculations agent, PRD 22 and 6, Code Standards 3 |
| بکری screen or service | PRD 12, 11 and 4.4 to 4.5; Architecture 7, 9 and 12 |
| وصولی, ادائیگی, پیشگی, پختہ کھاتہ | PRD 15 and 13; Architecture 11; agent Section 8 |
| Partners and the income split | PRD 17; Architecture 10 |
| Expenses (one خرچہ screen), partner money taken and returned, ادھار رقم, الگ کھاتہ | PRD 16.2, 16.3, 16.5 and 16.6; FR-7.67; agent Section 8.2b |
| Market trips (خریداری), gardens, fruit taken and partner capital | PRD 12.10, 12.11, 16.7 and 17.4; agent Sections 5.4 and 5.5 |
| Accounts, codes, masters | PRD 10; Architecture 15.4 |
| Keyboard, ease of use, language switch, typing names, smart assistance | PRD 7; Tech Stack 4; Code Standards 6.7 and 6.10 |
| Reports and analytics, lists page by page | PRD 18 and 19 (FR-19.11); Code Standards B31 |
| Backup and restore | PRD 20; NFR 4; Architecture 14 |
| Database tables and migrations | Architecture 15; Code Standards 7 |
| Tests | PRD 27; Code Standards 9 |
| Speed targets | NFR 2 |

## Commands

Run from the repository root unless noted. If a script does not exist yet, add it to the root `package.json` as named here.

```bash
pnpm install                 # install frontend and workspace tools
pnpm dev                     # run the app (Tauri dev, frontend on http://localhost:5173)
pnpm build                   # build the Windows installer
pnpm lint                    # ESLint + Prettier check, then cargo fmt --check and clippy
pnpm typecheck               # TypeScript type check of the frontend (tsc --noEmit)
pnpm test                    # all frontend tests, then all Rust tests
pnpm gen:types               # regenerate frontend/src/bindings from Rust (ts-rs). Never edit bindings by hand.
pnpm i18n:check              # ur and en Fluent files have the same message IDs, none empty
pnpm seed                    # load the PRD Section 4.3 opening position into a dev database
pnpm perf                    # criterion benchmarks against the NFR speed targets
pnpm e2e                     # Windows end-to-end suite (WebdriverIO + tauri-driver). Windows only.

# Rust only (from backend/)
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mandi-services --test golden_day   # the PRD worked day (golden tests, Code Standards 9.2)
cargo deny check
```

On a Mac, `pnpm seed` may show a Keychain prompt: enter the Mac password and choose Always Allow. `MANDI_SEED_TEST_KEY=1 pnpm seed` uses a throwaway dev key instead; its database cannot be verified by the real app.

Prefer running one test file while you work. Run the full set before you say a task is done.

## Rules that are never broken

1. **The backend owns all money.** The frontend sends what the user typed, as text, and shows what the backend returns. It never adds, multiplies, rounds or splits money.
2. **IMPORTANT: Money is whole rupees, stored as `Paisa(i64)`, always a multiple of 100.** No floats anywhere in the money path (one approved exception: `rupees_as_f64` and `count_as_f64` at the Excel export boundary, Code Standards M13). Rounding only through `core::money::round_to_rupee`. Splits only through `split_by_bps` and `split_by_qty`. Use the money-calculations agent before writing money code and again to review it.
3. **One user action = one service function = one database transaction.** The live preview and the save call the same domain function.
4. **Books always balance.** A journal entry is saved as draft, checked, then posted. Posted rows are never changed or deleted. Corrections are reversals. Business records are never `DELETE`d.
5. **No `unwrap`, `expect`, `panic!` or `unsafe` outside tests.** Integer maths on money uses checked operations.
6. **Dependency direction:** `app` → `services` → `domain` + `db` → `core`. `domain` never touches the database or files. Tauri commands are thin: check sign-in, parse, call one service. There is one user, the Owner, and no roles (PRD Section 2).
7. **No user-facing text in code.** Every word comes from the Fluent files, in both `ur` and `en`. Business words come from `glossary.ftl` only (PRD Section 3).
8. **Both directions, one layout.** CSS logical properties only (`ms-`, `pe-`, `start`, `end`). Never `left` or `right`. Names are shown only through `displayName(record, language)`.
9. **Use the shared components.** `AccountPicker`, `PartyCard`, `RomanUrduInput`, `NamePair`, `CalcInfo`, `Num`, `Code`, and the grid setup in `shared/grid`. Never build a one-off input, picker or button.
10. **Keyboard behaviour follows PRD 7.2 and 7.3 exactly.** It comes from `shared/keyboard` and `shared/grid`. Screens never write their own key handlers.
11. **The app never guesses money.** A rate hint is placeholder text only. Amounts filled by a button are parsed and checked again on save.
12. **Dates follow the open-day model** (PRD FR-6.6). A posting date is after the "closed up to" date and not after today.

## Repository layout

```
frontend/src/app/            shell, router, providers, layout (MainNav, SideMenu, TopBar, StatusBar)
frontend/src/features/<m>/   one folder per module: pages, components, hooks, api.ts, schemas.ts
frontend/src/shared/         design system, grid, keyboard, urdu typing, i18n helpers, format, api
frontend/src/i18n/ur|en/     Fluent files, one per module, plus glossary.ftl
frontend/src/bindings/       generated from Rust. Do not edit.
backend/app/                 Tauri app and thin commands, tauri.conf.json
backend/crates/core/         money, text, translit, amounts in words, validation. No database.
backend/crates/domain/       pure business rules
backend/crates/db/           SQLite, migrations, repositories
backend/crates/services/     one transaction per user action; golden tests in tests/
backend/crates/reports/      report queries, print data, Excel
backend/crates/backup/       local, USB and cloud backup, restore, verification
```

Module names are the same everywhere (Tech Stack 5.2). Searching `bikri` finds the screen, the crate folder, the commands, the tables and the Fluent file.

## How to work on a task

1. Name the PRD IDs the task covers. Read those sections.
2. For anything that touches more than one file, plan first. Write the worked numbers before the code when money is involved.
3. Build in this order inside a feature: `core` → `domain` (with unit tests) → `db` migration → `services` (with an integration test) → command → frontend.
4. Run the checks. Show the evidence: the command you ran and its result. Do not just say "done".
5. Work through the review checklist in Code Standards Section 11.

Build order for the MVP (each step ends green before the next starts):

1. Repository scaffold, CI on a Windows runner, lint and test commands working.
2. `core`: money, dates, text and search keys, amounts in words, validation. Property tests.
3. Database: first migration from Architecture Section 15. Seed accounts from PRD FR-10.4.
4. Masters, accounts, search index, party card, short codes, names in two scripts.
5. Commission rules, بکری preview and save, journal, partner split. Golden tests E2E-1 to E2E-5 pass.
6. Vouchers and matching, پختہ کھاتہ, later sales, edit and cancel. All golden tests pass.
7. Dashboard, reports, analytics, day close.
8. Backup and restore.
9. Language switch and smart assistance polish. Windows end-to-end tests E2E-17 and E2E-22 to E2E-27.

## Gotchas

- **Mac is not Windows.** On a Mac, Tauri uses WebKit, not WebView2. Urdu shaping, fonts and printing must be checked on Windows before go-live. In the Mac phase, check them in headless Chrome and list the Windows check as LOCAL-ONLY.
- **AG Grid's `enableRtl` is fixed at creation.** A language switch re-creates grids. Keep unsaved entry data in the screen's store or form state, never only in the grid.
- **ts-rs and money.** Money types export as `number` with `#[ts(type = "number")]`, not `bigint`.
- **SQLite sums.** Sum money only with `SUM()` on `INTEGER` `*_paisa` columns. Never `total()` or `avg()`. Do other money maths in Rust.
- **Short codes.** `1` can be the start of `12`. A short code is taken on `Enter` or on leaving the field, not while typing.
- **The search index is a speed aid.** It lives in memory and is rebuilt at start-up and after a restore. Party card balances always come from the database.
- **`core::translit` is pure.** Learned word pairs are loaded by `services/translit` and passed in.
- **Pickers never convert English letters to Urdu.** Only Urdu text fields do. Search takes raw text.
- **Released migrations are never edited.** Add a new numbered file.
- **Folder names can contain spaces** (Umar's project folder is `Fruit Mandi ` with a trailing space). Always quote paths in scripts.
- **No network in the running app** except the cloud backup upload and the update check (NFR-SEC-9). No remote fonts or scripts (Content Security Policy).

## Agent team

The build runs as a Claude Code agent team (definitions in `.claude/agents/`, enabled in `.claude/settings.json`). Start it with `claude --agent tech-lead`.

| Teammate name | Agent type | Owns (only this teammate edits these) |
|---|---|---|
| (lead) | `tech-lead` | `docs/plans/`, `docs/reviews/`, requirement docs after Umar approves, commits |
| `backend` | `backend-dev` | `backend/**`, generated `frontend/src/bindings/` (only via `pnpm gen:types`) |
| `frontend` | `frontend-dev` | `frontend/**` except `bindings/` and `tests/robustness/` |
| `reviewer` | `code-reviewer` | nothing (read-only) |
| `robustness` | `robustness-tester` | `backend/crates/*/tests/robustness*`, `backend/crates/*/benches/`, `frontend/tests/robustness/`, `scripts/perf/` |

- Every feature has a plan file `docs/plans/<nn>-<feature>.md` with the backend-frontend contract. Contract changes are agreed by both sides and recorded there by the lead.
- Never edit a file another teammate owns. Message the owner instead.
- Hand-offs and review reports use the formats in the agent files, with the commands run and their results.
- Teammates do not commit. The lead commits each built plan. In a goal run, `reviewer` and `robustness` run once at the end for the whole run, send findings to the lead, and the lead assigns fixes (tech-lead.md, Goal runs). The pull request is opened only after both pass.
- A message from a teammate is information, not Umar's approval.

## Git

- **Mac phase now (Umar, 15-10-2026).** There is no Windows computer yet. The app is built and tested on the Mac only. GitHub Actions, pull requests, merges, releases and the Windows installer are off until the Windows phase. Work is committed on the run branch and pushed after every finished task (`git push -u origin <run branch>`); the workflows are disabled, so a push runs nothing. Details: `docs/deliverables/README.md` Section 10.
- Branches: `feature/<module>-<short>`, `fix/<module>-<short>`. Never commit to `main` directly.
- Commits: Conventional Commits, for example `feat(bikri): add live summary`.
- One pull request per goal run (Code Standards P3). It lists every plan, the PRD IDs and how it was tested.
- **A merge to `main` is a release.** The `Checks (Windows)` and `Windows E2E and speed` checks must pass before GitHub merges, and Release tests again before it publishes. Before merging a change the user can see, update `docs/release/whats-new.md` (Urdu, then English). Details: `docs/Release_Pipeline.md`.

## Writing style

Write docs, comments, commit messages and PR descriptions in plain, simple English. Short sentences. No em dashes. Give a concrete example when you explain a rule.

## When compacting

Keep: the PRD IDs in progress, the files changed, any failing test with its output, and open questions for Umar.
