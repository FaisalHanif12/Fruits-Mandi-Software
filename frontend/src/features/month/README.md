# month (round 7, PRD FR-17.6, FR-21.2)

The month close, route `/month`. Reached from the partners overview (the "Month Close" button).

- The page is a list of the months from the first entry, newest first, each with where it stands
  (`closed`, `ready`, `days_open`, `not_finished`), and below it the month that is open.
- A month that is not closed shows the backend's own plan (`month_close_preview`, the same plan the
  close posts): the income and the shop expenses of the month, the profit between the partners and
  every line of the entry. The Close button asks in a confirm box first (FR-7.8), whose focus starts
  on Back. A month that cannot be closed yet (`ERR_DATE_IN_FUTURE`, `ERR_DAY_NOT_CLOSED` with the day
  the days are closed up to) is told in words with a link to the Day screen.
- A closed month shows when it was closed and how many times a close of it was reversed. Reverse asks
  for a reason in the shared reason box (`ERR_REASON_REQUIRED` under the box, `ERR_NOT_FOUND` when no
  close is in force). The reversal is a mirror entry dated today.
- A close changes no partner's کل رقم (C22); it moves amounts between his accounts. The screen
  shows what the backend sent and adds nothing up.
- The month screen has its own words for five refusals whose general words belong to the day screen
  (`text.ts` `monthErrorId`, `month.ftl`). The others use `errors.ftl`.
- After a change the list, the preview and the partner screens are asked again.
- `types.ts` re-exports the generated bindings and `api.ts` is the only file that knows the command
  shapes. Tests: `month-api-contract.test.ts` (fixtures, request shapes), `month-screen.test.tsx`
  (the screen on the backend's own answers, ur and en). Windows spec: `46-reports-month.e2e.mjs`.
