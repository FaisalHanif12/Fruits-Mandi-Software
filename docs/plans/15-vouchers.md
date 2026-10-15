# Plan 15: وصولی, ادائیگی and the rest of پیشگی

Deliverable: D4, plan file 15. Branch: `feature/bikri` (goal run D3+D4). Version 1.0, 15-10-2026.

## 1. Goal

One voucher screen with a type list (وصولی, ادائیگی, پیشگی) takes money from buyers, pays sellers and others, and gives and takes back advances. Receipts settle a buyer's oldest bills first. Payments show what is payable, with a seller's opening credit and جنرل credits counted as payable. A payment to a buyer is cash lent. A seller can return advance money. All three types can be edited and cancelled by reversal. The collection list and the receipt slip print.

## 2. Requirements

PRD 15.1 to 15.4 (FR-15.1 to FR-15.7, FR-15.9 to FR-15.12), FR-6.7 (cash confirm), FR-7.67 and FR-7.68, 13 (FR-13.7, FR-13.8), FR-7.38, FR-7.58 to FR-7.61, FR-5.4, FR-12.22 (cancel gives credit back), R26 data, R24 data. v1.5: a seller's opening credit and a جنرل credit on a seller are payable (FR-7.59, money agent 8.2a); a جنرل line never adjusts advances (16.4). NFR-SPD-9, NFR-SPD-21, NFR-SEC-4, NFR-SEC-5. Architecture 11. The money-calculations agent (Sections 8.1 to 8.3) before and after. E2E-6, E2E-7, E2E-21, E2E-27, E2E-28 (the payment part), E2E-31, E2E-32. Acceptance: D4-A9 to D4-A13, D4-A19, D4-A29, D4-A30 (the voucher part), D4-A22 (the voucher screen).

## 3. Worked numbers (the tests use exactly these)

After Events 1 and 2 of the worked day unless stated.

| Check | Working | Expected |
|---|---|---|
| E2E-6 وصولی | اکرم owes 12,880, pays 10,000 | open 2,880 بنام; oldest open bill is بکری 1 dated 10-10-26, 0 days |
| E2E-7 ادائیگی | رحیم credit 24,316 − نقد 5,000 − advance 15,000 | payable 4,316; paying 4,316 leaves 0; پیشگی report: given 15,000, adjusted 15,000, open 0 |
| E2E-27 | ادائیگی to کریم | payable postings: بکری 2, صافی 10,160, open 10,160; "پورا بقایا" gives 10,160 |
| E2E-21 | شاہد owes 1,000 and ناصر 2,000 (two پختہ buyers); receipt of 1,000 for شاہد | شاہد 0, ناصر 2,000 still due; shاہد's payment never settles ناصر's bill |
| FR-15.5 | اکرم pays 15,000 against 12,880 | 12,880 settled, 2,120 stays as his credit and settles his next bill by itself |
| E2E-28, payment part | tick-off run: رحیم payable 24,316 − 5,000 = 19,316; paying 19,316 | payable 0; ledger 15,000 بنام; advance open 15,000; رحیم not in "بیوپاری کے ذمے"; trial balance balances |
| E2E-31 | opening position; 10-10-26 رحیم returns 10,000 of his advance in cash (وصولی on his account); then Event 1 | after the return: advance open 5,000, cash 60,000; after بکری 1: advance adjusted 5,000; پیشگی report given 15,000, returned 10,000, adjusted 5,000, open 0; payable 24,316 − 5,000 − 5,000 = 14,316; ledger 14,316 جمع |
| E2E-32 | after Events 1 to 5, ادائیگی of 3,000 cash to اکرم, reason نقد ادھار | اکرم dues 2,880 + 3,000 = 5,880 (the 3,000 is a bill dated 10-10-26 after his بکری 1 bill); cash 55,484; Σ buyer debit balances 21,564 + 3,000 = 24,564 |
| D4-A29 | کریم opens with 10,160 جمع; ادائیگی of 4,000 | paid 4,000, still open 6,160, balance 6,160 جمع, no پیشگی note. A جنرل credit of 1,000 on رحیم (advance 15,000 open): advance stays 15,000, 1,000 payable; Σ still open − Σ open advances = his balance |
| FR-6.7 | cash 20,000, پیشگی 25,000 in cash | cash after −5,000; refused without the confirmation, saved with it |

## 4. Contract

New error codes (`errors.ftl` in `ur` and `en`, `# review: pending`): `ERR_PAYMENT_TO_BUYER_NEEDS_REASON` (reason, field `reason`), `ERR_RETURN_EXCEEDS_OPEN` (value `open`; a seller's return above what he owes the firm plus his open advances), `ERR_VOUCHER_CANCELLED`, `ERR_PERSON_REQUIRED` (a پختہ account needs its person), `ERR_CHEQUE_REQUIRED` (cheque mode needs number and date). Existing: required, more than zero, decimals, date closed or future, account inactive, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`, version changed, not found.

### Backend (module `vouchers`; commands in `app/src/commands/vouchers.rs`, all need sign-in)

- `vouchers_save_receipt { date, account_id, person_key?, amount, mode: cash|bank|cheque, bank_account_id?, cheque_no?, cheque_date?, collector_staff_id?, narration? }` -> `VoucherSaved { number, voucher_id, entry_id, new_balance }`. Buyer: cash or bank بنام, buyer جمع; `match_bills` settles his oldest open bills first, by account or by پختہ person key; the extra stays as credit (FR-15.5). Seller (FR-15.12): the same posting; it adjusts his open advances oldest first, then settles what he owes ("بیوپاری کے ذمے"); more than both together is `ERR_RETURN_EXCEEDS_OPEN`. Number series و-1 (Urdu) / R-1 (English).
- `vouchers_save_payment { date, account_id, person_key?, amount, mode, bank_account_id?, cheque_no?, cheque_date?, reason?, confirm_cash_below_zero, narration? }` -> `VoucherSaved`. Account بنام, cash or bank جمع. To a seller above his open payable: a warning in the preview, still saved, the extra shows in "بیوپاری کے ذمے" and is not an advance (FR-15.7). To a buyer: cash lent (نقد ادھار), `reason` required, it becomes one of his bills for matching, aging and اگراہی (FR-15.11). Series ا-1 / P-1. A payment never adjusts advances.
- `vouchers_save_advance` (from D2, kept as it is) now also needs `confirm_cash_below_zero`; series پ-1 / A-1. A shared `VoucherInput` base is allowed so the three types share parsing, but each type keeps its own service function and posting (B3, FR-7.67).
- `vouchers_preview { type, input }` -> `VoucherPreview { new_balance, bill_effects: [{ bill_ref, settled, still_open }], warnings, cash }`: the bills the amount will settle, the payable and the warning of FR-15.7, the cash after (FR-6.7), the FR-7.68 hint flags (`payable_is_zero_for_seller`). Same domain function as the save (B5). Read-only. Extends the D2 `vouchers_preview_advances`.
- `vouchers_edit { id, version, ...same fields as the type }` and `vouchers_cancel { id, version, reason }`: while the posting date is open (`ERR_DATE_CLOSED` otherwise), reverse the posting (same date) and post the new version in one transaction; matching rows of the old posting are switched off with who and when, and matching is run again for the party, oldest first (FR-15.3, Architecture 12). `vouchers.current_entry_id` points to the live posting; a cancelled voucher keeps its number. Audit rows with old and new values and the reason. For پیشگی the reversal also switches off its `advance_allocations`.
- Reads (read-only, B29): `assist_open_bills { account_id, person_key? }` -> open bills oldest first (date, source بکری no or opening or cash lent or جنرل, amount, still open, days) and the total (FR-7.58); `assist_open_payables { seller_id }` -> payable postings (date, source, credit, advance adjusted, paid, still open) oldest first and the total, worked out at read time (FR-7.59, money agent 8.2a), counting the seller's opening credit, بکری credits, جنرل credits and (plan 16b) purchase credits; `assist_advances { seller_id }` (D2) now includes returned and adjusted; `vouchers_list { type, from, to, account_id? }`; `vouchers_get { id }`.
- Advances query (R26 data): `advances_report { seller_id?, as_of }` -> per seller given, returned, adjusted, open, last date. E2E-7 checks it; D5 prints it.
- اگراہی فہرست (FR-15.6, FR-7.61): `assist_collection_list { date, collector_staff_id?, city_id? }` -> buyers planned for the date (by collection weekday) with their dues, grouped by collector and city; `vouchers_save_receipts_batch { rows: [{ account_id, person_key?, amount, ... }] }` saves each receipt as its own posting event in its own transaction, skipping rows with amount 0, and returns a result per row (FR-15.1, FR-7.61).
- پختہ (FR-13.7, FR-13.8): receipts and payments with a person key; `pukhta_make_regular { person_key, side, account_input }` makes a proper account with `F4` rules and moves the person's open balance to it with a جنرل entry (balanced, split rules apply, audited).
- Receipt slip print data: `reports_voucher_print_data { voucher_id, language }` with the amount in words (FR-15.2), using the plan 14 print layout (A5 or 80 mm).
- Global search (FR-5.4) gains voucher numbers.
- Tests: golden_day gains Events 3 and 4, E2E-6, E2E-7, E2E-21, E2E-28 (payment part), E2E-31, E2E-32; `assist.rs` gains the backend part of E2E-27; integration tests for edit and cancel of all three types (reversal rows, matching re-run, audit), bank and cheque modes, the cash confirm, closed-date refusal, sign-in refusal; property tests (the sum of a buyer's open bills equals his balance after random receipts and credits; a payable's still open never negative; an advance's open amount equals given − returned − adjusted). NFR-SPD-9 voucher save under 150 ms, NFR-SPD-21 party card with 10 years of open bills (bench and `scripts/perf`).
- Money agent before and after.

### Frontend (feature `vouchers`)

- One voucher screen with a type list at the top: وصولی, ادائیگی, پیشگی (FR-7.67, FR-15.10). The D2 پیشگی screen becomes its پیشگی type; its command and tests stay. The side menu items and `Alt+V`, `Alt+A`, `Alt+P` open it with the type chosen. Changing the type keeps date, amount, mode and narration and clears the rest. Entries can be made one after another with a running total from the preview (FR-15.1).
- Common fields of PRD 15.1; `AccountPicker` with `PartyCard` (oldest open bill and age for a buyer; payable postings with the advance already adjusted for a seller); the open bills panel with the bills the amount will settle marked (from `bill_effects`); the payables panel; "پورا بقایا" and "پرانا بل" put the backend's amount into the field as text; for a seller's وصولی the open advances it will adjust; the reason field when the ادائیگی account is a buyer; the FR-7.68 note "اس بیوپاری کا کوئی بقایا نہیں۔ کیا یہ پیشگی ہے؟" with a button پیشگی بنائیں (never automatic); the amber cash note and `CashConfirmDialog` (plan 12) on save for ادائیگی and پیشگی.
- The اگراہی فہرست screen and the collection round (one button opens a receipt list already filled; the user types amounts and saves all); "make regular"; edit and cancel (confirm dialog with reason, FR-7.8); the receipt slip template (A5 and 80 mm); the voucher list below the form.
- Keyboard only, both languages, 1024 × 600, four states, language switch in the middle of an entry loses nothing. Fluent `vouchers.ftl` additions and errors. Design fallback as in plan 12 (`/design` is not available): `frontend-design` plugin and `bilingual-screen` skill inside `docs/design/design-system.md`.
- Windows specs written and syntax-checked: the voucher screen by keyboard, receipt, payment, advance (run LOCAL-ONLY).

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, stubs returning `ERR_NOT_IMPLEMENTED`, error codes, `pnpm gen:types` | backend | plan 14 built |
| T2 | Receipt, payment, advance changes, preview, edit and cancel, reads, collection list, make regular, tests, money agent | backend | T1 |
| T3 | The voucher screen, panels, notes, collection list, receipt slip against the stubs; Fluent | frontend | T1 |
| T4 | Wire to the real commands; keyboard run; screenshots | frontend | T2, T3 |

## 6. Acceptance checklist

1. E2E-6, E2E-7, E2E-21 pass in `golden_day.rs` (D4-A9, D4-A10, D4-A11).
2. E2E-27 passes in `assist.rs` (D4-A12).
3. Edit and cancel of all three types: reversal rows exist, matching is re-run, audit rows written, refused on a closed date (D4-A13).
4. The receipt slip prints with the amount in words in both languages (snapshots; PDFs LOCAL-ONLY).
5. E2E-28 payment part, E2E-31, E2E-32 pass (D4-A19).
6. The opening-credit and جنرل-credit payable rules of D4-A29 pass; the cash confirm for ادائیگی and پیشگی passes (D4-A30, voucher part).
7. NFR-SPD-9 and NFR-SPD-21 at their runner thresholds (D4-A18).
8. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` exit 0; every earlier test still passes.

## 7. Out of scope

Later sales, edit and cancel of a بکری (plan 16); the روکڑ area link to this screen (plan 17); R20 to R26 reports and the dashboard tiles (D5); cloud backup (D5).

## 8. Open questions

None for Umar. Team defaults: the collection list uses the buyer's collection weekday; a receipt batch skips rows with 0.
