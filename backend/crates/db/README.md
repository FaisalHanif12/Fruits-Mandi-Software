# mandi-db

SQLite: the connection, numbered migrations, the transaction wrapper, the seed and the audit writer (Code Standards 5.3, Architecture 15, plan 03).

- May depend on: `mandi-core` only.
- Released migrations are never edited. Add a new numbered file to `migrate::migrations()`.

## What is here

| Part | What it does |
|---|---|
| `migrations/0001_init.sql` | Every table of Architecture 15 (37), STRICT, with `CHECK (x_paisa % 100 = 0)` on every money column, every index of 15.8, and the triggers of B16 and B17. |
| `migrations/0002_balance_indexes.sql` | Indexes only, no table and no stored total (PRD FR-6.1): a covering index `(account_id, entry_id, dr_paisa, cr_paisa)` so a balance is summed from the index alone, a partial index on the unposted entries (empty outside a posting), and a partial index on the lines flagged as an advance. The two indexes of Architecture 15.8 stay. A balance leaves out the few unposted entries instead of looking up the entry of every line; `tests/balance_queries.rs` keeps the old join as the reference and checks the same numbers and the query plans. A balance for a day on a small account (up to `PER_LINE_UP_TO_LINES` lines) still looks at the entry of each line, because leaving out the entries after an early day would list every entry of the journal; `ledger::day_method` picks the way and both give the same answer. The date of the last entry of an account (`ledger::last_entry_date`, the party card) reads the newest 1,000 lines of a busy account, then the entries dated after that (the cash account has half a million lines but a buyer served yesterday has a few days of later entries). An account whose newest lines are old has a million later entries, so when more than `LAST_DATE_LATER_ENTRIES` (50,000) entries are dated after them the join over its own lines is used instead (measured on the NFR-SCL-1 volume: a dormant account with 1,000 to 60,000 lines went from 230 to 430 ms to 1.5 to 20 ms on the card). The answer is the old join's in every case. |
| `migrations/0003_system_accounts_stay_active.sql` | One trigger, no table, column, index or row change: a system account cannot be made inactive (`ERR_SYSTEM_ACCOUNT`, PRD FR-10.4, Code Standards 4.1 layer 4). Making one active, and every other edit of it, stays allowed. `services::accounts` refuses first with the field; the trigger is the last line of defence. |
| `migrations/0004_bikri_records_are_never_deleted.sql` | Seven triggers, no table, column, index or row change (plan 13, lead decision): no `DELETE` on `bikri_entries`, `bikri_lots` and `bikri_sales`, a بکری keeps its number once it exists (`ERR_APPEND_ONLY`), and `INSERT OR REPLACE` cannot remove a row (`no_replace`, also with `recursive_triggers` off). A cancelled بکری keeps its rows. What plans 16 need stays allowed: `status`, `cancel_reason`, `cancelled_by`, `cancelled_at` and `print_count` of a بکری, the running totals and `qty_written_off` of a lot, `is_active` and `replaced_by_sale_id` of a line. `tests/migration.rs` upgrades a populated version 3 file, keeps every row and number, and checks the refusals and the allowed updates. |
| `Db` (`db.rs`) | One write connection, one read connection. WAL, `foreign_keys = ON`, `synchronous = FULL`, `recursive_triggers = ON`, busy timeout. |
| `Db::write(\|tx\| ...)` | One user action = one call = one `IMMEDIATE` transaction. Checks the start-up state and 50 MB of free disk first. Commits only if the closure returns `Ok`. |
| `Db::read(\|conn\| ...)` | Reads on the second connection. |
| `migrate` | Runs the migrations. Before it changes an existing file it copies it with the online backup API to `<backups>/pre-migration-<from>-<to>-<utc>.db`, checks the copy, and puts it back if the upgrade fails (`ERR_MIGRATION_FAILED`). |
| Start-up check | `Db::spawn_quick_check` runs `PRAGMA quick_check` in the background. Saves wait for it. On failure every save returns `ERR_DATA_CHECK_FAILED`. |
| `seed::system_accounts` | The 6 heads, 10 controls and 16 system accounts of PRD FR-10.4. Runs inside the first migration. Idempotent. |
| `audit` | `Auditor::write` adds an audit row inside the caller's transaction with a keyed blake3 hash chain. `verify_audit_chain` finds the first edited or removed row. The key lives in Credential Manager (`keyring`). |
| `bikri` | The بکری tables (plans 12 and 13). `bikri` (mod.rs) only reads for the entry screen: the facts of the accounts, items and masters a draft names (one statement for a list of ids), `last_rate`, `item_sales_between`, the memories of trucks and sellers. `bikri::write` adds the rows of a save (number, entry, lots with their final totals, sale lines with their snapshots, stock moves, analytics facts). `bikri::saved` reads a saved بکری (all stored values), the list and find, first / previous / next / last, the one-time people and the register lines. |
| `matching` | The matching rows (plan 13): the live lines of a key (not reversed, not a reversal), the open bills, unmatched credits and open advances with what is already matched, `has_later_entry`, and the inserts of `bill_allocations` and `advance_allocations` (the undo of a bill row sets `is_active = 0` with who and when). SQL only. |
| `drafts` | The `drafts` table: keep, read and delete the draft of a screen. |
| `partners` | Partners, their share history and the append-only `partner_share_rows` (plan 11): `shares_on` (each partner's latest row on or before a day, the row saved last on a tie), `insert_share_row`, `share_sums` and `taken_between` (SUM on INTEGER columns; a reversal's credits are taken off a partner's money taken). No rules here: the split is in `mandi_domain::partners`, and the only code that writes share rows is the hook in `services::ledger::post`. |
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
