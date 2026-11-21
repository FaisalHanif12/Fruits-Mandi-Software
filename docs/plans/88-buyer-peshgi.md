# Plan 88: پیشگی can be given to a buyer (posted as cash lent)

| | |
|---|---|
| Status | Umar's request and choice, 20-11-2026 |
| Branch | `fix/gardens-new-form` (one pull request for plans 81 to 88) |
| PRD | 15.4 and FR-15.11 (v1.51 line), FR-15.4, FR-15.10, FR-6.7 |

## Decision

The پیشگی tab of the voucher screen lists buyers as well as sellers, each marked (بیوپاری / گاہک). For a seller nothing changes (a seller advance, FR-15.8). For a buyer it saves exactly as نقد ادھار (FR-15.11): buyer بنام, روکڑ جمع, a reason is required, it is one of his bills (FR-15.4), shows in his dues, اگراہی and ledger as "نقد ادھار", and is collected with a normal وصولی. No new money rule, no new posting type; the posting is the existing one of an ادائیگی to a buyer.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: the advance save accepts a buyer and routes it to the cash-lent posting of FR-15.11 (same function, same checks: reason required, cash below zero asks, dead amount refused); the voucher keeps its own number series as decided below; list and print show it; tests (worked day: a پیشگی of 3,000 to اکرم gives the E2E-32 numbers: his dues 5,880, cash 55,484, R20 24,564) | `backend-peshgi` |
| T2 | Frontend: the پیشگی picker lists buyers and sellers with their type; the reason field shows for a buyer; the helpers (open advances for a seller, open bills for a buyer); ur and en; tests | `frontend-peshgi` |

Number series: a buyer پیشگی is saved as the same voucher type as cash lent today (payment to a buyer), so its number is from the ادائیگی series; the پیشگی list shows it with its type. [A, lead]

## Acceptance

- The پیشگی picker shows buyers and sellers; a buyer پیشگی posts and reads exactly like E2E-32 (dues 5,880, cash 55,484, R20 24,564); a seller پیشگی is unchanged (E2E-7, E2E-31 green).
- Reason required for a buyer; cash below zero asks (FR-6.7); a buyer in the dead amount is refused.
- Golden tests unchanged; every new test fails before; clippy, typecheck, lint, i18n:check pass.
