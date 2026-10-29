# Plan 37: R33 total commission, and a per-lot charge once per goods-arrived entry

Version 1.0, 30-10-2026. Lead: tech-lead. Umar's answers of 30-10-2026: "Yes it should be the total commission column" (R33) and "goods arrived entry" (a garden's per-lot charge is taken once per goods-arrived entry). Branch `feature/d5-early`.

## Requirements

R1. **R33 total commission.** R33 (تجارتی منافع / نقصان) gets a column **کل کمیشن** (Total commission) = seller commission + buyer commission + لاگا of the sales in our shop, with the same definition as plan 36 (the trip card, analytics J). The existing commission column stays and is labelled as the seller's commission. Rows, totals, print and Excel. E2E-34: seller commission 540, total commission 1,200. E2E-41: 6,720 and 14,080. No result changes (the result is after the seller commission, as now).

R2. **A per-lot charge once per goods-arrived entry.** For a garden lot linked to a goods-arrived entry (plan 33), a charge worked out **per lot by a rule** (seller commission per lot, labour, munshiana or phone per lot) is charged only on the FIRST active lot of that entry. Later lots of the same entry carry 0 for that charge, with a working that says it was charged on the entry's first lot. If the lot that carries it is cancelled, or edited so that it no longer exists, the next NEW lot of that entry carries it. It is never charged twice and never lost while the entry still has goods sold in a later lot. Per-نگ and percent charges are unchanged. An amount typed by hand on a lot is kept as typed. Hand-typed garden lots without an entry, and trip lots, are unchanged (per lot, as today).

R3. **Money and tests.** Only R2 changes an amount, and only for per-lot rules on linked garden lots split over several lots. No golden number changes (E2E-40 is one lot). Tests: an entry of 100 sold 60 then 40 with a per-lot munshiana 50 (first 50, second 0); the first lot cancelled, then a third lot (carries 50); an edit of the first lot keeps its stored charge (FR-12.20); a per-نگ rule unchanged; a hand-typed amount kept.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | R33 column (rows, totals, print, Excel, fixtures); R2 in the domain lot charge resolution with the money-calculations agent before and after; tests; `pnpm gen:types` if types change; tell frontend | backend |
| F1 | R33 screen column and words (کل کمیشن / Total Commission, the seller column label), the "charged on the entry's first lot" working words, tests, screenshots | frontend |

Targeted checks only.
