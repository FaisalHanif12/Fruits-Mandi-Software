// Facet VEG of the team bank (Umar, 06-11-2026): the assistant must handle vegetables as well as fruit.
//
// The year books sell five vegetables over the year: mooli (radish), aloo (potato), tamatar (tomato), piyaz
// (onion) and saag, with no لاگا (PRD 4.2: the لاگا rule is for the fruit group). The facts are the oracle's:
// `a.<period>.<item>.*` (view A), `group.<fruit|vegetable>.<period>.*` (the sum of the items of a group),
// `group_top.vegetable.<period>` (the vegetable with the most commission), `est_item.<item>.*` (the forecast
// of an item) and `veg.laga_lines` (the sale lines of a vegetable that carry a لاگا: 0).
//
// 27 groups of 3 wordings: one item's figure, the sabzi group total, commission on vegetables, the group
// against the fruit, the top vegetable, a forecast for a vegetable, a print for a vegetable, mixed language
// and misspellings, and two checks that no answer gives a لاگا on a vegetable line (`nolaga=1`: a table of the
// answer with an amount in its لاگا column fails, and the figure itself must be 0).
//
// The same format as bank-spec.mjs; `cat` is added there unless a group sets its own.

const r = (text) => ['roman', text];
const rm = (text) => ['roman', text, 'mis'];
const u = (text) => ['urdu', text];
const e = (text) => ['en', text];
const em = (text) => ['en', text, 'mis'];

const three = (prefix) =>
  ['same_pace', 'same_growth', 'same_season'].map((m) => `@${prefix}.${m}`).join('|');

/** One group of facet VEG. */
const v = (g, section, kind, filters, figures, q, over = {}) => ({
  g,
  section,
  kind,
  world: 'year',
  outcome: 'answered',
  filters: `facet=VEG;${filters}`,
  figures,
  source: 'Umar 06-11-2026: vegetables',
  q,
  ...over,
});

const VEG = [
  // ---- one vegetable's figure -------------------------------------------------------------------
  v('VG01', 'item', 'item', 'item=Tomato;period=this year', 'sale=@a.year.tomato.gross', [
    r('tamatar ki is saal ki bikri kitni hui'),
    u('ٹماٹر کی اس سال کی بکری کتنی ہوئی'),
    e('how much tomato did we sell this year'),
  ]),
  v(
    'VG02',
    'item',
    'item',
    'item=Onion;period=last month',
    'qty=@a.last_month.onion.qty;sale=@a.last_month.onion.gross',
    [
      rm('piyaz ki pichle mahine ki bikri aur nag batao'),
      u('پیاز کی پچھلے مہینے کی بکری اور نگ بتائیں'),
      e('onion sales and quantity for last month'),
    ],
  ),
  v('VG03', 'item', 'item', 'item=Radish;period=this year', 'comm=@a.year.radish.total_comm', [
    r('mooli ka is saal ka commission kitna bana'),
    u('مولی کا اس سال کا کمیشن کتنا بنا'),
    e('radish commission this year'),
  ]),
  v(
    'VG04',
    'item',
    'item',
    'item=Saag;period=this year',
    'qty=@a.year.saag.qty;sale=@a.year.saag.gross',
    [
      r('saag ka is saal ka nag aur bikri'),
      u('ساگ کی اس سال کی بکری اور نگ'),
      e('saag quantity and sales this year'),
    ],
  ),
  // ---- the sabzi group -----------------------------------------------------------------------------
  v(
    'VG05',
    'period',
    'group',
    'group=vegetable;period=this year',
    'veg=@group.vegetable.year.gross',
    [
      r('sabzi ki is saal ki kul bikri kitni hui'),
      u('سبزی کی اس سال کی کل بکری کتنی ہوئی'),
      e('total vegetable sales this year'),
    ],
  ),
  v(
    'VG06',
    'period',
    'group',
    'group=vegetable;period=this year',
    'comm=@group.vegetable.year.total_comm',
    [
      r('sabziyon se is saal kitna commission mila'),
      u('سبزیوں سے اس سال کتنا کمیشن ملا'),
      e('commission earned on vegetables this year'),
    ],
  ),
  v(
    'VG07',
    'period',
    'group',
    'group=vegetable;period=this year',
    'veg=@group.vegetable.year.gross;fruit=@group.fruit.year.gross',
    [
      r('sabzi aur phal ki is saal ki bikri alag alag batao'),
      u('سبزی اور پھل کی اس سال کی بکری الگ الگ بتائیں'),
      e('vegetable sales against fruit sales this year'),
    ],
  ),
  v('VG08', 'item', 'top', 'group=vegetable;period=this year', 'top!=@group_top.vegetable.year', [
    r('sabzi mein sab se zyada commission kis ne diya'),
    u('سبزیوں میں سب سے زیادہ کمیشن کس سے ملا'),
    e('which vegetable gave the most commission this year'),
  ]),
  // ---- a forecast for a vegetable ---------------------------------------------------------------------
  v(
    'VG09',
    'forecast',
    'estimates',
    'item=Onion;ahead=3',
    `est=${three('est_item.onion.commission.a3')}`,
    [
      r('piyaz ka agle 3 mahine ka commission kitna hoga'),
      u('پیاز کا اگلے 3 مہینوں کا کمیشن کتنا ہو گا'),
      e('estimate the onion commission for the next 3 months'),
    ],
    { cat: 'C', outcome: 'estimate' },
  ),
  // ---- a print for a vegetable -----------------------------------------------------------------------
  v(
    'VG10',
    'print',
    'print',
    'item=Tomato;period=this month',
    'print.report=R7;print.item=Tomato;print.from=@cal.month_start;print.to=@cal.today;print.steps=1',
    [
      r('tamatar ki is mahine ki bikri ka print'),
      u('ٹماٹر کی اس مہینے کی بکری کا پرنٹ'),
      e('print the tomato sales for this month'),
    ],
  ),
  // ---- mixed language and misspellings -----------------------------------------------------------------
  v('VG11', 'item', 'mixed', 'item=Tomato;period=this year', 'sale=@a.year.tomato.gross', [
    r('tamatar ki sale this year kitni hui'),
    u('ٹماٹر کی sale اس سال کتنی ہوئی'),
    e('how much tamatar bika this year'),
  ]),
  v('VG12', 'item', 'item', 'item=Onion;period=this year', 'sale=@a.year.onion.gross', [
    rm('pyaz ki bikri is sal kitni hui'),
    u('پیاز ki بکری اس سال کتنی ہوئی'),
    em('how much onoin did we sell this year'),
  ]),
  v('VG13', 'item', 'mixed', 'item=Radish;period=this year', 'sale=@a.year.radish.gross', [
    r('mooli ka sale amount is year ka'),
    u('مولی کی total بکری اس سال کی'),
    e('radish ki bikri this year'),
  ]),
  // ---- no لاگا on a vegetable line ----------------------------------------------------------------------
  v('VG14', 'item', 'laga', 'item=Tomato;period=this year;nolaga=1', 'laga=@a.year.tomato.laga', [
    r('tamatar ke bill par is saal laga kitna laga'),
    u('ٹماٹر کے بل پر اس سال لاگا کتنا لگا'),
    e('how much laga was charged on tomato this year'),
  ]),
  v(
    'VG15',
    'period',
    'laga',
    'group=vegetable;period=this year;nolaga=1',
    'laga=@group.vegetable.year.laga',
    [
      r('sabzi par is saal kul laga kitna laga'),
      u('سبزی پر اس سال کل لاگا کتنا لگا'),
      e('how much laga did vegetables carry this year'),
    ],
  ),
  // ---- more about the vegetables (Umar, 06-11-2026) ----------------------------------------------------
  v(
    'VG16',
    'period',
    'group',
    'group=vegetable;period=this year',
    'comm=@group.vegetable.year.total_comm',
    [
      rm('is sal sabzi se kitni commision bani'),
      u('اس سال سبزی سے کتنی کمیشن بنی'),
      em('how much comision did vegetables make this year'),
    ],
  ),
  v(
    'VG17',
    'period',
    'group',
    'group=vegetable;period=this month',
    'comm=@group.vegetable.month.total_comm',
    [
      r('is mahine sabzi se kitna commission bana'),
      u('اس مہینے سبزی سے کتنا کمیشن بنا'),
      e('commission from vegetables this month'),
    ],
  ),
  v(
    'VG18',
    'stock',
    'stock',
    'group=vegetable',
    'potato=@veg.stock.potato;tomato=@veg.stock.tomato;total=@veg.stock.total',
    [
      r('konsi sabzi kitni baqi hai'),
      u('کون سی سبزی کتنی باقی ہے'),
      em('how much of each vegetable is left unsold'),
    ],
  ),
  v('VG19', 'stock', 'stock', 'item=Tomato', 'qty=@veg.stock.tomato', [
    rm('tamatar ka kitna mal baqi hai'),
    u('ٹماٹر کا کتنا مال باقی ہے'),
    e('how many nag of tomato are left'),
  ]),
  v(
    'VG20',
    'unknown',
    'nosuch',
    'item=Capsicum;asked_role=item',
    '',
    [
      r('shimla mirch ki is mahine ki sale'),
      u('شملہ مرچ کی اس مہینے کی بکری'),
      em('capsicum sales this month'),
    ],
    { cat: 'E', outcome: 'not_found' },
  ),
  v('VG21', 'item', 'item', 'item=Radish;period=this month', 'sale=@veg.month.radish.gross', [
    r('mooli ki is mahine ki sale kitni hui'),
    u('مولی کی اس مہینے کی بکری کتنی ہوئی'),
    em('how much radish did we sell this month'),
  ]),
  v('VG22', 'item', 'item', 'item=Tomato', 'last=@veg.last_rate.tomato', [
    r('tamatar ka aaj ka rate kya hai'),
    u('ٹماٹر کا آج کا ریٹ کیا ہے'),
    e("what is today's rate of tomato"),
  ]),
  v('VG23', 'item', 'item', 'item=Onion', 'rate=@veg.last_rate.onion', [
    rm('piyaz ka akhri rate kitna tha'),
    u('پیاز کا آخری ریٹ کتنا تھا'),
    e('what was the last rate of onion'),
  ]),
  v('VG24', 'buyer', 'top', 'item=Potato', 'top!=@veg.top_buyer.potato', [
    r('aloo sab se zyada kis gahak ne khareeda'),
    u('آلو سب سے زیادہ کس گاہک نے خریدا'),
    e('which buyer bought the most potato'),
  ]),
  v('VG25', 'seller', 'top', 'item=Potato', 'top!=@veg.top_seller.potato', [
    r('aloo kis bepari ka sab se zyada aya'),
    u('آلو کس بیوپاری کا سب سے زیادہ آیا'),
    em('which seller sent the most potato'),
  ]),
  v('VG26', 'item', 'compare', 'item=Potato;period=this month', 'diff=@veg.month_vs_last.potato', [
    r('aloo ki is mahine ki bikri pichle mahine se kitni zyada ya kam'),
    u('آلو کی اس مہینے کی بکری پچھلے مہینے سے کتنی زیادہ یا کم'),
    e('potato sales this month against last month'),
  ]),
  v(
    'VG27',
    'item',
    'top',
    'group=vegetable;period=this year',
    'top!=@group_best.vegetable.year.gross',
    [
      r('is saal sab se zyada kaunsi sabzi bikti hai'),
      u('اس سال سب سے زیادہ کون سی سبزی بکتی ہے'),
      em('which vegetable sold the most this year'),
    ],
  ),
];

export default VEG;
