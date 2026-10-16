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
# A charge of a line that no rule applies to, for example { -laga } on a vegetable (PRD 11.3).
# review: pending
working-none-line = اس لائن پر کوئی اصول نہیں لگا، اس لیے رقم <num>{ $result }</num> ہے۔
# A one-time seller is paid his whole net at once while { -naqd } is empty (PRD FR-13.10).
# review: pending
working-cash-pukhta = { -pukhta-khata } کے بیوپاری کو اس کی پوری { -safi-bikri } اسی وقت دی جاتی ہے: <num>{ $result }</num>
