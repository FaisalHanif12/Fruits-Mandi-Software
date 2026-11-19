# day (plan 10)

Closing and reopening days (PRD FR-21.1, FR-21.2), route `/day`.

- Close: the date to close up to, the preview of what exists now (first open day, days,
  entries, trial balance totals and difference, cash in hand), and a confirm box that says exactly what will happen.
- The summary of the days (plan 17, PRD FR-21.1, 4.9) is the backend's `DayCloseSummary`, shown by
  `DaySummary`: the number of بکری, خام بکری, the commission, the shared income, the lots left in
  باقی مال, and each partner's income, expense and profit share. The screen adds nothing up. Tests:
  `day-close-real-shapes.test.ts` (the canned answer has the real shape, the request equals the
  backend's own) and `day-close-real-answers.test.tsx` (the screen on the real answer, ur and en).
- Reopen: a date and a required reason. Reopening a date reopens every later date too.
- After a change the day status is read again, so the status bar (state and "closed up to") and
  every entry screen follow.
- `types.ts` re-exports the generated bindings and `api.ts` is the only file that knows the command
  shapes. With every day up to today closed, the close card says there is nothing to close and asks
  the backend nothing.
