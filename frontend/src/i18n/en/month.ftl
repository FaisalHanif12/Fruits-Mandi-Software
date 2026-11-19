## The month close (round 7, PRD FR-17.6, FR-21.2).

# review: pending
month-title = Month Close

# review: pending
month-help = At the end of a month, its income and its { -dukan-kharcha } move into the partners' capital accounts, each partner by his { -hissa }. The { -kul-raqam } of every partner stays the same.

# review: pending
month-open = Month Close

# review: pending
month-load-failed = The months could not be opened

# review: pending
month-empty = There is no month yet

# review: pending
month-empty-help = A month appears here after the first entry is saved.

## The list of months.

# review: pending
month-list-label = Months

# review: pending
month-col-month = Month

# review: pending
month-col-state = State

# review: pending
month-col-closed-on = Closed on

# review: pending
month-col-reversed = Reversed

# review: pending
month-state-closed = Closed

# review: pending
month-state-ready = Ready to close

# review: pending
month-state-days-open = Days still open

# review: pending
month-state-not-finished = Not finished

## A month that is not closed: what closing it posts.

# review: pending
month-preview-title = Closing { $month }

# review: pending
month-preview-help = Closing posts one entry that moves this income and these expenses into the partners' capital accounts. Nothing is posted until you press Close.

# review: pending
month-fact-income = Income

# review: pending
month-fact-expense = { -dukan-kharcha }

# review: pending
month-fact-profit = Profit shared between the partners

# review: pending
month-fact-date = The entry is dated

# review: pending
month-lines-label = The lines of the closing entry

# review: pending
month-col-account = Account

# review: pending
month-col-kind = What it is

# review: pending
month-col-debit = { -banaam }

# review: pending
month-col-credit = { -jama }

# review: pending
month-total = { -meezan }

# review: pending
month-kind-income = Income account

# review: pending
month-kind-expense = Shop expense account

# review: pending
month-kind-partner-capital = Partner capital

# review: pending
month-close-button = Close the month

# review: pending
month-close-confirm-title = Close { $month }?

# review: pending
month-close-confirm-description = The income and the { -dukan-kharcha } of { $month } will be moved into the partners' capital accounts by one entry dated { $date }. No { -kul-raqam } changes. You can reverse the close later with a reason.

# review: pending
month-closed-toast = { $month } is closed

## A month that is closed.

# review: pending
month-closed-title = { $month } is closed

# review: pending
month-closed-help = Its income and { -dukan-kharcha } are already in the partners' capital accounts. To close it again, reverse this close first.

# review: pending
month-closed-on = Closed on

# review: pending
month-reversed-before = { $count ->
    [1] A close of this month was reversed once before
   *[other] A close of this month was reversed { $count } times before
}

# review: pending
month-reverse-button = Reverse the close

# review: pending
month-reverse-title = Reverse the close of { $month }

# review: pending
month-reverse-description = One entry dated today will undo the close of { $month }. The reason is kept in the audit log. The month can be closed again after it.

# review: pending
month-reverse-reason = Reason

# review: pending
month-reverse-confirm = Reverse

# review: pending
month-reversed-toast = The close of { $month } is reversed

## What the backend refuses, in the month screen's own words.

# review: pending
month-open-day-close = Open Day Close

# review: pending
month-error-not-finished = { $month } is not finished yet. It can be closed after its last day.

# review: pending
month-error-days-open = Every day of { $month } must be closed first. The days are closed up to { $closed_up_to }. Close the days, then come back.

# review: pending
month-error-already-closed = { $month } is already closed. To close it again, reverse the close first.

# review: pending
month-error-nothing-to-close = { $month } has no income and no { -dukan-kharcha }, so there is nothing to close.

# review: pending
month-error-not-in-force = { $month } has no close to reverse. Open the list again.
