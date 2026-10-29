# Plan 35: many items in one entry, and the split between our shop and another shop

Version 1.0, 29-10-2026. Lead: tech-lead. Asked by Umar on 29-10-2026 ("in a single market trip we can add multiple items with their quantity ... the garden goods can consist of multiple items on one day ... today in one entry half of the goods can be sold in our shop, or all, or half in another shop; we have complete flexibility; it must be on record"). Branch `feature/d5-early`. Builds on plans 33 and 34.

## 1. What exists and what is missing (checked in the code, 29-10-2026)

| Point | Today | Gap |
|---|---|---|
| Trip with many items | A trip holds many purchases; each `PurchaseInput` has ONE item | One entry cannot hold several items |
| Garden goods of many items | `ArrivalInput` has ONE item | One entry cannot hold several items |
| Split our shop / another shop | Trip: `OtherSaleInput` takes qty from one purchase (any part). Garden: `GardenOtherSaleInput` has qty and money but no link to goods | The split is not recorded when goods come in; ready lots (plan 33) bring ALL goods left; a garden sale in another shop does not reduce its goods |

## 2. Requirements (PRD FR-12.24, FR-12.32, FR-12.36, FR-12.40, new FR-12.41, v1.8)

R1. **One purchase entry, many items.** The trip purchase form has a grid of item lines: item, packing, qty, weight (weight items), rate, freight, labour, other cost, and the split (R3). One save writes every line in ONE transaction; each line is its own purchase (own number خ-n, own posting exactly as one purchase today, its own cost, C21 and C24 unchanged). The partners' sources (FR-12.30) are chosen once for the entry and apply to every line. A line with errors stops the whole save; errors point to the line and field (`lines[i].qty`). Preview shows each line and the entry total.

R2. **One goods-arrived entry, many items.** The garden مال آیا form has a grid of item lines: item, packing, qty, weight, and the split (R3), with ONE date and ONE date to sell for the entry. One save writes every line in one transaction; each line is its own arrival (own number). No money, as today.

R3. **The split, per line.** Each line has "ہماری دکان" (for our shop) qty and "دوسری دکان" (for another shop) qty, plus an optional shop name. Our shop + another shop = the line's qty. It starts as all for our shop. It can be changed later while goods are left (a "change the split" action), never below what each part has already sold. No money effect.

R4. **Only our-shop goods become ready lots.** Plan 33's ready lots, the own lot and the linked garden lot take only the our-shop part that is left. The other-shop part never comes into a sale.

R5. **A sale in another shop takes from the other-shop part.** Trip (`OtherSaleInput`) and garden (`GardenOtherSaleInput`): the entry picks the line (a purchase or an arrival) and its qty comes off that line's other-shop part (`ERR_QTY_EXCEEDS_LOT` with `available`). The money posts exactly as today (FR-12.32, FR-12.36). To sell more in another shop, first change the split. A garden sale in another shop with no arrival (typed by hand, as today) still works for gardens with no arrivals.

R6. **Shown everywhere stock is shown.** باقی مال, the trip and garden screens, R10, the bell, the dashboard tile: each line shows our-shop left and other-shop left. Attention "waiting" counts our-shop goods (other-shop goods wait for their sale entry and are listed as such).

R7. **Money unchanged.** No posting, formula or golden number changes: a one-line entry gives exactly today's purchase or arrival. E2E-34, 34b, 40, 40b, 41 stay exactly.

## 3. Worked example

Trip 0070, one purchase entry: line 1 مالٹا 80 پیٹی at 1,200 (our shop 50, another shop 30, "حاجی شاپ"); line 2 کیلا 40 پیٹی at 900 (all our shop). Save gives خ-1 (96,000) and خ-2 (36,000), each posted as one purchase today. Next new sale: ready lots مالٹا 50 and کیلا 40. A sale in another shop of 30 مالٹا (money 39,000) takes the 30 from خ-1's other-shop part. خ-1 then has 0 left in both parts once the 50 are sold.

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Contract: multi-line purchase and arrival inputs and preview, the split columns (new migration), change-split command, other-shop sales linked to a line, ready lots and stock lists by part, errors with fields; `pnpm gen:types`; tell frontend | backend |
| T2 | Logic and tests with the money-calculations agent before and after (one transaction, each line = today's posting; a one-line entry equals today exactly; split never below sold; two sales cannot oversell either part); fixtures | backend |
| F1 | Purchase form and garden مال آیا form as line grids (shared/grid, keyboard PRD 7.2), the split per line, change-split action, other-shop sale picks the line, stock lists by part, Fluent ur and en; tests and screenshots | frontend |

Targeted checks only (Umar's rule).

## 5. Acceptance

R1 to R7 each shown by a test or screenshot; a one-line entry identical to today (fixture diff); golden numbers unchanged; keyboard-only entry of a 3-line purchase.

## 6. Contract (backend T1, 29-10-2026) and decisions

Migration 0010: `purchases` and `garden_arrivals` get `other_qty`, `qty_sold_other`, `shop_name` (triggers keep the split true; old purchases sold in other shops get other_qty = qty_sold_other = those sales, so every old number stays true); `venture_entries.arrival_id` links a garden sale in another shop to an arrival. Parts from one pure function `mandi_core::split::StockParts::of` (our_left, other_left, our_sold, other_sold). New commands `purchases_entry_preview/save`, `gardens_arrival_entry_preview/save` (many lines, one transaction, errors `lines[i].field`), `purchases_change_split`, `gardens_arrival_change_split`; old single-item commands stay. New errors ERR_SPLIT_ABOVE_QTY, ERR_SPLIT_BELOW_SOLD, ERR_TOO_MANY_LINES. C21 cost uses the total left (unchanged).

D1 (lead, Umar's "complete flexibility"): a sale in another shop takes from the other-shop part first, and moves any rest from our shop's part in the same save (preview says so). It is refused only above the line's whole qty left. No separate split step is needed.
D2: a write-off comes off our shop's part first. The bell counts our shop's goods; the tile and day close count all goods left.
