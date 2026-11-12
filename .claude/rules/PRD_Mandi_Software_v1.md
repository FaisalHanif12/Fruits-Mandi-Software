# Product Requirements Document (PRD)
# Mandi Software for Commission Agents (آڑھتی)

| | |
|---|---|
| Version | 1.34 (functional requirements; Umar's changes of 11-10-2026, 14-10-2026, 15-10-2026, 18-10-2026, 19-10-2026, 22-10-2026, 27-10-2026 and 30-10-2026, 31-10-2026, 01-11-2026 (three), 03-11-2026 (two), 04-11-2026 and 05-11-2026 (two) and 06-11-2026, his demo feedback of 08-11-2026 (D8, decided 09-11-2026), his decisions of 12-11-2026 (plan 71), and his answers of 15-10-2026 and 30-10-2026) |
| Date | 12 November 2026 |
| Status | Approved for build. Will be revised after the client demo. |
| Client | Maher Adnan and Brothers, New Sabzi Mandi, Kot Radha Kishan |
| Platform | Windows desktop app, one computer, works offline |
| Language | Urdu by default (right-to-left). One switch turns the whole app, including names, into English (left-to-right). |

---

## 0. How to read this document

### 0.1 The document set

This is the **functional** document: what the software must do, screen by screen, field by field, and how every part connects. It is the master. If any other document disagrees with it, this one wins and the other is corrected.

| # | Document | Covers | Status |
|---|---|---|---|
| 1 | `PRD_Mandi_Software_v1.md` (this file) | Functional requirements | v1.34 |
| 2 | `NFR_Mandi_Software_v1.md` | Speed, scalability, reliability, backup quality, security, usability, Urdu and English quality, the assistant | v1.17 |
| 3 | `Tech_Stack_v1.md` | Technologies, Urdu and English stack, repository structure, naming | v1.10 |
| 4 | `Code_Standards_v1.md` | How the code is written, validated and tested | v1.23 |
| 5 | `Architecture_v1.md` | Diagrams and the full data dictionary | v1.22 |
| 6 | `.claude/agents/money-calculations.md` | Claude Code agent for every piece of money code | v1.12 |

**Versions.** All six documents started at version 1.1, the baseline for build. Version 1.2 (12-10-2026) applies Umar's changes of 11-10-2026: one user with no roles, a choice to deduct an advance or not, partner money taken and returned, money lent to staff and others, cash lent to buyers, items in party ledgers, the shop's total اگراہی, and stricter Urdu spelling checks. Version 1.3 (15-10-2026) applies Umar's changes of 14-10-2026: partner capital (the money partners invest), the firm's own goods (bought and sold by the firm for profit or loss), and printing for any day and any party. Version 1.4 (15-10-2026) applies Umar's change of 15-10-2026: one خرچہ screen for shop expenses and a partner's personal spending, and related entries kept on one screen so the app is easy to use. Version 1.5 (15-10-2026) applies Umar's answers to the team's open questions after D1 and D2: commission typed on the بکری line by default, the rule tie, opening balances, a seller's opening credit, the cash warning and return-by dates. Version 1.6 (19-10-2026) applies Umar's changes of 18-10-2026 and 19-10-2026: gardens and market trips owned by the partners in them, one cash account with no bank, fruit taken by partners, complete analytics, the party day sheet, page-by-page lists, and dated lines in each partner's statement. Version 1.7 (22-10-2026) applies what Umar settled from the client's own prints of 18-10-2026: the buyer pays commission and لاگا and the seller commission only, three prints in the client's own layout (the buyer's day slip, ادھاری تفصیل and the all-buyers balance sheet), a clear layout for every other print, a short tag (نشان) after a name, and the firm's contact persons and trade mark on every print. Version 1.8 (27-10-2026) applies Umar's change of 27-10-2026, made while he tested the app himself: short codes by type (the first seller is 0010), an accounts list of sellers, buyers and پختہ کھاتہ only, and pickers that open on a click with a button to add what is missing. Version 1.9 (30-10-2026) applies Umar's change of 30-10-2026: in every table and print, a column of numbers and its total are centred under the column heading. Version 1.10 (30-10-2026) applies Umar's second change of that day: the dashboard shows buyers' dues, sellers' dues, the total اگراہی and the dead amount (ڈوبی رقم); a person who owes and has died or left the business is moved to the dead amount; and every Urdu word says exactly what its English says. Version 1.11 (31-10-2026) applies Umar's change of 31-10-2026: any missing list entry (an expense head, an item, a packing, a city, a seller ...) is added on the spot, from the list where it is needed, and is chosen at once. Version 1.12 (31-10-2026) applies Umar's second change of that day: the buyer's and the seller's day slips show each item with its total and the commission and لاگا on their own lines, the day summary lists every buyer and seller, and every print header carries the trade mark, the owners and the munshi. Version 1.16 (01-11-2026) applies Umar's third change of 01-11-2026: the Owner asks questions about his books in plain words, in Urdu, Roman Urdu or English, and the app answers from the live data with checked figures and a button to the full report (Section 7.7). It was amended on 02-11-2026, before release: the model is the brain, reads every section through read-only tools, adds up, compares and estimates the future, and writes the answer itself, while the app checks every figure. It also advises from the books, like an experienced munshi (FR-7.88). Version 1.32 (09-11-2026) applies Umar's demo feedback of 08-11-2026 (deliverable D8): money paid on the spot (موقع پر جمع) replaces the ادھاری / نقدی toggle, چکتا کھاتہ for walk-in buyers replaces the old one-time accounts, a party ledgers screen, find and edit by buyer, a second print button for ادھاری تفصیل, printing on the Mac, digits 0 to 9 on every print, and Backspace and Delete that only delete. From now on, any change to a document raises its version (1.2, 1.3 ...) and adds a line to its change log. If the change affects other documents, they are updated in the same step.

### 0.2 What we take from the old software

We take **only the business requirements** from the old software: what data is entered, what is calculated, what reports exist, how the modules connect. We do **not** copy its design, layout, colours, speed or way of working. The client's main complaint is that the old software is hard to use. Our design and user experience are our own, and they are the top priority.

### 0.3 Source tags

| Tag | Meaning | What happens at the demo |
|---|---|---|
| **[C]** Confirmed | Umar stated it, or it is plainly required | Build exactly as written |
| **[S]** Seen | Clearly visible in the old software video (as a requirement, not as a design) | Build the requirement. The client will recognise the data. |
| **[A]** Assumed | Our best reading of a vague point. A sensible default, configurable where we can. | Show it in the demo, ask the client, then keep or fix it |

### 0.4 Five rules for the whole project

1. **When something is unclear, we do not stop.** We pick the most sensible option, tag it [A], add it to the Assumptions Register (Section 24), and make it easy to change later.
2. **Everything must be connected.** The demo fails if one screen works alone but does not update the others. Section 6 lists every connection. Section 27 has end-to-end tests with exact expected numbers.
3. **No missing inputs.** Every screen lists every field, including fields the video did not show (hidden inputs) and fields the system fills by itself (system fields).
4. **Money is whole rupees.** Every amount typed, calculated, stored as a result, shown or printed is a whole number of rupees (Section 22).
5. **Two languages, one set of data.** Every screen, print and name exists in Urdu and in English. Urdu is the default. Switching language changes only what is shown, never the data (Section 7.5).

Requirement IDs look like `FR-12.3` (functional requirement, section 12, item 3).

---

## 1. Product overview

### 1.1 The problem

The firm runs a commission agent business in a vegetable and fruit mandi. Trucks arrive with goods from growers. The firm auctions the goods to buyers, charges commission, collects money from buyers (often later), and pays the growers. The firm is owned by several partners who share its income.

The old software:

- Is hard to use. The client's main complaint is poor user experience.
- Shows three different errors in a 4-minute recording.
- Has a daily journal that does not balance (1,316,042 on one side, 1,316,040 on the other).
- Has a manual backup that stays on the same laptop.
- Shows "Session End 31-12-2026" at login. It may stop working on 1 January 2027.
- Has no analytics and no live view of each partner's share.

### 1.2 What we are building

A modern, simple, Urdu-first desktop app that anyone can sit down and use. It covers everything the old software does for daily mandi work, plus:

- Flexible commission rules (by item, seller, buyer).
- Live income split between partners on every posting.
- Analytics: commission and sales by item, seller, buyer, month and year.
- Books that always balance.
- Automatic local and cloud backups.
- Urdu and English: one switch changes the whole app, including every name. Names can be typed in English letters and become Urdu by themselves.
- Gardens and market trips: some partners can rent a garden or send a person to a bigger market to buy goods. The shop sells the goods and takes its normal commission; the profit or loss goes only to the partners in that garden or trip, by their %.
- Fruit taken: fruit a partner takes home, or fruit for the shop's guests, recorded by item and quantity.
- Partner capital: what each partner invests in the shop and takes back.
- Smart entry: type a code or a few letters and the app fills the rest (party details, balances, last values), so the user mostly checks rather than types.
- Ask the app: type a question in any wording, such as "ikram ke total bakaya kitne hai", and the assistant reads the live books, checks every figure, adds up, compares, estimates the months ahead and advises from the books, and answers in clear Urdu with a button to the full report (Section 7.7).

### 1.3 Goals for the MVP

| ID | Goal | How we measure it |
|---|---|---|
| G1 | A new user can record a sale without training | A person who has never seen the app enters بکری 1 (Section 4.4) with only on-screen hints |
| G2 | A munshi can record a full day of mandi work | The day in Section 4 is entered with no help |
| G3 | Every module is connected | All end-to-end tests in Section 27 pass with the exact expected numbers |
| G4 | Books always balance | Trial balance difference is always 0 |
| G5 | Partners see their share live | Partner totals update the moment a بکری is saved |
| G6 | Analytics answer "how much commission from کیلا, from this seller, this month?" | Answer on one screen with a few filter choices |
| G7 | No data loss | Cloud backup every day; restore tested |
| G8 | Natural, professional Urdu | Munshi reads every screen without asking what a word means |
| G9 | The whole app works in English too | Switching to English mid-entry shows every screen, name and print in English, with no data lost (E2E-22) |
| G10 | The app fills, the user checks | Typing a party code anywhere shows his full details at once; a sale line needs only buyer, qty and rate (Section 7.6, E2E-24 to E2E-27) |
| G11 | The Owner asks the books in plain words | "ikram ke total bakaya kitne hai" answers اکرم ٹول ٹیکس 2,880 بنام, checked, with a button to R13; the estimate of the next 6 months' commission matches E2E-59; E2E-47 to E2E-63 pass; NFR-AI-2 is met |

---

## 2. The user

Umar (11-10-2026): only one person uses the app, the commission agent who owns the shop. There are no roles.

| Who | Urdu | Can do | Source |
|---|---|---|---|
| Owner (the commission agent) | مالک | Everything: entries, reports, settings, commission rules, partner shares, day close and reopen, backup and restore | [C] |

FR-2.1 [C] The app has one user account, the Owner, made in the first-run wizard (user name, password, recovery code). There is no screen to add users and there are no roles.
FR-2.2 [C] Every action is open to the signed-in Owner. The backend still checks, before every command except first run, sign-in, recovery, app info and data status, that the Owner is signed in and the app is not locked (NFR-SEC-4).
FR-2.3 [C] The audit log records the Owner as the user of every action. Where this document says "munshi", it means whoever is typing at the computer. It is not a separate login.
FR-2.4 [C] Partners do not log in. They see their share on the dashboard, in the partner overview and in their printed statement (R31).

---

## 3. Glossary (Urdu term, English term, meaning, example)

Developers must use these exact Urdu words on Urdu screens and these exact English words on English screens (FR-7.37). The Urdu words are the ones the munshi already uses. Mandi words with no exact English word keep their Roman name in English.

| Urdu | English (on screen) | Meaning | Example |
|---|---|---|---|
| آڑھتی | Commission Agent (Arhti) | Commission agent (our client) | The firm itself |
| انڈکس | Index | Master list of all accounts | "Add a new buyer to the index" |
| بیوپاری / زمیندار / بزگر | Seller / Zamindar / Grower | Seller side: trader, landowner, grower | رحیم sends 50 پیٹی of کیلا |
| گاہک / خریدار | Buyer | Buyer | اکرم ٹول ٹیکس buys 20 پیٹی |
| چکتا کھاتہ | Chukta Khata | Walk-in buyers who pay the whole bill on the spot and leave. Buyers only. Each is kept by name, is charged like any other buyer, and has no ledger and no balance (Section 13, v1.32) | A walk-in buyer شاہد buys 10 پیٹی سیب and pays all 12,800 |
| نگ | Nag | One unit or package (پیٹی، کریٹ، بوری، دانہ) | 10 نگ = 10 پیٹی |
| بکری | Sale | Sale | |
| بکری بک | Sale Book | Sale book; one بکری = one truck's goods sold | بکری نمبر 1 |
| لاٹ | Lot | One seller's goods of one item on a truck | رحیم's 50 پیٹی کیلا |
| قسم مال | Item | Item | کیلا، سیب، انگور، جاپانی پھل، مولی |
| پیکنگ | Packing | Packing unit | پیٹی، کریٹ، بوری، دانہ |
| مارکہ | Mark | Mark on crates showing whose goods they are | "AR" on رحیم's crates |
| درجہ: اول / دوم / سوم | Quality: First / Second / Third | The quality of the goods on one sale line, which explains a different rate (FR-12.42) | سیب اول @ 1,200, سیب دوم @ 800 |
| اڈا | Adda | Truck stand the truck came from | سوات اڈا |
| خام بکری | Gross Sale | Gross sale before deductions | 29,400 |
| صافی بکری | Net Sale | Net sale payable to seller | 19,316 |
| کمیشن | Commission | Commission: a percent, a fixed amount per نگ, or a fixed amount per lot | 6% of خام بکری (a rule can make it a fixed amount per نگ) |
| لاگا | Laga | Small charge on the buyer, per نگ | 8 per نگ |
| مزدوری | Labour | Labour (loading, unloading) | 250 typed on a lot |
| منشیانہ | Munshiana | Munshi's fee, charged to the seller, paid to the munshi | 50 per lot |
| کرایہ | Freight | Truck freight | 3,000 |
| کرایہ سٹور | Store Rent | Store or cold-storage rent | 0 |
| نقد | Cash Paid | Cash given to the seller at the time of sale | 5,000 |
| فون | Phone Charge | Phone charge on the seller | 20 |
| نقدی منڈی | Naqdi Mandi | The cash buyer with no name (system account 03040000) | Walk-in buyer who pays at once |
| ریٹ فرق | Rate Difference | Buyer value − seller amount when the two rates differ; firm income | Buyer rate 610, seller rate 600, 20 نگ: 200 |
| ادھاری / نقدی | Credit / Cash | The names of the reports R4 and R3, and the state of a sale line: ادھاری when something is left on account, نقدی when something was paid on the spot. A part-paid line is both. There is no toggle (v1.32) | Bill 10,000, paid 3,000: 3,000 نقدی, 7,000 ادھاری |
| موقع پر جمع | Paid on the Spot | Money a buyer pays at the time of the sale, typed at the end of his sale line (FR-12.44). What is left (کھاتے میں باقی) stays on his account | Bill 10,000, paid 3,000: 7,000 left |
| روکڑ | Cash Book (the book); Cash (the account) | The shop's one cash account and its book. From v1.6 there is no bank: روکڑ is all the shop's money, in the drawer or in the bank | Cash in hand 58,484 |
| وصولی | Receipt | Receipt from a buyer | Buyer pays 10,000 |
| اگراہی | Collection | Collecting what people owe the shop, mostly credit buyers. کل اگراہی (Total Dues) is the total of all of it (9.1) | The collection round |
| ادائیگی | Payment | Payment out: mainly to sellers; to a buyer it is cash lent (FR-15.11) | Pay رحیم 4,316 |
| پیشگی کھاتہ | Advance | Advance given to a seller before his goods are sold | رحیم takes 15,000 before season |
| جنرل | Journal | General journal entry | Transfers, corrections |
| خرچہ | Expense (Shop Expense / Personal Drawing) | Expense. دکان خرچہ = shop, split between partners. ذاتی خرچہ = money one partner takes for himself, charged to him only. Both are entered on one خرچہ screen, picked from a list (16.2) | Electricity bill 3,000 |
| ادھار رقم | Money Lent | Money the shop lends to staff or anyone else, to be given back | Staff member اسلم takes 5,000 |
| واپسی | Return | Money given back: by a partner, by a person who borrowed, or by a seller returning his advance | اسلم returns 3,000 |
| نقد ادھار | Cash Lent | Cash the shop lends to a buyer; it is collected with his other dues | اکرم takes 3,000 |
| خالص | Net | A partner's profit share minus the money he took and has not returned (C20) | الف: 996 − 2,000 = −1,004 |
| الگ کھاتہ | Separate Khata | An account kept apart from the partners, commission and profit, for example a family khata (خاندانی کھاتہ). Money paid out makes it minus; money in makes it plus. A garden rented by partners is not a الگ کھاتہ (see باغ). | خاندانی کھاتہ: −10,000, then +4,000 back |
| کل رقم | Partner Total | Everything a partner has in the firm: his investment plus his share of all commission and profit, plus or minus his gardens, trips and fruit taken, less what he took (C22) | ب: 50,000 + 746 = 50,746 |
| سرمایہ | Capital | Money a partner invests in the shop, to start and run it | Partner ب invests 50,000 |
| اپنا مال | Own Goods | Goods bought on a market trip. They belong to the trip's partners; the shop sells them like any lot (12.10) | 20 پیٹی مالٹا bought at 500 |
| خریداری | Purchase | Goods bought on a market trip | خریداری خ-1 |
| تجارتی منافع / نقصان | Trading Profit / Loss | Profit or loss of a market trip or of one of its purchases | 660 on 15 پیٹی مالٹا |
| مارکیٹ سفر | Market Trip | A day trip: a person is sent to a bigger market to buy goods for some partners (12.10) | سفر-1: 80 پیٹی مالٹا |
| باغ / باغات | Garden / Gardens | A garden rented by some partners for a season; its fruit is sold by the shop, and its profit or loss goes only to those partners (12.11) | شاہ پور باغ |
| ٹھیکہ | Garden Rent | The price of a garden for the season, paid in installments | 50,000 |
| قسط | Installment | One part of a garden's rent, paid on a date | 20,000 |
| دکان سے / اپنی جیب سے | From the Shop / Own Pocket | Where a partner's part of a garden or trip payment comes from (FR-12.29) | الف دکان سے، ب اپنی جیب سے |
| پھل | Fruit Taken | Fruit a partner takes home, or fruit for the shop's guests, recorded by item and qty (16.7) | الف: 2 پیٹی مالٹا @ 1,400 = 2,800 |
| سٹاک / باقی مال | Unsold Stock | Goods received but not yet sold | 5 پیٹی of کیلا left |
| شراکت دار / حصہ | Partner / Share | Partner / share | Partner الف, 40% |
| تجزیات | Analytics | Analytics | Commission by item for October |
| روزنامچہ | Day Book | Daily journal | All entries of 10-10-26 |
| کھاتہ / فرد حساب | Ledger / Statement | Party ledger / statement handed to a party | Printed for a buyer |
| بنام / جمع | Debit / Credit | Debit / Credit | |
| میزان | Total | Total | |
| تازہ بنام | Today's Bill | What a party was charged on one day, on his day slip (FR-19.12) | 25,574 |
| سابقہ رقم | Previous Balance | A party's balance at the start of the day | 33,287 |
| ٹوٹل رقم | Total Due | سابقہ رقم + تازہ بنام − وصولی | 58,861 |
| بقایا | Balance | What is still owed after the day | 11,765 |
| کل اگراہی | Total Dues | Everything people owe the shop: buyers' dues + sellers' dues + the dead amount (9.1) | 21,564 |
| ڈوبی رقم | Dead Amount | What a person still owes the shop after he has died or left the business. It stays owed and in کل اگراہی, shown on its own because it may never be collected (FR-10.10) | ارشد: 18,684 |
| نشان | Tag | A short tag after a name, in English capital letters, that tells two people with the same name apart (10.2) | عاشق ظفر K |
| ٹرائل بیلنس | Trial Balance | Trial balance | |

Everyday comparison: the آڑھتی is like a property dealer. The seller is the house owner, the buyer is the purchaser. The dealer takes a fee from both, keeps a record of who owes what, and pays the owner what is left after costs. The dealer's office is owned by three brothers, and every rupee the office earns is split between them by their agreed shares.

---

## 4. The business flow and one full worked day

### 4.1 The daily flow

```
Truck arrives -> بکری بک (date + lots + sales + live summary)
              -> Save = one posting event, which updates:
                   buyer ledgers, seller ledgers, چکتا buyers,
                   income and payable accounts, cash,
                   partner share ledger (live split),
                   stock (باقی مال), analytics, dashboard, اگراہی list
Later         -> sell leftover stock from باقی مال (a new posting event)
              -> بکری side menu: وصولی/اگراہی, ادائیگی, پیشگی کھاتہ
              -> روکڑ: خرچہ (دکان خرچہ or ذاتی خرچہ), پھل, جنرل
              -> باغات, مارکیٹ سفر: the partners' gardens and trips; their goods are sold in بکری like any lot
Any time      -> تجزیات, رپورٹ, شراکت دار (including partner capital)
Closing time  -> Day close + automatic cloud backup
```

### 4.2 Default settings used in the example

All of these are editable. The commission agent can set any other rate for a seller, a buyer, an item or a group of items (Section 11), for example 3%, 4%, 7% or 8%.

| Setting | Default | Evidence |
|---|---|---|
| Seller commission | **6%** of sale value | [C] Umar: 6% by default. Old video: the sales book shows 6% (276 ÷ 4,600) and 4% (180 ÷ 4,500) on different lots, so rates do vary by seller or item. |
| Buyer commission | **6%** of sale value, for every buyer, including چکتا buyers | [C] Umar: 6% by default. (The old video's bills show 5%, for example مولی 700 → 735. The client's current default is 6%.) |
| لاگا | **8 per نگ** on fruit items; none on vegetables | [C] Umar: 8 per نگ on all fruit items. [A] None on vegetables: in the old video, the مولی and ساگ bills carry exactly 5% and nothing more (700 → 735, 1,000 → 1,050). |
| مزدوری, منشیانہ, فون, کرایہ, کرایہ سٹور | **No default.** Typed on each lot, or filled by a rule the commission agent makes. | [C] Umar: set by the commission agent. Old video: مزدوری is 0 on most lots and 5 or 3 per نگ on a few. |
| Partners | الف 40%, ب 30%, ج 30% | [A] Placeholder names and shares |

In the worked day below, the munshi types مزدوری at 5 per نگ, منشیانہ 50 and فون 20 on each lot.

Which accounts are income and which are payables (FR-10.4, FR-17.3):

| Charge | Kind | Shared between partners? |
|---|---|---|
| Seller commission, buyer commission, لاگا, فون, ریٹ فرق | Firm income | Yes, all income is shared |
| مزدوری, کرایہ, کرایہ سٹور, منشیانہ | Money the firm collects for others (labour, transporters, store, munshi) | No, they are payables, not income |
| دکان خرچہ | Firm expense | Yes, every shop expense is split |
| ذاتی خرچہ | Money one partner takes for himself | Charged to that partner only; money he returns adds back (FR-16.2) |
| A garden's or trip's money (payments, sales money after commission) | Belongs to the partners in that garden or trip (12.10, 12.11) | Only to them, by its own %. The commission on its goods is firm income, shared as usual. |
| پھل (fruit taken) | Taken by one partner, or for the shop's guests (16.7) | One partner: his total only. Guests: split equally between the partners, not by the shares. |
| سرمایہ (partner capital) | A partner's own money in the shop (17.4) | No, it is not income; it stays on his capital account |
| الگ کھاتہ (separate khata) | Money kept apart from the firm's business (16.6) | No, never income or expense, never in any partner's total |

### 4.3 Opening position and how tests set it up

This is the position at the start of 10-10-26:

| Account | Code | بنام (Dr) | جمع (Cr) |
|---|---|---|---|
| روکڑ (cash in hand) | 01010001 | 50,000 | |
| رحیم (seller, open advance from 01-09-26) | 0010 | 15,000 | |
| ابتدائی سرمایہ (opening capital) | 06010001 | | 65,000 |
| **میزان** | | **65,000** | **65,000** |

How it is created (this is exactly what the golden test does, and what a new client does on setup):

1. On 31-08-26, opening balances are typed on the account forms: روکڑ opening بنام 65,000; ابتدائی سرمایہ opening جمع 65,000. These post as one opening entry against the system account ابتدائی فرق (06019999), which ends at 0 because both sides are equal (FR-10.9).
2. On 01-09-26, a پیشگی voucher gives رحیم 15,000 cash. Cash becomes 65,000 − 15,000 = 50,000.
3. Days up to and including 09-10-26 are closed. 10-10-26 is open.

### 4.4 Event 1: بکری 1 (regular buyers)

A truck brings رحیم's goods. One lot, رحیم، کیلا-1 (FR-12.45): seller رحیم (0010), item کیلا, packing پیٹی, 50 نگ received. Freight 3,000, paid by the firm. رحیم is given 5,000 cash on arrival (نقد). On the lot the munshi types مزدوری 250 (5 per نگ × 50), منشیانہ 50 and فون 20.

**Sales.** Each line works out its own commissions, so analytics can show commission by buyer as well as by seller.

| Buyer | Qty | Rate | Value (qty × rate) | Seller comm 6% | Buyer comm 6% | لاگا (qty × 8) | Buyer pays |
|---|---|---|---|---|---|---|---|
| اکرم ٹول ٹیکس 0020 | 20 | 600 | 12,000 | 720 | 720 | 160 | 12,000 + 720 + 160 = 12,880 |
| ارشد ملتانی 0021 | 30 | 580 | 17,400 | 1,044 | 1,044 | 240 | 17,400 + 1,044 + 240 = 18,684 |
| **Total** | **50** | | **29,400** | **1,764** | **1,764** | **400** | **31,564** |

How a cell is worked out, for example the first seller commission: 12,000 × 6 ÷ 100 = 720.

Both buyers pay 0 on the spot (موقع پر جمع), so each whole bill stays on his account.

**Seller summary.**

| Line | Working | Amount |
|---|---|---|
| خام بکری | 12,000 + 17,400 | 29,400 |
| کرایہ | entered | 3,000 |
| کمیشن | 720 + 1,044 | 1,764 |
| مزدوری | typed: 5 per نگ × 50 | 250 |
| منشیانہ | typed | 50 |
| نقد | entered | 5,000 |
| کرایہ سٹور | none | 0 |
| فون | typed | 20 |
| **کل اخراجات** | 3,000 + 1,764 + 250 + 50 + 5,000 + 0 + 20 | **10,084** |
| **صافی بکری** | 29,400 − 10,084 | **19,316** |

**The posting event (one journal entry, dated 10-10-26).** It has a sale part and a cash part. Each part balances on its own, so the whole entry balances.

Sale part:

| Account | بنام | جمع | Why |
|---|---|---|---|
| اکرم ٹول ٹیکس | 12,880 | | He owes this |
| ارشد ملتانی | 18,684 | | He owes this |
| رحیم | | 24,316 | 29,400 − (10,084 − 5,000 نقد) |
| کمیشن آمدن (بیوپاری) | | 1,764 | Seller commission |
| کمیشن آمدن (گاہک) | | 1,764 | Buyer commission |
| لاگا آمدن | | 400 | |
| فون آمدن | | 20 | |
| مزدوری کھاتہ | | 250 | Owed to labour |
| کرایہ کھاتہ | | 3,000 | Owed to the transporter |
| منشیانہ کھاتہ | | 50 | Owed to the munshi |
| **Sale part میزان** | **31,564** | **31,564** | Balanced |

Cash part (the نقد given on arrival):

| Account | بنام | جمع |
|---|---|---|
| رحیم | 5,000 | |
| روکڑ | | 5,000 |

Whole entry: بنام 31,564 + 5,000 = 36,564; جمع 31,564 + 5,000 = 36,564.

**Partner split, in the same save.** Shared income = all income in this posting = 1,764 + 1,764 + 400 + 20 = 3,948.

| Partner | Share | Exact | After the rounding rule |
|---|---|---|---|
| الف | 40% | 3,948 × 40 ÷ 100 = 1,579.2 | 1,580 |
| ب | 30% | 3,948 × 30 ÷ 100 = 1,184.4 | 1,184 |
| ج | 30% | 1,184.4 | 1,184 |
| **Total** | 100% | | **3,948** |

Rounding rule: round each share down to whole rupees (1,579 + 1,184 + 1,184 = 3,947), then give the leftover 1 rupee to the partner with the biggest share (if two are equal, the one created first). The total always equals the shared income exactly.

### 4.5 Event 2: بکری 2 (a چکتا buyer)

A truck brings کریم's goods. One lot, کریم، سیب-1: seller کریم (0011), item سیب, packing پیٹی, 10 نگ. Freight 1,000, paid by the firm. No نقد. On the lot the munshi types مزدوری 50 (5 per نگ × 10), منشیانہ 50 and فون 20.

A walk-in buyer, شاہد (phone 0300-0000000), buys all 10 پیٹی at 1,200. He has no account of his own, so the line goes to **چکتا کھاتہ**, code 0030, under his name. His name and phone are saved on the line. He is charged like any buyer: 6% commission and لاگا 8 per نگ. A چکتا buyer always pays the whole bill on the spot, so موقع پر جمع is 12,800 and nothing is left.

| Item | Working | Amount |
|---|---|---|
| Value | 10 × 1,200 | 12,000 |
| Seller commission 6% | 12,000 × 6 ÷ 100 | 720 |
| Buyer commission 6% | 12,000 × 6 ÷ 100 | 720 |
| لاگا | 10 × 8 | 80 |
| Buyer pays | 12,000 + 720 + 80 | 12,800 |
| Seller کل اخراجات | 1,000 + 720 + (10 × 5 = 50) + 50 + 0 + 0 + 20 | 1,840 |
| Seller صافی بکری | 12,000 − 1,840 | 10,160 |

Sale part: چکتا کھاتہ بنام 12,800. کریم جمع 10,160, کمیشن آمدن (بیوپاری) 720, کمیشن آمدن (گاہک) 720, لاگا 80, فون 20, مزدوری 50, کرایہ 1,000, منشیانہ 50. Total جمع = 12,800. Balanced.
Cash part (because he paid 12,800 on the spot): روکڑ بنام 12,800, چکتا کھاتہ جمع 12,800.
Whole entry: 25,600 on each side.

Partner split: shared income = 720 + 720 + 80 + 20 = 1,540. الف 616, ب 462, ج 462. Total 1,540 (no leftover).

### 4.6 Event 3: وصولی (collection)

اکرم ٹول ٹیکس pays 10,000 cash. روکڑ بنام 10,000, اکرم جمع 10,000. He still owes 12,880 − 10,000 = 2,880.

### 4.7 Event 4: ادائیگی to رحیم

رحیم's account: 15,000 بنام (advance) + 5,000 بنام (نقد) − 24,316 جمع (بکری) = 4,316 جمع. So the firm owes him 4,316. Pay it in cash: رحیم بنام 4,316, روکڑ جمع 4,316. His balance becomes 0. The پیشگی report shows his 15,000 advance as fully adjusted.

### 4.8 Event 5: expenses

- دکان خرچہ: electricity 3,000 cash. Split across partners by share: الف 1,200, ب 900, ج 900.
- ذاتی خرچہ: partner الف takes 2,000 cash for home (reason: گھر کا خرچ). Charged to الف only.

### 4.9 End of day: what every screen must show

| Screen | Expected |
|---|---|
| Cash in hand | 50,000 − 5,000 + 12,800 + 10,000 − 4,316 − 3,000 − 2,000 = **58,484** |
| Today's خام بکری | 29,400 + 12,000 = **41,400** (2 بکری) |
| Today's commission (both sides + لاگا) | (1,764 + 1,764 + 400) + (720 + 720 + 80) = 3,928 + 1,520 = **5,448** |
| Today's shared income (commission + لاگا + فون) | 3,948 + 1,540 = **5,488** |
| Buyers' dues | اکرم 2,880 + ارشد 18,684 = **21,564** |
| Sellers payable | کریم **10,160** (رحیم 0) |
| Advances open | **0** (رحیم's 15,000 fully adjusted) |
| Stock (باقی مال) | **0** (everything sold) |
| Partner الف | income share 1,580 + 616 = 2,196; expense share 1,200; profit share **996**; drawings 2,000 (not returned); خالص 996 − 2,000 = −1,004 (FR-16.3) |
| Partner ب | 1,184 + 462 = 1,646; expense 900; profit share **746** |
| Partner ج | 1,646; expense 900; profit share **746** |
| Firm profit today | 5,488 − 3,000 = **2,488** = 996 + 746 + 746 |
| Trial balance | Both sides **85,048** (Section 27, test E2E-10) |

شاہد, the چکتا buyer of Event 2, is in today's sales, commission and partner split. He paid all 12,800 on the spot, so he is not in Buyers' dues.

### 4.10 The same day in English

Every name above has an English name (FR-7.24). The tests use these:

| Urdu | English | Code |
|---|---|---|
| رحیم | Rahim | 0010 |
| کریم | Karim | 0011 |
| اکرم ٹول ٹیکس | Akram Toll Tax | 0020 |
| ارشد ملتانی | Arshad Multani | 0021 |
| شاہد (چکتا buyer) | Shahid | 0030, phone 0300-0000000 |
| کیلا / سیب | Banana / Apple | |
| پیٹی | Box | |
| سوات اڈا / لاہور اڈا | Swat Adda / Lahore Adda | |
| الف / ب / ج | Partner A / Partner B / Partner C | |
| بجلی (دکان خرچہ) | Electricity | |

In the English app every number in Section 4 is the same. Only words and names change.

---

## 5. Navigation and screen map

FR-5.1 [C] After login, the user lands on the Dashboard.

FR-5.2 [C] Main navigation bar on the start side of the screen: on the right in Urdu, on the left in English (FR-7.31). Items, in this order:

| # | Urdu | English | Section |
|---|---|---|---|
| 1 | ڈیش بورڈ | Dashboard | 9 |
| 2 | انڈکس | Masters (accounts, items, commission rules) | 10, 11 |
| 3 | بکری | Sales area, with its own side menu | 12 to 15 |
| 4 | روکڑ | Cash book, expenses, fruit taken, journal | 16 |
| 5 | باغات | Gardens | 12.11 |
| 6 | مارکیٹ سفر | Market Trips (with their purchases) | 12.10 |
| 7 | شراکت دار | Partners | 17 |
| 8 | تجزیات | Analytics | 18 |
| 9 | رپورٹ | Reports | 19 |
| 10 | بیک اپ | Backup | 20 |
| 11 | ترتیبات | Settings | 21 |

FR-5.3 [C] **The بکری area has its own side menu.** When the user opens بکری, a second menu appears next to the main bar:

| # | Urdu | What it opens | Section |
|---|---|---|---|
| 1 | بکری بک | Sales voucher | 12 |
| 2 | وصولی / اگراہی | Receipts from buyers | 15.2 |
| 3 | ادائیگی | Payments to sellers, transporters and others | 15.3 |
| 4 | پیشگی کھاتہ | Advances to sellers | 15.4 |
| 5 | آج کی بکری | List of today's بکری | 12.7 |
| 6 | پارٹی کھاتے | Party Ledgers: each party's balance, day by day (v1.32) | 12.12 |
| 7 | باقی مال | Goods not yet sold | 12.8 |

خریداری (purchases) is in the مارکیٹ سفر section from v1.6, because every purchase belongs to a trip (12.10).

Items 2, 3 and 4 open one voucher screen, with that type already chosen (FR-7.67, FR-15.10).

FR-5.4 [A] Global search (`Ctrl+K`, or the search box at the top) finds any account, بکری number, truck number (an old بکری only, v1.34), voucher number or screen, by Urdu name, English name or code, with the smart search of FR-7.40.

FR-5.5 [A] Bottom status bar: user, today's date, day status (کھلا / بند), last cloud backup time with a coloured dot, and the button "سوال پوچھیں F1" that opens the question panel (FR-7.73, v1.16).

FR-5.6 [A] Top bar: global search box (FR-5.4), attention bell with its count (FR-7.44), language switch اردو | English (FR-7.30), and the user menu (text size, change password, logout).

---

## 6. How the modules connect

### 6.1 Posting events

Every action that changes money is a **posting event**. A posting event has one date, one journal entry, and all its side effects, saved together. Types: بکری (first save), later sale (Section 12.9), reversal (for an edit or cancel), وصولی, ادائیگی, پیشگی, دکان خرچہ, ذاتی خرچہ and its واپسی, ادھار رقم and its واپسی, a trip purchase or trip cost, a garden installment or cost, a sale of trip or garden goods in another shop, a garden's closing, پھل, سرمایہ جمع and واپسی, الگ کھاتہ entries, جنرل, opening balances, month close.

### 6.2 What each action updates (all inside one save)

| Action | Updates immediately |
|---|---|
| Add or edit an account | Pickers everywhere; its opening balance posting; trial balance; dashboard totals |
| Add or edit a commission rule | New sale lines from now on. Saved lines keep their stored values (FR-11.6). |
| Add a partner share change | Splits of postings dated on or after its effective date (FR-17.2) |
| Save a بکری | Buyer ledgers; seller ledgers; the day's sale book with its چکتا buyers; cash (for نقد and money paid on the spot); income and payable accounts; partner share ledger; stock; analytics facts; dashboard; اگراہی list; seller payables; advance matching; bill matching; کرایہ کھاتہ (freight owed); روزنامچہ; trial balance; audit log; draft removed |
| Later sale from باقی مال | Same as above, for the new lines only (Section 12.9) |
| Edit a بکری posting (day open) | Reversal of that posting (journal, split, stock, facts, matching), then the new version posted; audit log keeps the old version |
| Cancel a بکری (all its days open) | Reversal of every posting of that بکری; receipts that were matched to its lines become unmatched credit on the buyer and are re-matched to his next oldest bills |
| Save وصولی | Buyer ledger, cash, bill matching and aging, اگراہی list, dashboard. From a seller: his ledger and advance matching (FR-15.12) |
| Save ادائیگی | Account ledger, cash, seller payables, dashboard. To a buyer: his ledger, bill matching and aging, اگراہی list (FR-15.11) |
| Save پیشگی | Seller ledger (بنام), cash, advance list, پیشگی report, dashboard |
| Save خرچہ as دکان خرچہ | Expense account, cash, partner expense shares, partner profit, dashboard |
| Save خرچہ as ذاتی خرچہ or its واپسی | That partner's drawings, cash, partner overview and statement, attention list, dashboard |
| Save ادھار رقم or its واپسی | The person's ادھار رقم account, cash, attention list, R32, dashboard. A salary deduction also posts the تنخواہ shop expense and its partner split (FR-16.4) |
| Save a trip purchase or trip cost | The trip and its stock (باقی مال), cash and each partner's total for his part from the shop's cash (FR-12.30), dashboard |
| Sell trip goods (a بکری lot of اپنا مال) | Everything a بکری save updates, plus the purchase's stock, the trip's result and each trip partner's total (FR-12.26) |
| Sell trip or garden goods in another shop | Cash, the purchase's stock and each trip partner's total, or the garden's account (FR-12.32, FR-12.36) |
| Write off trip goods | The purchase's stock and the trip's result. Nothing is posted (FR-12.28) |
| Save a garden installment or cost | The garden's record and running balance, cash and each partner's total for his part from the shop's cash (FR-12.35) |
| Close a garden | The garden's account to 0, each garden partner's total and statement line (FR-12.38) |
| Save پھل | The partner's total (or each partner's, for guests), the lot's seller or the other shop's account, the lot's stock when it comes from our shop, the پھل list (FR-16.10, FR-16.11) |
| Save a الگ کھاتہ entry | That khata, cash, its statement, dashboard cash. Nothing for partners (FR-16.6) |
| Save سرمایہ جمع or واپسی | That partner's capital account, cash, partner overview and statement, dashboard |
| Save جنرل | Every account in the entry; partner shares for any line on an income or shop expense account; bill matching for buyer lines; seller lines count in his payables only, never in advance matching (16.4); trial balance |
| Day close | Locks the day; starts the cloud backup; dashboard day status |
| Month close | Moves the month's income and shop expenses into partner capital accounts (FR-17.6) |
| Restore a backup | Everything (whole data replaced after a safety copy) |
| Move to dead amount, or make active again | The account's active state, the dashboard dues tiles, R20 to R22, R24, R26, R32, R38, the attention list, the collection round, the audit log. Nothing posts (FR-10.10) |
| Ask a question (7.7) | Nothing in the books. Only the question log (FR-7.84), in its own save after the answer |

### 6.3 Rules that hold everywhere

FR-6.1 [C] No screen keeps its own separate totals. Every total is read from the posted records (journal, partner share ledger, stock moves, analytics facts), all written in the same save, so totals can never disagree between screens. (The dead amount tiles also read `dead_marks`, which only sorts people into groups and holds no money, FR-10.10.)
FR-6.2 [C] A journal entry whose بنام total does not equal its جمع total is rejected. This cannot be bypassed by any user.
FR-6.3 [C] A posted entry is never changed or deleted. Corrections are made with a reversing entry, so the history is always complete.
FR-6.4 [C] All money is whole rupees (Section 22). The same rounded number is used on every side: on the bill, in the journal, in the partner split and in analytics.
FR-6.5 [C] Every posting that touches an income account or a shop expense account writes partner share rows in the same save, whatever module it comes from (including جنرل). So, for any period, the sum of all partner shares equals income minus shop expenses exactly.
FR-6.6 [A] **Dates and open days.** Days are closed in order. The system keeps one "closed up to" date. A posting can only be dated after that date and no later than today. The one exception is the opening balance of a new account, which may fall on a closed day (FR-10.11, v1.22). Reopening is described in FR-21.2.
FR-6.7 [C] **Cash below zero asks, never blocks.** Umar (15-10-2026): cash in hand can really go below zero, for example when the partners' investment has gone out as advances, buyers have not paid yet, and expenses and partner drawings still go out. So it is allowed, but the user must confirm it, because it is often a typing mistake. It applies to every save that would take روکڑ (cash in hand, 01010001) below 0: پیشگی, ادائیگی, خرچہ, ادھار رقم, سرمایہ واپسی, الگ کھاتہ رقم دی, the shop's part of a trip purchase or cost and of a garden installment or cost (FR-12.30), نقد on a بکری lot, a جنرل with a cash credit, and an edit, cancel or undo that takes cash back.
- While the entry is typed, an amber note shows: "روکڑ صفر سے کم ہو جائے گی" (Cash in hand will go below zero), with the cash in hand after the entry.
- On save, a confirm box asks: "اس اندراج کے بعد روکڑ −5,000 ہو جائے گی۔ کیا یہ اندراج درست ہے؟" (After this entry cash in hand will be −5,000. Is this entry correct?) with two buttons: "ہاں، محفوظ کریں" (Yes, save) and "نہیں، واپس جائیں" (No, go back). The focus starts on "نہیں، واپس جائیں", so pressing `Enter` out of habit never saves a mistake. Yes saves; No returns to the entry with nothing saved and the amount field selected.
- Where the save already asks to confirm (a cancel, FR-7.8), the same box shows the cash line. `Ctrl+Enter` (FR-7.57) asks before it saves and prints.
- The backend works out the cash after the entry with the same function as the save, and accepts such a save only when it carries the user's confirmation. The screen only asks.
Example: روکڑ is 20,000 and a پیشگی of 25,000 is typed in cash. The note shows −5,000. On save the box asks; the munshi sees he meant 2,500, presses "نہیں", fixes it and saves with no question. If 25,000 was right, he presses "ہاں" and it saves.

---

## 7. Ease of use (functional UX requirements)

The client's main complaint is that the old software is hard to use. These requirements describe **behaviour** that makes the app simple. Visual design (colours, fonts, spacing) is covered in the NFR and Code Standards documents.

### 7.1 Simplicity rules

FR-7.1 [C] A person who has never used the software can enter a بکری with only on-screen hints. Every field has a clear Urdu label and a short grey hint inside it (for example "گاہک کا کوڈ یا نام لکھیں").
FR-7.2 [C] One main job per screen. No screen needs scrolling sideways at any supported window size (NFR-USE-15).
FR-7.3 [C] Every action is a visible, labelled button. Keyboard keys are a shortcut, never the only way.
FR-7.4 [A] Each button shows its key next to its name (for example "محفوظ کریں F10").
FR-7.5 [A] Totals and results update live as the user types. There is no "calculate" or "refresh" step to remember.
FR-7.6 [A] Mistakes are explained in plain words (in the app language) next to the field, with what to do (for example "مقدار باقی مال سے زیادہ ہے: باقی 5 نگ ہیں"). No error codes on screen.
FR-7.7 [A] Success is shown as a small message that disappears by itself ("بکری 1 محفوظ ہو گئی"). It never needs an OK click.
FR-7.8 [A] Before anything that cannot be undone easily (cancel, day close, restore), a simple confirm box in Urdu says exactly what will happen.
FR-7.9 [A] Leaving a screen with unsaved work asks "محفوظ کریں؟" with three buttons: محفوظ کریں، محفوظ کیے بغیر جائیں، واپس.
FR-7.10 [A] Pickers search as you type, by code, short code, Urdu name or English name (FR-7.39, FR-7.40). [C] A mouse click on a picker opens its list with every record of that field's type (sellers in a seller field, items in an item field ...), recent first and then by code (Umar, 27-10-2026). Typing filters it by code or by name. A new record of the field's type is added on the spot from the picker, without leaving the screen (FR-7.69).
FR-7.11 [A] Large, easy click targets and readable text for older users. Text size can be changed with `Ctrl +` / `Ctrl −` (within the limits in NFR-USE-18), and the app remembers it per user.
FR-7.12 [A] First-run guide: on first login, a 5-step guided tour of the dashboard, انڈکس, بکری بک, وصولی and reports. Can be replayed from the help menu.
FR-7.67 [C] **Related entries share one screen.** (Numbered after FR-7.66 because it was added in v1.4.) Umar (15-10-2026): the app must not feel like many separate programs. Entries that do the same kind of job are made on one screen. A type list (قسم) at the top says what the entry is. Only the fields for that type show, always in the same places, and the list below the form shows that type's history. Changing the type keeps what the types share (date, amount, narration) and clears the rest. Each type keeps its own number series, its own posting and its own checks, so nothing about the money changes.

| Screen | Types in its list | Section |
|---|---|---|
| خرچہ | دکان خرچہ, ذاتی خرچہ, ذاتی خرچہ واپسی | 16.2, 16.3 |
| واؤچر (opened as وصولی, ادائیگی or پیشگی) | وصولی, ادائیگی, پیشگی | 15 |
| ادھار رقم | رقم دی, واپسی | 16.5 |
| الگ کھاتہ | رقم دی, رقم آئی | 16.6 |
| سرمایہ | سرمایہ جمع, سرمایہ واپسی | 17.4 |
| باغ (one garden) | قسط, باغ خرچ, دوسری دکان میں فروخت, مال آیا | 12.11 |
| مارکیٹ سفر (one trip) | خریداری, سفر خرچ, دوسری دکان میں فروخت | 12.10 |

Example: the munshi opens خرچہ with `Alt+K` and types 3,000 as دکان خرچہ for بجلی. Then he remembers it was الف's home bill. He changes the type to ذاتی خرچہ: the date and 3,000 stay, the expense head is cleared, and the screen asks for the partner and the reason.

FR-7.68 [A] **The app points to the right type.** When an entry looks like another type on the same screen, a note under it offers to switch, keeping what was typed. It never switches by itself. Two cases:
- An ادائیگی to a seller with nothing payable: "اس بیوپاری کو دینے کے لیے کچھ باقی نہیں۔ کیا یہ پیشگی ہے؟" with a button پیشگی بنائیں.
- A دکان خرچہ whose "ادا کیا" is a partner's name (in either script): "یہ شراکت دار کا ذاتی خرچہ تو نہیں؟" with a button ذاتی خرچہ بنائیں.

FR-7.69 [C] **Add on the spot, from any list.** (Added in v1.11.) Umar (31-10-2026): "if there is something available or not, we can add it here and it will save in the list, and the next time it should be shown in the drop down. I'm talking about any kind of drop down: packing, items, any kind of things which we need to add, whether in بکری, خرچہ, روکڑ." The user never leaves his entry to add a missing record.
- **Where.** Every picker on an entry screen: accounts (seller, buyer, چکتا buyer, ادھار رقم person, الگ کھاتہ), item, packing, city, اڈا, مارکہ, store, standard narration, expense head and staff. Not on filter boxes (reports, analytics, list filters, find, the rule test).
- **How.** The open list always ends with one row, for example "+ نئی خرچہ مد شامل کریں" (Add a new expense head). If text was typed, the row names it: "+ «بجلی» کو نئی خرچہ مد بنائیں". In the buyer picker of a بکری line there are two such rows: a new buyer and a new چکتا buyer (FR-13.3). A click, or `↓` to the row and `Enter`, or `F4` in the picker, opens a short form over the screen. The typed text is already in the name.
- **The short form** asks only what the record needs: both names (filled from each other, FR-7.24) and the tag for an account, plus:

| List | Also asks |
|---|---|
| Seller, buyer | City, phone (both optional) |
| Item | Item group (پھل / سبزی), sold by (نگ / وزن, starts at نگ) |
| Packing | Approx weight kg (optional) |
| اڈا | City (optional) |
| مارکہ | The mark text; the owner is the lot's seller when it is opened from a lot |
| Narration | Urdu and English text |
| Staff | Job, phone (optional) |
| Expense head, city, store | Nothing more |
| ادھار رقم person, الگ کھاتہ | As their own `F4` forms (16.5, 16.6) |
| چکتا buyer | Phone (optional) |

  The code is given by itself (FR-10.3). Similar records already in the list show under the name while typing (FR-7.63, FR-10.7), and one can be picked instead.
- **After save.** The record is saved in its master list, is chosen in the field at once, and the cursor moves on with `Enter` as usual. It is in the list next time, on every screen. `Esc` closes the form and leaves the field as it was. Nothing typed on the screen underneath is lost.
- **More details.** A link "مکمل فارم" (Full form) in the short form opens the انڈکس form of that type over the screen, as in v1.8, for fields the short form does not ask (opening balance, credit limit and others).

Example: on the خرچہ screen the munshi opens خرچہ مد and the list is empty, saying "ابھی کوئی خرچہ مد نہیں". He types بجلی and presses `Enter` on "+ «بجلی» کو نئی خرچہ مد بنائیں". The short form shows بجلی and Electricity. He presses `F10`, and بجلی is chosen in the field. Next time بجلی is in the list.

FR-7.70 [C] **Edit every entry, the same way, safely.** (FR-7.70 and FR-7.71 were added in v1.13.) Umar (31-10-2026): a mistake found later, for example a wrong rate or item on a saved بکری, must be corrected on that day's entry. Only what a person typed can be changed; every amount the app works out stays locked and is worked out again.
- **One way in.** Every saved entry, in its own screen and in every list row (آج کی بکری, vouchers, خرچہ, ادھار رقم, الگ کھاتہ, سرمایہ, جنرل, پھل, trips, gardens, the Index, and the rows of روکڑ بک and of every report that opens an entry), has the same two buttons in the same place: تبدیل کریں (Edit) and منسوخ کریں (Cancel). Edit opens the entry as it was saved, with an amber band "آپ بکری 5 (10-10-26) میں تبدیلی کر رہے ہیں" (You are changing sale 5 of 10-10-26).
- **What can change.** Only typed fields: party, item, packing, quantity, rate, weight, a typed lot charge (کرایہ, مزدوری, منشیانہ, فون, نقد, کرایہ سٹور), the commission and لاگا rate on a line (FR-11.10), a voucher's account and amount, an expense's type, head and partner, narration, reason and note; a line can be added or removed.
- **What never changes by hand.** Commission and لاگا amounts, line and buyer totals, خام بکری, کل اخراجات, صافی بکری, balances, اگراہی, payables, cash in hand, partner shares, کل رقم, capital balances, trip and garden results, the trial balance, the entry's number and **its date** [C]. To move an entry to another day, cancel it and enter it again on the right day.
- **Worked out again.** On save the old posting is reversed and the corrected one posted on the same date, in one save (FR-12.19, FR-6.3). Every total in every screen, report and print follows at once, because totals are read from the posted records (FR-6.1). Bill and advance matching run again oldest first.
- **Shown before save.** The screen lists each change ("رحیم، لائن 1: ریٹ 600 تھا، اب 610") and its effect on balances, cash and partner shares, worked out by the backend with the same function as the save (Code Standards B5).
- **Reason.** Required when the entry is not of today; optional for today's entries [A].
- **Any day.** A sale or any other entry of any day can be edited. An open day is edited directly. A closed day is edited through FR-21.4.
- **Where an entry rests on goods or money that moved on, it is locked** [C]: a trip purchase, trip cost or garden payment can be edited only while nothing of the goods it touched has been sold or written off (FR-12.24, FR-12.31); the seller of a lot stays fixed while a later sale, a write-off or fruit taken rests on that lot (A74). The screen says why.

| Section | Can be edited | Locked or checked |
|---|---|---|
| بکری, first posting | Header, lots, lines, the buyer of a line and the seller of a lot (v1.32); add or remove a line or lot | Sold not more than received (FR-12.6); a lot of a closed garden (FR-12.38) |
| Later sale | Its own lines | Lot charges belong to the first posting (FR-12.17) |
| وصولی, ادائیگی, پیشگی | Account or person, amount, narration, collector | The same refusals as a new entry (FR-15.12) |
| خرچہ | Type, head, partner, amount, paid to, reason, return by | The split is worked out again |
| ادھار رقم, الگ کھاتہ, سرمایہ | Person or khata or partner, amount, mode, reason, return by | The same refusals as a new entry |
| پھل | As FR-16.12 | |
| Trip purchase, trip cost, garden installment or cost, sale in another shop | Every typed field | Only while nothing of its goods is sold or written off |
| Accounts and lists | Names, tag, father or firm, city, اڈا, phones, address, credit, active, note; opening balance (FR-10.9) | Code and type |
| Commission rules | A rule no saved line uses | A used rule: add a newer rule from a date instead (FR-11.5, FR-11.6) |
| Partner shares | | Never back-dated (FR-17.2) |

Example (E2E-45): رحیم's buyer اکرم was charged 600 on 10-10-26 but the right rate was 610. The day is closed. The munshi opens بکری 1, presses تبدیل کریں, the app reopens the day with a reason (FR-21.4), he types 610 and sees "اکرم +212، کمیشن +24، الف +10" before saving. After the save every figure is that of E2E-13 and the days are closed again.

Example (E2E-68, v1.32): بکری 1 line 1 was for طارق, not اکرم. After the edit اکرم no longer has the 12,880 bill, so his 10,000 receipt is left over and he has 10,000 جمع. طارق owes 12,880 بنام. Commission and partner shares do not change. The trial balance is 95,048 on both sides. Changing it back returns 85,048.

FR-7.71 [C] **Every edit is stamped where it happened.** Umar (31-10-2026): an edited entry shows when it was changed and by whom, at the place it was changed.
- The entry's header shows "تبدیل شدہ: 31-10-26 4:35 شام، Owner" (Edited: 31-10-26 4:35 pm, Owner), with the reason when one was given. Several edits show the latest, with "2 بار" (2 times).
- Each list row of an edited entry carries a small "تبدیل شدہ" (Edited) mark with the same time and name on hover.
- Inside the entry, each field, line or lot that changed carries a small mark; hovering or `Ctrl+I` shows "پہلے: 600 → اب: 610، 31-10-26 4:35 شام، Owner".
- A "تبدیلیوں کی تاریخ" (Edit history) button lists every version: time, who, reason, and old against new for each field.
- A print of an edited entry, and of a report that holds it, says "نظر ثانی شدہ: 31-10-26" (Revised) in its header.
- The time and name come from the audit log written in the same save (FR-21.3, NFR-SEC-5). Who is the signed-in user's display name [A]. Nothing is typed for the stamp and it cannot be changed or removed.

### 7.2 Arrow-key navigation [C]

Umar: make the keyboard work with the arrow keys (up, down, left, right), so a non-technical person can move around easily.

FR-7.13 [C] **Arrows move the way the user sees.** Because the app is right-to-left, the next field is usually to the left. `←` moves to the field on the left, `→` to the field on the right, `↑` to the field or row above, `↓` to the field or row below.

FR-7.14 [C] **Grids work like a spreadsheet.**

| Key | Moving between cells | While typing in a cell |
|---|---|---|
| `↑` `↓` | Move to the cell above or below | Finish the cell, then move up or down |
| `←` `→` | Move to the neighbouring cell | Move the text cursor inside the cell; at the edge of the text (FR-7.15), finish and move to the neighbouring cell |
| `Enter` | Move to the next **editable** cell in reading order, skipping read-only (calculated) cells. An editable cell that the app filled (FR-7.51, FR-7.52) is still a stop, so the user sees and confirms it. On the last editable cell of a row, start a new row and go to its first editable cell. | Finish the cell, then the same as moving |
| Any letter or number | Start typing and replace the cell content | |
| `↓` on the last row | Create a new empty row | |
| `Esc` | (see FR-7.18) | Cancel the typing in this cell |
| `Backspace` or `Delete` | Clear the cell if it is editable; nothing on a read-only cell (FR-7.91) | Delete text in the cell |

Example for a sale line: buyer code `Enter`, qty `Enter`, rate `Enter`, paid on the spot `Enter` and the cursor is on the buyer code of a new line. Four fields, four `Enter`s. For a buyer who must pay in full the paid cell cannot be typed in, so three (NFR-USE-6, v1.32).

FR-7.15 [C] **Forms.** In normal fields, `↑` and `↓` move to the field above or below. When the user enters a field its whole value is selected; while it is fully selected, `←` and `→` move to the neighbouring field. Once the user types or moves the cursor, `←` and `→` move the cursor inside the text, and when the cursor is already at the visual edge of the text in the arrow's direction, the next press moves to the neighbouring field. `Backspace` and `Delete` delete text only (FR-7.91).

FR-7.16 [C] **Pickers.** Typing in a picker opens its list of matches. While the list is open, `↑` `↓` move through matches and `Enter` picks. While the list is closed, `↑` `↓` move between fields like any other field. `Alt+↓` opens the full list without typing. [C] (Umar, 01-11-2026, v1.14) The list opens as soon as the picker gets the focus, by click or by keyboard, with recent first (FR-7.41). It replaces the v1.8 rule that arriving by keyboard never opens it. While it is open, `↑` `↓` choose in the list, `Enter` picks and moves to the next field, `←` `→` and `Tab` leave the field, and `Esc` closes the list; with the list closed, `↑` `↓` move between fields and rows again. Example: in a بکری line, `Enter` from the qty lands on the buyer box and its list is already open.

FR-7.17 [C] **Menus, lists and reports.** In the main bar and the بکری side menu, `↑` `↓` move between items and `Enter` opens. `←` moves from the menu into the screen, `→` moves back to the menu. In lists and reports, `↑` `↓` move between rows, `Enter` opens the selected row, `Page Up` / `Page Down` move a page.

FR-7.18 [C] **Esc, in this order:** close an open list or dialog; else cancel typing in the current cell; else leave the screen (asking first if there is unsaved work, FR-7.9).

FR-7.19 [A] **Dates.** In a date field, `+` = next day and `-` = previous day (arrows still move between fields). Typing `10` alone fills the current month and year; `10-9` fills the current year.

FR-7.20 [A] **Numbers.** Amount and quantity fields accept the number keypad. Thousands separators appear by themselves. Only whole numbers are accepted for money (Section 22).

FR-7.91 [C] **Backspace and Delete only delete.** (Numbered after FR-7.90 because it was added in v1.32.) Umar (D8 bug 2): in a field they delete text. On a grid cell that is not being edited they clear the cell if it is editable, like a spreadsheet; on a read-only cell they do nothing. They never leave a screen and never open the box of FR-7.9. The same on every entry screen, on Mac and Windows. Example: with the rate cell of a sale line selected, `Delete` empties it, the screen stays, and no box opens.

### 7.3 Other keys

All keys are shortcuts; every action also has a button. The app turns off the built-in browser keys of the Windows web view (reload, browser find, browser zoom, browser print), so these keys only do what is listed here.

| Key | Where | Action |
|---|---|---|
| `F2` | Any entry screen | New (بکری or voucher on the current screen). Inside a grid, F2 does not start cell editing. |
| `F3` | Any entry screen | Find |
| `F4` | In a picker | Add a new account without leaving the screen |
| `F9` | Any screen | Print |
| `Shift+F9` | The sale book | Print ادھاری تفصیل (R5), the second print button (FR-12.12) |
| `F10` or `Ctrl+S` | Entry screens | Save |
| `Ctrl+Enter` | Entry screens | Save, print the ticked documents and start a new one (FR-7.57) |
| `Ctrl+Delete` | In a grid, before save | Remove the current row |
| `Alt+U` | In a text field | Typing mode: رومن سے اردو, اردو کی بورڈ, انگریزی (FR-7.21) |
| `Alt+L` | Anywhere | Language: اردو or English (FR-7.30) |
| `Ctrl+K` | Anywhere | Global search |
| `F1` | Anywhere | Open or close the question panel (FR-7.73) |
| `Ctrl+I` | On a calculated amount | How was this worked out? (FR-7.43) |
| `Ctrl+D` | In a sale-line cell | Copy the value from the line above (FR-7.54) |
| `Alt+Z` | While the saved message shows | Undo the save (FR-7.47) |
| `Ctrl +` / `Ctrl −` / `Ctrl 0` | Anywhere | Text size bigger / smaller / normal |
| `Alt+B` | Anywhere | New بکری |
| `Alt+V` | Anywhere | New وصولی |
| `Alt+A` | Anywhere | New ادائیگی |
| `Alt+P` | Anywhere | New پیشگی |
| `Alt+K` | Anywhere | New خرچہ, starting as دکان خرچہ (16.2) |
| `Backspace` / `Delete` | In a field or grid cell | Delete only, never leave the screen (FR-7.91) |
| `Esc` | Anywhere | See FR-7.18 |

### 7.4 Typing names in two scripts [C]

Umar: users may type in English letters, for example "Arshad". When the app is in Urdu, it should become Urdu.

Every name in the app is kept in **two scripts**: an Urdu name (shown in the Urdu app) and an English name (shown in the English app). The user types only one of them. The app writes the other, and the user only checks it.

FR-7.21 [C] **Three ways to type Urdu, with no Windows setting.**
1. **رومن سے اردو** (English letters to Urdu): type the word in English letters and the app converts it (FR-7.22).
2. **اردو کی بورڈ** (built-in phonetic keyboard): English keys type Urdu letters (`a` → ا, `b` → ب), for exact spelling.
3. A Windows Urdu keyboard, if the user has one, also works: Urdu letters are taken as typed.

`Alt+U` switches a text field between رومن سے اردو, اردو کی بورڈ and انگریزی (no conversion). The current mode shows as a small tag inside the field. Code, number and phone fields always take English digits and are never converted.

FR-7.22 [C] **English typing becomes Urdu.** In the Urdu app, Urdu text fields (names, notes, narrations, reasons) start in رومن سے اردو mode.
- As the user types a word in English letters, up to 3 Urdu spellings appear under the field, the most likely one highlighted.
- `Space`, `Enter` or leaving the field takes the highlighted spelling. `Space` then stays in the field for the next word; `Enter` then moves to the next field as usual. While the list is open, `↑` `↓` move the highlight (as in pickers, FR-7.16).
- `Esc` closes the list and keeps the English letters.
- `Backspace` straight after a conversion brings back the English letters of that word.

Example: typing `arshad multani` gives ارشد ملتانی.

Pickers, global search and code, number and phone fields never convert: they take the letters as typed and search both scripts (FR-7.40).

FR-7.23 [C] **Professional language in both.** All Urdu on screens and prints is professional, natural mandi Urdu, using the glossary (Section 3). All English is plain, professional English, using the glossary's English column. No word-by-word machine translation in either language. Umar (11-10-2026): the Urdu must say exactly what the English says. Every Urdu label, message and print text is checked against its English text by the reviewer, word for meaning (NFR-URD-29). [C] Umar (30-10-2026): where an Urdu text is looser or wider than its English, the Urdu is changed to say the same. Example: the tile "Buyers' dues" is گاہکوں کے ذمے, not وصولی باقی (وصولی باقی reads as everything still to collect). The mandi words of the glossary stay as they are.

FR-7.24 [C] **Every name has an Urdu and an English name.** This covers accounts, cities, items, packings, truck stands, stores, standard narrations, expense heads, staff, partners, the firm, and چکتا buyers. Both names are required. The user types one; the other is filled by itself:
- Typed in English letters (and converted to Urdu by FR-7.22): the English name is exactly what was typed, with capital first letters. `arshad multani` → "Arshad Multani".
- Typed in Urdu letters: the English name is suggested from the Urdu. ارشد ملتانی → "Arshad Multani".
- In the English app the order is reversed: the user types the English name and the Urdu name is suggested from it.

The filled name stays editable. Once the user edits it, it is never overwritten again.

FR-7.25 [A] **What is not translated.** Codes, amounts, dates, phone numbers, truck numbers, tags (نشان), مارکہ text, the market, supplier and other shop typed on a trip, garden or پھل entry, and free text (notes, hand-typed narrations, reasons) are stored once, exactly as typed, and look the same in both languages.

FR-7.26 [A] **The app learns.** When the user picks another suggestion or corrects a converted word, the pair (English spelling → Urdu spelling) is remembered and offered first next time on this computer. The Owner can see and remove learned pairs in ترتیبات. Learned pairs are part of every backup. Learning never changes a name that is already saved.

FR-7.27 [A] **Starting word list.** The app ships with a reviewed word list of at least 15,000 entries (NFR-URD-24): common Pakistani first names, family and firm names, cities and towns of Punjab and Khyber Pakhtunkhwa, fruits, vegetables, packings and the mandi words of the glossary. When Umar exports the old software's account names (about 1,572), they are added after review. Words not in the list are converted by spelling rules and checked by FR-7.66. Everything works offline.

FR-7.28 [A] **Script checks.** An Urdu name must be in Urdu letters. An English name must be in English letters. Digits, spaces and `. - ' ( ) & /` are allowed in both. A wrong script shows "اردو نام اردو حروف میں لکھیں" or "انگریزی نام انگریزی حروف میں لکھیں". The tag (نشان, 10.2) is the one place English letters go with an Urdu name: it is its own field, not part of the name (v1.7).

FR-7.29 [C] **چکتا buyers in the picker (v1.32).** The buyer picker of a بکری line lists چکتا buyers beside ordinary buyers, each marked چکتا, with his phone if he has one. Search works on both his names. Picking one reuses his exact names and phone, so his name is spelt the same way next time. The add row offers "نیا گاہک" (new buyer) and "نیا چکتا گاہک" (new چکتا buyer) (FR-7.69). The other-script name is filled by FR-7.24. Other pickers do not list چکتا buyers (FR-13.4).

FR-7.66 [A] **Spelling check before save.** (Numbered after FR-7.65 because it was added in v1.2.) A converted word that came only from the spelling rules, not from the word list or a learned pair, is marked in amber with "ہجے دیکھ لیں" (check the spelling). On save, if a name still has such a word, the app shows the name with that word marked and asks the user to confirm or fix it. Confirming learns the pair (FR-7.26). The same applies to an English name suggested from Urdu by rules only. A rule-only spelling is never saved without the user confirming it. Free text (notes, reasons) is not checked.

### 7.5 Language switch: اردو | English [C]

Umar: a toggle that stays on Urdu by default. Switched to English, the whole app, including the words and the names, is in English.

FR-7.30 [C] A switch "اردو | English" is always visible in the top bar (FR-5.6) and on the login screen. `Alt+L` also switches. At first run the app is in اردو. The last choice is remembered, and the login screen uses it.

FR-7.31 [C] **Everything changes.** In English, these are all in English: labels, menus, buttons, hints, messages, confirm boxes, report titles and columns, chart labels, month names, amounts in words, and every name (FR-7.24). The layout becomes left-to-right: the main bar moves to the left, and forms, grids and charts are mirrored. In Urdu, all of these are in Urdu, right-to-left.

FR-7.32 [C] **The data never changes.** Switching changes only how things are shown. Codes, amounts, dates, balances and saved records are identical in both languages. Digits are 0 to 9 on screen and on every print and PDF, in both languages, with the same number grouping (Section 21). Urdu digits (۰ to ۹) are never used, and no setting offers them (v1.32, D8 bug 3). Example: 2024, 10,000 and 06-11-2026 on an Urdu print. Amounts in words stay in Urdu words on Urdu prints.

FR-7.33 [C] **Instant, and nothing is lost.** The switch works at once on the open screen, with no restart and no logout. Unsaved entries, the selected row, open lists, filters and the cursor position stay. Example: halfway through بکری 1, the user switches to English. The lines now read "Akram Toll Tax" and "Arshad Multani", the item reads "Banana", the packing "Box", all numbers are the same, and the user carries on typing.

FR-7.34 [C] **Keys follow what the user sees.** Arrow keys move in the direction shown on screen in both layouts (FR-7.13). In English the next field is usually to the right.

FR-7.35 [A] **Search finds both.** Pickers and global search find a record by its Urdu or its English name, whichever language the app is in.

FR-7.36 [A] **Prints.** Every print and PDF uses the app language by default. The print box has a language choice (اردو / English), so, for example, a buyer slip can be printed in Urdu while the app is in English. Excel export uses the app language. The readable backup pack (FR-20.5) uses the default language set in Section 21.

FR-7.37 [A] **Mandi words in English.** Where English has a clear word, it is used (Sale, Receipt, Payment, Advance, Commission, Freight, Labour). Mandi words with no exact English word keep their usual Roman name (Laga, Munshiana, Chukta Khata, Adda, Nag). The glossary's English column (Section 3) and the English names of the seeded accounts (FR-10.4) are the only source.

### 7.6 Smart assistance: the app fills, the user checks [C]

Umar: once a party is in the index, typing his code anywhere should bring up his whole detail. Make every page as easy as possible.

The principle: **the user types as little as possible, the app fills what it already knows, and the user only checks.** Nothing filled by the app is saved without being visible on screen, and every filled value can be changed. The app never fills a rate or an amount of money by guessing.

**Everywhere**

FR-7.38 [C] **A code brings the whole party.** In every account field on every screen (بکری header and lots, sale lines, وصولی, ادائیگی, پیشگی, جنرل, ledgers, statements, report filters), the moment a full code is complete the name fills and a **party card** appears next to the field. A short code (FR-7.39) is taken on `Enter` or on leaving the field, because `1` may be the start of `12`; while typing, the card previews the top match. It shows:
- For everyone: name with father or firm, city, phone, current balance (بنام or جمع), last entry date.
- For a buyer: credit limit and how much of it is used; oldest unpaid bill and its days.
- For a seller: open advance and amount payable.
- For a چکتا buyer: only his name, phone and the words "چکتا: پورا نقد". He has no balance (FR-13.6, v1.32).
- Warnings in amber or red, with a word as well as a colour: inactive, ڈوبی رقم (with its date and reason, FR-10.10), over credit limit, owes the firm.

No extra key is needed: `Enter` moves on as usual.

FR-7.39 [C] **Short codes.** In a seller, buyer, item, packing or staff field, the running number alone is enough (FR-10.2). In a buyer field `0` means 0020 and `11` means 00211; in a seller field `7` means 0017. The full code always works too. Short codes cover sellers, buyers, items, packings and staff; other accounts are found by full code or name. In a field that takes any account (جنرل, ledgers), a short code that fits both a seller and a buyer lists both with their type, for example "گاہک 0020 اکرم ٹول ٹیکس" and "بیوپاری 0010 رحیم".

FR-7.40 [A] **Smart search in every picker.**
- Matches code, short code, Urdu name, English name, tag (نشان), father or firm, city and phone, in either language. Example: "ashiq k" finds عاشق ظفر K.
- Small spelling mistakes are forgiven: "arshd" finds ارشد ملتانی, and "اکرام" finds اکرم ٹول ٹیکس.
- Order: exact code first, then names that start with the typed text, then recently and often used, then close matches.
- Each row shows code, name, father or firm, city and balance, so two people with the same name are easy to tell apart.

FR-7.41 [A] **Recent first.** Opening a picker without typing (a mouse click or `Alt+↓`) shows the last 10 records used on that screen, then the rest by code.

FR-7.42 [A] **Unknown code.** A code that does not exist shows "اس کوڈ کا کوئی گاہک نہیں" (or بیوپاری) with an `F4` button to add a new account with that code. [C] When a picker's list has no match, or nothing of that type exists yet, the list says so in words and offers to add one on the spot (FR-7.69). (v1.11: this replaces the button of v1.8 that opened the انڈکس form.)

FR-7.43 [A] **How was this worked out?** Every calculated amount (commission, لاگا, صافی بکری, partner share, balance, aging days) has a small ⓘ. Clicking it, or `Ctrl+I` on it, shows the working with the real numbers and the rule used, for example "12,000 × 6 ÷ 100 = 720 (اصول: عام بیوپاری کمیشن 6%)".

FR-7.44 [A] **Attention list (توجہ طلب).** A bell in the top bar shows a count. Its list shows what needs action, and each item opens the right screen:
- Unsaved drafts.
- Earlier days that are not closed.
- Lots waiting in باقی مال for more than 2 days.
- Buyers over their credit limit.
- Buyer dues older than 30 days.
- Seller payables older than 7 days.
- Last cloud backup older than 24 hours.
- Money lent (FR-16.4) and partner money taken (FR-16.2) past its return-by date.
- Records with a missing English or Urdu name (possible only after the Phase 2 import of old data).

People in the dead amount are left out of the credit limit, buyer dues, seller payables and money lent items (FR-10.10).

The dashboard shows the same list as "آج کے کام" (Section 9.3).

FR-7.45 [A] **Remembers the last choices.** Per user: the screen that was open at logout, report and analytics filters, print choices (FR-7.57) and text size. The user starts where he left off.

FR-7.46 [A] **Helpful empty screens.** A list with nothing in it says why and what to do next, with a button. Example: "آج کوئی بکری نہیں۔ نئی بکری کے لیے Alt+B دبائیں".

FR-7.47 [A] **Undo straight after saving.** After a بکری or voucher is saved, the success message shows a "واپس لیں" button (`Alt+Z`) for 10 seconds. Undo cancels that posting with a reversal (FR-6.3) with the reason "محفوظ کرنے کے فوراً بعد واپس", then opens a new unsaved entry with the same values so it can be corrected and saved again. The cancelled number is not reused. Undo works only while the date is open.

FR-7.48 [A] **Simple sums in number fields.** Quantity and money fields accept `+` and `−` between whole numbers. Typing `20+25` gives 45. The result shows before the field is left. A result below 0 is not accepted. (In date fields, `+` and `−` still mean next and previous day, FR-7.19.)

**بکری**

FR-7.49 [A] **Truck memory.** (Retired in v1.34: the بکری header has no truck number, plan 71 H2.)

FR-7.50 [A] **Same truck today.** (Retired in v1.34: the بکری header has no truck number, plan 71 H2.)

FR-7.51 [A] **Seller memory.** Choosing a lot's seller fills item and packing from his last lot, and مارکہ from his default مارکہ, else his last lot (all editable). His card shows his open advance and payable. A new lot's seller starts as the previous lot's seller; the first lot starts empty, because the header has no زمیندار (v1.34).

FR-7.52 [A] **Remaining quantity.** A new sale line's quantity starts as the lot's remaining quantity, already selected, so typing replaces it. Example: 50 received and the first line sold 20, so the second line starts at 30. A row whose buyer and rate are both empty is a blank row: the preview and the save ignore it, and save removes it.

FR-7.53 [A] **Rate help, never a guess.** The rate field stays empty and shows a hint with the last rate of the same item and packing: from the earlier lines of this بکری first (saved or not), else today's saved sales, else the last day it sold. The hint must never look like a typed value (v1.32, D8 bug 1): it is smaller and italic, carries a word such as "پچھلا 900", and is not aligned like a number. The hint is never saved by itself. The user types the rate, or presses `Ctrl+D` to copy the rate of the line above.

FR-7.54 [A] **Copy from the line above.** `Ctrl+D` in any sale-line cell copies the value of the same cell in the line above, as in a spreadsheet.

FR-7.55 [A] **Rate check.** When the rate check setting is on (Section 21), a rate more than 30% above or below the item's average rate of the last 7 days shows an amber warning "ریٹ معمول سے بہت مختلف ہے (اوسط 600)". It never blocks saving.

FR-7.56 [A] **Buyer memory.** Choosing a buyer on a line shows his card (FR-7.38). His credit setting decides whether he can leave money on account (Section 12.4). If what is left on the line takes him over his credit limit, the warning shows on the line straight away (FR-12.4), not only at save.

FR-7.57 [A] **Save, print and next in one step.** `Ctrl+Enter` saves, prints the documents ticked in the "محفوظ کرنے کے بعد پرنٹ" list (seller bill, buyer slips, چکتا buyers included), and opens a new بکری. The ticked list is remembered per user.

**وصولی, ادائیگی, پیشگی, خرچہ**

FR-7.58 [A] **Receipts show what is owed.** After choosing a buyer in وصولی, his open bills show oldest first (date, بکری no, amount, still open, days). Buttons "پورا بقایا" (full balance) and "پرانا بل" (oldest bill) fill the amount. While the amount is typed, the list marks which bills it will settle (FR-15.4).

FR-7.59 [A] **Payments show what is payable.** After choosing a seller in ادائیگی, his payable postings show (date, بکری no, seller credit, advance adjusted, paid, still open), with a "پورا بقایا" button. "Paid" and "still open" are worked out when shown, oldest first: the seller's debits that are not advances (ادائیگی, نقد, جنرل) settle his oldest credits left after advance matching (FR-15.8). Nothing extra is saved. The same working gives the age used in FR-7.44. A credit for fruit taken from his lot or from his shop (FR-16.11) is a payable posting too. So is a seller's opening credit (FR-10.9), as his oldest one [C] (Umar, 15-10-2026), and a جنرل credit on a seller, because it never adjusts his advances (16.4). Example: کریم starts with 10,160 جمع from the old books. A payment of 4,000 to him is an ادائیگی against it and leaves 6,160 payable; the پیشگی note of FR-7.68 does not show, because something is payable.

FR-7.60 [A] **Advances show history.** In پیشگی, the seller's card shows his open advances and the date and amount of his last advance.

FR-7.61 [A] **Collection round.** From the اگراہی فہرست (FR-15.6), one button opens a receipt list already filled with that collector's buyers and their dues. The user types only the amounts received and saves them together (FR-15.1). A buyer left at 0 is skipped. People in the dead amount are left out (FR-10.10).

FR-7.62 [A] **Expenses.** The most used expense heads are at the top of the list. "پچھلے مہینے والے دہرائیں" fills this month's regular expenses (for example shop rent) from last month, for the user to check and save.

**انڈکس and reports**

FR-7.63 [A] **Duplicate check while typing.** While a new account's name is typed, similar existing accounts show below it (in both scripts, with code, father or firm, city and phone), before save. This extends FR-10.7.

FR-7.64 [A] **Add a missing city or اڈا on the spot** from the account form (FR-7.69), without losing what is already typed.

FR-7.65 [A] **Party reports from the card.** The party card has buttons کھاتہ (ledger, R13) and فرد حساب (statement, R14) for that party, with the date range already filled.

### 7.7 Ask the app (سوال پوچھیں): a thinking assistant over the books [C]

Umar (01-11-2026 and 02-11-2026): the Owner wants to ask questions about his own business in plain words and get the answer at once, from the live data of every section. Examples: "ikram ke total bakaya kitne hai", "aaj ki bikri kitni hui", "is mahine ka commission", "partner ki kiya total hain abhi tak". The same question comes in many forms ("akram ka hisab kya hai", "akram ka khata dikhao", "akram se kitna lena hai"), and the assistant must understand all of them. The model is the brain: it understands the question, decides what to read, reads it through the app's tools (its arms, which reach every section), checks the figures, works out sums and comparisons, makes estimates of the future from the past, gives advice from the books like an experienced munshi, and writes the answer itself in clear Urdu (or English). A wrong number would defeat the purpose, so no figure reaches the screen unless the app has checked that it came from the books.

The principle: **the model thinks and writes; every figure comes from the books through the app.** The model never types a number from its own head. When it needs a figure, it calls a tool, and the app answers from the same functions the screens use. When it needs a sum, a percent or an estimate, it asks the app's calculator or estimator, which work in whole rupees with the shared money rules. In its answer the model writes a reference where each figure goes, and the app puts the exact figure there before showing it. Example: the model writes "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں"; the app shows "اکرم ٹول ٹیکس (0020) کے ذمے 2,880 بنام باقی ہیں ✓".

Example (after Events 1 to 5): "ikram ke total bakaya kitne hai".
1. The model calls `find` with "ikram" as a buyer. The app finds one match: اکرم ٹول ٹیکس (0020).
2. The model calls `party_card` and `report R13` for 0020. The app reads 2,880 بنام on the party card and 2,880 بنام as the closing balance of R13: the two places agree, so the figure is marked checked (FR-7.76).
3. The model writes its answer with references, and the app fills them: "اکرم ٹول ٹیکس (0020) کے ذمے 2,880 بنام باقی ہیں ✓۔ آج بکری 1 میں ان کا بل 12,880 بنا اور ان سے 10,000 وصول ہوئے۔" Every figure in it is a reference to a tool result.
4. Under the text: his last ledger rows, the source line "ماخذ: پارٹی کارڈ، R13 کھاتہ، 10-10-26 6:12 شام" and the button "پوری رپورٹ کھولیں" for R13.

FR-7.72 [C] **What it does.** The Owner types a question in Urdu letters, in Roman Urdu or in English, in any wording, with spelling mistakes allowed. The assistant answers from the books as they are at that moment, from every section: parties, sales (with چکتا buyers and money paid on the spot), commission, cash, dues, the dead amount, payables, advances, stock, expenses, profit, partners, capital, money lent, الگ کھاتے, trips, gardens and fruit taken. It can:
- give a figure ("akram ka bakaya"), a list ("kis kis ke zimme 10,000 se zyada hai"), a history in short with a link to the full record ("rahim ka pichle 2 saal ka hisaab");
- compare and add up ("is mahine aur pichle mahine ka commission", "akram aur arshad dono ka mila kar");
- explain what it finds in plain words ("commission kyun kam hua": the items and sellers where it fell, from analytics);
- estimate the future from the past ("agar commission isi raftar se chala to agle 6 mahine mein kitna hoga"), always marked as an estimate (FR-7.86);
- advise from the books ("kis se pehle wasooli karun", "kis ko pehle paise dun"), always marked as advice (FR-7.88).
It is read-only: it never saves, edits, cancels or posts anything. It has no tool that writes. The only thing written is its own question log (FR-7.84).

FR-7.73 [C] **Where it lives.** (Changed in v1.17.) A round button with the assistant's icon floats at the bottom of every screen, at the end side (bottom right in English, bottom left in Urdu), just above the status bar. The status bar keeps "سوال پوچھیں F1" (Ask F1) and today's count (FR-5.5). `F1`, or a click on either button, opens the chat window above the floating button, and the same button then closes it. The chat window is a card in the app's own colours:
- at the top: a header with the assistant's name, its state (ready, working, offline), "نیا سوال" (New question), the size button and close;
- in the middle: the conversation, newest at the bottom;
- at the bottom: the quick questions (FR-7.80), then the question box with its send button.

It has two sizes, because answers carry tables and charts:
- normal: about half the window wide and most of its height, never narrower than a report table needs;
- large: nearly the whole window, for long tables.

The size button, or `Ctrl+↑` and `Ctrl+↓` inside the window, switches the size. The normal size stops under the top bar, so search, the bell, the language switch and the user menu stay in reach; the large size covers the whole window. The window never covers a control while it is shut: the floating button keeps its corner clear. Example: the munshi presses `F1` on the بکری screen, asks "akram ka bakaya", reads the answer in the normal window, and presses `Ctrl+↑` to see R13 in full.
- The panel stays open while the user moves between screens, so he can follow a link and come back to the answer.
- The screen under the panel keeps everything typed in it. Opening, using or closing the panel never changes an entry. A link that leaves a screen with unsaved work asks first (FR-7.9).
- `Esc` closes an open list in the panel first, then the panel (FR-7.18). The focus goes back to the field it came from.
- In the question box, `Enter` sends, `↑` brings back the last question, and `Alt+U` does not apply: the box takes the letters as typed, like global search (FR-7.22).
- The moment a question is sent, the panel shows "سوال سمجھ رہا ہوں…" (Understanding the question…). While the assistant works, it then shows each step in the app's own words as it happens, for example "اکرم ٹول ٹیکس کا کھاتہ دیکھ رہا ہوں…" then "حساب جانچ رہا ہوں…" (NFR-AI-3).
- The conversation is kept until the user starts a new one ("نیا سوال" New question), logs out, the app locks (FR-8.4) or closes.
- It works in both directions: in Urdu the panel and its answers are right-to-left, in English left-to-right (Section 7.5).

FR-7.74 [C] **The answer.** Every answer is one block in the conversation with, in this order:
1. **The answer text**, written by the model in the app language: a direct first sentence with the main figure, then up to 5 short points. Every figure and every name in it is put in by the app from the tool results (FR-7.75), names through `displayName` with the tag. A checked figure carries ✓.
2. **Tables and charts** the model chose to show from its tool results: up to 7 rows of a report or view with that source's own columns and the source's totals row over the whole filter (FR-19.11), or a small chart of a monthly series or an estimate. Their cells are drawn by the app from the tool result, never by the model.
3. **The bands.** For an estimate (FR-7.86): amber, "اندازہ، یقینی نہیں" (Estimate, not certain), with the method and the months it is based on. For advice (FR-7.88): "مشورہ، فیصلہ آپ کا" (Advice: the decision is yours). An answer can carry both.
4. **Buttons**: "پوری رپورٹ کھولیں" (Open the full report) for each source read, opening that screen with the same filters already in place; each table row opens its own record (FR-19.4); "پرنٹ" for a source report (FR-19.10).
5. **The source line**: every source read, with the time, for example "ماخذ: پارٹی کارڈ، R13 کھاتہ، 10-10-26 6:12 شام".
6. **Up to 3 next questions** the model suggests, as chips ("پچھلے مہینے کا؟", "اکرم کی پرانی بکریاں"). They pass the same word checks as the answer, hold no figure, and hold a name only as a reference (FR-7.76). A click sends it as a new question.

The panel's own words (buttons, labels, the band, messages) come from the Fluent files in both languages (NFR-URD-4). The answer text is written by the model under the checks of FR-7.76.

Example of a long history: "pichle 2 saal mein rahim ka hisaab". The model reads R13 for رحیم from 01-11-24 to 01-11-26 and writes his balance today, what he sold and was paid in the 2 years, and his advances, from the report's totals. Below: the last 7 rows, and "پوری رپورٹ کھولیں" opens R13 for رحیم with those dates (NFR-SPD-24).

FR-7.75 [C] **Every figure comes from the books, never from the model.** A figure in an answer is one of:
- a cell, a totals cell or a summary line of a source the app read for this question (a report R1 to R38, an analytics view A to K, a dashboard tile of 9.1, or the party card of FR-7.38);
- the result of the app's calculator (`calculate`): a sum, a difference, an average, a percent of, or a change in percent, of figures already read for this question. It works in whole rupees with the shared money rules (Section 22); its working is shown with ⓘ, as in FR-7.43;
- the result of the app's estimator (`estimate`, FR-7.86);
- a number the user typed in his own question ("agle 6 mahine", "10% zyada", "3 lakh"), read by the app from the question, shown in a plain style and never with ✓.
In its answer the model writes a reference to each figure, never the figure itself: no digits and no number words. The app puts the exact figure in. Counts, dates, months and names are references too. A money figure is shown with its side (بنام or جمع) when it has one, for example a balance; a sum of signed balances takes the side of its result. A difference or a change in percent is shown with its direction word (زیادہ or کم; more or less), put in by the app, so the model never chooses a side or a direction word for a figure. Rows of a report are referred to by their record, not by their place in the list, so a reference always means the same entry.
- Example: "akram aur arshad dono ka mila kar kitna hai". The model reads both balances, each checked (2,880 and 18,684), and asks `calculate` for their sum: 2,880 + 18,684 = 21,564. The answer: "دونوں کے ذمے مل کر 21,564 بنام ✓ (اکرم 2,880 بنام ✓، ارشد 18,684 بنام ✓)", and ⓘ on 21,564 shows the working.
- Example: "is mahine ka commission pichle mahine se zyada hai?" The model reads analytics view A with the compare filter (18.1), which gives both months and the difference, and writes the answer from them.

FR-7.76 [C] **Checked before it is shown.** No answer reaches the screen until the app has checked it:
1. **Every figure is a reference** to a value of this question's tool results, the calculator, the estimator, or a number the app read from the user's question. Outside a reference, the answer and its next questions may hold no digit of any script (0-9, ۰-۹, ٠-٩ and every other), no number word on the app's list (ایک to دس, سو, ہزار, لاکھ, کروڑ, ارب, آدھا, ڈیڑھ, ڈھائی, one to ten, hundred, thousand, lakh, crore, million, half, double and the like), no month name, no name returned by this question's tools (the names of parties, partners, items, gardens, trips, cities, اڈے and staff; a name that is also a glossary word or a seeded system account, such as روکڑ or بجلی, is not checked), and no ✓ (only the app puts it on a checked figure); any of these refuses the answer. A reference to a value that does not exist refuses it, and so does a reference to a result of an earlier question: every figure is read for this question, from the books as they are now.
2. **Checked twice.** Where the app keeps one figure in two places that must agree, every tool result that holds that figure (a party's balance in `find`, `party_card` or R13; a tile; a report total) carries both places, read in the same moment, and the figure is marked checked (✓) only when they are equal. A reference to such a figure whose two places differ refuses the answer; the panel then shows "ریکارڈ میں فرق ملا، رپورٹ کھول کر دیکھیں" (A difference was found in the records; open the report), and the attention list gets an item (FR-7.44). The pairs:

| Figure | First place | Second place |
|---|---|---|
| A party's balance | Party card (FR-7.38) | Closing balance of R13 as of now |
| Buyers' dues | Tile گاہکوں کے ذمے (9.1) | Total of R20 (FR-19.9) |
| Sellers' dues and the dead amount | Tiles بیوپاریوں کے ذمے and ڈوبی رقم | The بیوپاری کے ذمے total of R24, and the total of R38 |
| Total dues | Tile کل اگراہی | Each of its three parts checked as in the two rows above |
| Cash in hand | Tile موجود روکڑ | Closing balance of R12 as of now |
| One day's sales and commission | Tiles آج کی بکری and آج کا کمیشن (today) or R34 (any day) | View A for the same day, totals row |
| A partner's کل رقم | Partner overview (17.3) | Closing total of R31 |

A figure from the calculator is checked when every figure it was made from is checked; its working is shown with ⓘ.

3. **Words.** The answer and its next questions are in the app language and in its script (Urdu letters in Urdu, English letters in English, apart from references). They use the glossary words (Section 3), never a word on the app's never-use list (for example پکا کھاتہ or ووچر, NFR-URD-7). The answer is at most 130 words, counted after the references are filled, each filled reference as one word [A]. **Side words:** around each balance, up to the nearest punctuation mark, new line, point mark or other money figure on each side (a name or date never ends it), the words of the other side refuse the answer: "کے ذمے" or "سے لینا" (he owes the shop) with a جمع balance, or "کو ... دینے" or "کی ادائیگی باقی" (the shop owes him) with a بنام balance. The word lists are in `docs/deliverables/D6-system-prompt.md` Section 6.
4. **Kind.** An answer that uses an estimate is of kind estimate or advice and carries the estimate band (FR-7.86); advice carries the advice band (FR-7.88); a figure from an estimate can never be shown as a figure from the books.
If a check fails, the app tells the model exactly what failed, and the model writes the answer once more; a figure whose two places differ is the one failure that is final, with no rewrite (point 2). If the second answer also fails, the panel shows the tool results the model read, as plain tables with their sources, and "میں جانچا ہوا جواب نہیں لکھ سکا، نیچے اصل ریکارڈ ہے" (I could not write a checked answer; the records are below). An unchecked figure is never shown.

Example: "kul ugrahi kitni hai" after Events 1 to 5. The tile کل اگراہی is 21,564, and each part agrees with its report: buyers' dues 21,564 = R20 total 21,564; sellers' dues 0 = R24 0; dead amount 0 = R38 0. The answer shows "کل اگراہی 21,564 روپے ✓".

FR-7.90 [C] **Instant answers for simple figures.** (Added in v1.25.) Umar (05-11-2026): a simple real-time figure question is answered by the app itself, with no model call, so the answer comes in under a second.
- **When.** The app has understood the question for sure, and it asks for one figure, or for one figure per named party, item or period, from one read: a party's balance, dues, payable or open advance; today's or a period's sale or commission, for the shop, an item, a seller or a buyer; cash in hand; the shop's total dues or payables; goods left; a partner's کل رقم (behind the partner code, FR-7.82).
- **How.**
  - The app reads the figure with the same tools and the same checks as FR-7.76, including the two places that must agree.
  - It writes the answer from a fixed, reviewed sentence in the app language, with the figure as a checked reference.
  - It adds the same buttons as any answer.
- **Everything else** still goes to the model: comparisons, advice, forecasts, two-part questions, follow-ups and anything not sure (FR-7.87).
- **Example.** "akram ka bakaya" after Events 1 to 5 gives "اکرم ٹول ٹیکس کا بقایا: 2,880 بنام ✓" with the buttons کھاتہ and فرد حساب, with no model call and no count in the daily limit (FR-7.81).

FR-7.77 [C] **Names and periods.** The model finds people and things with the tool `find`, which uses the smart search of FR-7.40 (code, short code, both names, tag, city; small mistakes forgiven) in the role the question names (buyer, seller, چکتا buyer, ادھار رقم person, الگ کھاتہ, partner, garden, trip, item).
- [C] (v1.34, plan 71 L9) A lot is found by its name too (FR-12.45). A question that names the lot ("ehsan aam-2", "ehsan-2 ke aam") is answered at once. "ehsan ke 200 peti aam" with more than one mango lot of احسان asks "کون سا؟" with a button for each lot: its name, date and received qty.
- One match: it is used. Two to five matches: the answer asks "کون سا؟" (Which one?) with a button for each (name with tag, code, city, balance); a click answers with no new question counted. More than five: the five closest, and "اور بھی ہیں، کوڈ لکھیں" (There are more, type the code). No match: "یہ نام نہیں ملا" (Name not found) with the three closest names as buttons. A person in the dead amount is found and shown as his party card shows him (FR-10.10).
- Periods are worked out by the tool `period`, from the computer's date, by a fixed list: آج, کل (yesterday), یہ ہفتہ, پچھلا ہفتہ, یہ مہینہ, پچھلا مہینہ, یہ سال, پچھلا سال, اب تک (up to now), a named month (the latest one that has started), a day (DD-MM, this year), a range, "last N days", "last N months", "last N years" (each from the same date N back to today: on 01-11-26, "pichle 2 saal" is 01-11-24 to 01-11-26, and a month end moves to the last day of a shorter month) and "next N months" (for estimates). A number of days, months or years must be a number of the question with that word after it. A period the list does not hold is not guessed: the answer asks for the dates. A date the model passes to any tool is checked like a typed date (Code Standards 4.2).
- There is no default party. A question that needs a party and names none asks for one.

FR-7.78 [C] **Follow-up questions.** The panel keeps the conversation: the earlier questions of this panel, the tools called and their results, and the answers. "aur pichle mahine?" after a question about اکرم asks about اکرم for last month. Only the last 3 questions are sent with a new one, within the size limit of NFR-AI-5. "نیا سوال" forgets the conversation.

FR-7.79 [C] **When it cannot answer, it says so.** It never guesses.
- A question outside the books (tomorrow's rate in Lahore, the weather, legal advice): "یہ سوال کھاتوں سے باہر ہے" (This question is outside the books), with the nearest report when there is one. Advice from the books is allowed (FR-7.88).
- A request to change something ("akram ki 5000 wasooli likh do"): "میں صرف بتا سکتا ہوں، کچھ لکھ نہیں سکتا" (I can only tell, not write), with a button that opens the right entry screen with the party chosen and every amount empty (Section 7.6: the app never fills money). Nothing is saved.
- A tool call the app cannot run (an unknown report, a filter that report does not have, a bad date) gets an error back, and the model may try again within the limits of NFR-AI-5. If it still cannot answer: "سوال سمجھ نہیں آیا، دوسرے الفاظ میں پوچھیں" (I did not understand; please ask in other words). Everything is logged (FR-7.84).

FR-7.80 [C] **Quick questions, with no internet and no model.** When the panel opens, a row of chips answers the common questions at once from the app alone. They work offline and are not counted (FR-7.81): آج کی بکری, آج کا کمیشن, موجود روکڑ, گاہکوں کے ذمے, کل اگراہی, ڈوبی رقم, بیوپاریوں کو ادائیگی باقی, پیشگی باقی, باقی مال, and کسی پارٹی کا کھاتہ (a party's ledger), which opens a party picker inside the panel. Their answers use fixed reviewed words with the figure, the check mark of FR-7.76 where the figure has a second place, the source and the buttons of FR-7.74.

FR-7.81 [C] **Internet, the key and the daily limit.** A question to the model works only when the assistant is on, the OpenRouter key is saved (21) and the computer is online.
- **Limit:** 20 questions a day, a hard stop (Umar, 01-11-2026 and 02-11-2026). One question counts once, however many tools the model calls for it. The day starts at midnight, Pakistan time. A question counts when the model's first reply arrives; a call that fails before any reply does not count [A]. Chips, a click on a "کون سا؟" button, the partner code of FR-7.82 and the automatic rewrite of FR-7.76 do not count; a click on a suggested next question does. The panel shows "آج 5 / 20". At 20: "آج کے 20 سوال پورے ہو گئے، کل دوبارہ پوچھیں" (Today's 20 questions are used up; ask again tomorrow), and the chips still work.
- **No internet:** "سوال کے لیے انٹرنیٹ چاہیے، نیچے والے فوری سوال چلتے ہیں" (Questions need the internet; the quick questions below still work).
- **No reply in time, or an error from OpenRouter:** "جواب نہیں آ سکا، دوبارہ کوشش کریں" (No answer came; try again) (NFR-AI-3).
- **Off, or no key:** the button in the status bar opens the panel with the chips only, and a line that the Owner can turn questions on in Settings.
- The rest of the app never waits for the assistant and never fails because of it (NFR-AI-7).

FR-7.82 [C] **Partner figures stay behind the partner code** (FR-17.11). A tool that reads a partner's totals or history (R30, R31, view F, or any source with a partner filter) is refused with `ERR_PARTNER_LOCKED` (NFR-SEC-14) while the section is locked. Every other source behind the code in FR-17.11 runs, but without its partner parts (R29, R33, R34, R36 and R37 by partner, the partner columns of views G, I, J and K), which are marked locked, as on its screen. The question pauses at once: the answer shows the "کوڈ درج کریں" box, and no model call is made with a locked result. After the code, the app runs the paused tool calls and sends their results, and the model goes on, with no new count. Leaving the answer locks it again, as on any screen. Nothing of a partner's figures is sent to the model while the section is locked.

Example: "partner ki kiya total hain abhi tak", after Events 1 to 5 and the code 4826: "شراکت داروں کی کل رقم: الف −1,004، ب 746، ج 746 ✓" (checked against R31), with "پوری رپورٹ کھولیں" for the partner overview.

FR-7.83 [C] **What leaves the computer.** The client agreed that questions, and the records the model needs to answer them, are sent to OpenRouter (Umar, 01-11-2026 and 02-11-2026). These go: the question as typed; the conversation of FR-7.78; the app language and today's date; the fixed instructions with the list of tools, reports, views and tiles and the glossary; and the results of the tools the model calls for this question (the names, codes, tags, cities, amounts, quantities, rates and dates of the records it reads, including a چکتا buyer's name). These never go: phone numbers, addresses, father or firm names, notes, reasons, narrations typed by hand, driver names, the supplier and shop names typed on trip, garden and پھل entries, the market and the person sent typed on a trip, مارکہ text, the audit log, passwords, the partner code, the backup and cloud keys, the OpenRouter key, and partner figures while the partner section is locked. They are left out of the conversation of FR-7.78 too: a partner result of an earlier question is not sent again once the section is locked. The request goes only to the model's Azure endpoint on OpenRouter, which keeps no data and does not train on it, and never falls back to another endpoint (Umar, 02-11-2026; NFR-AI-4). OpenAI's own endpoint is not used, because it is not a zero-retention endpoint. If the Azure endpoint is down, the question gets the message of FR-7.81 and the quick questions still work. The key never leaves the backend and is never shown to the screens.

FR-7.84 [C] **Question log (سوالات کا ریکارڈ).** Every question is written to a log on the computer, in its own save after the answer: the time, the question as typed, whether it was a chip or a model question, each tool the model called with its inputs, the answer as the model wrote it (with its references, not the figures), the outcome (answered, estimate, advice, which one, need dates, need party, not found, outside the books, change refused, not understood, checks failed, records differ, locked, limit reached, offline, error), the model name, the number of model calls and the tokens used, and whether it counts in the daily 20. A continuation (after "کون سا؟" or the partner code) is its own row, linked to its question and not counted; a connection test is its own row and counted; reading an answer again is its own row and not counted. The figures shown are not stored: the tools and their inputs are, so an answer can be read again. The log is part of every backup. In Settings the Owner sees the log, page by page, and can open any answer again: the tools run again on today's books and their tables show, with a note that figures may have changed since; the model's old words are not shown over new figures. The log is never edited or cleared from the app. The daily count of FR-7.81 is the number of today's counted rows.

FR-7.85 [C] **Settings for the assistant** (Section 21, group "سوال پوچھیں"):
- On or off. It starts off, and can be turned on only once a key is saved.
- The OpenRouter key: pasted once, kept in Windows Credential Manager (NFR-SEC-7, NFR-SEC-15), shown afterwards only as "••••" with its last 4 characters; "تبدیل کریں" (Replace) and "ہٹائیں" (Remove). Saving, replacing and removing need the Owner's password and are in the audit log, without the key.
- The model: `openai/gpt-6-luna` by default, changeable to another OpenRouter model name. A change is in the audit log, and NFR-AI-2 is run again for the new model before it is used at the shop [A]. The new model must have a zero-retention Azure endpoint (FR-7.83); if it does not, the connection test fails and says so.
- "کنکشن جانچیں" (Test the connection): sends one fixed test question and shows whether it worked, with the time it took. It counts as a question.
- Today's count (FR-7.81) and the question log (FR-7.84).
The daily limit of 20 is fixed, not a setting.

FR-7.86 [C] **Estimates of the future (اندازہ).** (Added 02-11-2026.) Umar: "if the commission is going at this speed, what would it be in the upcoming 6 months?" The model asks the app's estimator, which works out an estimate from the books by a fixed method. The model never makes up a future figure.
- **What can be estimated:** any monthly series the books give: sales (خام بکری), commission (seller, buyer, لاگا, or all), for the whole shop or one item, seller or buyer; shop expenses; profit; وصولی; a garden's or trip's sales.
- **Balances too** (v1.26, Umar 05-11-2026). "akram ka bakaya isi raftar se agle mahine kitna hoga", the cash in hand, or what all the buyers owe. The estimate is the balance now (from the books, the same figure as the party card or the tile) plus the average net change of a month times the months ahead. Only the same pace method is used, and it needs 3 full months.
  - The net change of a month is its debits less its credits on that account. Left out, because they are not the normal pace of business: openings and their reversals, partner capital put in or taken back (cash, 17.4), and the dead amount (buyers' dues, v1.27).
  - A party's balance can end on either side, and it is shown with its side word (بنام or جمع). An estimate of what the buyers owe that would fall below 0 is refused.
  - A balance has no target, no what-if and no year end.
  - Example: اکرم owes 2,880 today. His net change in the last 3 full months was +3,000, −1,000 and +4,000, so the average is +2,000 a month. After 2 months he would owe 2,880 + 2,000 × 2 = 6,880 بنام.
- **Methods** [A]:
  1. **Same pace** (اسی رفتار سے): the average of the last 3 full months, for each month ahead.
  2. **Same growth** (اسی اضافے سے): the last full month, plus the average change per month over the last 6 full months, for each month ahead; a month is never below 0.
  3. **Same season** (پچھلے سال جیسا): the same months of last year, as they really were (needs 12 full months).
  By default the answer shows methods 1 and 2 side by side; 3 is added when the books hold 12 full months. The current month is not used, because it is not finished.
- **Too little history:** method 1 needs 3 full months, method 2 needs 6; with less, the answer says how many months there are and does not estimate.
- **How it shows:** the amber band of FR-7.74, "اندازہ، یقینی نہیں", with the method and the months used; a small chart of the past months and the months ahead. An estimate is never saved in the books, never shown in a report or on the dashboard, and never checked twice (there is no second place); its working is shown with ⓘ.
- **Targets:** "3 lakh ka target kab tak pura hoga". The app reads 3 lakh = 3,00,000 from the question, and the estimator says, by each method, in which month the running total reaches it (at least the target), counting from the first month ahead, which is the current month, as in E2E-59; or that it does not within 12 months. Example (E2E-59 data): same pace 48,000 × 6 = 288,000 is short, 48,000 × 7 = 336,000 reaches it, so April 2027; same growth 54,000 + 57,000 + 60,000 + 63,000 + 66,000 = 300,000 reaches it exactly, so February 2027.
- **To the end of the year** (v1.20, Umar 02-11-2026: "look at the previous sales and commission, calculate the rest of the months, and provide the total at the end"). "saal ke aakhir tak kitna" gives the whole year's total: the months of this year so far, read from the books (view G), plus the estimate for the months left. The part still to come is shown too. Example (E2E-59 data, today 10-10-26, same pace): January to September from the books 261,000, plus October to December 48,000 × 3 = 144,000; whole year 405,000.
- **What-if** (v1.20). "agar raftar double / aadhi / teen guna ho" and "agar raftar 10% zyada / kam ho" scale the same-pace month by that factor: one rounding, half up, through the shared function. The total is that month times the months asked. The app checks that the Owner said the scenario: the model never chooses a factor. Example: 48,000 doubled is 96,000 a month, 288,000 in 3 months.
- **How far:** at most 12 months ahead [A], or to the end of the year.
- All of its arithmetic is in whole rupees with the shared money rules (Section 22; Code Standards M14).

Example (E2E-59): today 10-10-26; the shop's commission (view A, all commission) in the last six full months: April 36,000; May 39,000; June 42,000; July 45,000; August 48,000; September 51,000. The question: "agar commission isi raftar se chalta raha to agle 6 mahine mein kitna hoga".
- Same pace: (45,000 + 48,000 + 51,000) ÷ 3 = 48,000 a month; October 2026 to March 2027: 48,000 × 6 = 288,000.
- Same growth: (51,000 − 36,000) ÷ 5 = 3,000 a month; October 54,000, November 57,000, December 60,000, January 63,000, February 66,000, March 69,000; total 369,000.
- The answer: "اگر کمیشن پچھلے 3 مہینوں کی رفتار سے چلتا رہا تو اکتوبر 2026 سے مارچ 2027 تک تقریباً 288,000 بنے گا (ہر مہینے 48,000)۔ اگر ہر مہینہ پچھلے سے 3,000 زیادہ رہا تو 369,000۔" with the band "اندازہ، یقینی نہیں".

FR-7.87 [C] **The tools (the assistant's arms).** (Added 02-11-2026.) The model can only reach the books through these tools. Each one is read-only and calls the same function as the screen it names (Code Standards Q3).

| Tool | What it gives | Same as |
|---|---|---|
| `find` | People and things that match a word, in a role: code, name with tag, city, balance | Global search and pickers (FR-7.40) |
| `party_card` | A party's card: balance, credit, oldest bill, open advance and payable, warnings | FR-7.38 |
| `report` | A report R1 to R38 with its filters, a filter on a column ("10,000 se zyada") and a sort: up to 100 rows (20 unless asked), the totals over the whole filter, the summary lines | The report screens (19) |
| `analytics` | A view A to K with its filters, including compare and monthly series, a filter on a column and a sort | The analytics screens (18) |
| `tiles` | The dashboard tiles of 9.1, each with its second place where one exists | The dashboard (9) |
| `period` | From and to dates for a period word (FR-7.77) | The date shortcuts |
| `calculate` | Sum, difference, average, percent of, change in percent, of figures already read for this question | The money rules (22) |
| `estimate` | An estimate by the methods of FR-7.86 | (new, FR-7.86) |

The model can call several tools in one reply and make up to 6 model calls for one question, plus one rewrite if its answer fails the checks (FR-7.76, NFR-AI-5; 4 until v1.19). The last allowed call has the tools switched off, so the model always answers from what it has read (v1.19). A `find` and the tool that reads the party it found may go in the same reply; the app reads that party only when the find had exactly one match, else it asks "کون سا؟" (v1.19, Umar: fewer round trips). A continuation after "کون سا؟" or the partner code uses what is left of the same question's limits. A tool result names its source and filters, so the app can build the buttons and the source line of FR-7.74. The exact instructions, tool definitions and answer schema the model reads are in `docs/deliverables/D6-system-prompt.md`, which plan 48 copies word for word; a change to them is a requirement change, and NFR-AI-2 runs again.

FR-7.88 [C] **Advice from the books (مشورہ).** (Added 02-11-2026.) Umar: the assistant must be "the expert for the recommendation and future request". When the Owner asks what to do, whom first, or how to do better ("kis se pehle wasooli karun", "akram ko aur udhar dun?", "kis ko pehle paise dun", "commission kaise barhe", "kaun sa maal para hai"), the model reads the sources that bear on it and recommends, like an experienced munshi.
- **From the books first.** Every point that names a figure rests on figures read for this question, cited as references and checked like any answer (FR-7.75, FR-7.76), with its reason ("ان کا بل 36 دن پرانا ہے"). At most 3 points, the most important first.
- **General advice when the books cannot answer** (v1.19, Umar 02-11-2026). When the books have too little data for the question, or the question is general ("sale kaise barhayen", "udhar kaise kam karun"), the assistant gives short, practical mandi advice: fair and steady rates, weekly collection of old dues, a call to buyers past 30 days, sellers paid on time, every sale entered the same day. It is marked "عام مشورہ" (general advice), it names no figure, and it takes one model call. When the books do have the data, the advice comes from the books first.
- **Within the shop's own work.** Each point is an action the Owner can take: collect, call, hold credit, take cash, pay, sell first, check an entry, open a report. Advice never: changes, deletes or back-dates an entry (the assistant still writes nothing, FR-7.72); gives legal, tax, loan or investment advice; names a market rate, a rate to charge or any figure of its own; judges a person's honesty or character; suggests moving anyone to the dead amount; touches a partner's private matters.
- **Unasked, one point.** When an answer's own figures plainly call for action (a buyer over his credit limit, a bill older than 90 days, goods waiting more than 2 days, cash below zero), the model may add one advice point [A].
- **How it shows.** The band "مشورہ، فیصلہ آپ کا" (Advice: the decision is yours), from the Fluent files, above the answer; the outcome in the log is advice (FR-7.84). Advice that rests on an estimate also carries the estimate band (FR-7.86). Advice is never saved and never appears in a report, tile or print.
- **Which sources for which question** (the playbook) is part of the fixed instructions (`D6-system-prompt.md` Section 2).

Example (E2E-63): today 15-11-26, nothing entered after Events 1 to 5. "kis se pehle wasooli karun". The model reads R20 sorted by the days of the oldest open bill, then by balance: ارشد ملتانی 18,684 بنام ✓ and اکرم ٹول ٹیکس 2,880 بنام ✓, both bills 36 days old (10-10-26 to 15-11-26: 21 days to the end of October, then 15). Both bills are equally old, so the larger comes first. Under the band: "پہلے ارشد ملتانی سے وصولی کریں: 18,684 بنام ✓، ان کا بل 36 دن پرانا ہے۔ • پھر اکرم ٹول ٹیکس سے: 2,880 بنام ✓، بل 36 دن پرانا۔", with the R20 rows and "پوری رپورٹ کھولیں" for R20.

FR-7.89 [C] **Help with the app itself (رہنمائی).** (Added in v1.18, Umar 02-11-2026: "anything can be asked in any way ... learning ways how to use that desktop app".) The Owner can ask how to use the app, or what a screen or a word means, in any wording. Examples: "bikri kaise add karun", "market safar kya hai", "wasooli kaise likhun", "partner ka hisaab kahan dekhun".
- The model reads the app's own guide through the tool `app_help`. There is one topic for every screen and module, and each topic gives what it is for, when to use it, the steps on screen, the keys and one worked example in the PRD's words (`D6-system-prompt.md`). The answer explains in plain steps from that guide, never from guesswork.
- A help answer can carry a button to open that screen. The screen opens empty; the assistant never fills or saves an entry (FR-7.79).
- **Answer length follows the question.** A question about figures ("faisal ka khata") gets a short, direct answer. Advice (FR-7.88) and help may explain in a few short points. Both stay inside the 130-word cap.
Example: "is main market safar kya hai?" gets three short points: a trip is goods bought by some partners in another market; the shop sells them like any lot and keeps its commission; the profit or loss goes only to the trip's partners. Then a button opens مارکیٹ سفر.

---

## 8. Module: Login and security

### 8.1 Login screen

| Field | Urdu label | Type | Rules | Source |
|---|---|---|---|---|
| User name | صارف کا نام | Text | Required | [C][S] |
| Password | پاس ورڈ | Password | Required, masked | [C][S] |
| Show password | پاس ورڈ دکھائیں | Eye icon | | [A] |
| Login | داخل ہوں | Button | `Enter` also logs in | [C][S] |
| Exit | باہر | Button | Closes the app | [S] |
| Language | اردو \| English | Switch | Last language used on this computer (FR-7.30) | [C] |

Display only: firm name in Urdu and English, mandi name, app version.

FR-8.1 [C] Username and password are checked. Passwords are never stored in readable form.
FR-8.2 [A] Wrong passwords: after 5 wrong tries, wait 1 minute; after 5 more, wait 5 minutes; after that, 15 minutes each time.
FR-8.3 [A] The old "Session Start / Session End" dates are not used to block login. Report date ranges replace them.
FR-8.4 [A] Auto-lock after 15 minutes idle (setting). Unlock needs the password; unsaved work stays.
FR-8.5 [C] A recovery code is shown once and printed at first setup. With it the Owner sets a new password if he forgets his. There is no other way in (NFR-SEC-12).
FR-8.6 [A] Every login, failed login and logout is written to the audit log.

---

## 9. Module: Dashboard (ڈیش بورڈ)

### 9.1 Tiles (today, live)

| Tile | Urdu | Calculation |
|---|---|---|
| Today's sales | آج کی بکری | Sum of خام بکری of today's postings, and count of بکری |
| Today's commission | آج کا کمیشن | Seller + buyer commission + لاگا, today |
| Cash in hand | موجود روکڑ | Balance of the cash account now |
| Buyers' dues | گاہکوں کے ذمے | Sum of buyer debit balances, leaving out چکتا کھاتہ 0030 (FR-13.9) and anyone in the dead amount (FR-10.10). Always equals the میزان of R20 (FR-19.9). |
| Sellers' dues | بیوپاریوں کے ذمے | What sellers owe the shop beyond their open advances (the "بیوپاری کے ذمے" list of R24, FR-15.9), leaving out anyone in the dead amount [C] (Umar, 30-10-2026) |
| Total dues | کل اگراہی | Buyers' dues + sellers' dues + dead amount: everything people owe the shop. Open advances and money lent have their own tiles [C] |
| Dead amount | ڈوبی رقم | What people in the dead amount still owe, and how many they are (FR-10.10). Opens R38 [C] |
| Sellers payable | بیوپاریوں کو ادائیگی باقی | Sum of each seller's open payable (FR-7.59), leaving out anyone in the dead amount. Open advances are not taken off it; they show in پیشگی باقی. |
| Advances open | پیشگی باقی | Sum of advances not yet adjusted (FR-15.8), leaving out anyone in the dead amount |
| Goods not sold | باقی مال | Count of نگ in stock and number of lots |
| Market trips | مارکیٹ سفر | Open trips: نگ of their goods not yet sold, and their cost left (FR-12.25) |
| Gardens | باغات | Open gardens and each one's running balance (FR-12.37) |
| Money lent open | ادھار رقم باقی | Sum of open ادھار رقم, and how much is past its return-by date (FR-16.4), leaving out anyone in the dead amount |
| Today's expenses | آج کا خرچہ | دکان خرچہ today (0501 heads only) |
| Last backup | آخری بیک اپ | Green under 24 hours, red above |

FR-9.1 [A] Every tile is clickable and opens the screen or report behind it.

### 9.2 Partner strip [C]

One card per partner: today's share, this month's share, this year's share (from the partner share ledger, income shares minus expense shares), and his کل رقم (C22). Click opens the partner statement (R31).

### 9.3 Lists and quick actions

- Today's بکری: number, seller, خام بکری, صافی بکری, status.
- Top 10 oldest buyer dues: name, city, amount, days, leaving out anyone in the dead amount.
- Quick action buttons: نئی بکری, وصولی, ادائیگی, پیشگی, خرچہ, روزنامچہ.
- آج کے کام: the attention list (FR-7.44), each item one click from its screen.

### 9.4 Mini charts [A]

- Last 30 days: خام بکری and commission.
- This month's commission by top 5 items.

---

## 10. Module: انڈکس (Masters)

### 10.1 Chart of accounts: three levels [S]

```
Head (مد)            01 Assets | 02 Payables | 03 Parties | 04 Income | 05 Expenses | 06 Capital
  Control (کنٹرول)    0101 Cash, 0103 Money Lent, 0201 Payables, 03011 Sellers, 0304 Buyers, 0305 Separate Khatas, 0306 Gardens, 0401 Income ...
    Account (کھاتہ)    0010 رحیم
```

FR-10.1 [S] Screens: Chart of Account Head, Chart of Account Control, Chart of Account Tree (all three levels with live balances, expand and collapse with arrow keys).

FR-10.2 [C] **Codes by type** (Umar, 27-10-2026). Each kind of record has a type number. Its code is the type number followed by a running number that starts at 0, with no padding. Sellers and buyers still sit under their controls (03011, 0304) in the chart of accounts; only their codes change.

| Kind | Type number | Codes | Source |
|---|---|---|---|
| Seller (بیوپاری / زمیندار / بزگر) | **001** | 0010, 0011 ... 0019, 00110, 00111 | [C] |
| Buyer (گاہک) | **002** | 0020, 0021 ... 00210 | [C] |
| چکتا کھاتہ | **003** | 0030: the one system account of چکتا buyers (Section 13). 0031 is an old closed account (FR-13.9). No other code is made in this series (v1.32) | [C] |
| Item (قسم مال) | **004** | 0040, 0041 ... | [C] |
| Packing (پیکنگ) | **005** | 0050, 0051 ... | [C] |
| Staff (عملہ) | **006** | 0060, 0061 ... | [C] |
| Market trip (مارکیٹ سفر) | **007** | 0070, 0071 ... (the trip's number) | [C] |
| Garden (باغ) | **008** | 0080, 0081 ... (the garden's number and its account's code) | [C] |
| Others | keep their control code + 4-digit serial | 01010001 cash, 04010001 seller commission, 03040000 نقدی منڈی, 030119998 اپنا مال, money lent 0103xxxx, separate khatas 0305xxxx, gardens 0306xxxx; cities, addas, marks, stores, narrations and expense heads keep their codes | [A] |

FR-10.3 [C] **Running numbers.** The next code suggested is the highest running number of that type + 1. The user can type a different free code of the same type. There is no limit in practice (a code has at most 10 digits, so 9,999,999 of each type), and a code is never reused (Umar, 27-10-2026). Example: the first seller is 0010, the eleventh 00110.

FR-10.4 [A] Seeded at install (system accounts, the rows below with a fixed code: cannot be deleted or made inactive; codes cannot be changed. Umar, 15-10-2026: a posting refuses an inactive account, so an inactive روکڑ would stop every voucher. The per-person rows, such as 0103xxxx and 0305xxxx, are normal accounts):

| Code | Account | English name | Kind | Partner-shared |
|---|---|---|---|---|
| 01010001 | روکڑ (the shop's one cash account: all its money, in the drawer or in the bank, v1.6) | Cash | Asset | |
| 01020001 | بینک (from the first migration; not used from v1.6: never offered, nothing posts to it) | Bank | Asset | |
| 0103xxxx | One ادھار رقم account per person (staff or anyone else) | Money Lent, <name> | Asset (money to come back) | No |
| 02010001 | مزدوری کھاتہ | Labour Payable | Payable to labour | No |
| 02010002 | کرایہ کھاتہ | Freight Payable | Payable to transporters | No |
| 02010003 | کرایہ سٹور کھاتہ | Store Rent Payable | Payable to store owners | No |
| 02010004 | منشیانہ کھاتہ | Munshiana Payable | Payable to the munshi | No |
| 0031 | پرانا بند کھاتہ (an old closed account, FR-13.9) | Old Closed Account | Seller. Inactive from v1.32, the one inactive system account; nothing posts to it | |
| 030119998 | اپنا مال (the seller of a trip's goods) | Own Goods (Trips) | Seller, used only as the seller of a trip's lots and for their charge rules. It never holds a balance (FR-12.26). | |
| 0305xxxx | One account per الگ کھاتہ (family or any other) | Separate Khata, <name> | Party, outside the partners (16.6) | No |
| 0306xxxx | One account per garden (12.11), made with the garden | Garden, <name> | Party: holds the garden's sales money until it is closed | No: its money goes only to the garden's partners (FR-12.38) |
| 03040000 | نقدی منڈی (cash buyer with no name) | Naqdi Mandi (Cash Buyer) | Buyer | |
| 0030 | چکتا کھاتہ | Chukta Khata | Buyer. Holds no balance after any sale (FR-13.1) | |
| 04010001 | کمیشن آمدن (بیوپاری) | Commission Income (Seller) | Income | Yes |
| 04010002 | کمیشن آمدن (گاہک) | Commission Income (Buyer) | Income | Yes |
| 04010003 | لاگا آمدن | Laga Income | Income | Yes |
| 04010004 | فون آمدن | Phone Charge Income | Income | Yes |
| 04010005 | ریٹ فرق | Rate Difference | Income (can be negative) | Yes |
| 0501xxxx | دکان خرچہ heads (بجلی، کرایہ دکان، چائے، مہمان نوازی، تنخواہ ...) | Shop expense heads (Electricity, Shop Rent, Tea, Guests' Food and Drinks, Salaries ...) | Expense | Yes, always split |
| 06010001 | ابتدائی سرمایہ | Opening Capital | Capital | |
| 06019999 | ابتدائی فرق (opening difference) | Opening Difference | Capital | |
| 0602xxxx | One capital account per partner | Capital, <partner name> | Capital | |
| 0603xxxx | One drawings account (ذاتی خرچہ) per partner | Drawings, <partner name> | Capital | |
| 0604xxxx | One garden, trip and fruit account per partner (v1.6) | Gardens, Trips and Fruit, <partner name> | Capital | No: only his own amounts (FR-12.30, FR-16.10) |

Seller and buyer commission go to two income accounts, so analytics and the profit report can show each side. Heads and controls also have English names: Assets, Payables, Parties, Income, Expenses, Capital; Cash, Bank (not used), Money Lent, Payables, Sellers, Buyers, Separate Khatas, Gardens, Income, Shop Expenses, Capital, Drawings, Gardens Trips and Fruit. All seeded English names follow the glossary (FR-7.37).

### 10.2 Account (انڈکس) form

| Field | Urdu label | Type | Req | Rules | Source |
|---|---|---|---|---|---|
| Account type | قسم کھاتہ | Select: بیوپاری، گاہک (v1.8: a person for ادھار رقم and a الگ کھاتہ are added on their own screens with `F4`) | Yes | Sets the control and code | [C] |
| Head (display) | مد | Read-only | | Shown for the chosen control | [S] |
| Control | کنٹرول | Select | Yes | From type, can change for دیگر | [S] |
| Account code | کوڈ | Text | Yes | Auto (FR-10.3), unique | [S][C] |
| Urdu name | اردو نام | Text | Yes | Type in Urdu or English letters (FR-7.22); Urdu letters only (FR-7.28) | [S] |
| English name | انگریزی نام | Text | Yes | Filled by itself from the Urdu name, editable (FR-7.24); English letters only | [S] field, required [C] |
| Tag | نشان | Text | No | 1 to 4 characters: English capital letters and dots (K, BA, GM, D.P). Shown after the name in both languages and on every print, for example "عاشق ظفر K" and "Ashiq Zafar K". Stored once, as typed (FR-7.25) [C] (Umar, 20-10-2026) | [C] |
| Father / firm name | ولدیت / فرم | Text, Urdu and English (FR-7.24) | No | Tells apart two people with the same name | [A] hidden |
| City | شہر | Select | No | From city master | [S] |
| اڈا | اڈا | Select | No | Sellers usually come from one اڈا | [A] hidden |
| Phone 1 | فون | Text | No | Pakistani mobile check | [A] |
| Phone 2 | فون 2 | Text | No | | [A] |
| Address | پتہ | Text | No | | [A] |
| Linked account | منسلک کھاتہ | Account picker | No | Same person as both seller and buyer: links his two accounts so a combined balance can be shown | [A] hidden |
| Opening date | ابتدائی تاریخ | Date | Yes | Default today; not after today. A new account's opening may be on a closed day; a changed opening needs an open date (FR-10.11, v1.22) | [S] |
| Opening debit | ابتدائی بنام | Money | No | Only one of debit or credit | [S] |
| Opening credit | ابتدائی جمع | Money | No | | [S] |
| Opening is an advance | ابتدائی پیشگی | Checkbox | No | Sellers only, with an opening debit: the opening debit counts as an advance (FR-15.8) | [A] hidden |
| Credit allowed | ادھار کی اجازت | Checkbox | No | Ticked by default (as in the old account form). Off = cash only | [S] |
| Credit limit | ادھار حد | Money | No | Warning on بکری when exceeded | [A] |
| Commission rules | کمیشن اصول | Link | No | Opens this party's rules (Section 11) | [C] |
| Default مارکہ | مارکہ | Select | No | Sellers | [A] |
| Collector | اگراہی والا | Select (staff) | No | Who collects from this buyer | [A] hidden |
| Active | فعال | Checkbox | Yes | Inactive = hidden from pickers, kept in reports. A person in the dead amount still shows, marked ڈوبی, in the وصولی and ادھار رقم واپسی pickers (FR-10.10) | [A] |
| Note | نوٹ | Text | No | | [A] |
| System fields | | Auto | | Created by and at, updated by and at | [A] |

FR-10.5 [S] List screen columns: code, Urdu name, English name, city, opening date, opening debit, opening credit, credit allowed, current balance. Footer: total debit, total credit. [C] The list shows only sellers and buyers, with a type filter: all, بیوپاری, گاہک (Umar, 27-10-2026; v1.32). چکتا buyers are not accounts and are not listed here. روکڑ, the payables, income, capital and every other account are seen in the chart of accounts (FR-10.1).
FR-10.6 [A] An account with postings cannot be deleted, only made inactive.
FR-10.7 [A] Duplicate warning when name, tag and phone match an existing account. اسلم BA and اسلم are two different people. Similar names show while typing (FR-7.63).
FR-10.8 [C] Quick add from any picker (FR-7.69) opens a short form without leaving the current screen. The full انڈکس form is still used to edit a record later.
FR-10.9 [A] **Opening balances post.** An opening debit or credit posts as an opening entry dated the opening date: the account on its side, and ابتدائی فرق (06019999) on the other side. When all openings are entered correctly, ابتدائی فرق is 0. The trial balance shows it, so a wrong opening is visible. Changing an opening reverses the old opening posting and posts the new one.
- [C] (Umar, 15-10-2026) Each account has at most one opening balance: one debit or one credit. Changing it reverses that account's opening posting and posts the new one, as above.
- [C] An opening is refused on income (head 04) and expense (head 05) accounts, ابتدائی فرق, نقدی منڈی, the accounts 0030 and 0031 and اپنا مال (030119998, which never holds a balance, FR-12.26). From v1.6 it is also refused on بینک (01020001), a garden's account (0306) and a partner's garden, trip and fruit account (0604) [A].
- [C] A partner's drawings account (0603) can take an opening debit. It counts as money he took (C20, C22), dated the opening date. Example: the app starts in mid-year and partner الف has already taken 2,000 this year.
FR-10.10 [C] **Dead amount (ڈوبی رقم).** (Added in v1.10.) Umar (30-10-2026): a person who owes the shop may die or leave the business. The shop still tries to recover the money, and his account is made inactive. What he still owes stays in the shop's total اگراہی (کل اگراہی), but it is shown on its own as ڈوبی رقم, because it may never be collected.
- **Who.** Anyone who owes the shop: a buyer, a seller whose ledger balance is بنام, or a person with ادھار رقم (control 0103). Partners, الگ کھاتے, gardens, چکتا buyers and system accounts are never moved.
- **How.** On the account form, the party card, or a row of R20, R24 or R32, the button "ڈوبی رقم میں ڈالیں" (Move to dead amount) shows the balance owed and asks for the date (today by default; an open date, and not before his last posting), the reason (فوت ہو گئے died, کاروبار چھوڑ دیا left the business, دیگر other, which needs a note) and a note. Saving makes the account inactive (FR-10.6) and records the move with the balance on that date. An account that owes nothing cannot be moved (`ERR_NOTHING_OWED`); an account is simply made inactive. A linked account (E10) is not moved by itself; the screen shows its balance [A].
- **Money.** Nothing posts. The balance stays on his account, so the trial balance, the partner split and every partner's کل رقم do not change (A65). The dead amount is his whole ledger بنام balance: for a seller, his open advance and open payables are netted into it and leave پیشگی باقی and بیوپاریوں کو ادائیگی باقی; for ادھار رقم, his open loans leave ادھار رقم باقی. All of it joins کل اگراہی [A].
- **Recovery.** A وصولی on him (or an ادھار رقم واپسی, for money lent) is still allowed, with its normal matching. The pickers of those two entries show him marked "ڈوبی". A recovery more than he still owes is refused (`ERR_RETURN_EXCEEDS_OPEN`, with the amount still owed). Any other new entry on him is refused, as on any inactive account (`ERR_ACCOUNT_INACTIVE`). His dead amount goes down by what is recovered; at 0 the move stays in his history as "پوری وصول ہو گئی" (fully recovered).
- **Corrections.** An edit, cancel or undo of a posting made before the move is still allowed while its day is open. It changes the dead amount, and R38 shows it as a correction, not as a recovery. An edit or cancel of a recovery changes the recovered amount itself: 4,000 recovered and then cancelled shows recovered 0, and edited to 3,000 shows 3,000. A receipt saved before the move is already inside what he owed then, so its cancel is a correction [A]. An edit of an earlier ادائیگی, پیشگی or جنرل entry on him is refused; it can be cancelled and entered again after he is made active.
- **Undo.** "دوبارہ فعال کریں" (Make active again), with a reason, ends the move and makes the account active. His balance goes back into the normal tiles and lists.
- **Where it shows.** The tiles of 9.1 and report R38. His party card and ledger heading say ڈوبی رقم with the date and reason. He leaves R20 to R22, the بیوپاری کے ذمے list of R24, R26, R32, the attention list (FR-7.44), the dashboard list of oldest dues (9.3) and the collection round (FR-7.61). Every move, recovery and undo is in the audit log (FR-21.3).

Example (E2E-44): after Events 1 to 5, ارشد owes 18,684 and dies on 11-10-26. Buyers' dues 2,880; dead amount 18,684 (1 person); total اگراہی 2,880 + 0 + 18,684 = 21,564, the same as before. His son pays 4,000 on 12-10-26: dead amount 14,684, total اگراہی 17,564, cash 58,484 + 4,000 = 62,484, trial balance still 85,048 = 85,048 (روکڑ 62,484, ارشد 14,684).

FR-10.11 [C] **Add or change an account on any day.** (Added in v1.13.) Umar (31-10-2026): a new seller or buyer must be addable even when the day is closed. Saving an account, a list record or its details posts nothing, so it never needs an open day. An unchanged opening is never checked again, so changing a phone or a name works on any day. Only a new or changed opening balance posts (FR-10.9). [C] Umar (03-11-2026): a **new** account can be added with its opening balance at any time. Its opening posts on the opening date the user types (today by default, never after today), even when that day is closed. This is the one posting allowed on a closed day (FR-6.6). It is safe for the books, because its other side is ابتدائی فرق (06019999), so every closed day still balances. The audit log records it. Changing or removing the opening of an account that already exists still needs an open date. When the date is closed, the form offers to reopen the day (FR-21.4) [A]. Example: the days up to 03-11-26 are closed. At 9 pm a new buyer طارق is added with an opening of 5,000 بنام. He saves at once, and his 5,000 posts on 03-11-26 against ابتدائی فرق.

### 10.3 Other masters

All have add, edit and inactive, and cannot be deleted once used. Every name is entered once and filled in the other script by itself (FR-7.24). From v1.6 there is no bank master: all money is روکڑ (Umar, 19-10-2026).

| Master | Urdu | Fields | Source |
|---|---|---|---|
| City | شہر | Code, Urdu name, English name | [S] |
| Item | قسم مال | Code, Urdu name, English name, item group (پھل / سبزی), default packing, sold by (نگ / وزن), active | [S] fields [A] |
| Packing | پیکنگ | Code, Urdu name, English name, approx weight kg | [S] fields [A] |
| Truck stand | اڈا | Code, Urdu name, English name, city | [S] |
| Mark | مارکہ | Code, mark text (as typed, FR-7.25), owner seller | [S] |
| Store | سٹور | Code, Urdu name, English name | [S] |
| Standard narration | تفصیل | Urdu text, English text, used in (voucher types) | [S] |
| Expense head | خرچہ مد | Code, Urdu name, English name (always a دکان خرچہ account under 0501) | [A] |
| Staff | عملہ | Urdu name, English name, job (منشی، اگراہی والا، مزدور), phone, active (فعال / غیر فعال, a switch on each row and in the form), linked ادھار رقم account (made by itself the first time he is lent money, FR-16.4). An active munshi prints in every header; an inactive one does not (Section 21, v1.23). Example: آصف leaves, so he is made inactive and drops off every print; the new munshi بلال is added as active and prints from the next print. | [A] |

---

## 11. Module: Commission rules (کمیشن اصول) [C]

Umar: commission changes depending on the item, the seller or the buyer.

### 11.1 Rule fields

| Field | Urdu | Type | Req | Notes |
|---|---|---|---|---|
| Side | جانب | Select: بیوپاری (seller), گاہک (buyer) | Yes | Seller side is deducted from the seller. Buyer side is added to the buyer's bill. |
| Charge type | قسم | Select (allowed list in 11.2) | Yes | |
| Party | کھاتہ | Account picker | No | Blank = any party |
| Item | قسم مال | Picker | No | Blank = any item |
| Item group | گروپ | Select: پھل، سبزی | No | Blank = any group. Only when Item and Party are both blank: there is no party + group level (11.3). |
| Method | طریقہ | Select (allowed list in 11.2) | Yes | |
| Value | شرح | Number | Yes | Percent with up to 2 decimals (6 or 2.5), or whole rupees per نگ or per lot |
| From date | تاریخ سے | Date | Yes | |
| To date | تاریخ تک | Date | No | Blank = no end |
| Active | فعال | Checkbox | Yes | |
| Note | نوٹ | Text | No | |

### 11.2 Allowed charges

Only these combinations exist. The rule screen does not offer any other.

[C] Umar (21-10-2026): every buyer's bill carries two charges on each line, commission and لاگا. The seller is charged commission only, never لاگا; his other deductions are the lot charges below (labour, munshiana, phone), which are not commission. The rate of each charge comes from the rules (11.3), and can differ by item: the client's slips of 18-10-26 show, for example, 50 per نگ on سیب and about 5% plus 3 per نگ on کیلا. These rates are set as rules at setup; nothing in the code changes.

| Side | Charge | Methods allowed | Applied to | Posts to |
|---|---|---|---|---|
| بیوپاری | کمیشن | فیصد, فی نگ, فی لاٹ | Each sale line (فی لاٹ: once per lot, see FR-11.9) | کمیشن آمدن (بیوپاری) |
| بیوپاری | مزدوری | فی نگ, فی لاٹ | The lot, on qty received | مزدوری کھاتہ |
| بیوپاری | منشیانہ | فی نگ, فی لاٹ | The lot, on qty received | منشیانہ کھاتہ |
| بیوپاری | فون | فی نگ, فی لاٹ | The lot, on qty received | فون آمدن |
| گاہک | کمیشن | فیصد, فی نگ | Each sale line | کمیشن آمدن (گاہک) |
| گاہک | لاگا | فی نگ, فیصد | Each sale line | لاگا آمدن |

### 11.3 Which rule wins (most specific first)

| Priority | Rule matches on | Example |
|---|---|---|
| 1 | Typed on the line (only commission and لاگا; allowed by default, FR-11.10) | Munshi types 4% for this one line |
| 2 | Party + item | رحیم selling کیلا: 5% |
| 3 | Party only | رحیم, any item: 4% |
| 4 | Item only | جاپانی پھل, any seller: 30 per نگ |
| 5 | Item group (پھل or سبزی) | All fruit: لاگا 8 per نگ |
| 6 | Global default for that side and charge | Seller 6%, buyer 6%; لاگا 0; مزدوری, منشیانہ, فون 0 (no default). For a lot whose seller is a الگ کھاتہ, every seller-side default is 0 (FR-16.7). A trip's lot (اپنا مال) and a garden's lot use the normal defaults. |

The rule date used is the date of the posting the line belongs to. The party is the seller for seller-side charges and the buyer for buyer-side charges.

Worked example with these rules, each on 20 نگ × 600 = 12,000:

| Sale | Which rule | Commission |
|---|---|---|
| رحیم sells کیلا | Priority 2 (party + item) 5% | 12,000 × 5 ÷ 100 = 600 |
| رحیم sells سیب | Priority 3 (party) 4% | 12,000 × 4 ÷ 100 = 480 |
| کریم sells جاپانی پھل | Priority 4 (item) 30 per نگ | 20 × 30 = 600 |
| کریم sells کیلا | Priority 6 (default) 6% | 12,000 × 6 ÷ 100 = 720 |

### 11.4 Requirements

FR-11.1 [C] Seller-side and buyer-side commission are worked out per sale line using the winning rule.
FR-11.2 [C] Methods supported: percent of line value, fixed amount per نگ, fixed amount per lot (seller side only).
FR-11.3 [A] The same rule system works out لاگا, مزدوری, منشیانہ and فون (Section 11.2).
FR-11.4 [A] Each sale line shows a small tag with the rule used (for example "رحیم + کیلا 5%"). Pointing at it shows the full rule.
FR-11.5 [C] If two rules at the same priority overlap in dates, the newer one wins and the rule screen warns about the overlap. "Newer" means the rule with the later start date; on the same start date, the one saved last (Umar, 15-10-2026). Example: "7% from 01-11" is saved first, then "5% from 01-10": October is 5% and November is 7%, so a planned change is never undone.
FR-11.6 [C] The method, value, rule and resulting amount are saved on each sale line and lot (snapshot). Changing a rule later never changes saved results, including when an unchanged line is part of an edit (FR-12.20).
FR-11.7 [C] چکتا کھاتہ follows the same rules as any buyer (6% by default). A rule on the account 0030 changes the rate for all چکتا buyers.
FR-11.8 [A] A "test a rule" box: pick seller, buyer, item, qty and rate, and see which rule wins and the amount, before saving the rule.
FR-11.9 [A] Seller commission "فی لاٹ" is charged once per lot, on the first sale line of the lot's first posting. Later sales of that lot do not charge it again. [C] If an edit removes that line, the charge moves to the first remaining line of that lot in that posting, with its stored method and value (no new rule lookup), so the lot is still charged exactly once (Umar, 15-10-2026). This is the one change FR-12.20 allows on a line the user did not edit. If no line of that lot is left in that posting, the charge goes with the removed line, as before. Example: a per-lot commission of 500 sits on line 1 of 2; line 1 is removed; line 2 now carries 500, and the lot's seller commission includes 500 once.
FR-11.10 [A] Lot charges (مزدوری, منشیانہ, فون) are suggested by the rules but are always editable amounts on the lot. [C] Umar (15-10-2026): the default is 6% for every seller, buyer and item, and the commission agent can change it on the بکری itself. So seller commission, buyer commission and لاگا are editable on each sale line by default (the "allow override" setting starts on; the Owner can turn it off in Section 21). Example: on one line he types 4% for the seller and 5% for the buyer instead of 6%. An edited value is saved with method "دستی" (manual). A rule (11.1) is for a rate that should apply every time, for example رحیم always 5%.

---

## 12. Module: بکری بک (Sales voucher)

### 12.1 What the screen must let the user do

The layout is our own design. Functionally, one screen must let the user:

1. Enter the date (the header holds only the date and the بکری number, v1.34).
2. Add one or more lots (one per seller per item on the truck).
3. For the selected lot, add its sales to buyers.
4. See a live summary for the lot and for the whole بکری: seller side, buyer side, partner split.
5. Save, print, find, move between بکری, and cancel.

FR-12.1 [S] Lots and their sales are linked: selecting a lot shows only its sales. A "پوری بکری" view shows all sales of all lots.
FR-12.2 [C] Everything fits on one screen without scrolling sideways (FR-7.2). Less-used fields sit behind a "مزید" (more) toggle per lot, so the main view stays simple. [C] (Umar, 12-11-2026, v1.34) The header keeps only the date and the بکری number. They sit in the top row of the page, beside the title and the buttons, so there is no header card and its space goes to the lots and lines. Nothing overlaps or scrolls sideways at 1024, 1366 and 1920, in both languages.

### 12.2 Header fields

From v1.34 (plan 71, H1 to H3) a new بکری has only the date and the بکری number. The other fields are retired: an old saved بکری keeps them in the database, they show read-only, and an old print shows them when they are there. Freight, store and seller stay on each lot.

| Field | Urdu | Type | Req | Default / rule | Source |
|---|---|---|---|---|---|
| بکری number | بکری نمبر | Auto | Yes | Given at first save (drafts have no number). Never reused. | [S] |
| Entry date | تاریخ | Date | Yes | Today. Must be an open date (FR-6.6). | [S] |
| Book / series (retired in v1.34) | بک | Read-only | | An old بکری only | [S] meaning [A] |
| Bill date (retired in v1.34) | تاریخ بل | Read-only | | A new بکری's bill date is its entry date; an old بکری keeps its own | [S] field, meaning [A] |
| Truck number (retired in v1.34) | ٹرک نمبر | Read-only | | An old بکری only. Truck memory and the same-truck warning are retired (FR-7.49, FR-7.50) | [S] |
| Zamindar (retired in v1.34) | زمیندار کوڈ + name | Read-only | | An old بکری only. It no longer becomes the first lot's seller | [S] |
| Truck stand (retired in v1.34) | اڈا کوڈ + name | Read-only | | An old بکری only | [S] |
| Driver name (retired in v1.34) | ڈرائیور | Read-only | | An old بکری only | [A] hidden |
| Driver phone (retired in v1.34) | ڈرائیور فون | Read-only | | An old بکری only | [A] hidden |
| Total truck freight (retired in v1.34) | کل کرایہ | Read-only | | An old بکری only. The split over lots (C17) is retired: freight is typed on each lot | [A] hidden |
| Store (retired in v1.34) | سٹور | Read-only | | An old بکری only. It is no longer a default for lots | [S] |
| Note (retired in v1.34) | نوٹ | Read-only | | An old بکری only | [A] |
| Status | حالت | Read-only: محفوظ، منسوخ | | | [A] system |
| System fields | | Auto | | Created/updated by and at, print count | [A] system |

### 12.3 Lot fields

Fields seen [S]: بزگر code, بزگر name, فون, منشیانہ, نقد, کرایہ سٹور, سٹور. The old grid scrolled sideways, so it had more fields than the video shows. We add every input the calculation needs [A].

| Field | Urdu | Type | Req | Default / rule | Source |
|---|---|---|---|---|---|
| Lot no | لاٹ | Auto (1, 2, 3 ...) | Yes | | [A] system |
| Lot name | لاٹ کا نام | Read-only | | Seller, item and counter, for example احسان، آم-2 (FR-12.45) | [C] |
| Seller code | بزگر کوڈ | Seller picker: the sellers; اپنا مال only while a trip has goods left, a garden only while it is open, and the الگ کھاتے that exist [C] (Umar, 01-11-2026, v1.14) | Yes | Previous lot's seller; the first lot starts empty (FR-7.51) | [S] |
| Seller name | بزگر | Read-only | | | [S] |
| Trip purchase | خریداری | Picker of purchases of open trips with stock left | If the seller is اپنا مال | Fills item and packing from the purchase (FR-12.26) | [C] |
| Item | قسم مال | Picker | Yes | Seller's last item (FR-7.51) | [A] hidden |
| Packing | پیکنگ | Picker | Yes | Seller's last packing, else the item's default | [A] hidden |
| Mark | مارکہ | Picker | No | Seller's default, else his last مارکہ | [A] hidden |
| Quantity received | تعداد | Whole number | Yes | More than 0 | [A] hidden |
| Weight received | وزن (کلو) | Number, up to 3 decimals | If the item is sold by weight | | [A] hidden |
| Freight | کرایہ | Money | No | Typed on the lot (v1.34: no header split) | [S] summary |
| Freight paid by | کرایہ ادا | Select: آڑھتی، بیوپاری | Yes | آڑھتی. If بیوپاری (the seller paid the driver himself), freight is shown but not deducted or posted. | [A] hidden |
| Labour | مزدوری | Money | No | Typed. Filled by a rule only if the commission agent made one. Starts at 0. | [S] |
| Munshiana | منشیانہ | Money | No | Typed, or from a rule. Starts at 0. | [S] |
| Phone charge | فون | Money | No | Typed, or from a rule. Starts at 0. | [S] |
| Cash paid now | نقد | Money | No | Cash given to the seller in this posting | [S] |
| Deduct from advance | پیشگی سے کاٹیں | Tick | | Shown only when the seller has an open advance, with the open amount. Ticked by default (FR-12.23). | [C] |
| Store rent | کرایہ سٹور | Money | No | Typed by the munshi | [S] |
| Store | سٹور | Picker | No | Empty (v1.34: no header store) | [S] |
| Seller commission | کمیشن | Read-only | | Sum of the lot's line commissions | [C] |
| Sold qty | فروخت | Read-only | | Sum of this lot's sold qty in all postings | [A] system |
| Written-off qty | ضائع | Read-only | | FR-12.16 | [A] system |
| Balance qty | باقی | Read-only | | Received − sold − written off − taken as fruit (C16). Red if more than 0. | [A] system |
| Lot gross | خام بکری | Read-only | | | [S] |
| Lot net | صافی بکری | Read-only | | | [S] |

FR-12.23 [C] **Advance deduction is a choice per lot.** (Numbered after FR-12.22 because it was added in v1.2.) Umar (11-10-2026): the advance is deducted by default, and the munshi can untick it. Ticked: the seller credit of this lot adjusts his open advances, oldest first (FR-15.8). Unticked: the advance stays open and the whole seller credit is payable, so he can take his money now and repay the advance later. Later sales of the lot follow the same tick; an edit can change it. The tick changes only matching, never the journal. Example E2E-28.

FR-12.45 [C] **Lot names (لاٹ کا نام).** (Added in v1.34.) Umar (12-11-2026, plan 71 L1 to L8):
- Every lot has a name made of its seller, its item and a counter: احسان، آم-1, then احسان، آم-2 for his next mango lot, and احسان، کیلا-1 for his first banana lot. In English: "Ehsan, Mango-1".
- The counter belongs to one seller and one item, and runs for the seller's whole life. It never resets, and a number is never reused, even after a cancel. The first lot is numbered too (-1).
- The number is given when the lot is first saved, like the بکری number. Before the save the screen shows the number the lot will get, worked out by the backend from the saved lots and the lots above it in the same draft.
- An edit that changes a lot's seller or item gives the lot the next number of its new seller and item. The old number stays unused.
- Old lots get numbers by a migration: per seller and item, in order of first posting date, then lot id. No amount, entry or chain hash changes.
- The name is shown wherever a lot is named: the lot labels of the بکری screen (FR-12.6), باقی مال, later sale, edit, R2, R6, R10, R27, the day detail of party ledgers (FR-12.43) and the assistant (FR-7.77).
- Every print of a seller's lots shows them sorted (FR-12.12).

Example (E2E-70): Ehsan's lots in one بکری, typed آم, کیلا, آم, show احسان، آم-1, احسان، کیلا-1, احسان، آم-2, before and after the save. His next بکری's mango lot is احسان، آم-3.

### 12.4 Sale line fields

Fields seen [S]: تاریخ, کوڈ گاہک, گاہک, تعداد بیوپاری, ریٹ بیوپاری, رقم بیوپاری, تعداد گاہک, ریٹ گاہک, کمیشن ریٹ, لاگا ریٹ, رقم گاہک, میزان.

| Field | Urdu | Type | Req | Default / rule | Source |
|---|---|---|---|---|---|
| Line no | نمبر | Auto | Yes | | [A] system |
| Date | تاریخ | Read-only | | The date of the posting this line belongs to | [S] |
| Buyer code | کوڈ گاہک | Buyer picker (includes نقدی منڈی and چکتا buyers, each marked چکتا, FR-7.29) | Yes | Code, short code, Urdu or English name (FR-7.39, FR-7.40); shows the party card (FR-7.38) | [S] |
| Buyer name | گاہک | Read-only | | | [S] |
| Qty (نگ) | تعداد | Whole number | Yes | Starts as the lot's remaining qty (FR-7.52). Shown in both the تعداد بیوپاری and تعداد گاہک columns; they are always equal (A29) | [S] |
| Weight sold | وزن (کلو) | Number, up to 3 decimals | If the item is sold by weight | | [A] hidden |
| Seller rate | ریٹ بیوپاری | Money | Yes | Per نگ, or per kg for weight items. Empty, with a hint of the last rate (FR-7.53) | [S] |
| Seller amount | رقم بیوپاری | Read-only | | C1 | [S] |
| Seller commission | کمیشن بیوپاری | Amount worked out; its rate tag can be edited unless the override setting is off (FR-11.10) | | From rule, per line | [C] |
| Buyer rate | ریٹ گاہک | Money | Yes | Copies seller rate; editable only after "ریٹ مختلف" is switched on | [S] copy [A] |
| Buyer value | قیمت گاہک | Read-only | | C1 with the buyer rate | [A] system |
| Buyer commission rate | کمیشن ریٹ | Rate tag, editable unless the override setting is off (FR-11.10) | | From rule | [S] |
| Buyer commission | کمیشن گاہک | Read-only | | | [S] |
| لاگا rate | لاگا ریٹ | Rate tag, editable unless the override setting is off (FR-11.10) | | From rule | [S] |
| لاگا | لاگا | Read-only | | | [S] |
| Buyer total | رقم گاہک | Read-only | | C5 | [S] |
| Paid on the spot | موقع پر جمع | Money | No | The last money cell of the line. Empty means 0. From 0 up to the line's buyer total (FR-12.44). For a buyer who must pay in full the backend fills it and it cannot be typed in. | [C] |
| Left on account | کھاتے میں باقی | Read-only | | Buyer total − paid (FR-12.44) | [C] |
| Quality | درجہ | Dropdown: blank, اول, دوم, سوم (English: First, Second, Third) | No | Blank by default. A fixed list: no other words. It tells why lines of one lot have different rates; it never changes a figure (FR-12.42, v1.30) | [C] |
| Credit warning | | Icon | | Shows if over credit limit | [A] |
| Note | نوٹ | Text | No | | [A] |

FR-12.3 [A] By default each line has one quantity and one rate, and the buyer rate equals the seller rate. A "ریٹ مختلف" toggle on the line lets the buyer rate differ. The difference (buyer value − seller amount) posts to ریٹ فرق (income, shared). It can be negative (a loss).
FR-12.4 [A] Credit limit exceeded: warning (Owner setting can make it a block). The check uses what is left on the line, not the buyer total (v1.32).
FR-12.5 [C] A چکتا line needs a چکتا buyer with a name. His phone is optional (FR-13.2, v1.32).
FR-12.6 [A] A lot's sold qty can never be more than its received qty. [C] Umar (06-11-2026): a line that would sell more than the lot has left shows the error under its quantity at once, for example "اتنی تعداد دستیاب نہیں: صرف 50 نگ باقی ہیں", and is not worked out: it shows no amount, commission or total, and the lot and بکری totals leave it out until the quantity is fixed. In line order, the line that takes the lot over is the one refused. Every lot label shows its lot name (FR-12.45, v1.34), for example "لاٹ 1: احسان، سیب-1". [C] (D8 bug 1, v1.32) The cursor cannot leave that quantity with `Enter` until it is fixed. A lot sold exactly (200 of 200) is complete, and that is fine. A line with something missing (buyer, qty or rate, once something is typed on it) says in words, in the row, what is missing. A blank row stays silent (FR-7.52).

FR-12.42 [C] **Quality of a sale line (درجہ).** (Added in v1.30.) Umar (06-11-2026): the goods of one lot can differ in quality, and each quality sells at its own rate. The munshi sells them as separate lines of the lot, each with its own quantity and rate, and can mark each line with its quality from a fixed dropdown: اول, دوم, سوم (First, Second, Third), or leave it blank.
- It is optional and never changes money: the line is still qty × rate with its own commission and لاگا (C1 to C5).
- `Enter` does not stop on it (the line stays buyer, qty, rate and paid on the spot, NFR-USE-6). It is reached with the arrow keys or the mouse.
- It prints after the item on every print that shows sale lines, for the buyer and for the seller: the seller bill (R1), the buyer slips, ادھاری تفصیل (R5, "(سیب اول 10 . 1200)"), the party day slip (R35), the ledger with items (R13, R14) and the sales register (R2). A blank quality prints nothing.
- An edit that changes only the quality keeps the line's stored rules and amounts (FR-12.20).
- The assistant reads and writes it: "awwal seb aaj kitne ka bika", "doyam kele ka rate", and a line it shows carries its quality.
Example: Ehsan's 20 پیٹی سیب: اکرم 10 اول @ 1,200 = 12,000; اکرم 6 دوم @ 800 = 4,800; ارشد 4 سوم @ 500 = 2,000. خام بکری 18,800; seller commission 720 + 288 + 120 = 1,128.

FR-12.44 [C] **Money paid on the spot (موقع پر جمع).** (Added in v1.32.) Umar (D8 Feature 4): the toggle ادھاری / نقدی is replaced by a money field at the end of each buyer line: how much the buyer paid right there. He can pay all of it, part of it, or nothing.
- Whole rupees, from 0 up to the line's buyer total. Empty means 0. More than the total is refused (`ERR_PAID_EXCEEDS_TOTAL`, with the most allowed). A negative amount or decimals are refused like any money input.
- A buyer who may not take credit, نقدی منڈی and a چکتا buyer always pay the full amount. The backend sets paid = buyer total. The cell shows it and cannot be typed in.
- What is left (buyer total − paid) stays on his account as normal credit. The backend works it out, never the screen.
- The money comes into روکڑ through the cash part of the journal (FR-12.9).
- It settles that line's own bill first. What is left of the bill is a normal open bill, matched oldest first (FR-15.4).
- The credit limit check uses what is left (FR-12.4).
- `Enter` stops on it, so a line takes four `Enter`s; a cell that cannot be typed in is skipped, so three (NFR-USE-6).
- A line whose only change in an edit is the paid amount keeps its stored rules and amounts (FR-12.20).
- Old saved lines keep their meaning: a cash line counts as paid in full, a credit line as paid 0. No saved number changes.
- Every place that used ادھاری / نقدی follows this field: the live summary, the journal, R3, R4, R5, R13, R14, R34, R35, dues, the dashboard and the assistant.

Example (E2E-65, the lot of D8):

| Buyer | Buyer total | Paid on the spot | Left on account |
|---|---|---|---|
| مانی | 21,360 | 21,360 | 0 |
| انس | 48,100 | 10,000 | 38,100 |
| ہارون | 125,060 | 0 | 125,060 |
| **Total** | **194,520** | **31,360** | **163,160** |

Cash in hand goes up by 31,360. ہارون already had 10,000 جمع, so after the sale he owes 125,060 − 10,000 = 115,060 بنام.

### 12.5 Live summary [S]

For the selected lot, or the whole بکری with "پوری بکری":

| Line | Urdu | Formula |
|---|---|---|
| Gross sale | خام بکری | Sum of seller amounts |
| Freight | کرایہ | Sum of lot freight paid by آڑھتی |
| Commission | کمیشن | Sum of seller commission per line |
| Labour | مزدوری | Sum |
| Munshiana | منشیانہ | Sum |
| Cash paid | نقد | Sum |
| Store rent | کرایہ سٹور | Sum |
| Phone | فون | Sum |
| Total deductions | کل اخراجات | Sum of the 7 lines above |
| Net to seller | صافی بکری | خام بکری − کل اخراجات |
| Buyer commission | گاہک کمیشن | Sum |
| لاگا | لاگا | Sum |
| Rate difference | ریٹ فرق | Sum, shown only if not 0 |
| Buyer bills: left on account / paid on the spot | ادھاری / نقدی | Sum of what is left on each line, and sum of the money paid on the spot. Together they equal the sum of buyer totals |
| Partner split | شراکت داروں کا حصہ | Each partner's share of this posting's shared income |

FR-12.7 [A] If a lot's net is negative (deductions more than gross), the summary shows it in red with "بیوپاری کے ذمے" and the save is allowed. The seller's account then shows a debit.

### 12.6 Saving (first posting of a بکری)

FR-12.8 [C] Save writes, as **one action** (all or nothing): the بکری, lots and lines with their snapshots, one journal entry (sale part + cash part), the partner split, stock moves, analytics facts, bill matching, advance matching, and the audit entry.

Checks before save:

1. At least one lot and one sale line.
2. All required fields filled.
3. Sold qty more than received qty: blocked (FR-12.6).
4. Sold qty less than received qty: warning "5 نگ باقی ہیں". Save is allowed; the rest goes to باقی مال (Section 12.8).
5. بنام = جمع for the whole entry (cannot be bypassed) [C].
6. Partner split total = shared income (cannot be bypassed) [C].
7. Active partner shares total 100% on the posting date (cannot be bypassed) [C].
8. The date is an open date (FR-6.6).
9. Paid on the spot is not more than the line's buyer total (FR-12.44).

FR-12.9 [C] **Journal lines of a بکری posting.** All amounts come from the stored line and lot values.

Sale part:

| Account | Side | Amount |
|---|---|---|
| Buyer of each line (own account, نقدی منڈی, or چکتا کھاتہ) | بنام | Line buyer total |
| Seller of each lot (a garden lot: the garden's account, FR-12.36) | جمع (بنام if negative) | Seller credit, C8 |
| A trip's own lot instead: each trip partner's 0604 account (FR-12.26) | جمع (بنام if negative) | His % of the seller credit, C25 |
| کمیشن آمدن (بیوپاری) | جمع | Σ seller commission |
| کمیشن آمدن (گاہک) | جمع | Σ buyer commission |
| لاگا آمدن | جمع | Σ لاگا |
| فون آمدن | جمع | Σ فون |
| ریٹ فرق | جمع (بنام if negative) | Σ rate difference |
| مزدوری کھاتہ, کرایہ کھاتہ, منشیانہ کھاتہ, کرایہ سٹور کھاتہ | جمع | Their sums (freight only when paid by آڑھتی) |

Cash part:

| Case | بنام | جمع |
|---|---|---|
| Each lot with نقد | Seller: نقد | روکڑ: نقد |
| Each line with paid above 0 | روکڑ: paid | The line's buyer account: paid |

A line paid in full posts exactly what a نقدی line posted before v1.32, and a line paid 0 exactly what an ادھاری line posted. Money paid on the spot by a named regular buyer still appears on his ledger (bought and paid the same day), and on his buyer slip.

FR-12.10 [A] Auto-save draft every 5 seconds while typing. After a crash or power cut, the app offers to reopen it.
FR-12.11 [A] Cancel (منسوخ): see Section 12.9. Reason required.
FR-12.12 [S] Print: seller bill, buyer slips for this بکری, چکتا buyers included. [C] A second button, next to the seller-bill print, prints ادھاری تفصیل (R5): the whole day's buyers by default, with a choice of this بکری only (D8 Feature 8, v1.32). The seller page stays separate. On the Mac these prints follow FR-19.3. [C] (Umar, 12-11-2026, v1.34) Every print of a seller's lots, the seller bill (R1) and the seller's day slip (R35), shows them sorted, whatever order they were typed in: by item (the Urdu order of the item name, the English order in an English print), then by the lot counter (FR-12.45). Example: lots typed آم-3, کیلا-1, آم-2 print آم-2, آم-3, کیلا-1. The backend sorts; the screen draws in the order given.
FR-12.13 [S] Move between saved بکری: first, previous, next, last, and find by number, date, seller, buyer or item (v1.32), and by truck for an old بکری that has one (v1.34).

### 12.7 Today's بکری list (آج کی بکری)

Columns: بکری no, time, seller(s), lot names (FR-12.45), نگ, خام بکری, صافی بکری, buyer total, status. Filters: date, seller, item, status. `Enter` or click opens it.

### 12.8 Goods not yet sold (باقی مال) [S + A]

The old reports menu has a stock report ("مکمل سٹاک") and an Inventory Reports menu. In a commission business, "stock" means goods received but not yet sold.

FR-12.14 [A] The باقی مال screen lists every lot with balance qty more than 0: date received, بکری no, lot name (seller, item and counter, FR-12.45), packing, received, sold, written off, taken as fruit, balance, days waiting.
FR-12.15 [A] "فروخت کریں" on a row opens a **later sale** for that lot (Section 12.9).
FR-12.16 [A] A lot balance can be written off with a reason (خراب، واپس). This records a stock move only. It has no money effect unless a جنرل entry is also made. Trip goods are written off on their purchase, also with no posting (FR-12.28). A write-off can be undone the same way as an edit.

### 12.9 Postings of a بکری: first save, later sales, edits, cancel

A بکری can have more than one posting over time. Each posting is a posting event (Section 6.1) with its own date and journal entry.

**Later sale** (selling leftover stock on a later day):

FR-12.17 [A] A later sale adds new sale lines to an existing lot. Its date is the day it happens (an open date). It posts only the new lines:
- Buyer side and seller commission for the new lines, by the rules on the later sale's date.
- Seller credit = Σ new seller amounts − Σ new seller commission.
- Lot charges (freight, مزدوری, منشیانہ, فون, کرایہ سٹور, per-lot commission) are **not** charged again. They belong to the first posting.
- Partner split by shares on the later sale's date.
- The original posting and its day are not touched, even if that day is closed.

**Edit:**

FR-12.18 [A] Any one posting of a بکری can be edited while that posting's date is open. Editing a posting in a closed day needs the day reopened first; the Edit button does it in one step (FR-21.4). The posting's date never changes (FR-7.70).
FR-12.19 [A] An edit is saved as a reversal of that posting (same date) plus a new posting (same date), in one action.
FR-12.20 [A] In an edit, a line or lot charge the user did not change keeps its stored method, value and amount exactly (FR-11.6). [C] Umar (10-11-2026): an edit that changes only the buyer of a line looks up the buyer-side rules again (buyer commission and لاگا) and keeps the seller side as stored; an edit that changes only the seller of a lot looks up the seller-side rules again and keeps the buyer side and the money paid on the spot as stored, on every line of that lot. Example: the buyer default goes to 7% after the save and only the seller is corrected: اکرم stays 12,880. The only exception is a per-lot commission moved from a removed line (FR-11.9). Only changed or new lines look up the rules again (by the posting date). When only a lot's received qty is corrected, its per نگ lot charges keep their stored method and value and their amount is worked out again (C3); a typed amount stays as typed. A line whose only change is its quality or its paid on the spot keeps its stored rules and amounts (v1.32).
FR-12.21 [A] The partner split of the edited posting is recalculated from the new shared income, with the shares of the posting date.

**Cancel:**

FR-12.22 [A] Cancelling a بکری reverses every posting of that بکری. All their dates must be open; otherwise the Owner reopens the days first. The بکری is marked منسوخ. Receipts that were matched to its lines become unmatched credit on each buyer and are re-matched to that buyer's next oldest bills.

Example (E2E-16 and E2E-16b): on 10-10-26 only 45 of رحیم's 50 کیلا are sold. On 11-10-26 the last 5 are sold to ارشد at 560. The later sale posts 2,800 of sales and 168 of seller commission; freight, labour, munshiana and phone are not charged again.

---

### 12.10 Market trips (مارکیٹ سفر) and their goods (اپنا مال) [C]

Umar (14-10-2026, 18-10-2026 and 19-10-2026): some partners send a person to a bigger market for the day. He buys goods (oranges, bananas or any item), and the goods are sold in our shop or in another shop. The goods belong to the partners in that trip, not to the firm. The shop still takes its normal commission when it sells them; that commission is firm income, shared by all partners as usual. The trip's own profit or loss goes only to the trip's partners, by their %, on the day the goods are sold. Trips have their own section in the main bar (FR-5.2), and every purchase belongs to a trip.

| Field | Urdu | Type | Req | Rule |
|---|---|---|---|---|
| Trip no | سفر نمبر | Auto at save | Yes | Code by type 007 (FR-10.2): the first trip is 0070, then 0071 ... (v1.8; before: سفر-1, سفر-2) |
| Date | تاریخ | Date | Yes | An open date |
| Market | منڈی | Text | No | Where the goods are bought, as typed (FR-7.25) |
| Sent | بھیجا گیا | Text | No | Who went, as typed |
| Partners and shares | شراکت دار اور حصہ | Per active partner: % (up to 2 decimals) and the money source (دکان سے or اپنی جیب سے) | Yes | FR-12.29. Filled from the last trip; the first trip from the firm's shares [A] |
| Note | نوٹ | Text | No | |

FR-12.29 [C] **Shares and where the money comes from.** (FR-12.29 to FR-12.38 were added in v1.6.) A trip and a garden (12.11) are made with the same questions, in this order:
1. **Shares.** Which partners are in it, and each one's %. The % of the partners in it total exactly 100%. A partner who is not in it has 0%; he still gets his normal share of the shop's commission on its goods.
2. **Money source,** for each partner in it: دکان سے (from the shop's cash) or اپنی جیب سے (from his own pocket).
3. **The details** (the fields above, or the garden's fields in 12.11).

The shares cannot change after the first entry [A]. Each partner's source is filled on every entry and can be changed on that entry [A].

FR-12.30 [C] **Money paid in.** Every payment of a trip or a garden (a purchase, a trip cost, a garden installment, a garden running cost) is split between its partners by its % (C23).
- A part from the shop's cash posts: that partner's garden, trip and fruit account (0604) بنام, روکڑ جمع. It comes off his کل رقم at once (C22). If it would take روکڑ below 0, the save asks first (FR-6.7).
- A part from his own pocket posts nothing. The trip or garden records it, and his کل رقم does not change.
- A payment where every part is from a pocket posts no journal entry.

Example: a purchase of 96,000 with الف 50% from the shop and ب 50% from his pocket: الف's 0604 بنام 48,000, روکڑ جمع 48,000; ب's 48,000 is recorded on the trip only.

| Field | Urdu | Type | Req | Rule |
|---|---|---|---|---|
| Purchase no | خریداری نمبر | Auto at save | Yes | Own series (خ-1, خ-2 ...) |
| Trip | سفر | Picker of open trips | Yes | Every purchase belongs to a trip |
| Date | تاریخ | Date | Yes | An open date, not before the trip's date |
| Supplier | فروخت کنندہ | Text | No | Who the goods were bought from, as typed. Nothing is owed to him: a purchase is paid at once [A] |
| Item, packing | قسم مال، پیکنگ | Pickers | Yes | |
| Quantity | تعداد | Whole number | Yes | More than 0 |
| Weight | وزن (کلو) | Up to 3 decimals | If sold by weight | |
| Rate | ریٹ | Money | Yes | Per نگ, or per kg for weight items |
| Value | قیمت | Read-only | | C1 |
| Freight, labour, other cost | کرایہ، مزدوری، دیگر خرچ | Money | No | Paid at once with the purchase |
| Paid now | ادا شدہ | Read-only | | Value + freight + labour + other cost, split by the trip's % (FR-12.30) |
| Total cost | کل لاگت | Read-only | | Paid now + its part of the trip costs (C24) |
| Note | نوٹ | Text | No | |

FR-12.24 [C] **Purchase.** Saving records the goods in the trip and posts the paid-now amount as in FR-12.30. Nothing posts to a stock account: the goods are not the firm's. A purchase can be cancelled with a reason only while nothing of it has been sold or written off; the cancel reverses its posting.

FR-12.31 [C] **Trip costs.** A cost of the whole trip (truck, food) is entered on the trip with its amount and reason. It is paid in as in FR-12.30, and spread over the trip's purchases by نگ bought (C24), so each purchase's total cost includes its part. It is spread when it is saved, over the trip's purchases that still have goods left, and its part is added to each one's cost left; a trip with no purchase with goods left takes no cost (`ERR_TRIP_NO_STOCK`) [A]. A trip cost, or a purchase that carries part of one, can be cancelled only while nothing of the purchases it touched has been sold or written off [A]. A closed trip takes no new purchase or cost. Example: truck 4,000 on a trip whose only purchase is 80 نگ for 96,000: total cost 100,000, 1,250 per نگ.

FR-12.25 [C] **Trip stock.** Each purchase is stock of its trip until it is sold or written off: bought, sold, written off, left (نگ), cost left, days waiting. It shows on the trip, and in باقی مال marked اپنا مال with its trip. The dashboard tile مارکیٹ سفر shows the open trips' نگ left and cost left.

FR-12.26 [C] **Selling trip goods in بکری.** In بکری, a lot whose seller is اپنا مال takes its goods from one purchase with stock left.
- Its received qty is taken from the purchase at save and must equal the qty sold in that lot (`ERR_OWN_LOT_NOT_FULLY_SOLD`). An own lot never has a balance in باقی مال and never has a later sale; goods not sold stay in the purchase and are sold in another بکری, sold in another shop or written off. Selling more than the purchase's stock left is refused (`ERR_QTY_EXCEEDS_LOT`).
- On an own lot, نقد and the پیشگی سے کاٹیں tick are hidden and stay 0, and freight starts at 0 with a warning if typed (the purchase's costs are already in its cost).
- اپنا مال is offered only in the lot seller field, and only while a trip has a purchase with goods left (v1.14); every other picker leaves it out, so its account never holds a balance.
- Sale lines, the buyer side, لاگا and the live summary work like any lot. Seller-side commission and lot charges are charged as for any seller, by the rules on the اپنا مال account (default 6%) [C]. This commission is firm income, split by the firm's shares (FR-6.5).
- The seller credit (C8) goes, in the same entry, to the trip's partners: each one's 0604 account جمع his % of it (C25). It adds to his کل رقم that day.
- The cost of the goods sold (C21) is stored on the line and shown with the profit or loss. It is not posted.
- In an edit, an unchanged own line keeps its stored cost (FR-12.20). A reversal gives back exactly the stored qty and cost to the purchase and reverses the partners' lines. Example E2E-34.

FR-12.32 [C] **Selling trip goods in another shop.** A sale in another shop is entered on the trip: date, shop (as typed), purchase, qty and the money received. The qty comes off the purchase's stock, with its cost (C21). The money posts: روکڑ بنام, and each trip partner's 0604 جمع his % of it (C25) [A: the money comes into روکڑ].

FR-12.27 [C] **Profit or loss.** Per purchase: its sales money after commission (the seller credits, and money from other shops) − cost of goods sold − cost written off. Per day and per trip: the sum over its purchases. It is shown on the purchase, on the trip, in report R33 and in analytics, with each partner's %, his money in and his money back. The shop's commission on the goods (seller commission + buyer commission + لاگا of the sales in our shop) is shown next to it: on each purchase, on the trip, in the trips list (v1.8, Umar 29-10-2026). It can be negative (a loss), shown in red. Each trip partner's کل رقم already holds his part of it: his share of the sales money minus the part he paid from the shop's cash (E2E-34b).

FR-12.41 [C] **Many items in one entry, and the split between our shop and another shop.** (Umar, 29-10-2026, v1.8.) A trip purchase entry and a garden مال آیا entry each hold several item lines (item, packing, qty; for a purchase also rate and its costs), saved together in one action; each line is its own purchase or arrival, posted exactly as one is today. Each line also records its split: the qty for our shop (ہماری دکان) and the qty for another shop (دوسری دکان, with the shop's name if known). It starts as all for our shop and can be changed while goods are left, never below what each part has sold. Only the our-shop part comes into a new sale (FR-12.39, FR-12.40). A sale in another shop (FR-12.32, FR-12.36) picks the line and takes from its other-shop part; its money posts as before. Example: one purchase of مالٹا 80 (our shop 50, another shop 30) and کیلا 40 (all our shop): the next new sale gets مالٹا 50 and کیلا 40; the 30 مالٹا are sold in the other shop with its money. In these item lines `Enter` stops only at item, packing, qty and rate (a purchase) or item, packing and qty (goods arrived), and at weight when the item is sold by weight; freight, labour, other cost, the other-shop qty and the shop name stay on the line, optional, reached by a click or the arrow keys (Umar, 29-10-2026; an exception to FR-7.14).

FR-12.39 [C] **Trip and garden goods come into a new sale.** (Umar, 27-10-2026, v1.8.) Each trip purchase has a date to sell (تاریخ فروخت), starting as its date and changeable to a later day. When a new بکری opens, every purchase with goods left whose date to sell is on or before that بکری's date is already there as a lot: seller اپنا مال, that purchase, its item and packing, qty = the goods left, before anything is typed. The user adds the buyers, qty and rate and saves; the lot is charged and posted like any own lot (FR-12.26), so the commission is never missed. The user can remove such a lot before saving. Goods not sold on their day stay in باقی مال and come again into the next new بکری until they are sold or written off. Opening a saved بکری, a later sale or an edit never adds lots. Example: purchase خ-1 of 80 پیٹی مالٹا with date to sell 28-10-26; on 28-10-26 `Alt+B` opens with lot 1 = اپنا مال, مالٹا, پیٹی, 80; if 60 are sold, the other 20 come again on 29-10-26.

FR-12.40 [C] **Garden goods arrived.** (Umar, 27-10-2026, v1.8.) A garden's entry list has **مال آیا** (Goods arrived): date to sell, item, packing, qty. It posts no money and changes no partner total; it is the garden's stock. A new بکری gets a ready lot for it as in FR-12.39, with the garden as seller (FR-12.36). Like an own lot, the lot's qty equals what is sold in it, and the rest stays in the arrival for the next day. A charge a rule works out per lot (a per-lot commission, labour, munshiana or phone) is taken once per goods-arrived entry: on its first active lot; if that lot is cancelled, the next new lot of the entry carries it (Umar, 30-10-2026). An amount typed by hand stays as typed. A garden lot typed by hand still works as before. A closed garden takes no arrival; leftover garden goods are sold or written off (no money) before closing.

FR-12.28 [C] **Spoiled trip goods.** Writing off trip stock (FR-12.16) takes the qty off the purchase with its cost (C21) and posts nothing: the partners have paid for the goods already, and no money comes back. The loss shows in the trip's result. Undoing the write-off gives the qty and cost back. Example E2E-34b.

FR-12.33 [C] **A trip closes by itself** when it has at least one purchase and every purchase has nothing left (all sold or written off). A trip with no purchase yet stays open. Goods left over carry to the next day, and the trip stays open. A closed trip opens again if a sale or a write-off of it is reversed. At closing, each trip partner's statement gets a note line with the trip's total cost, sales money, result, his % and his part of the result (FR-17.10).

### 12.11 Gardens (باغات) [C]

Umar (18-10-2026 and 19-10-2026): some partners rent a garden (باغ) for a season. They pay the rent (ٹھیکہ) in installments, and pay its running costs. The fruit comes to our shop and is sold in بکری with the normal commission, or is sold in another shop. When the goods stop coming, the garden is closed, and its money goes to its partners by their %. Gardens have their own section in the main bar (FR-5.2).

| Field | Urdu | Type | Req | Rule |
|---|---|---|---|---|
| Garden no | باغ نمبر | Auto at save | Yes | Code by type 008 (FR-10.2): the first garden is 0080, then 0081 ...; its account under 0306 has the same code (v1.8; before: باغ-1, باغ-2) |
| Name | نام | Text, Urdu and English (FR-7.24) | Yes | |
| Date | تاریخ | Date | Yes | An open date |
| Item | قسم مال | Picker | No | The main fruit |
| Rent | کل ٹھیکہ | Money | Yes | The full price of the garden for the season |
| Paid now | پہلی قسط | Money | No | Saved as the first installment (FR-12.35) |
| Season end | سیزن ختم | Date | No | |
| Partners and shares | شراکت دار اور حصہ | As for a trip | Yes | FR-12.29. Filled from the firm's shares [A] |
| Note | نوٹ | Text | No | |

FR-12.34 [C] **Garden made.** A garden is made with the questions of FR-12.29. Saving makes its own account under control 0306 (FR-10.4). The rent still due (rent − installments paid) is shown on the garden. It is owed by the garden's partners, not by the shop, so nothing posts for it.

FR-12.35 [C] **Installments and running costs.** An installment (قسط) or a running cost (labour, spray, transport) is entered on the garden with its date, amount and reason, and paid in as in FR-12.30: split by the garden's %, with each partner's source. Example: rent 10,00,000; الف 60% from the shop, ب 40% from his own pocket. An installment of 6,00,000: الف's 3,60,000 comes from the shop's cash and off his کل رقم; ب pays his 2,40,000 himself, and his total does not change.

FR-12.36 [C] **Sales.**
- In our shop: a بکری lot can have an open garden as its seller. Seller-side commission and lot charges follow the rules as for any seller (default 6%); this commission is firm income, split by the firm's shares. The seller credit (C8) goes to the garden's account جمع. On a garden lot, نقد and the پیشگی سے کاٹیں tick are hidden and stay 0 [A].
- In another shop: date, shop (as typed), qty and the money received. The money posts: روکڑ بنام, the garden's account جمع [A: the money comes into روکڑ].

FR-12.37 [C] **Running balance.** (v1.8: the gardens list, the garden card and each goods-arrived line also show the shop's commission on the garden's goods sold in our shop, as for trips, FR-12.27.) The garden screen shows, for every date, its money in (sales money after commission) minus everything paid (installments and costs, from the shop and from pockets), and the rent still due. It starts in minus. Example: rent 50,000 with 30,000 paid: −30,000.

FR-12.38 [C] **Closing.** When the goods stop coming, the Owner closes the garden, after the confirm box of FR-7.8.
- The garden's account balance (all its money in) goes to its partners by its % (C26): the garden's account بنام, each partner's 0604 جمع. This gives each partner back what he paid, plus or minus his share of the profit or loss. If the balance is بنام (charges were more than sales), it is split the same way and posted on the other sides (C11).
- The garden's account is then 0, and the garden takes no more entries or lots. A lot of that garden that still has goods left takes no later sale and gives no fruit (FR-16.11) while the garden is closed (`ERR_GARDEN_CLOSED`) [A].
- Each partner's statement gets the closing line: garden, date, total cost, total sales money, profit or loss, his %, his part of the profit or loss and his money back (FR-17.10).
- Closing can be undone by reversal while its date is open. Until it is undone, a بکری posting with a lot of that garden cannot be edited or cancelled [A].

Example (E2E-40): money in 66,400, cost 55,000, profit 11,400. الف (60%, paid 33,000 from the shop) gets 39,840, so his total changes by −33,000 + 39,840 = +6,840 = 60% of 11,400. ب (40%, paid 22,000 himself) gets 26,560 = his 22,000 back + 4,560 (40% of 11,400).

### 12.12 Party ledgers (پارٹی کھاتے) [C]

FR-12.43 [C] **Party ledgers.** (Added in v1.32.) Umar (D8 Feature 6): a new item in the بکری side menu, above باقی مال, named پارٹی کھاتے (Party Ledgers). It shows each party's own ledger, in three levels:

| Level | What it shows |
|---|---|
| 1. List | A toggle for buyers or sellers. One row per party: code, name, current balance and its side (بنام / جمع) |
| 2. Statement | Click a party: one row per date with an entry: the previous balance, charged that day, paid that day, and the updated balance |
| 3. Day detail | Click a date: the full day detail: items, qty, rate, commission, لاگا, the day's total, previous balance, paid on the spot, what is left, and the new total |
| Print | The day detail has a print button: a complete day slip |

- The same for sellers: sales, deductions, money paid to him, and his balance.
- Level 3 and its print reuse the R35 day slip (FR-19.12), so the figures cannot differ. Every figure matches the party card and R13.
- چکتا buyers are not listed: they have no ledger (FR-13.6).
- It changes nothing and loads page by page (FR-19.11).

Example (E2E-67, after the worked day): buyers list: اکرم ٹول ٹیکس 2,880 بنام; ارشد ملتانی 18,684 بنام. اکرم's statement: 10-10-26: previous 0, charged 12,880, paid 10,000, balance 2,880. اکرم's day detail: کیلا, پیٹی, 20 @ 600 = 12,000; commission 720; لاگا 160; day's total 12,880; paid 10,000; new balance 2,880.

## 13. Module: چکتا کھاتہ (walk-in buyers) [C]

Umar (demo feedback of 08-11-2026, D8 Feature 5; decided 09-11-2026): the old one-time accounts are removed completely, and چکتا کھاتہ takes their place. A چکتا buyer is a walk-in buyer who pays the whole bill on the spot and leaves.

FR-13.1 [C] **One system account.** چکتا کھاتہ / Chukta Khata, code 0030, control 0304, type number 003 (FR-10.2). It holds no balance after any sale, because every line on it is paid in full.
FR-13.2 [C] **The list of چکتا buyers.** Each چکتا buyer has both names (FR-7.24) and an optional phone. There is one record per person, compared by name and phone after Urdu normalisation. He is found again by name, so his name is spelt the same way next time.
FR-13.3 [C] **Added on the spot.** The buyer picker of a بکری line has an add row "نیا چکتا گاہک" beside "نیا گاہک" (FR-7.69). The short form asks his name and an optional phone. He is saved at once and chosen at once.
FR-13.4 [C] **Only in that picker.** The buyer picker of a بکری line lists چکتا buyers beside ordinary buyers, each marked چکتا (FR-7.29). It does not offer the account 0030 itself. Other pickers do not list چکتا buyers: they are not parties.
FR-13.5 [C] **A sale line.** A چکتا line is on the account 0030 with the person: his names, phone and person key are on the line and on its journal lines. He is a buyer only, never a seller. He is charged the normal commission and لاگا (FR-11.7). He always pays the full amount: the backend sets paid = buyer total (FR-12.44).
FR-13.6 [C] **No personal ledger.** He has no balance, no dues and no aging. The party card shows only his name, phone and the words "چکتا: پورا نقد" (FR-7.38). He is not in R13, R14, R20 to R24, R26, party ledgers (FR-12.43), the voucher pickers, the attention list or the dead amount.
FR-13.7 [C] **He stays in the day's records.** The day's sale book, sales, commission and partner split count him like any sale. He is shown by name in R2, R3, R5, R34, the all-parties table of R35 and analytics.
FR-13.8 [C] **Removed for good (v1.32).** The account 0031 and one-time sellers; the own accounts of the old series from 0032; the old register R9 and its menu item; the person picker on وصولی and ادائیگی; make regular; the old filter of the accounts list; dead-amount marks by person; and the one-time lines of R24 and R26. R9 is retired; other report numbers do not move.
FR-13.9 [C] **Old data.** Only development databases can hold old entries, because no shop uses the app yet. A new migration renames the account 0030, makes the people of old lines on 0030 into چکتا buyers, makes 0031 inactive and renames it to say it is an old closed account, and leaves the old own accounts as plain sellers or buyers. No posted entry, amount or chain hash is touched. Old open amounts on 0030 stay in the books and the trial balance, and do not show in dues lists.
FR-13.10 (retired in v1.32).
FR-13.11 (retired in v1.32).

Example (Section 4.5): شاہد buys 10 پیٹی سیب at 1,200. Value 12,000; commission 720; لاگا 80. He pays 12,800 on the spot and leaves. He is in the day's sale book, and he has no balance.

---

## 14. Freight (کرایہ) [S]

The old software's بکری menu has one button, labelled "کرایہ بک", and it opens the بکری بک screen. So the old software has no separate freight register, and we do not build one.

FR-14.1 [S] Freight is typed on each lot. The split from a header total (C17) is retired in v1.34. When the firm pays the driver, it posts in the بکری journal to کرایہ کھاتہ (02010002), the money owed to transporters (FR-12.9).
FR-14.2 [A] A transporter is paid with a normal ادائیگی from کرایہ کھاتہ (Section 15.3). The ledger of کرایہ کھاتہ (R13) shows what is still owed.

---

## 15. Module: وصولی, ادائیگی, پیشگی (in the بکری side menu) [C]

### 15.1 Common voucher fields

| Field | Urdu | Type | Req | Rule |
|---|---|---|---|---|
| Voucher no | واؤچر نمبر | Auto at save | Yes | Own series per type (و-1, ا-1, پ-1) |
| Date | تاریخ | Date | Yes | Today; must be an open date |
| Account | کھاتہ | Picker | Yes | Filtered by type. چکتا buyers are never listed (FR-13.6) |
| Current balance | موجودہ بقایا | Read-only | | Shown as soon as the account is picked |
| Amount | رقم | Money | Yes | More than 0 |
| Narration | تفصیل | Text + standard narration | No | |
| New balance | نیا بقایا | Read-only | | After this voucher |
| System fields | | Auto | | Created/updated by and at, status, cancel reason |

There is no mode field: all money is روکڑ, the shop's one cash account, in the drawer or in the bank (Umar, 19-10-2026). There is no bank account and no cheque.

FR-15.1 [A] Several vouchers can be entered one after another in a list, with a running total, and saved together (each is its own posting event).
FR-15.2 [A] Print a receipt slip with the amount in words (FR-19.7).
FR-15.3 [A] Edit while its date is open (reversal + new posting), or in a closed day through FR-21.4; cancel with reason; all effects reversed (Section 6). Edits follow FR-7.70 and are stamped (FR-7.71).

### 15.2 وصولی / اگراہی (receipt from buyer) [C]

Umar: the buyer took goods but did not pay. The firm collects later (اگراہی).

Extra field: collector (اگراہی والا) [A].

- Picker shows buyers first, oldest due at the top. After a buyer is picked, his open bills show (FR-7.58).
- Effect: روکڑ بنام, buyer جمع.
- FR-15.4 [A] **Bill matching (oldest first).** A buyer's "bills" are every debit posted to his account: sale line totals, opening debits, cash lent (FR-15.11) and جنرل debits. A receipt settles the oldest open bills first. This keeps aging correct. Money paid on the spot settles its own line's bill first; what is left of that bill is a normal open bill, matched oldest first against the buyer's unmatched credits (FR-12.44, v1.32).
- FR-15.5 [A] If a buyer pays more than he owes, the extra stays as his credit and settles his next bills automatically. A person in the dead amount cannot pay more than he still owes (FR-10.10).
- FR-15.6 [A] "اگراہی فہرست" button: every buyer who owes the shop, by collector and city, printable. [C] Umar (03-11-2026): a buyer can be collected from on any day, so there is no collection day on the account and no weekday plan. Example: on any day the list shows اکرم 2,880 and ارشد 18,684 under their collector. People in the dead amount are left out (FR-10.10).
- Example: اکرم owes 12,880, pays 10,000, still owes 2,880.

### 15.3 ادائیگی (payment) [C]

- Picker shows sellers first, with payable balance. After a seller is picked, his payable postings show (FR-7.59).
- Effect: account بنام, روکڑ جمع.
- FR-15.7 [A] If the payment is more than the open payable (FR-7.59), a warning shows "یہ رقم بقایا سے زیادہ ہے". The user can still save. The extra shows as the seller owing the firm ("بیوپاری کے ذمے" list). It is not an advance; advances are only made with پیشگی.
- Example: pay رحیم 4,316. Balance 0.
- FR-15.11 [C] **Cash lent to a buyer.** An ادائیگی to a buyer is cash lent to him (نقد ادھار). A reason is required. It posts buyer بنام, روکڑ جمع. It is one of his bills (FR-15.4): it ages, shows in his اگراہی and is collected with a normal وصولی. His ledger shows it as "نقد ادھار" with the reason. Example E2E-32.

### 15.4 پیشگی کھاتہ (advance to seller) [C]

Umar: the seller takes money in advance, before his goods are sold.

Extra field: expected goods or season (text) [A].

- Effect: seller بنام, روکڑ جمع. The voucher is an advance.
- FR-15.8 [A] **Advance matching (oldest first).** A seller's advances are his پیشگی vouchers and any opening debit marked as an advance (FR-10.2). Each later seller credit from a بکری posting adjusts his open advances, oldest first, when the lot's "پیشگی سے کاٹیں" tick is on (FR-12.23). A seller's وصولی that returns advance money also adjusts them (FR-15.12). The پیشگی report shows given, returned, adjusted and open; the dashboard tile shows the open total.
- FR-15.9 [A] If a seller's advances are more than his sales, the rest stays as his open advance (پیشگی باقی) and his balance shows as بنام. The "بیوپاری کے ذمے" list shows only what a seller owes beyond his open advances (for example an overpayment, FR-15.7), so nothing is counted twice.
- Example: رحیم took 15,000 on 01-09-26. His بکری on 10-10-26 credits 24,316, and he got 5,000 نقد. Advance adjusted 15,000 (fully). Payable 24,316 − 5,000 − 15,000 = 4,316. With the tick off (E2E-28): payable 24,316 − 5,000 = 19,316, advance open 15,000.
- FR-15.12 [C] **A seller returns advance money.** A وصولی from a seller posts روکڑ بنام, seller جمع. It adjusts his open advances, oldest first, then settles what he owes the firm ("بیوپاری کے ذمے"). An amount more than both together is refused (`ERR_RETURN_EXCEEDS_OPEN`, with the open amount). Example E2E-31.

FR-15.10 [A] وصولی, ادائیگی and پیشگی are one voucher screen with a type list at the top (FR-7.67). The side menu items, `Alt+V`, `Alt+A` and `Alt+P` open it with that type already chosen. The same screen is also reachable from the روکڑ area. It is one screen, not copies. Each type keeps its own helpers (open bills, payables, open advances), its own number series and its own posting.

---

## 16. Module: روکڑ (Cash book, expenses, journal)

The روکڑ area shows: روکڑ بک, خرچہ (دکان خرچہ and ذاتی خرچہ on one screen, 16.2), پھل (16.7), ادھار رقم, الگ کھاتہ, جنرل, and the voucher screen for وصولی, ادائیگی and پیشگی (FR-15.10). The old software's four-part روکڑ menu (وصولی, پیشگی, ادائیگی, جنرل) is fully covered [S].

### 16.1 روکڑ بک (cash book)

Opening cash, every cash in and out from all modules, closing cash. Filters: date range.

FR-16.1 [C] **No cash count.** Umar (19-10-2026): the روکڑ بک shows the book only; there is no box to type the counted cash. روکڑ is the shop's one cash account, in the drawer or in the bank.

### 16.2 خرچہ: shop expense and a partner's personal spending, one screen [C]

Umar (15-10-2026): one screen for all spending. A list at the top says what it is. A shop expense (electricity, rent, tea and food for guests) is split between all partners by their shares. A partner's personal spending (his home, a wedding) is charged to that partner only.

| Field | Urdu | Type | Req |
|---|---|---|---|
| Type | قسم | Select: دکان خرچہ (shop expense), ذاتی خرچہ (a partner's personal spending), ذاتی خرچہ واپسی (a partner gives money back) | Yes. Starts as دکان خرچہ |
| Date | تاریخ | Date | Yes |
| Expense head | خرچہ مد | Picker | دکان خرچہ only |
| Partner | شراکت دار | Picker | ذاتی خرچہ and واپسی only |
| Amount | رقم | Money | Yes |
| Paid to | ادا کیا | Text | No. دکان خرچہ only |
| Reason | وجہ | Text | Yes for ذاتی خرچہ |
| Return by | واپسی کی تاریخ | Date | No. ذاتی خرچہ only |
| Taken by | لینے والا | Text | No. ذاتی خرچہ only. The person who took the cash, if not the partner himself (for example his son) |
| Narration | تفصیل | Text | No |

Effect of دکان خرچہ: expense head بنام, روکڑ جمع. Every shop expense is split between partners by their shares on that date (FR-6.5). The effect of ذاتی خرچہ and its واپسی is in 16.3.

FR-16.8 [C] **The screen shows the effect before save.** (Numbered after FR-16.7 because it was added in v1.4.) While the entry is typed, the screen shows what the save will do. The backend works it out with the same function as the save (Code Standards B5), so the screen and the books never differ.
- دکان خرچہ: each partner's share (C11). Example: بجلی 3,000 shows الف 1,200, ب 900, ج 900.
- ذاتی خرچہ or واپسی: only the chosen partner, with his کل رقم now and after this entry (C22); the other partners show as not affected. Example: الف takes 2,000 after Events 1 to 4 and the electricity bill: his کل رقم 996 becomes −1,004.

Below the form: for دکان خرچہ, the day's shop expenses with their heads; for ذاتی خرچہ and واپسی, that partner's list of money taken (FR-16.2). Each type keeps its own number series and its own posting (shop_expense, personal_expense, personal_return), so reports, the partner statement and every test number stay as they were. Example E2E-39.

### 16.3 ذاتی خرچہ: money a partner takes and gives back [C]

Umar (11-10-2026): a partner taking money and giving it back is recorded with who, date, amount and reason. Both are types on the خرچہ screen (16.2, FR-7.67).

FR-16.2 [C] **Effect.** Taken: that partner's drawings account بنام, روکڑ جمع. Returned: روکڑ بنام, drawings جمع. Never split. Below the form, the screen lists that partner's money taken: date, amount, reason, taken by, return by, returned, still open. Returns settle his oldest open amounts first. This is worked out when shown; nothing extra is saved. A return more than his open amount is refused (`ERR_RETURN_EXCEEDS_OPEN`, with the open amount). Each entry gets a number in its own series, as in 15.1. An amount past its return-by date shows in the attention list (FR-7.44).

FR-16.3 [C] **Partner's net.** For any period: net taken = taken − returned; خالص (net) = profit share (C12) − net taken (C20). It can be negative. Example E2E-29: on 10-10-26 الف has profit share 996 and took 2,000, so خالص is −1,004; after he returns 2,000 on 11-10-26, net taken is 0 and خالص is 996.

### 16.4 جنرل (journal) [S]

A list of lines: account, بنام, جمع, narration. Cannot save unless totals match. Used for corrections, transfers and goods returns. Lines on income or shop expense accounts write partner shares (FR-6.5); lines on buyer accounts take part in bill matching (FR-15.4). [C] A جنرل line on a seller account never adjusts his advances (Umar, 15-10-2026): advances are adjusted only by بکری credits and the seller's وصولی (FR-15.8). A جنرل debit on a seller settles his payables, and a جنرل credit on a seller is a payable posting (FR-7.59). Example: رحیم has 15,000 of advance open and a جنرل correction credits him 1,000. His advance stays 15,000 open, and the 1,000 is payable to him.

### 16.5 ادھار رقم (money lent to staff and others) [C]

Umar (11-10-2026): a group for staff and anyone else who borrows money from the shop.

| Field | Urdu | Type | Req |
|---|---|---|---|
| Entry | اندراج | Select: رقم دی (lent), واپسی (returned) | Yes |
| Date | تاریخ | Date | Yes |
| Person | نام | Picker of ادھار رقم accounts (control 0103). `F4` adds a person: Urdu and English name, phone, staff record if he is staff. | Yes |
| Amount | رقم | Money | Yes |
| Mode | ذریعہ | واپسی only: روکڑ or تنخواہ سے کٹوتی (deducted from salary) | For واپسی |
| Salary head | خرچہ مد | Expense head picker, default تنخواہ | If تنخواہ سے کٹوتی |
| Reason | وجہ | Text | Yes, when lent |
| Return by | واپسی کی تاریخ | Date | No (lent only) |
| Narration | تفصیل | Text | No |

FR-16.4 [C] **Effect.** Lent: the person's ادھار رقم account بنام, روکڑ جمع. Returned in cash: روکڑ بنام, person جمع. Deducted from salary: the salary head بنام, person جمع, and no cash moves. That part of the salary is a shop expense, so it is split between partners (FR-6.5); the rest of the salary is paid as a normal دکان خرچہ. Below the form, the screen lists the person's loans: date, amount, reason, return by, returned, still open, days past the return-by date. A return or salary deduction more than the open amount is refused (`ERR_RETURN_EXCEEDS_OPEN`, with the open amount). Each entry gets a number in its own series, as in 15.1.

FR-16.5 [C] **Open amounts.** Returns settle the person's oldest open loans first. This is worked out when shown; nothing extra is saved. A loan past its return-by date shows in the attention list (FR-7.44). Report R32 lists everyone. Example E2E-30.

FR-16.9 [C] **Return-by date.** (Numbered after FR-16.8 because it was added in v1.5.) Umar (15-10-2026): a return-by date (ذاتی خرچہ, 16.2; ادھار رقم, 16.5) cannot be earlier than the entry's date. It is refused (`ERR_RETURN_BY_BEFORE_ENTRY`) with "واپسی کی تاریخ اندراج کی تاریخ سے پہلے نہیں ہو سکتی" (The return-by date cannot be before the entry date). The same day is allowed. Example: money lent on 15-10-26 with return by 10-10-26 is refused; 15-10-26 or later saves.

### 16.6 الگ کھاتہ (separate khata) [C]

Umar (15-10-2026): sometimes money has nothing to do with the partners, commission or profit, for example a family khata (خاندانی کھاتہ). A garden rented by partners is not a الگ کھاتہ: it has its own section (12.11). Money given out for it is written as minus, money coming back is plus, and goods from it sold in the shop add to it. Every entry has its date and reason.

| Field | Urdu | Type | Req |
|---|---|---|---|
| Entry | اندراج | Select: رقم دی (paid out, minus), رقم آئی (received, plus) | Yes |
| Date | تاریخ | Date | Yes |
| Khata | کھاتہ | Picker of الگ کھاتے (control 0305). `F4` adds one: Urdu and English name, note. | Yes |
| Amount | رقم | Money | Yes |
| Reason | وجہ | Text | Yes |
| Narration | تفصیل | Text | No |

FR-16.6 [C] **Effect.** Paid out: the khata بنام, روکڑ جمع. Received: روکڑ بنام, the khata جمع. A الگ کھاتہ is never income or expense, is never split, and is never part of any partner's total (C22). Its balance is shown with a sign from the khata's side: plus = the shop holds money for it, minus = it has taken more than it brought. Below the form, the screen lists the khata's entries in date order with reason, amount and running balance; it prints for any date range (FR-19.10). Each entry gets a number in its own series, as in 15.1.

FR-16.7 [C] **Selling its goods.** A بکری lot can have a الگ کھاتہ as its seller (for example کیلا from the family's land). Its seller credit (C8) goes to that khata as plus. Seller-side charges (commission, labour, munshiana, phone) are 0 by default; a rule on that khata can set them. The buyer side (buyer commission, لاگا) is charged to the buyer as on any line [A]. Example E2E-38: the family khata pays out 10,000 (−10,000), gets 4,000 back (−6,000), then sells 10 پیٹی کیلا at 900: 9,000 to the khata, so it shows +3,000.

---

### 16.7 پھل: fruit taken by partners [C]

Umar (19-10-2026): partners take fruit home, and fruit is served to the shop's guests. It is tracked by item and quantity, not only by amount, so it has its own screen in the خرچہ area (a tab next to خرچہ).

| Field | Urdu | Type | Req |
|---|---|---|---|
| For | کس کے لیے | Select: one partner (گھر کے لیے), or مہمان (the shop's guests, shared) | Yes |
| Date | تاریخ | Date | Yes |
| Source | کہاں سے | Select: اپنی دکان (from a lot in باقی مال), دوسری دکان (another commission agent's shop) | Yes |
| Lot | لاٹ | Picker of lots with balance | If اپنی دکان |
| Agent | آڑھتی | Seller picker | If دوسری دکان |
| Item, packing | قسم مال، پیکنگ | From the lot, or pickers | Yes |
| Quantity | تعداد | Whole number | Yes |
| Weight | وزن (کلو) | Up to 3 decimals | If sold by weight |
| Rate | ریٹ | Money | Yes. Typed; the last rate of the item shows as a hint only (FR-7.53) |
| Value | قیمت | Read-only | C1, worked out by the backend |
| Note | نوٹ | Text | No |

FR-16.10 [C] **Effect.** (FR-16.10 to FR-16.12 were added in v1.6.)
- For one partner: his garden, trip and fruit account (0604) بنام the value. It comes off his کل رقم only; the other partners are not affected.
- For the shop's guests: the value is split equally between the partners active on that date (C27), not by the commission shares. Each one's 0604 بنام his part.
- Fruit is never a دکان خرچہ and is never split by the firm's shares. Tea and food for guests stay a دکان خرچہ head (مہمان نوازی, E55).

Example: الف takes 2 پیٹی مالٹا at 1,400: value 2,800, his total −2,800. Guest fruit of 3,000 with three partners: 1,000 each; with two partners: 1,500 each.

FR-16.11 [C] **Source.**
- From our shop: the fruit comes from a lot with balance. The lot's balance goes down by the qty (C16), and the lot's seller is credited the value (the seller's account or the garden), with no commission [A]. The credit is a payable posting (FR-7.59) and never adjusts an advance. A qty more than the lot's balance is refused (`ERR_QTY_EXCEEDS_LOT`). Trip goods are not offered: they belong to the trip's partners, not to a lot [A].
- From another commission agent's shop: the agent (kept as a بیوپاری account) is credited the value. The shop pays him later with a normal ادائیگی.
- No cash moves on the پھل screen.

FR-16.12 [C] **List and print.** Below the form, the screen lists fruit taken: date, for whom, item, qty, rate, value and source. It filters by partner, item and date (one day, a month or a year), and prints and exports (FR-19.10, R37). Each partner's statement shows a line for each, for example "19-10-26 گھر کے لیے پھل: مالٹا 2 پیٹی @ 1,400 = 2,800" (FR-17.10). Each entry has a number in its own series (پھل-1, پھل-2 ...). An entry is edited or cancelled by reversal while its date is open, as in FR-15.3.

## 17. Module: شراکت دار (Partners and income split) [C]

Umar: there are multiple partners. As each sale is done, the commission is divided and added to each partner's total. Commission itself changes by item, seller and buyer (Section 11).

### 17.1 Partner setup

| Field | Urdu | Type | Req |
|---|---|---|---|
| Partner name | نام | Text, Urdu and English (FR-7.24) | Yes |
| Capital account | سرمایہ کھاتہ | Auto-created | Yes |
| Drawings account | ذاتی خرچہ کھاتہ | Auto-created | Yes |
| Gardens, trips and fruit account | باغ، سفر اور پھل کھاتہ | Auto-created (0604, v1.6) | Yes |
| Share % | حصہ فیصد | Number, up to 2 decimals | Yes |
| Opening capital | ابتدائی سرمایہ | Money | No. Posts as the opening of his capital account (FR-10.9). Use either partner opening capitals or the firm's ابتدائی سرمایہ 06010001, not both, or the same money is counted twice. The worked day uses 06010001 only. |
| Effective from | تاریخ سے | Date | Yes |
| Active | فعال | Checkbox | Yes |

FR-17.1 [C] Shares of active partners must total exactly 100% for every date. The screen shows the running total and will not save otherwise.
FR-17.2 [A] A share change takes effect from its effective date, which must be today or later (no back-dating). Share history is kept. Postings already saved keep their split.
FR-17.3 [A] **What is shared.** All income accounts (head 04) are shared. All shop expense accounts (head 05) are split. Payables (مزدوری, کرایہ, کرایہ سٹور, منشیانہ) are not income and are not shared. So no income or expense is ever left without an owner. Where this document says "shop expense account" for partner shares and month close (FR-6.5, FR-17.4, FR-17.6), it means every head 05 account; from v1.6 these are the دکان خرچہ heads (0501) only. A garden's or trip's money is never income or a shop expense: it goes only to that garden's or trip's partners (FR-12.30).

### 17.2 Live split

FR-17.4 [C] Every posting that touches income or shop expense accounts records each partner's share at once: income shares positive, expense shares negative, using C10 and C11, with the shares on the posting's date. A reversal records the exact opposite rows.
FR-17.5 [A] Partner share rows are kept per posting (not per sale line).
FR-17.6 [A] **Month close (مہینہ بند).** The Owner presses it for a finished month. It posts one journal: each income account بنام its month balance, each shop expense account جمع its month balance, and each partner's capital account جمع (or بنام if negative) the sum of that partner's share rows for the month. It balances because of FR-6.5. After month close, the month's days must be closed.

### 17.3 Partner screens

**Partner overview** (one card or row per partner):

| Column | Meaning |
|---|---|
| آج کا حصہ | Today's income share |
| مہینے کا حصہ | This month's income share |
| سال کا حصہ | This year's income share |
| خرچہ حصہ | Expense share this month |
| منافع حصہ | Income share − expense share |
| ذاتی خرچہ | Money taken this month (FR-16.2) |
| واپسی | Money returned this month |
| خالص | This month: منافع حصہ − (ذاتی خرچہ − واپسی), C20 |
| سرمایہ لگایا | Capital put in this month, less capital taken back (FR-17.7) |
| سرمایہ بقایا | Capital balance |
| باغ، سفر اور پھل | This month's net from his gardens, trips and fruit taken (FR-17.10) |
| کل رقم | Partner total (C22, FR-17.9): one number for everything he has in the firm |

Under each partner's card: the open gardens and trips he is in, each with his %, his money in so far (from the shop and from his pocket), his money back so far, and the running balance (garden) or result (trip). Each opens its screen (Umar, 18-10-2026).

**Partner statement** (R31, per partner, date range): a dated record of every movement (FR-17.9), with the running کل رقم, printable.

Example from Section 4: الف today: income share 2,196, expense share 1,200, profit share 996, taken 2,000, returned 0, خالص −1,004.

### 17.4 Partner capital (سرمایہ): investment in and out [C]

Umar (14-10-2026): to start and run the shop, the partners put their own money in. That money is used to give advances, lend to buyers and run the اگراہی. Each partner's investment must be recorded.

| Field | Urdu | Type | Req |
|---|---|---|---|
| Entry | اندراج | Select: سرمایہ جمع (invested), سرمایہ واپسی (taken back) | Yes |
| Date | تاریخ | Date | Yes |
| Partner | شراکت دار | Picker | Yes |
| Amount | رقم | Money | Yes |
| Narration | تفصیل | Text | No |

FR-17.7 [C] **Effect.** Invested: روکڑ بنام, that partner's capital account (0602) جمع. Taken back: capital account بنام, روکڑ جمع. Capital is never income or expense and is never split. Below the form, the screen shows the partner's capital history and balance. Each entry gets a number in its own series, as in 15.1. Example E2E-35.

FR-17.8 [A] **Capital and shares.** Capital does not change profit shares; shares stay as set in 17.1. Taking back more than the capital balance shows a warning, and can still be saved. Capital taken back is not the same as ذاتی خرچہ: ذاتی خرچہ is money taken for himself out of his profit share (it lowers خالص); capital taken back is his own investment returned. Example: ب invests 50,000 on 11-10-26 and takes 10,000 back on 20-10-26; his capital is 40,000 (before any month-close profit) and his خالص does not change.

FR-17.9 [C] **Partner total (کل رقم) and its record.** Umar (15-10-2026): a partner owns his investment, his share of the commission and his share of the profit, so the app shows them together as one number, and keeps a dated record of everything that made it. کل رقم (C22) = opening capital + capital invested − capital taken back + all his income shares (commission, لاگا, فون, ریٹ فرق) − all his expense shares (shop expenses) − money taken (ذاتی خرچہ) + money returned + his garden, trip and fruit account (money credited from gardens and trips, less his parts paid from the shop's cash and the fruit he took, v1.6). It is live: it changes with every posting, with no need to wait for month close. Month close (FR-17.6) only moves amounts between his accounts, so it never changes کل رقم.
- The partner statement (R31) lists every movement in date order, one line each, with its date, what it was and where it came from, its amount and the running کل رقم:
  - each posting's income share, with its source (for example "10-10-26 بکری 1: کمیشن، لاگا، فون 1,184");
  - each expense share (for example "10-10-26 دکان خرچہ بجلی −900");
  - capital invested and taken back;
  - money taken (with its reason) and returned;
  - garden, trip and fruit lines (FR-17.10);
  - month close, shown as a note that moves profit into capital and leaves the total the same.
- Every line comes from posted records (share rows, journal lines, vouchers), so nothing is typed twice and the record can never disagree with the books.
- Check that always holds: Σ کل رقم of all partners + ابتدائی سرمایہ 06010001 = the firm's equity in the trial balance (capital accounts, including the garden, trip and fruit accounts, + income − expenses − drawings).
- Example E2E-37: after Events 1 to 5 and ب's 50,000 on 11-10-26: الف 2,196 − 1,200 − 2,000 = −1,004; ب 50,000 + 1,646 − 900 = 50,746; ج 1,646 − 900 = 746. Σ 50,488 + 65,000 = 115,488 = equity in the books.

FR-17.10 [C] **Statement lines.** (Added in v1.6.) Umar (18-10-2026 and 19-10-2026): every amount that reaches a partner has its own line in his statement (R31), made from posted records and never typed by hand: date, heading (where it came from), reason, amount and the running کل رقم.
- Headings: بکری (his share of the commission, with the بکری number), خرچہ (his expense share, or money taken with its reason), سرمایہ, باغ (his part paid from the shop's cash; the closing), سفر (his part paid from the shop's cash; the sales money credited to him), پھل (item, qty and rate).
- Note lines show no amount and leave the running total the same: a month close (FR-17.9), a trip's closing (FR-12.33) and a write-off of trip goods (FR-12.28).
- Examples: "10-10-26 سفر-1: خریداری خ-1، دکان سے −4,160"; "10-10-26 سفر-1، بکری 1: مالٹا 15 پیٹی، فروخت 8,460، لاگت 7,800، منافع 660، آپ کا 40%: +3,384"; "13-10-26 باغ-1 شاہ پور باغ بند: لاگت 55,000، فروخت 66,400، منافع 11,400، آپ کا 60%: 6,840، واپسی +39,840".

FR-17.11 [C] **Partner code (شراکت دار کوڈ).** (Added in v1.13.) Umar (31-10-2026): the partner section is the most sensitive, so it opens only with a 4-digit code.
- **Setup.** The first time the شراکت دار section is opened, the app asks to set the code: 4 digits, typed twice. Setting it needs the Owner's password, so nobody else at the computer can set it [A]. Changing it asks for the old code (or the Owner's password).
- **Every time.** Opening the شراکت دار section asks for the code. It stays open while the user stays on screens behind the code; going to any other screen, the idle lock (FR-8.4), a logout or closing the app locks it again [A].
- **Also behind the code** [A]: every place that shows a partner's totals or history: the dashboard partner strip (9.2, shown as a locked card with "کوڈ درج کریں"); reports R30 and R31, the partner parts of R29, R33, R34 and R36, and R37 by partner; analytics view F and the partner columns of views G, I, J and K; the partner lines of the day close and month close summaries; and a partner's کل رقم before and after an entry (the خرچہ and پھل previews, FR-16.8). On such a screen the partner figures show a small "کوڈ درج کریں" (Enter the code) box in their place; the rest of the screen works.
- **Not behind the code:** the names of the partners in a picker, and the split of the one entry on screen (a بکری's split, a trip's or garden's parts of one payment, fruit parts), because the munshi checks each entry with it. They show no totals.
- **Wrong code.** After 5 wrong codes, wait 1 minute; after 5 more, 5 minutes; then 15 minutes each time (as FR-8.2). Each wrong try is in the audit log.
- **Forgotten code.** "کوڈ بھول گئے" resets it with the Owner's password or the recovery code (FR-8.5).
- The code is kept only as a one-way hash (NFR-SEC-1). The backend refuses every partner command and report while the section is locked (`ERR_PARTNER_LOCKED`); hiding a screen is not enough (NFR-SEC-4, NFR-SEC-14).
Example (E2E-46): the code is set to 4826. Opening شراکت دار asks for it; 1111 is refused; 4826 opens the overview with الف 996, ب 746, ج 746. Going to بکری and back asks again.

---

## 18. Module: تجزیات (Analytics) [C]

Umar: filter by month or year, and see how much commission we got by item (for example banana), by seller, by buyer.

### 18.1 Filters

| Filter | Urdu | Options |
|---|---|---|
| Period | مدت | آج، یہ ہفتہ، یہ مہینہ، پچھلا مہینہ، یہ سال، پچھلا سال، custom from-to |
| Group by | ترتیب | Day, month (ماہانہ), year (سالانہ) |
| Item | قسم مال | All, or pick one or more |
| Group | گروپ | All, پھل (fruit) or سبزی (vegetable): all the items of that group (v1.29, views A, B, C, D, G and H). Never together with Item. Under a group, view G leaves its gardens column empty, because installments and costs have no item. The group is read from the item list, so changing an item's group moves its history too (Umar, 06-11-2026, v1.31). |
| Seller | بیوپاری | All, or pick |
| Buyer | گاہک | All, or pick. چکتا buyers are in the rows by name, but not in this picker (FR-13.4, FR-13.7) |
| City | شہر | All, or pick |
| اڈا | اڈا | All, or pick |
| Garden | باغ | All, or pick (v1.6) |
| Trip | سفر | All, or pick (v1.6) |
| Goods | مال | All, consigned goods, trip goods, garden goods (v1.6) |
| Partner | شراکت دار | Views F, G, I, J and K only |
| Commission side | جانب | Both, seller only, buyer only |
| Compare | موازنہ | Off, previous period, same period last year. A period of whole calendar months is compared with the same number of whole months before it (September with August, a quarter with the quarter before); any other period with the same number of days just before it [A] (A91, v1.21) |

FR-18.1 [C] In views A to E and H, item, seller, buyer, city, اڈا, garden, trip, goods, side and period combine freely. Example: "کیلا, seller رحیم, monthly, 2026". The partner filter applies to views F, G, I, J and K, which work on periods (FR-17.5).

### 18.2 Views

| View | Urdu | Rows | Columns |
|---|---|---|---|
| A. By item | قسم مال وار | Item | نگ, خام بکری, avg rate, seller comm, buyer comm, لاگا, total comm, % of all commission |
| B. By seller | بیوپاری وار | Seller | بکری count, نگ, خام بکری, seller comm, buyer comm on his goods, total comm, صافی بکری |
| C. By buyer | گاہک وار | Buyer | نگ, purchase value, buyer comm, لاگا, dues now (a buyer in the dead amount shows his balance marked ڈوبی), avg days to pay |
| D. Item × month | قسم مال × مہینہ | Item | One column per month, total commission in each |
| E. Seller × item | بیوپاری × قسم مال | Seller | One column per item |
| F. Partner | شراکت دار وار | Partner | Income share, expense share, profit share per period; and side by side (v1.6): gardens, trips, fruit taken and his net change |
| G. Trend | رجحان | Period | خام بکری, commission, shared income, shop expenses, profit, gardens' and trips' results (chart) |
| H. Top 10 | سرفہرست | Top items, sellers, buyers by total commission | |
| I. Gardens | باغات | Garden | Paid (shop and pockets), sales money (our shop, other shops), commission on its goods, running or closed result, each partner's % and result |
| J. Trips | مارکیٹ سفر | Trip, purchase or item, day | Bought (نگ, cost), sold (نگ, sales money), commission on its goods, written off, left, result, each partner's % and result |
| K. Fruit taken | پھل | Partner or guests, item | نگ, value, source; by day, month or year |

Definitions: "total comm" = seller comm + buyer comm + لاگا. "avg rate" = خام بکری ÷ نگ (C18). "% of all commission" = row total comm ÷ grand total comm × 100, 2 decimals. "avg days to pay" = average days from bill date to the date the bill was fully paid, for bills fully paid in the period; empty if none.

### 18.3 Behaviour

FR-18.2 [A] Every number can be opened (click or `Enter`) to see the lines behind it, and from a line, its بکری.
FR-18.3 [A] Each view has a simple chart above the table.
FR-18.4 [C] Every view prints and exports to Excel, by day, month or year, with its filters in the print header (Umar, 18-10-2026).
FR-18.5 [A] Saved views: save a filter set with a name ("کیلا کمیشن ماہانہ") and open it from the dashboard.
FR-18.6 [C] Sales numbers come from the stored values of each line (analytics facts), never recalculated with today's rules. Expense and profit numbers come from the journal and the partner share ledger.
FR-18.7 [A] Cancelled postings cancel themselves out (their reversals are included), so a cancelled بکری adds nothing.
FR-18.8 [C] **Complete and read only.** (Added in v1.6.) Umar (18-10-2026 and 19-10-2026): analytics changes nothing and covers everything: day, month or year; totals, then by item, buyer and seller, then the lines behind any number (FR-18.2). Gardens, trips and fruit taken have their own views (I, J, K), and the partner view (F) shows each partner's money from commission, expenses, gardens, trips and fruit side by side. Every view is Must (Section 26). Long tables load page by page (FR-19.11).

### 18.4 Example: how to read the views

Filter: October 2026, everything else "all", both sides. Data = the two بکری from Section 4.

View A, by item:

| قسم مال | نگ | خام بکری | Avg rate | Seller comm | Buyer comm | لاگا | Total comm | % |
|---|---|---|---|---|---|---|---|---|
| کیلا | 50 | 29,400 | 29,400 ÷ 50 = 588 | 1,764 | 1,764 | 400 | 3,928 | 3,928 ÷ 5,448 × 100 = 72.10 |
| سیب | 10 | 12,000 | 1,200 | 720 | 720 | 80 | 1,520 | 27.90 |
| **میزان** | **60** | **41,400** | | **2,484** | **2,484** | **480** | **5,448** | **100.00** |

How to read it: in the کیلا row, seller comm 1,764 is 720 + 1,044 from the two lines of بکری 1 (Section 4.4). Total comm = 1,764 + 1,764 + 400 = 3,928. The grand total 5,448 matches today's commission on the dashboard.

View B, by seller:

| بیوپاری | بکری | نگ | خام بکری | Seller comm | Buyer comm on his goods | Total comm | صافی بکری |
|---|---|---|---|---|---|---|---|
| رحیم | 1 | 50 | 29,400 | 1,764 | 1,764 | 3,928 | 19,316 |
| کریم | 1 | 10 | 12,000 | 720 | 720 | 1,520 | 10,160 |

View C, by buyer:

| گاہک | نگ | Purchase value | Buyer comm | لاگا | Dues now | Avg days to pay |
|---|---|---|---|---|---|---|
| ارشد ملتانی | 30 | 17,400 | 1,044 | 240 | 18,684 | (empty, nothing fully paid) |
| اکرم ٹول ٹیکس | 20 | 12,000 | 720 | 160 | 2,880 | (empty, nothing fully paid) |
| چکتا: شاہد | 10 | 12,000 | 720 | 80 | 0 | 0 |

View F, partners, 10-10-26:

| شراکت دار | Income share | Expense share | Profit share |
|---|---|---|---|
| الف | 2,196 | 1,200 | 996 |
| ب | 1,646 | 900 | 746 |
| ج | 1,646 | 900 | 746 |
| **میزان** | **5,488** | **3,000** | **2,488** |

---

## 19. Module: رپورٹ (Reports)

### 19.1 Common behaviour

FR-19.1 [S] Each report first shows its filters: از, تا, and where relevant بیوپاری, زمیندار, گاہک, قسم مال, شہر, اڈا. From v1.34 a new بکری has no زمیندار and no اڈا, so those filters find old بکری only. [C] Umar (06-11-2026, v1.29): the sales reports R2, R3, R4, R7, R10 and R27 also take گروپ (پھل or سبزی) in place of one item, so "is mahine sabzi ki bikri" opens and prints with all the vegetables together.
FR-19.2 [A] Date shortcuts: آج، کل، یہ ہفتہ، یہ مہینہ، پچھلا مہینہ.
FR-19.3 [A] Every report: on-screen view, print (A4, in the print language of FR-7.36, page numbers, firm header, print time), save as PDF, Excel. Mac phase (v1.32): on the Mac, Print and PDF go through the native print dialog of the web view, whose PDF button saves a PDF, from the same print templates. One-click PDF is left out on the Mac. The Windows path does not change.
FR-19.4 [S] Open a row (click or `Enter`) to see its source بکری or voucher.
FR-19.5 [C] Totals (میزان) always shown. بنام and جمع totals match where applicable.
FR-19.6 [C] Numbers never flip next to Urdu text (`(16 @ 900)` stays correct), and Urdu text inside an English print (a note typed in Urdu) keeps its own direction. Every number on every print and PDF, in Urdu and in English, uses digits 0 to 9 with grouping: years like 2024, amounts like 10,000, dates like 06-11-2026. Never ۰ to ۹ (v1.32, D8 bug 3).
FR-19.7 [A] Amount in words on bills and statements, in the print language: 19,316 = "انیس ہزار تین سو سولہ روپے" or "Nineteen thousand three hundred sixteen rupees only". Urdu words always use لاکھ, کروڑ and ارب. English words follow the number grouping setting: thousand, million, billion for 1,316,040 style; thousand, lakh, crore for 13,16,040 style.
FR-19.8 [C] **Items in party ledgers.** Umar (11-10-2026): add the item. In R13 and R14, each sale line on a buyer's ledger shows its item, qty, packing and rate, then commission and لاگا, for example "بکری 1: کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160" with 12,880 بنام. Money paid on the spot shows after its sale line as جمع, marked موقع پر جمع (v1.32). Each seller credit shows the lot's item, qty, خام بکری and deductions. Cash lent shows "نقد ادھار" and its reason. Example E2E-33.
FR-19.9 [C] **Each buyer's اگراہی and the shop's total.** R20 has one row per buyer not in the dead amount. Its میزان always equals the dashboard tile گاہکوں کے ذمے. The shop's total اگراہی (tile کل اگراہی) = the R20 میزان + the بیوپاری کے ذمے total of R24 + the R38 میزان (v1.10). Example: after Events 1 to 5, اکرم 2,880 + ارشد 18,684 = 21,564; no seller owes and nobody is dead, so کل اگراہی is also 21,564.
FR-19.10 [C] **Print any day, any party.** Umar (14-10-2026): everything can be printed for one day or a date range, and for one party where that applies, to share with partners or the party. Every report, ledger, register and list screen (آج کی بکری, باقی مال, مارکیٹ سفر and خریداری, باغات, voucher lists, expenses, پھل, partner money, partner capital, ادھار رقم, الگ کھاتے) has the date filter (one day or from and to, with the shortcuts of FR-19.2) and a party filter (buyer, seller, partner, person) where it applies, and the same print, PDF and Excel actions (FR-19.3). Each print shows its filters (dates and party) in its header. Balance lists (R20, R24, R26, R32) take an "as of" date instead of a range; one day's collections are R23 for that day. The partner capital and ادھار رقم list screens print the same way. Examples: one buyer's goods on 10-10-26 (R5, buyer and date); one seller's whole history (R13 from the first date); all sales of one day (R2 and R6 for آج); one day's expenses (R29); one partner's month (R31); the whole day (R34). Example E2E-36.

FR-19.11 [C] **Large lists, page by page.** (FR-19.11 and FR-19.12 were added in v1.6.) Umar (18-10-2026): a list, ledger, register, report or analytics table loads its rows from the backend one page at a time (for example 100 rows) and draws only the rows on screen. Its totals come from the backend over the whole filter, never from the rows on screen. Print, PDF and Excel always hold every row. Speed: NFR-SPD-24. [C] (Umar, 30-10-2026, v1.8.) Under every such table a pager shows rows per page (25, 50 or 100, default 25, remembered per screen), "صفحہ 2 از 14" (Page 2 of 14), the rows shown out of the total, and the buttons First, Previous, Next and Last; `Page Up` and `Page Down` move a page. A new filter starts again at page 1. Example: a buyer's ledger of 10 years opens with its first 25 rows and its closing balance at once; Next shows rows 26 to 50.

FR-19.12 [C] **Party day sheet (R35), a compulsory print.** Umar (18-10-2026): the collector takes a slip for each party to the shops. For a chosen day, for each party:
- Buyers: previous balance (at the start of the day), bought that day (each line: item, qty, rate, amount with commission and لاگا), paid on the spot (موقع پر جمع), paid that day (وصولی), balance remaining, and an empty column for the collector's note.
- Sellers: previous balance, sold that day (each lot: item, qty, credit), paid on the spot (نقد), paid that day (ادائیگی), balance remaining.
- One slip for one party, or all parties with a balance or an entry that day at once. The amounts the collector brings back are entered with the collection round (FR-7.61).
- [C] (Umar, 20-10-2026) **The client's own layouts.** The client showed his old prints of 18-10-26, and R35 follows them:
  - **One party's slip.** (v1.12, Umar 31-10-2026; it replaces the v1.7 slip of the client's layout.) Two slips, a buyer's and a seller's, each with the print header of FR-19.13, the party (short code and name with tag) and the date. All figures come from the backend; the screen adds nothing.
    - **Buyer's day slip.** A goods table of every sale line of that day, from every بکری: قسم مال, پیکنگ, تعداد, ریٹ, رقم (the line's goods value, qty × buyer rate, C1). Lines of the same item and packing stay together; when an item has more than one line, a row "<item> کل" gives its total نگ and amount. Then, each on its own line: مال کی رقم (goods value), کمیشن (buyer commission), لاگا, آج کا کل (today's total = goods value + commission + لاگا), دیگر اندراج (other entries that day, such as cash lent or a جنرل line; shown only when not 0), آج ادا (paid today: موقع پر جمع and وصولی; left blank for the collector when 0), آج کا باقی (today's total + other − paid), سابقہ رقم (balance at the start of the day), نیا بقایا (new balance = سابقہ رقم + آج کا باقی), and the total نگ. A line that can be below 0 (صافی بکری, دیگر اندراج, the balances) is shown with its side word. An item total is per item, also across packings.
    - **Seller's day slip.** A goods table of every sale line of his lots that day, each lot by its name and sorted as FR-12.12 (v1.34): قسم مال, پیکنگ, تعداد, ریٹ (seller rate), رقم (seller amount), with the same item totals. Then: خام بکری (gross), each deduction on its own line and only when not 0 (کمیشن, کرایہ, مزدوری, منشیانہ, فون, کرایہ سٹور), آج کی صافی بکری (his credit of the day, C8), دیگر اندراج (fruit credit, جنرل; only when not 0), آج ادا (paid to him today: نقد on his lots and ادائیگی), آج کا باقی, سابقہ رقم and نیا بقایا, each balance with its side word (بنام: he owes the shop; جمع: the shop owes him).
    Example: a buyer owes 15,000 at the start of the day, buys goods with commission and لاگا worth 5,000, and pays 2,000: آج کا باقی 3,000, نیا بقایا 18,000.
  - **All parties as one table** (the all-buyers balance sheet; sellers in a second table): کوڈ (the short code, FR-7.39), پارٹی (name with tag), سابقہ, بنام (everything debited that day), جمع (everything credited that day), بقایا, and a blank column for the collector's note; sorted by Urdu name; a grand total row. Each row holds: سابقہ + بنام − جمع = بقایا. The buyers' total بقایا equals the میزان of R20 (FR-19.9). A person in the dead amount shows only on a day he has an entry, marked ڈوبی; his row is left out of the grand total, and a second total row adds it (v1.10).

Example (E2E-43): اکرم on 10-10-26: previous 0 + bought 12,880 − paid on the spot 0 − paid that day 10,000 = 2,880 remaining. His slip (v1.12) shows one row "کیلا, پیٹی, 20, 600, 12,000", then مال کی رقم 12,000, کمیشن 720, لاگا 160, آج کا کل 12,880, آج ادا 10,000, آج کا باقی 2,880, سابقہ رقم 0, نیا بقایا 2,880, total نگ 20 (the buyer's slip ends with the total نگ). رحیم's slip shows کیلا, پیٹی, 20, 600, 12,000 and 30, 580, 17,400, the row کیلا کل 50, 29,400; خام بکری 29,400, کمیشن 1,764, کرایہ 3,000, مزدوری 250, منشیانہ 50, فون 20, آج کی صافی بکری 24,316, آج ادا 9,316 (نقد 5,000 + ادائیگی 4,316), آج کا باقی 15,000 جمع, سابقہ رقم 15,000 بنام, نیا بقایا 0. In the buyers' table: اکرم 0, 12,880, 10,000, 2,880; ارشد 0, 18,684, 0, 18,684; شاہد 0, 12,800, 12,800, 0; total 0 + 44,364 − 22,800 = 21,564 = R20. The client's own slip of 18-10-26 reads the same way: سابقہ رقم 33,287 + تازہ بنام 25,574 = ٹوٹل رقم 58,861, with وصولی blank.

FR-19.13 [C] **Clear prints everywhere.** (Added in v1.7.) Umar (20-10-2026): R5 and the all-parties table of R35 follow the client's own layouts (FR-19.12, R5). Every other print (reports, ledgers, statements, expenses, partners, gardens, trips, fruit, analytics) uses our own clearest layout, the same on every print:
- Header: the firm name, the mandi name, the trade mark in a box, the owners' names and the munshis' names with their phones (Section 21, v1.12; active munshis from the staff list, v1.23), then the print's title, its filters (dates and party, FR-19.10) and the print date and time.
- Body: every column labelled; a party shown with its short code and its name with tag; numbers with equal-width digits, each column of numbers and its total centred under its heading (NFR-USE-19, v1.9); a running balance where the print has one.
- Footer: a totals row under its columns (FR-19.5), amounts in words where FR-19.7 asks for them, and "Page x of y".
Example: R29 for October prints "Maher Adnan and Brothers", the three contacts and the trade mark, then "خرچہ رپورٹ، 01-10-26 تا 31-10-26", the heads with their totals, and the میزان.

### 19.2 Report catalogue

**Sales** (filters as in the old sales report box [S]: از، تا، بیوپاری، زمیندار، قسم مال)

| ID | Report | Urdu | Columns | Source |
|---|---|---|---|---|
| R1 | Sale book print | بکری بک | One بکری in full (seller bill); a seller's lots sorted by item, then lot counter (FR-12.12, v1.34) | [S] |
| R2 | Full sales register | مکمل بکری | بکری no, lot name (v1.34), buyer, city, item, (qty @ rate), amount | [S] |
| R3 | Cash sales | نقدی | R2 for every line with money paid on the spot, with the columns paid and left. A part-paid line is on both R3 and R4 (v1.32) | [S] |
| R4 | Credit sales | ادھاری | Every line with something left: بکری no, buyer, (qty @ rate), amount, paid, left (v1.32) | [S] |
| R5 | Buyer bills | ادھاری تفصیل (Print All) | One row per buyer of the day, also a buyer who paid in full, with 0 left (v1.32), as in the client's print of 18-10-26 (v1.7): serial, party (name with tag), تفصیل with each line written "(item qty . rate)", for example "(سیب 8 . 2600)", رقم (value), رقم بعد لاگا (value + buyer commission + لاگا, C5), paid on the spot, left. Example: انس, (آم 50 . 900), 45,000, 48,100, paid 10,000, left 38,100. Print All = one slip per buyer. | [S][C] |
| R6 | Seller summary | بکری تفصیل / روزنامچہ بکری | Per lot: بکری no, lot name (v1.34), تعداد, بیوپاری, خام بکری, کل خرچہ, صافی بکری, کرایہ, کمیشن, مزدوری, منشیانہ, نقد, فون | [S] |
| R7 | Item-wise sales | قسم مال وار بکری | Item, نگ, gross, avg rate, seller comm, buyer comm, لاگا, total comm (as view A) | [C] |
| R8 | (retired in v1.34) | | | |
| R9 | (retired in v1.32) | | | |
| R10 | Stock | مکمل سٹاک / باقی مال | Lot name (v1.34), date, seller, item, received, sold, written off, taken as fruit, balance, days | [S] menu, columns [A] |

**Accounts**

| ID | Report | Urdu | Columns | Source |
|---|---|---|---|---|
| R11 | Daily journal | روزنامچہ | Date, voucher, account, narration, بنام, جمع, میزان | [S] |
| R12 | Cash book | روکڑ | Opening, receipts, payments, closing | [S] |
| R13 | Party ledger | کھاتہ | Opening, entries with their items (FR-19.8), running balance, closing | [S] |
| R14 | Party statement | فرد حساب | Opening, total bought or sold, paid, balance; option "صرف بیلنس" (balance only) or with detail. From-date defaults to the first date of data. | [S] |
| R15 | Vouchers list | ووچر | Vouchers by type and date | [S] |
| R16 | Account index | انڈکس | Accounts with code, city, balance | [S] |
| R17 | Head balances | مدوں بیلنس | Balance per head and control | [S] |
| R18 | Trial balance | ٹرائل بیلنس | Every account, totals equal | [S] |
| R19 | Opening trial | اوپن ٹرائل | Opening balances only | [S] |

**Collections, payments, advances**

| ID | Report | Urdu | Columns | Source |
|---|---|---|---|---|
| R20 | Buyer dues | گاہکوں کے ذمے رپورٹ | Buyer, city, phone, balance, oldest open bill date, days, aging 0-7, 8-30, 31-90, 91+ days; people in the dead amount left out; میزان = the tile گاہکوں کے ذمے (FR-19.9) | [C] |
| R21 | Collection by city | اگراہی بحساب شہر | R20 grouped by city, with city totals; people in the dead amount left out; total = the tile گاہکوں کے ذمے | [S] |
| R22 | Collection by collector | اگراہی بحساب اگراہی والا | R20 grouped by collector; people in the dead amount left out; total = the tile گاہکوں کے ذمے | [S] second menu item (blurry), meaning [A] |
| R23 | Receipts register | وصولی رجسٹر | Date, voucher, buyer, amount, collector | [A] |
| R24 | Seller payables | بیوپاریوں کو ادائیگی باقی | Seller, open payable (FR-7.59), open advance, ledger balance (open payable − open advance), last بکری date; separate list "بیوپاری کے ذمے" for what sellers owe beyond their open advances (FR-15.9), people in the dead amount left out | [C] |
| R25 | Payments register | ادائیگی رجسٹر | Date, voucher, account, amount | [A] |
| R26 | Advances | پیشگی کھاتہ | Seller, given, returned, adjusted, open, date; people in the dead amount left out | [C][S] |
| R27 | Bill series | بیجک سیریز | Seller bills (بیجک) in number order for a date range: بکری no, date, seller, خام بکری, کل اخراجات, صافی بکری. Under a Group filter, each bill shows only that group's lots, and the print header says so (v1.31). Each lot shows its name (v1.34). | [S] menu (shown as "بجک سیریز"), columns [A] |

چکتا buyers are never in R20 to R26 (FR-13.6). Old open amounts on 0030 stay in the books and the trial balance, and do not show here (FR-13.9).

**Business**

| ID | Report | Urdu | Columns |
|---|---|---|---|
| R28 | Income report | آمدن رپورٹ | Each income account by day or month |
| R29 | Expense report | خرچہ رپورٹ | دکان خرچہ by head; ذاتی خرچہ taken and returned by partner |
| R30 | Profit and partner split | منافع و تقسیم | Income − shop expenses = profit; each partner's share |
| R31 | Partner statement | شراکت دار کھاتہ | Every dated movement and the running کل رقم (FR-17.9), Section 17.3 |
| R32 | Money lent | ادھار رقم رپورٹ | Person, staff or not, lent, returned (cash, salary), open, oldest open date, return by, days past it; میزان |
| R33 | Trip profit or loss | تجارتی منافع / نقصان | Per trip and per purchase: date, item, bought (نگ, cost), sold (نگ, sales money after commission, seller commission, total commission = seller + buyer commission + لاگا, v1.8), cost of goods sold, written off, left (نگ, cost), profit or loss; each partner's %, money in and money back; by item, day and month; میزان |
| R34 | Day summary (the day total sheet) | روزانہ خلاصہ | For one day (or each day of a range): بکری count, خام بکری and نگ; commission in its own lines (seller, buyer, لاگا, total); buyers: billed today, cash received (موقع پر جمع and وصولی), left; sellers: net sale today, cash paid out; then a short buyers' table (code, buyer, سابقہ رقم, آج کا کل, آج ادا, نیا بقایا) and a short sellers' table (code, seller, سابقہ رقم, آج کی صافی بکری, آج ادا, نیا بقایا), each with a total row (v1.12); cash opening, cash in, cash out, closing; receipts, payments, advances, expenses, partner money, capital, money lent; trips (bought, sold), gardens (paid, money in), fruit taken; partner shares; buyers' dues (گاہکوں کے ذمے), sellers' dues (بیوپاریوں کے ذمے), dead amount (ڈوبی رقم), total dues (کل اگراہی); lots in باقی مال |
| R35 | Party day sheet | پارٹی کی روزانہ پرچی | For one day (FR-19.12): one buyer's or seller's day slip (v1.12; a seller's lots sorted as FR-12.12, v1.34), or all parties as one table (code, party, سابقہ, بنام, جمع, بقایا, collector's column) with a grand total |
| R36 | Gardens | باغات رپورٹ | Per garden: installments and costs (shop and pockets), sales money (our shop, other shops), commission on its goods, running balance, rent still due, closed result; each partner's %, money in and money back |
| R37 | Fruit taken | پھل رجسٹر | Date, for whom, item, qty, rate, value, source; by partner, item, day, month or year; میزان |
| R38 | Dead amount | ڈوبی رقم رپورٹ | Person, type (گاہک, بیوپاری, ادھار رقم), code, reason, date moved, owed then, recovered since, still owed, last recovery date; filter: still owed or all; میزان = the tile ڈوبی رقم (FR-10.10) |

---

## 20. Module: بیک اپ (Backup and restore) [C]

Quality targets (frequency checks, encryption, provider) are in the NFR and Tech Stack documents.

FR-20.1 [C] Backups happen by themselves. Nobody needs a USB stick to keep data safe.
FR-20.2 [A] A local copy every 15 minutes while data is changing.
FR-20.3 [C] A cloud copy at closing time (setting, default 9:00 PM) and when the app is closed (the app finishes the backup before it exits, showing progress).
FR-20.4 [A] A USB copy at the same time, if a USB drive is plugged in.
FR-20.5 [A] A readable daily pack in the cloud: روزنامچہ PDF, and spreadsheets for buyers, sellers, دکان خرچہ, ذاتی خرچہ, partner shares.
FR-20.6 [A] No internet: the cloud copy waits and retries by itself. The status bar shows the last good cloud backup, red if older than 24 hours.
FR-20.7 [A] Each backup is checked after it is made. A failed check shows an alert on the dashboard.
FR-20.8 [A] Backup screen: list of backups (date, size, place, status), بیک اپ ابھی لیں (backup now), بحال کریں (restore), test restore.
FR-20.9 [A] Restore: pick a backup, see its date and totals, confirm. The current data is backed up first, then replaced. A warning says entries made after that backup will be lost.
FR-20.10 [A] USB and cloud backups are locked with a key that is shown once at setup and printed for the Owner.
FR-20.11 [C] **Restore on a new computer, from the first screen.** (Added in v1.24.) Umar (04-11-2026): if the laptop is lost, a new laptop with the app must bring back all the books and carry on where the shop left off.
- The very first screen of a new install offers two choices before any setup: نئی دکان شروع کریں (start a new shop, the wizard of NFR-INS-2) and پچھلا ڈیٹا واپس لائیں (bring back my data).
- "Bring back my data" asks where the copy is: the cloud or a USB drive or file. For the cloud it asks the cloud address, the bucket and the two cloud keys (the same fields as Settings), then lists the copies, newest first, with date and size.
- The Owner picks a copy (the newest is chosen) and types the backup key from the printed sheet (FR-20.10). The app checks the copy, then shows its date and its totals (accounts, entries, the newest entry date, total debit and credit) before anything is restored.
- Restore puts the books in place exactly as they were in that copy: every section, every setting, partners, rules, learned words, the open and closed days, the question log. The tamper checks (NFR-SEC-6) pass on the new computer. The cloud settings and the backup key are kept on the new computer, so the daily copy goes on to the same place.
- The Owner then signs in with his old user name and password. If he has forgotten the password, his printed recovery code works (FR-8.5). Keys that never travel in a copy (the OpenRouter key) are asked for again in Settings.
- Nothing on the new computer is lost if the restore fails half way: it starts again from the first screen.
Example: the laptop is stolen on 15-03-2028. On a new laptop the Owner installs the app, presses پچھلا ڈیٹا واپس لائیں, types the Cloudflare details and the printed key, picks the copy of 14-03-2028 21:00, sees 2,413 accounts and the totals, restores, signs in, and the dashboard shows the same balances as on 14-03-2028 at closing (NFR-REL-4: under 30 minutes).

---

## 21. Module: ترتیبات (Settings), day close and audit

| Group | Settings |
|---|---|
| Firm | Firm name (Urdu, English), mandi name, address, phones, logo; the people printed in every header, up to 6, each a name (Urdu and English), a role (مالک owner or منشی munshi) and a phone (optional), for example "منشی آصف بھٹی 0300-4058860"; the owners come first, then the munshi (v1.12, Umar 31-10-2026). [C] From v1.23 (Umar, 03-11-2026), every **active** staff member whose job is منشی also prints, after the people typed here, with his phone, at most 3 of them and at most 6 people in all. A munshi made inactive in the staff list leaves every print at once, and a new munshi added as active appears at once. A person who is both typed here and an active munshi prints once; trade mark text (for example "RH"), printed in a box (FR-19.13, v1.7) |
| Codes | Type numbers: seller 001, buyer 002, چکتا کھاتہ 003, item 004, packing 005, staff 006, market trip 007, garden 008 (fixed, FR-10.2) |
| Charges | Global defaults: seller commission 6%, buyer commission 6%, لاگا 8 per نگ on fruit (as an item-group rule). مزدوری, منشیانہ, فون have no default. The commission agent changes any of them (Section 11). |
| Overrides | Allow line-level commission and لاگا override (on / off, default on, FR-11.10) |
| Checks | Credit limit: warn or block. Rate check (FR-7.55) on or off, with its percent (default 30%). |
| Day | Closing time; auto day close on or off |
| Security | Partner code (FR-17.11): set, change, reset |
| سوال پوچھیں (Ask the app) | On or off; the OpenRouter key (kept in Credential Manager, shown as •••• and its last 4 characters); the model (default `openai/gpt-6-luna`); test the connection; today's count of 20; the question log (FR-7.85, v1.16) |
| Backup | Cloud account details, local backup folder, USB on / off, closing time |
| Printing | Paper size per document (A4, A5, 80 mm), printer, copies. Every print uses digits 0 to 9 (FR-19.6, v1.32). |
| Language | Language of the backup pack (default اردو); the app language is switched with FR-7.30. Number grouping (1,316,040 or 13,16,040). Default text size. Learned words list (FR-7.26): view and remove. |

Rounding (Section 22) and the split leftover rule (C10) are fixed rules, not settings.

FR-21.1 [A] **Day close (دن بند کریں).** Closes the oldest open day (and can close several days up to a chosen date, in order). It shows the day summary first: بکری count, خام بکری, commission, partner shares, cash in hand, trial balance difference (must be 0), lots with باقی مال. Confirm moves the "closed up to" date forward and starts the cloud backup.
FR-21.2 [A] **Reopen.** Reopening needs a reason. Reopening a date reopens that date and every later date (the "closed up to" date moves back to the day before it). It is written to the audit log. A month that has been month-closed cannot be reopened until its month close is reversed by the Owner.
FR-21.3 [A] **Audit log.** Every create, edit, cancel, login, rule change, share change, opening change, day close, reopen, month close and restore: user, time, screen, record, old value, new value, reason. Read-only. Owner can filter it.
FR-21.4 [C] **Reopen and fix in one step.** (Added in v1.13.) Umar (31-10-2026): an entry of a closed day is corrected by reopening, fixing and closing again. On such an entry the Edit button reads "دن کھولیں اور درست کریں" (Reopen the day and fix). It asks for the reason, reopens that date and every later closed date (FR-21.2), opens the entry for editing, and after the save, or when the user leaves without saving, closes the same days again in order with the checks of FR-21.1. If a check fails, the days stay open, and the attention list (FR-7.44) says which. A month that is month-closed is refused until its month close is reversed (FR-21.2). Reopen and close again are both in the audit log.

---

## 22. Calculation reference (all formulas in one place)

Money rule: every money input and every stored money result is a **whole number of rupees**. Rates are whole rupees. Percent values can have up to 2 decimals (2.5%). Weights can have up to 3 decimals (kg). Any calculation that is not a whole rupee is rounded once, at the point shown, to the nearest rupee, half up (337.5 → 338, 337.4 → 337).

| # | Formula | Example |
|---|---|---|
| C1 | Line value (by نگ) = qty × rate. Line value (by weight) = round(weight kg × rate per kg). | 20 × 600 = 12,000. 12.345 kg × 100 = 1,234.5 → 1,235. |
| C2 | Percent charge = round(base × percent ÷ 100) | 12,000 × 6 ÷ 100 = 720; 5,630 × 6 ÷ 100 = 337.8 → 338 |
| C3 | Per-نگ charge = qty × value (no rounding needed) | 10 × 20 = 200 |
| C4 | Per-lot charge = value, once per lot | منشیانہ 50 |
| C5 | Buyer total = buyer value + buyer commission + لاگا | 12,000 + 720 + 160 = 12,880 |
| C6 | Lot deductions = freight (if paid by آڑھتی) + seller commission + labour + munshiana + نقد + store rent + phone | 3,000 + 1,764 + 250 + 50 + 5,000 + 0 + 20 = 10,084 |
| C7 | Lot net (صافی بکری) = lot gross − lot deductions | 29,400 − 10,084 = 19,316 |
| C8 | Seller credit in a posting = seller amounts − seller commission − lot charges of that posting except نقد. (Lot charges only in the first posting.) | 29,400 − 1,764 − 3,000 − 250 − 50 − 0 − 20 = 24,316 |
| C9 | Shared income of a posting = sum of all income lines in it (seller comm + buyer comm + لاگا + فون + ریٹ فرق) | 1,764 + 1,764 + 400 + 20 = 3,948 |
| C10 | Partner share = round down (shared × share% ÷ 100) to whole rupees; leftover to the largest share (tie: partner created first) | 3,948 → 1,579 + 1 = 1,580; 1,184; 1,184 |
| C11 | Expense share = C10 on the expense amount, then made negative | 3,000 → −1,200 / −900 / −900 |
| C12 | Partner profit share = income shares − expense shares | 2,196 − 1,200 = 996 |
| C13 | Buyer balance = opening + Σ debits − Σ credits | 0 + 12,880 − 10,000 = 2,880 |
| C14 | Aging days = today − date of the oldest open bill after matching | |
| C15 | Rounding = nearest whole rupee, half up. Fixed rule. | 337.8 → 338 |
| C16 | Lot balance qty = received − sold − written off − taken as fruit (FR-16.11) | 50 − 50 − 0 − 0 = 0 |
| C17 | (retired in v1.34: there is no header freight) | |
| C18 | Average rate (display only) = round(خام بکری ÷ نگ) | 29,400 ÷ 50 = 588 |
| C19 | Rate difference = buyer value − seller amount (can be negative) | |
| C20 | Partner خالص = profit share − (money taken − money returned) | 996 − (2,000 − 0) = −1,004 |
| C21 | Cost of trip goods sold (in بکری or another shop) or written off = `split_by_qty` of the purchase's cost left over [qty taken, qty left]: round each part down, leftover to the larger qty (tie: the qty taken). When the last نگ is taken, all the cost left goes with it. Stored and shown for the profit or loss; never posted (FR-12.26). | 10,400 over 20 نگ, 15 sold: 10,400 × 15 ÷ 20 = 7,800; the last 5 take 2,600 |
| C22 | Partner کل رقم = opening capital + capital invested − capital taken back + Σ his income shares − Σ his expense shares − money taken + money returned + his garden, trip and fruit account (جمع − بنام) | ب after E2E-35: 0 + 50,000 − 0 + 1,646 − 900 − 0 + 0 + 0 = 50,746. الف after E2E-40: 3,200 − 33,000 + 39,840 = 10,040 |
| C23 | A garden or trip payment = `split_by_bps`(amount, its shares). A part from the shop's cash posts on that partner's 0604; a part from his pocket is recorded only (FR-12.30) | 30,000 at 60 / 40: 18,000 / 12,000 |
| C24 | A trip cost over its purchases with goods left = `split_by_qty`(cost, نگ bought of each), added to each one's total cost and cost left (FR-12.31) | 4,000 over one purchase of 80 نگ: 4,000; 1,000 over 30 and 20 نگ: 600 and 400 |
| C25 | A trip's sales money to its partners = `split_by_bps`(seller credit of its lot, or money from another shop, its shares) | 8,460 at 40 / 30 / 30: 3,384 / 2,538 / 2,538 |
| C26 | A garden's closing = `split_by_bps`(the garden account's balance, its shares) | 66,400 at 60 / 40: 39,840 / 26,560 |
| C27 | Fruit for the shop's guests = `split_by_qty`(value, 1 for each partner active on the date), in the order the partners were created | 3,000 over 3: 1,000 each; 1,000 over 3: 334 / 333 / 333 |

---

## 23. Edge cases (consolidated)

| # | Case | Required behaviour |
|---|---|---|
| E1 | One truck, several sellers | Several lots, each with its own seller and its own "freight paid by" |
| E2 | One seller, several items on one truck | Several lots, same seller |
| E3 | Lot not fully sold today | Warning; save allowed; balance in باقی مال; sold later as a later sale (FR-12.17) |
| E4 | Goods spoiled or returned to seller | Write off the lot balance with a reason (FR-12.16) |
| E5 | Buyer returns goods after sale | جنرل entry for now [A]; dedicated return screen later |
| E6 | Seller already paid the driver | Lot "freight paid by بیوپاری": freight not deducted or posted |
| E7 | Same buyer on many lines | Allowed; buyer slip groups his lines |
| E8 | Rate typed wrong (6000 instead of 600, or 60 instead of 600) | Rate check warning (FR-7.55) |
| E9 | Partner shares not 100% | Posting cannot save; message points to partner setup |
| E10 | Same person is both seller and buyer | Two accounts (one per code series), linked; combined balance shown on his ledger [A] |
| E11 | Buyer pays more than he owes | Extra kept as his credit, settles his next bills |
| E12 | Payment to seller more than payable | Warning; seller then owes the firm ("بیوپاری کے ذمے"); not an advance |
| E13 | Seller's advance bigger than his sales | The rest stays as his open advance; his balance shows بنام (FR-15.9) |
| E14 | بکری edited after the buyer paid part of it | Receipts re-matched oldest first; aging updated |
| E15 | بکری cancelled after the buyer paid | Payment stays on his account as credit; re-matched to his next oldest bills |
| E16 | Back-dated entry | Only after the "closed up to" date; earlier dates need the Owner to reopen |
| E17 | Rule changed mid-day | New and changed lines use the new rule; saved and unchanged lines keep theirs |
| E18 | Share changed | Only from today or later; postings split by the share on their own date |
| E19 | Power cut while typing | Draft restored on next start |
| E20 | Power cut during save | Either the whole save happened or none of it |
| E21 | Master record in use | Cannot delete; can make inactive |
| E22 | (retired in v1.34: the بکری header has no truck number) | |
| E23 | Item sold by weight | Rate per kg; value = round(weight × rate); per-نگ charges use the نگ count; percent charges use the value |
| E24 | Zero or negative qty or rate | Not allowed |
| E25 | (retired in v1.32: every چکتا line is paid in full, and his phone is optional) | |
| E26 | Restore of an older backup | Warning listing what will be lost; safety copy first |
| E27 | Rounding leftover in a split | Goes to the largest share; totals always exact |
| E28 | Lot net negative (deductions more than gross) | Allowed with a red warning; seller ends up owing |
| E29 | A code typed that is already used | Refused; the next free code is suggested (FR-10.3). |
| E30 | Two چکتا buyers on one day | Each is kept by name; neither has a balance or a due (FR-13.2, FR-13.6, E2E-21) |
| E31 | Money typed with decimals ("850.50") | Not allowed: "رقم پورے روپوں میں لکھیں" |
| E32 | Language switched in the middle of an unsaved بکری | Values, selected row, cursor and draft stay; only words, names and direction change (FR-7.33) |
| E33 | English letters typed in a name field and the first suggestion is wrong | User picks another suggestion or edits the word; the pair is learned (FR-7.26); saved names never change |
| E34 | A rare name not in the word list | Spelling rules still give suggestions; for exact spelling the user switches to اردو کی بورڈ with `Alt+U` |
| E35 | Urdu letters in the English name, or English letters in the Urdu name | Not saved; clear message (FR-7.28) |
| E36 | A short code that fits a buyer and a seller, in a field that takes any account | Both listed with their type (FR-7.39) |
| E37 | A code or short code with no account | Message, and `F4` adds an account with that code (FR-7.42) |
| E38 | Two accounts with the same name | The tag tells them apart (اسلم BA and اسلم); search rows and the party card also show father or firm, city, phone and balance (FR-7.40, 10.2) |
| E39 | (retired in v1.34) | |
| E40 | Print language differs from the app language | Print uses the chosen language and the names in that script; notes print as typed (FR-7.36, FR-7.25) |
| E41 | A record with only one of its two names (possible only after the Phase 2 import) | Shown with the name it has, marked, and listed in the attention list until fixed (FR-7.44) |
| E42 | `20+25` typed in a money field; `10-20` typed | 45 accepted; a result below 0 refused (FR-7.48) |
| E43 | Undo tried after the day was closed | Not offered (FR-7.47) |
| E44 | Seller wants his money now and will repay the advance later | Untick "پیشگی سے کاٹیں" on the lot; advance stays open (FR-12.23) |
| E45 | Seller returns part of his advance in cash | وصولی on his account adjusts the advance (FR-15.12) |
| E46 | Staff member repays a loan from his salary | ادھار رقم واپسی by تنخواہ سے کٹوتی; that part is a split shop expense (FR-16.4) |
| E47 | Buyer borrows cash | ادائیگی to the buyer with a reason; it becomes his bill (FR-15.11) |
| E48 | Partner returns money he took | ذاتی خرچہ واپسی; adds back to his خالص (FR-16.3) |
| E49 | A name has a spelling made only by rules | Marked "ہجے دیکھ لیں"; saved only after the user confirms or fixes it (FR-7.66) |
| E50 | Trip goods sold below their cost | Allowed; the loss shows in red on the purchase, the trip and R33; each trip partner's total already carries his % of it (FR-12.27) |
| E51 | Trip goods spoil | Write off on the purchase; nothing posts; the trip's result shows the loss (FR-12.28) |
| E52 | Trip goods partly sold | The rest stays in the purchase's stock for a later بکری or another shop; the trip stays open (FR-12.26, FR-12.33) |
| E53 | A partner takes back more capital than his balance | Warning; save allowed (FR-17.8) |
| E54 | A الگ کھاتہ goes minus | Allowed; shown as minus with its entries (FR-16.6) |
| E55 | A shop expense for guests (tea, food, drinks) | On the خرچہ screen, type دکان خرچہ with a head such as مہمان نوازی; split between partners by share (16.2). A partner's personal spending (a wedding, his home) is type ذاتی خرچہ with that partner, and comes off his total only (16.3, C22). |
| E56 | An expense saved as the wrong type | Before save: change the type; the date and amount stay (FR-7.67). After save: edit it while its day is open; the old posting is reversed and the right type is posted (FR-15.3). |
| E57 | An ادائیگی to a seller who has nothing payable | A note offers to make it a پیشگی; nothing changes unless the user presses the button (FR-7.68) |
| E58 | Any entry that takes cash in hand below zero | Amber note while typing; on save a confirm box ("کیا یہ اندراج درست ہے؟"); Yes saves, No goes back (FR-6.7) |
| E59 | A return-by date before the entry date | Refused (FR-16.9) |
| E60 | A seller with an opening credit is paid | It is an ادائیگی against that credit, not a پیشگی (FR-7.59) |
| E61 | A partner who is not in a garden or trip | 0% of its result; he still gets his normal share of the commission on its goods (FR-12.29) |
| E62 | A partner pays his part of a garden or trip from his own pocket | Nothing posts and his total does not change; at closing (garden) or on sale (trip) he gets his % of its money: his money back plus or minus his share (FR-12.30, FR-12.38) |
| E63 | Every part of a payment is from pockets | No journal entry; the payment is recorded on the garden or trip (FR-12.30) |
| E64 | A garden closed with no sales | Its account is 0; nothing is credited; each shop-cash payer keeps the deduction, so the whole cost is the loss (FR-12.38) |
| E65 | An entry on a closed garden, or a new purchase or cost on a closed trip | Refused (FR-12.31, FR-12.38) |
| E66 | Changing a garden's or trip's shares after its first entry | Refused (FR-12.29) |
| E67 | Fruit taken from a lot is more than its balance | Refused (FR-16.11) |
| E68 | Guest fruit that does not divide equally | The leftover rupee goes to the partner created first (C27) |
| E69 | A partner made inactive while in an open garden or trip | Refused until the garden is closed or the trip is closed [A] |
| E70 | A list of thousands of rows | Loads page by page; totals over the whole filter; prints every row (FR-19.11) |
| E71 | A person who owes dies or leaves the business | Move him to ڈوبی رقم: his account is inactive, the money stays owed and in the total اگراہی; a recovery is a وصولی (FR-10.10) |
| E72 | A sale, payment or advance typed for a person in the dead amount | Refused as on any inactive account; only a وصولی, or an ادھار رقم واپسی, is allowed (FR-10.10) |
| E73 | A person in the dead amount comes back | دوبارہ فعال کریں with a reason; his balance returns to the normal tiles (FR-10.10) |
| E74 | A record is missing from a list in the middle of an entry | Added on the spot from the list and chosen at once; the entry is kept (FR-7.69) |
| E75 | The name added on the spot is like one already in the list | The similar records show under the name; one can be picked instead (FR-7.69, FR-10.7) |
| E76 | Someone tries to change the date of a saved entry | Not possible; cancel and enter it again on the right day (FR-7.70) |
| E77 | An entry of a closed day has a mistake | "دن کھولیں اور درست کریں": reopen, fix, close again (FR-21.4) |
| E78 | A trip purchase is wrong but some of its goods are sold | Edit refused with the reason; the sale is edited or cancelled first (FR-7.70) |
| E79 | A new buyer is needed after the day is closed | Saved at once, with his opening balance if he has one, posted on its date (FR-10.11, v1.22). Only a changed opening of an existing account needs an open date. |
| E80 | Wrong partner code typed many times, or the code is forgotten | Waits as FR-8.2; reset with the Owner's password or the recovery code (FR-17.11) |
| E81 | (retired in v1.32) | |
| E82 | A question names a party that two to five accounts match ("akram": اکرم ٹول ٹیکس and اکرم K) | The answer asks "کون سا؟" with a button for each; a click answers with no new question counted (FR-7.77) |
| E83 | A question names a party that matches nobody | "یہ نام نہیں ملا" with the three closest names as buttons; no figure (FR-7.77) |
| E84 | A period the app's list does not hold ("eid se pehle") | The answer asks for the dates; nothing is guessed (FR-7.77) |
| E85 | A question asks to save or change something | Refused; a button opens the entry screen with the party chosen and every amount empty; nothing saved (FR-7.79) |
| E86 | A question outside the books (tomorrow's rate, the weather, legal advice) | "This question is outside the books", with the nearest report, if any (FR-7.79) |
| E87 | A question needs a figure that no report gives | The calculator works it out from figures read for the question, with its working under ⓘ; if the books cannot give it (for example profit per kg where weights are not recorded), the answer says so (FR-7.75) |
| E88 | The two places of a checked figure disagree | The answer is refused; "A difference was found in the records"; an attention item (FR-7.76) |
| E89 | No internet, an error from OpenRouter, or no reply in time | A clear message; the quick questions still work; the rest of the app is not affected (FR-7.81, NFR-AI-7) |
| E90 | The 21st question of the day | Refused with the limit message, no request sent; the quick questions still work (FR-7.81) |
| E91 | A question about partner figures while the partner section is locked | The partner tools are refused; the "کوڈ درج کریں" box in the answer; nothing of a partner's figures is sent (FR-7.82) |
| E92 | A name in the books is written like an instruction to the model ("ignore the rules") | It is sent as data only. The model has no tool that writes, and every figure is checked, so it changes nothing; notes and reasons are never sent (FR-7.83, FR-7.76) |
| E93 | A question in English while the app is in Urdu, or in Roman Urdu while the app is in English | Answered in the app language (A80) |
| E94 | The model types a figure itself, or a wrong one | The answer is refused and written once more; if it fails again, the records are shown as plain tables with no figure from the model (FR-7.76) |
| E95 | An estimate with too little history (fewer than 3 full months) | The answer says how many full months the books hold and gives no estimate (FR-7.86) |
| E96 | The model keeps calling tools and does not finish | The 6th call goes out with the tools off, so the model answers from what it read (v1.19); only a size limit stops it, with "I could not finish reading the books; ask in smaller parts" and the tables already read; counted once (NFR-AI-5) |
| E97 | A question asks for a phone number or an address ("akram ka phone number") | The tools never give it; the answer offers to open his party card, where the app shows it (FR-7.83) |
| E98 | Advice is asked to change an entry or to set a rate ("akram ka rate barha dun?", "yeh entry mita dun?") | The change part gets the change refused answer (FR-7.79); the advice names no rate or figure of its own and points only to what the books show (FR-7.88) |
| E99 | Paid on the spot is more than the line's buyer total | Refused (`ERR_PAID_EXCEEDS_TOTAL`, with the most allowed) (FR-12.44) |
| E100 | A buyer pays part of his bill on the spot | The part paid comes into cash and what is left stays on his account; the line is on both R3 and R4 (FR-12.44) |
| E101 | A چکتا buyer is looked for in a voucher or ledger picker | Not listed: he has no ledger (FR-13.4, FR-13.6) |
| E102 | Backspace or Delete on a grid cell that is not being edited | An editable cell is cleared; a read-only cell does nothing; the screen stays and the leave box never opens (FR-7.91) |
| E103 | A quantity above what the lot has left | The error shows under the qty at once, the line has no figures, and `Enter` cannot leave the qty until it is fixed (FR-12.6) |

---

## 24. Assumptions register (to validate at the demo)

| ID | Assumption | Default | Ask the client |
|---|---|---|---|
| A1 | Seller commission is a percent, with rules by seller and item | 6% | Real rates per seller and item? |
| A2 | لاگا on vegetables | None (fruit only, 8 per نگ) | Is there any لاگا on vegetables? |
| A3 | Resolved in v1.7: a buyer pays commission and لاگا on every line; a seller pays commission only; the rate of each is set per item as rules (11.2). The client's real rates are typed in as rules at setup | | |
| A4 | Resolved in v1.32: چکتا کھاتہ is for buyers only (Section 13) | | |
| A5 | مزدوری, when a rule is made, is per نگ received | No default | On received or sold qty? |
| A6 | منشیانہ and فون are typed per lot | No default | Usual amounts, so a rule can fill them? |
| A7 | Buyer rate normally equals seller rate | Same rate | When do they differ? Who keeps the difference? |
| A8 | All income (commission, لاگا, فون, ریٹ فرق) is shared between partners | | Correct? |
| A9 | All shop expenses are split by the same shares | | Correct? |
| A10 | Partner names and shares | الف 40, ب 30, ج 30 | Real names and shares? |
| A11 | Split leftover to the largest share (tie: partner created first) | | OK? |
| A12 | One truck can carry several sellers' lots | Yes | Correct? |
| A13 | Resolved in v1.34: a new بکری's bill date is its entry date (12.2) | | |
| A14 | Resolved in v1.8: codes are a type number + a running number, first seller 0010, first buyer 0020 (FR-10.2) | | |
| A15 | Rounding to the nearest rupee, half up | | Correct? |
| A16 | Receipts settle the oldest bills first | | OK? |
| A17 | No separate freight register. The old menu's "کرایہ بک" button opens the بکری بک screen. | Freight on lots; transporters paid by ادائیگی | Correct? |
| A18 | Credit limit = warning | | Should it block? |
| A19 | Resolved in v1.2: one user (the Owner), no roles (Section 2) | | |
| A20 | English digits, 1,316,040 style | | Lakh style? |
| A21 | Bills on A4, slips on A5 or 80 mm | | Printer and paper? |
| A22 | Resolved in v1.3, changed in v1.6: goods are bought on market trips by some partners (12.10). "Stock" in the old menus is unsold goods, consigned or bought. | | |
| A23 | Second اگراہی report = by collector | | What was the second اگراہی report? |
| A24 | Old KACHA_KHATA table (empty) is not needed | | Was کچا کھاتہ ever used? |
| A25 | Old "Session End 31-12-2026" | | Does the old software stop then? |
| A26 | منشیانہ is paid to the munshi (a payable, not firm income) | | Correct? |
| A27 | All money is whole rupees, including rates | | Any rates with paisa (for example per kg)? |
| A28 | Lot charges are charged once, on the first posting | | Correct? |
| A29 | Buyer qty always equals seller qty on a line | | Is there ever a quantity difference? |
| A30 | Resolved in v1.6: there is no bank and no cheque; all money is روکڑ (Umar, 19-10-2026) | | |
| A31 | Partner shares cannot be back-dated | | OK? |
| A32 | Weight items: rate per kg, نگ still counted | | Which items are sold by weight? |
| A33 | English screens use English words, with Roman names for mandi words | Glossary English column (Laga, Munshiana, Chukta Khata, Adda, Nag) | Who will use English? Are these the words they use? |
| A34 | Prints use the app language, with a choice on each print | App language | Should buyer slips always be in Urdu? |
| A35 | Short code = the running number alone, by field type | `0` = 0020 in a buyer field | OK? |
| A36 | A rate is never filled by itself: only a hint and `Ctrl+D` | Hint only | Would they rather have the last rate filled in? |
| A37 | Urdu text fields start in English-to-Urdu typing | رومن سے اردو | Does the munshi prefer the phonetic keyboard? |
| A38 | A new sale line's qty starts at the lot's remaining qty | On | OK? |
| A39 | A seller's item and packing come from his last lot; مارکہ from his default, else his last lot | On | OK? |
| A40 | Attention list limits | باقی مال 2 days, dues 30 days, payables 7 days | Real limits? |
| A41 | Undo straight after save, as a reversal | 10 seconds | OK? |
| A43 | A loan deducted from salary is posted as that much تنخواہ shop expense | Expense head تنخواہ | Is that how salary loans are settled? |
| A44 | A return-by date is optional and only warns | Attention list only | Should overdue loans block new loans? |
| A45 | The cost of trip goods sold is the purchase's average cost per نگ (C21), also for items bought by weight | Average per purchase, by نگ | OK? |
| A46 | Resolved in v1.6: a trip's result goes only to the trip's partners, by the trip's % (12.10) | | |
| A47 | Freight, labour and other costs of a purchase are paid at once and added to its cost | Paid at once | Are they ever owed to the supplier? |
| A48 | Capital does not change profit shares; taking back more than the balance only warns | Warn | Should it block? |
| A49 | Resolved in v1.6: trip and garden goods pay the normal commission, which is firm income shared by the firm's shares; their result is after it (E2E-34: margin 1,200 = commission 540 + result 660) | | |
| A50 | On a lot whose seller is a الگ کھاتہ, the buyer still pays buyer commission and لاگا, which are firm income shared by partners; seller-side charges are 0 | As stated | Should the buyer side also be 0 for the khata's own goods? |
| A51 | The right-type notes only suggest; the user decides (FR-7.68) | Note with a button | Should a partner's name in "ادا کیا" block the save as دکان خرچہ instead? |
| A52 | Resolved in v1.5: Umar confirmed the cash check on every entry that takes cash below 0, as a confirm box (FR-6.7) | | |
| A53 | A trip purchase is paid at once (from the shop or a pocket); nothing is bought on credit | Paid at once | Do trips ever buy on credit? |
| A54 | Each partner's money source is set when the garden or trip is made, filled on every entry, and can be changed on one entry | As stated | OK? |
| A55 | A garden's or trip's shares cannot change after its first entry | Fixed | OK? |
| A56 | The first trip and every garden start with the firm's shares filled in; a later trip starts with the last trip's | As stated | OK? |
| A57 | Money from a sale in another shop is received into روکڑ | روکڑ | Does it ever stay with a partner? |
| A58 | Fruit from our shop comes from a lot in باقی مال; the lot's seller is credited qty × the typed rate, with no commission, and it does not adjust his advance; trip goods are not offered | As stated | OK? |
| A59 | Fruit for the shop's guests is split equally between the active partners | Equal | Should it be a different fixed %? |
| A60 | Another commission agent's shop is kept as a بیوپاری account, so the fruit money owed to him is paid with ادائیگی | Seller account | OK? |
| A61 | A garden's lot in بکری pays the normal seller charges; نقد and the advance tick are hidden on it | As stated | OK? |
| A62 | A trip closes by itself when nothing is left; a garden is closed by the Owner; a garden's closing can be undone only while its date is open; a closed garden's lot takes no later sale and gives no fruit | As stated | OK? |
| A63 | A partner cannot be made inactive while he is in an open garden or trip | Refused | OK? |
| A64 | A trip cost is spread when it is saved, over the purchases that still have goods left, and is entered after the goods are bought (FR-12.31) | As stated | Is the truck fare ever known before anything is bought? |
| A65 | Moving a person to the dead amount posts nothing: the partners keep the commission already counted on his goods, and his balance stays in the books | No posting | Should a dead amount that is never recovered be written off later as a loss shared by the partners? |
| A66 | A dead seller's open advance and payables, and a dead person's open loans, are netted into his dead amount and join کل اگراہی | Whole balance | Should only what is beyond his advance count? |
| A67 | Edits and cancels of a dead person's earlier postings are still allowed while their day is open, shown in R38 as corrections; a cancelled recovery lowers the recovered amount | Allowed | Should they wait until he is made active again? |
| A68 | Moving a person does not move his linked account (E10) | Not moved | Should both move together? |
| A69 | R34's buyers' and sellers' tables list only the parties with an entry that day; a person in the dead amount follows the R35 table rule (marked ڈوبی, out of the table total, a second total row adds him). The summary lines count everyone | Entry that day | Should the tables also list every party with a balance? |
| A70 | R34's seller sums and table count only seller accounts (control 03011); garden, الگ کھاتہ and own-goods lots have no seller ledger | Sellers only | Should gardens and khatas show too? |
| A71 | The seller's day slip shows his ledger balance; with the advance tick off it can differ from his payable (FR-7.59) | Ledger balance | OK? |
| A72 | On the seller's day slip and in R34, "paid today" is نقد and ادائیگی only; a پیشگی given to the seller that day shows in "other entries" | Other entries | Should a same-day advance count as paid today? |
| A73 | On a day slip, a line of an item sold by weight shows the kg, the rate per kg and the stored amount (12.345 کلو @ 100 = 1,235) | Kg shown | OK? |
| A74 | In an edit, the seller of a lot stays fixed while a later sale, a write-off or fruit taken rests on that lot; to change him, those entries are undone first | Fixed | OK? |
| A75 | The edit stamp names the signed-in user (there is one login, Section 2) | Signed-in user | Should the reason box also ask who made the change (for example the munshi), typed by hand? |
| A76 | The partner code guards every partner total and history (strip, R30, R31, partner parts of R29, R33, R34, R36, R37, analytics F and partner columns, day and month close partner lines, partner totals in previews); partner names in pickers and one entry's split stay open | As stated | Should the split on the بکری screen also need the code? |
| A77 | The partner code stays open while the user stays on screens behind the code | Until he leaves them | Should it ask on every screen? |
| A78 | A reason is optional when today's entry is edited | Optional | Should every edit need a reason? |
| A79 | (retired in v1.32) | | |
| A80 | The answer is in the app language, whatever language the question is in | App language | Should an English question get an English answer in the Urdu app? |
| A81 | A model call that fails before a reply arrives is not counted in the 20 | Not counted | OK? |
| A82 | A table in an answer shows up to 7 rows, newest first for a history | 7 rows | More or fewer? |
| A83 | The panel's conversation clears on logout, lock and closing the app | Clears | Keep it across days? |
| A84 | The quick questions of FR-7.80 | As listed | Which questions does the Owner ask most? |
| A85 | A new model name is checked with the NFR-AI-2 question list before it is used at the shop | Checked first | OK? |
| A86 | The estimate methods of FR-7.86: same pace (last 3 full months), same growth (last 6), same season (last year) | Methods 1 and 2 shown; 3 when 12 months exist | Which one does the Owner trust most? |
| A87 | Up to 6 model calls, plus one rewrite, and 12 tool calls for one question (NFR-AI-5) | 6 and 12 (4 and 8 until v1.19) | Enough for the hardest questions? |
| A88 | Left out of what is sent: phones, addresses, father or firm names, notes and reasons (FR-7.83) | Left out | Should the father or firm name be sent, to tell two people apart in the text? |
| A89 | The answer text is at most 130 words (filled references count as one word each): one direct sentence and up to 5 points | 130 words | Longer or shorter? |
| A90 | When an answer's own figures plainly call for action, the model may add one advice point unasked (FR-7.88) | One point unasked; at most 3 when asked | More, fewer, or only when asked? |
| A91 | "Previous period" of whole calendar months is the whole months before it; of any other period, the same number of days just before it (FR-18.1) | September against August, not 2 to 31 August | Is that how the shop compares months? |
| A92 | A buyer who paid in full still prints on ادھاری تفصیل, with 0 left | Prints | Should he be left out? |
| A93 | `Enter` stops on the paid on the spot cell (4 `Enter`s a line) | Stops | Should it be skipped? |
| A94 | Money paid on the spot settles its own bill first | Its own bill | Or the buyer's oldest bill? |
| A95 | The new screen's name | پارٹی کھاتے (Party Ledgers) | Is that the word the shop uses? |
| A96 | The R5 button prints the whole day, with a choice of this بکری only | Whole day | OK? |
| A97 | A part-paid line is on both R3 and R4 | On both | OK? |
| A98 | Old open amounts on 0030 stay in the books and the trial balance, and do not show in dues lists | As stated | OK? |
| A99 | On the Mac, Print and PDF use the native print dialog; there is no one-click PDF | As stated | OK for the Mac phase? |
| A100 | Old lots are numbered per seller and item by first posting date, then lot id (FR-12.45) | As stated | OK? |
| A42 | Some old lines (mostly کیلا) have no rate and an amount that is not qty × a whole rate (4 نگ = 2,950) | Weight items (rate per kg) | Is کیلا sold by weight, or is a lump amount typed? |

---

## 25. Verification against the old software video

Every item seen in the video, and where this PRD covers it.

### 25.1 Screens and menus

| Seen in the video | Covered in |
|---|---|
| Login: user name, password, session start/end, login, exit, firm name, mandi name | Section 8 (session dates replaced by report ranges) |
| Main menu: Setup Forms, Transactions Forms, Financial Reports, Inventory Reports, Backup | Section 5 |
| Inventory Reports menu, "مکمل سٹاک" report, PUR_ITEM tables | Section 12.8, R10 |
| Side buttons: انڈکس, بکری, روکڑ, رپورٹ, بیک اپ | Section 5 |
| Welcome user, today's date | Section 5 (status bar) |
| انڈکس menu: COA Head, Control, Tree, انڈکس, شہر, قسم مال, پیکنگ | Section 10 |
| Account grid: S.Code, A/C Code, Description, Urdu Name, City, Open Date, Open Debit, Open Credit, Credit Allowed, totals | Section 10.2, FR-10.5 |
| Account grid footer: Main/Head Title, Control Head | Section 10.2 (Head display) |
| بکری menu: one button labelled کرایہ بک, which opens the بکری بک screen | Section 12 (no separate freight register, Section 14) |
| بکری screen header: بکری نمبر (2 boxes), تاریخ بل, تاریخ, ٹرک نمبر, زمیندار کوڈ + name, اڈا + name, ریفریش | Section 12.2 (refresh replaced by live totals, FR-7.5); from v1.34 only بکری نمبر and تاریخ stay |
| Upper grid: بزگر code/name, فون, منشیانہ, نقد, کرایہ سٹور, سٹور, sideways scroll | Section 12.3 (+ hidden inputs) |
| Lower grid: all 11 columns + میزان | Section 12.4 |
| Summary: خام بکری ... صافی بکری | Section 12.5 |
| Toolbar: save, print, first, previous, next, last, new record, delete record, query | FR-12.11 to FR-12.13 |
| روکڑ: وصولی, پیشگی کھاتہ, ادائیگی, جنرل | Sections 15, 16 |
| Reports left: انڈکس, روکڑ, فرد حساب اور کھاتہ, روزنامچہ, ووچر, مکمل سٹاک, پیشگی کھاتہ | R16, R12, R13/R14, R11, R15, R10, R26 |
| Reports right: بکری, بجک سیریز (بیجک), اڈا رپورٹ | R1 to R6, R27; R8 retired in v1.34 |
| Reports bottom: اوپن ٹرائل, ٹرائل بیلنس, اگراہی بحساب شہر, second اگراہی item | R19, R18, R21, R22 |
| بکری report box: از, تا, بیوپاری, زمیندار, قسم مال; buttons بکری بک, مکمل بکری, نقدی, ادھاری, بکری تفصیل, اخراج | FR-19.1, R1 to R6 |
| فرد حساب box: از (01-01-18), تا, کھاتہ نام, بیلنس, buttons فرد حساب / کھاتہ | R13, R14 |
| Daily report box: single From date | R11 |
| Backup button runs a full export | Section 20 |

### 25.2 Report contents

| Seen | Covered |
|---|---|
| ادھاری: بکری no, party, (qty @ rate), amount, drill-down arrow | R4, FR-19.4 |
| ادھاری تفصیل: party, detail, رقم, رقم بعد لاگا, Print All, page x of y | R5, FR-19.3 |
| مکمل بکری: بکری no, party, city/منڈی, detail | R2 |
| روزنامچہ بکری: بکری no, تعداد, بیوپاری, خام بکری, کل خرچہ, صافی بکری, ٹرک کرایہ, کمیشن, مزدوری | R6 |
| روزنامچہ (voucher list): firm header, date, account, بنام, جمع, totals | R11 |
| پختہ کھاتہ in seller column | Removed in v1.32: چکتا کھاتہ is for buyers only (Section 13) |
| Cash buyers shown as نقدی / منڈی | نقدی منڈی account, FR-10.4 |

### 25.3 Old database tables

| Old table | Covered by |
|---|---|
| CHART | Accounts, Section 10 |
| CITY | City master |
| CONFIG | Settings, Section 21 |
| ENTRY_TYPE (about 10 types) | Posting event types, Section 6.1 |
| JMST / JDTL (journal) | Posting rules, Sections 6 and 12.6 |
| KACHA_KHATA (empty) | Not used; A24 |
| LOCATION | Store master |
| MARKA | Mark master |
| NARR | Standard narration master |
| PACKING | Packing master |
| PUR_ITEM, PUR_ITEM_GROUP | Item master with item group |
| SDTL / SDTL_B | Lots and sale lines |

### 25.4 Problems seen in the old software, and how this PRD prevents them

| Problem | Prevented by |
|---|---|
| Journal off by 2 rupees | FR-6.2, FR-6.4, C15: same rounded number on every side; unbalanced entries rejected |
| Missing program file for ووچر | One app, no separate report programs |
| Reports failing to launch | Reports are part of the same app |
| Backup only on the same laptop | Section 20 |
| Numbers flipping in Urdu reports | FR-19.6 |
| Hard to use | Section 7 |

### 25.5 Second check of the video (before build)

After Umar's review, every frame of the video was read again. These findings changed the PRD:

| Seen in the video | What the PRD had | Now |
|---|---|---|
| بکری header field "تاریخ بل" | "تاریخ مال" (misread) | تاریخ بل, bill date (Section 12.2, A13); retired in v1.34: the entry date |
| The بکری menu has one button, "کرایہ بک", and it opens the بکری بک screen | A separate freight register | Removed (Section 14, A17) |
| Reports panel: "بجک سیریز" | Cheque series | بیجک (bill) series, R27 |
| Sales book: seller commission 6% (276 on 4,600) and 4% (180 on 4,500) on different lots | 6% default | 6% default confirmed by Umar; other rates by rules |
| Sales book: مزدوری 0 on most lots; 25 on 5 نگ (5 each), 15 on 5 نگ (3 each) | 5 per نگ on every lot | No default; typed or set by a rule |
| Credit bills: مولی and ساگ exactly 5% (700 → 735, 1,000 → 1,050); سیب 99 per نگ; کیلا 400 → 423 | Buyer 5% + لاگا 3 | Umar: buyer 6%, لاگا 8 per نگ on fruit; item and group rules for exceptions (A2, A3) |
| پختہ کھاتہ seller lots: کل خرچہ equals خام بکری, so صافی بکری is 0 | One-time sellers paid later | Removed in v1.32: there are no one-time sellers (Section 13) |
| Account form: "Allowed" is ticked by default; opening date defaults to today | Credit allowed off by default | Ticked by default (Section 10.2) |
| Many کیلا lines have no rate, and amounts like 4 نگ = 2,950 | Not covered | A42 |
| Day book 1,316,042 against 1,316,040; one lot with صافی −1 | Already noted | Prevented by FR-6.2 and C15 |

---

## 26. MVP scope

| Priority | Items |
|---|---|
| **Must (demo, all connected)** | Login; one edit model on every entry with the edit stamp and reopen-and-fix (FR-7.70, FR-7.71, FR-21.4); accounts on any day (FR-10.11); the partner code (FR-17.11); dashboard with partner strip; انڈکس with seller and buyer codes by type (0010, 0020) and opening balances; all masters; commission rules; بکری بک with side menu, later sales, edit and cancel; money paid on the spot (FR-12.44); چکتا کھاتہ (13); party ledgers (FR-12.43); printing on the Mac (FR-19.3); باقی مال; وصولی/اگراہی; ادائیگی; پیشگی; one خرچہ screen for دکان خرچہ and ذاتی خرچہ with the effect shown before save (16.2, FR-16.8); related entries on one screen and the right-type notes (FR-7.67, FR-7.68); جنرل; partners with live split; partner capital in and out (17.4); partner total کل رقم and its dated record (FR-17.9); الگ کھاتہ (FR-16.6, FR-16.7); market trips with their purchases, costs, sales, write-offs and result (12.10); gardens with installments, costs, sales and closing (12.11); fruit taken (16.7); one cash account with no bank (v1.6); dated statement lines (FR-17.10); partner money taken and returned (FR-16.2, FR-16.3); ادھار رقم (FR-16.4, FR-16.5); advance tick and advance return (FR-12.23, FR-15.12); cash lent to buyers (FR-15.11); spelling check (FR-7.66); day close and reopen; every analytics view A to K, printable by day, month or year (FR-18.4, FR-18.8); the dues tiles and the dead amount (9.1, FR-10.10); reports R1, R2, R4, R5, R6, R7, R10, R11, R12, R13, R14, R15, R18, R20, R21, R22, R23, R24, R25, R26, R27, R29, R30, R31, R32, R33, R34, R35, R36, R37, R38; the party day sheet (FR-19.12) and the collection round (FR-7.61); large lists page by page (FR-19.11); print any day and any party (FR-19.10); items in ledgers and the shop's total اگراہی (FR-19.8, FR-19.9); Excel export (needed for the backup pack); arrow-key navigation; Urdu keyboard; language switch with the whole app in English (Section 7.5); names in two scripts with English-to-Urdu typing and learning (Section 7.4); the tag after a name (10.2); prints in the client's layouts and a clear layout everywhere else, with the firm's contacts and trade mark (FR-19.12, FR-19.13); party card, short codes, smart search, recent first, unknown code add (FR-7.38 to FR-7.42); how was this worked out (FR-7.43); attention list and آج کے کام (FR-7.44); helpful empty screens (FR-7.46); seller and buyer memory, lot names (FR-12.45), remaining qty, rate hint, copy from above, save-print-next (FR-7.51 to FR-7.54, FR-7.56, FR-7.57); receipt, payment and advance helpers (FR-7.58 to FR-7.60); backup and restore; printing in both languages; audit log (written); ask the app: the model reads every section through read-only tools and writes checked answers, sums, estimates and advice; quick questions, the partner code, the daily limit and the question log (7.7, v1.16) |
| **Should (right after demo)** | Saved analytics views; reports R3, R16, R17, R19, R28; month close; audit log viewer; first-run guide; undo after save (FR-7.47); simple sums (FR-7.48); rate check (FR-7.55); repeat last month's expenses (FR-7.62); duplicate check while typing (FR-7.63); remember last screen (FR-7.45); party report buttons (FR-7.65) |
| **Could (Phase 2)** | Import of 2018 to 2026 data from the old software; crate (پیٹی) in and out tracking; goods return screen; interest on old dues; daily rate board |
| **Won't (for now)** | Multi-computer, tax filing |

---

## 27. Demo script and end-to-end acceptance tests

### 27.1 Demo script (about 45 minutes)

1. Log in. Show the dashboard and partner strip.
2. Show partners (الف 40, ب 30, ج 30) and commission rules. Use "test a rule".
3. Add a new seller and a new buyer using only the arrow keys and `Enter`: type the name in English letters, see it become Urdu with the English name filled. Show the codes.
4. Enter بکری 1 (Section 4.4) with short codes (`0`, `1`). Show the party card, remaining qty, rate hint, live summary and live partner split, and ⓘ on a commission. Halfway, switch to English with `Alt+L`, carry on, switch back. Save with `Ctrl+Enter`, which prints the seller bill and buyer slips.
5. Enter بکری 2 with a چکتا buyer (Section 4.5). Show the seller memory and the lot name (FR-12.45).
6. Record وصولی (open bills, پورا بقایا) and ادائیگی on the voucher screen. Then, on the one خرچہ screen, the electricity bill as دکان خرچہ (show the split before save) and الف's 2,000 as ذاتی خرچہ (Sections 4.6 to 4.8).
6a. Show the attention bell and آج کے کام. Print a buyer slip in Urdu and in English.
6b. Show a market trip and a garden (E2E-41, E2E-40), a partner's statement with their dated lines, and الف's پھل. Print the party day sheet (R35) for all buyers.
7. Dashboard: every tile matches Section 4.9.
8. Analytics: views A, B, C, F match Section 18.4.
9. Partner overview matches Section 4.9.
10. روزنامچہ and trial balance: both balance.
10a. Press `F1` and ask "ikram ke total bakaya kitne hai", "akram aur arshad dono ka mila kar kitna hai", "aaj ki bikri kitni hui" and "partner ki kiya total hain abhi tak" (with the partner code). On the E2E-59 data, ask "agar commission isi raftar se chalta raha to agle 6 mahine mein kitna hoga". On the E2E-63 date, ask "kis se pehle wasooli karun" (advice). Open the full report from an answer. Turn the internet off and use a quick question.
11. Backup screen.
12. Walk through the Assumptions Register and note answers.

### 27.2 End-to-end tests (must pass before the demo)

Start from the opening position created as in Section 4.3, then run Events 1 to 5, unless a test says otherwise. Where each test runs is set in Code Standards Section 9.2.

| Test | Check | Expected |
|---|---|---|
| E2E-1 | بکری 1 summary | خام 29,400; کل اخراجات 10,084; صافی 19,316 |
| E2E-2 | بکری 1 buyer totals | اکرم 12,880; ارشد 18,684 |
| E2E-3 | بکری 1 journal entry | Sale part بنام 31,564 = جمع 31,564; whole entry with the نقد lines 36,564 = 36,564 |
| E2E-4 | بکری 1 partner split | Shared income 3,948; 1,580 / 1,184 / 1,184 |
| E2E-5 | بکری 2 (چکتا buyer) | Buyer pays 12,800; seller صافی 10,160; whole entry 25,600 = 25,600; shared income 1,540; split 616 / 462 / 462; the day's sale book shows شاہد, paid |
| E2E-6 | اکرم after وصولی | 2,880 بنام; aging 0 days |
| E2E-7 | رحیم after ادائیگی | Balance 0; پیشگی report: given 15,000, adjusted 15,000, open 0 |
| E2E-8 | Cash in hand | 58,484 |
| E2E-9 | Partner shares for the day | Income 2,196 / 1,646 / 1,646; expense −1,200 / −900 / −900; profit 996 / 746 / 746; الف drawings 2,000 |
| E2E-10 | Trial balance | Table below; both sides 85,048 |
| E2E-11 | Analytics | View A as in Section 18.4 (including avg rate 588 / 1,200 and 72.10% / 27.90%); view F as in Section 18.4 |
| E2E-12 | Stock | باقی مال empty |
| E2E-13 | Edit بکری 1: اکرم's rate 600 → 610 | His line 20 × 610 = 12,200; buyer comm 12,200 × 6 ÷ 100 = 732; he pays 12,200 + 732 + 160 = 13,092; خام بکری 29,600; seller comm 732 + 1,044 = 1,776; کل اخراجات 10,096; صافی 19,504; shared income 1,776 + 1,776 + 400 + 20 = 3,972; split 1,590 / 1,191 / 1,191; audit log has the old version |
| E2E-14 | Edit back to 600 | Every number returns exactly to E2E-1 to E2E-4 |
| E2E-15 | Cancel بکری 2 | The day's sale book shows it cancelled; analytics excludes سیب; split reversed (616 / 462 / 462 removed); cash down by 12,800; trial balance still balances |
| E2E-16 | Separate run: Event 1 with only 45 of 50 sold (اکرم 20 at 600, ارشد 25 at 580) | خام 26,500; seller comm 720 + 870 = 1,590; کل اخراجات 3,000 + 1,590 + 250 + 50 + 5,000 + 0 + 20 = 9,910; صافی 16,590; buyer comm 720 + 870 = 1,590; لاگا 45 × 8 = 360; shared income 1,590 + 1,590 + 360 + 20 = 3,560; split 1,424 / 1,068 / 1,068; باقی مال 5 نگ کیلا |
| E2E-16b | Continue E2E-16: on 11-10-26 a later sale of the 5 نگ to ارشد at 560 | Value 2,800; seller comm 168; buyer comm 168; لاگا 5 × 8 = 40; ارشد pays 3,008; seller credit 2,800 − 168 = 2,632; no lot charges again; shared income 168 + 168 + 40 = 376; split 152 / 112 / 112; lot totals خام 29,300, کل اخراجات 10,078, صافی 19,222; باقی مال empty |
| E2E-13b | Separate run: after Event 1, the Owner changes the default buyer commission to 7%, then edits اکرم's rate to 610 | اکرم's changed line uses 7%: 12,200 × 7 ÷ 100 = 854, he pays 12,200 + 854 + 160 = 13,214; ارشد's unchanged line keeps 1,044; shared income 1,776 + 1,898 + 400 + 20 = 4,094; split 1,638 / 1,228 / 1,228 |
| E2E-17 | Close the app while typing a بکری; reopen | Draft offered and restored |
| E2E-18 | Backup, then restore | All balances identical after restore |
| E2E-19 | Opening set-up only (Section 4.3) | Trial balance 65,000 = 65,000; ابتدائی فرق 0; advances open 15,000; cash 50,000 |
| E2E-20 | A shop expense of 3,384 | Shares −1,354 / −1,015 / −1,015 (sum −3,384) |
| E2E-21 | Two چکتا buyers in one day, شاہد 1,000 and ناصر 2,000, each line paid in full | Neither has a balance or a due; 0030 ends at 0; both are in the day's sale book |
| E2E-22 | Start بکری 1, enter اکرم's line, switch to English (`Alt+L`), enter ارشد's line, save, print the seller bill in English, switch back | After the switch: line reads "Akram Toll Tax", "Banana", "Box", 20, 600, 12,000; layout left-to-right; nothing typed is lost. Saved numbers identical to E2E-1 to E2E-4. English bill: Gross Sale 29,400, Net Sale 19,316, "Nineteen thousand three hundred sixteen rupees only". Back in Urdu, the same بکری shows Urdu names. |
| E2E-23 | New buyer: type `tariq mehmood` in the Urdu name field, accept, save | Urdu name طارق محمود; English name "Tariq Mehmood" filled by itself; code 0022. Typing "tarq" in a buyer picker finds him in both languages. |
| E2E-24 | After Events 1 to 5: new وصولی, buyer field `0` | 0020 اکرم ٹول ٹیکس; card shows 2,880 بنام, oldest open bill 10-10-26 (بکری 1), 0 days; "پورا بقایا" fills 2,880. In a جنرل line, `0` lists both 0020 اکرم ٹول ٹیکس and 0010 رحیم with their types. |
| E2E-25 | After Events 1 and 2: new بکری, lot seller `0` | The lot fills کیلا and پیٹی (FR-7.51), and its name before the save is رحیم، کیلا-2 (FR-12.45). The truck parts of this test are retired in v1.34 |
| E2E-26 | In بکری 1, after اکرم's line (20 of 50) | ارشد's line qty starts at 30; rate field empty with hint 600; `Ctrl+D` in the rate cell copies 600; ⓘ on اکرم's seller commission shows "12,000 × 6 ÷ 100 = 720". |
| E2E-27 | After Event 2, ادائیگی to کریم | Payable postings show بکری 2, صافی بکری 10,160, open 10,160; "پورا بقایا" fills 10,160. |
| E2E-28 | Separate run: opening position, then Event 1 with "پیشگی سے کاٹیں" unticked | رحیم payable 24,316 − 5,000 = 19,316; advance open 15,000; ledger 4,316 جمع. The بیوپاریوں کو ادائیگی باقی tile and R24 show 19,316, with advance 15,000. After an ادائیگی of 19,316: payable 0, ledger 15,000 بنام, advance open 15,000, and رحیم is not in "بیوپاری کے ذمے". Trial balance balances. |
| E2E-29 | After Events 1 to 5, on 11-10-26 الف returns 2,000 cash (ذاتی خرچہ واپسی) | On 10-10-26: الف taken 2,000 (reason گھر کا خرچ), خالص 996 − 2,000 = −1,004. After the return: net taken 0, خالص for 10-10 to 11-10 is 996; cash 60,484; his take shows fully returned. |
| E2E-30 | After Events 1 to 5: staff اسلم is lent 5,000 cash on 10-10-26 (reason گھریلو ضرورت, return by 15-10-26); returns 3,000 cash on 11-10-26; 2,000 deducted from salary on 12-10-26 | 10-10: cash 53,484, open 5,000. 11-10: open 2,000, cash 56,484. 12-10: open 0, cash still 56,484; تنخواہ 2,000 split −800 / −600 / −600; R32: lent 5,000, returned 5,000 (cash 3,000, salary 2,000), open 0. Trial balance balances. |
| E2E-30b | Separate run: as E2E-30 up to 11-10-26, no salary deduction; "today" set to 16-10-26 | The attention list shows اسلم 2,000 past its return-by date 15-10-26 (1 day); R32 shows him overdue |
| E2E-31 | Separate run: opening position; on 10-10-26 رحیم returns 10,000 of his advance in cash (وصولی on his account); then Event 1 | After the return: advance open 5,000; cash 60,000. After بکری 1: advance adjusted 5,000; پیشگی report given 15,000, returned 10,000, adjusted 5,000, open 0; رحیم payable 14,316; ledger 14,316 جمع. |
| E2E-32 | After Events 1 to 5: ادائیگی of 3,000 cash to اکرم (reason نقد ادھار) | اکرم dues 2,880 + 3,000 = 5,880, the 3,000 is a bill dated 10-10-26 after his بکری 1 bill; cash 55,484; R20 میزان and the dashboard tile گاہکوں کے ذمے 24,564. |
| E2E-33 | After Events 1 to 5: اکرم's ledger (R13) and R20 | 10-10-26 بکری 1: کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160, بنام 12,880; وصولی جمع 10,000; closing 2,880 بنام. R20 rows اکرم 2,880 and ارشد 18,684; میزان 21,564 = dashboard گاہکوں کے ذمے (and کل اگراہی, nobody being in the dead amount). |
| E2E-34 | Separate run: opening position (4.3). On 10-10-26 a market trip (partners الف 40%, ب 30%, ج 30%, all from the shop's cash) buys 20 پیٹی مالٹا (fruit) at 500, freight 400. A بکری lot with seller اپنا مال sells 15 of them to اکرم at 600, ادھاری, no lot charges. | Purchase cost 10,000 + 400 = 10,400 (520 per نگ), paid from the shop's cash 4,160 / 3,120 / 3,120 (each comes off his total); cash 50,000 − 10,400 = 39,600. Sale: value 9,000; seller comm 540; buyer comm 540; لاگا 15 × 8 = 120; اکرم pays 9,660; entry 9,660 = 540 + 540 + 120 + 8,460. Commission 1,200 split by the firm's shares 480 / 360 / 360. The seller credit 9,000 − 540 = 8,460 goes to the trip's partners 3,384 / 2,538 / 2,538. Shown, not posted: cost of goods sold 10,400 × 15 ÷ 20 = 7,800; result on the 15 = 660. Partner totals −4,160 + 480 + 3,384 = −296, −3,120 + 360 + 2,538 = −222, −222. Stock left 5 نگ, cost 2,600. |
| E2E-34b | Continue E2E-34: on 11-10-26 the last 5 spoil and are written off | Nothing posts. The trip's result 8,460 − 10,400 = −1,940 (R33), each partner's part −776 / −582 / −582, already in his total (3,384 − 4,160 = −776); own stock 0; the trip closes; partner totals still −296 / −222 / −222; Σ −740 + 65,000 = 64,260 = equity; trial balance 66,200 = 66,200. |
| E2E-35 | After Events 1 to 5, on 11-10-26 partner ب invests 50,000 cash (سرمایہ جمع) | Cash 58,484 + 50,000 = 108,484; ب capital 50,000 جمع; ب profit share still 746; shares unchanged; trial balance 135,048 = 135,048. |
| E2E-36 | After Events 1 to 5: R34 for 10-10-26, and R5 filtered to اکرم and 10-10-26 | R34: 2 بکری; خام 41,400; commission 5,448; cash opening 50,000, in 12,800 + 10,000 = 22,800, out 5,000 + 4,316 + 3,000 + 2,000 = 14,316, closing 58,484; buyers' dues 21,564; sellers' dues 0; dead amount 0; total dues 21,564; commission seller 2,484, buyer 2,484, لاگا 480, total 5,448; 60 نگ; buyers billed 44,364, cash received 22,800, left 21,564; sellers net sale 34,476, paid out 9,316 (an "other entries" line shows for each side only when not 0). Buyers' table: اکرم 0, 12,880, 10,000, 2,880; ارشد 0, 18,684, 0, 18,684; شاہد 0, 12,800, 12,800, 0; total 0, 44,364, 22,800, 21,564. Sellers' table: رحیم 15,000 بنام, 24,316, 9,316, 0; کریم 0, 10,160, 0, 10,160 جمع; total previous 15,000 بنام, net sale 34,476, paid 9,316, new 10,160 جمع. R5 shows only اکرم's row: (کیلا 20 . 600), 12,000, 12,880. Both headers show the date and, for R5, the buyer. |
| E2E-37 | After Events 1 to 5 and E2E-35: partner totals and R31 for ب | کل رقم الف −1,004, ب 50,746, ج 746; Σ 50,488 + ابتدائی سرمایہ 65,000 = 115,488 = equity in the trial balance (65,000 + 50,000 + 5,488 − 3,000 − 2,000). R31 for ب, in date order: 10-10-26 بکری 1 share 1,184 (running 1,184); بکری 2 share 462 (1,646); دکان خرچہ بجلی −900 (746); 11-10-26 سرمایہ جمع 50,000 (50,746). |
| E2E-38 | Separate run: opening position. A الگ کھاتہ خاندانی کھاتہ (Family Khata). 10-10-26: pays out 10,000 cash (reason خاندان کا خرچ). 11-10-26: gets 4,000 back in cash; then a بکری lot with seller خاندانی کھاتہ sells 10 پیٹی کیلا to ارشد at 900, ادھاری | After paying out: خاندانی کھاتہ −10,000, cash 40,000. After 4,000 back: −6,000, cash 44,000. The sale: value 9,000; no seller-side charges; buyer comm 540; لاگا 80; ارشد pays 9,620; entry 9,620 = 9,000 (خاندانی کھاتہ) + 540 + 80; خاندانی کھاتہ +3,000. Shared income 620, split 248 / 186 / 186; nothing from its 9,000 goes to any partner. Trial balance balances. |
| E2E-39 | Event 5 on the one خرچہ screen (`Alt+K`). First entry: بجلی 3,000 as دکان خرچہ. Second entry: 2,000 typed as دکان خرچہ by mistake, then the type changed to ذاتی خرچہ before saving, partner الف, reason گھر کا خرچ | First entry, before save: الف 1,200, ب 900, ج 900. Second entry: changing the type keeps 10-10-26 and 2,000, clears the expense head and asks for partner and reason; then it shows الف's کل رقم 996 → −1,004 and ب, ج not affected. After both saves: cash 58,484 (E2E-8) and the partner numbers of E2E-9. |
| E2E-40 | Separate run: opening position. A garden شاہ پور باغ (Shahpur Garden): الف 60% from the shop's cash, ب 40% from his own pocket, ج not in it; rent 50,000. 10-10-26: first installment 30,000. 11-10-26: installment 20,000 and labour 5,000. 12-10-26: a بکری lot with seller شاہ پور باغ sells 100 پیٹی کیلا to ارشد at 600, ادھاری, no lot charges; 20 پیٹی sold in another shop bring 10,000 cash. 13-10-26: the garden is closed | Parts 18,000 / 12,000, then 12,000 / 8,000 and 3,000 / 2,000: الف paid 33,000 from the shop (his total −33,000; cash 17,000), ب paid 22,000 himself (his total unchanged). The sale: value 60,000; seller comm 3,600; buyer comm 3,600; لاگا 800; ارشد pays 64,400; the garden's account جمع 56,400; commission 8,000 split by the firm's shares 3,200 / 2,400 / 2,400. Money in 56,400 + 10,000 = 66,400; cash 27,000; running balance 66,400 − 55,000 = 11,400. Closing: 66,400 → 39,840 / 26,560; the garden's account 0. الف −33,000 + 39,840 = +6,840 (60% of 11,400); ب +26,560 (22,000 back + 4,560). Partner totals 10,040 / 28,960 / 2,400; Σ 41,400 + 65,000 = 106,400 = equity; trial balance 106,400 = 106,400. |
| E2E-40b | Separate run: as E2E-40, but with no بکری sale; only the other shop's 10,000 comes in; then the garden is closed | Loss 10,000 − 55,000 = −45,000. Closing: 10,000 → 6,000 / 4,000. الف −33,000 + 6,000 = −27,000 (60% of the loss); ب +4,000 (his 22,000 back − 18,000, 40% of the loss); cash 27,000. |
| E2E-41 | Separate run: opening position. A market trip: الف 50% from the shop's cash, ب 50% from his own pocket, ج not in it. It buys 80 پیٹی مالٹا at 1,200 and pays 4,000 for the truck (a trip cost). A بکری lot with seller اپنا مال sells all 80 to ارشد at 1,400, ادھاری, no lot charges | Purchase 96,000: parts 48,000 / 48,000; truck 2,000 / 2,000. الف paid 50,000 from the shop (cash 0); ب paid 50,000 himself. Cost 100,000 (1,250 per نگ). Sale: value 112,000; seller comm 6,720; buyer comm 6,720; لاگا 640; ارشد pays 119,360; commission 14,080 split by the firm's shares 5,632 / 4,224 / 4,224; the trip's 105,280 → 52,640 / 52,640; trip result 5,280; the trip closes. الف −50,000 + 52,640 = +2,640; ب +52,640 (50,000 back + 2,640). Partner totals 8,272 / 56,864 / 4,224; trial balance 134,360 = 134,360. |
| E2E-42 | Separate run: after E2E-16 (5 کیلا left in رحیم's lot). 10-10-26: الف takes 2 پیٹی کیلا home from that lot at 580; then fruit for the shop's guests: 3 پیٹی سیب at 1,000 from another commission agent's shop (حاجی نذیر, seller 0012) | Home fruit 2 × 580 = 1,160: الف's total −1,160; رحیم credited 1,160 with no commission, his payable 1,590 + 1,160 = 2,750 and ledger 2,750 جمع; lot balance 3. Guest fruit 3,000 split equally 1,000 / 1,000 / 1,000; حاجی نذیر payable 3,000. Cash unchanged at 45,000. Partner totals 1,424 − 2,160 = −736, 1,068 − 1,000 = 68, 68; Σ −600 + 65,000 = 64,400 = equity; trial balance 77,610 = 77,610. |
| E2E-43 | After Events 1 to 5: R35 party day sheet for 10-10-26 | Buyers (previous, bought, paid on the spot, paid that day, remaining): اکرم 0, 12,880 (کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160), 0, 10,000, 2,880; ارشد 0, 18,684, 0, 0, 18,684; شاہد (چکتا) 0, 12,800, 12,800, 0, 0. All buyers remaining 21,564 = R20 میزان. Sellers (previous, sold, نقد, paid that day, remaining): رحیم 15,000 بنام, 24,316, 5,000, 4,316, 0; کریم 0, 10,160, 0, 0, 10,160 جمع. The day slips of اکرم and رحیم read as in the FR-19.12 example (v1.12). |
| E2E-44 | After Events 1 to 5: on 11-10-26 ارشد is moved to ڈوبی رقم (reason فوت ہو گئے); on 12-10-26 a وصولی of 4,000 cash on him; then a new بکری line for him | After the move: گاہکوں کے ذمے 2,880 (R20 shows only اکرم); بیوپاریوں کے ذمے 0; ڈوبی رقم 18,684, 1 person (R38); کل اگراہی 21,564; his account inactive; trial balance still 85,048 = 85,048; partner totals unchanged. After the وصولی: ڈوبی رقم 14,684; کل اگراہی 17,564; cash 62,484; R38 shows owed then 18,684, recovered 4,000, still 14,684. The بکری line is refused (ERR_ACCOUNT_INACTIVE). |
| E2E-45 | After Events 1 to 5 with 10-10-26 closed: بکری 1 is edited through "دن کھولیں اور درست کریں", اکرم's rate 600 to 610 | Every figure equals E2E-13 (اکرم pays 13,092; صافی 19,504; split 1,590 / 1,191 / 1,191); the date stays 10-10-26; the closed-up-to date is 10-10-26 again; the entry shows "تبدیل شدہ" with the time and Owner, line 1 rate marked "پہلے 600"; the audit log has the reopen, the edit with its reason and the close |
| E2E-46 | After Events 1 to 5, the partner code set to 4826 | Partner commands and R31 are refused with ERR_PARTNER_LOCKED until 4826 is given; 1111 is refused and logged; with 4826 the overview shows profit shares 996 / 746 / 746; leaving the section locks it again |
| E2E-47 | After Events 1 to 5 (today 10-10-26), with the recorded model replies of Code Standards 9.2: "ikram ke total bakaya kitne hai", then "karim ko kitne paise dene hain" | Answer 1: the model calls `find` (one match, اکرم ٹول ٹیکس 0020), `party_card` and R13; the text shows 2,880 بنام ✓ (party card 2,880 = R13 closing 2,880), 12,880 and 10,000, every figure from a reference; the table: 10-10-26 وصولی جمع 10,000, 10-10-26 بکری 1 بنام 12,880; "پوری رپورٹ کھولیں" opens R13 for اکرم. Answer 2: کریم (0011), 10,160 جمع ✓. Two questions counted |
| E2E-48 | After Events 1 to 5: "aaj ki bikri kitni hui aur commission kitna bana" | One answer: آج کی بکری 41,400 in 2 بکری ✓ (tile = view A for 10-10-26); آج کا کمیشن 5,448 ✓ (tile = view A total commission). One question counted, whatever the number of tools |
| E2E-49 | After Events 1 to 5: "kele se is mahine kitna commission aya" | View A, period یہ مہینہ (01-10-26 to 10-10-26), item کیلا: total commission 3,928, 72.10% of all; the button opens view A with these filters |
| E2E-50 | After Events 1 to 5, partner code 4826: "partner ki kiya total hain abhi tak" | Locked: the partner tools are refused with ERR_PARTNER_LOCKED; the answer shows the code box and no figure; no request body holds a partner figure. With 4826 the model goes on, not counted again: کل رقم الف −1,004, ب 746, ج 746 ✓ against R31 |
| E2E-51 | After Events 1 to 5: "kul ugrahi kitni hai"; then the quick question موجود روکڑ | Answer: کل اگراہی 21,564 ✓ (21,564 = R20, 0 = R24 بیوپاری کے ذمے, 0 = R38). Chip: 58,484 ✓ (R12 closing), no model call, not counted |
| E2E-52 | Separate run: Events 1 to 5 and a new buyer اکرم with tag K (0022), no entries: "akram ka bakaya" | The answer asks "کون سا؟" with اکرم ٹول ٹیکس (0020) 2,880 and اکرم K (0022) 0; a click on 0020 shows 2,880 ✓; one question counted |
| E2E-53 | After Events 1 to 5: "akram ki 5000 wasooli likh do" | Refused; the row count of every table is the same as before, except one new question log row; the button opens وصولی with اکرم chosen and the amount empty |
| E2E-54 | Every request body sent in E2E-47 to E2E-63 | None holds a phone (for example شاہد's 0300-0000000), an address, a father or firm name, a note or reason from the books, the key, or a partner figure while the section is locked; each has the provider block of Tech Stack 4.8 (Azure only, no fallback, zero data retention). Also: after E2E-50 with the code given, the section is locked again and a follow-up is asked; its body holds none of the earlier partner figures |
| E2E-55 | After 20 answered questions on one day: a 21st; then the network turned off | The 21st is refused with the limit message and no request is sent. Offline: the offline message, nothing counted. In both cases the chip موجود روکڑ still shows 58,484 |
| E2E-56 | Recorded model replies that are wrong: one calls `report` with a report number the app does not have and then gives up; one passes the date 31-02 | The unknown report: the tool returns an error, then "سوال سمجھ نہیں آیا"; nothing shown; all of it in the log. 31-02: the tool refuses the date and the answer asks for the dates |
| E2E-57 | After Events 1 to 5: "rahim ka pura hisaab" | رحیم (0010), بقایا 0 ✓ (party card 0 = R13 closing 0); the table rows are the last rows of R13 for رحیم from the first date, cell for cell, newest first: they start with the ادائیگی of 10-10-26 (بنام 4,316) and end with the پیشگی of 01-09-26 (بنام 15,000); the button opens R13 for رحیم from the first date |
| E2E-58 | After E2E-47: "aur pichle mahine?" | اکرم ٹول ٹیکس for پچھلا مہینہ (01-09-26 to 30-09-26): no entries, balance 0; the request carries the earlier questions and answers of the panel (FR-7.78) |
| E2E-59 | Separate run, today 10-10-26: the shop's commission (view A, all commission) by full month: April 36,000, May 39,000, June 42,000, July 45,000, August 48,000, September 51,000. Question: "agar commission isi raftar se chalta raha to agle 6 mahine mein kitna hoga". Then the same with only August and September in the books | Same pace: (45,000 + 48,000 + 51,000) ÷ 3 = 48,000 a month, October 2026 to March 2027: 288,000. Same growth: (51,000 − 36,000) ÷ 5 = 3,000 a month: 54,000, 57,000, 60,000, 63,000, 66,000, 69,000 = 369,000. The answer carries the band "اندازہ، یقینی نہیں" with both methods; nothing is saved; no report or tile changes. Then "commission ka 3 lakh ka target kab pura hoga": same pace April 2027 (288,000 after 6 months, 336,000 after 7), same growth February 2027 (54,000 + 57,000 + 60,000 + 63,000 + 66,000 = 300,000, reached exactly). With 2 months: "only 2 full months", no estimate, and the reply is a plain answer with no estimate band |
| E2E-60 | After Events 1 to 5: "ikram ke total bakaya kitne hai", with a recorded first answer that types "2,800" as digits instead of a reference, and a second that uses the reference; then a run where both answers type digits | First run: the first answer is refused, the model is told why, the second shows 2,880 ✓; counted once. Second run: the panel shows the party card and R13 rows as plain tables with "میں جانچا ہوا جواب نہیں لکھ سکا"; no figure typed by the model is shown. Third run: recorded answers that write the figure in words ("دو ہزار آٹھ سو اسی"), in Arabic-Indic digits (٢٨٨٠), as a name typed outside a reference, through a next question that holds a figure, through an unchecked balance from `find`, with a ✓ typed by the model, with اکرم's بنام balance written with the other side's words ("... کو ... دینے ہیں"), or with a reference to a result of an earlier question: each is refused |
| E2E-61 | After Events 1 to 5: "akram aur arshad dono ka mila kar kitna hai" | `calculate` sum: 2,880 + 18,684 = 21,564; the answer shows 21,564 بنام ✓ with 2,880 بنام ✓ and 18,684 بنام ✓, and ⓘ shows the working |
| E2E-62 | After Events 1 to 5, four questions: "akram ka hisab kya hai", "akram ka khata dikhao", "اکرم سے کتنا لینا ہے", "how much does akram owe" | Each answer shows 2,880 بنام ✓ for اکرم ٹول ٹیکس (0020) from the party card and R13. With recorded replies here; the real model is checked on the same wordings in NFR-AI-2 |
| E2E-63 | Separate run, today 15-11-26, nothing entered after Events 1 to 5: "kis se pehle wasooli karun" | Kind advice, with the band "مشورہ، فیصلہ آپ کا". R20 read sorted by days, then balance: ارشد ملتانی 18,684 بنام ✓ first, then اکرم ٹول ٹیکس 2,880 بنام ✓, both 36 days (10-10-26 to 15-11-26), each checked against the party card and R13; the R20 total 21,564 = the tile گاہکوں کے ذمے. The text names no rate or figure of its own; the button opens R20 as of 15-11-26; one question counted; outcome advice in the log |
| E2E-64 | Image 1 of D8: 200 of 200 sold in three lines with the rate of line 3 equal to the hint | Line totals 21,360 / 48,100 / 125,060; خام بکری 182,000; کل اخراجات 21,920; صافی بکری 160,080; shared income 23,440; split 9,376 / 7,032 / 7,032 |
| E2E-65 | E2E-64 with paid 21,360 / 10,000 / 0, and ہارون holding 10,000 جمع before the sale | Cash up 31,360; left 0 / 38,100 / 125,060; ہارون after the sale 115,060 بنام |
| E2E-66 | Old data: a line saved as cash and a line saved as credit before the migration | paid = total and paid = 0; every balance as before |
| E2E-67 | Party ledgers after Events 1 to 5 | Buyers اکرم 2,880 بنام, ارشد 18,684 بنام; اکرم on 10-10-26: previous 0, charged 12,880, paid 10,000, balance 2,880 |
| E2E-68 | بکری 1 line 1 moved from اکرم to طارق after Events 1 to 5, then back | اکرم 10,000 جمع; طارق 12,880 بنام; commission and shares unchanged; trial balance 95,048; back gives 85,048 |
| E2E-69 | Urdu PDFs of R1, R5, R35 | No ۰ to ۹ or ٠ to ٩; "10,000" and "2026" present |
| E2E-70 | Separate run: one بکری of احسان with lots typed آم, کیلا, آم; then a second بکری with a آم lot of his; then R1 of the first | Names احسان، آم-1, احسان، کیلا-1, احسان، آم-2 before and after the save; the second بکری's lot is احسان، آم-3; R1 prints آم-1, آم-2, کیلا-1. After migration 0022 old lots are numbered per seller and item by first posting date, then lot id, with every amount, entry and chain hash unchanged |

E2E-10 expected trial balance (after Events 1 to 5):

| Account | بنام | جمع |
|---|---|---|
| روکڑ | 58,484 | |
| اکرم ٹول ٹیکس | 2,880 | |
| ارشد ملتانی | 18,684 | |
| دکان خرچہ (بجلی) | 3,000 | |
| الف ذاتی خرچہ | 2,000 | |
| رحیم | | 0 |
| کریم | | 10,160 |
| چکتا کھاتہ | | 0 |
| ابتدائی فرق | | 0 |
| کمیشن آمدن (بیوپاری) | | 2,484 |
| کمیشن آمدن (گاہک) | | 2,484 |
| لاگا آمدن | | 480 |
| فون آمدن | | 40 |
| مزدوری کھاتہ | | 300 |
| کرایہ کھاتہ | | 4,000 |
| منشیانہ کھاتہ | | 100 |
| ابتدائی سرمایہ | | 65,000 |
| **میزان** | **85,048** | **85,048** |

How to check the total: بنام = 58,484 + 2,880 + 18,684 + 3,000 + 2,000 = 85,048. جمع = 10,160 + 2,484 + 2,484 + 480 + 40 + 300 + 4,000 + 100 + 65,000 = 85,048.

---

## 28. Research: similar projects and what we took

| Project | What it is | What we took |
|---|---|---|
| designer-haxxan/ghallamandiapp (Pakistan) | Offline ghalla mandi app, Urdu RTL | Peshgi, oldest-first matching, aging buckets, cancel with reversal, credit limit warning |
| httpsasad/Grain-Market-Management-System (Pakistan) | Urdu and English mandi app with aamad, bikri, running khata | Commission as %, per unit or fixed; both sides recorded on each sale; slips |
| Devesh2325/arhatpro-mandi-erp (India) | Mandi ERP with Munshi and Accountant roles | Roles, unbalanced-entry rejection, per-box labour |
| Iamvive/MandiSamiti (India) | Offline ledger for commission agents | Integer money storage, cash register formula |
| smihheemmaanntt/Aadhat (India) | Desktop mandi software | Keyboard shortcut map, crate manager idea for Phase 2 |
| BUSY for Mandi (commercial) | Mandi accounting | Configurable deduction list, item-wise profit |

None of these has a live partner split or a commission rule system by item, seller and buyer. Those are our own design for this client.

---

## 29. Change log

| Version | Date | Change |
|---|---|---|
| 1.0 | 10-10-2026 | Pre-build drafts and reviews: Umar's requirements, analysis of the old software video, research, client review comments and full cross-document reviews. |
| 1.1 | 10-10-2026 | **Final baseline for build.** From the start of development, every change raises the version (1.2, 1.3 ...) and adds a line here. |
| 1.2 | 12-10-2026 | Umar's changes of 11-10-2026. One user, no roles (Section 2, FR-8.5, FR-21.2, A19 resolved). Advance deduction tick per lot (FR-12.23) and seller advance return (FR-15.12). Partner money taken and returned on one screen (16.3, FR-16.2, FR-16.3, C20). Money lent to staff and others (16.5, control 0103, R32). Cash lent to a buyer (FR-15.11). Items in ledgers and the shop's total اگراہی (FR-19.8, FR-19.9). Urdu accuracy: 15,000-word list and spelling check (FR-7.23, FR-7.27, FR-7.66). Lot qty correction in an edit (FR-12.20). Open advances are no longer counted in "بیوپاری کے ذمے" (FR-15.9, R24, ادائیگی باقی tile). A return more than the open amount is refused. New tests E2E-28 to E2E-33 and E2E-30b, numbers from `worked_day.py`. |
| 1.3 | 15-10-2026 | Umar's changes of 14-10-2026. Partner capital in and out (17.4, FR-17.7, FR-17.8, opening capital in 17.1). The firm's own goods: purchase, sale with commission, cost of goods sold, write-off, profit or loss (12.10, FR-12.24 to FR-12.28, C21, accounts 01040001, 030119998, 04010006, 05020001, R33; A22 resolved; removed from "Won't"). Print any day and any party (FR-19.10) and the day summary R34; R15, R23, R25 and R29 moved to Must. Partner total کل رقم with a dated record of every movement (FR-17.9, C22, R31, partner strip). الگ کھاتہ, outside the partners (16.6, FR-16.6, FR-16.7, control 0305, A50). New tests E2E-34, E2E-34b, E2E-35, E2E-36, E2E-37, E2E-38, numbers from `worked_day.py`. |
| 1.4 | 15-10-2026 | Umar's change of 15-10-2026, for ease of use. One خرچہ screen with a type list for دکان خرچہ, ذاتی خرچہ and its واپسی, showing the split or the partner's total before save (16.2, 16.3, FR-16.8). Related entries share one screen (FR-7.67): وصولی, ادائیگی and پیشگی are one voucher screen (FR-15.10). The app points to the right type (FR-7.68, A51). New edge cases E56, E57 and test E2E-39. No money rule, posting or test number changed. |
| 1.5 | 15-10-2026 | Umar's answers of 15-10-2026 to the team's open questions after D1 and D2, approved by Umar. Commission is typed on the بکری line by default (FR-11.10, 12.4, Section 21), and there is no party + group rule level (11.1). "Newer wins" means the later start date (FR-11.5). A removed per-lot commission line moves its charge (FR-11.9). A seller's opening credit and a جنرل credit on a seller are payable (FR-7.59). A جنرل line never adjusts advances (16.4). Cash going below zero asks the user to confirm, on every entry (FR-6.7, A52 resolved). A return-by date cannot be before the entry date (FR-16.9). Opening balance rules (FR-10.9) and system accounts stay active (FR-10.4). New edge cases E58 to E60. No posting or test number changed. |
| 1.6 | 19-10-2026 | Umar's changes of 18-10-2026 and 19-10-2026, approved by Umar. Market trips (12.10 rewritten, FR-12.29 to FR-12.33): goods bought by some partners, paid from the shop's cash or their own pockets, sold in بکری or another shop; the commission is firm income; the trip's result goes only to its partners by its %, on the day of the sale; خریداری moves into the مارکیٹ سفر section; the accounts 01040001, 04010006 and 05020001 are removed (never in a released migration). Gardens (12.11, FR-12.34 to FR-12.38, control 0306). A garden, trip and fruit account per partner (0604). Fruit taken (16.7, FR-16.10 to FR-16.12). One cash account: no bank, no cheque, no mode (15.1, 16, 17.4, FR-10.4, 10.3); no cash count box (FR-16.1). Analytics complete and printable by day, month or year (FR-18.4, FR-18.8, views I to K, all views Must). Large lists page by page (FR-19.11). Party day sheet R35 (FR-19.12) and the collection round are Must; R7 with commission, R36, R37; R33 is the trip report. Dated statement lines (FR-17.10). E2E-38 renamed (خاندانی کھاتہ). Formulas C16, C21, C22 changed; C23 to C27 new. E2E-34 and E2E-34b numbers changed (approved by Umar); new tests E2E-40, E2E-40b, E2E-41, E2E-42 and E2E-43; edge cases E61 to E70; assumptions A53 to A64. Numbers from `worked_day.py`. The worked day (Section 4) and E2E-1 to E2E-33 and E2E-35 to E2E-39 are unchanged. |
| 1.7 | 22-10-2026 | Umar's answers of 20-10-2026 and 21-10-2026, from the client's own prints of 18-10-26, approved by Umar. The buyer pays commission and لاگا on every line; the seller pays commission only (11.2, A3 resolved). R35 in the client's two layouts, the day slip and the all-parties table (FR-19.12); R5 lines written "(item qty . rate)"; a clear layout for every other print (FR-19.13). The tag نشان after a name (10.2, FR-7.28, FR-7.40, FR-10.7, E38). The firm's contact persons and trade mark (Section 21). Glossary: تازہ بنام, سابقہ رقم, ٹوٹل رقم, بقایا, نشان. E2E-36 writes R5 the new way (no number changed). Still open: کیلا lines with no rate (A42). No money rule, posting or test number changed. |
| 1.8 | 27-10-2026 | Umar's change of 27-10-2026, made while testing the app himself, approved by Umar. Codes by type: seller 001, buyer 002, پختہ کھاتہ 003, item 004, packing 005, staff 006, each code the type number + a running number from 0 with no padding (FR-10.2, FR-10.3, FR-10.4, FR-13.1, Section 21, E29, A14 resolved). Short codes are the running number (FR-7.39, A35). The accounts list shows sellers, buyers and پختہ کھاتہ only, and the account form makes sellers and buyers only (10.2, FR-10.5). A click opens a picker's list; a list with no match offers a button to add one in the انڈکس and comes back with nothing lost (FR-7.10, FR-7.16, FR-7.41, FR-7.42). The worked day's codes change (4.3 to 4.5, 4.10, E2E-23, E2E-24, E2E-25, E2E-42, demo step 4). Trip goods (with a date to sell) and garden goods arrived come into a new sale as ready lots (FR-12.39, FR-12.40). Many items in one purchase or goods-arrived entry, with each line split between our shop and another shop (FR-12.41). Market trips (007) and gardens (008) get codes by type (FR-10.2, 12.10, 12.11); purchase numbers (خ-1) stay. No money rule, posting or test number changed. |
| 1.9 | 30-10-2026 | Umar's change of 30-10-2026, made while testing the app himself, approved by Umar: in every table, grid and print, the numbers of a column and its total are centred under the column heading (FR-19.13, NFR-USE-19). Example: 17 sits under the middle of تعداد. No money rule, posting or test number changed. |
| 1.10 | 30-10-2026 | Umar's second change of 30-10-2026, made while testing the app himself, approved by Umar. The dashboard tiles گاہکوں کے ذمے, بیوپاریوں کے ذمے, کل اگراہی and ڈوبی رقم (9.1). A person who owes and has died or left the business is moved to the dead amount: nothing posts, his account is inactive, a recovery is a وصولی (FR-10.10, R38, FR-19.9, R20 to R22, R24, R26, R34, R35, 6.2, FR-6.1, FR-7.29, FR-7.38, FR-7.44, FR-7.61, FR-13.8, FR-15.5, FR-15.6, view C, E71 to E73, A65 to A68, E2E-44); R20 and R24 Urdu titles. Every Urdu word says exactly what its English says (FR-7.23); tile names in 9.1 changed. Glossary: کل اگراہی, ڈوبی رقم. No posting or earlier test number changed. |
| 1.11 | 31-10-2026 | Umar's change of 31-10-2026, made while testing the app himself, approved by Umar: any missing record (expense head, item, packing, city, اڈا, مارکہ, store, narration, staff, account) is added on the spot from the picker with a short form and chosen at once (FR-7.69, new; FR-7.10, FR-7.42, FR-7.64, FR-10.8 updated; E74, E75). It replaces the v1.8 button that opened the انڈکس form. No money rule, posting or test number changed. |
| 1.12 | 31-10-2026 | Umar's second change of 31-10-2026, approved by Umar. The buyer's and the seller's day slips of R35 show each item with its total, then commission, لاگا and every deduction on their own lines, what was paid today, and the previous and new balance (FR-19.12; it replaces the v1.7 slip layout; the all-parties table stays). R34 is the day total sheet with commission lines and a short buyers' and sellers' table. Every print header carries the trade mark, the owners and the munshi, up to 6 people with a role (Section 21, FR-19.13). E2E-36 and E2E-43 gain these figures; A69 to A74. No money rule, posting or earlier number changed. |
| 1.13 | 31-10-2026 | Umar's answers of 31-10-2026, approved by Umar. One edit model on every entry: only typed fields change, every worked-out amount is locked and worked out again, the date never changes, the effect shows before save (FR-7.70). Every edit is stamped with its time and user where it happened, with an edit history (FR-7.71). An entry of a closed day is fixed by reopen, fix and close again in one step (FR-21.4, FR-12.18, FR-15.3). Accounts and lists can be added on any day; only an opening balance needs an open date (FR-10.11). A 4-digit partner code opens the partner section and the partner figures (FR-17.11, Section 21). New E76 to E80, E2E-45, E2E-46, A75 to A78. No money rule, posting or earlier test number changed. |
| 1.14 | 01-11-2026 | Umar's change of 01-11-2026, made while testing the app himself, approved by Umar. A picker's list opens as soon as the picker gets the focus, by keyboard too, and `↑` `↓` choose in it while it is open (FR-7.16; replaces the v1.8 keyboard rule). The lot seller list shows the sellers and پختہ کھاتہ; اپنا مال only while a trip has goods left, a garden only while it is open (12.3, FR-12.26). No money rule, posting or test number changed. |
| 1.15 | 01-11-2026 | Umar's second change of 01-11-2026, approved by Umar. A rare party can be made his own پختہ account (پختہ کھاتہ گاہک or پختہ کھاتہ بیوپاری) on the account form, with the next code of the پختہ series from 0032, his own ledger, and cash first (FR-13.11, new; glossary, FR-10.2, 10.2, FR-10.5; E81, A79). The shared پختہ کھاتہ with the name typed on the line stays. No money rule, posting or test number changed. |
| 1.16 | 01-11-2026 | Umar's third change of 01-11-2026, approved by Umar: ask the app (Section 7.7, FR-7.72 to FR-7.88, new). Amended 02-11-2026 before release, approved by Umar: every request goes only to the zero-retention Azure endpoint of the model, with no fallback; and the model is the brain. GPT-6 Luna through OpenRouter understands the question in any wording, reads every section through read-only tools (FR-7.87), adds up and compares through the app's calculator, estimates the months ahead through the app's estimator (FR-7.86), and writes the answer itself in the app language. Every figure in the answer is a reference that the app fills from the books and checks before it is shown, with a check from two places (FR-7.75, FR-7.76). The records the model reads are sent to OpenRouter (the client agreed, 02-11-2026), but never the fields FR-7.83 lists (phones, addresses, father or firm names, notes, reasons, other free text, keys, locked partner figures). Quick questions work with no internet; 20 questions a day; partner figures behind the partner code; a question log. Also G11, FR-5.5, 6.2, 7.3 (`F1`), Section 21, E82 to E98, A80 to A90, the Must list (26), demo step 10a, E2E-47 to E2E-63. Further amended 02-11-2026, approved by Umar ("the expert for the recommendation and future request"): advice from the books (FR-7.88, A90, E98, E2E-63); the exact instructions, tools and answer schema in `docs/deliverables/D6-system-prompt.md` (FR-7.87); a difference filled with its direction word (FR-7.75); a typed ✓, the other side's words and a figure of an earlier question refused (FR-7.76, E2E-60); a locked partner question pauses with no model call (FR-7.82); the FR-7.79 advice example moved to FR-7.88, reading R20, which has the days, not R21; after an independent review: the name check covers record names only, a difference between two places is final with no rewrite, side words are judged around each balance, the partner list of FR-7.82 follows FR-17.11, the market and person sent of a trip are never sent (FR-7.83), last N months and years (FR-7.77), and a target counts from the current month and is reached at the target (FR-7.86, E2E-59). No money rule, posting or earlier test number changed. |
| 1.17 | 02-11-2026 | Umar's change of 02-11-2026, approved by Umar: ask the app opens as a chat window above a floating round button at the end of the screen, in the app's colours, with a normal size and a large size for long tables (FR-7.73). The status bar keeps "سوال پوچھیں F1" and today's count. No money rule, posting or test number changed. |
| 1.18 | 02-11-2026 | Umar's change of 02-11-2026, approved by Umar: the assistant also helps with the app itself, from a guide of every screen through the tool `app_help` (FR-7.89); the answer's length follows the question (FR-7.89). With NFR v1.10, a model call may take 30 seconds and a question 60. No money rule, posting or test number changed. |
| 1.19 | 02-11-2026 | Umar's decisions of 02-11-2026 (plan 52), approved by Umar: up to 6 model calls and 12 tool calls a question, with the last call always answering (FR-7.87, E96, A87); find and read in one call when the find has one match; general advice marked "عام مشورہ" when the books cannot answer (FR-7.88). With NFR v1.12. No money rule, posting or test number changed. |
| 1.20 | 03-11-2026 | Umar's decisions for plan 52, approved by Umar: an estimate to the end of the year gives the whole year (the books so far plus the months left), and what-if estimates scale the same pace (FR-7.86). The fixed part of a request may be 11,000 tokens (NFR v1.13). No money rule, posting or test number changed. |
| 1.21 | 03-11-2026 | Lead's [A] item for Umar (A91), found by the live test of plan 52: "compare with the previous period" for whole calendar months means the whole months before (FR-18.1). No money rule or test number changed. |
| 1.22 | 03-11-2026 | Umar's changes of 03-11-2026, made while testing the app himself, approved by Umar. A new account can be added with its opening balance at any time: the opening posts on its date even when that day is closed, against ابتدائی فرق, the one exception to FR-6.6 (FR-10.11, 10.2, E79). There is no collection day: the اگراہی فہرست lists every buyer who owes, by collector and city (10.2, FR-15.6). No money rule or test number changed. |
| 1.23 | 03-11-2026 | Umar's second change of 03-11-2026, made while testing the app himself, approved by Umar. Munshis on prints are managed in the staff list: every active staff member whose job is منشی prints in every header after the people typed in Settings (at most 3 munshis, at most 6 people), and an inactive one does not. Each staff row has an active switch (10.3, Section 21, FR-19.13). No money rule, table or test number changed. |
| 1.24 | 04-11-2026 | Umar's change of 04-11-2026, approved by Umar: restore on a new computer from the first screen, with a choice between a new shop and bringing back the data from the cloud or a USB drive or file (FR-20.11, NFR-INS-2, NFR-INS-6). No money rule, table or test number changed. |
| 1.25 | 05-11-2026 | Umar's change of 05-11-2026, approved by Umar: simple real-time figure questions are answered by the app with a reviewed sentence and the checked figure, with no model call (FR-7.90). It changes his decision D6-Q1 of 02-11-2026 for those questions only. No money rule or test number changed. |
| 1.26 | 05-11-2026 | Umar's second change of 05-11-2026, approved by Umar: the estimator also estimates a balance (a party's balance, the cash in hand, what the buyers owe) at the same pace (FR-7.86). Display only, marked as an estimate. No money rule, posting or test number changed. |
| 1.27 | 06-11-2026 | Approved by Umar: a balance estimate also leaves out make-regular moves, partner capital and the dead amount (FR-7.86). The assistant's fixed part may be 11,500 tokens (NFR v1.15). No money rule, posting or test number changed. |
| 1.28 | 06-11-2026 | Umar's change of 06-11-2026 from his test of the بکری screen: a line over the lot's quantity gets its error at once and no figures, and every lot label names its seller and item (FR-12.6). No money rule, posting or test number changed. |
| 1.29 | 06-11-2026 | Approved by Umar: the shop sells vegetables as well as fruit; a Group filter (پھل / سبزی) on analytics views A, B, C, D, G and H and the sales reports R2 to R4, R7, R10 and R27, on screen, in print and in the assistant (18.1, FR-19.1). The assistant's fixed part may be 11,600 tokens (NFR v1.16). No money rule, posting or test number changed. |
| 1.30 | 06-11-2026 | Umar's change of 06-11-2026: an optional quality (درجہ: اول, دوم, سوم) on each sale line, from a fixed dropdown, printed on the buyer's and the seller's prints and known to the assistant (12.4, FR-12.42, glossary). No money rule, posting or test number changed. |
| 1.31 | 06-11-2026 | Umar's answers of 06-11-2026 on the Group filter: under a group, view G leaves its gardens column empty; an item's group stays changeable and its history moves with it; R27 under a group shows that group's lots of each bill, with a note in the print header (18.1, R27). No money rule, posting or test number changed. |
| 1.32 | 09-11-2026 | Umar's demo feedback of 08-11-2026, deliverable D8, with his decisions of 09-11-2026, approved by Umar. Money paid on the spot (موقع پر جمع) replaces the ادھاری / نقدی toggle (FR-12.44, the cash part of FR-12.9, 12.4, FR-15.4, R3, R4, R5). پختہ کھاتہ is removed and چکتا کھاتہ for walk-in buyers takes its place (Section 13 rewritten: FR-13.1 to FR-13.9, FR-13.10 and FR-13.11 retired; glossary, 4.5, 5.3, FR-7.29, FR-10.2, FR-10.4, FR-10.5, FR-10.10; R9 retired, other report numbers unchanged). Party ledgers (FR-12.43). Find by buyer, and a change of buyer or seller in an edit (FR-12.13, FR-7.70). A second print button for ادھاری تفصیل (FR-12.12). Printing on the Mac (FR-19.3). Digits 0 to 9 on every print, and the Urdu digits setting removed (FR-7.32, FR-19.6, Section 21). Backspace and Delete only delete (FR-7.91, FR-7.14, FR-7.15). A rate hint that never looks like a value (FR-7.53). Over-quantity and incomplete lines (FR-12.6). New E99 to E103, A92 to A99 and E2E-64 to E2E-69. No golden number changed: E2E-5, E2E-10, E2E-15 and E2E-43 change wording only (پختہ to چکتا, and the register becomes the day's sale book), and E2E-21 is replaced because two one-time credit buyers can no longer exist. No money rule changed: a line paid in full posts what a نقدی line posted, and a line paid 0 posts what an ادھاری line posted. |
| 1.33 | 10-11-2026 | Umar's decision of 10-11-2026, found by the money review of D8 plan 67: an edit that changes only the buyer of a line looks up the buyer-side rules again and keeps the seller side as stored; an edit that changes only the seller of a lot looks up the seller-side rules again and keeps the buyer side and the money paid on the spot as stored (FR-12.20). Also the key Shift+F9 for the second print button (7.3). No golden number changed. |
| 1.34 | 12-11-2026 | Umar's decisions of 12-11-2026 (plan 71), approved by Umar. Lot names with a counter per seller and item (FR-12.45, 12.3, FR-12.6, FR-12.14, R2, R6, R10, R27); a seller's lots printed sorted on R1 and R35 (FR-12.12, FR-19.12); the assistant asks which lot, or answers a named lot (FR-7.77). The بکری header keeps only the date and the بکری number (FR-12.2, 12.2); the other header fields are retired and kept read-only for old data; bill date is the entry date. Retired: truck memory and the same-truck warning (FR-7.49, FR-7.50, E22, E39), the header freight split (C17, FR-14.1), the header زمیندار as the first lot's seller (FR-7.51), the header store default, and R8. E2E-25 keeps only its seller memory part, with the lot name; new E2E-70, A100. No golden number changed. No money rule changed. |
