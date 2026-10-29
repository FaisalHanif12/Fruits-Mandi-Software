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
# C11: one partner's part of a shop expense (PRD 16.2, FR-16.8). $bps is a percent: 40 is 40%.
# The rupees left over go to the largest share, so the message does not claim an exact equation.
# review: pending
working-expense-share = { -dukan-kharcha } <num>{ $amount }</num>, { -hissa } <num>{ $bps }</num>% (each part is rounded down and the rupees left over go to the largest { -hissa }): <num>{ $result }</num>
# C22: the partner total before and after one entry of the { -zati-kharcha } screen (PRD FR-16.8).
# The same message is used for money taken and for money returned, so it does not say plus or minus.
# review: pending
working-partner-total = { -kul-raqam } before <num>{ $before }</num>, this entry <num>{ $amount }</num>, after <num>{ $after }</num>
# C10, C11: one partner's part of the income and shop expense lines of a { -general } entry (PRD FR-6.5, 16.4).
# The part is the net of all such lines: expense is positive, income is negative. The rupees left over go to the largest share.
# review: pending
working-general-share = Net of the income and { -dukan-kharcha } lines of this entry, { -hissa } <num>{ $bps }</num>% (each part is rounded down and the rupees left over go to the largest { -hissa }): <num>{ $result }</num>
# A charge of a line that no rule applies to, for example { -laga } on a vegetable (PRD 11.3).
# review: pending
working-none-line = No rule applies to this line, so the amount is <num>{ $result }</num>.
# A one-time seller is paid his whole net at once while { -naqd } is empty (PRD FR-13.10).
# review: pending
working-cash-pukhta = A { -pukhta-khata } seller is paid his whole { -safi-bikri } on the spot: <num>{ $result }</num>.

## The workings of trip goods and trips (plan 16b, PRD C21, C23, C24, C25, FR-12.24, FR-12.26, FR-12.27).
## The backend names the message and gives its values.

# C21: the cost of the trip goods sold or written off, a part of the cost left (the cost of one
# { -nag } is the average of the purchase, PRD A45). The leftover rupees of the split go to the
# larger part, so the message does not claim an exact equation (money review of plan 16b).
# review: pending
working-cost-of-goods = Cost left <num>{ $cost_left }</num>, the part for <num>{ $qty }</num> of <num>{ $qty_left }</num> { -nag } (each part is rounded down and the rupees left over go to the larger part): <num>{ $result }</num>
# C21 when the last { -nag } are taken: all the cost left goes with them.
# review: pending
working-cost-of-goods-last = The last <num>{ $qty }</num> { -nag } take all the cost left: <num>{ $result }</num>
# FR-12.20: a line an edit left as it was keeps the cost it had.
# review: pending
working-cost-of-goods-kept = This line was left as it was, so it keeps the cost it had: <num>{ $result }</num>
# C23: a partner's part of a payment of a trip. $bps is in basis points: 4000 is 40%. The rupees
# left over go to the largest share, so the message does not claim an exact equation.
# review: pending
working-venture-part = Payment <num>{ $amount }</num>, <num>{ $bps }</num> ÷ 10,000 of it (each part is rounded down and the rupees left over go to the largest { -hissa }): <num>{ $result }</num>
# C25: a partner's part of money that comes in. $bps is in basis points: 4000 is 40%. The rupees
# left over go to the largest share, so the message does not claim an exact equation.
# review: pending
working-venture-credit = Money in <num>{ $amount }</num>, <num>{ $bps }</num> ÷ 10,000 of it (each part is rounded down and the rupees left over go to the largest { -hissa }): <num>{ $result }</num>
# PRD 16.7, C1: the value of the fruit taken (qty × rate).
# review: pending
working-fruit-value = <num>{ $qty }</num> × <num>{ $rate }</num> = <num>{ $result }</num>
# PRD 16.7, C1 for fruit of an item sold by weight: the weight is in grams, the rate is for each kilo.
# review: pending
working-fruit-value-weight = <num>{ $grams }</num> g × <num>{ $rate }</num> per kg ÷ <num>1,000</num> = <num>{ $result }</num> (to the nearest rupee)
# PRD 16.7, C27: fruit for the shop's guests is shared equally between the partners who have a share
# on the day. Each part is rounded down and the rupees left over go to the partner made first, so
# $result is the first partner's part and the message does not claim an exact equation.
# review: pending
working-fruit-guests = Value <num>{ $value }</num> shared equally between <num>{ $count }</num> partners (each part is rounded down and the rupees left over go to the partner made first): the first part <num>{ $result }</num>
# C24: the part of a trip cost that one purchase carries, by the { -nag } bought. The rupees left
# over go to the larger part, so the message does not claim an exact equation.
# review: pending
working-trip-cost-spread = Cost <num>{ $cost }</num>, the part for <num>{ $qty }</num> of <num>{ $total_qty }</num> { -nag } (each part is rounded down and the rupees left over go to the larger part): <num>{ $result }</num>
# FR-12.27: the result of a trip (sales money and money from other shops, less the cost sold and written off).
# review: pending
working-trip-result = Sales money <num>{ $sales }</num> + money from other shops <num>{ $other_shop }</num> − cost of goods sold <num>{ $cost_sold }</num> − cost written off <num>{ $cost_written_off }</num> = <num>{ $result }</num>
# FR-12.27: what a partner has so far: money back less money in from the shop and from his pocket.
# review: pending
working-trip-partner-net = Money back <num>{ $back }</num> − paid from the shop <num>{ $in_shop }</num> − paid from his pocket <num>{ $in_pocket }</num> = <num>{ $result }</num>
# FR-12.37: the running balance of a garden: its money in less everything paid in (installments and
# costs, from the shop and from pockets). It starts in minus.
# review: pending
working-garden-balance = Money in <num>{ $money_in }</num> − everything paid (installments and costs, from the shop and from pockets) <num>{ $paid }</num> = <num>{ $result }</num>
# FR-12.27: the result of one purchase.
# review: pending
working-purchase-result = Sales money <num>{ $sales }</num> + money from other shops <num>{ $other_shop }</num> − cost of goods sold <num>{ $cost_sold }</num> − cost written off <num>{ $cost_written_off }</num> = <num>{ $result }</num>
# FR-12.32: the result of one sale in another shop.
# review: pending
working-other-sale-result = Money received <num>{ $amount }</num> − cost of goods sold <num>{ $cost }</num> = <num>{ $result }</num>
# C21 for a whole lot of trip goods: the cost of the goods its lines took (the sum of the lines).
# review: pending
working-own-lot-cost = Cost of the <num>{ $qty }</num> { -nag } sold in this lot, taken from the purchase: <num>{ $result }</num>
# FR-12.27: the result of a lot of trip goods (the seller credit less the cost of the goods sold).
# review: pending
working-own-lot-result = Seller credit <num>{ $credit }</num> − cost of goods sold <num>{ $cost }</num> = <num>{ $result }</num>
# FR-12.24: the total cost of a purchase: what was paid now and its part of the trip costs.
# review: pending
working-purchase-total-cost = Paid now <num>{ $paid_now }</num> + part of the { -market-safar } costs <num>{ $trip_cost }</num> = <num>{ $result }</num>
# PRD A45: the cost of one { -nag }, worked out for the whole purchase.
# review: pending
working-purchase-avg-cost = <num>{ $total }</num> ÷ <num>{ $qty }</num> = <num>{ $result }</num> (to the nearest rupee)
# FR-12.24: what is paid now. Freight, labour and other cost are always paid at once.
# review: pending
working-purchase-paid-now = <num>{ $value }</num> + <num>{ $freight }</num> + <num>{ $labour }</num> + <num>{ $other_cost }</num> = <num>{ $result }</num>
# FR-12.41: a partner's part of a purchase entry with many lines. $lines: how many lines; $amount: his parts of all of them added together.
# review: pending
working-entry-part-sum = His part of each of the <num>{ $lines }</num> lines, added together: <num>{ $amount }</num>

## The attention list and the bell (plan 19, PRD FR-7.44). Every Urdu line says what its English line says (NFR-URD-29).

# review: pending
attention-bell-label = Needs attention: { $count }

# review: pending
attention-title = Needs your attention

# review: pending
attention-empty-title = Nothing needs your attention

# review: pending
attention-empty-help = When something needs doing, it shows here.

# review: pending
attention-load-failed = The list could not be opened

# review: pending
attention-update = A new version <num>{ $version }</num> is ready. Install it from Settings when the day is done.

# review: pending
attention-open-first = Most urgent: <name>{ $name }</name>

# review: pending
attention-drafts = { $count ->
    [one] <num>{ $count }</num> unsaved draft
   *[other] <num>{ $count }</num> unsaved drafts
}

# review: pending
attention-open-days = { $count ->
    [one] <num>{ $count }</num> earlier day is not closed
   *[other] <num>{ $count }</num> earlier days are not closed
}

# review: pending
attention-stock-waiting = { $count ->
    [one] <num>{ $count }</num> { -lot } is waiting in { -baqi-maal }, <num>{ $qty }</num> { -nag } in all
   *[other] <num>{ $count }</num> { -lot }s are waiting in { -baqi-maal }, <num>{ $qty }</num> { -nag } in all
}

# review: pending
attention-stock-waiting-aged = { $count ->
    [one] <num>{ $count }</num> { -lot } is waiting in { -baqi-maal }, <num>{ $qty }</num> { -nag } in all, the oldest for <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        }
   *[other] <num>{ $count }</num> { -lot }s are waiting in { -baqi-maal }, <num>{ $qty }</num> { -nag } in all, the oldest for <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        }
}

# review: pending
attention-over-credit-limit = { $count ->
    [one] <num>{ $count }</num> { -gahak } is over the credit limit
   *[other] <num>{ $count }</num> { -gahak }s are over the credit limit
}

# review: pending
attention-dues-old = { $count ->
    [one] <num>{ $count }</num> { -gahak } has an old due: <num>{ $amount }</num>
   *[other] <num>{ $count }</num> { -gahak }s have old dues: <num>{ $amount }</num>
}

# review: pending
attention-dues-old-aged = { $count ->
    [one] <num>{ $count }</num> { -gahak } has an old due: <num>{ $amount }</num>, the bill is <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        } old
   *[other] <num>{ $count }</num> { -gahak }s have old dues: <num>{ $amount }</num>, the oldest bill is <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        } old
}

# review: pending
attention-payables-old = { $count ->
    [one] <num>{ $count }</num> { -beopari } has waited for payment: <num>{ $amount }</num>
   *[other] <num>{ $count }</num> { -beopari }s have waited for payment: <num>{ $amount }</num>
}

# review: pending
attention-payables-old-aged = { $count ->
    [one] <num>{ $count }</num> { -beopari } has waited for payment: <num>{ $amount }</num>, for <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        }
   *[other] <num>{ $count }</num> { -beopari }s have waited for payment: <num>{ $amount }</num>, the longest for <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        }
}

# review: pending
attention-backup-old = The last cloud backup is <num>{ $days }</num> { $days ->
        [one] day
       *[other] days
    } old

# review: pending
attention-backup-never = No cloud backup has been made yet

# review: pending
attention-loans-overdue = { $count ->
    [one] <num>{ $count }</num> { -adhar-raqam } entry is past its return-by date: <num>{ $amount }</num>
   *[other] <num>{ $count }</num> { -adhar-raqam } entries are past their return-by date: <num>{ $amount }</num>
}

# review: pending
attention-loans-overdue-aged = { $count ->
    [one] <num>{ $count }</num> { -adhar-raqam } entry is <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        } past its return-by date: <num>{ $amount }</num>
   *[other] <num>{ $count }</num> { -adhar-raqam } entries are past their return-by date, up to <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        }: <num>{ $amount }</num>
}

# review: pending
attention-takes-overdue = { $count ->
    [one] <num>{ $count }</num> amount taken by a partner is past its return-by date: <num>{ $amount }</num>
   *[other] <num>{ $count }</num> amounts taken by partners are past their return-by date: <num>{ $amount }</num>
}

# review: pending
attention-takes-overdue-aged = { $count ->
    [one] <num>{ $count }</num> amount taken by a partner is <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        } past its return-by date: <num>{ $amount }</num>
   *[other] <num>{ $count }</num> amounts taken by partners are past their return-by date, up to <num>{ $days }</num> { $days ->
            [one] day
           *[other] days
        }: <num>{ $amount }</num>
}

# review: pending
attention-names-missing = { $count ->
    [one] <num>{ $count }</num> record has only one of its two names
   *[other] <num>{ $count }</num> records have only one of their two names
}
