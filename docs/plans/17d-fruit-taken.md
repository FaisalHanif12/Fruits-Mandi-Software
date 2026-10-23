# Plan 17d: fruit taken (پھل)

Version 1.0, 23-10-2026. Lead: tech-lead. Brief: `docs/deliverables/D4R-refresh-v1.6.md` Section 2 "17d-fruit-taken" (full text, worked check E2E-42). Branch `feature/refresh-v16`. Needs 17c (fruit can come from a garden's lot).

## 1. Goal

On the پھل tab next to خرچہ the Owner records fruit a partner took home or fruit served to the shop's guests, by item and quantity, from a lot of our shop or from another agent's shop, and every partner's statement shows it.

## 2. Requirements

PRD 16.7 (FR-16.10 to FR-16.12), C16, C27, FR-7.53 (rate hint only), FR-7.59 (a fruit credit is a payable posting), FR-15.8 (never adjusts an advance), FR-17.10, FR-19.11, E67, A58 to A60; Architecture 15.6 (`fruit_takes`, `bikri_lots.qty_taken`, stock move kind `taken`); money agent Sections 5.5 and 8.2a. Acceptance: D4R-A7 and the fruit part of D4R-A8.

## 3. Worked numbers

E2E-42 (after E2E-16, 5 کیلا left in رحیم's lot): home fruit 2 × 580 = 1,160, الف's total −1,160, رحیم's payable 1,590 + 1,160 = 2,750, lot balance 3; guest fruit 3,000 split 1,000 / 1,000 / 1,000, حاجی نذیر payable 3,000; cash 45,000 unchanged; totals −736 / 68 / 68; Σ −600 + 65,000 = 64,400 = equity; trial balance 77,610. C27: 1,000 over 3 partners gives 334 / 333 / 333.

## 4. Contract

`backend` T1 writes the exact types (`fruit_preview`, `fruit_save`, `fruit_list`, `fruit_edit`, `fruit_cancel`, the lot picker for the source, errors including `ERR_QTY_EXCEEDS_LOT`); the lead copies them into Section 10. Posting type `fruit_taken`, numbers پھل-1 and FRT-1. No migration.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, codes, stubs, `pnpm gen:types`, command tables | `backend` | 17c T2 |
| T2 | Logic with the money-calculations agent before and for review: partner and guest split (C27, never the firm's shares), lot source (qty_taken, stock move, seller credit, no commission, C16 with taken), other-shop source, payable working, statement fruit lines, lists paged, golden E2E-42, tests and refusals, fixtures | `backend` | T1 |
| F1 | The پھل tab beside خرچہ: for whom, source with the lot or agent picker, item, qty, rate (last rate as a hint only), value and each partner's part from the backend before save, the list with partner, item and date filters, Fluent ur and en | `frontend` | T1 |
| F2 | Wire to T2 fixtures, shape tests, keyboard and language-switch tests, screenshots, Windows spec | `frontend` | T2 |

## 6. Acceptance checklist

D4R-A7 (E2E-42, C27 334 / 333 / 333, a fruit credit is payable and never adjusts an advance, more than the lot balance refused) and the fruit lines of D4R-A8.

## 7. Out of scope

R37, analytics view K and the print and Excel of the list (D5).

## 8. Open questions

None (A58 to A60 are built as their defaults).

## 10. Contract as built in T1

(To be written by the lead from the backend T1 hand-off.)
