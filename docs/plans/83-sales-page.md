# Plan 83: The آج کی بکری tile opens its own sales page

| | |
|---|---|
| Status | Umar's request, 20-11-2026: "آج کی بکری navigates to پچھلی بکری, but it should not; as we built the آج کی کمیشن page, آج کی بکری must be handled in the same way." |
| Branch | `fix/gardens-new-form` (one working tree; one pull request for plans 81 to 83) |
| PRD | FR-9.9 (new in v1.47), FR-9.1, 9.1 row آج کی بکری, FR-9.3 (the model to copy), FR-12.46, FR-19.11, NFR-SPD-24 |

## Goal

Clicking آج کی بکری on the dashboard opens a dedicated sales page (route `/sales`), built the same way as the commission page (`/commission`, FR-9.3). پچھلی بکری stays where it is, in the بکری menu.

## Contract (backend to frontend)

Copy the shape of the commission page commands (find them: `dashboard_commission_month` and its period and party variants, plan 77) so the frontend can reuse its page parts.

1. `dashboard_sales_period(SalesPeriodInput { period: { kind: date|month|year|range, date?, month?, year?, from?, to? }, side: all|buyer|seller, party_id: Option, item_id: Option, goods: all|consigned|trip|garden }) -> SalesPeriod { rows: [ { key (date, or month for a year), bikri_count, qty, gross_paisa, buyer_total_paisa, cash_paisa, credit_paisa } ], totals (same fields), today_marker }`. Money as Paisa numbers. Errors on the field: `ERR_DATE_IN_FUTURE`, `ERR_FORMAT_INVALID`, `ERR_NOT_FOUND` (party, item).
2. `dashboard_sales_day(SalesDayInput { date, side, party_id, item_id, goods, offset, limit }) -> SalesDay { rows: [ { bikri_id, bikri_number, sellers (with lot names), gross_paisa, buyer_total_paisa, cash_paisa, credit_paisa } ], total_count, totals }`.
3. `_print` and `_print_data` twins, and an Excel kind `sales_period`.

A day counts the sales posted that day (later sales and carried lots included), the same rule as the tile and R2 (Q-R5-5). One grouped indexed query each (B31, NFR-SPD-24); totals over the whole filter from the backend.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: the two commands, print and Excel, sign-in check, tests (worked day numbers below, tile equals today's row, month total equals R2, filters, speed plan via EXPLAIN) | `backend-sales` |
| T2 | Frontend: the sales page (route, month navigation like the commission page, day opened in place, period choice, side and party, item and goods filters, print, Excel, four states, both languages, keyboard), the tile now opens `/sales`, the Back parent of `/sales` is the dashboard | `frontend-sales` |

## Worked numbers (after Events 1 to 5, October 2026)

| Day | بکری | نگ | خام بکری | Buyer total | نقدی | ادھار |
|---|---|---|---|---|---|---|
| 10-10-26 | 2 | 60 | 41,400 | 44,364 | 12,800 | 31,564 |

Buyer total 44,364 = 12,880 + 18,684 + 12,800. نقدی 12,800 (شاہد, چکتا). ادھار 31,564 = 44,364 − 12,800. The day opens بکری 1 (رحیم، کیلا-1, gross 29,400) and بکری 2 (کریم، سیب-1, gross 12,000).

## Acceptance

- The tile opens `/sales`, not پچھلی بکری; the Back button returns to the dashboard.
- Today's row equals the tile; the month total equals R2 for the same days; the worked numbers above.
- Period and filters work and their totals come from the backend.
- Every test fails before and passes after; golden tests stay green; typecheck, lint, i18n:check (with its new missing-id guard), the touched tests and `cargo clippy -D warnings` pass.
