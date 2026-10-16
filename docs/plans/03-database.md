# Plan 03: Database, seed, audit, errors and logs

Deliverable: D1, plan file 03. Branch: `feature/db-schema`.

## 1. Goal

Create the full SQLite database of Architecture Section 15 in one migration, with every column, check and index. Run migrations safely (a full copy first). Give every service the one-transaction wrapper with a disk-space check. Seed the system accounts of PRD FR-10.4. Write audit rows with a keyed hash chain. Set the error model (codes, reference numbers that match log lines) and the log files. `pnpm seed` makes a dev database.

## 2. Requirements

- Architecture 15 (all tables) and 16 (error flow). Code Standards 4.3, 5.3 (B13 to B21), 7 (D1 to D5), 8 (S3, S7).
- PRD FR-10.4 (seeded accounts), FR-21.3 (writing audit rows), FR-6.2 and FR-6.3 (balance and append-only triggers).
- NFR-REL-1, NFR-REL-6, NFR-REL-7, NFR-REL-10, NFR-REL-13, NFR-SEC-5 to NFR-SEC-7, NFR-OBS-1 to NFR-OBS-3, NFR-SCL-8.
- Acceptance items: D1-A9, D1-A10, D1-A11 (audit hash part).

## 3. Worked numbers

No money calculation. Seed check: the 16 fixed system accounts of FR-10.4 (codes 01010001, 01020001, 02010001 to 02010004, 030119999, 03040000, 03049999, 04010001 to 04010005, 06010001, 06019999) plus the heads (01 to 06) and the controls the table implies (0101, 0102, 0201, 03011, 0304, 0401, 0501, 0601, 0602, 0603), with the Urdu and English names exactly as the FR-10.4 table and its note under it. `0501xxxx`, `0602xxxx` and `0603xxxx` accounts are not seeded in D1 (shop expense heads come with masters in D2; partner accounts come with partners in D3).

## 4. Contract

### 4.1 Schema

- `backend/crates/db/migrations/0001_init.sql`: every table in Architecture 15.3 to 15.7, with the system columns of 15.2 (`id`, `created_at`, `created_by`, `updated_at`, `updated_by`, `version`), `STRICT`, foreign keys, `CHECK (x_paisa % 100 = 0)` on every `*_paisa` column, the `charge_rules` checks of 15.5, the `journal_lines` check of 15.7, and every index of 15.8.
- Triggers (Code Standards B16, B17): journal entry `draft` to `posted` rejected unless Σ dr = Σ cr for that entry; no lines inserted into a posted entry; no other status change; `UPDATE` and `DELETE` blocked on `journal_lines`, `audit_log`, `partner_share_rows`, `stock_moves`, `analytics_facts`; no `DELETE` on `bill_allocations`, `advance_allocations` (only `is_active = 0` with who and when).
- Columns are exactly the Architecture names. If a column is unclear, ask the lead; never invent a name.
- `rusqlite_migration` runs numbered files. Before migrating an existing file with pending migrations, the runner copies the whole database with the SQLite online backup API to `backups/pre-migration-<from>-<to>-<utc>.db`, checks the copy, then migrates. If a migration fails, it restores the copy and returns `ERR_MIGRATION_FAILED` (NFR-REL-13).
- Connection settings (B13, B21): WAL, `foreign_keys = ON`, `synchronous = FULL`, `busy_timeout`, one write connection, reads on a second connection.

### 4.2 Transaction wrapper (`mandi-db`)

`Db::write(|tx| { ... })`: checks free disk space first (at least 50 MB, a named constant) and returns `ERR_DISK_FULL` without writing if there is not enough (NFR-REL-10); begins `IMMEDIATE`; commits only if the closure returns `Ok`; rolls back otherwise. Every service uses it (CLAUDE.md rule 3). `Db::read(|conn| ...)` for reads.

### 4.2b Data status command

`app_data_status` (no role, shown before login): input none, output `DataStatus { state: "checking" | "ok" | "failed", reference: Option<string> }`. The frontend uses it to show `DataCheckFailedState` with the reference number when the start-up check failed (NFR-REL-7). Add it to `APP_COMMANDS` in build.rs, the capability file and the capability guard test.

### 4.2c Decisions (lead, 10-10-2026)

- `day_state`: one row, id fixed to 1 by a CHECK, created by the migration; `closed_up_to_date` is NULL until a day is closed (D2).
- Urdu names for heads and controls are provisional, built from PRD words, marked `provisional, needs Urdu reviewer (NFR-URD-4)` in the seed and on Umar's review list.
- New crates approved: `getrandom` (random audit key), `fs4` (free disk space), `tempfile` (dev only). Each must pass `cargo deny check`.
- Errors shown next to a field (the user's typing mistakes) carry no reference number; errors shown for a whole form, dialog or state carry one (NFR-OBS-3).

### 4.3 Start-up check

`quick_check` runs in the background at start-up (`PRAGMA quick_check`); saving is held until it passes; on failure the app state becomes `DataCheckFailed` and every write returns `ERR_DATA_CHECK_FAILED` (NFR-REL-7). The restore offer comes in D5; in D1 the frontend shows a clear message (Fluent) with the reference number.

### 4.4 Seed

`mandi_db::seed::system_accounts(tx)`: inserts the heads, controls and accounts of Section 3 with `is_system = 1`, idempotent (a second run changes nothing). Called by the first migration run on an empty file, inside the same transaction. `pnpm seed` (`scripts/seed/`, owned by backend) creates `dev-data/mandi.db` (gitignored) with the seed, and prints the counts.

### 4.5 Audit writer

`mandi_db::audit::write(tx, AuditEntry { user_id, action, entity, entity_id, old_json, new_json, reason })`: fills `at` (UTC, from an injected clock) and `chain_hash = blake3::keyed_hash(key, previous_chain_hash || canonical_json(row without chain_hash))`. The key is a 32-byte random key in Windows Credential Manager through `keyring` (service `pk.mandisoftware.app`, entry `audit-chain-key`), created on first run. On a Mac in development, `keyring` uses the macOS keychain. Tests inject an in-memory key store. `verify_audit_chain(conn, key) -> Result<(), ChainBreak { at_id }>` finds the first edited or removed row.

### 4.6 Error model (Architecture 16, NFR-OBS-3, NFR-REL-6)

- One `thiserror` enum per crate. `mandi-app` maps each to a `UiErrorItem` code. Each `UiError` gets `ref` = `E-<yyyymmdd>-<4-digit counter per day>`, and the same `ref` is written in the log line for that error. No error closes the app; command panics are not possible because `panic` is denied, and the Tauri command runner is wrapped so an unexpected internal error becomes `ERR_INTERNAL` with a ref.
- New codes in this plan: `ERR_DISK_FULL`, `ERR_DATA_CHECK_FAILED`, `ERR_MIGRATION_FAILED`, `ERR_INTERNAL`, `ERR_NOT_IMPLEMENTED`, `ERR_DB_BUSY`.

### 4.7 Logs (NFR-OBS-1, NFR-OBS-2)

`tracing` + `tracing-appender` daily files in `<app_data_dir>/logs/`. Our clean-up at start-up keeps 30 days and at most 50 MB in total (deletes oldest first). A redaction layer: fields named `password`, `key`, `secret`, `token`, `recovery_code`, `balance` are never written (replaced with `[hidden]`). A test logs a password field and checks the file does not contain it.

### 4.8 Frontend

`frontend/src/shared/api/errors.ts`: maps a `UiError` to messages: each item's `code` to `err-<kebab>` in the current language, with its `values`; the field path to the field to highlight; the `ref` shown under general errors ("حوالہ نمبر E-20261010-0007"). Never shows the raw code (NFR-USE-14). `ErrorState` and field errors use it. Messages for the new codes in both languages.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Migration file, `Db` API signatures, audit and error types, `pnpm gen:types`. Message the lead. | backend | 02 merged or stacked |
| T2 | Runner with pre-migration copy, triggers, wrapper with disk check, quick check, seed, audit chain, error refs, logs. Integration tests: migration on an empty file; re-run is a no-op; table list equals Architecture 15 (a test that reads `sqlite_master` and compares with a list kept in the test, written from Architecture 15 by hand); seeded rows equal FR-10.4 row for row; unbalanced post rejected; posted line update rejected; audit row edited outside the app is found by `verify_audit_chain`; `ERR_DISK_FULL` path (inject a disk-space probe); a failed migration restores the copy. | backend | T1 |
| T3 | Error helper and messages; `ErrorState` shows the ref. | frontend | T1 |
| T5 | Review | reviewer | T2, T3 |
| T6 | Robustness: D1 attack list item 3 (kill during migration then restart, full disk before save, corrupted file at start-up gives a clear message and no crash) and the log secret check | robustness | T2 |
| T7 | Gate and merge | lead | T5, T6 |

## 6. Acceptance checklist

1. Every Architecture 15 table exists with its columns (test compares names and columns).
2. Re-running migrations is safe; a copy is taken before migrating an existing file.
3. Seeded accounts match FR-10.4 row for row (code, Urdu name, English name, kind).
4. Balance trigger and append-only triggers work.
5. Audit chain: an edited row is detected.
6. `ERR_DISK_FULL` saves nothing; a corrupted file gives `ERR_DATA_CHECK_FAILED` with a ref, no crash.
7. Logs: 30 days, 50 MB cap, no secrets.
8. `pnpm seed` runs.
9. `pnpm lint`, `pnpm test`, `pnpm i18n:check` exit 0.

## 7. Out of scope

Login and users (04), accounts screens (D2), backup and restore screens (D5).

## 8. Open questions

None.

## 9. v1.2 additions (12-10-2026, PRD v1.2 and Architecture 15 v1.2)

Migration 0001 is not released, so these go into `0001_init.sql` itself, with tests and the table and column list of the schema test updated:

- `bikri_lots.adjust_advance` INTEGER NOT NULL DEFAULT 1, CHECK 0 or 1 (FR-12.23, پیشگی سے کاٹیں).
- `vouchers.type` adds `personal_return`, `money_lent`, `money_lent_return`; `vouchers.mode` adds `salary`, allowed only when type is `money_lent_return` (CHECK); new columns `reason` (text, null), `return_by_date` (date text, null; on `personal_expense` and `money_lent`, PRD 16.3 and 16.5), `taken_by` (text, null, on `personal_expense` only: the person who took the cash, for example the partner's son, PRD 16.3) as Architecture 15.7 v1.2 names them. (Corrected 12-10-2026: the PRD wins over this plan's first wording.) Read the exact column list in Architecture 15.7 and use those names.
- `journal_entries.posting_type` adds `personal_return`, `money_lent`, `money_lent_return`.
- `staff.account_id` INTEGER NULL REFERENCES accounts(id) (his ادھار رقم account under control 0103).
- `advance_allocations.credit_line_id` may be a seller credit line from a بکری posting whose lot has `adjust_advance = 1`, or from a seller's receipt voucher (FR-15.12); adjust the matching trigger and its tests.
- Seed: control `0103` ادھار رقم / Money Lent under head 01 (FR-10.4 v1.2), names from the PRD table.
- `users.role`, `users.partner_id` and `partners.user_id` stay, unused (Architecture 15.3 v1.2).
