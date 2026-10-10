# D4. Money in and out: changes to sales, vouchers, expenses and the full day

Version 1.0, 10-10-2026. Deliverable 4 of 5. Read `README.md` in this folder first.
Depends on: D3 done. Plan files: `15-vouchers`, `16-later-sales-edit-cancel`, `17-cash-book-and-day-close`.
Build order step: CLAUDE.md MVP step 6. At the end, the whole PRD worked day (Section 4) runs, and every golden test passes except E2E-11 (analytics) and E2E-18 (backup), which come in D5.

## Decisions

None open. If a question comes up, follow README Section 8.

## 1. What Umar can do at the end

The munshi runs the whole of 10-10-26 from PRD Section 4, by keyboard:

1. بکری 1 and بکری 2 (from D3).
2. وصولی: اکرم pays 10,000. His card showed 12,880 and his open bill from بکری 1; after saving, he owes 2,880.
3. ادائیگی: رحیم's card shows 4,316 payable, and that his 15,000 advance was already adjusted when بکری 1 was saved (D3). "پورا بقایا" fills 4,316. His balance becomes 0.
4. دکان خرچہ: electricity 3,000, split 1,200 / 900 / 900. ذاتی خرچہ: الف takes 2,000.
5. The cash book shows 58,484. The munshi counts the cash and types it in the cash count box.
6. Day close shows the day summary: 2 بکری, خام بکری 41,400, the commission, the partner shares, cash 58,484, trial balance difference 0, nothing left in باقی مال. He confirms.

He can also correct a mistake: edit اکرم's rate from 600 to 610, and every number follows; edit it back, and every number returns exactly. He can cancel بکری 2, and everything it did is reversed. If a lot is not fully sold, its remaining goods sit in باقی مال until a later sale on another day.

## 2. Plan files

### 15-vouchers: وصولی, ادائیگی and the rest of پیشگی

**Requirements:** PRD 15.1 to 15.4 (FR-15.1 to FR-15.7, FR-15.9, FR-15.10), 13 (FR-13.7, FR-13.8), R26 data (the advances query), FR-7.38 (oldest open bill and payables on the party card), FR-7.58, FR-7.59, FR-7.61, FR-5.4 (voucher numbers in global search); NFR-SPD-9, NFR-SPD-21; Architecture 11; the money-calculations agent. E2E-6, E2E-7, E2E-21, E2E-27.

**backend builds:**

- وصولی / اگراہی: cash or bank بنام, buyer جمع, with the collector field. It uses the bill matching engine built in D3 (FR-15.4): oldest bills first, by account or by پختہ person key. A payment above what is owed stays as his credit and settles his next bills by itself (FR-15.5).
- اگراہی فہرست: today's planned collections by collector and city, printable (FR-15.6), and the collection round that opens one receipt after another from it (FR-7.61).
- ادائیگی: account بنام, cash or bank جمع. A payment above the payable warns but can save; the extra shows in "بیوپاری کے ذمے" (FR-15.7). Sellers whose advances are more than their sales are in that list too (FR-15.9).
- Receipts from one-time buyers and payments to one-time sellers through their person key (FR-13.7). "Make regular" creates an account and moves the open balance with a جنرل entry (FR-13.8).
- Edit (reversal + new posting) and cancel with reason for all three voucher types, including the پیشگی of D2 (FR-15.3). Bank and cheque modes with the bank account and cheque details (15.1).
- The party card gains the oldest open bill with its age, and the payable postings with the advance already adjusted against them (FR-7.38, FR-7.58, FR-7.59). "پورا بقایا" fills the full open amount. Advances are adjusted only at بکری save (FR-15.8, built in D3); ادائیگی never adjusts them again.
- The advances query (given, adjusted, open per seller) that E2E-7 checks and that D5 shows as report R26.
- The receipt slip print data with the amount in words (FR-15.2), with the `print-and-export` skill.
- `golden_day.rs`: Events 3 and 4, E2E-6, E2E-7, E2E-21. `assist.rs`: the backend part of E2E-27.

**frontend builds:** the وصولی and ادائیگی list screens (entered one after another, with a running total; FR-15.1) in the بکری side menu, next to the پیشگی screen from D2. (Plan file 17 links all three from the روکڑ area, as the same screens; FR-15.10.) the open bills and payables panels; "پورا بقایا"; the اگراہی فہرست and the collection round; "make regular"; the receipt slip template.

**Done when:** E2E-6, E2E-7, E2E-21 and E2E-27 pass.

### 16-later-sales-edit-cancel: باقی مال, later sales, edit, cancel and undo

**Requirements:** PRD 12.8 (FR-12.14 to FR-12.16), 12.9 (FR-12.17 to FR-12.22), FR-12.11, FR-7.47, FR-6.3, FR-6.6; NFR-SEC-5; Architecture 12; the money-calculations agent. E2E-13, E2E-13b, E2E-14, E2E-15, E2E-16, E2E-16b.

**backend builds:**

- باقی مال: every lot with balance qty above 0 (FR-12.14); "sell" opens a later sale (FR-12.15); write-off with a reason records a stock movement (FR-12.16).
- Later sale: new sale lines on an existing lot, dated the day it happens, as its own posting (FR-12.17). No lot charges a second time (see E2E-16b).
- Edit: any one posting of a بکری, while its date is open (FR-12.18). Saved as a reversal of that posting plus a new posting, both on the same date (FR-12.19). Unchanged charges keep their stored method, value and amount (FR-12.20). The partner split is recalculated from the new shared income (FR-12.21).
- Cancel: reverses every posting of the بکری; every date must be open; a reason is required (FR-12.22, FR-12.11). Receipts matched to its lines become unmatched credit and are matched again to the buyer's next oldest bills (FR-12.22). An edit re-runs the matching the same way.
- Analytics facts are reversed with their postings (FR-18.7). For E2E-15, the check "analytics excludes سیب" is made on the analytics fact rows (سیب nets to zero). D5 checks the view itself.
- Undo straight after saving: the same reversal path as cancel, offered in the success message (FR-7.47).
- Audit rows for every edit, cancel and write-off.
- `golden_day.rs`: E2E-13, E2E-13b, E2E-14, E2E-15, E2E-16, E2E-16b.

**frontend builds:** the باقی مال screen with "sell" and write-off; the later sale form; edit mode on a saved بکری (what changed is visible); cancel with reason (confirm dialog, FR-7.8); the undo action in the success message.

**Worked checks** (PRD 27.2; the tests use exactly these):

- E2E-13: اکرم's rate 600 → 610. His line 20 × 610 = 12,200; buyer comm 12,200 × 6 ÷ 100 = 732; he pays 12,200 + 732 + 160 = 13,092; خام بکری 29,600. The rest as PRD 27.2.
- E2E-14: back to 600; every number returns exactly to E2E-1 to E2E-4.
- E2E-16: only 45 of 50 sold; خام 26,500; seller comm 720 + 870 = 1,590; the rest as PRD 27.2. E2E-16b: the 5 نگ sold to ارشد at 560 on 11-10-26; value 2,800; seller comm 168; buyer comm 168; لاگا 5 × 8 = 40; ارشد pays 3,008; seller credit 2,800 − 168 = 2,632.

**Done when:** the six golden tests above pass with the PRD numbers.

### 17-cash-book-and-day-close: روکڑ, expenses, جنرل and the full day

**Requirements:** PRD 16 (16.1 to 16.4, FR-16.1), FR-7.62, FR-6.5, FR-21.1 (full day summary), 4.6 to 4.9; NFR-REL-8; Architecture 6 and 10; the money-calculations agent. E2E-8, E2E-9, E2E-10, E2E-12, E2E-20, E2E-24.

**backend builds:**

- روکڑ بک: opening cash, every cash in and out from all modules, closing cash, for a date range (16.1). Cash count box: shows the difference from the book; posts nothing (FR-16.1).
- دکان خرچہ: expense head بنام, cash or bank جمع, split between partners by their shares on that date (16.2, FR-6.5). Most-used heads first, and "repeat last month's expenses" (FR-7.62).
- ذاتی خرچہ: that partner's drawings بنام, cash or bank جمع; never split (16.3).
- جنرل: lines of account, بنام, جمع, narration (and person for پختہ lines); saves only when the totals match; lines on income or shop expense accounts are split (16.4).
- The full day close summary of FR-21.1: بکری count, خام بکری, commission, partner shares, cash in hand, trial balance difference, lots with باقی مال.
- `golden_day.rs`: Event 5 and the whole day, E2E-8, E2E-9, E2E-10, E2E-12, E2E-20. `assist.rs`: the backend part of E2E-24.

**frontend builds:** the روکڑ area with روکڑ بک and the cash count box, دکان خرچہ, ذاتی خرچہ, and the جنرل grid (with the `entry-grid` skill), plus links to the وصولی, ادائیگی and پیشگی screens (FR-15.10); the full day close summary.

**Worked checks** (PRD 4.9 and 27.2):

- E2E-8: cash 50,000 − 5,000 + 12,800 + 10,000 − 4,316 − 3,000 − 2,000 = 58,484.
- E2E-9: income shares 2,196 / 1,646 / 1,646; expense shares −1,200 / −900 / −900; profit 996 / 746 / 746; الف drawings 2,000.
- E2E-10: the trial balance table of PRD 27.2. بنام 58,484 + 2,880 + 18,684 + 3,000 + 2,000 = 85,048. جمع 10,160 + 2,484 + 2,484 + 480 + 40 + 300 + 4,000 + 100 + 65,000 = 85,048.
- E2E-20: an expense of 3,384 splits −1,354 / −1,015 / −1,015 (sum −3,384).

**Done when:** the whole worked day passes in `golden_day.rs`, and the day close summary shows the PRD 4.9 values.

## 3. Design for D4

The lead runs `/design` inside `docs/design/design-system.md` for:

1. The وصولی screen with the buyer's open bills and "پورا بقایا", in both languages.
2. The ادائیگی screen with payable postings and the "more than due" warning.
3. The باقی مال list and the later sale form.
4. A saved بکری in edit mode, showing what changed, and the cancel dialog.
5. The روکڑ بک with the cash count box; دکان خرچہ; ذاتی خرچہ; the جنرل grid.
6. The full day close summary.
7. The receipt slip (A5 and 80 mm).

## 4. Team flow for D4

- Plan files run in order: 15, 16, 17 (README Section 4).
- 15 (vouchers) before 16 (edit and cancel), because a cancel must turn matched receipts back into credit (FR-12.22), and the attack list mixes both.
- 17 last: E2E-10 and E2E-24 need everything before it.
- `backend` uses the money-calculations agent for all three.

## 5. Review focus for D4

- Reversals: every edit and cancel is a reversal plus a new posting. No posted row is ever changed (CLAUDE.md rule 4). Show the rows before and after an edit.
- Matching: oldest first, against FR-15.4; the sum of a buyer's open bills always equals his balance.
- Partner shares on every income and expense posting, never on payables, never on ذاتی خرچہ.
- Golden tests: every number in PRD 27.2 for E2E-1 to E2E-10, E2E-12 to E2E-16b, E2E-13b and E2E-19 to E2E-21, unchanged. (E2E-11 is analytics, checked in D5.)

## 6. Robustness attack list for D4

1. **Random days.** Generate thousands of random days: sales, later sales, edits, edits of edits, cancels, receipts (part, full, more than owed), payments, advances, expenses, جنرل entries. After every step: the trial balance difference is 0; each buyer's open bills add up to his balance; each split adds up to its shared income; cash in the cash book equals the روکڑ account.
2. **Closed dates.** Edit, cancel, voucher and expense on a closed date are all refused; after an Owner reopen they work; the audit log has every step.
3. **Order of events.** Cancel a بکری that has a later sale; edit a posting after a receipt has matched its bill (the bill matching recalculates; nothing is lost); pay a seller twice.
4. **Power cut** during an edit (the reversal and the new posting are saved together or not at all).
5. **Speed.** Save any voucher under NFR-SPD-9; party card with 10 years of open bills under NFR-SPD-21; matching against a buyer with 10,000 open bills.
6. **Bad inputs.** Amount 0, negative, decimal; cheque mode with no cheque number; a جنرل entry that does not balance; a ذاتی خرچہ with no partner.

## 7. Acceptance (D4-A1 to D4-A18)

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

## 8. Not in D4

- Dashboard, attention bell, reports (R2 onwards), analytics, month close, audit viewer: D5.
- Cloud backup started by day close: D5.
