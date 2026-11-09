**Plan 65 pre-review, for `backend`. Verdict: PASS with one change to P3/P11, in point 10a.** Nothing was changed or run. This is from reading the working tree on `feature/d8-demo-feedback`.

**1. Line calc** (`domain/src/bikri/line.rs`, `calc_line`, after `buyer_total`)
- Add `checks::must_pay_in_full(buyer)`, true for `is_naqdi_mandi`, `is_chukta` or `!credit_allowed`. Delete `checks::payment_type` and `resolve_payment`.
- `PreparedSale.paid: Paisa` comes from `optional_money` at `lots[i].sales[j].paid`. Empty is 0, and `20+25` works. A row with only `paid` stays blank (FR-7.52).
- Rule, one function:
  - Forced buyer: `paid = buyer_total` and any typed value is ignored. See 10a for the one exception.
  - Other buyers: `paid` is the typed value.
  - `left = buyer_total.checked_sub(paid)?`. Compare only with `Paisa` operators.
- Typed `paid > buyer_total`: `ERR_PAID_EXCEEDS_TOTAL` at `.paid` with `max` in rupees, as `max_rupees()` does. The line keeps its figures and counts as `paid = 0`, so cash is not inflated. The error blocks the save.
- `LineCalc` gains `paid`, `left`, `paid_fixed` and workings. `Acc::add_line` and `saved.rs:324` use `buyer_credit += left`, `buyer_cash += paid`.
- Remove `ErrCreditNotAllowed` (variant, ftl messages, 3 test files).
- Tests: 21,360 / 10,000 / 0; forced buyer with typed 5,000 still gets the total; total + 1 refused and the save refused; property: paid + left = buyer total, working equals stored.

**2. Journal** (`ledger/bikri.rs`)
- `JournalSale.payment_type` becomes `paid: Paisa`.
- In the cash part: `if sale.paid > Paisa::ZERO`, push روکڑ Dr `paid`, then the buyer Cr `paid`, on `sale.buyer_account_id` with `sale.person_key.as_deref()`.
- `slot.cash_credit = at` stays. It is `Some` exactly when paid is above 0. Line order does not change.
- Tests: paid 0 and paid in full are line-for-line what credit and cash build today; partly paid balances; `cash_credit` points at the buyer Cr line; a چکتا line carries its key.

**3. Matching (Q3)**
- `matching.rs` needs only doc changes. `pinned` means same-entry, and `plan_key` uses the row amount. Every paid credit is fully pinned, so no other row pairs lines of one entry.
- `rows.rs:565`: `if row.calc.paid > ZERO`, with `amount: row.calc.paid`.
- `rows.rs:600`: `if row.calc.left > ZERO`, with `NewLine { amount: row.calc.left }`. Sending the buyer total would hit `ERR_ALLOCATION_AMOUNT`. Pin before matching, as now.
- Edit and cancel: no change. Both call `rematch_party` on touched keys, and a reversed entry's pin goes stale and is switched off.
- Tests:
  - An older open bill of 5,000 stays open when 10,000 is paid on a new 48,100 bill.
  - After a receipt on the left part, an edit or cancel re-matches it.
  - Add partly paid lines to `after_any_sequence_..._canonical_ones`.

**4. Credit limit** (`calc_line`, `checks::credit_limit`)
- Rename the argument to `left` and call it only `if left > ZERO`, so a fully paid line never warns.
- `Run.owed` then adds `left`, on top of the signed balance.
- Tests:
  - Limit 40,000: 48,100 paid 10,000 is fine; paid 5,000 warns (43,100).
  - Two lines add up.
  - ہارون at −10,000 with limit 120,000 is fine (115,060).

**5. Edit (P11)**
- `unchanged_stored`: delete the payment comparison. `paid` is outside the test, so a paid-only change keeps `args.stored` and its rules.
- `StoredSale.payment_type` becomes `paid`.
- Empty value on a saved paid-in-full line of an ordinary buyer: paid becomes 0, rules are kept, cash falls and the change shows before save (P2). That is the user clearing the cell.
- So `load.rs:257` must send the stored paid as text.
- Audit and change list: `paid_paisa` in `edit.rs:423`; `("paid", money)` in `edit/bikri.rs:157`.
- Test: E2E-13b style. Default goes to 7%, edit only ارشد's paid: 1,044 and the split do not change, only the cash part.

**6. Migration 0021**, in this order:
1. `ALTER TABLE bikri_sales ADD COLUMN paid_paisa INTEGER NOT NULL DEFAULT 0 CHECK (paid_paisa >= 0) CHECK (paid_paisa % 100 = 0) CHECK (paid_paisa <= buyer_total_paisa);`
2. `UPDATE bikri_sales SET paid_paisa = buyer_total_paisa WHERE payment_type = 'cash';` with no `is_active` filter and no `version` change.
3. `DROP` and recreate `bikri_sales_snapshot_is_final` with the `chukta_*` names, plus `OR NEW.paid_paisa IS NOT OLD.paid_paisa`.

`journal_lines`, `bill_allocations` and the chains are untouched (`canonical_bytes` hashes only entries and lines). Do not tie `payment_type` to paid in a CHECK. Writer: `PaymentType::of(paid, total)`, `cash` only when equal.

**7. Reads**

| Place | Change |
|---|---|
| R3, R4 (`register.rs`) | `Kind` gives a condition: `s.paid_paisa > 0`; `s.paid_paisa < s.buyer_total_paisa`. Paid and left are the last columns; R2 unchanged. |
| R5 (`buyer_bills.rs`) | `conds(typed, None)`; rows, totals and slips gain paid and left. R5 total now equals R2. Test at `reports_sales.rs:1015` and the fixtures change; this is not a golden test. |
| R13, R14 | Balances are already right. Add `s.paid_paisa` to `load_sales`; `detail_of_line` gets a buyer credit branch (FR-19.8 mark). |
| R34, R35 (`figures_load.rs:282`, `figures.rs:346`) | `s.paid_paisa`; `BuyerLine.cash` becomes `paid`; `add(&mut on_spot, line.paid)`. The guard at 352 proves it against the journal. |
| Print slip (`pages.rs:282`) | `any_cash = paid > 0`, `any_credit = left > 0`. |
| Dues, dashboard, party card, aging, view C | Journal and allocations; no change. |
| Assistant | R3, R4, R5 column lists in plan 70. |

Do not write `total - paid` in SQL: `SUM` both and use `checked_sub` in Rust.

**8. Old data and the second bug**
- E2E-66: build at migration 20, with cash, credit and replaced lines. After 0021: paid = total or 0, journal rows identical, chains verify, balances equal, `paid_paisa` update refused, paid above total refused.
- `books_of` in `shared.rs`: add `Err(LedgerError::Empty) => { if errors.is_empty() { errors.push(FieldError::new(ErrorCode::ErrNoSaleLines, "lots")); } return Ok((Vec::new(), Paisa::ZERO, None)); }` before the catch-all.
- Test: one lot, all rows blank, no charges: the preview returns `ErrNoSaleLines`, and the save refuses with it.

**9. E2E-65 by hand: confirmed, and the entry balances**

| Part | Working | Total |
|---|---|---|
| Sale Dr | 21,360 + 48,100 + 125,060 | 194,520 |
| Sale Cr | seller 160,080 + 10,920 + 10,920 + لاگا 1,600 + مزدوری 1,000 + کرایہ 10,000 | 194,520 |
| Cash part | 21,360 + 10,000, each side | 31,360 |
| Whole entry | each side | 225,880 |

Seller credit: 182,000 − 10,920 − 10,000 − 1,000 = 160,080. Left 0 / 38,100 / 125,060 = 163,160.

ہارون: his earlier 10,000 credit settles the new bill, so −10,000 + 125,060 = 115,060 بنام, open 115,060. Dues from these three are 38,100 + 115,060 = 153,160, not 163,160. Assert that too.

**10. Plan 60**
- **a. Must change, P3/P11.** A forced buyer with `args.stored` present keeps `stored.paid`. Otherwise an edit of an old credit line, whose buyer later lost credit (or an old 0030 line), turns into full cash with no action by the user. A changed line follows P3. Test both.
- b. Missing from P13: the buyer slip. When paid is above 0, add paid and left lines to `slip_page`; otherwise a part-paid slip shows no payment at all.
- c. P4 does not say the line keeps its figures; use point 1.
- P1, P2 and P4 to P14 are otherwise sound. Old auto-saved drafts lose a cash choice and show paid empty; only development data has them.
