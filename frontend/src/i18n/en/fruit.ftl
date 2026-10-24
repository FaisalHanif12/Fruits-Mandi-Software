# The { -phal } screen, a tab next to { -kharcha } in the { -rokar } area (plan 17d, PRD 16.7,
# FR-16.10 to FR-16.12, FR-7.53, C16, C27).
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The screen.
# review: pending
fruit-title = { -phal }
# review: pending
fruit-help = Fruit a partner takes home, or fruit served to the shop's { -mehman }, recorded by { -qism-maal } and quantity. A partner's fruit comes off his { -kul-raqam } only. Fruit for { -mehman } is shared equally between the partners, not by their shares of the { -commission }. No cash moves on this screen.
# review: pending
fruit-load-failed = The { -phal } screen could not be opened

## The fields.
# review: pending
fruit-field-for = For
# review: pending
fruit-for-guests = { -mehman }
# review: pending
fruit-for-hint = A partner: for his home. { -mehman }: for the shop's guests, shared equally between the partners.
# review: pending
fruit-partners-failed = The partners could not be loaded
# review: pending
fruit-field-date = Date
# review: pending
fruit-field-source = From where
# review: pending
fruit-source-own = { -apni-dukan }
# review: pending
fruit-source-other = { -dusri-dukan }
# review: pending
fruit-source-own-help = Fruit from a { -lot } in { -baqi-maal }. The balance of the { -lot } goes down, and its { -beopari } is credited the value, with no { -commission }.
# review: pending
fruit-source-other-help = Fruit from another { -arhti }'s shop. His account is credited the value. The shop pays him later with a normal { -adaigi }.
# review: pending
fruit-field-lot = { -lot }
# review: pending
fruit-lot-hint = Only lots with goods left are offered.
# $bikri, $lot: the numbers; $item, $packing, $seller: names; $left: the lot's balance.
# review: pending
fruit-lot-option = { -bikri } { $bikri }, { -lot } { $lot }: { $item }, { $packing }, { $seller }, { $left } left
# review: pending
fruit-lot-empty = No lot has goods left
# review: pending
fruit-lot-failed = The lots could not be loaded
# review: pending
fruit-field-agent = { -arhti }
# review: pending
fruit-agent-hint = The seller account of the other commission agent. Add it with F4 if it is not there yet.
# review: pending
fruit-item-from-lot = Taken from the lot
# review: pending
fruit-field-qty = Quantity
# $left: the lot's balance, from the backend.
# review: pending
fruit-qty-left = The lot has { $left } left
# review: pending
fruit-field-weight = Weight (kg)
# review: pending
fruit-field-rate = Rate
# review: pending
fruit-rate-hint-note = The grey text is only a hint. Type the rate.
# review: pending
fruit-field-note = Note

## What the save will do, from the backend before the save (PRD FR-16.10, C27).
# review: pending
fruit-numbers-title = This entry
# review: pending
fruit-numbers-empty = Fill in the form and the figures show here.
# review: pending
fruit-numbers-incomplete = The figures show when the form is complete.
# review: pending
fruit-numbers-failed = The figures could not be worked out
# review: pending
fruit-numbers-value = Value
# $name: the lot's seller, or the other agent.
# review: pending
fruit-numbers-credit = Credited to { $name }
# review: pending
fruit-numbers-lot-after = Balance of the lot after
# $name: the partner.
# review: pending
fruit-parts-title-home = Comes off the { -kul-raqam } of { $name }
# review: pending
fruit-parts-title-guests = Shared equally between the partners
# review: pending
fruit-parts-note = A rupee that does not divide goes to the partner made first. The { -commission } shares are not used.

## Saving, editing and cancelling (PRD FR-16.12).
# $number: the number of the entry in its own series.
# review: pending
fruit-number = FRT-{ $number }
# $number: the entry number, written with fruit-number.
# review: pending
fruit-saved = { $number } saved
# review: pending
fruit-edit = Edit
# review: pending
fruit-editing = Editing { $number }. The old entry is reversed and the new one is posted on the same day.
# review: pending
fruit-cancel = Cancel entry
# review: pending
fruit-cancel-title = Cancel this entry?
# review: pending
fruit-cancel-text = { $number } is reversed: the partners' totals, the lot's balance and the seller's account go back as they were. It stays in the list as cancelled.
# review: pending
fruit-cancel-reason = Reason
# review: pending
fruit-cancel-confirm = Cancel entry
# review: pending
fruit-cancelled-toast = { $number } cancelled

## The list under the form (PRD FR-16.12, FR-19.11).
# review: pending
fruit-list-title = Fruit taken
# review: pending
fruit-list-label = List of fruit taken
# review: pending
fruit-list-failed = The list could not be opened
# review: pending
fruit-list-empty = No fruit recorded yet
# review: pending
fruit-list-empty-help = Fill in the form above and save. Each entry shows here.
# review: pending
fruit-list-no-match = No entry matches these choices
# review: pending
fruit-list-no-match-help = Try another partner, item or period.
# review: pending
fruit-list-clear = Show everything
# review: pending
fruit-col-number = Number
# review: pending
fruit-col-date = Date
# review: pending
fruit-col-for = For
# review: pending
fruit-col-item = { -qism-maal } and { -packing }
# review: pending
fruit-col-qty = { -nag }
# review: pending
fruit-col-rate = Rate
# review: pending
fruit-col-value = Value
# review: pending
fruit-col-source = From where
# review: pending
fruit-col-from = From
# review: pending
fruit-col-state = State
# review: pending
fruit-state-cancelled = Cancelled
# review: pending
fruit-filter-for = For
# review: pending
fruit-filter-for-all = Everyone
# review: pending
fruit-filter-item = { -qism-maal }
# review: pending
fruit-filter-item-all = All items
# review: pending
fruit-filter-from = From
# review: pending
fruit-filter-to = To
