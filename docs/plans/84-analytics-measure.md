# Plan 84: Analytics shows sales and every figure, not only commission

| | |
|---|---|
| Status | Umar's request, 20-11-2026: "inside analytics, if I select any buyer or seller and choose the goods type, all analytics are based on the commission; I want the sales or commission or all others as well, not only commission." |
| Branch | `fix/gardens-new-form` (one working tree; one pull request for plans 81 to 84) |
| PRD | FR-18.10 (new in v1.48), 18.1, 18.2, FR-18.1 to FR-18.9, FR-6.1 |

## What is wrong today (from the code)

- Every chart plots only `total_comm`: `reports/src/analytics/chart.rs:51` (`Plan::Ranked("total_comm")`) and `:58` (`Plan::TrendColumns("total_comm")`).
- The matrices fill only commission: view D (item x month), view E (seller x item), view L (buyer x item, `buyer_matrix.rs:72, 86, 215`, `item.rs:54, 95`).
- View H (Top 10) ranks only by total commission.

## Goal

A "ناپ" (Measure) choice on views A, B, C, D, E, G, H and L: خام بکری (sales), خریدار کی رقم (buyer total), نگ, کل کمیشن, بیوپاری کمیشن, گاہک کمیشن, لاگا. It decides the chart series, the matrix cells and their totals, and view H's order. Tables of A, B, C and G keep all their columns. Default خام بکری [A]. Remembered per user, kept in saved views, print and Excel (header names the measure).

## Contract

- Every analytics view input gains `measure: gross | buyer_total | qty | total_comm | seller_comm | buyer_comm | laga` (default `gross`). A view that does not take a measure (F, I, J, K, M) ignores it or refuses it with `ERR_OUT_OF_RANGE` on `measure` (choose one, say which).
- The result names the measure used, so the screen and the print can label it.
- Quantity is a count, not money (its column kind is Count, never formatted as rupees).

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: the measure in the view inputs, charts, matrices D, E, L with totals, view H ranking, print and Excel; tests | `backend-analytics` |
| T2 | Frontend: the ناپ picker on the filter bar for the views that take it, chart and matrix labels, saved views, remembered per user, print and Excel follow; tests in ur and en | `frontend-analytics` |

## Worked numbers (October 2026, after Events 1 to 5, both sides)

| View D by month | خام بکری | کل کمیشن | نگ |
|---|---|---|---|
| کیلا, October | 29,400 | 3,928 | 50 |
| سیب, October | 12,000 | 1,520 | 10 |
| Total | 41,400 | 5,448 | 60 |

View H by خام بکری: items کیلا 29,400 then سیب 12,000; sellers رحیم 29,400 then کریم 12,000; buyers ارشد 17,400, اکرم 12,000, شاہد 12,000.

## Acceptance

- Each measure gives the worked numbers above in charts, matrices and rankings; tables unchanged.
- A buyer, seller, item or goods filter with any measure gives backend totals over the whole filter.
- Existing analytics tests stay green (where a test assumed commission only, it passes the measure `total_comm`); golden tests unchanged.
- Every test fails before and passes after; clippy, fmt (own files), typecheck, lint and i18n:check pass.
