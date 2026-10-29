# The { -baghat } screens: the list of gardens, a new garden and one garden with its entries,
# its running balance, its history and its closing (plan 17c, PRD 12.11, FR-5.2, FR-7.67,
# FR-12.29 to FR-12.38).
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The list of gardens.
# review: pending
gardens-title = { -baghat }
# review: pending
gardens-help = کچھ شراکت دار ایک موسم کے لیے باغ ٹھیکے پر لیتے ہیں۔ وہ { -thaika } کی رقم { -qist } وار ادا کرتے ہیں اور باغ کے خرچ بھی دیتے ہیں۔ اس کا پھل دکان پر عام { -commission } کے ساتھ بکتا ہے، یا کسی اور دکان میں۔ باغ کا نفع یا نقصان صرف انہی شراکت داروں کا ہوتا ہے جو اس میں شامل ہیں۔
# review: pending
gardens-new = نیا باغ
# review: pending
gardens-load-failed = نئے باغ کا فارم نہیں کھل سکا
# review: pending
gardens-list-label = باغات کی فہرست
# review: pending
gardens-list-from = از
# review: pending
gardens-list-to = تا
# review: pending
gardens-list-status = دکھائیں
# review: pending
gardens-list-search = باغ تلاش کریں
# review: pending
gardens-state-all = سب
# review: pending
gardens-state-open = کھلے
# review: pending
gardens-state-closed = بند
# review: pending
gardens-list-failed = باغات کی فہرست نہیں کھل سکی
# review: pending
gardens-list-empty = ابھی کوئی باغ نہیں
# review: pending
gardens-list-empty-help = بٹن دبا کر پہلا باغ بنائیں۔ اس میں شامل شراکت دار چنیں، پھر { -thaika } اور ادا شدہ رقم درج کریں۔
# review: pending
gardens-list-no-match = ان انتخابات سے کوئی باغ نہیں ملا
# review: pending
gardens-list-no-match-help = دوسری تاریخیں، کھلے یا بند باغ، یا دوسرے الفاظ آزمائیں۔
# review: pending
gardens-list-clear = سب باغ دکھائیں

## The columns of the lists.
# review: pending
gardens-col-number = کوڈ
# review: pending
gardens-col-name = نام
# review: pending
gardens-col-date = تاریخ
# review: pending
gardens-col-status = حالت
# review: pending
gardens-col-balance = چلتا حساب
# review: pending
gardens-col-rent-due = باقی ٹھیکہ
# review: pending
gardens-col-rent = { -thaika }
# review: pending
gardens-col-paid = ادا شدہ
# review: pending
gardens-col-money-in = آنے والی رقم
# review: pending
gardens-col-entry = اندراج
# review: pending
gardens-col-effect = رقم
# review: pending
gardens-col-running-balance = اس کے بعد کا حساب
# review: pending
gardens-col-state = منسوخ
# review: pending
gardens-status-open = کھلا
# review: pending
gardens-status-closed = بند
# $date: the closing day as DD-MM-YYYY.
# review: pending
gardens-status-closed-on = بند ہوا { $date } کو
# review: pending
gardens-entry-cancelled = منسوخ

## What a row of the history is.
# review: pending
gardens-kind-installment = { -qist }
# review: pending
gardens-kind-cost = باغ خرچ
# review: pending
gardens-kind-other-sale = دوسری دکان میں فروخت
# review: pending
gardens-kind-sale = اپنی دکان میں فروخت
# review: pending
gardens-kind-fruit = { -phal }
# review: pending
gardens-kind-close = بندش
# $kind: what the row is; $detail: the reason, the shop or the sale.
# review: pending
gardens-history-entry = { $kind }: { $detail }
# review: pending
gardens-history-detail-sale-elsewhere = { $shop }، { $qty } { -nag }
# review: pending
gardens-history-detail-bikri = { -bikri } { $number }

## A new garden: the shares first, then where the money comes from, then the details (PRD FR-12.29).
# review: pending
gardens-new-title = نیا باغ
# review: pending
gardens-new-help = پہلے چنیں کہ اس باغ میں کون سے شراکت دار ہیں اور ہر ایک کا کتنے فیصد ہے۔ پھر بتائیں کہ ہر شراکت دار کی رقم کہاں سے آئے گی۔ آخر میں تفصیل بھریں۔
# review: pending
gardens-new-save = باغ بنائیں
# review: pending
gardens-new-shares-title = شراکت دار اور حصے
# review: pending
gardens-new-from-defaults = فرم کے { -hissa } یا پچھلے باغ کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔
# review: pending
gardens-new-shares-help = باغ میں شامل شراکت داروں کے حصے ٹھیک 100 فیصد ہونے چاہییں۔ جس کا حصہ نہ ہو وہ اس میں شامل نہیں۔ پہلے اندراج کے بعد حصے بدلے نہیں جا سکتے۔
# review: pending
gardens-new-sources-help = ہر شراکت دار کی ہر ادائیگی کا حصہ دکان کی رقم سے آتا ہے یا اس کی اپنی جیب سے۔ دکان کی رقم سے ادا ہونے والا حصہ اس کی کل رقم سے کٹتا ہے اور { -rokar } سے نکلتا ہے۔ اپنی جیب سے ادا ہونے والا حصہ صرف درج ہوتا ہے: کچھ پوسٹ نہیں ہوتا۔
# review: pending
gardens-new-no-partners = ابھی کوئی شراکت دار نہیں بنا، اس لیے باغ نہیں بن سکتا۔
# review: pending
gardens-new-no-partners-link = شراکت داروں کی ترتیب کھولیں
# review: pending
gardens-new-details-title = تفصیل
# review: pending
gardens-field-name-ur = اردو نام
# review: pending
gardens-field-name-en = انگریزی نام
# review: pending
gardens-field-date = تاریخ
# review: pending
gardens-field-item-hint = اصل پھل، اگر کوئی ہو۔
# review: pending
gardens-field-rent = پورا { -thaika }
# review: pending
gardens-field-rent-hint = موسم کے لیے باغ کی پوری قیمت، پورے روپوں میں۔
# review: pending
gardens-field-season-end = موسم کا اختتام
# review: pending
gardens-field-paid-now = پہلی { -qist }
# review: pending
gardens-field-paid-now-hint = آج کچھ ادا نہ ہو تو خالی چھوڑ دیں۔ یہ پہلی قسط کے طور پر محفوظ ہوگی۔
# review: pending
gardens-field-paid-now-reason = پہلی قسط کیوں دی گئی
# review: pending
gardens-field-paid-now-reason-hint = خود بھر دیا گیا ہے۔ ٹھیک نہ ہو تو بدل دیں۔
# The words put in the reason of the first installment when its amount is typed (PRD FR-12.35). Free text: saved as shown, in the language of the screen at that moment. Plain words on purpose, with no glossary reference, so the saved text has no invisible marks in it.
# review: pending
gardens-first-installment-reason = پہلی قسط
# review: pending
gardens-field-note = نوٹ
# review: pending
gardens-first-parts-title = پہلی قسط میں ہر شراکت دار کا حصہ
# review: pending
gardens-preview-failed = اعداد نہیں نکل سکے
# $code: the code of the garden, type 008.
# review: pending
gardens-created = { $code } بن گیا

## One garden.
# review: pending
gardens-back = باغات پر واپس
# $number: the number of the garden.
# review: pending
gardens-detail-title = { $code }
# review: pending
gardens-detail-failed = باغ نہیں کھل سکا
# review: pending
gardens-none = کوئی نہیں
# review: pending
gardens-closed-on = بند ہوا
# review: pending
gardens-balance = چلتا حساب (آنے والی رقم منفی کل ادائیگی)
# review: pending
gardens-rent-due = باقی ٹھیکہ
# review: pending
gardens-figures-title = باغ کے اعداد
# review: pending
gardens-partners-title = شراکت دار
# review: pending
gardens-shares-locked = اس باغ میں اندراج یا بکری ہو چکی ہے، اس لیے حصے اب طے ہیں۔
# review: pending
gardens-fact-rent = { -thaika }
# review: pending
gardens-fact-installments = { -qist } وار ادا شدہ
# review: pending
gardens-fact-costs = باغ کے خرچ میں ادا شدہ
# review: pending
gardens-fact-paid = کل ادا شدہ
# review: pending
gardens-fact-paid-shop = دکان کی رقم سے ادا شدہ
# review: pending
gardens-fact-paid-pocket = جیبوں سے ادا شدہ
# review: pending
gardens-fact-money-in = آنے والی رقم ({ -commission } کے بعد فروخت)
# review: pending
gardens-fact-other-shop = اس میں سے دوسری دکانوں کی فروخت
# review: pending
gardens-fact-account = اس وقت باغ کے کھاتے میں
# $date: the closing day as DD-MM-YYYY.
# review: pending
gardens-closed-notice = یہ باغ { $date } کو بند ہو چکا ہے۔ اس میں نیا اندراج، نیا لاٹ یا پھل نہیں لیا جا سکتا۔ اگر اس کی بندش واپس لی جائے تو یہ دوبارہ کھل جاتا ہے۔

## Each partner's money in the garden (PRD FR-12.38, FR-17.10).
# review: pending
gardens-partner-results-title = ہر شراکت دار کی رقم
# review: pending
gardens-partner-results-empty = ابھی کوئی شراکت دار نہیں۔
# review: pending
gardens-partner-paid-shop = دکان سے ادا
# review: pending
gardens-partner-paid-pocket = اپنی جیب سے ادا
# review: pending
gardens-partner-back = واپس ملنے والی رقم
# review: pending
gardens-partner-net-open = اب تک کا نتیجہ
# review: pending
gardens-partner-net-closed = آخری نتیجہ
# review: pending
gardens-partner-loss = نقصان

## The entry form and its type list (PRD FR-7.67, FR-12.35, FR-12.36).
# review: pending
gardens-type-label = اندراج کی قسم
# review: pending
gardens-type-installment = { -qist }
# review: pending
gardens-type-cost = باغ خرچ
# review: pending
gardens-type-other-sale = دوسری دکان میں فروخت
# review: pending
gardens-entry-new = نیا اندراج
# review: pending
gardens-sources-title = رقم کہاں سے آئے گی
# review: pending
gardens-sources-hint = باغ کے مطابق بھرا گیا ہے۔ اگر کوئی شراکت دار اس اندراج میں کسی اور طرح ادا کرے تو بدل دیں۔ اپنی جیب سے ادا ہونے والا حصہ کچھ پوسٹ نہیں کرتا۔
# review: pending
gardens-parts-title = ہر شراکت دار کا حصہ
# review: pending
gardens-installment-amount = { -qist } کی رقم
# review: pending
gardens-cost-amount = خرچ کی رقم
# review: pending
gardens-payment-amount-hint = پورے روپے۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 20000+5000۔
# review: pending
gardens-payment-reason = وجہ
# review: pending
gardens-installment-reason-hint = مثلاً: ٹھیکے کی پہلی قسط۔
# review: pending
gardens-cost-reason-hint = مثلاً: مزدوری، سپرے، ٹرانسپورٹ۔
# review: pending
gardens-sale-shop = دکان
# review: pending
gardens-sale-qty = بکے ہوئے { -nag }
# review: pending
gardens-sale-qty-hint = پورا عدد۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 20+25۔
# review: pending
gardens-sale-amount = ملنے والی رقم
# review: pending
gardens-sale-arrival = مال کہاں سے

# review: pending
gardens-sale-arrival-hint = صرف وہ مال نظر آتا ہے جو باقی ہے۔ پہلے { -dusri-dukan } کے لیے رکھا ہوا مال بکتا ہے۔ اس سے زیادہ بیچیں تو باقی { -apni-dukan } کے مال سے آتا ہے، اور اسکرین بتاتی ہے کتنا۔

# $number: the entry; $other_left, $our_left: how many { -nag } are left for { -dusri-dukan } and for { -apni-dukan }.
# review: pending
gardens-sale-arrival-option = { $number }: { $item }، { $packing } ({ -dusri-dukan } کے لیے { $other_left }، { -apni-dukan } کے لیے { $our_left } باقی)

# review: pending
gardens-sale-amount-hint = رقم { -rokar } میں آتی ہے اور باغ بند ہونے تک باغ کے کھاتے میں رہتی ہے۔
# review: pending
gardens-saved-installment = { -qist } { $number } محفوظ ہو گئی
# review: pending
gardens-saved-cost = باغ خرچ { $number } محفوظ ہو گیا
# $number: the number of the entry in its own series.
# review: pending
gardens-saved-other-sale = دوسری دکان میں فروخت { $number } محفوظ ہو گئی

## What the entry does, live under the form (PRD FR-12.37).
# review: pending
gardens-numbers-title = یہ اندراج
# review: pending
gardens-numbers-empty = فارم بھریں تو اعداد یہاں نظر آئیں گے۔
# review: pending
gardens-numbers-incomplete = فارم مکمل ہوگا تو اعداد نظر آئیں گے۔
# review: pending
gardens-numbers-amount = رقم
# review: pending
gardens-numbers-balance-before = اس سے پہلے کا چلتا حساب
# review: pending
gardens-numbers-balance-after = اس کے بعد کا چلتا حساب
# review: pending
gardens-numbers-rent-due-after = اس کے بعد باقی ٹھیکہ

## The history of the garden (PRD FR-12.37, FR-19.11).
# review: pending
gardens-history-title = تاریخچہ
# review: pending
gardens-history-label = باغ کا تاریخچہ
# review: pending
gardens-history-empty = ابھی کچھ درج نہیں ہوا
# $key: the key that saves.
# review: pending
gardens-history-empty-help = اس باغ کی { -qist }، خرچ اور فروخت یہاں نظر آتی ہیں۔ اوپر کا فارم بھریں اور { $key } دبائیں۔
# review: pending
gardens-history-count = { $total } میں سے { $shown } نظر آ رہے ہیں
# review: pending
gardens-history-more = مزید دکھائیں
# review: pending
gardens-history-cancel = اندراج منسوخ کریں
# review: pending
gardens-entry-cancelled-reason = یہ اندراج منسوخ ہو چکا ہے۔ وجہ: { $reason }

## Cancelling an entry (PRD FR-7.8).
# review: pending
gardens-cancel-title = یہ اندراج منسوخ کریں؟
# $entry: what the entry is, for example Sale in another shop.
# review: pending
gardens-cancel-text = { $entry } منسوخ ہو جائے گا۔ اس کی پوسٹنگ الٹ دی جائے گی، ہر شراکت دار کا حصہ جیسا تھا ویسا ہو جائے گا، اور اندراج منسوخ کے نشان کے ساتھ تاریخچے میں رہے گا۔
# review: pending
gardens-cancel-reason = وجہ
# review: pending
gardens-cancel-confirm = اندراج منسوخ کریں
# review: pending
gardens-cancel-confirm-cash = ہاں، منسوخ کریں
# review: pending
gardens-cancelled = { $entry } کو منسوخ کر دیا گیا۔

## Closing the garden (PRD FR-12.38, FR-7.8, E64).
# review: pending
gardens-close-title = باغ بند کریں
# review: pending
gardens-close-help = جب پھل آنا بند ہو جائے تو باغ بند کریں۔ اس کی رقم شراکت داروں میں ان کے حصوں کے مطابق بٹ جاتی ہے: ہر ایک کو اس کی ادا کی ہوئی رقم واپس ملتی ہے، اور نفع یا نقصان میں اس کا حصہ جمع یا منفی ہوتا ہے۔ اس کے بعد باغ میں مزید اندراج نہیں ہوتا۔
# review: pending
gardens-close-date = بندش کا دن
# $date: the day of the latest entry as DD-MM-YYYY.
# review: pending
gardens-close-earliest = { $date } سے پہلے نہیں، یہ باغ کا آخری اندراج ہے۔
# review: pending
gardens-close-numbers-empty = بندش کا دن چنیں تو اعداد یہاں نظر آئیں گے۔
# review: pending
gardens-close-money-in = آنے والی رقم
# review: pending
gardens-close-total-cost = کل لاگت
# review: pending
gardens-close-balance = شراکت داروں کو ملنے والی رقم
# review: pending
gardens-close-loss-note = باغ کو نقصان ہوا۔ ہر شراکت دار کا نقصان کا حصہ نیچے اس کے آخری نتیجے میں ہے۔
# review: pending
gardens-close-credits-title = ہر شراکت دار کی واپس ملنے والی رقم
# review: pending
gardens-close-nothing-posted = باغ کے کھاتے میں کچھ نہیں، اس لیے کچھ پوسٹ نہیں ہوگا۔ باغ پھر بھی بند ہو جائے گا۔
# review: pending
gardens-close-button = باغ بند کریں
# review: pending
gardens-close-confirm-title = یہ باغ بند کریں؟
# $name: the garden name; $date: the closing day; $balance: the money that goes to the partners.
# review: pending
gardens-close-confirm-text = باغ { $name } { $date } کو بند ہو جائے گا۔ { $balance } روپے اس کے شراکت داروں میں ان کے حصوں کے مطابق بٹ جائیں گے۔ اس کے بعد اس میں نیا اندراج، نیا لاٹ یا پھل نہیں لیا جا سکے گا۔ جب تک اس دن کا حساب کھلا ہے بندش واپس لی جا سکتی ہے۔
# review: pending
gardens-close-confirm = ہاں، بند کریں
# review: pending
gardens-closed-toast = { $code } بند ہو گیا

## Undoing the closing (PRD FR-12.38).
# review: pending
gardens-reopen-button = بندش واپس لیں
# review: pending
gardens-reopen-title = اس باغ کی بندش واپس لیں؟
# review: pending
gardens-reopen-text = بندش الٹ دی جائے گی: رقم واپس باغ کے کھاتے میں چلی جائے گی اور شراکت داروں کی کل رقم سے نکل جائے گی، اور باغ دوبارہ کھل جائے گا۔ بندش تاریخچے میں رہے گی۔
# review: pending
gardens-reopen-reason = وجہ
# review: pending
gardens-reopen-confirm = بندش واپس لیں
# review: pending
gardens-reopened-toast = { $code } دوبارہ کھل گیا

# review: pending
gardens-filter-all = تمام باغات

# review: pending
gardens-filter-failed = باغات کی فہرست نہیں کھل سکی

## The number of a goods arrived entry: its own series (plan 33 version 1.1, PRD FR-12.40).

# $number: the number of the entry in its series.
# review: pending
gardens-arrival-number = مآ-{ $number }

## Goods arrived (مال آیا): a garden's stock (plan 33 version 1.1, PRD FR-12.40). They carry no money.

# review: pending
gardens-type-arrival = { -maal-aaya }

# $number: the number of the entry.
# review: pending
gardens-saved-arrival = { -maal-aaya } { $number } محفوظ ہو گیا

# $count: how many lines were saved; $first, $last: the first and the last number.
# review: pending
gardens-saved-arrivals = { -maal-aaya } کے { $count } اندراج محفوظ ہو گئے: { $first } سے { $last } تک

# review: pending
gardens-arrival-qty = آئے ہوئے { -nag }

# review: pending
gardens-arrival-qty-hint = پورا عدد۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 20+25۔

# review: pending
gardens-arrival-sell-date-hint = شروع میں مال آنے کا دن ہوتا ہے۔ اس دن سے مال ہر نئی { -bikri } میں آتا رہتا ہے جب تک بک نہ جائے۔

# review: pending
gardens-arrival-posts-nothing = اس مال کے ساتھ کوئی رقم نہیں ہے۔ یہ { -bagh } کے ذخیرے میں رہتا ہے جب تک { -bikri } میں بک نہ جائے یا ضائع نہ لکھا جائے۔ { -sell-date } سے نئی { -bikri } میں اس کا { -lot } پہلے سے موجود ہوتا ہے۔

# review: pending
gardens-arrival-title = { -maal-aaya }

# review: pending
gardens-arrival-help = اس { -bagh } سے آیا ہوا وہ مال جو ابھی بکا نہیں۔ ہر نئی { -bikri } میں اس کی { -sell-date } پر اس کا ایک { -lot } پہلے سے موجود ہوتا ہے۔ جو نہ بکے وہ اگلے دن پھر آتا ہے۔

# review: pending
gardens-arrival-failed = آیا ہوا مال پڑھا نہیں جا سکا

# review: pending
gardens-arrival-empty = ابھی کوئی مال نہیں آیا

# review: pending
gardens-arrival-empty-help = اوپر اندراج کی قسم میں { -maal-aaya } چنیں، { -qism-maal }، { -packing } اور { -nag } لکھیں اور محفوظ کریں۔

# review: pending
gardens-arrival-label = آئے ہوئے مال کی فہرست

# review: pending
gardens-arrival-col-number = نمبر

# review: pending
gardens-arrival-col-item = { -qism-maal }

# review: pending
gardens-arrival-col-left = باقی

# review: pending
gardens-arrival-col-date = تاریخ

# review: pending
gardens-arrival-col-qty = آیا

# review: pending
gardens-arrival-item-packing = { $item }، { $packing }

# review: pending
gardens-arrival-item-cancelled = { $item } (منسوخ)

# $number: the number of the entry; $item: its item.
# review: pending
gardens-arrival-card-title = { -maal-aaya }: { $number }، { $item }

# $reason: why it was cancelled.
# review: pending
gardens-arrival-cancelled = یہ اندراج منسوخ ہے۔ { $reason }

# The lots of the sales that took goods from the entry.
# review: pending
gardens-arrival-lots = اس مال کی فروخت

# review: pending
gardens-arrival-lot-bikri = { -bikri }

# review: pending
gardens-arrival-cancel = { -maal-aaya } منسوخ کریں

# $number: the entry.
# review: pending
gardens-arrival-sell-date-title = { -sell-date } بدلیں

# $number: the entry.
# review: pending
gardens-arrival-sell-date-text = اس دن سے { $number } کا مال ہر نئی { -bikri } میں آئے گا جب تک بک نہ جائے۔ یہ دن مال آنے کے دن سے پہلے نہیں ہو سکتا۔

# review: pending
gardens-arrival-reason = وجہ

# review: pending
gardens-arrival-cancel-title = یہ اندراج منسوخ کریں؟

# $number: the entry.
# review: pending
gardens-arrival-cancel-text = { $number } منسوخ ہو جائے گا اور مال ذخیرے سے نکل جائے گا۔ یہ تب ہی منسوخ ہو سکتا ہے جب اس میں سے کچھ بکا یا ضائع نہ ہوا ہو۔ کچھ پوسٹ نہیں ہوگا۔

# review: pending
gardens-arrival-cancel-confirm = { -maal-aaya } منسوخ کریں

# $number: the entry.
# review: pending
gardens-arrival-cancelled-toast = { $number } منسوخ ہو گیا

# review: pending
gardens-arrival-write-off-title = مال ضائع لکھیں

# $number: the entry; $item, $packing: what it is; $qty_left: how many are left.
# review: pending
gardens-arrival-write-off-text = { $number } ({ $item }، { $packing }) کا خراب یا گم مال ذخیرے سے نکال دیا جائے گا۔ ابھی { $qty_left } { -nag } باقی ہیں۔ کچھ پوسٹ نہیں ہوگا۔

# review: pending
gardens-arrival-write-off-qty = ضائع { -nag }

# review: pending
gardens-arrival-write-off-confirm = ضائع لکھیں

# $qty: how many were written off.
# review: pending
gardens-arrival-write-off-done = { $qty } { -nag } ضائع لکھ دیے گئے

# review: pending
gardens-arrival-write-off-undo-title = ضائع واپس لیں

# review: pending
gardens-arrival-write-off-undo-text = مال واپس ذخیرے میں چلا جائے گا۔ ضائع منتخب کریں اور وجہ لکھیں۔

# review: pending
gardens-arrival-write-off-undo-confirm = ضائع واپس لیں

# review: pending
gardens-arrival-write-off-pick = ضائع

# $qty: how many were written off; $reason: why.
# review: pending
gardens-arrival-write-off-option = { $qty } { -nag }: { $reason }

# review: pending
gardens-arrival-write-off-undone = ضائع واپس لے لیا گیا

# review: pending
gardens-numbers-left-after = اس کے بعد اندراج میں باقی
