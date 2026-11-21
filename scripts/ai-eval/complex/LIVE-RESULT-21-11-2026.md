# Live check of the complex questions (plan 82, lane C), 21-11-2026

Books: dev copy (today 20-11-2026) and the rich year (today 05-11-2026). The app as it is (instant answers ON, warm-up ON, reasoning low), a fresh copy of the books per question, the real model. Figures are whole rupees from `oracle.py` (SQL over posted rows). `log id` is the id in the question log of the throwaway copy of that question (46 on the dev copy, 1 on the year books; 47 and 48 for follow-ups); the full row (tools, calls, answer, tokens) is in `results-{dev,year}-live.jsonl` under the question id.

| id | question | expected | outcome | path | calls | secs | figures | party | verdict (1st run) | after one retry |
|---|---|---|---|---|---|---|---|---|---|---|
| D01 | mani, haroon aur ans ne abhi tak shop ko kitni commissi... | mani=2,640; haroon=11,070; ans=10,140 | answered | local | 0 | 0.6 | right | right | RIGHT |  |
| D02 | mani ne pichle mahine kitni khareedari ki aur kitni com... | 0 (honest zero) | answered | local | 0 | 0.1 | right | right | ZERO-OK |  |
| D03 | is mahine har gahaq ne alag alag kitni bikri li aur kit... | mani.value=36,000; mani.comm=2,640; haroon.value=172,000; haroon.comm=11,070; ans.value=149,000; ans.comm=1... | time_limit | model | 0 | 40.4 | - | right | FAIL | RIGHT |
| D04 | is mahine sab se zyada commission kis gahaq ne di ? | haroon=11,070 | answered | model | 1 | 19.3 | right | right | RIGHT |  |
| D05 | faisal, ehsan aur adnan beopariyon ki abhi tak ki bikri... | faisal=71,000; ehsan=232,000; adnan=79,000 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| D06 | ehsan ke maal se shop ko abhi tak kitni commission mili ? | comm_on_goods=28,870 | answered | model | 1 | 4.1 | right | right | RIGHT |  |
| D07 | is hafte ki bikri aur pichle hafte ki bikri ka farq batao | this_week=200,000; diff=200,000 | answered | model | 2 | 12.8 | right | right | RIGHT |  |
| D08 | pichle 7 din mein shop ko kitni commission mili ? | commission=26,400 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| D09 | 6 november aur 16 november ki commission alag alag batao | 6 nov=22,070; 16 nov=26,400 | answered | model | 1 | 5.1 | right | right | RIGHT |  |
| D10 | is saal mahine ke hisab se kitni commission bani ? | 2026-11=48,470 | answered | model | 2 | 8.3 | right | right | RIGHT |  |
| D11 | pump wala bagh mein abhi tak kitna paisa laga aur partn... | paid=200,000 | locked | model | 3 | 18.6 | right | right | LOCKED-OK |  |
| D12 | bagh ka hisab batao: kitna paisa diya, kitni bikri aayi... | paid=200,000 | time_limit | model | 1 | 45.1 | - | right | FAIL | RIGHT |
| D13 | pehle safar ka nateeja kya raha, kitna nafa ya nuksan ? | 0 (honest zero) | error | model | 0 | 45.1 | - | right | FAIL | ASKED |
| D14 | pichle saal se is mahine tak shop ne kharche par kitna ... | 0 (honest zero) | answered | local | 0 | 0.1 | right | right | ZERO-OK |  |
| D15 | bijli ka kharcha is mahine kitna hua aur pichle mahine ... | 0 (honest zero) | answered | model | 1 | 14.4 | right | right | ZERO-OK |  |
| D16 | is mahine shop ka nafa kitna hai ? | profit=48,470 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| D17 | mani ka kitna bakaya hai aur is mahine usne kitna maal ... | balance=47,280; purchase=36,000 | answered | model | 2 | 33.2 | right | right | RIGHT |  |
| D18 | kin gahakon ke zimme sab se zyada paisa hai, top 3 batao | haroon=173,070; ans=74,140; mani=47,280 | answered | model | 1 | 6.5 | right | right | RIGHT |  |
| D19 | ans ne pichle 3 mahine mein kitna maal liya aur kitni c... | purchase=149,000; comm=10,140 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| D20 | مانی نے اب تک دکان کو کتنا کمیشن دیا ہے؟ | comm=2,640 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| D21 | ہارون اور انس نے اس مہینے کتنی خریداری کی اور کتنا کمیش... | haroon.value=172,000; haroon.comm=11,070; ans.value=149,000; ans.comm=10,140 | error | model | 1 | 44.8 | - | right | FAIL | RIGHT |
| D22 | How much commission did the shop earn from Haroon and A... | together=21,210 | error | model | 0 | 40.4 | - | right | FAIL | RIGHT |
| D23 | Which seller sent the most goods this month and how muc... | comm=28,870 | answered | model | 2 | 30.3 | right | right | RIGHT |  |
| D24 | Compare this month's commission with last month and tel... | this=48,470; last=0; diff=48,470 | answered | model | 5 | 19.5 | right | right | RIGHT |  |
| D25 | What share of the shop's total commission came from Man... | pct=5.45 | locked | model | 1 | 2.5 | - | right | ASKED |  |
| D26a | faisal ki abhi tak ki bikri kitni hai ? | faisal=71,000 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| D26b | aur ehsan ki ? | ehsan=232,000 | answered | model | 4 | 13.4 | right | right | RIGHT |  |
| D26c | dono ki mila kar kitni hui ? | sum=303,000 | answered | model | 5 | 16.5 | right | right | RIGHT |  |
| D27 | mani aur ans se is mahine aur pichle mahine kitni commi... | this_month=12,780 | answered | model | 1 | 3.5 | right | right | RIGHT |  |
| D28 | chukta khate walon ne is mahine kitna maal khareeda ? | purchase=25,000 | answered | model | 1 | 3.6 | right | right | RIGHT |  |
| D29 | 6 november ko ehsan ne kitna maal bheja aur kis kis gah... | ehsan.gross=182,000; mani=20,000; ans=45,000; haroon=117,000 | answered | model | 3 | 11.4 | WRONG/missing | WRONG | PARTIAL |  |
| D30 | malta ki abhi tak ki bikri aur commission kitni hai ? | gross=50,000; comm=6,800 | answered | model | 1 | 3.7 | right | right | RIGHT |  |
| D31 | ehsan ko abhi tak kitna paisa dena hai aur ans se kitna... | ehsan_payable=120,780; ans_due=74,140 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| D32 | pichle saal se is mahine tak shop ki bikri, commission ... | sale=382,000; commission=48,470 | answered | model | 1 | 3.3 | right | right | RIGHT |  |
| M01 | abhi tak ki safi bikri kitni hai ? | net_sale=332,780 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M02 | abhi tak ki khaam bikri kitni hui ? | gross_sale=382,000 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M03 | abhi tak gahakon ko kul kitne ke bill bane (commission ... | buyer_bills=407,750 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M04 | اب تک کی صافی بکری کتنی ہے؟ | net_sale=332,780 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M05 | اب تک کی خام بکری کتنی ہوئی؟ | gross_sale=382,000 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M06 | What is the net sale so far? | net_sale=332,780 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M07 | What is the total gross sale so far? | gross_sale=382,000 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M08 | What is the total of all buyer bills so far? | buyer_bills=407,750 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M09 | ehsan ki safi bikri kitni hai ? | net_sale=200,780 | answered | model | 1 | 3.5 | right | right | RIGHT |  |
| M10 | ehsan ki khaam bikri kitni hai ? | gross_sale=232,000 | answered | model | 1 | 3.7 | right | right | RIGHT |  |
| M11 | mani ke bill ki kul raqam kitni hai (commission aur lag... | buyer_bill=38,640 | answered | model | 1 | 4.5 | right | right | RIGHT |  |
| M12 | mani ne kitne ke maal ki khareedari ki (commission aur ... | purchase_value=36,000 | answered | model | 1 | 3.9 | right | right | RIGHT |  |
| M13 | mani ne commission aur laga alag alag kitna diya ? | commission=2,160; laga=480 | answered | model | 1 | 3.1 | right | right | RIGHT |  |
| M14 | shop ne abhi tak sirf laga kitna kamaya ? | laga=4,000 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M15 | How much commission, not counting laga, has the shop ea... | commission_without_laga=44,470 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M16 | ehsan ne shop ko kitni commission di aur shop ne uske m... | given_by_seller=13,720; earned_on_goods=28,870 | answered | model | 2 | 5.5 | right | right | RIGHT |  |
| M21 | احسان کی صافی بکری کتنی ہے؟ | net_sale=200,780 | answered | model | 1 | 2.6 | right | right | RIGHT |  |
| M22 | How much commission, not counting laga, did the shop ea... | commission_without_laga=9,150 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M23 | How much commission did Ehsan give the shop, and how mu... | given_by_seller=13,720; earned_on_goods=28,870 | answered | model | 2 | 6.2 | right | right | RIGHT |  |
| M24 | ہارون کے بلوں کی کل رقم کتنی ہے؟ | buyer_bill=183,070 | answered | model | 1 | 2.9 | right | right | RIGHT |  |
| M25 | gahakon se hum ne abhi tak kitna commission aur kitna l... | buyer_commission=21,750; laga=4,000 | answered | model | 2 | 6.8 | right | right | RIGHT |  |
| Y01 | pichle saal se is mahine tak shop ne kharche par kitna ... | total=2,093,500 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| Y02 | pichle 6 mahine ka kharcha head ke hisab se alag alag b... | shop rent=240,000; electricity=151,000; salaries=542,000; tea=60,900; guests food and drinks=38,000 | answered | model | 3 | 17.9 | right | right | RIGHT |  |
| Y03 | bijli ka kharcha mahine ke hisab se pichle saal se ab t... | 2025-11=24,000; 2025-12=20,000; 2026-01=45,000; 2026-02=20,000; 2026-03=34,000; 2026-04=45,000; 2026-05=26,... | answered | model | 1 | 7.5 | WRONG/missing | right | PARTIAL |  |
| Y04 | How much did the shop spend on salaries and tea last mo... | salaries_last=90,000; tea_last=10,700; salaries_this=90,000; tea_this=2,600 | answered | model | 5 | 27.0 | right | right | RIGHT |  |
| Y05 | pichle mahine kul commission kitni bani aur is mahine a... | last=1,117,002; this=181,892 | answered | model | 3 | 10.6 | right | right | RIGHT |  |
| Y06 | pichle 3 mahine mein sab se zyada commission dene wale ... | Rizwan Butt=148,843; Tariq Mehmood=132,813; Kashif Ali=128,314 | answered | model | 1 | 3.9 | right | right | RIGHT |  |
| Y07 | pichle saal se is mahine tak sab se zyada bikri karne w... | Sarfraz=11,635,380; Haji Nazeer=11,349,390; Rahim=10,859,310; Saleem=10,186,530; Karim=10,179,170 | answered | model | 1 | 5.3 | right | right | RIGHT |  |
| Y08 | rahim aur karim ne is saal shop ko kitni commission di ... | rahim.sale=9,683,650; rahim.comm=581,015; karim.sale=7,330,660; karim.comm=439,841 | answered | model | 3 | 13.6 | right | right | RIGHT |  |
| Y09 | akram baloch ne pichle 3 mahine mein kitna maal liya au... | purchase=1,773,150; comm=109,980 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| Y10 | akram ne is mahine kitni commission di ? | ask which one | which_one | local | 0 | 0.1 | right | right | ASKED-OK |  |
| Y11 | faisal gahaq ne is saal mahine ke hisab se kitna maal l... | 2026-01=509,380; 2026-02=538,590; 2026-03=476,270; 2026-04=530,030; 2026-05=344,470; 2026-06=776,470; 2026-... | answered | model | 3 | 15.3 | WRONG/missing | right | PARTIAL |  |
| Y12 | pichle mahine aur is mahine ki bikri ka farq batao | last=9,154,800; this=1,490,500; diff=7,664,300 | answered | model | 2 | 7.0 | right | right | RIGHT |  |
| Y13 | pehle safar ka nateeja kya raha, kitna nafa ya nuksan h... | result=1,690 | not_found | model | 3 | 22.0 | - | right | ASKED |  |
| Y14 | pehle safar mein kitna paisa laga, kitna wapas aya aur ... | paid=287,000; money_in=288,690; result=1,690 | answered | model | 5 | 24.7 | right | right | RIGHT |  |
| Y15 | pichle saal se is mahine tak shop ka nafa nuksan kitna ... | profit=9,617,660 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| Y16 | pichle mahine ka nafa batao aur har partner ka hissa ki... | profit=948,102 | answered | model | 2 | 10.3 | right | right | RIGHT |  |
| Y17 | is saal partnaron ne zaati kharche mein kitna kitna pai... | partner a=250,000; partner b=75,000; usman=16,000 | answered | model | 2 | 8.3 | right | right | RIGHT |  |
| Y18 | is mahine aur pichle mahine ka kharcha compare karo | last=170,600; this=132,600; diff=38,000 | answered | model | 3 | 13.1 | WRONG/missing | WRONG | WRONG-DIRECTION |  |
| Y19 | agar commission isi raftar se chalta raha to agle 3 mah... | per_month=958,313; three_months=2,874,939 | estimate | model | 2 | 12.5 | right | right | RIGHT |  |
| Y20 | rahim ne kele ki bikri pichle 6 mahine mein kitni ki ? | sale=2,319,980 | answered | model | 1 | 7.8 | right | right | RIGHT |  |
| Y21a | rahim ki is saal ki bikri kitni hai ? | rahim=9,683,650 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| Y21b | aur karim ki ? | karim=7,330,660 | answered | model | 4 | 14.6 | right | right | RIGHT |  |
| Y21c | dono ki commission mila kar kitni hai ? | sum=2,080,228 | answered | model | 5 | 17.9 | right | right | RIGHT |  |
| Y22 | How much commission did the shop earn in each month fro... | 2026-05=900,722; 2026-06=1,218,672; 2026-07=1,056,324; 2026-08=877,780; 2026-09=880,156; 2026-10=1,117,002 | answered | model | 2 | 8.6 | right | right | RIGHT |  |
| Y23 | پچھلے مہینے دکان کا خرچہ مدوں کے حساب سے کتنا ہوا؟ | shop rent=40,000; electricity=23,000; salaries=90,000; tea=10,700; guests food and drinks=6,900 | answered | model | 1 | 4.1 | right | right | RIGHT |  |
| Y24 | رحیم نے اس سال دکان کو کتنا کمیشن دیا؟ | comm=581,015 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| Y25 | 10 october se 20 october ke beech shop ki bikri aur com... | sale=3,320,730; commission=405,410 | answered | model | 1 | 4.3 | right | right | RIGHT |  |
| Y26 | pichle saal se is mahine tak kis mahine mein sab se zya... | best_month_commission=1,218,672 | answered | model | 3 | 9.5 | right | right | RIGHT |  |
| Y27 | doosre safar mein abhi tak kitna maal khareeda aur kitn... | paid=117,500; bought_qty=50; sold_qty=20 | not_found | model | 3 | 12.0 | - | right | ASKED |  |
| Y28 | pichle mahine commission bikri walon se kitni li aur kh... | from_sellers=549,287; from_buyers=567,715 | checks_failed | model | 3 | 21.8 | - | right | FAIL |  |
| Y29 | is saal shop ki commission, kharcha aur nafa teeno batao | commission=9,752,880; expense=1,886,100; profit=7,876,720 | answered | model | 1 | 8.3 | right | right | RIGHT |  |
| Y30 | How much did Karim sell last month and how many boxes w... | sale=1,162,160; boxes=377 | answered | model | 1 | 4.1 | right | right | RIGHT |  |
| Y31a | How much did the shop spend on electricity this year? | electricity=295,000 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| Y31b | and on tea? | tea=98,000 | answered | model | 3 | 10.6 | right | right | RIGHT |  |
| Y31c | what is the total of both? | sum=393,000 | answered | model | 4 | 15.0 | WRONG/missing | right | WRONG |  |
| Y32 | shahpur bagh ka hisab batao: kitna paisa lagaya, kitni ... | paid=370,000; balance=1,737,824 | answered | model | 1 | 5.0 | right | right | RIGHT |  |
| Y33 | is saal sab se zyada commission dene wale 5 gahaq kaun ... | Akram Baloch=392,332; Tariq Mehmood=392,027; Rizwan Butt=384,849; Kashif Ali=376,336; Nadeem Traders=373,370 | answered | model | 1 | 4.7 | right | right | RIGHT |  |
| Y34 | rahim ki bikri is saal pichle saal ke muqable mein kitn... | this_year=9,683,650; last_year=1,175,660 | answered | model | 2 | 9.1 | right | right | RIGHT |  |
| Y35 | faisal gahaq ne pichle 3 mahine mein kitni commission d... | purchase=1,952,280; comm=121,552 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M17 | rahim ki safi bikri abhi tak kitni hai ? | net_sale=9,916,553 | answered | local | 0 | 0.1 | right | right | RIGHT |  |
| M18 | pichle mahine ki khaam bikri aur gahakon ke kul bill al... | gross_sale=9,154,800; buyer_bills=9,723,115 | answered | model | 1 | 4.6 | WRONG/missing | right | WRONG-COLUMN |  |
| M19 | اس سال کمیشن اور لاگا الگ الگ بتاؤ | commission_without_laga=9,592,152; laga=160,728 | answered | model | 5 | 22.5 | right | right | RIGHT |  |
| M20 | What is the total of all buyer bills last month, with c... | buyer_bills=9,723,115 | answered | local | 0 | 0.1 | right | right | RIGHT |  |

## Summary (first run, 98 questions; the six failed ones asked again once)

See the message to the lead and `report-live-first.md` / `report-live-merged.md` in `backend/target/ai-eval-complex/` for the findings, p50/p95, calls and cost.
