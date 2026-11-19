# Urdu changes: plan 40, task F2 (exact Urdu words)

Asked by Umar on 30-10-2026: "use the exact Urdu word, like in English you are using buyers' dues but in Urdu you are using وصولی باقی, which indicates the whole. Go through all the pages."

**210 messages changed** in 27 files, all marked `# review: pending`. Two glossary terms were added, کل اگراہی (Total Dues) and ڈوبی رقم (Dead Amount), as in PRD v1.10 Section 3. No English text was changed. The glossary mandi words (بکری، روکڑ، وصولی، اگراہی، لاگا، منشیانہ، پختہ کھاتہ، پیشگی ...) are unchanged.

In the tables, a glossary word inside a message is written out in its own language (for example {-bikri} shows as بکری or Sale), and {name} marks a value the app fills in.

## The words that were fixed most often

| English meaning | Old Urdu | New Urdu |
|---|---|---|
| Buyers' dues | وصولی باقی | گاہکوں کے ذمے |
| Sellers' dues | (none) | بیوپاریوں کے ذمے |
| Total dues | (none) | کل اگراہی |
| Dead amount | (none) | ڈوبی رقم |
| Cash in hand | روکڑ بقایا | موجود روکڑ |
| Sellers payable | ادائیگی باقی | بیوپاریوں کو ادائیگی باقی |
| dues, owes, owed (any screen) | بقایا | ذمے (بقایا now means only a balance) |
| payable | قابل ادائیگی | ادائیگی باقی |
| "sum" you can type (20+25) | جمع (which is the credit side) | جوڑ |
| books (all the records) | کھاتے (only the accounts) | حساب |
| filters / choices | چھانٹیاں، انتخابات | فلٹر |

## Dashboard

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `dashboard-tile-cash` | Cash in hand | روکڑ بقایا | موجود روکڑ | "Cash in hand": the Urdu "روکڑ بقایا" read as the cash book balance. Fixed in PRD 9.1 as موجود روکڑ. |
| `dashboard-tile-dues` | Buyers' dues | وصولی باقی | گاہکوں کے ذمے | "Buyers' dues": the Urdu "وصولی باقی" read as everything still to collect. PRD 9.1: گاہکوں کے ذمے. |
| `dashboard-tile-payable` | Sellers payable | ادائیگی باقی | بیوپاریوں کو ادائیگی باقی | "Sellers payable": the Urdu "ادائیگی باقی" did not say whose payment. PRD 9.1: بیوپاریوں کو ادائیگی باقی. |
| `dashboard-bikri-empty-help` | Press {$key} to enter the first one. | نئی بکری کے لیے {$key} دبائیں۔ | پہلی بکری لکھنے کے لیے {$key} دبائیں۔ | The English says "the first one"; the Urdu said only "a new sale". |
| `dashboard-dues-title` | Oldest buyer dues | سب سے پرانے بقایا | گاہکوں کے سب سے پرانے ذمے | "Oldest buyer dues": the Urdu "بقایا" is a balance and did not say buyers. |
| `dashboard-dues-all` | All buyer dues | تمام گاہکوں کے بقایا | گاہکوں کے تمام ذمے | "All buyer dues": "بقایا" is a balance, not dues. |
| `dashboard-dues-empty-help` | Dues show here, the oldest first. | بقایا یہاں نظر آئیں گے، سب سے پرانا سب سے اوپر۔ | ذمے یہاں نظر آئیں گے، سب سے پرانے سب سے اوپر۔ | "Dues": "بقایا" is a balance, not dues. |
| `dashboard-chart-items-label` | Commission of the top items this month | اس مہینے کی سب سے اوپر قسم مال کا کمیشن | اس مہینے کی سرفہرست قسم مال کا کمیشن | "top items": "سب سے اوپر" is stiff and not what a munshi says; سرفہرست is the PRD word. |

## App frame, menus and the first-run guide

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `guide-step-dashboard-body` | Today at a glance: sales, cash, dues, partners and what needs your attention. Press any tile to open the screen behind it. | آج کی تصویر ایک نظر میں: بکری، روکڑ، بقایا، شراکت دار اور توجہ طلب کام۔ کسی بھی خانے کو دبائیں تو اس کے پیچھے کی اسکرین کھل جائے گی۔ | آج کی تصویر ایک نظر میں: بکری، روکڑ، گاہکوں اور بیوپاریوں کے ذمے، شراکت دار اور توجہ طلب کام۔ کسی بھی خانے کو دبائیں تو اس کے پیچھے کی اسکرین کھل جائے گی۔ | "dues": "بقایا" is a balance. Now says buyers' and sellers' dues, like the tiles. |

## Shared words and the save-below-zero box

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `common-period-custom` | Choose the dates | تاریخیں خود لکھیں | تاریخیں چنیں | "Choose the dates": the Urdu said "write the dates yourself". |

## Index: accounts and opening balances

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `accounts-inactive-title` | Make this inactive? | غیر فعال کریں؟ | اسے غیر فعال کریں؟ | English says "Make **this** inactive?"; the Urdu dropped "this". Now "اسے غیر فعال کریں؟". |
| `accounts-field-opening-advance` | This opening amount is an Advance | یہ ابتدائی رقم پیشگی کھاتہ ہے | یہ ابتدائی رقم پیشگی ہے | The advance money, not the register: used `{ -peshgi }` (the glossary term for the money given), same as other files. Was `{ -peshgi-khata }` (the screen/register). |
| `accounts-tree-label` | Chart of accounts tree | کھاتوں کا درخت | کھاتوں کی ترتیب کا درخت | English "Chart of **accounts** tree"; Urdu said only "accounts tree". Now "کھاتوں کی ترتیب کا درخت", same words as chart-title. |
| `openings-help` | Write the opening Debit or Credit against each account. Leave the accounts that have none empty. | ہر کھاتے کے سامنے اس کا ابتدائی بنام یا جمع لکھیں۔ جن کھاتوں کا کچھ نہیں، انہیں خالی چھوڑ دیں۔ | ہر کھاتے کے سامنے اس کا ابتدائی بنام یا جمع لکھیں۔ جن کھاتوں کا ابتدائی بقایا نہیں ہے، انہیں خالی چھوڑ دیں۔ | "Leave the accounts that **have none** empty" was "جن کھاتوں کا کچھ نہیں" (nothing, loose). Now says "ابتدائی بقایا نہیں ہے". |
| `openings-name-advance` | {$name} (Advance) | {$name} (پیشگی کھاتہ) | {$name} (پیشگی) | Same reason: the marker beside a seller whose opening debit is an advance now uses `{ -peshgi }`. |

## Index: other masters

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `masters-no-match-title` | Nothing found | کوئی نام نہیں ملا | کچھ نہیں ملا | "Nothing found": the Urdu said "no name found", but lists are also searched by code. |
| `masters-not-found-title` | This one was not found | یہ نام نہیں ملا | یہ نہیں ملا | "This one was not found": the Urdu said "this name". |

## Commission rules and the charges settings

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `rules-help` | Every charge has its rule here. The most specific rule wins. When there is none, the default rule applies. | ہر چارج کا اصول یہاں لکھا جاتا ہے۔ سب سے خاص اصول جیتتا ہے۔ کوئی اصول نہ ہو تو عام اصول لگتا ہے۔ | ہر چارج کا اصول یہاں لکھا جاتا ہے۔ سب سے مخصوص اصول لگتا ہے۔ کوئی اصول نہ ہو تو عام اصول لگتا ہے۔ | "The most specific rule wins": "سب سے خاص" is loose. Now "سب سے مخصوص اصول لگتا ہے". |
| `rules-level-line-override` | Typed on the line | لائن پر لکھی گئی رقم | لائن پر لکھی گئی شرح | "Typed on the line": what is typed on a line is a rate (for example 4%), not an amount. |
| `rules-test-nothing` | Fill in the fields above and press Test to see the result. | نتیجہ دیکھنے کے لیے اوپر کی باتیں لکھ کر آزمائیں دبائیں۔ | نتیجہ دیکھنے کے لیے اوپر کے خانے بھر کر آزمائیں دبائیں۔ | "Fill in the fields": "اوپر کی باتیں" was vague. Now "اوپر کے خانے". |
| `settings-charges-note` | Labour, Munshiana and Phone Charge have no default: they are typed on each Lot. | مزدوری، منشیانہ اور فون کا کوئی عام اصول نہیں؛ وہ ہر لاٹ پر لکھے جاتے ہیں۔ | مزدوری، منشیانہ اور فون کی کوئی عام شرح نہیں؛ وہ ہر لاٹ پر لکھے جاتے ہیں۔ | "no default": the default of a charge is a rate (شرح), as in settings-charges-help, not an "اصول". |
| `settings-overrides-rate-check-hint` | A warning shows on the line when a rate is far above or below the item's average of the last 7 days. It never stops a save. | ریٹ پچھلے 7 دن کی اوسط سے بہت زیادہ یا کم ہو تو لائن پر تنبیہ آتی ہے۔ یہ محفوظ کرنے سے نہیں روکتی۔ | ریٹ اس قسم مال کی پچھلے 7 دن کی اوسط سے بہت زیادہ یا کم ہو تو لائن پر تنبیہ آتی ہے۔ یہ محفوظ کرنے سے نہیں روکتی۔ | English says "the item's average"; the Urdu left out the item. |

## بکری (Sale Book, goods left)

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `bikri-lots-label` | Lots of this sale | لاٹ کی فہرست | اس بکری کے لاٹ | Left out "of this sale": now "اس بکری کے لاٹ". |
| `bikri-more-advance-open` | Open advance: {$amount} | کھلی پیشگی: {$amount} | پیشگی باقی: {$amount} | "Open advance" was "کھلی پیشگی"; fixed word is "پیشگی باقی". |
| `bikri-method-manual` | Whole amount | کل رقم | پوری رقم | "Whole amount" was "کل رقم", which is the glossary word for Partner Total. Now "پوری رقم". |
| `bikri-summary-buyer-total` | Buyer bills | گاہک کی کل رقم | گاہک کے بل | English says "Buyer bills", Urdu said "buyer's total amount". Now "گاہک کے بل". |
| `bikri-draft-text` | A Sale was being typed before the app closed. It was kept at {$time}. Do you want to open it again? | ایپ بند ہونے سے پہلے ایک بکری لکھی جا رہی تھی۔ وہ {$time} پر محفوظ کی گئی تھی۔ کیا اسے دوبارہ کھولنا ہے؟ | ایپ بند ہونے سے پہلے ایک بکری لکھی جا رہی تھی۔ وہ {$time} پر رکھ لی گئی تھی۔ کیا اسے دوبارہ کھولنا ہے؟ | A draft is "kept", not "saved" (محفوظ means a posted sale). Now "رکھ لی گئی تھی". |
| `bikri-stock-from` | Received from | وصول ہونے کی تاریخ سے | مال آنے کی تاریخ سے | "Received" goods said "وصول" (money-receipt word). Now "مال آنے کی تاریخ سے". |
| `bikri-stock-to` | Received up to | وصول ہونے کی تاریخ تک | مال آنے کی تاریخ تک | Same: "مال آنے کی تاریخ تک". |
| `bikri-stock-col-received-date` | Received on | وصولی کا دن | مال آنے کا دن | "Received on" said "وصولی کا دن" (Receipt day). Now "مال آنے کا دن". |
| `bikri-cancel-text` | Sale {$number} will be cancelled. Nothing is deleted: every entry below is reversed on its own day, the money goes back, and the receipts of the buyers become their credit again and are matched to their next oldest bills. The number is never used again. | بکری {$number} منسوخ ہو جائے گی۔ کچھ مٹایا نہیں جاتا: نیچے لکھا ہر اندراج اپنے ہی دن پر الٹ دیا جائے گا، رقم واپس چلی جائے گی، اور گاہک کی وصولیاں اس کا جمع بن کر اس کے اگلے پرانے بلوں سے مل جائیں گی۔ یہ نمبر دوبارہ استعمال نہیں ہوگا۔ | بکری {$number} منسوخ ہو جائے گی۔ کچھ مٹایا نہیں جاتا: نیچے لکھا ہر اندراج اپنے ہی دن پر الٹ دیا جائے گا، رقم واپس چلی جائے گی، اور گاہک کی وصولیاں اس کی جمع رقم بن کر اس کے اگلے سب سے پرانے بلوں پر لگ جائیں گی۔ یہ نمبر دوبارہ استعمال نہیں ہوگا۔ | "Next oldest bills" was "اگلے پرانے بلوں"; "matched to" was "سے مل جائیں گی". Now "اگلے سب سے پرانے بلوں پر لگ جائیں گی"; "اس کا جمع" fixed to "اس کی جمع رقم". |
| `bikri-own-purchase-hint` | The Item, the Packing and the quantity come from the purchase. The lines must sell all of the quantity. Cash paid and the advance tick do not apply to these goods. | قسم مال، پیکنگ اور مقدار خریداری سے آتے ہیں۔ لائنوں میں ساری مقدار بکنی چاہیے۔ اس مال پر نقد اور پیشگی کا نشان لاگو نہیں ہوتے۔ | قسم مال، پیکنگ اور تعداد خریداری سے آتے ہیں۔ لائنوں میں ساری تعداد بکنی چاہیے۔ اس مال پر نقد اور پیشگی کا نشان لاگو نہیں ہوتے۔ | "Quantity" said مقدار; this file says تعداد everywhere else. |
| `bikri-own-numbers-pending` | Enter the quantity and the lines to see what this lot gives to the partners of the trip. | سفر کے شراکت داروں کو اس لاٹ سے کیا ملتا ہے، یہ دیکھنے کے لیے مقدار اور لائنیں لکھیں۔ | سفر کے شراکت داروں کو اس لاٹ سے کیا ملتا ہے، یہ دیکھنے کے لیے تعداد اور لائنیں لکھیں۔ | Same: مقدار to تعداد. |

## وصولی, ادائیگی, پیشگی (the voucher screen)

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `vouchers-receipt-help` | Take money from a Buyer. It settles his oldest bills first. If a Seller is giving back his Advance, choose Seller in "Receipt from". | گاہک سے رقم لیں۔ رقم اس کے پرانے بلوں پر پہلے لگتی ہے۔ اگر بیوپاری اپنی پیشگی واپس کر رہا ہو تو "وصولی کس سے" میں بیوپاری چن لیں۔ | گاہک سے رقم لیں۔ رقم اس کے سب سے پرانے بلوں پر پہلے لگتی ہے۔ اگر بیوپاری اپنی پیشگی واپس کر رہا ہو تو "وصولی کس سے" میں بیوپاری چن لیں۔ | "Oldest bills first" was only "پرانے". Now "سب سے پرانے". |
| `vouchers-editing-note` | {$kind -> [receipt] You are changing Receipt { $number }. [payment] You are changing Payment { $number }. *[advance] You are changing Advance { $number }. } When you save, the old entry is reversed and stays in the history. | {$kind -> [receipt] آپ وصولی { $number } میں تبدیلی کر رہے ہیں۔ [payment] آپ ادائیگی { $number } میں تبدیلی کر رہے ہیں۔ *[advance] آپ پیشگی کھاتہ { $number } میں تبدیلی کر رہے ہیں۔ } محفوظ کرنے پر پرانا اندراج الٹ دیا جاتا ہے اور تاریخ میں موجود رہتا ہے۔ | {$kind -> [receipt] آپ وصولی { $number } میں تبدیلی کر رہے ہیں۔ [payment] آپ ادائیگی { $number } میں تبدیلی کر رہے ہیں۔ *[advance] آپ پیشگی کھاتہ { $number } میں تبدیلی کر رہے ہیں۔ } محفوظ کرنے پر پرانا اندراج الٹ دیا جاتا ہے اور تاریخچے میں موجود رہتا ہے۔ | "Stays in the history" said "تاریخ میں" (reads as "on the date"). Now "تاریخچے میں" (same word as gardens). |
| `vouchers-fill-oldest` | Oldest bill | پرانا بل | سب سے پرانا بل | "Oldest bill" was "پرانا بل". Now "سب سے پرانا بل". |
| `vouchers-reason-lent-hint` | This amount becomes one of his bills and is collected with his other dues (Collection). | یہ رقم اس کے بلوں میں شامل ہو جاتی ہے اور باقی بلوں کے ساتھ اگراہی میں آتی ہے۔ | یہ رقم اس کے بلوں میں شامل ہو جاتی ہے اور اس کے باقی ذمے کے ساتھ اگراہی میں آتی ہے۔ | "His other dues" was "باقی بلوں" (bills, narrower). Now "اس کے باقی ذمے". |
| `vouchers-extra-credit-note` | This is more than all his bills. The extra stays as his credit and settles his next bill: | یہ رقم اس کے سب بلوں سے زیادہ ہے۔ زائد رقم اس کا کریڈٹ رہے گی اور اگلے بل میں خود لگ جائے گی: | یہ رقم اس کے سب بلوں سے زیادہ ہے۔ زائد رقم اس کی جمع رقم رہے گی اور اس کے اگلے بل میں خود لگ جائے گی: | "کریڈٹ" is an English loan; glossary word is جمع. Now "اس کی جمع رقم"; also "اس کے اگلے بل" ("his next bill"). |
| `vouchers-owes-settled-note` | This much settles what he owes the firm: | اتنی رقم اس کے ذمے کے بقایا میں لگے گی: | اتنی رقم سے اس کے ذمے دکان کی رقم ادا ہو جائے گی: | The old wording was hard to follow. Now: "with this amount, what he owes the shop is paid". |
| `vouchers-advance-hint` | Nothing is payable to this Seller. Is this an Advance? | اس بیوپاری کا کوئی بقایا نہیں۔ کیا یہ پیشگی ہے؟ | اس بیوپاری کو دینے کے لیے کچھ باقی نہیں۔ کیا یہ پیشگی ہے؟ | "Nothing is payable" used بقایا for payable. Now "کو دینے کے لیے کچھ باقی نہیں" (same as payables-none). |
| `vouchers-bills-open` | Total open | کل کھلا بقایا | کل کھلی رقم | "Total open" (bills) was "کل کھلا بقایا" (reads as balance). Now "کل کھلی رقم". |
| `vouchers-bills-credit` | Credit apart from the bills | بلوں سے الگ کریڈٹ | بلوں سے الگ جمع رقم | کریڈٹ to "جمع رقم". |
| `vouchers-payables-title` | Payable | قابل ادائیگی | ادائیگی باقی | "Payable" was "قابل ادائیگی"; fixed word is "ادائیگی باقی". |
| `vouchers-payables-failed` | The payable amount could not be opened | قابل ادائیگی رقم نہیں کھل سکی | ادائیگی باقی کی رقم نہیں کھل سکی | Same: "ادائیگی باقی کی رقم". |
| `vouchers-payables-open` | Total payable | کل قابل ادائیگی | کل ادائیگی باقی | "Total payable": "کل ادائیگی باقی". |
| `vouchers-advances-title` | Advance of this Seller | اس بیوپاری کی پیشگی کھاتہ | اس بیوپاری کا پیشگی کھاتہ | Gender: پیشگی کھاتہ is masculine, so "کا", not "کی". |
| `vouchers-advances-open` | Open Advance | باقی پیشگی کھاتہ | پیشگی باقی | "Open Advance" was "باقی پیشگی کھاتہ". Now "پیشگی باقی". |
| `vouchers-advances-last` | Last Advance | آخری پیشگی کھاتہ | آخری پیشگی | "Last Advance": "آخری پیشگی کھاتہ" (an account) to "آخری پیشگی". |
| `vouchers-cancel-title` | Cancel this voucher? | واؤچر منسوخ کریں؟ | یہ واؤچر منسوخ کریں؟ | Left out "this": "یہ واؤچر منسوخ کریں؟". |
| `vouchers-cancel-text-advance` | Advance {$number} ({$amount}, {$name}) will be cancelled. Cash Book and his account go back as they were, and what was adjusted against this Advance becomes payable to the Seller again. | پیشگی کھاتہ {$number} ({$amount}، {$name}) منسوخ ہو جائے گا۔ روکڑ اور اس کا کھاتہ پہلے جیسے ہو جائیں گے، اور جو رقم اس پیشگی سے منہا ہو چکی تھی وہ دوبارہ بیوپاری کو قابل ادائیگی ہو جائے گی۔ | پیشگی کھاتہ {$number} ({$amount}، {$name}) منسوخ ہو جائے گا۔ روکڑ اور اس کا کھاتہ پہلے جیسے ہو جائیں گے، اور جو رقم اس پیشگی سے منہا ہو چکی تھی وہ دوبارہ بیوپاری کو دینی ہوگی۔ | "Becomes payable to the seller" was "قابل ادائیگی ہو جائے گی". Now "بیوپاری کو دینی ہوگی". |
| `vouchers-cancel-confirm` | Cancel voucher | منسوخ کریں | واؤچر منسوخ کریں | English "Cancel voucher"; Urdu was only "منسوخ کریں". Now "واؤچر منسوخ کریں". |
| `vouchers-collection-help` | The buyers to collect from on this day. Type only the amounts received and save them all together. A row left empty or at 0 is skipped. | اس دن جن گاہک سے وصولی کرنی ہے ان کی فہرست۔ صرف وصول ہونے والی رقم لکھیں اور سب ایک ساتھ محفوظ کریں۔ خالی یا 0 والی لائن چھوڑ دی جاتی ہے۔ | اس دن جن گاہکوں سے وصولی کرنی ہے ان کی فہرست۔ صرف وصول ہونے والی رقم لکھیں اور سب ایک ساتھ محفوظ کریں۔ خالی یا 0 والی لائن چھوڑ دی جاتی ہے۔ | Number agreement: "جن گاہک سے" is wrong. Now "جن گاہکوں سے". |
| `vouchers-collection-total` | Total dues | کل بقایا | کل ذمے | "Total dues" of the buyers on this one list. کل اگراہی is the shop-wide tile, so this says کل ذمے, the total of the ذمے column. (English "Total dues" here collides with the tile name; see English issues.) |
| `vouchers-collection-col-dues` | Dues | بقایا | ذمے | "Dues" was "بقایا"; now "ذمے". |

## روکڑ, خرچہ, ادھار رقم, الگ کھاتہ

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `cashbook-help` | All the shop's cash in one place: every amount that came in or went out, from every part of the app, with the cash in hand after each. | دکان کا سارا نقد ایک جگہ: ایپ کے ہر حصے سے آنے اور جانے والی ہر رقم، اور ہر ایک کے بعد ہاتھ میں موجود نقد۔ | دکان کا سارا نقد ایک جگہ: ایپ کے ہر حصے سے آنے اور جانے والی ہر رقم، اور ہر ایک کے بعد موجود روکڑ۔ | "Cash in hand" was "ہاتھ میں موجود نقد"; fixed word is "موجود روکڑ". |
| `cashbook-opening` | Cash at the start | شروع میں نقد | شروع میں روکڑ | Cash balance: نقد to روکڑ (same screen now uses one word). |
| `cashbook-closing` | Cash at the end | آخر میں نقد | آخر میں روکڑ | Same. |
| `cashbook-col-out` | Out | رخصت | نکاسی | "Out" was رخصت (farewell). Money out is نکاسی. |
| `cashbook-col-running` | Cash after | بعد میں نقد | بعد میں روکڑ | Same: "بعد میں روکڑ". |
| `cashbook-col-days-past` | Days late | دن گزرے | تاخیر کے دن | "Days late" was "دن گزرے" (days passed since what?). Now "تاخیر کے دن". |
| `cashbook-kind-bikri-later` | Later sale | بعد کی بکری | بعد کی فروخت | "Later sale" was "بعد کی بکری"; bikri.ftl says "بعد کی فروخت". |
| `cashbook-kind-opening` | Opening balance | ابتدائی بیلنس | ابتدائی بقایا | "Opening balance" was "ابتدائی بیلنس"; the whole app says "ابتدائی بقایا". |
| `cashbook-take-opening` | Opening balance | ابتدائی بیلنس | ابتدائی بقایا | Same. |
| `cashbook-khata-sign-hint` | Plus: the shop holds money for this khata. Minus: it has taken more than it brought. On the balance after an entry, Credit is plus and Debit is minus. | مثبت: دکان کے پاس اس کھاتے کی رقم ہے۔ منفی: اس نے لایا کم اور لیا زیادہ۔ اندراج کے بعد کے بقایا میں جمع مثبت ہے اور بنام منفی۔ | مثبت: دکان کے پاس اس کھاتے کی رقم ہے۔ منفی: اس کھاتے نے لائی ہوئی رقم سے زیادہ رقم لی ہے۔ اندراج کے بعد کے بقایا میں جمع مثبت ہے اور بنام منفی۔ | "It has taken more than it brought" was a broken "اس نے لایا کم اور لیا زیادہ". Now "اس کھاتے نے لائی ہوئی رقم سے زیادہ رقم لی ہے". |
| `cashbook-running-date-only` | The cash after each line shows in date order only. | ہر اندراج کے بعد کی نقد صرف تاریخ کی ترتیب میں نظر آتی ہے۔ | ہر اندراج کے بعد کی روکڑ صرف تاریخ کی ترتیب میں نظر آتی ہے۔ | Same word; this also fixes a gender slip (نقد is masculine, the verb was feminine). |

## پھل (fruit taken)

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `fruit-list-no-match` | No entry matches these choices | ان انتخابات سے کوئی اندراج نہیں ملا | ان فلٹروں سے کوئی اندراج نہیں ملا | "choices" were انتخابات (stiff). Now says the filters, like the other lists. |

## پختہ کھاتہ register

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `pukhta-clear` | Clear the filters | چھانٹیاں ہٹائیں | فلٹر ہٹائیں | "Clear the filters": "چھانٹیاں" -> "فلٹر ہٹائیں" (same as reports-clear-filters, report-clear-filters, audit). |

## Gardens

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `gardens-help` | Some partners rent a garden for a season. They pay the rent (Garden Rent) in Installments and pay its running costs. Its fruit is sold in the shop with the normal Commission, or in another shop. The profit or loss of a garden goes only to the partners in it. | کچھ شراکت دار ایک موسم کے لیے باغ ٹھیکے پر لیتے ہیں۔ وہ ٹھیکہ کی رقم قسط وار ادا کرتے ہیں اور باغ کے خرچ بھی دیتے ہیں۔ اس کا پھل دکان پر عام کمیشن کے ساتھ بکتا ہے، یا کسی اور دکان میں۔ باغ کا نفع یا نقصان صرف انہی شراکت داروں کا ہوتا ہے جو اس میں شامل ہیں۔ | کچھ شراکت دار ایک موسم کے لیے باغ ٹھیکے پر لیتے ہیں۔ وہ ٹھیکہ کی رقم قسط وار ادا کرتے ہیں اور باغ کے خرچ بھی دیتے ہیں۔ اس کا پھل دکان پر عام کمیشن کے ساتھ بکتا ہے، یا کسی اور دکان میں۔ باغ کا منافع یا نقصان صرف انہی شراکت داروں کا ہوتا ہے جو اس میں شامل ہیں۔ | Profit: "نفع" changed to "منافع", the glossary and PRD word used for profit everywhere else. |
| `gardens-list-no-match` | No garden matches these choices | ان انتخابات سے کوئی باغ نہیں ملا | ان فلٹروں سے کوئی باغ نہیں ملا | "choices" were stiff words. Now says the filters. |
| `gardens-col-balance` | Running balance | چلتا حساب | چلتا بقایا | "Running balance": "حساب" is the account, not the balance. Now "بقایا" (glossary Balance). |
| `gardens-col-rent-due` | Rent still due | باقی ٹھیکہ | ٹھیکہ باقی | "Rent still due": same words, now in the "X باقی" order used for ادائیگی باقی and پیشگی باقی. |
| `gardens-col-money-in` | Money in | آنے والی رقم | آئی ہوئی رقم | "Money in": "آنے والی" means still to come. Money in is money that has come. |
| `gardens-col-running-balance` | Balance after | اس کے بعد کا حساب | اس کے بعد کا بقایا | "Balance after": "حساب" (account) changed to "بقایا" (balance). |
| `gardens-status-closed-on` | Closed on {$date} | بند ہوا {$date} کو | {$date} کو بند ہوا | Same meaning. Natural order "{date} کو بند ہوا". |
| `gardens-new-from-defaults` | Filled in from the firm's shares or the last garden. Check it and change what is different. | فرم کے حصہ یا پچھلے باغ کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔ | فرم کے حصوں یا پچھلے باغ کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔ | English says "firm's shares" (plural). {-hissa} is singular, so "فرم کے حصہ" was wrong. Now "حصوں". |
| `gardens-balance` | Running balance (money in less everything paid) | چلتا حساب (آنے والی رقم منفی کل ادائیگی) | چلتا بقایا (آئی ہوئی رقم منفی کل ادا شدہ) | "حساب" to "بقایا"; "آنے والی" to "آئی ہوئی"; "کل ادائیگی" to "کل ادا شدہ", the same as "Everything paid" (gardens-fact-paid). |
| `gardens-rent-due` | Rent still due | باقی ٹھیکہ | ٹھیکہ باقی | "Rent still due": same words, "X باقی" order. |
| `gardens-fact-money-in` | Money in (sales after Commission) | آنے والی رقم (کمیشن کے بعد فروخت) | آئی ہوئی رقم (کمیشن کے بعد فروخت) | "Money in": "آنے والی" (still to come) to "آئی ہوئی" (has come). |
| `gardens-partner-back` | Money back | واپس ملنے والی رقم | واپس ملی رقم | "Money back": "ملنے والی" is money still to be received. The table shows what he got back. |
| `gardens-partner-net-open` | Net so far | اب تک کا نتیجہ | اب تک کا خالص | "Net so far": the Urdu said "result so far" and left out Net. Now "اب تک کا خالص", with the glossary word خالص. |
| `gardens-partner-net-closed` | Net result | آخری نتیجہ | خالص نتیجہ | "Net result": the Urdu "آخری نتیجہ" (last result) left out Net. Now "خالص نتیجہ". |
| `gardens-payment-amount-hint` | Whole rupees. You can type a sum, for example 20000+5000. | پورے روپے۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 20000+5000۔ | پورے روپے۔ آپ جوڑ بھی لکھ سکتے ہیں، مثلاً 20000+5000۔ | "A sum": "جمع" is the glossary word Credit. A sum you type is "جوڑ". |
| `gardens-sale-qty-hint` | A whole number. You can type a sum, for example 20+25. | پورا عدد۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 20+25۔ | پورا عدد۔ آپ جوڑ بھی لکھ سکتے ہیں، مثلاً 20+25۔ | "A sum": "جمع" (Credit) changed to "جوڑ". |
| `gardens-sale-amount` | Money received | ملنے والی رقم | ملی ہوئی رقم | "Money received": "ملنے والی" (to be received) changed to "ملی ہوئی" (received). |
| `gardens-numbers-balance-before` | Running balance before | اس سے پہلے کا چلتا حساب | اس سے پہلے کا چلتا بقایا | "Running balance before": "حساب" to "بقایا". |
| `gardens-numbers-balance-after` | Running balance after | اس کے بعد کا چلتا حساب | اس کے بعد کا چلتا بقایا | "Running balance after": "حساب" to "بقایا". |
| `gardens-numbers-rent-due-after` | Rent still due after | اس کے بعد باقی ٹھیکہ | اس کے بعد ٹھیکہ باقی | "Rent still due after": "X باقی" order. |
| `gardens-cancel-text` | {$entry} will be cancelled. Its posting is reversed, each partner's part goes back as it was, and the entry stays in the history, marked cancelled. | {$entry} منسوخ ہو جائے گا۔ اس کی پوسٹنگ الٹ دی جائے گی، ہر شراکت دار کا حصہ جیسا تھا ویسا ہو جائے گا، اور اندراج منسوخ کے نشان کے ساتھ تاریخچے میں رہے گا۔ | {$entry} کو منسوخ کر دیا جائے گا۔ اس کی پوسٹنگ الٹ دی جائے گی، ہر شراکت دار کا حصہ جیسا تھا ویسا ہو جائے گا، اور اندراج منسوخ کے نشان کے ساتھ تاریخچے میں رہے گا۔ | The entry can be a قسط (feminine) or a خرچ (masculine), so "منسوخ ہو جائے گا" did not always agree. The passive with کو fits both, like gardens-cancelled. |
| `gardens-close-help` | When the goods stop coming, close the garden. Its money goes to the partners by their shares: each gets back what he paid, plus or minus his share of the profit or loss. After that the garden takes no more entries. | جب پھل آنا بند ہو جائے تو باغ بند کریں۔ اس کی رقم شراکت داروں میں ان کے حصوں کے مطابق بٹ جاتی ہے: ہر ایک کو اس کی ادا کی ہوئی رقم واپس ملتی ہے، اور نفع یا نقصان میں اس کا حصہ جمع یا منفی ہوتا ہے۔ اس کے بعد باغ میں مزید اندراج نہیں ہوتا۔ | جب مال آنا بند ہو جائے تو باغ بند کریں۔ اس کی رقم شراکت داروں میں ان کے حصوں کے مطابق بٹ جاتی ہے: ہر ایک کو اس کی ادا کی ہوئی رقم واپس ملتی ہے، اس کے ساتھ منافع میں اس کا حصہ ملتا ہے یا نقصان میں اس کا حصہ کٹ جاتا ہے۔ اس کے بعد باغ میں مزید اندراج نہیں ہوتا۔ | "goods" (not fruit); "profit/loss" said plainly: his share of profit is added, his share of loss is taken off. The old "جمع یا منفی" used جمع, the glossary word Credit. |
| `gardens-close-money-in` | Money in | آنے والی رقم | آئی ہوئی رقم | "Money in": "آنے والی" to "آئی ہوئی". |
| `gardens-close-loss-note` | The garden made a loss. Each partner's part of the loss is in his net result below. | باغ کو نقصان ہوا۔ ہر شراکت دار کا نقصان کا حصہ نیچے اس کے آخری نتیجے میں ہے۔ | باغ کو نقصان ہوا۔ نقصان میں ہر شراکت دار کا حصہ نیچے اس کے خالص نتیجے میں ہے۔ | Now "net result" like the column heading (خالص نتیجہ); word order "his part of the loss" is clearer. |
| `gardens-filter-failed` | The list of gardens could not be read | باغات کی فہرست نہیں کھل سکی | باغات کی فہرست پڑھی نہیں جا سکی | English says "could not be read", the Urdu said "could not be opened". |
| `gardens-arrival-qty-hint` | A whole number. You can type a sum, for example 20+25. | پورا عدد۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 20+25۔ | پورا عدد۔ آپ جوڑ بھی لکھ سکتے ہیں، مثلاً 20+25۔ | "A sum": "جمع" (Credit) changed to "جوڑ". |

## Market trips and purchases

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `purchases-qty-hint` | You can type a sum, for example 20+25. | آپ جمع بھی لکھ سکتے ہیں، مثلاً 20+25۔ | آپ جوڑ بھی لکھ سکتے ہیں، مثلاً 20+25۔ | "A sum": "جمع" (Credit) changed to "جوڑ". |
| `purchases-detail-sales` | Sales money in our shop | ہماری دکان میں فروخت کی رقم | اپنی دکان میں فروخت کی رقم | "our shop": the glossary word is اپنی دکان, not "ہماری دکان". |
| `purchases-entry-total` | Paid now, whole entry | پورے اندراج کی ادا شدہ رقم | پورے اندراج کی ابھی ادا شدہ رقم | "Paid now, whole entry": the Urdu left out "now". |
| `trips-list-no-match` | No trip matches these choices | ان انتخابات سے کوئی سفر نہیں ملا | ان فلٹروں سے کوئی سفر نہیں ملا | "choices" were stiff words. Now says the filters. |
| `trips-new-firm-shares` | Filled in from the firm's Share. Check it and change what is different. | فرم کے حصہ کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔ | فرم کے حصوں کے مطابق بھرا گیا ہے۔ دیکھ لیں اور جو مختلف ہو وہ بدل دیں۔ | Plural: the firm's shares. "فرم کے حصہ" was wrong. Now "حصوں". |
| `trips-partner-results-empty` | No money has moved yet. | ابھی کوئی رقم نہیں چلی۔ | ابھی رقم کا کوئی لین دین نہیں ہوا۔ | "No money has moved yet": "رقم نہیں چلی" is not what a munshi says. Now "کوئی لین دین نہیں ہوا". |
| `trips-partner-in-shop` | Paid from the shop | دکان سے لگائی | دکان سے ادا | "Paid from the shop": "لگائی" (put in) changed to "ادا" (paid), like gardens. |
| `trips-partner-in-pocket` | Paid from his pocket | اپنی جیب سے لگائی | اپنی جیب سے ادا | "Paid from his pocket": "لگائی" to "ادا". |
| `trips-saved-purchases` | {$count} Purchase entries saved: {$first} to {$last} | {$count} خریداری محفوظ ہو گئیں: {$first} سے {$last} تک | خریداری کے {$count} اندراج محفوظ ہو گئے: {$first} سے {$last} تک | English: "{count} Purchase entries saved". "{count} خریداری ... گئیں" is not plural. Now "{-khareedari} کے {count} اندراج", like gardens-saved-arrivals. |
| `trips-cost-amount-hint` | The cost of the whole trip. You can type a sum, for example 3000+1000. | پورے سفر کا خرچ۔ آپ جمع بھی لکھ سکتے ہیں، مثلاً 3000+1000۔ | پورے سفر کا خرچ۔ آپ جوڑ بھی لکھ سکتے ہیں، مثلاً 3000+1000۔ | "A sum": "جمع" (Credit) changed to "جوڑ". |
| `trips-sale-qty` | Nag sold | بیچے ہوئے نگ | بکے ہوئے نگ | "{-nag} sold": "بیچے ہوئے" to "بکے ہوئے", the same as gardens-sale-qty. |
| `trips-entries-title` | History of this trip | اس سفر کا ریکارڈ | اس سفر کا تاریخچہ | "History of this trip": "ریکارڈ" (record) to "تاریخچہ" (history). |
| `trips-entries-label` | History of this trip | اس سفر کا ریکارڈ | اس سفر کا تاریخچہ | "History of this trip": "ریکارڈ" to "تاریخچہ". |

## Partners

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `partners-overview-empty-help` | Add the partners and type their shares. Then the income of every sale is split between them. | شراکت دار شامل کریں اور ان کے حصہ لکھیں، تب ہر بکری کی آمدن ان میں بٹے گی۔ | شراکت دار شامل کریں اور ان کے حصے لکھیں، تب ہر بکری کی آمدن ان میں بٹے گی۔ | English "their shares" is plural. {-hissa} is singular, so "ان کے حصہ" was wrong. Now "حصے". |
| `partners-list-label` | Partners and their shares | شراکت دار اور ان کے حصہ | شراکت دار اور ان کے حصے | "Partners and their shares": plural "حصے". |
| `partners-field-from` | Shares apply from | حصہ کب سے لاگو ہوں | حصے کب سے لاگو ہوں | "Shares apply from": plural "حصے" (the verb ہوں is plural). |
| `partners-save` | Save shares | حصہ محفوظ کریں | حصے محفوظ کریں | "Save shares": plural "حصے". |
| `partners-saved` | Shares saved. They apply from {$date}. | حصہ محفوظ ہو گئے۔ یہ {$date} سے لاگو ہیں۔ | حصے محفوظ ہو گئے۔ یہ {$date} سے لاگو ہیں۔ | "Shares saved": plural "حصے" agrees with "ہو گئے". |
| `partners-history-title` | Share history | حصہ کی تاریخ | حصہ کا تاریخچہ | "Share history": "تاریخ" also means a date. "تاریخچہ" is history (as in gardens and trips). |
| `partners-capital-history-title` | Capital history | سرمایہ کی تاریخ | سرمایہ کا تاریخچہ | "Capital history": "تاریخ" to "تاریخچہ". |
| `partners-capital-pick-help` | His Capital history and balance show here. | اس کے سرمایہ کی تاریخ اور بقایا یہاں نظر آئے گا۔ | اس کے سرمایہ کا تاریخچہ اور بقایا یہاں نظر آئیں گے۔ | "history and balance show here": "تاریخچہ" and the plural verb. |
| `partners-capital-label` | Capital history of the partner | شراکت دار کے سرمایہ کی تاریخ | شراکت دار کے سرمایہ کا تاریخچہ | "Capital history of the partner": "تاریخ" to "تاریخچہ". |
| `partners-capital-failed` | The Capital history could not be opened | سرمایہ کی تاریخ نہیں کھل سکی | سرمایہ کا تاریخچہ نہیں کھل سکا | "history could not be opened": "تاریخ" to "تاریخچہ" and the verb agrees. |
| `partners-statement-pick-help` | His statement shows here. | اس کی تفصیل یہاں نظر آئے گی۔ | اس کا فرد حساب یہاں نظر آئے گا۔ | "His statement": the glossary word for Statement is فرد حساب, not کھاتہ. |
| `partners-statement-detail-trip-write-off` | Market Trip {$trip}: trip goods written off. Nothing is posted; the loss shows in the result of the trip. | مارکیٹ سفر {$trip}: سفر کا مال ضائع کیا گیا۔ کوئی اندراج نہیں ہوا؛ نقصان سفر کے نتیجے میں نظر آتا ہے۔ | مارکیٹ سفر {$trip}: سفر کا مال ضائع کیا گیا۔ کچھ پوسٹ نہیں ہوتا؛ نقصان سفر کے نتیجے میں نظر آتا ہے۔ | "Nothing is posted": "کوئی اندراج نہیں ہوا" said no entry was made. Now "کچھ پوسٹ نہیں ہوتا", like the other posting notes. |
| `partners-statement-detail-fruit-earlier` | Fruit Taken {$fruit}: an earlier posting of an entry that was changed, or its reversal | پھل {$fruit}: ایسے اندراج کی پرانی پوسٹنگ جو بعد میں بدلا گیا، یا اس کی الٹ | پھل {$fruit}: ایسے اندراج کی پرانی پوسٹنگ جو بعد میں بدلا گیا، یا اسے الٹنے والی پوسٹنگ | "or its reversal": "اس کی الٹ" was loose. Now "the posting that reverses it". |
| `partners-ventures-col-back` | Money back | واپس ملنے والی رقم | واپس ملی رقم | "Money back": "ملنے والی" to "ملی". |
| `partners-ventures-col-balance` | Balance or result | چلتا حساب یا نتیجہ | بقایا یا نتیجہ | English says "Balance or result". The Urdu added "running" ("چلتا"). Now "بقایا یا نتیجہ". |

## Analytics

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `analytics-view-c-help` | Each Buyer: purchases, Commission, Laga, what he owes now and how many days he takes to pay. | ہر گاہک: خریداری، کمیشن، لاگا، موجودہ بقایا اور ادائیگی میں لگنے والے اوسط دن۔ | ہر گاہک: خریداری، کمیشن، لاگا، اس وقت اس کے ذمے رقم اور رقم ادا کرنے میں لگنے والے اوسط دن۔ | "what he owes now" was "موجودہ بقایا"; now "اس وقت اس کے ذمے رقم". "days he takes to pay": ادائیگی is the glossary word for paying a seller; a buyer pays by وصولی, so now plain "رقم ادا کرنے میں". |
| `analytics-col-dues-now` | Dues now | موجودہ بقایا | موجودہ ذمے | "Dues now": بقایا is a balance, not dues. The column now reads موجودہ ذمے (what they owe now). |
| `analytics-col-avg-days-to-pay` | Days to pay (average) | ادائیگی کے اوسط دن | رقم ادا کرنے کے اوسط دن | Same ادائیگی clash as view-c-help. Now "رقم ادا کرنے کے اوسط دن". |
| `analytics-col-other-items` | Other items | باقی اقسام | دیگر اقسام | "Other items": "باقی" (left/due) -> "دیگر" (same as analytics-col-other-money, report-doc-other). |
| `analytics-lines-hint` | Use ↑ ↓ to pick a line and Enter to open its Sale. A reversal shows its amounts below zero, so the lines add up to the figure. | لائن چننے کے لیے ↑ ↓ اور اس کی بکری کھولنے کے لیے Enter دبائیں۔ واپس کی گئی بکری کی رقم صفر سے نیچے دکھائی جاتی ہے، تاکہ لائنیں رقم کے برابر رہیں۔ | لائن چننے کے لیے ↑ ↓ اور اس کی بکری کھولنے کے لیے Enter دبائیں۔ الٹ اندراج کی رقم صفر سے نیچے دکھائی جاتی ہے، تاکہ لائنیں رقم کے برابر رہیں۔ | English "A reversal shows its amounts below zero"; Urdu narrowed it to "a returned sale". Now "الٹ اندراج" (same word as report-doc-reversal). |
| `analytics-chart-others` | Others | باقی | دیگر | "Others": "باقی" -> "دیگر" (باقی means due or left). |

## Reports

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `reports-catalog-empty-help` | The reports come in a later version. None of your data is affected. | رپورٹیں اگلے ورژن میں آئیں گی۔ اس وقت آپ کا کوئی ڈیٹا ضائع نہیں ہوتا۔ | رپورٹیں اگلے ورژن میں آئیں گی۔ اس سے آپ کا کوئی ڈیٹا متاثر نہیں ہوتا۔ | English "is affected"; Urdu said "ضائع نہیں" (not lost). Now "متاثر نہیں". |
| `report-r11-help` | Every entry of the days you choose, with Debit and Credit. | چنے ہوئے دنوں کی ہر اندراج، بنام اور جمع کے ساتھ۔ | چنے ہوئے دنوں کا ہر اندراج، بنام اور جمع کے ساتھ۔ | Gender: اندراج is masculine ("ہر اندراج" with "کی" was wrong). Now "کا ہر اندراج". |
| `report-r13-help` | Every entry of one account, with the goods and the running balance. | ایک کھاتے کی ہر اندراج، سامان کی تفصیل اور چلتے بقایا کے ساتھ۔ | ایک کھاتے کا ہر اندراج، مال کی تفصیل اور چلتے بقایا کے ساتھ۔ | Same gender fix; also "سامان" -> "مال" (the app word for goods). |
| `report-empty` | No entries for these filters | ان تاریخوں میں کوئی اندراج نہیں | ان فلٹروں میں کوئی اندراج نہیں | English "these filters"; Urdu said only "these dates" (too narrow). Now "ان فلٹروں میں". |
| `report-col-detail` | Goods | سامان | مال | English "Goods"; "سامان" -> "مال" (same word as report-r5, print-items-value, analytics-goods). |
| `report-r20-title` | Buyer Dues | اگراہی رپورٹ | گاہکوں کے ذمے | English "Buyer Dues"; Urdu was "اگراہی رپورٹ" (reads as all collection). Now "گاہکوں کے ذمے" (fixed word). |
| `report-r21-help` | The same dues, grouped by city, with the total of each city. | یہی بقایا، شہر کے حساب سے، ہر شہر کے میزان کے ساتھ۔ | گاہکوں کے ذمے یہی رقم، شہر کے حساب سے، ہر شہر کے میزان کے ساتھ۔ | "The same dues"; Urdu said "بقایا" (balance). Now "گاہکوں کے ذمے یہی رقم". |
| `report-r22-help` | The same dues, grouped by collector, with the total of each collector. | یہی بقایا، اگراہی والے کے حساب سے، ہر ایک کے میزان کے ساتھ۔ | گاہکوں کے ذمے یہی رقم، اگراہی والے کے حساب سے، ہر ایک کے میزان کے ساتھ۔ | Same as r21-help: "dues" is ذمے, not بقایا. |
| `report-r24-title` | Seller Payables | ادائیگی باقی | بیوپاریوں کو ادائیگی باقی | English "Seller Payables"; Urdu "ادائیگی باقی" left out the sellers. Now "بیوپاریوں کو ادائیگی باقی" (fixed word). |
| `report-r24-help` | What the shop owes each seller, his open Advance and what he owes beyond it. | دکان کے ذمے ہر بیوپاری کا بقایا، اس کی کھلی پیشگی اور جو وہ اس سے زیادہ دکان کا دینا ہے۔ | ہر بیوپاری کو دکان کی کتنی ادائیگی باقی ہے، اس کی باقی پیشگی اور اس سے زیادہ جو رقم اس کے ذمے ہے۔ | "What the shop owes each seller" was "بقایا"; now ادائیگی باقی. "open advance" now "باقی پیشگی" (fixed word); "what he owes beyond it" now "اس کے ذمے". The old tail "دکان کا دینا ہے" was garbled. |
| `report-col-owes-firm` | Owes the shop | دکان کا دینا | بیوپاری کے ذمے | "Owes the shop": "دکان کا دینا" -> "بیوپاری کے ذمے" (owes = ذمے; matches the PRD list name). |
| `report-section-payable` | Payable to sellers | بیوپاریوں کو دینا | بیوپاریوں کو ادائیگی باقی | "Payable to sellers": "بیوپاریوں کو دینا" -> "بیوپاریوں کو ادائیگی باقی" (same as the report title and the dashboard tile). |
| `report-r32-help` | Everyone who borrowed from the shop: lent, returned, still open, and the days past the return-by date. | دکان سے ادھار لینے والا ہر شخص: دی ہوئی، واپس ہوئی اور کھلی رقم، اور واپسی کی تاریخ کے بعد گزرے دن۔ | دکان سے ادھار لینے والا ہر شخص: دی ہوئی، واپس ہوئی اور ابھی باقی رقم، اور واپسی کی تاریخ کے بعد گزرے دن۔ | "still open" was "کھلی رقم"; now "ابھی باقی رقم" (money lent open = باقی). |
| `report-r32-overdue` | Open past the return-by date | واپسی کی تاریخ کے بعد بھی کھلی رقم | واپسی کی تاریخ کے بعد بھی باقی رقم | "Open past the return-by date": "کھلی" -> "باقی" (same reason). |
| `report-r10-help` | Goods received and not yet sold: what came, what was sold, what is left and for how many days. | وصول ہوا مال جو ابھی بکا نہیں: کتنا آیا، کتنا بکا، کتنا باقی ہے اور کتنے دن سے۔ | آیا ہوا مال جو ابھی بکا نہیں: کتنا آیا، کتنا بکا، کتنا باقی ہے اور کتنے دن سے۔ | "Goods received": used "وصول ہوا" (the word for recovered money). Now "آیا ہوا مال" (same as column "آیا"). |
| `report-r23-help` | Every receipt with the buyer, the amount and the collector. A seller returning his advance is here too. A cancelled receipt is listed and not counted. | ہر وصولی خریدار، رقم اور اگراہی والے کے ساتھ۔ پیشگی واپس کرنے والا بیوپاری بھی یہاں ہے۔ منسوخ وصولی فہرست میں رہتی ہے مگر رقم میں شمار نہیں ہوتی۔ | ہر وصولی گاہک، رقم اور اگراہی والے کے ساتھ۔ پیشگی واپس کرنے والا بیوپاری بھی یہاں ہے۔ منسوخ وصولی فہرست میں رہتی ہے مگر رقم میں شمار نہیں ہوتی۔ | "buyer": "خریدار" -> {-gahak} (the word used everywhere else in these files). |
| `report-r34-buyers-dues` | Buyers' Dues | وصولی باقی | گاہکوں کے ذمے | English "Buyers' Dues"; Urdu was "وصولی باقی" (reads as the whole collection). Now "گاہکوں کے ذمے". |
| `report-r35-help` | For one day: what each party owed at the start, what was debited and credited, and what is left. Pick a party to see his slip. A bill edited or cancelled on its own day counts once in the day's columns; the balances agree with the ledger. | ایک دن کا: ہر پارٹی کا شروع کا بقایا، اس دن کا بنام اور جمع، اور جو باقی ہے۔ پرچی دیکھنے کے لیے پارٹی چنیں۔ جو بل اسی دن بدلا یا منسوخ ہوا وہ دن کے خانوں میں ایک ہی بار گنا جاتا ہے؛ بقایا کھاتے سے ملتے ہیں۔ | ایک دن کا: ہر پارٹی کے ذمے شروع میں کتنی رقم تھی، اس دن کا بنام اور جمع، اور جو باقی ہے۔ پرچی دیکھنے کے لیے پارٹی چنیں۔ جو بل اسی دن بدلا یا منسوخ ہوا وہ دن کے خانوں میں ایک ہی بار گنا جاتا ہے؛ بقایا کھاتے سے ملتے ہیں۔ | "what each party owed at the start" was "شروع کا بقایا" (balance, a different thing). Now "ہر پارٹی کے ذمے شروع میں کتنی رقم تھی". See English issues. |
| `report-col-installments` | Installments | قسط | قسطیں | Plural: "Installments" -> "قسطیں" (was the singular term {-qist}). |

## Prints

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `print-preview-empty` | There is nothing to print in this document. | اس دستاویز میں چھاپنے کے لیے کچھ نہیں۔ | اس دستاویز میں پرنٹ کرنے کے لیے کچھ نہیں۔ | "print" was "چھاپنا" here, "پرنٹ" in the rest of the file. Now پرنٹ. |
| `print-box-nothing-printed` | There was nothing to print. | چھاپنے کے لیے کچھ نہیں تھا۔ | پرنٹ کرنے کے لیے کچھ نہیں تھا۔ | Same: "چھاپنے" -> "پرنٹ کرنے". |
| `print-preview-help` | The document will print on paper like this. | کاغذ پر دستاویز ایسے ہی چھپے گی۔ | کاغذ پر دستاویز ایسے ہی پرنٹ ہو گی۔ | Spelling of the future tense (ہو گی, as the style guide fixes ہو گئی). |
| `print-collection-col-dues` | Dues | بقایا | ذمے | "Dues": بقایا is a balance, not dues. Now ذمے. |

## Day close

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `day-reopen-help` | Reopening a date reopens every later date too. A reason is required and is written to the audit log. | ایک تاریخ کھولنے سے اس کے بعد کی ہر تاریخ بھی کھل جاتی ہے۔ وجہ لکھنا ضروری ہے اور وہ آڈٹ لاگ میں لکھی جاتی ہے۔ | ایک تاریخ دوبارہ کھولنے سے اس کے بعد کی ہر تاریخ بھی دوبارہ کھل جاتی ہے۔ وجہ لکھنا ضروری ہے اور وہ آڈٹ لاگ میں لکھی جاتی ہے۔ | English "Reopening ... reopens"; Urdu dropped "again". Added دوبارہ (twice). |
| `day-reopen-confirm-description` | {$date} and every later date will be reopened. The reason will be written to the audit log. | {$date} اور اس کے بعد کی ہر تاریخ کھل جائے گی۔ وجہ آڈٹ لاگ میں لکھی جائے گی۔ | {$date} اور اس کے بعد کی ہر تاریخ دوبارہ کھل جائے گی۔ وجہ آڈٹ لاگ میں لکھی جائے گی۔ | "will be reopened": added دوبارہ. |
| `day-reopened` | {$date} and the days after it are reopened | {$date} اور اس کے بعد کے دن کھل گئے | {$date} اور اس کے بعد کے دن دوبارہ کھل گئے | "are reopened": added دوبارہ. |

## Audit log

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `audit-action-day-reopen` | Day reopened | دن کھولا | دن دوبارہ کھولا گیا | English "Day reopened"; Urdu "دن کھولا" lost "re-" and had no passive. Now "دن دوبارہ کھولا گیا". |
| `audit-action-make-regular` | Made a regular account | مستقل کھاتہ بنایا گیا | باقاعدہ کھاتہ بنایا گیا | "regular": "مستقل" -> "باقاعدہ" (the word on the Pukhta Khata screen title, pukhta-make-regular-title). |
| `audit-entity-backup-cloud-keys` | Cloud backup keys | کلاؤڈ بیک اپ کی کنجیاں | کلاؤڈ بیک اپ کی چابیاں | "key" is "چابی" in backup.ftl; "کنجیاں" -> "چابیاں". |
| `audit-entity-backup-key` | Backup key | بیک اپ کی | بیک اپ کی چابی | English "Backup key"; Urdu "بیک اپ کی" had lost the word "key" (read as "of backup"). Now "بیک اپ کی چابی" (same as backup.ftl). |
| `audit-entity-bank-account` | Bank account | بینک اکاؤنٹ | بینک کھاتہ | "Bank account": "اکاؤنٹ" -> "کھاتہ" (the app word for account, as in audit-entity-account). |

## Settings

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `settings-lock-minutes` | Auto lock after (minutes) | خودکار تالا (منٹ) | خودکار تالا (منٹ بعد) | English "Auto lock **after** (minutes)"; Urdu lost "after". Now "خودکار تالا (منٹ بعد)". |
| `settings-day-auto-close-help` | When this is on, the app closes today by itself at the closing time. After that, no more entries can be made for today unless you reopen the day with a reason. It waits while a draft is not saved. | یہ آن ہو تو ایپ بند کرنے کے وقت پر آج کا دن خود بند کر دیتی ہے۔ اس کے بعد آج کی تاریخ میں کوئی اندراج نہیں ہو سکتا جب تک آپ وجہ لکھ کر دن دوبارہ نہ کھولیں۔ اگر کوئی مسودہ محفوظ نہ ہو تو یہ انتظار کرتی ہے۔ | یہ آن ہو تو ایپ بندش کے وقت پر آج کا دن خود بند کر دیتی ہے۔ اس کے بعد آج کی تاریخ میں کوئی اندراج نہیں ہو سکتا جب تک آپ وجہ لکھ کر دن دوبارہ نہ کھولیں۔ اگر کوئی مسودہ محفوظ نہ ہو تو یہ انتظار کرتی ہے۔ | "closing time" was "بند کرنے کا وقت"; the label of the same setting is "بندش کا وقت". Made the same word. |
| `settings-day-auto-close-off-note` | Off: you close the day yourself from the day close screen. | بند: آپ دن خود «دن بند کریں» اسکرین سے بند کرتے ہیں۔ | آف: آپ دن خود «دن بند کریں» اسکرین سے بند کرتے ہیں۔ | "Off:" was "بند:", which next to "close the day" reads as "closed". Now "آف:" (pairs with "آن ہو تو" in the help text). |
| `settings-grouping-title` | Number grouping | ہندسوں کی ترتیب | ہندسوں کی گروپ بندی | "Number grouping" was "ہندسوں کی ترتیب" (order of digits). Now "ہندسوں کی گروپ بندی". |
| `settings-grouping-help` | How big amounts are written on the screen and on prints. In the lakh style the digits after the first three go in pairs. | بڑی رقمیں اسکرین اور پرنٹ پر کیسے لکھی جائیں۔ لاکھ کی ترتیب میں پہلے تین ہندسوں کے بعد دو دو ہندسے ساتھ آتے ہیں۔ | بڑی رقمیں اسکرین اور پرنٹ پر کیسے لکھی جائیں۔ لاکھ کے انداز میں پہلے تین ہندسوں کے بعد دو دو ہندسے ساتھ آتے ہیں۔ | "In the lakh **style**" was "لاکھ کی ترتیب"; now "لاکھ کے انداز میں" (the word "ترتیب" is no longer used for two ideas). Numbers unchanged. |
| `settings-check-help` | Checks that the books balance, that nobody has changed the records outside the app, that the data file is healthy and that the backups are recent. It also runs by itself about once a day. | یہ دیکھتی ہے کہ حساب برابر ہے، ایپ کے باہر کسی نے ریکارڈ نہیں بدلا، ڈیٹا فائل ٹھیک ہے اور بیک اپ تازہ ہیں۔ یہ دن میں ایک بار خود بھی چلتی ہے۔ | یہ دیکھتی ہے کہ حساب برابر ہے، ایپ کے باہر کسی نے ریکارڈ نہیں بدلا، ڈیٹا فائل ٹھیک ہے اور بیک اپ تازہ ہیں۔ یہ دن میں تقریباً ایک بار خود بھی چلتی ہے۔ | English "runs by itself **about** once a day"; Urdu left out "about". Now "تقریباً ایک بار". |
| `settings-check-journal-ok` | No posted entry has been changed outside the app. | ایپ کے باہر کوئی اندراج نہیں بدلا گیا۔ | ایپ کے باہر کوئی پوسٹ شدہ اندراج نہیں بدلا گیا۔ | English "No **posted** entry has been changed"; Urdu left out "posted". Now "پوسٹ شدہ اندراج". |
| `settings-check-journal-bad` | The posted entry {$entry} does not match its seal. Someone may have changed the records outside the app. Contact support. | اندراج {$entry} اپنی مہر سے نہیں ملتا۔ ہو سکتا ہے کسی نے ایپ کے باہر ریکارڈ بدلا ہو۔ مدد کے لیے رابطہ کریں۔ | پوسٹ شدہ اندراج {$entry} اپنی مہر سے نہیں ملتا۔ ہو سکتا ہے کسی نے ایپ کے باہر ریکارڈ بدلا ہو۔ مدد کے لیے رابطہ کریں۔ | Same: "The **posted** entry" now "پوسٹ شدہ اندراج". |
| `settings-check-journal-bad-unknown` | A posted entry does not match its seal. Someone may have changed the records outside the app. Contact support. | ایک اندراج اپنی مہر سے نہیں ملتا۔ ہو سکتا ہے کسی نے ایپ کے باہر ریکارڈ بدلا ہو۔ مدد کے لیے رابطہ کریں۔ | ایک پوسٹ شدہ اندراج اپنی مہر سے نہیں ملتا۔ ہو سکتا ہے کسی نے ایپ کے باہر ریکارڈ بدلا ہو۔ مدد کے لیے رابطہ کریں۔ | Same: "A **posted** entry" now "ایک پوسٹ شدہ اندراج". |
| `settings-check-data-ok` | The data file passed its check at start-up. | شروع میں ڈیٹا فائل کی جانچ ٹھیک رہی۔ | ایپ شروع ہوتے وقت ڈیٹا فائل کی جانچ ٹھیک رہی۔ | "at start-up" was "شروع میں" (at the beginning, vague). Now "ایپ شروع ہوتے وقت". |
| `settings-check-backup-unknown` | The state of the backups is not known. | بیک اپ کا حال معلوم نہیں۔ | بیک اپ کی حالت معلوم نہیں۔ | "state" was "حال"; the backup files use "حالت". Now "بیک اپ کی حالت معلوم نہیں۔" |
| `settings-check-backup-queue` | {$count -> [one] { $count } copy is waiting to be sent. *[other] { $count } copies are waiting to be sent. } | {$count} کاپیاں بھیجنے کے انتظار میں ہیں۔ | {$count -> [one] { $count } کاپی بھیجنے کے انتظار میں ہے۔ *[other] { $count } کاپیاں بھیجنے کے انتظار میں ہیں۔ } | English has one/other; Urdu had none, so 1 read "1 کاپیاں". Added `[one]`/`*[other]` with correct number agreement. |
| `settings-check-backup-errors` | {$count -> [one] { $count } backup problem. Look at the backup screen. *[other] { $count } backup problems. Look at the backup screen. } | بیک اپ کی {$count} خرابیاں ہیں۔ بیک اپ کی اسکرین دیکھیں۔ | {$count -> [one] بیک اپ کی { $count } خرابی ہے۔ بیک اپ کی اسکرین دیکھیں۔ *[other] بیک اپ کی { $count } خرابیاں ہیں۔ بیک اپ کی اسکرین دیکھیں۔ } | Same: 1 read "1 خرابیاں". Added `[one]`/`*[other]`. |
| `settings-update-has-drafts` | There are unsaved entries ({$drafts}). Save them or throw them away before you update. | غیر محفوظ اندراجات ہیں ({$drafts})۔ اپڈیٹ سے پہلے انہیں محفوظ کریں یا چھوڑ دیں۔ | غیر محفوظ اندراجات ہیں ({$drafts})۔ اپڈیٹ سے پہلے انہیں محفوظ کریں یا ختم کر دیں۔ | "throw them away" was "چھوڑ دیں" (leave them). Now "ختم کر دیں", the same word as `bikri-draft-discard`. |
| `settings-support-help` | Collects the log, the app version and the check results into one file that you can send to us. It has no password, no key and no balances. | لاگ، ایپ کا ورژن اور جانچ کے نتیجے ایک فائل میں جمع کرتی ہے جو آپ ہمیں بھیج سکتے ہیں۔ اس میں کوئی پاس ورڈ، کوئی کلید اور کوئی بقایا نہیں ہوتا۔ | لاگ، ایپ کا ورژن اور جانچ کے نتیجے ایک فائل میں جمع کرتی ہے جو آپ ہمیں بھیج سکتے ہیں۔ اس میں کوئی پاس ورڈ، کوئی چابی اور کوئی بقایا نہیں ہوتا۔ | "no key" was "کلید" (which also means a keyboard key). The backup key is "چابی" everywhere in backup.ftl. Now "کوئی چابی". |

## Backup and restore

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `backup-restore-help` | Your books are replaced by the books in this copy. Look at what is in it first. | آپ کے کھاتے اس کاپی کے کھاتوں سے بدل جائیں گے۔ پہلے دیکھ لیں کہ اس میں کیا ہے۔ | آپ کا حساب اس کاپی کے حساب سے بدل جائے گا۔ پہلے دیکھ لیں کہ اس میں کیا ہے۔ | English "Your **books** are replaced by the books in this copy". Urdu said "کھاتے" (accounts), which is narrower: a restore replaces all the records, not only the accounts list. Now "آپ کا حساب اس کاپی کے حساب سے بدل جائے گا۔" ("حساب" is the app's word for "books": `settings-check-books`, `dashboard-tile-health`). |
| `backup-restore-totals-label` | Books in this copy and books now | اس کاپی کے کھاتے اور اب کے کھاتے | اس کاپی کا حساب اور اب کا حساب | "Books in this copy and books now": "کھاتے" changed to "حساب". |
| `backup-restore-upgrade` | This copy is from an older version of the app. It is upgraded by itself after the restore. | یہ کاپی ایپ کے پرانے ورژن کی ہے۔ بحالی کے بعد وہ خود بخود نئی کر دی جائے گی۔ | یہ کاپی ایپ کے پرانے ورژن کی ہے۔ بحالی کے بعد وہ خود بخود نئے ورژن کے مطابق ہو جائے گی۔ | "It is **upgraded**" was "نئی کر دی جائے گی" (made new, vague). Now "نئے ورژن کے مطابق ہو جائے گی". |
| `backup-restore-confirm-description` | {$lost -> [0] A safety copy of your present data is made first. Then all your books are replaced by the copy made on { $date }. You are signed out. *[other] A safety copy of your present data is made first. Then all your books are replaced by the copy made on { $date }. These will be lost: { $journal } posted entries, { $bikri } Sale entries and { $vouchers } vouchers. You are signed out. } | {$lost -> [0] پہلے آپ کے موجودہ ڈیٹا کی حفاظتی کاپی بنے گی۔ پھر آپ کے سب کھاتے { $date } کو بنی کاپی سے بدل جائیں گے۔ آپ باہر ہو جائیں گے۔ *[other] پہلے آپ کے موجودہ ڈیٹا کی حفاظتی کاپی بنے گی۔ پھر آپ کے سب کھاتے { $date } کو بنی کاپی سے بدل جائیں گے۔ یہ ضائع ہو جائیں گے: { $journal } پوسٹ شدہ اندراج، { $bikri } بکری اندراج اور { $vouchers } واؤچر۔ آپ باہر ہو جائیں گے۔ } | {$lost -> [0] پہلے آپ کے موجودہ ڈیٹا کی حفاظتی کاپی بنے گی۔ پھر آپ کا سارا حساب { $date } کو بنی کاپی سے بدل جائے گا۔ آپ باہر ہو جائیں گے۔ *[other] پہلے آپ کے موجودہ ڈیٹا کی حفاظتی کاپی بنے گی۔ پھر آپ کا سارا حساب { $date } کو بنی کاپی سے بدل جائے گا۔ یہ ضائع ہو جائیں گے: { $journal } پوسٹ شدہ اندراج، { $bikri } بکری اندراج اور { $vouchers } واؤچر۔ آپ باہر ہو جائیں گے۔ } | "all your **books** are replaced": "سب کھاتے" changed to "آپ کا سارا حساب ... بدل جائے گا" (both branches). All numbers, `<num>`, `{ -bikri }`, selector kept. |
| `backup-restored` | Restored from the copy of {$date}. Please sign in again. | کاپی {$date} سے بحال ہو گئی۔ براہ کرم دوبارہ داخل ہوں۔ | {$date} کی کاپی سے ڈیٹا بحال ہو گیا۔ براہ کرم دوبارہ داخل ہوں۔ | Subject was wrong: "کاپی ... بحال ہو گئی" says the copy was restored. English: data restored **from** the copy. Now "{ $date } کی کاپی سے ڈیٹا بحال ہو گیا۔" |
| `backup-totals-what` | Books | کھاتے | حساب | Table heading "Books" was "کھاتے", the same word as the row "Accounts". Now "حساب". |
| `backup-totals-accounts` | Accounts | کھاتوں کی تعداد | کھاتے | English "Accounts"; Urdu added "تعداد" (count) that no other row has. Now "کھاتے". |
| `backup-totals-cash` | Cash in hand | روکڑ بقایا | موجود روکڑ | English "Cash in hand"; fixed word is "موجود روکڑ". Was "روکڑ بقایا". |
| `backup-test-ok` | This copy is good. It restores correctly and its books balance. | یہ کاپی ٹھیک ہے۔ یہ درست بحال ہوتی ہے اور اس کے کھاتے برابر ہیں۔ | یہ کاپی ٹھیک ہے۔ یہ درست بحال ہوتی ہے اور اس کا حساب برابر ہے۔ | "its books balance" was "اس کے کھاتے برابر ہیں". Now "اس کا حساب برابر ہے". |
| `backup-test-totals-label` | Books in the test copy and live books | ٹیسٹ کاپی کے کھاتے اور موجودہ کھاتے | ٹیسٹ کاپی کا حساب اور موجودہ حساب | "Books in the test copy and live books": "کھاتے" changed to "حساب". |
| `backup-check-balanced` | The books balance | کھاتے برابر ہیں | حساب برابر ہے | "The books balance" was "کھاتے برابر ہیں". Now "حساب برابر ہے" (same as `settings-check-help`). |

## Search and typing

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `search-list-label` | Matching records | ملتے جلتے ناموں کی فہرست | ملنے والے ناموں کی فہرست | "Matching" was "ملتے جلتے" (similar). Now "ملنے والے". |
| `translit-pair-english-hint` | English letters only. It fills by itself from the Urdu name, and you can change it. | انگریزی حروف میں لکھیں۔ یہ اردو نام سے خود بھر جاتا ہے، اسے بدل سکتے ہیں۔ | صرف انگریزی حروف میں لکھیں۔ یہ اردو نام سے خود بھر جاتا ہے، اسے بدل سکتے ہیں۔ | Left out "only": "صرف انگریزی حروف میں لکھیں". |

## Error messages

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `err-entry-too-large` | This entry is too large. One line, or the total of an entry, can be at most Rs {$max}. | یہ اندراج بہت بڑا ہے۔ ایک سطر، یا پورے اندراج کا میزان، زیادہ سے زیادہ {$max} روپے ہو سکتا ہے۔ | یہ اندراج بہت بڑا ہے۔ ایک لائن، یا پورے اندراج کا میزان، زیادہ سے زیادہ {$max} روپے ہو سکتا ہے۔ | "line" was سطر here but لائن everywhere else (err-no-sale-lines, err-too-many-lines). Same idea, now لائن. |
| `err-share-backdated` | The share change cannot start in the past. Enter today ({$today}) or a later date. | حصہ کی نئی تاریخ پرانی نہیں ہو سکتی۔ آج ({$today}) یا اس کے بعد کی تاریخ لکھیں۔ | نیا حصہ پچھلی تاریخ سے شروع نہیں ہو سکتا۔ آج ({$today}) یا اس کے بعد کی تاریخ لکھیں۔ | Urdu said "the new date of the share cannot be old"; English says the share change cannot START in the past. Now "نیا حصہ پچھلی تاریخ سے شروع نہیں ہو سکتا" (also removes the wrong "حصہ کی" inflection). |
| `err-user-inactive` | This account cannot sign in. Use the recovery code. | اس کھاتے سے لاگ ان نہیں ہو سکتا۔ ریکوری کوڈ استعمال کریں۔ | اس صارف سے لاگ ان نہیں ہو سکتا۔ ریکوری کوڈ استعمال کریں۔ | "This account cannot sign in": کھاتہ means a ledger account in this app and confuses. Now صارف. |
| `err-version-changed` | This was changed after you opened it. Open it again. | آپ کے کھولنے کے بعد یہ اندراج بدل چکا ہے۔ اسے دوبارہ کھولیں۔ | آپ کے کھولنے کے بعد یہ بدل چکا ہے۔ اسے دوبارہ کھولیں۔ | Urdu narrowed "This" to "یہ اندراج" (an entry). English is any record. Now just "یہ". |
| `err-rule-not-allowed` | This combination is not allowed for a rule. Choose a charge and a method from the lists, and a group only when no party or Item is chosen. The default rates are changed in Settings, Charges. | اصول کی یہ ترتیب درست نہیں۔ چارج اور طریقہ فہرست میں سے چنیں، اور گروپ صرف اسی وقت چنیں جب کوئی کھاتہ یا قسم مال نہ چنا ہو۔ عام شرحیں ترتیبات کے چارجز میں بدلتی ہیں۔ | اصول کی یہ ترتیب درست نہیں۔ چارج اور طریقہ فہرست میں سے چنیں، اور گروپ صرف اسی وقت چنیں جب کوئی کھاتہ یا قسم مال نہ چنا ہو۔ عام شرحیں ترتیبات کے چارجز میں بدلی جاتی ہیں۔ | "default rates are changed in Settings, Charges": Urdu "بدلتی ہیں" means they change by themselves. Now passive "بدلی جاتی ہیں". |
| `err-cash-below-zero-not-confirmed` | After this entry cash in hand would be {$cash_after}. Confirm the entry to save it. | اس اندراج کے بعد روکڑ {$cash_after} ہو جائے گی۔ محفوظ کرنے کے لیے اندراج کی تصدیق کریں۔ | اس اندراج کے بعد موجود روکڑ {$cash_after} ہو جائے گی۔ محفوظ کرنے کے لیے اندراج کی تصدیق کریں۔ | English "cash in hand"; Urdu had only روکڑ. Now the fixed word "موجود روکڑ". |
| `err-payment-above-payable` | This is {$over} more than what is payable to the Seller ({$payable}). You can still save it. The extra shows as owed by the Seller; it is not an Advance. | یہ رقم بیوپاری کی قابل ادائیگی رقم ({$payable}) سے {$over} زیادہ ہے۔ آپ پھر بھی محفوظ کر سکتے ہیں۔ زیادہ رقم بیوپاری کے ذمے دکھائی جائے گی، یہ پیشگی نہیں ہوگی۔ | بیوپاری کو جو رقم دینی ہے ({$payable}) یہ اس سے {$over} زیادہ ہے۔ آپ پھر بھی محفوظ کر سکتے ہیں۔ زیادہ رقم بیوپاری کے ذمے دکھائی جائے گی، یہ پیشگی نہیں ہوگی۔ | "payable" was the stiff "قابل ادائیگی رقم". Now the fixed natural form "جو رقم دینی ہے" (what the shop must pay the seller). Rest unchanged ("owed by the seller" already ذمے). |
| `err-write-off-exceeds-balance` | You cannot write off more than is left. Only {$available} Nag are left. | باقی مال سے زیادہ ضائع نہیں ہو سکتا۔ صرف {$available} نگ باقی ہیں۔ | جتنا مال باقی ہے اس سے زیادہ ضائع نہیں کیا جا سکتا۔ صرف {$available} نگ باقی ہیں۔ | English "more than is left"; Urdu used the screen name "باقی مال", which reads as the stock screen. Now "جتنا مال باقی ہے اس سے زیادہ". |
| `err-purchase-has-sales` | This purchase cannot be cancelled, because {$qty_sold} Nag were sold and {$qty_written_off} Nag were written off. Cancel the sales or undo the write-off first. | یہ خریداری منسوخ نہیں ہو سکتی، کیونکہ {$qty_sold} نگ بک چکے ہیں اور {$qty_written_off} نگ ضائع ہو چکے ہیں۔ پہلے بکری منسوخ کریں یا ضائع واپس لیں۔ | یہ خریداری منسوخ نہیں ہو سکتی، کیونکہ {$qty_sold} نگ بک چکے ہیں اور {$qty_written_off} نگ ضائع ہو چکے ہیں۔ پہلے بکری منسوخ کریں یا ضائع واپس کریں۔ | "undo the write-off": ضائع واپس لیں differs from the Sale screen's "ضائع واپس کریں" (bikri). Same idea, same verb now. |
| `err-entry-charge-already-taken` | {$bikri_number_zero -> [yes] This charge was already taken on Lot { $lot_no } of Goods Arrived entry { $entry }. The amount you typed is kept. *[no] This charge was already taken on Lot { $lot_no } of Sale { $bikri_number }, Goods Arrived entry { $entry }. The amount you typed is kept. } | {$bikri_number_zero -> [yes] یہ خرچہ مال آیا اندراج { $entry } کی لاٹ { $lot_no } پر پہلے ہی لگ چکا ہے۔ آپ کی لکھی ہوئی رقم برقرار رہے گی۔ *[no] یہ خرچہ بکری { $bikri_number } (مال آیا اندراج { $entry }) کی لاٹ { $lot_no } پر پہلے ہی لگ چکا ہے۔ آپ کی لکھی ہوئی رقم برقرار رہے گی۔ } | {$bikri_number_zero -> [yes] یہ چارج مال آیا اندراج { $entry } کی لاٹ { $lot_no } پر پہلے ہی لگ چکا ہے۔ آپ کی لکھی ہوئی رقم برقرار رہے گی۔ *[no] یہ چارج بکری { $bikri_number } (مال آیا اندراج { $entry }) کی لاٹ { $lot_no } پر پہلے ہی لگ چکا ہے۔ آپ کی لکھی ہوئی رقم برقرار رہے گی۔ } | English "This charge"; Urdu said "یہ خرچہ" (khercha = Expense, a glossary word). Now "یہ چارج", as in assist working-per-lot-already-charged (both branches of the selector). |
| `err-trip-cost-has-sales` | This cost cannot be cancelled, because {$qty_sold} Nag of the purchases it was spread over were sold and {$qty_written_off} Nag were written off. Cancel the sales or undo the write-off first. | یہ خرچ منسوخ نہیں ہو سکتا، کیونکہ جن خریداریوں پر یہ بانٹا گیا ان کے {$qty_sold} نگ بک چکے ہیں اور {$qty_written_off} نگ ضائع ہو چکے ہیں۔ پہلے بکری منسوخ کریں یا ضائع واپس لیں۔ | یہ خرچ منسوخ نہیں ہو سکتا، کیونکہ جن خریداریوں پر یہ بانٹا گیا ان کے {$qty_sold} نگ بک چکے ہیں اور {$qty_written_off} نگ ضائع ہو چکے ہیں۔ پہلے بکری منسوخ کریں یا ضائع واپس کریں۔ | Same as err-purchase-has-sales. |
| `err-garden-has-stock` | This Garden still has goods that are not sold ({$qty_left} Nag). Sell them or write them off before closing it. | اس باغ میں ابھی بغیر بکا مال ہے ({$qty_left} نگ)۔ بند کرنے سے پہلے اسے بیچیں یا ضائع لکھیں۔ | اس باغ میں ابھی ایسا مال ہے جو بکا نہیں ({$qty_left} نگ)۔ بند کرنے سے پہلے اسے بیچیں یا ضائع کریں۔ | "بغیر بکا مال" is stiff and not Urdu. Now "ایسا مال ہے جو بکا نہیں"; "write them off" was "ضائع لکھیں", now "ضائع کریں" (same as bikri). |

## Attention list, party card and working

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `accountpicker-reserved` | This code is kept for system accounts. An ordinary account cannot have it. | یہ کوڈ خاص کھاتوں کے لیے ہے۔ عام کھاتے کا یہ کوڈ نہیں ہو سکتا۔ | یہ کوڈ نظام کے کھاتوں کے لیے رکھا گیا ہے۔ عام کھاتے کا یہ کوڈ نہیں ہو سکتا۔ | English "system accounts"; Urdu "خاص کھاتوں" (special accounts) is a different thing. Now "نظام کے کھاتوں" (same word as err-system-account). |
| `working-per-lot-already-charged` | This charge is made once per Lot and was made in the first Sale, so nothing is added now. | لاٹ کا یہ چارج پہلی بکری میں لگ چکا ہے، اس لیے اب کچھ نہیں لگا۔ | یہ چارج ہر لاٹ پر ایک ہی بار لگتا ہے اور پہلی بکری میں لگ چکا ہے، اس لیے اب کچھ نہیں جڑتا۔ | Urdu left out "made once per lot". Now "ہر لاٹ پر ایک ہی بار لگتا ہے"; "nothing is added" now "کچھ نہیں جڑتا" (as the sibling message). |
| `working-purchase-total-cost` | Paid now {$paid_now} + part of the Market Trip costs {$trip_cost} = {$result} | ادا شدہ {$paid_now} + مارکیٹ سفر کے خرچ کا حصہ {$trip_cost} = {$result} | ابھی ادا شدہ {$paid_now} + مارکیٹ سفر کے خرچ کا حصہ {$trip_cost} = {$result} | English "Paid now"; Urdu "ادا شدہ" lost "now". Now "ابھی ادا شدہ" (same as the purchases screen). |
| `attention-dues-old` | {$count -> [one] { $count } Buyer has an old due: { $amount } *[other] { $count } Buyers have old dues: { $amount } } | {$count} گاہک کے پرانے بقایا: {$amount} | {$count} گاہک کے ذمے پرانی رقم ہے: {$amount} | English "has an old due"; Urdu "پرانے بقایا" used the balance word for dues. Now "کے ذمے پرانی رقم ہے". |
| `attention-dues-old-aged` | {$count -> [one] { $count } Buyer has an old due: { $amount }, the bill is { $days } { $days -> [one] day *[other] days } old *[other] { $count } Buyers have old dues: { $amount }, the oldest bill is { $days } { $days -> [one] day *[other] days } old } | {$count} گاہک کے پرانے بقایا: {$amount}، سب سے پرانا بل {$days} دن پرانا ہے | {$count} گاہک کے ذمے پرانی رقم ہے: {$amount}، سب سے پرانا بل {$days} دن پرانا ہے | Same fix as attention-dues-old. |
| `attention-payables-old` | {$count -> [one] { $count } Seller has waited for payment: { $amount } *[other] { $count } Sellers have waited for payment: { $amount } } | {$count} بیوپاری کی ادائیگی باقی ہے: {$amount} | {$count} بیوپاری ادائیگی کے انتظار میں ہیں: {$amount} | English "has waited for payment" (it is old); Urdu "ادائیگی باقی ہے" lost the waiting/age. Now "ادائیگی کے انتظار میں ہیں". |
| `attention-payables-old-aged` | {$count -> [one] { $count } Seller has waited for payment: { $amount }, for { $days } { $days -> [one] day *[other] days } *[other] { $count } Sellers have waited for payment: { $amount }, the longest for { $days } { $days -> [one] day *[other] days } } | {$count} بیوپاری کی ادائیگی باقی ہے: {$amount}، سب سے پرانی {$days} دن سے | {$count} بیوپاری ادائیگی کے انتظار میں ہیں: {$amount}، سب سے زیادہ {$days} دن سے | Same; "the longest for D days" is now "سب سے زیادہ D دن سے". |
| `attention-loans-overdue` | {$count -> [one] { $count } Money Lent entry is past its return-by date: { $amount } *[other] { $count } Money Lent entries are past their return-by date: { $amount } } | {$count} ادھار رقم کی واپسی کی تاریخ گزر چکی ہے: {$amount} | {$count -> [one] ادھار رقم کے { $count } اندراج کی واپسی کی تاریخ گزر چکی ہے: { $amount } *[other] ادھار رقم کے { $count } اندراجات کی واپسی کی تاریخ گزر چکی ہے: { $amount } } | English counts "entries"; Urdu counted the money lent itself ("N ادھار رقم"). Now "ادھار رقم کے N اندراج / اندراجات" with an [one]/[other] selector like the English. |
| `attention-loans-overdue-aged` | {$count -> [one] { $count } Money Lent entry is { $days } { $days -> [one] day *[other] days } past its return-by date: { $amount } *[other] { $count } Money Lent entries are past their return-by date, up to { $days } { $days -> [one] day *[other] days }: { $amount } } | {$count} ادھار رقم کی واپسی کی تاریخ گزر چکی ہے، زیادہ سے زیادہ {$days} دن: {$amount} | {$count -> [one] ادھار رقم کے { $count } اندراج کی واپسی کی تاریخ کو { $days } دن گزر چکے ہیں: { $amount } *[other] ادھار رقم کے { $count } اندراجات کی واپسی کی تاریخ گزر چکی ہے، زیادہ سے زیادہ { $days } دن: { $amount } } | Same; singular now says "تاریخ کو D دن گزر چکے ہیں" (English "D days past"). |
| `attention-takes-overdue` | {$count -> [one] { $count } amount taken by a partner is past its return-by date: { $amount } *[other] { $count } amounts taken by partners are past their return-by date: { $amount } } | شراکت داروں کی لی ہوئی {$count} رقموں کی واپسی کی تاریخ گزر چکی ہے: {$amount} | {$count -> [one] شراکت دار کی لی ہوئی { $count } رقم کی واپسی کی تاریخ گزر چکی ہے: { $amount } *[other] شراکت داروں کی لی ہوئی { $count } رقموں کی واپسی کی تاریخ گزر چکی ہے: { $amount } } | Urdu always said "رقموں" ("1 رقموں" is wrong). Added [one]/[other] as in the English. |
| `attention-takes-overdue-aged` | {$count -> [one] { $count } amount taken by a partner is { $days } { $days -> [one] day *[other] days } past its return-by date: { $amount } *[other] { $count } amounts taken by partners are past their return-by date, up to { $days } { $days -> [one] day *[other] days }: { $amount } } | شراکت داروں کی لی ہوئی {$count} رقموں کی واپسی کی تاریخ گزر چکی ہے، زیادہ سے زیادہ {$days} دن: {$amount} | {$count -> [one] شراکت دار کی لی ہوئی { $count } رقم کی واپسی کی تاریخ کو { $days } دن گزر چکے ہیں: { $amount } *[other] شراکت داروں کی لی ہوئی { $count } رقموں کی واپسی کی تاریخ گزر چکی ہے، زیادہ سے زیادہ { $days } دن: { $amount } } | Same as attention-takes-overdue; singular says "تاریخ کو D دن گزر چکے ہیں". |

## Sign-in and first-run wizard

| Message ID | English | Old Urdu | New Urdu | Why |
|---|---|---|---|---|
| `auth-wizard-partners-help` | Add each partner's name and share. You can also do this later, but until the shares total exactly 100%, a Sale cannot be saved. | ہر شراکت دار کا نام اور حصہ لکھیں۔ یہ کام بعد میں بھی ہو سکتا ہے، مگر جب تک حصہ پورے 100 فیصد نہ ہوں، بکری محفوظ نہیں ہو گی۔ | ہر شراکت دار کا نام اور حصہ لکھیں۔ یہ کام بعد میں بھی ہو سکتا ہے، مگر جب تک کل حصہ ٹھیک 100 فیصد نہ ہو، بکری محفوظ نہیں ہو گی۔ | "حصہ ... پورے 100 فیصد نہ ہوں": plural agreement was wrong with the fixed term, and "exactly" was weaker. Now "جب تک کل حصہ ٹھیک 100 فیصد نہ ہو". |

## English texts that look wrong or vague (the English was NOT changed)

- `report-r20-help`: says "Each buyer's balance" while the title (and Umar's point) is "Dues". Suggest "What each buyer owes, with the age of his open bills." The Urdu still says "بقایا" because that is what the English says. If the English changes, change the Urdu to "ہر گاہک کے ذمے رقم".
- `report-r34-help`: "what is still owed" does not say by whom. It means buyers' dues (and the row list also has unsold lots). Suggest "buyers' dues". Urdu left as "جو ابھی لینا ہے" (same vagueness).
- `report-r35-help`: "what each party owed at the start" is one-way. For a seller it is the shop that owes him. The column is "Previous Balance". Suggest "the balance of each party at the start". Urdu now follows the English ("ہر پارٹی کے ذمے ... تھی"); if the English becomes "balance", the Urdu goes back to "شروع کا بقایا".
- `day-close-cash`: `{-rokar} balance` shows as "Cash Book balance" in English (glossary -rokar = Cash Book). The PRD tile is "Cash in hand". Urdu "روکڑ بقایا" is fine as is.
- `report-col-installments`: English is `{-qist}s` (plural made by adding "s" to a term). Breaks if the term changes. Better a plain "Installments".
- `audit-filter-entity` and `audit-col-entity`: "Area" is vague (it means the part of the app: bikri, voucher, account ...). Maybe "Record type".
- `print-doc-receipt-slip` ("Receipt slip") vs `print-receipt-slip-title` ("{-wasooli} slip"): two English wordings for the receipt, and the Urdu differs too ("رقم کی رسید" vs "وصولی کی رسید"). `print-doc-receipt-slip` is the generic slip of any voucher (receipt, payment, advance), so "Voucher slip" would be clearer.
- `bikri-summary-buyer-total`: English says "Buyer bills" but the row is an amount; other places call the same figure "Buyer total" (`bikri-col-buyer-total`, `bikri-list-col-buyer-total`).
- `bikri-posting-reversed` ("Taken back"), `bikri-undo-done` ("taken back") and `bikri-posting-reversal` ("Reversal"): three English words for reversal.
- `bikri-edit-reason-text`: "is not of today" is awkward English.
- `cashbook-saved` (capital_in, "put in") vs `cashbook-kind-capital-in` ("Capital invested") and `cashbook-type-capital-in` ("put in"): two English words for one thing.
- `cashbook-help`, `cashbook-opening`, `cashbook-closing`, `cashbook-col-running`: English mixes "cash in hand", "Cash at the start" and "Cash after". The fixed word for the dashboard is "Cash in hand".
- `cashbook-col-days-past`: "Days late" does not say late against the return-by date.
- `cashbook-col-in` / `cashbook-col-out`: "In" / "Out" are bare; "Cash in" / "Cash out" would be clearer.
- `cashbook-type-money-lent` ("Money lent"): not the glossary form "Money Lent".
- `vouchers-bills-open`: "Total open" does not say open what (bills).
- `vouchers-return-limit`: "Most he can return" is a sentence fragment as a label.
- `fruit-parts-col-amount` ("Part"): glossary word is "Share" (حصہ); fine in Urdu, but English uses two words.
- trips-new-firm-shares: English uses the term `{-hissa}` (reads "the firm's Share"). The plural "shares" is meant, as in gardens-new-from-defaults.
- partners-ventures-col-balance: "Balance or result" does not say that the balance is the garden's running balance (PRD FR-12.37). Suggest "Running balance or result".
- gardens-close-credits-title ("Each partner's money back") and gardens-partner-back ("Money back"): the same English for two different times. In the close preview it is what he WILL get back, in the table what he HAS got back. The Urdu is different on purpose (ملنے والی / ملی).
- gardens-partner-net-open ("Net so far"): English does not say what is netted (money back less money paid). Not a meaning problem for the Urdu, but a user may confuse it with the partner خالص of C20.
- partners-statement-title ("Partner statement"): the glossary English for فرد حساب is Statement, but the PRD names R31 شراکت دار کھاتہ (ledger). The Urdu follows the PRD name.
- masters-marka-help and masters-marka-text-hint say "crate", the Urdu says "پیٹی" (glossary Box). Equal for a munshi; left alone.
- err-credit-limit-exceeded: says "the buyer's balance would be X" but the value is `would_owe`; "what the buyer would owe" is the clear meaning. Urdu kept as بقایا to match the English word.
- err-system-account (errors) and accounts-system-stays-active (accounts) are the same sentence twice; and system accounts are called "system accounts" (accountpicker-reserved, err-system-account) and "the app's own accounts" (err-code-reserved). One name would be better.
- attention-dues-old and attention-dues-old-aged: "has an old due" does not say how old (the rule is over 30 days). Both go with the attention-list limits in PRD A40.
- auth-wizard-owner-title and err-user-inactive use "account" for the sign-in user. "account" also means a ledger account in this app.
- assist-warning-owes-firm: "This party owes the firm" gives no amount; the card shows the balance, so this is fine, but it is vague alone.
- `accounts-field-tag-hint`: says "English capital letters" only. PRD 10.2 and Code Standards 4.2 also allow dots (K, BA, GM, D.P). The Urdu mirrors the English, so both need "and dots" if Umar agrees.
- `settings-language-default-help`: says the default language is used for the sign-in screen and the daily backup pack. PRD FR-7.30 and Architecture 15.3 say the sign-in screen uses the last language used on this computer (`login_language`); only the backup pack uses `default_language` (PRD Section 21). The Urdu follows the English. One of them is wrong.
- `settings-grouping-help`: "the digits after the first three go in pairs" is unclear (first three counted from the right: 13,16,040). The Urdu copies the same words. Suggest "the last three digits stand together, the rest go in pairs".
- `backup-set-closing-time` ("Daily copy time") and `settings-day-closing-time` ("Closing time") are the same setting under two names; the hint even says "the same time as on the backup screen". Use one label in both screens (Urdu: "روزانہ کاپی کا وقت" vs "بندش کا وقت").
- `backup-col-created` "Made on" suggests a date, but the column shows date and time (copies are made every 15 minutes). The Urdu says "بننے کا وقت".
- `settings-check-backup-errors` says "backup problem(s)", the Urdu word is "خرابی". Fine, but the backup screen uses "failed its check" for the same thing. Not changed.
- `accounts-col-other-name` reads "Urdu name" in English and "انگریزی نام" in Urdu on purpose (the other script). Not an error; listed so the reviewer does not "fix" it.

## Things the Urdu reviewer should look at

- **PRD quotes.** `vouchers-advance-hint` now reads اس بیوپاری کو دینے کے لیے کچھ باقی نہیں (Nothing is payable to this seller). PRD FR-7.68 and E57 still quote the old wording اس بیوپاری کا کوئی بقایا نہیں. The report names R20 (وصولی / اگراہی رپورٹ) and R24 (ادائیگی باقی) in PRD 19.2 also differ from the screen titles now (گاہکوں کے ذمے, بیوپاریوں کو ادائیگی باقی). The PRD needs the same change when Umar approves.
- **Cash in hand.** The tile and the backup totals say موجود روکڑ. The save-below-zero box says روکڑ, as PRD FR-6.7 has it.
- `report-preview` ("پرنٹ پر نظر") and `report-print-title` ("رپورٹ کی پرنٹ پر نظر"): odd phrase for "Print preview". print.ftl uses "پیش منظر" for preview. NOT changed because the e2e scripts click this button by its Urdu name (`frontend/tests/e2e/alignment/run.mjs` line 127, `shots-plan30.playwright.txt` and `shots-plan31.playwright.txt` line 24). If changed to "پرنٹ کا پیش منظر", those three files need the new name too.
- `report-r3-title` / `report-r4-title`: English "Cash Sales" / "Credit Sales", Urdu "نقدی" / "ادھاری" (the client's old menu words, PRD R3 and R4). Kept. A reviewer may want "نقدی بکری" / "ادھاری بکری".
- `report-r19-title`: English "Opening Trial Balance", Urdu "اوپن ٹرائل" (client's old word, PRD R19). Kept.
- `report-r31-title`: English "Partner Statement", Urdu "شراکت دار کھاتہ" (PRD R31 name; Statement would be فرد حساب). Kept.
- `report-r20-help`, `report-r35-help`: see English issues; they depend on whether the English is fixed.
- `audit-filter-entity`, `audit-col-entity`: "Area" -> "حصہ", but حصہ is also the glossary word for Share. "شعبہ" may be clearer. Not changed.
- `audit-action-day-close` ("دن بند کیا") and `audit-action-month-close` ("مہینہ بند کیا") have no "گیا", unlike most other audit actions. Not changed.
- Filter word: bikri.ftl uses "چھانٹیاں" for "filters" (bikri-list-clear, bikri-list-no-match-help) while reports, analytics, audit and now pukhta use "فلٹر". Lead may want to make bikri.ftl match (not my file).
- `month-lines-label`: "سطریں" kept. cashbook.ftl and partners.ftl also use "سطریں" for journal lines; "لائن" is for sale lines.
- `cashbook-general-*`: journal lines are "سطر" (also used in month.ftl, partners.ftl); sale lines in bikri are "لائن". Kept as is. Decide if one word is wanted.
- `cashbook-general-totals-hint`: "میزان یہاں نظر آئے گی" treats میزان as feminine, and `cashbook-general-total-dr` / `-cr` say "کی میزان". Check the gender of میزان and add `.gender` to the glossary term.
- `vouchers-col-adjusted` and `vouchers-col-returns` use "منہا"; reports.ftl (`report-col-adjusted`, `report-r26-help`) uses "ایڈجسٹ" for the same advance figure. The lead should pick one.
- `vouchers-advance-hint`: PRD FR-7.68 quotes the old words "اس بیوپاری کا کوئی بقایا نہیں". I changed it to match the fixed word table. The PRD text needs the same change when Umar approves.
- `cashbook-field-paid-to` ("ادا کیا" for "Paid to") and `cashbook-field-person` ("نام" for "Person") follow the PRD labels; the Urdu is narrower than the English.
- `cashbook-empty`, `cashbook-mode-cash`, `cashbook-salary-note` still say "نقد" for physical cash moving; only cash-balance wording was moved to روکڑ.
- `bikri-list-qty`: the Urdu order is swapped on purpose (a comment in the file explains the left-to-right number cell); not an error.
- `vouchers-return-limit` ("زیادہ سے زیادہ واپسی") could read as "highest return"; consider "زیادہ سے زیادہ جو واپس ہو سکتا ہے".
- Spelling of "ہوگی" / "ہو گی" differs between files (vouchers, bikri use "ہوگی"); the style guide only fixes "ہو گئی".
- purchases-lots-title: the plural is made as `{ -bikri }وں` (term + suffix) so it reads "بکریوں". If the Urdu reviewer prefers, write the plural by hand.
- Stock is written "ذخیرہ" in gardens-arrival-posts-nothing, gardens-arrival-cancel-text, gardens-arrival-write-off-text, gardens-arrival-write-off-undo-text, purchases-write-off-text, purchases-write-off-undo-text, purchases-cancel-text. The glossary word for stock is سٹاک. Same scope, so not changed.
- gardens-close-balance and gardens-close-credits-title keep "ملنے والی رقم" (future, it is a preview before closing). The table headings use "ملی" (past).
- rules-level-line-override: "لائن پر لکھی گئی شرح" adds the noun شرح (a rate) to the English "Typed on the line". A rate is what is typed there (PRD FR-11.10).
- gardens-partner-net-open, gardens-partner-net-closed, partners-metric-net: خالص is the glossary Net. For a garden it means money back less money paid; for the partner overview it is C20. Please confirm one word for both is right.
- Other files (not mine) have the same wrong word "انتخابات" for "choices": fruit.ftl fruit-list-no-match ("ان انتخابات سے کوئی اندراج نہیں ملا"). The glossary term {-hissa} is singular, so any plural "shares" in other files ("ان کے {-hissa}", "{-hissa} محفوظ ہو گئے") has the same number problem.
- masters-col-other-name: English "Urdu name", Urdu "انگریزی نام". Correct on purpose, the column shows the name in the other language (the English app shows the Urdu name, the Urdu app the English name). Left alone.
- masters-menu-help, masters-city-empty and similar "... is empty" messages: Urdu adds "ابھی". Harmless, left alone.
- "Sign in": errors.ftl and audit.ftl say لاگ ان (err-not-logged-in, err-already-signed-in, err-already-set-up, err-user-inactive); auth.ftl, backup.ftl and settings.ftl say داخل ہوں. The PRD uses both (button داخل ہوں, map لاگ ان). Left as is; one word should be chosen app-wide.
- auth-wizard-owner-title "مالک کا کھاتہ بنائیں": کھاتہ may be read as a ledger. Kept (a natural phrase), but the reviewer may prefer "مالک کا صارف بنائیں" or اکاؤنٹ.
- Oblique case of glossary terms: some messages still read "{ -hissa } کی فہرست" (err-partner-not-listed), "{ -shirakat-dar } کے { -zati-kharcha } کھاتے". A Fluent term cannot take the oblique form (حصے, لاٹ is fine). Not rewritten here.
- System account Urdu: now "نظام کا/کے کھاتے" in err-system-account (unchanged) and accountpicker-reserved (changed); err-code-reserved keeps "ایپ کے اپنے کھاتوں" (matches its English). The lead may want one word.
- attention-loans-overdue and attention-takes-overdue now have [one]/[other] selectors in Urdu (the English already had them). Please check the singular wording with the Urdu reviewer.
- err-code-taken "یہ کوڈ کسی اور کے پاس ہے" is colloquial for "already used by another one"; kept.
- err-payment-above-payable "ہوگی" could be spelled "ہو گی" like the rest of the file (style guide); left as the file had it.
- backup (all): I made the Urdu for "books" = "حساب" (settings and dashboard already do this); `accounts-system-stays-active` and `err-system-account` use "حساب کتاب" for the same English "books". Reviewer to choose one for both.
- `backup-col-size`: "حجم" is correct but formal; "سائز" is what a munshi says (settings uses "سائز" for text size). Not changed.
- `accounts-system-stays-active`: "نظام کا کھاتہ" ("system account"); "سسٹم کھاتہ" may be more usual. Same text is in `errors.ftl` (another reviewer's file).
- `openings-locked-help`: "خانے بند ہیں" for "cells are locked" and "نہیں لگ سکتا" for "cannot have". Understandable, but "لاک" and "نہیں لکھا جا سکتا" may be clearer.
- `settings-firm-contact-title` and related: "رابطہ شخص" is stiff Urdu; "رابطے کا شخص" or "رابطہ نمبر والا" may sound better.
- `backup-state-pack-pending`: "ابھی نہیں بنتا" ("is not made yet"); "ابھی نہیں بنا" is the exact tense. Left, because both are understood.
- `backup-restore-confirm-title`: "کیا یہ بیک اپ بحال کریں؟" has "کیا" with a command form; other confirm titles ("غیر فعال کریں؟") do not. Left.
- Dead amount: when the button is added to the account form, its wording must keep "غیر فعال" (inactive) and "فعال" (active) exactly as now; the old Urdu "ڈوبی رقم" is already in `glossary.ftl`.

- `vouchers-collection-total`: the English "Total dues" on the collection list is the total of that one list, but "Total dues" is also the dashboard tile (کل اگراہی, the whole shop). The Urdu says کل ذمے here so the two numbers are not mixed up. Suggest the English says "Total of these dues".