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
