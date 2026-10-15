# D3. بکری: sale entry, journal, پختہ کھاتہ, partners and prints

Version 1.3, 15-10-2026 (PRD v1.5: commission typed on the line by default, cash warning; PRD v1.3: opening capital per partner; v1.2: advance tick, one user). Deliverable 3 of 5. Read `README.md` in this folder first.
Depends on: D2 done. Plan files: `11-partners`, `12-bikri-entry`, `13-bikri-save`, `14-bikri-prints`.
Build order step: CLAUDE.md MVP step 5. Golden tests E2E-1 to E2E-5 pass.

This is the most important deliverable. The بکری screen is where the munshi spends his day, and every rupee of commission starts here.

## Decisions

| ID | Question | Recommended | Umar's answer |
|---|---|---|---|
| D3-Q1 | Which cells of a sale line does `Enter` stop at? | Buyer, qty and rate only, as the FR-7.14 example and NFR-USE-6 say (three fields, three `Enter`s). The payment type toggle and the note are reached with the arrow keys or the mouse. | (waiting) |

This decision does not stop the work. Until Umar answers, plan file 12 builds the FR-7.14 example order (buyer, qty, rate) and the lead lists D3-Q1 under open questions in `status.md`. If Umar later chooses differently, it is a small follow-up change to the grid's column setup.

## 1. What Umar can do at the end

The munshi opens بکری بک. He types truck `LES-1234`, اڈا سوات اڈا and زمیندار رحیم. In the lot he types seller `1`, کیلا, پیٹی, 50 نگ, freight 3,000, مزدوری 5 per نگ, منشیانہ 50, فون 20, نقد 5,000. (From the second بکری on, the app remembers: the same truck fills its اڈا and زمیندار and warns that it already has a بکری today, and seller `1` fills کیلا and پیٹی from رحیم's last lot. That is E2E-25.)

In the sale lines he types buyer `1`, qty 20, rate 600. اکرم's card shows. The next line's qty starts at 30 (what is left), and the rate field shows a grey hint of 600. He types buyer `2`, rate 580. All the time, the live summary shows خام بکری 29,400, کل اخراجات 10,084, صافی بکری 19,316, and the live partner split 1,580 / 1,184 / 1,184. He presses ⓘ on اکرم's commission: "12,000 × 6 ÷ 100 = 720".

Halfway, he presses `Alt+L`. Everything turns to English; nothing typed is lost. He presses `Ctrl+Enter`: the بکری saves in one step, the seller bill and buyer slips print, and a new empty بکری opens. The journal shows one balanced entry of 36,564. رحیم's 15,000 advance is adjusted.

Then بکری 2: a one-time buyer, شاہد, buys 10 پیٹی سیب at 1,200 through پختہ کھاتہ. He pays 12,800 cash, and the پختہ کھاتہ register shows him as paid.

If the power goes off in the middle of typing, the app offers the draft when it starts again.

## 2. Plan files

### 11-partners: partner setup and the live split

**Requirements:** PRD 17.1 to 17.3 (FR-17.1 to FR-17.5), FR-6.5, FR-2.4, 4.2 (partners الف 40, ب 30, ج 30); NFR-INS-2 (the wizard's partners step), NFR-SCL-5, NFR-SEC-5; Architecture 10; the money-calculations agent (C10, C11).

**backend builds:**

- Partners with the fields of 17.1: names in both scripts, capital and drawings accounts created by themselves, opening capital (posted as the opening of his capital account, FR-10.9), share with up to 2 decimals, effective date, active. The capital in and out screen comes in D4 (17.4).
- Shares of active partners total exactly 100% for every date (FR-17.1). Changes take effect from today or later, never back-dated; history kept; saved postings keep their split (FR-17.2).
- The split service: every posting that touches an income account (head 04) or a shop expense account (head 05) writes one share row per partner, per posting, with `split_by_bps` (FR-17.3 to FR-17.5, C10, C11). Payables are never split.
- The partner overview data of 17.3 (today, month, year, expense share, profit share, drawings, capital balance). The returned and خالص columns come in D4 with the واپسی entry (FR-16.3).
- Partners do not log in (FR-2.4).

**frontend builds:** partner setup with the running share total; the partners step in the first-run wizard; the partner overview.

**Done when:** a share change dated today saves, a back-dated one is refused, and the split of 3,948 gives 1,580 / 1,184 / 1,184.

### 12-bikri-entry: the بکری screen and the live summary

**Requirements:** PRD 12.1 to 12.5 (FR-12.1 to FR-12.7), 4.4 and 4.5, PRD 22, FR-17.4 (live split in the summary), FR-7.43, FR-7.48, FR-7.49 to FR-7.56; Settings "Checks" (credit limit, rate check); NFR-SPD-4, NFR-SPD-5, NFR-SPD-7, NFR-SPD-23, NFR-SCL-3, NFR-USE-1, NFR-USE-2, NFR-USE-6, NFR-USE-26, NFR-USE-27; Architecture 7 and 8; Code Standards 6.7 and 6.10. E2E-26. (E2E-25 needs saved بکری, so it is checked in 13.)

**The rule that matters most:** the live preview and the save call the **same** domain function (CLAUDE.md rule 3). The frontend sends what was typed, as text, and shows what comes back. It never adds, multiplies or rounds (rule 1).

**backend builds:**

- The بکری domain: header (12.2), lots (12.3), sale lines (12.4), charges from the rules of D2 with line overrides when allowed, every amount with its working record (Code Standards B30). PRD 22 formulas only, through `core::money`.
- `bikri_preview`: takes the typed draft, returns the full summary (12.5), every line's amounts, the seller summary, the live partner split, and all field errors together.
- Checks: sold qty never more than received (FR-12.6), negative net shown as a warning (FR-12.7), credit limit warn or block (FR-12.4), پختہ line needs name, and phone when credit (FR-12.5), rate check when its setting is on (FR-7.55).
- Assistance data: truck memory and same-truck-today (FR-7.49, FR-7.50), seller memory (FR-7.51), remaining qty (FR-7.52), the rate hint as placeholder only (FR-7.53), buyer memory (FR-7.56). Each under NFR-SPD-23 at its runner threshold. Memories read saved بکری, so their tests use fixture data here and the real flow in 13.
- v1.5 (Umar, 15-10-2026): seller commission, buyer commission and لاگا can be typed on a sale line by default (FR-11.10). The "allow override" setting starts on: today `overrides_from` in `services/src/rules.rs` reads `false` when the setting is missing, so that default changes to `true`. The preview also returns the cash warning of FR-6.7 when a lot's نقد would take روکڑ below 0.

**frontend builds:**

- The بکری بک screen inside the بکری side menu: header fields, the lots grid and the sale lines grid (with the `entry-grid` skill), the live summary, the live partner split, and `PartyCard` beside the active party field.
- `CalcInfo` (ⓘ) on every calculated amount, showing its working (FR-7.43).
- Copy from the line above with `Ctrl+D` (FR-7.54); simple sums like `20+5` in qty and money fields, sent as text for the backend to parse (FR-7.48).
- The preview call 50 ms after the last key press, cancelling the call before it (Code Standards F35).
- Everything on one screen at 1024 × 600, with no sideways scroll; less-used fields behind a "more" row (FR-12.2, FR-7.2).

**Done when:** typing Event 1 shows exactly the E2E-1 and E2E-2 numbers in the summary, and E2E-26 passes.

### 13-bikri-save: save, journal, پختہ کھاتہ, freight and drafts

**Requirements:** PRD 12.6 (FR-12.8 to FR-12.10, FR-12.13), 12.7, 13 (FR-13.1 to FR-13.6, FR-13.10), FR-7.29, 14 (FR-14.1, FR-14.2), FR-15.4 and FR-15.5 (the bill matching engine), FR-15.8 and FR-12.23 (advance adjusted by the seller credit, with the پیشگی سے کاٹیں tick on the lot), FR-5.4 (بکری numbers in global search), FR-6.1 to FR-6.6; NFR-SPD-8, NFR-REL-1, NFR-REL-2, NFR-REL-10; Architecture 7, 9 and 10; Code Standards 9.2. E2E-1 to E2E-5, E2E-17, E2E-25, E2E-28.

`backend` calls the money-calculations agent before writing the save and the journal lines, and again to review them.

**backend builds:**

- `bikri_save`: one action, all or nothing (FR-12.8). It writes everything FR-12.8 lists: the بکری, lots and lines with their stored values, the journal entry of FR-12.9 (sale part and cash part, each balanced), the partner share rows, the stock moves (received and sold qty per lot), the analytics facts that D5 reads (FR-18.6), the bill matching, the advance matching (FR-15.8), the پختہ کھاتہ lines, and the audit row. It runs the FR-12.8 checks first. Under NFR-SPD-8 for 30 lines at its runner threshold.
- The bill matching engine of FR-15.4 and FR-15.5, built here because the save needs it: every debit to a buyer (or a پختہ person key) is an open bill, and credits settle the oldest open bills first. In a بکری, a نقدی line is settled at once by its own cash. That is how E2E-5 shows شاہد as paid. D4's وصولی uses the same engine.
- پختہ کھاتہ: the two system accounts (FR-13.1); the person key, name + phone after Urdu normalisation (FR-13.5); the same rules as any party (FR-13.3); نقدی by default (FR-13.4); one-time sellers paid on the spot, so نقد starts at صافی بکری (FR-13.10); the register (FR-13.6).
- One-time people suggestions in پختہ name and phone fields (FR-7.29).
- Freight: typed per lot or split from the header total (C17), posting to کرایہ کھاتہ (FR-14.1).
- Drafts: auto-save every 5 seconds while typing; offered back after a crash or power cut (FR-12.10, NFR-REL-2).
- Today's بکری list (12.7), moving between saved بکری (FR-12.13), and بکری numbers in global search.
- `golden_day.rs`: E2E-1 to E2E-5 and E2E-28, after the D2 setup. `assist.rs`: the backend parts of E2E-25 and E2E-26.

**frontend builds:** save with `F10`, the draft offer at start, today's list, first / previous / next / last and find, the پختہ person fields with their suggestions, and the پختہ کھاتہ register screen.

**Worked numbers** (PRD 4.4 and 4.5; the tests use exactly these):

| Check | Expected |
|---|---|
| E2E-1 | خام 29,400; کل اخراجات 3,000 + 1,764 + 250 + 50 + 5,000 + 0 + 20 = 10,084; صافی 29,400 − 10,084 = 19,316 |
| E2E-2 | اکرم 12,000 + 720 + 160 = 12,880; ارشد 17,400 + 1,044 + 240 = 18,684 |
| E2E-3 | Sale part 31,564 = 31,564; whole entry with the نقد lines 36,564 = 36,564 |
| E2E-4 | Shared income 1,764 + 1,764 + 400 + 20 = 3,948; split 1,580 / 1,184 / 1,184 |
| E2E-5 | شاہد pays 12,800; seller صافی 10,160; whole entry 25,600 = 25,600; shared income 1,540; split 616 / 462 / 462; register shows شاہد, paid |
| Advance | رحیم's credit 24,316; نقد 5,000; advance adjusted 15,000; payable 24,316 − 5,000 − 15,000 = 4,316 (FR-15.8 example) |
| Advance tick off (E2E-28) | Separate run: same بکری with پیشگی سے کاٹیں unticked; credit 24,316, advance adjusted 0, advance open 15,000, ledger 4,316 جمع. The payable 19,316 and the ادائیگی part of E2E-28 are checked in D4. |

**Done when:** all the checks above pass in `golden_day.rs`, E2E-25 passes, and E2E-17 passes in the Windows suite.

### 14-bikri-prints: seller bill, buyer slips and the پختہ slip

**Requirements:** FR-12.12, FR-7.36, FR-7.57, FR-19.3 to FR-19.7, R1 and R5 (PRD 19.2), Settings "Printing"; NFR-URD-14 to NFR-URD-16, NFR-URD-28, NFR-SPD-14, NFR-SEC-13; Tech Stack 4.5. E2E-22.

Use the `print-and-export` skill for all of it.

**backend builds:** print data for the seller bill (R1), buyer slips with "print all" (R5, one slip per buyer, each on a new page), and the پختہ کھاتہ slip. Every value formatted by the backend, every label a Fluent ID, amounts in words in the print language. Printer, paper and copies from Settings. Print count raised on each print.

**frontend builds:** the shared print layout and the three templates (A4 bill; A5 or 80 mm slips), the print box with its language choice, the hidden print window, and `Ctrl+Enter` = save, print the ticked documents, open the next بکری (FR-7.57).

**Done when:** the بکری 1 seller bill shows 29,400, 10,084, 19,316 and "انیس ہزار تین سو سولہ روپے" (Urdu) or "Nineteen thousand three hundred sixteen rupees only" (English), and E2E-22 passes on Windows.

## 3. Design for D3

The lead runs `/design` inside `docs/design/design-system.md` for:

1. **The بکری بک screen**, filled with Event 1, in Urdu and English, at 1024 × 600 and 1920 × 1080. This is the screen to get right. It must show the header, lots, sale lines, live summary, live split and the party card at once, with one clear "save" action, and calm, scannable numbers.
2. The ⓘ working popover, the same-truck warning, the rate hint in an empty rate cell, the remaining-qty start value.
3. A sale line with پختہ کھاتہ and its name and phone fields.
4. The draft offer at start-up.
5. Partner setup with the running total, and the partner overview.
6. The پختہ کھاتہ register.
7. The seller bill (A4), a buyer slip (A5 and 80 mm) and the پختہ slip, in both languages.

Test the design against NFR-USE-1 and NFR-USE-2: a new user can enter a بکری from on-screen hints only; a trained munshi enters 10 lines in under 90 seconds by keyboard.

## 4. Team flow for D3

- Plan files run in order: 11, 12, 13, 14 (README Section 4).
- 11 first. The split must exist before a بکری can save.
- 12 builds the domain, the preview and the screen. 13 adds the save on top of the same domain function. Never a second copy of the calculation.
- 14 after 13, because prints read saved بکری.
- `backend` calls the money-calculations agent for 11, 12 and 13. `reviewer` calls it again to review.

## 5. Review focus for D3

- Every number on the screen comes from the backend. Search the frontend for any arithmetic on money or quantity values.
- Preview and save use one domain function. Show the call path in the review.
- The journal lines against FR-12.9 and Architecture 9, account by account.
- Golden tests unchanged and passing.
- The keyboard order of the grids against PRD 7.2, FR-7.14 and the D3-Q1 answer.
- The prints match the screen to the rupee.

## 6. Robustness attack list for D3

1. **Money properties.** For thousands of random بکری (random lots, lines, rates, charges and rules): the sale part balances; the whole entry balances; the split sum equals shared income; buyer totals add up to the sale part's بنام; preview and save give the same numbers.
2. **Bad inputs.** Qty sold above received (refused); qty × rate beyond the money limit (clear error, no overflow); rate 0; negative or decimal values; Urdu digits; a line with no buyer; a closed date; a پختہ credit line with no phone.
3. **Power cut.** Kill the app during save, many times: after restart, the بکری is either fully saved or not at all, and the books balance (NFR-REL-1). Kill it while typing: the draft returns, missing at most 5 seconds (NFR-REL-2). Full disk before save (NFR-REL-10).
4. **Speed.** 50 lots and 500 lines with no lag (NFR-SCL-3); summary update (NFR-SPD-7); save of 30 lines (NFR-SPD-8); language switch with 30 unsaved lines (NFR-SPD-19); 500 بکری and 5,000 lines in one day (NFR-SCL-2); print preview of 15 pages (NFR-SPD-14).
5. **Race.** Fast typing with slow preview answers: an old answer never overwrites a newer one.
6. **Sign-in.** Before sign-in or while locked, `bikri_save`, `bikri_preview` and the partner commands are refused and write nothing.

## 7. Acceptance (D3-A1 to D3-A20)

| ID | Item | Evidence expected |
|---|---|---|
| D3-A1 | Shares total 100% for every date; no back-dating; history kept (FR-17.1, FR-17.2) | Test output |
| D3-A2 | Wizard partners step on a clean install; partner overview | Windows E2E names |
| D3-A3 | The بکری screen fits at 1024 × 600 with no sideways scroll, in both languages | Screenshots |
| D3-A4 | Preview and save call one domain function; live summary update under NFR-SPD-7 | Code path in the review; timing |
| D3-A5 | E2E-1 and E2E-2 pass | golden_day output |
| D3-A6 | E2E-3 passes | golden_day output |
| D3-A7 | E2E-4 passes; the live split on screen shows the same numbers | golden_day output; screenshot |
| D3-A8 | E2E-5 passes, with the register showing شاہد, paid | golden_day output |
| D3-A9 | Advance adjustment matches the FR-15.8 example (adjusted 15,000, payable 4,316); with the tick off, E2E-28's بکری part (advance adjusted 0, advance open 15,000, ledger 4,316 جمع) | Test output |
| D3-A10 | E2E-25 and E2E-26 pass | assist.rs output and Windows E2E |
| D3-A11 | ⓘ shows the working on every calculated amount, for example "12,000 × 6 ÷ 100 = 720" | Component test and screenshot |
| D3-A12 | Save is all or nothing under repeated kills; books always balance | Robustness report |
| D3-A13 | E2E-17 passes on Windows (draft offered and restored) | Windows E2E result |
| D3-A14 | Freight per lot and split from the header total post to کرایہ کھاتہ (FR-14.1, C17) | Test output |
| D3-A15 | Seller bill, buyer slips (print all) and پختہ slip print in Urdu and English on their paper sizes; the bill shows 29,400, 10,084, 19,316 and the words | Print data snapshots; PDFs from the Windows run |
| D3-A16 | E2E-22 passes on Windows | Windows E2E result |
| D3-A17 | Speed: NFR-SPD-7, NFR-SPD-8, NFR-SPD-14, NFR-SPD-19, NFR-SPD-23 and NFR-SCL-3 met at their runner thresholds; a scripted 10-line بکری by keyboard in under 90 seconds in the E2E run (NFR-USE-2) | `pnpm perf` and E2E timings |
| D3-A18 | The full test suite, including every D1 and D2 test, passes on the final main; every D3 screen and print matches the design | Full test run; `windows-verify` report |
| D3-A19 | With no setting changed, seller commission, buyer commission and لاگا can be typed on a sale line (for example 4% instead of 6%); the value is saved with method دستی and shows in the working; with the setting turned off they are read-only (FR-11.10) | Test output; screenshot |
| D3-A20 | A lot's نقد that takes روکڑ below 0 shows "روکڑ صفر سے کم ہو جائے گی" in the live summary, and the save still works (FR-6.7) | Test output; screenshot |

## 8. Not in D3

- Later sales, باقی مال, edit, cancel and undo after save: D4.
- وصولی and ادائیگی, including receipts from one-time buyers (FR-13.7) and "make regular" (FR-13.8): D4.
- Month close (FR-17.6) and the partner statement report (R31): D5.
- Analytics views on the facts written here: D5.
