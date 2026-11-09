# party_ledgers (plan 66, PRD FR-12.43)

پارٹی کھاتے, Party Ledgers: each buyer's and each seller's own ledger, in three levels. It is the
sixth item of the بکری side menu, above باقی مال, at `/bikri/party-ledgers`. The screen only reads: it
changes nothing and works out no figure (CLAUDE.md rule 1).

| Level        | Address                         | What it shows                                                                                                                                                                                                                                                            | Where the figures come from                                                              |
| ------------ | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| 1. List      | `?side=buyer` or `?side=seller` | One row for each party by code: code, name, the balance under بنام or جمع. A person in the dead amount is marked ڈوبی. The footer is the plain total of the debit and of the credit.                                                                                     | `party_ledgers_list`                                                                     |
| 2. Statement | `?party=20`                     | One row for each date with an entry, newest first: the balance before the day, what the day charged, what was paid, the balance after it, each balance with its side word. Two date boxes narrow it. The footer is the balance before, the totals and the balance after. | `party_ledgers_statement`                                                                |
| 3. Day slip  | `?party=20&date=2026-10-10`     | The full R35 slip of that party and that day, and its print preview (F9).                                                                                                                                                                                                | `reports_run` and `reports_print_data` for R35, by `reports/components/PartyDaySlip.tsx` |

## Rules the screen keeps

- The level is in the address. Esc and the back buttons are plain steps back (slip, statement, list,
  then the dashboard), a language switch keeps the level, and another screen can open a party's
  ledger with `?party=`. A bad address shows level 1 (`search.ts`).
- Enter (or a double click) on a row opens it. Everything works with the keyboard alone.
- Every list loads page by page (25, 50 or 100 rows) and the totals are the backend's over the whole
  filter, never a sum of the rows on screen (PRD FR-19.11, Code Standards F34).
- A date the backend refuses (a half-typed one) shows its words under its own box and the rows
  already on screen stay.
- Level 3 is not rebuilt here. The slip, its print and its Excel sheet are the report's own, asked
  with the same filters the report screen sends when a party is picked in its box
  (`reports/party-day.ts`), so no figure can differ from R35, the party card or R13.
- The four states of every level: loading, an error with a retry (also for a party that has no
  ledger), nothing yet or nothing found (with a way out), and the rows.
- Both languages and both directions from one layout: logical properties only, names through
  `displayName`, numbers and codes through the grid's cell types.
