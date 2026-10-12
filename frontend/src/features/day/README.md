# day (plan 10)

Closing and reopening days (PRD FR-21.1, FR-21.2), route `/day`.

- Close: the date to close up to, the preview of what exists now (days, trial balance difference,
  cash in hand), and a confirm box that says exactly what will happen.
- Reopen: a date and a required reason. Reopening a date reopens every later date too.
- After a change the day status is read again, so the status bar (state and "closed up to") and
  every entry screen follow.
- `types.ts` and `api.ts` are the only files that know the command shapes. They are interim until
  the generated bindings arrive.
