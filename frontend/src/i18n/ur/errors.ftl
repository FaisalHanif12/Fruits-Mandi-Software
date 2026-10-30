### One message per backend error code: ERR_SOME_CODE becomes err-some-code.
### Each message says what went wrong and what to do (NFR-USE-13). No technical words (NFR-USE-14).

# review: pending
err-not-implemented = یہ حصہ ابھی تیار نہیں۔ یہ اگلے ورژن میں آئے گا۔
# review: pending
err-internal = کچھ غلط ہو گیا۔ دوبارہ کوشش کریں۔
# Shown under an error when the backend sends a reference number (NFR-OBS-3).
# review: pending
err-reference = مدد کے لیے حوالہ نمبر <code>{ $ref }</code> بتائیں۔

## Input checks (plan 02, contract 4.6; Code Standards 4.2). Numbers in { $max } arrive as
## digits and are grouped before they reach the message.

# review: pending
err-required = یہ خانہ خالی ہے۔ اسے بھریں۔
# review: pending
err-not-a-number = یہاں نمبر لکھیں، مثلاً 12,000۔ حروف اور غلط جگہ پر لگا ہوا کاما نہیں لکھ سکتے۔
# PRD E31.
# review: pending
err-decimals-not-allowed = رقم پورے روپوں میں لکھیں۔
# A quantity typed with a decimal point (tech-lead decision, plan 02). Money uses err-decimals-not-allowed.
# review: pending
err-qty-not-whole = تعداد پورے { -nag } میں لکھیں۔
# review: pending
err-negative-not-allowed = منفی نمبر نہیں لکھ سکتے۔ نمبر منفی نشان کے بغیر لکھیں۔
# $max: the largest amount in rupees.
# review: pending
err-amount-too-large = رقم بہت زیادہ ہے۔ زیادہ سے زیادہ { $max } روپے لکھ سکتے ہیں۔
# A journal entry: one line, or the total of the entry, is over the largest amount allowed.
# $max: the largest amount in rupees.
# review: pending
err-entry-too-large = یہ اندراج بہت بڑا ہے۔ ایک لائن، یا پورے اندراج کا { -meezan }، زیادہ سے زیادہ { $max } روپے ہو سکتا ہے۔
# PRD E24.
# review: pending
err-must-be-more-than-zero = صفر سے بڑا نمبر لکھیں۔
# $max: how many digits may follow the decimal point (2 for a percent, 3 for a weight).
# review: pending
err-too-many-decimals = اعشاریہ کے بعد زیادہ سے زیادہ { $max } ہندسے لکھیں۔
# review: pending
err-percent-over-100 = فیصد 100 سے زیادہ نہیں ہو سکتا۔
# review: pending
err-qty-too-large = تعداد بہت زیادہ ہے۔ زیادہ سے زیادہ { $max } { -nag } لکھ سکتے ہیں۔
# $max: kilograms.
# review: pending
err-weight-too-large = وزن بہت زیادہ ہے۔ زیادہ سے زیادہ { $max } کلو لکھ سکتے ہیں۔
# review: pending
err-date-invalid = یہ تاریخ درست نہیں۔ دن، مہینہ اور سال درست لکھیں۔
# PRD FR-6.6. $closed_up_to: DD-MM-YYYY.
# review: pending
err-date-closed = یہ دن بند ہو چکا ہے۔ { $closed_up_to } کے بعد کی تاریخ لکھیں، یا مالک سے دن کھلوائیں۔
# PRD FR-6.6. $today: DD-MM-YYYY.
# review: pending
err-date-in-future = آگے کی تاریخ نہیں لکھ سکتے۔ آج ({ $today }) یا اس سے پہلے کی تاریخ لکھیں۔
# PRD FR-7.39.
# review: pending
err-code-invalid = کوڈ درست نہیں۔ پورا کوڈ لکھیں، یا 1 سے 4 ہندسوں کا مختصر کوڈ۔
# PRD FR-7.28.
# review: pending
err-name-ur-script = اردو نام اردو حروف میں لکھیں۔
# PRD FR-7.28.
# review: pending
err-name-en-script = انگریزی نام انگریزی حروف میں لکھیں۔
# PRD 10.2, Code Standards 4.2 (Tag). The code name is the backend's (plan 17e T1).
# review: pending
err-tag-invalid = نشان 1 سے 4 انگریزی بڑے حروف یا نقطوں کا ہونا چاہیے، جن میں کم از کم ایک حرف ہو۔ مثلاً K، BA یا D.P۔

# review: pending
err-name-too-long = نام بہت لمبا ہے۔ زیادہ سے زیادہ { $max } حروف لکھیں۔
# review: pending
err-phone-invalid = فون نمبر درست نہیں۔ 03 سے شروع ہونے والا 11 ہندسوں کا نمبر لکھیں۔
# review: pending
err-text-too-long = تحریر بہت لمبی ہے۔ زیادہ سے زیادہ { $max } حروف لکھیں۔
# review: pending
err-control-chars = اس خانے میں چھپا ہوا حرف ہے، مثلاً ٹیب، نئی سطر یا تحریر کی سمت کا نشان۔ اسے مٹائیں اور تحریر دوبارہ لکھیں۔
# Code Standards M7.
# review: pending
err-overflow = رقم بہت بڑی ہو گئی ہے۔ رقمیں دوبارہ دیکھیں اور پھر کوشش کریں۔
# PRD FR-17.1, E9.
# review: pending
# $total: the total of the typed shares as a percent, for example 90.
err-shares-not-100 = تمام { -shirakat-dar } کا کل { -hissa } { $total } فیصد ہے، 100 فیصد نہیں۔ { -shirakat-dar } کی ترتیب میں اسے درست کریں۔

# PRD FR-17.2, E18, A31. $today: today, DD-MM-YYYY.
# review: pending
err-share-backdated = نیا { -hissa } پچھلی تاریخ سے شروع نہیں ہو سکتا۔ آج ({ $today }) یا اس کے بعد کی تاریخ لکھیں۔

# PRD FR-17.1. A partner with a share cannot be made inactive.
# review: pending
err-partner-has-share = اس { -shirakat-dar } کا { -hissa } ابھی 0 نہیں ہے۔ پہلے اس کا { -hissa } 0 کر کے محفوظ کریں، پھر اسے غیر فعال کریں۔

# PRD FR-17.1. The share list must name every active partner once.
# review: pending
err-partner-not-listed = { -hissa } کی فہرست میں ہر فعال { -shirakat-dar } ایک بار ہونا چاہیے۔ صفحہ دوبارہ کھولیں اور پھر کوشش کریں۔

# NFR-SCL-5. $max: the most partners.
# review: pending
err-too-many-partners = { $max } سے زیادہ { -shirakat-dar } شامل نہیں ہو سکتے۔

## The data file (plan 03, contract 4.6; NFR-REL-6, NFR-REL-7, NFR-REL-10, NFR-REL-13).
## The reference number follows each of these messages (err-reference), so they do not repeat it.

# NFR-REL-10. Nothing was written.
# review: pending
err-disk-full = ڈسک بھر گئی ہے۔ کچھ جگہ خالی کریں اور دوبارہ کوشش کریں۔ کچھ بھی محفوظ نہیں ہوا۔
# NFR-REL-7. Saving stays stopped until the check passes.
# review: pending
err-data-check-failed = ڈیٹا فائل جانچ میں ٹھیک نہیں نکلی، اس لیے آپ کے ڈیٹا کو بچانے کے لیے محفوظ کرنا روک دیا گیا ہے۔ ایپ بند کر کے دوبارہ کھولیں۔ یہ پیغام پھر آئے تو مدد کے لیے رابطہ کریں۔
# NFR-REL-13. The copy made before the update was put back.
# review: pending
err-migration-failed = نئے ورژن کے لیے ڈیٹا کی تبدیلی مکمل نہیں ہو سکی۔ آپ کا ڈیٹا محفوظ ہے: تبدیلی سے پہلے بنائی گئی کاپی واپس رکھ دی گئی ہے۔ پچھلا ورژن دوبارہ لگائیں، یا مدد کے لیے رابطہ کریں۔
# The data file was in use by another task. Nothing was written.
# review: pending
err-db-busy = ڈیٹا فائل ابھی مصروف ہے۔ ذرا دیر بعد دوبارہ کوشش کریں۔

## Sign-in and settings (plan 04, contract 4. NFR-SEC-1, NFR-SEC-2, NFR-SEC-4, PRD FR-8.2).
## The reference number follows these messages when the backend sends one (err-reference).

# $min: the shortest password, in characters.
# review: pending
err-password-too-short = پاس ورڈ کم از کم { $min } حروف کا ہونا چاہیے۔
# review: pending
err-passwords-differ = دونوں پاس ورڈ ایک جیسے نہیں۔ دوبارہ لکھیں۔
# review: pending
err-username-invalid = صارف کا نام 3 سے 32 حروف کا ہو۔ اس میں انگریزی چھوٹے حروف، ہندسے، نقطہ، نیچے کی لکیر یا درمیانی لکیر ہی لکھ سکتے ہیں۔
# The same message for a wrong password and an unknown name, so nobody learns which names exist.
# $tries_left: tries left before the sign-in waits.
# review: pending
err-wrong-password = صارف کا نام یا پاس ورڈ غلط ہے۔ باقی کوششیں: { $tries_left }
# PRD FR-8.2. $seconds: seconds left to wait. The sign-in screen counts down by itself.
# review: pending
err-locked-wait = بہت زیادہ غلط کوششیں ہو گئیں۔ { $seconds } سیکنڈ رک کر دوبارہ کوشش کریں۔
# review: pending
err-user-inactive = اس صارف سے لاگ ان نہیں ہو سکتا۔ ریکوری کوڈ استعمال کریں۔
# review: pending
err-already-signed-in = آپ پہلے ہی لاگ ان ہیں۔
# review: pending
err-not-locked = ایپ لاک نہیں ہے۔
# review: pending
err-recovery-code-wrong = صارف کا نام یا ریکوری کوڈ غلط ہے۔ دیکھ کر دوبارہ لکھیں۔
# review: pending
err-already-set-up = پہلی ترتیب پہلے ہی مکمل ہو چکی ہے۔ لاگ ان کریں۔
# $min and $max: the allowed range.
# review: pending
err-out-of-range = یہ قیمت { $min } سے { $max } کے درمیان ہونی چاہیے۔
# review: pending
err-not-logged-in = پہلے لاگ ان کریں۔
# PRD FR-8.4.
# review: pending
err-session-locked = ایپ لاک ہے۔ پاس ورڈ لکھ کر کھولیں۔
# review: pending
err-not-found = یہ نہیں ملا۔ صفحہ دوبارہ کھولیں اور دوبارہ چنیں۔
# PRD FR-7.66, E49.
# review: pending
err-spelling-unconfirmed = ان لفظوں کے ہجے کی تصدیق نہیں ہوئی: { $words }۔ ہجے دیکھیں، پھر تصدیق کریں یا درست کریں۔

## The master lists (plan 06).

# review: pending
err-code-taken = یہ کوڈ کسی اور کے پاس ہے۔ کوئی دوسرا کوڈ لکھیں۔
# review: pending
err-version-changed = آپ کے کھولنے کے بعد یہ بدل چکا ہے۔ اسے دوبارہ کھولیں۔
# review: pending
err-format-invalid = یہاں لکھی ہوئی بات درست شکل میں نہیں۔ دیکھ کر دوبارہ لکھیں۔
# review: pending
err-series-full = اس سلسلے کے سب کوڈ استعمال ہو چکے ہیں۔ کوئی اور سلسلہ استعمال کریں۔

## Accounts, the chart and opening balances (plan 07).

# PRD FR-6.2. Nothing was saved.
# review: pending
err-journal-unbalanced = اندراج کا { -banaam } اور { -jama } برابر نہیں، اس لیے کچھ محفوظ نہیں ہوا۔ رقمیں دیکھ کر دوبارہ کوشش کریں۔
# PRD FR-10.3.
# review: pending
err-code-reserved = یہ کوڈ ایپ کے اپنے کھاتوں کے لیے رکھا گیا ہے۔ کوئی دوسرا کوڈ لکھیں۔
# PRD FR-10.2.
# review: pending
err-opening-both-sides = ابتدائی { -banaam } یا ابتدائی { -jama } لکھیں، دونوں نہیں۔
# PRD FR-10.2, FR-15.8.
# review: pending
err-advance-not-seller = ابتدائی { -peshgi } صرف { -beopari } کے ابتدائی { -banaam } پر لگ سکتی ہے۔

# plan 17b: an old client sent a mode, a bank or a cheque; the screens never do (PRD 15.1).
# review: pending
err-field-not-allowed = یہ خانہ اب استعمال نہیں ہوتا۔ اسے چھوڑ دیں۔

# PRD FR-10.9, FR-6.5, FR-13.5, C20. Income and shop expense accounts, Opening Difference and Pukhta Khata cannot take one. A partner's drawings account and a person's money lent account take an opening debit only.
# review: pending
err-opening-not-allowed = اس کھاتے میں یہ ابتدائی بقایا نہیں لکھا جا سکتا۔ آمدن اور { -dukan-kharcha } کے کھاتوں، { -naqdi-mandi }، { -pukhta-khata } اور ابتدائی فرق میں ابتدائی بقایا نہیں لگتا۔ { -shirakat-dar } کے { -zati-kharcha } کھاتے اور { -adhar-raqam } کے کھاتے میں صرف ابتدائی { -banaam } لگ سکتا ہے۔

## Advances and closing days (plan 10).

# PRD FR-15.4. An advance goes to a seller, never to Pukhta Khata.
# review: pending
err-not-a-seller = یہ { -beopari } کا کھاتہ نہیں۔ { -peshgi } صرف { -beopari } کو دی جا سکتی ہے، { -pukhta-khata } کو نہیں۔
# PRD FR-10.2. Hidden from pickers.
# review: pending
err-account-inactive = یہ کھاتہ غیر فعال ہے اور نئے اندراج میں استعمال نہیں ہو سکتا۔ کوئی دوسرا کھاتہ چنیں۔
# PRD FR-10.4. A system account is used by the books, so it stays active.
# review: pending
err-system-account = یہ نظام کا کھاتہ ہے۔ حساب کتاب میں اس کا استعمال ہوتا ہے، اس لیے اسے غیر فعال نہیں کیا جا سکتا۔
# PRD FR-21.2.
# review: pending
err-reason-required = وجہ لکھنا ضروری ہے۔
# NFR-REL-8. Nothing was closed.
# review: pending
err-trial-not-balanced = { -trial-balance } میں فرق ہے، اس لیے دن بند نہیں ہو سکتا۔ پہلے { -trial-balance } دیکھ کر فرق ٹھیک کریں۔
# PRD FR-21.2.
# review: pending
err-day-not-closed = یہ دن بند ہی نہیں، اس لیے کھولنے کے لیے کچھ نہیں۔ بند دن کی تاریخ لکھیں۔
# PRD FR-17.6.
# review: pending
err-month-closed = اس تاریخ کا مہینہ بند ہو چکا ہے۔ دن کھولنے سے پہلے مہینے کی بندش واپس کریں۔

# PRD FR-21.1. Every day up to today is closed already.
# review: pending
err-nothing-to-close = آج تک کے سب دن پہلے ہی بند ہیں، اس لیے بند کرنے کے لیے کوئی دن نہیں۔

## Commission rules (plan 09).

# PRD 11.2, 11.3. The field says which part.
# review: pending
err-rule-not-allowed = اصول کی یہ ترتیب درست نہیں۔ چارج اور طریقہ فہرست میں سے چنیں، اور گروپ صرف اسی وقت چنیں جب کوئی کھاتہ یا { -qism-maal } نہ چنا ہو۔ عام شرحیں ترتیبات کے چارجز میں بدلی جاتی ہیں۔
# review: pending
err-date-range = آخری تاریخ پہلی تاریخ سے پہلے ہے۔ آخری تاریخ پہلی تاریخ کے دن یا اس کے بعد کی لکھیں۔

## The sale screen (plan 12). Warnings use the same kind of message.

# PRD FR-12.6. A line sells more than the lot has left.
# review: pending
err-qty-exceeds-lot = تعداد { -lot } کی باقی تعداد سے زیادہ ہے۔ صرف { $available } { -nag } باقی ہیں۔
# PRD FR-11.10. The setting is off.
# review: pending
err-override-not-allowed = لائن پر { -commission } یا { -laga } بدلنا بند ہے۔ مالک اسے ترتیبات میں چالو کر سکتا ہے۔
# PRD FR-12.4. $limit and $would_owe are amounts in rupees.
# review: pending
err-credit-limit-exceeded = اس { -bikri } سے { -gahak } کا بقایا { $would_owe } ہو جائے گا، جو اس کی ادھار کی حد { $limit } سے زیادہ ہے۔
# review: pending
err-seller-required = اس { -lot } کا { -beopari } چنیں۔
# review: pending
err-buyer-required = اس لائن کا { -gahak } چنیں۔
# PRD FR-13.2.
# review: pending
err-pukhta-name-required = { -pukhta-khata } میں نام لکھنا ضروری ہے۔
# PRD FR-13.2.
# review: pending
err-pukhta-phone-required = اس { -pukhta-khata } والے کا فون نمبر لکھنا ضروری ہے۔
# PRD FR-12.8.
# review: pending
err-no-lots = کم از کم ایک { -lot } لکھیں۔
# PRD FR-12.8.
# review: pending
err-no-sale-lines = کم از کم { -bikri } کی ایک لائن لکھیں۔
# PRD E23.
# review: pending
err-weight-required = وزن کلو میں لکھیں۔ یہ مال وزن کے حساب سے بکتا ہے۔
# PRD FR-6.7. $cash_after is cash in hand after the entry, in rupees.
# review: pending
err-cash-below-zero-not-confirmed = اس اندراج کے بعد موجود { -rokar } { $cash_after } ہو جائے گی۔ محفوظ کرنے کے لیے اندراج کی تصدیق کریں۔
# review: pending
err-not-a-buyer = یہ { -gahak } کا کھاتہ نہیں۔ کوئی { -gahak } چنیں۔
# PRD 12.4, 10.2. The buyer's credit is off, or he is the nameless cash buyer.
# review: pending
err-credit-not-allowed = اس { -gahak } کے لیے ادھار کی اجازت نہیں۔ لائن کو { -naqdi } کر دیں۔
# A warning (PRD FR-12.8): the rest goes to باقی مال.
# review: pending
err-qty-left-unsold = { $left } { -nag } باقی ہیں۔ یہ { -baqi-maal } میں رہیں گے۔
# A warning (PRD FR-12.7): the save is allowed. $net is in rupees.
# review: pending
err-lot-net-negative = اس { -lot } کی { -safi-bikri } صفر سے کم ({ $net }) ہے۔ یہ رقم { -beopari } کے ذمے ہے۔
# A warning (PRD FR-7.55). $average is in rupees, $percent is the setting.
# review: pending
err-rate-far-from-average = یہ ریٹ پچھلے 7 دن کی اوسط ({ $average }) سے { $percent } فیصد سے زیادہ مختلف ہے۔ اسے دیکھ لیں۔

## Drafts (plan 13, PRD FR-12.10).

# $max_bytes: the most the app keeps for one unfinished entry. The word is not shown.
# review: pending
err-draft-invalid = لکھا ہوا کام عارضی طور پر محفوظ نہیں ہو سکا۔ اندراج ابھی محفوظ کر لیں۔ بار بار ہو تو مدد کے لیے بتائیں۔

## Printing (plan 14).

# PRD FR-12.12. Printing needs Windows; on other computers the preview shows how it will print.
# review: pending
err-print-windows-only = پرنٹنگ صرف ونڈوز پر ہو سکتی ہے۔ اس کمپیوٹر پر پیش منظر میں دیکھ لیں کہ دستاویز کیسے چھپے گی۔

# $printer is the printer name in Settings.
# review: pending
err-printer-not-found = پرنٹر { $printer } نہیں ملا۔ دیکھیں کہ وہ آن ہے اور جڑا ہوا ہے، یا ترتیبات میں دوسرا پرنٹر چنیں۔

# review: pending
err-print-failed = دستاویز پرنٹ نہیں ہو سکی۔ پرنٹر دیکھ لیں اور دوبارہ کوشش کریں۔ بار بار ہو تو مدد کے لیے بتائیں۔

## Vouchers (plan 15, PRD 15.1 to 15.4).

# PRD FR-15.11. Money paid to a buyer is cash lent to him.
# review: pending
err-payment-to-buyer-needs-reason = { -gahak } کو رقم دینا { -naqd-udhar } ہے۔ اس کی وجہ لکھیں۔

# PRD FR-15.12, FR-16.2. $open is the most that can be returned, in rupees.
# review: pending
err-return-exceeds-open = یہ رقم واپس ہونے والی رقم سے زیادہ ہے۔ زیادہ سے زیادہ { $open } واپس ہو سکتے ہیں۔

# PRD FR-15.3.
# review: pending
err-voucher-cancelled = یہ واؤچر منسوخ ہو چکا ہے، اس میں تبدیلی نہیں ہو سکتی۔

# PRD FR-13.5, FR-13.7.
# review: pending
err-person-required = { -pukhta-khata } کے لیے اس شخص کا نام یا فون لکھ کر اسے فہرست سے چنیں۔

# A warning (PRD FR-15.7): the save is allowed. $payable is what is payable, $over how much the
# amount is more than it, both in rupees.
# review: pending
err-payment-above-payable = { -beopari } کو جو رقم دینی ہے ({ $payable }) یہ اس سے { $over } زیادہ ہے۔ آپ پھر بھی محفوظ کر سکتے ہیں۔ زیادہ رقم { -beopari } کے ذمے دکھائی جائے گی، یہ { -peshgi } نہیں ہوگی۔

## بعد کی فروخت، ترمیم، منسوخی اور ضائع مال (پلان 16)۔

# review: pending
err-bikri-cancelled = یہ { -bikri } منسوخ ہو چکی ہے اور بدلی نہیں جا سکتی۔

# review: pending
err-lot-not-found = یہ { -lot } نہیں ملا۔ { -baqi-maal } کی فہرست کھول کر دوبارہ چنیں۔

# review: pending
err-nothing-to-sell = اس { -lot } میں بیچنے کے لیے کچھ باقی نہیں۔

# $available: { -lot } میں کتنے { -nag } باقی ہیں۔
# review: pending
err-write-off-exceeds-balance = جتنا مال باقی ہے اس سے زیادہ ضائع نہیں کیا جا سکتا۔ صرف { $available } { -nag } باقی ہیں۔

# review: pending
err-date-before-received = دن مال وصول ہونے کے دن سے پہلے کا نہیں ہو سکتا۔

# review: pending
err-qty-below-sold = وصول شدہ تعداد اس { -lot } سے پہلے بکی ہوئی تعداد سے کم نہیں ہو سکتی۔

# PRD FR-12.18 to FR-12.20. The edit rules of a saved { -bikri }. $rule is a fixed name from the
# backend, one case each: the day, a lot, its order, its seller or item, a line, a later sale's lot.
# Every case says what to do. The field points at the lot or the line.
# review: pending
err-edit-not-allowed = { $rule ->
    [day_changed] محفوظ شدہ اندراج کا دن ترمیم میں بدلا نہیں جا سکتا۔ پرانا دن واپس لکھیں۔
    [lot_removed] محفوظ شدہ { -lot } ہٹایا نہیں جا سکتا، کیونکہ اس کی فروخت اور مال اس پر قائم ہیں۔ { -lot } واپس رکھیں۔ پوری { -bikri } واپس لینی ہو تو اسے منسوخ کریں۔
    [lot_not_in_posting] یہ { -lot } اس اندراج کا حصہ نہیں جس کی ترمیم ہو رہی ہے۔ ترمیم بند کر کے اندراج دوبارہ کھولیں۔
    [lot_order_changed] محفوظ شدہ { -lot } اپنی ترتیب میں رہتے ہیں اور نئے { -lot } ان کے بعد آتے ہیں۔ نئے { -lot } کو آخر میں رکھیں۔
    [lot_seller_locked] اس { -lot } پر بعد کا اندراج یا فروخت قائم ہو چکی ہے، اس لیے { -beopari } نہیں بدلا جا سکتا۔ پرانا { -beopari } واپس چنیں۔
    [lot_item_locked] اس { -lot } پر بعد کا اندراج یا فروخت قائم ہو چکی ہے، اس لیے { -qism-maal } نہیں بدلی جا سکتی۔ پرانی { -qism-maal } واپس چنیں۔
    [line_not_in_posting] یہ لائن اس اندراج کا حصہ نہیں جس کی ترمیم ہو رہی ہے۔ ترمیم بند کر کے اندراج دوبارہ کھولیں۔
    [posting_lots_locked] بعد کی فروخت میں صرف اس کی اپنی لائنیں بدلی جا سکتی ہیں، { -lot } نہیں۔ { -lot } کی تبدیلی پہلے اندراج میں کریں۔
   *[other] اس اندراج میں یہ تبدیلی نہیں ہو سکتی۔ ترمیم بند کر کے اندراج دوبارہ کھولیں۔
}

## Trip goods and trips (plan 16b, PRD 12.10, FR-12.24, FR-12.26, FR-12.29, FR-12.31).

# PRD FR-12.26. $qty_left: how many { -nag } of the quantity taken are not sold on the lines.
# review: pending
err-own-lot-not-fully-sold = { -apna-maal } کا یہ { -lot } اسی { -bikri } میں پورا بکنا چاہیے۔ لائنوں میں ابھی { $qty_left } { -nag } نہیں بکے۔ انہیں لائنوں میں بیچیں، یا خریداری سے کم { -nag } لیں۔ جو نہ لیں وہ خریداری میں رہتا ہے۔

# PRD FR-12.26. The account offered only as the seller of a lot.
# review: pending
err-own-goods-account-restricted = { -apna-maal } صرف { -bikri } کے { -lot } کا { -beopari } ہو سکتا ہے۔ کوئی اور کھاتہ چنیں۔

# PRD FR-12.24. $qty_sold and $qty_written_off: how many { -nag }.
# review: pending
err-purchase-has-sales = یہ خریداری منسوخ نہیں ہو سکتی، کیونکہ { $qty_sold } { -nag } بک چکے ہیں اور { $qty_written_off } { -nag } ضائع ہو چکے ہیں۔ پہلے { -bikri } منسوخ کریں یا ضائع واپس کریں۔

# review: pending
err-purchase-cancelled = یہ خریداری منسوخ ہو چکی ہے۔ کوئی اور خریداری چنیں۔

# A warning, not a refusal (PRD FR-12.26).
# review: pending
err-own-lot-freight-in-cost = خریداری کا { -kiraya } اس کی لاگت میں پہلے سے شامل ہے۔ یہاں لکھا ہوا { -kiraya } دوبارہ گنا جائے گا۔ جب تک یہی مقصد نہ ہو، اسے خالی رہنے دیں۔

# A warning (plan 37, PRD FR-11.9): the save is allowed and the typed amount is kept. $lot_no is the
# earlier lot that took the charge, $bikri_number its sale (0 means a lot of this same sale; the
# message chooses by $bikri_number_zero) and $entry the goods-arrived entry.
# review: pending
err-entry-charge-already-taken = { $bikri_number_zero ->
    [yes] یہ چارج { -maal-aaya } اندراج { $entry } کی { -lot } { $lot_no } پر پہلے ہی لگ چکا ہے۔ آپ کی لکھی ہوئی رقم برقرار رہے گی۔
   *[no] یہ چارج { -bikri } { $bikri_number } ({ -maal-aaya } اندراج { $entry }) کی { -lot } { $lot_no } پر پہلے ہی لگ چکا ہے۔ آپ کی لکھی ہوئی رقم برقرار رہے گی۔
}

# PRD FR-12.31, FR-12.33: a closed trip takes no new purchase or cost.
# review: pending
err-trip-closed = یہ { -market-safar } بند ہو چکا ہے۔ اس میں نئی خریداری یا نیا خرچ نہیں ہو سکتا۔

# PRD FR-12.31: a trip cost is spread over the purchases that still have goods left.
# review: pending
err-trip-no-stock = اس { -market-safar } کی کسی خریداری میں مال باقی نہیں، اس لیے خرچ بانٹنے کو کچھ نہیں۔ پہلے خریداری درج کریں۔

# PRD FR-12.29: the shares are fixed after the first entry.
# review: pending
err-venture-shares-locked = اس { -market-safar } میں اندراج ہو چکا ہے، اس لیے شراکت داروں کا { -hissa } اب نہیں بدل سکتا۔

# PRD FR-12.24. $trip_date: the day of the trip.
# review: pending
err-purchase-before-trip = تاریخ { -market-safar } کی تاریخ ({ $trip_date }) سے پہلے کی ہے۔ سفر کی تاریخ یا اس کے بعد کی تاریخ چنیں۔

# PRD FR-12.31. $qty_sold and $qty_written_off: how many { -nag } of the purchases it was spread over.
# review: pending
err-trip-cost-has-sales = یہ خرچ منسوخ نہیں ہو سکتا، کیونکہ جن خریداریوں پر یہ بانٹا گیا ان کے { $qty_sold } { -nag } بک چکے ہیں اور { $qty_written_off } { -nag } ضائع ہو چکے ہیں۔ پہلے { -bikri } منسوخ کریں یا ضائع واپس کریں۔

# PRD A63: a partner cannot be made inactive while he is in an open trip.
# review: pending
err-partner-in-open-venture = یہ { -shirakat-dar } ابھی کھلے { -market-safar } میں ہے، اس لیے اسے غیر فعال نہیں کیا جا سکتا۔ پہلے { -market-safar } بند ہونے دیں۔

# PRD FR-12.31: cancel the cost first.
# review: pending
err-purchase-carries-trip-cost = اس خریداری میں { -market-safar } کے خرچ کا حصہ شامل ہے، اس لیے یہ منسوخ نہیں ہو سکتی۔ پہلے وہ خرچ منسوخ کریں۔

# review: pending
err-trip-entry-cancelled = یہ اندراج پہلے ہی منسوخ ہو چکا ہے۔

## بیک اپ اور بحالی (plan 18, PRD Section 20)۔ `BackupCode` کے وہ کوڈ جو core میں ابھی نہیں ہیں۔
## جو سات core میں پہلے سے ہیں (ڈسک بھر گئی، اندرونی، ڈیٹا کی جانچ، مصروف، ضروری، حد سے باہر،
## ڈیٹا اپ ڈیٹ) ان کے الفاظ اوپر والے ہی ہیں۔

# review: pending
err-backup-not-found = یہ کاپی نہیں ملی۔ ہو سکتا ہے ہٹا دی گئی ہو۔ فہرست دوبارہ کھولیں اور پھر کوشش کریں۔

# review: pending
err-backup-key-not-set = اس کمپیوٹر پر ابھی بیک اپ کی چابی نہیں ہے۔ پہلے چابی بنائیں۔

# review: pending
err-backup-key-already-set = بیک اپ کی چابی پہلے سے استعمال میں ہے، اس لیے نئی نہیں بن سکتی۔

# review: pending
err-backup-key-format = چابی صحیح طرح نہیں لکھی گئی۔ اس میں چار چار حرفوں اور ہندسوں کے چھ حصے ہوتے ہیں، جن کے درمیان ڈیش ہوتی ہے۔

# review: pending
err-backup-key-wrong = یہ درست چابی نہیں ہے۔ پرنٹ کی ہوئی پرچی دیکھیں اور دوبارہ کوشش کریں۔ کچھ نہیں بدلا۔

# review: pending
err-backup-key-required = یہ کاپی بیک اپ کی چابی سے بند ہے۔ پرنٹ کی ہوئی پرچی سے چابی لکھیں۔

# review: pending
err-backup-corrupt = یہ کاپی خراب ہے اور استعمال نہیں ہو سکتی۔ کچھ نہیں بدلا۔ کوئی اور کاپی استعمال کریں۔

# review: pending
err-backup-check-failed = کاپی بن گئی لیکن جانچ میں پوری نہیں اتری، اس لیے اس پر بھروسا نہیں کیا جا سکتا۔ نئی کاپی بنائیں۔

# review: pending
err-backup-cloud-not-configured = کلاؤڈ کاپی ابھی ترتیب نہیں دی گئی۔ بیک اپ کی ترتیبات میں کلاؤڈ کی تفصیل بھریں۔

# $reason: offline، refused یا not_found۔
# review: pending
err-backup-cloud-failed =
    { $reason ->
        [offline] کلاؤڈ تک رسائی نہیں ہو سکی۔ انٹرنیٹ دیکھیں۔ کاپی انتظار میں رہتی ہے اور خود بخود دوبارہ بھیجی جاتی ہے۔
        [refused] کلاؤڈ نے کاپی قبول نہیں کی۔ بیک اپ کی ترتیبات میں کلاؤڈ کی چابیاں دیکھیں۔
        [not_found] کلاؤڈ کا بکٹ یا کاپی نہیں ملی۔ بیک اپ کی ترتیبات میں بکٹ کا نام دیکھیں۔
       *[other] کلاؤڈ کاپی نہیں بن سکی۔ کچھ دیر بعد دوبارہ کوشش کریں۔
    }

# review: pending
err-backup-usb-not-present = یو ایس بی ڈرائیو لگی ہوئی نہیں۔ اسے لگائیں، یا بیک اپ کی ترتیبات میں یو ایس بی کاپی بند کر دیں۔

# review: pending
err-backup-usb-failed = یو ایس بی ڈرائیو پر کاپی نہیں بن سکی۔ ڈرائیو اور فولڈر دیکھیں۔

# review: pending
err-backup-local-failed = اس کمپیوٹر پر کاپی نہیں بن سکی۔ بیک اپ کی ترتیبات میں فولڈر اور ڈسک کی خالی جگہ دیکھیں۔

# review: pending
err-backup-busy = بیک اپ یا بحالی پہلے سے چل رہی ہے۔ اس کے مکمل ہونے تک انتظار کریں۔

# review: pending
err-backup-not-confirmed = بحالی شروع ہونے سے پہلے تصدیق کریں۔ کچھ نہیں بدلا۔

# review: pending
err-backup-newer-than-app = یہ کاپی اس کمپیوٹر کی ایپ سے نئے ورژن کی ہے۔ پہلے ایپ اپ ڈیٹ کریں۔

# review: pending
err-backup-not-started = بیک اپ کی سروس ابھی چل نہیں رہی۔ ذرا انتظار کریں اور دوبارہ کوشش کریں۔

# review: pending
err-backup-restore-failed = بحالی مکمل نہیں ہو سکی۔ ایپ بند کر کے دوبارہ کھولیں۔ اگر کھاتے غلط لگیں تو مدد کے لیے رابطہ کریں: بحالی سے پہلے حفاظتی کاپی بنائی گئی تھی۔

## Plan 17: the cash book area (PRD 16, FR-16.9, FR-17.8, 16.4).

# PRD FR-16.9. The same day is allowed.
# review: pending
err-return-by-before-entry = واپسی کی تاریخ اندراج کی تاریخ سے پہلے نہیں ہو سکتی۔

# review: pending
err-partner-required = { -shirakat-dar } چنیں۔

# review: pending
err-expense-head-required = خرچہ مد چنیں۔

# review: pending
err-lent-person-required = شخص چنیں۔

# review: pending
err-khata-required = { -alag-khata } چنیں۔

# PRD 16.4. Nothing is saved. $dr and $cr are the two totals in rupees.
# review: pending
err-general-not-balanced = { -banaam } کا میزان { $dr } اور { -jama } کا میزان { $cr } ہے۔ اندراج محفوظ کرنے سے پہلے دونوں برابر ہونے چاہییں۔

# review: pending
err-general-too-few-lines = { -general } اندراج میں کم از کم دو لائنیں ہونی چاہییں۔

# PRD FR-17.8. A warning only: the entry can still be saved. $balance is the capital balance and $over is the part above it, in rupees.
# review: pending
err-capital-exceeds-balance = یہ شراکت دار کے { -sarmaya } بقایا { $balance } سے { $over } زیادہ ہے۔ پھر بھی محفوظ کیا جا سکتا ہے۔

## Plan 22: updates and the support file (PRD NFR-INS-3, NFR-INS-4, NFR-SEC-11, NFR-OBS-4).

# review: pending
err-update-check-failed = ایپ نیا ورژن نہیں دیکھ سکی۔ انٹرنیٹ دیکھیں اور دوبارہ کوشش کریں۔ آپ کا کام محفوظ ہے۔

# review: pending
err-update-not-signed = یہ اپڈیٹ ہماری طرف سے دستخط شدہ نہیں، اس لیے انسٹال نہیں ہوئی۔ کچھ نہیں بدلا اور کوئی کاپی نہیں بنی۔ یہ اپڈیٹ استعمال نہ کریں۔

# $drafts is the number of unsaved entries.
# review: pending
err-update-during-work = غیر محفوظ اندراجات ہیں ({ $drafts })۔ انہیں محفوظ کریں یا چھوڑ دیں، پھر اپڈیٹ کریں۔ کچھ انسٹال نہیں ہوا۔

# review: pending
err-update-not-confirmed = اپڈیٹ کی تصدیق نہیں ہوئی، اس لیے کچھ انسٹال نہیں ہوا۔ اپڈیٹ کا بٹن دبائیں اور سوال کا جواب دیں۔

# review: pending
err-update-backup-failed = اپڈیٹ سے پہلے حفاظتی کاپی نہیں بن سکی، اس لیے کچھ انسٹال نہیں ہوا۔ بیک اپ کی اسکرین دیکھیں اور دوبارہ کوشش کریں۔

# review: pending
err-update-changed = اب کوئی اور ورژن دستیاب ہے۔ نیا ورژن دوبارہ دیکھیں اور اپڈیٹ سے پہلے نیا کیا ہے پڑھ لیں۔

# review: pending
err-update-none = کوئی نیا ورژن نہیں ہے۔ ایپ تازہ ترین ہے۔

## Plan 17c: gardens (PRD 12.11, FR-12.34 to FR-12.38).

# PRD FR-16.12: the fruit entry was cancelled already (it cannot be changed or cancelled again).
# review: pending
err-fruit-cancelled = یہ { -phal } کا اندراج پہلے ہی منسوخ ہو چکا ہے، اس لیے اس میں تبدیلی نہیں ہو سکتی۔

# PRD 16.11, FR-12.22: fruit was taken from a lot of this sale, so the sale cannot be cancelled or undone.
# $qty_taken: the نگ taken from its lots.
# review: pending
err-bikri-has-fruit = اس { -bikri } کے { -lot } سے { $qty_taken } { -nag } { -phal } لیا گیا ہے، اس لیے یہ منسوخ نہیں ہو سکتی۔ پہلے { -phal } کا اندراج منسوخ کریں۔

# PRD FR-12.38, E65. Also shown when an entry of a closed garden is cancelled, or a { -bikri } with a lot of it is edited or cancelled.
# review: pending
err-garden-closed = یہ { -bagh } بند ہے، اس لیے اس میں نیا اندراج، نیا لاٹ یا پھل نہیں لیا جا سکتا۔ پہلے اس کی بندش واپس لیں۔

# PRD FR-12.38. The garden is open, so there is no closing to undo.
# review: pending
err-garden-not-closed = یہ { -bagh } کھلا ہے، اس لیے واپس لینے کے لیے کوئی بندش نہیں۔

# PRD FR-12.34. $garden_date: the day of the garden.
# review: pending
err-date-before-garden = تاریخ { -bagh } کی تاریخ ({ $garden_date }) سے پہلے کی ہے۔ باغ کی تاریخ یا اس کے بعد کی تاریخ چنیں۔

# Plan 17c. $last_date: the day of the latest entry or lot of the garden.
# review: pending
err-close-before-last-entry = { -bagh } اپنے آخری اندراج یا لاٹ ({ $last_date }) سے پہلے بند نہیں ہو سکتا۔ وہی دن یا اس کے بعد کا دن چنیں۔

# Plan 17c. The entry of the garden was cancelled already.
# review: pending
err-garden-entry-cancelled = { -bagh } کا یہ اندراج پہلے ہی منسوخ ہو چکا ہے۔

## Plan 33 version 1.1: the date to sell, the goods that arrived at a garden.

# PRD FR-12.39. $entry_date: the day of the purchase or arrival.
# review: pending
err-sell-date-before-entry = { -sell-date } اس دن سے پہلے نہیں ہو سکتی جب مال آیا ({ $entry_date })۔ وہی دن یا اس کے بعد کا دن چنیں۔

# PRD FR-12.40. The goods arrived entry was cancelled already.
# review: pending
err-arrival-cancelled = یہ { -maal-aaya } کا اندراج پہلے ہی منسوخ ہو چکا ہے۔

# PRD FR-12.40. $qty_sold, $qty_written_off: what has been sold and written off from it.
# review: pending
err-arrival-has-sales = { -maal-aaya } کا یہ اندراج منسوخ نہیں ہو سکتا: { $qty_sold } { -nag } بک چکے ہیں اور { $qty_written_off } ضائع ہو چکے ہیں۔

# PRD FR-12.40. $qty_left: how many { -nag } of the arrival are not sold on the lines.
# review: pending
err-arrival-lot-not-fully-sold = { -bagh } کے مال کا یہ { -lot } اسی { -bikri } میں پورا بکنا چاہیے۔ لائنوں میں ابھی { $qty_left } { -nag } نہیں بکے۔ انہیں لائنوں میں بیچیں، یا کم { -nag } لیں۔ جو نہ لیں وہ { -maal-aaya } میں رہتا ہے۔

# PRD FR-12.40. $qty_left: how many { -nag } of garden goods are still left.
# review: pending
err-garden-has-stock = اس { -bagh } میں ابھی ایسا مال ہے جو بکا نہیں ({ $qty_left } { -nag })۔ بند کرنے سے پہلے اسے بیچیں یا ضائع کریں۔

# PRD FR-12.41. $min_other: the most that other shops have sold; $max_other: the { -nag } less what { -apni-dukan } has sold.
# review: pending
err-split-below-sold = { -dusri-dukan } کے { -nag } { $min_other } اور { $max_other } کے درمیان ہونے چاہییں۔ جو مال بک چکا وہ دوسرے حصے میں نہیں جا سکتا۔

# PRD FR-12.41. $qty: how many { -nag } the entry has in all.
# review: pending
err-split-above-qty = { -dusri-dukan } کے { -nag } اس اندراج کے کل { $qty } { -nag } سے زیادہ نہیں ہو سکتے۔

# PRD FR-12.41. $max: the most lines an entry can have.
# review: pending
err-too-many-lines = ایک اندراج میں زیادہ سے زیادہ { $max } لائنیں ہو سکتی ہیں۔

# PRD FR-10.10, plan 40. The person owes nothing, so there is nothing to move.
# review: pending
err-nothing-owed = اس کے ذمے کچھ نہیں، اس لیے ڈالنے کے لیے کچھ نہیں۔ اگر وہ جا چکا ہے تو صرف کھاتہ غیر فعال کر دیں۔

# PRD FR-10.10, plan 40.
# review: pending
err-dead-not-allowed = شراکت دار، { -alag-khata }، { -bagh } اور نظام کے کھاتے کو { -doobi-raqam } میں نہیں ڈالا جا سکتا۔

# PRD FR-10.10, plan 40.
# review: pending
err-already-dead = وہ پہلے ہی { -doobi-raqam } میں ہے۔ اسے فعال کرنے کے لیے «دوبارہ فعال کریں» استعمال کریں۔

# PRD FR-10.10, plan 40. The reason Other needs a note.
# review: pending
err-note-required = وجہ دیگر ہو تو نوٹ لکھنا ضروری ہے۔

# PRD FR-10.10, E72, plan 40. A line, a lot or an entry for a person who is in the dead amount.
# review: pending
err-person-in-dead-amount = یہ شخص { -doobi-raqam } میں ہے۔ اس کے لیے صرف { -wasooli } یا { -adhar-raqam } کی { -wapsi } لکھی جا سکتی ہے۔

# PRD FR-10.10, plan 40. $last_posting_date: the day of his last entry, DD-MM-YYYY.
# review: pending
err-date-before-last-posting = تاریخ اس کے آخری اندراج ({ $last_posting_date }) سے پہلے کی نہیں ہو سکتی۔
