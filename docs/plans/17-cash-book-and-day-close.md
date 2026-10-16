# Plan 17: روکڑ, expenses, partner money, capital, الگ کھاتہ, جنرل and the full day

Deliverable: D4, plan file 17. Branch: `feature/bikri` (goal run D3+D4). Version 1.0, 15-10-2026.

## 1. Goal

All money that moves outside sales is recorded and shown: the cash book and cash count, one خرچہ screen (shop expense, a partner's personal spending and its return), ادھار رقم for staff and others, partner capital in and out with the partner total (کل رقم) and its dated record, الگ کھاتہ kept apart from the partners, and the جنرل entry. Day close shows the full summary. After this plan the whole PRD worked day (Section 4) runs and every golden test of D4 passes.

## 2. Requirements

PRD 16 (16.1 to 16.6, FR-16.1 to FR-16.9), FR-7.67, FR-7.68 (the partner-name note), FR-7.44 (overdue loans and partner takes, the data), FR-7.62, FR-6.5, FR-6.7, 17.3 and 17.4 (FR-17.7 to FR-17.9, C20, C22), FR-10.9 (v1.5: a drawings opening debit counts as money taken), FR-16.7, FR-21.1 (full day summary), 4.6 to 4.9. NFR-REL-8, NFR-URD-29, NFR-SEC-5. Architecture 6, 10, 15.7 and 18; the money agent Sections 5, 8.2b and 9. The migration 0005 of plan 16b supplies the new voucher and posting types and the accounts. E2E-8, E2E-9, E2E-10, E2E-12, E2E-20, E2E-24 (backend part), E2E-29, E2E-30, E2E-30b, E2E-35, E2E-37, E2E-38, E2E-39 (preview numbers). Acceptance: D4-A14 to D4-A17, D4-A20 to D4-A22, D4-A25 to D4-A28, D4-A30 (the other cash-out types and the return-by date), D4-A31 (the drawings opening).

## 3. Worked numbers (the tests use exactly these)

| Test | Working | Expected |
|---|---|---|
| E2E-8 cash after Events 1 to 5 | 50,000 − 5,000 + 12,800 + 10,000 − 4,316 − 3,000 − 2,000 | 58,484 |
| E2E-9 partner shares of the day | income 3,948 + 1,540; expense 3,000; drawings 2,000 | income 2,196 / 1,646 / 1,646; expense −1,200 / −900 / −900; profit 996 / 746 / 746; الف drawings 2,000 |
| E2E-10 trial balance | بنام 58,484 + 2,880 + 18,684 + 3,000 + 2,000; جمع 10,160 + 2,484 + 2,484 + 480 + 40 + 300 + 4,000 + 100 + 65,000 | 85,048 = 85,048 |
| E2E-12 | everything sold | باقی مال empty |
| E2E-20 | expense 3,384 at 40 / 30 / 30 | −1,354 / −1,015 / −1,015 (sum −3,384) |
| E2E-39 preview (دکان خرچہ بجلی 3,000) | C11 | الف 1,200, ب 900, ج 900 before save; then 2,000 as ذاتی خرچہ for الف: his کل رقم 2,196 − 1,200 = 996 now, 996 − 2,000 = −1,004 after; ب and ج not affected; after both saves cash 58,484 and the E2E-9 numbers |
| E2E-29 | 11-10-26 الف returns 2,000 (ذاتی خرچہ واپسی) | on 10-10-26 خالص 996 − 2,000 = −1,004; after the return net taken 0, خالص 996, cash 60,484; the take shows fully returned |
| E2E-30 اسلم | 10-10-26 lent 5,000 cash (reason گھریلو ضرورت, return by 15-10-26); 11-10-26 returns 3,000 cash; 12-10-26 2,000 from salary | 10-10: cash 53,484, open 5,000. 11-10: open 2,000, cash 56,484. 12-10: open 0, cash 56,484; تنخواہ 2,000 split −800 / −600 / −600; R32 data: lent 5,000, returned 5,000 (cash 3,000, salary 2,000), open 0 |
| E2E-30b | as E2E-30 to 11-10-26, no salary deduction, "today" 16-10-26 | the attention data lists اسلم 2,000 past its return-by date 15-10-26 (1 day); the R32 data shows him overdue |
| E2E-35 | 11-10-26 ب invests 50,000 cash (سرمایہ جمع) | cash 108,484; ب capital 50,000 جمع; his profit share still 746; trial balance 135,048 = 135,048; never split |
| E2E-37 | after Events 1 to 5 and E2E-35 | کل رقم الف 2,196 − 1,200 − 2,000 = −1,004; ب 50,000 + 1,646 − 900 = 50,746; ج 1,646 − 900 = 746; Σ 50,488 + ابتدائی سرمایہ 65,000 = 115,488 = equity in the trial balance (65,000 + 50,000 + 5,488 − 3,000 − 2,000). ب's statement in date order: 10-10-26 بکری 1 share 1,184 (running 1,184); بکری 2 share 462 (1,646); دکان خرچہ بجلی −900 (746); 11-10-26 سرمایہ جمع 50,000 (50,746) |
| E2E-38 (separate run) | opening position; الگ کھاتہ باغ; 10-10-26 pays out 10,000 cash (reason باغ کی مزدوری); 11-10-26 gets 4,000 back; then a lot with seller باغ sells 10 پیٹی کیلا to ارشد at 900 ادھاری | after paying: باغ −10,000, cash 40,000; after 4,000 back −6,000, cash 44,000; the sale: value 9,000, no seller-side charges, buyer comm 540, لاگا 80, ارشد pays 9,620, entry 9,620 = 9,000 (باغ) + 540 + 80, باغ +3,000; shared income 620 → 248 / 186 / 186; nothing of the 9,000 reaches any partner |
| E2E-24 (backend part) | after Events 1 to 5, buyer field `1` | 03040001 اکرم ٹول ٹیکس; card 2,880 بنام, oldest open bill 10-10-26 (بکری 1), 0 days; "پورا بقایا" fills 2,880. In a جنرل line `1` lists both 03040001 اکرم ٹول ٹیکس and 030110001 رحیم with their types |
| D4-A30 | return-by date | money lent on 15-10-26 with return by 10-10-26 is refused (`ERR_RETURN_BY_BEFORE_ENTRY`); 15-10-26 or later saves |
| D4-A31 | drawings opening | a drawings opening debit of 2,000 for الف with nothing else posted: خالص −2,000 and کل رقم −2,000 |
| Day close (PRD 4.9) | 10-10-26 | 2 بکری; خام 41,400; commission 5,448; shared income 5,488; partner shares 996 / 746 / 746; cash 58,484; trial difference 0; باقی مال 0 |

## 4. Contract

New error codes (`errors.ftl` in `ur` and `en`, `# review: pending`): `ERR_RETURN_BY_BEFORE_ENTRY` (message "واپسی کی تاریخ اندراج کی تاریخ سے پہلے نہیں ہو سکتی" / "The return-by date cannot be before the entry date"), `ERR_PARTNER_REQUIRED`, `ERR_EXPENSE_HEAD_REQUIRED`, `ERR_LENT_PERSON_REQUIRED`, `ERR_KHATA_REQUIRED`, `ERR_GENERAL_NOT_BALANCED` (values `dr`, `cr`; the entry is not saved), `ERR_GENERAL_TOO_FEW_LINES`. Reused: `ERR_RETURN_EXCEEDS_OPEN` (plan 15), `ERR_REASON_REQUIRED`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`, date codes.

### Backend (commands in `app/src/commands/vouchers.rs`, `cashbook.rs`, `partners.rs`, `day.rs`; all need sign-in)

Each type keeps its own service function, posting type, number series and checks (FR-7.67, B3); preview and save call one domain function (B5, FR-16.8). Every save that can take روکڑ below 0 takes `confirm_cash_below_zero` and uses the shared check of plan 12 (FR-6.7).

- **Cash book** (16.1): `cashbook_get { from, to }` -> opening cash, every cash in and out from all modules (date, source, in, out, running), closing; `cashbook_count { counted }` -> the difference from the book; posts nothing (FR-16.1).
- **خرچہ** (16.2, 16.3): `vouchers_save_shop_expense { date, expense_head_id, amount, mode, bank_account_id?, paid_to?, narration?, confirm }`: head بنام, cash or bank جمع, split by the shares of the date through the plan 11 hook (E2E-20). `vouchers_save_personal_expense { date, partner_id, amount, mode, reason, return_by?, taken_by?, narration?, confirm }`: the partner's drawings account بنام, cash or bank جمع, never split. `vouchers_save_personal_return { date, partner_id, amount, mode, narration? }`: the opposite; a return above his open amount is `ERR_RETURN_EXCEEDS_OPEN`. Return-by before the entry date is `ERR_RETURN_BY_BEFORE_ENTRY` (FR-16.9). `vouchers_preview_expense { type, input }` -> for دکان خرچہ each partner's share (C11); for ذاتی خرچہ and واپسی the chosen partner's کل رقم now and after, the others "not affected" (FR-16.8); cash note data. `partner_takes { partner_id }` -> his money taken: date, amount, reason, taken by, return by, returned, still open (oldest first, read time, FR-16.2). `vouchers_repeat_last_month_preview { month }` -> this month's regular expenses proposed from last month for the user to check and save (FR-7.62); `masters_expense_heads` returns most-used heads first.
- **ادھار رقم** (16.5): `vouchers_save_money_lent { date, person_account_id, amount, mode, bank_account_id?, reason, return_by?, narration?, confirm }`: person بنام, cash or bank جمع; `vouchers_save_money_lent_return { date, person_account_id, amount, mode: cash|bank|salary, bank_account_id?, salary_head_id?, narration? }`: cash or bank بنام, person جمع, or with salary: the salary head (default تنخواہ) بنام, person جمع, no cash moves, split as a shop expense (FR-16.4). A return above the open amount is `ERR_RETURN_EXCEEDS_OPEN`. `lent_person_create { name_ur, name_en, phone?, staff_id? }` makes the person's account under control 0103 (the `F4` of the screen). `lent_list { person_id }` -> his loans: date, amount, reason, return by, returned, still open, days past return-by (oldest first at read time, FR-16.5). `lent_report { as_of }` (R32 data: person, staff or not, lent, returned by cash and by salary, open, oldest open date, return-by, days past, totals). `assist_attention_overdue()` -> overdue loans and partner takes past their return-by date (the data of FR-7.44 for these two; D5 builds the bell).
- **سرمایہ** (17.4): `vouchers_save_capital_in { date, partner_id, amount, mode, bank_account_id?, narration? }` (cash or bank بنام, his capital account 0602 جمع) and `vouchers_save_capital_out { ..., confirm }` (the opposite). Never split, never income. Taking back more than his capital balance gives a warning in the preview and still saves (FR-17.8). `partners_overview` gains returned, خالص (C20: profit share − net taken), capital invested and کل رقم (C22 = opening capital + invested − taken back + Σ income shares − Σ expense shares − taken + returned). A drawings opening debit (FR-10.9) counts as money taken, dated its opening date. `partners_statement { partner_id, from, to }` -> every dated movement with source and the running کل رقم (FR-17.9, the data of R31): income shares with their source, expense shares, capital in and out, money taken with reason and returned, and month close as a note that moves profit into capital without changing the total. Every line comes from posted records.
- **الگ کھاتہ** (16.6): `khata_create { name_ur, name_en, note? }` (account under control 0305, `F4`), `vouchers_save_khata_out { date, khata_account_id, amount, mode, reason, narration?, confirm }` (khata بنام, cash or bank جمع: minus) and `vouchers_save_khata_in` (cash or bank بنام, khata جمع: plus), `khata_statement { khata_account_id, from, to }` -> entries in date order with reason, amount and running signed balance. Never income or expense, never split, never in any partner's total. A الگ کھاتہ can be a lot's seller in بکری (FR-16.7): the lot seller picker offers it; its seller credit goes to the khata; every seller-side default is 0 for such a lot (a rule on that khata can set them, in `domain::rules`); the buyer side is charged as usual. The صافی part of the credit follows C8.
- **جنرل** (16.4): `general_preview { lines }` and `general_save { date, narration?, lines: [{ account_id, person_key?, dr, cr, narration? }], confirm }`: saves only when Σ بنام = Σ جمع (`ERR_GENERAL_NOT_BALANCED`, nothing written); at least two lines; each line has one side; lines on income or shop expense accounts are split by the hook (FR-6.5); lines on buyer accounts take part in bill matching (FR-15.4); a line on a seller account never adjusts advances (a debit settles his payables; a credit is a payable posting); lines on اپنا مال are refused; a cash credit uses the cash confirm.
- **Day close** (FR-21.1): `day_close_preview` gains, for the day to close, the بکری count, خام بکری, commission (seller + buyer + لاگا), shared income, each partner's share, cash in hand, the trial balance difference (must be 0) and the lots in باقی مال; `day_close` is unchanged. The cloud backup it starts is D5.
- **Search/assist**: `assist.rs` gets the backend part of E2E-24.
- Tests: `golden_day.rs` gains Event 5 (with its reason) and the whole day, E2E-8, E2E-9, E2E-10, E2E-12, E2E-20, E2E-29, E2E-30, E2E-30b (clock set to 16-10-26), E2E-35, E2E-37, E2E-38 and the preview numbers of E2E-39, with the numbers above and no expected number changed. Property tests (Code Standards 9.3): Σ partner کل رقم + ابتدائی سرمایہ 06010001 = equity in the trial balance (random postings and capital moves; when ابتدائی فرق is not 0 the check includes it, so random openings must balance or be included); lent and take returns never make an open amount negative and open amounts equal the account balance; month close leaves کل رقم unchanged; a الگ کھاتہ never reaches a partner row. Integration tests: cash confirm on each cash-out type, return-by rule, type switching keeps the saved type and posting matching what is shown, opening on اپنا مال refused, drawings opening counted, sign-in refusal.
- Money agent before and after (Sections 5, 8.2b, 9).

### Frontend (features `cashbook`, `vouchers`, `partners`)

- The روکڑ area (main nav): the روکڑ بک with the cash count box (shows the difference, posts nothing); the one خرچہ screen (`Alt+K` opens it as دکان خرچہ) with the type list دکان خرچہ, ذاتی خرچہ, ذاتی خرچہ واپسی, only the fields of the chosen type in the same places, the split or the partner total shown before save from `vouchers_preview_expense`, the day's expenses or the partner's money-taken list below the form, most-used heads first, "repeat last month", and the FR-7.68 note "یہ شراکت دار کا ذاتی خرچہ تو نہیں؟" with a button ذاتی خرچہ بنائیں when "ادا کیا" matches a partner's name in either script (never automatic); changing the type keeps date, amount, mode and narration and clears the rest (F44, E56). The ادھار رقم screen (lent and returned on one screen; `F4` adds a person; the person's loan list below the form). The الگ کھاتہ screen (paid out and received on one screen; signed running balance list). The جنرل grid (with the `entry-grid` skill; totals from the backend; save disabled until balanced). A link from the روکڑ area to the voucher screen (FR-15.10).
- The شراکت دار area: the سرمایہ screen (invested and taken back on one screen with the partner's capital history and balance); the overview with returned, خالص, capital invested and کل رقم; the partner statement screen with the running کل رقم.
- The day close screen shows the full summary. Every cash-out screen uses the amber note and `CashConfirmDialog` of plan 12.
- Keyboard only, both languages, 1024 × 600 with no sideways scroll, four states, language switch in the middle of any entry loses nothing. Fluent `cashbook.ftl`, `partners.ftl` and `vouchers.ftl` additions and errors, all `# review: pending`; each Urdu message says exactly what the English says (NFR-URD-29). Design fallback as in plan 12.
- Windows specs written and syntax-checked: the whole worked day by keyboard, E2E-39 (run LOCAL-ONLY).

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, stubs, error codes, `pnpm gen:types` | backend | plan 16b built |
| T2a | Cash book, خرچہ (three types), partner takes, repeat last month, ادھار رقم, return-by rule, tests | backend | T1 |
| T2b | Capital, partner overview extras and statement, الگ کھاتہ and its lot seller, جنرل, tests | backend | T2a |
| T2c | Day close summary, attention data, the whole-day golden tests, property tests, money agent | backend | T2b |
| T3 | All screens against the stubs; Fluent | frontend | T1 |
| T4 | Wire to the real commands in the order T2a, T2b, T2c; keyboard run of the whole day; screenshots | frontend | T2a to T2c, T3 |

## 6. Acceptance checklist

1. The whole worked day passes in `golden_day.rs`: E2E-8, E2E-9, E2E-10 (85,048 on both sides), E2E-12 (D4-A14, D4-A16).
2. E2E-20 passes; ذاتی خرچہ is never split; an unbalanced جنرل cannot save (D4-A15).
3. The day close summary shows the PRD 4.9 values; the cash count box shows the difference and posts nothing (D4-A17).
4. E2E-29, E2E-30, E2E-30b pass; the partner overview shows taken, returned and خالص (D4-A20, D4-A21).
5. The خرچہ, voucher and ادھار رقم screens work by keyboard in both languages with the list below the form (D4-A22; Windows specs LOCAL-ONLY).
6. E2E-35 and E2E-37 pass with the property test (D4-A25, D4-A26); E2E-38 passes (D4-A27); E2E-39 preview passes and type switching keeps the shared fields (D4-A28).
7. Cash confirm on every cash-out type; the return-by date rule; the drawings opening gives −2,000 and −2,000 (D4-A30, D4-A31).
8. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` exit 0; every earlier test still passes.

## 7. Out of scope

The dashboard tiles, the attention bell and list screen, R-reports (R29 to R34 data queries beyond those named), analytics, month close and the audit viewer (D5). Month close is only checked here for leaving کل رقم unchanged if D5's service is not built yet: that check moves to D5 then (note it in the hand-off).

## 8. Open questions

None for Umar. Team defaults: the English voucher letters for the new types are chosen by `backend` and recorded in the plan hand-off; the FR-7.68 name match uses normalised names.
