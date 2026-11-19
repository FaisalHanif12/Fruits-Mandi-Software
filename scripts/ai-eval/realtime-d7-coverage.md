# D7 bank: every kind of record and figure the app keeps

Written by `scripts/ai-eval/build-bank-d7.mjs` from `bank-spec-d7.mjs`. Do not edit by hand. The bank is `realtime-d7.tsv`.
The kinds come from the PRD (Sections 6, 9 to 22, the report catalogue R1 to R38, the analytics views A to K) and the tables of Architecture 15. Every kind has question ids, or a reason why it has none.

| # | Kind of record or figure | Where it comes from | Questions (ids) |
|---|---|---|---|
| 1 | Sellers: sold, bills, net, commission taken from them | PRD 10, 12, R6, R27, view B; accounts, bikri_lots | D022-D025, D033-D038, D042-D044 |
| 2 | A seller on one day: items, qty, rate, amount | PRD 12, view A with a seller; bikri_lots, bikri_sales | D030-D032 |
| 3 | Seller bill deductions: commission, freight, labour, munshiana, phone, cash | PRD 12.5, C6, R6; bikri_lots | D039-D041 |
| 4 | Seller balance and payables (what the shop owes) | PRD 15.3, FR-7.59, R24; journal_lines | D026-D027, D045-D047, D313-D314, D337-D338 |
| 5 | Seller advances: given, adjusted, open | PRD 15.4, FR-15.8, R26; advance_allocations | D028-D029, D349-D350 |
| 6 | Buyers: bought, by item, by day, with commission and laga | PRD 12.4, view C, R5, R35; bikri_sales | D001-D006, D010-D011, D014-D016, D165-D167 |
| 7 | A buyer on one day: cash on the spot, credit left, balance (the day sheet) | PRD FR-19.12, R35; bikri_sales, journal_lines | D001-D009 |
| 8 | Buyer balance, oldest open bill, days, aging buckets | PRD 15.2, FR-15.4, C13, C14, R20; bill_allocations | D012-D013, D085-D089, D311-D312, D315-D316 |
| 9 | Receipts from a buyer (وصولی) and how many | PRD 15.2, R23; vouchers | D017-D018, D101-D104, D333-D334 |
| 10 | Payments to sellers (ادائیگی) and how many | PRD 15.3, R25; vouchers | D101-D104 |
| 11 | Cash and credit sales (نقدی، ادھاری), cash against credit | PRD 12.4, R3, R4; bikri_sales.payment_type | D001-D006, D148-D153, D158-D159, D165-D167 |
| 12 | The dead amount (ڈوبی رقم): who, owed then, still owed | PRD FR-10.10, R38; dead_marks | D019-D021, D090-D091 |
| 13 | Collectors and the collection list (اگراہی) | PRD 15.6, R22; staff, accounts | D087-D089, D193-D195 |
| 14 | Sale entries (بکری): number, date, truck, count | PRD 12.2, 12.7; bikri_entries | D037-D038, D118-D121, D190-D192 |
| 15 | Lots: goods that came in, by item and period | PRD 12.3; bikri_lots | D128-D129 |
| 16 | Sale lines: item, qty, rate, amount, commission, laga, total | PRD 12.4, C1 to C5; bikri_sales | D001-D003, D014-D016, D030-D036, D130-D131, D140-D141 |
| 17 | Quality of a sale line (اول، دوم، سوم) | PRD FR-12.42; bikri_sales.grade | D134-D136, D145-D147, D162-D164 |
| 18 | Trucks (ٹرک): times, last day, gross, net | PRD 12.2, FR-7.49; bikri_entries | D190-D192 |
| 19 | Commission: seller side against buyer side, laga | PRD 11, 12.4; bikri_sales | D024-D025, D033-D034, D058-D059, D130-D131, D140-D141 |
| 20 | Commission and charge rules (what wins) | PRD 11.1 to 11.3; charge_rules | D186-D189 |
| 21 | Items and item groups (phal, sabzi) | PRD 10.3, 18, views A, D, H; items | D124-D125, D130-D133, D142-D144, D160-D161, D168-D170, D317-D320 |
| 22 | Rates: last, average, highest, lowest | PRD FR-7.53, C18, view A; bikri_sales | D014-D016, D030-D032, D035-D036, D134-D139, D321-D322 |
| 23 | Stock (باقی مال): by item, by group, waiting days, lots | PRD 12.8, R10; bikri_lots, stock_moves | D122-D127, D335-D336 |
| 24 | Cash in hand and the cash book (R12) | PRD 16.1, R12; journal_lines on 01010001 | D094-D100, D331-D332 |
| 25 | Day book (روزنامچہ) and journal lines | PRD 6, R11; journal_entries, journal_lines | D105-D107 |
| 26 | Trial balance, opening trial, account index | PRD R16, R18, R19; accounts, journal_lines | D108-D115, D343-D344 |
| 27 | Day summary of today: sales, bills, commission (the tiles, R34) | PRD 9.1, R34; dashboard tiles | D120-D121, D345-D348 (R34 reads the same books as these tiles; its cash lines are the cash book (T44, T45).) |
| 28 | Dues tiles: buyers, sellers, total dues (اگراہی) | PRD 9.1, FR-19.9, R20, R24; dashboard tiles | D045-D047, D081-D084, D337-D338 |
| 29 | Open advances tile | PRD 9.1, FR-15.8; advance_allocations | D028-D029 |
| 30 | Day state: closed up to, open days; attention list | PRD 6.3, FR-6.6, FR-7.44, 21; day_state | D092-D093, D116-D117, D126-D127 |
| 31 | Audit log of an entry | PRD FR-21.3; audit_log | D118-D119 |
| 32 | Partner shares in percent | PRD 17.1; partner_shares | D054-D055 |
| 33 | Partner profit share: income share, expense share, profit | PRD 17.2, 17.3, C12, R30, view F; partner_share_rows | D048-D049, D058-D059 |
| 34 | Partner drawings taken and returned | PRD 16.2, 16.3, C20, R29; vouchers, journal_lines | D050-D051, D060-D062, D065-D066 |
| 35 | Partner capital in and out; partner total (کل رقم) | PRD 17.4, 17.9, C22, R31; journal_lines, partners | D052-D055, D323-D326 |
| 36 | Partner gardens, trips and fruit taken | PRD 12.10, 12.11, 16.7, view F; ventures, fruit_takes | D056-D057 |
| 37 | Who took the most (a ranking over the partners) | PRD 17.3; partner_share_rows | D060-D062 |
| 38 | Shop expenses by head and period; expense heads | PRD 16.2, R29; expense_heads, vouchers | D063-D073, D196-D197, D327-D328 |
| 39 | Money lent (ادھار رقم): lent, returned, open, return-by | PRD 16.5, FR-16.4, R32; vouchers, journal_lines | D074-D077, D329-D330 |
| 40 | الگ کھاتہ (separate khata) | PRD 16.6; accounts of control 0305 | D078-D080 (The rich year has the Family Khata with three entries (seed.rs d7_richer_cases).) |
| 41 | Market trips and their purchases (stock, cost, result) | PRD 12.10, R33, view J; ventures, purchases | D171-D173 |
| 42 | Gardens: rent, installments, due, balance | PRD 12.11, R36, view I; ventures, venture_entries | D174-D176, D339-D340 |
| 43 | Fruit taken (پھل): by partner, for guests, from other shops | PRD 16.7, R37, view K; fruit_takes | D177-D185, D341-D342 |
| 44 | Staff (munshi, collector, labour) | PRD 10.3; staff | D193-D195 |
| 45 | Packings, stores, marks, expense heads | PRD 10.3; packings, stores, markas, expense_heads | D196-D197 |
| 46 | The newest sale line and the last rate by the lead rule (own date, then بکری number, then line number; the seller rate) | plan 59 Section 11, R1, S-A2; bikri_sales, bikri_entries | D137-D139, D198-D206, D210-D215 (The rich year has grapes only in the last three days: a later sale of an older بکری, an edit and a day of several lines, built so the row id gives another rate (seed.rs d7_last_rate_cases).) |
| 47 | A day with several lines, an edited بکری and a later sale: every line counts once | PRD 12.9, FR-12.17, FR-12.19; bikri_sales.is_active | D207-D209, D224-D226 |
| 48 | Commission taken from a seller on his item (the seller side, never both sides added) | plan 59 R2, R10; bikri_sales | D216-D218 |
| 49 | A بکری by its number (the whole bill) | plan 59 R3; bikri_entries.number | D219-D226 |
| 50 | A voucher by its number (receipt, payment) | plan 59 R7; vouchers.number | D242-D247 |
| 51 | Quality: the last rate, and every quality side by side | plan 59 R4, R11; bikri_sales.grade | D227-D232 |
| 52 | The same name in two accounts: ask which one, never guess | PRD FR-7.77, E82; plan 59 R5 | D233-D235 |
| 53 | An item in one packing (potatoes in boxes are 0) | plan 59 R6; bikri_lots.packing_id | D236-D241 |
| 54 | Cash or credit word on a sale, never the whole day | plan 59 R8; bikri_sales.payment_type | D248-D250 |
| 55 | Uncommon words for a period (مئی, septmber, parso, ab tak) | plan 59 R9, R13b | D251-D262, D269-D270 |
| 56 | An expense head with a period | plan 59 R12; R29 | D263-D265 |
| 57 | A number the Owner types is never the answer | plan 59 R13 | D266-D268 |
| 58 | A day with cash AND credit lines of one buyer: on the spot, credit left, balance | PRD FR-19.12, R35; bikri_sales.payment_type, journal_lines | D271-D282 |
| 59 | A buyer with his own account and a balance (Nadir) | PRD FR-10.5, R20; accounts | D283-D284 |
| 60 | A separate khata (الگ کھاتہ) with entries: paid out, received, the balance | PRD 16.6, FR-16.6; vouchers khata_out and khata_in | D078-D080, D285-D286 |
| 61 | A city on accounts: the dues of its buyers (R21), how many owe | PRD 10.3, FR-15.6, R21; cities, accounts.city_id | D287-D295 |
| 62 | A sale line with ریٹ مختلف: seller rate, buyer rate, the rate difference | PRD FR-12.3, C19, FR-7.53; bikri_sales.buyer_rate_paisa, rate_diff_paisa | D296-D304 |
| 63 | A later sale of an older بکری on a day with a newer بکری of the same item | PRD FR-12.17; plan 59 Section 11 | D305-D310 |
| 64 | Zero: nothing happened (a day, a filter, an item) is 0, never "not found" | PRD 23 (E24 family), plan 59 Section 1 | D042-D044, D067-D068, D098-D102, D154-D161 |
| 65 | Cities and اڈے (truck stands) | PRD 10.3; cities, addas | none: The rich year gives no figure for them (no buyer has a city, one adda row). The team bank R8 questions cover the list. |
| 66 | Settings, firm details, user preferences, translit pairs, backup log, the question log | PRD 21, 20, 7.84; settings, user_prefs, translit_pairs, backup_log, assistant_questions | none: Not figures of the books. The assistant answers them with help or a screen button (category D of the team bank), not with a real-time figure. |
| 67 | Month close, drafts, edit windows, bill and advance allocation rows themselves | PRD 17.6, 7.9, 7.70; month_closes, drafts, edit_windows | none: Working tables. Their figures are asked through the aging (T05, T38), the advance (T13) and the day state (T52). |

## Common short questions (one figure asked), by area

| Area | Groups | Questions (ids) |
|---|---|---|
| advance | 1 | D421-D422 |
| audit | 1 | D118-D119 |
| buyer | 16 | D251-D256, D266-D268, D274-D279, D283-D284, D311-D312, D315-D316, D351-D366 |
| cash | 6 | D094-D095, D331-D334, D433-D434, D441-D444 |
| dues | 7 | D081-D086, D287-D292, D337-D338, D437-D438 |
| expense | 8 | D063-D064, D067-D071, D263-D265, D327-D328, D415-D420 |
| fruit | 2 | D341-D342, D431-D432 |
| garden | 3 | D339-D340, D425-D428 |
| item | 9 | D140-D141, D216-D218, D236-D238, D317-D320, D395-D402 |
| khata | 1 | D285-D286 |
| lent | 2 | D329-D330, D423-D424 |
| partner | 6 | D052-D053, D323-D326, D409-D414 |
| period | 3 | D257-D262, D269-D270 |
| quality | 1 | D227-D229 |
| rate | 2 | D321-D322, D403-D404 |
| rules | 2 | D186-D189 |
| sale | 1 | D222-D223 |
| seller | 13 | D024-D027, D313-D314, D349-D350, D369-D382, D447-D450 |
| stock | 3 | D335-D336, D405-D408 |
| summary | 9 | D345-D348, D383-D394, D439-D440 |
| trial | 3 | D114-D115, D343-D344, D435-D436 |
| trip | 1 | D429-D430 |
| zero | 3 | D248-D250, D367-D368, D445-D446 |

221 of 450 questions (49%) ask one figure in a plain wording (mix=short); the rest ask several figures or several filters at once (mix=picky). Run one kind with `--mix short` or `--mix picky`.

## Size of the bank

450 questions in 191 groups.

By language: Roman Urdu 155, Urdu letters 143, English 152. Misspelt on purpose: 55 (12%).

| Area (section) | Questions | Roman Urdu | Urdu | English | Misspelt |
|---|---|---|---|---|---|
| advance | 2 | 1 | 0 | 1 | 0 |
| audit | 2 | 1 | 1 | 0 | 0 |
| buyer | 67 | 24 | 21 | 22 | 14 |
| cash | 21 | 9 | 5 | 7 | 5 |
| daybook | 3 | 1 | 1 | 1 | 0 |
| days | 2 | 1 | 0 | 1 | 0 |
| dues | 26 | 7 | 9 | 10 | 3 |
| expense | 22 | 7 | 7 | 8 | 3 |
| fruit | 13 | 4 | 4 | 5 | 2 |
| garden | 9 | 3 | 3 | 3 | 1 |
| group | 5 | 1 | 2 | 2 | 0 |
| item | 43 | 16 | 13 | 14 | 1 |
| khata | 5 | 1 | 2 | 2 | 0 |
| lent | 8 | 2 | 3 | 3 | 1 |
| master | 5 | 2 | 2 | 1 | 0 |
| partner | 25 | 10 | 6 | 9 | 7 |
| period | 8 | 3 | 3 | 2 | 2 |
| quality | 15 | 5 | 5 | 5 | 1 |
| rate | 31 | 10 | 11 | 10 | 0 |
| rules | 4 | 1 | 1 | 2 | 0 |
| sale | 8 | 3 | 3 | 2 | 0 |
| seller | 51 | 17 | 17 | 17 | 10 |
| stock | 14 | 6 | 3 | 5 | 1 |
| summary | 20 | 7 | 6 | 7 | 2 |
| trial | 12 | 2 | 5 | 5 | 1 |
| trip | 5 | 2 | 2 | 1 | 0 |
| truck | 3 | 1 | 1 | 1 | 0 |
| voucher | 6 | 2 | 2 | 2 | 0 |
| zero | 15 | 6 | 5 | 4 | 1 |

Questions with three or more filters at once: 58.

## Groups

| Group | Area | Kind | Questions | What it asks (first wording) |
|---|---|---|---|---|
| T01 | buyer | day | D001-D003 | پچھلے مہینے 14 تاریخ کو اکرم نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار رہا |
| T02 | buyer | day | D004-D006 | رضوان بٹ نے پچھلے مہینے کی 14 کو کیا کیا لیا، کتنا نقد دیا اور ادھار کتنا بچا |
| T03 | buyer | day | D007-D009 | faisal ki aaj ki parchi batao: pehle kitna baqi tha, aaj ka bill kitna bana aur ab kul kitna ho gaya |
| T04 | buyer | item | D010-D011 | فیصل نے اس سال کیلا کتنی تعداد میں اور کتنے روپے کا خریدا |
| T05 | buyer | dues | D012-D013 | faisal ka sab se purana bil kis tareekh ka he aur kitne din ho gye |
| T07 | buyer | complex | D014-D016 | pichle mahine faisal ne rahim ka kitna maal liya, kis rate par aur kitne ka |
| T08 | buyer | paid | D017-D018 | akram toll tax ne pichle mahine kitne paise diye aur kitni baar |
| T09 | buyer | dead | D019-D021 | bilal ahmed ki raqam doobi raqam me kab gayi aur ab bhi kitna dena hai |
| T10 | seller | month | D022-D023 | rahim ne pichle mahine kitna mal becha, kitne bil bane aur khalis rakam kitni bani |
| T11 | seller | commission | D024-D025 | rahim se pichle mahine kitna kamishan kata |
| T12 | seller | payable | D026-D027 | rahim ka hisab kya hai, hum ne usko kitna dena hai |
| T13 | seller | advance | D028-D029 | طاہر کو کل کتنی پیشگی دی گئی، کتنی کٹ چکی اور کتنی ابھی کھلی ہے |
| T14 | seller | day | D030-D032 | 14 oct ko basheer ka kya kya bika, kitni qty, kis rate pr aur kitne ka |
| T15 | seller | complex | D033-D034 | رحیم كا كیلا اس سال كتنا بكا، كتنے كا، اور اس پر كمیشن كتنا لگا |
| T16 | seller | complex | D035-D036 | کریم کا سیب پچھلے مہینے کتنا بکا، سب سے مہنگا اور سب سے سستا ریٹ کیا تھا |
| T17 | seller | bill | D037-D038 | karim ka sab se naya bill kis tarikh ka hai, uska number, khaam bikri aur khalis raqam kya hai |
| T18 | seller | deductions | D039-D041 | rahim ke is saal ke bills me commission, kiraya, mazdoori aur munshiana kitna kata, kul katautiyan aur net kitna |
| T19 | seller | zero | D042-D044 | sarfraz ka angoor is saal kitna bika |
| T20 | seller | payable | D045-D047 | kitne bayopariyon ko abhi paise dene hain aur kul kitne |
| T21 | partner | share | D048-D049 | alif ka is mahine ka hisa kitna bna aur is sal ka kitna |
| T22 | partner | drawings | D050-D051 | alif ne ab tak kitne paise liye aur kitne wapas kiye |
| T23 | partner | capital | D052-D053 | b partner ne dukaan me kitna sarmaya lagaya hua hai |
| T24 | partner | total | D054-D055 | usman shirakat daar ka hissa kitne feesad hai aur uski kul raqam kitni hai |
| T25 | partner | ventures | D056-D057 | alif ko is sal bagh, safr aur phal me kitna laga ya mila |
| T26 | partner | profit | D058-D059 | الف کی اس سال کی آمدن کا حصہ، خرچے کا حصہ اور منافع کتنا ہے |
| T27 | partner | rank | D060-D062 | kis shirakat daar ne sab se zyada paise liye aur kitne |
| T28 | expense | head | D063-D064 | اب تک بجلی کے بل پر کل کتنا خرچ ہوا |
| T29 | expense | period | D065-D066 | pichle mahine dukan ka kharcha kitna hua aur sharkat daron ne kitna liya |
| T30 | expense | zero | D067-D068 | aaj dukaan ka koi kharcha hua |
| T31 | expense | sum | D069-D071 | dukaan ke kiraye aur bijli ka kharcha mila kar ab tak kitna hua |
| T32 | expense | rank | D072-D073 | اس سال سب سے زیادہ خرچہ کس مد میں ہوا اور کتنا |
| T33 | lent | person | D074-D075 | aslam ko kitna udhaar diya tha, kitna cash wapas aya, kitna tankhwah se kata aur kitna abhi baqi hai |
| T34 | lent | overdue | D076-D077 | aslam ne paise wapas karne ki tarikh kya di thi aur kitne din guzar gaye |
| T35 | khata | khata | D078-D080 | khandani khata me abhi kitna paisa para hai, kitna diya aur kitna wapas aya |
| T36 | dues | total | D081-D082 | گاہكوں كے ذمے كل كتنا باقي ہے |
| T37 | dues | total | D083-D084 | كل اگراہی كتنی بنتی ہے |
| T38 | dues | aging | D085-D086 | 91 din se purane bakaye kitne hain |
| T39 | dues | collector | D087-D089 | rasheed ke hisse me kitni wasooli hai, usme se 31 se 90 din purani kitni |
| T40 | dues | dead | D090-D091 | ڈوبی رقم کل کتنی ہے اور کتنے لوگوں کی |
| T41 | dues | attention | D092-D093 | kitne gahakon ka bakaya 30 din se purana ho gaya aur kitna |
| T43 | cash | now | D094-D095 | galle me abhi kitna cash hai |
| T44 | cash | book | D096-D097 | is mahine ki roknad ki kitaab: shuru me kitna tha, kitna aya, kitna gya aur ab kitna he |
| T45 | cash | day | D098-D100 | kal galle me kitna aya, kitna gaya aur raat ko kitna bacha |
| T46 | cash | zero | D101-D102 | aaj gahakon se kitni wasooli hui aur bayopariyon ko kitni adaigi hui |
| T47 | cash | period | D103-D104 | pichle mahine kitni wasooli hui, kitni rasidein bani aur bayopariyon ko kitna diya, kitni baar |
| T48 | daybook | today | D105-D107 | aaj ke rozname me kitni entries hain aur banaam ka total kitna hai |
| T49 | trial | balance | D108-D110 | trial balance ka banaam aur jama kitna hai aur farq kitna hai |
| T50 | trial | opening | D111-D113 | shuru ke trial balance me kitna banaam aur jama tha |
| T51 | trial | index | D114-D115 | کل کتنے کھاتے بنے ہوئے ہیں |
| T52 | days | state | D116-D117 | din kis tarikh tak band ho chuka hai aur kitne din abhi khule hain |
| T90 | audit | count | D118-D119 | bikri number 1 ka record kitni baar likha gaya |
| T53 | summary | today | D120-D121 | aj ki kul bikri, kitne bil bane aur kamishan kitna hoa |
| T55 | stock | item | D122-D123 | moosami ka kitna maal bacha hua hai aur kitne lot me |
| T56 | stock | group | D124-D125 | sabziyon ka kul kitna maal baqi hai aur kitne lot |
| T57 | stock | waiting | D126-D127 | سب سے پرانا مال کتنے دن سے پڑا ہے اور کتنے لاٹ دو دن سے زیادہ رکے ہوئے ہیں |
| T59 | stock | received | D128-D129 | pichle mahine kele ka kitna maal aya aur kitne lot me |
| T61 | item | month | D130-D131 | pichle mahine kele ki bikri, qty aur kul kamishan kitna raha |
| T62 | group | year | D132-D133 | اس سال پھل کی بکری کتنی ہوئی اور سبزی کی کتنی |
| T63 | quality | complex | D134-D136 | pichle mahine awwal darje ka seb kitna bika, kitne ka aur ausat rate kya raha |
| T64 | rate | last | D137-D139 | is hafte seb ka aakhri rate kya laga |
| T65 | item | commission | D140-D141 | pichle mahine faisal se kele par kitna commission liya |
| T66 | item | compare | D142-D144 | is mahine kele aur seb me se kis ki bikri zyada hui aur farq kitna hai |
| T67 | quality | complex | D145-D147 | usman ghani ne is saal awwal darje ka seb kitni baar liya, kitna aur kitne ka |
| T68 | item | cash | D148-D150 | is saal kele ki naqdi bikri kitni hui, kitne lines, kitni qty aur kitne ki |
| T69 | group | credit | D151-D153 | pichle mahine phal ki udhaar bikri kitni rahi, kitni qty aur kitne gahakon ko |
| T70 | zero | complex | D154-D155 | akram baloch ne aaj aam kitne ke liye |
| T71 | zero | complex | D156-D157 | nadeem traders ne is hafte aloo kitna uthaya |
| T72 | zero | cash | D158-D159 | aaj nakad bikri kitni hui aur udhaar kitni |
| T73 | zero | group | D160-D161 | aaj sabziyon ki bikri kitni hui |
| T74 | quality | complex | D162-D164 | akram baloch ne pichle mahine doyam darje ka kela kitni baar, kitna aur kitne ka liya |
| T75 | item | complex | D165-D167 | shahid raza ne september me kele ka kitna maal aur kitne ka liya, commission mila kar kitna diya |
| T76 | item | best | D168-D170 | is saal kele ka sab se acha mahina konsa raha aur us mahine kitni bikri hui |
| T77 | trip | open | D171-D173 | chalu safar me kitna maal kharida, kitna bika, kitna bacha aur ab tak nafa nuqsan kitna hai |
| T79 | garden | rent | D174-D176 | shahpur baagh ka thekah kitna tha, kitni qiston me di aur kitna baqi hai |
| T81 | fruit | total | D177-D179 | ab tak phal kitna liya gaya aur uski qeemat kitni bani |
| T82 | fruit | partner | D180-D182 | alif ne ghar ke liye kitna phal liya aur uski kul qeemat kitni |
| T83 | fruit | guest | D183-D185 | haji nazir ki dukaan se mehmano ke liye kitna phal liya, kis rate par, kitne ka aur kis tarikh ko |
| T85 | rules | rule | D186-D187 | رحیم کے کیلے پر ہم کتنے فیصد کمیشن لیتے ہیں |
| T86 | rules | laga | D188-D189 | seb par gahak se laga kitna lagta hai |
| T87 | truck | last | D190-D192 | LES-1057 truck kitni baar aya, akhri baar kab, khaam bikri aur khalis raqam kya thi |
| T88 | master | staff | D193-D195 | dukaan me kitne munshi, kitne ugrahi wale aur kul kitne mulazim hain |
| T89 | master | lists | D196-D197 | kitni kharcha madein, kitni packing, kitne store aur kitne marke bane hue hain |
| N01 | rate | last | D198-D200 | aaj angoor ka aakhri rate kya laga |
| N02 | rate | last | D201-D203 | shahid raza ko aaj angoor ka aakhri rate kya diya |
| N03 | rate | range | D204-D206 | is hafte angoor ka sab se mehnga aur sab se sasta rate kya raha |
| N04 | item | day | D207-D209 | aaj angoor kitne lines me, kitna maal aur kitne ka bika |
| N05 | seller | complex | D210-D212 | karim ka aaj angoor kitna bika, kitne ka aur akhri rate kya tha |
| N06 | rate | last | D213-D215 | is hafte aloo ka aakhri rate kya tha |
| N07 | item | commission | D216-D218 | rahim ke kele ka pichle mahine kitna kamishan kata |
| N08 | sale | number | D219-D221 | bikri number 100 me kitna maal bika, kitne ka, kitne lines aur kis tarikh ko |
| N09 | sale | number | D222-D223 | bikri number 100 ka total commission kitna tha |
| N10 | sale | number | D224-D226 | bikri number 466 ki kul bikri, qty aur lines batao |
| N11 | quality | last | D227-D229 | pichle mahine awwal darje ke seb ka aakhri rate kya tha |
| N12 | quality | complex | D230-D232 | pichle mahine seb awwal, doyam aur soyam darje ka alag alag kitna bika |
| N13 | buyer | which | D233-D235 | اکرم کا بقایا کتنا ہے |
| N14 | item | packing | D236-D238 | pichle mahine aloo ki peti me kitni bikri hui |
| N15 | item | packing | D239-D241 | pichle mahine aloo ki boriyan kitni bikin aur kitne ki |
| N16 | voucher | number | D242-D244 | wasooli number 100 kitne ki thi, kis ki aur kab |
| N17 | voucher | number | D245-D247 | adaigi number 50 kitne ki thi, kis ko aur kab |
| N18 | zero | cash | D248-D250 | aaj udhaar me kitni bikri hui |
| N19 | buyer | period | D251-D253 | اکرم ٹول ٹیکس کا مئی کا کمیشن |
| N20 | buyer | period | D254-D256 | faisal se septmber me kitna kamishan liya gaya |
| N21 | period | day | D257-D259 | parso ki bikri kitni thi |
| N22 | period | all | D260-D262 | ab tak ki kul bikri kitni hui |
| N23 | expense | head | D263-D265 | is saal chai ka kharcha kitna hua |
| N24 | buyer | trap | D266-D268 | kya faisal ka bakaya 5000 hai |
| N25 | period | ambiguous | D269-D270 | kal commission kitna hai |
| N26 | buyer | day | D271-D273 | پچھلے مہینے 14 تاریخ کو نوید ٹریڈرز نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار رہا |
| N27 | buyer | cash | D274-D276 | naveed traders ne 14 october ko nakad kitna diya |
| N28 | buyer | credit | D277-D279 | naveed traders ka 14 october ka udhar kitna raha |
| N29 | buyer | cash | D280-D282 | naveed traders ne 14 october ko nakad me kela kitna, kitne ka aur kis rate par liya |
| N31 | buyer | short | D283-D284 | nadir ka bakaya kitna hai |
| N32 | khata | short | D285-D286 | خاندانی کھاتے میں کتنا ہے |
| N33 | dues | city | D287-D289 | kasur ke gahakon ke zimme kul kitna baqi hai |
| N34 | dues | city | D290-D292 | kasur shehar ke kitne gahakon ka bakaya hai |
| N35 | dues | city | D293-D295 | shehar kasur ka bakaya aur usme se 8 se 30 din purana kitna hai |
| N36 | rate | last | D296-D298 | 22 october ko seb ka aakhri rate kya tha |
| N37 | rate | buyer | D299-D301 | rafique brothers ne 22 october ko seb gahak ke rate par kitne ka liya, kis rate par aur kitna |
| N38 | rate | difference | D302-D304 | jis line me gahak ka rate alag tha uska farq kitna bana, bayopari ka rate aur gahak ka rate kya tha |
| N39 | rate | last | D305-D307 | 19 october ko moosami ka aakhri rate kya laga |
| N40 | item | day | D308-D310 | 19 october ko moosami kitne lines me, kitna aur kitne ka bika, sab se mehnga aur sab se sasta rate |
| S01 | buyer | simple | D311-D312 | nadeem trader ka kitna bkaya hai |
| S02 | seller | simple | D313-D314 | بشیر کو کتنی رقم دینی ہے |
| S03 | buyer | simple | D315-D316 | tarik mehmod ke zime kitna he |
| S04 | item | simple | D317-D318 | pichle mahine kela kitne ka bika |
| S05 | item | simple | D319-D320 | is saal aam kitni qty bika |
| S06 | rate | simple | D321-D322 | اس مہینے کیلے کا اوسط ریٹ کیا ہے |
| S07 | partner | simple | D323-D324 | alif ki kul raqam kitni hai |
| S08 | partner | simple | D325-D326 | عثمان نے کتنا سرمایہ لگایا ہے |
| S09 | expense | simple | D327-D328 | ab tk chae par kitna kharcha hua |
| S10 | lent | simple | D329-D330 | ادھار رقم میں کل کتنی واپس آنی ہے |
| S11 | cash | simple | D331-D332 | kal rat ko roknad kitni thi |
| S12 | cash | simple | D333-D334 | پچھلے مہینے کل کتنی وصولی ہوئی |
| S13 | stock | simple | D335-D336 | kele ka kitna mal baki hai |
| S14 | dues | simple | D337-D338 | بیوپاریوں کو کل کتنی ادائیگی باقی ہے |
| S15 | garden | simple | D339-D340 | shahpur bag ka abhi hisab kitna he |
| S16 | fruit | simple | D341-D342 | alif ke liye phal ki kul qeemat kitni bani |
| S17 | trial | simple | D343-D344 | ٹرائل بیلنس میں فرق کتنا ہے |
| S18 | summary | simple | D345-D346 | aj kamishan kitna bna |
| S19 | summary | simple | D347-D348 | اس مہینے اب تک کل کتنی بکری ہوئی |
| S20 | seller | simple | D349-D350 | tahir ki peshgi abhi kitni khuli he |
| S21 | buyer | short | D351-D352 | akram baloch ka bakaya kitna hai |
| S22 | buyer | short | D353-D354 | ارشد ملتانی کا بقایا کتنا ہے |
| S23 | buyer | short | D355-D356 | danish khan ke zimme kitna hai |
| S24 | buyer | short | D357-D358 | imran froot ka hisab kya hai |
| S25 | buyer | short | D359-D360 | کاشف علی کے ذمے کتنے ہیں |
| S26 | buyer | short | D361-D362 | usman ghani ka baqaya batao |
| S27 | buyer | short | D363-D364 | وقاص برادرز کا بقایا کتنا ہے |
| S28 | buyer | short | D365-D366 | zahid mehmood ka udhaar kitna hai |
| S29 | zero | short | D367-D368 | شاہد رضا کے ذمے کتنا ہے |
| S30 | seller | short | D369-D370 | karim ko abhi kitne paise dene hain |
| S31 | seller | short | D371-D372 | naseer ka hisab kya he |
| S32 | seller | short | D373-D374 | جمیل کو کتنے دینے ہیں |
| S33 | seller | short | D375-D376 | saleem ko kitna dena hai |
| S34 | seller | short | D377-D378 | غلام رسول کو کتنی رقم دینی ہے |
| S35 | seller | short | D379-D380 | haji nazeer ko kitna dena hai |
| S36 | seller | short | D381-D382 | sarfaraz ka baqaya kitna he |
| S37 | summary | short | D383-D384 | kal ki bikri kitni thi |
| S38 | summary | short | D385-D386 | اس ہفتے کی کل بکری کتنی ہے |
| S39 | summary | short | D387-D388 | pichle mahine ki kul bikri kitni rahi |
| S40 | summary | short | D389-D390 | اس سال کی کل بکری کتنی ہوئی |
| S41 | summary | short | D391-D392 | kal ka kamishan kitna bana tha |
| S42 | summary | short | D393-D394 | پچھلے مہینے کا کل کمیشن کتنا بنا |
| S43 | item | short | D395-D396 | pichle mahine seb kitne ka bika |
| S44 | item | short | D397-D398 | is saal moosami kitni qty bik gayi |
| S45 | item | short | D399-D400 | اس مہینے آلو کتنے کے بکے |
| S46 | item | short | D401-D402 | pichle mahine tamatar kitne nag bika |
| S47 | rate | short | D403-D404 | پچھلے مہینے سیب کا اوسط ریٹ کیا رہا |
| S48 | stock | short | D405-D406 | aam ka kitna maal bacha hai |
| S49 | stock | short | D407-D408 | سیب کا کتنا مال باقی ہے |
| S50 | partner | short | D409-D410 | b partner ki kul raqam kitni hai |
| S51 | partner | short | D411-D412 | عثمان کا اس سال کا حصہ کتنا بنا |
| S52 | partner | short | D413-D414 | b ka is mahine ka hisa kitna bana |
| S53 | expense | short | D415-D416 | ab tak dukaan ke kiraye par kitna kharcha hua |
| S54 | expense | short | D417-D418 | اب تک تنخواہوں پر کتنا خرچ ہوا |
| S55 | expense | short | D419-D420 | is saal dukaan ka kul kharcha kitna hua |
| S56 | advance | short | D421-D422 | bayopariyon ko ab tak kul kitni peshgi di |
| S57 | lent | short | D423-D424 | ادھار رقم میں سے کتنی کی تاریخ گزر چکی |
| S58 | garden | short | D425-D426 | shahpur baagh ka thekah abhi kitna baqi hai |
| S59 | garden | short | D427-D428 | شاہ پور باغ کی کل کتنی قسطیں دی گئیں |
| S60 | trip | short | D429-D430 | chalu safar ka ab tak nafa nuqsan kitna hai |
| S61 | fruit | short | D431-D432 | اب تک پھل کی کل قیمت کتنی بنی |
| S62 | cash | short | D433-D434 | is mahine ke akhir me roknad kitni hai |
| S63 | trial | short | D435-D436 | ٹرائل بیلنس کا بنام کل کتنا ہے |
| S64 | dues | short | D437-D438 | aath se tees din purane bakaye kitne hain |
| S65 | summary | short | D439-D440 | aaj kitne bill bane |
| S66 | cash | short | D441-D442 | کل روکڑ میں سے کتنا گیا |
| S67 | cash | short | D443-D444 | pichle mahine bayopariyon ko kul kitna diya |
| S68 | zero | short | D445-D446 | is hafte gahakon se kitni wasooli hui |
| S69 | seller | short | D447-D448 | کریم نے پچھلے مہینے کتنا مال بیچا |
| S70 | seller | short | D449-D450 | bashir ki pichle mahine ki khalis raqam kitni bani |

