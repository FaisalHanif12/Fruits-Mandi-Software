// Facet CAP, part 4 (plan 55 Section 11): the "no such party or the wrong role" cell of every row that
// takes a named record. One question each, in a name nobody has; the languages rotate. The app
// must end as not_found with the role it was asked for (`asked_role`), call no model for it, and show
// no figure (PRD FR-7.77, plan 52 Section 8).
//
// Names nobody has in the books of the year: Sultan Mirza, Hamid Kareem, Irfan Qasim, Babar Awan,
// Zafar Iqbal, Rahat Bibi, Bilal Zaman, Gulabi (a garden), Narnia (a city and an adda), kiwi (an
// item), trip 99.

const NOSUCH = [
  // [row, asked role, [language, text, 'mis'?]]
  ['R6', 'seller', ['roman', 'Hamid Kareem ke maal ki bikri ka khulasa dikhao']],
  ['R8', 'any', ['urdu', 'نارنیا اڈا کی رپورٹ دکھائیں']],
  ['R13', 'any', ['roman', 'Sultan Mirza ka khata dikhao']],
  ['R14', 'any', ['urdu', 'سلطان مرزا کا فرد حساب دکھائیں']],
  ['R21', 'any', ['en', 'what do buyers from Narnia owe']],
  ['R24', 'seller', ['roman', 'Irfan Qasim ko kitna dena hai']],
  ['R26', 'seller', ['urdu', 'عرفان قاسم کی پیشگی کتنی ہے']],
  ['R31', 'partner', ['en', "what is partner Zafar Iqbal's total"]],
  ['R32', 'any', ['roman', 'Zafar Iqbal ne udhar raqam kitni li hai']],
  ['R33', 'trip', ['urdu', 'سفر 99 کا نفع کتنا رہا']],
  ['R35', 'any', ['en', "show Sultan Mirza's slip for today"]],
  ['R36', 'garden', ['roman', 'Gulabi baagh ka balance kitna hai']],
  ['view-A', 'item', ['urdu', 'کیوی کا اس سال کا کمیشن کتنا ہے']],
  ['view-B', 'seller', ['en', "how much of Hamid Kareem's goods sold this year"]],
  ['view-C', 'buyer', ['roman', 'Sultan Mirza ne is saal kitna maal khareeda']],
  ['view-F', 'partner', ['urdu', 'پارٹنر ظفر کا اس سال کا حصہ کتنا ہے']],
  ['view-I', 'garden', ['en', 'show the report of the Gulabi garden']],
  ['view-J', 'trip', ['roman', 'safar 99 ki report dikhao']],
  ['view-K', 'partner', ['urdu', 'پارٹنر ظفر نے پھل کتنا لیا']],
  ['record-chukta', 'any', ['en', 'who is Babar Awan in chukta khata']],
  ['record-money_lent', 'any', ['roman', 'Irfan Qasim ko udhar raqam kitni di thi']],
  ['record-separate_khata', 'any', ['urdu', 'راحت بی بی کے الگ کھاتے میں کتنا ہے']],
  ['record-partner', 'partner', ['roman', 'partner Zafar ka sarmaya kitna hai']],
  ['record-garden', 'garden', ['urdu', 'گلابی باغ کا ٹھیکہ کتنا ہے']],
  ['record-trip', 'trip', ['en', 'how much stock is left on trip 99']],
  ['record-item', 'item', ['roman', 'kiwi ka kitna maal baqi hai']],
  ['record-city', 'any', ['urdu', 'نارنیا شہر کے گاہکوں کا بقایا کتنا ہے']],
  ['record-adda', 'any', ['en', 'how many trucks came from the Narnia adda']],
  ['record-collector', 'any', ['roman', 'agrahi wale Bilal Zaman ki list dikhao']],
];

export default NOSUCH.map(([row, role, wording]) => ({
  g: `CN-${row}-nosuch`,
  cat: 'E',
  section: 'unknown',
  kind: 'nosuch',
  world: 'year',
  outcome: 'not_found',
  tools: 'find',
  filters: `facet=CAP;asked_role=${role}`,
  figures: '',
  source: `capability.tsv ${row} / nosuch`,
  q: [wording],
}));
