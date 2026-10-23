# Plan 17b: one cash account and lists page by page on the screens already built

Version 1.0, 23-10-2026. Lead: tech-lead. Brief: `docs/deliverables/D4R-refresh-v1.6.md` Section 2 "17b-refresh-built-screens" (it holds the full text; this file adds the task split and the contract). Branch: `feature/refresh-v16`, made from `feature/bikri` after plan 17 is committed.

## 1. Goal

After this plan no screen, command, print or Fluent file has a payment mode, a bank or a cheque (the one choice kept is cash or salary deduction on ادھار رقم واپسی). Every voucher posts روکڑ. Every long list built in plans 05 to 16 opens at once with its first page and its totals over the whole filter.

## 2. Requirements

PRD 15.1, FR-7.67, FR-10.4 (01010001 "Cash", 01020001 never offered), 10.3 (no bank master), FR-10.9 (no opening on 01020001), FR-16.1, FR-19.11, E70; NFR-SPD-24; Code Standards B31, F34, F44; Architecture 15.4 and 15.7. Acceptance: D4R-A1, D4R-A2, D4R-A3 (D4R file Section 7).

## 3. Worked numbers

No new money number. Every golden and assist expected number of the D3+D4 run stays exactly as it is (D4R-A2). Paging: for any list, Σ of every page's rows equals the totals row that comes with the first page.

## 4. Contract (backend T1 writes the exact types; the lead copies them into Section 10)

- Voucher inputs (`vouchers_save_receipt`, `_payment`, `_advance`, the batch, edit, preview, cancel): `mode`, `bank_account_id`, `cheque_no`, `cheque_date` are removed. An old client that still sends them gets a field error `ERR_FIELD_NOT_ALLOWED` (new code if none exists; field = the name sent) and nothing is posted. Old saved rows keep their columns and still show and print as saved.
- Masters: the bank master leaves `masters_*` commands and the list of masters. 01020001 is left out of every picker and search; an opening on 01020001 answers `ERR_OPENING_NOT_ALLOWED` on the opening field.
- Every built list command takes `{offset, limit}` (limit 0 = 100) and returns `{rows, total_count, totals}` over the whole filter, like `accounts_list`: `bikri_list`, `bikri_stock_list`, `masters_list`, `vouchers_list`, `pukhta_register`, `rules_list`, `advances_report`, `translit_pairs_list`, and any other built list command the backend finds by reading `backend/app/src/commands` (it lists them in its hand-off).

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: new input and output types, the paged list shapes, refusal code, stubs where logic changes, `pnpm gen:types`, command tables in the app tests | `backend` | |
| T2 | Logic: vouchers cash only (domain, services, db reads still show old rows), bank master and 01020001 rules, paging of every built list, tests (plan 15 tests rewritten for cash only, refusal test, paging tests), fixtures, a benchmark for NFR-SPD-24 | `backend` | T1 |
| F1 | Voucher screen without mode, bank and cheque (`PaymentModeFields` removed), the receipt slip without a mode line, the bank master gone from انڈکس, Fluent messages no longer used removed in both languages, tests updated | `frontend` | T1 |
| F2 | Every built list screen on the paged grid of `shared/grid` with the backend totals, against T2 fixtures; keyboard and language-switch tests kept | `frontend` | T2 |

## 6. Acceptance checklist

D4R-A1 (no mode, bank, cheque anywhere; old rows still show), D4R-A2 (every golden and assist number unchanged), D4R-A3 (every built list pages with whole-filter totals). Checks the builders run before hand-off: targeted tests while working; at the end of the plan `cargo fmt`, `clippy`, the touched crates' tests, `golden_day`, `assist`, and for the frontend `pnpm lint`, `pnpm typecheck`, `pnpm i18n:check` and targeted vitest. The whole-suite run and `pnpm perf` happen once at the final gate (Umar, 23-10-2026).

## 7. Out of scope

Gardens (17c), fruit (17d), the tag and prints (17e), reports and analytics paging (D5).

## 8. Open questions

None.

## 10. Contract as built in T1

(To be written by the lead from the backend T1 hand-off.)
