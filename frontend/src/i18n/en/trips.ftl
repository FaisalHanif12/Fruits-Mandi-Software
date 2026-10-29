# The { -market-safar } screens: the list of trips, a new trip and one trip with its purchases,
# costs and sales in other shops (plan 16b, PRD 12.10, FR-5.2, FR-7.67, FR-12.24 to FR-12.33).
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The list of trips.

# review: pending
trips-title = { -market-safar }

# review: pending
trips-help = Some partners send a person to a bigger market to buy goods. The goods belong to those partners. The shop sells them and takes its normal { -commission }. Each trip is shown here with its goods and its result.

# review: pending
trips-new = New trip

# review: pending
trips-load-failed = The trip could not be opened

# review: pending
trips-list-label = List of trips

# review: pending
trips-list-from = From

# review: pending
trips-list-to = To

# review: pending
trips-list-status = Show

# review: pending
trips-list-search = Find a trip

# review: pending
trips-state-all = All

# review: pending
trips-state-open = Open

# review: pending
trips-state-closed = Closed

# review: pending
trips-list-failed = The list of trips could not be opened

# review: pending
trips-list-empty = No trip yet

# review: pending
trips-list-empty-help = Press the button to make the first trip. Choose the partners in it, then record what is bought.

# review: pending
trips-list-no-match = No trip matches these choices

# review: pending
trips-list-no-match-help = Try other days, open or closed trips, or other words.

# review: pending
trips-list-clear = Show all trips

# $shown: how many are on screen; $total: how many match.
# review: pending
trips-list-count = Showing { $shown } of { $total }

# review: pending
trips-list-more = Show more

## Numbers: a trip has a code of type 007 (plan 34); its entries have series with their own letters.

# A trip cost.
# review: pending
trips-cost-number = TC-{ $number }

# A sale in another shop.
# review: pending
trips-sale-number = OS-{ $number }

## The columns of the lists.

# review: pending
trips-col-number = Code

# review: pending
trips-col-trip = Trip

# review: pending
trips-col-date = Day

# review: pending
trips-col-market = Market

# review: pending
trips-col-status = State

# review: pending
trips-col-sales = Sales money

# review: pending
trips-col-entry = Entry

# review: pending
trips-col-amount = Amount

# review: pending
trips-col-state = Cancelled

# review: pending
trips-col-detail = Details

## A new trip: the shares first, then where the money comes from, then the details (PRD FR-12.29).

# review: pending
trips-new-title = New trip

# review: pending
trips-new-help = First choose the partners in this trip and the % of each. Then say where each partner's money comes from. Then fill in the details.

# review: pending
trips-new-save = Make the trip

# review: pending
trips-new-shares-title = Partners and shares

# The form starts as the last trip was.
# review: pending
trips-new-from-last = Filled in from the last trip. Check it and change what is different.

# The first trip starts from the firm's shares.
# review: pending
trips-new-firm-shares = Filled in from the firm's { -hissa }. Check it and change what is different.

# review: pending
trips-new-shares-help = The shares of the partners in the trip must total exactly 100%. A partner with no share is not in it. The shares cannot be changed after the first entry.

# review: pending
trips-new-no-partners = No partner is set up yet, so a trip cannot be made.

# review: pending
trips-new-no-partners-link = Open the partner setup

# review: pending
trips-new-details-title = Details

# $number: the number of the new trip.
# review: pending
trips-created = { $code } made

# review: pending
trips-field-date = Date

# review: pending
trips-field-market = Market

# review: pending
trips-field-market-hint = Where the goods are bought, as you want it written.

# review: pending
trips-field-sent = Who was sent

# review: pending
trips-field-note = Note

## One trip.

# $number: the number of the trip.
# review: pending
trips-detail-title = { $code }

# review: pending
trips-detail-failed = The trip could not be opened

# review: pending
trips-back = All trips

# review: pending
trips-status-open = Open

# review: pending
trips-status-closed = Closed

# review: pending
trips-none = Not given

# review: pending
trips-closed-on = Closed on

# review: pending
trips-figures-title = Goods and result

# review: pending
trips-partners-title = Partners in this trip

# review: pending
trips-shares-locked = The shares are fixed now, because the trip has an entry.

# A trip closes by itself and opens again if a sale or a write-off is reversed (PRD FR-12.33).
# review: pending
trips-closed-notice = This trip is closed, because everything bought was sold or written off. It takes no new purchase or cost. It opens again if a sale or a write-off of it is reversed.

# review: pending
trips-fact-bought = { -nag } bought

# review: pending
trips-fact-cost = Total cost

# review: pending
trips-loss-note = This trip is making a loss.

## What each partner has put in and got back (PRD FR-12.27).

# review: pending
trips-partner-results-title = Each partner's money in this trip

# review: pending
trips-partner-results-empty = No money has moved yet.

# review: pending
trips-partner-in-shop = Paid from the shop

# review: pending
trips-partner-in-pocket = Paid from his pocket

# review: pending
trips-partner-back = Money back

# His part of the result so far: money back less money in.
# review: pending
trips-partner-net = Result so far

# Read by screen readers; the number is red as well.
# review: pending
trips-partner-loss = Loss

## The entry form: the type list and what each type needs (PRD FR-7.67).

# review: pending
trips-type-label = Type of entry

# review: pending
trips-type-purchase = { -khareedari }

# A cost of the whole trip, for example the truck or the food.
# review: pending
trips-type-cost = Trip cost

# review: pending
trips-type-other-sale = Sale in another shop

# review: pending
trips-entry-new = New entry

# review: pending
trips-parts-title = Each partner's part

# review: pending
trips-sources-title = Where the money comes from

# review: pending
trips-sources-hint = Filled in from the trip. Change it for this entry if a partner pays another way.

# review: pending
trips-preview-failed = The numbers could not be worked out

# $number: the number of the purchase.
# review: pending
trips-saved-purchase = { -khareedari } { $number } saved

# $count: how many lines were saved; $first, $last: the first and the last number.
# review: pending
trips-saved-purchases = { $count } { -khareedari } entries saved: { $first } to { $last }

# $number: the number of the trip cost.
# review: pending
trips-saved-cost = Trip cost { $number } saved

# $number: the number of the sale.
# review: pending
trips-saved-sale = Sale in another shop { $number } saved

# review: pending
trips-saved-sale-closed = Sale in another shop { $number } saved. Everything is sold now, so the trip is closed.

## A trip cost (PRD FR-12.31, C24).

# review: pending
trips-cost-amount = Amount

# review: pending
trips-cost-amount-hint = The cost of the whole trip. You can type a sum, for example 3000+1000.

# review: pending
trips-cost-reason = Reason

# review: pending
trips-cost-reason-hint = For example the truck or the food.

# review: pending
trips-cost-numbers-title = How the cost is spread

# review: pending
trips-cost-numbers-empty = Fill in the form and the spread shows here.

# review: pending
trips-cost-numbers-incomplete = Enter the amount and the reason to see how the cost is spread.

# The cost is spread over the purchases that still have goods left, by the { -nag } bought.
# review: pending
trips-cost-spread-title = Part for each purchase

# $qty: the { -nag } bought in that purchase.
# review: pending
trips-cost-spread-qty = ({ $qty } { -nag })

## A sale in another shop (PRD FR-12.32, C21, C25).

# review: pending
trips-sale-purchase = Goods from

# review: pending
trips-sale-purchase-hint = Only purchases of this trip with goods left are listed. The goods set for { -dusri-dukan } are sold first. If you sell more, the rest comes from { -apni-dukan }, and the screen tells you how many.

# $number: the purchase; $other_left, $our_left: how many { -nag } are left for { -dusri-dukan } and for { -apni-dukan }.
# review: pending
trips-sale-purchase-option = { $number }: { $item }, { $packing } ({ $other_left } left for { -dusri-dukan }, { $our_left } for { -apni-dukan })

# review: pending
trips-sale-shop = Shop

# review: pending
trips-sale-qty = { -nag } sold

# review: pending
trips-sale-amount = Money received

# review: pending
trips-sale-amount-hint = The money comes into { -rokar }.

# review: pending
trips-sale-numbers-title = This sale

# review: pending
trips-sale-numbers-empty = Fill in the form and the result of the sale shows here.

# review: pending
trips-sale-numbers-incomplete = Choose the goods and enter the quantity and the money to see the result.

# review: pending
trips-sale-left-after = Left in the purchase after this

# review: pending
trips-sale-other-left-after = Left for { -dusri-dukan } after this

# review: pending
trips-sale-cost-left-after = Cost of what is left

# What each partner gets of the money that comes in (PRD C25).
# review: pending
trips-sale-credits-title = Each partner's part of the money

## The purchases of the trip.

# review: pending
trips-purchases-title = Purchases of this trip

# review: pending
trips-purchases-label = List of the purchases of this trip

# review: pending
trips-purchases-empty = No purchase yet

# $key: the key that saves.
# review: pending
trips-purchases-empty-help = Fill in the form above and press { $key }.

# review: pending
trips-purchases-select-help = Move to a purchase with ↑ ↓. Its figures, result and actions show under the list.

## The history of the trip (PRD FR-19.11).

# review: pending
trips-entries-title = History of this trip

# review: pending
trips-entries-label = History of this trip

# review: pending
trips-entries-empty = Nothing has been entered yet

# $key: the key that saves.
# review: pending
trips-entries-empty-help = The purchases, costs and sales of this trip are listed here. Fill in the form above and press { $key }.

# $shown: how many are on screen; $total: how many the trip has.
# review: pending
trips-entries-count = Showing { $shown } of { $total }

# review: pending
trips-entries-more = Show more

# review: pending
trips-entries-cancel = Cancel entry

# $kind: the type of entry; $number: its number in its own series.
# review: pending
trips-entry-label = { $kind } { $number }

# review: pending
trips-entry-cancelled = Cancelled

# $reason: why it was cancelled.
# review: pending
trips-entry-cancelled-reason = This entry is cancelled. Reason: { $reason }

# $shop: the shop as typed; $qty: { -nag } sold; $purchase: the purchase.
# review: pending
trips-entry-detail-sale = { $shop }: { $qty } { -nag } from { $purchase }

# $number: the purchase; $qty: the { -nag } it was bought in.
# review: pending
trips-entry-spread-line = Part for { -khareedari } { $number } ({ $qty } { -nag })

## Cancelling a trip cost or a sale in another shop (PRD FR-12.31, FR-12.32, FR-7.8).

# review: pending
trips-cancel-title = Cancel this entry?

# $entry: the entry and its number.
# review: pending
trips-cancel-text-cost = { $entry } will be cancelled. Each partner's part goes back as it was, and the part of the cost that the purchases carried is taken off.

# $entry: the entry and its number.
# review: pending
trips-cancel-text-sale = { $entry } will be cancelled. The money goes out of { -rokar } again, the goods go back into the purchase, and each partner's part is taken back.

# review: pending
trips-cancel-reason = Reason

# review: pending
trips-cancel-confirm = Cancel entry

# review: pending
trips-cancel-confirm-cash = Yes, cancel it

# $entry: the entry and its number.
# review: pending
trips-cancelled = { $entry } cancelled

# $entry: the entry and its number.
# review: pending
trips-cancelled-reopened = { $entry } cancelled. The trip is open again.
