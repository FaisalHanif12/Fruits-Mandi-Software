// The D7 bank (plan 59 task B1): real-time figures of the books, asked the way the Owner asks, picky on purpose.
//
// Goal of the bank: find flaws, not pass. Every question is category A (a figure, a list or a count from the books).
// Help, advice, forecasts and what-ifs are NOT here (plan 59 Section 8).
//
// - Every expected figure is an oracle fact (`@key`), never typed. The facts come from the app's own reports and
//   analytics (the older families) or from SQL on the books (the `d7.*` family of `oracle.rs`, `d7_facts`).
// - Simple and complex questions. A complex one holds several filters at once (a buyer and a seller and an item
//   and a period, cash or credit, a quality); every one of them must hold.
// - Roman Urdu, Urdu letters and English, about 15% misspelt (`mis`), a few with mixed words.
// - Zero days and zero filters ("nothing happened") must say 0, never "not found".
// - The words of a question never repeat a question of `questions.tsv`.
//
// Format as `bank-spec-a2.mjs`: `g` (group key), `section` (the area, used by the offline report), `kind`, `world`,
// `filters` (`facet=D7;area=...;buyer=...`), `figures`, `outcome`, `q` (wordings).
//
// `AREAS` at the bottom lists every kind of record and figure the app keeps, with the groups that cover it
// (`realtime-d7-coverage.md` is written from it by `build-bank-d7.mjs`).

const r = (text) => ['roman', text];
const rm = (text) => ['roman', text, 'mis'];
const u = (text) => ['urdu', text];
const um = (text) => ['urdu', text, 'mis'];
const e = (text) => ['en', text];
const em = (text) => ['en', text, 'mis'];

/** One group of the D7 bank. */
const z = (g, section, kind, filters, figures, q, over = {}) => ({
  g,
  section,
  kind,
  world: 'year',
  filters: `facet=D7;area=${section};${filters}`,
  figures,
  source: 'plan 59 B1',
  cat: 'A',
  q,
  ...over,
});

const BD = (buyer, day) => `d7.bd.${buyer}.${day}`;
const AK = BD('akram_toll_tax', 'd2026_10_14');
const RZ = BD('rizwan', 'd2026_10_14');
const FD = BD('faisal', 'today');

const ALL = [
  // ================================ buyers ======================================================================
  // Umar's own example. Two Akrams exist, so the app asks "which one"; the harness clicks Akram Toll Tax.
  // He bought apple and potato on credit that day, paid nothing on the spot, so the cash is 0.
  z(
    'T01',
    'buyer',
    'day',
    'click=1;buyer=Akram Toll Tax;day=2026-10-14',
    `apple_qty=@${AK}.apple.qty;apple_rate=@${AK}.apple.rate;apple_amount=@${AK}.apple.amount;potato_qty=@${AK}.potato.qty;potato_rate=@${AK}.potato.rate;potato_amount=@${AK}.potato.amount;cash=@${AK}.cash;credit=@${AK}.credit;balance=@${AK}.balance_end|@${AK}.balance_now`,
    [
      u('پچھلے مہینے 14 تاریخ کو اکرم نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار رہا'),
      rm(
        'pichle mahiney 14 tareekh ko akrm ne kya kya khareeda, moqe pr kitna naqad dya aur kitna udhaar raha',
      ),
      e(
        'what did Akram buy on the 14th of last month, how much did he pay in cash on the spot and how much stayed on credit',
      ),
    ],
  ),
  // The same shape for a cash buyer: everything paid on the spot, nothing left.
  z(
    'T02',
    'buyer',
    'day',
    'buyer=Rizwan Butt;day=2026-10-14',
    `banana_qty=@${RZ}.banana.qty;banana_rate=@${RZ}.banana.rate;banana_amount=@${RZ}.banana.amount;potato_qty=@${RZ}.potato.qty;potato_rate=@${RZ}.potato.rate;potato_amount=@${RZ}.potato.amount;cash=@${RZ}.cash;credit=@${RZ}.credit`,
    [
      u('رضوان بٹ نے پچھلے مہینے کی 14 کو کیا کیا لیا، کتنا نقد دیا اور ادھار کتنا بچا'),
      rm(
        'rizwan but ne pichle mahine ki 14 ko kya kya liya, nakad kitna diya aur udhaar kitna bacha',
      ),
      e(
        'what did Rizwan Butt take on the 14th of last month, how much cash did he pay and how much is left on credit',
      ),
    ],
  ),
  // The day slip of today (R35): before, the bill, after.
  z(
    'T03',
    'buyer',
    'day',
    'buyer=Faisal;day=today',
    `previous=@${FD}.balance_before;today_bill=@${FD}.charges;new_balance=@${FD}.balance_end`,
    [
      r(
        'faisal ki aaj ki parchi batao: pehle kitna baqi tha, aaj ka bill kitna bana aur ab kul kitna ho gaya',
      ),
      u('فیصل کی آج کی پرچی: پہلے کتنا باقی تھا، آج کا بل کتنا بنا اور اب کل کتنا ہو گیا'),
      em(
        'Faisals slip for todays: what was his balence before, what is todays bil and what does he owe now',
      ),
    ],
  ),
  // A buyer and an item and a year (view A with a buyer).
  z(
    'T04',
    'buyer',
    'item',
    'buyer=Faisal;item=Banana',
    'qty=@ix.year.banana.faisal.qty;amount=@ix.year.banana.faisal.gross|@ix.year.banana.faisal.with_charges',
    [
      r('faisal ne is saal kele ka kitna maal liya aur kitne ka'),
      u('فیصل نے اس سال کیلا کتنی تعداد میں اور کتنے روپے کا خریدا'),
      em('how many banannas did Faisal buy this year and for how much'),
    ],
  ),
  // The oldest open bill of a buyer.
  z(
    'T05',
    'buyer',
    'dues',
    'buyer=Faisal',
    'oldest=@r20.faisal.oldest_date;days=@r20.faisal.days',
    [
      rm('faisal ka sab se purana bil kis tareekh ka he aur kitne din ho gye'),
      u('فیصل کا سب سے پرانا بل کس تاریخ کا ہے اور کتنے دن ہو گئے'),
      e("which date is Faisal's oldest unpaid bill and how many days old is it"),
    ],
  ),
  // Aging buckets of one buyer (R20).
  z(
    'T06',
    'buyer',
    'dues',
    'buyer=Faisal',
    'age_0_7=@r20.faisal.age_0_7;age_8_30=@r20.faisal.age_8_30;age_31_90=@r20.faisal.age_31_90',
    [
      rm(
        'faisal ka bakaya kitne din ke hisab se bata do, hafte bhar ka, mahine bhar ka aur teen mahine ka',
      ),
      u('فیصل کے بقایا میں سے ایک ہفتے تک کا، آٹھ سے تیس دن کا اور اکتیس سے نوے دن کا کتنا ہے'),
      e(
        'of what Faisal owes, how much is up to 7 days old, 8 to 30 days old and 31 to 90 days old',
      ),
    ],
  ),
  // Two parties and an period (a buyer, a seller, last month): every filter must hold.
  z(
    'T07',
    'buyer',
    'complex',
    'buyer=Faisal;seller=Rahim;period=last month',
    'qty=@d7.sale.faisal_rahim_last_month.qty;amount=@d7.sale.faisal_rahim_last_month.gross;rate=@d7.sale.faisal_rahim_last_month.avg_rate',
    [
      r('pichle mahine faisal ne rahim ka kitna maal liya, kis rate par aur kitne ka'),
      u('پچھلے مہینے فیصل نے رحیم کا کتنا مال لیا، کس ریٹ پر اور کتنے کا'),
      em('last month how much of Rahims goods did Faisal take, at what rate and for how much'),
    ],
  ),
  // What one buyer paid in a month, and how many times (receipts, R23).
  z(
    'T08',
    'buyer',
    'paid',
    'buyer=Akram Toll Tax;period=last month',
    'paid=@mx.last_month.r23.akram_toll_tax.total.amount;times=@mx.last_month.r23.akram_toll_tax.count',
    [
      r('akram toll tax ne pichle mahine kitne paise diye aur kitni baar'),
      u('اکرم ٹول ٹیکس سے پچھلے مہینے کتنی وصولی ہوئی اور کتنی بار'),
      em('how much did we colect from Akram Toll Tax last month and in how many reciepts'),
    ],
  ),
  // A buyer in the dead amount (R38): still owed, and what he owed on the day he was moved.
  z(
    'T09',
    'buyer',
    'dead',
    'buyer=Bilal Ahmed',
    'still_owed=@r38.bilal_ahmed.still_owed;owed_then=@r38.bilal_ahmed.owed_then;moved=@r38.bilal_ahmed.date',
    [
      r('bilal ahmed ki raqam doobi raqam me kab gayi aur ab bhi kitna dena hai'),
      um('بلال احمد ڈوبی رقم میں كب گیا، اس پر ابھی كتنا واجب ہے'),
      e(
        'Bilal Ahmed is in the dead amount: how much does he still owe and when was he moved there',
      ),
    ],
  ),

  // ================================ sellers =====================================================================
  // A seller's month: what he sold, the bills, his net.
  z(
    'T10',
    'seller',
    'month',
    'seller=Rahim;period=last month',
    'gross=@b.last_month.rahim.gross;net=@b.last_month.rahim.net;qty=@b.last_month.rahim.qty;bills=@b.last_month.rahim.bikri_count',
    [
      rm('rahim ne pichle mahine kitna mal becha, kitne bil bane aur khalis rakam kitni bani'),
      u('رحیم نے پچھلے مہینے کتنا مال بیچا، کتنے بل بنے اور صافی بکری کتنی رہی'),
      e('how much did Rahim sell last month, in how many bills and what was his net'),
    ],
  ),
  // The commission TAKEN FROM a seller is the seller side only (not the buyer's commission on his goods).
  z(
    'T11',
    'seller',
    'commission',
    'seller=Rahim;period=last month',
    'seller_comm=@b.last_month.rahim.seller_comm',
    [
      rm('rahim se pichle mahine kitna kamishan kata'),
      u('پچھلے مہینے رحیم کی بکری سے ہم نے اس کا کتنا کمیشن کاٹا'),
      e('how much commission did we deduct from Rahim last month'),
    ],
  ),
  // A seller's balance: what the shop owes him.
  z('T12', 'seller', 'payable', 'seller=Rahim', 'balance=@seller.rahim.balance', [
    r('rahim ka hisab kya hai, hum ne usko kitna dena hai'),
    u('رحیم کو ہم نے کتنے پیسے دینے ہیں'),
    em('what do we still ow Rahim'),
  ]),
  // A seller's advance: given, adjusted, open (R26).
  z(
    'T13',
    'seller',
    'advance',
    'seller=Tahir',
    'given=@r26.tahir.given;adjusted=@r26.tahir.adjusted;open=@r26.tahir.open',
    [
      r('tahir ko kul kitni peshgi di, kitni kat chuki aur kitni abhi baqi hai'),
      u('طاہر کو کل کتنی پیشگی دی گئی، کتنی کٹ چکی اور کتنی ابھی کھلی ہے'),
      em(
        'how much advance was given to Tahir in total, how much is adjusted and how much is still open',
      ),
    ],
  ),
  // A seller on a past day: what he sold that day, each item with its qty, rate and amount.
  z(
    'T14',
    'seller',
    'day',
    'seller=Bashir;day=2026-10-14',
    'qty=@seller_day.bashir.d2026_10_14.banana.qty;rate=@seller_day.bashir.d2026_10_14.banana.avg_rate;amount=@seller_day.bashir.d2026_10_14.banana.gross',
    [
      rm('14 oct ko basheer ka kya kya bika, kitni qty, kis rate pr aur kitne ka'),
      u('بشیر کا 14 اکتوبر کو کیا کیا بکا، کتنی تعداد، کس ریٹ پر اور کتنے کا'),
      e('what did Bashir sell on 14 October, how many, at what rate and for how much'),
    ],
  ),
  // A seller and an item and a year: three filters.
  z(
    'T15',
    'seller',
    'complex',
    'seller=Rahim;item=Banana;period=this year',
    'qty=@d7.sale.rahim_banana_year.qty;amount=@d7.sale.rahim_banana_year.gross;seller_comm=@d7.sale.rahim_banana_year.comm_seller',
    [
      r('rahim ka kela is saal kitna bika, kitne ka, aur uska kamishan kitna laga'),
      um('رحیم كا كیلا اس سال كتنا بكا، كتنے كا، اور اس پر كمیشن كتنا لگا'),
      e(
        "how much of Rahim's banana was sold this year, for how much, and what commission was charged on it",
      ),
    ],
  ),
  // A seller and an item and a month: with the dearest and the cheapest rate.
  z(
    'T16',
    'seller',
    'complex',
    'seller=Karim;item=Apple;period=last month',
    'qty=@d7.sale.karim_apple_last_month.qty;max=@d7.sale.karim_apple_last_month.max_rate;min=@d7.sale.karim_apple_last_month.min_rate',
    [
      r('karim ka seb pichle mahine kitna bika, sab se mehnga aur sab se sasta rate kya tha'),
      um('کریم کا سیب پچھلے مہینے کتنا بکا، سب سے مہنگا اور سب سے سستا ریٹ کیا تھا'),
      e(
        "Karim's apples last month: how many were sold, and what were the highest and the lowest rate",
      ),
    ],
  ),
  // A seller's newest bill.
  z(
    'T17',
    'seller',
    'bill',
    'seller=Karim',
    'date=@d7.last_bill.karim.date;number=@d7.last_bill.karim.number;gross=@d7.last_bill.karim.gross;net=@d7.last_bill.karim.net',
    [
      r(
        'karim ka sab se naya bill kis tarikh ka hai, uska number, khaam bikri aur khalis raqam kya hai',
      ),
      u('کریم کا سب سے نیا بل کس تاریخ کا ہے، اس کا نمبر، خام بکری اور صافی بکری کیا ہے'),
      e("what is the date, number, gross and net of Karim's newest bill"),
    ],
  ),
  // The deductions of a seller's bills for the year (R6): every kind, then the total.
  z(
    'T18',
    'seller',
    'deductions',
    'seller=Rahim;period=this year',
    'commission=@r6.rahim.year.total.commission;freight=@r6.rahim.year.total.freight;labour=@r6.rahim.year.total.labour;munshiana=@r6.rahim.year.total.munshiana;deductions=@r6.rahim.year.total.deductions;net=@r6.rahim.year.total.net',
    [
      r(
        'rahim ke is saal ke bills me commission, kiraya, mazdoori aur munshiana kitna kata, kul katautiyan aur net kitna',
      ),
      u(
        'رحیم کے اس سال کے بلوں میں کمیشن، کرایہ، مزدوری اور منشیانہ کتنا کٹا، کل کٹوتی اور صافی کتنی',
      ),
      e(
        "in Rahim's bills this year, how much was deducted for commission, freight, labour and munshiana, and what are the total deductions and the net",
      ),
    ],
  ),
  // A seller and an item with NO sale: the answer is 0, never "not found".
  z(
    'T19',
    'seller',
    'zero',
    'seller=Sarfraz;item=Grapes;period=this year',
    'qty=@d7.sale.sarfraz_grapes_year.qty;amount=@d7.sale.sarfraz_grapes_year.gross',
    [
      r('sarfraz ka angoor is saal kitna bika'),
      u('سرفراز کا انگور اس سال کتنے کا بکا'),
      e('how much of Sarfraz grapes was sold this year'),
    ],
  ),
  // How many sellers are waiting for money, and how much in all.
  z('T20', 'seller', 'payable', 'period=now', 'count=@r24.count;total=@tiles.sellers_payable', [
    r('kitne bayopariyon ko abhi paise dene hain aur kul kitne'),
    u('کتنے بیوپاریوں کو ابھی ادائیگی باقی ہے اور کل کتنی رقم بنتی ہے'),
    e('how many sellers are still to be paid and how much is it in all'),
  ]),

  // ================================ partners ====================================================================
  z(
    'T21',
    'partner',
    'share',
    'partner=Partner A',
    'month=@partner.partner_a.month_share;year=@partner.partner_a.year_share',
    [
      rm('alif ka is mahine ka hisa kitna bna aur is sal ka kitna'),
      u('الف کا اس مہینے کا حصہ کتنا بنا اور اس سال کا کتنا'),
      e("what is Partner A's share of profit this month and this year"),
    ],
  ),
  z(
    'T22',
    'partner',
    'drawings',
    'partner=Partner A',
    'taken=@r29.all.partner_a.taken;returned=@r29.all.partner_a.returned',
    [
      r('alif ne ab tak kitne paise liye aur kitne wapas kiye'),
      u('الف نے اب تک ذاتی خرچ میں کتنے پیسے لیے اور کتنے واپس کیے'),
      em('how much has Partner A taken for himself so far and how much has he retured'),
    ],
  ),
  z('T23', 'partner', 'capital', 'partner=Partner B', 'capital=@partner.partner_b.capital', [
    r('b partner ne dukaan me kitna sarmaya lagaya hua hai'),
    u('ب شراکت دار کا کتنا سرمایہ لگا ہوا ہے'),
    e('how much capital has Partner B put into the shop'),
  ]),
  // The share is kept as 3,000 basis points and shown as 30%; the total is another figure.
  z(
    'T24',
    'partner',
    'total',
    'partner=Usman',
    'share=@d7.partner.usman.share_pct;total=@partner.usman.total',
    [
      rm('usman shirakat daar ka hissa kitne feesad hai aur uski kul raqam kitni hai'),
      u('عثمان شراکت دار کا حصہ کتنے فیصد ہے اور اس کی کل رقم کتنی ہے'),
      e("what percent is Usman's share and what is his total in the shop"),
    ],
  ),
  // A partner's money from gardens, trips and fruit taken (view F).
  z(
    'T25',
    'partner',
    'ventures',
    'partner=Partner A;period=this year',
    'gardens=@f.year.partner_a.gardens;trips=@f.year.partner_a.trips;fruit=@f.year.partner_a.fruit',
    [
      rm('alif ko is sal bagh, safr aur phal me kitna laga ya mila'),
      u('الف کا اس سال باغ، سفر اور پھل میں کتنا لگا یا ملا'),
      e('what did Partner A put in or get from gardens, trips and fruit taken this year'),
    ],
  ),
  z(
    'T26',
    'partner',
    'profit',
    'partner=Partner A;period=this year',
    'income=@f.year.partner_a.income_share;expense=@f.year.partner_a.expense_share;profit=@f.year.partner_a.profit_share',
    [
      r('alif ki is saal ki aamdani ka hissa, kharche ka hissa aur munafa kitna hai'),
      u('الف کی اس سال کی آمدن کا حصہ، خرچے کا حصہ اور منافع کتنا ہے'),
      em('Partner As share of incom, share of expences and profit this year'),
    ],
  ),
  // Who took the most money, and how much (a ranking over the partners).
  z(
    'T27',
    'partner',
    'rank',
    'period=all',
    'partner!=@d7.top.drawings.name;amount=@d7.top.drawings.amount',
    [
      r('kis shirakat daar ne sab se zyada paise liye aur kitne'),
      u('کس شراکت دار نے سب سے زیادہ ذاتی رقم لی اور کتنی'),
      e('which partner has taken the most money for himself and how much'),
    ],
  ),

  // ================================ expenses ====================================================================
  z(
    'T28',
    'expense',
    'head',
    'head=Electricity;period=all',
    'expense=@r29.all.electricity.expense',
    [
      r('ab tak bijli par kitna kharcha hua'),
      u('اب تک بجلی کے بل پر کل کتنا خرچ ہوا'),
      e('how much has been spent on electricity so far'),
    ],
  ),
  z(
    'T29',
    'expense',
    'period',
    'period=last month',
    'expense=@mx.last_month.r29.total.expense;taken=@mx.last_month.r29.total.taken',
    [
      rm('pichle mahine dukan ka kharcha kitna hua aur sharkat daron ne kitna liya'),
      u('پچھلے مہینے دکان کا خرچہ کتنا ہوا اور شراکت داروں نے ذاتی کتنا لیا'),
      e("last month's shop expenses and the money the partners took for themselves"),
    ],
  ),
  // Nothing was spent today: 0.
  z('T30', 'expense', 'zero', 'period=today', 'expense=@tiles.expenses_today', [
    r('aaj dukaan ka koi kharcha hua'),
    u('آج دکان کا کتنا خرچہ ہوا'),
    e('how much was spent on the shop today'),
  ]),
  // Two heads together (a sum the app must work out).
  z('T31', 'expense', 'sum', 'period=all', 'sum=@d7.exp.rent_plus_electricity', [
    r('dukaan ke kiraye aur bijli ka kharcha mila kar ab tak kitna hua'),
    u('دکان کے کرایے اور بجلی کا خرچہ ملا کر اب تک کتنا ہوا'),
    em('shop rent and electricity together, how much so far'),
  ]),
  z(
    'T32',
    'expense',
    'rank',
    'period=this year',
    'head!=@top.expense_head_year.name;amount=@top.expense_head_year.amount',
    [
      r('is saal sab se zyada kharcha kis mad me hua aur kitna'),
      u('اس سال سب سے زیادہ خرچہ کس مد میں ہوا اور کتنا'),
      e('which expense head has the most spending this year and how much'),
    ],
  ),

  // ================================ money lent, separate khata ==================================================
  z(
    'T33',
    'lent',
    'person',
    'person=Aslam',
    'lent=@r32.aslam.lent;returned_cash=@r32.aslam.returned_cash;returned_salary=@r32.aslam.returned_salary;open=@r32.aslam.open',
    [
      r(
        'aslam ko kitna udhaar diya tha, kitna cash wapas aya, kitna tankhwah se kata aur kitna abhi baqi hai',
      ),
      u(
        'اسلم کو کتنی رقم ادھار دی تھی، کتنی نقد واپس آئی، کتنی تنخواہ سے کٹی اور کتنی ابھی باقی ہے',
      ),
      e(
        'how much was lent to Aslam, how much came back in cash, how much was cut from his salary and how much is still open',
      ),
    ],
  ),
  z(
    'T34',
    'lent',
    'overdue',
    'person=Aslam',
    'return_by=@r32.aslam.return_by;days=@r32.aslam.days_past',
    [
      rm('aslam ne paise wapas karne ki tarikh kya di thi aur kitne din guzar gaye'),
      u('اسلم نے رقم واپس کرنے کی کون سی تاریخ دی تھی اور کتنے دن گزر چکے'),
      e('what return date did Aslam give and how many days past it are we'),
    ],
  ),
  // The separate khata (الگ کھاتہ): 23,000 paid out, 5,000 received, so it stands at minus 18,000 (seed.rs d7_richer_cases).
  z(
    'T35',
    'khata',
    'khata',
    'khata=Family Khata',
    'balance=@d7.khata.family.balance_signed|@d7.khata.family.balance;paid=@d7.khata.family.paid_out;received=@d7.khata.family.received',
    [
      r('khandani khata me abhi kitna paisa para hai, kitna diya aur kitna wapas aya'),
      u('خاندانی کھاتے میں اس وقت کتنی رقم ہے، کتنی دی اور کتنی واپس آئی'),
      e('how much is in the family khata now, how much was paid out and how much came back'),
    ],
  ),

  // ================================ dues and payables ===========================================================
  z('T36', 'dues', 'total', 'period=now', 'buyers=@tiles.buyers_dues', [
    r('gahakon ke zimme kul kitna baqi hai'),
    um('گاہكوں كے ذمے كل كتنا باقي ہے'),
    e('how much do the buyers owe us in all'),
  ]),
  z('T37', 'dues', 'total', 'period=now', 'total=@tiles.total_dues', [
    r('kul ugrahi kitni banti hai'),
    um('كل اگراہی كتنی بنتی ہے'),
    e('what is the total dues of the shop'),
  ]),
  z('T38', 'dues', 'aging', 'period=now', 'over_90=@r20.total.age_91', [
    r('91 din se purane bakaye kitne hain'),
    u('نوے دن سے زیادہ پرانا بقایا کل کتنا ہے'),
    e('how much of the dues is more than 90 days old'),
  ]),
  // A collector's list (R22).
  z(
    'T39',
    'dues',
    'collector',
    'person=Rasheed',
    'balance=@r22.rasheed.balance;age_31_90=@r22.rasheed.age_31_90',
    [
      r('rasheed ke hisse me kitni wasooli hai, usme se 31 se 90 din purani kitni'),
      u('رشید کے ذمے کتنی اگراہی ہے اور اس میں سے اکتیس سے نوے دن پرانی کتنی'),
      e("how much is on Rasheed's collection list and how much of it is 31 to 90 days old"),
    ],
  ),
  z('T40', 'dues', 'dead', 'period=now', 'dead=@tiles.dead;persons=@tiles.dead_count', [
    r('doobi raqam kitni hai aur kitne logon ki'),
    u('ڈوبی رقم کل کتنی ہے اور کتنے لوگوں کی'),
    e('how much is in the dead amount and for how many people'),
  ]),
  // The attention list counts, as plain real-time figures.
  z(
    'T41',
    'dues',
    'attention',
    'period=now',
    'count=@attention.dues_old.count;amount=@attention.dues_old.amount',
    [
      r('kitne gahakon ka bakaya 30 din se purana ho gaya aur kitna'),
      u('کتنے گاہکوں کا بقایا تیس دن سے پرانا ہو گیا اور کل کتنا'),
      e('how many buyers owe a bill older than 30 days and how much in all'),
    ],
  ),
  z(
    'T42',
    'advance',
    'open',
    'period=now',
    'open=@tiles.advances_open;given=@r26.total.given;adjusted=@r26.total.adjusted',
    [
      r('sab bayopariyon ko peshgi kitni di, kitni adjust hui aur kitni khuli hai'),
      u('سب بیوپاریوں کو پیشگی کتنی دی، کتنی ایڈجسٹ ہوئی اور کتنی کھلی ہے'),
      e('how much advance was given to all sellers, how much is adjusted and how much is open'),
    ],
  ),

  // ================================ cash book, day book, trial balance ===========================================
  z('T43', 'cash', 'now', 'period=now', 'cash=@tiles.cash', [
    r('galle me abhi kitna cash hai'),
    um('روكڑ میں اس وقت كتنی رقم ہے'),
    e('how much cash do we have in hand right now'),
  ]),
  z(
    'T44',
    'cash',
    'book',
    'period=this month',
    'opening=@r12.month.sum.report_opening_balance;cash_in=@r12.month.total.cash_in;cash_out=@r12.month.total.cash_out;closing=@r12.month.sum.report_closing_balance',
    [
      rm('is mahine ki roknad ki kitaab: shuru me kitna tha, kitna aya, kitna gya aur ab kitna he'),
      u('اس مہینے کی روکڑ بک: شروع میں کتنا تھا، کتنا آیا، کتنا گیا اور اب کتنا ہے'),
      e('the cash book this month: opening, cash in, cash out and closing'),
    ],
  ),
  // Yesterday: money only went out (cash in is 0).
  z(
    'T45',
    'cash',
    'day',
    'period=yesterday',
    'cash_in=@r12.yesterday.total.cash_in;cash_out=@r12.yesterday.total.cash_out;closing=@r12.yesterday.sum.report_closing_balance',
    [
      r('kal galle me kitna aya, kitna gaya aur raat ko kitna bacha'),
      u('کل روکڑ میں کتنا آیا، کتنا گیا اور رات کو کتنا بچا'),
      em('yesterday how much cash came in, how much went out and how much was left at nite'),
    ],
  ),
  // Today: nothing was received and nothing was paid out: 0 and 0.
  z(
    'T46',
    'cash',
    'zero',
    'period=today',
    'received=@r23.today.total.amount;paid=@r25.today.total.amount',
    [
      r('aaj gahakon se kitni wasooli hui aur bayopariyon ko kitni adaigi hui'),
      u('آج گاہکوں سے کتنی وصولی ہوئی اور بیوپاریوں کو کتنی ادائیگی ہوئی'),
      e('how much was received from buyers today and how much was paid to sellers'),
    ],
  ),
  z(
    'T47',
    'cash',
    'period',
    'period=last month',
    'received=@mx.last_month.r23.total.amount;receipts=@mx.last_month.r23.count;paid=@mx.last_month.r25.total.amount;payments=@mx.last_month.r25.count',
    [
      r(
        'pichle mahine kitni wasooli hui, kitni rasidein bani aur bayopariyon ko kitna diya, kitni baar',
      ),
      u('پچھلے مہینے کتنی وصولی ہوئی، کتنی رسیدیں بنیں اور بیوپاریوں کو کتنا دیا، کتنی بار'),
      e(
        'last month how much was received in how many receipts, and how much was paid to sellers in how many payments',
      ),
    ],
  ),
  z(
    'T48',
    'daybook',
    'today',
    'period=today',
    'lines=@r11.today.count;debit=@r11.today.total.debit',
    [
      r('aaj ke rozname me kitni entries hain aur banaam ka total kitna hai'),
      u('آج کے روزنامچے میں کتنی اندراجات ہیں اور بنام کا میزان کتنا ہے'),
      e("how many entries are in today's day book and what is the debit total"),
    ],
  ),
  // The trial balance: both sides equal, the difference is 0.
  z(
    'T49',
    'trial',
    'balance',
    'period=now',
    'debit=@r18.total.debit;credit=@r18.total.credit;difference=@r18.sum.report_difference',
    [
      r('trial balance ka banaam aur jama kitna hai aur farq kitna hai'),
      u('ٹرائل بیلنس کا بنام اور جمع کتنا ہے اور دونوں میں فرق کتنا ہے'),
      e(
        'what are the debit and credit totals of the trial balance and the difference between them',
      ),
    ],
  ),
  z(
    'T50',
    'trial',
    'opening',
    'period=opening',
    'debit=@r19.total.debit;credit=@r19.total.credit',
    [
      rm('shuru ke trial balance me kitna banaam aur jama tha'),
      u('ابتدائی ٹرائل بیلنس میں بنام اور جمع کتنا تھا'),
      e('what were the debit and credit of the opening trial balance'),
    ],
  ),
  z('T51', 'trial', 'index', 'period=now', 'accounts=@r16.count', [
    r('kul kitne khate bane hue hain'),
    u('کل کتنے کھاتے بنے ہوئے ہیں'),
    e('how many accounts are there in all'),
  ]),
  z(
    'T52',
    'days',
    'state',
    'period=now',
    'closed=@days.closed_up_to;open_days=@attention.open_days.count',
    [
      r('din kis tarikh tak band ho chuka hai aur kitne din abhi khule hain'),
      u('دن کس تاریخ تک بند ہو چکے ہیں اور کتنے دن ابھی کھلے ہیں'),
      e('up to which date are the days closed and how many days are still open'),
    ],
  ),

  // ================================ today and counts ============================================================
  // The audit log: how many times a bill was written (the first save).
  z('T90', 'audit', 'count', 'bikri=1', 'changes=@audit.bikri1.count', [
    r('bikri number 1 ka record kitni baar likha gaya'),
    u('بکری نمبر 1 کا ریکارڈ کتنی بار لکھا گیا ہے'),
    e('how many audit entries does sale number 1 have'),
  ]),
  z(
    'T53',
    'summary',
    'today',
    'period=today',
    'sales=@tiles.sales_today;bills=@tiles.count_today;commission=@tiles.commission_today',
    [
      rm('aj ki kul bikri, kitne bil bane aur kamishan kitna hoa'),
      u('آج کی کل بکری کتنی ہوئی، کتنے بل بنے اور کمیشن کتنا ہوا'),
      e("today's sales, the number of bills and the commission"),
    ],
  ),
  z(
    'T54',
    'summary',
    'count',
    'period=many',
    'today=@d7.bikri.today.count;month=@d7.bikri.month.count;last_month=@d7.bikri.last_month.count',
    [
      r('aaj, is mahine aur pichle mahine kitni bikri (bill) bani'),
      u('آج، اس مہینے اور پچھلے مہینے کتنی بکریاں بنیں'),
      e('how many sale bills were made today, this month and last month'),
    ],
  ),

  // ================================ stock =======================================================================
  z('T55', 'stock', 'item', 'item=Mosambi', 'qty=@stock.mosambi.qty;lots=@stock.mosambi.lots', [
    r('moosami ka kitna maal bacha hua hai aur kitne lot me'),
    u('موسمی کا کتنا مال بچا ہوا ہے اور کتنے لاٹ میں'),
    e('how much mosambi is left in stock and in how many lots'),
  ]),
  // A group of items (a sum the app must work out from the items).
  z(
    'T56',
    'stock',
    'group',
    'group=vegetable',
    'qty=@d7.stock.vegetable.qty;lots=@d7.stock.vegetable.lots',
    [
      r('sabziyon ka kul kitna maal baqi hai aur kitne lot'),
      u('سبزیوں کا کل کتنا مال باقی ہے اور کتنے لاٹ'),
      e('how much vegetable stock is left in all and in how many lots'),
    ],
  ),
  z(
    'T57',
    'stock',
    'waiting',
    'period=now',
    'days=@attention.stock_waiting.days;lots=@attention.stock_waiting.count',
    [
      r('sab se purana maal kitne din se para hai aur kitne lot do din se zyada ruke hue hain'),
      u('سب سے پرانا مال کتنے دن سے پڑا ہے اور کتنے لاٹ دو دن سے زیادہ رکے ہوئے ہیں'),
      e(
        'how many days has the oldest stock been waiting and how many lots have waited more than 2 days',
      ),
    ],
  ),
  z('T58', 'stock', 'trip', 'period=now', 'qty=@tiles.trips_qty_left;cost=@tiles.trips_cost_left', [
    r('safar ka kitna maal abhi bika nahi aur uski lagat kitni hai'),
    u('سفر کا کتنا مال ابھی بکا نہیں اور اس کی لاگت کتنی ہے'),
    e('how much of the trip goods is still unsold and what is its cost'),
  ]),
  z(
    'T59',
    'stock',
    'received',
    'item=Banana;period=last month',
    'qty=@d7.recv.banana_last_month.qty;lots=@d7.recv.banana_last_month.lots',
    [
      r('pichle mahine kele ka kitna maal aya aur kitne lot me'),
      u('پچھلے مہینے کیلے کا کتنا مال آیا اور کتنے لاٹ میں'),
      e('how much banana came in last month and in how many lots'),
    ],
  ),
  z(
    'T60',
    'stock',
    'totals',
    'period=all',
    'received=@r10.total.received;sold=@r10.total.sold;written_off=@r10.total.written_off',
    [
      r('ab tak kul kitna maal aya, kitna bika aur kitna zaya hua'),
      u('اب تک کل کتنا مال آیا، کتنا بکا اور کتنا ضائع ہوا'),
      e('how much goods have come in so far, how much has been sold and how much written off'),
    ],
  ),

  // ================================ items, groups, quality, rates ===============================================
  z(
    'T61',
    'item',
    'month',
    'item=Banana;period=last month',
    'gross=@ix.last_month.banana.gross;qty=@ix.last_month.banana.qty;commission=@ix.last_month.banana.comm_total',
    [
      r('pichle mahine kele ki bikri, qty aur kul kamishan kitna raha'),
      u('پچھلے مہینے کیلے کی بکری، تعداد اور کل کمیشن کتنا رہا'),
      e("last month's banana sales, quantity and total commission"),
    ],
  ),
  z(
    'T62',
    'group',
    'year',
    'group=fruit,vegetable;period=this year',
    'fruit=@group.fruit.year.gross;vegetable=@group.vegetable.year.gross',
    [
      r('is saal phal ki bikri zyada hui ya sabzi ki, dono ki kitni kitni'),
      u('اس سال پھل کی بکری کتنی ہوئی اور سبزی کی کتنی'),
      e('this year how much did fruit sell for and how much did vegetables sell for'),
    ],
  ),
  // One quality of one item in one month: every filter must hold.
  z(
    'T63',
    'quality',
    'complex',
    'item=Apple;grade=first;period=last month',
    'qty=@ixq.last_month.apple.first.qty;amount=@ixq.last_month.apple.first.gross;avg_rate=@ixq.last_month.apple.first.avg_rate',
    [
      r('pichle mahine awwal darje ka seb kitna bika, kitne ka aur ausat rate kya raha'),
      u('پچھلے مہینے اول درجے کا سیب کتنا بکا، کتنے کا اور اوسط ریٹ کیا رہا'),
      em('last month how much first quality apple sold, for how much and the average rate'),
    ],
  ),
  // The newest rate of an item this week (an open critical of plan 59: the date and the newest line).
  z(
    'T64',
    'rate',
    'last',
    'item=Apple;period=this week',
    'rate=@d7.sale.apple_week.last_rate;date=@d7.sale.apple_week.last_date',
    [
      r('is hafte seb ka aakhri rate kya laga'),
      u('اس ہفتے سیب کا آخری ریٹ کیا لگا'),
      e('what was the latest rate of apple this week'),
    ],
  ),
  // The commission taken FROM a buyer is the buyer side only (an open critical of plan 59).
  z(
    'T65',
    'item',
    'commission',
    'buyer=Faisal;item=Banana;period=last month',
    'commission=@ix.last_month.banana.faisal.comm_buyer',
    [
      r('pichle mahine faisal se kele par kitna commission liya'),
      u('پچھلے مہینے فیصل سے کیلے پر کتنا کمیشن لیا گیا'),
      e('how much commission did we take from Faisal on banana last month'),
    ],
  ),
  // Two items against each other: which sold more and by how much.
  z(
    'T66',
    'item',
    'compare',
    'item=Banana,Apple;period=this month',
    'more!=@d7.cmp.banana_apple.month.more;diff=@d7.cmp.banana_apple.month.diff',
    [
      r('is mahine kele aur seb me se kis ki bikri zyada hui aur farq kitna hai'),
      u('اس مہینے کیلے اور سیب میں سے کس کی بکری زیادہ ہوئی اور فرق کتنا ہے'),
      e('this month which sold more, banana or apple, and by how much'),
    ],
  ),
  // A buyer and an item and a quality and a year: four filters.
  z(
    'T67',
    'quality',
    'complex',
    'buyer=Usman Ghani;item=Apple;grade=first;period=this year',
    'lines=@d7.sale.usman_apple_first_year.lines;qty=@d7.sale.usman_apple_first_year.qty;amount=@d7.sale.usman_apple_first_year.gross',
    [
      r('usman ghani ne is saal awwal darje ka seb kitni baar liya, kitna aur kitne ka'),
      u('عثمان غنی نے اس سال اول درجے کا سیب کتنی بار لیا، کتنا اور کتنے کا'),
      e(
        'how many times did Usman Ghani buy first quality apple this year, how much and for how much',
      ),
    ],
  ),
  // An item and cash (نقدی) only, over a year.
  z(
    'T68',
    'item',
    'cash',
    'item=Banana;pay=cash;period=this year',
    'lines=@d7.sale.banana_cash_year.lines;qty=@d7.sale.banana_cash_year.qty;amount=@d7.sale.banana_cash_year.gross',
    [
      r('is saal kele ki naqdi bikri kitni hui, kitne lines, kitni qty aur kitne ki'),
      u('اس سال کیلے کی نقدی بکری کتنی ہوئی، کتنی لائنیں، کتنی تعداد اور کتنے کی'),
      e('banana sold for cash this year: how many lines, how many units and for how much'),
    ],
  ),
  // A group and credit only, over a month.
  z(
    'T69',
    'group',
    'credit',
    'group=fruit;pay=credit;period=last month',
    'amount=@d7.sale.fruit_credit_last_month.gross;qty=@d7.sale.fruit_credit_last_month.qty;buyers=@d7.sale.fruit_credit_last_month.buyers_n',
    [
      r('pichle mahine phal ki udhaar bikri kitni rahi, kitni qty aur kitne gahakon ko'),
      u('پچھلے مہینے پھل کی ادھاری بکری کتنی رہی، کتنی تعداد اور کتنے گاہکوں کو'),
      e('fruit sold on credit last month: for how much, how many units and to how many buyers'),
    ],
  ),
  // Zeros: a buyer and an item with no sale today; nothing is "not found".
  z(
    'T70',
    'zero',
    'complex',
    'buyer=Akram Baloch;item=Mango;period=today',
    'qty=@d7.sale.akram_baloch_mango_today.qty;amount=@d7.sale.akram_baloch_mango_today.gross',
    [
      r('akram baloch ne aaj aam kitne ke liye'),
      u('اکرم بلوچ نے آج آم کتنے کے لیے'),
      e('how much mango did Akram Baloch buy today'),
    ],
  ),
  z(
    'T71',
    'zero',
    'complex',
    'buyer=Nadeem Traders;item=Potato;period=this week',
    'qty=@d7.sale.nadeem_potato_week.qty;amount=@d7.sale.nadeem_potato_week.gross',
    [
      rm('nadeem traders ne is hafte aloo kitna uthaya'),
      u('ندیم ٹریڈرز نے اس ہفتے آلو کتنا لیا'),
      e('how much potato did Nadeem Traders take this week'),
    ],
  ),
  // Cash against credit today: no cash sale, so cash is 0 and credit is the whole day.
  z(
    'T72',
    'zero',
    'cash',
    'period=today',
    'cash=@d7.sale.cash_today.gross;credit=@d7.sale.credit_today.gross',
    [
      r('aaj nakad bikri kitni hui aur udhaar kitni'),
      u('آج نقد بکری کتنی ہوئی اور ادھار کتنی'),
      e('how much was sold for cash today and how much on credit'),
    ],
  ),
  // Vegetables sold nothing today: 0.
  z(
    'T73',
    'zero',
    'group',
    'group=vegetable;period=today',
    'amount=@group.vegetable.today.gross;qty=@group.vegetable.today.qty',
    [
      r('aaj sabziyon ki bikri kitni hui'),
      u('آج سبزیوں کی بکری کتنی ہوئی'),
      e('how much vegetables were sold today'),
    ],
  ),
  // A buyer and an item and a quality and a month.
  z(
    'T74',
    'quality',
    'complex',
    'buyer=Akram Baloch;item=Banana;grade=second;period=last month',
    'lines=@d7.sale.akram_baloch_banana_second_last_month.lines;qty=@d7.sale.akram_baloch_banana_second_last_month.qty;amount=@d7.sale.akram_baloch_banana_second_last_month.gross',
    [
      r('akram baloch ne pichle mahine doyam darje ka kela kitni baar, kitna aur kitne ka liya'),
      u('اکرم بلوچ نے پچھلے مہینے دوم درجے کا کیلا کتنی بار، کتنا اور کتنے کا لیا'),
      e(
        'how many times did Akram Baloch buy second quality banana last month, how much and for how much',
      ),
    ],
  ),
  // A cash buyer in a named month.
  z(
    'T75',
    'item',
    'complex',
    'buyer=Shahid Raza;item=Banana;period=September',
    'qty=@d7.sale.shahid_banana_sep.qty;amount=@d7.sale.shahid_banana_sep.gross;paid=@d7.sale.shahid_banana_sep.charges',
    [
      r(
        'shahid raza ne september me kele ka kitna maal aur kitne ka liya, commission mila kar kitna diya',
      ),
      u('شاہد رضا نے ستمبر میں کیلے کا کتنا مال اور کتنے کا لیا، کمیشن ملا کر کتنا دیا'),
      e(
        'how much banana did Shahid Raza take in September, for how much, and how much did he pay with commission added',
      ),
    ],
  ),
  // The best month of an item, and the best vegetable.
  z(
    'T76',
    'item',
    'best',
    'item=Banana;period=this year',
    'month=@ix.year.banana.best_month;gross=@ix.year.banana.best_month_gross',
    [
      r('is saal kele ka sab se acha mahina konsa raha aur us mahine kitni bikri hui'),
      u('اس سال کیلے کا سب سے اچھا مہینہ کون سا رہا اور اس مہینے کتنی بکری ہوئی'),
      e('which month was the best for banana this year and how much sold in that month'),
    ],
  ),

  // ================================ trips, gardens, fruit taken ==========================================
  z(
    'T77',
    'trip',
    'open',
    'trip=0071',
    'bought=@trip2.bought_qty;sold=@trip2.sold_qty;left=@trip2.left_qty;result=@trip2.result',
    [
      r(
        'chalu safar me kitna maal kharida, kitna bika, kitna bacha aur ab tak nafa nuqsan kitna hai',
      ),
      u('چالو سفر میں کتنا مال خریدا، کتنا بکا، کتنا بچا اور اب تک نفع نقصان کتنا ہے'),
      e(
        'in the open trip how much was bought, sold and left, and what is the profit or loss so far',
      ),
    ],
  ),
  z(
    'T78',
    'trip',
    'closed',
    'trip=0070',
    'bought=@trip1.bought_cost;written_off=@trip1.written_off_qty;result=@trip1.result',
    [
      r(
        'band shuda safar me kitni lagat lagi thi, kitna maal zaya hua aur akhir me nateeja kya raha',
      ),
      u('بند شدہ سفر میں کتنی لاگت لگی تھی، کتنا مال ضائع ہوا اور آخر میں نتیجہ کیا رہا'),
      e('in the closed trip what was the cost, how much was written off and what was the result'),
    ],
  ),
  z(
    'T79',
    'garden',
    'rent',
    'garden=Shahpur Garden',
    'rent=@r36.shahpur_garden.rent;installments=@r36.shahpur_garden.installments;rent_due=@r36.shahpur_garden.rent_due',
    [
      r('shahpur baagh ka thekah kitna tha, kitni qiston me di aur kitna baqi hai'),
      u('شاہ پور باغ کا ٹھیکہ کتنا تھا، کتنی قسطیں دیں اور کتنا باقی ہے'),
      e(
        'what is the rent of Shahpur garden, how much was paid in installments and how much is due',
      ),
    ],
  ),
  z(
    'T80',
    'garden',
    'paid',
    'garden=Shahpur Garden',
    'paid_shop=@r36.shahpur_garden.paid_shop;paid_pocket=@r36.shahpur_garden.paid_pocket;costs=@r36.shahpur_garden.costs;balance=@r36.shahpur_garden.balance',
    [
      r(
        'shahpur baagh ke liye dukaan se kitna diya, apni jeb se kitna aur doosre kharche kitne, hisab kitna hai',
      ),
      u('شاہ پور باغ کے لیے دکان سے کتنا دیا، اپنی جیب سے کتنا، دوسرے خرچے کتنے اور حساب کتنا ہے'),
      e(
        'for Shahpur garden how much came from the shop, how much from own pockets, what are the costs and the running balance',
      ),
    ],
  ),
  z('T81', 'fruit', 'total', 'period=all', 'qty=@r37.all.total.qty;value=@r37.all.total.value', [
    r('ab tak phal kitna liya gaya aur uski qeemat kitni bani'),
    u('اب تک پھل کتنا لیا گیا اور اس کی قیمت کتنی بنی'),
    e('how much fruit has been taken so far and what is its value'),
  ]),
  z(
    'T82',
    'fruit',
    'partner',
    'partner=Partner A',
    'qty=@k.all.partner_a.qty;value=@k.all.partner_a.value',
    [
      r('alif ne ghar ke liye kitna phal liya aur uski kul qeemat kitni'),
      u('الف نے گھر کے لیے کتنا پھل لیا اور اس کی کل قیمت کتنی'),
      e('how much fruit did Partner A take for home and what is its total value'),
    ],
  ),
  z(
    'T83',
    'fruit',
    'guest',
    'seller=Haji Nazeer',
    'qty=@r37.all.haji_nazeer.qty;rate=@r37.all.haji_nazeer.rate;value=@r37.all.haji_nazeer.value;date=@r37.all.haji_nazeer.date',
    [
      rm(
        'haji nazir ki dukaan se mehmano ke liye kitna phal liya, kis rate par, kitne ka aur kis tarikh ko',
      ),
      u('حاجی نذیر کی دکان سے مہمانوں کے لیے کتنا پھل لیا، کس ریٹ پر، کتنے کا اور کس تاریخ کو'),
      e(
        "how much fruit did we take for guests from Haji Nazeer's shop, at what rate, for how much and on which date",
      ),
    ],
  ),

  // ================================ rules, trucks, masters ======================================================
  z(
    'T85',
    'rules',
    'rule',
    'seller=Rahim;item=Banana',
    'seller_commission=@rules.rahim.banana.seller_commission',
    [
      r('rahim ke kele par hum kitna kamishan lete hain'),
      u('رحیم کے کیلے پر ہم کتنے فیصد کمیشن لیتے ہیں'),
      e("what percent commission do we charge on Rahim's banana"),
    ],
  ),
  z('T86', 'rules', 'laga', 'item=Apple', 'laga=@rules.apple.buyer_laga', [
    r('seb par gahak se laga kitna lagta hai'),
    u('سیب پر گاہک سے لاگا کتنا لگتا ہے'),
    e('how much laga is charged to the buyer on apple'),
  ]),
  z(
    'T87',
    'truck',
    'last',
    'truck=LES-1057',
    'count=@truck.les_1057.count;date=@truck.les_1057.last_date;gross=@truck.les_1057.last_gross;net=@truck.les_1057.last_net',
    [
      r('LES-1057 truck kitni baar aya, akhri baar kab, khaam bikri aur khalis raqam kya thi'),
      u('ٹرک LES-1057 کتنی بار آیا، آخری بار کب، خام بکری اور صافی بکری کیا تھی'),
      e(
        'how many times did truck LES-1057 come, when was the last time and what were the gross and the net',
      ),
    ],
  ),
  z(
    'T88',
    'master',
    'staff',
    'period=now',
    'munshi=@staff.munshi;collector=@staff.collector;all=@staff.all',
    [
      r('dukaan me kitne munshi, kitne ugrahi wale aur kul kitne mulazim hain'),
      u('دکان میں کتنے منشی، کتنے اگراہی والے اور کل کتنے ملازم ہیں'),
      e('how many munshis, how many collectors and how many staff in all'),
    ],
  ),
  z(
    'T89',
    'master',
    'lists',
    'period=now',
    'heads=@heads.count;packings=@packings.count;stores=@stores.count;markas=@markas.count',
    [
      r('kitni kharcha madein, kitni packing, kitne store aur kitne marke bane hue hain'),
      u('کتنی خرچہ مدیں، کتنی پیکنگ، کتنے سٹور اور کتنے مارکے بنے ہوئے ہیں'),
      e('how many expense heads, packings, stores and marks are set up'),
    ],
  ),
];

// ---- the findings of the reviewer study (plan 59 Section 5.5, R1 to R13b) and the newest-line cases ------------------
//
// Each group is a question that goes wrong when the finding is not fixed. They are picky on purpose.
//
// The newest line (lead rule, plan 59 Section 11): "rate" is the SELLER rate, and the newest sale line is the one with
// the latest posting date, then the highest بکری number, then the highest line number, never the row id. The rich year
// has grapes only in the last three days (`seed.rs`, `d7_last_rate_cases`): a بکری of 03-11 with a LATER SALE on
// 05-11 (4,600), a بکری of 05-11 edited last (the newest row ids, 5,200), and the highest بکری of 05-11 with four
// lines and an edit (its last line, 5,000). The newest grapes rate is 5,000 by the rule; the row id says 5,200.

const GT = 'd7.sale.grapes_today';
const GW = 'd7.sale.grapes_week';
const KG = 'd7.sale.shahid_grapes_today';
const NV = 'd7.bd.naveed.d2026_10_14';

const FINDINGS = [
  // R1: the last rate, by the rule, with its date (an empty date or the first line of the day is a miss).
  z(
    'N01',
    'rate',
    'last',
    'item=Grapes;period=today',
    `rate=@${GT}.last_rate;date=@${GT}.last_date`,
    [
      r('aaj angoor ka aakhri rate kya laga'),
      u('آج انگور کا آخری ریٹ کیا لگا'),
      e('what was the latest rate of grapes today'),
    ],
  ),
  // The last rate for one buyer: his newest line is a line of the highest بکری (4,900), not the line of the edit written last (5,050, the row id).
  z(
    'N02',
    'rate',
    'last',
    'buyer=Shahid Raza;item=Grapes;period=today',
    `rate=@${KG}.last_rate;date=@${KG}.last_date`,
    [
      r('shahid raza ko aaj angoor ka aakhri rate kya diya'),
      u('شاہد رضا کو آج انگور کا آخری ریٹ کیا لگا'),
      e('what was the last rate of grapes that Shahid Raza got today'),
    ],
  ),
  // The dearest and the cheapest rate: the cheapest is the later sale of an older بکری.
  z(
    'N03',
    'rate',
    'range',
    'item=Grapes;period=this week',
    `max=@${GW}.max_rate;min=@${GW}.min_rate`,
    [
      r('is hafte angoor ka sab se mehnga aur sab se sasta rate kya raha'),
      u('اس ہفتے انگور کا سب سے مہنگا اور سب سے سستا ریٹ کیا رہا'),
      e('this week what was the highest and the lowest rate of grapes'),
    ],
  ),
  // A day with several lines, an edited بکری and a later sale: every line counts once.
  z(
    'N04',
    'item',
    'day',
    'item=Grapes;period=today',
    `lines=@${GT}.lines;qty=@${GT}.qty;amount=@${GT}.gross`,
    [
      r('aaj angoor kitne lines me, kitna maal aur kitne ka bika'),
      u('آج انگور کتنی لائنوں میں، کتنا مال اور کتنے کا بکا'),
      e('how much grapes were sold today, in how many lines and for how much'),
    ],
  ),
  z(
    'N05',
    'seller',
    'complex',
    'seller=Karim;item=Grapes;period=today',
    `qty=@d7.sale.karim_grapes_today.qty;amount=@d7.sale.karim_grapes_today.gross;rate=@d7.sale.karim_grapes_today.last_rate`,
    [
      r('karim ka aaj angoor kitna bika, kitne ka aur akhri rate kya tha'),
      u('کریم کا آج انگور کتنا بکا، کتنے کا اور آخری ریٹ کیا تھا'),
      e("how much of Karim's grapes were sold today, for how much, and what was the last rate"),
    ],
  ),
  z(
    'N06',
    'rate',
    'last',
    'item=Potato;period=this week',
    'rate=@d7.sale.potato_week.last_rate;date=@d7.sale.potato_week.last_date',
    [
      r('is hafte aloo ka aakhri rate kya tha'),
      u('اس ہفتے آلو کا آخری ریٹ کیا تھا'),
      e('what was the latest rate of potato this week'),
    ],
  ),
  // R2 and R10: commission taken from a seller on his item is the seller side (the buyer side is named, never added).
  z(
    'N07',
    'item',
    'commission',
    'seller=Rahim;item=Banana;period=last month',
    'seller_comm=@d7.sale.rahim_banana_last_month.comm_seller;buyer_comm?=@d7.sale.rahim_banana_last_month.comm_buyer',
    [
      r('rahim ke kele ka pichle mahine kitna kamishan kata'),
      u('رحیم کے کیلے کا پچھلے مہینے کتنا کمیشن کٹا'),
      e("how much commission was deducted on Rahim's banana last month"),
    ],
  ),
  // R3: a بکری number is a filter (the whole bill), never dropped.
  z(
    'N08',
    'sale',
    'number',
    'bikri=100',
    'gross=@d7.bikri_no.100.gross;qty=@d7.bikri_no.100.qty;lines=@d7.bikri_no.100.lines;date=@d7.bikri_no.100.date',
    [
      r('bikri number 100 me kitna maal bika, kitne ka, kitne lines aur kis tarikh ko'),
      u('بکری نمبر 100 میں کتنا مال بکا، کتنے کا، کتنی لائنیں اور کس تاریخ کو'),
      e('in sale number 100 how much was sold, for how much, how many lines and on which date'),
    ],
  ),
  z('N09', 'sale', 'number', 'bikri=100', 'commission=@d7.bikri_no.100.comm_total', [
    r('bikri number 100 ka total commission kitna tha'),
    u('بکری نمبر 100 کا کل کمیشن کتنا تھا'),
  ]),
  // A بکری that was edited today: its lines are the rows of the edit, counted once.
  z(
    'N10',
    'sale',
    'number',
    'bikri=466',
    'gross=@d7.bikri_no.466.gross;qty=@d7.bikri_no.466.qty;lines=@d7.bikri_no.466.lines',
    [
      r('bikri number 466 ki kul bikri, qty aur lines batao'),
      u('بکری نمبر 466 کی کل بکری، تعداد اور لائنیں بتائیں'),
      e('the total sale, quantity and number of lines of sale number 466'),
    ],
  ),
  // R4: a quality with the last rate.
  z(
    'N11',
    'quality',
    'last',
    'item=Apple;grade=first;period=last month',
    'rate=@ixq.last_month.apple.first.last_rate',
    [
      r('pichle mahine awwal darje ke seb ka aakhri rate kya tha'),
      u('پچھلے مہینے اول درجے کے سیب کا آخری ریٹ کیا تھا'),
      e('what was the last rate of first quality apple last month'),
    ],
  ),
  // R11: every quality of an item, side by side (no cut list to add up).
  z(
    'N12',
    'quality',
    'complex',
    'item=Apple;period=last month',
    'first=@ixq.last_month.apple.first.qty;second=@ixq.last_month.apple.second.qty;third=@ixq.last_month.apple.third.qty',
    [
      r('pichle mahine seb awwal, doyam aur soyam darje ka alag alag kitna bika'),
      u('پچھلے مہینے سیب اول، دوم اور سوم درجے کا الگ الگ کتنا بکا'),
      e('last month how much apple sold in first, second and third quality, each'),
    ],
  ),
  // R5: the same name in two accounts: ask which one, never guess (outcome which_one, no click).
  z(
    'N13',
    'buyer',
    'which',
    'asked=Akram',
    '',
    [u('اکرم کا بقایا کتنا ہے'), r('akram ka bakaya kitna hai'), e('how much does Akram owe')],
    { outcome: 'which_one' },
  ),
  // R6: a packing word. Potatoes come in bags only, so potatoes in boxes are 0 (or the app says it cannot split by packing).
  z(
    'N14',
    'item',
    'packing',
    'item=Potato;packing=Box;period=last month',
    'qty=@d7.pack.potato_box_last_month.qty',
    [
      r('pichle mahine aloo ki peti me kitni bikri hui'),
      u('پچھلے مہینے آلو کی پیٹی میں کتنی بکری ہوئی'),
      e('how many potatoes were sold in boxes last month'),
    ],
    { outcome: 'safe' },
  ),
  z(
    'N15',
    'item',
    'packing',
    'item=Potato;packing=Bag;period=last month',
    'qty=@d7.pack.potato_bag_last_month.qty;amount=@d7.pack.potato_bag_last_month.gross',
    [
      r('pichle mahine aloo ki boriyan kitni bikin aur kitne ki'),
      u('پچھلے مہینے آلو کی بوریاں کتنی بکیں اور کتنے کی'),
      e('how many bags of potato were sold last month and for how much'),
    ],
  ),
  // R7: a voucher number is a filter: that receipt, that payment, that expense.
  z(
    'N16',
    'voucher',
    'number',
    'voucher=receipt 100',
    'amount=@d7.voucher.receipt_100.amount;date=@d7.voucher.receipt_100.date;party!=@d7.voucher.receipt_100.party',
    [
      r('wasooli number 100 kitne ki thi, kis ki aur kab'),
      u('وصولی نمبر 100 کتنے کی تھی، کس کی اور کب'),
      e('how much was receipt number 100, from whom and when'),
    ],
  ),
  z(
    'N17',
    'voucher',
    'number',
    'voucher=payment 50',
    'amount=@d7.voucher.payment_50.amount;date=@d7.voucher.payment_50.date;party!=@d7.voucher.payment_50.party',
    [
      r('adaigi number 50 kitne ki thi, kis ko aur kab'),
      u('ادائیگی نمبر 50 کتنے کی تھی، کس کو اور کب'),
      e('how much was payment number 50, to whom and when'),
    ],
  ),
  // R8: a cash or credit word on a sale: only the credit sales, never the whole day.
  z('N18', 'zero', 'cash', 'pay=credit;period=today', 'credit=@d7.sale.credit_today.gross', [
    r('aaj udhaar me kitni bikri hui'),
    u('آج ادھار میں کتنی بکری ہوئی'),
    e('how much was sold on credit today'),
  ]),
  // R9: uncommon period words. May (مئی) and an Urdu possessive; the commission taken from a buyer is his side.
  z(
    'N19',
    'buyer',
    'period',
    'buyer=Akram Toll Tax;period=May',
    'commission=@d7.sale.akram_toll_tax_may.comm_buyer',
    [
      u('اکرم ٹول ٹیکس کا مئی کا کمیشن'),
      r('akram toll tax ka may ka commission kitna tha'),
      e('how much commission did we take from Akram Toll Tax in May'),
    ],
  ),
  z(
    'N20',
    'buyer',
    'period',
    'buyer=Faisal;period=September',
    'commission=@d7.sale.faisal_sep.comm_buyer',
    [
      rm('faisal se septmber me kitna kamishan liya gaya'),
      um('فیصل سے ستمبر میں كتنا كمیشن لیا گیا'),
      e('how much commission was taken from Faisal in September'),
    ],
  ),
  z('N21', 'period', 'day', 'period=day before yesterday', 'sales=@ix.d2026_11_03.all.gross', [
    r('parso ki bikri kitni thi'),
    u('پرسوں کی بکری کتنی تھی'),
    e('how much was sold the day before yesterday'),
  ]),
  z('N22', 'period', 'all', 'period=up to now', 'sales=@ix.up_to_now.all.gross', [
    rm('ab tak ki kul bikri kitni hui'),
    u('اب تک کی کل بکری کتنی ہوئی'),
    em('total sales up to now'),
  ]),
  // R12: an expense head with a period.
  z('N23', 'expense', 'head', 'head=Tea;period=this year', 'expense=@r29.year.tea.expense', [
    r('is saal chai ka kharcha kitna hua'),
    u('اس سال چائے کا خرچہ کتنا ہوا'),
    e('how much was spent on tea this year'),
  ]),
  // R13: a figure the Owner types ("5000") is never the answer: the real balance is.
  z('N24', 'buyer', 'trap', 'buyer=Faisal', 'balance=@buyer.faisal.balance', [
    r('kya faisal ka bakaya 5000 hai'),
    u('کیا فیصل کا بقایا 5000 ہے'),
    e('is the balance of Faisal 5000'),
  ]),
  // R13b: "kal" with no verb: yesterday (the Urdu "کل" can also mean "total", so both readings are true figures).
  z(
    'N25',
    'period',
    'ambiguous',
    'period=yesterday',
    'commission=@ix.yesterday.all.comm_total|@ix.up_to_now.all.comm_total',
    [r('kal commission kitna hai'), u('کل کمیشن کتنا ہے')],
  ),

  // ---- the richer books (plan 59 B2, seed.rs d7_richer_cases) -----------------------------------------------------------
  // Umar's example with a buyer who has BOTH a cash line and a credit line on the same past day: Naveed Traders took
  // 10 banana for cash and 12 apple on credit on 14-10-2026.
  z(
    'N26',
    'buyer',
    'day',
    'buyer=Naveed Traders;day=2026-10-14',
    `banana_qty=@${NV}.banana.qty;banana_rate=@${NV}.banana.rate;banana_amount=@${NV}.banana.amount;apple_qty=@${NV}.apple.qty;apple_rate=@${NV}.apple.rate;apple_amount=@${NV}.apple.amount;cash=@${NV}.cash;credit=@${NV}.credit;balance=@${NV}.balance_end|@${NV}.balance_now`,
    [
      u(
        'پچھلے مہینے 14 تاریخ کو نوید ٹریڈرز نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار رہا',
      ),
      r(
        'pichle mahine 14 tarikh ko naveed traders ne kya kya khareeda, moqe par kitna naqad diya aur kitna udhaar raha',
      ),
      e(
        'what did Naveed Traders buy on the 14th of last month, how much cash did he pay on the spot and how much stayed on credit',
      ),
    ],
  ),
  z('N27', 'buyer', 'cash', 'buyer=Naveed Traders;day=2026-10-14;pay=cash', `cash=@${NV}.cash`, [
    r('naveed traders ne 14 october ko nakad kitna diya'),
    u('نوید ٹریڈرز نے 14 اکتوبر کو نقد کتنا دیا'),
    e('how much did Naveed Traders pay in cash on 14 October'),
  ]),
  z(
    'N28',
    'buyer',
    'credit',
    'buyer=Naveed Traders;day=2026-10-14;pay=credit',
    `credit=@${NV}.credit`,
    [
      rm('naveed traders ka 14 october ka udhar kitna raha'),
      u('نوید ٹریڈرز کا 14 اکتوبر کا ادھار کتنا رہا'),
      e('how much stayed on credit for Naveed Traders on 14 October'),
    ],
  ),
  z(
    'N29',
    'buyer',
    'cash',
    'buyer=Naveed Traders;item=Banana;pay=cash;day=2026-10-14',
    'qty=@d7.sale.naveed_cash_oct14.qty;amount=@d7.sale.naveed_cash_oct14.gross;rate=@d7.sale.naveed_cash_oct14.last_rate',
    [
      r('naveed traders ne 14 october ko nakad me kela kitna, kitne ka aur kis rate par liya'),
      u('نوید ٹریڈرز نے 14 اکتوبر کو نقد میں کیلا کتنا، کتنے کا اور کس ریٹ پر لیا'),
      e(
        'how much banana did Naveed Traders buy for cash on 14 October, for how much and at what rate',
      ),
    ],
  ),
  // A buyer with his own account (Nadir) owes something (he was a پختہ party's own account before v1.32).
  z('N31', 'buyer', 'short', 'buyer=Nadir', 'balance=@buyer.nadir.balance', [
    r('nadir ka bakaya kitna hai'),
    u('نادر کے ذمے کتنا ہے'),
  ]),
  // The separate khata, short.
  z(
    'N32',
    'khata',
    'short',
    'khata=Family Khata',
    'balance=@d7.khata.family.balance_signed|@d7.khata.family.balance',
    [u('خاندانی کھاتے میں کتنا ہے'), e('how much is in the family khata')],
  ),
  // A city: the dues of its buyers (R21), and how many owe.
  z('N33', 'dues', 'city', 'city=Kasur', 'dues=@d7.city.kasur.dues', [
    r('kasur ke gahakon ke zimme kul kitna baqi hai'),
    u('قصور کے گاہکوں کے ذمے کل کتنا باقی ہے'),
    e('how much do the buyers of Kasur owe in all'),
  ]),
  z('N34', 'dues', 'city', 'city=Kasur', 'buyers=@d7.city.kasur.buyers', [
    r('kasur shehar ke kitne gahakon ka bakaya hai'),
    u('قصور شہر کے کتنے گاہکوں کا بقایا ہے'),
    e('how many buyers of Kasur have something to pay'),
  ]),
  z('N35', 'dues', 'city', 'city=Kasur', 'dues=@r21.kasur.balance;over_8=@r21.kasur.age_8_30', [
    rm('shehar kasur ka bakaya aur usme se 8 se 30 din purana kitna hai'),
    u('شہر قصور کا بقایا اور اس میں سے آٹھ سے تیس دن پرانا کتنا ہے'),
    e('the dues of the city Kasur and how much of it is 8 to 30 days old'),
  ]),
  // A line with ریٹ مختلف: the rate is the seller's 4,300 unless the buyer's is asked (4,350); the buyer's own rate
  // and the difference are figures of their own.
  z(
    'N36',
    'rate',
    'last',
    'item=Apple;day=2026-10-22',
    'rate=@d7.sale.apple_oct22.last_rate;date=@d7.sale.apple_oct22.last_date',
    [
      r('22 october ko seb ka aakhri rate kya tha'),
      u('22 اکتوبر کو سیب کا آخری ریٹ کیا تھا'),
      e('what was the last rate of apple on 22 October'),
    ],
  ),
  z(
    'N37',
    'rate',
    'buyer',
    'buyer=Rafique Brothers;item=Apple;day=2026-10-22',
    'buyer_rate=@d7.ratediff.buyer_rate;qty=@d7.ratediff.qty;amount=@d7.ratediff.buyer_value',
    [
      r(
        'rafique brothers ne 22 october ko seb gahak ke rate par kitne ka liya, kis rate par aur kitna',
      ),
      u('رفیق برادرز نے 22 اکتوبر کو سیب گاہک کے ریٹ پر کتنے کا لیا، کس ریٹ پر اور کتنا'),
      e(
        "at the buyer's rate, what rate and how much apple did Rafique Brothers take on 22 October, and for how much",
      ),
    ],
  ),
  z(
    'N38',
    'rate',
    'difference',
    'period=all',
    'diff=@d7.ratediff.diff;seller_rate=@d7.ratediff.seller_rate;buyer_rate=@d7.ratediff.buyer_rate',
    [
      r(
        'jis line me gahak ka rate alag tha uska farq kitna bana, bayopari ka rate aur gahak ka rate kya tha',
      ),
      u(
        'جس لائن میں گاہک کا ریٹ الگ تھا اس کا فرق کتنا بنا، بیوپاری کا ریٹ اور گاہک کا ریٹ کیا تھا',
      ),
      e(
        'on the line where the buyer rate differed: what was the difference, the seller rate and the buyer rate',
      ),
    ],
  ),
  // A later sale of an older بکری's leftover on a day that also has a newer بکری of the same item: the newest
  // mosambi line of 19-10 is the newer بکری's last line (2,550), never the later sale (2,400, the row id).
  z(
    'N39',
    'rate',
    'last',
    'item=Mosambi;day=2026-10-19',
    'rate=@d7.sale.mosambi_oct19.last_rate;date=@d7.sale.mosambi_oct19.last_date',
    [
      r('19 october ko moosami ka aakhri rate kya laga'),
      u('19 اکتوبر کو موسمی کا آخری ریٹ کیا لگا'),
      e('what was the latest rate of mosambi on 19 October'),
    ],
  ),
  z(
    'N40',
    'item',
    'day',
    'item=Mosambi;day=2026-10-19',
    'lines=@d7.sale.mosambi_oct19.lines;qty=@d7.sale.mosambi_oct19.qty;amount=@d7.sale.mosambi_oct19.gross;max=@d7.sale.mosambi_oct19.max_rate;min=@d7.sale.mosambi_oct19.min_rate',
    [
      r(
        '19 october ko moosami kitne lines me, kitna aur kitne ka bika, sab se mehnga aur sab se sasta rate',
      ),
      u('19 اکتوبر کو موسمی کتنی لائنوں میں، کتنا اور کتنے کا بکا، سب سے مہنگا اور سب سے سستا ریٹ'),
      e(
        'on 19 October how much mosambi sold, in how many lines and for how much, and the highest and lowest rate',
      ),
    ],
  ),
];

// ---- simple questions: ONE figure, a plain wording ------------------------------------------------------------------
//
// The Owner asks most of his questions like these. They are the ones the app should answer by itself with no model
// call (PRD FR-7.90). Two wordings each, so the three languages stay even over the bank.

const SIMPLE = [
  z('S01', 'buyer', 'simple', 'buyer=Nadeem Traders', 'balance=@buyer.nadeem.balance', [
    rm('nadeem trader ka kitna bkaya hai'),
    e('how much does Nadeem Traders owe'),
  ]),
  z('S02', 'seller', 'simple', 'seller=Bashir', 'balance=@seller.bashir.balance', [
    u('بشیر کو کتنی رقم دینی ہے'),
    e('how much do we owe Bashir'),
  ]),
  z('S03', 'buyer', 'simple', 'buyer=Tariq Mehmood', 'balance=@buyer.tariq.balance', [
    rm('tarik mehmod ke zime kitna he'),
    u('طارق محمود کا بقایا کتنا ہے'),
  ]),
  z('S04', 'item', 'simple', 'item=Banana;period=last month', 'gross=@ix.last_month.banana.gross', [
    r('pichle mahine kela kitne ka bika'),
    u('پچھلے مہینے کیلا کتنے کا بکا'),
  ]),
  z('S05', 'item', 'simple', 'item=Mango;period=this year', 'qty=@ix.year.mango.qty', [
    r('is saal aam kitni qty bika'),
    em('how many units of mangoe were sold this year'),
  ]),
  z('S06', 'rate', 'simple', 'item=Banana;period=this month', 'rate=@ix.month.banana.avg_rate', [
    u('اس مہینے کیلے کا اوسط ریٹ کیا ہے'),
    e('what is the average banana rate this month'),
  ]),
  z('S07', 'partner', 'simple', 'partner=Partner A', 'total=@partner.partner_a.total', [
    r('alif ki kul raqam kitni hai'),
    em('what is Partner As totel in the shop'),
  ]),
  z('S08', 'partner', 'simple', 'partner=Usman', 'capital=@partner.usman.capital', [
    u('عثمان نے کتنا سرمایہ لگایا ہے'),
    r('usman ka sarmaya kitna hai'),
  ]),
  z('S09', 'expense', 'simple', 'head=Tea;period=all', 'expense=@r29.all.tea.expense', [
    rm('ab tk chae par kitna kharcha hua'),
    e('how much has been spent on tea so far'),
  ]),
  z('S10', 'lent', 'simple', 'period=now', 'open=@r32.total.open', [
    u('ادھار رقم میں کل کتنی واپس آنی ہے'),
    e('how much of the money lent is still to come back'),
  ]),
  z(
    'S11',
    'cash',
    'simple',
    'period=yesterday',
    'closing=@r12.yesterday.sum.report_closing_balance',
    [rm('kal rat ko roknad kitni thi'), e('what was the cash at the end of yesterday')],
  ),
  z('S12', 'cash', 'simple', 'period=last month', 'received=@mx.last_month.r23.total.amount', [
    u('پچھلے مہینے کل کتنی وصولی ہوئی'),
    rm('pichle mahine kul wasuli kitni hoi'),
  ]),
  z('S13', 'stock', 'simple', 'item=Banana', 'qty=@stock.banana.qty', [
    rm('kele ka kitna mal baki hai'),
    e('how much banana is left in stock'),
  ]),
  z('S14', 'dues', 'simple', 'period=now', 'total=@tiles.sellers_payable', [
    u('بیوپاریوں کو کل کتنی ادائیگی باقی ہے'),
    e('how much is payable to the sellers in all'),
  ]),
  z('S15', 'garden', 'simple', 'garden=Shahpur Garden', 'balance=@r36.shahpur_garden.balance', [
    rm('shahpur bag ka abhi hisab kitna he'),
    u('شاہ پور باغ کا اب تک کا حساب کتنا ہے'),
  ]),
  z('S16', 'fruit', 'simple', 'partner=Partner A', 'value=@k.all.partner_a.value', [
    r('alif ke liye phal ki kul qeemat kitni bani'),
    em('what is the valew of the frut Partner A took'),
  ]),
  z('S17', 'trial', 'simple', 'period=now', 'difference=@r18.sum.report_difference', [
    u('ٹرائل بیلنس میں فرق کتنا ہے'),
    e('is there any difference in the trial balance, how much'),
  ]),
  z('S18', 'summary', 'simple', 'period=today', 'commission=@tiles.commission_today', [
    rm('aj kamishan kitna bna'),
    e('what is the commission today'),
  ]),
  z('S19', 'summary', 'simple', 'period=this month', 'sales=@ix.month.all.gross', [
    u('اس مہینے اب تک کل کتنی بکری ہوئی'),
    e('total sales so far this month'),
  ]),
  z('S20', 'seller', 'simple', 'seller=Tahir', 'open=@r26.tahir.open', [
    rm('tahir ki peshgi abhi kitni khuli he'),
    u('طاہر کی پیشگی ابھی کتنی کھلی ہے'),
  ]),
];

// ---- the common short questions (Umar, 07-11-2026: the mix must match how the shop really asks) ------------------------
//
// One figure, a plain wording, no filter beyond a name or a period. The Owner asks most of his questions like these,
// so the share of questions answered with no model call is honest only when they are in the bank in this number.

const SHORT = [
  z('S21', 'buyer', 'short', 'buyer=Akram Baloch', 'balance=@buyer.akram_baloch.balance', [
    r('akram baloch ka bakaya kitna hai'),
    u('اکرم بلوچ کے ذمے کتنا ہے'),
  ]),
  z('S22', 'buyer', 'short', 'buyer=Arshad Multani', 'balance=@buyer.arshad.balance', [
    u('ارشد ملتانی کا بقایا کتنا ہے'),
    e('how much does Arshad Multani owe'),
  ]),
  z('S23', 'buyer', 'short', 'buyer=Danish Khan', 'balance=@buyer.danish.balance', [
    r('danish khan ke zimme kitna hai'),
    e('what is the balance of Danish Khan'),
  ]),
  z('S24', 'buyer', 'short', 'buyer=Imran Fruit', 'balance=@buyer.imran.balance', [
    rm('imran froot ka hisab kya hai'),
    u('عمران فروٹ کا بقایا کتنا ہے'),
  ]),
  z('S25', 'buyer', 'short', 'buyer=Kashif Ali', 'balance=@buyer.kashif.balance', [
    u('کاشف علی کے ذمے کتنے ہیں'),
    e('how much does Kashif Ali owe us'),
  ]),
  z('S26', 'buyer', 'short', 'buyer=Usman Ghani', 'balance=@buyer.usman.balance', [
    r('usman ghani ka baqaya batao'),
    e('Usman Ghani balance please'),
  ]),
  z('S27', 'buyer', 'short', 'buyer=Waqas Brothers', 'balance=@buyer.waqas.balance', [
    u('وقاص برادرز کا بقایا کتنا ہے'),
    r('waqas brothers se kitna lena hai'),
  ]),
  z('S28', 'buyer', 'short', 'buyer=Zahid Mehmood', 'balance=@buyer.zahid.balance', [
    r('zahid mehmood ka udhaar kitna hai'),
    e('how much is Zahid Mehmood due to pay'),
  ]),
  // A cash buyer owes nothing: the answer is 0.
  z('S29', 'zero', 'short', 'buyer=Shahid Raza', 'balance=@buyer.shahid.balance', [
    u('شاہد رضا کے ذمے کتنا ہے'),
    e('how much does Shahid Raza owe'),
  ]),
  z('S30', 'seller', 'short', 'seller=Karim', 'balance=@seller.karim.balance', [
    r('karim ko abhi kitne paise dene hain'),
    u('کریم کو کتنی ادائیگی باقی ہے'),
  ]),
  z('S31', 'seller', 'short', 'seller=Naseer', 'balance=@seller.naseer.balance', [
    rm('naseer ka hisab kya he'),
    e('how much do we owe Naseer'),
  ]),
  z('S32', 'seller', 'short', 'seller=Jameel', 'balance=@seller.jameel.balance', [
    u('جمیل کو کتنے دینے ہیں'),
    r('jameel ka payable kitna hai'),
  ]),
  z('S33', 'seller', 'short', 'seller=Saleem', 'balance=@seller.saleem.balance', [
    r('saleem ko kitna dena hai'),
    e('amount payable to Saleem'),
  ]),
  z('S34', 'seller', 'short', 'seller=Ghulam Rasool', 'balance=@seller.ghulam_rasool.balance', [
    u('غلام رسول کو کتنی رقم دینی ہے'),
    e('how much do we owe Ghulam Rasool'),
  ]),
  z('S35', 'seller', 'short', 'seller=Haji Nazeer', 'balance=@seller.haji_nazeer.balance', [
    r('haji nazeer ko kitna dena hai'),
    u('حاجی نذیر کو کتنا دینا ہے'),
  ]),
  z('S36', 'seller', 'short', 'seller=Sarfraz', 'balance=@seller.sarfraz.balance', [
    rm('sarfaraz ka baqaya kitna he'),
    e('how much do we owe Sarfraz'),
  ]),
  z('S37', 'summary', 'short', 'period=yesterday', 'sales=@ix.yesterday.all.gross', [
    r('kal ki bikri kitni thi'),
    u('کل کی بکری کتنی ہوئی تھی'),
  ]),
  z('S38', 'summary', 'short', 'period=this week', 'sales=@ix.week.all.gross', [
    u('اس ہفتے کی کل بکری کتنی ہے'),
    e('how much was sold this week'),
  ]),
  z('S39', 'summary', 'short', 'period=last month', 'sales=@ix.last_month.all.gross', [
    r('pichle mahine ki kul bikri kitni rahi'),
    e('total sales of last month'),
  ]),
  z('S40', 'summary', 'short', 'period=this year', 'sales=@ix.year.all.gross', [
    u('اس سال کی کل بکری کتنی ہوئی'),
    r('is saal ki bikri kitni hui'),
  ]),
  z('S41', 'summary', 'short', 'period=yesterday', 'commission=@ix.yesterday.all.comm_total', [
    r('kal ka kamishan kitna bana tha'),
    e('how much commission was made yesterday'),
  ]),
  z('S42', 'summary', 'short', 'period=last month', 'commission=@ix.last_month.all.comm_total', [
    u('پچھلے مہینے کا کل کمیشن کتنا بنا'),
    e('total commission of last month'),
  ]),
  z('S43', 'item', 'short', 'item=Apple;period=last month', 'gross=@ix.last_month.apple.gross', [
    r('pichle mahine seb kitne ka bika'),
    u('پچھلے مہینے سیب کتنے کا بکا'),
  ]),
  z('S44', 'item', 'short', 'item=Mosambi;period=this year', 'qty=@ix.year.mosambi.qty', [
    r('is saal moosami kitni qty bik gayi'),
    e('how many units of mosambi were sold this year'),
  ]),
  z('S45', 'item', 'short', 'item=Potato;period=this month', 'gross=@ix.month.potato.gross', [
    u('اس مہینے آلو کتنے کے بکے'),
    e('how much potato sold this month'),
  ]),
  z('S46', 'item', 'short', 'item=Tomato;period=last month', 'qty=@ix.last_month.tomato.qty', [
    r('pichle mahine tamatar kitne nag bika'),
    e('how many units of tomato were sold last month'),
  ]),
  z('S47', 'rate', 'short', 'item=Apple;period=last month', 'rate=@ix.last_month.apple.avg_rate', [
    u('پچھلے مہینے سیب کا اوسط ریٹ کیا رہا'),
    r('pichle mahine seb ka ausat rate kya tha'),
  ]),
  z('S48', 'stock', 'short', 'item=Mango', 'qty=@stock.mango.qty', [
    r('aam ka kitna maal bacha hai'),
    e('how much mango is left in stock'),
  ]),
  z('S49', 'stock', 'short', 'item=Apple', 'qty=@stock.apple.qty', [
    u('سیب کا کتنا مال باقی ہے'),
    r('seb ka kitna stock baqi hai'),
  ]),
  z('S50', 'partner', 'short', 'partner=Partner B', 'total=@partner.partner_b.total', [
    r('b partner ki kul raqam kitni hai'),
    e("what is Partner B's total in the shop"),
  ]),
  z('S51', 'partner', 'short', 'partner=Usman', 'share=@partner.usman.year_share', [
    u('عثمان کا اس سال کا حصہ کتنا بنا'),
    e("what is Usman's share of profit this year"),
  ]),
  z('S52', 'partner', 'short', 'partner=Partner B', 'share=@partner.partner_b.month_share', [
    rm('b ka is mahine ka hisa kitna bana'),
    u('ب کا اس مہینے کا حصہ کتنا ہے'),
  ]),
  z('S53', 'expense', 'short', 'head=Shop Rent;period=all', 'expense=@r29.all.shop_rent.expense', [
    r('ab tak dukaan ke kiraye par kitna kharcha hua'),
    e('how much has been spent on shop rent so far'),
  ]),
  z('S54', 'expense', 'short', 'head=Salaries;period=all', 'expense=@r29.all.salaries.expense', [
    u('اب تک تنخواہوں پر کتنا خرچ ہوا'),
    e('how much has been spent on salaries so far'),
  ]),
  z('S55', 'expense', 'short', 'period=this year', 'expense=@r29.year.total.expense', [
    r('is saal dukaan ka kul kharcha kitna hua'),
    u('اس سال دکان کا کل خرچہ کتنا ہوا'),
  ]),
  z('S56', 'advance', 'short', 'period=all', 'given=@r26.total.given', [
    r('bayopariyon ko ab tak kul kitni peshgi di'),
    e('how much advance has been given to sellers in all'),
  ]),
  z('S57', 'lent', 'short', 'period=now', 'overdue=@tiles.lent_overdue', [
    u('ادھار رقم میں سے کتنی کی تاریخ گزر چکی'),
    e('how much of the money lent is past its return date'),
  ]),
  z('S58', 'garden', 'short', 'garden=Shahpur Garden', 'rent_due=@r36.shahpur_garden.rent_due', [
    r('shahpur baagh ka thekah abhi kitna baqi hai'),
    e('how much rent is still due for Shahpur garden'),
  ]),
  z(
    'S59',
    'garden',
    'short',
    'garden=Shahpur Garden',
    'installments=@r36.shahpur_garden.installments',
    [
      u('شاہ پور باغ کی کل کتنی قسطیں دی گئیں'),
      e('how much has been paid in installments for Shahpur garden'),
    ],
  ),
  z('S60', 'trip', 'short', 'trip=0071', 'result=@trip2.result', [
    r('chalu safar ka ab tak nafa nuqsan kitna hai'),
    u('چالو سفر کا اب تک نفع نقصان کتنا ہے'),
  ]),
  z('S61', 'fruit', 'short', 'period=all', 'value=@r37.all.total.value', [
    u('اب تک پھل کی کل قیمت کتنی بنی'),
    e('what is the total value of the fruit taken so far'),
  ]),
  z('S62', 'cash', 'short', 'period=this month', 'closing=@r12.month.sum.report_closing_balance', [
    r('is mahine ke akhir me roknad kitni hai'),
    e('what is the cash balance of the cash book this month'),
  ]),
  z('S63', 'trial', 'short', 'period=now', 'debit=@r18.total.debit', [
    u('ٹرائل بیلنس کا بنام کل کتنا ہے'),
    e('what is the total debit of the trial balance'),
  ]),
  z('S64', 'dues', 'short', 'period=now', 'age_8_30=@r20.total.age_8_30', [
    r('aath se tees din purane bakaye kitne hain'),
    e('how much of the dues is 8 to 30 days old'),
  ]),
  z('S65', 'summary', 'short', 'period=today', 'bills=@tiles.count_today', [
    r('aaj kitne bill bane'),
    u('آج کتنے بل بنے'),
  ]),
  z('S66', 'cash', 'short', 'period=yesterday', 'cash_out=@r12.yesterday.total.cash_out', [
    u('کل روکڑ میں سے کتنا گیا'),
    e('how much cash went out yesterday'),
  ]),
  z('S67', 'cash', 'short', 'period=last month', 'paid=@mx.last_month.r25.total.amount', [
    r('pichle mahine bayopariyon ko kul kitna diya'),
    e('how much was paid to sellers last month'),
  ]),
  // Nothing was received this week: 0.
  z('S68', 'zero', 'short', 'period=this week', 'received=@mx.week.r23.total.amount', [
    r('is hafte gahakon se kitni wasooli hui'),
    u('اس ہفتے گاہکوں سے کتنی وصولی ہوئی'),
  ]),
  z('S69', 'seller', 'short', 'seller=Karim;period=last month', 'gross=@b.last_month.karim.gross', [
    u('کریم نے پچھلے مہینے کتنا مال بیچا'),
    e('how much did Karim sell last month'),
  ]),
  z('S70', 'seller', 'short', 'seller=Bashir;period=last month', 'net=@b.last_month.bashir.net', [
    r('bashir ki pichle mahine ki khalis raqam kitni bani'),
    e("what was Bashir's net last month"),
  ]),
];

// ---- the size of the bank --------------------------------------------------------------------------------------
//
// The lead asked for about 200 questions. Groups that only repeat what another group already proves are left out
// (kept in the file for the next round), and the simplest groups keep two of their three wordings, in turn the
// Roman and Urdu, the Urdu and English and the Roman and English ones, so each language stays about even.

const LEFT_OUT = new Set([
  'T06', // aging buckets of one buyer: T05 (oldest bill) and T38 and T39 (buckets) cover the kind
  'T42', // advances of all sellers: T13 covers the kind
  'T54', // counts of bills: T53 and T90 cover the kind
  'T58', // trip goods tile: T77 covers the kind
  'T60', // received, sold, written off: T55 to T57 and T59 cover the kind
  'T78', // a closed trip: T77 covers the kind
  'T80', // garden paid by shop or pocket: T79 covers the kind
]);

const TWO_WORDINGS = [
  'T12',
  'T23',
  'T28',
  'T30',
  'T36',
  'T37',
  'T38',
  'T40',
  'T43',
  'T46',
  'T51',
  'T52',
  'T55',
  'T56',
  'T59',
  'T65',
  'T70',
  'T71',
  'T72',
  'T73',
  'T85',
  'T86',
  'T89',
  'T90',
  'T04',
  'T05',
  'T08',
  'T10',
  'T11',
  'T13',
  'T15',
  'T16',
  'T17',
  'T21',
  'T22',
  'T24',
  'T25',
  'T26',
  'T29',
  'T32',
  'T33',
  'T34',
  'T41',
  'T44',
  'T47',
  'T53',
  'T57',
  'T61',
  'T62',
];
const PAIRS = [
  [0, 1],
  [1, 2],
  [0, 2],
];

let turn = 0;
/** The two wordings a group keeps: the turn's pair, but never without its misspelt wording (about 15% are misspelt). */
const pairFor = (g) => {
  const pair = [...PAIRS[turn++ % PAIRS.length]];
  const mis = g.q.findIndex((w) => w[2] === 'mis');
  if (mis >= 0 && !pair.includes(mis)) pair[0] = mis;
  return pair.sort((a, b) => a - b);
};
export const D7 = [...ALL, ...FINDINGS, ...SIMPLE, ...SHORT]
  .filter((g) => !LEFT_OUT.has(g.g))
  .map((g) => (TWO_WORDINGS.includes(g.g) ? { ...g, use: pairFor(g) } : g));

// ---- every kind of record and figure the app keeps -------------------------------------------------------------
//
// From the PRD (Sections 6 and 10 to 22, the report catalogue R1 to R38 and the analytics views A to K) and the
// tables of Architecture Section 15. For each kind: where it comes from, which groups of this bank ask it, and
// when it is not asked, why. `build-bank-d7.mjs` turns the group keys into question ids in
// `realtime-d7-coverage.md`, and fails if a kind has neither a group nor a reason.

export const COVERAGE = [
  // ---- the people ------------------------------------------------------------------------------------------------
  {
    kind: 'Sellers: sold, bills, net, commission taken from them',
    from: 'PRD 10, 12, R6, R27, view B; accounts, bikri_lots',
    groups: ['T10', 'T11', 'T15', 'T16', 'T17', 'T19'],
  },
  {
    kind: 'A seller on one day: items, qty, rate, amount',
    from: 'PRD 12, view A with a seller; bikri_lots, bikri_sales',
    groups: ['T14'],
  },
  {
    kind: 'Seller bill deductions: commission, freight, labour, munshiana, phone, cash',
    from: 'PRD 12.5, C6, R6; bikri_lots',
    groups: ['T18'],
  },
  {
    kind: 'Seller balance and payables (what the shop owes)',
    from: 'PRD 15.3, FR-7.59, R24; journal_lines',
    groups: ['T12', 'T20', 'S02', 'S14'],
  },
  {
    kind: 'Seller advances: given, adjusted, open',
    from: 'PRD 15.4, FR-15.8, R26; advance_allocations',
    groups: ['T13', 'S20'],
  },
  {
    kind: 'Buyers: bought, by item, by day, with commission and laga',
    from: 'PRD 12.4, view C, R5, R35; bikri_sales',
    groups: ['T01', 'T02', 'T04', 'T07', 'T75'],
  },
  {
    kind: 'A buyer on one day: cash on the spot, credit left, balance (the day sheet)',
    from: 'PRD FR-19.12, R35; bikri_sales, journal_lines',
    groups: ['T01', 'T02', 'T03'],
  },
  {
    kind: 'Buyer balance, oldest open bill, days, aging buckets',
    from: 'PRD 15.2, FR-15.4, C13, C14, R20; bill_allocations',
    groups: ['T05', 'T38', 'T39', 'S01', 'S03'],
  },
  {
    kind: 'Receipts from a buyer (وصولی) and how many',
    from: 'PRD 15.2, R23; vouchers',
    groups: ['T08', 'T46', 'T47', 'S12'],
  },
  {
    kind: 'Payments to sellers (ادائیگی) and how many',
    from: 'PRD 15.3, R25; vouchers',
    groups: ['T46', 'T47'],
  },
  {
    kind: 'Cash and credit sales (نقدی، ادھاری), cash against credit',
    from: 'PRD 12.4, R3, R4; bikri_sales.payment_type',
    groups: ['T01', 'T02', 'T68', 'T69', 'T72', 'T75'],
  },
  {
    kind: 'The dead amount (ڈوبی رقم): who, owed then, still owed',
    from: 'PRD FR-10.10, R38; dead_marks',
    groups: ['T09', 'T40'],
  },
  {
    kind: 'Collectors and the collection list (اگراہی)',
    from: 'PRD 15.6, R22; staff, accounts',
    groups: ['T39', 'T88'],
  },
  // ---- the sale itself --------------------------------------------------------------------------------------------
  {
    kind: 'Sale entries (بکری): number, date, truck, count',
    from: 'PRD 12.2, 12.7; bikri_entries',
    groups: ['T87', 'T90', 'T53', 'T17'],
  },
  {
    kind: 'Lots: goods that came in, by item and period',
    from: 'PRD 12.3; bikri_lots',
    groups: ['T59'],
  },
  {
    kind: 'Sale lines: item, qty, rate, amount, commission, laga, total',
    from: 'PRD 12.4, C1 to C5; bikri_sales',
    groups: ['T01', 'T07', 'T14', 'T15', 'T16', 'T61', 'T65'],
  },
  {
    kind: 'Quality of a sale line (اول، دوم، سوم)',
    from: 'PRD FR-12.42; bikri_sales.grade',
    groups: ['T63', 'T67', 'T74'],
  },
  {
    kind: 'Trucks (ٹرک): times, last day, gross, net',
    from: 'PRD 12.2, FR-7.49; bikri_entries',
    groups: ['T87'],
  },
  {
    kind: 'Commission: seller side against buyer side, laga',
    from: 'PRD 11, 12.4; bikri_sales',
    groups: ['T11', 'T15', 'T65', 'T61', 'T26'],
  },
  {
    kind: 'Commission and charge rules (what wins)',
    from: 'PRD 11.1 to 11.3; charge_rules',
    groups: ['T85', 'T86'],
  },
  {
    kind: 'Items and item groups (phal, sabzi)',
    from: 'PRD 10.3, 18, views A, D, H; items',
    groups: ['T61', 'T62', 'T66', 'T76', 'T56', 'T73', 'S04', 'S05'],
  },
  {
    kind: 'Rates: last, average, highest, lowest',
    from: 'PRD FR-7.53, C18, view A; bikri_sales',
    groups: ['T64', 'T16', 'T07', 'T63', 'T14', 'S06'],
  },
  {
    kind: 'Stock (باقی مال): by item, by group, waiting days, lots',
    from: 'PRD 12.8, R10; bikri_lots, stock_moves',
    groups: ['T55', 'T56', 'T57', 'S13'],
  },
  // ---- money and books --------------------------------------------------------------------------------------------
  {
    kind: 'Cash in hand and the cash book (R12)',
    from: 'PRD 16.1, R12; journal_lines on 01010001',
    groups: ['T43', 'T44', 'T45', 'S11'],
  },
  {
    kind: 'Day book (روزنامچہ) and journal lines',
    from: 'PRD 6, R11; journal_entries, journal_lines',
    groups: ['T48'],
  },
  {
    kind: 'Trial balance, opening trial, account index',
    from: 'PRD R16, R18, R19; accounts, journal_lines',
    groups: ['T49', 'T50', 'T51', 'S17'],
  },
  {
    kind: 'Day summary of today: sales, bills, commission (the tiles, R34)',
    from: 'PRD 9.1, R34; dashboard tiles',
    groups: ['T53', 'S18', 'S19'],
    note: 'R34 reads the same books as these tiles; its cash lines are the cash book (T44, T45).',
  },
  {
    kind: 'Dues tiles: buyers, sellers, total dues (اگراہی)',
    from: 'PRD 9.1, FR-19.9, R20, R24; dashboard tiles',
    groups: ['T36', 'T37', 'T20', 'S14'],
  },
  { kind: 'Open advances tile', from: 'PRD 9.1, FR-15.8; advance_allocations', groups: ['T13'] },
  {
    kind: 'Day state: closed up to, open days; attention list',
    from: 'PRD 6.3, FR-6.6, FR-7.44, 21; day_state',
    groups: ['T52', 'T41', 'T57'],
  },
  { kind: 'Audit log of an entry', from: 'PRD FR-21.3; audit_log', groups: ['T90'] },
  // ---- partners and the partners' money ---------------------------------------------------------------------------
  { kind: 'Partner shares in percent', from: 'PRD 17.1; partner_shares', groups: ['T24'] },
  {
    kind: 'Partner profit share: income share, expense share, profit',
    from: 'PRD 17.2, 17.3, C12, R30, view F; partner_share_rows',
    groups: ['T21', 'T26'],
  },
  {
    kind: 'Partner drawings taken and returned',
    from: 'PRD 16.2, 16.3, C20, R29; vouchers, journal_lines',
    groups: ['T22', 'T29', 'T27'],
  },
  {
    kind: 'Partner capital in and out; partner total (کل رقم)',
    from: 'PRD 17.4, 17.9, C22, R31; journal_lines, partners',
    groups: ['T23', 'T24', 'S07', 'S08'],
  },
  {
    kind: 'Partner gardens, trips and fruit taken',
    from: 'PRD 12.10, 12.11, 16.7, view F; ventures, fruit_takes',
    groups: ['T25'],
  },
  {
    kind: 'Who took the most (a ranking over the partners)',
    from: 'PRD 17.3; partner_share_rows',
    groups: ['T27'],
  },
  // ---- expenses, loans ---------------------------------------------------------------------------------------------
  {
    kind: 'Shop expenses by head and period; expense heads',
    from: 'PRD 16.2, R29; expense_heads, vouchers',
    groups: ['T28', 'T29', 'T30', 'T31', 'T32', 'T89', 'S09'],
  },
  {
    kind: 'Money lent (ادھار رقم): lent, returned, open, return-by',
    from: 'PRD 16.5, FR-16.4, R32; vouchers, journal_lines',
    groups: ['T33', 'T34', 'S10'],
  },
  {
    kind: 'الگ کھاتہ (separate khata)',
    from: 'PRD 16.6; accounts of control 0305',
    groups: ['T35'],
    note: 'The rich year has the Family Khata with three entries (seed.rs d7_richer_cases).',
  },
  // ---- trips, gardens, fruit ----------------------------------------------------------------------------------------
  {
    kind: 'Market trips and their purchases (stock, cost, result)',
    from: 'PRD 12.10, R33, view J; ventures, purchases',
    groups: ['T77'],
  },
  {
    kind: 'Gardens: rent, installments, due, balance',
    from: 'PRD 12.11, R36, view I; ventures, venture_entries',
    groups: ['T79', 'S15'],
  },
  {
    kind: 'Fruit taken (پھل): by partner, for guests, from other shops',
    from: 'PRD 16.7, R37, view K; fruit_takes',
    groups: ['T81', 'T82', 'T83', 'S16'],
  },
  // ---- masters ------------------------------------------------------------------------------------------------------
  { kind: 'Staff (munshi, collector, labour)', from: 'PRD 10.3; staff', groups: ['T88'] },
  {
    kind: 'Packings, stores, marks, expense heads',
    from: 'PRD 10.3; packings, stores, markas, expense_heads',
    groups: ['T89'],
  },
  // ---- the findings of the reviewer study (plan 59 Section 5.5) -------------------------------------------------------
  {
    kind: 'The newest sale line and the last rate by the lead rule (own date, then بکری number, then line number; the seller rate)',
    from: 'plan 59 Section 11, R1, S-A2; bikri_sales, bikri_entries',
    groups: ['N01', 'N02', 'N03', 'N05', 'N06', 'T64'],
    note: 'The rich year has grapes only in the last three days: a later sale of an older بکری, an edit and a day of several lines, built so the row id gives another rate (seed.rs d7_last_rate_cases).',
  },
  {
    kind: 'A day with several lines, an edited بکری and a later sale: every line counts once',
    from: 'PRD 12.9, FR-12.17, FR-12.19; bikri_sales.is_active',
    groups: ['N04', 'N10'],
  },
  {
    kind: 'Commission taken from a seller on his item (the seller side, never both sides added)',
    from: 'plan 59 R2, R10; bikri_sales',
    groups: ['N07'],
  },
  {
    kind: 'A بکری by its number (the whole bill)',
    from: 'plan 59 R3; bikri_entries.number',
    groups: ['N08', 'N09', 'N10'],
  },
  {
    kind: 'A voucher by its number (receipt, payment)',
    from: 'plan 59 R7; vouchers.number',
    groups: ['N16', 'N17'],
  },
  {
    kind: 'Quality: the last rate, and every quality side by side',
    from: 'plan 59 R4, R11; bikri_sales.grade',
    groups: ['N11', 'N12'],
  },
  {
    kind: 'The same name in two accounts: ask which one, never guess',
    from: 'PRD FR-7.77, E82; plan 59 R5',
    groups: ['N13'],
  },
  {
    kind: 'An item in one packing (potatoes in boxes are 0)',
    from: 'plan 59 R6; bikri_lots.packing_id',
    groups: ['N14', 'N15'],
  },
  {
    kind: 'Cash or credit word on a sale, never the whole day',
    from: 'plan 59 R8; bikri_sales.payment_type',
    groups: ['N18'],
  },
  {
    kind: 'Uncommon words for a period (مئی, septmber, parso, ab tak)',
    from: 'plan 59 R9, R13b',
    groups: ['N19', 'N20', 'N21', 'N22', 'N25'],
  },
  { kind: 'An expense head with a period', from: 'plan 59 R12; R29', groups: ['N23'] },
  { kind: 'A number the Owner types is never the answer', from: 'plan 59 R13', groups: ['N24'] },
  // ---- the richer books (plan 59 B2) ---------------------------------------------------------------------------------
  {
    kind: 'A day with cash AND credit lines of one buyer: on the spot, credit left, balance',
    from: 'PRD FR-19.12, R35; bikri_sales.payment_type, journal_lines',
    groups: ['N26', 'N27', 'N28', 'N29'],
  },
  {
    kind: 'A buyer with his own account and a balance (Nadir)',
    from: 'PRD FR-10.5, R20; accounts',
    groups: ['N31'],
  },
  {
    kind: 'A separate khata (الگ کھاتہ) with entries: paid out, received, the balance',
    from: 'PRD 16.6, FR-16.6; vouchers khata_out and khata_in',
    groups: ['T35', 'N32'],
  },
  {
    kind: 'A city on accounts: the dues of its buyers (R21), how many owe',
    from: 'PRD 10.3, FR-15.6, R21; cities, accounts.city_id',
    groups: ['N33', 'N34', 'N35'],
  },
  {
    kind: 'A sale line with ریٹ مختلف: seller rate, buyer rate, the rate difference',
    from: 'PRD FR-12.3, C19, FR-7.53; bikri_sales.buyer_rate_paisa, rate_diff_paisa',
    groups: ['N36', 'N37', 'N38'],
  },
  {
    kind: 'A later sale of an older بکری on a day with a newer بکری of the same item',
    from: 'PRD FR-12.17; plan 59 Section 11',
    groups: ['N39', 'N40'],
  },
  // ---- zero days and zero filters -----------------------------------------------------------------------------------
  {
    kind: 'Zero: nothing happened (a day, a filter, an item) is 0, never "not found"',
    from: 'PRD 23 (E24 family), plan 59 Section 1',
    groups: ['T19', 'T30', 'T46', 'T70', 'T71', 'T72', 'T73', 'T45'],
  },
  // ---- not asked, with the reason -----------------------------------------------------------------------------------
  {
    kind: 'Cities and اڈے (truck stands)',
    from: 'PRD 10.3; cities, addas',
    groups: [],
    reason:
      'The rich year gives no figure for them (no buyer has a city, one adda row). The team bank R8 questions cover the list.',
  },
  {
    kind: 'Settings, firm details, user preferences, translit pairs, backup log, the question log',
    from: 'PRD 21, 20, 7.84; settings, user_prefs, translit_pairs, backup_log, assistant_questions',
    groups: [],
    reason:
      'Not figures of the books. The assistant answers them with help or a screen button (category D of the team bank), not with a real-time figure.',
  },
  {
    kind: 'Month close, drafts, edit windows, bill and advance allocation rows themselves',
    from: 'PRD 17.6, 7.9, 7.70; month_closes, drafts, edit_windows',
    groups: [],
    reason:
      'Working tables. Their figures are asked through the aging (T05, T38), the advance (T13) and the day state (T52).',
  },
];

export default D7;
