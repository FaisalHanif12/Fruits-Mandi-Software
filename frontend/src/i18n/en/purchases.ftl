# The purchases of a trip (plan 16b, PRD 12.10, FR-12.24 to FR-12.28). They are made on the trip
# screen; these are the words of their fields, cards and boxes.
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The number of a purchase: its own series.

# review: pending
purchases-number = PUR-{ $number }

# $number: the number of the purchase in its series.
# review: pending
purchases-cancelled = { -khareedari } { $number } cancelled

# Who the goods were bought from, when no supplier is named.
# review: pending
purchases-market = Bought in the market

## The fields.

# review: pending
purchases-field-date = Date

# review: pending
purchases-field-supplier = Bought from

# review: pending
purchases-supplier-hint = As you want it written. Nothing is owed to him: a purchase is paid at once.

# review: pending
purchases-field-qty = { -nag } bought

# review: pending
purchases-qty-hint = You can type a sum, for example 20+25.

# review: pending
purchases-field-weight = Weight (kg)

# review: pending
purchases-field-rate = Rate

# review: pending
purchases-rate-hint = For each { -nag }, or for each kg for an item sold by weight.

# review: pending
purchases-field-other-cost = Other cost

# PRD FR-12.39. The box shows the day of the purchase until it is changed.
# review: pending
purchases-sell-date-hint = Starts as the day of the purchase. From this day the goods come into every new { -bikri } until they are sold.

# review: pending
purchases-field-note = Note

## What the purchase costs, live under the form.

# review: pending
purchases-numbers-title = Cost

# review: pending
purchases-numbers-empty = Fill in the form and the cost shows here.

# review: pending
purchases-numbers-incomplete = Enter the { -qism-maal }, the { -packing }, the quantity and the rate to see the cost.

# review: pending
purchases-line-value = Value of the goods

# review: pending
purchases-line-total-cost = Total cost

# The cost of one { -nag }, worked out for the whole purchase (PRD A45).
# review: pending
purchases-line-avg-cost = Cost of one { -nag }

# Paid at once: the value, the freight, the labour and the other cost.
# review: pending
purchases-line-paid-now = Paid now

## The columns of the list of a trip's purchases.

# review: pending
purchases-col-number = No.

# review: pending
purchases-col-supplier = Bought from

# review: pending
purchases-col-item = { -qism-maal } and { -packing }

# review: pending
purchases-col-left = Left

# review: pending
purchases-col-cost-left = Cost left

# review: pending
purchases-col-date = Day
# review: pending
purchases-col-sell-date = { -sell-date }

# review: pending
purchases-col-qty = Bought

# review: pending
purchases-col-total-cost = Total cost

# review: pending
purchases-col-result = Profit or loss

# review: pending
purchases-item-packing = { $item }, { $packing }

# A cancelled purchase is told in words, not only dimmed (NFR-USE-12).
# review: pending
purchases-item-cancelled = { $item } (cancelled)

## The card of the purchase that has focus.

# review: pending
purchases-detail-title = { $number }, { $item }

# review: pending
purchases-detail-failed = The purchase could not be opened

# review: pending
purchases-detail-cancelled = This purchase is cancelled.

# review: pending
purchases-detail-cancel-reason = Reason: { $reason }

# review: pending
purchases-detail-written-off = Written off

# review: pending
purchases-detail-days = Days waiting

# The part of the trip costs (the truck, the food) that this purchase carries (PRD C24).
# review: pending
purchases-detail-trip-cost = Part of the trip costs

# review: pending
purchases-detail-parts-title = What each partner paid

# PRD FR-12.27: sales money + other shops − cost sold − cost written off.
# review: pending
purchases-result-title = Profit or loss of this purchase

# review: pending
purchases-detail-sales = Sales money in our shop

# review: pending
purchases-detail-other-shop = Money from other shops

# review: pending
purchases-detail-cost-sold = Cost of goods sold

# review: pending
purchases-detail-cost-written-off = Cost written off

# review: pending
purchases-loss-note = This is making a loss.

# review: pending
purchases-lots-title = Sold in these sales

# review: pending
purchases-lot-bikri = Sale

# $number: the number of the { -bikri }; $lot: the number of the lot.
# review: pending
purchases-lot-open = { -bikri } { $number }, { -lot } { $lot }

# review: pending
purchases-lot-qty-sold = Sold

# The seller credit of the lot, which goes to the trip's partners (PRD C25).
# review: pending
purchases-lot-sales = Sales money

# review: pending
purchases-write-offs-title = Written off

# review: pending
purchases-write-off = Write off

# review: pending
purchases-undo-write-off = Undo a write-off

# review: pending
purchases-cancel = Cancel purchase

## Writing trip goods off (PRD FR-12.28). Nothing is posted.

# review: pending
purchases-write-off-title = Write off trip goods

# $number: the purchase; $qty_left: how many { -nag } are left.
# review: pending
purchases-write-off-text = Goods of { $number } ({ $item }, { $packing }) that are spoiled or lost are taken off the stock. { $qty_left } { -nag } are left now. Nothing is posted, because the partners paid for the goods already. The cost of the goods written off shows as a loss in the result of the trip.

# review: pending
purchases-write-off-qty = { -nag } to write off

# review: pending
purchases-write-off-reason = Reason (for example spoiled)

# review: pending
purchases-write-off-confirm = Write off

# $qty: how many { -nag }; $cost: the cost that shows as a loss.
# review: pending
purchases-write-off-done = { $qty } { -nag } written off. Loss in the result: <num>{ $cost }</num>

# review: pending
purchases-write-off-done-closed = { $qty } { -nag } written off. Loss in the result: <num>{ $cost }</num>. Nothing is left, so the trip is closed.

# review: pending
purchases-write-off-undo-title = Undo a write-off

# review: pending
purchases-write-off-undo-text = The goods go back to the stock and the loss is taken back. Choose the write-off and give the reason.

# review: pending
purchases-write-off-undo-which = Write-off

# review: pending
purchases-write-off-undo-option = { $date }: { $qty } { -nag } ({ $reason })

# review: pending
purchases-write-off-undo-confirm = Undo the write-off

# review: pending
purchases-write-off-undone = The write-off was undone

# review: pending
purchases-write-off-undone-reopened = The write-off was undone. The trip is open again.

## Cancelling a purchase (PRD FR-12.24).

# review: pending
purchases-cancel-title = Cancel this purchase?

# $cost: what was paid now.
# review: pending
purchases-cancel-text = { $number } ({ $qty } { -nag } { $item }, { $packing }, paid <num>{ $cost }</num>, from { $supplier }) will be cancelled. What the partners paid from the shop's cash goes back, and the stock is taken off.

# review: pending
purchases-cancel-reason = Reason

# review: pending
purchases-cancel-confirm = Cancel purchase

# review: pending
purchases-cancel-confirm-cash = Yes, cancel it

# A purchase cannot be cancelled once any of it was sold or written off (PRD FR-12.24).
# review: pending
purchases-cancel-blocked = This purchase cannot be cancelled, because { $sold } { -nag } were sold and { $written_off } { -nag } were written off. Cancel the sales or undo the write-off first.

# A purchase that carries a part of a trip cost cannot be cancelled either (PRD FR-12.31).
# review: pending
purchases-cancel-blocked-cost = This purchase carries a part of a trip cost, so it cannot be cancelled. Cancel that cost first.

## Changing the date to sell (PRD FR-12.39).

# review: pending
purchases-change-sell-date = Change the { -sell-date }

# review: pending
purchases-sell-date-title = Change the { -sell-date }

# $number: the purchase.
# review: pending
purchases-sell-date-text = From this day the goods of { $number } come into every new { -bikri } until they are sold. The day cannot be before the day of the purchase.

# review: pending
purchases-sell-date-done = The { -sell-date } was changed.
