// Facet CAP, part 5 (plan 55 Section 11): "print or open" of the reports that plan 55 ruling 1 gives a
// word-to-report plan (the cash book, the expense report, dues, payables, the trial balance, the day
// book, a partner's statement, fruit taken) and that the print facet had no question for. One
// question each, the languages rotating. The button must open the right report with the exact dates
// (the dates come from the oracle's calendar, which the app's own period helper works out); a
// shortcut instead of dates is a miss. Only the reports with a party are checked for their steps.

const MONTH = 'print.from=@cal.month_start;print.to=@cal.today';
const LAST = 'print.from=@cal.last_month_start;print.to=@cal.last_month_end';

// [report, section, period, figures, [language, text]]
const PRINTS = [
  [
    'R12',
    'cash',
    'month',
    `print.report=R12;${MONTH};print.steps=1`,
    ['roman', 'is mahine ki rokar book ka print nikalna hai'],
  ],
  [
    'R12',
    'cash',
    'last_month',
    `print.report=R12;${LAST};print.steps=1`,
    ['urdu', 'پچھلے مہینے کی روکڑ بک کا پرنٹ'],
  ],
  [
    'R29',
    'expense',
    'last_month',
    `print.report=R29;${LAST};print.steps=1`,
    ['en', "print last month's expense report"],
  ],
  [
    'R29',
    'expense',
    'month',
    `print.report=R29;${MONTH};print.steps=1`,
    ['roman', 'is mahine ke kharchon ka print'],
  ],
  [
    'R20',
    'dues',
    'today',
    'print.report=R20;print.as_of=@cal.today;print.steps=1',
    ['urdu', 'گاہکوں کے بقایا کا پرنٹ نکالنا ہے'],
  ],
  [
    'R24',
    'dues',
    'today',
    'print.report=R24;print.as_of=@cal.today;print.steps=1',
    ['en', 'print the list of what we owe the sellers'],
  ],
  [
    'R18',
    'accounts',
    'today',
    'print.report=R18;print.as_of=@cal.today;print.steps=1',
    ['roman', 'trial balance ka print'],
  ],
  [
    'R11',
    'day',
    'yesterday',
    'print.report=R11;print.from=@cal.yesterday;print.to=@cal.yesterday;print.steps=1',
    ['urdu', 'کل کے روزنامچے کا پرنٹ'],
  ],
  [
    'R11',
    'day',
    'today',
    'print.report=R11;print.from=@cal.today;print.to=@cal.today;print.steps=1',
    ['en', "print today's day book"],
  ],
  [
    'R37',
    'fruit',
    'month',
    `print.report=R37;${MONTH};print.steps=1`,
    ['roman', 'is mahine ke phal register ka print'],
  ],
  [
    'R31',
    'partner',
    'month',
    `print.report=R31;print.partner=Partner A;${MONTH}`,
    ['en', "print Partner A's statement for this month"],
  ],
  [
    'R31',
    'partner',
    'last_month',
    `print.report=R31;print.partner=Partner B;${LAST}`,
    ['roman', 'partner b ka pichle mahine ka statement print karna hai'],
  ],
];

export default PRINTS.map(([report, section, period, figures, wording], i) => ({
  g: `CPR-${report}-${period}-${i}`,
  cat: 'A',
  section: 'print',
  kind: 'print',
  world: 'year',
  outcome: 'answered',
  tools: '',
  filters: `facet=CAP;pkind=report;pperiod=${period};section=${section}`,
  figures,
  source: `capability.tsv ${report} / print`,
  q: [wording],
}));
