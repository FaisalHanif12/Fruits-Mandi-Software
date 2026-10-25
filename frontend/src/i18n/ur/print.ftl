### Prints (plan 14, PRD FR-12.12, FR-7.36, FR-19.3 to FR-19.7): the seller bill, the buyer slips,
### the پختہ slip, the print box and the Printing settings. The backend sends the ids of titles,
### facts, roles and amounts (plan 14 contract 6) and the template resolves them in the PRINT
### language, which can differ from the app language.

## Titles

# review: pending
print-seller-bill-title = { -beopari } کا بل

# review: pending
print-buyer-slip-title = { -gahak } کی پرچی

# review: pending
print-pukhta-slip-title = { -pukhta-khata } کی پرچی

## Facts under the title

# review: pending
print-fact-number = { -bikri } نمبر

# review: pending
print-fact-date = تاریخ

# review: pending
print-fact-bill-date = تاریخ بل

# review: pending
print-fact-truck = ٹرک نمبر

## Who the page is for

# review: pending
print-party-seller = { -beopari }

# review: pending
print-party-buyer = { -gahak }

# review: pending
print-party-onetime-buyer = { -pukhta-khata } ({ -kharidar })

# review: pending
print-party-code = کوڈ

# review: pending
print-party-phone = فون نمبر

## The amounts of the seller bill (PRD 4.4)

# review: pending
print-gross-sale = { -kham-bikri }

# review: pending
print-freight = { -kiraya }

# review: pending
print-commission = { -commission }

# review: pending
print-labour = { -mazdoori }

# review: pending
print-munshiana = { -munshiana }

# review: pending
print-cash-paid = { -naqd }

# review: pending
print-store-rent = { -kiraya-store }

# review: pending
print-phone-charge = { -phone }

# review: pending
print-total-deductions = کل اخراجات

# review: pending
print-net-sale = { -safi-bikri }

# The net is below zero (PRD FR-12.7): the amount is printed without a minus sign.
# review: pending
print-seller-owes = { -beopari } کے ذمے

## The amounts of a buyer slip

# review: pending
print-items-value = مال کی قیمت

# review: pending
print-buyer-commission = { -commission }

# review: pending
print-laga = { -laga }

# review: pending
print-buyer-pays = کل رقم

# review: pending
print-in-words = رقم حروف میں

# review: pending
print-payment-cash = { -naqdi } بکری: رقم ادا ہو چکی ہے

# review: pending
print-payment-credit = { -udhari } بکری: رقم باقی ہے

## Column heads of the goods table

# review: pending
print-col-date = تاریخ

# review: pending
print-col-buyer = { -gahak }

# review: pending
print-col-item = { -qism-maal }

# review: pending
print-col-packing = { -packing }

# review: pending
print-col-qty = تعداد

# review: pending
print-col-weight = وزن (کلو)

# review: pending
print-col-rate = ریٹ

# review: pending
print-col-amount = رقم

## The page footer and the stamp

# review: pending
print-printed-at = پرنٹ کا وقت

# review: pending
print-page-label = صفحہ

# review: pending
print-page-of = از

# review: pending
print-cancelled-stamp = منسوخ

# review: pending
print-preview-empty = اس دستاویز میں چھاپنے کے لیے کچھ نہیں۔

## The documents of a بکری (the names in the print box and in the Printing settings)

# review: pending
print-doc-seller-bill = { -beopari } کا بل

# review: pending
print-doc-buyer-slips = { -gahak } کی پرچیاں

# review: pending
print-doc-pukhta-slip = { -pukhta-khata } کی پرچی

## The print box (opened with F9 on the sale book)

# review: pending
print-box-title = { -bikri } { $number } پرنٹ کریں

# $key is the key that saves and prints, for example Ctrl+Enter.
# review: pending
print-box-help = یہاں جو دستاویزیں چنیں گے، وہی { -bikri } محفوظ کرتے وقت { $key } سے بھی پرنٹ ہوں گی۔

# review: pending
print-box-language = پرنٹ کی زبان

# review: pending
print-box-docs = دستاویزیں

# review: pending
print-box-none-ticked = کم از کم ایک دستاویز چنیں۔

# review: pending
print-box-only-buyer = کس { -gahak } کی پرچی

# review: pending
print-box-all-buyers = سب { -gahak }

# review: pending
print-box-copies = کاپیاں

# review: pending
print-box-copies-hint = خالی چھوڑیں تو ترتیبات والی تعداد استعمال ہوگی۔

# review: pending
print-box-to = کہاں بھیجنا ہے

# review: pending
print-box-to-printer = پرنٹر

# review: pending
print-box-to-pdf = PDF فائل

# review: pending
print-box-print = پرنٹ کریں

# review: pending
print-box-save-pdf = PDF محفوظ کریں

# review: pending
print-box-preview = پیش منظر

# review: pending
print-box-close = بند کریں

# $docs are the printed documents, for example "بیوپاری کا بل اور گاہک کی پرچیاں".
# review: pending
print-box-done-printed = پرنٹر کو بھیج دیا گیا: { $docs }

# review: pending
print-box-done-pdf = PDF فائلیں محفوظ ہو گئیں:

# review: pending
print-box-nothing-printed = چھاپنے کے لیے کچھ نہیں تھا۔

# review: pending
print-save-first = پرنٹ کرنے کے لیے پہلے { -bikri } محفوظ کریں۔

# Shown after Ctrl+Enter when the documents went to the printer.
# review: pending
print-toast-printed = پرنٹر کو بھیج دیا گیا

## The preview

# review: pending
print-preview-title = پیش منظر

# review: pending
print-preview-help = کاغذ پر دستاویز ایسے ہی چھپے گی۔

# review: pending
print-preview-failed = پیش منظر نہیں کھل سکا

## Settings, tab Printing (PRD Section 21)

# review: pending
print-settings-paper = کاغذ کا سائز

# review: pending
print-settings-paper-help = ہر دستاویز کے لیے کاغذ چنیں۔ پرنٹ اسی سائز پر ہوگا۔

# review: pending
print-paper-a4 = A4 کاغذ

# review: pending
print-paper-a5 = A5 کاغذ

# review: pending
print-paper-mm80 = 80 ملی میٹر رول

# review: pending
print-settings-printer = پرنٹر

# review: pending
print-settings-printer-default = طے شدہ پرنٹر

# review: pending
print-settings-printer-none = اس کمپیوٹر پر پرنٹروں کی فہرست نہیں ملی۔ طے شدہ پرنٹر استعمال ہوگا۔

# review: pending
print-settings-printers-failed = پرنٹروں کی فہرست نہیں کھل سکی

# review: pending
print-settings-copies = کاپیاں

# review: pending
print-settings-copies-hint = 1 سے 99 تک کی تعداد۔

# review: pending
print-settings-urdu-digits = اردو پرنٹ میں اردو ہندسے

# review: pending
print-settings-urdu-digits-hint = بند ہو تو اردو پرنٹ میں بھی انگریزی ہندسے (0 سے 9) چھپتے ہیں۔

# review: pending
print-settings-after-save = محفوظ کرنے کے بعد پرنٹ

# $key is the key that saves and prints, for example Ctrl+Enter.
# review: pending
print-settings-after-save-hint = { $key } دبانے پر { -bikri } یا واؤچر محفوظ ہو کر یہ دستاویزیں پرنٹ ہوں گی۔

# review: pending
print-settings-load-failed = پرنٹنگ کی ترتیبات نہیں کھل سکیں

## The receipt slip of a voucher (plan 15, PRD FR-15.2). The backend sends these ids (contract 9).

# review: pending
print-receipt-slip-title = { -wasooli } کی رسید

# review: pending
print-payment-slip-title = { -adaigi } کی رسید

# review: pending
print-advance-slip-title = { -peshgi-khata } کی رسید

# review: pending
print-fact-voucher-number = نمبر

# review: pending
print-fact-reason = وجہ

# review: pending
print-fact-collector = اگراہی والا

# review: pending
print-fact-narration = تفصیل

# review: pending
print-party-onetime-seller = { -pukhta-khata } ({ -beopari })

# review: pending
print-party-account = کھاتہ

# review: pending
print-slip-amount-received = وصول شدہ رقم

# review: pending
print-slip-amount-paid = ادا کی گئی رقم

# review: pending
print-slip-amount-advance = دی گئی { -peshgi }

# review: pending
print-doc-receipt-slip = رقم کی رسید

## R5 and the buyer slips in the client's layout; the header of every print (plan 17e, PRD R5, FR-19.13).

# Read by a screen reader: the name of the box in the header.
# review: pending
print-header-trade-mark = { -trade-mark }

# R5 (PRD 19.2), in the client's layout.
# review: pending
print-r5-title = { -udhari } تفصیل

# review: pending
print-r5-col-serial = نمبر

# review: pending
print-r5-col-party = پارٹی

# review: pending
print-r5-col-details = تفصیل

# review: pending
print-r5-col-value = رقم

# Value + buyer commission + لاگا (PRD C5).
# review: pending
print-r5-col-total = رقم بعد { -laga }

# One line of a buyer, the client's way: "(کیلا 20 . 600)". $item: the item name in the print language; $qty and $rate are numbers.
# review: pending
print-r5-line = ({ $item } { $qty } . { $rate })
