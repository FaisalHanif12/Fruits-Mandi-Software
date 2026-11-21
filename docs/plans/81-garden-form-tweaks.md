# Plan 81: New garden form tweaks

| | |
|---|---|
| Status | Umar's request, 20-11-2026 |
| Branch | `fix/gardens-new-form` from `main` (c3b89303) |
| PRD | 12.11 (FR-12.34, FR-12.35), FR-12.29 |

## Goal

On the new garden form (نیا باغ):

1. Remove the input "پہلی قسط کیوں دی گئی" (reason of the first installment). A first installment typed on the form is saved with the standard reason words (`gardens-first-installment-reason`, "پہلی قسط"), with no field for it.
2. Season end (موسم کا اختتام) and note (نوٹ) are optional. Leaving them empty must never stop the save. A season end may be a future date (it is the end of the season, not a posting date); it must not be before the garden's date.
3. The "باغ بنائیں F10" button must work when the required fields are filled. Today it stays disabled: the share total box keeps showing "جانچ ہو رہی ہے" (checking) with "—", so `canSave` never becomes true (`NewGardenPage.tsx:143`). Find why the shares check never answers (a refused or failing command, a changed input type, a hook that never runs) and fix the cause.

Required to create a garden: name (both scripts), date, rent, shares totalling 100% with a source per partner. Optional: items, first installment, season end, note.

## Contract changes

- `GardenInput.paid_now_reason`: no longer sent by the screen. The backend uses the standard reason words when `paid_now` is given and no reason is sent (remove the `ERR_REASON_REQUIRED` on `paid_now_reason` in `services/src/gardens/make.rs:244`). Keep the field optional in the type so old callers still work.
- `season_end`: checked as a date not before the garden's date; a future date is allowed.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: reason default, season end future allowed, find any backend cause of the stuck shares check; tests | backend |
| T2 | Frontend: remove the reason field, keep season end and note optional, fix the stuck shares check and the disabled button; tests in ur and en | frontend |

## Acceptance

- A garden is created with only name, date, rent and 100% shares; season end, note and first installment empty.
- With a first installment of 300,000 and no reason, the garden and its first installment are saved; the installment's reason is the standard words.
- A season end after today is accepted; one before the garden's date is refused at `season_end`.
- The share total shows 100% (not "checking") as soon as the shares are typed, and the button turns on.
- Every fix has a test that fails before and passes after. Golden tests stay green. `pnpm typecheck`, lint, `pnpm i18n:check` and the touched test files pass.

## Item 4 (Umar, 20-11-2026): no raw message id on any screen or print

The partner month statement print showed `print-partner-month-col-advances` in Urdu mode (the advances column of FR-17.12 and FR-17.13 had no Fluent message). Every message id the screens, prints and backend labels can produce must exist in ur and en (NFR-URD-1, NFR-URD-22). Owner: frontend-2 (Fluent files, a guard test that fails on any missing id, a check of backend label ids), backend for any backend id.
