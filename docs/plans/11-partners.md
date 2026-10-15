# Plan 11: Partners and the live split

Deliverable: D3, plan file 11. Branch: `feature/bikri` (goal run D3+D4). Version 1.0, 15-10-2026.

## 1. Goal

Partners exist with their shares, and every posting that touches an income or a shop expense account writes one share row per partner in the same save, whatever module it comes from. The partner setup screen, the wizard's partners step and the partner overview work in Urdu and English. After this plan a بکری can split its income (plan 13) without any new split code.

## 2. Requirements

PRD 17.1 to 17.3 (FR-17.1 to FR-17.5), FR-6.5, FR-2.4, FR-7.24, FR-7.66, 4.2 (الف 40, ب 30, ج 30), C9, C10, C11, C12, E9, E18, A31. NFR-INS-2 (partners step), NFR-SCL-5 (up to 10 partners), NFR-SEC-5 and Code Standards S7 (audit), NFR-SPD-9. Architecture 10 and 15.5. The money-calculations agent (C10, C11, Sections 5 and 8). Acceptance: D3-A1, D3-A2.

## 3. Worked numbers (the tests use exactly these)

Shares الف 4000 bps, ب 3000, ج 3000. A part is `floor(amount × bps ÷ 10,000)`, the leftover goes to the largest share (tie: the partner with the lowest id, created first).

| Case | Working | Rows |
|---|---|---|
| Income 3,948 (E2E-4) | 1,579.2 / 1,184.4 / 1,184.4 → 1,579 + 1,184 + 1,184 = 3,947, leftover 1 | 1,580 / 1,184 / 1,184 |
| Income 1,540 (E2E-5) | 616 / 462 / 462, leftover 0 | 616 / 462 / 462 |
| Expense 3,000 | 1,200 / 900 / 900 | −1,200 / −900 / −900 |
| Expense 3,384 (E2E-20) | 1,353.6 / 1,015.2 / 1,015.2 → 1,353 + 1,015 + 1,015 = 3,383, leftover 1 | −1,354 / −1,015 / −1,015 (sum −3,384) |
| Negative income −300 | split 300 as above: 120 / 90 / 90, then negated | −120 / −90 / −90 |
| One entry with income and cost (E2E-34) | income 9,660; cost 7,800 | income 3,864 / 2,898 / 2,898; cost −3,120 / −2,340 / −2,340 |
| Shares not 100% | 4000 + 3000 + 2000 | `ERR_SHARES_NOT_100`, total 9,000 bps, nothing saved |
| Share change on 20-10-26 to 5000 / 3000 / 2000 | a posting dated 10-10-26 keeps 40 / 30 / 30; a posting dated 20-10-26 splits 50 / 30 / 20 | |

The shares of a date D are, for each partner, his latest share row with `effective_from` on or before D (same date: the row saved last). Postings already saved keep their rows (FR-17.2).

## 4. Contract

New error codes (added in `core::error`, with `errors.ftl` messages in `ur` and `en`, `# review: pending`): `ERR_SHARES_NOT_100` (value `total_bps`), `ERR_SHARE_BACKDATED` (value `today`), `ERR_PARTNER_HAS_SHARE` (a partner with a share today or later cannot be made inactive), `ERR_PARTNER_NOT_LISTED` (a share list must name every active partner once; field `shares`), `ERR_TOO_MANY_PARTNERS` (value `max` 10). Existing codes are reused for names, percent text, versions and not found.

### Backend (module `partners`; commands in `app/src/commands/partners.rs`, all need sign-in)

- `partners_list()` -> `Vec<PartnerRow { id, name_ur, name_en, capital_account_id, capital_account_code, drawings_account_id, drawings_account_code, share_bps, is_active, version }>`. `share_bps` is the share on today's date. Active first, then by id.
- `partners_get { id }` -> `PartnerDetail { row, shares: Vec<ShareHistoryRow { share_bps, effective_from }> }`.
- `partners_create { name_ur, name_en, opening_capital: Option<String>, spelling }` -> `PartnerRow`. Same name inputs and spelling confirmation as `masters_save` (plans 05 and 06). Makes the capital account (control 0602, code 0602xxxx) and the drawings account (control 0603, code 0603xxxx) with names in both scripts built from the glossary words (the way plan 03 names the 0103 accounts). A new partner has no share (0 bps) until `partners_set_shares`. The opening capital, if given, is posted as the opening credit of his capital account dated today (FR-10.9, through the accounts opening service). Errors: name codes, money text codes, `ERR_TOO_MANY_PARTNERS`, `ERR_DATE_CLOSED` for the opening.
- `partners_update { id, version, name_ur, name_en, is_active, spelling }` -> `PartnerRow`. Names only change the partner and his two accounts' names. Making a partner inactive needs his share to be 0 from today (`ERR_PARTNER_HAS_SHARE`). Errors: `ERR_VERSION_CHANGED`, `ERR_NOT_FOUND`.
- `partners_set_shares { effective_from: String, shares: Vec<ShareInput { partner_id, share: String }> }` -> `Vec<PartnerRow>`. `share` is percent text with up to 2 decimals (`core::money::parse_bps`). Every active partner appears once. The total must be exactly 10,000 bps. `effective_from` is today or later (`ERR_SHARE_BACKDATED`, E18, A31); this also applies to the first set. One row per listed partner is written in one transaction, with one audit row (old and new shares, S7). Field errors point to `shares[i].share`.
- `partners_check_shares { shares }` -> `SharesCheck { total_bps, is_100, errors }`. Read-only. It exists so the screen shows the running total without adding percents itself (rule 1). Same parsing as the save.
- `partners_overview()` -> `Vec<PartnerOverviewRow { partner_id, name_ur, name_en, share_bps, today_income_paisa, month_income_paisa, year_income_paisa, month_expense_paisa, month_profit_paisa, month_taken_paisa, capital_balance_paisa }>`. "Month" and "year" are the calendar month and year up to today on the injected clock. Income is the sum of the partner's income rows. `month_expense_paisa` is a positive number (1,200 for الف in the worked day), and `month_profit_paisa` is income minus expense (C12). `month_taken_paisa` is the debit total of his drawings account in the month (opening debits included from D4). `capital_balance_paisa` is the credit balance of his capital account. Money travels as paisa numbers (ts-rs `number`). The returned, خالص, capital invested and کل رقم columns come in D4.
- Domain (`domain::partners`, pure): `split_posting(shares_on_date, income, expense) -> Vec<ShareRowDraft { partner_id, kind, base, share_bps, amount }>` using `core::money::split_by_bps` only. A negative amount is split as positive and negated at the end (M9, C11). Shares not totalling 10,000 bps give `SharesNot100`. Pure, no database. The same function serves `bikri_preview` and `bikri_save` later (B5).
- The hook (services, `ledger::post`): after an entry is posted, `ledger::post` works out the entry's income base (Σ credit − Σ debit on head 04 lines) and expense base (Σ debit − Σ credit on head 05 lines) and writes the share rows for each base that is not zero, with the shares of the entry date. No module writes share rows itself. Not done for posting types `opening` and `month_close` (month close only moves amounts, FR-17.6). A reversal copies the original entry's rows with the opposite sign (base and amount), not a new split (FR-17.4). If shares do not total 100% on the date, the post is refused with `ERR_SHARES_NOT_100` and nothing is written. Existing tests that post to head 04 or 05 get a partners fixture; no expected number changes.
- Table changes: none expected (`partners`, `partner_shares`, `partner_share_rows` exist in migration 0001). If one is needed, it is a new numbered migration, agreed with the lead first.
- Property tests (Code Standards 9.3): split parts always add up to the amount; every posting's rows sum to its income base minus its expense base; a reversal plus its entry gives zero per partner.
- Partners do not log in (FR-2.4): no user is made for a partner.
- `pnpm seed` creates الف 40, ب 30, ج 30 with Urdu and English names (Partner A / B / C), effective from the injected "today" of the opening position, through these services.
- Money agent: `backend` calls it before writing the split and the hook, and again to review them; the report is attached to the hand-off.

### Frontend

- Route `/partners` (replaces the coming-soon placeholder): the partner overview page (cards or rows, columns of 17.3 that exist in D3) with a link to the setup page. Setup page: the list of partners with a share input each (percent text), the running total from `partners_check_shares` (green at 100, red otherwise), "effective from" date, add partner (dialog with `NamePair` and `RomanUrduInput`, opening capital with `MoneyInput` and a hint that either partner openings or the firm's ابتدائی سرمایہ is used, not both), edit names, make inactive. The save button stays disabled until the total is 100 (the backend checks again).
- The wizard's partners step (NFR-INS-2): after the Owner is created, the same setup form with الف, ب, ج as empty rows to fill; "later" is allowed (a بکری then refuses with `ERR_SHARES_NOT_100`, E9). The auth feature imports the form only through `features/partners/index.ts`.
- Fluent: `partners.ftl` in `ur` and `en`, the new error messages, the glossary words (شراکت دار, حصہ, سرمایہ). `# review: pending` on each new Urdu line.
- Four states on each page, keyboard only (arrows between rows, `F10` save), both directions, 1024 × 600 with no sideways scroll, language switch in the middle of an edit loses nothing (form state above the screen).
- Design: `/design` is not available in this session. Fallback of README Section 5: `frontend` designs inside `docs/design/design-system.md` with the `frontend-design` plugin and the `bilingual-screen` skill, and notes any change in `docs/design/`.

### Contract as built in T1 (agreed 15-10-2026, recorded by the lead)

- Every command except `partners_list` and `partners_overview` takes one `input` object.
- `ERR_SHARES_NOT_100` already existed. New codes: `ERR_SHARE_BACKDATED` (value `today`), `ERR_PARTNER_HAS_SHARE`, `ERR_PARTNER_NOT_LISTED` (field `shares`), `ERR_TOO_MANY_PARTNERS` (value `max` = 10). Their `errors.ftl` messages (ur and en, `# review: pending`) are written by `frontend`.
- The name spelling confirmation is the masters one: `confirmed_rules_words: string[]` (default empty), not a `spelling` field. `PartnerCreateInput` is `{ name_ur, name_en, opening_capital: string | null, confirmed_rules_words }`. `PartnerUpdateInput` is `{ id, version, name_ur, name_en, is_active, confirmed_rules_words }`.
- `share_bps` is a plain number (4000 = 40%). `effective_from` in `PartnersSetSharesInput` is DD-MM-YYYY text; `ShareHistoryRow.effective_from` comes back as YYYY-MM-DD, like the other lists.
- `SharesCheck = { total_bps, is_100, errors: LineError[] }` with field paths `shares[i].share`.
- `PartnerOverviewRow` money fields are signed `Paisa`; `capital_balance_paisa` is negative if the capital account stands on the debit side.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: Rust types, command stubs returning `ERR_NOT_IMPLEMENTED`, new error codes, `pnpm gen:types` (done) | backend | |
| T2 | Domain split, the hook in `ledger::post`, services, commands, tests, seed, money agent before and after | backend | T1 |
| T3 | Setup, overview and wizard step against the stubs; Fluent in both languages | frontend | T1 |
| T4 | Wire to the real commands; wizard step end to end | frontend | T2, T3 |

Reviewer and robustness run once at the end of the run (README 4a).

## 6. Acceptance checklist

1. Shares total exactly 100% for every date; a back-dated change is refused; history is kept; a saved posting keeps its split (D3-A1; `cargo test -p mandi-services --test partners`).
2. The worked numbers above pass in domain unit tests and through `ledger::post`: 3,948 gives 1,580 / 1,184 / 1,184; 3,384 expense gives −1,354 / −1,015 / −1,015; the combined entry of E2E-34 gives both row kinds.
3. A reversal writes the exact opposite rows; month close and opening write none.
4. Wizard partners step on a clean install and the overview page work; Windows E2E specs written and syntax-checked, run LOCAL-ONLY (D3-A2).
5. All partners commands are refused before sign-in and while locked, and write nothing.
6. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo clippy --workspace --all-targets -- -D warnings` exit 0; every D1 and D2 test and `golden_day` (E2E-19, E2E-23) still pass.

## 7. Out of scope

Capital in and out, returned and خالص columns, کل رقم and the partner statement (D4 plan 17); month close (D5); the split call from the بکری (plan 13).

## 8. Open questions

None for Umar. Team defaults: a partner is created without a share; the share list names every active partner; up to 10 partners (NFR-SCL-5).
