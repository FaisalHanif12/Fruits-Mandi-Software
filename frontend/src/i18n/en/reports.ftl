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

## Round 4 (plan 20, step F3): the Sales, Collections and Business reports, the person box, the month names, global search and the party card buttons.

## The reports: a title and one line that says what it shows (PRD 19.2).

# review: pending
report-r2-title = Full Sales Register

# review: pending
report-r2-help = Every sale of the days you choose, one row for each { -gahak } line: item, quantity, rate and what the buyer pays.

# review: pending
report-r3-title = { -naqdi } Sales

# review: pending
report-r3-help = Only the { -naqdi } sales of the days you choose.

# review: pending
report-r4-title = { -udhari } Sales

# review: pending
report-r4-help = Only the { -udhari } sales of the days you choose.

# review: pending
report-r6-title = { -bikri } Details

# review: pending
report-r6-help = Every lot of the days you choose, with the { -kham-bikri }, the deductions and the { -safi-bikri }.

# review: pending
report-r7-title = Item-wise Sales

# review: pending
report-r7-help = Sales by item: quantity, { -kham-bikri }, average rate and the { -commission } of both sides.

# review: pending
report-r8-title = { -adda } Report

# review: pending
report-r8-help = Trucks, lots, quantity and { -kham-bikri } for each { -adda }.

# review: pending
report-r9-title = { -pukhta-khata } Register

# review: pending
report-r9-help = Every line of the one-time people of { -pukhta-khata }, with what is paid and what is due.

# review: pending
report-r20-title = { -gahak } Dues

# review: pending
report-r20-help = Each buyer's balance with the age of his open bills. The { -meezan } is the shop's total { -agrahi }.

# review: pending
report-r21-title = { -agrahi } by City

# review: pending
report-r21-help = The same dues, grouped by city, with the total of each city.

# review: pending
report-r22-title = { -agrahi } by Collector

# review: pending
report-r22-help = The same dues, grouped by collector, with the total of each collector.

# review: pending
report-r24-title = Seller Payables

# review: pending
report-r24-help = What the shop owes each seller, his open { -peshgi } and what he owes beyond it.

# review: pending
report-r26-title = { -peshgi-khata }

# review: pending
report-r26-help = For each seller: { -peshgi } given, returned, adjusted and still open.

# review: pending
report-r27-title = Bill Series

# review: pending
report-r27-help = The sellers' bills in number order, with the { -kham-bikri }, the deductions and the { -safi-bikri }.

# review: pending
report-r28-title = Income Report

# review: pending
report-r28-help = Every income account by month or by day. The { -meezan } is the net income.

# review: pending
report-r30-title = Profit and Partner Split

# review: pending
report-r30-help = Income less shop expenses, and each partner's { -hissa }, money taken and net.

## The columns of these reports.

# review: pending
report-col-buyer = { -gahak }

# review: pending
report-col-seller = { -beopari }

# review: pending
report-col-item = { -qism-maal }

# review: pending
report-col-qty = Quantity

# review: pending
report-col-rate = Rate

# review: pending
report-col-value = Value

# review: pending
report-col-buyer-total = Buyer total

# review: pending
report-col-gross = { -kham-bikri }

# review: pending
report-col-freight = { -kiraya }

# review: pending
report-col-commission = { -commission }

# review: pending
report-col-labour = { -mazdoori }

# review: pending
report-col-munshiana = { -munshiana }

# review: pending
report-col-cash-paid = { -naqd }

# review: pending
report-col-store-rent = { -kiraya-store }

# review: pending
report-col-phone-charge = { -phone }

# review: pending
report-col-deductions = { print-total-deductions }

# review: pending
report-col-net = { -safi-bikri }

# review: pending
report-col-avg-rate = Average rate

# review: pending
report-col-seller-comm = Seller { -commission }

# review: pending
report-col-buyer-comm = Buyer { -commission }

# review: pending
report-col-laga = { -laga }

# review: pending
report-col-total-comm = Total { -commission }

# review: pending
report-col-adda = { -adda }

# review: pending
report-col-trucks = Trucks

# review: pending
report-col-lots = Lots

# review: pending
report-col-side = Side

# review: pending
report-col-person = Name

# review: pending
report-col-phone = Phone

# review: pending
report-col-status = Status

# review: pending
report-col-oldest-date = Oldest bill

# review: pending
report-col-days = Days

# review: pending
report-col-age-0-7 = 0-7 days

# review: pending
report-col-age-8-30 = 8-30 days

# review: pending
report-col-age-31-90 = 31-90 days

# review: pending
report-col-age-91 = Over 90 days

# review: pending
report-col-collector = Collector

# review: pending
report-col-section = List

# review: pending
report-col-payable = Payable

# review: pending
report-col-advance-open = Open { -peshgi }

# review: pending
report-col-owes-firm = Owes the shop

# review: pending
report-col-last-date = Last date

# review: pending
report-col-given = Given

# review: pending
report-col-returned = Returned

# review: pending
report-col-adjusted = Adjusted

# review: pending
report-col-open = Open

# review: pending
report-col-bill-date = Bill date

# review: pending
report-col-period = Period

# review: pending
report-col-amount = Amount

# review: pending
report-col-income-share = Income { -hissa }

# review: pending
report-col-expense-share = Expense { -hissa }

# review: pending
report-col-profit-share = Profit { -hissa }

# review: pending
report-col-taken = { -zati-kharcha }

# review: pending
report-col-partner-net = { -khalis }

# review: pending
report-col-partner = { -shirakat-dar }

## The words inside the rows and above the table.

# review: pending
report-side-buyer = { -gahak }

# review: pending
report-side-seller = { -beopari }

# review: pending
report-section-payable = Payable to sellers

# review: pending
report-section-owes-firm = Sellers who owe the shop

# review: pending
report-status-all = All lines

# review: pending
report-status-paid = Paid

# review: pending
report-status-due = Due

# review: pending
report-status-cancelled = Cancelled

# review: pending
report-r30-income = Total income

# review: pending
report-r30-expenses = Total shop expenses

# review: pending
report-r30-profit = Profit

# review: pending
report-filter-paid = Paid lines only

# review: pending
report-filter-due = Due lines only

# review: pending
report-filter-by-day = By day

# review: pending
report-filter-by-month = By month

## The new filter boxes.

# review: pending
report-filter-person = One-time person

# review: pending
report-filter-person-hint = Type a name or a phone number

# review: pending
report-filter-status = Show

# review: pending
report-filter-group-by = Group by

# review: pending
report-group-by-month = Month

# review: pending
report-group-by-day = Day

## The names of the months (R28 by month; PRD 19.2, NFR-URD-16).

# review: pending
report-month-1 = January

# review: pending
report-month-2 = February

# review: pending
report-month-3 = March

# review: pending
report-month-4 = April

# review: pending
report-month-5 = May

# review: pending
report-month-6 = June

# review: pending
report-month-7 = July

# review: pending
report-month-8 = August

# review: pending
report-month-9 = September

# review: pending
report-month-10 = October

# review: pending
report-month-11 = November

# review: pending
report-month-12 = December

## The party card buttons (PRD FR-7.65).

# review: pending
report-card-ledger = { -khata }

# review: pending
report-card-statement = { -fard-hisab }

# review: pending
report-card-open-ledger = Open the { -khata } of this account

# review: pending
report-card-open-statement = Open the { -fard-hisab } of this account

# review: pending
report-card-reports-label = Reports of this account

## Global search: one-time people and screens (PRD FR-5.4).

# review: pending
report-search-person = One-time person

# review: pending
report-search-person-last = Last { $date }

# review: pending
report-search-screen = Screen

# review: pending
report-search-report = Report

## The fold of the filter boxes in a compact window (Code Standards F22).

# review: pending
report-filters-more = More filters

# review: pending
report-filters-fewer = Fewer filters

## The heading of the buyers with no city or no collector (R21, R22).

# review: pending
report-group-no-city = No city

# review: pending
report-group-no-collector = No collector

# review: pending
report-adda-none = No Adda

## The cash book, the expense report, the partner statement and money lent (round 7, R12, R29, R31, R32).

# review: pending
report-r12-title = { -rokar }

# review: pending
report-r12-help = The opening cash, every cash in and out of the days chosen, and the closing cash, with the running balance.

# review: pending
report-r29-title = { -kharcha } Report

# review: pending
report-r29-help = { -dukan-kharcha } by head, and the { -zati-kharcha } each partner took and returned.

# review: pending
report-r31-title = { -shirakat-dar } Statement

# review: pending
report-r31-help = Every dated movement of one partner, with his running { -kul-raqam }.

# review: pending
report-r32-title = { -adhar-raqam } Report

# review: pending
report-r32-help = Everyone who borrowed from the shop: lent, returned, still open, and the days past the return-by date.

# review: pending
report-col-cash-in = Cash In

# review: pending
report-col-cash-out = Cash Out

# review: pending
report-col-name = Name

# review: pending
report-col-expense = { -kharcha }

# review: pending
report-col-staff = Staff

# review: pending
report-col-lent = Lent

# review: pending
report-col-returned-cash = { -wapsi } (Cash)

# review: pending
report-col-returned-salary = Salary Deduction

# review: pending
report-col-return-by = Return By

# review: pending
report-col-days-past = Days Late

# review: pending
report-staff-yes = Yes

# review: pending
report-r32-overdue = Open past the return-by date

# review: pending
report-section-shop-expenses = { -dukan-kharcha }

# review: pending
report-section-partner-money = { -zati-kharcha }

# review: pending
report-filter-partner = { -shirakat-dar }

# review: pending
report-filter-borrower = Borrower

# review: pending
report-partner-all = All partners

# review: pending
report-pick-partner = Pick a partner first

# review: pending
report-pick-partner-help = This statement is for one partner. Choose him above.

## Round 8: stock (R10), vouchers (R15), receipts (R23), payments (R25), trip profit or loss (R33) and the day summary (R34).

# review: pending
report-r10-title = { -baqi-maal }

# review: pending
report-r10-help = Goods received and not yet sold: what came, what was sold, what is left and for how many days.

# review: pending
report-r15-title = Vouchers

# review: pending
report-r15-help = Every voucher by kind and date. With no kind picked, the money of each kind is shown above the table. A cancelled voucher is listed and not counted.

# review: pending
report-r23-title = { -wasooli } Register

# review: pending
report-r23-help = Every receipt with the buyer, the amount and the collector. A seller returning his advance is here too. A cancelled receipt is listed and not counted.

# review: pending
report-r25-title = { -adaigi } Register

# review: pending
report-r25-help = Every payment with the account and the amount. A cancelled payment is listed and not counted.

# review: pending
report-r33-title = Trading Profit / Loss

# review: pending
report-r33-help = What the market trips bought, sold, wrote off and still hold, and the profit or loss of each trip, by trip, item, day, month or partner.

# review: pending
report-r34-title = Day Summary

# review: pending
report-r34-help = One block for each day: sales, cash, receipts, payments, expenses, the partners' money, trips and what is still owed.

## The columns of these reports.

# review: pending
report-col-packing = { -packing }

# review: pending
report-col-received = Received

# review: pending
report-col-sold = Sold

# review: pending
report-col-written-off = Written Off

# review: pending
report-col-balance-qty = Balance

# review: pending
report-col-days-waiting = Days Waiting

# review: pending
report-col-trip = { -safar }

# review: pending
report-col-purchase = { -khareedari }

# review: pending
report-col-bought-qty = Quantity Bought

# review: pending
report-col-bought-cost = Bought Cost

# review: pending
report-col-sales-money = Sales Money

# review: pending
report-col-cost-sold = Cost of Goods Sold

# review: pending
report-col-written-off-cost = Written-off Cost

# review: pending
report-col-left-qty = Left

# review: pending
report-col-left-cost = Cost Left

# review: pending
report-col-result = Result

# review: pending
report-col-share = { -hissa }

# review: pending
report-col-money-in-shop = Paid: { -dukan-se }

# review: pending
report-col-money-in-pocket = Paid: { -apni-jeb-se }

# review: pending
report-col-money-back = Money Back

# review: pending
report-col-part-of-result = Part So Far

# review: pending
report-col-line = Detail

# review: pending
report-col-count = Count

## The words of a result, the new filter boxes and the print facts.

# review: pending
report-result-profit = Profit

# review: pending
report-result-loss = Loss

# review: pending
report-filter-voucher-type = Type

# review: pending
report-voucher-type-all = All kinds

# review: pending
report-filter-trip-view = Show by

# review: pending
report-filter-trip-by-trip = By trip

# review: pending
report-filter-trip-by-item = By item

# review: pending
report-filter-trip-by-day = By day

# review: pending
report-filter-trip-by-month = By month

# review: pending
report-filter-trip-by-partner = By partner

## The lines of the day summary (R34).

# review: pending
report-r34-bikri = { -kham-bikri }

# review: pending
report-r34-commission = { -commission } and { -laga }

# review: pending
report-r34-cash-opening = Cash Opening

# review: pending
report-r34-cash-in = Cash In

# review: pending
report-r34-cash-out = Cash Out

# review: pending
report-r34-cash-closing = Cash Closing

# review: pending
report-r34-receipts = { -wasooli }

# review: pending
report-r34-payments = { -adaigi }

# review: pending
report-r34-advances = { -peshgi }

# review: pending
report-r34-expenses = { -dukan-kharcha }

# review: pending
report-r34-partner-taken = { -zati-kharcha }

# review: pending
report-r34-partner-returned = { -zati-kharcha } { -wapsi }

# review: pending
report-r34-capital = { -sarmaya }

# review: pending
report-r34-money-lent = { -adhar-raqam }

# review: pending
report-r34-money-lent-back = { -adhar-raqam } { -wapsi }

# review: pending
report-r34-trips-bought = { -market-safar }: { -khareedari }

# review: pending
report-r34-trips-sold = { -market-safar }: { -bikri }

# review: pending
report-r34-partner-shares = Partners' { -hissa }

# review: pending
report-r34-buyers-dues = Buyers' Dues

# review: pending
report-r34-open-lots = Unsold { -lot }

# review: pending
report-r34-open-qty = { -nag } Left

## R35, the party day sheet (plan 27, PRD FR-19.12): the table of every party and the slip of one party.

# review: pending
report-r35-title = Party Day Sheet

# review: pending
report-r35-help = For one day: what each party owed at the start, what was debited and credited, and what is left. Pick a party to see his slip.

# review: pending
report-col-code = Code

# review: pending
report-col-previous = { -sabiqa-raqam }

# review: pending
report-r35-buyers = Buyers

# review: pending
report-r35-sellers = Sellers

# review: pending
report-r35-total-qty = Total { -nag }

# review: pending
report-r35-previous = { -sabiqa-raqam }

# review: pending
report-r35-bought = { -taza-banam }

# review: pending
report-r35-sold = Sold That Day

# review: pending
report-r35-other-debits = Other { -banaam }

# review: pending
report-r35-other-credits = Other { -jama }

# review: pending
report-r35-on-spot = Paid on the Spot

# review: pending
report-r35-cash-given = { -naqd }

# review: pending
report-r35-received = { -wasooli }

# review: pending
report-r35-paid = { -adaigi }

# review: pending
report-r35-remaining = { -total-raqam }

# review: pending
report-r35-collection = Total { -agrahi }

# review: pending
report-r35-sellers-credit = Sellers' { -jama }

# review: pending
report-r35-sellers-debit = Sellers' { -banaam }

# review: pending
report-r35-sellers-remaining = Sellers' { -baqaya }

# review: pending
report-filter-day = Day

# review: pending
report-day-shown = Sheet for { $date }

# review: pending
report-day-now = Sheet for today

# review: pending
report-r35-all-parties = All parties

## R36, the gardens report (round 11, plan 27, PRD 19.2 R36, FR-12.37), and the garden rows of R34.

# review: pending
report-r36-title = { -baghat } Report

# review: pending
report-r36-help = Each garden: rent, installments, costs, what the shop and the partners paid, the money that came in, the running balance and the rent still due. By garden or by partner.

# review: pending
report-col-garden = { -bagh }

# review: pending
report-col-rent = { -thaika }

# review: pending
report-col-installments = { -qist }s

# review: pending
report-col-garden-costs = Garden Costs

# review: pending
report-col-sales-our-shop = Sales in Our Shop

# review: pending
report-col-sales-other-shops = Sales in Other Shops

# review: pending
report-col-other-in = Other Money In

# review: pending
report-col-money-in = Money In

# review: pending
report-col-running-balance = Running Balance

# review: pending
report-col-rent-due = Rent Due

# review: pending
report-garden-open = Open

# review: pending
report-garden-closed = Closed

# review: pending
report-filter-garden = { -bagh }

# review: pending
report-filter-garden-view = Show by

# review: pending
report-filter-garden-by-garden = By garden

# review: pending
report-filter-garden-by-partner = By partner

# review: pending
report-r34-gardens-paid = { -baghat } Paid

# review: pending
report-r34-gardens-in = { -baghat } Money In

## R1, the sale book, and R5, the credit buyers' details (round 12, PRD 19.2 R1 and R5, FR-12.12).

# review: pending
report-r1-title = { -bikri-book }

# review: pending
report-r1-help = The lots of one { -bikri }: what came, what was sold, the charges and the net sale. Its bill is printed from here.

# review: pending
report-r5-title = { -udhari } Details

# review: pending
report-r5-help = Each { -udhari } buyer with his goods, what they come to and what he owes with the { -commission } and { -laga }.

# review: pending
report-col-serial = No.

# review: pending
report-filter-bikri = { -bikri } number

# review: pending
report-pick-bikri = Pick a { -bikri } first

# review: pending
report-pick-bikri-help = This report is for one { -bikri }. Type its number above.

# review: pending
report-bikri-hit = { -bikri } <num>{ $number }</num> · <num>{ $date }</num>

# review: pending
report-detail-sale-short = ({ $item } { $qty } . { $rate })

## Print all the credit buyers' slips (R5, PRD 19.2 R5 "Print All"): one slip for each buyer.

# review: pending
report-print-all = Print all slips

# review: pending
report-buyer-bills-slips = Slips
