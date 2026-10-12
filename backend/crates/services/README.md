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
| `assist` | Read-only help: the party card and a seller's open advances (plans 08, 10). Nothing here writes. |
| `rules` | Commission rules: the six levels, the rule form checks, "test a rule", the three defaults in Settings, the overrides (plan 09). The rules in memory are thrown away after every change. |
| `vouchers` | The voucher framework: numbering per type, the common fields, the running list and the پیشگی voucher (plan 10). |
| `ports` | `Monotonic`, `RandomSource`, `PasswordHasher` (Argon2id) behind traits. |
| `testing` | Fake parts for tests. Never used by the running app. |

## Rules to remember

- Passwords and codes are `Secret`: they print as `Secret([hidden])` and never go into an audit row or a log line.
- A wrong user name behaves like a wrong password (same error, same counting, same time).
- Lot gross, deductions and net must be written in one UPDATE (the database checks `net = gross - deductions`).
