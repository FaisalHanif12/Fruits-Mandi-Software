# mandi-reports

Report queries, print data and Excel export (PRD 19).

- May depend on: `mandi-db` and `mandi-core`.
- Reads stored values only. Never recalculates money for a report (Code Standards B20, B23).

## `print`: the seller bill, the buyer slips and the پختہ slip (plan 14)

`print::bikri_print_data(db, input)` reads a saved بکری and the settings (Firm, number grouping, Printing) and calls `print::build`, which is pure and is tested with snapshots.

- Every number is a stored value of the saved بکری (`BikriDetail`). A sum is a checked sum of stored amounts, the same sums `bikri_get` sends. Nothing is rounded or worked out again.
- Values are text, already formatted: grouping by the number grouping setting, English digits unless the Urdu digits setting is on and the print language is Urdu (`print::format`). Dates are `DD-MM-YYYY`. Codes, phones and truck numbers stay as typed.
- Titles and labels are Fluent message IDs (`print-seller-bill-title`, `print-net-sale`, ...). The frontend defines them in `print.ftl`. The only sentences built here are the amounts in words (`core::urdu_words`, `core::english_words`).
- One generic shape for all documents: `PrintData` has a list of `PrintPage`. The seller bill has one page per seller (a seller with two lots gets one page), the buyer slips one page per buyer account (the one-time buyer is left out), the پختہ slip one page per one-time person (by his person key). An empty page list means nothing to print.
- A net below 0 (the seller owes the firm, PRD FR-12.7) prints as a positive amount under `print-seller-owes`; its words are the words of that amount.
- Lines with `is_active = false` are not printed. A cancelled بکری is flagged `cancelled`.

Tests: `tests/print_data.rs` (snapshots in `tests/snapshots`, written with `INSTA_UPDATE=always cargo test -p mandi-reports`, then read in the diff) on the real answers of `bikri_get` (`crates/services/tests/fixtures`). The proof that a printed total equals `bikri_get` and the journal is `crates/services/tests/print.rs`.

## `print::voucher_print_data`: the slip of a voucher (plan 15, PRD FR-15.2)

`voucher_print_data(db, input)` reads a saved وصولی, ادائیگی or پیشگی and builds the same generic `PrintData` with doc `receipt_slip`: one page, the party (the one-time person by his own names), no lines, one `Total` amount (`print-slip-amount-received`, `-paid` or `-advance`), the amount in words, the way the money moved in `payment_id` (`print-mode-cash`, `-bank`, `-cheque`), and facts for the number, the day, the bank, the cheque, the reason, the collector and the note. The title is `print-receipt-slip-title`, `print-payment-slip-title` or `print-advance-slip-title`. A cancelled voucher is flagged `cancelled`. Paper: `paper_receipt_slip` of the Printing settings (starts as A5).
