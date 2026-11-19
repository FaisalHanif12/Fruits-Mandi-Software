# Plan 68: Feature 8, a second print button for ادھاری تفصیل

Deliverable D8, item 8. Requirements: PRD v1.32 FR-12.12, R5, FR-19.13. Decision R1, P13 and Q5 in plan 60.

## 1. Goal

On the بکری page, next to the seller-bill print, a second button prints ادھاری تفصیل (R5): the day's buyers and what each bought, with paid and left. It prints the whole day by default, with a choice of this بکری only.

## 2. Worked numbers

انس, (آم 50 . 900), 45,000, after commission and لاگا 48,100, paid 10,000, left 38,100. A buyer who paid in full prints with 0 left (Q1).

## 3. Tasks

T1 backend: R5 print data takes `bikri_id` or a day (depends on plan 65's columns). T2 frontend: the button with its key and the choice, the print template columns, both languages. No new money logic in the frontend.

## 4. Acceptance (D8-A11)

The button prints the day's buyers with paid and left, matching R5 on screen.
