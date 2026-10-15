# D4. Money in and out: changes to sales, vouchers, expenses and the full day

Version 1.4, 15-10-2026 (PRD v1.5: opening credit payable, cash warning, return-by date, per-lot commission on edit, opening rules; PRD v1.4: one خرچہ screen and one voucher screen; v1.3: own goods and partner capital; v1.2: partner money returns, ادھار رقم, cash lent to buyers, seller advance returns). Deliverable 4 of 5. Read `README.md` in this folder first.
Depends on: D3 done. Plan files: `15-vouchers`, `16-later-sales-edit-cancel`, `16b-own-goods`, `17-cash-book-and-day-close`.
Build order step: CLAUDE.md MVP step 6. At the end, the whole PRD worked day (Section 4) runs, and every golden test passes except E2E-11, E2E-18, E2E-33 and E2E-36 (analytics, backup and reports), which come in D5.

## Decisions

None open. If a question comes up, follow README Section 8.

## 1. What Umar can do at the end

The munshi runs the whole of 10-10-26 from PRD Section 4, by keyboard:

1. بکری 1 and بکری 2 (from D3).
2. وصولی: اکرم pays 10,000. His card showed 12,880 and his open bill from بکری 1; after saving, he owes 2,880.
3. ادائیگی: رحیم's card shows 4,316 payable, and that his 15,000 advance was already adjusted when بکری 1 was saved (D3). "پورا بقایا" fills 4,316. His balance becomes 0.
4. On the one خرچہ screen: electricity 3,000 as دکان خرچہ, with the split 1,200 / 900 / 900 shown before save; then الف's 2,000 as ذاتی خرچہ, charged to him only (E2E-39).
5. The cash book shows 58,484. The munshi counts the cash and types it in the cash count box.
6. Day close shows the day summary: 2 بکری, خام بکری 41,400, the commission, the partner shares, cash 58,484, trial balance difference 0, nothing left in باقی مال. He confirms.

He can also record the money that moves outside sales. When الف brings back the 2,000 he took, the munshi enters it on the same خرچہ screen as type ذاتی خرچہ واپسی, and الف's خالص goes from −1,004 back to 996 (E2E-29). Staff member اسلم borrows 5,000 under ادھار رقم with a reason and a return date; he returns 3,000 in cash and 2,000 from his salary (E2E-30). A seller can bring back part of his advance (E2E-31), and a buyer can borrow cash, which then sits in his اگراہی like any bill (E2E-32).

The firm can also trade its own goods. On خریداری he records 20 پیٹی مالٹا bought at 500 with 400 freight (cost 10,400). In a بکری he sells 15 of them to اکرم at 600, and the app shows the commission, the cost of the goods sold (7,800) and the profit (660) at once (E2E-34). When the last 5 spoil, the write-off shows the loss (E2E-34b). When partner ب puts 50,000 into the shop, he records it as سرمایہ جمع (E2E-35).

He can also correct a mistake: edit اکرم's rate from 600 to 610, and every number follows; edit it back, and every number returns exactly. He can cancel بکری 2, and everything it did is reversed. If a lot is not fully sold, its remaining goods sit in باقی مال until a later sale on another day.

## 2. Plan files

### 15-vouchers: وصولی, ادائیگی and the rest of پیشگی

**Requirements:** PRD 15.1 to 15.4 (FR-15.1 to FR-15.7, FR-15.9 to FR-15.12), FR-7.67 and FR-7.68 (one voucher screen, the پیشگی note), 13 (FR-13.7, FR-13.8), R26 data (the advances query), FR-7.38 (oldest open bill and payables on the party card), FR-7.58, FR-7.59, FR-7.61, FR-5.4 (voucher numbers in global search); NFR-SPD-9, NFR-SPD-21; Architecture 11; the money-calculations agent. E2E-6, E2E-7, E2E-21, E2E-27, E2E-28 (the ادائیگی part), E2E-31, E2E-32.

**backend builds:**

- وصولی / اگراہی: cash or bank بنام, buyer جمع, with the collector field. It uses the bill matching engine built in D3 (FR-15.4): oldest bills first, by account or by پختہ person key. A payment above what is owed stays as his credit and settles his next bills by itself (FR-15.5).
- اگراہی فہرست: today's planned collections by collector and city, printable (FR-15.6), and the collection round that opens one receipt after another from it (FR-7.61).
- ادائیگی: account بنام, cash or bank جمع. A payment above the payable warns but can save; the extra shows in "بیوپاری کے ذمے" (FR-15.7). Sellers whose advances are more than their sales are in that list too (FR-15.9).
- ادائیگی to a buyer is cash lent to him (نقد ادھار): reason required; it becomes one of his bills for matching, aging and اگراہی (FR-15.11).
- وصولی from a seller (he returns advance money): cash بنام, seller جمع; it adjusts his open advances oldest first, else settles what he owes (FR-15.12).
- Receipts from one-time buyers and payments to one-time sellers through their person key (FR-13.7). "Make regular" creates an account and moves the open balance with a جنرل entry (FR-13.8).
- Edit (reversal + new posting) and cancel with reason for all three voucher types, including the پیشگی of D2 (FR-15.3). Bank and cheque modes with the bank account and cheque details (15.1).
- The party card gains the oldest open bill with its age, and the payable postings with the advance already adjusted against them (FR-7.38, FR-7.58, FR-7.59). "پورا بقایا" fills the full open amount. Advances are adjusted at بکری save when the lot's tick is on (FR-15.8, FR-12.23, built in D3) and by a seller's وصولی (FR-15.12); ادائیگی never adjusts them.
- v1.5 (Umar, 15-10-2026): a seller's opening credit (his oldest) and his جنرل credits are payable postings (FR-7.59, money agent 8.2a), in the party card, the ادائیگی helper, the R24 data and the FR-7.68 note. A جنرل line never adjusts advances (16.4; D2 already builds it this way, keep it). Every cash-out type on the voucher screen uses the FR-6.7 cash check: an amber note while typing, then a confirm box on save.
- The advances query (given, returned, adjusted, open per seller) that E2E-7 checks and that D5 shows as report R26.
- The receipt slip print data with the amount in words (FR-15.2), with the `print-and-export` skill.
- `golden_day.rs`: Events 3 and 4, E2E-6, E2E-7, E2E-21, E2E-28 (open payable 19,316; after the payment of 19,316: ledger 15,000 بنام, advance open 15,000, not in بیوپاری کے ذمے), E2E-31, E2E-32 (Σ buyer debit balances 24,564; R20 and the tile are checked in D5). `assist.rs`: the backend part of E2E-27.

**frontend builds:** one voucher screen with a type list at the top: وصولی, ادائیگی, پیشگی (FR-7.67, FR-15.10). The پیشگی screen from D2 becomes its پیشگی type; its backend command and tests stay as they are. The side menu items and `Alt+V`, `Alt+A`, `Alt+P` open it with the type chosen. Entries can be made one after another, with a running total (FR-15.1). Changing the type keeps date, amount, mode and narration. An ادائیگی to a seller with nothing payable shows the note that offers پیشگی (FR-7.68). (Plan file 17 links the screen from the روکڑ area.) Also: the open bills and payables panels; "پورا بقایا"; the اگراہی فہرست and the collection round; "make regular"; the receipt slip template; the reason field when the ادائیگی account is a buyer; on a seller's وصولی, the open advances it will adjust.

**Done when:** E2E-6, E2E-7, E2E-21, E2E-27, E2E-28, E2E-31 and E2E-32 pass.

### 16-later-sales-edit-cancel: باقی مال, later sales, edit, cancel and undo

**Requirements:** PRD 12.8 (FR-12.14 to FR-12.16), 12.9 (FR-12.17 to FR-12.22), FR-12.11, FR-7.47, FR-6.3, FR-6.6; NFR-SEC-5; Architecture 12; the money-calculations agent. E2E-13, E2E-13b, E2E-14, E2E-15, E2E-16, E2E-16b.

**backend builds:**

- باقی مال: every lot with balance qty above 0 (FR-12.14); "sell" opens a later sale (FR-12.15); write-off with a reason records a stock movement (FR-12.16).
- Later sale: new sale lines on an existing lot, dated the day it happens, as its own posting (FR-12.17). No lot charges a second time (see E2E-16b).
- Edit: any one posting of a بکری, while its date is open (FR-12.18). Saved as a reversal of that posting plus a new posting, both on the same date (FR-12.19). Unchanged charges keep their stored method, value and amount (FR-12.20). The partner split is recalculated from the new shared income (FR-12.21).
- Cancel: reverses every posting of the بکری; every date must be open; a reason is required (FR-12.22, FR-12.11). Receipts matched to its lines become unmatched credit and are matched again to the buyer's next oldest bills (FR-12.22). An edit re-runs the matching the same way.
- Analytics facts are reversed with their postings (FR-18.7). For E2E-15, the check "analytics excludes سیب" is made on the analytics fact rows (سیب nets to zero). D5 checks the view itself.
- Undo straight after saving: the same reversal path as cancel, offered in the success message (FR-7.47).
- v1.5: when an edit removes the line that carries a per-lot seller commission, the charge moves with its stored method and value to the first remaining line of that lot in that posting, so the lot is charged once (FR-11.9, the one exception to FR-12.20).
- Audit rows for every edit, cancel and write-off.
- `golden_day.rs`: E2E-13, E2E-13b, E2E-14, E2E-15, E2E-16, E2E-16b.

**frontend builds:** the باقی مال screen with "sell" and write-off; the later sale form; edit mode on a saved بکری (what changed is visible); cancel with reason (confirm dialog, FR-7.8); the undo action in the success message.

**Worked checks** (PRD 27.2; the tests use exactly these):

- E2E-13: اکرم's rate 600 → 610. His line 20 × 610 = 12,200; buyer comm 12,200 × 6 ÷ 100 = 732; he pays 12,200 + 732 + 160 = 13,092; خام بکری 29,600. The rest as PRD 27.2.
- E2E-14: back to 600; every number returns exactly to E2E-1 to E2E-4.
- E2E-16: only 45 of 50 sold; خام 26,500; seller comm 720 + 870 = 1,590; the rest as PRD 27.2. E2E-16b: the 5 نگ sold to ارشد at 560 on 11-10-26; value 2,800; seller comm 168; buyer comm 168; لاگا 5 × 8 = 40; ارشد pays 3,008; seller credit 2,800 − 168 = 2,632.

**Done when:** the six golden tests above pass with the PRD numbers.

### 16b-own-goods: the firm's own goods (خریداری)

**Requirements:** PRD 12.10 (FR-12.24 to FR-12.28), C21, FR-12.16 (write-off of own goods), 4.2, FR-7.59 (purchase credits are payable), 9.1 (اپنا مال tile data), FR-10.4 (seeded accounts 01040001, 030119998, 04010006, 05020001), E50 to E52, A45 to A47; Architecture 6 and 15.6 (`purchases`, `bikri_lots.purchase_id`, `bikri_sales.cost_paisa`); the money agent Section 5.4. E2E-34, E2E-34b.

`backend` calls the money-calculations agent before writing and again to review.

**backend builds:**

- A migration with the `purchases` table, the new columns and the four seeded accounts (a new numbered file if 0001 is released by then).
- خریداری save, cancel (only while nothing is sold or written off) and list: stock بنام total cost; supplier جمع the value on ادھار, else cash or bank; costs paid at once (FR-12.24). Numbers in their own series (خ-1).
- Own stock per purchase in باقی مال, marked اپنا مال (FR-12.25).
- In `bikri_preview` and `bikri_save`: a lot with seller اپنا مال picks a purchase with stock left; its seller credit posts to تجارتی فروخت; cost of goods sold (C21) posts in the same entry; both are split (FR-12.26). An own lot must be fully sold in its بکری (no balance, no later sale); unsold goods stay in the purchase. Edit and cancel give back exactly the stored cost and qty. نقد, the advance tick and freight are 0 on own lots; اپنا مال is offered only in the lot seller field.
- Write-off of own stock posts its cost as a loss (FR-12.28).
- Profit or loss per purchase (FR-12.27), as data for R33 in D5.
- `golden_day.rs`: E2E-34 and E2E-34b.

**frontend builds:** the خریداری screen in the بکری side menu (form, list, stock and profit per purchase); اپنا مال in the lot seller picker with the purchase picker; own lots marked in باقی مال; the cost and profit lines in the lot summary, each with `CalcInfo`.

**Worked checks** (PRD 27.2):

- E2E-34: cost 10,000 + 400 = 10,400; cash 39,600. Sale of 15 at 600: value 9,000; seller comm 540; buyer comm 540; لاگا 120; اکرم pays 9,660; trading sales 8,460; cost 10,400 × 15 ÷ 20 = 7,800; profit 660; entry 17,460 = 17,460; split income 3,864 / 2,898 / 2,898, cost −3,120 / −2,340 / −2,340.
- E2E-34b: the last 5 written off: loss 2,600, split −1,040 / −780 / −780; purchase profit or loss −1,940.

**Done when:** E2E-34 and E2E-34b pass, and the property test of Code Standards 9.3 for the cost of own goods passes.

### 17-cash-book-and-day-close: روکڑ, expenses, جنرل and the full day

**Requirements:** PRD 16 (16.1 to 16.5, FR-16.1 to FR-16.5, FR-16.8), FR-7.67, FR-7.68 (the partner-name note), 17.3 (taken, returned and خالص columns), C20, FR-7.44 (overdue loans and partner money), FR-7.62, FR-6.5, FR-21.1 (full day summary), 4.6 to 4.9; NFR-REL-8, NFR-URD-29; Architecture 6, 10 and 15.7; the money agent Section 8.2b; partner capital and the partner total (17.4, FR-17.7 to FR-17.9, C22); الگ کھاتہ (16.6, FR-16.6, FR-16.7, control 0305). E2E-8, E2E-9, E2E-10, E2E-12, E2E-20, E2E-24, E2E-29, E2E-30, E2E-30b, E2E-35.

**backend builds:**

- روکڑ بک: opening cash, every cash in and out from all modules, closing cash, for a date range (16.1). Cash count box: shows the difference from the book; posts nothing (FR-16.1).
- One خرچہ screen (16.2) with the types دکان خرچہ, ذاتی خرچہ and ذاتی خرچہ واپسی. A preview command returns, before save, each partner's share (دکان خرچہ) or the chosen partner's کل رقم now and after (ذاتی خرچہ, واپسی), using the same domain functions as the save (FR-16.8, B5). It writes nothing.
- دکان خرچہ: expense head بنام, cash or bank جمع, split between partners by their shares on that date (16.2, FR-6.5). Most-used heads first, and "repeat last month's expenses" (FR-7.62).
- ذاتی خرچہ and its واپسی, types on the same خرچہ screen (16.3): money taken (reason required, optional return-by date and taken-by name) posts drawings بنام, cash or bank جمع; money returned posts the opposite. Never split. The list of takes with returned and still open, worked out at read time (FR-16.2). The partner overview gains taken, returned and خالص (FR-16.3, C20).
- ادھار رقم (16.5): one account per person under control 0103, made with `F4` (and linked to the staff record when he is staff). Lend: person بنام, cash or bank جمع. Return in cash or bank, or by salary deduction: salary head بنام, person جمع, split as a shop expense (FR-16.4). Open amounts oldest first at read time; overdue loans and partner takes in the attention list data (FR-16.5, FR-7.44). The R32 data query (D5 shows the report).
- سرمایہ جمع and سرمایہ واپسی on one screen in the شراکت دار area (17.4): cash or bank and the partner's capital account; never split; a warning when more than his capital is taken back (FR-17.7, FR-17.8). The partner overview gains سرمایہ لگایا and کل رقم (C22). The partner statement data: every dated movement with the running کل رقم (FR-17.9); D5 prints it as R31.
- الگ کھاتہ on one screen in the روکڑ area (16.6): paid out (minus) and received (plus), reason required, list with running signed balance; never split, never in any partner's total (FR-16.6). A الگ کھاتہ can be a lot's seller in بکری: its seller credit goes to the khata, seller-side defaults 0, buyer side as usual (FR-16.7).
- v1.5 (Umar, 15-10-2026): a return-by date before the entry date is refused for ذاتی خرچہ and ادھار رقم; the same day is allowed (FR-16.9). Every save that would take روکڑ below 0 uses the FR-6.7 cash check (note, then confirm box): خرچہ, ادھار رقم, سرمایہ واپسی, الگ کھاتہ رقم دی, خریداری, a جنرل with a cash credit, and an edit, cancel or undo that takes cash back. A partner's drawings opening debit (FR-10.9) shows in his money-taken list and counts in خالص and کل رقم (C20, C22). اپنا مال refuses an opening (FR-10.9).
- جنرل: lines of account, بنام, جمع, narration (and person for پختہ lines); saves only when the totals match; lines on income or shop expense accounts are split (16.4).
- The full day close summary of FR-21.1: بکری count, خام بکری, commission, partner shares, cash in hand, trial balance difference, lots with باقی مال.
- `golden_day.rs`: Event 5 (with its reason) and the whole day, E2E-8, E2E-9, E2E-10, E2E-12, E2E-20, E2E-29, E2E-30, E2E-30b, E2E-35, E2E-37, E2E-38, and the preview numbers of E2E-39 (1,200 / 900 / 900; الف 996 → −1,004). `assist.rs`: the backend part of E2E-24.

**frontend builds:** the روکڑ area with روکڑ بک and the cash count box, the one خرچہ screen (type list; split or partner total shown before save; the day's expenses or the partner's list below the form; the partner-name note of FR-7.68), ادھار رقم (one screen for lent and returned, with the person's list below the form), partner capital (one screen for invested and taken back), الگ کھاتہ (one screen, signed balance), and the جنرل grid (with the `entry-grid` skill), plus a link to the voucher screen (FR-15.10); the full day close summary. The Windows run of E2E-39.

**Worked checks** (PRD 4.9 and 27.2):

- E2E-8: cash 50,000 − 5,000 + 12,800 + 10,000 − 4,316 − 3,000 − 2,000 = 58,484.
- E2E-9: income shares 2,196 / 1,646 / 1,646; expense shares −1,200 / −900 / −900; profit 996 / 746 / 746; الف drawings 2,000.
- E2E-10: the trial balance table of PRD 27.2. بنام 58,484 + 2,880 + 18,684 + 3,000 + 2,000 = 85,048. جمع 10,160 + 2,484 + 2,484 + 480 + 40 + 300 + 4,000 + 100 + 65,000 = 85,048.
- E2E-20: an expense of 3,384 splits −1,354 / −1,015 / −1,015 (sum −3,384).
- E2E-29: الف خالص on 10-10-26 = 996 − 2,000 = −1,004; after returning 2,000 on 11-10-26, net taken 0, خالص 996, cash 60,484.
- E2E-30: اسلم lent 5,000 (cash 53,484); returns 3,000 (open 2,000, cash 56,484); 2,000 from salary (open 0, cash 56,484, تنخواہ split −800 / −600 / −600).
- E2E-35: ب invests 50,000 on 11-10-26: cash 108,484; ب capital 50,000 جمع; his profit share still 746; trial balance 135,048 = 135,048.
- E2E-37: کل رقم الف −1,004, ب 50,746, ج 746; Σ 50,488 + 65,000 = 115,488 = equity; ب's record runs 1,184, 1,646, 746, 50,746.
- E2E-38: باغ −10,000 (cash 40,000), −6,000 (cash 44,000), then the sale of 10 کیلا at 900 adds 9,000: +3,000; ارشد pays 9,620; shared income 620 → 248 / 186 / 186.
- E2E-39: preview of بجلی 3,000 = 1,200 + 900 + 900 (C11, same as E2E-9); الف's کل رقم before his 2,000 = 2,196 − 1,200 = 996, after = 996 − 2,000 = −1,004 (same as E2E-37).

**Done when:** the whole worked day passes in `golden_day.rs`, and the day close summary shows the PRD 4.9 values.

## 3. Design for D4

The lead runs `/design` inside `docs/design/design-system.md` for:

1. The one voucher screen: type وصولی with the buyer's open bills and "پورا بقایا", in both languages.
2. The same screen as ادائیگی, with payable postings, the "more than due" warning and the پیشگی note.
3. The باقی مال list and the later sale form.
4. A saved بکری in edit mode, showing what changed, and the cancel dialog.
5. The روکڑ بک with the cash count box; the one خرچہ screen (each type, with the split or the partner total before save); ادھار رقم on one screen; the جنرل grid.
6. The full day close summary.
7. The receipt slip (A5 and 80 mm).
8. The خریداری screen with stock and profit per purchase, and an own lot in بکری with its cost and profit lines.
9. The partner capital screen.

## 4. Team flow for D4

- Plan files run in order: 15, 16, 16b, 17 (README Section 4).
- 16b after 16: own lots use the later sale, edit, cancel and write-off paths built there.
- 15 (vouchers) before 16 (edit and cancel), because a cancel must turn matched receipts back into credit (FR-12.22), and the attack list mixes both.
- 17 last: E2E-10 and E2E-24 need everything before it.
- `backend` uses the money-calculations agent for all four.

## 5. Review focus for D4

- Reversals: every edit and cancel is a reversal plus a new posting. No posted row is ever changed (CLAUDE.md rule 4). Show the rows before and after an edit.
- Matching: oldest first, against FR-15.4; the sum of a buyer's open bills always equals his balance.
- Partner shares on every income and expense posting, never on payables, never on ذاتی خرچہ or its واپسی. A salary deduction of a loan is split like any shop expense.
- Urdu labels of the new screens say exactly what the English says (NFR-URD-29).
- Shared screens (FR-7.67): each type calls its own service and posting; the preview and the save use the same function; no second screen was built for a type.
- Own goods: the seller credit of an own lot never reaches a seller account; cost of goods sold is in the same entry; Σ cost released per purchase never passes its total cost (C21).
- Partner capital is never split and never counted as income.
- Golden tests: every number in PRD 27.2 for E2E-1 to E2E-10, E2E-12 to E2E-16b, E2E-13b and E2E-19 to E2E-21, unchanged. (E2E-11 is analytics, checked in D5.)

## 6. Robustness attack list for D4

1. **Random days.** Generate thousands of random days: sales, later sales, edits, edits of edits, cancels, receipts (part, full, more than owed), payments, advances, expenses, جنرل entries. After every step: the trial balance difference is 0; each buyer's open bills add up to his balance; each split adds up to its shared income; cash in the cash book equals the روکڑ account.
2. **Closed dates.** Edit, cancel, voucher and expense on a closed date are all refused; after an Owner reopen they work; the audit log has every step.
3. **Order of events.** Cancel a بکری that has a later sale; edit a posting after a receipt has matched its bill (the bill matching recalculates; nothing is lost); pay a seller twice.
4. **Power cut** during an edit (the reversal and the new posting are saved together or not at all).
5. **Speed.** Save any voucher under NFR-SPD-9; party card with 10 years of open bills under NFR-SPD-21; matching against a buyer with 10,000 open bills.
6. **Bad inputs.** Amount 0, negative, decimal; cheque mode with no cheque number; a جنرل entry that does not balance; a ذاتی خرچہ with no partner or no reason; an ادھار رقم with no reason; a salary deduction larger than the open loan; an ادائیگی to a buyer with no reason.
7. **Type switching.** On the خرچہ and voucher screens, change the type many times in the middle of an entry, then save: the saved type, posting and partner split match the type shown at save; no field of the old type is saved.
8. **Returns.** Random sequences of lends, takes and returns (cash, bank, salary): open amounts always equal the account balance; a return never makes an open amount negative.
9. **Own goods.** Random purchases, sales (whole, part, several بکری), edits, cancels and write-offs: Σ cost released per purchase equals its total cost when nothing is left; own stock in باقی مال equals bought − sold − written off; profit or loss per purchase equals trading sales − cost released. Selling more than the stock left, or saving an own lot that is not fully sold, is refused. No voucher or جنرل can post to the اپنا مال account.

## 7. Acceptance (D4-A1 to D4-A31)

| ID | Item | Evidence expected |
|---|---|---|
| D4-A1 | باقی مال lists open lots; write-off with reason (FR-12.14, FR-12.16) | Test output |
| D4-A2 | E2E-16 and E2E-16b pass | golden_day output |
| D4-A3 | E2E-13 passes | golden_day output |
| D4-A4 | E2E-13b passes | golden_day output |
| D4-A5 | E2E-14 passes: every number returns exactly | golden_day output |
| D4-A6 | E2E-15 passes, with "analytics excludes سیب" checked on the analytics fact rows | golden_day output |
| D4-A7 | Edit and cancel refused on a closed date and allowed after reopen; every step audited | Test output |
| D4-A8 | Undo straight after saving reverses everything (FR-7.47) | Test output |
| D4-A9 | E2E-6 passes | golden_day output |
| D4-A10 | Overpayment stays as credit and settles the next bill (FR-15.5); E2E-21 passes | Test output |
| D4-A11 | E2E-7 passes | golden_day output |
| D4-A12 | E2E-24 and E2E-27 pass | assist.rs output and Windows E2E |
| D4-A13 | Edit and cancel for all three voucher types; the receipt slip prints with the amount in words in both languages | Test output; PDFs |
| D4-A14 | E2E-8 and E2E-9 pass | golden_day output |
| D4-A15 | E2E-20 passes; ذاتی خرچہ never split; an unbalanced جنرل entry cannot save | Test output |
| D4-A16 | E2E-10 passes (85,048 on both sides) and E2E-12 passes (باقی مال empty) | golden_day output |
| D4-A17 | The day close summary shows the PRD 4.9 values; the cash count box shows the difference and posts nothing | Test output; screenshot |
| D4-A18 | NFR-SPD-9 and NFR-SPD-21 met at their runner thresholds; the full test suite, including every earlier test, passes on the final main; every D4 screen matches the design | `pnpm perf`; full test run; `windows-verify` report |
| D4-A19 | E2E-28 (open payable 19,316 and the payment part), E2E-31 and E2E-32 (Σ buyer debit balances 24,564) pass | golden_day output |
| D4-A20 | E2E-29 passes; the partner overview shows taken, returned and خالص | golden_day output; screenshot |
| D4-A21 | E2E-30 and E2E-30b pass (an overdue loan shows in the attention list data) | golden_day output |
| D4-A22 | The خرچہ, voucher and ادھار رقم screens work by keyboard in both languages, with the list below the form | Windows E2E names; screenshots |
| D4-A23 | E2E-34 and E2E-34b pass; the cost of own goods property test passes | golden_day output; property test output |
| D4-A24 | خریداری screen and own lots in بکری work by keyboard in both languages; own stock and profit per purchase shown | Windows E2E names; screenshots |
| D4-A25 | E2E-35 passes; partner capital is never split; taking back more than the balance warns | golden_day output; test output |
| D4-A26 | E2E-37 passes; the partner total property test passes (Σ کل رقم + ابتدائی سرمایہ = equity); month close leaves کل رقم unchanged | golden_day output; property test output |
| D4-A27 | E2E-38 passes; a الگ کھاتہ never reaches any partner's total or the partner split | golden_day output; test output |
| D4-A28 | E2E-39 passes (preview numbers in golden_day, the screen run in the Windows suite); changing the type keeps the shared fields; the FR-7.68 notes show and never switch by themselves | golden_day output; Windows E2E names; screenshots |
| D4-A29 | A seller's opening credit is payable (FR-7.59): کریم with an opening 10,160 جمع and an ادائیگی of 4,000 shows paid 4,000, still open 6,160, balance 6,160 جمع, and no پیشگی note. A جنرل credit of 1,000 on رحیم (advance 15,000 open) leaves the advance at 15,000 and shows 1,000 payable; Σ still open − Σ open advances = his balance (16.4, money agent 8.2a) | Test output |
| D4-A30 | Every cash-out entry that takes روکڑ below 0 shows the note and, on save, the confirm box; Yes saves, No saves nothing, and the backend refuses it without the confirmation (FR-6.7). Example: روکڑ 20,000, پیشگی 25,000 cash shows −5,000; a return-by date before the entry date is refused and the same day saves (FR-16.9) | Test output |
| D4-A31 | A per-lot seller commission of 500 sits on line 1 of 2; an edit removes line 1; line 2 now carries 500 with the stored method and value, and the lot's seller commission includes 500 once (FR-11.9). An opening on اپنا مال is refused. A drawings opening of 2,000 for الف, with nothing else posted, gives خالص −2,000 and کل رقم −2,000 (FR-10.9, C20, C22) | Test output |

## 8. Not in D4

- Dashboard, attention bell, reports (R2 onwards), analytics, month close, audit viewer: D5.
- Cloud backup started by day close: D5.
