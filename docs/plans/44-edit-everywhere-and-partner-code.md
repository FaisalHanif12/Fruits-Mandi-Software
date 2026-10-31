# Plan 44: edit every entry safely, the edit stamp, accounts on any day, and the partner code

Version 1.0, 31-10-2026. Lead: tech-lead. Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`.

Umar's words, in short (31-10-2026):
- "A mistake found later, for example in the seller amount or the item, must be corrected on that day's entry. Direct commission, direct total sale and direct capital stay as they are."
- His answers:
  1. The date of an entry cannot be changed.
  2. Closed days: reopen, fix, close again.
  3. A sale of any day can be edited.
  4. A trip purchase cannot be edited once its goods are sold.
- "Once edited, make sure it is recalculated."
- "There should be a timestamp of the edit and who made it, displayed where it was edited."
- "We must be able to add a seller or buyer even when the day is closed."
- "A 4-digit code to open the partner section, set up once, asked every time."

## 1. Requirements

PRD v1.13:
- FR-7.70 (one edit model) and FR-7.71 (the edit stamp);
- FR-21.4 (reopen and fix in one step), FR-12.18, FR-15.3;
- FR-10.11 (accounts on any day);
- FR-17.11 (partner code), Section 21 Security;
- E76 to E80, E2E-45, E2E-46, A75 to A78.

Also:
- NFR v1.7: NFR-SEC-14.
- Code Standards v1.12: S8, S9, 9.2.
- Architecture v1.11: 15.3 settings keys.

Kept as they are:
- FR-6.3 (reversal, never overwrite);
- FR-12.19 to FR-12.21 (edit by reversal, snapshots kept);
- FR-21.1 and FR-21.2 (close and reopen);
- FR-21.3 (audit);
- FR-8.2 (wait after wrong tries).

## 2. Worked numbers

E2E-45: after Events 1 to 5, 10-10-26 is closed. بکری 1 is edited through "دن کھولیں اور درست کریں". اکرم's rate goes from 600 to 610.

| Figure | Before | After |
|---|---|---|
| اکرم's line | 20 × 600 = 12,000 | 20 × 610 = 12,200 |
| Buyer commission 6% | 720 | 732 |
| اکرم pays | 12,880 | 12,200 + 732 + 160 = 13,092 |
| خام بکری | 29,400 | 29,600 |
| Seller commission | 1,764 | 732 + 1,044 = 1,776 |
| کل اخراجات | 10,084 | 10,096 |
| صافی بکری | 19,316 | 19,504 |
| Shared income | 3,948 | 1,776 + 1,776 + 400 + 20 = 3,972 |
| Split | 1,580 / 1,184 / 1,184 | 1,590 / 1,191 / 1,191 |

What else must hold after the edit:
- The date stays 10-10-26.
- The closed-up-to date is 10-10-26 again.
- The entry shows "تبدیل شدہ" with the time and Owner, and line 1's rate is marked "پہلے 600".
- The audit log has the reopen, the edit with its reason, and the close again.

These are exactly the E2E-13 numbers. No new money rule is needed.

E2E-46: after Events 1 to 5, the partner code is 4826.
- Partner commands and R31 are refused with ERR_PARTNER_LOCKED.
- The code 1111 is refused and logged.
- With 4826, the overview shows profit shares 996 / 746 / 746.
- Leaving the section locks it again.

## 3. Contract (T1 writes it exactly; I record it here when T1 is done)

- **Edit coverage.** Every entry type in the FR-7.70 table has a preview of the edit and a save of the edit. Existing commands are kept:
  - `bikri_open_for_edit`, `bikri_edit_preview`, `bikri_edit_save`, `vouchers_edit`, `fruit_edit`.
  - New ones are only for the gaps found in T0, for example a trip purchase, a trip cost, a garden entry, or a sale in another shop.
  - Every edit preview returns the list of changes (field, before, after) and the effect (balances, cash, partner shares). Both come from the same function as the save (B5).
  - A changed date is refused: `ERR_EDIT_DATE_FIXED` on the date field.
  - An edit resting on moved goods is refused with `ERR_EDIT_NOT_ALLOWED` and a rule name, as today.
- **Stamp and history.**
  - `audit_edit_stamp(entity, ids[])` returns, for each id: last edited at, by (display name), count, and the last reason.
  - `audit_edit_history(entity, id)` returns the versions with field-level old and new values.
  - Both are read-only and come from `audit_log`.
  - List and report rows carry an `edited` flag with the time.
  - Print data carries a `revised_on` date.
- **Reopen and fix.**
  - `day_fix_open(date, reason)` reopens the date and every later closed date, and returns a fix token with the list of days.
  - `day_fix_close(token)` closes the same days again in order, with the FR-21.1 checks. It returns the days that stayed open, and why.
  - A month-closed month is refused.
- **Accounts on any day.**
  - `masters_save` and `accounts_save` without an opening balance do not check the day.
  - With an opening on a closed date, they return `ERR_DATE_CLOSED` on `opening_date` with a value `today`, so the form can offer to use today.
- **Partner code.**
  - `partners_code_status()` returns: is a code set, is the section unlocked, the wait in seconds.
  - `partners_code_set(password, code, code_again)`.
  - `partners_code_change(old_code_or_password, code, code_again)`.
  - `partners_code_unlock(code)` and `partners_code_lock()`.
  - `partners_code_reset(password_or_recovery, code, code_again)`.
  - Every partner command and partner report is refused with `ERR_PARTNER_LOCKED` while the section is locked. This covers:
    - the partner screens and capital;
    - R30 and R31;
    - analytics F and the partner columns of G, I, J and K;
    - the partner strip;
    - the partner totals in the خرچہ preview.
  - Commands that only pick a partner stay open, and return no totals while locked.

## 4. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T0 | Audit: every entry type × (edit service, edit command, cancel). Reproduce why adding an account says the day is closed (Umar's data: closed up to 30-10-26). List every partner command and report. Send the list before building. | backend | |
| F0 | Audit: on every screen and list, where Edit and Cancel are reachable today and where they are missing. Send the list before building. | frontend | |
| T1 | Contract of Section 3: types, stubs returning ERR_NOT_IMPLEMENTED, error codes, `pnpm gen:types`, fixtures. | backend | T0 |
| T2a | Edits for the missing entry types, with the change list and effect preview. Money agent before and after. Integration tests per type. | backend | T1 |
| T2b | Edit stamp and history from the audit log; the edited flag on list and report rows; `revised_on` in print data. | backend | T1 |
| T2c | Reopen and fix (`day_fix_open`, `day_fix_close`); golden E2E-45 in `golden_day.rs`. | backend | T1 |
| T2d | Accounts on any day (FR-10.11), with a test of the closed-day case Umar hit. | backend | T0 |
| T2e | Partner code: Argon2id hash, the unlock in app state, the waits, audit entries, the ERR_PARTNER_LOCKED check on every listed command (S8); E2E-46 in `services/tests/partner_code.rs`. | backend | T1 |
| F1 | One edit entry on every screen and list row (تبدیل کریں, منسوخ کریں); the amber band; the change list and effect box before save; the reason box (required for older entries); "دن کھولیں اور درست کریں" with its confirm. | frontend | T1, F0 |
| F2 | The stamp: header line, row mark, field marks with "پہلے ...", the history dialog, "نظر ثانی شدہ" in print headers. | frontend | T1 |
| F3 | Partner code: setup (password, code twice), unlock box on entering the section, lock on leaving, idle lock and logout, the locked partner strip card, change and reset in Settings → Security, waits and messages. | frontend | T1 |
| F4 | Account form: no day check without an opening; the "use today" offer. Wire F1 to F3 to the real commands; screenshots ur and en at 1366 and 1024. | frontend | T2a to T2e, F1 to F3 |

Rules for all tasks:
- Fluent text goes in ur and en, with the exact Urdu (plan 40 rule), marked review pending.
- Targeted checks only (Umar's rule).
- Every number from Section 2 must show on screen exactly.

## 5. Acceptance

- E2E-45 and E2E-46 pass. All earlier golden numbers are unchanged.
- Every entry type in the FR-7.70 table can be edited from its screen and from its list row, or refuses with its reason.
- The date field cannot be changed in an edit.
- Every edited entry shows its stamp in the header, the row and the changed field. The history lists every version. A reprint says نظر ثانی شدہ.
- A closed day's entry is fixed in one step, and the days close again.
- A buyer with no opening balance saves when the day is closed.
- The partner section, strip and reports open only with the code. The backend refuses without it.

## 6. Out of scope

- Several logins and roles: there is one Owner. The stamp names the signed-in user (A75).
- Changing an entry's date.
- Editing posted journal lines directly.

## 7. Open questions for Umar (defaults are being built)

- A75: should the reason box also ask who made the change (the munshi), typed by hand?
- A76: is the list of screens behind the partner code right?
- A77: does the code stay open while inside the partner section, or is it asked on every screen?
- A78: is a reason optional for today's edits?
