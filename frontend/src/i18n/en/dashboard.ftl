### The dashboard and the parts that are not built yet (plan 04, PRD FR-5.1, FR-7.46).

# review: pending
dashboard-greeting = Hello, <name>{ $name }</name>

# review: pending
dashboard-today = Today: <num>{ $date }</num>

# review: pending
dashboard-new-bikri = New { -bikri }

## The dashboard (plan 19, PRD Section 9). Every Urdu line says what its English line says (NFR-URD-29).

# review: pending
dashboard-load-failed = The dashboard could not be opened

# review: pending
dashboard-quick-title = Quick actions

# review: pending
dashboard-new-receipt = New { -wasooli }

# review: pending
dashboard-new-payment = New { -adaigi }

# review: pending
dashboard-new-advance = New { -peshgi }

# review: pending
dashboard-new-expense = New { -kharcha }

# review: pending
dashboard-open-daybook = { -roznamcha }

# review: pending
dashboard-tiles-title = Today

# review: pending
dashboard-tile-sales = Today's { -bikri }

# review: pending
dashboard-tile-sales-count = { $count ->
    [one] <num>{ $count }</num> { -bikri }
   *[other] <num>{ $count }</num> { -bikri }s
}

# review: pending
dashboard-tile-commission = Today's { -commission }

# review: pending
dashboard-tile-cash = Cash in hand

# review: pending
dashboard-tile-cash-below = Below zero

# review: pending
dashboard-tile-dues = Buyers' dues

# review: pending
dashboard-tile-payable = Sellers payable

# review: pending
dashboard-tile-advances = Advances open

# review: pending
dashboard-tile-stock = { -baqi-maal }

# review: pending
dashboard-tile-stock-note = { $lots ->
    [one] <num>{ $lots }</num> { -lot }
   *[other] <num>{ $lots }</num> { -lot }s
}

# review: pending
dashboard-figure-nag = <num>{ $qty }</num> <unit>{ -nag }</unit>

# review: pending
dashboard-tile-trips = { -market-safar }s

# review: pending
dashboard-tile-trips-note = <num>{ $open }</num> open, cost left <money></money>

# review: pending
dashboard-tile-gardens = { -baghat }

# review: pending
dashboard-tile-gardens-note = <num>{ $open }</num> open

# review: pending
dashboard-tile-gardens-none = No open garden

# review: pending
dashboard-gardens-title = Open { -baghat }

# review: pending
dashboard-gardens-col-garden = { -bagh }

# review: pending
dashboard-gardens-col-balance = Running balance

# review: pending
dashboard-gardens-col-rent-due = Rent due

# review: pending
dashboard-gardens-all = All { -baghat }

# review: pending
dashboard-tile-lent = { -adhar-raqam } open

# review: pending
dashboard-tile-lent-overdue = <money></money> past its return-by date

# review: pending
dashboard-tile-expenses = Today's { -dukan-kharcha }

# review: pending
dashboard-tile-health = Books check

# review: pending
dashboard-tile-checking = Checking…

# review: pending
dashboard-tile-unknown = Could not be read

# review: pending
dashboard-health-ok = All good

# review: pending
dashboard-health-bad = A problem was found

# review: pending
dashboard-health-never = Not checked yet

# review: pending
dashboard-health-when = Checked <num>{ $when }</num>

# review: pending
dashboard-partners-title = Partners

# review: pending
dashboard-partner-open = Statement of { $name }

# review: pending
dashboard-partner-today = Today

# review: pending
dashboard-partner-month = This month

# review: pending
dashboard-partner-year = This year

# review: pending
dashboard-partner-total = { -kul-raqam }

# review: pending
dashboard-partners-empty-title = No partner yet

# review: pending
dashboard-partners-empty-help = Add the partners and their shares, and each one's share shows here.

# review: pending
dashboard-partners-setup = Set up the partners

# review: pending
dashboard-bikri-title = Today's { -bikri }

# review: pending
dashboard-bikri-col-number = { -bikri }

# review: pending
dashboard-bikri-col-seller = { -beopari }

# review: pending
dashboard-bikri-col-gross = { -kham-bikri }

# review: pending
dashboard-bikri-col-net = { -safi-bikri }

# review: pending
dashboard-bikri-col-state = Status

# review: pending
dashboard-bikri-number = { -bikri } <num>{ $number }</num>

# review: pending
dashboard-bikri-sellers = <name>{ $name }</name> and <num>{ $more }</num> more

# review: pending
dashboard-bikri-cancelled = Cancelled

# review: pending
dashboard-bikri-showing = Showing <num>{ $shown }</num> of <num>{ $count }</num>

# review: pending
dashboard-bikri-all = All of today's { -bikri }

# review: pending
dashboard-bikri-empty-title = No { -bikri } today

# review: pending
dashboard-bikri-empty-help = Press <key>{ $key }</key> to enter the first one.

# review: pending
dashboard-dues-title = Oldest buyer dues

# review: pending
dashboard-dues-col-party = { -gahak }

# review: pending
dashboard-dues-col-city = City

# review: pending
dashboard-dues-col-balance = Balance

# review: pending
dashboard-dues-col-days = Days

# review: pending
dashboard-dues-all = All buyer dues

# review: pending
dashboard-dues-empty-title = No buyer owes anything

# review: pending
dashboard-dues-empty-help = Dues show here, the oldest first.

# review: pending
dashboard-tasks-title = Today's tasks

# review: pending
dashboard-tasks-empty-title = Nothing needs your attention

# review: pending
dashboard-tasks-empty-help = When something needs doing, it shows here.

# review: pending
dashboard-charts-title = Charts

# review: pending
dashboard-chart-days-title = Last <num>30</num> days

# review: pending
dashboard-chart-days-label = { -kham-bikri } and { -commission } of each of the last 30 days

# review: pending
dashboard-chart-items-title = This month's { -commission } by item

# review: pending
dashboard-chart-items-label = { -commission } of the top items this month

# review: pending
dashboard-chart-gross = { -kham-bikri }

# review: pending
dashboard-chart-commission = { -commission }

# review: pending
dashboard-chart-empty-title = Nothing sold in this period yet

# review: pending
dashboard-chart-empty-help = The chart fills in as sales are saved.

# review: pending
dashboard-update-note = A new version <num>{ $version }</num> is ready. Install it from Settings when the day is done.

# review: pending
dashboard-update-open = Open Settings

# review: pending
soon-title = This part is coming soon

# review: pending
soon-help = It is being built. Go back to the dashboard.

# review: pending
soon-back = Go to the dashboard

## The dues tiles (plan 40, PRD 9.1, FR-10.10). Buyers' dues is dashboard-tile-dues above.

# review: pending
dashboard-tile-dues-sellers = Sellers' dues

# review: pending
dashboard-tile-dues-total = { -kul-agrahi }

# review: pending
dashboard-tile-dues-total-note = Buyers + sellers + { -doobi-raqam }

# review: pending
dashboard-tile-dead = { -doobi-raqam }

# review: pending
dashboard-tile-dead-note = { $count ->
    [one] <num>{ $count }</num> person
   *[other] <num>{ $count }</num> people
}

# review: pending
dashboard-tile-dead-none = Nobody

## The buyer dues page (plan 76 point 9, PRD FR-9.4): the tile "Buyers' dues" opens it.

# review: pending
dues-title = Buyers' dues

# review: pending
dues-help = Every { -gahak } who owes the shop, with what he owes now and the day his balance last changed. The total is the tile's figure. The age of each due is in report R20.


# review: pending
dues-search = Search by code or name

# review: pending
dues-label = Buyers who owe

# review: pending
dues-failed = The buyers' dues could not be read

# review: pending
dues-empty = No buyer owes anything

# review: pending
dues-empty-help = A buyer shows here as soon as he owes something.

# review: pending
dues-no-match = No buyer matches

# review: pending
dues-no-match-help = Change the words, or clear the search.

# review: pending
dues-clear = Clear the search

# review: pending
dues-col-last-change = Last change

## The commission page (plan 76 point 7, PRD FR-9.3): the tile "Today's commission" opens it.

# review: pending
commission-title = { -commission }

# review: pending
commission-help = Pick a period: one date, a month, a year or a range of dates. Pick a { -gahak } or a { -beopari } to see only that party's { -commission } and sales. For all parties each row shows the { -commission } of the seller and of the buyer, { -laga }, the total, the shared income and each partner's share of it. Open a day to see the sales that made it. The partners' shares add up to the shared income, not to the total commission.


# review: pending
commission-label = Commission by day

# review: pending
commission-failed = The commission could not be read

# review: pending
commission-col-date = Date

# review: pending
commission-col-seller = Seller { -commission }

# review: pending
commission-col-buyer = Buyer { -commission }

# review: pending
commission-col-laga = { -laga }

# review: pending
commission-col-total = Total { -commission }

# review: pending
commission-col-shared = Shared income

# review: pending
commission-empty = No commission in this month

# review: pending
commission-empty-help = Use the buttons below to go to another month.

# review: pending
commission-empty-other-help = Choose another period above, or use the buttons below.

# review: pending
commission-bikri = { -bikri } <num>{ $number }</num>

# review: pending
commission-posting-bikri-first = Sale

# review: pending
commission-posting-bikri-later = Later sale

# review: pending
commission-posting-reversal = Taken back

# review: pending
commission-posting-general = Journal

# review: pending
commission-posting-other = Other entry

# review: pending
commission-day-failed = The sales of this day could not be read

# review: pending
commission-day-empty = Nothing behind this day

# review: pending
commission-day-empty-help = No entry is left for this day.

# review: pending
commission-day-more = Showing <num>{ $shown }</num> of <num>{ $total }</num> entries of this day.

## The period and the party of the commission page (plan 77, PRD FR-9.3): one date, a month, a year or a range, and one buyer or seller.

# review: pending
commission-filters-label = Commission filters

# review: pending
period-label = Period

# review: pending
period-day = Date

# review: pending
period-month = Month

# review: pending
period-year = Year

# review: pending
period-range = Date range

# review: pending
commission-side-label = Party

# review: pending
commission-side-all = All

# review: pending
commission-side-buyer = { -gahak }

# review: pending
commission-side-seller = { -beopari }

# review: pending
commission-pick-buyer = Pick a { -gahak } to see his { -commission }

# review: pending
commission-pick-seller = Pick a { -beopari } to see his { -commission } and sales

# review: pending
commission-pick-help = Type his name or code in the box above, or choose All to see every party.



# review: pending
period-goto-date = Go to date

# review: pending
period-goto-date-hint = Type a date, for example 10-10-2026. A short form works too: 10 is the 10th of this month.


# review: pending
period-goto-year = Go to year

# review: pending
period-goto-year-hint = Type a year, for example 2026. A short form works too: 26 is 2026.

# review: pending
period-goto-year-placeholder = YYYY

# review: pending
period-date-nav-label = Move by day

# review: pending
period-date-earlier = Previous day

# review: pending
period-date-later = Next day

# review: pending
period-date-latest-note = This is today

# review: pending
period-date-latest-button = Today

# review: pending
period-date-title = <num>{ $date }</num>

# review: pending
period-year-nav-label = Move by year

# review: pending
period-year-earlier = Previous year

# review: pending
period-year-later = Next year

# review: pending
period-year-first-note = This is the first year with entries

# review: pending
period-year-latest-note = This is the current year

# review: pending
period-year-latest-button = This year

# review: pending
period-year-title = Year <num>{ $year }</num>

## The kharch page (plan 77, PRD FR-9.5): the tile "Today's Shop Expense" opens it.

# review: pending
kharch-title = { -kharcha }

# review: pending
kharch-help = Two parts for one period (one date, a month, a year or a range): the { -dukan-kharcha } of each head with the total of the whole shop, and what each { -shirakat-dar } spent himself ({ -zati-kharcha }) with the total. Open a figure to see its entries: date, amount and reason. The partner part waits for the partner code; the shop part shows without it.


# review: pending
kharch-filters-label = { -kharcha } filters


# review: pending
commission-col-month = Month

# review: pending
commission-col-sale = { -kham-bikri }

# review: pending
commission-col-purchase = Purchase value

# review: pending
commission-col-charged = { -commission } charged to him

# review: pending
commission-col-goods = { -commission } on his goods

# review: pending
commission-empty-day = No { -commission } on this date

# review: pending
commission-empty-year = No { -commission } in this year

# review: pending
commission-empty-range = No { -commission } in these dates

# review: pending
commission-empty-party = No { -commission } from this party in this period

# review: pending
commission-empty-party-help = Choose another period above, or choose All.

# review: pending
commission-party-lost = This party could not be found

# review: pending
commission-party-lost-help = Pick him again in the box above, or choose All.



## The tile of garden owners and others (plan 76 point 10, PRD FR-9.1, FR-16.13): it replaces the last backup tile.

# review: pending
dashboard-tile-others = Garden owners' and others' dues

# review: pending
dashboard-tile-others-note = { $count ->
    [one] <num>{ $count }</num> person
   *[other] <num>{ $count }</num> people
}
