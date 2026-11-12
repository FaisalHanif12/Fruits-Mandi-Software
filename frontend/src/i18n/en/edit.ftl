# The one edit kit (plan 44, PRD FR-7.70, FR-7.71, FR-21.4). Every saved entry uses these words:
# the two buttons, the amber band, what was changed, the reason, the edit stamp and the history.
# Business words come from glossary.ftl. Every message here waits for the English reviewer.

# The two buttons of every saved entry, in the same place (PRD FR-7.70).
# review: pending
edit-action-edit = Edit

# review: pending
edit-action-cancel = Cancel

# The edit button of an entry of a closed day (PRD FR-21.4).
# review: pending
edit-action-fix-day = Reopen the day and fix

# review: pending
edit-action-history = Edit history

# The amber band over an entry that is open for edit. $what: the kind of entry (for example
# "Sale"), $number: its number, $date: its day, DD-MM-YYYY.
# Between two dates in a list of days.
# review: pending
edit-list-separator = { ", " }

# review: pending
edit-band = You are changing { $what } { $number } ({ $date })

# review: pending
edit-band-date-fixed = Its day cannot be changed. To move it to another day, cancel it and enter it again on the right day.

# The days that were reopened for this fix. $days: the dates, DD-MM-YYYY, one after another.
# review: pending
edit-band-days = Days reopened: { $days }. They close again after you save.

# review: pending
edit-back = Back without changing

# review: pending
edit-save = Save changes

# Why an entry cannot be changed (the backend's `locked_reason`). The id is `edit-locked-` and the
# code with dashes.
# review: pending
edit-locked-goods-moved = Its goods have been sold or written off, so it cannot be changed.

# review: pending
edit-locked-lot-seller-locked = A later sale of this { -lot } rests on this seller, so his name and phone cannot be changed.

# review: pending
edit-locked-garden-closed = The { -bagh } is closed, so this entry cannot be changed.

# review: pending
edit-locked-month-closed = Its month is closed. The Owner must reverse the month close first.

# review: pending
edit-locked-cancelled = This entry is cancelled.

# review: pending
edit-locked-returns-stand = Returns are recorded against it, so it cannot be changed.

# review: pending
edit-locked-unknown = This entry cannot be changed now.

# What was changed, before the save (PRD FR-7.70). The backend lists each change and its effect.
# review: pending
edit-review-title = What you changed

# review: pending
edit-review-none = Nothing is changed yet.

# review: pending
edit-review-loading = Checking the changes…

# review: pending
edit-review-failed = The effect of the changes could not be checked.

# One change. $where: the place ("Lot 1, line 2 (Akram)") or empty, $field: the name of the field,
# $before and $after: the old and the new value.
# review: pending
edit-review-change = { $where }{ $field } was { $before }, now { $after }

# review: pending
edit-review-where-header = Header:

# $name: the lot's name in brackets, for example " (Ehsan, Mango-2)", or nothing.
# review: pending
edit-review-where-lot = { -lot } { $lot }{ $name }:

# review: pending
edit-review-where-line = { -lot } { $lot }, line { $line }{ $label }:

# review: pending
edit-review-where-part = Part{ $label }:

# review: pending
edit-review-where-charge = Charge{ $label }:

# The figures that follow from the change and are worked out again.
# review: pending
edit-review-worked-title = Figures worked out again

# review: pending
edit-review-effect-title = Its effect

# review: pending
edit-review-effect-none = No account balance changes.

# review: pending
edit-review-effect-account = Account

# review: pending
edit-review-effect-before = Before

# review: pending
edit-review-effect-after = After

# $before and $after: the cash in hand before and after the change.
# review: pending
edit-review-effect-cash = { -rokar } was { $before }, now { $after }

# review: pending
edit-review-effect-partner = Partner

# The partner figures are behind the partner code (PRD FR-17.11).
# review: pending
edit-review-partners-locked = Partner figures are not shown without the partner code.

# review: pending
edit-review-unlock = Enter the code

# The reason of an edit. Asked when the entry is not of today (PRD FR-7.70).
# review: pending
edit-reason-title = Why is this entry being changed?

# $date: the day of the entry, DD-MM-YYYY.
# review: pending
edit-reason-text = The entry of { $date } is not of today. The reason is written to the audit log with the old and the new values.

# review: pending
edit-reason-label = Reason

# review: pending
edit-reason-confirm = Save changes

# The reopen and fix in one step (PRD FR-21.4).
# review: pending
edit-fix-title = Reopen the day and fix?

# $date: the day of the entry, DD-MM-YYYY. The days after it open too.
# review: pending
edit-fix-text = The day { $date } and every closed day after it will reopen. You fix the entry, and after you save, the days close again in the same order. Write the reason and the Owner's { -dastakhat }.

# review: pending
edit-fix-label = Reason for reopening

# The Owner's sign, below the reason (PRD FR-21.4, plan 73).
# review: pending
edit-fix-sign-label = { -dastakhat }

# review: pending
edit-fix-sign-hint = Only the Owner's { -dastakhat } opens a closed day.

# review: pending
edit-fix-confirm = Reopen the days

# review: pending
edit-fix-closed-toast = Days closed again: { $days }

# review: pending
edit-fix-stayed-title = These days could not be closed

# $date: the day that stayed open.
# review: pending
edit-fix-stayed-text = The day { $date } stayed open. It will not close until this is put right.

# review: pending
edit-fix-stayed-ok = OK

# review: pending
edit-fix-failed = The days could not be closed

# The edit stamp (PRD FR-7.71). $date: DD-MM-YYYY, $time: the time with its part of the day,
# $by: who.
# review: pending
edit-stamp = Edited: { $date } { $time }, { $by }

# The same, when the edit was made through a reopened day: $sign is the sign given then (PRD FR-7.71, v1.36).
# review: pending
edit-stamp-signed = Edited: { $date } { $time }, { $by }, { -dastakhat }: { $sign }

# review: pending
edit-stamp-reason = Reason: { $reason }

# review: pending
edit-stamp-count = { $count } times

# The small mark on a list row of an edited entry.
# review: pending
edit-mark = Edited

# review: pending
edit-mark-tip = Edited: { $date } { $time }, { $by }

# review: pending
edit-mark-tip-signed = Edited: { $date } { $time }, { $by }, { -dastakhat }: { $sign }

# The time of day. $hour and $minute are the clock, $meridian is am or pm.
# review: pending
edit-time = { $hour }:{ $minute } { $meridian ->
    [am] am
   *[pm] pm
}

# The mark on a field that was changed. $before and $after: the old and the new value.
# review: pending
edit-field-mark = Before: { $before } → now: { $after }

# The history of an entry.
# review: pending
edit-history-title = Edit history

# review: pending
edit-history-empty = No change is recorded for this entry.

# review: pending
edit-history-empty-help = When the entry is changed, each change will be listed here.

# review: pending
edit-history-failed = The edit history could not be opened.

# review: pending
edit-history-version = { $date } { $time }, { $by }

# A version made through a reopened day: its sign follows the name (PRD FR-7.71, v1.36).
# review: pending
edit-history-version-signed = { $date } { $time }, { $by }, { -dastakhat }: { $sign }

# review: pending
edit-history-field = Field

# review: pending
edit-history-old = Before

# review: pending
edit-history-new = Now

# review: pending
edit-history-truncated = There are more changes that are not shown here.

# review: pending
edit-history-close = Close

# What an old action was.
# review: pending
edit-history-action-create = Entry made

# review: pending
edit-history-action-edit = Entry changed

# review: pending
edit-history-action-cancel = Entry cancelled

# review: pending
edit-history-action-other = Other action

# An empty value, and a value that is yes or no.
# review: pending
edit-empty = (empty)

# The print header of an edited entry, and of a report that holds one (PRD FR-7.71). $date.
# review: pending
edit-print-revised = Revised: { $date }

# The names of the fields of an edit (the backend's field keys). The id is `edit-field-` and the key
# with dashes.
# review: pending
edit-field-entry-date = Date

# review: pending
edit-field-bill-date = Bill date

# review: pending
edit-field-truck-no = Truck number

# review: pending
edit-field-seller = { -beopari }

# review: pending
edit-field-buyer = { -gahak }

# review: pending
edit-field-account = Account

# review: pending
edit-field-person = Name

# review: pending
edit-field-partner = Partner

# review: pending
edit-field-item = { -qism-maal }

# review: pending
edit-field-packing = { -packing }

# review: pending
edit-field-marka = { -marka }

# review: pending
edit-field-adda = { -adda }

# review: pending
edit-field-store = Store

# review: pending
edit-field-qty = Quantity

# review: pending
edit-field-weight = Weight

# review: pending
edit-field-rate = Rate

# review: pending
edit-field-seller-rate = Rate, { -beopari }

# review: pending
edit-field-buyer-rate = Rate, { -gahak }

# review: pending
edit-field-laga = { -laga }

# review: pending
edit-field-freight = { -kiraya }

# review: pending
edit-field-freight-paid-by = { -kiraya } paid by

# review: pending
edit-field-labour = { -mazdoori }

# review: pending
edit-field-munshiana = { -munshiana }

# review: pending
edit-field-phone = { -phone }

# review: pending
edit-field-cash = { -naqd }

# review: pending
edit-field-store-rent = { -kiraya-store }

# review: pending
edit-field-adjust-advance = Deduct from advance

# review: pending
edit-field-paid = { -mauqa-par-jama }

# review: pending
edit-field-amount = Amount

# review: pending
edit-field-value = Value

# review: pending
edit-field-collector = Collector

# review: pending
edit-field-head = Expense head

# review: pending
edit-field-paid-to = Paid to

# review: pending
edit-field-return-by = Return by

# review: pending
edit-field-taken-by = Taken by

# review: pending
edit-field-mode = Mode

# review: pending
edit-field-narration = Narration

# review: pending
edit-field-reason = Reason

# review: pending
edit-field-note = Note

# review: pending
edit-field-name = Name

# review: pending
edit-field-name-ur = Urdu name

# review: pending
edit-field-name-en = English name

# review: pending
edit-field-supplier = Supplier

# review: pending
edit-field-shop-name = Shop

# review: pending
edit-field-market = Market

# review: pending
edit-field-sent-person = Sent

# review: pending
edit-field-cost = Cost

# review: pending
edit-field-source = Source

# review: pending
edit-field-share = Share

# review: pending
edit-field-money-source = Money from

# review: pending
edit-field-sell-date = Sell day

# review: pending
edit-field-lot = { -lot }

# review: pending
edit-field-credit-limit = Credit limit

# review: pending
edit-field-city = City

# review: pending
edit-field-address = Address

# review: pending
edit-field-active = Active

# review: pending
edit-field-value-text = Rate

# review: pending
edit-field-to-date = To date

# review: pending
edit-field-other = Other field

# review: pending
edit-field-seller-comm = { -commission }, { -beopari }

# review: pending
edit-field-buyer-comm = { -commission }, { -gahak }

# review: pending
edit-field-gross = { -kham-bikri }

# review: pending
edit-field-net = { -safi-bikri }

# review: pending
edit-field-seller-amount = Seller amount

# review: pending
edit-field-buyer-value = Value, { -gahak }

# review: pending
edit-field-buyer-total = Buyer total

# The edit of an entry has been saved (plan 44). $number: the number of the entry.
# review: pending
edit-saved = { $number } changed

# The fields the real edit answers of a purchase, a garden and a trip name (plan 44).
# review: pending
edit-field-paid-now = Paid now

# review: pending
edit-field-rent = Full { -thaika }

# review: pending
edit-field-total-cost = Total cost

# The small mark on a field that an edit changed (PRD FR-7.71): what it held before. $before.
# review: pending
edit-mark-before = Before { $before }

# The whole mark with the time and the name of the edit: "Before: 600 → now: 610, 31-10-2026 4:35 pm,
# Owner". $mark: the old against the new value, $when: the day, the time and who.
# review: pending
edit-field-mark-full = { $mark }, { $when }

# What the mark says when the old value is a record this screen cannot name.
# review: pending
edit-mark-other = (another one)
