# Plan 65: Feature 4, money paid on the spot (موقع پر جمع)

Deliverable D8, item 4. Requirements: PRD v1.32 12.4, FR-12.9, FR-15.4, R3, R4, R5, R13, R14, R34, R35; money agent. Decisions P1 to P14 in plan 60 Section 3.1. Money agent review before and after.

## 1. Goal

A money field at the end of each buyer line replaces the ادھاری / نقدی toggle: how much the buyer paid right there. Everything that used the toggle follows the new field. Old saved lines keep their meaning.

## 2. Worked numbers (D8 Feature 4)

| Buyer | Buyer total | Paid | Left |
|---|---|---|---|
| مانی | 21,360 | 21,360 | 0 |
| انس | 48,100 | 10,000 | 38,100 |
| ہارون | 125,060 | 0 | 125,060 |
| Total | 194,520 | 31,360 | 163,160 |

Cash up 31,360. Journal: sale part as before; cash part روکڑ بنام 21,360, مانی جمع 21,360; روکڑ بنام 10,000, انس جمع 10,000. ہارون held 10,000 جمع before, so after the sale 125,060 − 10,000 = 115,060 بنام (E2E-65).
Worked day: اکرم, ارشد paid 0; شاہد (چکتا) paid 12,800: all numbers of Section 4 stay.

## 3. Contract

`SaleInput`: `paid: string | null` replaces `payment_type`.

`LineCalc` (preview): `paid_paisa`, `left_paisa`, `paid_fixed: boolean` (true for P3 buyers; the cell shows it read-only); working for `paid` and `left`. `SummaryTotals`: credit and cash totals are Σ left and Σ paid. `SavedSale`: `paid_paisa`, `left_paisa`.

Refusal: `ERR_PAID_EXCEEDS_TOTAL` with `max` at `lots[i].sales[j].paid`.

Migration `0021_paid_on_the_spot.sql`: P9, and E2E-66 (old cash line = paid in full; old credit line = paid 0).

Print data of R5 gains `paid` and `left` per row and in totals; R3 and R4 gain two columns; R13, R14, R34, R35 follow P5 and P6.

## 4. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract, migration, `pnpm gen:types` | backend | plan 64 |
| T2 | Domain (line, lot, journal, credit limit), matching, edit (P11), reports R3, R4, R5, R13, R14, R34, R35, dues, dashboard; golden E2E-65, E2E-66; property test: paid + left = buyer total, the entry balances | backend | T1 |
| T3 | Grid: the paid column at the end, Enter stop (4 Enters, Q2), fixed cell for P3 buyers, toggle out of LineDetails and the rest; summary words; R3, R4, R5 columns and print templates; both languages; keyboard tests updated | frontend | T1 |
| T4 | End-to-end path with the real preview fixtures | frontend | T2, T3 |

## 5. Acceptance (D8-A7)

The numbers of Section 2 in tests, on screen and in the journal. Old data keeps its numbers. `golden_day` and `assist` unchanged apart from plan 60 Section 3.4.

## 6. Notes

To be filled by the lead.
