# Plan 85: A new بکری on a closed day, auto close that waits, and lines that say why

| | |
|---|---|
| Status | Umar's request and decisions, 20-11-2026 |
| Branch | `fix/gardens-new-form` (one working tree; one pull request for plans 81 to 85) |
| PRD | FR-21.4 (v1.49 new bullets), FR-21.1, FR-21.2, FR-12.6 (v1.49 line), FR-6.6, FR-7.71 |

## Root cause (investigation, 20-11-2026)

- The automatic day close (setting `auto_day_close` on, closing time 21:00) closed 17-11 to 20-11 at 21:40, one minute after login (audit rows 349 and 350). Today became a closed day.
- A closed header date makes `prepare_draft` give ERR_DATE_CLOSED (`core/src/bikri/prepare.rs:373`), so the domain has no date (`domain/src/bikri/mod.rs:270`), no lot is complete (`process.rs:144`), and `calc_line` never runs (`process.rs:619`). Every line stays empty and the summary says "ابھی حساب نہیں ہوا". The only words are a small note under the date (`HeadFields.tsx:47`) with no button.
- The save is refused the same way (`save.rs:61`). No reopen is offered for a new بکری (the notice and `useFixDayFlow` serve edits only, `BikriBookPage.tsx:678, 1052`).
- `app/src/day_auto.rs` ignores `day_fix_pending` and a manual reopen: after closing time it closes a reopened day again within 5 minutes (its TICK), which also breaks a fix in progress.
- Older silent bug: a lot missing its seller or packing makes its lines empty with no message until a save try (`problems.ts:17-25, 79` hold the error back; `grid-views.ts:236-241` drops the line note).

## Umar's decisions

1. A new بکری on a closed day: reopen with the sign, save, close again (FR-21.4 flow).
2. Auto day close never closes a day that is in use.
3. The reason a line has no figures shows at once, on the lot; a closed date shows the notice with the button.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: `fix_open` and `fix_close` usable for a new entry on a closed date (no entry id needed; the token holds the days); the sign and reason in the audit; `day_auto.rs` skips a day with `day_fix_pending`, a day reopened by hand after its last close (remember the manual reopen until the Owner closes it), and a day with a saved draft; tests | `backend-day` |
| T2 | Frontend: on the new بکری screen (and the other new-entry screens that post on a date: vouchers, خرچہ, پھل, ادھار رقم, الگ کھاتہ, سرمایہ, garden and trip entries) a closed date shows the closed-day notice with "دن کھولیں اور محفوظ کریں" (reason and sign box, reusing `useFixDayFlow`), the figures fill after the reopen, the save closes the days again; the lot row says what is missing at once when its lines cannot be worked out (seller, item, packing, received qty, weight); tests in ur and en | `frontend-day` |

## Acceptance

- With today closed, typing a بکری shows the notice; reopen with reason and "adnan" fills the figures; save posts on today; the days close again; the audit has the reopen, the save and the close with the sign; a wrong sign opens nothing.
- With auto close on and its time passed, a day reopened by hand, or held by a fix, or with an unsaved entry, is not closed by the timer; it closes when the Owner closes it or the work ends.
- A lot with no packing (new seller with no last lot) shows "پیکنگ چنیں" (or the right words) under its packing cell at once when a line has qty and rate.
- Golden tests stay green; every fix has a test that fails before; clippy, typecheck, lint, i18n:check pass.
