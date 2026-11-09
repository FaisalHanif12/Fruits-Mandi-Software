// The capability matrix of "ask the app" (plan 55 Section 11): the ROWS, read from the code, and what
// the assistant can do with each in every column. `build-capability.mjs` turns this into
// `capability.tsv` and counts the bank questions that reach each cell.
//
// Rows come from the code, not from memory:
//   reports        backend/crates/reports/src/reports/mod.rs    SPECS (R1 to R38) and their filters
//   analytics      backend/crates/reports/src/analytics/mod.rs  VIEWS (A to K)
//   tiles          backend/crates/assistant/src/tools/tiles.rs  the tiles of the dashboard
//   records        backend/crates/assistant/src/tools/find.rs   the roles of `find`
//   masters        PRD 10.3: city, item, packing, adda, mark, store, narration, expense head, staff
//   topics         scripts/ai-eval/run.mjs HELP_TOPICS (the 38 topics of the guide) and the screens of
//                  backend/crates/assistant/src/model_answer.rs EntryScreenName
//   outside        backend/crates/assistant/src/understand/mod.rs OutsideKind
//   figures        partner, garden, trip and fruit figures (PRD 12.10, 12.11, 16.7, 17)
//
// Columns (the question types): figure, list, top, compare, day (a party's or a shop's one day),
// print (print or open), howto, forecast (a forecast or a what-if), advice, mixed (a real figure and a
// forecast in ONE question), nosuch (no such party, or the wrong role), outside (not in the app), change
// (a request to write).
//
// A cell is one of:
//   A:<route>        the assistant CAN answer: the tool and the filter that give it
//   G:<id>:<what>    a GAP: it should answer and cannot; <id> is in GAPS below
//   N:<answer>       NOT BY DESIGN: the honest answer it gives instead (the lead approves it)
//   X:<why>          no such question exists for this row (a cell that is not meaningful)
//
// Gap ids: F = forecast, Q = read (a tool is missing), P = a filter the report tool does not pass,
// W = print or open has no word-to-report plan, S = an entry screen is missing.

export const QTYPES = [
  'figure',
  'list',
  'top',
  'compare',
  'day',
  'print',
  'howto',
  'forecast',
  'advice',
  'mixed',
  'nosuch',
  'outside',
  'change',
];

/** The gaps, with what is missing and who fixes it (first list sent to the lead on 05-11-2026). */
export const GAPS = {
  F1: [
    'M',
    'backend',
    'estimate has 5 measures with no series (commission_buyer, laga, receipts, garden_sales, trip_sales): they always fail',
  ],
  F2: [
    'M',
    'backend',
    'no forecast per expense head: no head in estimate, no per-head series in view G, no head filter in R29',
  ],
  F3: ['m', 'backend', 'no partner in estimate although view G takes a partner'],
  F4: [
    'M',
    'lead+backend',
    "no measure for a party's balance or dues (Umar's own mixed example); needs a ruling",
  ],
  F5: ['m', 'backend', 'no cash, stock or total dues projection'],
  F6: [
    'M',
    'backend3',
    'an item, seller or buyer with months of no sale (apple, mosambi, mango, seller Karim) gets no estimate: a month with no sale inside its active range must count as 0',
  ],
  Q1: ['M', 'backend', 'no tool reads the commission and charge rules'],
  Q2: ['M', 'backend', 'no tool reads the attention list (آج کے کام, the bell)'],
  Q3: ['m', 'backend', 'no tool reads the day and month state (closed up to, month closed)'],
  Q4: ['m', 'backend', 'no tool reads the audit log, and no audit screen name to open'],
  Q4b: ['m', 'lead', 'the guide has no audit-log topic although the audit screen exists (D6 v1.5)'],
  Q5: [
    'm',
    'backend',
    'find has no role for packing, mark, store, narration, expense head or staff (only collector)',
  ],
  Q6: ['m', 'backend', 'the report tool takes a bikri by number only, not by truck number'],
  P1: ['M', 'backend', 'report tool has no `guests` filter (R37)'],
  P2: [
    'm',
    'backend',
    'report tool has no voucher_type (R15), status (R38), detail (R14), trip_view (R33), garden_view (R36)',
  ],
  P3: [
    'm',
    'backend',
    'report tool `party` goes to the BUYER on R2-R5, R7, R27: no seller or zamindar filter there',
  ],
  P4: ['m', 'backend', 'no collector filter in any report (R21, R22)'],
  W1: [
    'm',
    'backend2',
    'print or open has no word-to-report plan for this report (the model answers, with no steps)',
  ],
  W2: ['m', 'backend2', 'print or open of an analytics view has no plan'],
  W3: [
    'm',
    'backend2',
    'print or open of a named item has no plan (no link carries an item filter)',
  ],
  S1: [
    'm',
    'backend2',
    'no entry screen name for garden entries, trip entries, later sale, edit or cancel',
  ],
};

const cell = (s) => s;
const A = (route) => `A:${route}`;
const G = (id) => `G:${id}:${GAPS[id]?.[2] ?? id}`;
const N = (text) => `N:${text}`;
const X = (why = 'not meaningful') => `X:${why}`;

// ---- reports -----------------------------------------------------------------------------------------

// The reports the print/open plan knows (understand/plan.rs, plan 55 ruling 1).
const PLANNED = new Set([
  'R35',
  'R5',
  'R13',
  'R14',
  'R6',
  'R34',
  'R2',
  'R31',
  'R12',
  'R29',
  'R20',
  'R24',
  'R18',
  'R10',
  'R11',
  'R36',
  'R33',
  'R37',
  'R21',
  'R8',
  // Plan 55 ruling 1, built by backend2 (tests/understand_reports.rs): every report but R1.
  'R3',
  'R4',
  'R7',
  'R15',
  'R16',
  'R17',
  'R19',
  'R22',
  'R23',
  'R25',
  'R26',
  'R27',
  'R28',
  'R30',
  'R32',
  'R38',
]);

/** A report row. `o` changes a cell; everything else takes the default of a report with dates. */
function report(id, name, filters, o = {}) {
  const route = (extra = '') => A(`report ${id}${extra}`);
  const dated = filters.includes('from');
  const asOf = filters.includes('as_of');
  const cells = {
    figure: route(),
    list: route(' rows'),
    top: route(' sort+rows'),
    compare: dated || asOf ? A(`report ${id} twice (two periods) + calculate`) : X('no period'),
    day: dated ? route(' from=to') : asOf ? route(' as_of=day') : X('no day'),
    print: PLANNED.has(id) ? A(`local plan ${id} (button + steps)`) : G('W1'),
    howto: X('covered by the cell topic-reports / howto'),
    forecast: X('not a monthly series'),
    advice: A(`${id} read + the model's advice from the rows`),
    mixed: X('no forecast of this report'),
    nosuch: X('the report has no named record'),
    outside: X('not an outside question'),
    change: X('a report is read only'),
    ...o,
  };
  return {
    id,
    kind: 'report',
    name,
    source: `reports/mod.rs SPECS (${filters})`,
    cells,
  };
}

const forecastOf = (measure, extra = '') => A(`estimate measure=${measure}${extra}`);
const mixedOf = (fc) =>
  fc.startsWith('A:') ? A(`${fc.slice(2)} + the real figure in ONE question`) : fc;

const REPORTS = [
  report('R1', 'Sale book (one bikri)', 'bikri', {
    top: X('one bikri'),
    compare: X('one bikri'),
    day: X('read by number; a day is R2/R34'),
    print: A('local plan R1 by the bikri number (backend2)'),
    nosuch: A('report R1 bikri=N: ERR_NOT_FOUND, the model says no such bikri'),
    advice: X('one bill'),
    figure: A('report R1 bikri=N; by truck number: G Q6'),
  }),
  report('R2', 'Full sales register', 'from,to,party(buyer),seller,zamindar,item,city,adda', {
    forecast: forecastOf('sales'),
    list: A('report R2 rows; a seller filter: G P3'),
  }),
  report('R3', 'Cash sales', 'from,to,party,seller,item,city,adda', {
    forecast: N('the app forecasts all sales, not cash sales alone'),
    advice: X('a register'),
  }),
  report('R4', 'Credit sales', 'from,to,party,seller,item,city,adda', {
    forecast: N('the app forecasts all sales, not credit sales alone'),
  }),
  report('R5', 'Buyer bills (credit detail)', 'from,to,party,seller,item,city,adda', {
    forecast: forecastOf('sales', ' buyer=X'),
  }),
  report('R6', 'Seller details (per lot)', 'from,to,seller,zamindar,item,adda', {
    forecast: forecastOf('sales', ' seller=X'),
    nosuch: A('find seller: not_found with asked_role/found_as'),
  }),
  report('R7', 'Item-wise sales', 'from,to,party,seller,item,city,adda', {
    forecast: forecastOf('sales', ' item=X'),
    print: A('local plan R7 (button + steps)'),
  }),
  report('R8', 'Truck stand report', 'from,to,adda', {
    forecast: N('the app has no forecast per adda'),
    nosuch: A('find adda: not_found'),
  }),
  report('R10', 'Unsold stock', 'from,to,seller,item', {
    compare: X('stock is read as of now'),
    day: X('stock is read as of now'),
    forecast: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast the stock; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
    mixed: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast the stock; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
    advice: A('R10 days waiting: what to sell first'),
  }),
  report('R11', 'Day book (روزنامچہ)', 'from,to,party', {
    compare: X('a book'),
    top: A('report R11 sort amount'),
    advice: X('a book'),
  }),
  report('R12', 'Cash book (روکڑ)', 'from,to', {
    forecast: forecastOf('cash_in_hand', ' (the balance now plus the average change of a month)'),
    mixed: mixedOf(
      forecastOf('cash_in_hand', ' (the balance now plus the average change of a month)'),
    ),
  }),
  report('R13', 'Party ledger (کھاتہ)', 'from,to,party*', {
    forecast: X('a ledger'),
    nosuch: A('find buyer/seller: not_found'),
  }),
  report('R14', 'Party statement (فرد حساب)', 'from,to,party*,detail', {
    figure: A('report R14; the detail switch: G P2'),
    forecast: X('a statement'),
    nosuch: A('find buyer/seller: not_found'),
  }),
  report('R15', 'Vouchers list', 'from,to,party,voucher_type', {
    figure: A('report R15'),
    list: A('report R15 voucher_type: the ask "sirf wasooli ki list"'),
    print: A('local plan R15 (button + steps)'),
    forecast: X('a register'),
    advice: X('a register'),
  }),
  report('R16', 'Account index with balances', 'as_of,city', {
    print: A('local plan R16 (button + steps)'),
    compare: X('as of one day'),
    forecast: X('not a series'),
    advice: X('a list'),
  }),
  report('R17', 'Head balances', 'as_of', {
    top: X('few heads'),
    compare: X('as of one day'),
    print: A('local plan R17 (button + steps)'),
    forecast: X('not a series'),
    advice: X('accounts'),
  }),
  report('R18', 'Trial balance', 'as_of', {
    top: X('totals'),
    compare: X('as of one day'),
    forecast: X('not a series'),
    advice: X('accounts'),
  }),
  report('R19', 'Opening trial balance', '', {
    list: A('report R19 rows'),
    top: X('openings'),
    compare: X('openings'),
    day: X('openings'),
    print: A('local plan R19 (button + steps)'),
    forecast: X('openings'),
    advice: X('openings'),
  }),
  report('R20', 'Buyer dues (اگراہی)', 'as_of,party,city', {
    forecast: forecastOf('buyers_dues', ' (R20 total, the same way)'),
    mixed: mixedOf(forecastOf('buyers_dues', ' (R20 total, the same way)')),
  }),
  report('R21', 'Dues by city', 'as_of,party,city', {
    forecast: X('not a series'),
    nosuch: A('find city: not_found'),
  }),
  report('R22', 'Dues by collector', 'as_of,party,city', {
    figure: A('report R22; one collector: G P4'),
    list: A('report R22 rows'),
    print: A('local plan R22 (button + steps)'),
    forecast: X('not a series'),
    compare: X('rare'),
  }),
  report('R23', 'Receipts register', 'from,to,party', {
    forecast: forecastOf('receipts'),
    mixed: mixedOf(forecastOf('receipts')),
    print: A('local plan R23 (button + steps)'),
  }),
  report('R24', 'Seller payables', 'as_of,party', {
    forecast: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast what is payable to sellers; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
    mixed: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast what is payable to sellers; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
    nosuch: A('find seller: not_found'),
  }),
  report('R25', 'Payments register', 'from,to,party', {
    forecast: N('the app has no forecast of payments'),
    print: A('local plan R25 (button + steps)'),
  }),
  report('R26', 'Advances (پیشگی)', 'as_of,seller', {
    top: A('report R26 sort open'),
    compare: X('as of one day'),
    forecast: X('rare'),
    print: A('local plan R26 (button + steps)'),
    nosuch: A('find seller: not_found'),
  }),
  report('R27', 'Bill series', 'from,to,seller,zamindar,item,adda', {
    compare: X('a series'),
    print: A('local plan R27 (button + steps)'),
    forecast: X('a series of numbers'),
    advice: X('a series'),
  }),
  report('R28', 'Income report', 'from,to,party,group_by', {
    forecast: forecastOf('commission_all'),
    mixed: mixedOf(forecastOf('commission_all')),
    print: A('local plan R28 (button + steps)'),
  }),
  report('R29', 'Expense report (by head)', 'from,to,partner', {
    forecast: forecastOf('shop_expenses', ' (total); one head: G F2'),
    mixed: mixedOf(forecastOf('shop_expenses')),
    nosuch: X('a head is a row, not a record to find'),
  }),
  report('R30', 'Profit and partner split', 'from,to', {
    forecast: forecastOf('profit', ' (shop); one partner: G F3'),
    mixed: mixedOf(forecastOf('profit')),
    print: A('local plan R30 (button + steps)'),
  }),
  report('R31', 'Partner statement (کل رقم)', 'from,to,partner*', {
    forecast: forecastOf('profit', ' with a partner (his profit share)'),
    mixed: mixedOf(forecastOf('profit', ' with a partner (his profit share)')),
    advice: A('R31 read; partner section locked: the question pauses'),
    nosuch: A('find partner: not_found'),
  }),
  report('R32', 'Money lent (ادھار رقم)', 'as_of,party', {
    compare: X('as of one day'),
    forecast: X('rare'),
    print: A('local plan R32 (button + steps)'),
    nosuch: A('find money_lent: not_found'),
  }),
  report('R33', 'Trading profit / loss (trips)', 'from,to,item,trip_view', {
    forecast: forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)'),
    mixed: mixedOf(forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)')),
    compare: X('trips are few'),
    nosuch: A('find trip: not_found'),
  }),
  report('R34', 'Day summary', 'from,to', {
    top: X('one summary'),
    forecast: X('a summary'),
  }),
  report('R35', 'Party day sheet', 'as_of,party', {
    top: A('report R35 sort'),
    compare: X('one day'),
    forecast: X('a slip'),
    advice: X('a slip'),
    nosuch: A('find buyer/seller: not_found'),
  }),
  report('R36', 'Gardens report', 'from,to,item,partner,garden,garden_view', {
    compare: X('gardens are few'),
    forecast: forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)'),
    mixed: mixedOf(forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)')),
    nosuch: A('find garden: not_found'),
  }),
  report('R37', 'Fruit taken register', 'from,to,partner,guests,item,garden', {
    figure: A('report R37; guests only: G P1'),
    forecast: N('the app has no forecast of fruit taken'),
    advice: X('a register'),
  }),
  report('R38', 'Dead amount', 'status', {
    day: X('status only'),
    compare: X('status only'),
    print: A('local plan R38 (button + steps)'),
    forecast: X('rare'),
  }),
];

// ---- analytics views ---------------------------------------------------------------------------------

function view(v, name, filters, o = {}) {
  const route = (extra = '') => A(`analytics ${v}${extra}`);
  const cells = {
    figure: route(),
    list: route(' rows'),
    top: route(' sort+rows'),
    compare:
      v === 'G' || v === 'J'
        ? A('two months of the series + calculate')
        : route(' compare=previous|last_year'),
    day: route(' from=to'),
    print: A('local plan: the analytics view by its title (backend2, tests/understand_reports.rs)'),
    howto: X('covered by the cell topic-analytics / howto'),
    forecast: X('not a series'),
    advice: A(`analytics ${v} read + the model's advice`),
    mixed: X('no forecast of this view'),
    nosuch: X('the view has no named record'),
    outside: X('not an outside question'),
    change: X('analytics is read only'),
    ...o,
  };
  return {
    id: `view-${v}`,
    kind: 'analytics',
    name,
    source: `analytics/mod.rs VIEWS (${filters})`,
    cells,
  };
}

const VIEWS = [
  view('A', 'By item', 'period,item,seller,buyer,city,adda,side,trip,goods,garden,compare', {
    forecast: forecastOf('sales', ' item=X (or commission)'),
    mixed: mixedOf(forecastOf('commission_all', ' item=X')),
    nosuch: A('find item: not_found'),
  }),
  view('B', 'By seller', 'period,item,seller,...', {
    forecast: forecastOf('sales', ' seller=X'),
    mixed: mixedOf(forecastOf('sales', ' seller=X')),
    nosuch: A('find seller: not_found'),
  }),
  view('C', 'By buyer', 'period,item,buyer,...', {
    forecast: forecastOf('sales', ' buyer=X (his purchases)'),
    mixed: mixedOf(forecastOf('sales', ' buyer=X')),
    nosuch: A('find buyer: not_found'),
  }),
  view('D', 'Item by period', 'period,group_by,...', {
    forecast: X('a table of months; see view G'),
  }),
  view('E', 'Seller x item', 'period,...', {
    forecast: X('a table'),
    compare: A('analytics E compare'),
  }),
  view('F', 'By partner', 'period,partner,...', {
    forecast: forecastOf('profit', ' with a partner (his profit share)'),
    mixed: mixedOf(forecastOf('profit', ' with a partner (his profit share)')),
    advice: A('analytics F read; the partner section may be locked'),
    nosuch: A('find partner: not_found'),
  }),
  view('G', 'Trend (monthly series)', 'period,group_by,partner,...', {
    forecast: forecastOf('sales|commission|shop_expenses|profit'),
    mixed: mixedOf(forecastOf('sales|commission|shop_expenses|profit')),
  }),
  view('H', 'Top 10', 'period,...', {
    forecast: X('a ranking'),
    top: A('analytics H'),
    print: G('W2'),
  }),
  view('I', 'Gardens', 'period,item,side,partner,garden', {
    forecast: forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)'),
    mixed: mixedOf(forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)')),
    compare: A('analytics I compare'),
    nosuch: A('find garden: not_found'),
  }),
  view('J', 'Market trips', 'period,...,trip_view', {
    forecast: forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)'),
    mixed: mixedOf(forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)')),
    nosuch: A('find trip: not_found'),
  }),
  view('K', 'Fruit taken', 'period,item,partner,garden,goods', {
    forecast: N('the app has no forecast of fruit taken'),
    figure: A('analytics K (the guests row is a row of the view)'),
    nosuch: A('find partner: not_found'),
  }),
];

// ---- dashboard tiles ---------------------------------------------------------------------------------

function tile(name, note, o = {}) {
  return {
    id: `tile-${name}`,
    kind: 'tile',
    name: note,
    source: 'tools/tiles.rs',
    cells: {
      figure: A(`tiles ${name}`),
      list: X('a single figure'),
      top: X('a single figure'),
      compare: X('read now; compare with analytics'),
      day: X('read now'),
      print: A('local: open the dashboard'),
      howto: X('covered by the cell topic-dashboard / howto'),
      forecast: X('a tile has no series'),
      advice: A('tile read + the advice'),
      mixed: X('no forecast of this tile'),
      nosuch: X('no named record'),
      outside: X('not an outside question'),
      change: X('a tile is read only'),
      ...o,
    },
  };
}

const TILES = [
  tile('today_sales', "Today's sales", {
    forecast: forecastOf('sales'),
    mixed: mixedOf(forecastOf('sales')),
    compare: A('analytics A today vs yesterday'),
  }),
  tile('today_commission', "Today's commission", {
    forecast: forecastOf('commission_all'),
    mixed: mixedOf(forecastOf('commission_all')),
    compare: A('analytics A today vs yesterday'),
  }),
  tile('cash_in_hand', 'Cash in hand', {
    forecast: forecastOf('cash_in_hand', ' (the balance now plus the average change of a month)'),
    mixed: mixedOf(
      forecastOf('cash_in_hand', ' (the balance now plus the average change of a month)'),
    ),
    compare: A('R12 two days + calculate'),
  }),
  tile('buyers_dues', "Buyers' dues", {
    forecast: forecastOf('buyers_dues', ' (R20 total, the same way)'),
    mixed: mixedOf(forecastOf('buyers_dues', ' (R20 total, the same way)')),
  }),
  tile('sellers_dues', 'Sellers owe the shop', {}),
  tile('total_dues', 'Total dues', {
    forecast: forecastOf(
      'buyers_dues',
      ' (the tile total is the real figure; the estimate is of the buyers dues only)',
    ),
    mixed: mixedOf(
      forecastOf(
        'buyers_dues',
        ' (the tile total is the real figure; the estimate is of the buyers dues only)',
      ),
    ),
  }),
  tile('dead_amount', 'Dead amount', {}),
  tile('sellers_payable', "Sellers' payable", {
    forecast: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast what is payable to sellers; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
    mixed: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast what is payable to sellers; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
  }),
  tile('advances_open', 'Open advances', {}),
  tile('stock', 'Unsold stock', {
    forecast: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast the stock; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
    mixed: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast the stock; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
  }),
  tile('trips', 'Market trips', {
    forecast: forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)'),
    mixed: mixedOf(forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)')),
  }),
  tile('gardens', 'Gardens', {
    forecast: forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)'),
    mixed: mixedOf(forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)')),
  }),
  tile('money_lent_open', 'Money lent open', {}),
  tile('today_expenses', "Today's expenses", {
    forecast: forecastOf('shop_expenses'),
    mixed: mixedOf(forecastOf('shop_expenses')),
  }),
  tile('last_backup', 'Last backup', { advice: A('tile read: when to back up') }),
  {
    id: 'dash-attention',
    kind: 'tile',
    name: 'Attention list (آج کے کام, the bell)',
    source: 'PRD FR-7.44, FR-9.3',
    cells: {
      figure: A('app_state attention (answerable after prompt v1.5)'),
      list: A('app_state attention (answerable after prompt v1.5)'),
      top: X('a list'),
      compare: X('now'),
      day: X('now'),
      print: A('local: open the dashboard'),
      howto: A('app_help dashboard'),
      forecast: X('now'),
      advice: A('app_state attention (answerable after prompt v1.5)'),
      mixed: X('now'),
      nosuch: X('no named record'),
      outside: X('not an outside question'),
      change: X('read only'),
    },
  },
  {
    id: 'dash-day-state',
    kind: 'tile',
    name: 'Day state and month close (closed up to, month closed)',
    source: 'PRD FR-6.6, FR-21.1',
    cells: {
      figure: A('app_state days (answerable after prompt v1.5)'),
      list: A('app_state days (answerable after prompt v1.5)'),
      top: X('a state'),
      compare: X('a state'),
      day: A('app_state days (answerable after prompt v1.5)'),
      print: A('local: open day_close or month_close'),
      howto: A('app_help day_close, month_close'),
      forecast: X('a state'),
      advice: A('app_state days (answerable after prompt v1.5)'),
      mixed: X('a state'),
      nosuch: X('no named record'),
      outside: X('not an outside question'),
      change: A('local: day_close or month_close screen'),
    },
  },
  {
    id: 'dash-audit',
    kind: 'tile',
    name: 'Audit log (who changed what)',
    source: 'PRD FR-21.3',
    cells: {
      figure: A('app_state audit (answerable after prompt v1.5)'),
      list: A('app_state audit (answerable after prompt v1.5)'),
      top: X('a log'),
      compare: X('a log'),
      day: A('app_state audit (answerable after prompt v1.5)'),
      print: A('local: the audit log opens by words with a screen button (backend2)'),
      howto: X('covered by the cell topic-audit / howto (the guide topic audit, prompt v1.5)'),
      forecast: X('a log'),
      advice: X('a log'),
      mixed: X('a log'),
      nosuch: X('no named record'),
      outside: X('not an outside question'),
      change: X('the log cannot be edited'),
    },
  },
  {
    id: 'rules',
    kind: 'master',
    name: 'Commission and charge rules (کمیشن اصول)',
    source: 'PRD 11; screen commission_rules',
    cells: {
      figure: A('app_state rules (answerable after prompt v1.5)'),
      list: A('app_state rules (answerable after prompt v1.5)'),
      top: X('few rules'),
      compare: X('rules'),
      day: A('app_state rules (answerable after prompt v1.5)'),
      print: A('local: open the commission_rules screen'),
      howto: A('app_help commission_rules'),
      forecast: X('rules'),
      advice: A('app_state rules (answerable after prompt v1.5)'),
      mixed: X('rules'),
      nosuch: A('app_state rules (answerable after prompt v1.5)'),
      outside: X('not an outside question'),
      change: A('local: commission_rules screen'),
    },
  },
];

// ---- records (the roles of find) ---------------------------------------------------------------------

function record(id, name, source, o) {
  return {
    id: `record-${id}`,
    kind: 'record',
    name,
    source,
    cells: {
      figure: X('x'),
      list: X('x'),
      top: X('x'),
      compare: X('x'),
      day: X('x'),
      print: X('x'),
      howto: X('covered by the topic row of its screen'),
      forecast: X('x'),
      advice: X('x'),
      mixed: X('x'),
      nosuch: A(`find ${id}: not_found with asked_role (and found_as in another role)`),
      outside: X('not an outside question'),
      change: X('x'),
      ...o,
    },
  };
}

const RECORDS = [
  record('buyer', 'A buyer (گاہک)', 'find.rs Role::Buyer', {
    figure: A('party_card balance, dues, credit used; report R13/R20'),
    list: A('report R20 rows (all buyers)'),
    top: A('report R20 sort balance; analytics C, H'),
    compare: A('analytics C compare; R13 two periods'),
    day: A('R35, R5 with the day; buyer_day'),
    print: A('local plan R35 / R5 + R13'),
    forecast: forecastOf('sales', ' buyer=X (his purchases)'),
    advice: A('R20 + party_card: whom to collect from'),
    mixed: mixedOf(forecastOf('sales', ' buyer=X')),
    change: A('local: wasooli screen with the buyer'),
  }),
  record('seller', 'A seller (بیوپاری)', 'find.rs Role::Seller', {
    figure: A('party_card payable, advance; report R24, R13'),
    list: A('report R24 rows'),
    top: A('report R24 sort; analytics B'),
    compare: A('analytics B compare'),
    day: A('R6 with the day; seller_day'),
    print: A('local plan R6 + R13'),
    forecast: forecastOf('sales', ' seller=X'),
    advice: A('R24: whom to pay first'),
    mixed: mixedOf(forecastOf('sales', ' seller=X')),
    change: A('local: adaigi or peshgi screen with the seller'),
  }),
  record('chukta', 'A walk-in buyer (چکتا کھاتہ)', 'find.rs Role::Chukta', {
    figure: N('a چکتا buyer pays in full and has no balance; find gives his name and phone'),
    list: N('the register of one-time parties is gone (PRD v1.32)'),
    day: N('a چکتا buyer is in the day sales (R2) like any buyer, with no ledger'),
    print: N('there is no report of چکتا buyers'),
    change: A('local: chukta_khata screen'),
  }),
  record('money_lent', 'A person lent money (ادھار رقم)', 'find.rs Role::MoneyLent', {
    figure: A('report R32 + find money_lent'),
    list: A('report R32 rows'),
    day: A('report R32 as_of'),
    print: A('local plan R32 (button + steps)'),
    advice: A('R32 overdue: whom to ask'),
    change: A('local: udhar_raqam screen'),
  }),
  record('separate_khata', 'A separate khata (الگ کھاتہ)', 'find.rs Role::SeparateKhata', {
    figure: A('party_card + report R13'),
    list: A('report R13 rows'),
    compare: A('R13 two periods'),
    day: A('R13 from=to'),
    print: A('local plan R13/R14'),
    change: A('local: alag_khata screen'),
  }),
  record('partner', 'A partner (شراکت دار)', 'find.rs Role::Partner', {
    figure: A('report R31 (کل رقم), R30, analytics F; partner section lock'),
    list: A('report R30 rows; analytics F rows'),
    top: A('analytics F sort'),
    compare: A('analytics F compare'),
    day: A('report R31 from=to'),
    print: A('local plan R31'),
    forecast: forecastOf('profit', ' with a partner (his profit share)'),
    advice: A('R31/R30 read'),
    mixed: mixedOf(forecastOf('profit', ' with a partner (his profit share)')),
    change: A('local: kharcha, sarmaya or phal screen with the partner'),
  }),
  record('garden', 'A garden (باغ)', 'find.rs Role::Garden', {
    figure: A('report R36, analytics I'),
    list: A('report R36 rows'),
    top: A('analytics I sort'),
    day: A('report R36 from=to'),
    print: A('local plan R36'),
    forecast: forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)'),
    advice: A('R36 read'),
    mixed: mixedOf(forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)')),
    change: A('local: gardens screen (entries: G S1)'),
  }),
  record('trip', 'A market trip (مارکیٹ سفر)', 'find.rs Role::Trip', {
    figure: A('report R33, analytics J'),
    list: A('report R33 rows'),
    top: A('analytics J sort'),
    day: A('report R33 from=to'),
    print: A('local plan R33 (no one-trip filter: the steps say which row)'),
    forecast: forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)'),
    advice: A('R33 read'),
    mixed: mixedOf(forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)')),
    change: A('local: market_trips screen (entries: G S1)'),
  }),
  record('item', 'An item (قسم مال)', 'find.rs Role::Item', {
    figure: A('analytics A item=X; report R10 item=X (stock)'),
    list: A('analytics A rows; D, E'),
    top: A('analytics A, H sort'),
    compare: A('analytics A compare'),
    day: A('analytics A from=to'),
    print: A('local plan: R7, R10 or R27 with the item (backend2)'),
    forecast: forecastOf('sales|commission_all', ' item=X'),
    advice: A('analytics A: which item to push'),
    mixed: mixedOf(forecastOf('commission_all', ' item=X')),
  }),
  record('city', 'A city (شہر)', 'find.rs Role::City', {
    figure: A('report R21 city=X; analytics A city=X'),
    list: A('report R21 rows'),
    top: A('report R21 sort'),
    compare: A('analytics A city=X compare'),
    print: A('local plan R21'),
    advice: A('R21 read'),
  }),
  record('adda', 'A truck stand (اڈا)', 'find.rs Role::Adda', {
    figure: A('report R8 adda=X; analytics A adda=X'),
    list: A('report R8 rows'),
    top: A('report R8 sort'),
    compare: A('analytics A adda=X compare'),
    print: A('local plan R8'),
  }),
  record('collector', 'A collector (اگراہی والا)', 'find.rs Role::Collector', {
    figure: A('report R22 collector=<staff> (the collector filter, backend)'),
    list: A('report R22 rows (all collectors)'),
    top: A('report R22 sort'),
    print: A('local plan R22 (button + steps)'),
  }),
  record('expense_head', 'An expense head (خرچہ مد)', 'PRD 10.3; reports R29 rows', {
    figure: A('report R29 (the head is a row; no head filter)'),
    list: A('report R29 rows'),
    top: A('report R29 sort'),
    compare: A('R29 two periods + calculate'),
    day: A('report R29 from=to'),
    print: A('local plan R29 (no head filter: the steps say which row)'),
    forecast: forecastOf('shop_expenses', ' with a head (one expense head)'),
    advice: A('R29: which expense to cut'),
    mixed: mixedOf(forecastOf('shop_expenses', ' with a head (one expense head)')),
    nosuch: A('find expense_head: not_found'),
    change: A('local: kharcha screen'),
  }),
  record(
    'item_seasonal',
    'An item with months of no sale (apple, mosambi, mango)',
    'oracle: no est_item keys for them',
    {
      forecast: forecastOf('commission|sales', ' of an item with months of no sale'),
      mixed: mixedOf(forecastOf('commission|sales', ' of an item with months of no sale')),
      nosuch: X('same as record-item'),
    },
  ),
  record('staff', 'Staff (عملہ: munshi, labour)', 'PRD 10.3; find has the collector role only', {
    figure: A('find staff (with count)'),
    list: A('find staff (with count)'),
    nosuch: A('find staff (with count)'),
  }),
  record('packing', 'A packing (پیکنگ)', 'PRD 10.3', {
    figure: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
    list: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
    nosuch: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
  }),
  record('marka', 'A mark (مارکہ)', 'PRD 10.3', {
    figure: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
    list: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
    nosuch: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
  }),
  record('store', 'A store (سٹور)', 'PRD 10.3', {
    figure: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
    list: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
    nosuch: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
  }),
  record('narration', 'A standard narration (تفصیل)', 'PRD 10.3', {
    figure: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
    list: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
    nosuch: A('find packing, marka, store or narration (backend4, tests/state_reads.rs)'),
  }),
  record('truck', 'A truck number (ٹرک نمبر)', 'PRD 12.2; services bikri::find', {
    figure: A('find truck'),
    list: A('find truck'),
    nosuch: A('find truck'),
    day: A('find truck'),
  }),
];

// ---- figures of partners, gardens, trips and fruit --------------------------------------------------

function figure(id, name, source, cells) {
  const base = {
    figure: X('x'),
    list: X('x'),
    top: X('x'),
    compare: X('x'),
    day: X('x'),
    print: X('x'),
    howto: X('covered by the topic row of its screen'),
    forecast: X('x'),
    advice: X('x'),
    mixed: X('x'),
    nosuch: X('x'),
    outside: X('not an outside question'),
    change: X('x'),
  };
  return { id: `figure-${id}`, kind: 'figure', name, source, cells: { ...base, ...cells } };
}

const FIGURES = [
  figure('partner_total', "A partner's total (کل رقم)", 'PRD FR-17.9, C22', {
    figure: A('report R31 closing; partner section lock'),
    list: A('report R31 rows (his dated lines)'),
    compare: A('R31 two dates + calculate'),
    day: A('R31 from=to'),
    print: A('local plan R31'),
    forecast: forecastOf('profit', ' with a partner (his profit share)'),
    advice: A('R31 read'),
    mixed: mixedOf(forecastOf('profit', ' with a partner (his profit share)')),
  }),
  figure('partner_share', "A partner's share (today, month, year)", 'PRD 17.3, R30, analytics F', {
    figure: A('report R30; analytics F'),
    list: A('report R30 rows'),
    top: A('analytics F sort'),
    compare: A('analytics F compare'),
    day: A('report R30 from=to'),
    print: A('local plan R31 (button + steps)'),
    forecast: forecastOf('profit', ' with a partner (his profit share)'),
    mixed: mixedOf(forecastOf('profit', ' with a partner (his profit share)')),
  }),
  figure('partner_capital', "A partner's capital (in, out, balance)", 'PRD 17.4', {
    figure: A('report R31 + capital history'),
    list: A('capital history rows'),
    day: A('R31 from=to'),
    print: A('local plan R31'),
    forecast: X('capital is not forecast'),
    advice: X('capital'),
  }),
  figure('partner_taken', 'Money a partner took and returned (خالص)', 'PRD 16.3, R29, R31', {
    figure: A('report R31, R29 partner=X'),
    list: A('report R29 partner=X rows'),
    day: A('R31 from=to'),
    print: A('local plan R31 / R29'),
  }),
  figure(
    'garden_balance',
    "A garden's running balance and rent due",
    'PRD 12.11, R36, analytics I',
    {
      figure: A('report R36; tiles gardens'),
      list: A('report R36 rows'),
      day: A('R36 from=to'),
      print: A('local plan R36'),
      forecast: forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)'),
      advice: A('R36 read'),
      mixed: mixedOf(forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)')),
    },
  ),
  figure('garden_result', "A garden's installments, costs, sales and result", 'PRD 12.11, R36', {
    figure: A('report R36 rows'),
    list: A('report R36 rows'),
    print: A('local plan R36'),
    forecast: forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)'),
    mixed: mixedOf(forecastOf('garden_sales', ' (all gardens; a garden or an item as a filter)')),
  }),
  figure(
    'trip_result',
    "A trip's profit or loss and its partners' parts",
    'PRD 12.10, R33, analytics J',
    {
      figure: A('report R33; analytics J'),
      list: A('report R33 rows'),
      day: A('R33 from=to'),
      print: A('local plan R33'),
      forecast: forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)'),
      advice: A('R33 read'),
      mixed: mixedOf(forecastOf('trip_sales', ' (all trips; a trip or an item as a filter)')),
    },
  ),
  figure('trip_stock', "A trip's goods left (نگ, cost)", 'PRD FR-12.25, tiles trips, R10', {
    figure: A('tiles trips; report R33 left'),
    list: A('report R10 own goods rows'),
    print: A('local plan R10 / R33'),
    forecast: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast a trip's stock; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
    mixed: N(
      "plan 55 11.2 (lead, 05-11-2026): the app does not forecast a trip's stock; it gives today's real figure with its button, says so, and offers what it can estimate",
    ),
  }),
  figure('fruit_partner', 'Fruit a partner took (qty, value)', 'PRD 16.7, R37, analytics K', {
    figure: A('report R37 partner=X; analytics K'),
    list: A('report R37 rows'),
    top: A('analytics K sort'),
    compare: A('analytics K compare'),
    day: A('report R37 from=to'),
    print: A('local plan R37'),
    forecast: N('the app has no forecast of fruit taken'),
  }),
  figure('fruit_guests', 'Fruit for the shop guests', 'PRD 16.7, R37, analytics K', {
    figure: A('analytics K (the guests row); R37 guests only: G P1'),
    list: A('report R37 partner=guests'),
    day: A('report R37 from=to'),
    print: A('local plan R37 with the guests (backend2)'),
    forecast: N('the app has no forecast of fruit taken'),
  }),
];

// ---- the guide: topics, screens, outside -----------------------------------------------------------

/** topic -> the entry screen name a request to write opens, or null. */
const TOPICS = [
  ['bikri_book', 'bikri'],
  ['today_sales', 'today_sales'],
  ['unsold_stock', 'unsold_stock'],
  ['later_sale', null, 'S1'],
  ['receipt', 'wasooli'],
  ['collection_list', 'collection_list'],
  ['payment', 'adaigi'],
  ['advance', 'peshgi'],
  ['chukta_khata', 'chukta_khata'],
  ['edit_and_cancel', null, 'S1'],
  ['cash_book', 'cash_book'],
  ['shop_expense', 'kharcha'],
  ['personal_expense', 'kharcha'],
  ['money_lent', 'udhar_raqam'],
  ['separate_khata', 'alag_khata'],
  ['general_journal', 'general'],
  ['fruit_taken', 'phal'],
  ['gardens', 'gardens', 'S1'],
  ['market_trips', 'market_trips', 'S1'],
  ['partners', 'partners'],
  ['partner_capital', 'sarmaya'],
  ['analytics', 'analytics'],
  ['reports', 'reports'],
  ['dashboard', 'dashboard'],
  ['day_close', 'day_close'],
  ['month_close', 'month_close'],
  ['backup', 'backup'],
  ['settings', 'settings'],
  ['audit', null],
  ['masters', 'masters'],
  ['accounts', 'accounts'],
  ['search', null],
  ['commission_rules', 'commission_rules'],
  ['dead_amount', 'dead_amount'],
  ['language_switch', null],
  ['keys', null],
  ['printing', null],
  ['ask_the_app', null],
  ['sign_in', null],
];

/** The entry screens (model_answer.rs `is_entry_screen`): the others are main screens. */
const ENTRY_SCREENS = new Set([
  'bikri',
  'wasooli',
  'adaigi',
  'peshgi',
  'kharcha',
  'udhar_raqam',
  'alag_khata',
  'sarmaya',
  'phal',
  'general',
  'account',
]);

const TOPIC_ROWS = TOPICS.map(([topic, screen, gap]) => ({
  id: `topic-${topic}`,
  kind: 'topic',
  name: `Guide topic ${topic}${screen ? ` (screen ${screen})` : ''}`,
  source: 'run.mjs HELP_TOPICS; model_answer.rs EntryScreenName',
  cells: {
    figure: X('a guide topic holds no figure'),
    list: X('x'),
    top: X('x'),
    compare: X('x'),
    day: X('x'),
    print: screen ? A(`local: open the ${screen} screen`) : X('no screen to open'),
    howto: A(`app_help ${topic} (local, steps from the guide)`),
    forecast: X('x'),
    advice: X('x'),
    mixed: X('x'),
    nosuch: X('x'),
    outside: X('not an outside question'),
    // Only the eleven entry screens take a request to write; a main screen is only opened (the print cell).
    change: ENTRY_SCREENS.has(screen)
      ? gap
        ? `A:local: ${screen} screen opens (the specific entry: G ${gap})`
        : A(`local: ${screen} screen with the party`)
      : gap
        ? A(
            `local: the entry screens of ${topic} (backend2, tests/understand_reports.rs: garden_installment, garden_cost, garden_sale, garden_close, trip_purchase, trip_cost, trip_sale, bikri_later_sale, bikri_edit, bikri_cancel)`,
          )
        : X('a main screen takes no entry; opening it is the print cell'),
  },
}));

const OUTSIDE = [
  ['weather', 'The weather'],
  ['tax', 'GST or any tax'],
  ['bank', 'A bank balance'],
  ['market_rate', 'The rate in another market'],
  ['prices', 'The dollar, gold, petrol, the stock market'],
  ['other', 'News, cricket, a joke, anything else the app does not keep'],
].map(([k, name]) => ({
  id: `outside-${k}`,
  kind: 'outside',
  name,
  source: 'understand/mod.rs OutsideKind',
  cells: {
    figure: X('x'),
    list: X('x'),
    top: X('x'),
    compare: X('x'),
    day: X('x'),
    print: X('x'),
    howto: X('x'),
    forecast: X('x'),
    advice: X('x'),
    mixed: X('x'),
    nosuch: X('x'),
    outside: A(`local compose_outside: "the app does not keep this", with the nearest report`),
    change: X('x'),
  },
}));

export const ROWS = [
  ...REPORTS,
  ...VIEWS,
  ...TILES,
  ...RECORDS,
  ...FIGURES,
  ...TOPIC_ROWS,
  ...OUTSIDE,
];

export { cell };
