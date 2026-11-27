### Prints (plan 14, PRD FR-12.12, FR-7.36, FR-19.3 to FR-19.7): the seller bill, the buyer slips,
### the print box and the Printing settings. The backend sends the ids of
### titles, facts, roles and amounts (plan 14 contract 6) and the template resolves them in the
### PRINT language, which can differ from the app language.

## Titles

# The title of the sale book's seller bill (R1), as it was: the seller's word and "bill".
# review: pending
print-seller-bill-title = { -beopari } Bill

# The title of one seller's bill when it names him (plan 78 A3). $name: the seller, with his tag.
# review: pending
print-seller-bill-named-title = Bill: { $name }

# review: pending
print-buyer-slip-title = { -gahak } Slip

## Facts under the title

# review: pending
print-fact-number = { -bikri } No.

# review: pending
print-fact-date = Date

# review: pending
print-fact-bill-date = Bill date

# review: pending
print-fact-truck = Truck No.

## Who the page is for

# review: pending
print-party-seller = { -beopari }

# review: pending
print-party-buyer = { -gahak }

# review: pending
print-party-code = Code

# review: pending
print-party-phone = Phone

## The amounts of the seller bill (PRD 4.4)

# review: pending
print-gross-sale = { -kham-bikri }

# review: pending
print-freight = { -kiraya }

# review: pending
print-commission = { -commission }

# review: pending
print-labour = { -mazdoori }

# review: pending
print-munshiana = { -munshiana }

# review: pending
print-cash-paid = { -naqd-adaigi }

# The advance a seller took when he brought the goods of this sale (plan 95 item 11, PRD FR-12.52): a line of the
# seller bill, printed only when it is not 0.
# review: pending
print-seller-advance = { -peshgi }

# review: pending
print-store-rent = { -kiraya-store }

# review: pending
print-phone-charge = { -phone }

# review: pending
print-total-deductions = Total deductions

# review: pending
print-net-sale = { -safi-bikri }

# The net is below zero (PRD FR-12.7): the amount is printed without a minus sign.
# review: pending
print-seller-owes = { -beopari } owes

## The amounts of a buyer slip

# review: pending
print-items-value = Value of goods

# review: pending
print-buyer-commission = { -commission }

# review: pending
print-laga = { -laga }

# review: pending
print-buyer-pays = Total amount

# review: pending
print-in-words = Amount in words

# review: pending
print-payment-cash = { -naqdi } sale: paid

# review: pending
print-payment-credit = { -udhari } sale: payment due

# The two lines of a buyer slip when he paid part of the bill on the spot (PRD FR-12.44).
# review: pending
print-paid-on-spot = Cash

# review: pending
print-left-on-account = Credit

## Column heads of the goods table

# review: pending
print-col-date = Date

# review: pending
print-col-buyer = { -gahak }

# review: pending
print-col-item = { -qism-maal }

# review: pending
print-col-packing = { -packing }

# review: pending
print-col-qty = Qty

# review: pending
print-col-weight = Weight (kg)

# review: pending
print-col-rate = Rate

# review: pending
print-col-amount = Amount

## The page footer and the stamp

# review: pending
print-printed-at = Printed

# review: pending
print-page-label = Page

# review: pending
print-page-of = of

# review: pending
print-cancelled-stamp = Cancelled

# review: pending
print-preview-empty = There is nothing to print in this document.

## The documents of a sale (the names in the print box and in the Printing settings)

# review: pending
print-doc-seller-bill = { -beopari } bill

# review: pending
print-doc-buyer-slips = { -gahak } slips

## The print box (opened with F9 on the sale book)

# review: pending
print-box-title = Print { -bikri } { $number }

# $key is the key that saves and prints, for example Ctrl+Enter.
# review: pending
print-box-help = The documents you tick here are also the ones { $key } prints when it saves a { -bikri }.

# review: pending
print-box-language = Print language

# review: pending
print-box-docs = Documents

# review: pending
print-box-none-ticked = Tick at least one document.

# review: pending
print-box-only-buyer = { -gahak } slip for

# review: pending
print-box-all-buyers = All buyers

# review: pending
print-box-copies = Copies

# review: pending
print-box-copies-hint = Leave empty to use the number in Settings.

# review: pending
print-box-to = Send to

# review: pending
print-box-to-printer = Printer

# review: pending
print-box-to-pdf = PDF file

# review: pending
print-box-print = Print

# The short word of the print button when the top row of the sale book is short of room (plan 95).
# review: pending
print-box-print-short = Print

# review: pending
print-box-save-pdf = Save PDF

# review: pending
print-box-preview = Preview

# review: pending
print-box-close = Close

# $docs are the printed documents, for example "Seller bill and Buyer slips".
# review: pending
print-box-done-printed = Sent to the printer: { $docs }

# review: pending
print-box-done-pdf = PDF files saved:

# $docs are the printed documents. On a Mac Print and Save PDF both open the system print window.
# review: pending
print-box-done-dialog = The print window is open: { $docs }

# Shown on a Mac under the print choices (plan 69). The words PDF and Save as PDF are the Mac's own.
# review: pending
print-dialog-hint = On a Mac the system print window opens. To make a PDF, choose PDF in it, then Save as PDF.

# review: pending
print-box-nothing-printed = There was nothing to print.

# review: pending
print-save-first = Save the { -bikri } first, then print it.

# Shown after Ctrl+Enter when the documents went to the printer.
# review: pending
print-toast-printed = Sent to the printer

# Shown after Ctrl+Enter on a Mac, where the system print window opens instead (plan 69).
# review: pending
print-toast-dialog = The print window is open

## The second print button of the sale book: ادھاری تفصیل (R5, plan 68)

# review: pending
print-adhari-title = Print { report-r5-title }

# review: pending
print-menu-help = Each print does one job. Choose one, then look at the page below.

# review: pending
print-menu-scope = Which lines

# $date is the day of the { -bikri } on screen.
# review: pending
print-menu-whole-day = Whole day (<num>{ $date }</num>)

# review: pending
print-menu-this-bikri = This { -bikri } only (<num>{ $number }</num>)

# The save had carried lots: the new { -bikri } and the old { -bikri } of each carried lot. $numbers: their numbers, new first.
# review: pending
print-menu-this-save = This save only ({ -bikri } <num>{ $numbers }</num>)

## The preview

# review: pending
print-preview-title = Preview

# review: pending
print-preview-help = The document will print on paper like this.

# review: pending
print-preview-failed = The preview could not be opened

## Settings, tab Printing (PRD Section 21)

# review: pending
print-settings-paper = Paper size

# review: pending
print-settings-paper-help = Choose the paper for each document. It prints at this size.

# review: pending
print-paper-a4 = A4 paper

# review: pending
print-paper-a5 = A5 paper

# review: pending
print-paper-mm80 = 80 mm roll

# review: pending
print-settings-printer = Printer

# review: pending
print-settings-printer-default = Default printer

# review: pending
print-settings-printers-failed = The list of printers could not be read

# review: pending
print-settings-copies = Copies

# review: pending
print-settings-copies-hint = A number from 1 to 99.

# review: pending
print-settings-after-save = Print after save

# $key is the key that saves and prints, for example Ctrl+Enter.
# review: pending
print-settings-after-save-hint = { $key } saves the { -bikri } or the voucher and prints these documents.

# review: pending
print-settings-load-failed = Printing settings could not be opened

## The receipt slip of a voucher (plan 15, PRD FR-15.2). The backend sends these ids (contract 9).

# review: pending
print-receipt-slip-title = { -wasooli } slip

# review: pending
print-payment-slip-title = { -adaigi } slip

# review: pending
print-advance-slip-title = { -peshgi-khata } slip

# review: pending
print-fact-voucher-number = No.

# review: pending
print-fact-reason = Reason

# review: pending
print-fact-collector = Collector

# review: pending
print-fact-narration = Narration

# review: pending
print-party-account = Account

# review: pending
print-slip-amount-received = Amount received

# review: pending
print-slip-amount-paid = Amount paid

# review: pending
print-slip-amount-advance = { -peshgi } given

# review: pending
print-doc-receipt-slip = Receipt slip

## R5 and the buyer slips in the client's layout; the header of every print (plan 17e, PRD R5, FR-19.13).

# Read by a screen reader: the name of the box in the header.
# review: pending
print-header-trade-mark = { -trade-mark }

# R5 (PRD 19.2), in the client's layout.
# review: pending
print-r5-title = { -udhari } details

# review: pending
print-r5-col-serial = No.

# review: pending
print-r5-col-party = Party

# review: pending
print-r5-col-details = Details

# review: pending
print-r5-col-value = Amount

# Value + buyer commission + لاگا (PRD C5).
# review: pending
print-r5-col-total = Amount after { -laga }

# A line of an item sold by weight in the buyer's bill (R5): the weight in kilograms and the rate for each
# kilo, written the client's way. A line of a نگ item is written by the backend ("(کیلا 20 . 600)").
# $item: the item in the print language; $weight: kilograms; $rate: the rate for each kilo.
# review: pending
print-r5-line-weight = ({ $item } { $weight } kg . { $rate })

## The collection list on paper (plan 17e, PRD FR-15.6, FR-7.61).

# The name of the document in the Printing settings and in the print box.
# review: pending
print-doc-collection-list = { -agrahi } list

# The title in the header of the collection list.
# review: pending
print-collection-list-title = { -agrahi } list

# A fact under the title: the city the list is filtered by.
# review: pending
print-fact-city = City

# The heading of a group with no collector, and of one with no city.
# review: pending
print-collection-no-collector = No collector set

# review: pending
print-collection-no-city = No city set

# review: pending
print-collection-col-serial = No.

# The short code of the buyer.
# review: pending
print-collection-col-code = Code

# review: pending
print-collection-col-party = Buyer

# review: pending
print-collection-col-dues = Credit

# The date of the oldest bill still open, and how many days old it is.
# review: pending
print-collection-col-oldest = Oldest bill

# review: pending
print-collection-col-days = Days

# Left empty, for the collector to write in.
# review: pending
print-collection-col-note = Note

# The total of one collector and city.
# review: pending
print-collection-group-total = Total

# The number of buyers on the list.
# review: pending
print-collection-buyers = Buyers on this list

# The total of THIS list only: the buyers planned for this day, not every buyer who owes.
# review: pending
print-collection-total = Total of this list

# The role word before a person's name in the print header (PRD Section 21, FR-19.13, v1.12): the
# owners print first, then the munshi.
# review: pending
print-role-owner = Owner

# review: pending
print-role-munshi = Munshi

# A line of an item sold by weight on a party's day slip on the 80 mm roll (PRD A73): the kilos and the
# rate per kilo, then the kilos of an item total.
# review: pending
print-slip-weight-line = { $weight } kg @ { $rate }

# review: pending
print-slip-weight-kg = { $weight } kg

# The print and the Excel of پچھلی بکری (plan 72 P8, PRD 12.7). The ids are the ones the backend sends
# (`bikri_list_print_data`, `bikri_list_export_excel`): the title, the facts of the header, the column
# of the time, the status of an edited بکری, and the heading of today's section and of each day.
# review: pending
bikri-list-print-title = Past Sales

# review: pending
bikri-list-print-all = Every { -bikri } shown

# review: pending
bikri-list-print-search = Search

# review: pending
bikri-list-col-time = Time

# The status of a بکری that was edited: its status, then the mark (PRD FR-7.71). `$status` is a message.
# review: pending
bikri-list-status-edited = { $status } · { edit-mark }

# review: pending
bikri-list-today-heading = Today's { -bikri }s, { $date }: { $count }

# review: pending
bikri-list-day-heading = { $date }: { -bikri }s { $count }

# review: pending
bikri-list-day-heading-empty = { $date }: No { -bikri }s

## The buyer parchi and the overall sale print (plan 76 point 3, PRD FR-19.15, FR-12.12).

# The title of the buyer parchi print (the backend sends it in the header).
# review: pending
print-parchi-title = Buyer slips

# The title of the overall sale print: the seller slips and the buyer slips of one { -bikri } or of the day.
# review: pending
print-sale-title = Whole sale

# The heading of the sellers' part and of the buyers' part of the overall sale print.
# review: pending
print-sale-sellers = Sellers

# review: pending
print-sale-buyers = Buyers

# The empty line at the foot of a parchi, for the collector to write what he got.
# review: pending
print-parchi-collect = { -wasooli }

# review: pending
print-parchi-empty = No buyer has a parchi for this day

# review: pending
print-parchi-label = Buyer slips, one for each buyer

# review: pending
print-sale-label = Whole sale, the sellers and the buyers

# review: pending
print-menu-kind = What to print

# review: pending
print-menu-buyers = Buyer slips

# review: pending
print-menu-sale = Whole sale

## The titles, facts, totals and column heads of the table prints of plan 76 (the backend writes their ids into the print data).

# review: pending
print-commission-title = { -commission } of the month

# review: pending
print-buyer-dues-title = Buyers' dues

# review: pending
print-collection-sheet-title = { report-r5-title }

# review: pending
print-partner-month-title = { -shirakat-dar } statement by month

# review: pending
print-fact-month = Month

# review: pending
print-fact-search = Search

# review: pending
print-fact-partner = { -shirakat-dar }

# review: pending
print-commission-total = { -meezan }

# review: pending
print-buyer-dues-total = { -meezan }

# review: pending
print-collection-sheet-total = { -meezan }

# review: pending
print-collection-sheet-dead-total = { -doobi-raqam }, listed apart

# review: pending
print-collection-sheet-credit-total = { -gahak } in { -jama }, listed apart

# review: pending
print-partner-month-total = { -meezan }

# review: pending
print-partner-month-opening = Opening { -kul-raqam }

# review: pending
print-commission-col-date = Date

# review: pending
print-commission-col-seller = Seller { -commission }

# review: pending
print-commission-col-buyer = Buyer { -commission }

# review: pending
print-commission-col-laga = { -laga }

# review: pending
print-commission-col-total = Total { -commission }

# review: pending
print-commission-col-shared = Shared income

# review: pending
print-commission-col-partner = { -shirakat-dar }

# review: pending
print-commission-title-period = { -commission } of the period

# review: pending
print-fact-year = Year

# review: pending
print-fact-from = From

# review: pending
print-fact-to = To

# review: pending
print-fact-seller = { -beopari }

# review: pending
print-fact-buyer = { -gahak }

# review: pending
print-commission-col-sale = { -kham-bikri }

# review: pending
print-commission-col-purchase = Purchase value

# review: pending
print-commission-col-goods = { -commission } on his goods


# review: pending
print-buyer-dues-col-party = { -gahak }

# review: pending
print-buyer-dues-col-city = City

# review: pending
print-buyer-dues-col-balance = Balance

# review: pending
print-buyer-dues-col-last = Last change

# review: pending
print-collection-sheet-col-code = Code

# review: pending
print-collection-sheet-col-party = { -gahak }

# review: pending
print-collection-sheet-col-detail = Goods

# review: pending
print-collection-sheet-col-previous = { -sabiqa-raqam }

# review: pending
print-collection-sheet-col-bill = Today's Bill

# review: pending
print-collection-sheet-col-paid = Cash

# review: pending
print-collection-sheet-col-due = Total Credit

# review: pending
print-partner-month-col-date = Date

# review: pending
print-partner-month-col-commission = { -commission }

# review: pending
print-partner-month-col-expense = { -kharcha }

# review: pending
print-partner-month-col-gardens-trips = { -bagh } / { -safar }

# review: pending
print-partner-month-col-fruit = { -phal }

# review: pending
print-partner-month-col-taken-returned = Taken and returned

# review: pending
print-partner-month-col-capital = { -sarmaya }

# The advances to sellers that went out and came back in the month (PRD FR-17.13). Same word as the column of the screen, `partners-month-col-advances`.
# review: pending
print-partner-month-col-advances = { -peshgi }

# review: pending
print-partner-month-col-net = Day's net

# review: pending
print-partner-month-col-running = { -kul-raqam }

## Plan 78: the prints that each do one job, with the party's name in the title (PRD FR-19.15, FR-12.12, plan 78 A2 to A6).

# The title of a buyer's day slip and of a seller's day slip: the words, then the party's name.
# $name: the name of the party in the print language, with his tag.
# review: pending
print-buyer-day-slip-title = Day slip: { $name }

# review: pending
print-seller-day-slip-title = Day slip: { $name }

# The title of the print of every seller's bill (the frame of the job).
# review: pending
print-seller-bills-title = Seller bills

# The side of a balance on a slip, written for the party who reads it (the party's own words, not the books').
# review: pending
print-slip-side-dr = You owe

# review: pending
print-slip-side-cr = The shop owes you

## The whole sale print: the sellers table, then the buyers table (PRD FR-19.15).

# review: pending
print-sale-buyers-title = Buyers

# review: pending
print-sale-buyers-total = { -meezan }

# review: pending
print-sale-col-buyer = Buyer

# What the buyer bought, in short: (item quantity . rate).
# review: pending
print-sale-col-bought = Goods

# The value of the goods, the buyer's commission and لاگا: the three parts of the bill (plan 95 item 13, PRD FR-19.15).
# review: pending
print-sale-col-value = { report-r35-goods }

# review: pending
print-sale-col-commission = { -commission }

# review: pending
print-sale-col-laga = { -laga }

# The advance of a lot, in the sellers table (plan 95 item 11, PRD FR-12.52).
# review: pending
print-sale-col-advance = { -peshgi }

# review: pending
print-sale-col-bill = Bill

# review: pending
print-sale-col-cash = Cash

# review: pending
print-sale-col-credit = Credit

## ادھاری تفصیل: how much the shop has to take from its buyers (PRD R5, FR-19.15).

# review: pending
print-adhari-sheet-title = { report-r5-title }

# The totals row of today's buyers.
# review: pending
print-adhari-sheet-total = Today's buyers

# The buyers who owed at the start of the day and bought nothing today, as one total.
# review: pending
print-adhari-sheet-others = Other buyers who did not buy today

# Everything every buyer owes the shop.
# review: pending
print-adhari-sheet-all = Total credit, all buyers

# review: pending
print-adhari-sheet-col-party = Buyer

# review: pending
print-adhari-sheet-col-detail = Goods Today

# review: pending
print-adhari-sheet-col-bill = Today's Bill

# review: pending
print-adhari-sheet-col-cash = Cash

# review: pending
print-adhari-sheet-col-today-credit = Credit Today

# review: pending
print-adhari-sheet-col-previous = { -sabiqa-raqam }

# review: pending
print-adhari-sheet-col-total = Total Credit

## The print menu of the sale book, F9 (PRD FR-12.12, plan 78 A1): three prints, each with one purpose.

# review: pending
print-menu-sellers = Seller bills

# What each print is for, one line, shown under the choice (plan 78 A0).
# review: pending
print-menu-buyers-purpose = What each buyer bought today and what he owes. Several slips to a page, with cut lines.

# review: pending
print-menu-sellers-purpose = What each seller sold today, his deductions and what the shop owes him. One bill for each seller.

# review: pending
print-menu-sale-purpose = Everything sold today in one place: every seller and every buyer, on one or two pages.

# review: pending
print-seller-bills-label = Seller bills, one for each seller

# review: pending
print-seller-bills-empty = No seller has a bill for this day

# review: pending
print-sale-empty = Nothing was sold on this day

# What ادھاری تفصیل is for, one line (plan 78 A0, A5).
# review: pending
print-adhari-box-help = One row for each of today's buyers: the bill, today's credit, the balance before and the total credit. At the end, the total credit of all buyers.

## The prints of the dashboard pages: sellers, total dues, money lent, advances and cash sales (plan 79, PRD FR-19.10).

# review: pending
print-seller-dues-title = Sellers' balances

# review: pending
print-total-dues-title = { -kul-agrahi }

# review: pending
print-total-dues-col-section = Part

# review: pending
print-total-dues-col-count = People

# review: pending
print-total-dues-col-total = Amount

# review: pending
print-total-dues-buyers = Buyers' dues

# review: pending
print-total-dues-sellers = Sellers' dues

# review: pending
print-total-dues-dead = { -doobi-raqam }

# review: pending
print-total-dues-others = { -adhar-raqam }

# review: pending
print-lent-open-title = { -adhar-raqam } open

# review: pending
print-lent-open-col-type = Type

# review: pending
print-lent-open-col-overdue = Past its return date

# review: pending
print-cash-sales-month-title = Cash sales of the month

# review: pending
print-cash-sales-day-title = Cash sales of one day

# review: pending
print-cash-sales-col-buyers = Buyers

# review: pending
print-cash-sales-col-cash = Cash

# review: pending
print-cash-sales-col-bikri = { -bikri }

# review: pending
print-cash-sales-col-bill = Bill
