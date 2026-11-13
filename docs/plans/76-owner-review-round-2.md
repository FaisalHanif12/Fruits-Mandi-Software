# Plan 76: Owner review, round 2 (11 points)

| | |
|---|---|
| Asked by | Umar, 14-11-2026 (points 1 to 10 at 02:31, point 11 at 02:39), with his answers of the same day |
| Branch | `feature/d8-demo-feedback` (Mac phase) |
| Status | Draft for Umar's approval. Nothing is built until he approves. |
| Requirements touched | PRD 9.1, 12.9, FR-12.17, FR-12.39, 13, FR-16.4, FR-16.5, FR-16.6, FR-17.9, FR-17.10, 18.2, FR-18.3, FR-19.11, FR-19.12, FR-19.13, R1, R5, R20, R31, R35; NFR-BAK-10, NFR-SPD-24; Code Standards B31, F34, M9 |
| Patterns to follow | Plan 72 (پچھلی بکری: window, arrows, go to date, days closed by default), plan 74 (party statement: window, day opened in place, statement print) |
| Money agent | Required for points 1, 5, 7, 10 and 11, before the code and again in review |

## 0. How to use this plan

Each point says what is wrong or missing, what the Owner should see when it is done, and an example. It does not say how to build it. The team finds the cause, designs the change and records the design here (Section 14) before writing code.

Rules for the whole plan:

- Requirement documents change first (PRD v1.41 and the others it touches), with version lines and change log lines, committed before the code.
- No golden test number changes. If a point seems to need one, stop and ask Umar.
- Any new or changed journal posting (points 1, 5 and 10) is checked by the money agent. If it differs from what this plan says, stop and ask Umar before building it.
- Every figure comes from the backend. Screens and prints add nothing up.
- Both languages, every window size, keyboard only, the four states (loading, empty, error, data).
- Partner figures stay behind the partner code (FR-17.11).

---

## 1. Point 1: add new lots in a بکری that holds carried stock

**Issue.** Unsold stock (باقی مال) comes into the next day's بکری correctly. But in that بکری the munshi cannot add a new lot, for a new seller or an existing one. In a real day, carried stock and a new truck are sold side by side.

**When done.**

- The بکری that holds carried lots also takes new lots, exactly like a normal بکری: add lot, choose seller, item, packing, qty, lines.
- One save, one print, one undo.
- A carried lot still follows FR-12.17: its lot charges (کرایہ, مزدوری, منشیانہ, فون, کرایہ سٹور, a per-lot commission) are not charged again, and its old بکری and day are not touched.
- A new lot is a first posting with its own charges and its own lot name (FR-12.45).
- Stock, partner split, analytics, matching and the پچھلی بکری list all show both kinds correctly.

**Example.** On 11-10-26, رحیم's lot has 5 کیلا left from بکری 1. The munshi sells them to ارشد at 560 (as E2E-16b: value 2,800, seller commission 168, no lot charges). In the same screen he adds a new lot for کریم, 10 سیب, freight 1,000, and sells it. After one save: the carried lot reads exactly as E2E-16b, and the new lot reads like any first lot, with its freight posted.

**For the team to work out.** Why the add-lot option is missing on this path, and how one save holds a later sale of an old lot and the first posting of new lots (one بکری number or two, how they print). Record the design in Section 14. The money agent checks the journal and split of the mixed save.

---

## 2. Point 2: ادھاری تفصیل (R5) becomes one compact list

**What ادھاری تفصیل means.** The total amount the shop still has to take from its buyers, buyer by buyer.

**Issue.** R5 prints one page per buyer. That wastes paper and is not what the shop uses. The shop needs one list of who owes what (the اگراہی sheet).

**Umar's answer.** List every buyer who owes, plus every buyer who bought today (even if he paid in full). The grand total is the whole اگراہی. There is no وصولی column: وصولی is collected later in the day, after the sale, so it shows up in the next day's previous due (Umar, 14-11-2026).

**When done.**

- One compact list, many buyers on a page, sorted by name.
- Each row: code, buyer (with tag), what he bought today in short (for example "(کیلا 20 . 600)"), previous due, today's bill, paid on the spot, total due.
- Previous due is his balance at the start of the day. It already has every وصولی up to the day before taken off.
- Total due = previous due + today's bill − paid on the spot.
- Today's وصولی is not on today's sheet. It lowers the next day's previous due by itself, because every figure is read from the books.
- Totals row: previous due of all, today's bills of all, paid on the spot by all, and the total due (the whole اگراہی after today's sales).
- The total due equals the اگراہی at the start of the day + today's bills − paid on the spot. On a day with no وصولی entered, it also equals the R20 total and the tile گاہکوں کے ذمے.
- The list grows day by day: a buyer who still owes appears every day until he is clear.
- People in the dead amount follow the R35 table rule (shown only on a day they have an entry, out of the main total).
- The one-slip-per-buyer print moves to the buyer parchi of point 3.

**Example (after Events 1 to 5, 10-10-26).** اکرم's 10,000 وصولی was collected later that day, so it is not on this sheet.

| Buyer | Previous due | Today's bill | Paid on the spot | Total due |
|---|---|---|---|---|
| اکرم ٹول ٹیکس | 0 | 12,880 | 0 | 12,880 |
| ارشد ملتانی | 0 | 18,684 | 0 | 18,684 |
| شاہد (چکتا) | 0 | 12,800 | 12,800 | 0 |
| **Total** | **0** | **44,364** | **12,800** | **31,564** |

How each total due is worked out: اکرم 0 + 12,880 − 0 = 12,880. شاہد 0 + 12,800 − 12,800 = 0. Total 0 + 44,364 − 12,800 = 31,564.

**The next day (11-10-26), with no sales.** The 10,000 is now inside اکرم's previous due:

| Buyer | Previous due | Today's bill | Paid on the spot | Total due |
|---|---|---|---|---|
| اکرم ٹول ٹیکس | 2,880 | 0 | 0 | 2,880 |
| ارشد ملتانی | 18,684 | 0 | 0 | 18,684 |
| **Total** | **21,564** | **0** | **0** | **21,564** |

21,564 = R20 = the tile. شاہد is gone: he owes nothing and bought nothing today.

---

## 3. Point 3: sale prints that each do one job, compact

**Umar's answer.** Buyer slips print several to one A4 page, with cut lines.

**3a. Buyer parchi.**

- Several slips on one A4 page, with dashed cut lines between them. A slip never splits across a cut.
- Each slip stays short: header line, the buyer, his lines, today's total, previous balance, new balance.
- An empty line at the bottom: "وصولی ____" for the collector to write in.

**3b. Overall sale print, seller section.** For each seller: items sold (lot names, sorted as FR-12.12), deductions on their own lines (کرایہ, منشیانہ, مزدوری, نقد / advance), net (صافی بکری), previous balance, new total, each balance with its side word.

**3c. Overall sale print, buyer section.** For each buyer: his purchases, paid today (on the spot), previous outstanding, deposited today (وصولی), total now.

**For every print.** It serves one purpose, holds no empty rows, keeps blocks together across pages, uses digits 0 to 9, centres number columns, and takes every figure from the backend (R35 and R13 functions, no new money maths).

**Example (رحیم, 10-10-26).** Items: کیلا 20 @ 600 = 12,000; 30 @ 580 = 17,400. Deductions: کرایہ 3,000, کمیشن 1,764, مزدوری 250, منشیانہ 50, فون 20, نقد 5,000. Net 19,316. Previous 15,000 بنام (advance). New total after the day: 0.

---

## 4. Point 4: interactive graphs in analytics

**Issue.** Analytics shows tables. Umar wants modern, interactive graphs of the same data beside them.

**When done.**

- Each view (A to K) shows a graph of its current filter next to or above its table (FR-18.3).
- Hover shows the exact figure. A click on a bar or point opens the lines behind it (FR-18.2).
- The graph follows every filter, the compare choice and day / month / year grouping.
- The graph plots the backend's figures over the whole filter, never a sum of the rows on one page.
- Works in both directions, both themes, every window size; prints with the view.

**Example.** View A for October 2026: bars کیلا 3,928 and سیب 1,520; a click on کیلا opens its sale lines.

---

## 5. Point 5: link a الگ کھاتہ to capital

**Issue.** A الگ کھاتہ today belongs to nobody. Umar wants it tied to the shop or to one partner.

**Umar's answer.** Money paid out to a linked khata works like money the partner took. It lowers his کل رقم like a drawing, and money back raises it. It is never income or expense, so profit and commission do not change.

**When done.**

- Making a الگ کھاتہ asks: link to the shop, to one partner, or none.
- Shop: each payment out and each return is split between the active partners by their shares on that date (split_by_bps, M9). Each part moves that partner's کل رقم like a drawing.
- Partner: the whole amount moves that one partner's کل رقم.
- None: works as today.
- Every movement shows as a line in each affected partner's statement (point 11), with the khata name and reason.
- The khata's own page still shows its running balance (FR-16.6).
- FR-17.9 still holds: Σ کل رقم of all partners + ابتدائی سرمایہ = the firm's equity in the trial balance.
- Profit, commission and the partner split of income do not change.

**Example.** خاندانی کھاتہ linked to the shop, shares 40 / 30 / 30. Pays out 10,000: الف −4,000, ب −3,000, ج −3,000. Gets 4,000 back: الف +1,600, ب +1,200, ج +1,200. Linked to الف only: −10,000, then +4,000, all on الف.

**For the money agent.** The posting that does this and keeps the FR-17.9 check true. If it changes any existing posting or golden number, stop and ask Umar.

---

## 6. Point 6: fruit picture in every print header

**When done.**

- A fruit picture at the top left of every print and PDF header. The owners and munshis stay on the right.
- The picture is bundled in the app (no network, CSP), with a clear licence (our own drawing, or CC0), credited if needed.
- It prints crisp in colour and in black and white, and does not push content to a second page.
- Same position in Urdu and English prints [A].

---

## 7. Point 7: the commission card opens its own page

**When done.**

- The dashboard tile آج کا کمیشن opens a new page.
- One row per day: date, seller commission, buyer commission, لاگا, total commission, shared income, and each partner's share.
- One month at a time. Previous and next month buttons, go to month, Page Up and Page Down. It opens on the current month and never goes past it.
- A month totals row from the backend.
- A click on a day opens that day's detail (the بکری that made it).
- Partner columns stay behind the partner code; without it they show the locked box and the rest works.
- Print and Excel of the month.
- The same month-window idea is used wherever years of data are shown (see point 8).

**Example (10-10-26).** Seller 2,484; buyer 2,484; لاگا 480; total 5,448; shared income 5,488 (5,448 + فون 40); الف 2,196, ب 1,646, ج 1,646 (sum 5,488).

Note: partner shares are shares of the shared income (commission, لاگا, فون, ریٹ فرق), so they add up to the shared income column, not to total commission.

---

## 8. Point 8: windows and paging everywhere

**Issue.** Some screens load years of data at once.

**When done.**

- Every list, ledger, register, report, analytics table and dashboard drill-down loads page by page (FR-19.11, B31) and has its filters.
- A screen with many rows a day (sales, journal, cash book) gets a 7-day window like پچھلی بکری. A screen with few rows a day (expenses, capital, fruit, money lent, الگ کھاتہ, partner money) gets a 1-month window.
- Arrows, Page Up and Page Down, go to date, totals from the backend, print and Excel hold every row of the window or filter.
- Speed as NFR-SPD-24 on 10 years of data.

**First task for the team.** An audit table, added to Section 14: each screen, what it loads now, and what it gets (7-day, 1-month or already fine). The lead checks the table, then the work starts.

---

## 9. Point 9: a simple buyer dues page from the dashboard

**When done.**

- The tile گاہکوں کے ذمے opens a simple page: code, buyer, city, total balance, the date his balance last changed, and a grand total.
- No aging buckets (0-7, 8-30, 31-90, 91+) on this page.
- Sorted by name; search and paging as point 8.
- The grand total equals the tile and R20.
- R20 in the reports menu keeps its aging columns, because the assistant reads them (D6 prompt, E2E-63) [A]. So no prompt file changes.

**Example (after Events 1 to 5).** اکرم ٹول ٹیکس 2,880, last change 10-10-26; ارشد ملتانی 18,684, last change 10-10-26; total 21,564.

---

## 10. Point 10: dues of garden owners and others, replacing the backup tile

**Umar's answer.** Reuse ادھار رقم. Add a type for garden owners and others, plus the shop-or-partner link. One place, no duplicate books.

**When done.**

- The dashboard backup tile is replaced by "باغوں کے مالکوں / دوسروں کے ذمے" (Garden owners' and others' dues). It shows the total still open and how many people.
- Backup status stays in the status bar and the attention list (NFR-BAK-10), so nothing about backup is lost.
- ادھار رقم gets a person type: staff, garden owner, market-trip person, other.
- Each entry asks: shop or a specific partner.
  - Partner: like point 5, money given lowers that partner's کل رقم and money back raises it.
  - Shop: the money is still owed to the shop, so partner totals do not change; each partner's statement shows his % part as a note line [A].
- The tile opens a page: each amount given with date, person, type, reason and link; each return that adds back; open amount per person; filters by type and link; a 1-month window.
- R32 and the attention list keep working, now with the type.

**Example.** On 01-11-26 the shop gives نواز (garden owner) 20,000, reason "اگلے سیزن کا باغ", linked to الف. الف's کل رقم −20,000. On 10-11-26 نواز returns 5,000: open 15,000, الف +5,000. The tile shows 15,000, 1 person.

**For the money agent.** Same check as point 5: FR-17.9 holds, existing postings and golden numbers unchanged, or stop and ask Umar.

---

## 11. Point 11: partner statement, one month at a time

**Issue.**

- The partner page shows each partner's open gardens and trips as large cards. With one trip it already fills the screen; with years of data it will be unusable.
- There is no clear statement that shows, day by day, what came in and went out of each partner's capital, and where it came from.

**When done: the statement page.**

- Each partner row on the partner page has a "کھاتہ / Statement" button (the فرد حساب button can become it).
- The statement shows one month at a time, opening on the current month. Previous and next month, go to month, Page Up and Page Down, as plans 72 and 74. It goes back to the first month of data.
- The top row is the opening کل رقم of the month (carried from before). The first month starts with his opening capital.
- One row per day that has a movement. Columns:
  - commission (his share of the day's income),
  - expense share,
  - gardens and trips (his credits and his parts paid from the shop),
  - fruit taken,
  - money taken and returned (drawings, linked الگ کھاتہ, linked ادھار رقم),
  - capital in and out,
  - day net,
  - کل رقم at the end of the day.
- Month totals row. The closing of the current month equals his کل رقم on the partner page and R31.
- **A day opens a detail card** (click or Enter, in place, as plan 74). It lists every movement of that day with its source, reason and amount, and a link to the entry:
  - commission: each بکری with the shared income and his %;
  - trip: purchase paid from the shop, sales money credited to him, and the trip's sale, cost and result as a note;
  - garden: installments, closing with cost, sales, profit and his %;
  - expense, fruit, drawings, khata, money lent, capital: each with its reason.
  - The card ends with "after today, کل رقم = …".
- A month statement print (the plan 74 style): opening, one block per day, totals, closing, amount in words.
- Everything that reaches a partner appears here: commission, expenses, trips, gardens, fruit, drawings, capital, and the new links of points 5 and 10. Nothing is typed twice; every line comes from posted records (FR-17.10).
- Behind the partner code.

**When done: the partner page, uncluttered.**

- The open gardens and trips no longer show as big cards under each partner. They become one compact line per partner (for example "کھلے: 1 سفر، 0 باغ") that opens the list on demand, or a separate tab.
- The page stays short as years of data and many trips grow.

**Example (ب, October 2026, after Events 1 to 5 and E2E-35).**

| Date | Commission | Expense | Capital | Day net | کل رقم |
|---|---|---|---|---|---|
| Opening | | | | | 0 |
| 10-10-26 | 1,646 | −900 | | 746 | 746 |
| 11-10-26 | | | 50,000 | 50,000 | 50,746 |

Opening the 10-10-26 card: بکری 1 his 30% of 3,948 = 1,184; بکری 2 his 30% of 1,540 = 462; دکان خرچہ بجلی −900; after today 746.

**Example with a trip (الف, E2E-34, 10-10-26).** Card: trip purchase خ-1 from the shop −4,160; commission of the بکری 480; trip sales money, his 40% of 8,460 = 3,384 (note: sale 8,460, cost 7,800, result 660, his 264); after today −296.

---

## 12. Answers and defaults

| # | Point | Decision |
|---|---|---|
| A1 | 2 | All who owe plus today's buyers; columns previous due, today's bill, paid on the spot, total due; no وصولی column, today's وصولی shows in the next day's previous due (Umar) |
| A2 | 3 | Several buyer slips on one A4, with cut lines (Umar) |
| A3 | 5 | A linked khata moves a partner's کل رقم like a drawing; not income or expense (Umar) |
| A4 | 10 | Reuse ادھار رقم with a type and the link (Umar) |
| A5 | 3 | [A] "Overall sale print" is the print of one بکری (R1 seller bill grown into a seller section and a buyer section), with a whole-day choice like R5 |
| A6 | 9 | [A] The dashboard opens a new simple page; R20 keeps its aging for the assistant |
| A7 | 10 | [A] A shop-linked loan does not change partner totals until it is returned or written off; the statement shows a note line |
| A8 | 6 | [A] Picture left, owners right, in both print languages |
| A9 | 11 | [A] One row per day, current month first, day detail opens in place |
| A10 | 2 | The R5 data the assistant reads stays as it is (columns serial, buyer, detail, value, buyer_total, paid, left; today's buyers). The new اگراہی sheet is how R5 shows on screen and in print, so the D6 prompt files do not change (Umar, 14-11-2026) |
| A11 | 1 | Consigned leftover stock is not added to a new بکری by itself (only trip and garden goods are, FR-12.39). The normal بکری screen gets a "+ باقی مال سے" picker that adds a carried lot (its lot charges locked, FR-12.17) beside new lots; one save, one undo. The old later-sale route opens this screen (Umar, 14-11-2026) |
| A12 | 3a | Buyer parchi: as many slips as fit on an A4 (about 6, in 2 columns); a slip never splits (Umar, 14-11-2026) |
| A13 | 5, 10 | If the money check finds a linked khata or loan cannot be built without changing an existing posting or golden number, that point is parked for Umar and the rest is built (Umar, 14-11-2026) |
| A14 | 6 | [A] Lead: the print header uses physical left (picture) and right (owners and munshis) as Umar asked; an approved exception to logical properties, in the header only |
| A15 | 4 | [A] Lead: the chart is always shown beside or above the table at 1366 × 768 and larger; at 1024 × 600 a toggle |
| A16 | 9 | [A] Lead: "date his balance last changed" = the date of the last posted line on his account |

## 13. Tasks

| # | Task | Owner |
|---|---|---|
| T0 | PRD v1.41 and the other documents: R5, R1, R20 page, tiles 9.1, FR-12.17 mixed save, FR-16.4 types and link, FR-16.6 link, FR-17.9 and FR-17.10 new lines, R31 month view, commission page, paging rule, print header picture. Version and change log lines. Committed first. | lead |
| T1 | Money design for points 1, 5, 10 (postings) and 7, 11 (read paths), recorded in Section 14 with worked numbers; money agent PASS | backend + money agent |
| T2 | Point 8 audit table | frontend + backend |
| T3 | Backend: reads, saves, print data, tests, `pnpm gen:types` | backend |
| T4 | Screens and prints | frontend |
| T5 | Review and robustness at the end (10 years of data for points 7, 8, 9, 11) | reviewer, robustness |

## 14. Design notes (filled by the team before code)

- Point 1 design:
- Point 5 posting:
- Point 10 posting:
- Point 7 read:
- Point 11 read:
- Point 8 audit table:
- Contracts (command names and types):

## 15. Done when

Every point above works as described, proven by passing tests, with every check green (`pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo test --workspace`, golden tests unchanged), and nothing that works today broken. Proof for each print is a PDF in both languages. Proof for each paged screen is a speed run on 10 years of data within NFR-SPD-24.
