# Plan 16: باقی مال, later sales, edit, cancel and undo

Deliverable: D4, plan file 16. Branch: `feature/bikri` (goal run D3+D4). Version 1.0, 15-10-2026.

## 1. Goal

Goods that are not sold on the first day wait in باقی مال and are sold later as their own posting. A saved بکری can be corrected: any one posting is edited by a reversal plus a new posting on the same date, and the lines the user did not change keep their stored values exactly. A whole بکری can be cancelled, and the save can be undone within 10 seconds. Nothing posted is ever changed or deleted. Matching is run again after every reversal.

## 2. Requirements

PRD 12.8 (FR-12.14 to FR-12.16), 12.9 (FR-12.17 to FR-12.22), FR-12.11, FR-7.47, FR-11.6, FR-11.9 (v1.5: the per-lot commission moves), FR-6.3, FR-6.6, FR-6.7, FR-15.4, FR-15.5, FR-15.8, FR-18.7, FR-21.2, E3, E4, E14, E15, E16, E17, E43. NFR-SEC-5. Architecture 12 and 13.1 to 13.3. Code Standards B3, B17, B20. The money-calculations agent (Sections 6, 7 and 8.3) before and after. E2E-13, E2E-13b, E2E-14, E2E-15, E2E-16, E2E-16b. Acceptance: D4-A1 to D4-A8, D4-A31 (the commission move).

## 3. Worked numbers (the tests use exactly these)

| Test | Working | Expected |
|---|---|---|
| E2E-13: Event 1, edit اکرم's rate 600 → 610 | line 20 × 610 = 12,200; buyer comm 12,200 × 6 ÷ 100 = 732; he pays 12,200 + 732 + 160 = 13,092 | خام 29,600; seller comm 732 + 1,044 = 1,776; کل اخراجات 3,000 + 1,776 + 250 + 50 + 5,000 + 0 + 20 = 10,096; صافی 19,504; shared income 1,776 + 1,776 + 400 + 20 = 3,972; split 1,590 / 1,191 / 1,191; the audit log holds the old version |
| E2E-14: edit back to 600 | | every number returns exactly to E2E-1 to E2E-4 (29,400; 10,084; 19,316; 12,880; 3,948; 1,580 / 1,184 / 1,184) |
| E2E-13b (separate run) | after Event 1 the Owner changes the default buyer commission to 7%, then edits اکرم's rate to 610 | his changed line: 12,200 × 7 ÷ 100 = 854, he pays 12,200 + 854 + 160 = 13,214; ارشد's unchanged line keeps 1,044; shared income 1,776 + 1,898 + 400 + 20 = 4,094; split 1,638 / 1,228 / 1,228 |
| E2E-15: cancel بکری 2 | | the پختہ register shows it cancelled; the سیب facts net to zero; the split 616 / 462 / 462 is reversed; cash down by 12,800; trial balance still balances |
| E2E-16 (separate run) | Event 1 with 45 of 50 sold (اکرم 20 at 600, ارشد 25 at 580) | خام 26,500; seller comm 720 + 870 = 1,590; کل اخراجات 3,000 + 1,590 + 250 + 50 + 5,000 + 0 + 20 = 9,910; صافی 16,590; buyer comm 720 + 870 = 1,590; لاگا 45 × 8 = 360; shared income 1,590 + 1,590 + 360 + 20 = 3,560; split 1,424 / 1,068 / 1,068; باقی مال 5 نگ کیلا |
| E2E-16b | 11-10-26 later sale of the 5 نگ to ارشد at 560 | value 2,800; seller comm 168; buyer comm 168; لاگا 5 × 8 = 40; ارشد pays 3,008; seller credit 2,800 − 168 = 2,632; no lot charges again; shared income 168 + 168 + 40 = 376; split 152 / 112 / 112; lot totals خام 29,300, کل اخراجات 10,078, صافی 19,222; باقی مال empty |
| FR-11.9 (D4-A31) | a per-lot seller commission of 500 sits on line 1 of 2; an edit removes line 1 | line 2 now carries 500 with the stored method and value; the lot's seller commission includes 500 once |
| Cash back (FR-6.7) | cash was 20,000; a بکری with نقد 25,000 was saved with the confirmation (cash −5,000); it is cancelled | cash after the cancel is 20,000, no confirm needed (cash goes up); an edit that lowers a نقدی sale and takes cash below 0 asks |
| Re-match | اکرم paid 10,000 against 12,880; بکری 1 cancelled | the 10,000 becomes unmatched credit on his account and is matched again to his next oldest bills |

## 4. Contract

New error codes (`errors.ftl` in `ur` and `en`, `# review: pending`): `ERR_BIKRI_CANCELLED`, `ERR_LOT_NOT_FOUND`, `ERR_NOTHING_TO_SELL` (a lot with no balance), `ERR_WRITE_OFF_EXCEEDS_BALANCE` (value `available`), `ERR_UNDO_WINDOW_PASSED` is not used (the 10 seconds are the screen's; the backend only needs the date to be open). Reused: `ERR_DATE_CLOSED` (value `closed_up_to`), `ERR_QTY_EXCEEDS_LOT`, `ERR_REASON_REQUIRED`, `ERR_VERSION_CHANGED`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.

### Backend (module `bikri`, commands in `app/src/commands/bikri.rs`, all need sign-in)

- One calculation function for every posting kind: `domain::bikri::calc_posting(draft, target, rules, shares, stored)` where `target` is `New`, `LaterSale { lot_id }` or `Edit { entry_id }`. Preview and save always share it (B5). Lines carry `sale_id` when they already exist.
- A line is **unchanged** when its buyer, person key, qty, weight, seller rate, buyer rate, payment type and typed charge inputs equal the stored ones. An unchanged line copies its stored method, value, rule id and amounts exactly (FR-12.20, FR-11.6). A changed or new line looks the rules up again with the posting date; a typed value stays as typed (method دستی, value kept). When only a lot's received qty changes, its per نگ lot charges keep method and value and the amount is worked out again (C3); a typed amount stays. The one exception: if an edit removes the line carrying a per-lot seller commission, the charge moves with its stored method and value to the first remaining line of that lot in that posting (FR-11.9).
- `bikri_stock_list { filters }` -> lots with balance above 0: date received, بکری no, seller, item, packing, received, sold, written off, balance, days waiting (FR-12.14). Own goods rows join in plan 16b.
- Later sale: `bikri_later_sale_preview { lot_id, date, sales: Vec<SaleInput>, confirm? }` and `bikri_later_sale_save { lot_id, date, sales, confirm_cash_below_zero }`. A new posting on the existing lot on its own date (open). Charges for the new lines by the rules of that date. Seller credit = Σ new seller amounts − Σ new seller commission (C8, later sale). No lot charges, no per-lot commission again. The partner split uses the shares of the later date. The original posting and its day are not touched, even if that day is closed. Matching and stock moves as in the save.
- Edit: `bikri_open_for_edit { bikri_id, entry_id }` -> `EditableBikri { draft, stored: [per line and lot: method, value, amount], postings }`, a draft in the plan 12 shape built from the stored values (a typed charge is returned as a typed charge so the screen shows it). `bikri_edit_preview { entry_id, draft }` and `bikri_edit_save { entry_id, version, draft, reason?, confirm_cash_below_zero }`. Allowed only while the posting's date is open (FR-12.18); closed needs the Owner's reopen first (`day_reopen`, D2). One transaction: `ledger::reverse` of that posting's entry (same date), then the new posting; old `bikri_sales` rows are set inactive with `replaced_by_sale_id`; the lot totals are written in ONE update; stock reversal moves and new moves; analytics reversal rows (sign −1) and new rows (+1); the plan 11 hook negates and rewrites the partner rows with the shares of the posting date (FR-12.21); the matching rows of the reversed entry are switched off with who and when, then matching is run again, oldest first, for each affected buyer and for the seller's advances, in the same transaction (a test checks no seller shows too high an open advance afterwards); an audit row with old and new values and the reason. A posting cannot be edited if the بکری is cancelled.
- Cancel: `bikri_cancel { id, version, reason, confirm_cash_below_zero }`. Reason required (FR-12.11). Reverses every posting of the بکری; every posting date must be open (FR-12.22). Marks it منسوخ (number kept, never reused). Receipts matched to its lines become unmatched credit on each buyer and are matched again to his next oldest bills (E15). Analytics reversal rows (FR-18.7). Audit.
- Undo (FR-7.47): `bikri_undo { id }` = a cancel with the fixed reason "محفوظ کرنے کے فوراً بعد واپس" (a Fluent ID), allowed only while the date is open, returning `UndoResult { draft }` so the screen opens a new unsaved entry with the same values. The cancelled number is not reused. The 10-second button is the screen's.
- Write-off: `bikri_write_off { lot_id, qty, reason }` records a `written_off` stock move (−qty) and no journal entry for consigned goods (FR-12.16); `bikri_write_off_undo { move_id, reason }` records the reversal move. Errors `ERR_WRITE_OFF_EXCEEDS_BALANCE`, `ERR_REASON_REQUIRED`. Audit.
- The party card and the buyer's open bills (plan 15) read the re-matched rows, so they follow every edit and cancel.
- Tests: `golden_day.rs` gains E2E-13, E2E-13b, E2E-14, E2E-15, E2E-16, E2E-16b with the numbers above; integration tests: the rows before and after an edit (posted rows untouched, a reversal and a new entry exist), edit and cancel refused on a closed date and allowed after reopen with every step audited, undo, write-off and its undo, the per-lot commission move (D4-A31), an edit after a receipt matched the bill (matching recalculated, nothing lost), cancel of a بکری that has a later sale, edit of a later sale, a power-cut test (forced failure after the reversal leaves nothing), sign-in refusal. Property tests: an entry plus its reversal gives zero on every account; after random sales, later sales, edits and cancels the trial balance difference is 0, each buyer's open bills equal his balance, each split equals its shared income. 
- Money agent before and after.

### Frontend (feature `bikri`)

- The باقی مال screen: the list of 12.14 with filters (date, seller, item, party filter per FR-19.10), "فروخت کریں" opens the later sale, write-off with a reason, undo of a write-off. Later sale form: the lot's header (read-only) and new sale lines only, with remaining qty and the rate hint as in plan 12, the live summary and split from `bikri_later_sale_preview`.
- Edit mode on a saved بکری: pick a posting, the screen loads `bikri_open_for_edit`; changed cells are marked by comparing the typed text with the loaded text (no arithmetic); save asks for a reason when the date is not today; the audit comparison is visible; the same preview and keyboard rules as the book screen.
- Cancel with a reason (confirm dialog, FR-7.8, naming what will be reversed and the cash line if cash changes, `CashConfirmDialog` when it goes below 0); the undo button in the success toast for 10 seconds, `Alt+Z` (FR-7.47); a closed date shows the refusal with a hint to reopen the day.
- Keyboard only, both languages, 1024 × 600, four states, language switch in the middle of an edit loses nothing. Fluent additions (`bikri.ftl`, `stock.ftl` if the module naming needs it) and errors, `# review: pending`. Design fallback as in plan 12.
- Windows specs written and syntax-checked: later sale, edit, cancel, undo (run LOCAL-ONLY).

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, stubs, error codes, `pnpm gen:types` | backend | plan 15 built |
| T2 | `calc_posting` targets, later sale, edit, cancel, undo, write-off, stock list, re-match, golden tests, money agent | backend | T1 |
| T3 | باقی مال, later sale, edit mode, cancel and undo UI against the stubs; Fluent | frontend | T1 |
| T4 | Wire to the real commands; keyboard run; screenshots | frontend | T2, T3 |

## 6. Acceptance checklist

1. باقی مال lists open lots; write-off with reason and its undo (D4-A1).
2. E2E-16 and E2E-16b pass (D4-A2); E2E-13 (D4-A3); E2E-13b (D4-A4); E2E-14, every number returns exactly (D4-A5); E2E-15 including the سیب fact rows netting to zero (D4-A6).
3. Edit and cancel refused on a closed date, allowed after reopen, every step audited (D4-A7).
4. Undo straight after saving reverses everything and returns the draft (D4-A8).
5. The per-lot commission move passes (D4-A31, the commission part).
6. No posted row changed by any edit (rows shown before and after); matching is re-run; the books balance after each step.
7. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` exit 0; every earlier test still passes.

## 7. Out of scope

Own goods lots and their cost (plan 16b); edit and cancel of vouchers (plan 15); reports and the audit viewer (D5); a dedicated goods-return screen (Phase 2, E5 uses جنرل).

## 8. Open questions

None for Umar. Team default: an edit of a posting dated today needs no reason; an edit of an earlier open day asks for one (it is written to the audit log).
