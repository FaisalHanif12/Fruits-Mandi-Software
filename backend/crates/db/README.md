# mandi-db

SQLite: the connection, numbered migrations, the transaction wrapper, the seed and the audit writer (Code Standards 5.3, Architecture 15, plan 03).

- May depend on: `mandi-core` only.
- Released migrations are never edited. Add a new numbered file to `migrate::migrations()`.

## What is here

| Part | What it does |
|---|---|
| `migrations/0001_init.sql` | Every table of Architecture 15 (37), STRICT, with `CHECK (x_paisa % 100 = 0)` on every money column, every index of 15.8, and the triggers of B16 and B17. |
| `Db` (`db.rs`) | One write connection, one read connection. WAL, `foreign_keys = ON`, `synchronous = FULL`, `recursive_triggers = ON`, busy timeout. |
| `Db::write(\|tx\| ...)` | One user action = one call = one `IMMEDIATE` transaction. Checks the start-up state and 50 MB of free disk first. Commits only if the closure returns `Ok`. |
| `Db::read(\|conn\| ...)` | Reads on the second connection. |
| `migrate` | Runs the migrations. Before it changes an existing file it copies it with the online backup API to `<backups>/pre-migration-<from>-<to>-<utc>.db`, checks the copy, and puts it back if the upgrade fails (`ERR_MIGRATION_FAILED`). |
| Start-up check | `Db::spawn_quick_check` runs `PRAGMA quick_check` in the background. Saves wait for it. On failure every save returns `ERR_DATA_CHECK_FAILED`. |
| `seed::system_accounts` | The 6 heads, 10 controls and 16 system accounts of PRD FR-10.4. Runs inside the first migration. Idempotent. |
| `audit` | `Auditor::write` adds an audit row inside the caller's transaction with a keyed blake3 hash chain. `verify_audit_chain` finds the first edited or removed row. The key lives in Credential Manager (`keyring`). |
| `DbError` | The one error type. `DbError::code()` gives the `ErrorCode` for the screen. |

## Rules to remember

- Trigger messages are `ERR_` tokens. `DbError` turns them into variants (`JournalUnbalanced`, `JournalPosted`, `AppendOnly`, `SystemAccount`).
- A journal entry is inserted as `draft`, its lines are inserted, then one `UPDATE ... SET status = 'posted', total_paisa = ...` posts it. The trigger checks the lines, the balance and the total. `chain_hash` must be set in that same update, because a posted row never changes.
- A saved `bikri_sales` row is frozen. An edit marks it `is_active = 0` and sets `replaced_by_sale_id`.
- Tests open files, never `:memory:`, because the read and write connections must see one database.

## Dev database

```bash
pnpm seed            # creates dev-data/mandi.db (ignored by git) and prints the counts
pnpm seed -- --reset # starts again
```
