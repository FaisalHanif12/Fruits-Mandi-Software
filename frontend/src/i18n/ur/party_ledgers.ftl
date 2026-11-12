### Party ledgers (plan 66, PRD FR-12.43): each buyer's and each seller's own ledger, in three levels.
### Level 1 is the list of parties, level 2 the statement of one party, level 3 the day slip.
### Every Urdu line says what its English line says (NFR-URD-29).

## The screen

# review: pending
party-ledgers-title = پارٹی کھاتے

# review: pending
party-ledgers-help = پہلے گاہک یا بیوپاری چنیں اور کسی پارٹی پر Enter دبائیں: اس کے کھاتے میں ہر اس تاریخ کی ایک سطر آئے گی جس دن اس کا کوئی اندراج ہوا، ایک وقت میں 30 دن۔ کسی تاریخ پر Enter دبائیں: اس دن کی پرچی اسی سطر کے نیچے کھل جائے گی اور وہیں سے پرنٹ بھی ہو سکتی ہے۔ نیچے کے بٹنوں یا Page Up اور Page Down سے 30 دن آگے پیچھے کریں۔ Esc سے فہرست پر واپس جاتے ہیں۔

# review: pending
party-ledgers-side-label = پارٹی کی قسم

# review: pending
party-ledgers-side-buyer = { -gahak }

# review: pending
party-ledgers-side-seller = { -beopari }

## Level 1: the list of parties

# review: pending
party-ledgers-search = کوڈ یا نام لکھیں

# review: pending
party-ledgers-list-label = پارٹی کھاتوں کی فہرست

# review: pending
party-ledgers-list-failed = پارٹی کھاتے نہیں کھل سکے

# review: pending
party-ledgers-col-code = کوڈ

# review: pending
party-ledgers-col-name = نام

# review: pending
party-ledgers-col-debit = { -banaam }

# review: pending
party-ledgers-col-credit = { -jama }

# review: pending
party-ledgers-empty-buyer = ابھی کوئی { -gahak } نہیں

# review: pending
party-ledgers-empty-seller = ابھی کوئی { -beopari } نہیں

# review: pending
party-ledgers-empty-help = انڈکس میں نیا گاہک یا بیوپاری شامل کریں۔ { -chukta-khata } کے گاہکوں کا اپنا کھاتہ نہیں ہوتا۔

# review: pending
party-ledgers-empty-action = انڈکس کھولیں

# review: pending
party-ledgers-no-match = کوئی پارٹی نہیں ملی

# review: pending
party-ledgers-no-match-help = کوئی اور کوڈ یا نام لکھ کر دیکھیں۔

# review: pending
party-ledgers-clear = تلاش ہٹائیں

## Level 2: the statement of one party

# review: pending
party-ledgers-back-list = پارٹیوں کی فہرست

# review: pending
party-ledgers-statement-label = اس پارٹی کا کھاتہ

# review: pending
party-ledgers-statement-failed = کھاتہ نہیں کھل سکا

# review: pending
party-ledgers-from = از

# review: pending
party-ledgers-to = تا

# review: pending
party-ledgers-col-date = تاریخ

# review: pending
party-ledgers-col-previous = { -sabiqa-raqam }

# review: pending
party-ledgers-col-buyer-charged = اس دن کا { -banaam }

# review: pending
party-ledgers-col-buyer-paid = اس دن ادا

# review: pending
party-ledgers-col-seller-charged = اس دن کا { -jama }

# review: pending
party-ledgers-col-seller-paid = اس دن اسے ادا

# review: pending
party-ledgers-col-balance = نیا { -baqaya }

# review: pending
party-ledgers-statement-no-match = اس مدت میں کوئی اندراج نہیں

# review: pending
party-ledgers-statement-no-match-help = پہلی اور آخری تاریخ بدل کر دیکھیں۔

# review: pending
party-ledgers-clear-dates = تاریخیں ہٹائیں

## Level 2: 30 days at a time and a day opened in place (PRD FR-12.47, plan 74)

# review: pending
party-ledgers-nav-label = 30 دن آگے پیچھے کریں

# review: pending
party-ledgers-back-days = پچھلے <num>{ $days }</num> دن

# review: pending
party-ledgers-forward-days = اگلے <num>{ $days }</num> دن

# review: pending
party-ledgers-range = <num>{ $from }</num> تا <num>{ $to }</num>

# review: pending
party-ledgers-nav-latest = پہلے ہی تازہ ترین دن

# review: pending
party-ledgers-nav-latest-button = آج تک کے دنوں پر واپس

# review: pending
party-ledgers-goto = تاریخ پر جائیں

# review: pending
party-ledgers-goto-hint = دن لکھیں، جیسے 05-11-2026۔ اس دن کا اندراج کھل جائے گا۔ مختصر شکل بھی چلتی ہے: 5 یعنی اس مہینے کی 5 تاریخ۔

# review: pending
party-ledgers-goto-placeholder = دن-مہینہ-سال

# review: pending
party-ledgers-goto-none = اس تاریخ کو کوئی اندراج نہیں: <num>{ $date }</num>۔ اس کے آس پاس کے دن نیچے ہیں۔

# review: pending
party-ledgers-window-empty = ان دنوں میں کوئی اندراج نہیں

# review: pending
party-ledgers-window-empty-help = نیچے پچھلے دنوں کا بٹن دبائیں، یا کسی تاریخ پر جائیں۔

# review: pending
party-ledgers-day-heading = <num>{ $date }</num> کی پرچی

# review: pending
party-ledgers-day-print-name = { $date } کی پرچی کا پرنٹ پیش نظارہ

## Level 3: the day slip

# review: pending
party-ledgers-back-statement = کھاتے پر واپس

# review: pending
party-ledgers-day-title = <name>{ $name }</name> <code>{ $code }</code>، <date>{ $date }</date>
