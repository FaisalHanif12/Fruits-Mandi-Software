# Plan 13: Save a بکری: journal, پختہ کھاتہ, freight and drafts

Deliverable: D3, plan file 13. Branch: `feature/bikri` (goal run D3+D4). Version 1.0, 15-10-2026.

## 1. Goal

The munshi presses `F10` and the بکری is saved in one action, all or nothing: the بکری with its lots and lines and their stored values, one balanced journal entry (sale part and cash part), the partner share rows, stock moves, analytics facts, bill matching, advance matching, the پختہ کھاتہ person lines and the audit row. A power cut leaves either all of it or none. Unsaved typing comes back after a crash. After this plan E2E-1 to E2E-5 and E2E-28 (its بکری part) pass.

## 2. Requirements

PRD 12.6 (FR-12.8, FR-12.9, FR-12.10, FR-12.13), 12.7, 13 (FR-13.1 to FR-13.6, FR-13.10), FR-7.29, 14 (FR-14.1, FR-14.2), FR-15.4 and FR-15.5 (bill matching engine), FR-15.8 and FR-12.23 (advance adjusted by the seller credit), FR-5.4 (بکری numbers in global search), FR-6.1 to FR-6.7, FR-7.50 (open the existing بکری), FR-7.57 (`Ctrl+Enter` saves and opens the next; printing comes in plan 14), PRD 22 (C1 to C9, C17), FR-17.4, E3, E19, E20, E30. NFR-SPD-8, NFR-REL-1, NFR-REL-2, NFR-REL-10, NFR-SEC-4, NFR-SEC-5. Architecture 7, 9, 10, 11 and 15.6. Code Standards B3, B5, B16, B17, B20, 9.2. The money-calculations agent before writing the save and the journal, and again to review them. E2E-1 to E2E-5, E2E-17 (Windows), E2E-25, E2E-28. Acceptance: D3-A5 to D3-A9, D3-A10 (E2E-25), D3-A12, D3-A13, D3-A14, D3-A20 (the save part).

## 3. Worked numbers (the tests use exactly these)

| Check | Expected |
|---|---|
| E2E-1 | خام 29,400; کل اخراجات 3,000 + 1,764 + 250 + 50 + 5,000 + 0 + 20 = 10,084; صافی 19,316 |
| E2E-2 | اکرم 12,000 + 720 + 160 = 12,880; ارشد 17,400 + 1,044 + 240 = 18,684 |
| E2E-3 | Sale part: بنام 12,880 + 18,684 = 31,564; جمع 24,316 + 1,764 + 1,764 + 400 + 20 + 250 + 3,000 + 50 = 31,564. Cash part 5,000 = 5,000. Whole entry 36,564 = 36,564 |
| E2E-4 | Shared income 1,764 + 1,764 + 400 + 20 = 3,948; split 1,580 / 1,184 / 1,184 (rows written by the plan 11 hook) |
| E2E-5 | بکری 2: شاہد (پختہ) 10 × 1,200 نقدی; buyer pays 12,800; seller صافی 10,160; sale part 12,800 = 12,800; cash part 12,800 = 12,800; whole entry 25,600 = 25,600; shared income 720 + 720 + 80 + 20 = 1,540; split 616 / 462 / 462; the register shows شاہد, paid |
| Advance (FR-15.8) | رحیم's opening advance 15,000; seller credit 24,316; نقد 5,000; advance adjusted 15,000 (fully); payable 24,316 − 5,000 − 15,000 = 4,316 |
| E2E-28, بکری part | Same بکری with the tick off: credit 24,316; advance adjusted 0 (no `advance_allocations` row); advance open 15,000; ledger 4,316 جمع. The payable 19,316 and the ادائیگی part are checked in D4 |
| Cash after event 1 | 50,000 − 5,000 = 45,000. After event 2 (+12,800): 57,800 |
| Cash confirm | cash 20,000 and نقد 25,000: `bikri_save` without the confirmation fails with `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (cash after −5,000) and writes nothing; with it, it saves |
| Freight | header total 1,000 over lots of 30 and 20 نگ: 600 and 400 (C17); posts to کرایہ کھاتہ 02010002 |
| پختہ seller (FR-13.10) | gross 12,000, deductions before نقد 1,840 − 0 = 1,840: نقد starts at 10,160, صافی 0 |

## 4. Contract

New error codes (with `errors.ftl` in `ur` and `en`, `# review: pending`): `ERR_DRAFT_INVALID` (a draft payload that is not valid JSON or is over 1 MB). The codes of plan 12 (`ERR_NO_LOTS`, `ERR_NO_SALE_LINES`, `ERR_QTY_EXCEEDS_LOT`, `ERR_PUKHTA_*`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`, ...) are now raised by the save too.

### Backend (commands in `app/src/commands/bikri.rs` and `drafts.rs`, all need sign-in)

- `bikri_save { draft: BikriDraft, confirm_cash_below_zero: bool }` -> `BikriSaved { bikri_id, number, entry_id, preview: BikriPreview }`. `services::bikri::save`: one transaction. It calls `domain::bikri::calc_posting`, the same function as `bikri_preview` (B5); a test asserts the numbers of a preview and of the save of the same draft are equal. Order inside the transaction: checks (FR-12.8: at least one lot and one line, required fields, پختہ name and phone, qty rules, open date, shares total 100%, cash confirm) -> number given (next integer, never reused, drafts have none) -> insert `bikri_entries`, `bikri_lots`, `bikri_sales` with their stored method, value, rule id and amount (D5 snapshot, B20, FR-11.6) -> build the journal entry of FR-12.9 with `domain::ledger::build_bikri_entry` (balanced, or `LedgerError`) -> `ledger::post` (draft, lines, one update to posted; the plan 11 hook writes the partner rows) -> stock moves -> analytics facts -> matching -> lot totals written in ONE update (`qty_sold`, `gross`, `seller_comm`, `deductions`, `net`) -> `first_entry_id` -> audit row -> remove the draft. The unbalanced case is a bug signal, logged, nothing saved.
- Journal lines (FR-12.9, Architecture 9): buyer of each line بنام buyer total (own account, نقدی منڈی, or پختہ کھاتہ with the person key); the seller جمع seller credit (C8; بنام if negative; never a negative number in a line); کمیشن آمدن (بیوپاری), کمیشن آمدن (گاہک), لاگا آمدن, فون آمدن, ریٹ فرق جمع (بنام if negative); مزدوری, منشیانہ, کرایہ سٹور and کرایہ کھاتہ جمع (freight only when paid by آڑھتی). Cash part: each lot with نقد: seller بنام, روکڑ جمع; each نقدی line: روکڑ بنام, the same buyer account جمع (with the person key for پختہ). The seller's opening advance lines keep `is_advance`.
- Stock moves: one `received` move per lot (+qty received) and one `sold` move per sale line (−qty), linked to the journal entry. Balance = Σ.
- Analytics facts (what D5 reads, FR-18.6): one row per sale line, sign +1, with year, month, seller, buyer, person key, item, qty, the stored amounts. Team default: `city_id` is the buyer's city and `adda_id` is the بکری's truck stand.
- Bill matching engine (FR-15.4, FR-15.5; `domain::matching::match_bills`, `services::matching`): every debit on a buyer account (or on a پختہ person key) is an open bill; credits settle the oldest bills first by the line order of the journal; a نقدی line is settled at once by its own cash credit; `bill_allocations` rows are written, never deleted (undo sets `is_active = 0` with who and when). D4's وصولی, edit and cancel use the same engine through `services::matching::rematch(account, person_key)`.
- Advance matching (FR-15.8, FR-12.23): when the lot's `adjust_advance = 1`, the seller credit of the posting adjusts his open advances (debit lines with `is_advance`), oldest first, writing `advance_allocations`. With the tick off, nothing is written. A جنرل credit never adjusts advances (16.4).
- پختہ کھاتہ: the person key from name and phone after Urdu normalisation (`core::text`, FR-13.5), written on every line of the person; same rules as any party (FR-13.3); payment type نقدی by default (FR-13.4); a one-time seller's نقد starts at his صافی before نقد (FR-13.10), done inside `calc_posting` so the preview shows it too (extends plan 12).
- `search_pukhta_people { text, side }` -> `Vec<PukhtaPerson { name_ur, name_en, phone, last_date, person_key }>` (known one-time people, newest first; FR-7.29). The merged list with Urdu spellings is shown by `RomanUrduInput`.
- `pukhta_register { from, to, side?, name?, phone?, state?: paid|due }` -> rows (date, بکری no, side, name, phone, item, qty, amount, commission, paid or due with the open amount) (FR-13.6). Paid or due comes from the matching rows.
- Freight (FR-14.1): typed per lot or split by `bikri_split_freight` (plan 12); posts to کرایہ کھاتہ.
- Reads: `bikri_get { id }` -> the saved بکری with stored values, postings and status; `bikri_list { date?, from?, to?, seller_id?, item_id?, status? }` (today's list, FR-12.7 columns: number, time, truck, sellers, items, qty, خام, صافی, buyer total, status); `bikri_find { number?, truck_no?, seller_id?, date?, item_id? }`; `bikri_navigate { current_id?, to: first|previous|next|last }` -> id (FR-12.13). Global search (FR-5.4) gains بکری number and truck number results that open the screen (extends the plan 06/08 search command).
- Drafts (FR-12.10, NFR-REL-2): `drafts_save { screen, payload_json }` (upsert per user and screen, JSON checked, over 1 MB refused), `drafts_get { screen }` -> `Option<DraftRow { payload_json, saved_at }>`, `drafts_discard { screen }`. The save removes the draft in its transaction. The frontend auto-saves every 5 seconds (`DRAFT_SAVE_INTERVAL_MS = 5_000`).
- Tests: `backend/crates/services/tests/golden_day.rs` gains E2E-1 to E2E-5 and the بکری part of E2E-28, after the D2 setup (E2E-19) and plan 11 partners, with the injected clock at 10-10-26. `assist.rs` gains the backend parts of E2E-25 (truck memory and the same-truck notice, seller memory) and E2E-26 on saved data. Integration tests: all or nothing (a forced failure after each step leaves no row), numbering never reused, cash confirm, advance tick on and off, پختہ register, navigation, drafts, sign-in refusal. Property tests: the sale part and the whole entry balance for random بکری; Σ share rows = shared income; buyer totals add up to the sale part's بنام; preview equals save. NFR-SPD-8: 30 lines saved under 300 ms at the runner threshold (criterion bench in `backend/crates/services/benches/`, thresholds in `scripts/perf/`).
- Money agent before and after; its report goes into the hand-off.

### Frontend

- Save: `F10` or `Ctrl+S` saves; `Ctrl+Enter` saves and opens a new empty بکری (printing joins in plan 14). On `cash.needs_confirm` the `CashConfirmDialog` of plan 12 opens first; Yes sends `confirm_cash_below_zero: true`; No returns with nothing saved and the نقد field selected. Field errors from the backend are shown at their cells (`lots[i].sales[j].qty`); a toast "بکری { $number } محفوظ ہو گئی" (message gender per the glossary), then a new empty بکری is ready.
- The draft: auto-save every 5 s while typing (cancelled when the draft is empty); at start-up the app offers "پچھلی بکری مکمل کریں؟" with restore and discard; the draft is removed after a save.
- Today's list screen (آج کی بکری) with its filters; first / previous / next / last buttons and find on the book screen; a saved بکری opens read-only (edit comes in plan 16); the same-truck notice's "کھولیں" opens the existing بکری.
- The پختہ fields on a lot and a line: name and phone with the merged list (known people first, then Urdu spellings), required name, and phone when credit.
- The پختہ کھاتہ register screen (route under بکری side menu), with date, name, phone and paid-or-due filters; four states.
- Fluent: `bikri.ftl` additions, `pukhta.ftl` (or inside `bikri.ftl` if the module naming says so, Tech Stack 5.2 uses `pukhta_khata`), errors. New Urdu words `# review: pending`.
- Windows E2E specs (written and syntax-checked, run LOCAL-ONLY): the D3 story by keyboard (Event 1 and Event 2) and E2E-17 (close the app while typing, reopen, the draft is offered and restored).

### Contract as built in T1 (agreed 16-10-2026, recorded by the lead)

New commands, each with one `{ input }` object and sign-in: `bikri_save`, `bikri_get`, `bikri_list`, `bikri_find`, `bikri_navigate`, `search_pukhta_people`, `pukhta_register`, `drafts_save`, `drafts_get`, `drafts_discard`. `search_global` keeps its name and gains a result kind. New code `ERR_DRAFT_INVALID` (field `payload_json`, value `max_bytes`), its ur and en messages written by `frontend`. Matches the plan: `bikri_save { draft, confirm_cash_below_zero }` -> `BikriSaved { bikri_id, number, entry_id, preview }`; `drafts_get` -> `DraftRow | null`; `bikri_navigate` -> number or null; `search_pukhta_people { text, side: seller | buyer }` -> `PukhtaPerson[] { name_ur, name_en, phone, last_date, person_key }`. Details the plan did not fix:

1. `bikri_find` returns `BikriListRow[]` (newest first, at most 50), not ids; `truck_no` is a case-insensitive START match. The same-truck notice has only `bikri_number`, so "کھولیں" = `bikri_find { number }` -> `row.id` -> `bikri_get { id }`.
2. `bikri_list` and `bikri_find` dates are optional text (null or "" = none; DD-MM-YYYY or a shortcut); `date` wins over `from` and `to`; `status` is `saved` or `cancelled`. `BikriListRow` = id, number, entry_date, created_at (UTC text; the screen shows Pakistan time), truck_no, sellers, pukhta_sellers, items, qty_received, qty_sold, gross, net (can be negative), buyer_total, status.
3. `bikri_get` returns `BikriDetail` with stored values only (never recalculated): header, zamindar, `lots[]` (`SavedLot` with stored charges: method, value, amount, rule_id; quantities; gross, seller_comm, deductions, net; adjust_advance; first_entry_id; `sales[]` as `SavedSale` with the three charge snapshots, buyer_total, rate_diff, payment_type, is_active), `postings[]` and `split[]` from the saved share rows.
4. `drafts_save` returns `DraftSaved { saved_at }`. The screen key of the بکری draft is the text `bikri` (a-z 0-9 _ . - up to 40 characters, else `ERR_FORMAT_INVALID` on `screen`). Invalid JSON or over 1 MB is `ERR_DRAFT_INVALID`. No audit row for drafts. Discarding nothing is fine.
5. `confirm_cash_below_zero` has a serde default of false, but the TypeScript field is required: always send it.
6. `GlobalHit` gains `{ kind: 'bikri', hit: BikriHit { id, number, entry_date, truck_no, status, matched: 'number' | 'truck' } }`. Order: accounts, then the بکری whose number equals the typed digits, then the newest بکری whose truck starts with the text (2 or more letters, at most 5), all within `limit`. Opening a hit opens the screen on `hit.id`.
7. `pukhta_register` rows: entry_date, bikri_id, bikri_number, side, name_ur, name_en, phone, person_key, item, qty, amount (a buyer line: buyer total; a seller lot: gross), commission, paid, open (paisa still open), cancelled. A cancelled line has paid false, open 0, cancelled true.
8. Implementation note: `bikri_lots` has CHECKs tying deductions and net to the lot charges at INSERT time, so the save inserts each lot with its final totals in one write (the same numbers as `calc_posting`); sale rows are written after the journal entry (`journal_entry_id` is NOT NULL). Same effect as "totals in ONE update".

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, stubs returning `ERR_NOT_IMPLEMENTED`, error codes, `pnpm gen:types` | backend | plan 12 built |
| T2 | Save, journal, matching, stock, facts, پختہ, reads, drafts, golden tests, money agent before and after | backend | T1 |
| T3 | Save flow, draft offer, list, navigation, پختہ fields and register against the stubs; Fluent | frontend | T1 |
| T4 | Wire to the real commands; keyboard run of Event 1 and Event 2; screenshots | frontend | T2, T3 |

## 6. Acceptance checklist

1. E2E-1 to E2E-5 pass in `golden_day.rs` with the numbers of section 3 (D3-A5 to D3-A8; `cargo test -p mandi-services --test golden_day`).
2. Advance adjustment 15,000 / payable 4,316; with the tick off, the E2E-28 بکری part (D3-A9).
3. E2E-25 passes in `assist.rs`; the Windows spec is written (D3-A10).
4. All or nothing under forced failures after every step; the books balance after each (D3-A12; robustness repeats it at the end of the run).
5. Freight per lot and split from the header total post to کرایہ کھاتہ (D3-A14).
6. Cash below zero: save refused without confirmation, accepted with it (D3-A20).
7. NFR-SPD-8 measured by the bench at the runner threshold; the draft offer test; the E2E-17 spec written (D3-A13, D3-A17).
8. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` exit 0; every earlier test still passes.

## 7. Out of scope

Prints and the `Ctrl+Enter` printing (plan 14); later sales, باقی مال, edit, cancel, undo (plan 16); وصولی and ادائیگی (plan 15); own goods (plan 16b); the dashboard and analytics screens (D5).

## 8. Open questions

None for Umar. Team defaults: the analytics `city_id` is the buyer's city (D5 may revisit); a saved بکری opens read-only until plan 16.
