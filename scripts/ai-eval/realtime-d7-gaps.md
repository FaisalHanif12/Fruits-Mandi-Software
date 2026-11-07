# D7 gap list: which wordings still go to the model, and which instant plan would answer them

Written by `robustness` on 08-11-2026 from the offline run of `realtime-d7.tsv` (455 questions, no model, no key) and the
dump test `backend/crates/assistant/tests/robustness_gap.rs` (what `understand` and `plan_reads` made of each question).
This file is about the MAIN bank only. Reproduce:

    node scripts/ai-eval/run.mjs --bank scripts/ai-eval/realtime-d7.tsv --offline
    MANDI_GAP_BANK=$PWD/scripts/ai-eval/realtime-d7.tsv MANDI_GAP_OUT=/some/gap.tsv \
      cargo test -j 4 -p mandi-assistant --test robustness_gap -- --nocapture      # from backend/, about 4 minutes

Tree of the run: after C1 (partner overview) and F3a in progress. Right with no model call: **148 of 455 (32.5%)**, target 60%. Needs the model: 307.
Needs the model with one call (the app already read the books, only the instant line is missing): 114. With two calls (the model chooses the tools): 193.
By language, right with no call: Roman Urdu 33.3%, Urdu letters 29.7%, English 34.4%. The gap is the shape of the question, not the language.

## Ranked causes (by wordings that would move to "right with no call")

A count is an upper bound: a wording moves only when every figure it asks is given and no word of it is unknown.

| # | Cause (the instant plan that is missing) | Wordings | 1 call now | 2 calls now |
|---|---|---|---|---|
| 1 | J. Trip, garden, fruit taken, pukhta, khata, money lent, advance (one record, its own report row) | 41 | 2 | 39 |
| 2 | C. Party or item or group, more than one figure from one row of view A/B/C (qty, amount, commission, rate, lines, count) | 40 | 34 | 6 |
| 3 | G. Dues with an age bucket, a collector, a city, or a count of buyers (R20, R21, R22) | 28 | 9 | 19 |
| 4 | B. One party on one day: items, qty, rate, amount, cash on the spot, credit, balance before/after | 27 | 0 | 27 |
| 5 | D. Rate questions: last rate with "laga" as a verb, highest and lowest, rate of a party | 25 | 2 | 23 |
| 6 | H. Partner figures: share, capital, taken and returned, gardens/trips/fruit, ranking | 23 | 12 | 11 |
| 7 | K. Counts and states: accounts, staff, heads, packings, day book entries, trial balance, closed days | 22 | 8 | 14 |
| 8 | M. A document number: sale number, voucher number, truck, audit | 19 | 0 | 19 |
| 9 | F. Cash book, receipts and payments totals (R12, R23, R25), with counts | 18 | 12 | 6 |
| 10 | E. Quality (awwal / doyam / soyam) | 15 | 15 | 0 |
| 11 | I. Expense heads, one or two together, and a head ranking | 9 | 4 | 5 |
| 12 | L. Stock: quantity and lots, oldest waiting, goods received | 8 | 2 | 6 |
| 13 | O. Comparisons, best month, two periods, cash against credit | 8 | 5 | 3 |
| 14 | Q. Rules and the "kal" total word | 6 | 2 | 4 |
| 15 | N. A packing word (peti, boriyan, box, bag) | 6 | 6 | 0 |
| 16 | R. Wording of one figure: Urdu or Roman forms the app does not know (no new plan, a wording fix) | 5 | 1 | 4 |
| 17 | P. Zero or thin-day questions with no figure word ("kitne ke liye", "how much potato did X take") | 4 | 0 | 4 |
| 18 | A. Ambiguous name: the app should ask "which one" itself (2 to 5 matches) | 3 | 0 | 3 |

Cheapest first (the read exists, the instant line does not): causes C, E, N, F, H and K have most of their wordings at one call.

## Wording causes that cross the families (a word or form the app does not know)

A word the app does not know sets `unexplained`, which switches off every instant line that is a report total
(`keep_lines`), so a plan that exists still sends the question to the model. 130 of the 307 questions have such a word,
mostly a different word each time (a long tail), so a longer word list alone cannot reach the target. These are the words
that come back, with the questions they hold:

| Cause | Questions | What to do |
|---|---|---|
| "laga" / لگا / "got" as the VERB after a rate ("kya laga", "کیا لگا", "what rate did X get") is read as the Laga charge: Roman reads two figures [Rate, Laga], Urdu leaves لگا unknown, English "got" is unknown. Also "lagaya / لگایا" (invested) | D137 D200 D310 (Roman, 2 figures), D138 D201 D204 D311 (Urdu), D205 (English "got"), D052 D053 D330 (lagaya, capital), D033 | A verb reading after "rate / ریٹ / bhao" and after "sarmaya"; the English group of these three questions is already right |
| A misspelt name with ONE close match makes `sure` false, so the model is asked ("nadeem trader", "tarik mehmod") | D316, D320 (also D009, D016, D030) | Answer with the matched name shown, when one account is close and no other is |
| Expense head words in Roman or English that are not the head's own name ("chai", "chae", "kiraye", "rent", "shop rent") | D265 D332 D420 D421 D069 D071 | Roman and English aliases for the heads (the head list is small) |
| Urdu "کل" meaning TOTAL is read as yesterday, so the question names two periods and is not sure | D334 ("کل کتنی واپس آنی ہے"), D436 ("اب تک پھل کی کل قیمت") | "کل" before a figure word, or beside "اب تک", is total |
| `happened` fires on "due to pay" and on an Urdu yes-or-no with an amount | D371 ("how much is Zahid Mehmood due to pay"), D269 ("کیا فیصل کا بقایا 5000 ہے") | "due to pay / ke zimme" is the balance, not an event |
| Urdu forms: وقت ("اس وقت"), آنی, تعداد (count), صافی (net), گزر (passed), چالو (open trip) | D095 D079, D334, D010 D131 D152 D031 D149 D227, D023 D038 D193, D077 D428 D076, D172 D435 D171 D434 | Add the forms |
| Words for a part of a figure: "debit / credit", "lot(s) / لاٹ", "old / older / purane", "farq / فرق / difference", "mehnga / sasta", "times / baar", "spot / موقع" | D107 D113 D441, D122-D129, D013 D089 D093 D300 D443, D108 D109 D142 D143 D307 D309 D348 D349, D035 D206 D207 D313, D147 D164 D194, D001 D003 D273 D274 D275 | Add as measure or filter words where a plan can use them |
| English misspelling inside a normal question ("Partner As totel") makes it a how-to | D329 | Spelling distance for "total / kul" |
| Two to five accounts match a name ("Akram", "Basheer"): the app could ask "کون سا؟" itself from `find` | D001 D002 D003 D030 D235 D236 D237 | Compose the which-one answer with no model call |

## J. Trip, garden, fruit taken, pukhta, khata, money lent, advance (one record, its own report row)

Plan: R33 / R36 / R37 / R9 / R13 (khata) / R32 / R26 rows as labelled instant lines (several figures per record).

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D028 | urdu | 2 | [Advance] | طاہر کو کل کتنی پیشگی دی گئی، کتنی کٹ چکی اور کتنی ابھی کھلی ہے | flags happened,unexplained,how_much_twice; doubts UnknownWords; unknown word: کٹ; no read planned |
| D029 | en* | 2 | [Advance] | how much advance was given to Tahir in total, how much is adjusted and how much is still open | flags sum_asked,happened,unexplained,how_much_twice; doubts UnknownWords; no read planned |
| D074 | roman | 2 | [MoneyLent, Cash] | aslam ko kitna udhaar diya tha, kitna cash wapas aya, kitna tankhwah se kata aur kitna abhi baqi hai | flags capital_flow,happened,how_much_twice,measure_other; no read planned |
| D075 | en | 2 | [Cash] | how much was lent to Aslam, how much came back in cash, how much was cut from his salary and how much is still open | flags capital_flow,unexplained,how_much_twice; doubts UnknownWords; no read planned |
| D076 | roman* | 2 | [Receipts] | aslam ne paise wapas karne ki tarikh kya di thi aur kitne din guzar gaye | flags capital_flow,unexplained; doubts UnknownWords; unknown word: guzar; no read planned |
| D077 | urdu | 2 | [Receipts] | اسلم نے رقم واپس کرنے کی کون سی تاریخ دی تھی اور کتنے دن گزر چکے | flags capital_flow,unexplained; doubts UnknownWords; unknown word: گزر; no read planned |
| D078 | roman | 2 | [Receipts] | khandani khata me abhi kitna paisa para hai, kitna diya aur kitna wapas aya | flags capital_flow,unexplained,how_much_twice; unknown word: para; no read planned |
| D079 | urdu | 2 | [Receipts] | خاندانی کھاتے میں اس وقت کتنی رقم ہے، کتنی دی اور کتنی واپس آئی | flags capital_flow,unexplained,how_much_twice; doubts UnknownWords; unknown word: وقت; no read planned |
| D080 | en | 2 | [] | how much is in the family khata now, how much was paid out and how much came back | not sure; flags capital_flow,unexplained,how_much_twice; doubts UnknownWords, NoFigure; no read planned |
| D171 | roman | 2 | [Profit] | chalu safar me kitna maal kharida, kitna bika, kitna bacha aur ab tak nafa nuqsan kitna hai | flags scoped,unexplained,how_much_twice; doubts UnknownWords; unknown word: chalu; no read planned |
| D172 | urdu | 2 | [Profit] | چالو سفر میں کتنا مال خریدا، کتنا بکا، کتنا بچا اور اب تک نفع نقصان کتنا ہے | flags scoped,unexplained,how_much_twice; doubts UnknownWords; unknown word: چالو; no read planned |
| D173 | en | 2 | [Profit] | in the open trip how much was bought, sold and left, and what is the profit or loss so far | flags scoped,list; no read planned |
| D174 | roman | 2 | [Balance] | shahpur baagh ka thekah kitna tha, kitni qiston me di aur kitna baqi hai | flags scoped,happened,unexplained,how_much_twice; doubts UnknownWords; unknown word: ka+qiston thekah+qiston; no read planned |
| D175 | urdu | 2 | [Balance] | شاہ پور باغ کا ٹھیکہ کتنا تھا، کتنی قسطیں دیں اور کتنا باقی ہے | flags scoped,unexplained,how_much_twice; doubts UnknownWords; unknown word: قسطیں+دیں; no read planned |
| D176 | en | 2 | [] | what is the rent of Shahpur garden, how much was paid in installments and how much is due | not sure; flags scoped,unexplained,how_much_twice; doubts UnknownWords, NoFigure; no read planned |
| D177 | roman | 2 | [Rate] | ab tak phal kitna liya gaya aur uski qeemat kitni bani | not sure; flags item_group,how_much_twice,value_asked; doubts NoParty; no read planned |
| D178 | urdu | 2 | [Rate] | اب تک پھل کتنا لیا گیا اور اس کی قیمت کتنی بنی | not sure; flags item_group,how_much_twice,value_asked; doubts NoParty; no read planned |
| D179 | en | 2 | [] | how much fruit has been taken so far and what is its value | not sure; flags item_group; doubts NoFigure; no read planned |
| D180 | roman | 2 | [FruitTaken, Rate] | alif ne ghar ke liye kitna phal liya aur uski kul qeemat kitni | not sure; flags sum_asked,how_much_twice; doubts TwoParts, NoParty; no read planned |
| D181 | urdu | 2 | [FruitTaken, Rate] | الف نے گھر کے لیے کتنا پھل لیا اور اس کی کل قیمت کتنی | not sure; flags how_much_twice; doubts TwoParts, NoParty; no read planned |
| D182 | en | 1 | [FruitTaken] | how much fruit did Partner A take for home and what is its total value | flags sum_asked,unexplained; unknown word: home; planned find,report:R37 |
| D186 | urdu | 2 | [Sale] | پختہ کھاتہ میں اب تک کتنی بکری ہوئی اور کتنا باقی ہے | flags scoped,how_much_twice; no read planned |
| D187 | en | 2 | [Sale] | how much has been sold through pukhta khata so far and how much is still due | flags scoped,unexplained,how_much_twice; doubts UnknownWords; no read planned |
| D290 | urdu | 2 | [Ledger] | خاندانی کھاتے میں کتنا ہے | no read planned |
| D291 | en | 2 | [] | how much is in the family khata | not sure; doubts NoFigure; no read planned |
| D334 | urdu | 2 | [MoneyLent] | ادھار رقم میں کل کتنی واپس آنی ہے | flags capital_flow,happened,unexplained; doubts UnknownWords; unknown word: آنی; no read planned |
| D344 | roman* | 2 | [Ledger] | shahpur bag ka abhi hisab kitna he | flags scoped; no read planned |
| D345 | urdu | 2 | [Ledger] | شاہ پور باغ کا اب تک کا حساب کتنا ہے | flags scoped; no read planned |
| D346 | roman | 2 | [FruitTaken, Rate] | alif ke liye phal ki kul qeemat kitni bani | not sure; flags sum_asked,value_asked; doubts NoParty; no read planned |
| D347 | en* | 1 | [FruitTaken] | what is the valew of the frut Partner A took | flags unexplained,value_asked; unknown word: A took; planned find,report:R37 |
| D426 | roman | 2 | [Advance] | bayopariyon ko ab tak kul kitni peshgi di | flags sum_asked,happened,role_word; no read planned |
| D427 | en | 2 | [Advance] | how much advance has been given to sellers in all | flags sum_asked,happened,unexplained,role_word; unknown word: given; no read planned |
| D428 | urdu | 2 | [MoneyLent] | ادھار رقم میں سے کتنی کی تاریخ گزر چکی | flags unexplained; doubts UnknownWords; unknown word: گزر; no read planned |
| D429 | en | 2 | [MoneyLent] | how much of the money lent is past its return date | flags unexplained; doubts UnknownWords; unknown word: past+return; no read planned |
| D430 | roman | 2 | [Balance] | shahpur baagh ka thekah abhi kitna baqi hai | flags scoped,unexplained; doubts UnknownWords; unknown word: ka thekah; no read planned |
| D431 | en | 2 | [] | how much rent is still due for Shahpur garden | not sure; flags scoped,unexplained; doubts UnknownWords, NoFigure; no read planned |
| D432 | urdu | 2 | [] | شاہ پور باغ کی کل کتنی قسطیں دی گئیں | not sure; flags scoped,unexplained; doubts UnknownWords, NoFigure; unknown word: قسطیں+گئیں; no read planned |
| D433 | en | 2 | [] | how much has been paid in installments for Shahpur garden | not sure; flags scoped,unexplained; doubts UnknownWords, NoFigure; unknown word: paid+installments; no read planned |
| D434 | roman | 2 | [Profit] | chalu safar ka ab tak nafa nuqsan kitna hai | flags scoped,unexplained,how_much_twice; doubts UnknownWords; unknown word: chalu safar; no read planned |
| D435 | urdu | 2 | [Profit] | چالو سفر کا اب تک نفع نقصان کتنا ہے | flags scoped,unexplained; doubts UnknownWords; unknown word: چالو سفر; no read planned |
| D436 | urdu | 2 | [Rate] | اب تک پھل کی کل قیمت کتنی بنی | not sure; flags item_group,value_asked; doubts SeveralPeriods, NoParty; no read planned |

## C. Party or item or group, more than one figure from one row of view A/B/C (qty, amount, commission, rate, lines, count)

Plan: Instant lines for several labelled columns of one view row (view A item/group/party, B seller, C buyer); needs a "number of lines" and "number of buyers" column.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D010 | urdu | 1 | [Balance] | فیصل نے اس سال کیلا کتنی تعداد میں اور کتنے روپے کا خریدا | flags unexplained,how_much_twice; doubts UnknownWords; unknown word: تعداد; no read planned |
| D011 | en* | 1 | [Purchases] | how many banannas did Faisal buy this year and for how much | flags asks_count,how_much_twice; planned period,find,find,report:R2 |
| D014 | roman | 1 | [Rate] | pichle mahine faisal ne rahim ka kitna maal liya, kis rate par aur kitne ka | not sure; flags how_much_twice; doubts NoParty; no read planned |
| D015 | urdu | 1 | [Rate] | پچھلے مہینے فیصل نے رحیم کا کتنا مال لیا، کس ریٹ پر اور کتنے کا | not sure; flags how_much_twice; doubts NoParty; no read planned |
| D016 | en* | 1 | [Rate] | last month how much of Rahims goods did Faisal take, at what rate and for how much | not sure; flags scoped,how_much_twice; doubts CloseSpelling, NoParty; no read planned |
| D022 | roman* | 1 | [SaleCount, PartnerTotal] | rahim ne pichle mahine kitna mal becha, kitne bil bane aur khalis rakam kitni bani | flags asks_count,how_much_twice,measure_quantity; no read planned |
| D023 | urdu | 1 | [SaleCount, Sale] | رحیم نے پچھلے مہینے کتنا مال بیچا، کتنے بل بنے اور صافی بکری کتنی رہی | flags asks_count,unexplained,how_much_twice,measure_other; doubts UnknownWords; unknown word: صافی; no read planned |
| D033 | urdu* | 1 | [Commission] | رحیم كا كیلا اس سال كتنا بكا، كتنے كا، اور اس پر كمیشن كتنا لگا | flags unexplained,how_much_twice; doubts UnknownWords; unknown word: لگا; no read planned |
| D034 | en | 1 | [Commission] | how much of Rahim's banana was sold this year, for how much, and what commission was charged on it | flags unexplained,how_much_twice,list; doubts UnknownWords; unknown word: charged; no read planned |
| D037 | roman | 1 | [Sale, PartnerTotal] | karim ka sab se naya bill kis tarikh ka hai, uska number, khaam bikri aur khalis raqam kya hai | flags two_figures,list,measure_breakdown; no read planned |
| D038 | urdu | 1 | [Sale] | کریم کا سب سے نیا بل کس تاریخ کا ہے، اس کا نمبر، خام بکری اور صافی بکری کیا ہے | flags unexplained,list,measure_other; doubts UnknownWords; unknown word: صافی; no read planned |
| D039 | roman | 1 | [Commission, PartnerTotal] | rahim ke is saal ke bills me commission, kiraya, mazdoori aur munshiana kitna kata, kul katautiyan aur net kitna | flags sum_asked,unexplained,how_much_twice,commission_side,measure_other; doubts UnknownWords; no read planned |
| D040 | urdu | 1 | [Commission] | رحیم کے اس سال کے بلوں میں کمیشن، کرایہ، مزدوری اور منشیانہ کتنا کٹا، کل کٹوتی اور صافی کتنی | not sure; flags unexplained,how_much_twice,commission_side,measure_other; doubts UnknownWords, SeveralPeriods; no read planned |
| D041 | en | 1 | [Commission, PartnerTotal] | in Rahim's bills this year, how much was deducted for commission, freight, labour and munshiana, and what are the total deductions and the net | flags sum_asked,unexplained,commission_side,measure_other; doubts UnknownWords; unknown word: commission,+and commission,+munshiana, labour+munshiana,; no read planned |
| D042 | roman | 1 | [Sale] | sarfraz ka angoor is saal kitna bika | planned period,find,find,analytics:A |
| D043 | urdu | 1 | [Sale] | سرفراز کا انگور اس سال کتنے کا بکا | planned period,find,find,analytics:A |
| D044 | en | 1 | [Sale] | how much of Sarfraz grapes was sold this year | planned period,find,find,analytics:A |
| D120 | roman* | 2 | [Sale, SaleCount, Commission] | aj ki kul bikri, kitne bil bane aur kamishan kitna hoa | flags sum_asked,asks_count,how_much_twice; no read planned |
| D121 | en | 1 | [Sale, SaleCount, Commission] | today's sales, the number of bills and the commission | flags asks_count; no read planned |
| D130 | roman | 1 | [Sale, Commission] | pichle mahine kele ki bikri, qty aur kul kamishan kitna raha | flags sum_asked,measure_quantity; planned period,find,analytics:A |
| D131 | urdu | 1 | [Sale, Commission] | پچھلے مہینے کیلے کی بکری، تعداد اور کل کمیشن کتنا رہا | flags unexplained,measure_quantity; doubts UnknownWords; unknown word: تعداد; no read planned |
| D132 | urdu | 1 | [Sale] | اس سال پھل کی بکری کتنی ہوئی اور سبزی کی کتنی | flags item_group,how_much_twice; planned period,analytics:A |
| D133 | en | 1 | [Sale] | this year how much did fruit sell for and how much did vegetables sell for | flags item_group,how_much_twice; planned period,analytics:A |
| D148 | roman | 1 | [Report("R3")] | is saal kele ki naqdi bikri kitni hui, kitne lines, kitni qty aur kitne ki | flags how_much_twice; no read planned |
| D149 | urdu | 1 | [Report("R3")] | اس سال کیلے کی نقدی بکری کتنی ہوئی، کتنی لائنیں، کتنی تعداد اور کتنے کی | flags unexplained,how_much_twice; doubts UnknownWords; unknown word: لائنیں،+تعداد; no read planned |
| D150 | en | 1 | [Cash] | banana sold for cash this year: how many lines, how many units and for how much | flags asks_count,pay_asked,pay_kind,how_much_twice; no read planned |
| D151 | roman | 1 | [MoneyLent, Sale] | pichle mahine phal ki udhaar bikri kitni rahi, kitni qty aur kitne gahakon ko | flags two_figures,item_group,asks_count,pay_asked,pay_kind,role_word,how_much_twice,measure_quantity; no read planned |
| D152 | urdu | 1 | [Report("R4")] | پچھلے مہینے پھل کی ادھاری بکری کتنی رہی، کتنی تعداد اور کتنے گاہکوں کو | flags item_group,asks_count,unexplained,role_word,how_much_twice; doubts UnknownWords; unknown word: تعداد; no read planned |
| D153 | en | 1 | [Sale] | fruit sold on credit last month: for how much, how many units and to how many buyers | flags item_group,asks_count,qty_asked,pay_asked,pay_kind,role_word,how_much_twice,measure_quantity; no read planned |
| D160 | roman | 1 | [Sale] | aaj sabziyon ki bikri kitni hui | flags item_group; planned period,analytics:A |
| D161 | urdu | 1 | [Sale] | آج سبزیوں کی بکری کتنی ہوئی | flags item_group; planned period,analytics:A |
| D165 | roman | 1 | [Commission] | shahid raza ne september me kele ka kitna maal aur kitne ka liya, commission mila kar kitna diya | flags sum_asked,sum_strong,how_much_twice,measure_quantity; planned period,find,find,analytics:A |
| D166 | urdu | 1 | [Commission] | شاہد رضا نے ستمبر میں کیلے کا کتنا مال اور کتنے کا لیا، کمیشن ملا کر کتنا دیا | flags sum_asked,sum_strong,how_much_twice,measure_quantity; planned period,find,find,analytics:A |
| D167 | en | 2 | [Commission] | how much banana did Shahid Raza take in September, for how much, and how much did he pay with commission added | flags unexplained,how_much_twice,commission_side; doubts UnknownWords; unknown word: added; no read planned |
| D209 | roman | 1 | [Sale] | aaj angoor kitne lines me, kitna maal aur kitne ka bika | flags how_much_twice,measure_quantity; planned period,find,analytics:A |
| D210 | urdu | 2 | [Sale] | آج انگور کتنی لائنوں میں، کتنا مال اور کتنے کا بکا | flags unexplained,how_much_twice,measure_quantity; doubts UnknownWords; unknown word: لائنوں; no read planned |
| D211 | en | 1 | [Sale] | how much grapes were sold today, in how many lines and for how much | flags asks_count,how_much_twice; planned period,find,analytics:A |
| D212 | roman | 2 | [Rate] | karim ka aaj angoor kitna bika, kitne ka aur akhri rate kya tha | flags last_one,how_much_twice,list; no read planned |
| D213 | urdu | 2 | [Rate] | کریم کا آج انگور کتنا بکا، کتنے کا اور آخری ریٹ کیا تھا | flags last_one,how_much_twice,list; no read planned |
| D214 | en | 2 | [Rate] | how much of Karim's grapes were sold today, for how much, and what was the last rate | flags last_one,how_much_twice,list; no read planned |

## G. Dues with an age bucket, a collector, a city, or a count of buyers (R20, R21, R22)

Plan: R20 age buckets, R22 collector total, R21 city total, and the row count, as instant lines.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D012 | roman* | 2 | [] | faisal ka sab se purana bil kis tareekh ka he aur kitne din ho gye | not sure; flags oldest,unexplained,list; doubts UnknownWords, NoFigure; unknown word: bil; no read planned |
| D013 | en | 2 | [] | which date is Faisal's oldest unpaid bill and how many days old is it | not sure; flags oldest,asks_count,unexplained,list; doubts UnknownWords, TimeWord, NoFigure; unknown word: unpaid+old; no read planned |
| D019 | roman | 2 | [DeadAmount, Balance] | bilal ahmed ki raqam doobi raqam me kab gayi aur ab bhi kitna dena hai | flags happened; no read planned |
| D020 | urdu* | 2 | [DeadAmount] | بلال احمد ڈوبی رقم میں كب گیا، اس پر ابھی كتنا واجب ہے | flags unexplained; doubts UnknownWords; unknown word: واجب; no read planned |
| D021 | en | 2 | [DeadAmount, Balance] | Bilal Ahmed is in the dead amount: how much does he still owe and when was he moved there | not sure; flags unexplained; doubts UnknownWords, TwoParts; unknown word: still+moved; no read planned |
| D045 | roman | 1 | [Payable] | kitne bayopariyon ko abhi paise dene hain aur kul kitne | flags sum_asked,asks_count,role_word,how_much_twice; planned report:R24 |
| D046 | urdu | 1 | [Payable] | کتنے بیوپاریوں کو ابھی ادائیگی باقی ہے اور کل کتنی رقم بنتی ہے | flags asks_count,role_word,how_much_twice; planned report:R24 |
| D047 | en | 2 | [] | how many sellers are still to be paid and how much is it in all | not sure; flags sum_asked,asks_count,unexplained,role_word,how_much_twice; doubts NoFigure; unknown word: still+paid; no read planned |
| D085 | roman | 2 | [Balance] | 91 din se purane bakaye kitne hain | not sure; doubts NoParty; no read planned |
| D086 | urdu | 2 | [Balance] | نوے دن سے زیادہ پرانا بقایا کل کتنا ہے | not sure; flags compare; doubts Comparison, NoParty; no read planned |
| D087 | roman | 2 | [PartnerShare] | rasheed ke hisse me kitni wasooli hai, usme se 31 se 90 din purani kitni | flags unexplained,how_much_twice; unknown word: usme; no read planned |
| D088 | urdu | 2 | [Balance] | رشید کے ذمے کتنی اگراہی ہے اور اس میں سے اکتیس سے نوے دن پرانی کتنی | not sure; flags how_much_twice; doubts NoParty; no read planned |
| D089 | en | 2 | [Dues] | how much is on Rasheed's collection list and how much of it is 31 to 90 days old | not sure; flags unexplained,how_much_twice; doubts UnknownWords, TimeWord; unknown word: old; no read planned |
| D090 | urdu | 1 | [DeadAmount] | ڈوبی رقم کل کتنی ہے اور کتنے لوگوں کی | flags asks_count,how_much_twice; planned tiles |
| D091 | en | 1 | [DeadAmount] | how much is in the dead amount and for how many people | flags asks_count,unexplained,how_much_twice; doubts UnknownWords; unknown word: people; no read planned |
| D092 | roman | 2 | [Dues] | kitne gahakon ka bakaya 30 din se purana ho gaya aur kitna | flags happened,asks_count,role_word,how_much_twice; no read planned |
| D093 | en | 2 | [Dues] | how many buyers owe a bill older than 30 days and how much in all | not sure; flags sum_asked,asks_count,unexplained,role_word,how_much_twice; doubts UnknownWords, TimeWord; unknown word: older; no read planned |
| D292 | roman | 1 | [Dues] | kasur ke gahakon ke zimme kul kitna baqi hai | flags sum_asked,role_word; planned find,report:R21 |
| D293 | urdu | 1 | [Dues] | قصور کے گاہکوں کے ذمے کل کتنا باقی ہے | flags role_word; planned find,report:R21 |
| D294 | en | 1 | [Dues] | how much do the buyers of Kasur owe in all | flags sum_asked,role_word; planned find,report:R21 |
| D295 | roman | 1 | [Dues] | kasur shehar ke kitne gahakon ka bakaya hai | flags asks_count,role_word; planned find,report:R21 |
| D296 | urdu | 1 | [Dues] | قصور شہر کے کتنے گاہکوں کا بقایا ہے | flags asks_count,role_word; planned find,report:R21 |
| D297 | en | 2 | [Dues] | how many buyers of Kasur have something to pay | flags happened,asks_count,unexplained,role_word; doubts UnknownWords; unknown word: have something; no read planned |
| D298 | roman* | 2 | [Balance] | shehar kasur ka bakaya aur usme se 8 se 30 din purana kitna hai | not sure; flags unexplained,how_much_twice; doubts NoParty; unknown word: usme; no read planned |
| D299 | urdu | 2 | [Balance] | شہر قصور کا بقایا اور اس میں سے آٹھ سے تیس دن پرانا کتنا ہے | not sure; doubts NoParty; no read planned |
| D300 | en | 2 | [Dues] | the dues of the city Kasur and how much of it is 8 to 30 days old | not sure; flags unexplained; doubts UnknownWords, TimeWord; unknown word: old; no read planned |
| D442 | roman | 2 | [Balance] | aath se tees din purane bakaye kitne hain | not sure; flags unexplained; doubts UnknownWords, NoParty; unknown word: aath+tees; no read planned |
| D443 | en | 2 | [Dues] | how much of the dues is 8 to 30 days old | not sure; flags unexplained; doubts UnknownWords, TimeWord; unknown word: old; no read planned |

## B. One party on one day: items, qty, rate, amount, cash on the spot, credit, balance before/after

Plan: R35 party day sheet (buyer or seller, as_of) or R2/R3/R4 filtered by party and day; labelled instant lines for each part asked.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D001 | urdu | 2 | [Cash, MoneyLent] | پچھلے مہینے 14 تاریخ کو اکرم نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار رہا | not sure; flags happened,unexplained,how_much_twice,list,measure_other; doubts UnknownWords, SeveralMatches, SeveralPeriods; unknown word: موقع; no read planned |
| D002 | roman* | 2 | [Cash, MoneyLent] | pichle mahiney 14 tareekh ko akrm ne kya kya khareeda, moqe pr kitna naqad dya aur kitna udhaar raha | not sure; flags unexplained,how_much_twice,list,measure_other; doubts UnknownWords, SeveralMatches, SeveralPeriods; no read planned |
| D003 | en | 2 | [Cash] | what did Akram buy on the 14th of last month, how much did he pay in cash on the spot and how much stayed on credit | not sure; flags happened,pay_asked,pay_kind,unexplained,how_much_twice,list; doubts UnknownWords, SeveralMatches; unknown word: spot+stayed; no read planned |
| D004 | urdu | 2 | [Cash, Balance] | رضوان بٹ نے پچھلے مہینے کی 14 کو کیا کیا لیا، کتنا نقد دیا اور ادھار کتنا بچا | flags two_figures,happened,how_much_twice; no read planned |
| D005 | roman* | 2 | [Cash, Balance] | rizwan but ne pichle mahine ki 14 ko kya kya liya, nakad kitna diya aur udhaar kitna bacha | flags happened,how_much_twice,list; no read planned |
| D006 | en | 2 | [Cash] | what did Rizwan Butt take on the 14th of last month, how much cash did he pay and how much is left on credit | flags happened,pay_asked,pay_kind,how_much_twice; no read planned |
| D007 | roman | 2 | [Balance] | faisal ki aaj ki parchi batao: pehle kitna baqi tha, aaj ka bill kitna bana aur ab kul kitna ho gaya | not sure; flags sum_asked,happened,how_much_twice; doubts SeveralPeriods; no read planned |
| D008 | urdu | 2 | [Balance] | فیصل کی آج کی پرچی: پہلے کتنا باقی تھا، آج کا بل کتنا بنا اور اب کل کتنا ہو گیا | not sure; flags happened,how_much_twice; doubts SeveralPeriods; no read planned |
| D009 | en* | 2 | [Balance] | Faisals slip for todays: what was his balence before, what is todays bil and what does he owe now | not sure; flags unexplained; doubts CloseSpelling, UnknownWords; unknown word: bil; no read planned |
| D030 | roman* | 2 | [Rate] | 14 oct ko basheer ka kya kya bika, kitni qty, kis rate pr aur kitne ka | not sure; flags unexplained,how_much_twice,list; doubts CloseSpelling, UnknownWords, SeveralMatches; unknown word: pr; no read planned |
| D031 | urdu | 2 | [Rate] | بشیر کا 14 اکتوبر کو کیا کیا بکا، کتنی تعداد، کس ریٹ پر اور کتنے کا | flags unexplained,how_much_twice,list; doubts UnknownWords; unknown word: تعداد،; no read planned |
| D032 | en | 2 | [Rate] | what did Bashir sell on 14 October, how many, at what rate and for how much | flags asks_count,how_much_twice,list; no read planned |
| D273 | urdu | 2 | [Cash, Balance] | پچھلے مہینے 14 تاریخ کو نوید ٹریڈرز نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار رہا | not sure; flags happened,unexplained,how_much_twice,list; doubts UnknownWords, SeveralPeriods; unknown word: موقع; no read planned |
| D274 | roman | 2 | [Cash, Balance] | pichle mahine 14 tarikh ko naveed traders ne kya kya khareeda, moqe par kitna naqad diya aur kitna udhaar raha | not sure; flags happened,unexplained,how_much_twice,list; doubts UnknownWords, SeveralPeriods; unknown word: moqe; no read planned |
| D275 | en | 2 | [Cash] | what did Naveed Traders buy on the 14th of last month, how much cash did he pay on the spot and how much stayed on credit | flags happened,pay_asked,pay_kind,unexplained,how_much_twice,list; doubts UnknownWords; unknown word: spot+stayed; no read planned |
| D276 | roman | 2 | [Cash] | naveed traders ne 14 october ko nakad kitna diya | flags happened; no read planned |
| D277 | urdu | 2 | [Cash] | نوید ٹریڈرز نے 14 اکتوبر کو نقد کتنا دیا | flags happened; no read planned |
| D278 | en | 2 | [Cash] | how much did Naveed Traders pay in cash on 14 October | flags happened; no read planned |
| D279 | roman* | 2 | [Balance] | naveed traders ka 14 october ka udhar kitna raha | no read planned |
| D280 | urdu | 2 | [Balance] | نوید ٹریڈرز کا 14 اکتوبر کا ادھار کتنا رہا | no read planned |
| D281 | en | 2 | [] | how much stayed on credit for Naveed Traders on 14 October | not sure; flags pay_asked,pay_kind,unexplained; doubts UnknownWords, NoFigure; unknown word: stayed; no read planned |
| D282 | roman | 2 | [Cash, Rate] | naveed traders ne 14 october ko nakad me kela kitna, kitne ka aur kis rate par liya | flags happened,how_much_twice; no read planned |
| D283 | urdu | 2 | [Cash, Rate] | نوید ٹریڈرز نے 14 اکتوبر کو نقد میں کیلا کتنا، کتنے کا اور کس ریٹ پر لیا | flags happened,how_much_twice; no read planned |
| D284 | en | 2 | [Cash, Rate] | how much banana did Naveed Traders buy for cash on 14 October, for how much and at what rate | flags how_much_twice,list; no read planned |
| D285 | roman | 2 | [Balance] | pukhta khata ke aamir ne kitna maal liya, kitne ka aur ab kitna baqi hai | not sure; flags scoped,happened,unexplained,how_much_twice; doubts UnknownWords, NoParty; unknown word: ke aamir; no read planned |
| D286 | urdu | 2 | [Balance] | پختہ کھاتہ کے عامر نے کتنا مال لیا، کتنے کا اور اب کتنا باقی ہے | not sure; flags scoped,happened,unexplained,how_much_twice; doubts UnknownWords, NoParty; unknown word: عامر; no read planned |
| D287 | en | 2 | [Balance] | how much did Aamir of pukhta khata take, for how much and how much does he owe now | not sure; flags scoped,happened,unexplained,how_much_twice; doubts UnknownWords, NoParty; unknown word: Aamir; no read planned |

## D. Rate questions: last rate with "laga" as a verb, highest and lowest, rate of a party

Plan: "kya laga / لگا / got" after rate is a verb (not the Laga charge); R2 rows (newest, min, max) as instant lines.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D035 | urdu* | 1 | [Rate] | کریم کا سیب پچھلے مہینے کتنا بکا، سب سے مہنگا اور سب سے سستا ریٹ کیا تھا | flags unexplained,list; doubts UnknownWords; unknown word: مہنگا+سستا; no read planned |
| D036 | en | 1 | [Rate] | Karim's apples last month: how many were sold, and what were the highest and the lowest rate | flags asks_count,list; no read planned |
| D137 | roman | 2 | [Rate, Laga] | is hafte seb ka aakhri rate kya laga | flags last_one; no read planned |
| D138 | urdu | 2 | [Rate] | اس ہفتے سیب کا آخری ریٹ کیا لگا | flags last_one,unexplained; doubts UnknownWords; unknown word: لگا; no read planned |
| D183 | roman* | 2 | [FruitTaken, Rate] | haji nazir ki dukaan se mehmano ke liye kitna phal liya, kis rate par, kitne ka aur kis tarikh ko | not sure; flags guests,how_much_twice; doubts NoParty; no read planned |
| D184 | urdu | 2 | [FruitTaken, Rate] | حاجی نذیر کی دکان سے مہمانوں کے لیے کتنا پھل لیا، کس ریٹ پر، کتنے کا اور کس تاریخ کو | not sure; flags guests,how_much_twice; doubts NoParty; no read planned |
| D185 | en | 2 | [FruitTaken, Rate] | how much fruit did we take for guests from Haji Nazeer's shop, at what rate, for how much and on which date | not sure; flags guests,how_much_twice; doubts NoParty; no read planned |
| D200 | roman | 2 | [Rate, Laga] | aaj angoor ka aakhri rate kya laga | flags last_one; no read planned |
| D201 | urdu | 2 | [Rate] | آج انگور کا آخری ریٹ کیا لگا | flags last_one,unexplained; doubts UnknownWords; unknown word: لگا; no read planned |
| D204 | urdu | 2 | [Rate] | شاہد رضا کو آج انگور کا آخری ریٹ کیا لگا | flags last_one,unexplained; doubts UnknownWords; unknown word: لگا; no read planned |
| D205 | en | 2 | [Rate] | what was the last rate of grapes that Shahid Raza got today | flags last_one,unexplained; doubts UnknownWords; unknown word: got; no read planned |
| D206 | roman | 2 | [Rate] | is hafte angoor ka sab se mehnga aur sab se sasta rate kya raha | flags unexplained,list; doubts UnknownWords; unknown word: mehnga+sasta; no read planned |
| D207 | urdu | 2 | [Rate] | اس ہفتے انگور کا سب سے مہنگا اور سب سے سستا ریٹ کیا رہا | flags unexplained,list; doubts UnknownWords; unknown word: مہنگا+سستا; no read planned |
| D208 | en | 2 | [Rate] | this week what was the highest and the lowest rate of grapes | flags list; no read planned |
| D304 | roman | 2 | [Rate] | rafique brothers ne 22 october ko seb gahak ke rate par kitne ka liya, kis rate par aur kitna | flags buyer_rate,role_word,how_much_twice; no read planned |
| D305 | urdu | 2 | [Rate] | رفیق برادرز نے 22 اکتوبر کو سیب گاہک کے ریٹ پر کتنے کا لیا، کس ریٹ پر اور کتنا | flags buyer_rate,role_word,how_much_twice; no read planned |
| D306 | en | 2 | [Rate] | at the buyer's rate, what rate and how much apple did Rafique Brothers take on 22 October, and for how much | flags role_word,how_much_twice; no read planned |
| D307 | roman | 2 | [Rate] | jis line me gahak ka rate alag tha uska farq kitna bana, bayopari ka rate aur gahak ka rate kya tha | not sure; flags compare,buyer_rate,unexplained,role_word; doubts UnknownWords, Comparison, NoParty; unknown word: alag+farq; no read planned |
| D308 | urdu | 2 | [Rate] | جس لائن میں گاہک کا ریٹ الگ تھا اس کا فرق کتنا بنا، بیوپاری کا ریٹ اور گاہک کا ریٹ کیا تھا | not sure; flags compare,buyer_rate,unexplained,role_word; doubts UnknownWords, Comparison, NoParty; no read planned |
| D309 | en | 2 | [Rate] | on the line where the buyer rate differed: what was the difference, the seller rate and the buyer rate | not sure; flags compare,buyer_rate,unexplained,role_word; doubts UnknownWords, Comparison; unknown word: rate+difference, differed:+difference,; no read planned |
| D310 | roman | 2 | [Rate, Laga] | 19 october ko moosami ka aakhri rate kya laga | flags last_one; no read planned |
| D311 | urdu | 2 | [Rate] | 19 اکتوبر کو موسمی کا آخری ریٹ کیا لگا | flags last_one,unexplained; doubts UnknownWords; unknown word: لگا; no read planned |
| D313 | roman | 2 | [Rate] | 19 october ko moosami kitne lines me, kitna aur kitne ka bika, sab se mehnga aur sab se sasta rate | flags unexplained,how_much_twice; doubts UnknownWords; unknown word: mehnga+sasta; no read planned |
| D314 | urdu | 2 | [Rate] | 19 اکتوبر کو موسمی کتنی لائنوں میں، کتنا اور کتنے کا بکا، سب سے مہنگا اور سب سے سستا ریٹ | flags unexplained,how_much_twice; doubts UnknownWords; no read planned |
| D315 | en | 2 | [Rate] | on 19 October how much mosambi sold, in how many lines and for how much, and the highest and lowest rate | flags asks_count,how_much_twice,list; no read planned |

## H. Partner figures: share, capital, taken and returned, gardens/trips/fruit, ranking

Plan: View F row and R29 partner lines as labelled instant lines (income share, expense share, profit, taken, returned, capital, percent, total).

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D048 | roman* | 2 | [PartnerShare] | alif ka is mahine ka hisa kitna bna aur is sal ka kitna | not sure; flags how_much_twice; doubts SeveralPeriods; no read planned |
| D049 | en | 1 | [PartnerShare, Profit] | what is Partner A's share of profit this month and this year | not sure; doubts SeveralPeriods; no read planned |
| D050 | roman | 2 | [] | alif ne ab tak kitne paise liye aur kitne wapas kiye | not sure; flags capital_flow,how_much_twice; doubts NoFigure; no read planned |
| D051 | en* | 2 | [] | how much has Partner A taken for himself so far and how much has he retured | not sure; flags unexplained,how_much_twice; doubts UnknownWords, NoFigure; unknown word: he retured; no read planned |
| D052 | roman | 2 | [Capital] | b partner ne dukaan me kitna sarmaya lagaya hua hai | flags capital_flow,happened,unexplained; doubts UnknownWords; unknown word: lagaya; no read planned |
| D053 | urdu | 2 | [Capital] | ب شراکت دار کا کتنا سرمایہ لگا ہوا ہے | flags unexplained; doubts UnknownWords; unknown word: لگا; no read planned |
| D054 | roman* | 2 | [PartnerShare, PartnerTotal] | usman shirakat daar ka hissa kitne feesad hai aur uski kul raqam kitni hai | flags sum_asked,how_much_twice; no read planned |
| D055 | en | 2 | [PartnerShare] | what percent is Usman's share and what is his total in the shop | flags sum_asked; no read planned |
| D056 | roman* | 1 | [FruitTaken] | alif ko is sal bagh, safr aur phal me kitna laga ya mila | flags scoped; no read planned |
| D057 | en | 1 | [FruitTaken] | what did Partner A put in or get from gardens, trips and fruit taken this year | flags scoped,capital_flow,unexplained; doubts UnknownWords; unknown word: get; no read planned |
| D058 | urdu | 1 | [PartnerShare, Expense, Profit] | الف کی اس سال کی آمدن کا حصہ، خرچے کا حصہ اور منافع کتنا ہے | flags unexplained; doubts UnknownWords; unknown word: آمدن; no read planned |
| D059 | en* | 1 | [PartnerShare, Expense, Profit] | Partner As share of incom, share of expences and profit this year | flags unexplained; doubts UnknownWords; unknown word: As+incom, share+of; no read planned |
| D060 | roman | 2 | [] | kis shirakat daar ne sab se zyada paise liye aur kitne | not sure; flags list; doubts NoFigure; no read planned |
| D061 | urdu | 2 | [] | کس شراکت دار نے سب سے زیادہ ذاتی رقم لی اور کتنی | not sure; flags unexplained,list; doubts UnknownWords, NoFigure; unknown word: ذاتی; no read planned |
| D062 | en | 2 | [PartnerTaken] | which partner has taken the most money for himself and how much | flags list; no read planned |
| D065 | roman* | 1 | [Expense] | pichle mahine dukan ka kharcha kitna hua aur sharkat daron ne kitna liya | flags how_much_twice; planned period,report:R29 |
| D066 | en | 1 | [Expense, PartnerTaken] | last month's shop expenses and the money the partners took for themselves | no read planned |
| D329 | en* | 1 | [] | what is Partner As totel in the shop | not sure; flags unexplained; doubts UnknownWords; unknown word: As+totel; no read planned |
| D330 | urdu | 2 | [Capital] | عثمان نے کتنا سرمایہ لگایا ہے | flags capital_flow,happened,unexplained; doubts UnknownWords; unknown word: لگایا; no read planned |
| D416 | urdu | 1 | [PartnerShare] | عثمان کا اس سال کا حصہ کتنا بنا | no read planned |
| D417 | en | 1 | [PartnerShare, Profit] | what is Usman's share of profit this year | no read planned |
| D418 | roman* | 1 | [PartnerShare] | b ka is mahine ka hisa kitna bana | no read planned |
| D419 | urdu | 1 | [PartnerShare] | ب کا اس مہینے کا حصہ کتنا ہے | no read planned |

## K. Counts and states: accounts, staff, heads, packings, day book entries, trial balance, closed days

Plan: Count of rows and the totals of R11, R18, R19, the day state, the master lists (app_state), as instant lines.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D105 | roman | 1 | [DayBook] | aaj ke rozname me kitni entries hain aur banaam ka total kitna hai | flags sum_asked,asks_count,how_much_twice; planned period,report:R11 |
| D106 | urdu | 1 | [DayBook] | آج کے روزنامچے میں کتنی اندراجات ہیں اور بنام کا میزان کتنا ہے | flags sum_asked,how_much_twice; planned period,report:R11 |
| D107 | en | 2 | [DayBook] | how many entries are in today's day book and what is the debit total | flags sum_asked,asks_count,unexplained; doubts UnknownWords; unknown word: debit; no read planned |
| D108 | roman | 2 | [TrialBalance] | trial balance ka banaam aur jama kitna hai aur farq kitna hai | not sure; flags compare,unexplained,how_much_twice; doubts Comparison; unknown word: farq; no read planned |
| D109 | urdu | 2 | [TrialBalance] | ٹرائل بیلنس کا بنام اور جمع کتنا ہے اور دونوں میں فرق کتنا ہے | not sure; flags compare,unexplained,how_much_twice; doubts UnknownWords, Comparison; unknown word: جمع+فرق; no read planned |
| D110 | en | 2 | [TrialBalance] | what are the debit and credit totals of the trial balance and the difference between them | not sure; flags compare,pay_asked,pay_kind,unexplained; doubts UnknownWords, Comparison; no read planned |
| D111 | roman* | 1 | [TrialBalance] | shuru ke trial balance me kitna banaam aur jama tha | planned report:R18 |
| D112 | urdu | 2 | [Report("R19"), TrialBalance] | ابتدائی ٹرائل بیلنس میں بنام اور جمع کتنا تھا | flags two_figures,unexplained; doubts UnknownWords; unknown word: جمع; no read planned |
| D113 | en | 2 | [Report("R19")] | what were the debit and credit of the opening trial balance | flags pay_asked,pay_kind,unexplained; doubts UnknownWords; unknown word: debit; no read planned |
| D114 | urdu | 2 | [Ledger] | کل کتنے کھاتے بنے ہوئے ہیں | not sure; flags asks_count; doubts NoParty; no read planned |
| D115 | en | 2 | [] | how many accounts are there in all | not sure; flags sum_asked,asks_count; doubts NoFigure; no read planned |
| D116 | roman | 2 | [Balance] | din kis tarikh tak band ho chuka hai aur kitne din abhi khule hain | not sure; flags happened; doubts NoParty; no read planned |
| D117 | en | 1 | [Ask("days")] | up to which date are the days closed and how many days are still open | flags asks_count,unexplained; unknown word: still; planned app_state |
| D195 | roman | 1 | [Ask("staff_munshi")] | dukaan me kitne munshi, kitne ugrahi wale aur kul kitne mulazim hain | flags sum_asked,how_much_twice; planned find |
| D196 | urdu | 1 | [Ask("staff_munshi"), Ask("staff_collector"), Ask("staff_all")] | دکان میں کتنے منشی، کتنے اگراہی والے اور کل کتنے ملازم ہیں | flags how_much_twice; planned find |
| D197 | en | 1 | [Ask("staff_munshi"), Ask("staff_collector"), Ask("staff_all")] | how many munshis, how many collectors and how many staff in all | flags sum_asked,asks_count,how_much_twice; planned find |
| D198 | urdu | 2 | [Expense, Ask("packings"), Ask("stores")] | کتنی خرچہ مدیں، کتنی پیکنگ، کتنے سٹور اور کتنے مارکے بنے ہوئے ہیں | not sure; flags unexplained,how_much_twice; doubts UnknownWords, OtherRecord, UnknownWords; unknown word: مدیں،+مارکے; no read planned |
| D199 | en | 2 | [Ask("heads")] | how many expense heads, packings, stores and marks are set up | flags asks_count,unexplained; doubts UnknownWords; no read planned |
| D348 | urdu | 2 | [TrialBalance] | ٹرائل بیلنس میں فرق کتنا ہے | not sure; flags compare,unexplained; doubts UnknownWords, Comparison; unknown word: فرق; no read planned |
| D349 | en | 2 | [TrialBalance] | is there any difference in the trial balance, how much | not sure; flags compare,unexplained; doubts UnknownWords, Comparison; unknown word: difference; no read planned |
| D440 | urdu | 1 | [TrialBalance] | ٹرائل بیلنس کا بنام کل کتنا ہے | planned report:R18 |
| D441 | en | 2 | [TrialBalance] | what is the total debit of the trial balance | flags sum_asked,unexplained; doubts UnknownWords; unknown word: debit; no read planned |

## M. A document number: sale number, voucher number, truck, audit

Plan: R1 with the bikri filter (gross, lines, date, commission); R15 / R23 / R25 by voucher number; R1 by truck; audit count.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D118 | roman | 2 | [Sale] | bikri number 1 ka record kitni baar likha gaya | flags document_number,bikri_number,asks_count,unexplained; doubts UnknownWords; unknown word: likha; no read planned |
| D119 | urdu | 2 | [Sale] | بکری نمبر 1 کا ریکارڈ کتنی بار لکھا گیا ہے | flags document_number,bikri_number,asks_count,unexplained; doubts UnknownWords; unknown word: لکھا; no read planned |
| D192 | roman | 2 | [Sale, PartnerTotal] | LES-1057 truck kitni baar aya, akhri baar kab, khaam bikri aur khalis raqam kya thi | flags two_figures,amount_named,truck,last_one,asks_count,unexplained,measure_other; doubts UnknownWords; unknown word: truck; no read planned |
| D193 | urdu | 2 | [Sale] | ٹرک LES-1057 کتنی بار آیا، آخری بار کب، خام بکری اور صافی بکری کیا تھی | flags amount_named,truck,last_one,asks_count,unexplained,measure_other; doubts UnknownWords; unknown word: ٹرک+صافی; no read planned |
| D194 | en | 2 | [Sale, PartnerTotal] | how many times did truck LES-1057 come, when was the last time and what were the gross and the net | not sure; flags two_figures,amount_named,truck,last_one,asks_count,unexplained,measure_other; doubts TimeWord; unknown word: times+come,; no read planned |
| D221 | roman | 2 | [Sale] | bikri number 100 me kitna maal bika, kitne ka, kitne lines aur kis tarikh ko | flags amount_named,document_number,bikri_number,how_much_twice,measure_quantity; no read planned |
| D222 | urdu | 2 | [Sale] | بکری نمبر 100 میں کتنا مال بکا، کتنے کا، کتنی لائنیں اور کس تاریخ کو | flags amount_named,document_number,bikri_number,unexplained,how_much_twice,measure_quantity; doubts UnknownWords; unknown word: کتنی لائنیں; no read planned |
| D223 | en | 2 | [Sale] | in sale number 100 how much was sold, for how much, how many lines and on which date | flags amount_named,document_number,bikri_number,asks_count,how_much_twice; no read planned |
| D224 | roman | 2 | [Sale, Commission] | bikri number 100 ka total commission kitna tha | flags sum_asked,amount_named,document_number,bikri_number; no read planned |
| D225 | urdu | 2 | [Sale, Commission] | بکری نمبر 100 کا کل کمیشن کتنا تھا | not sure; flags amount_named,document_number,bikri_number; doubts TenseUnclear; no read planned |
| D226 | roman | 2 | [Sale] | bikri number 466 ki kul bikri, qty aur lines batao | flags sum_asked,amount_named,document_number,bikri_number,measure_other; no read planned |
| D227 | urdu | 2 | [Sale] | بکری نمبر 466 کی کل بکری، تعداد اور لائنیں بتائیں | not sure; flags amount_named,document_number,bikri_number,unexplained,measure_other; doubts UnknownWords, TenseUnclear; unknown word: تعداد+لائنیں; no read planned |
| D228 | en | 2 | [Sale] | the total sale, quantity and number of lines of sale number 466 | flags sum_asked,amount_named,document_number,bikri_number,asks_count,measure_other; no read planned |
| D244 | roman | 2 | [Dues] | wasooli number 100 kitne ki thi, kis ki aur kab | flags amount_named,document_number,happened; no read planned |
| D245 | urdu | 2 | [Dues] | وصولی نمبر 100 کتنے کی تھی، کس کی اور کب | flags amount_named,document_number,happened; no read planned |
| D246 | en | 2 | [] | how much was receipt number 100, from whom and when | not sure; flags amount_named,document_number; doubts NoFigure; no read planned |
| D247 | roman | 2 | [Payable] | adaigi number 50 kitne ki thi, kis ko aur kab | flags document_number,happened; no read planned |
| D248 | urdu | 2 | [Payable] | ادائیگی نمبر 50 کتنے کی تھی، کس کو اور کب | flags document_number,happened; no read planned |
| D249 | en | 2 | [] | how much was payment number 50, to whom and when | not sure; flags document_number; doubts NoFigure; no read planned |

## F. Cash book, receipts and payments totals (R12, R23, R25), with counts

Plan: Whole-filter totals of R12 (opening, in, out, closing), R23 (receipts, count), R25 (payments, count) as instant lines; a role word ("from buyers", "to sellers") picks R23 / R25.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D017 | roman | 1 | [Receipts] | akram toll tax ne pichle mahine kitne paise diye aur kitni baar | flags how_much_twice; no read planned |
| D018 | en* | 1 | [] | how much did we colect from Akram Toll Tax last month and in how many reciepts | not sure; flags asks_count,unexplained,how_much_twice; doubts UnknownWords, NoFigure; unknown word: we colect; no read planned |
| D095 | urdu* | 2 | [Cash] | روكڑ میں اس وقت كتنی رقم ہے | flags unexplained; doubts UnknownWords; unknown word: وقت; no read planned |
| D096 | roman* | 1 | [Cash] | is mahine ki roknad ki kitaab: shuru me kitna tha, kitna aya, kitna gya aur ab kitna he | flags happened,unexplained,how_much_twice; doubts UnknownWords; unknown word: kitaab:; no read planned |
| D097 | en | 1 | [Cash] | the cash book this month: opening, cash in, cash out and closing | flags happened,unexplained; doubts UnknownWords; no read planned |
| D098 | roman | 2 | [Cash] | kal galle me kitna aya, kitna gaya aur raat ko kitna bacha | flags happened,how_much_twice; no read planned |
| D099 | urdu | 2 | [Cash] | کل روکڑ میں کتنا آیا، کتنا گیا اور رات کو کتنا بچا | not sure; flags happened,how_much_twice; doubts TenseUnclear; no read planned |
| D100 | en* | 1 | [Cash] | yesterday how much cash came in, how much went out and how much was left at nite | flags unexplained,how_much_twice; doubts UnknownWords; no read planned |
| D101 | roman | 2 | [Payable] | aaj gahakon se kitni wasooli hui aur bayopariyon ko kitni adaigi hui | flags happened,role_word,how_much_twice; no read planned |
| D102 | en | 2 | [] | how much was received from buyers today and how much was paid to sellers | not sure; flags unexplained,role_word,how_much_twice; doubts NoFigure; unknown word: paid; no read planned |
| D103 | roman | 1 | [Receipts] | pichle mahine kitni wasooli hui, kitni rasidein bani aur bayopariyon ko kitna diya, kitni baar | flags unexplained,role_word,how_much_twice; doubts UnknownWords; unknown word: rasidein; no read planned |
| D104 | urdu | 1 | [Receipts] | پچھلے مہینے کتنی وصولی ہوئی، کتنی رسیدیں بنیں اور بیوپاریوں کو کتنا دیا، کتنی بار | flags unexplained,role_word,how_much_twice; doubts UnknownWords; unknown word: رسیدیں+بنیں اور+بیوپاریوں; no read planned |
| D446 | urdu | 2 | [Cash] | کل روکڑ میں سے کتنا گیا | not sure; flags happened; doubts TenseUnclear; no read planned |
| D447 | en | 1 | [Cash] | how much cash went out yesterday | flags unexplained; doubts UnknownWords; unknown word: out; no read planned |
| D448 | roman | 1 | [] | pichle mahine bayopariyon ko kul kitna diya | not sure; flags sum_asked,role_word; doubts NoFigure; no read planned |
| D449 | en | 1 | [] | how much was paid to sellers last month | not sure; flags unexplained,role_word; doubts NoFigure; unknown word: paid; no read planned |
| D450 | roman | 1 | [Receipts] | is hafte gahakon se kitni wasooli hui | flags role_word; planned period,report:R23 |
| D451 | urdu | 1 | [Receipts] | اس ہفتے گاہکوں سے کتنی وصولی ہوئی | flags role_word; planned period,report:R23 |

## E. Quality (awwal / doyam / soyam)

Plan: View A grouped by grade (or R2 with the grade filter) as instant lines; last rate with a grade.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D134 | roman | 1 | [Rate] | pichle mahine awwal darje ka seb kitna bika, kitne ka aur ausat rate kya raha | flags quality,average_asked,how_much_twice,list; planned period,find,analytics:A |
| D135 | urdu | 1 | [Rate] | پچھلے مہینے اول درجے کا سیب کتنا بکا، کتنے کا اور اوسط ریٹ کیا رہا | flags quality,average_asked,how_much_twice,list; planned period,find,analytics:A |
| D136 | en* | 1 | [Rate] | last month how much first quality apple sold, for how much and the average rate | flags quality,average_asked,how_much_twice; planned period,find,analytics:A |
| D145 | roman | 1 | [] | usman ghani ne is saal awwal darje ka seb kitni baar liya, kitna aur kitne ka | not sure; flags quality,asks_count,how_much_twice; doubts NoFigure; no read planned |
| D146 | urdu | 1 | [] | عثمان غنی نے اس سال اول درجے کا سیب کتنی بار لیا، کتنا اور کتنے کا | not sure; flags quality,asks_count,how_much_twice; doubts NoFigure; no read planned |
| D147 | en | 1 | [Purchases] | how many times did Usman Ghani buy first quality apple this year, how much and for how much | flags quality,asks_count,unexplained,how_much_twice; unknown word: times; planned period,find,find,analytics:A |
| D162 | roman | 1 | [Purchases] | akram baloch ne pichle mahine doyam darje ka kela kitni baar, kitna aur kitne ka liya | flags quality,asks_count,how_much_twice; planned period,find,find,analytics:A |
| D163 | urdu | 1 | [] | اکرم بلوچ نے پچھلے مہینے دوم درجے کا کیلا کتنی بار، کتنا اور کتنے کا لیا | not sure; flags quality,asks_count,how_much_twice; doubts NoFigure; no read planned |
| D164 | en | 1 | [Purchases] | how many times did Akram Baloch buy second quality banana last month, how much and for how much | flags quality,asks_count,unexplained,how_much_twice; unknown word: times; planned period,find,find,analytics:A |
| D229 | roman | 1 | [Rate] | pichle mahine awwal darje ke seb ka aakhri rate kya tha | flags quality,last_one; planned period,find,report:R2 |
| D230 | urdu | 1 | [Rate] | پچھلے مہینے اول درجے کے سیب کا آخری ریٹ کیا تھا | flags quality,last_one; planned period,find,report:R2 |
| D231 | en | 1 | [Rate] | what was the last rate of first quality apple last month | flags quality,last_one; planned period,find,report:R2 |
| D232 | roman | 1 | [Sale] | pichle mahine seb awwal, doyam aur soyam darje ka alag alag kitna bika | flags quality,unexplained; doubts UnknownWords; unknown word: kitna; no read planned |
| D233 | urdu | 1 | [Sale] | پچھلے مہینے سیب اول، دوم اور سوم درجے کا الگ الگ کتنا بکا | flags quality,unexplained; doubts UnknownWords; unknown word: الگ+الگ; no read planned |
| D234 | en | 1 | [Sale] | last month how much apple sold in first, second and third quality, each | flags quality,measure_breakdown; planned period,find,report:R2 |

## I. Expense heads, one or two together, and a head ranking

Plan: R29 row of a head (head names in Roman: chai, kiraya, bijli); two heads added together by the calculator; ranking by head.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D069 | roman | 2 | [Expense] | dukaan ke kiraye aur bijli ka kharcha mila kar ab tak kitna hua | flags sum_asked,sum_strong,unexplained; doubts UnknownWords; unknown word: kiraye aur; no read planned |
| D070 | urdu | 2 | [Expense] | دکان کے کرایے اور بجلی کا خرچہ ملا کر اب تک کتنا ہوا | flags sum_asked,sum_strong; no read planned |
| D071 | en* | 2 | [] | shop rent and electricity together, how much so far | not sure; flags sum_asked,sum_strong,unexplained; doubts UnknownWords, NoFigure; unknown word: rent+together,; no read planned |
| D072 | urdu | 1 | [Expense] | اس سال سب سے زیادہ خرچہ کس مد میں ہوا اور کتنا | flags unexplained,list; doubts UnknownWords; unknown word: مد; no read planned |
| D073 | en | 1 | [Expense] | which expense head has the most spending this year and how much | flags unexplained,list; unknown word: head; no read planned |
| D265 | roman | 1 | [Expense] | is saal chai ka kharcha kitna hua | flags unexplained; unknown word: chai; planned period,report:R29 |
| D332 | roman* | 2 | [Expense] | ab tk chae par kitna kharcha hua | flags unexplained; doubts UnknownWords; unknown word: chae; no read planned |
| D420 | roman | 2 | [Expense] | ab tak dukaan ke kiraye par kitna kharcha hua | flags unexplained; doubts UnknownWords; unknown word: kiraye; no read planned |
| D421 | en | 1 | [Expense] | how much has been spent on shop rent so far | flags unexplained; unknown word: rent; planned period,report:R29 |

## L. Stock: quantity and lots, oldest waiting, goods received

Plan: R10 totals (qty, number of lots, days waiting) and the lots received (R6 / R10) as instant lines.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D122 | roman | 2 | [Stock] | moosami ka kitna maal bacha hua hai aur kitne lot me | flags asks_count,unexplained,how_much_twice; doubts UnknownWords; unknown word: lot; no read planned |
| D123 | en | 2 | [Stock] | how much mosambi is left in stock and in how many lots | flags asks_count,unexplained,how_much_twice; doubts UnknownWords; unknown word: lots; no read planned |
| D124 | roman | 2 | [Stock] | sabziyon ka kul kitna maal baqi hai aur kitne lot | flags sum_asked,item_group,asks_count,unexplained,how_much_twice; doubts UnknownWords; unknown word: lot; no read planned |
| D125 | urdu | 2 | [Stock] | سبزیوں کا کل کتنا مال باقی ہے اور کتنے لاٹ | flags item_group,unexplained,how_much_twice; doubts UnknownWords; unknown word: لاٹ; no read planned |
| D126 | urdu | 2 | [Stock] | سب سے پرانا مال کتنے دن سے پڑا ہے اور کتنے لاٹ دو دن سے زیادہ رکے ہوئے ہیں | not sure; flags compare,unexplained,how_much_twice; doubts UnknownWords, Comparison; unknown word: لاٹ+رکے; no read planned |
| D127 | en | 2 | [Stock] | how many days has the oldest stock been waiting and how many lots have waited more than 2 days | not sure; flags asks_count,unexplained,how_much_twice,list; doubts UnknownWords, TimeWord; no read planned |
| D128 | roman | 1 | [] | pichle mahine kele ka kitna maal aya aur kitne lot me | not sure; flags asks_count,unexplained,how_much_twice; doubts UnknownWords, NoFigure; unknown word: lot; no read planned |
| D129 | en | 1 | [] | how much banana came in last month and in how many lots | not sure; flags asks_count,unexplained,how_much_twice; doubts UnknownWords, NoFigure; unknown word: came+lots; no read planned |

## O. Comparisons, best month, two periods, cash against credit

Plan: View A compare / D by month / two reads, with a labelled difference line; R3 and R4 for cash against credit.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D142 | roman | 1 | [Sale] | is mahine kele aur seb me se kis ki bikri zyada hui aur farq kitna hai | not sure; flags compare,unexplained,how_much_twice,measure_quantity; doubts Comparison; unknown word: farq; no read planned |
| D143 | urdu | 1 | [Sale] | اس مہینے کیلے اور سیب میں سے کس کی بکری زیادہ ہوئی اور فرق کتنا ہے | not sure; flags compare,unexplained,measure_quantity; doubts UnknownWords, Comparison; unknown word: فرق; no read planned |
| D144 | en | 1 | [Sale] | this month which sold more, banana or apple, and by how much | not sure; flags compare,unexplained; doubts Comparison; unknown word: more,; no read planned |
| D158 | roman | 2 | [Cash, Sale, MoneyLent] | aaj nakad bikri kitni hui aur udhaar kitni | flags happened,pay_asked,how_much_twice,measure_other; no read planned |
| D159 | en | 2 | [Cash] | how much was sold for cash today and how much on credit | flags pay_asked,how_much_twice; no read planned |
| D168 | roman | 1 | [Sale] | is saal kele ka sab se acha mahina konsa raha aur us mahine kitni bikri hui | flags unexplained,measure_quantity; doubts UnknownWords; unknown word: acha; no read planned |
| D169 | urdu | 2 | [Sale] | اس سال کیلے کا سب سے اچھا مہینہ کون سا رہا اور اس مہینے کتنی بکری ہوئی | not sure; flags unexplained,measure_quantity; doubts UnknownWords, SeveralPeriods; unknown word: اچھا+سا; no read planned |
| D170 | en | 1 | [Sale] | which month was the best for banana this year and how much sold in that month | flags list; no read planned |

## Q. Rules and the "kal" total word

Plan: The charge rule of a party and item (app_state rules) as an instant answer; "kal" before a figure noun asks "کل کون سا؟" instead of the model.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D188 | roman | 2 | [Commission] | rahim ke kele par hum kitna kamishan lete hain | flags unexplained; doubts UnknownWords; unknown word: kamishan lete; no read planned |
| D189 | en | 2 | [Commission] | what percent commission do we charge on Rahim's banana | flags measure_other; no read planned |
| D190 | roman | 1 | [Ask("rules")] | seb par gahak se laga kitna lagta hai | flags role_word; planned find,app_state |
| D191 | urdu | 1 | [Ask("rules")] | سیب پر گاہک سے لاگا کتنا لگتا ہے | flags role_word; planned find,app_state |
| D271 | roman | 2 | [Commission] | kal commission kitna hai | not sure; flags how_much_twice; doubts TenseUnclear; no read planned |
| D272 | urdu | 2 | [Commission] | کل کمیشن کتنا ہے | not sure; doubts TenseUnclear; no read planned |

## N. A packing word (peti, boriyan, box, bag)

Plan: A packing filter on view A and R2 (known gap R6).

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D238 | roman | 1 | [Sale] | pichle mahine aloo ki peti me kitni bikri hui | flags measure_quantity; planned period,find,analytics:A |
| D239 | urdu | 1 | [Sale] | پچھلے مہینے آلو کی پیٹی میں کتنی بکری ہوئی | flags measure_quantity; planned period,find,analytics:A |
| D240 | en | 1 | [Sale] | how many potatoes were sold in boxes last month | flags asks_count,measure_quantity; planned period,find,analytics:A |
| D241 | roman | 1 | [Sale] | pichle mahine aloo ki boriyan kitni bikin aur kitne ki | flags how_much_twice,measure_quantity; planned period,find,analytics:A |
| D242 | urdu | 1 | [Sale] | پچھلے مہینے آلو کی بوریاں کتنی بکیں اور کتنے کی | flags how_much_twice,measure_quantity; planned period,find,analytics:A |
| D243 | en | 1 | [Sale] | how many bags of potato were sold last month and for how much | flags asks_count,unexplained,how_much_twice; doubts UnknownWords; unknown word: bags; no read planned |

## R. Wording of one figure: Urdu or Roman forms the app does not know (no new plan, a wording fix)

Plan: Words: "نام + ٹوٹل رقم", "aap hi ka hisab", Roman or misspelt names (nadeem trader, tarik mehmod), "is hafte"... see the list.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D269 | urdu | 2 | [Balance] | کیا فیصل کا بقایا 5000 ہے | flags amount_named,happened; no read planned |
| D316 | roman* | 2 | [Balance] | nadeem trader ka kitna bkaya hai | not sure; doubts CloseSpelling; no read planned |
| D320 | roman* | 2 | [] | tarik mehmod ke zime kitna he | not sure; flags unexplained; doubts CloseSpelling, NoFigure; unknown word: zime; no read planned |
| D371 | en | 2 | [Balance] | how much is Zahid Mehmood due to pay | flags happened; no read planned |
| D452 | urdu | 1 | [Sale] | کریم نے پچھلے مہینے کتنا مال بیچا | flags measure_quantity; planned period,find,analytics:B |

## P. Zero or thin-day questions with no figure word ("kitne ke liye", "how much potato did X take")

Plan: A default figure for "how much <item> did <buyer> take / kitna uthaya": the sale amount and qty of that buyer and item, 0 when none.

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D154 | roman | 2 | [] | akram baloch ne aaj aam kitne ke liye | not sure; doubts NoFigure; no read planned |
| D155 | urdu | 2 | [] | اکرم بلوچ نے آج آم کتنے کے لیے | not sure; doubts NoFigure; no read planned |
| D156 | roman* | 2 | [] | nadeem traders ne is hafte aloo kitna uthaya | not sure; flags unexplained; doubts UnknownWords, NoFigure; unknown word: uthaya; no read planned |
| D157 | en | 2 | [] | how much potato did Nadeem Traders take this week | not sure; doubts NoFigure; no read planned |

## A. Ambiguous name: the app should ask "which one" itself (2 to 5 matches)

Plan: Several matches -> compose the "کون سا؟" answer from the find result with no model call (buttons: name, tag, code, city, balance).

| id | lang | calls | figures asked | question | what the app made of it |
|---|---|---|---|---|---|
| D235 | urdu | 2 | [Balance] | اکرم کا بقایا کتنا ہے | not sure; doubts SeveralMatches; no read planned |
| D236 | roman | 2 | [Balance] | akram ka bakaya kitna hai | not sure; flags how_much_twice; doubts SeveralMatches; no read planned |
| D237 | en | 2 | [Balance] | how much does Akram owe | not sure; doubts SeveralMatches; no read planned |

