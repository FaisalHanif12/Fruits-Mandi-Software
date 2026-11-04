// Facet CAP, part 6 (plan 55 Section 11): the forecast cell and the mixed cell (a real figure and a
// forecast in ONE question) of the rows that have a forecast route: total sales, commission, shop
// expenses and profit (view G), and the sales of one item, one seller or one buyer. The estimate is the
// oracle's `est.*` / `est_item.*` / `est_party.*` fact (the app's own domain forecast); any of the
// three methods counts (the band). In a mixed question the labels `real_` and `est_` keep the two
// apart (`mix=1` in the filters turns on the same rule as facet MIX).

const three = (key) =>
  ['same_pace', 'same_growth', 'same_season'].map((m) => `@${key}.${m}`).join('|');

let rotate = 0;

/** A forecast cell: category C, ends as an estimate. */
const forecast = (row, filters, key, q) => ({
  g: `CF-${row}-forecast`,
  cat: 'C',
  section: 'forecast',
  kind: 'estimates',
  world: 'year',
  outcome: 'estimate',
  tools: 'estimate',
  filters: `facet=CAP;${filters}`,
  figures: `est=${three(key)}`,
  source: `capability.tsv ${row} / forecast`,
  q: [q[rotate++ % q.length]],
});

/** A mixed cell: a real figure of the books and an estimate. */
const mixed = (row, filters, real, key, q) => ({
  g: `CF-${row}-mixed`,
  cat: 'C',
  section: 'forecast',
  kind: 'mixed_forecast',
  world: 'year',
  outcome: 'estimate',
  tools: '',
  filters: `facet=CAP;mix=1;${filters}`,
  figures: `real_fig=${real};est_next=${three(key)}`,
  source: `capability.tsv ${row} / mixed`,
  q: [q[rotate++ % q.length]],
});

const r = (text) => ['roman', text];
const u = (text) => ['urdu', text];
const e = (text) => ['en', text];

export default [
  // total sales (R2), commission (R28), shop expenses (R29), profit (R30)
  forecast('R2', 'view=R2;ahead=3', 'est.sales.a3', [
    r('sales register ke hisab se agle 3 mahine ki bikri ka andaza'),
    u('سیلز رجسٹر کے حساب سے اگلے 3 مہینوں کی بکری کا اندازہ'),
    e('estimate the sales of the next 3 months from the sales register'),
  ]),
  forecast('R28', 'view=R28;ahead=3', 'est.commission.a3', [
    r('income report ke hisab se agle 3 mahine ki aamdan ka andaza'),
    u('انکم رپورٹ کے حساب سے اگلے 3 مہینوں کی آمدن کا اندازہ'),
    e('estimate the income of the next 3 months'),
  ]),
  forecast('R29', 'view=R29;ahead=3', 'est.shop_expenses.a3', [
    r('agle 3 mahine ke dukan ke kharche ka andaza batao'),
    u('اگلے 3 مہینوں کے دکان کے خرچے کا اندازہ بتائیں'),
    e('estimate the shop expenses of the next 3 months'),
  ]),
  forecast('R30', 'view=R30;ahead=3', 'est.profit.a3', [
    r('agle 3 mahine ka munafa kitna rahega'),
    u('اگلے 3 مہینوں کا منافع کتنا رہے گا'),
    e('what profit can we expect in the next 3 months'),
  ]),
  // one buyer, one seller, one item
  forecast('R5', 'buyer=Faisal;ahead=3', 'est_party.faisal.sales.a3', [
    u('فیصل کی اگلے 3 مہینوں کی خریداری کا اندازہ'),
    e('estimate how much Faisal will buy in the next 3 months'),
    r('faisal agle 3 mahine mein kitne ka maal khareedega'),
  ]),
  forecast('R6', 'seller=Rahim;ahead=3', 'est_party.rahim.sales.a3', [
    e("estimate the sales of Rahim's goods for the next 3 months"),
    r('rahim ke maal ki agle 3 mahine ki bikri ka andaza'),
    u('رحیم کے مال کی اگلے 3 مہینوں کی بکری کا اندازہ'),
  ]),
  forecast('R7', 'item=Banana;ahead=3', 'est_item.banana.sales.a3', [
    r('kele ki agle 3 mahine ki bikri kitni hogi'),
    u('کیلے کی اگلے 3 مہینوں کی بکری کتنی ہو گی'),
    e('how much banana will sell in the next 3 months'),
  ]),
  forecast('view-C', 'buyer=Tariq Mehmood;ahead=3', 'est_party.tariq.sales.a3', [
    u('طارق محمود اگلے 3 مہینوں میں کتنے کا مال خریدے گا'),
    e('how much will Tariq Mehmood buy in the next 3 months'),
    r('tariq mehmood agle 3 mahine mein kitna maal lega'),
  ]),
  forecast('tile-today_commission', 'view=tiles;ahead=1', 'est.commission.a1', [
    e('estimate the commission of this whole month'),
    r('is poore mahine ka commission kitna hoga'),
    u('اس پورے مہینے کا کمیشن کتنا ہو گا'),
  ]),
  forecast('tile-today_expenses', 'view=tiles;ahead=1', 'est.shop_expenses.a1', [
    r('is poore mahine dukan ka kharcha kitna hoga'),
    u('اس پورے مہینے دکان کا خرچہ کتنا ہو گا'),
    e('estimate the shop expenses of this whole month'),
  ]),
  // real figure + forecast
  mixed(
    'R2',
    'view=R2;ahead=3',
    '@r2.year.total.value|@r2.year.total.buyer_total',
    'est.sales.a3',
    [
      r('is saal ki sales register ka total aur agle 3 mahine ka andaza'),
      u('اس سال کی سیلز رجسٹر کا ٹوٹل اور اگلے 3 مہینوں کا اندازہ'),
      e('the sales register total this year and the estimate for the next 3 months'),
    ],
  ),
  mixed(
    'R5',
    'buyer=Faisal;ahead=3',
    '@r5.faisal.year.total.buyer_total|@r5.faisal.year.total.value',
    'est_party.faisal.sales.a3',
    [
      u('فیصل نے اس سال کتنا خریدا اور اگلے 3 مہینوں کا اندازہ'),
      e('how much did Faisal buy this year and what is the estimate for the next 3 months'),
      r('faisal ne is saal kitna khareeda aur agle 3 mahine ka andaza'),
    ],
  ),
  mixed('R6', 'seller=Rahim;ahead=3', '@r6.rahim.year.total.gross', 'est_party.rahim.sales.a3', [
    e("Rahim's gross sale this year and the estimate for the next 3 months"),
    r('rahim ki is saal ki khaam bikri aur agle 3 mahine ka andaza'),
    u('رحیم کی اس سال کی خام بکری اور اگلے 3 مہینوں کا اندازہ'),
  ]),
  mixed('R7', 'item=Banana;ahead=3', '@a.year.banana.gross', 'est_item.banana.sales.a3', [
    r('kele ki is saal ki bikri aur agle 3 mahine ka andaza'),
    u('کیلے کی اس سال کی بکری اور اگلے 3 مہینوں کا اندازہ'),
    e('banana sales this year and the estimate for the next 3 months'),
  ]),
  mixed(
    'view-C',
    'buyer=Tariq Mehmood;ahead=3',
    '@c.year.tariq_mehmood.purchase_value',
    'est_party.tariq.sales.a3',
    [
      u('طارق محمود نے اس سال کتنا خریدا اور اگلے 3 مہینوں کا اندازہ'),
      e('how much did Tariq Mehmood buy this year, and the next 3 months estimated'),
      r('tariq mehmood ki is saal ki khareedari aur agle 3 mahine ka andaza'),
    ],
  ),
  mixed(
    'tile-today_commission',
    'view=tiles;ahead=1',
    '@tiles.commission_today',
    'est.commission.a1',
    [
      e("today's commission and the estimate of the whole month"),
      r('aaj ka commission aur poore mahine ka andaza'),
      u('آج کا کمیشن اور پورے مہینے کا اندازہ'),
    ],
  ),
];
