// Facet CAP, part 2 (plan 55 Section 11): one question for a cell of the capability matrix
// (scripts/ai-eval/capability.tsv) that had none, with the figure from the oracle. The lead's ruling
// (05-11-2026): ONE question for each answerable cell, rotating Roman Urdu, Urdu letters, English and
// mixed, about a quarter misspelled; full groups of three wordings only for the cells a mandi owner is
// likely to ask (a figure, a list, a top, a party's day and a print on the main reports, views and
// tiles). The names of the groups tell the cell: CM-<row>-<question type>.
//
// A group whose figures name an oracle fact that is not in oracle-keys.txt yet is left out by
// bank-spec.mjs until the oracle has run again.

const r = (text) => ['roman', text];
const rm = (text) => ['roman', text, 'mis'];
const u = (text) => ['urdu', text];
const um = (text) => ['urdu', text, 'mis'];
const e = (text) => ['en', text];
const em = (text) => ['en', text, 'mis'];

// A compare and an advice cell are less likely questions: ONE wording goes into the bank, and the
// language rotates (Roman Urdu, Urdu letters, English). The other two wordings stay in the spec for
// a later round (the lead's ruling, 05-11-2026).
let rotate = 0;

/** One cell: a group named after its row and question type. */
const cell = (row, qtype, section, filters, figures, q, over = {}) => ({
  g: `CM-${row}-${qtype}`,
  ...(qtype === 'compare' || qtype === 'advice' ? { use: [rotate++ % 3] } : {}),
  cat: 'A',
  section,
  kind: over.kind ?? section,
  world: 'year',
  outcome: 'answered',
  tools: over.tools ?? '',
  filters: `facet=CAP;${filters}`,
  figures,
  source: `capability.tsv ${row} / ${qtype}`,
  q,
  ...over,
});

/** A request for advice from the books (category B). */
const advice = (row, filters, figures, q, over = {}) =>
  cell(row, 'advice', 'advice', `${filters};general=0`, figures, q, {
    cat: 'B',
    kind: 'advice',
    outcome: 'advice',
    ...over,
  });

const MORE = [
  // ---- the buyers' dues (R20, tile buyers_dues) ---------------------------------------------------
  cell(
    'R20',
    'list',
    'dues',
    'view=R20',
    'total=@tiles.buyers_dues|@tiles.total_dues',
    [
      r('sab gahakon ki baqaya list dikhao'),
      u('تمام گاہکوں کے بقایا کی فہرست دکھائیں'),
      e('show the list of what every buyer owes'),
    ],
    { tools: 'report:R20' },
  ),
  cell(
    'R20',
    'top',
    'dues',
    'view=R20;sort=balance',
    'top!=@top.buyer_dues.name;amount=@top.buyer_dues.amount',
    [
      r('sab se zyada udhar kis gahak ke zimme hai'),
      u('سب سے زیادہ ادھار کس گاہک کے ذمے ہے'),
      e('which buyer owes the most'),
    ],
    { tools: 'report:R20' },
  ),
  advice(
    'tile-buyers_dues',
    'view=R20',
    'first=@top.collect_first',
    [
      rm('baqaya wasool karne ke liye pehle kis ko phone karun'),
      um('بقایا وصول کرنے کے لیے پہلے کس کو فون کروں'),
      e('whom should I call first to collect the dues'),
    ],
    { tools: 'report:R20' },
  ),

  // ---- the sellers' payable (R24, tile sellers_payable) ------------------------------------------
  cell(
    'R24',
    'top',
    'dues',
    'view=R24;sort=payable',
    'top!=@top.seller_payable.name;amount=@top.seller_payable.amount',
    [
      r('sab se zyada paise kis bepari ko dene hain'),
      u('سب سے زیادہ رقم کس بیوپاری کو دینی ہے'),
      e('which seller do we owe the most'),
    ],
    { tools: 'report:R24' },
  ),
  cell(
    'R24',
    'list',
    'dues',
    'view=R24',
    'payable=@tiles.sellers_payable|@r24.total.balance',
    [
      rm('sab byopariyon ka dena kitna kitna hai'),
      u('تمام بیوپاریوں کا دینا الگ الگ دکھائیں'),
      e('list what we owe each seller'),
    ],
    { tools: 'report:R24' },
  ),
  advice(
    'tile-sellers_payable',
    'view=R24',
    'first=@top.pay_first',
    [
      r('byopariyon ko paise dene mein pehle kis ko dun'),
      um('بیوپاریوں کو پیسے دینے میں پہلے کس کو دوں'),
      e('which seller should I pay first'),
    ],
    { tools: 'report:R24' },
  ),

  // ---- advances (R26, tile advances_open) --------------------------------------------------------
  cell(
    'R26',
    'list',
    'dues',
    'view=R26',
    'open=@r26.total.open;given=@r26.total.given',
    [
      r('peshgi ki list dikhao, kis ko kitni di thi'),
      u('پیشگی کی فہرست دکھائیں، کس کو کتنی دی تھی'),
      e('show the advances list: who was given how much'),
    ],
    { tools: 'report:R26' },
  ),
  cell(
    'R26',
    'top',
    'dues',
    'view=R26;sort=open',
    'open=@r26.total.open',
    [
      rm('sab se zyada peshgi kis ke pas hai'),
      u('سب سے زیادہ پیشگی کس کے پاس ہے'),
      e('who holds the biggest open advance'),
    ],
    { tools: 'report:R26' },
  ),
  advice(
    'tile-advances_open',
    'view=R26',
    '',
    [
      r('khuli peshgi wapas lene ke liye kya karun'),
      u('کھلی پیشگی واپس لینے کے لیے کیا کروں'),
      em('what should I do to get the open advance back'),
    ],
    { tools: 'report:R26' },
  ),

  // ---- cash (R12, tile cash_in_hand) -------------------------------------------------------------
  cell(
    'R12',
    'list',
    'cash',
    'view=R12;period=this month',
    'closing=@r12.month.sum.report_closing_balance',
    [
      r('is mahine ki rokar book dikhao'),
      u('اس مہینے کی روکڑ بک دکھائیں'),
      e('show the cash book for this month'),
    ],
    { tools: 'report:R12' },
  ),
  cell(
    'R12',
    'day',
    'cash',
    'view=R12;period=yesterday',
    'closing=@r12.yesterday.sum.report_closing_balance',
    [
      rm('kal raat rokar kitni bachi thi'),
      u('کل رات روکڑ کتنی بچی تھی'),
      e('how much cash was left at the end of yesterday'),
    ],
    { tools: 'report:R12' },
  ),
  cell(
    'R12',
    'compare',
    'cash',
    'view=R12',
    'now=@r12.today.sum.report_closing_balance;before=@r12.last_month.sum.report_closing_balance',
    [
      r('rokar pichle mahine ke end se ab kitni zyada ya kam hai'),
      u('روکڑ پچھلے مہینے کے آخر سے اب کتنی زیادہ یا کم ہے'),
      e('is the cash now more or less than at the end of last month'),
    ],
    { tools: 'report:R12' },
  ),
  advice(
    'tile-cash_in_hand',
    'view=R12',
    '',
    [
      r('cash kam ho to kya karun'),
      u('کیش کم ہو تو کیا کروں'),
      e('what should I do if cash runs low'),
    ],
    { tools: 'report:R12' },
  ),

  // ---- registers: receipts (R23), payments (R25) -------------------------------------------------
  cell(
    'R23',
    'figure',
    'dues',
    'view=R23;period=last month',
    'received=@r23.last_month.total.amount',
    [
      r('pichle mahine gahakon se kitni wasooli hui'),
      u('پچھلے مہینے گاہکوں سے کتنی وصولی ہوئی'),
      e('how much did we collect from buyers last month'),
    ],
    { tools: 'report:R23' },
  ),
  cell(
    'R23',
    'compare',
    'dues',
    'view=R23',
    'last=@r23.last_month.total.amount;year=@r23.year.total.amount',
    [
      rm('wasooli pichle mahine kitni hui aur is saal kul kitni hui'),
      u('وصولی پچھلے مہینے کتنی ہوئی اور اس سال کل کتنی ہوئی'),
      e('collections last month against the whole year'),
    ],
    { tools: 'report:R23' },
  ),
  cell(
    'R25',
    'figure',
    'dues',
    'view=R25;period=this month',
    'paid=@r25.month.total.amount',
    [
      r('is mahine byopariyon ko kitni adaigi hui'),
      u('اس مہینے بیوپاریوں کو کتنی ادائیگی ہوئی'),
      em('how much did we pay sellers this mont'),
    ],
    { tools: 'report:R25' },
  ),
  cell(
    'R25',
    'compare',
    'dues',
    'view=R25',
    'month=@r25.month.total.amount;last=@r25.last_month.total.amount',
    [
      r('adaigi is mahine pichle mahine se kitni kam ya zyada hai'),
      u('ادائیگی اس مہینے پچھلے مہینے سے کتنی کم یا زیادہ ہے'),
      e('are payments this month more or less than last month'),
    ],
    { tools: 'report:R25' },
  ),

  // ---- the expense report (R29) and the profit split (R30) ---------------------------------------
  cell(
    'R29',
    'figure',
    'expense',
    'view=R29;period=this year',
    'total=@r29.year.total.expense',
    [
      r('is saal dukan ka kul kharcha kitna hua'),
      u('اس سال دکان کا کل خرچہ کتنا ہوا'),
      e('what is the total shop expense this year'),
    ],
    { tools: 'report:R29' },
  ),
  cell(
    'R29',
    'top',
    'expense',
    'view=R29;sort=expense',
    'top!=@top.expense_head_year.name;amount=@top.expense_head_year.amount',
    [
      rm('is sal sab se zyada kharcha kis mad mein hua'),
      u('اس سال سب سے زیادہ خرچہ کس مد میں ہوا'),
      e('which expense head cost the most this year'),
    ],
    { tools: 'report:R29' },
  ),
  cell(
    'R29',
    'list',
    'expense',
    'view=R29;period=last month',
    'total=@r29.last_month.total.expense',
    [
      r('pichle mahine ke tamam kharchon ki list'),
      u('پچھلے مہینے کے تمام خرچوں کی فہرست'),
      e('list all expenses of last month'),
    ],
    { tools: 'report:R29' },
  ),
  cell(
    'R29',
    'compare',
    'expense',
    'view=R29',
    'last=@r29.last_month.total.expense;year=@r29.year.total.expense',
    [
      r('kharcha pichle mahine kitna tha aur is saal kul kitna'),
      u('خرچہ پچھلے مہینے کتنا تھا اور اس سال کل کتنا'),
      e('expenses last month compared with the whole year'),
    ],
    { tools: 'report:R29' },
  ),
  cell(
    'R30',
    'figure',
    'partner',
    'view=R30;period=this year',
    'profit=@r30.year.total.profit_share',
    [
      r('is saal ka kul munafa kitna hai'),
      u('اس سال کا کل منافع کتنا ہے'),
      e('what is the total profit this year'),
    ],
    { tools: 'report:R30' },
  ),
  cell(
    'R30',
    'list',
    'partner',
    'view=R30;period=this year',
    'a=@r30.year.partner_a.profit_share;b=@r30.year.partner_b.profit_share;c=@r30.year.usman.profit_share',
    [
      rm('har shirakat dar ka is saal ka munafa alag alag batao'),
      u('ہر شراکت دار کا اس سال کا منافع الگ الگ بتائیں'),
      e("show each partner's profit share this year"),
    ],
    { tools: 'report:R30' },
  ),
  cell(
    'R30',
    'compare',
    'partner',
    'view=R30',
    'year=@r30.year.total.profit_share;last=@r30.last_month.total.profit_share',
    [
      r('munafa pichle mahine kitna tha aur is saal kul kitna'),
      u('منافع پچھلے مہینے کتنا تھا اور اس سال کل کتنا'),
      e('profit last month against the whole year'),
    ],
    { tools: 'report:R30' },
  ),
  advice(
    'R30',
    'view=R30',
    '',
    [
      r('munafa barhane ke liye kharche ya commission mein se kya dekhun'),
      u('منافع بڑھانے کے لیے خرچے یا کمیشن میں سے کیا دیکھوں'),
      e('to raise profit should I look at expenses or commission'),
    ],
    { tools: 'report:R30,report:R29' },
  ),

  // ---- money lent (R32), the dead amount (R38) ---------------------------------------------------
  cell(
    'R32',
    'list',
    'dues',
    'view=R32',
    'open=@r32.total.open;lent=@r32.total.lent',
    [
      r('udhar raqam ki list dikhao, kis ne kitni li'),
      u('ادھار رقم کی فہرست دکھائیں، کس نے کتنی لی'),
      e('show money lent: who took how much'),
    ],
    { tools: 'report:R32', kind: 'lent' },
  ),
  cell(
    'R32',
    'top',
    'dues',
    'view=R32;sort=open',
    'open=@r32.total.open',
    [
      rm('udhar raqam mein sab se zyada kis ke zimme hai'),
      u('ادھار رقم میں سب سے زیادہ کس کے ذمے ہے'),
      e('who owes the most of the money lent'),
    ],
    { tools: 'report:R32', kind: 'lent' },
  ),
  advice(
    'R32',
    'view=R32',
    '',
    [
      r('jis ne udhar raqam wapas nahi ki us ka kya karun'),
      u('جس نے ادھار رقم واپس نہیں کی اس کا کیا کروں'),
      e('what do I do about the one who has not returned the money lent'),
    ],
    { tools: 'report:R32' },
  ),
  cell(
    'R38',
    'figure',
    'dues',
    'view=R38',
    'dead=@tiles.dead',
    [r('doobi hui raqam kitni hai'), u('ڈوبی ہوئی رقم کتنی ہے'), e('how much is the dead amount')],
    { tools: 'report:R38,tiles' },
  ),
  cell(
    'R38',
    'list',
    'dues',
    'view=R38',
    'dead=@tiles.dead;owed=@r38.total.owed_then',
    [
      rm('doobi raqam mein kis kis ka naam hai'),
      u('ڈوبی رقم میں کس کس کا نام ہے'),
      e('whose names are in the dead amount'),
    ],
    { tools: 'report:R38' },
  ),
  advice(
    'R38',
    'view=R38',
    '',
    [
      r('doobi raqam wapas kaise aaye'),
      u('ڈوبی رقم واپس کیسے آئے'),
      e('how can the dead amount be recovered'),
    ],
    { tools: 'report:R38' },
  ),

  // ---- trips (R33, view J), gardens (R36, view I), fruit (R37, view K) ---------------------------
  cell(
    'R33',
    'figure',
    'trip',
    'view=R33',
    'result=@trip1.result;second=@trip2.result',
    [
      r('market safar ka nafa ya nuqsan kitna raha'),
      u('مارکیٹ سفر کا نفع یا نقصان کتنا رہا'),
      e('what was the profit or loss of the market trips'),
    ],
    { tools: 'report:R33' },
  ),
  cell(
    'R33',
    'list',
    'trip',
    'view=R33',
    'cost1=@trip1.bought_cost;cost2=@trip2.bought_cost',
    [
      r('tamam market safar ki list, kitne ka maal kharida'),
      u('تمام مارکیٹ سفر کی فہرست، کتنے کا مال خریدا'),
      e('list all market trips and what was bought'),
    ],
    { tools: 'report:R33' },
  ),
  cell(
    'R33',
    'top',
    'trip',
    'view=R33;sort=result',
    'best=@trip2.result',
    [
      rm('kis safar mein sab se zyada nafa hua'),
      u('کس سفر میں سب سے زیادہ نفع ہوا'),
      e('which trip made the most profit'),
    ],
    { tools: 'report:R33' },
  ),
  advice(
    'R33',
    'view=R33',
    '',
    [
      r('market safar se maal khareedna theek rahta hai ya nahi'),
      u('مارکیٹ سفر سے مال خریدنا ٹھیک رہتا ہے یا نہیں'),
      e('is buying goods on market trips worth it'),
    ],
    { tools: 'report:R33' },
  ),
  cell(
    'R36',
    'figure',
    'garden',
    'view=R36',
    'balance=@r36.total.balance',
    [
      r('baagh ka abhi ka balance kya hai'),
      u('باغ کا ابھی کا بیلنس کیا ہے'),
      e('what is the garden balance now'),
    ],
    { tools: 'report:R36,tiles' },
  ),
  cell(
    'R36',
    'list',
    'garden',
    'view=R36',
    'costs=@r36.total.costs;commission=@r36.total.commission',
    [
      rm('baagh ke kharche aur commission alag alag batao'),
      u('باغ کے خرچے اور کمیشن الگ الگ بتائیں'),
      e("show the garden's costs and commission"),
    ],
    { tools: 'report:R36' },
  ),
  advice(
    'R36',
    'view=R36',
    '',
    [
      r('baagh ka thaka kab band karun'),
      u('باغ کا ٹھیکہ کب بند کروں'),
      e('when should I close the garden'),
    ],
    { tools: 'report:R36' },
  ),
  cell(
    'R37',
    'figure',
    'fruit',
    'view=R37;period=this year',
    'total=@r37.year.total.value',
    [
      r('is saal partnaron ne kitne ka phal liya'),
      u('اس سال شراکت داروں نے کتنے کا پھل لیا'),
      e('how much fruit did the partners take this year'),
    ],
    { tools: 'report:R37' },
  ),
  cell(
    'R37',
    'compare',
    'fruit',
    'view=R37',
    'year=@r37.year.total.value;last=@r37.last_month.total.value',
    [
      r('phal is saal kitna liya aur pichle mahine kitna'),
      u('پھل اس سال کتنا لیا اور پچھلے مہینے کتنا'),
      e('fruit taken this year against last month'),
    ],
    { tools: 'report:R37' },
  ),

  // ---- stock (R10) -------------------------------------------------------------------------------
  cell(
    'R10',
    'list',
    'stock',
    'view=R10',
    'banana=@stock.banana.qty;apple=@stock.apple.qty;mango=@stock.mango.qty;mosambi=@stock.mosambi.qty;potato=@stock.potato.qty',
    [
      r('kis kis item ka kitna maal baqi hai'),
      u('کس کس آئٹم کا کتنا مال باقی ہے'),
      e('how much unsold stock is there of each item'),
    ],
    { tools: 'report:R10' },
  ),
  cell(
    'R10',
    'top',
    'stock',
    'view=R10;sort=balance',
    'potato=@stock.potato.qty',
    [
      rm('sab se zyada kis item ka maal pada hua hai'),
      u('سب سے زیادہ کس آئٹم کا مال پڑا ہوا ہے'),
      e('which item has the most unsold stock'),
    ],
    { tools: 'report:R10' },
  ),
  advice(
    'R10',
    'view=R10',
    '',
    [
      r('purana maal pehle bechna chahiye ya naya'),
      u('پرانا مال پہلے بیچنا چاہیے یا نیا'),
      e('should I sell the old stock first or the new'),
    ],
    { tools: 'report:R10' },
  ),

  // ---- partners (R31, view F) --------------------------------------------------------------------
  cell(
    'R31',
    'compare',
    'partner',
    'view=R31;partner=Partner A',
    'total=@partner.partner_a.total',
    [
      r('partner a ki kul raqam pichle mahine ke mukable mein kitni badli'),
      u('پارٹنر الف کی کل رقم پچھلے مہینے کے مقابلے میں کتنی بدلی'),
      e("how did Partner A's total change since last month"),
    ],
    { tools: 'report:R31' },
  ),
  cell(
    'view-F',
    'top',
    'partner',
    'view=F;sort=income_share',
    'a=@f.all.partner_a.income_share',
    [
      rm('sab se zyada hissa kis shirakat dar ka bana'),
      u('سب سے زیادہ حصہ کس شراکت دار کا بنا'),
      e('which partner has the biggest share'),
    ],
    { tools: 'analytics:F' },
  ),
  cell(
    'figure-partner_total',
    'compare',
    'partner',
    'view=R31',
    'a=@partner.partner_a.total;b=@partner.partner_b.total',
    [
      r('partner a aur partner b ki kul raqam mein farq kitna hai'),
      u('پارٹنر الف اور پارٹنر ب کی کل رقم میں فرق کتنا ہے'),
      e("what is the difference between Partner A's and Partner B's total"),
    ],
    { tools: 'report:R31' },
  ),
  cell(
    'figure-partner_share',
    'figure',
    'partner',
    'view=R30;period=this month',
    'a=@partner.partner_a.month_share',
    [
      r('partner a ka is mahine ka hissa kitna hai'),
      u('پارٹنر الف کا اس مہینے کا حصہ کتنا ہے'),
      e("Partner A's share this month"),
    ],
    { tools: 'report:R30' },
  ),
  cell(
    'figure-partner_share',
    'top',
    'partner',
    'view=F;period=this year',
    'a=@partner.partner_a.year_share',
    [
      r('is saal kis partner ka hissa sab se zyada raha'),
      u('اس سال کس پارٹنر کا حصہ سب سے زیادہ رہا'),
      e('which partner had the biggest share this year'),
    ],
    { tools: 'analytics:F' },
  ),
  cell(
    'figure-partner_capital',
    'list',
    'capital',
    'view=R31',
    'b=@partner.partner_b.capital;usman=@partner.usman.capital',
    [
      rm('har partner ka sarmaya kitna laga hua hai'),
      u('ہر پارٹنر کا سرمایہ کتنا لگا ہوا ہے'),
      e("each partner's capital in the shop"),
    ],
    { tools: 'report:R31' },
  ),

  // ---- the rest of the tiles ---------------------------------------------------------------------
  cell(
    'tile-sellers_dues',
    'figure',
    'dues',
    'view=tiles',
    'owe=@tiles.sellers_owe',
    [
      r('kisi byopari ne humein paise dene hain'),
      u('کیا کسی بیوپاری نے ہمیں پیسے دینے ہیں'),
      e('does any seller owe the shop money'),
    ],
    { tools: 'tiles' },
  ),
  cell(
    'tile-today_expenses',
    'figure',
    'expense',
    'view=tiles;period=today',
    'exp=@tiles.expenses_today',
    [
      r('aaj ka dukan ka kharcha kitna hua'),
      u('آج کا دکان کا خرچہ کتنا ہوا'),
      e("today's shop expenses"),
    ],
    { tools: 'tiles' },
  ),
  cell(
    'tile-today_sales',
    'compare',
    'period',
    'view=A;period=today',
    'today=@a.today.total.gross;yesterday=@a.yesterday.total.gross',
    [
      r('aaj ki bikri kal se kitni zyada ya kam hai'),
      u('آج کی بکری کل سے کتنی زیادہ یا کم ہے'),
      e("is today's sale more or less than yesterday"),
    ],
    { tools: 'analytics:A' },
  ),
  cell(
    'tile-today_commission',
    'compare',
    'period',
    'view=A;period=today',
    'today=@a.today.total.total_comm;yesterday=@a.yesterday.total.total_comm',
    [
      rm('aaj ka commission kal ke mukable mein kitna hai'),
      u('آج کا کمیشن کل کے مقابلے میں کتنا ہے'),
      e("today's commission against yesterday's"),
    ],
    { tools: 'analytics:A' },
  ),
  cell(
    'tile-stock',
    'advice',
    'advice',
    'view=tiles',
    '',
    [
      r('baqi maal bahut ho gaya hai kya karun'),
      u('باقی مال بہت ہو گیا ہے کیا کروں'),
      e('there is a lot of unsold stock, what should I do'),
    ],
    { cat: 'B', kind: 'advice', outcome: 'advice', tools: 'tiles,report:R10' },
  ),
  cell(
    'tile-trips',
    'advice',
    'advice',
    'view=tiles',
    '',
    [
      r('safar ka baqi maal kaise nikalun'),
      u('سفر کا باقی مال کیسے نکالوں'),
      e('how do I sell off what is left of the trip goods'),
    ],
    { cat: 'B', kind: 'advice', outcome: 'advice', tools: 'tiles,report:R33' },
  ),
  cell(
    'tile-gardens',
    'advice',
    'advice',
    'view=tiles',
    '',
    [
      r('baagh ka balance minus hai kya karun'),
      u('باغ کا بیلنس مائنس ہے کیا کروں'),
      e('the garden balance is negative, what should I do'),
    ],
    { cat: 'B', kind: 'advice', outcome: 'advice', tools: 'tiles,report:R36' },
  ),
  cell(
    'tile-money_lent_open',
    'advice',
    'advice',
    'view=tiles',
    '',
    [
      rm('udhar raqam ka tagaza kaise karun'),
      u('ادھار رقم کا تقاضا کیسے کروں'),
      e('how do I ask for the money lent back'),
    ],
    { cat: 'B', kind: 'advice', outcome: 'advice', tools: 'tiles,report:R32' },
  ),
  cell(
    'tile-dead_amount',
    'advice',
    'advice',
    'view=tiles',
    '',
    [
      r('doobi raqam ka hisab kaise saaf karun'),
      u('ڈوبی رقم کا حساب کیسے صاف کروں'),
      e('how do I clear the dead amount from the books'),
    ],
    { cat: 'B', kind: 'advice', outcome: 'advice', tools: 'tiles,report:R38' },
  ),
  cell(
    'tile-total_dues',
    'advice',
    'advice',
    'view=tiles',
    '',
    [
      r('kul baqaya bahut hai, cash kaise nikaloon'),
      u('کل بقایا بہت ہے، کیش کیسے نکالوں'),
      e('total dues are high, how do I bring in cash'),
    ],
    { cat: 'B', kind: 'advice', outcome: 'advice', tools: 'tiles,report:R20' },
  ),

  // ---- items, sellers, buyers (views A, B, C) ----------------------------------------------------
  cell(
    'view-A',
    'top',
    'item',
    'view=A;period=this year',
    'top!=@top.item_commission_year.name;amount=@top.item_commission_year.amount',
    [
      r('is saal kis item se sab se zyada commission mila'),
      u('اس سال کس آئٹم سے سب سے زیادہ کمیشن ملا'),
      e('which item gave the most commission this year'),
    ],
    { tools: 'analytics:A' },
  ),
  cell(
    'view-A',
    'list',
    'item',
    'view=A;period=this month',
    'total=@a.month.total.gross',
    [
      rm('is mahine har item ki bikri alag alag dikhao'),
      u('اس مہینے ہر آئٹم کی بکری الگ الگ دکھائیں'),
      e('show this month sales item by item'),
    ],
    { tools: 'analytics:A' },
  ),
  cell(
    'view-A',
    'compare',
    'item',
    'view=A;period=this year;compare=last_year',
    'year=@a.year.total.total_comm;last=@a.last_year.total.total_comm',
    [
      r('is saal ka commission pichle saal se kitna zyada hai'),
      u('اس سال کا کمیشن پچھلے سال سے کتنا زیادہ ہے'),
      e('is the commission this year more than last year'),
    ],
    { tools: 'analytics:A' },
  ),
  cell(
    'view-B',
    'top',
    'seller',
    'view=B;period=this year',
    'top!=@top.seller_commission_year.name;amount=@top.seller_commission_year.amount',
    [
      r('is saal sab se zyada commission kis bepari ke maal se bana'),
      u('اس سال سب سے زیادہ کمیشن کس بیوپاری کے مال سے بنا'),
      e('whose goods gave the most commission this year'),
    ],
    { tools: 'analytics:B' },
  ),
  cell(
    'view-B',
    'list',
    'seller',
    'view=B;period=this month',
    'total=@b.month.total.gross',
    [
      r('is mahine har bepari ka maal kitna bika'),
      u('اس مہینے ہر بیوپاری کا مال کتنا بکا'),
      e("how much of each seller's goods sold this month"),
    ],
    { tools: 'analytics:B' },
  ),
  cell(
    'view-C',
    'top',
    'buyer',
    'view=C;period=this year',
    'top!=@top.buyer_purchase_year.name;amount=@top.buyer_purchase_year.amount',
    [
      rm('is saal sab se zyada maal kis gahak ne khareeda'),
      u('اس سال سب سے زیادہ مال کس گاہک نے خریدا'),
      e('which buyer bought the most this year'),
    ],
    { tools: 'analytics:C' },
  ),
  cell(
    'view-C',
    'list',
    'buyer',
    'view=C;period=this month',
    'total=@c.month.total.purchase_value',
    [
      r('is mahine har gahak ne kitne ka maal khareeda'),
      u('اس مہینے ہر گاہک نے کتنے کا مال خریدا'),
      e('how much did each buyer buy this month'),
    ],
    { tools: 'analytics:C' },
  ),
  cell(
    'view-G',
    'figure',
    'period',
    'view=G;period=this year',
    'year=@a.year.total.total_comm',
    [
      r('mahine ba mahine commission ka trend dikhao, is saal ka kul kitna'),
      u('مہینے بہ مہینے کمیشن کا ٹرینڈ دکھائیں، اس سال کا کل کتنا'),
      e('show the monthly commission trend and the year total'),
    ],
    { tools: 'analytics:G' },
  ),
  cell(
    'view-G',
    'top',
    'month',
    'view=G;sort=commission',
    'top!=@top.month_commission.name;amount=@top.month_commission.amount',
    [
      r('kis mahine mein sab se zyada commission bana'),
      u('کس مہینے میں سب سے زیادہ کمیشن بنا'),
      e('which month gave the most commission'),
    ],
    { tools: 'analytics:G' },
  ),
  cell(
    'view-H',
    'figure',
    'item',
    'view=H;period=this year',
    'top!=@top.item_commission_year.name',
    [
      r('top 10 mein sab se upar kaun sa item hai'),
      u('ٹاپ 10 میں سب سے اوپر کون سا آئٹم ہے'),
      e('which item is at the top of the top 10'),
    ],
    { tools: 'analytics:H' },
  ),
];

export default MORE;
