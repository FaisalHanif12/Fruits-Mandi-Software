// Facet CAP, part 3 (plan 55 Section 11): one question for a cell of the capability matrix on the
// registers and lists that the oracle did not dump before the run of 05-11-2026: R2 to R7, R11, R13
// to R19, R21, R22, R27, R28, R34 and R35, with the figure from the oracle (the app's own report).
// The same rulings as part 2: one question per cell, languages rotating, about a quarter misspelled;
// three wordings for the cells a mandi owner is likely to ask (the figure of a main report).
//
// Notes on the year world:
// Item and seller days of views B and C are the party's-day questions of facet `day`.

const r = (text) => ['roman', text];
const rm = (text) => ['roman', text, 'mis'];
const u = (text) => ['urdu', text];
const um = (text) => ['urdu', text, 'mis'];
const e = (text) => ['en', text];
const em = (text) => ['en', text, 'mis'];

let rotate = 0;

/** One cell. `n` is the number of wordings kept in the bank: 3 or 1 (the language rotates). */
const cell = (row, qtype, section, filters, figures, q, over = {}) => {
  const { n = 3, ...rest } = over;
  return {
    g: `CR-${row}-${qtype}`,
    cat: 'A',
    section,
    kind: rest.kind ?? section,
    world: 'year',
    outcome: 'answered',
    tools: rest.tools ?? '',
    filters: `facet=CAP;${filters}`,
    figures,
    source: `capability.tsv ${row} / ${qtype}`,
    q,
    ...(n === 1 ? { use: [rotate++ % q.length] } : {}),
    ...rest,
  };
};

const sales = (key) => `@${key}.total.value|@${key}.total.buyer_total`;

const REGISTERS = [
  // ---- the sale registers: R2 all, R3 cash, R4 credit, R5 buyer bills ---------------------------
  cell(
    'R2',
    'figure',
    'period',
    'view=R2;period=this year',
    `total=${sales('r2.year')}`,
    [
      r('is saal ki poori sales register ka total kitna hai'),
      u('اس سال کی پوری سیلز رجسٹر کا ٹوٹل کتنا ہے'),
      e('what is the total of the full sales register this year'),
    ],
    { tools: 'report:R2' },
  ),
  cell(
    'R2',
    'list',
    'period',
    'view=R2;period=last month',
    `total=${sales('r2.last_month')}`,
    [
      rm('pichle mahine ki puri sales register dikhao'),
      u('پچھلے مہینے کی پوری سیلز رجسٹر دکھائیں'),
      e('show the full sales register of last month'),
    ],
    { tools: 'report:R2', n: 1 },
  ),
  cell(
    'R2',
    'day',
    'day',
    'view=R2;period=yesterday',
    `total=${sales('r2.yesterday')}`,
    [
      r('kal ki sari bikri ki lines ka total'),
      u('کل کی ساری بکری کی لائنوں کا ٹوٹل'),
      e('total of all the sale lines of yesterday'),
    ],
    { tools: 'report:R2', n: 1 },
  ),
  cell(
    'R2',
    'compare',
    'period',
    'view=R2',
    `year=${sales('r2.year')};month=${sales('r2.last_month')}`,
    [
      r('sales register ka total is saal aur pichle mahine ka alag alag'),
      u('سیلز رجسٹر کا ٹوٹل اس سال اور پچھلے مہینے کا الگ الگ'),
      e('sales register total for this year and for last month'),
    ],
    { tools: 'report:R2', n: 1 },
  ),
  cell(
    'R3',
    'figure',
    'period',
    'view=R3;period=this year',
    `total=${sales('r3.year')}`,
    [
      r('is saal ki naqdi bikri kitni hui'),
      u('اس سال کی نقدی بکری کتنی ہوئی'),
      em('cash sales this yaer'),
    ],
    { tools: 'report:R3' },
  ),
  cell(
    'R3',
    'day',
    'day',
    'view=R3;period=yesterday',
    `total=${sales('r3.yesterday')}`,
    [
      r('kal ki naqdi bikri kitni thi'),
      u('کل کی نقدی بکری کتنی تھی'),
      e('cash sales of yesterday'),
    ],
    { tools: 'report:R3', n: 1 },
  ),
  cell(
    'R3',
    'compare',
    'period',
    'view=R3',
    `cash=${sales('r3.year')};credit=${sales('r4.year')}`,
    [
      rm('naqdi aur udhari bikri mein is saal kitna farq hai'),
      u('نقدی اور ادھاری بکری میں اس سال کتنا فرق ہے'),
      e('how do cash sales compare with credit sales this year'),
    ],
    { tools: 'report:R3,report:R4', n: 1 },
  ),
  cell(
    'R4',
    'figure',
    'period',
    'view=R4;period=this year',
    `total=${sales('r4.year')}`,
    [
      r('is saal ki udhari bikri kitni hai'),
      u('اس سال کی ادھاری بکری کتنی ہے'),
      e('credit sales this year'),
    ],
    { tools: 'report:R4' },
  ),
  cell(
    'R4',
    'day',
    'day',
    'view=R4;period=yesterday',
    `total=${sales('r4.yesterday')}`,
    [
      r('kal udhar par kitna maal gaya'),
      u('کل ادھار پر کتنا مال گیا'),
      e('how much went on credit yesterday'),
    ],
    { tools: 'report:R4', n: 1 },
  ),
  cell(
    'R5',
    'figure',
    'period',
    'view=R5;period=this year',
    `total=${sales('r5.year')}`,
    [
      rm('udhari bills ka is saal ka total kitna hai'),
      u('ادھاری بلوں کا اس سال کا ٹوٹل کتنا ہے'),
      e('what is the total of the credit bills this year'),
    ],
    { tools: 'report:R5' },
  ),
  cell(
    'R5',
    'list',
    'buyer',
    'view=R5;buyer=Faisal;period=this year',
    'bill=@r5.faisal.year.total.buyer_total|@r5.faisal.year.total.value',
    [
      r('faisal ke udhari bills dikhao'),
      u('فیصل کے ادھاری بل دکھائیں'),
      e("show Faisal's credit bills"),
    ],
    { tools: 'report:R5', n: 1 },
  ),

  // ---- seller details (R6), item-wise sales (R7) (R8 was retired in plan 71) -----------------------------
  cell(
    'R6',
    'figure',
    'seller',
    'view=R6;period=this year',
    'gross=@r6.year.total.gross;net=@r6.year.total.net',
    [
      r('is saal bepariyon ko kitna sabit hua, khaam bikri aur safi bikri'),
      u('اس سال بیوپاریوں کی خام بکری اور صافی بکری کتنی رہی'),
      e('gross sale and net sale to the sellers this year'),
    ],
    { tools: 'report:R6' },
  ),
  cell(
    'R6',
    'day',
    'day',
    'view=R6;period=yesterday',
    'gross=@r6.yesterday.total.gross',
    [
      rm('kal bepariyon ki khaam bikri kitni thi'),
      u('کل بیوپاریوں کی خام بکری کتنی تھی'),
      e("the sellers' gross sale yesterday"),
    ],
    { tools: 'report:R6', n: 1 },
  ),
  cell(
    'R6',
    'list',
    'seller',
    'view=R6;seller=Rahim;period=this year',
    'gross=@r6.rahim.year.total.gross;net=@r6.rahim.year.total.net',
    [
      r('rahim ke maal ki is saal ki khaam aur safi bikri'),
      u('رحیم کے مال کی اس سال کی خام اور صافی بکری'),
      e("Rahim's gross and net sale this year"),
    ],
    { tools: 'report:R6', n: 1 },
  ),
  cell(
    'R7',
    'figure',
    'item',
    'view=R7;period=this year',
    'gross=@r7.year.total.gross;comm=@r7.year.total.total_comm',
    [
      r('har item ki is saal ki bikri ka total aur commission'),
      u('ہر آئٹم کی اس سال کی بکری کا ٹوٹل اور کمیشن'),
      e('item-wise sales total and commission this year'),
    ],
    { tools: 'report:R7' },
  ),

  // ---- accounts: day book (R11), ledgers (R13, R14), vouchers (R15), index (R16) ----------------
  cell(
    'R11',
    'figure',
    'day',
    'view=R11;period=yesterday',
    'debit=@r11.yesterday.total.debit',
    [
      r('kal ke roznamche ka total kitna hai'),
      u('کل کے روزنامچے کا ٹوٹل کتنا ہے'),
      e("what is the total of yesterday's day book"),
    ],
    { tools: 'report:R11' },
  ),
  cell(
    'R11',
    'list',
    'day',
    'view=R11;period=yesterday',
    'count=@r11.yesterday.count',
    [
      rm('kal roznamche mein kitni entries hain'),
      u('کل روزنامچے میں کتنی اندراجات ہیں'),
      e("how many entries are in yesterday's day book"),
    ],
    { tools: 'report:R11', n: 1 },
  ),
  cell(
    'R13',
    'figure',
    'buyer',
    'view=R13;buyer=Faisal',
    'balance=@r13.faisal.total.balance',
    [
      r('faisal ke khate ka aakhri balance kitna hai'),
      u('فیصل کے کھاتے کا آخری بیلنس کتنا ہے'),
      e("what is the closing balance of Faisal's ledger"),
    ],
    { tools: 'report:R13' },
  ),
  cell(
    'R13',
    'list',
    'buyer',
    'view=R13;buyer=Faisal',
    'debit=@r13.faisal.total.debit;credit=@r13.faisal.total.credit',
    [
      rm('faisal ke khate mein kitna naam hua aur kitna jama hua'),
      u('فیصل کے کھاتے میں کتنا بنام ہوا اور کتنا جمع'),
      e("how much was debited and credited in Faisal's ledger"),
    ],
    { tools: 'report:R13', n: 1 },
  ),
  cell(
    'R14',
    'figure',
    'buyer',
    'view=R14;buyer=Faisal',
    'bought=@r14.faisal.sum.report_statement_bought;paid=@r14.faisal.sum.report_statement_paid',
    [
      r('faisal ka fard hisab: kitna khareeda aur kitna diya'),
      u('فیصل کا فرد حساب: کتنا خریدا اور کتنا دیا'),
      e("Faisal's statement: how much he bought and how much he paid"),
    ],
    { tools: 'report:R14' },
  ),
  cell(
    'R15',
    'figure',
    'period',
    'view=R15;period=this year',
    'receipts=@r15.year.sum.report_doc_receipt;payments=@r15.year.sum.report_doc_payment',
    [
      r('is saal wasooli aur adaigi ke vouchers ka total'),
      u('اس سال وصولی اور ادائیگی کے واؤچرز کا ٹوٹل'),
      e('total of the receipt and payment vouchers this year'),
    ],
    { tools: 'report:R15' },
  ),
  cell(
    'R15',
    'list',
    'period',
    'view=R15;period=this year',
    'count=@r15.year.count',
    [
      rm('is saal kitne vouchers bane'),
      u('اس سال کتنے واؤچر بنے'),
      e('how many vouchers were made this year'),
    ],
    { tools: 'report:R15', n: 1 },
  ),
  cell(
    'R16',
    'figure',
    'accounts',
    'view=R16',
    'count=@r16.count',
    [
      r('index mein kitne khate hain'),
      u('انڈکس میں کتنے کھاتے ہیں'),
      e('how many accounts are in the index'),
    ],
    { tools: 'report:R16' },
  ),
  cell(
    'R17',
    'figure',
    'accounts',
    'view=R17',
    'total=@r17.total.debit',
    [
      rm('madon ke balance ka total kitna hai'),
      u('مدوں کے بیلنس کا ٹوٹل کتنا ہے'),
      e('what is the total of the head balances'),
    ],
    { tools: 'report:R17' },
  ),
  cell(
    'R18',
    'figure',
    'accounts',
    'view=R18',
    'total=@r18.total.debit;diff=@r18.sum.report_difference',
    [
      r('trial balance ka total kitna hai aur farq kitna hai'),
      u('ٹرائل بیلنس کا ٹوٹل کتنا ہے اور فرق کتنا ہے'),
      e('what is the trial balance total and the difference'),
    ],
    { tools: 'report:R18' },
  ),
  cell(
    'R19',
    'figure',
    'accounts',
    'view=R19',
    'total=@r19.total.debit',
    [
      r('opening trial balance ka total kitna hai'),
      u('اوپننگ ٹرائل بیلنس کا ٹوٹل کتنا ہے'),
      e('what is the total of the opening trial balance'),
    ],
    { tools: 'report:R19' },
  ),

  // ---- balance lists: dues by city (R21), by collector (R22) --------------------------------------
  cell(
    'R21',
    'figure',
    'dues',
    'view=R21',
    'total=@r21.total.balance',
    [
      r('shehar ke hisab se gahakon ka kul baqaya'),
      u('شہر کے حساب سے گاہکوں کا کل بقایا'),
      e('total dues from buyers by city'),
    ],
    { tools: 'report:R21' },
  ),
  cell(
    'R22',
    'figure',
    'dues',
    'view=R22',
    'total=@r22.total.balance',
    [
      rm('agrahi wale ke hisab se kul baqaya kitna hai'),
      u('اگراہی والے کے حساب سے کل بقایا کتنا ہے'),
      e('total dues by collector'),
    ],
    { tools: 'report:R22' },
  ),

  // ---- bills (R27), income (R28), the day summary (R34), the day sheet (R35) --------------------
  cell(
    'R27',
    'figure',
    'seller',
    'view=R27;period=this year',
    'gross=@r27.year.total.gross;net=@r27.year.total.net',
    [
      r('is saal ke sab bills ki khaam aur safi bikri'),
      u('اس سال کے سب بلوں کی خام اور صافی بکری'),
      e('gross and net sale of all the bills this year'),
    ],
    { tools: 'report:R27' },
  ),
  cell(
    'R27',
    'list',
    'seller',
    'view=R27;period=this year',
    'count=@r27.year.count',
    [
      rm('is saal kitne bill bane'),
      u('اس سال کتنے بل بنے'),
      e('how many bills were made this year'),
    ],
    { tools: 'report:R27', n: 1 },
  ),
  cell(
    'R28',
    'figure',
    'period',
    'view=R28;period=this year',
    'income=@r28.year.total.amount',
    [
      r('is saal ki kul aamdan kitni hai'),
      u('اس سال کی کل آمدن کتنی ہے'),
      e('what is the total income this year'),
    ],
    { tools: 'report:R28' },
  ),
  cell(
    'R34',
    'figure',
    'day',
    'view=R34;period=yesterday',
    'sales=@a.yesterday.total.gross;comm=@a.yesterday.total.total_comm',
    [
      r('kal ka din ka khulasa batao, bikri aur commission'),
      u('کل کے دن کا خلاصہ بتائیں، بکری اور کمیشن'),
      e("give me yesterday's day summary: sales and commission"),
    ],
    { tools: 'report:R34' },
  ),
  cell(
    'R35',
    'figure',
    'buyer',
    'view=R35;buyer=Faisal;period=today',
    'today=@r35.faisal.today.sum.report_r35_today_total;previous=@r35.faisal.today.sum.report_r35_previous',
    [
      r('faisal ki aaj ki parchi mein aaj ka bill aur pichla baqaya kitna hai'),
      u('فیصل کی آج کی پرچی میں آج کا بل اور پچھلا بقایا کتنا ہے'),
      e("what are Faisal's bill today and his previous balance on today's slip"),
    ],
    { tools: 'report:R35' },
  ),
  cell(
    'R35',
    'list',
    'buyer',
    'view=R35;buyer=Faisal;period=today',
    'goods=@r35.faisal.today.sum.report_r35_goods;qty=@r35.faisal.today.sum.report_r35_total_qty',
    [
      rm('faisal ne aaj kitne nag khareede aur kitne ka maal'),
      u('فیصل نے آج کتنے نگ خریدے اور کتنے کا مال'),
      e('how many nag did Faisal buy today and for how much'),
    ],
    { tools: 'report:R35', n: 1 },
  ),
];

export default REGISTERS;
