# Plan 78: Owner review, round 3 (20 points)

| | |
|---|---|
| Asked by | Umar, 17-11-2026 (points U1 to U6 at 01:16, points U7 to U20 at 01:53), after testing plans 76 and 77 |
| Branch | `feature/d8-demo-feedback` (Mac phase) |
| Status | Approved by Umar, 17-11-2026, with the answers in Section 7 and the defaults marked [A] |
| Top priority | **Part A, printing.** The way the prints work today does not match what the shop needs. |
| Screenshots | `docs/plans/assets/78/` (index in Section 9) |
| Overlap | Where this plan and plans 76 or 77 say different things, this plan wins. |

## 0. How to use this plan

Each item says where it is, what happens now, what is needed, and an example. It does not say how to build it. The team finds the cause, designs the change, and writes the design into Section 8 before writing code.

Rules for the whole plan:

- Requirement documents change first (PRD and the others it touches), with version and change log lines.
- No golden test number changes. If an item seems to need one, stop and ask Umar.
- Any new or changed posting (U14 relief, U18 advance tick) goes to the money agent first. If it differs from this plan, stop and ask Umar.
- Every figure comes from the backend. Screens and prints add nothing up.
- Both languages, every window size, keyboard only, and the four states (loading, empty, error, data).
- Umar's point numbers are kept as U1 to U20 so each one can be traced.

---

## Part A. Printing (top priority)

### A0. One rule for every print (U20)

Every print has one purpose, and the title says it. Before building or changing a print, the team writes its purpose in one line.

| Print | Purpose | Who it is for |
|---|---|---|
| Buyer slip (گاہک کی پرچی) | What one buyer bought today and what he owes | The buyer |
| Seller bill (بیوپاری کی پرچی) | What one seller sold today, his deductions and what the shop owes him | The seller |
| Whole sale print (پوری بکری) | Everything sold today: every seller and every buyer, in one place | The shop |
| ادھاری تفصیل | How much the shop has to take from its buyers: today and in total | The shop, the collector |
| Statement (اسٹیٹمنٹ) | One party's account over a period: opening, every entry, closing | The party |

For every print:

- **Same branding on every page:** firm name, mandi, owners and munshis, and the fruit picture.
- **Party name in the title** when the print is for one party (A6).
- **Compact:** no page per party unless it is a slip, no empty rows, no wasted space.
- **Clear Urdu words** that a buyer or seller understands at once (A6).
- Digits 0 to 9, number columns centred.

### A1. The print menu after saving a بکری (U11)

**Where.** The بکری page. The Print button (F9) and the ادھاری تفصیل button (Shift+F9). Screenshot s13.

**Now.** The ادھاری تفصیل button opens a box with two choices, گاہک کی پرچی and بکری کی مکمل پرنٹ. That is wrong: those are sale prints, not ادھاری تفصیل.

**Needed.**

- **Print button (F9)** opens a menu with three choices:
  1. **گاہکوں کی پرچیاں** (buyer slips): the same data as today's buyer slip, several slips to an A4 page with cut lines (plan 76).
  2. **بیوپاریوں کی پرچیاں** (seller bills): the same data as today's seller bill.
  3. **پوری بکری** (whole sale print): A4.
- **ادھاری تفصیل button (Shift+F9)** prints only ادھاری تفصیل (A5). Nothing else is in that box except language, paper and print or PDF.
- Each choice keeps "this بکری only" or "the whole day", as today.

### A2. Buyer slip (U5, U6)

**Needed.**

- Data as today, several slips to an A4 page, with cut lines.
- The title names the buyer: "انس کی روزانہ پرچی" (Anas's day slip).
- The money lines use the clear words of A6.
- An empty "وصولی ____" line at the bottom for the collector.

### A3. Seller bill (U10, U6)

**Where.** Screenshot s12 (title "بیوپاری کا بل") and s8 (seller day slip).

**Needed.**

- The data is the same as today.
- The title names the seller: "عدنان کا بل" (Adnan's bill), not "بیوپاری کا بل".
- The money lines use the clear words of A6.

### A4. Whole sale print, پوری بکری (U11, U16)

**Where.** Screenshot s17 shows بکری تفصیل (R6). It has the right style, but only the sellers.

**Now.** The whole sale print gives one buyer or seller a full page each.

**Needed.** One compact print of today's sale, like s17, with both sides:

- **Sellers table** (as in s17): one row per lot with date, بکری, seller, item (lot name), qty, gross, freight, commission, labour, munshiana, نقد, store rent, phone, total deductions, net. A totals row.
- **Buyers table** (new): one row per buyer with buyer, what he bought in short (for example "(سیب 30 . 1,500)"), bill, نقد (paid on the spot), ادھار (left on account). A totals row.
- Both tables follow one after the other. There is never a page per party.
- Purpose: the shop sees everything sold today on one or two pages.

**Example (from s17, 16-11-2026, بکری 2).** The sellers table is as in s17:

| Seller | Lot | Gross | Net |
|---|---|---|---|
| عدنان | سیب-1 | 79,000 | 69,260 |
| فیصل F | کیلا-1 | 71,000 | 62,740 |
| احسان | مالٹا-1 | 50,000 | 40,500 |
| **Total** | | **200,000** | **172,500** |

The buyers table below it lists every buyer of the same بکری, with his bill, his نقد and his ادھار.

### A5. ادھاری تفصیل (U11)

**What it means.** The total amount the shop has to take from its buyers.

**Where.** The ادھاری تفصیل button on the بکری page.

**Now.** It prints one buyer per page (s13).

**Needed.**

- **Buyers only.** No sellers.
- **All of today's buyers on one page, one row each.** Never one page per buyer.
- **Rows are today's buyers. The کل ادھار total covers everyone who owes,** including buyers who bought nothing today (Umar, 17-11-2026).
- **Columns:**
  - buyer;
  - what he bought today, in short;
  - today's bill;
  - نقد (what he paid on the spot);
  - آج کا ادھار (today's bill − نقد);
  - پچھلا بقایا (what he owed before today);
  - کل ادھار (پچھلا بقایا + آج کا ادھار).
- **Totals at the end:**
  - total نقد received today;
  - total ادھار of today;
  - total کل ادھار: the whole amount owed by all buyers, from all previous days and years, including buyers who bought nothing today.
- وصولی collected later in the day is not on this sheet. It shows in the next day's پچھلا بقایا (plan 76, A1).

**Example (after Events 1 to 5, 10-10-26).**

| Buyer | Bought today | Bill | نقد | آج کا ادھار | پچھلا بقایا | کل ادھار |
|---|---|---|---|---|---|---|
| اکرم ٹول ٹیکس | (کیلا 20 . 600) | 12,880 | 0 | 12,880 | 0 | 12,880 |
| ارشد ملتانی | (کیلا 30 . 580) | 18,684 | 0 | 18,684 | 0 | 18,684 |
| شاہد (چکتا) | (سیب 10 . 1,200) | 12,800 | 12,800 | 0 | 0 | 0 |
| **Total** | | **44,364** | **12,800** | **31,564** | **0** | **31,564** |

How a row is read: اکرم's bill is 12,880 and he paid 0, so today's ادھار is 12,880. He owed 0 before, so his کل ادھار is 0 + 12,880 = 12,880.

Total ادھار of all buyers is 31,564, because nobody else owes.

### A6. Party name in titles and clear words (U5, U6, U10)

**Where.** Screenshot s7 (title "پارٹی کی روزانہ پرچی"), s8 (seller day slip), s12.

**Needed.**

| Now | Needed | Where |
|---|---|---|
| پارٹی کی روزانہ پرچی | `<name>` کی روزانہ پرچی, for example "احسان کی روزانہ پرچی" | Every party day slip |
| بیوپاری کا بل | `<name>` کا بل, for example "عدنان کا بل" | Seller bill |
| سابقہ رقم | پچھلا بقایا | Every slip and bill |
| آج ادا (buyer) | نقد جمع کروایا | Buyer slip |
| آج ادا (seller) | نقد ادا کر دیا (Umar) | Seller bill |
| دیگر اندراج, when it is really the opening balance | ابتدائی بقایا, or added into پچھلا بقایا | Every slip and bill |

**Found in s8.** احسان's slip for 06-11-2026 shows سابقہ رقم 0 and دیگر اندراج 20,000 جمع. The 20,000 is his opening balance from when his account was made. It should read as an opening balance, not as an "other entry".

The team also reads every other print and lists any word a buyer or seller would not understand, with the clearer word, in Section 8.

### A7. Statement print (U8)

**Where.** Screenshot s10, احسان's statement.

**Now.** One بکری takes half a page. A seller with 300 بکری would need hundreds of pages.

**Needed.**

- **One row per entry,** in a table: date, entry (بکری 1, ادائیگی 1), short detail on one line, بنام, جمع, بقایا.
- **Opening balance first,** totals and closing last, with the amount in words.
- **Item lines are optional,** off by default. With them on, each item is one short line under its بکری.
- **20 to 30 rows on one A4 page,** with no wasted space.
- **The screen statement follows the same style** (B3).

**Example (احسان, the data of s10, compact).**

| Date | Entry | Detail | بنام | جمع | بقایا |
|---|---|---|---|---|---|
| 06-11-2026 | ابتدائی بقایا | | | 20,000 | 20,000 جمع |
| 06-11-2026 | بکری 1 | آم 200 پیٹی، خام 182,000، کٹوتی 21,720 | | 160,280 | 180,280 جمع |
| 16-11-2026 | ادائیگی 1 | | 100,000 | | 80,280 جمع |
| | **Total** | | **100,000** | **180,280** | **80,280 جمع** |

Three lines instead of a full page.

---

## Part B. Statements and history

### B1. Opening balance everywhere (U7, U20)

**Where.**

- The party khata page (s9).
- The dues pages that open from the dashboard tiles گاہکوں کے ذمے and بیوپاریوں کو ادائیگی باقی (s21).
- The partner statement.
- روکڑ.
- Every other statement or history.

**Now.** An opening balance typed when an account is made does not show in some statements. In s9 احسان's first row (06-11-2026) has سابقہ رقم 0 and جمع 180,280. The 180,280 hides his 20,000 opening, so the amounts look wrong.

**Needed.** Every statement and history shows the opening balance as its own first line: ابتدائی بقایا, with its date and side. Then every entry follows. Nothing from the account's start is missing.

**Example.** احسان: ابتدائی بقایا 20,000 جمع on 06-11-2026. The بکری that day adds 160,280, so 180,280 جمع.

### B2. One line per بکری in party history (U19)

**Where.** Dashboard → گاہکوں کے ذمے or بیوپاریوں کو ادائیگی باقی → a party → his history (the ledger). Screenshots s19 (buyer) and s20 (seller).

**Now.** One بکری shows up many times:

- one line for each item;
- a separate موقع پر جمع line;
- in the screenshots, many "الٹ اندراج" (reversal) lines, which come from edits.

**Needed.**

- **One بکری = one line** in a party's history. Every item of that بکری is listed inside that line. Money paid on the spot is shown in the same line (a نقد column), not as a separate line.
- **An edited entry shows once,** in its latest form, with its "تبدیل شدہ" (edited) mark. The old version and its reversal are hidden from the normal view and stay in the edit history.
- **Balances unchanged:** the running balance and the totals stay exactly the same.

**For the team.** Find why one بکری repeats. Is it the item lines, the reversals, or both? Check every history and ledger that has the same problem.

### B3. Compact statement on screen (U8, U20)

The screen statement of a party (پارٹی کھاتے) and its print use the same compact style as A7: one row per entry, the opening first, the items optional. The 30-day window of plan 74 stays.

### B4. Partner statement shows everything (U3)

**Where.** The partner statement. Screenshot s5. Its columns today: کمیشن, خرچہ, باغ/سفر, پھل, لیا اور واپس, سرمایہ, دن کا خالص, کل رقم.

**Umar's ask.** Some inputs are missing, for example ادائیگی and payments to parties or sellers. Everything must show in the statement.

**Needed.** The team lists every kind of entry in the app and says whether it shows in the partner statement. Anything missing is added.

**Umar's answers (17-11-2026), sellers in the partner statement.**

| Entry | Partner total | In the partner statement |
|---|---|---|
| Advance to a seller (پیشگی) | Goes down by each partner's share | Yes, one line |
| The advance comes back (settled by his goods with the tick on, or returned in cash) | Goes back up by the same shares | Yes, one line |
| The seller's sale money | No change | No |
| Paying a seller his sale money (ادائیگی) | No change | Not shown at all |

An advance that is never settled stays off the partner totals. The partners can see exactly how much of their money is out with sellers.

**Example (shares 40 / 30 / 30).**

- 01-09-26: رحیم takes a 15,000 advance. الف −6,000, ب −4,500, ج −4,500. Line: "پیشگی رحیم 15,000، آپ کا حصہ 40%: −6,000".
- 10-10-26: his goods are sold and 15,000 of his sale settles the advance. الف +6,000, ب +4,500, ج +4,500. Line: "پیشگی واپس رحیم 15,000، آپ کا حصہ 40%: +6,000".
- The 4,316 paid to him the same day is not shown.
- At the end of 10-10-26 the totals are the same as today: الف −1,004, ب 746, ج 746.

**For the money agent (stop point).**

- **Books check.** PRD FR-17.9 says all partner totals + opening capital = the shop's capital in the trial balance. Design the advance lines so this still holds.
- **Golden tests.** E2E-37 lists ب's statement lines and running total. New advance lines change that list, and E2E-19 and E2E-28 leave an advance open. If any golden number would change, stop and show Umar the list first.

---

## Part C. The بکری screen

### C1. Three load buttons on the بکری page, each shown only when it has goods (U2)

**Where.** The بکری page, the "باقی مال سے" button (s4). The card it opens (s3). Trip goods lines (s2).

**Now.** The باقی مال button is always there. In the test, its card said "بیچنے کے لیے کوئی مال باقی نہیں" (no goods left to sell).

**Needed (Umar, 17-11-2026).** Each kind of goods has its own button. A button shows only when it has goods to load, and is hidden otherwise.

| Button | Shows when | Its card lists |
|---|---|---|
| باقی مال سے | A lot of an earlier بکری still has goods unsold | Only those unsold lots: lot name, seller, item, qty left |
| مارکیٹ سفر کا مال | A market trip's goods were entered in the مارکیٹ سفر section and are not sold yet | Only that trip goods: trip, purchase, item, qty left |
| باغ کا مال | A garden's goods arrived (مال آیا) in the باغات section and are not sold yet | Only those garden goods: garden, item, qty left |

- **باقی مال means only unsold stock** from earlier بکری. Trip goods and garden goods are not in its card.
- **Loading is by hand.** The munshi presses the button and picks the goods. Each picked row becomes a lot of this بکری, posted like today (a carried lot as FR-12.17, a trip lot as FR-12.26, a garden lot as FR-12.36).
- **Nothing is missing.** Every unsold row of each kind is in its card.

**Note.** Today, trip and garden goods come into a new بکری by themselves as ready lots (PRD FR-12.39, FR-12.40). With these buttons they no longer come in by themselves; the munshi loads them. The PRD changes accordingly [A, Umar may keep the automatic lots as well].

**Example.** On 17-11-26 there are 5 کیلا left from بکری 1, a trip with 80 پیٹی مالٹا, and no garden goods. The بکری page shows "باقی مال سے" and "مارکیٹ سفر کا مال", but no "باغ کا مال". The باقی مال card lists only the 5 کیلا.

### C2. A dropdown to jump to any lot's summary (U9)

**Where.** The بکری page, the summary (خلاصہ) card. Screenshot s11. Today it has a switch "لاٹ 2: فیصل F، کیلا-1 | پوری بکری".

**Now.** With 10 to 20 lots, to see another lot's summary the munshi scrolls up, picks the lot, and scrolls back down.

**Needed.** The blue lot switch in the summary becomes a dropdown that lists every lot of the بکری (lot number, seller, lot name). Picking a lot shows that lot's summary at once. "پوری بکری" stays as one choice in the list. It works with the keyboard.

**Example.** The munshi is on lot 1's summary. He opens the dropdown, picks "لاٹ 7: کریم، سیب-2", and the summary shows lot 7, without scrolling.

### C3. Advance deduction: a tick per seller in the summary, off by default (U18)

**Where.** The بکری page, the summary of each seller.

**Now.** When a seller who took an advance has his goods sold, the advance is taken off his sale by itself. (Today the tick پیشگی سے کاٹیں is on by default, PRD FR-12.23, and sits in the lot's extra fields.)

**Needed.**

- Each seller's summary shows a tick: "پیشگی اس بکری سے کاٹیں" (take the advance off this sale), with his open advance amount.
- **Off by default.** The advance is not taken off unless the munshi ticks it.
- Ticked: the advance is taken off today's sale, as today.

**Example (رحیم, Event 1).** He has a 15,000 advance. His sale credit is 24,316, and he got 5,000 نقد.

- Not ticked: payable 24,316 − 5,000 = 19,316. The advance stays open at 15,000 (as E2E-28).
- Ticked: payable 24,316 − 5,000 − 15,000 = 4,316 (as Event 1).

**Note.** This changes the default of PRD FR-12.23. Golden tests that rely on the default today must set the tick themselves, so their numbers stay the same. The team lists those tests before changing them (stop point, Q3).

---

## Part D. Market trips and gardens

### D1. Default or custom partner shares on each trip (U1)

**Where.** New market trip, the partners part. Screenshot s1. It shows each partner's default % (عدنان 50%, عثمان 50%) and only the money source (دکان سے or اپنی جیب سے).

**Needed.**

- On every new trip: a choice, "default shares" or "custom shares".
- Custom: any % for any partner, up to 2 decimals, a partner can be 0%, the total must be exactly 100%.
- The same choice for a new garden [A], because both use the same rule (PRD FR-12.29).

**Example.** Default 50 / 50. For this trip: عدنان 70%, عثمان 30%.

### D2. A garden with several items (U13)

**Where.** New garden form, the قسم مال field. Screenshot s15. It takes one item.

**Needed.** A garden can have one or several items (for example آم and مالٹا), chosen in the same field. Each goods-arrived line still picks its own item.

### D3. Relief from the garden owner: edit the rent with the sign (U14)

**Where.** A garden's page, its edit (تبدیل کریں).

**Now.** The garden's rent (کل ٹھیکہ) is fixed once entries rest on it. When the garden ends in a loss, the owner sometimes gives the shop relief, and there is no way to record it.

**Needed (Umar, 17-11-2026).**

- **Edit the rent.** In the garden's edit, the Owner can change its rent (کل ٹھیکہ, the total price the garden was bought for), even after its goods are sold.
- **Reason, compulsory.** For example "مالک سے رعایت ملی" (relief received from the owner).
- **Sign, compulsory.** The edit opens only when the sign matches the Owner's sign (دستخط, "adnan" by default, the same sign as PRD FR-21.4). A wrong sign is refused, nothing opens, and the try is in the audit log.
- **The figures follow.** The lower rent means a lower cost, so the loss goes down (or the profit goes up). The rent still due, the running balance, the result and each partner's part all follow.
- **Recorded.** The edit is kept in the audit log with the old and new rent, the reason and the sign. It carries the edit stamp (FR-7.71).
- **Shown everywhere.** The new rent and the relief show on the garden, in R36 and in each partner's statement line for the closing.
- **Before closing.** It is done before the garden is closed. A closed garden is first reopened by undoing its closing (FR-12.38) [A].

**Example.** Rent 50,00,000 plus running costs 5,00,000, so the cost is 55,00,000. Sales money is 30,00,000, so the loss is 25,00,000. The owner gives 5,00,000 relief. The rent becomes 45,00,000, the cost 50,00,000, and the final loss 20,00,000. At 60 / 40: −12,00,000 and −8,00,000.

**For the money agent.**

- **Rent still due:** if part of the rent is still unpaid, the relief takes it off that first.
- **Rent already paid:** if the installments already paid are more than the new rent, the difference is money the owner owes back. Show it on the garden and ask Umar how it comes back before building it.

### D4. Commission from trips and gardens (U14)

**Where.** The commission page (from the dashboard tile) and analytics.

**Needed.** A goods filter on the commission page: all, consigned goods, trip goods, garden goods. Analytics already has a goods filter (FR-18.1). The team checks it works on every view.

**Example.** October, garden goods: the commission earned on garden lots only.

---

## Part E. روکڑ, the cash book (U12)

**Where.** روکڑ. Screenshot s14.

**Needed.**

- **روکڑ is the shop's net cash.** Every rupee in and out shows here: وصولی, ادائیگی to sellers, expenses, نقد on lots, paid on the spot, money lent and returned, partner money, capital and the rest.
- **The starting money shows as rows:** the opening cash and every partner investment (capital), each labelled as what it is, from the very first day.
- **Running balance and closing** after every row.
- **Years of data:** the 7-day window with arrows and go-to-date stays (plan 76). The team checks that going back to the first day shows the opening and capital rows correctly, and that the totals come from the backend.

- **Only cash, never the whole sale.** A sale adds to روکڑ only what was paid in cash: on the spot (موقع پر جمع) and later وصولی. The part left on credit is not cash until it is collected. This is how it works today, and it stays (Umar, 17-11-2026).
- **The capital investment is one row on the day it comes in.** روکڑ never adds the capital total on top of the cash. The capital itself is kept in سرمایہ and the partner statement.
- **Opening capital asks where the money is (new).** When a partner's opening capital is typed on the partner form, the form asks "یہ رقم کہاں ہے؟" (where is this money?), with روکڑ as the default. If it is cash, روکڑ goes up by the same amount on the same date. Today that opening only records the capital, and its other side goes to ابتدائی فرق, so روکڑ can show less cash than the shop has.
- **"Where the capital is" card (new).** On the روکڑ page, a card shows where the partners' money is now. Every figure comes from the backend. Behind the partner code.

  | Where it is | Example |
  |---|---|
  | Cash in hand (روکڑ) | 5,00,000 |
  | Buyers owe the shop | 2,00,000 |
  | Advances with sellers | 3,00,000 |
  | The shop owes sellers | −0 |
  | **Total** | **10,00,000** = partners' capital + profit so far |

**Example (worked day, 10-10-26).**

| Entry | روکڑ after |
|---|---|
| Opening (ابتدائی روکڑ) | 50,000 |
| بکری 1: نقد to رحیم | 45,000 |
| بکری 2: شاہد pays on the spot | 57,800 |
| وصولی اکرم | 67,800 |
| ادائیگی رحیم | 63,484 |
| دکان خرچہ بجلی | 60,484 |
| ذاتی خرچہ الف | 58,484 |

The sales that day were 41,400. Only 22,800 of it came in as cash. On 11-10-26, partner ب invests 50,000 in cash: one row "سرمایہ جمع، ب +50,000", and روکڑ becomes 108,484.

**Check in the screenshots (s5 and s14).** Each partner starts at ابتدائی 5,00,000 in the partner statement, but روکڑ does not start with 10,00,000. The team checks how that opening capital was entered. If it was cash, روکڑ is short by that amount today, and it is fixed by the "where is this money" choice above.

---

## Part F. Navigation and the assistant

### F1. A واپس (back) button on every page (U4)

**Where.** For example the سرمایہ page opened from the partner page (s6). It has no way back.

**Now.** A non-technical user has to go to the side menu and find the page again.

**Needed.**

- Every page opened from another page has a واپس button in the same place (the top of the page, start side). It goes back to the page it came from, in the same state.
- Esc already leaves a screen (FR-7.18). The button is the visible way.
- The team lists every page and adds the button where it is missing.

### F2. Assistant layout breaks after a few questions (U15, U17)

**Where.** The question panel. Screenshots s16 and s18.

**Now.** After 3 or 4 questions, the question box jumps up to the middle, a large empty space is left below it, and the answer area shrinks.

**Needed.**

- The question box always stays at the bottom of the panel.
- The conversation scrolls above it and uses all the space.
- The quick-question chips never push the box up.
- Test with 10 questions in a row, in both sizes and both languages.

---

## Part G. Years of data stay organized (U20)

Every page that grows with time stays short and tidy after 10 years of data:

- party history and statements;
- partner statement;
- روکڑ;
- ledgers;
- the dashboard drill pages;
- trips and gardens lists.

They use windows (7 days or a month), days closed by default, compact rows and paging, as plans 72, 74 and 76 set. The team checks each such page with 10 years of data and lists any that still feels long or messy.

---

## 7. Open questions for Umar

| # | Item | Question | Default if no answer |
|---|---|---|---|
| Q1 | A5 | Answered (Umar, 17-11-2026): rows for today's buyers; the total کل ادھار covers everyone who owes | |
| Q2 | B4 | Answered (Umar, 17-11-2026): an advance lowers partner totals by share and comes back when settled; the seller's sale money and its payment are not in the partner statement | |
| Q3 | C3 | Golden tests that rely on today's default (advance ticked): the tests set the tick themselves, so their numbers stay the same; the agent lists them first | Yes (lead's default; Umar may change it) |
| Q4 | D3 | Answered (Umar, 17-11-2026): the garden's rent is edited in its edit section, with a compulsory reason and the Owner's sign | |
| Q5 | A6 | Answered (Umar, 17-11-2026): the seller bill says "نقد ادا کر دیا" | |
| Q6 | B4 | Answered (Umar, 17-11-2026, after the money design): option R, worked out at read time; he approves the new golden numbers E2E-34, 34b, 37 (list), 40, 40b, 41, the FR-17.9 wording with the open advances, the new split `split_by_open_parts` (C28), and opening advances counting with the shares of their date | |
| Q7 | D3 | Answered (Umar, 17-11-2026): O1, the owner's refund comes into روکڑ as a garden entry رعایت واپسی | |
| Q8 | A6 | Answered (Umar, 17-11-2026): on slips only, balances read آپ کے ذمے / دکان کے ذمے instead of بنام / جمع | |
| Q9 | Gate | Answered (Umar, 17-11-2026): after the build, stop and wait for his "continue" before reviewer and robustness | |

## 8. Design notes (filled by the team before code)

Requirement documents: PRD v1.43 (FR-7.73, FR-7.92, FR-9.3, 4.4, 12.3, FR-12.23, FR-12.29, FR-12.39, FR-12.40, FR-12.47, FR-12.49, FR-12.50, 12.11, FR-16.14, FR-19.8, FR-19.12, FR-19.15, FR-19.16, FR-19.17, R5, glossary). B4, D3 and the opening-capital posting are added after Umar answers the stop points.

- **Print menu and the purpose line of each print** (backend Group 1, approved 17-11-2026):
  - Buyer slips: `reports_buyer_parchi_print_data` / `reports_buyer_parchi_print`, unchanged. Purpose: what one buyer bought today and what he owes.
  - Seller bills: NEW `reports_seller_bills_print_data(DaySlipsInput) -> DaySlipsPrintData` (only `sellers` filled) and `reports_seller_bills_print({slips,to,copies})`. It is the R35 seller slip per seller, titled "<name> کا بل". Scope: whole day, or this بکری / `bikri_ids` (only those lots, the day's balances). R1 stays for print-after-save, with the named title. Purpose: what one seller sold today, his deductions and what the shop owes him.
  - Whole sale print: `reports_sale_print_data` / `reports_sale_print`, output NEW `SalePrintData { frame, paper, date, sellers: ReportPrintData, buyers: PrintReportTable }`. Sellers table = the s17 columns, one row per (بکری, lot) built from the day's sale lines (not R6, so carried lots show), lot charges only on the first posting. Buyers table: buyer, bought in short, bill, نقد, ادھار. Checks: Σ bills = Σ buyer totals; Σ gross = Σ seller amounts. Purpose: everything sold today on one or two pages.
  - ادھاری تفصیل: `reports_collection_sheet_print_data` / `_print` gain `scope` (default `everyone_who_owes`, the plan 76 sheet, kept for its reads; NEW `day_buyers` for this print). Rows = today's buyers. Totals: row 1 today's buyers; row 2 "دوسرے گاہک جو آج نہیں خریدے" with their count; row 3 "کل ادھار، سب گاہک" = row 1 + row 2 (= R20 when there is no وصولی). Dead and جمع buyers keep their own total rows. An opening dated the sheet day counts in پچھلا بقایا on this sheet. Always the whole day. Purpose: how much the shop has to take from its buyers.
  - Statement: see A7 below.
- **Titles:** `ReportPrintData` gains `party: Option<PrintReportParty>` and `named_title_id`. Ids `print-buyer-day-slip-title`, `print-seller-day-slip-title` ("<name> کی روزانہ پرچی"), `print-seller-bill-title` ("<name> کا بل").
- **A6 opening on a slip:** its own line ابتدائی بقایا after پچھلا بقایا, only when the party's opening entry is dated that day, carved out of دیگر اندراج with checked maths, so پچھلا بقایا + ابتدائی بقایا + آج کا باقی = نیا بقایا and پچھلا بقایا stays the balance at the start of the day.
- **Word changes on prints (A6 list):** Umar's: پچھلا بقایا (glossary `-sabiqa-raqam`, one change for every place), نقد جمع کروایا / Cash Paid In (buyer), نقد ادا کر دیا / Cash Paid Out (seller), ابتدائی بقایا / Opening Balance. The one key `report-r35-paid-today` splits into a buyer key and a seller key; R34 "گاہکوں نے نقد جمع کروایا", "بیوپاریوں کو نقد ادا کر دیا". Titles "{name} کی روزانہ پرچی" / "Day slip: {name}", "{name} کا بل" / "Bill: {name}", "{name} کی اسٹیٹمنٹ"; the all-parties table "روزانہ پارٹی فہرست" / "Party day list". Frontend's audit, prints only, screens keep the glossary words, approved by the lead [A, shown to Umar]: (1) آج کا کل → آج کا بل / Today's Bill; (2) آج کا باقی → آج کا ادھار / Credit Today (buyer), آج دینا باقی / Still to Pay Today (seller); (3) کھاتے میں باقی → ادھار / On Credit, موقع پر جمع → نقد / Paid in Cash in buyer print columns; (4) ٹوٹل رقم → کل ادھار, تازہ بنام → آج کا بل; (5) کل اخراجات on the seller bill → کل کٹوتی / Total deductions; (6) پارٹی column on buyers-only sheets → گاہک, ذمے → ادھار; (8) the party line under a named title keeps only the code. Waiting for Umar: (7) آپ کے ذمے / دکان کے ذمے instead of بنام / جمع on slips.
- **F2 cause (frontend, fixed 17-11-2026):** the ✓ marks are `sr-only` spans (position absolute); the conversation scroller was not `relative`, so they attached to the chat window and grew its scroll area to 5,215 px, and `scrollIntoView` scrolled the whole window. Fix: the conversation is `relative` and the only scroller, scrolled directly (no scrollIntoView), the window is `overflow-clip`. Measured after 10 questions: window scrollTop 0, box 13 px from the bottom, ur and en, normal and large, 1366 and 1024. Test `tests/unit/assistant-layout.test.tsx`.
- **F1 and Esc (lead, 17-11-2026):** Back and Esc both go back to the page the user came from (one page trail, falling back to the page's parent, then the dashboard). The FR-7.18 order stays. Pages opened from the main nav or the بکری side menu have no Back button.
- **A1 screen:** one print box: what to print (3 choices), which lines (this بکری / whole day), language, paper, preview, print, PDF. PrintDoc gains `sale_print` (also in "print after save"); `adhari` leaves the F9 box. ادھاری تفصیل has its own box, whole day only.
- **A7, B3 compact statement:** NEW `party_statement_rows(account_id, from, to, offset, limit) -> PartyStatementRows { head, brought_forward, rows, count, totals {dr, cr}, closing }`; each row is one live journal entry of the party (date, kind, number, source, dr, cr, on_spot, running balance, parts = items, edited mark). Print: `party_statement_rows_print_data` and `party_ledgers_statement_rows_print`, items optional (off by default). The 30-day window stays. Check: احسان, opening 20,000 جمع, بکری 1 160,280 (180,280 جمع), ادائیگی 1 100,000 (80,280 جمع); totals 100,000 / 180,280.
- **B1 opening line, place by place:** R13 and R14 already have it; MISSING in پارٹی کھاتے day rows (s09), the statement print (s10) and the R35 slips (s08), fixed by the rows read and the A6 line. R35 table and R34 keep the opening in the day's بنام/جمع (the identity holds). Partner statement has it. روکڑ, الگ کھاتہ and ادھار رقم: checked in Group 2.
- **B2 cause of repeated lines:** all three. (1) a بکری posts one buyer debit line per sale line and R13 lists journal lines; (2) money paid on the spot is a separate credit line per sale line; (3) an edit or cancel writes a same-day reversal and a new entry, so one edit makes 6 rows. Same in R13, R14, the پارٹی کھاتے day detail and the statement print. Fix: the history pages read one row per live entry; a same-day pair (an entry and its reversal, the `SAME_DAY_PAIR` rule of R35) is hidden and the new entry shows once with its edited mark; a reversal on another day stays visible. The closing and the balance after every visible row are the books' figures; the visible dr/cr totals drop by the hidden pairs (lead, 17-11-2026). `include_history` ("منسوخ بھی دکھائیں") shows everything, with totals equal to R13. A cancelled بکری is hidden by default. R13 itself stays line-based for the assistant fixtures.
- **Gap found from plans 76 and 77 (backend, 17-11-2026):** the print and PDF job commands `reports_buyer_parchi_print`, `reports_sale_print`, `reports_collection_sheet_print`, `dashboard_buyer_dues_print`, `dashboard_commission_print` and `partners_statement_month_print` were called by the frontend but never registered, so those prints could not print or save a PDF. Plan 78 adds all of them, plus `reports_seller_bills_print` and `party_ledgers_statement_rows_print`, in the pattern of `party_ledgers_statement_print` (job in `print_report.rs`, a route for the hidden print window, a permission in `capabilities/print.json`, the exact-list test in `tauri_conf.rs`), and a test that every print command the frontend calls is registered.
- **B4 design (money agent, 17-11-2026, option R approved):** read time, nothing posted. Out lines from each پیشگی and each opening debit marked as an advance, split with `split_by_bps` at the advance date's shares (no shares yet: the first shares ever set). Back lines from active `advance_allocations` rows, split with the NEW `core::money::split_by_open_parts` over the parts still open (floor, then the leftover one rupee each from the largest open part, tie lower partner id; never above an open part). `total_of` subtracts each partner's open parts; `events_of` adds the lines. `reports/src/dashboard/strip.rs` has its own `total_of`: both must call one shared function (B5), also the month statement, overview and the assistant check. Property tests `partner_money_properties` and `linked_properties` get the open-advances term. Approved golden changes: E2E-34/34b −6,296 / −4,722 / −4,722 (check 49,260 + 15,000 = 64,260), E2E-37 ب lines (01-09 −4,500; 10-10 1,184, +4,500, 462, −900; 11-10 50,000; closing 50,746), E2E-40 4,040 / 24,460 / −2,100, E2E-40b −33,000 / −500 / −4,500, E2E-41 2,272 / 52,364 / −276, and the plan 76 month statements of those tests (E2E-34 October opening −6,000). Unchanged: E2E-9, 10, 19 (65,000), 28, 29, 30, 31, 35, 36, 42, 44, 46, 50, 64, 65; every trial balance.
- **C3 tests that set the tick (money agent):** the backend has no default (`LotInput.adjust_advance` is a required bool) and every Rust test and JSON fixture already sets it, so no Rust test changes. Old lots keep 1; a later sale, an edit and a carried lot read the stored tick. Frontend: `features/bikri/draft.ts:211` (`!== '0'` treats empty as ticked) becomes ticked only when on; `components/LotMore.tsx:211` moves to the seller summary; `mixed-undo.ts:36` takes the stored tick. Frontend tests that must tick it: `tests/unit/bikri-contract.test.tsx` (Event 1, E2E-26), `tests/helpers/bikri-data.ts`, `bikri-real.ts`, `bikri-over-qty-real.ts`. The summary tick sets every NEW lot of that seller; a carried lot keeps its stored tick (lead [A]).
- **D3 relief posting (money agent, O1 approved):** the rent edit (`services/src/gardens/header_edit.rs`) posts nothing when the rent still due covers the relief. A rent below what is paid leaves "مالک کے ذمے"; the refund is a new garden entry kind `relief_refund` (رعایت واپسی): Dr روکڑ, Cr the garden's 0306, refused above what the owner owes, split at closing (C26). New checks: an installment above the rent still due is refused (`ERR_INSTALLMENT_EXCEEDS_RENT_DUE`). A rent change always asks reason and sign (today the sign is recorded only on a reopened day, `day::fix_sign`). Example: −12,00,000 / −8,00,000 at 60/40 in both cases.
- **E opening capital (money agent):** new field `where`: روکڑ (Dr 01010001, Cr 0602, the screen default) or already counted (Dr 06019999, today's posting, and the default when the field is missing, so `services/tests/partners/setup.rs` stays). A change reverses and reposts on an open date. No golden impact. Card formula: Σ heads 01 + 02 + 03 = Σ کل رقم + 06010001 + 06019999 + open advances; lines in PRD FR-16.14; worked day 65,488 = 65,488. The screenshot shop (openings against ابتدائی فرق, no cash opening) shows 0 = 10,00,000 − 10,00,000 on the card; Umar fixes his data by reopen and fix, or a جنرل dated today (Dr روکڑ, Cr ابتدائی فرق).
- **F1 pages missing واپس:** (frontend list to come)
- **Group 2 (backend, approved 17-11-2026):**
  - C1: NEW `bikri_goods_sources({date}) -> GoodsSources { stock_lots, trip_rows, garden_rows }` (three indexed counts; a button shows only above 0). NEW `bikri_goods_to_load({kind: trip|garden}) -> ReadyGoods` (no sell-date filter; `sell_date` kept as information). باقی مال stays `bikri_stock_list`. The save does not change; `bikri_ready_goods` stays until the screen has switched.
  - D1: already supported (`TripInput.partners[i].share`, percent text with 2 decimals, a 0% partner skipped, total exactly 100% or `ERR_SHARES_NOT_100`); defaults from `trips_defaults` / `gardens_defaults`. Default or custom is a screen choice. Tests: 70/30, 33.33/33.33/33.34, 0% skipped, 99.99 refused.
  - D2: migration 0025, table `venture_items` (venture_id, item_id, unique pair), backfilled from `ventures.item_id`, which keeps the first item. `GardenInput.item_ids`, `GardenHeaderEditInput.item_ids`, `GardenRow.items` and `GardenDetail.items` (additive). Arrival lines are not limited to these items.
  - D4: `goods` (all | consigned | trip | garden) on `CommissionMonthInput`, `CommissionDayInput`, `CommissionPrintInput`, with the analytics SQL fragment; partner and shared-income columns hidden under a goods filter. A test runs views A to E, H and L by goods and checks consigned + trip + garden = all.
  - E read side: the cash book already has a row per cash entry with running balance and from/to windows. Added: `Cashbook.first_entry_date`, `CashbookRow.kind` (opening_cash | opening_capital | capital_in | capital_out | other) with the partner. Cause of s05/s14: `post_opening_capital` posts Dr ابتدائی فرق, Cr 0602, never روکڑ.
- **G pages checked (backend, in code):** party statement rows, R13, R14, partner statement, روکڑ, dashboard drills, trips and gardens lists, ادھاری تفصیل: all windowed or paged with backend totals. Watch: the statement print of a 10-year period for a heavy buyer reads about 55,000 lines. Speed runs at 10 years go to robustness at the gate: `party_ledgers_statement_rows` (30 days, 1 year), the ادھاری تفصیل sheet, `reports_sale_print_data`.
- **Contracts (command names and types):** as above; exact ts types after backend's `pnpm gen:types`.

## 9. Screenshot index

All in `docs/plans/assets/78/`.

| File | Shows | Item |
|---|---|---|
| s01-trip-shares.png | New trip: fixed partner %, only the money source | D1 |
| s02-trip-goods-lines.png | Trip goods lines | C1 |
| s03-baqi-maal-card-empty.png | باقی مال card says nothing left | C1 |
| s04-bikri-baqi-maal-button.png | بکری page with the باقی مال button | C1 |
| s05-partner-statement.png | Partner statement columns | B4 |
| s06-capital-page-no-back.png | سرمایہ page with no back button | F1 |
| s07-print-title-party-slip.png | Title "پارٹی کی روزانہ پرچی" | A6 |
| s08-seller-day-slip-words.png | Seller day slip: سابقہ رقم 0, دیگر اندراج 20,000 | A6 |
| s09-party-khata-no-opening.png | Party khata with no opening line | B1 |
| s10-seller-statement-print-long.png | Statement print, one بکری takes half a page | A7 |
| s11-bikri-summary-lot-switch.png | Summary with the lot switch | C2 |
| s12-seller-bill-title.png | Title "بیوپاری کا بل" | A3, A6 |
| s13-udhari-tafseel-dialog.png | ادھاری تفصیل box with the wrong choices | A1, A5 |
| s14-rokar-page.png | روکڑ page | E |
| s15-garden-one-item.png | Garden form with one item | D2 |
| s16-assistant-ui-broken-1.png | Assistant box pushed up | F2 |
| s17-bikri-tafseel-print-sellers-only.png | بکری تفصیل print, sellers only | A4 |
| s18-assistant-ui-broken-2.png | Assistant box pushed up, again | F2 |
| s19-buyer-history-repeated.png | Buyer history, one بکری on many lines | B2 |
| s20-seller-history-repeated.png | Seller history, one بکری on many lines | B2 |
| s21-dashboard-dues-tiles.png | Dashboard tiles that open the dues pages | B1, B2 |

## 10. Done when

Every item above works as described, proven by passing tests. Every check is green (`pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo test --workspace`), golden tests are unchanged, and nothing that works today breaks.

Proof:

- **Prints:** a PDF of each one (buyer slips, seller bills, whole sale, ادھاری تفصیل, statement) in both languages. Umar checks them against Part A.
- **Paged pages:** a speed run on 10 years of data within NFR-SPD-24.
