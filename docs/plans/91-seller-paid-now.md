# Plan 91: A seller's money paid on the spot (موقع پر ادائیگی), the rest cut from what he owes

| | |
|---|---|
| Status | Umar's request and choices, 21-11-2026 (the commission agent: "today's sale is 20,000; give me 5,000 and cut the rest from my previous بنام, then show the total") |
| Branch | `fix/gardens-new-form` |
| PRD | FR-12.23 (the tick beside it), 12.3 نقد, C6, C8, FR-12.9 cash part, FR-6.7, FR-12.12 and FR-19.15 (seller bill), FR-19.12 (day slip), v1.53 line |

## What the books already do

A lot's نقد is cash given to the seller in the same posting: seller بنام نقد, روکڑ جمع; his sale credit (C8) is جمع. So his balance after the sale is: previous + نقد − sale credit. Example: previous 30,000 بنام, today's credit 20,000, نقد 5,000: 30,000 − 20,000 + 5,000 = 15,000 بنام. The other 15,000 is already cut from what he owes. No new money rule and no new posting.

## The change

1. **One field per seller, in his summary on the بکری page:** "موقع پر ادائیگی" (Paid on the spot), beside the پیشگی tick (FR-12.23). The summary shows: پچھلا بقایا (his balance before this بکری, with side), آج کی صافی بکری (today's credit of his lots, before cash), موقع پر ادائیگی, بکری کی باقی رقم، کھاتے میں جمع (credit − paid; it lowers what he owes, or adds to what the shop owes him), نیا بقایا (with side). All from the backend preview.
2. **Spread over his lots:** the backend puts the amount into the نقد of his lots of this بکری, lot by lot in lot order, each up to that lot's credit, any rest on his last lot; the lots' نقد fields stay visible and editable (the summary field is the sum of them; typing the summary field re-spreads). Done in the domain with checked Paisa (money agent).
3. **Limit:** from 0. Above today's credit the app warns "یہ آج کی بکری سے زیادہ ہے، بقایا بڑھے گا" and the save asks to confirm (like the cash question), unless the shop owed him (his previous balance جمع) and the amount is within today's credit + that جمع. Cash below zero asks as always (FR-6.7).
4. **Prints:** the seller bill (FR-19.15) and his day slip (FR-19.12) show the same lines: previous balance, today's net, paid on the spot (نقد ادا کر دیا), cut from previous, new balance with side.
5. Edit: changing the amount in an edit re-spreads it (FR-7.70); a later sale and carried lots keep their own نقد.

## Worked numbers (tests)

- Previous 30,000 بنام, today's credit 20,000, paid 5,000: cut 15,000, new 15,000 بنام; cash −5,000; journal balanced.
- Two lots (credit 12,000 and 8,000), paid 15,000: lot 1 نقد 12,000, lot 2 نقد 3,000.
- Previous 10,000 جمع (shop owed him), credit 20,000, paid 28,000: no warning (within 30,000), new 2,000 جمع.
- Paid 25,000 when previous 30,000 بنام and credit 20,000: warning, confirm, new 35,000 بنام.
- Worked day E2E-1 (رحیم, نقد 5,000) unchanged.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: the summary lines in the preview, the per-seller amount in the draft input spread into lot نقد (domain, checked), the warning flag and the confirm, prints data; tests above; money-calculations review | `backend-seller` |
| T2 | Frontend: the field in each seller's summary, the lines, the warning and confirm, lots' نقد in step with it, prints; ur and en; tests | `frontend-capital` |

## Acceptance

Worked numbers above pass; golden tests unchanged; the field and lines show and print; warning and confirm work; every new test fails before; clippy, typecheck, lint, i18n:check pass.
