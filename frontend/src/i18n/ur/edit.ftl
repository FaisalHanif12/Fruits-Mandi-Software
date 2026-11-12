# The one edit kit (plan 44, PRD FR-7.70, FR-7.71, FR-21.4). Every saved entry uses these words:
# the two buttons, the amber band, what was changed, the reason, the edit stamp and the history.
# Business words come from glossary.ftl. Every message here waits for the Urdu reviewer.

# The two buttons of every saved entry, in the same place (PRD FR-7.70).
# review: pending
edit-action-edit = تبدیل کریں

# review: pending
edit-action-cancel = منسوخ کریں

# The edit button of an entry of a closed day (PRD FR-21.4).
# review: pending
edit-action-fix-day = دن کھولیں اور درست کریں

# review: pending
edit-action-history = تبدیلیوں کی تاریخ

# The amber band over an entry that is open for edit. $what: the kind of entry (for example
# "بکری"), $number: its number, $date: its day, DD-MM-YYYY.
# Between two dates in a list of days.
# review: pending
edit-list-separator = { "، " }

# review: pending
edit-band = آپ { $what } { $number } ({ $date }) میں تبدیلی کر رہے ہیں

# review: pending
edit-band-date-fixed = اس کا دن نہیں بدلتا۔ دن بدلنا ہو تو اسے منسوخ کر کے صحیح دن پر دوبارہ لکھیں۔

# The days that were reopened for this fix. $days: the dates, DD-MM-YYYY, one after another.
# review: pending
edit-band-days = دوبارہ کھلے ہوئے دن: { $days }۔ محفوظ کرنے کے بعد یہ دوبارہ بند ہو جائیں گے۔

# review: pending
edit-back = بغیر تبدیلی کے واپس

# review: pending
edit-save = تبدیلیاں محفوظ کریں

# Why an entry cannot be changed (the backend's `locked_reason`). The id is `edit-locked-` and the
# code with dashes.
# review: pending
edit-locked-goods-moved = اس کا مال آگے بک چکا ہے یا ضائع ہو چکا ہے، اس لیے یہ بدلا نہیں جا سکتا۔

# review: pending
edit-locked-lot-seller-locked = اس { -lot } کی بعد کی فروخت اسی بیوپاری کے نام پر ہے، اس لیے اس کا نام اور فون نہیں بدل سکتے۔

# review: pending
edit-locked-garden-closed = { -bagh } بند ہو چکا ہے، اس لیے یہ اندراج بدلا نہیں جا سکتا۔

# review: pending
edit-locked-month-closed = اس کا مہینہ بند ہو چکا ہے۔ پہلے مالک کو مہینے کی بندش ختم کرنی ہو گی۔

# review: pending
edit-locked-cancelled = یہ اندراج منسوخ ہو چکا ہے۔

# review: pending
edit-locked-returns-stand = اس کی واپسیاں درج ہیں، اس لیے یہ بدلا نہیں جا سکتا۔

# review: pending
edit-locked-unknown = یہ اندراج اب بدلا نہیں جا سکتا۔

# What was changed, before the save (PRD FR-7.70). The backend lists each change and its effect.
# review: pending
edit-review-title = آپ نے کیا بدلا

# review: pending
edit-review-none = ابھی کچھ نہیں بدلا۔

# review: pending
edit-review-loading = تبدیلیاں دیکھی جا رہی ہیں…

# review: pending
edit-review-failed = تبدیلیوں کا اثر نہیں دیکھا جا سکا۔

# One change. $where: the place ("لاٹ 1، لائن 2 (اکرم)") or empty, $field: the name of the field,
# $before and $after: the old and the new value.
# review: pending
edit-review-change = { $where }{ $field } پہلے { $before } تھا، اب { $after }

# review: pending
edit-review-where-header = اوپر کی معلومات:

# $name: { -lot } کا نام قوسین میں، مثلاً " (احسان، آم-2)"، یا کچھ نہیں۔
# review: pending
edit-review-where-lot = { -lot } { $lot }{ $name }:

# review: pending
edit-review-where-line = { -lot } { $lot }، لائن { $line }{ $label }:

# review: pending
edit-review-where-part = حصہ{ $label }:

# review: pending
edit-review-where-charge = خرچ{ $label }:

# The figures that follow from the change and are worked out again.
# review: pending
edit-review-worked-title = اس سے دوبارہ نکلنے والی رقمیں

# review: pending
edit-review-effect-title = اس کا اثر

# review: pending
edit-review-effect-none = کسی کھاتے کا بقایا نہیں بدلتا۔

# review: pending
edit-review-effect-account = کھاتہ

# review: pending
edit-review-effect-before = پہلے

# review: pending
edit-review-effect-after = بعد میں

# $before and $after: the cash in hand before and after the change.
# review: pending
edit-review-effect-cash = { -rokar } پہلے { $before } تھی، اب { $after }

# review: pending
edit-review-effect-partner = شراکت دار

# The partner figures are behind the partner code (PRD FR-17.11).
# review: pending
edit-review-partners-locked = شراکت داروں کی رقمیں کوڈ کے بغیر نہیں دکھائی جاتیں۔

# review: pending
edit-review-unlock = کوڈ درج کریں

# The reason of an edit. Asked when the entry is not of today (PRD FR-7.70).
# review: pending
edit-reason-title = یہ اندراج کیوں بدلا جا رہا ہے؟

# $date: the day of the entry, DD-MM-YYYY.
# review: pending
edit-reason-text = { $date } کا اندراج آج کا نہیں ہے۔ وجہ پرانی اور نئی قدروں کے ساتھ آڈٹ لاگ میں لکھی جائے گی۔

# review: pending
edit-reason-label = وجہ

# review: pending
edit-reason-confirm = تبدیلیاں محفوظ کریں

# The reopen and fix in one step (PRD FR-21.4).
# review: pending
edit-fix-title = دن کھولیں اور درست کریں؟

# $date: the day of the entry, DD-MM-YYYY. The days after it open too.
# review: pending
edit-fix-text = { $date } اور اس کے بعد کے سب بند دن دوبارہ کھل جائیں گے۔ آپ اندراج درست کریں گے، اور محفوظ کرنے کے بعد یہ دن اسی ترتیب سے دوبارہ بند ہو جائیں گے۔ وجہ لکھیں۔

# review: pending
edit-fix-label = دن کھولنے کی وجہ

# review: pending
edit-fix-confirm = دن کھولیں

# review: pending
edit-fix-closed-toast = دن دوبارہ بند ہو گئے: { $days }

# review: pending
edit-fix-stayed-title = یہ دن بند نہیں ہو سکے

# $date: the day that stayed open.
# review: pending
edit-fix-stayed-text = { $date } کا دن کھلا رہا۔ جب تک یہ ٹھیک نہ ہو، دن بند نہیں ہو گا۔

# review: pending
edit-fix-stayed-ok = ٹھیک ہے

# review: pending
edit-fix-failed = دن بند نہیں ہو سکے

# The edit stamp (PRD FR-7.71). $date: DD-MM-YYYY, $time: the time with its part of the day,
# $by: who.
# review: pending
edit-stamp = تبدیل شدہ: { $date } { $time }، { $by }

# review: pending
edit-stamp-reason = وجہ: { $reason }

# review: pending
edit-stamp-count = { $count } بار

# The small mark on a list row of an edited entry.
# review: pending
edit-mark = تبدیل شدہ

# review: pending
edit-mark-tip = تبدیل شدہ: { $date } { $time }، { $by }

# The time of day. $hour and $minute are the clock, $part tells the part of the day.
# review: pending
edit-time = { $hour }:{ $minute } { $part ->
    [night] رات
    [morning] صبح
    [noon] دوپہر
   *[evening] شام
}

# The mark on a field that was changed. $before and $after: the old and the new value.
# review: pending
edit-field-mark = پہلے: { $before } ← اب: { $after }

# The history of an entry.
# review: pending
edit-history-title = تبدیلیوں کی تاریخ

# review: pending
edit-history-empty = اس اندراج میں کوئی تبدیلی درج نہیں۔

# review: pending
edit-history-empty-help = جب اندراج بدلا جائے گا تو اس کی ہر تبدیلی یہاں لکھی جائے گی۔

# review: pending
edit-history-failed = تبدیلیوں کی تاریخ نہیں کھل سکی۔

# review: pending
edit-history-version = { $date } { $time }، { $by }

# review: pending
edit-history-field = خانہ

# review: pending
edit-history-old = پہلے

# review: pending
edit-history-new = اب

# review: pending
edit-history-truncated = اور بھی تبدیلیاں ہیں جو یہاں نہیں دکھائی گئیں۔

# review: pending
edit-history-close = بند کریں

# What an old action was.
# review: pending
edit-history-action-create = اندراج لکھا گیا

# review: pending
edit-history-action-edit = اندراج بدلا گیا

# review: pending
edit-history-action-cancel = اندراج منسوخ ہوا

# review: pending
edit-history-action-other = دوسری کارروائی

# An empty value, and a value that is yes or no.
# review: pending
edit-empty = (خالی)

# The print header of an edited entry, and of a report that holds one (PRD FR-7.71). $date.
# review: pending
edit-print-revised = نظر ثانی شدہ: { $date }

# The names of the fields of an edit (the backend's field keys). The id is `edit-field-` and the key
# with dashes.
# review: pending
edit-field-entry-date = تاریخ

# review: pending
edit-field-bill-date = تاریخ بل

# review: pending
edit-field-truck-no = ٹرک نمبر

# review: pending
edit-field-seller = { -beopari }

# review: pending
edit-field-buyer = { -gahak }

# review: pending
edit-field-account = کھاتہ

# review: pending
edit-field-person = نام

# review: pending
edit-field-partner = شراکت دار

# review: pending
edit-field-item = { -qism-maal }

# review: pending
edit-field-packing = { -packing }

# review: pending
edit-field-marka = { -marka }

# review: pending
edit-field-adda = { -adda }

# review: pending
edit-field-store = سٹور

# review: pending
edit-field-qty = تعداد

# review: pending
edit-field-weight = وزن

# review: pending
edit-field-rate = ریٹ

# review: pending
edit-field-seller-rate = ریٹ { -beopari }

# review: pending
edit-field-buyer-rate = ریٹ { -gahak }

# review: pending
edit-field-laga = { -laga }

# review: pending
edit-field-freight = { -kiraya }

# review: pending
edit-field-freight-paid-by = { -kiraya } کس نے دیا

# review: pending
edit-field-labour = { -mazdoori }

# review: pending
edit-field-munshiana = { -munshiana }

# review: pending
edit-field-phone = { -phone }

# review: pending
edit-field-cash = { -naqd }

# review: pending
edit-field-store-rent = { -kiraya-store }

# review: pending
edit-field-adjust-advance = پیشگی سے کاٹیں

# review: pending
edit-field-paid = { -mauqa-par-jama }

# review: pending
edit-field-amount = رقم

# review: pending
edit-field-value = قیمت

# review: pending
edit-field-collector = اگراہی والا

# review: pending
edit-field-head = خرچہ مد

# review: pending
edit-field-paid-to = ادا کیا

# review: pending
edit-field-return-by = واپسی کی تاریخ

# review: pending
edit-field-taken-by = لینے والا

# review: pending
edit-field-mode = ذریعہ

# review: pending
edit-field-narration = تفصیل

# review: pending
edit-field-reason = وجہ

# review: pending
edit-field-note = نوٹ

# review: pending
edit-field-name = نام

# review: pending
edit-field-name-ur = اردو نام

# review: pending
edit-field-name-en = انگریزی نام

# review: pending
edit-field-supplier = فروخت کنندہ

# review: pending
edit-field-shop-name = دکان

# review: pending
edit-field-market = منڈی

# review: pending
edit-field-sent-person = بھیجا گیا

# review: pending
edit-field-cost = خرچ

# review: pending
edit-field-source = ذریعہ

# review: pending
edit-field-share = حصہ

# review: pending
edit-field-money-source = رقم کہاں سے

# review: pending
edit-field-sell-date = بیچنے کا دن

# review: pending
edit-field-lot = { -lot }

# review: pending
edit-field-credit-limit = ادھار حد

# review: pending
edit-field-city = شہر

# review: pending
edit-field-address = پتہ

# review: pending
edit-field-active = فعال

# review: pending
edit-field-value-text = شرح

# review: pending
edit-field-to-date = تاریخ تک

# review: pending
edit-field-other = دوسرا خانہ

# review: pending
edit-field-seller-comm = { -commission } { -beopari }

# review: pending
edit-field-buyer-comm = { -commission } { -gahak }

# review: pending
edit-field-gross = { -kham-bikri }

# review: pending
edit-field-net = { -safi-bikri }

# review: pending
edit-field-seller-amount = رقم { -beopari }

# review: pending
edit-field-buyer-value = قیمت { -gahak }

# review: pending
edit-field-buyer-total = رقم { -gahak }

# The edit of an entry has been saved (plan 44). $number: the number of the entry.
# review: pending
edit-saved = { $number } میں تبدیلی ہو گئی

# The fields the real edit answers of a purchase, a garden and a trip name (plan 44).
# review: pending
edit-field-paid-now = ابھی ادا شدہ

# review: pending
edit-field-rent = پورا { -thaika }

# review: pending
edit-field-total-cost = کل لاگت

# The small mark on a field that an edit changed (PRD FR-7.71): what it held before. $before.
# review: pending
edit-mark-before = پہلے { $before }

# The whole mark with the time and the name of the edit: "Before: 600 → now: 610, 31-10-2026 4:35 pm,
# Owner". $mark: the old against the new value, $when: the day, the time and who.
# review: pending
edit-field-mark-full = { $mark }، { $when }

# What the mark says when the old value is a record this screen cannot name.
# review: pending
edit-mark-other = (کوئی اور)
