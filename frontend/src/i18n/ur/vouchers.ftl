# The vouchers (plans 10 and 15, PRD 15.1 to 15.4): ONE screen with a type list for وصولی, ادائیگی and
# پیشگی (FR-7.67), the panels under the form, the list, the cancel box and the اگراہی فہرست.
# Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The three types and their screens.

# review: pending
vouchers-field-type = قسم

# review: pending
vouchers-type-receipt = { -wasooli }

# review: pending
vouchers-type-payment = { -adaigi }

# review: pending
vouchers-type-advance = { -peshgi-khata }

# review: pending
vouchers-receipt-title = { -wasooli }

# review: pending
vouchers-receipt-help = { -gahak } سے رقم لیں۔ رقم اس کے سب سے پرانے بلوں پر پہلے لگتی ہے۔ اگر { -beopari } اپنی { -peshgi } واپس کر رہا ہو تو "وصولی کس سے" میں { -beopari } چن لیں۔

# review: pending
vouchers-payment-title = { -adaigi }

# review: pending
vouchers-payment-help = { -beopari } یا کسی اور کھاتے کو رقم دیں۔ کسی { -gahak } کو رقم دینا { -naqd-udhar } ہے، اس کی وجہ لکھنی ہوگی۔

# review: pending
vouchers-advance-title = { -peshgi-khata }

# review: pending
vouchers-advance-help = { -beopari } کو اس کا مال بکنے سے پہلے رقم دیں۔ محفوظ کرنے کے بعد اس کا نیا بقایا نظر آتا ہے۔

# review: pending
vouchers-load-failed = واؤچر کی اسکرین کھل نہیں سکی

# review: pending
vouchers-new = نیا اندراج

# review: pending
vouchers-print = پرنٹ

# review: pending
vouchers-edit = تبدیل کریں

# review: pending
vouchers-cancel = واؤچر منسوخ کریں

# review: pending
vouchers-collection-open = { -agrahi } فہرست

## The number of a voucher: its series letter and its number.

# review: pending
vouchers-number-receipt = و-{ $number }

# review: pending
vouchers-number-payment = ا-{ $number }

# review: pending
vouchers-number-advance = پ-{ $number }

## After a save, an edit and a cancel. $kind is receipt, payment or advance; $number is the number
## in its series.

# review: pending
vouchers-saved = { $kind ->
    [receipt] { -wasooli } { $number } محفوظ ہو گئی
    [payment] { -adaigi } { $number } محفوظ ہو گئی
   *[advance] { -peshgi-khata } { $number } محفوظ ہو گیا
}

# review: pending
vouchers-edited = { $kind ->
    [receipt] { -wasooli } { $number } میں تبدیلی محفوظ ہو گئی
    [payment] { -adaigi } { $number } میں تبدیلی محفوظ ہو گئی
   *[advance] { -peshgi-khata } { $number } میں تبدیلی محفوظ ہو گئی
}

# review: pending
vouchers-cancelled = { $kind ->
    [receipt] { -wasooli } { $number } منسوخ ہو گئی
    [payment] { -adaigi } { $number } منسوخ ہو گئی
   *[advance] { -peshgi-khata } { $number } منسوخ ہو گیا
}

# review: pending
vouchers-editing-note = { $kind ->
    [receipt] آپ { -wasooli } { $number } میں تبدیلی کر رہے ہیں۔
    [payment] آپ { -adaigi } { $number } میں تبدیلی کر رہے ہیں۔
   *[advance] آپ { -peshgi-khata } { $number } میں تبدیلی کر رہے ہیں۔
} محفوظ کرنے پر پرانا اندراج الٹ دیا جاتا ہے اور تاریخچے میں موجود رہتا ہے۔

## The fields of a voucher.

# review: pending
vouchers-field-date = تاریخ

# review: pending
vouchers-field-from = وصولی کس سے

# review: pending
vouchers-from-buyer = { -gahak }

# review: pending
vouchers-from-seller = { -beopari }

# review: pending
vouchers-field-buyer = { -gahak }

# review: pending
vouchers-field-seller = { -beopari }

# review: pending
vouchers-field-paid-to = کس کو ادائیگی

# review: pending
vouchers-field-person = نام یا فون نمبر

# review: pending
vouchers-person-hint = { -pukhta-khata } میں پہلے سے درج شخص کا نام یا فون لکھیں اور فہرست سے اسے چن لیں۔

# review: pending
vouchers-make-regular = باقاعدہ کھاتہ بنائیں

# review: pending
vouchers-field-amount = رقم

# review: pending
vouchers-fill-full = پورا بقایا

# review: pending
vouchers-fill-oldest = سب سے پرانا بل

# review: pending
vouchers-field-collector = اگراہی والا

# review: pending
vouchers-field-reason-lent = { -naqd-udhar } کی وجہ

# review: pending
vouchers-reason-lent-hint = یہ رقم اس کے بلوں میں شامل ہو جاتی ہے اور اس کے باقی ذمے کے ساتھ { -agrahi } میں آتی ہے۔

# review: pending
vouchers-field-expected = متوقع مال یا موسم

# review: pending
vouchers-field-expected-hint = مثلاً آم کا موسم۔

# review: pending
vouchers-field-narration = تفصیل

# review: pending
vouchers-field-edit-reason = تبدیلی کی وجہ

# review: pending
vouchers-edit-reason-hint = لکھنا ضروری نہیں۔ یہ وجہ تبدیلی کے ریکارڈ میں رہتی ہے۔

## What the backend says about the entry on the form (PRD FR-6.7, FR-7.68, FR-15.5, FR-15.12).

# review: pending
vouchers-current-balance = موجودہ بقایا

# review: pending
vouchers-new-balance = نیا بقایا

# review: pending
vouchers-return-limit = زیادہ سے زیادہ واپسی

# review: pending
vouchers-extra-credit-note = یہ رقم اس کے سب بلوں سے زیادہ ہے۔ زائد رقم اس کی جمع رقم رہے گی اور اس کے اگلے بل میں خود لگ جائے گی:

# review: pending
vouchers-owes-settled-note = اتنی رقم سے اس کے ذمے دکان کی رقم ادا ہو جائے گی:

# review: pending
vouchers-advance-hint = اس { -beopari } کو دینے کے لیے کچھ باقی نہیں۔ کیا یہ { -peshgi } ہے؟

# review: pending
vouchers-make-advance = { -peshgi } بنائیں

## The open bills of a buyer, under the form (PRD FR-7.58).

# review: pending
vouchers-bills-title = کھلے بل

# review: pending
vouchers-bills-failed = کھلے بل نہیں کھل سکے

# review: pending
vouchers-bills-none = اس { -gahak } کا کوئی بل کھلا نہیں۔

# review: pending
vouchers-bills-open = کل کھلی رقم

# review: pending
vouchers-bills-oldest = سب سے پرانا بل

# review: pending
vouchers-bills-credit = بلوں سے الگ جمع رقم

# review: pending
vouchers-days = { $days ->
    [1] ایک دن
   *[other] { $days } دن
}

# review: pending
vouchers-effect-full = پورا

# review: pending
vouchers-effect-part = کچھ حصہ

## Where a bill or a payable posting came from. $number is the number of the record, when it has one.

# review: pending
vouchers-source-bikri = { -bikri } { $number }

# review: pending
vouchers-source-opening = ابتدائی بقایا

# review: pending
vouchers-source-cash-lent = { -naqd-udhar }

# review: pending
vouchers-source-receipt = { -wasooli } { $number }

# review: pending
vouchers-source-general = { -general } { $number }

# review: pending
vouchers-source-purchase = خریداری { $number }

# A fruit credit on a seller (PRD FR-16.11, FR-7.59): $number is the پھل number.
# review: pending
vouchers-source-fruit = { -phal } { $number }

# review: pending
vouchers-source-other = دیگر اندراج

## What is payable to a seller, under the form (PRD FR-7.59).

# review: pending
vouchers-payables-title = ادائیگی باقی

# review: pending
vouchers-payables-failed = ادائیگی باقی کی رقم نہیں کھل سکی

# review: pending
vouchers-payables-none = اس { -beopari } کو دینے کے لیے کچھ باقی نہیں۔

# review: pending
vouchers-payables-open = کل ادائیگی باقی

# review: pending
vouchers-payables-owes = { -beopari } کے ذمے

## The advances of a seller, under his party card (PRD FR-7.60).

# review: pending
vouchers-advances-title = اس { -beopari } کا { -peshgi-khata }

# review: pending
vouchers-advances-none = اس { -beopari } کو ابھی کوئی { -peshgi } نہیں دی گئی۔

# review: pending
vouchers-advances-open = { -peshgi } باقی

# review: pending
vouchers-advances-last = آخری { -peshgi }

# review: pending
vouchers-advances-failed = { -peshgi-khata } کی فہرست کھل نہیں سکی

## The columns of the panels and the list.

# review: pending
vouchers-col-date = تاریخ

# review: pending
vouchers-col-number = نمبر

# review: pending
vouchers-col-source = ذریعہ

# review: pending
vouchers-col-account = کھاتہ

# review: pending
vouchers-col-amount = رقم

# review: pending
vouchers-col-bill = بل کی رقم

# review: pending
vouchers-col-credit = جمع

# review: pending
vouchers-col-paid = ادا شدہ

# review: pending
vouchers-col-adjusted = منہا شدہ

# review: pending
vouchers-col-open = باقی

# review: pending
vouchers-col-days = دن

# review: pending
vouchers-col-settles = یہ رقم نمٹائے گی

# review: pending
vouchers-col-returns = یہ رقم منہا کرے گی

# review: pending
vouchers-col-state = حالت

# review: pending
vouchers-state-saved = محفوظ

# review: pending
vouchers-state-cancelled = منسوخ

# A voucher found by the global search that was cancelled.
# review: pending
vouchers-hit-cancelled = منسوخ

## The list under the form.

# review: pending
vouchers-list-title-receipt = وصولیوں کی فہرست

# review: pending
vouchers-list-title-payment = ادائیگیوں کی فہرست

# review: pending
vouchers-list-title-advance = { -peshgi-khata } کی فہرست

# review: pending
vouchers-list-label = واؤچروں کی فہرست

# review: pending
vouchers-list-from = از

# review: pending
vouchers-list-to = تا

# review: pending
vouchers-list-account = کھاتہ

# review: pending
vouchers-list-failed = فہرست کھل نہیں سکی

# review: pending
vouchers-list-empty-receipt = آج کوئی وصولی نہیں

# review: pending
vouchers-list-empty-payment = آج کوئی ادائیگی نہیں

# review: pending
vouchers-list-empty-advance = آج کوئی { -peshgi } نہیں

# $key is the key that saves.
# review: pending
vouchers-list-empty-help = اوپر فارم بھریں اور { $key } دبائیں۔

# review: pending
vouchers-list-no-match = ان تاریخوں اور کھاتے کا کوئی واؤچر نہیں

# review: pending
vouchers-list-no-match-help = تاریخیں یا کھاتہ بدل کر دیکھیں۔

# review: pending
vouchers-list-clear = آج کی فہرست دکھائیں

# review: pending
vouchers-list-select-help = کسی واؤچر پر ↑ ↓ سے جائیں اور Enter دبا کر اسے تبدیل کریں۔

## A saved voucher that cannot be changed (PRD FR-15.3, FR-6.6).

# review: pending
vouchers-view-cancelled = یہ واؤچر منسوخ ہو چکا ہے۔ وجہ:

# review: pending
vouchers-view-closed = اس واؤچر کا دن بند ہو چکا ہے، اس لیے اسے تبدیل نہیں کیا جا سکتا۔ مالک دن دوبارہ کھولے تو تبدیلی ہو سکتی ہے۔

# review: pending
vouchers-other-screen = یہ واؤچر کسی اور اسکرین کا ہے

# review: pending
vouchers-other-screen-help = اسے { -rokar } میں کھولیں۔

# review: pending
vouchers-other-screen-open = { -rokar } کھولیں

## Cancelling a voucher (PRD FR-7.8, FR-15.3). $number is its number, $amount its amount, $name the party.

# review: pending
vouchers-cancel-title = یہ واؤچر منسوخ کریں؟

# review: pending
vouchers-cancel-text-receipt = { -wasooli } { $number } (<num>{ $amount }</num>، { $name }) منسوخ ہو جائے گی۔ { -rokar } اور اس کا کھاتہ پہلے جیسے ہو جائیں گے، اور جو بل اس سے نمٹے تھے وہ دوبارہ کھل جائیں گے۔

# review: pending
vouchers-cancel-text-payment = { -adaigi } { $number } (<num>{ $amount }</num>، { $name }) منسوخ ہو جائے گی۔ { -rokar } اور اس کا کھاتہ پہلے جیسے ہو جائیں گے۔ اگر یہ { -gahak } کو { -naqd-udhar } تھا تو اس کا بل بھی ختم ہو جائے گا۔

# review: pending
vouchers-cancel-text-advance = { -peshgi-khata } { $number } (<num>{ $amount }</num>، { $name }) منسوخ ہو جائے گا۔ { -rokar } اور اس کا کھاتہ پہلے جیسے ہو جائیں گے، اور جو رقم اس { -peshgi } سے منہا ہو چکی تھی وہ دوبارہ { -beopari } کو دینی ہوگی۔

# review: pending
vouchers-cancel-reason = وجہ

# review: pending
vouchers-cancel-confirm = واؤچر منسوخ کریں

# review: pending
vouchers-cancel-confirm-cash = ہاں، منسوخ کریں

## The print box of a voucher (PRD FR-15.2).

# $number is the number in its series.
# review: pending
vouchers-print-title = واؤچر { $number } پرنٹ کریں

## اگراہی فہرست (PRD FR-15.6, FR-7.61).

# review: pending
vouchers-collection-title = { -agrahi } فہرست

# review: pending
vouchers-collection-help = اس دن جن گاہکوں سے وصولی کرنی ہے ان کی فہرست۔ صرف وصول ہونے والی رقم لکھیں اور سب ایک ساتھ محفوظ کریں۔ خالی یا 0 والی لائن چھوڑ دی جاتی ہے۔

# review: pending
vouchers-collection-back = { -wasooli } پر واپس

# $count is the number of buyers on the list.
# review: pending
vouchers-collection-count = { $count ->
    [1] فہرست میں ایک { -gahak } ہے۔
   *[other] فہرست میں { $count } { -gahak } ہیں۔
}

# review: pending
vouchers-collection-total = کل ذمے

# review: pending
vouchers-collection-col-group = اگراہی والا اور شہر

# review: pending
vouchers-collection-col-dues = ذمے

# review: pending
vouchers-collection-col-received = وصول رقم

# review: pending
vouchers-collection-group-separator = {"، "}

# review: pending
vouchers-collection-save = سب محفوظ کریں

# $count is the number of receipts saved; $refused the number that were not saved.
# review: pending
vouchers-collection-saved = { $count ->
    [1] ایک وصولی محفوظ ہو گئی
   *[other] { $count } وصولیاں محفوظ ہو گئیں
}

# review: pending
vouchers-collection-some-refused = { $count ->
    [1] ایک وصولی محفوظ ہو گئی
   *[other] { $count } وصولیاں محفوظ ہو گئیں
}، { $refused } محفوظ نہیں ہو سکیں۔ وجہ لائن میں دیکھیں۔

# review: pending
vouchers-collection-problems = یہ وصولیاں محفوظ نہیں ہوئیں

# review: pending
vouchers-collection-empty = اس دن کسی { -gahak } کی { -agrahi } مقرر نہیں

# review: pending
vouchers-collection-empty-help = { -gahak } کے کھاتے میں اگراہی کا دن لکھا ہو تو وہ اس دن کی فہرست میں آتا ہے۔ دوسری تاریخ، اگراہی والا یا شہر چن کر دیکھیں۔

# review: pending
vouchers-collection-failed = { -agrahi } فہرست کھل نہیں سکی

# review: pending
vouchers-collection-grid-label = { -agrahi } کی فہرست

# $key is the key that saves and prints, for example Ctrl+Enter.
# review: pending
vouchers-print-help = یہاں جو پرچی چنیں گے، وہی واؤچر محفوظ کرتے وقت { $key } سے بھی پرنٹ ہوگی۔

# This row is not touched by the amount typed (a table cell, read by a screen reader).
# review: pending
vouchers-effect-none = کچھ نہیں

# F9 pressed before anything was saved: the slip is made from a saved voucher.
# review: pending
vouchers-print-save-first = پرنٹ کرنے کے لیے پہلے واؤچر محفوظ کریں۔

## The waiting list: several vouchers entered one after another and saved together (PRD FR-15.1).

# review: pending
vouchers-add-to-list = فہرست میں رکھیں

# review: pending
vouchers-waiting-title = محفوظ ہونا باقی

# review: pending
vouchers-waiting-help = { $key } دبانے پر یہ سب ایک ایک کر کے محفوظ ہوں گے۔ جو واؤچر محفوظ نہ ہو سکے وہ وجہ کے ساتھ یہیں رہے گا۔

# review: pending
vouchers-waiting-count = { $count ->
    [one] ایک واؤچر محفوظ ہونا باقی
   *[other] { $count } واؤچر محفوظ ہونا باقی
}

# review: pending
vouchers-inactive-hint = پہلے اس کھاتے کو فعال کریں ({ -index } میں کھاتہ کھول کر)، پھر واؤچر دوبارہ لکھیں۔

# review: pending
vouchers-truncated-note = { $total } میں سے سب سے پرانے { $shown } دکھائے گئے ہیں۔

# review: pending
vouchers-col-type = قسم

# review: pending
vouchers-state-waiting = محفوظ ہونا باقی

# review: pending
vouchers-state-declined = محفوظ نہیں ہوا: { -rokar } کے سوال پر آپ نے واپس جانا چنا

# review: pending
vouchers-remove = فہرست سے ہٹائیں

# review: pending
vouchers-saved-many = { $count } واؤچر محفوظ ہو گئے

# review: pending
vouchers-some-refused = { $count ->
    [one] ایک واؤچر محفوظ نہیں ہو سکا۔ فہرست میں وجہ دیکھیں۔
   *[other] { $count } واؤچر محفوظ نہیں ہو سکے۔ فہرست میں وجہ دیکھیں۔
}

## The number of a voucher of the روکڑ area (plan 17): its series letters and its number.

# review: pending
vouchers-number-shop-expense = خ-{ $number }

# review: pending
vouchers-number-personal-expense = ذ-{ $number }

# review: pending
vouchers-number-personal-return = ذو-{ $number }

# review: pending
vouchers-number-money-lent = ر-{ $number }

# review: pending
vouchers-number-money-lent-return = رو-{ $number }

# review: pending
vouchers-number-capital-in = س-{ $number }

# review: pending
vouchers-number-capital-out = سو-{ $number }

# review: pending
vouchers-number-khata-out = کد-{ $number }

# review: pending
vouchers-number-khata-in = کا-{ $number }

# review: pending
vouchers-number-general = ج-{ $number }

## اگراہی فہرست پرنٹ کرنا (plan 17e, PRD FR-15.6).

# The title of the print box of the collection list.
# review: pending
vouchers-collection-print-title = { -agrahi } فہرست پرنٹ کریں

# $date is the day of the list, for example 10-10-2026.
# review: pending
vouchers-collection-print-help = { $date } کی { -agrahi } فہرست، جیسی اسکرین پر ہے ویسی ہی پرنٹ ہوگی۔ زبان اور پرنٹ کی جگہ چنیں۔

# The hint inside the search box of the collection list (the code, the name or the phone of a buyer).
# review: pending
vouchers-collection-search = گاہک کا کوڈ، نام یا فون لکھیں
