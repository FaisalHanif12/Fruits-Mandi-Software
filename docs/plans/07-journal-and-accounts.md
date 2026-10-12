# Plan 07: The journal, the chart of accounts and accounts

Deliverable: D2, plan file 07. Branch: the run branch (goal run, README 4a).

## 1. Goal

The one way any module writes to the books: a journal entry saved as draft, checked, then posted, never changed afterwards; corrections by reversal. The chart of accounts in three levels, account codes and serial rules, the account form with every field of PRD 10.2, opening balances posted against ابتدائی فرق, the trial balance and ledger queries. E2E-23 passes.

## 2. Requirements

PRD 6 (FR-6.1 to FR-6.6), 10.1 and 10.2 (FR-10.1 to FR-10.3, FR-10.5 to FR-10.7, FR-10.9) v1.2 (account type ادھار رقم, control 0103), FR-7.63, Settings "Codes". NFR-REL-8, NFR-REL-11, NFR-SEC-5, NFR-SEC-6, NFR-SCL-1. Architecture 6, 13, 15.4, 15.7. Code Standards 3, 5.3, 7. E2E-23. Acceptance: D2-A7, D2-A8, D2-A9.

The money-calculations agent is called before writing the journal engine and again to review it (agent Section 7).

## 3. Worked numbers (tests)

- PRD 4.3 openings: روکڑ 01010001 opening بنام 65,000 and ابتدائی سرمایہ 06010001 opening جمع 65,000, both dated 31-08-26: two opening entries against ابتدائی فرق 06019999; after both, ابتدائی فرق is 0 and the trial balance is 65,000 = 65,000.
- A change of an opening (65,000 to 60,000) reverses the old opening entry and posts the new one; the trial balance stays equal at every step.
- E2E-23: buyers 03040001 اکرم ٹول ٹیکس and 03040002 ارشد ملتانی exist; a new buyer typed `tariq mehmood` gets code 03040003, `name_ur` طارق محمود, `name_en` "Tariq Mehmood"; "tarq" finds him in both languages (search index, plan 06).
- Serial rules (FR-10.3): next code is the highest normal serial + 1; 0000 and 9000 to 9999 are refused for a normal account (`ERR_CODE_RESERVED`); after 8999, `ERR_SERIES_FULL`.
- Unbalanced entry (بنام 100, جمع 90): refused `ERR_JOURNAL_UNBALANCED`, nothing written.

## 4. Contract

### Backend

- `domain::ledger`: pure builders `build_entry(lines) -> Result<BalancedEntry>` (each line one side only, positive, whole rupees, Σ بنام = Σ جمع, checked arithmetic), `build_reversal(posted_entry) -> BalancedEntry` (sides swapped, same date by service rule), `build_opening(account, side, amount, date)`.
- `services::ledger::post(tx, PostingEvent)`: inserts the entry as `draft`, its lines, then one UPDATE to `posted` that also writes `total_paisa` and `chain_hash` (keyed blake3 chain over the entry and the previous entry's hash, NFR-SEC-6), so the database balance trigger runs (B16). The posting date must pass `check_posting_date` (FR-6.6, `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE`). This is the only function that writes journal rows; every later module calls it. `verify_journal_chain` finds an edited row.
- `services::ledger::reverse(tx, entry_id, reason)`.
- Queries as service functions (screens in D5): `trial_balance(as_of)`, `ledger(account_id, from, to)` with running balance and side, `account_balance(account_id)` (used by the party card in plan 08), all reading posted rows only (FR-6.1), sums with `SUM()` on `*_paisa` only.
- Chart of accounts: heads, controls and accounts as seeded (plan 03, with 0103 ادھار رقم). Commands `coa_tree()` -> heads with controls with accounts and live balances (FR-10.1); `coa_heads_list`, `coa_controls_list`.
- Accounts: `accounts_save { id?, version?, account_type ("seller" | "buyer" | "money_lent" | "other"), control_id (for other), code?, name_ur, name_en, confirmed_rules_words, father_or_firm_ur, father_or_firm_en, city_id, adda_id, phone1, phone2, address, linked_account_id, opening_date, opening_dr, opening_cr, opening_is_advance, credit_allowed, credit_limit, default_marka_id, collector_staff_id, collection_weekday, is_active, note }` (money as text) -> `AccountRow`. One transaction: validate every field (Code Standards 4.2; script rules; spelling check FR-7.66), code (suggested by `accounts_next_code { account_type, control_id }`, or typed and checked), the search keys, the opening entry (FR-10.9; only one of debit or credit; `opening_is_advance` only for a seller with a debit), the audit row (old and new values), then the search index update after commit. `accounts_set_active`. No delete (FR-10.6). `accounts_list { filter }` -> rows with the columns of FR-10.5 (code, Urdu name, English name, city, opening date, opening debit, opening credit, credit allowed, current balance) and footer totals. `accounts_similar { name_ur?, name_en?, phone? }` -> close matches for the duplicate warning while typing (FR-10.7, FR-7.63), by search keys and phone.
- Openings grid (FR-10.9, typing many openings at once): `accounts_openings_preview { lines: [{ account_id, version, opening_date, opening_dr, opening_cr, opening_is_advance }] }` -> `{ rows (each with its errors), total_dr, total_cr, opening_difference_after: Balance }` (read-only, the same domain function as the save, B5); `accounts_openings_save { lines }` -> the saved `AccountRow`s and the ابتدائی فرق balance. One transaction for the whole grid, all or nothing; each changed line reverses its old opening entry and posts the new one; unchanged lines are left alone; errors point to `lines[i].field`. Audit row per changed account.
- Balance shape on every row and card: `Balance { amount_paisa: Paisa, side: "dr" | "cr" | "zero" }` (frontend's proposal, accepted 12-10-2026). `accounts_get { id }` -> `AccountRow` for the edit form.
- List size (NFR-SCL-1, NFR-SPD-18, decided 12-10-2026): `accounts_list` returns every matching account, up to the 20,000 of NFR-SCL-1, with no 2,000 cap. The grids draw only the rows in view. A timing test with 20,000 generated accounts keeps the call under 300 ms (NFR-SPD-3) in a release build; robustness benches it at the end gate.
- Openings grid design (frontend, accepted): one row per active account, only changed rows are sent, one date for accounts with no opening yet; the advance tick stays on the account form only (PRD 10.2), not in the grid.
- Decided 12-10-2026: a typed money value is limited to 10,00,00,00,000 rupees (Code Standards 4.2, `ERR_AMOUNT_TOO_LARGE`); the wider per-line limit in the journal (10^12 rupees) is an internal safety limit for sums and is never the limit a user meets first. Openings are refused (`ERR_OPENING_NOT_ALLOWED`) on income and shop expense accounts, ابتدائی فرق, نقدی منڈی and the two پختہ کھاتہ accounts; this is a team default waiting for Umar (status.md). Empty opening fields on an edit clear the opening (reversal and audit row), so the form must always send the stored opening back unchanged when the user did not touch it (frontend test).
- Added 12-10-2026: `AccountRow.can_take_opening: bool`, from the same named list the save checks, so the openings grid leaves those accounts out.
- Errors: `ERR_JOURNAL_UNBALANCED`, `ERR_CODE_TAKEN`, `ERR_CODE_RESERVED`, `ERR_SERIES_FULL`, `ERR_OPENING_BOTH_SIDES`, `ERR_ADVANCE_NOT_SELLER`, `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE`, plus the validate codes with field paths. Sign-in check first in every command (one user, PRD v1.2).
- `backend/crates/services/tests/assist.rs` starts here with the backend part of E2E-23.
- Property test: any random mix of openings, changes of openings and reversals keeps Σ بنام = Σ جمع over all accounts and ابتدائی فرق equal to minus the sum of the other openings.

### Frontend

- انڈکس screens: Chart of Account Head, Control and Tree (tree with live balances, expand and collapse with arrow keys, FR-10.1).
- Account list with the FR-10.5 columns and footer totals; filter; show inactive.
- Account form with every field of PRD 10.2: account type (seller, buyer, ادھار رقم, other) setting the control and the suggested code; `NamePair` for the name and for father or firm; city and اڈا pickers (plan 06 Picker); phones; linked account (AccountPicker comes in plan 08: use the plain Picker on accounts until then); opening date, opening debit and credit as money text; the advance tick only for a seller with a debit; credit allowed (ticked by default) and limit; collector and collection day; active; note. Similar accounts show below the name while typing (FR-7.63). Four states, keyboard only, both languages.
- An opening balances grid (entry-grid skill, `shared/grid`) for typing many openings at once: account, debit, credit, opening date; totals and the running ابتدائی فرق from the backend.
- Tests in ur and en: keyboard-only create of the E2E-23 buyer (code 03040003, both names), the opening grid, the tree keys, a language switch in the middle of the form.

## 5. Tasks

T1 backend contract and stubs; T2 backend; T3 frontend; T4 wire. No per-plan review (goal run).

## 6. Acceptance checklist

1. Journal: unbalanced refused and nothing written; posted rows cannot change; reversal works; the keyed hash chain finds an edited row (D2-A7).
2. Chart of accounts in three levels; codes and serial rules of FR-10.2 and FR-10.3 (D2-A8).
3. E2E-23 passes (D2-A9).
4. Property test: any mix of openings keeps the trial balance equal.
5. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check` exit 0.

## 7. Out of scope

Party card, short codes and global search (08), rules (09), vouchers and day close (10), report screens (D5).
