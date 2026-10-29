### The party card and the account picker (plan 08, PRD FR-7.38 to FR-7.42). Every Urdu line says what its English line says (NFR-URD-29).

# review: pending
assist-card-loading = کارڈ کھل رہا ہے…
# review: pending
assist-card-failed = کارڈ نہیں کھل سکا
# review: pending
assist-card-balance = بقایا
# review: pending
assist-card-last-entry = آخری اندراج
# review: pending
assist-card-never = ابھی کوئی اندراج نہیں
# review: pending
assist-card-credit-limit = ادھار کی حد
# review: pending
assist-card-credit-used = استعمال شدہ
# review: pending
assist-card-advance = { -peshgi-khata } باقی
# review: pending
assist-warning-inactive = یہ کھاتہ غیر فعال ہے
# review: pending
assist-warning-over-credit-limit = ادھار کی حد پوری ہو چکی ہے
# review: pending
assist-warning-owes-firm = اس پارٹی کے ذمے فرم کی رقم ہے

## The account picker: codes that do not resolve, and the quick add.

# review: pending
accountpicker-unknown-seller = اس کوڈ کا کوئی { -beopari } نہیں
# review: pending
accountpicker-unknown-buyer = اس کوڈ کا کوئی { -gahak } نہیں
# review: pending
accountpicker-unknown-any = اس کوڈ کا کوئی کھاتہ نہیں
# review: pending
accountpicker-reserved = یہ کوڈ خاص کھاتوں کے لیے ہے۔ عام کھاتے کا یہ کوڈ نہیں ہو سکتا۔
# review: pending
accountpicker-inactive = { $name } ({ $code }) غیر فعال ہے۔ نئے اندراج میں اسے استعمال نہیں کیا جا سکتا۔
# review: pending
accountpicker-several = اس کوڈ کے ایک سے زیادہ کھاتے ہیں۔ ایک چنیں۔
# review: pending
accountpicker-add = نیا کھاتہ شامل کریں
# review: pending
accountpicker-add-title = نیا کھاتہ
# review: pending
accountpicker-add-help = صرف وہ باتیں لکھیں جن سے یہ کھاتہ پہچانا جائے۔ باقی تفصیل بعد میں { -index } سے بھر سکتے ہیں۔

## How was this worked out? (PRD FR-7.43). The backend names the message and gives its values.

# review: pending
calc-info-label = یہ کیسے نکالا گیا؟
# review: pending
calc-info-title = حساب
# review: pending
calc-info-unknown = اس رقم کا حساب یہاں نہیں دکھایا جا سکتا۔
# The level of the rule that was used, under the working (PRD FR-7.43).
# review: pending
calc-info-rule = (اصول کا درجہ: { $level })
# review: pending
working-percent = <num>{ $base }</num> × <num>{ $bps }</num> ÷ 100 = <num>{ $result }</num>
# review: pending
working-per-unit = <num>{ $qty }</num> × <num>{ $value }</num> = <num>{ $result }</num>
# A per-Nag charge of a lot is worked out on the quantity received.
# review: pending
working-per-unit-received = <num>{ $qty_received }</num> { -nag } وصول × <num>{ $value }</num> = <num>{ $result }</num>
# review: pending
working-per-lot = <num>{ $value }</num> فی { -lot } = <num>{ $result }</num>
# review: pending
working-manual = ہاتھ سے لکھی گئی رقم: <num>{ $result }</num>
# review: pending
working-none = کوئی اصول نہیں لگا۔ رقم { -lot } پر لکھی جاتی ہے۔

# A value typed on a sale line, and a per-lot charge that is not due on this line (PRD FR-7.43, FR-11.9).
# review: pending
working-manual-percent = ہاتھ سے لکھا گیا: <num>{ $base }</num> × <num>{ $bps }</num> ÷ 100 = <num>{ $result }</num>
# review: pending
working-manual-per-unit = ہاتھ سے لکھا گیا: <num>{ $qty }</num> × <num>{ $value }</num> = <num>{ $result }</num>
# review: pending
working-manual-per-unit-received = ہاتھ سے لکھا گیا: <num>{ $qty_received }</num> { -nag } وصول × <num>{ $value }</num> = <num>{ $result }</num>
# review: pending
working-manual-per-lot = ہاتھ سے لکھا گیا: <num>{ $value }</num> فی { -lot } = <num>{ $result }</num>
# review: pending
working-per-lot-already-charged = { -lot } کا یہ چارج پہلی { -bikri } میں لگ چکا ہے، اس لیے اب کچھ نہیں لگا۔

## How the amounts of the sale screen were worked out (plan 12, PRD 22). The backend names the message and gives its values.

# C1: the value of a line.
# review: pending
working-line-value = <num>{ $qty }</num> × <num>{ $rate }</num> = <num>{ $result }</num>
# C1 for an item sold by weight: the weight is in grams, the rate is for each kilo.
# review: pending
working-weight-value = <num>{ $grams }</num> گرام × <num>{ $rate }</num> فی کلو ÷ <num>1,000</num> = <num>{ $result }</num> (قریب ترین روپے تک)
# C5: what the buyer pays for a line.
# review: pending
working-buyer-total = قیمت <num>{ $value }</num> + { -commission } <num>{ $comm }</num> + { -laga } <num>{ $laga }</num> = <num>{ $result }</num>
# C19: the rate difference of a line.
# review: pending
working-rate-diff = { -gahak } کی قیمت <num>{ $buyer_value }</num> − { -beopari } کی رقم <num>{ $seller_amount }</num> = <num>{ $result }</num>
# A total of the lines.
# review: pending
working-sum = میزان ({ $count } لائن) = <num>{ $result }</num>
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
working-partner-share = { -hissa } <num>{ $bps }</num> فیصد از <num>{ $base }</num> = <num>{ $result }</num>
# C11: one partner's part of a shop expense (PRD 16.2, FR-16.8). $bps is a percent: 40 is 40%.
# The rupees left over go to the largest share, so the message does not claim an exact equation.
# review: pending
working-expense-share = { -dukan-kharcha } <num>{ $amount }</num>، { -hissa } <num>{ $bps }</num> فیصد (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے بڑے { -hissa } کو): <num>{ $result }</num>
# C22: the partner total before and after one entry of the { -zati-kharcha } screen (PRD FR-16.8).
# The same message is used for money taken and for money returned, so it does not say plus or minus.
# review: pending
working-partner-total = { -kul-raqam } پہلے <num>{ $before }</num>، یہ اندراج <num>{ $amount }</num>، بعد میں <num>{ $after }</num>
# C10, C11: one partner's part of the income and shop expense lines of a { -general } entry (PRD FR-6.5, 16.4).
# The part is the net of all such lines: expense is positive, income is negative. The rupees left over go to the largest share.
# review: pending
working-general-share = اس اندراج کی آمدن اور { -dukan-kharcha } کی لائنوں کا خالص، { -hissa } <num>{ $bps }</num> فیصد (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے بڑے { -hissa } کو): <num>{ $result }</num>
# A charge of a line that no rule applies to, for example { -laga } on a vegetable (PRD 11.3).
# review: pending
working-none-line = اس لائن پر کوئی اصول نہیں لگا، اس لیے رقم <num>{ $result }</num> ہے۔
# A one-time seller is paid his whole net at once while { -naqd } is empty (PRD FR-13.10).
# review: pending
working-cash-pukhta = { -pukhta-khata } کے بیوپاری کو اس کی پوری { -safi-bikri } اسی وقت دی جاتی ہے: <num>{ $result }</num>

## The workings of trip goods and trips (plan 16b, PRD C21, C23, C24, C25, FR-12.24, FR-12.26, FR-12.27).
## The backend names the message and gives its values.

# C21: the cost of the trip goods sold or written off, a part of the cost left (the cost of one
# { -nag } is the average of the purchase, PRD A45). The leftover rupees of the split go to the
# larger part, so the message does not claim an exact equation (money review of plan 16b).
# review: pending
working-cost-of-goods = باقی لاگت <num>{ $cost_left }</num>، <num>{ $qty_left }</num> { -nag } میں سے <num>{ $qty }</num> { -nag } کا حصہ (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے بڑے حصے کو): <num>{ $result }</num>
# C21 when the last { -nag } are taken: all the cost left goes with them.
# review: pending
working-cost-of-goods-last = آخری <num>{ $qty }</num> { -nag } کے ساتھ باقی ساری لاگت جاتی ہے: <num>{ $result }</num>
# FR-12.20: a line an edit left as it was keeps the cost it had.
# review: pending
working-cost-of-goods-kept = یہ لائن جیسی تھی ویسی رہی، اس لیے اس کی پرانی لاگت وہی رہتی ہے: <num>{ $result }</num>
# C23: a partner's part of a payment of a trip. $bps is in basis points: 4000 is 40%. The rupees
# left over go to the largest share, so the message does not claim an exact equation.
# review: pending
working-venture-part = ادائیگی <num>{ $amount }</num>، اس کا <num>{ $bps }</num> ÷ 10,000 (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے بڑے { -hissa } کو): <num>{ $result }</num>
# C25: a partner's part of money that comes in. $bps is in basis points: 4000 is 40%. The rupees
# left over go to the largest share, so the message does not claim an exact equation.
# review: pending
working-venture-credit = آنے والی رقم <num>{ $amount }</num>، اس کا <num>{ $bps }</num> ÷ 10,000 (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے بڑے { -hissa } کو): <num>{ $result }</num>
# PRD 16.7, C1: the value of the fruit taken (qty × rate).
# review: pending
working-fruit-value = <num>{ $qty }</num> × <num>{ $rate }</num> = <num>{ $result }</num>
# PRD 16.7, C1 for fruit of an item sold by weight: the weight is in grams, the rate is for each kilo.
# review: pending
working-fruit-value-weight = <num>{ $grams }</num> گرام × <num>{ $rate }</num> فی کلو ÷ <num>1,000</num> = <num>{ $result }</num> (قریب ترین روپے تک)
# PRD 16.7, C27: fruit for the shop's guests is shared equally between the partners who have a share
# on the day. Each part is rounded down and the rupees left over go to the partner made first, so
# $result is the first partner's part and the message does not claim an exact equation.
# review: pending
working-fruit-guests = قیمت <num>{ $value }</num> کو <num>{ $count }</num> { -shirakat-dar } میں برابر بانٹا (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے پہلے بنائے گئے { -shirakat-dar } کو): پہلا حصہ <num>{ $result }</num>
# C24: the part of a trip cost that one purchase carries, by the { -nag } bought. The rupees left
# over go to the larger part, so the message does not claim an exact equation.
# review: pending
working-trip-cost-spread = خرچ <num>{ $cost }</num>، <num>{ $total_qty }</num> { -nag } میں سے <num>{ $qty }</num> { -nag } کا حصہ (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے بڑے حصے کو): <num>{ $result }</num>
# FR-12.27: the result of a trip (sales money and money from other shops, less the cost sold and written off).
# review: pending
working-trip-result = فروخت کی رقم <num>{ $sales }</num> + دوسری دکانوں کی رقم <num>{ $other_shop }</num> − بکے ہوئے مال کی لاگت <num>{ $cost_sold }</num> − ضائع مال کی لاگت <num>{ $cost_written_off }</num> = <num>{ $result }</num>
# FR-12.27: what a partner has so far: money back less money in from the shop and from his pocket.
# review: pending
working-trip-partner-net = واپس ملا <num>{ $back }</num> − دکان سے لگایا <num>{ $in_shop }</num> − اپنی جیب سے لگایا <num>{ $in_pocket }</num> = <num>{ $result }</num>
# FR-12.37: the running balance of a garden: its money in less everything paid in (installments and
# costs, from the shop and from pockets). It starts in minus.
# review: pending
working-garden-balance = آنے والی رقم <num>{ $money_in }</num> − ادا کی گئی ہر رقم (قسطیں اور باغ کے خرچ، دکان سے اور اپنی جیب سے) <num>{ $paid }</num> = <num>{ $result }</num>
# FR-12.27: the result of one purchase.
# review: pending
working-purchase-result = فروخت کی رقم <num>{ $sales }</num> + دوسری دکانوں کی رقم <num>{ $other_shop }</num> − بکے ہوئے مال کی لاگت <num>{ $cost_sold }</num> − ضائع مال کی لاگت <num>{ $cost_written_off }</num> = <num>{ $result }</num>
# FR-12.32: the result of one sale in another shop.
# review: pending
working-other-sale-result = ملی ہوئی رقم <num>{ $amount }</num> − بکے ہوئے مال کی لاگت <num>{ $cost }</num> = <num>{ $result }</num>
# C21 for a whole lot of trip goods: the cost of the goods its lines took (the sum of the lines).
# review: pending
working-own-lot-cost = اس لاٹ کے بکے ہوئے <num>{ $qty }</num> { -nag } کی لاگت، جو خریداری سے لی گئی: <num>{ $result }</num>
# FR-12.27: the result of a lot of trip goods (the seller credit less the cost of the goods sold).
# review: pending
working-own-lot-result = بیوپاری کی جمع <num>{ $credit }</num> − بکے ہوئے مال کی لاگت <num>{ $cost }</num> = <num>{ $result }</num>
# FR-12.24: the total cost of a purchase: what was paid now and its part of the trip costs.
# review: pending
working-purchase-total-cost = ادا شدہ <num>{ $paid_now }</num> + { -market-safar } کے خرچ کا حصہ <num>{ $trip_cost }</num> = <num>{ $result }</num>
# PRD A45: the cost of one { -nag }, worked out for the whole purchase.
# review: pending
working-purchase-avg-cost = <num>{ $total }</num> ÷ <num>{ $qty }</num> = <num>{ $result }</num> (قریب ترین روپے تک)
# FR-12.24: what is paid now. Freight, labour and other cost are always paid at once.
# review: pending
working-purchase-paid-now = <num>{ $value }</num> + <num>{ $freight }</num> + <num>{ $labour }</num> + <num>{ $other_cost }</num> = <num>{ $result }</num>
# FR-12.41: a partner's part of a purchase entry with many lines. $lines: how many lines; $amount: his parts of all of them added together.
# review: pending
working-entry-part-sum = <num>{ $lines }</num> لائنوں میں سے ہر ایک میں اس کا حصہ، جمع کر کے: <num>{ $amount }</num>

## The attention list and the bell (plan 19, PRD FR-7.44). Every Urdu line says what its English line says (NFR-URD-29).

# review: pending
attention-bell-label = توجہ طلب: { $count }

# review: pending
attention-title = توجہ طلب

# review: pending
attention-empty-title = کسی کام پر توجہ کی ضرورت نہیں

# review: pending
attention-empty-help = جب کوئی کام باقی ہو گا تو یہاں نظر آئے گا۔

# review: pending
attention-load-failed = فہرست کھل نہیں سکی

# review: pending
attention-update = نیا ورژن <num>{ $version }</num> تیار ہے۔ دن ختم کر کے ترتیبات سے لگائیں۔

# review: pending
attention-open-first = سب سے ضروری: <name>{ $name }</name>

# review: pending
attention-drafts = { $count ->
    [one] <num>{ $count }</num> غیر محفوظ مسودہ
   *[other] <num>{ $count }</num> غیر محفوظ مسودے
}

# review: pending
attention-open-days = { $count ->
    [one] <num>{ $count }</num> پچھلا دن بند نہیں ہوا
   *[other] <num>{ $count }</num> پچھلے دن بند نہیں ہوئے
}

# review: pending
attention-stock-waiting = { -baqi-maal } میں <num>{ $count }</num> { -lot } انتظار میں ہیں، کل <num>{ $qty }</num> { -nag }

# review: pending
attention-stock-waiting-aged = { -baqi-maal } میں <num>{ $count }</num> { -lot } انتظار میں ہیں، کل <num>{ $qty }</num> { -nag }، سب سے پرانا <num>{ $days }</num> دن سے

# review: pending
attention-over-credit-limit = <num>{ $count }</num> { -gahak } ادھار کی حد سے آگے ہیں

# review: pending
attention-dues-old = <num>{ $count }</num> { -gahak } کے پرانے بقایا: <num>{ $amount }</num>

# review: pending
attention-dues-old-aged = <num>{ $count }</num> { -gahak } کے پرانے بقایا: <num>{ $amount }</num>، سب سے پرانا بل <num>{ $days }</num> دن پرانا ہے

# review: pending
attention-payables-old = <num>{ $count }</num> { -beopari } کی ادائیگی باقی ہے: <num>{ $amount }</num>

# review: pending
attention-payables-old-aged = <num>{ $count }</num> { -beopari } کی ادائیگی باقی ہے: <num>{ $amount }</num>، سب سے پرانی <num>{ $days }</num> دن سے

# review: pending
attention-backup-old = آخری کلاؤڈ بیک اپ <num>{ $days }</num> دن پرانا ہے

# review: pending
attention-backup-never = ابھی تک کلاؤڈ بیک اپ نہیں بنا

# review: pending
attention-loans-overdue = <num>{ $count }</num> { -adhar-raqam } کی واپسی کی تاریخ گزر چکی ہے: <num>{ $amount }</num>

# review: pending
attention-loans-overdue-aged = <num>{ $count }</num> { -adhar-raqam } کی واپسی کی تاریخ گزر چکی ہے، زیادہ سے زیادہ <num>{ $days }</num> دن: <num>{ $amount }</num>

# review: pending
attention-takes-overdue = شراکت داروں کی لی ہوئی <num>{ $count }</num> رقموں کی واپسی کی تاریخ گزر چکی ہے: <num>{ $amount }</num>

# review: pending
attention-takes-overdue-aged = شراکت داروں کی لی ہوئی <num>{ $count }</num> رقموں کی واپسی کی تاریخ گزر چکی ہے، زیادہ سے زیادہ <num>{ $days }</num> دن: <num>{ $amount }</num>

# review: pending
attention-names-missing = <num>{ $count }</num> ریکارڈ میں دو ناموں میں سے ایک ہی نام ہے
