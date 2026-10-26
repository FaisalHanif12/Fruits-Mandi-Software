# Plan 32: simple codes, a simple accounts list, and pickers that open on click

Version 1.0, 27-10-2026. Lead: tech-lead. Asked by Umar on 27-10-2026 while testing the app himself (his answers to the lead's four questions are in Section 8). Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5` (it holds all of Run A too). Run A is merged later, as planned. The review gate still waits for Umar's "continue".

## 1. Goal

Make the Index and every picker simple for the client. Parties and lists get short codes by type (first seller 0010). The Accounts list shows only sellers, buyers and pukhta. Clicking any picker shows all records of that box's type, filtered by code or name as you type. When nothing matches, a button takes you to the Index to add one, and you come back to your entry with nothing lost. The test data is cleaned so the client starts from zero.

## 2. Requirements (approved by Umar, 27-10-2026; the lead writes PRD 1.8 from this)

R1. **Type numbers.** Seller `001`, buyer `002`, pukhta `003`, item `004`, packing `005`, staff `006`.

R2. **Code = type number + a running number from 0, written with no padding.** Sellers: 0010, 0011 … 0019, 00110, 00111 … Buyers: 0020, 0021 … 00210. Items: 0040, 0041 … Packings 0050 …, staff 0060 … No upper limit. The next code is the highest running number of that type + 1. A code is never reused. The code box on a form is filled by itself and stays editable to a free code of the same type (as today).

R3. **Pukhta.** It keeps today's meaning (Umar: a rare buyer who pays on the spot or later; kept per person by name and phone, PRD 13). Only its codes change: pukhta buyer account **0030**, pukhta seller account **0031**. No person becomes an account. (Lead's reading of Umar's answer; flagged to him.)

R4. **Other accounts keep their codes.** Cash 01010001, bank, the payables, income, expense, capital, drawings, 0604, نقدی منڈی 03040000, اپنا مال 030119998, money lent (0103…), separate khatas (0305…), gardens (0306…). Cities, addas, marks, stores, narrations and expense heads keep their codes. The chart of accounts still shows every account.

R5. **Short code.** In a seller box, the running number alone finds the seller: `0` → 0010, `11` → 00111. Same for buyers, items, packings, staff in their boxes. The full code also works. In a box that takes any account (journal), a number lists the seller and the buyer with that running number, each with its type (as today). Taken on Enter or leaving the box, as today.

R6. **Accounts list (Index → Accounts).** Shows only sellers, buyers and the two pukhta accounts. Type filter: All, Seller (بیوپاری), Buyer (گاہک), Pukhta (پختہ کھاتہ). Money Lent and Other are gone from the filter. Cash, bank, payables, income, capital, نقدی منڈی, اپنا مال, money-lent people, khatas and gardens never show here. Footer totals are over what the list shows. The new-account form offers only Seller and Buyer (money-lent people and khatas are still added from their own screens with F4).

R7. **Pickers open on click.** Every picker (sellers, buyers, any account, items, packings, staff, cities, addas, marks, stores, narrations, expense heads, partners, lots, purchases, gardens, trips): a **mouse click** on the box opens its dropdown with all records of that box's type: recent first, then the rest by code. Typing filters by code (full code, start of code, or running number) **or** by name (Urdu, English, tag, phonetic), as search does today. Arriving by keyboard (Enter, Tab, arrows) does **not** open the list, so PRD 7.2 is kept: ↑ ↓ still move between fields, and typing or `Alt+↓` opens the list. Long lists draw only what is on screen and load more on scroll (B31, F34).

R8. **No match: a button to add one.** When the list is empty or nothing matches, the dropdown shows a button, for example "نیا بیوپاری بنائیں" (Add a new seller) or "نئی قسم مال بنائیں" (Add a new item). It opens the Index form for that type, with the name already filled with what was typed. After Save (or Esc), the app returns to the same screen. Everything typed is still there. The new record is not filled in by itself; the user picks it (it is now in the list). F4 quick add stays as it is.

R9. **Clean data.** A tool makes a clean data folder for Umar's app: the owner keeps the same user name and password, the firm details stay (name, mandi, address, phones, contacts, trade mark), and there are no partners (his data has none today). No sellers, buyers, items, packings, staff, cities or entries. Only the system accounts and the starting rules (6%, 6%, لاگا 8 on fruit). Days: none closed. The old folder is kept beside it (`…old-<time>`), never deleted. The first seller then gets **0010**.

R10. **Nothing else changes.** No money rule, posting, formula or golden number changes. Tests that check a code change to the new code (PRD 1.8 rewrites the codes in Sections 4.3, 4.10, 10 and 27; every amount stays).

## 3. Worked examples (for tests)

| Case | Result |
|---|---|
| First seller saved | 0010; second 0011; eleventh 00110 |
| First buyer, first item, first packing, first staff | 0020, 0040, 0050, 0060 |
| Seller box, type `0` + Enter | the seller 0010 |
| Seller box, type `11` + Enter | the seller 00111 (if it exists), else "no seller has this code" |
| Journal line, type `1` + Enter | list: seller 0011 and buyer 0021, each with its type |
| Click the buyer box of a sale line with 3 buyers | dropdown with the 3 buyers, recent first |
| Type `arsh` in it | only Arshad Multani |
| Type `xyz` in it | empty list and the button "نیا گاہک بنائیں"; pressing it opens Index → New account, type Buyer, Urdu name box holds `xyz`; Save → back to the sale with every line still there |
| Worked day with new codes | Rahim 0010, Karim 0011, Akram 0020, Arshad 0021, Tariq (E2E-23) 0022, حاجی نذیر (E2E-42) 0012; every amount of PRD 4 and 27 unchanged |

## 4. Contract (from `backend` T1, 27-10-2026; `pnpm gen:types` wrote 668 files)

1. **Codes** (`mandi_core::codes`): type numbers seller 001, buyer 002, pukhta 003, item 004, packing 005, staff 006. Code = type number + running number from 0, no padding, at most 10 digits. Functions `build_code`, `parse_code`, `parse_running`, `running_of`, `next_running`, `check_typed_code`, `short_code_of_party`, `code_sort_key`. Pukhta buyer 0030 (running 0), pukhta seller 0031 (running 1). A full code starts with `00`, so it is never taken for a short code. A short code is a running number with no leading zero (`0`, `11`). `short_code_of(code)` gives the running number for sellers and buyers, none for pukhta and others (reports use it).
2. **Accounts list:** `AccountFilter { kind: AccountListKind, text, include_inactive }` (the old `account_type` field is gone). `AccountListKind` = all, seller, buyer, pukhta. "all" = regular sellers (control 03011, not system) + regular buyers (control 0304, not system) + the two pukhta accounts, nothing else. The footer is over the same rows. Backend `AccountType` is unchanged (seller, buyer, money_lent, other); the Accounts form offers Seller and Buyer only.
3. **Next code:** `accounts_next_code` gives 0010, 0011 … and 0020 … for seller and buyer; other types unchanged (control code + 4 digits). `masters_next_code` gives 0040 …, 0050 …, 0060 … for item, packing, staff; other masters unchanged. Staff gets a `code` column (migration 0007), `StaffInput.code`, `StaffRow.code`.
4. **Errors** (all on `code`): `ERR_CODE_INVALID` (wrong type, padded, letters), `ERR_CODE_TAKEN`, `ERR_SERIES_FULL` (numbers used up). `ERR_CODE_RESERVED` is no longer returned for sellers and buyers; `resolve_code` never returns "reserved".
5. **Search:** `SearchMastersInput` and `SearchAccountsInput` gain `offset`. Empty text = the whole list: the records chosen last first, then every other active record by code, pages of at most 50 (a page shorter than `limit` is the last). With text: exact code, running number, start of code or running number, names, used before, close spellings. `search_recent` and `search_accounts_recent` order the rest by code. `search_global` with empty text returns nothing. `resolve_code`: a full code exactly; a running number as a short code (found, several, unknown with `suggested_code`, inactive).
6. **Migration** `0007_simple_party_codes.sql` + a Rust hook (new file; no old migration edited): renumbers non-system sellers and buyers, the two pukhta accounts (03049999 → 0030, 030119999 → 0031), items, packings and staff, keeping their order; sets `accounts.serial` to the running number; lifts the 0 to 9999 limit on `accounts.serial` (same schema edit as migration 0006); recreates the trigger `accounts_system_fixed`. No amount, journal line, id or chain hash is touched.
7. **Order by code:** one SQL helper (type number, length, code), so 00110 sorts after 0019 (D3).

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, enum, command changes as stubs or real, migration file, `pnpm gen:types`, hand-off to `frontend` and lead | backend | |
| T2 | Logic: code builder, next code per type, short codes, list filter, empty-text search, migration, clean-data tool; update every backend test and fixture that checks a code (amounts never change); `golden_day` must pass with only codes changed | backend | T1 |
| F1 | Pickers open on click (R7) and the no-match button with return (R8) in `shared/ui` (AccountPicker, Picker, PickerBase, ControlAccountPicker, master pickers), for every screen that has a picker; Accounts list filter and form types (R6); Fluent ur and en | frontend | T1 |
| F2 | Wire to T2; update every frontend test and fixture with codes; keyboard tests prove PRD 7.2 still holds (keyboard arrival does not open the list); language switch; screenshots of the picker open and of the no-match button in both languages | frontend | T2, F1 |
| L1 | PRD 1.8, Architecture, Code Standards, NFR, Tech Stack and money-agent text for the new codes; `check_docs.py` | tech-lead | |
| L2 | Run the clean-data tool on Umar's folder (after he says yes), restart the app, check the first seller gets 0010 | tech-lead | T2 |

Builders run only targeted checks; the full suites stay for the single final gate (Umar's rule).

## 6. Acceptance checklist

- R1 to R10 each shown by a test name or a screenshot.
- `golden_day` passes; no amount in any golden test changed (diff shows only code strings).
- Keyboard-only runs still pass (PRD 7.2): Enter moves through a sale line without opening a list.
- Accounts list shows no cash, bank, payable or other system account; the filter has exactly All, Seller, Buyer, Pukhta.
- A picker opened by click on every screen that has one (list of screens in the hand-off).
- No-match button returns to the same screen with nothing lost, on the بکری screen and on the voucher screen at least.
- Clean-data tool keeps the old folder and the owner's password.

## 7. Out of scope

Pukhta persons as accounts. Renumbering cities, addas, marks, stores, narrations, expense heads or system accounts. Any money change.

## 8. Umar's answers (27-10-2026)

1. Codes grow: 0010 … 0019, 00110 (his pick, with the preview "short code 0 = 0010, 11 = 00111").
2. Pukhta: "a user [who] rarely comes to buy the things; may be he can give the cash on the spot or can give later on against the goods he buys". Lead's reading: today's pukhta, codes 0030 and 0031 (R3). To confirm with Umar.
3. Clean data: keep sign-in and partners (his data has no partners), remove all sellers, buyers, items, packings, staff and entries.
4. No-match button: go to the Index; the entry is kept and comes back.

## 8b. Decisions of the lead

D1 (27-10-2026, from `frontend`'s F1 question). An entry screen keeps its unsaved form in its own state, and leaving the route asks "save changes?". So the R8 button does not change the route. It opens the real Index form (the same AccountForm or MasterForm, same title, the main menu showing انڈکس) full screen over the work area. Save or Esc closes it. The screen underneath never unmounts, so nothing typed is lost. After Save, the list re-opens with the new record visible. Focus goes back to the picker that opened it. To Umar this is the Index form, as he chose. It is not the "small form" he declined. He is told and can still ask for a real route change.

D2 (27-10-2026). A code has at most 10 digits, so each type can have 9,999,999 records. This is no limit in practice; PRD FR-10.3 says so.

D3 (27-10-2026). Lists sorted by code use type number, then length, then code, so 0019 comes before 00110.

## 9. Open questions

- R3 reading of pukhta (above).
