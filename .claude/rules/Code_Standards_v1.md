# Code Standards
# Mandi Software for Commission Agents (آڑھتی)

| | |
|---|---|
| Version | 1.8 (Umar's changes of 14-10-2026, 15-10-2026, 18-10-2026, 19-10-2026, 22-10-2026 and 23-10-2026) |
| Date | 23 October 2026 |
| Status | Final for build |
| Depends on | PRD v1.7 (the master), NFR v1.4, Tech Stack v1.5 |
| Applies to | Every line of code, written by people or by Claude Code |

---

## 0. How to use this document

These are rules, not suggestions. Words mean:

- **MUST** / **MUST NOT**: always. A pull request that breaks one is not merged.
- **SHOULD**: the normal way. Breaking it needs a short comment explaining why.

Most rules are checked by tools (formatter, linter, tests in the build pipeline). The rest are checked in review using the checklist in Section 11.

All money rules are summarised in Section 3. The full detail, with every formula and worked example, is in `.claude/agents/money-calculations.md`. Claude Code MUST use that agent for any code that touches money.

---

## 1. Principles

1. **Correct before fast before pretty.** A wrong rupee is worse than a slow screen.
2. **One place for each thing.** One money module, one rounding function, one design token file, one keyboard engine, one translation system. Never two ways to do the same thing.
3. **The backend is the source of truth.** The frontend shows; the backend decides.
4. **Fail loudly, never silently.** If something is wrong, stop and show a clear message. Never guess, never continue with a half result.
5. **Simple code over clever code.** A new developer (or Claude in a new session) must understand a function in one read.
6. **Every rule from the PRD has a test.** If it is not tested, it is not done.

---

## 2. Rules for all code

| # | Rule |
|---|---|
| G1 | MUST pass the formatter and linter with zero warnings (`cargo fmt`, `cargo clippy -D warnings`, `prettier`, `eslint`) |
| G2 | MUST use the names and folders from Tech Stack Section 5 and 6 |
| G3 | Functions SHOULD be under 50 lines; files SHOULD be under 400 lines. Split by meaning, not by count. |
| G4 | MUST NOT leave commented-out code, `TODO` without an issue number, or `console.log` / `println!` |
| G5 | Comments explain **why**, not what. Every public function has a doc comment with one example for anything involving money. |
| G6 | MUST NOT copy and paste logic. If two places need it, it moves to `shared/` (frontend) or `core/` (backend). |
| G7 | Every PRD rule implemented in code carries its ID in a comment, for example `// PRD FR-12.8` or `// PRD C10`. This lets us trace any requirement to its code. |
| G8 | Magic numbers MUST NOT appear. Use named constants or settings. `const DRAFT_SAVE_INTERVAL_MS = 5_000;` |
| G9 | Dates in business data are plain dates (`YYYY-MM-DD`). Times (audit, backups) are stored in UTC and shown in Pakistan time. |
| G10 | Dependencies: only those listed in the Tech Stack. Adding one needs a reason in the pull request and a licence check. |

---

## 3. Money and calculations (summary)

This is the most important section. The business is commission on sales, split between partners. A one-rupee error destroys trust.

| # | Rule | Example |
|---|---|---|
| M1 | Every business amount is a **whole rupee** (PRD Section 22). Inside the code, money is a 64-bit integer number of paisa (`Paisa(i64)` in Rust), always a multiple of 100. Never a float. | 1,050 rupees = `Paisa(105000)` |
| M2 | Percent rates are **basis points** in an integer (`Bps(i32)`, 1% = 100 bps) | 6% = `Bps(600)`, 2.5% = `Bps(250)` |
| M3 | MUST NOT use `f32`, `f64`, JavaScript `Number` arithmetic, `parseFloat`, `toFixed`, `Math.round` or SQL `REAL` for money. The one approved exception is the Excel export boundary (M13). | |
| M4 | Multiply first, divide last, in `i128`, then round once with the shared rounding function | 12,000 × 6%: 1,200,000 paisa × 600 ÷ 10,000 = 72,000 paisa = 720 rupees |
| M5 | Rounding happens in exactly one function, `core::money::round_to_rupee` (nearest rupee, half up; a fixed rule, not a setting). It is called only by `apply_bps`, `weight_value` and `avg_rate`. Splits use `split_by_bps` and `split_by_qty`. | 337.8 → 338 |
| M6 | A rounded amount is calculated once and the same value is posted to every side | Never round commission separately for the buyer bill and for the journal |
| M7 | All arithmetic on money is checked (overflow is an error, never a wrap-around) | `a.checked_add(b).ok_or(Error::Overflow)?` |
| M8 | Money from the user arrives as **text** and is parsed by the backend (`parse_rupees`). Never parsed as a float. Decimals are rejected. | `"1,200"` → `Paisa(120000)`; `"1,200.50"` → `ERR_DECIMALS_NOT_ALLOWED` |
| M9 | Splits (partners, expenses, freight) MUST add back exactly to the total. Leftover rule is fixed and tested. Negative amounts are split as positive and then negated. | 3,948 at 40/30/30 → 1,580 + 1,184 + 1,184 = 3,948 |
| M10 | Every journal entry MUST balance before it is written, and the database rejects it if not (B16) | |
| M11 | Every money function has unit tests with the PRD worked numbers and property tests (Section 9) | |
| M12 | The frontend MUST NOT calculate money. It formats paisa received from the backend, nothing else. | `formatPaisa(1200000)` → `"12,000"` |
| M13 | **Excel export exception (Umar, 23-10-2026, D5-Q1).** Excel stores every number as floating point, so money is written to a spreadsheet through ONE function, `rupees_as_f64`, in one file, `numbers.rs`, of the export code of `mandi-reports`. It divides whole paisa by 100 with integer division first, refuses a value of 2^53 or more (so the conversion is exact), and only then converts. A whole-number quantity (a نگ count) goes through its twin `count_as_f64` in the same file, with the same limit (Umar, 23-10-2026, second answer). A unit test covers the largest allowed amount (10,00,00,00,000 rupees) and the refusal. An Excel value is never read back into the app and never added up by our code, and no other code may use `f64` for money or quantities. A percent is written as text from the integer formatter, never as a float. | 12,000 rupees = `Paisa(1200000)` → cell `12000` |

---

## 4. Validation standards

### 4.1 Four layers

Validation happens in four places. Only layers 2, 3 and 4 protect the data. Layer 1 is only to help the user.

| Layer | Where | Purpose | Example |
|---|---|---|---|
| 1. On-screen hints | Frontend (Zod) | Instant feedback while typing | Qty field turns red when empty |
| 2. Command input | Backend `app/commands` + `core::validate` | Parse and check every value from the screen. Reject anything wrong. | `"abc"` as qty → `ERR_QTY_NOT_NUMBER` |
| 3. Business rules | Backend `domain` | Rules that need context | Sold qty more than lot qty → `ERR_QTY_EXCEEDS_LOT` |
| 4. Database constraints | SQLite `CHECK`, `NOT NULL`, `UNIQUE`, foreign keys, triggers | Last line of defence if code has a bug | Journal entry with Σ dr ≠ Σ cr cannot be inserted |

Rule V0: the backend MUST treat everything from the frontend as untrusted, even though both are our code.

### 4.2 Field rules

| Field type | Accepted input | Rejected | Stored as |
|---|---|---|---|
| Money | Whole rupees: digits with optional commas. Urdu digits (۰-۹) accepted and converted. Max 10,00,00,00,000 (1,000 crore) per value | Letters, any decimal point, negative, empty when required | `INTEGER` paisa (multiple of 100), column `*_paisa` |
| Rate (money per نگ or per kg) | Same as money, must be more than 0 | 0, negative, decimals | `INTEGER` paisa |
| Percent | Digits with up to 2 decimals, 0 to 100 | More than 100, more than 2 decimals | `INTEGER` bps |
| Quantity (نگ) | Whole number, 1 to 1,000,000. A simple sum such as `20+25` (PRD FR-7.48) is evaluated by the backend parser `core::validate::parse_sum`, then checked like a typed number. The same applies to money. | 0, negative, decimals, `×` or `÷`, a sum below 0 | `INTEGER` |
| Weight | Up to 3 decimals, more than 0 | | `INTEGER` grams |
| Date | `DD-MM-YYYY`, or shortcuts (PRD FR-7.19) | Impossible dates; for postings, dates on or before the "closed up to" date or after today (PRD FR-6.6) | `TEXT` `YYYY-MM-DD` |
| Account code | Digits only: control code + 4-digit serial (8 digits, or 9 for sellers). 1 to 4 digits are a short code (PRD FR-7.39), resolved by the backend on `Enter` or leaving the field: in a seller or buyer field, to the full code of that type; in a field that takes any account, to a list of the seller and the buyer with that serial, each with its type. Must exist and be active for new postings. | Inactive accounts in new postings; reserved serials for user accounts (PRD FR-10.3) | `TEXT` |
| Name (Urdu) | 1 to 100 characters: Urdu letters, digits, spaces and `. - ' ( ) / &`; at least one Urdu letter (PRD FR-7.28) | Empty, only spaces, any Latin letter, control characters | `TEXT` `name_ur` (original) + `search_ur` (normalised) |
| Name (English) | 1 to 100 characters: Latin letters, digits, spaces and `. - ' ( ) / &`; at least one Latin letter | Empty, only spaces, any Urdu letter, control characters | `TEXT` `name_en` (original) + `search_en` (lower case) + `search_phonetic` |
| Tag (نشان) | Empty, or 1 to 4 characters: English capital letters A to Z and `.`, at least one letter; lower-case letters are made capital (PRD 10.2) | Urdu letters, digits, spaces, any other sign | `TEXT` `tag` (null), added to `search_en` and `search_phonetic` |
| Phone | `03XXXXXXXXX` or `03XX-XXXXXXX` | Other lengths | `TEXT` digits only |
| Free text (notes) | Up to 500 characters | Control characters | `TEXT` trimmed |

All text input is trimmed. Unicode is normalised (NFC). Zero-width characters are removed from names.

### 4.3 Error handling

- Each error has a code (`ERR_...`) and optional values (`{"available": 5}`). The frontend turns the code into Urdu (`errors.ftl`).
- Field errors point to the exact field or grid cell, so the screen can highlight it.
- Several errors at once are returned together, so the user fixes all of them in one go.

Example response:

```json
{ "ok": false, "errors": [
  { "code": "ERR_QTY_EXCEEDS_LOT", "field": "lots[0].sales[2].qty", "values": { "available": 5 } }
]}
```

Shown as: "مقدار باقی مال سے زیادہ ہے: باقی 5 نگ ہیں" under that cell.

---

## 5. Backend (Rust) standards

### 5.1 Structure

| # | Rule |
|---|---|
| B1 | Follow the dependency direction: `app` → `services` → `domain` + `db` → `core`. `domain` MUST NOT touch the database, files, clock or network. |
| B2 | Tauri commands are thin: check that the Owner is signed in, parse input, call one service function, return the result. No business logic in commands. |
| B3 | One user action = one service function = one database transaction. Save of a بکری writes the بکری, journal, cash entries, partner split, stock, analytics, matching and audit in one transaction (PRD FR-12.8). |
| B4 | Business rules are pure functions in `domain`: inputs in, results out. This makes them easy to test. |
| B5 | The "preview" (live summary) and the "save" call the **same** domain function. There is never a second calculation path. |
| B6 | Use typed IDs (`AccountId(i64)`, `BikriId(i64)`), never plain `i64`, so an account ID cannot be passed where a بکری ID is expected |
| B7 | Enums for every fixed list (`ChargeMethod::Percent | PerUnit | PerLot`, `PaymentType::Credit | Cash`). No magic strings. |

### 5.2 Errors and safety

| # | Rule |
|---|---|
| B8 | MUST NOT use `unwrap()`, `expect()`, `panic!`, `unreachable!` or array indexing that can panic, outside tests. Use `?` and proper errors. (Clippy lints `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` set to deny.) |
| B9 | Each crate has one error enum (`thiserror`). The `app` crate maps them to `ERR_` codes. |
| B10 | No `unsafe` code |
| B11 | All integer maths on money uses checked operations (M7). Clippy `arithmetic_side_effects` is denied in `core::money` and `domain`. |
| B12 | Errors are logged once, where they are handled, with context (module, action, record ID), never with secrets |

### 5.3 Database

| # | Rule |
|---|---|
| B13 | All tables use `STRICT`. Foreign keys ON. WAL mode. `synchronous = FULL`. |
| B14 | Every SQL statement uses parameters (`?1`, `:name`). MUST NOT build SQL by joining strings with user input. |
| B15 | Money columns are `INTEGER NOT NULL` named `*_paisa` with `CHECK` where a sign is known (`CHECK (amount_paisa > 0)`) |
| B16 | Journal balance check in the database (PRD FR-6.2). SQLite triggers fire per row, so: a journal entry is inserted with `status = 'draft'`, its lines are inserted, then one `UPDATE ... SET status = 'posted'` fires a trigger that rejects the update if Σ `dr_paisa` ≠ Σ `cr_paisa` for that entry. Another trigger rejects inserting lines into a posted entry. The whole save is one transaction, so a rejected entry leaves nothing behind. |
| B17 | Append-only tables (PRD FR-6.3): `journal_lines`, `audit_log`, `partner_share_rows`, `stock_moves`, `analytics_facts`; triggers block `UPDATE` and `DELETE`. `journal_entries` allows only the one `draft` to `posted` status change. Matching tables (`bill_allocations`, `advance_allocations`) are never deleted; undoing a match sets `is_active = 0` and records who and when. |
| B18 | Every foreign key column and every column used in a filter has an index. Query plans for reports are checked (`EXPLAIN QUERY PLAN`) and kept in tests. |
| B19 | Migrations are numbered SQL files, never edited after release. Every migration is tested on a copy of real-shaped data. A backup is taken before migrations run (NFR-REL-13). |
| B20 | Stored computed values (lot net, buyer total) are written at save time and never recalculated for reports (PRD FR-11.6, FR-18.6) |
| B21 | Only one write connection exists. Reads may use a second connection. |

### 5.4 Performance

| # | Rule |
|---|---|
| B22 | No database query inside a loop over rows. Fetch in one query. |
| B23 | Reports and analytics query indexed tables or the `analytics_facts` table, never recalculate from raw lines |
| B24 | Any command expected to take more than 300 ms runs in a background task and reports progress |
| B25 | Speed targets from NFR Section 2 have benchmarks in `criterion`; a slower result fails the release check |
| B31 | A list, ledger, register, report or analytics command takes a page (a key or an offset, and a limit of 100 by default) and returns that page's rows with the row count and the totals over the whole filter, each from one indexed query. A print or export command reads every row, in the background when long (B24). Totals are never summed from a page (PRD FR-19.11, NFR-SPD-24). |

### 5.5 Names, search and assistance

| # | Rule |
|---|---|
| B26 | Every save of a named record validates both names and the tag (Section 4.2) and writes `search_ur`, `search_en` and `search_phonetic` in the same transaction. The search keys are built only by `core::text`. |
| B27 | The in-memory search index (Tech Stack 4.7) is a speed aid, never a source of truth. It is updated after the transaction commits, rebuilt at start-up and after a restore, and balances on the party card are always read from the database. |
| B28 | `core::translit` is pure and deterministic: the same word list, learned pairs and input give the same suggestions. It never panics on any input and never changes a saved name. |
| B29 | `assist_*` and `search_*` commands are read-only. They MUST NOT write anything. A rate or amount they return is a hint and is labelled as such in its type (`RateHint`), so it cannot be passed where an input value is expected. |
| B30 | Every preview result carries a `working` record for each calculated amount (formula, inputs, rule used), built by the same domain function that calculates the amount (extends B5). |

### 5.6 Example: the shape of a command

```rust
/// Saves a بکری. PRD FR-12.8.
#[tauri::command]
pub async fn bikri_save(state: State<'_, AppState>, input: BikriInput) -> Result<BikriSaved, UiError> {
    // Async command so the window never waits; blocking SQLite work runs on a worker thread.
    let db = state.db.clone();
    let user = state.require_login()?;                       // signed in and not locked (NFR-SEC-4)
    tauri::async_runtime::spawn_blocking(move || {
        let parsed = BikriDraft::parse(input)?;              // layer 2 validation, text -> Paisa
        services::bikri::save(&db, &user, parsed)            // one transaction
    })
    .await
    .map_err(UiError::internal)?
}
```

Sign-in check: `require_login()` returns the signed-in Owner, or `ERR_NOT_LOGGED_IN` / `ERR_SESSION_LOCKED`. There are no roles (PRD Section 2).

---

## 6. Frontend (TypeScript and React) standards

### 6.1 TypeScript

| # | Rule |
|---|---|
| F1 | `strict: true`, `noUncheckedIndexedAccess: true`. MUST NOT use `any`, `as unknown as`, `@ts-ignore`. |
| F2 | Types for backend data come only from `src/bindings/` (generated). MUST NOT hand-write a type that the backend owns. |
| F3 | Money values are typed as `Paisa` (a branded type). The only functions that accept `Paisa` are display formatters and sort comparisons. A lint rule blocks `+ - * / %` on `Paisa` everywhere except inside `shared/format/money.ts`, where `formatPaisa` may use integer `%` and `Math.trunc` to build the text. |
| F4 | Each module calls the backend only through its own `api.ts`. Components never call `invoke()` directly. |

### 6.2 Components

| # | Rule |
|---|---|
| F5 | Function components only. One component per file. Props typed with an interface. |
| F6 | Pages compose components; components do not fetch data themselves (hooks do) |
| F7 | Every screen uses components from `shared/ui`. MUST NOT create a one-off button, input or dialog. If something is missing, add it to `shared/ui` first. |
| F8 | Every list, grid, report and screen handles four states: loading, empty, error, data. Each has a designed look. |

### 6.3 Consistency (no visual inconsistency)

| # | Rule |
|---|---|
| F9 | All colours, spacing, font sizes, radii and shadows come from `styles/tokens.css`. MUST NOT write raw values (`#2e7d32`, `13px`) in components. Lint rule blocks arbitrary Tailwind values like `p-[13px]`. |
| F10 | Spacing uses one scale: 4, 8, 12, 16, 24, 32, 48 px (tokens `--space-1` to `--space-7`) |
| F11 | Type scale, 5 sizes each (NFR-USE-20): numbers and Latin text 14, 16, 18, 22, 28 px; Urdu Nastaliq 18, 20, 24, 28, 34 px. No other sizes. |
| F12 | Same action, same look, same place, same word, same key on every screen (NFR-USE-10). The primary action button is always in the same position. |
| F13 | Icons only from Lucide, one size per context (16 px in grids, 20 px in buttons) |
| F14 | A visual check (screenshot test) exists for each shared component and each main screen, so changes that break the look are caught |

### 6.4 Alignment

| # | Rule |
|---|---|
| F15 | Every screen uses the layout grid from `shared/ui` (12 columns, gutters from tokens). Fields line up on that grid. |
| F16 | Labels sit above fields, aligned to the field's start edge (right edge in RTL) |
| F17 | Numbers use tabular digits and sit on the **physical right edge** of their cell in every table, grid, tile and print, so units line up under units (NFR-USE-19). Number cells use `direction: ltr; text-align: end`, set once in `shared/grid` column types and the `<Num>` component. |
| F18 | In grids, column widths are set per column type (money, qty, account, date) in `shared/grid`, so the same column type has the same width everywhere |
| F19 | Totals rows align exactly under their columns |

### 6.5 Responsiveness (window sizes, not phones)

| # | Rule |
|---|---|
| F20 | Layouts MUST work from a 1024 × 600 content area up to 2560 × 1440 with no horizontal scroll anywhere in that range (NFR-USE-15, 16). The window cannot be made smaller than 1024 × 600. |
| F21 | Use flexible layouts (CSS grid, flex, `minmax`). MUST NOT give page containers fixed pixel widths. |
| F22 | Three layout steps by window width: compact (1024 to 1365), normal (1366 to 1919), wide (1920 and above). In compact, the بکری summary moves under the grids and less-used columns go behind "مزید". |
| F23 | Every screen is checked at each supported size and scaling combination in NFR-USE-17, at the smallest and largest allowed in-app zoom (NFR-USE-18) |

### 6.6 Right-to-left

| # | Rule |
|---|---|
| F24 | Use logical CSS properties only (`margin-inline-start`, Tailwind `ms-`, `pe-`, `start-`, `text-align: end`). MUST NOT use `left`/`right` or `ml-`/`pr-`. Lint rule enforces this. (Number cells reach the physical right edge with `direction: ltr; text-align: end`, so they need no exception.) |
| F25 | Every number, code, phone and truck number inside text uses `<Num>` or `<Code>` (bidi isolation, NFR-URD-13) |
| F26 | Direction-sensitive icons (arrows, chevrons, "back") are mirrored in RTL; others are not |

### 6.7 Keyboard (PRD Section 7.2)

| # | Rule |
|---|---|
| F27 | Keyboard behaviour follows PRD Sections 7.2 and 7.3 exactly. Forms, pickers and menus get it from `shared/keyboard`. Grids get it from `shared/grid`, which configures AG Grid (cell navigation, Enter skipping read-only calculated cells but stopping on editable cells the app filled, F2 not starting edit, Esc order) to match. Screens only declare their fields, order and which cells are editable; they MUST NOT write their own navigation key handlers. |
| F28 | Every interactive element is reachable by keyboard and has a visible focus outline |
| F29 | Every shortcut is registered in one place (`shared/keyboard/shortcuts.ts`) and shown on its button |
| F30 | Each screen has a test that completes its main task with the keyboard only |

### 6.8 Text and language

| # | Rule |
|---|---|
| F31 | MUST NOT write Urdu or English user-facing text in components. All text comes from Fluent files. Lint rule blocks string literals in JSX. |
| F32 | New Fluent messages are added to both `ur` and `en`, with a reviewer noted for each (NFR-URD-4, NFR-URD-21). `pnpm i18n:check` must pass (NFR-URD-22). |
| F33 | Business terms come from `glossary.ftl` only, in both languages (PRD Section 3) |

### 6.9 Performance

| # | Rule |
|---|---|
| F34 | Every list, ledger, register, report and analytics table loads its rows page by page from the backend (B31) and draws only the visible rows (AG Grid does this). Its totals row shows the backend's totals over the whole filter, never a sum of the loaded rows. Print and Excel ask the backend for every row (PRD FR-19.11). |
| F35 | Live summary calls wait 50 ms after the last key press and cancel the previous call (NFR-SPD-7) |
| F36 | Screens load their code only when opened (lazy routes) |

### 6.10 Two languages and smart assistance (PRD Sections 7.4 to 7.6)

| # | Rule |
|---|---|
| F37 | Every screen works in both directions. Layout comes only from logical properties and the direction provider. MUST NOT write `if (language === 'ur')` for layout. Screenshot tests (F14) run in both languages. |
| F38 | Names are shown only through `displayName(record, language)`, which picks `name_ur` or `name_en` and adds the tag after it ("عاشق ظفر K"). MUST NOT read `name_ur` or `name_en` directly in a component. Prints pass the print language. |
| F39 | Unsaved entry data lives in the screen's store or form state, never only inside a grid, so a language switch (which re-creates grids) loses nothing. Each entry screen has a test that switches language in the middle of an entry. |
| F40 | Every account field uses `AccountPicker`, which shows `PartyCard`. MUST NOT build a screen's own party lookup. |
| F41 | Every Urdu text field uses `RomanUrduInput`; pickers and search boxes never do (they search raw text, PRD FR-7.22). The پختہ کھاتہ name field shows one merged list (PRD FR-7.29). Every pair of name fields uses `NamePair`, which fills the other script and stops filling once the user edits it (PRD FR-7.24). |
| F42 | Every calculated amount on screen has `CalcInfo`, which shows the backend's `working` (B30). The frontend never builds a working itself. |
| F43 | A value the app fills is put into the visible field like a typed value, never into a hidden field (NFR-USE-27). Rate fields are never filled from history or hints: the last rate is shown only as placeholder text (PRD FR-7.53). Copying the seller rate to the buyer rate, `Ctrl+D`, restoring a draft, undo and opening an edit are not hints and are allowed. |
| F44 | Entries that do the same kind of job share one screen with a type list at the top (PRD FR-7.67), for example the خرچہ screen for دکان خرچہ and ذاتی خرچہ. Changing the type keeps the shared fields (date, amount, narration) in form state and clears the rest. MUST NOT build a second screen for a type that belongs on an existing one. Each type still calls its own service function and posting (B3), and its preview comes from the backend (B5, PRD FR-16.8). |

---

## 7. Database design standards

| # | Rule |
|---|---|
| D1 | Names follow Tech Stack Section 6 (snake_case, plural tables, `_paisa`, `_bps`, `_date`, `_at`) |
| D2 | Every table has `id`, `created_at`, `created_by`, `updated_at`, `updated_by`, `version` |
| D3 | Soft states only: `is_active`, `status`. No `DELETE` of business records (see B17 for the append-only tables). |
| D4 | Lookups by name use the indexed columns `search_ur`, `search_en` and `search_phonetic` (B26), never the display names |
| D5 | Each sale line stores the rule used and its value (snapshot), never only a rule ID (PRD FR-11.6) |

---

## 8. Security standards

| # | Rule |
|---|---|
| S1 | The sign-in check is the first line of every command, except first run, sign-in, recovery, app info and data status (NFR-SEC-4) |
| S2 | Passwords: Argon2id only; never logged, never returned to the frontend |
| S3 | Secrets only in Windows Credential Manager (`keyring`) |
| S4 | Tauri capabilities allow only our own commands. No shell, no general file access from the screens. |
| S5 | Content Security Policy: no remote scripts, no inline scripts, no remote fonts |
| S6 | `cargo deny` and `pnpm audit` run in CI; known vulnerabilities block release |
| S7 | Every change to money, rules, shares, the password or settings writes an audit entry in the same transaction |

---

## 9. Testing standards

### 9.1 What every change needs

| Change | Required tests |
|---|---|
| Any money function | Unit tests with PRD numbers + property tests + review by the money-calculations agent |
| New domain rule | Unit tests for normal case, each edge case in PRD Section 23 that applies, and each error |
| New service (user action) | Integration test on a temporary database: data written, journal balanced, audit written, everything in PRD Section 6 for that action updated |
| New screen | Component test for the four states (F8), keyboard-only test (F30), screenshot tests in Urdu and English (F14, F37), language switch in the middle of an entry (F39) |
| Any user-facing text | Both languages, `pnpm i18n:check`, reviewer noted for each language |
| Change to `translit` or the word list | The 1,000-name accuracy test (NFR-URD-24, NFR-URD-25) must not drop |
| Bug fix | A test that fails before the fix and passes after |

### 9.2 Golden tests

The PRD worked day (Section 4) and its end-to-end tests (Section 27) are automated:

| Tests | Where | When |
|---|---|---|
| E2E-1 to E2E-16b, E2E-13b, E2E-19 to E2E-21, E2E-28 to E2E-32, E2E-30b, E2E-34, E2E-34b, E2E-35, E2E-37, E2E-38, the preview numbers of E2E-39, E2E-40, E2E-40b, E2E-41 and E2E-42 | `backend/crates/services/tests/golden_day.rs` | Every commit |
| E2E-33 (ledger with items, R20 total), E2E-36 (day summary R34, filtered print data) and E2E-43 (party day sheet R35) | `backend/crates/reports/tests/ledger_items.rs` | Every commit |
| E2E-18 (backup and restore) | `backend/crates/backup/tests/round_trip.rs` | Every commit |
| E2E-17 (draft after closing the app), E2E-39 (the خرچہ screen) and a full keyboard run of Section 4 | Windows end-to-end suite (WebdriverIO + `tauri-driver`) | Before every release |
| E2E-22 to E2E-27, backend parts (conversion and new code in E2E-23, party card and short codes in E2E-24, memories in E2E-25, remaining qty, rate hint and working in E2E-26, open payables in E2E-27) | `backend/crates/services/tests/assist.rs` | Every commit |
| E2E-22 to E2E-27, full screen runs (including the language switch of E2E-22) | Windows end-to-end suite | Before every release |

They MUST pass. Their expected numbers MUST NOT be changed without a PRD change.

### 9.3 Property tests (money)

With `proptest`, for thousands of random inputs:

- Partner split parts always add up to the total.
- For any random sequence of postings, Σ partner share rows = income − expenses of head 05 (PRD FR-6.5, FR-17.3).
- Every built journal entry balances.
- For any sequence of sales (in بکری or another shop) and write-offs of one trip purchase, the cost released never exceeds its total cost and equals it once nothing is left (PRD C21).
- For any garden or trip: the parts of every payment add up to the payment (C23); the money credited to its partners equals its sales money (trip, C25) or its account balance at closing (garden, C26); a garden's account is 0 after closing; once a garden or a trip is closed, Σ over its partners of (money credited − parts paid from the shop − parts paid from pockets) = its result.
- Guest fruit parts add up to the value, and every part is at least the value ÷ the number of partners, rounded down (C27).
- For any random sequence of postings, Σ partner کل رقم + ابتدائی سرمایہ = the firm's equity in the trial balance (PRD FR-17.9, C22).
- Parse then format then parse gives the same paisa.
- Commission is never negative and never more than the line value for percent rules up to 100%.
- Reversal of an entry plus the entry gives zero on every account.
- Amount in words (Urdu and English) returns text for every amount from 0 to the maximum and never panics.
- `translit` and the search key builders never panic on any Unicode input, and always return at most 3 suggestions.

### 9.4 Coverage gates

| Area | Minimum |
|---|---|
| `core::money`, `domain` | 100% of lines |
| `core::translit`, `core::text`, `core::urdu_words`, `core::english_words` | 95% of lines |
| Rest of backend | 80% |
| Frontend `shared/` | 80% |

---

## 10. Git and pull requests

| # | Rule |
|---|---|
| P1 | `main` is always releasable. Work happens on branches (`feature/...`, `fix/...`). |
| P2 | Commit messages follow Conventional Commits: `feat(bikri): ...`, `fix(partners): ...`, `test(money): ...` |
| P3 | One pull request per goal run (for example D1 and D2 together), opened after the final review and robustness gate passes. It lists every plan in it. Commits inside it stay small: one per plan step, Conventional Commits. |
| P4 | The pull request description lists the PRD IDs it implements and how it was tested |
| P5 | CI must be green before merge |

---

## 11. Review checklist (Definition of Done)

A change is done only when every box is ticked:

- [ ] Implements the PRD IDs listed, nothing more, nothing less
- [ ] Formatter and linters clean, no warnings
- [ ] No floats, no JS arithmetic on money; rounding only through `core::money::round_to_rupee`; splits only through `split_by_bps` / `split_by_qty`; all amounts whole rupees
- [ ] Splits add back exactly; journals balance; audit written
- [ ] Backend validates every input; errors have codes and point to fields
- [ ] One transaction per user action; preview and save share the same function
- [ ] No `unwrap`, `expect`, `panic` outside tests
- [ ] UI uses only `shared/ui` components and tokens; aligned on the grid; tabular numbers
- [ ] Works at 1366 × 768 and 1920 × 1080, 100% and 150% scaling
- [ ] Correct in Urdu (right-to-left) and English (left-to-right); numbers isolated with `<Num>`/`<Code>`; names only through `displayName`
- [ ] Keyboard-only path works; focus visible
- [ ] All text from Fluent in both languages; Urdu and English reviewed; `i18n:check` passes
- [ ] Tests added as in Section 9; golden tests still pass
- [ ] Speed targets still met (for screens and commands on the NFR list)
- [ ] Module README and doc comments updated

---

## 12. Change log

| Version | Date | Change |
|---|---|---|
| 1.0 | 10-10-2026 | Pre-build drafts and reviews: Umar's requirements, analysis of the old software video, research, client review comments and full cross-document reviews. |
| 1.1 | 10-10-2026 | **Final baseline for build.** From the start of development, every change raises the version (1.2, 1.3 ...) and adds a line here. |
| 1.2 | 12-10-2026 | PRD v1.2: sign-in check instead of roles (B2, 5.6, S1, S7); golden tests E2E-28 to E2E-33 (9.2); one pull request per goal run (P3). |
| 1.3 | 15-10-2026 | PRD v1.3: golden tests E2E-34 to E2E-38 (9.2); property tests for the cost of own goods and the partner total (9.3). |
| 1.4 | 15-10-2026 | PRD v1.4: related entries share one screen (F44); E2E-39 (9.2). |
| 1.5 | 19-10-2026 | Approved by Umar (PRD v1.6): lists page by page (B31, F34); no mode among the shared fields (F44); golden tests E2E-40 to E2E-43 (9.2); property tests for gardens, trips and guest fruit, and the trip purchase cost (9.3). |
| 1.6 | 22-10-2026 | Approved by Umar (PRD v1.7): the tag field (4.2, B26, F38). |
| 1.7 | 23-10-2026 | Approved by Umar (D5-Q1): one exception to the no-float rule, at the Excel export boundary only (M3, M13). |
| 1.8 | 23-10-2026 | Approved by Umar: the same exception also covers whole-number quantities, through `count_as_f64` in the same file (M13). |
