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
dashboard-tile-backup = Last backup

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
dashboard-bikri-col-truck = Truck

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
