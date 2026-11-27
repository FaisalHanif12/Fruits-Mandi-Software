# Plan 99: perfect Urdu typing, a clear روکڑ, a faster assistant, garden page, small fixes

| | |
|---|---|
| Status | DRAFT, waiting for Umar's approval (27-11-2026, screenshots 96 to 106) |
| Branch | `fix/rokar-garden` (same goal run as plans 95 to 98) |
| PRD | FR-7.21 to FR-7.27, FR-7.66, NFR-URD-24; FR-10.9, FR-16.14, FR-15.11; FR-7.72 to FR-7.90, FR-7.87, NFR-AI-3, NFR-AI-5; FR-12.11, FR-12.37, FR-12.40, FR-12.41; FR-17.3, FR-19.11 |

## 0. Before this plan starts

Plans 97 and 98 are built but not committed (311 changed files). The lead runs their checks (tests, lint, typecheck, i18n:check, golden tests). When they are green, the lead commits them as two commits and pushes. Plan 99 then starts on a clean tree.

## 1. Goal

Five things Umar found while testing:
1. **Urdu typing.** Typed Roman Urdu must become correct, readable Urdu. Example: "gher ly kr gaye" must become گھر لے کر گئے, not غرلی کرگی. "karaya" must become کرایہ, not کرعیہ.
2. **روکڑ.** The cash book must be simple and exact: buyer and seller openings go through cash, every row has its right name, there is no سرمایہ کہاں ہے card, and the opening card means what Umar expects.
3. **Assistant.** Complex questions must go straight to the right section and answer faster.
4. **Small fixes.** The نقد ادھار reason becomes optional. The new account form must not be cut off. The partner page's open gardens and trips list must page by month.
5. **Garden page.** مال آیا is split into sold and not yet sold. The figures card is open by default. History comes above the figures. A new section shows where the garden's sale money is.

## 2. What the investigation found

### 2.1 Urdu typing
- The word list (12,944 rows) is almost all names and places. It has only 663 everyday words, and **every row is still `pending`**, so no row has been reviewed.
- 4,730 rows are machine-made "places" (source `engine`) and are often wrong. Examples: `ghar → غار`, `kar → کر` (as a place), `aya → عیہ`. Also `hai → حئی` comes from a Wikidata name.
- Common words are missing: le, ly, gaye, gae, diya, liya, gaya, paise, wapas, bhi, ko, ne, hain, karaya, bijli and many more.
- Ranking: an unreviewed list entry scores 0.95 and always beats the spelling rules, which are capped at 0.5. The "two list words written together" rule then joins wrong fragments: karaya = کر + عیہ = کرعیہ.
- Rules: for `gh` the options are غ, گھ and گ, but غ wins because the seed has 272 غ against 45 گھ. So ghar becomes غر. The rule `g("o")` appears twice (graphemes.rs:57-58).
- Free-text fields (reasons, narrations) convert one word at a time, with no context.

### 2.2 روکڑ (real data, a copy of Umar's database)
- **شروع میں روکڑ = 0 is correct as built.** It is the cash on the day before the 30-day window starts. The first cash entries are the capital openings of 29-10-2026, which fall inside the window, so nothing is before it.
- **Buyer and seller openings never touch cash.** They post against ابتدائی فرق (06019999), which today stands at 85,000 جمع. Examples: مانی 30,000 بنام, عدنان (seller) 60,000 بنام, احسان 20,000 جمع.
- **A پیشگی to a buyer** is saved exactly as an ادائیگی (voucher type `payment`), with no record that it was typed on the پیشگی tab. So روکڑ shows "ادائیگی 2" for مانی's 50,000.
- Labels are generic: "باغ کی ادائیگی" (for both an installment and a garden cost), "خریداری" and "دوسری دکان میں فروخت" give no garden or trip name. A بکری row mixes cash in and cash out, with an empty account column.
- Edits and cancels show three rows each (original, الٹا اندراج, new), and there is no way to hide them.

### 2.3 Assistant (52 real questions from the log)
- Tools take 156 ms in total. **Model round trips are 98.9% of the time:** each call takes 3 to 10 s, and 11 of 25 model questions took over 15 s.
- Main waste: `period` almost always gets its own round, costing 4 to 10 s each time.
- Other wasted calls:
  - `tiles` called for a how-to question.
  - Analytics called with no dates, which returns `ERR_REQUIRED`; then a retry.
  - R2 (sale lines) used for "the last بکری" instead of R1.
  - "How to raise sales" answered with 3 failing tool calls instead of general advice.
- There is no direct read for "the last بکری" or for a party's one-row-per-بکری history.

### 2.4 Garden, partner page, small items
- The arrivals table always asks for every state. The backend already has `in_stock | sold_out` (db/src/arrivals.rs:459), but the screen never uses it. That is why sold rows show 0 / 0 / 0.
- The figures card (`GardenFiguresFold`) starts folded (`FoldSection` default false). History is below it.
- No query lists the garden's sale lines with their buyers and what happened to the money.
- The partner list of open ventures returns everything, with no paging.
- The new account form is pinned to the screen height (`fill-to-corner`) and has an inner scroll box with no visible scroll bar, so the name card is cut off. MasterForm does it correctly.
- The نقد ادھار reason is required by `ERR_PAYMENT_TO_BUYER_NEEDS_REASON` and by a database trigger in migration 0001.

## 3. Requirement changes (Umar approves them with this plan)

| Doc | Change |
|---|---|
| PRD FR-10.9, 10.2 | A seller's or buyer's opening balance goes through روکڑ (Umar, 27-11-2026): an opening بنام posts party بنام, روکڑ جمع (cash went out to him); an opening جمع posts روکڑ بنام, party جمع (cash came in from him). A change reverses and reposts the same way. Taking cash below 0 asks first (FR-6.7). ادھار رقم persons and الگ کھاتے too (Q2). Other accounts keep ابتدائی فرق. |
| PRD FR-16.14 | The سرمایہ کہاں ہے card is removed, along with its print and the plan 87 placing flow. The top cards follow Umar's answer to Q1. Rows are named by what they are, with the party, garden or trip. Edited and cancelled entries show once, with a switch. There is a type filter and a summary by type. |
| PRD FR-15.11 | The reason for cash lent to a buyer (and for a buyer پیشگی) is optional. A new migration drops the trigger. |
| PRD FR-15.11, 15.4 | A پیشگی to a buyer still posts as cash lent, but it is remembered and shown as پیشگی everywhere (روکڑ, his ledger, the پیشگی list). |
| PRD FR-7.87, D6 prompt | `report`, `analytics` and `estimate` take a period word directly. R1 takes `bikri: "last"` and a date. R13 gets a `layout: "bikri"` (one row per بکری, the پارٹی کھاتے function). The routing table is rewritten. NFR-AI-5's fixed part goes up to 12,800 tokens if needed. |
| PRD FR-12.37 (new FR-12.53) | Garden page: arrivals split into sold and not yet sold; figures open by default; history above the figures; the garden sale money list. |
| PRD FR-17.3 | The open gardens and trips list pages by month. |
| NFR-URD-24 | Add an everyday-words test list (Roman Urdu as people type it) with a gate: first suggestion at least 95%, top 3 at least 99%. |

## 4. Worked numbers

**Openings through cash (Umar's copy).** Today cash is 1,057,840 and ابتدائی فرق is 85,000 جمع.
- Openings بنام: مانی 30,000, انس 15,000, عدنان (seller) 60,000, عبدالرحمان 20,000. Total 125,000 out of cash.
- Openings جمع: فیصل 10,000, ہارون 10,000, احسان 20,000. Total 40,000 into cash.
- After the one-time fix:
  - Cash = 1,057,840 − 125,000 + 40,000 = **972,840**.
  - ابتدائی فرق = 85,000 − 125,000 + 40,000 = **0**.
- The trial balance stays balanced, and no party balance changes.

**Golden test E2E-65.** ہارون's opening is 10,000 جمع. If that test reads total cash, its expected number goes up by 10,000. The lead lists every golden number that would change, and none is changed until Umar approves it.

**Garden sale money (example).** A garden lot sells 20 پیٹی آم to مانی at 1,000:
- Value 20,000. Buyer commission 1,200, لاگا 160, so his bill is 21,360.
- He pays 0 on the spot.
- Seller commission is 1,200, so the garden is credited 18,800 (no other lot charges).
- The row shows: buyer مانی, bill 21,360, paid on the spot 0, still on his account 21,360 (or less, after a later وصولی matched to that bill), garden credit 18,800.

The money agent checks every figure before the code is written.

## 5. Contract (backend to frontend)

Exact field names are written here by the lead after T1 of each area, before the frontend builds against them.
- `cashbook_get`:
  - New input `show_reversed: bool` and `kinds: CashKind[] | null`.
  - Each row gets `kind` (one enum value per row type) and a `refs` list (party, garden, trip, بکری, voucher number) for the label.
  - A بکری's cash is split into rows: موقع پر جمع (in), نقد ادائیگی (out), پیشگی (out).
  - New `summary_by_kind` over the whole filter.
  - Opening and closing as before. Q1 decides the top card.
- `cashbook_capital_where`, `cashbook_capital_print_data`, `cashbook_capital_print` and `partners_place_opening_capital` are removed.
- `accounts_openings_to_cash_preview` and `accounts_openings_to_cash` (the one-time fix):
  - The preview lists each opening, its cash effect, and cash before and after.
  - The save needs a reason and the sign (FR-21.4) and reopens and closes days if needed.
- `vouchers` rows carry `entered_as: "advance" | null`.
- `gardens_arrivals_list` input `state: in_stock | sold_out | null`, used by the two sections.
- New `gardens_sale_money`:
  - Input: garden id, month window, offset and limit, search.
  - Rows: date, بکری, lot, buyer (or other shop), qty, rate, value, bill, paid on the spot, collected since, still on account, garden credit.
  - Totals over the whole filter.
- `partners_open_ventures`: input month window, offset and limit; output rows and totals.
- **Agreed R1 contract (backend-99r, 27-11-2026):**
  - `cashbook_get` input `kinds: CashKind[] | null` and `show_reversed`.
  - Each row has `status: live | reversed | reversal`, `kind`, and `refs { parties (at most 3), party_count, partner, garden, trip, expense_head }`. The row's `account` and `partner` fields are removed.
  - `summary: [{ group (the 12 groups of FR-16.14), kinds, count, in, out }]` covers the date range and is not narrowed by the type filter.
  - The cards get `total_capital_paisa` beside the opening and closing.
  - The one-time fix is `accounts_openings_to_cash_preview` and `accounts_openings_to_cash { reason, sign, confirm_cash_below_zero }`, with `ERR_OPENINGS_ALREADY_IN_CASH`.
  - `VoucherRow.entered_as: "advance" | null`.
  - `ERR_PAYMENT_TO_BUYER_NEEDS_REASON` and `ERR_NOTHING_TO_PLACE` are removed.
- `translit_suggest` (agreed 27-11-2026, backend-99u): `SuggestInput` gains `kind: Kind | null` ("word" for free text, a name kind for name fields) and `before: string | null` (the field text before the word being typed, for context). Core exposes `suggest_for(text, learned, list, direction, &Hint { kind, before })`; the old `suggest(...)` stays and means name ranking (assistant unchanged). backend-99r passes it through services and the command and runs `pnpm gen:types`; frontend-99a's `useSuggestions` sends the field's kind and the preceding text, and keeps sending only the current word as `text`.
- Errors: no new codes except the one-time fix (`ERR_OPENINGS_ALREADY_IN_CASH`) and the removal of `ERR_PAYMENT_TO_BUYER_NEEDS_REASON` as a block.

## 6. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T0 | Commit plans 97 and 98 after a green run | lead | |
| U1a | Urdu vocabulary, the big part (Umar, 27-11-2026: "so extensive that whatever word we write shows exactly, professionally"). Build a reviewed everyday dictionary from the most frequent ~15,000 Urdu words of an open-licensed Urdu corpus (licence checked and credited, Tech Stack 4.4), not only the words Umar named: every verb in every form (kar, kr, ker, karna, karte, kiya, kia, kiye ...), pronouns, postpositions, numbers in words, days, months, relations, food, home, health, travel, money and shop words, mandi words, fruits, vegetables, English loanwords used in Urdu (bill, petrol, school, fees, mobile). Each Urdu word gets all the common Roman spellings people type (about 40,000 Roman forms: ghar/gher/ghr, le/ly/lay, gaye/gae/gye/gaey, kiraya/karaya/kraya ...). Every row is made by one agent and checked by a second, independent checker against the Urdu spelling of a standard dictionary; disagreements go to the lead. Rows get source `everyday` and review `checked` | backend-99u, plus an independent checker agent | T0 |
| U1b | Urdu engine: remove or demote the 4,730 machine-made rows and fix wrong rows (ghar غار, aya عیہ, hai حئی ...); ranking: in free-text fields everyday words first, in name fields names first, unreviewed rows below checked ones, no joining of short fragments; rules for gh, bh, ph, th, dh, kh, ch, rh, the endings -a/-ah/-aya/-ay/-ai/-y/-e, doubled letters, ain and hamza words; phrase-aware conversion with common two- and three-word phrases (ly kr, chala gaya, de diya ...); fix the duplicate `g("o")` rule | backend-99u | U1a (in parts) |
| U1c | Urdu tests: the held-out list is written by a separate agent (urdu-test-writer) that never reads core/data; backend-99u writes the harness and gates. An everyday test list of at least 3,000 items typed the way Pakistanis really type (single words, spelling variants, 500 full sentences from reasons, narrations and expense heads), kept apart from the dictionary so it measures, not memorises; gate: first suggestion at least 95%, top 3 at least 99%, every sentence fully right at least 90%; the 1,000-name list must not drop | backend-99u | U1b |
| U2 | Urdu: check every Urdu text field of every screen uses the right kind (name or word); free text sends the phrase for context; live typing test in the app of at least 300 phrases from real mandi use (reasons, narrations, expense heads, names), and every miss goes back to U1a or U1b | frontend-99a | U1c |
| R1 | روکڑ backend: openings through cash (new and edited) and the one-time fix; buyer پیشگی marker (migration); row kinds and refs, split بکری rows, hide reversed, kind filter, summary; remove the capital card commands; drop the reason trigger (migration); money agent before and after | backend-99r | T0 |
| R2 | روکڑ screen: new cards, labels, filter, switch, summary; remove the card and its print; account form opening shows "روکڑ سے" note and cash preview; one-time fix button with sign; reason optional on the voucher screen | frontend-99b | R1 contract |
| A1 | Assistant backend: period word on reads; R1 latest and date; R13 bikri layout; instant pre-reads for a garden or trip with no name and for "pichli bikri"; analytics defaults instead of ERR_REQUIRED; check the warm-up | backend-99ai | T0 |
| A2 | Assistant prompt: rewrite the routing table and playbook in D6 (lead writes the D6 text, Umar's approval covers it), copy it word for word, keep P1 to P7 green, fixed part within the new limit; new golden questions for each route | backend-99ai | A1 |
| A3 | Assistant measure: replay the 52 real questions and the NFR-AI-2 list live, before and after; report model calls, time and accuracy per question | backend-99ai | A2 |
| G1 | Garden backend: `gardens_sale_money` with bill matching status; arrivals state filter check; partner open ventures paging; money agent | backend-99r | R1 |
| G2 | Garden screen: two arrival sections (بکنا باقی / بک چکا) with their own columns; figures open; history above figures; sale money section with a month window, pages, search and totals; partner list paging; new account form scroll fix | frontend-99a | G1 contract, U2 |
| T5 | Review (whole run, plans 95 to 99) | reviewer | all, Umar's go-ahead |
| T6 | Robustness (whole run) | robustness | all, Umar's go-ahead |
| T7 | Fix loop, docs, what's new, commit, push | lead | T5, T6 |

No two teammates edit the same file. backend-99u owns `core/src/translit` and its data and tests. backend-99ai owns `crates/assistant`. backend-99r owns the rest of the backend for this plan. frontend-99a owns gardens, partners, masters and shared/urdu. frontend-99b owns cashbook, vouchers and accounts openings.

## 7. Acceptance checklist

**Urdu**
- About 15,000 everyday Urdu words with about 40,000 Roman spellings, every row checked twice.
- The everyday test list (3,000 items, 500 sentences) passes: first suggestion at least 95%, top 3 at least 99%, sentences fully right at least 90%.
- Umar's examples pass: "gher ly kr gaye" gives گھر لے کر گئے, "karaya" and "kiraya" give کرایہ, "hai" in a free-text field gives ہے.
- The 1,000-name list does not drop.
- The lead types 100 phrases of his own in the running app and every one is right.

**روکڑ**
- New openings post through cash. After the one-time fix on the copy: cash 972,840, ابتدائی فرق 0, trial balance balanced, party balances unchanged. The closing card equals the cash tile and R12.
- Every row is named for what it is, with its party, garden or trip. A buyer پیشگی shows as پیشگی.
- Edited and cancelled rows are hidden by default; the switch shows them; the totals are the same either way.
- The capital card is gone from the screen, the code, the commands, the prints and the tests.

**Assistant**
- Every golden question passes.
- Replay of the 52 questions: average model calls at most 2.0 (2.36 now). No `period`-only round. The number of questions over 15 s halves or better. No wrong figure.
- NFR-AI-2: at least 97 of 100.

**Garden**
- Sold and unsold arrivals are separate, with correct non-zero figures.
- Figures are open by default; history is above them.
- The sale money list totals equal the garden's money in from our shop and from other shops (R36).
- Every list pages; four states; both languages; 1024 to 1920; keyboard.

**Small fixes**
- A cash-lent voucher saves with no reason.
- The new account form scrolls fully at 1024 × 600.
- The partner list pages by month.

**General**
- Golden tests are unchanged except numbers Umar approves.
- lint, typecheck, i18n:check and the full test suite are green.

## Follow-ups

- backend-99r edits robustness-owned files `services/tests/robustness_gate_t2_dash.rs` and `speed_gate_f` as little as possible, with the lead's approval, because they call the removed capital card. The robustness tester reviews them at the gate.

## 8. Out of scope

- New assistant tools beyond the input changes above.
- A full rewrite of the Fluent screen text. Only the conversion of typed text and its field kinds are in scope.

## 9. Open questions for Umar

- **Q1. The top cards of روکڑ.**
  - A (recommended): three cards: کل سرمایہ (capital put in less taken back, up to the window end), شروع میں روکڑ (cash before the window), آخر میں روکڑ.
  - B: the first card shows the capital instead of the opening cash.
- **Q2. Openings of ادھار رقم persons and الگ کھاتے.**
  - Also through cash, or keep ابتدائی فرق (default)?
- **Q3. Live assistant tests.** They use the OpenRouter key: about 150 questions, roughly $0.30, plus the daily limit's test rows. OK?

## 10. Umar's answers (27-11-2026)

- Q1: three cards: کل سرمایہ (capital put in less taken back, up to the window end), شروع میں روکڑ (cash before the window), آخر میں روکڑ.
- Q2: yes. Openings of ادھار رقم persons and الگ کھاتے also go through روکڑ, both sides, in the same way and in the one-time fix.
- Q3: yes. Live assistant tests through the OpenRouter key (about 150 questions).
- Golden change (Umar, 27-11-2026): E2E-65 cash 50,000 → 60,000 and 81,360 → 91,360, because ہارون's opening of 10,000 جمع now comes into روکڑ. No other golden number changes.
- Openings on a closed day (Umar, 27-11-2026): an opening through روکڑ dated on a closed day, including a new account's, needs reopen with a reason and the sign (FR-21.4), then closes again. Accounts still on ابتدائی فرق keep FR-10.11.

## 11. Agreed G1 contract (backend-99r, 27-11-2026; bindings regenerated)

- `gardens_sale_money(GardensSaleMoneyInput { garden_id, from, to, arrival_id, search, offset, limit (0 = 100) })` returns `{ rows, total_count, totals }`.
  - `Row`:
    - kind and source: `kind` (bikri | lot_charges | other_shop), date, bikri_id, bikri_number, `lot { lot_id, lot_no, lot_counter, item, packing } | null`, item | null, buyer | null, shop_name, entry_id;
    - figures: qty, weight_grams, rate_paisa, value_paisa, buyer_total_paisa, paid_paisa, collected_paisa, still_due_paisa, garden_credit_paisa;
    - `charges { freight, labour, munshiana, phone, store_rent } | null` and `working`.
  - `totals { qty, value, buyer_total, paid, collected, still_due, lot_charges (positive), garden_credit }`. garden_credit over the whole garden equals money in from our shop and other shops.
  - Errors: ERR_NOT_FOUND (garden_id), date codes (from, to).
- `ArrivalRow` adds `sale_value_paisa` and `last_sale_date`. `ArrivalsTotals` adds `qty_our_sold`, `qty_other_sold`, `qty_our_left`, `qty_other_left` and `sale_value_paisa`. `state` in_stock / sold_out works.
- `partners_open_ventures(PartnersOpenVenturesInput { partner_id, from, to, offset, limit })` returns `{ rows (each with entry_date), total_count, totals { paid_shop, paid_pocket, money_back, gardens_balance, trips_result } }`. The month is the day the garden or trip was made; newest first.
- `WindowSource` adds `{ kind: 'garden_sales', garden_id }` and `{ kind: 'partner_ventures', partner_id }` (the second needs the partner code).
- Proof on a throwaway copy of Umar's data:
  - Openings fix: 7 openings move (Dr 125,000, Cr 40,000); cash 1,057,840 → 972,840; ابتدائی فرق 85,000 → 0; trial balance 0. A second run gives ERR_OPENINGS_ALREADY_IN_CASH, and a wrong sign changes nothing.
  - Garden sale money: bill 95,880, paid 64,000, due 31,880, credit 99,600 = money in.
- Lead decision, partner list: a month window by the date each garden or trip was made, opening on the latest month that has one. The compact line keeps "کل کھلے: N" for all open ones, so nothing is hidden, and the arrows step back month by month.

## 12. State at the usage-limit stop (27-11-2026, about 03:55)

All builders stopped at the usage limit. Nothing is committed, and the tree is not green.

- **backend-99r.** Written: R1, G1, the translit pass-through, the buyer پیشگی flag in R13/R14, history and the day slip, and the R36 refund fix. Migration LATEST is 31.
  - Left: triage the failing services tests (accounts, accounts_any_day, assist, bikri_1000_lines, collection_print, day_money_fixtures, edit_review, golden_day, ledger); the migration and schema tests; the robustness_security hang (print and updater commands in the mock app); `CashLent.advance` plus gen:types; fixtures regenerated; fmt, clippy; the final money review.
- **backend-99ai.** A1 and A2 written; D6 v1.15; prefix measured at 12,122. Fixed a real bug: a new question after a partner-locked one failed with Azure 400.
  - Before (live): 27 of 43 questions used the model; 61 calls, average 2.26; 4 over 15 s.
  - Left: the full suite, clippy, the live after-replay, NFR-AI-2 live, reading `CashLent.advance`.
- **backend-99u.** Fresh-list official blind result: 82.43 / 90.03 / 90.69. After general rules: 86.42 / 92.77 / 92.58. First list, blind half: 98.11 / 99.29 / 97.96.
  - Left: checker on batches 23 to 28 and the ~220 round-3 rows; more coverage; a third fresh list; NFR-SPD-20 re-timed.
- **frontend-99a.** T0 fixes, the account form fix and the U2 code are done. G2 is written but not verified.
  - Left: garden and partner tests, fixtures, screenshots, the live 300-phrase test.
- **frontend-99b.** The روکڑ page, the one-time fix view, the account form note, the buyer پیشگی labels and two real screen bugs (ListGrid focus echo, R37 overflow) are done.
  - Left: cashbook fixtures and real-shape tests, typecheck, lint, the full suite, screenshots.
- **Umar (27-11-2026):** "make sure we bring a lot of improvement in the AI assistant compared to the previous version; no regression at all." Gate: every one of the 43 replayed questions and the NFR-AI-2 list must be equal or better after (same or better outcome, same figures, no new error), plus a clear drop in model calls and time.
- E2E-65 detail (backend-99r, 27-11-2026): the approved change of the starting cash, 50,000 → 60,000, also appears in the other E2E-65 tests in the same file (lines 298, 309 = 60,000 + 194,520, and 349). It is the same change. No other golden number changes.
- Robustness-owned follow-ups (smallest edits by backend-99r, with the lead's approval):
  - `robustness_fault` and `robustness_journal`: party openings now need the cash confirm and an open date.
  - `robustness_gate_t1`: no sign text in the audit since v1.58.
  - `app/robustness_security`: the mock hostile test skips the real-print and updates_ commands.
- Assistant (lead decision, 27-11-2026, option A, inside the plan Umar approved and his no-regression rule): a view with no dates reads all the books as before; only a comparison with a missing date is filled. PRD FR-7.87 is reworded.
- Golden change (Umar, 27-11-2026): golden_day/plan88_buyer_peshgi.rs: the buyer list total goes from 3,000 to 6,000 (preview and saved), kinds [Payment, Advance, Payment], and the refusal asserts now expect success, because the reason is optional. E2E-32 figures are unchanged. Money review of R1/G1: PASS WITH MINOR.
- Urdu final (Umar, 27-11-2026): the third list, run once blind, scored 94.99 / 97.31 / 94.98. These numbers are a floor now; 95 / 99 must be met before go-live (NFR-URD-30). Engine final; dictionary 16,417 words and 69,942 rows, all checked twice.
- Lead decision (27-11-2026): the کل سرمایہ card on روکڑ stays visible without the partner code. It is a shop-level total, and the روکڑ rows already show each capital entry with the partner name (FR-16.14). FR-17.11 guards per-partner totals and history, which this card does not show.
- Follow-ups (Urdu):
  - The "checked" mark comes from the AI checkers. The human Urdu reviewer still has to review the dictionary before go-live (NFR-URD-4).
  - The 18 misses from the live 324-phrase test (304 of 324 phrases exact) wait for a later plan.
  - phrase.rs and wordlist.rs are over 400 lines (G3 SHOULD).
- Robustness-owned follow-up: robustness_opening (5 tests) now saves new party openings on an open day and tests the FR-10.11 mechanism with a payables account (smallest edit, approved by the lead). Pre-existing flake, not from plan 99: app runner::every_error_reference_is_written_to_the_log fails with 2 test threads and passes single-thread.
