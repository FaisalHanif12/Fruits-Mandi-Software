# The vouchers (plans 10 and 15, PRD 15.1 to 15.4): ONE screen with a type list for Receipt, Payment
# and Advance (FR-7.67), the panels under the form, the list, the cancel box and the collection list.
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The three types and their screens.

# review: pending
vouchers-field-type = Type

# review: pending
vouchers-type-receipt = { -wasooli }

# review: pending
vouchers-type-payment = { -adaigi }

# review: pending
vouchers-type-advance = { -peshgi-khata }

# review: pending
vouchers-receipt-title = { -wasooli }

# review: pending
vouchers-receipt-help = Take money from a { -gahak }. It settles his oldest bills first. If a { -beopari } is giving back his { -peshgi }, choose { -beopari } in "Receipt from".

# review: pending
vouchers-payment-title = { -adaigi }

# review: pending
vouchers-payment-help = Pay a { -beopari } or another account. Money paid to a { -gahak } is { -naqd-udhar }, and a reason is needed.

# review: pending
vouchers-advance-title = { -peshgi-khata }

# review: pending
vouchers-advance-help = Give a { -beopari } money before his goods are sold. After you save, his new balance shows.

# review: pending
vouchers-load-failed = The voucher screen could not be opened

# review: pending
vouchers-new = New entry

# review: pending
vouchers-print = Print

# review: pending
vouchers-edit = Change

# review: pending
vouchers-cancel = Cancel voucher

# review: pending
vouchers-collection-open = { -agrahi } list

## The number of a voucher: its series letter and its number.

# review: pending
vouchers-number-receipt = R-{ $number }

# review: pending
vouchers-number-payment = P-{ $number }

# review: pending
vouchers-number-advance = A-{ $number }

## After a save, an edit and a cancel. $kind is receipt, payment or advance; $number is the number
## in its series.

# review: pending
vouchers-saved = { $kind ->
    [receipt] { -wasooli } { $number } saved
    [payment] { -adaigi } { $number } saved
   *[advance] { -peshgi-khata } { $number } saved
}

# review: pending
vouchers-edited = { $kind ->
    [receipt] Changes to { -wasooli } { $number } saved
    [payment] Changes to { -adaigi } { $number } saved
   *[advance] Changes to { -peshgi-khata } { $number } saved
}

# review: pending
vouchers-cancelled = { $kind ->
    [receipt] { -wasooli } { $number } cancelled
    [payment] { -adaigi } { $number } cancelled
   *[advance] { -peshgi-khata } { $number } cancelled
}

# review: pending
vouchers-editing-note = { $kind ->
    [receipt] You are changing { -wasooli } { $number }.
    [payment] You are changing { -adaigi } { $number }.
   *[advance] You are changing { -peshgi-khata } { $number }.
} When you save, the old entry is reversed and stays in the history.

## The fields of a voucher.

# review: pending
vouchers-field-date = Date

# review: pending
vouchers-field-from = Receipt from

# review: pending
vouchers-from-buyer = { -gahak }

# review: pending
vouchers-from-seller = { -beopari }

# review: pending
vouchers-field-buyer = { -gahak }

# review: pending
vouchers-field-seller = { -beopari }

# review: pending
vouchers-field-paid-to = Pay to

# review: pending
vouchers-field-person = Name or phone

# review: pending
vouchers-person-hint = Type the name or phone of a person already recorded under { -pukhta-khata } and pick him from the list.

# review: pending
vouchers-make-regular = Make a regular account

# review: pending
vouchers-field-amount = Amount

# review: pending
vouchers-fill-full = Full balance

# review: pending
vouchers-fill-oldest = Oldest bill

# review: pending
vouchers-field-collector = Collector

# review: pending
vouchers-field-reason-lent = Reason for the { -naqd-udhar }

# review: pending
vouchers-reason-lent-hint = This amount becomes one of his bills and is collected with his other dues ({ -agrahi }).

# review: pending
vouchers-field-expected = Expected goods or season

# review: pending
vouchers-field-expected-hint = For example, the mango season.

# review: pending
vouchers-field-narration = Narration

# review: pending
vouchers-field-edit-reason = Reason for the change

# review: pending
vouchers-edit-reason-hint = Optional. The reason is kept in the record of the change.

## What the backend says about the entry on the form (PRD FR-6.7, FR-7.68, FR-15.5, FR-15.12).

# review: pending
vouchers-current-balance = Current balance

# review: pending
vouchers-new-balance = New balance

# review: pending
vouchers-return-limit = Most he can return

# review: pending
vouchers-extra-credit-note = This is more than all his bills. The extra stays as his credit and settles his next bill:

# review: pending
vouchers-owes-settled-note = This much settles what he owes the firm:

# review: pending
vouchers-advance-hint = Nothing is payable to this { -beopari }. Is this an { -peshgi }?

# review: pending
vouchers-make-advance = Make it an { -peshgi }

## The open bills of a buyer, under the form (PRD FR-7.58).

# review: pending
vouchers-bills-title = Open bills

# review: pending
vouchers-bills-failed = The open bills could not be opened

# review: pending
vouchers-bills-none = This { -gahak } has no open bill.

# review: pending
vouchers-bills-open = Total open

# review: pending
vouchers-bills-oldest = Oldest bill

# review: pending
vouchers-bills-credit = Credit apart from the bills

# review: pending
vouchers-days = { $days ->
    [1] 1 day
   *[other] { $days } days
}

# review: pending
vouchers-effect-full = Full

# review: pending
vouchers-effect-part = Part

## Where a bill or a payable posting came from. $number is the number of the record, when it has one.

# review: pending
vouchers-source-bikri = { -bikri } { $number }

# review: pending
vouchers-source-opening = Opening balance

# review: pending
vouchers-source-cash-lent = { -naqd-udhar }

# review: pending
vouchers-source-receipt = { -wasooli } { $number }

# review: pending
vouchers-source-general = { -general } { $number }

# review: pending
vouchers-source-purchase = Purchase { $number }

# A fruit credit on a seller (PRD FR-16.11, FR-7.59): $number is the پھل number.
# review: pending
vouchers-source-fruit = { -phal } { $number }

# review: pending
vouchers-source-other = Other entry

## What is payable to a seller, under the form (PRD FR-7.59).

# review: pending
vouchers-payables-title = Payable

# review: pending
vouchers-payables-failed = The payable amount could not be opened

# review: pending
vouchers-payables-none = Nothing is payable to this { -beopari }.

# review: pending
vouchers-payables-open = Total payable

# review: pending
vouchers-payables-owes = Owed by the { -beopari }

## The advances of a seller, under his party card (PRD FR-7.60).

# review: pending
vouchers-advances-title = { -peshgi-khata } of this { -beopari }

# review: pending
vouchers-advances-none = This { -beopari } has no { -peshgi-khata } yet.

# review: pending
vouchers-advances-open = Open { -peshgi-khata }

# review: pending
vouchers-advances-last = Last { -peshgi-khata }

# review: pending
vouchers-advances-failed = The list of { -peshgi-khata } could not be opened

## The columns of the panels and the list.

# review: pending
vouchers-col-date = Date

# review: pending
vouchers-col-number = Number

# review: pending
vouchers-col-source = Source

# review: pending
vouchers-col-account = Account

# review: pending
vouchers-col-amount = Amount

# review: pending
vouchers-col-bill = Bill

# review: pending
vouchers-col-credit = Credit

# review: pending
vouchers-col-paid = Paid

# review: pending
vouchers-col-adjusted = Adjusted

# review: pending
vouchers-col-open = Open

# review: pending
vouchers-col-days = Days

# review: pending
vouchers-col-settles = Settled by this

# review: pending
vouchers-col-returns = Adjusted by this

# review: pending
vouchers-col-state = State

# review: pending
vouchers-state-saved = Saved

# review: pending
vouchers-state-cancelled = Cancelled

# A voucher found by the global search that was cancelled.
# review: pending
vouchers-hit-cancelled = cancelled

## The list under the form.

# review: pending
vouchers-list-title-receipt = List of receipts

# review: pending
vouchers-list-title-payment = List of payments

# review: pending
vouchers-list-title-advance = List of advances

# review: pending
vouchers-list-label = List of vouchers

# review: pending
vouchers-list-from = From

# review: pending
vouchers-list-to = To

# review: pending
vouchers-list-account = Account

# review: pending
vouchers-list-failed = The list could not be opened

# review: pending
vouchers-list-empty-receipt = No receipts today

# review: pending
vouchers-list-empty-payment = No payments today

# review: pending
vouchers-list-empty-advance = No advances today

# $key is the key that saves.
# review: pending
vouchers-list-empty-help = Fill in the form above and press { $key }.

# review: pending
vouchers-list-no-match = No voucher matches these dates and this account

# review: pending
vouchers-list-no-match-help = Try other dates or another account.

# review: pending
vouchers-list-clear = Show today's list

# review: pending
vouchers-list-select-help = Move to a voucher with ↑ ↓ and press Enter to change it.

## A saved voucher that cannot be changed (PRD FR-15.3, FR-6.6).

# review: pending
vouchers-view-cancelled = This voucher is cancelled. Reason:

# review: pending
vouchers-view-closed = The day of this voucher is closed, so it cannot be changed. If the Owner reopens the day, it can be.

# review: pending
vouchers-other-screen = This voucher belongs to another screen

# review: pending
vouchers-other-screen-help = Open it in the { -rokar } area.

# review: pending
vouchers-other-screen-open = Open { -rokar }

## Cancelling a voucher (PRD FR-7.8, FR-15.3). $number is its number, $amount its amount, $name the party.

# review: pending
vouchers-cancel-title = Cancel this voucher?

# review: pending
vouchers-cancel-text-receipt = { -wasooli } { $number } (<num>{ $amount }</num>, { $name }) will be cancelled. { -rokar } and his account go back as they were, and the bills it settled are open again.

# review: pending
vouchers-cancel-text-payment = { -adaigi } { $number } (<num>{ $amount }</num>, { $name }) will be cancelled. { -rokar } and the account go back as they were. If it was { -naqd-udhar } to a { -gahak }, that bill is removed too.

# review: pending
vouchers-cancel-text-advance = { -peshgi-khata } { $number } (<num>{ $amount }</num>, { $name }) will be cancelled. { -rokar } and his account go back as they were, and what was adjusted against this { -peshgi } becomes payable to the { -beopari } again.

# review: pending
vouchers-cancel-reason = Reason

# review: pending
vouchers-cancel-confirm = Cancel voucher

# review: pending
vouchers-cancel-confirm-cash = Yes, cancel it

## The print box of a voucher (PRD FR-15.2).

# $number is the number in its series.
# review: pending
vouchers-print-title = Print voucher { $number }

## The collection list (PRD FR-15.6, FR-7.61).

# review: pending
vouchers-collection-title = { -agrahi } list

# review: pending
vouchers-collection-help = The buyers to collect from on this day. Type only the amounts received and save them all together. A row left empty or at 0 is skipped.

# review: pending
vouchers-collection-back = Back to { -wasooli }

# $count is the number of buyers on the list.
# review: pending
vouchers-collection-count = { $count ->
    [1] 1 buyer is on the list.
   *[other] { $count } buyers are on the list.
}

# review: pending
vouchers-collection-total = Total dues

# review: pending
vouchers-collection-col-group = Collector and city

# review: pending
vouchers-collection-col-dues = Dues

# review: pending
vouchers-collection-col-received = Received

# review: pending
vouchers-collection-group-separator = {", "}

# review: pending
vouchers-collection-save = Save all

# $count is the number of receipts saved; $refused the number that were not saved.
# review: pending
vouchers-collection-saved = { $count ->
    [1] 1 receipt saved
   *[other] { $count } receipts saved
}

# review: pending
vouchers-collection-some-refused = { $count ->
    [1] 1 receipt saved
   *[other] { $count } receipts saved
}, { $refused } could not be saved. See the reason on the row.

# review: pending
vouchers-collection-problems = These receipts were not saved

# review: pending
vouchers-collection-empty = No collection is planned for this day

# review: pending
vouchers-collection-empty-help = A buyer comes on the list on the collection day set in his account. Try another date, collector or city.

# review: pending
vouchers-collection-failed = The collection list could not be opened

# review: pending
vouchers-collection-grid-label = Collection list

# $key is the key that saves and prints, for example Ctrl+Enter.
# review: pending
vouchers-print-help = The slip you tick here is also the one { $key } prints when it saves a voucher.

# This row is not touched by the amount typed (a table cell, read by a screen reader).
# review: pending
vouchers-effect-none = Nothing

# F9 pressed before anything was saved: the slip is made from a saved voucher.
# review: pending
vouchers-print-save-first = Save the voucher first, then print it.

## The waiting list: several vouchers entered one after another and saved together (PRD FR-15.1).

# review: pending
vouchers-add-to-list = Add to list

# review: pending
vouchers-waiting-title = Waiting to be saved

# review: pending
vouchers-waiting-help = { $key } saves all of them, one by one. A voucher that cannot be saved stays here with the reason.

# review: pending
vouchers-waiting-count = { $count ->
    [one] One voucher waiting
   *[other] { $count } vouchers waiting
}

# review: pending
vouchers-inactive-hint = Make this account active first ({ -index }, then the account), then enter the voucher again.

# review: pending
vouchers-truncated-note = The oldest { $shown } of { $total } are shown.

# review: pending
vouchers-col-type = Type

# review: pending
vouchers-state-waiting = Waiting

# review: pending
vouchers-state-declined = Not saved: you went back at the question about { -rokar }

# review: pending
vouchers-remove = Remove from the list

# review: pending
vouchers-saved-many = { $count } vouchers saved

# review: pending
vouchers-some-refused = { $count ->
    [one] One voucher could not be saved. See the reason in the list.
   *[other] { $count } vouchers could not be saved. See the reason in the list.
}

## The number of a voucher of the روکڑ area (plan 17): its series letters and its number.

# review: pending
vouchers-number-shop-expense = E-{ $number }

# review: pending
vouchers-number-personal-expense = D-{ $number }

# review: pending
vouchers-number-personal-return = DR-{ $number }

# review: pending
vouchers-number-money-lent = L-{ $number }

# review: pending
vouchers-number-money-lent-return = LR-{ $number }

# review: pending
vouchers-number-capital-in = CI-{ $number }

# review: pending
vouchers-number-capital-out = CO-{ $number }

# review: pending
vouchers-number-khata-out = KO-{ $number }

# review: pending
vouchers-number-khata-in = KI-{ $number }

# review: pending
vouchers-number-general = G-{ $number }

## Printing the collection list (plan 17e, PRD FR-15.6).

# The title of the print box of the collection list.
# review: pending
vouchers-collection-print-title = Print the { -agrahi } list

# $date is the day of the list, for example 10-10-2026.
# review: pending
vouchers-collection-print-help = Prints the { -agrahi } list of { $date }, as it is on the screen. Choose the language and where it goes.
