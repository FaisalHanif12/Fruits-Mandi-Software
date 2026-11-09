# Plan 64: Feature 5, پختہ کھاتہ removed, چکتا کھاتہ added

Deliverable D8, item 5. Requirements: PRD v1.32 glossary, FR-10.2, FR-10.4, Section 13, FR-7.29, FR-7.69, R9; Architecture 15. Decisions C1 to C11 in plan 60 Section 3.2. Money agent review before and after.

## 1. Goal

پختہ کھاتہ is gone from the app. چکتا کھاتہ exists for walk-in buyers: buyer only, added on the spot, pays in full, has no ledger, is found again by name.

## 2. Worked numbers

Worked day Event 2: شاہد is a چکتا buyer of 10 پیٹی سیب at 1,200: value 12,000, commission 720, لاگا 80, paid 12,800. Whole entry 25,600 = 25,600. Shared income 1,540. Every number of the worked day stays. Golden changes allowed: plan 60 Section 3.4 only.

E2E-21 (replaced): شاہد 1,000 and ناصر 2,000, both چکتا, both paid in full: neither has a balance; 0030 ends at 0.

## 3. Contract

| Command | Input | Output | Errors |
|---|---|---|---|
| `chukta_search` (replaces `search_pukhta_people`) | `{ text: string }` | `ChuktaBuyer[]`: `{ id, name_ur, name_en, phone, last_date }`, newest first, at most 10 | |
| `chukta_add` | `{ name_ur, name_en, phone: string \| null }` | `ChuktaBuyer` (an existing person with the same key is returned, not doubled) | name and phone errors as FR-7.28, on `name_ur`, `name_en`, `phone` |
| `search_accounts` scope `buyer` | as today | hits gain `kind: 'account' \| 'chukta'`; a چکتا hit has `chukta_id` and the name; 0030 itself is never a hit | |
| `bikri_preview`, `bikri_save`, edit | `SaleInput.chukta: { name_ur, name_en, phone } \| null` when `buyer_account_id` = 0030 (the old `pukhta` input, renamed). Lots lose `pukhta`. | | `ERR_CHUKTA_NAME_REQUIRED` at `lots[i].sales[j].chukta.name_ur` |

Removed commands: `pukhta_register`, `pukhta_make_regular`, `search_pukhta_people`, and the person picks of vouchers. Permissions files follow. The word list of `assistant` follows in the next section.

Migration `0020_chukta_khata.sql`: exactly decision C9, tested on a copy of real-shaped data, including that `bikri_sales_snapshot_is_final` and the other triggers still work after the column renames, and that every `chain_hash` is unchanged.

D6-system-prompt v1.8 for the engine: `find` role `chukta`, app_help topic `chukta_khata` (also in the answer schema), column enum without `side` and `person` and with `paid` and `left`, R3, R4 and R5 list those columns, or test P8 fails. Its fixed part measures 11,599 tokens, so nothing may be added to it.

Prompt files: copy `docs/deliverables/D6-system-prompt.md` Sections 2, 3, 4 and 6 to `crates/assistant/src/prompt/` word for word (test P7). Prefix under 11,600 tokens.

## 4. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Migration; contract types; `pnpm gen:types` | backend | plan 60 commit |
| T2a | Delete seller-side, party-account, register, make-regular and voucher-person code, with their tests | backend | T1 |
| T2b | Rename the remaining buyer-side plumbing from pukhta to chukta; چکتا rules (paid in full is in plan 65; until then the line is forced cash); search index; party card; golden tests with the wording changes; E2E-21 | backend | T2a |
| T3 | Screens: the buyer picker with چکتا hits and the add row with the short form; PukhtaFields out; register page, menu item, account form types, accounts filter, voucher person pickers out; Fluent `pukhta_khata.ftl` out and `chukta` words in, both languages; the one-time slip template out, buyer slips include چکتا buyers (decision C9a) | frontend | T1 |
| T4 | The repository search of D8-A8 | lead | T2, T3 |

Deletions: only tracked files that belong to the removed feature, which git can always bring back. No file outside the repository is touched (D8 rule 5).

## 5. Acceptance (D8-A8, part of A14)

- `git grep -il "pukhta\|پختہ"` outside old migrations 0001 to 0019, `docs/plans/0*` to `59`, `docs/reviews`, change-log lines, and the rule files' history lines returns nothing that is live.
- A چکتا buyer is added on the spot from the بکری, pays in full, shows in R2, R34, R35; is not in R13, R20, dues, party ledgers.
- `golden_day` and `assist` green with only the listed changes.

## 6. Notes

Contract as built (lead's record, 09-11-2026): `chukta_search { text }` returns `ChuktaBuyer[]` with `{ id, name_ur, name_en, phone, last_date }`. `chukta_add { name_ur, name_en, phone }` returns the existing row for the same name and phone. `search_accounts` gains `with_chukta` (default false, sent only by the بکری line buyer picker); its hits gain `kind`, `chukta_id`, `phone`; a چکتا hit has the id of account 0030. `assist_party_card` takes an optional `chukta_id`. `SaleInput.chukta`; lots lose the person; `reports_find_people` removed. Account 0031 stays as an inactive old closed account in every database (PRD FR-10.4), because the old seed still runs.
