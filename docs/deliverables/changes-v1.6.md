# Requirement changes v1.6 (read before resuming the D3+D4 run)

Written for `tech-lead`, 19-10-2026. Umar approved every change below (his messages of 18-10-2026 and 19-10-2026). They are in the working tree, not committed yet.

## 1. Short answer

- **What changed.** Market trips replace the firm's own goods: trip goods belong to the trip's partners, not the firm. Gardens are new. Fruit taken is new. The shop has one cash account (no bank, no cheque, no mode). The cash count box is gone. Every amount that reaches a partner gets a dated line in his statement. Large lists load page by page. Analytics and reports are wider (views I to K, R35 to R37), and all of them are Must.
- **The worked day is unchanged.** E2E-1 to E2E-33 and E2E-35 to E2E-39 keep every number; the trial balance is still 85,048 on both sides. E2E-34 and E2E-34b have new numbers (Umar approved). E2E-40, E2E-40b, E2E-41, E2E-42 and E2E-43 are new. All numbers come from `worked_day.py`.
- **Built work stays as built.** Plans 05 to 16 are not reopened in this run. Their v1.6 changes are in the new refresh deliverable `D4R-refresh-v1.6.md`, run after this run and before D5 (README Section 0).
- **Plan 16b is reworked, not restarted.** Its code is uncommitted and migration 0006 is unreleased, so it is finished to v1.6 now (Section 3). This is cheaper than building the old design and undoing it later.
- **Migration 0006 holds every v1.6 database change** (trips, gardens, fruit, 0604 and 0306), so plan 17 and D4R need no migration.
- **Order:** commit the documents (Section 6), finish plan 16b, then plan 17, then the one end gate and the D3+D4 DONE REPORT; then D4R; then D5.

## 2. The changes

| # | Change | PRD | Built today? | Who builds it | Proved by |
|---|---|---|---|---|---|
| 1 | Market trips: some partners buy goods; each partner's part is paid from the shop's cash (off his total at once, on his 0604 account) or his own pocket (recorded only); the goods are sold in بکری or another shop; the commission is firm income split by the firm's shares; the seller credit goes to the trip's partners by the trip's % the same day; the cost (C21) is shown, never posted | 12.10, FR-12.24 to FR-12.33, C21, C23 to C25 | Half built as the firm's own goods, uncommitted | Plan 16b | D4-A23, D4-A24, D4-A32 |
| 2 | A trip cost is spread when it is saved over the purchases with goods left; none left is refused (`ERR_TRIP_NO_STOCK`) | FR-12.31, C24, A64 | No | Plan 16b | D4-A23 |
| 3 | Gardens: shares and sources as trips; installments and costs; sales in بکری (seller credit to the garden's account) or another shop; running balance; closing by the Owner splits the garden's balance by its %; a closed garden's lot takes no later sale and gives no fruit | 12.11, FR-12.34 to FR-12.38, C26 | No | D4R plan 17c | D4R-A4 to D4R-A6 |
| 4 | Fruit taken: for one partner (his total only) or for guests (equal split, C27); from our lot (seller credited, no commission, lot balance down; trip goods not offered) or another agent's shop | 16.7, FR-16.10 to FR-16.12, C16, C27 | No | D4R plan 17d | D4R-A7 |
| 5 | One cash account: no mode, bank or cheque anywhere (ادھار رقم واپسی keeps its salary deduction); 01010001 English "Cash"; 01020001 never offered; no bank master | 15.1, 16, 17.4, FR-10.4, 10.3 | Bank and cheque modes built in plan 15; bank master in plan 06 | New screens of 16b and 17: cash only. Built screens: D4R plan 17b | D4-A34, D4R-A1 |
| 6 | No cash count box | FR-16.1 | No | Plan 17 | D4-A17 |
| 7 | Statement lines: one dated line, with heading and reason, for every amount that reaches a partner; note lines for a month close, a trip's closing and a write-off of trip goods | FR-17.10 | No | Plan 17 (shares, capital, drawings, trips); D4R (gardens, fruit) | D4-A33, D4R-A8 |
| 8 | کل رقم includes the partner's 0604 account | C22, FR-17.9 | No | Plan 17 | D4-A26 |
| 9 | Open gardens and trips under each partner's card | 17.3 | No | D4R plan 17c | D4R-A5 |
| 10 | Lists page by page, totals from the backend over the whole filter | FR-19.11, NFR-SPD-24, B31, F34 | Only `accounts_list` | New lists in 16b and 17; built lists in D4R 17b; reports and analytics in D5 | D4-A34, D4R-A3, D5-A25 |
| 11 | E2E-38's الگ کھاتہ is خاندانی کھاتہ (Family Khata); numbers unchanged | 27.2 | No | Plan 17 | D4-A27 |
| 12 | Main bar: باغات item 5, مارکیٹ سفر item 6; خریداری leaves the بکری side menu (7 items); پھل is a tab next to خرچہ | FR-5.2, FR-5.3, 16 | No (the 8th side menu item is uncommitted) | 16b, 17c, 17d | D4-A24, D4R-A5 |
| 13 | Accounts: controls 0104 and 0502 and accounts 01040001, 04010006, 05020001 removed; 030119998 English "Own Goods (Trips)"; control 0306 (gardens); control 0604 and one 0604 account per partner; no opening on 01020001, 0306 or 0604 | FR-10.4, FR-10.9 | Seeded in the uncommitted 0006 | Plan 16b (migration, 0604 openings); 17b (01020001); 17c (0306) | D4-A32, D4R-A1, D4R-A6 |
| 14 | Dashboard tiles مارکیٹ سفر and باغات replace اپنا مال | 9.1 | No | D5 plan 19 (data from 16b and 17c) | D5-A1 |
| 15 | Reports: R7 with commission; R33 is the trip report; R34 adds trips, gardens and fruit; R23 and R25 without mode; new R35 (party day sheet), R36 (gardens), R37 (fruit) | 19.2, FR-19.12 | No | D5 plan 20 | D5-A3, D5-A21, D5-A23 |
| 16 | Analytics: every view A to K is Must; views I, J, K new; view F side by side; filters for garden, trip and goods; print and Excel by day, month or year | 18 | No | D5 plan 21 | D5-A24 |

New Urdu words (with English) come from the PRD glossary: مارکیٹ سفر (Market Trip), باغ / باغات (Garden / Gardens), ٹھیکہ (Rent), قسط (Installment), دکان سے / اپنی جیب سے (From the Shop / Own Pocket), پھل (Fruit Taken), باغ، سفر اور پھل (Gardens, Trips and Fruit). Mark new messages `# review: pending`.

## 3. Plan 16b: what to keep and what to change in the working tree

D4 Section 16b is the brief. Rewrite `docs/plans/16b-own-goods.md` from it (keep the file name; replace its "Contract as built in T1" with the v1.6 contract).

**Keep** (it still fits v1.6):

- The CHECK edit hook (`widen_checks`) and its exactly-once check; the migration test harness on a populated file.
- From 0006: `bikri_lots.purchase_id`, `bikri_sales.cost_paisa` and the snapshot trigger that freezes it, `stock_moves.purchase_id` with the nullable `lot_id` and the reversal trigger, `analytics_facts.purchase_id` and `cost_paisa`; posting and voucher types `capital_in`, `capital_out`, `khata_out`, `khata_in`; party type `separate` and control 0305.
- C21 (`split_by_qty` of the cost left, the last نگ takes all), stored on each line and on the purchase.
- The own lot path in `calc_posting`: purchase picker, qty taken = qty sold, `ERR_OWN_LOT_NOT_FULLY_SOLD`, `ERR_QTY_EXCEEDS_LOT`, hidden نقد and advance tick, the freight warning, edit and cancel giving back exactly the stored qty and cost.
- The trip rows in `bikri_stock_list`, the write-off commands and their undo, the guard `ERR_OWN_GOODS_ACCOUNT_RESTRICTED`, the purchase list and get shape, the خ- and PUR- numbers.
- Frontend: the purchases feature (form, list, detail, cancel and write-off boxes) and `OwnLotPanel`, moved under مارکیٹ سفر; `purchases.ftl`.

**Change:**

- **Migration 0006:** drop controls 0104 and 0502, accounts 01040001, 04010006 and 05020001, and posting type `own_write_off`. Add posting types `trip_cost`, `garden_payment`, `venture_receipt`, `garden_close`, `fruit_taken`, source types `venture` and `fruit`, party type `garden`, controls 0306 and 0604, one 0604 account per existing partner and `partners.venture_account_id`, the tables `ventures`, `venture_partners`, `venture_entries`, `venture_entry_parts`, `fruit_takes`, the column `bikri_lots.qty_taken` and stock move kind `taken`, `analytics_facts.venture_id`. Reshape `purchases` to Architecture 15.6 (`venture_id`, `supplier_text`, `paid_now_paisa`, `trip_cost_paisa`; no supplier account, پختہ columns, payment type or bank). Rename 030119998 (English "Own Goods (Trips)") and 01010001 (English "Cash") with their search columns. The file name may stay; update its header comment.
- **Partners:** the partner save of plan 11 also makes his 0604 account.
- **Purchase posting:** no stock account. The paid-now amount is split by the trip's % (C23): each shop part posts his 0604 بنام, روکڑ جمع; a pocket part posts nothing. `PurchaseInput` gets `trip_id`, a supplier text and each partner's source; `supplier_account_id`, `pukhta`, `payment` and `bank_account_id` go; `PurchaseNumbers.owed` and `ERR_SUPPLIER_REQUIRED` go; the preview returns each partner's part.
- **Own lot posting:** the seller credit (C8) goes to each trip partner's 0604 جمع (C25), not to تجارتی فروخت; no cost lines; the shared income of the lot is its commission only (1,200 in E2E-34, not 9,660). `SplitPart.cost_paisa` and `net_paisa` and the cost lines of `SummaryTotals` and `SavedSplit` no longer apply; the preview shows each trip partner's part, the cost and the result, each with its working.
- **Write-off:** posts nothing; its `entry_id` is always null; no split.
- **Result:** sales money (seller credits and other-shop money) − cost released; rename `trading_sales_paisa` to match. `purchases_summary` becomes `trips_summary` (open trips' نگ left and cost left).
- **New:** ventures (shares, sources, C23, `venture_entry_parts`, the shares lock), trips (create, list, get, close by itself once it has a purchase and nothing is left, open again on a reversal), trip costs (C24, `ERR_TRIP_NO_STOCK`), sales in another shop (C25, `venture_receipt`).
- **Golden tests:** replace `golden_day/own_goods.rs` with the v1.6 E2E-34 and E2E-34b numbers, and add E2E-41 (D4 Section 16b worked checks).
- **Frontend:** the 8th بکری side menu item goes; مارکیٹ سفر joins the main bar; the trip screens; no payment way or bank field; `OwnLotPanel` shows each trip partner's part instead of the old profit split.

## 4. Plan 17 changes

Rewrite `docs/plans/17-cash-book-and-day-close.md` from D4 Section 17:

- Every command: no `mode` or `bank_account_id`; money posts روکڑ. `vouchers_save_money_lent_return` keeps the salary deduction (mode cash or salary).
- No cash count box (D4-A17).
- The partner statement data: one line per amount, with heading, reason and the running کل رقم, from posted records (FR-17.10), including the trip lines of plan 16b and the note lines; C22 with the 0604 account; the overview column باغ، سفر اور پھل.
- E2E-38 uses خاندانی کھاتہ (Family Khata); every number stays.
- Every list pages (FR-19.11).

## 5. D4R and D5

- `D4R-refresh-v1.6.md` (new): plans 17b (one cash account and paging on the screens built in plans 05 to 16), 17c (gardens, E2E-40, E2E-40b), 17d (fruit taken, E2E-42). Its goal line is in README Section 3.
- `D5-reports-backup-golive.md` (v1.5): the tiles, R35 to R37, the changed reports, analytics views A to K, paging, E2E-43, D5-A23 to D5-A25. Its goal line now names E2E-1 to E2E-43 and D5-A1 to D5-A25.
- D1, D2 and D3 are not changed. D4 Sections 15 and 16 describe the work as built.

## 6. Files in the documents commit

**Replaced by `changes-v1.7.md` Section 5** (22-10-2026): the v1.6 and v1.7 documents go in one commit, `docs: requirements v1.6 and v1.7`. The list below is the v1.6 part of it.

One commit, `docs: requirements v1.6`, with only these files:

- `.claude/rules/PRD_Mandi_Software_v1.md` (1.6), `Architecture_v1.md` (1.6), `Code_Standards_v1.md` (1.5), `NFR_Mandi_Software_v1.md` (1.4), `Tech_Stack_v1.md` (1.4)
- `.claude/agents/money-calculations.md` (1.5), `.claude/CLAUDE.md`
- `.claude/skills/requirement-change/scripts/worked_day.py`, `.claude/skills/tauri-command/templates.md`, `.claude/skills/urdu-english-text/SKILL.md`, `.claude/skills/urdu-english-text/style-guide.md`, `.claude/skills/print-and-export/SKILL.md`
- `docs/deliverables/README.md`, `D4-money-in-out.md`, `D4R-refresh-v1.6.md`, `D5-reports-backup-golive.md`, and this file

Do not put the uncommitted plan 16b code in this commit. `.v16src.tgz` in the project folder is a transfer file: delete it, never commit it.

## 7. Files the lead updates (lead-owned)

- `docs/plans/16b-own-goods.md` and `docs/plans/17-cash-book-and-day-close.md`: rewritten from D4 (Sections 3 and 4 above).
- `docs/deliverables/status.md`: a new "Current" note (resumed under v1.6; plan 16b being reworked), and rows for D4R plans 17b, 17c and 17d.
- `docs/deliverables/progress.json`: plans `17b-refresh-built-screens`, `17c-gardens`, `17d-fruit-taken` as waiting.
- `docs/design/design-system.md`: remove the mode, bank and cheque parts (with D4R plan 17b).
- `docs/release/whats-new.md`: when the Windows phase publishes.

## 8. How to resume, step by step

1. `git status`: the documents above and the plan 16b work are uncommitted. Commit the documents (Section 6) and push.
2. Run `python3 .claude/skills/requirement-change/scripts/check_docs.py` (it prints OK) and `python3 .claude/skills/requirement-change/scripts/worked_day.py` (it prints the E2E-34, E2E-34b and E2E-40 to E2E-43 numbers).
3. Rewrite plan file 16b (Section 3). Spawn a fresh `backend` and `frontend` with the plan file as the brief. `backend` first finishes migration 0006 and the contract types (T1 again, short), runs `pnpm gen:types`, then T2 with the money agent before and after. `frontend` continues T3 against the new types, then T4.
4. Run the full check script once on the idle machine; commit plan 16b (one `feat(trips)` commit, one `docs` commit) and push.
5. Plan 17 the same way (Section 4).
6. The end gate for the whole D3+D4 run, the fix loop, the DONE REPORT.
7. Then the D4R goal line, then D5.

## 9. Still open, with the team default (do not stop for these)

A53 to A64 in PRD Section 24, each built as its default: purchases paid at once (A53); sources set per venture, changeable per entry (A54); shares fixed after the first entry (A55); the first trip and every garden start from the firm's shares (A56); money from another shop comes into روکڑ (A57); fruit from our lot credits the seller with no commission (A58); guests split equally (A59); another agent is a seller account (A60); a garden lot pays the normal seller charges (A61); a trip closes by itself, a garden by the Owner (A62); a partner in an open garden or trip cannot be made inactive (A63); a trip cost is spread over the purchases with goods left (A64). List them in the DONE REPORT under open questions.

## 10. Document versions now

| Document | Version |
|---|---|
| PRD | 1.6 |
| Architecture | 1.6 |
| Code Standards | 1.5 |
| NFR | 1.4 |
| Tech Stack | 1.4 |
| Money-calculations agent | 1.5 |
| Deliverables | README 1.6, D4 1.5, D4R 1.0, D5 1.5 (D1, D2 and D3 unchanged) |

`python3 .claude/skills/requirement-change/scripts/check_docs.py` passes.
