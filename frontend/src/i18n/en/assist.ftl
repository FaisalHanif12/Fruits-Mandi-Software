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

## How the amounts of the sale screen were worked out (plan 12, PRD 22). The backend names the message and gives its values.

# C1: the value of a line.
# review: pending
working-line-value = <num>{ $qty }</num> × <num>{ $rate }</num> = <num>{ $result }</num>
# C1 for an item sold by weight: the weight is in grams, the rate is for each kilo.
# review: pending
working-weight-value = <num>{ $grams }</num> g × <num>{ $rate }</num> per kg ÷ <num>1,000</num> = <num>{ $result }</num> (to the nearest rupee)
# C5: what the buyer pays for a line.
# review: pending
working-buyer-total = Value <num>{ $value }</num> + { -commission } <num>{ $comm }</num> + { -laga } <num>{ $laga }</num> = <num>{ $result }</num>
# C19: the rate difference of a line.
# review: pending
working-rate-diff = Buyer value <num>{ $buyer_value }</num> − seller amount <num>{ $seller_amount }</num> = <num>{ $result }</num>
# A total of the lines.
# review: pending
working-sum = Total ({ $count } line(s)) = <num>{ $result }</num>
# C6: the deductions of a lot.
# review: pending
working-deductions = <num>{ $freight }</num> + <num>{ $comm }</num> + <num>{ $labour }</num> + <num>{ $munshiana }</num> + <num>{ $cash }</num> + <num>{ $store_rent }</num> + <num>{ $phone }</num> = <num>{ $result }</num>
# C7: the net of a lot.
# review: pending
working-net = <num>{ $gross }</num> − <num>{ $deductions }</num> = <num>{ $result }</num>
# C8: what the posting credits the seller.
# review: pending
working-seller-credit = <num>{ $gross }</num> − <num>{ $comm }</num> − <num>{ $freight }</num> − <num>{ $labour }</num> − <num>{ $munshiana }</num> − <num>{ $store_rent }</num> − <num>{ $phone }</num> = <num>{ $result }</num>
# C9: the income shared between the partners.
# review: pending
working-shared-income = <num>{ $seller_comm }</num> + <num>{ $buyer_comm }</num> + <num>{ $laga }</num> + <num>{ $phone }</num> + <num>{ $rate_diff }</num> = <num>{ $result }</num>
# C10: the share of one partner.
# The largest share also gets the rupees left over, so this is not base × percent ÷ 100.
# review: pending
working-partner-share = { -hissa } <num>{ $bps }</num>% of <num>{ $base }</num> = <num>{ $result }</num>
# A charge of a line that no rule applies to, for example { -laga } on a vegetable (PRD 11.3).
# review: pending
working-none-line = No rule applies to this line, so the amount is <num>{ $result }</num>.
# A one-time seller is paid his whole net at once while { -naqd } is empty (PRD FR-13.10).
# review: pending
working-cash-pukhta = A { -pukhta-khata } seller is paid his whole { -safi-bikri } on the spot: <num>{ $result }</num>.
