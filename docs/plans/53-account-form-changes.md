# Plan 53: account form changes from Umar's own test (03-11-2026)

Lead: tech-lead. Branch `feature/assistant`, folder `Fruit Mandi Parallel/d5`. PRD v1.22, Architecture v1.17.

## 1. Goal

Umar tested the account form on a closed day and asked for two changes:

1. "It can be add anytime while adding the new khata." A new account must save with its opening balance on any day. Umar chose: the opening posts on its date, even when that day is closed.
2. "Remove that اگراہی کا دن. It can be collected any day." There is no collection day on the account. The اگراہی فہرست lists every buyer who owes.

## 2. Requirements

- PRD FR-10.11, 10.2, FR-6.6, E79 (the opening); 10.2, FR-15.6, FR-7.61 (the collection list).
- Golden tests unchanged: golden_day, E2E-19 (opening set-up), E2E-24.

## 3. Worked numbers

- **Opening on a closed day.** The days up to 03-11-26 are closed. A new buyer طارق is saved on 03-11-26 with an opening of 5,000 بنام.
  - The opening entry is dated 03-11-26: طارق بنام 5,000, ابتدائی فرق (06019999) جمع 5,000.
  - The day's trial balance still balances. ابتدائی فرق moves by −5,000, and that is correct: his other side is in the old books.
  - The same opening dated 04-11-26 (after today) is refused.
- **Changed opening.** Changing an existing account's opening, or clearing it, on a closed day is still refused with ERR_DATE_CLOSED.
- **Collection list.** After Events 1 to 5 it shows اکرم 2,880 and ارشد 18,684. The list total, 21,564, equals R20 when no filter is set.

## 4. Contract

- **Account save (`masters_save_account` and its preview).**
  - For a NEW account (no id), `opening_date` must be a real date not after today. A closed day is allowed when there is an opening amount.
  - For an existing account, nothing changes: an unchanged opening is never checked again, and a new or changed opening needs an open date (ERR_DATE_CLOSED, with `today` and `today_open` as now).
  - The closed-day opening of a new account writes an audit row with reason `opening on a closed day` (S7).
- **`collection_weekday`.**
  - It is removed from the account input type. The column stays in the database, unused (Architecture 15.4; a released migration is never edited). A new account writes NULL.
  - Any old value is ignored everywhere.
- **`assist_collection_list`.**
  - The input loses its weekday. It keeps `collector_staff_id?`, `city_id?` and the page.
  - Rows are every active buyer whose balance is بنام more than 0: no پختہ کھاتہ, no نقدی منڈی, and none in the dead amount. Each row has the same fields as now.
  - The totals come over the whole filter (B31).
- Run `pnpm gen:types`. backend tells frontend2 the exact changed types.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | The opening rule (core prepare, service posting, audit), with tests. Money agent before and after. | backend | its current safety fixes |
| T2 | The collection list with no weekday; weekday removed from the account input; gen:types | backend | T1 |
| T3 | Remove the اگراہی کا دن field. Fix the closed-day opening message so it never asks for a date after today. Collection list screen and print: no day. Tests in both languages. | frontend2 | T2 types |

## 6. Acceptance

- A new account with an opening on a closed day saves, its entry is dated that day, the trial balance balances, and an audit row exists.
- A changed opening on a closed day is refused. A future opening date is refused.
- The account form has no اگراہی کا دن in Urdu or English.
- The collection list shows every buyer who owes, by collector and city, with no day picker.
- golden_day, E2E-19 and E2E-24 pass, with no golden number changed. i18n:check, typecheck and lint are clean.

## 7. Out of scope

Changing an existing account's opening on a closed day. Dropping the database column.
