# day (plan 10)

Closing and reopening days (PRD FR-21.1, FR-21.2), route `/day`.

- Close: the date to close up to, the preview of what exists now (first open day, days,
  entries, trial balance totals and difference, cash in hand), and a confirm box that says exactly what will happen.
- Reopen: a date and a required reason. Reopening a date reopens every later date too.
- After a change the day status is read again, so the status bar (state and "closed up to") and
  every entry screen follow.
- `types.ts` re-exports the generated bindings and `api.ts` is the only file that knows the command
  shapes. With every day up to today closed, the close card says there is nothing to close and asks
  the backend nothing.
