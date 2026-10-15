# Plan 12: The بکری screen and the live summary

Deliverable: D3, plan file 12. Branch: `feature/bikri` (goal run D3+D4). Version 1.0, 15-10-2026.

## 1. Goal

The munshi types a whole بکری (header, lots, sale lines) and sees every number live: line amounts, commissions with their working, the seller summary, the buyer totals, the live partner split, and warnings. Nothing is saved yet (plan 13). The live preview and the later save call the **same** domain function (CLAUDE.md rule 3, B5). The frontend sends text and shows what comes back; it never adds, multiplies or rounds.

## 2. Requirements

PRD 12.1 to 12.5 (FR-12.1 to FR-12.7), 4.4 and 4.5, PRD 22 (C1 to C9, C17, C19), FR-11.1 to FR-11.4, FR-11.6, FR-11.9, FR-11.10 (v1.5), FR-6.7 (v1.5, the cash note and the shared confirm piece), FR-17.4 (live split), FR-7.38, FR-7.43, FR-7.48, FR-7.49 to FR-7.56, FR-13.2, FR-13.4, FR-13.5 (person key), E1, E2, E6, E8, E23, E28. NFR-SPD-4, SPD-5, SPD-7, SPD-23, SCL-3, USE-1, USE-2, USE-6, USE-26, USE-27. Architecture 3.1, 7 and 8. Code Standards B5, B30, F27, F35, F39, F40, F42, F43. E2E-26 (E2E-25 needs saved بکری, so it is checked in plan 13). Acceptance: D3-A3, D3-A4, D3-A7 (screen part), D3-A10 (E2E-26), D3-A11, D3-A19, D3-A20 (note and shared confirm piece). The money-calculations agent before and after.

## 3. Worked numbers (the tests use exactly these)

Event 1 (PRD 4.4): lot رحیم, کیلا, پیٹی, 50 received, freight 3,000 (firm), labour 5 per نگ, munshiana 50, phone 20, نقد 5,000. Lines: اکرم 20 × 600, ارشد 30 × 580. Defaults: seller 6%, buyer 6%, لاگا 8 per نگ (fruit).

| Item | Working | Expected |
|---|---|---|
| Line 1 value | 20 × 600 | 12,000 |
| Line 2 value | 30 × 580 | 17,400 |
| Seller commission | 12,000 × 6 ÷ 100 = 720; 17,400 × 6 ÷ 100 = 1,044 | 720 and 1,044, total 1,764 |
| Buyer commission | same | 720 and 1,044, total 1,764 |
| لاگا | 20 × 8 and 30 × 8 | 160 and 240, total 400 |
| Buyer totals | 12,000 + 720 + 160; 17,400 + 1,044 + 240 | 12,880 and 18,684 (sum 31,564) |
| Labour | 50 × 5 | 250 |
| Lot deductions (C6) | 3,000 + 1,764 + 250 + 50 + 5,000 + 0 + 20 | 10,084 |
| Lot net (C7) | 29,400 − 10,084 | 19,316 |
| Seller credit (C8) | 29,400 − 1,764 − 3,000 − 250 − 50 − 0 − 20 | 24,316 |
| Shared income (C9) | 1,764 + 1,764 + 400 + 20 | 3,948 |
| Live split | plan 11 | 1,580 / 1,184 / 1,184 |
| E2E-26 | after line 1 (20 of 50) | line 2 starts at 30; rate hint 600; working of line 1 seller commission "12,000 × 6 ÷ 100 = 720" |

More numbers: بکری 2 (PRD 4.5) buyer پختہ 10 × 1,200: buyer comm 720, لاگا 80, buyer pays 12,800, net 10,160, shared income 1,540. Typed on a line (FR-11.10): seller commission 4% on 12,000 = 480, saved with method دستی. Partial sale (E2E-16): 45 sold, خام 26,500, balance qty 5 (warning "5 نگ باقی ہیں"). Cash note (FR-6.7): cash in hand 20,000, نقد 25,000 on a lot gives cash after −5,000.

## 4. Contract

### Rules the backend follows

- One domain function builds the whole posting: `domain::bikri::calc_posting(draft, rules, shares, context) -> PostingCalc` (header checks, lots, lines, charges, summary, split input). `bikri_preview` and (plan 13) `bikri_save` both call it. No second copy. Money only through `core::money` (`line_value`, `weight_value`, `apply_bps`, `per_unit`, `split_by_qty`).
- Charges come from `domain::rules::resolve_charge` (plan 09) with the entry date, the lot seller for seller-side charges and the line buyer for buyer-side charges. A typed value (FR-11.10) is used only when the "allow line override" setting is on (starts on), is saved with method دستی, and also stores its percent or per نگ value. When the setting is off, a typed value is refused with `ERR_OVERRIDE_NOT_ALLOWED` on its field. A charge with no winning rule for seller or buyer commission is an error (broken data), never a zero.
- Per-lot seller commission (FR-11.9): charged once, on the first line of the lot.
- Lot charges (labour, munshiana, phone): input is `{ method, value }` with method per_unit (on qty received), per_lot, or manual; value is whole-rupee text. If the field is empty the rules may suggest one (source `rule`); the result always shows the effective method, value and amount so the screen can show it in the field (NFR-USE-27, F43).
- Freight: typed per lot, or split from the header total by `bikri_split_freight` (C17, `split_by_qty`, leftover to the lot with the most نگ, tie: lowest lot number). Freight paid by بیوپاری is shown but not deducted or posted (E6).
- Weight items (`sold_by = weight`): the line takes a weight; value is `weight_value(weight, rate per kg)` (E23); per-نگ charges still use the نگ count; percent charges use the value.
- Nothing from اپنا مال or a الگ کھاتہ seller in D3: the lot seller picker offers sellers and the پختہ کھاتہ (seller) account only (own goods come in plan 16b, الگ کھاتہ in plan 17).
- Cash in hand after the entry = current روکڑ balance + Σ نقدی lines' buyer totals − Σ lots' نقد. The shared function `services::cash::cash_after_entry` (built here, reused by plans 13, 15, 16, 16b, 17) returns it. The note and the confirm are required only when the result is below 0 **and** the entry lowers cash (net cash out above 0). A cash balance already below 0 does not ask again for an entry that does not lower it. The money agent confirms this reading.
- Date: the entry date must be open (FR-6.6). Drafts are not numbered (plan 13).

### New error codes (in `core::error`, with `errors.ftl` messages in `ur` and `en`, `# review: pending`)

`ERR_QTY_EXCEEDS_LOT` (values `available`; field `lots[i].sales[j].qty`), `ERR_OVERRIDE_NOT_ALLOWED`, `ERR_CREDIT_LIMIT_EXCEEDED` (only when the setting is block; values `limit`, `would_owe`), `ERR_SELLER_REQUIRED`, `ERR_BUYER_REQUIRED`, `ERR_PUKHTA_NAME_REQUIRED`, `ERR_PUKHTA_PHONE_REQUIRED`, `ERR_NO_LOTS`, `ERR_NO_SALE_LINES`, `ERR_WEIGHT_REQUIRED`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (shared; values `cash_after_paisa`; used by every cash-out command). Reused: required, must-be-more-than-zero, decimals, date codes, shares not 100, account inactive, not a seller.

### Backend commands (module `bikri`, all need sign-in; read-only ones write nothing, B29)

- `bikri_preview { draft: BikriDraft }` -> `BikriPreview`. Takes everything as text. **BikriDraft**: `header { entry_date, bill_date, book?, truck_no?, zamindar_account_id?, adda_id?, driver_name?, driver_phone?, total_freight?, store_id?, note? }`; `lots: Vec<LotInput { key, seller_account_id?, pukhta?: { name_ur, name_en, phone }, item_id?, packing_id?, marka_id?, store_id?, qty_received, weight_received?, freight?, freight_paid_by: firm|seller, labour?: ChargeInput, munshiana?: ChargeInput, phone?: ChargeInput, cash_paid?, store_rent?, adjust_advance: bool, sales: Vec<SaleInput> }>`; **SaleInput** `{ key, buyer_account_id?, pukhta?: { name_ur, name_en, phone }, qty, weight?, seller_rate, rate_differs: bool, buyer_rate?, payment_type?: credit|cash, seller_comm?: LineChargeInput, buyer_comm?: LineChargeInput, laga?: LineChargeInput, note? }`; **LineChargeInput** `{ method: percent|per_unit, value }`; **ChargeInput** `{ method: per_unit|per_lot|manual, value }`. Quantities and money accept sums such as `20+25` (FR-7.48, `parse_sum_*`). A sale row whose buyer and seller rate are both empty is a blank row: ignored for totals, but it still gets `remaining_qty` and `rate_hint`.
- **BikriPreview** returns (money as paisa numbers): `lots[]` (each: `lot_no`, `lines[]`, lot totals: `gross`, `freight`, `seller_comm`, `labour`, `munshiana`, `cash_paid`, `store_rent`, `phone`, `deductions`, `net`, `sold_qty`, `balance_qty`, `net_is_negative`, the effective lot charge fields with `source`), `lines[]` per sale (`seller_amount`, `buyer_value`, `seller_comm`, `buyer_comm`, `laga` each `{ method, value, amount, rule_label, source: rule|typed|default }`, `buyer_total`, `rate_diff`, `payment_type`, `remaining_qty`, `rate_hint: Option<RateHint>`, `warnings`, per-amount `working`), `summary` for the whole بکری and per lot (the 12.5 lines: gross, freight, commission, labour, munshiana, cash paid, store rent, phone, total deductions, net, buyer commission, لاگا, rate difference, buyer bills credit and cash), `split[]` (`partner_id`, names, `income_paisa`), `cash { cash_after_paisa, below_zero, needs_confirm }`, `errors[]` and `warnings[]` (all together, each with code, field, values). `RateHint` is its own type `{ rate_paisa, source: this_bikri|today|last_day, date }`, never accepted as input (B29).
- `bikri_split_freight { total_freight, lots: Vec<{ key, qty_received }> }` -> `Vec<{ key, freight_paisa }>` (C17). Read-only.
- `assist_truck_memory { truck_no, date }` -> `TruckMemory { last?: { bikri_number, date, adda_id, zamindar_account_id, driver_name, driver_phone }, same_day?: { bikri_number } }` (FR-7.49, FR-7.50). Reads saved بکری.
- `assist_seller_memory { seller_account_id }` -> `SellerMemory { item_id?, packing_id?, marka_id?, from_last_lot }` (FR-7.51).
- `assist_last_rate { item_id, packing_id, date }` -> `Option<RateHint>` for saved data (FR-7.53); inside the preview the unsaved earlier lines of the draft come first.
- Buyer memory is the existing `assist_party_card` (plan 08) plus the payment type default in the line preview (credit if the buyer has credit allowed, else cash; نقدی منڈی and پختہ default to cash, FR-13.4).
- The checks (FR-12.4 to FR-12.7, FR-7.55): sold qty above received is an error; negative net is a warning "بیوپاری کے ذمے" and allowed; credit limit warns, or errors when the setting is block (the buyer's balance plus all his credit lines in this draft); a پختہ line needs name, and phone when credit; rate check when on: a rate more than the percent above or below the item's average rate of the last 7 days (saved facts, weighted by qty, integer maths only) is a warning with the average, never blocks.
- Settings: extend the existing overrides and checks setting (plan 09) with `rate_check_enabled` (default on, E8) and `rate_check_percent` (default 30). A stored value is kept as stored.
- The one `Working` record type and `CalcInfo` from plan 09 are reused and extended with the new amounts; the working is built inside the same function as the amount, and a test checks working result == amount for every amount of Event 1 (B30).
- Speed: `bikri_preview` for 30 lines under 100 ms server time at its runner threshold (NFR-SPD-7 is 150 ms with the 50 ms pause); 50 lots and 500 lines without lag (NFR-SCL-3). Rules are read once per call from the cache.
- Tests: unit tests of `calc_posting` with every worked number; integration tests in `backend/crates/services/tests/bikri_preview.rs` for Event 1, بکری 2, the typed 4%, the override off, the cash note, the partial sale, a negative net, weight items, the credit limit warn and block, and sign-in refusal; `assist.rs` gets the backend part of E2E-26 (fixture data for the memories); property test: preview twice gives the same result, buyer totals sum to the sale part's بنام, Σ line commissions = lot commission. The test also writes `backend/crates/services/tests/fixtures/bikri_preview_event1.json`, which `frontend` copies for its tests.
- The money agent is called before the domain function is written and again to review it.

### Frontend (feature `bikri`; route `/bikri/book`, linked from the بکری side menu)

- The بکری بک screen: header fields, the lots grid and the sale lines grid (`entry-grid` skill, `shared/grid`, `shared/keyboard`), the live summary (per lot and "پوری بکری"), the live partner split, and `PartyCard` beside the active party field (`AccountPicker` everywhere, F40). Truck number list with memory and the same-truck-today notice with "کھولیں" and "نئی بکری جاری رکھیں" (the open action is wired in plan 13). Seller pick fills item, packing and مارکہ from the memory as visible values. The پختہ name and phone fields use the one merged list (FR-7.29, wired fully in plan 13).
- `Enter` order on a sale line: buyer, qty, rate (D3-Q1 team default). Lots grid: the fields typed on most lots (seller, item, packing, qty received, freight, labour, munshiana, نقد) in the main row; the rest behind "مزید" (phone, store rent, مارکہ, weight, store, freight paid by, advance tick). The final column order goes into the hand-off.
- A new sale line's qty starts as the backend's `remaining_qty`, selected; the rate stays empty with the `rate_hint` as grey placeholder text only (F43); `Ctrl+D` copies the cell above as text; `20+5` sums are sent as typed.
- `CalcInfo` (ⓘ, `Ctrl+I`) on every calculated amount, showing the backend `working`, never built in the frontend (F42). The rule tag on a line shows the rule label; a typed value shows "دستی".
- Typing a commission or لاگا on a line (FR-11.10): the rate tag cell is editable when the preview says the setting is on; read-only when off.
- Preview hook: call 50 ms after the last key press, cancel the call before it, never let an old answer overwrite a newer one (F35). The draft lives in a Zustand store above the screen so a language switch loses nothing (F39).
- The amber note "روکڑ صفر سے کم ہو جائے گی" with the cash after the entry, from `cash`. A shared component `shared/ui/CashConfirmDialog` is built here for every later cash-out save: text "اس اندراج کے بعد روکڑ { $amount } ہو جائے گی۔ کیا یہ اندراج درست ہے؟", buttons "ہاں، محفوظ کریں" and "نہیں، واپس جائیں", focus starts on No, Esc is No (FR-6.7). Its use on save comes in plan 13.
- Settings tab: the two new rate check fields.
- Everything on one screen at 1024 × 600 with no sideways scroll (FR-12.2), both languages, the four states, keyboard only. Fluent: `bikri.ftl` in `ur` and `en`, the new errors, new words `# review: pending`.
- Design: `/design` is not available. Fallback of README Section 5: `frontend` designs inside `docs/design/design-system.md` with the `frontend-design` plugin and the `bilingual-screen` skill. It covers the screen with Event 1 filled in Urdu and English at 1024 × 600 and 1920 × 1080, the ⓘ popover, the same-truck notice, the rate hint, remaining qty, a پختہ line, and the cash note and confirm box. It records the screen in `docs/design/` and checks it with headless Chrome screenshots.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: Rust types, command stubs returning `ERR_NOT_IMPLEMENTED`, new error codes, settings fields, `pnpm gen:types` | backend | plan 11 built |
| T2 | `domain::bikri`, `bikri_preview`, assist commands, `cash_after_entry`, tests, fixture JSON, money agent before and after | backend | T1 |
| T3 | Screen, grids, summary, split, hooks, `CashConfirmDialog`, Fluent, component and keyboard tests against canned responses | frontend | T1 |
| T4 | Wire to the real commands; E2E-26 flow; screenshots in both languages | frontend | T2, T3 |

## 6. Acceptance checklist

1. Typing Event 1 returns exactly the numbers of section 3 (`cargo test -p mandi-services --test bikri_preview`); the screen test shows them from the fixture (D3-A5 and D3-A7, screen part).
2. Preview and save share one domain function: the review shows the call path (`domain::bikri::calc_posting`); a timing test for NFR-SPD-7 (D3-A4).
3. E2E-26 backend part passes in `assist.rs`; the screen starts line 2 at 30 with the hint 600 and `Ctrl+D` works (D3-A10).
4. ⓘ shows "12,000 × 6 ÷ 100 = 720" on line 1 seller commission; every calculated amount has a working equal to its amount (D3-A11).
5. With no setting changed, 4% can be typed on a line and saves with method دستی; with the setting off it is refused (D3-A19).
6. Cash 20,000 and نقد 25,000 shows −5,000 in the note; the shared confirm dialog and `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` exist and are tested (D3-A20, the save part in plan 13).
7. The screen fits at 1024 × 600 with no sideways scroll in both languages (headless Chrome, D3-A3); the keyboard order is buyer, qty, rate.
8. No arithmetic on money or quantity in the frontend (a search is shown in the hand-off).
9. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` exit 0; every earlier test still passes.

## 7. Out of scope

Saving, the journal, numbering, drafts, the پختہ register, global search of بکری numbers (plan 13); prints (plan 14); later sales, edit, cancel (plan 16); own goods and الگ کھاتہ lots (plans 16b and 17).

## 8. Open questions

- D3-Q1 (team default): `Enter` stops at buyer, qty and rate.
- Rate check starts on at 30% (E8). Team default; the PRD does not say whether it starts on.
- Cash confirm asks only when the entry lowers cash and the result is below 0 (see the rule above). Team default; the money agent reviews it.
