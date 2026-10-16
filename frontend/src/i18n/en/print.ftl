### Prints (plan 14, PRD FR-12.12, FR-7.36, FR-19.3 to FR-19.7): the seller bill, the buyer slips,
### the Pukhta Khata slip, the print box and the Printing settings. The backend sends the ids of
### titles, facts, roles and amounts (plan 14 contract 6) and the template resolves them in the
### PRINT language, which can differ from the app language.

## Titles

# review: pending
print-seller-bill-title = { -beopari } Bill

# review: pending
print-buyer-slip-title = { -gahak } Slip

# review: pending
print-pukhta-slip-title = { -pukhta-khata } Slip

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
print-party-onetime-buyer = { -pukhta-khata } ({ -kharidar })

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
print-cash-paid = { -naqd }

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

# review: pending
print-doc-pukhta-slip = { -pukhta-khata } slip

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

# review: pending
print-box-nothing-printed = There was nothing to print.

# review: pending
print-save-first = Save the { -bikri } first, then print it.

# Shown after Ctrl+Enter when the documents went to the printer.
# review: pending
print-toast-printed = Sent to the printer

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
print-settings-printer-none = No list of printers is available on this computer. The default printer will be used.

# review: pending
print-settings-printers-failed = The list of printers could not be read

# review: pending
print-settings-copies = Copies

# review: pending
print-settings-copies-hint = A number from 1 to 99.

# review: pending
print-settings-urdu-digits = Urdu digits on Urdu prints

# review: pending
print-settings-urdu-digits-hint = When off, Urdu prints also use English digits (0 to 9).

# review: pending
print-settings-after-save = Print after save

# $key is the key that saves and prints, for example Ctrl+Enter.
# review: pending
print-settings-after-save-hint = { $key } saves the { -bikri } and prints these documents.

# review: pending
print-settings-load-failed = Printing settings could not be opened
