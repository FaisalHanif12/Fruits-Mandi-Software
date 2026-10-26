# How to test every screen: put the values, see the result

Written by `tech-lead`, 26-10-2026, for Umar. It follows the menu from انڈکس to ترتیبات, then the dashboard. Every number comes from the PRD (Sections 4, 18.4 and 27.2). If a screen shows a different number, the app is wrong. Write down the screen, what you typed and what you saw, and send it to `tech-lead`.

Related files: `docs/demo/demo-checklist.md` is the 45-minute demo. This guide is the full test of every screen. `docs/demo/urdu-review.md` is the word list for the Urdu reviewer.

Which app: the app that is running now (the D5 folder) has every screen in this guide. The main project folder gets analytics, reports, backup, settings, day close and month close when the merge that follows your "continue" is done. Until then, run this guide in the D5 folder.

## 0. How each part is written

Every screen has the same blocks, in the same order.

| Block | What it tells you |
|---|---|
| **Open** | The menu path and the key that opens it |
| **Type** | The exact values to enter |
| **You must see** | The numbers and messages that must appear |
| **Try to break it** | Wrong entries. Each must be refused with a clear message. Do not save them. |
| **Not the same as** | The look-alike screens, so you do not mix them up |

Three tags on a test step:

- **[BOOKS]** The step saves entries and changes the books. Do it once, in the order of Section 2.
- **[LOOK]** The step only looks. Repeat it as often as you like.
- **[WINDOWS]** It cannot be proved on the Mac. It is in the list in Section 15.

Words: "short code" means the serial alone (`1` for 03040001). "Nag" is نگ. "Lot" is لاٹ. "Partner A, B, C" are الف, ب, ج.

## 1. Before you start

### 1.1 Start the app and sign in

1. The app runs from `/Users/faisalhanif/MyProfile/My Projects /Fruit Mandi Parallel/d5` with `pnpm dev`. It is running now. Close and start it again the same way after a reset.
2. Sign in: user `owner`, password `mandi-dev-1`.
3. macOS asks about the Keychain item `audit-chain-key`. Type the Mac password and choose **Always Allow**. Without this the sign-in says "something went wrong".

### 1.2 Choose your data (do this once, then follow Section 2)

| Option | Use it when | What you get |
|---|---|---|
| **A. What is there** | You only want to look around | Whatever the app holds now. Check first: انڈکس → Accounts shows رحیم, کریم, اکرم, ارشد, and روکڑ shows 50,000. If someone has entered other test entries, use option B. |
| **B. The worked-day start (recommended)** | You want the PRD numbers exactly | The opening position of PRD 4.3 and the masters of the worked day. Steps below. |
| **C. An empty app** | You want to test the first-run wizard and the Index by hand | Nothing in it. You type the masters and the opening position yourself (Section 4 and PRD 4.3). It takes about 30 minutes more. |

**Option B, step by step.** These commands were not run by the lead on your Mac. They keep your old data, so nothing is lost. If the app says the data is damaged, put the old folder back and tell `tech-lead`.

```bash
# 1. Quit the app: close its window, then press Ctrl+C in the terminal that runs pnpm dev.
cd "/Users/faisalhanif/MyProfile/My Projects /Fruit Mandi Parallel/d5"
APP="$HOME/Library/Application Support/pk.mandisoftware.app"

# 2. Keep the old data under another name.
mv "$APP" "$APP.old-$(date +%Y%m%d-%H%M)"

# 3. Make the worked-day opening position. The first run builds the code and takes a few minutes.
#    Answer the Keychain window with Always Allow.
pnpm seed -- --reset

# 4. Put it where the app reads it.
mkdir -p "$APP/data"
cp dev-data/mandi.db "$APP/data/mandi.db"

# 5. Start the app.
pnpm dev
```

After step 5 the app holds: روکڑ 50,000; رحیم with an open advance of 15,000; ابتدائی سرمایہ 65,000; trial balance 65,000 = 65,000. Partners الف 40%, ب 30%, ج 30%. Sellers رحیم 030110001 and کریم 030110002. Buyers اکرم ٹول ٹیکس 03040001 and ارشد ملتانی 03040002. Items کیلا (Banana), سیب (Apple) and مولی (Radish). Packing پیٹی (Box). Addas سوات اڈا and لاہور اڈا. Expense heads بجلی and تنخواہ. Days up to 09-10-2026 are closed. The default rules are in: seller 6%, buyer 6%, لاگا 8 per نگ on fruit.

Some tests need a **fresh start** (Run 3 in Section 2). Do the five steps again for each of them.

### 1.3 Which date to type

The PRD calls the first day 10-10-26 and the second day 11-10-26. The numbers do not depend on the dates. Leave the date field as it is (today, the date in the bottom bar) for the worked day. Then the dashboard ("today") agrees with the PRD. Where a test needs a later day, use today again. Where a test needs an old date (a return-by date that is already past), the step says which date to type.

Date field tricks: `+` is the next day, `-` the day before. Typing `10` fills the day of the current month. A date on or before the closed-up-to date, or after today, is refused.

### 1.4 The keys you will use

| Key | Does |
|---|---|
| `Enter` | Next editable cell or field. On the last cell of a line, starts a new line. |
| Arrows | Move the way you see (in Urdu `←` is the next field) |
| `F10` or `Ctrl+S` | Save |
| `Ctrl+Enter` | Save, print the ticked documents and start a new entry |
| `F2` new, `F3` find, `F4` add an account from a picker, `F9` print | |
| `Alt+B` new بکری, `Alt+V` وصولی, `Alt+A` ادائیگی, `Alt+P` پیشگی, `Alt+K` خرچہ | |
| `Alt+L` | Language: اردو or English |
| `Alt+U` | Typing mode in a text field: رومن سے اردو, اردو کی بورڈ, انگریزی |
| `Ctrl+K` | Global search |
| `Ctrl+I` | "How was this worked out?" on a calculated amount (same as clicking ⓘ) |
| `Ctrl+D` | Copy the cell above (sale lines) |
| `Alt+Z` | Undo a save, within 10 seconds |
| `Ctrl +`, `Ctrl −`, `Ctrl 0` | Text size |
| `Esc` | Closes a list or dialog, then cancels typing, then leaves the screen |

### 1.5 How to read the screen

- **Party card.** Type a code in any account field and press `Enter`. The name, city, phone, balance, last entry date and warnings appear next to the field. A buyer also shows his credit limit and oldest unpaid bill. A seller shows his open advance and payable.
- **ⓘ.** Every calculated amount has it. It shows the working with real numbers, for example "12,000 × 6 ÷ 100 = 720".
- **Colours always come with a word or an icon.** Green is money in or balanced. Red is due or error. Amber is a warning.
- **Messages** appear next to the field, in the app language, with what to do. No error codes show.
- Money is **whole rupees**. `850.50` is refused. In quantity and money fields `20+25` gives 45.

### 1.6 The language check (do it on every screen)

Press `Alt+L`. The whole screen must change at once, with nothing typed lost.

| Must change | Must NOT change |
|---|---|
| Labels, buttons, hints, messages, menu names | Every amount and quantity |
| Every name (Urdu name or English name) | Codes, short codes, serials |
| Direction: right-to-left in Urdu, left-to-right in English, main menu on the right or left | Dates (always DD-MM-YYYY) |
| Month names, amount in words | Phone numbers, truck numbers, tags, مارکہ text, notes and reasons you typed |

Digits are English digits (0-9) in both languages, on screen and on prints. Urdu digits (۰-۹) must not appear anywhere. A number inside Urdu text must keep its order, for example "(کیلا 20 . 600)".

### 1.7 If something is wrong

Write: the screen name, the language, what you typed, what you expected, what you saw, and the reference number if the message shows one. Logs are in `~/Library/Application Support/pk.mandisoftware.app/logs/`.

## 2. The test plan, in order

Follow the runs in this order. Each row points to the Section that has the detail. Run 1 and Run 2 use the same data. Every row of Run 3 starts from a fresh start (Section 1.2, option B).

| Step | What | Section | Changes books |
|---|---|---|---|
| **Run 1: the worked day** | | | |
| 1.0 | Sign in, look at the dashboard (cash 50,000), the chart, the advance screen. Show the cash-below-zero box and answer No. | 14.1, 4.2, 5.3 | No |
| 1.1 | Index: accounts, masters, rules dated in the future | 4 | Masters only |
| 1.2 | Event 1: بکری 1 (with the language switch and hints) | 5.1 | Yes |
| 1.3 | Event 2: بکری 2 with a one-time buyer | 5.2 | Yes |
| 1.4 | Look: pukhta register, today's list, collection list, stock | 5.4 to 5.7 | No |
| 1.5 | Event 3: وصولی 10,000. Event 4: ادائیگی 4,316 | 5.3 | Yes |
| 1.6 | Event 5: دکان خرچہ 3,000 and ذاتی خرچہ 2,000 | 6.2 | Yes |
| 1.7 | **Checkpoint 1:** dashboard, partners, analytics, reports, trial balance | 9.1, 10, 11, 14 | No |
| 1.8 | Edit بکری 1 and edit back | 5.1 | Nets to zero |
| 1.9 | Draft recovery, undo, backup and test restore | 5.8, 12 | Nets to zero |
| **Run 2: more tests on the same data, in this order** | | | |
| 2.1 | Capital: ب invests 50,000. Partner statement. | 9.3, 9.4 | Yes |
| 2.2 | الف returns 2,000. Cash lent to a buyer: 3,000 to اکرم | 6.2, 5.3 | Yes |
| 2.3 | Money lent: اسلم (with the overdue bell) | 6.4 | Yes |
| 2.4 | Cancel بکری 2 | 5.9 | Yes |
| 2.5 | Close the day, try to edit, reopen with a reason. Settings checks. | 13 | No |
| 2.6 | Restore a backup | 12 | Replaces all |
| **Run 3: fresh start each** | | | |
| 3a | Part-sold بکری, later sale, write-off, fruit taken (E2E-16, 16b, 42) | 5.10, 6.3 | Yes |
| 3b | Garden (E2E-40, 40b) | 7 | Yes |
| 3c | Market trip (E2E-34, 34b, 41) | 8 | Yes |
| 3d | Separate khata (E2E-38) | 6.5 | Yes |
| 3e | Advance tick, advance return, journal credit on a seller (E2E-28, 31) | 5.3, 6.6 | Yes |
| 3f | 7% default and edit, two one-time buyers, expense 3,384, opening balances (E2E-13b, 21, 20) | 4.4, 5.1, 5.4, 6.2 | Yes |

## 3. The differences at a glance

Keep this page open while you test. Most mistakes come from mixing two look-alike screens.

### 3.1 Money in and out on the voucher screen (one screen, three types)

| | وصولی (Receipt) | ادائیگی (Payment) | پیشگی (Advance) |
|---|---|---|---|
| Direction | Money **in** | Money **out** | Money **out** |
| Usual party | A buyer | A seller | A seller, **before** his goods are sold |
| Books | روکڑ بنام, party جمع | Party بنام, روکڑ جمع | Seller بنام, روکڑ جمع |
| Helper shown | Buyer's open bills, oldest first, buttons پورا بقایا and پرانا بل | Seller's payable postings, button پورا بقایا | His open advances and the last advance |
| Other use | From a **seller**: he returns advance money (E2E-31) | To a **buyer**: cash lent, reason required, becomes his bill (E2E-32) | Never a payment of what is owed |
| Number series | و-1 | ا-1 | پ-1 |

### 3.2 The entries on the خرچہ and روکڑ screens

| Entry | Who it touches | Cash | Split between partners | Partner کل رقم |
|---|---|---|---|---|
| دکان خرچہ (shop expense) | The shop | Out | **Yes, by share** | Every partner, by his share |
| ذاتی خرچہ (personal drawing) | One partner | Out | **No** | That partner only, goes down |
| ذاتی خرچہ واپسی (return) | One partner | In | No | That partner only, goes up |
| پھل, for one partner | One partner | None | No | That partner only |
| پھل, for guests | The shop's guests | None | **Equal parts**, not by share | Every partner, equal |
| ادھار رقم (money lent) | A person (staff or other) | Out | No | None |
| الگ کھاتہ (separate khata) | A family or other khata | Out or in | No | None. Never income or expense. |
| سرمایہ (capital) | One partner | In or out | No | That partner, goes up or down |

### 3.3 Garden against trip

| | باغ (Garden) | مارکیٹ سفر (Trip) |
|---|---|---|
| What it is | A garden rented for a season, with rent in installments | A day trip to a bigger market to buy goods |
| Goods come as | A lot with the **garden** as seller | A **purchase** (خریداری); sold as a lot with seller **اپنا مال** |
| Own account | Yes, under 0306; holds the sales money until closing | No |
| Ends when | The Owner closes it (confirm box) | It closes **by itself** when nothing is left |
| Partners' money | Parts of installments and costs | Parts of purchases and trip costs |
| Sales money goes to | The garden's account, split at closing | The trip's partners at once, at the sale |

In both: the partners in it and their % total exactly 100. A partner outside it has 0% but still gets his normal share of the shop's commission on its goods. Each partner's money comes from the shop (دکان سے, comes off cash and his total) or his own pocket (اپنی جیب سے, posts nothing).

### 3.4 Who is the seller of a lot

| Seller | When | Special |
|---|---|---|
| A seller account | Normal | Commission, lot charges, advance tick |
| پختہ کھاتہ (بیوپاری) | One-time seller | Name and phone required; نقد starts as his net (he is paid on the spot) |
| اپنا مال | Selling a trip's goods | Takes goods from one purchase; نقد and the advance tick are hidden; the lot must be fully sold |
| A garden | Selling a garden's goods | نقد and the advance tick are hidden; the credit goes to the garden's account |
| A الگ کھاتہ | Selling the family's goods | Seller-side charges are 0; the credit goes to the khata |

### 3.5 Who is the buyer of a line

| Buyer | Payment type | Notes |
|---|---|---|
| A named buyer (credit allowed) | ادھاری | Appears on his ledger and in R20 |
| A named buyer, credit not allowed | نقدی | Still on his ledger and slip (bought and paid the same day) |
| نقدی منڈی (03040000) | نقدی always | No name, no balance |
| پختہ کھاتہ (خریدار) 03049999 | نقدی by default | Name required; phone also required if ادھاری; balances kept **per person** |

### 3.6 Edit, cancel, undo, write off, reopen

| Action | What happens | Day must be |
|---|---|---|
| **Edit** | Old posting reversed, new posting made. Unchanged lines keep their stored values. Audit keeps the old version. | Open |
| **Cancel** | Every posting reversed, بکری marked منسوخ. Reason required. Receipts become credit and are matched again. | Open |
| **Undo** (`Alt+Z`) | Cancels what you just saved and opens a new entry with the same values. Only for 10 seconds. | Open |
| **Write off** | Takes the leftover qty off stock with a reason. No money posts. | Open |
| **Reopen a day** | Needs a reason. Reopens that day **and every later day**. Written to the audit log. | Closed |

### 3.7 Things that take goods out of باقی مال

| | Later sale | Write off | Fruit taken (پھل) |
|---|---|---|---|
| Money | A new posting with the normal commission | None | Seller is credited the value, **no commission** |
| Lot charges again? | No (first posting only) | No | No |
| Seller's advance | Adjusted if the tick is on | Not touched | **Never** adjusted |

### 3.8 Prints of the same buyer: which one is which

| Print | What it is | Lines | Who |
|---|---|---|---|
| Buyer slip (from the بکری, F9 or `Ctrl+Enter`) | One buyer's slip of this بکری. The client's layout. | Item, qty, rate, amount with commission and لاگا | A named buyer, including his cash lines |
| **R5 ادھاری تفصیل** | One row per buyer. Lines written "(item qty . rate)". | Value, and value after commission and لاگا | **Credit buyers only** |
| **R35 party day sheet** | The collector's slip for a day: previous balance, bought, paid on the spot, paid that day, balance | One buyer's slip, or all parties in one table | Buyers and sellers |
| R13 ledger (کھاتہ) | Every entry of one party with a running balance | Each sale line with item, qty, rate, commission, لاگا | One party, any dates |
| R14 statement (فرد حساب) | Opening, bought or sold, paid, balance. Option "balance only". | Short | One party |
| R2 full register | All sales of a period | One row per line | All |

### 3.9 Other pairs

| Pair | Difference |
|---|---|
| **Gross and net** | خام بکری is the sum of seller amounts. صافی بکری is gross minus all deductions of the lot. |
| **Shared income and payables** | Commission, لاگا, فون and ریٹ فرق are income, **shared** by the partners. مزدوری, کرایہ, کرایہ سٹور and منشیانہ are **payables**, not shared. |
| **Reports and analytics** | Reports read the books and give ledgers and registers for a day and a party. Analytics read the stored value of each sale line and give totals by item, seller, buyer, partner, garden, trip and fruit. |
| **Day close and month close** | Day close locks the days and starts the backup. Month close moves a finished month's income and expenses into partner capital accounts. It never changes کل رقم. |
| **Back up now, test restore, restore** | Back up now makes a copy. Test restore opens a copy in a **separate place** and checks it. Restore **replaces** your data after a safety copy. |
| **Local, USB, cloud copy** | Local every 15 minutes (plain). USB and cloud are encrypted with the backup key. Cloud needs a real account. |
| **Mac and Windows** | The Mac runs the same app in WebKit. Urdu shaping, the real printer, PDF, the installer and speed on the shop laptop are only proved on Windows. |

## 4. انڈکس (Index)

**Open:** main menu → انڈکس. You see the lists: Chart, Accounts, Opening balances, Commission rules, and the master lists (cities, items, packings, addas, marks, stores, narrations, expense heads, staff).

### 4.1 The menu [LOOK]

Each card opens its list. The words on cards are from the glossary in both languages.

### 4.2 Chart of accounts [LOOK]

**Open:** انڈکس → Chart. **You must see** three levels: Head (01 Assets, 02 Payables, 03 Parties, 04 Income, 05 Expenses, 06 Capital), Control, Account, with live balances. Arrow keys expand and collapse.

| Account | Code | After the seed |
|---|---|---|
| روکڑ (Cash) | 01010001 | 50,000 بنام |
| رحیم (Rahim) | 030110001 | 15,000 بنام (his advance) |
| ابتدائی سرمایہ | 06010001 | 65,000 جمع |
| ابتدائی فرق | 06019999 | 0 |
| System accounts | 02010001 to 02010004, 04010001 to 04010005, 03040000, 03049999, 030119999, 030119998 | present, 0 |

**Try to break it:** make روکڑ inactive. It must be refused (system accounts cannot be made inactive).

### 4.3 Accounts (list, new account, tag, search)

**Open:** انڈکس → Accounts. **List [LOOK]:** columns are code, Urdu name, English name, city, opening date, opening debit, opening credit, credit allowed, current balance. The footer shows the total debit and total credit. The filter takes All, Seller, Buyer, Money lent, Other.

**T4.3-a New buyer, English letters to Urdu [BOOKS: masters only] (E2E-23).**

| Field | Type |
|---|---|
| Account type | Buyer (گاہک) |
| Urdu name | `tariq mehmood` (badge shows رومن). Press `Space` or `Enter` to take the first suggestion. |
| Phone | `03001234567` |
| Credit is allowed | ticked (default) |

You must see: Urdu name طارق محمود; English name "Tariq Mehmood" filled by itself; the next code **03040003** (on option B). After save, the card shows the phone as 0300-1234567. In a buyer picker, typing `tarq` finds him. Switch to English with `Alt+L`: he shows as "Tariq Mehmood". The Urdu spelling you pick may differ from طارق محمود. The test is that a spelling is offered and the English name is filled.

**T4.3-b New seller.** Account type Seller, Urdu name field `bashir ahmad`. The next code is **030110003**.

**T4.3-c The tag (نشان) and the duplicate check.**

1. Open a new buyer. Name `tariq mehmood`, phone `03001234567`, no tag. While you type, the similar account (Tariq, with code, phone) shows below the name. Do not save.
2. Same name and phone, tag `k` (lower case). It becomes `K`. It is **not** a duplicate. Save. The picker now shows "طارق محمود K" and "Tariq Mehmood K".
3. Open one more, same name, phone and tag `K`. The duplicate warning shows. Do not save.
4. In a buyer picker, type `tariq k`. The account with the tag comes first.

**Try to break it (do not save):**

| Typed | Must say |
|---|---|
| English letters in the Urdu name | "اردو نام اردو حروف میں لکھیں" |
| Urdu letters in the English name | "انگریزی نام انگریزی حروف میں لکھیں" |
| Tag `K1`, `کے`, `KKKKK` or a space | Refused (1 to 4 English capital letters and dots only) |
| Phone `0300123` | Refused (wrong length) |
| Empty name | Required |
| Decimal opening `500.50` | "رقم پورے روپوں میں لکھیں" |

**T4.3-d The three typing modes.** In an Urdu name field press `Alt+U`. In **اردو کی بورڈ** mode, `a` types ا, `b` ب, `p` پ, `T` ٹ. In **انگریزی** mode nothing converts. The badge in the field shows the mode. Codes, numbers and phone fields never convert.

**T4.3-e Spelling check.** Type a rare name, for example `shamroz`. A word that came only from the spelling rules shows amber "ہجے دیکھ لیں". On save the app asks you to confirm or fix it. A word from the word list shows no mark.

**T4.3-f Quick add and unknown code.** Do this on the بکری screen (Section 5.1): in a buyer field type `99` and press `Enter`. You see "اس کوڈ کا کوئی گاہک نہیں" with an `F4` button. `F4` opens a small form and keeps what you typed.

**T4.3-g Inactive.** Edit a test account, untick Active. It disappears from every picker and stays in reports. An account with entries cannot be deleted, only made inactive.

**T4.3-h Search in both scripts.** In a buyer picker: `akrm` finds اکرم ٹول ٹیکس; `اکرام` finds him too; the Urdu `اكرم` with an Arabic ك finds him. `Alt+↓` with the field empty shows the last 10 used.

### 4.4 Opening balances [BOOKS, nets to zero] (Run 3f)

**Open:** انڈکس → Opening balances. **Type** on a new throw-away buyer: opening date today, opening debit `1,000`. **You must see:** ابتدائی فرق becomes 1,000 and the trial balance still balances. Set the opening back to empty: the difference is 0 again.

**Try to break it:** an opening on an income account (head 04), ابتدائی فرق, نقدی منڈی, the two پختہ accounts, اپنا مال, بینک, a garden's account or a partner's 0604 account must be refused. Both debit and credit on one account is refused. "This opening amount is an advance" shows only for a seller with an opening debit.

### 4.5 The other master lists

**Open:** انڈکس → each list. Add one record in each. The code fills by itself. The other-script name fills by itself.

| List | Type | Notes |
|---|---|---|
| City | `multan` | |
| Item (قسم مال) | `orange` → مالٹا, group **fruit**, sold by unit, default packing Box | **Needed for Run 3c (trips).** |
| Packing | `crate` | Approx weight in kg |
| Adda | `multan adda`, city Multan | |
| Mark (مارکہ) | text `AR`, owner رحیم | Kept as typed, one script |
| Store | `main store` | |
| Narration | Urdu text and English text, used in | |
| Expense head | `tea` | An expense head is always a دکان خرچہ head |
| Staff | `aslam`, job Labourer | **Needed for money lent (Section 6.4).** |

You must see in each list: the filter box ("Type a code or a name") narrows the rows; "Show the inactive ones too" brings inactive rows back; a record used in an entry cannot be deleted.

### 4.6 Commission rules and "Test a rule"

**Open:** انڈکس → Commission rules. **You must see** the starting rules: seller commission 6%, buyer commission 6%, and buyer لاگا 8 per نگ for the group fruit.

**T4.6-a Test the default.** Open "Test a rule". Seller کریم, buyer اکرم, item کیلا (Banana), date today, qty 20, rate 600, press Test.

| Charge | You must see |
|---|---|
| Seller commission | 720, from the global default (level 6) |
| Buyer commission | 720 |
| لاگا | 160, from the item group fruit (level 5) |

Item مولی (Radish): no لاگا. The screen also lists the levels it skipped.

**T4.6-b New rules dated in the future (so they do not touch today's entries).** Use From date `01-12-2026` and test with that date.

| New rule | Test | Result |
|---|---|---|
| Seller commission, party رحیم, item Banana, percent 5 | رحیم sells Banana 20 × 600 | **600**, level party + item |
| Seller commission, party رحیم, no item, percent 4 | رحیم sells Apple 20 × 600 | **480**, level party |
| Seller commission, no party, item Apple, per nag 30 | کریم sells Apple 20 × 600 | **600**, level item |
| (none) | کریم sells Banana 20 × 600 | **720**, the default |

Untick Active on these three rules when you finish.

**Try to break it:** seller side must not offer لاگا; buyer side must not offer "per lot"; labour, munshiana and phone offer per nag and per lot only; Group works only when party and item are both empty. Two overlapping rules at one level show a warning; the newer start date wins.

**Not the same as:** a rule is for a rate that applies every time. A rate typed on one sale line (for example 4%) is saved with method دستی (manual) and affects only that line.

## 5. بکری (Sale area)

**Open:** main menu → بکری. The side menu: بکری بک, وصولی / اگراہی, ادائیگی, پیشگی کھاتہ, پختہ کھاتہ, آج کی بکری, باقی مال. The first three after بکری بک open **one** voucher screen (Section 5.3).

### 5.1 بکری بک: Event 1 [BOOKS] (E2E-1 to E2E-4, E2E-22, E2E-26)

**Open:** بکری → بکری بک, or `Alt+B`. Date: leave it (today).

**Type, header:**

| Field | Value |
|---|---|
| Truck number | `LES-1234` |
| Zamindar | `1` then `Enter` (رحیم) |
| Truck stand (اڈا) | سوات اڈا (comes from the zamindar if set; otherwise pick it) |

**Type, lot 1** (open "مزید" for the extra fields):

| Field | Value |
|---|---|
| Seller | `1` (رحیم) |
| Item, packing | کیلا (Banana), پیٹی (Box) |
| Quantity received | `50` |
| Freight | `3,000`, paid by آڑھتی |
| Labour (مزدوری) | `250` |
| Munshiana (منشیانہ) | `50` |
| Phone (فون) | `20` |
| Cash paid (نقد) | `5,000` |
| Store rent | empty (0) |
| Deduct from advance | ticked (default) |

**Type, sale lines** (credit, ادھاری). Use `Enter` between cells.

| Line | Buyer | Qty | Rate |
|---|---|---|---|
| 1 | `1` (اکرم) | `20` | `600` |
| 2 | `2` (ارشد) | starts at **30**; rate hint "600"; press `Ctrl+D` to copy 600, then type `580` | `580` |

**You must see, on the lines:**

| | اکرم | ارشد |
|---|---|---|
| Value | 12,000 | 17,400 |
| Seller commission | 720 | 1,044 |
| Buyer commission | 720 | 1,044 |
| لاگا (qty × 8) | 160 | 240 |
| Buyer pays | **12,880** | **18,684** |

**You must see, in the live summary:**

| Line | Amount |
|---|---|
| خام بکری | 29,400 |
| کرایہ | 3,000 |
| کمیشن | 1,764 |
| مزدوری | 250 |
| منشیانہ | 50 |
| نقد | 5,000 |
| کرایہ سٹور | 0 |
| فون | 20 |
| کل اخراجات | **10,084** |
| صافی بکری | **19,316** |
| گاہک کمیشن | 1,764 |
| لاگا | 400 |
| Buyer bills, credit | 31,564 |
| Partner split of 3,948 | الف **1,580**, ب **1,184**, ج **1,184** |

**While you type, check these helpers:**

1. **Party card (E2E-24).** After buyer `1` + `Enter`: name, balance 0, credit limit. After `2`: same for ارشد.
2. **Remaining qty (E2E-26).** Line 2 qty starts at 30 (50 − 20).
3. **Rate hint (E2E-26).** The rate cell is empty with a grey "آج کا آخری ریٹ 600". It is never saved by itself.
4. **ⓘ (E2E-26).** Click ⓘ (or `Ctrl+I`) on اکرم's seller commission: "12,000 × 6 ÷ 100 = 720 (اصول: ...)". The rule tag on the line names the rule.
5. **Language switch (E2E-22).** After اکرم's line press `Alt+L`. The line reads "Akram Toll Tax", "Banana", "Box", 20, 600, 12,000. The layout is left-to-right. Enter ارشد's line. Press `Alt+L` again and carry on.
6. **Save** with `F10`. You see "بکری 1 محفوظ ہو گئی" with a "واپس لیں" button for 10 seconds. Do not press it now.

**After the save [LOOK]:** the number is **1**. Print (`F9`): seller bill (R1), buyer slips, پختہ slip. See Section 11.5 for what a print must contain. English bill: "Gross Sale 29,400", "Net Sale 19,316", "Nineteen thousand three hundred sixteen rupees only".

**Journal check (later, in R11):** sale part بنام 31,564 = جمع 31,564; the whole entry with the cash part 36,564 = 36,564.

**Try to break it (do not save):**

| Typed | Must say |
|---|---|
| Lines 30 + 30 on a 50-nag lot | Blocked: "مقدار باقی مال سے زیادہ ہے: باقی 20 نگ ہیں" |
| Qty `0`, empty rate, rate `0` | Required, more than 0 |
| Rate `600.50` | "رقم پورے روپوں میں لکھیں" |
| Qty `20+25` | Becomes 45 before you leave the field |
| A line with no buyer | Required |
| Date tomorrow, or on or before the closed-up-to date | Refused |
| Qty 45 of 50 sold | Warning "5 نگ باقی ہیں". Save is allowed (see Run 3a). |
| Buyer `99` | "اس کوڈ کا کوئی گاہک نہیں" with `F4` (T4.3-f) |
| Type `4` in a line's commission rate tag | The line is saved with method دستی. Then undo it. |
| A buyer with a low credit limit | Amber warning on the line straight away |
| Truck `LES-1234` typed again | Truck list; picking it fills اڈا, زمیندار, driver from the last بکری |

**Edit and edit back [BOOKS, nets to zero] (E2E-13, E2E-14).** Do this at step 1.8. Open بکری 1 (`F3`, or from آج کی بکری). Change اکرم's rate from 600 to `610`. Save.

| After the edit | Amount |
|---|---|
| اکرم's line | 20 × 610 = 12,200 |
| Buyer commission | 12,200 × 6 ÷ 100 = 732 |
| He pays | 12,200 + 732 + 160 = **13,092** |
| خام بکری | 29,600 |
| Seller commission | 732 + 1,044 = 1,776 |
| کل اخراجات | 10,096 |
| صافی بکری | **19,504** |
| Shared income | 1,776 + 1,776 + 400 + 20 = 3,972 |
| Split | 1,590 / 1,191 / 1,191 |

ارشد's line must not change (unchanged lines keep their stored values). The audit log shows the old and the new version. Edit the rate back to `600`: every number returns exactly to the table above.

**E2E-13b, a new default rate does not change saved lines [BOOKS] (Run 3f, fresh start).** Enter Event 1. Then add a rule: buyer side, commission, no party, no item, no group, percent `7`, From date = the date of بکری 1 (today). Edit اکرم's rate to `610` and save.

| After the edit | Amount |
|---|---|
| اکرم's changed line uses 7% | 12,200 × 7 ÷ 100 = **854**; he pays 12,200 + 854 + 160 = **13,214** |
| ارشد's unchanged line | keeps its stored **1,044** |
| Shared income | 1,776 + 1,898 + 400 + 20 = **4,094** |
| Split | **1,638 / 1,228 / 1,228** |

**Not the same as:** the day must be open to edit. If the day is closed, the screen shows a notice with a button to the day screen.

**Both languages:** the header, lot fields, line columns, summary labels and the toast change; truck number, quantities, rates and amounts do not.

### 5.2 بکری 2: Event 2, the one-time buyer [BOOKS] (E2E-5, E2E-25)

**Open:** `Alt+B` again. Date: same as بکری 1.

**Check first (E2E-25), on a throw-away form.** Type truck `LES-1234` and the seller `1` in a lot. You see "اس ٹرک کی آج بکری نمبر 1 پہلے سے موجود ہے" with two buttons: کھولیں and نئی بکری جاری رکھیں. The warning shows when the form's date is the same as the earlier بکری. Choose "continue". The app fills سوات اڈا and زمیندار رحیم, and the lot fills کیلا and پیٹی. Press `Esc` and leave without saving. Then press `Alt+B` again for the real بکری 2.

**Type, header and lot:**

| Field | Value |
|---|---|
| Truck | `LHR-0077` |
| Zamindar | `2` (کریم) |
| Truck stand | لاہور اڈا |
| Lot seller | `2` (کریم), item سیب (Apple), packing پیٹی, qty `10` |
| Freight | `1,000`, paid by آڑھتی |
| Labour, munshiana, phone | `50`, `50`, `20` |
| Cash paid | empty |

**Sale line:**

| Field | Value |
|---|---|
| Buyer | پختہ کھاتہ (خریدار) 03049999 (type `pukhta` or `پختہ` and pick it) |
| One-time name | `shahid` → شاہد / Shahid |
| Phone | `03000000000` |
| Qty, rate | `10`, `1200` |
| Payment type | **نقدی** (cash) |

**You must see:**

| | Amount |
|---|---|
| Value | 12,000 |
| Seller commission, buyer commission | 720, 720 |
| لاگا | 80 |
| Buyer pays | **12,800** |
| Seller deductions | 1,000 + 720 + 50 + 50 + 20 = **1,840** |
| Seller net | **10,160** |
| Shared income | 1,540 |
| Split | الف **616**, ب **462**, ج **462** |
| Whole entry | 25,600 = 25,600 (sale part 12,800, cash part 12,800) |

**Try to break it (do not save):** a one-time buyer with no name; a credit (ادھاری) one-time buyer with no phone. While you type the name, the list shows known one-time people first, then Urdu spellings. Picking an old person reuses his exact name and phone.

**Not the same as:** a named buyer's cash line stays on his own ledger. A one-time buyer's line goes to the پختہ account under his own person key (name + phone).

### 5.3 The voucher screen: وصولی, ادائیگی, پیشگی

**Open:** side menu → وصولی / اگراہی, ادائیگی or پیشگی کھاتہ, or `Alt+V`, `Alt+A`, `Alt+P`. It is **one** screen. The type list at the top says which. Changing the type keeps date, amount and narration and clears the rest.

**T5.3-a Look at the advance screen before Event 1 (or in a fresh start).** Pick type پیشگی and seller `1`. The card shows his open advance **15,000** and the date and amount of his last advance (01-09-2026, 15,000). Do not save.

**T5.3-b Cash below zero asks, never blocks (FR-6.7).** In پیشگی, seller رحیم, amount `60,000`. While you type, an amber note: "روکڑ صفر سے کم ہو جائے گی" with the cash after the entry (**−10,000** when cash is 50,000). Press save: a box asks "اس اندراج کے بعد روکڑ −10,000 ہو جائے گی۔ کیا یہ اندراج درست ہے؟". The focus is on **"نہیں، واپس جائیں"**. Press `Enter`. Nothing is saved and the amount field is selected. Then type `2,500` and note there is no question.

**T5.3-c Event 3: وصولی [BOOKS] (E2E-6, E2E-24).**

| Field | Value |
|---|---|
| Type | وصولی |
| Buyer | `1` (اکرم) |

You must see first: his card shows **12,880** بنام and the oldest open bill (بکری 1, 12,880, still open 12,880, 0 days). The buttons "پورا بقایا" and "پرانا بل" fill the amount. Press "پورا بقایا", see 12,880, then change to `10,000`. The list marks the bill that the money settles. Save.

You must see after save: اکرم **2,880** بنام; aging 0 days. The receipt slip shows the amount in words: "دس ہزار روپے" / "Ten thousand rupees only".

**T5.3-d Event 4: ادائیگی [BOOKS] (E2E-7).**

| Field | Value |
|---|---|
| Type | ادائیگی |
| Seller | `1` (رحیم) |

His payable postings show: بکری 1, seller credit 24,316, advance adjusted 15,000, paid 5,000, still open **4,316**. Press "پورا بقایا" (4,316) and save. You must see: رحیم balance **0**. The پیشگی report: given 15,000, adjusted 15,000, open 0.

**T5.3-e Payment to کریم, do not save (E2E-27).** His payable shows بکری 2, صافی بکری 10,160, still open 10,160; "پورا بقایا" fills 10,160. Press `Esc`.

**T5.3-f The app points to the right type (FR-7.68).** Open a payment for رحیم **after** Event 4. A note says "اس بیوپاری کا کوئی بقایا نہیں۔ کیا یہ پیشگی ہے؟" with a button پیشگی بنائیں. It switches only when you press it.

**T5.3-g Cash lent to a buyer [BOOKS] (E2E-32), Run 2 step 2.2.** Type ادائیگی, party اکرم, amount `3,000`, reason `نقد ادھار`. The reason is required. You must see: اکرم's dues 2,880 + 3,000 = **5,880**; the 3,000 is a bill dated after his بکری 1 bill, it ages and shows in his اگراہی; cash is 3,000 lower; R20 میزان and the dashboard tile "وصولی باقی" are **24,564**.

**T5.3-h Advance variants [BOOKS] (Run 3e).** Each is a fresh start.

| Test | Steps | You must see |
|---|---|---|
| **E2E-28** advance tick off | Do Event 1 with "پیشگی سے کاٹیں" **unticked** | رحیم payable 24,316 − 5,000 = **19,316**; advance open **15,000**; ledger 4,316 جمع. The "ادائیگی باقی" tile and R24 show 19,316 with advance 15,000. After an ادائیگی of 19,316: payable 0, ledger 15,000 بنام, advance open 15,000, and رحیم is **not** in "بیوپاری کے ذمے". |
| **E2E-31** seller returns advance | On the voucher screen, type وصولی, seller رحیم, `10,000` cash. Then Event 1 | After the return: advance open **5,000**, cash **60,000**. After بکری 1: advance adjusted 5,000. The پیشگی report: given 15,000, returned 10,000, adjusted 5,000, open 0. رحیم payable **14,316**, ledger 14,316 جمع. |
| More than he owes | وصولی from a seller for more than his open advance and what he owes | Refused with the open amount |

**Try to break it (do not save):** amount `0`; amount with decimals; a date in a closed day; a return larger than the open amount; a payment above the payable (a warning "یہ رقم بقایا سے زیادہ ہے", save allowed, the extra shows under "بیوپاری کے ذمے" and is **not** an advance); an ادائیگی to a buyer with no reason.

**Not the same as:** see Section 3.1.

### 5.4 پختہ کھاتہ register [LOOK]

**Open:** بکری → پختہ کھاتہ. After Event 2 it lists one line: date, side خریدار, name شاہد, phone 0300-0000000, item سیب, 10 نگ, amount, commission, **paid**. Filters: date, name, phone, paid or due. A person's open lines come from his own person key. "Make regular" (Run 3) creates a proper account and moves his open balance by a جنرل entry.

Two one-time credit buyers (a fresh run, Run 3f, E2E-21): two sales with radish, so there is no لاگا. شاہد 10 × 100 (bill **1,060**) and ناصر 20 × 100 (bill **2,120**), both ادھاری with phone. A وصولی for شاہد of 1,060 leaves شاہد **0** and ناصر **2,120** still due. The PRD numbers 1,000 and 2,000 are the values before commission.

### 5.5 Collection round: اگراہی فہرست [LOOK]

**Open:** the voucher screen in وصولی type → "اگراہی فہرست". **You must see** (before Event 3): اکرم 12,880 and ارشد 18,684, filterable by collector, city and weekday. One row per buyer with his due and an empty column for the amount received. A buyer left at 0 is skipped. The list prints. After Event 3 the list shows اکرم 2,880. Save a round only when you really want those receipts.

### 5.6 آج کی بکری [LOOK] (today's list)

**Open:** بکری → آج کی بکری. After Events 1 and 2 two rows:

| No | Truck | Seller | Item | نگ | خام | صافی |
|---|---|---|---|---|---|---|
| 1 | LES-1234 | رحیم | کیلا | 50 | 29,400 | 19,316 |
| 2 | LHR-0077 | کریم | سیب | 10 | 12,000 | 10,160 |

Filters: date, seller, item, status. `Enter` opens the بکری. With no بکری today: "آج کوئی بکری نہیں۔ نئی بکری کے لیے Alt+B دبائیں".

### 5.7 باقی مال [LOOK] (E2E-12)

**Open:** بکری → باقی مال. After Events 1 and 2 the list is **empty**, with a helpful message. Columns once a lot has a balance: date, بکری no, seller, item, packing, received, sold, written off, taken as fruit, balance, days waiting. A trip's goods show marked اپنا مال with the trip.

### 5.8 Draft recovery and undo [BOOKS, nets to zero]

**Draft (E2E-17).** Start a بکری, type the header and one line. Wait 6 seconds (auto-save is every 5 seconds). Quit the app. Start it again. The app offers to reopen the draft. Reopen it and discard it. A draft has no بکری number.

**Undo (`Alt+Z`).** Save a small دکان خرچہ of `100`. Within 10 seconds press `Alt+Z`. The entry is reversed with the reason "محفوظ کرنے کے فوراً بعد واپس" and a new entry opens with the same values. Discard it. Cash returns to what it was. The cancelled number is not used again.

### 5.9 Cancel بکری 2 [BOOKS] (E2E-15, Run 2 step 2.4)

Open بکری 2, cancel with a reason. Compare with the values **before** the cancel:

| After the cancel | |
|---|---|
| پختہ register | The line is marked cancelled |
| Analytics | سیب is gone |
| Split | 616 / 462 / 462 removed |
| Cash | 12,800 lower |
| Trial balance | Still balances |

A cancelled بکری cannot be cancelled again. A receipt that was matched to a cancelled بکری's bill becomes credit on the buyer and is matched to his next oldest bill.

### 5.10 Part-sold بکری, later sale, write-off [BOOKS] (Run 3a, E2E-16 and E2E-16b)

**E2E-16.** Fresh start. Enter Event 1 again, but sell only 45 of 50: اکرم 20 at 600, ارشد **25** at 580. A warning says "5 نگ باقی ہیں". Save is allowed.

| Result | Amount |
|---|---|
| خام بکری | 26,500 |
| Seller commission | 720 + 870 = 1,590 |
| کل اخراجات | 3,000 + 1,590 + 250 + 50 + 5,000 + 0 + 20 = 9,910 |
| صافی بکری | 16,590 |
| Buyer commission | 1,590 |
| لاگا | 45 × 8 = 360 |
| Shared income | 3,560 |
| Split | 1,424 / 1,068 / 1,068 |
| باقی مال | **5 نگ کیلا** |

**E2E-16b.** Open باقی مال, press "فروخت کریں" on the row. A later sale opens. Sell the 5 نگ to ارشد at `560`.

| Result | Amount |
|---|---|
| Value | 2,800 |
| Seller commission, buyer commission | 168, 168 |
| لاگا | 5 × 8 = 40 |
| ارشد pays | 3,008 |
| Seller credit | 2,800 − 168 = 2,632 |
| Lot charges again? | **No** (freight, labour, munshiana, phone not charged again) |
| Shared income | 376 |
| Split | 152 / 112 / 112 |
| Lot totals | خام 29,300, کل اخراجات 10,078, صافی 19,222 |
| باقی مال | empty |

**Write off.** Instead of the later sale, write off the 5 نگ with a reason (خراب). No money posts. The stock row goes. Undo the write-off the same way as an edit.

**Not the same as:** see Section 3.7.

## 6. روکڑ (Cash book area)

**Open:** main menu → روکڑ. Side menu: روکڑ, خرچہ (with the پھل tab next to it), ادھار رقم, الگ کھاتہ, جنرل, and a link to the voucher screen.

### 6.1 روکڑ بک [LOOK] (E2E-36)

**Open:** روکڑ. Filter by date. After Events 1 to 5 for the worked day:

| | Amount |
|---|---|
| Opening | 50,000 |
| In | 12,800 + 10,000 = 22,800 |
| Out | 5,000 + 4,316 + 3,000 + 2,000 = 14,316 |
| Closing | **58,484** |

There is no box to type a counted cash. The book shows the book only.

### 6.2 خرچہ: Event 5 [BOOKS] (E2E-39, E2E-20)

**Open:** روکڑ → خرچہ, or `Alt+K`. The type list at the top has three types.

**First entry (دکان خرچہ).**

| Field | Value |
|---|---|
| Type | دکان خرچہ (starts here) |
| Expense head | بجلی (Electricity) |
| Amount | `3,000` |
| Paid to | optional |

**You must see before save:** each partner's share of the expense: الف **1,200**, ب **900**, ج **900**. The list below shows the day's shop expenses. Save.

**Second entry, with a type change.**

1. Type دکان خرچہ, amount `2,000`. (It is a mistake on purpose.)
2. Change the type to **ذاتی خرچہ**. The date and 2,000 stay. The expense head is cleared. The screen asks for **partner** and **reason**.
3. Partner الف, reason `گھر کا خرچ`. Return by and "taken by" are optional.
4. You must see: الف's کل رقم **996 → −1,004**; ب and ج show "not affected".
5. Save.

**You must see after Event 5 (E2E-8, E2E-9):** cash **58,484**. Partners: income 2,196 / 1,646 / 1,646; expense −1,200 / −900 / −900; profit **996 / 746 / 746**; الف drawings 2,000.

**More tests.**

| Test | Steps | You must see |
|---|---|---|
| **E2E-20** | A دکان خرچہ of `3,384` (do not save, or on a fresh start) | Shares **−1,354 / −1,015 / −1,015**, sum −3,384 |
| **E2E-29** return [BOOKS] (Run 2 step 2.2) | Type **ذاتی خرچہ واپسی**, partner الف, `2,000` | Cash +2,000; his net taken is 0; his خالص for the period is **996**; the take shows fully returned |
| Right-type note | دکان خرچہ with "Paid to" = `Partner A` (or `الف`) | "یہ شراکت دار کا ذاتی خرچہ تو نہیں؟" with a button ذاتی خرچہ بنائیں. It switches only when pressed. |
| Return-by date | ذاتی خرچہ with return by a day **before** the entry date | "واپسی کی تاریخ اندراج کی تاریخ سے پہلے نہیں ہو سکتی". The same day is allowed. |
| Repeat last month | The button "پچھلے مہینے والے دہرائیں" | Fills this month's regular expenses for you to check and save (needs expenses in the last month) |

**Try to break it:** a return more than his open amount is refused with the open amount; ذاتی خرچہ without a reason; amount with decimals.

**Not the same as:** see Section 3.2.

### 6.3 پھل (fruit taken) [BOOKS] (Run 3a, E2E-42)

**Open:** روکڑ → خرچہ → the **پھل** tab. **Needs:** the part-sold بکری of Section 5.10 (E2E-16 only, before the later sale), so رحیم's lot has **5 نگ** left. Cash is 45,000.

**Entry 1, home fruit.**

| Field | Value |
|---|---|
| For | Partner الف (گھر کے لیے) |
| Source | اپنی دکان (our shop) |
| Lot | رحیم's کیلا lot (5 left) |
| Qty, rate | `2`, `580` (the last rate shows as a grey hint only) |

You must see before save: value **1,160**; الف's کل رقم down by 1,160; رحیم credited 1,160 **with no commission**. After save: رحیم's payable 1,590 + 1,160 = **2,750**, ledger 2,750 جمع; lot balance **3**. His advance is not touched.

**Entry 2, guest fruit from another agent's shop.**

| Field | Value |
|---|---|
| For | مہمان (guests) |
| Source | دوسری دکان (another shop) |
| Agent | حاجی نذیر (a new seller, code 030110003 if free) |
| Item, packing, qty, rate | سیب, Box, `3`, `1,000` |

You must see: value 3,000 split **equally 1,000 / 1,000 / 1,000** (1,000 over 3 partners would be 334 / 333 / 333); حاجی نذیر payable **3,000**; cash still **45,000**.

**After both (E2E-42):** partner totals 1,424 − 2,160 = **−736**, 1,068 − 1,000 = **68**, **68**. Σ −600 + 65,000 = 64,400 = equity. Trial balance **77,610 = 77,610**.

**Try to break it:** qty above the lot balance ("ERR_QTY_EXCEEDS_LOT" shows as the qty message); a trip's goods are not offered in the lot list; cancel the بکری of a lot that has fruit taken (refused until the fruit is cancelled).

The list below the form filters by partner, item and date (day, month or year) and prints (R37). Each partner's statement has a line like "گھر کے لیے پھل: کیلا 2 پیٹی @ 580 = 1,160".

### 6.4 ادھار رقم (money lent) [BOOKS] (E2E-30, E2E-30b, Run 2 step 2.3)

**Open:** روکڑ → ادھار رقم. **Needs:** the staff member اسلم (Section 4.5).

| Step | Date | Entry |
|---|---|---|
| 1 | type `10-10-2026` | Entry **رقم دی**, person اسلم (`F4` adds him if he is not found), amount `5,000`, reason `گھریلو ضرورت`, return by `15-10-2026` |
| 2 | today | Entry **واپسی**, mode روکڑ, amount `3,000` |
| 3 | | **Look at the bell** (E2E-30b) |
| 4 | today | Entry **واپسی**, mode **تنخواہ سے کٹوتی**, salary head تنخواہ, amount `2,000` |

**You must see.** After step 1: open **5,000**, cash 5,000 lower. After step 2: open **2,000**, cash 3,000 higher. At step 3: the bell lists "اسلم 2,000 past its return-by date" (days = today minus 15-10-2026), and R32 marks him overdue. After step 4: open **0**; **cash does not change**; تنخواہ 2,000 is a shop expense split **−800 / −600 / −600**.

R32 shows: lent 5,000, returned 5,000 (cash 3,000, salary 2,000), open 0.

**Try to break it:** a return of 6,000 when 2,000 is open ("ERR_RETURN_EXCEEDS_OPEN" with the open amount); return-by before the entry date; no reason when lending.

### 6.5 الگ کھاتہ (separate khata) [BOOKS] (Run 3d, E2E-38)

**Open:** روکڑ → الگ کھاتہ. **Fresh start.**

| Step | Date | Entry |
|---|---|---|
| 1 | day 1 | `F4`: new khata `family khata` → خاندانی کھاتہ / Family Khata. Entry **رقم دی**, `10,000`, reason `خاندان کا خرچ` |
| 2 | day 2 | Entry **رقم آئی**, `4,000` |
| 3 | day 2 | A بکری lot with seller خاندانی کھاتہ: 10 Box کیلا to ارشد at `900`, ادھاری |

**You must see.** After step 1: the khata **−10,000**, cash 40,000. After step 2: **−6,000**, cash 44,000. After step 3: value 9,000; no seller-side charges; buyer commission 540; لاگا 80; ارشد pays **9,620**; the entry is 9,620 = 9,000 + 540 + 80; the khata shows **+3,000**; shared income 620 split **248 / 186 / 186**; nothing from the 9,000 goes to any partner. The trial balance balances. The screen lists the khata's entries by date with reason, amount and running balance.

**Not the same as:** a garden rented by partners is **not** a الگ کھاتہ. A separate khata is never income or expense and is in nobody's total.

### 6.6 جنرل (journal) [BOOKS]

**Open:** روکڑ → جنرل. Lines: account, بنام, جمع, narration (and person for a پختہ line).

**Try to break it (do not save):** بنام 100 against جمع 90 cannot be saved; both sides on one line; zero amount.

**Short code in a journal line (E2E-24).** Type `1` in an account cell. The list shows **both** "گاہک 03040001 اکرم ٹول ٹیکس" and "بیوپاری 030110001 رحیم" with their types.

**A credit on a seller (Run 3e, fresh start before Event 1).** Debit روکڑ `1,000`, credit رحیم `1,000`. You must see: his advance stays **15,000 open** (a جنرل never adjusts an advance); the 1,000 is payable to him.

## 7. باغات (Gardens) [BOOKS] (Run 3b, E2E-40 and E2E-40b)

**Open:** main menu → باغات. Fresh start. "Day 1 to Day 4" below can all be today (Section 1.3).

**Make the garden.**

| Field | Value |
|---|---|
| Name | `shahpur garden` → شاہ پور باغ / Shahpur Garden |
| Date | day 1 |
| Item | Banana |
| Rent (ٹھیکہ) | `50,000` |
| Paid now (first installment) | `30,000` |
| Partners and shares | الف **60%**, source **دکان سے**; ب **40%**, source **اپنی جیب سے**; ج **0%** |

Saving makes the garden's own account under 0306. **Try to break it first:** shares that do not total 100 are refused.

**Enter, in order.**

| # | Day | Entry | Amount |
|---|---|---|---|
| 1 | 1 | Installment (paid now) | 30,000 |
| 2 | 2 | Installment | 20,000 |
| 3 | 2 | Running cost (labour) | 5,000 |
| 4 | 3 | بکری lot, seller **شاہ پور باغ**, 100 Box کیلا to ارشد at `600`, ادھاری, no lot charges | |
| 5 | 3 | Sale in another shop: shop name, qty 20, money received | 10,000 |
| 6 | 4 | Close the garden (confirm box) | |

**You must see.**

| After | Must see |
|---|---|
| 1 | Parts 18,000 / 12,000; running balance −30,000; rent still due 20,000 |
| 2 and 3 | Parts 12,000 / 8,000 and 3,000 / 2,000. الف paid **33,000** from the shop (total −33,000, cash **17,000**). ب paid **22,000** himself (his total unchanged). Rent still due 0. |
| 4 | Value 60,000; seller commission 3,600; buyer commission 3,600; لاگا 800; ارشد pays **64,400**; the garden's account جمع **56,400**; commission 8,000 split **3,200 / 2,400 / 2,400**. On the lot, نقد and the advance tick are hidden. |
| 5 | Cash **27,000**; money in 56,400 + 10,000 = **66,400**; running balance 66,400 − 55,000 = **11,400** |
| 6 | 66,400 → **39,840** to الف and **26,560** to ب; the garden's account **0**; the garden takes no more entries. |

**Partner totals after closing:** الف −33,000 + 39,840 + 3,200 = **10,040** (his money back plus 60% of the 11,400 profit); ب 26,560 + 2,400 = **28,960**; ج **2,400**. Σ 41,400 + 65,000 = 106,400 = equity. Trial balance **106,400 = 106,400**. Each partner's statement gets the closing line (cost 55,000, sales 66,400, profit 11,400, his %, his part).

**E2E-40b (optional second fresh start).** Same garden, **no** بکری. Only the other shop's 10,000 comes in. Close. Loss 10,000 − 55,000 = **−45,000**. 10,000 → 6,000 / 4,000. الف −33,000 + 6,000 = **−27,000**; ب +4,000; cash 27,000.

**Try to break it:** an entry on a closed garden (refused); changing shares after the first entry (refused); making a partner inactive while he is in an open garden (refused); editing or cancelling the بکری of a closed garden's lot (refused until the closing is undone); a later sale or fruit from a closed garden's lot (refused). Undo the closing by reversal while its date is open.

**Look:** the garden screen shows the running balance for every date and the rent still due. The dashboard tile "باغات" shows the open gardens. The partner overview card lists the gardens he is in.

## 8. مارکیٹ سفر (Market trips) [BOOKS] (Run 3c)

**Open:** main menu → مارکیٹ سفر. **Needs:** the item Orange (مالٹا). On a fresh start it is not there: add it first (Section 4.5). Purchases (خریداری) are inside the trip. Day 1 and Day 2 can both be today.

### 8.1 Trip A: E2E-34 and E2E-34b (fresh start)

**Make the trip.**

| Field | Value |
|---|---|
| Date | day 1 |
| Market, sent | `Multan Mandi`, `Aslam` (kept as typed) |
| Partners | الف 40%, ب 30%, ج 30%, all **دکان سے** |

**Purchase.** Item Orange, packing Box, qty `20`, rate `500`, freight `400`. You must see: value 10,000; paid now 10,400; cost per nag **520**; parts **4,160 / 3,120 / 3,120**; cash **39,600**. Nothing posts to a stock account.

**Sell 15 in بکری.** New بکری; lot seller **اپنا مال**; pick the purchase; qty `15`; sell all 15 to اکرم at `600`, ادھاری. The lot has no نقد and no advance tick. Freight starts at 0.

| After the sale | Amount |
|---|---|
| Value | 9,000 |
| Seller commission, buyer commission | 540, 540 |
| لاگا | 15 × 8 = 120 |
| اکرم pays | **9,660** |
| Entry | 9,660 = 540 + 540 + 120 + 8,460 |
| Commission 1,200 split by the firm's shares | 480 / 360 / 360 |
| The seller credit 8,460 goes to the trip's partners | **3,384 / 2,538 / 2,538** |
| Cost of goods sold (shown, not posted) | 10,400 × 15 ÷ 20 = **7,800** |
| Result on the 15 | 8,460 − 7,800 = **660** |
| Partner totals | −4,160 + 480 + 3,384 = **−296**; −3,120 + 360 + 2,538 = **−222**; **−222** |
| Stock left | 5 نگ, cost left 2,600 |

**E2E-34b, day 2.** Write off the last 5 (spoiled). You must see: **nothing posts**; the trip's result 8,460 − 10,400 = **−1,940** in red (R33), each partner's part **−776 / −582 / −582**; own stock 0; the trip **closes by itself**; partner totals still −296 / −222 / −222; Σ −740 + 65,000 = 64,260 = equity; trial balance **66,200 = 66,200**.

### 8.2 Trip B: E2E-41 (another fresh start)

| Field | Value |
|---|---|
| Partners | الف **50%**, source **دکان سے**; ب **50%**, source **اپنی جیب سے**; ج not in it |
| Purchase | 80 Box Orange at `1,200` (96,000) |
| Trip cost | truck `4,000` (reason) |
| Sale | A lot with seller اپنا مال sells all 80 to ارشد at `1,400`, ادھاری, no lot charges |

You must see. Purchase 96,000: parts 48,000 / 48,000. Truck: 2,000 / 2,000, spread over the purchase. الف paid 50,000 from the shop (cash **0**). ب paid 50,000 himself. Total cost 100,000 (**1,250 per nag**). Sale: value 112,000; seller commission 6,720; buyer commission 6,720; لاگا 640; ارشد pays **119,360**; commission 14,080 split **5,632 / 4,224 / 4,224**; the trip's 105,280 → **52,640 / 52,640**; trip result **5,280**; the trip closes. Partner totals: الف −50,000 + 5,632 + 52,640 = **8,272**; ب 4,224 + 52,640 = **56,864**; ج **4,224**. Trial balance **134,360 = 134,360**.

**Try to break it:** a trip cost when no purchase has goods left ("ERR_TRIP_NO_STOCK"); a new purchase or cost on a closed trip; cancel a purchase after part of it is sold or written off; selling more than the purchase has left; selling only part of an own lot (it must be fully sold); changing the shares after the first entry; a payment where every part is from a pocket posts **no** journal entry (the trip records it).

**Also sell in another shop:** date, shop (as typed), purchase, qty, money. The money posts: روکڑ بنام, each trip partner's account جمع his %.

**Look:** R33, analytics view J, the dashboard tile مارکیٹ سفر (open trips, نگ left, cost left), باقی مال marks the stock as اپنا مال with its trip.

## 9. شراکت دار (Partners)

**Open:** main menu → شراکت دار. Screens: overview, setup, capital, statement.

### 9.1 Overview [LOOK] (Checkpoint 1)

One card per partner. After Events 1 to 5:

| Partner | Today's share | Expense share | Profit share | Taken | خالص |
|---|---|---|---|---|---|
| الف | 2,196 | 1,200 | **996** | 2,000 | **−1,004** |
| ب | 1,646 | 900 | **746** | 0 | 746 |
| ج | 1,646 | 900 | **746** | 0 | 746 |
| Total | 5,488 | 3,000 | 2,488 | | |

Each card also shows month share, year share, capital, gardens, trips and fruit (باغ، سفر اور پھل) and کل رقم. Under a card: the open gardens and trips he is in.

### 9.2 Setup

**Open:** شراکت دار → setup. Names in both scripts, share %, effective from, active.

**Try to break it (do not save):**

| Typed | Must say |
|---|---|
| Shares 50 / 30 / 30 (total 110) | Will not save. The running total shows. |
| Effective date in the past | Refused (no back-dating) |
| A new share does not change saved postings | The old split stays |

Until the shares total exactly 100%, a بکری cannot be saved.

### 9.3 Capital [BOOKS] (E2E-35, Run 2 step 2.1)

**Open:** شراکت دار → capital. **Type:** entry **سرمایہ جمع**, partner ب, `50,000`.

**You must see (on the worked-day data):** cash 58,484 + 50,000 = **108,484**; ب's capital 50,000 جمع; his profit share still **746**; shares unchanged; trial balance **135,048 = 135,048**.

**Try to break it:** سرمایہ واپسی of more than his balance shows a warning and can still be saved (capital is not the same as ذاتی خرچہ: it is his own money returned and does not change his خالص).

### 9.4 Statement R31 and کل رقم [LOOK] (E2E-37)

**Open:** شراکت دار → statement, partner ب. After 9.3: lines in date order:

| Date | What | Amount | Running کل رقم |
|---|---|---|---|
| Day 1 | بکری 1 share | 1,184 | 1,184 |
| Day 1 | بکری 2 share | 462 | 1,646 |
| Day 1 | دکان خرچہ بجلی | −900 | 746 |
| Day 2 | سرمایہ جمع | 50,000 | **50,746** |

**Partner totals now:** الف **−1,004**, ب **50,746**, ج **746**. Σ 50,488 + ابتدائی سرمایہ 65,000 = **115,488** = the firm's equity in the trial balance.

**Running order for Run 2.** Totals after each step (worked by the lead from the PRD numbers):

| After | الف | ب | ج |
|---|---|---|---|
| 9.3 capital | −1,004 | 50,746 | 746 |
| الف returns 2,000 (6.2) | **996** | 50,746 | 746 |
| اسلم salary 2,000 (6.4) | 196 | 50,146 | 146 |
| Cancel بکری 2 (5.9) | −420 | 49,684 | −316 |

**Month close (مہینہ بند).** From the overview: the Month Close button. The list shows the months; the preview shows the lines of the closing entry (income accounts بنام, shop expenses جمع, each partner's capital account). Close it, then compare each partner's کل رقم: it must **not** change. "Reverse the close" undoes it. A month that is not finished, or whose days are not closed, is not offered or is refused. October 2026 can only be closed after it ends.

## 10. تجزیات (Analytics)

**Open:** main menu → تجزیات. Eleven views A to K. **Run at Checkpoint 1** (Events 1 to 5 only).

**Filters** (all combine): period (today, this week, this month, last month, this year, last year, custom), group by (day, month, year), item, seller, buyer, city, adda, garden, trip, goods (all, consigned, trip, garden), partner (views F, G, I, J, K), commission side (both, seller, buyer), compare (off, previous period, same period last year). Each view has a chart above the table. Every number opens the lines behind it, and from a line its بکری. Every view prints and exports to Excel with its filters in the header.

| View | Filter: this month, all else "all" | You must see |
|---|---|---|
| **A. By item** | | کیلا: 50 نگ, خام 29,400, avg rate **588**, seller 1,764, buyer 1,764, لاگا 400, total **3,928**, **72.10%**. سیب: 10, 12,000, **1,200**, 720, 720, 80, **1,520**, **27.90%**. Total: 60, 41,400, 2,484, 2,484, 480, **5,448**, 100.00%. |
| **B. By seller** | | رحیم: 1 بکری, 50 نگ, 29,400, 1,764, 1,764, 3,928, صافی 19,316. کریم: 1, 10, 12,000, 720, 720, 1,520, 10,160. |
| **C. By buyer** | | ارشد: 30, 17,400, 1,044, 240, dues **18,684**. اکرم: 20, 12,000, 720, 160, dues **2,880**. پختہ شاہد: 10, 12,000, 720, 80, dues 0, avg days to pay 0. (Empty "avg days" where nothing is fully paid.) |
| **D. Item × period** | group by month | کیلا 3,928; سیب 1,520 for October |
| **E. Seller × item** | | رحیم × کیلا 3,928; کریم × سیب 1,520 |
| **F. By partner** | | الف 2,196 / 1,200 / 996; ب 1,646 / 900 / 746; ج 1,646 / 900 / 746; total 5,488 / 3,000 / 2,488. Side by side: gardens, trips and fruit, and his net change. |
| **G. Trend** | group by day | The day: خام 41,400, commission 5,448, shared income 5,488, shop expenses 3,000, profit 2,488 |
| **H. Top 10** | | کیلا first among items; رحیم first among sellers |
| **I. Gardens** | after Run 3b | Per garden: paid, sales (our shop, other shops), commission, result, each partner's % and result |
| **J. Trips** | after Run 3c | Per trip, purchase, item or day: bought, sold, commission, written off, left, result |
| **K. Fruit** | after Run 3a | Per partner or guests and item: nag and value, by source |

**Combine filters (G1):** item کیلا, seller رحیم, group by month, 2026 → 3,928. Item سیب → کریم only.

**Compare:** "previous period" adds the comparison column. **Cancelled postings cancel themselves out** (a cancelled بکری adds nothing, E2E-15).

**Try to break it:** a custom period with "to" before "from"; a view letter that does not exist ("This view does not exist", nothing is affected). Analytics changes nothing: it is read only.

**Not the same as:** see Section 3.9.

## 11. رپورٹ (Reports)

**Open:** main menu → رپورٹ. Four groups: بکری, Accounts, اگراہی / ادائیگی / پیشگی, Business. Each report first shows filters (از, تا, party, item, city, adda) with the date shortcuts آج, کل, یہ ہفتہ, یہ مہینہ, پچھلا مہینہ. Every report has: screen view, print, PDF, Excel. A row opens its source بکری or voucher (Enter or click). Lists load page by page (100 rows) with the **total over the whole filter**; print and Excel hold every row.

### 11.1 The numbers to check at Checkpoint 1 (day = the worked day)

| ID | Report | You must see |
|---|---|---|
| R1 | بکری بک | بکری 1 in full: gross 29,400, deductions 10,084, net 19,316 |
| R2 | Full Sales Register | Every line: بکری no, buyer, item, (qty @ rate), amount |
| R3 | نقدی sales | Only شاہد's line, 12,800 |
| R4 | ادھاری sales | اکرم and ارشد |
| R5 | ادھاری تفصیل | اکرم: "(کیلا 20 . 600)", **12,000**, **12,880**. ارشد: "(کیلا 30 . 580)", **17,400**, **18,684**. **No** cash buyer. Print All = one slip per buyer. |
| R6 | بکری تفصیل | Per lot. بکری 1: 29,400, 10,084, 19,316, freight 3,000, commission 1,764, labour 250, munshiana 50, cash 5,000, phone 20. بکری 2: 12,000, 1,840, 10,160, 1,000, 720, 50, 50, 0, 20. |
| R7 | Item-wise Sales | Same as analytics view A |
| R8 | اڈا Report | Swat: 1 truck, 1 lot, 50 نگ, 29,400. Lahore: 1, 1, 10, 12,000. |
| R9 | پختہ کھاتہ register | شاہد, paid |
| R10 | باقی مال | Empty |
| R11 | روزنامچہ | Every entry; total بنام = total جمع |
| R12 | روکڑ | Opening 50,000; in 22,800; out 14,316; closing **58,484** |
| R13 | کھاتہ (ledger), اکرم | "بکری 1: کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160" **12,880** بنام; وصولی **10,000** جمع; closing **2,880** بنام (E2E-33). Sellers show the lot's item, qty, gross and deductions. |
| R14 | فرد حساب, اکرم | Opening 0, bought 12,880, paid 10,000, balance **2,880**. The option "balance only" shortens it. From-date defaults to the first date of data. |
| R15 | Vouchers | The advance (01-09-2026), the receipt, the payment, the two expenses |
| R16 | Account Index | Accounts with code, city, balance |
| R17 | Head Balances | Balance per head and control |
| R18 | Trial Balance | **85,048 = 85,048** (the table in PRD E2E-10) |
| R19 | Opening Trial Balance | 65,000 = 65,000 |
| R20 | Buyer Dues | ارشد 18,684, اکرم 2,880. میزان **21,564** = the dashboard tile. Aging columns 0-7, 8-30, 31-90, 91+. |
| R21 | اگراہی by City | The same dues grouped by city (the seeded buyers have no city: group "No city") |
| R22 | اگراہی by Collector | Grouped by collector (group "No collector") |
| R23 | وصولی Register | اکرم 10,000 |
| R24 | Seller Payables | کریم **10,160** payable; رحیم 0; open advance 0 |
| R25 | ادائیگی Register | رحیم 4,316 |
| R26 | پیشگی کھاتہ | رحیم: given 15,000, returned 0, adjusted 15,000, open 0 |
| R27 | Bill Series | بکری 1 and 2 in number order: gross, deductions, net |
| R28 | Income Report | Seller commission 2,484; buyer commission 2,484; لاگا 480; فون 40. Total **5,488**. |
| R29 | خرچہ Report | بجلی 3,000; ذاتی خرچہ taken by الف 2,000 |
| R30 | Profit and Partner Split | 5,488 − 3,000 = **2,488** → 996 / 746 / 746 |
| R31 | Partner Statement | See Section 9.4 |
| R32 | Money Lent | After Section 6.4 |
| R33 | Trading Profit / Loss | After Run 3c |
| R34 | Day Summary | Day: 2 بکری; خام 41,400; commission 5,448; cash opening 50,000, in 22,800, out 14,316, closing 58,484; buyers' dues 21,564 (E2E-36) |
| R35 | Party Day Sheet | See 11.2 |
| R36 | Gardens Report | After Run 3b |
| R37 | Fruit Register | After Run 3a |

R5 filtered to اکرم and the day: one row. Both headers show the date and, for R5, the buyer.

### 11.2 R35 party day sheet (E2E-43)

**Open:** رپورٹ → Party Day Sheet, day = the worked day.

**One party's slip (the client's layout).** For اکرم: header (firm, contacts, trade mark), "بنام اکرم ٹول ٹیکس", date, one row **"20, پیٹی کیلا, 600, 12,880"**, total نگ 20, then تازہ بنام **12,880**, سابقہ رقم **0**, وصولی **10,000**, ٹوٹل رقم **2,880**.

**All parties in one table.** Columns: کوڈ (short code), پارٹی (name with tag), سابقہ, بنام, جمع, بقایا, a blank column for the collector, sorted by Urdu name, with a grand total.

| Buyer | سابقہ | بنام | جمع | بقایا |
|---|---|---|---|---|
| اکرم | 0 | 12,880 | 10,000 | 2,880 |
| ارشد | 0 | 18,684 | 0 | 18,684 |
| شاہد (پختہ) | 0 | 12,800 | 12,800 | 0 |
| **Total** | 0 | 44,364 | 22,800 | **21,564** |

The total بقایا equals the R20 میزان. Sellers are in a second table: رحیم 15,000 بنام → 24,316 sold, 5,000 on the spot, 4,316 paid that day, remaining 0; کریم 0, 10,160, 0, 0, 10,160 جمع. Each row satisfies سابقہ + بنام − جمع = بقایا.

### 11.3 Filters, paging and the party filter

- Pick a buyer in the party filter: only his rows, and the print header shows his name.
- Dates: one day or from and to. Balance lists (R20, R24, R26, R32) take an "as of" date instead.
- Page by page: a long list shows the first 100 rows and the total at once.

### 11.4 Excel

Press the Excel button. The file opens with the same headings as the screen in the app language. Money cells are whole rupees. A percent is text. Open it in Numbers or Excel on the Mac to look.

### 11.5 What every print must contain (FR-19.13)

Check this on every print you open (preview on screen on the Mac):

| Part | Must be there |
|---|---|
| Header | Firm name, mandi name, up to **3 contact persons with phones**, the **trade mark in a box** (set in Settings → Firm), the title, the filters (dates and party), the print date and time |
| Body | Every column labelled; a party shown with its **short code** and **name with tag**; numbers on the right edge with equal-width digits; a running balance where the report has one |
| Footer | Totals under their columns; amounts in words where the PRD asks (bills, slips, receipts); **"صفحہ x از y"** / "Page x of y" |
| Digits | English digits 0-9 in both languages. Dates as DD-MM-YYYY. |
| Print language | The print dialog offers اردو / English. A slip can print in Urdu while the app is in English. Names follow the print language; notes print as typed. |
| Mixed text | An Urdu note inside an English print keeps its own direction. "(کیلا 20 . 600)" never flips. |

A phone prints as 0300-1234567 (the digits are stored as 11 digits).

**Not the same as:** R5 and R35 use the **client's layout**. Every other print uses our one clear layout.

## 12. بیک اپ (Backup)

**Open:** main menu → بیک اپ. Copies are made by themselves: a local copy every 15 minutes and on exit; USB when a drive is plugged in; the cloud copy at closing time and at exit.

**T12-a State [LOOK].** The state box shows the last copy, the last test restore, and whether a test restore is due ("No test restore has worked in the last 30 days"). The bottom bar shows "Cloud backup: not set up yet" until a cloud account is set. It turns red after 24 hours without a good cloud copy.

**T12-b Back up now [BOOKS: none].** Press **Back up now** at Checkpoint 1. A progress shows; a new row appears in the list with date, size, place (local) and status OK. The row was checked after it was made (integrity, record counts, checksum).

**T12-c Test restore.** Select the row, press **Test selected** (or Test restore). The copy is restored into a **separate place**; your real data is not touched. You must see "This copy is good. It restores correctly and its books balance." and the totals of the test copy next to the live books. Press Close.

**T12-d Restore [BOOKS: replaces all] (E2E-18, Run 2 step 2.6).**

1. Note on screen: cash, the trial balance total, and each partner's کل رقم.
2. Take a backup (Back up now).
3. Make one more entry (a دکان خرچہ of `100`).
4. Select the backup and press **Restore selected** (or `Enter` on the row).
5. The preview shows the date of the copy, the totals in the copy and now, and "Everything entered after this copy will be lost: N posted entries ...". It says a safety copy is made first.
6. Confirm. You are signed out. Sign in again.
7. You must see: the 100 entry is gone and **every number equals what you noted** in step 1. At Checkpoint 1 that is cash 58,484 and trial balance 85,048.

A copy from an older version of the app upgrades by itself after the restore.

**T12-e Key and cloud [WINDOWS/LOCAL-ONLY].** A USB or cloud copy asks for the **backup key** on restore if this computer does not have it ("Copy it exactly as printed, with the dashes."). The key is shown **once** at setup and printed. A real cloud upload needs a real Cloudflare R2 account (D5-Q2). With no internet the copy waits and retries every 10 minutes.

**Try to break it:** restore with a wrong key; restore with nothing selected; start a restore while a backup runs ("A backup or restore is running. This page follows it.").

**Not the same as:** see Section 3.9.

## 13. ترتیبات (Settings, day close, audit, updates)

**Open:** main menu → ترتیبات. Tabs: Firm, Language and text, Auto lock, Printing, Day and check, Audit log, App and updates.

### 13.1 Firm

Firm name (Urdu, English), mandi name, address, phones, logo. **Contact persons 1 to 3:** a name in both scripts and a phone, for example "منشی آصف بھٹی" / `0300-4058860`. **Trade mark:** text such as `RH`. Save. Open any print preview: the header shows the firm, the three contacts and the trade mark in a box. An audit entry is written for the change.

### 13.2 Language and text

Default language of the backup pack (default اردو). **Number grouping:** International (1,316,040) or Lakh (13,16,040). **Check:** with Lakh, an amount of 108,484 (cash after the capital step) reads 1,08,484 on screen and on prints. Amounts under 100,000 look the same in both. Default text size. The learned words list: view and remove a pair. A learned pair never changes a saved name.

### 13.3 Auto lock

Idle lock after 15 minutes (setting). Set 1 minute to test. The lock screen asks for the password. Unsaved work stays. The user menu → "Lock the app" locks at once.

### 13.4 Printing

Paper per document (A4, A5, 80 mm), printer, copies. **Urdu digits on Urdu prints: off (default).** Leave it off. The rule for this test is that no print shows Urdu digits (۰-۹). The real printer and PDF are [WINDOWS].

### 13.5 Day and check

Closing time and automatic day close; credit limit **warn** or **block**; rate check on or off with its percent (30%); line-override of commission and لاگا on or off (default on); the **Books check** result (the trial balance difference is 0 and the books balance).

**Credit limit test.** Give ارشد a credit limit of `10,000`. A line for him in a بکری shows the amber warning. With **block**, the save is refused. Put it back.

**Rate check test.** Type a rate 40% above the item's 7-day average. The amber note "ریٹ معمول سے بہت مختلف ہے (اوسط ...)" shows. It never blocks.

### 13.6 Day close and reopen [BOOKS] (Run 2 step 2.5)

**Open:** click the day text in the bottom bar ("Day open"), or the bell item "earlier days not closed".

1. **Preview.** Choose "Close up to" = today. You see: the days that will close, the entries in them, the trial balance total بنام, total جمع and the difference (**must be 0**), the cash balance. Press **Close the day** and confirm: "N days up to <date> will be closed."
2. The bottom bar reads "Days up to <date> are closed". Try to edit بکری 1 or save a new entry on that date. It is refused with a notice and a button to the day screen.
3. **Reopen.** Type the date and a **reason**. It reopens that date **and every later date**. Edit now works.
4. The audit log has the close and the reopen with the reason.

**Month close:** see Section 9.4.

### 13.7 Audit log

**Open:** ترتیبات → Audit log. Filters: Who, Action, Area, Record number, Words (looked for in the reason and in old and new values), dates. **You must see** an entry for every create, edit, cancel, login, failed login, logout, rule change, share change, opening change, day close, reopen, month close and restore, with old and new values and the reason. After the edit of Section 5.1, the entry for بکری 1 shows rate 600 and 610. The log is **read only**: there is no edit or delete.

### 13.8 App and updates, support information

Shows the version. "Check for updates" needs a network. An update installs only when you agree, after a backup, never during work. Real updates are [WINDOWS]. **Support information** collects the log, the app version and system info into one file. Open the file: it must hold **no password, key or full balances**.

### 13.9 Sign-in, lock-out, recovery code, first run

| Test | Steps | You must see |
|---|---|---|
| Wrong password | Five wrong tries on the sign-in screen | Wait 1 minute; five more: wait 5 minutes; then 15 minutes each time. It is safe, you just wait. |
| Recovery | "Reset the password with the recovery code" on the sign-in screen | The code printed at setup lets you set a new password. The seed prints it in the terminal. |
| Change password | User menu → Change password | At least 6 characters. The new password works at the next sign-in. |
| Language on sign-in | The اردو / English switch on the sign-in screen | The sign-in screen uses the last language chosen on this computer |
| First-run wizard (option C) | Empty folder | Steps: Firm details, Owner password, Recovery code (print it; "I have printed the code"), Partners (shares must total exactly 100), then the dashboard |
| Guide | User menu → "Show the guide" | A five-step tour: dashboard, انڈکس, بکری بک, وصولی, reports. `Skip` and `Next` work. |

## 14. Dashboard, top bar and bell (check last)

**Open:** main menu → Dashboard (the first screen after sign-in).

### 14.1 Tiles at Checkpoint 1 [LOOK]

Entries dated today (Section 1.3) so "today" tiles match.

| Tile | You must see |
|---|---|
| Today's sales (آج کی بکری) | **41,400**, 2 بکری |
| Today's commission (آج کا کمیشن) | **5,448** |
| Cash in hand (روکڑ بقایا) | **58,484** (red "Below zero" if it ever goes under 0) |
| Buyers' dues (وصولی باقی) | **21,564** (equals R20 میزان) |
| Sellers payable (ادائیگی باقی) | **10,160** |
| Advances open (پیشگی باقی) | **0** |
| Unsold stock (باقی مال) | **0** |
| Trips and Gardens | 0 open (before Run 3) |
| Money lent open | 0 |
| Today's expenses (آج کا خرچہ) | **3,000** (دکان خرچہ only) |
| Last backup | Green under 24 hours, red above |
| Books check | Balanced (the difference is 0) |

**Partner strip:** one card per partner with today's, month's and year's share and کل رقم (الف −1,004, ب 746, ج 746 before Run 2). Click opens his statement.

**Lists:** today's بکری (number, truck, seller, gross, net); the 10 oldest buyer dues; quick actions (new بکری, وصولی, ادائیگی, پیشگی, خرچہ, روزنامچہ); "آج کے کام" (the attention list). **Charts:** last 30 days (خام and commission) and this month's commission by item.

**Every tile is clickable** and opens the screen or report behind it. Press each once.

**Trial balance (E2E-10).** R18: روکڑ 58,484, اکرم 2,880, ارشد 18,684, بجلی 3,000, الف drawings 2,000 on the بنام side = **85,048**; کریم 10,160, commissions 2,484 and 2,484, لاگا 480, فون 40, مزدوری 300, کرایہ 4,000, منشیانہ 100, ابتدائی سرمایہ 65,000 on the جمع side = **85,048**.

### 14.2 The bell (توجہ طلب) and the attention list

The bell shows a count. Each item opens the right screen. Make each one happen:

| Item | How to make it |
|---|---|
| Unsaved drafts | Leave a draft (Section 5.8) |
| Earlier days not closed | The seed closes up to 09-10-2026; any open day before today shows |
| Lots waiting more than 2 days | A part-sold lot, back-dated entry (Run 3a) |
| Buyers over the credit limit | Set a limit lower than a buyer's dues (13.5) |
| Buyer dues older than 30 days | An old entry |
| Seller payables older than 7 days | A back-dated entry |
| Last cloud backup older than 24 hours | Until a cloud account is set, this shows |
| Money lent or partner money taken, past its return-by date | Section 6.4 |
| A record with a missing name | Only after an old-data import |

The empty list reads "Nothing needs your attention".

### 14.3 Top bar and global search

- **Global search (`Ctrl+K`).** Type `akram`, `اکرم`, `1` (the serial), a بکری number, a truck number (`LES-1234`), a voucher number. Results open the right screen.
- **Language switch** `اردو | English` (`Alt+L`): Section 1.6.
- **User menu:** text size, change password, lock the app, sign out, show the guide.
- **Text size:** `Ctrl +` up to 150%, `Ctrl −` down to 80%, `Ctrl 0` normal. Zooming stops where the content would be narrower than 1024 points. Nothing overlaps at any step.

### 14.4 Window sizes

Resize the window from 1024 × 600 to your largest. The window cannot be made smaller than 1024 × 600. There is no sideways scrolling anywhere. Check in both languages. At 1024 and 1366 the analytics filters take much of the first screen (known; it may get a "More filters" fold).

## 15. Windows only (LOCAL-ONLY)

Not proved on the Mac. Tick each one on the shop computer before go-live.

| What | Why only Windows |
|---|---|
| The installer (MSI and NSIS), offline WebView2, the update check and install | Built by the Windows runner |
| Urdu Nastaliq shaping in WebView2, and that **numbers print as English digits** with the Latin digit font | The Mac uses WebKit |
| The real printer: bill, slips, receipt, reports at A4, A5 and 80 mm; PDF files | Tauri prints through WebView2 |
| The full Windows end-to-end suite (WebdriverIO and `tauri-driver`), including the keyboard-only run of PRD Section 4 and the fruit and print specs | Windows only |
| A real cloud upload to Cloudflare R2 (D5-Q2) | Needs the account and keys |
| The Windows code-signing certificate (D5-Q3) | Needs the certificate |
| BitLocker on the laptop disk (NFR-SEC-8), the UPS | Shop computer |
| The speed targets of NFR Section 2 on the reference laptop (Core i3, 4 GB, 1366 × 768): cold start under 2 seconds, key press under 50 ms, save a 30-line بکری under 300 ms, 100,000 vouchers and 1,200,000 sale lines | The Mac is faster than the reference laptop |
| Windows scaling 100%, 125%, 150% | Windows only |

## Appendix A. The worked day on one page

| Event | Entry | Key numbers |
|---|---|---|
| Opening | Cash 65,000 → advance to رحیم 15,000 on 01-09 | Cash 50,000; trial balance 65,000 |
| 1 | بکری 1: رحیم, 50 کیلا, اکرم 20 @ 600, ارشد 30 @ 580 | Gross 29,400; net 19,316; اکرم 12,880; ارشد 18,684; shared 3,948 |
| 2 | بکری 2: کریم, 10 سیب, شاہد (پختہ, cash) @ 1,200 | Buyer pays 12,800; net 10,160; shared 1,540 |
| 3 | وصولی from اکرم | 10,000 → he owes 2,880 |
| 4 | ادائیگی to رحیم | 4,316 → balance 0 |
| 5 | دکان خرچہ بجلی 3,000; ذاتی خرچہ الف 2,000 | Cash 58,484 |
| End | Trial balance | 85,048 = 85,048 |

## Appendix B. Running cash through Run 1 and Run 2

| After | Cash |
|---|---|
| Opening | 50,000 |
| Event 1 (نقد 5,000 to رحیم) | 45,000 |
| Event 2 (شاہد pays 12,800) | 57,800 |
| Event 3 (اکرم 10,000) | 67,800 |
| Event 4 (رحیم 4,316) | 63,484 |
| Event 5a (بجلی 3,000) | 60,484 |
| Event 5b (الف 2,000) | **58,484** |
| Capital ب +50,000 | 108,484 |
| الف returns 2,000 | 110,484 |
| Cash lent to اکرم −3,000 | 107,484 |
| اسلم lent −5,000 | 102,484 |
| اسلم returns +3,000 (salary 2,000 moves no cash) | 105,484 |
| Cancel بکری 2 (−12,800) | **92,684** |

Buyers' dues through Run 2: 21,564 → 24,564 after the cash lent to اکرم.
