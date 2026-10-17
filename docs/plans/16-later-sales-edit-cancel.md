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

### Contract as built in T1 (agreed 18-10-2026, recorded by the lead)

Commands (sign-in needed; stubs return `ERR_NOT_IMPLEMENTED` until T2): `bikri_stock_list(StockListInput)` -> `StockList`; `bikri_later_sale_preview(LaterSaleInput { lot_id, date, sales })` -> `BikriPreview`; `bikri_later_sale_save(LaterSaleSaveInput { .., confirm_cash_below_zero })` -> `LaterSaleSaved`; `bikri_open_for_edit(EditOpenInput { bikri_id, entry_id })` -> `EditableBikri { .., draft, stored: SavedLot[], postings }`; `bikri_edit_preview(EditPreviewInput { entry_id, draft })` -> `BikriPreview`; `bikri_edit_save(EditSaveInput { entry_id, version, draft, reason?, confirm_cash_below_zero })` -> `BikriEdited`; `bikri_cancel_preview(CancelPreviewInput { id })` -> `CancelPreview`; `bikri_cancel(CancelInput { id, version, reason, confirm_cash_below_zero })` -> `BikriCancelled`; `bikri_undo(UndoInput { id, confirm_cash_below_zero })` -> `UndoResult { draft }`; `bikri_write_off(WriteOffInput { lot_id, qty, reason })` -> `WriteOffDone`; `bikri_write_off_undo(WriteOffUndoInput { move_id, reason })` -> `WriteOffUndone`. Differences from the text above:

1. Extra command `bikri_cancel_preview { id }`: the confirm box needs the cash line and the closed days from the backend (FR-6.7, FR-7.8).
2. `bikri_undo` takes `confirm_cash_below_zero` (FR-6.7 applies to an undo that lowers cash).
3. `bikri_later_sale_preview` has no `confirm`; the cash note is `BikriPreview.cash`.
4. New codes beyond the plan text: `ERR_DATE_BEFORE_RECEIVED`, `ERR_QTY_BELOW_SOLD` (plus the planned `ERR_BIKRI_CANCELLED`, `ERR_LOT_NOT_FOUND`, `ERR_NOTHING_TO_SELL`, `ERR_WRITE_OFF_EXCEEDS_BALANCE`).
5. `bikri_stock_list` has `include_written_off`, so a write-off that left nothing can still be undone.
6. New fields: `SaleInput.sale_id`, `LotInput.lot_id` (optional), `BikriDetail.version`, `SavedPosting.reversed` and `.editable`, `SavedSale.state` (`in_force | replaced | removed | cancelled`).
7. The undo reason is the Fluent ID `bikri-undo-reason` (the frontend adds the message, ur and en).
8. **Migration 0005** (stable order key): `journal_entries.order_key` (matching orders by date, opening first, `COALESCE(order_key, id)`, id; an edit's new posting and every reversal copy the key; hashed only when set, so old chains are unchanged), `stock_moves.reverses_move_id` (unique index, mirror trigger), `bikri_sales.ended` (`replaced | removed | cancelled`; null means `is_active = 1`). Plan 16b's migration is therefore `0006`. Vouchers edit now keeps its place through the key and re-matches with the full working.

#### Field detail for the frontend (from backend's T1 message; all inputs are `{ input: ... }`)

1. `bikri_stock_list` `StockListInput { date, from, to (DD-MM-YYYY, shortcut or null), seller_id, item_id, include_written_off }` -> `StockList { rows: StockRow[], balance_total }`. `StockRow`: `lot_id, lot_no, bikri_id, bikri_number, received_date, seller (RecordRef), pukhta, item, packing, qty_received, qty_sold, qty_written_off, balance, days_waiting, write_offs: WriteOffRow { move_id, move_date, qty, reason }[]`. `include_written_off = true` also lists a lot with an undoable write-off at balance 0.
2. `bikri_later_sale_preview` `LaterSaleInput { lot_id, date, sales: SaleInput[] }` -> `BikriPreview` (one lot, `lots[0]`; line `remaining_qty` and `lots[0].balance_qty` use what the lot has left; the cash note is `cash`).
3. `bikri_later_sale_save` -> `LaterSaleSaved { bikri_id, number, lot_id, entry_id, preview }`. No نقد and no lot charges in a later sale. Field paths `lots[0].sales[j].<field>`; a date error is on `date`.
4. `bikri_open_for_edit` -> `EditableBikri { bikri_id, number, version, entry_id, posting_type ('bikri_first' | 'bikri_later'), entry_date, editable, draft: BikriDraft, stored: SavedLot[], postings: SavedPosting[] }`. The draft is in the plan 12 shape, built from STORED values; keys `lot:<id>` and `sale:<id>`; every lot has `lot_id` and every line `sale_id` (send them back unchanged). A charge a rule decided comes back EMPTY in the draft; a charge the user typed comes back typed. `stored` has method, value, rule and amount per line and lot (this posting's lines in force) for the tags and ⓘ. Compare typed text with loaded text; never add money. A one-time seller paid on the spot comes back with `cash_paid` empty. Rules: the day cannot change (`draft.header.entry_date` must equal `entry_date`); in a first posting lines can be added, removed or changed and lots can be ADDED (`lot_id` null) but not removed, saved lots keep their order, and the seller or item of a lot cannot change once it has a later sale; in a later posting only its lines are editable.
5. `bikri_edit_preview` -> `BikriPreview`; `cash` is the NET effect of the edit.
6. `bikri_edit_save` -> `BikriEdited { bikri_id, number, version, old_entry_id, reversal_entry_id, entry_id, preview }`. `entry_id` is the NEW posting: reload with `bikri_open_for_edit`. Ask for a reason when the day is not today (optional for the backend).
7. `bikri_cancel_preview` -> `CancelPreview { bikri_id, number, version, status, postings, cash: CashCheck, closed_dates[], can_cancel }`.
8. `bikri_cancel` -> `BikriCancelled { bikri_id, number, reversal_entry_ids[] }`. Reason required.
9. `bikri_undo` -> `UndoResult { bikri_id, number, reversal_entry_ids, draft }` (the draft has no ids: open it as a NEW unsaved entry). The stored reason is the Fluent ID `bikri-undo-reason`; the frontend adds it to `bikri.ftl` (ur "محفوظ کرنے کے فوراً بعد واپس", en "Undone straight after saving", review pending).
10. `bikri_write_off` `WriteOffInput { lot_id, qty (text, sums allowed), reason }` -> `WriteOffDone { lot_id, move_id, qty, qty_written_off, balance }`. Dated today.
11. `bikri_write_off_undo` -> `WriteOffUndone { lot_id, reversal_move_id, qty_written_off, balance }`. The write-off day must be open.

Existing types: `SaleInput.sale_id?`, `LotInput.lot_id?` (optional), the draft types are now also serialised, `BikriDetail.version`, `SavedPosting.reversed` and `.editable` (use it to offer "edit this posting"), `SavedSale.state`. A CANCELLED بکری keeps its lines with state `cancelled` (`is_active = false`) and its stored lot totals: show those under the cancelled banner; its `SavedTotals` count them. Fixtures `bikri_get_event1` and `bikri_get_event2` changed (new fields); copy them unchanged from `backend/crates/services/tests/fixtures`. New fixture pairs come with T2.

New error codes (`errors.ftl` ur and en, `# review: pending`): `ERR_BIKRI_CANCELLED` (id, entry_id or lot_id); `ERR_LOT_NOT_FOUND` (lot_id); `ERR_NOTHING_TO_SELL` (lot_id); `ERR_WRITE_OFF_EXCEEDS_BALANCE` (qty; value `available`); `ERR_DATE_BEFORE_RECEIVED` (later sale `date`; value `received_date`); `ERR_QTY_BELOW_SOLD` (`lots[i].qty_received`; value `sold`). Reused: `ERR_DATE_CLOSED` (`closed_up_to`), `ERR_QTY_EXCEEDS_LOT` (`available`), `ERR_REASON_REQUIRED`, `ERR_VERSION_CHANGED` (field `version`), `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (`cash_after_paisa`), `ERR_NOT_FOUND`, `ERR_FORMAT_INVALID` (the field says which). Matching and aging after an edit or cancel are done by the backend.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, stubs, error codes, `pnpm gen:types` | backend | plan 15 built |
| T2 | `calc_posting` targets, later sale, edit, cancel, undo, write-off, stock list, re-match, golden tests, money agent | backend | T1 |
| T3 | باقی مال, later sale, edit mode, cancel and undo UI against the stubs; Fluent | frontend | T1 |
| T4 | Wire to the real commands; keyboard run; screenshots | frontend | T2, T3 |

### Carried in from plan 15 (decided by the lead on 17-10-2026, money agent finding 1)

An edit or cancel writes the new posting as a NEW journal entry (higher id). Matching orders postings by date, opening first, then entry id, so a same-day edit moves the posting after a بکری saved later the same day. For advance matching this can refuse an edited seller return (`ERR_RETURN_EXCEEDS_OPEN`) or drop the adjustment of an edited پیشگی. Plan 16 fixes it once, for vouchers and for the بکری edit, cancel and later sale:

- **T1 adds migration `0005_*.sql`** with a stable order key on `journal_entries` (null means the entry's own id; an edit's new posting copies the order key of the posting it replaces, a reversal copies its original's). The matching walks and `seller_credit_lines` order by (date, opening first, order key, entry id). Plan 16b's migration then becomes `0006` and plan 17 refers to `0006` (renumbered in both plan files on 17-10-2026).
- Test: a same-day seller return edited (narration only) after a later same-day ticked بکری stays allowed; an edited پیشگی keeps its adjustment; buyer bill order and every earlier golden number are unchanged. The migration test checks a populated file and the journal chain. Money agent before and after.
- Every edit or cancel calls the full re-match (never the fast paths) for every affected key, as `vouchers/edit.rs` does; teach `matching::rematch_advances` and `seller_credit_lines` about `bikri_later` credits and their tick; reuse `ledger::reverse`.

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
