# party_ledgers (plans 66 and 74, PRD FR-12.43, FR-12.47)

پارٹی کھاتے, Party Ledgers: each buyer's and each seller's own ledger, in three levels. It is the
sixth item of the بکری side menu, above باقی مال, at `/bikri/party-ledgers`. The screen only reads: it
changes nothing and works out no figure (CLAUDE.md rule 1).

| Level        | Address                             | What it shows                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Where the figures come from                                                                |
| ------------ | ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| 1. List      | `?side=buyer` or `?side=seller`     | One row for each party by code: code, name, the balance under بنام or جمع. A person in the dead amount is marked ڈوبی. The footer is the plain total of the debit and of the credit.                                                                                                                                                                                                                                                                                                                                                                                                                                      | `party_ledgers_list`                                                                       |
| 2. Statement | `?party=20` (and `&end=2026-09-10`) | One row for each date with an entry, newest first, **30 days at a time**: the balance before the day, what the day charged, what was paid, the balance after it, each balance with its side word. A date opens its day **under its own row** (the R35 slip, with its own print button). "پچھلے 30 دن" and "اگلے 30 دن" under the table move the window, with its days between them; Page Up and Page Down do the same. "تاریخ پر جائیں" opens a date's window with the row open. Two date boxes (از, تا) make it a range, page by page. The footer is the window's: the balance before, the totals and the balance after. | `party_ledgers_statement`, `party_ledgers_go_to_date`; the open day: `reports_run` for R35 |
| 3. Day slip  | `?party=20&date=2026-10-10`         | The full R35 slip of that party and that day, and its print preview (F9).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | `reports_run` and `reports_print_data` for R35, by `reports/components/PartyDaySlip.tsx`   |

## Rules the screen keeps

- The level is in the address. Esc and the back buttons are plain steps back (slip, statement, list,
  then the dashboard), a language switch keeps the level, and another screen can open a party's
  ledger with `?party=`. A bad address shows level 1 (`search.ts`).
- Enter (or a double click) on a row of the list opens it. In the statement, Enter or a click opens a day
  in place and a second one shuts it (`StatementDays`: one tab stop, ↑ ↓ Home End, the arrow at the
  children opens, the other shuts, from `shared/keyboard`). Several days can be open; moving the window
  shuts them. Everything works with the keyboard alone.
- The statement window is `statement-window.ts` (30 days that end today at first; the later button stops
  at today). Its end is in the address (`?end=`, nothing for today). The window is one read of the same
  command as a range, so its totals are the backend's. Page Up and Page Down move it, but not while the
  keys are inside an open day.
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
