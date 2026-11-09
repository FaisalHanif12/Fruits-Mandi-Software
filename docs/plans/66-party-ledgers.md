# Plan 66: Feature 6, party ledgers (پارٹی کھاتے)

Deliverable D8, item 6. Requirements: PRD v1.32 FR-12.43, FR-5.3, R13, R35 (FR-19.12), FR-19.11, NFR-SPD-24. Decision L1 in plan 60. New module `party_ledgers`.

## 1. Goal

A new item in the بکری side menu, above باقی مال. Level 1: buyers or sellers, one row per party with his balance. Level 2: one row per date with an entry. Level 3: the full day detail, which prints as a complete day slip.

## 2. Worked numbers (after Events 1 to 5)

Buyers: اکرم ٹول ٹیکس 2,880 بنام; ارشد ملتانی 18,684 بنام. اکرم on 10-10-26: previous 0, charged 12,880, paid 10,000, balance 2,880. His day: کیلا, پیٹی, 20 @ 600 = 12,000; commission 720; لاگا 160; day's total 12,880; paid 10,000; new balance 2,880. Sellers likewise (رحیم on 10-10-26, previous 15,000 بنام, net 24,316, paid 9,316, balance 0). Every figure equals the party card and R13.

## 3. Contract

| Command | Input | Output |
|---|---|---|
| `party_ledgers_list` | `{ side: 'buyer' \| 'seller', text: string \| null, offset, limit }` | page of `{ account_id, code, name_ur, name_en, tag, balance_paisa, side }`, count, totals |
| `party_ledgers_statement` | `{ account_id, from: string \| null, to: string \| null, offset, limit }` | page of `{ date, previous_paisa, charged_paisa, paid_paisa, balance_paisa }` (balances signed, with side) |
| Level 3 and its print | the existing R35 slip command and print route for one party and one date | |

چکتا buyers and the 0030 account are not listed. A person in the dead amount shows his balance, marked ڈوبی (FR-10.10). Queries are indexed and paged (B31), no totals from a page.

## 4. Tasks

T1 contract and `pnpm gen:types` (backend); T2 services, queries, golden E2E-67, parity with R13 and the party card (backend); T3 the screen in `features/party_ledgers`, keyboard, four states, both languages, side menu item (frontend); T4 wire (frontend).

## 5. Acceptance (D8-A9)

Three levels and the print with the numbers of Section 2, equal to the party card and R13.
