## ادھاری تفصیل on screen: how much the shop has to take from its buyers (R5, PRD R5 v1.43, plan 80 Q-R5-2).
## The column words and the words of the totals are the print's own (print.ftl), so the screen and the paper say the same.

# review: pending
adhari-help = اس دن دکان کو اپنے { -gahakon } سے کتنا لینا ہے: ہر وہ { -gahak } جس نے خریداری کی، ایک قطار میں: اس کا مال، آج کا بل، آج کا ادھار، { -sabiqa-raqam } اور کل ادھار۔ جدول کے نیچے وہ { -gahak } جن کے ذمے رقم تھی مگر آج خریداری نہیں کی، اور سب { -gahakon } کا کل ادھار۔ میزان سب { -gahakon } کا ہے، صرف سامنے کی قطاروں کا نہیں۔

# review: pending
adhari-label = دن کے گاہک

# review: pending
adhari-failed = شیٹ نہیں پڑھی جا سکی

# review: pending
adhari-empty = اس دن کسی { -gahak } نے خریداری نہیں کی

# review: pending
adhari-empty-help = دوسرا دن چنیں، یا پہلے { -bikri } لکھیں۔

# review: pending
adhari-goto = دن پر جائیں

# review: pending
adhari-goto-hint = دن لکھیں، مثلاً 10-10-2026۔ مختصر شکل بھی چلتی ہے: 10 اس مہینے کی 10 تاریخ ہے۔

# review: pending
adhari-goto-placeholder = دن-مہینہ-سال

# review: pending
adhari-today = آج

# The buyers who owed at the start of the day and bought nothing today, as one total.
# review: pending
adhari-others-line = { print-adhari-sheet-others } (<count>{ $count }</count>): { -sabiqa-raqam } <prev></prev>، { print-adhari-sheet-col-total } <due></due>

# Everything every buyer owes the shop.
# review: pending
adhari-all-line = { print-adhari-sheet-all }: <due></due>

# review: pending
adhari-dead-line = { -doobi-raqam } (الگ شمار، اوپر کے میزان میں نہیں): { -sabiqa-raqam } <prev></prev>، { -taza-banam } <bill></bill>، { print-adhari-sheet-col-total } <due></due>

# The buyers whose total credit ends in { -jama } (the shop owes them): listed apart so the total above stays what the buyers owe.
# review: pending
adhari-credit-line = { -jama } والے { -gahak } (الگ شمار، اوپر کے میزان میں نہیں): { -sabiqa-raqam } <prev></prev>، { -taza-banam } <bill></bill>، { print-adhari-sheet-col-total } <due></due>
