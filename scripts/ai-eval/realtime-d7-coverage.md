# D7 bank: every kind of record and figure the app keeps

Written by `scripts/ai-eval/build-bank-d7.mjs` from `bank-spec-d7.mjs`. Do not edit by hand. The bank is `realtime-d7.tsv`.
The kinds come from the PRD (Sections 6, 9 to 22, the report catalogue R1 to R38, the analytics views A to K) and the tables of Architecture 15. Every kind has question ids, or a reason why it has none.

| # | Kind of record or figure | Where it comes from | Questions (ids) |
|---|---|---|---|
| 1 | Sellers: sold, bills, net, commission taken from them | PRD 10, 12, R6, R27, view B; accounts, bikri_lots | D022-D025, D033-D038, D042-D044 |
| 2 | A seller on one day: items, qty, rate, amount | PRD 12, view A with a seller; bikri_lots, bikri_sales | D030-D032 |
| 3 | Seller bill deductions: commission, freight, labour, munshiana, phone, cash | PRD 12.5, C6, R6; bikri_lots | D039-D041 |
| 4 | Seller balance and payables (what the shop owes) | PRD 15.3, FR-7.59, R24; journal_lines | D026-D027, D045-D047, D318-D319, D342-D343 |
| 5 | Seller advances: given, adjusted, open | PRD 15.4, FR-15.8, R26; advance_allocations | D028-D029, D354-D355 |
| 6 | Buyers: bought, by item, by day, with commission and laga | PRD 12.4, view C, R5, R35; bikri_sales | D001-D006, D010-D011, D014-D016, D165-D167 |
| 7 | A buyer on one day: cash on the spot, credit left, balance (the day sheet) | PRD FR-19.12, R35; bikri_sales, journal_lines | D001-D009 |
| 8 | Buyer balance, oldest open bill, days, aging buckets | PRD 15.2, FR-15.4, C13, C14, R20; bill_allocations | D012-D013, D085-D089, D316-D317, D320-D321 |
| 9 | Receipts from a buyer (وصولی) and how many | PRD 15.2, R23; vouchers | D017-D018, D101-D104, D338-D339 |
| 10 | Payments to sellers (ادائیگی) and how many | PRD 15.3, R25; vouchers | D101-D104 |
| 11 | Cash and credit sales (نقدی، ادھاری), cash against credit | PRD 12.4, R3, R4; bikri_sales.payment_type | D001-D006, D148-D153, D158-D159, D165-D167 |
| 12 | The dead amount (ڈوبی رقم): who, owed then, still owed | PRD FR-10.10, R38; dead_marks | D019-D021, D090-D091 |
| 13 | پختہ کھاتہ persons and the shared accounts | PRD 13, R9; accounts, bikri_sales.pukhta_* | D186-D187 (The rich year has one typed one-time person (Aamir) and one own پختہ account (Nadir), both with a balance.) |
| 14 | Collectors and the collection list (اگراہی) | PRD 15.6, R22; staff, accounts | D087-D089, D195-D197 |
| 15 | Sale entries (بکری): number, date, truck, count | PRD 12.2, 12.7; bikri_entries | D037-D038, D118-D121, D192-D194 |
| 16 | Lots: goods that came in, by item and period | PRD 12.3; bikri_lots | D128-D129 |
| 17 | Sale lines: item, qty, rate, amount, commission, laga, total | PRD 12.4, C1 to C5; bikri_sales | D001-D003, D014-D016, D030-D036, D130-D131, D140-D141 |
| 18 | Quality of a sale line (اول، دوم، سوم) | PRD FR-12.42; bikri_sales.grade | D134-D136, D145-D147, D162-D164 |
| 19 | Trucks (ٹرک): times, last day, gross, net | PRD 12.2, FR-7.49; bikri_entries | D192-D194 |
| 20 | Commission: seller side against buyer side, laga | PRD 11, 12.4; bikri_sales | D024-D025, D033-D034, D058-D059, D130-D131, D140-D141 |
| 21 | Commission and charge rules (what wins) | PRD 11.1 to 11.3; charge_rules | D188-D191 |
| 22 | Items and item groups (phal, sabzi) | PRD 10.3, 18, views A, D, H; items | D124-D125, D130-D133, D142-D144, D160-D161, D168-D170, D322-D325 |
| 23 | Rates: last, average, highest, lowest | PRD FR-7.53, C18, view A; bikri_sales | D014-D016, D030-D032, D035-D036, D134-D139, D326-D327 |
| 24 | Stock (باقی مال): by item, by group, waiting days, lots | PRD 12.8, R10; bikri_lots, stock_moves | D122-D127, D340-D341 |
| 25 | Cash in hand and the cash book (R12) | PRD 16.1, R12; journal_lines on 01010001 | D094-D100, D336-D337 |
| 26 | Day book (روزنامچہ) and journal lines | PRD 6, R11; journal_entries, journal_lines | D105-D107 |
| 27 | Trial balance, opening trial, account index | PRD R16, R18, R19; accounts, journal_lines | D108-D115, D348-D349 |
| 28 | Day summary of today: sales, bills, commission (the tiles, R34) | PRD 9.1, R34; dashboard tiles | D120-D121, D350-D353 (R34 reads the same books as these tiles; its cash lines are the cash book (T44, T45).) |
| 29 | Dues tiles: buyers, sellers, total dues (اگراہی) | PRD 9.1, FR-19.9, R20, R24; dashboard tiles | D045-D047, D081-D084, D342-D343 |
| 30 | Open advances tile | PRD 9.1, FR-15.8; advance_allocations | D028-D029 |
| 31 | Day state: closed up to, open days; attention list | PRD 6.3, FR-6.6, FR-7.44, 21; day_state | D092-D093, D116-D117, D126-D127 |
| 32 | Audit log of an entry | PRD FR-21.3; audit_log | D118-D119 |
| 33 | Partner shares in percent | PRD 17.1; partner_shares | D054-D055 |
| 34 | Partner profit share: income share, expense share, profit | PRD 17.2, 17.3, C12, R30, view F; partner_share_rows | D048-D049, D058-D059 |
| 35 | Partner drawings taken and returned | PRD 16.2, 16.3, C20, R29; vouchers, journal_lines | D050-D051, D060-D062, D065-D066 |
| 36 | Partner capital in and out; partner total (کل رقم) | PRD 17.4, 17.9, C22, R31; journal_lines, partners | D052-D055, D328-D331 |
| 37 | Partner gardens, trips and fruit taken | PRD 12.10, 12.11, 16.7, view F; ventures, fruit_takes | D056-D057 |
| 38 | Who took the most (a ranking over the partners) | PRD 17.3; partner_share_rows | D060-D062 |
| 39 | Shop expenses by head and period; expense heads | PRD 16.2, R29; expense_heads, vouchers | D063-D073, D198-D199, D332-D333 |
| 40 | Money lent (ادھار رقم): lent, returned, open, return-by | PRD 16.5, FR-16.4, R32; vouchers, journal_lines | D074-D077, D334-D335 |
| 41 | الگ کھاتہ (separate khata) | PRD 16.6; accounts of control 0305 | D078-D080 (The rich year has the Family Khata with three entries (seed.rs d7_richer_cases).) |
| 42 | Market trips and their purchases (stock, cost, result) | PRD 12.10, R33, view J; ventures, purchases | D171-D173 |
| 43 | Gardens: rent, installments, due, balance | PRD 12.11, R36, view I; ventures, venture_entries | D174-D176, D344-D345 |
| 44 | Fruit taken (پھل): by partner, for guests, from other shops | PRD 16.7, R37, view K; fruit_takes | D177-D185, D346-D347 |
| 45 | Staff (munshi, collector, labour) | PRD 10.3; staff | D195-D197 |
| 46 | Packings, stores, marks, expense heads | PRD 10.3; packings, stores, markas, expense_heads | D198-D199 |
| 47 | The newest sale line and the last rate by the lead rule (own date, then بکری number, then line number; the seller rate) | plan 59 Section 11, R1, S-A2; bikri_sales, bikri_entries | D137-D139, D200-D208, D212-D217 (The rich year has grapes only in the last three days: a later sale of an older بکری, an edit and a day of several lines, built so the row id gives another rate (seed.rs d7_last_rate_cases).) |
| 48 | A day with several lines, an edited بکری and a later sale: every line counts once | PRD 12.9, FR-12.17, FR-12.19; bikri_sales.is_active | D209-D211, D226-D228 |
| 49 | Commission taken from a seller on his item (the seller side, never both sides added) | plan 59 R2, R10; bikri_sales | D218-D220 |
| 50 | A بکری by its number (the whole bill) | plan 59 R3; bikri_entries.number | D221-D228 |
| 51 | A voucher by its number (receipt, payment) | plan 59 R7; vouchers.number | D244-D249 |
| 52 | Quality: the last rate, and every quality side by side | plan 59 R4, R11; bikri_sales.grade | D229-D234 |
| 53 | The same name in two accounts: ask which one, never guess | PRD FR-7.77, E82; plan 59 R5 | D235-D237 |
| 54 | An item in one packing (potatoes in boxes are 0) | plan 59 R6; bikri_lots.packing_id | D238-D243 |
| 55 | Cash or credit word on a sale, never the whole day | plan 59 R8; bikri_sales.payment_type | D250-D252 |
| 56 | Uncommon words for a period (مئی, septmber, parso, ab tak) | plan 59 R9, R13b | D253-D264, D271-D272 |
| 57 | An expense head with a period | plan 59 R12; R29 | D265-D267 |
| 58 | A number the Owner types is never the answer | plan 59 R13 | D268-D270 |
| 59 | A day with cash AND credit lines of one buyer: on the spot, credit left, balance | PRD FR-19.12, R35; bikri_sales.payment_type, journal_lines | D273-D284 |
| 60 | The one-time person of the shared پختہ کھاتہ and a پختہ party account, with a balance | PRD 13, FR-13.5, FR-13.11, R9, R20; accounts.is_pukhta_party, bikri_sales.pukhta_* | D186-D187, D285-D289 |
| 61 | A separate khata (الگ کھاتہ) with entries: paid out, received, the balance | PRD 16.6, FR-16.6; vouchers khata_out and khata_in | D078-D080, D290-D291 |
| 62 | A city on accounts: the dues of its buyers (R21), how many owe | PRD 10.3, FR-15.6, R21; cities, accounts.city_id | D292-D300 |
| 63 | A sale line with ریٹ مختلف: seller rate, buyer rate, the rate difference | PRD FR-12.3, C19, FR-7.53; bikri_sales.buyer_rate_paisa, rate_diff_paisa | D301-D309 |
| 64 | A later sale of an older بکری on a day with a newer بکری of the same item | PRD FR-12.17; plan 59 Section 11 | D310-D315 |
| 65 | Zero: nothing happened (a day, a filter, an item) is 0, never "not found" | PRD 23 (E24 family), plan 59 Section 1 | D042-D044, D067-D068, D098-D102, D154-D161, D186-D187 |
| 66 | Cities and اڈے (truck stands) | PRD 10.3; cities, addas | none: The rich year gives no figure for them (no buyer has a city, one adda row). The team bank R8 questions cover the list. |
| 67 | Settings, firm details, user preferences, translit pairs, backup log, the question log | PRD 21, 20, 7.84; settings, user_prefs, translit_pairs, backup_log, assistant_questions | none: Not figures of the books. The assistant answers them with help or a screen button (category D of the team bank), not with a real-time figure. |
| 68 | Month close, drafts, edit windows, bill and advance allocation rows themselves | PRD 17.6, 7.9, 7.70; month_closes, drafts, edit_windows | none: Working tables. Their figures are asked through the aging (T05, T38), the advance (T13) and the day state (T52). |

## Common short questions (one figure asked), by area

| Area | Groups | Questions (ids) |
|---|---|---|
| advance | 1 | D426-D427 |
| audit | 1 | D118-D119 |
| buyer | 15 | D253-D258, D268-D270, D276-D281, D316-D317, D320-D321, D356-D371 |
| cash | 6 | D094-D095, D336-D339, D438-D439, D446-D449 |
| dues | 7 | D081-D086, D292-D297, D342-D343, D442-D443 |
| expense | 8 | D063-D064, D067-D071, D265-D267, D332-D333, D420-D425 |
| fruit | 2 | D346-D347, D436-D437 |
| garden | 3 | D344-D345, D430-D433 |
| item | 9 | D140-D141, D218-D220, D238-D240, D322-D325, D400-D407 |
| khata | 1 | D290-D291 |
| lent | 2 | D334-D335, D428-D429 |
| partner | 6 | D052-D053, D328-D331, D414-D419 |
| period | 3 | D259-D264, D271-D272 |
| pukhta | 1 | D288-D289 |
| quality | 1 | D229-D231 |
| rate | 2 | D326-D327, D408-D409 |
| rules | 2 | D188-D191 |
| sale | 1 | D224-D225 |
| seller | 13 | D024-D027, D318-D319, D354-D355, D374-D387, D452-D455 |
| stock | 3 | D340-D341, D410-D413 |
| summary | 9 | D350-D353, D388-D399, D444-D445 |
| trial | 3 | D114-D115, D348-D349, D440-D441 |
| trip | 1 | D434-D435 |
| zero | 3 | D250-D252, D372-D373, D450-D451 |

221 of 455 questions (49%) ask one figure in a plain wording (mix=short); the rest ask several figures or several filters at once (mix=picky). Run one kind with `--mix short` or `--mix picky`.

## Size of the bank

455 questions in 193 groups.

By language: Roman Urdu 156, Urdu letters 145, English 154. Misspelt on purpose: 55 (12%).

| Area (section) | Questions | Roman Urdu | Urdu | English | Misspelt |
|---|---|---|---|---|---|
| advance | 2 | 1 | 0 | 1 | 0 |
| audit | 2 | 1 | 1 | 0 | 0 |
| buyer | 65 | 23 | 20 | 22 | 14 |
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
| master | 5 | 1 | 2 | 2 | 0 |
| partner | 25 | 10 | 6 | 9 | 7 |
| period | 8 | 3 | 3 | 2 | 2 |
| pukhta | 7 | 2 | 3 | 2 | 0 |
| quality | 15 | 5 | 5 | 5 | 1 |
| rate | 31 | 10 | 11 | 10 | 0 |
| rules | 4 | 2 | 1 | 1 | 0 |
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
| T84 | pukhta | person | D186-D187 | پختہ کھاتہ میں اب تک کتنی بکری ہوئی اور کتنا باقی ہے |
| T85 | rules | rule | D188-D189 | rahim ke kele par hum kitna kamishan lete hain |
| T86 | rules | laga | D190-D191 | seb par gahak se laga kitna lagta hai |
| T87 | truck | last | D192-D194 | LES-1057 truck kitni baar aya, akhri baar kab, khaam bikri aur khalis raqam kya thi |
| T88 | master | staff | D195-D197 | dukaan me kitne munshi, kitne ugrahi wale aur kul kitne mulazim hain |
| T89 | master | lists | D198-D199 | کتنی خرچہ مدیں، کتنی پیکنگ، کتنے سٹور اور کتنے مارکے بنے ہوئے ہیں |
| N01 | rate | last | D200-D202 | aaj angoor ka aakhri rate kya laga |
| N02 | rate | last | D203-D205 | shahid raza ko aaj angoor ka aakhri rate kya diya |
| N03 | rate | range | D206-D208 | is hafte angoor ka sab se mehnga aur sab se sasta rate kya raha |
| N04 | item | day | D209-D211 | aaj angoor kitne lines me, kitna maal aur kitne ka bika |
| N05 | seller | complex | D212-D214 | karim ka aaj angoor kitna bika, kitne ka aur akhri rate kya tha |
| N06 | rate | last | D215-D217 | is hafte aloo ka aakhri rate kya tha |
| N07 | item | commission | D218-D220 | rahim ke kele ka pichle mahine kitna kamishan kata |
| N08 | sale | number | D221-D223 | bikri number 100 me kitna maal bika, kitne ka, kitne lines aur kis tarikh ko |
| N09 | sale | number | D224-D225 | bikri number 100 ka total commission kitna tha |
| N10 | sale | number | D226-D228 | bikri number 466 ki kul bikri, qty aur lines batao |
| N11 | quality | last | D229-D231 | pichle mahine awwal darje ke seb ka aakhri rate kya tha |
| N12 | quality | complex | D232-D234 | pichle mahine seb awwal, doyam aur soyam darje ka alag alag kitna bika |
| N13 | buyer | which | D235-D237 | اکرم کا بقایا کتنا ہے |
| N14 | item | packing | D238-D240 | pichle mahine aloo ki peti me kitni bikri hui |
| N15 | item | packing | D241-D243 | pichle mahine aloo ki boriyan kitni bikin aur kitne ki |
| N16 | voucher | number | D244-D246 | wasooli number 100 kitne ki thi, kis ki aur kab |
| N17 | voucher | number | D247-D249 | adaigi number 50 kitne ki thi, kis ko aur kab |
| N18 | zero | cash | D250-D252 | aaj udhaar me kitni bikri hui |
| N19 | buyer | period | D253-D255 | اکرم ٹول ٹیکس کا مئی کا کمیشن |
| N20 | buyer | period | D256-D258 | faisal se septmber me kitna kamishan liya gaya |
| N21 | period | day | D259-D261 | parso ki bikri kitni thi |
| N22 | period | all | D262-D264 | ab tak ki kul bikri kitni hui |
| N23 | expense | head | D265-D267 | is saal chai ka kharcha kitna hua |
| N24 | buyer | trap | D268-D270 | kya faisal ka bakaya 5000 hai |
| N25 | period | ambiguous | D271-D272 | kal commission kitna hai |
| N26 | buyer | day | D273-D275 | پچھلے مہینے 14 تاریخ کو نوید ٹریڈرز نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار رہا |
| N27 | buyer | cash | D276-D278 | naveed traders ne 14 october ko nakad kitna diya |
| N28 | buyer | credit | D279-D281 | naveed traders ka 14 october ka udhar kitna raha |
| N29 | buyer | cash | D282-D284 | naveed traders ne 14 october ko nakad me kela kitna, kitne ka aur kis rate par liya |
| N30 | pukhta | person | D285-D287 | pukhta khata ke aamir ne kitna maal liya, kitne ka aur ab kitna baqi hai |
| N31 | pukhta | short | D288-D289 | nadir ka bakaya kitna hai |
| N32 | khata | short | D290-D291 | خاندانی کھاتے میں کتنا ہے |
| N33 | dues | city | D292-D294 | kasur ke gahakon ke zimme kul kitna baqi hai |
| N34 | dues | city | D295-D297 | kasur shehar ke kitne gahakon ka bakaya hai |
| N35 | dues | city | D298-D300 | shehar kasur ka bakaya aur usme se 8 se 30 din purana kitna hai |
| N36 | rate | last | D301-D303 | 22 october ko seb ka aakhri rate kya tha |
| N37 | rate | buyer | D304-D306 | rafique brothers ne 22 october ko seb gahak ke rate par kitne ka liya, kis rate par aur kitna |
| N38 | rate | difference | D307-D309 | jis line me gahak ka rate alag tha uska farq kitna bana, bayopari ka rate aur gahak ka rate kya tha |
| N39 | rate | last | D310-D312 | 19 october ko moosami ka aakhri rate kya laga |
| N40 | item | day | D313-D315 | 19 october ko moosami kitne lines me, kitna aur kitne ka bika, sab se mehnga aur sab se sasta rate |
| S01 | buyer | simple | D316-D317 | nadeem trader ka kitna bkaya hai |
| S02 | seller | simple | D318-D319 | بشیر کو کتنی رقم دینی ہے |
| S03 | buyer | simple | D320-D321 | tarik mehmod ke zime kitna he |
| S04 | item | simple | D322-D323 | pichle mahine kela kitne ka bika |
| S05 | item | simple | D324-D325 | is saal aam kitni qty bika |
| S06 | rate | simple | D326-D327 | اس مہینے کیلے کا اوسط ریٹ کیا ہے |
| S07 | partner | simple | D328-D329 | alif ki kul raqam kitni hai |
| S08 | partner | simple | D330-D331 | عثمان نے کتنا سرمایہ لگایا ہے |
| S09 | expense | simple | D332-D333 | ab tk chae par kitna kharcha hua |
| S10 | lent | simple | D334-D335 | ادھار رقم میں کل کتنی واپس آنی ہے |
| S11 | cash | simple | D336-D337 | kal rat ko roknad kitni thi |
| S12 | cash | simple | D338-D339 | پچھلے مہینے کل کتنی وصولی ہوئی |
| S13 | stock | simple | D340-D341 | kele ka kitna mal baki hai |
| S14 | dues | simple | D342-D343 | بیوپاریوں کو کل کتنی ادائیگی باقی ہے |
| S15 | garden | simple | D344-D345 | shahpur bag ka abhi hisab kitna he |
| S16 | fruit | simple | D346-D347 | alif ke liye phal ki kul qeemat kitni bani |
| S17 | trial | simple | D348-D349 | ٹرائل بیلنس میں فرق کتنا ہے |
| S18 | summary | simple | D350-D351 | aj kamishan kitna bna |
| S19 | summary | simple | D352-D353 | اس مہینے اب تک کل کتنی بکری ہوئی |
| S20 | seller | simple | D354-D355 | tahir ki peshgi abhi kitni khuli he |
| S21 | buyer | short | D356-D357 | akram baloch ka bakaya kitna hai |
| S22 | buyer | short | D358-D359 | ارشد ملتانی کا بقایا کتنا ہے |
| S23 | buyer | short | D360-D361 | danish khan ke zimme kitna hai |
| S24 | buyer | short | D362-D363 | imran froot ka hisab kya hai |
| S25 | buyer | short | D364-D365 | کاشف علی کے ذمے کتنے ہیں |
| S26 | buyer | short | D366-D367 | usman ghani ka baqaya batao |
| S27 | buyer | short | D368-D369 | وقاص برادرز کا بقایا کتنا ہے |
| S28 | buyer | short | D370-D371 | zahid mehmood ka udhaar kitna hai |
| S29 | zero | short | D372-D373 | شاہد رضا کے ذمے کتنا ہے |
| S30 | seller | short | D374-D375 | karim ko abhi kitne paise dene hain |
| S31 | seller | short | D376-D377 | naseer ka hisab kya he |
| S32 | seller | short | D378-D379 | جمیل کو کتنے دینے ہیں |
| S33 | seller | short | D380-D381 | saleem ko kitna dena hai |
| S34 | seller | short | D382-D383 | غلام رسول کو کتنی رقم دینی ہے |
| S35 | seller | short | D384-D385 | haji nazeer ko kitna dena hai |
| S36 | seller | short | D386-D387 | sarfaraz ka baqaya kitna he |
| S37 | summary | short | D388-D389 | kal ki bikri kitni thi |
| S38 | summary | short | D390-D391 | اس ہفتے کی کل بکری کتنی ہے |
| S39 | summary | short | D392-D393 | pichle mahine ki kul bikri kitni rahi |
| S40 | summary | short | D394-D395 | اس سال کی کل بکری کتنی ہوئی |
| S41 | summary | short | D396-D397 | kal ka kamishan kitna bana tha |
| S42 | summary | short | D398-D399 | پچھلے مہینے کا کل کمیشن کتنا بنا |
| S43 | item | short | D400-D401 | pichle mahine seb kitne ka bika |
| S44 | item | short | D402-D403 | is saal moosami kitni qty bik gayi |
| S45 | item | short | D404-D405 | اس مہینے آلو کتنے کے بکے |
| S46 | item | short | D406-D407 | pichle mahine tamatar kitne nag bika |
| S47 | rate | short | D408-D409 | پچھلے مہینے سیب کا اوسط ریٹ کیا رہا |
| S48 | stock | short | D410-D411 | aam ka kitna maal bacha hai |
| S49 | stock | short | D412-D413 | سیب کا کتنا مال باقی ہے |
| S50 | partner | short | D414-D415 | b partner ki kul raqam kitni hai |
| S51 | partner | short | D416-D417 | عثمان کا اس سال کا حصہ کتنا بنا |
| S52 | partner | short | D418-D419 | b ka is mahine ka hisa kitna bana |
| S53 | expense | short | D420-D421 | ab tak dukaan ke kiraye par kitna kharcha hua |
| S54 | expense | short | D422-D423 | اب تک تنخواہوں پر کتنا خرچ ہوا |
| S55 | expense | short | D424-D425 | is saal dukaan ka kul kharcha kitna hua |
| S56 | advance | short | D426-D427 | bayopariyon ko ab tak kul kitni peshgi di |
| S57 | lent | short | D428-D429 | ادھار رقم میں سے کتنی کی تاریخ گزر چکی |
| S58 | garden | short | D430-D431 | shahpur baagh ka thekah abhi kitna baqi hai |
| S59 | garden | short | D432-D433 | شاہ پور باغ کی کل کتنی قسطیں دی گئیں |
| S60 | trip | short | D434-D435 | chalu safar ka ab tak nafa nuqsan kitna hai |
| S61 | fruit | short | D436-D437 | اب تک پھل کی کل قیمت کتنی بنی |
| S62 | cash | short | D438-D439 | is mahine ke akhir me roknad kitni hai |
| S63 | trial | short | D440-D441 | ٹرائل بیلنس کا بنام کل کتنا ہے |
| S64 | dues | short | D442-D443 | aath se tees din purane bakaye kitne hain |
| S65 | summary | short | D444-D445 | aaj kitne bill bane |
| S66 | cash | short | D446-D447 | کل روکڑ میں سے کتنا گیا |
| S67 | cash | short | D448-D449 | pichle mahine bayopariyon ko kul kitna diya |
| S68 | zero | short | D450-D451 | is hafte gahakon se kitni wasooli hui |
| S69 | seller | short | D452-D453 | کریم نے پچھلے مہینے کتنا مال بیچا |
| S70 | seller | short | D454-D455 | bashir ki pichle mahine ki khalis raqam kitni bani |

