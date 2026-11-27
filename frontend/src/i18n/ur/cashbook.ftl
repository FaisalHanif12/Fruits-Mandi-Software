### The روکڑ area (plan 17, PRD 16): the cash book, the ONE خرچہ screen (دکان خرچہ, ذاتی خرچہ and its واپسی), ادھار رقم and الگ کھاتہ.
### Business words come from glossary.ftl only. Urdu says exactly what English says (NFR-URD-29).

## The cash book (PRD 16.1, FR-16.1, FR-19.11). There is no box to type counted cash.

# review: pending
cashbook-title = { -rokar }

# review: pending
cashbook-help = دکان کا سارا نقد ایک جگہ: ایپ کے ہر حصے سے آنے اور جانے والی ہر رقم، اور ہر ایک کے بعد موجود روکڑ۔

# review: pending
cashbook-new-expense = نیا { -dukan-kharcha }

# review: pending
cashbook-filter-from = از

# review: pending
cashbook-filter-to = تا

# review: pending
cashbook-closing = آخر میں روکڑ

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
cashbook-col-narration = تفصیل

# review: pending
cashbook-col-in = آمد

# review: pending
cashbook-col-out = نکاسی

# review: pending
cashbook-col-running = بعد میں روکڑ

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
cashbook-col-partner = شراکت دار

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
cashbook-col-days-past = تاخیر کے دن

# review: pending
cashbook-col-lent = دی گئی

# review: pending
cashbook-col-balance = بقایا

# review: pending
cashbook-state-cancelled = منسوخ

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

# The entry is on a linked khata or a loan linked to a partner or the shop: what the save will do to each partner's total.
# review: pending
cashbook-linked-parts-title = ہر شراکت دار کی { -kul-raqam } پر اس کا اثر

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
cashbook-takes-title = شراکت دار کی لی ہوئی رقم

# review: pending
cashbook-takes-label = شراکت دار کی لی ہوئی رقم

# review: pending
cashbook-takes-empty = اس شراکت دار نے کوئی رقم نہیں لی

# review: pending
cashbook-takes-empty-help = جو رقم وہ اپنے لیے لے اور جو واپس کرے وہ یہاں لکھی جاتی ہے۔

# review: pending
cashbook-takes-failed = شراکت دار کی لی ہوئی رقم کھل نہیں سکی

# The list under the form with no partner chosen (plan 98): every partner's entries of the type.
# review: pending
cashbook-all-takes-title = شراکت داروں کی لی ہوئی رقم

# review: pending
cashbook-all-failed = شراکت داروں کی رقم کھل نہیں سکی

# review: pending
cashbook-all-takes-empty = اس مدت میں کسی شراکت دار نے کوئی رقم نہیں لی

# review: pending
cashbook-all-empty-help = اوپر فارم بھر کر محفوظ کریں تو یہاں آ جائے گا، یا شراکت دار چنیں تو صرف اس کی رقم نظر آئے گی۔

# review: pending
cashbook-take-opening = ابتدائی بقایا

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
cashbook-khata-title = اس کھاتے کا حساب

# review: pending
cashbook-khata-label = اس کھاتے کا حساب

# review: pending
cashbook-khata-opening = شروع میں بقایا

# review: pending
cashbook-khata-closing = اب بقایا

# review: pending
cashbook-khata-sign-hint = مثبت: دکان کے پاس اس کھاتے کی رقم ہے۔ منفی: اس کھاتے نے لائی ہوئی رقم سے زیادہ رقم لی ہے۔ اندراج کے بعد کے بقایا میں { -jama } مثبت ہے اور { -banaam } منفی۔

# review: pending
cashbook-khata-empty = اس کھاتے میں کوئی اندراج نہیں

# review: pending
cashbook-khata-empty-help = دی گئی اور آئی ہوئی رقم وجہ کے ساتھ یہاں لکھی جائے گی۔

# review: pending
cashbook-khata-failed = کھاتے کے اندراجات کھل نہیں سکے

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
cashbook-type-capital-in = { -sarmaya } جمع

# review: pending
cashbook-type-capital-out = { -sarmaya } واپسی

# The tick of the lists of money taken and money lent: a take or a loan that is fully returned is
# left out until it is on.
# review: pending
cashbook-show-returned = واپس شدہ بھی دکھائیں

# Shown above the list when it is sorted by anything but the day: the cash after a line needs the order of days.
# review: pending
cashbook-running-date-only = ہر اندراج کے بعد کی روکڑ صرف تاریخ کی ترتیب میں نظر آتی ہے۔

## An entry of the روکڑ area changed (plan 44, PRD FR-7.70).

# $number: the number of the entry, for example "E-3".
# review: pending
cashbook-edited = { $number } میں تبدیلی ہو گئی

# The list of saved journal entries under the جنرل form (plan 44, PRD FR-7.70).
# review: pending
cashbook-general-list-title = محفوظ شدہ { -general } اندراج

# review: pending
cashbook-general-list-label = محفوظ شدہ { -general } اندراج

# review: pending
cashbook-general-list-failed = محفوظ شدہ { -general } اندراج کھل نہیں سکے

# review: pending
cashbook-general-list-empty = ان تاریخوں میں کوئی { -general } اندراج نہیں

# review: pending
cashbook-general-list-empty-help = اوپر سطریں بھر کر محفوظ کریں تو یہاں آ جائے گا۔ دوسرا مہینہ دیکھنے کے لیے نیچے کے بٹن استعمال کریں۔

## The link of a { -alag-khata } and of a person who is lent money, and the type of that person (plan 76 points 5 and 10, PRD FR-16.6, FR-16.13)

# review: pending
cashbook-link-label = جڑا ہوا

# review: pending
cashbook-link-for = کس کی طرف سے

# review: pending
cashbook-link-kind-none = کوئی نہیں (فرم کا اپنا)

# review: pending
cashbook-link-kind-shop = دکان (حصوں کے مطابق تقسیم)

# review: pending
cashbook-link-kind-partner = ایک { -shirakat-dar }

# review: pending
cashbook-link-partner = { -shirakat-dar }

# review: pending
cashbook-link-hint-khata = نکالی گئی رقم جن شراکت داروں سے جڑی ہو ان کی { -kul-raqam } لی گئی رقم کی طرح کم کرتی ہے، اور واپسی اسے بڑھاتی ہے۔ یہ کبھی آمدن یا خرچہ نہیں بنتی۔

# review: pending
cashbook-link-hint-person = { -shirakat-dar } سے جڑی ہو تو دی گئی رقم اس شراکت دار کی { -kul-raqam } کم کرتی ہے۔ دکان سے جڑی ہو تو رقم پھر بھی دکان کو واپس کرنی ہے اور کسی شراکت دار کی کل رقم نہیں بدلتی۔

# review: pending
cashbook-link-locked = جب تک بقایا صفر نہیں ہوتا، جوڑ بدلا نہیں جا سکتا۔

# review: pending
cashbook-link-change = جوڑ بدلیں

# review: pending
cashbook-link-saved = جوڑ محفوظ ہو گیا

# review: pending
cashbook-person-type = قسم

# review: pending
cashbook-person-type-staff = عملہ

# review: pending
cashbook-person-type-garden-owner = باغ کا مالک

# review: pending
cashbook-person-type-trip-person = { -market-safar } کا شخص

# review: pending
cashbook-person-type-other = دیگر

# review: pending
cashbook-person-change = قسم یا جوڑ بدلیں

# review: pending
cashbook-person-saved = شخص کی تفصیل محفوظ ہو گئی

## The page behind the dashboard tile of garden owners and others (plan 76 point 10, PRD FR-16.13)

# review: pending
cashbook-dues-title = باغوں کے مالکوں / دوسروں کے ذمے

# review: pending
cashbook-dues-help = ان لوگوں کو دی گئی { -adhar-raqam } جو عملے میں نہیں (باغ کے مالک، { -market-safar } والے اور دوسرے) اور جو واپس آئی، ایک وقت میں ایک مہینہ۔ اعداد ٹائل والے ہی ہیں۔


# review: pending
cashbook-dues-filter-type = قسم

# review: pending
cashbook-dues-filter-link = جڑا ہوا

# review: pending
cashbook-dues-type-default = عملے کے سوا سب

# review: pending
cashbook-dues-link-all = سب

# review: pending
cashbook-dues-link-shop = دکان

# review: pending
cashbook-dues-link-partner = ایک { -shirakat-dar }

# review: pending
cashbook-dues-open = ابھی باقی

# review: pending
cashbook-dues-people = جن کے ذمے رقم ہے

# review: pending
cashbook-dues-given = اس مہینے دی گئی

# review: pending
cashbook-dues-returned = اس مہینے واپس آئی

# review: pending
cashbook-dues-people-title = جن کے ذمے رقم باقی ہے

# review: pending
cashbook-dues-people-label = وہ لوگ جن کے ذمے رقم ہے

# review: pending
cashbook-dues-people-empty = کسی کے ذمے کچھ نہیں

# review: pending
cashbook-dues-people-empty-help = کسی کو رقم دی جائے اور وہ واپس نہ کرے تو وہ یہاں آ جاتا ہے۔

# review: pending
cashbook-dues-entries-title = مہینے کے اندراجات

# review: pending
cashbook-dues-entries-label = مہینے میں دی گئی اور واپس آئی رقم

# review: pending
cashbook-dues-failed = ذمے کی رقم پڑھی نہیں جا سکی

# review: pending
cashbook-dues-empty = اس مہینے میں نہ کوئی رقم دی گئی نہ واپس آئی

# review: pending
cashbook-dues-empty-help = دوسرے مہینے کے لیے نیچے کے بٹن استعمال کریں۔

# review: pending
cashbook-dues-col-person = نام

# review: pending
cashbook-dues-col-type = قسم

# review: pending
cashbook-dues-col-link = جڑا ہوا

# review: pending
cashbook-dues-col-kind = اندراج

# review: pending
cashbook-dues-col-amount = رقم

# review: pending
cashbook-dues-col-lent = دی گئی

# review: pending
cashbook-dues-col-returned = واپس

# review: pending
cashbook-dues-col-open = باقی

# review: pending
cashbook-dues-kind-given = دی گئی

# review: pending
cashbook-dues-kind-returned = واپس

# The short words of a link, for a list or a card (the choice box has the long ones above).
# review: pending
cashbook-link-word-none = کوئی نہیں

# review: pending
cashbook-link-word-shop = دکان

# review: pending
cashbook-link-word-partner = ایک { -shirakat-dar }

## The روکڑ بک (PRD FR-16.14 v1.60, plan 99): the cards, the type filter, the switch of old and cancelled
## postings and the summary by type.

# review: pending
cashbook-card-capital = کل { -sarmaya }

# review: pending
cashbook-filter-type = قسم

# review: pending
cashbook-filter-type-all = سب

# review: pending
cashbook-show-reversed = پرانے اور منسوخ بھی دکھائیں

# review: pending
cashbook-running-all-types-only = ہر اندراج کے بعد کی { -rokar } صرف تمام اقسام کے ساتھ نظر آتی ہے۔

# review: pending
cashbook-empty-type = ان دنوں میں اس قسم کا کوئی اندراج نہیں

# review: pending
cashbook-empty-type-help = تمام اقسام دیکھیں، یا دوسرے دن چنیں۔

# review: pending
cashbook-empty-type-action = تمام اقسام دکھائیں

# review: pending
cashbook-summary-title = قسم کے حساب سے آمد اور نکاسی

# review: pending
cashbook-summary-col-type = قسم

# review: pending
cashbook-summary-col-count = اندراجات

## The types of the filter and of the summary.

# review: pending
cashbook-group-receipt = { -wasooli }

# review: pending
cashbook-group-payment = { -adaigi }

# review: pending
cashbook-group-advance = { -peshgi }

# review: pending
cashbook-group-lent = { -adhar-raqam } اور { -naqd-udhar }

# review: pending
cashbook-group-expense = { -kharcha }

# review: pending
cashbook-group-bikri = { -bikri }

# review: pending
cashbook-group-garden = { -bagh }

# review: pending
cashbook-group-trip = { -safar }

# review: pending
cashbook-group-capital = { -sarmaya }

# review: pending
cashbook-group-khata = { -alag-khata }

# review: pending
cashbook-group-opening = { -ibtidai-baqaya }

# review: pending
cashbook-group-other = دیگر

## What a row of the book is (the backend says the kind).

# review: pending
cashbook-row-kind-opening-cash = ابتدائی { -rokar }

# review: pending
cashbook-row-kind-opening-capital = ابتدائی { -sarmaya }

# review: pending
cashbook-row-kind-opening-balance = { -ibtidai-baqaya }

# review: pending
cashbook-row-kind-receipt = { -wasooli }

# review: pending
cashbook-row-kind-payment = { -adaigi }

# review: pending
cashbook-row-kind-advance = { -peshgi }

# review: pending
cashbook-row-kind-buyer-advance = { -peshgi }

# review: pending
cashbook-row-kind-cash-lent = { -naqd-udhar }

# review: pending
cashbook-row-kind-money-lent = { -adhar-raqam }

# review: pending
cashbook-row-kind-money-lent-return = { -adhar-raqam } { -wapsi }

# review: pending
cashbook-row-kind-shop-expense = { -dukan-kharcha }

# review: pending
cashbook-row-kind-personal-expense = { -zati-kharcha }

# review: pending
cashbook-row-kind-personal-return = { -zati-kharcha } { -wapsi }

# review: pending
cashbook-row-kind-bikri-paid = { -mauqa-par-jama }

# review: pending
cashbook-row-kind-bikri-seller-cash = { -naqd-adaigi }

# review: pending
cashbook-row-kind-bikri-advance = { -peshgi }

# review: pending
cashbook-row-kind-garden-installment = { -bagh } کی { -qist }

# review: pending
cashbook-row-kind-garden-cost = { -bagh } خرچ

# review: pending
cashbook-row-kind-garden-other-sale = { -dusri-dukan } میں فروخت

# review: pending
cashbook-row-kind-garden-refund = رعایت { -wapsi }

# review: pending
cashbook-row-kind-purchase = { -khareedari }

# review: pending
cashbook-row-kind-trip-cost = { -safar } خرچ

# review: pending
cashbook-row-kind-trip-other-sale = { -dusri-dukan } میں فروخت

# review: pending
cashbook-row-kind-capital-in = { -sarmaya } جمع

# review: pending
cashbook-row-kind-capital-out = { -sarmaya } { -wapsi }

# review: pending
cashbook-row-kind-khata-out = { -alag-khata }: رقم دی

# review: pending
cashbook-row-kind-khata-in = { -alag-khata }: رقم آئی

# review: pending
cashbook-row-kind-general = { -general }

# review: pending
cashbook-row-kind-fruit = { -phal }

# review: pending
cashbook-row-kind-other = دیگر اندراج

## How a row is put together: its kind and number, then the party, garden, trip, head or partner. The pieces are
## joined by the screen with these separators (no finished message is ever passed into another one).

# $number: the number of the sale; the words of its cash follow after a separator.
# review: pending
cashbook-row-bikri = { -bikri } { $number }

# review: pending
cashbook-row-name-gap = ،{ " " }

# $count: how many more parties the row has than the names shown.
# review: pending
cashbook-row-more = اور { $count } مزید

# Written before an entry's own words.
# review: pending
cashbook-row-reversal = الٹا اندراج:{ " " }

# review: pending
cashbook-row-reversed = پرانا اندراج:{ " " }

## The one-time fix: old opening balances of parties into روکڑ (PRD FR-10.9 v1.60, plan 99).

# review: pending
cashbook-openings-offer = <num>{ $count }</num> کھاتوں کا پرانا { -ibtidai-baqaya } ابھی { -rokar } میں درج نہیں۔

# review: pending
cashbook-openings-button = پرانے ابتدائی بقایا { -rokar } میں درج کریں

# review: pending
cashbook-openings-title = پرانے ابتدائی بقایا { -rokar } میں

# review: pending
cashbook-openings-failed = پرانے { -ibtidai-baqaya } نہیں کھل سکے

# review: pending
cashbook-openings-nothing = پرانے سب { -ibtidai-baqaya } { -rokar } میں درج ہیں۔

# review: pending
cashbook-openings-explain = { -gahakon }، { -beopariyon }، { -adhar-raqam } اور { -alag-khata } کے یہ { -ibtidai-baqaya } پہلے ابتدائی فرق میں درج ہوئے تھے۔ ہر ایک اپنی تاریخ پر الٹ کر { -rokar } کے ساتھ دوبارہ درج ہوگا: { -banaam } والی رقم { -rokar } سے جائے گی، اور { -jama } والی رقم { -rokar } میں آئے گی۔ کسی کا بقایا اور { -shirakat-dar } کی { -kul-raqam } نہیں بدلے گی۔

# review: pending
cashbook-openings-cash-before = { -rokar } اس وقت

# review: pending
cashbook-openings-cash-after = اس کے بعد { -rokar }

# review: pending
cashbook-openings-difference-before = ابتدائی فرق اس وقت

# review: pending
cashbook-openings-difference-after = اس کے بعد ابتدائی فرق

# review: pending
cashbook-openings-reopens = <num>{ $date }</num> سے بند دن کھلیں گے اور خود دوبارہ بند ہو جائیں گے۔

# review: pending
cashbook-openings-below-zero = اس کے بعد { -rokar } صفر سے کم ہوگی۔ محفوظ کرنے سے پہلے دوبارہ پوچھا جائے گا۔

# review: pending
cashbook-openings-table = وہ کھاتے جن کا { -ibtidai-baqaya } { -rokar } میں جائے گا

# review: pending
cashbook-openings-col-code = کوڈ

# review: pending
cashbook-openings-col-account = { -khata }

# review: pending
cashbook-openings-col-kind = قسم

# review: pending
cashbook-openings-col-out = { -rokar } سے ({ -banaam })

# review: pending
cashbook-openings-col-in = { -rokar } میں ({ -jama })

# review: pending
cashbook-openings-kind-seller = { -beopari }

# review: pending
cashbook-openings-kind-buyer = { -gahak }

# review: pending
cashbook-openings-kind-money-lent = { -adhar-raqam }

# review: pending
cashbook-openings-kind-separate-khata = { -alag-khata }

# review: pending
cashbook-openings-ask-title = پرانے ابتدائی بقایا { -rokar } میں درج کریں؟

# review: pending
cashbook-openings-ask-text = وجہ اور مالک کا { -dastakhat } لکھیں۔

# review: pending
cashbook-openings-reason = وجہ

# review: pending
cashbook-openings-sign-hint = صرف مالک کے { -dastakhat } سے درج ہوگا۔

# review: pending
cashbook-openings-confirm = درج کریں

# review: pending
cashbook-openings-done = پرانے { -ibtidai-baqaya } { -rokar } میں درج ہو گئے

## The saved entries under the forms of the روکڑ area, 30 days at a time (plan 100 F, PRD FR-19.11).

# review: pending
cashbook-col-person = نام

# review: pending
cashbook-col-khata = { -alag-khata }

# review: pending
cashbook-entries-shop-title = { -dukan-kharcha } کے اندراج

# review: pending
cashbook-entries-lent-title = { -adhar-raqam } کے اندراج

# review: pending
cashbook-entries-khata-title = { -alag-khata } کے اندراج

# review: pending
cashbook-entries-failed = اندراج نہیں کھل سکے

# review: pending
cashbook-entries-empty = پچھلے 30 دن میں کوئی اندراج نہیں

# review: pending
cashbook-entries-empty-help = اوپر فارم بھر کر محفوظ کریں۔ پہلے کے دن دیکھنے کے لیے نیچے والے بٹن استعمال کریں۔

# review: pending
cashbook-entries-no-match = ان دنوں میں کوئی ملتا جلتا اندراج نہیں

# review: pending
cashbook-entries-no-match-help = دوسرے الفاظ سے تلاش کریں، یا دوسرے دن دیکھیں۔
