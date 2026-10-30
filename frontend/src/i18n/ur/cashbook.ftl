### The روکڑ area (plan 17, PRD 16): the cash book, the ONE خرچہ screen (دکان خرچہ, ذاتی خرچہ and its واپسی), ادھار رقم and الگ کھاتہ.
### Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The cash book (PRD 16.1, FR-16.1, FR-19.11). There is no box to type counted cash.

# review: pending
cashbook-title = { -rokar }

# review: pending
cashbook-help = دکان کا سارا نقد ایک جگہ: ایپ کے ہر حصے سے آنے اور جانے والی ہر رقم، اور ہر ایک کے بعد ہاتھ میں موجود نقد۔

# review: pending
cashbook-new-expense = نیا { -dukan-kharcha }

# review: pending
cashbook-filter-from = از

# review: pending
cashbook-filter-to = تا

# review: pending
cashbook-opening = شروع میں نقد

# review: pending
cashbook-closing = آخر میں نقد

# review: pending
cashbook-list-label = { -rokar }

# review: pending
cashbook-empty = ان دنوں میں کوئی نقد نہیں آیا یا گیا

# review: pending
cashbook-empty-help = دوسرے دن چنیں، یا کوئی خرچہ درج کریں۔

# review: pending
cashbook-load-failed = { -rokar } کھل نہیں سکی

## Columns of the lists.

# review: pending
cashbook-col-date = تاریخ

# review: pending
cashbook-col-kind = اندراج

# review: pending
cashbook-col-account = کھاتہ

# review: pending
cashbook-col-narration = تفصیل

# review: pending
cashbook-col-in = آمد

# review: pending
cashbook-col-out = رخصت

# review: pending
cashbook-col-running = بعد میں نقد

# review: pending
cashbook-col-number = نمبر

# review: pending
cashbook-col-head = مد

# review: pending
cashbook-col-amount = رقم

# review: pending
cashbook-col-state = حالت

# review: pending
cashbook-col-reason = وجہ

# review: pending
cashbook-col-taken-by = لینے والا

# review: pending
cashbook-col-return-by = واپسی کی تاریخ

# review: pending
cashbook-col-taken = لی گئی

# review: pending
cashbook-col-returned = واپس

# review: pending
cashbook-col-open = باقی

# review: pending
cashbook-col-days-past = دن گزرے

# review: pending
cashbook-col-lent = دی گئی

# review: pending
cashbook-col-balance = بقایا

# review: pending
cashbook-state-cancelled = منسوخ

## What a row of the cash book is. The kind is the stored posting type, written with hyphens.

# $kind: the word for the kind; $number: its number.
# review: pending
cashbook-kind-number = { $kind } { $number }

# review: pending
cashbook-kind-bikri-first = { -bikri }

# review: pending
cashbook-kind-bikri-later = بعد کی بکری

# review: pending
cashbook-kind-reversal = الٹا اندراج

# review: pending
cashbook-kind-receipt = { -wasooli }

# review: pending
cashbook-kind-payment = { -adaigi }

# review: pending
cashbook-kind-advance = { -peshgi-khata }

# review: pending
cashbook-kind-shop-expense = { -dukan-kharcha }

# review: pending
cashbook-kind-personal-expense = { -zati-kharcha }

# review: pending
cashbook-kind-personal-return = { -zati-kharcha } { -wapsi }

# review: pending
cashbook-kind-money-lent = { -adhar-raqam }

# review: pending
cashbook-kind-money-lent-return = { -adhar-raqam } { -wapsi }

# review: pending
cashbook-kind-capital-in = سرمایہ جمع

# review: pending
cashbook-kind-capital-out = سرمایہ واپسی

# review: pending
cashbook-kind-khata-out = { -alag-khata }: رقم دی

# review: pending
cashbook-kind-khata-in = { -alag-khata }: رقم آئی

# review: pending
cashbook-kind-purchase = { -khareedari }

# review: pending
cashbook-kind-trip-cost = سفر خرچ

# review: pending
cashbook-kind-garden-payment = باغ کی ادائیگی

# review: pending
cashbook-kind-venture-receipt = دوسری دکان میں فروخت

# review: pending
cashbook-kind-garden-close = باغ بند

# review: pending
cashbook-kind-fruit-taken = پھل

# review: pending
cashbook-kind-general = { -general }

# review: pending
cashbook-kind-opening = ابتدائی بیلنس

# review: pending
cashbook-kind-month-close = مہینہ بند

## Shared by the entry screens.

# review: pending
cashbook-new = نیا اندراج

# review: pending
cashbook-field-date = تاریخ

# review: pending
cashbook-field-amount = رقم

# review: pending
cashbook-field-narration = تفصیل

# review: pending
cashbook-field-reason = وجہ

# review: pending
cashbook-field-return-by = واپسی کی تاریخ

# review: pending
cashbook-preview-failed = اس اندراج کا اثر معلوم نہیں ہو سکا

# review: pending
cashbook-balance-now = اب بقایا

# review: pending
cashbook-balance-after = اس اندراج کے بعد بقایا

# $kind: the voucher type; $number: the number in its series, made by vouchers-number-<type>.
# review: pending
cashbook-saved = { $kind ->
    [shop_expense] { -dukan-kharcha } { $number } محفوظ ہو گیا
    [personal_expense] { -zati-kharcha } { $number } محفوظ ہو گیا
    [personal_return] { -zati-kharcha } { -wapsi } { $number } محفوظ ہو گئی
    [money_lent] { -adhar-raqam } { $number } محفوظ ہو گئی
    [money_lent_return] { -adhar-raqam } { -wapsi } { $number } محفوظ ہو گئی
    [khata_out] { -alag-khata }: رقم دی { $number } محفوظ ہو گئی
    [general] { -general } { $number } محفوظ ہو گیا
    [capital_in] { -sarmaya } جمع { $number } محفوظ ہو گیا
    [capital_out] { -sarmaya } واپسی { $number } محفوظ ہو گئی
   *[khata_in] { -alag-khata }: رقم آئی { $number } محفوظ ہو گئی
}

## The type lists (PRD FR-7.67).

# review: pending
cashbook-type-shop-expense = { -dukan-kharcha }

# review: pending
cashbook-type-personal-expense = { -zati-kharcha }

# review: pending
cashbook-type-personal-return = { -zati-kharcha } { -wapsi }

# review: pending
cashbook-type-money-lent = رقم دی

# review: pending
cashbook-type-money-lent-return = { -wapsi }

# review: pending
cashbook-type-khata-out = رقم دی

# review: pending
cashbook-type-khata-in = رقم آئی

# review: pending
cashbook-tab-expense = { -kharcha }

# review: pending
cashbook-tab-fruit = { -phal }

# review: pending
cashbook-tabs-label = { -kharcha } کے حصے

## The { -kharcha } screen (PRD 16.2, 16.3, FR-16.8, FR-7.68, FR-7.62).

# review: pending
cashbook-expense-title = { -kharcha }

# review: pending
cashbook-expense-help = پہلے قسم چنیں۔ { -dukan-kharcha } شراکت داروں میں ان کے حصوں کے مطابق بٹتا ہے۔ { -zati-kharcha } صرف ایک شراکت دار کے کھاتے میں جاتا ہے۔ فارم کے نیچے نظر آتا ہے کہ محفوظ کرنے سے کیا ہوگا۔

# review: pending
cashbook-expense-type = { -kharcha } کی قسم

# review: pending
cashbook-field-head = خرچہ مد

# review: pending
cashbook-field-partner = { -shirakat-dar }

# review: pending
cashbook-field-paid-to = ادا کیا

# review: pending
cashbook-field-taken-by = لینے والا

# review: pending
cashbook-heads-failed = خرچہ مدیں کھل نہیں سکیں

# review: pending
cashbook-partners-failed = شراکت دار کھل نہیں سکے

# review: pending
cashbook-preview-hint-shop = رقم لکھیں اور مد چنیں۔ محفوظ کرنے سے پہلے یہاں ہر شراکت دار کا حصہ نظر آئے گا۔

# review: pending
cashbook-preview-hint-personal = شراکت دار چنیں اور رقم لکھیں۔ محفوظ کرنے سے پہلے یہاں اس کی کل رقم اب اور بعد میں نظر آئے گی۔

# review: pending
cashbook-shares-title = ہر شراکت دار کا حصہ

# review: pending
cashbook-shares-none = اس تاریخ پر بانٹنے کے لیے کوئی حصے نہیں ہیں۔

# review: pending
cashbook-effect-title = شراکت دار پر اس کا اثر

# review: pending
cashbook-effect-none = اثر دیکھنے کے لیے شراکت دار چنیں۔

# review: pending
cashbook-effect-total-now = کل رقم اب

# review: pending
cashbook-effect-total-after = کل رقم بعد میں

# review: pending
cashbook-effect-open-now = لی گئی اور واپس نہ ہوئی، اب

# review: pending
cashbook-effect-open-after = لی گئی اور واپس نہ ہوئی، بعد میں

# review: pending
cashbook-others-not-affected = باقی شراکت داروں پر کوئی اثر نہیں پڑتا۔

# PRD FR-7.68. $name: the partner the "paid to" text matches.
# review: pending
cashbook-personal-hint = کیا یہ شراکت دار کا ذاتی خرچہ تو نہیں؟ { $name } شراکت دار ہے۔

# review: pending
cashbook-personal-make = { -zati-kharcha } بنائیں

## Repeat last month (PRD FR-7.62).

# review: pending
cashbook-repeat-open = پچھلے مہینے والے دہرائیں

# review: pending
cashbook-repeat-title = پچھلے مہینے کے باقاعدہ خرچے

# review: pending
cashbook-repeat-help = یہ مدیں پچھلے مہینے درج ہوئیں اور اس مہینے ابھی درج نہیں ہوئیں۔ ایک کو فارم میں بھریں، دیکھ لیں اور محفوظ کریں۔

# review: pending
cashbook-repeat-fill = فارم میں بھریں

# review: pending
cashbook-repeat-last-date = پچھلی بار: <num>{ $date }</num>

# review: pending
cashbook-repeat-total = ان کا میزان

# review: pending
cashbook-repeat-close = بند کریں

# review: pending
cashbook-repeat-empty = دہرانے کو کچھ نہیں

# review: pending
cashbook-repeat-empty-help = پچھلے مہینے کی ہر مد اس مہینے درج ہو چکی ہے، یا کوئی مد تھی ہی نہیں۔

# review: pending
cashbook-repeat-failed = پچھلے مہینے کے خرچے کھل نہیں سکے

## The list under the form.

# review: pending
cashbook-day-title = اس دن کے دکان خرچے

# review: pending
cashbook-day-label = اس دن کے دکان خرچے

# review: pending
cashbook-day-empty = اس دن کوئی دکان خرچہ نہیں

# review: pending
cashbook-day-empty-help = اوپر فارم بھر کر محفوظ کریں تو یہاں آ جائے گا۔

# review: pending
cashbook-day-failed = اس دن کے دکان خرچے کھل نہیں سکے

# review: pending
cashbook-takes-title = شراکت دار کی لی ہوئی رقم

# review: pending
cashbook-takes-label = شراکت دار کی لی ہوئی رقم

# review: pending
cashbook-takes-empty = اس شراکت دار نے کوئی رقم نہیں لی

# review: pending
cashbook-takes-empty-help = جو رقم وہ اپنے لیے لے اور جو واپس کرے وہ یہاں لکھی جاتی ہے۔

# review: pending
cashbook-takes-failed = شراکت دار کی لی ہوئی رقم کھل نہیں سکی

# review: pending
cashbook-takes-pick = شراکت دار چنیں

# review: pending
cashbook-takes-pick-help = اس کی لی ہوئی اور واپس کی ہوئی رقم یہاں نظر آئے گی۔

# review: pending
cashbook-take-opening = ابتدائی بیلنس

## The { -adhar-raqam } screen (PRD 16.5, FR-16.4, FR-16.5).

# review: pending
cashbook-lent-screen-title = { -adhar-raqam }

# review: pending
cashbook-lent-screen-help = وہ رقم جو دکان عملے یا کسی اور کو ادھار دے، اور جو وہ واپس کریں۔ واپسی نقد ہو سکتی ہے یا تنخواہ سے کٹوتی۔ نیا شخص شامل کرنے کے لیے نام کے خانے میں F4 دبائیں۔

# review: pending
cashbook-lent-type = اندراج کی قسم

# review: pending
cashbook-field-person = نام

# review: pending
cashbook-field-mode = واپسی کیسے

# review: pending
cashbook-mode-cash = نقد

# review: pending
cashbook-mode-salary = تنخواہ سے کٹوتی

# review: pending
cashbook-field-salary-head = تنخواہ کی خرچہ مد

# review: pending
cashbook-field-salary-head-hint = خالی چھوڑیں تو معمول کی تنخواہ والی مد لگے گی۔

# review: pending
cashbook-salary-head-default = تنخواہ (معمول)

# review: pending
cashbook-salary-note = کوئی نقد نہیں ہلتا۔ تنخواہ کا یہ حصہ دکان خرچہ گنا جاتا ہے اور شراکت داروں میں بٹتا ہے۔

# review: pending
cashbook-lent-title = اس شخص کی ادھار رقمیں

# review: pending
cashbook-lent-label = اس شخص کی ادھار رقمیں

# review: pending
cashbook-lent-balance = اس کے ذمے باقی

# review: pending
cashbook-lent-empty = اس شخص کی کوئی ادھار رقم نہیں

# review: pending
cashbook-lent-empty-help = اسے دی گئی رقم اور اس کی واپسی یہاں لکھی جائے گی۔

# review: pending
cashbook-lent-failed = اس شخص کی ادھار رقمیں کھل نہیں سکیں

# review: pending
cashbook-lent-pick = شخص چنیں

# review: pending
cashbook-lent-pick-help = اس کی ادھار رقمیں، واپسی اور باقی رقم یہاں نظر آئے گی۔

## The { -alag-khata } screen (PRD 16.6, FR-16.6).

# review: pending
cashbook-khata-screen-title = { -alag-khata }

# review: pending
cashbook-khata-screen-help = وہ رقم جس کا شراکت داروں، کمیشن یا منافع سے کوئی تعلق نہیں، مثلاً خاندانی کھاتہ۔ رقم دینے سے کھاتہ منفی ہوتا ہے، رقم آنے سے مثبت۔ نیا کھاتہ شامل کرنے کے لیے کھاتے کے خانے میں F4 دبائیں۔

# review: pending
cashbook-khata-type = اندراج کی قسم

# review: pending
cashbook-field-khata = { -alag-khata }

# review: pending
cashbook-khata-title = کھاتے کے اندراجات

# review: pending
cashbook-khata-label = کھاتے کے اندراجات

# review: pending
cashbook-khata-opening = شروع میں بقایا

# review: pending
cashbook-khata-closing = اب بقایا

# review: pending
cashbook-khata-sign-hint = مثبت: دکان کے پاس اس کھاتے کی رقم ہے۔ منفی: اس نے لایا کم اور لیا زیادہ۔ اندراج کے بعد کے بقایا میں { -jama } مثبت ہے اور { -banaam } منفی۔

# review: pending
cashbook-khata-empty = اس کھاتے میں کوئی اندراج نہیں

# review: pending
cashbook-khata-empty-help = دی گئی اور آئی ہوئی رقم وجہ کے ساتھ یہاں لکھی جائے گی۔

# review: pending
cashbook-khata-failed = کھاتے کے اندراجات کھل نہیں سکے

# review: pending
cashbook-khata-pick = کھاتہ چنیں

# review: pending
cashbook-khata-pick-help = اس کے اندراجات اور چلتا بقایا یہاں نظر آئے گا۔

# review: pending
cashbook-khata-kind-paid-out = رقم دی

# review: pending
cashbook-khata-kind-received = رقم آئی

# review: pending
cashbook-khata-kind-sale = مال کی فروخت

## Adding a person or a khata without leaving the screen (F4).

# review: pending
cashbook-add-person-title = نیا شخص

# review: pending
cashbook-add-person-help = ایسا شخص شامل کریں جو دکان سے ادھار رقم لے سکے۔ عملے کے فرد کو اس کے عملے کے ریکارڈ سے جوڑا جا سکتا ہے۔

# review: pending
cashbook-add-person-saved = شخص شامل ہو گیا

# review: pending
cashbook-add-khata-title = نیا { -alag-khata }

# review: pending
cashbook-add-khata-help = ایسا کھاتہ شامل کریں جو شراکت داروں، کمیشن اور منافع سے الگ رکھا جائے۔

# review: pending
cashbook-add-khata-saved = کھاتہ شامل ہو گیا

# review: pending
cashbook-add-name-ur = اردو نام

# review: pending
cashbook-add-name-en = انگریزی نام

# review: pending
cashbook-add-phone = فون

# review: pending
cashbook-add-staff = عملے کا ریکارڈ

# review: pending
cashbook-add-staff-hint = صرف اگر وہ دکان پر کام کرتا ہو۔

# review: pending
cashbook-add-note = نوٹ

## The جنرل screen (route /rokar/general, PRD 16.4).

# review: pending
cashbook-general-screen-title = { -general }

# review: pending
cashbook-general-screen-help = سطروں کا اندراج، ہر سطر پر کھاتہ اور { -banaam } یا { -jama } کی طرف رقم۔ دونوں طرف برابر ہوں تو ہی محفوظ ہو سکتا ہے۔

# review: pending
cashbook-general-lines-title = سطریں

# review: pending
cashbook-general-grid-label = { -general } اندراج کی سطریں

# review: pending
cashbook-general-add-line = سطر شامل کریں

# review: pending
cashbook-general-col-account = کھاتہ

# review: pending
cashbook-general-col-person = شخص

# review: pending
cashbook-general-totals-title = { -meezan }

# review: pending
cashbook-general-totals-hint = سطریں لکھیں۔ { -meezan } یہاں نظر آئے گی۔

# review: pending
cashbook-general-total-dr = { -banaam } کی { -meezan }

# review: pending
cashbook-general-total-cr = { -jama } کی { -meezan }

# review: pending
cashbook-general-difference = فرق ({ -banaam } منفی { -jama })

# review: pending
cashbook-general-balanced = دونوں طرف برابر ہیں۔ اندراج محفوظ ہو سکتا ہے۔

# review: pending
cashbook-general-not-balanced = دونوں طرف ابھی برابر نہیں ہیں۔ برابر ہونے تک اندراج محفوظ نہیں ہو سکتا۔

# review: pending
cashbook-general-split-title = ہر { -shirakat-dar } کا حصہ

# review: pending
cashbook-general-split-help = آمدن اور { -dukan-kharcha } کے کھاتوں کی سطریں شراکت داروں میں ان کے { -hissa } کے مطابق بٹتی ہیں۔

# review: pending
cashbook-general-people-title = ایک بار آنے والے لوگ

# review: pending
cashbook-general-people-help = { -pukhta-khata } کھاتے کی رقم ہر شخص کے الگ حساب سے رکھی جاتی ہے۔ ان میں سے ہر سطر کے لیے شخص چنیں۔

# $line: the number of the line; $account: the name of its account.
# review: pending
cashbook-general-person-of = سطر { $line } کا شخص: { $account }

# review: pending
cashbook-type-capital-in = { -sarmaya } جمع

# review: pending
cashbook-type-capital-out = { -sarmaya } واپسی

# The tick of the lists of money taken and money lent: a take or a loan that is fully returned is
# left out until it is on.
# review: pending
cashbook-show-returned = واپس شدہ بھی دکھائیں

# Shown above the list when it is sorted by anything but the day: the cash after a line needs the order of days.
# review: pending
cashbook-running-date-only = ہر اندراج کے بعد کی نقد صرف تاریخ کی ترتیب میں نظر آتی ہے۔
