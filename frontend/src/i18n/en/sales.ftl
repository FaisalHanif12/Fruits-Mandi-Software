### The sales page (route /sales; PRD FR-9.9, FR-19.11, plan 83): the tile Today's sale opens it.

## The page

# review: pending
sales-title = { -bikri }s

# review: pending
sales-help = Pick a period: one date, a month, a year or a range of dates. Each row is one day with the number of { -bikri }s, the { -nag } sold, the { -kham-bikri }, the bill of the { -gahakon }, what was paid on the spot ({ -naqdi }) and what was left on account ({ -udhari }). Pick a { -gahak } or a { -beopari }, an item, or the goods (consigned, trip or garden) to see only those. Open a day to see its { -bikri }s, and open any one of them.

# review: pending
sales-label = { -bikri }s by day

# review: pending
sales-failed = The { -bikri }s could not be read

# review: pending
sales-today = Today

## The columns of the days

# review: pending
sales-col-date = Date

# review: pending
sales-col-month = Month

# review: pending
sales-col-count = Number of { -bikri }s

# review: pending
sales-col-qty = { -nag }

# review: pending
sales-col-gross = { -kham-bikri }

# review: pending
sales-col-buyer-total = Buyers' bill

# review: pending
sales-col-cash = { -naqdi }

# review: pending
sales-col-credit = { -udhari }

## The empty states

# review: pending
sales-empty = No { -bikri }s in this month

# review: pending
sales-empty-help = Use the buttons below to go to another month.

# review: pending
sales-empty-other-help = Choose another period above, or use the buttons below.

# review: pending
sales-empty-day = No { -bikri }s on this date

# review: pending
sales-empty-year = No { -bikri }s in this year

# review: pending
sales-empty-range = No { -bikri }s in these dates

# review: pending
sales-empty-filtered = No { -bikri }s match these filters in this period

# review: pending
sales-empty-filtered-help = Choose another period or filter above, or choose All.

## The filters: the period (the boxes of the commission page), the party, the item and the goods

# review: pending
sales-filters-label = { -bikri }s filters

# review: pending
sales-side-label = { commission-side-label }

# review: pending
sales-side-all = { commission-side-all }

# review: pending
sales-side-buyer = { commission-side-buyer }

# review: pending
sales-side-seller = { commission-side-seller }

# review: pending
sales-pick-buyer = Pick a { -gahak } to see his { -bikri }s

# review: pending
sales-pick-seller = Pick a { -beopari } to see his { -bikri }s

# review: pending
sales-pick-help = Type his name or code in the box above, or choose All to see every party.

## One open day

# review: pending
sales-day-label = The { -bikri }s of this day

# review: pending
sales-day-failed = The { -bikri }s of this day could not be read

# review: pending
sales-day-empty = No { -bikri }s behind this day

# review: pending
sales-day-empty-help = No sale is left for this day in these filters.

# review: pending
sales-day-more = Showing <num>{ $shown }</num> of <num>{ $total }</num> { -bikri }s of this day.

# review: pending
sales-day-col-bikri = { -bikri }

# review: pending
sales-day-col-sellers = { -beopari } and { -lot }

# review: pending
sales-day-col-qty = { -nag }

# review: pending
sales-day-col-gross = { -kham-bikri }

# review: pending
sales-day-col-buyer-total = Buyers' bill

# review: pending
sales-day-col-cash = { -naqdi }

# review: pending
sales-day-col-credit = { -udhari }

# review: pending
sales-day-total = { -meezan }: { $count ->
    [one] <num>{ $count }</num> { -bikri }
   *[other] <num>{ $count }</num> { -bikri }s
}

# The button that opens one بکری from the list of the day.
# review: pending
sales-day-open = { -bikri } <num>{ $number }</num>

## The title of the print and of the Excel file (the backend writes these ids into the print data; its columns and
## facts use the ids of the other prints)

# review: pending
print-sales-title = { -bikri }s of the month

# review: pending
print-sales-title-period = { -bikri }s of the period

