# Plan 79: Dashboard pages and final tweaks (7 points)

| | |
|---|---|
| Asked by | Umar, 18-11-2026, after testing plan 78 |
| Branch | `feature/d8-demo-feedback` (Mac phase) |
| Status | Approved by Umar, 18-11-2026. Defaults marked [A] can be changed by him. |
| Main focus | **The dashboard.** Every tile opens its own page, and that page does the tile's job. |
| Screenshots | `docs/plans/assets/79/` (index in Section 9) |
| Overlap | Where this plan and plans 76 to 78 say different things, this plan wins. |

## 0. How to use this plan

Each item says where it is, what happens now, what is needed, and an example. It does not say how to build it. The team finds the cause, designs the change, and writes the design into Section 8 before writing code.

Rules for the whole plan:

- Requirement documents change first (PRD and the others it touches), with version and change log lines.
- No golden test number changes. If an item seems to need one, stop and ask Umar.
- Every figure comes from the backend. Screens and prints add nothing up.
- Both languages, every window size, keyboard only, and the four states (loading, empty, error, data).
- Every new page loads page by page with its totals from the backend (FR-19.11), and has a 7-day or 1-month window where its data grows over the years.
- Partner figures stay behind the partner code (FR-17.11).

---

## 1. Filters start at "سب" (All) (U1)

**Where.** The page that opens from the dashboard tile بیوپاریوں کو ادائیگی باقی (screenshot s01). Also every other page opened from the dashboard, and every report or list with a party filter.

**Now.** The کھاتہ filter has no "سب" (All) choice. The page shows one seller at a time (احسان), not the total of all sellers.

**Needed.**

- Every party filter has "سب" (All) as its first choice.
- A page opens with "سب" selected, so it shows every party and the full total.
- The user can then narrow it to one buyer or one seller.
- Going back to "سب" shows everyone again.

**Example.** بیوپاریوں کو ادائیگی باقی opens with "سب": فیصل F 72,740, احسان 120,780, عدنان 9,260, total 202,780, which equals the tile. Picking احسان shows only his row, 120,780.

**For the team.** Check every page and report with a party filter. List in Section 8 the ones that had no "سب" and were fixed.

---

## 2. New dashboard pages: sellers, total dues and today's نقدی (U2)

### 2a. Seller dues page, the same as the buyer dues page

**Where.** The dashboard tiles بیوپاریوں کو ادائیگی باقی and بیوپاریوں کے ذمے (screenshot s02).

**Now.** The buyer dues page (FR-9.4) is simple and works well. Sellers have no page like it.

**Needed.**

- **One list of sellers,** like the buyer dues page:
  - every seller with a balance;
  - his code, name with tag, city, balance with its side (بنام or جمع), and the date his balance last changed;
  - a grand total.
- **A side filter:**
  - سب (default);
  - "shop owes the seller" (بیوپاریوں کو ادائیگی باقی);
  - "seller owes the shop" (بیوپاریوں کے ذمے).
- **Totals for each side** sit at the top. Each side's total equals its tile.
- **A click on a seller** opens his full history:
  - his opening balance first;
  - every advance (پیشگی);
  - every بکری, as one line each (plan 78 B2);
  - every ادائیگی;
  - every return of an advance;
  - a running balance after each line.
- Both tiles open this page.

**Example.** The page opens with "سب". At the top: shop owes sellers 202,780; sellers owe the shop 0. A click on احسان shows: ابتدائی بقایا 20,000 جمع; بکری 1 جمع 160,280 (balance 180,280 جمع); ادائیگی 1 بنام 100,000 (balance 80,280 جمع); بکری 2 جمع 40,500 (balance 120,780 جمع).

### 2b. کل اگراہی gets its own page

**Where.** The dashboard tile کل اگراہی.

**Now.** It opens the buyer dues page. That shows only buyers, so it does not match the tile.

**Needed.**

- **Its own page with sections:**
  - **Buyers:** every buyer who owes, with a buyers total.
  - **Sellers:** every seller who owes the shop, with a sellers total.
  - **Dead amount:** every person in the dead amount, with its total.
  - **Others:** people who borrowed (ادھار رقم: staff, garden owners and others), with their total.
- **The grand total** of everyone who owes the shop is the last line.
- Each row opens that person's history.

**Open question Q1.** Today the tile کل اگراہی counts buyers, sellers and the dead amount only. Money lent has its own tile (PRD 9.1). Umar asked for borrowed money to be included as well. Default [A]: the page shows the Others section with its own total, and the grand total and the tile then include it. The PRD 9.1 definition changes to match. If Umar says no, the Others section shows below the grand total as information.

**Example (after Events 1 to 5).** Buyers: اکرم 2,880, ارشد 18,684, total 21,564. Sellers: none, 0. Dead amount: 0. Others: 0. Grand total 21,564, which equals the tile.

### 2c. Replace حساب کی جانچ with آج کی نقدی

**Where.** The dashboard tile حساب کی جانچ (screenshot s03).

**Needed.**

- **The new tile:** آج کی نقدی replaces حساب کی جانچ. It shows today's cash received on the spot (موقع پر جمع) from all buyers.
- **Its page:**
  - one row per day: the date, how many buyers paid, and the نقدی total;
  - a 1-month window with previous and next, go to month, and the month total.
- **A click on a day** opens that day's buyers. Each row shows the buyer, his بکری, his bill, and the نقدی he paid. A total row sits at the bottom.
- **The health check stays visible elsewhere** (NFR-OBS-5): in the attention list when something is wrong, and as a line in Settings or on the status bar [A]. It is not lost.

**Example (after Events 1 to 5, 10-10-26).** The tile shows آج کی نقدی 12,800. The month row for 10-10-26 shows 1 buyer, 12,800. A click shows شاہد (چکتا), بکری 2, bill 12,800, نقدی 12,800.

---

## 3. Every dashboard tile has its own page that does its job (U3)

**Where.** Every tile on the dashboard (screenshot s02).

**Now.** Some tiles open a general page, or another tile's page, so the page does not show what the tile counts. For example, پیشگی باقی needs a page of open advances.

**Needed.**

- **Every tile opens its own page.**
- **Each page shows exactly what its tile counts.** Its total equals the tile.
- **Each row opens its history.**
- Filters start at "سب" (item 1).

The team checks each tile and fills this table in Section 8 before coding:

| Tile | Its page must show | Total equals |
|---|---|---|
| آج کی بکری | Today's بکری (and other days by window), each opening its بکری | Tile |
| آج کا کمیشن | The commission page (FR-9.3) | Tile |
| موجود روکڑ | روکڑ بک, opening row, every cash in and out, closing (plan 78 E) | Tile |
| گاہکوں کے ذمے | Buyer dues page (FR-9.4) | Tile |
| بیوپاریوں کے ذمے | Seller page, side "seller owes the shop" (2a) | Tile |
| بیوپاریوں کو ادائیگی باقی | Seller page, side "shop owes the seller" (2a) | Tile |
| کل اگراہی | Total dues page (2b) | Tile |
| ڈوبی رقم | Dead amount page (R38) | Tile |
| پیشگی باقی | Every seller with an open advance: given, adjusted, returned, open; a click opens his advances | Tile |
| ادھار رقم باقی | Every person with open money lent; a click opens his loans | Tile |
| باغوں کے مالکوں / دوسروں کے ذمے | Its page (FR-16.13) | Tile |
| آج کا دکان خرچہ | The kharch page (FR-9.5) | Tile |
| باقی مال | Unsold stock, lot by lot | Tile |
| مارکیٹ سفر | Open trips with goods left and cost left | Tile |
| باغات | Open gardens with their running balance | Tile |
| آج کی نقدی | Day by day نقدی (2c) | Tile |

**Example.** پیشگی باقی shows 0 after Events 1 to 5, and its page says so. On the opening position (before Event 1) it shows رحیم: given 15,000, adjusted 0, open 15,000, total 15,000, which equals the tile.

---

## 4. The بکری screen: only critical warnings (U4)

**Where.** The بکری page, the sale lines.

**Now.** Each sale line shows a warning, for example that stock goes to unsold stock (باقی مال). A بکری has 50 to hundreds of lines, so the screen fills with warning lines.

**Needed.**

- **No warning line under every sale line.**
- **Only critical warnings show,** and each one disappears the moment it is fixed. Critical means:
  - a qty more than the lot has left (FR-12.6);
  - paid on the spot more than the line's total (FR-12.44);
  - something required is missing when the user saves;
  - cash going below zero, as the confirm box at save (FR-6.7).
- **Information is not a warning.** Goods left in a lot show once, in the lot's own row or summary as "باقی: 5 نگ", not on every line.
- **Over the credit limit** shows as a small mark on the buyer cell, not as a warning line [A].
- **The rate check warning** (FR-7.55) is off by default [A].

**Example.** A lot of 200 نگ, with 3 lines typed so far for 120. No line shows a warning. The lot row shows "باقی: 80". On line 4 the munshi types 90. The error shows under that qty: "صرف 80 نگ باقی ہیں". He changes it to 80, and the error goes at once.

---

## 5. Remove the rate hint (U5)

**Where.** The rate cell of a sale line.

**Now.** The rate cell shows the last rate as a hint (FR-7.53). Each buyer gets his own rate, so the hint looks like a real rate and confuses a non-technical user.

**Needed.**

- The rate cell is empty, with no hint and no placeholder number.
- `Ctrl+D` still copies the rate of the line above (FR-7.54), because the munshi asks for that himself.
- PRD FR-7.53 is retired. E2E-26 drops its hint part, and its other numbers stay the same.

---

## 6. Opening balance in every statement and history (U6)

**Where.** Every statement and history: party khata, R13, R14, the buyer and seller history from the dashboard pages, the partner statement, روکڑ, سرمایہ, ادھار رقم and الگ کھاتہ.

**Needed.** This was asked in plan 78 (B1). This plan only checks it is done everywhere, including the new pages of items 2 and 3.

- The opening balance typed when an account is made shows as the first line, ابتدائی بقایا, with its date and side.
- One test per statement or history proves it is there, and that the closing equals the party card or the tile.

**Example.** احسان (opening 20,000 جمع on 06-11-2026). Every place that shows his history starts with "06-11-2026 ابتدائی بقایا 20,000 جمع".

---

## 7. Buyer slip: better branding, no repeated total (U7)

**Where.** The buyer slip (گاہک کی پرچی), several to an A4 page. Screenshot s04.

**Now.**

- The slip's header is plain: the firm name and the slip title only.
- At the bottom, under the double line, "کل نگ" repeats a total that is already on the slip.

**Needed.**

- **Better branding,** with each slip keeping the same size:
  - the fruit picture on one side;
  - the firm name;
  - the owners' and munshis' names with their phone numbers, small, on the other side.
  - All of it stays in the slip's top band.
- **Remove the repeated total:** the double line at the bottom and the "کل نگ" line under it. Each figure shows once on the slip.
- **Keep everything else:** the "وصولی ____" line stays.

**Example.** انس's slip: header with the fruit picture, "مہر عدنان اینڈ مہر عثمان برادرز" and the phone numbers. Then his lines, مال کی رقم 104,000, کمیشن 6,240, لاگا 800, آج کا بل 111,040, نقد جمع کروایا 100,000, آج کا ادھار 11,040, پچھلا بقایا 63,100, نیا بقایا 74,140, and then وصولی ____. There is no "کل نگ 100" line at the end.

---

## 7a. Open questions for Umar

| # | Item | Question | Default if no answer |
|---|---|---|---|
| Q1 | 2b | Should کل اگراہی (the tile and its page) also include money lent to staff, garden owners and others? | Yes, as a fourth section, and the tile follows |
| Q2 | 2c | Where should the health check show once its tile is gone? | Attention list when something is wrong, and a line in Settings |

## 8. Design notes (filled by the team before code)

- Pages and reports that had no "سب" filter, now fixed (frontend-2, 18-11-2026): the shared picker has a سب first row and shows سب as its value. Applied to the report pickers (party, seller, zamindar, item, city, adda, collector, lent person), analytics, the commission page, پچھلی بکری, باقی مال, rules, the voucher list, the fruit list, the collection page, the find dialog and the partner box of the others' dues page. Not on R13, R14 and R31, which need one party. The new pages of items 2 and 3 open on سب.
- Dashboard tile table (item 3) with status:
- Seller page and history (2a):
- Total dues page (2b) and the PRD 9.1 change:
- آج کی نقدی tile and page (2c):
- Warnings kept and removed on the بکری screen (item 4):
- Statements checked for the opening line (item 6):
- Contracts (command names and types), from `backend-2`, 18-11-2026. All read-only, sign-in checked, money as Paisa numbers:
  1. `dashboard_seller_dues(SellerDuesInput{side: all|shop_owes|seller_owes, party_id: null = سب, search?, sort?, offset, limit<=100}) -> SellerDuesPage{rows[{party, city, balance (with side), payable_paisa, advance_open_paisa, owes_paisa, last_change_date}], total_count, shop_owes_paisa (= tile بیوپاریوں کو ادائیگی باقی), seller_owes_paisa (= tile بیوپاریوں کے ذمے), advances_open_paisa (= tile پیشگی باقی), total}`. Side totals are over the party and search match, not cut by side. `ERR_OUT_OF_RANGE` on sort. Print: `dashboard_seller_dues_print_data`. Seller history: existing `party_ledgers_statement_rows(account_id)` (opening first, one row per بکری).
  2. `dashboard_total_dues() -> TotalDues{parts: buyers, sellers, dead, others (always 4, in that order; count, total_paisa), grand_total_paisa (= tile کل اگراہی)}`. Section rows: `dashboard_buyer_dues`; `dashboard_seller_dues{side: seller_owes}`; `reports_dead_amount{filter: owed}`; `dashboard_lent_open`.
  3. `dashboard_lent_open(LentOpenInput{search?, sort?, offset, limit}) -> LentOpenPage{rows[{party, person_type, open_paisa, oldest_open_date, overdue_paisa}], total_count, total_paisa (= tile ادھار رقم باقی), overdue_paisa}`. A person's loans: existing `lent_list(person_id)`.
  4. `dashboard_cash_sales_month({month: YYYY-MM|null}) -> CashSalesMonth{month, rows[{date, buyer_count, cash_paisa}], buyer_days_total, total_paisa}`; errors `ERR_FORMAT_INVALID`, `ERR_DATE_IN_FUTURE` on month. `dashboard_cash_sales_day({date, offset, limit}) -> CashSalesDay{date, rows[{party, is_chukta, bikri_id, bikri_number, bill_paisa, cash_paisa}], total_count, bill_total_paisa, cash_total_paisa}`.
  5. `DashboardTiles` gains `cash_sales_today_paisa` and `cash_sales_today_buyers`. The حساب کی جانچ tile was frontend only (`health_last`); `health_last` and `health_run` stay for the Settings line. New attention kind `health_check` when the last stored result is not ok.
  6. `total_dues_paisa` = buyers + sellers + dead + open money lent (dead left out). The assistant's tiles check gets a fourth part against the R32 total. No D6 prompt file changes (prompt/ names total_dues only as a tile key).
  7. Lead decision: `dashboard_buyer_dues` also takes `party_id` (null = سب), like the seller page (item 1). `advances_report` takes an optional `only_open` flag for the پیشگی باقی page.
- Dashboard tile table (item 3), page command per tile: آج کی بکری `bikri_list`; آج کا کمیشن `dashboard_commission_month`; موجود روکڑ `cashbook_get`; گاہکوں کے ذمے `dashboard_buyer_dues`; بیوپاریوں کے ذمے and بیوپاریوں کو ادائیگی باقی `dashboard_seller_dues` on their side; کل اگراہی `dashboard_total_dues` and its four section reads; ڈوبی رقم `reports_dead_amount`; پیشگی باقی `advances_report(only_open)`; ادھار رقم باقی `dashboard_lent_open`; باغوں کے مالکوں / دوسروں کے ذمے `lent_dues`; آج کا خرچہ view M (kharch page); باقی مال `bikri_stock_list`; مارکیٹ سفر `trips_summary`; باغات `gardens_summary`; آج کی نقدی the two cash sales commands. Each has a T2 test: page total = tile.

## 9. Screenshot index

All in `docs/plans/assets/79/`.

| File | Shows | Item |
|---|---|---|
| s01-seller-payables-no-all-filter.png | Seller payables page: the party filter has no "سب" | 1, 2a |
| s02-dashboard-tiles.png | Dashboard tiles | 2, 3 |
| s03-health-check-tile.png | The حساب کی جانچ tile to replace | 2c |
| s04-buyer-slips.png | Buyer slips: plain header, repeated "کل نگ" at the bottom | 7 |

## 10. Done when

Every item above works as described, proven by passing tests. Every check is green (`pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo test --workspace`), golden numbers are unchanged, and nothing that works today breaks.

Proof:

- **Dashboard:** for every tile, a test shows that its page total equals the tile.
- **Buyer slip:** a PDF in both languages.
- **New pages:** a speed run on 10 years of data within NFR-SPD-24.
