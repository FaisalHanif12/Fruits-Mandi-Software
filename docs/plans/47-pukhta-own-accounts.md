# Plan 47: a پختہ party's own account

Version 1.0, 01-11-2026. Lead: tech-lead. Asked by Umar on 01-11-2026 ("while adding the account there is no option of پختہ کھاتہ buyer or seller"). Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`. It replaces plan 45's "keep the hint only" choice; the hint text changes (F1).

## 1. Goal

The account form gets two more types, پختہ کھاتہ گاہک and پختہ کھاتہ بیوپاری. Such a party is a full account of his side, with his own code in the پختہ series (0032 on), his own ledger, and cash first. The shared پختہ accounts 0030 and 0031, with the name typed on the line, stay for walk-ins.

## 2. Requirements

PRD v1.15 (approved by Umar, 01-11-2026): FR-13.11 (new), glossary پختہ کھاتہ, FR-10.2 code table, 10.2 account type, FR-10.5, E81, A79. Also kept: FR-13.1 to FR-13.10 (shared accounts unchanged), FR-7.38, FR-7.39, FR-7.40, FR-7.69 (quick add), FR-10.3, FR-10.7, FR-10.9 to FR-10.11, FR-12.5 (no name required on his lines), FR-13.6 (R9), FR-19.9.

Umar's answers: keep both ways; one 003 series shared by buyers and sellers; cash first.

No money rule, posting or golden number changes. A پختہ party's lines post exactly like a buyer's or seller's of his side.

## 3. Worked example

نذیر, پختہ کھاتہ گاہک, phone 0300-1111111, code 0032. A sale line to him of 10 پیٹی سیب @ 1,200: value 12,000, buyer commission 720, لاگا 80, he pays 12,800. The line starts as نقدی (credit off). Journal as any نقدی line (FR-12.9): نذیر بنام 12,800 then روکڑ بنام 12,800 / نذیر جمع 12,800. His ledger shows the purchase and the payment, balance 0. Short code `2` in a buyer box finds him; in a seller box it does not.

A پختہ seller کمال, code 0033: his lot's نقد starts as the lot's صافی بکری.

## 4. Contract (T1 by backend; the lead writes it here when agreed)

Backend proposes in T1, before T2, and messages the lead and frontend:
- How the account is marked. It must NOT be `is_pukhta_khata = 1`, which means "shared account, person typed on the line" everywhere today. Expected: a new column (for example `is_pukhta_party`) in a new migration 0013, party_type stays seller or buyer, control stays the seller or buyer control. Backend may propose better.
- `AccountType` gains `pukhta_buyer` and `pukhta_seller` (or an equivalent the frontend can show as two types). `accounts_next_code` for them gives the next 003 running number from 0032, never reusing 0030 or 0031.
- `AccountRow` and `AccountHit` carry the side and the پختہ mark so the list, picker row and party card can show them.
- The پختہ list filter (`AccountListKind::Pukhta`) returns the shared accounts and the own accounts.
- Short codes: a 003 running number resolves in a buyer box only to a پختہ buyer, in a seller box only to a پختہ seller, and also as before for the shared ones.
- Defaults: credit allowed off for a پختہ buyer on create. Seller نقد = صافی بکری on a lot whose seller is a پختہ seller (as FR-13.10).
- R9 lists their lines next to the typed people; their rows show the account code and name.
- Error codes and fields for anything new.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract above: types, migration, command stubs or changes, `pnpm gen:types`, fixtures. Message lead and frontend. | backend | |
| T2 | Logic, migration 0013 tested on a copy of real-shaped data, search index, short codes, R9, defaults. Money agent before and after (the نقد default and posting path). Tests: create both types, codes 0032 and 0033, side-only short codes, a sale and a lot posting like a buyer and seller, cash-first defaults, R9 rows, list filter. golden_day, assist, search, pickers_all, lot_seller_list, print_data must pass. | backend | T1 |
| F1 | Account form: the two types, their fields (as a buyer or seller, credit starting off for a پختہ buyer), the side shown in the list, picker rows and party card. Replace the plan 45 hint with: "ایک بار آنے والا گاہک یا بیوپاری بکری کی لائن پر پختہ کھاتہ چن کر نام اور فون سے لکھا جاتا ہے۔ جو بار بار آئے اس کا یہاں پختہ کھاتہ کھاتہ بنائیں۔" / "A one-time buyer or seller is typed on the sale line under Pukhta Khata, with name and phone. Make an account here for one who comes again." (glossary terms). Quick add (FR-7.69) offers the two types where an account is added. Both languages. | frontend | T1 |
| F2 | Wire to the real data; lot نقد default and line نقدی default on screen; tests for the form, list filter, picker rows, short codes by side, quick add; screenshots ur and en at 1366 x 900. | frontend | T2, F1 |

Targeted checks only (Umar's rule).

## 6. Acceptance

- New account form offers بیوپاری, گاہک, پختہ کھاتہ گاہک, پختہ کھاتہ بیوپاری; the first پختہ account gets 0032.
- Section 3 numbers on screen and in the ledger.
- The shared 0030 and 0031 still work by typed name, unchanged.
- Golden tests unchanged.

## 7. Out of scope

Changing a typed person into a پختہ account (make regular stays as it is); moving old typed lines.
