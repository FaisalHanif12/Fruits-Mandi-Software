# The { -market-safar } screens: the list of trips, a new trip and one trip with its purchases,
# costs and sales in other shops (plan 16b, PRD 12.10, FR-5.2, FR-7.67, FR-12.24 to FR-12.33).
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The list of trips.

# review: pending
trips-title = { -market-safar }

# review: pending
trips-help = کچھ شراکت دار کسی بڑی منڈی میں آدمی بھیج کر مال خریدواتے ہیں۔ یہ مال انہی شراکت داروں کا ہوتا ہے۔ دکان اسے بیچتی ہے اور اپنا معمول کا { -commission } لیتی ہے۔ ہر سفر یہاں اپنے مال اور نتیجے کے ساتھ نظر آتا ہے۔

# review: pending
trips-new = نیا سفر

# review: pending
trips-load-failed = سفر نہیں کھل سکا

# review: pending
trips-list-label = سفروں کی فہرست

# review: pending
trips-list-from = از

# review: pending
trips-list-to = تا

# review: pending
trips-list-status = دکھائیں

# review: pending
trips-list-search = سفر تلاش کریں

# review: pending
trips-state-all = سب

# review: pending
trips-state-open = کھلے

# review: pending
trips-state-closed = بند

# review: pending
trips-list-failed = سفروں کی فہرست نہیں کھل سکی

# review: pending
trips-list-empty = ابھی کوئی سفر نہیں

# review: pending
trips-list-empty-help = بٹن دبا کر پہلا سفر بنائیں۔ اس میں شامل شراکت دار چنیں، پھر جو خریدا گیا وہ درج کریں۔

# review: pending
trips-list-no-match = ان انتخابات سے کوئی سفر نہیں ملا

# review: pending
trips-list-no-match-help = دوسری تاریخیں، کھلے یا بند سفر، یا دوسرے الفاظ آزمائیں۔

# review: pending
trips-list-clear = سب سفر دکھائیں

## Numbers: a trip has a code of type 007 (plan 34); its entries have series with their own letters.

# A trip cost.
# review: pending
trips-cost-number = سخ-{ $number }

# A sale in another shop.
# review: pending
trips-sale-number = دف-{ $number }

## The columns of the lists.

# review: pending
trips-col-number = کوڈ

# review: pending
trips-col-trip = سفر

# review: pending
trips-col-date = تاریخ

# review: pending
trips-col-market = منڈی

# review: pending
trips-col-status = حالت

# review: pending
trips-col-sales = فروخت کی رقم

# review: pending
trips-col-entry = اندراج

# review: pending
trips-col-amount = رقم

# review: pending
trips-col-state = منسوخ

# review: pending
trips-col-detail = تفصیل

## A new trip: the shares first, then where the money comes from, then the details (PRD FR-12.29).

# review: pending
trips-new-title = نیا سفر

# review: pending
trips-new-help = پہلے چنیں کہ اس سفر میں کون سے شراکت دار ہیں اور ہر ایک کا کتنے فیصد ہے۔ پھر بتائیں کہ ہر شراکت دار کی رقم کہاں سے آئے گی۔ آخر میں تفصیل بھریں۔

# review: pending
trips-new-save = سفر بنائیں

# review: pending
trips-new-shares-title = شراکت دار اور حصے

# The form starts as the last trip was.
# review: pending
trips-new-from-last = پچھلے سفر کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔

# The first trip starts from the firm's shares.
# review: pending
trips-new-firm-shares = فرم کے { -hissa } کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔

# review: pending
trips-new-shares-help = سفر میں شامل شراکت داروں کے حصے ٹھیک 100 فیصد ہونے چاہییں۔ جس کا حصہ نہ ہو وہ اس میں شامل نہیں۔ پہلے اندراج کے بعد حصے بدلے نہیں جا سکتے۔

# review: pending
trips-new-no-partners = ابھی کوئی شراکت دار نہیں بنا، اس لیے سفر نہیں بن سکتا۔

# review: pending
trips-new-no-partners-link = شراکت داروں کی ترتیب کھولیں

# review: pending
trips-new-details-title = تفصیل

# $number: the number of the new trip.
# review: pending
trips-created = { $code } بن گیا

# review: pending
trips-field-date = تاریخ

# review: pending
trips-field-market = منڈی

# review: pending
trips-field-market-hint = جہاں سے مال خریدا جائے، جیسا آپ لکھنا چاہیں۔

# review: pending
trips-field-sent = کس کو بھیجا

# review: pending
trips-field-note = نوٹ

## One trip.

# $number: the number of the trip.
# review: pending
trips-detail-title = { $code }

# review: pending
trips-detail-failed = سفر نہیں کھل سکا

# review: pending
trips-back = سب سفر

# review: pending
trips-status-open = کھلا

# review: pending
trips-status-closed = بند

# review: pending
trips-none = درج نہیں

# review: pending
trips-closed-on = بند ہوا

# review: pending
trips-figures-title = مال اور نتیجہ

# review: pending
trips-partners-title = اس سفر کے شراکت دار

# review: pending
trips-shares-locked = اس سفر میں اندراج ہو چکا ہے، اس لیے حصے اب طے ہیں۔

# A trip closes by itself and opens again if a sale or a write-off is reversed (PRD FR-12.33).
# review: pending
trips-closed-notice = یہ سفر بند ہے، کیونکہ جو خریدا گیا تھا وہ سب بک گیا یا ضائع ہو گیا۔ اس میں نئی خریداری یا نیا خرچ نہیں ہو سکتا۔ اگر اس کی کوئی فروخت یا ضائع واپس لیا جائے تو یہ دوبارہ کھل جاتا ہے۔

# review: pending
trips-fact-bought = خریدے { -nag }

# review: pending
trips-fact-cost = کل لاگت

# review: pending
trips-loss-note = اس سفر میں نقصان ہو رہا ہے۔

## What each partner has put in and got back (PRD FR-12.27).

# review: pending
trips-partner-results-title = اس سفر میں ہر شراکت دار کی رقم

# review: pending
trips-partner-results-empty = ابھی کوئی رقم نہیں چلی۔

# review: pending
trips-partner-in-shop = دکان سے لگائی

# review: pending
trips-partner-in-pocket = اپنی جیب سے لگائی

# review: pending
trips-partner-back = واپس ملی رقم

# His part of the result so far: money back less money in.
# review: pending
trips-partner-net = اب تک کا نتیجہ

# Read by screen readers; the number is red as well.
# review: pending
trips-partner-loss = نقصان

## The entry form: the type list and what each type needs (PRD FR-7.67).

# review: pending
trips-type-label = اندراج کی قسم

# review: pending
trips-type-purchase = { -khareedari }

# A cost of the whole trip, for example the truck or the food.
# review: pending
trips-type-cost = سفر خرچ

# review: pending
trips-type-other-sale = دوسری دکان میں فروخت

# review: pending
trips-entry-new = نیا اندراج

# review: pending
trips-parts-title = ہر شراکت دار کا حصہ

# review: pending
trips-sources-title = رقم کہاں سے آئے گی

# review: pending
trips-sources-hint = سفر کے مطابق بھرا گیا ہے۔ اگر کوئی شراکت دار اس اندراج میں کسی اور طرح ادا کرے تو بدل دیں۔

# review: pending
trips-preview-failed = اعداد نہیں نکل سکے

# $number: the number of the purchase.
# review: pending
trips-saved-purchase = { -khareedari } { $number } محفوظ ہو گئی

# $count: how many lines were saved; $first, $last: the first and the last number.
# review: pending
trips-saved-purchases = { $count } { -khareedari } محفوظ ہو گئیں: { $first } سے { $last } تک

# $number: the number of the trip cost.
# review: pending
trips-saved-cost = سفر خرچ { $number } محفوظ ہو گیا

# $number: the number of the sale.
# review: pending
trips-saved-sale = دوسری دکان میں فروخت { $number } محفوظ ہو گئی

# review: pending
trips-saved-sale-closed = دوسری دکان میں فروخت { $number } محفوظ ہو گئی۔ اب سب مال بک چکا ہے، اس لیے سفر بند ہو گیا۔

## A trip cost (PRD FR-12.31, C24).

# review: pending
trips-cost-amount = رقم

# review: pending
trips-cost-amount-hint = پورے سفر کا خرچ۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 3000+1000۔

# review: pending
trips-cost-reason = وجہ

# review: pending
trips-cost-reason-hint = مثلاً ٹرک یا کھانا۔

# review: pending
trips-cost-numbers-title = خرچ کیسے بانٹا گیا

# review: pending
trips-cost-numbers-empty = فارم بھریں تو بانٹ یہاں نظر آئے گی۔

# review: pending
trips-cost-numbers-incomplete = خرچ کی بانٹ دیکھنے کے لیے رقم اور وجہ لکھیں۔

# The cost is spread over the purchases that still have goods left, by the { -nag } bought.
# review: pending
trips-cost-spread-title = ہر خریداری کا حصہ

# $qty: the { -nag } bought in that purchase.
# review: pending
trips-cost-spread-qty = ({ $qty } { -nag })

## A sale in another shop (PRD FR-12.32, C21, C25).

# review: pending
trips-sale-purchase = مال کہاں سے

# review: pending
trips-sale-purchase-hint = صرف اسی سفر کی وہ خریداریاں نظر آتی ہیں جن میں مال باقی ہے۔ پہلے { -dusri-dukan } کے لیے رکھا ہوا مال بکتا ہے۔ اس سے زیادہ بیچیں تو باقی { -apni-dukan } کے مال سے آتا ہے، اور اسکرین بتاتی ہے کتنا۔

# $number: the purchase; $other_left, $our_left: how many { -nag } are left for { -dusri-dukan } and for { -apni-dukan }.
# review: pending
trips-sale-purchase-option = { $number }: { $item }، { $packing } ({ -dusri-dukan } کے لیے { $other_left }، { -apni-dukan } کے لیے { $our_left } باقی)

# review: pending
trips-sale-shop = دکان

# review: pending
trips-sale-qty = بیچے ہوئے { -nag }

# review: pending
trips-sale-amount = ملی ہوئی رقم

# review: pending
trips-sale-amount-hint = یہ رقم { -rokar } میں آتی ہے۔

# review: pending
trips-sale-numbers-title = یہ فروخت

# review: pending
trips-sale-numbers-empty = فارم بھریں تو فروخت کا نتیجہ یہاں نظر آئے گا۔

# review: pending
trips-sale-numbers-incomplete = نتیجہ دیکھنے کے لیے مال چنیں اور مقدار اور رقم لکھیں۔

# review: pending
trips-sale-left-after = اس کے بعد خریداری میں باقی

# review: pending
trips-sale-other-left-after = اس کے بعد { -dusri-dukan } کے لیے باقی

# review: pending
trips-sale-cost-left-after = باقی مال کی لاگت

# What each partner gets of the money that comes in (PRD C25).
# review: pending
trips-sale-credits-title = رقم میں ہر شراکت دار کا حصہ

## The purchases of the trip.

# review: pending
trips-purchases-title = اس سفر کی خریداریاں

# review: pending
trips-purchases-label = اس سفر کی خریداریوں کی فہرست

# review: pending
trips-purchases-empty = ابھی کوئی خریداری نہیں

# $key: the key that saves.
# review: pending
trips-purchases-empty-help = اوپر کا فارم بھریں اور { $key } دبائیں۔

# review: pending
trips-purchases-select-help = ↑ ↓ سے خریداری پر جائیں۔ اس کے اعداد، نتیجہ اور کام فہرست کے نیچے نظر آتے ہیں۔

## The history of the trip (PRD FR-19.11).

# review: pending
trips-entries-title = اس سفر کا ریکارڈ

# review: pending
trips-entries-label = اس سفر کا ریکارڈ

# review: pending
trips-entries-empty = ابھی کچھ درج نہیں ہوا

# $key: the key that saves.
# review: pending
trips-entries-empty-help = اس سفر کی خریداریاں، خرچ اور فروخت یہاں درج نظر آتی ہیں۔ اوپر کا فارم بھریں اور { $key } دبائیں۔

# $shown: how many are on screen; $total: how many the trip has.
# review: pending
trips-entries-count = { $total } میں سے { $shown } نظر آ رہے ہیں

# review: pending
trips-entries-cancel = اندراج منسوخ کریں

# $kind: the type of entry; $number: its number in its own series.
# review: pending
trips-entry-label = { $kind } { $number }

# review: pending
trips-entry-cancelled = منسوخ

# $reason: why it was cancelled.
# review: pending
trips-entry-cancelled-reason = یہ اندراج منسوخ ہو چکا ہے۔ وجہ: { $reason }

# $shop: the shop as typed; $qty: { -nag } sold; $purchase: the purchase.
# review: pending
trips-entry-detail-sale = { $shop }: { $purchase } سے { $qty } { -nag }

# $number: the purchase; $qty: the { -nag } it was bought in.
# review: pending
trips-entry-spread-line = { -khareedari } { $number } کا حصہ ({ $qty } { -nag })

## Cancelling a trip cost or a sale in another shop (PRD FR-12.31, FR-12.32, FR-7.8).

# review: pending
trips-cancel-title = یہ اندراج منسوخ کریں؟

# $entry: the entry and its number.
# review: pending
trips-cancel-text-cost = { $entry } منسوخ ہو جائے گا۔ ہر شراکت دار کا حصہ جیسا تھا ویسا ہو جائے گا، اور خریداریوں پر لگا اس خرچ کا حصہ ہٹ جائے گا۔

# $entry: the entry and its number.
# review: pending
trips-cancel-text-sale = { $entry } منسوخ ہو جائے گی۔ رقم دوبارہ { -rokar } سے نکل جائے گی، مال واپس خریداری میں چلا جائے گا، اور ہر شراکت دار کا حصہ واپس ہو جائے گا۔

# review: pending
trips-cancel-reason = وجہ

# review: pending
trips-cancel-confirm = اندراج منسوخ کریں

# review: pending
trips-cancel-confirm-cash = ہاں، منسوخ کریں

# $entry: the entry and its number.
# review: pending
trips-cancelled = { $entry } کو منسوخ کر دیا گیا۔

# $entry: the entry and its number.
# review: pending
trips-cancelled-reopened = { $entry } کو منسوخ کر دیا گیا۔ سفر دوبارہ کھل گیا ہے۔
