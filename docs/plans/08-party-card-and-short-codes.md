# Plan 08: A code brings the whole party

Deliverable: D2, plan file 08. Branch: the run branch (goal run).

## 1. Goal

In any account field, a full code or a short code brings the account and its party card. Unknown codes offer to add an account without leaving the screen. Global search (`Ctrl+K`) finds accounts.

## 2. Requirements

FR-7.38, FR-7.39, FR-7.42, FR-7.64, FR-10.8, FR-5.4 (accounts part). NFR-SPD-21, NFR-USE-25. Architecture 20. Code Standards F40, B27, B29. Acceptance: D2-A10, D2-A11.

## 3. Worked examples (tests)

| Field | Typed | Result |
|---|---|---|
| buyer | `1` then `Enter` | 03040001 اکرم ٹول ٹیکس, card shown |
| buyer | `0001` | same |
| seller | `7` then `Enter` | 030110007 if it exists, else the unknown-code path for seller 030110007 |
| any account (جنرل) | `1` | two rows: "گاہک 03040001 اکرم ٹول ٹیکس" and "بیوپاری 030110001 رحیم" |
| buyer | `1` then still typing `2` | no account taken until `Enter` or leaving the field; the card previews the top match while typing |
| buyer | `9999` | refused as reserved (system serial), never a short code for a normal account |
| buyer | `03045555` (does not exist) | "اس کوڈ کا کوئی گاہک نہیں" with an F4 add button |

The card in D2 shows: name in the current language with father or firm, city, phone, balance with its side (بنام or جمع) read from the database, last entry date, for a buyer the credit limit and how much is used, for a seller the open advance (from posted rows and advance matching; the payable comes in D4), and warnings with a word as well as a colour (inactive, over credit limit, owes the firm). Under NFR-SPD-21 (100 ms) with 10 years of data (bench with generated data).

## 4. Contract

### Backend

- `search_accounts { scope: "seller" | "buyer" | "any" | control_id, text, limit }` -> hits in the plan 06 order, with exact code first and exact serial for the scope (FR-7.39): in a seller or buyer scope the serial maps to that control; in `any` the seller and the buyer with that serial are both listed with their type. Read-only (B29).
- `accounts_resolve_code { scope, text }` -> `ResolvedCode { status: "found" | "several" | "unknown" | "reserved", account?: AccountHit, candidates?: Vec<AccountHit>, suggested_code? }`, used on `Enter` or leaving the field only.
- `assist_party_card { account_id }` -> `PartyCard { name_ur, name_en, father_or_firm_ur, father_or_firm_en, city_ur, city_en, phone, balance: Paisa, side, last_entry_date, credit_limit?, credit_used?, advance_open?, warnings: Vec<CardWarning> }`; balances from `journal_lines` (never from the index, B27); one indexed query (NFR-SPD-21; bench).
- A timing test like plan 06's over 20,000 generated accounts for the account scopes.
- Recent and touch use the plan 06 commands `search_recent { scope }` and `search_touch { scope, id }`, with `SearchScope` extended by `seller`, `buyer` and `any_account`. No separate account commands.
- Added 12-10-2026: for the account scopes, `search_recent` returns `AccountHit` rows (with `account_type`), so a money lent person is not shown as "other"; the `control` scope has a recent list too.
- `search_global { text }` -> accounts now (later deliverables add their own kinds).
- Sign-in check first in every command.

### Frontend

- `AccountPicker` on top of the plan 06 `Picker`, filtered by scope; short code taken on `Enter` or leaving the field (never while typing); the several-candidates list; the unknown-code message with `F4` add.
- `PartyCard` next to the field, shown as soon as a code is complete (preview of the top match while typing), the warnings with a word and a colour (NFR-USE-12), balances through `Num`; nothing hidden (NFR-USE-27).
- Quick add with `F4` or "+" in any account picker (FR-10.8): a compact account form (plan 07 form fields, `NamePair`) in a dialog, prefilled with the typed code; on save it returns to the field with the new account. From the account form, `F4` on the city or اڈا picker adds a city or اڈا without losing what is typed (FR-7.64).
- Global search in the top bar (`Ctrl+K`) for accounts: results open the account.
- Replace the plain Picker on the account form's linked account with `AccountPicker`.
- Tests in ur and en: every row of Section 3, the F4 path returning to the field, the card in both languages, keyboard only.

### Recorded during the build (12-10-2026)

- `scope` is `"seller" | "buyer" | "any" | "control"` with a separate `control_id`. `accounts_resolve_code` has a fifth status, `inactive`.
- `accounts_balances { ids }` (at most 500) loads row balances after the list is drawn, so typing never waits for journal sums (FR-7.40, NFR-SPD-6). No stored running balance (FR-6.1).
- `PartyCard.balance` is the `Balance` object. `assist_party_card` is in the `assist_` group. Text over 200 characters: `ERR_TEXT_TOO_LONG`; more than 500 ids: `ERR_OUT_OF_RANGE`.
- The two پختہ کھاتہ cards show the balance only (many people inside, FR-13.5).
- Deferred, not lost: the buyer's oldest unpaid bill and its days needs bill matching (D3, with وصولی); the seller's amount payable comes with D4 (FR-7.59). Until advance matching from بکری exists (D3), `advance_open` is given minus adjusted.
- Timing (debug build, Mac): 20,098 accounts, index build 317 ms, slowest account search 74 ms; party card over 10,000 entries 9 ms (NFR-SPD-21 asks 100 ms).

## 5. Tasks

T1 backend contract; T2 backend with bench; T3 frontend; T4 wire.

## 6. Acceptance checklist

1. Short code `1` + `Enter` in a buyer field brings 03040001 and his card; card under NFR-SPD-21 (D2-A10).
2. Quick add with `F4` and the unknown-code path add an account without leaving the screen (D2-A11).
3. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check` exit 0.
