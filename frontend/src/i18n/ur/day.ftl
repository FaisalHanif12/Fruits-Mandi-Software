# Closing and reopening days (plan 10, PRD FR-21.1, FR-21.2).

# review: pending
day-title = دن بند کریں

# review: pending
day-today = آج کی تاریخ

# review: pending
day-closed-up-to = بند ہونے کی آخری تاریخ

# review: pending
day-none-closed = ابھی کوئی دن بند نہیں

# review: pending
day-load-failed = دن کی حالت کھل نہیں سکی

## Close.

# review: pending
day-close-title = دن بند کریں

# review: pending
day-close-help = دن بند ہونے کے بعد اس دن کے اندراجات اس وقت تک نہیں بدل سکتے جب تک مالک وہ دن دوبارہ نہ کھولے۔

# review: pending
day-close-up-to = اس تاریخ تک بند کریں

# review: pending
day-close-days = بند ہونے والے دن

# review: pending
day-close-from = پہلا کھلا دن

# review: pending
day-close-entries = ان دنوں کے اندراجات

# review: pending
day-close-trial-dr = { -trial-balance }: { -banaam } { -meezan }

# review: pending
day-close-trial-cr = { -trial-balance }: { -jama } { -meezan }

# review: pending
day-close-trial = { -trial-balance } کا فرق

# review: pending
day-close-cash = { -rokar } بقایا

# review: pending
day-close-nothing = اس تاریخ تک کوئی دن کھلا نہیں۔

# review: pending
day-close-button = دن بند کریں

# review: pending
day-close-preview-failed = دن بند کرنے کا حساب کھل نہیں سکا

# review: pending
day-close-confirm-title = دن بند کریں؟

# $date: the date to close up to. $days: how many days close.
# review: pending
day-close-confirm-description = { $days ->
    [one] { $date } تک کا { $days } دن بند ہو جائے گا۔ اس دن کے اندراجات اس وقت تک نہیں بدل سکیں گے جب تک آپ وہ دن دوبارہ نہ کھولیں۔
   *[other] { $date } تک کے { $days } دن بند ہو جائیں گے۔ ان دنوں کے اندراجات اس وقت تک نہیں بدل سکیں گے جب تک آپ وہ دن دوبارہ نہ کھولیں۔
}

# review: pending
day-closed = { $date } تک کے دن بند ہو گئے

## Reopen.

# review: pending
day-reopen-title = دن دوبارہ کھولیں

# review: pending
day-reopen-help = ایک تاریخ کھولنے سے اس کے بعد کی ہر تاریخ بھی کھل جاتی ہے۔ وجہ لکھنا ضروری ہے اور وہ آڈٹ لاگ میں لکھی جاتی ہے۔

# review: pending
day-reopen-date = کھولنے کی تاریخ

# review: pending
day-reopen-reason = وجہ

# review: pending
day-reopen-button = دوبارہ کھولیں

# review: pending
day-reopen-confirm-title = دن دوبارہ کھولیں؟

# review: pending
day-reopen-confirm-description = { $date } اور اس کے بعد کی ہر تاریخ کھل جائے گی۔ وجہ آڈٹ لاگ میں لکھی جائے گی۔

# review: pending
day-reopened = { $date } اور اس کے بعد کے دن کھل گئے

# review: pending
day-reopen-none-title = ابھی کوئی دن بند نہیں

# review: pending
day-reopen-none-help = جب آپ کوئی دن بند کریں گے تو اسے یہاں دوبارہ کھول سکیں گے۔

## The full summary of the days to close (PRD FR-21.1, 4.9).

# review: pending
day-summary-title = ان دنوں کا حاصل

# review: pending
day-summary-bikri-count = { -bikri } کی تعداد

# review: pending
day-summary-gross = { -kham-bikri }

# review: pending
day-summary-commission = { -commission } اور { -laga }، دونوں طرف

# review: pending
day-summary-shared-income = مشترکہ آمدن

# review: pending
day-summary-open-lots = { -baqi-maal } میں لاٹ

# review: pending
day-summary-open-qty = { -baqi-maal } میں { -nag }

# $lots: lots with a balance left. $qty: their nag. A reminder only: the day can still be closed.
# review: pending
day-summary-stock-left-note = { $lots ->
    [one] <num>{ $lots }</num> لاٹ کے <num>{ $qty }</num> { -nag } ابھی { -baqi-maal } میں ہیں۔ دن پھر بھی بند ہو سکتا ہے، یہ مال بعد میں بک سکتا ہے۔
   *[other] <num>{ $lots }</num> لاٹ کے <num>{ $qty }</num> { -nag } ابھی { -baqi-maal } میں ہیں۔ دن پھر بھی بند ہو سکتا ہے، یہ مال بعد میں بک سکتا ہے۔
}

# review: pending
day-summary-no-partners = دکھانے کے لیے { -shirakat-dar } کا کوئی حصہ نہیں۔

# review: pending
day-summary-partners-label = ان دنوں میں ہر { -shirakat-dar } کا حصہ

# review: pending
day-summary-col-partner = { -shirakat-dar }

# review: pending
day-summary-col-income = آمدن کا { -hissa }

# review: pending
day-summary-col-expense = { -kharcha } کا { -hissa }

# review: pending
day-summary-col-profit = منافع کا { -hissa }
