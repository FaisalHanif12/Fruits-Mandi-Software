### The statement print of one party on Party Ledgers (plan 74 T3, PRD FR-12.47). The ids that start with
### `print-statement-` are the ones the backend sends (the title, the labels of a day's lines and of the
### totals) and the words the template draws itself; the ids that start with `statement-print-` are the box.

## The page

# The title of the print and of its header.
# review: pending
print-statement-title = Statement

# review: pending
print-statement-col-date = Date

# review: pending
print-statement-col-details = Details

# review: pending
print-statement-col-amount = Amount

# review: pending
print-statement-col-balance = { -baqaya }

# review: pending
print-statement-opening = { -sabiqa-raqam }

# review: pending
print-statement-day-end = Balance at the end of the day

# One line of goods: "Mango 20 × 1,000 = 20,000". The numbers are drawn by the template as numbers.
# review: pending
print-statement-item = <item>{ $item }</item> <num>{ $qty }</num> <num>×</num> <num>{ $rate }</num> <num>=</num> <num>{ $amount }</num>

# An item sold by weight: its kilograms and its rate for each kilo.
# review: pending
print-statement-item-weight = <item>{ $item }</item> <num>{ $weight }</num> kg <num>×</num> <num>{ $rate }</num> <num>=</num> <num>{ $amount }</num>

# review: pending
print-statement-paid-on-spot = { -mauqa-par-jama }

# The day's bill of a buyer and the day's net sale of a seller (the backend sends one of the two).
# review: pending
print-statement-bill-buyer = Bill of the day

# review: pending
print-statement-bill-seller = { -safi-bikri } of the day

# What was entered that day: the backend sends the kind of each entry.
# review: pending
print-statement-entry-receipt = { -wasooli }

# review: pending
print-statement-entry-payment = { -adaigi }

# review: pending
print-statement-entry-advance = { -peshgi }

# Anything else posted that day (cash lent, a جنرل line): one amount for each side, as on the party day slip.
# review: pending
print-statement-entry-other-debit = Other entry ({ -banaam })
# review: pending
print-statement-entry-other-credit = Other entry ({ -jama })

# The totals of the period.
# review: pending
print-statement-total-bought = Total bought

# review: pending
print-statement-total-sold = Total sold

# review: pending
print-statement-total-charges = of which { -commission } and { -laga }

# review: pending
print-statement-total-paid-on-spot = Total { -mauqa-par-jama }

# review: pending
print-statement-total-received = Total received

# review: pending
print-statement-total-paid = Total paid

# A seller's deductions (the نقد is not one of them), what was given to him in cash with his lots, and the totals
# of the advances and of the other entries.
# review: pending
print-statement-total-deductions = Total deductions
# review: pending
print-statement-total-cash = Total { -naqd }
# review: pending
print-statement-total-advanced = Total { -peshgi }
# review: pending
print-statement-total-other-debit = Total other entries ({ -banaam })
# review: pending
print-statement-total-other-credit = Total other entries ({ -jama })
# review: pending
print-statement-total-closing = Closing { -baqaya }

## The box

# review: pending
statement-print-button = Statement print

# review: pending
statement-print-title = Statement print

# review: pending
statement-print-help = A short, clean print of this party's ledger for a period, the oldest date first. Choose the period, then Print or Save as PDF.

# review: pending
statement-print-period = Period

# review: pending
statement-print-last-10 = Last <num>10</num> days

# review: pending
statement-print-last-30 = Last <num>30</num> days

# review: pending
statement-print-last-6-months = Last <num>6</num> months

# review: pending
statement-print-custom = Own dates

# review: pending
statement-print-from = From

# review: pending
statement-print-to = To

# The dates the print is for, under the choice of the period.
# review: pending
statement-print-range = <num>{ $from }</num> to <num>{ $to }</num>
