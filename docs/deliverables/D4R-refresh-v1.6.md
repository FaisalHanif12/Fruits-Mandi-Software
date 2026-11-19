# D4R. Refresh to PRD v1.6 and v1.7: one cash account, lists page by page, gardens, fruit taken, the tag and clear prints

Version 1.1, 22-10-2026 (PRD v1.7: plan 17e, the tag after a name and clear prints on the prints already built; 1.0, 19-10-2026: PRD v1.6). The file name keeps "v1.6" so every link to it still works. A refresh deliverable, run after the D3+D4 run and before D5. Read `README.md` in this folder first.
Depends on: the D3+D4 run done (plans 11 to 17 built, its DONE REPORT posted). Plan files: `17b-refresh-built-screens`, `17c-gardens`, `17d-fruit-taken`, `17e-tag-and-prints`.
Build order step: between CLAUDE.md MVP steps 6 and 7.

**Why this file exists.** PRD v1.6 (Umar, 18-10-2026 and 19-10-2026) changed some screens that are already built in D1 to D4 (plans 04 to 16). The files of built deliverables (D1, D2, D3, and D4 Sections 15 and 16) stay as they were built, so their history is true. This file holds every v1.6 change to built work, plus the two new v1.6 features that need D4 first (gardens and fruit taken), and every PRD v1.7 change (Umar, 20-10-2026 and 21-10-2026) to built work: the tag after a name and the clear print layout (plan 17e). Where this file and an earlier deliverable file differ, this file wins (README Section 0).

**No migration.** Migration 0006 (plan 16b, D4) already holds every v1.6 table, column, posting type and seeded account, and the v1.7 column `accounts.tag` (Architecture 1.6 and 1.7 change logs). The v1.7 firm contacts and trade mark are two keys in the `settings` table, which needs no schema change. If a plan here seems to need a schema change, stop and ask: it means 0006 missed something, and 0006 is fixed while it is still unreleased.

## Decisions

None open for D4R. A42 (کیلا lines with no rate) is still open with Umar and does not touch D4R. If a question comes up, follow README Section 8.

## 1. What Umar can do at the end

1. Every screen has one cash account, روکڑ (English "Cash"). No screen asks for a mode, a bank or a cheque (ادھار رقم واپسی keeps its choice of cash or salary deduction). The voucher screen asks only date, account, amount and narration (and the type's own fields).
2. Every long list (accounts, masters, آج کی بکری, باقی مال, پختہ کھاتہ register, voucher lists, rules, advances) opens at once with its first page and its totals over the whole filter.
3. He opens a garden, شاہ پور باغ: rent 50,000; الف 60% from the shop's cash, ب 40% from his own pocket, ج not in it. He pays installments and labour; الف's parts come off his total at once, ب's change nothing. 100 پیٹی کیلا from the garden are sold in a بکری, and 20 more in another shop. The garden screen shows its running balance every day. He closes it: الف gets 39,840 and ب 26,560, so الف is up 6,840 and ب gets his 22,000 back plus 4,560 (E2E-40). Each partner's statement shows the dated lines.
4. On the پھل tab next to خرچہ, he records 2 پیٹی کیلا that الف took home from رحیم's lot at 580 (الف's total −1,160; رحیم is owed 1,160), and 3 پیٹی سیب for the shop's guests from another agent's shop at 1,000 (1,000 off each partner) (E2E-42).
5. He opens buyer عاشق ظفر in انڈکس and types the tag K. Everywhere the name shows, on screen and on prints, it reads "عاشق ظفر K" (English "Ashiq Zafar K"). Typing "ashiq k" in any picker finds him. A second عاشق ظفر with no tag is a different person, and the duplicate check does not stop him.
6. In ترتیبات he types three contact persons with their phones and the trade mark "RH". Every print already built (seller bill, buyer slips, پختہ slip, receipt slip, اگراہی فہرست) now has the same clear header with the firm, the contacts and the trade mark, each party with its short code and name with tag, a totals row and "Page x of y". The buyer slips list each line the client's way: "(کیلا 20 . 600)".

## 2. Plan files

### 17b-refresh-built-screens: one cash account and lists page by page

**Requirements:** PRD 15.1 (no mode, bank or cheque), FR-7.67 (the shared fields are date, amount and narration), FR-10.4 (01010001 "Cash"; 01020001 never offered and nothing posts to it), 10.3 (no bank master), FR-10.9 (no opening on 01020001), FR-16.1, FR-19.11, E70; NFR-SPD-24; Code Standards B31, F34, F44; Architecture 15.4 and 15.7 (columns kept, never written).

**What is built today (v1.5):** the voucher screen of plan 15 has a mode (نقد، بینک، چیک) with bank account and cheque fields (`PaymentModeFields`, `vouchers_save_receipt`, `vouchers_save_payment`, `vouchers_save_advance` and their edit, cancel and batch paths, the receipt slip); the masters of plan 06 include the bank master; only `accounts_list` loads page by page.

**backend builds:**

- Voucher commands (receipt, payment, advance, the batch, edit, preview, cancel): no mode, bank or cheque in the inputs. Every voucher posts روکڑ. Old rows keep their columns and still show and print as saved. An input that names a bank or cheque is refused, never posted to 01020001.
- The bank master leaves the masters list and the command list. Its table stays (Architecture 15.4). Every account picker and search leaves out 01020001. An opening on 01020001 is refused (FR-10.9).
- Every list command built in plans 05 to 16 that does not page yet (for example `bikri_list`, `bikri_stock_list`, `masters_list`, `vouchers_list`, `pukhta_register`, `rules_list`, `advances_report`, `translit_pairs_list`) takes `offset` and `limit` and returns its totals over every matching row, in the same shape as `accounts_list` (FR-19.11, B31). A benchmark for NFR-SPD-24.
- Tests: the plan 15 tests rewritten for cash only; every golden number unchanged; a refusal test for a bank or cheque input; paging tests (totals over the whole filter equal the sum of every page).

**frontend builds:** the voucher screen without the mode, bank and cheque fields (`PaymentModeFields` removed); changing the type keeps date, amount and narration; the receipt slip without a mode line; the bank master gone from انڈکس; every list from plans 05 to 16 on the paged grid of `shared/grid` (F34), with the backend totals. Fluent messages no longer used are removed in both languages (`pnpm i18n:check`).

**Done when:** no screen or command has a mode, bank or cheque field; every list pages; the full suite and every golden test pass unchanged.

### 17c-gardens: باغات

**Requirements:** PRD 12.11 (FR-12.34 to FR-12.38), FR-12.29 and FR-12.30 (shares and sources, built in plan 16b), C23, C26, C22, FR-5.2 (باغات, main bar item 5), FR-7.67 (one garden screen), 9.1 (data of the باغات tile), FR-10.4 (0306), FR-10.9 (no opening on 0306), 17.3 (open gardens under each partner's card), FR-17.10 (garden lines), FR-6.7, FR-19.11, E61 to E66, E69, A54 to A57, A61 to A63; Architecture 6 and 15.5; the money agent Section 5.5. E2E-40, E2E-40b.

`backend` calls the money-calculations agent before writing and again to review.

**backend builds:**

- Garden create (shares first, then sources, then details, FR-12.29): its own account under control 0306 with both names; the first installment if typed; numbers باغ-1 and GDN-1.
- Installments and running costs through the ventures code of plan 16b (C23, posting type `garden_payment`): shop parts post his 0604 بنام, روکڑ جمع; pocket parts post nothing; the FR-6.7 check on the shop parts.
- A garden lot in بکری: an open garden in the lot seller picker; charges by the rules (default 6%), firm income; the seller credit (C8) to the garden's account; نقد and the advance tick hidden and 0 (FR-12.36).
- A sale in another shop: روکڑ بنام, the garden's account جمع (posting type `venture_receipt`).
- Running balance and rent still due, per date (FR-12.37); `gardens_summary()` for the D5 tile; the R36 data.
- Closing (FR-12.38): C26 over the garden's balance (a بنام balance split as positive and negated); the garden's account ends at 0; no more entries or lots (E65), and a lot of that garden with goods left takes no later sale and gives no fruit (`ERR_GARDEN_CLOSED`); undo by reversal while its date is open; until undone, a بکری posting with a lot of that garden cannot be edited or cancelled.
- The partner overview: under each card, his open gardens and trips with his %, money in and money back so far (17.3). The statement gains garden lines: a shop part, and the closing line with total cost, sales money, result, his %, his part and his money back (FR-17.10).
- Lists page by page. Tests: `golden_day.rs` gains E2E-40 and E2E-40b; the venture property tests for gardens; refusals (shares not 100, shares locked, an entry or lot on a closed garden, an opening on 0306, a partner made inactive while in an open garden).

**frontend builds:** باغات in the main bar as item 5 (before مارکیٹ سفر); the gardens list; the new garden form (shares, then sources, then details); one garden screen with a type list: قسط, باغ خرچ, دوسری دکان میں فروخت (FR-7.67), each partner's part shown before save; the running balance by date; close with the confirm box (FR-7.8) and its undo; an open garden in the بکری lot seller picker; the open gardens and trips under each partner's card. Keyboard only, both languages, four states.

**Worked checks** (PRD 27.2; `worked_day.py` gives the same):

- E2E-40: parts 18,000 / 12,000, then 12,000 / 8,000 and 3,000 / 2,000; الف paid 33,000 from the shop (cash 17,000), ب 22,000 himself. Sale of 100 at 600: ارشد pays 64,400; the garden's account جمع 56,400; commission 8,000 → 3,200 / 2,400 / 2,400. Other shop 10,000: cash 27,000; money in 66,400; running balance 11,400. Closing 66,400 → 39,840 / 26,560; the garden's account 0. Totals 10,040 / 28,960 / 2,400; Σ 41,400 + 65,000 = 106,400 = equity; trial balance 106,400.
- E2E-40b: only the other shop's 10,000; loss −45,000; closing 10,000 → 6,000 / 4,000; الف −27,000; ب +4,000; cash 27,000.

**Done when:** E2E-40 and E2E-40b pass with the property tests.

### 17d-fruit-taken: پھل

**Requirements:** PRD 16.7 (FR-16.10 to FR-16.12), C16, C27, FR-7.53 (rate hint only), FR-7.59 (a fruit credit is a payable posting), FR-15.8 (it never adjusts an advance), FR-17.10 (fruit lines), FR-19.11, E67, A58 to A60; Architecture 15.6 (`fruit_takes`, `bikri_lots.qty_taken`, stock move kind `taken`); the money agent Sections 5.5 and 8.2a. E2E-42.

**backend builds:**

- `fruit_preview`, `fruit_save`, `fruit_list`, edit and cancel by reversal while the date is open (FR-16.12); numbers پھل-1 and FRT-1; posting type `fruit_taken`.
- For one partner: his 0604 بنام the value. For the shop's guests: C27 over the partners active on the date, each one's 0604 بنام his part. Never split by the firm's shares, never a دکان خرچہ.
- From our shop: a lot with balance; `qty_taken` and a `taken` stock move; the lot's seller (account, پختہ person or garden account) جمع the value, no commission; more than the balance is `ERR_QTY_EXCEEDS_LOT`. C16 now subtracts taken, in باقی مال and everywhere a lot balance shows. From another shop: the agent's seller account جمع the value. No cash moves.
- The payable working (money agent 8.2a) counts a fruit credit as a payable posting; it never adjusts an advance.
- The statement gains fruit lines (FR-17.10); the R37 data for D5. Lists page by page.
- Tests: `golden_day.rs` gains E2E-42; C27 with 1,000 over 3 partners gives 334 / 333 / 333; refusals (more than the lot balance, a rate of 0, a lot of a closed garden); trip goods are not offered (FR-16.11).

**frontend builds:** the پھل tab next to خرچہ in the روکڑ area: for whom (a partner, or مہمان), source (اپنی دکان with the lot picker, or دوسری دکان with the agent picker), item, qty, rate (the last rate as a hint only), the value and each partner's part from the backend before save; the list below the form with partner, item and date filters (one day, a month or a year). The list's print and Excel come with R37 in D5.

**Worked checks** (after E2E-16, 5 کیلا left in رحیم's lot): home fruit 2 × 580 = 1,160, الف's total −1,160, رحیم payable 1,590 + 1,160 = 2,750, lot balance 3; guest fruit 3,000 → 1,000 / 1,000 / 1,000, حاجی نذیر payable 3,000; cash 45,000 unchanged; totals −736 / 68 / 68; Σ −600 + 65,000 = 64,400 = equity; trial balance 77,610.

**Done when:** E2E-42 passes.

### 17e-tag-and-prints: the tag after a name, and clear prints on the prints already built

**Requirements:** PRD 10.2 (the tag field), FR-7.25, FR-7.28, FR-7.40, FR-10.7, E38, glossary (نشان), Section 21 Firm (up to 3 contact persons and the trade mark), FR-19.13, R5 (the client's layout), FR-12.12, FR-15.2, FR-15.6, FR-19.3 to FR-19.7; Code Standards 4.2 (Tag), B26, F38, S7; Architecture 15.3 (`firm_contacts`, `trade_mark`) and 15.4 (`accounts.tag`); Tech Stack 4.7. Use the `print-and-export` skill for the prints.

**What is built today (v1.6):** the account form of plan 07 has no tag; `displayName` shows the name only; the firm settings of plan 04 hold name, mandi, address and phones; the prints of plans 14 and 15 (seller bill R1, buyer slips R5, پختہ slip, receipt slip, اگراہی فہرست) use the shared print layout of plan 14, and the buyer slips write each line as "(item qty @ rate)".

**backend builds:**

- **Tag.** The account save and edit take `tag`: empty, or 1 to 4 characters, English capital letters A to Z and `.`, at least one letter; lower-case letters are made capital ("k" saves as "K"); anything else is refused with a field error. It is stored once in `accounts.tag` (the column from migration 0006), never inside `name_ur` or `name_en`. `core::text` adds it to `search_en` and `search_phonetic` in the same transaction (B26); the in-memory search index carries it (Tech Stack 4.7). Accounts with no tag keep exactly the search keys they have.
- **Duplicate check** (FR-10.7): it compares name, tag and phone. Same name and phone with a different tag (or one with a tag and one without) is not a duplicate.
- **Names everywhere carry the tag.** Every result type that carries an account's names for display (search rows, the party card, list rows, ledger rows, print data) carries `tag` too, so `displayName` can add it.
- **Firm contacts and trade mark** (Section 21): the firm settings commands of plan 04 also read and write `firm_contacts` (0 to 3 entries, each a name in both scripts, checked as names, and a phone, checked as a phone, Code Standards 4.2) and `trade_mark` (text, as typed). Each change writes an audit entry (S7).
- **One print header and footer** in `mandi-reports`, used by every print: firm name and mandi in the print language, the contact persons (name in the print language and phone), the trade mark, then the title, the filters or document number, and the print date and time; the footer has "Page x of y" (FR-19.13). The built prints use it: seller bill (R1), buyer slips (R5), پختہ slip, receipt slip (FR-15.2) and the اگراہی فہرست (FR-15.6). A contact or trade mark not set is left out, with no empty box.
- **Body of the built prints.** The seller bill, پختہ slip, receipt slip and اگراہی فہرست show each party with its short code (FR-7.39) and its name with tag, and keep their columns and numbers. R5 keeps only the client's columns: serial (the row number 1, 2, 3), party (name with tag), تفصیل with each line written "(item qty . rate)", رقم (value) and رقم بعد لاگا (value + buyer commission + لاگا, C5). All five use the FR-19.13 header and footer.
- **Buyer slips and R5.** The بکری buyer slips (FR-12.12) still print for every named buyer of that بکری, including his نقدی lines (FR-12.9), in the R5 columns. The R5 ادھاری تفصیل report lists credit buyers only. (Team reading of FR-12.9 and R5; listed in `changes-v1.7.md` Section 7.)
- Tests: tag rules (refused: "1A", "ABCDE", "ب", "K-"; saved: "K", "BA", "D.P", "k" as "K"); search "ashiq k" finds عاشق ظفر K in both languages; the duplicate check with and without a tag; print data snapshots of the five built prints in both languages, with and without contacts; every golden and assist number unchanged.

**frontend builds:** the نشان field on the account form after the English name (English capital letters, never converted to Urdu, FR-7.25); `displayName(record, language)` adds the tag after the name in both languages, in one place (F38), so every screen and print shows "عاشق ظفر K" and "Ashiq Zafar K"; the firm settings screen gains up to 3 contact persons (`NamePair` and a phone each) and the trade mark; the shared print layout gains the new header and footer; the R5 template follows the client's layout. Fluent messages in both languages (`pnpm i18n:check`).

**Worked checks** (after Events 1 to 5): R5 for 10-10-26 has two rows, اکرم ٹول ٹیکس "(کیلا 20 . 600)", رقم 12,000, رقم بعد لاگا 12,880, and ارشد ملتانی "(کیلا 30 . 580)", 17,400, 18,684 (شاہد is نقدی, so not on R5); the seller bill still shows 29,400, 10,084, 19,316 and the words; a buyer with the tag K prints as "اکرم ٹول ٹیکس K" on R5, and with short code 1 on the receipt slip.

**Done when:** the tag works on the form, in search, in the duplicate check and on every screen and print; the five built prints have the FR-19.13 header and footer; R5 follows the client's layout; every golden number is unchanged.

## 3. Design for D4R

The lead runs `/design` inside `docs/design/design-system.md` for: the voucher screen with no mode; the gardens list, the new garden form, one garden with its running balance and the closing box; the partner card with its open gardens and trips; the پھل tab; the account form with the نشان field; the firm settings with the contacts and trade mark; the shared print header and footer, and R5 in the client's layout (the client's prints of 18-10-26 are the reference for R5 and R35). In `docs/design/design-system.md` the 80 mm buyer slip line becomes the R5 form "(کیلا 20 . 600)". The lead also removes the mode, bank and cheque parts from `docs/design/design-system.md`.

## 4. Team flow for D4R

- Plan files run in order: 17b, 17c, 17d, 17e (README Section 4). One goal run, one end gate (Section 4a).
- 17b first: the voucher screen and the lists that 17c and 17d reuse must be in their final shape.
- 17c before 17d: fruit can be taken from a garden's lot.
- 17e last: 17b changes the receipt slip, and 17e then gives every built print the same header and footer.
- `backend` uses the money-calculations agent for 17c and 17d.

## 5. Review focus for D4R

- Golden tests: every number of every golden and assist test of the D3+D4 run unchanged after 17b.
- No mode, bank or cheque left in any screen, command, print or Fluent file (old rows still show as saved). The one choice kept is the salary deduction on ادھار رقم واپسی.
- Garden money: commission is firm income split by the firm's shares; a garden's own money goes only to its partners by its %; a pocket part posts nothing; the garden's account is 0 after closing.
- Fruit: never split by the firm's shares; the guest split is equal (C27); a fruit credit never adjusts an advance.
- Lists: the totals come from the backend over the whole filter, never from the rows on screen.
- Tag: stored only in `accounts.tag`, never inside a name; shown only through `displayName`; in the search keys; an account with no tag keeps its old keys.
- Prints: every built print has the FR-19.13 header and footer; R5 is in the client's layout; no print number changed.
- PRD v1.7 11.2 (the buyer pays commission and لاگا, the seller commission only) changes no code: check that the rule screen and `charge_rules` still allow لاگا on the buyer side only.

## 6. Robustness attack list for D4R

1. **Random gardens.** Random shares and sources, installments, costs, sales in بکری and other shops, edits, cancels, closings and their undo: after every step the trial balance difference is 0; Σ parts of each payment = the payment; Σ shop parts = its روکڑ جمع; a closed garden's account is 0 and Σ over its partners of (money credited − shop parts − pocket parts) = its result.
2. **Closed garden.** An entry, a lot, an edit or a cancel of a بکری with its lot are refused while it is closed, and work after the closing is undone.
3. **Fruit.** Random fruit takes from lots and other shops: lot balance = received − sold − written off − taken; a seller's Σ still open − Σ open advances = his balance; guest parts add up to the value.
4. **Old inputs.** A voucher input that still carries a mode, a bank account or a cheque number is refused and posts nothing.
5. **Paging.** 100,000 vouchers and 10,000 lots: every list opens its first page within NFR-SPD-24; totals equal the sum over every page.
6. **Tag input.** Random Unicode, spaces, digits, signs and long text in the tag field: only a valid tag saves; nothing else reaches the database; search and the party card never break.

## 7. Acceptance (D4R-A1 to D4R-A12)

| ID | Item | Evidence expected |
|---|---|---|
| D4R-A1 | No screen, command, print or Fluent file has a mode, bank or cheque (except the salary deduction on ادھار رقم واپسی); every voucher posts روکڑ; 01020001 is never offered and refuses an opening; the bank master is gone; old rows still show | Test output; screenshots |
| D4R-A2 | Every golden and assist test of the D3+D4 run passes with no expected number changed after 17b | golden_day and assist output |
| D4R-A3 | Every list built in plans 05 to 16 loads page by page with totals over the whole filter; NFR-SPD-24 met at its runner threshold | Test output; `pnpm perf` |
| D4R-A4 | E2E-40 and E2E-40b pass; the venture property tests for gardens pass | golden_day output; property test output |
| D4R-A5 | The garden screens work by keyboard in both languages; باغات is main bar item 5; each partner's card lists his open gardens and trips | Windows E2E names; screenshots |
| D4R-A6 | A closed garden refuses entries and lots, and a later sale or fruit from its lot with goods left (`ERR_GARDEN_CLOSED`); a بکری with its lot cannot be edited or cancelled until the closing is undone; the undo works while its date is open; a partner in an open garden cannot be made inactive | Test output |
| D4R-A7 | E2E-42 passes; C27 gives 334 / 333 / 333 for 1,000 over 3; a fruit credit is payable and never adjusts an advance; more than the lot balance is refused | golden_day output; test output |
| D4R-A8 | The partner statement shows the garden and fruit lines of FR-17.10 with the running کل رقم; E2E-40: الف's lines run −18,000, −12,000, −3,000, +3,200, +39,840, ending at 10,040 | Test output; screenshot |
| D4R-A9 | `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `pnpm perf` and `cargo deny check` exit 0; the full suite, including every D1 to D4 test, passes | Command output |
| D4R-A10 | The tag (PRD 10.2): the rules of Code Standards 4.2 hold ("k" saves as "K"; "1A", "ABCDE", "ب" are refused); `displayName` shows "عاشق ظفر K" and "Ashiq Zafar K"; "ashiq k" finds him; the duplicate check compares name, tag and phone; accounts with no tag keep their search keys | Test output; screenshots in both languages |
| D4R-A11 | Up to 3 contact persons and the trade mark save in the firm settings with an audit entry, and show in the header of every built print (seller bill, buyer slips, پختہ slip, receipt slip, اگراہی فہرست); with none set, the header has no empty box | Test output; print data snapshots; PDFs |
| D4R-A12 | The five built prints carry the FR-19.13 header and footer; the four other than R5 also show each party with short code and name with tag, totals and words where FR-19.7 asks; R5 keeps the client's columns; the بکری buyer slips still show a named buyer's نقدی lines; R5 for 10-10-26 shows "(کیلا 20 . 600)" 12,000 12,880 and "(کیلا 30 . 580)" 17,400 18,684; the seller bill still shows 29,400, 10,084, 19,316 and the words | Print data snapshots in both languages; PDFs |

## 8. Not in D4R

- The dashboard tiles, R33, R35 to R37, analytics views I to K, and every print and Excel of these screens: D5.
- R35 in the client's two layouts, and the FR-19.13 header and footer on every report, analytics view and list print that D5 builds: D5 (it reuses the header and footer of 17e).
