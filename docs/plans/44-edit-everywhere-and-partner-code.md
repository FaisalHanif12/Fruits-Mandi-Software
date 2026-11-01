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

## 8. F0 audit and lead decisions (31-10-2026)

The F0 audit found the following.
- Edit exists today only for بکری (with later sales), وصولی / ادائیگی / پیشگی, and پھل.
- These have no edit at all: خرچہ (all three types), ادھار رقم, الگ کھاتہ, سرمایہ and جنرل.
- Trip and garden entries have no edit, only cancel.
- جنرل has no list, and روکڑ بک rows do not open their entry.
- A closed day is a dead end: there is a note and a link to /day, and nothing closes the day again.
- There are three different edit styles today.
- The account quick-add sends the computer's date as the opening date, with no amount.

Decisions:
1. **One shared edit kit** goes in `shared/ui`, used by every entry: the amber band, the change list with its effect, the reason box, the closed-day button and the stamp pieces. The three old styles move onto it. They are not copied a fourth time.
2. **One way in.**
   - جنرل gets a small list of saved entries.
   - Rows of روکڑ بک, R15, R23, R25, R29 and R37 open their entry on its own screen. This replaces the dead-end page "on another screen".
3. **Row flags.** Every saved-entry row (VoucherRow, FruitRow, the بکری list row, TripEntryRow, GardenHistoryRow, PurchaseRow) carries:
   - `can_edit`, `can_cancel`;
   - `locked_reason` (a code: goods_sold, lot_seller_locked, garden_closed, month_closed);
   - `needs_reopen`;
   - `edited` (time, by, count).
   - RuleRow gets `used`.
   - The old `editable` field is removed.
4. **Accounts (PRD FR-10.11, refined).**
   - An unchanged opening is never checked again. The frontend leaves out `opening_date` when there is no amount and nothing changed.
   - Quick-add sends no opening.
   - When no open date exists (today closed at night), the form offers to save the account without the opening, or to reopen the day.
5. **Partner code (PRD FR-17.11 and A76, refined).**
   - Behind the code: every partner total and history, as the FR-17.11 list says. On those screens the figures are replaced by a "کوڈ درج کریں" box, and the rest of the screen works.
   - Open without the code: `partners_list` (names only) and the split of the one entry on screen.
   - The unlock lasts while the user stays on screens behind the code. One route-change hook in the app shell locks it again.
   - The partner setup step of the first-run wizard needs no code, because none exists yet.
6. **Vouchers.** Backend answers in T0 whether `vouchers_edit` and `vouchers_cancel` take every voucher type. If not, T2a adds the missing types.

## 9. T0 audit and lead decisions for T1 (31-10-2026)

T0 found the following.
- Every edit is a reversal plus a new posting, and every edit and cancel stops on a closed day.
- `vouchers_edit` already covers every voucher type, including جنرل. But vouchers and پھل still let the date change.
- These have no edit at all:
  - trip purchase, trip cost, sale in another shop;
  - garden installment, cost and sale, and garden arrival;
  - the trip and garden header.
- There is no change list with its effect for any type, except the two existing previews.
- A new account with no opening amount already saves on a closed day. Only an opening amount on a closed date is refused.
- The audit log is enough for the stamp and history of every type except بکری. Its edit event lacks the header, the lot fields and charges, and line identity.

Decisions:
1. **Dates.** One code, `ERR_EDIT_DATE_FIXED` on the date field, for every edit: vouchers, پھل, and بکری (rename `day_changed`; update its fixture and test).
2. **Reason (A78).** Required when the entry's date is not today, optional for today. This applies to every type, including پھل, whose reason is required today.
3. **New edits (T2a).**
   - Trip purchase, trip cost, sale in another shop, garden installment, cost and sale, and garden arrival: each allowed only while nothing of the goods it touched is sold or written off. Otherwise it is refused with `ERR_EDIT_NOT_ALLOWED` and the rule `goods_moved`.
   - The trip or garden header (name, market, sent, rent, item, season end, note): editable at any time. The shares stay locked after the first entry (FR-12.29).
   - Money agent before and after.
4. **Change list and effect.** Every edit preview returns `changes` (where, field, before, after) and `effect` (each party's balance before and after, cash before and after, and each partner's share change while the partner code is open). Both come from the same function as the save.
5. **Rules.** RuleRow gets `used`. Saving a used rule may change only its end date and active flag. Anything else is refused with `ERR_RULE_USED`, and the screen offers "add a newer rule from a date".
6. **Accounts (FR-10.11).**
   - Keep the no-amount path free of any day check, and add a regression test with Umar's data.
   - `ERR_DATE_CLOSED` on `opening_date` carries `closed_up_to`, `today` and `today_open`. The form then offers "use today" when today is open. Otherwise it offers "save without the opening" or "reopen the day".
   - An unchanged opening is never checked again.
7. **Partner code: refuse.** These are refused while locked with `ERR_PARTNER_LOCKED`:
   - `partners_get`, `partners_create`, `partners_update`, `partners_set_shares`, `partners_check_shares`, `partners_overview`, `partners_statement`;
   - `capital_history`, the capital in and out saves, `partner_takes`;
   - in the one report and analytics dispatch: R30, R31, the partner parts of R29, R33, R34, R36 and R37 by partner, analytics F and the partner columns of G, I, J and K. This covers run, print, the print job and Excel.
8. **Partner code: hide.** These keep working, but the partner fields are emptied and a `partners_locked` flag is set:
   - `partners_list`: names only, no `share_bps`;
   - `dashboard_get` (the strip);
   - the partner totals of `vouchers_preview_expense` and the fruit preview;
   - the partner lines of `day_close_preview` and `month_close_preview`;
   - the whole-trip and whole-garden partner results (TripPartnerResult, GardenPartnerResult, PartnerVentureCard, the garden close numbers per partner);
   - the amount of an overdue partner take in the attention list;
   - the old and new values of partner and share events in `audit_list`.
9. **Partner code: open.** These stay open: the split of one entry (the بکری preview, saved بکری, purchase, trip and garden entry parts, fruit parts), the first-run wizard, and the backup job and its pack.
10. **Reopen and fix.** `day_fix_open(date, reason)` returns a token and the days. `day_fix_close(token)` closes them again with the FR-21.1 checks and returns the days that stayed open. Both are in the audit log. Golden E2E-45.
11. **Edit stamp (T2b).** Enrich the بکری edit audit event with the header, lots with all typed fields and charges, and lines with their lot and line identity, before and after, so the history is exact. Add `audit_edit_stamp` and `audit_edit_history`. The history reuses the audit flatten code. Add the row flags of Section 8.3 and `revised_on` in print data.
12. **Ask Umar** what he typed when the account was refused (probably an opening amount with an older date).

## 10. T1 as built (31-10-2026), accepted

The contract has:
- 27 new commands, nine new error codes and the row flags;
- `partners_locked` flags, `RuleRow.used`, and `ERR_EDIT_DATE_FIXED` for بکری;
- bindings (773 files) and 29 contract fixtures.

Changes from Sections 3 and 9, all accepted:
1. **Review commands.** The review of the existing edits uses new names: `bikri_edit_review`, `vouchers_edit_preview`, `fruit_edit_preview`.
   - They return `EditPreview {errors, review{entry_date, changes[], effect, reason_required, needs_reopen}}`.
   - `bikri_edit_preview` keeps returning the live summary.
   - New edits are `<x>_edit_preview` and `<x>_edit_save` for: purchases, trips_cost, trips_other_sale, gardens_entry, gardens_arrival, trips_header and gardens_header.
2. **Rule name and locked reasons.** The rule name for moved goods is `goods_moved`. LockedReason also has `cancelled` and `returns_stand`.
3. **Partner list.** `partners_list` returns `PartnersList {partners, partners_locked}`. While locked, `share_bps` is 0.
4. **Row flags.**
   - The flags are flattened into each row. The old `editable`, `can_change` and per-row `can_cancel` are removed.
   - The flags keep the old meaning until T2b, which switches to the new rule (on a closed day: `can_edit` true and `needs_reopen` true).
   - BikriDetail also carries the flags.
5. **Stamp and history.** `audit_edit_stamp` and `audit_edit_history` are in `mandi-reports::audit`. EditEntity covers: bikri, voucher, fruit_take, purchase, trip_cost, trip_other_sale, garden_entry, garden_arrival, trip, garden, account and charge_rule.
6. **The `partners_locked` field** is on Dashboard, AttentionList, AuditPage, ExpensePreview, FruitPreview, TripDetail, GardenDetail, GardenCloseNumbers, DayCloseSummary and MonthPreview. The overview, venture card and statement are refused when locked instead.
7. **Error values.**
   - `ERR_DATE_CLOSED` on `opening_date` carries `closed_up_to`, `today` and `today_open`.
   - `ERR_RULE_USED` carries `saved_lines`.

T2 order: d, e, a, c, b. There is one short HANDOFF after each part. The money agent runs before and after a and c.

## 11. T2d and T2e as built (31-10-2026), accepted

**T2d (accounts on any day).**
- The no-amount path and an unchanged opening already had no day check.
- New: `ERR_DATE_CLOSED` on `opening_date` carries `closed_up_to`, `today` and `today_open`.
- `services/tests/accounts_any_day.rs` tests this with Umar's data.

**T2e (partner code).** It is built as Section 9 says. Backend's choices, accepted:
1. **Before a code is set, the section is open in the backend.** The frontend must still show the setup screen first, with no way to skip it, the first time the section or any locked figure is opened (FR-17.11 Setup).
2. **Analytics.** View K is refused whole, because its rows are per partner. Views I and J lose their partner columns.
3. **Reports.** R33, R36 and R37 are refused only for the partner view or a partner filter. Their plain listings stay open.
4. **Month close preview.** Its partner capital lines come back with amounts 0 and `partners_locked` true.

Frontend rule: when `partners_locked` is true, a figure is never shown as 0. Its place shows the "کوڈ درج کریں" box. This applies to the month close lines, the fruit preview totals, `share_bps` and every hidden field.

## 12. T2a part 1, T2c and T2b as built (31-10-2026), accepted

**T2a part 1: the three existing edits (بکری, voucher, پھل).**
- The edit previews are rehearsals of the real save, rolled back.
- They return `changes` and `effect`. The partner part of `effect` covers only the one posting being edited.
- All three refuse a changed date with `ERR_EDIT_DATE_FIXED`.
- A reason is required only when the entry is not of today.
- `ERR_RULE_USED` and `RuleRow.used` are in.
- Money agent, 2 passes: the final pass found no HIGH or MEDIUM problem, and every LOW was fixed.

**T2c: reopen and fix.**
- `day_fix_open` and `day_fix_close` work as decided.
- A stale or foreign token is refused with `ERR_FIX_TOKEN_INVALID`.
- Each day closes in its own transaction with the trial balance check.
- Golden test: `golden_day/fix.rs` (E2E-45).
- An edit save on a closed day is still refused. The screen opens the fix first.

**T2b: the stamp.**
- `audit_edit_stamp` and `audit_edit_history` work.
- The بکری audit event now holds the whole posting before and after. For E2E-45 the history lists exactly 7 changes.
- The row flags follow the new rule: on a closed day `needs_reopen` is true, and locked entries carry `cancelled`, `month_closed`, `goods_moved` or `garden_closed`.
- `edited` carries {at, local_date, local_time, by, count, reason}.
- `revised_on` is in the بکری and voucher print data.
- `lot_seller_locked` and `returns_stand` are refused at save only. They are not flagged on the row.

**Lead decision on the new edits (trip and garden).** Option 1 was chosen: migration 0012 `edit_windows` (Architecture v1.12, 15.3).

## 13. T2a part 2 and vouchers_open_for_edit as built (31-10-2026), accepted

**What is built.**
- 14 new commands, a preview and a save for each of:
  - purchase;
  - trip cost and trip sale in another shop;
  - garden entry and garden arrival;
  - trip header and garden header.
- `vouchers_open_for_edit` opens all 13 voucher types with their saved values.
- Migration 0012 `edit_windows` (schema 12). Released migrations 0001 to 0011 are untouched. A test checks that 0012 changes nothing else.

**Contract facts.**
- `confirm_cash_below_zero` is on TripOtherSaleEditInput and GardenEntryEditInput. The cash question is about the net change of the edit.
- A missing reason is reported at the field `edit_reason` for trip cost and garden payment edits, and at `reason` elsewhere.
- Some edit errors are unprefixed (`date`, `trip_id`). The frontend maps them by code first.
- A trip cost edit is spread again over the same purchases it touched.
- The trip and garden headers are editable at any time.

**Refusals when goods have moved.**
- `goods_moved`.
- `carries_cost`: a purchase that carries a part of a trip cost keeps its qty.
- A sale in another shop can be edited only when it is the only mover of its purchase or arrival.

**Money agent.** One review, after the code; there was no separate review before the code for part 2. It found no HIGH problem. Both MEDIUM findings and the LOW findings L1 to L4 and L6 are fixed and tested. L5 (lock the header after the first entry) was not done on purpose, per decision 3.

**Release note.** Migration 0012 goes in `docs/release/whats-new.md` when the run is released.

## 14. Row sources (31-10-2026), accepted

**What is built.**
- `CashbookRow.source` is real.
- A test proves that the R15, R23, R25 and R37 rows carry a source for all 13 voucher types.
- R29 head rows open the head's account.
- جنرل is paged by `vouchers_list`.
- ReportSourceKind gains "trip" and "garden": a row of such an entry opens the trip or the garden itself.

**Lead decisions.**
- R11, R12 and R13 rows also switch from "venture" (the id of an entry inside the trip or garden) to "trip" or "garden" with the trip's or garden's own id, so every report row opens its entry (FR-7.70).
- **Follow-up, not in this run:** a "نظر ثانی شدہ" mark on list and report prints that hold an edited entry (the second half of the FR-7.71 print bullet). Entry prints (بکری and voucher slips) already carry `revised_on`. Fruit, purchase, trip and garden entries have no entry print.

## 15. F2c field and line marks as built (01-11-2026), accepted

**What is built.**
- Inside an entry open for edit, each field the last edit changed shows a chip "پہلے <old value>" / "Before <old value>", from `audit_edit_history`, with a tooltip (old, new, date and time, who).
- In the بکری grids a changed cell gets a corner mark with the same tooltip (`EntryGrid` `cellMarks`).
- Ids in old values are shown as names. An id that cannot be named shows "(another one)".
- An entry with no edit stamp loads no history.
- Edit forms are done for trips, gardens, purchases, cash book rows, جنرل and سرمایہ. Rows open from the cash book and from report rows.

**Checks (frontend).** `pnpm typecheck`, frontend lint, `pnpm i18n:check` clean; targeted vitest on the edit-mark files passed. Screenshots ur and en, 1366 and 1024, in `frontend/tests/e2e/output/screenshots/plan44/`. Windows WebView2: LOCAL-ONLY.

**Follow-ups, not in this run.**
- Marks in the saved (read-only) بکری view and in a later-sale edit. They need the posting `entry_id` on each `EditVersion` (backend), so a mark never lands on the wrong posting.
- `Ctrl+I` on a mark (tooltip only for now).
- An old partner id, and a master that is not in the recent list, show "(another one)" instead of the name.
- The edit stamp on masters (city, item and the rest).
- The expense edit screenshot shows a review card from another fixture (10,000 to 4,000) next to chips for 2,500 to 3,000. Check at the gate that the real screen shows one entry's numbers. The same in `marks-bikri-line-*`: the cell shows 600 while its tooltip says "now: 610". At the gate, check that the mark's "now" value always equals the value in the field.
- All new Urdu and English text waits for the reviewers.
- F2 final: garden forms use GardenHistoryRow `number`, `arrival_id` and `note`; trip and garden entries show their own number in the band; the stale ARRIVALS_IN_STOCK override is removed from `gardens-edit.test.tsx` (14 passed).
