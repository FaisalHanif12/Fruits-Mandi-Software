# Plan 33: today's trip goods come into a new sale by themselves

Version 1.1, 27-10-2026 (1.1: Umar's answers on the date to sell, leftovers and garden goods, Section 6). Lead: tech-lead. Asked by Umar on 27-10-2026 while testing ("when the market trip is added on the current day, it should automatically add to the current بکری lots in selling, because although it is our own goods it must be sold and carry the commission"). His pick from three options: "ready lots in a new sale". Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`.

## 1. Goal

Trip goods bought today are never forgotten. A new sale opens with them already in it as lots, so the normal commission and لاگا are charged when they are sold.

## 2. Requirements (PRD FR-12.39, v1.8)

R1. When a **new** بکری opens (`Alt+B`, the menu, or "new" after a save), every purchase of an **open trip** that is **dated the same day as the new بکری** and still has **goods left** is added as a lot, before the user types anything:
- seller اپنا مال (030119998), that purchase chosen, item and packing from the purchase, qty = the goods left;
- one lot per purchase, in purchase number order, after nothing (they are the first lots); the truck lots the user adds come after them.

R2. Each such lot is an ordinary own-goods lot (PRD FR-12.26): the user adds buyers, qty and rate. Seller commission, buyer commission and لاگا follow the rules as for any lot. نقد and the advance tick stay hidden. The lot must be fully sold in this sale; the existing check `ERR_OWN_LOT_NOT_FULLY_SOLD` stays.

R3. The user can remove such a lot before saving (the normal remove-lot action, `Ctrl+Delete` on the lot row). A removed lot is not added back to the same form. Nothing is saved until the user saves; an untouched form with only these lots is not a draft worth keeping (no "save changes?" question for it alone).

R4. Not added: a purchase of a closed trip, one with no goods left, one dated another day, and one already in another lot of this form. Opening an existing بکری, a later sale or an edit never adds lots. If the user changes the date of the new بکری before typing a line, the lots follow the new date.

R5. A short note above the lots says where they came from, for example "آج کی خریداری سے 1 لاٹ شامل" / "1 lot added from today's trip purchases".

R6. Nothing changes in money: the posting of an own lot is exactly as today (E2E-34, E2E-41).

## 3. Worked example

Trip سفر-1 today, purchase خ-1: 80 پیٹی مالٹا. Alt+B: lot 1 = اپنا مال, مالٹا, پیٹی, 80, purchase خ-1. The user sells 80 to ارشد at 1,400: value 112,000, seller commission 6,720, buyer commission 6,720, لاگا 640, ارشد pays 119,360 (E2E-41 numbers, unchanged).

## 4. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Check the purchases list command the own-lot picker uses: it must give, for a date, the purchases of open trips with goods left (date, trip, item, packing, qty left). Add a filter or field only if missing; `pnpm gen:types`; tell `frontend`. Test. | backend | |
| F1 | R1 to R5 on the Sale Book, fixtures, Fluent ur and en, tests (new sale with a purchase today, none, two purchases, closed trip, other day, date change, remove, existing بکری untouched, keyboard and language switch), screenshots | frontend | T1 if a change is needed |

Targeted checks only (Umar's rule).

## 5. Acceptance

R1 to R6 each shown by a test; E2E-34 and E2E-41 numbers unchanged; screenshots of a new sale with a trip lot in ur and en.

## 6. Version 1.1 (Umar, 27-10-2026, his answers to three questions)

These replace R1's "dated the same day" rule and add gardens.

R7. **Date to sell on a trip purchase.** Each purchase gets a box "تاریخ فروخت" (Date to sell). It starts as the purchase date and can be set to the same day or a later day (never earlier than the purchase date; `ERR_SELL_DATE_BEFORE_ENTRY` on `sell_date`). It can be changed while the purchase has goods left. No money effect.

R8. **Which goods come into a new sale.** A new بکری gets a ready lot for every stock record with goods left whose date to sell is **on or before** the بکری's date. Goods not sold on their day stay in باقی مال and come again into the next new sale, until they are sold or written off (Umar: "will go automatically in unsold stock and then will come again the next day"). For a trip purchase, the ready lot is the own lot of R1.

R9. **Garden goods arrived.** The garden screen's type list gets a new entry, **مال آیا** (Goods arrived): date to sell, item, packing, qty (and weight for weight items), note. It posts **no money** and changes no partner total. It is garden stock, shown on the garden screen and in باقی مال marked as the garden's goods.

R10. **A garden ready lot.** For each garden arrival with goods left (R8), a new بکری gets a ready lot: seller = the garden (its 0306 account), item, packing, qty = the goods left, linked to that arrival. Like an own lot, its qty must equal what is sold in it, and the leftover stays in the arrival (so it comes again, R8). Charges and posting are those of a garden lot today (FR-12.36): seller credit to the garden's account, نقد and the advance tick hidden. A garden lot typed by hand without an arrival still works as today.

R11. **Closing and refusals.** A closed garden takes no new arrival (`ERR_GARDEN_CLOSED`), and its arrivals with goods left stop coming into new sales. A garden whose arrivals still have goods left can still be closed only after the goods are sold or written off (written off like trip goods, with a reason, no money). An arrival can be cancelled with a reason only while nothing of it is sold or written off.

R12. **Money unchanged.** No posting of an own lot or a garden lot changes. E2E-34, 34b, 40, 40b, 41 stay exactly.

### Tasks for 1.1

| # | Task | Owner |
|---|---|---|
| T2 | `purchases.sell_date` (new migration; existing rows = their entry date), its check and edit; garden arrivals as a stock record of the garden (backend designs the table and the stock moves; no journal entry), arrivals list, cancel, write-off; a garden lot linked to an arrival takes its qty from it and must be fully sold; the list command for a new sale returns trip purchases and garden arrivals with goods left and date to sell on or before a given date, oldest first; tests (R7 to R12); fixtures; `pnpm gen:types`; tell `frontend` | backend (before plan 34) |
| F2 | Date to sell box on the purchase form and screen; the مال آیا type on the garden screen with its list; ready lots for both kinds in a new sale, oldest date first, then number; the note counts both; باقی مال shows both; tests and screenshots ur and en | frontend |
