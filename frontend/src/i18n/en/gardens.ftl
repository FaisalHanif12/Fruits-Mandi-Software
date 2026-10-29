# The { -baghat } screens: the list of gardens, a new garden and one garden with its entries,
# its running balance, its history and its closing (plan 17c, PRD 12.11, FR-5.2, FR-7.67,
# FR-12.29 to FR-12.38).
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The list of gardens.
# review: pending
gardens-title = { -baghat }
# review: pending
gardens-help = Some partners rent a garden for a season. They pay the rent ({ -thaika }) in { -qist }s and pay its running costs. Its fruit is sold in the shop with the normal { -commission }, or in another shop. The profit or loss of a garden goes only to the partners in it.
# review: pending
gardens-new = New garden
# review: pending
gardens-load-failed = The form of a new garden could not be opened
# review: pending
gardens-list-label = List of gardens
# review: pending
gardens-list-from = From
# review: pending
gardens-list-to = To
# review: pending
gardens-list-status = Show
# review: pending
gardens-list-search = Find a garden
# review: pending
gardens-state-all = All
# review: pending
gardens-state-open = Open
# review: pending
gardens-state-closed = Closed
# review: pending
gardens-list-failed = The list of gardens could not be opened
# review: pending
gardens-list-empty = No garden yet
# review: pending
gardens-list-empty-help = Press the button to make the first garden. Choose the partners in it, then record the rent and what is paid.
# review: pending
gardens-list-no-match = No garden matches these choices
# review: pending
gardens-list-no-match-help = Try other days, open or closed gardens, or other words.
# review: pending
gardens-list-clear = Show all gardens

## The columns of the lists.
# review: pending
gardens-col-number = Code
# review: pending
gardens-col-name = Name
# review: pending
gardens-col-date = Date
# review: pending
gardens-col-status = State
# review: pending
gardens-col-balance = Running balance
# review: pending
gardens-col-rent-due = Rent still due
# review: pending
gardens-col-rent = { -thaika }
# review: pending
gardens-col-paid = Paid
# review: pending
gardens-col-money-in = Money in
# review: pending
gardens-col-entry = Entry
# review: pending
gardens-col-effect = Amount
# review: pending
gardens-col-running-balance = Balance after
# review: pending
gardens-col-state = Cancelled
# review: pending
gardens-status-open = Open
# review: pending
gardens-status-closed = Closed
# $date: the closing day as DD-MM-YYYY.
# review: pending
gardens-status-closed-on = Closed on { $date }
# review: pending
gardens-entry-cancelled = Cancelled

## What a row of the history is.
# review: pending
gardens-kind-installment = { -qist }
# review: pending
gardens-kind-cost = Garden cost
# review: pending
gardens-kind-other-sale = Sale in another shop
# review: pending
gardens-kind-sale = Sale in our shop
# review: pending
gardens-kind-fruit = { -phal }
# review: pending
gardens-kind-close = Closing
# $kind: what the row is; $detail: the reason, the shop or the sale.
# review: pending
gardens-history-entry = { $kind }: { $detail }
# review: pending
gardens-history-detail-sale-elsewhere = { $shop }, { $qty } { -nag }
# review: pending
gardens-history-detail-bikri = { -bikri } { $number }

## A new garden: the shares first, then where the money comes from, then the details (PRD FR-12.29).
# review: pending
gardens-new-title = New garden
# review: pending
gardens-new-help = First choose which partners are in this garden and the % of each. Then say where each partner's money comes from. At the end fill in the details.
# review: pending
gardens-new-save = Make the garden
# review: pending
gardens-new-shares-title = Partners and shares
# review: pending
gardens-new-from-defaults = Filled in from the firm's shares or the last garden. Check it and change what is different.
# review: pending
gardens-new-shares-help = The shares of the partners in the garden must total exactly 100%. A partner with no share is not in it. The shares cannot be changed after the first entry.
# review: pending
gardens-new-sources-help = Each partner's part of a payment comes from the shop's cash or from his own pocket. A part from the shop's cash comes off his total and out of { -rokar }. A part from his own pocket is only recorded: nothing is posted.
# review: pending
gardens-new-no-partners = No partner has been made yet, so a garden cannot be made.
# review: pending
gardens-new-no-partners-link = Open the partner setup
# review: pending
gardens-new-details-title = Details
# review: pending
gardens-field-name-ur = Urdu name
# review: pending
gardens-field-name-en = English name
# review: pending
gardens-field-date = Date
# review: pending
gardens-field-item-hint = The main fruit, if there is one.
# review: pending
gardens-field-rent = Full { -thaika }
# review: pending
gardens-field-rent-hint = The full price of the garden for the season, in whole rupees.
# review: pending
gardens-field-season-end = End of the season
# review: pending
gardens-field-paid-now = First { -qist }
# review: pending
gardens-field-paid-now-hint = Leave it empty if nothing is paid today. It is saved as the first installment.
# review: pending
gardens-field-paid-now-reason = Why the first installment was paid
# review: pending
gardens-field-paid-now-reason-hint = Filled in for you. Change it if it is not right.
# The words put in the reason of the first installment when its amount is typed (PRD FR-12.35). Free text: saved as shown, in the language of the screen at that moment. Plain words on purpose, with no glossary reference, so the saved text has no invisible marks in it.
# review: pending
gardens-first-installment-reason = First installment
# review: pending
gardens-field-note = Note
# review: pending
gardens-first-parts-title = Each partner's part of the first installment
# review: pending
gardens-preview-failed = The numbers could not be worked out
# $code: the code of the garden, type 008.
# review: pending
gardens-created = { $code } has been made

## One garden.
# review: pending
gardens-back = Back to the gardens
# $number: the number of the garden.
# review: pending
gardens-detail-title = { $code }
# review: pending
gardens-detail-failed = The garden could not be opened
# review: pending
gardens-none = None
# review: pending
gardens-closed-on = Closed on
# review: pending
gardens-balance = Running balance (money in less everything paid)
# review: pending
gardens-rent-due = Rent still due
# review: pending
gardens-figures-title = The garden in figures
# review: pending
gardens-partners-title = Partners
# review: pending
gardens-shares-locked = This garden has an entry or a sale, so the shares are fixed now.
# review: pending
gardens-fact-rent = { -thaika }
# review: pending
gardens-fact-installments = Paid in { -qist }s
# review: pending
gardens-fact-costs = Paid as garden costs
# review: pending
gardens-fact-paid = Everything paid
# review: pending
gardens-fact-paid-shop = Paid from the shop's cash
# review: pending
gardens-fact-paid-pocket = Paid from pockets
# review: pending
gardens-fact-money-in = Money in (sales after { -commission })
# review: pending
gardens-fact-other-shop = Of it, from sales in other shops
# review: pending
gardens-fact-account = In the garden's account now
# $date: the closing day as DD-MM-YYYY.
# review: pending
gardens-closed-notice = This garden was closed on { $date }. It takes no new entry, no lot and no fruit. If its closing is undone, it opens again.

## Each partner's money in the garden (PRD FR-12.38, FR-17.10).
# review: pending
gardens-partner-results-title = Each partner's money
# review: pending
gardens-partner-results-empty = No partner yet.
# review: pending
gardens-partner-paid-shop = Paid from the shop
# review: pending
gardens-partner-paid-pocket = Paid from his pocket
# review: pending
gardens-partner-back = Money back
# review: pending
gardens-partner-net-open = Net so far
# review: pending
gardens-partner-net-closed = Net result
# review: pending
gardens-partner-loss = Loss

## The entry form and its type list (PRD FR-7.67, FR-12.35, FR-12.36).
# review: pending
gardens-type-label = Kind of entry
# review: pending
gardens-type-installment = { -qist }
# review: pending
gardens-type-cost = Garden cost
# review: pending
gardens-type-other-sale = Sale in another shop
# review: pending
gardens-entry-new = New entry
# review: pending
gardens-sources-title = Where the money comes from
# review: pending
gardens-sources-hint = Filled in from the garden. If a partner pays in another way on this entry, change it. A part from his own pocket posts nothing.
# review: pending
gardens-parts-title = Each partner's part
# review: pending
gardens-installment-amount = { -qist } amount
# review: pending
gardens-cost-amount = Cost amount
# review: pending
gardens-payment-amount-hint = Whole rupees. You can type a sum, for example 20000+5000.
# review: pending
gardens-payment-reason = Why
# review: pending
gardens-installment-reason-hint = For example: first { -qist } of the rent.
# review: pending
gardens-cost-reason-hint = For example: labour, spray, transport.
# review: pending
gardens-sale-shop = Shop
# review: pending
gardens-sale-qty = { -nag } sold
# review: pending
gardens-sale-qty-hint = A whole number. You can type a sum, for example 20+25.
# review: pending
gardens-sale-amount = Money received
# review: pending
gardens-sale-arrival = Goods from

# review: pending
gardens-sale-arrival-hint = Only goods that are left are listed. The goods set for { -dusri-dukan } are sold first. If you sell more, the rest comes from { -apni-dukan }, and the screen tells you how many.

# $number: the entry; $other_left, $our_left: how many { -nag } are left for { -dusri-dukan } and for { -apni-dukan }.
# review: pending
gardens-sale-arrival-option = { $number }: { $item }, { $packing } ({ $other_left } left for { -dusri-dukan }, { $our_left } for { -apni-dukan })

# review: pending
gardens-sale-amount-hint = The money comes into { -rokar } and goes to the garden's account until the garden is closed.
# review: pending
gardens-saved-installment = { -qist } { $number } has been saved
# review: pending
gardens-saved-cost = Garden cost { $number } has been saved
# $number: the number of the entry in its own series.
# review: pending
gardens-saved-other-sale = Sale in another shop { $number } has been saved

## What the entry does, live under the form (PRD FR-12.37).
# review: pending
gardens-numbers-title = This entry
# review: pending
gardens-numbers-empty = Fill in the form and the figures show here.
# review: pending
gardens-numbers-incomplete = The figures show when the form is complete.
# review: pending
gardens-numbers-amount = Amount
# review: pending
gardens-numbers-balance-before = Running balance before
# review: pending
gardens-numbers-balance-after = Running balance after
# review: pending
gardens-numbers-rent-due-after = Rent still due after

## The history of the garden (PRD FR-12.37, FR-19.11).
# review: pending
gardens-history-title = History
# review: pending
gardens-history-label = History of the garden
# review: pending
gardens-history-empty = Nothing has been recorded yet
# $key: the key that saves.
# review: pending
gardens-history-empty-help = The { -qist }, costs and sales of this garden show here. Fill in the form above and press { $key }.
# review: pending
gardens-history-count = Showing { $shown } of { $total }
# review: pending
gardens-history-more = Show more
# review: pending
gardens-history-cancel = Cancel the entry
# review: pending
gardens-entry-cancelled-reason = This entry was cancelled. Reason: { $reason }

## Cancelling an entry (PRD FR-7.8).
# review: pending
gardens-cancel-title = Cancel this entry?
# $entry: what the entry is, for example Sale in another shop.
# review: pending
gardens-cancel-text = { $entry } will be cancelled. Its posting is reversed, each partner's part goes back as it was, and the entry stays in the history, marked cancelled.
# review: pending
gardens-cancel-reason = Reason
# review: pending
gardens-cancel-confirm = Cancel the entry
# review: pending
gardens-cancel-confirm-cash = Yes, cancel
# review: pending
gardens-cancelled = { $entry } has been cancelled.

## Closing the garden (PRD FR-12.38, FR-7.8, E64).
# review: pending
gardens-close-title = Close the garden
# review: pending
gardens-close-help = When the goods stop coming, close the garden. Its money goes to the partners by their shares: each gets back what he paid, plus or minus his share of the profit or loss. After that the garden takes no more entries.
# review: pending
gardens-close-date = Closing day
# $date: the day of the latest entry as DD-MM-YYYY.
# review: pending
gardens-close-earliest = Not before { $date }, the last entry of the garden.
# review: pending
gardens-close-numbers-empty = Choose the closing day and the figures show here.
# review: pending
gardens-close-money-in = Money in
# review: pending
gardens-close-total-cost = Total cost
# review: pending
gardens-close-balance = Goes to the partners
# review: pending
gardens-close-loss-note = The garden made a loss. Each partner's part of the loss is in his net result below.
# review: pending
gardens-close-credits-title = Each partner's money back
# review: pending
gardens-close-nothing-posted = Nothing is in the garden's account, so nothing is posted. The garden still closes.
# review: pending
gardens-close-button = Close the garden
# review: pending
gardens-close-confirm-title = Close this garden?
# $name: the garden name; $date: the closing day; $balance: the money that goes to the partners.
# review: pending
gardens-close-confirm-text = The garden { $name } will be closed on { $date }. { $balance } goes to its partners by their shares. After this it takes no more entries, no lot and no fruit. The closing can be undone while its day is open.
# review: pending
gardens-close-confirm = Yes, close it
# review: pending
gardens-closed-toast = { $code } has been closed

## Undoing the closing (PRD FR-12.38).
# review: pending
gardens-reopen-button = Undo the closing
# review: pending
gardens-reopen-title = Undo the closing of this garden?
# review: pending
gardens-reopen-text = The closing is reversed: the money goes back into the garden's account and out of the partners' totals, and the garden opens again. The closing stays in the history.
# review: pending
gardens-reopen-reason = Reason
# review: pending
gardens-reopen-confirm = Undo the closing
# review: pending
gardens-reopened-toast = { $code } is open again

# review: pending
gardens-filter-all = All gardens

# review: pending
gardens-filter-failed = The list of gardens could not be read

## The number of a goods arrived entry: its own series (plan 33 version 1.1, PRD FR-12.40).

# $number: the number of the entry in its series.
# review: pending
gardens-arrival-number = GA-{ $number }

## Goods arrived (مال آیا): a garden's stock (plan 33 version 1.1, PRD FR-12.40). They carry no money.

# review: pending
gardens-type-arrival = { -maal-aaya }

# $number: the number of the entry.
# review: pending
gardens-saved-arrival = { -maal-aaya } { $number } has been saved

# $count: how many lines were saved; $first, $last: the first and the last number.
# review: pending
gardens-saved-arrivals = { $count } { -maal-aaya } entries saved: { $first } to { $last }

# review: pending
gardens-arrival-qty = { -nag } arrived

# review: pending
gardens-arrival-qty-hint = A whole number. You can type a sum, for example 20+25.

# review: pending
gardens-arrival-sell-date-hint = Starts as the day the goods arrived. From this day they come into every new { -bikri } until they are sold.

# review: pending
gardens-arrival-posts-nothing = These goods carry no money. They stay in the stock of the { -bagh } until they are sold in a { -bikri } or written off. A new { -bikri } holds a lot for them from the { -sell-date }.

# review: pending
gardens-arrival-title = { -maal-aaya }

# review: pending
gardens-arrival-help = The goods that came from this { -bagh } and are not sold yet. A new { -bikri } holds a lot for each one on its { -sell-date }. What is not sold comes again the next day.

# review: pending
gardens-arrival-failed = The goods arrived could not be read

# review: pending
gardens-arrival-empty = No goods have arrived yet

# review: pending
gardens-arrival-empty-help = Choose { -maal-aaya } in the kind of entry above, fill in the item, the packing and the { -nag }, and save.

# review: pending
gardens-arrival-label = List of the goods arrived

# review: pending
gardens-arrival-col-number = No.

# review: pending
gardens-arrival-col-item = { -qism-maal }

# review: pending
gardens-arrival-col-left = Left

# review: pending
gardens-arrival-col-date = Day

# review: pending
gardens-arrival-col-qty = Arrived

# review: pending
gardens-arrival-item-packing = { $item }, { $packing }

# review: pending
gardens-arrival-item-cancelled = { $item } (cancelled)

# $number: the number of the entry; $item: its item.
# review: pending
gardens-arrival-card-title = { -maal-aaya }: { $number }, { $item }

# $reason: why it was cancelled.
# review: pending
gardens-arrival-cancelled = This entry is cancelled. { $reason }

# The lots of the sales that took goods from the entry.
# review: pending
gardens-arrival-lots = Sales of these goods

# review: pending
gardens-arrival-lot-bikri = { -bikri }

# review: pending
gardens-arrival-cancel = Cancel { -maal-aaya }

# $number: the entry.
# review: pending
gardens-arrival-sell-date-title = Change the { -sell-date }

# $number: the entry.
# review: pending
gardens-arrival-sell-date-text = From this day the goods of { $number } come into every new { -bikri } until they are sold. The day cannot be before the day the goods arrived.

# review: pending
gardens-arrival-reason = Reason

# review: pending
gardens-arrival-cancel-title = Cancel this entry?

# $number: the entry.
# review: pending
gardens-arrival-cancel-text = { $number } will be cancelled and the goods are taken off the stock. It can be cancelled only while nothing of it is sold or written off. Nothing is posted.

# review: pending
gardens-arrival-cancel-confirm = Cancel { -maal-aaya }

# $number: the entry.
# review: pending
gardens-arrival-cancelled-toast = { $number } has been cancelled

# review: pending
gardens-arrival-write-off-title = Write off goods

# $number: the entry; $item, $packing: what it is; $qty_left: how many are left.
# review: pending
gardens-arrival-write-off-text = Goods of { $number } ({ $item }, { $packing }) that are spoiled or lost are taken off the stock. { $qty_left } { -nag } are left now. Nothing is posted.

# review: pending
gardens-arrival-write-off-qty = { -nag } to write off

# review: pending
gardens-arrival-write-off-confirm = Write off

# $qty: how many were written off.
# review: pending
gardens-arrival-write-off-done = { $qty } { -nag } have been written off

# review: pending
gardens-arrival-write-off-undo-title = Undo a write-off

# review: pending
gardens-arrival-write-off-undo-text = The goods go back to the stock. Choose the write-off and give the reason.

# review: pending
gardens-arrival-write-off-undo-confirm = Undo the write-off

# review: pending
gardens-arrival-write-off-pick = Write-off

# $qty: how many were written off; $reason: why.
# review: pending
gardens-arrival-write-off-option = { $qty } { -nag }: { $reason }

# review: pending
gardens-arrival-write-off-undone = The write-off was undone

# review: pending
gardens-numbers-left-after = Left in the entry after this
