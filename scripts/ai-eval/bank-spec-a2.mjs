// Facet A2 of the team bank (plan 55 Section 10, lead 05-11-2026): 120 more real-time figure
// questions, "asked any way". Umar wants the most focus on figures, with accuracy and speed.
//
// 40 groups of 3 wordings. Every group is one meaning in three wordings that must reach the same
// figure (NFR-AI-2). Figures come from the oracle only (`@key`), never typed here. The wordings vary
// on purpose: very short, with no question word, long, polite, rude, Roman Urdu, Urdu letters,
// English, Roman Urdu with English words, Urdu with English words, and misspelled.
//
// Figure kinds: a buyer's balance (dues), a seller's payable, an advance, the shop's cash, total
// dues, total payable, a day or a period's sale and commission (today, yesterday, this week, this
// month, last month, a named month, a day, this year, up to now), an item, a seller, a buyer, goods
// left, a partner's total. They are the figures the app answers by itself without a model call
// (PRD FR-7.90), so the speed lines in run.mjs read this facet apart.
//
// The same format as bank-spec.mjs; `cat` is added by that file (all are category A).

const r = (text) => ['roman', text];
const rm = (text) => ['roman', text, 'mis'];
const u = (text) => ['urdu', text];
const um = (text) => ['urdu', text, 'mis'];
const e = (text) => ['en', text];
const em = (text) => ['en', text, 'mis'];

/** One group of facet A2: `facet=A2` in the filters, and the key of its figure family. */
const z = (g, section, kind, filters, figures, q, over = {}) => ({
  g,
  section,
  kind,
  world: 'year',
  filters: `facet=A2;${filters}`,
  figures,
  source: 'plan 55 Section 10',
  q,
  ...over,
});

const A2 = [
  // ---- a buyer's balance (what he owes): 12 buyers, one is at zero -------------------------------
  z('Z01', 'buyer', 'buyer', 'buyer=Faisal', 'balance=@buyer.faisal.balance', [
    r('faisal ka bakaya'),
    u('فیصل کے ذمے کتنا ہے'),
    e('what does Faisal owe'),
  ]),
  z('Z02', 'buyer', 'buyer', 'buyer=Imran Fruit', 'balance=@buyer.imran.balance', [
    r('imran fruit se kitna lena hai'),
    u('عمران فروٹ کا بقایا بتائیں'),
    em('imran froot balance?'),
  ]),
  z('Z03', 'buyer', 'buyer', 'buyer=Tariq Mehmood', 'balance=@buyer.tariq.balance', [
    rm('oye tariq mehmod ka hisab jaldi bata'),
    u('طارق محمود نے کتنے پیسے دینے ہیں'),
    e('how much does Tariq Mehmood still owe us'),
  ]),
  z('Z04', 'buyer', 'buyer', 'buyer=Zahid Mehmood', 'balance=@buyer.zahid.balance', [
    r('zahid mehmood ka udhar kitna chal raha hai'),
    u('زاہد محمود کا ادھار'),
    e('Zahid Mehmood outstanding'),
  ]),
  z('Z05', 'buyer', 'buyer', 'buyer=Kashif Ali', 'balance=@buyer.kashif.balance', [
    rm('kashif ali ka bkaya kitna hy'),
    u('کاشف علی کا بقایا کتنا ہے'),
    e('could you please tell me how much Kashif Ali owes'),
  ]),
  z('Z06', 'buyer', 'buyer', 'buyer=Akram Baloch', 'balance=@buyer.akram_baloch.balance', [
    r('akram baloch ka bakaya'),
    u('اکرم بلوچ کا بقایا کتنا ہے'),
    e('akram baloch balance'),
  ]),
  z('Z07', 'buyer', 'buyer', 'buyer=Akram Toll Tax', 'balance=@buyer.akram_toll_tax.balance', [
    rm('akrm tol tax ka hisab batao'),
    u('اکرم ٹول ٹیکس کا بقایا'),
    e('how much is Akram Toll Tax due to pay'),
  ]),
  z('Z09', 'buyer', 'buyer', 'buyer=Waqas Brothers', 'balance=@buyer.waqas.balance', [
    r('waqas brothers ka baqaya kya hai'),
    u('وقاص برادرز کا بقایا'),
    em('wakas brothrs balance'),
  ]),
  z('Z12', 'buyer', 'buyer', 'buyer=Rizwan Butt', 'balance=@buyer.rizwan.balance', [
    r('rizwan butt ka kuch baqi hai'),
    u('رضوان بٹ کے ذمے کچھ ہے'),
    e('does Rizwan Butt owe anything'),
  ]),

  // ---- a seller's payable: what the shop owes him -------------------------------------------------
  z('Z13', 'seller', 'seller', 'seller=Rahim', 'balance=@seller.rahim.balance', [
    rm('rahim ko kitna dena he'),
    u('رحیم کو کتنے پیسے دینے ہیں'),
    e('how much do we owe Rahim'),
  ]),
  z('Z14', 'seller', 'seller', 'seller=Karim', 'balance=@seller.karim.balance', [
    rm('kareem ka hisab?'),
    u('کریم کو کتنا دینا ہے'),
    e('karim payable'),
  ]),
  z('Z15', 'seller', 'seller', 'seller=Haji Nazeer', 'balance=@seller.haji_nazeer.balance', [
    rm('haji nazir ki paymnt kitni banti he'),
    u('حاجی نذیر کو کتنی ادائیگی کرنی ہے'),
    e('amount payable to Haji Nazeer'),
  ]),
  z('Z16', 'seller', 'seller', 'seller=Bashir', 'balance=@seller.bashir.balance', [
    r('bashir ko kitna dena hai'),
    u('بشیر کو کتنے دینے ہیں'),
    e('how much to pay Bashir'),
  ]),
  z(
    'Z17',
    'seller',
    'seller',
    'seller=Tahir',
    // He has an open advance of 30,000: the payable is 966,624, the balance after the advance
    // 936,624. Each is a true figure of the app (R24).
    'balance=@seller.tahir.balance|@r24.tahir.payable',
    [r('tahir ko kitna dena hai'), u('طاہر کو کتنے پیسے دینے ہیں'), e('what do we owe Tahir')],
  ),
  z('Z19', 'seller', 'dues', 'seller=Tahir', 'advance=@r24.tahir.advance_open', [
    r('tahir ne kitni peshgi li hui hai'),
    u('طاہر کی کتنی پیشگی کھلی ہے'),
    e('how much advance is open for Tahir'),
  ]),

  // ---- the shop: cash, dues, payables, advances -------------------------------------------------
  z('Z20', 'dues', 'dues', 'view=tiles', 'advance=@tiles.advances_open', [
    r('sab ki peshgi kitni khuli hai'),
    u('کل پیشگی کتنی باقی ہے'),
    e('total open advances'),
  ]),
  z('Z21', 'cash', 'cash', 'view=tiles', 'cash=@tiles.cash', [
    rm('rokad kitni he abhi'),
    u('ہاتھ میں کتنی نقدی ہے'),
    e('cash in hand?'),
  ]),
  z('Z22', 'dues', 'dues', 'view=R20', 'total=@tiles.buyers_dues|@tiles.total_dues', [
    r('total bakaya sab buyers ka'),
    u('کل وصولی باقی کتنی ہے'),
    e('total dues from buyers'),
  ]),
  z(
    'Z23',
    'dues',
    'dues',
    'view=R24',
    // The tile counts every seller credit; R24 shows the balance after the open advance. Two true
    // figures of the app.
    'payable=@tiles.sellers_payable|@r24.total.balance',
    [
      r('byopariyon ko kul kitna dena hai'),
      u('سب بیوپاریوں کو کتنا دینا ہے'),
      e('total payable to sellers'),
    ],
  ),

  // ---- the shop's sale and commission by period --------------------------------------------------
  z(
    'Z24',
    'period',
    'period',
    'period=today',
    'gross=@tiles.sales_today;count?=@tiles.count_today',
    [r('aaj ki bikri kitni hui'), u('آج کتنی بکری ہوئی'), e('sales today?')],
  ),
  z('Z25', 'period', 'period', 'period=today', 'comm=@tiles.commission_today', [
    r('today ka commission kitna bana'),
    u('آج کا کمیشن بتاؤ'),
    e("what is today's commission"),
  ]),
  z('Z26', 'period', 'period', 'period=yesterday', 'gross=@a.yesterday.total.gross', [
    rm('kl kitni bikri hui thi'),
    u('کل کی بکری کتنی تھی'),
    e('yesterday sales'),
  ]),
  z('Z27', 'period', 'period', 'period=yesterday', 'comm=@a.yesterday.total.total_comm', [
    r('kal ka commission'),
    u('کل کتنا کمیشن بنا'),
    e('commission yesterday please'),
  ]),
  z('Z28', 'period', 'period', 'period=this week', 'gross=@a.week.total.gross', [
    r('is hafte ki bikri'),
    u('اس ہفتے کتنی بکری ہوئی'),
    em('how much did we sel this week'),
  ]),
  z('Z29', 'period', 'period', 'period=this week', 'comm=@a.week.total.total_comm', [
    r('is hafte commission kitna aya'),
    u('اس week کا commission'),
    e('commission this week'),
  ]),
  z('Z30', 'month', 'period', 'period=this month', 'gross=@a.month.total.gross', [
    r('is month ki sale kitni hai'),
    u('اس مہینے کی کل بکری'),
    e('total sales this month'),
  ]),
  z('Z31', 'month', 'period', 'period=last month', 'comm=@a.last_month.total.total_comm', [
    rm('pichle mahine ka comision kitna tha'),
    u('پچھلے مہینے کا کمیشن کتنا بنا'),
    e('commission last month'),
  ]),
  z('Z32', 'month', 'period', 'period=September', 'gross=@a.m2026_09.total.gross', [
    r('september ki bikri kitni thi'),
    u('ستمبر میں کتنی بکری ہوئی'),
    e('sales in September'),
  ]),
  z('Z33', 'month', 'period', 'period=August', 'gross=@a.m2026_08.total.gross', [
    r('august mein kitna maal biki'),
    u('اگست کی بکری کتنی رہی'),
    e('sales for August'),
  ]),
  z('Z34', 'day', 'period', 'period=15 October', 'gross=@a.d2026_10_15.total.gross', [
    r('15 october ko kitni bikri hui'),
    u('15 اکتوبر کی بکری'),
    e('sales on 15 October'),
  ]),
  z('Z35', 'period', 'period', 'period=this year', 'gross=@a.year.total.gross', [
    r('is saal ki kul bikri'),
    u('اس سال کی کل بکری کتنی ہے'),
    e('total sales this year'),
  ]),
  z('Z36', 'period', 'period', 'period=this year', 'comm=@a.year.total.total_comm', [
    rm('is sal ka comision kitna bana'),
    u('اس سال کا کل کمیشن'),
    e('commission this year'),
  ]),
  z('Z37', 'period', 'period', 'period=up to now', 'gross=@a.up_to_now.total.gross', [
    r('ab tak ki kul bikri'),
    u('اب تک کی total sale'),
    e('sales so far, all time'),
  ]),

  // ---- an item, a seller, a buyer over a period; goods left; a partner ---------------------------
  z(
    'Z38',
    'item',
    'item',
    'item=Banana;period=today',
    'qty=@a.today.banana.qty;gross=@a.today.banana.gross',
    [r('aaj kele ki kitni bikri hui'), u('آج کیلا کتنا بکا'), e('how much banana was sold today')],
  ),
  z('Z39', 'item', 'item', 'item=Apple;period=this year', 'comm=@a.year.apple.total_comm', [
    rm('sayb ka is sal ka comision'),
    u('سیب کا اس سال کتنا کمیشن بنا'),
    e('apple commission this year'),
  ]),
  z('Z40', 'item', 'item', 'item=Mango;period=this year', 'gross=@a.year.mango.gross', [
    r('aam ki is saal ki bikri'),
    u('آم کی اس سال کی بکری'),
    e('mango sales this year'),
  ]),
  z('Z41', 'seller', 'seller', 'seller=Karim;period=this year', 'gross=@b.year.karim.gross', [
    r('karim ka is saal kitna maal biki'),
    u('کریم کا اس سال کتنا مال بکا'),
    e("how much of Karim's goods sold this year"),
  ]),
  z(
    'Z42',
    'buyer',
    'buyer',
    'buyer=Tariq Mehmood;period=this year',
    'purchase=@c.year.tariq_mehmood.purchase_value',
    [
      r('tariq mehmood ne is saal kitne ka maal khareeda'),
      u('طارق محمود نے اس سال کتنے کا مال خریدا'),
      e('how much did Tariq Mehmood buy this year'),
    ],
  ),
  z('Z43', 'stock', 'stock', 'item=Mosambi', 'qty=@stock.mosambi.qty', [
    rm('mosmi ka kitna mal bacha he'),
    u('موسمی کا کتنا مال باقی ہے'),
    e('unsold mosambi stock'),
  ]),
  z('Z44', 'partner', 'partner', 'partner=Partner B', 'total=@partner.partner_b.total', [
    r('partner b ki kul raqam kitni hai'),
    u('پارٹنر ب کی کل رقم'),
    em("what is partnr B's total"),
  ]),
];

export default A2;
