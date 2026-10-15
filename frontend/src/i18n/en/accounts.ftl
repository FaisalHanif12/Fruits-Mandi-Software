### The accounts and the chart of accounts (plan 07, PRD 10.1, 10.2). Every Urdu line says what its English line says (NFR-URD-29).

## The accounts: list, form, states (plan 07, PRD 10.1, 10.2, FR-10.5).

# review: pending
accounts-title = Accounts

# review: pending
accounts-help = Sellers, buyers, people who borrow money and the other accounts. Open an account to see or change it, or make a new one.

# review: pending
accounts-new = New account

# review: pending
accounts-new-title = New account

# review: pending
accounts-edit-title = Change { $name }

# review: pending
accounts-saved = Account saved

# review: pending
accounts-made-active = Account made active

# review: pending
accounts-made-inactive = Account made inactive

# review: pending
accounts-make-active = Make active

# review: pending
accounts-make-inactive = Make inactive

# A system account (cash, commission income and the rest, PRD FR-10.4) is used by the books.
# review: pending
accounts-system-stays-active = This is a system account. The books use it, so it cannot be made inactive.

# review: pending
accounts-inactive-title = Make this inactive?

# review: pending
accounts-inactive-description = { $name } will not be offered in new entries, but its old figures and the reports stay as they are. You can make it active again at any time.

# review: pending
accounts-inactive-note = This account is inactive, so it is not offered in new entries. Make it active to use it again.

# review: pending
accounts-status-inactive = Inactive

# review: pending
accounts-load-failed = The accounts could not be opened

# review: pending
accounts-empty = No accounts yet

# review: pending
accounts-empty-help = Press { $key } to make the first account.

# review: pending
accounts-no-match-title = No account found

# review: pending
accounts-no-match-help = Try another code or name, or show the inactive accounts too.

# review: pending
accounts-no-match-clear = Clear the search

# review: pending
accounts-not-found-title = This account was not found

# review: pending
accounts-not-found-help = It may have been changed. Open the list of accounts again.

# review: pending
accounts-filter-type = Type

# review: pending
accounts-filter-label = Search this list

# review: pending
accounts-show-inactive = Show the inactive ones too

## Types and weekdays.

# review: pending
accounts-type-all = All

# review: pending
accounts-type-seller = { -beopari }

# review: pending
accounts-type-buyer = { -gahak }

# review: pending
accounts-type-money_lent = { -adhar-raqam }

# review: pending
accounts-type-other = Other

# review: pending
accounts-weekday-sat = Saturday

# review: pending
accounts-weekday-sun = Sunday

# review: pending
accounts-weekday-mon = Monday

# review: pending
accounts-weekday-tue = Tuesday

# review: pending
accounts-weekday-wed = Wednesday

# review: pending
accounts-weekday-thu = Thursday

# review: pending
accounts-weekday-fri = Friday

## List columns and totals (PRD FR-10.5).

# review: pending
accounts-col-code = Code

# review: pending
accounts-col-name = Name

# review: pending
accounts-col-other-name = Urdu name

# review: pending
accounts-col-city = City

# review: pending
accounts-col-opening-date = Opening date

# review: pending
accounts-col-opening-dr = Opening { -banaam }

# review: pending
accounts-col-opening-cr = Opening { -jama }

# review: pending
accounts-col-credit = Credit allowed

# review: pending
accounts-col-balance = Current balance

# review: pending
accounts-col-side = Side

# review: pending
accounts-total-dr = Total opening { -banaam }

# review: pending
accounts-total-cr = Total opening { -jama }

## The form: sections and fields (PRD 10.2).

# review: pending
accounts-section-account = Account

# review: pending
accounts-section-names = Names

# review: pending
accounts-section-contact = Contact

# review: pending
accounts-section-opening = Opening balance

# review: pending
accounts-section-credit = { -udhari } and { -agrahi }

# review: pending
accounts-section-more = More

# review: pending
accounts-field-type = Account type

# review: pending
accounts-field-control = Control

# review: pending
accounts-field-code = Code

# review: pending
accounts-field-code-hint = The next code is already filled in. You can change it.

# review: pending
accounts-field-name-ur = Urdu name

# review: pending
accounts-field-name-en = English name

# review: pending
accounts-field-father-ur = Father or firm name, Urdu

# review: pending
accounts-field-father-en = Father or firm name, English

# review: pending
accounts-field-city = City

# review: pending
accounts-field-adda = { -adda }

# review: pending
accounts-field-phone1 = Phone number

# review: pending
accounts-field-phone2 = Second phone number

# review: pending
accounts-field-address = Address

# review: pending
accounts-field-opening-date = Opening date

# review: pending
accounts-field-opening-dr = Opening { -banaam }

# review: pending
accounts-field-opening-cr = Opening { -jama }

# review: pending
accounts-field-opening-advance = This opening amount is an { -peshgi-khata }

# review: pending
accounts-field-credit-allowed = Credit is allowed

# review: pending
accounts-field-credit-limit = Credit limit

# review: pending
accounts-field-collector = Collector

# review: pending
accounts-field-weekday = Collection day

# review: pending
accounts-field-linked = Linked account

# review: pending
accounts-field-linked-hint = If one person is both a seller and a buyer, link his two accounts.

# review: pending
accounts-field-marka = Default { -marka }

# review: pending
accounts-field-note = Note

# review: pending
accounts-similar-title = Similar accounts already exist

## The chart of accounts (PRD FR-10.1).

# review: pending
accounts-tree-label = Chart of accounts tree

# review: pending
chart-title = Chart of accounts

# review: pending
chart-help = Heads, controls and accounts, each with its current balance. Open and close them with the arrow keys.

# review: pending
chart-tabs = Parts of the chart

# review: pending
chart-tab-tree = Tree

# review: pending
chart-tab-heads = Heads

# review: pending
chart-tab-controls = Controls

# review: pending
chart-load-failed = The chart of accounts could not be opened

# review: pending
masters-accounts-title = Accounts

# review: pending
masters-accounts-help = Sellers, buyers and the other accounts.

# review: pending
masters-chart-title = Chart of accounts

# review: pending
masters-chart-help = Heads, controls and accounts as a tree.

## The opening balances grid (plan 07, PRD FR-10.9).

# review: pending
openings-open = Write opening balances

# review: pending
openings-title = Opening balances

# review: pending
openings-help = Write the opening Debit or Credit against each account. Leave the accounts that have none empty.

# review: pending
openings-grid-label = Opening balances list

# review: pending
openings-date-all = Date for new opening balances

# review: pending
openings-date-all-hint = Accounts with no opening balance yet get this date.

# review: pending
openings-summary-title = Total of the lines you changed

# review: pending
openings-changed = { $count ->
    [one] One account changed
   *[other] { $count } accounts changed
}

# review: pending
openings-none-changed = Nothing changed yet

# Opening Difference is the name of the account 06019999.
# review: pending
openings-difference = Opening Difference

# review: pending
openings-difference-help = It is zero when all the opening balances are right.

# review: pending
openings-saved = Opening balances saved

# review: pending
openings-load-failed = The opening balances could not be opened

# review: pending
openings-empty = No accounts yet

# review: pending
openings-empty-help = Make an account in { -index } first, then write its opening balance.

# review: pending
openings-errors-title = These lines have a problem

# One problem of the openings grid: the account, the column and what is wrong.
# review: pending
openings-problem = { $account } ({ $field }): { $message }

# review: pending
openings-locked-help = An account whose cells are locked cannot have an opening balance.

# The name of a seller whose opening debit is an advance, in the opening balances grid.
# review: pending
openings-name-advance = { $name } ({ -peshgi-khata })
