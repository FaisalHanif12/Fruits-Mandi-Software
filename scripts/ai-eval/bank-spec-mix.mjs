// Facet MIX of the team bank (Umar, 05-11-2026): ONE question that asks for a real figure AND a
// forecast or a what-if. "pichle mahine electricity ka kitna kharcha hua aur agle 3 mahine ka
// estimate kitna hai". The answer must give BOTH parts:
//
//   real_*   the figure of the books, from the oracle (never shown as an estimate)
//   est_*    the estimate of the months ahead (oracle `est.*` facts, worked out by the app's own
//            domain forecast), shown as an estimate with its working, and never as a book figure
//            (PRD FR-7.76 point 4). The app shows each method (same pace, same growth, same season):
//            the band. Any one of them counts.
//
// The scorer reads the prefixes real_ and est_ of the labels (run.mjs, mixSeparation).
//
// `pending=<reason>` in the filters: the question has no route yet (the expense-head estimate waits
// for backend's head filter; a party's balance and the cash have no forecast measure at all, gaps
// F2, F4 and F5 of capability.tsv). Such a question is PENDING in the report, neither a pass nor a
// miss, and the run cannot say "meets" while it is. Remove the flag when the route is built.

const r = (text) => ['roman', text];
const rm = (text) => ['roman', text, 'mis'];
const u = (text) => ['urdu', text];
const e = (text) => ['en', text];
const em = (text) => ['en', text, 'mis'];

const three = (key, measure, ahead) =>
  ['same_pace', 'same_growth', 'same_season'].map((m) => `@${key}.${ahead}.${m}`).join('|');
const yearOf = (prefix) =>
  ['year_end', 'year_total']
    .flatMap((k) => ['same_pace', 'same_growth', 'same_season'].map((m) => `@${prefix}.${k}.${m}`))
    .join('|');

const mix = (g, section, filters, figures, q, over = {}) => ({
  g,
  section,
  kind: 'mixed_forecast',
  world: 'year',
  outcome: 'estimate',
  filters: `facet=MIX;${filters}`,
  figures,
  source: 'Umar 05-11-2026',
  q,
  ...over,
});

const MIX = [
  mix(
    'MX01',
    'expense',
    'head=Electricity;period=last month;ahead=3;pending=head_estimate',
    'real_elec=@r29.last_month.electricity.expense',
    [
      r('pichle mahine electricity ka kitna kharcha hua aur agle 3 mahine ka estimate kitna hai'),
      u('پچھلے مہینے بجلی کا کتنا خرچہ ہوا اور اگلے 3 مہینوں کا اندازہ کتنا ہے'),
      e('how much did electricity cost last month and what is the estimate for the next 3 months'),
    ],
  ),
  mix(
    'MX02',
    'item',
    'item=Banana;period=this year;until=year_end',
    `real_comm=@a.year.banana.total_comm;est_year=${yearOf('est_item.banana.commission')}`,
    [
      r('is saal kele ka commission kitna hua aur saal ke aakhir tak kitna hoga'),
      rm('is sal kele ka comision kitna hua aur sal ke akhir tak kitna hoga'),
      e('what was the banana commission this year and what will it be by the end of the year'),
    ],
  ),
  mix(
    'MX03',
    'dues',
    'buyer=Faisal;ahead=2;pending=balance_forecast',
    'real_bal=@buyer.faisal.balance',
    [
      r('faisal ka bakaya kitna hai aur isi raftar se 2 mahine baad kitna hoga'),
      u('فیصل کا بقایا کتنا ہے اور اسی رفتار سے 2 مہینے بعد کتنا ہو گا'),
      em('what does Faisal owe now and how much at this pace after 2 monhts'),
    ],
  ),
  mix(
    'MX04',
    'period',
    'period=today;what_if=double;ahead=1',
    'real_today=@tiles.sales_today;est_double=@est.sales.a1.double_pace',
    [
      r('aaj ki bikri batao aur agar raftar double ho to mahine ka kitna'),
      u('آج کی بکری بتاؤ اور اگر رفتار دگنی ہو جائے تو مہینے کا کتنا'),
      e("tell me today's sales and what the month would be if the pace doubles"),
    ],
  ),
  mix(
    'MX05',
    'period',
    'period=last month;ahead=3',
    `real_last=@a.last_month.total.gross;est_next=${three('est.sales', 'sales', 'a3')}`,
    [
      r('pichle mahine ki bikri kitni thi aur agle 3 mahine ki bikri ka andaza batao'),
      u('پچھلے مہینے کی بکری کتنی تھی اور اگلے 3 مہینوں کی بکری کا اندازہ بتاؤ'),
      e('what were last month sales and estimate the sales of the next 3 months'),
    ],
  ),
  mix(
    'MX06',
    'month',
    'period=this month;ahead=1',
    `real_comm=@a.month.total.total_comm;est_month=${three('est.commission', 'commission', 'a1')}`,
    [
      rm('is mahine commision kitna hua aur mahine ke end tak kitna hoga'),
      u('اس مہینے کمیشن کتنا ہوا اور مہینے کے آخر تک کتنا ہو گا'),
      e('commission this month so far, and the estimate for the whole month'),
    ],
  ),
  mix('MX07', 'cash', 'view=tiles;ahead=1;pending=cash_forecast', 'real_cash=@tiles.cash', [
    r('cash abhi kitna hai aur agle mahine kitna ho jayega'),
    u('ابھی کیش کتنا ہے اور اگلے مہینے کتنا ہو جائے گا'),
    e('how much cash do we have now and how much will we have next month'),
  ]),
  mix(
    'MX08',
    'seller',
    'seller=Rahim;period=this year;ahead=3',
    `real_sales=@b.year.rahim.gross;est_next=${three('est_party.rahim.sales', 'sales', 'a3')}`,
    [
      r('rahim ka is saal kitna maal bika aur agle 3 mahine ka andaza'),
      u('رحیم کا اس سال کتنا مال بکا اور اگلے 3 مہینوں کا اندازہ'),
      e("how much of Rahim's goods sold this year and the estimate for the next 3 months"),
    ],
  ),
  mix(
    'MX09',
    'period',
    'period=this year;until=year_end',
    `real_profit=@r30.year.total.profit_share;est_profit=${yearOf('est.profit')}`,
    [
      r('is saal ka profit kitna hua aur saal ke aakhir tak kitna hoga'),
      u('اس سال کا منافع کتنا ہوا اور سال کے آخر تک کتنا ہو گا'),
      e('profit this year so far and what it will be by the end of the year'),
    ],
  ),
  mix(
    'MX10',
    'expense',
    'period=last month;ahead=3',
    `real_exp=@r29.last_month.total.expense;est_exp=${three('est.shop_expenses', 'shop_expenses', 'a3')}`,
    [
      r('pichle mahine dukan ka kharcha kitna tha aur agle 3 mahine ka estimate'),
      u('پچھلے مہینے دکان کا خرچہ کتنا تھا اور اگلے 3 مہینوں کا اندازہ'),
      e('shop expenses last month and the estimate for the next 3 months'),
    ],
  ),
];

export default MIX;
