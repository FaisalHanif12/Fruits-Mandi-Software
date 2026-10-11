# mandi-services

One function and one database transaction per user action (Code Standards B3, CLAUDE.md rule 3).

- May depend on: `mandi-domain` and `mandi-db`.
- Later plans add the in-memory search index and the read-only `assist` queries here.
- Golden tests of the PRD worked day go in `tests/golden_day.rs` (Code Standards 9.2).
- A `Context` holds the database, the clock (from the `Db`), the password hasher, the random source, the one signed-in session and the lockout memory. The app builds one at start-up; tests build their own with `testing::TestEnv`.
- Every command starts with `auth::require(ctx, "<command>", Role::X)`. It reads the role from the database, writes nothing for "not signed in" or "locked", and writes only an audit row `denied` for a role that is too low.
- Writes go through `ctx.db.write(|tx| ...)`: the change and its audit row (`ctx.db.auditor().write(tx, ...)`) commit together.
- Slow work (Argon2id) happens before the write lock is taken.

## Modules

| Module | What it does |
|---|---|
| `auth` | First run, sign-in, unlock, lock, logout, recovery, `require`. Lockout steps of PRD FR-8.2 with a monotonic memory (NFR-REL-11). |
| `settings` | Firm details, idle lock minutes, default language, per-user preferences. |
| `translit` | Name suggestions (read only) and the learned word pairs: learn (insert or one more use, with an audit row), list and remove (Owner). The pairs are kept in memory and loaded from `translit_pairs` the first time; `reload` after a restore. |
| `day` | Today and the closed-up-to date (PRD FR-6.6). |
| `ports` | `Monotonic`, `RandomSource`, `PasswordHasher` (Argon2id) behind traits. |
| `testing` | Fake parts for tests. Never used by the running app. |

## Rules to remember

- Passwords and codes are `Secret`: they print as `Secret([hidden])` and never go into an audit row or a log line.
- A wrong user name behaves like a wrong password (same error, same counting, same time).
- Lot gross, deductions and net must be written in one UPDATE (the database checks `net = gross - deductions`).
