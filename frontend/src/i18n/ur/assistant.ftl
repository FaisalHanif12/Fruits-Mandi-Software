### Ask the app: the chat window above a round button at the end of every screen (plans 49 and 51, PRD 7.7, FR-7.72 to FR-7.88).
### Every Urdu line says what its English line says (NFR-URD-29). The words of an ANSWER are written by the
### model under the checks of FR-7.76; only the panel's own fixed words are here. Business words come from
### glossary.ftl.

## The chat window, its launcher and the status bar button (FR-5.5, FR-7.73).

# review: pending
assistant-button = سوال پوچھیں
# review: pending
assistant-button-count = آج <num>{ $used }</num> / <num>{ $limit }</num>
# review: pending
assistant-dock-label = سوال پوچھیں
# review: pending
assistant-dock-title = سوال پوچھیں
# review: pending
assistant-close = چیٹ کھڑکی بند کریں
# review: pending
assistant-larger = کھڑکی بڑی کریں
# review: pending
assistant-smaller = کھڑکی چھوٹی کریں
# review: pending
assistant-state-ready = تیار
# review: pending
assistant-state-working = جواب تیار ہو رہا ہے
# review: pending
assistant-state-offline = انٹرنیٹ نہیں ہے
# review: pending
assistant-state-off = بند ہے
# review: pending
assistant-new-conversation = نیا سوال
# review: pending
assistant-conversation-label = سوالات اور جوابات
# review: pending
assistant-box-label = اپنا سوال لکھیں
# review: pending
assistant-box-placeholder = اپنا سوال لکھیں اور { $key } دبائیں
# review: pending
assistant-send = بھیجیں
# review: pending
assistant-empty-title = السلام علیکم! اپنے حساب کے بارے میں مجھ سے کچھ بھی پوچھیں۔
# review: pending
assistant-empty-help = مثلاً: اکرم کا کتنا بقایا ہے؟ آج کا { -commission } کتنا ہوا؟ نیچے کے فوری سوالوں سے بھی شروع کر سکتے ہیں۔
# review: pending
assistant-off = سوال پوچھنا بند ہے۔ آپ اسے ترتیبات میں چالو کر سکتے ہیں۔ فوری سوال چلتے ہیں۔
# review: pending
assistant-no-key = سوال پوچھنے کے لیے OpenRouter کی کلید ابھی محفوظ نہیں۔ ترتیبات میں کلید لگائیں۔ فوری سوال چلتے ہیں۔
# review: pending
assistant-open-settings = ترتیبات کھولیں
# review: pending
assistant-status-failed = سوال پوچھنے کی حالت نہیں پڑھی جا سکی
# review: pending
assistant-count-label = آج کے سوالوں کی گنتی

## The working state: the first line at once, then one line per step (NFR-AI-3).

# review: pending
assistant-understanding = سوال سمجھ رہا ہوں…
# review: pending
assistant-working-label = جواب تیار ہو رہا ہے
# review: pending
assistant-step-generic = جواب تیار کر رہا ہوں…
# review: pending
assistant-step-finding = { $text } کو تلاش کر رہا ہوں…
# review: pending
assistant-step-reading-card = { $name } کا کارڈ دیکھ رہا ہوں…
# review: pending
assistant-step-reading-ledger = { $name } کا { -khata } دیکھ رہا ہوں…
# review: pending
assistant-step-reading-report = رپورٹ { $report } دیکھ رہا ہوں…
# review: pending
assistant-step-reading-analytics = { -tajziat } دیکھ رہا ہوں…
# review: pending
assistant-step-reading-tiles = ڈیش بورڈ کی رقمیں دیکھ رہا ہوں…
# review: pending
assistant-step-period = مدت کی تاریخیں نکال رہا ہوں…
# review: pending
assistant-step-calculating = حساب لگا رہا ہوں…
# review: pending
assistant-step-estimating = اندازہ لگا رہا ہوں…
# review: pending
assistant-step-checking = حساب جانچ رہا ہوں…
# review: pending
assistant-step-rewriting = جواب دوبارہ لکھ رہا ہوں…

## The quick questions (FR-7.80). They need no internet and no model, and they are not counted.

# review: pending
assistant-chips-label = فوری سوال
# review: pending
assistant-chip-sales = آج کی { -bikri }
# review: pending
assistant-chip-commission = آج کا { -commission }
# review: pending
assistant-chip-cash = موجود { -rokar }
# review: pending
assistant-chip-dues = گاہکوں کے ذمے
# review: pending
assistant-chip-total-dues = { -kul-agrahi }
# review: pending
assistant-chip-dead = { -doobi-raqam }
# review: pending
assistant-chip-payable = بیوپاریوں کو { -adaigi } باقی
# review: pending
assistant-chip-advances = پیشگی باقی
# review: pending
assistant-chip-stock = { -baqi-maal }
# review: pending
assistant-chip-party = کسی پارٹی کا { -khata }
# review: pending
assistant-party-pick-label = پارٹی چنیں
# review: pending
assistant-party-pick-hint = پارٹی کا کوڈ یا نام لکھیں
# review: pending
assistant-party-cancel = رہنے دیں

## The answer block (FR-7.74, FR-7.75, FR-7.76).

# review: pending
assistant-you-asked = آپ کا سوال
# review: pending
assistant-answer-label = جواب
# review: pending
assistant-checked = جانچا ہوا
# review: pending
assistant-direction-more = زیادہ
# review: pending
assistant-direction-less = کم
# review: pending
assistant-direction-same = برابر
# review: pending
assistant-table-label = { $source } کی قطاریں
# review: pending
assistant-table-more = { $count } میں سے پہلی { $shown } قطاریں دکھائی ہیں، باقی پوری رپورٹ میں ہیں
# review: pending
assistant-table-totals = { -meezan }
# review: pending
assistant-chart-label = ماہانہ رقوم کا چارٹ
# review: pending
assistant-chart-past = گزرے مہینے
# review: pending
assistant-chart-ahead = آنے والے مہینے (اندازہ)
# review: pending
assistant-sources-line = ماخذ: { $sources }، { $time }
# review: pending
assistant-open-report = پوری رپورٹ کھولیں
# review: pending
assistant-open-report-of = { $source }: پوری رپورٹ کھولیں
# review: pending
assistant-print = پرنٹ
# review: pending
assistant-print-of = { $source } پرنٹ کریں
# review: pending
assistant-next-label = اگلے سوال
# review: pending
assistant-rerun-note = یہ پرانا سوال آج کے کھاتوں پر دوبارہ چلایا گیا ہے۔ رقمیں اس وقت سے بدل چکی ہو سکتی ہیں۔

## The two bands (FR-7.74, FR-7.86, FR-7.88).

# review: pending
assistant-band-estimate = اندازہ، یقینی نہیں
# review: pending
assistant-band-advice = مشورہ، فیصلہ آپ کا
# review: pending
assistant-band-general = عام مشورہ
# review: pending
assistant-band-general-note = دکان کے لیے عام مشورہ، آپ کے کھاتوں سے نہیں
# review: pending
assistant-estimate-same-pace = اسی رفتار سے
# review: pending
assistant-estimate-same-growth = اسی اضافے سے
# review: pending
assistant-estimate-same-season = پچھلے سال جیسا
# review: pending
assistant-estimate-what-if = اگر رفتار <num>{ $factor }</num> ہو
# review: pending
assistant-estimate-what-if-any = اگر رفتار بدل جائے
# review: pending
assistant-chart-what-if = اگر رفتار { $factor } ہو
# review: pending
assistant-estimate-year-so-far = اس سال اب تک (آپ کے کھاتوں سے)
# review: pending
assistant-estimate-year-months = کھاتوں کے <num>{ $count }</num> مہینے، { $from } سے { $to } تک
# review: pending
assistant-estimate-year-month = کھاتوں کا ایک مہینہ: { $from }
# review: pending
assistant-estimate-year-left = باقی مہینوں کا اندازہ
# review: pending
assistant-estimate-year-total = پورا سال
# review: pending
assistant-estimate-year-note = موجودہ مہینہ پورے اندازے والے مہینے کے طور پر گنا گیا ہے؛ اس کے اب تک کے دن الگ سے شامل نہیں کیے گئے۔
# review: pending
assistant-estimate-months = بنیاد: <num>{ $count }</num> مہینے، { $from } سے { $to } تک
# review: pending
assistant-estimate-ahead = اندازے کے مہینے: { $from } سے { $to } تک
# review: pending
assistant-estimate-few-months = کھاتوں میں صرف <num>{ $count }</num> پورے مہینے ہیں، اس لیے اندازہ نہیں لگایا جا سکا۔

## The states that are not an answer (FR-7.77, FR-7.79, FR-7.81, FR-7.82).

# review: pending
assistant-which-one = کون سا؟
# review: pending
assistant-which-one-more = اور بھی ہیں، کوڈ لکھیں
# review: pending
assistant-not-found = یہ نام نہیں ملا
# review: pending
assistant-not-found-closest = سب سے قریب کے نام:
# review: pending
assistant-option-balance-label = بقایا
# review: pending
assistant-need-dates = مدت واضح نہیں۔ شروع اور آخری تاریخ لکھ کر دوبارہ پوچھیں، مثلاً 01-10-2026 سے 15-10-2026 تک۔
# review: pending
assistant-need-party = کس پارٹی کا؟ اس کا نام یا کوڈ لکھیں۔
# review: pending
assistant-outside-books = یہ سوال کھاتوں سے باہر ہے
# review: pending
assistant-outside-books-nearest = سب سے قریب کی رپورٹ:
# review: pending
assistant-change-refused = میں صرف بتا سکتا ہوں، کچھ لکھ نہیں سکتا
# review: pending
assistant-change-refused-help = نیچے کے بٹن سے اندراج کی اسکرین کھلتی ہے۔ رقم خالی رہتی ہے، آپ خود لکھتے ہیں۔
# review: pending
assistant-open-entry = اندراج کی اسکرین کھولیں
# review: pending
assistant-open-screen = اسکرین کھولیں
# review: pending
assistant-entry-neutral = رقم خالی رہتی ہے، آپ خود لکھتے ہیں
# review: pending
assistant-not-understood = سوال سمجھ نہیں آیا، دوسرے الفاظ میں پوچھیں
# review: pending
assistant-stopped-calls = میں کھاتے پورے نہیں پڑھ سکا۔ سوال چھوٹے حصوں میں پوچھیں، مثلاً ایک پارٹی یا ایک مہینہ۔
# review: pending
assistant-stopped-tokens = پڑھے ہوئے کھاتے ایک جواب کے لیے بہت بڑے تھے۔ سوال چھوٹے حصوں میں پوچھیں، مثلاً کم مدت۔
# review: pending
assistant-stopped-tables = جو ریکارڈ میں پڑھ سکا وہ نیچے ہیں۔
# review: pending
assistant-checks-failed = میں جانچا ہوا جواب نہیں لکھ سکا، نیچے اصل ریکارڈ ہے
# review: pending
assistant-records-differ = ریکارڈ میں فرق ملا، رپورٹ کھول کر دیکھیں
# review: pending
assistant-limit-reached = آج کے <num>{ $limit }</num> سوال پورے ہو گئے، کل دوبارہ پوچھیں
# review: pending
assistant-limit-chips = فوری سوال اب بھی چلتے ہیں۔
# review: pending
assistant-offline = سوال کے لیے انٹرنیٹ چاہیے، نیچے والے فوری سوال چلتے ہیں
# review: pending
assistant-error = جواب نہیں آ سکا، دوبارہ کوشش کریں
# review: pending
assistant-error-retry = دوبارہ پوچھیں
# review: pending
assistant-quick-failed = فوری سوال کا جواب نہیں آ سکا، دوبارہ کوشش کریں

## Settings, the group "سوال پوچھیں" (FR-7.85).

# review: pending
assistant-settings-title = سوال پوچھیں
# review: pending
assistant-settings-help = اپنے حساب کے بارے میں سوال پوچھیں۔ سوال اور اس کے لیے درکار ریکارڈ OpenRouter کو جاتے ہیں۔ فون نمبر، پتے، نوٹ اور وجوہات کبھی نہیں جاتے۔
# review: pending
assistant-settings-enabled = سوال پوچھنا چالو
# review: pending
assistant-settings-enabled-needs-key = پہلے کلید محفوظ کریں، پھر اسے چالو کریں۔
# review: pending
assistant-settings-key = OpenRouter کی کلید
# review: pending
assistant-settings-key-paste = کلید یہاں چسپاں کریں
# review: pending
assistant-settings-key-save = کلید محفوظ کریں
# review: pending
assistant-settings-key-saved = کلید محفوظ ہے: <num>••••{ $last4 }</num>
# review: pending
assistant-settings-key-none = کلید ابھی محفوظ نہیں
# review: pending
assistant-settings-key-replace = تبدیل کریں
# review: pending
assistant-settings-key-remove = ہٹائیں
# review: pending
assistant-settings-key-password = اپنا پاس ورڈ لکھیں
# review: pending
assistant-settings-key-password-help = کلید لگانے، بدلنے یا ہٹانے کے لیے آپ کا پاس ورڈ چاہیے۔
# review: pending
assistant-settings-key-confirm-replace = تبدیل کریں
# review: pending
assistant-settings-key-confirm-remove = ہٹائیں
# review: pending
assistant-settings-key-cancel = رہنے دیں
# review: pending
assistant-settings-model = ماڈل
# review: pending
assistant-settings-model-help = نیا ماڈل لگانے سے پہلے اس کی جانچ ضروری ہے، اور اس کا ڈیٹا نہ رکھنے والا Azure اینڈ پوائنٹ ہونا چاہیے۔
# review: pending
assistant-settings-model-save = ماڈل محفوظ کریں
# review: pending
assistant-settings-test = کنکشن جانچیں
# review: pending
assistant-settings-test-help = ایک طے شدہ سوال بھیج کر دیکھتا ہے۔ یہ بھی سوالوں کی گنتی میں آتا ہے۔
# review: pending
assistant-settings-test-needs-key = کنکشن جانچنے کے لیے پہلے کلید محفوظ کریں۔
# review: pending
assistant-settings-test-working = جانچ ہو رہی ہے…
# review: pending
assistant-settings-test-ok = کنکشن ٹھیک ہے: <num>{ $ms }</num> ملی سیکنڈ
# review: pending
assistant-settings-test-failed = کنکشن نہیں بن سکا: { $error }
# review: pending
assistant-settings-count = آج کی گنتی: <num>{ $used }</num> / <num>{ $limit }</num>
# review: pending
assistant-settings-limit-fixed = روزانہ حد طے شدہ ہے، ترتیب نہیں۔
# review: pending
assistant-settings-failed = سوال پوچھنے کی ترتیبات نہیں پڑھی جا سکیں
# review: pending
assistant-settings-save-failed = یہ تبدیلی محفوظ نہیں ہو سکی

## The question log in Settings (FR-7.84).

# review: pending
assistant-log-title = سوالات کا ریکارڈ
# review: pending
assistant-log-help = ہر سوال یہاں درج ہے۔ کسی سوال پر دبائیں تو وہ آج کے کھاتوں پر دوبارہ چلتا ہے۔ ریکارڈ بدلا یا صاف نہیں کیا جا سکتا۔
# review: pending
assistant-log-empty = ابھی کوئی سوال نہیں پوچھا گیا
# review: pending
assistant-log-empty-help = جب آپ سوال پوچھیں گے تو وہ یہاں نظر آئے گا۔
# review: pending
assistant-log-failed = سوالات کا ریکارڈ نہیں کھل سکا
# review: pending
assistant-log-col-time = وقت
# review: pending
assistant-log-col-question = سوال
# review: pending
assistant-log-col-kind = قسم
# review: pending
assistant-log-col-outcome = نتیجہ
# review: pending
assistant-log-col-tools = استعمال ہونے والے اوزار
# review: pending
assistant-log-col-counted = گنتی میں
# review: pending
assistant-log-kind-model = ماڈل کا سوال
# review: pending
assistant-log-kind-quick = فوری سوال
# review: pending
assistant-log-kind-pick = انتخاب کے بعد
# review: pending
assistant-log-kind-rerun = دوبارہ دیکھا
# review: pending
assistant-log-kind-test = کنکشن کی جانچ
# review: pending
assistant-log-counted-yes = گنا گیا
# review: pending
assistant-log-counted-no = نہیں گنا گیا
# review: pending
assistant-log-open = دوبارہ دیکھیں
# review: pending
assistant-log-open-row = دوبارہ دیکھیں: { $question }
# review: pending
assistant-log-no-tools = کوئی اوزار نہیں
# review: pending
assistant-log-col-took = کتنی دیر
# review: pending
assistant-log-took = <num>{ $total }</num> { $total_unit } (ماڈل <num>{ $model }</num> { $model_unit }، اوزار <num>{ $tools }</num> { $tools_unit })
# review: pending
assistant-log-took-total = <num>{ $total }</num> { $total_unit }
# review: pending
assistant-log-took-none = درج نہیں
# review: pending
assistant-unit-s = سیکنڈ
# review: pending
assistant-unit-ms = ملی سیکنڈ

## The outcome of a question, in the log (FR-7.84).

# review: pending
assistant-outcome-answered = جواب دیا گیا
# review: pending
assistant-outcome-estimate = اندازہ دیا گیا
# review: pending
assistant-outcome-advice = مشورہ دیا گیا
# review: pending
assistant-outcome-which-one = کون سا؟ پوچھا گیا
# review: pending
assistant-outcome-need-dates = تاریخیں مانگی گئیں
# review: pending
assistant-outcome-need-party = پارٹی مانگی گئی
# review: pending
assistant-outcome-not-found = نام نہیں ملا
# review: pending
assistant-outcome-outside-books = کھاتوں سے باہر
# review: pending
assistant-outcome-change-refused = تبدیلی سے انکار
# review: pending
assistant-outcome-not-understood = سمجھ نہیں آیا
# review: pending
assistant-outcome-checks-failed = جانچ پوری نہیں ہوئی
# review: pending
assistant-outcome-records-differ = ریکارڈ میں فرق
# review: pending
assistant-outcome-locked = کوڈ کے پیچھے
# review: pending
assistant-outcome-limit-reached = حد پوری
# review: pending
assistant-outcome-offline = انٹرنیٹ نہیں
# review: pending
assistant-outcome-error = خرابی

## Sources, buttons and the estimate band (FR-7.74, FR-7.86).

# review: pending
assistant-source-party-card = پارٹی کارڈ
# review: pending
assistant-source-analytics = { -tajziat } { $title }
# review: pending
assistant-sources-separator = ، 
# review: pending
assistant-open-dashboard = ڈیش بورڈ کھولیں
# review: pending
assistant-open-entry-party = { $name } کے لیے اندراج کی اسکرین کھولیں
# review: pending
assistant-estimate-per-month = ہر مہینے
# review: pending
assistant-estimate-step = ہر مہینے تبدیلی
# review: pending
assistant-estimate-total = کل
# review: pending
assistant-estimate-target = ہدف
# review: pending
assistant-estimate-target-reached = ہدف { $month } میں پورا ہو گا
# review: pending
assistant-estimate-target-not = ہدف ان مہینوں میں پورا نہیں ہوتا

## The working under ⓘ (FR-7.43, FR-7.75). The titles are the ones the backend names.

# review: pending
assistant-working-average-of = اوسط:
# review: pending
assistant-working-sum = رقموں کا میزان
# review: pending
assistant-working-difference = رقموں کا فرق
# review: pending
assistant-working-average = رقموں کا اوسط
# review: pending
assistant-working-percent-of = فیصد
# review: pending
assistant-working-change-percent = تبدیلی کا فیصد
# review: pending
assistant-working-same-pace = اسی رفتار سے: پچھلے مہینوں کا اوسط، مہینوں سے ضرب
# review: pending
assistant-working-year-parts = آخری سطر: اب تک کے کھاتوں کے مہینے + باقی مہینوں کا اندازہ
# review: pending
assistant-working-what-if-pace = اسی رفتار، تبدیلی کے ساتھ
# review: pending
assistant-working-same-growth = اسی اضافے سے: ہر مہینے کی اوسط تبدیلی، ہر مہینے میں جمع
# review: pending
assistant-working-same-season = پچھلے سال جیسا: پچھلے سال کے وہی مہینے

## What an estimate is about, what a figure is, and where its two places are (FR-7.75, FR-7.76).

# review: pending
assistant-measure-sales = { -kham-bikri }
# review: pending
assistant-measure-commission-seller = بیوپاری کا { -commission }
# review: pending
assistant-measure-commission-buyer = گاہک کا { -commission }
# review: pending
assistant-measure-commission-laga = { -laga }
# review: pending
assistant-measure-commission-all = کل { -commission } ({ -laga } سمیت)
# review: pending
assistant-measure-expenses = { -dukan-kharcha }
# review: pending
assistant-measure-profit = منافع
# review: pending
assistant-measure-receipts = { -wasooli }
# review: pending
assistant-measure-garden-sales = { -bagh } کی فروخت
# review: pending
assistant-measure-trip-sales = { -safar } کی فروخت
# review: pending
assistant-figure-party-balance = پارٹی کا بقایا
# review: pending
assistant-figure-buyers-dues = گاہکوں کے ذمے
# review: pending
assistant-figure-sellers-dues = بیوپاریوں کے ذمے
# review: pending
assistant-figure-dead-amount = { -doobi-raqam }
# review: pending
assistant-figure-total-dues = { -kul-agrahi }
# review: pending
assistant-figure-cash = موجود { -rokar }
# review: pending
assistant-figure-day-sales = دن کی { -bikri }
# review: pending
assistant-figure-day-commission = دن کا { -commission }
# review: pending
assistant-figure-partner-total = شراکت دار کی { -kul-raqam }
# review: pending
assistant-place-party-card = پارٹی کارڈ
# review: pending
assistant-place-r13-closing = R13 کا آخری بقایا
# review: pending
assistant-place-tile-buyers-dues = ڈیش بورڈ: گاہکوں کے ذمے
# review: pending
assistant-place-r20-total = R20 کا میزان
# review: pending
assistant-place-tile-sellers-dues = ڈیش بورڈ: بیوپاریوں کے ذمے
# review: pending
assistant-place-r24-total = R24 کا میزان
# review: pending
assistant-place-tile-dead-amount = ڈیش بورڈ: { -doobi-raqam }
# review: pending
assistant-place-r38-total = R38 کا میزان
# review: pending
assistant-place-tile-total-dues = ڈیش بورڈ: { -kul-agrahi }
# review: pending
assistant-place-tile-cash = ڈیش بورڈ: موجود { -rokar }
# review: pending
assistant-place-r12-closing = R12 کا آخری بقایا
# review: pending
assistant-place-tile-day = ڈیش بورڈ: آج کی رقم
# review: pending
assistant-place-r34 = R34 دن کا خلاصہ
# review: pending
assistant-place-view-a-total = { -tajziat } A کا میزان
# review: pending
assistant-place-partner-overview = شراکت داروں کی فہرست
# review: pending
assistant-place-r31-closing = R31 کا آخری میزان

## Why a model call failed (D6 Section 2, FailureKind). One reviewed message for each.

# review: pending
assistant-failure-offline = انٹرنیٹ نہیں ملا۔
# review: pending
assistant-failure-dropped = رابطہ ٹوٹ گیا۔ براہ کرم دوبارہ پوچھیں۔
# review: pending
assistant-failure-timeout = جواب میں بہت دیر ہو گئی۔ دوبارہ پوچھیں۔
# review: pending
assistant-failure-bad-key = OpenRouter نے کلید قبول نہیں کی۔ ترتیبات میں کلید دیکھیں۔
# review: pending
assistant-failure-no-credit = OpenRouter کھاتے میں رقم ختم ہے۔
# review: pending
assistant-failure-no-endpoint = ڈیٹا نہ رکھنے والے کسی Azure سرور نے یہ درخواست قبول نہیں کی۔ دیکھیں کہ اس ماڈل کے OpenRouter پر Azure سرور موجود ہیں، اور یہ کہ ایپ کوئی ایسی سیٹنگ نہیں بھیج رہی جو Azure قبول نہیں کرتا۔ ماڈل ترتیبات میں ہے۔
# review: pending
assistant-failure-busy = OpenRouter ابھی مصروف ہے۔ تھوڑی دیر بعد کوشش کریں۔
# review: pending
assistant-failure-server = OpenRouter کی طرف سے خرابی آئی۔
# review: pending
assistant-failure-bad-reply = ماڈل کا جواب پڑھا نہیں جا سکا۔
# review: pending
assistant-failure-other = نامعلوم خرابی۔
