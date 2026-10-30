# Product Requirements Document (PRD)
# Mandi Software for Commission Agents (آڑھتی)

| | |
|---|---|
| Version | 1.11 (functional requirements; Umar's changes of 11-10-2026, 14-10-2026, 15-10-2026, 18-10-2026, 19-10-2026, 22-10-2026, 27-10-2026 and 30-10-2026, and 31-10-2026, and his answers of 15-10-2026 and 30-10-2026) |
| Date | 31 October 2026 |
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
| 1 | `PRD_Mandi_Software_v1.md` (this file) | Functional requirements | v1.11 |
| 2 | `NFR_Mandi_Software_v1.md` | Speed, scalability, reliability, backup quality, security, usability, Urdu and English quality | v1.6 |
| 3 | `Tech_Stack_v1.md` | Technologies, Urdu and English stack, repository structure, naming | v1.7 |
| 4 | `Code_Standards_v1.md` | How the code is written, validated and tested | v1.11 |
| 5 | `Architecture_v1.md` | Diagrams and the full data dictionary | v1.9 |
| 6 | `.claude/agents/money-calculations.md` | Claude Code agent for every piece of money code | v1.7 |

**Versions.** All six documents started at version 1.1, the baseline for build. Version 1.2 (12-10-2026) applies Umar's changes of 11-10-2026: one user with no roles, a choice to deduct an advance or not, partner money taken and returned, money lent to staff and others, cash lent to buyers, items in party ledgers, the shop's total اگراہی, and stricter Urdu spelling checks. Version 1.3 (15-10-2026) applies Umar's changes of 14-10-2026: partner capital (the money partners invest), the firm's own goods (bought and sold by the firm for profit or loss), and printing for any day and any party. Version 1.4 (15-10-2026) applies Umar's change of 15-10-2026: one خرچہ screen for shop expenses and a partner's personal spending, and related entries kept on one screen so the app is easy to use. Version 1.5 (15-10-2026) applies Umar's answers to the team's open questions after D1 and D2: commission typed on the بکری line by default, the rule tie, opening balances, a seller's opening credit, the cash warning and return-by dates. Version 1.6 (19-10-2026) applies Umar's changes of 18-10-2026 and 19-10-2026: gardens and market trips owned by the partners in them, one cash account with no bank, fruit taken by partners, complete analytics, the party day sheet, page-by-page lists, and dated lines in each partner's statement. Version 1.7 (22-10-2026) applies what Umar settled from the client's own prints of 18-10-2026: the buyer pays commission and لاگا and the seller commission only, three prints in the client's own layout (the buyer's day slip, ادھاری تفصیل and the all-buyers balance sheet), a clear layout for every other print, a short tag (نشان) after a name, and the firm's contact persons and trade mark on every print. Version 1.8 (27-10-2026) applies Umar's change of 27-10-2026, made while he tested the app himself: short codes by type (the first seller is 0010), an accounts list of sellers, buyers and پختہ کھاتہ only, and pickers that open on a click with a button to add what is missing. Version 1.9 (30-10-2026) applies Umar's change of 30-10-2026: in every table and print, a column of numbers and its total are centred under the column heading. Version 1.10 (30-10-2026) applies Umar's second change of that day: the dashboard shows buyers' dues, sellers' dues, the total اگراہی and the dead amount (ڈوبی رقم); a person who owes and has died or left the business is moved to the dead amount; and every Urdu word says exactly what its English says. Version 1.11 (31-10-2026) applies Umar's change of 31-10-2026: any missing list entry (an expense head, an item, a packing, a city, a seller ...) is added on the spot, from the list where it is needed, and is chosen at once. From now on, any change to a document raises its version (1.2, 1.3 ...) and adds a line to its change log. If the change affects other documents, they are updated in the same step.

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
| پختہ کھاتہ | Pukhta Khata | Shared account for one-time or rare parties who do not have their own account. The client's word is پختہ کھاتہ (not پکا کھاتہ). Charged like any other buyer or seller. | A walk-in buyer شاہد buys 10 پیٹی once |
| نگ | Nag | One unit or package (پیٹی، کریٹ، بوری، دانہ) | 10 نگ = 10 پیٹی |
| بکری | Sale | Sale | |
| بکری بک | Sale Book | Sale book; one بکری = one truck's goods sold | بکری نمبر 1 |
| لاٹ | Lot | One seller's goods of one item on a truck | رحیم's 50 پیٹی کیلا |
| قسم مال | Item | Item | کیلا، سیب، انگور، جاپانی پھل، مولی |
| پیکنگ | Packing | Packing unit | پیٹی، کریٹ، بوری، دانہ |
| مارکہ | Mark | Mark on crates showing whose goods they are | "AR" on رحیم's crates |
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
| ادھاری / نقدی | Credit / Cash | Credit sale / cash sale | |
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
Truck arrives -> بکری بک (truck details + lots + sales + live summary)
              -> Save = one posting event, which updates:
                   buyer ledgers, seller ledgers, پختہ کھاتہ,
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
| Buyer commission | **6%** of sale value, for every buyer, including one-time buyers in پختہ کھاتہ | [C] Umar: 6% by default. (The old video's bills show 5%, for example مولی 700 → 735. The client's current default is 6%.) |
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

Truck LES-1234 from سوات اڈا, header زمیندار رحیم (0010). One lot: seller رحیم (0010), item کیلا, packing پیٹی, 50 نگ received. Freight 3,000, paid by the firm. رحیم is given 5,000 cash on arrival (نقد). On the lot the munshi types مزدوری 250 (5 per نگ × 50), منشیانہ 50 and فون 20.

**Sales.** Each line works out its own commissions, so analytics can show commission by buyer as well as by seller.

| Buyer | Qty | Rate | Value (qty × rate) | Seller comm 6% | Buyer comm 6% | لاگا (qty × 8) | Buyer pays |
|---|---|---|---|---|---|---|---|
| اکرم ٹول ٹیکس 0020 | 20 | 600 | 12,000 | 720 | 720 | 160 | 12,000 + 720 + 160 = 12,880 |
| ارشد ملتانی 0021 | 30 | 580 | 17,400 | 1,044 | 1,044 | 240 | 17,400 + 1,044 + 240 = 18,684 |
| **Total** | **50** | | **29,400** | **1,764** | **1,764** | **400** | **31,564** |

How a cell is worked out, for example the first seller commission: 12,000 × 6 ÷ 100 = 720.

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

### 4.5 Event 2: بکری 2 (one-time buyer through پختہ کھاتہ)

Truck LHR-0077 from لاہور اڈا, header زمیندار کریم (0011). One lot: seller کریم (0011), item سیب, packing پیٹی, 10 نگ. Freight 1,000, paid by the firm. No نقد. On the lot the munshi types مزدوری 50 (5 per نگ × 10), منشیانہ 50 and فون 20.

A one-time buyer, شاہد (phone 0300-0000000), buys all 10 پیٹی at 1,200. He has no account, so the line goes to **پختہ کھاتہ (خریدار)**, code 0030. His name and phone are saved on the line. He is charged like any buyer: 6% commission and لاگا 8 per نگ. He pays cash, so the line is نقدی.

| Item | Working | Amount |
|---|---|---|
| Value | 10 × 1,200 | 12,000 |
| Seller commission 6% | 12,000 × 6 ÷ 100 | 720 |
| Buyer commission 6% | 12,000 × 6 ÷ 100 | 720 |
| لاگا | 10 × 8 | 80 |
| Buyer pays | 12,000 + 720 + 80 | 12,800 |
| Seller کل اخراجات | 1,000 + 720 + (10 × 5 = 50) + 50 + 0 + 0 + 20 | 1,840 |
| Seller صافی بکری | 12,000 − 1,840 | 10,160 |

Sale part: پختہ کھاتہ بنام 12,800. کریم جمع 10,160, کمیشن آمدن (بیوپاری) 720, کمیشن آمدن (گاہک) 720, لاگا 80, فون 20, مزدوری 50, کرایہ 1,000, منشیانہ 50. Total جمع = 12,800. Balanced.
Cash part (because the line is نقدی): روکڑ بنام 12,800, پختہ کھاتہ جمع 12,800.
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

### 4.10 The same day in English

Every name above has an English name (FR-7.24). The tests use these:

| Urdu | English | Code |
|---|---|---|
| رحیم | Rahim | 0010 |
| کریم | Karim | 0011 |
| اکرم ٹول ٹیکس | Akram Toll Tax | 0020 |
| ارشد ملتانی | Arshad Multani | 0021 |
| شاہد (one-time buyer) | Shahid | 0030, phone 0300-0000000 |
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
| 5 | پختہ کھاتہ | One-time parties register | 13 |
| 6 | آج کی بکری | List of today's بکری | 12.7 |
| 7 | باقی مال | Goods not yet sold | 12.8 |

خریداری (purchases) is in the مارکیٹ سفر section from v1.6, because every purchase belongs to a trip (12.10).

Items 2, 3 and 4 open one voucher screen, with that type already chosen (FR-7.67, FR-15.10).

FR-5.4 [A] Global search (`Ctrl+K`, or the search box at the top) finds any account, بکری number, truck number, voucher number or screen, by Urdu name, English name or code, with the smart search of FR-7.40.

FR-5.5 [A] Bottom status bar: user, today's date, day status (کھلا / بند), last cloud backup time with a coloured dot.

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
| Save a بکری | Buyer ledgers; seller ledgers; پختہ کھاتہ register; cash (for نقد and نقدی lines); income and payable accounts; partner share ledger; stock; analytics facts; dashboard; اگراہی list; seller payables; advance matching; bill matching; کرایہ کھاتہ (freight owed); روزنامچہ; trial balance; audit log; draft removed |
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

### 6.3 Rules that hold everywhere

FR-6.1 [C] No screen keeps its own separate totals. Every total is read from the posted records (journal, partner share ledger, stock moves, analytics facts), all written in the same save, so totals can never disagree between screens. (The dead amount tiles also read `dead_marks`, which only sorts people into groups and holds no money, FR-10.10.)
FR-6.2 [C] A journal entry whose بنام total does not equal its جمع total is rejected. This cannot be bypassed by any user.
FR-6.3 [C] A posted entry is never changed or deleted. Corrections are made with a reversing entry, so the history is always complete.
FR-6.4 [C] All money is whole rupees (Section 22). The same rounded number is used on every side: on the bill, in the journal, in the partner split and in analytics.
FR-6.5 [C] Every posting that touches an income account or a shop expense account writes partner share rows in the same save, whatever module it comes from (including جنرل). So, for any period, the sum of all partner shares equals income minus shop expenses exactly.
FR-6.6 [A] **Dates and open days.** Days are closed in order. The system keeps one "closed up to" date. A posting can only be dated after that date and no later than today. Reopening is described in FR-21.2.
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
- **Where.** Every picker on an entry screen: accounts (seller, buyer, ادھار رقم person, الگ کھاتہ), item, packing, city, اڈا, مارکہ, store, standard narration, expense head and staff. Not on filter boxes (reports, analytics, list filters, find, the rule test).
- **How.** The open list always ends with one row, for example "+ نئی خرچہ مد شامل کریں" (Add a new expense head). If text was typed, the row names it: "+ «بجلی» کو نئی خرچہ مد بنائیں". A click, or `↓` to the row and `Enter`, or `F4` in the picker, opens a short form over the screen. The typed text is already in the name.
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

  The code is given by itself (FR-10.3). Similar records already in the list show under the name while typing (FR-7.63, FR-10.7), and one can be picked instead.
- **After save.** The record is saved in its master list, is chosen in the field at once, and the cursor moves on with `Enter` as usual. It is in the list next time, on every screen. `Esc` closes the form and leaves the field as it was. Nothing typed on the screen underneath is lost.
- **More details.** A link "مکمل فارم" (Full form) in the short form opens the انڈکس form of that type over the screen, as in v1.8, for fields the short form does not ask (opening balance, credit limit and others).

Example: on the خرچہ screen the munshi opens خرچہ مد and the list is empty, saying "ابھی کوئی خرچہ مد نہیں". He types بجلی and presses `Enter` on "+ «بجلی» کو نئی خرچہ مد بنائیں". The short form shows بجلی and Electricity. He presses `F10`, and بجلی is chosen in the field. Next time بجلی is in the list.

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

Example for a sale line: buyer code `Enter`, qty `Enter`, rate `Enter` and the cursor is on the buyer code of a new line. Three fields, three `Enter`s (NFR-USE-6).

FR-7.15 [C] **Forms.** In normal fields, `↑` and `↓` move to the field above or below. When the user enters a field its whole value is selected; while it is fully selected, `←` and `→` move to the neighbouring field. Once the user types or moves the cursor, `←` and `→` move the cursor inside the text, and when the cursor is already at the visual edge of the text in the arrow's direction, the next press moves to the neighbouring field.

FR-7.16 [C] **Pickers.** Typing in a picker opens its list of matches. While the list is open, `↑` `↓` move through matches and `Enter` picks. While the list is closed, `↑` `↓` move between fields like any other field. `Alt+↓` opens the full list without typing. Arriving in a picker by keyboard (`Enter`, `Tab`, arrows) never opens the list; only a mouse click, typing or `Alt+↓` does (v1.8).

FR-7.17 [C] **Menus, lists and reports.** In the main bar and the بکری side menu, `↑` `↓` move between items and `Enter` opens. `←` moves from the menu into the screen, `→` moves back to the menu. In lists and reports, `↑` `↓` move between rows, `Enter` opens the selected row, `Page Up` / `Page Down` move a page.

FR-7.18 [C] **Esc, in this order:** close an open list or dialog; else cancel typing in the current cell; else leave the screen (asking first if there is unsaved work, FR-7.9).

FR-7.19 [A] **Dates.** In a date field, `+` = next day and `-` = previous day (arrows still move between fields). Typing `10` alone fills the current month and year; `10-9` fills the current year.

FR-7.20 [A] **Numbers.** Amount and quantity fields accept the number keypad. Thousands separators appear by themselves. Only whole numbers are accepted for money (Section 22).

### 7.3 Other keys

All keys are shortcuts; every action also has a button. The app turns off the built-in browser keys of the Windows web view (reload, browser find, browser zoom, browser print), so these keys only do what is listed here.

| Key | Where | Action |
|---|---|---|
| `F2` | Any entry screen | New (بکری or voucher on the current screen). Inside a grid, F2 does not start cell editing. |
| `F3` | Any entry screen | Find |
| `F4` | In a picker | Add a new account without leaving the screen |
| `F9` | Any screen | Print |
| `F10` or `Ctrl+S` | Entry screens | Save |
| `Ctrl+Enter` | Entry screens | Save, print the ticked documents and start a new one (FR-7.57) |
| `Ctrl+Delete` | In a grid, before save | Remove the current row |
| `Alt+U` | In a text field | Typing mode: رومن سے اردو, اردو کی بورڈ, انگریزی (FR-7.21) |
| `Alt+L` | Anywhere | Language: اردو or English (FR-7.30) |
| `Ctrl+K` | Anywhere | Global search |
| `Ctrl+I` | On a calculated amount | How was this worked out? (FR-7.43) |
| `Ctrl+D` | In a sale-line cell | Copy the value from the line above (FR-7.54) |
| `Alt+Z` | While the saved message shows | Undo the save (FR-7.47) |
| `Ctrl +` / `Ctrl −` / `Ctrl 0` | Anywhere | Text size bigger / smaller / normal |
| `Alt+B` | Anywhere | New بکری |
| `Alt+V` | Anywhere | New وصولی |
| `Alt+A` | Anywhere | New ادائیگی |
| `Alt+P` | Anywhere | New پیشگی |
| `Alt+K` | Anywhere | New خرچہ, starting as دکان خرچہ (16.2) |
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

FR-7.24 [C] **Every name has an Urdu and an English name.** This covers accounts, cities, items, packings, truck stands, stores, standard narrations, expense heads, staff, partners, the firm, and one-time (پختہ کھاتہ) people. Both names are required. The user types one; the other is filled by itself:
- Typed in English letters (and converted to Urdu by FR-7.22): the English name is exactly what was typed, with capital first letters. `arshad multani` → "Arshad Multani".
- Typed in Urdu letters: the English name is suggested from the Urdu. ارشد ملتانی → "Arshad Multani".
- In the English app the order is reversed: the user types the English name and the Urdu name is suggested from it.

The filled name stays editable. Once the user edits it, it is never overwritten again.

FR-7.25 [A] **What is not translated.** Codes, amounts, dates, phone numbers, truck numbers, tags (نشان), مارکہ text, the market, supplier and other shop typed on a trip, garden or پھل entry, and free text (notes, hand-typed narrations, reasons) are stored once, exactly as typed, and look the same in both languages.

FR-7.26 [A] **The app learns.** When the user picks another suggestion or corrects a converted word, the pair (English spelling → Urdu spelling) is remembered and offered first next time on this computer. The Owner can see and remove learned pairs in ترتیبات. Learned pairs are part of every backup. Learning never changes a name that is already saved.

FR-7.27 [A] **Starting word list.** The app ships with a reviewed word list of at least 15,000 entries (NFR-URD-24): common Pakistani first names, family and firm names, cities and towns of Punjab and Khyber Pakhtunkhwa, fruits, vegetables, packings and the mandi words of the glossary. When Umar exports the old software's account names (about 1,572), they are added after review. Words not in the list are converted by spelling rules and checked by FR-7.66. Everything works offline.

FR-7.28 [A] **Script checks.** An Urdu name must be in Urdu letters. An English name must be in English letters. Digits, spaces and `. - ' ( ) & /` are allowed in both. A wrong script shows "اردو نام اردو حروف میں لکھیں" or "انگریزی نام انگریزی حروف میں لکھیں". The tag (نشان, 10.2) is the one place English letters go with an Urdu name: it is its own field, not part of the name (v1.7).

FR-7.29 [A] **One-time people.** In a پختہ کھاتہ name or phone field, typing shows **one** list: first the one-time people already recorded (both names, phone, last date), then the Urdu spellings of the typed word (FR-7.22). Picking a person reuses his exact names and phone, so all his entries stay under one person key (FR-13.5). For a new person, the other-script name is filled by FR-7.24. A person in the dead amount shows marked ڈوبی and cannot be picked for a new line (FR-10.10).

FR-7.66 [A] **Spelling check before save.** (Numbered after FR-7.65 because it was added in v1.2.) A converted word that came only from the spelling rules, not from the word list or a learned pair, is marked in amber with "ہجے دیکھ لیں" (check the spelling). On save, if a name still has such a word, the app shows the name with that word marked and asks the user to confirm or fix it. Confirming learns the pair (FR-7.26). The same applies to an English name suggested from Urdu by rules only. A rule-only spelling is never saved without the user confirming it. Free text (notes, reasons) is not checked.

### 7.5 Language switch: اردو | English [C]

Umar: a toggle that stays on Urdu by default. Switched to English, the whole app, including the words and the names, is in English.

FR-7.30 [C] A switch "اردو | English" is always visible in the top bar (FR-5.6) and on the login screen. `Alt+L` also switches. At first run the app is in اردو. The last choice is remembered, and the login screen uses it.

FR-7.31 [C] **Everything changes.** In English, these are all in English: labels, menus, buttons, hints, messages, confirm boxes, report titles and columns, chart labels, month names, amounts in words, and every name (FR-7.24). The layout becomes left-to-right: the main bar moves to the left, and forms, grids and charts are mirrored. In Urdu, all of these are in Urdu, right-to-left.

FR-7.32 [C] **The data never changes.** Switching changes only how things are shown. Codes, amounts, dates, balances and saved records are identical in both languages. Digits are English digits (0-9) on screen in both, with the same number grouping (Section 21). Only Urdu prints can use Urdu digits, by the Printing setting.

FR-7.33 [C] **Instant, and nothing is lost.** The switch works at once on the open screen, with no restart and no logout. Unsaved entries, the selected row, open lists, filters and the cursor position stay. Example: halfway through بکری 1, the user switches to English. The lines now read "Akram Toll Tax" and "Arshad Multani", the item reads "Banana", the packing "Box", all numbers are the same, and the user carries on typing.

FR-7.34 [C] **Keys follow what the user sees.** Arrow keys move in the direction shown on screen in both layouts (FR-7.13). In English the next field is usually to the right.

FR-7.35 [A] **Search finds both.** Pickers and global search find a record by its Urdu or its English name, whichever language the app is in.

FR-7.36 [A] **Prints.** Every print and PDF uses the app language by default. The print box has a language choice (اردو / English), so, for example, a buyer slip can be printed in Urdu while the app is in English. Excel export uses the app language. The readable backup pack (FR-20.5) uses the default language set in Section 21.

FR-7.37 [A] **Mandi words in English.** Where English has a clear word, it is used (Sale, Receipt, Payment, Advance, Commission, Freight, Labour). Mandi words with no exact English word keep their usual Roman name (Laga, Munshiana, Pukhta Khata, Adda, Nag). The glossary's English column (Section 3) and the English names of the seeded accounts (FR-10.4) are the only source.

### 7.6 Smart assistance: the app fills, the user checks [C]

Umar: once a party is in the index, typing his code anywhere should bring up his whole detail. Make every page as easy as possible.

The principle: **the user types as little as possible, the app fills what it already knows, and the user only checks.** Nothing filled by the app is saved without being visible on screen, and every filled value can be changed. The app never fills a rate or an amount of money by guessing.

**Everywhere**

FR-7.38 [C] **A code brings the whole party.** In every account field on every screen (بکری header and lots, sale lines, وصولی, ادائیگی, پیشگی, جنرل, ledgers, statements, report filters), the moment a full code is complete the name fills and a **party card** appears next to the field. A short code (FR-7.39) is taken on `Enter` or on leaving the field, because `1` may be the start of `12`; while typing, the card previews the top match. It shows:
- For everyone: name with father or firm, city, phone, current balance (بنام or جمع), last entry date.
- For a buyer: credit limit and how much of it is used; oldest unpaid bill and its days.
- For a seller: open advance and amount payable.
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

FR-7.49 [A] **Truck memory.** Typing a truck number that was used before offers it in a list. Picking it fills اڈا, زمیندار and driver from that truck's last بکری (shown with its date). All are editable.

FR-7.50 [A] **Same truck today.** If the truck already has a بکری today, the app says "اس ٹرک کی آج بکری نمبر 5 پہلے سے موجود ہے" with two buttons: کھولیں (open it) and نئی بکری جاری رکھیں (continue with this new one). This stops the same truck being entered twice by mistake.

FR-7.51 [A] **Seller memory.** Choosing a lot's seller fills item and packing from his last lot, and مارکہ from his default مارکہ, else his last lot (all editable). His card shows his open advance and payable. A new lot's seller starts as the previous lot's seller (or the header زمیندار for the first lot).

FR-7.52 [A] **Remaining quantity.** A new sale line's quantity starts as the lot's remaining quantity, already selected, so typing replaces it. Example: 50 received and the first line sold 20, so the second line starts at 30. A row whose buyer and rate are both empty is a blank row: the preview and the save ignore it, and save removes it.

FR-7.53 [A] **Rate help, never a guess.** The rate field stays empty and shows a grey hint with the last rate of the same item and packing: from the earlier lines of this بکری first (saved or not), else today's saved sales, else the last day it sold. Example: "آج کا آخری ریٹ 600". The hint is never saved by itself. The user types the rate, or presses `Ctrl+D` to copy the rate of the line above.

FR-7.54 [A] **Copy from the line above.** `Ctrl+D` in any sale-line cell copies the value of the same cell in the line above, as in a spreadsheet.

FR-7.55 [A] **Rate check.** When the rate check setting is on (Section 21), a rate more than 30% above or below the item's average rate of the last 7 days shows an amber warning "ریٹ معمول سے بہت مختلف ہے (اوسط 600)". It never blocks saving.

FR-7.56 [A] **Buyer memory.** Choosing a buyer on a line shows his card (FR-7.38). The payment type comes from his credit setting (Section 12.4). If the line takes him over his credit limit, the warning shows on the line straight away (FR-12.4), not only at save.

FR-7.57 [A] **Save, print and next in one step.** `Ctrl+Enter` saves, prints the documents ticked in the "محفوظ کرنے کے بعد پرنٹ" list (seller bill, buyer slips, پختہ کھاتہ slip), and opens a new بکری. The ticked list is remembered per user.

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
| Buyers' dues | گاہکوں کے ذمے | Sum of buyer debit balances, including open پختہ کھاتہ dues per person, leaving out anyone in the dead amount (FR-10.10). Always equals the میزان of R20 (FR-19.9). |
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

- Today's بکری: number, truck, seller, خام بکری, صافی بکری, status.
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
| پختہ کھاتہ | **003** | 0030 (buyer side), 0031 (seller side); system accounts, 13 | [C] |
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
| 0031 | پختہ کھاتہ (بیوپاری) | Pukhta Khata (Seller) | Seller | |
| 030119998 | اپنا مال (the seller of a trip's goods) | Own Goods (Trips) | Seller, used only as the seller of a trip's lots and for their charge rules. It never holds a balance (FR-12.26). | |
| 0305xxxx | One account per الگ کھاتہ (family or any other) | Separate Khata, <name> | Party, outside the partners (16.6) | No |
| 0306xxxx | One account per garden (12.11), made with the garden | Garden, <name> | Party: holds the garden's sales money until it is closed | No: its money goes only to the garden's partners (FR-12.38) |
| 03040000 | نقدی منڈی (cash buyer with no name) | Naqdi Mandi (Cash Buyer) | Buyer | |
| 0030 | پختہ کھاتہ (خریدار) | Pukhta Khata (Buyer) | Buyer | |
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
| Opening date | ابتدائی تاریخ | Date | Yes | Default today; must be an open date | [S] |
| Opening debit | ابتدائی بنام | Money | No | Only one of debit or credit | [S] |
| Opening credit | ابتدائی جمع | Money | No | | [S] |
| Opening is an advance | ابتدائی پیشگی | Checkbox | No | Sellers only, with an opening debit: the opening debit counts as an advance (FR-15.8) | [A] hidden |
| Credit allowed | ادھار کی اجازت | Checkbox | No | Ticked by default (as in the old account form). Off = cash only | [S] |
| Credit limit | ادھار حد | Money | No | Warning on بکری when exceeded | [A] |
| Commission rules | کمیشن اصول | Link | No | Opens this party's rules (Section 11) | [C] |
| Default مارکہ | مارکہ | Select | No | Sellers | [A] |
| Collector | اگراہی والا | Select (staff) | No | Who collects from this buyer | [A] hidden |
| Collection day | اگراہی کا دن | Select (weekday) | No | For the اگراہی plan | [A] hidden |
| Active | فعال | Checkbox | Yes | Inactive = hidden from pickers, kept in reports. A person in the dead amount still shows, marked ڈوبی, in the وصولی and ادھار رقم واپسی pickers (FR-10.10) | [A] |
| Note | نوٹ | Text | No | | [A] |
| System fields | | Auto | | Created by and at, updated by and at | [A] |

FR-10.5 [S] List screen columns: code, Urdu name, English name, city, opening date, opening debit, opening credit, credit allowed, current balance. Footer: total debit, total credit. [C] The list shows only sellers, buyers and the two پختہ کھاتہ accounts, with a type filter: all, بیوپاری, گاہک, پختہ کھاتہ (Umar, 27-10-2026). روکڑ, the payables, income, capital and every other account are seen in the chart of accounts (FR-10.1).
FR-10.6 [A] An account with postings cannot be deleted, only made inactive.
FR-10.7 [A] Duplicate warning when name, tag and phone match an existing account. اسلم BA and اسلم are two different people. Similar names show while typing (FR-7.63).
FR-10.8 [C] Quick add from any picker (FR-7.69) opens a short form without leaving the current screen. The full انڈکس form is still used to edit a record later.
FR-10.9 [A] **Opening balances post.** An opening debit or credit posts as an opening entry dated the opening date: the account on its side, and ابتدائی فرق (06019999) on the other side. When all openings are entered correctly, ابتدائی فرق is 0. The trial balance shows it, so a wrong opening is visible. Changing an opening reverses the old opening posting and posts the new one.
- [C] (Umar, 15-10-2026) Each account has at most one opening balance: one debit or one credit. Changing it reverses that account's opening posting and posts the new one, as above.
- [C] An opening is refused on income (head 04) and expense (head 05) accounts, ابتدائی فرق, نقدی منڈی, the two پختہ کھاتہ accounts and اپنا مال (030119998, which never holds a balance, FR-12.26). From v1.6 it is also refused on بینک (01020001), a garden's account (0306) and a partner's garden, trip and fruit account (0604) [A].
- [C] A partner's drawings account (0603) can take an opening debit. It counts as money he took (C20, C22), dated the opening date. Example: the app starts in mid-year and partner الف has already taken 2,000 this year.
FR-10.10 [C] **Dead amount (ڈوبی رقم).** (Added in v1.10.) Umar (30-10-2026): a person who owes the shop may die or leave the business. The shop still tries to recover the money, and his account is made inactive. What he still owes stays in the shop's total اگراہی (کل اگراہی), but it is shown on its own as ڈوبی رقم, because it may never be collected.
- **Who.** Anyone who owes the shop: a buyer, a seller whose ledger balance is بنام, a پختہ کھاتہ person (by his person key, FR-13.5), or a person with ادھار رقم (control 0103). Partners, الگ کھاتے, gardens and system accounts are never moved.
- **How.** On the account form, the party card, or a row of R20, R24 or R32, the button "ڈوبی رقم میں ڈالیں" (Move to dead amount) shows the balance owed and asks for the date (today by default; an open date, and not before his last posting), the reason (فوت ہو گئے died, کاروبار چھوڑ دیا left the business, دیگر other, which needs a note) and a note. Saving makes the account inactive (FR-10.6) and records the move with the balance on that date. For a پختہ کھاتہ person nothing is made inactive (the پختہ accounts stay active, FR-10.4): his person key is marked, and a new line or lot with it is refused (`ERR_PERSON_IN_DEAD_AMOUNT`). An account or person that owes nothing cannot be moved (`ERR_NOTHING_OWED`); an account is simply made inactive. A linked account (E10) is not moved by itself; the screen shows its balance [A].
- **Money.** Nothing posts. The balance stays on his account, so the trial balance, the partner split and every partner's کل رقم do not change (A65). The dead amount is his whole ledger بنام balance: for a seller, his open advance and open payables are netted into it and leave پیشگی باقی and بیوپاریوں کو ادائیگی باقی; for ادھار رقم, his open loans leave ادھار رقم باقی. All of it joins کل اگراہی [A].
- **Recovery.** A وصولی on him (or an ادھار رقم واپسی, for money lent) is still allowed, with its normal matching. The pickers of those two entries show him marked "ڈوبی". A recovery more than he still owes is refused (`ERR_RETURN_EXCEEDS_OPEN`, with the amount still owed). Any other new entry on him is refused, as on any inactive account (`ERR_ACCOUNT_INACTIVE`). His dead amount goes down by what is recovered; at 0 the move stays in his history as "پوری وصول ہو گئی" (fully recovered).
- **Corrections.** An edit, cancel or undo of a posting made before the move is still allowed while its day is open. It changes the dead amount, and R38 shows it as a correction, not as a recovery. An edit or cancel of a recovery changes the recovered amount itself: 4,000 recovered and then cancelled shows recovered 0, and edited to 3,000 shows 3,000. A receipt saved before the move is already inside what he owed then, so its cancel is a correction [A]. An edit of an earlier ادائیگی, پیشگی or جنرل entry on him is refused; it can be cancelled and entered again after he is made active.
- **Undo.** "دوبارہ فعال کریں" (Make active again), with a reason, ends the move and makes the account (or the person key) active. His balance goes back into the normal tiles and lists.
- **Where it shows.** The tiles of 9.1 and report R38. His party card and ledger heading say ڈوبی رقم with the date and reason. He leaves R20 to R22, the بیوپاری کے ذمے list of R24, R26, R32, the attention list (FR-7.44), the dashboard list of oldest dues (9.3) and the collection round (FR-7.61). Every move, recovery and undo is in the audit log (FR-21.3).

Example (E2E-44): after Events 1 to 5, ارشد owes 18,684 and dies on 11-10-26. Buyers' dues 2,880; dead amount 18,684 (1 person); total اگراہی 2,880 + 0 + 18,684 = 21,564, the same as before. His son pays 4,000 on 12-10-26: dead amount 14,684, total اگراہی 17,564, cash 58,484 + 4,000 = 62,484, trial balance still 85,048 = 85,048 (روکڑ 62,484, ارشد 14,684).

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
| Staff | عملہ | Urdu name, English name, job (منشی، اگراہی والا، مزدور), phone, linked ادھار رقم account (made by itself the first time he is lent money, FR-16.4) | [A] |

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
FR-11.7 [C] پختہ کھاتہ follows the same rules as any party (6% by default). A rule on a پختہ کھاتہ account changes the rate for all one-time parties on that side.
FR-11.8 [A] A "test a rule" box: pick seller, buyer, item, qty and rate, and see which rule wins and the amount, before saving the rule.
FR-11.9 [A] Seller commission "فی لاٹ" is charged once per lot, on the first sale line of the lot's first posting. Later sales of that lot do not charge it again. [C] If an edit removes that line, the charge moves to the first remaining line of that lot in that posting, with its stored method and value (no new rule lookup), so the lot is still charged exactly once (Umar, 15-10-2026). This is the one change FR-12.20 allows on a line the user did not edit. If no line of that lot is left in that posting, the charge goes with the removed line, as before. Example: a per-lot commission of 500 sits on line 1 of 2; line 1 is removed; line 2 now carries 500, and the lot's seller commission includes 500 once.
FR-11.10 [A] Lot charges (مزدوری, منشیانہ, فون) are suggested by the rules but are always editable amounts on the lot. [C] Umar (15-10-2026): the default is 6% for every seller, buyer and item, and the commission agent can change it on the بکری itself. So seller commission, buyer commission and لاگا are editable on each sale line by default (the "allow override" setting starts on; the Owner can turn it off in Section 21). Example: on one line he types 4% for the seller and 5% for the buyer instead of 6%. An edited value is saved with method "دستی" (manual). A rule (11.1) is for a rate that should apply every time, for example رحیم always 5%.

---

## 12. Module: بکری بک (Sales voucher)

### 12.1 What the screen must let the user do

The layout is our own design. Functionally, one screen must let the user:

1. Enter the truck details (header).
2. Add one or more lots (one per seller per item on the truck).
3. For the selected lot, add its sales to buyers.
4. See a live summary for the lot and for the whole بکری: seller side, buyer side, partner split.
5. Save, print, find, move between بکری, and cancel.

FR-12.1 [S] Lots and their sales are linked: selecting a lot shows only its sales. A "پوری بکری" view shows all sales of all lots.
FR-12.2 [C] Everything fits on one screen without scrolling sideways (FR-7.2). Less-used fields sit behind a "مزید" (more) toggle per lot, so the main view stays simple.

### 12.2 Header fields

| Field | Urdu | Type | Req | Default / rule | Source |
|---|---|---|---|---|---|
| بکری number | بکری نمبر | Auto | Yes | Given at first save (drafts have no number). Never reused. | [S] |
| Book / series | بک | Text | No | Optional second number from the old screen (for example a yearly series) | [S] meaning [A] |
| Entry date | تاریخ | Date | Yes | Today. Must be an open date (FR-6.6). | [S] |
| Bill date | تاریخ بل | Date | Yes | Same as تاریخ. Can be later, when the seller's bill is made. | [S] field, meaning [A] |
| Truck number | ٹرک نمبر | Text | No | Suggestions from past entries; fills اڈا, زمیندار and driver (FR-7.49, FR-7.50) | [S] |
| Zamindar | زمیندار کوڈ + name | Seller picker | No | Becomes the default seller for new lots | [S] |
| Truck stand | اڈا کوڈ + name | Picker | No | Default from the zamindar's اڈا | [S] |
| Driver name | ڈرائیور | Text | No | | [A] hidden |
| Driver phone | ڈرائیور فون | Text | No | | [A] hidden |
| Total truck freight | کل کرایہ | Money | No | If filled, a "تقسیم" button splits it across lots by نگ received (C17) | [A] hidden |
| Store | سٹور | Picker | No | Default store for lots | [S] |
| Note | نوٹ | Text | No | | [A] |
| Status | حالت | Read-only: محفوظ، منسوخ | | | [A] system |
| System fields | | Auto | | Created/updated by and at, print count | [A] system |

### 12.3 Lot fields

Fields seen [S]: بزگر code, بزگر name, فون, منشیانہ, نقد, کرایہ سٹور, سٹور. The old grid scrolled sideways, so it had more fields than the video shows. We add every input the calculation needs [A].

| Field | Urdu | Type | Req | Default / rule | Source |
|---|---|---|---|---|---|
| Lot no | لاٹ | Auto (1, 2, 3 ...) | Yes | | [A] system |
| Seller code | بزگر کوڈ | Seller picker (includes پختہ کھاتہ بیوپاری, اپنا مال, الگ کھاتے and open gardens) | Yes | Previous lot's seller, else header زمیندار (FR-7.51) | [S] |
| Seller name | بزگر | Read-only, or typed if پختہ کھاتہ | | | [S] |
| One-time seller name | نام | Text, Urdu and English (FR-7.29) | If پختہ کھاتہ | Suggests known one-time people | [C] |
| One-time seller phone | فون نمبر | Text | If پختہ کھاتہ | Needed to pay him later | [A] |
| Trip purchase | خریداری | Picker of purchases of open trips with stock left | If the seller is اپنا مال | Fills item and packing from the purchase (FR-12.26) | [C] |
| Item | قسم مال | Picker | Yes | Seller's last item (FR-7.51) | [A] hidden |
| Packing | پیکنگ | Picker | Yes | Seller's last packing, else the item's default | [A] hidden |
| Mark | مارکہ | Picker | No | Seller's default, else his last مارکہ | [A] hidden |
| Quantity received | تعداد | Whole number | Yes | More than 0 | [A] hidden |
| Weight received | وزن (کلو) | Number, up to 3 decimals | If the item is sold by weight | | [A] hidden |
| Freight | کرایہ | Money | No | From header split, editable | [S] summary |
| Freight paid by | کرایہ ادا | Select: آڑھتی، بیوپاری | Yes | آڑھتی. If بیوپاری (the seller paid the driver himself), freight is shown but not deducted or posted. | [A] hidden |
| Labour | مزدوری | Money | No | Typed. Filled by a rule only if the commission agent made one. Starts at 0. | [S] |
| Munshiana | منشیانہ | Money | No | Typed, or from a rule. Starts at 0. | [S] |
| Phone charge | فون | Money | No | Typed, or from a rule. Starts at 0. | [S] |
| Cash paid now | نقد | Money | No | Cash given to the seller in this posting | [S] |
| Deduct from advance | پیشگی سے کاٹیں | Tick | | Shown only when the seller has an open advance, with the open amount. Ticked by default (FR-12.23). | [C] |
| Store rent | کرایہ سٹور | Money | No | Typed by the munshi | [S] |
| Store | سٹور | Picker | No | Header store | [S] |
| Seller commission | کمیشن | Read-only | | Sum of the lot's line commissions | [C] |
| Sold qty | فروخت | Read-only | | Sum of this lot's sold qty in all postings | [A] system |
| Written-off qty | ضائع | Read-only | | FR-12.16 | [A] system |
| Balance qty | باقی | Read-only | | Received − sold − written off − taken as fruit (C16). Red if more than 0. | [A] system |
| Lot gross | خام بکری | Read-only | | | [S] |
| Lot net | صافی بکری | Read-only | | | [S] |

FR-12.23 [C] **Advance deduction is a choice per lot.** (Numbered after FR-12.22 because it was added in v1.2.) Umar (11-10-2026): the advance is deducted by default, and the munshi can untick it. Ticked: the seller credit of this lot adjusts his open advances, oldest first (FR-15.8). Unticked: the advance stays open and the whole seller credit is payable, so he can take his money now and repay the advance later. Later sales of the lot follow the same tick; an edit can change it. The tick changes only matching, never the journal. Example E2E-28.

### 12.4 Sale line fields

Fields seen [S]: تاریخ, کوڈ گاہک, گاہک, تعداد بیوپاری, ریٹ بیوپاری, رقم بیوپاری, تعداد گاہک, ریٹ گاہک, کمیشن ریٹ, لاگا ریٹ, رقم گاہک, میزان.

| Field | Urdu | Type | Req | Default / rule | Source |
|---|---|---|---|---|---|
| Line no | نمبر | Auto | Yes | | [A] system |
| Date | تاریخ | Read-only | | The date of the posting this line belongs to | [S] |
| Buyer code | کوڈ گاہک | Buyer picker (includes نقدی منڈی and پختہ کھاتہ) | Yes | Code, short code, Urdu or English name (FR-7.39, FR-7.40); shows the party card (FR-7.38) | [S] |
| Buyer name | گاہک | Read-only, or typed if پختہ کھاتہ | | | [S] |
| One-time buyer name | نام | Text, Urdu and English (FR-7.29) | If پختہ کھاتہ | Suggests known one-time people | [C] |
| One-time buyer phone | فون نمبر | Text | If پختہ کھاتہ and ادھاری | | [A] |
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
| Payment type | ادھاری / نقدی | Toggle | Yes | ادھاری if the buyer has credit allowed, else نقدی. نقدی منڈی is always نقدی. | [S] |
| Credit warning | | Icon | | Shows if over credit limit | [A] |
| Note | نوٹ | Text | No | | [A] |

FR-12.3 [A] By default each line has one quantity and one rate, and the buyer rate equals the seller rate. A "ریٹ مختلف" toggle on the line lets the buyer rate differ. The difference (buyer value − seller amount) posts to ریٹ فرق (income, shared). It can be negative (a loss).
FR-12.4 [A] Credit limit exceeded: warning (Owner setting can make it a block).
FR-12.5 [C] پختہ کھاتہ lines require the one-time name. If ادھاری, phone is also required.
FR-12.6 [A] A lot's sold qty can never be more than its received qty.

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
| Buyer bills: credit / cash | ادھاری / نقدی | Sums of buyer totals by payment type |
| Partner split | شراکت داروں کا حصہ | Each partner's share of this posting's shared income |

FR-12.7 [A] If a lot's net is negative (deductions more than gross), the summary shows it in red with "بیوپاری کے ذمے" and the save is allowed. The seller's account then shows a debit.

### 12.6 Saving (first posting of a بکری)

FR-12.8 [C] Save writes, as **one action** (all or nothing): the بکری, lots and lines with their snapshots, one journal entry (sale part + cash part), the partner split, stock moves, analytics facts, bill matching, advance matching, and the audit entry.

Checks before save:

1. At least one lot and one sale line.
2. All required fields filled (including پختہ کھاتہ name).
3. Sold qty more than received qty: blocked (FR-12.6).
4. Sold qty less than received qty: warning "5 نگ باقی ہیں". Save is allowed; the rest goes to باقی مال (Section 12.8).
5. بنام = جمع for the whole entry (cannot be bypassed) [C].
6. Partner split total = shared income (cannot be bypassed) [C].
7. Active partner shares total 100% on the posting date (cannot be bypassed) [C].
8. The date is an open date (FR-6.6).

FR-12.9 [C] **Journal lines of a بکری posting.** All amounts come from the stored line and lot values.

Sale part:

| Account | Side | Amount |
|---|---|---|
| Buyer of each line (own account, نقدی منڈی, or پختہ کھاتہ) | بنام | Line buyer total |
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
| Each نقدی line | روکڑ: buyer total | The line's buyer account: buyer total |

So a نقدی line for a named regular buyer still appears on his ledger (bought and paid the same day), and on his buyer slip.

FR-12.10 [A] Auto-save draft every 5 seconds while typing. After a crash or power cut, the app offers to reopen it.
FR-12.11 [A] Cancel (منسوخ): see Section 12.9. Reason required.
FR-12.12 [S] Print: seller bill, buyer slips for this بکری, پختہ کھاتہ slip.
FR-12.13 [S] Move between saved بکری: first, previous, next, last, and find by number, truck, seller, date or item.

### 12.7 Today's بکری list (آج کی بکری)

Columns: بکری no, time, truck, seller(s), items, نگ, خام بکری, صافی بکری, buyer total, status. Filters: date, seller, item, status. `Enter` or click opens it.

### 12.8 Goods not yet sold (باقی مال) [S + A]

The old reports menu has a stock report ("مکمل سٹاک") and an Inventory Reports menu. In a commission business, "stock" means goods received but not yet sold.

FR-12.14 [A] The باقی مال screen lists every lot with balance qty more than 0: date received, بکری no, seller, item, packing, received, sold, written off, taken as fruit, balance, days waiting.
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

FR-12.18 [A] Any one posting of a بکری can be edited while that posting's date is open. Editing a posting in a closed day needs the Owner to reopen the day first (FR-21.2). There is no other path.
FR-12.19 [A] An edit is saved as a reversal of that posting (same date) plus a new posting (same date), in one action.
FR-12.20 [A] In an edit, a line or lot charge the user did not change keeps its stored method, value and amount exactly (FR-11.6). The only exception is a per-lot commission moved from a removed line (FR-11.9). Only changed or new lines look up the rules again (by the posting date). When only a lot's received qty is corrected, its per نگ lot charges keep their stored method and value and their amount is worked out again (C3); a typed amount stays as typed.
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
- اپنا مال is offered only in the lot seller field; every other picker leaves it out, so its account never holds a balance.
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

## 13. Module: پختہ کھاتہ (one-time parties) [C]

Umar: a new or rare customer who comes once and buys. He has no account of his own. His record goes into پختہ کھاتہ. Umar (review): one-time parties are charged the same as everyone else, 6% by default, plus لاگا.

Video evidence [S]: the old sales book report (بکری 20856) shows پختہ کھاتہ as the seller on every lot, with کل خرچہ equal to خام بکری and صافی بکری 0. So one-time sellers were paid their whole net in cash on the spot. We support both sides.

FR-13.1 [C] Two system accounts: پختہ کھاتہ (خریدار) 0030 and پختہ کھاتہ (بیوپاری) 0031 (type number 003, FR-10.2).
FR-13.2 [C] Using پختہ کھاتہ on a line or lot requires the person's name; phone is also required for a credit buyer and for a seller.
FR-13.3 [C] پختہ کھاتہ uses the same commission and لاگا rules as any party (FR-11.7).
FR-13.4 [A] Default payment type is نقدی.
FR-13.5 [A] **Person key.** Inside a پختہ کھاتہ account, each person is identified by name + phone (name compared after Urdu normalisation). Balances, bill matching, aging and dues are kept per person, not for the whole account. Example: شاہد's payment never settles ناصر's bill.
FR-13.6 [A] **پختہ کھاتہ register:** every پختہ کھاتہ line with date, بکری no, side, name, phone, item, نگ, amount, commission, paid or due. Filters: date, name, phone, paid or due.
FR-13.7 [A] Receiving from a one-time buyer: وصولی on پختہ کھاتہ (خریدار), then pick the person from his open lines. Paying a one-time seller: ادائیگی on پختہ کھاتہ (بیوپاری), then pick the person from his open lots.
FR-13.8 [A] **Make regular.** If a one-time person starts coming often, a button creates a proper account and moves his open balance to it with a جنرل entry. Not offered for a person in the dead amount (FR-10.10).
FR-13.9 [A] Analytics and reports can group پختہ کھاتہ by person, so "how much did شاہد buy" still works.
FR-13.10 [S] **One-time sellers are paid on the spot.** On a پختہ کھاتہ (بیوپاری) lot, نقد starts as the lot's صافی بکری, so his balance is 0 after the save. The munshi can change it if part is paid later.

Example (Section 4.5): شاہد buys 10 پیٹی سیب at 1,200. Commission 12,000 × 6 ÷ 100 = 720. لاگا 10 × 8 = 80. He pays 12,800 cash. The register shows one line, paid.

---

## 14. Freight (کرایہ) [S]

The old software's بکری menu has one button, labelled "کرایہ بک", and it opens the بکری بک screen. So the old software has no separate freight register, and we do not build one.

FR-14.1 [S] Freight is typed on each lot, or split from the header total (C17). When the firm pays the driver, it posts in the بکری journal to کرایہ کھاتہ (02010002), the money owed to transporters (FR-12.9).
FR-14.2 [A] A transporter is paid with a normal ادائیگی from کرایہ کھاتہ (Section 15.3). The ledger of کرایہ کھاتہ (R13) shows what is still owed.

---

## 15. Module: وصولی, ادائیگی, پیشگی (in the بکری side menu) [C]

### 15.1 Common voucher fields

| Field | Urdu | Type | Req | Rule |
|---|---|---|---|---|
| Voucher no | واؤچر نمبر | Auto at save | Yes | Own series per type (و-1, ا-1, پ-1) |
| Date | تاریخ | Date | Yes | Today; must be an open date |
| Account | کھاتہ | Picker | Yes | Filtered by type |
| پختہ کھاتہ person | نام | Picker of open persons | If the account is a پختہ کھاتہ | |
| Current balance | موجودہ بقایا | Read-only | | Shown as soon as the account is picked |
| Amount | رقم | Money | Yes | More than 0 |
| Narration | تفصیل | Text + standard narration | No | |
| New balance | نیا بقایا | Read-only | | After this voucher |
| System fields | | Auto | | Created/updated by and at, status, cancel reason |

There is no mode field: all money is روکڑ, the shop's one cash account, in the drawer or in the bank (Umar, 19-10-2026). There is no bank account and no cheque.

FR-15.1 [A] Several vouchers can be entered one after another in a list, with a running total, and saved together (each is its own posting event).
FR-15.2 [A] Print a receipt slip with the amount in words (FR-19.7).
FR-15.3 [A] Edit while its date is open (reversal + new posting); cancel with reason; all effects reversed (Section 6).

### 15.2 وصولی / اگراہی (receipt from buyer) [C]

Umar: the buyer took goods but did not pay. The firm collects later (اگراہی).

Extra field: collector (اگراہی والا) [A].

- Picker shows buyers first, oldest due at the top. After a buyer is picked, his open bills show (FR-7.58).
- Effect: روکڑ بنام, buyer جمع.
- FR-15.4 [A] **Bill matching (oldest first).** A buyer's "bills" are every debit posted to his account (or to his person key in پختہ کھاتہ): sale line totals, opening debits, cash lent (FR-15.11) and جنرل debits. A receipt settles the oldest open bills first. This keeps aging correct.
- FR-15.5 [A] If a buyer pays more than he owes, the extra stays as his credit and settles his next bills automatically. A person in the dead amount cannot pay more than he still owes (FR-10.10).
- FR-15.6 [A] "اگراہی فہرست" button: today's planned collections by collector and city, printable. People in the dead amount are left out (FR-10.10).
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

A list of lines: account, بنام, جمع, narration (and person for پختہ کھاتہ lines). Cannot save unless totals match. Used for corrections, transfers and goods returns. Lines on income or shop expense accounts write partner shares (FR-6.5); lines on buyer accounts take part in bill matching (FR-15.4). [C] A جنرل line on a seller account never adjusts his advances (Umar, 15-10-2026): advances are adjusted only by بکری credits and the seller's وصولی (FR-15.8). A جنرل debit on a seller settles his payables, and a جنرل credit on a seller is a payable posting (FR-7.59). Example: رحیم has 15,000 of advance open and a جنرل correction credits him 1,000. His advance stays 15,000 open, and the 1,000 is payable to him.

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
- From our shop: the fruit comes from a lot with balance. The lot's balance goes down by the qty (C16), and the lot's seller is credited the value (the seller's account, or the پختہ person, or the garden), with no commission [A]. The credit is a payable posting (FR-7.59) and never adjusts an advance. A qty more than the lot's balance is refused (`ERR_QTY_EXCEEDS_LOT`). Trip goods are not offered: they belong to the trip's partners, not to a lot [A].
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

---

## 18. Module: تجزیات (Analytics) [C]

Umar: filter by month or year, and see how much commission we got by item (for example banana), by seller, by buyer.

### 18.1 Filters

| Filter | Urdu | Options |
|---|---|---|
| Period | مدت | آج، یہ ہفتہ، یہ مہینہ، پچھلا مہینہ، یہ سال، پچھلا سال، custom from-to |
| Group by | ترتیب | Day, month (ماہانہ), year (سالانہ) |
| Item | قسم مال | All, or pick one or more |
| Seller | بیوپاری | All, or pick |
| Buyer | گاہک | All, or pick (پختہ کھاتہ persons included) |
| City | شہر | All, or pick |
| اڈا | اڈا | All, or pick |
| Garden | باغ | All, or pick (v1.6) |
| Trip | سفر | All, or pick (v1.6) |
| Goods | مال | All, consigned goods, trip goods, garden goods (v1.6) |
| Partner | شراکت دار | Views F, G, I, J and K only |
| Commission side | جانب | Both, seller only, buyer only |
| Compare | موازنہ | Off, previous period, same period last year |

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
| پختہ کھاتہ: شاہد | 10 | 12,000 | 720 | 80 | 0 | 0 |

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

FR-19.1 [S] Each report first shows its filters: از, تا, and where relevant بیوپاری, زمیندار, گاہک, قسم مال, شہر, اڈا.
FR-19.2 [A] Date shortcuts: آج، کل، یہ ہفتہ، یہ مہینہ، پچھلا مہینہ.
FR-19.3 [A] Every report: on-screen view, print (A4, in the print language of FR-7.36, page numbers, firm header, print time), save as PDF, Excel.
FR-19.4 [S] Open a row (click or `Enter`) to see its source بکری or voucher.
FR-19.5 [C] Totals (میزان) always shown. بنام and جمع totals match where applicable.
FR-19.6 [C] Numbers never flip next to Urdu text (`(16 @ 900)` stays correct), and Urdu text inside an English print (a note typed in Urdu) keeps its own direction.
FR-19.7 [A] Amount in words on bills and statements, in the print language: 19,316 = "انیس ہزار تین سو سولہ روپے" or "Nineteen thousand three hundred sixteen rupees only". Urdu words always use لاکھ, کروڑ and ارب. English words follow the number grouping setting: thousand, million, billion for 1,316,040 style; thousand, lakh, crore for 13,16,040 style.
FR-19.8 [C] **Items in party ledgers.** Umar (11-10-2026): add the item. In R13 and R14, each sale line on a buyer's ledger shows its item, qty, packing and rate, then commission and لاگا, for example "بکری 1: کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160" with 12,880 بنام. Each seller credit shows the lot's item, qty, خام بکری and deductions. Cash lent shows "نقد ادھار" and its reason. Example E2E-33.
FR-19.9 [C] **Each buyer's اگراہی and the shop's total.** R20 has one row per buyer (and per پختہ کھاتہ person) not in the dead amount. Its میزان always equals the dashboard tile گاہکوں کے ذمے. The shop's total اگراہی (tile کل اگراہی) = the R20 میزان + the بیوپاری کے ذمے total of R24 + the R38 میزان (v1.10). Example: after Events 1 to 5, اکرم 2,880 + ارشد 18,684 = 21,564; no seller owes and nobody is dead, so کل اگراہی is also 21,564.
FR-19.10 [C] **Print any day, any party.** Umar (14-10-2026): everything can be printed for one day or a date range, and for one party where that applies, to share with partners or the party. Every report, ledger, register and list screen (آج کی بکری, باقی مال, مارکیٹ سفر and خریداری, باغات, پختہ کھاتہ register, voucher lists, expenses, پھل, partner money, partner capital, ادھار رقم, الگ کھاتے) has the date filter (one day or from and to, with the shortcuts of FR-19.2) and a party filter (buyer, seller, partner, person) where it applies, and the same print, PDF and Excel actions (FR-19.3). Each print shows its filters (dates and party) in its header. Balance lists (R20, R24, R26, R32) take an "as of" date instead of a range; one day's collections are R23 for that day. The partner capital and ادھار رقم list screens print the same way. Examples: one buyer's goods on 10-10-26 (R5, buyer and date); one seller's whole history (R13 from the first date); all sales of one day (R2 and R6 for آج); one day's expenses (R29); one partner's month (R31); the whole day (R34). Example E2E-36.

FR-19.11 [C] **Large lists, page by page.** (FR-19.11 and FR-19.12 were added in v1.6.) Umar (18-10-2026): a list, ledger, register, report or analytics table loads its rows from the backend one page at a time (for example 100 rows) and draws only the rows on screen. Its totals come from the backend over the whole filter, never from the rows on screen. Print, PDF and Excel always hold every row. Speed: NFR-SPD-24. [C] (Umar, 30-10-2026, v1.8.) Under every such table a pager shows rows per page (25, 50 or 100, default 25, remembered per screen), "صفحہ 2 از 14" (Page 2 of 14), the rows shown out of the total, and the buttons First, Previous, Next and Last; `Page Up` and `Page Down` move a page. A new filter starts again at page 1. Example: a buyer's ledger of 10 years opens with its first 25 rows and its closing balance at once; Next shows rows 26 to 50.

FR-19.12 [C] **Party day sheet (R35), a compulsory print.** Umar (18-10-2026): the collector takes a slip for each party to the shops. For a chosen day, for each party:
- Buyers: previous balance (at the start of the day), bought that day (each line: item, qty, rate, amount with commission and لاگا), paid on the spot (نقدی lines), paid that day (وصولی), balance remaining, and an empty column for the collector's note.
- Sellers: previous balance, sold that day (each lot: item, qty, credit), paid on the spot (نقد), paid that day (ادائیگی), balance remaining.
- One slip for one party, or all parties with a balance or an entry that day at once. The amounts the collector brings back are entered with the collection round (FR-7.61).
- [C] (Umar, 20-10-2026) **The client's own layouts.** The client showed his old prints of 18-10-26, and R35 follows them:
  - **One party's slip** (the buyer's day slip): the print header of FR-19.13; بنام (name with tag) and the date; then one row per line of the day: تعداد, قسم مال (packing and item, for example "لونگر کیلا", "جالی مسمی"), ریٹ, رقم (the line's amount with commission and لاگا, C5); the total نگ; then the footer: تازہ بنام (bought that day), سابقہ رقم (previous balance), موقع پر ادا (paid on the spot, only when not 0), وصولی (paid that day, left blank for the collector when nothing is paid yet), ٹوٹل رقم (سابقہ رقم + تازہ بنام − موقع پر ادا − وصولی).
  - **All parties as one table** (the all-buyers balance sheet; sellers in a second table): کوڈ (the short code, FR-7.39), پارٹی (name with tag), سابقہ, بنام (everything debited that day), جمع (everything credited that day), بقایا, and a blank column for the collector's note; sorted by Urdu name; a grand total row. Each row holds: سابقہ + بنام − جمع = بقایا. The buyers' total بقایا equals the میزان of R20 (FR-19.9). A person in the dead amount shows only on a day he has an entry, marked ڈوبی; his row is left out of the grand total, and a second total row adds it (v1.10).

Example (E2E-43): اکرم on 10-10-26: previous 0 + bought 12,880 − paid on the spot 0 − paid that day 10,000 = 2,880 remaining. His slip shows one row "20, پیٹی کیلا, 600, 12,880", total نگ 20, تازہ بنام 12,880, سابقہ رقم 0, وصولی 10,000, ٹوٹل رقم 2,880. In the buyers' table: اکرم 0, 12,880, 10,000, 2,880; ارشد 0, 18,684, 0, 18,684; شاہد 0, 12,800, 12,800, 0; total 0 + 44,364 − 22,800 = 21,564 = R20. The client's own slip of 18-10-26 reads the same way: سابقہ رقم 33,287 + تازہ بنام 25,574 = ٹوٹل رقم 58,861, with وصولی blank.

FR-19.13 [C] **Clear prints everywhere.** (Added in v1.7.) Umar (20-10-2026): R5 and R35 follow the client's own layouts (FR-19.12, R5). Every other print (reports, ledgers, statements, expenses, partners, gardens, trips, fruit, analytics) uses our own clearest layout, the same on every print:
- Header: the firm name, the mandi name, the contact persons with their phones (up to 3) and the trade mark in a box (Section 21), then the print's title, its filters (dates and party, FR-19.10) and the print date and time.
- Body: every column labelled; a party shown with its short code and its name with tag; numbers with equal-width digits, each column of numbers and its total centred under its heading (NFR-USE-19, v1.9); a running balance where the print has one.
- Footer: a totals row under its columns (FR-19.5), amounts in words where FR-19.7 asks for them, and "Page x of y".
Example: R29 for October prints "Maher Adnan and Brothers", the three contacts and the trade mark, then "خرچہ رپورٹ، 01-10-26 تا 31-10-26", the heads with their totals, and the میزان.

### 19.2 Report catalogue

**Sales** (filters as in the old sales report box [S]: از، تا، بیوپاری، زمیندار، قسم مال)

| ID | Report | Urdu | Columns | Source |
|---|---|---|---|---|
| R1 | Sale book print | بکری بک | One بکری in full (seller bill) | [S] |
| R2 | Full sales register | مکمل بکری | بکری no, buyer, city, item, (qty @ rate), amount | [S] |
| R3 | Cash sales | نقدی | R2, نقدی lines only | [S] |
| R4 | Credit sales | ادھاری | بکری no, buyer, (qty @ rate), amount | [S] |
| R5 | Buyer bills | ادھاری تفصیل (Print All) | One row per credit buyer, as in the client's print of 18-10-26 (v1.7): serial, party (name with tag), تفصیل with each line written "(item qty . rate)", for example "(سیب 8 . 2600)", رقم (value), رقم بعد لاگا (value + buyer commission + لاگا, C5). Print All = one slip per buyer. | [S][C] |
| R6 | Seller summary | بکری تفصیل / روزنامچہ بکری | Per lot: بکری no, تعداد, بیوپاری, خام بکری, کل خرچہ, صافی بکری, کرایہ, کمیشن, مزدوری, منشیانہ, نقد, فون | [S] |
| R7 | Item-wise sales | قسم مال وار بکری | Item, نگ, gross, avg rate, seller comm, buyer comm, لاگا, total comm (as view A) | [C] |
| R8 | Truck stand report | اڈا رپورٹ | اڈا, trucks, lots, نگ, gross | [S] |
| R9 | پختہ کھاتہ register | پختہ کھاتہ رپورٹ | Date, side, name, phone, item, نگ, amount, commission, paid / due (per person) | [C] |
| R10 | Stock | مکمل سٹاک / باقی مال | Lot, date, seller, item, received, sold, written off, taken as fruit, balance, days | [S] menu, columns [A] |

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
| R20 | Buyer dues | گاہکوں کے ذمے رپورٹ | Buyer (or پختہ person), city, phone, balance, oldest open bill date, days, aging 0-7, 8-30, 31-90, 91+ days; people in the dead amount left out; میزان = the tile گاہکوں کے ذمے (FR-19.9) | [C] |
| R21 | Collection by city | اگراہی بحساب شہر | R20 grouped by city, with city totals; people in the dead amount left out; total = the tile گاہکوں کے ذمے | [S] |
| R22 | Collection by collector | اگراہی بحساب اگراہی والا | R20 grouped by collector; people in the dead amount left out; total = the tile گاہکوں کے ذمے | [S] second menu item (blurry), meaning [A] |
| R23 | Receipts register | وصولی رجسٹر | Date, voucher, buyer, amount, collector | [A] |
| R24 | Seller payables | بیوپاریوں کو ادائیگی باقی | Seller, open payable (FR-7.59), open advance, ledger balance (open payable − open advance), last بکری date; separate list "بیوپاری کے ذمے" for what sellers owe beyond their open advances (FR-15.9), people in the dead amount left out | [C] |
| R25 | Payments register | ادائیگی رجسٹر | Date, voucher, account, amount | [A] |
| R26 | Advances | پیشگی کھاتہ | Seller, given, returned, adjusted, open, date; people in the dead amount left out | [C][S] |
| R27 | Bill series | بیجک سیریز | Seller bills (بیجک) in number order for a date range: بکری no, date, seller, خام بکری, کل اخراجات, صافی بکری | [S] menu (shown as "بجک سیریز"), columns [A] |

**Business**

| ID | Report | Urdu | Columns |
|---|---|---|---|
| R28 | Income report | آمدن رپورٹ | Each income account by day or month |
| R29 | Expense report | خرچہ رپورٹ | دکان خرچہ by head; ذاتی خرچہ taken and returned by partner |
| R30 | Profit and partner split | منافع و تقسیم | Income − shop expenses = profit; each partner's share |
| R31 | Partner statement | شراکت دار کھاتہ | Every dated movement and the running کل رقم (FR-17.9), Section 17.3 |
| R32 | Money lent | ادھار رقم رپورٹ | Person, staff or not, lent, returned (cash, salary), open, oldest open date, return by, days past it; میزان |
| R33 | Trip profit or loss | تجارتی منافع / نقصان | Per trip and per purchase: date, item, bought (نگ, cost), sold (نگ, sales money after commission, seller commission, total commission = seller + buyer commission + لاگا, v1.8), cost of goods sold, written off, left (نگ, cost), profit or loss; each partner's %, money in and money back; by item, day and month; میزان |
| R34 | Day summary | روزانہ خلاصہ | For one day (or each day of a range): بکری count, خام بکری, commission, cash opening, cash in, cash out, closing; receipts, payments, advances, expenses, partner money, capital, money lent; trips (bought, sold), gardens (paid, money in), fruit taken; partner shares; buyers' dues (گاہکوں کے ذمے), sellers' dues (بیوپاریوں کے ذمے), dead amount (ڈوبی رقم), total dues (کل اگراہی); lots in باقی مال |
| R35 | Party day sheet | پارٹی کی روزانہ پرچی | For one day (FR-19.12): one party's slip in the client's layout, or all parties as one table (code, party, سابقہ, بنام, جمع, بقایا, collector's column) with a grand total |
| R36 | Gardens | باغات رپورٹ | Per garden: installments and costs (shop and pockets), sales money (our shop, other shops), commission on its goods, running balance, rent still due, closed result; each partner's %, money in and money back |
| R37 | Fruit taken | پھل رجسٹر | Date, for whom, item, qty, rate, value, source; by partner, item, day, month or year; میزان |
| R38 | Dead amount | ڈوبی رقم رپورٹ | Person, type (گاہک, بیوپاری, پختہ کھاتہ, ادھار رقم), code, reason, date moved, owed then, recovered since, still owed, last recovery date; filter: still owed or all; میزان = the tile ڈوبی رقم (FR-10.10) |

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

---

## 21. Module: ترتیبات (Settings), day close and audit

| Group | Settings |
|---|---|
| Firm | Firm name (Urdu, English), mandi name, address, phones, logo; up to 3 contact persons for prints, each a name (Urdu and English) and a phone, for example "منشی آصف بھٹی 0300-4058860"; trade mark text (for example "RH"), printed in a box (FR-19.13, v1.7) |
| Codes | Type numbers: seller 001, buyer 002, پختہ کھاتہ 003, item 004, packing 005, staff 006, market trip 007, garden 008 (fixed, FR-10.2) |
| Charges | Global defaults: seller commission 6%, buyer commission 6%, لاگا 8 per نگ on fruit (as an item-group rule). مزدوری, منشیانہ, فون have no default. The commission agent changes any of them (Section 11). |
| Overrides | Allow line-level commission and لاگا override (on / off, default on, FR-11.10) |
| Checks | Credit limit: warn or block. Rate check (FR-7.55) on or off, with its percent (default 30%). |
| Day | Closing time; auto day close on or off |
| Backup | Cloud account details, local backup folder, USB on / off, closing time |
| Printing | Paper size per document (A4, A5, 80 mm), printer, copies. Urdu digits (۰-۹) on Urdu prints: on or off (default off). |
| Language | Language of the backup pack (default اردو); the app language is switched with FR-7.30. Number grouping (1,316,040 or 13,16,040). Default text size. Learned words list (FR-7.26): view and remove. |

Rounding (Section 22) and the split leftover rule (C10) are fixed rules, not settings.

FR-21.1 [A] **Day close (دن بند کریں).** Closes the oldest open day (and can close several days up to a chosen date, in order). It shows the day summary first: بکری count, خام بکری, commission, partner shares, cash in hand, trial balance difference (must be 0), lots with باقی مال. Confirm moves the "closed up to" date forward and starts the cloud backup.
FR-21.2 [A] **Reopen.** Reopening needs a reason. Reopening a date reopens that date and every later date (the "closed up to" date moves back to the day before it). It is written to the audit log. A month that has been month-closed cannot be reopened until its month close is reversed by the Owner.
FR-21.3 [A] **Audit log.** Every create, edit, cancel, login, rule change, share change, opening change, day close, reopen, month close and restore: user, time, screen, record, old value, new value, reason. Read-only. Owner can filter it.

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
| C17 | Header freight split = round down (total × lot qty ÷ all lots' qty); leftover to the lot with the most نگ (tie: lowest lot number) | 1,000 over 30 and 20 نگ → 600 and 400 |
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
| E22 | Same truck twice in one day | Allowed; separate بکری |
| E23 | Item sold by weight | Rate per kg; value = round(weight × rate); per-نگ charges use the نگ count; percent charges use the value |
| E24 | Zero or negative qty or rate | Not allowed |
| E25 | پختہ کھاتہ credit sale without phone | Not allowed |
| E26 | Restore of an older backup | Warning listing what will be lost; safety copy first |
| E27 | Rounding leftover in a split | Goes to the largest share; totals always exact |
| E28 | Lot net negative (deductions more than gross) | Allowed with a red warning; seller ends up owing |
| E29 | A code typed that is already used | Refused; the next free code is suggested (FR-10.3). |
| E30 | Two one-time people in پختہ کھاتہ | Separate balances and matching per person (FR-13.5) |
| E31 | Money typed with decimals ("850.50") | Not allowed: "رقم پورے روپوں میں لکھیں" |
| E32 | Language switched in the middle of an unsaved بکری | Values, selected row, cursor and draft stay; only words, names and direction change (FR-7.33) |
| E33 | English letters typed in a name field and the first suggestion is wrong | User picks another suggestion or edits the word; the pair is learned (FR-7.26); saved names never change |
| E34 | A rare name not in the word list | Spelling rules still give suggestions; for exact spelling the user switches to اردو کی بورڈ with `Alt+U` |
| E35 | Urdu letters in the English name, or English letters in the Urdu name | Not saved; clear message (FR-7.28) |
| E36 | A short code that fits a buyer and a seller, in a field that takes any account | Both listed with their type (FR-7.39) |
| E37 | A code or short code with no account | Message, and `F4` adds an account with that code (FR-7.42) |
| E38 | Two accounts with the same name | The tag tells them apart (اسلم BA and اسلم); search rows and the party card also show father or firm, city, phone and balance (FR-7.40, 10.2) |
| E39 | The same truck entered twice on one day | Warning with a button to open the first بکری (FR-7.50) |
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

---

## 24. Assumptions register (to validate at the demo)

| ID | Assumption | Default | Ask the client |
|---|---|---|---|
| A1 | Seller commission is a percent, with rules by seller and item | 6% | Real rates per seller and item? |
| A2 | لاگا on vegetables | None (fruit only, 8 per نگ) | Is there any لاگا on vegetables? |
| A3 | Resolved in v1.7: a buyer pays commission and لاگا on every line; a seller pays commission only; the rate of each is set per item as rules (11.2). The client's real rates are typed in as rules at setup | | |
| A4 | پختہ کھاتہ on both buyer and seller side | Both | Only for buyers? |
| A5 | مزدوری, when a rule is made, is per نگ received | No default | On received or sold qty? |
| A6 | منشیانہ and فون are typed per lot | No default | Usual amounts, so a rule can fill them? |
| A7 | Buyer rate normally equals seller rate | Same rate | When do they differ? Who keeps the difference? |
| A8 | All income (commission, لاگا, فون, ریٹ فرق) is shared between partners | | Correct? |
| A9 | All shop expenses are split by the same shares | | Correct? |
| A10 | Partner names and shares | الف 40, ب 30, ج 30 | Real names and shares? |
| A11 | Split leftover to the largest share (tie: partner created first) | | OK? |
| A12 | One truck can carry several sellers' lots | Yes | Correct? |
| A13 | تاریخ بل = the date of the seller's bill | Same as تاریخ | What is the old تاریخ بل used for? |
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
| A33 | English screens use English words, with Roman names for mandi words | Glossary English column (Laga, Munshiana, Pukhta Khata, Adda, Nag) | Who will use English? Are these the words they use? |
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
| بکری screen header: بکری نمبر (2 boxes), تاریخ بل, تاریخ, ٹرک نمبر, زمیندار کوڈ + name, اڈا + name, ریفریش | Section 12.2 (refresh replaced by live totals, FR-7.5) |
| Upper grid: بزگر code/name, فون, منشیانہ, نقد, کرایہ سٹور, سٹور, sideways scroll | Section 12.3 (+ hidden inputs) |
| Lower grid: all 11 columns + میزان | Section 12.4 |
| Summary: خام بکری ... صافی بکری | Section 12.5 |
| Toolbar: save, print, first, previous, next, last, new record, delete record, query | FR-12.11 to FR-12.13 |
| روکڑ: وصولی, پیشگی کھاتہ, ادائیگی, جنرل | Sections 15, 16 |
| Reports left: انڈکس, روکڑ, فرد حساب اور کھاتہ, روزنامچہ, ووچر, مکمل سٹاک, پیشگی کھاتہ | R16, R12, R13/R14, R11, R15, R10, R26 |
| Reports right: بکری, بجک سیریز (بیجک), اڈا رپورٹ | R1 to R6, R27, R8 |
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
| پختہ کھاتہ in seller column | Section 13 (both sides) |
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
| بکری header field "تاریخ بل" | "تاریخ مال" (misread) | تاریخ بل, bill date (Section 12.2, A13) |
| The بکری menu has one button, "کرایہ بک", and it opens the بکری بک screen | A separate freight register | Removed (Section 14, A17) |
| Reports panel: "بجک سیریز" | Cheque series | بیجک (bill) series, R27 |
| Sales book: seller commission 6% (276 on 4,600) and 4% (180 on 4,500) on different lots | 6% default | 6% default confirmed by Umar; other rates by rules |
| Sales book: مزدوری 0 on most lots; 25 on 5 نگ (5 each), 15 on 5 نگ (3 each) | 5 per نگ on every lot | No default; typed or set by a rule |
| Credit bills: مولی and ساگ exactly 5% (700 → 735, 1,000 → 1,050); سیب 99 per نگ; کیلا 400 → 423 | Buyer 5% + لاگا 3 | Umar: buyer 6%, لاگا 8 per نگ on fruit; item and group rules for exceptions (A2, A3) |
| پختہ کھاتہ seller lots: کل خرچہ equals خام بکری, so صافی بکری is 0 | One-time sellers paid later | نقد starts as the net, paid on the spot (FR-13.10) |
| Account form: "Allowed" is ticked by default; opening date defaults to today | Credit allowed off by default | Ticked by default (Section 10.2) |
| Many کیلا lines have no rate, and amounts like 4 نگ = 2,950 | Not covered | A42 |
| Day book 1,316,042 against 1,316,040; one lot with صافی −1 | Already noted | Prevented by FR-6.2 and C15 |

---

## 26. MVP scope

| Priority | Items |
|---|---|
| **Must (demo, all connected)** | Login; dashboard with partner strip; انڈکس with seller and buyer codes by type (0010, 0020) and opening balances; all masters; commission rules; بکری بک with side menu, later sales, edit and cancel; پختہ کھاتہ; باقی مال; وصولی/اگراہی; ادائیگی; پیشگی; one خرچہ screen for دکان خرچہ and ذاتی خرچہ with the effect shown before save (16.2, FR-16.8); related entries on one screen and the right-type notes (FR-7.67, FR-7.68); جنرل; partners with live split; partner capital in and out (17.4); partner total کل رقم and its dated record (FR-17.9); الگ کھاتہ (FR-16.6, FR-16.7); market trips with their purchases, costs, sales, write-offs and result (12.10); gardens with installments, costs, sales and closing (12.11); fruit taken (16.7); one cash account with no bank (v1.6); dated statement lines (FR-17.10); partner money taken and returned (FR-16.2, FR-16.3); ادھار رقم (FR-16.4, FR-16.5); advance tick and advance return (FR-12.23, FR-15.12); cash lent to buyers (FR-15.11); spelling check (FR-7.66); day close and reopen; every analytics view A to K, printable by day, month or year (FR-18.4, FR-18.8); the dues tiles and the dead amount (9.1, FR-10.10); reports R1, R2, R4, R5, R6, R7, R9, R10, R11, R12, R13, R14, R15, R18, R20, R21, R22, R23, R24, R25, R26, R27, R29, R30, R31, R32, R33, R34, R35, R36, R37, R38; the party day sheet (FR-19.12) and the collection round (FR-7.61); large lists page by page (FR-19.11); print any day and any party (FR-19.10); items in ledgers and the shop's total اگراہی (FR-19.8, FR-19.9); Excel export (needed for the backup pack); arrow-key navigation; Urdu keyboard; language switch with the whole app in English (Section 7.5); names in two scripts with English-to-Urdu typing and learning (Section 7.4); the tag after a name (10.2); prints in the client's layouts and a clear layout everywhere else, with the firm's contacts and trade mark (FR-19.12, FR-19.13); party card, short codes, smart search, recent first, unknown code add (FR-7.38 to FR-7.42); how was this worked out (FR-7.43); attention list and آج کے کام (FR-7.44); helpful empty screens (FR-7.46); truck, seller and buyer memory, same truck warning, remaining qty, rate hint, copy from above, save-print-next (FR-7.49 to FR-7.54, FR-7.56, FR-7.57); receipt, payment and advance helpers (FR-7.58 to FR-7.60); backup and restore; printing in both languages; audit log (written) |
| **Should (right after demo)** | Saved analytics views; reports R3, R8, R16, R17, R19, R28; month close; audit log viewer; first-run guide; undo after save (FR-7.47); simple sums (FR-7.48); rate check (FR-7.55); repeat last month's expenses (FR-7.62); duplicate check while typing (FR-7.63); remember last screen (FR-7.45); party report buttons (FR-7.65) |
| **Could (Phase 2)** | Import of 2018 to 2026 data from the old software; crate (پیٹی) in and out tracking; goods return screen; interest on old dues; daily rate board |
| **Won't (for now)** | Multi-computer, tax filing |

---

## 27. Demo script and end-to-end acceptance tests

### 27.1 Demo script (about 45 minutes)

1. Log in. Show the dashboard and partner strip.
2. Show partners (الف 40, ب 30, ج 30) and commission rules. Use "test a rule".
3. Add a new seller and a new buyer using only the arrow keys and `Enter`: type the name in English letters, see it become Urdu with the English name filled. Show the codes.
4. Enter بکری 1 (Section 4.4) with short codes (`0`, `1`). Show the party card, remaining qty, rate hint, live summary and live partner split, and ⓘ on a commission. Halfway, switch to English with `Alt+L`, carry on, switch back. Save with `Ctrl+Enter`, which prints the seller bill and buyer slips.
5. Enter بکری 2 with پختہ کھاتہ (Section 4.5). Show the seller memory and the same truck warning.
6. Record وصولی (open bills, پورا بقایا) and ادائیگی on the voucher screen. Then, on the one خرچہ screen, the electricity bill as دکان خرچہ (show the split before save) and الف's 2,000 as ذاتی خرچہ (Sections 4.6 to 4.8).
6a. Show the attention bell and آج کے کام. Print a buyer slip in Urdu and in English.
6b. Show a market trip and a garden (E2E-41, E2E-40), a partner's statement with their dated lines, and الف's پھل. Print the party day sheet (R35) for all buyers.
7. Dashboard: every tile matches Section 4.9.
8. Analytics: views A, B, C, F match Section 18.4.
9. Partner overview matches Section 4.9.
10. روزنامچہ and trial balance: both balance.
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
| E2E-5 | بکری 2 (پختہ کھاتہ) | Buyer pays 12,800; seller صافی 10,160; whole entry 25,600 = 25,600; shared income 1,540; split 616 / 462 / 462; register shows شاہد, paid |
| E2E-6 | اکرم after وصولی | 2,880 بنام; aging 0 days |
| E2E-7 | رحیم after ادائیگی | Balance 0; پیشگی report: given 15,000, adjusted 15,000, open 0 |
| E2E-8 | Cash in hand | 58,484 |
| E2E-9 | Partner shares for the day | Income 2,196 / 1,646 / 1,646; expense −1,200 / −900 / −900; profit 996 / 746 / 746; الف drawings 2,000 |
| E2E-10 | Trial balance | Table below; both sides 85,048 |
| E2E-11 | Analytics | View A as in Section 18.4 (including avg rate 588 / 1,200 and 72.10% / 27.90%); view F as in Section 18.4 |
| E2E-12 | Stock | باقی مال empty |
| E2E-13 | Edit بکری 1: اکرم's rate 600 → 610 | His line 20 × 610 = 12,200; buyer comm 12,200 × 6 ÷ 100 = 732; he pays 12,200 + 732 + 160 = 13,092; خام بکری 29,600; seller comm 732 + 1,044 = 1,776; کل اخراجات 10,096; صافی 19,504; shared income 1,776 + 1,776 + 400 + 20 = 3,972; split 1,590 / 1,191 / 1,191; audit log has the old version |
| E2E-14 | Edit back to 600 | Every number returns exactly to E2E-1 to E2E-4 |
| E2E-15 | Cancel بکری 2 | پختہ کھاتہ register shows cancelled; analytics excludes سیب; split reversed (616 / 462 / 462 removed); cash down by 12,800; trial balance still balances |
| E2E-16 | Separate run: Event 1 with only 45 of 50 sold (اکرم 20 at 600, ارشد 25 at 580) | خام 26,500; seller comm 720 + 870 = 1,590; کل اخراجات 3,000 + 1,590 + 250 + 50 + 5,000 + 0 + 20 = 9,910; صافی 16,590; buyer comm 720 + 870 = 1,590; لاگا 45 × 8 = 360; shared income 1,590 + 1,590 + 360 + 20 = 3,560; split 1,424 / 1,068 / 1,068; باقی مال 5 نگ کیلا |
| E2E-16b | Continue E2E-16: on 11-10-26 a later sale of the 5 نگ to ارشد at 560 | Value 2,800; seller comm 168; buyer comm 168; لاگا 5 × 8 = 40; ارشد pays 3,008; seller credit 2,800 − 168 = 2,632; no lot charges again; shared income 168 + 168 + 40 = 376; split 152 / 112 / 112; lot totals خام 29,300, کل اخراجات 10,078, صافی 19,222; باقی مال empty |
| E2E-13b | Separate run: after Event 1, the Owner changes the default buyer commission to 7%, then edits اکرم's rate to 610 | اکرم's changed line uses 7%: 12,200 × 7 ÷ 100 = 854, he pays 12,200 + 854 + 160 = 13,214; ارشد's unchanged line keeps 1,044; shared income 1,776 + 1,898 + 400 + 20 = 4,094; split 1,638 / 1,228 / 1,228 |
| E2E-17 | Close the app while typing a بکری; reopen | Draft offered and restored |
| E2E-18 | Backup, then restore | All balances identical after restore |
| E2E-19 | Opening set-up only (Section 4.3) | Trial balance 65,000 = 65,000; ابتدائی فرق 0; advances open 15,000; cash 50,000 |
| E2E-20 | A shop expense of 3,384 | Shares −1,354 / −1,015 / −1,015 (sum −3,384) |
| E2E-21 | Two one-time credit buyers in پختہ کھاتہ: شاہد 1,000 and ناصر 2,000; receipt for شاہد 1,000 | شاہد 0, ناصر 2,000 still due |
| E2E-22 | Start بکری 1, enter اکرم's line, switch to English (`Alt+L`), enter ارشد's line, save, print the seller bill in English, switch back | After the switch: line reads "Akram Toll Tax", "Banana", "Box", 20, 600, 12,000; layout left-to-right; nothing typed is lost. Saved numbers identical to E2E-1 to E2E-4. English bill: Gross Sale 29,400, Net Sale 19,316, "Nineteen thousand three hundred sixteen rupees only". Back in Urdu, the same بکری shows Urdu names. |
| E2E-23 | New buyer: type `tariq mehmood` in the Urdu name field, accept, save | Urdu name طارق محمود; English name "Tariq Mehmood" filled by itself; code 0022. Typing "tarq" in a buyer picker finds him in both languages. |
| E2E-24 | After Events 1 to 5: new وصولی, buyer field `0` | 0020 اکرم ٹول ٹیکس; card shows 2,880 بنام, oldest open bill 10-10-26 (بکری 1), 0 days; "پورا بقایا" fills 2,880. In a جنرل line, `0` lists both 0020 اکرم ٹول ٹیکس and 0010 رحیم with their types. |
| E2E-25 | After Events 1 and 2: new بکری, truck `LES-1234`, lot seller `0` | Same truck warning names بکری 1; continuing fills سوات اڈا and زمیندار رحیم; the lot fills کیلا and پیٹی. |
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
| E2E-36 | After Events 1 to 5: R34 for 10-10-26, and R5 filtered to اکرم and 10-10-26 | R34: 2 بکری; خام 41,400; commission 5,448; cash opening 50,000, in 12,800 + 10,000 = 22,800, out 5,000 + 4,316 + 3,000 + 2,000 = 14,316, closing 58,484; buyers' dues 21,564; sellers' dues 0; dead amount 0; total dues 21,564. R5 shows only اکرم's row: (کیلا 20 . 600), 12,000, 12,880. Both headers show the date and, for R5, the buyer. |
| E2E-37 | After Events 1 to 5 and E2E-35: partner totals and R31 for ب | کل رقم الف −1,004, ب 50,746, ج 746; Σ 50,488 + ابتدائی سرمایہ 65,000 = 115,488 = equity in the trial balance (65,000 + 50,000 + 5,488 − 3,000 − 2,000). R31 for ب, in date order: 10-10-26 بکری 1 share 1,184 (running 1,184); بکری 2 share 462 (1,646); دکان خرچہ بجلی −900 (746); 11-10-26 سرمایہ جمع 50,000 (50,746). |
| E2E-38 | Separate run: opening position. A الگ کھاتہ خاندانی کھاتہ (Family Khata). 10-10-26: pays out 10,000 cash (reason خاندان کا خرچ). 11-10-26: gets 4,000 back in cash; then a بکری lot with seller خاندانی کھاتہ sells 10 پیٹی کیلا to ارشد at 900, ادھاری | After paying out: خاندانی کھاتہ −10,000, cash 40,000. After 4,000 back: −6,000, cash 44,000. The sale: value 9,000; no seller-side charges; buyer comm 540; لاگا 80; ارشد pays 9,620; entry 9,620 = 9,000 (خاندانی کھاتہ) + 540 + 80; خاندانی کھاتہ +3,000. Shared income 620, split 248 / 186 / 186; nothing from its 9,000 goes to any partner. Trial balance balances. |
| E2E-39 | Event 5 on the one خرچہ screen (`Alt+K`). First entry: بجلی 3,000 as دکان خرچہ. Second entry: 2,000 typed as دکان خرچہ by mistake, then the type changed to ذاتی خرچہ before saving, partner الف, reason گھر کا خرچ | First entry, before save: الف 1,200, ب 900, ج 900. Second entry: changing the type keeps 10-10-26 and 2,000, clears the expense head and asks for partner and reason; then it shows الف's کل رقم 996 → −1,004 and ب, ج not affected. After both saves: cash 58,484 (E2E-8) and the partner numbers of E2E-9. |
| E2E-40 | Separate run: opening position. A garden شاہ پور باغ (Shahpur Garden): الف 60% from the shop's cash, ب 40% from his own pocket, ج not in it; rent 50,000. 10-10-26: first installment 30,000. 11-10-26: installment 20,000 and labour 5,000. 12-10-26: a بکری lot with seller شاہ پور باغ sells 100 پیٹی کیلا to ارشد at 600, ادھاری, no lot charges; 20 پیٹی sold in another shop bring 10,000 cash. 13-10-26: the garden is closed | Parts 18,000 / 12,000, then 12,000 / 8,000 and 3,000 / 2,000: الف paid 33,000 from the shop (his total −33,000; cash 17,000), ب paid 22,000 himself (his total unchanged). The sale: value 60,000; seller comm 3,600; buyer comm 3,600; لاگا 800; ارشد pays 64,400; the garden's account جمع 56,400; commission 8,000 split by the firm's shares 3,200 / 2,400 / 2,400. Money in 56,400 + 10,000 = 66,400; cash 27,000; running balance 66,400 − 55,000 = 11,400. Closing: 66,400 → 39,840 / 26,560; the garden's account 0. الف −33,000 + 39,840 = +6,840 (60% of 11,400); ب +26,560 (22,000 back + 4,560). Partner totals 10,040 / 28,960 / 2,400; Σ 41,400 + 65,000 = 106,400 = equity; trial balance 106,400 = 106,400. |
| E2E-40b | Separate run: as E2E-40, but with no بکری sale; only the other shop's 10,000 comes in; then the garden is closed | Loss 10,000 − 55,000 = −45,000. Closing: 10,000 → 6,000 / 4,000. الف −33,000 + 6,000 = −27,000 (60% of the loss); ب +4,000 (his 22,000 back − 18,000, 40% of the loss); cash 27,000. |
| E2E-41 | Separate run: opening position. A market trip: الف 50% from the shop's cash, ب 50% from his own pocket, ج not in it. It buys 80 پیٹی مالٹا at 1,200 and pays 4,000 for the truck (a trip cost). A بکری lot with seller اپنا مال sells all 80 to ارشد at 1,400, ادھاری, no lot charges | Purchase 96,000: parts 48,000 / 48,000; truck 2,000 / 2,000. الف paid 50,000 from the shop (cash 0); ب paid 50,000 himself. Cost 100,000 (1,250 per نگ). Sale: value 112,000; seller comm 6,720; buyer comm 6,720; لاگا 640; ارشد pays 119,360; commission 14,080 split by the firm's shares 5,632 / 4,224 / 4,224; the trip's 105,280 → 52,640 / 52,640; trip result 5,280; the trip closes. الف −50,000 + 52,640 = +2,640; ب +52,640 (50,000 back + 2,640). Partner totals 8,272 / 56,864 / 4,224; trial balance 134,360 = 134,360. |
| E2E-42 | Separate run: after E2E-16 (5 کیلا left in رحیم's lot). 10-10-26: الف takes 2 پیٹی کیلا home from that lot at 580; then fruit for the shop's guests: 3 پیٹی سیب at 1,000 from another commission agent's shop (حاجی نذیر, seller 0012) | Home fruit 2 × 580 = 1,160: الف's total −1,160; رحیم credited 1,160 with no commission, his payable 1,590 + 1,160 = 2,750 and ledger 2,750 جمع; lot balance 3. Guest fruit 3,000 split equally 1,000 / 1,000 / 1,000; حاجی نذیر payable 3,000. Cash unchanged at 45,000. Partner totals 1,424 − 2,160 = −736, 1,068 − 1,000 = 68, 68; Σ −600 + 65,000 = 64,400 = equity; trial balance 77,610 = 77,610. |
| E2E-43 | After Events 1 to 5: R35 party day sheet for 10-10-26 | Buyers (previous, bought, paid on the spot, paid that day, remaining): اکرم 0, 12,880 (کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160), 0, 10,000, 2,880; ارشد 0, 18,684, 0, 0, 18,684; شاہد (پختہ کھاتہ) 0, 12,800, 12,800, 0, 0. All buyers remaining 21,564 = R20 میزان. Sellers (previous, sold, نقد, paid that day, remaining): رحیم 15,000 بنام, 24,316, 5,000, 4,316, 0; کریم 0, 10,160, 0, 0, 10,160 جمع. |
| E2E-44 | After Events 1 to 5: on 11-10-26 ارشد is moved to ڈوبی رقم (reason فوت ہو گئے); on 12-10-26 a وصولی of 4,000 cash on him; then a new بکری line for him | After the move: گاہکوں کے ذمے 2,880 (R20 shows only اکرم); بیوپاریوں کے ذمے 0; ڈوبی رقم 18,684, 1 person (R38); کل اگراہی 21,564; his account inactive; trial balance still 85,048 = 85,048; partner totals unchanged. After the وصولی: ڈوبی رقم 14,684; کل اگراہی 17,564; cash 62,484; R38 shows owed then 18,684, recovered 4,000, still 14,684. The بکری line is refused (ERR_ACCOUNT_INACTIVE). |

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
| پختہ کھاتہ (خریدار) | | 0 |
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

