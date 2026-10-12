# vouchers (plan 10)

The voucher framework and the first voucher, the پیشگی (advance) screen of PRD 15.1 and 15.4.

- `PaymentModeFields` (cash, bank, cheque) is shared by every voucher screen. Other modules import
  it from `index.ts` only.
- The screen keeps what is typed as text in its own state. The balances, the amounts and the total
  of the list come from the backend preview (the same function as the save).
- Several vouchers can wait in a list and be saved together. Each is its own posting event, so a
  refused one stays in the list with its message while the others are saved.
- The voucher number series is made in Fluent (`vouchers-number-advance`): پ-1 in Urdu, A-1 in
  English. Umar to confirm the English letters.
- `types.ts` and `api.ts` are the only files that know the command shapes. They are interim until
  the generated bindings arrive.
