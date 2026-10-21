# Plan 16b: Market trips (مارکیٹ سفر) and their goods (اپنا مال)

Deliverable: D4, plan file 16b. Branch: `feature/bikri` (goal run D3+D4). Version 2.0, 22-10-2026 (reworked to PRD v1.7). The file name stays so every link works. The old v1.0 text (the firm's own goods, a stock account, trading sales, cost of goods sold posted) is replaced: it is in git history (`572eed3`).

## 1. Goal

Some partners send a person to a bigger market to buy goods. The goods belong to those partners, not to the firm. The shop sells them in بکری (or the partners sell them in another shop) and takes its normal commission, which is firm income. The trip's own profit or loss goes only to the trip's partners, by their %, on the day the goods are sold. This plan also carries the one database migration (0006) that every later plan of D4, D4R and D5 needs.

## 2. Requirements

PRD 12.10 (FR-12.24 to FR-12.33), FR-12.29 and FR-12.30 (shares and money sources, shared later with gardens), C21, C22 (the 0604 account), C23, C24, C25, FR-12.16 and FR-12.28 (write-off), FR-5.2 and FR-5.3 (مارکیٹ سفر in the main bar; خریداری leaves the بکری side menu, so it has 7 items), FR-7.67 (one trip screen), 9.1 (data for the مارکیٹ سفر tile), FR-10.4 (030119998, the 0604 accounts), FR-10.9 (no opening on 030119998 or a 0604 account), FR-6.7, FR-19.11, E50 to E52, E61 to E63, E65, E66, A53 to A57, A62 to A64. Architecture 6, 9, 15.4 to 15.8. Code Standards B19, B31, 9.3. The money-calculations agent Sections 5.4 and 5.5, before and after. Acceptance: D4-A23, D4-A24, D4-A32, D4-A34 (the part for this plan), and the opening refusals of D4-A31.

## 3. Worked numbers (the tests use exactly these; `worked_day.py` gives the same)

All three runs start from the opening position (PRD 4.3). Partners الف 40, ب 30, ج 30.

**E2E-34.** Trip on 10-10-26, all three partners pay from the shop's cash. Purchase: 20 پیٹی مالٹا (fruit) at 500, freight 400.

| Item | Working | Expected |
|---|---|---|
| Value, cost | 20 × 500 = 10,000; plus freight 400 | total cost 10,400 (520 per نگ) |
| Parts paid from the shop (C23) | 10,400 at 40 / 30 / 30 | 4,160 / 3,120 / 3,120; each posts his 0604 بنام, روکڑ جمع; cash 50,000 − 10,400 = 39,600 |
| Lot of اپنا مال sells 15 to اکرم at 600, ادھاری, no lot charges | value 15 × 600 | 9,000 |
| Seller comm / buyer comm / لاگا | 9,000 × 6 ÷ 100 each; 15 × 8 | 540 / 540 / 120 |
| اکرم pays | 9,000 + 540 + 120 | 9,660 |
| Entry | بنام 9,660; جمع 540 + 540 + 120 + 8,460 | 9,660 = 9,660 |
| Seller credit (C8) to the trip's partners (C25) | 9,000 − 540 = 8,460 at 40 / 30 / 30 | 3,384 / 2,538 / 2,538 on their 0604 جمع |
| Shared income (commission only) | 540 + 540 + 120 = 1,200 | 480 / 360 / 360 by the firm's shares |
| Cost shown, never posted (C21) | 10,400 × 15 ÷ 20 | 7,800 |
| Result on the 15 | 8,460 − 7,800 | 660 |
| Partner totals | −4,160 + 480 + 3,384; −3,120 + 360 + 2,538 | −296 / −222 / −222 |
| Stock left | 20 − 15 | 5 نگ, cost 10,400 − 7,800 = 2,600 |

**E2E-34b.** On 11-10-26 the last 5 spoil and are written off: nothing posts. Cost written off 2,600 (the last نگ take all the cost left). Trip result 8,460 − 10,400 = −1,940 (R33), parts −776 / −582 / −582, already in each total (3,384 − 4,160 = −776). Own stock 0; the trip closes. Partner totals still −296 / −222 / −222. Σ −740 + 65,000 = 64,260 = equity. Trial balance 66,200 = 66,200.

**E2E-41.** Trip: الف 50% from the shop, ب 50% from his pocket, ج not in it. Purchase 80 پیٹی مالٹا at 1,200 = 96,000: parts 48,000 / 48,000. Truck 4,000 (a trip cost, C24 over the one purchase): parts 2,000 / 2,000. الف paid 50,000 from the shop, so cash 0; ب paid 50,000 himself and posts nothing. Cost 100,000 (1,250 per نگ). A lot of اپنا مال sells all 80 to ارشد at 1,400, ادھاری, no lot charges: value 112,000; seller comm 6,720; buyer comm 6,720; لاگا 640; ارشد pays 119,360. Commission 14,080 → 5,632 / 4,224 / 4,224. The seller credit 112,000 − 6,720 = 105,280 → 52,640 / 52,640 (الف and ب) on their 0604. Trip result 5,280; the trip closes. الف −50,000 + 52,640 = +2,640; ب +52,640 (50,000 back + 2,640). Partner totals 8,272 / 56,864 / 4,224. Trial balance 134,360 = 134,360.

Also tested by hand-made numbers: C24 (1,000 over 30 and 20 نگ = 600 and 400); C21 part sale then last نگ (10,400 over 20 نگ: 7,800 then 2,600, never more than 10,400); a sale in another shop (C25: 8,460 at 40 / 30 / 30 = 3,384 / 2,538 / 2,538).

## 4. What to keep and what to change in the working tree

The working tree holds plan 16b work from the paused run, written to the v1.3 design (the firm's own goods), uncommitted and never verified as a whole. It is the starting point, not the design. `changes-v1.6.md` Section 3 is the full list; this is the short form.

**Keep** (still fits): the CHECK edit hook (`widen_checks`, edits `sqlite_schema` in place, fails unless each text is found exactly once) and the migration test harness on a populated file; from 0006 `bikri_lots.purchase_id`, `bikri_sales.cost_paisa` with the snapshot trigger that freezes it, `stock_moves.purchase_id` with the nullable `lot_id` and its reversal trigger, `analytics_facts.purchase_id` and `cost_paisa`; the voucher and posting types `capital_in`, `capital_out`, `khata_out`, `khata_in`, party type `separate`, control 0305; C21 (`split_by_qty` of the cost left over [qty taken, qty left], the last نگ takes all), stored on each line and on the purchase; the own lot path in `calc_posting` (purchase picker, qty taken = qty sold, `ERR_OWN_LOT_NOT_FULLY_SOLD`, `ERR_QTY_EXCEEDS_LOT`, hidden نقد and advance tick, the freight warning, edit and cancel giving back exactly the stored qty and cost); the stock list rows, the write-off commands and their undo, the guard `ERR_OWN_GOODS_ACCOUNT_RESTRICTED`, the purchase list and get shape, the خ- and PUR- numbers. Frontend: the purchases feature (form, list, detail, cancel and write-off boxes) and `OwnLotPanel`, moved under مارکیٹ سفر; `purchases.ftl`.

**Change:**

- **Migration 0006** (unreleased, finished in place; a new file name is fine, the number stays 0006): drop controls 0104 and 0502, accounts 01040001, 04010006, 05020001 and posting type `own_write_off`. Add posting types `purchase`, `trip_cost`, `garden_payment`, `venture_receipt`, `garden_close`, `fruit_taken`; source types `purchase`, `venture`, `fruit`; party type `garden`; control 0306 and control 0604 (باغ، سفر اور پھل, head 06); one 0604 account for every existing partner and `partners.venture_account_id`; the tables `ventures`, `venture_partners`, `venture_entries`, `venture_entry_parts`, `fruit_takes` and `purchases` exactly as Architecture 15.5 and 15.6 (a purchase belongs to a trip: `venture_id`, `supplier_text`, `paid_now_paisa`, `trip_cost_paisa`, `total_cost_paisa`; no supplier account, پختہ columns, payment type or bank); the column `bikri_lots.qty_taken` (0) and stock move kind `taken`; `analytics_facts.venture_id`; `accounts.tag` (TEXT, null; D4R plan 17e builds its use). Rename, with their search columns: 030119998 English "Own Goods (Trips)", 01010001 English "Cash". Its test: a populated golden-day file migrates with every number and the journal chain unchanged; `PRAGMA integrity_check` and `foreign_key_check` pass; a new posting type, a new venture row and a 0604 account insert; `accounts.tag` exists and is null on every migrated account (D4-A32).
- **Partners:** the partner save of plan 11 also makes his 0604 account (code 0604 + serial, English "Gardens, Trips and Fruit, <name>"). An opening on a 0604 account, on 030119998 and on 0306 accounts is refused (FR-10.9).
- **Purchase posting:** no stock account. Paid now = value + freight + labour + other cost. It is split by the trip's % (C23, `split_by_bps`); a shop part posts his 0604 بنام, روکڑ جمع; a pocket part posts nothing; all-pocket posts no journal entry (`journal_entry_id` null). `PurchaseInput` gets `trip_id`, `supplier_text` and each partner's source for this entry; `supplier_account_id`, `pukhta`, `payment`, `bank_account_id` and `PurchaseNumbers.owed` and `ERR_SUPPLIER_REQUIRED` go; the preview returns each partner's part (shop or pocket) with its working.
- **Own lot posting:** the seller credit (C8) goes to each trip partner's 0604 جمع his part (C25), بنام if negative, instead of a trading-sales account; no cost lines in the entry; the shared income of the lot is its commission and فون only (1,200 in E2E-34, not 9,660). `SplitPart.cost_paisa` and `net_paisa` and the cost lines of the old `SummaryTotals`/`SavedSplit` no longer apply: the lot preview shows each trip partner's part of the seller credit, the cost (C21) and the result, each with a `working` (CalcInfo). The cost is stored on the line (`bikri_sales.cost_paisa`) and on the purchase, never posted.
- **Write-off:** `purchases_write_off` and its undo post nothing; `entry_id` is always null; no split. The qty and its C21 cost come off the purchase. The undo gives both back.
- **Result** (FR-12.27): per purchase, per day and per trip: sales money (the seller credits of its own lots and the money from other shops) − cost released (sold and written off). Rename `trading_sales_paisa` accordingly. `purchases_summary()` becomes `trips_summary()`: open trips' نگ left and cost left (the D5 tile).
- **New:** ventures (shares, sources, C23, `venture_entry_parts`, the shares lock), trips (create, list, get, closing by itself, opening again), trip costs (C24, `ERR_TRIP_NO_STOCK`), sales in another shop (C25, posting type `venture_receipt`).

## 5. Contract (the outline; the exact types are recorded in "Contract as built in T1" below)

All commands need sign-in; input wrapped as `{ input: ... }`; money and qty are text in inputs; stubs answer `ERR_NOT_IMPLEMENTED` until T2. Module names follow Tech Stack 5.2: `trips_`, `purchases_`, `ventures_` (no screen of its own).

- `trips_create`, `trips_list` (paged, FR-19.11), `trips_get`, `trips_defaults` (the partners with the % of the last trip, else the firm's shares; the sources of the last trip, else shop), `trips_summary`.
- `trips_cost_preview` / `trips_cost_save` / `trips_cost_cancel` (a trip cost, C24).
- `trips_other_sale_preview` / `trips_other_sale_save` / `trips_other_sale_cancel` (a sale in another shop, C25).
- `purchases_preview` / `purchases_save` / `purchases_cancel` / `purchases_list` / `purchases_get` / `purchases_write_off` / `purchases_write_off_undo`.
- Every entry input carries `parts: [{ partner_id, source: shop | pocket }]`, filled from the trip and changeable on that entry (A54); the preview returns the parts with amounts (C23), the cash after the entry and the FR-6.7 note data. Every save takes `confirm_cash_below_zero` and uses the shared cash check (the shop parts count).
- Lists return `{ rows, total_count, totals }` over the whole filter (B31). Entry numbers: trip سفر-1 / TRP-1, purchase خ-1 / PUR-1; trip costs and other-shop sales have their own series, letters chosen by `backend` and recorded below.
- Error codes (ur and en, `# review: pending`): `ERR_TRIP_CLOSED`, `ERR_TRIP_NO_STOCK`, `ERR_VENTURE_SHARES_LOCKED`, `ERR_SHARES_NOT_100` (already exists), `ERR_PURCHASE_BEFORE_TRIP`, `ERR_PURCHASE_HAS_SALES`, `ERR_PURCHASE_CANCELLED`, `ERR_TRIP_COST_HAS_SALES`, `ERR_OWN_LOT_NOT_FULLY_SOLD`, `ERR_OWN_GOODS_ACCOUNT_RESTRICTED`, `ERR_PARTNER_IN_OPEN_VENTURE`. Reused: `ERR_QTY_EXCEEDS_LOT`, `ERR_OPENING_NOT_ALLOWED`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`, `ERR_WRITE_OFF_EXCEEDS_BALANCE`, `ERR_REASON_REQUIRED`, `ERR_DATE_CLOSED`, `ERR_VERSION_CHANGED`.
- In بکری: `LotInput.purchase_id?` (seller 030119998 plus the purchase); the lot preview and saved view carry the trip partners' parts, cost and result with workings; `bikri_stock_list` returns trip purchase rows marked اپنا مال with their trip (`own_rows`, paged with the rest).

## 6. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract and migration 0006 (finish in place), types, stubs, error codes, `pnpm gen:types`; message to lead and frontend | backend | plan 16 built |
| T2a | Money agent first. Ventures, trips, purchases, trip costs, other-shop sales, write-off and undo, the 0604 account on partner save, guards, unit and integration tests | backend | T1 |
| T2b | Own lot in `calc_posting` (C25 parts, cost, result), edit and cancel of an own lot, stock list rows, `trips_summary`, golden tests E2E-34, E2E-34b, E2E-41, property tests, fixtures, money agent second pass | backend | T2a |
| T3 | Trip screens against the T1 types: main bar item, trips list, new trip form, one trip screen with the type list, purchase form, write-off boxes, own lot panel; Fluent | frontend | T1 |
| T4 | Wire to the real commands (trip screens after T2a, own lot and stock after T2b); keyboard run; screenshots; Windows specs | frontend | T2a, T2b, T3 |

## 7. Acceptance checklist

1. E2E-34, E2E-34b and E2E-41 pass with the numbers above, and the trial balances 66,200 and 134,360 (D4-A23; `cargo test -p mandi-services --test golden_day`).
2. The C21 property test and the trip property test pass: for a trip, Σ parts = payment, Σ shop parts = its روکڑ جمع, money credited = sales money, and once closed Σ over partners of (money credited − shop parts − pocket parts) = its result (D4-A23).
3. Migration 0006 keeps every number and the journal chain of a populated file; the v1.6 accounts exist (a 0604 account for each partner, control 0306, 030119998 "Own Goods (Trips)", روکڑ "Cash"); 01040001, 04010006 and 05020001 do not exist; `accounts.tag` exists and is null on every migrated account (D4-A32).
4. Refusals tested: shares not 100, shares locked after the first entry, a purchase or cost on a closed trip, a purchase dated before its trip, selling more than the stock left, an own lot not fully sold, an opening on 030119998 or a 0604 account, a voucher or جنرل posting to 030119998, a partner made inactive while in an open trip, a trip cost with no stock left.
5. The مارکیٹ سفر section and own lots in بکری work by keyboard in both languages; stock and result per purchase and per trip are shown with each partner's part; no mode, bank or cheque field anywhere; lists page by page (D4-A24, D4-A34; screenshots; Windows specs LOCAL-ONLY).
6. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` exit 0; every earlier test still passes with no expected number changed.

## 8. Out of scope

Gardens (D4R 17c) and fruit taken (D4R 17d): their tables and accounts are in 0006, but there is no code for them here. The partner statement lines and the partner overview columns (plan 17); R33 and the dashboard tile (D5). Buying from a consignor's lot (A49).

## 9. Open questions

None for Umar. Team defaults A53 to A57 and A62 to A64 as in `changes-v1.6.md` Section 9; English purchase number letters PUR-1.

## 10. Contract as built in T1

(To be written by the lead from backend's T1 hand-off.)
