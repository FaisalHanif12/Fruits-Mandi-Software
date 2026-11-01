# Plan 45: a پختہ کھاتہ hint on the new account form

Version 1.0, 01-11-2026. Lead: tech-lead. Asked by Umar on 01-11-2026. Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`. Owner: `frontend`.

## 1. Goal

Umar opened the new account form to add a one-time party and found only بیوپاری and گاہک in the type list. That is by design: a one-time person is never an account. He is typed on the بکری line (or a voucher) under پختہ کھاتہ, by name and phone (PRD 13, FR-13.5, FR-7.29). Umar chose to keep it that way and to add a short hint, so the user knows where to go.

## 2. Requirements

- PRD 13 (FR-13.2, FR-13.5, FR-13.7, FR-13.8), FR-7.29, FR-7.1 (on-screen hints), FR-7.46 (helpful text).
- Plan 32 R6 stays: the form offers only Seller and Buyer.
- No PRD change. No money, posting or data change.

## 3. What to build

R1. Under the type field of the **new account** form (not the edit form), a short grey note in the shared hint style:
- ur: "ایک بار آنے والے گاہک یا بیوپاری کا کھاتہ نہیں بنتا۔ بکری کی لائن پر { -pukhta-khata } چن کر اس کا نام اور فون لکھیں۔"
- en: "A one-time buyer or seller does not get an account. On the sale line, choose { -pukhta-khata } and type his name and phone."
- Use the glossary terms (`-pukhta-khata`, and `-bikri`, `-gahak`, `-beopari` if they fit). Exact Urdu as above, adjusted only to use glossary terms. Mark it review pending like other new text.

R2. Both languages, both directions. It must not push the form into a sideways scroll at 1024 x 600.

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| F1 | R1 and R2, Fluent ur and en, a component test that the note shows on the new form and not on the edit form, screenshots ur and en at 1366 x 900 | frontend |

Targeted checks only: typecheck, eslint on the changed files, `pnpm i18n:check`, the one test file.

## 5. Acceptance

- New account form shows the note in Urdu and in English. The edit form does not.
- Type list still offers only بیوپاری and گاہک.
- i18n check clean.

## 6. Out of scope

Adding a پختہ کھاتہ person from the account form (Umar declined, 01-11-2026).

## 7. F1 as built (01-11-2026), accepted

- The note is the shared Field `hint` on the type field, only when the form is new. It also shows on the new-account form opened from a picker; accepted, that is a new account too.
- English uses plain lower-case "buyer", "seller", "sale" mid-sentence, as in accounts-help; only Pukhta Khata comes from the glossary. Accepted.
- Checks: typecheck clean, eslint and prettier clean, i18n:check passed, account-form-pukhta-hint 6 passed. No sideways scroll at 1366 x 900 and 1024 x 600 in both languages. Urdu text review pending.
