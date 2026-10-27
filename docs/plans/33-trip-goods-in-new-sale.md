# Plan 33: today's trip goods come into a new sale by themselves

Version 1.0, 27-10-2026. Lead: tech-lead. Asked by Umar on 27-10-2026 while testing ("when the market trip is added on the current day, it should automatically add to the current بکری lots in selling, because although it is our own goods it must be sold and carry the commission"). His pick from three options: "ready lots in a new sale". Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`.

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
