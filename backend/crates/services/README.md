# mandi-services

One function and one database transaction per user action (Code Standards B3, CLAUDE.md rule 3).

- May depend on: `mandi-domain` and `mandi-db`.
- Golden tests of the PRD worked day go in `tests/golden_day.rs` (Code Standards 9.2). `opening_position()` there builds PRD 4.3 through the real services with an injected clock; later events build on it.
- `cargo run -p mandi-services --example seed_dev -- <db>` (run by `pnpm seed`) builds the same position in a development database.
- Never call `ctx.db.read` inside another `ctx.db.read`: the read connection is not re-entrant. Read what you need first, then open the read.
- A `Context` holds the database, the clock (from the `Db`), the password hasher, the random source, the one signed-in session and the lockout memory. The app builds one at start-up; tests build their own with `testing::TestEnv`.
- Every command starts with `auth::require(ctx)`: the Owner must be signed in and the app not locked (there are no roles, PRD Section 2). A refused call writes only an audit row `denied`.
- Writes go through `ctx.db.write(|tx| ...)`: the change and its audit row (`ctx.db.auditor().write(tx, ...)`) commit together.
- Slow work (Argon2id) happens before the write lock is taken.

## Modules

| Module | What it does |
|---|---|
| `auth` | First run, sign-in, unlock, lock, logout, recovery, `require`. Lockout steps of PRD FR-8.2 with a monotonic memory (NFR-REL-11). |
| `settings` | Firm details, idle lock minutes, default language, per-user preferences. |
| `translit` | Name suggestions (read only) and the learned word pairs: learn (insert or one more use, with an audit row), list and remove (Owner). The pairs are kept in memory and loaded from `translit_pairs` the first time; `reload` after a restore. |
| `day` | Today and the closed-up-to date (PRD FR-6.6); day close preview, close and reopen (FR-21.1, FR-21.2): days close oldest first, a close needs a balanced trial balance, a reopen needs a reason and opens that day and every later day. |
| `masters`, `search` | The master lists and the in-memory search index (plans 06, 08): codes, short codes, close spellings, `resolve_code`, row balances. |
| `accounts`, `coa` | Accounts, openings (one balanced entry against ابتدائی فرق for each account, FR-10.9), the chart of accounts with live balances (plan 07). |
| `ledger` | The journal engine: `post`, `reverse` (also switches off matching rows that rest on the reversed lines), trial balance, account ledger, balances (plan 07). |
| `assist` | Read-only help: the party card and a seller's open advances (plans 08, 10); truck memory, seller memory and the last saved rate of an item (plan 12). Nothing here writes. |
| `rules` | Commission rules: the six levels, the rule form checks, "test a rule", the three defaults in Settings, the overrides (plan 09). The rules in memory are thrown away after every change. |
| `partners` | Partners and their shares (plan 11, PRD 17.1 to 17.3): add and edit a partner (his capital account 0602xxxx and drawings account 0603xxxx are made with him, in both scripts), set the shares of all active partners from a day (exactly 100%, never back-dated, history kept), list, read, overview. `partners::rows` is the hook that `ledger::post` calls: it writes the share rows of every posting that touches head 04 or 05, so no module writes share rows itself (FR-6.5). A reversal copies the original rows with the opposite sign. `partners::split_entry` gives the rows a new posting would write; the بکری preview and the save both use it (B5). |
| `bikri` | The بکری entry screen (plan 12): `preview` reads the facts of the draft in one go, calls `domain::bikri::calc_posting`, builds the journal entry, splits the income with `partners::split_entry` (the income is never added up here) and reads cash in hand. Read only. Plan 13 adds the save, which calls the same domain functions. |
| `cash` | `cash_after_entry(conn, change)` and `require_confirmed(check, confirmed)` (`ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`): the one place every cash-out action asks whether cash goes below zero (plans 13, 15, 16, 16b, 17). The change comes from the journal entry (`domain::cash::net_change`). |
| `vouchers` | The voucher framework: numbering per type, the common fields, the running list and the پیشگی voucher (plan 10). |
| `ports` | `Monotonic`, `RandomSource`, `PasswordHasher` (Argon2id) behind traits. |
| `testing` | Fake parts for tests. Never used by the running app. |

## Rules to remember

- Partner share rows (`partners::rows`, written by `ledger::post`): for a period, "the sum of the shares = income - shop expenses" (PRD FR-6.5) holds only if the income and expense side of the books leaves out `month_close` entries and their reversals. A month close empties the income and expense balances on purpose and writes no share rows. Plan 17 and D5 must leave those entries out when they check the identity or build a report from the journal.
- A reversal has the date of the entry it reverses, except the reversal of a month close. `ledger::post` refuses any other (`partners::rows::plan`), because `db::partners::taken_between` and the monthly share sums rely on it.

- Passwords and codes are `Secret`: they print as `Secret([hidden])` and never go into an audit row or a log line.
- A wrong user name behaves like a wrong password (same error, same counting, same time).
- Lot gross, deductions and net must be written in one UPDATE (the database checks `net = gross - deductions`).
