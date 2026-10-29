# The purchases of a trip (plan 16b, PRD 12.10, FR-12.24 to FR-12.28). They are made on the trip
# screen; these are the words of their fields, cards and boxes.
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The number of a purchase: its own series.

# review: pending
purchases-number = خ-{ $number }

# $number: the number of the purchase in its series.
# review: pending
purchases-cancelled = { -khareedari } { $number } منسوخ ہو گئی

# Who the goods were bought from, when no supplier is named.
# review: pending
purchases-market = منڈی سے خریدا

## The fields.

# review: pending
purchases-field-date = تاریخ

# review: pending
purchases-field-supplier = کس سے خریدا

# review: pending
purchases-supplier-hint = جیسا آپ لکھنا چاہیں۔ اسے کچھ دینا نہیں ہوتا: خریداری اسی وقت ادا ہوتی ہے۔

# review: pending
purchases-field-qty = خریدے ہوئے { -nag }

# review: pending
purchases-qty-hint = آپ جمع بھی لکھ سکتے ہیں، مثلاً 20+25۔

# review: pending
purchases-field-weight = وزن (کلو)

# review: pending
purchases-field-rate = ریٹ

# review: pending
purchases-rate-hint = ہر { -nag } کا، یا وزن سے بکنے والی { -qism-maal } کے لیے ہر کلو کا۔

# review: pending
purchases-field-other-cost = دیگر خرچ

# PRD FR-12.39. The box shows the day of the purchase until it is changed.
# review: pending
purchases-sell-date-hint = شروع میں خریداری کا دن ہوتا ہے۔ اس دن سے مال ہر نئی { -bikri } میں آتا رہتا ہے جب تک بک نہ جائے۔

# review: pending
purchases-field-note = نوٹ

## What the purchase costs, live under the form.

# review: pending
purchases-numbers-title = لاگت

# review: pending
purchases-numbers-empty = فارم بھریں تو لاگت یہاں نظر آئے گی۔

# review: pending
purchases-numbers-incomplete = لاگت دیکھنے کے لیے { -qism-maal }، { -packing }، مقدار اور ریٹ لکھیں۔

# review: pending
purchases-line-value = مال کی قیمت

# review: pending
purchases-line-total-cost = کل لاگت

# The cost of one { -nag }, worked out for the whole purchase (PRD A45).
# review: pending
purchases-line-avg-cost = ایک { -nag } کی لاگت

# Paid at once: the value, the freight, the labour and the other cost.
# review: pending
purchases-line-paid-now = ابھی ادا شدہ

## The columns of the list of a trip's purchases.

# review: pending
purchases-col-number = نمبر

# review: pending
purchases-col-supplier = کس سے خریدا

# review: pending
purchases-col-item = { -qism-maal } اور { -packing }

# review: pending
purchases-col-left = باقی

# review: pending
purchases-col-cost-left = باقی کی لاگت

# review: pending
purchases-col-date = تاریخ
# review: pending
purchases-col-sell-date = { -sell-date }

# review: pending
purchases-col-qty = خریدے

# review: pending
purchases-col-total-cost = کل لاگت

# review: pending
purchases-col-result = منافع یا نقصان

# review: pending
purchases-item-packing = { $item }، { $packing }

# A cancelled purchase is told in words, not only dimmed (NFR-USE-12).
# review: pending
purchases-item-cancelled = { $item } (منسوخ)

## The card of the purchase that has focus.

# review: pending
purchases-detail-title = { $number }، { $item }

# review: pending
purchases-detail-failed = خریداری نہیں کھل سکی

# review: pending
purchases-detail-cancelled = یہ خریداری منسوخ ہے۔

# review: pending
purchases-detail-cancel-reason = وجہ: { $reason }

# review: pending
purchases-detail-written-off = ضائع

# review: pending
purchases-detail-days = انتظار کے دن

# The part of the trip costs (the truck, the food) that this purchase carries (PRD C24).
# review: pending
purchases-detail-trip-cost = سفر کے خرچ کا حصہ

# review: pending
purchases-detail-parts-title = ہر شراکت دار نے کیا دیا

# PRD FR-12.27: sales money + other shops − cost sold − cost written off.
# review: pending
purchases-result-title = اس خریداری کا منافع یا نقصان

# review: pending
purchases-detail-sales = ہماری دکان میں فروخت کی رقم

# review: pending
purchases-detail-other-shop = دوسری دکانوں سے ملی رقم

# review: pending
purchases-detail-cost-sold = بکے ہوئے مال کی لاگت

# review: pending
purchases-detail-cost-written-off = ضائع مال کی لاگت

# review: pending
purchases-loss-note = اس میں نقصان ہو رہا ہے۔

# review: pending
purchases-lots-title = ان { -bikri } میں بکا

# review: pending
purchases-lot-bikri = { -bikri }

# $number: the number of the { -bikri }; $lot: the number of the lot.
# review: pending
purchases-lot-open = { -bikri } { $number }، { -lot } { $lot }

# review: pending
purchases-lot-qty-sold = بکے

# The seller credit of the lot, which goes to the trip's partners (PRD C25).
# review: pending
purchases-lot-sales = فروخت کی رقم

# review: pending
purchases-write-offs-title = ضائع

# review: pending
purchases-write-off = ضائع کریں

# review: pending
purchases-undo-write-off = ضائع واپس لیں

# review: pending
purchases-cancel = خریداری منسوخ کریں

## Writing trip goods off (PRD FR-12.28). Nothing is posted.

# review: pending
purchases-write-off-title = سفر کا مال ضائع کریں

# $number: the purchase; $qty_left: how many { -nag } are left.
# review: pending
purchases-write-off-text = { $number } ({ $item }، { $packing }) کا خراب یا گم مال ذخیرے سے نکال دیا جائے گا۔ ابھی { $qty_left } { -nag } باقی ہیں۔ کچھ پوسٹ نہیں ہوگا، کیونکہ شراکت داروں نے مال کی رقم پہلے ہی دے دی ہے۔ ضائع مال کی لاگت سفر کے نتیجے میں نقصان بن کر نظر آئے گی۔

# review: pending
purchases-write-off-qty = ضائع کرنے والے { -nag }

# review: pending
purchases-write-off-reason = وجہ (مثلاً خراب)

# review: pending
purchases-write-off-confirm = ضائع کریں

# $qty: how many { -nag }; $cost: the cost that shows as a loss.
# review: pending
purchases-write-off-done = { $qty } { -nag } ضائع ہو گئے۔ نتیجے میں نقصان: <num>{ $cost }</num>

# review: pending
purchases-write-off-done-closed = { $qty } { -nag } ضائع ہو گئے۔ نتیجے میں نقصان: <num>{ $cost }</num>۔ اب کچھ باقی نہیں، اس لیے سفر بند ہو گیا۔

# review: pending
purchases-write-off-undo-title = ضائع واپس لیں

# review: pending
purchases-write-off-undo-text = مال واپس ذخیرے میں چلا جائے گا اور نقصان ختم ہو جائے گا۔ ضائع منتخب کریں اور وجہ لکھیں۔

# review: pending
purchases-write-off-undo-which = ضائع

# review: pending
purchases-write-off-undo-option = { $date }: { $qty } { -nag } ({ $reason })

# review: pending
purchases-write-off-undo-confirm = ضائع واپس لیں

# review: pending
purchases-write-off-undone = ضائع واپس لے لیا گیا

# review: pending
purchases-write-off-undone-reopened = ضائع واپس لے لیا گیا۔ سفر دوبارہ کھل گیا ہے۔

## Cancelling a purchase (PRD FR-12.24).

# review: pending
purchases-cancel-title = یہ خریداری منسوخ کریں؟

# $cost: what was paid now.
# review: pending
purchases-cancel-text = { $number } ({ $qty } { -nag } { $item }، { $packing }، ادا شدہ <num>{ $cost }</num>، { $supplier } سے) منسوخ ہو جائے گی۔ شراکت داروں نے دکان کی رقم سے جو دیا تھا وہ واپس آ جائے گا اور مال ذخیرے سے نکل جائے گا۔

# review: pending
purchases-cancel-reason = وجہ

# review: pending
purchases-cancel-confirm = خریداری منسوخ کریں

# review: pending
purchases-cancel-confirm-cash = ہاں، منسوخ کریں

# A purchase cannot be cancelled once any of it was sold or written off (PRD FR-12.24).
# review: pending
purchases-cancel-blocked = یہ خریداری منسوخ نہیں ہو سکتی، کیونکہ { $sold } { -nag } بک چکے ہیں اور { $written_off } { -nag } ضائع ہو چکے ہیں۔ پہلے { -bikri } منسوخ کریں یا ضائع واپس لیں۔

# A purchase that carries a part of a trip cost cannot be cancelled either (PRD FR-12.31).
# review: pending
purchases-cancel-blocked-cost = اس خریداری میں سفر کے خرچ کا حصہ شامل ہے، اس لیے یہ منسوخ نہیں ہو سکتی۔ پہلے وہ خرچ منسوخ کریں۔

## Changing the date to sell (PRD FR-12.39).

# review: pending
purchases-change-sell-date = { -sell-date } بدلیں

# review: pending
purchases-sell-date-title = { -sell-date } بدلیں

# $number: the purchase.
# review: pending
purchases-sell-date-text = اس دن سے { $number } کا مال ہر نئی { -bikri } میں آئے گا جب تک بک نہ جائے۔ یہ دن خریداری کے دن سے پہلے نہیں ہو سکتا۔

# review: pending
purchases-sell-date-done = { -sell-date } بدل گئی۔

## The item lines of an entry, and the split between our shop and another shop (plan 35, PRD FR-12.41).
## Used by a trip purchase entry and by a garden's goods arrived entry.

# review: pending
goods-lines-title = مال کی لائنیں

# review: pending
goods-lines-add = لائن شامل کریں

# review: pending
goods-lines-help-purchase = جو مال خریدا اس کی ہر قسم کی ایک لائن۔ جو مال دوسری دکان کو جائے اس کے { -nag } "{ -dusri-dukan }" میں لکھیں: باقی { -apni-dukan } کے لیے رہتا ہے۔

# review: pending
goods-lines-help-arrival = جو مال آیا اس کی ہر قسم کی ایک لائن۔ جو مال دوسری دکان کو جائے اس کے { -nag } "{ -dusri-dukan }" میں لکھیں: باقی { -apni-dukan } کے لیے رہتا ہے۔

# review: pending
goods-lines-label-purchase = خریدا ہوا مال

# review: pending
goods-lines-label-arrival = آیا ہوا مال

# review: pending
goods-line-col-qty = { -nag }

# review: pending
goods-line-col-weight = وزن (کلو)

# review: pending
goods-line-col-rate = ریٹ

# review: pending
goods-line-col-other-qty = { -dusri-dukan }

# review: pending
goods-line-col-shop = دکان کا نام

# review: pending
goods-line-col-our-qty = { -apni-dukan }

# review: pending
goods-line-col-value = قیمت

# review: pending
purchases-entry-title = اس اندراج کی لائنیں

# review: pending
purchases-entry-col-line = { -qism-maal }

# review: pending
purchases-entry-line-incomplete = ابھی مکمل نہیں

# review: pending
purchases-entry-no-lines = کوئی لائن بھریں تو اس کی لاگت یہاں نظر آئے گی۔

# review: pending
purchases-entry-total = پورے اندراج کی ادا شدہ رقم

## Changing the split between { -apni-dukan } and { -dusri-dukan } (used by a trip purchase and by a garden's goods arrived entry).

# review: pending
split-change = تقسیم بدلیں

# review: pending
split-title = تقسیم بدلیں

# $number: the entry; $qty_left, $our_left, $other_left: how many { -nag } are left in all, for { -apni-dukan } and for { -dusri-dukan }.
# review: pending
split-text = { $number }: { $item }، { $packing }۔ { $qty_left } { -nag } باقی ہیں: { $our_left } { -apni-dukan } کے لیے اور { $other_left } { -dusri-dukan } کے لیے۔ اس سے کوئی رقم نہیں بدلتی۔

# review: pending
split-other-qty = { -dusri-dukan } کے لیے { -nag }

# $qty: how many { -nag } the entry has in all.
# review: pending
split-other-qty-hint = اس اندراج کے کل { $qty } { -nag } میں سے۔ باقی { -apni-dukan } کے لیے رہتے ہیں۔

# review: pending
split-shop = دکان کا نام

# review: pending
split-done = تقسیم بدل دی گئی

# review: pending
purchases-col-our-left = { -apni-dukan } میں باقی

# review: pending
purchases-col-other-left = { -dusri-dukan } میں باقی

# review: pending
purchases-detail-qty-other = { -dusri-dukan } کے لیے { -nag }

# review: pending
purchases-detail-our-sold = { -apni-dukan } میں بکا

# review: pending
purchases-detail-other-sold = { -dusri-dukan } میں بکا

# review: pending
purchases-detail-our-left = { -apni-dukan } کے لیے باقی

# review: pending
purchases-detail-other-left = { -dusri-dukan } کے لیے باقی

# $count: how many { -nag } come from { -apni-dukan } because the part for { -dusri-dukan } is not enough.
# review: pending
split-moved-from-our = اس بکری سے { $count } { -nag } { -apni-dukan } سے { -dusri-dukan } میں منتقل ہوں گے۔
