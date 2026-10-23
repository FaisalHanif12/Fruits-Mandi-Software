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

## `reports`: the report engine and the Accounts group (plan 20, PRD 19)

One engine serves every report, so one screen can show them all (`reports::catalog`, `run`, `print_data`).

- **Catalogue** (`catalog()`): each report has its PRD ID, group, title ID and the filters it takes (`date_range`, `as_of`, `party`, `seller`, `zamindar`, `item`, `city`, `adda`, `detail`).
- **Run** (`run(db, ReportRunInput)`): reads and checks the filters, then returns one page: typed columns, typed rows (text, label, number, money, balance with side, date, account, names, document, items), `total_count`, the `totals` row over the whole filter, the filters as read, lines above the table (`summary`), and for each row the record it opens. A total is never a sum of the page. A filter the report does not take, a bad date, an unknown record and a bad page are refused together, each with its field (`filters.from`, `filters.party.account_id`, `offset`, `limit`).
- **Print data** (`print_data(db, ReportPrintInput)`): the same report with every row, formatted for the print language (grouping, digits, `DD-MM-YYYY`), with the firm header, the filters of the run, the totals row and, for R13 and R14, the closing balance in words. The template calculates nothing; "Page x of y" is drawn by the shared print layout.
- **Reads posted records only and writes nothing** (PRD FR-6.1, FR-18.8). A total is a `SUM()` of stored `INTEGER` paisa columns, or a checked sum of such sums. A balance is `Σ debit − Σ credit`, worked out in Rust with checked maths (`Balance::from_totals`). The running balance of a ledger page comes from `SUM() OVER` on the same columns.

| Report | Rows | Total row |
|---|---|---|
| R11 روزنامچہ | posted journal lines in the range, in the order `(entry_date, entry id, line no)` | Σ بنام, Σ جمع |
| R13 کھاتہ | the lines of one account (or one person of a پختہ کھاتہ account) with their items (FR-19.8) and the balance after each | Σ بنام, Σ جمع, closing balance; opening balance above |
| R14 فرد حساب | balance only: no lines; with `detail`: the lines of R13 | as R13; above: opening, bought or sold, paid, closing |
| R16 انڈکس | every account with its city and balance, by code | Σ of the debit balances, Σ of the credit balances |
| R17 مدوں بیلنس | each head with its controls, net of all their lines | Σ of the heads on each side |
| R18 ٹرائل بیلنس | every account with posted lines | Σ بنام, Σ جمع (equal); difference above |
| R19 اوپن ٹرائل | the opening balances, and ابتدائی فرق as the other side | Σ بنام, Σ جمع |

**Items on a ledger line** (`accounts/ledger_items.rs`): the journal line does not store the sale line it came from. The entry builder writes one buyer debit per sale line in `(lot, line)` order and one credit per lot, so the n-th debit of a buyer in the entry is his n-th sale line. The pairing is checked against the stored buyer total, and a line that does not match is shown with no items, never with wrong ones. A reversal is paired through the entry it reverses.

Tests: `crates/services/tests/reports_accounts.rs` on the PRD worked day (R18 is the E2E-10 table, R13 for اکرم is E2E-33, every account's ledger closes on the services balance, pages add up to the whole, a report writes nothing, query plans use indexes) and writes the fixtures of the commands in `crates/services/tests/fixtures/reports_*.json` (`MANDI_WRITE_FIXTURES=1` writes them again).

## `analytics`: the views A to E and H (plan 21, PRD 18)

`analytics::catalog`, `run`, `lines`, `print_data`, `views_get` and `views_save`.

- **Source.** Every sales figure is a stored value of a sale line, summed from `analytics_facts` (FR-18.6). A fact has a sign: +1 for a saved line, −1 for the row a reversal writes, with the same date and dimensions. Each column is summed twice in SQL (`SUM(CASE WHEN sign = 1 ...)`, `SUM(CASE WHEN sign = -1 ...)`), and the net is worked out in Rust with checked maths. A net below 0 is an error, never shown. A group with nothing left has no row, so a cancelled or edited-away sale adds nothing (FR-18.7).
- **Total commission** is seller commission + buyer commission + لاگا, or the side asked for. The percent of PRD 18.2 is `(part × 10,000 + total / 2) / total` in `i128` (hundredths of a percent, `calc.rs`); it is display only.
- **Views.** A by item (with the average rate and the share), B by seller (بکری count, and صافی بکری = facts less the charges of the saved lots dated in the period), C by buyer (dues now, and average days to pay), D item by period (day, month or year, at most 36 columns), E seller by item (at most 24 columns, the rest in one "other items" column), H top 10 items, sellers and buyers.
- **Filters** (FR-18.1): period shortcut or dates, item, seller, buyer (and person), the buyer's city, اڈا, side, compare (previous period or the same dates last year, needs both dates). Garden, trip, goods and partner filters wait for the later plans.
- **Lines behind a number** (FR-18.2): `lines` returns each fact row, a reversal row negative, so the lines add up to the number that was opened; each opens its بکری.
- **Saved views** (FR-18.5): the whole list of the signed-in user in the `user_prefs` row `analytics.views`. They keep filters only, never a figure.
- **Writes nothing** of the books (FR-18.8). The reports of the Collections group use `reports/collections/grouped.rs` for their sums of all sellers at once (Code Standards B22).

Tests: `crates/services/tests/analytics.rs` (PRD 18.4, E2E-11, E2E-13, E2E-15, E2E-16b, the journal reconciliation, random sequences), `analytics_fixtures.rs` (the fixtures `analytics_*.json`; `MANDI_WRITE_FIXTURES=1` writes them again), `reports_grouped.rs`.
