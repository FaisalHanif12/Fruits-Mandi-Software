# Plan 10: The first voucher, open days and the golden setup

Deliverable: D2, plan file 10. Branch: the run branch (goal run).

## 1. Goal

The voucher framework every voucher type will use, the پیشگی voucher (giving an advance), day close and reopen, `pnpm seed` building the PRD 4.3 opening position through the real services, and the golden test file with E2E-19.

## 2. Requirements

PRD 4.3; 15.1 and FR-15.1 (common voucher fields, the running list); 15.4 (giving an advance; adjustment comes in D3, returns in D4); FR-7.60; FR-21.1 and FR-21.2 v1.2 (reopen needs only a reason); FR-6.6. NFR-SPD-9. Code Standards 9.2. E2E-19. Acceptance: D2-A14, D2-A15, D2-A16. The money-calculations agent before and after.

## 3. Worked numbers (golden, never changed)

PRD 4.3, through the real services with an injected clock:
1. "Today" 31-08-26: opening بنام 65,000 on روکڑ 01010001; opening جمع 65,000 on ابتدائی سرمایہ 06010001; both against ابتدائی فرق 06019999.
2. "Today" 01-09-26: پیشگی voucher پ-1 to رحیم 030110001, 15,000 cash: رحیم بنام 15,000 (an advance line, `is_advance`), روکڑ جمع 15,000. Cash 50,000.
3. Close the days up to and including 09-10-26 (in order).
E2E-19 expects: trial balance 65,000 = 65,000; ابتدائی فرق 0; advances open 15,000; cash 50,000; "closed up to" 09-10-26; 10-10-26 open.

## 4. Contract

### Backend

- Voucher framework (`services::vouchers`): numbering per type given at save (پ-1, پ-2 ...; never reused), the common fields of PRD 15.1 (date, account, پختہ person later, amount as text, mode cash, bank or cheque with bank account and cheque number and date, narration), current and new balance, one transaction per voucher (posting through `services::ledger::post`, audit row), the running list of FR-15.1 (several vouchers saved together, each its own posting event and transaction, results per row).
- پیشگی: `vouchers_save_advance { date, seller_account_id, amount, mode, bank_account_id?, cheque_no?, cheque_date?, expected_goods?, narration? }` -> `VoucherSaved { number, entry_id, new_balance }`. Seller بنام with `is_advance`, cash or bank جمع. Under NFR-SPD-9 (150 ms). Errors: validate codes, `ERR_NOT_A_SELLER`, `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE`, `ERR_ACCOUNT_INACTIVE`.
- `vouchers_list { type, from, to }`, `assist_advances { seller_id }` (open advances and the last advance for the card, FR-7.60; read-only; open = given minus adjusted by `advance_allocations`).
- Day close (FR-21.1): `day_close_preview { up_to }` -> summary of what exists now (trial balance difference, must be 0; cash in hand; D3 and D4 add بکری and stock lines); `day_close { up_to }` closes the oldest open day or several in order, moves `closed_up_to_date`, audit row; refuses out of order and a non-zero trial balance difference (`ERR_TRIAL_NOT_BALANCED`). Cloud backup starts in D5.
- Reopen (FR-21.2 v1.2): `day_reopen { date, reason }`: reason required (`ERR_REASON_REQUIRED`), reopening a date reopens every later date, audit row with the reason. One user, so no role check beyond sign-in.
- `pnpm seed` (scripts/seed and the backend seed example): builds the PRD 4.3 position through the real services (not raw SQL) plus the masters and parties the worked day needs, names in both scripts: sellers رحیم 030110001 and کریم 030110002; buyers اکرم ٹول ٹیکس 03040001 and ارشد ملتانی 03040002; items کیلا (پھل), سیب (پھل), مولی (سبزی); packing پیٹی; cities and اڈے سوات اڈا and لاہور اڈا; the expense heads بجلی and تنخواہ (control 0501, PRD v1.2); the partners are D3. `pnpm seed -- --reset` starts again.
- `backend/crates/services/tests/golden_day.rs` starts here with the 4.3 setup and E2E-19, fixed "today" through the injected clock (Code Standards B1).

- From plan 07 (12-10-2026): `services::ledger::reverse` does not touch matching. Any reversal in this plan (a cancelled or edited پیشگی voucher, a changed opening advance) sets the related `advance_allocations` and `bill_allocations` rows inactive (who and when) and re-runs matching for that party, oldest first, in the same transaction (Architecture 12, B17).

- Added 12-10-2026 (frontend's request, accepted): `vouchers_preview_advances { lines }` -> each line's new balance and errors, and the list total, so the running list (FR-15.1) never adds money in the frontend (rule 1). Same domain function as the save (B5).
- Decided 12-10-2026: voucher numbers show as و-1, ا-1, پ-1 in Urdu (PRD 15.1) and R-1, P-1, A-1 in English (team default, on Umar's review list). The number is stored per type; the letter comes from Fluent.
- Decided 12-10-2026: day close defaults to the oldest open day (FR-21.1); the user may pick a later date to close several days in order.

### Frontend

- پیشگی screen in the بکری side menu (and later from روکڑ in D4): the common voucher form with `AccountPicker` (sellers), `PartyCard` with the seller's open advances and last advance (FR-7.60), amount as text, mode with bank fields, narration, expected goods; current and new balance from the backend; the running list of several vouchers with a running total and save all; no print button yet (the receipt slip comes with D5's print work). Four states, keyboard only (`F10` save, `F2` new), both languages.
- Day close screen: preview of the summary, confirm dialog (FR-7.8) that says exactly what will happen, then the new "closed up to" date in the status bar. Reopen screen: date, reason (required), confirm; the status bar updates.
- Tests in ur and en: the E2E-19 flow by keyboard against the real command shapes, day close out of order refused, reopen without a reason refused.

## 5. Tasks

T1, T2 (with the money agent before and after; golden_day.rs E2E-19), T3, T4.

## 6. Acceptance checklist

1. E2E-19 passes: 65,000 = 65,000; ابتدائی فرق 0; advances open 15,000; cash 50,000 (D2-A14, `cargo test -p mandi-services --test golden_day`).
2. Day close in order; reopen with a reason, in the audit log; out of order refused (D2-A15).
3. Every D2 screen matches the design in both languages at the three sizes (headless Chrome now, Windows WAITING-CI); the full test suite, including every D1 test, passes (D2-A16).
4. `pnpm seed` builds the opening position and prints the E2E-19 numbers.
5. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check` exit 0.
