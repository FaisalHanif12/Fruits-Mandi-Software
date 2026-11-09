# Plan 67: Feature 7, find and edit a saved بکری

Deliverable D8, item 7. Requirements: PRD v1.32 FR-12.13, FR-7.70, FR-7.71, FR-12.18 to FR-12.21, FR-15.4. Decision E1 in plan 60. Money agent review before and after.

## 1. Goal

Find any saved بکری by number, date, truck, seller, buyer or item. Correct any typed field, including a wrong buyer or seller. Every record follows.

## 2. Worked numbers (E2E-68, after Events 1 to 5)

بکری 1 line 1 moved from اکرم to طارق (a new buyer, default rules). اکرم: no 12,880 bill, his 10,000 receipt is left over: 10,000 جمع. طارق: 12,880 بنام. Commission and partner shares do not change. Trial balance 95,048 on both sides. Back to اکرم: 85,048 and every original number.

## 3. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Check what really works today. A test per filter and per field. Report what fails. | backend, frontend |
| T2 | `bikri_find` takes `buyer_account_id` (also matches چکتا lines by name if a text is given). | backend |
| T3 | Edit: change of a line's buyer and of a lot's seller, on open days and through FR-21.4; matching re-run for the old and the new party; golden E2E-68. Locks stay as FR-7.70 writes them. | backend |
| T4 | The find dialog gets the buyer filter, and the edit screen lets the buyer and seller pickers change. Keyboard, both languages. | frontend |

## 4. Acceptance (D8-A10)

Find by each filter. The edit with the buyer change gives the numbers of Section 2; changing back gives 85,048.
