# Complex questions with expected figures (plan 82, lane C)

Worked out by `oracle.py` (SQL over posted rows). Dev = copy of the dev database, today 20-11-2026. Year = the rich year of the robustness oracle, today 05-11-2026. Whole rupees.

## World: dev

| id | lang | question | period | expected (reading 1) | other readings |
|---|---|---|---|---|---|
| D01 | roman | mani, haroon aur ans ne abhi tak shop ko kitni commission di ? | up to now | mani=2640; haroon=11070; ans=10140 | mani=2160; haroon=9150; ans=8940 |
| D02 | roman | mani ne pichle mahine kitni khareedari ki aur kitni commission di ? | 2026-10-01 to 2026-10-31 | 0 (honest zero) |  |
| D03 | roman | is mahine har gahaq ne alag alag kitni bikri li aur kitni commission di ? | 2026-11-01 to 2026-11-30 | mani.value=36000; mani.comm=2640; haroon.value=172000; haroon.comm=11070; ans.value=149000; ans.comm=10140 | mani.value=36000; mani.comm=2160; haroon.value=172000; haroon.comm=9150; ans.value=149000; ans.comm=8940 |
| D04 | roman | is mahine sab se zyada commission kis gahaq ne di ? | 2026-11-01 to 2026-11-30 | haroon=11070 | haroon=9150 |
| D05 | roman | faisal, ehsan aur adnan beopariyon ki abhi tak ki bikri kitni hui ? | up to now | faisal=71000; ehsan=232000; adnan=79000 |  |
| D06 | roman | ehsan ke maal se shop ko abhi tak kitni commission mili ? | up to now | comm_on_goods=28870 | seller_comm=13720 |
| D07 | roman | is hafte ki bikri aur pichle hafte ki bikri ka farq batao | 2026-11-16 to 2026-11-22 vs 2026-11-09 to 2026-11-15 | this_week=200000; diff=200000 | this_week=214400 |
| D08 | roman | pichle 7 din mein shop ko kitni commission mili ? | 2026-11-13 to 2026-11-20 | commission=26400 |  |
| D09 | roman | 6 november aur 16 november ki commission alag alag batao | 06-11 and 16-11 | 6 nov=22070; 16 nov=26400 |  |
| D10 | roman | is saal mahine ke hisab se kitni commission bani ? | 2026-01-01 to 2026-12-31 | 2026-11=48470 |  |
| D11 | roman | pump wala bagh mein abhi tak kitna paisa laga aur partner ke hisab se kitna ? | up to now | paid=200000 |  |
| D12 | roman | bagh ka hisab batao: kitna paisa diya, kitni bikri aayi aur ab balance kya hai ? | up to now | paid=200000 |  |
| D13 | roman | pehle safar ka nateeja kya raha, kitna nafa ya nuksan ? | up to now | 0 (honest zero) |  |
| D14 | roman | pichle saal se is mahine tak shop ne kharche par kitna paisa lagaya ? | 2025-01-01 to 2026-11-30 | 0 (honest zero) |  |
| D15 | roman | bijli ka kharcha is mahine kitna hua aur pichle mahine kitna ? | this and last month | 0 (honest zero) |  |
| D16 | roman | is mahine shop ka nafa kitna hai ? | 2026-11-01 to 2026-11-30 | profit=48470 |  |
| D17 | roman | mani ka kitna bakaya hai aur is mahine usne kitna maal liya ? | 2026-11-01 to 2026-11-30 | balance=47280; purchase=36000 | balance=47280; purchase=38640 |
| D18 | roman | kin gahakon ke zimme sab se zyada paisa hai, top 3 batao | today | haroon=173070; ans=74140; mani=47280 |  |
| D19 | roman | ans ne pichle 3 mahine mein kitna maal liya aur kitni commission di ? | 2026-08-20 to 2026-11-20 | purchase=149000; comm=10140 | purchase=149000; comm=8940 |
| D20 | urdu | مانی نے اب تک دکان کو کتنا کمیشن دیا ہے؟ | up to now | comm=2640 | comm=2160 |
| D21 | urdu | ہارون اور انس نے اس مہینے کتنی خریداری کی اور کتنا کمیشن دیا؟ | 2026-11-01 to 2026-11-30 | haroon.value=172000; haroon.comm=11070; ans.value=149000; ans.comm=10140 | haroon.value=172000; haroon.comm=9150; ans.value=149000; ans.comm=8940 |
| D22 | english | How much commission did the shop earn from Haroon and Ans together this month? | 2026-11-01 to 2026-11-30 | together=21210 | together=18090 |
| D23 | english | Which seller sent the most goods this month and how much commission did the shop get on his goods? | 2026-11-01 to 2026-11-30 | comm=28870 | comm=13720 |
| D24 | english | Compare this month's commission with last month and tell me the difference | 2026-11-01 to 2026-11-30 vs 2026-10-01 to 2026-10-31 | this=48470; last=0; diff=48470 |  |
| D25 | english | What share of the shop's total commission came from Mani this month, in percent? | 2026-11-01 to 2026-11-30 | pct=5.45 | pct=4.46 || pct=9.49 |
| D26a | roman | faisal ki abhi tak ki bikri kitni hai ? | up to now | faisal=71000 |  |
| D26b | roman | aur ehsan ki ? | up to now | (follow-up of D26a) ehsan=232000 |  |
| D26c | roman | dono ki mila kar kitni hui ? | up to now | (follow-up of D26b) sum=303000 |  |
| D27 | roman | mani aur ans se is mahine aur pichle mahine kitni commission mili ? | 2026-11-01 to 2026-11-30 and 2026-10-01 to 2026-10-31 | this_month=12780 | this_month=11100 || mani=2640; ans=10140 || mani=2160; ans=8940 |
| D28 | roman | chukta khate walon ne is mahine kitna maal khareeda ? | 2026-11-01 to 2026-11-30 | purchase=25000 | purchase=26900 |
| D29 | roman | 6 november ko ehsan ne kitna maal bheja aur kis kis gahaq ne kitna liya ? | 2026-11-06 | ehsan.gross=182000; mani=20000; ans=45000; haroon=117000 | ehsan.gross=182000; mani.qty=20; ans.qty=50; haroon.qty=130 |
| D30 | roman | malta ki abhi tak ki bikri aur commission kitni hai ? | up to now | gross=50000; comm=6800 | gross=50000; comm=3800 |
| D31 | roman | ehsan ko abhi tak kitna paisa dena hai aur ans se kitna lena hai ? | today | ehsan_payable=120780; ans_due=74140 |  |
| D32 | roman | pichle saal se is mahine tak shop ki bikri, commission aur kharcha teeno batao | 2025-01-01 to 2026-11-30 | sale=382000; commission=48470 |  |
| M01 | roman | abhi tak ki safi bikri kitni hai ? | up to now | net_sale=332780 |  |
| M02 | roman | abhi tak ki khaam bikri kitni hui ? | up to now | gross_sale=382000 |  |
| M03 | roman | abhi tak gahakon ko kul kitne ke bill bane (commission aur laga mila kar) ? | up to now | buyer_bills=407750 |  |
| M04 | urdu | اب تک کی صافی بکری کتنی ہے؟ | up to now | net_sale=332780 |  |
| M05 | urdu | اب تک کی خام بکری کتنی ہوئی؟ | up to now | gross_sale=382000 |  |
| M06 | english | What is the net sale so far? | up to now | net_sale=332780 |  |
| M07 | english | What is the total gross sale so far? | up to now | gross_sale=382000 |  |
| M08 | english | What is the total of all buyer bills so far? | up to now | buyer_bills=407750 |  |
| M09 | roman | ehsan ki safi bikri kitni hai ? | up to now | net_sale=200780 |  |
| M10 | roman | ehsan ki khaam bikri kitni hai ? | up to now | gross_sale=232000 |  |
| M11 | roman | mani ke bill ki kul raqam kitni hai (commission aur laga samet) ? | up to now | buyer_bill=38640 |  |
| M12 | roman | mani ne kitne ke maal ki khareedari ki (commission aur laga ke baghair) ? | up to now | purchase_value=36000 |  |
| M13 | roman | mani ne commission aur laga alag alag kitna diya ? | up to now | commission=2160; laga=480 |  |
| M14 | roman | shop ne abhi tak sirf laga kitna kamaya ? | up to now | laga=4000 |  |
| M15 | english | How much commission, not counting laga, has the shop earned so far? | up to now | commission_without_laga=44470 | seller_commission=22720; buyer_commission=21750 |
| M16 | roman | ehsan ne shop ko kitni commission di aur shop ne uske maal par kul kitna kamaya ? | up to now | given_by_seller=13720; earned_on_goods=28870 |  |
| M21 | urdu | احسان کی صافی بکری کتنی ہے؟ | up to now | net_sale=200780 |  |
| M22 | english | How much commission, not counting laga, did the shop earn from Haroon so far? | up to now | commission_without_laga=9150 |  |
| M23 | english | How much commission did Ehsan give the shop, and how much did the shop earn in all on his goods? | up to now | given_by_seller=13720; earned_on_goods=28870 |  |
| M24 | urdu | ہارون کے بلوں کی کل رقم کتنی ہے؟ | up to now | buyer_bill=183070 |  |
| M25 | roman | gahakon se hum ne abhi tak kitna commission aur kitna laga alag alag liya ? | up to now | buyer_commission=21750; laga=4000 |  |

## World: year

| id | lang | question | period | expected (reading 1) | other readings |
|---|---|---|---|---|---|
| Y01 | roman | pichle saal se is mahine tak shop ne kharche par kitna paisa lagaya ? | 2025-01-01 to 2026-11-30 | total=2093500 |  |
| Y02 | roman | pichle 6 mahine ka kharcha head ke hisab se alag alag batao | 2026-05-05 to 2026-11-05 | shop rent=240000; electricity=151000; salaries=542000; tea=60900; guests food and drinks=38000 |  |
| Y03 | roman | bijli ka kharcha mahine ke hisab se pichle saal se ab tak kitna raha ? | 2025-11 to 2026-11 | 2025-11=24000; 2025-12=20000; 2026-01=45000; 2026-02=20000; 2026-03=34000; 2026-04=45000; 2026-05=26000; 2026-06=25000; 2026-07=20000; 2026-08=22000; 2026-09=35000; 2026-10=23000 |  |
| Y04 | english | How much did the shop spend on salaries and tea last month and this month so far? | 2026-10-01 to 2026-10-31 / 2026-11-01 to 2026-11-30 | salaries_last=90000; tea_last=10700; salaries_this=90000; tea_this=2600 |  |
| Y05 | roman | pichle mahine kul commission kitni bani aur is mahine ab tak kitni ? | 2026-10-01 to 2026-10-31 / 2026-11-01 to 2026-11-30 | last=1117002; this=181892 |  |
| Y06 | roman | pichle 3 mahine mein sab se zyada commission dene wale 3 gahaq kaun se hain ? | 2026-08-05 to 2026-11-05 | Rizwan Butt=148843; Tariq Mehmood=132813; Kashif Ali=128314 | Rizwan Butt=144779; Tariq Mehmood=129101; Kashif Ali=124978 |
| Y07 | roman | pichle saal se is mahine tak sab se zyada bikri karne wale 5 beopari kaun hain ? | up to now | Sarfraz=11635380; Haji Nazeer=11349390; Rahim=10859310; Saleem=10186530; Karim=10179170 |  |
| Y08 | roman | rahim aur karim ne is saal shop ko kitni commission di aur kitni bikri ki ? | 2026-01-01 to 2026-12-31 | rahim.sale=9683650; rahim.comm=581015; karim.sale=7330660; karim.comm=439841 | rahim.sale=9683650; rahim.comm=1186254; karim.sale=7330660; karim.comm=893974 |
| Y09 | roman | akram baloch ne pichle 3 mahine mein kitna maal liya aur kitni commission di ? | 2026-08-05 to 2026-11-05 | purchase=1773150; comm=109980 | purchase=1773150; comm=106388 |
| Y10 | roman | akram ne is mahine kitni commission di ? | 2026-11-01 to 2026-11-30 | ask 'which one' (Akram Toll Tax, Akram Baloch) |  |
| Y11 | roman | faisal gahaq ne is saal mahine ke hisab se kitna maal liya ? | 2026-01-01 to 2026-12-31 | 2026-01=509380; 2026-02=538590; 2026-03=476270; 2026-04=530030; 2026-05=344470; 2026-06=776470; 2026-07=432790; 2026-08=889880; 2026-09=533050; 2026-10=466350; 2026-11=185670 |  |
| Y12 | roman | pichle mahine aur is mahine ki bikri ka farq batao | 2026-10-01 to 2026-10-31 / 2026-11-01 to 2026-11-30 | last=9154800; this=1490500; diff=7664300 | diff=7664300 |
| Y13 | roman | pehle safar ka nateeja kya raha, kitna nafa ya nuksan hua ? | up to now | result=1690 |  |
| Y14 | roman | pehle safar mein kitna paisa laga, kitna wapas aya aur har partner ka hissa kitna raha ? | up to now | paid=287000; money_in=288690; result=1690 |  |
| Y15 | roman | pichle saal se is mahine tak shop ka nafa nuksan kitna raha ? | 2025-01-01 to 2026-11-30 | profit=9617660 |  |
| Y16 | roman | pichle mahine ka nafa batao aur har partner ka hissa kitna bana ? | 2026-10-01 to 2026-10-31 | profit=948102 |  |
| Y17 | roman | is saal partnaron ne zaati kharche mein kitna kitna paisa liya ? | 2026-01-01 to 2026-12-31 | partner a=250000; partner b=75000; usman=16000 | partner a=220000; partner b=75000; usman=16000 |
| Y18 | roman | is mahine aur pichle mahine ka kharcha compare karo | 2026-10-01 to 2026-10-31 / 2026-11-01 to 2026-11-30 | last=170600; this=132600; diff=38000 | last=170600; this=132600 |
| Y19 | roman | agar commission isi raftar se chalta raha to agle 3 mahine mein kitna hoga ? | Nov 2026 to Jan 2027 | per_month=958313; three_months=2874939 |  |
| Y20 | roman | rahim ne kele ki bikri pichle 6 mahine mein kitni ki ? | 2026-05-05 to 2026-11-05 | sale=2319980 |  |
| Y21a | roman | rahim ki is saal ki bikri kitni hai ? | 2026-01-01 to 2026-12-31 | rahim=9683650 |  |
| Y21b | roman | aur karim ki ? | 2026-01-01 to 2026-12-31 | (follow-up of Y21a) karim=7330660 |  |
| Y21c | roman | dono ki commission mila kar kitni hai ? | 2026-01-01 to 2026-12-31 | (follow-up of Y21b) sum=2080228 | sum=1020856 |
| Y22 | english | How much commission did the shop earn in each month from May to October 2026? | 2026-05 to 2026-10 | 2026-05=900722; 2026-06=1218672; 2026-07=1056324; 2026-08=877780; 2026-09=880156; 2026-10=1117002 |  |
| Y23 | urdu | پچھلے مہینے دکان کا خرچہ مدوں کے حساب سے کتنا ہوا؟ | 2026-10-01 to 2026-10-31 | shop rent=40000; electricity=23000; salaries=90000; tea=10700; guests food and drinks=6900 |  |
| Y24 | urdu | رحیم نے اس سال دکان کو کتنا کمیشن دیا؟ | 2026-01-01 to 2026-12-31 | comm=581015 | comm=1186254 |
| Y25 | roman | 10 october se 20 october ke beech shop ki bikri aur commission kitni rahi ? | 2026-10-10 to 2026-10-20 | sale=3320730; commission=405410 |  |
| Y26 | roman | pichle saal se is mahine tak kis mahine mein sab se zyada commission aaya ? | 2025-01-01 to 2026-11-30 | best_month_commission=1218672 |  |
| Y27 | roman | doosre safar mein abhi tak kitna maal khareeda aur kitna bik chuka ? | up to now | paid=117500; bought_qty=50; sold_qty=20 |  |
| Y28 | roman | pichle mahine commission bikri walon se kitni li aur khareedaron se kitni ? | 2026-10-01 to 2026-10-31 | from_sellers=549287; from_buyers=567715 | from_sellers=549287; from_buyers=549323 |
| Y29 | roman | is saal shop ki commission, kharcha aur nafa teeno batao | 2026-01-01 to 2026-12-31 | commission=9752880; expense=1886100; profit=7876720 |  |
| Y30 | english | How much did Karim sell last month and how many boxes was that? | 2026-10-01 to 2026-10-31 | sale=1162160; boxes=377 |  |
| Y31a | english | How much did the shop spend on electricity this year? | 2026-01-01 to 2026-12-31 | electricity=295000 |  |
| Y31b | english | and on tea? | 2026-01-01 to 2026-12-31 | (follow-up of Y31a) tea=98000 |  |
| Y31c | english | what is the total of both? | 2026-01-01 to 2026-12-31 | (follow-up of Y31b) sum=393000 |  |
| Y32 | roman | shahpur bagh ka hisab batao: kitna paisa lagaya, kitni bikri aayi aur ab balance kya hai ? | up to now | paid=370000; balance=1737824 | installments=350000; costs=20000; balance=1737824 |
| Y33 | roman | is saal sab se zyada commission dene wale 5 gahaq kaun hain aur kitni kitni ? | 2026-01-01 to 2026-12-31 | Akram Baloch=392332; Tariq Mehmood=392027; Rizwan Butt=384849; Kashif Ali=376336; Nadeem Traders=373370 | Akram Baloch=380564; Tariq Mehmood=380483; Rizwan Butt=373433; Kashif Ali=366328; Nadeem Traders=359210 |
| Y34 | roman | rahim ki bikri is saal pichle saal ke muqable mein kitni zyada ya kam hai ? | 2026-01-01 to 2026-12-31 / 2025-01-01 to 2025-12-31 | this_year=9683650; last_year=1175660 | diff=8507990 |
| Y35 | roman | faisal gahaq ne pichle 3 mahine mein kitni commission di aur kitna maal liya ? | 2026-08-05 to 2026-11-05 | purchase=1952280; comm=121552 | purchase=1952280; comm=117136 |
| M17 | roman | rahim ki safi bikri abhi tak kitni hai ? | up to now | net_sale=9916553 |  |
| M18 | roman | pichle mahine ki khaam bikri aur gahakon ke kul bill alag alag batao | 2026-10-01 to 2026-10-31 | gross_sale=9154800; buyer_bills=9723115 |  |
| M19 | urdu | اس سال کمیشن اور لاگا الگ الگ بتاؤ | 2026-01-01 to 2026-12-31 | commission_without_laga=9592152; laga=160728 |  |
| M20 | english | What is the total of all buyer bills last month, with commission and laga included? | 2026-10-01 to 2026-10-31 | buyer_bills=9723115 |  |
