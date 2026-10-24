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
cashbook-general-col-person = Person

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
cashbook-general-people-title = One-time people

# review: pending
cashbook-general-people-help = Money under a { -pukhta-khata } account is kept for each person. Choose the person for each of these lines.

# $line: the number of the line; $account: the name of its account.
# review: pending
cashbook-general-person-of = Person for line { $line }: { $account }

# review: pending
cashbook-type-capital-in = { -sarmaya } put in

# review: pending
cashbook-type-capital-out = { -sarmaya } taken back
