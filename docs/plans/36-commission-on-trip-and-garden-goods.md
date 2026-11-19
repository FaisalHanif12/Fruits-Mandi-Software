# Plan 36: show the commission earned on trip and garden goods

Version 1.0, 29-10-2026. Lead: tech-lead. Asked by Umar on 29-10-2026 with a picture of the trips list ("we should also mention the commission from those goods, on both trip and garden, each goods"). Branch `feature/d5-early`. Display only.

## Requirements (PRD FR-12.27, FR-12.37, v1.8)

R1. A **کمیشن** (Commission) figure: the shop's commission on the goods sold **in our shop** = seller commission + buyer commission + لاگا, summed from the stored sale lines (analytics facts), with reversals included. A sale in another shop adds nothing (no commission). It uses exactly the definition of analytics views J and I and report R36 (seller + buyer + لاگا). Report R33's commission column is the SELLER commission only (E2E-34: 540), so R33 and the trip card differ by design (correction after backend T1).

R2. Shown in: the trips list and the gardens list (one column per trip or garden, and in the total row); each purchase line on the trip screen and each goods-arrived line on the garden screen; the trip card and the garden card (total); with ⓘ working (CalcInfo, from the backend).

R3. Nothing posts and nothing changes in money. E2E-34: the trip's commission is 540 + 540 + 120 = 1,200. E2E-41: 6,720 + 6,720 + 640 = 14,080. E2E-40 (garden): 3,600 + 3,600 + 800 = 8,000.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Fields `commission_paisa` (and its working) on the trip and garden rows, purchase rows, arrival rows, and the summary; from one indexed query (B23, B31); tests with E2E-34, 40 and 41; fixtures; `pnpm gen:types`; tell frontend | backend |
| F1 | The columns and card lines (money column type, both languages), tests, screenshots ur and en | frontend |

Targeted checks only.

## Built (29-10-2026)

backend T1: `commission_paisa` and `commission_working` on TripRow, GardenRow, PurchaseRow, ArrivalRow, their list totals and the dashboard summaries; one grouped query per list (new db/src/commission.rs, no migration). frontend F1: the columns, total rows and card lines with ⓘ. Money review: nothing must fix. E2E-34 1,200, E2E-41 14,080, E2E-40 8,000.

Open for Umar: R33 shows the seller commission only (540) where the trip card shows 1,200. Should R33 add a total commission column?
