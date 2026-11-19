# Urdu review list

Generated 26-10-2026 by `tech-lead` from the Fluent files in `frontend/src/i18n/`. For the Urdu reviewer and the munshi (NFR-URD-4, NFR-URD-8, NFR-URD-29).

**How to review.** Every message below is marked `# review: pending` in its `ur` file. Read the Urdu next to its English. Check: the same meaning, the same numbers, the same business word from the glossary (Section 3 of the PRD), polite everyday mandi Urdu, one fixed spelling per word. Write the result in the last column: OK, or the better Urdu. A word with a better form goes back to `tech-lead`, who gives it to `frontend` to change in the `ur` file and then removes the `# review: pending` line. Words in `{ $name }` are filled by the app (a number, a name, a date). Business terms in curly braces were replaced by their glossary text here.

**Size.** 3314 messages in 30 files. Start with `glossary`, `common`, `errors`, `layout` and `print`: they appear on every screen.

| File | Messages |
|---|---|
| [accounts](#accounts) | 119 |
| [analytics](#analytics) | 151 |
| [assist](#assist) | 92 |
| [audit](#audit) | 98 |
| [auth](#auth) | 58 |
| [backup](#backup) | 161 |
| [bikri](#bikri) | 275 |
| [cashbook](#cashbook) | 191 |
| [common](#common) | 26 |
| [dashboard](#dashboard) | 91 |
| [day](#day) | 45 |
| [errors](#errors) | 156 |
| [export](#export) | 10 |
| [fruit](#fruit) | 91 |
| [gardens](#gardens) | 166 |
| [layout](#layout) | 66 |
| [masters](#masters) | 99 |
| [month](#month) | 50 |
| [partners](#partners) | 130 |
| [print](#print) | 122 |
| [pukhta_khata](#pukhta_khata) | 28 |
| [purchases](#purchases) | 75 |
| [reports](#reports) | 411 |
| [rules](#rules) | 137 |
| [search](#search) | 6 |
| [settings](#settings) | 106 |
| [translit](#translit) | 41 |
| [trips](#trips) | 130 |
| [ventures](#ventures) | 15 |
| [vouchers](#vouchers) | 168 |

## accounts

| ID | Urdu | English | Result |
|---|---|---|---|
| `accounts-title` | کھاتے | Accounts | |
| `accounts-help` | بیوپاری .gender = masculine، گاہک .gender = masculine، ادھار رقم .gender = feminine لینے والے اور باقی کھاتے۔ کھاتہ کھول کر دیکھیں یا بدلیں، یا نیا بنائیں۔ | Sellers, buyers, people who borrow money and the other accounts. Open an account to see or change it, or make a new one. | |
| `accounts-new` | نیا کھاتہ | New account | |
| `accounts-new-title` | نیا کھاتہ | New account | |
| `accounts-edit-title` | { $name } بدلیں | Change { $name } | |
| `accounts-saved` | کھاتہ محفوظ ہو گیا | Account saved | |
| `accounts-made-active` | کھاتہ فعال کر دیا گیا | Account made active | |
| `accounts-made-inactive` | کھاتہ غیر فعال کر دیا گیا | Account made inactive | |
| `accounts-make-active` | فعال کریں | Make active | |
| `accounts-make-inactive` | غیر فعال کریں | Make inactive | |
| `accounts-system-stays-active` | یہ نظام کا کھاتہ ہے۔ حساب کتاب میں اس کا استعمال ہوتا ہے، اس لیے اسے غیر فعال نہیں کیا جا سکتا۔ | This is a system account. The books use it, so it cannot be made inactive. | |
| `accounts-inactive-title` | غیر فعال کریں؟ | Make this inactive? | |
| `accounts-inactive-description` | { $name } نئے اندراج میں نہیں دکھایا جائے گا، لیکن اس کا پرانا حساب اور رپورٹیں ویسی ہی رہیں گی۔ جب چاہیں دوبارہ فعال کر سکتے ہیں۔ | { $name } will not be offered in new entries, but its old figures and the reports stay as they are. You can make it active again at any time. | |
| `accounts-inactive-note` | یہ کھاتہ غیر فعال ہے، اس لیے نئے اندراج میں نہیں دکھایا جاتا۔ دوبارہ استعمال کرنے کے لیے فعال کریں۔ | This account is inactive, so it is not offered in new entries. Make it active to use it again. | |
| `accounts-status-inactive` | غیر فعال | Inactive | |
| `accounts-load-failed` | کھاتے نہیں کھل سکے | The accounts could not be opened | |
| `accounts-empty` | ابھی کوئی کھاتہ نہیں | No accounts yet | |
| `accounts-empty-help` | پہلا کھاتہ بنانے کے لیے { $key } دبائیں۔ | Press { $key } to make the first account. | |
| `accounts-no-match-title` | کوئی کھاتہ نہیں ملا | No account found | |
| `accounts-no-match-help` | کوئی اور کوڈ یا نام لکھ کر دیکھیں، یا غیر فعال کھاتے بھی دکھائیں۔ | Try another code or name, or show the inactive accounts too. | |
| `accounts-no-match-clear` | تلاش صاف کریں | Clear the search | |
| `accounts-not-found-title` | یہ کھاتہ نہیں ملا | This account was not found | |
| `accounts-not-found-help` | ہو سکتا ہے یہ بدل چکا ہو۔ کھاتوں کی فہرست دوبارہ کھولیں۔ | It may have been changed. Open the list of accounts again. | |
| `accounts-filter-type` | قسم | Type | |
| `accounts-filter-label` | اس فہرست میں تلاش | Search this list | |
| `accounts-show-inactive` | غیر فعال بھی دکھائیں | Show the inactive ones too | |
| `accounts-more-columns` | مزید کالم | More columns | |
| `accounts-type-all` | سب | All | |
| `accounts-type-seller` | بیوپاری .gender = masculine | Seller | |
| `accounts-type-buyer` | گاہک .gender = masculine | Buyer | |
| `accounts-type-money_lent` | ادھار رقم .gender = feminine | Money Lent | |
| `accounts-type-other` | دیگر | Other | |
| `accounts-weekday-sat` | ہفتہ | Saturday | |
| `accounts-weekday-sun` | اتوار | Sunday | |
| `accounts-weekday-mon` | پیر | Monday | |
| `accounts-weekday-tue` | منگل | Tuesday | |
| `accounts-weekday-wed` | بدھ | Wednesday | |
| `accounts-weekday-thu` | جمعرات | Thursday | |
| `accounts-weekday-fri` | جمعہ | Friday | |
| `accounts-col-code` | کوڈ | Code | |
| `accounts-col-name` | نام | Name | |
| `accounts-col-other-name` | انگریزی نام | Urdu name | |
| `accounts-col-city` | شہر | City | |
| `accounts-col-opening-date` | ابتدائی تاریخ | Opening date | |
| `accounts-col-opening-dr` | ابتدائی بنام | Opening Debit | |
| `accounts-col-opening-cr` | ابتدائی جمع | Opening Credit | |
| `accounts-col-credit` | ادھار کی اجازت | Credit allowed | |
| `accounts-col-balance` | موجودہ بقایا | Current balance | |
| `accounts-col-side` | طرف | Side | |
| `accounts-total-dr` | کل ابتدائی بنام | Total opening Debit | |
| `accounts-total-cr` | کل ابتدائی جمع | Total opening Credit | |
| `accounts-section-account` | کھاتہ | Account | |
| `accounts-section-names` | نام | Names | |
| `accounts-section-contact` | رابطہ | Contact | |
| `accounts-section-opening` | ابتدائی بقایا | Opening balance | |
| `accounts-section-credit` | ادھاری اور اگراہی .gender = feminine | Credit and Collection | |
| `accounts-section-more` | مزید | More | |
| `accounts-field-type` | کھاتے کی قسم | Account type | |
| `accounts-field-control` | کنٹرول | Control | |
| `accounts-field-code` | کوڈ | Code | |
| `accounts-field-code-hint` | اگلا کوڈ پہلے سے لکھا ہوا ہے۔ آپ اسے بدل سکتے ہیں۔ | The next code is already filled in. You can change it. | |
| `accounts-field-name-ur` | اردو نام | Urdu name | |
| `accounts-field-name-en` | انگریزی نام | English name | |
| `accounts-field-tag` | نشان | Tag | |
| `accounts-field-tag-hint` | اختیاری۔ 1 سے 4 انگریزی بڑے حروف، مثلاً K یا BA۔ یہ نام کے بعد دکھایا جاتا ہے اور ایک جیسے نام والے دو لوگوں میں فرق بتاتا ہے۔ | Optional. 1 to 4 English capital letters, for example K or BA. It is shown after the name and tells two people with the same name apart. | |
| `accounts-field-father-ur` | ولدیت یا فرم، اردو | Father or firm name, Urdu | |
| `accounts-field-father-en` | ولدیت یا فرم، انگریزی | Father or firm name, English | |
| `accounts-field-city` | شہر | City | |
| `accounts-field-adda` | اڈا .gender = masculine | Adda | |
| `accounts-field-phone1` | فون نمبر | Phone number | |
| `accounts-field-phone2` | دوسرا فون نمبر | Second phone number | |
| `accounts-field-address` | پتہ | Address | |
| `accounts-field-opening-date` | ابتدائی تاریخ | Opening date | |
| `accounts-field-opening-dr` | ابتدائی بنام | Opening Debit | |
| `accounts-field-opening-cr` | ابتدائی جمع | Opening Credit | |
| `accounts-field-opening-advance` | یہ ابتدائی رقم پیشگی کھاتہ .gender = masculine ہے | This opening amount is an Advance | |
| `accounts-field-credit-allowed` | ادھار کی اجازت ہے | Credit is allowed | |
| `accounts-field-credit-limit` | ادھار کی حد | Credit limit | |
| `accounts-field-collector` | اگراہی .gender = feminine والا | Collector | |
| `accounts-field-weekday` | اگراہی .gender = feminine کا دن | Collection day | |
| `accounts-field-linked` | منسلک کھاتہ | Linked account | |
| `accounts-field-linked-hint` | ایک ہی شخص بیوپاری .gender = masculine بھی ہو اور گاہک .gender = masculine بھی تو اس کے دونوں کھاتے جوڑ دیں۔ | If one person is both a seller and a buyer, link his two accounts. | |
| `accounts-field-marka` | طے شدہ مارکہ .gender = masculine | Default Mark | |
| `accounts-field-note` | نوٹ | Note | |
| `accounts-similar-title` | ملتے جلتے کھاتے پہلے سے موجود ہیں | Similar accounts already exist | |
| `accounts-similar-tag-help` | ایک ہی نام کے دو لوگوں میں فرق نشان سے ہوتا ہے، مثلاً اسلم BA اور اسلم۔ صرف وہی نام، نشان اور فون ایک ہوں تو نیا کھاتہ دہرا مانا جاتا ہے۔ | Two people with the same name are told apart by the tag, for example Aslam BA and Aslam. Only the same name, tag and phone is a duplicate. | |
| `accounts-tree-label` | کھاتوں کا درخت | Chart of accounts tree | |
| `chart-title` | کھاتوں کی ترتیب | Chart of accounts | |
| `chart-help` | مدیں، کنٹرول اور کھاتے، ہر ایک کے موجودہ بقایا کے ساتھ۔ تیر والی کلیدوں سے کھولیں اور بند کریں۔ | Heads, controls and accounts, each with its current balance. Open and close them with the arrow keys. | |
| `chart-tabs` | ترتیب کے حصے | Parts of the chart | |
| `chart-tab-tree` | درخت | Tree | |
| `chart-tab-heads` | مدیں | Heads | |
| `chart-tab-controls` | کنٹرول | Controls | |
| `chart-load-failed` | کھاتوں کی ترتیب نہیں کھل سکی | The chart of accounts could not be opened | |
| `masters-accounts-title` | کھاتے | Accounts | |
| `masters-accounts-help` | بیوپاری .gender = masculine، گاہک .gender = masculine اور باقی کھاتے۔ | Sellers, buyers and the other accounts. | |
| `masters-chart-title` | کھاتوں کی ترتیب | Chart of accounts | |
| `masters-chart-help` | مدیں، کنٹرول اور کھاتے درخت کی صورت میں۔ | Heads, controls and accounts as a tree. | |
| `openings-open` | ابتدائی بقایا لکھیں | Write opening balances | |
| `openings-title` | ابتدائی بقایا | Opening balances | |
| `openings-help` | ہر کھاتے کے سامنے اس کا ابتدائی بنام یا جمع لکھیں۔ جن کھاتوں کا کچھ نہیں، انہیں خالی چھوڑ دیں۔ | Write the opening Debit or Credit against each account. Leave the accounts that have none empty. | |
| `openings-grid-label` | ابتدائی بقایا کی فہرست | Opening balances list | |
| `openings-date-all` | نئے ابتدائی بقایا کی تاریخ | Date for new opening balances | |
| `openings-date-all-hint` | جن کھاتوں کا ابتدائی بقایا ابھی نہیں، انہیں یہی تاریخ ملتی ہے۔ | Accounts with no opening balance yet get this date. | |
| `openings-summary-title` | بدلی ہوئی سطروں کا میزان | Total of the lines you changed | |
| `openings-changed` | { $count -> [one] ایک کھاتہ بدلا ہے *[other] { $count } کھاتے بدلے ہیں | { $count -> [one] One account changed *[other] { $count } accounts changed | |
| `openings-none-changed` | ابھی کوئی تبدیلی نہیں | Nothing changed yet | |
| `openings-difference` | ابتدائی فرق | Opening Difference | |
| `openings-difference-help` | جب سب ابتدائی بقایا درست ہوں تو یہ صفر ہوتا ہے۔ | It is zero when all the opening balances are right. | |
| `openings-saved` | ابتدائی بقایا محفوظ ہو گئے | Opening balances saved | |
| `openings-load-failed` | ابتدائی بقایا کھل نہیں سکے | The opening balances could not be opened | |
| `openings-empty` | ابھی کوئی کھاتہ نہیں | No accounts yet | |
| `openings-empty-help` | پہلے انڈکس میں کھاتہ بنائیں، پھر اس کا ابتدائی بقایا لکھیں۔ | Make an account in Index first, then write its opening balance. | |
| `openings-errors-title` | ان سطروں میں مسئلہ ہے | These lines have a problem | |
| `openings-problem` | { $account } ({ $field }): { $message } | { $account } ({ $field }): { $message } | |
| `openings-locked-help` | جن کھاتوں کے خانے بند ہیں، ان کا ابتدائی بقایا نہیں لگ سکتا۔ | An account whose cells are locked cannot have an opening balance. | |
| `openings-name-advance` | { $name } (پیشگی کھاتہ .gender = masculine) | { $name } (Advance) | |
| `accounts-similar-duplicate-title` | یہ کھاتہ پہلے سے موجود ہو سکتا ہے | This account may already exist | |
| `accounts-similar-duplicate-chip` | نام، نشان اور فون ایک جیسے ہیں | Same name, tag and phone | |

## analytics

| ID | Urdu | English | Result |
|---|---|---|---|
| `analytics-title` | تجزیات | Analytics | |
| `analytics-views-label` | تجزیات کی اقسام | Analytics views | |
| `analytics-catalog-failed` | تجزیات کی فہرست نہیں کھل سکی | The list of analytics views could not be opened | |
| `analytics-view-unknown` | یہ تجزیہ موجود نہیں | This view does not exist | |
| `analytics-view-unknown-help` | اوپر سے کوئی تجزیہ چنیں۔ آپ کا کوئی ڈیٹا متاثر نہیں ہوا۔ | Pick one of the views above. None of your data is affected. | |
| `analytics-back-to-views` | پہلے تجزیے پر جائیں | Go to the first view | |
| `analytics-view-a-title` | قسم مال وار | By Item | |
| `analytics-view-a-help` | ہر قسم مال کا کمیشن .gender = masculine: تعداد، خام بکری .gender = feminine، اوسط ریٹ، دونوں طرف کا کمیشن .gender = masculine، لاگا .gender = masculine اور کل کمیشن .gender = masculine میں فیصد۔ | Commission of each item: quantity, Gross Sale, average rate, the Commission of both sides, Laga and the share of all Commission. | |
| `analytics-view-b-title` | بیوپاری وار | By Seller | |
| `analytics-view-b-help` | ہر بیوپاری .gender = masculine: بکری، تعداد، خام بکری .gender = feminine، کمیشن .gender = masculine اور صافی بکری .gender = feminine۔ | Each Seller: sales, quantity, Gross Sale, Commission and Net Sale. | |
| `analytics-view-c-title` | گاہک وار | By Buyer | |
| `analytics-view-c-help` | ہر گاہک .gender = masculine: خریداری، کمیشن .gender = masculine، لاگا .gender = masculine، موجودہ بقایا اور ادائیگی میں لگنے والے اوسط دن۔ | Each Buyer: purchases, Commission, Laga, what he owes now and how many days he takes to pay. | |
| `analytics-view-d-title` | قسم مال × مدت | Item by Period | |
| `analytics-view-d-help` | ہر قسم مال کا کمیشن .gender = masculine دن، مہینے یا سال کے حساب سے۔ | The Commission of each item by day, by month or by year. | |
| `analytics-view-e-title` | بیوپاری × قسم مال | Seller × Item | |
| `analytics-view-e-help` | ہر بیوپاری .gender = masculine کا کمیشن .gender = masculine قسم مال کے حساب سے۔ | The Commission of each Seller by item. | |
| `analytics-view-h-title` | سرفہرست 10 | Top 10 | |
| `analytics-view-h-help` | کمیشن .gender = masculine کے لحاظ سے سرفہرست اقسام، بیوپاری اور گاہک۔ | The top items, sellers and buyers by Commission. | |
| `analytics-col-share` | کل کمیشن .gender = masculine میں فیصد | Share of Commission | |
| `analytics-col-bikri-count` | بکری | Sales | |
| `analytics-col-purchase-value` | خریداری کی رقم | Purchase value | |
| `analytics-col-dues-now` | موجودہ بقایا | Dues now | |
| `analytics-col-avg-days-to-pay` | ادائیگی کے اوسط دن | Days to pay (average) | |
| `analytics-col-compare` | موازنے کی مدت | Compared period | |
| `analytics-col-change` | فرق | Change | |
| `analytics-col-rank` | نمبر | Rank | |
| `analytics-col-name` | نام | Name | |
| `analytics-col-period` | مدت | Period | |
| `analytics-col-item-column` | قسم مال | Item | |
| `analytics-col-row-total` | کل کمیشن .gender = masculine | Total Commission | |
| `analytics-col-other-items` | باقی اقسام | Other items | |
| `analytics-top-items` | سرفہرست اقسام | Top items | |
| `analytics-top-sellers` | سرفہرست بیوپاری | Top sellers | |
| `analytics-top-buyers` | سرفہرست گاہک | Top buyers | |
| `analytics-filters-label` | تجزیات کے فلٹر | Analytics filters | |
| `analytics-period-this-year` | یہ سال | This year | |
| `analytics-period-last-year` | پچھلا سال | Last year | |
| `analytics-group-by-month` | مہینہ | Month | |
| `analytics-group-by-day` | دن | Day | |
| `analytics-group-by-year` | سال | Year | |
| `analytics-filter-buyer` | گاہک .gender = masculine | Buyer | |
| `analytics-filter-side` | جانب | Side | |
| `analytics-side-both` | دونوں جانب | Both sides | |
| `analytics-side-seller` | صرف بیوپاری .gender = masculine جانب | Seller side only | |
| `analytics-side-buyer` | صرف گاہک .gender = masculine جانب | Buyer side only | |
| `analytics-filter-compare` | موازنہ | Compare | |
| `analytics-compare-off` | موازنہ نہیں | No comparison | |
| `analytics-compare-previous` | اس سے پہلے کی مدت | The period before | |
| `analytics-compare-last-year` | پچھلے سال کی یہی مدت | The same period last year | |
| `analytics-compare-range` | موازنہ: { $from } تا { $to } | Compared with { $from } to { $to } | |
| `analytics-filter-by-year` | سال کے حساب سے | By year | |
| `analytics-filter-compare-from` | موازنہ از | Compared with, from | |
| `analytics-filter-compare-to` | موازنہ تا | Compared with, to | |
| `analytics-grid-label` | تجزیات کی جدول | Analytics table | |
| `analytics-load-failed` | تجزیہ نہیں کھل سکا | The analytics could not be opened | |
| `analytics-empty` | ان فلٹروں میں کوئی بکری نہیں | No sales for these filters | |
| `analytics-empty-help` | تاریخیں بدل کر یا فلٹر کم کر کے دیکھیں۔ | Try other dates or fewer filters. | |
| `analytics-open-hint` | رقم چننے کے لیے تیر کی کیز اور اس کی تفصیل دیکھنے کے لیے Enter دبائیں۔ | Use the arrow keys to pick a figure and Enter to see the lines behind it. | |
| `analytics-all-lines` | تمام لائنیں | All lines | |
| `analytics-drill-all` | ان فلٹروں کی ہر لائن | Every line of these filters | |
| `analytics-lines-title` | اس رقم کی تفصیل | Lines behind the figure | |
| `analytics-lines-back` | جدول پر واپس | Back to the table | |
| `analytics-lines-failed` | تفصیل نہیں کھل سکی | The lines could not be opened | |
| `analytics-lines-empty` | اس رقم کے پیچھے کوئی لائن نہیں | There are no lines behind this figure | |
| `analytics-lines-empty-help` | ہو سکتا ہے کسی اور اندراج سے رقم بدل گئی ہو۔ واپس جا کر دوبارہ دیکھیں۔ | The figure may have been changed by another entry. Go back and look again. | |
| `analytics-lines-hint` | لائن چننے کے لیے ↑ ↓ اور اس کی بکری .gender = feminine کھولنے کے لیے Enter دبائیں۔ واپس کی گئی بکری کی رقم صفر سے نیچے دکھائی جاتی ہے، تاکہ لائنیں رقم کے برابر رہیں۔ | Use ↑ ↓ to pick a line and Enter to open its Sale. A reversal shows its amounts below zero, so the lines add up to the figure. | |
| `analytics-lines-grid-label` | اس رقم کی تفصیل | Lines behind the figure | |
| `analytics-columns-shown` | <num>{ $total }</num> میں سے مدتیں <num>{ $from }</num> تا <num>{ $to }</num> | Periods <num>{ $from }</num> to <num>{ $to }</num> of <num>{ $total }</num> | |
| `analytics-columns-shown-items` | <num>{ $total }</num> میں سے اقسام <num>{ $from }</num> تا <num>{ $to }</num> | Items <num>{ $from }</num> to <num>{ $to }</num> of <num>{ $total }</num> | |
| `analytics-columns-earlier` | پہلے | Earlier | |
| `analytics-columns-later` | بعد کی | Later | |
| `analytics-columns-fit` | اس ونڈو میں <num>{ $total }</num> میں سے <num>{ $shown }</num> کالم آتے ہیں۔ پرنٹ میں سب کالم ہوتے ہیں۔ | <num>{ $shown }</num> of <num>{ $total }</num> columns fit this window. The print has every column. | |
| `analytics-columns-other` | باقی کالم دکھائیں | Show the other columns | |
| `analytics-columns-main` | اصل کالم پر واپس | Back to the main columns | |
| `analytics-chart-a` | قسم مال کے لحاظ سے کل کمیشن .gender = masculine | Total Commission by item | |
| `analytics-chart-b` | بیوپاری کے لحاظ سے کل کمیشن .gender = masculine | Total Commission by seller | |
| `analytics-chart-c` | گاہک کے لحاظ سے خریداری کی رقم | Purchase value by buyer | |
| `analytics-chart-d` | مدت کے لحاظ سے کمیشن .gender = masculine، ہر قسم مال کا الگ حصہ | Commission by period, one part for each item | |
| `analytics-chart-e` | بیوپاری کے لحاظ سے کمیشن .gender = masculine، ہر قسم مال کا الگ حصہ | Commission by seller, one part for each item | |
| `analytics-chart-h` | کمیشن .gender = masculine کے لحاظ سے سرفہرست اقسام | Top items by Commission | |
| `analytics-chart-limit` | چارٹ میں { $total } میں سے پہلی { $shown } قطاریں دکھائی گئی ہیں۔ | The chart shows the first { $shown } of { $total } rows. | |
| `analytics-saved-label` | محفوظ کیے ہوئے تجزیے | Saved views | |
| `analytics-saved-none` | ابھی کوئی محفوظ تجزیہ نہیں | No saved views yet | |
| `analytics-saved-open` | محفوظ تجزیہ کھولیں | Open a saved view | |
| `analytics-saved-save` | یہ تجزیہ محفوظ کریں | Save this view | |
| `analytics-saved-delete` | یہ محفوظ تجزیہ ہٹائیں | Delete this saved view | |
| `analytics-saved-failed` | محفوظ تجزیے نہیں پڑھے جا سکے | The saved views could not be read | |
| `analytics-saved-dialog-title` | یہ تجزیہ محفوظ کریں | Save this view | |
| `analytics-saved-name` | نام | Name | |
| `analytics-saved-name-hint` | مثال: کیلا کمیشن ماہانہ | For example: Banana commission, monthly | |
| `analytics-saved-confirm` | محفوظ کریں | Save | |
| `analytics-saved-done` | "{ $name }" محفوظ ہو گیا | "{ $name }" is saved | |
| `analytics-saved-deleted` | محفوظ تجزیہ ہٹا دیا گیا | The saved view is deleted | |
| `analytics-saved-delete-title` | یہ محفوظ تجزیہ ہٹائیں؟ | Delete this saved view? | |
| `analytics-saved-delete-body` | "{ $name }" آپ کے محفوظ تجزیوں سے ہٹ جائے گا۔ آپ کی رقمیں متاثر نہیں ہوں گی۔ | "{ $name }" will be removed from your saved views. Your figures are not affected. | |
| `analytics-view-f-title` | شراکت دار وار | By Partner | |
| `analytics-view-f-help` | ہر شراکت دار: آمدن کا حصہ، خرچہ کا حصہ، منافع حصہ، باغات، سفر اور پھل سے تبدیلی اور اس کی کل رقم میں خالص تبدیلی۔ | Each Partner: income share, expense share, profit share, what his Gardens, trips and Fruit Taken changed and the net change of his Partner Total. | |
| `analytics-view-g-title` | رجحان | Trend | |
| `analytics-view-g-help` | ہر دن، مہینے یا سال کا: خام بکری .gender = feminine، کمیشن .gender = masculine، مشترکہ آمدن، دکان خرچہ، منافع اور سفر کا نتیجہ۔ | Each day, month or year: Gross Sale, Commission, shared income, shop expenses, profit and the result of the trips. | |
| `analytics-view-j-title` | مارکیٹ سفر .gender = masculine | Market Trip | |
| `analytics-view-j-help` | ہر سفر، خریداری، قسم مال یا دن کا: کیا خریدا اور بیچا گیا، لاگت، نتیجہ اور ہر شراکت دار کا حصہ۔ | Each trip, purchase, item or day: what was bought and sold, the cost, the result and the part of each Partner. | |
| `analytics-col-gardens` | باغ | Garden | |
| `analytics-col-trips` | سفر | Trip | |
| `analytics-col-fruit` | پھل | Fruit Taken | |
| `analytics-col-other-money` | دیگر | Other | |
| `analytics-col-net-change` | خالص تبدیلی | Net Change | |
| `analytics-col-shared-income` | مشترکہ آمدن | Shared Income | |
| `analytics-col-shop-expenses` | دکان خرچہ .gender = masculine | Shop Expense | |
| `analytics-col-profit` | منافع | Profit | |
| `analytics-col-trips-result` | سفر کا نتیجہ | Trip Result | |
| `analytics-col-partner-share` | حصہ .gender = masculine | Share | |
| `analytics-col-partner-heading` | { $name }: { $title } | { $name }: { $title } | |
| `analytics-col-partner-part` | نتیجے میں حصہ | Part of Result | |
| `analytics-col-share-kind` | قسم | Kind | |
| `analytics-col-share-percent` | حصہ .gender = masculine | Share | |
| `analytics-col-share-base` | تقسیم ہونے والی رقم | Amount Shared | |
| `analytics-col-share-amount` | اس کا حصہ | His Share | |
| `analytics-filter-trip` | سفر | Trip | |
| `analytics-trip-all` | تمام سفر | All trips | |
| `analytics-trip-failed` | سفروں کی فہرست نہیں کھل سکی | The list of trips could not be read | |
| `analytics-filter-goods` | مال | Goods | |
| `analytics-goods-all` | تمام مال | All goods | |
| `analytics-goods-consigned` | بیوپاری کا مال | Consigned goods | |
| `analytics-goods-trip` | سفر کا مال | Trip goods | |
| `analytics-goods-garden` | باغ کا مال | Garden goods | |
| `analytics-filter-trip-by-purchase` | خریداری وار | By purchase | |
| `analytics-chart-f` | ہر شراکت دار کا منافع حصہ اور خالص تبدیلی | Profit share and net change of each Partner | |
| `analytics-chart-g` | ہر مدت کا کمیشن .gender = masculine، مشترکہ آمدن، دکان خرچہ، منافع اور سفر کا نتیجہ | Each period: Commission, shared income, shop expenses, profit and the result of the trips | |
| `analytics-chart-j` | فروخت کی رقم اور نتیجہ | Sales money and result | |
| `analytics-view-i-title` | باغات | Gardens | |
| `analytics-view-i-help` | ہر باغ کا: ادائیگی، ہماری دکان اور دوسری دکانوں میں فروخت، کمیشن .gender = masculine، نتیجہ اور ہر شراکت دار کا حصہ۔ | Each garden: what was paid, the sales in our shop and in other shops, the Commission, the result and the share of each Partner. | |
| `analytics-col-garden` | باغ | Garden | |
| `analytics-col-garden-status` | حالت | Status | |
| `analytics-col-paid-shop` | دکان سے ادا | Paid From the Shop | |
| `analytics-col-paid-pocket` | جیب سے ادا | Paid From Pockets | |
| `analytics-col-sales-our-shop` | ہماری دکان میں فروخت | Sales in Our Shop | |
| `analytics-col-sales-other-shops` | دوسری دکانوں میں فروخت | Sales in Other Shops | |
| `analytics-col-other-in` | دیگر آمد | Other Money In | |
| `analytics-col-garden-result` | نتیجہ | Result | |
| `analytics-col-partner-result` | نتیجہ | Result | |
| `analytics-col-gardens-result` | باغات میں تبدیلی | Gardens Change | |
| `analytics-filter-garden` | باغ | Garden | |
| `analytics-chart-i` | ہر باغ کی فروخت اور نتیجہ | Sales money and result of each garden | |
| `analytics-filter-side-seller` | { analytics-side-seller } | { analytics-side-seller } | |
| `analytics-filter-side-buyer` | { analytics-side-buyer } | { analytics-side-buyer } | |
| `analytics-filter-goods-consigned` | { analytics-goods-consigned } | { analytics-goods-consigned } | |
| `analytics-filter-goods-trip` | { analytics-goods-trip } | { analytics-goods-trip } | |
| `analytics-filter-goods-garden` | { analytics-goods-garden } | { analytics-goods-garden } | |
| `analytics-view-k-title` | پھل | Fruit Taken | |
| `analytics-view-k-help` | ہر شراکت دار (یا دکان کے مہمان) اور قسم مال کا لیا گیا پھل: تعداد اور قیمت، ہماری دکان سے اور دوسری دکانوں سے۔ | The fruit taken for each partner (or the shop's Guests) and Item: the quantity and the value, from our shop and from other shops. | |
| `analytics-chart-k` | ہماری دکان اور دوسری دکانوں سے لیا گیا پھل | Fruit taken from our shop and from other shops | |

## assist

| ID | Urdu | English | Result |
|---|---|---|---|
| `assist-card-loading` | کارڈ کھل رہا ہے… | Opening the card… | |
| `assist-card-failed` | کارڈ نہیں کھل سکا | The card could not be opened | |
| `assist-card-balance` | بقایا | Balance | |
| `assist-card-last-entry` | آخری اندراج | Last entry | |
| `assist-card-never` | ابھی کوئی اندراج نہیں | No entry yet | |
| `assist-card-credit-limit` | ادھار کی حد | Credit limit | |
| `assist-card-credit-used` | استعمال شدہ | Used | |
| `assist-card-advance` | پیشگی کھاتہ .gender = masculine باقی | Advance open | |
| `assist-warning-inactive` | یہ کھاتہ غیر فعال ہے | This account is inactive | |
| `assist-warning-over-credit-limit` | ادھار کی حد پوری ہو چکی ہے | The credit limit is used up | |
| `assist-warning-owes-firm` | اس پارٹی کے ذمے فرم کی رقم ہے | This party owes the firm | |
| `accountpicker-unknown-seller` | اس کوڈ کا کوئی بیوپاری .gender = masculine نہیں | There is no Seller with this code | |
| `accountpicker-unknown-buyer` | اس کوڈ کا کوئی گاہک .gender = masculine نہیں | There is no Buyer with this code | |
| `accountpicker-unknown-any` | اس کوڈ کا کوئی کھاتہ نہیں | There is no account with this code | |
| `accountpicker-reserved` | یہ کوڈ خاص کھاتوں کے لیے ہے۔ عام کھاتے کا یہ کوڈ نہیں ہو سکتا۔ | This code is kept for system accounts. An ordinary account cannot have it. | |
| `accountpicker-inactive` | { $name } ({ $code }) غیر فعال ہے۔ نئے اندراج میں اسے استعمال نہیں کیا جا سکتا۔ | { $name } ({ $code }) is inactive. It cannot be used in a new entry. | |
| `accountpicker-several` | اس کوڈ کے ایک سے زیادہ کھاتے ہیں۔ ایک چنیں۔ | More than one account has this code. Pick one. | |
| `accountpicker-add` | نیا کھاتہ شامل کریں | Add a new account | |
| `accountpicker-add-title` | نیا کھاتہ | New account | |
| `accountpicker-add-help` | صرف وہ باتیں لکھیں جن سے یہ کھاتہ پہچانا جائے۔ باقی تفصیل بعد میں انڈکس سے بھر سکتے ہیں۔ | Write only what tells this account apart. You can fill in the rest later from the Index. | |
| `calc-info-label` | یہ کیسے نکالا گیا؟ | How was this worked out? | |
| `calc-info-title` | حساب | Working | |
| `calc-info-unknown` | اس رقم کا حساب یہاں نہیں دکھایا جا سکتا۔ | The working of this amount cannot be shown here. | |
| `calc-info-rule` | (اصول کا درجہ: { $level }) | (Rule level: { $level }) | |
| `working-percent` | <num>{ $base }</num> × <num>{ $bps }</num> ÷ 100 = <num>{ $result }</num> | <num>{ $base }</num> × <num>{ $bps }</num> ÷ 100 = <num>{ $result }</num> | |
| `working-per-unit` | <num>{ $qty }</num> × <num>{ $value }</num> = <num>{ $result }</num> | <num>{ $qty }</num> × <num>{ $value }</num> = <num>{ $result }</num> | |
| `working-per-unit-received` | <num>{ $qty_received }</num> نگ وصول × <num>{ $value }</num> = <num>{ $result }</num> | <num>{ $qty_received }</num> Nag received × <num>{ $value }</num> = <num>{ $result }</num> | |
| `working-per-lot` | <num>{ $value }</num> فی لاٹ .gender = masculine = <num>{ $result }</num> | <num>{ $value }</num> per Lot = <num>{ $result }</num> | |
| `working-manual` | ہاتھ سے لکھی گئی رقم: <num>{ $result }</num> | Amount typed by hand: <num>{ $result }</num> | |
| `working-none` | کوئی اصول نہیں لگا۔ رقم لاٹ .gender = masculine پر لکھی جاتی ہے۔ | No rule applies. The amount is typed on the Lot. | |
| `working-manual-percent` | ہاتھ سے لکھا گیا: <num>{ $base }</num> × <num>{ $bps }</num> ÷ 100 = <num>{ $result }</num> | Typed by hand: <num>{ $base }</num> × <num>{ $bps }</num> ÷ 100 = <num>{ $result }</num> | |
| `working-manual-per-unit` | ہاتھ سے لکھا گیا: <num>{ $qty }</num> × <num>{ $value }</num> = <num>{ $result }</num> | Typed by hand: <num>{ $qty }</num> × <num>{ $value }</num> = <num>{ $result }</num> | |
| `working-manual-per-unit-received` | ہاتھ سے لکھا گیا: <num>{ $qty_received }</num> نگ وصول × <num>{ $value }</num> = <num>{ $result }</num> | Typed by hand: <num>{ $qty_received }</num> Nag received × <num>{ $value }</num> = <num>{ $result }</num> | |
| `working-manual-per-lot` | ہاتھ سے لکھا گیا: <num>{ $value }</num> فی لاٹ .gender = masculine = <num>{ $result }</num> | Typed by hand: <num>{ $value }</num> per Lot = <num>{ $result }</num> | |
| `working-per-lot-already-charged` | لاٹ .gender = masculine کا یہ چارج پہلی بکری .gender = feminine میں لگ چکا ہے، اس لیے اب کچھ نہیں لگا۔ | This charge is made once per Lot and was made in the first Sale, so nothing is added now. | |
| `working-line-value` | <num>{ $qty }</num> × <num>{ $rate }</num> = <num>{ $result }</num> | <num>{ $qty }</num> × <num>{ $rate }</num> = <num>{ $result }</num> | |
| `working-weight-value` | <num>{ $grams }</num> گرام × <num>{ $rate }</num> فی کلو ÷ <num>1,000</num> = <num>{ $result }</num> (قریب ترین روپے تک) | <num>{ $grams }</num> g × <num>{ $rate }</num> per kg ÷ <num>1,000</num> = <num>{ $result }</num> (to the nearest rupee) | |
| `working-buyer-total` | قیمت <num>{ $value }</num> + کمیشن .gender = masculine <num>{ $comm }</num> + لاگا .gender = masculine <num>{ $laga }</num> = <num>{ $result }</num> | Value <num>{ $value }</num> + Commission <num>{ $comm }</num> + Laga <num>{ $laga }</num> = <num>{ $result }</num> | |
| `working-rate-diff` | گاہک .gender = masculine کی قیمت <num>{ $buyer_value }</num> − بیوپاری .gender = masculine کی رقم <num>{ $seller_amount }</num> = <num>{ $result }</num> | Buyer value <num>{ $buyer_value }</num> − seller amount <num>{ $seller_amount }</num> = <num>{ $result }</num> | |
| `working-sum` | میزان ({ $count } لائن) = <num>{ $result }</num> | Total ({ $count } line(s)) = <num>{ $result }</num> | |
| `working-deductions` | <num>{ $freight }</num> + <num>{ $comm }</num> + <num>{ $labour }</num> + <num>{ $munshiana }</num> + <num>{ $cash }</num> + <num>{ $store_rent }</num> + <num>{ $phone }</num> = <num>{ $result }</num> | <num>{ $freight }</num> + <num>{ $comm }</num> + <num>{ $labour }</num> + <num>{ $munshiana }</num> + <num>{ $cash }</num> + <num>{ $store_rent }</num> + <num>{ $phone }</num> = <num>{ $result }</num> | |
| `working-net` | <num>{ $gross }</num> − <num>{ $deductions }</num> = <num>{ $result }</num> | <num>{ $gross }</num> − <num>{ $deductions }</num> = <num>{ $result }</num> | |
| `working-seller-credit` | <num>{ $gross }</num> − <num>{ $comm }</num> − <num>{ $freight }</num> − <num>{ $labour }</num> − <num>{ $munshiana }</num> − <num>{ $store_rent }</num> − <num>{ $phone }</num> = <num>{ $result }</num> | <num>{ $gross }</num> − <num>{ $comm }</num> − <num>{ $freight }</num> − <num>{ $labour }</num> − <num>{ $munshiana }</num> − <num>{ $store_rent }</num> − <num>{ $phone }</num> = <num>{ $result }</num> | |
| `working-shared-income` | <num>{ $seller_comm }</num> + <num>{ $buyer_comm }</num> + <num>{ $laga }</num> + <num>{ $phone }</num> + <num>{ $rate_diff }</num> = <num>{ $result }</num> | <num>{ $seller_comm }</num> + <num>{ $buyer_comm }</num> + <num>{ $laga }</num> + <num>{ $phone }</num> + <num>{ $rate_diff }</num> = <num>{ $result }</num> | |
| `working-partner-share` | حصہ .gender = masculine <num>{ $bps }</num> فیصد از <num>{ $base }</num> = <num>{ $result }</num> | Share <num>{ $bps }</num>% of <num>{ $base }</num> = <num>{ $result }</num> | |
| `working-expense-share` | دکان خرچہ .gender = masculine <num>{ $amount }</num>، حصہ .gender = masculine <num>{ $bps }</num> فیصد (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے بڑے حصہ .gender = masculine کو): <num>{ $result }</num> | Shop Expense <num>{ $amount }</num>, Share <num>{ $bps }</num>% (each part is rounded down and the rupees left over go to the largest Share): <num>{ $result }</num> | |
| `working-partner-total` | کل رقم پہلے <num>{ $before }</num>، یہ اندراج <num>{ $amount }</num>، بعد میں <num>{ $after }</num> | Partner Total before <num>{ $before }</num>, this entry <num>{ $amount }</num>, after <num>{ $after }</num> | |
| `working-general-share` | اس اندراج کی آمدن اور دکان خرچہ .gender = masculine کی لائنوں کا خالص، حصہ .gender = masculine <num>{ $bps }</num> فیصد (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے بڑے حصہ .gender = masculine کو): <num>{ $result }</num> | Net of the income and Shop Expense lines of this entry, Share <num>{ $bps }</num>% (each part is rounded down and the rupees left over go to the largest Share): <num>{ $result }</num> | |
| `working-none-line` | اس لائن پر کوئی اصول نہیں لگا، اس لیے رقم <num>{ $result }</num> ہے۔ | No rule applies to this line, so the amount is <num>{ $result }</num>. | |
| `working-cash-pukhta` | پختہ کھاتہ .gender = masculine کے بیوپاری کو اس کی پوری صافی بکری .gender = feminine اسی وقت دی جاتی ہے: <num>{ $result }</num> | A Pukhta Khata seller is paid his whole Net Sale on the spot: <num>{ $result }</num>. | |
| `working-cost-of-goods` | باقی لاگت <num>{ $cost_left }</num>، <num>{ $qty_left }</num> نگ میں سے <num>{ $qty }</num> نگ کا حصہ (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے بڑے حصے کو): <num>{ $result }</num> | Cost left <num>{ $cost_left }</num>, the part for <num>{ $qty }</num> of <num>{ $qty_left }</num> Nag (each part is rounded down and the rupees left over go to the larger part): <num>{ $result }</num> | |
| `working-cost-of-goods-last` | آخری <num>{ $qty }</num> نگ کے ساتھ باقی ساری لاگت جاتی ہے: <num>{ $result }</num> | The last <num>{ $qty }</num> Nag take all the cost left: <num>{ $result }</num> | |
| `working-cost-of-goods-kept` | یہ لائن جیسی تھی ویسی رہی، اس لیے اس کی پرانی لاگت وہی رہتی ہے: <num>{ $result }</num> | This line was left as it was, so it keeps the cost it had: <num>{ $result }</num> | |
| `working-venture-part` | ادائیگی <num>{ $amount }</num>، اس کا <num>{ $bps }</num> ÷ 10,000 (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے بڑے حصہ .gender = masculine کو): <num>{ $result }</num> | Payment <num>{ $amount }</num>, <num>{ $bps }</num> ÷ 10,000 of it (each part is rounded down and the rupees left over go to the largest Share): <num>{ $result }</num> | |
| `working-venture-credit` | آنے والی رقم <num>{ $amount }</num>، اس کا <num>{ $bps }</num> ÷ 10,000 (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے بڑے حصہ .gender = masculine کو): <num>{ $result }</num> | Money in <num>{ $amount }</num>, <num>{ $bps }</num> ÷ 10,000 of it (each part is rounded down and the rupees left over go to the largest Share): <num>{ $result }</num> | |
| `working-fruit-value` | <num>{ $qty }</num> × <num>{ $rate }</num> = <num>{ $result }</num> | <num>{ $qty }</num> × <num>{ $rate }</num> = <num>{ $result }</num> | |
| `working-fruit-value-weight` | <num>{ $grams }</num> گرام × <num>{ $rate }</num> فی کلو ÷ <num>1,000</num> = <num>{ $result }</num> (قریب ترین روپے تک) | <num>{ $grams }</num> g × <num>{ $rate }</num> per kg ÷ <num>1,000</num> = <num>{ $result }</num> (to the nearest rupee) | |
| `working-fruit-guests` | قیمت <num>{ $value }</num> کو <num>{ $count }</num> شراکت دار میں برابر بانٹا (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے سب سے پہلے بنائے گئے شراکت دار کو): پہلا حصہ <num>{ $result }</num> | Value <num>{ $value }</num> shared equally between <num>{ $count }</num> partners (each part is rounded down and the rupees left over go to the partner made first): the first part <num>{ $result }</num> | |
| `working-trip-cost-spread` | خرچ <num>{ $cost }</num>، <num>{ $total_qty }</num> نگ میں سے <num>{ $qty }</num> نگ کا حصہ (ہر حصہ نیچے کے پورے روپے تک، اور بچے ہوئے روپے بڑے حصے کو): <num>{ $result }</num> | Cost <num>{ $cost }</num>, the part for <num>{ $qty }</num> of <num>{ $total_qty }</num> Nag (each part is rounded down and the rupees left over go to the larger part): <num>{ $result }</num> | |
| `working-trip-result` | فروخت کی رقم <num>{ $sales }</num> + دوسری دکانوں کی رقم <num>{ $other_shop }</num> − بکے ہوئے مال کی لاگت <num>{ $cost_sold }</num> − ضائع مال کی لاگت <num>{ $cost_written_off }</num> = <num>{ $result }</num> | Sales money <num>{ $sales }</num> + money from other shops <num>{ $other_shop }</num> − cost of goods sold <num>{ $cost_sold }</num> − cost written off <num>{ $cost_written_off }</num> = <num>{ $result }</num> | |
| `working-trip-partner-net` | واپس ملا <num>{ $back }</num> − دکان سے لگایا <num>{ $in_shop }</num> − اپنی جیب سے لگایا <num>{ $in_pocket }</num> = <num>{ $result }</num> | Money back <num>{ $back }</num> − paid from the shop <num>{ $in_shop }</num> − paid from his pocket <num>{ $in_pocket }</num> = <num>{ $result }</num> | |
| `working-garden-balance` | آنے والی رقم <num>{ $money_in }</num> − ادا کی گئی ہر رقم (قسطیں اور باغ کے خرچ، دکان سے اور اپنی جیب سے) <num>{ $paid }</num> = <num>{ $result }</num> | Money in <num>{ $money_in }</num> − everything paid (installments and costs, from the shop and from pockets) <num>{ $paid }</num> = <num>{ $result }</num> | |
| `working-purchase-result` | فروخت کی رقم <num>{ $sales }</num> + دوسری دکانوں کی رقم <num>{ $other_shop }</num> − بکے ہوئے مال کی لاگت <num>{ $cost_sold }</num> − ضائع مال کی لاگت <num>{ $cost_written_off }</num> = <num>{ $result }</num> | Sales money <num>{ $sales }</num> + money from other shops <num>{ $other_shop }</num> − cost of goods sold <num>{ $cost_sold }</num> − cost written off <num>{ $cost_written_off }</num> = <num>{ $result }</num> | |
| `working-other-sale-result` | ملی ہوئی رقم <num>{ $amount }</num> − بکے ہوئے مال کی لاگت <num>{ $cost }</num> = <num>{ $result }</num> | Money received <num>{ $amount }</num> − cost of goods sold <num>{ $cost }</num> = <num>{ $result }</num> | |
| `working-own-lot-cost` | اس لاٹ کے بکے ہوئے <num>{ $qty }</num> نگ کی لاگت، جو خریداری سے لی گئی: <num>{ $result }</num> | Cost of the <num>{ $qty }</num> Nag sold in this lot, taken from the purchase: <num>{ $result }</num> | |
| `working-own-lot-result` | بیوپاری کی جمع <num>{ $credit }</num> − بکے ہوئے مال کی لاگت <num>{ $cost }</num> = <num>{ $result }</num> | Seller credit <num>{ $credit }</num> − cost of goods sold <num>{ $cost }</num> = <num>{ $result }</num> | |
| `working-purchase-total-cost` | ادا شدہ <num>{ $paid_now }</num> + مارکیٹ سفر .gender = masculine کے خرچ کا حصہ <num>{ $trip_cost }</num> = <num>{ $result }</num> | Paid now <num>{ $paid_now }</num> + part of the Market Trip costs <num>{ $trip_cost }</num> = <num>{ $result }</num> | |
| `working-purchase-avg-cost` | <num>{ $total }</num> ÷ <num>{ $qty }</num> = <num>{ $result }</num> (قریب ترین روپے تک) | <num>{ $total }</num> ÷ <num>{ $qty }</num> = <num>{ $result }</num> (to the nearest rupee) | |
| `working-purchase-paid-now` | <num>{ $value }</num> + <num>{ $freight }</num> + <num>{ $labour }</num> + <num>{ $other_cost }</num> = <num>{ $result }</num> | <num>{ $value }</num> + <num>{ $freight }</num> + <num>{ $labour }</num> + <num>{ $other_cost }</num> = <num>{ $result }</num> | |
| `attention-bell-label` | توجہ طلب: { $count } | Needs attention: { $count } | |
| `attention-title` | توجہ طلب | Needs your attention | |
| `attention-empty-title` | کسی کام پر توجہ کی ضرورت نہیں | Nothing needs your attention | |
| `attention-empty-help` | جب کوئی کام باقی ہو گا تو یہاں نظر آئے گا۔ | When something needs doing, it shows here. | |
| `attention-load-failed` | فہرست کھل نہیں سکی | The list could not be opened | |
| `attention-update` | نیا ورژن <num>{ $version }</num> تیار ہے۔ دن ختم کر کے ترتیبات سے لگائیں۔ | A new version <num>{ $version }</num> is ready. Install it from Settings when the day is done. | |
| `attention-open-first` | سب سے ضروری: <name>{ $name }</name> | Most urgent: <name>{ $name }</name> | |
| `attention-drafts` | { $count -> [one] <num>{ $count }</num> غیر محفوظ مسودہ *[other] <num>{ $count }</num> غیر محفوظ مسودے | { $count -> [one] <num>{ $count }</num> unsaved draft *[other] <num>{ $count }</num> unsaved drafts | |
| `attention-open-days` | { $count -> [one] <num>{ $count }</num> پچھلا دن بند نہیں ہوا *[other] <num>{ $count }</num> پچھلے دن بند نہیں ہوئے | { $count -> [one] <num>{ $count }</num> earlier day is not closed *[other] <num>{ $count }</num> earlier days are not closed | |
| `attention-stock-waiting` | باقی مال میں <num>{ $count }</num> لاٹ .gender = masculine انتظار میں ہیں، کل <num>{ $qty }</num> نگ | { $count -> [one] <num>{ $count }</num> Lot is waiting in Unsold Stock, <num>{ $qty }</num> Nag in all *[other] <num>{ $count }</num> Lots are waiting in Unsold Stock, <num>{ $qty }</num> Nag in all | |
| `attention-stock-waiting-aged` | باقی مال میں <num>{ $count }</num> لاٹ .gender = masculine انتظار میں ہیں، کل <num>{ $qty }</num> نگ، سب سے پرانا <num>{ $days }</num> دن سے | { $count -> [one] <num>{ $count }</num> Lot is waiting in Unsold Stock, <num>{ $qty }</num> Nag in all, the oldest for <num>{ $days }</num> { $days -> [one] day *[other] days } *[other] <num>{ $count }</num> Lots are waiting in Unsold Stock, <num>{ $qty }</num> Nag in all, the oldest for <num>{ $days }</num> { $days -> [one] day *[other] days } | |
| `attention-over-credit-limit` | <num>{ $count }</num> گاہک .gender = masculine ادھار کی حد سے آگے ہیں | { $count -> [one] <num>{ $count }</num> Buyer is over the credit limit *[other] <num>{ $count }</num> Buyers are over the credit limit | |
| `attention-dues-old` | <num>{ $count }</num> گاہک .gender = masculine کے پرانے بقایا: <num>{ $amount }</num> | { $count -> [one] <num>{ $count }</num> Buyer has an old due: <num>{ $amount }</num> *[other] <num>{ $count }</num> Buyers have old dues: <num>{ $amount }</num> | |
| `attention-dues-old-aged` | <num>{ $count }</num> گاہک .gender = masculine کے پرانے بقایا: <num>{ $amount }</num>، سب سے پرانا بل <num>{ $days }</num> دن پرانا ہے | { $count -> [one] <num>{ $count }</num> Buyer has an old due: <num>{ $amount }</num>, the bill is <num>{ $days }</num> { $days -> [one] day *[other] days } old *[other] <num>{ $count }</num> Buyers have old dues: <num>{ $amount }</num>, the oldest bill is <num>{ $days }</num> { $days -> [one] day *[other] days } old | |
| `attention-payables-old` | <num>{ $count }</num> بیوپاری .gender = masculine کی ادائیگی باقی ہے: <num>{ $amount }</num> | { $count -> [one] <num>{ $count }</num> Seller has waited for payment: <num>{ $amount }</num> *[other] <num>{ $count }</num> Sellers have waited for payment: <num>{ $amount }</num> | |
| `attention-payables-old-aged` | <num>{ $count }</num> بیوپاری .gender = masculine کی ادائیگی باقی ہے: <num>{ $amount }</num>، سب سے پرانی <num>{ $days }</num> دن سے | { $count -> [one] <num>{ $count }</num> Seller has waited for payment: <num>{ $amount }</num>, for <num>{ $days }</num> { $days -> [one] day *[other] days } *[other] <num>{ $count }</num> Sellers have waited for payment: <num>{ $amount }</num>, the longest for <num>{ $days }</num> { $days -> [one] day *[other] days } | |
| `attention-backup-old` | آخری کلاؤڈ بیک اپ <num>{ $days }</num> دن پرانا ہے | The last cloud backup is <num>{ $days }</num> { $days -> [one] day *[other] days } old | |
| `attention-backup-never` | ابھی تک کلاؤڈ بیک اپ نہیں بنا | No cloud backup has been made yet | |
| `attention-loans-overdue` | <num>{ $count }</num> ادھار رقم .gender = feminine کی واپسی کی تاریخ گزر چکی ہے: <num>{ $amount }</num> | { $count -> [one] <num>{ $count }</num> Money Lent entry is past its return-by date: <num>{ $amount }</num> *[other] <num>{ $count }</num> Money Lent entries are past their return-by date: <num>{ $amount }</num> | |
| `attention-loans-overdue-aged` | <num>{ $count }</num> ادھار رقم .gender = feminine کی واپسی کی تاریخ گزر چکی ہے، زیادہ سے زیادہ <num>{ $days }</num> دن: <num>{ $amount }</num> | { $count -> [one] <num>{ $count }</num> Money Lent entry is <num>{ $days }</num> { $days -> [one] day *[other] days } past its return-by date: <num>{ $amount }</num> *[other] <num>{ $count }</num> Money Lent entries are past their return-by date, up to <num>{ $days }</num> { $days -> [one] day *[other] days }: <num>{ $amount }</num> | |
| `attention-takes-overdue` | شراکت داروں کی لی ہوئی <num>{ $count }</num> رقموں کی واپسی کی تاریخ گزر چکی ہے: <num>{ $amount }</num> | { $count -> [one] <num>{ $count }</num> amount taken by a partner is past its return-by date: <num>{ $amount }</num> *[other] <num>{ $count }</num> amounts taken by partners are past their return-by date: <num>{ $amount }</num> | |
| `attention-takes-overdue-aged` | شراکت داروں کی لی ہوئی <num>{ $count }</num> رقموں کی واپسی کی تاریخ گزر چکی ہے، زیادہ سے زیادہ <num>{ $days }</num> دن: <num>{ $amount }</num> | { $count -> [one] <num>{ $count }</num> amount taken by a partner is <num>{ $days }</num> { $days -> [one] day *[other] days } past its return-by date: <num>{ $amount }</num> *[other] <num>{ $count }</num> amounts taken by partners are past their return-by date, up to <num>{ $days }</num> { $days -> [one] day *[other] days }: <num>{ $amount }</num> | |
| `attention-names-missing` | <num>{ $count }</num> ریکارڈ میں دو ناموں میں سے ایک ہی نام ہے | { $count -> [one] <num>{ $count }</num> record has only one of its two names *[other] <num>{ $count }</num> records have only one of their two names | |

## audit

| ID | Urdu | English | Result |
|---|---|---|---|
| `audit-title` | آڈٹ لاگ | Audit log | |
| `audit-help` | ہر تبدیلی کا ریکارڈ: کس نے کی، کب کی، کیا تھی اور پرانی اور نئی قیمتیں۔ یہاں اسے صرف پڑھا جا سکتا ہے۔ اسے بدلا یا مٹایا نہیں جا سکتا۔ | A record of every change: who did it, when, what it was and the old and new values. It can only be read here. It cannot be edited or deleted. | |
| `audit-filters-label` | آڈٹ کے فلٹر | Audit filters | |
| `audit-filter-user` | کس نے | Who | |
| `audit-filter-action` | کام | Action | |
| `audit-filter-entity` | حصہ | Area | |
| `audit-filter-record` | ریکارڈ نمبر | Record number | |
| `audit-filter-record-hint` | حصے کے ساتھ | With an area | |
| `audit-filter-text` | الفاظ | Words | |
| `audit-filter-text-hint` | وجہ اور پرانی اور نئی قیمتوں میں ڈھونڈے جاتے ہیں | Looked for in the reason and in the old and new values | |
| `audit-filter-any` | کوئی بھی | Any | |
| `audit-grid-label` | آڈٹ کے اندراجات | Audit entries | |
| `audit-col-when` | کب | When | |
| `audit-col-user` | کس نے | Who | |
| `audit-col-action` | کام | Action | |
| `audit-col-entity` | حصہ | Area | |
| `audit-col-record` | ریکارڈ | Record | |
| `audit-col-reason` | وجہ | Reason | |
| `audit-col-fields` | بدلے ہوئے خانے | Fields changed | |
| `audit-system-user` | ایپ خود | The app itself | |
| `audit-open-hint` | اندراج چننے کے لیے تیر کی کیز اور کیا بدلا دیکھنے کے لیے Enter دبائیں۔ | Use the arrow keys to pick an entry and Enter to see what changed. | |
| `audit-count` | { $total } میں سے { $shown } اندراجات دکھائے گئے ہیں | Showing { $shown } of { $total } entries | |
| `audit-more` | مزید دکھائیں | Show more | |
| `audit-empty` | ان فلٹروں میں کوئی اندراج نہیں۔ | No entry matches these filters. | |
| `audit-empty-help` | تاریخیں بدل کر یا فلٹر کم کر کے دیکھیں۔ | Change the dates or take a filter away. | |
| `audit-load-failed` | آڈٹ لاگ نہیں کھل سکا | The audit log could not be opened | |
| `audit-options-failed` | فلٹروں کی فہرستیں نہیں پڑھی جا سکیں۔ آپ پھر بھی الفاظ اور تاریخیں لکھ سکتے ہیں۔ | The lists of the filters could not be read. You can still type the words and dates. | |
| `audit-detail-title` | اندراج { $number } | Entry { $number } | |
| `audit-detail-back` | فہرست پر واپس | Back to the list | |
| `audit-detail-field` | خانہ | Field | |
| `audit-detail-old` | پہلے | Before | |
| `audit-detail-new` | بعد میں | After | |
| `audit-detail-grid-label` | بدلے ہوئے خانے | Fields that changed | |
| `audit-detail-none` | اس اندراج کا کوئی الگ خانہ درج نہیں۔ | No single field is listed for this entry. | |
| `audit-detail-truncated` | یہاں دکھائے گئے خانوں سے زیادہ بدلے تھے۔ پہلے { $shown } درج ہیں۔ | More fields changed than are shown here. The first { $shown } are listed. | |
| `audit-value-none` | درج نہیں | Not set | |
| `audit-value-yes` | ہاں | Yes | |
| `audit-value-no` | نہیں | No | |
| `audit-value-hidden` | بدلا گیا (دکھایا نہیں جاتا) | Changed (not shown) | |
| `audit-action-create` | بنایا گیا | Created | |
| `audit-action-edit` | بدلا گیا | Changed | |
| `audit-action-cancel` | منسوخ کیا گیا | Cancelled | |
| `audit-action-login` | لاگ ان | Signed in | |
| `audit-action-login-failed` | ناکام لاگ ان | Failed sign-in | |
| `audit-action-logout` | لاگ آؤٹ | Signed out | |
| `audit-entity-settings` | ترتیبات | Settings | |
| `audit-entity-backup-settings` | بیک اپ کی ترتیبات | Backup settings | |
| `audit-entity-day-settings` | دن کی ترتیبات | Day settings | |
| `audit-entity-user` | صارف | User | |
| `audit-entity-account` | کھاتہ | Account | |
| `audit-entity-bikri` | بکری .gender = feminine | Sale | |
| `audit-entity-voucher` | واؤچر | Voucher | |
| `audit-entity-rule` | کمیشن اصول | Commission rule | |
| `audit-entity-partner` | شراکت دار | Partner | |
| `audit-action-day-close` | دن بند کیا | Day closed | |
| `audit-action-day-auto-close` | دن خود بخود بند ہوا | Day closed automatically | |
| `audit-action-day-reopen` | دن کھولا | Day reopened | |
| `audit-action-month-close` | مہینہ بند کیا | Month closed | |
| `audit-action-month-close-reverse` | مہینے کی بندش واپس کی | Month close reversed | |
| `audit-entity-day` | دن | Day | |
| `audit-entity-month` | مہینہ | Month | |
| `audit-action-unlock` | لاک کھولا گیا | Unlocked | |
| `audit-action-lock` | لاک کیا گیا | Locked | |
| `audit-action-recover` | پاس ورڈ نیا بنایا گیا | Password reset | |
| `audit-action-change-password` | پاس ورڈ بدلا گیا | Password changed | |
| `audit-action-make-regular` | مستقل کھاتہ بنایا گیا | Made a regular account | |
| `audit-action-update` | تبدیل کیا گیا | Updated | |
| `audit-action-learn` | سیکھا گیا | Learned | |
| `audit-action-remove` | ہٹایا گیا | Removed | |
| `audit-action-set-shares` | حصہ .gender = masculine بدلا گیا | Share changed | |
| `audit-action-later-sale` | بعد کی بکری .gender = feminine | Later Sale | |
| `audit-action-print` | پرنٹ کیا گیا | Printed | |
| `audit-action-write-off` | ضائع کیا گیا | Written off | |
| `audit-action-write-off-undo` | ضائع واپس لیا گیا | Write-off undone | |
| `audit-action-confirm` | تصدیق کی گئی | Confirmed | |
| `audit-action-check-failed` | جانچ ناکام ہوئی | Check failed | |
| `audit-action-restore` | بحال کیا گیا | Restored | |
| `audit-entity-backup-cloud-keys` | کلاؤڈ بیک اپ کی کنجیاں | Cloud backup keys | |
| `audit-entity-backup-key` | بیک اپ کی | Backup key | |
| `audit-entity-backup` | بیک اپ | Backup | |
| `audit-entity-health` | حساب کی جانچ | Books check | |
| `audit-entity-staff` | عملہ | Staff | |
| `audit-entity-city` | شہر | City | |
| `audit-entity-item` | قسم مال | Item | |
| `audit-entity-packing` | پیکنگ .gender = feminine | Packing | |
| `audit-entity-adda` | اڈا .gender = masculine | Adda | |
| `audit-entity-marka` | مارکہ .gender = masculine | Mark | |
| `audit-entity-store` | سٹور | Store | |
| `audit-entity-narration` | تفصیل | Narration | |
| `audit-entity-expense-head` | خرچہ مد | Expense head | |
| `audit-entity-bank-account` | بینک اکاؤنٹ | Bank account | |
| `audit-entity-charge-rule` | کمیشن اصول | Commission rule | |
| `audit-entity-translit-pair` | سیکھا ہوا لفظ | Learned word | |
| `audit-entity-bikri-lot` | لاٹ .gender = masculine | Lot | |
| `audit-entity-trip` | مارکیٹ سفر .gender = masculine | Market Trip | |
| `audit-entity-purchase` | خریداری | Purchase | |
| `audit-entity-trip-cost` | سفر خرچ | Trip cost | |
| `audit-entity-trip-other-sale` | دوسری دکان میں فروخت | Sale in another shop | |

## auth

| ID | Urdu | English | Result |
|---|---|---|---|
| `auth-login-title` | داخل ہوں | Sign in | |
| `auth-login-username` | صارف کا نام | User name | |
| `auth-login-password` | پاس ورڈ | Password | |
| `auth-login-submit` | داخل ہوں | Sign in | |
| `auth-login-exit` | باہر | Exit | |
| `auth-login-forgot` | پاس ورڈ بھول گئے؟ ریکوری کوڈ سے کھولیں | Forgot the password? Open with the recovery code | |
| `auth-recover-title` | ریکوری کوڈ سے پاس ورڈ بدلیں | Reset the password with the recovery code | |
| `auth-recover-help` | اس طرح صرف مالک کا پاس ورڈ بدلا جا سکتا ہے۔ | Only the Owner's password can be reset this way. | |
| `auth-recover-code` | ریکوری کوڈ | Recovery code | |
| `auth-recover-code-hint` | کوڈ پرنٹ کی ہوئی پرچی سے دیکھ کر لکھیں۔ | Copy it from the printed slip. | |
| `auth-recover-new-password` | نیا پاس ورڈ | New password | |
| `auth-recover-new-password-repeat` | نیا پاس ورڈ دوبارہ | New password again | |
| `auth-recover-submit` | پاس ورڈ بدلیں | Change password | |
| `auth-recover-done-title` | پاس ورڈ بدل گیا | The password is changed | |
| `auth-recover-done-help` | پرانا ریکوری کوڈ ختم ہو گیا۔ یہ نیا کوڈ پرنٹ کر کے محفوظ رکھیں۔ | The old recovery code is used up. Print this new one and keep it safe. | |
| `auth-recover-done-continue` | داخل ہونے کے صفحے پر جائیں | Go to sign in | |
| `auth-wizard-title` | پہلی بار ترتیب | First-time setup | |
| `auth-wizard-steps` | ترتیب کے مراحل | Setup steps | |
| `auth-wizard-step-firm` | فرم کی تفصیل | Firm details | |
| `auth-wizard-step-owner` | مالک کا پاس ورڈ | Owner password | |
| `auth-wizard-step-code` | ریکوری کوڈ | Recovery code | |
| `auth-wizard-step-partners` | شراکت دار | Partners | |
| `auth-wizard-partners-title` | شراکت دار شامل کریں | Add the partners | |
| `auth-wizard-partners-help` | ہر شراکت دار کا نام اور حصہ .gender = masculine لکھیں۔ یہ کام بعد میں بھی ہو سکتا ہے، مگر جب تک حصہ .gender = masculine پورے 100 فیصد نہ ہوں، بکری .gender = feminine محفوظ نہیں ہو گی۔ | Add each partner's name and share. You can also do this later, but until the shares total exactly 100%, a Sale cannot be saved. | |
| `auth-wizard-partners-save` | محفوظ کر کے مکمل کریں | Save and finish | |
| `auth-wizard-partners-later` | بعد میں کریں | Do this later | |
| `auth-wizard-firm-title` | فرم کے بارے میں بتائیں | Tell us about the firm | |
| `auth-wizard-firm-name-ur` | فرم کا نام (اردو) | Firm name (Urdu) | |
| `auth-wizard-firm-name-en` | فرم کا نام (انگریزی) | Firm name (English) | |
| `auth-wizard-mandi-ur` | منڈی (اردو) | Mandi (Urdu) | |
| `auth-wizard-mandi-en` | منڈی (انگریزی) | Mandi (English) | |
| `auth-wizard-phone` | فون | Phone | |
| `auth-wizard-address` | پتہ | Address | |
| `auth-wizard-address-hint` | دکان نمبر اور گلی لکھیں۔ | Shop number and street. | |
| `auth-wizard-later` | یہ سب بعد میں ترتیبات میں بدل سکتے ہیں۔ | You can change these later in Settings. | |
| `auth-wizard-next` | آگے | Next | |
| `auth-wizard-owner-title` | مالک کا کھاتہ بنائیں | Create the Owner's account | |
| `auth-wizard-owner-name` | مالک کا نام | Owner's name | |
| `auth-wizard-password` | پاس ورڈ | Password | |
| `auth-wizard-password-repeat` | دوبارہ پاس ورڈ | Password again | |
| `auth-wizard-password-hint` | کم از کم { $min } حروف۔ | At least { $min } characters. | |
| `auth-wizard-create` | بنائیں | Create | |
| `auth-wizard-code-title` | ریکوری کوڈ پرنٹ کر کے محفوظ رکھیں | Print the recovery code and keep it safe | |
| `auth-wizard-code-help` | اگر مالک کا پاس ورڈ بھول جائے تو یہی کوڈ دوبارہ داخل ہونے کا واحد راستہ ہے۔ یہ کوڈ دوبارہ نہیں دکھایا جائے گا۔ | If the Owner forgets the password, this code is the only way back in. It will not be shown again. | |
| `auth-wizard-printed` | میں نے کوڈ پرنٹ کر لیا ہے اور محفوظ جگہ رکھ دیا ہے | I have printed the code and put it in a safe place | |
| `auth-wizard-print` | پرنٹ کریں | Print | |
| `auth-wizard-finish` | مکمل کریں | Finish | |
| `auth-lock-title` | ایپ لاک ہے | The app is locked | |
| `auth-lock-help` | اپنا پاس ورڈ لکھیں۔ آپ کا کام جوں کا توں رہے گا۔ | Type your password. Your work stays as it was. | |
| `auth-lock-user` | صارف: <name>{ $name }</name> | User: <name>{ $name }</name> | |
| `auth-lock-unlock` | کھولیں | Unlock | |
| `auth-change-title` | پاس ورڈ بدلیں | Change password | |
| `auth-change-help` | جو پاس ورڈ ابھی استعمال کرتے ہیں وہ لکھیں، پھر نیا پاس ورڈ دو بار لکھیں۔ | Type the password you use now, then the new password twice. | |
| `auth-change-current` | موجودہ پاس ورڈ | Current password | |
| `auth-change-new` | نیا پاس ورڈ | New password | |
| `auth-change-repeat` | نیا پاس ورڈ دوبارہ | New password again | |
| `auth-change-wrong-current` | موجودہ پاس ورڈ غلط ہے۔ باقی کوششیں: { $tries_left } | The current password is wrong. Tries left: { $tries_left } | |
| `auth-change-saved` | پاس ورڈ بدل دیا گیا۔ | Password changed | |

## backup

| ID | Urdu | English | Result |
|---|---|---|---|
| `backup-title` | بیک اپ | Backup | |
| `backup-help` | آپ کے ڈیٹا کی کاپیاں خود بخود بنتی رہتی ہیں۔ یہاں آپ انہیں دیکھ سکتے ہیں، ابھی کاپی بنا سکتے ہیں اور اپنا ڈیٹا واپس لا سکتے ہیں۔ | Copies of your data are made by themselves. Here you can see them, make one now, and get your data back. | |
| `backup-load-failed` | بیک اپ کی حالت کھل نہیں سکی | The state of the backups could not be opened | |
| `backup-now` | بیک اپ ابھی لیں | Back up now | |
| `backup-now-done` | بیک اپ مکمل ہو گیا | Backup finished | |
| `backup-test-now` | بحالی کی جانچ | Test restore | |
| `backup-state-title` | بیک اپ کی حالت | State of the backups | |
| `backup-state-local` | اس کمپیوٹر پر آخری کاپی | Last copy on this computer | |
| `backup-state-usb` | یو ایس بی ڈرائیو پر آخری کاپی | Last copy on the USB drive | |
| `backup-state-cloud` | کلاؤڈ میں آخری کاپی | Last copy in the cloud | |
| `backup-state-queue` | انٹرنیٹ کے انتظار میں کلاؤڈ کاپیاں | Cloud copies waiting for the internet | |
| `backup-state-last-test` | آخری بحالی کی جانچ | Last test restore | |
| `backup-state-never` | ابھی تک نہیں | None yet | |
| `backup-state-off` | بند ہے | Switched off | |
| `backup-state-usb-missing` | یو ایس بی ڈرائیو لگی ہوئی نہیں | The USB drive is not plugged in | |
| `backup-state-cloud-old` | ایک دن سے زیادہ پرانی | Older than one day | |
| `backup-state-cloud-fresh` | تازہ ہے | Up to date | |
| `backup-state-running` | بیک اپ یا بحالی چل رہی ہے۔ یہ صفحہ اسے دیکھتا رہتا ہے۔ | A backup or restore is running. This page follows it. | |
| `backup-state-test-due` | پچھلے 30 دنوں میں بحالی کی کوئی جانچ کامیاب نہیں ہوئی۔ جانچ سے ثابت ہوتا ہے کہ آپ کی کاپیاں واقعی آپ کا ڈیٹا واپس لا سکتی ہیں۔ | No test restore has worked in the last 30 days. A test restore proves that your copies can really bring your data back. | |
| `backup-state-pack-pending` | روزانہ پڑھنے کے قابل پیک (روزنامچہ کی PDF اور اسپریڈ شیٹ) ابھی نہیں بنتا۔ | The daily readable pack (the day book as a PDF, and spreadsheets) is not made yet. | |
| `backup-run-ok` | کاپی بن گئی اور جانچ لی گئی | copy made and checked | |
| `backup-run-queued` | کاپی انٹرنیٹ کے انتظار میں ہے۔ وہ خود بخود بھیج دی جائے گی۔ | copy is waiting for the internet. It is sent by itself. | |
| `backup-run-failed` | کاپی نہیں بن سکی۔ | copy failed. | |
| `backup-kind-local` | اس کمپیوٹر پر | This computer | |
| `backup-kind-usb` | یو ایس بی ڈرائیو | USB drive | |
| `backup-kind-cloud` | کلاؤڈ | Cloud | |
| `backup-list-title` | کاپیاں | Copies | |
| `backup-list-label` | بیک اپ کاپیاں | Backup copies | |
| `backup-filter-label` | ان جگہوں کی کاپیاں دکھائیں | Show copies from | |
| `backup-filter-all` | سب | All | |
| `backup-filter-local` | اس کمپیوٹر پر | This computer | |
| `backup-filter-usb` | یو ایس بی ڈرائیو | USB drive | |
| `backup-filter-cloud` | کلاؤڈ | Cloud | |
| `backup-list-failed` | کاپیوں کی فہرست کھل نہیں سکی | The list of copies could not be opened | |
| `backup-list-empty` | ابھی کوئی کاپی نہیں | No copies yet | |
| `backup-list-empty-help` | پہلی کاپی ابھی بنانے کے لیے بٹن دبائیں۔ | Press the button to make the first copy now. | |
| `backup-list-count` | { $total } میں سے { $shown } کاپیاں دکھائی جا رہی ہیں | Showing { $shown } of { $total } copies | |
| `backup-list-more` | مزید دکھائیں | Show more | |
| `backup-list-how` | کاپی چن کر Enter دبائیں، یا اس پر ڈبل کلک کریں، تو وہ بحال ہو گی۔ | Select a copy and press Enter, or double-click it, to restore it. | |
| `backup-restore-selected` | چنی ہوئی کاپی بحال کریں | Restore selected | |
| `backup-test-selected` | چنی ہوئی کاپی کی جانچ کریں | Test selected | |
| `backup-list-cloud-unreachable` | کلاؤڈ تک رسائی نہیں ہو سکی، اس لیے اس کی کاپیاں اس فہرست میں نہیں ہیں۔ | The cloud could not be reached, so its copies are not in this list. | |
| `backup-col-created` | بننے کا وقت | Made on | |
| `backup-col-place` | جگہ | Place | |
| `backup-col-size` | حجم | Size | |
| `backup-col-status` | جانچ | Check | |
| `backup-col-locked` | چابی سے بند | Locked with key | |
| `backup-col-name` | فائل | File | |
| `backup-status-good` | ٹھیک | Good | |
| `backup-status-bad` | جانچ میں ناکام | Failed its check | |
| `backup-status-unchecked` | ابھی جانچ نہیں ہوئی | Not checked yet | |
| `backup-locked-yes` | ہاں | Yes | |
| `backup-locked-no` | نہیں | No | |
| `backup-size-value` | { $value } { $unit } | { $value } { $unit } | |
| `backup-size-bytes` | بائٹ | bytes | |
| `backup-size-kb` | کے بی | KB | |
| `backup-size-mb` | ایم بی | MB | |
| `backup-size-gb` | جی بی | GB | |
| `backup-restore-title` | یہ بیک اپ بحال کریں | Restore this backup | |
| `backup-restore-help` | آپ کے کھاتے اس کاپی کے کھاتوں سے بدل جائیں گے۔ پہلے دیکھ لیں کہ اس میں کیا ہے۔ | Your books are replaced by the books in this copy. Look at what is in it first. | |
| `backup-restore-key-help` | یہ کاپی بیک اپ کی چابی سے بند ہے اور اس کمپیوٹر پر چابی نہیں ہے۔ پرنٹ کی ہوئی پرچی سے چابی لکھیں۔ | This copy is locked with the backup key, and this computer does not have the key. Type the key from your printed sheet. | |
| `backup-restore-key-label` | بیک اپ کی چابی | Backup key | |
| `backup-restore-key-hint` | جیسی پرنٹ ہوئی ہے ویسی ہی لکھیں، ڈیش سمیت۔ | Copy it exactly as printed, with the dashes. | |
| `backup-restore-key-continue` | آگے بڑھیں | Continue | |
| `backup-restore-preview-failed` | یہ کاپی پڑھی نہیں جا سکی | This copy could not be read | |
| `backup-restore-made-on` | یہ کاپی <num>{ $date }</num> کو بنی تھی۔ | This copy was made on <num>{ $date }</num>. | |
| `backup-restore-totals-label` | اس کاپی کے کھاتے اور اب کے کھاتے | Books in this copy and books now | |
| `backup-restore-lost-none` | کچھ ضائع نہیں ہو گا: اس کاپی کے بعد کا کوئی اندراج نہیں ہے۔ | Nothing will be lost: there is nothing newer than this copy. | |
| `backup-restore-lost` | اس کاپی کے بعد کیے گئے سب اندراج ضائع ہو جائیں گے: <num>{ $journal }</num> پوسٹ شدہ اندراج، <num>{ $bikri }</num> بکری .gender = feminine اندراج اور <num>{ $vouchers }</num> واؤچر۔ | Everything entered after this copy will be lost: <num>{ $journal }</num> posted entries, <num>{ $bikri }</num> Sale entries and <num>{ $vouchers }</num> vouchers. | |
| `backup-restore-upgrade` | یہ کاپی ایپ کے پرانے ورژن کی ہے۔ بحالی کے بعد وہ خود بخود نئی کر دی جائے گی۔ | This copy is from an older version of the app. It is upgraded by itself after the restore. | |
| `backup-restore-safety` | کچھ بدلنے سے پہلے آپ کے موجودہ ڈیٹا کی حفاظتی کاپی بنا کر جانچی جاتی ہے۔ بحالی کے بعد آپ باہر ہو جائیں گے اور دوبارہ داخل ہوں گے۔ | Before anything is replaced, a safety copy of your present data is made and checked. You are signed out after the restore and sign in again. | |
| `backup-restore-button` | بحال کریں | Restore | |
| `backup-restore-confirm-title` | کیا یہ بیک اپ بحال کریں؟ | Restore this backup? | |
| `backup-restore-confirm-description` | { $lost -> [0] پہلے آپ کے موجودہ ڈیٹا کی حفاظتی کاپی بنے گی۔ پھر آپ کے سب کھاتے <num>{ $date }</num> کو بنی کاپی سے بدل جائیں گے۔ آپ باہر ہو جائیں گے۔ *[other] پہلے آپ کے موجودہ ڈیٹا کی حفاظتی کاپی بنے گی۔ پھر آپ کے سب کھاتے <num>{ $date }</num> کو بنی کاپی سے بدل جائیں گے۔ یہ ضائع ہو جائیں گے: <num>{ $journal }</num> پوسٹ شدہ اندراج، <num>{ $bikri }</num> بکری .gender = feminine اندراج اور <num>{ $vouchers }</num> واؤچر۔ آپ باہر ہو جائیں گے۔ } | { $lost -> [0] A safety copy of your present data is made first. Then all your books are replaced by the copy made on <num>{ $date }</num>. You are signed out. *[other] A safety copy of your present data is made first. Then all your books are replaced by the copy made on <num>{ $date }</num>. These will be lost: <num>{ $journal }</num> posted entries, <num>{ $bikri }</num> Sale entries and <num>{ $vouchers }</num> vouchers. You are signed out. } | |
| `backup-restored` | کاپی { $date } سے بحال ہو گئی۔ براہ کرم دوبارہ داخل ہوں۔ | Restored from the copy of { $date }. Please sign in again. | |
| `backup-totals-what` | کھاتے | Books | |
| `backup-totals-in-backup` | اس کاپی میں | In this copy | |
| `backup-totals-now` | اب | Now | |
| `backup-totals-restored` | بحال کی ہوئی ٹیسٹ کاپی | Restored test copy | |
| `backup-totals-live` | موجودہ ڈیٹا | Live data | |
| `backup-totals-accounts` | کھاتوں کی تعداد | Accounts | |
| `backup-totals-bikri` | بکری .gender = feminine اندراج | Sale entries | |
| `backup-totals-vouchers` | واؤچر | Vouchers | |
| `backup-totals-journal` | پوسٹ شدہ اندراج | Posted entries | |
| `backup-totals-last-entry` | تازہ ترین اندراج کی تاریخ | Date of the newest entry | |
| `backup-totals-debit` | کل بنام | Total Debit | |
| `backup-totals-credit` | کل جمع | Total Credit | |
| `backup-totals-cash` | روکڑ بقایا | Cash in hand | |
| `backup-totals-none` | کوئی نہیں | None | |
| `backup-test-title` | بحالی کی جانچ | Test restore | |
| `backup-test-help` | کاپی کو الگ جگہ پر بحال کر کے جانچا جاتا ہے۔ آپ کے اصل ڈیٹا کو ہاتھ نہیں لگتا۔ | The copy is restored into a separate place and checked. Your real data is not touched. | |
| `backup-test-running` | کاپی کو ٹیسٹ جگہ پر بحال کر کے جانچا جا رہا ہے۔ براہ کرم انتظار کریں۔ | Restoring the copy into a test place and checking it. Please wait. | |
| `backup-test-again` | دوبارہ جانچیں | Test again | |
| `backup-test-failed` | بحالی کی جانچ نہیں ہو سکی | The test restore could not run | |
| `backup-test-close` | بند کریں | Close | |
| `backup-test-ok` | یہ کاپی ٹھیک ہے۔ یہ درست بحال ہوتی ہے اور اس کے کھاتے برابر ہیں۔ | This copy is good. It restores correctly and its books balance. | |
| `backup-test-not-ok` | یہ کاپی ایک جانچ میں ناکام رہی۔ اس پر بھروسا نہ کریں۔ ابھی نئی کاپی بنائیں۔ | This copy failed a check. Do not rely on it. Make a new copy now. | |
| `backup-test-totals-label` | ٹیسٹ کاپی کے کھاتے اور موجودہ کھاتے | Books in the test copy and live books | |
| `backup-check-opens` | کاپی کھل جاتی ہے | The copy opens | |
| `backup-check-checksum` | کاپی خراب نہیں ہے | The copy is not damaged | |
| `backup-check-integrity` | ڈیٹا فائل درست ہے | The data file is sound | |
| `backup-check-counts` | اندراجات کی تعداد درست ہے | The number of records is right | |
| `backup-check-balanced` | کھاتے برابر ہیں | The books balance | |
| `backup-check-passed` | پاس | Passed | |
| `backup-check-failed` | ناکام | Failed | |
| `backup-key-title` | بیک اپ کی چابی | Backup key | |
| `backup-key-help` | یہ چابی یو ایس بی ڈرائیو اور کلاؤڈ کی کاپیوں کو بند کرتی ہے۔ اس کے بغیر انہیں کوئی نہیں کھول سکتا، ہم بھی نہیں۔ یہ صرف ایک بار دکھائی جاتی ہے۔ اسے پرنٹ کریں اور اس کمپیوٹر سے دور کسی محفوظ جگہ رکھیں۔ | This key locks the copies on the USB drive and in the cloud. Without it nobody can open them, not even us. It is shown only once. Print it and keep it in a safe place, away from this computer. | |
| `backup-key-failed` | بیک اپ کی چابی نہیں بن سکی | The backup key could not be made | |
| `backup-key-already` | بیک اپ کی چابی پہلے سے استعمال میں ہے، اس لیے نئی نہیں بن سکتی۔ چابی دوبارہ نہیں دکھائی جاتی۔ | A backup key is in use already, so a new one cannot be made. The key is not shown again. | |
| `backup-key-continue` | آگے بڑھیں | Continue | |
| `backup-key-made-on` | <num>{ $date }</num> کو بنی | Made on <num>{ $date }</num> | |
| `backup-key-printed` | میں نے چابی پرنٹ کر کے محفوظ جگہ رکھ دی ہے | I have printed the key and put it in a safe place | |
| `backup-key-print` | پرنٹ کریں | Print | |
| `backup-key-type-label` | پرنٹ کی ہوئی پرچی سے چابی لکھیں | Type the key from the printed sheet | |
| `backup-key-type-hint` | اس سے ثابت ہوتا ہے کہ چابی آپ کے پاس ہے۔ جیسی پرنٹ ہوئی ہے ویسی ہی لکھیں، ڈیش سمیت۔ | This proves that you have the key. Copy it exactly as printed, with the dashes. | |
| `backup-key-confirm` | چابی کی تصدیق کریں | Confirm the key | |
| `backup-key-confirmed` | بیک اپ کی چابی کی تصدیق ہو گئی۔ | The backup key is confirmed. | |
| `backup-key-later` | یہ بعد میں کریں | Do this later | |
| `backup-key-later-help` | ابھی چھوڑ دیں تو یہ چابی محفوظ نہیں رہتی۔ آپ بیک اپ کی سکرین پر نئی چابی بنا سکتے ہیں۔ | The key is not kept if you leave now. You can make a new one on the Backup screen. | |
| `backup-key-sheet-title` | بیک اپ کی چابی | Backup key | |
| `backup-key-sheet-note` | اس پرچی کو کمپیوٹر سے دور محفوظ جگہ رکھیں۔ جس کے پاس یہ چابی ہو وہ آپ کے ڈیٹا کی کاپیاں کھول سکتا ہے۔ یہ کھو جائے تو یو ایس بی ڈرائیو اور کلاؤڈ کی کاپیاں نہیں کھل سکیں گی۔ | Keep this sheet in a safe place, away from the computer. Anyone who has this key can open the copies of your data. If it is lost, the copies on the USB drive and in the cloud cannot be opened. | |
| `backup-key-make` | بیک اپ کی چابی بنائیں | Make the backup key | |
| `backup-key-missing` | ابھی بیک اپ کی چابی نہیں ہے۔ اس کے بغیر کاپیاں بند کر کے یو ایس بی ڈرائیو یا کلاؤڈ پر نہیں بھیجی جا سکتیں۔ | There is no backup key yet. Without it, copies cannot be locked and sent to the USB drive or the cloud. | |
| `backup-key-confirm-due` | آپ نے بیک اپ کی چابی 7 دن سے زیادہ پہلے بنائی تھی اور اس کی تصدیق نہیں کی۔ اسے پرنٹ کی ہوئی پرچی سے لکھیں۔ | You made the backup key more than 7 days ago and have not confirmed it. Type it from your printed sheet. | |
| `backup-key-unconfirmed` | آپ نے بیک اپ کی چابی کی ابھی تصدیق نہیں کی۔ ثابت کرنے کے لیے اسے پرنٹ کی ہوئی پرچی سے لکھیں۔ | You have not confirmed the backup key yet. Type it from your printed sheet to prove that you have it. | |
| `backup-key-lost` | پرنٹ کی ہوئی پرچی کھو گئی ہے | I lost the printed sheet | |
| `backup-key-lost-help` | نئی چابی تب ہی بن سکتی ہے جب یو ایس بی ڈرائیو یا کلاؤڈ کی کوئی کاپی پرانی چابی سے بند نہ ہو۔ | A new key can be made only while no copy on the USB drive or in the cloud is locked with the old one. | |
| `backup-wizard-step-key` | بیک اپ کی چابی | Backup key | |
| `backup-wizard-key-title` | بیک اپ کی چابی پرنٹ کر کے محفوظ رکھیں | Print the backup key and keep it safe | |
| `backup-bar-ok` | کلاؤڈ بیک اپ: <num>{ $date }</num> | Cloud backup: <num>{ $date }</num> | |
| `backup-bar-ok-queue` | کلاؤڈ بیک اپ: <num>{ $date }</num>، <num>{ $queued }</num> انتظار میں | Cloud backup: <num>{ $date }</num>, <num>{ $queued }</num> waiting | |
| `backup-bar-old` | کلاؤڈ بیک اپ پرانا ہے: <num>{ $date }</num> | Cloud backup is old: <num>{ $date }</num> | |
| `backup-bar-never` | ابھی کلاؤڈ بیک اپ نہیں ہوا | No cloud backup yet | |
| `backup-bar-check-failed` | ایک بیک اپ جانچ میں ناکام رہا | A backup failed its check | |
| `backup-bar-unknown` | بیک اپ: حالت معلوم نہیں | Backup: state unknown | |
| `backup-exit-title` | بند کرنے سے پہلے بیک اپ لیا جا رہا ہے | Saving a backup before closing | |
| `backup-exit-help` | بیک اپ مکمل ہوتے ہی ایپ خود بند ہو جائے گی۔ براہ کرم انتظار کریں۔ | The app closes by itself as soon as the backup is done. Please wait. | |
| `backup-set-title` | بیک اپ کی ترتیبات | Backup settings | |
| `backup-settings-failed` | بیک اپ کی ترتیبات کھل نہیں سکیں | The backup settings could not be opened | |
| `backup-settings-saved` | بیک اپ کی ترتیبات محفوظ ہو گئیں | Backup settings saved | |
| `backup-set-local-dir` | اس کمپیوٹر پر کاپیوں کا فولڈر | Folder for the copies on this computer | |
| `backup-set-local-dir-hint` | خالی چھوڑیں تو <code>{ $folder }</code> استعمال ہو گا۔ پورا راستہ لکھیں۔ دوسری ڈسک پر فولڈر ہو تو یہ ڈسک خراب ہونے پر بھی آپ کی کاپیاں بچی رہتی ہیں۔ | Leave it empty to use <code>{ $folder }</code>. Use a full path. A folder on a second disk keeps your copies if this disk fails. | |
| `backup-set-closing-time` | روزانہ کاپی کا وقت | Daily copy time | |
| `backup-set-closing-time-hint` | گھنٹے اور منٹ، مثلاً 21:00۔ یو ایس بی اور کلاؤڈ کی کاپی ہر روز اسی وقت بنتی ہے۔ | Hours and minutes, for example 21:00. The USB and cloud copies are made at this time every day. | |
| `backup-set-usb-on` | یو ایس بی ڈرائیو پر کاپی بنائیں | Make a copy on a USB drive | |
| `backup-set-usb-folder` | یو ایس بی ڈرائیو پر فولڈر | Folder on the USB drive | |
| `backup-set-usb-folder-hint` | پورا راستہ، مثلاً E:\MandiBackup | The full path, for example E:\MandiBackup | |
| `backup-set-cloud-on` | کلاؤڈ میں کاپی بنائیں | Make a copy in the cloud | |
| `backup-set-endpoint` | کلاؤڈ کا پتہ | Cloud address | |
| `backup-set-endpoint-hint` | مثلاً https://your-account.r2.cloudflarestorage.com | For example https://your-account.r2.cloudflarestorage.com | |
| `backup-set-bucket` | بکٹ کا نام | Bucket name | |
| `backup-set-region` | علاقہ | Region | |
| `backup-set-region-hint` | Cloudflare R2 میں auto لکھا جاتا ہے۔ | Cloudflare R2 uses auto. | |
| `backup-set-prefix` | بکٹ کے اندر فولڈر | Folder inside the bucket | |
| `backup-set-prefix-hint` | اس دکان کی سب کاپیوں کا ایک نام، مثلاً mandi۔ | A name for all the copies of this shop, for example mandi. | |
| `backup-set-key-id` | کلاؤڈ کی رسائی کی چابی | Cloud access key | |
| `backup-set-secret` | کلاؤڈ کی خفیہ چابی | Cloud secret key | |
| `backup-set-key-stored` | چابی محفوظ ہے۔ بدلنے کے لیے نئی لکھیں۔ | A key is stored. Type a new one to replace it. | |
| `backup-set-key-none` | کوئی چابی محفوظ نہیں۔ | No key is stored. | |
| `backup-set-key-will-remove` | محفوظ کرنے پر محفوظ چابی ہٹا دی جائے گی۔ | The stored key will be removed when you save. | |
| `backup-set-key-remove` | محفوظ چابی ہٹائیں | Remove the stored key | |

## bikri

| ID | Urdu | English | Result |
|---|---|---|---|
| `bikri-help` | ٹرک، لاٹ .gender = masculine اور بکری .gender = feminine کی لائنیں لکھیں۔ ہر رقم لکھتے ہی خود بنتی ہے۔ | Enter the truck, the lots and the sale lines. Every amount is worked out as you type. | |
| `bikri-load-failed` | بکری بک کھل نہیں سکی | The sale book could not open | |
| `bikri-preview-failed` | حساب نہیں بن سکا | The numbers could not be worked out | |
| `bikri-empty-title` | ابھی کچھ نہیں لکھا | Nothing entered yet | |
| `bikri-empty-help` | ٹرک نمبر یا بیوپاری .gender = masculine سے شروع کریں۔ ہر رقم لکھتے ہی یہاں نظر آئے گی۔ | Start with the truck number or the seller. Every amount shows here as you type. | |
| `bikri-empty-action` | ٹرک نمبر لکھیں | Enter the truck number | |
| `bikri-header-date` | تاریخ | Date | |
| `bikri-header-bill-date` | تاریخ بل | Bill date | |
| `bikri-header-truck` | ٹرک نمبر | Truck number | |
| `bikri-header-total-freight` | کل کرایہ .gender = masculine | Total Freight | |
| `bikri-header-split-freight` | تقسیم | Split | |
| `bikri-header-book` | بک | Book | |
| `bikri-header-store` | سٹور | Store | |
| `bikri-header-driver` | ڈرائیور | Driver | |
| `bikri-header-driver-phone` | ڈرائیور فون | Driver phone | |
| `bikri-header-note` | نوٹ | Note | |
| `bikri-header-more` | مزید | More | |
| `bikri-header-less` | کم | Less | |
| `bikri-header-filled-from` | اڈا .gender = masculine، زمیندار .gender = masculine اور ڈرائیور اس ٹرک کی بکری .gender = feminine نمبر { $number } ({ $date }) سے بھرے گئے ہیں۔ آپ انہیں بدل سکتے ہیں۔ | The Adda, Zamindar and driver were filled from sale number { $number } of this truck ({ $date }). You can change them. | |
| `bikri-truck-last` | آخری بکری .gender = feminine نمبر { $number }، { $date } | Last sale: number { $number }, { $date } | |
| `bikri-truck-same-day` | اس ٹرک کی { $date } کی بکری .gender = feminine نمبر { $number } پہلے سے موجود ہے۔ | This truck already has sale number { $number } on { $date }. | |
| `bikri-truck-open` | کھولیں | Open | |
| `bikri-truck-continue` | نئی بکری .gender = feminine جاری رکھیں | Continue with this new sale | |
| `bikri-lots-title` | لاٹ .gender = masculine | Lots | |
| `bikri-lots-label` | لاٹ .gender = masculine کی فہرست | Lots of this sale | |
| `bikri-lots-add` | لاٹ .gender = masculine شامل کریں | Add a lot | |
| `bikri-sales-title` | لاٹ .gender = masculine { $lot } کی بکری .gender = feminine | Sales of lot { $lot } | |
| `bikri-sales-label` | بکری .gender = feminine کی لائنیں | Sale lines | |
| `bikri-col-seller` | بیوپاری .gender = masculine | Seller | |
| `bikri-col-qty-received` | آمد | Received | |
| `bikri-col-balance` | باقی | Left | |
| `bikri-col-qty` | تعداد | Qty | |
| `bikri-col-rate` | ریٹ | Rate | |
| `bikri-col-seller-amount` | رقم بیوپاری .gender = masculine | Seller amount | |
| `bikri-col-seller-comm` | کمیشن .gender = masculine بیوپاری .gender = masculine | Seller commission | |
| `bikri-col-buyer-comm` | کمیشن .gender = masculine گاہک .gender = masculine | Buyer commission | |
| `bikri-col-buyer-total` | رقم گاہک .gender = masculine | Buyer total | |
| `bikri-col-payment` | ادھاری / نقدی | Credit / Cash | |
| `bikri-rate-hint-this-bikri` | اسی بکری .gender = feminine میں آخری ریٹ { $rate } | Last rate in this sale: { $rate } | |
| `bikri-rate-hint-today` | آج کا آخری ریٹ { $rate } | Today's last rate: { $rate } | |
| `bikri-rate-hint-last-day` | { $date } کا آخری ریٹ { $rate } | Last rate on { $date }: { $rate } | |
| `bikri-more-toggle` | لاٹ .gender = masculine { $lot } کی مزید باتیں | More about lot { $lot } | |
| `bikri-more-weight` | وزن (کلو) | Weight (kg) | |
| `bikri-more-freight-paid-by` | کرایہ .gender = masculine کس نے دیا | Freight paid by | |
| `bikri-more-adjust-advance` | پیشگی .gender = feminine سے کاٹیں | Deduct from advance | |
| `bikri-more-advance-open` | کھلی پیشگی .gender = feminine: <num>{ $amount }</num> | Open advance: <num>{ $amount }</num> | |
| `bikri-more-charge-method` | مزدوری .gender = feminine، منشیانہ اور فون .gender = masculine کی لکھی ہوئی رقم کا مطلب | What the amount typed for Labour, Munshiana and Phone Charge means | |
| `bikri-method-manual` | کل رقم | Whole amount | |
| `bikri-method-per-unit` | فی نگ | Per Nag | |
| `bikri-method-per-lot` | فی لاٹ .gender = masculine | Per Lot | |
| `bikri-more-rule-per-unit` | اصول: <num>{ $value }</num> فی نگ | Rule: <num>{ $value }</num> per Nag | |
| `bikri-more-rule-per-lot` | اصول: <num>{ $value }</num> فی لاٹ .gender = masculine | Rule: <num>{ $value }</num> per Lot | |
| `bikri-more-rule-manual` | اصول: <num>{ $value }</num> | Rule: <num>{ $value }</num> | |
| `bikri-line-title` | لائن { $line } کی تفصیل | Details of line { $line } | |
| `bikri-line-seller-comm` | کمیشن .gender = masculine بیوپاری .gender = masculine | Seller commission | |
| `bikri-line-buyer-comm` | کمیشن .gender = masculine گاہک .gender = masculine | Buyer commission | |
| `bikri-line-method-percent` | % | % | |
| `bikri-line-method-per-unit` | فی نگ | Per Nag | |
| `bikri-line-override-off` | لائن پر کمیشن .gender = masculine اور لاگا .gender = masculine بدلنا بند ہے۔ مالک اسے ترتیبات میں چالو کر سکتا ہے۔ | Changing the Commission and Laga on a line is switched off. The Owner can switch it on in Settings. | |
| `bikri-line-differs` | ریٹ مختلف | Different rate | |
| `bikri-line-buyer-rate` | ریٹ گاہک .gender = masculine | Buyer rate | |
| `bikri-line-weight` | وزن (کلو) | Weight (kg) | |
| `bikri-line-note` | نوٹ | Note | |
| `bikri-tag-manual` | دستی | Manual | |
| `bikri-tag-percent` | { $value }% | { $value }% | |
| `bikri-tag-per-unit` | { $value } فی نگ | { $value } per Nag | |
| `bikri-pukhta-name-ur` | اردو نام | Urdu name | |
| `bikri-pukhta-name-en` | انگریزی نام | English name | |
| `bikri-pukhta-phone` | فون نمبر | Phone number | |
| `bikri-summary-title` | خلاصہ | Summary | |
| `bikri-summary-lot` | لاٹ .gender = masculine { $lot } | Lot { $lot } | |
| `bikri-summary-whole` | پوری بکری .gender = feminine | Whole sale | |
| `bikri-summary-incomplete` | اس لاٹ .gender = masculine کا حساب ابھی نہیں بن سکتا۔ گاہک .gender = masculine، تعداد اور ریٹ لکھیں۔ | This lot cannot be worked out yet. Enter the buyer, the quantity and the rate. | |
| `bikri-summary-seller-side` | بیوپاری .gender = masculine کی طرف | Seller side | |
| `bikri-summary-buyer-side` | گاہک .gender = masculine کی طرف | Buyer side | |
| `bikri-summary-deductions` | کل اخراجات | Total deductions | |
| `bikri-summary-buyer-comm` | کمیشن .gender = masculine گاہک .gender = masculine | Buyer commission | |
| `bikri-summary-buyer-total` | گاہک .gender = masculine کی کل رقم | Buyer bills | |
| `bikri-summary-net-negative` | صافی بکری .gender = feminine صفر سے کم ہے۔ یہ رقم بیوپاری .gender = masculine کے ذمے ہے۔ | The Net Sale is below zero. The seller owes this amount. | |
| `bikri-split-title` | شراکت داروں کا حصہ .gender = masculine | Partners' share | |
| `bikri-split-shared` | مشترکہ آمدن | Shared income | |
| `bikri-where-lot` | لاٹ .gender = masculine { $lot }: | Lot { $lot }: | |
| `bikri-where-line` | لاٹ .gender = masculine { $lot }، لائن { $line }: | Lot { $lot }, line { $line }: | |
| `bikri-shares-open-setup` | شراکت دار کی ترتیب کھولیں | Open Partner setup | |
| `bikri-col-lot` | لاٹ .gender = masculine | Lot | |
| `bikri-all-lot` | { $lot } { $seller } | { $lot } { $seller } | |
| `bikri-all-lot-no-seller` | { $lot } | { $lot } | |
| `bikri-sales-title-all` | بکری .gender = feminine کی تمام لائنیں | All lines of the sale | |
| `bikri-strip-label` | اس بکری .gender = feminine کے اہم اعداد | Key numbers of this sale | |
| `bikri-strip-incomplete` | ابھی حساب نہیں ہوا | Not worked out yet | |
| `bikri-sales-view-label` | کون سی بکری .gender = feminine کی لائنیں دکھائیں | Which sale lines to show | |
| `bikri-save` | محفوظ کریں | Save | |
| `bikri-new` | نئی بکری .gender = feminine | New Sale | |
| `bikri-saved-toast` | بکری .gender = feminine { $number } محفوظ ہو گئی | Sale { $number } saved | |
| `bikri-last-saved` | آخری محفوظ بکری .gender = feminine: نمبر { $number } | Last saved Sale: number { $number } | |
| `bikri-open-failed` | بکری .gender = feminine کھل نہیں سکی۔ دوبارہ کوشش کریں۔ | The Sale could not be opened. Please try again. | |
| `bikri-nav-label` | بکری .gender = feminine کے درمیان جائیں | Move between sales | |
| `bikri-nav-first` | پہلی بکری .gender = feminine | First Sale | |
| `bikri-nav-previous` | پچھلی بکری .gender = feminine | Previous Sale | |
| `bikri-nav-next` | اگلی بکری .gender = feminine | Next Sale | |
| `bikri-nav-last` | آخری بکری .gender = feminine | Last Sale | |
| `bikri-nav-none` | ابھی کوئی بکری .gender = feminine محفوظ نہیں۔ | No Sale is saved yet. | |
| `bikri-nav-no-previous` | اس سے پچھلی کوئی بکری .gender = feminine نہیں۔ | There is no Sale before this one. | |
| `bikri-nav-no-next` | اس کے بعد کوئی بکری .gender = feminine نہیں۔ | There is no Sale after this one. | |
| `bikri-find-open` | تلاش | Find | |
| `bikri-find-title` | بکری .gender = feminine تلاش کریں | Find a Sale | |
| `bikri-find-help` | نمبر، ٹرک، بیوپاری .gender = masculine، تاریخ یا قسم مال میں سے جو معلوم ہو لکھیں۔ | Type whatever you know: number, truck, Seller, date or Item. | |
| `bikri-find-number` | بکری .gender = feminine نمبر | Sale number | |
| `bikri-find-hint` | اوپر کوئی ایک بات لکھیں۔ ملتی ہوئی بکری .gender = feminine یہاں نظر آئیں گی۔ | Type at least one thing above. The sales that match will show here. | |
| `bikri-find-failed` | تلاش نہیں ہو سکی | The search could not be done | |
| `bikri-find-none` | کوئی بکری .gender = feminine نہیں ملی | No Sale found | |
| `bikri-find-none-help` | لکھی ہوئی باتیں جانچیں۔ ساری باتیں ایک ساتھ ملنی چاہییں۔ | Check what you typed. Everything you typed must match together. | |
| `bikri-find-results` | ملی ہوئی بکری .gender = feminine | Sales found | |
| `bikri-hit-title` | بکری .gender = feminine نمبر { $number } | Sale number { $number } | |
| `bikri-status-saved` | محفوظ | Saved | |
| `bikri-status-cancelled` | منسوخ | Cancelled | |
| `bikri-hit-cancelled` | { bikri-status-cancelled } | { bikri-status-cancelled } | |
| `bikri-draft-title` | پچھلی بکری .gender = feminine مکمل کریں؟ | Finish the last Sale? | |
| `bikri-draft-text` | ایپ بند ہونے سے پہلے ایک بکری .gender = feminine لکھی جا رہی تھی۔ وہ { $time } پر محفوظ کی گئی تھی۔ کیا اسے دوبارہ کھولنا ہے؟ | A Sale was being typed before the app closed. It was kept at { $time }. Do you want to open it again? | |
| `bikri-draft-later` | ابھی نہیں | Not now | |
| `bikri-draft-discard` | ختم کر دیں | Discard | |
| `bikri-draft-restore` | مکمل کریں | Finish it | |
| `bikri-saved-title` | بکری .gender = feminine نمبر { $number } | Sale number { $number } | |
| `bikri-saved-readonly` | یہ محفوظ شدہ بکری .gender = feminine ہے۔ یہاں اسے صرف دیکھا جا سکتا ہے۔ | This Sale is saved. Here it can only be viewed. | |
| `bikri-saved-cancelled` | یہ بکری .gender = feminine منسوخ ہو چکی ہے۔ | This Sale has been cancelled. | |
| `bikri-saved-cancel-reason` | وجہ: { $reason } | Reason: { $reason } | |
| `bikri-saved-at` | محفوظ ہونے کا وقت | Time saved | |
| `bikri-saved-failed` | بکری .gender = feminine کھل نہیں سکی | The Sale could not be opened | |
| `bikri-saved-lot-title` | لاٹ .gender = masculine { $lot }: { $seller } | Lot { $lot }: { $seller } | |
| `bikri-saved-pick-lot` | کون سی لاٹ .gender = masculine دکھانی ہے | Which Lot to show | |
| `bikri-saved-qty-sold` | فروخت | Sold | |
| `bikri-saved-qty-written-off` | ضائع | Written off | |
| `bikri-saved-qty-taken` | پھل | Fruit Taken | |
| `bikri-split-income` | آمدن کا حصہ | Share of income | |
| `bikri-postings-title` | حساب کے اندراج | Entries in the books | |
| `bikri-posting-kind` | اندراج | Entry | |
| `bikri-posting-total` | رقم | Amount | |
| `bikri-posting-first` | پہلا اندراج | First entry | |
| `bikri-posting-later` | بعد کی فروخت | Later sale | |
| `bikri-posting-reversal` | الٹا اندراج | Reversal | |
| `bikri-posting-other` | اندراج | Entry | |
| `bikri-list-label` | بکری .gender = feminine کی فہرست | List of sales | |
| `bikri-list-help` | دن، بیوپاری .gender = masculine اور قسم مال سے چھانٹیں۔ کھولنے کے لیے Enter دبائیں۔ | Filter by day, Seller and Item. Press Enter to open one. | |
| `bikri-list-failed` | فہرست کھل نہیں سکی | The list could not be opened | |
| `bikri-list-empty` | آج کوئی بکری .gender = feminine نہیں | No sales today | |
| `bikri-list-empty-help` | نئی بکری .gender = feminine کے لیے { $key } دبائیں۔ | Press { $key } for a new Sale. | |
| `bikri-list-no-match` | کوئی بکری .gender = feminine نہیں ملی | No Sale found | |
| `bikri-list-no-match-help` | دن یا باقی چھانٹیاں بدل کر دیکھیں۔ | Try changing the day or the other filters. | |
| `bikri-list-clear` | چھانٹیاں ہٹائیں | Clear the filters | |
| `bikri-list-status-all` | سب | All | |
| `bikri-list-separator` | { "، " } | { ", " } | |
| `bikri-list-col-number` | نمبر | Number | |
| `bikri-list-col-when` | تاریخ اور وقت | Date and time | |
| `bikri-list-col-truck` | ٹرک | Truck | |
| `bikri-list-col-sellers` | بیوپاری .gender = masculine | Seller | |
| `bikri-list-col-items` | قسم مال | Item | |
| `bikri-list-col-buyer-total` | گاہک .gender = masculine کی کل رقم | Buyer total | |
| `bikri-list-col-status` | حالت | Status | |
| `bikri-list-col-qty` | نگ ({ bikri-saved-qty-sold } / { bikri-col-qty-received }) | Nag ({ bikri-saved-qty-sold } / { bikri-col-qty-received }) | |
| `bikri-list-qty` | { $received } / { $sold } | { $sold } / { $received } | |
| `bikri-stock-help` | وہ لاٹ .gender = masculine جن کا مال ابھی بکا نہیں۔ بچا ہوا مال بیچنے کے لیے لاٹ .gender = masculine پر Enter دبائیں۔ | Lots with goods that are not sold yet. Press Enter on a lot to sell what is left. | |
| `bikri-stock-label` | بچے ہوئے مال والے لاٹ .gender = masculine | Lots with goods left | |
| `bikri-stock-failed` | باقی مال کی فہرست کھل نہیں سکی | The list of Unsold Stock could not be opened | |
| `bikri-stock-empty` | کوئی مال باقی نہیں | No goods are waiting | |
| `bikri-stock-empty-help` | جو مال آیا وہ سب بک چکا ہے۔ نئی بکری .gender = feminine کے لیے { $key } دبائیں۔ | Everything received has been sold. Press { $key } for a new Sale. | |
| `bikri-stock-no-match` | کوئی لاٹ .gender = masculine نہیں ملا | No lot found | |
| `bikri-stock-no-match-help` | دن، بیوپاری .gender = masculine یا قسم مال بدل کر دیکھیں۔ | Try changing the days, the Seller or the Item. | |
| `bikri-stock-from` | وصول ہونے کی تاریخ سے | Received from | |
| `bikri-stock-to` | وصول ہونے کی تاریخ تک | Received up to | |
| `bikri-stock-include-written-off` | وہ لاٹ .gender = masculine بھی جن کا سارا مال ضائع ہو چکا ہے، تاکہ ضائع مال واپس کیا جا سکے | Also lots that are fully written off, so a write-off can be undone | |
| `bikri-stock-col-received-date` | وصولی کا دن | Received on | |
| `bikri-stock-col-item` | قسم مال اور پیکنگ .gender = feminine | Item and Packing | |
| `bikri-stock-item-packing` | { $item }، { $packing } | { $item }, { $packing } | |
| `bikri-stock-col-written-off` | ضائع | Written off | |
| `bikri-stock-col-taken` | پھل | Fruit Taken | |
| `bikri-stock-col-days` | انتظار کے دن | Days waiting | |
| `bikri-stock-lot-title` | بکری .gender = feminine { $number }، لاٹ .gender = masculine { $lot } | Sale { $number }, Lot { $lot } | |
| `bikri-stock-sell` | بچا ہوا مال بیچیں | Sell what is left | |
| `bikri-stock-write-off` | ضائع کریں | Write off | |
| `bikri-stock-undo-write-off` | ضائع مال واپس کریں | Undo a write-off | |
| `bikri-write-off-title` | مال ضائع کریں | Write off goods | |
| `bikri-write-off-text` | بکری .gender = feminine { $number } ({ $seller }، { $item }) کا خراب یا واپس آیا ہوا مال لاٹ .gender = masculine سے نکالا جائے گا۔ اب { $balance } نگ باقی ہیں۔ یہ صرف مال کا اندراج ہے، کوئی رقم نہیں ہلتی۔ | Goods of Sale { $number } ({ $seller }, { $item }) that are spoiled or came back are taken off the lot. { $balance } Nag are left now. This only changes the stock; no money moves. | |
| `bikri-write-off-qty` | ضائع کرنے والے نگ | Nag to write off | |
| `bikri-write-off-reason` | وجہ (مثلاً خراب، واپس) | Reason (for example spoiled, returned) | |
| `bikri-write-off-confirm` | ضائع کریں | Write off | |
| `bikri-write-off-done` | { $qty } نگ ضائع ہو گئے | { $qty } Nag written off | |
| `bikri-write-off-undo-title` | ضائع مال واپس کریں | Undo a write-off | |
| `bikri-write-off-undo-text` | مال بغیر بکے مال کی طرح لاٹ .gender = masculine میں واپس آ جائے گا۔ ضائع کا اندراج چنیں اور وجہ لکھیں۔ | The goods go back to the lot as unsold stock. Choose the write-off and give the reason. | |
| `bikri-write-off-undo-which` | ضائع کا اندراج | Write-off | |
| `bikri-write-off-undo-option` | { $date }: { $qty } نگ ({ $reason }) | { $date }: { $qty } Nag ({ $reason }) | |
| `bikri-write-off-undo-confirm` | ضائع واپس کریں | Undo the write-off | |
| `bikri-write-off-undone` | ضائع کا اندراج واپس ہو گیا | The write-off was undone | |
| `bikri-later-title` | بکری .gender = feminine { $number } کی بعد کی فروخت | Later sale of Sale { $number } | |
| `bikri-later-help` | بچے ہوئے مال کی نئی لائنیں لکھیں۔ صرف یہی لائنیں نیچے لکھے دن پر لگیں گی۔ پہلا اندراج نہیں چھیڑا جائے گا۔ | Type the new lines for the goods that are left. Only these lines are posted, on the day below. The first entry is not touched. | |
| `bikri-later-lot-title` | لاٹ .gender = masculine | The lot | |
| `bikri-later-no-lot-charges` | لاٹ .gender = masculine کا کرایہ، مزدوری .gender = feminine، منشیانہ اور باقی خرچے دوبارہ نہیں لگتے۔ کمیشن .gender = masculine اور لاگا .gender = masculine اس فروخت کے دن کے اصولوں سے بنتے ہیں۔ | Freight, Labour, Munshiana and the other charges of the lot are not charged again. Commission and Laga are worked out by the rules of the day of this sale. | |
| `bikri-later-date` | اس فروخت کا دن | Day of this sale | |
| `bikri-lines-title` | فروخت کی نئی لائنیں | New sale lines | |
| `bikri-later-save` | فروخت محفوظ کریں | Save the sale | |
| `bikri-later-saved-toast` | بکری .gender = feminine { $number } کی بعد کی فروخت محفوظ ہو گئی | Later sale of Sale { $number } saved | |
| `bikri-later-to-stock` | باقی مال پر واپس | Back to Unsold Stock | |
| `bikri-later-no-lot` | پہلے لاٹ .gender = masculine چنیں | Choose a lot first | |
| `bikri-later-no-lot-help` | بعد کی فروخت بچے ہوئے مال کی فہرست سے شروع ہوتی ہے۔ | A later sale starts from the list of goods left. | |
| `bikri-later-failed` | لاٹ .gender = masculine کھل نہیں سکا | The lot could not be opened | |
| `bikri-later-lot-missing` | یہ لاٹ .gender = masculine نہیں ملا | This lot was not found | |
| `bikri-later-lot-missing-help` | ہو سکتا ہے یہ منسوخ ہو چکا ہو۔ بچے ہوئے مال کی فہرست کھول کر دوبارہ چنیں۔ | It may have been cancelled. Open the list of goods left and choose again. | |
| `bikri-later-nothing-left` | اس لاٹ .gender = masculine میں کچھ باقی نہیں | Nothing is left in this lot | |
| `bikri-later-nothing-left-help` | سارا مال بک چکا ہے یا ضائع ہو چکا ہے۔ | All of it is sold or written off. | |
| `bikri-edit` | ترمیم | Edit | |
| `bikri-cancel-bikri` | بکری .gender = feminine منسوخ کریں | Cancel the Sale | |
| `bikri-edit-back` | بکری .gender = feminine پر واپس | Back to the Sale | |
| `bikri-edit-save` | تبدیلیاں محفوظ کریں | Save changes | |
| `bikri-edit-title` | بکری .gender = feminine { $number } میں ترمیم | Edit Sale { $number } | |
| `bikri-edit-help` | آپ { $date } کے اندراج میں ترمیم کر رہے ہیں۔ اس کا دن وہی رہے گا۔ پرانا اندراج الٹ دیا جائے گا اور اس کی جگہ نیا اندراج لگے گا۔ | You are editing the entry of { $date }. Its day stays the same. The old entry is reversed and the new one is posted in its place. | |
| `bikri-edit-help-later` | آپ { $date } کی بعد کی فروخت میں ترمیم کر رہے ہیں۔ اس کا دن وہی رہے گا۔ صرف لائنیں بدلی جا سکتی ہیں۔ | You are editing the later sale of { $date }. Its day stays the same. Only the lines can be changed. | |
| `bikri-edit-failed` | اندراج ترمیم کے لیے کھل نہیں سکا | The entry could not be opened for editing | |
| `bikri-edit-saved-toast` | بکری .gender = feminine { $number } میں تبدیلی ہو گئی | Sale { $number } changed | |
| `bikri-edit-reason-title` | یہ اندراج کیوں بدلا جا رہا ہے؟ | Why is this entry being changed? | |
| `bikri-edit-reason-text` | { $date } کا اندراج آج کا نہیں ہے۔ وجہ پرانی اور نئی قدروں کے ساتھ آڈٹ لاگ میں لکھی جائے گی۔ | The entry of { $date } is not of today. The reason is written to the audit log with the old and the new values. | |
| `bikri-edit-reason-label` | وجہ | Reason | |
| `bikri-edit-reason-confirm` | تبدیلیاں محفوظ کریں | Save changes | |
| `bikri-edit-changes-title` | آپ نے کیا بدلا | What you changed | |
| `bikri-edit-no-changes` | ابھی کچھ نہیں بدلا۔ | Nothing is changed yet. | |
| `bikri-edit-change` | { $where }: { $field } پہلے { $before } تھا، اب { $after } | { $where }: { $field } was { $before }, now { $after } | |
| `bikri-edit-added` | { $where }: نیا { $field } { $after } | { $where }: new { $field } { $after } | |
| `bikri-edit-removed-line` | { $where }: لائن ہٹا دی گئی ({ $buyer }) | { $where }: line removed ({ $buyer }) | |
| `bikri-edit-where-header` | اوپر کی معلومات | Header | |
| `bikri-edit-where-lot` | لاٹ .gender = masculine { $lot } | Lot { $lot } | |
| `bikri-edit-where-line` | لاٹ .gender = masculine { $lot }، لائن { $line } | Lot { $lot }, line { $line } | |
| `bikri-edit-empty` | (خالی) | (empty) | |
| `bikri-edit-was` | بدلا گیا۔ پہلے یہ تھا: { $value } | Changed. It was: { $value } | |
| `bikri-edit-was-empty` | بدلا گیا۔ پہلے یہ خالی تھا۔ | Changed. It was empty. | |
| `bikri-edit-compare-what` | رقم | Amount | |
| `bikri-edit-before` | پہلے | Before | |
| `bikri-edit-after` | بعد میں | After | |
| `bikri-closed-day` | { $date } کا دن بند ہے، اس لیے یہ اندراج بدلا یا منسوخ نہیں ہو سکتا۔ پہلے مالک کو دن کھولنا ہو گا۔ | The day { $date } is closed, so this entry cannot be changed or cancelled. The Owner must reopen the day first. | |
| `bikri-closed-day-open` | دن کی اسکرین کھولیں | Open the day screen | |
| `bikri-posting-state` | حالت | State | |
| `bikri-posting-reversed` | الٹ دیا گیا | Taken back | |
| `bikri-posting-open` | ترمیم ہو سکتی ہے | Open for changes | |
| `bikri-posting-not-editable` | دن بند ہے | Day closed | |
| `bikri-posting-edit` | یہ اندراج بدلیں | Edit this entry | |
| `bikri-cancel-title` | یہ بکری .gender = feminine منسوخ کریں؟ | Cancel this Sale? | |
| `bikri-cancel-text` | بکری .gender = feminine { $number } منسوخ ہو جائے گی۔ کچھ مٹایا نہیں جاتا: نیچے لکھا ہر اندراج اپنے ہی دن پر الٹ دیا جائے گا، رقم واپس چلی جائے گی، اور گاہک .gender = masculine کی وصولیاں اس کا جمع بن کر اس کے اگلے پرانے بلوں سے مل جائیں گی۔ یہ نمبر دوبارہ استعمال نہیں ہوگا۔ | Sale { $number } will be cancelled. Nothing is deleted: every entry below is reversed on its own day, the money goes back, and the receipts of the buyers become their credit again and are matched to their next oldest bills. The number is never used again. | |
| `bikri-cancel-reverses` | یہ اندراج الٹ دیے جائیں گے: | These entries will be reversed: | |
| `bikri-cancel-cash` | روکڑ: اب <num>{ $before }</num>، منسوخی کے بعد <num>{ $after }</num>۔ | Cash Book: <num>{ $before }</num> now, <num>{ $after }</num> after the cancel. | |
| `bikri-cancel-reason` | وجہ | Reason | |
| `bikri-cancel-confirm` | بکری .gender = feminine منسوخ کریں | Cancel the Sale | |
| `bikri-cancel-preview-failed` | منسوخی کا اثر پڑھا نہیں جا سکا۔ یہ ڈبہ بند کر کے دوبارہ کوشش کریں۔ | What the cancel would do could not be read. Close this box and try again. | |
| `bikri-cancelled-toast` | بکری .gender = feminine { $number } منسوخ ہو گئی | Sale { $number } cancelled | |
| `bikri-undo` | واپس لیں | Undo | |
| `bikri-undo-reason` | محفوظ کرنے کے فوراً بعد واپس | Undone straight after saving | |
| `bikri-undo-done` | بکری .gender = feminine { $number } واپس لے لی گئی۔ اسے درست کر کے دوبارہ محفوظ کریں۔ | Sale { $number } was taken back. Correct it and save again. | |
| `bikri-undo-failed` | محفوظ کیا ہوا اندراج واپس نہیں ہو سکا | The save could not be undone | |
| `bikri-undo-busy` | آپ نے دوسری بکری .gender = feminine شروع کر دی ہے۔ پہلے اسے مکمل کریں یا صاف کریں، پھر واپس لیں۔ | You have started another Sale. Finish it or clear it first, then undo. | |
| `bikri-own-title` | اپنا مال: مارکیٹ سفر .gender = masculine کا مال | Own Goods: goods of a Market Trip | |
| `bikri-own-purchase` | خریداری | Purchase | |
| `bikri-own-purchase-hint` | قسم مال، پیکنگ .gender = feminine اور مقدار خریداری سے آتے ہیں۔ لائنوں میں ساری مقدار بکنی چاہیے۔ اس مال پر نقد .gender = masculine اور پیشگی کا نشان لاگو نہیں ہوتے۔ | The Item, the Packing and the quantity come from the purchase. The lines must sell all of the quantity. Cash paid and the advance tick do not apply to these goods. | |
| `bikri-own-option` | { $number } ({ $trip }): { $item }، { $packing } ({ $qty_left } باقی) | { $number } ({ $trip }): { $item }, { $packing } ({ $qty_left } left) | |
| `bikri-own-load-failed` | خریداریاں پڑھی نہیں جا سکیں۔ یہ لاٹ .gender = masculine بند کر کے دوبارہ کھولیں۔ | The purchases could not be read. Close this lot and open it again. | |
| `bikri-own-none` | کسی خریداری میں مال باقی نہیں۔ پہلے مارکیٹ سفر .gender = masculine میں خریداری درج کریں۔ | No purchase has goods left. Record the purchase first, on a Market Trip. | |
| `bikri-own-numbers-pending` | سفر کے شراکت داروں کو اس لاٹ .gender = masculine سے کیا ملتا ہے، یہ دیکھنے کے لیے مقدار اور لائنیں لکھیں۔ | Enter the quantity and the lines to see what this lot gives to the partners of the trip. | |
| `bikri-own-commission` | کمیشن .gender = masculine (فرم کی آمدن) | Commission (income of the firm) | |
| `bikri-own-credit` | سفر کے شراکت داروں کو ملتا ہے | Goes to the partners of the trip | |
| `bikri-saved-own-title` | سفر کا مال | Trip goods | |
| `bikri-saved-cost` | بکے ہوئے مال کی لاگت | Cost of goods sold | |
| `bikri-saved-purchase` | خریداری | Purchase | |
| `bikri-saved-purchase-open` | { $trip } میں { $number } کھولیں | Open { $number } in { $trip } | |
| `bikri-stock-own-title` | اپنا مال | Own Goods | |
| `bikri-stock-own-help` | مارکیٹ سفر .gender = masculine میں خریدا ہوا وہ مال جو ابھی بکا نہیں۔ یہ سفر کے شراکت داروں کا ہے۔ اسے بکری .gender = feminine میں بیچیں: لاٹ .gender = masculine کے بیوپاری کی جگہ اپنا مال چنیں۔ | Goods bought on a Market Trip and not sold yet. They belong to the partners of the trip. Sell them in a Sale: choose Own Goods as the seller of a lot. | |
| `bikri-stock-own-label` | سفر کے باقی مال کی فہرست | List of trip goods left | |
| `bikri-stock-own-card-title` | اپنا مال: { $number }، { $item } | Own Goods: { $number }, { $item } | |
| `bikri-stock-own-open` | سفر کھولیں | Open the trip | |
| `bikri-stock-own-total-cost` | کل لاگت | Total cost | |
| `bikri-stock-own-trip` | مارکیٹ سفر .gender = masculine | Market Trip | |

## cashbook

| ID | Urdu | English | Result |
|---|---|---|---|
| `cashbook-title` | روکڑ | Cash Book | |
| `cashbook-help` | دکان کا سارا نقد ایک جگہ: ایپ کے ہر حصے سے آنے اور جانے والی ہر رقم، اور ہر ایک کے بعد ہاتھ میں موجود نقد۔ | All the shop's cash in one place: every amount that came in or went out, from every part of the app, with the cash in hand after each. | |
| `cashbook-new-expense` | نیا دکان خرچہ .gender = masculine | New Shop Expense | |
| `cashbook-filter-from` | از | From | |
| `cashbook-filter-to` | تا | To | |
| `cashbook-opening` | شروع میں نقد | Cash at the start | |
| `cashbook-closing` | آخر میں نقد | Cash at the end | |
| `cashbook-list-label` | روکڑ | Cash Book | |
| `cashbook-empty` | ان دنوں میں کوئی نقد نہیں آیا یا گیا | No cash moved on these days | |
| `cashbook-empty-help` | دوسرے دن چنیں، یا کوئی خرچہ درج کریں۔ | Choose other days, or enter an expense. | |
| `cashbook-load-failed` | روکڑ کھل نہیں سکی | The Cash Book could not be opened | |
| `cashbook-col-date` | تاریخ | Date | |
| `cashbook-col-kind` | اندراج | Entry | |
| `cashbook-col-account` | کھاتہ | Account | |
| `cashbook-col-narration` | تفصیل | Narration | |
| `cashbook-col-in` | آمد | In | |
| `cashbook-col-out` | رخصت | Out | |
| `cashbook-col-running` | بعد میں نقد | Cash after | |
| `cashbook-col-number` | نمبر | No. | |
| `cashbook-col-head` | مد | Head | |
| `cashbook-col-amount` | رقم | Amount | |
| `cashbook-col-state` | حالت | State | |
| `cashbook-col-reason` | وجہ | Reason | |
| `cashbook-col-taken-by` | لینے والا | Taken by | |
| `cashbook-col-return-by` | واپسی کی تاریخ | Return by | |
| `cashbook-col-taken` | لی گئی | Taken | |
| `cashbook-col-returned` | واپس | Returned | |
| `cashbook-col-open` | باقی | Still open | |
| `cashbook-col-days-past` | دن گزرے | Days late | |
| `cashbook-col-lent` | دی گئی | Lent | |
| `cashbook-col-balance` | بقایا | Balance | |
| `cashbook-state-cancelled` | منسوخ | Cancelled | |
| `cashbook-kind-number` | { $kind } { $number } | { $kind } { $number } | |
| `cashbook-kind-bikri-first` | بکری .gender = feminine | Sale | |
| `cashbook-kind-bikri-later` | بعد کی بکری | Later sale | |
| `cashbook-kind-reversal` | الٹا اندراج | Reversal | |
| `cashbook-kind-receipt` | وصولی .gender = feminine | Receipt | |
| `cashbook-kind-payment` | ادائیگی .gender = feminine | Payment | |
| `cashbook-kind-advance` | پیشگی کھاتہ .gender = masculine | Advance | |
| `cashbook-kind-shop-expense` | دکان خرچہ .gender = masculine | Shop Expense | |
| `cashbook-kind-personal-expense` | ذاتی خرچہ .gender = masculine | Personal Drawing | |
| `cashbook-kind-personal-return` | ذاتی خرچہ .gender = masculine واپسی .gender = feminine | Personal Drawing Return | |
| `cashbook-kind-money-lent` | ادھار رقم .gender = feminine | Money Lent | |
| `cashbook-kind-money-lent-return` | ادھار رقم .gender = feminine واپسی .gender = feminine | Money Lent Return | |
| `cashbook-kind-capital-in` | سرمایہ جمع | Capital invested | |
| `cashbook-kind-capital-out` | سرمایہ واپسی | Capital taken back | |
| `cashbook-kind-khata-out` | الگ کھاتہ .gender = masculine: رقم دی | Separate Khata: money paid out | |
| `cashbook-kind-khata-in` | الگ کھاتہ .gender = masculine: رقم آئی | Separate Khata: money received | |
| `cashbook-kind-purchase` | خریداری | Purchase | |
| `cashbook-kind-trip-cost` | سفر خرچ | Trip cost | |
| `cashbook-kind-garden-payment` | باغ کی ادائیگی | Garden payment | |
| `cashbook-kind-venture-receipt` | دوسری دکان میں فروخت | Sale in another shop | |
| `cashbook-kind-garden-close` | باغ بند | Garden closed | |
| `cashbook-kind-fruit-taken` | پھل | Fruit taken | |
| `cashbook-kind-general` | جنرل | Journal | |
| `cashbook-kind-opening` | ابتدائی بیلنس | Opening balance | |
| `cashbook-kind-month-close` | مہینہ بند | Month close | |
| `cashbook-new` | نیا اندراج | New entry | |
| `cashbook-field-date` | تاریخ | Date | |
| `cashbook-field-amount` | رقم | Amount | |
| `cashbook-field-narration` | تفصیل | Narration | |
| `cashbook-field-reason` | وجہ | Reason | |
| `cashbook-field-return-by` | واپسی کی تاریخ | Return by | |
| `cashbook-preview-failed` | اس اندراج کا اثر معلوم نہیں ہو سکا | What this entry will do could not be worked out | |
| `cashbook-balance-now` | اب بقایا | Balance now | |
| `cashbook-balance-after` | اس اندراج کے بعد بقایا | Balance after this entry | |
| `cashbook-saved` | { $kind -> [shop_expense] دکان خرچہ .gender = masculine { $number } محفوظ ہو گیا [personal_expense] ذاتی خرچہ .gender = masculine { $number } محفوظ ہو گیا [personal_return] ذاتی خرچہ .gender = masculine واپسی .gender = feminine { $number } محفوظ ہو گئی [money_lent] ادھار رقم .gender = feminine { $number } محفوظ ہو گئی [money_lent_return] ادھار رقم .gender = feminine واپسی .gender = feminine { $number } محفوظ ہو گئی [khata_out] الگ کھاتہ .gender = masculine: رقم دی { $number } محفوظ ہو گئی [general] جنرل { $number } محفوظ ہو گیا [capital_in] سرمایہ جمع { $number } محفوظ ہو گیا [capital_out] سرمایہ واپسی { $number } محفوظ ہو گئی *[khata_in] الگ کھاتہ .gender = masculine: رقم آئی { $number } محفوظ ہو گئی | { $kind -> [shop_expense] Shop Expense { $number } saved [personal_expense] Personal Drawing { $number } saved [personal_return] Personal Drawing Return { $number } saved [money_lent] Money Lent { $number } saved [money_lent_return] Money Lent Return { $number } saved [khata_out] Separate Khata: money paid out { $number } saved [general] Journal { $number } saved [capital_in] Capital put in { $number } saved [capital_out] Capital taken back { $number } saved *[khata_in] Separate Khata: money received { $number } saved | |
| `cashbook-type-shop-expense` | دکان خرچہ .gender = masculine | Shop Expense | |
| `cashbook-type-personal-expense` | ذاتی خرچہ .gender = masculine | Personal Drawing | |
| `cashbook-type-personal-return` | ذاتی خرچہ .gender = masculine واپسی .gender = feminine | Personal Drawing Return | |
| `cashbook-type-money-lent` | رقم دی | Money lent | |
| `cashbook-type-money-lent-return` | واپسی .gender = feminine | Return | |
| `cashbook-type-khata-out` | رقم دی | Paid out | |
| `cashbook-type-khata-in` | رقم آئی | Received | |
| `cashbook-tab-expense` | خرچہ .gender = masculine | Expense | |
| `cashbook-tab-fruit` | پھل | Fruit Taken | |
| `cashbook-tabs-label` | خرچہ .gender = masculine کے حصے | Sections of Expense | |
| `cashbook-expense-title` | خرچہ .gender = masculine | Expense | |
| `cashbook-expense-help` | پہلے قسم چنیں۔ دکان خرچہ .gender = masculine شراکت داروں میں ان کے حصوں کے مطابق بٹتا ہے۔ ذاتی خرچہ .gender = masculine صرف ایک شراکت دار کے کھاتے میں جاتا ہے۔ فارم کے نیچے نظر آتا ہے کہ محفوظ کرنے سے کیا ہوگا۔ | Pick the type first. A Shop Expense is split between the partners by their shares. A Personal Drawing is charged to one partner only. Below the form you see what saving will do. | |
| `cashbook-expense-type` | خرچہ .gender = masculine کی قسم | Type of Expense | |
| `cashbook-field-head` | خرچہ مد | Expense head | |
| `cashbook-field-partner` | شراکت دار | Partner | |
| `cashbook-field-paid-to` | ادا کیا | Paid to | |
| `cashbook-field-taken-by` | لینے والا | Taken by | |
| `cashbook-heads-failed` | خرچہ مدیں کھل نہیں سکیں | The expense heads could not be opened | |
| `cashbook-partners-failed` | شراکت دار کھل نہیں سکے | The partners could not be opened | |
| `cashbook-preview-hint-shop` | رقم لکھیں اور مد چنیں۔ محفوظ کرنے سے پہلے یہاں ہر شراکت دار کا حصہ نظر آئے گا۔ | Type the amount and choose the head. Each partner's share shows here before you save. | |
| `cashbook-preview-hint-personal` | شراکت دار چنیں اور رقم لکھیں۔ محفوظ کرنے سے پہلے یہاں اس کی کل رقم اب اور بعد میں نظر آئے گی۔ | Choose the partner and type the amount. His total now and after shows here before you save. | |
| `cashbook-shares-title` | ہر شراکت دار کا حصہ | Each partner's share | |
| `cashbook-shares-none` | اس تاریخ پر بانٹنے کے لیے کوئی حصے نہیں ہیں۔ | There are no shares to split this by on this date. | |
| `cashbook-effect-title` | شراکت دار پر اس کا اثر | What this does to the partner | |
| `cashbook-effect-none` | اثر دیکھنے کے لیے شراکت دار چنیں۔ | Choose the partner to see the effect. | |
| `cashbook-effect-total-now` | کل رقم اب | Partner total now | |
| `cashbook-effect-total-after` | کل رقم بعد میں | Partner total after | |
| `cashbook-effect-open-now` | لی گئی اور واپس نہ ہوئی، اب | Taken and not returned now | |
| `cashbook-effect-open-after` | لی گئی اور واپس نہ ہوئی، بعد میں | Taken and not returned after | |
| `cashbook-others-not-affected` | باقی شراکت داروں پر کوئی اثر نہیں پڑتا۔ | The other partners are not affected. | |
| `cashbook-personal-hint` | کیا یہ شراکت دار کا ذاتی خرچہ تو نہیں؟ { $name } شراکت دار ہے۔ | Is this a partner's own spending? { $name } is a partner. | |
| `cashbook-personal-make` | ذاتی خرچہ .gender = masculine بنائیں | Make it Personal Drawing | |
| `cashbook-repeat-open` | پچھلے مہینے والے دہرائیں | Repeat last month | |
| `cashbook-repeat-title` | پچھلے مہینے کے باقاعدہ خرچے | Last month's regular expenses | |
| `cashbook-repeat-help` | یہ مدیں پچھلے مہینے درج ہوئیں اور اس مہینے ابھی درج نہیں ہوئیں۔ ایک کو فارم میں بھریں، دیکھ لیں اور محفوظ کریں۔ | These heads were entered last month and not yet this month. Fill one into the form, check it and save it. | |
| `cashbook-repeat-fill` | فارم میں بھریں | Fill the form | |
| `cashbook-repeat-last-date` | پچھلی بار: <num>{ $date }</num> | Last month: <num>{ $date }</num> | |
| `cashbook-repeat-total` | ان کا میزان | Total of these | |
| `cashbook-repeat-close` | بند کریں | Close | |
| `cashbook-repeat-empty` | دہرانے کو کچھ نہیں | Nothing to repeat | |
| `cashbook-repeat-empty-help` | پچھلے مہینے کی ہر مد اس مہینے درج ہو چکی ہے، یا کوئی مد تھی ہی نہیں۔ | Every head from last month is already entered this month, or there was none. | |
| `cashbook-repeat-failed` | پچھلے مہینے کے خرچے کھل نہیں سکے | Last month's expenses could not be opened | |
| `cashbook-day-title` | اس دن کے دکان خرچے | Shop expenses of the day | |
| `cashbook-day-label` | اس دن کے دکان خرچے | Shop expenses of the day | |
| `cashbook-day-empty` | اس دن کوئی دکان خرچہ نہیں | No shop expense on this day | |
| `cashbook-day-empty-help` | اوپر فارم بھر کر محفوظ کریں تو یہاں آ جائے گا۔ | Fill the form above and save to add one. | |
| `cashbook-day-failed` | اس دن کے دکان خرچے کھل نہیں سکے | The shop expenses of the day could not be opened | |
| `cashbook-takes-title` | شراکت دار کی لی ہوئی رقم | Money the partner took | |
| `cashbook-takes-label` | شراکت دار کی لی ہوئی رقم | Money the partner took | |
| `cashbook-takes-empty` | اس شراکت دار نے کوئی رقم نہیں لی | This partner has taken no money | |
| `cashbook-takes-empty-help` | جو رقم وہ اپنے لیے لے اور جو واپس کرے وہ یہاں لکھی جاتی ہے۔ | What he takes for himself and what he gives back is listed here. | |
| `cashbook-takes-failed` | شراکت دار کی لی ہوئی رقم کھل نہیں سکی | The partner's money taken could not be opened | |
| `cashbook-takes-pick` | شراکت دار چنیں | Choose a partner | |
| `cashbook-takes-pick-help` | اس کی لی ہوئی اور واپس کی ہوئی رقم یہاں نظر آئے گی۔ | His money taken and given back shows here. | |
| `cashbook-take-opening` | ابتدائی بیلنس | Opening balance | |
| `cashbook-lent-screen-title` | ادھار رقم .gender = feminine | Money Lent | |
| `cashbook-lent-screen-help` | وہ رقم جو دکان عملے یا کسی اور کو ادھار دے، اور جو وہ واپس کریں۔ واپسی نقد ہو سکتی ہے یا تنخواہ سے کٹوتی۔ نیا شخص شامل کرنے کے لیے نام کے خانے میں F4 دبائیں۔ | Money the shop lends to staff or anyone else, and what they give back. A return can be in cash or taken from his salary. Press F4 in the person field to add a new person. | |
| `cashbook-lent-type` | اندراج کی قسم | Type of entry | |
| `cashbook-field-person` | نام | Person | |
| `cashbook-field-mode` | واپسی کیسے | How it is returned | |
| `cashbook-mode-cash` | نقد | In cash | |
| `cashbook-mode-salary` | تنخواہ سے کٹوتی | Deducted from salary | |
| `cashbook-field-salary-head` | تنخواہ کی خرچہ مد | Expense head of the salary | |
| `cashbook-field-salary-head-hint` | خالی چھوڑیں تو معمول کی تنخواہ والی مد لگے گی۔ | Left empty, the default salary head is used. | |
| `cashbook-salary-head-default` | تنخواہ (معمول) | Salary (default) | |
| `cashbook-salary-note` | کوئی نقد نہیں ہلتا۔ تنخواہ کا یہ حصہ دکان خرچہ گنا جاتا ہے اور شراکت داروں میں بٹتا ہے۔ | No cash moves. That part of the salary counts as a shop expense and is split between the partners. | |
| `cashbook-lent-title` | اس شخص کی ادھار رقمیں | The person's loans | |
| `cashbook-lent-label` | اس شخص کی ادھار رقمیں | The person's loans | |
| `cashbook-lent-balance` | اس کے ذمے باقی | He still owes | |
| `cashbook-lent-empty` | اس شخص کی کوئی ادھار رقم نہیں | This person has no loan | |
| `cashbook-lent-empty-help` | اسے دی گئی رقم اور اس کی واپسی یہاں لکھی جائے گی۔ | Money lent to him and what he gives back will be listed here. | |
| `cashbook-lent-failed` | اس شخص کی ادھار رقمیں کھل نہیں سکیں | The person's loans could not be opened | |
| `cashbook-lent-pick` | شخص چنیں | Choose a person | |
| `cashbook-lent-pick-help` | اس کی ادھار رقمیں، واپسی اور باقی رقم یہاں نظر آئے گی۔ | His loans, what he returned and what is still open show here. | |
| `cashbook-khata-screen-title` | الگ کھاتہ .gender = masculine | Separate Khata | |
| `cashbook-khata-screen-help` | وہ رقم جس کا شراکت داروں، کمیشن یا منافع سے کوئی تعلق نہیں، مثلاً خاندانی کھاتہ۔ رقم دینے سے کھاتہ منفی ہوتا ہے، رقم آنے سے مثبت۔ نیا کھاتہ شامل کرنے کے لیے کھاتے کے خانے میں F4 دبائیں۔ | Money that has nothing to do with the partners, commission or profit, for example a family khata. Money paid out makes it minus; money received makes it plus. Press F4 in the khata field to add a new khata. | |
| `cashbook-khata-type` | اندراج کی قسم | Type of entry | |
| `cashbook-field-khata` | الگ کھاتہ .gender = masculine | Separate Khata | |
| `cashbook-khata-title` | کھاتے کے اندراجات | The khata's entries | |
| `cashbook-khata-label` | کھاتے کے اندراجات | The khata's entries | |
| `cashbook-khata-opening` | شروع میں بقایا | Balance at the start | |
| `cashbook-khata-closing` | اب بقایا | Balance now | |
| `cashbook-khata-sign-hint` | مثبت: دکان کے پاس اس کھاتے کی رقم ہے۔ منفی: اس نے لایا کم اور لیا زیادہ۔ اندراج کے بعد کے بقایا میں جمع مثبت ہے اور بنام منفی۔ | Plus: the shop holds money for this khata. Minus: it has taken more than it brought. On the balance after an entry, Credit is plus and Debit is minus. | |
| `cashbook-khata-empty` | اس کھاتے میں کوئی اندراج نہیں | This khata has no entry | |
| `cashbook-khata-empty-help` | دی گئی اور آئی ہوئی رقم وجہ کے ساتھ یہاں لکھی جائے گی۔ | Money paid out and received will be listed here, with the reason. | |
| `cashbook-khata-failed` | کھاتے کے اندراجات کھل نہیں سکے | The khata's entries could not be opened | |
| `cashbook-khata-pick` | کھاتہ چنیں | Choose a khata | |
| `cashbook-khata-pick-help` | اس کے اندراجات اور چلتا بقایا یہاں نظر آئے گا۔ | Its entries and its running balance show here. | |
| `cashbook-khata-kind-paid-out` | رقم دی | Paid out | |
| `cashbook-khata-kind-received` | رقم آئی | Received | |
| `cashbook-khata-kind-sale` | مال کی فروخت | Goods sold | |
| `cashbook-add-person-title` | نیا شخص | New person | |
| `cashbook-add-person-help` | ایسا شخص شامل کریں جو دکان سے ادھار رقم لے سکے۔ عملے کے فرد کو اس کے عملے کے ریکارڈ سے جوڑا جا سکتا ہے۔ | Add a person who can borrow money from the shop. Staff can be linked to their staff record. | |
| `cashbook-add-person-saved` | شخص شامل ہو گیا | Person added | |
| `cashbook-add-khata-title` | نیا الگ کھاتہ .gender = masculine | New Separate Khata | |
| `cashbook-add-khata-help` | ایسا کھاتہ شامل کریں جو شراکت داروں، کمیشن اور منافع سے الگ رکھا جائے۔ | Add a khata that is kept apart from the partners, commission and profit. | |
| `cashbook-add-khata-saved` | کھاتہ شامل ہو گیا | Khata added | |
| `cashbook-add-name-ur` | اردو نام | Urdu name | |
| `cashbook-add-name-en` | انگریزی نام | English name | |
| `cashbook-add-phone` | فون | Phone | |
| `cashbook-add-staff` | عملے کا ریکارڈ | Staff record | |
| `cashbook-add-staff-hint` | صرف اگر وہ دکان پر کام کرتا ہو۔ | Only if he works for the shop. | |
| `cashbook-add-note` | نوٹ | Note | |
| `cashbook-general-screen-title` | جنرل | Journal | |
| `cashbook-general-screen-help` | سطروں کا اندراج، ہر سطر پر کھاتہ اور بنام یا جمع کی طرف رقم۔ دونوں طرف برابر ہوں تو ہی محفوظ ہو سکتا ہے۔ | An entry of lines, each with an account and an amount on the Debit or the Credit side. It can be saved only when the two sides are equal. | |
| `cashbook-general-lines-title` | سطریں | Lines | |
| `cashbook-general-grid-label` | جنرل اندراج کی سطریں | Lines of the Journal entry | |
| `cashbook-general-add-line` | سطر شامل کریں | Add line | |
| `cashbook-general-col-account` | کھاتہ | Account | |
| `cashbook-general-col-person` | شخص | Person | |
| `cashbook-general-totals-title` | میزان | Total | |
| `cashbook-general-totals-hint` | سطریں لکھیں۔ میزان یہاں نظر آئے گی۔ | Type the lines. The totals show here. | |
| `cashbook-general-total-dr` | بنام کی میزان | Debit total | |
| `cashbook-general-total-cr` | جمع کی میزان | Credit total | |
| `cashbook-general-difference` | فرق (بنام منفی جمع) | Difference (Debit less Credit) | |
| `cashbook-general-balanced` | دونوں طرف برابر ہیں۔ اندراج محفوظ ہو سکتا ہے۔ | The two sides are equal. The entry can be saved. | |
| `cashbook-general-not-balanced` | دونوں طرف ابھی برابر نہیں ہیں۔ برابر ہونے تک اندراج محفوظ نہیں ہو سکتا۔ | The two sides are not equal yet. The entry cannot be saved until they are. | |
| `cashbook-general-split-title` | ہر شراکت دار کا حصہ | Each Partner's part | |
| `cashbook-general-split-help` | آمدن اور دکان خرچہ .gender = masculine کے کھاتوں کی سطریں شراکت داروں میں ان کے حصہ .gender = masculine کے مطابق بٹتی ہیں۔ | Lines on income and Shop Expense accounts are split between the partners by their shares. | |
| `cashbook-general-people-title` | ایک بار آنے والے لوگ | One-time people | |
| `cashbook-general-people-help` | پختہ کھاتہ .gender = masculine کھاتے کی رقم ہر شخص کے الگ حساب سے رکھی جاتی ہے۔ ان میں سے ہر سطر کے لیے شخص چنیں۔ | Money under a Pukhta Khata account is kept for each person. Choose the person for each of these lines. | |
| `cashbook-general-person-of` | سطر { $line } کا شخص: { $account } | Person for line { $line }: { $account } | |
| `cashbook-type-capital-in` | سرمایہ جمع | Capital put in | |
| `cashbook-type-capital-out` | سرمایہ واپسی | Capital taken back | |

## common

| ID | Urdu | English | Result |
|---|---|---|---|
| `common-app-name` | منڈی سافٹ ویئر | Mandi Software | |
| `common-app-tagline` | آڑھتی .gender = masculine کا حساب کتاب | Accounts for the Commission Agent (Arhti) | |
| `common-version` | ورژن <code>{ $version }</code> | Version <code>{ $version }</code> | |
| `common-loading` | ذرا انتظار کریں… | Please wait… | |
| `common-retry` | دوبارہ کوشش کریں | Try again | |
| `common-language-switch` | زبان بدلیں | Change language | |
| `common-language-hint` | زبان بدلنے کے لیے { $key } دبائیں۔ | Press { $key } to change the language. | |
| `common-back` | واپس | Back | |
| `common-select-placeholder` | چنیں | Choose | |
| `common-notifications` | اطلاعات | Notifications | |
| `common-data-check-title` | ایپ آپ کا ڈیٹا ابھی نہیں کھول سکی | The app cannot open your data yet | |
| `common-password-show` | پاس ورڈ دکھائیں | Show password | |
| `common-password-hide` | پاس ورڈ چھپائیں | Hide password | |
| `common-side-dr` | بنام | Debit | |
| `common-side-cr` | جمع | Credit | |
| `common-yes` | ہاں | Yes | |
| `common-no` | نہیں | No | |
| `common-percent-sign` | % | % | |
| `common-cash-confirm-title` | روکڑ صفر سے کم ہو جائے گی | Cash in hand will go below zero | |
| `common-cash-confirm-text` | اس اندراج کے بعد روکڑ <num>{ $amount }</num> ہو جائے گی۔ کیا یہ اندراج درست ہے؟ | After this entry cash in hand will be <num>{ $amount }</num>. Is this entry correct? | |
| `common-cash-after` | اس اندراج کے بعد روکڑ <num>{ $amount }</num> ہو جائے گی۔ | After this entry cash in hand will be <num>{ $amount }</num>. | |
| `common-cash-confirm-yes` | ہاں، محفوظ کریں | Yes, save | |
| `common-cash-confirm-no` | نہیں، واپس جائیں | No, go back | |
| `common-more-columns` | مزید کالم | More columns | |
| `common-list-count` | { $total } میں سے { $shown } نظر آ رہے ہیں | Showing { $shown } of { $total } | |
| `common-list-more` | مزید دکھائیں | Show more | |

## dashboard

| ID | Urdu | English | Result |
|---|---|---|---|
| `dashboard-greeting` | السلام علیکم، <name>{ $name }</name> | Hello, <name>{ $name }</name> | |
| `dashboard-today` | آج: <num>{ $date }</num> | Today: <num>{ $date }</num> | |
| `dashboard-new-bikri` | نئی بکری .gender = feminine | New Sale | |
| `dashboard-load-failed` | ڈیش بورڈ کھل نہیں سکا | The dashboard could not be opened | |
| `dashboard-quick-title` | فوری کام | Quick actions | |
| `dashboard-new-receipt` | نئی وصولی .gender = feminine | New Receipt | |
| `dashboard-new-payment` | نئی ادائیگی .gender = feminine | New Payment | |
| `dashboard-new-advance` | نئی پیشگی .gender = feminine | New Advance | |
| `dashboard-new-expense` | نیا خرچہ .gender = masculine | New Expense | |
| `dashboard-open-daybook` | روزنامچہ .gender = masculine | Day Book | |
| `dashboard-tiles-title` | آج | Today | |
| `dashboard-tile-sales` | آج کی بکری .gender = feminine | Today's Sale | |
| `dashboard-tile-sales-count` | <num>{ $count }</num> بکری .gender = feminine | { $count -> [one] <num>{ $count }</num> Sale *[other] <num>{ $count }</num> Sales | |
| `dashboard-tile-commission` | آج کا کمیشن .gender = masculine | Today's Commission | |
| `dashboard-tile-cash` | روکڑ بقایا | Cash in hand | |
| `dashboard-tile-cash-below` | صفر سے کم | Below zero | |
| `dashboard-tile-dues` | وصولی باقی | Buyers' dues | |
| `dashboard-tile-payable` | ادائیگی باقی | Sellers payable | |
| `dashboard-tile-advances` | پیشگی باقی | Advances open | |
| `dashboard-tile-stock` | باقی مال | Unsold Stock | |
| `dashboard-tile-stock-note` | <num>{ $lots }</num> لاٹ .gender = masculine | { $lots -> [one] <num>{ $lots }</num> Lot *[other] <num>{ $lots }</num> Lots | |
| `dashboard-figure-nag` | <num>{ $qty }</num> <unit>نگ</unit> | <num>{ $qty }</num> <unit>Nag</unit> | |
| `dashboard-tile-trips` | مارکیٹ سفر .gender = masculine | Market Trips | |
| `dashboard-tile-trips-note` | <num>{ $open }</num> کھلے، باقی لاگت <money></money> | <num>{ $open }</num> open, cost left <money></money> | |
| `dashboard-tile-gardens` | باغات | Gardens | |
| `dashboard-tile-gardens-note` | <num>{ $open }</num> کھلے | <num>{ $open }</num> open | |
| `dashboard-tile-gardens-none` | کوئی باغ کھلا نہیں | No open garden | |
| `dashboard-gardens-title` | کھلے باغات | Open Gardens | |
| `dashboard-gardens-col-garden` | باغ | Garden | |
| `dashboard-gardens-col-balance` | چلتا بقایا | Running balance | |
| `dashboard-gardens-col-rent-due` | ٹھیکہ باقی | Rent due | |
| `dashboard-gardens-all` | تمام باغات | All Gardens | |
| `dashboard-tile-lent` | ادھار رقم .gender = feminine باقی | Money Lent open | |
| `dashboard-tile-lent-overdue` | <money></money> کی واپسی کی تاریخ گزر چکی ہے | <money></money> past its return-by date | |
| `dashboard-tile-expenses` | آج کا دکان خرچہ .gender = masculine | Today's Shop Expense | |
| `dashboard-tile-backup` | آخری بیک اپ | Last backup | |
| `dashboard-tile-health` | حساب کی جانچ | Books check | |
| `dashboard-tile-checking` | دیکھ رہے ہیں… | Checking… | |
| `dashboard-tile-unknown` | پڑھا نہیں جا سکا | Could not be read | |
| `dashboard-health-ok` | سب ٹھیک ہے | All good | |
| `dashboard-health-bad` | مسئلہ ملا ہے | A problem was found | |
| `dashboard-health-never` | ابھی جانچ نہیں ہوئی | Not checked yet | |
| `dashboard-health-when` | جانچ: <num>{ $when }</num> | Checked <num>{ $when }</num> | |
| `dashboard-partners-title` | شراکت دار | Partners | |
| `dashboard-partner-open` | { $name } کا فرد حساب | Statement of { $name } | |
| `dashboard-partner-today` | آج | Today | |
| `dashboard-partner-month` | اس مہینے | This month | |
| `dashboard-partner-year` | اس سال | This year | |
| `dashboard-partner-total` | کل رقم | Partner Total | |
| `dashboard-partners-empty-title` | ابھی کوئی شراکت دار نہیں | No partner yet | |
| `dashboard-partners-empty-help` | شراکت دار اور ان کے حصے لکھیں، پھر ہر ایک کا حصہ یہاں نظر آئے گا۔ | Add the partners and their shares, and each one's share shows here. | |
| `dashboard-partners-setup` | شراکت دار ترتیب دیں | Set up the partners | |
| `dashboard-bikri-title` | آج کی بکری .gender = feminine | Today's Sale | |
| `dashboard-bikri-col-number` | بکری .gender = feminine | Sale | |
| `dashboard-bikri-col-truck` | ٹرک | Truck | |
| `dashboard-bikri-col-seller` | بیوپاری .gender = masculine | Seller | |
| `dashboard-bikri-col-gross` | خام بکری .gender = feminine | Gross Sale | |
| `dashboard-bikri-col-net` | صافی بکری .gender = feminine | Net Sale | |
| `dashboard-bikri-col-state` | حالت | Status | |
| `dashboard-bikri-number` | بکری .gender = feminine <num>{ $number }</num> | Sale <num>{ $number }</num> | |
| `dashboard-bikri-sellers` | <name>{ $name }</name> اور <num>{ $more }</num> مزید | <name>{ $name }</name> and <num>{ $more }</num> more | |
| `dashboard-bikri-cancelled` | منسوخ | Cancelled | |
| `dashboard-bikri-showing` | <num>{ $count }</num> میں سے <num>{ $shown }</num> دکھائی گئیں | Showing <num>{ $shown }</num> of <num>{ $count }</num> | |
| `dashboard-bikri-all` | آج کی ساری بکری .gender = feminine | All of today's Sale | |
| `dashboard-bikri-empty-title` | آج کوئی بکری .gender = feminine نہیں | No Sale today | |
| `dashboard-bikri-empty-help` | نئی بکری .gender = feminine کے لیے <key>{ $key }</key> دبائیں۔ | Press <key>{ $key }</key> to enter the first one. | |
| `dashboard-dues-title` | سب سے پرانے بقایا | Oldest buyer dues | |
| `dashboard-dues-col-party` | گاہک .gender = masculine | Buyer | |
| `dashboard-dues-col-city` | شہر | City | |
| `dashboard-dues-col-balance` | بقایا | Balance | |
| `dashboard-dues-col-days` | دن | Days | |
| `dashboard-dues-all` | تمام گاہکوں کے بقایا | All buyer dues | |
| `dashboard-dues-empty-title` | کسی گاہک کے ذمے کچھ نہیں | No buyer owes anything | |
| `dashboard-dues-empty-help` | بقایا یہاں نظر آئیں گے، سب سے پرانا سب سے اوپر۔ | Dues show here, the oldest first. | |
| `dashboard-tasks-title` | آج کے کام | Today's tasks | |
| `dashboard-tasks-empty-title` | کسی کام پر توجہ کی ضرورت نہیں | Nothing needs your attention | |
| `dashboard-tasks-empty-help` | جب کوئی کام باقی ہو گا تو یہاں نظر آئے گا۔ | When something needs doing, it shows here. | |
| `dashboard-charts-title` | چارٹ | Charts | |
| `dashboard-chart-days-title` | پچھلے <num>30</num> دن | Last <num>30</num> days | |
| `dashboard-chart-days-label` | پچھلے 30 دنوں میں ہر دن کی خام بکری .gender = feminine اور کمیشن .gender = masculine | Gross Sale and Commission of each of the last 30 days | |
| `dashboard-chart-items-title` | اس مہینے کا کمیشن .gender = masculine بلحاظ قسم مال | This month's Commission by item | |
| `dashboard-chart-items-label` | اس مہینے کی سب سے اوپر قسم مال کا کمیشن .gender = masculine | Commission of the top items this month | |
| `dashboard-chart-gross` | خام بکری .gender = feminine | Gross Sale | |
| `dashboard-chart-commission` | کمیشن .gender = masculine | Commission | |
| `dashboard-chart-empty-title` | اس مدت میں ابھی کچھ نہیں بکا | Nothing sold in this period yet | |
| `dashboard-chart-empty-help` | بکری .gender = feminine محفوظ ہوتے ہی چارٹ بھرتا جائے گا۔ | The chart fills in as sales are saved. | |
| `dashboard-update-note` | نیا ورژن <num>{ $version }</num> تیار ہے۔ دن ختم کر کے ترتیبات سے لگائیں۔ | A new version <num>{ $version }</num> is ready. Install it from Settings when the day is done. | |
| `dashboard-update-open` | ترتیبات کھولیں | Open Settings | |
| `soon-title` | یہ حصہ جلد آ رہا ہے | This part is coming soon | |
| `soon-help` | اس کی تیاری ہو رہی ہے۔ ڈیش بورڈ پر واپس جائیں۔ | It is being built. Go back to the dashboard. | |
| `soon-back` | ڈیش بورڈ پر جائیں | Go to the dashboard | |

## day

| ID | Urdu | English | Result |
|---|---|---|---|
| `day-title` | دن بند کریں | Close the day | |
| `day-today` | آج کی تاریخ | Today's date | |
| `day-closed-up-to` | بند ہونے کی آخری تاریخ | Closed up to | |
| `day-none-closed` | ابھی کوئی دن بند نہیں | No day is closed yet | |
| `day-load-failed` | دن کی حالت کھل نہیں سکی | The state of the day could not be opened | |
| `day-close-title` | دن بند کریں | Close the day | |
| `day-close-help` | دن بند ہونے کے بعد اس دن کے اندراجات اس وقت تک نہیں بدل سکتے جب تک مالک وہ دن دوبارہ نہ کھولے۔ | After a day is closed, its entries cannot be changed unless the Owner reopens that day. | |
| `day-close-up-to` | اس تاریخ تک بند کریں | Close up to this date | |
| `day-close-days` | بند ہونے والے دن | Days that will close | |
| `day-close-from` | پہلا کھلا دن | First open day | |
| `day-close-entries` | ان دنوں کے اندراجات | Entries in these days | |
| `day-close-trial-dr` | ٹرائل بیلنس .gender = masculine: بنام میزان | Trial Balance: total Debit | |
| `day-close-trial-cr` | ٹرائل بیلنس .gender = masculine: جمع میزان | Trial Balance: total Credit | |
| `day-close-trial` | ٹرائل بیلنس .gender = masculine کا فرق | Difference in the Trial Balance | |
| `day-close-cash` | روکڑ بقایا | Cash Book balance | |
| `day-close-nothing` | اس تاریخ تک کوئی دن کھلا نہیں۔ | No day is open up to this date. | |
| `day-close-button` | دن بند کریں | Close the day | |
| `day-close-preview-failed` | دن بند کرنے کا حساب کھل نہیں سکا | What closing would do could not be opened | |
| `day-close-confirm-title` | دن بند کریں؟ | Close the day? | |
| `day-close-confirm-description` | { $days -> [one] { $date } تک کا { $days } دن بند ہو جائے گا۔ اس دن کے اندراجات اس وقت تک نہیں بدل سکیں گے جب تک آپ وہ دن دوبارہ نہ کھولیں۔ *[other] { $date } تک کے { $days } دن بند ہو جائیں گے۔ ان دنوں کے اندراجات اس وقت تک نہیں بدل سکیں گے جب تک آپ وہ دن دوبارہ نہ کھولیں۔ | { $days -> [one] { $days } day up to { $date } will be closed. Its entries cannot be changed until you reopen that day. *[other] { $days } days up to { $date } will be closed. Their entries cannot be changed until you reopen those days. | |
| `day-closed` | { $date } تک کے دن بند ہو گئے | Days up to { $date } are closed | |
| `day-reopen-title` | دن دوبارہ کھولیں | Reopen a day | |
| `day-reopen-help` | ایک تاریخ کھولنے سے اس کے بعد کی ہر تاریخ بھی کھل جاتی ہے۔ وجہ لکھنا ضروری ہے اور وہ آڈٹ لاگ میں لکھی جاتی ہے۔ | Reopening a date reopens every later date too. A reason is required and is written to the audit log. | |
| `day-reopen-date` | کھولنے کی تاریخ | Date to reopen | |
| `day-reopen-reason` | وجہ | Reason | |
| `day-reopen-button` | دوبارہ کھولیں | Reopen | |
| `day-reopen-confirm-title` | دن دوبارہ کھولیں؟ | Reopen the day? | |
| `day-reopen-confirm-description` | { $date } اور اس کے بعد کی ہر تاریخ کھل جائے گی۔ وجہ آڈٹ لاگ میں لکھی جائے گی۔ | { $date } and every later date will be reopened. The reason will be written to the audit log. | |
| `day-reopened` | { $date } اور اس کے بعد کے دن کھل گئے | { $date } and the days after it are reopened | |
| `day-reopen-none-title` | ابھی کوئی دن بند نہیں | No day is closed yet | |
| `day-reopen-none-help` | جب آپ کوئی دن بند کریں گے تو اسے یہاں دوبارہ کھول سکیں گے۔ | When you close a day, you can reopen it here. | |
| `day-summary-title` | ان دنوں کا حاصل | What these days came to | |
| `day-summary-bikri-count` | بکری .gender = feminine کی تعداد | Number of Sale | |
| `day-summary-gross` | خام بکری .gender = feminine | Gross Sale | |
| `day-summary-commission` | کمیشن .gender = masculine اور لاگا .gender = masculine، دونوں طرف | Commission and Laga, both sides | |
| `day-summary-shared-income` | مشترکہ آمدن | Shared income | |
| `day-summary-open-lots` | باقی مال میں لاٹ | Lots in Unsold Stock | |
| `day-summary-open-qty` | باقی مال میں نگ | Nag in Unsold Stock | |
| `day-summary-stock-left-note` | { $lots -> [one] <num>{ $lots }</num> لاٹ کے <num>{ $qty }</num> نگ ابھی باقی مال میں ہیں۔ دن پھر بھی بند ہو سکتا ہے، یہ مال بعد میں بک سکتا ہے۔ *[other] <num>{ $lots }</num> لاٹ کے <num>{ $qty }</num> نگ ابھی باقی مال میں ہیں۔ دن پھر بھی بند ہو سکتا ہے، یہ مال بعد میں بک سکتا ہے۔ | { $lots -> [one] <num>{ $lots }</num> lot with <num>{ $qty }</num> Nag is still in Unsold Stock. The day can still be closed; the goods can be sold later. *[other] <num>{ $lots }</num> lots with <num>{ $qty }</num> Nag are still in Unsold Stock. The day can still be closed; the goods can be sold later. | |
| `day-summary-no-partners` | دکھانے کے لیے شراکت دار کا کوئی حصہ نہیں۔ | There are no partner shares to show. | |
| `day-summary-partners-label` | ان دنوں میں ہر شراکت دار کا حصہ | Each partner's share of these days | |
| `day-summary-col-partner` | شراکت دار | Partner | |
| `day-summary-col-income` | آمدن کا حصہ .gender = masculine | Income Share | |
| `day-summary-col-expense` | خرچہ .gender = masculine کا حصہ .gender = masculine | Expense Share | |
| `day-summary-col-profit` | منافع کا حصہ .gender = masculine | Profit Share | |

## errors

| ID | Urdu | English | Result |
|---|---|---|---|
| `err-not-implemented` | یہ حصہ ابھی تیار نہیں۔ یہ اگلے ورژن میں آئے گا۔ | This part is not ready yet. It comes in a later version. | |
| `err-internal` | کچھ غلط ہو گیا۔ دوبارہ کوشش کریں۔ | Something went wrong. Please try again. | |
| `err-reference` | مدد کے لیے حوالہ نمبر <code>{ $ref }</code> بتائیں۔ | For help, give the reference number <code>{ $ref }</code>. | |
| `err-required` | یہ خانہ خالی ہے۔ اسے بھریں۔ | This field is empty. Please fill it in. | |
| `err-not-a-number` | یہاں نمبر لکھیں، مثلاً 12,000۔ حروف اور غلط جگہ پر لگا ہوا کاما نہیں لکھ سکتے۔ | Enter a number, for example 12,000. Letters and misplaced commas cannot be used. | |
| `err-decimals-not-allowed` | رقم پورے روپوں میں لکھیں۔ | Enter the amount in whole rupees. | |
| `err-qty-not-whole` | تعداد پورے نگ میں لکھیں۔ | Write the quantity in whole Nag. | |
| `err-negative-not-allowed` | منفی نمبر نہیں لکھ سکتے۔ نمبر منفی نشان کے بغیر لکھیں۔ | A negative number is not allowed. Write it without a minus sign. | |
| `err-amount-too-large` | رقم بہت زیادہ ہے۔ زیادہ سے زیادہ { $max } روپے لکھ سکتے ہیں۔ | The amount is too large. The most you can enter is Rs { $max }. | |
| `err-entry-too-large` | یہ اندراج بہت بڑا ہے۔ ایک سطر، یا پورے اندراج کا میزان، زیادہ سے زیادہ { $max } روپے ہو سکتا ہے۔ | This entry is too large. One line, or the total of an entry, can be at most Rs { $max }. | |
| `err-must-be-more-than-zero` | صفر سے بڑا نمبر لکھیں۔ | Enter a number more than zero. | |
| `err-too-many-decimals` | اعشاریہ کے بعد زیادہ سے زیادہ { $max } ہندسے لکھیں۔ | Enter at most { $max } digits after the decimal point. | |
| `err-percent-over-100` | فیصد 100 سے زیادہ نہیں ہو سکتا۔ | A percent cannot be more than 100. | |
| `err-qty-too-large` | تعداد بہت زیادہ ہے۔ زیادہ سے زیادہ { $max } نگ لکھ سکتے ہیں۔ | The quantity is too large. The most you can enter is { $max } Nag. | |
| `err-weight-too-large` | وزن بہت زیادہ ہے۔ زیادہ سے زیادہ { $max } کلو لکھ سکتے ہیں۔ | The weight is too large. The most you can enter is { $max } kg. | |
| `err-date-invalid` | یہ تاریخ درست نہیں۔ دن، مہینہ اور سال درست لکھیں۔ | This date is not valid. Enter the day, month and year correctly. | |
| `err-date-closed` | یہ دن بند ہو چکا ہے۔ { $closed_up_to } کے بعد کی تاریخ لکھیں، یا مالک سے دن کھلوائیں۔ | This day is closed. Enter a date after { $closed_up_to }, or ask the Owner to reopen the day. | |
| `err-date-in-future` | آگے کی تاریخ نہیں لکھ سکتے۔ آج ({ $today }) یا اس سے پہلے کی تاریخ لکھیں۔ | A future date is not allowed. Enter today ({ $today }) or an earlier date. | |
| `err-code-invalid` | کوڈ درست نہیں۔ پورا کوڈ لکھیں، یا 1 سے 4 ہندسوں کا مختصر کوڈ۔ | This code is not valid. Enter the full code, or a short code of 1 to 4 digits. | |
| `err-name-ur-script` | اردو نام اردو حروف میں لکھیں۔ | Write the Urdu name in Urdu letters. | |
| `err-name-en-script` | انگریزی نام انگریزی حروف میں لکھیں۔ | Write the English name in English letters. | |
| `err-tag-invalid` | نشان 1 سے 4 انگریزی بڑے حروف یا نقطوں کا ہونا چاہیے، جن میں کم از کم ایک حرف ہو۔ مثلاً K، BA یا D.P۔ | The tag must be 1 to 4 English capital letters or dots, with at least one letter. For example K, BA or D.P. | |
| `err-name-too-long` | نام بہت لمبا ہے۔ زیادہ سے زیادہ { $max } حروف لکھیں۔ | The name is too long. Use at most { $max } characters. | |
| `err-phone-invalid` | فون نمبر درست نہیں۔ 03 سے شروع ہونے والا 11 ہندسوں کا نمبر لکھیں۔ | This phone number is not valid. Enter an 11-digit number that starts with 03. | |
| `err-text-too-long` | تحریر بہت لمبی ہے۔ زیادہ سے زیادہ { $max } حروف لکھیں۔ | The text is too long. Use at most { $max } characters. | |
| `err-control-chars` | اس خانے میں چھپا ہوا حرف ہے، مثلاً ٹیب، نئی سطر یا تحریر کی سمت کا نشان۔ اسے مٹائیں اور تحریر دوبارہ لکھیں۔ | This field has a hidden character, such as a tab, a line break or a text-direction mark. Delete it and type the text again. | |
| `err-overflow` | رقم بہت بڑی ہو گئی ہے۔ رقمیں دوبارہ دیکھیں اور پھر کوشش کریں۔ | The amount has become too large. Check the amounts and try again. | |
| `err-shares-not-100` | تمام شراکت دار کا کل حصہ .gender = masculine { $total } فیصد ہے، 100 فیصد نہیں۔ شراکت دار کی ترتیب میں اسے درست کریں۔ | The Partner Share total is { $total }%, not 100%. Correct it in Partner setup. | |
| `err-share-backdated` | حصہ .gender = masculine کی نئی تاریخ پرانی نہیں ہو سکتی۔ آج ({ $today }) یا اس کے بعد کی تاریخ لکھیں۔ | The share change cannot start in the past. Enter today ({ $today }) or a later date. | |
| `err-partner-has-share` | اس شراکت دار کا حصہ .gender = masculine ابھی 0 نہیں ہے۔ پہلے اس کا حصہ .gender = masculine 0 کر کے محفوظ کریں، پھر اسے غیر فعال کریں۔ | This Partner still has a Share. Set his Share to 0 and save the shares first, then make him inactive. | |
| `err-partner-not-listed` | حصہ .gender = masculine کی فہرست میں ہر فعال شراکت دار ایک بار ہونا چاہیے۔ صفحہ دوبارہ کھولیں اور پھر کوشش کریں۔ | The share list must have every active partner once. Open the page again and try again. | |
| `err-too-many-partners` | { $max } سے زیادہ شراکت دار شامل نہیں ہو سکتے۔ | No more than { $max } partners can be added. | |
| `err-disk-full` | ڈسک بھر گئی ہے۔ کچھ جگہ خالی کریں اور دوبارہ کوشش کریں۔ کچھ بھی محفوظ نہیں ہوا۔ | The disk is full. Free some space, then try again. Nothing was saved. | |
| `err-data-check-failed` | ڈیٹا فائل جانچ میں ٹھیک نہیں نکلی، اس لیے آپ کے ڈیٹا کو بچانے کے لیے محفوظ کرنا روک دیا گیا ہے۔ ایپ بند کر کے دوبارہ کھولیں۔ یہ پیغام پھر آئے تو مدد کے لیے رابطہ کریں۔ | The data file did not pass its check, so saving is stopped to protect your data. Close the app and open it again. If this shows again, contact support. | |
| `err-migration-failed` | نئے ورژن کے لیے ڈیٹا کی تبدیلی مکمل نہیں ہو سکی۔ آپ کا ڈیٹا محفوظ ہے: تبدیلی سے پہلے بنائی گئی کاپی واپس رکھ دی گئی ہے۔ پچھلا ورژن دوبارہ لگائیں، یا مدد کے لیے رابطہ کریں۔ | The data could not be updated for the new version. Your data is safe: the copy made before the update was put back. Install the previous version, or contact support. | |
| `err-db-busy` | ڈیٹا فائل ابھی مصروف ہے۔ ذرا دیر بعد دوبارہ کوشش کریں۔ | The data file is busy. Try again in a moment. | |
| `err-password-too-short` | پاس ورڈ کم از کم { $min } حروف کا ہونا چاہیے۔ | The password must be at least { $min } characters long. | |
| `err-passwords-differ` | دونوں پاس ورڈ ایک جیسے نہیں۔ دوبارہ لکھیں۔ | The two passwords are not the same. Type them again. | |
| `err-username-invalid` | صارف کا نام 3 سے 32 حروف کا ہو۔ اس میں انگریزی چھوٹے حروف، ہندسے، نقطہ، نیچے کی لکیر یا درمیانی لکیر ہی لکھ سکتے ہیں۔ | The user name must be 3 to 32 characters. Use only small English letters, digits, a dot, an underscore or a hyphen. | |
| `err-wrong-password` | صارف کا نام یا پاس ورڈ غلط ہے۔ باقی کوششیں: { $tries_left } | The user name or password is wrong. Tries left: { $tries_left } | |
| `err-locked-wait` | بہت زیادہ غلط کوششیں ہو گئیں۔ { $seconds } سیکنڈ رک کر دوبارہ کوشش کریں۔ | Too many wrong tries. Wait { $seconds } seconds, then try again. | |
| `err-user-inactive` | اس کھاتے سے لاگ ان نہیں ہو سکتا۔ ریکوری کوڈ استعمال کریں۔ | This account cannot sign in. Use the recovery code. | |
| `err-already-signed-in` | آپ پہلے ہی لاگ ان ہیں۔ | You are already signed in. | |
| `err-not-locked` | ایپ لاک نہیں ہے۔ | The app is not locked. | |
| `err-recovery-code-wrong` | صارف کا نام یا ریکوری کوڈ غلط ہے۔ دیکھ کر دوبارہ لکھیں۔ | The user name or the recovery code is wrong. Check and type them again. | |
| `err-already-set-up` | پہلی ترتیب پہلے ہی مکمل ہو چکی ہے۔ لاگ ان کریں۔ | First-time setup is already done. Please sign in. | |
| `err-out-of-range` | یہ قیمت { $min } سے { $max } کے درمیان ہونی چاہیے۔ | This value must be between { $min } and { $max }. | |
| `err-not-logged-in` | پہلے لاگ ان کریں۔ | Please sign in first. | |
| `err-session-locked` | ایپ لاک ہے۔ پاس ورڈ لکھ کر کھولیں۔ | The app is locked. Type your password to unlock it. | |
| `err-not-found` | یہ نہیں ملا۔ صفحہ دوبارہ کھولیں اور دوبارہ چنیں۔ | This was not found. Open the page again and choose again. | |
| `err-spelling-unconfirmed` | ان لفظوں کے ہجے کی تصدیق نہیں ہوئی: { $words }۔ ہجے دیکھیں، پھر تصدیق کریں یا درست کریں۔ | The spelling of these words is not confirmed: { $words }. Check the spelling, then confirm it or fix it. | |
| `err-code-taken` | یہ کوڈ کسی اور کے پاس ہے۔ کوئی دوسرا کوڈ لکھیں۔ | This code is already used by another one. Enter a different code. | |
| `err-version-changed` | آپ کے کھولنے کے بعد یہ اندراج بدل چکا ہے۔ اسے دوبارہ کھولیں۔ | This was changed after you opened it. Open it again. | |
| `err-format-invalid` | یہاں لکھی ہوئی بات درست شکل میں نہیں۔ دیکھ کر دوبارہ لکھیں۔ | What is written here is not in the right form. Check it and write it again. | |
| `err-series-full` | اس سلسلے کے سب کوڈ استعمال ہو چکے ہیں۔ کوئی اور سلسلہ استعمال کریں۔ | Every code of this series is used. Use another series. | |
| `err-journal-unbalanced` | اندراج کا بنام اور جمع برابر نہیں، اس لیے کچھ محفوظ نہیں ہوا۔ رقمیں دیکھ کر دوبارہ کوشش کریں۔ | The Debit and the Credit of this entry are not equal, so nothing was saved. Check the amounts and try again. | |
| `err-code-reserved` | یہ کوڈ ایپ کے اپنے کھاتوں کے لیے رکھا گیا ہے۔ کوئی دوسرا کوڈ لکھیں۔ | This code is kept for the app's own accounts. Enter a different code. | |
| `err-opening-both-sides` | ابتدائی بنام یا ابتدائی جمع لکھیں، دونوں نہیں۔ | Enter an opening Debit or an opening Credit, not both. | |
| `err-advance-not-seller` | ابتدائی پیشگی .gender = feminine صرف بیوپاری .gender = masculine کے ابتدائی بنام پر لگ سکتی ہے۔ | An opening Advance can only be set on the opening Debit of a Seller. | |
| `err-field-not-allowed` | یہ خانہ اب استعمال نہیں ہوتا۔ اسے چھوڑ دیں۔ | This field is no longer used. Leave it out. | |
| `err-opening-not-allowed` | اس کھاتے میں یہ ابتدائی بقایا نہیں لکھا جا سکتا۔ آمدن اور دکان خرچہ .gender = masculine کے کھاتوں، نقدی منڈی، پختہ کھاتہ .gender = masculine اور ابتدائی فرق میں ابتدائی بقایا نہیں لگتا۔ شراکت دار کے ذاتی خرچہ .gender = masculine کھاتے اور ادھار رقم .gender = feminine کے کھاتے میں صرف ابتدائی بنام لگ سکتا ہے۔ | This account cannot take this opening balance. Income and Shop Expense accounts, Naqdi Mandi, Pukhta Khata and Opening Difference cannot take one. A partner's Personal Drawing account and a Money Lent account can take an opening debit only. | |
| `err-not-a-seller` | یہ بیوپاری .gender = masculine کا کھاتہ نہیں۔ پیشگی .gender = feminine صرف بیوپاری .gender = masculine کو دی جا سکتی ہے، پختہ کھاتہ .gender = masculine کو نہیں۔ | This is not a Seller account. An Advance can only be given to a Seller, not to Pukhta Khata. | |
| `err-account-inactive` | یہ کھاتہ غیر فعال ہے اور نئے اندراج میں استعمال نہیں ہو سکتا۔ کوئی دوسرا کھاتہ چنیں۔ | This account is inactive and cannot be used in a new entry. Choose another account. | |
| `err-system-account` | یہ نظام کا کھاتہ ہے۔ حساب کتاب میں اس کا استعمال ہوتا ہے، اس لیے اسے غیر فعال نہیں کیا جا سکتا۔ | This is a system account. The books use it, so it cannot be made inactive. | |
| `err-reason-required` | وجہ لکھنا ضروری ہے۔ | A reason is required. | |
| `err-trial-not-balanced` | ٹرائل بیلنس .gender = masculine میں فرق ہے، اس لیے دن بند نہیں ہو سکتا۔ پہلے ٹرائل بیلنس .gender = masculine دیکھ کر فرق ٹھیک کریں۔ | The Trial Balance has a difference, so the day cannot be closed. Check the Trial Balance and fix the difference first. | |
| `err-day-not-closed` | یہ دن بند ہی نہیں، اس لیے کھولنے کے لیے کچھ نہیں۔ بند دن کی تاریخ لکھیں۔ | This day is not closed, so there is nothing to reopen. Enter the date of a closed day. | |
| `err-month-closed` | اس تاریخ کا مہینہ بند ہو چکا ہے۔ دن کھولنے سے پہلے مہینے کی بندش واپس کریں۔ | The month of this date is closed. Reverse the month close before reopening the day. | |
| `err-nothing-to-close` | آج تک کے سب دن پہلے ہی بند ہیں، اس لیے بند کرنے کے لیے کوئی دن نہیں۔ | Every day up to today is already closed, so there is no day to close. | |
| `err-rule-not-allowed` | اصول کی یہ ترتیب درست نہیں۔ چارج اور طریقہ فہرست میں سے چنیں، اور گروپ صرف اسی وقت چنیں جب کوئی کھاتہ یا قسم مال نہ چنا ہو۔ عام شرحیں ترتیبات کے چارجز میں بدلتی ہیں۔ | This combination is not allowed for a rule. Choose a charge and a method from the lists, and a group only when no party or Item is chosen. The default rates are changed in Settings, Charges. | |
| `err-date-range` | آخری تاریخ پہلی تاریخ سے پہلے ہے۔ آخری تاریخ پہلی تاریخ کے دن یا اس کے بعد کی لکھیں۔ | The last date is before the first date. Enter a last date on or after the first date. | |
| `err-qty-exceeds-lot` | تعداد لاٹ .gender = masculine کی باقی تعداد سے زیادہ ہے۔ صرف { $available } نگ باقی ہیں۔ | The quantity is more than the lot has left. Only { $available } Nag are left. | |
| `err-override-not-allowed` | لائن پر کمیشن .gender = masculine یا لاگا .gender = masculine بدلنا بند ہے۔ مالک اسے ترتیبات میں چالو کر سکتا ہے۔ | Changing the Commission or Laga on a line is switched off. The Owner can switch it on in Settings. | |
| `err-credit-limit-exceeded` | اس بکری .gender = feminine سے گاہک .gender = masculine کا بقایا { $would_owe } ہو جائے گا، جو اس کی ادھار کی حد { $limit } سے زیادہ ہے۔ | This sale takes the buyer's balance to { $would_owe }, which is over his credit limit of { $limit }. | |
| `err-seller-required` | اس لاٹ .gender = masculine کا بیوپاری .gender = masculine چنیں۔ | Choose the seller of this lot. | |
| `err-buyer-required` | اس لائن کا گاہک .gender = masculine چنیں۔ | Choose the buyer of this line. | |
| `err-pukhta-name-required` | پختہ کھاتہ .gender = masculine میں نام لکھنا ضروری ہے۔ | A name is required for Pukhta Khata. | |
| `err-pukhta-phone-required` | اس پختہ کھاتہ .gender = masculine والے کا فون نمبر لکھنا ضروری ہے۔ | A phone number is required for this Pukhta Khata person. | |
| `err-no-lots` | کم از کم ایک لاٹ .gender = masculine لکھیں۔ | Add at least one lot. | |
| `err-no-sale-lines` | کم از کم بکری .gender = feminine کی ایک لائن لکھیں۔ | Add at least one sale line. | |
| `err-weight-required` | وزن کلو میں لکھیں۔ یہ مال وزن کے حساب سے بکتا ہے۔ | Enter the weight in kilograms. This item is sold by weight. | |
| `err-cash-below-zero-not-confirmed` | اس اندراج کے بعد روکڑ { $cash_after } ہو جائے گی۔ محفوظ کرنے کے لیے اندراج کی تصدیق کریں۔ | After this entry cash in hand would be { $cash_after }. Confirm the entry to save it. | |
| `err-not-a-buyer` | یہ گاہک .gender = masculine کا کھاتہ نہیں۔ کوئی گاہک .gender = masculine چنیں۔ | This is not a Buyer account. Choose a buyer. | |
| `err-credit-not-allowed` | اس گاہک .gender = masculine کے لیے ادھار کی اجازت نہیں۔ لائن کو نقدی کر دیں۔ | Credit is not allowed for this buyer. Change the line to cash. | |
| `err-qty-left-unsold` | { $left } نگ باقی ہیں۔ یہ باقی مال میں رہیں گے۔ | { $left } Nag are left. They will stay in Unsold Stock. | |
| `err-lot-net-negative` | اس لاٹ .gender = masculine کی صافی بکری .gender = feminine صفر سے کم ({ $net }) ہے۔ یہ رقم بیوپاری .gender = masculine کے ذمے ہے۔ | The Net Sale of this lot is below zero ({ $net }). The seller owes this amount. | |
| `err-rate-far-from-average` | یہ ریٹ پچھلے 7 دن کی اوسط ({ $average }) سے { $percent } فیصد سے زیادہ مختلف ہے۔ اسے دیکھ لیں۔ | This rate is more than { $percent }% away from the average of the last 7 days ({ $average }). Check it. | |
| `err-draft-invalid` | لکھا ہوا کام عارضی طور پر محفوظ نہیں ہو سکا۔ اندراج ابھی محفوظ کر لیں۔ بار بار ہو تو مدد کے لیے بتائیں۔ | Your unfinished work could not be kept for now. Save the entry now. If this happens again, tell us. | |
| `err-print-windows-only` | پرنٹنگ صرف ونڈوز پر ہو سکتی ہے۔ اس کمپیوٹر پر پیش منظر میں دیکھ لیں کہ دستاویز کیسے چھپے گی۔ | Printing works only on Windows. On this computer, use the preview to see how the document will print. | |
| `err-printer-not-found` | پرنٹر { $printer } نہیں ملا۔ دیکھیں کہ وہ آن ہے اور جڑا ہوا ہے، یا ترتیبات میں دوسرا پرنٹر چنیں۔ | The printer { $printer } was not found. Check that it is on and connected, or choose another printer in Settings. | |
| `err-print-failed` | دستاویز پرنٹ نہیں ہو سکی۔ پرنٹر دیکھ لیں اور دوبارہ کوشش کریں۔ بار بار ہو تو مدد کے لیے بتائیں۔ | The document could not be printed. Check the printer and try again. If it keeps happening, ask for help. | |
| `err-payment-to-buyer-needs-reason` | گاہک .gender = masculine کو رقم دینا نقد ادھار .gender = masculine ہے۔ اس کی وجہ لکھیں۔ | Money paid to a Buyer is Cash Lent. Please write the reason. | |
| `err-return-exceeds-open` | یہ رقم واپس ہونے والی رقم سے زیادہ ہے۔ زیادہ سے زیادہ { $open } واپس ہو سکتے ہیں۔ | This is more than can be returned. The most that can be returned is { $open }. | |
| `err-voucher-cancelled` | یہ واؤچر منسوخ ہو چکا ہے، اس میں تبدیلی نہیں ہو سکتی۔ | This voucher is cancelled and cannot be changed. | |
| `err-person-required` | پختہ کھاتہ .gender = masculine کے لیے اس شخص کا نام یا فون لکھ کر اسے فہرست سے چنیں۔ | For Pukhta Khata, type the name or phone of the person and pick him from the list. | |
| `err-payment-above-payable` | یہ رقم بیوپاری .gender = masculine کی قابل ادائیگی رقم ({ $payable }) سے { $over } زیادہ ہے۔ آپ پھر بھی محفوظ کر سکتے ہیں۔ زیادہ رقم بیوپاری .gender = masculine کے ذمے دکھائی جائے گی، یہ پیشگی .gender = feminine نہیں ہوگی۔ | This is { $over } more than what is payable to the Seller ({ $payable }). You can still save it. The extra shows as owed by the Seller; it is not an Advance. | |
| `err-bikri-cancelled` | یہ بکری .gender = feminine منسوخ ہو چکی ہے اور بدلی نہیں جا سکتی۔ | This Sale is cancelled and cannot be changed. | |
| `err-lot-not-found` | یہ لاٹ .gender = masculine نہیں ملا۔ باقی مال کی فہرست کھول کر دوبارہ چنیں۔ | This lot was not found. Open the list of Unsold Stock and choose again. | |
| `err-nothing-to-sell` | اس لاٹ .gender = masculine میں بیچنے کے لیے کچھ باقی نہیں۔ | Nothing is left in this lot to sell. | |
| `err-write-off-exceeds-balance` | باقی مال سے زیادہ ضائع نہیں ہو سکتا۔ صرف { $available } نگ باقی ہیں۔ | You cannot write off more than is left. Only { $available } Nag are left. | |
| `err-date-before-received` | دن مال وصول ہونے کے دن سے پہلے کا نہیں ہو سکتا۔ | The day cannot be before the day the goods were received. | |
| `err-qty-below-sold` | وصول شدہ تعداد اس لاٹ .gender = masculine سے پہلے بکی ہوئی تعداد سے کم نہیں ہو سکتی۔ | The quantity received cannot be less than the quantity already sold from this lot. | |
| `err-edit-not-allowed` | { $rule -> [day_changed] محفوظ شدہ اندراج کا دن ترمیم میں بدلا نہیں جا سکتا۔ پرانا دن واپس لکھیں۔ [lot_removed] محفوظ شدہ لاٹ .gender = masculine ہٹایا نہیں جا سکتا، کیونکہ اس کی فروخت اور مال اس پر قائم ہیں۔ لاٹ .gender = masculine واپس رکھیں۔ پوری بکری .gender = feminine واپس لینی ہو تو اسے منسوخ کریں۔ [lot_not_in_posting] یہ لاٹ .gender = masculine اس اندراج کا حصہ نہیں جس کی ترمیم ہو رہی ہے۔ ترمیم بند کر کے اندراج دوبارہ کھولیں۔ [lot_order_changed] محفوظ شدہ لاٹ .gender = masculine اپنی ترتیب میں رہتے ہیں اور نئے لاٹ .gender = masculine ان کے بعد آتے ہیں۔ نئے لاٹ .gender = masculine کو آخر میں رکھیں۔ [lot_seller_locked] اس لاٹ .gender = masculine پر بعد کا اندراج یا فروخت قائم ہو چکی ہے، اس لیے بیوپاری .gender = masculine نہیں بدلا جا سکتا۔ پرانا بیوپاری .gender = masculine واپس چنیں۔ [lot_item_locked] اس لاٹ .gender = masculine پر بعد کا اندراج یا فروخت قائم ہو چکی ہے، اس لیے قسم مال نہیں بدلی جا سکتی۔ پرانی قسم مال واپس چنیں۔ [line_not_in_posting] یہ لائن اس اندراج کا حصہ نہیں جس کی ترمیم ہو رہی ہے۔ ترمیم بند کر کے اندراج دوبارہ کھولیں۔ [posting_lots_locked] بعد کی فروخت میں صرف اس کی اپنی لائنیں بدلی جا سکتی ہیں، لاٹ .gender = masculine نہیں۔ لاٹ .gender = masculine کی تبدیلی پہلے اندراج میں کریں۔ *[other] اس اندراج میں یہ تبدیلی نہیں ہو سکتی۔ ترمیم بند کر کے اندراج دوبارہ کھولیں۔ | { $rule -> [day_changed] The day of a saved entry cannot be changed in an edit. Write the old day back. [lot_removed] A saved lot cannot be removed, because its sales and its goods rest on it. Put the lot back. To take the whole sale back, cancel it. [lot_not_in_posting] This lot is not part of the entry being edited. Close the edit and open the entry again. [lot_order_changed] Saved lots keep their order, and new lots come after them. Move the new lot to the end. [lot_seller_locked] A later entry or sale already rests on this lot, so its seller cannot be changed. Choose the old seller again. [lot_item_locked] A later entry or sale already rests on this lot, so its item cannot be changed. Choose the old item again. [line_not_in_posting] This line is not part of the entry being edited. Close the edit and open the entry again. [posting_lots_locked] In a later sale only its own lines can be changed, not the lot. Change the lot in the first entry. *[other] This change is not allowed in this entry. Close the edit and open the entry again. | |
| `err-own-lot-not-fully-sold` | اپنا مال کا یہ لاٹ .gender = masculine اسی بکری .gender = feminine میں پورا بکنا چاہیے۔ لائنوں میں ابھی { $qty_left } نگ نہیں بکے۔ انہیں لائنوں میں بیچیں، یا خریداری سے کم نگ لیں۔ جو نہ لیں وہ خریداری میں رہتا ہے۔ | This lot of Own Goods must be sold in full in this Sale. { $qty_left } Nag are not sold on the lines yet. Sell them on the lines, or take fewer Nag from the purchase. What you do not take stays in the purchase. | |
| `err-own-goods-account-restricted` | اپنا مال صرف بکری .gender = feminine کے لاٹ .gender = masculine کا بیوپاری .gender = masculine ہو سکتا ہے۔ کوئی اور کھاتہ چنیں۔ | Own Goods can only be the seller of a lot in the Sale. Choose another account. | |
| `err-purchase-has-sales` | یہ خریداری منسوخ نہیں ہو سکتی، کیونکہ { $qty_sold } نگ بک چکے ہیں اور { $qty_written_off } نگ ضائع ہو چکے ہیں۔ پہلے بکری .gender = feminine منسوخ کریں یا ضائع واپس لیں۔ | This purchase cannot be cancelled, because { $qty_sold } Nag were sold and { $qty_written_off } Nag were written off. Cancel the sales or undo the write-off first. | |
| `err-purchase-cancelled` | یہ خریداری منسوخ ہو چکی ہے۔ کوئی اور خریداری چنیں۔ | This purchase is cancelled. Choose another purchase. | |
| `err-own-lot-freight-in-cost` | خریداری کا کرایہ .gender = masculine اس کی لاگت میں پہلے سے شامل ہے۔ یہاں لکھا ہوا کرایہ .gender = masculine دوبارہ گنا جائے گا۔ جب تک یہی مقصد نہ ہو، اسے خالی رہنے دیں۔ | The freight of the purchase is already in its cost. A freight typed here is counted a second time. Leave it empty unless that is what you mean. | |
| `err-trip-closed` | یہ مارکیٹ سفر .gender = masculine بند ہو چکا ہے۔ اس میں نئی خریداری یا نیا خرچ نہیں ہو سکتا۔ | This Market Trip is closed. It takes no new purchase or cost. | |
| `err-trip-no-stock` | اس مارکیٹ سفر .gender = masculine کی کسی خریداری میں مال باقی نہیں، اس لیے خرچ بانٹنے کو کچھ نہیں۔ پہلے خریداری درج کریں۔ | No purchase of this Market Trip has goods left, so there is nothing to spread the cost over. Record the purchase first. | |
| `err-venture-shares-locked` | اس مارکیٹ سفر .gender = masculine میں اندراج ہو چکا ہے، اس لیے شراکت داروں کا حصہ .gender = masculine اب نہیں بدل سکتا۔ | This Market Trip already has an entry, so the Share of its partners can no longer be changed. | |
| `err-purchase-before-trip` | تاریخ مارکیٹ سفر .gender = masculine کی تاریخ ({ $trip_date }) سے پہلے کی ہے۔ سفر کی تاریخ یا اس کے بعد کی تاریخ چنیں۔ | The day is before the day of the Market Trip ({ $trip_date }). Choose the day of the trip or a later day. | |
| `err-trip-cost-has-sales` | یہ خرچ منسوخ نہیں ہو سکتا، کیونکہ جن خریداریوں پر یہ بانٹا گیا ان کے { $qty_sold } نگ بک چکے ہیں اور { $qty_written_off } نگ ضائع ہو چکے ہیں۔ پہلے بکری .gender = feminine منسوخ کریں یا ضائع واپس لیں۔ | This cost cannot be cancelled, because { $qty_sold } Nag of the purchases it was spread over were sold and { $qty_written_off } Nag were written off. Cancel the sales or undo the write-off first. | |
| `err-partner-in-open-venture` | یہ شراکت دار ابھی کھلے مارکیٹ سفر .gender = masculine میں ہے، اس لیے اسے غیر فعال نہیں کیا جا سکتا۔ پہلے مارکیٹ سفر .gender = masculine بند ہونے دیں۔ | This Partner is in an open Market Trip, so he cannot be made inactive. Let the Market Trip close first. | |
| `err-purchase-carries-trip-cost` | اس خریداری میں مارکیٹ سفر .gender = masculine کے خرچ کا حصہ شامل ہے، اس لیے یہ منسوخ نہیں ہو سکتی۔ پہلے وہ خرچ منسوخ کریں۔ | This purchase carries a part of a Market Trip cost, so it cannot be cancelled. Cancel that cost first. | |
| `err-trip-entry-cancelled` | یہ اندراج پہلے ہی منسوخ ہو چکا ہے۔ | This entry is cancelled already. | |
| `err-backup-not-found` | یہ کاپی نہیں ملی۔ ہو سکتا ہے ہٹا دی گئی ہو۔ فہرست دوبارہ کھولیں اور پھر کوشش کریں۔ | This copy was not found. It may have been removed. Refresh the list and try again. | |
| `err-backup-key-not-set` | اس کمپیوٹر پر ابھی بیک اپ کی چابی نہیں ہے۔ پہلے چابی بنائیں۔ | There is no backup key on this computer yet. Make the key first. | |
| `err-backup-key-already-set` | بیک اپ کی چابی پہلے سے استعمال میں ہے، اس لیے نئی نہیں بن سکتی۔ | A backup key is in use already, so a new one cannot be made. | |
| `err-backup-key-format` | چابی صحیح طرح نہیں لکھی گئی۔ اس میں چار چار حرفوں اور ہندسوں کے چھ حصے ہوتے ہیں، جن کے درمیان ڈیش ہوتی ہے۔ | The key is not written the right way. It has six groups of four letters and numbers, with dashes between them. | |
| `err-backup-key-wrong` | یہ درست چابی نہیں ہے۔ پرنٹ کی ہوئی پرچی دیکھیں اور دوبارہ کوشش کریں۔ کچھ نہیں بدلا۔ | This is not the right key. Check the printed sheet and try again. Nothing was changed. | |
| `err-backup-key-required` | یہ کاپی بیک اپ کی چابی سے بند ہے۔ پرنٹ کی ہوئی پرچی سے چابی لکھیں۔ | This copy is locked with the backup key. Type the key from your printed sheet. | |
| `err-backup-corrupt` | یہ کاپی خراب ہے اور استعمال نہیں ہو سکتی۔ کچھ نہیں بدلا۔ کوئی اور کاپی استعمال کریں۔ | This copy is damaged and cannot be used. Nothing was changed. Use another copy. | |
| `err-backup-check-failed` | کاپی بن گئی لیکن جانچ میں پوری نہیں اتری، اس لیے اس پر بھروسا نہیں کیا جا سکتا۔ نئی کاپی بنائیں۔ | The copy was made but did not pass its check, so it cannot be trusted. Make a new copy. | |
| `err-backup-cloud-not-configured` | کلاؤڈ کاپی ابھی ترتیب نہیں دی گئی۔ بیک اپ کی ترتیبات میں کلاؤڈ کی تفصیل بھریں۔ | The cloud copy is not set up. Fill in the cloud details in the backup settings. | |
| `err-backup-cloud-failed` | { $reason -> [offline] کلاؤڈ تک رسائی نہیں ہو سکی۔ انٹرنیٹ دیکھیں۔ کاپی انتظار میں رہتی ہے اور خود بخود دوبارہ بھیجی جاتی ہے۔ [refused] کلاؤڈ نے کاپی قبول نہیں کی۔ بیک اپ کی ترتیبات میں کلاؤڈ کی چابیاں دیکھیں۔ [not_found] کلاؤڈ کا بکٹ یا کاپی نہیں ملی۔ بیک اپ کی ترتیبات میں بکٹ کا نام دیکھیں۔ *[other] کلاؤڈ کاپی نہیں بن سکی۔ کچھ دیر بعد دوبارہ کوشش کریں۔ } | { $reason -> [offline] The cloud could not be reached. Check the internet. The copy waits and is sent again by itself. [refused] The cloud refused the copy. Check the cloud keys in the backup settings. [not_found] The cloud bucket or the copy was not found. Check the bucket name in the backup settings. *[other] The cloud copy failed. Try again later. } | |
| `err-backup-usb-not-present` | یو ایس بی ڈرائیو لگی ہوئی نہیں۔ اسے لگائیں، یا بیک اپ کی ترتیبات میں یو ایس بی کاپی بند کر دیں۔ | The USB drive is not plugged in. Plug it in, or switch the USB copy off in the backup settings. | |
| `err-backup-usb-failed` | یو ایس بی ڈرائیو پر کاپی نہیں بن سکی۔ ڈرائیو اور فولڈر دیکھیں۔ | The copy to the USB drive failed. Check the drive and the folder. | |
| `err-backup-local-failed` | اس کمپیوٹر پر کاپی نہیں بن سکی۔ بیک اپ کی ترتیبات میں فولڈر اور ڈسک کی خالی جگہ دیکھیں۔ | The copy on this computer failed. Check the folder in the backup settings and the free space on the disk. | |
| `err-backup-busy` | بیک اپ یا بحالی پہلے سے چل رہی ہے۔ اس کے مکمل ہونے تک انتظار کریں۔ | A backup or restore is running already. Wait until it is done. | |
| `err-backup-not-confirmed` | بحالی شروع ہونے سے پہلے تصدیق کریں۔ کچھ نہیں بدلا۔ | Please confirm before the restore starts. Nothing was changed. | |
| `err-backup-newer-than-app` | یہ کاپی اس کمپیوٹر کی ایپ سے نئے ورژن کی ہے۔ پہلے ایپ اپ ڈیٹ کریں۔ | This copy is from a newer version of the app than the one on this computer. Update the app first. | |
| `err-backup-not-started` | بیک اپ کی سروس ابھی چل نہیں رہی۔ ذرا انتظار کریں اور دوبارہ کوشش کریں۔ | The backup service is not running yet. Wait a moment and try again. | |
| `err-backup-restore-failed` | بحالی مکمل نہیں ہو سکی۔ ایپ بند کر کے دوبارہ کھولیں۔ اگر کھاتے غلط لگیں تو مدد کے لیے رابطہ کریں: بحالی سے پہلے حفاظتی کاپی بنائی گئی تھی۔ | The restore could not be finished. Close the app and open it again. If your books look wrong, contact support: a safety copy was made before the restore. | |
| `err-return-by-before-entry` | واپسی کی تاریخ اندراج کی تاریخ سے پہلے نہیں ہو سکتی۔ | The return-by date cannot be before the entry date. | |
| `err-partner-required` | شراکت دار چنیں۔ | Choose the Partner. | |
| `err-expense-head-required` | خرچہ مد چنیں۔ | Choose the expense head. | |
| `err-lent-person-required` | شخص چنیں۔ | Choose the person. | |
| `err-khata-required` | الگ کھاتہ .gender = masculine چنیں۔ | Choose the Separate Khata. | |
| `err-general-not-balanced` | بنام کا میزان { $dr } اور جمع کا میزان { $cr } ہے۔ اندراج محفوظ کرنے سے پہلے دونوں برابر ہونے چاہییں۔ | The Debit total is { $dr } and the Credit total is { $cr }. They must be equal before the entry can be saved. | |
| `err-general-too-few-lines` | جنرل اندراج میں کم از کم دو لائنیں ہونی چاہییں۔ | A Journal entry needs at least two lines. | |
| `err-capital-exceeds-balance` | یہ شراکت دار کے سرمایہ بقایا { $balance } سے { $over } زیادہ ہے۔ پھر بھی محفوظ کیا جا سکتا ہے۔ | This is more than the partner's Capital balance of { $balance }, by { $over }. You can still save it. | |
| `err-update-check-failed` | ایپ نیا ورژن نہیں دیکھ سکی۔ انٹرنیٹ دیکھیں اور دوبارہ کوشش کریں۔ آپ کا کام محفوظ ہے۔ | The app could not look for a new version. Check the internet connection and try again. Your work is not affected. | |
| `err-update-not-signed` | یہ اپڈیٹ ہماری طرف سے دستخط شدہ نہیں، اس لیے انسٹال نہیں ہوئی۔ کچھ نہیں بدلا اور کوئی کاپی نہیں بنی۔ یہ اپڈیٹ استعمال نہ کریں۔ | This update is not signed by us, so it was not installed. Nothing was changed and no copy was made. Do not use this update. | |
| `err-update-during-work` | غیر محفوظ اندراجات ہیں ({ $drafts })۔ انہیں محفوظ کریں یا چھوڑ دیں، پھر اپڈیٹ کریں۔ کچھ انسٹال نہیں ہوا۔ | There are unsaved entries ({ $drafts }). Save them or throw them away, then update. Nothing was installed. | |
| `err-update-not-confirmed` | اپڈیٹ کی تصدیق نہیں ہوئی، اس لیے کچھ انسٹال نہیں ہوا۔ اپڈیٹ کا بٹن دبائیں اور سوال کا جواب دیں۔ | The update was not confirmed, so nothing was installed. Press the update button and answer the question. | |
| `err-update-backup-failed` | اپڈیٹ سے پہلے حفاظتی کاپی نہیں بن سکی، اس لیے کچھ انسٹال نہیں ہوا۔ بیک اپ کی سکرین دیکھیں اور دوبارہ کوشش کریں۔ | The safety copy before the update could not be made, so nothing was installed. Look at the backup screen and try again. | |
| `err-update-changed` | اب کوئی اور ورژن دستیاب ہے۔ نیا ورژن دوبارہ دیکھیں اور اپڈیٹ سے پہلے نیا کیا ہے پڑھ لیں۔ | A different version is on offer now. Look for the new version again and read what is new before you update. | |
| `err-update-none` | کوئی نیا ورژن نہیں ہے۔ ایپ تازہ ترین ہے۔ | There is no newer version. The app is up to date. | |
| `err-fruit-cancelled` | یہ پھل کا اندراج پہلے ہی منسوخ ہو چکا ہے، اس لیے اس میں تبدیلی نہیں ہو سکتی۔ | This fruit entry was cancelled already, so it cannot be changed. | |
| `err-bikri-has-fruit` | اس بکری .gender = feminine کے لاٹ .gender = masculine سے { $qty_taken } نگ پھل لیا گیا ہے، اس لیے یہ منسوخ نہیں ہو سکتی۔ پہلے پھل کا اندراج منسوخ کریں۔ | { $qty_taken } Nag of Fruit Taken were taken from a lot of this Sale, so it cannot be cancelled. Cancel the Fruit Taken entry first. | |
| `err-garden-closed` | یہ باغ بند ہے، اس لیے اس میں نیا اندراج، نیا لاٹ یا پھل نہیں لیا جا سکتا۔ پہلے اس کی بندش واپس لیں۔ | This Garden is closed, so it takes no new entry, no lot and no fruit. Undo its closing first. | |
| `err-garden-not-closed` | یہ باغ کھلا ہے، اس لیے واپس لینے کے لیے کوئی بندش نہیں۔ | This Garden is open, so there is no closing to undo. | |
| `err-date-before-garden` | تاریخ باغ کی تاریخ ({ $garden_date }) سے پہلے کی ہے۔ باغ کی تاریخ یا اس کے بعد کی تاریخ چنیں۔ | The day is before the day of the Garden ({ $garden_date }). Choose the day of the garden or a later day. | |
| `err-close-before-last-entry` | باغ اپنے آخری اندراج یا لاٹ ({ $last_date }) سے پہلے بند نہیں ہو سکتا۔ وہی دن یا اس کے بعد کا دن چنیں۔ | The Garden cannot be closed before its latest entry or lot ({ $last_date }). Choose that day or a later day. | |
| `err-garden-entry-cancelled` | باغ کا یہ اندراج پہلے ہی منسوخ ہو چکا ہے۔ | This entry of the Garden is cancelled already. | |

## export

| ID | Urdu | English | Result |
|---|---|---|---|
| `export-excel` | ایکسل | Excel | |
| `export-file-type` | ایکسل فائل | Excel file | |
| `export-close` | بند کریں | Close | |
| `export-reading` | ڈیٹا پڑھا جا رہا ہے... | Reading the data... | |
| `export-writing` | شیٹ لکھی جا رہی ہے: { $total } میں سے { $done } قطاریں | Writing the sheet: { $done } of { $total } rows | |
| `export-finishing` | فائل مکمل کی جا رہی ہے... | Finishing the file... | |
| `export-progress-label` | ایکسل فائل میں محفوظ ہو رہا ہے | Saving as an Excel file | |
| `export-done` | ایکسل فائل { $path } کے نام سے محفوظ ہو گئی۔ اس میں { $rows } قطاریں ہیں۔ | The Excel file is saved as { $path }. It holds { $rows } rows. | |
| `export-path-refused` | پوری فائل کا نام چنیں جو .xlsx پر ختم ہو اور فولڈر موجود ہو۔ | Choose a full file name ending in .xlsx, in a folder that exists. | |
| `export-too-many-rows` | ایک ایکسل شیٹ کے لیے قطاریں بہت زیادہ ہیں۔ تاریخیں کم کریں یا کوئی فلٹر ہٹائیں، پھر دوبارہ کوشش کریں۔ | There are too many rows for one Excel sheet. Narrow the dates or take a filter away, then try again. | |

## fruit

| ID | Urdu | English | Result |
|---|---|---|---|
| `fruit-title` | پھل | Fruit Taken | |
| `fruit-help` | کوئی شراکت دار پھل گھر لے جائے یا دکان کے مہمان کے لیے ہو، وہ قسم مال اور تعداد کے ساتھ درج ہوتا ہے۔ شراکت دار کا پھل صرف اس کی کل رقم سے کٹتا ہے۔ مہمان کا پھل شراکت داروں میں برابر بٹتا ہے، کمیشن .gender = masculine کے حصوں کے مطابق نہیں۔ اس اسکرین سے روکڑ میں کوئی رقم نہیں ہلتی۔ | Fruit a partner takes home, or fruit served to the shop's Guests, recorded by Item and quantity. A partner's fruit comes off his Partner Total only. Fruit for Guests is shared equally between the partners, not by their shares of the Commission. No cash moves on this screen. | |
| `fruit-load-failed` | پھل کی اسکرین نہیں کھل سکی | The Fruit Taken screen could not be opened | |
| `fruit-field-for` | کس کے لیے | For | |
| `fruit-for-guests` | مہمان | Guests | |
| `fruit-for-hint` | شراکت دار: اس کے گھر کے لیے۔ مہمان: دکان کے مہمانوں کے لیے، شراکت داروں میں برابر بٹے گا۔ | A partner: for his home. Guests: for the shop's guests, shared equally between the partners. | |
| `fruit-partners-failed` | شراکت دار نہیں آ سکے | The partners could not be loaded | |
| `fruit-field-date` | تاریخ | Date | |
| `fruit-field-source` | کہاں سے | From where | |
| `fruit-source-own` | اپنی دکان | Our Shop | |
| `fruit-source-other` | دوسری دکان | Another Shop | |
| `fruit-source-own-help` | باقی مال کے کسی لاٹ .gender = masculine سے پھل۔ لاٹ .gender = masculine کا باقی مال کم ہوگا اور اس کے بیوپاری .gender = masculine کے کھاتے میں قیمت جمع ہوگی، کمیشن .gender = masculine کے بغیر۔ | Fruit from a Lot in Unsold Stock. The balance of the Lot goes down, and its Seller is credited the value, with no Commission. | |
| `fruit-source-other-help` | کسی دوسرے آڑھتی .gender = masculine کی دکان سے پھل۔ اس کے کھاتے میں قیمت جمع ہوگی۔ دکان اسے بعد میں عام ادائیگی .gender = feminine سے ادا کرے گی۔ | Fruit from another Commission Agent (Arhti)'s shop. His account is credited the value. The shop pays him later with a normal Payment. | |
| `fruit-field-lot` | لاٹ .gender = masculine | Lot | |
| `fruit-lot-hint` | صرف وہ لاٹ .gender = masculine دکھائے جاتے ہیں جن میں مال باقی ہے۔ | Only lots with goods left are offered. | |
| `fruit-lot-option` | بکری .gender = feminine { $bikri }، لاٹ .gender = masculine { $lot }: { $item }، { $packing }، { $seller }، باقی { $remaining } | Sale { $bikri }, Lot { $lot }: { $item }, { $packing }, { $seller }, { $remaining } left | |
| `fruit-lot-empty` | کسی لاٹ .gender = masculine میں مال باقی نہیں | No lot has goods left | |
| `fruit-lot-failed` | لاٹ .gender = masculine نہیں آ سکے | The lots could not be loaded | |
| `fruit-field-agent` | آڑھتی .gender = masculine | Commission Agent (Arhti) | |
| `fruit-agent-hint` | دوسرے آڑھتی .gender = masculine کا بیوپاری .gender = masculine کھاتہ۔ اگر موجود نہ ہو تو F4 سے بنائیں۔ | The seller account of the other commission agent. Add it with F4 if it is not there yet. | |
| `fruit-item-from-lot` | لاٹ .gender = masculine سے لیا گیا | Taken from the lot | |
| `fruit-field-qty` | تعداد | Quantity | |
| `fruit-qty-left` | لاٹ .gender = masculine میں { $remaining } باقی ہیں | The lot has { $remaining } left | |
| `fruit-field-weight` | وزن (کلو) | Weight (kg) | |
| `fruit-field-rate` | ریٹ | Rate | |
| `fruit-rate-hint-note` | ہلکے رنگ کا لکھا صرف اشارہ ہے۔ ریٹ خود لکھیں۔ | The grey text is only a hint. Type the rate. | |
| `fruit-field-note` | نوٹ | Note | |
| `fruit-numbers-title` | یہ اندراج | This entry | |
| `fruit-numbers-empty` | فارم بھریں تو اعداد یہاں نظر آئیں گے۔ | Fill in the form and the figures show here. | |
| `fruit-numbers-incomplete` | فارم مکمل ہو تو اعداد نظر آئیں گے۔ | The figures show when the form is complete. | |
| `fruit-numbers-failed` | اعداد نہیں نکل سکے | The figures could not be worked out | |
| `fruit-numbers-value` | قیمت | Value | |
| `fruit-numbers-credit` | کھاتے میں جمع | Credited to | |
| `fruit-numbers-lot-before` | ابھی لاٹ .gender = masculine کا باقی مال | Balance of the lot now | |
| `fruit-numbers-lot-after` | اس کے بعد لاٹ .gender = masculine کا باقی مال | Balance of the lot after | |
| `fruit-parts-title-home` | { $name } کی کل رقم سے کٹے گا | Comes off the Partner Total of { $name } | |
| `fruit-parts-title-guests` | شراکت داروں میں برابر بٹے گا | Shared equally between the partners | |
| `fruit-parts-note` | جو روپیہ برابر نہ بٹے وہ سب سے پہلے بنائے گئے شراکت دار کو ملتا ہے۔ کمیشن .gender = masculine کے حصے استعمال نہیں ہوتے۔ | A rupee that does not divide goes to the partner made first. The Commission shares are not used. | |
| `fruit-lot-left` | باقی { $remaining } | { $remaining } left | |
| `fruit-parts-col-partner` | شراکت دار | Partner | |
| `fruit-parts-col-amount` | حصہ | Part | |
| `fruit-parts-col-now` | ابھی کی کل رقم | Partner Total now | |
| `fruit-parts-col-after` | اس کے بعد کی کل رقم | Partner Total after | |
| `fruit-field-reason` | تبدیلی کی وجہ | Why is it changed | |
| `fruit-number` | پھل-{ $number } | FRT-{ $number } | |
| `fruit-saved` | { $number } محفوظ ہو گیا | { $number } saved | |
| `fruit-edit` | ترمیم | Edit | |
| `fruit-editing` | { $number } میں ترمیم ہو رہی ہے۔ پرانا اندراج الٹ دیا جائے گا اور نیا اسی دن لگے گا۔ | Editing { $number }. The old entry is reversed and the new one is posted on the same day. | |
| `fruit-cancel` | اندراج منسوخ کریں | Cancel entry | |
| `fruit-cancel-title` | یہ اندراج منسوخ کریں؟ | Cancel this entry? | |
| `fruit-cancel-text` | { $number } الٹ دیا جائے گا: شراکت داروں کی کل رقم، لاٹ .gender = masculine کا باقی مال اور بیوپاری .gender = masculine کا کھاتہ پہلے جیسے ہو جائیں گے۔ یہ فہرست میں منسوخ کے طور پر رہے گا۔ | { $number } is reversed: the partners' totals, the lot's balance and the seller's account go back as they were. It stays in the list as cancelled. | |
| `fruit-cancel-reason` | وجہ | Reason | |
| `fruit-cancel-confirm` | اندراج منسوخ کریں | Cancel entry | |
| `fruit-cancelled-toast` | { $number } منسوخ ہو گیا | { $number } cancelled | |
| `fruit-entry-title` | اندراج { $number } | Entry { $number } | |
| `fruit-entry-failed` | یہ اندراج نہیں کھل سکا | This entry could not be opened | |
| `fruit-entry-close` | بند کریں | Close | |
| `fruit-entry-parts` | حصے | Parts | |
| `fruit-entry-cancelled` | یہ اندراج منسوخ ہو چکا ہے۔ وجہ: { $reason } | This entry was cancelled. Reason: { $reason } | |
| `fruit-list-title` | پھل | Fruit taken | |
| `fruit-list-label` | لیے گئے پھل کی فہرست | List of fruit taken | |
| `fruit-list-failed` | فہرست نہیں کھل سکی | The list could not be opened | |
| `fruit-list-empty` | ابھی کوئی پھل درج نہیں ہوا | No fruit recorded yet | |
| `fruit-list-empty-help` | اوپر کا فارم بھر کر محفوظ کریں۔ ہر اندراج یہاں نظر آئے گا۔ | Fill in the form above and save. Each entry shows here. | |
| `fruit-list-no-match` | ان انتخابات سے کوئی اندراج نہیں ملا | No entry matches these choices | |
| `fruit-list-no-match-help` | دوسرا شراکت دار، قسم مال یا مدت آزمائیں۔ | Try another partner, item or period. | |
| `fruit-list-clear` | سب دکھائیں | Show everything | |
| `fruit-col-number` | نمبر | Number | |
| `fruit-col-date` | تاریخ | Date | |
| `fruit-col-for` | کس کے لیے | For | |
| `fruit-col-item` | قسم مال اور پیکنگ .gender = feminine | Item and Packing | |
| `fruit-col-qty` | نگ | Nag | |
| `fruit-col-weight` | وزن (کلو) | Weight (kg) | |
| `fruit-from-lot` | لاٹ .gender = masculine { $lot } | Lot { $lot } | |
| `fruit-col-rate` | ریٹ | Rate | |
| `fruit-col-value` | قیمت | Value | |
| `fruit-col-source` | کہاں سے | From where | |
| `fruit-col-from` | کس سے | From | |
| `fruit-for-cancelled` | { $name }، منسوخ | { $name }, cancelled | |
| `fruit-filter-for` | شراکت دار یا مہمان | Partner or guests | |
| `fruit-filter-for-all` | سب | Everyone | |
| `fruit-filter-item` | فہرست میں قسم مال | Item in the list | |
| `fruit-filter-item-all` | سب قسمیں | All items | |
| `fruit-filter-from` | از | From | |
| `fruit-filter-to` | تا | To | |
| `fruit-period-today` | آج | Today | |
| `fruit-period-month` | یہ مہینہ | This month | |
| `fruit-period-year` | یہ سال | This year | |
| `fruit-period-all` | سب دن | All days | |
| `fruit-print` | فہرست پرنٹ کریں | Print the list | |
| `fruit-list-select-help` | کسی اندراج پر جا کر Enter دبائیں تو اس میں تبدیلی ہو سکتی ہے۔ | Move to an entry and press Enter to change it. | |

## gardens

| ID | Urdu | English | Result |
|---|---|---|---|
| `gardens-title` | باغات | Gardens | |
| `gardens-help` | کچھ شراکت دار ایک موسم کے لیے باغ ٹھیکے پر لیتے ہیں۔ وہ ٹھیکہ کی رقم قسط وار ادا کرتے ہیں اور باغ کے خرچ بھی دیتے ہیں۔ اس کا پھل دکان پر عام کمیشن .gender = masculine کے ساتھ بکتا ہے، یا کسی اور دکان میں۔ باغ کا نفع یا نقصان صرف انہی شراکت داروں کا ہوتا ہے جو اس میں شامل ہیں۔ | Some partners rent a garden for a season. They pay the rent (Garden Rent) in Installments and pay its running costs. Its fruit is sold in the shop with the normal Commission, or in another shop. The profit or loss of a garden goes only to the partners in it. | |
| `gardens-new` | نیا باغ | New garden | |
| `gardens-load-failed` | نئے باغ کا فارم نہیں کھل سکا | The form of a new garden could not be opened | |
| `gardens-list-label` | باغات کی فہرست | List of gardens | |
| `gardens-list-from` | از | From | |
| `gardens-list-to` | تا | To | |
| `gardens-list-status` | دکھائیں | Show | |
| `gardens-list-search` | باغ تلاش کریں | Find a garden | |
| `gardens-state-all` | سب | All | |
| `gardens-state-open` | کھلے | Open | |
| `gardens-state-closed` | بند | Closed | |
| `gardens-list-failed` | باغات کی فہرست نہیں کھل سکی | The list of gardens could not be opened | |
| `gardens-list-empty` | ابھی کوئی باغ نہیں | No garden yet | |
| `gardens-list-empty-help` | بٹن دبا کر پہلا باغ بنائیں۔ اس میں شامل شراکت دار چنیں، پھر ٹھیکہ اور ادا شدہ رقم درج کریں۔ | Press the button to make the first garden. Choose the partners in it, then record the rent and what is paid. | |
| `gardens-list-no-match` | ان انتخابات سے کوئی باغ نہیں ملا | No garden matches these choices | |
| `gardens-list-no-match-help` | دوسری تاریخیں، کھلے یا بند باغ، یا دوسرے الفاظ آزمائیں۔ | Try other days, open or closed gardens, or other words. | |
| `gardens-list-clear` | سب باغ دکھائیں | Show all gardens | |
| `gardens-number` | باغ-{ $number } | GRD-{ $number } | |
| `gardens-col-number` | نمبر | Number | |
| `gardens-col-name` | نام | Name | |
| `gardens-col-date` | تاریخ | Date | |
| `gardens-col-status` | حالت | State | |
| `gardens-col-balance` | چلتا حساب | Running balance | |
| `gardens-col-rent-due` | باقی ٹھیکہ | Rent still due | |
| `gardens-col-rent` | ٹھیکہ | Garden Rent | |
| `gardens-col-paid` | ادا شدہ | Paid | |
| `gardens-col-money-in` | آنے والی رقم | Money in | |
| `gardens-col-entry` | اندراج | Entry | |
| `gardens-col-effect` | رقم | Amount | |
| `gardens-col-running-balance` | اس کے بعد کا حساب | Balance after | |
| `gardens-col-state` | منسوخ | Cancelled | |
| `gardens-status-open` | کھلا | Open | |
| `gardens-status-closed` | بند | Closed | |
| `gardens-status-closed-on` | بند ہوا { $date } کو | Closed on { $date } | |
| `gardens-entry-cancelled` | منسوخ | Cancelled | |
| `gardens-kind-installment` | قسط | Installment | |
| `gardens-kind-cost` | باغ خرچ | Garden cost | |
| `gardens-kind-other-sale` | دوسری دکان میں فروخت | Sale in another shop | |
| `gardens-kind-sale` | اپنی دکان میں فروخت | Sale in our shop | |
| `gardens-kind-fruit` | پھل | Fruit Taken | |
| `gardens-kind-close` | بندش | Closing | |
| `gardens-history-entry` | { $kind }: { $detail } | { $kind }: { $detail } | |
| `gardens-history-detail-sale-elsewhere` | { $shop }، { $qty } نگ | { $shop }, { $qty } Nag | |
| `gardens-history-detail-bikri` | بکری .gender = feminine { $number } | Sale { $number } | |
| `gardens-new-title` | نیا باغ | New garden | |
| `gardens-new-help` | پہلے چنیں کہ اس باغ میں کون سے شراکت دار ہیں اور ہر ایک کا کتنے فیصد ہے۔ پھر بتائیں کہ ہر شراکت دار کی رقم کہاں سے آئے گی۔ آخر میں تفصیل بھریں۔ | First choose which partners are in this garden and the % of each. Then say where each partner's money comes from. At the end fill in the details. | |
| `gardens-new-save` | باغ بنائیں | Make the garden | |
| `gardens-new-shares-title` | شراکت دار اور حصے | Partners and shares | |
| `gardens-new-from-defaults` | فرم کے حصہ .gender = masculine یا پچھلے باغ کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔ | Filled in from the firm's shares or the last garden. Check it and change what is different. | |
| `gardens-new-shares-help` | باغ میں شامل شراکت داروں کے حصے ٹھیک 100 فیصد ہونے چاہییں۔ جس کا حصہ نہ ہو وہ اس میں شامل نہیں۔ پہلے اندراج کے بعد حصے بدلے نہیں جا سکتے۔ | The shares of the partners in the garden must total exactly 100%. A partner with no share is not in it. The shares cannot be changed after the first entry. | |
| `gardens-new-sources-help` | ہر شراکت دار کی ہر ادائیگی کا حصہ دکان کی رقم سے آتا ہے یا اس کی اپنی جیب سے۔ دکان کی رقم سے ادا ہونے والا حصہ اس کی کل رقم سے کٹتا ہے اور روکڑ سے نکلتا ہے۔ اپنی جیب سے ادا ہونے والا حصہ صرف درج ہوتا ہے: کچھ پوسٹ نہیں ہوتا۔ | Each partner's part of a payment comes from the shop's cash or from his own pocket. A part from the shop's cash comes off his total and out of Cash Book. A part from his own pocket is only recorded: nothing is posted. | |
| `gardens-new-no-partners` | ابھی کوئی شراکت دار نہیں بنا، اس لیے باغ نہیں بن سکتا۔ | No partner has been made yet, so a garden cannot be made. | |
| `gardens-new-no-partners-link` | شراکت داروں کی ترتیب کھولیں | Open the partner setup | |
| `gardens-new-details-title` | تفصیل | Details | |
| `gardens-field-name-ur` | اردو نام | Urdu name | |
| `gardens-field-name-en` | انگریزی نام | English name | |
| `gardens-field-date` | تاریخ | Date | |
| `gardens-field-item-hint` | اصل پھل، اگر کوئی ہو۔ | The main fruit, if there is one. | |
| `gardens-field-rent` | پورا ٹھیکہ | Full Garden Rent | |
| `gardens-field-rent-hint` | موسم کے لیے باغ کی پوری قیمت، پورے روپوں میں۔ | The full price of the garden for the season, in whole rupees. | |
| `gardens-field-season-end` | موسم کا اختتام | End of the season | |
| `gardens-field-paid-now` | پہلی قسط | First Installment | |
| `gardens-field-paid-now-hint` | آج کچھ ادا نہ ہو تو خالی چھوڑ دیں۔ یہ پہلی قسط کے طور پر محفوظ ہوگی۔ | Leave it empty if nothing is paid today. It is saved as the first installment. | |
| `gardens-field-paid-now-reason` | پہلی قسط کیوں دی گئی | Why the first installment was paid | |
| `gardens-field-paid-now-reason-hint` | خود بھر دیا گیا ہے۔ ٹھیک نہ ہو تو بدل دیں۔ | Filled in for you. Change it if it is not right. | |
| `gardens-first-installment-reason` | پہلی قسط | First installment | |
| `gardens-field-note` | نوٹ | Note | |
| `gardens-first-parts-title` | پہلی قسط میں ہر شراکت دار کا حصہ | Each partner's part of the first installment | |
| `gardens-preview-failed` | اعداد نہیں نکل سکے | The numbers could not be worked out | |
| `gardens-created` | { $number } بن گیا | { $number } has been made | |
| `gardens-back` | باغات پر واپس | Back to the gardens | |
| `gardens-detail-title` | { $number } | { $number } | |
| `gardens-detail-failed` | باغ نہیں کھل سکا | The garden could not be opened | |
| `gardens-none` | کوئی نہیں | None | |
| `gardens-closed-on` | بند ہوا | Closed on | |
| `gardens-balance` | چلتا حساب (آنے والی رقم منفی کل ادائیگی) | Running balance (money in less everything paid) | |
| `gardens-rent-due` | باقی ٹھیکہ | Rent still due | |
| `gardens-figures-title` | باغ کے اعداد | The garden in figures | |
| `gardens-partners-title` | شراکت دار | Partners | |
| `gardens-shares-locked` | اس باغ میں اندراج یا بکری ہو چکی ہے، اس لیے حصے اب طے ہیں۔ | This garden has an entry or a sale, so the shares are fixed now. | |
| `gardens-fact-rent` | ٹھیکہ | Garden Rent | |
| `gardens-fact-installments` | قسط وار ادا شدہ | Paid in Installments | |
| `gardens-fact-costs` | باغ کے خرچ میں ادا شدہ | Paid as garden costs | |
| `gardens-fact-paid` | کل ادا شدہ | Everything paid | |
| `gardens-fact-paid-shop` | دکان کی رقم سے ادا شدہ | Paid from the shop's cash | |
| `gardens-fact-paid-pocket` | جیبوں سے ادا شدہ | Paid from pockets | |
| `gardens-fact-money-in` | آنے والی رقم (کمیشن .gender = masculine کے بعد فروخت) | Money in (sales after Commission) | |
| `gardens-fact-other-shop` | اس میں سے دوسری دکانوں کی فروخت | Of it, from sales in other shops | |
| `gardens-fact-account` | اس وقت باغ کے کھاتے میں | In the garden's account now | |
| `gardens-closed-notice` | یہ باغ { $date } کو بند ہو چکا ہے۔ اس میں نیا اندراج، نیا لاٹ یا پھل نہیں لیا جا سکتا۔ اگر اس کی بندش واپس لی جائے تو یہ دوبارہ کھل جاتا ہے۔ | This garden was closed on { $date }. It takes no new entry, no lot and no fruit. If its closing is undone, it opens again. | |
| `gardens-partner-results-title` | ہر شراکت دار کی رقم | Each partner's money | |
| `gardens-partner-results-empty` | ابھی کوئی شراکت دار نہیں۔ | No partner yet. | |
| `gardens-partner-paid-shop` | دکان سے ادا | Paid from the shop | |
| `gardens-partner-paid-pocket` | اپنی جیب سے ادا | Paid from his pocket | |
| `gardens-partner-back` | واپس ملنے والی رقم | Money back | |
| `gardens-partner-net-open` | اب تک کا نتیجہ | Net so far | |
| `gardens-partner-net-closed` | آخری نتیجہ | Net result | |
| `gardens-partner-loss` | نقصان | Loss | |
| `gardens-type-label` | اندراج کی قسم | Kind of entry | |
| `gardens-type-installment` | قسط | Installment | |
| `gardens-type-cost` | باغ خرچ | Garden cost | |
| `gardens-type-other-sale` | دوسری دکان میں فروخت | Sale in another shop | |
| `gardens-entry-new` | نیا اندراج | New entry | |
| `gardens-sources-title` | رقم کہاں سے آئے گی | Where the money comes from | |
| `gardens-sources-hint` | باغ کے مطابق بھرا گیا ہے۔ اگر کوئی شراکت دار اس اندراج میں کسی اور طرح ادا کرے تو بدل دیں۔ اپنی جیب سے ادا ہونے والا حصہ کچھ پوسٹ نہیں کرتا۔ | Filled in from the garden. If a partner pays in another way on this entry, change it. A part from his own pocket posts nothing. | |
| `gardens-parts-title` | ہر شراکت دار کا حصہ | Each partner's part | |
| `gardens-installment-amount` | قسط کی رقم | Installment amount | |
| `gardens-cost-amount` | خرچ کی رقم | Cost amount | |
| `gardens-payment-amount-hint` | پورے روپے۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 20000+5000۔ | Whole rupees. You can type a sum, for example 20000+5000. | |
| `gardens-payment-reason` | وجہ | Why | |
| `gardens-installment-reason-hint` | مثلاً: ٹھیکے کی پہلی قسط۔ | For example: first Installment of the rent. | |
| `gardens-cost-reason-hint` | مثلاً: مزدوری، سپرے، ٹرانسپورٹ۔ | For example: labour, spray, transport. | |
| `gardens-sale-shop` | دکان | Shop | |
| `gardens-sale-qty` | بکے ہوئے نگ | Nag sold | |
| `gardens-sale-qty-hint` | پورا عدد۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 20+25۔ | A whole number. You can type a sum, for example 20+25. | |
| `gardens-sale-amount` | ملنے والی رقم | Money received | |
| `gardens-sale-amount-hint` | رقم روکڑ میں آتی ہے اور باغ بند ہونے تک باغ کے کھاتے میں رہتی ہے۔ | The money comes into Cash Book and goes to the garden's account until the garden is closed. | |
| `gardens-saved-installment` | قسط { $number } محفوظ ہو گئی | Installment { $number } has been saved | |
| `gardens-saved-cost` | باغ خرچ { $number } محفوظ ہو گیا | Garden cost { $number } has been saved | |
| `gardens-saved-other-sale` | دوسری دکان میں فروخت { $number } محفوظ ہو گئی | Sale in another shop { $number } has been saved | |
| `gardens-numbers-title` | یہ اندراج | This entry | |
| `gardens-numbers-empty` | فارم بھریں تو اعداد یہاں نظر آئیں گے۔ | Fill in the form and the figures show here. | |
| `gardens-numbers-incomplete` | فارم مکمل ہوگا تو اعداد نظر آئیں گے۔ | The figures show when the form is complete. | |
| `gardens-numbers-amount` | رقم | Amount | |
| `gardens-numbers-balance-before` | اس سے پہلے کا چلتا حساب | Running balance before | |
| `gardens-numbers-balance-after` | اس کے بعد کا چلتا حساب | Running balance after | |
| `gardens-numbers-rent-due-after` | اس کے بعد باقی ٹھیکہ | Rent still due after | |
| `gardens-history-title` | تاریخچہ | History | |
| `gardens-history-label` | باغ کا تاریخچہ | History of the garden | |
| `gardens-history-empty` | ابھی کچھ درج نہیں ہوا | Nothing has been recorded yet | |
| `gardens-history-empty-help` | اس باغ کی قسط، خرچ اور فروخت یہاں نظر آتی ہیں۔ اوپر کا فارم بھریں اور { $key } دبائیں۔ | The Installment, costs and sales of this garden show here. Fill in the form above and press { $key }. | |
| `gardens-history-count` | { $total } میں سے { $shown } نظر آ رہے ہیں | Showing { $shown } of { $total } | |
| `gardens-history-more` | مزید دکھائیں | Show more | |
| `gardens-history-cancel` | اندراج منسوخ کریں | Cancel the entry | |
| `gardens-entry-cancelled-reason` | یہ اندراج منسوخ ہو چکا ہے۔ وجہ: { $reason } | This entry was cancelled. Reason: { $reason } | |
| `gardens-cancel-title` | یہ اندراج منسوخ کریں؟ | Cancel this entry? | |
| `gardens-cancel-text` | { $entry } منسوخ ہو جائے گا۔ اس کی پوسٹنگ الٹ دی جائے گی، ہر شراکت دار کا حصہ جیسا تھا ویسا ہو جائے گا، اور اندراج منسوخ کے نشان کے ساتھ تاریخچے میں رہے گا۔ | { $entry } will be cancelled. Its posting is reversed, each partner's part goes back as it was, and the entry stays in the history, marked cancelled. | |
| `gardens-cancel-reason` | وجہ | Reason | |
| `gardens-cancel-confirm` | اندراج منسوخ کریں | Cancel the entry | |
| `gardens-cancel-confirm-cash` | ہاں، منسوخ کریں | Yes, cancel | |
| `gardens-cancelled` | { $entry } کو منسوخ کر دیا گیا۔ | { $entry } has been cancelled. | |
| `gardens-close-title` | باغ بند کریں | Close the garden | |
| `gardens-close-help` | جب پھل آنا بند ہو جائے تو باغ بند کریں۔ اس کی رقم شراکت داروں میں ان کے حصوں کے مطابق بٹ جاتی ہے: ہر ایک کو اس کی ادا کی ہوئی رقم واپس ملتی ہے، اور نفع یا نقصان میں اس کا حصہ جمع یا منفی ہوتا ہے۔ اس کے بعد باغ میں مزید اندراج نہیں ہوتا۔ | When the goods stop coming, close the garden. Its money goes to the partners by their shares: each gets back what he paid, plus or minus his share of the profit or loss. After that the garden takes no more entries. | |
| `gardens-close-date` | بندش کا دن | Closing day | |
| `gardens-close-earliest` | { $date } سے پہلے نہیں، یہ باغ کا آخری اندراج ہے۔ | Not before { $date }, the last entry of the garden. | |
| `gardens-close-numbers-empty` | بندش کا دن چنیں تو اعداد یہاں نظر آئیں گے۔ | Choose the closing day and the figures show here. | |
| `gardens-close-money-in` | آنے والی رقم | Money in | |
| `gardens-close-total-cost` | کل لاگت | Total cost | |
| `gardens-close-balance` | شراکت داروں کو ملنے والی رقم | Goes to the partners | |
| `gardens-close-loss-note` | باغ کو نقصان ہوا۔ ہر شراکت دار کا نقصان کا حصہ نیچے اس کے آخری نتیجے میں ہے۔ | The garden made a loss. Each partner's part of the loss is in his net result below. | |
| `gardens-close-credits-title` | ہر شراکت دار کی واپس ملنے والی رقم | Each partner's money back | |
| `gardens-close-nothing-posted` | باغ کے کھاتے میں کچھ نہیں، اس لیے کچھ پوسٹ نہیں ہوگا۔ باغ پھر بھی بند ہو جائے گا۔ | Nothing is in the garden's account, so nothing is posted. The garden still closes. | |
| `gardens-close-button` | باغ بند کریں | Close the garden | |
| `gardens-close-confirm-title` | یہ باغ بند کریں؟ | Close this garden? | |
| `gardens-close-confirm-text` | باغ { $name } { $date } کو بند ہو جائے گا۔ { $balance } روپے اس کے شراکت داروں میں ان کے حصوں کے مطابق بٹ جائیں گے۔ اس کے بعد اس میں نیا اندراج، نیا لاٹ یا پھل نہیں لیا جا سکے گا۔ جب تک اس دن کا حساب کھلا ہے بندش واپس لی جا سکتی ہے۔ | The garden { $name } will be closed on { $date }. { $balance } goes to its partners by their shares. After this it takes no more entries, no lot and no fruit. The closing can be undone while its day is open. | |
| `gardens-close-confirm` | ہاں، بند کریں | Yes, close it | |
| `gardens-closed-toast` | { $number } بند ہو گیا | { $number } has been closed | |
| `gardens-reopen-button` | بندش واپس لیں | Undo the closing | |
| `gardens-reopen-title` | اس باغ کی بندش واپس لیں؟ | Undo the closing of this garden? | |
| `gardens-reopen-text` | بندش الٹ دی جائے گی: رقم واپس باغ کے کھاتے میں چلی جائے گی اور شراکت داروں کی کل رقم سے نکل جائے گی، اور باغ دوبارہ کھل جائے گا۔ بندش تاریخچے میں رہے گی۔ | The closing is reversed: the money goes back into the garden's account and out of the partners' totals, and the garden opens again. The closing stays in the history. | |
| `gardens-reopen-reason` | وجہ | Reason | |
| `gardens-reopen-confirm` | بندش واپس لیں | Undo the closing | |
| `gardens-reopened-toast` | { $number } دوبارہ کھل گیا | { $number } is open again | |
| `gardens-filter-all` | تمام باغات | All gardens | |
| `gardens-filter-failed` | باغات کی فہرست نہیں کھل سکی | The list of gardens could not be read | |

## layout

| ID | Urdu | English | Result |
|---|---|---|---|
| `layout-nav-label` | مرکزی مینو | Main menu | |
| `layout-nav-dashboard` | ڈیش بورڈ | Dashboard | |
| `layout-nav-index` | انڈکس | Index | |
| `layout-nav-bikri` | بکری .gender = feminine | Sale | |
| `layout-nav-rokar` | روکڑ | Cash Book | |
| `layout-nav-gardens` | باغات | Gardens | |
| `layout-nav-trips` | مارکیٹ سفر .gender = masculine | Market Trip | |
| `layout-nav-partners` | شراکت دار | Partners | |
| `layout-nav-analytics` | تجزیات | Analytics | |
| `layout-nav-reports` | رپورٹ | Reports | |
| `layout-nav-backup` | بیک اپ | Backup | |
| `layout-nav-settings` | ترتیبات | Settings | |
| `layout-module-label` | بکری .gender = feminine کا مینو | Sale menu | |
| `layout-module-bikri-book` | بکری بک | Sale Book | |
| `layout-module-receipt` | وصولی .gender = feminine / اگراہی .gender = feminine | Receipt / Collection | |
| `layout-module-payment` | ادائیگی .gender = feminine | Payment | |
| `layout-module-advance` | پیشگی کھاتہ .gender = masculine | Advance | |
| `layout-module-pukhta` | پختہ کھاتہ .gender = masculine | Pukhta Khata | |
| `layout-module-today` | آج کی بکری .gender = feminine | Today's Sale | |
| `layout-module-stock` | باقی مال | Unsold Stock | |
| `layout-search-label` | تلاش | Search | |
| `layout-search-placeholder` | کوڈ، نام، بکری .gender = feminine نمبر یا ٹرک تلاش کریں | Search by code, name, Sale number or truck | |
| `layout-user-menu` | صارف کا مینو | User menu | |
| `layout-status-user` | صارف: <name>{ $name }</name> | User: <name>{ $name }</name> | |
| `layout-status-date` | تاریخ: <num>{ $date }</num> | Date: <num>{ $date }</num> | |
| `layout-status-day-open` | دن کھلا | Day open | |
| `layout-status-day-closed` | دن بند | Day closed | |
| `layout-status-closed-up-to` | بند: <num>{ $date }</num> تک | Closed up to <num>{ $date }</num> | |
| `layout-status-backup-none` | کلاؤڈ بیک اپ: ابھی ترتیب نہیں | Cloud backup: not set up yet | |
| `layout-stepper-label` | مراحل | Steps | |
| `layout-save-title` | محفوظ کریں؟ | Save changes? | |
| `layout-save-description` | اس صفحے پر ایسی تبدیلیاں ہیں جو محفوظ نہیں ہوئیں۔ | This page has changes that are not saved. | |
| `layout-save-leave` | محفوظ کیے بغیر جائیں | Leave without saving | |
| `layout-save-save` | محفوظ کریں | Save | |
| `layout-user-change-password` | پاس ورڈ بدلیں | Change password | |
| `layout-user-lock` | ایپ لاک کریں | Lock the app | |
| `layout-user-signout` | لاگ آؤٹ | Sign out | |
| `layout-loading-session` | ایپ کھل رہی ہے… | Opening the app… | |
| `layout-loading-failed` | ایپ نہیں کھل سکی | The app could not open | |
| `layout-search-results` | تلاش کے نتائج | Search results | |
| `layout-search-searching` | تلاش جاری ہے… | Searching… | |
| `layout-search-none` | کوئی کھاتہ نہیں ملا۔ | No account found. | |
| `layout-search-failed` | تلاش نہیں ہو سکی۔ | The search could not be done. | |
| `layout-rokar-label` | روکڑ کا مینو | Cash Book menu | |
| `layout-rokar-cashbook` | روکڑ | Cash Book | |
| `layout-rokar-expense` | خرچہ .gender = masculine | Expense | |
| `layout-rokar-lent` | ادھار رقم .gender = feminine | Money Lent | |
| `layout-rokar-khata` | الگ کھاتہ .gender = masculine | Separate Khata | |
| `layout-rokar-voucher` | وصولی .gender = feminine / ادائیگی .gender = feminine / پیشگی کھاتہ .gender = masculine | Receipt / Payment / Advance | |
| `layout-rokar-general` | جنرل | Journal | |
| `layout-user-guide` | رہنمائی دوبارہ دیکھیں | Show the guide | |
| `guide-progress` | مرحلہ <num>{ $step }</num> از <num>{ $total }</num> | Step <num>{ $step }</num> of <num>{ $total }</num> | |
| `guide-next` | آگے | Next | |
| `guide-back` | پیچھے | Back | |
| `guide-skip` | رہنمائی چھوڑیں | Skip the guide | |
| `guide-done` | مکمل | Done | |
| `guide-step-dashboard-title` | ڈیش بورڈ | The dashboard | |
| `guide-step-dashboard-body` | آج کی تصویر ایک نظر میں: بکری، روکڑ، بقایا، شراکت دار اور توجہ طلب کام۔ کسی بھی خانے کو دبائیں تو اس کے پیچھے کی اسکرین کھل جائے گی۔ | Today at a glance: sales, cash, dues, partners and what needs your attention. Press any tile to open the screen behind it. | |
| `guide-step-index-title` | انڈکس | Index | |
| `guide-step-index-body` | یہاں اپنے بیوپاری، گاہک، قسم مال اور شہر لکھیں۔ ہر کھاتے کا ایک کوڈ ہوتا ہے۔ کہیں بھی کوڈ، یا نام کے چند حروف لکھیں تو پارٹی سامنے آ جاتی ہے۔ | Add your sellers, buyers, items and cities here. Every account has a code. Type a code, or a few letters of a name, anywhere to bring up the party. | |
| `guide-step-bikri-title` | بکری بک | Sale Book | |
| `guide-step-bikri-body` | ایک ٹرک لکھیں: اس کے لاٹ، پھر ہر گاہک، تعداد اور ریٹ۔ خلاصہ اور شراکت داروں کے حصے لکھتے ہی بدلتے جاتے ہیں۔ محفوظ کرنے کے لیے <key>{ $key }</key> دبائیں۔ | Enter a truck: its lots, then each buyer with the quantity and the rate. The summary and the partners' shares change as you type. Save with <key>{ $key }</key>. | |
| `guide-step-receipt-title` | وصولی .gender = feminine | Receipt | |
| `guide-step-receipt-body` | گاہک کی دی ہوئی رقم لکھیں۔ اس کے کھلے بل سب سے پرانے سے شروع ہو کر دکھائے جاتے ہیں اور رقم پہلے سب سے پرانے بل میں لگتی ہے۔ | Record the money a buyer pays. His open bills show, the oldest first, and the money settles the oldest bill first. | |
| `guide-step-reports-title` | رپورٹ | Reports | |
| `guide-step-reports-body` | ہر رپورٹ میں تاریخ اور پارٹی کے فلٹر ہوتے ہیں، اور اسے پرنٹ یا ایکسل میں محفوظ کیا جا سکتا ہے۔ | Every report has dates and party filters, and can be printed or saved to Excel. | |

## masters

| ID | Urdu | English | Result |
|---|---|---|---|
| `masters-menu-title` | انڈکس | Index | |
| `masters-menu-help` | یہ وہ فہرستیں ہیں جن سے بکری .gender = feminine یا کوئی اور اندراج کرتے وقت نام چنے جاتے ہیں۔ کوئی فہرست کھولیں، پھر نیا نام شامل کریں یا پرانا بدلیں۔ | These are the lists that names are picked from when you enter a Sale or any other entry. Open a list, then add a new one or change an old one. | |
| `masters-menu-back` | انڈکس پر جائیں | Go to the Index | |
| `masters-unknown-title` | یہ فہرست موجود نہیں | This list does not exist | |
| `masters-unknown-help` | انڈکس سے فہرست چنیں۔ | Pick a list from the Index. | |
| `masters-city-title` | شہروں کی فہرست | City list | |
| `masters-city-help` | وہ شہر جہاں سے مال اور گاہک .gender = masculine آتے ہیں۔ | The cities that goods and buyers come from. | |
| `masters-city-new` | نیا شہر | New city | |
| `masters-city-empty` | { masters-city-title } ابھی خالی ہے | { masters-city-title } is empty | |
| `masters-item-title` | قسم مال کی فہرست | Item list | |
| `masters-item-help` | پھل اور سبزی کی قسمیں، جیسے کیلا اور مولی۔ | The kinds of fruit and vegetable, such as banana and radish. | |
| `masters-item-new` | نئی قسم مال | New Item | |
| `masters-item-empty` | { masters-item-title } ابھی خالی ہے | { masters-item-title } is empty | |
| `masters-packing-title` | پیکنگ .gender = feminine کی فہرست | Packing list | |
| `masters-packing-help` | مال کی پیکنگ .gender = feminine کی قسمیں، جیسے پیٹی اور کریٹ۔ | The ways goods are packed, such as box and crate. | |
| `masters-packing-new` | نئی پیکنگ .gender = feminine | New Packing | |
| `masters-packing-empty` | { masters-packing-title } ابھی خالی ہے | { masters-packing-title } is empty | |
| `masters-adda-title` | اڈا .gender = masculine کی فہرست | Adda list | |
| `masters-adda-help` | وہ اڈے جہاں سے ٹرک آتے ہیں۔ | The truck stands that trucks come from. | |
| `masters-adda-new` | نیا اڈا .gender = masculine | New Adda | |
| `masters-adda-empty` | { masters-adda-title } ابھی خالی ہے | { masters-adda-title } is empty | |
| `masters-marka-title` | مارکہ .gender = masculine کی فہرست | Mark list | |
| `masters-marka-help` | پیٹیوں پر لگے نشان جو بتاتے ہیں کہ مال کس کا ہے۔ | The marks on crates that show whose goods they are. | |
| `masters-marka-new` | نیا مارکہ .gender = masculine | New Mark | |
| `masters-marka-empty` | { masters-marka-title } ابھی خالی ہے | { masters-marka-title } is empty | |
| `masters-store-title` | سٹوروں کی فہرست | Store list | |
| `masters-store-help` | وہ سٹور جہاں مال رکھا جاتا ہے۔ | The stores where goods are kept. | |
| `masters-store-new` | نیا سٹور | New store | |
| `masters-store-empty` | { masters-store-title } ابھی خالی ہے | { masters-store-title } is empty | |
| `masters-narration-title` | تفصیل کی فہرست | Narration list | |
| `masters-narration-help` | وہ جملے جو واؤچروں میں بار بار لکھے جاتے ہیں۔ | The sentences that are written again and again in vouchers. | |
| `masters-narration-new` | نئی تفصیل | New narration | |
| `masters-narration-empty` | { masters-narration-title } ابھی خالی ہے | { masters-narration-title } is empty | |
| `masters-expense-head-title` | خرچہ .gender = masculine مد کی فہرست | Expense head list | |
| `masters-expense-head-help` | دکان کے خرچوں کی مدیں، جیسے بجلی اور چائے۔ | The heads of the shop's expenses, such as electricity and tea. | |
| `masters-expense-head-new` | نئی خرچہ .gender = masculine مد | New expense head | |
| `masters-expense-head-empty` | { masters-expense-head-title } ابھی خالی ہے | { masters-expense-head-title } is empty | |
| `masters-staff-title` | عملے کی فہرست | Staff list | |
| `masters-staff-help` | منشی، اگراہی .gender = feminine والے اور مزدور۔ | Munshis, collectors and labourers. | |
| `masters-staff-new` | عملے کا نیا فرد | New staff member | |
| `masters-staff-empty` | { masters-staff-title } ابھی خالی ہے | { masters-staff-title } is empty | |
| `masters-filter-label` | اس فہرست میں تلاش | Search this list | |
| `masters-filter-placeholder` | کوڈ یا نام لکھیں | Type a code or a name | |
| `masters-show-inactive` | غیر فعال بھی دکھائیں | Show the inactive ones too | |
| `masters-col-code` | کوڈ | Code | |
| `masters-col-name` | نام | Name | |
| `masters-col-other-name` | انگریزی نام | Urdu name | |
| `masters-col-status` | حالت | Status | |
| `masters-status-active` | فعال | Active | |
| `masters-status-inactive` | غیر فعال | Inactive | |
| `masters-empty-help` | نیا شامل کرنے کے لیے { $key } دبائیں۔ | Press { $key } to add a new one. | |
| `masters-no-match-title` | کوئی نام نہیں ملا | Nothing found | |
| `masters-no-match-help` | کوئی اور کوڈ یا نام لکھ کر دیکھیں، یا غیر فعال بھی دکھائیں۔ | Try another code or name, or show the inactive ones too. | |
| `masters-no-match-clear` | تلاش صاف کریں | Clear the search | |
| `masters-load-failed` | فہرست نہیں کھل سکی | The list could not be opened | |
| `masters-filter-failed` | تلاش نہیں ہو سکی | The search could not be done | |
| `masters-not-found-title` | یہ نام نہیں ملا | This one was not found | |
| `masters-not-found-help` | ہو سکتا ہے یہ بدل چکا ہو۔ فہرست دوبارہ کھولیں۔ | It may have been changed. Open the list again. | |
| `masters-edit-title` | { $name } بدلیں | Change { $name } | |
| `masters-field-code` | کوڈ | Code | |
| `masters-field-code-hint` | اگلا کوڈ پہلے سے لکھا ہوا ہے۔ آپ اسے بدل سکتے ہیں۔ | The next code is already filled in. You can change it. | |
| `masters-field-name-ur` | اردو نام | Urdu name | |
| `masters-field-name-en` | انگریزی نام | English name | |
| `masters-narration-text-ur` | اردو متن | Urdu text | |
| `masters-narration-text-en` | انگریزی متن | English text | |
| `masters-account-code` | کھاتے کا کوڈ | Account code | |
| `masters-account-code-hint` | یہ کھاتہ محفوظ کرتے وقت خود بن جاتا ہے۔ | This account is made by itself when you save. | |
| `masters-saved` | محفوظ ہو گیا | Saved | |
| `masters-make-inactive` | غیر فعال کریں | Make inactive | |
| `masters-make-active` | فعال کریں | Make active | |
| `masters-made-inactive` | غیر فعال کر دیا گیا | Made inactive | |
| `masters-made-active` | فعال کر دیا گیا | Made active | |
| `masters-inactive-title` | غیر فعال کریں؟ | Make this inactive? | |
| `masters-inactive-description` | { $name } فہرستوں میں نہیں دکھایا جائے گا، لیکن پرانے اندراج اور رپورٹوں میں رہے گا۔ جب چاہیں دوبارہ فعال کر سکتے ہیں۔ | { $name } will not be offered in the lists, but it stays in the old entries and the reports. You can make it active again at any time. | |
| `masters-inactive-note` | یہ غیر فعال ہے، اس لیے فہرستوں میں نہیں دکھایا جاتا۔ دوبارہ استعمال کرنے کے لیے فعال کریں۔ | This is inactive, so it is not offered in the lists. Make it active to use it again. | |
| `masters-item-group` | گروپ | Group | |
| `masters-item-group-fruit` | پھل | Fruit | |
| `masters-item-group-vegetable` | سبزی | Vegetable | |
| `masters-item-packing` | طے شدہ پیکنگ .gender = feminine | Default Packing | |
| `masters-item-packing-hint` | نئے لاٹ .gender = masculine میں یہی پیکنگ .gender = feminine پہلے سے لکھی ہوگی۔ | A new Lot starts with this Packing. | |
| `masters-item-sold-by` | بکنے کا طریقہ | Sold by | |
| `masters-sold-by-unit` | نگ کے حساب سے | By Nag | |
| `masters-sold-by-weight` | وزن کے حساب سے | By weight | |
| `masters-packing-weight` | تقریباً وزن (کلو) | Approximate weight (kg) | |
| `masters-packing-weight-hint` | مثلاً 20۔ معلوم نہ ہو تو خالی چھوڑیں۔ | For example 20. Leave it empty if you do not know. | |
| `masters-adda-city` | شہر | City | |
| `masters-marka-text` | مارکہ .gender = masculine کی تحریر | Mark text | |
| `masters-marka-text-hint` | جیسا پیٹی پر لکھا ہے ویسا ہی لکھیں۔ یہ ترجمہ نہیں ہوتا۔ | Write it as it is written on the crate. It is not translated. | |
| `masters-marka-owner` | بیوپاری .gender = masculine | Seller | |
| `masters-marka-owner-hint` | وہ بیوپاری .gender = masculine جس کے مال پر یہ مارکہ .gender = masculine لگتا ہے۔ خالی بھی چھوڑ سکتے ہیں۔ | The Seller whose goods carry this Mark. You can leave it empty. | |
| `masters-narration-used-in` | استعمال کی جگہ | Used in | |
| `masters-narration-used-in-hint` | جن واؤچروں میں یہ تفصیل پیش ہو، ان پر نشان لگائیں۔ کسی پر نشان نہ لگائیں تو یہ ہر واؤچر میں پیش ہوگی۔ | Tick the vouchers where this narration is offered. Tick none and it is offered in every voucher. | |
| `masters-staff-job` | کام | Job | |
| `masters-staff-job-munshi` | منشی | Munshi | |
| `masters-staff-job-collector` | اگراہی .gender = feminine والا | Collector | |
| `masters-staff-job-labour` | مزدور | Labourer | |
| `masters-staff-phone` | فون نمبر | Phone number | |
| `masters-staff-account` | ادھار رقم .gender = feminine کا کھاتہ | Money Lent account | |
| `masters-staff-account-hint` | جب اسے پہلی بار ادھار رقم .gender = feminine دی جائے گی تو یہ کھاتہ خود بن جائے گا۔ | This account is made by itself the first time money is lent to this person. | |

## month

| ID | Urdu | English | Result |
|---|---|---|---|
| `month-title` | مہینہ بند | Month Close | |
| `month-help` | مہینے کے آخر میں اس کی آمدن اور دکان خرچہ .gender = masculine شراکت داروں کے سرمایہ کھاتوں میں چلے جاتے ہیں، ہر شراکت دار اپنے حصہ .gender = masculine کے مطابق۔ ہر شراکت دار کی کل رقم وہی رہتی ہے۔ | At the end of a month, its income and its Shop Expense move into the partners' capital accounts, each partner by his Share. The Partner Total of every partner stays the same. | |
| `month-open` | مہینہ بند | Month Close | |
| `month-load-failed` | مہینے کھل نہیں سکے | The months could not be opened | |
| `month-empty` | ابھی کوئی مہینہ نہیں | There is no month yet | |
| `month-empty-help` | پہلا اندراج محفوظ ہونے کے بعد مہینہ یہاں آ جاتا ہے۔ | A month appears here after the first entry is saved. | |
| `month-list-label` | مہینے | Months | |
| `month-col-month` | مہینہ | Month | |
| `month-col-state` | حالت | State | |
| `month-col-closed-on` | بند ہوا | Closed on | |
| `month-col-reversed` | واپس کیا گیا | Reversed | |
| `month-state-closed` | بند | Closed | |
| `month-state-ready` | بند کرنے کے لیے تیار | Ready to close | |
| `month-state-days-open` | دن ابھی کھلے ہیں | Days still open | |
| `month-state-not-finished` | مہینہ ابھی ختم نہیں ہوا | Not finished | |
| `month-preview-title` | { $month } بند کرنا | Closing { $month } | |
| `month-preview-help` | بند کرنے سے ایک اندراج بنتا ہے جو یہ آمدن اور یہ خرچے شراکت داروں کے سرمایہ کھاتوں میں منتقل کرتا ہے۔ جب تک آپ بند کریں نہ دبائیں، کچھ نہیں بنتا۔ | Closing posts one entry that moves this income and these expenses into the partners' capital accounts. Nothing is posted until you press Close. | |
| `month-fact-income` | آمدن | Income | |
| `month-fact-expense` | دکان خرچہ .gender = masculine | Shop Expense | |
| `month-fact-profit` | شراکت داروں میں بٹنے والا منافع | Profit shared between the partners | |
| `month-fact-date` | اندراج کی تاریخ | The entry is dated | |
| `month-lines-label` | بندش کے اندراج کی سطریں | The lines of the closing entry | |
| `month-col-account` | کھاتہ | Account | |
| `month-col-kind` | کیا ہے | What it is | |
| `month-col-debit` | بنام | Debit | |
| `month-col-credit` | جمع | Credit | |
| `month-total` | میزان | Total | |
| `month-kind-income` | آمدن کھاتہ | Income account | |
| `month-kind-expense` | دکان خرچہ کھاتہ | Shop expense account | |
| `month-kind-partner-capital` | شراکت دار کا سرمایہ | Partner capital | |
| `month-close-button` | مہینہ بند کریں | Close the month | |
| `month-close-confirm-title` | { $month } بند کریں؟ | Close { $month }? | |
| `month-close-confirm-description` | { $month } کی آمدن اور دکان خرچہ .gender = masculine ایک اندراج کے ذریعے، جس کی تاریخ { $date } ہے، شراکت داروں کے سرمایہ کھاتوں میں چلے جائیں گے۔ کسی شراکت دار کی کل رقم نہیں بدلے گی۔ بعد میں وجہ لکھ کر یہ بندش واپس کی جا سکتی ہے۔ | The income and the Shop Expense of { $month } will be moved into the partners' capital accounts by one entry dated { $date }. No Partner Total changes. You can reverse the close later with a reason. | |
| `month-closed-toast` | { $month } بند ہو گیا | { $month } is closed | |
| `month-closed-title` | { $month } بند ہے | { $month } is closed | |
| `month-closed-help` | اس کی آمدن اور دکان خرچہ .gender = masculine شراکت داروں کے سرمایہ کھاتوں میں جا چکے ہیں۔ دوبارہ بند کرنے کے لیے پہلے یہ بندش واپس کریں۔ | Its income and Shop Expense are already in the partners' capital accounts. To close it again, reverse this close first. | |
| `month-closed-on` | بند ہوا | Closed on | |
| `month-reversed-before` | { $count -> [1] اس مہینے کی بندش پہلے ایک بار واپس کی گئی *[other] اس مہینے کی بندش پہلے { $count } بار واپس کی گئی | { $count -> [1] A close of this month was reversed once before *[other] A close of this month was reversed { $count } times before | |
| `month-reverse-button` | بندش واپس کریں | Reverse the close | |
| `month-reverse-title` | { $month } کی بندش واپس کریں | Reverse the close of { $month } | |
| `month-reverse-description` | آج کی تاریخ کا ایک اندراج { $month } کی بندش ختم کر دے گا۔ وجہ آڈٹ لاگ میں رہتی ہے۔ اس کے بعد مہینہ دوبارہ بند ہو سکتا ہے۔ | One entry dated today will undo the close of { $month }. The reason is kept in the audit log. The month can be closed again after it. | |
| `month-reverse-reason` | وجہ | Reason | |
| `month-reverse-confirm` | واپس کریں | Reverse | |
| `month-reversed-toast` | { $month } کی بندش واپس ہو گئی | The close of { $month } is reversed | |
| `month-open-day-close` | دن بند کرنے کی اسکرین کھولیں | Open Day Close | |
| `month-error-not-finished` | { $month } ابھی ختم نہیں ہوا۔ اس کے آخری دن کے بعد بند ہو سکتا ہے۔ | { $month } is not finished yet. It can be closed after its last day. | |
| `month-error-days-open` | { $month } کا ہر دن پہلے بند ہونا ضروری ہے۔ دن { $closed_up_to } تک بند ہیں۔ دن بند کریں، پھر واپس آئیں۔ | Every day of { $month } must be closed first. The days are closed up to { $closed_up_to }. Close the days, then come back. | |
| `month-error-already-closed` | { $month } پہلے ہی بند ہے۔ دوبارہ بند کرنے کے لیے پہلے بندش واپس کریں۔ | { $month } is already closed. To close it again, reverse the close first. | |
| `month-error-nothing-to-close` | { $month } میں نہ آمدن ہے نہ دکان خرچہ .gender = masculine، اس لیے بند کرنے کے لیے کچھ نہیں۔ | { $month } has no income and no Shop Expense, so there is nothing to close. | |
| `month-error-not-in-force` | { $month } کی کوئی بندش نہیں جو واپس کی جائے۔ فہرست دوبارہ کھولیں۔ | { $month } has no close to reverse. Open the list again. | |

## partners

| ID | Urdu | English | Result |
|---|---|---|---|
| `partners-title` | شراکت دار | Partners | |
| `partners-overview-help` | ہر شراکت دار کا آج، اس مہینے اور اس سال کا حصہ .gender = masculine۔ | Each partner's share for today, this month and this year. | |
| `partners-setup-open` | شراکت دار کی ترتیب | Partner setup | |
| `partners-share-now` | موجودہ حصہ .gender = masculine | Current Share | |
| `partners-metric-today` | آج کا حصہ .gender = masculine | Today's Share | |
| `partners-metric-month` | مہینے کا حصہ .gender = masculine | This month's Share | |
| `partners-metric-year` | سال کا حصہ .gender = masculine | This year's Share | |
| `partners-metric-expense` | خرچہ .gender = masculine حصہ .gender = masculine، اس مہینے | Expense Share, this month | |
| `partners-metric-profit` | منافع حصہ .gender = masculine، اس مہینے | Profit Share, this month | |
| `partners-metric-taken` | ذاتی خرچہ .gender = masculine، اس مہینے | Personal Drawing, this month | |
| `partners-metric-capital` | سرمایہ بقایا | Capital balance | |
| `partners-load-failed` | شراکت دار کی معلومات نہیں کھل سکیں۔ | The partner details could not be loaded. | |
| `partners-overview-empty` | ابھی کوئی شراکت دار نہیں | No partners yet | |
| `partners-overview-empty-help` | شراکت دار شامل کریں اور ان کے حصہ .gender = masculine لکھیں، تب ہر بکری .gender = feminine کی آمدن ان میں بٹے گی۔ | Add the partners and type their shares. Then the income of every sale is split between them. | |
| `partners-overview-empty-action` | شراکت دار شامل کریں | Add partners | |
| `partners-setup-title` | شراکت دار کی ترتیب | Partner setup | |
| `partners-setup-help` | ہر شراکت دار کا حصہ .gender = masculine فیصد میں لکھیں۔ کل ٹھیک 100 فیصد ہونا چاہیے۔ | Type each partner's share as a percent. The total must be exactly 100%. | |
| `partners-list-label` | شراکت دار اور ان کے حصہ .gender = masculine | Partners and their shares | |
| `partners-column-partner` | شراکت دار | Partner | |
| `partners-column-share` | حصہ .gender = masculine (فیصد) | Share (percent) | |
| `partners-share-of` | { $name } کا حصہ .gender = masculine (فیصد) | Share of { $name } (percent) | |
| `partners-blank-hint` | ہر شراکت دار کا حصہ .gender = masculine لکھیں۔ جس کا حصہ .gender = masculine نہیں، اس کے لیے 0 لکھیں۔ | Type a share for every partner. Type 0 for a partner with no share. | |
| `partners-edit` | تفصیل | Details | |
| `partners-edit-of` | { $name } کی تفصیل بدلیں | Change the details of { $name } | |
| `partners-add` | نیا شراکت دار | Add partner | |
| `partners-inactive-title` | غیر فعال شراکت دار | Inactive partners | |
| `partners-badge-inactive` | غیر فعال | Inactive | |
| `partners-field-from` | حصہ .gender = masculine کب سے لاگو ہوں | Shares apply from | |
| `partners-field-from-hint` | آج یا اس کے بعد کی تاریخ لکھیں۔ پرانی تاریخ نہیں چلتی۔ پہلے سے محفوظ بکری .gender = feminine کا بٹوارہ نہیں بدلتا۔ | Today or a later date. A past date is not allowed. Sales already saved keep their split. | |
| `partners-total-label` | کل حصہ .gender = masculine | Total Share | |
| `partners-total-ok` | پورے 100 فیصد ہیں | Adds up to 100% | |
| `partners-total-bad` | کل ٹھیک 100 فیصد ہونا چاہیے | The total must be exactly 100% | |
| `partners-total-checking` | جانچ ہو رہی ہے… | Checking… | |
| `partners-save` | حصہ .gender = masculine محفوظ کریں | Save shares | |
| `partners-saved` | حصہ .gender = masculine محفوظ ہو گئے۔ یہ <num>{ $date }</num> سے لاگو ہیں۔ | Shares saved. They apply from <num>{ $date }</num>. | |
| `partners-added` | { $name } کو شراکت دار میں شامل کر دیا گیا۔ | { $name } was added as a partner. | |
| `partners-updated` | { $name } کی تفصیل محفوظ ہو گئی۔ | The details of { $name } were saved. | |
| `partners-setup-empty` | ابھی کوئی شراکت دار نہیں | No partners yet | |
| `partners-setup-empty-help` | پہلے شراکت دار کا نام لکھیں۔ پھر اس کا حصہ .gender = masculine فیصد میں لکھیں۔ مثال: الف، ب، ج۔ | Add the first partner's name, then type the share as a percent. For example: Partner A, Partner B, Partner C. | |
| `partners-setup-failed` | شراکت دار کی فہرست نہیں کھل سکی۔ | The partner list could not be loaded. | |
| `partners-add-title` | نیا شراکت دار | Add a partner | |
| `partners-add-help` | نام لکھیں۔ حصہ .gender = masculine بعد میں فہرست میں لکھا جاتا ہے۔ | Enter the name. The share is typed in the list afterwards. | |
| `partners-field-name-ur` | اردو نام | Name (Urdu) | |
| `partners-field-name-en` | انگریزی نام | Name (English) | |
| `partners-field-capital` | ابتدائی سرمایہ | Opening Capital | |
| `partners-field-capital-hint` | اختیاری، پورے روپوں میں۔ یا شراکت داروں کا ابتدائی سرمایہ لکھیں، یا فرم کا ابتدائی سرمایہ کھاتہ استعمال کریں، دونوں نہیں، ورنہ ایک ہی رقم دو بار گنی جائے گی۔ | Optional, in whole rupees. Use either the partners' opening capital or the firm's Opening Capital account, not both, or the same money is counted twice. | |
| `partners-add-save-more` | محفوظ کر کے اگلا | Save and add another | |
| `partners-edit-title` | شراکت دار کی تفصیل | Partner details | |
| `partners-field-active` | فعال | Active | |
| `partners-field-active-hint` | غیر فعال شراکت دار کا ریکارڈ رہتا ہے، مگر پہلے اس کا حصہ .gender = masculine 0 ہونا چاہیے۔ | An inactive partner's history is kept, but his share must be 0 first. | |
| `partners-history-title` | حصہ .gender = masculine کی تاریخ | Share history | |
| `partners-history-row` | <num>{ $date }</num> سے <num>{ $share }</num> فیصد | From <num>{ $date }</num>: <num>{ $share }</num>% | |
| `partners-history-empty` | ابھی کوئی حصہ .gender = masculine مقرر نہیں ہوا۔ | No share has been set yet. | |
| `partners-metric-returned` | واپسی .gender = feminine، اس مہینے | Return, this month | |
| `partners-metric-net` | خالص .gender = masculine، اس مہینے | Net, this month | |
| `partners-metric-capital-in` | سرمایہ لگایا، اس مہینے | Capital put in, this month | |
| `partners-metric-ventures` | باغ، سفر اور پھل، اس مہینے | Garden, Trip and Fruit Taken, this month | |
| `partners-metric-total` | کل رقم | Partner Total | |
| `partners-open-statement` | فرد حساب | Statement | |
| `partners-open-statement-of` | { $name } کی فرد حساب | Statement of { $name } | |
| `partners-open-capital` | سرمایہ | Capital | |
| `partners-open-capital-of` | { $name } کا سرمایہ | Capital of { $name } | |
| `partners-capital-open` | سرمایہ | Capital | |
| `partners-capital-screen-title` | سرمایہ | Capital | |
| `partners-capital-screen-help` | شراکت دار جو رقم دکان میں لگاتا ہے اور جو واپس لیتا ہے۔ یہ نہ آمدن ہے نہ خرچہ، اور کبھی تقسیم نہیں ہوتی۔ | Money a partner puts into the shop, and money he takes back. It is never income or expense and is never split. | |
| `partners-capital-type` | سرمایہ اندراج کی قسم | Kind of Capital entry | |
| `partners-capital-history-title` | سرمایہ کی تاریخ | Capital history | |
| `partners-capital-pick` | شراکت دار چنیں | Choose a partner | |
| `partners-capital-pick-help` | اس کے سرمایہ کی تاریخ اور بقایا یہاں نظر آئے گا۔ | His Capital history and balance show here. | |
| `partners-capital-balance` | سرمایہ بقایا | Capital balance | |
| `partners-capital-invested` | کل سرمایہ جمع | Total put in | |
| `partners-capital-taken-back` | کل سرمایہ واپسی | Total taken back | |
| `partners-capital-label` | شراکت دار کے سرمایہ کی تاریخ | Capital history of the partner | |
| `partners-capital-failed` | سرمایہ کی تاریخ نہیں کھل سکی | The Capital history could not be opened | |
| `partners-capital-empty` | ابھی کوئی سرمایہ اندراج نہیں | No Capital entries yet | |
| `partners-capital-empty-help` | اوپر پہلی رقم لکھیں اور محفوظ کریں۔ | Type the first amount above and save it. | |
| `partners-capital-col-balance` | سرمایہ بقایا | Capital balance | |
| `partners-capital-kind-opening` | ابتدائی سرمایہ | Opening Capital | |
| `partners-capital-kind-invested` | سرمایہ جمع | Put in | |
| `partners-capital-kind-taken-back` | سرمایہ واپسی | Taken back | |
| `partners-capital-kind-month-close` | مہینہ بند: منافع سرمایہ میں منتقل ہوا | Month close: profit moved into Capital | |
| `partners-statement-title` | شراکت دار کھاتہ .gender = masculine | Partner statement | |
| `partners-statement-help` | شراکت دار تک پہنچنے والی ہر رقم، تاریخ وار، چلتے ہوئے کل رقم کے ساتھ۔ | Every amount that reaches the partner, in date order, with his running Partner Total. | |
| `partners-statement-opening` | شروع میں کل رقم | Partner Total at the start | |
| `partners-statement-closing` | آخر میں کل رقم | Partner Total at the end | |
| `partners-statement-plus` | اضافہ | Added | |
| `partners-statement-minus` | کمی | Taken away | |
| `partners-statement-pick` | شراکت دار چنیں | Choose a partner | |
| `partners-statement-pick-help` | اس کی تفصیل یہاں نظر آئے گی۔ | His statement shows here. | |
| `partners-statement-empty` | ان دنوں میں کچھ نہیں | Nothing in these days | |
| `partners-statement-empty-help` | دوسرے دن چنیں، یا تاریخیں مٹا دیں۔ | Choose other days, or clear the dates. | |
| `partners-statement-label` | شراکت دار کھاتہ .gender = masculine کی سطریں | Lines of the partner statement | |
| `partners-statement-failed` | کھاتہ .gender = masculine نہیں کھل سکا | The statement could not be opened | |
| `partners-statement-col-heading` | ذریعہ | From | |
| `partners-statement-col-detail` | کیا تھا | What it was | |
| `partners-statement-col-running` | کل رقم | Partner Total | |
| `partners-statement-heading-bikri` | بکری .gender = feminine | Sale | |
| `partners-statement-heading-expense` | خرچہ .gender = masculine | Expense | |
| `partners-statement-heading-capital` | سرمایہ | Capital | |
| `partners-statement-heading-drawings` | خرچہ .gender = masculine | Expense | |
| `partners-statement-heading-trip` | مارکیٹ سفر .gender = masculine | Market Trip | |
| `partners-statement-heading-other` | دیگر | Other | |
| `partners-statement-detail-income-share` | بکری .gender = feminine { $number }: کمیشن .gender = masculine، لاگا .gender = masculine اور فون .gender = masculine | Sale { $number }: Commission, Laga and Phone Charge | |
| `partners-statement-detail-expense-share` | دکان خرچہ .gender = masculine { $voucher }: { $subject } | Shop Expense { $voucher }: { $subject } | |
| `partners-statement-detail-capital-opening` | ابتدائی سرمایہ | Opening Capital | |
| `partners-statement-detail-capital-in` | سرمایہ لگایا { $voucher } | Capital put in { $voucher } | |
| `partners-statement-detail-capital-out` | سرمایہ واپس لیا { $voucher } | Capital taken back { $voucher } | |
| `partners-statement-detail-month-close` | مہینہ بند: منافع سرمایہ میں منتقل ہوا۔ کل رقم وہی رہتا ہے۔ | Month close: profit moved into Capital. The total stays the same. | |
| `partners-statement-detail-taken` | ذاتی خرچہ .gender = masculine { $voucher }: { $reason } | Personal Drawing { $voucher }: { $reason } | |
| `partners-statement-detail-returned` | ذاتی خرچہ .gender = masculine واپسی .gender = feminine { $voucher } | Personal Drawing Return { $voucher } | |
| `partners-statement-detail-drawings-opening` | ابتدائی لی ہوئی رقم | Opening money taken | |
| `partners-statement-detail-trip-paid` | { $kind -> [purchase] مارکیٹ سفر .gender = masculine { $trip }: خریداری { $secondary }، دکان سے [cost] مارکیٹ سفر .gender = masculine { $trip }: سفر کا خرچ { $secondary }، دکان سے *[other] مارکیٹ سفر .gender = masculine { $trip }: { $secondary }، دکان سے | { $kind -> [purchase] Market Trip { $trip }: Purchase { $secondary }, From the Shop [cost] Market Trip { $trip }: trip cost { $secondary }, From the Shop *[other] Market Trip { $trip }: { $secondary }, From the Shop | |
| `partners-statement-detail-trip-credit` | { $source -> [bikri] مارکیٹ سفر .gender = masculine { $trip }، بکری .gender = feminine { $secondary }: { $subject } { $qty }، فروخت { $sales }، لاگت { $cost }، نتیجہ { $result }، آپ کا { $share } فیصد *[other] مارکیٹ سفر .gender = masculine { $trip }، دوسری دکان میں فروخت: { $subject } { $qty }، رقم { $sales }، لاگت { $cost }، نتیجہ { $result }، آپ کا { $share } فیصد | { $source -> [bikri] Market Trip { $trip }, Sale { $secondary }: { $subject } { $qty }, sales { $sales }, cost { $cost }, result { $result }, your { $share }% *[other] Market Trip { $trip }, sale in another shop: { $subject } { $qty }, money { $sales }, cost { $cost }, result { $result }, your { $share }% | |
| `partners-statement-detail-trip-write-off` | مارکیٹ سفر .gender = masculine { $trip }: سفر کا مال ضائع کیا گیا۔ کوئی اندراج نہیں ہوا؛ نقصان سفر کے نتیجے میں نظر آتا ہے۔ | Market Trip { $trip }: trip goods written off. Nothing is posted; the loss shows in the result of the trip. | |
| `partners-statement-detail-trip-closed` | مارکیٹ سفر .gender = masculine { $trip } بند: لاگت { $cost }، فروخت { $sales }، نتیجہ { $result }، آپ کا { $share } فیصد، آپ کا خالص .gender = masculine { $net } | Market Trip { $trip } closed: cost { $cost }, sales { $sales }, result { $result }, your { $share }%, your Net { $net } | |
| `partners-statement-heading-garden` | باغ | Garden | |
| `partners-statement-detail-garden-paid` | { $garden }: { $reason }، دکان سے | { $garden }: { $reason }, From the Shop | |
| `partners-statement-detail-garden-closed` | { $garden } بند: لاگت { $cost }، فروخت { $sales }، نتیجہ { $result }، آپ کا { $share } فیصد: { $part }، واپسی .gender = feminine { $back } | { $garden } closed: cost { $cost }, sales { $sales }, result { $result }, your { $share }%: { $part }, Return { $back } | |
| `partners-statement-heading-fruit` | پھل | Fruit Taken | |
| `partners-statement-detail-fruit-taken` | { $for -> [guests] مہمان کے لیے پھل: { $subject } { $qty } { $packing } @ { $rate } = { $value }، آپ کا برابر حصہ *[home] گھر کے لیے پھل: { $subject } { $qty } { $packing } @ { $rate } = { $value } | { $for -> [guests] Fruit Taken for Guests: { $subject } { $qty } { $packing } @ { $rate } = { $value }, your equal part *[home] Fruit Taken for home: { $subject } { $qty } { $packing } @ { $rate } = { $value } | |
| `partners-statement-detail-fruit-earlier` | پھل { $fruit }: ایسے اندراج کی پرانی پوسٹنگ جو بعد میں بدلا گیا، یا اس کی الٹ | Fruit Taken { $fruit }: an earlier posting of an entry that was changed, or its reversal | |
| `partners-statement-detail-other` | { $reason } | { $reason } | |
| `partners-ventures-title` | کھلے باغ / سفر | Open Garden / Trip | |
| `partners-ventures-label` | { $name } کے کھلے باغ / سفر | Open Garden / Trip of { $name } | |
| `partners-ventures-col-venture` | باغ / سفر | Garden / Trip | |
| `partners-ventures-col-kind` | قسم | Kind | |
| `partners-ventures-col-shop` | دکان سے ادا | Paid from the shop | |
| `partners-ventures-col-pocket` | اپنی جیب سے ادا | Paid from his pocket | |
| `partners-ventures-col-back` | واپس ملنے والی رقم | Money back | |
| `partners-ventures-col-balance` | چلتا حساب یا نتیجہ | Balance or result | |

## print

| ID | Urdu | English | Result |
|---|---|---|---|
| `print-seller-bill-title` | بیوپاری .gender = masculine کا بل | Seller Bill | |
| `print-buyer-slip-title` | گاہک .gender = masculine کی پرچی | Buyer Slip | |
| `print-pukhta-slip-title` | پختہ کھاتہ .gender = masculine کی پرچی | Pukhta Khata Slip | |
| `print-fact-number` | بکری .gender = feminine نمبر | Sale No. | |
| `print-fact-date` | تاریخ | Date | |
| `print-fact-bill-date` | تاریخ بل | Bill date | |
| `print-fact-truck` | ٹرک نمبر | Truck No. | |
| `print-party-seller` | بیوپاری .gender = masculine | Seller | |
| `print-party-buyer` | گاہک .gender = masculine | Buyer | |
| `print-party-onetime-buyer` | پختہ کھاتہ .gender = masculine (خریدار .gender = masculine) | Pukhta Khata (Buyer) | |
| `print-party-code` | کوڈ | Code | |
| `print-party-phone` | فون نمبر | Phone | |
| `print-gross-sale` | خام بکری .gender = feminine | Gross Sale | |
| `print-freight` | کرایہ .gender = masculine | Freight | |
| `print-commission` | کمیشن .gender = masculine | Commission | |
| `print-labour` | مزدوری .gender = feminine | Labour | |
| `print-munshiana` | منشیانہ | Munshiana | |
| `print-cash-paid` | نقد .gender = masculine | Cash Paid | |
| `print-store-rent` | کرایہ سٹور .gender = masculine | Store Rent | |
| `print-phone-charge` | فون .gender = masculine | Phone Charge | |
| `print-total-deductions` | کل اخراجات | Total deductions | |
| `print-net-sale` | صافی بکری .gender = feminine | Net Sale | |
| `print-seller-owes` | بیوپاری .gender = masculine کے ذمے | Seller owes | |
| `print-items-value` | مال کی قیمت | Value of goods | |
| `print-buyer-commission` | کمیشن .gender = masculine | Commission | |
| `print-laga` | لاگا .gender = masculine | Laga | |
| `print-buyer-pays` | کل رقم | Total amount | |
| `print-in-words` | رقم حروف میں | Amount in words | |
| `print-payment-cash` | نقدی بکری: رقم ادا ہو چکی ہے | Cash sale: paid | |
| `print-payment-credit` | ادھاری بکری: رقم باقی ہے | Credit sale: payment due | |
| `print-col-date` | تاریخ | Date | |
| `print-col-buyer` | گاہک .gender = masculine | Buyer | |
| `print-col-item` | قسم مال | Item | |
| `print-col-packing` | پیکنگ .gender = feminine | Packing | |
| `print-col-qty` | تعداد | Qty | |
| `print-col-weight` | وزن (کلو) | Weight (kg) | |
| `print-col-rate` | ریٹ | Rate | |
| `print-col-amount` | رقم | Amount | |
| `print-printed-at` | پرنٹ کا وقت | Printed | |
| `print-page-label` | صفحہ | Page | |
| `print-page-of` | از | of | |
| `print-cancelled-stamp` | منسوخ | Cancelled | |
| `print-preview-empty` | اس دستاویز میں چھاپنے کے لیے کچھ نہیں۔ | There is nothing to print in this document. | |
| `print-doc-seller-bill` | بیوپاری .gender = masculine کا بل | Seller bill | |
| `print-doc-buyer-slips` | گاہک .gender = masculine کی پرچیاں | Buyer slips | |
| `print-doc-pukhta-slip` | پختہ کھاتہ .gender = masculine کی پرچی | Pukhta Khata slip | |
| `print-box-title` | بکری .gender = feminine { $number } پرنٹ کریں | Print Sale { $number } | |
| `print-box-help` | یہاں جو دستاویزیں چنیں گے، وہی بکری .gender = feminine محفوظ کرتے وقت { $key } سے بھی پرنٹ ہوں گی۔ | The documents you tick here are also the ones { $key } prints when it saves a Sale. | |
| `print-box-language` | پرنٹ کی زبان | Print language | |
| `print-box-docs` | دستاویزیں | Documents | |
| `print-box-none-ticked` | کم از کم ایک دستاویز چنیں۔ | Tick at least one document. | |
| `print-box-only-buyer` | کس گاہک .gender = masculine کی پرچی | Buyer slip for | |
| `print-box-all-buyers` | سب گاہک .gender = masculine | All buyers | |
| `print-box-copies` | کاپیاں | Copies | |
| `print-box-copies-hint` | خالی چھوڑیں تو ترتیبات والی تعداد استعمال ہوگی۔ | Leave empty to use the number in Settings. | |
| `print-box-to` | کہاں بھیجنا ہے | Send to | |
| `print-box-to-printer` | پرنٹر | Printer | |
| `print-box-to-pdf` | PDF فائل | PDF file | |
| `print-box-print` | پرنٹ کریں | Print | |
| `print-box-save-pdf` | PDF محفوظ کریں | Save PDF | |
| `print-box-preview` | پیش منظر | Preview | |
| `print-box-close` | بند کریں | Close | |
| `print-box-done-printed` | پرنٹر کو بھیج دیا گیا: { $docs } | Sent to the printer: { $docs } | |
| `print-box-done-pdf` | PDF فائلیں محفوظ ہو گئیں: | PDF files saved: | |
| `print-box-nothing-printed` | چھاپنے کے لیے کچھ نہیں تھا۔ | There was nothing to print. | |
| `print-save-first` | پرنٹ کرنے کے لیے پہلے بکری .gender = feminine محفوظ کریں۔ | Save the Sale first, then print it. | |
| `print-toast-printed` | پرنٹر کو بھیج دیا گیا | Sent to the printer | |
| `print-preview-title` | پیش منظر | Preview | |
| `print-preview-help` | کاغذ پر دستاویز ایسے ہی چھپے گی۔ | The document will print on paper like this. | |
| `print-preview-failed` | پیش منظر نہیں کھل سکا | The preview could not be opened | |
| `print-settings-paper` | کاغذ کا سائز | Paper size | |
| `print-settings-paper-help` | ہر دستاویز کے لیے کاغذ چنیں۔ پرنٹ اسی سائز پر ہوگا۔ | Choose the paper for each document. It prints at this size. | |
| `print-paper-a4` | A4 کاغذ | A4 paper | |
| `print-paper-a5` | A5 کاغذ | A5 paper | |
| `print-paper-mm80` | 80 ملی میٹر رول | 80 mm roll | |
| `print-settings-printer` | پرنٹر | Printer | |
| `print-settings-printer-default` | طے شدہ پرنٹر | Default printer | |
| `print-settings-printer-none` | اس کمپیوٹر پر پرنٹروں کی فہرست نہیں ملی۔ طے شدہ پرنٹر استعمال ہوگا۔ | No list of printers is available on this computer. The default printer will be used. | |
| `print-settings-printers-failed` | پرنٹروں کی فہرست نہیں کھل سکی | The list of printers could not be read | |
| `print-settings-copies` | کاپیاں | Copies | |
| `print-settings-copies-hint` | 1 سے 99 تک کی تعداد۔ | A number from 1 to 99. | |
| `print-settings-urdu-digits` | اردو پرنٹ میں اردو ہندسے | Urdu digits on Urdu prints | |
| `print-settings-urdu-digits-hint` | بند ہو تو اردو پرنٹ میں بھی انگریزی ہندسے (0 سے 9) چھپتے ہیں۔ | When off, Urdu prints also use English digits (0 to 9). | |
| `print-settings-after-save` | محفوظ کرنے کے بعد پرنٹ | Print after save | |
| `print-settings-after-save-hint` | { $key } دبانے پر بکری .gender = feminine یا واؤچر محفوظ ہو کر یہ دستاویزیں پرنٹ ہوں گی۔ | { $key } saves the Sale or the voucher and prints these documents. | |
| `print-settings-load-failed` | پرنٹنگ کی ترتیبات نہیں کھل سکیں | Printing settings could not be opened | |
| `print-receipt-slip-title` | وصولی .gender = feminine کی رسید | Receipt slip | |
| `print-payment-slip-title` | ادائیگی .gender = feminine کی رسید | Payment slip | |
| `print-advance-slip-title` | پیشگی کھاتہ .gender = masculine کی رسید | Advance slip | |
| `print-fact-voucher-number` | نمبر | No. | |
| `print-fact-reason` | وجہ | Reason | |
| `print-fact-collector` | اگراہی والا | Collector | |
| `print-fact-narration` | تفصیل | Narration | |
| `print-party-onetime-seller` | پختہ کھاتہ .gender = masculine (بیوپاری .gender = masculine) | Pukhta Khata (Seller) | |
| `print-party-account` | کھاتہ | Account | |
| `print-slip-amount-received` | وصول شدہ رقم | Amount received | |
| `print-slip-amount-paid` | ادا کی گئی رقم | Amount paid | |
| `print-slip-amount-advance` | دی گئی پیشگی .gender = feminine | Advance given | |
| `print-doc-receipt-slip` | رقم کی رسید | Receipt slip | |
| `print-header-trade-mark` | ٹریڈ مارک | Trade Mark | |
| `print-r5-title` | ادھاری تفصیل | Credit details | |
| `print-r5-col-serial` | نمبر | No. | |
| `print-r5-col-party` | پارٹی | Party | |
| `print-r5-col-details` | تفصیل | Details | |
| `print-r5-col-value` | رقم | Amount | |
| `print-r5-col-total` | رقم بعد لاگا .gender = masculine | Amount after Laga | |
| `print-r5-line-weight` | ({ $item } { $weight } کلو . { $rate }) | ({ $item } { $weight } kg . { $rate }) | |
| `print-doc-collection-list` | اگراہی .gender = feminine فہرست | Collection list | |
| `print-collection-list-title` | اگراہی .gender = feminine فہرست | Collection list | |
| `print-fact-city` | شہر | City | |
| `print-collection-no-collector` | اگراہی .gender = feminine والا مقرر نہیں | No collector set | |
| `print-collection-no-city` | شہر درج نہیں | No city set | |
| `print-collection-col-serial` | نمبر | No. | |
| `print-collection-col-code` | کوڈ | Code | |
| `print-collection-col-party` | پارٹی | Party | |
| `print-collection-col-dues` | بقایا | Dues | |
| `print-collection-col-oldest` | پرانا بل | Oldest bill | |
| `print-collection-col-days` | دن | Days | |
| `print-collection-col-note` | نوٹ | Note | |
| `print-collection-group-total` | میزان | Total | |
| `print-collection-buyers` | اس فہرست میں گاہک .gender = masculine | Buyers on this list | |
| `print-collection-total` | اس فہرست کا میزان | Total of this list | |

## pukhta_khata

| ID | Urdu | English | Result |
|---|---|---|---|
| `pukhta-help` | ایک بار آنے والے گاہک .gender = masculine اور بیوپاری .gender = masculine کی سب لائنیں۔ نام یا فون لکھ کر ڈھونڈیں۔ | Every line of a one-time Buyer and Seller. Search by name or phone. | |
| `pukhta-load-failed` | پختہ کھاتہ .gender = masculine کی فہرست کھل نہیں سکی | The Pukhta Khata list could not be opened | |
| `pukhta-empty` | پختہ کھاتہ .gender = masculine میں ابھی کوئی لائن نہیں | No lines in Pukhta Khata yet | |
| `pukhta-empty-help` | جب کسی بکری .gender = feminine میں پختہ کھاتہ .gender = masculine استعمال ہو گا تو اس کی لائن یہاں آ جائے گی۔ | When a Sale uses Pukhta Khata, its line will show here. | |
| `pukhta-no-match` | کوئی لائن نہیں ملی | No line found | |
| `pukhta-no-match-help` | تاریخ، نام یا فون بدل کر دیکھیں۔ | Try changing the date, name or phone. | |
| `pukhta-clear` | چھانٹیاں ہٹائیں | Clear the filters | |
| `pukhta-register-label` | پختہ کھاتہ .gender = masculine کی فہرست | Pukhta Khata list | |
| `pukhta-filter-from` | تاریخ سے | From date | |
| `pukhta-filter-to` | تاریخ تک | To date | |
| `pukhta-filter-due` | باقی | Due | |
| `pukhta-all` | سب | All | |
| `pukhta-col-date` | تاریخ | Date | |
| `pukhta-col-bikri` | بکری .gender = feminine نمبر | Sale number | |
| `pukhta-col-side` | جانب | Side | |
| `pukhta-col-name` | نام | Name | |
| `pukhta-col-phone` | فون نمبر | Phone number | |
| `pukhta-col-item` | قسم مال | Item | |
| `pukhta-col-amount` | رقم | Amount | |
| `pukhta-col-state` | ادا / باقی | Paid / Due | |
| `pukhta-side-seller` | بیوپاری .gender = masculine | Seller | |
| `pukhta-side-buyer` | گاہک .gender = masculine | Buyer | |
| `pukhta-state-paid` | ادا | Paid | |
| `pukhta-state-due` | باقی { $amount } | Due { $amount } | |
| `pukhta-state-cancelled` | منسوخ | Cancelled | |
| `pukhta-make-regular-title` | باقاعدہ کھاتہ بنائیں | Make a regular account | |
| `pukhta-make-regular-help` | اس شخص کا نام اور فون یہاں بھر دیا گیا ہے۔ محفوظ کرنے پر اس کا کھلا بقایا نئے کھاتے میں چلا جائے گا۔ | The name and phone of this person are filled in. When you save, his open balance moves to the new account. | |
| `pukhta-make-regular-done` | کھاتہ بن گیا اور بقایا اس میں منتقل ہو گیا | The account is made and the balance moved to it | |

## purchases

| ID | Urdu | English | Result |
|---|---|---|---|
| `purchases-number` | خ-{ $number } | PUR-{ $number } | |
| `purchases-cancelled` | خریداری { $number } منسوخ ہو گئی | Purchase { $number } cancelled | |
| `purchases-market` | منڈی سے خریدا | Bought in the market | |
| `purchases-field-date` | تاریخ | Date | |
| `purchases-field-supplier` | کس سے خریدا | Bought from | |
| `purchases-supplier-hint` | جیسا آپ لکھنا چاہیں۔ اسے کچھ دینا نہیں ہوتا: خریداری اسی وقت ادا ہوتی ہے۔ | As you want it written. Nothing is owed to him: a purchase is paid at once. | |
| `purchases-field-qty` | خریدے ہوئے نگ | Nag bought | |
| `purchases-qty-hint` | آپ جمع بھی لکھ سکتے ہیں، مثلاً 20+25۔ | You can type a sum, for example 20+25. | |
| `purchases-field-weight` | وزن (کلو) | Weight (kg) | |
| `purchases-field-rate` | ریٹ | Rate | |
| `purchases-rate-hint` | ہر نگ کا، یا وزن سے بکنے والی قسم مال کے لیے ہر کلو کا۔ | For each Nag, or for each kg for an item sold by weight. | |
| `purchases-field-other-cost` | دیگر خرچ | Other cost | |
| `purchases-field-note` | نوٹ | Note | |
| `purchases-numbers-title` | لاگت | Cost | |
| `purchases-numbers-empty` | فارم بھریں تو لاگت یہاں نظر آئے گی۔ | Fill in the form and the cost shows here. | |
| `purchases-numbers-incomplete` | لاگت دیکھنے کے لیے قسم مال، پیکنگ .gender = feminine، مقدار اور ریٹ لکھیں۔ | Enter the Item, the Packing, the quantity and the rate to see the cost. | |
| `purchases-line-value` | مال کی قیمت | Value of the goods | |
| `purchases-line-total-cost` | کل لاگت | Total cost | |
| `purchases-line-avg-cost` | ایک نگ کی لاگت | Cost of one Nag | |
| `purchases-line-paid-now` | ابھی ادا شدہ | Paid now | |
| `purchases-col-number` | نمبر | No. | |
| `purchases-col-supplier` | کس سے خریدا | Bought from | |
| `purchases-col-item` | قسم مال اور پیکنگ .gender = feminine | Item and Packing | |
| `purchases-col-left` | باقی | Left | |
| `purchases-col-cost-left` | باقی کی لاگت | Cost left | |
| `purchases-col-date` | تاریخ | Day | |
| `purchases-col-qty` | خریدے | Bought | |
| `purchases-col-total-cost` | کل لاگت | Total cost | |
| `purchases-col-result` | منافع یا نقصان | Profit or loss | |
| `purchases-item-packing` | { $item }، { $packing } | { $item }, { $packing } | |
| `purchases-item-cancelled` | { $item } (منسوخ) | { $item } (cancelled) | |
| `purchases-detail-title` | { $number }، { $item } | { $number }, { $item } | |
| `purchases-detail-failed` | خریداری نہیں کھل سکی | The purchase could not be opened | |
| `purchases-detail-cancelled` | یہ خریداری منسوخ ہے۔ | This purchase is cancelled. | |
| `purchases-detail-cancel-reason` | وجہ: { $reason } | Reason: { $reason } | |
| `purchases-detail-written-off` | ضائع | Written off | |
| `purchases-detail-days` | انتظار کے دن | Days waiting | |
| `purchases-detail-trip-cost` | سفر کے خرچ کا حصہ | Part of the trip costs | |
| `purchases-detail-parts-title` | ہر شراکت دار نے کیا دیا | What each partner paid | |
| `purchases-result-title` | اس خریداری کا منافع یا نقصان | Profit or loss of this purchase | |
| `purchases-detail-sales` | ہماری دکان میں فروخت کی رقم | Sales money in our shop | |
| `purchases-detail-other-shop` | دوسری دکانوں سے ملی رقم | Money from other shops | |
| `purchases-detail-cost-sold` | بکے ہوئے مال کی لاگت | Cost of goods sold | |
| `purchases-detail-cost-written-off` | ضائع مال کی لاگت | Cost written off | |
| `purchases-loss-note` | اس میں نقصان ہو رہا ہے۔ | This is making a loss. | |
| `purchases-lots-title` | ان بکری .gender = feminine میں بکا | Sold in these sales | |
| `purchases-lot-bikri` | بکری .gender = feminine | Sale | |
| `purchases-lot-open` | بکری .gender = feminine { $number }، لاٹ .gender = masculine { $lot } | Sale { $number }, Lot { $lot } | |
| `purchases-lot-qty-sold` | بکے | Sold | |
| `purchases-lot-sales` | فروخت کی رقم | Sales money | |
| `purchases-write-offs-title` | ضائع | Written off | |
| `purchases-write-off` | ضائع کریں | Write off | |
| `purchases-undo-write-off` | ضائع واپس لیں | Undo a write-off | |
| `purchases-cancel` | خریداری منسوخ کریں | Cancel purchase | |
| `purchases-write-off-title` | سفر کا مال ضائع کریں | Write off trip goods | |
| `purchases-write-off-text` | { $number } ({ $item }، { $packing }) کا خراب یا گم مال ذخیرے سے نکال دیا جائے گا۔ ابھی { $qty_left } نگ باقی ہیں۔ کچھ پوسٹ نہیں ہوگا، کیونکہ شراکت داروں نے مال کی رقم پہلے ہی دے دی ہے۔ ضائع مال کی لاگت سفر کے نتیجے میں نقصان بن کر نظر آئے گی۔ | Goods of { $number } ({ $item }, { $packing }) that are spoiled or lost are taken off the stock. { $qty_left } Nag are left now. Nothing is posted, because the partners paid for the goods already. The cost of the goods written off shows as a loss in the result of the trip. | |
| `purchases-write-off-qty` | ضائع کرنے والے نگ | Nag to write off | |
| `purchases-write-off-reason` | وجہ (مثلاً خراب) | Reason (for example spoiled) | |
| `purchases-write-off-confirm` | ضائع کریں | Write off | |
| `purchases-write-off-done` | { $qty } نگ ضائع ہو گئے۔ نتیجے میں نقصان: <num>{ $cost }</num> | { $qty } Nag written off. Loss in the result: <num>{ $cost }</num> | |
| `purchases-write-off-done-closed` | { $qty } نگ ضائع ہو گئے۔ نتیجے میں نقصان: <num>{ $cost }</num>۔ اب کچھ باقی نہیں، اس لیے سفر بند ہو گیا۔ | { $qty } Nag written off. Loss in the result: <num>{ $cost }</num>. Nothing is left, so the trip is closed. | |
| `purchases-write-off-undo-title` | ضائع واپس لیں | Undo a write-off | |
| `purchases-write-off-undo-text` | مال واپس ذخیرے میں چلا جائے گا اور نقصان ختم ہو جائے گا۔ ضائع منتخب کریں اور وجہ لکھیں۔ | The goods go back to the stock and the loss is taken back. Choose the write-off and give the reason. | |
| `purchases-write-off-undo-which` | ضائع | Write-off | |
| `purchases-write-off-undo-option` | { $date }: { $qty } نگ ({ $reason }) | { $date }: { $qty } Nag ({ $reason }) | |
| `purchases-write-off-undo-confirm` | ضائع واپس لیں | Undo the write-off | |
| `purchases-write-off-undone` | ضائع واپس لے لیا گیا | The write-off was undone | |
| `purchases-write-off-undone-reopened` | ضائع واپس لے لیا گیا۔ سفر دوبارہ کھل گیا ہے۔ | The write-off was undone. The trip is open again. | |
| `purchases-cancel-title` | یہ خریداری منسوخ کریں؟ | Cancel this purchase? | |
| `purchases-cancel-text` | { $number } ({ $qty } نگ { $item }، { $packing }، ادا شدہ <num>{ $cost }</num>، { $supplier } سے) منسوخ ہو جائے گی۔ شراکت داروں نے دکان کی رقم سے جو دیا تھا وہ واپس آ جائے گا اور مال ذخیرے سے نکل جائے گا۔ | { $number } ({ $qty } Nag { $item }, { $packing }, paid <num>{ $cost }</num>, from { $supplier }) will be cancelled. What the partners paid from the shop's cash goes back, and the stock is taken off. | |
| `purchases-cancel-reason` | وجہ | Reason | |
| `purchases-cancel-confirm` | خریداری منسوخ کریں | Cancel purchase | |
| `purchases-cancel-confirm-cash` | ہاں، منسوخ کریں | Yes, cancel it | |
| `purchases-cancel-blocked` | یہ خریداری منسوخ نہیں ہو سکتی، کیونکہ { $sold } نگ بک چکے ہیں اور { $written_off } نگ ضائع ہو چکے ہیں۔ پہلے بکری .gender = feminine منسوخ کریں یا ضائع واپس لیں۔ | This purchase cannot be cancelled, because { $sold } Nag were sold and { $written_off } Nag were written off. Cancel the sales or undo the write-off first. | |
| `purchases-cancel-blocked-cost` | اس خریداری میں سفر کے خرچ کا حصہ شامل ہے، اس لیے یہ منسوخ نہیں ہو سکتی۔ پہلے وہ خرچ منسوخ کریں۔ | This purchase carries a part of a trip cost, so it cannot be cancelled. Cancel that cost first. | |

## reports

| ID | Urdu | English | Result |
|---|---|---|---|
| `reports-title` | رپورٹ | Reports | |
| `reports-help` | ایک رپورٹ چنیں۔ تاریخیں لکھیں تو نتیجہ فوراً سامنے آ جاتا ہے۔ ہر رپورٹ پرنٹ ہو سکتی ہے۔ | Pick a report. Enter the dates and the result shows at once. Every report can be printed. | |
| `reports-catalog-failed` | رپورٹوں کی فہرست نہیں کھل سکی | The list of reports could not be opened | |
| `reports-catalog-empty` | ابھی کوئی رپورٹ تیار نہیں | No report is ready yet | |
| `reports-catalog-empty-help` | رپورٹیں اگلے ورژن میں آئیں گی۔ اس وقت آپ کا کوئی ڈیٹا ضائع نہیں ہوتا۔ | The reports come in a later version. None of your data is affected. | |
| `reports-group-label` | رپورٹ کا گروپ: { $group } | Report group: { $group } | |
| `reports-open-report` | { $title } کھولیں | Open { $title } | |
| `report-group-sales` | بکری .gender = feminine | Sale | |
| `report-group-accounts` | حسابات | Accounts | |
| `report-group-collections` | اگراہی .gender = feminine، ادائیگی .gender = feminine اور پیشگی .gender = feminine | Collection, Payment and Advance | |
| `report-group-business` | کاروبار | Business | |
| `report-r11-title` | روزنامچہ .gender = masculine | Day Book | |
| `report-r11-help` | چنے ہوئے دنوں کی ہر اندراج، بنام اور جمع کے ساتھ۔ | Every entry of the days you choose, with Debit and Credit. | |
| `report-r13-title` | کھاتہ .gender = masculine | Ledger | |
| `report-r13-help` | ایک کھاتے کی ہر اندراج، سامان کی تفصیل اور چلتے بقایا کے ساتھ۔ | Every entry of one account, with the goods and the running balance. | |
| `report-r14-title` | فرد حساب | Statement | |
| `report-r14-help` | ایک کھاتے کا خلاصہ: ابتدائی بقایا، لین دین، ادائیگیاں اور آخری بقایا۔ | A summary of one account: opening balance, dealings, payments and closing balance. | |
| `report-r16-title` | کھاتوں کا انڈکس | Account Index | |
| `report-r16-help` | تمام کھاتے، شہر اور بقایا کے ساتھ۔ | All accounts with their city and balance. | |
| `report-r17-title` | مدوں بیلنس | Head Balances | |
| `report-r17-help` | ہر مد اور ہر کنٹرول کا بقایا۔ | The balance of every head and control. | |
| `report-r18-title` | ٹرائل بیلنس .gender = masculine | Trial Balance | |
| `report-r18-help` | ہر کھاتے کا بقایا۔ دونوں طرف کا میزان برابر ہونا چاہیے۔ | The balance of every account. Both sides must give the same Total. | |
| `report-r19-title` | اوپن ٹرائل | Opening Trial Balance | |
| `report-r19-help` | صرف ابتدائی بقایا، دونوں طرف کے میزان کے ساتھ۔ | Opening balances only, with the Total of both sides. | |
| `report-back` | رپورٹوں کی فہرست | All reports | |
| `report-load-failed` | رپورٹ نہیں کھل سکی | The report could not be opened | |
| `report-grid-label` | رپورٹ کی جدول | Report table | |
| `report-empty` | ان تاریخوں میں کوئی اندراج نہیں | No entries for these filters | |
| `report-empty-help` | تاریخیں یا کھاتہ بدل کر دیکھیں۔ | Try other dates or another account. | |
| `report-clear-filters` | فلٹر ہٹائیں | Clear the filters | |
| `report-pick-party` | پہلے کھاتہ چنیں | Pick an account first | |
| `report-pick-party-help` | یہ رپورٹ ایک کھاتے کی ہوتی ہے۔ اس کا کوڈ یا نام اوپر لکھیں۔ | This report is for one account. Type its code or name above. | |
| `report-balance-only-help` | صرف بقایا دکھائے گئے ہیں۔ ہر اندراج دیکھنے کے لیے "تفصیل کے ساتھ" پر نشان لگائیں۔ | Only the balances are shown. Tick "With detail" to see every line. | |
| `report-count` | <num>{ $total }</num> میں سے <num>{ $shown }</num> قطاریں دکھائی گئی ہیں | Showing <num>{ $shown }</num> of <num>{ $total }</num> rows | |
| `report-more` | مزید قطاریں | Show more rows | |
| `report-open-hint` | ↑ ↓ سے قطار چنیں اور Enter سے اس کا اصل اندراج کھولیں۔ | Use ↑ ↓ to pick a row and Enter to open its source. | |
| `report-preview` | پرنٹ پر نظر | Print preview | |
| `report-filters-label` | رپورٹ کے فلٹر | Report filters | |
| `report-filter-period` | مدت | Period | |
| `report-period-custom` | تاریخیں خود لکھیں | Choose the dates | |
| `report-period-today` | آج | Today | |
| `report-period-yesterday` | کل | Yesterday | |
| `report-period-this-week` | یہ ہفتہ | This week | |
| `report-period-this-month` | یہ مہینہ | This month | |
| `report-period-last-month` | پچھلا مہینہ | Last month | |
| `report-filter-from` | از | From | |
| `report-filter-to` | تا | To | |
| `report-filter-as-of` | اس تاریخ تک | As of | |
| `report-filter-party` | کھاتہ | Account | |
| `report-filter-seller` | بیوپاری .gender = masculine | Seller | |
| `report-filter-zamindar` | زمیندار .gender = masculine | Zamindar | |
| `report-filter-item` | قسم مال | Item | |
| `report-filter-city` | شہر | City | |
| `report-filter-adda` | اڈا .gender = masculine | Adda | |
| `report-filter-detail` | تفصیل کے ساتھ | With detail | |
| `report-filter-balance-only` | صرف بقایا | Balances only | |
| `report-range-from-to` | <num>{ $from }</num> سے <num>{ $to }</num> تک | From <num>{ $from }</num> to <num>{ $to }</num> | |
| `report-range-from` | <num>{ $from }</num> سے | From <num>{ $from }</num> | |
| `report-range-to` | <num>{ $to }</num> تک | Up to <num>{ $to }</num> | |
| `report-range-all` | تمام تاریخیں | All dates | |
| `report-as-of-shown` | <num>{ $date }</num> تک کا بقایا | Balances as of <num>{ $date }</num> | |
| `report-as-of-now` | آج تک کا بقایا | Balances up to now | |
| `report-total` | میزان | Total | |
| `report-opening-balance` | ابتدائی بقایا | Opening balance | |
| `report-closing-balance` | آخری بقایا | Closing balance | |
| `report-difference` | فرق | Difference | |
| `report-statement-bought` | کل خریداری | Total bought | |
| `report-statement-sold` | کل فروخت | Total sold | |
| `report-statement-debit` | کل بنام | Total Debit | |
| `report-statement-paid` | ادا شدہ | Paid | |
| `report-statement-credit` | کل جمع | Total Credit | |
| `report-side-dr` | بنام | Debit | |
| `report-side-cr` | جمع | Credit | |
| `report-side-zero` | صفر | Nil | |
| `report-col-date` | تاریخ | Date | |
| `report-col-doc` | اندراج | Entry | |
| `report-col-account` | کھاتہ | Account | |
| `report-col-narration` | تفصیل | Narration | |
| `report-col-debit` | بنام | Debit | |
| `report-col-credit` | جمع | Credit | |
| `report-col-balance` | بقایا | Balance | |
| `report-col-detail` | سامان | Goods | |
| `report-col-city` | شہر | City | |
| `report-col-head` | مد | Head | |
| `report-col-opening-date` | ابتدائی تاریخ | Opening date | |
| `report-detail-sale` | بکری .gender = feminine { $number }: { $item } { $qty } { $packing } @ { $rate } = { $value }، کمیشن .gender = masculine { $commission }، لاگا .gender = masculine { $laga } | Sale { $number }: { $item } { $qty } { $packing } @ { $rate } = { $value }, Commission { $commission }, Laga { $laga } | |
| `report-detail-lot` | بکری .gender = feminine { $number }: { $item } { $qty } { $packing }، خام بکری .gender = feminine { $gross }، { print-total-deductions } { $deductions } | Sale { $number }: { $item } { $qty } { $packing }, Gross Sale { $gross }, { print-total-deductions } { $deductions } | |
| `report-detail-cash-lent` | نقد ادھار .gender = masculine: { $reason } | Cash Lent: { $reason } | |
| `report-doc-number` | { $kind } { $number } | { $kind } { $number } | |
| `report-doc-bikri-first` | بکری .gender = feminine | Sale | |
| `report-doc-bikri-later` | بعد کی بکری .gender = feminine | Later Sale | |
| `report-doc-reversal` | الٹ اندراج | Reversal | |
| `report-doc-receipt` | وصولی .gender = feminine | Receipt | |
| `report-doc-payment` | ادائیگی .gender = feminine | Payment | |
| `report-doc-advance` | پیشگی .gender = feminine | Advance | |
| `report-doc-shop-expense` | دکان خرچہ .gender = masculine | Shop Expense | |
| `report-doc-personal-expense` | ذاتی خرچہ .gender = masculine | Personal Drawing | |
| `report-doc-personal-return` | ذاتی خرچہ .gender = masculine واپسی .gender = feminine | Personal Drawing Return | |
| `report-doc-money-lent` | ادھار رقم .gender = feminine | Money Lent | |
| `report-doc-money-lent-return` | ادھار رقم .gender = feminine واپسی .gender = feminine | Money Lent Return | |
| `report-doc-capital-in` | سرمایہ جمع | Capital invested | |
| `report-doc-capital-out` | سرمایہ واپسی .gender = feminine | Capital taken back | |
| `report-doc-khata-out` | الگ کھاتہ: رقم دی | Separate Khata: paid out | |
| `report-doc-khata-in` | الگ کھاتہ: رقم آئی | Separate Khata: received | |
| `report-doc-purchase` | خریداری | Purchase | |
| `report-doc-trip-cost` | سفر خرچ | Trip cost | |
| `report-doc-garden-payment` | باغ کی ادائیگی | Garden payment | |
| `report-doc-venture-receipt` | دوسری دکان میں فروخت | Sale in another shop | |
| `report-doc-garden-close` | باغ کی بندش | Garden closing | |
| `report-doc-fruit-taken` | پھل | Fruit taken | |
| `report-doc-general` | جنرل | Journal | |
| `report-doc-opening` | ابتدائی بقایا | Opening balance | |
| `report-doc-month-close` | مہینہ بند | Month close | |
| `report-doc-other` | دیگر | Other | |
| `report-print-title` | رپورٹ کی پرنٹ پر نظر | Report print preview | |
| `report-print-failed` | پرنٹ کی تیاری نہیں ہو سکی | The print could not be prepared | |
| `report-print-empty` | پرنٹ کرنے کے لیے کوئی قطار نہیں | There are no rows to print | |
| `report-print-row-count` | قطاریں | Rows | |
| `report-print-paper` | کاغذ | Paper | |
| `report-print-print` | پرنٹ کریں | Print | |
| `report-print-pdf` | PDF محفوظ کریں | Save as PDF | |
| `report-print-sent` | پرنٹر کو بھیج دیا گیا۔ | Sent to the printer. | |
| `report-print-close` | بند کریں | Close | |
| `report-r2-title` | مکمل بکری .gender = feminine | Full Sales Register | |
| `report-r2-help` | چنے ہوئے دنوں کی ہر بکری، گاہک .gender = masculine کی ہر لائن الگ: قسم مال، تعداد، ریٹ اور گاہک کی رقم۔ | Every sale of the days you choose, one row for each Buyer line: item, quantity, rate and what the buyer pays. | |
| `report-r3-title` | نقدی | Cash Sales | |
| `report-r3-help` | چنے ہوئے دنوں کی صرف نقدی بکری۔ | Only the Cash sales of the days you choose. | |
| `report-r4-title` | ادھاری | Credit Sales | |
| `report-r4-help` | چنے ہوئے دنوں کی صرف ادھاری بکری۔ | Only the Credit sales of the days you choose. | |
| `report-r6-title` | بکری .gender = feminine تفصیل | Sale Details | |
| `report-r6-help` | چنے ہوئے دنوں کا ہر لاٹ، خام بکری .gender = feminine، کل اخراجات اور صافی بکری .gender = feminine کے ساتھ۔ | Every lot of the days you choose, with the Gross Sale, the deductions and the Net Sale. | |
| `report-r7-title` | قسم مال وار بکری .gender = feminine | Item-wise Sales | |
| `report-r7-help` | قسم مال کے حساب سے بکری: تعداد، خام بکری .gender = feminine، اوسط ریٹ اور دونوں طرف کا کمیشن .gender = masculine۔ | Sales by item: quantity, Gross Sale, average rate and the Commission of both sides. | |
| `report-r8-title` | اڈا .gender = masculine رپورٹ | Adda Report | |
| `report-r8-help` | ہر اڈا .gender = masculine کے ٹرک، لاٹ، تعداد اور خام بکری .gender = feminine۔ | Trucks, lots, quantity and Gross Sale for each Adda. | |
| `report-r9-title` | پختہ کھاتہ .gender = masculine رپورٹ | Pukhta Khata Register | |
| `report-r9-help` | پختہ کھاتہ .gender = masculine کے ایک بار آنے والے لوگوں کی ہر لائن، ادا اور باقی کے ساتھ۔ | Every line of the one-time people of Pukhta Khata, with what is paid and what is due. | |
| `report-r20-title` | اگراہی .gender = feminine رپورٹ | Buyer Dues | |
| `report-r20-help` | ہر گاہک .gender = masculine کا بقایا اور اس کے کھلے بلوں کی عمر۔ میزان دکان کی کل اگراہی .gender = feminine ہے۔ | Each buyer's balance with the age of his open bills. The Total is the shop's total Collection. | |
| `report-r21-title` | اگراہی .gender = feminine بحساب شہر | Collection by City | |
| `report-r21-help` | یہی بقایا، شہر کے حساب سے، ہر شہر کے میزان کے ساتھ۔ | The same dues, grouped by city, with the total of each city. | |
| `report-r22-title` | اگراہی .gender = feminine بحساب اگراہی والا | Collection by Collector | |
| `report-r22-help` | یہی بقایا، اگراہی والے کے حساب سے، ہر ایک کے میزان کے ساتھ۔ | The same dues, grouped by collector, with the total of each collector. | |
| `report-r24-title` | ادائیگی .gender = feminine باقی | Seller Payables | |
| `report-r24-help` | دکان کے ذمے ہر بیوپاری .gender = masculine کا بقایا، اس کی کھلی پیشگی .gender = feminine اور جو وہ اس سے زیادہ دکان کا دینا ہے۔ | What the shop owes each seller, his open Advance and what he owes beyond it. | |
| `report-r26-title` | پیشگی کھاتہ .gender = masculine | Advance | |
| `report-r26-help` | ہر بیوپاری .gender = masculine کی پیشگی .gender = feminine: دی گئی، واپس، ایڈجسٹ اور ابھی باقی۔ | For each seller: Advance given, returned, adjusted and still open. | |
| `report-r27-title` | بیجک سیریز | Bill Series | |
| `report-r27-help` | بیوپاریوں کے بل نمبر کی ترتیب میں، خام بکری .gender = feminine، کل اخراجات اور صافی بکری .gender = feminine کے ساتھ۔ | The sellers' bills in number order, with the Gross Sale, the deductions and the Net Sale. | |
| `report-r28-title` | آمدن رپورٹ | Income Report | |
| `report-r28-help` | ہر آمدن کھاتہ مہینے یا دن کے حساب سے۔ میزان خالص آمدن ہے۔ | Every income account by month or by day. The Total is the net income. | |
| `report-r30-title` | منافع و تقسیم | Profit and Partner Split | |
| `report-r30-help` | آمدن منفی دکان خرچہ، اور ہر شراکت دار کا حصہ .gender = masculine، لی گئی رقم اور خالص .gender = masculine۔ | Income less shop expenses, and each partner's Share, money taken and net. | |
| `report-col-buyer` | گاہک .gender = masculine | Buyer | |
| `report-col-seller` | بیوپاری .gender = masculine | Seller | |
| `report-col-item` | قسم مال | Item | |
| `report-col-qty` | تعداد | Quantity | |
| `report-col-rate` | ریٹ | Rate | |
| `report-col-value` | قیمت | Value | |
| `report-col-buyer-total` | رقم گاہک | Buyer total | |
| `report-col-gross` | خام بکری .gender = feminine | Gross Sale | |
| `report-col-freight` | کرایہ .gender = masculine | Freight | |
| `report-col-commission` | کمیشن .gender = masculine | Commission | |
| `report-col-labour` | مزدوری .gender = feminine | Labour | |
| `report-col-munshiana` | منشیانہ | Munshiana | |
| `report-col-cash-paid` | نقد .gender = masculine | Cash Paid | |
| `report-col-store-rent` | کرایہ سٹور .gender = masculine | Store Rent | |
| `report-col-phone-charge` | فون .gender = masculine | Phone Charge | |
| `report-col-deductions` | { print-total-deductions } | { print-total-deductions } | |
| `report-col-net` | صافی بکری .gender = feminine | Net Sale | |
| `report-col-avg-rate` | اوسط ریٹ | Average rate | |
| `report-col-seller-comm` | کمیشن .gender = masculine بیوپاری | Seller Commission | |
| `report-col-buyer-comm` | کمیشن .gender = masculine گاہک | Buyer Commission | |
| `report-col-laga` | لاگا .gender = masculine | Laga | |
| `report-col-total-comm` | کل کمیشن .gender = masculine | Total Commission | |
| `report-col-adda` | اڈا .gender = masculine | Adda | |
| `report-col-trucks` | ٹرک | Trucks | |
| `report-col-lots` | لاٹ | Lots | |
| `report-col-side` | جانب | Side | |
| `report-col-person` | نام | Name | |
| `report-col-phone` | فون | Phone | |
| `report-col-status` | حالت | Status | |
| `report-col-oldest-date` | پرانا بل | Oldest bill | |
| `report-col-days` | دن | Days | |
| `report-col-age-0-7` | ⁧0 تا 7 دن⁩ | 0-7 days | |
| `report-col-age-8-30` | ⁧8 تا 30 دن⁩ | 8-30 days | |
| `report-col-age-31-90` | ⁧31 تا 90 دن⁩ | 31-90 days | |
| `report-col-age-91` | ⁧90 دن سے زیادہ⁩ | Over 90 days | |
| `report-col-collector` | اگراہی والا | Collector | |
| `report-col-section` | فہرست | List | |
| `report-col-payable` | ادائیگی .gender = feminine باقی | Payable | |
| `report-col-advance-open` | پیشگی .gender = feminine باقی | Open Advance | |
| `report-col-owes-firm` | دکان کا دینا | Owes the shop | |
| `report-col-last-date` | آخری تاریخ | Last date | |
| `report-col-given` | دی گئی | Given | |
| `report-col-returned` | واپس | Returned | |
| `report-col-adjusted` | ایڈجسٹ | Adjusted | |
| `report-col-open` | باقی | Open | |
| `report-col-bill-date` | تاریخ بل | Bill date | |
| `report-col-period` | مدت | Period | |
| `report-col-amount` | رقم | Amount | |
| `report-col-income-share` | آمدن حصہ .gender = masculine | Income Share | |
| `report-col-expense-share` | خرچہ حصہ .gender = masculine | Expense Share | |
| `report-col-profit-share` | منافع حصہ .gender = masculine | Profit Share | |
| `report-col-taken` | ذاتی خرچہ .gender = masculine | Personal Drawing | |
| `report-col-partner-net` | خالص .gender = masculine | Net | |
| `report-col-partner` | شراکت دار | Partner | |
| `report-side-buyer` | گاہک .gender = masculine | Buyer | |
| `report-side-seller` | بیوپاری .gender = masculine | Seller | |
| `report-section-payable` | بیوپاریوں کو دینا | Payable to sellers | |
| `report-section-owes-firm` | وہ بیوپاری جن کے ذمے دکان کی رقم ہے | Sellers who owe the shop | |
| `report-status-all` | تمام لائنیں | All lines | |
| `report-status-paid` | ادا | Paid | |
| `report-status-due` | باقی | Due | |
| `report-status-cancelled` | منسوخ | Cancelled | |
| `report-r30-income` | کل آمدن | Total income | |
| `report-r30-expenses` | کل دکان خرچہ | Total shop expenses | |
| `report-r30-profit` | منافع | Profit | |
| `report-filter-paid` | صرف ادا شدہ لائنیں | Paid lines only | |
| `report-filter-due` | صرف باقی لائنیں | Due lines only | |
| `report-filter-by-day` | دن کے حساب سے | By day | |
| `report-filter-by-month` | مہینے کے حساب سے | By month | |
| `report-filter-person` | ایک بار آنے والا فرد | One-time person | |
| `report-filter-person-hint` | نام یا فون نمبر لکھیں | Type a name or a phone number | |
| `report-filter-status` | دکھائیں | Show | |
| `report-filter-group-by` | ترتیب | Group by | |
| `report-group-by-month` | مہینہ | Month | |
| `report-group-by-day` | دن | Day | |
| `report-month-1` | جنوری | January | |
| `report-month-2` | فروری | February | |
| `report-month-3` | مارچ | March | |
| `report-month-4` | اپریل | April | |
| `report-month-5` | مئی | May | |
| `report-month-6` | جون | June | |
| `report-month-7` | جولائی | July | |
| `report-month-8` | اگست | August | |
| `report-month-9` | ستمبر | September | |
| `report-month-10` | اکتوبر | October | |
| `report-month-11` | نومبر | November | |
| `report-month-12` | دسمبر | December | |
| `report-card-ledger` | کھاتہ .gender = masculine | Ledger | |
| `report-card-statement` | فرد حساب | Statement | |
| `report-card-open-ledger` | اس کھاتے کا کھاتہ .gender = masculine کھولیں | Open the Ledger of this account | |
| `report-card-open-statement` | اس کھاتے کا فرد حساب کھولیں | Open the Statement of this account | |
| `report-card-reports-label` | اس کھاتے کی رپورٹیں | Reports of this account | |
| `report-search-person` | ایک بار آنے والا فرد | One-time person | |
| `report-search-person-last` | آخری { $date } | Last { $date } | |
| `report-search-screen` | اسکرین | Screen | |
| `report-search-report` | رپورٹ | Report | |
| `report-filters-more` | مزید فلٹر | More filters | |
| `report-filters-fewer` | کم فلٹر | Fewer filters | |
| `report-group-no-city` | شہر کے بغیر | No city | |
| `report-group-no-collector` | اگراہی والے کے بغیر | No collector | |
| `report-adda-none` | اڈا کے بغیر | No Adda | |
| `report-r12-title` | روکڑ | Cash Book | |
| `report-r12-help` | چنی ہوئی تاریخوں کی ابتدائی روکڑ، ہر رقم جو آئی اور دی گئی، اور آخری روکڑ، چلتے بقایا کے ساتھ۔ | The opening cash, every cash in and out of the days chosen, and the closing cash, with the running balance. | |
| `report-r29-title` | خرچہ .gender = masculine رپورٹ | Expense Report | |
| `report-r29-help` | دکان خرچہ .gender = masculine مد کے حساب سے، اور ہر شراکت دار کا لیا اور واپس کیا ہوا ذاتی خرچہ .gender = masculine۔ | Shop Expense by head, and the Personal Drawing each partner took and returned. | |
| `report-r31-title` | شراکت دار کھاتہ | Partner Statement | |
| `report-r31-help` | ایک شراکت دار کی ہر تاریخ وار حرکت، اس کی چلتی کل رقم کے ساتھ۔ | Every dated movement of one partner, with his running Partner Total. | |
| `report-r32-title` | ادھار رقم .gender = feminine رپورٹ | Money Lent Report | |
| `report-r32-help` | دکان سے ادھار لینے والا ہر شخص: دی ہوئی، واپس ہوئی اور کھلی رقم، اور واپسی کی تاریخ کے بعد گزرے دن۔ | Everyone who borrowed from the shop: lent, returned, still open, and the days past the return-by date. | |
| `report-col-cash-in` | رقم آئی | Cash In | |
| `report-col-cash-out` | رقم دی | Cash Out | |
| `report-col-name` | نام | Name | |
| `report-col-expense` | خرچہ .gender = masculine | Expense | |
| `report-col-staff` | عملہ | Staff | |
| `report-col-lent` | رقم دی | Lent | |
| `report-col-returned-cash` | واپسی .gender = feminine (نقد) | Return (Cash) | |
| `report-col-returned-salary` | تنخواہ سے کٹوتی | Salary Deduction | |
| `report-col-return-by` | واپسی کی تاریخ | Return By | |
| `report-col-days-past` | تاخیر کے دن | Days Late | |
| `report-staff-yes` | ہاں | Yes | |
| `report-r32-overdue` | واپسی کی تاریخ کے بعد بھی کھلی رقم | Open past the return-by date | |
| `report-section-shop-expenses` | دکان خرچہ .gender = masculine | Shop Expense | |
| `report-section-partner-money` | ذاتی خرچہ .gender = masculine | Personal Drawing | |
| `report-filter-partner` | شراکت دار | Partner | |
| `report-filter-borrower` | ادھار لینے والا | Borrower | |
| `report-partner-all` | تمام شراکت دار | All partners | |
| `report-pick-partner` | پہلے شراکت دار چنیں | Pick a partner first | |
| `report-pick-partner-help` | یہ کھاتہ ایک شراکت دار کا ہوتا ہے۔ اسے اوپر سے چنیں۔ | This statement is for one partner. Choose him above. | |
| `report-r10-title` | باقی مال | Unsold Stock | |
| `report-r10-help` | وصول ہوا مال جو ابھی بکا نہیں: کتنا آیا، کتنا بکا، کتنا باقی ہے اور کتنے دن سے۔ | Goods received and not yet sold: what came, what was sold, what is left and for how many days. | |
| `report-r15-title` | واؤچر | Vouchers | |
| `report-r15-help` | ہر واؤچر قسم اور تاریخ کے حساب سے۔ قسم نہ چنیں تو ہر قسم کی رقم جدول کے اوپر دکھائی جاتی ہے۔ منسوخ واؤچر فہرست میں رہتا ہے مگر رقم میں شمار نہیں ہوتا۔ | Every voucher by kind and date. With no kind picked, the money of each kind is shown above the table. A cancelled voucher is listed and not counted. | |
| `report-r23-title` | وصولی .gender = feminine رجسٹر | Receipt Register | |
| `report-r23-help` | ہر وصولی خریدار، رقم اور اگراہی والے کے ساتھ۔ پیشگی واپس کرنے والا بیوپاری بھی یہاں ہے۔ منسوخ وصولی فہرست میں رہتی ہے مگر رقم میں شمار نہیں ہوتی۔ | Every receipt with the buyer, the amount and the collector. A seller returning his advance is here too. A cancelled receipt is listed and not counted. | |
| `report-r25-title` | ادائیگی .gender = feminine رجسٹر | Payment Register | |
| `report-r25-help` | ہر ادائیگی کھاتے اور رقم کے ساتھ۔ منسوخ ادائیگی فہرست میں رہتی ہے مگر رقم میں شمار نہیں ہوتی۔ | Every payment with the account and the amount. A cancelled payment is listed and not counted. | |
| `report-r33-title` | تجارتی منافع / نقصان | Trading Profit / Loss | |
| `report-r33-help` | مارکیٹ سفر نے کیا خریدا، بیچا، ضائع کیا اور کیا باقی ہے، اور ہر سفر کا منافع یا نقصان، سفر، قسم مال، دن، مہینے یا شراکت دار کے حساب سے۔ | What the market trips bought, sold, wrote off and still hold, and the profit or loss of each trip, by trip, item, day, month or partner. | |
| `report-r34-title` | روزانہ خلاصہ | Day Summary | |
| `report-r34-help` | ہر دن کا ایک حصہ: بکری، روکڑ، وصولی، ادائیگی، خرچہ، شراکت داروں کی رقم، سفر اور جو ابھی لینا ہے۔ | One block for each day: sales, cash, receipts, payments, expenses, the partners' money, trips and what is still owed. | |
| `report-col-packing` | پیکنگ .gender = feminine | Packing | |
| `report-col-received` | آیا | Received | |
| `report-col-sold` | بکا | Sold | |
| `report-col-written-off` | ضائع | Written Off | |
| `report-col-balance-qty` | باقی | Balance | |
| `report-col-days-waiting` | انتظار کے دن | Days Waiting | |
| `report-col-trip` | سفر | Trip | |
| `report-col-purchase` | خریداری | Purchase | |
| `report-col-bought-qty` | خریدی گئی تعداد | Quantity Bought | |
| `report-col-bought-cost` | خریداری کی لاگت | Bought Cost | |
| `report-col-sales-money` | بکری کی رقم | Sales Money | |
| `report-col-cost-sold` | بکے مال کی لاگت | Cost of Goods Sold | |
| `report-col-written-off-cost` | ضائع کی لاگت | Written-off Cost | |
| `report-col-left-qty` | بچا | Left | |
| `report-col-left-cost` | بچی ہوئی لاگت | Cost Left | |
| `report-col-result` | نتیجہ | Result | |
| `report-col-share` | حصہ .gender = masculine | Share | |
| `report-col-money-in-shop` | ادا: دکان سے | Paid: From the Shop | |
| `report-col-money-in-pocket` | ادا: اپنی جیب سے | Paid: Own Pocket | |
| `report-col-money-back` | واپس ملا | Money Back | |
| `report-col-part-of-result` | اب تک کا حصہ | Part So Far | |
| `report-col-line` | تفصیل | Detail | |
| `report-col-count` | تعداد | Count | |
| `report-result-profit` | منافع | Profit | |
| `report-result-loss` | نقصان | Loss | |
| `report-filter-voucher-type` | قسم | Type | |
| `report-voucher-type-all` | تمام اقسام | All kinds | |
| `report-filter-trip-view` | ترتیب | Show by | |
| `report-filter-trip-by-trip` | سفر وار | By trip | |
| `report-filter-trip-by-item` | قسم مال وار | By item | |
| `report-filter-trip-by-day` | دن وار | By day | |
| `report-filter-trip-by-month` | مہینہ وار | By month | |
| `report-filter-trip-by-partner` | شراکت دار وار | By partner | |
| `report-r34-bikri` | خام بکری .gender = feminine | Gross Sale | |
| `report-r34-commission` | کمیشن .gender = masculine اور لاگا .gender = masculine | Commission and Laga | |
| `report-r34-cash-opening` | ابتدائی روکڑ | Cash Opening | |
| `report-r34-cash-in` | رقم آئی | Cash In | |
| `report-r34-cash-out` | رقم دی | Cash Out | |
| `report-r34-cash-closing` | آخری روکڑ | Cash Closing | |
| `report-r34-receipts` | وصولی .gender = feminine | Receipt | |
| `report-r34-payments` | ادائیگی .gender = feminine | Payment | |
| `report-r34-advances` | پیشگی .gender = feminine | Advance | |
| `report-r34-expenses` | دکان خرچہ .gender = masculine | Shop Expense | |
| `report-r34-partner-taken` | ذاتی خرچہ .gender = masculine | Personal Drawing | |
| `report-r34-partner-returned` | ذاتی خرچہ .gender = masculine واپسی .gender = feminine | Personal Drawing Return | |
| `report-r34-capital` | سرمایہ | Capital | |
| `report-r34-money-lent` | ادھار رقم .gender = feminine | Money Lent | |
| `report-r34-money-lent-back` | ادھار رقم .gender = feminine واپسی .gender = feminine | Money Lent Return | |
| `report-r34-trips-bought` | مارکیٹ سفر .gender = masculine: خریداری | Market Trip: Purchase | |
| `report-r34-trips-sold` | مارکیٹ سفر .gender = masculine: بکری .gender = feminine | Market Trip: Sale | |
| `report-r34-partner-shares` | شراکت دار کا حصہ .gender = masculine | Partners' Share | |
| `report-r34-buyers-dues` | وصولی باقی | Buyers' Dues | |
| `report-r34-open-lots` | باقی لاٹ .gender = masculine | Unsold Lot | |
| `report-r34-open-qty` | باقی نگ | Nag Left | |
| `report-r35-title` | پارٹی کی روزانہ پرچی | Party Day Sheet | |
| `report-r35-help` | ایک دن کا: ہر پارٹی کا شروع کا بقایا، اس دن کا بنام اور جمع، اور جو باقی ہے۔ پرچی دیکھنے کے لیے پارٹی چنیں۔ جو بل اسی دن بدلا یا منسوخ ہوا وہ دن کے خانوں میں ایک ہی بار گنا جاتا ہے؛ بقایا کھاتے سے ملتے ہیں۔ | For one day: what each party owed at the start, what was debited and credited, and what is left. Pick a party to see his slip. A bill edited or cancelled on its own day counts once in the day's columns; the balances agree with the ledger. | |
| `report-col-code` | کوڈ | Code | |
| `report-col-previous` | سابقہ رقم | Previous Balance | |
| `report-col-party` | پارٹی | Party | |
| `report-r35-balance` | بقایا | Balance | |
| `report-r35-buyers` | گاہک .gender = masculine | Buyer | |
| `report-r35-sellers` | بیوپاری .gender = masculine | Seller | |
| `report-r35-total-qty` | کل نگ | Total Nag | |
| `report-r35-previous` | سابقہ رقم | Previous Balance | |
| `report-r35-bought` | تازہ بنام | Today's Bill | |
| `report-r35-sold` | اس دن کی بکری .gender = feminine | Sold That Day | |
| `report-r35-other-debits` | دیگر بنام | Other Debit | |
| `report-r35-other-credits` | دیگر جمع | Other Credit | |
| `report-r35-on-spot` | موقع پر ادا | Paid on the Spot | |
| `report-r35-cash-given` | نقد .gender = masculine | Cash Paid | |
| `report-r35-received` | وصولی .gender = feminine | Receipt | |
| `report-r35-paid` | ادائیگی .gender = feminine | Payment | |
| `report-r35-remaining` | ٹوٹل رقم | Total Due | |
| `report-r35-collection` | کل اگراہی .gender = feminine | Total Collection | |
| `report-r35-sellers-credit` | بیوپاریوں کا جمع | Sellers' Credit | |
| `report-r35-sellers-debit` | بیوپاریوں کا بنام | Sellers' Debit | |
| `report-r35-sellers-remaining` | بیوپاریوں کا بقایا | Sellers' Balance | |
| `report-filter-day` | دن | Day | |
| `report-day-shown` | <num>{ $date }</num> کی پرچی | Sheet for <num>{ $date }</num> | |
| `report-day-now` | آج کی پرچی | Sheet for today | |
| `report-r35-all-parties` | تمام پارٹیاں | All parties | |
| `report-r36-title` | باغات رپورٹ | Gardens Report | |
| `report-r36-help` | ہر باغ کا: ٹھیکہ، قسطیں، خرچ، دکان اور شراکت داروں کی ادائیگی، آنے والی رقم، چلتا بقایا اور باقی ٹھیکہ۔ باغ وار یا شراکت دار وار۔ | Each garden: rent, installments, costs, what the shop and the partners paid, the money that came in, the running balance and the rent still due. By garden or by partner. | |
| `report-col-garden` | باغ | Garden | |
| `report-col-rent` | ٹھیکہ | Garden Rent | |
| `report-col-installments` | قسط | Installments | |
| `report-col-garden-costs` | باغ خرچ | Garden Costs | |
| `report-col-sales-our-shop` | ہماری دکان میں فروخت | Sales in Our Shop | |
| `report-col-sales-other-shops` | دوسری دکانوں میں فروخت | Sales in Other Shops | |
| `report-col-other-in` | دیگر آمد | Other Money In | |
| `report-col-money-in` | آنے والی رقم | Money In | |
| `report-col-running-balance` | چلتا بقایا | Running Balance | |
| `report-col-rent-due` | ٹھیکہ باقی | Rent Due | |
| `report-garden-open` | کھلا | Open | |
| `report-garden-closed` | بند | Closed | |
| `report-filter-garden` | باغ | Garden | |
| `report-filter-garden-view` | ترتیب | Show by | |
| `report-filter-garden-by-garden` | باغ وار | By garden | |
| `report-filter-garden-by-partner` | شراکت دار وار | By partner | |
| `report-r34-gardens-paid` | باغات کی ادائیگی | Gardens Paid | |
| `report-r34-gardens-in` | باغات میں آمد | Gardens Money In | |
| `report-r1-title` | بکری بک | Sale Book | |
| `report-r1-help` | ایک بکری .gender = feminine کے لاٹ: کیا آیا، کیا بکا، خرچے اور صافی بکری۔ اس کا بل یہیں سے پرنٹ ہوتا ہے۔ | The lots of one Sale: what came, what was sold, the charges and the net sale. Its bill is printed from here. | |
| `report-r5-title` | ادھاری تفصیل | Credit Details | |
| `report-r5-help` | ہر ادھاری گاہک کا مال، اس کی رقم اور کمیشن .gender = masculine اور لاگا .gender = masculine کے ساتھ اس کے ذمے رقم۔ | Each Credit buyer with his goods, what they come to and what he owes with the Commission and Laga. | |
| `report-col-serial` | نمبر شمار | No. | |
| `report-filter-bikri` | بکری .gender = feminine نمبر | Sale number | |
| `report-pick-bikri` | پہلے بکری .gender = feminine چنیں | Pick a Sale first | |
| `report-pick-bikri-help` | یہ رپورٹ ایک بکری .gender = feminine کی ہے۔ اوپر اس کا نمبر لکھیں۔ | This report is for one Sale. Type its number above. | |
| `report-bikri-hit` | بکری .gender = feminine <num>{ $number }</num> · <num>{ $date }</num> | Sale <num>{ $number }</num> · <num>{ $date }</num> | |
| `report-detail-sale-short` | ({ $item } { $qty } . { $rate }) | ({ $item } { $qty } . { $rate }) | |
| `report-print-all` | تمام پرچیاں پرنٹ کریں | Print all slips | |
| `report-buyer-bills-slips` | پرچیاں | Slips | |
| `report-r37-title` | پھل رجسٹر | Fruit Taken Register | |
| `report-r37-help` | شراکت داروں کا گھر لے جایا گیا پھل اور دکان کے مہمان کے لیے پھل: تاریخ، کس کے لیے، لاٹ کا بیوپاری، قسم مال، تعداد، ریٹ اور قیمت۔ شراکت دار، مہمان، قسم مال یا دن کے حساب سے۔ | The fruit partners took home and the fruit served to the shop's Guests: the date, who it was for, the seller of the lot, the item, the quantity, the rate and the value. By partner, Guests, item and day. | |
| `report-col-fruit-taken` | پھل | Fruit Taken | |
| `report-r34-fruit-taken` | پھل | Fruit Taken | |

## rules

| ID | Urdu | English | Result |
|---|---|---|---|
| `masters-rules-title` | کمیشن .gender = masculine اصول | Commission rules | |
| `masters-rules-help` | کمیشن .gender = masculine، لاگا .gender = masculine اور باقی چارجز کے اصول، کھاتے، قسم مال یا گروپ کے حساب سے۔ | The rules for Commission, Laga and the other charges, by party, Item or group. | |
| `rules-title` | کمیشن .gender = masculine اصول | Commission rules | |
| `rules-help` | ہر چارج کا اصول یہاں لکھا جاتا ہے۔ سب سے خاص اصول جیتتا ہے۔ کوئی اصول نہ ہو تو عام اصول لگتا ہے۔ | Every charge has its rule here. The most specific rule wins. When there is none, the default rule applies. | |
| `rules-new` | نیا اصول | New rule | |
| `rules-test-open` | اصول آزمائیں | Test a rule | |
| `rules-col-side` | جانب | Side | |
| `rules-col-charge` | چارج | Charge | |
| `rules-col-party` | کھاتہ | Party | |
| `rules-col-item` | قسم مال یا گروپ | Item or group | |
| `rules-col-method` | طریقہ | Method | |
| `rules-col-value` | شرح | Value | |
| `rules-col-from` | تاریخ سے | From | |
| `rules-col-to` | تاریخ تک | To | |
| `rules-col-level` | درجہ | Level | |
| `rules-col-status` | حالت | Status | |
| `rules-status-active` | فعال | Active | |
| `rules-status-inactive` | غیر فعال | Inactive | |
| `rules-value-percent` | { $value } فیصد | { $value } percent | |
| `rules-value-per-unit` | { $value } فی نگ | { $value } per Nag | |
| `rules-value-per-lot` | { $value } فی لاٹ .gender = masculine | { $value } per Lot | |
| `rules-summary` | { $side } { $charge }، { $party }، { $item }، { $value } | { $side } { $charge }, { $party }, { $item }, { $value } | |
| `rules-any-party` | کوئی بھی کھاتہ | Any party | |
| `rules-any-item` | کوئی بھی قسم مال | Any Item | |
| `rules-open-ended` | کوئی آخری تاریخ نہیں | No end date | |
| `rules-side-seller` | بیوپاری .gender = masculine | Seller | |
| `rules-side-buyer` | گاہک .gender = masculine | Buyer | |
| `rules-charge-commission` | کمیشن .gender = masculine | Commission | |
| `rules-charge-laga` | لاگا .gender = masculine | Laga | |
| `rules-charge-labour` | مزدوری .gender = feminine | Labour | |
| `rules-charge-munshiana` | منشیانہ | Munshiana | |
| `rules-charge-phone` | فون .gender = masculine | Phone Charge | |
| `rules-method-percent` | فیصد | Percent | |
| `rules-method-per_unit` | فی نگ | Per Nag | |
| `rules-method-per_lot` | فی لاٹ .gender = masculine | Per Lot | |
| `rules-group-none` | کوئی گروپ نہیں | No group | |
| `rules-group-fruit` | پھل | Fruit | |
| `rules-group-vegetable` | سبزی | Vegetable | |
| `rules-level-line-override` | لائن پر لکھی گئی رقم | Typed on the line | |
| `rules-level-party-item` | کھاتہ اور قسم مال | Party and Item | |
| `rules-level-party` | کھاتہ | Party | |
| `rules-level-item` | قسم مال | Item | |
| `rules-level-item-group` | قسم مال کا گروپ | Item group | |
| `rules-level-default` | عام اصول | Default | |
| `rules-all` | سب | All | |
| `rules-filter-side` | جانب | Side | |
| `rules-filter-charge` | چارج | Charge | |
| `rules-filter-party` | کھاتہ | Party | |
| `rules-filter-item` | قسم مال | Item | |
| `rules-show-inactive` | غیر فعال بھی دکھائیں | Show the inactive ones too | |
| `rules-load-failed` | اصول نہیں کھل سکے | The rules could not be opened | |
| `rules-empty` | ابھی کوئی اصول نہیں | No rules yet | |
| `rules-empty-help` | نیا اصول بنانے کے لیے { $key } دبائیں۔ | Press { $key } to make a new rule. | |
| `rules-no-match-title` | کوئی اصول نہیں ملا | No rule found | |
| `rules-no-match-help` | فلٹر بدل کر دیکھیں، یا غیر فعال بھی دکھائیں۔ | Change the filters, or show the inactive ones too. | |
| `rules-no-match-clear` | فلٹر صاف کریں | Clear the filters | |
| `rules-not-found-title` | یہ اصول نہیں ملا | This rule was not found | |
| `rules-not-found-help` | ہو سکتا ہے یہ بدل چکا ہو۔ اصولوں کی فہرست دوبارہ کھولیں۔ | It may have been changed. Open the list of rules again. | |
| `rules-new-title` | نیا اصول | New rule | |
| `rules-edit-title` | اصول بدلیں | Change the rule | |
| `rules-field-side` | جانب | Side | |
| `rules-field-charge` | چارج | Charge | |
| `rules-field-party` | کھاتہ | Party | |
| `rules-field-party-hint` | خالی چھوڑیں تو ہر کھاتے کے لیے لگے گا۔ | Leave it empty and it applies to every party. | |
| `rules-field-item` | قسم مال | Item | |
| `rules-field-item-hint` | خالی چھوڑیں تو ہر قسم مال کے لیے لگے گا۔ | Leave it empty and it applies to every Item. | |
| `rules-field-group` | گروپ | Group | |
| `rules-field-group-hint` | صرف اس وقت جب کوئی کھاتہ یا قسم مال نہ چنا ہو۔ | Only when no party or Item is chosen. | |
| `rules-field-method` | طریقہ | Method | |
| `rules-field-value` | شرح | Value | |
| `rules-value-hint-percent` | فیصد میں لکھیں، مثلاً 6 یا 2.5۔ | Write a percent, for example 6 or 2.5. | |
| `rules-value-hint-per_unit` | ہر نگ کے پورے روپے لکھیں، مثلاً 8۔ | Write whole rupees for each Nag, for example 8. | |
| `rules-value-hint-per_lot` | ہر لاٹ .gender = masculine کے پورے روپے لکھیں، مثلاً 50۔ | Write whole rupees for each Lot, for example 50. | |
| `rules-field-from` | تاریخ سے | From date | |
| `rules-field-to` | تاریخ تک | To date | |
| `rules-field-to-hint` | خالی چھوڑیں تو کوئی آخری تاریخ نہیں۔ | Leave it empty for no end date. | |
| `rules-field-active` | فعال | Active | |
| `rules-field-note` | نوٹ | Note | |
| `rules-saved` | اصول محفوظ ہو گیا | Rule saved | |
| `rules-make-active` | فعال کریں | Make active | |
| `rules-make-inactive` | غیر فعال کریں | Make inactive | |
| `rules-made-active` | اصول فعال کر دیا گیا | Rule made active | |
| `rules-made-inactive` | اصول غیر فعال کر دیا گیا | Rule made inactive | |
| `rules-inactive-title` | غیر فعال کریں؟ | Make this inactive? | |
| `rules-inactive-description` | غیر فعال اصول آئندہ نہیں لگے گا۔ پہلے سے محفوظ بکری .gender = feminine میں کوئی تبدیلی نہیں ہوگی۔ جب چاہیں دوبارہ فعال کر سکتے ہیں۔ | An inactive rule is not used from now on. A Sale that is already saved does not change. You can make it active again at any time. | |
| `rules-overlap-title` | اسی درجے کا ایک اور اصول انہی تاریخوں میں لگتا ہے | Another rule at the same level covers the same dates | |
| `rules-overlap-help` | جو اصول بعد میں شروع ہوتا ہے وہ اپنی شروع کی تاریخ سے لگتا ہے۔ دوسرا اصول نہیں بدلا۔ | The rule that starts later is used from its start day. The other rule is not changed. | |
| `rules-overlap-done` | ٹھیک ہے | Done | |
| `rules-test-title` | اصول آزمائیں | Test a rule | |
| `rules-test-help` | بیوپاری .gender = masculine، گاہک .gender = masculine، قسم مال، تاریخ، تعداد اور ریٹ لکھیں۔ کچھ محفوظ نہیں ہوتا۔ | Enter the Seller, the Buyer, the Item, the date, the quantity and the rate. Nothing is saved. | |
| `rules-test-seller` | بیوپاری .gender = masculine | Seller | |
| `rules-test-buyer` | گاہک .gender = masculine | Buyer | |
| `rules-test-item` | قسم مال | Item | |
| `rules-test-date` | تاریخ | Date | |
| `rules-test-qty` | تعداد | Quantity | |
| `rules-test-rate` | ریٹ | Rate | |
| `rules-test-run` | آزمائیں | Test | |
| `rules-test-nothing` | نتیجہ دیکھنے کے لیے اوپر کی باتیں لکھ کر آزمائیں دبائیں۔ | Fill in the fields above and press Test to see the result. | |
| `rules-test-failed` | اصول نہیں آزمایا جا سکا | The rule could not be tested | |
| `rules-test-winner` | جیتنے والا اصول | Winning rule | |
| `rules-test-level` | درجہ | Level | |
| `rules-test-skipped` | چھوڑے گئے درجے | Levels skipped | |
| `rules-test-skipped-none` | کوئی نہیں | None | |
| `rules-test-amount` | رقم | Amount | |
| `rules-test-no-rule` | کوئی اصول نہیں۔ رقم لاٹ .gender = masculine پر لکھی جاتی ہے۔ | No rule. The amount is typed on the Lot. | |
| `rules-test-result` | نتیجہ | Result | |
| `settings-tab-charges` | چارجز | Charges | |
| `settings-charges-help` | یہ عام شرحیں ہیں۔ نئی شرح اسی دن سے لاگو ہوتی ہے جس دن آپ اسے بدلیں؛ پرانے دن پرانی شرح پر رہتے ہیں۔ کسی کھاتے یا قسم مال کی الگ شرح اصولوں میں لکھی جاتی ہے۔ | These are the default rates. A new rate applies from the day you change it; earlier days keep the old rate. A rate for one party or Item is written as a rule. | |
| `settings-charges-from` | یہ شرح { $date } سے لاگو ہے۔ | This rate applies from { $date }. | |
| `settings-charges-seller` | بیوپاری .gender = masculine کمیشن .gender = masculine | Seller Commission | |
| `settings-charges-buyer` | گاہک .gender = masculine کمیشن .gender = masculine | Buyer Commission | |
| `settings-charges-laga` | پھلوں پر لاگا .gender = masculine | Laga on fruit | |
| `settings-charges-note` | مزدوری .gender = feminine، منشیانہ اور فون .gender = masculine کا کوئی عام اصول نہیں؛ وہ ہر لاٹ .gender = masculine پر لکھے جاتے ہیں۔ | Labour, Munshiana and Phone Charge have no default: they are typed on each Lot. | |
| `settings-charges-saved` | چارجز محفوظ ہو گئے | Charges saved | |
| `settings-charges-open-rules` | کھاتے یا قسم مال کے الگ اصول دیکھیں | See the rules for one party or Item | |
| `settings-tab-overrides` | دستی تبدیلی اور جانچ | Overrides and checks | |
| `settings-overrides-allow` | کمیشن .gender = masculine اور لاگا .gender = masculine لائن پر بدلنے کی اجازت ہے | The Commission and Laga can be changed on a line | |
| `settings-overrides-allow-hint` | بدلی ہوئی رقم لائن پر "دستی" لکھی جاتی ہے۔ | A changed amount is saved on the line as typed by hand. | |
| `settings-overrides-credit` | ادھار کی حد پار ہو جائے تو | When the credit limit is passed | |
| `settings-overrides-credit-warn` | صرف خبردار کریں | Only warn | |
| `settings-overrides-credit-block` | بکری .gender = feminine روک دیں | Stop the Sale | |
| `settings-overrides-saved` | ترتیب محفوظ ہو گئی | Settings saved | |
| `settings-load-charges-failed` | چارجز نہیں کھل سکے | The charges could not be opened | |
| `rules-dates` | { $from } سے { $to } تک | From { $from } to { $to } | |
| `rules-overlap-open` | اصول کھولیں | Open the rule | |
| `rules-dates-open` | { $from } سے، کوئی آخری تاریخ نہیں | From { $from }, no end date | |
| `rules-test-manual` | لائن پر ہاتھ سے لکھی گئی | Typed by hand on the line | |
| `rules-test-line-value` | لائن کی قیمت (تعداد × ریٹ) | Value of the line (quantity × rate) | |
| `rules-status-default` | عام | Default | |
| `rules-default-title` | عام شرح | Default rate | |
| `rules-default-notice` | یہ عام شرح ہے۔ اسے ترتیبات کے چارجز میں بدلیں۔ | This is a default rate. Change it in Settings, Charges. | |
| `rules-default-open-settings` | ترتیبات کھولیں | Open Settings | |
| `rules-status-default-ended` | عام، { $to } تک | Default, until { $to } | |
| `settings-overrides-rate-check` | ریٹ بہت مختلف ہو تو خبردار کریں | Warn when a rate is very different | |
| `settings-overrides-rate-check-hint` | ریٹ پچھلے 7 دن کی اوسط سے بہت زیادہ یا کم ہو تو لائن پر تنبیہ آتی ہے۔ یہ محفوظ کرنے سے نہیں روکتی۔ | A warning shows on the line when a rate is far above or below the item's average of the last 7 days. It never stops a save. | |
| `settings-overrides-rate-check-percent` | کتنے فیصد فرق پر | How far apart, in percent | |
| `settings-overrides-rate-check-percent-hint` | 1 سے 1,000 کے درمیان پورا عدد لکھیں، مثلاً 30۔ | Enter a whole number from 1 to 1,000, for example 30. | |

## search

| ID | Urdu | English | Result |
|---|---|---|---|
| `search-open-list` | فہرست کھولیں | Open the list | |
| `search-list-label` | ملتے جلتے ناموں کی فہرست | Matching records | |
| `search-searching` | تلاش جاری ہے… | Searching… | |
| `search-error` | فہرست نہیں کھل سکی۔ | The list could not be opened. | |
| `search-no-match` | کوئی نام نہیں ملا۔ | Nothing found. | |
| `search-add-new` | نیا شامل کریں | Add new | |

## settings

| ID | Urdu | English | Result |
|---|---|---|---|
| `settings-title` | ترتیبات | Settings | |
| `settings-tabs` | ترتیبات کے حصے | Settings sections | |
| `settings-tab-firm` | فرم | Firm | |
| `settings-tab-language` | زبان اور متن | Language and text | |
| `settings-tab-lock` | خودکار تالا | Auto lock | |
| `settings-saved` | ترتیبات محفوظ ہو گئیں | Settings saved | |
| `settings-load-failed` | ترتیبات نہیں کھل سکیں | Settings could not open | |
| `settings-firm-name-ur` | فرم کا نام (اردو) | Firm name (Urdu) | |
| `settings-firm-name-en` | فرم کا نام (انگریزی) | Firm name (English) | |
| `settings-firm-mandi-ur` | منڈی (اردو) | Mandi (Urdu) | |
| `settings-firm-mandi-en` | منڈی (انگریزی) | Mandi (English) | |
| `settings-firm-phone` | فون | Phone | |
| `settings-firm-address` | پتہ | Address | |
| `settings-firm-contacts-title` | پرنٹ کے لیے رابطہ افراد | Contact persons for prints | |
| `settings-firm-contacts-help` | زیادہ سے زیادہ 3 افراد۔ ہر نام اور فون ہر پرنٹ میں فرم کے نام کے نیچے چھپتا ہے، مثلاً "منشی آصف بھٹی 0300-4058860"۔ | Up to 3 people. Each name and phone prints under the firm name on every print, for example "Munshi Asif Bhatti 0300-4058860". | |
| `settings-firm-contacts-empty` | ابھی کوئی رابطہ شخص نہیں۔ پرنٹ کے اوپر رابطے کی سطر نہیں آئے گی۔ | No contact person yet. The print header shows no contact line. | |
| `settings-firm-contact-title` | رابطہ شخص { $number } | Contact person { $number } | |
| `settings-firm-contact-name-ur` | نام (اردو) | Name (Urdu) | |
| `settings-firm-contact-name-en` | نام (انگریزی) | Name (English) | |
| `settings-firm-contact-phone` | فون | Phone | |
| `settings-firm-contact-add` | رابطہ شخص شامل کریں | Add a contact person | |
| `settings-firm-contact-remove` | ہٹائیں | Remove | |
| `settings-firm-contact-remove-label` | رابطہ شخص { $number } ہٹائیں | Remove contact person { $number } | |
| `settings-firm-contacts-full` | زیادہ سے زیادہ تین رابطہ افراد ہو سکتے ہیں۔ نیا شامل کرنے کے لیے کسی کو ہٹائیں۔ | Three contact persons are the most. Remove one to add another. | |
| `settings-firm-trade-mark` | ٹریڈ مارک | Trade Mark | |
| `settings-firm-trade-mark-hint` | ہر پرنٹ کے اوپر ایک ڈبے میں چھپتا ہے، مثلاً RH۔ یہ دونوں زبانوں میں ایک ہی رہتا ہے۔ خالی چھوڑیں تو ڈبہ نہیں آئے گا۔ | Printed in a box in every print header, for example RH. It is the same in both languages. Leave it empty for no box. | |
| `settings-language-default` | طے شدہ زبان | Default language | |
| `settings-language-default-help` | یہ زبان داخل ہونے کے صفحے اور روزانہ بیک اپ پیک کے لیے استعمال ہوتی ہے۔ اپنی زبان آپ اوپر کی پٹی سے خود بدل سکتے ہیں۔ | This language is used for the sign-in screen and the daily backup pack. You change your own language from the top bar. | |
| `settings-text-size` | متن کا سائز | Text size | |
| `settings-text-size-help` | یہ سائز آپ کے لیے محفوظ رہتا ہے۔ | This size is kept for you. | |
| `settings-text-size-value` | موجودہ سائز: <num>{ $size }%</num> | Current size: <num>{ $size }%</num> | |
| `settings-text-bigger` | بڑا کریں | Bigger | |
| `settings-text-smaller` | چھوٹا کریں | Smaller | |
| `settings-text-normal` | عام سائز | Normal size | |
| `settings-lock-minutes` | خودکار تالا (منٹ) | Auto lock after (minutes) | |
| `settings-lock-hint` | اتنے منٹ کوئی کلید نہ دبانے پر ایپ لاک ہو جائے گی۔ آپ کا کام جوں کا توں رہے گا۔ | The app locks after this many minutes with no key pressed. Your work stays as it was. | |
| `settings-tab-printing` | پرنٹنگ | Printing | |
| `settings-tab-day` | دن اور جانچ | Day and check | |
| `settings-tab-audit` | آڈٹ لاگ | Audit log | |
| `settings-tab-app` | ایپ اور اپڈیٹ | App and updates | |
| `settings-day-title` | بندش کا وقت اور خودکار بندش | Closing time and automatic close | |
| `settings-day-closing-time` | بندش کا وقت | Closing time | |
| `settings-day-closing-time-hint` | 24 گھنٹے کے وقت میں گھنٹے اور منٹ لکھیں، مثلاً 21:00 (پاکستان کا وقت)۔ روزانہ کلاؤڈ اور USB بیک اپ اسی وقت ہوتا ہے۔ یہ وہی وقت ہے جو بیک اپ کی سکرین پر ہے۔ | Hours and minutes in 24-hour time, for example 21:00, Pakistan time. The daily cloud and USB backup runs at this time. It is the same time as on the backup screen. | |
| `settings-day-auto-close` | بندش کے وقت دن خود بند ہو جائے | Close the day by itself at closing time | |
| `settings-day-auto-close-help` | یہ آن ہو تو ایپ بند کرنے کے وقت پر آج کا دن خود بند کر دیتی ہے۔ اس کے بعد آج کی تاریخ میں کوئی اندراج نہیں ہو سکتا جب تک آپ وجہ لکھ کر دن دوبارہ نہ کھولیں۔ اگر کوئی مسودہ محفوظ نہ ہو تو یہ انتظار کرتی ہے۔ | When this is on, the app closes today by itself at the closing time. After that, no more entries can be made for today unless you reopen the day with a reason. It waits while a draft is not saved. | |
| `settings-day-auto-close-off-note` | بند: آپ دن خود «دن بند کریں» اسکرین سے بند کرتے ہیں۔ | Off: you close the day yourself from the day close screen. | |
| `settings-grouping-title` | ہندسوں کی ترتیب | Number grouping | |
| `settings-grouping-help` | بڑی رقمیں سکرین اور پرنٹ پر کیسے لکھی جائیں۔ لاکھ کی ترتیب میں پہلے تین ہندسوں کے بعد دو دو ہندسے ساتھ آتے ہیں۔ | How big amounts are written on the screen and on prints. In the lakh style the digits after the first three go in pairs. | |
| `settings-grouping-international` | بین الاقوامی (<num>1,316,040</num>) | International (<num>1,316,040</num>) | |
| `settings-grouping-lakh` | لاکھ (<num>13,16,040</num>) | Lakh (<num>13,16,040</num>) | |
| `settings-check-title` | حساب کی جانچ | Books check | |
| `settings-check-help` | یہ دیکھتی ہے کہ حساب برابر ہے، ایپ کے باہر کسی نے ریکارڈ نہیں بدلا، ڈیٹا فائل ٹھیک ہے اور بیک اپ تازہ ہیں۔ یہ دن میں ایک بار خود بھی چلتی ہے۔ | Checks that the books balance, that nobody has changed the records outside the app, that the data file is healthy and that the backups are recent. It also runs by itself about once a day. | |
| `settings-check-run` | ابھی جانچیں | Check now | |
| `settings-check-running` | جانچ ہو رہی ہے... | Checking... | |
| `settings-check-none` | ابھی کوئی جانچ نہیں ہوئی۔ "ابھی جانچیں" دبائیں۔ | Nothing has been checked yet. Press "Check now". | |
| `settings-check-failed` | جانچ کا نتیجہ پڑھا نہیں جا سکا۔ | The check could not be read. | |
| `settings-check-last` | آخری جانچ: { $time } | Last checked: { $time } | |
| `settings-check-ok` | سب ٹھیک ہے | Everything is in order | |
| `settings-check-bad` | کچھ توجہ طلب ہے | Something needs your attention | |
| `settings-check-books` | حساب | Books | |
| `settings-check-books-balanced` | بنام اور جمع کے میزان برابر ہیں (<num>{ $entries }</num> اندراجات دیکھے گئے)۔ | The Debit and Credit totals are equal (<num>{ $entries }</num> entries checked). | |
| `settings-check-books-difference` | بنام اور جمع کے میزان میں { $difference } کا فرق ہے۔ مدد کے لیے رابطہ کریں۔ | The Debit and Credit totals differ by { $difference }. Contact support. | |
| `settings-check-books-not-run` | حساب کی جانچ نہیں ہو سکی۔ دوبارہ کوشش کریں۔ پھر بھی نہ ہو تو مدد کے لیے رابطہ کریں۔ | The books could not be checked. Try again. If it happens again, contact support. | |
| `settings-check-journal-ok` | ایپ کے باہر کوئی اندراج نہیں بدلا گیا۔ | No posted entry has been changed outside the app. | |
| `settings-check-journal-bad` | اندراج { $entry } اپنی مہر سے نہیں ملتا۔ ہو سکتا ہے کسی نے ایپ کے باہر ریکارڈ بدلا ہو۔ مدد کے لیے رابطہ کریں۔ | The posted entry { $entry } does not match its seal. Someone may have changed the records outside the app. Contact support. | |
| `settings-check-journal-bad-unknown` | ایک اندراج اپنی مہر سے نہیں ملتا۔ ہو سکتا ہے کسی نے ایپ کے باہر ریکارڈ بدلا ہو۔ مدد کے لیے رابطہ کریں۔ | A posted entry does not match its seal. Someone may have changed the records outside the app. Contact support. | |
| `settings-check-audit-ok` | آڈٹ لاگ بدلا نہیں گیا۔ | The audit log has not been changed. | |
| `settings-check-audit-bad` | آڈٹ کا اندراج { $entry } اپنی مہر سے نہیں ملتا۔ ہو سکتا ہے کسی نے ایپ کے باہر ریکارڈ بدلا ہو۔ مدد کے لیے رابطہ کریں۔ | The audit entry { $entry } does not match its seal. Someone may have changed the records outside the app. Contact support. | |
| `settings-check-audit-bad-unknown` | آڈٹ کا ایک اندراج اپنی مہر سے نہیں ملتا۔ ہو سکتا ہے کسی نے ایپ کے باہر ریکارڈ بدلا ہو۔ مدد کے لیے رابطہ کریں۔ | An audit entry does not match its seal. Someone may have changed the records outside the app. Contact support. | |
| `settings-check-data` | ڈیٹا فائل | Data file | |
| `settings-check-data-ok` | شروع میں ڈیٹا فائل کی جانچ ٹھیک رہی۔ | The data file passed its check at start-up. | |
| `settings-check-data-checking` | ڈیٹا فائل کی جانچ ابھی ہو رہی ہے۔ | The data file is still being checked. | |
| `settings-check-data-failed` | ڈیٹا فائل کی جانچ ناکام ہوئی۔ نیا کام نہ لکھیں۔ آخری اچھا بیک اپ بحال کریں اور مدد کے لیے رابطہ کریں۔ | The data file check failed. Do not enter new work. Restore the last good backup and contact support. | |
| `settings-check-data-reference` | حوالہ: <code>{ $reference }</code> | Reference: <code>{ $reference }</code> | |
| `settings-check-backup` | بیک اپ | Backups | |
| `settings-check-backup-unknown` | بیک اپ کا حال معلوم نہیں۔ | The state of the backups is not known. | |
| `settings-check-backup-off` | کلاؤڈ بیک اپ ابھی ترتیب نہیں دیا گیا۔ | Cloud backup is not set up yet. | |
| `settings-check-backup-never` | ابھی تک کوئی کلاؤڈ کاپی نہیں بنی۔ | No cloud copy has been made yet. | |
| `settings-check-backup-green` | آخری کلاؤڈ کاپی { $hours } گھنٹے پرانی ہے۔ | The last cloud copy is { $hours } hours old. | |
| `settings-check-backup-red` | آخری کلاؤڈ کاپی { $hours } گھنٹے پرانی ہے: ایک دن سے زیادہ۔ | The last cloud copy is { $hours } hours old: more than a day. | |
| `settings-check-backup-alert` | آخری کلاؤڈ کاپی { $hours } گھنٹے پرانی ہے: دو دن سے زیادہ۔ | The last cloud copy is { $hours } hours old: more than two days. | |
| `settings-check-backup-queue` | { $count } کاپیاں بھیجنے کے انتظار میں ہیں۔ | { $count -> [one] { $count } copy is waiting to be sent. *[other] { $count } copies are waiting to be sent. } | |
| `settings-check-backup-errors` | بیک اپ کی { $count } خرابیاں ہیں۔ بیک اپ کی سکرین دیکھیں۔ | { $count -> [one] { $count } backup problem. Look at the backup screen. *[other] { $count } backup problems. Look at the backup screen. } | |
| `settings-app-title` | اپڈیٹ | Updates | |
| `settings-app-version` | ورژن { $version } | Version { $version } | |
| `settings-update-help` | جب انٹرنیٹ ملتا ہے تو ایپ نیا ورژن دیکھتی ہے۔ ایپ اپڈیٹ صرف آپ کی اجازت سے، کام کے دوران کبھی نہیں، اور مکمل بیک اپ کے بعد ہی کرتی ہے۔ | The app looks for a new version when it can reach the internet. It installs one only when you agree, never in the middle of work, and only after a full backup. | |
| `settings-update-checking` | نیا ورژن دیکھا جا رہا ہے... | Looking for a new version... | |
| `settings-update-up-to-date` | ایپ تازہ ترین ہے۔ | The app is up to date. | |
| `settings-update-available` | نیا ورژن دستیاب ہے: { $version } | A new version is available: { $version } | |
| `settings-update-date` | جاری ہوا: { $date } | Released: { $date } | |
| `settings-update-notes` | نیا کیا ہے | What is new | |
| `settings-update-no-notes` | اس ورژن کے ساتھ کوئی نوٹ نہیں آیا۔ | No notes came with this version. | |
| `settings-update-check-again` | دوبارہ دیکھیں | Look again | |
| `settings-update-install` | ابھی اپڈیٹ کریں | Update now | |
| `settings-update-has-drafts` | غیر محفوظ اندراجات ہیں ({ $drafts })۔ اپڈیٹ سے پہلے انہیں محفوظ کریں یا چھوڑ دیں۔ | There are unsaved entries ({ $drafts }). Save them or throw them away before you update. | |
| `settings-update-confirm-title` | ورژن { $version } پر اپڈیٹ کریں؟ | Update to version { $version }? | |
| `settings-update-confirm-body` | ایپ پہلے مکمل بیک اپ لیتی ہے، پھر نیا ورژن انسٹال کر کے بند ہو جاتی ہے۔ بند ہونے کے بعد اسے دوبارہ کھولیں۔ اپڈیٹ دن بند کرنے کے بعد کریں، کام کے بیچ میں نہیں۔ ابھی کوئی غیر محفوظ اندراج نہیں ہے۔ | The app makes a full backup first, then installs the new version and closes. Open it again when it has closed. Update after you close the day, not in the middle of work. There are no unsaved entries now. | |
| `settings-update-confirm-yes` | ہاں، اپڈیٹ کریں | Yes, update | |
| `settings-update-installing` | اپڈیٹ ہو رہی ہے۔ ایپ حفاظتی کاپی بناتی ہے، نیا ورژن انسٹال کرتی ہے اور خود بند ہو جاتی ہے۔ کمپیوٹر بند نہ کریں۔ | Updating. The app makes a safety copy, installs the new version and closes by itself. Do not switch off the computer. | |
| `settings-update-done` | ورژن { $version } انسٹال ہو گیا۔ حفاظتی کاپی { $file } ہے۔ ایپ اب بند ہو رہی ہے۔ | Version { $version } is installed. The safety copy is { $file }. The app closes now. | |
| `settings-support-title` | مدد کے لیے معلومات | Information for support | |
| `settings-support-help` | لاگ، ایپ کا ورژن اور جانچ کے نتیجے ایک فائل میں جمع کرتی ہے جو آپ ہمیں بھیج سکتے ہیں۔ اس میں کوئی پاس ورڈ، کوئی کلید اور کوئی بقایا نہیں ہوتا۔ | Collects the log, the app version and the check results into one file that you can send to us. It has no password, no key and no balances. | |
| `settings-support-save` | مدد کی فائل محفوظ کریں | Save support file | |
| `settings-support-saved` | مدد کی فائل { $path } کے نام سے محفوظ ہو گئی۔ اس میں لاگ کی { $lines } سطریں ہیں۔ | The support file is saved as { $path }. It holds { $lines } log lines. | |
| `settings-support-path-refused` | پوری فائل کا نام چنیں جو .txt پر ختم ہو اور فولڈر موجود ہو۔ | Choose a full file name ending in .txt, in a folder that exists. | |
| `settings-support-file-type` | ٹیکسٹ فائل | Text file | |

## translit

| ID | Urdu | English | Result |
|---|---|---|---|
| `translit-mode-roman` | رومن | Roman | |
| `translit-mode-urdu` | اردو | Urdu | |
| `translit-mode-english` | EN | EN | |
| `translit-mode-label` | ٹائپ کا طریقہ | Typing mode | |
| `translit-mode-full-roman` | رومن سے اردو: انگریزی حروف لکھیں، اردو بنتی ہے | Roman to Urdu: type English letters and Urdu appears | |
| `translit-mode-full-urdu` | اردو کی بورڈ: انگریزی کلیدیں اردو حروف لکھتی ہیں | Urdu keyboard: English keys type Urdu letters | |
| `translit-mode-full-english` | انگریزی: کوئی تبدیلی نہیں | English: no conversion | |
| `translit-mode-switch-hint` | طریقہ بدلنے کے لیے { $key } دبائیں۔ | Press { $key } to change the mode. | |
| `translit-suggestions-label` | اردو ہجے کی تجاویز | Urdu spellings | |
| `translit-keymap-open` | کی بورڈ کا نقشہ | Key map | |
| `translit-keymap-title` | اردو کی بورڈ: ہر حرف کی کلید | Urdu keyboard: the key for each letter | |
| `translit-keymap-close` | نقشہ بند کریں | Close the key map | |
| `translit-words-title` | سیکھے ہوئے الفاظ | Learned words | |
| `translit-words-help` | جب آپ کسی لفظ کا دوسرا ہجے چنتے ہیں تو ایپ اسے یاد رکھتی ہے اور اگلی بار پہلے دکھاتی ہے۔ یہاں سے کسی لفظ کو ہٹا سکتے ہیں۔ | When you pick another spelling for a word, the app remembers it and offers it first next time. You can remove a word here. | |
| `translit-words-table` | سیکھے ہوئے الفاظ کی فہرست | List of learned words | |
| `translit-words-col-roman` | انگریزی ہجے | English spelling | |
| `translit-words-col-urdu` | اردو ہجے | Urdu spelling | |
| `translit-words-col-kind` | قسم | Kind | |
| `translit-words-col-count` | استعمال کتنی بار | Times used | |
| `translit-words-col-last` | آخری استعمال | Last used | |
| `translit-words-kind-name` | نام | Name | |
| `translit-words-kind-place` | جگہ | Place | |
| `translit-words-kind-item` | قسم مال | Item | |
| `translit-words-kind-word` | لفظ | Word | |
| `translit-words-remove` | ہٹائیں | Remove | |
| `translit-words-confirm-title` | یہ لفظ ہٹائیں؟ | Remove this word? | |
| `translit-words-confirm-description` | { $roman } اور { $urdu } کی جوڑی ہٹا دی جائے گی۔ ایپ آئندہ اسے پہلے نہیں دکھائے گی۔ محفوظ شدہ نام نہیں بدلیں گے۔ | The pair { $roman } and { $urdu } will be removed. The app will not offer it first any more. Saved names do not change. | |
| `translit-words-removed` | لفظ ہٹا دیا گیا | The word is removed | |
| `translit-words-empty-title` | ابھی کوئی لفظ نہیں سیکھا | No learned words yet | |
| `translit-words-empty-help` | جب آپ کسی نام کا دوسرا ہجے چنیں گے تو وہ یہاں آ جائے گا۔ | When you pick another spelling for a name, it will appear here. | |
| `translit-words-load-failed` | سیکھے ہوئے الفاظ نہیں کھل سکے | The learned words could not open | |
| `translit-pair-urdu-hint` | اردو حروف میں لکھیں، یا انگریزی حروف لکھیں تو اردو بن جائے گی۔ طریقہ بدلنے کے لیے { $key } دبائیں۔ | Type Urdu letters, or type English letters and Urdu appears. Press { $key } to change the mode. | |
| `translit-pair-english-hint` | انگریزی حروف میں لکھیں۔ یہ اردو نام سے خود بھر جاتا ہے، اسے بدل سکتے ہیں۔ | English letters only. It fills by itself from the Urdu name, and you can change it. | |
| `translit-pair-english-hint-first` | نام انگریزی حروف میں لکھیں۔ اردو نام خود بھر جائے گا۔ | Type the name in English letters. The Urdu name fills by itself. | |
| `translit-pair-urdu-hint-second` | یہ انگریزی نام سے خود بھر جاتا ہے۔ آپ اردو حروف بھی لکھ سکتے ہیں، یا { $key } سے طریقہ بدل سکتے ہیں۔ | This fills by itself from the English name. You can also type Urdu letters, or change the mode with { $key }. | |
| `translit-spelling-check` | ہجے دیکھ لیں | Check the spelling | |
| `translit-spelling-title` | ہجے دیکھ لیں | Check the spelling | |
| `translit-spelling-body` | نشان زدہ لفظوں کے ہجے ایپ نے اپنے اصولوں سے بنائے ہیں، کیونکہ یہ لفظ اس کی لفظوں کی فہرست میں نہیں ہیں۔ ہجے ٹھیک ہوں تو تصدیق کریں۔ ٹھیک نہ ہوں تو واپس جا کر درست کریں۔ | The app made the spelling of the marked words from its own rules, because they are not in its word list. If the spelling is right, confirm it. If it is not right, go back and fix it. | |
| `translit-spelling-urdu-name` | اردو نام | Urdu name | |
| `translit-spelling-english-name` | انگریزی نام | English name | |
| `translit-spelling-confirm` | ہجے ٹھیک ہیں | Spelling is right | |

## trips

| ID | Urdu | English | Result |
|---|---|---|---|
| `trips-title` | مارکیٹ سفر .gender = masculine | Market Trip | |
| `trips-help` | کچھ شراکت دار کسی بڑی منڈی میں آدمی بھیج کر مال خریدواتے ہیں۔ یہ مال انہی شراکت داروں کا ہوتا ہے۔ دکان اسے بیچتی ہے اور اپنا معمول کا کمیشن .gender = masculine لیتی ہے۔ ہر سفر یہاں اپنے مال اور نتیجے کے ساتھ نظر آتا ہے۔ | Some partners send a person to a bigger market to buy goods. The goods belong to those partners. The shop sells them and takes its normal Commission. Each trip is shown here with its goods and its result. | |
| `trips-new` | نیا سفر | New trip | |
| `trips-load-failed` | سفر نہیں کھل سکا | The trip could not be opened | |
| `trips-list-label` | سفروں کی فہرست | List of trips | |
| `trips-list-from` | از | From | |
| `trips-list-to` | تا | To | |
| `trips-list-status` | دکھائیں | Show | |
| `trips-list-search` | سفر تلاش کریں | Find a trip | |
| `trips-state-all` | سب | All | |
| `trips-state-open` | کھلے | Open | |
| `trips-state-closed` | بند | Closed | |
| `trips-list-failed` | سفروں کی فہرست نہیں کھل سکی | The list of trips could not be opened | |
| `trips-list-empty` | ابھی کوئی سفر نہیں | No trip yet | |
| `trips-list-empty-help` | بٹن دبا کر پہلا سفر بنائیں۔ اس میں شامل شراکت دار چنیں، پھر جو خریدا گیا وہ درج کریں۔ | Press the button to make the first trip. Choose the partners in it, then record what is bought. | |
| `trips-list-no-match` | ان انتخابات سے کوئی سفر نہیں ملا | No trip matches these choices | |
| `trips-list-no-match-help` | دوسری تاریخیں، کھلے یا بند سفر، یا دوسرے الفاظ آزمائیں۔ | Try other days, open or closed trips, or other words. | |
| `trips-list-clear` | سب سفر دکھائیں | Show all trips | |
| `trips-list-count` | { $total } میں سے { $shown } نظر آ رہے ہیں | Showing { $shown } of { $total } | |
| `trips-list-more` | مزید دکھائیں | Show more | |
| `trips-number` | سفر-{ $number } | TRP-{ $number } | |
| `trips-cost-number` | سخ-{ $number } | TC-{ $number } | |
| `trips-sale-number` | دف-{ $number } | OS-{ $number } | |
| `trips-col-number` | نمبر | No. | |
| `trips-col-trip` | سفر | Trip | |
| `trips-col-date` | تاریخ | Day | |
| `trips-col-market` | منڈی | Market | |
| `trips-col-status` | حالت | State | |
| `trips-col-sales` | فروخت کی رقم | Sales money | |
| `trips-col-entry` | اندراج | Entry | |
| `trips-col-amount` | رقم | Amount | |
| `trips-col-state` | منسوخ | Cancelled | |
| `trips-col-detail` | تفصیل | Details | |
| `trips-new-title` | نیا سفر | New trip | |
| `trips-new-help` | پہلے چنیں کہ اس سفر میں کون سے شراکت دار ہیں اور ہر ایک کا کتنے فیصد ہے۔ پھر بتائیں کہ ہر شراکت دار کی رقم کہاں سے آئے گی۔ آخر میں تفصیل بھریں۔ | First choose the partners in this trip and the % of each. Then say where each partner's money comes from. Then fill in the details. | |
| `trips-new-save` | سفر بنائیں | Make the trip | |
| `trips-new-shares-title` | شراکت دار اور حصے | Partners and shares | |
| `trips-new-from-last` | پچھلے سفر کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔ | Filled in from the last trip. Check it and change what is different. | |
| `trips-new-firm-shares` | فرم کے حصہ .gender = masculine کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔ | Filled in from the firm's Share. Check it and change what is different. | |
| `trips-new-shares-help` | سفر میں شامل شراکت داروں کے حصے ٹھیک 100 فیصد ہونے چاہییں۔ جس کا حصہ نہ ہو وہ اس میں شامل نہیں۔ پہلے اندراج کے بعد حصے بدلے نہیں جا سکتے۔ | The shares of the partners in the trip must total exactly 100%. A partner with no share is not in it. The shares cannot be changed after the first entry. | |
| `trips-new-no-partners` | ابھی کوئی شراکت دار نہیں بنا، اس لیے سفر نہیں بن سکتا۔ | No partner is set up yet, so a trip cannot be made. | |
| `trips-new-no-partners-link` | شراکت داروں کی ترتیب کھولیں | Open the partner setup | |
| `trips-new-details-title` | تفصیل | Details | |
| `trips-created` | { $number } بن گیا | { $number } made | |
| `trips-field-date` | تاریخ | Date | |
| `trips-field-market` | منڈی | Market | |
| `trips-field-market-hint` | جہاں سے مال خریدا جائے، جیسا آپ لکھنا چاہیں۔ | Where the goods are bought, as you want it written. | |
| `trips-field-sent` | کس کو بھیجا | Who was sent | |
| `trips-field-note` | نوٹ | Note | |
| `trips-detail-title` | { $number } | { $number } | |
| `trips-detail-failed` | سفر نہیں کھل سکا | The trip could not be opened | |
| `trips-back` | سب سفر | All trips | |
| `trips-status-open` | کھلا | Open | |
| `trips-status-closed` | بند | Closed | |
| `trips-none` | درج نہیں | Not given | |
| `trips-closed-on` | بند ہوا | Closed on | |
| `trips-figures-title` | مال اور نتیجہ | Goods and result | |
| `trips-partners-title` | اس سفر کے شراکت دار | Partners in this trip | |
| `trips-shares-locked` | اس سفر میں اندراج ہو چکا ہے، اس لیے حصے اب طے ہیں۔ | The shares are fixed now, because the trip has an entry. | |
| `trips-closed-notice` | یہ سفر بند ہے، کیونکہ جو خریدا گیا تھا وہ سب بک گیا یا ضائع ہو گیا۔ اس میں نئی خریداری یا نیا خرچ نہیں ہو سکتا۔ اگر اس کی کوئی فروخت یا ضائع واپس لیا جائے تو یہ دوبارہ کھل جاتا ہے۔ | This trip is closed, because everything bought was sold or written off. It takes no new purchase or cost. It opens again if a sale or a write-off of it is reversed. | |
| `trips-fact-bought` | خریدے نگ | Nag bought | |
| `trips-fact-cost` | کل لاگت | Total cost | |
| `trips-loss-note` | اس سفر میں نقصان ہو رہا ہے۔ | This trip is making a loss. | |
| `trips-partner-results-title` | اس سفر میں ہر شراکت دار کی رقم | Each partner's money in this trip | |
| `trips-partner-results-empty` | ابھی کوئی رقم نہیں چلی۔ | No money has moved yet. | |
| `trips-partner-in-shop` | دکان سے لگائی | Paid from the shop | |
| `trips-partner-in-pocket` | اپنی جیب سے لگائی | Paid from his pocket | |
| `trips-partner-back` | واپس ملی رقم | Money back | |
| `trips-partner-net` | اب تک کا نتیجہ | Result so far | |
| `trips-partner-loss` | نقصان | Loss | |
| `trips-type-label` | اندراج کی قسم | Type of entry | |
| `trips-type-purchase` | خریداری | Purchase | |
| `trips-type-cost` | سفر خرچ | Trip cost | |
| `trips-type-other-sale` | دوسری دکان میں فروخت | Sale in another shop | |
| `trips-entry-new` | نیا اندراج | New entry | |
| `trips-parts-title` | ہر شراکت دار کا حصہ | Each partner's part | |
| `trips-sources-title` | رقم کہاں سے آئے گی | Where the money comes from | |
| `trips-sources-hint` | سفر کے مطابق بھرا گیا ہے۔ اگر کوئی شراکت دار اس اندراج میں کسی اور طرح ادا کرے تو بدل دیں۔ | Filled in from the trip. Change it for this entry if a partner pays another way. | |
| `trips-preview-failed` | اعداد نہیں نکل سکے | The numbers could not be worked out | |
| `trips-saved-purchase` | خریداری { $number } محفوظ ہو گئی | Purchase { $number } saved | |
| `trips-saved-cost` | سفر خرچ { $number } محفوظ ہو گیا | Trip cost { $number } saved | |
| `trips-saved-sale` | دوسری دکان میں فروخت { $number } محفوظ ہو گئی | Sale in another shop { $number } saved | |
| `trips-saved-sale-closed` | دوسری دکان میں فروخت { $number } محفوظ ہو گئی۔ اب سب مال بک چکا ہے، اس لیے سفر بند ہو گیا۔ | Sale in another shop { $number } saved. Everything is sold now, so the trip is closed. | |
| `trips-cost-amount` | رقم | Amount | |
| `trips-cost-amount-hint` | پورے سفر کا خرچ۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 3000+1000۔ | The cost of the whole trip. You can type a sum, for example 3000+1000. | |
| `trips-cost-reason` | وجہ | Reason | |
| `trips-cost-reason-hint` | مثلاً ٹرک یا کھانا۔ | For example the truck or the food. | |
| `trips-cost-numbers-title` | خرچ کیسے بانٹا گیا | How the cost is spread | |
| `trips-cost-numbers-empty` | فارم بھریں تو بانٹ یہاں نظر آئے گی۔ | Fill in the form and the spread shows here. | |
| `trips-cost-numbers-incomplete` | خرچ کی بانٹ دیکھنے کے لیے رقم اور وجہ لکھیں۔ | Enter the amount and the reason to see how the cost is spread. | |
| `trips-cost-spread-title` | ہر خریداری کا حصہ | Part for each purchase | |
| `trips-cost-spread-qty` | ({ $qty } نگ) | ({ $qty } Nag) | |
| `trips-sale-purchase` | مال کہاں سے | Goods from | |
| `trips-sale-purchase-hint` | صرف اسی سفر کی وہ خریداریاں نظر آتی ہیں جن میں مال باقی ہے۔ | Only purchases of this trip with goods left are listed. | |
| `trips-sale-purchase-option` | { $number }: { $item }، { $packing } ({ $qty_left } باقی) | { $number }: { $item }, { $packing } ({ $qty_left } left) | |
| `trips-sale-shop` | دکان | Shop | |
| `trips-sale-qty` | بیچے ہوئے نگ | Nag sold | |
| `trips-sale-amount` | ملی ہوئی رقم | Money received | |
| `trips-sale-amount-hint` | یہ رقم روکڑ میں آتی ہے۔ | The money comes into Cash Book. | |
| `trips-sale-numbers-title` | یہ فروخت | This sale | |
| `trips-sale-numbers-empty` | فارم بھریں تو فروخت کا نتیجہ یہاں نظر آئے گا۔ | Fill in the form and the result of the sale shows here. | |
| `trips-sale-numbers-incomplete` | نتیجہ دیکھنے کے لیے مال چنیں اور مقدار اور رقم لکھیں۔ | Choose the goods and enter the quantity and the money to see the result. | |
| `trips-sale-left-after` | اس کے بعد خریداری میں باقی | Left in the purchase after this | |
| `trips-sale-cost-left-after` | باقی مال کی لاگت | Cost of what is left | |
| `trips-sale-credits-title` | رقم میں ہر شراکت دار کا حصہ | Each partner's part of the money | |
| `trips-purchases-title` | اس سفر کی خریداریاں | Purchases of this trip | |
| `trips-purchases-label` | اس سفر کی خریداریوں کی فہرست | List of the purchases of this trip | |
| `trips-purchases-empty` | ابھی کوئی خریداری نہیں | No purchase yet | |
| `trips-purchases-empty-help` | اوپر کا فارم بھریں اور { $key } دبائیں۔ | Fill in the form above and press { $key }. | |
| `trips-purchases-select-help` | ↑ ↓ سے خریداری پر جائیں۔ اس کے اعداد، نتیجہ اور کام فہرست کے نیچے نظر آتے ہیں۔ | Move to a purchase with ↑ ↓. Its figures, result and actions show under the list. | |
| `trips-entries-title` | اس سفر کا ریکارڈ | History of this trip | |
| `trips-entries-label` | اس سفر کا ریکارڈ | History of this trip | |
| `trips-entries-empty` | ابھی کچھ درج نہیں ہوا | Nothing has been entered yet | |
| `trips-entries-empty-help` | اس سفر کی خریداریاں، خرچ اور فروخت یہاں درج نظر آتی ہیں۔ اوپر کا فارم بھریں اور { $key } دبائیں۔ | The purchases, costs and sales of this trip are listed here. Fill in the form above and press { $key }. | |
| `trips-entries-count` | { $total } میں سے { $shown } نظر آ رہے ہیں | Showing { $shown } of { $total } | |
| `trips-entries-more` | مزید دکھائیں | Show more | |
| `trips-entries-cancel` | اندراج منسوخ کریں | Cancel entry | |
| `trips-entry-label` | { $kind } { $number } | { $kind } { $number } | |
| `trips-entry-cancelled` | منسوخ | Cancelled | |
| `trips-entry-cancelled-reason` | یہ اندراج منسوخ ہو چکا ہے۔ وجہ: { $reason } | This entry is cancelled. Reason: { $reason } | |
| `trips-entry-detail-sale` | { $shop }: { $purchase } سے { $qty } نگ | { $shop }: { $qty } Nag from { $purchase } | |
| `trips-entry-spread-line` | خریداری { $number } کا حصہ ({ $qty } نگ) | Part for Purchase { $number } ({ $qty } Nag) | |
| `trips-cancel-title` | یہ اندراج منسوخ کریں؟ | Cancel this entry? | |
| `trips-cancel-text-cost` | { $entry } منسوخ ہو جائے گا۔ ہر شراکت دار کا حصہ جیسا تھا ویسا ہو جائے گا، اور خریداریوں پر لگا اس خرچ کا حصہ ہٹ جائے گا۔ | { $entry } will be cancelled. Each partner's part goes back as it was, and the part of the cost that the purchases carried is taken off. | |
| `trips-cancel-text-sale` | { $entry } منسوخ ہو جائے گی۔ رقم دوبارہ روکڑ سے نکل جائے گی، مال واپس خریداری میں چلا جائے گا، اور ہر شراکت دار کا حصہ واپس ہو جائے گا۔ | { $entry } will be cancelled. The money goes out of Cash Book again, the goods go back into the purchase, and each partner's part is taken back. | |
| `trips-cancel-reason` | وجہ | Reason | |
| `trips-cancel-confirm` | اندراج منسوخ کریں | Cancel entry | |
| `trips-cancel-confirm-cash` | ہاں، منسوخ کریں | Yes, cancel it | |
| `trips-cancelled` | { $entry } کو منسوخ کر دیا گیا۔ | { $entry } cancelled | |
| `trips-cancelled-reopened` | { $entry } کو منسوخ کر دیا گیا۔ سفر دوبارہ کھل گیا ہے۔ | { $entry } cancelled. The trip is open again. | |

## ventures

| ID | Urdu | English | Result |
|---|---|---|---|
| `ventures-shares-label` | شراکت دار، ان کے حصے اور ان کی رقم کہاں سے آتی ہے | Partners, their shares and where their money comes from | |
| `ventures-share-of` | { $name } کا حصہ | Share of { $name } | |
| `ventures-source-of` | { $name } کی رقم کہاں سے آتی ہے | Where the money of { $name } comes from | |
| `ventures-not-in` | شامل نہیں | Not included | |
| `ventures-source-shop` | دکان سے | From the Shop | |
| `ventures-source-pocket` | اپنی جیب سے | Own Pocket | |
| `ventures-share-percent` | { $value }% | { $value }% | |
| `ventures-sources-label` | ہر شراکت دار کا حصہ کہاں سے آتا ہے | Where each partner's part comes from | |
| `ventures-parts-empty` | فارم بھریں تو ہر شراکت دار کا حصہ یہاں نظر آئے گا۔ | Fill in the form and each partner's part shows here. | |
| `ventures-part-shop` | دکان سے | From the Shop | |
| `ventures-part-pocket` | اپنی جیب سے | Own Pocket | |
| `ventures-parts-shop-total` | روکڑ سے نکلتا ہے اور شراکت داروں کی کل رقم سے کٹتا ہے | Comes out of Cash Book and off the partners' totals | |
| `ventures-parts-pocket-total` | جیب سے ادا ہوا، صرف درج ہوتا ہے | Paid from pockets, recorded only | |
| `ventures-parts-no-posting` | ہر حصہ جیب سے ہے، اس لیے کچھ پوسٹ نہیں ہوتا اور روکڑ نہیں بدلتی۔ | Every part is from a pocket, so nothing is posted and Cash Book does not change. | |
| `ventures-credits-none` | ابھی کوئی رقم نہیں آئی۔ | No money comes in yet. | |

## vouchers

| ID | Urdu | English | Result |
|---|---|---|---|
| `vouchers-field-type` | قسم | Type | |
| `vouchers-type-receipt` | وصولی .gender = feminine | Receipt | |
| `vouchers-type-payment` | ادائیگی .gender = feminine | Payment | |
| `vouchers-type-advance` | پیشگی کھاتہ .gender = masculine | Advance | |
| `vouchers-receipt-title` | وصولی .gender = feminine | Receipt | |
| `vouchers-receipt-help` | گاہک .gender = masculine سے رقم لیں۔ رقم اس کے پرانے بلوں پر پہلے لگتی ہے۔ اگر بیوپاری .gender = masculine اپنی پیشگی .gender = feminine واپس کر رہا ہو تو "وصولی کس سے" میں بیوپاری .gender = masculine چن لیں۔ | Take money from a Buyer. It settles his oldest bills first. If a Seller is giving back his Advance, choose Seller in "Receipt from". | |
| `vouchers-payment-title` | ادائیگی .gender = feminine | Payment | |
| `vouchers-payment-help` | بیوپاری .gender = masculine یا کسی اور کھاتے کو رقم دیں۔ کسی گاہک .gender = masculine کو رقم دینا نقد ادھار .gender = masculine ہے، اس کی وجہ لکھنی ہوگی۔ | Pay a Seller or another account. Money paid to a Buyer is Cash Lent, and a reason is needed. | |
| `vouchers-advance-title` | پیشگی کھاتہ .gender = masculine | Advance | |
| `vouchers-advance-help` | بیوپاری .gender = masculine کو اس کا مال بکنے سے پہلے رقم دیں۔ محفوظ کرنے کے بعد اس کا نیا بقایا نظر آتا ہے۔ | Give a Seller money before his goods are sold. After you save, his new balance shows. | |
| `vouchers-load-failed` | واؤچر کی سکرین کھل نہیں سکی | The voucher screen could not be opened | |
| `vouchers-new` | نیا اندراج | New entry | |
| `vouchers-print` | پرنٹ | Print | |
| `vouchers-edit` | تبدیل کریں | Change | |
| `vouchers-cancel` | واؤچر منسوخ کریں | Cancel voucher | |
| `vouchers-collection-open` | اگراہی .gender = feminine فہرست | Collection list | |
| `vouchers-number-receipt` | و-{ $number } | R-{ $number } | |
| `vouchers-number-payment` | ا-{ $number } | P-{ $number } | |
| `vouchers-number-advance` | پ-{ $number } | A-{ $number } | |
| `vouchers-saved` | { $kind -> [receipt] وصولی .gender = feminine { $number } محفوظ ہو گئی [payment] ادائیگی .gender = feminine { $number } محفوظ ہو گئی *[advance] پیشگی کھاتہ .gender = masculine { $number } محفوظ ہو گیا | { $kind -> [receipt] Receipt { $number } saved [payment] Payment { $number } saved *[advance] Advance { $number } saved | |
| `vouchers-edited` | { $kind -> [receipt] وصولی .gender = feminine { $number } میں تبدیلی محفوظ ہو گئی [payment] ادائیگی .gender = feminine { $number } میں تبدیلی محفوظ ہو گئی *[advance] پیشگی کھاتہ .gender = masculine { $number } میں تبدیلی محفوظ ہو گئی | { $kind -> [receipt] Changes to Receipt { $number } saved [payment] Changes to Payment { $number } saved *[advance] Changes to Advance { $number } saved | |
| `vouchers-cancelled` | { $kind -> [receipt] وصولی .gender = feminine { $number } منسوخ ہو گئی [payment] ادائیگی .gender = feminine { $number } منسوخ ہو گئی *[advance] پیشگی کھاتہ .gender = masculine { $number } منسوخ ہو گیا | { $kind -> [receipt] Receipt { $number } cancelled [payment] Payment { $number } cancelled *[advance] Advance { $number } cancelled | |
| `vouchers-editing-note` | { $kind -> [receipt] آپ وصولی .gender = feminine { $number } میں تبدیلی کر رہے ہیں۔ [payment] آپ ادائیگی .gender = feminine { $number } میں تبدیلی کر رہے ہیں۔ *[advance] آپ پیشگی کھاتہ .gender = masculine { $number } میں تبدیلی کر رہے ہیں۔ | { $kind -> [receipt] You are changing Receipt { $number }. [payment] You are changing Payment { $number }. *[advance] You are changing Advance { $number }. | |
| `vouchers-field-date` | تاریخ | Date | |
| `vouchers-field-from` | وصولی کس سے | Receipt from | |
| `vouchers-from-buyer` | گاہک .gender = masculine | Buyer | |
| `vouchers-from-seller` | بیوپاری .gender = masculine | Seller | |
| `vouchers-field-buyer` | گاہک .gender = masculine | Buyer | |
| `vouchers-field-seller` | بیوپاری .gender = masculine | Seller | |
| `vouchers-field-paid-to` | کس کو ادائیگی | Pay to | |
| `vouchers-field-person` | نام یا فون نمبر | Name or phone | |
| `vouchers-person-hint` | پختہ کھاتہ .gender = masculine میں پہلے سے درج شخص کا نام یا فون لکھیں اور فہرست سے اسے چن لیں۔ | Type the name or phone of a person already recorded under Pukhta Khata and pick him from the list. | |
| `vouchers-make-regular` | باقاعدہ کھاتہ بنائیں | Make a regular account | |
| `vouchers-field-amount` | رقم | Amount | |
| `vouchers-fill-full` | پورا بقایا | Full balance | |
| `vouchers-fill-oldest` | پرانا بل | Oldest bill | |
| `vouchers-field-collector` | اگراہی والا | Collector | |
| `vouchers-field-reason-lent` | نقد ادھار .gender = masculine کی وجہ | Reason for the Cash Lent | |
| `vouchers-reason-lent-hint` | یہ رقم اس کے بلوں میں شامل ہو جاتی ہے اور باقی بلوں کے ساتھ اگراہی .gender = feminine میں آتی ہے۔ | This amount becomes one of his bills and is collected with his other dues (Collection). | |
| `vouchers-field-expected` | متوقع مال یا موسم | Expected goods or season | |
| `vouchers-field-expected-hint` | مثلاً آم کا موسم۔ | For example, the mango season. | |
| `vouchers-field-narration` | تفصیل | Narration | |
| `vouchers-field-edit-reason` | تبدیلی کی وجہ | Reason for the change | |
| `vouchers-edit-reason-hint` | لکھنا ضروری نہیں۔ یہ وجہ تبدیلی کے ریکارڈ میں رہتی ہے۔ | Optional. The reason is kept in the record of the change. | |
| `vouchers-current-balance` | موجودہ بقایا | Current balance | |
| `vouchers-new-balance` | نیا بقایا | New balance | |
| `vouchers-return-limit` | زیادہ سے زیادہ واپسی | Most he can return | |
| `vouchers-extra-credit-note` | یہ رقم اس کے سب بلوں سے زیادہ ہے۔ زائد رقم اس کا کریڈٹ رہے گی اور اگلے بل میں خود لگ جائے گی: | This is more than all his bills. The extra stays as his credit and settles his next bill: | |
| `vouchers-owes-settled-note` | اتنی رقم اس کے ذمے کے بقایا میں لگے گی: | This much settles what he owes the firm: | |
| `vouchers-advance-hint` | اس بیوپاری .gender = masculine کا کوئی بقایا نہیں۔ کیا یہ پیشگی .gender = feminine ہے؟ | Nothing is payable to this Seller. Is this an Advance? | |
| `vouchers-make-advance` | پیشگی .gender = feminine بنائیں | Make it an Advance | |
| `vouchers-bills-title` | کھلے بل | Open bills | |
| `vouchers-bills-failed` | کھلے بل نہیں کھل سکے | The open bills could not be opened | |
| `vouchers-bills-none` | اس گاہک .gender = masculine کا کوئی بل کھلا نہیں۔ | This Buyer has no open bill. | |
| `vouchers-bills-open` | کل کھلا بقایا | Total open | |
| `vouchers-bills-oldest` | سب سے پرانا بل | Oldest bill | |
| `vouchers-bills-credit` | بلوں سے الگ کریڈٹ | Credit apart from the bills | |
| `vouchers-days` | { $days -> [1] ایک دن *[other] { $days } دن | { $days -> [1] 1 day *[other] { $days } days | |
| `vouchers-effect-full` | پورا | Full | |
| `vouchers-effect-part` | کچھ حصہ | Part | |
| `vouchers-source-bikri` | بکری .gender = feminine { $number } | Sale { $number } | |
| `vouchers-source-opening` | ابتدائی بقایا | Opening balance | |
| `vouchers-source-cash-lent` | نقد ادھار .gender = masculine | Cash Lent | |
| `vouchers-source-receipt` | وصولی .gender = feminine { $number } | Receipt { $number } | |
| `vouchers-source-general` | جنرل { $number } | Journal { $number } | |
| `vouchers-source-purchase` | خریداری { $number } | Purchase { $number } | |
| `vouchers-source-fruit` | پھل { $number } | Fruit Taken { $number } | |
| `vouchers-source-other` | دیگر اندراج | Other entry | |
| `vouchers-payables-title` | قابل ادائیگی | Payable | |
| `vouchers-payables-failed` | قابل ادائیگی رقم نہیں کھل سکی | The payable amount could not be opened | |
| `vouchers-payables-none` | اس بیوپاری .gender = masculine کو دینے کے لیے کچھ باقی نہیں۔ | Nothing is payable to this Seller. | |
| `vouchers-payables-open` | کل قابل ادائیگی | Total payable | |
| `vouchers-payables-owes` | بیوپاری .gender = masculine کے ذمے | Owed by the Seller | |
| `vouchers-advances-title` | اس بیوپاری .gender = masculine کی پیشگی کھاتہ .gender = masculine | Advance of this Seller | |
| `vouchers-advances-none` | اس بیوپاری .gender = masculine کو ابھی کوئی پیشگی .gender = feminine نہیں دی گئی۔ | This Seller has no Advance yet. | |
| `vouchers-advances-open` | باقی پیشگی کھاتہ .gender = masculine | Open Advance | |
| `vouchers-advances-last` | آخری پیشگی کھاتہ .gender = masculine | Last Advance | |
| `vouchers-advances-failed` | پیشگی کھاتہ .gender = masculine کی فہرست کھل نہیں سکی | The list of Advance could not be opened | |
| `vouchers-col-date` | تاریخ | Date | |
| `vouchers-col-number` | نمبر | Number | |
| `vouchers-col-source` | ذریعہ | Source | |
| `vouchers-col-account` | کھاتہ | Account | |
| `vouchers-col-amount` | رقم | Amount | |
| `vouchers-col-bill` | بل کی رقم | Bill | |
| `vouchers-col-credit` | جمع | Credit | |
| `vouchers-col-paid` | ادا شدہ | Paid | |
| `vouchers-col-adjusted` | منہا شدہ | Adjusted | |
| `vouchers-col-open` | باقی | Open | |
| `vouchers-col-days` | دن | Days | |
| `vouchers-col-settles` | یہ رقم نمٹائے گی | Settled by this | |
| `vouchers-col-returns` | یہ رقم منہا کرے گی | Adjusted by this | |
| `vouchers-col-state` | حالت | State | |
| `vouchers-state-saved` | محفوظ | Saved | |
| `vouchers-state-cancelled` | منسوخ | Cancelled | |
| `vouchers-hit-cancelled` | منسوخ | cancelled | |
| `vouchers-list-title-receipt` | وصولیوں کی فہرست | List of receipts | |
| `vouchers-list-title-payment` | ادائیگیوں کی فہرست | List of payments | |
| `vouchers-list-title-advance` | پیشگی کھاتہ .gender = masculine کی فہرست | List of advances | |
| `vouchers-list-label` | واؤچروں کی فہرست | List of vouchers | |
| `vouchers-list-from` | از | From | |
| `vouchers-list-to` | تا | To | |
| `vouchers-list-account` | کھاتہ | Account | |
| `vouchers-list-failed` | فہرست کھل نہیں سکی | The list could not be opened | |
| `vouchers-list-empty-receipt` | آج کوئی وصولی نہیں | No receipts today | |
| `vouchers-list-empty-payment` | آج کوئی ادائیگی نہیں | No payments today | |
| `vouchers-list-empty-advance` | آج کوئی پیشگی .gender = feminine نہیں | No advances today | |
| `vouchers-list-empty-help` | اوپر فارم بھریں اور { $key } دبائیں۔ | Fill in the form above and press { $key }. | |
| `vouchers-list-no-match` | ان تاریخوں اور کھاتے کا کوئی واؤچر نہیں | No voucher matches these dates and this account | |
| `vouchers-list-no-match-help` | تاریخیں یا کھاتہ بدل کر دیکھیں۔ | Try other dates or another account. | |
| `vouchers-list-clear` | آج کی فہرست دکھائیں | Show today's list | |
| `vouchers-list-select-help` | کسی واؤچر پر ↑ ↓ سے جائیں اور Enter دبا کر اسے تبدیل کریں۔ | Move to a voucher with ↑ ↓ and press Enter to change it. | |
| `vouchers-view-cancelled` | یہ واؤچر منسوخ ہو چکا ہے۔ وجہ: | This voucher is cancelled. Reason: | |
| `vouchers-view-closed` | اس واؤچر کا دن بند ہو چکا ہے، اس لیے اسے تبدیل نہیں کیا جا سکتا۔ مالک دن دوبارہ کھولے تو تبدیلی ہو سکتی ہے۔ | The day of this voucher is closed, so it cannot be changed. If the Owner reopens the day, it can be. | |
| `vouchers-other-screen` | یہ واؤچر کسی اور سکرین کا ہے | This voucher belongs to another screen | |
| `vouchers-other-screen-help` | اسے روکڑ میں کھولیں۔ | Open it in the Cash Book area. | |
| `vouchers-other-screen-open` | روکڑ کھولیں | Open Cash Book | |
| `vouchers-cancel-title` | واؤچر منسوخ کریں؟ | Cancel this voucher? | |
| `vouchers-cancel-text-receipt` | وصولی .gender = feminine { $number } (<num>{ $amount }</num>، { $name }) منسوخ ہو جائے گی۔ روکڑ اور اس کا کھاتہ پہلے جیسے ہو جائیں گے، اور جو بل اس سے نمٹے تھے وہ دوبارہ کھل جائیں گے۔ | Receipt { $number } (<num>{ $amount }</num>, { $name }) will be cancelled. Cash Book and his account go back as they were, and the bills it settled are open again. | |
| `vouchers-cancel-text-payment` | ادائیگی .gender = feminine { $number } (<num>{ $amount }</num>، { $name }) منسوخ ہو جائے گی۔ روکڑ اور اس کا کھاتہ پہلے جیسے ہو جائیں گے۔ اگر یہ گاہک .gender = masculine کو نقد ادھار .gender = masculine تھا تو اس کا بل بھی ختم ہو جائے گا۔ | Payment { $number } (<num>{ $amount }</num>, { $name }) will be cancelled. Cash Book and the account go back as they were. If it was Cash Lent to a Buyer, that bill is removed too. | |
| `vouchers-cancel-text-advance` | پیشگی کھاتہ .gender = masculine { $number } (<num>{ $amount }</num>، { $name }) منسوخ ہو جائے گا۔ روکڑ اور اس کا کھاتہ پہلے جیسے ہو جائیں گے، اور جو رقم اس پیشگی .gender = feminine سے منہا ہو چکی تھی وہ دوبارہ بیوپاری .gender = masculine کو قابل ادائیگی ہو جائے گی۔ | Advance { $number } (<num>{ $amount }</num>, { $name }) will be cancelled. Cash Book and his account go back as they were, and what was adjusted against this Advance becomes payable to the Seller again. | |
| `vouchers-cancel-reason` | وجہ | Reason | |
| `vouchers-cancel-confirm` | منسوخ کریں | Cancel voucher | |
| `vouchers-cancel-confirm-cash` | ہاں، منسوخ کریں | Yes, cancel it | |
| `vouchers-print-title` | واؤچر { $number } پرنٹ کریں | Print voucher { $number } | |
| `vouchers-collection-title` | اگراہی .gender = feminine فہرست | Collection list | |
| `vouchers-collection-help` | اس دن جن گاہک .gender = masculine سے وصولی کرنی ہے ان کی فہرست۔ صرف وصول ہونے والی رقم لکھیں اور سب ایک ساتھ محفوظ کریں۔ خالی یا 0 والی لائن چھوڑ دی جاتی ہے۔ | The buyers to collect from on this day. Type only the amounts received and save them all together. A row left empty or at 0 is skipped. | |
| `vouchers-collection-back` | وصولی .gender = feminine پر واپس | Back to Receipt | |
| `vouchers-collection-count` | { $count -> [1] فہرست میں ایک گاہک .gender = masculine ہے۔ *[other] فہرست میں { $count } گاہک .gender = masculine ہیں۔ | { $count -> [1] 1 buyer is on the list. *[other] { $count } buyers are on the list. | |
| `vouchers-collection-total` | کل بقایا | Total dues | |
| `vouchers-collection-col-group` | اگراہی والا اور شہر | Collector and city | |
| `vouchers-collection-col-dues` | بقایا | Dues | |
| `vouchers-collection-col-received` | وصول رقم | Received | |
| `vouchers-collection-group-separator` | {"، "} | {", "} | |
| `vouchers-collection-save` | سب محفوظ کریں | Save all | |
| `vouchers-collection-saved` | { $count -> [1] ایک وصولی محفوظ ہو گئی *[other] { $count } وصولیاں محفوظ ہو گئیں | { $count -> [1] 1 receipt saved *[other] { $count } receipts saved | |
| `vouchers-collection-some-refused` | { $count -> [1] ایک وصولی محفوظ ہو گئی *[other] { $count } وصولیاں محفوظ ہو گئیں | { $count -> [1] 1 receipt saved *[other] { $count } receipts saved | |
| `vouchers-collection-problems` | یہ وصولیاں محفوظ نہیں ہوئیں | These receipts were not saved | |
| `vouchers-collection-empty` | اس دن کسی گاہک .gender = masculine کی اگراہی .gender = feminine مقرر نہیں | No collection is planned for this day | |
| `vouchers-collection-empty-help` | گاہک .gender = masculine کے کھاتے میں اگراہی کا دن لکھا ہو تو وہ اس دن کی فہرست میں آتا ہے۔ دوسری تاریخ، اگراہی والا یا شہر چن کر دیکھیں۔ | A buyer comes on the list on the collection day set in his account. Try another date, collector or city. | |
| `vouchers-collection-failed` | اگراہی .gender = feminine فہرست کھل نہیں سکی | The collection list could not be opened | |
| `vouchers-collection-grid-label` | اگراہی .gender = feminine کی فہرست | Collection list | |
| `vouchers-print-help` | یہاں جو پرچی چنیں گے، وہی واؤچر محفوظ کرتے وقت { $key } سے بھی پرنٹ ہوگی۔ | The slip you tick here is also the one { $key } prints when it saves a voucher. | |
| `vouchers-effect-none` | کچھ نہیں | Nothing | |
| `vouchers-print-save-first` | پرنٹ کرنے کے لیے پہلے واؤچر محفوظ کریں۔ | Save the voucher first, then print it. | |
| `vouchers-add-to-list` | فہرست میں رکھیں | Add to list | |
| `vouchers-waiting-title` | محفوظ ہونا باقی | Waiting to be saved | |
| `vouchers-waiting-help` | { $key } دبانے پر یہ سب ایک ایک کر کے محفوظ ہوں گے۔ جو واؤچر محفوظ نہ ہو سکے وہ وجہ کے ساتھ یہیں رہے گا۔ | { $key } saves all of them, one by one. A voucher that cannot be saved stays here with the reason. | |
| `vouchers-waiting-count` | { $count -> [one] ایک واؤچر محفوظ ہونا باقی *[other] { $count } واؤچر محفوظ ہونا باقی | { $count -> [one] One voucher waiting *[other] { $count } vouchers waiting | |
| `vouchers-inactive-hint` | پہلے اس کھاتے کو فعال کریں (انڈکس میں کھاتہ کھول کر)، پھر واؤچر دوبارہ لکھیں۔ | Make this account active first (Index, then the account), then enter the voucher again. | |
| `vouchers-truncated-note` | { $total } میں سے سب سے پرانے { $shown } دکھائے گئے ہیں۔ | The oldest { $shown } of { $total } are shown. | |
| `vouchers-col-type` | قسم | Type | |
| `vouchers-state-waiting` | محفوظ ہونا باقی | Waiting | |
| `vouchers-state-declined` | محفوظ نہیں ہوا: روکڑ کے سوال پر آپ نے واپس جانا چنا | Not saved: you went back at the question about Cash Book | |
| `vouchers-remove` | فہرست سے ہٹائیں | Remove from the list | |
| `vouchers-saved-many` | { $count } واؤچر محفوظ ہو گئے | { $count } vouchers saved | |
| `vouchers-some-refused` | { $count -> [one] ایک واؤچر محفوظ نہیں ہو سکا۔ فہرست میں وجہ دیکھیں۔ *[other] { $count } واؤچر محفوظ نہیں ہو سکے۔ فہرست میں وجہ دیکھیں۔ | { $count -> [one] One voucher could not be saved. See the reason in the list. *[other] { $count } vouchers could not be saved. See the reason in the list. | |
| `vouchers-number-shop-expense` | خ-{ $number } | E-{ $number } | |
| `vouchers-number-personal-expense` | ذ-{ $number } | D-{ $number } | |
| `vouchers-number-personal-return` | ذو-{ $number } | DR-{ $number } | |
| `vouchers-number-money-lent` | ر-{ $number } | L-{ $number } | |
| `vouchers-number-money-lent-return` | رو-{ $number } | LR-{ $number } | |
| `vouchers-number-capital-in` | س-{ $number } | CI-{ $number } | |
| `vouchers-number-capital-out` | سو-{ $number } | CO-{ $number } | |
| `vouchers-number-khata-out` | کد-{ $number } | KO-{ $number } | |
| `vouchers-number-khata-in` | کا-{ $number } | KI-{ $number } | |
| `vouchers-number-general` | ج-{ $number } | G-{ $number } | |
| `vouchers-collection-print-title` | اگراہی .gender = feminine فہرست پرنٹ کریں | Print the Collection list | |
| `vouchers-collection-print-help` | { $date } کی اگراہی .gender = feminine فہرست، جیسی سکرین پر ہے ویسی ہی پرنٹ ہوگی۔ زبان اور پرنٹ کی جگہ چنیں۔ | Prints the Collection list of { $date }, as it is on the screen. Choose the language and where it goes. | |
