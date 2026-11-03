#!/usr/bin/env node
// The live check of "ask the app" (D6 Section 3, plan 50 R2, plan 52 items 12 to 14, NFR-AI-2).
//
// It asks the questions of scripts/ai-eval/questions.tsv through the app's own question path
// (run::ask, the function behind the assistant_ask command) on seeded databases, with the real
// model, then scores each answer against figures that the APP ITSELF worked out (the oracle of the
// rich year: backend/crates/assistant/tests/robustness_year/oracle.rs), and prints the score and
// the time of each of the five categories of plan 52 Section 2, every miss, the tool calls, the
// model calls, the tokens, and writes scripts/ai-eval/urdu-review.tsv for the Urdu reviewer.
//
// ONLY THE LEAD RUNS IT, with Umar's key. Never from a test, never from CI.
//
//   THE KEY. The runner never sees it. Umar pastes it in Settings (question group) and the app keeps
//   it in Windows Credential Manager (the macOS keychain in development). The Rust side
//   (backend/crates/assistant/tests/robustness_live.rs, ignored by default) reads it from there
//   through the app's own key store, just before each call. The key is not an argument, not an
//   environment variable, not a file. On a Mac the keychain may ask for the Mac password: choose
//   Always Allow. Without a saved key the run stops with "WAITING FOR KEY".
//
// Usage:
//   node scripts/ai-eval/run.mjs                      the full run (the whole bank)
//   node scripts/ai-eval/run.mjs --category A         only the questions of category A (A,B also)
//   node scripts/ai-eval/run.mjs --world year         only the questions of one world
//   node scripts/ai-eval/run.mjs --only Q001,Q016     only these ids
//   node scripts/ai-eval/run.mjs --threads 4          questions at once (default 1: honest times)
//   node scripts/ai-eval/run.mjs --check              check the bank, no model, no cargo
//   node scripts/ai-eval/run.mjs --self-test          check the scorer on made-up replies, no model
//   node scripts/ai-eval/run.mjs --results FILE       score a results file again, no model
//   --model NAME        another model name (default: the app's default)
//   --review-out FILE   write the Urdu review sheet here instead of scripts/ai-eval/urdu-review.tsv
//   --detail-out FILE   write the detail of every miss here (default backend/target/ai-eval/misses.txt)
//
// The bank is built from questions-110.tsv (the frozen first 110) and bank-spec.mjs by
// build-bank.mjs; `--check` fails if questions.tsv is not what they build.
//
// Exit codes: 0 every category that ran meets plan 52 Section 2; 1 one does not; 2 the bank or the
// run is broken; 3 waiting for the key.
//
// The project folder has spaces in its name. Every path here is built with node:path and every
// process is started without a shell, so nothing is split.

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { buildText } from './build-bank.mjs';

const args = process.argv.slice(2);
const flag = (name) => args.includes(name);
const option = (name) => {
  const at = args.indexOf(name);
  return at >= 0 ? args[at + 1] : undefined;
};

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, '..', '..');
const BACKEND = join(ROOT, 'backend');
const QUESTIONS = join(HERE, 'questions.tsv');
const FROZEN = join(HERE, 'questions-110.tsv');
const MANIFEST = join(HERE, 'oracle-keys.txt');
const REVIEW = option('--review-out')
  ? resolve(option('--review-out'))
  : join(HERE, 'urdu-review.tsv');
const OUT_DIR = join(BACKEND, 'target', 'ai-eval');
const RESULTS = join(OUT_DIR, 'results.json');
const DETAIL = option('--detail-out') ? resolve(option('--detail-out')) : join(OUT_DIR, 'misses.txt');
const SUMMARY = join(OUT_DIR, 'summary.json');

// D6-A19, NFR v1.13: the fixed part is at most 11,000 tokens counted with o200k. The model counts
// about 3.4% more, so as the model counts it the limit is about 11,400.
const PREFIX_O200K = 11000;
const MODEL_COUNTS_MORE = 1.034;
const PREFIX_MODEL = 11400;
const WORLDS = new Set(['day', 'late', 'e59', 'e42', 'e34', 'year', 'thin']);
const OUTCOMES = new Set([
  'answered',
  'estimate',
  'advice',
  'which_one',
  'need_dates',
  'need_party',
  'not_found',
  'outside_books',
  'change_refused',
  'not_understood',
  'checks_failed',
  'records_differ',
  'safe',
]);
// A safe end: the app asked, refused or said it could not, and showed no wrong figure.
const SAFE_ENDS = new Set([
  'which_one',
  'need_dates',
  'need_party',
  'not_found',
  'not_understood',
  'checks_failed',
  'records_differ',
]);
// The kinds of failure that are the network's, not the model's (plan 52 item 6).
const NETWORK_KINDS = new Set(['offline', 'dropped', 'timeout', 'busy', 'server']);

/** The five categories of plan 52 Section 2: their target times and what "meets" means. */
const CATS = {
  A: { title: 'real-time data', p50: 6000, p95: 12000, rule: 'no wrong figure, at least 98% answered' },
  B: { title: 'suggestions', p50: 6000, p95: 12000, rule: 'at least 95% good' },
  C: { title: 'forecasts and what-ifs', p50: 15000, p95: 25000, rule: 'no wrong figure, at least 98% answered' },
  D: { title: 'help with the app', p50: 5000, p95: 10000, rule: 'at least 98% the right topic' },
  E: { title: 'mixed and hard', p50: 6000, p95: 12000, rule: 'safe ends only, never a wrong figure' },
};

const HELP_TOPICS = [
  'bikri_book', 'today_sales', 'unsold_stock', 'later_sale', 'receipt', 'collection_list', 'payment',
  'advance', 'pukhta_khata', 'edit_and_cancel', 'cash_book', 'shop_expense', 'personal_expense',
  'money_lent', 'separate_khata', 'general_journal', 'fruit_taken', 'gardens', 'market_trips',
  'partners', 'partner_capital', 'analytics', 'reports', 'dashboard', 'day_close', 'month_close',
  'backup', 'settings', 'masters', 'accounts', 'search', 'commission_rules', 'dead_amount',
  'language_switch', 'keys', 'printing', 'ask_the_app', 'sign_in',
];

// ---- the list ------------------------------------------------------------------------------

/** Reads a bank file into objects. */
function loadList(file = QUESTIONS) {
  const lines = readFileSync(file, 'utf8')
    .split('\n')
    .filter((l) => l.trim() !== '');
  const head = lines[0].split('\t');
  return lines.slice(1).map((line) => {
    const cells = line.split('\t');
    const row = Object.fromEntries(head.map((h, i) => [h, cells[i] ?? '']));
    row.misspelled = row.misspelled === '1';
    return row;
  });
}

/** The mix of D6 Section 3 for the frozen first 110 questions (kept as a subset). */
const KINDS_OF_110 = {
  party: 15, sales: 9, analytics: 11, dues: 9, cash: 7, partners: 6, stock: 8, sums: 7,
  estimates: 10, advice: 8, followup: 5, trap: 5, help: 10,
};

function checkFrozen110(list) {
  const problems = [];
  const count = (key, value) => list.filter((q) => q[key] === value).length;
  if (list.length !== 110) problems.push(`the frozen subset has ${list.length} questions, not 110`);
  for (const [kind, want] of Object.entries(KINDS_OF_110)) {
    if (count('kind', kind) !== want)
      problems.push(`frozen kind ${kind}: ${count('kind', kind)}, want ${want}`);
  }
  const roman = count('lang', 'roman');
  const urdu = count('lang', 'urdu');
  const en = count('lang', 'en');
  if (roman < 42 || roman > 46) problems.push(`frozen Roman Urdu ${roman} (about 44)`);
  if (urdu < 36 || urdu > 40) problems.push(`frozen Urdu letters ${urdu} (about 38)`);
  if (en < 26 || en > 30) problems.push(`frozen English ${en} (about 28)`);
  return problems;
}

function manifestKeys() {
  if (!existsSync(MANIFEST)) return null;
  return new Set(
    readFileSync(MANIFEST, 'utf8')
      .split('\n')
      .filter((l) => l !== ''),
  );
}

/** The `@keys` a figures text names. */
function refsOf(figures) {
  return [...figures.matchAll(/@([A-Za-z0-9_.]+)/g)].map((m) => m[1]);
}

/** Checks the whole bank. Returns problems and a summary. */
function checkList(list) {
  const problems = [];
  const count = (pred) => list.filter(pred).length;
  const n = list.length;
  const byCat = Object.fromEntries(Object.keys(CATS).map((c) => [c, count((q) => q.cat === c)]));
  const languages = {
    roman: count((q) => q.lang === 'roman'),
    urdu: count((q) => q.lang === 'urdu'),
    en: count((q) => q.lang === 'en'),
  };
  const misspelled = count((q) => q.misspelled);
  const groups = new Map();
  for (const q of list) if (q.group) groups.set(q.group, (groups.get(q.group) ?? 0) + 1);
  const bigGroups = [...groups.values()].filter((k) => k >= 3).length;
  const ids = new Set();
  for (const q of list) {
    if (ids.has(q.id)) problems.push(`${q.id} is used twice`);
    ids.add(q.id);
  }
  for (const q of list) {
    if (q.after && !ids.has(q.after)) problems.push(`${q.id} follows ${q.after}, which is not in the list`);
    if (!q.question || !q.outcome) problems.push(`${q.id} has no question or outcome`);
    if (!OUTCOMES.has(q.outcome)) problems.push(`${q.id} has the unknown outcome ${q.outcome}`);
    if (!WORLDS.has(q.world)) problems.push(`${q.id} has the unknown world ${q.world}`);
    if (!CATS[q.cat]) problems.push(`${q.id} has no category`);
    if (!q.section) problems.push(`${q.id} has no section`);
    if (/@/.test(q.question)) problems.push(`${q.id}: a question may not hold a reference`);
  }
  // The frozen first 110 keep the mix of D6 Section 3.
  problems.push(...checkFrozen110(list.slice(0, 110)));
  // Size and mix of the whole bank.
  if (n < 300) problems.push(`only ${n} questions (at least 300)`);
  const MIN_CAT = { A: 150, B: 40, C: 35, D: 45, E: 40 };
  for (const [c, min] of Object.entries(MIN_CAT))
    if (byCat[c] < min) problems.push(`category ${c}: ${byCat[c]} questions (at least ${min})`);
  const share = (k) => (100 * k) / Math.max(1, n);
  if (share(languages.roman) < 38 || share(languages.roman) > 50)
    problems.push(`Roman Urdu is ${share(languages.roman).toFixed(0)}% (38 to 50)`);
  if (share(languages.urdu) < 25 || share(languages.urdu) > 40)
    problems.push(`Urdu letters are ${share(languages.urdu).toFixed(0)}% (25 to 40)`);
  if (share(languages.en) < 20 || share(languages.en) > 32)
    problems.push(`English is ${share(languages.en).toFixed(0)}% (20 to 32)`);
  if (misspelled < 40) problems.push(`only ${misspelled} misspelled (at least 40)`);
  if (bigGroups < 35) problems.push(`only ${bigGroups} groups of 3 or more wordings (at least 35)`);
  // Category A covers every section of the books.
  const sectionCount = (s) => count((q) => q.cat === 'A' && q.section === s);
  const NEED3 = ['item', 'seller', 'buyer', 'partner', 'period', 'cash', 'dues', 'stock', 'day', 'month', 'expense'];
  const NEED2 = ['garden', 'trip', 'fruit', 'capital'];
  for (const s of NEED3) if (sectionCount(s) < 3) problems.push(`category A: section ${s} has ${sectionCount(s)} (at least 3)`);
  for (const s of NEED2) if (sectionCount(s) < 2) problems.push(`category A: section ${s} has ${sectionCount(s)} (at least 2)`);
  // Every item, and the names of Umar's own examples, are in the bank.
  const allFigures = list.map((q) => q.figures).join(';');
  for (const item of ['banana', 'apple', 'mosambi', 'mango', 'grapes', 'potato'])
    if (!allFigures.includes(`.${item}.`)) problems.push(`no question reads the item ${item}`);
  for (const text of [
    'kela ke is saal kitne commission or sale huwe',
    'Faisal ne kitne ka saman khareeda',
    'sales bherane ke tareeky batao',
  ])
    if (!list.some((q) => q.question.includes(text))) problems.push(`Umar's own question is missing: ${text}`);
  if (!list.some((q) => /akram/i.test(q.question) && q.outcome === 'which_one'))
    problems.push('no question about the two Akrams ends in which_one');
  // Category D covers every topic of the guide.
  const topics = new Set(list.map((q) => (q.filters.match(/topic=([a-z_]+)/) ?? [])[1]).filter(Boolean));
  for (const t of HELP_TOPICS) if (!topics.has(t)) problems.push(`no help question for the topic ${t}`);
  // General advice is tested where the books are thin.
  if (count((q) => q.filters.includes('general=1')) < 8) problems.push('fewer than 8 general advice questions');
  // Every `@key` is a fact the oracle gives.
  const keys = manifestKeys();
  if (!keys) problems.push('scripts/ai-eval/oracle-keys.txt is missing');
  else
    for (const q of list)
      for (const ref of refsOf(q.figures))
        if (!keys.has(ref)) problems.push(`${q.id}: the oracle has no fact @${ref}`);
  // The file is what the spec builds.
  try {
    if (readFileSync(QUESTIONS, 'utf8') !== buildText())
      problems.push('questions.tsv is not what bank-spec.mjs builds: run node scripts/ai-eval/build-bank.mjs');
  } catch (error) {
    problems.push(`the spec does not build: ${error.message}`);
  }
  return {
    problems,
    summary: { total: n, byCat, languages, misspelled, groups: groups.size, bigGroups },
  };
}

// ---- the oracle ----------------------------------------------------------------------------

/** Puts the oracle's facts in place of the `@keys` of a figures text. Throws for a missing fact. */
function resolveFigures(figures, oracle) {
  return figures.replace(/@([A-Za-z0-9_.]+)/g, (_, key) => {
    const value = oracle?.[key];
    if (value === undefined) throw new Error(`oracle fact @${key} is missing`);
    return value;
  });
}

// ---- reading an answer ---------------------------------------------------------------------

/** The numbers a cell holds, in whole rupees (money, balance), counts and hundredths of a percent. */
function cellValues(cell) {
  if (!cell || typeof cell !== 'object') return [];
  switch (cell.type) {
    case 'money':
      return [{ n: cell.paisa / 100, kind: 'money' }];
    case 'balance':
      return [{ n: cell.paisa / 100, kind: 'balance', side: cell.side }];
    case 'number':
      return [{ n: cell.value, kind: 'number' }];
    case 'percent':
      return [{ n: cell.hundredths, kind: 'percent' }];
    case 'weight':
      return [{ n: cell.grams, kind: 'weight' }];
    default:
      return [];
  }
}

/** Every number, month and name the reply shows: its figure parts, its tables and its estimate. */
function collect(reply) {
  const out = { parts: [], all: [], months: new Set(), names: new Set(), order: [], text: '' };
  const answer = reply?.answer;
  if (!answer) return out;
  for (const part of answer.parts ?? []) {
    if (part.type === 'figure') {
      const values = cellValues(part.cell);
      out.parts.push(...values);
      out.all.push(...values);
    } else if (part.type === 'name') {
      out.names.add(part.name.name_en.toLowerCase());
      out.order.push(part.name.name_en.toLowerCase());
    } else if (part.type === 'month') {
      out.months.add(`${part.year}-${String(part.month).padStart(2, '0')}`);
    } else if (part.type === 'text') {
      out.text += part.text;
    }
  }
  for (const table of answer.tables ?? []) {
    const cells = [...(table.totals ?? [])];
    for (const row of table.rows ?? []) cells.push(...(row.row?.cells ?? []));
    for (const cell of cells) {
      out.all.push(...cellValues(cell));
      if (cell?.type === 'names') out.names.add(cell.names.en.toLowerCase());
      if (cell?.type === 'party') out.names.add(cell.party.name_en.toLowerCase());
    }
    // The first names of the rows, in order, for "whom first".
    for (const row of table.rows ?? []) {
      const first = (row.row?.cells ?? []).find((c) => c?.type === 'party' || c?.type === 'names');
      if (first) out.order.push((first.party?.name_en ?? first.names?.en ?? '').toLowerCase());
    }
  }
  const monthOf = (m) => (m ? `${m.year}-${String(m.month).padStart(2, '0')}` : null);
  for (const method of answer.estimate?.methods ?? []) {
    out.all.push(...cellValues(method.total), ...cellValues(method.per_month), ...cellValues(method.step));
    const target = monthOf(method.target_month);
    if (target) out.months.add(`${method.kind}:${target}`);
  }
  for (const chart of answer.charts ?? []) {
    for (const series of chart.series ?? []) for (const cell of series.values ?? []) out.all.push(...cellValues(cell));
  }
  return out;
}

/** The words that name a side of the commission in the model's own text (`#seller` after a value). */
const SIDE_WORDS = { seller: ['بیوپاری', 'seller', 'byopari', 'biyopari'] };

/**
 * One value of a figures text: a number (with a side), a percent, a month, a date or a name. A
 * value ending in `#seller` counts only when the answer's text names the side (a bare
 * "commission" is the total, as the prompt says).
 */
function parseValue(label, value, strict) {
  const tag = (value.match(/#(seller)$/) ?? [])[1];
  if (tag) return { ...parseValue(label, value.slice(0, -tag.length - 1), strict), needs: tag };
  let m;
  if (strict) return { kind: 'name', name: value.toLowerCase(), strict: true };
  if ((m = value.match(/^(-?\d+(?:\.\d+)?)%$/))) return { kind: 'percent', n: Math.round(Number(m[1]) * 100) };
  if (/^\d{4}-\d{2}$/.test(label === 'date' ? '' : value)) return { kind: 'month', month: value };
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return { kind: 'date', date: value };
  if ((m = value.match(/^(-?\d+)(?:\s+(Dr|Cr|up|down))?/)))
    return { kind: 'amount', n: Number(m[1]), side: m[2]?.toLowerCase(), text: value };
  // "Arshad Multani 18684 Dr 36 days": a name with its amount (advice).
  if ((m = value.match(/^([A-Za-z ]+?)\s+(-?\d+)(?:\s+(Dr|Cr))?/)))
    return { kind: 'amount', n: Number(m[2]), side: m[3]?.toLowerCase(), text: value, name: m[1].toLowerCase() };
  return { kind: 'name', name: value.toLowerCase() };
}

/**
 * Splits `a=1;b=2 Dr` into expected figures. A label ending in `?` is optional, in `!` a name that
 * must be in the answer. `x|y` after the `=` lists values of which one is enough (two honest
 * readings of the question, each a true figure of the app).
 */
function expectedFigures(text) {
  return text
    .split(';')
    .filter(Boolean)
    .map((piece) => {
      const at = piece.indexOf('=');
      let label = piece.slice(0, at).trim();
      const value = piece.slice(at + 1).trim();
      let optional = false;
      let strict = false;
      if (label.endsWith('?')) {
        optional = true;
        label = label.slice(0, -1);
      }
      if (label.endsWith('!')) {
        strict = true;
        label = label.slice(0, -1);
      }
      const alts = value.split('|').map((v) => parseValue(label, v.trim(), strict));
      return { label, optional, strict, alts };
    });
}

/** True when the reply shows this number (a balance may show a negative as the credit side). */
function hasNumber(found, want) {
  return found.all.some((v) => {
    if (want.kind === 'percent') return v.kind === 'percent' && v.n === want.n;
    if (v.kind === 'percent') return false;
    const same = v.n === want.n || (want.n < 0 && v.kind === 'balance' && v.side === 'cr' && v.n === -want.n);
    const side = !want.side || want.side === 'up' || want.side === 'down' || v.side === want.side || v.n === 0;
    return same && side;
  });
}

// ---- scoring one question ------------------------------------------------------------------

function toolSeen(log, token) {
  const [tool, detail] = token.split(':');
  return (log ?? []).some((c) => c.tool === tool && (!detail || c.input?.id === detail || c.input?.view === detail));
}

/** True when every group of the tools note was used (another route is a note, not a miss). */
function routeMatched(question, result) {
  return question.tools
    .split(',')
    .filter(Boolean)
    .every((group) => group.split('/').some((t) => toolSeen(result.tool_calls, t)));
}

/** Does the answer meet one alternative of an expected figure? */
function altMet(alt, label, found, reply) {
  if (alt.needs) {
    const text = found.text.toLowerCase();
    if (!SIDE_WORDS[alt.needs].some((w) => text.includes(w))) return false;
  }
  if (alt.kind === 'name') {
    if (found.names.has(alt.name)) return true;
    // A name that need not be in the answer text counts when the record was read (a note only).
    return !alt.strict && JSON.stringify(reply?.answer?.sources ?? []).toLowerCase().includes(alt.name);
  }
  if (alt.kind === 'month') {
    // A target month belongs to its method: `pace_month` is the same-pace method's month.
    const word = (label.match(/^(pace|growth|season)_month$/) ?? [])[1];
    return [...found.months].some((m) => (word ? m.startsWith(`same_${word}:`) : true) && m.endsWith(alt.month));
  }
  if (alt.kind === 'date') return true; // a date is shown as a date part; the text is not checked further
  return hasNumber(found, alt);
}

/**
 * The verdict of one question: pass or a miss with its reasons, and notes that are not misses.
 * A wrong figure is always a miss: an expected figure that is not shown fails the question.
 */
function score(question, result, oracle) {
  const reply = result.reply;
  const outcome = reply?.outcome ?? 'none';
  const found = collect(reply);
  const reasons = [];
  const notes = [];
  const figureMisses = [];
  let wanted = [];
  try {
    wanted = expectedFigures(resolveFigures(question.figures, oracle));
  } catch (error) {
    return { id: question.id, cat: question.cat, pass: false, kind: 'MISS (oracle)', reasons: [error.message], notes, outcome, figureMisses };
  }
  const expectedOutcome = question.outcome;
  if (expectedOutcome === 'safe') {
    const ok = outcome === 'answered' || outcome === 'outside_books' || outcome === 'change_refused' || SAFE_ENDS.has(outcome);
    if (!ok) reasons.push(`outcome ${outcome}`);
    if (found.all.some((v) => v.n === 99999)) reasons.push('shows 99,999');
  } else if (outcome !== expectedOutcome) {
    // PRD FR-7.88, "unasked, one point" (Umar): a figure question may end as advice when the advice
    // part is one point at most (one bullet). The figures are scored below as always, so a wrong
    // or missing figure is still a miss.
    const onePoint = (found.text.match(/•/g) ?? []).length <= 1;
    const unasked = expectedOutcome === 'answered' && outcome === 'advice' && reply?.answer?.general_advice !== true && onePoint;
    if (unasked) notes.push('advice: one unasked point (FR-7.88)');
    else reasons.push(`outcome ${outcome}, wanted ${expectedOutcome}`);
  }
  const matched = routeMatched(question, result);
  const nothingShown = found.all.length === 0 && found.parts.length === 0;
  for (const want of wanted) {
    if (want.alts.some((alt) => altMet(alt, want.label, found, reply))) continue;
    // A zero said in words: the expected value is 0, the route matched and no other figure shows.
    const zero = want.alts.some((alt) => alt.kind === 'amount' && alt.n === 0 && !alt.side);
    if (zero && matched && nothingShown && outcome === expectedOutcome) {
      notes.push(`${want.label}=0 said in words`);
      continue;
    }
    if (want.optional) continue;
    const first = want.alts[0];
    if (first.kind === 'name' && !first.strict) {
      notes.push(`name ${first.name} not seen`);
    } else if (first.kind === 'name') {
      reasons.push(`name ${want.alts.map((a) => a.name).join(' or ')} (${want.label}) missing`);
      figureMisses.push(want.label);
    } else if (first.kind === 'month') {
      reasons.push(`month ${want.alts.map((a) => a.month).join(' or ')} (${want.label}) missing`);
      figureMisses.push(want.label);
    } else {
      reasons.push(`figure ${want.label}=${want.alts.map((a) => a.text ?? a.n).join(' or ')} missing`);
      figureMisses.push(want.label);
    }
  }
  // "Whom first": the first expected name comes before the second.
  const figuresText = (() => {
    try {
      return resolveFigures(question.figures, oracle);
    } catch {
      return '';
    }
  })();
  const first = figuresText.match(/first=([A-Za-z ]+?)\s+\d/);
  const second = figuresText.match(/second=([A-Za-z ]+?)\s+\d/);
  if (first && second) {
    const a = found.order.indexOf(first[1].toLowerCase());
    const b = found.order.indexOf(second[1].toLowerCase());
    if (a < 0 || (b >= 0 && a > b)) {
      reasons.push(`order: ${first[1]} should come before ${second[1]}`);
      figureMisses.push('order');
    }
  }
  // A help question must ask the guide for its topic (plan 51 B3).
  const topicWant = question.filters.match(/topic=([a-z_]+)/);
  if (topicWant && !(result.tool_calls ?? []).some((c) => c.tool === 'app_help' && c.input?.topic === topicWant[1]))
    reasons.push(`app_help topic ${topicWant[1]} not asked`);
  // A change request ends in a refusal that points to the right entry screen.
  const route = question.filters.match(/route=([a-z_]+)/);
  if (route && reply?.entry_route?.screen !== route[1]) reasons.push(`entry route is not ${route[1]}`);
  if (question.outcome === 'change_refused' && question.id === 'Q096' && reply?.entry_route?.screen !== 'wasooli')
    reasons.push('entry route is not wasooli');
  if (question.outcome === 'outside_books' && question.filters.includes('nearest=party_card')) {
    if (!JSON.stringify(reply?.answer?.sources ?? []).includes('party_card')) reasons.push('no party card button');
  }
  // General advice (plan 52 item 11): marked, and carrying no figure of its own. With data the
  // answer stands on the books (FR-7.88): a source, and not marked general.
  const general = reply?.answer?.general_advice === true;
  if (general) {
    if (outcome !== 'advice') reasons.push(`general advice with outcome ${outcome}`);
    if (!nothingShown || (reply.answer.tables ?? []).length > 0 || reply.answer.estimate) {
      reasons.push('general advice carries a figure, a table or an estimate');
      figureMisses.push('general-figure');
    }
    if (/[0-9۰-۹]/.test(found.text)) notes.push('general advice text has digits');
  }
  if (question.filters.includes('general=1') && !general) reasons.push('not marked as general advice');
  if (question.filters.includes('general=0')) {
    if (general) reasons.push('marked general advice although the books have the data');
    else if (outcome === 'advice' && (reply?.answer?.sources ?? []).length === 0) reasons.push('advice with no source in the books');
  }
  if (question.filters.includes('onecall=1') && (result.model_calls ?? 0) > 1) notes.push(`${result.model_calls} model calls (one wanted)`);
  // Tools are a note: another route to the same figure is not a miss.
  if (!matched) {
    for (const group of question.tools.split(',').filter(Boolean))
      if (!group.split('/').some((t) => toolSeen(result.tool_calls, t))) notes.push(`tool ${group} not used`);
  }
  if (result.clicked) notes.push(`which_one clicked: ${result.clicked.name}`);
  const afterReading = outcome === 'not_understood' && (result.tool_calls ?? []).some((c) => !c.error);
  const failure = reply?.failure?.kind ?? null;
  const network = !!reply && (outcome === 'offline' || outcome === 'error' || NETWORK_KINDS.has(failure ?? ''));
  const shows = found.parts.length + found.all.length > 0;
  let kind;
  if (reasons.length === 0) kind = 'PASS';
  else if (!reply) kind = 'MISS (harness)';
  else if (network) kind = 'MISS (network)';
  else if (SAFE_ENDS.has(outcome)) kind = 'MISS (safe end)';
  else if (shows || outcome === 'answered') kind = 'MISS (REVIEW: figure, party, period or source)';
  else kind = 'MISS';
  return { id: question.id, cat: question.cat, pass: reasons.length === 0, kind, reasons, notes, outcome, figureMisses, afterReading, network, failure };
}

/**
 * Questions of one group must reach the same figures (NFR-AI-2): compared on the REQUIRED figures
 * (an optional figure and the extra figures one wording happens to show do not count).
 */
function groupAgreement(list, verdicts, results) {
  const byGroup = new Map();
  for (const q of list) if (q.group) byGroup.set(q.group, [...(byGroup.get(q.group) ?? []), q]);
  const bad = [];
  for (const [group, members] of byGroup) {
    // A group is compared only when the run has all its questions.
    if (members.some((q) => !results.has(q.id) || !verdicts.has(q.id))) continue;
    const signs = members.map((q) => {
      const v = verdicts.get(q.id);
      if (q.kind === 'help') {
        // A help group agrees on the topic it asked for.
        const topics = (results.get(q.id)?.tool_calls ?? []).filter((c) => c.tool === 'app_help').map((c) => c.input?.topic);
        return [...new Set(topics)].sort().join(',');
      }
      if (v.network) return 'network'; // a lost connection says nothing about the figures
      return `${v.pass ? 'ok' : 'miss'}:${[...new Set(v.figureMisses)].sort().join(',')}`;
    });
    const real = signs.filter((s) => s !== 'network');
    if (new Set(real).size > 1) bad.push({ group, members: members.map((q) => q.id), signs });
  }
  return bad;
}

// ---- the review sheet ----------------------------------------------------------------------

const MONTHS_UR = ['جنوری', 'فروری', 'مارچ', 'اپریل', 'مئی', 'جون', 'جولائی', 'اگست', 'ستمبر', 'اکتوبر', 'نومبر', 'دسمبر'];
const group3 = (n) => Math.abs(n).toLocaleString('en-US');
const DIR_UR = { up: 'زیادہ', down: 'کم', same: 'برابر' };

/** One figure as the panel shows it. */
function showFigure(part) {
  const cell = part.cell;
  let text = '';
  if (cell.type === 'money') text = `${cell.paisa < 0 ? '-' : ''}${group3(cell.paisa / 100)}`;
  else if (cell.type === 'balance')
    text = `${group3(cell.paisa / 100)} ${cell.side === 'dr' ? 'بنام' : cell.side === 'cr' ? 'جمع' : ''}`.trim();
  else if (cell.type === 'number') text = group3(cell.value);
  else if (cell.type === 'percent') text = `${(cell.hundredths / 100).toFixed(2)}%`;
  else text = JSON.stringify(cell);
  if (part.dir) text += ` ${DIR_UR[part.dir] ?? ''}`;
  return part.checked ? `${text} ✓` : text;
}

/** The answer as shown: the model's words with the app's figures, names and dates filled in. */
function answerAsShown(reply) {
  const answer = reply?.answer;
  if (!answer) return `[${reply?.outcome ?? 'no reply'}]`;
  const pieces = answer.parts.map((p) => {
    if (p.type === 'text') return p.text;
    if (p.type === 'figure') return showFigure(p);
    if (p.type === 'name') return `${p.name.name_ur}${p.name.tag ? ` ${p.name.tag}` : ''}`;
    if (p.type === 'date') return p.date ? p.date.split('-').reverse().join('-') : '';
    if (p.type === 'month') return `${MONTHS_UR[p.month - 1] ?? p.month} ${p.year}`;
    return '';
  });
  const extra = [];
  if (answer.tables?.length) extra.push(`[جدول: ${answer.tables.map((t) => t.rows.length).join('، ')} قطاریں]`);
  if (answer.estimate) extra.push('[اندازہ، یقینی نہیں]');
  if (answer.general_advice) extra.push('[عام مشورہ]');
  else if (answer.advice) extra.push('[مشورہ، فیصلہ آپ کا]');
  if (answer.next?.length) extra.push(`[اگلے سوال: ${answer.next.join(' | ')}]`);
  return [pieces.join('') || `[${reply.outcome}]`, ...extra].join(' ').replace(/\s+/g, ' ').trim();
}

function writeReview(list, results) {
  const head = ['id', 'question', 'outcome', 'answer_as_shown', 'good', 'note', 'advice_inside_fr_7_88', 'cat'];
  const clean = (s) => String(s).replace(/[\t\r\n]+/g, ' ');
  const rows = list.map((q) => {
    const r = results.get(q.id);
    return [
      q.id,
      q.question,
      r?.reply?.outcome ?? 'not run',
      answerAsShown(r?.reply),
      '',
      '',
      q.cat === 'B' ? '' : 'n/a',
      q.cat,
    ].map(clean);
  });
  writeFileSync(REVIEW, [head, ...rows].map((r) => r.join('\t')).join('\n') + '\n', 'utf8');
}

// ---- the report ----------------------------------------------------------------------------

const sum = (xs) => xs.reduce((a, b) => a + b, 0);
function percentile(xs, p) {
  if (xs.length === 0) return 0;
  const sorted = [...xs].sort((a, b) => a - b);
  return sorted[Math.max(1, Math.ceil(p * sorted.length)) - 1];
}
const ms = (n) => (n >= 1000 ? `${(n / 1000).toFixed(1)} s` : `${Math.round(n)} ms`);

/** The numbers of one category: counts, rates and times (model time apart from network time). */
function categoryStats(cat, list, verdicts, results) {
  const qs = list.filter((q) => q.cat === cat && verdicts.has(q.id));
  const vs = qs.map((q) => verdicts.get(q.id));
  const n = qs.length;
  const passed = vs.filter((v) => v.pass).length;
  const wrong = vs.filter((v) => v.kind.includes('REVIEW')).length;
  const misses = vs.filter((v) => !v.pass);
  const unsafe = misses.filter((v) => !SAFE_ENDS.has(v.outcome) && !v.network && !v.kind.includes('REVIEW')).length;
  const network = vs.filter((v) => v.network).length;
  const safe = misses.filter((v) => v.kind.includes('safe')).length;
  const modelTimes = [];
  const networkTimes = [];
  for (const q of qs) {
    const r = results.get(q.id);
    (verdicts.get(q.id).network ? networkTimes : modelTimes).push(r.millis ?? 0);
  }
  const p50 = percentile(modelTimes, 0.5);
  const p95 = percentile(modelTimes, 0.95);
  const answeredRate = n === 0 ? 1 : (passed + wrong) / n;
  const passRate = n === 0 ? 1 : passed / n;
  let rulePassed;
  switch (cat) {
    case 'A':
    case 'C':
      rulePassed = wrong === 0 && answeredRate >= 0.98;
      break;
    case 'B':
      rulePassed = passRate >= 0.95;
      break;
    case 'D':
      rulePassed = passRate >= 0.98;
      break;
    default:
      rulePassed = wrong === 0 && unsafe === 0 && network === 0; // E: every miss is a safe end
  }
  const target = CATS[cat];
  const timeOk = n === 0 || (p50 <= target.p50 && p95 <= target.p95);
  return { cat, n, passed, wrong, safe, unsafe, network, answeredRate, passRate, p50, p95, networkTimes, rulePassed, timeOk };
}

function report(list, data) {
  const oracle = data.oracle ?? {};
  const results = new Map(data.results.map((r) => [r.id, r]));
  const ran = list.filter((q) => results.has(q.id));
  const verdicts = new Map(ran.map((q) => [q.id, score(q, results.get(q.id), oracle)]));
  const vlist = ran.map((q) => verdicts.get(q.id));
  const passed = vlist.filter((v) => v.pass).length;
  const lines = [];
  const say = (s = '') => lines.push(s);
  say(`AI EVAL  model ${data.meta.model}`);
  say(`endpoint ${data.meta.endpoint}`);
  say(`date ${new Date().toISOString().slice(0, 10)}   questions run ${ran.length} of ${list.length}   threads ${data.meta.threads ?? 1}`);
  say(`oracle facts ${Object.keys(oracle).length}   prefind ${data.meta.prefind === undefined ? 'not recorded' : data.meta.prefind ? 'ON' : 'off (as the app)'}`);
  say();
  say(`SCORE ${passed} of ${ran.length}`);
  say();
  say('CATEGORY                 pass      wrong  answered  model time p50 / p95 (target)           network  rule');
  const cats = Object.keys(CATS).filter((c) => ran.some((q) => q.cat === c));
  const stats = cats.map((c) => categoryStats(c, list, verdicts, results));
  for (const s of stats) {
    const t = CATS[s.cat];
    say(
      `${s.cat} ${t.title.padEnd(22)} ${`${s.passed}/${s.n}`.padEnd(8)} ${String(s.wrong).padEnd(6)} ${`${(100 * s.answeredRate).toFixed(1)}%`.padEnd(9)} ${ms(s.p50)} / ${ms(s.p95)} (${ms(t.p50)} / ${ms(t.p95)}) ${s.timeOk ? 'OK ' : 'OVER'}   ${String(s.network).padEnd(7)} ${s.rulePassed ? 'MEETS' : 'NOT YET'}  [${t.rule}]`,
    );
  }
  const netTimes = stats.flatMap((s) => s.networkTimes);
  say(`  (time is the model's: ${netTimes.length} questions that ended on a network failure or a timeout are counted apart, their time ${netTimes.length ? `p95 ${ms(percentile(netTimes, 0.95))}` : 'none'})`);
  say();
  const misses = vlist.filter((v) => !v.pass);
  const review = misses.filter((v) => v.kind.includes('REVIEW'));
  const count = (word) => misses.filter((v) => v.kind.includes(word)).length;
  say(`MISSES ${misses.length}: wrong or missing figures to review ${review.length}, safe ends ${count('safe')}, network ${count('network')}, harness ${count('harness')}, oracle ${count('oracle')}, other ${misses.length - review.length - count('safe') - count('network') - count('harness') - count('oracle')}`);
  for (const v of misses) {
    const q = list.find((x) => x.id === v.id);
    say(`  ${v.id} [${q.cat}] [${v.kind}] ${q.question}`);
    say(`      expected ${q.outcome}: ${q.figures || q.filters}`);
    say(`      got ${v.outcome}${v.failure ? ` (${v.failure})` : ''}; ${v.reasons.join('; ')}`);
  }
  const noted = vlist.filter((v) => v.notes.length > 0);
  say();
  say(`notes (not misses): ${noted.length} questions used another tool route, said a zero in words, or showed no name`);
  for (const v of noted.slice(0, 40)) say(`  ${v.id}: ${v.notes.join('; ')}`);
  // No "not understood" after the model has read data; no network failure.
  const afterReading = vlist.filter((v) => v.afterReading);
  say();
  say(`NOT UNDERSTOOD AFTER READING: ${afterReading.length} ${afterReading.length ? afterReading.map((v) => v.id).join(' ') : '(none, as plan 52 wants)'}`);
  const clicked = ran.map((q) => results.get(q.id)).filter((r) => r.clicked);
  if (clicked.length) {
    say(`CLICKED  ${clicked.length} misspelled names were asked "which one" and the matching option was clicked, as on the panel (both times counted): ${clicked.map((r) => `${r.id} ${r.clicked.name}`).join(', ')}`);
  }
  const failures = vlist.filter((v) => v.network);
  const byKind = {};
  for (const v of failures) byKind[v.failure ?? v.outcome] = (byKind[v.failure ?? v.outcome] ?? 0) + 1;
  say(`NETWORK FAILURES: ${failures.length} ${failures.length ? JSON.stringify(byKind) : '(none, as plan 52 wants)'}`);
  // Groups of the same meaning.
  const bad = groupAgreement(list, verdicts, results);
  say();
  say(`GROUPS of the same question (required figures only): ${bad.length === 0 ? 'every group reaches the same figures' : `${bad.length} DISAGREE`}`);
  for (const g of bad) say(`  ${g.group} ${g.members.join(' ')}: ${g.signs.join('  |  ')}`);
  // Tools, model calls, tokens.
  const all = ran.map((q) => results.get(q.id));
  const toolCalls = all.map((r) => (r.tool_calls ?? []).filter((c) => !c.error).length);
  const modelCalls = all.map((r) => r.model_calls ?? 0);
  const t = (k) => sum(all.map((r) => r.tokens?.[k] ?? 0));
  const parallel = all.filter((r) => (r.calls ?? []).some((c) => c.tool_calls >= 2));
  const badParallel = sum(parallel.map((r) => (r.tool_calls ?? []).filter((c) => c.error).length));
  say();
  say(`TOOL CALLS ${sum(toolCalls)} (${(sum(toolCalls) / Math.max(1, all.length)).toFixed(1)} a question)   MODEL CALLS ${sum(modelCalls)} (${(sum(modelCalls) / Math.max(1, all.length)).toFixed(1)} a question)`);
  say(`TOKENS  input ${t('in')}  cached ${t('cached')}  output ${t('out')}  reasoning ${t('reasoning')}`);
  say(`  per question: input ${Math.round(t('in') / Math.max(1, all.length))}, cached ${Math.round(t('cached') / Math.max(1, all.length))}, output ${Math.round(t('out') / Math.max(1, all.length))}`);
  // The fixed part as the model counts it (plan 52 item 0c): the prompt tokens of the FIRST call of a
  // fresh question, less the question itself.
  const fixed = all
    // A call the provider filtered answers with 0 prompt tokens (content_filter): nothing to measure.
    .filter((r) => !r.followup && typeof r.calls?.[0]?.prompt === 'number' && r.calls[0].prompt > 0)
    .map((r) => r.calls[0].prompt - (r.user_tokens ?? 0));
  const fixedMax = fixed.length ? Math.max(...fixed) : 0;
  const fixedO200k = Math.round(fixedMax / MODEL_COUNTS_MORE);
  say(
    `  fixed part, first call of ${fixed.length} fresh questions, as the model counts it: min ${fixed.length ? Math.min(...fixed) : 0}, p50 ${percentile(fixed, 0.5)}, max ${fixedMax} ${fixedMax <= PREFIX_MODEL ? 'OK' : 'OVER'} (limit about ${PREFIX_MODEL}); about ${fixedO200k} by o200k ${fixedO200k <= PREFIX_O200K ? 'OK' : 'OVER'} (D6-A19, NFR v1.13: at most ${PREFIX_O200K}; the prompt tokens of the first call less the question)`,
  );
  say(`PARALLEL tool calls that arrived with a bad input: ${badParallel} (in ${parallel.length} questions that asked for tools together)`);
  const limited = all.filter((r) => r.reply?.outcome === 'not_understood' && r.reply?.stop_reason);
  say(`LIMITS  ${limited.length} questions stopped at a limit (calls or tokens)`);
  const oneCall = list.filter((q) => q.filters.includes('onecall=1') && results.has(q.id));
  if (oneCall.length) {
    const one = oneCall.filter((q) => (results.get(q.id).model_calls ?? 0) <= 1).length;
    say(`ONE CALL  ${one} of ${oneCall.length} help or general advice questions needed one model call`);
  }
  const callTimes = all.flatMap((r) => (r.calls ?? []).map((c) => c.millis));
  say(`TIME  total ${ms(sum(all.map((r) => r.millis ?? 0)))}   one model call: p50 ${ms(percentile(callTimes, 0.5))}, p95 ${ms(percentile(callTimes, 0.95))}, max ${ms(callTimes.length ? Math.max(...callTimes) : 0)}   a follow-up counts its own time only`);
  say();
  say(`URDU REVIEW SHEET ${REVIEW} (${ran.length} answers; the reviewer marks good or not, and for advice answers inside FR-7.88 or not; at least 95 of 100 advice answers must be good)`);
  const advice = list.filter((q) => q.cat === 'B' && results.has(q.id));
  say(`  advice answers to read for FR-7.88: ${advice.map((q) => q.id).join(' ')}`);
  const meets = stats.length > 0 && stats.every((s) => s.rulePassed && s.timeOk) && bad.length === 0 && afterReading.length === 0 && failures.length === 0;
  say();
  say(
    meets
      ? 'RESULT: every category that ran meets plan 52 Section 2 (the Urdu review is still to do)'
      : `RESULT: does NOT yet meet plan 52 Section 2 (${stats.filter((s) => !s.rulePassed).map((s) => `${s.cat} score`).concat(stats.filter((s) => !s.timeOk).map((s) => `${s.cat} time`)).join(', ') || 'groups, reads or network'})`,
  );
  return { text: lines.join('\n'), meets, results, verdicts, stats, bad };
}

/** One text of every miss for the loop of plan 52 item 14: the answer, the calls, the time. */
function writeDetail(list, data, verdicts) {
  const oracle = data.oracle ?? {};
  const out = [];
  for (const q of list) {
    const v = verdicts.get(q.id);
    const r = (data.results ?? []).find((x) => x.id === q.id);
    if (!v || v.pass || !r) continue;
    let expected = q.figures;
    try {
      expected = resolveFigures(q.figures, oracle);
    } catch {
      /* keep the references */
    }
    out.push(`${q.id} [${q.cat}] ${q.world} ${v.kind}`);
    out.push(`  question  ${q.question}${q.after ? `   (after ${q.after})` : ''}`);
    out.push(`  expected  ${q.outcome}: ${expected || q.filters}`);
    out.push(`  got       ${v.outcome}${v.failure ? ` (${v.failure})` : ''}; ${v.reasons.join('; ')}`);
    out.push(`  answer    ${answerAsShown(r.reply)}`);
    out.push(`  tools     ${(r.tool_calls ?? []).map((c) => `${c.tool}${c.error ? '!' : ''}(${JSON.stringify(c.input ?? {}).slice(0, 110)})`).join('  ')}`);
    out.push(`  calls     ${(r.calls ?? []).map((c) => `${c.millis} ms ${c.finish} +${c.tool_calls}t`).join(' | ')}   ${r.millis} ms, ${r.model_calls} calls, finish ${r.finish_reason ?? '-'}`);
    if (r.calls_json) out.push(`  calls_json ${String(r.calls_json).slice(0, 600)}`);
    if (r.error) out.push(`  harness   ${r.error}`);
    out.push('');
  }
  mkdirSync(dirname(DETAIL), { recursive: true });
  writeFileSync(DETAIL, out.join('\n') + '\n', 'utf8');
}

// ---- running -------------------------------------------------------------------------------

/** The ids a run asks about, from --only, --category and --world. Empty: all of them. */
function selectIds(list, { only, category, world }) {
  const wanted = new Set((only ?? '').split(',').map((s) => s.trim()).filter(Boolean));
  const cats = new Set((category ?? '').split(',').map((s) => s.trim().toUpperCase()).filter(Boolean));
  const worlds = new Set((world ?? '').split(',').map((s) => s.trim()).filter(Boolean));
  if (wanted.size === 0 && cats.size === 0 && worlds.size === 0) return [];
  return list
    .filter((q) => (wanted.size === 0 || wanted.has(q.id)) && (cats.size === 0 || cats.has(q.cat)) && (worlds.size === 0 || worlds.has(q.world)))
    .map((q) => q.id);
}

/** Runs the ignored live test through cargo. The key is not passed: the app reads its own store. */
function runLive(ids, model, threads) {
  mkdirSync(OUT_DIR, { recursive: true });
  const env = { ...process.env, MANDI_EVAL_QUESTIONS: QUESTIONS, MANDI_EVAL_OUT: RESULTS };
  if (ids.length) env.MANDI_EVAL_ONLY = ids.join(',');
  if (model) env.MANDI_EVAL_MODEL = model;
  if (threads) env.MANDI_EVAL_THREADS = String(threads);
  const cmd = ['test', '-j', '4', '-p', 'mandi-assistant', '--test', 'robustness_live', '--', '--ignored', '--nocapture', 'live_100_questions'];
  console.log(`ai-eval: cargo ${cmd.join(' ')} in ${BACKEND}`);
  const res = spawnSync('cargo', cmd, { cwd: BACKEND, env, stdio: ['ignore', 'inherit', 'inherit'] });
  if (res.error) throw new Error(`cannot start cargo: ${res.error.message}`);
  if (res.status !== 0) throw new Error(`the live test failed with exit code ${res.status}`);
}

// ---- the self test -------------------------------------------------------------------------

/** Made-up replies for the scorer. Not a run: no model, no number here counts as a result. */
function selfTest() {
  const list = loadList();
  const { problems } = checkList(list);
  const fail = (m) => {
    console.error(`self-test FAILED: ${m}`);
    process.exit(2);
  };
  if (problems.length) fail(problems.join('; '));
  const expect = (cond, m) => {
    if (!cond) fail(m);
  };
  const dr = (n) => ({ type: 'figure', cell: { type: 'balance', paisa: n * 100, side: 'dr' }, dir: null, checked: true });
  const money = (n) => ({ type: 'figure', cell: { type: 'money', paisa: n * 100 }, dir: null, checked: true });
  const name = (en, ur) => ({ type: 'name', name: { name_en: en, name_ur: ur, tag: null, code: null } });
  const text = (t) => ({ type: 'text', text: t });
  const answered = (parts, extra = {}, answerExtra = {}) => ({
    outcome: 'answered',
    answer: { parts, tables: [], charts: [], estimate: null, advice: false, sources: [], differences: [], next: [], ...answerExtra },
    ...extra,
  });
  const byId = (id) => list.find((q) => q.id === id);

  // The old cases still hold.
  const q1 = byId('Q001');
  expect(score(q1, { reply: answered([name('Akram Toll Tax', 'اکرم ٹول ٹیکس'), text(' کے ذمے '), dr(2880)]), tool_calls: [{ tool: 'find' }, { tool: 'party_card' }] }, {}).pass, 'a right answer missed');
  const wrong = score(q1, { reply: answered([dr(2800)]), tool_calls: [] }, {});
  expect(!wrong.pass && wrong.kind.includes('REVIEW'), 'a wrong figure was not flagged for review');
  const ask = score(q1, { reply: { outcome: 'which_one', answer: null }, tool_calls: [] }, {});
  expect(!ask.pass && ask.kind.includes('safe'), 'which_one was not a safe end');
  const q96 = byId('Q096');
  expect(score(q96, { reply: { outcome: 'change_refused', answer: null, entry_route: { screen: 'wasooli' } }, tool_calls: [] }, {}).pass, 'a refusal missed');
  const sheet = answerAsShown(answered([name('Akram Toll Tax', 'اکرم ٹول ٹیکس'), text(' کے ذمے '), dr(2880)]));
  expect(sheet.includes('2,880 بنام ✓'), `the review text is "${sheet}"`);

  // Figures: forms, alternatives, optional, strict names.
  const e = expectedFigures('same_pace=288000;pace_month=2027-04;share=72.10%;sum=21564 Dr;opt?=5;top!=Banana|Apple;x=1|2');
  expect(e.length === 7 && e[1].alts[0].kind === 'month' && e[2].alts[0].n === 7210 && e[3].alts[0].side === 'dr', 'expected figures were read wrongly');
  expect(e[4].optional && e[5].strict && e[5].alts.length === 2 && e[6].alts.length === 2, 'optional, strict or alternative forms were read wrongly');
  const oracle = { 'a.x': '500', 'a.y': '700', 'n.top': 'Banana' };
  expect(resolveFigures('p=@a.x;q=@a.y', oracle) === 'p=500;q=700', 'the oracle was not filled in');
  let threw = false;
  try {
    resolveFigures('p=@a.nope', oracle);
  } catch {
    threw = true;
  }
  expect(threw, 'a missing oracle fact did not stop the scorer');
  const mk = (over) => ({ id: 'T1', cat: 'A', kind: 'x', outcome: 'answered', tools: '', filters: '', figures: '', ...over });
  // Either of two honest readings passes; neither fails.
  const alt = mk({ figures: 'v=@a.x|@a.y' });
  expect(score(alt, { reply: answered([money(700)]), tool_calls: [] }, oracle).pass, 'the second reading was not accepted');
  expect(!score(alt, { reply: answered([money(600)]), tool_calls: [] }, oracle).pass, 'a wrong figure passed as a reading');
  // An optional figure may be missing; a required one may not.
  expect(score(mk({ figures: 'g=@a.x;c?=@a.y' }), { reply: answered([money(500)]), tool_calls: [] }, oracle).pass, 'an optional figure was required');
  expect(!score(mk({ figures: 'g=@a.x;c=@a.y' }), { reply: answered([money(500)]), tool_calls: [] }, oracle).pass, 'a required figure was optional');
  // A strict name must be named.
  const top = mk({ figures: 'top!=@n.top' });
  expect(score(top, { reply: answered([name('Banana', 'کیلا')]), tool_calls: [] }, oracle).pass, 'a strict name missed');
  expect(!score(top, { reply: answered([name('Apple', 'سیب')]), tool_calls: [] }, oracle).pass, 'a wrong strict name passed');
  // A zero said in words passes only with the route matched and no other figure; a wrong one never.
  const zero = mk({ figures: 'gross=0', tools: 'tiles' });
  expect(score(zero, { reply: answered([text('آج کوئی بکری نہیں')]), tool_calls: [{ tool: 'tiles' }] }, {}).pass, 'a zero said in words missed');
  expect(!score(zero, { reply: answered([text('آج کوئی بکری نہیں')]), tool_calls: [{ tool: 'report' }] }, {}).pass, 'a zero passed with another route');
  expect(!score(zero, { reply: answered([money(1200)]), tool_calls: [{ tool: 'tiles' }] }, {}).pass, 'a zero passed beside another figure');
  const zeroNote = mk({ figures: 'gross=100' });
  expect(!score(zeroNote, { reply: answered([text('کوئی بکری نہیں')]), tool_calls: [] }, {}).pass, 'a words-only answer passed for a figure that is not 0');
  // Months belong to their method.
  const est = (kind, m) => ({
    outcome: 'estimate',
    answer: { parts: [], tables: [], charts: [], advice: false, sources: [], differences: [], next: [], estimate: { methods: [{ kind, total: { type: 'money', paisa: 100 }, target_month: { year: 2027, month: m } }] } },
  });
  const pm = mk({ outcome: 'estimate', figures: 'pace_month=2027-02' });
  expect(score(pm, { reply: est('same_pace', 2), tool_calls: [] }, {}).pass, 'the pace month missed');
  expect(!score(pm, { reply: est('same_growth', 2), tool_calls: [] }, {}).pass, 'the growth month passed for the pace month');
  // General advice: marked, with no figure; with data, from the books and not marked.
  const adviceReply = (general, parts, sources = []) => ({
    outcome: 'advice',
    answer: { parts, tables: [], charts: [], estimate: null, advice: true, general_advice: general, sources, differences: [], next: [] },
  });
  const gen = mk({ outcome: 'advice', filters: 'general=1;onecall=1' });
  expect(score(gen, { reply: adviceReply(true, [text('قیمت مناسب رکھیں')]), tool_calls: [], model_calls: 1 }, {}).pass, 'general advice missed');
  expect(!score(gen, { reply: adviceReply(true, [text('قیمت'), money(5)]), tool_calls: [], model_calls: 1 }, {}).pass, 'general advice with a figure passed');
  expect(!score(gen, { reply: adviceReply(false, [text('قیمت')], [{ kind: 'report' }]), tool_calls: [], model_calls: 1 }, {}).pass, 'unmarked advice passed as general');
  const books = mk({ outcome: 'advice', filters: 'general=0' });
  expect(score(books, { reply: adviceReply(false, [text('قیمت')], [{ kind: 'report' }]), tool_calls: [] }, {}).pass, 'advice from the books missed');
  expect(!score(books, { reply: adviceReply(true, [text('قیمت')]), tool_calls: [] }, {}).pass, 'general advice passed where the books have data');
  // One side of the commission passes only when the answer names the side (lead 03-11-2026).
  const side = mk({ figures: 'c=@a.x|@a.y#seller' });
  expect(score(side, { reply: answered([text('بیوپاری کا کمیشن '), money(700)]), tool_calls: [] }, oracle).pass, 'the seller side named was not accepted');
  expect(!score(side, { reply: answered([text('کمیشن '), money(700)]), tool_calls: [] }, oracle).pass, 'a bare commission passed for the seller side');
  expect(score(side, { reply: answered([text('کمیشن '), money(500)]), tool_calls: [] }, oracle).pass, 'the total was refused');
  // A which_one that the harness clicked is noted, and its answer is scored like any other.
  const clickedRow = score(mk({ figures: 'g=@a.x' }), { reply: answered([money(500)]), tool_calls: [], clicked: { name: 'Rahim' } }, oracle);
  expect(clickedRow.pass && clickedRow.notes.some((n) => n.includes('clicked')), 'a clicked question was not noted');
  // A figure question that ends as advice passes with one unasked point and the right figures only.
  const unasked = mk({ figures: 'g=@a.x' });
  const withAdvice = (bullets, general = false) => ({
    outcome: 'advice',
    answer: { parts: [money(500), text(`${'\n• نکتہ'.repeat(bullets)}`)], tables: [], charts: [], estimate: null, advice: true, general_advice: general, sources: [], differences: [], next: [] },
  });
  expect(score(unasked, { reply: withAdvice(1), tool_calls: [] }, oracle).pass, 'one unasked point was refused');
  expect(!score(unasked, { reply: withAdvice(2), tool_calls: [] }, oracle).pass, 'two unasked points passed');
  expect(!score(mk({ figures: 'g=@a.y' }), { reply: withAdvice(1), tool_calls: [] }, oracle).pass, 'advice with a wrong figure passed');
  expect(!score(unasked, { reply: withAdvice(0, true), tool_calls: [] }, oracle).pass, 'general advice passed for a figure question');
  // A network failure is its own kind of miss, and is kept out of the model's time.
  const down = score(mk({ figures: 'g=@a.x' }), { reply: { outcome: 'offline', answer: null, failure: { kind: 'offline' } }, tool_calls: [] }, oracle);
  expect(!down.pass && down.kind.includes('network') && down.network, 'a network failure was not its own kind');
  expect(score(mk({}), { reply: { outcome: 'not_understood', answer: null }, tool_calls: [{ tool: 'report' }] }, {}).afterReading, 'not understood after reading was not seen');

  // Groups agree on the required figures only.
  const g1 = { id: 'G1', group: 'GG', kind: 'x', cat: 'A' };
  const g2 = { id: 'G2', group: 'GG', kind: 'x', cat: 'A' };
  const same = new Map([['G1', { pass: true, figureMisses: [], network: false }], ['G2', { pass: true, figureMisses: [], network: false }]]);
  expect(groupAgreement([g1, g2], same, new Map([['G1', {}], ['G2', {}]])).length === 0, 'a group of passes disagreed');
  const mixed = new Map([['G1', { pass: true, figureMisses: [], network: false }], ['G2', { pass: false, figureMisses: ['gross'], network: false }]]);
  expect(groupAgreement([g1, g2], mixed, new Map([['G1', {}], ['G2', {}]])).length === 1, 'a group with one wrong member agreed');

  // Categories: the rule of each, the time apart from the network.
  const vs = new Map([
    ['C1', { pass: true, kind: 'PASS', outcome: 'answered', network: false }],
    ['C2', { pass: false, kind: 'MISS (REVIEW: figure, party, period or source)', outcome: 'answered', network: false }],
  ]);
  const cl = [{ id: 'C1', cat: 'A' }, { id: 'C2', cat: 'A' }];
  const rs = new Map([['C1', { millis: 5000 }], ['C2', { millis: 7000 }]]);
  const st = categoryStats('A', cl, vs, rs);
  expect(!st.rulePassed && st.wrong === 1, 'category A met its rule with a wrong figure');
  const vs2 = new Map([['N1', { pass: true, kind: 'PASS', outcome: 'answered', network: false }], ['N2', { pass: false, kind: 'MISS (network)', outcome: 'offline', network: true }]]);
  const st2 = categoryStats('D', [{ id: 'N1', cat: 'D' }, { id: 'N2', cat: 'D' }], vs2, new Map([['N1', { millis: 4000 }], ['N2', { millis: 60000 }]]));
  expect(st2.p95 === 4000 && st2.networkTimes.length === 1, 'network time was counted as model time');

  // Selecting by category, world and id.
  expect(selectIds(list, { category: 'D' }).every((id) => byId(id).cat === 'D'), '--category chose a wrong question');
  expect(selectIds(list, { world: 'thin' }).length > 0 && selectIds(list, { only: 'Q001,Q002' }).length === 2, '--world or --only chose wrongly');
  console.log(`self-test ok: the bank has ${list.length} questions in 5 categories and the scorer reads right, wrong, optional, zero, strict-name, general-advice and network cases`);
}

// ---- main ----------------------------------------------------------------------------------

function main() {
  if (flag('--self-test')) return selfTest();
  const list = loadList();
  const { problems, summary } = checkList(list);
  console.log(`ai-eval: bank ${QUESTIONS}`);
  const c = summary.byCat;
  console.log(
    `  ${summary.total} questions: A ${c.A}, B ${c.B}, C ${c.C}, D ${c.D}, E ${c.E}; Roman Urdu ${summary.languages.roman}, Urdu letters ${summary.languages.urdu}, English ${summary.languages.en}; ${summary.misspelled} misspelled; ${summary.groups} groups (${summary.bigGroups} of 3 or more)`,
  );
  if (problems.length) {
    console.error(`the bank breaks its rules:\n  ${problems.join('\n  ')}`);
    process.exit(2);
  }
  if (flag('--check')) {
    console.log('the bank meets its rules: the first 110 keep the D6 mix, all five categories, every section, every help topic, every @key is a fact of the oracle, and questions.tsv is what the spec builds');
    return;
  }
  const file = option('--results');
  const ids = selectIds(list, { only: option('--only'), category: option('--category'), world: option('--world') });
  if (!file) {
    try {
      runLive(ids, option('--model'), option('--threads'));
    } catch (error) {
      console.error(`ai-eval: ${error.message}`);
      process.exit(2);
    }
  }
  const path = file ? resolve(file) : RESULTS;
  if (!existsSync(path)) {
    console.error(`ai-eval: no results file at ${path}`);
    process.exit(2);
  }
  const data = JSON.parse(readFileSync(path, 'utf8'));
  if (data.waiting_for_key) {
    console.log('WAITING FOR KEY: no OpenRouter key is saved in the app. Umar pastes it in Settings, in the question group, then run this again.');
    process.exit(3);
  }
  const shown = ids.length ? new Set(ids) : null;
  const scored = shown ? list.filter((q) => shown.has(q.id)) : list;
  const { text, meets, results, verdicts, stats } = report(scored, data);
  console.log(text);
  writeReview(scored.filter((q) => results.has(q.id)), results);
  writeDetail(scored, data, verdicts);
  mkdirSync(OUT_DIR, { recursive: true });
  writeFileSync(
    SUMMARY,
    JSON.stringify(
      {
        date: new Date().toISOString(),
        model: data.meta.model,
        categories: stats.map(({ networkTimes, ...s }) => ({ ...s, networkQuestions: networkTimes.length })),
        misses: [...verdicts.values()].filter((v) => !v.pass).map((v) => v.id),
        meets,
      },
      null,
      2,
    ) + '\n',
    'utf8',
  );
  console.log(`DETAIL of every miss: ${DETAIL}`);
  process.exit(meets ? 0 : 1);
}

main();
