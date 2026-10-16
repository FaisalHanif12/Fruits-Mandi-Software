# Plan 16b: The firm's own goods (خریداری)

Deliverable: D4, plan file 16b. Branch: `feature/bikri` (goal run D3+D4). Version 1.0, 15-10-2026.

## 1. Goal

The firm can buy goods itself, keep them as its own stock, sell them in a بکری like any lot, and see the profit or loss of every purchase. Commission is still taken. Spoiled goods are written off as a loss. The profit or loss is split between the partners. This plan also carries the one database migration that the rest of D4 needs (own goods, partner capital, الگ کھاتہ).

## 2. Requirements

PRD 12.10 (FR-12.24 to FR-12.28), C21, FR-12.16 (write-off of own goods), 4.2, FR-7.59 (a purchase on credit is a payable posting), 9.1 (data for the اپنا مال tile), FR-10.4 (accounts 01040001, 030119998, 04010006, 05020001), FR-6.7, E50, E51, E52, A45 to A47, A49. Architecture 6 and 15.6 (`purchases`, `bikri_lots.purchase_id`, `bikri_sales.cost_paisa`, `stock_moves.purchase_id`, `analytics_facts.purchase_id` and `cost_paisa`), 15.7 (new voucher and posting types). Code Standards B19, 9.3. The money-calculations agent Section 5.4 and Section 6 before and after. E2E-34, E2E-34b. Acceptance: D4-A23, D4-A24, D4-A31 (the اپنا مال opening refusal).

## 3. Worked numbers (the tests use exactly these)

Opening position (PRD 4.3); on 10-10-26 the firm buys 20 پیٹی مالٹا (fruit) at 500 for cash, freight 400 paid in cash. A بکری lot with seller اپنا مال sells 15 to اکرم at 600, ادھاری, no lot charges.

| Item | Working | Expected |
|---|---|---|
| Purchase cost (E2E-34) | value 20 × 500 = 10,000; plus freight 400 | total cost 10,400 (520 per نگ); cash 50,000 − 10,400 = 39,600; stock بنام 10,400 |
| Line value | 15 × 600 | 9,000 |
| Seller commission (default 6%, rule on اپنا مال) | 9,000 × 6 ÷ 100 | 540 |
| Buyer commission | 9,000 × 6 ÷ 100 | 540 |
| لاگا | 15 × 8 | 120 |
| اکرم pays | 9,000 + 540 + 120 | 9,660 |
| Seller credit (C8) goes to تجارتی فروخت | 9,000 − 540 | 8,460 |
| Cost of goods sold (C21) | 10,400 × 15 ÷ 20 | 7,800 |
| Entry | بنام 9,660 + 7,800 = 17,460; جمع 540 + 540 + 120 + 8,460 + 7,800 | 17,460 = 17,460 |
| Profit on the 15 | trading sales 8,460 − cost 7,800 | 660 |
| Split of income 9,660 | 3,864 (leftover 0: 3,864 + 2,898 + 2,898 = 9,660) | 3,864 / 2,898 / 2,898 |
| Split of cost 7,800 | 3,120 / 2,340 / 2,340, negated | −3,120 / −2,340 / −2,340 |
| Net to partners | 3,864 − 3,120; 2,898 − 2,340 | 744 / 558 / 558 (sum 1,860) |
| Stock left | 20 − 15 | 5 نگ, cost 10,400 − 7,800 = 2,600 |
| E2E-34b: the last 5 spoil, 11-10-26 | cost of the 5 (the last نگ take all cost left) | loss 2,600, split −1,040 / −780 / −780; purchase profit or loss 660 − 2,600 = −1,940 (R33); own stock 0; the trial balance balances |
| C21 with a part sale | 10,400 over 20 نگ, 15 sold then the last 5 | 7,800, then 2,600 (never more than 10,400 in all) |

## 4. Contract

New error codes (`errors.ftl` in `ur` and `en`, `# review: pending`): `ERR_OWN_LOT_NOT_FULLY_SOLD` (an own lot must be fully sold in its بکری; values `qty_left`), `ERR_OWN_GOODS_ACCOUNT_RESTRICTED` (اپنا مال is offered only as a lot seller; any other posting to it is refused), `ERR_PURCHASE_HAS_SALES` (cancel refused while anything was sold or written off), `ERR_PURCHASE_CANCELLED`, `ERR_SUPPLIER_REQUIRED` (payment way ادھار needs a supplier). Reused: `ERR_QTY_EXCEEDS_LOT` (selling more than the purchase's stock left), `ERR_OPENING_NOT_ALLOWED` (an opening on اپنا مال, FR-10.9), `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.

### Backend

- **Migration `0004_*.sql`** (a new numbered file; released migrations are never edited, B19). It is the one migration of D4:
  - rebuilds `journal_entries` (posting types add `purchase`, `own_write_off`, `capital_in`, `capital_out`, `khata_out`, `khata_in`; source types add `purchase`) and `vouchers` (types add `capital_in`, `capital_out`, `khata_out`, `khata_in`; keep every constraint and the money-lent triggers), recreating their triggers and indexes; ids, rows and the keyed `chain_hash` values stay byte for byte the same (a test: `verify_journal_chain` and the golden day numbers are identical before and after, on a copy of a populated file);
  - adds `purchases` (Architecture 15.6: number unique, entry_date, supplier_account_id null, پختہ person columns, item, packing, qty, weight_grams null, rate_paisa, value_paisa, freight_paisa, labour_paisa, other_cost_paisa, total_cost_paisa, payment_type credit|cash|bank, bank_account_id, journal_entry_id, status saved|cancelled, cancel_reason, note, qty_sold, qty_written_off, cost_released_paisa, system columns; money columns multiples of 100) with indexes;
  - adds `bikri_lots.purchase_id`, `bikri_sales.cost_paisa` (default 0, multiple of 100), `stock_moves.purchase_id` (and a nullable `journal_entry_id` where a write-off has none), `analytics_facts.purchase_id` and `cost_paisa`;
  - seeds the system accounts (cannot be deleted or made inactive, FR-10.4): control 0104 and account 01040001 اپنا مال سٹاک (Own Goods Stock); account 030119998 اپنا مال (Own Goods (Firm), seller side, reserved serial, never holds a balance) with its charge rule rows found by the default rules; 04010006 تجارتی فروخت (Trading Sales); control 0502 and account 05020001 اپنا مال لاگت (Cost of Own Goods Sold); control 0305 الگ کھاتہ (Separate Khatas, party type separate) for plan 17. Urdu and English names follow the glossary and are `# review: pending`.
  - A test migrates a version-3 file that holds a populated golden day and checks every number. The migration runs after the automatic backup (NFR-REL-13).
- `purchases_preview` / `purchases_save { date, supplier_account_id?, pukhta?, item_id, packing_id, qty, weight?, rate, freight?, labour?, other_cost?, payment: credit|cash|bank, bank_account_id?, note?, confirm_cash_below_zero }` -> `PurchaseSaved { id, number, entry_id, total_cost }`. Posting (FR-12.24): اپنا مال سٹاک بنام total cost; the supplier جمع the value (ادھار), or cash or bank جمع the value; cash or bank جمع freight, labour and other cost. Numbers in their own series خ-1 (Urdu) and PUR-1 (English, team default). The cost-below-zero check of FR-6.7 uses the shared function. A purchase from a پختہ supplier carries the person key (FR-13.2). A supplier on credit is paid later with a normal ادائیگی, and his credit counts as a payable posting (FR-7.59, plan 15).
- `purchases_cancel { id, version, reason }` only while `qty_sold = 0` and `qty_written_off = 0` (`ERR_PURCHASE_HAS_SALES`), by reversal; `purchases_list { from, to, supplier_id?, item_id?, state? }`; `purchases_get { id }` -> the purchase with bought, sold, written off, left (نگ), cost left, days waiting and the profit or loss (FR-12.27: trading sales of its sales − cost of goods sold − cost written off; negative shown as a loss). It is the data of R33 for D5.
- Own lots in `bikri_preview` and `bikri_save` (extends `calc_posting`, FR-12.26): the lot seller is account 030119998 with a `purchase_id`; item and packing come from the purchase; the lot's received qty is taken from the purchase at save and must equal the qty sold in the lot (`ERR_OWN_LOT_NOT_FULLY_SOLD`); selling more than the stock left is `ERR_QTY_EXCEEDS_LOT`; an own lot never has a balance in باقی مال and never a later sale; `cash_paid`, the advance tick and freight stay 0 (a typed freight gives a warning, the cost already holds it). Seller-side commission and lot charges come from the rules on the اپنا مال account (default 6%). The seller credit (C8) posts to تجارتی فروخت (بنام if negative) instead of a seller account. In the same entry, the cost of goods sold posts: اپنا مال لاگت بنام, اپنا مال سٹاک جمع, with C21 worked per sale line in line order (`split_by_qty` of the cost left over [qty taken, qty left]: round down, leftover to the larger qty, tie: the qty taken; the line that takes the last نگ takes all the cost left) and stored on the line (`cost_paisa`). Income and cost both reach the plan 11 hook, so each partner gets his share of the profit or loss. In the preview, the lot summary also returns cost of goods sold and profit, each with its `working`.
- Edit and cancel of an own lot (plan 16 paths): an unchanged own line keeps its stored cost (FR-12.20); a reversal gives back exactly the stored cost and qty to the purchase (`qty_sold`, `cost_released_paisa`, stock moves), and matching and facts follow as in plan 16.
- Write-off of own stock (`bikri_write_off` for a purchase, FR-12.28): the cost of the written-off qty (C21) posts اپنا مال لاگت بنام, اپنا مال سٹاک جمع (posting type `own_write_off`), split between partners as a loss; undo reverses it. A write-off needs a reason and cannot exceed the stock left.
- `bikri_stock_list` (plan 16) now includes purchases with stock left, marked اپنا مال: bought, sold, written off, left, cost left, days waiting (FR-12.25). Data for the dashboard tile اپنا مال (نگ left and cost) is returned by `purchases_summary()`; D5 shows the tile.
- Account guard: any posting to 030119998 from a voucher, a جنرل entry or a purchase supplier field is refused with `ERR_OWN_GOODS_ACCOUNT_RESTRICTED`; the account picker offers it only in the lot seller field.
- Analytics: facts of an own-lot line carry `purchase_id` and `cost_paisa`; reversals carry sign −1 (D5 reads them).
- Tests: `golden_day.rs` gains E2E-34 and E2E-34b with the numbers above; the property test of Code Standards 9.3 (for any sequence of sales and write-offs of one purchase, cost released never exceeds its total cost and equals it once nothing is left; own stock = bought − sold − written off; profit or loss = trading sales − cost released); selling more than the stock left and saving a lot that is not fully sold are refused; no voucher or جنرل can post to اپنا مال; the migration test; sign-in refusal. Money agent before and after.

### Frontend (features `purchases` and `bikri`)

- The خریداری screen in the بکری side menu (form of PRD 12.10 with the cost lines from the backend, the list below it, stock and profit or loss per purchase with a red loss, cancel with a reason); the supplier picker leaves اپنا مال out; payment way, bank account and cash confirm.
- In the lot seller field, اپنا مال is offered with a purchase picker (purchases with stock left); an own lot is marked; the lot summary shows the cost and profit lines, each with `CalcInfo`; the lot hides نقد, the advance tick and freight (warning if typed). Own stock rows in باقی مال are marked اپنا مال, with write-off and its undo.
- Keyboard only, both languages, 1024 × 600, four states, language switch loses nothing. Fluent `purchases.ftl` and additions, errors, `# review: pending`. Design fallback as in plan 12.
- Windows specs written and syntax-checked: purchase, own lot, write-off by keyboard (run LOCAL-ONLY).

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: migration 0004 with its test, types, stubs, error codes, `pnpm gen:types` | backend | plan 16 built |
| T2 | Purchases, own lots in `calc_posting`, cost C21, write-off, stock list, guard, golden tests, money agent | backend | T1 |
| T3 | خریداری screen, own lot UI, باقی مال marks against the stubs; Fluent | frontend | T1 |
| T4 | Wire to the real commands; keyboard run; screenshots | frontend | T2, T3 |

## 6. Acceptance checklist

1. E2E-34 and E2E-34b pass with the numbers above (D4-A23; `cargo test -p mandi-services --test golden_day`).
2. The C21 property test passes (D4-A23).
3. Migration 0004 keeps every number and the journal chain of a populated file (test output); the Windows-style seed of the new accounts exists.
4. An opening on اپنا مال is refused (D4-A31, the opening part); no voucher or جنرل can post to it.
5. خریداری and own lots work by keyboard in both languages; own stock and profit per purchase are shown (D4-A24, screenshots; Windows specs LOCAL-ONLY).
6. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` exit 0; every earlier test still passes.

## 7. Out of scope

Partner capital and الگ کھاتہ screens and postings (plan 17; only the migration and accounts are here); R33 and the dashboard tile (D5); buying from a consignor's lot in the firm's own بکری (A49, open question for the client).

## 8. Open questions

None for Umar. Team defaults: the English purchase number letters are PUR-1; the cost is the purchase's average cost per نگ (A45); profit or loss is shared by the partner shares (A46); freight, labour and other costs are paid at once (A47).
