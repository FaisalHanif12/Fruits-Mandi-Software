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
err-shares-not-100 = تمام { -shirakat-dar } کا کل { -hissa } 100 فیصد نہیں بنتا۔ مالک سے کہیں کہ { -shirakat-dar } کی ترتیب میں اسے درست کریں۔

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
err-advance-not-seller = ابتدائی { -peshgi-khata } صرف { -beopari } کے ابتدائی { -banaam } پر لگ سکتی ہے۔

# PRD FR-10.9, FR-6.5, FR-13.5. Income and shop expense accounts, Opening Difference and Pukhta Khata cannot take one.
# review: pending
err-opening-not-allowed = اس کھاتے میں ابتدائی بقایا نہیں لکھا جا سکتا۔ آمدن اور { -dukan-kharcha } کے کھاتوں، { -naqdi-mandi }، { -pukhta-khata } اور ابتدائی فرق میں ابتدائی بقایا نہیں لگتا۔

## Commission rules (plan 09).

# PRD 11.2, 11.3. The field says which part.
# review: pending
err-rule-not-allowed = اصول کی یہ ترتیب درست نہیں۔ چارج اور طریقہ فہرست میں سے چنیں، اور گروپ صرف اسی وقت چنیں جب کوئی کھاتہ یا { -qism-maal } نہ چنا ہو۔ عام شرحیں ترتیبات کے چارجز میں بدلتی ہیں۔
# review: pending
err-date-range = آخری تاریخ پہلی تاریخ سے پہلے ہے۔ آخری تاریخ پہلی تاریخ کے دن یا اس کے بعد کی لکھیں۔
