### The روکڑ area (plan 17, PRD 16): the cash book, the ONE خرچہ screen (دکان خرچہ, ذاتی خرچہ and its واپسی), ادھار رقم and الگ کھاتہ.
### Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The cash book (PRD 16.1, FR-16.1, FR-19.11). There is no box to type counted cash.

# review: pending
cashbook-title = { -rokar }

# review: pending
cashbook-help = All the shop's cash in one place: every amount that came in or went out, from every part of the app, with the cash in hand after each.

# review: pending
cashbook-new-expense = New { -dukan-kharcha }

# review: pending
cashbook-filter-from = From

# review: pending
cashbook-filter-to = To

# review: pending
cashbook-opening = Cash at the start

# review: pending
cashbook-closing = Cash at the end

# review: pending
cashbook-list-label = { -rokar }

# review: pending
cashbook-empty = No cash moved on these days

# review: pending
cashbook-empty-help = Choose other days, or enter an expense.

# review: pending
cashbook-load-failed = The { -rokar } could not be opened

## Columns of the lists.

# review: pending
cashbook-col-date = Date

# review: pending
cashbook-col-kind = Entry

# review: pending
cashbook-col-account = Account

# review: pending
cashbook-col-narration = Narration

# review: pending
cashbook-col-in = In

# review: pending
cashbook-col-out = Out

# review: pending
cashbook-col-running = Cash after

# review: pending
cashbook-col-number = No.

# review: pending
cashbook-col-head = Head

# review: pending
cashbook-col-amount = Amount

# review: pending
cashbook-col-state = State

# review: pending
cashbook-col-reason = Reason

# review: pending
cashbook-col-taken-by = Taken by

# review: pending
cashbook-col-return-by = Return by

# review: pending
cashbook-col-taken = Taken

# review: pending
cashbook-col-returned = Returned

# review: pending
cashbook-col-open = Still open

# review: pending
cashbook-col-days-past = Days late

# review: pending
cashbook-col-lent = Lent

# review: pending
cashbook-col-balance = Balance

# review: pending
cashbook-state-cancelled = Cancelled

## What a row of the cash book is. The kind is the stored posting type, written with hyphens.

# $kind: the word for the kind; $number: its number.
# review: pending
cashbook-kind-number = { $kind } { $number }

# review: pending
cashbook-kind-bikri-first = { -bikri }

# review: pending
cashbook-kind-bikri-later = Later sale

# review: pending
cashbook-kind-reversal = Reversal

# review: pending
cashbook-kind-receipt = { -wasooli }

# review: pending
cashbook-kind-payment = { -adaigi }

# review: pending
cashbook-kind-advance = { -peshgi-khata }

# The advance a seller took with the goods of a sale (plan 95 item 11, PRD FR-12.52): the number after it is the sale's.
# review: pending
cashbook-kind-bikri-advance = { -peshgi } with { -bikri }

# review: pending
cashbook-kind-shop-expense = { -dukan-kharcha }

# review: pending
cashbook-kind-personal-expense = { -zati-kharcha }

# review: pending
cashbook-kind-personal-return = { -zati-kharcha } { -wapsi }

# review: pending
cashbook-kind-money-lent = { -adhar-raqam }

# review: pending
cashbook-kind-money-lent-return = { -adhar-raqam } { -wapsi }

# review: pending
cashbook-kind-capital-in = Capital invested

# review: pending
cashbook-kind-capital-out = Capital taken back

# review: pending
cashbook-kind-khata-out = { -alag-khata }: money paid out

# review: pending
cashbook-kind-khata-in = { -alag-khata }: money received

# review: pending
cashbook-kind-purchase = { -khareedari }

# review: pending
cashbook-kind-trip-cost = Trip cost

# review: pending
cashbook-kind-garden-payment = Garden payment

# review: pending
cashbook-kind-venture-receipt = Sale in another shop

# review: pending
cashbook-kind-garden-close = Garden closed

# review: pending
cashbook-kind-fruit-taken = Fruit taken

# review: pending
cashbook-kind-general = { -general }

# review: pending
cashbook-kind-opening = Opening balance

# review: pending
cashbook-kind-month-close = Month close

## Shared by the entry screens.

# review: pending
cashbook-new = New entry

# review: pending
cashbook-field-date = Date

# review: pending
cashbook-field-amount = Amount

# review: pending
cashbook-field-narration = Narration

# review: pending
cashbook-field-reason = Reason

# review: pending
cashbook-field-return-by = Return by

# review: pending
cashbook-preview-failed = What this entry will do could not be worked out

# review: pending
cashbook-balance-now = Balance now

# review: pending
cashbook-balance-after = Balance after this entry

# $kind: the voucher type; $number: the number in its series, made by vouchers-number-<type>.
# review: pending
cashbook-saved = { $kind ->
    [shop_expense] { -dukan-kharcha } { $number } saved
    [personal_expense] { -zati-kharcha } { $number } saved
    [personal_return] { -zati-kharcha } { -wapsi } { $number } saved
    [money_lent] { -adhar-raqam } { $number } saved
    [money_lent_return] { -adhar-raqam } { -wapsi } { $number } saved
    [khata_out] { -alag-khata }: money paid out { $number } saved
    [general] { -general } { $number } saved
    [capital_in] { -sarmaya } put in { $number } saved
    [capital_out] { -sarmaya } taken back { $number } saved
   *[khata_in] { -alag-khata }: money received { $number } saved
}

## The type lists (PRD FR-7.67).

# review: pending
cashbook-type-shop-expense = { -dukan-kharcha }

# review: pending
cashbook-type-personal-expense = { -zati-kharcha }

# review: pending
cashbook-type-personal-return = { -zati-kharcha } { -wapsi }

# review: pending
cashbook-type-money-lent = Money lent

# review: pending
cashbook-type-money-lent-return = { -wapsi }

# review: pending
cashbook-type-khata-out = Paid out

# review: pending
cashbook-type-khata-in = Received

# review: pending
cashbook-tab-expense = { -kharcha }

# review: pending
cashbook-tab-fruit = { -phal }

# review: pending
cashbook-tabs-label = Sections of { -kharcha }

## The { -kharcha } screen (PRD 16.2, 16.3, FR-16.8, FR-7.68, FR-7.62).

# review: pending
cashbook-expense-title = { -kharcha }

# review: pending
cashbook-expense-help = Pick the type first. A { -dukan-kharcha } is split between the partners by their shares. A { -zati-kharcha } is charged to one partner only. Below the form you see what saving will do.

# review: pending
cashbook-expense-type = Type of { -kharcha }

# review: pending
cashbook-field-head = Expense head

# review: pending
cashbook-field-partner = { -shirakat-dar }

# review: pending
cashbook-field-paid-to = Paid to

# review: pending
cashbook-field-taken-by = Taken by

# review: pending
cashbook-heads-failed = The expense heads could not be opened

# review: pending
cashbook-partners-failed = The partners could not be opened

# review: pending
cashbook-preview-hint-shop = Type the amount and choose the head. Each partner's share shows here before you save.

# review: pending
cashbook-preview-hint-personal = Choose the partner and type the amount. His total now and after shows here before you save.

# review: pending
cashbook-shares-title = Each partner's share

# review: pending
cashbook-shares-none = There are no shares to split this by on this date.

# review: pending
cashbook-effect-title = What this does to the partner

# review: pending
cashbook-effect-none = Choose the partner to see the effect.

# The entry is on a linked khata or a loan linked to a partner or the shop: what the save will do to each partner's total.
# review: pending
cashbook-linked-parts-title = What this does to each partner's { -kul-raqam }

# review: pending
cashbook-effect-total-now = Partner total now

# review: pending
cashbook-effect-total-after = Partner total after

# review: pending
cashbook-effect-open-now = Taken and not returned now

# review: pending
cashbook-effect-open-after = Taken and not returned after

# review: pending
cashbook-others-not-affected = The other partners are not affected.

# PRD FR-7.68. $name: the partner the "paid to" text matches.
# review: pending
cashbook-personal-hint = Is this a partner's own spending? { $name } is a partner.

# review: pending
cashbook-personal-make = Make it { -zati-kharcha }

## Repeat last month (PRD FR-7.62).

# review: pending
cashbook-repeat-open = Repeat last month

# review: pending
cashbook-repeat-title = Last month's regular expenses

# review: pending
cashbook-repeat-help = These heads were entered last month and not yet this month. Fill one into the form, check it and save it.

# review: pending
cashbook-repeat-fill = Fill the form

# review: pending
cashbook-repeat-last-date = Last month: <num>{ $date }</num>

# review: pending
cashbook-repeat-total = Total of these

# review: pending
cashbook-repeat-close = Close

# review: pending
cashbook-repeat-empty = Nothing to repeat

# review: pending
cashbook-repeat-empty-help = Every head from last month is already entered this month, or there was none.

# review: pending
cashbook-repeat-failed = Last month's expenses could not be opened

## The list under the form.

# review: pending
cashbook-day-title = Shop expenses of the day

# review: pending
cashbook-day-label = Shop expenses of the day

# review: pending
cashbook-day-empty = No shop expense on this day

# review: pending
cashbook-day-empty-help = Fill the form above and save to add one.

# review: pending
cashbook-day-failed = The shop expenses of the day could not be opened

# review: pending
cashbook-takes-title = Money the partner took

# review: pending
cashbook-takes-label = Money the partner took

# review: pending
cashbook-takes-empty = This partner has taken no money

# review: pending
cashbook-takes-empty-help = What he takes for himself and what he gives back is listed here.

# review: pending
cashbook-takes-failed = The partner's money taken could not be opened

# review: pending
cashbook-takes-pick = Choose a partner

# review: pending
cashbook-takes-pick-help = His money taken and given back shows here.

# review: pending
cashbook-take-opening = Opening balance

## The { -adhar-raqam } screen (PRD 16.5, FR-16.4, FR-16.5).

# review: pending
cashbook-lent-screen-title = { -adhar-raqam }

# review: pending
cashbook-lent-screen-help = Money the shop lends to staff or anyone else, and what they give back. A return can be in cash or taken from his salary. Press F4 in the person field to add a new person.

# review: pending
cashbook-lent-type = Type of entry

# review: pending
cashbook-field-person = Person

# review: pending
cashbook-field-mode = How it is returned

# review: pending
cashbook-mode-cash = In cash

# review: pending
cashbook-mode-salary = Deducted from salary

# review: pending
cashbook-field-salary-head = Expense head of the salary

# review: pending
cashbook-field-salary-head-hint = Left empty, the default salary head is used.

# review: pending
cashbook-salary-head-default = Salary (default)

# review: pending
cashbook-salary-note = No cash moves. That part of the salary counts as a shop expense and is split between the partners.

# review: pending
cashbook-lent-title = The person's loans

# review: pending
cashbook-lent-label = The person's loans

# review: pending
cashbook-lent-balance = He still owes

# review: pending
cashbook-lent-empty = This person has no loan

# review: pending
cashbook-lent-empty-help = Money lent to him and what he gives back will be listed here.

# review: pending
cashbook-lent-failed = The person's loans could not be opened

# review: pending
cashbook-lent-pick = Choose a person

# review: pending
cashbook-lent-pick-help = His loans, what he returned and what is still open show here.

## The { -alag-khata } screen (PRD 16.6, FR-16.6).

# review: pending
cashbook-khata-screen-title = { -alag-khata }

# review: pending
cashbook-khata-screen-help = Money that has nothing to do with the partners, commission or profit, for example a family khata. Money paid out makes it minus; money received makes it plus. Press F4 in the khata field to add a new khata.

# review: pending
cashbook-khata-type = Type of entry

# review: pending
cashbook-field-khata = { -alag-khata }

# review: pending
cashbook-khata-title = The khata's entries

# review: pending
cashbook-khata-label = The khata's entries

# review: pending
cashbook-khata-opening = Balance at the start

# review: pending
cashbook-khata-closing = Balance now

# review: pending
cashbook-khata-sign-hint = Plus: the shop holds money for this khata. Minus: it has taken more than it brought. On the balance after an entry, { -jama } is plus and { -banaam } is minus.

# review: pending
cashbook-khata-empty = This khata has no entry

# review: pending
cashbook-khata-empty-help = Money paid out and received will be listed here, with the reason.

# review: pending
cashbook-khata-failed = The khata's entries could not be opened

# review: pending
cashbook-khata-pick = Choose a khata

# review: pending
cashbook-khata-pick-help = Its entries and its running balance show here.

# review: pending
cashbook-khata-kind-paid-out = Paid out

# review: pending
cashbook-khata-kind-received = Received

# review: pending
cashbook-khata-kind-sale = Goods sold

## Adding a person or a khata without leaving the screen (F4).

# review: pending
cashbook-add-person-title = New person

# review: pending
cashbook-add-person-help = Add a person who can borrow money from the shop. Staff can be linked to their staff record.

# review: pending
cashbook-add-person-saved = Person added

# review: pending
cashbook-add-khata-title = New { -alag-khata }

# review: pending
cashbook-add-khata-help = Add a khata that is kept apart from the partners, commission and profit.

# review: pending
cashbook-add-khata-saved = Khata added

# review: pending
cashbook-add-name-ur = Urdu name

# review: pending
cashbook-add-name-en = English name

# review: pending
cashbook-add-phone = Phone

# review: pending
cashbook-add-staff = Staff record

# review: pending
cashbook-add-staff-hint = Only if he works for the shop.

# review: pending
cashbook-add-note = Note

## The جنرل screen (route /rokar/general, PRD 16.4).

# review: pending
cashbook-general-screen-title = { -general }

# review: pending
cashbook-general-screen-help = An entry of lines, each with an account and an amount on the { -banaam } or the { -jama } side. It can be saved only when the two sides are equal.

# review: pending
cashbook-general-lines-title = Lines

# review: pending
cashbook-general-grid-label = Lines of the { -general } entry

# review: pending
cashbook-general-add-line = Add line

# review: pending
cashbook-general-col-account = Account

# review: pending
cashbook-general-totals-title = { -meezan }

# review: pending
cashbook-general-totals-hint = Type the lines. The totals show here.

# review: pending
cashbook-general-total-dr = { -banaam } total

# review: pending
cashbook-general-total-cr = { -jama } total

# review: pending
cashbook-general-difference = Difference ({ -banaam } less { -jama })

# review: pending
cashbook-general-balanced = The two sides are equal. The entry can be saved.

# review: pending
cashbook-general-not-balanced = The two sides are not equal yet. The entry cannot be saved until they are.

# review: pending
cashbook-general-split-title = Each { -shirakat-dar }'s part

# review: pending
cashbook-general-split-help = Lines on income and { -dukan-kharcha } accounts are split between the partners by their shares.

# review: pending
cashbook-type-capital-in = { -sarmaya } put in

# review: pending
cashbook-type-capital-out = { -sarmaya } taken back

# The tick of the lists of money taken and money lent: a take or a loan that is fully returned is
# left out until it is on.
# review: pending
cashbook-show-returned = Show returned too

# Shown above the list when it is sorted by anything but the day: the cash after a line needs the order of days.
# review: pending
cashbook-running-date-only = The cash after each line shows in date order only.

## An entry of the روکڑ area changed (plan 44, PRD FR-7.70).

# $number: the number of the entry, for example "E-3".
# review: pending
cashbook-edited = { $number } changed

# The list of saved journal entries under the جنرل form (plan 44, PRD FR-7.70).
# review: pending
cashbook-general-list-title = Saved { -general } entries

# review: pending
cashbook-general-list-label = Saved { -general } entries

# review: pending
cashbook-general-list-failed = The saved { -general } entries could not be opened

# review: pending
cashbook-general-list-empty = No { -general } entry in these dates

# review: pending
cashbook-general-list-empty-help = Fill the lines above and save to add one. Use the buttons below to see another month.

## The link of a { -alag-khata } and of a person who is lent money, and the type of that person (plan 76 points 5 and 10, PRD FR-16.6, FR-16.13)

# review: pending
cashbook-link-label = Linked to

# review: pending
cashbook-link-for = For

# review: pending
cashbook-link-kind-none = Nobody (the firm's own)

# review: pending
cashbook-link-kind-shop = The shop (split by shares)

# review: pending
cashbook-link-kind-partner = One { -shirakat-dar }

# review: pending
cashbook-link-partner = { -shirakat-dar }

# review: pending
cashbook-link-hint-khata = Money paid out lowers the { -kul-raqam } of the partners it is linked to, like money taken, and money back raises it. It is never income or expense.

# review: pending
cashbook-link-hint-person = Linked to a { -shirakat-dar }, the money lent lowers that partner's { -kul-raqam }. Linked to the shop, it is still owed to the shop and no partner total changes.

# review: pending
cashbook-link-locked = The link cannot be changed while the balance is not zero.

# review: pending
cashbook-link-change = Change the link

# review: pending
cashbook-link-saved = The link is saved

# review: pending
cashbook-person-type = Type

# review: pending
cashbook-person-type-staff = Staff

# review: pending
cashbook-person-type-garden-owner = Garden owner

# review: pending
cashbook-person-type-trip-person = { -market-safar } person

# review: pending
cashbook-person-type-other = Other

# review: pending
cashbook-person-change = Change type or link

# review: pending
cashbook-person-saved = The person is saved

## The page behind the dashboard tile of garden owners and others (plan 76 point 10, PRD FR-16.13)

# review: pending
cashbook-dues-title = Garden owners' and others' dues

# review: pending
cashbook-dues-help = { -adhar-raqam } given to people who are not staff (garden owners, { -market-safar } people and others) and what came back, one month at a time. The figures are the ones of the tile.


# review: pending
cashbook-dues-filter-type = Type

# review: pending
cashbook-dues-filter-link = Linked to

# review: pending
cashbook-dues-type-default = Everyone except staff

# review: pending
cashbook-dues-link-all = All

# review: pending
cashbook-dues-link-shop = The shop

# review: pending
cashbook-dues-link-partner = One { -shirakat-dar }

# review: pending
cashbook-dues-open = Still open

# review: pending
cashbook-dues-people = People who owe

# review: pending
cashbook-dues-given = Given this month

# review: pending
cashbook-dues-returned = Returned this month

# review: pending
cashbook-dues-people-title = People with something open

# review: pending
cashbook-dues-people-label = People who owe money

# review: pending
cashbook-dues-people-empty = Nobody owes anything

# review: pending
cashbook-dues-people-empty-help = A person shows here as soon as he is given money and has not returned it.

# review: pending
cashbook-dues-entries-title = Entries of the month

# review: pending
cashbook-dues-entries-label = Money given and returned in the month

# review: pending
cashbook-dues-failed = The dues could not be read

# review: pending
cashbook-dues-empty = Nothing was given or returned in this month

# review: pending
cashbook-dues-empty-help = Use the buttons below to go to another month.

# review: pending
cashbook-dues-col-person = Person

# review: pending
cashbook-dues-col-type = Type

# review: pending
cashbook-dues-col-link = Linked to

# review: pending
cashbook-dues-col-kind = Entry

# review: pending
cashbook-dues-col-amount = Amount

# review: pending
cashbook-dues-col-lent = Given

# review: pending
cashbook-dues-col-returned = Returned

# review: pending
cashbook-dues-col-open = Open

# review: pending
cashbook-dues-kind-given = Given

# review: pending
cashbook-dues-kind-returned = Returned

# The short words of a link, for a list or a card (the choice box has the long ones above).
# review: pending
cashbook-link-word-none = Nobody

# review: pending
cashbook-link-word-shop = The shop

# review: pending
cashbook-link-word-partner = One { -shirakat-dar }

## The rows that start the book or move a partner's capital (plan 78 E, PRD FR-16.14). $partner: the partner, with his tag.

# review: pending
cashbook-row-opening-cash = Opening cash

# review: pending
cashbook-row-opening-capital = Opening { -sarmaya }: { $partner }

# review: pending
cashbook-row-capital-in = { -sarmaya } invested: { $partner }

# review: pending
cashbook-row-capital-out = { -sarmaya } taken back: { $partner }

## Where the capital is, a card of the cash book behind the partner code (plan 87, PRD FR-16.14). It reads like a statement:
## a headline, two blocks (to receive and in hand, to pay), whose it is, and a warning for the opening difference.

# review: pending
cashbook-capital-title = Where the { -sarmaya } is

# review: pending
cashbook-capital-help = Where the partners' money is now, and whose it is. The totals of the two sides are always equal.

# review: pending
cashbook-capital-failed = The { -sarmaya } could not be read

# review: pending
cashbook-capital-headline = Partners' total { -sarmaya }

# review: pending
cashbook-capital-equal = Both sides are equal

# review: pending
cashbook-capital-unequal = The two sides differ, check the reports

# review: pending
cashbook-capital-as-of = As of { $date }

# review: pending
cashbook-capital-go-label = Show it as of

# review: pending
cashbook-capital-go-info = Type a date, for example 10-10-2026. The { -sarmaya } is shown as it stood on that day.

# review: pending
cashbook-capital-go-placeholder = Type a date

# review: pending
cashbook-capital-go-today = Up to today

# review: pending
cashbook-capital-empty = No { -sarmaya } is recorded yet

# review: pending
cashbook-capital-empty-help = First put in a partner's { -sarmaya }. Then this shows where the money is and whose it is.

# review: pending
cashbook-capital-empty-action = Put in { -sarmaya }

# review: pending
cashbook-capital-receive-title = To receive and in hand

# review: pending
cashbook-capital-owe-title = To pay

# review: pending
cashbook-capital-whose-title = Whose it is

# review: pending
cashbook-capital-net-label = Net

# review: pending
cashbook-capital-col-what = Detail

# review: pending
cashbook-capital-col-amount = Amount

# review: pending
cashbook-capital-col-share = { -hissa }

# review: pending
cashbook-capital-where-cash = Cash in hand

# review: pending
cashbook-capital-where-buyers-owe = Owed by { -gahakon }

# review: pending
cashbook-capital-where-dead-amount = { -doobi-raqam }

# review: pending
cashbook-capital-where-seller-advances = { -peshgi } with { -beopariyon }

# review: pending
cashbook-capital-where-sellers-owe-beyond = Owed by { -beopariyon } beyond their { -peshgi }

# review: pending
cashbook-capital-where-money-lent = { -adhar-raqam } not yet returned

# review: pending
cashbook-capital-where-separate-khatas = { -alag-khata }

# review: pending
cashbook-capital-where-cash-short = Cash short

# review: pending
cashbook-capital-cash-short-note = Cash in hand is below zero, so this amount is on the paying side.

# review: pending
cashbook-capital-where-sellers-payable = Owed to { -beopariyon }

# review: pending
cashbook-capital-where-owed-for-charges = Owed for { -mazdoori }, { -kiraya }, { -munshiana } and stores

# review: pending
cashbook-capital-where-gardens-held = Money of { -baghat } held until they close

# review: pending
cashbook-capital-where-linked-in-partner-totals = Linked money already in the partners' { -kul-raqam }

# review: pending
cashbook-capital-where-other = Other

# review: pending
cashbook-capital-whose-partner-total = { -kul-raqam }: { $partner }

# review: pending
cashbook-capital-whose-owes = Owes the firm

# review: pending
cashbook-capital-whose-opening-capital = Opening { -sarmaya }

# review: pending
cashbook-capital-whose-advances-open = Partners' part of the open { -peshgi }

# review: pending
cashbook-capital-whose-other = Other

# review: pending
cashbook-capital-share = <num>{ $percent }</num>%

## The warning of the opening difference. $amount and its side are drawn by the screen (<balance>).

# review: pending
cashbook-capital-warning-place = The partners' opening { -sarmaya } is not in { -rokar }

# review: pending
cashbook-capital-warning-old = Difference in the old books: <balance></balance>

# review: pending
cashbook-capital-warning-place-text = This money was written against the opening difference, so { -rokar } shows less than it really is. Use the button below to put it into { -rokar }.

# review: pending
cashbook-capital-warning-old-text = The opening amounts typed when the accounts were made do not match each other. Below are the accounts that make this difference.

# review: pending
cashbook-capital-warning-sum-partner = Partners' opening { -sarmaya } that is not in { -rokar }

# review: pending
cashbook-capital-warning-sum-party = Difference of the buyers' and sellers' opening balances

# review: pending
cashbook-capital-warning-sum-other = Other entries

# review: pending
cashbook-capital-warning-part-other = Entry that is not an opening

# review: pending
cashbook-capital-warning-part-other-entry = Other entry

# review: pending
cashbook-capital-col-date = Date

# review: pending
cashbook-capital-net-owe = Short

# review: pending
cashbook-capital-where-buyer-credits = Held for buyers who paid ahead

# review: pending
cashbook-capital-warning-parts = These accounts make the difference

# review: pending
cashbook-capital-warning-part-partner = Opening { -sarmaya }, not in { -rokar }

# review: pending
cashbook-capital-warning-part-party = { -ibtidai-baqaya }

# review: pending
cashbook-capital-col-account = Account

# review: pending
cashbook-capital-col-opening = Amount in the difference

# review: pending
cashbook-capital-warning-place-button = Put the { -sarmaya } into { -rokar }

## The box that asks for the reason and the Owner's sign before the opening capital is put into cash.

# review: pending
cashbook-capital-place-title = Put the { -sarmaya } into { -rokar }?

# review: pending
cashbook-capital-place-text = The opening { -sarmaya } of each partner is reversed on its own date and written again with { -rokar }. Closed days reopen and close again by themselves. { -rokar } goes up; the partners' { -kul-raqam } and the { -trial-balance } do not change. Write the reason and the Owner's { -dastakhat }.

# review: pending
cashbook-capital-place-label = Reason

# review: pending
cashbook-capital-place-sign-hint = Only the Owner's { -dastakhat } puts it in.

# review: pending
cashbook-capital-place-confirm = Put it in

# review: pending
cashbook-capital-placed-toast = The { -sarmaya } is now in { -rokar }
