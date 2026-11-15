# Plan 77: commission page party filter, kharch page, assistant party commission, analytics combined filters

Asked by Umar, 14-11-2026, while testing plan 76. Same goal run as plan 76 (branch feature/d8-demo-feedback). Built after the plan 76 handoffs; reviewed with plan 76 at the one end gate.

## 1. Goal

On the commission page, the Owner picks گاہک (buyer) or بیوپاری (seller), then one party, and a period (one date, a month, a year or a date range), and sees the commission and sale that came from that one party in that period, with its total. Analytics covers every area the Owner reads by period: sales, commission, sellers, buyers, expenses, gardens, market trips and fruit. The assistant answers the same question in words ("rahim se is mahine kitna commission aur bikri"). Analytics shows a party and an item together in one view, with the buyer filter in plain sight and a buyer × item view.

## 2. Requirements

PRD v1.42: FR-9.3 (party filter), FR-18.1 (buyer filter in plain sight, party + item combined), 18.2 (new view L, buyer × item), FR-7.90 (instant answer for a party's commission and sale, with a button to the filtered commission page). NFR-SPD-24 (pages under 1 second on 10 years of data). Golden tests unchanged.

## 3. Worked numbers (after Events 1 to 5, 10-10-26)

- Commission page, October 2026, filter بیوپاری رحیم: one row 10-10-26: sale (خام بکری) 29,400; seller commission (charged to him) 1,764; commission on his goods 1,764 + 1,764 + 400 = 3,928 (seller + buyer commission + لاگا of his lots, view B "total comm"). Month total the same.
- Filter بیوپاری کریم: 10-10-26 sale 12,000; seller commission 720; on his goods 1,520.
- Filter گاہک اکرم ٹول ٹیکس: 10-10-26 purchase value 12,000; buyer commission 720; لاگا 160; total 880.
- Filter گاہک ارشد ملتانی: purchase value 17,400; buyer commission 1,044; لاگا 240; total 1,284.
- Check, buyer side: اکرم 880 + ارشد 1,284 + شاہد (چکتا) 720 + 80 = 800 gives 2,964 = the day's buyer commission 2,484 + لاگا 480. Seller side: commission on goods رحیم 3,928 + کریم 1,520 = 5,448 = the day's total commission.
- Assistant: "rahim se is mahine kitna commission aur bikri" (today 10-10-26): bikri 29,400 ✓; commission charged to him 1,764 ✓ and on his goods 3,928 ✓ (view B for October, rahim row); button opens the commission page, October, بیوپاری رحیم.

## 4. Contract

- **Period (Umar, 14-11-2026, second request).** The commission read takes `period: { kind: "day" | "month" | "year" | "range", date?, month?, year?, from?, to? }`. Today's input (`month` only) stays valid and means kind month. Rows: a day gives that day's one row (its detail opens below, as today's day detail); a month gives one row per day; a year gives one row per month; a range gives one row per day, paged (B31). Totals over the whole period from the backend. A future date is refused (`ERR_DATE_IN_FUTURE`); a range with from after to is refused (`ERR_RANGE_REVERSED`). It works with and without a party filter. Screen: a period choice (تاریخ, مہینہ, سال, از تا) with the date shortcuts of FR-7.19; the month arrows stay for kind month, and year arrows for kind year. Example: year 2026, بیوپاری رحیم: one row October 2026, 29,400 / 1,764 / 3,928, total the same.
- `dashboard_commission_month` input gains `party: Option<{ side: "buyer" | "seller", account_id: number, chukta_person_key?: string }>`. Null = today's page exactly.
  - With a seller: rows per day with `sale_paisa`, `seller_comm_paisa`, `goods_comm_paisa` (seller + buyer commission + لاگا on his lots), month totals of the same. Read from analytics_facts `SUM(sign × value)` where `seller_account_id = ?`, the same numbers as view B.
  - With a buyer: rows per day with `sale_paisa` (purchase value), `buyer_comm_paisa`, `laga_paisa`, `total_paisa`, month totals. Same numbers as view C.
  - Partner columns are not shown with a party filter (shares belong to whole postings, not to one party) [A].
  - Errors: `ERR_NOT_FOUND` on `party.account_id`; `ERR_WRONG_PARTY_TYPE` when the account is not of the chosen side.
- `dashboard_commission_day` takes the same `party` and lists only that party's lines.
- Print data and `tables_export_excel` kind commission_month take the same `party`, and the print header names the side and party.
- Analytics: new view `L` (buyer × item, گاہک × قسم مال), the mirror of E: one row per buyer, one column per item, total commission in each; drill to lines. `AnalyticsFilters` unchanged (seller_id, buyer, item_id already combine).
- **Analytics coverage audit.** Every view must take day, month and year grouping and a custom range, and combine its party and item filters where it has them: A, B, C, D, E, L (sales, commission, items, sellers, buyers), F and G (partners, expenses, profit), I (gardens), J (market trips), K (fruit). New view M, expenses by head (خرچہ مد وار): one row per دکان خرچہ head with its total, by day, month or year, drill to the vouchers, read from the journal (FR-18.6) [A]. Backend and frontend list any view that misses a grouping or a filter combination in their handoff and fix it.
- **View M, shop and partner kharch (Umar, 14-11-2026, third request).** View M has two parts, both with the period box (date, month, year, range) and day, month or year grouping:
  1. **Shop kharch (دکان خرچہ):** one row per expense head with its total, and a total row for the whole shop.
  2. **Partner kharch (شراکت دار خرچہ), simple (Umar, 14-11-2026):** one row per partner with the kharch he spent himself (ذاتی خرچہ taken, 0603 debits) and a total row. No share of the shop kharch, no returns column. A click opens his kharch entries (date, amount, reason). A partner filter shows one partner. Behind the partner code (FR-17.11): while locked, part 2 shows the code box and part 1 works [A].
  Example (after Events 1 to 5, October 2026): shop بجلی 3,000, total 3,000. Partners: الف 2,000, ب 0, ج 0, total 2,000.
- **Kharch page from the dashboard tile (Umar, 14-11-2026, PRD FR-9.5).** The tile آج کا خرچہ opens a new page `/kharch` with the same two parts as view M (shop kharch per head + total; each partner's own kharch + total), a period choice (date, month default, year, range), a partner filter, a click to the entries, the partner part behind the code, and print, PDF and Excel. The page and view M use ONE backend read (`dashboard_kharch` or the view M read, backend77 decides and tells frontend77), so the two can never differ. Excel kind `kharch`.
- Assistant: no prompt file change. The instant path (FR-7.90) already has seller and buyer commission and sale; it must answer a combined "commission and sale" of one party in one reply, and its button for a party commission figure opens the commission page with that party filter.

**Contract decisions during build (14-11-2026, lead):** a reversed range uses the existing `ERR_DATE_RANGE` (no `ERR_RANGE_REVERSED`); the only new error code is `ERR_WRONG_PARTY_TYPE`. The assistant's analytics catalogue stays A to K, because it must equal the fixed prompt files (Code Standards Q11); the screen catalogue `screen_catalog()` sent by `analytics_catalog` holds all 13 views. Answers about a party's commission get the new source `AnswerSource::Commission`, whose button opens the filtered commission page. A test file owned by robustness (`robustness_live.rs`) gained one match arm for the new source, forced by the type change.

**Review decision (14-11-2026):** views A, B, C, E and L get row grouping by day, month and year as F, I, J, K do (a period cell first, then that period's rows; totals are the ungrouped view). H (top 10) keeps only the period box.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract types and `pnpm gen:types` | backend | plan 76 handoff |
| T2 | Period and party filter on the commission read, day detail, print, Excel; views L and M; the coverage audit of Section 4; assistant combined commission + sale for one party and its button; tests with the numbers of Section 3 | backend | T1 |
| T3 | Commission page: period choice (date, month, year, range) and side toggle (سب / گاہک / بیوپاری) and an AccountPicker of that side only (no match = nothing listed); columns follow the side; filter kept in the URL; print and Excel follow it. Analytics: buyer filter beside the seller filter (not behind مزید فلٹر), view L and view M tabs with charts, and the coverage audit of Section 4 on the screen side. Both languages, four states, keyboard | frontend | T1 |
| T4 | Wire and test end to end | frontend | T2, T3 |
| T5 | Review and robustness with plan 76 at the end gate | reviewer, robustness | T2, T4 |

## 6. Acceptance checklist

- The four filtered pages of Section 3 show exactly those numbers, and the unfiltered page is unchanged (FR-9.3 example 5,448).
- A name that does not match a party of the chosen side lists nothing.
- The commission page by one date, by month, by year and by range gives the right rows and totals, with and without a party; year 2026 with رحیم gives one October row, 29,400 / 1,764 / 3,928.
- View M for October 2026 shows shop بجلی 3,000 (total 3,000) and partner kharch الف 2,000, ب 0, ج 0 (total 2,000); by day, month and year; one partner with the partner filter; part 2 locked without the partner code.
- Every analytics view takes day, month and year grouping and a range; any gap found is listed and fixed.
- The assistant answers "rahim se is mahine kitna commission aur bikri" and "akram se kitna commission mila" with checked figures and the filtered page button.
- View L for October: اکرم ٹول ٹیکس کیلا 880; ارشد ملتانی کیلا 1,284; شاہد سیب 800.
- View A with seller رحیم and item کیلا shows one row; with buyer اکرم and item سیب shows none.
- Print and Excel of the filtered commission page name the party in the header.
- Golden tests unchanged; lint, typecheck, tests, i18n green.

## 7. Out of scope

Partner shares of one party's commission; a new prompt file version for the assistant; any posting change.

## 8. Open questions

None blocking. [A] items: partner columns hidden under a party filter; view L as the buyer × item view; view M as the expenses view.
