### The party card and the account picker (plan 08, PRD FR-7.38 to FR-7.42). Every Urdu line says what its English line says (NFR-URD-29).

# review: pending
assist-card-loading = Opening the card…
# review: pending
assist-card-failed = The card could not be opened
# review: pending
assist-card-balance = Balance
# review: pending
assist-card-last-entry = Last entry
# review: pending
assist-card-never = No entry yet
# review: pending
assist-card-credit-limit = Credit limit
# review: pending
assist-card-credit-used = Used
# review: pending
assist-card-advance = { -peshgi-khata } open
# review: pending
assist-warning-inactive = This account is inactive
# review: pending
assist-warning-over-credit-limit = The credit limit is used up
# review: pending
assist-warning-owes-firm = This party owes the firm

## The account picker: codes that do not resolve, and the quick add.

# review: pending
accountpicker-unknown-seller = There is no { -beopari } with this code
# review: pending
accountpicker-unknown-buyer = There is no { -gahak } with this code
# review: pending
accountpicker-unknown-any = There is no account with this code
# review: pending
accountpicker-reserved = This code is kept for system accounts. An ordinary account cannot have it.
# review: pending
accountpicker-inactive = { $name } ({ $code }) is inactive. It cannot be used in a new entry.
# review: pending
accountpicker-several = More than one account has this code. Pick one.
# review: pending
accountpicker-add = Add a new account
# review: pending
accountpicker-add-title = New account
# review: pending
accountpicker-add-help = Write only what tells this account apart. You can fill in the rest later from the { -index }.

## How was this worked out? (PRD FR-7.43). The backend names the message and gives its values.

# review: pending
calc-info-label = How was this worked out?
# review: pending
calc-info-title = Working
# review: pending
calc-info-unknown = The working of this amount cannot be shown here.
# The level of the rule that was used, under the working (PRD FR-7.43).
# review: pending
calc-info-rule = (Rule level: { $level })
# review: pending
working-percent = <num>{ $base }</num> × <num>{ $bps }</num> ÷ 100 = <num>{ $result }</num>
# review: pending
working-per-unit = <num>{ $qty }</num> × <num>{ $value }</num> = <num>{ $result }</num>
# A per-Nag charge of a lot is worked out on the quantity received.
# review: pending
working-per-unit-received = <num>{ $qty_received }</num> { -nag } received × <num>{ $value }</num> = <num>{ $result }</num>
# review: pending
working-per-lot = <num>{ $value }</num> per { -lot } = <num>{ $result }</num>
# review: pending
working-manual = Amount typed by hand: <num>{ $result }</num>
# review: pending
working-none = No rule applies. The amount is typed on the { -lot }.

# A value typed on a sale line, and a per-lot charge that is not due on this line (PRD FR-7.43, FR-11.9).
# review: pending
working-manual-percent = Typed by hand: <num>{ $base }</num> × <num>{ $bps }</num> ÷ 100 = <num>{ $result }</num>
# review: pending
working-manual-per-unit = Typed by hand: <num>{ $qty }</num> × <num>{ $value }</num> = <num>{ $result }</num>
# review: pending
working-manual-per-unit-received = Typed by hand: <num>{ $qty_received }</num> { -nag } received × <num>{ $value }</num> = <num>{ $result }</num>
# review: pending
working-manual-per-lot = Typed by hand: <num>{ $value }</num> per { -lot } = <num>{ $result }</num>
# review: pending
working-per-lot-already-charged = This charge is made once per { -lot } and was made in the first { -bikri }, so nothing is added now.
