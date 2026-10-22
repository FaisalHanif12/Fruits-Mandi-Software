### Reports (plan 20, PRD 19; the report engine and the Accounts reports).
### Every message here is `# review: pending` until the Urdu and English reviewers have read it (NFR-URD-4).

## The رپورٹ area: the catalogue, the filters, the table and the print preview (plan 20, PRD 19).

# review: pending
reports-title = Reports

# review: pending
reports-help = Pick a report. Enter the dates and the result shows at once. Every report can be printed.

# review: pending
reports-catalog-failed = The list of reports could not be opened

# review: pending
reports-catalog-empty = No report is ready yet

# review: pending
reports-catalog-empty-help = The reports come in a later version. None of your data is affected.

# review: pending
reports-group-label = Report group: { $group }

# review: pending
reports-open-report = Open { $title }

## The groups of the catalogue (PRD 19.2).

# review: pending
report-group-sales = { -bikri }

# review: pending
report-group-accounts = Accounts

# review: pending
report-group-collections = { -agrahi }, { -adaigi } and { -peshgi }

# review: pending
report-group-business = Business

## The reports: a title and one line that says what it shows.

# review: pending
report-r11-title = { -roznamcha }

# review: pending
report-r11-help = Every entry of the days you choose, with { -banaam } and { -jama }.

# review: pending
report-r13-title = { -khata }

# review: pending
report-r13-help = Every entry of one account, with the goods and the running balance.

# review: pending
report-r14-title = { -fard-hisab }

# review: pending
report-r14-help = A summary of one account: opening balance, dealings, payments and closing balance.

# review: pending
report-r16-title = Account { -index }

# review: pending
report-r16-help = All accounts with their city and balance.

# review: pending
report-r17-title = Head Balances

# review: pending
report-r17-help = The balance of every head and control.

# review: pending
report-r18-title = { -trial-balance }

# review: pending
report-r18-help = The balance of every account. Both sides must give the same { -meezan }.

# review: pending
report-r19-title = Opening Trial Balance

# review: pending
report-r19-help = Opening balances only, with the { -meezan } of both sides.

## The report screen.

# review: pending
report-back = All reports

# review: pending
report-load-failed = The report could not be opened

# review: pending
report-grid-label = Report table

# review: pending
report-empty = No entries for these filters

# review: pending
report-empty-help = Try other dates or another account.

# review: pending
report-clear-filters = Clear the filters

# review: pending
report-pick-party = Pick an account first

# review: pending
report-pick-party-help = This report is for one account. Type its code or name above.

# review: pending
report-balance-only-help = Only the balances are shown. Tick "With detail" to see every line.

# review: pending
report-count = Showing { $shown } of { $total } rows

# review: pending
report-more = Show more rows

# review: pending
report-open-hint = Use ↑ ↓ to pick a row and Enter to open its source.

# review: pending
report-preview = Print preview

## The filters (PRD FR-19.1, FR-19.2, FR-19.10).

# review: pending
report-filters-label = Report filters

# review: pending
report-filter-period = Period

# review: pending
report-period-custom = Choose the dates

# review: pending
report-period-today = Today

# review: pending
report-period-yesterday = Yesterday

# review: pending
report-period-this-week = This week

# review: pending
report-period-this-month = This month

# review: pending
report-period-last-month = Last month

# review: pending
report-filter-from = From

# review: pending
report-filter-to = To

# review: pending
report-filter-as-of = As of

# review: pending
report-filter-party = Account

# review: pending
report-filter-seller = { -beopari }

# review: pending
report-filter-zamindar = { -zamindar }

# review: pending
report-filter-item = { -qism-maal }

# review: pending
report-filter-city = City

# review: pending
report-filter-adda = { -adda }

# review: pending
report-filter-detail = With detail

# review: pending
report-filter-balance-only = Balances only

# review: pending
report-range-from-to = From { $from } to { $to }

# review: pending
report-range-from = From { $from }

# review: pending
report-range-to = Up to { $to }

# review: pending
report-range-all = All dates

# review: pending
report-as-of-shown = Balances as of { $date }

# review: pending
report-as-of-now = Balances up to now

## The table: totals and the lines above it (PRD FR-19.5).

# review: pending
report-total = { -meezan }

# review: pending
report-opening-balance = Opening balance

# review: pending
report-closing-balance = Closing balance

# review: pending
report-difference = Difference

# review: pending
report-statement-bought = Total bought

# review: pending
report-statement-sold = Total sold

# review: pending
report-statement-debit = Total { -banaam }

# review: pending
report-statement-paid = Paid

# review: pending
report-statement-credit = Total { -jama }

# review: pending
report-side-dr = { -banaam }

# review: pending
report-side-cr = { -jama }

# review: pending
report-side-zero = Nil

# review: pending
report-col-date = Date

# review: pending
report-col-doc = Entry

# review: pending
report-col-account = Account

# review: pending
report-col-narration = Narration

# review: pending
report-col-debit = { -banaam }

# review: pending
report-col-credit = { -jama }

# review: pending
report-col-balance = Balance

# review: pending
report-col-detail = Goods

# review: pending
report-col-city = City

# review: pending
report-col-head = Head

# review: pending
report-col-opening-date = Opening date

## The goods behind a ledger line (PRD FR-19.8). The numbers are put in by the screen; no sentence comes from Rust.

# review: pending
report-detail-sale = { -bikri } { $number }: { $item } { $qty } { $packing } @ { $rate } = { $value }, { -commission } { $commission }, { -laga } { $laga }

# review: pending
report-detail-lot = { -bikri } { $number }: { $item } { $qty } { $packing }, { -kham-bikri } { $gross }, { print-total-deductions } { $deductions }

# review: pending
report-detail-cash-lent = { -naqd-udhar }: { $reason }

# review: pending
report-doc-number = { $kind } { $number }

## The kind of each entry, one for each posting type (PRD 6.1).

# review: pending
report-doc-bikri-first = { -bikri }

# review: pending
report-doc-bikri-later = Later { -bikri }

# review: pending
report-doc-reversal = Reversal

# review: pending
report-doc-receipt = { -wasooli }

# review: pending
report-doc-payment = { -adaigi }

# review: pending
report-doc-advance = { -peshgi }

# review: pending
report-doc-shop-expense = { -dukan-kharcha }

# review: pending
report-doc-personal-expense = { -zati-kharcha }

# review: pending
report-doc-personal-return = { -zati-kharcha } { -wapsi }

# review: pending
report-doc-money-lent = { -adhar-raqam }

# review: pending
report-doc-money-lent-return = { -adhar-raqam } { -wapsi }

# review: pending
report-doc-capital-in = { -sarmaya } invested

# review: pending
report-doc-capital-out = { -sarmaya } taken back

# review: pending
report-doc-khata-out = Separate Khata: paid out

# review: pending
report-doc-khata-in = Separate Khata: received

# review: pending
report-doc-purchase = { -khareedari }

# review: pending
report-doc-trip-cost = Trip cost

# review: pending
report-doc-garden-payment = Garden payment

# review: pending
report-doc-venture-receipt = Sale in another shop

# review: pending
report-doc-garden-close = Garden closing

# review: pending
report-doc-fruit-taken = Fruit taken

# review: pending
report-doc-general = { -general }

# review: pending
report-doc-opening = Opening balance

# review: pending
report-doc-month-close = Month close

# review: pending
report-doc-other = Other

## The print preview (PRD FR-7.36, FR-19.3, FR-19.13). The preview is drawn in the print language, which can differ from the app language.

# review: pending
report-print-title = Report print preview

# review: pending
report-print-failed = The print could not be prepared

# review: pending
report-print-empty = There are no rows to print

# review: pending
report-print-row-count = Rows

# review: pending
report-print-paper = Paper

# review: pending
report-print-print = Print

# review: pending
report-print-pdf = Save as PDF

# review: pending
report-print-job-later = Sending to the printer and saving as PDF come in a later version. For now you can look at the page.

# review: pending
report-print-close = Close
