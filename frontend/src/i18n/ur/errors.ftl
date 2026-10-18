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
err-entry-too-large = یہ اندراج بہت بڑا ہے۔ ایک سطر، یا پورے اندراج کا { -meezan }، زیادہ سے زیادہ { $max } روپے ہو سکتا ہے۔
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
err-share-backdated = { -hissa } کی نئی تاریخ پرانی نہیں ہو سکتی۔ آج ({ $today }) یا اس کے بعد کی تاریخ لکھیں۔

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
err-user-inactive = اس کھاتے سے لاگ ان نہیں ہو سکتا۔ ریکوری کوڈ استعمال کریں۔
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
err-not-found = یہ چیز نہیں ملی۔ فہرست دوبارہ کھولیں۔
# PRD FR-7.66, E49.
# review: pending
err-spelling-unconfirmed = ان لفظوں کے ہجے کی تصدیق نہیں ہوئی: { $words }۔ ہجے دیکھیں، پھر تصدیق کریں یا درست کریں۔

## The master lists (plan 06).

# review: pending
err-code-taken = یہ کوڈ کسی اور کے پاس ہے۔ کوئی دوسرا کوڈ لکھیں۔
# review: pending
err-version-changed = آپ کے کھولنے کے بعد یہ اندراج بدل چکا ہے۔ اسے دوبارہ کھولیں۔
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

# PRD FR-10.9, FR-6.5, FR-13.5. Income and shop expense accounts, Opening Difference and Pukhta Khata cannot take one.
# review: pending
err-opening-not-allowed = اس کھاتے میں ابتدائی بقایا نہیں لکھا جا سکتا۔ آمدن اور { -dukan-kharcha } کے کھاتوں، { -naqdi-mandi }، { -pukhta-khata } اور ابتدائی فرق میں ابتدائی بقایا نہیں لگتا۔

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
err-rule-not-allowed = اصول کی یہ ترتیب درست نہیں۔ چارج اور طریقہ فہرست میں سے چنیں، اور گروپ صرف اسی وقت چنیں جب کوئی کھاتہ یا { -qism-maal } نہ چنا ہو۔ عام شرحیں ترتیبات کے چارجز میں بدلتی ہیں۔
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
err-cash-below-zero-not-confirmed = اس اندراج کے بعد { -rokar } { $cash_after } ہو جائے گی۔ محفوظ کرنے کے لیے اندراج کی تصدیق کریں۔
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

# PRD 15.1.
# review: pending
err-cheque-required = چیک کا نمبر اور چیک کی تاریخ لکھیں۔

# A warning (PRD FR-15.7): the save is allowed. $payable is what is payable, $over how much the
# amount is more than it, both in rupees.
# review: pending
err-payment-above-payable = یہ رقم { -beopari } کی قابل ادائیگی رقم ({ $payable }) سے { $over } زیادہ ہے۔ آپ پھر بھی محفوظ کر سکتے ہیں۔ زیادہ رقم { -beopari } کے ذمے دکھائی جائے گی، یہ { -peshgi } نہیں ہوگی۔

## بعد کی فروخت، ترمیم، منسوخی اور ضائع مال (پلان 16)۔

# review: pending
err-bikri-cancelled = یہ { -bikri } منسوخ ہو چکی ہے اور بدلی نہیں جا سکتی۔

# review: pending
err-lot-not-found = یہ { -lot } نہیں ملا۔ { -baqi-maal } کی فہرست کھول کر دوبارہ چنیں۔

# review: pending
err-nothing-to-sell = اس { -lot } میں بیچنے کے لیے کچھ باقی نہیں۔

# $available: { -lot } میں کتنے { -nag } باقی ہیں۔
# review: pending
err-write-off-exceeds-balance = باقی مال سے زیادہ ضائع نہیں ہو سکتا۔ صرف { $available } { -nag } باقی ہیں۔

# review: pending
err-date-before-received = فروخت کا دن مال وصول ہونے کے دن سے پہلے کا نہیں ہو سکتا۔

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
