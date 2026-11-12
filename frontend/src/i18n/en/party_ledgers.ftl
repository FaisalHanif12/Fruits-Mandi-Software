### Party ledgers (plan 66, PRD FR-12.43): each buyer's and each seller's own ledger, in three levels.
### Level 1 is the list of parties, level 2 the statement of one party, level 3 the day slip.
### Every Urdu line says what its English line says (NFR-URD-29).

## The screen

# review: pending
party-ledgers-title = Party Ledgers

# review: pending
party-ledgers-help = First choose Buyer or Seller and press Enter on a party: his ledger shows one row for each date on which he has an entry, 30 days at a time. Press Enter on a date: that day's slip opens right under its row, and you can print it there. Use the buttons under the table, or Page Up and Page Down, to move 30 days back or forward. Esc goes back to the list.

# review: pending
party-ledgers-side-label = Kind of party

# review: pending
party-ledgers-side-buyer = { -gahak }

# review: pending
party-ledgers-side-seller = { -beopari }

## Level 1: the list of parties

# review: pending
party-ledgers-search = Type a code or a name

# review: pending
party-ledgers-list-label = List of party ledgers

# review: pending
party-ledgers-list-failed = The party ledgers could not be opened

# review: pending
party-ledgers-col-code = Code

# review: pending
party-ledgers-col-name = Name

# review: pending
party-ledgers-col-debit = { -banaam }

# review: pending
party-ledgers-col-credit = { -jama }

# review: pending
party-ledgers-empty-buyer = No { -gahak } yet

# review: pending
party-ledgers-empty-seller = No { -beopari } yet

# review: pending
party-ledgers-empty-help = Add a buyer or a seller in the Index. { -chukta-khata } buyers have no ledger of their own.

# review: pending
party-ledgers-empty-action = Open the Index

# review: pending
party-ledgers-no-match = No party found

# review: pending
party-ledgers-no-match-help = Try another code or name.

# review: pending
party-ledgers-clear = Clear the search

## Level 2: the statement of one party

# review: pending
party-ledgers-back-list = Party list

# review: pending
party-ledgers-statement-label = Ledger of this party

# review: pending
party-ledgers-statement-failed = The ledger could not be opened

# review: pending
party-ledgers-from = From

# review: pending
party-ledgers-to = To

# review: pending
party-ledgers-col-date = Date

# review: pending
party-ledgers-col-previous = { -sabiqa-raqam }

# review: pending
party-ledgers-col-buyer-charged = Charged

# review: pending
party-ledgers-col-buyer-paid = Paid

# review: pending
party-ledgers-col-seller-charged = Credited

# review: pending
party-ledgers-col-seller-paid = Paid to him

# review: pending
party-ledgers-col-balance = New { -baqaya }

# review: pending
party-ledgers-statement-no-match = No entries in this period

# review: pending
party-ledgers-statement-no-match-help = Try other first and last dates.

# review: pending
party-ledgers-clear-dates = Clear the dates

## Level 2: 30 days at a time and a day opened in place (PRD FR-12.47, plan 74)

# review: pending
party-ledgers-nav-label = Move 30 days back or forward

# review: pending
party-ledgers-back-days = Earlier <num>{ $days }</num> days

# review: pending
party-ledgers-forward-days = Later <num>{ $days }</num> days

# review: pending
party-ledgers-range = <num>{ $from }</num> to <num>{ $to }</num>

# review: pending
party-ledgers-nav-latest = Already the latest days

# review: pending
party-ledgers-nav-latest-button = Back to the days up to today

# review: pending
party-ledgers-goto = Go to date

# review: pending
party-ledgers-goto-hint = Type a day, for example 05-11-2026. The entry of that day opens. A short form works too: 5 is the 5th of this month.

# review: pending
party-ledgers-goto-placeholder = DD-MM-YYYY

# review: pending
party-ledgers-goto-none = No entry on this date: <num>{ $date }</num>. The days around it are below.

# review: pending
party-ledgers-window-empty = No entries in these days

# review: pending
party-ledgers-window-empty-help = Press the earlier button below, or go to a date.

# review: pending
party-ledgers-day-heading = Slip of <num>{ $date }</num>

# review: pending
party-ledgers-day-print-name = Print preview of the slip of { $date }

## Level 3: the day slip

# review: pending
party-ledgers-back-statement = Back to the ledger

# review: pending
party-ledgers-day-title = <name>{ $name }</name> <code>{ $code }</code>, <date>{ $date }</date>
