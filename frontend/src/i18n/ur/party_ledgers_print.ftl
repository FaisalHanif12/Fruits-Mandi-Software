### The statement print of one party on Party Ledgers (plan 74 T3, PRD FR-12.47). The ids that start with
### `print-statement-` are the ones the backend sends (the title, the labels of a day's lines and of the
### totals) and the words the template draws itself; the ids that start with `statement-print-` are the box.

## The page

# The title of the print and of its header.
# review: pending
print-statement-title = اسٹیٹمنٹ

# review: pending
print-statement-col-date = تاریخ

# review: pending
print-statement-col-details = تفصیل

# review: pending
print-statement-col-amount = رقم

# review: pending
print-statement-col-balance = { -baqaya }

# review: pending
print-statement-opening = { -sabiqa-raqam }

# review: pending
print-statement-day-end = دن کے آخر میں { -baqaya }

# One line of goods: "آم 20 × 1,000 = 20,000". The numbers are drawn by the template as numbers.
# review: pending
print-statement-item = <item>{ $item }</item> <num>{ $qty }</num> <num>×</num> <num>{ $rate }</num> <num>=</num> <num>{ $amount }</num>

# An item sold by weight: its kilograms and its rate for each kilo.
# review: pending
print-statement-item-weight = <item>{ $item }</item> <num>{ $weight }</num> کلو <num>×</num> <num>{ $rate }</num> <num>=</num> <num>{ $amount }</num>

# review: pending
print-statement-paid-on-spot = { -mauqa-par-jama }

# The day's bill of a buyer and the day's net sale of a seller (the backend sends one of the two).
# review: pending
print-statement-bill-buyer = اس دن کا بل

# review: pending
print-statement-bill-seller = اس دن کی { -safi-bikri }

# What was entered that day: the backend sends the kind of each entry.
# review: pending
print-statement-entry-receipt = { -wasooli }

# review: pending
print-statement-entry-payment = { -adaigi }

# review: pending
print-statement-entry-advance = { -peshgi }

# Anything else posted that day (cash lent, a جنرل line): one amount for each side, as on the party day slip.
# review: pending
print-statement-entry-other-debit = دیگر اندراج ({ -banaam })
# review: pending
print-statement-entry-other-credit = دیگر اندراج ({ -jama })

# The totals of the period.
# review: pending
print-statement-total-bought = کل خریداری

# review: pending
print-statement-total-sold = کل فروخت

# review: pending
print-statement-total-charges = جس میں { -commission } اور { -laga } شامل ہیں

# review: pending
print-statement-total-paid-on-spot = کل { -mauqa-par-jama }

# review: pending
print-statement-total-received = کل وصولی

# review: pending
print-statement-total-paid = کل ادائیگی

# A seller's deductions (the نقد is not one of them), what was given to him in cash with his lots, and the totals
# of the advances and of the other entries.
# review: pending
print-statement-total-deductions = کل کٹوتیاں
# review: pending
print-statement-total-cash = کل { -naqd }
# review: pending
print-statement-total-advanced = کل { -peshgi }
# review: pending
print-statement-total-other-debit = کل دیگر اندراج ({ -banaam })
# review: pending
print-statement-total-other-credit = کل دیگر اندراج ({ -jama })
# review: pending
print-statement-total-closing = آخری { -baqaya }

## The box

# review: pending
statement-print-button = اسٹیٹمنٹ پرنٹ

# review: pending
statement-print-title = اسٹیٹمنٹ پرنٹ

# review: pending
statement-print-help = اس پارٹی کے کھاتے کا کسی مدت کے لیے مختصر اور صاف پرنٹ، پرانی تاریخ سب سے پہلے۔ مدت چنیں، پھر پرنٹ کریں یا PDF محفوظ کریں۔

# review: pending
statement-print-period = مدت

# review: pending
statement-print-last-10 = آخری <num>10</num> دن

# review: pending
statement-print-last-30 = آخری <num>30</num> دن

# review: pending
statement-print-last-6-months = آخری <num>6</num> مہینے

# review: pending
statement-print-custom = اپنی تاریخیں

# review: pending
statement-print-from = از

# review: pending
statement-print-to = تا

# The dates the print is for, under the choice of the period.
# review: pending
statement-print-range = <num>{ $from }</num> تا <num>{ $to }</num>
