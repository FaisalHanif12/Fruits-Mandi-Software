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
//   node scripts/ai-eval/run.mjs --facet ITEM         only the questions of a facet (ITEM, VEG, CAP, A2 ...)
//   node scripts/ai-eval/run.mjs --facet ITEM --every 5   every fifth question of the selection (a sample)
//   node scripts/ai-eval/run.mjs --misses             ask again every miss and every question lost to
//                                                     the network, put the new rows in results.json
//   node scripts/ai-eval/run.mjs --only Q111,Q112 --merge   the same for a list you give
//   node scripts/ai-eval/run.mjs --threads 4          questions at once (default 3; --threads 1 is the quietest line).
//                                                     Each question has its own copy of its books and its own
//                                                     conversation, and its time is its own.
//   node scripts/ai-eval/run.mjs --check              check the bank, no model, no cargo
//   node scripts/ai-eval/run.mjs --self-test          check the scorer on made-up replies, no model
//   node scripts/ai-eval/run.mjs --results FILE       score a results file again, no model
//   --bank FILE         another bank file (for example a hidden one). Only the per-row rules apply (no
//                       size or mix rules, no frozen 110, no spec rebuild). Run live or with --check, scored
//                       the same way (speed budget and button checks too). The results go to
//                       backend/target/ai-eval-<name of the file>/, never into the team bank's folder.
//   --oracle-dump FILE  write the oracle facts with their values, and the names of the parties, items,
//                       partners, gardens and trips, as JSON. No model, no key. For the independent tester.
//   --model NAME        another model name (default: the app's default)
//   --review-out FILE   write the Urdu review sheet here instead of scripts/ai-eval/urdu-review.tsv
//   --detail-out FILE   write the detail of every miss here (default backend/target/ai-eval/misses.txt)
//   --summary-out FILE  write the summary here (default backend/target/ai-eval/summary.json)
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
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { WAITING } from './bank-spec.mjs';
import { buildText } from './build-bank.mjs';
import { CAPABILITY_FILE, matrix } from './build-capability.mjs';

const args = process.argv.slice(2);
const flag = (name) => args.includes(name);
const option = (name) => {
  const at = args.indexOf(name);
  return at >= 0 ? args[at + 1] : undefined;
};

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, '..', '..');
const BACKEND = join(ROOT, 'backend');
// --bank FILE: another bank (a hidden one). Its results live in their own folder.
const BANK = option('--bank') ? resolve(option('--bank')) : null;
const QUESTIONS = BANK ?? join(HERE, 'questions.tsv');
const FROZEN = join(HERE, 'questions-110.tsv');
const MANIFEST = join(HERE, 'oracle-keys.txt');
const OUT_DIR = join(
  BACKEND,
  'target',
  BANK ? `ai-eval-${basename(BANK).replace(/\.[^.]*$/, '')}` : 'ai-eval',
);
const REVIEW = option('--review-out')
  ? resolve(option('--review-out'))
  : BANK
    ? join(OUT_DIR, 'urdu-review.tsv')
    : join(HERE, 'urdu-review.tsv');
const THREADS_DEFAULT = '3';
const RESULTS = join(OUT_DIR, 'results.json');
const DETAIL = option('--detail-out')
  ? resolve(option('--detail-out'))
  : join(OUT_DIR, 'misses.txt');
const SUMMARY = option('--summary-out')
  ? resolve(option('--summary-out'))
  : join(OUT_DIR, 'summary.json');

// D6-A19, NFR v1.13: the fixed part is at most 11,000 tokens counted with o200k. The model counts
// about 3.4% more, so as the model counts it the limit is about 11,400.
// The limit of the fixed part of the prompt is read from the app (`limits::MAX_PREFIX_TOKENS`), never copied:
// the lead and Umar change it there. 11,000 only when the file cannot be read.
const PREFIX_O200K = (() => {
  try {
    const text = readFileSync(join(BACKEND, 'crates', 'assistant', 'src', 'limits.rs'), 'utf8');
    const found = text.match(/MAX_PREFIX_TOKENS:\s*u32\s*=\s*([0-9_]+)/);
    if (found) return Number(found[1].replaceAll('_', ''));
  } catch {
    // The file is not there (a copy of the scripts alone): the documented limit.
  }
  return 11000;
})();
const MODEL_COUNTS_MORE = 1.034;
// What the model counts for the same text: a little more than o200k, rounded up to a hundred.
const PREFIX_MODEL = Math.ceil((PREFIX_O200K * MODEL_COUNTS_MORE) / 100) * 100;
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
  'time_limit',
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
  'time_limit',
]);
// The kinds of failure that are the network's, not the model's (plan 52 item 6).
const NETWORK_KINDS = new Set(['timeout', 'busy', 'server']);
// The internet was lost: not a result of the app (the harness waits and asks again).
const LOST_KINDS = new Set(['offline', 'dropped']);

/** The five categories of plan 52 Section 2: their target times and what "meets" means. */
const CATS = {
  A: {
    title: 'real-time data',
    p50: 6000,
    p95: 12000,
    rule: 'no wrong figure, at least 98% answered',
  },
  B: { title: 'suggestions', p50: 6000, p95: 12000, rule: 'at least 95% good' },
  C: {
    title: 'forecasts and what-ifs',
    p50: 15000,
    p95: 25000,
    rule: 'no wrong figure, at least 98% answered',
  },
  D: { title: 'help with the app', p50: 5000, p95: 10000, rule: 'at least 98% the right topic' },
  E: {
    title: 'mixed and hard',
    p50: 6000,
    p95: 12000,
    rule: 'safe ends only, never a wrong figure',
  },
};

const HELP_TOPICS = [
  'bikri_book',
  'today_sales',
  'unsold_stock',
  'later_sale',
  'receipt',
  'collection_list',
  'payment',
  'advance',
  'pukhta_khata',
  'edit_and_cancel',
  'cash_book',
  'shop_expense',
  'personal_expense',
  'money_lent',
  'separate_khata',
  'general_journal',
  'fruit_taken',
  'gardens',
  'market_trips',
  'partners',
  'partner_capital',
  'analytics',
  'reports',
  'dashboard',
  'day_close',
  'month_close',
  'backup',
  'settings',
  'audit',
  'masters',
  'accounts',
  'search',
  'commission_rules',
  'dead_amount',
  'language_switch',
  'keys',
  'printing',
  'ask_the_app',
  'sign_in',
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
  party: 15,
  sales: 9,
  analytics: 11,
  dues: 9,
  cash: 7,
  partners: 6,
  stock: 8,
  sums: 7,
  estimates: 10,
  advice: 8,
  followup: 5,
  trap: 5,
  help: 10,
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

/**
 * Checks the whole bank. Returns problems and a summary. With `perRow` (a bank given with --bank)
 * only the rules of each row apply: unique ids, a known follow-up, a question and an outcome, a known
 * outcome, world and category, a section, no reference in a question, and every `@key` a fact of the
 * oracle. The size, the mix, the frozen 110, the facets and the spec rebuild belong to the team bank.
 */
function checkList(list, perRow = false) {
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
    if (q.after && !ids.has(q.after))
      problems.push(`${q.id} follows ${q.after}, which is not in the list`);
    if (!q.question || !q.outcome) problems.push(`${q.id} has no question or outcome`);
    if (!OUTCOMES.has(q.outcome)) problems.push(`${q.id} has the unknown outcome ${q.outcome}`);
    if (!WORLDS.has(q.world)) problems.push(`${q.id} has the unknown world ${q.world}`);
    if (!CATS[q.cat]) problems.push(`${q.id} has no category`);
    if (!q.section) problems.push(`${q.id} has no section`);
    if (/@/.test(q.question)) problems.push(`${q.id}: a question may not hold a reference`);
  }
  if (!perRow) {
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
    if (bigGroups < 35)
      problems.push(`only ${bigGroups} groups of 3 or more wordings (at least 35)`);
    // Category A covers every section of the books.
    const sectionCount = (s) => count((q) => q.cat === 'A' && q.section === s);
    const NEED3 = [
      'item',
      'seller',
      'buyer',
      'partner',
      'period',
      'cash',
      'dues',
      'stock',
      'day',
      'month',
      'expense',
    ];
    const NEED2 = ['garden', 'trip', 'fruit', 'capital'];
    for (const s of NEED3)
      if (sectionCount(s) < 3)
        problems.push(`category A: section ${s} has ${sectionCount(s)} (at least 3)`);
    for (const s of NEED2)
      if (sectionCount(s) < 2)
        problems.push(`category A: section ${s} has ${sectionCount(s)} (at least 2)`);
    // Every item, and the names of Umar's own examples, are in the bank.
    const allFigures = list.map((q) => q.figures).join(';');
    for (const item of ['banana', 'apple', 'mosambi', 'mango', 'grapes', 'potato'])
      if (!allFigures.includes(`.${item}.`)) problems.push(`no question reads the item ${item}`);
    for (const text of [
      'kela ke is saal kitne commission or sale huwe',
      'Faisal ne kitne ka saman khareeda',
      'sales bherane ke tareeky batao',
    ])
      if (!list.some((q) => q.question.includes(text)))
        problems.push(`Umar's own question is missing: ${text}`);
    if (!list.some((q) => /akram/i.test(q.question) && q.outcome === 'which_one'))
      problems.push('no question about the two Akrams ends in which_one');
    // Category D covers every topic of the guide.
    const topics = new Set(
      list.map((q) => (q.filters.match(/topic=([a-z_]+)/) ?? [])[1]).filter(Boolean),
    );
    for (const t of HELP_TOPICS)
      if (!topics.has(t)) problems.push(`no help question for the topic ${t}`);
    // General advice is tested where the books are thin.
    if (count((q) => q.filters.includes('general=1')) < 8)
      problems.push('fewer than 8 general advice questions');
    // The three facets of plan 52 section 8.1: M (mixed language), a party's day, no such party.
    const facetOf = (q) => (q.filters.match(/(?:^|;)facet=([A-Za-z0-9]+)/) ?? [])[1];
    const facets = {
      M: list.filter((q) => facetOf(q) === 'M'),
      day: list.filter((q) => facetOf(q) === 'day'),
      nosuch: list.filter((q) => facetOf(q) === 'nosuch'),
      print: list.filter((q) => facetOf(q) === 'print'),
      U: list.filter((q) => facetOf(q) === 'U'),
      A2: list.filter((q) => facetOf(q) === 'A2'),
      MIX: list.filter((q) => facetOf(q) === 'MIX'),
      CAP: list.filter((q) => facetOf(q) === 'CAP'),
      VEG: list.filter((q) => facetOf(q) === 'VEG'),
      ITEM: list.filter((q) => facetOf(q) === 'ITEM'),
      MONEY: list.filter((q) => facetOf(q) === 'MONEY'),
    };
    const A_SECTIONS = [
      'item',
      'seller',
      'buyer',
      'partner',
      'period',
      'cash',
      'dues',
      'stock',
      'garden',
      'trip',
      'fruit',
      'expense',
      'capital',
      'day',
      'month',
    ];
    const mixedSections = new Set(facets.M.map((q) => q.section));
    for (const s of A_SECTIONS)
      if (!mixedSections.has(s))
        problems.push(`facet M: no mixed-language question for the section ${s}`);
    for (const lang of ['roman', 'urdu', 'en'])
      for (const [name, qs] of Object.entries(facets))
        if (!qs.some((q) => q.lang === lang)) problems.push(`facet ${name}: no ${lang} wording`);
    // Facets CAP and ITEM hold one question for a cell of the capability matrix (the lead's ruling, 05-11-2026):
    // a single wording needs no group.
    for (const [name, qs] of Object.entries(facets))
      for (const q of qs)
        if (!q.group && name !== 'CAP' && name !== 'ITEM' && name !== 'MONEY')
          problems.push(`${q.id} (facet ${name}) is in no group of the same meaning`);
    const dayText = facets.day.map((q) => q.filters).join(' ');
    for (const need of ['buyer=', 'seller=', 'period=today', 'period=yesterday', 'period=2026-'])
      if (!dayText.includes(need)) problems.push(`facet day: no question with ${need}`);
    if (!facets.day.some((q) => (q.figures.match(/_qty=/g) ?? []).length >= 2))
      problems.push('facet day: no day with two items');
    // Print (plan 54 Section 6): every kind of party for every period, in three languages.
    const printCells = new Set(
      facets.print.map(
        (q) =>
          `${(q.filters.match(/pkind=([a-z]+)/) ?? [])[1]}/${(q.filters.match(/pperiod=([a-z_]+)/) ?? [])[1]}`,
      ),
    );
    for (const kind of ['buyer', 'seller', 'all', 'khata'])
      for (const period of [
        'today',
        'yesterday',
        'month',
        'last_month',
        'named',
        'range',
        'last_week',
      ])
        if (!printCells.has(`${kind}/${period}`))
          problems.push(`facet print: no question for ${kind} / ${period}`);
    if (!facets.print.some((q) => q.filters.includes('pkind=ambiguous')))
      problems.push('facet print: no question for a name that two accounts share');
    if (!facets.print.some((q) => q.filters.includes('pkind=nosuch')))
      problems.push('facet print: no question for a name nobody has');
    if (
      facets.print
        .filter((q) => q.figures.includes('print.report='))
        .some((q) => !q.figures.includes('print.steps='))
    )
      problems.push('facet print: a print question without the steps');
    // Umar's own questions (plan 55 gate 4), word for word, each with an Urdu and an English twin.
    for (const text of [
      'mujhay faisal ka khata batao?',
      'usman shirakat daar ka kitna hai abhi ?',
      'apni sale ko ziyada kaise karain ?',
      'hum apni sale ko kaise bhera sakhty hain ?',
      'mujhay bikri add krne ka tareeka batao?',
      'is main market safar kiya hai iski mujhay samaj nahi aa rahi ?',
      'sales bherane ke tareeky batao ?',
      'faisal ki aj ki bikri ka main ne print nikalna hai ?',
      'main ne faisal ki aj ki bikri ka print nikalna hai ?',
    ])
      if (!facets.U.some((q) => q.question === text))
        problems.push(`facet U: Umar's question is missing or changed: ${text}`);
    const nsText = facets.nosuch.map((q) => q.filters).join(' ');
    for (const need of [
      'asked_role=seller',
      'asked_role=buyer',
      'found_as=buyer',
      'found_as=seller',
    ])
      if (!nsText.includes(need)) problems.push(`facet nosuch: no question with ${need}`);
    if (!facets.nosuch.some((q) => q.misspelled && q.outcome === 'answered'))
      problems.push('facet nosuch: no misspelled real name that must be found');
    // Facet CAP (plan 55 Section 11): the cells of the capability matrix that had no question.
    if (facets.CAP.length < 60)
      problems.push(`facet CAP: ${facets.CAP.length} questions (at least 60)`);
    // Facet MIX (Umar 05-11-2026): a real figure and a forecast in ONE question.
    const mix = facets.MIX;
    if (mix.length < 24) problems.push(`facet MIX: ${mix.length} questions (at least 24)`);
    if (mix.some((q) => q.cat !== 'C' || q.outcome !== 'estimate'))
      problems.push('facet MIX: every question is category C and ends as an estimate');
    for (const q of mix) {
      if (!/real_/.test(q.figures)) problems.push(`${q.id} (facet MIX) has no real_ figure`);
      if (!/pending=/.test(q.filters) && !/est_/.test(q.figures))
        problems.push(`${q.id} (facet MIX) has no est_ figure and is not marked pending`);
    }
    // Facet VEG (Umar, 06-11-2026): vegetables as well as fruit. One item's figure, the group, its commission,
    // a forecast, a print, mixed language, misspellings, and the checks that a vegetable carries no laga.
    const veg = facets.VEG;
    if (veg.length < 81) problems.push(`facet VEG: ${veg.length} questions (at least 81)`);
    const vegText = (pick) => veg.filter(pick).length;
    if (!vegText((q) => q.outcome === 'estimate'))
      problems.push('facet VEG: no forecast for a vegetable');
    if (!vegText((q) => q.figures.includes('print.report=')))
      problems.push('facet VEG: no print for a vegetable');
    if (!vegText((q) => q.filters.includes('nolaga=1')))
      problems.push('facet VEG: no check that a vegetable carries no laga');
    if (!vegText((q) => q.figures.includes('@group.vegetable.')))
      problems.push('facet VEG: no question on the vegetable group');
    if (!vegText((q) => q.misspelled)) problems.push('facet VEG: no misspelled wording');
    // Facet ITEM (Umar, 06-11-2026): the item axis of the capability matrix, one question for each answerable cell
    // of capability-items.tsv (measures x subjects x periods). At least 700 questions, every measure and every
    // subject asked, in every language, some misspelled; a vegetable never carries a laga; an item that is not in
    // the books ends as not_found.
    const item = facets.ITEM;
    if (item.length < 700) problems.push(`facet ITEM: ${item.length} questions (at least 700)`);
    const itemTag = (name) =>
      new Set(
        item.map((q) => (q.filters.match(new RegExp(`(?:^|;)${name}=([a-z_0-9]+)`)) ?? [])[1]),
      );
    for (const m of [
      'gross',
      'qty',
      'comm_seller',
      'comm_buyer',
      'laga',
      'comm_total',
      'avg_rate',
      'last_rate',
      'buyers_n',
      'sellers_n',
      'top_buyer',
      'top_seller',
      'rank',
      'top_item',
      'vs',
      'best_month',
      'trend',
      'stock',
      'forecast',
      'print',
      'open',
    ])
      if (!itemTag('measure').has(m)) problems.push(`facet ITEM: no question for the measure ${m}`);
    for (const s of ['fruit', 'veg', 'g_fruit', 'g_veg', 'all', 'party', 'unknown', 'zero'])
      if (!itemTag('subject').has(s)) problems.push(`facet ITEM: no question for the subject ${s}`);
    for (const lang of ['roman', 'urdu', 'en'])
      if (!item.some((q) => q.lang === lang)) problems.push(`facet ITEM: no ${lang} wording`);
    if (!item.some((q) => q.misspelled)) problems.push('facet ITEM: no misspelled wording');
    if (!item.some((q) => q.filters.includes('nolaga=1')))
      problems.push('facet ITEM: no check that a vegetable carries no laga');
    if (!item.some((q) => q.outcome === 'not_found' && q.filters.includes('asked_role=item')))
      problems.push('facet ITEM: no item that is not in the books');
    // Facet MONEY (lead, 06-11-2026): the period axis of the money and party subjects, one question for each
    // answerable cell of capability-money.tsv (subjects x figure, list, compare and top x periods). At least 400
    // questions, every kind of question and every language, some misspelled.
    const money = facets.MONEY;
    if (money.length < 400) problems.push(`facet MONEY: ${money.length} questions (at least 400)`);
    const moneyTag = new Set(money.map((q) => (q.filters.match(/(?:^|;)qtype=([a-z]+)/) ?? [])[1]));
    for (const t of ['figure', 'list', 'compare', 'top'])
      if (!moneyTag.has(t)) problems.push(`facet MONEY: no ${t} question`);
    for (const lang of ['roman', 'urdu', 'en'])
      if (!money.some((q) => q.lang === lang)) problems.push(`facet MONEY: no ${lang} wording`);
    if (!money.some((q) => q.misspelled)) problems.push('facet MONEY: no misspelled wording');
    // Facet A2 (plan 55 Section 10): 120 real-time figure questions, asked any way.
    const a2 = facets.A2;
    if (a2.length < 120) problems.push(`facet A2: ${a2.length} questions (at least 120)`);
    if (a2.some((q) => q.cat !== 'A' || q.outcome !== 'answered'))
      problems.push('facet A2: every question is category A and ends in an answer');
    if (a2.filter((q) => q.misspelled).length < 12)
      problems.push('facet A2: fewer than 12 misspelled wordings');
    const a2Text = a2.map((q) => `${q.filters};${q.figures}`).join(' ');
    for (const need of [
      'balance=',
      'advance=',
      'cash=',
      'total=',
      'payable=',
      'gross=',
      'comm=',
      'qty=',
      'buyer=',
      'seller=',
      'item=',
      'partner=',
      'period=today',
      'period=yesterday',
      'period=this week',
      'period=this month',
      'period=last month',
      'period=September',
      'period=15 October',
      'period=this year',
      'period=up to now',
    ])
      if (!a2Text.includes(need)) problems.push(`facet A2: no question with ${need}`);
    // The same figure family in the wordings: short, with no question word, mixed language.
    const words = (q) => q.question.trim().split(/\s+/).length;
    if (!a2.some((q) => words(q) <= 3)) problems.push('facet A2: no very short wording');
    if (!a2.some((q) => /[A-Za-z]/.test(q.question) && /[\u0600-\u06FF]/.test(q.question)))
      problems.push('facet A2: no mixed Urdu letters and English wording');
  }
  // Every `@key` is a fact the oracle gives.
  const keys = manifestKeys();
  if (!keys) problems.push('scripts/ai-eval/oracle-keys.txt is missing');
  else
    for (const q of list)
      for (const ref of refsOf(q.figures))
        if (!keys.has(ref)) problems.push(`${q.id}: the oracle has no fact @${ref}`);
  // The capability matrix (plan 55 Section 11) has no empty cell and is what the rows and the bank build.
  if (!perRow) {
    try {
      const built = matrix();
      problems.push(...built.problems);
      if (!existsSync(CAPABILITY_FILE) || readFileSync(CAPABILITY_FILE, 'utf8') !== built.text)
        problems.push(
          'capability.tsv is not what capability-rows.mjs and the bank build: run node scripts/ai-eval/build-capability.mjs',
        );
    } catch (error) {
      problems.push(`the capability matrix does not build: ${error.message}`);
    }
  }
  // The file is what the spec builds.
  if (!perRow) {
    try {
      if (readFileSync(QUESTIONS, 'utf8') !== buildText())
        problems.push(
          'questions.tsv is not what bank-spec.mjs builds: run node scripts/ai-eval/build-bank.mjs',
        );
    } catch (error) {
      problems.push(`the spec does not build: ${error.message}`);
    }
  }
  const facetOf = (q) => (q.filters.match(/(?:^|;)facet=([A-Za-z0-9]+)/) ?? [])[1];
  const facets = Object.fromEntries(
    ['M', 'day', 'nosuch', 'print', 'U', 'A2', 'MIX', 'CAP', 'VEG', 'ITEM', 'MONEY'].map((k) => [
      k,
      list.filter((q) => facetOf(q) === k),
    ]),
  );
  return {
    problems,
    summary: {
      total: n,
      byCat,
      languages,
      misspelled,
      groups: groups.size,
      bigGroups,
      facets: Object.fromEntries(Object.entries(facets).map(([k, v]) => [k, v.length])),
    },
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
    out.all.push(
      ...cellValues(method.total),
      ...cellValues(method.per_month),
      ...cellValues(method.step),
    );
    const target = monthOf(method.target_month);
    if (target) out.months.add(`${method.kind}:${target}`);
  }
  for (const chart of answer.charts ?? []) {
    for (const series of chart.series ?? [])
      for (const cell of series.values ?? []) out.all.push(...cellValues(cell));
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
  if ((m = value.match(/^(-?\d+(?:\.\d+)?)%$/)))
    return { kind: 'percent', n: Math.round(Number(m[1]) * 100) };
  if (/^\d{4}-\d{2}$/.test(label === 'date' ? '' : value)) return { kind: 'month', month: value };
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return { kind: 'date', date: value };
  if ((m = value.match(/^(-?\d+)(?:\s+(Dr|Cr|up|down))?/)))
    return { kind: 'amount', n: Number(m[1]), side: m[2]?.toLowerCase(), text: value };
  // "Arshad Multani 18684 Dr 36 days": a name with its amount (advice).
  if ((m = value.match(/^([A-Za-z ]+?)\s+(-?\d+)(?:\s+(Dr|Cr))?/)))
    return {
      kind: 'amount',
      n: Number(m[2]),
      side: m[3]?.toLowerCase(),
      text: value,
      name: m[1].toLowerCase(),
    };
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
    const same =
      v.n === want.n || (want.n < 0 && v.kind === 'balance' && v.side === 'cr' && v.n === -want.n);
    const side =
      !want.side || want.side === 'up' || want.side === 'down' || v.side === want.side || v.n === 0;
    return same && side;
  });
}

// ---- scoring one question ------------------------------------------------------------------

/** The titles of the workings that make an estimate or a what-if (never a figure of the books). */
const FORECAST_WORKING =
  /^assistant-working-(same-pace|same-growth|same-season|what-if-pace|percent-more|target|balance-pace)/;
// `balance-pace` also covers `balance-pace-cash` and `balance-pace-dues` (the estimates of a balance, PRD FR-7.86).

/**
 * Splits the figures an answer shows into those it gives as BOOK figures (a figure with no working
 * or with a working of the books, and every table cell) and those it gives as ESTIMATES (a figure
 * whose working is a forecast or a what-if, a method of the estimate, and the months of a forecast
 * chart). A mixed question (a real figure and a forecast, facet MIX, PRD FR-7.76 point 4) must keep
 * them apart: the real figure is never shown as an estimate and the estimate never as a book figure.
 */
function bookAndEstimate(reply) {
  const answer = reply?.answer;
  const book = { parts: [], all: [] };
  const estimate = { parts: [], all: [] };
  if (!answer) return { book, estimate };
  for (const part of answer.parts ?? []) {
    if (part.type !== 'figure') continue;
    const values = cellValues(part.cell);
    const target = FORECAST_WORKING.test(part.working?.title_id ?? '') ? estimate : book;
    target.parts.push(...values);
    target.all.push(...values);
  }
  for (const table of answer.tables ?? []) {
    const cells = [...(table.totals ?? [])];
    for (const row of table.rows ?? []) cells.push(...(row.row?.cells ?? []));
    for (const cell of cells) book.all.push(...cellValues(cell));
  }
  for (const method of answer.estimate?.methods ?? []) {
    estimate.all.push(
      ...cellValues(method.total),
      ...cellValues(method.per_month),
      ...cellValues(method.year_total),
    );
  }
  for (const chart of answer.charts ?? [])
    for (const series of chart.series ?? [])
      if (series.kind !== 'actual')
        for (const cell of series.values ?? []) estimate.all.push(...cellValues(cell));
  return { book, estimate };
}

/**
 * The separation rule of a mixed question. A figure labelled `real_...` must be shown as a book figure;
 * a figure labelled `est_...` must be shown as an estimate. Returns the reasons of the figures that are
 * there but under the wrong kind (a figure that is not there at all is reported as missing already).
 */
function mixSeparation(reply, wanted) {
  const reasons = [];
  const { book, estimate } = bookAndEstimate(reply);
  const met = (found, want) =>
    want.alts.some(
      (alt) =>
        alt.kind !== 'name' && alt.kind !== 'month' && alt.kind !== 'date' && hasNumber(found, alt),
    );
  for (const want of wanted) {
    if (want.label.startsWith('real_') && !met(book, want) && met(estimate, want))
      reasons.push(`mixed: ${want.label} is a figure of the books but is shown as an estimate`);
    if (want.label.startsWith('est_') && !met(estimate, want) && met(book, want))
      reasons.push(`mixed: ${want.label} is an estimate but is shown as a figure of the books`);
  }
  return reasons;
}

function toolSeen(log, token) {
  const [tool, detail] = token.split(':');
  return (log ?? []).some(
    (c) => c.tool === tool && (!detail || c.input?.id === detail || c.input?.view === detail),
  );
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
    return (
      !alt.strict &&
      JSON.stringify(reply?.answer?.sources ?? [])
        .toLowerCase()
        .includes(alt.name)
    );
  }
  if (alt.kind === 'month') {
    // A target month belongs to its method: `pace_month` is the same-pace method's month.
    const word = (label.match(/^(pace|growth|season)_month$/) ?? [])[1];
    return [...found.months].some(
      (m) => (word ? m.startsWith(`same_${word}:`) : true) && m.endsWith(alt.month),
    );
  }
  if (alt.kind === 'date') return true; // a date is shown as a date part; the text is not checked further
  return hasNumber(found, alt);
}

/**
 * The `print.` pieces of an expected-figures text (plan 54 Section 6): the report the button must open
 * and its party and dates, apart from the figures. `print.report=R5;print.party=Faisal;print.from=..`.
 */
function splitPrint(text) {
  const want = {};
  const rest = [];
  for (const piece of text.split(';').filter(Boolean)) {
    const at = piece.indexOf('=');
    const label = piece.slice(0, at).trim();
    if (label.startsWith('print.')) want[label.slice(6)] = piece.slice(at + 1).trim();
    else rest.push(piece);
  }
  return { want: Object.keys(want).length ? want : null, rest: rest.join(';') };
}

/** The report sources (the buttons) of an answer. */
function reportSources(reply) {
  // An analytics view (A to K) is a button too (plan 55 W2): its letter is its id, like a report's R-code.
  return (reply?.answer?.sources ?? [])
    .filter((s) => s.kind === 'report' || s.kind === 'analytics')
    .map((s) => (s.kind === 'analytics' ? { ...s, id: s.view } : s));
}

/**
 * A filters object as the question log writes it for a local answer: only the filters that are set
 * (an empty text, false and null are left out), at any depth, so two objects compare as text.
 */
function compactFilters(value) {
  if (Array.isArray(value)) return value.map(compactFilters);
  if (value && typeof value === 'object') {
    const out = {};
    for (const key of Object.keys(value).sort()) {
      const v = compactFilters(value[key]);
      if (v === null || v === false || v === '' || v === undefined) continue;
      if (v && typeof v === 'object' && !Array.isArray(v) && Object.keys(v).length === 0) continue;
      out[key] = v;
    }
    return out;
  }
  return value;
}

/**
 * The filters of a local `report` log entry. The app writes them beside the report id in `input`
 * (backend's final shape, 05-11-2026: `{"id":"R35","as_of":"...","party":{...}}`); a nested `filters`
 * key is read as well.
 */
function filtersOfEntry(input) {
  if (input?.filters && typeof input.filters === 'object') return input.filters;
  const { id: _id, ...rest } = input ?? {};
  return rest;
}

/**
 * A local answer writes one `report` entry in its log for each report button, in order, with the
 * button's own filters (`input.id`, `input.filters`). They must be the same as the buttons of the
 * answer: a log that says one thing and a button that opens another is a defect.
 */
function logButtonProblems(result, reply) {
  const entries = (result?.tool_calls ?? []).filter(
    (c) => c?.local === true && c.tool === 'report' && c.input,
  );
  if (entries.length === 0) return [];
  const buttons = reportSources(reply);
  const problems = [];
  if (entries.length !== buttons.length)
    problems.push(
      `the local log has ${entries.length} report entries, the answer ${buttons.length} report buttons`,
    );
  entries.slice(0, buttons.length).forEach((entry, i) => {
    const button = buttons[i];
    if (entry.input.id !== button.id)
      problems.push(`log entry ${i + 1} is ${entry.input.id}, the button is ${button.id}`);
    else if (
      JSON.stringify(compactFilters(filtersOfEntry(entry.input))) !==
      JSON.stringify(compactFilters(button.filters ?? {}))
    )
      problems.push(`log entry ${i + 1} (${entry.input.id}) has other filters than its button`);
  });
  return problems;
}

/**
 * The reads of a LOCAL answer against its buttons (PRD FR-7.90, plan 55 Section 10): an instant figure
 * answer reads with the tools, and a local print, help or screen answer logs what it used. Either way
 * every read of a report, a view, the tiles or a party card must have its button (the same report, the
 * same view), and every such button must have a read, so the log never says one thing and the button
 * another. When a read and its button both carry dates they must be the same dates.
 */
function figureLogProblems(result, reply) {
  const reads = (result?.tool_calls ?? []).filter(
    (c) =>
      c?.local === true &&
      !c.error &&
      ['report', 'analytics', 'tiles', 'party_card'].includes(c.tool),
  );
  if (reads.length === 0) return [];
  const keyOfRead = (c) =>
    c.tool === 'report'
      ? `report:${c.input?.id ?? '?'}`
      : c.tool === 'analytics'
        ? `analytics:${c.input?.view ?? '?'}`
        : c.tool;
  const buttons = (reply?.answer?.sources ?? []).filter((src) =>
    ['report', 'analytics', 'tiles', 'party_card'].includes(src.kind),
  );
  const keyOfButton = (src) =>
    src.kind === 'report'
      ? `report:${src.id ?? '?'}`
      : src.kind === 'analytics'
        ? `analytics:${src.view ?? '?'}`
        : src.kind;
  const count = (keys) => {
    const out = new Map();
    for (const k of keys) out.set(k, (out.get(k) ?? 0) + 1);
    return out;
  };
  const readCount = count(reads.map(keyOfRead));
  const buttonCount = count(buttons.map(keyOfButton));
  const problems = [];
  for (const [key, n] of readCount)
    if ((buttonCount.get(key) ?? 0) < 1)
      problems.push(`the log has ${n} read(s) of ${key} and the answer has no such button`);
  for (const [key, n] of buttonCount)
    if ((readCount.get(key) ?? 0) < 1)
      problems.push(`the answer has ${n} button(s) ${key} and the log has no such read`);
  // The dates of a read are its button's dates (one button of the same kind carries them).
  for (const read of reads) {
    const key = keyOfRead(read);
    const mine = buttons.filter((src) => keyOfButton(src) === key);
    if (mine.length === 0) continue;
    const input = filtersOfEntry(read.input ?? {});
    for (const field of ['from', 'to', 'as_of']) {
      const wanted = input[field];
      if (typeof wanted !== 'string' || !wanted) continue;
      if (!mine.some((src) => (src.filters ?? {})[field] === wanted))
        problems.push(`the read of ${key} has ${field} ${wanted}, its button has another`);
    }
  }
  return problems;
}

/** One line for a report button: the report, the party and the dates it opens with. */
function describeButton(source) {
  const f = source.filters ?? {};
  const name = (rec) => (rec?.record ?? rec)?.name_en;
  const who = name(f.party) ?? name(f.seller) ?? name(f.zamindar) ?? 'no party';
  const dates = f.as_of
    ? `as_of ${f.as_of}`
    : f.shortcut && !f.from && !f.to
      ? `shortcut ${f.shortcut}`
      : `${f.from || '-'}..${f.to || '-'}`;
  return `${source.id} ${who} ${dates}`;
}

/** Does one report source meet what the print question wants? Returns the reasons it does not. */
function printProblems(want, source) {
  const f = source.filters ?? {};
  const problems = [];
  const name = (rec) => (rec?.record ?? rec)?.name_en?.toLowerCase();
  if (!want.report.split('|').includes(source.id))
    problems.push(`report ${source.id}, wanted ${want.report}`);
  if (want.party && name(f.party) !== want.party.toLowerCase())
    problems.push(`party ${name(f.party) ?? 'none'}, wanted ${want.party}`);
  if (
    want.seller &&
    ![name(f.seller), name(f.zamindar), name(f.party)].includes(want.seller.toLowerCase())
  )
    problems.push(`seller ${name(f.seller) ?? name(f.party) ?? 'none'}, wanted ${want.seller}`);
  // A partner, a garden, a city or a truck stand named in the question is the record the button is set to.
  for (const kind of ['partner', 'garden', 'city', 'adda', 'item']) {
    if (want[kind] && name(f[kind]) !== want[kind].toLowerCase())
      problems.push(`${kind} ${name(f[kind]) ?? 'none'}, wanted ${want[kind]}`);
  }
  // The shop's guests (R37) and one بکری by its number (R1) are filters of their own.
  if (want.guests === '1' && f.guests_only !== true) problems.push('not set to the guests');
  if (want.bikri && Number(f.bikri?.number) !== Number(want.bikri))
    problems.push(`bikri ${f.bikri?.number ?? 'none'}, wanted ${want.bikri}`);
  if (want.as_of) {
    if (f.as_of !== want.as_of) problems.push(`as_of ${f.as_of || 'empty'}, wanted ${want.as_of}`);
  } else if (want.dates === 'none') {
    // No period named, and the report is of a person or a thing: the app prints the whole books, with
    // no dates set (plan.rs `dated(.., whole_books)`).
    if (f.from || f.to)
      problems.push(
        `dates ${f.from || 'empty'} to ${f.to || 'empty'}, wanted none (no period named: the whole books)`,
      );
  } else {
    if (f.from !== want.from)
      problems.push(
        `from ${f.from || 'empty'}${f.shortcut ? ` (shortcut ${f.shortcut})` : ''}, wanted ${want.from}`,
      );
    if (f.to !== want.to) problems.push(`to ${f.to || 'empty'}, wanted ${want.to}`);
  }
  return problems;
}

/** True when the reply carries the steps field of the print contract: `answer.how` (plan 55 Section 7). */
function hasStepsField(reply) {
  return !!reply?.answer && typeof reply.answer === 'object' && 'how' in reply.answer;
}

/** The value of a step argument: a text, the English name of a `names`, or a number. */
function stepArg(step, name) {
  const arg = (step?.args ?? []).find((a) => a.name === name);
  const v = arg?.value;
  if (!v) return null;
  if (v.type === 'text') return v.text;
  if (v.type === 'names') return v.names?.en ?? null;
  if (v.type === 'number') return String(v.value);
  return null;
}

/**
 * The steps of a print answer against its button (plan 55 Section 7): open the report, set the party
 * (his real role, name and code), set the day or the dates, press print. Each value is the button's.
 */
function stepProblems(want, source, how) {
  const problems = [];
  const f = source.filters ?? {};
  const steps = Array.isArray(how) ? how : [];
  if (steps.length === 0) return ['no steps'];
  const ids = steps.map((st) => st.id);
  const byId = (id) => steps.find((st) => st.id === id);
  const isView = source.kind === 'analytics';
  const openId = isView ? 'assistant-how-open-analytics' : 'assistant-how-open-report';
  if (ids[0] !== openId) problems.push(`the first step is ${ids[0]}, not ${openId}`);
  if (want.intent === 'navigate') {
    // An open request (`print.intent=navigate`): the same steps up to the filters, and no press-print.
    if (ids.includes('assistant-how-press-print'))
      problems.push('an open request must not press print');
  } else if (ids[ids.length - 1] !== 'assistant-how-press-print')
    problems.push(`the last step is ${ids[ids.length - 1]}, not press-print`);
  const open = byId(openId);
  const openedWhat = open && stepArg(open, isView ? 'view' : 'report');
  if (open && openedWhat !== (isView ? source.view : source.id))
    problems.push(`steps open ${openedWhat}, the button opens ${isView ? source.view : source.id}`);
  const partyWanted = want.party ?? want.seller;
  // The item of a report is set with the same kind of step as a party; only the role tells them apart.
  const setParty = steps.find(
    (st) => st.id === 'assistant-how-set-party' && stepArg(st, 'role') !== 'item',
  );
  if (partyWanted) {
    if (!setParty) problems.push('no set-party step');
    else {
      if ((stepArg(setParty, 'party') ?? '').toLowerCase() !== partyWanted.toLowerCase())
        problems.push(`steps set the party ${stepArg(setParty, 'party')}, wanted ${partyWanted}`);
      if (want.role && stepArg(setParty, 'role') !== want.role)
        problems.push(`steps set the role ${stepArg(setParty, 'role')}, he is a ${want.role}`);
      const record =
        (f.party ?? f.seller ?? f.zamindar)?.record ?? f.party ?? f.seller ?? f.zamindar;
      if (record?.code && stepArg(setParty, 'code') !== record.code)
        problems.push(
          `steps give the code ${stepArg(setParty, 'code')}, the button has ${record.code}`,
        );
    }
  } else if (setParty && !want.optionalParty) {
    // A report with no party (R34, R2) names none in its steps.
    problems.push('a set-party step on a report with no party');
  }
  if (want.as_of) {
    const day = byId('assistant-how-set-day');
    if (!day || stepArg(day, 'day') !== want.as_of)
      problems.push(
        `steps set the day ${day ? stepArg(day, 'day') : 'none'}, wanted ${want.as_of}`,
      );
  } else if (want.dates === 'none') {
    // The whole books: the steps set no dates and no day.
    if (byId('assistant-how-set-dates') || byId('assistant-how-set-day'))
      problems.push('steps set dates, but no period was named (the whole books)');
  } else {
    const dates = byId('assistant-how-set-dates');
    const day = byId('assistant-how-set-day');
    if (!dates && day && want.from && want.from === want.to) {
      // One day (from = to): plan 55 Section 7 lets the steps say "set the day" for it.
      if (stepArg(day, 'day') !== want.from)
        problems.push(`steps set the day ${stepArg(day, 'day')}, wanted ${want.from}`);
    } else if (!dates) problems.push('no set-dates step');
    else {
      if (stepArg(dates, 'from') !== want.from)
        problems.push(`steps set from ${stepArg(dates, 'from')}, wanted ${want.from}`);
      if (stepArg(dates, 'to') !== want.to)
        problems.push(`steps set to ${stepArg(dates, 'to')}, wanted ${want.to}`);
    }
  }
  return problems;
}

/** The text a numeric cell is compared by: whole rupees, a count, `p<hundredths>`, `g<grams>`. */
function cellKey(cell) {
  switch (cell?.type) {
    case 'money':
    case 'balance':
      return String(Math.abs(Math.trunc(cell.paisa / 100)));
    case 'number':
      return String(Math.abs(cell.value));
    case 'percent':
      return `p${cell.hundredths}`;
    case 'weight':
      return `g${cell.grams}`;
    default:
      return null;
  }
}

/**
 * The guide topics a question was answered from. A model answer asks the guide (an `app_help` call).
 * A local answer (plan 55 Section 2, no model call) makes no tool call; the app writes its guide
 * topic in the question log as a local entry of `tool_calls_json`. The shape is read loosely so the
 * log may name the topic in `input.topic`, `topic` or `args.topic`: an entry of `app_help`, or any
 * entry marked `local`.
 */
function helpTopicsOf(result) {
  const topics = [];
  for (const c of result?.tool_calls ?? []) {
    if (!c || (c.tool !== 'app_help' && c.local !== true)) continue;
    const topic = c.input?.topic ?? c.topic ?? c.args?.topic;
    if (typeof topic === 'string') topics.push(topic);
  }
  return topics;
}

/** True when the app logs its local answers (any entry of the row is marked `local`). */
function logsLocalEntries(result) {
  return (result?.tool_calls ?? []).some((c) => c && c.local === true);
}

/**
 * The numbers a question itself holds, as the keys of a figure: its digits, and a number with a unit
 * word spelled out ("30 lakh" is 3,000,000, "1 crore" 10,000,000, "5 hazar" 5,000). The app gives
 * them back in its answer (a target), but they are not figures of the books.
 */
function numbersOfQuestion(text) {
  const keys = new Set((text.match(/\d+/g) ?? []).map((d) => String(Number(d))));
  const unit = /(\d+)\s*(lakh|lac|lakhs|لاکھ|crore|karor|کروڑ|hazar|hazaar|hazar|ہزار)/giu;
  const times = (word) =>
    /^(lakh|lac|lakhs|لاکھ)$/i.test(word)
      ? 100000
      : /^(crore|karor|کروڑ)$/i.test(word)
        ? 10000000
        : 1000;
  for (const m of text.matchAll(unit)) keys.add(String(Number(m[1]) * times(m[2])));
  return keys;
}

/**
 * The values an answer gives as a forecast, by their keys: the months of a chart that are not
 * actual, and the totals and monthly values of every method of the estimate. A forecast is the app's
 * own working, so it is never in a report of the books.
 */
function forecastKeys(answer) {
  const keys = new Set();
  const add = (cell) => {
    const key = cellKey(cell);
    if (key) keys.add(key);
  };
  for (const chart of answer?.charts ?? [])
    for (const series of chart.series ?? [])
      if (series.kind !== 'actual') for (const value of series.values ?? []) add(value);
  for (const method of answer?.estimate?.methods ?? []) {
    add(method.per_month);
    add(method.total);
    add(method.year_total);
    for (const step of method.working?.steps ?? []) add(step.result);
    // The same-growth method adds one step to the last month, again and again; the months it
    // projects appear as inputs of its last sum but are not made by a step of the working.
    const stepPaisa = method.step?.paisa;
    if (Number.isInteger(stepPaisa)) {
      const sign = method.step_dir === 'down' ? -1 : 1;
      const bases = (method.working?.steps ?? []).flatMap((st) =>
        (st.inputs ?? []).filter((i) => i?.type === 'money').map((i) => i.paisa),
      );
      for (const base of bases)
        for (let k = 1; k <= 24; k += 1) add({ type: 'money', paisa: base + sign * k * stepPaisa });
    }
  }
  return keys;
}

/**
 * The figures of an answer a button must hold (plan 55 gate 3), as {key, count}: every figure of
 * its text, except one that came from a calculation (then the inputs of its working are the
 * figures), and every number of its tables. A `count` is a plain number (a number of entries, a
 * number of days), not an amount.
 *
 * In a working, three kinds of input are not figures of a report and are left out (plan 55 gate 3,
 * lead 04-11-2026): the result of an earlier step; a constant of the calculation (a `number` or a
 * `percent` input: the months used, a multiplier, a factor); and a value of the forecast the answer
 * itself makes. The history the forecast stands on (the actual months) stays checked.
 */
/**
 * The monthly changes of a balance estimate (PRD FR-7.86, `domain::forecast::balance_at_pace`): the first step of its
 * working is the average of the net changes of the last months (in minus out of each month), each one a figure the app
 * read from `balance_series`, not a cell of the R12 or R13 button. They count as found in the books when the step is
 * an average of at least two of them that adds up: the sum of the changes over their number is the step's result,
 * within one rupee (a party's balance changes are signed by their side: debit plus, credit minus).
 */
function monthlyChangesAgree(step) {
  if (step?.op !== 'average') return false;
  const signed = (cell) => {
    if (cell?.type === 'money') return cell.paisa;
    if (cell?.type === 'balance') return cell.side === 'cr' ? -cell.paisa : cell.paisa;
    return null;
  };
  const months = (step.inputs ?? []).filter((i) => i?.type === 'money' || i?.type === 'balance');
  const values = months.map(signed);
  const result = signed(step.result);
  if (months.length < 2 || result === null || values.some((v) => v === null)) return false;
  const mean = values.reduce((a, b) => a + b, 0) / values.length;
  return Math.abs(mean - result) <= 100;
}

function answerFigures(answer) {
  const out = [];
  const forecast = forecastKeys(answer);
  // The result of any step of any working in the answer: a figure the app worked out itself, which
  // may be shown beside the working that made it (the total of the months already booked).
  const worked = new Set();
  for (const part of answer?.parts ?? [])
    for (const st of part.working?.steps ?? []) worked.add(cellKey(st.result));
  worked.delete(null);
  const isForecast = (key) => forecast.size > 0 && (forecast.has(key) || worked.has(key));
  for (const part of answer?.parts ?? []) {
    if (part.type !== 'figure') continue;
    const steps = part.working?.steps ?? [];
    if (steps.length) {
      const results = new Set(steps.map((st) => cellKey(st.result)));
      const balancePace = /^assistant-working-balance-pace/.test(part.working?.title_id ?? '');
      for (const [at, st] of steps.entries())
        for (const input of st.inputs ?? []) {
          if (input?.type === 'number' || input?.type === 'percent') continue;
          // A monthly change of a balance estimate is found in the books (see monthlyChangesAgree).
          if (balancePace && at === 0 && monthlyChangesAgree(st)) continue;
          const key = cellKey(input);
          // The result of an earlier step, or a forecast value, is not a figure of a report.
          if (key && !results.has(key) && !forecast.has(key)) out.push({ key, count: false });
        }
    } else {
      const key = cellKey(part.cell);
      if (key && !isForecast(key)) out.push({ key, count: part.cell?.type === 'number' });
    }
  }
  for (const table of answer?.tables ?? []) {
    const cells = [...(table.totals ?? [])];
    for (const row of table.rows ?? []) cells.push(...(row.row?.cells ?? []));
    for (const cell of cells) {
      const key = cellKey(cell);
      if (key) out.push({ key, count: cell?.type === 'number' });
    }
  }
  return out;
}

/** The English name of a `found_as` record. */
function roleName(found) {
  return found?.name?.name_en ?? null;
}

/**
 * The sides a named party is read on in the sources of an answer: `seller` when a source is filtered
 * to him as a seller, `buyer` as a buyer. A party card or a ledger takes neither side.
 */
function sourceSides(reply, who) {
  const sides = new Set();
  const is = (rec) => {
    const r = rec?.record ?? rec;
    return !!r && typeof r.name_en === 'string' && r.name_en.toLowerCase() === who;
  };
  for (const source of reply?.answer?.sources ?? []) {
    const f = source.filters ?? {};
    if (is(f.seller) || is(f.zamindar)) sides.add('seller');
    if (is(f.buyer)) sides.add('buyer');
  }
  return sides;
}

/** The role a `found_as` holds: a word, or a record with its role (`role`, `type`, `party_type`, `kind`). */
function roleOf(found) {
  if (!found) return null;
  if (typeof found === 'string') return found;
  return found.role ?? found.type ?? found.party_type ?? found.kind ?? found.party?.type ?? null;
}

/**
 * The verdict of one question: pass or a miss with its reasons, and notes that are not misses.
 * A wrong figure is always a miss: an expected figure that is not shown fails the question.
 */
function score(question, result, oracle, secrets = null) {
  const reply = result.reply;
  const outcome = reply?.outcome ?? 'none';
  // A name found in another role (plan 52 Section 8.5): the question waited, the harness clicked his
  // name, and `reply` is the answer after the click. The first reply (outcome and roles) is judged
  // apart, the answer after the click on his figures.
  const clickFirst = result.first_reply ?? null;
  const gate = clickFirst ?? reply;
  const judged = gate?.outcome ?? 'none';
  const found = collect(reply);
  const reasons = [];
  const notes = [];
  const figureMisses = [];
  let wanted = [];
  let printWant = null;
  try {
    const split = splitPrint(resolveFigures(question.figures, oracle));
    printWant = split.want;
    wanted = expectedFigures(split.rest);
  } catch (error) {
    return {
      id: question.id,
      cat: question.cat,
      pass: false,
      kind: 'MISS (oracle)',
      reasons: [error.message],
      notes,
      outcome,
      figureMisses,
    };
  }
  const expectedOutcome = question.outcome;
  // A mixed question (a real figure and a forecast) ends as an estimate or as an answer.
  const isMix = /(?:^|;)(?:facet=MIX|mix=1)(?:;|$)/.test(question.filters);
  if (expectedOutcome === 'safe') {
    const ok =
      outcome === 'answered' ||
      outcome === 'outside_books' ||
      outcome === 'change_refused' ||
      SAFE_ENDS.has(outcome);
    if (!ok) reasons.push(`outcome ${outcome}`);
    if (found.all.some((v) => v.n === 99999)) reasons.push('shows 99,999');
  } else if (judged !== expectedOutcome && !(isMix && ['estimate', 'answered'].includes(judged))) {
    // PRD FR-7.88, "unasked, one point" (Umar): a figure question may end as advice when the advice
    // part is one point at most (one bullet). The figures are scored below as always, so a wrong
    // or missing figure is still a miss.
    const onePoint = (found.text.match(/•/g) ?? []).length <= 1;
    const unasked =
      expectedOutcome === 'answered' &&
      outcome === 'advice' &&
      reply?.answer?.general_advice !== true &&
      onePoint;
    if (unasked) notes.push('advice: one unasked point (FR-7.88)');
    else reasons.push(`outcome ${judged}, wanted ${expectedOutcome}`);
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
      reasons.push(
        `figure ${want.label}=${want.alts.map((a) => a.text ?? a.n).join(' or ')} missing`,
      );
      figureMisses.push(want.label);
    }
  }
  // A vegetable carries no laga (PRD 4.2: the laga rule is for the fruit group). `nolaga=1` on a question about
  // vegetables: no table of the answer may hold a laga column with an amount in it, in a row or in the totals.
  if (/(?:^|;)nolaga=1(?:;|$)/.test(question.filters) && reply) {
    for (const table of reply.answer?.tables ?? []) {
      const at = (table.columns ?? []).findIndex((c) => c.key === 'laga');
      if (at < 0) continue;
      const cells = [
        ...(table.totals ?? []).slice(at, at + 1),
        ...(table.rows ?? []).map((r) => r.row?.cells?.[at]),
      ];
      const bad = cells.find((cell) => cell?.type === 'money' && Number(cell.paisa) !== 0);
      if (bad) {
        reasons.push(`a laga of ${Math.trunc(bad.paisa / 100)} is shown on a vegetable (PRD 4.2)`);
        figureMisses.push('nolaga');
        break;
      }
    }
  }
  // Privacy (PRD FR-7.83): `privacy=1;never=mark_text|narration_words|phone|address`. The real values come from
  // the books of the run (`meta.secrets`, written by the harness, never in the oracle dump). Nothing the user
  // sees or the model was shown may hold one: the answer text and tables, the sources and buttons, the log of
  // the tools and what the tools gave the model (`tool_results`). A phone is compared by its digits.
  if (/(?:^|;)privacy=1(?:;|$)/.test(question.filters)) {
    const never = ((question.filters.match(/(?:^|;)never=([^;]+)/) ?? [])[1] ?? '')
      .split('|')
      .map((k) => k.trim())
      .filter(Boolean);
    if (!secrets) {
      notes.push('privacy: the results hold no secrets of the books, so the check was not made');
    } else {
      const haystack = JSON.stringify([
        reply?.answer ?? null,
        reply?.entry_route ?? null,
        result.buttons ?? [],
        result.tool_calls ?? [],
        result.tool_results ?? '',
        result.answer_json ?? '',
      ]).toLowerCase();
      const digits = haystack.replace(/[^0-9]/g, '');
      for (const kind of never) {
        for (const value of secrets[kind] ?? []) {
          const text = String(value).trim().toLowerCase();
          if (!text) continue;
          const leaked =
            kind === 'phone'
              ? (() => {
                  const d = text.replace(/[^0-9]/g, '');
                  return d.length >= 7 && digits.includes(d);
                })()
              : haystack.includes(text) || haystack.includes(JSON.stringify(text).slice(1, -1));
          if (leaked) {
            reasons.push(
              `privacy: the ${kind} "${value}" is in what the app showed or read (FR-7.83)`,
            );
            figureMisses.push('privacy');
            break;
          }
        }
      }
    }
  }
  // Text the app must never give (PRD FR-7.83): `hide=XQZ-77|Electricity bill paid` in the filters. A mark's text
  // and a narration's words are kept back from the model, so they are not in the answer in any form.
  const hideWant = (question.filters.match(/(?:^|;)hide=([^;]+)/) ?? [])[1];
  if (hideWant && reply) {
    const shown =
      JSON.stringify(reply.answer ?? {}).toLowerCase() + (found.text ?? '').toLowerCase();
    for (const secret of hideWant
      .split('|')
      .map((t) => t.trim().toLowerCase())
      .filter(Boolean))
      if (shown.includes(secret)) {
        reasons.push(`the answer shows "${secret}", which the app keeps back (FR-7.83)`);
        figureMisses.push('hide');
      }
  }
  // A figure the app does not forecast (plan 55 Section 11.2, `noforecast=1`): today's real figure, and no
  // forecast of any kind in the answer. Saying it does not forecast is the honest answer.
  if (/(?:^|;)noforecast=1(?:;|$)/.test(question.filters) && reply) {
    const shown = bookAndEstimate(reply).estimate.all.length;
    if (reply.answer?.estimate || shown > 0) {
      reasons.push('a forecast is shown for a figure the app does not forecast');
      figureMisses.push('noforecast');
    }
  }
  if (isMix) {
    for (const why of mixSeparation(reply, wanted)) {
      reasons.push(why);
      figureMisses.push('mixed');
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
  const second = figuresText.match(/second\??=([A-Za-z ]+?)\s+\d/);
  if (first && second) {
    const a = found.order.indexOf(first[1].toLowerCase());
    const b = found.order.indexOf(second[1].toLowerCase());
    if (a < 0 || (b >= 0 && a > b)) {
      reasons.push(`order: ${first[1]} should come before ${second[1]}`);
      figureMisses.push('order');
    }
  }
  // A help question must ask the guide for its topic (plan 51 B3).
  // A local answer (finish_reason `local`, no model call) asks the guide through the app itself; the
  // question log names its topic in a local entry. A row of a build that does not log it yet cannot
  // be judged: it is PENDING (neither a pass nor a miss) and the run cannot say "meets".
  const topicWant = question.filters.match(/topic=([a-z_]+)/);
  let helpPending = false;
  if (topicWant && !helpTopicsOf(result).includes(topicWant[1])) {
    const localAnswer = result.finish_reason === 'local' && (result.model_calls ?? 0) === 0;
    if (localAnswer && !logsLocalEntries(result)) {
      helpPending = true;
      notes.push(
        `help (pending the local log entry): the guide topic of a local answer is not in the log`,
      );
    } else reasons.push(`app_help topic ${topicWant[1]} not asked`);
  }
  // A change request ends in a refusal that points to the right entry screen.
  const route = question.filters.match(/route=([a-z_]+)/);
  if (route && reply?.entry_route?.screen !== route[1])
    reasons.push(`entry route is not ${route[1]}`);
  // "No such party" (plan 52 section 8): a not_found says which role was asked (`asked_role`) and,
  // when the name is a party of another role, which (`found_as`). Until the app puts the field in
  // the reply the question is PENDING, not a pass and not a miss.
  const wantsRole = (question.filters.match(/(?:^|;)asked_role=([a-z_]+)/) ?? [])[1];
  const wantsFound = (question.filters.match(/(?:^|;)found_as=([a-z_]+)/) ?? [])[1];
  const fieldThere = !!gate && ('asked_role' in gate || 'found_as' in gate);
  // A row from a build that has the fields (`asked_role_field`) is scored on them; a row from an
  // older build, with neither field, waits.
  const contract = result.asked_role_field === true;
  const pending = !!(wantsRole || wantsFound) && !!gate && !fieldThere && !contract;
  if (!fieldThere && contract && (wantsRole || wantsFound) && judged === 'not_found')
    reasons.push('a not_found with no asked_role (the build has the field)');
  if (fieldThere) {
    if (wantsRole && gate.asked_role !== wantsRole)
      reasons.push(`asked_role is ${gate.asked_role ?? 'empty'}, wanted ${wantsRole}`);
    const foundRole = roleOf(gate.found_as);
    if (wantsFound && foundRole !== wantsFound)
      reasons.push(`found_as is ${foundRole ?? 'empty'}, wanted ${wantsFound}`);
    if (wantsRole && !wantsFound && foundRole)
      reasons.push(`found_as is ${foundRole} for a name nobody has`);
  }
  // Plan 55 gate 3, every button against the data: each report or view the answer offers was opened
  // by the harness with the button's own filters through the same backend command; the answer's
  // figures must be in what it holds. A figure that is not there is a wrong place or wrong filters.
  if (Array.isArray(result.buttons) && result.buttons.length > 0 && reply?.answer) {
    // R31 (a partner's statement) is built from the services' statement, not by `reports::run`. A
    // row made before the harness opened it that way holds the refusal of `reports::run`; that says
    // nothing about the answer, so such a button is a note and not a miss.
    const harnessGap = (b) => /R31 is built from the statement of the services/.test(b.error ?? '');
    if (result.buttons.some(harnessGap))
      notes.push('button R31: the old harness could not open it, not checked');
    const real = result.buttons.filter((b) => !b.skipped && !harnessGap(b));
    if (real.length === 0) {
      notes.push('buttons: only the dashboard tiles, not checked');
    } else {
      for (const b of real)
        if (b.error) reasons.push(`button ${b.label} fails to open: ${b.error}`);
      // What a button holds: the numbers of its cells and the number of its rows (an answer says
      // "128 entries" or "9 lots"; the screen shows that count as its row count).
      const held = new Set(
        real.flatMap((b) => [
          ...(b.numbers ?? []),
          ...(Number.isInteger(b.rows) ? [String(b.rows)] : []),
        ]),
      );
      const ownNumbers = numbersOfQuestion(question.question);
      // A party card button (R13 with his name) opens his ledger; the days or the entries the card
      // gave ("16 days old") are not a cell of that ledger, so a plain count is not checked there.
      const hasCard = real.some((b) => /^R13 /.test(b.label ?? ''));
      const missing = [
        ...new Set(
          answerFigures(reply.answer)
            .filter((f) => !held.has(f.key) && !ownNumbers.has(f.key) && !(f.count && hasCard))
            .map((f) => f.key),
        ),
      ];
      if (missing.length) {
        const why = `button: ${missing.length} figure(s) of the answer are not in what the button opens (${real.map((b) => b.label).join(', ')}): ${missing.slice(0, 4).join(', ')}`;
        if (result.buttons.some((b) => b.skipped))
          notes.push(
            `${why} (a screen button, the dashboard or the rules, is also there, not checked)`,
          );
        else {
          reasons.push(why);
          figureMisses.push('button');
        }
      }
    }
  }
  // A print request (plan 54 Section 6): the answer is a button on the right report with the party
  // in his real role and the exact dates, and the steps; it is never a change request. Until the
  // app puts the steps in the reply (the S2 contract) the question is PENDING, and the button check
  // is only a note.
  // FR-7.90: the log of a local or instant answer is its buttons (a print has its own, stricter check).
  if (!printWant && reply && result.finish_reason === 'local')
    for (const why of figureLogProblems(result, reply)) reasons.push(`log: ${why}`);
  let printPending = false;
  if (printWant && reply) {
    const contractIn = result.print_contract === true || hasStepsField(reply);
    // A local print answer writes one report entry for each button; they must agree (plan 55).
    for (const why of logButtonProblems(result, reply)) reasons.push(`log: ${why}`);
    const found = reportSources(reply);
    const problems = [];
    if (reply.outcome === 'change_refused' || reply.entry_route)
      problems.push('a print request was answered as a change request');
    if (!found.length) problems.push('no report button');
    else if (!found.some((src) => printProblems(printWant, src).length === 0)) {
      problems.push(
        `the button is wrong: ${found.map((src) => `${describeButton(src)} (${printProblems(printWant, src).join('; ')})`).join(' | ')}`,
      );
    }
    if (contractIn && printWant.steps) {
      // The button is the first source (plan 55 Section 7), and the steps are made from its values.
      const first = reply.answer?.sources?.[0];
      if (
        !first ||
        !['report', 'analytics'].includes(first.kind) ||
        printProblems(printWant, first.kind === 'analytics' ? { ...first, id: first.view } : first)
          .length > 0
      )
        problems.push(
          `the first source is not the right button: ${first ? (['report', 'analytics'].includes(first.kind) ? describeButton(first.kind === 'analytics' ? { ...first, id: first.view } : first) : first.kind) : 'none'}`,
        );
      const matching = found.find((src) => printProblems(printWant, src).length === 0);
      if (matching)
        for (const why of stepProblems(printWant, matching, reply.answer?.how)) problems.push(why);
      else problems.push('no steps to check, the button is wrong');
    }
    if (contractIn) {
      for (const why of problems) reasons.push(`print: ${why}`);
      if (problems.length) figureMisses.push('print');
    } else {
      printPending = true;
      notes.push(
        problems.length
          ? `print (pending the steps field): ${problems.join('; ')}`
          : 'print (pending the steps field): the button is right',
      );
    }
  }
  // After the click on a name found in another role: his figures in his real role, never in the role
  // that was asked ("Faisal ne becha": his purchases, no sale figure).
  if (clickFirst && wantsFound) {
    const who = (roleName(clickFirst.found_as) ?? '').toLowerCase();
    const sides = sourceSides(reply, who);
    if (wantsFound === 'buyer' && sides.has('seller'))
      reasons.push(
        `after the click: ${who} is read as a seller, but he is a buyer (sales instead of purchases)`,
      );
    if (wantsFound === 'seller' && sides.has('buyer'))
      reasons.push(
        `after the click: ${who} is read as a buyer, but he is a seller (purchases instead of sales)`,
      );
    if (reply?.outcome !== 'answered')
      reasons.push(
        `after the click: outcome ${reply?.outcome ?? 'none'}, wanted answered (his figures)`,
      );
  }
  if (
    question.outcome === 'change_refused' &&
    question.id === 'Q096' &&
    reply?.entry_route?.screen !== 'wasooli'
  )
    reasons.push('entry route is not wasooli');
  if (question.outcome === 'outside_books' && question.filters.includes('nearest=party_card')) {
    if (!JSON.stringify(reply?.answer?.sources ?? []).includes('party_card'))
      reasons.push('no party card button');
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
  if (question.filters.includes('general=1') && !general)
    reasons.push('not marked as general advice');
  if (question.filters.includes('general=0')) {
    if (general) reasons.push('marked general advice although the books have the data');
    else if (outcome === 'advice' && (reply?.answer?.sources ?? []).length === 0)
      reasons.push('advice with no source in the books');
  }
  if (question.filters.includes('onecall=1') && (result.model_calls ?? 0) > 1)
    notes.push(`${result.model_calls} model calls (one wanted)`);
  // Tools are a note: another route to the same figure is not a miss.
  if (!matched) {
    for (const group of question.tools.split(',').filter(Boolean))
      if (!group.split('/').some((t) => toolSeen(result.tool_calls, t)))
        notes.push(`tool ${group} not used`);
  }
  if (result.clicked)
    notes.push(
      `clicked after ${result.clicked.first_outcome ?? 'which_one'}: ${result.clicked.name}`,
    );
  const afterReading =
    outcome === 'not_understood' && (result.tool_calls ?? []).some((c) => !c.error);
  const failure = reply?.failure?.kind ?? null;
  // LOST: the internet itself went (offline, or a connection that dropped). Such a question is not
  // scored at all: it is listed on its own and asked again (the harness waits for the connection and
  // asks it again first; `network_lost` says it still failed).
  const lost =
    !!reply &&
    (outcome === 'offline' || LOST_KINDS.has(failure ?? '') || result.network_lost === true);
  // NETWORK: the service failed (a timeout, busy, a server error): a real result, a miss.
  const network = !!reply && !lost && (outcome === 'error' || NETWORK_KINDS.has(failure ?? ''));
  const shows = found.parts.length + found.all.length > 0;
  // `pending=<reason>`: the question has no route yet (a gap of capability.tsv); not a pass, not a miss.
  const routePending = (question.filters.match(/(?:^|;)pending=([a-z_]+)/) ?? [])[1] ?? null;
  let kind;
  if (lost) kind = 'LOST (network)';
  else if (pending) kind = 'PENDING (asked_role is not in the reply yet)';
  else if (printPending) kind = 'PENDING (the print steps are not in the reply yet)';
  else if (helpPending) kind = 'PENDING (the guide topic of a local answer is not in the log yet)';
  else if (routePending) kind = `PENDING (no route yet: ${routePending})`;
  else if (reasons.length === 0) kind = 'PASS';
  else if (!reply) kind = 'MISS (harness)';
  else if (network) kind = 'MISS (network)';
  else if (SAFE_ENDS.has(outcome)) kind = 'MISS (safe end)';
  else if (shows || outcome === 'answered') kind = 'MISS (REVIEW: figure, party, period or source)';
  else kind = 'MISS';
  return {
    id: question.id,
    cat: question.cat,
    pass:
      reasons.length === 0 && !lost && !pending && !printPending && !helpPending && !routePending,
    kind,
    reasons,
    notes,
    outcome,
    figureMisses,
    afterReading,
    network,
    lost,
    pending: pending || printPending || helpPending || !!routePending,
    failure,
  };
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
        const topics = helpTopicsOf(results.get(q.id));
        if (v.pending) return 'network';
        return [...new Set(topics)].sort().join(',');
      }
      if (v.network || v.lost || v.pending) return 'network'; // a lost connection, or a field not built yet, says nothing about the figures
      return `${v.pass ? 'ok' : 'miss'}:${[...new Set(v.figureMisses)].sort().join(',')}`;
    });
    const real = signs.filter((s) => s !== 'network');
    if (new Set(real).size > 1) bad.push({ group, members: members.map((q) => q.id), signs });
  }
  return bad;
}

// ---- the review sheet ----------------------------------------------------------------------

const MONTHS_UR = [
  'جنوری',
  'فروری',
  'مارچ',
  'اپریل',
  'مئی',
  'جون',
  'جولائی',
  'اگست',
  'ستمبر',
  'اکتوبر',
  'نومبر',
  'دسمبر',
];
const group3 = (n) => Math.abs(n).toLocaleString('en-US');
const DIR_UR = { up: 'زیادہ', down: 'کم', same: 'برابر' };

/** One figure as the panel shows it. */
function showFigure(part) {
  const cell = part.cell;
  let text = '';
  if (cell.type === 'money') text = `${cell.paisa < 0 ? '-' : ''}${group3(cell.paisa / 100)}`;
  else if (cell.type === 'balance')
    text =
      `${group3(cell.paisa / 100)} ${cell.side === 'dr' ? 'بنام' : cell.side === 'cr' ? 'جمع' : ''}`.trim();
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
  if (answer.tables?.length)
    extra.push(`[جدول: ${answer.tables.map((t) => t.rows.length).join('، ')} قطاریں]`);
  if (answer.estimate) extra.push('[اندازہ، یقینی نہیں]');
  if (answer.general_advice) extra.push('[عام مشورہ]');
  else if (answer.advice) extra.push('[مشورہ، فیصلہ آپ کا]');
  if (answer.next?.length) extra.push(`[اگلے سوال: ${answer.next.join(' | ')}]`);
  return [pieces.join('') || `[${reply.outcome}]`, ...extra].join(' ').replace(/\s+/g, ' ').trim();
}

function writeReview(list, results) {
  const head = [
    'id',
    'question',
    'outcome',
    'answer_as_shown',
    'good',
    'note',
    'advice_inside_fr_7_88',
    'cat',
  ];
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
  mkdirSync(dirname(REVIEW), { recursive: true });
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
  const qs = list.filter(
    (q) =>
      q.cat === cat &&
      verdicts.has(q.id) &&
      !verdicts.get(q.id).lost &&
      !verdicts.get(q.id).pending,
  );
  const vs = qs.map((q) => verdicts.get(q.id));
  const n = qs.length;
  const passed = vs.filter((v) => v.pass).length;
  const wrong = vs.filter((v) => v.kind.includes('REVIEW')).length;
  const misses = vs.filter((v) => !v.pass);
  const unsafe = misses.filter(
    (v) => !SAFE_ENDS.has(v.outcome) && !v.network && !v.kind.includes('REVIEW'),
  ).length;
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
  return {
    cat,
    n,
    passed,
    wrong,
    safe,
    unsafe,
    network,
    answeredRate,
    passRate,
    p50,
    p95,
    networkTimes,
    rulePassed,
    timeOk,
  };
}

const mean = (xs) => (xs.length ? sum(xs) / xs.length : 0);

/**
 * Plan 54: how many model calls a question needed and where its time went. Over the scored
 * questions of a group (a category, or the help questions): the calls (none, one, two, more), the
 * stages (before the first call, each call, between calls, after the last one, the app's own tools)
 * and the reasoning tokens of each call. The stage times need `calls[].at_ms` (the harness writes
 * it); a row without it is counted in the calls but left out of the stages.
 */
function speedStats(qs, verdicts, results) {
  const rows = qs
    .filter(
      (q) =>
        verdicts.has(q.id) &&
        !verdicts.get(q.id).lost &&
        !verdicts.get(q.id).pending &&
        !verdicts.get(q.id).network,
    )
    .map((q) => results.get(q.id))
    .filter(Boolean);
  const calls = (r) => r.model_calls ?? (r.calls ?? []).length;
  const by = { none: 0, one: 0, two: 0, more: 0 };
  for (const r of rows) {
    const n = calls(r);
    if (n === 0) by.none += 1;
    else if (n === 1) by.one += 1;
    else if (n === 2) by.two += 1;
    else by.more += 1;
  }
  const stage = {
    before: [],
    call1: [],
    call2: [],
    call3: [],
    between: [],
    after: [],
    tools: [],
    noCall: [],
  };
  const reasoning = { call1: [], call2: [], call3: [] };
  for (const r of rows) {
    const cs = r.calls ?? [];
    cs.forEach((c, i) => {
      const key = `call${Math.min(i + 1, 3)}`;
      (reasoning[key] ??= []).push(c.reasoning ?? 0);
      stage[key].push(c.millis ?? 0);
    });
    stage.tools.push(r.tool_millis ?? 0);
    if (cs.length === 0) stage.noCall.push(r.millis ?? 0);
    const timed = cs.length > 0 && cs.every((c) => typeof c.at_ms === 'number') && !r.clicked;
    if (!timed) continue;
    stage.before.push(cs[0].at_ms);
    for (let i = 0; i + 1 < cs.length; i += 1)
      stage.between.push(Math.max(0, cs[i + 1].at_ms - (cs[i].at_ms + cs[i].millis)));
    const last = cs[cs.length - 1];
    stage.after.push(Math.max(0, (r.millis ?? 0) - (last.at_ms + last.millis)));
  }
  const n = rows.length;
  return {
    n,
    by,
    oneOrNone: n ? (by.none + by.one) / n : 0,
    none: n ? by.none / n : 0,
    stage,
    reasoning,
  };
}

/**
 * The instant-answer figures of plan 55 Section 10 for a set of questions: how many were answered
 * right with no model call ("instant"), right in no call or one, and the wait of each group. A
 * question lost to the network, pending or ended on a network failure is left out.
 */
function instantStats(qs, verdicts, results) {
  const rows = qs
    .filter((q) => {
      const v = verdicts.get(q.id);
      return v && results.has(q.id) && !v.lost && !v.pending && !v.network;
    })
    .map((q) => ({ v: verdicts.get(q.id), r: results.get(q.id) }));
  const calls = (r) => r.model_calls ?? (r.calls ?? []).length;
  const wait = (r) => r.millis ?? 0;
  const n = rows.length;
  const noCall = rows.filter((x) => calls(x.r) === 0);
  const instant = rows.filter((x) => x.v.pass && calls(x.r) === 0);
  const oneOrNone = rows.filter((x) => x.v.pass && calls(x.r) <= 1);
  const all = rows.map((x) => wait(x.r));
  const times = (xs) => xs.map((x) => wait(x.r));
  const wrong = rows.filter((x) => !x.v.pass).length;
  const p50 = percentile(all, 0.5);
  const p95 = percentile(all, 0.95);
  const instantShare = n ? instant.length / n : 0;
  const oneShare = n ? oneOrNone.length / n : 0;
  return {
    n,
    noCall: noCall.length,
    wrong,
    instant: instant.length,
    instantShare,
    oneOrNone: oneOrNone.length,
    oneShare,
    instantTimes: times(instant),
    oneTimes: times(oneOrNone),
    p50,
    p95,
    ok: n === 0 || (instantShare >= 0.6 && oneShare >= 0.85 && p50 <= 2000 && p95 <= 8000),
  };
}

/** The lines of the instant-answer report for one group. */
function instantLines(name, st) {
  if (st.n === 0) return [`    ${name}: no question ran`];
  const pct = (x) => `${(100 * x).toFixed(1)}%`;
  const flag = (ok) => (ok ? 'OK' : 'UNDER');
  const lines = [
    `    ${name}: ${st.n} questions   no model call at all ${st.noCall} (${pct(st.noCall / st.n)}), of them right ${st.instant}`,
    `      instant (right, no model call): ${st.instant} (${pct(st.instantShare)}) ${flag(st.instantShare >= 0.6)} vs 60%   wait p50 / p95 ${p50p95(st.instantTimes)}`,
    `      instant or one call (right): ${st.oneOrNone} (${pct(st.oneShare)}) ${flag(st.oneShare >= 0.85)} vs 85%   wait p50 / p95 ${p50p95(st.oneTimes)}`,
    `      every question: wait p50 ${ms(st.p50)} ${flag(st.p50 <= 2000)} vs 2 s, p95 ${ms(st.p95)} ${flag(st.p95 <= 8000)} vs 8 s   not right: ${st.wrong}`,
  ];
  return lines;
}

/**
 * The speed budget of plan 55 Section 3, by kind of question, on the time the Owner waits (the whole
 * question, a network failure left out):
 *
 * - how-to, print, navigation, no such party, not in the app: p50 1 s, p95 2 s, hard limit 3 s;
 * - figures and lists (the rest of category A): 5 s, 10 s, hard limit 25 s;
 * - advice, forecasts and what-ifs (B and C): 10 s, 20 s, hard limit 25 s;
 * - mixed and hard (E, not named by the plan): the plan 52 time, 6 s and 12 s, hard limit 25 s.
 *
 * `ok` is true when every group is inside its budget and no question is over its hard limit.
 */
function speedBudget(list, verdicts, results) {
  const local = (q) =>
    q.kind === 'help' ||
    q.kind === 'print' ||
    q.outcome === 'not_found' ||
    q.outcome === 'outside_books';
  const groups = [
    {
      name: 'how-to, print, navigation, no such party, not in the app',
      pick: local,
      p50: 1000,
      p95: 2000,
      hard: 3000,
    },
    {
      name: 'figures and lists (A)',
      pick: (q) => q.cat === 'A' && !local(q),
      p50: 5000,
      p95: 10000,
      hard: 25000,
    },
    {
      name: 'advice, forecasts, what-ifs (B, C)',
      pick: (q) => (q.cat === 'B' || q.cat === 'C') && !local(q),
      p50: 10000,
      p95: 20000,
      hard: 25000,
    },
    {
      name: 'mixed and hard (E)',
      pick: (q) => q.cat === 'E' && !local(q),
      p50: 6000,
      p95: 12000,
      hard: 25000,
    },
  ];
  const out = groups.map((g) => {
    const qs = list.filter(
      (q) =>
        g.pick(q) &&
        results.has(q.id) &&
        verdicts.has(q.id) &&
        !verdicts.get(q.id).lost &&
        !verdicts.get(q.id).pending &&
        !verdicts.get(q.id).network,
    );
    const times = qs.map((q) => results.get(q.id).millis ?? 0);
    const p50 = percentile(times, 0.5);
    const p95 = percentile(times, 0.95);
    const over = qs.filter((q) => (results.get(q.id).millis ?? 0) > g.hard).map((q) => q.id);
    return {
      ...g,
      n: qs.length,
      p50v: p50,
      p95v: p95,
      over,
      within: qs.length === 0 || (p50 <= g.p50 && p95 <= g.p95),
    };
  });
  return { groups: out, ok: out.every((g) => g.within && g.over.length === 0) };
}

/** `p50 / p95` of a list of milliseconds, or a dash. */
const p50p95 = (xs) =>
  xs.length ? `${ms(percentile(xs, 0.5))} / ${ms(percentile(xs, 0.95))}` : '-';

/** The lines of the speed report for one group of questions. */
function speedLines(label, st, target) {
  if (st.n === 0) return [];
  const out = [];
  const { none, one, two, more } = st.by;
  const share = `${(100 * st.oneOrNone).toFixed(1)}%`;
  const flag =
    target === undefined
      ? ''
      : st.oneOrNone >= target
        ? `  OK (target ${100 * target}%)`
        : `  UNDER (target ${100 * target}%)`;
  out.push(
    `  ${label}  ${st.n} questions   model calls 0 / 1 / 2 / 3+: ${none} / ${one} / ${two} / ${more}   one call or none ${share}${flag}`,
  );
  out.push(
    `      time p50 / p95:  before call 1 ${p50p95(st.stage.before)}   call 1 ${p50p95(st.stage.call1)}   call 2 ${p50p95(st.stage.call2)}   call 3+ ${p50p95(st.stage.call3)}   between calls ${p50p95(st.stage.between)}   after last call ${p50p95(st.stage.after)}   app tools ${p50p95(st.stage.tools)}   no model call ${p50p95(st.stage.noCall)}`,
  );
  const r = (xs) =>
    xs.length
      ? `p50 ${Math.round(percentile(xs, 0.5))}, p95 ${Math.round(percentile(xs, 0.95))}, mean ${Math.round(mean(xs))}`
      : '-';
  out.push(
    `      reasoning tokens a call:  call 1 ${r(st.reasoning.call1)}   call 2 ${r(st.reasoning.call2)}   call 3+ ${r(st.reasoning.call3)}`,
  );
  return out;
}

/**
 * How many results the app read for a question BEFORE its first model call (the guide of a how-to,
 * a period, the tiles). Such a result is in the log's tool list with `prefetch` (or, for the guide,
 * with an id `h1`), and in no call's own tool list. A card the app reads right after the model's
 * find is in the call's list with `prefetch` too, and is not counted here.
 */
function preReads(r) {
  let calls;
  try {
    calls = typeof r.calls_json === 'string' ? JSON.parse(r.calls_json) : (r.calls_json ?? []);
  } catch {
    return 0;
  }
  const tools = calls.flatMap((c) => c.tools ?? []);
  const flagged = (r.tool_calls ?? []).filter((t) => t.prefetch === true).length;
  const flaggedInCalls = tools.filter((t) => t.prefetch === true).length;
  const guides = (r.tool_calls ?? []).filter(
    (t) => typeof t.id === 'string' && t.id.startsWith('h'),
  ).length;
  const guidesInCalls = tools.filter((t) => t.name === 'app_help').length;
  return Math.max(0, flagged - flaggedInCalls) + Math.max(0, guides - guidesInCalls);
}

function report(list, data) {
  const oracle = data.oracle ?? {};
  const results = new Map(data.results.map((r) => [r.id, r]));
  const ran = list.filter((q) => results.has(q.id));
  const verdicts = new Map(
    ran.map((q) => [q.id, score(q, results.get(q.id), oracle, data.meta?.secrets ?? null)]),
  );
  const vlist = ran.map((q) => verdicts.get(q.id));
  const lostV = vlist.filter((v) => v.lost);
  const pendingV = vlist.filter((v) => v.pending);
  const scoredV = vlist.filter((v) => !v.lost && !v.pending);
  const passed = scoredV.filter((v) => v.pass).length;
  const lines = [];
  const say = (s = '') => lines.push(s);
  say(`AI EVAL  model ${data.meta.model}`);
  say(`endpoint ${data.meta.endpoint}`);
  say(
    `date ${new Date().toISOString().slice(0, 10)}   questions run ${ran.length} of ${list.length}   threads ${data.meta.threads ?? 1}${BANK ? '   bank ' + basename(BANK) : ''}`,
  );
  say(
    `oracle facts ${Object.keys(oracle).length}   prefind ${data.meta.prefind === undefined ? 'not recorded' : data.meta.prefind ? 'ON' : 'off (as the app)'}`,
  );
  say();
  say(
    `SCORE ${passed} of ${scoredV.length} scored${lostV.length ? `   (${lostV.length} lost to the network are NOT scored: they are listed below and must be asked again)` : ''}${pendingV.length ? `   (${pendingV.length} PENDING, not scored: the reply lacks a field of the contract, see below)` : ''}`,
  );
  say();
  say(
    'CATEGORY                 pass      wrong  answered  model time p50 / p95 (target)           network  rule',
  );
  const cats = Object.keys(CATS).filter((c) => ran.some((q) => q.cat === c));
  const stats = cats.map((c) => categoryStats(c, list, verdicts, results));
  for (const s of stats) {
    const t = CATS[s.cat];
    say(
      `${s.cat} ${t.title.padEnd(22)} ${`${s.passed}/${s.n}`.padEnd(8)} ${String(s.wrong).padEnd(6)} ${`${(100 * s.answeredRate).toFixed(1)}%`.padEnd(9)} ${ms(s.p50)} / ${ms(s.p95)} (${ms(t.p50)} / ${ms(t.p95)}) ${s.timeOk ? 'OK ' : 'OVER'}   ${String(s.network).padEnd(7)} ${s.rulePassed ? 'MEETS' : 'NOT YET'}  [${t.rule}]`,
    );
  }
  // Plan 54: model calls and the time of each stage, by category and for the help and print questions.
  say();
  say(
    "SPEED (plan 54): how many model calls a question needed, and where its time went (model time apart from the app's own work)",
  );
  const speed = {};
  for (const c of cats) {
    const st = speedStats(
      list.filter((q) => q.cat === c),
      verdicts,
      results,
    );
    speed[c] = st;
    for (const line of speedLines(c, st, c === 'A' ? 0.7 : undefined)) say(line);
  }
  const kindOf = (k) => list.filter((q) => q.kind === k);
  for (const [k, label] of [
    ['help', 'help (how-to)'],
    ['print', 'print'],
  ]) {
    const st = speedStats(kindOf(k), verdicts, results);
    speed[k] = st;
    const lines = speedLines(label, st);
    if (lines.length) {
      for (const line of lines) say(line);
      say(
        `      answered with no model call: ${st.by.none} of ${st.n} (${(100 * st.none).toFixed(1)}%)`,
      );
    }
  }
  // Plan 55 gate 3: the buttons of the answers, each opened with its own filters.
  const withButtons = ran.filter(
    (q) => (results.get(q.id)?.buttons ?? []).length > 0 && !verdicts.get(q.id)?.lost,
  );
  if (ran.some((q) => Array.isArray(results.get(q.id)?.buttons))) {
    const onlyTiles = withButtons.filter((q) => results.get(q.id).buttons.every((b) => b.skipped));
    const wrong = withButtons.filter((q) =>
      (verdicts.get(q.id)?.reasons ?? []).some((x) => x.startsWith('button')),
    );
    say(
      `  BUTTONS (plan 55 gate 3): ${withButtons.length - onlyTiles.length} answers with a report or view button were opened with the button's filters and their figures looked for in it; ${onlyTiles.length} answers have only the dashboard tiles (not checked); wrong place or wrong filters: ${wrong.length === 0 ? 'none' : wrong.map((q) => q.id).join(' ')}`,
    );
  }
  // Plan 55 Section 3: the speed budget by kind and the hard limits. They are part of "meets".
  const budget = speedBudget(list, verdicts, results);
  say(
    '  SPEED BUDGET (plan 55 Section 3), the time the Owner waits, p50 / p95 against the budget, and the hard limit:',
  );
  for (const g of budget.groups) {
    if (g.n === 0) continue;
    say(
      `    ${g.name}: ${g.n} questions   ${ms(g.p50v)} / ${ms(g.p95v)} (budget ${ms(g.p50)} / ${ms(g.p95)}) ${g.within ? 'OK' : 'OVER'}   hard limit ${ms(g.hard)}: ${g.over.length === 0 ? 'none over' : `${g.over.length} OVER: ${g.over.join(' ')}`}`,
    );
  }
  const oneCallOk = !speed.A || speed.A.n === 0 || speed.A.oneOrNone >= 0.7;
  // Plan 55 Section 10: real-time figures answered instantly (no model call) and right. Shown for
  // facet A2 (the 120 simple figures; gated) and for the whole of category A (shown, not gated: it
  // also holds comparisons, two-part questions and follow-ups that need the model).
  const facetOfQ = (q) => (q.filters.match(/(?:^|;)facet=([A-Za-z0-9]+)/) ?? [])[1];
  const instant = {
    A2: instantStats(
      list.filter((q) => facetOfQ(q) === 'A2'),
      verdicts,
      results,
    ),
    A: instantStats(
      list.filter((q) => q.cat === 'A'),
      verdicts,
      results,
    ),
  };
  const a2Ok = instant.A2.n === 0 || instant.A2.ok;
  say();
  say(
    '  REAL-TIME FIGURES (plan 55 Section 10): a right answer with no model call ("instant"), and in no call or one; the wait of each. Target for A2: instant at least 60%, instant or one call at least 85%, p50 2 s, p95 8 s:',
  );
  for (const [name, st] of [
    ['A2 (the 120 simple figures)', instant.A2],
    ['A  (all of category A, not gated)', instant.A],
  ])
    for (const line of instantLines(name, st)) say(line);
  const over25 = budget.groups.flatMap((g) => (g.hard === 25000 ? g.over : []));
  const slowLocal = budget.groups.flatMap((g) => (g.hard === 3000 ? g.over : []));
  say(
    `  HARD LIMITS: no question over 25 s: ${over25.length === 0 ? 'ok' : `${over25.length} over: ${over25.join(' ')}`}   how-to, print, navigation under 3 s: ${slowLocal.length === 0 ? 'ok' : `${slowLocal.length} over: ${slowLocal.join(' ')}`}   category A in one call or none, at least 70%: ${speed.A && speed.A.n ? `${(100 * speed.A.oneOrNone).toFixed(1)}% ${oneCallOk ? 'ok' : 'UNDER'}` : 'not run'}`,
  );
  const netTimes = stats.flatMap((s) => s.networkTimes);
  say(
    `  (time is the model's: ${netTimes.length} questions that ended on a network failure or a timeout are counted apart, their time ${netTimes.length ? `p95 ${ms(percentile(netTimes, 0.95))}` : 'none'})`,
  );
  say();
  const misses = scoredV.filter((v) => !v.pass);
  const review = misses.filter((v) => v.kind.includes('REVIEW'));
  const count = (word) => misses.filter((v) => v.kind.includes(word)).length;
  say(
    `MISSES ${misses.length}: wrong or missing figures to review ${review.length}, safe ends ${count('safe')}, network ${count('network')}, harness ${count('harness')}, oracle ${count('oracle')}, other ${misses.length - review.length - count('safe') - count('network') - count('harness') - count('oracle')}`,
  );
  for (const v of misses) {
    const q = list.find((x) => x.id === v.id);
    say(`  ${v.id} [${q.cat}] [${v.kind}] ${q.question}`);
    say(`      expected ${q.outcome}: ${q.figures || q.filters}`);
    say(`      got ${v.outcome}${v.failure ? ` (${v.failure})` : ''}; ${v.reasons.join('; ')}`);
  }
  if (lostV.length) {
    say();
    say(
      `LOST TO THE NETWORK: ${lostV.length} questions, none of them scored: ${lostV.map((v) => v.id).join(' ')}`,
    );
    say('  ask them again with: node scripts/ai-eval/run.mjs --misses');
  }
  if (pendingV.length) {
    say();
    say(
      `PENDING (a field of the contract is not in the reply yet; neither a pass nor a miss): ${pendingV.length} questions`,
    );
    const pendingBy = (word) => pendingV.filter((v) => v.kind.includes(word)).map((v) => v.id);
    if (pendingBy('asked_role').length)
      say(`  asked_role / found_as: ${pendingBy('asked_role').join(' ')}`);
    if (pendingBy('print').length)
      say(`  print steps (plan 54 S2): ${pendingBy('print').join(' ')}`);
    if (pendingBy('no route yet').length)
      say(`  no route yet (a gap of capability.tsv): ${pendingBy('no route yet').join(' ')}`);
    if (pendingBy('guide topic').length)
      say(`  guide topic of a local answer (the log entry): ${pendingBy('guide topic').join(' ')}`);
  }
  const noted = vlist.filter((v) => v.notes.length > 0);
  say();
  say(
    `notes (not misses): ${noted.length} questions used another tool route, said a zero in words, or showed no name`,
  );
  for (const v of noted.slice(0, 40)) say(`  ${v.id}: ${v.notes.join('; ')}`);
  // No "not understood" after the model has read data; no network failure.
  const afterReading = vlist.filter((v) => v.afterReading);
  say();
  say(
    `NOT UNDERSTOOD AFTER READING: ${afterReading.length} ${afterReading.length ? afterReading.map((v) => v.id).join(' ') : '(none, as plan 52 wants)'}`,
  );
  const clicked = ran.map((q) => results.get(q.id)).filter((r) => r.clicked);
  if (clicked.length) {
    say(
      `CLICKED  ${clicked.length} questions waited for a click (a misspelled name asked "which one", or a name found in another role) and the matching button was clicked, as on the panel (both times counted): ${clicked.map((r) => `${r.id} ${r.clicked.name}`).join(', ')}`,
    );
  }
  const failures = vlist.filter((v) => v.network);
  const byKind = {};
  for (const v of failures)
    byKind[v.failure ?? v.outcome] = (byKind[v.failure ?? v.outcome] ?? 0) + 1;
  say(
    `NETWORK FAILURES: ${failures.length} ${failures.length ? JSON.stringify(byKind) : '(none, as plan 52 wants)'}`,
  );
  // Groups of the same meaning.
  const bad = groupAgreement(list, verdicts, results);
  say();
  say(
    `GROUPS of the same question (required figures only): ${bad.length === 0 ? 'every group reaches the same figures' : `${bad.length} DISAGREE`}`,
  );
  for (const g of bad) say(`  ${g.group} ${g.members.join(' ')}: ${g.signs.join('  |  ')}`);
  // Tools, model calls, tokens.
  const all = ran.map((q) => results.get(q.id));
  const toolCalls = all.map((r) => (r.tool_calls ?? []).filter((c) => !c.error).length);
  const modelCalls = all.map((r) => r.model_calls ?? 0);
  const t = (k) => sum(all.map((r) => r.tokens?.[k] ?? 0));
  const parallel = all.filter((r) => (r.calls ?? []).some((c) => c.tool_calls >= 2));
  const badParallel = sum(parallel.map((r) => (r.tool_calls ?? []).filter((c) => c.error).length));
  say();
  say(
    `TOOL CALLS ${sum(toolCalls)} (${(sum(toolCalls) / Math.max(1, all.length)).toFixed(1)} a question)   MODEL CALLS ${sum(modelCalls)} (${(sum(modelCalls) / Math.max(1, all.length)).toFixed(1)} a question)`,
  );
  say(
    `TOKENS  input ${t('in')}  cached ${t('cached')}  output ${t('out')}  reasoning ${t('reasoning')}`,
  );
  say(
    `  per question: input ${Math.round(t('in') / Math.max(1, all.length))}, cached ${Math.round(t('cached') / Math.max(1, all.length))}, output ${Math.round(t('out') / Math.max(1, all.length))}`,
  );
  // The fixed part as the model counts it (plan 52 item 0c): the prompt tokens of the FIRST call of a
  // fresh question, less the question itself, for the questions whose first call carried nothing the
  // app had read for it. What the app reads before the first call (the guide of a how-to, a period,
  // the tiles) is added to that call but is not fixed: it is reported apart.
  const firstCalls = all.filter(
    (r) => !r.followup && typeof r.calls?.[0]?.prompt === 'number' && r.calls[0].prompt > 0,
  );
  const readAhead = (r) => preReads(r);
  const plain = firstCalls
    .filter((r) => readAhead(r) === 0)
    .map((r) => r.calls[0].prompt - (r.user_tokens ?? 0));
  const fixed = plain.length
    ? plain
    : firstCalls.map((r) => r.calls[0].prompt - (r.user_tokens ?? 0));
  const fixedMax = fixed.length ? Math.max(...fixed) : 0;
  const fixedO200k = Math.round(fixedMax / MODEL_COUNTS_MORE);
  say(
    `  FIXED PART (system, tools, answer schema), first call of ${fixed.length} fresh questions with nothing read ahead, as the model counts it: min ${fixed.length ? Math.min(...fixed) : 0}, p50 ${percentile(fixed, 0.5)}, max ${fixedMax} ${fixedMax <= PREFIX_MODEL ? 'OK' : 'OVER'} (limit about ${PREFIX_MODEL}); about ${fixedO200k} by o200k ${fixedO200k <= PREFIX_O200K ? 'OK' : 'OVER'} (D6-A19, NFR v1.13: at most ${PREFIX_O200K})`,
  );
  const base = percentile(fixed, 0.5);
  const extra = firstCalls
    .filter((r) => readAhead(r) > 0)
    .map((r) => r.calls[0].prompt - (r.user_tokens ?? 0) - base);
  if (extra.length) {
    say(
      `  READ AHEAD by the app before the first call (NOT fixed: per question): ${extra.length} questions, +${Math.min(...extra)} to +${Math.max(...extra)} tokens, p50 +${percentile(extra, 0.5)}`,
    );
  }
  say(
    `PARALLEL tool calls that arrived with a bad input: ${badParallel} (in ${parallel.length} questions that asked for tools together)`,
  );
  const limited = all.filter((r) => r.reply?.outcome === 'not_understood' && r.reply?.stop_reason);
  say(`LIMITS  ${limited.length} questions stopped at a limit (calls or tokens)`);
  const oneCall = list.filter((q) => q.filters.includes('onecall=1') && results.has(q.id));
  if (oneCall.length) {
    const one = oneCall.filter((q) => (results.get(q.id).model_calls ?? 0) <= 1).length;
    say(
      `ONE CALL  ${one} of ${oneCall.length} help or general advice questions needed one model call`,
    );
  }
  const callTimes = all.flatMap((r) => (r.calls ?? []).map((c) => c.millis));
  say(
    `TIME  total ${ms(sum(all.map((r) => r.millis ?? 0)))}   one model call: p50 ${ms(percentile(callTimes, 0.5))}, p95 ${ms(percentile(callTimes, 0.95))}, max ${ms(callTimes.length ? Math.max(...callTimes) : 0)}   a follow-up counts its own time only`,
  );
  say();
  say(
    `URDU REVIEW SHEET ${REVIEW} (${ran.length} answers; the reviewer marks good or not, and for advice answers inside FR-7.88 or not; at least 95 of 100 advice answers must be good)`,
  );
  const advice = list.filter((q) => q.cat === 'B' && results.has(q.id));
  say(`  advice answers to read for FR-7.88: ${advice.map((q) => q.id).join(' ')}`);
  const meets =
    stats.length > 0 &&
    stats.every((s) => s.rulePassed && s.timeOk) &&
    bad.length === 0 &&
    afterReading.length === 0 &&
    failures.length === 0 &&
    lostV.length === 0 &&
    pendingV.length === 0 &&
    budget.ok &&
    oneCallOk &&
    a2Ok;
  say();
  say(
    meets
      ? 'RESULT: every category that ran meets plan 52 Section 2 and the speed of plan 55 Section 3 (the Urdu review is still to do)'
      : `RESULT: does NOT yet meet plan 52 Section 2 and plan 55 Section 3 (${
          stats
            .filter((s) => !s.rulePassed)
            .map((s) => `${s.cat} score`)
            .concat(stats.filter((s) => !s.timeOk).map((s) => `${s.cat} time`))
            .join(', ') || 'groups, reads or network'
        }${lostV.length ? '; and questions lost to the network' : ''}${pendingV.length ? '; and questions pending a field of the contract' : ''}${budget.ok ? '' : '; and the speed budget or a hard limit'}${oneCallOk ? '' : '; and category A under 70% in one call'}${a2Ok ? '' : '; and the real-time figures (A2) under their instant, one-call or wait target'})`,
  );
  return { text: lines.join('\n'), meets, results, verdicts, stats, bad, speed, budget, instant };
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
    out.push(
      `  got       ${v.outcome}${v.failure ? ` (${v.failure})` : ''}; ${v.reasons.join('; ')}`,
    );
    if (r.first_reply)
      out.push(
        `  first     ${r.first_reply.outcome}; asked_role ${r.first_reply.asked_role ?? '-'}; found_as ${roleOf(r.first_reply.found_as) ?? '-'} ${roleName(r.first_reply.found_as) ?? ''} (the answer below is after the click)`,
      );
    out.push(`  answer    ${answerAsShown(r.reply)}`);
    out.push(
      `  tools     ${(r.tool_calls ?? []).map((c) => `${c.tool}${c.error ? '!' : ''}(${JSON.stringify(c.input ?? {}).slice(0, 110)})`).join('  ')}`,
    );
    out.push(
      `  calls     ${(r.calls ?? []).map((c) => `${c.millis} ms ${c.finish} +${c.tool_calls}t`).join(' | ')}   ${r.millis} ms, ${r.model_calls} calls, finish ${r.finish_reason ?? '-'}`,
    );
    if (r.calls_json) out.push(`  calls_json ${String(r.calls_json).slice(0, 600)}`);
    if (r.error) out.push(`  harness   ${r.error}`);
    out.push('');
  }
  mkdirSync(dirname(DETAIL), { recursive: true });
  writeFileSync(DETAIL, out.join('\n') + '\n', 'utf8');
}

// ---- running -------------------------------------------------------------------------------

/** The ids a run asks about, from --only, --category, --world, --facet and --every. Empty: all of them. */
function selectIds(list, { only, category, world, facet, every }) {
  const wanted = new Set(
    (only ?? '')
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean),
  );
  const cats = new Set(
    (category ?? '')
      .split(',')
      .map((s) => s.trim().toUpperCase())
      .filter(Boolean),
  );
  const worlds = new Set(
    (world ?? '')
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean),
  );
  const facets = new Set(
    (facet ?? '')
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean),
  );
  const facetOfQ = (q) => (q.filters.match(/(?:^|;)facet=([A-Za-z0-9]+)/) ?? [])[1];
  const stride = Math.max(1, Number.parseInt(every ?? '1', 10) || 1);
  if (
    wanted.size === 0 &&
    cats.size === 0 &&
    worlds.size === 0 &&
    facets.size === 0 &&
    stride === 1
  )
    return [];
  // `--every 5` keeps every fifth question of the selection (the first, the sixth ...): a cheap sample of
  // a big facet such as ITEM that still touches every measure, subject and period in turn.
  return list
    .filter(
      (q) =>
        (wanted.size === 0 || wanted.has(q.id)) &&
        (cats.size === 0 || cats.has(q.cat)) &&
        (worlds.size === 0 || worlds.has(q.world)) &&
        (facets.size === 0 || facets.has(facetOfQ(q))),
    )
    .filter((_, i) => i % stride === 0)
    .map((q) => q.id);
}

/** Runs the ignored live test through cargo. The key is not passed: the app reads its own store. */
function runLive(ids, model, threads, outFile = RESULTS) {
  mkdirSync(OUT_DIR, { recursive: true });
  const env = { ...process.env, MANDI_EVAL_QUESTIONS: QUESTIONS, MANDI_EVAL_OUT: outFile };
  if (ids.length) env.MANDI_EVAL_ONLY = ids.join(',');
  if (model) env.MANDI_EVAL_MODEL = model;
  if (threads) env.MANDI_EVAL_THREADS = String(threads);
  const cmd = [
    'test',
    '-j',
    '4',
    '-p',
    'mandi-assistant',
    '--test',
    'robustness_live',
    '--',
    '--ignored',
    '--nocapture',
    'live_100_questions',
  ];
  console.log(`ai-eval: cargo ${cmd.join(' ')} in ${BACKEND}`);
  const res = spawnSync('cargo', cmd, {
    cwd: BACKEND,
    env,
    stdio: ['ignore', 'inherit', 'inherit'],
  });
  if (res.error) throw new Error(`cannot start cargo: ${res.error.message}`);
  if (res.status !== 0) throw new Error(`the live test failed with exit code ${res.status}`);
}

/** The ids to ask again: every question that missed, was lost to the network, or was never asked. */
function idsToAskAgain(list) {
  if (!existsSync(RESULTS)) throw new Error(`--misses needs a results file at ${RESULTS}`);
  const data = JSON.parse(readFileSync(RESULTS, 'utf8'));
  if (data.waiting_for_key) throw new Error('the results file only says WAITING FOR KEY');
  const oracle = data.oracle ?? {};
  const results = new Map(data.results.map((r) => [r.id, r]));
  return list
    .filter((q) => {
      const r = results.get(q.id);
      if (!r) return true;
      const v = score(q, r, oracle, data.meta?.secrets ?? null);
      return v.lost || v.pending || !v.pass;
    })
    .map((q) => q.id);
}

/**
 * Puts the rows of a partial run (only some ids) into the whole results file, a new row for an old
 * one of the same id. The file before the merge is kept as results.before-merge.json.
 */
function mergeResults(part, whole = RESULTS) {
  const old = existsSync(whole)
    ? JSON.parse(readFileSync(whole, 'utf8'))
    : { meta: {}, oracle: {}, results: [] };
  if (old.results)
    writeFileSync(join(dirname(whole), 'results.before-merge.json'), JSON.stringify(old));
  const byId = new Map((old.results ?? []).map((r) => [r.id, r]));
  for (const r of part.results ?? []) byId.set(r.id, r);
  const merged = {
    meta: { ...(old.meta ?? {}), ...(part.meta ?? {}), questions: byId.size, merged: true },
    oracle: { ...(old.oracle ?? {}), ...(part.oracle ?? {}) },
    results: [...byId.values()],
  };
  writeFileSync(whole, JSON.stringify(merged));
  return merged;
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
  const dr = (n) => ({
    type: 'figure',
    cell: { type: 'balance', paisa: n * 100, side: 'dr' },
    dir: null,
    checked: true,
  });
  const money = (n) => ({
    type: 'figure',
    cell: { type: 'money', paisa: n * 100 },
    dir: null,
    checked: true,
  });
  const name = (en, ur) => ({
    type: 'name',
    name: { name_en: en, name_ur: ur, tag: null, code: null },
  });
  const text = (t) => ({ type: 'text', text: t });
  const answered = (parts, extra = {}, answerExtra = {}) => ({
    outcome: 'answered',
    answer: {
      parts,
      tables: [],
      charts: [],
      estimate: null,
      advice: false,
      sources: [],
      differences: [],
      next: [],
      ...answerExtra,
    },
    ...extra,
  });
  const byId = (id) => list.find((q) => q.id === id);

  // The old cases still hold.
  const q1 = byId('Q001');
  expect(
    score(
      q1,
      {
        reply: answered([name('Akram Toll Tax', 'اکرم ٹول ٹیکس'), text(' کے ذمے '), dr(2880)]),
        tool_calls: [{ tool: 'find' }, { tool: 'party_card' }],
      },
      {},
    ).pass,
    'a right answer missed',
  );
  const wrong = score(q1, { reply: answered([dr(2800)]), tool_calls: [] }, {});
  expect(!wrong.pass && wrong.kind.includes('REVIEW'), 'a wrong figure was not flagged for review');
  const ask = score(q1, { reply: { outcome: 'which_one', answer: null }, tool_calls: [] }, {});
  expect(!ask.pass && ask.kind.includes('safe'), 'which_one was not a safe end');
  const q96 = byId('Q096');
  expect(
    score(
      q96,
      {
        reply: { outcome: 'change_refused', answer: null, entry_route: { screen: 'wasooli' } },
        tool_calls: [],
      },
      {},
    ).pass,
    'a refusal missed',
  );
  const sheet = answerAsShown(
    answered([name('Akram Toll Tax', 'اکرم ٹول ٹیکس'), text(' کے ذمے '), dr(2880)]),
  );
  expect(sheet.includes('2,880 بنام ✓'), `the review text is "${sheet}"`);

  // Figures: forms, alternatives, optional, strict names.
  const e = expectedFigures(
    'same_pace=288000;pace_month=2027-04;share=72.10%;sum=21564 Dr;opt?=5;top!=Banana|Apple;x=1|2',
  );
  expect(
    e.length === 7 &&
      e[1].alts[0].kind === 'month' &&
      e[2].alts[0].n === 7210 &&
      e[3].alts[0].side === 'dr',
    'expected figures were read wrongly',
  );
  expect(
    e[4].optional && e[5].strict && e[5].alts.length === 2 && e[6].alts.length === 2,
    'optional, strict or alternative forms were read wrongly',
  );
  const oracle = { 'a.x': '500', 'a.y': '700', 'n.top': 'Banana' };
  expect(resolveFigures('p=@a.x;q=@a.y', oracle) === 'p=500;q=700', 'the oracle was not filled in');
  let threw = false;
  try {
    resolveFigures('p=@a.nope', oracle);
  } catch {
    threw = true;
  }
  expect(threw, 'a missing oracle fact did not stop the scorer');
  const mk = (over) => ({
    id: 'T1',
    cat: 'A',
    kind: 'x',
    outcome: 'answered',
    tools: '',
    filters: '',
    figures: '',
    ...over,
  });
  // Either of two honest readings passes; neither fails.
  const alt = mk({ figures: 'v=@a.x|@a.y' });
  expect(
    score(alt, { reply: answered([money(700)]), tool_calls: [] }, oracle).pass,
    'the second reading was not accepted',
  );
  expect(
    !score(alt, { reply: answered([money(600)]), tool_calls: [] }, oracle).pass,
    'a wrong figure passed as a reading',
  );
  // An optional figure may be missing; a required one may not.
  expect(
    score(
      mk({ figures: 'g=@a.x;c?=@a.y' }),
      { reply: answered([money(500)]), tool_calls: [] },
      oracle,
    ).pass,
    'an optional figure was required',
  );
  expect(
    !score(
      mk({ figures: 'g=@a.x;c=@a.y' }),
      { reply: answered([money(500)]), tool_calls: [] },
      oracle,
    ).pass,
    'a required figure was optional',
  );
  // A strict name must be named.
  const top = mk({ figures: 'top!=@n.top' });
  expect(
    score(top, { reply: answered([name('Banana', 'کیلا')]), tool_calls: [] }, oracle).pass,
    'a strict name missed',
  );
  expect(
    !score(top, { reply: answered([name('Apple', 'سیب')]), tool_calls: [] }, oracle).pass,
    'a wrong strict name passed',
  );
  // A zero said in words passes only with the route matched and no other figure; a wrong one never.
  const zero = mk({ figures: 'gross=0', tools: 'tiles' });
  expect(
    score(
      zero,
      { reply: answered([text('آج کوئی بکری نہیں')]), tool_calls: [{ tool: 'tiles' }] },
      {},
    ).pass,
    'a zero said in words missed',
  );
  expect(
    !score(
      zero,
      { reply: answered([text('آج کوئی بکری نہیں')]), tool_calls: [{ tool: 'report' }] },
      {},
    ).pass,
    'a zero passed with another route',
  );
  expect(
    !score(zero, { reply: answered([money(1200)]), tool_calls: [{ tool: 'tiles' }] }, {}).pass,
    'a zero passed beside another figure',
  );
  const zeroNote = mk({ figures: 'gross=100' });
  expect(
    !score(zeroNote, { reply: answered([text('کوئی بکری نہیں')]), tool_calls: [] }, {}).pass,
    'a words-only answer passed for a figure that is not 0',
  );
  // Months belong to their method.
  const est = (kind, m) => ({
    outcome: 'estimate',
    answer: {
      parts: [],
      tables: [],
      charts: [],
      advice: false,
      sources: [],
      differences: [],
      next: [],
      estimate: {
        methods: [
          { kind, total: { type: 'money', paisa: 100 }, target_month: { year: 2027, month: m } },
        ],
      },
    },
  });
  const pm = mk({ outcome: 'estimate', figures: 'pace_month=2027-02' });
  expect(
    score(pm, { reply: est('same_pace', 2), tool_calls: [] }, {}).pass,
    'the pace month missed',
  );
  expect(
    !score(pm, { reply: est('same_growth', 2), tool_calls: [] }, {}).pass,
    'the growth month passed for the pace month',
  );
  // General advice: marked, with no figure; with data, from the books and not marked.
  const adviceReply = (general, parts, sources = []) => ({
    outcome: 'advice',
    answer: {
      parts,
      tables: [],
      charts: [],
      estimate: null,
      advice: true,
      general_advice: general,
      sources,
      differences: [],
      next: [],
    },
  });
  const gen = mk({ outcome: 'advice', filters: 'general=1;onecall=1' });
  expect(
    score(
      gen,
      { reply: adviceReply(true, [text('قیمت مناسب رکھیں')]), tool_calls: [], model_calls: 1 },
      {},
    ).pass,
    'general advice missed',
  );
  expect(
    !score(
      gen,
      { reply: adviceReply(true, [text('قیمت'), money(5)]), tool_calls: [], model_calls: 1 },
      {},
    ).pass,
    'general advice with a figure passed',
  );
  expect(
    !score(
      gen,
      {
        reply: adviceReply(false, [text('قیمت')], [{ kind: 'report' }]),
        tool_calls: [],
        model_calls: 1,
      },
      {},
    ).pass,
    'unmarked advice passed as general',
  );
  const books = mk({ outcome: 'advice', filters: 'general=0' });
  expect(
    score(
      books,
      { reply: adviceReply(false, [text('قیمت')], [{ kind: 'report' }]), tool_calls: [] },
      {},
    ).pass,
    'advice from the books missed',
  );
  expect(
    !score(books, { reply: adviceReply(true, [text('قیمت')]), tool_calls: [] }, {}).pass,
    'general advice passed where the books have data',
  );
  // One side of the commission passes only when the answer names the side (lead 03-11-2026).
  const side = mk({ figures: 'c=@a.x|@a.y#seller' });
  expect(
    score(
      side,
      { reply: answered([text('بیوپاری کا کمیشن '), money(700)]), tool_calls: [] },
      oracle,
    ).pass,
    'the seller side named was not accepted',
  );
  expect(
    !score(side, { reply: answered([text('کمیشن '), money(700)]), tool_calls: [] }, oracle).pass,
    'a bare commission passed for the seller side',
  );
  expect(
    score(side, { reply: answered([text('کمیشن '), money(500)]), tool_calls: [] }, oracle).pass,
    'the total was refused',
  );
  // A which_one that the harness clicked is noted, and its answer is scored like any other.
  const clickedRow = score(
    mk({ figures: 'g=@a.x' }),
    { reply: answered([money(500)]), tool_calls: [], clicked: { name: 'Rahim' } },
    oracle,
  );
  expect(
    clickedRow.pass && clickedRow.notes.some((n) => n.includes('clicked')),
    'a clicked question was not noted',
  );
  // "Whom first": the second party is optional (`second?=`); when both are shown the order counts.
  const order = mk({
    outcome: 'advice',
    figures: 'first=Zahid Mehmood 500 Dr 202 days;second?=Tariq Mehmood 700 Dr 90 days',
  });
  const adviceOf = (parts) => ({
    outcome: 'advice',
    answer: {
      parts,
      tables: [],
      charts: [],
      estimate: null,
      advice: true,
      general_advice: false,
      sources: [{ kind: 'report' }],
      differences: [],
      next: [],
    },
  });
  expect(
    score(order, { reply: adviceOf([name('Zahid Mehmood', 'زاہد'), dr(500)]), tool_calls: [] }, {})
      .pass,
    'the first alone was refused',
  );
  expect(
    score(
      order,
      {
        reply: adviceOf([
          name('Zahid Mehmood', 'زاہد'),
          dr(500),
          name('Tariq Mehmood', 'طارق'),
          dr(700),
        ]),
        tool_calls: [],
      },
      {},
    ).pass,
    'both in order were refused',
  );
  expect(
    !score(
      order,
      {
        reply: adviceOf([
          name('Tariq Mehmood', 'طارق'),
          dr(700),
          name('Zahid Mehmood', 'زاہد'),
          dr(500),
        ]),
        tool_calls: [],
      },
      {},
    ).pass,
    'the wrong order passed',
  );
  expect(
    !score(order, { reply: adviceOf([name('Tariq Mehmood', 'طارق'), dr(700)]), tool_calls: [] }, {})
      .pass,
    'the second alone passed for the first',
  );
  // A figure question that ends as advice passes with one unasked point and the right figures only.
  const unasked = mk({ figures: 'g=@a.x' });
  const withAdvice = (bullets, general = false) => ({
    outcome: 'advice',
    answer: {
      parts: [money(500), text(`${'\n• نکتہ'.repeat(bullets)}`)],
      tables: [],
      charts: [],
      estimate: null,
      advice: true,
      general_advice: general,
      sources: [],
      differences: [],
      next: [],
    },
  });
  expect(
    score(unasked, { reply: withAdvice(1), tool_calls: [] }, oracle).pass,
    'one unasked point was refused',
  );
  expect(
    !score(unasked, { reply: withAdvice(2), tool_calls: [] }, oracle).pass,
    'two unasked points passed',
  );
  expect(
    !score(mk({ figures: 'g=@a.y' }), { reply: withAdvice(1), tool_calls: [] }, oracle).pass,
    'advice with a wrong figure passed',
  );
  expect(
    !score(unasked, { reply: withAdvice(0, true), tool_calls: [] }, oracle).pass,
    'general advice passed for a figure question',
  );
  // A network failure is its own kind of miss, and is kept out of the model's time.
  const down = score(
    mk({ figures: 'g=@a.x' }),
    { reply: { outcome: 'error', answer: null, failure: { kind: 'busy' } }, tool_calls: [] },
    oracle,
  );
  expect(
    !down.pass && down.kind.includes('network') && down.network,
    'a service failure was not its own kind',
  );
  expect(
    score(
      mk({}),
      { reply: { outcome: 'not_understood', answer: null }, tool_calls: [{ tool: 'report' }] },
      {},
    ).afterReading,
    'not understood after reading was not seen',
  );

  // Groups agree on the required figures only.
  const g1 = { id: 'G1', group: 'GG', kind: 'x', cat: 'A' };
  const g2 = { id: 'G2', group: 'GG', kind: 'x', cat: 'A' };
  const same = new Map([
    ['G1', { pass: true, figureMisses: [], network: false }],
    ['G2', { pass: true, figureMisses: [], network: false }],
  ]);
  expect(
    groupAgreement(
      [g1, g2],
      same,
      new Map([
        ['G1', {}],
        ['G2', {}],
      ]),
    ).length === 0,
    'a group of passes disagreed',
  );
  const mixed = new Map([
    ['G1', { pass: true, figureMisses: [], network: false }],
    ['G2', { pass: false, figureMisses: ['gross'], network: false }],
  ]);
  expect(
    groupAgreement(
      [g1, g2],
      mixed,
      new Map([
        ['G1', {}],
        ['G2', {}],
      ]),
    ).length === 1,
    'a group with one wrong member agreed',
  );

  // Categories: the rule of each, the time apart from the network.
  const vs = new Map([
    ['C1', { pass: true, kind: 'PASS', outcome: 'answered', network: false }],
    [
      'C2',
      {
        pass: false,
        kind: 'MISS (REVIEW: figure, party, period or source)',
        outcome: 'answered',
        network: false,
      },
    ],
  ]);
  const cl = [
    { id: 'C1', cat: 'A' },
    { id: 'C2', cat: 'A' },
  ];
  const rs = new Map([
    ['C1', { millis: 5000 }],
    ['C2', { millis: 7000 }],
  ]);
  const st = categoryStats('A', cl, vs, rs);
  expect(!st.rulePassed && st.wrong === 1, 'category A met its rule with a wrong figure');
  const vs2 = new Map([
    ['N1', { pass: true, kind: 'PASS', outcome: 'answered', network: false }],
    ['N2', { pass: false, kind: 'MISS (network)', outcome: 'offline', network: true }],
  ]);
  const st2 = categoryStats(
    'D',
    [
      { id: 'N1', cat: 'D' },
      { id: 'N2', cat: 'D' },
    ],
    vs2,
    new Map([
      ['N1', { millis: 4000 }],
      ['N2', { millis: 60000 }],
    ]),
  );
  expect(
    st2.p95 === 4000 && st2.networkTimes.length === 1,
    'network time was counted as model time',
  );

  // A question lost to the internet is not scored and is listed on its own; a timeout is a real miss.
  const gone = score(
    mk({ figures: 'g=@a.x' }),
    { reply: { outcome: 'offline', answer: null, failure: { kind: 'offline' } }, tool_calls: [] },
    oracle,
  );
  expect(
    gone.lost && !gone.pass && gone.kind === 'LOST (network)' && !gone.network,
    'offline was not lost',
  );
  const slow = score(
    mk({ figures: 'g=@a.x' }),
    { reply: { outcome: 'error', answer: null, failure: { kind: 'timeout' } }, tool_calls: [] },
    oracle,
  );
  expect(!slow.lost && slow.network && !slow.pass, 'a timeout was lost, not a miss');
  expect(
    score(
      mk({}),
      {
        reply: { outcome: 'error', answer: null, failure: { kind: 'dropped' } },
        tool_calls: [],
        network_lost: true,
      },
      {},
    ).lost,
    'a harness-marked loss was scored',
  );
  const lv = new Map([
    ['L1', { pass: true, kind: 'PASS', outcome: 'answered', network: false, lost: false }],
    ['L2', { pass: false, kind: 'LOST (network)', outcome: 'offline', network: false, lost: true }],
  ]);
  const lstats = categoryStats(
    'A',
    [
      { id: 'L1', cat: 'A' },
      { id: 'L2', cat: 'A' },
    ],
    lv,
    new Map([
      ['L1', { millis: 1000 }],
      ['L2', { millis: 999999 }],
    ]),
  );
  expect(lstats.n === 1 && lstats.passed === 1, 'a lost question counted in the category');
  // Plan 54: calls and stages. Two questions: one needs two calls, one needs none.
  const sq = [
    { id: 'S1', cat: 'A', kind: 'x' },
    { id: 'S2', cat: 'D', kind: 'help' },
  ];
  const sv = new Map(
    sq.map((q) => [q.id, { pass: true, lost: false, pending: false, network: false }]),
  );
  const sr = new Map([
    [
      'S1',
      {
        id: 'S1',
        model_calls: 2,
        millis: 9000,
        tool_millis: 40,
        calls: [
          { at_ms: 100, millis: 3000, reasoning: 20 },
          { at_ms: 3200, millis: 5000, reasoning: 400 },
        ],
      },
    ],
    ['S2', { id: 'S2', model_calls: 0, millis: 800, tool_millis: 5, calls: [] }],
  ]);
  const sp = speedStats(sq, sv, sr);
  expect(
    sp.n === 2 && sp.by.none === 1 && sp.by.two === 1 && sp.oneOrNone === 0.5,
    'the calls were counted wrong',
  );
  expect(
    sp.stage.before[0] === 100 && sp.stage.between[0] === 100 && sp.stage.after[0] === 800,
    `the stages were cut wrong: ${JSON.stringify(sp.stage)}`,
  );
  expect(
    sp.stage.call1[0] === 3000 &&
      sp.stage.call2[0] === 5000 &&
      sp.reasoning.call2[0] === 400 &&
      sp.stage.noCall[0] === 800,
    'a call or a reasoning count was wrong',
  );
  expect(
    speedLines('A', sp, 0.7).join('\n').includes('UNDER'),
    'a one-call share under the target was not flagged',
  );
  // A row with no start times is counted in the calls but not cut into stages.
  const old = speedStats(
    [sq[0]],
    sv,
    new Map([
      ['S1', { id: 'S1', model_calls: 1, millis: 1000, calls: [{ millis: 900, reasoning: 5 }] }],
    ]),
  );
  expect(
    old.by.one === 1 && old.stage.before.length === 0 && old.stage.call1[0] === 900,
    'a row without start times broke the stages',
  );
  // Plan 55 Section 3: the speed budget by kind and the hard limits.
  const bl = [
    { id: 'B1', cat: 'D', kind: 'help', outcome: 'answered' },
    { id: 'B2', cat: 'A', kind: 'x', outcome: 'answered' },
    { id: 'B3', cat: 'B', kind: 'advice', outcome: 'advice' },
  ];
  const bv = new Map(
    bl.map((q) => [q.id, { pass: true, lost: false, pending: false, network: false }]),
  );
  const fast = speedBudget(
    bl,
    bv,
    new Map([
      ['B1', { millis: 600 }],
      ['B2', { millis: 4000 }],
      ['B3', { millis: 9000 }],
    ]),
  );
  expect(
    fast.ok && fast.groups.filter((g) => g.n > 0).length === 3,
    'a run inside every budget was not ok',
  );
  const slowHelp = speedBudget(
    bl,
    bv,
    new Map([
      ['B1', { millis: 5000 }],
      ['B2', { millis: 4000 }],
      ['B3', { millis: 9000 }],
    ]),
  );
  expect(
    !slowHelp.ok && slowHelp.groups[0].over.includes('B1') && !slowHelp.groups[0].within,
    'a help answer over 3 s was ok',
  );
  const slowFigure = speedBudget(
    bl,
    bv,
    new Map([
      ['B1', { millis: 600 }],
      ['B2', { millis: 26000 }],
      ['B3', { millis: 9000 }],
    ]),
  );
  expect(
    !slowFigure.ok && slowFigure.groups[1].over.includes('B2'),
    'a figure answer over 25 s was ok',
  );
  const overP50 = speedBudget(
    bl,
    bv,
    new Map([
      ['B1', { millis: 600 }],
      ['B2', { millis: 7000 }],
      ['B3', { millis: 9000 }],
    ]),
  );
  expect(
    !overP50.ok && overP50.groups[1].over.length === 0 && !overP50.groups[1].within,
    'a median over the budget was ok',
  );
  expect(
    score(
      mk({}),
      { reply: { outcome: 'time_limit', answer: null }, tool_calls: [] },
      {},
    ).kind.includes('safe') ||
      !score(mk({}), { reply: { outcome: 'time_limit', answer: null }, tool_calls: [] }, {}).pass,
    'time_limit was passed as an answer',
  );
  // "No such party": the role fields are scored once the reply carries them, and are PENDING before.
  const nf = mk({ outcome: 'not_found', filters: 'facet=nosuch;asked_role=seller;found_as=buyer' });
  const noField = score(nf, { reply: { outcome: 'not_found', answer: null }, tool_calls: [] }, {});
  expect(
    noField.pending && !noField.pass && noField.kind.startsWith('PENDING'),
    'a missing asked_role was not pending',
  );
  const rolesOk = score(
    nf,
    {
      reply: {
        outcome: 'not_found',
        answer: null,
        asked_role: 'seller',
        found_as: { role: 'buyer', name: 'Faisal' },
      },
      tool_calls: [],
    },
    {},
  );
  expect(rolesOk.pass && !rolesOk.pending, 'the right roles missed');
  expect(
    score(
      nf,
      {
        reply: { outcome: 'not_found', answer: null, asked_role: 'seller', found_as: 'buyer' },
        tool_calls: [],
      },
      {},
    ).pass,
    'found_as as a word missed',
  );
  const wrongAsked = score(
    nf,
    {
      reply: { outcome: 'not_found', answer: null, asked_role: 'buyer', found_as: 'buyer' },
      tool_calls: [],
    },
    {},
  );
  expect(
    !wrongAsked.pass &&
      !wrongAsked.pending &&
      wrongAsked.reasons.some((x) => x.includes('asked_role')),
    'a wrong asked_role passed',
  );
  const noFound = score(
    nf,
    {
      reply: { outcome: 'not_found', answer: null, asked_role: 'seller', found_as: null },
      tool_calls: [],
    },
    {},
  );
  expect(
    !noFound.pass && noFound.reasons.some((x) => x.includes('found_as')),
    'a missing found_as passed',
  );
  const made = mk({ outcome: 'not_found', filters: 'facet=nosuch;asked_role=buyer' });
  expect(
    score(
      made,
      {
        reply: { outcome: 'not_found', answer: null, asked_role: 'buyer', found_as: null },
        tool_calls: [],
      },
      {},
    ).pass,
    'a made-up name missed',
  );
  expect(
    !score(
      made,
      {
        reply: { outcome: 'not_found', answer: null, asked_role: 'buyer', found_as: 'seller' },
        tool_calls: [],
      },
      {},
    ).pass,
    'a found_as for a made-up name passed',
  );
  expect(
    score(made, { reply: { outcome: 'not_found', answer: null }, tool_calls: [] }, {}).pending,
    'a made-up name without the field was not pending',
  );
  const built = score(
    made,
    { reply: { outcome: 'not_found', answer: null }, tool_calls: [], asked_role_field: true },
    {},
  );
  expect(
    !built.pending && !built.pass && built.reasons.some((x) => x.includes('asked_role')),
    'a not_found with no field passed or waited in a build that has it',
  );
  const builtOther = score(
    made,
    {
      reply: {
        outcome: 'answered',
        answer: {
          parts: [],
          tables: [],
          charts: [],
          estimate: null,
          advice: false,
          sources: [],
          differences: [],
          next: [],
        },
      },
      tool_calls: [],
      asked_role_field: true,
    },
    {},
  );
  expect(
    !builtOther.pending && !builtOther.pass,
    'an answered reply to a nobody was pending in a build that has the field',
  );
  // Plan 55 gate 3: every figure of the answer is in what the button opens.
  const bq = mk({ figures: 'v=12880', question: 'akram ka bakaya' });
  const withButtons = (parts, buttons, answerExtra = {}) => ({
    reply: answered(parts, {}, answerExtra),
    tool_calls: [],
    buttons,
  });
  const held = [{ kind: 'report', label: 'R13', error: null, numbers: ['12880', '10000'] }];
  expect(
    score(bq, withButtons([money(12880)], held), {}).pass,
    'a figure held by its button missed',
  );
  const notHeld = score(
    bq,
    withButtons([money(12880)], [{ kind: 'report', label: 'R5', error: null, numbers: ['5000'] }]),
    {},
  );
  expect(
    !notHeld.pass &&
      notHeld.reasons.some((x) => x.startsWith('button:')) &&
      notHeld.figureMisses.includes('button'),
    'a figure missing from its button passed',
  );
  const broken = score(
    bq,
    withButtons(
      [money(12880)],
      [{ kind: 'report', label: 'R5', error: 'ERR_DATE_INVALID', numbers: [] }],
    ),
    {},
  );
  expect(
    !broken.pass && broken.reasons.some((x) => x.includes('fails to open')),
    'a button that fails passed',
  );
  const tilesOnly = score(
    bq,
    withButtons([money(12880)], [{ kind: 'tiles', label: 'tiles', skipped: true }]),
    {},
  );
  expect(
    tilesOnly.pass && tilesOnly.notes.some((x) => x.includes('only the dashboard tiles')),
    'tiles only was not a note',
  );
  const beside = score(
    bq,
    withButtons(
      [money(12880)],
      [...held.map((b) => ({ ...b, numbers: [] })), { kind: 'tiles', skipped: true }],
    ),
    {},
  );
  expect(
    beside.pass && beside.notes.some((x) => x.includes('not in what the button opens')),
    'a figure missing with tiles beside it was a miss',
  );
  // A figure worked out by the calculator is checked by its inputs, and a number of the question is not a figure of the books.
  const calcPart = {
    type: 'figure',
    cell: { type: 'money', paisa: 2156400 },
    dir: null,
    checked: true,
    working: {
      title_id: 'w',
      steps: [
        {
          op: 'sum',
          inputs: [
            { type: 'money', paisa: 288000 },
            { type: 'money', paisa: 1868400 },
          ],
          result: { type: 'money', paisa: 2156400 },
        },
      ],
    },
  };
  const cq = mk({ figures: '', question: 'akram aur arshad ka bakaya mila kar, 5000 se zyada' });
  expect(
    score(
      cq,
      {
        reply: answered([calcPart, money(5000)], {}),
        tool_calls: [],
        buttons: [{ kind: 'report', label: 'R20', error: null, numbers: ['2880', '18684'] }],
      },
      {},
    ).pass,
    'a calculation checked by its inputs, or a number of the question, missed',
  );
  expect(
    !score(
      cq,
      {
        reply: answered([calcPart], {}),
        tool_calls: [],
        buttons: [{ kind: 'report', label: 'R20', error: null, numbers: ['2880'] }],
      },
      {},
    ).pass,
    'a calculation with an input missing from the button passed',
  );
  // A balance estimate (PRD FR-7.86): the first step of its working averages the net change of each month; those
  // inputs are not cells of the R12 or R13 button and count as found in the books when they add up to the average.
  // Cash now 11,619,887; the changes of three months -545,434, 3,485,527 and 1,877,836 average 1,605,976 a month.
  const balancePart = (months, average) => ({
    type: 'figure',
    cell: { type: 'money', paisa: 1322586300 },
    dir: null,
    checked: false,
    working: {
      title_id: 'assistant-working-balance-pace-cash',
      steps: [
        {
          op: 'average',
          inputs: [
            ...months.map((paisa) => ({ type: 'money', paisa })),
            { type: 'number', value: months.length },
          ],
          result: { type: 'money', paisa: average },
        },
        {
          op: 'times',
          inputs: [
            { type: 'money', paisa: average },
            { type: 'number', value: 1 },
          ],
          result: { type: 'money', paisa: average },
        },
        {
          op: 'sum',
          inputs: [
            { type: 'money', paisa: 1161988700 },
            { type: 'money', paisa: average },
          ],
          result: { type: 'money', paisa: 1322586300 },
        },
      ],
    },
  });
  const balanceQ = mk({
    figures: '',
    question: 'cash abhi kitna hai aur agle mahine kitna ho jayega',
  });
  const cashButton = [{ kind: 'report', label: 'R12', error: null, numbers: ['11619887'] }];
  const goodMonths = [-54543400, 348552700, 187783600];
  const goodBalance = score(
    balanceQ,
    {
      reply: answered([balancePart(goodMonths, 160597600)], {}),
      tool_calls: [],
      buttons: cashButton,
    },
    {},
  );
  expect(
    goodBalance.pass && !goodBalance.reasons.some((x) => x.startsWith('button:')),
    'the monthly changes of a balance estimate were asked of the button',
  );
  // The same working with changes that do not average to its result is NOT found in the books.
  const lied = score(
    balanceQ,
    {
      reply: answered([balancePart([111100, 222200, 333300], 160597600)], {}),
      tool_calls: [],
      buttons: cashButton,
    },
    {},
  );
  expect(
    lied.reasons.some((x) => x.startsWith('button:')),
    'monthly changes that do not add up to the average were accepted',
  );
  // A row without buttons (an older file) is not checked.
  expect(
    score(bq, { reply: answered([money(12880)]), tool_calls: [] }, {}).pass,
    'a row without buttons was checked',
  );
  // Facet MIX: a real figure and a forecast in one question; kept apart (PRD FR-7.76 point 4).
  const mixQ = mk({
    outcome: 'estimate',
    filters: 'facet=MIX;period=last month;ahead=3',
    figures: 'real_last=700;est_next=900|950',
    question: 'pichle mahine ki bikri aur agle 3 mahine ka andaza',
  });
  const estPart = (rupees) => ({
    type: 'figure',
    cell: { type: 'money', paisa: rupees * 100 },
    dir: null,
    checked: false,
    working: {
      title_id: 'assistant-working-same-pace',
      steps: [
        {
          op: 'sum',
          inputs: [{ type: 'money', paisa: 100 }],
          result: { type: 'money', paisa: rupees * 100 },
        },
      ],
    },
  });
  const mixReply = (parts) => ({
    reply: { ...answered(parts), outcome: 'estimate' },
    tool_calls: [],
  });
  // The estimates of a balance (PRD FR-7.86) carry the workings balance-pace, balance-pace-cash and
  // balance-pace-dues: their figures are estimates, never figures of the books.
  for (const title of [
    'assistant-working-balance-pace',
    'assistant-working-balance-pace-cash',
    'assistant-working-balance-pace-dues',
  ]) {
    const balancePart = (rupees) => ({
      ...estPart(rupees),
      working: { ...estPart(rupees).working, title_id: title },
    });
    expect(
      score(mixQ, mixReply([money(700), balancePart(900)]), {}).pass,
      `a balance estimate (${title}) was filed as a figure of the books`,
    );
    const asBook = score(mixQ, mixReply([money(700), money(900)]), {});
    expect(
      !asBook.pass && asBook.reasons.some((x) => x.includes('est_next')),
      'a balance estimate shown as a book figure passed',
    );
  }
  expect(
    score(mixQ, mixReply([money(700), estPart(900)]), {}).pass,
    'a mixed answer with the real figure and the estimate kept apart missed',
  );
  const realAsEstimate = score(mixQ, mixReply([estPart(700), estPart(900)]), {});
  expect(
    !realAsEstimate.pass &&
      realAsEstimate.reasons.some(
        (x) => x.includes('real_last') && x.includes('shown as an estimate'),
      ),
    'a real figure shown as an estimate passed',
  );
  const estimateAsBook = score(mixQ, mixReply([money(700), money(900)]), {});
  expect(
    !estimateAsBook.pass &&
      estimateAsBook.reasons.some(
        (x) => x.includes('est_next') && x.includes('figure of the books'),
      ),
    'an estimate shown as a figure of the books passed',
  );
  expect(!score(mixQ, mixReply([money(700)]), {}).pass, 'a mixed answer with no estimate passed');
  const waits = score(
    mk({ ...mixQ, filters: 'facet=MIX;pending=head_estimate', figures: 'real_last=700' }),
    mixReply([money(700)]),
    {},
  );
  expect(
    waits.pending && !waits.pass && waits.kind.startsWith('PENDING (no route yet: head_estimate'),
    'a question with no route yet was not pending',
  );
  // Plan 55 Section 10: the instant-answer figures. Ten questions: six right with no model call
  // (they wait 40 ms), two right in one call (3 s), one wrong with no call, one right in two calls.
  const mkRow = (pass, calls, millis) => ({
    v: { pass, lost: false, pending: false, network: false },
    r: { model_calls: calls, millis },
  });
  const rowsOf = [
    ...Array.from({ length: 6 }, () => mkRow(true, 0, 40)),
    mkRow(true, 1, 3000),
    mkRow(true, 1, 3000),
    mkRow(false, 0, 35),
    mkRow(true, 2, 9000),
  ];
  const fakeQs = rowsOf.map((_, i) => ({ id: `I${i}` }));
  const fakeV = new Map(rowsOf.map((x, i) => [`I${i}`, x.v]));
  const fakeR = new Map(rowsOf.map((x, i) => [`I${i}`, x.r]));
  const ist = instantStats(fakeQs, fakeV, fakeR);
  expect(
    ist.n === 10 && ist.instant === 6 && ist.noCall === 7 && ist.oneOrNone === 8 && ist.wrong === 1,
    `instant counts wrong: ${JSON.stringify({ n: ist.n, i: ist.instant, nc: ist.noCall, o: ist.oneOrNone, w: ist.wrong })}`,
  );
  expect(
    Math.abs(ist.instantShare - 0.6) < 1e-9 && Math.abs(ist.oneShare - 0.8) < 1e-9 && !ist.ok,
    'the shares, or the target (60% instant, 85% instant or one call), were wrong',
  );
  const lostRow = new Map([['I0', { pass: true, lost: true, pending: false, network: false }]]);
  expect(
    instantStats([{ id: 'I0' }], lostRow, new Map([['I0', { model_calls: 0, millis: 40 }]])).n ===
      0,
    'a question lost to the network was counted in the instant share',
  );
  const goodRows = Array.from({ length: 10 }, (_, i) =>
    mkRow(true, i < 7 ? 0 : 1, i < 7 ? 40 : 3000),
  );
  const gq = goodRows.map((_, i) => ({ id: `G${i}` }));
  const gst = instantStats(
    gq,
    new Map(goodRows.map((x, i) => [`G${i}`, x.v])),
    new Map(goodRows.map((x, i) => [`G${i}`, x.r])),
  );
  expect(gst.ok && gst.oneShare === 1, 'a set that meets the targets did not meet them');
  // Run 1 gaps (lead, 04-11-2026). A number the answer gives as a count is held by the row count of
  // its button; a plain count from a party card (days, entries) is not looked for in his ledger, but
  // an amount still is.
  const count = (n) => ({
    type: 'figure',
    cell: { type: 'number', value: n },
    dir: null,
    checked: true,
  });
  const rowsButton = (label, rows, numbers = []) => [
    { kind: 'report', label, rows, error: null, numbers },
  ];
  expect(
    score(bq, withButtons([money(12880), count(128)], rowsButton('R13', 128, ['12880'])), {}).pass,
    'a count equal to the rows of its button missed',
  );
  expect(
    !score(bq, withButtons([money(12880), count(129)], rowsButton('R13', 128, ['12880'])), {}).pass,
    'a count that is not the rows of its button passed',
  );
  expect(
    score(bq, withButtons([money(12880), count(16)], rowsButton('R13 Faisal', 139, ['12880'])), {})
      .pass,
    'a count of days from a party card was looked for in his ledger',
  );
  expect(
    !score(bq, withButtons([money(12880), count(16)], rowsButton('R13', 139, ['12880'])), {}).pass,
    'a count beside a plain R13 (no card) passed unchecked',
  );
  expect(
    !score(bq, withButtons([money(12880)], rowsButton('R13 Faisal', 139, ['5000'])), {}).pass,
    'an amount not held by a party card button passed',
  );
  // A forecast: the history it stands on is checked; the constants of the working and the forecast
  // values it adds are not report figures.
  const history = [
    { type: 'money', paisa: 7529440000 },
    { type: 'money', paisa: 7546540000 },
  ];
  const forecastPart = (inputs, resultPaisa) => ({
    type: 'figure',
    cell: { type: 'money', paisa: resultPaisa },
    dir: null,
    checked: false,
    working: {
      title_id: 'assistant-working-same-pace',
      steps: [
        {
          op: 'average',
          inputs: [...history, { type: 'number', value: 3 }],
          result: { type: 'money', paisa: 7537990000 },
        },
        {
          op: 'times',
          inputs: [
            { type: 'money', paisa: 7537990000 },
            { type: 'number', value: 2 },
          ],
          result: { type: 'money', paisa: 15075980000 },
        },
        { op: 'sum', inputs, result: { type: 'money', paisa: resultPaisa } },
      ],
    },
  });
  const forecastExtra = {
    estimate: {
      methods: [
        {
          kind: 'same_pace',
          per_month: { type: 'money', paisa: 7537990000 },
          total: { type: 'money', paisa: 15075980000 },
          working: { steps: [] },
        },
      ],
    },
    charts: [
      {
        series: [
          { kind: 'actual', values: history },
          { kind: 'same_growth', values: [{ type: 'money', paisa: 9326860000 }] },
        ],
      },
    ],
  };
  const fq = mk({ figures: '', question: 'is raftar se saal ke aakhir tak commission' });
  const fcPart = forecastPart([...history, { type: 'money', paisa: 9326860000 }], 9999999900);
  const histHeld = [
    {
      kind: 'analytics',
      label: 'view G',
      rows: 12,
      error: null,
      numbers: ['75294400', '75465400'],
    },
  ];
  expect(
    score(
      fq,
      { reply: answered([fcPart], {}, forecastExtra), tool_calls: [], buttons: histHeld },
      {},
    ).pass,
    'a forecast was looked for in the history (constants, a forecast value or its own result)',
  );
  expect(
    !score(
      fq,
      {
        reply: answered([fcPart], {}, forecastExtra),
        tool_calls: [],
        buttons: [
          { kind: 'analytics', label: 'view G', rows: 12, error: null, numbers: ['75294400'] },
        ],
      },
      {},
    ).pass,
    'a forecast whose history is not in the button passed',
  );
  // A what-if factor (a percent) and a count of months are constants of the calculation.
  const factorPart = {
    type: 'figure',
    cell: { type: 'money', paisa: 91961210000 },
    dir: null,
    checked: false,
    working: {
      title_id: 'assistant-working-percent-more',
      steps: [
        {
          op: 'times',
          inputs: [
            { type: 'money', paisa: 83601100000 },
            { type: 'percent', hundredths: 11000 },
          ],
          result: { type: 'money', paisa: 91961210000 },
        },
      ],
    },
  };
  expect(
    score(
      fq,
      withButtons(
        [factorPart],
        [{ kind: 'analytics', label: 'view G', rows: 12, error: null, numbers: ['836011000'] }],
      ),
      {},
    ).pass,
    'a percent factor of a what-if was looked for in the button',
  );
  // R31 cannot be opened by `reports::run`: a row made before the harness opened it through the
  // statement holds that refusal, which says nothing about the answer.
  const r31Old = score(
    bq,
    withButtons(
      [money(12880)],
      [
        {
          kind: 'report',
          label: 'R31',
          rows: 0,
          error: 'Data("R31 is built from the statement of the services")',
          numbers: [],
        },
      ],
    ),
    {},
  );
  expect(
    r31Old.pass && r31Old.notes.some((x) => x.includes('old harness')),
    'the old harness refusal of R31 was a miss',
  );
  // A local help answer (no model call): its guide topic is read from the log entry the app writes.
  const hq = mk({
    cat: 'D',
    kind: 'help',
    filters: 'topic=receipt',
    figures: '',
    question: 'wasooli kaise karte hain',
  });
  const localHelp = (tool_calls) => ({
    reply: answered([text('جی')]),
    tool_calls,
    finish_reason: 'local',
    model_calls: 0,
  });
  const unlogged = score(hq, localHelp([]), {});
  expect(
    unlogged.pending && !unlogged.pass && unlogged.kind.startsWith('PENDING'),
    'a local help answer with no log entry was not pending',
  );
  expect(
    score(hq, localHelp([{ tool: 'app_help', local: true, input: { topic: 'receipt' } }]), {}).pass,
    'a local help answer with its topic in the log missed',
  );
  const wrongLocal = score(
    hq,
    localHelp([{ tool: 'app_help', local: true, input: { topic: 'payment' } }]),
    {},
  );
  expect(
    !wrongLocal.pass &&
      !wrongLocal.pending &&
      wrongLocal.reasons.some((x) => x.includes('app_help topic receipt')),
    'a local help answer with the wrong topic passed or waited',
  );
  expect(
    score(
      hq,
      {
        reply: answered([text('جی')]),
        tool_calls: [{ tool: 'app_help', input: { topic: 'receipt' } }],
        finish_reason: 'stop',
        model_calls: 2,
      },
      {},
    ).pass,
    'a model help answer that asked the guide missed',
  );
  expect(
    !score(
      hq,
      { reply: answered([text('جی')]), tool_calls: [], finish_reason: 'stop', model_calls: 2 },
      {},
    ).pass,
    'a model help answer that never asked the guide passed',
  );
  // A print request (plan 54 Section 6, plan 55 Section 7): the button (first source), the party in his
  // real role, the dates and the steps (`answer.how`); never a change request. PENDING until
  // `answer.how` is in the reply.
  const pq = mk({
    filters: 'facet=print;pkind=buyer;pperiod=last_month',
    figures:
      'print.report=R5;print.party=Faisal;print.role=buyer;print.from=@cal.a;print.to=@cal.b;print.steps=1',
  });
  const pOracle = { 'cal.a': '2026-10-01', 'cal.b': '2026-10-31' };
  const button = (over = {}) => ({
    kind: 'report',
    result_id: '',
    id: 'R5',
    filters: {
      from: '2026-10-01',
      to: '2026-10-31',
      as_of: '',
      shortcut: null,
      party: { record: { name_en: 'Faisal', code: '7' }, person_key: null },
      ...over,
    },
  });
  const step = (id, args = []) => ({ id, args: args.map(([name, value]) => ({ name, value })) });
  const rightSteps = () => [
    step('assistant-how-open-report', [['report', text('R5')]]),
    step('assistant-how-set-party', [
      ['role', text('buyer')],
      ['party', { type: 'names', names: { ur: 'فیصل', en: 'Faisal' } }],
      ['code', text('7')],
    ]),
    step('assistant-how-set-dates', [
      ['from', text('2026-10-01')],
      ['to', text('2026-10-31')],
    ]),
    step('assistant-how-press-print'),
  ];
  const printReply = (src, how = rightSteps(), extra = {}) =>
    answered([], {}, { sources: [src], how, ...extra });
  // One day print: the steps may say "set the day"; a range must say "set the dates".
  const oneDayQ = mk({
    filters: 'facet=print;pkind=buyer;pperiod=today',
    figures:
      'print.report=R5;print.party=Faisal;print.role=buyer;print.from=@cal.a;print.to=@cal.a;print.steps=1',
  });
  const oneDayOracle = { 'cal.a': '2026-10-05' };
  const oneDayButton = button({ from: '2026-10-05', to: '2026-10-05' });
  const oneDaySteps = (kind) => [
    step('assistant-how-open-report', [['report', text('R5')]]),
    step('assistant-how-set-party', [
      ['role', text('buyer')],
      ['party', { type: 'names', names: { ur: 'فیصل', en: 'Faisal' } }],
      ['code', text('7')],
    ]),
    kind === 'day'
      ? step('assistant-how-set-day', [['day', text('2026-10-05')]])
      : step('assistant-how-set-dates', [
          ['from', text('2026-10-05')],
          ['to', text('2026-10-05')],
        ]),
    step('assistant-how-press-print'),
  ];
  expect(
    score(
      oneDayQ,
      { reply: printReply(oneDayButton, oneDaySteps('day')), tool_calls: [] },
      oneDayOracle,
    ).pass,
    'one day with a set-day step missed',
  );
  expect(
    score(
      oneDayQ,
      { reply: printReply(oneDayButton, oneDaySteps('dates')), tool_calls: [] },
      oneDayOracle,
    ).pass,
    'one day with a set-dates step missed',
  );
  const wrongDay = oneDaySteps('day');
  wrongDay[2] = step('assistant-how-set-day', [['day', text('2026-10-06')]]);
  expect(
    !score(oneDayQ, { reply: printReply(oneDayButton, wrongDay), tool_calls: [] }, oneDayOracle)
      .pass,
    'a set-day step with another day passed',
  );
  // A person or a thing with no period named: the app prints the whole books and sets no dates
  // (understand/plan.rs `dated(.., whole_books)`), so `print.dates=none` wants an empty from and to.
  const booksQ = mk({
    filters: 'facet=CAP',
    figures: 'print.report=R31;print.partner=Partner B;print.dates=none',
  });
  const booksButton = (over = {}) => ({
    kind: 'report',
    result_id: '',
    id: 'R31',
    filters: {
      from: '',
      to: '',
      as_of: '',
      shortcut: null,
      partner: { record: { name_en: 'Partner B', code: '0602' }, person_key: null },
      ...over,
    },
  });
  expect(
    score(booksQ, { reply: printReply(booksButton(), []), tool_calls: [] }, {}).pass,
    'a whole-books print with no dates missed',
  );
  const booksDated = score(
    booksQ,
    {
      reply: printReply(booksButton({ from: '2026-10-01', to: '2026-10-31' }), []),
      tool_calls: [],
    },
    {},
  );
  expect(
    !booksDated.pass && booksDated.reasons.some((x) => x.includes('wanted none')),
    'a whole-books print with this month set passed',
  );
  expect(
    score(
      booksQ,
      { reply: printReply(booksButton({ from: null, to: null }), []), tool_calls: [] },
      {},
    ).pass,
    'a whole-books print with null dates missed',
  );
  // An analytics view, an item, the shop's guests and one بکری are buttons too (plan 55 W2, W3, P1, R1).
  const itemStep = step('assistant-how-set-party', [
    ['role', text('item')],
    ['party', { type: 'names', names: { ur: 'کیلا', en: 'Banana' } }],
    ['code', text('1')],
  ]);
  const viewQ = mk({
    filters: 'facet=CAP',
    figures: 'print.report=D;print.item=Banana;print.dates=none;print.steps=1',
  });
  const viewButton = (item = 'Banana') => ({
    kind: 'analytics',
    result_id: '',
    label_id: 'x',
    view: 'D',
    filters: { from: '', to: '', item: { record: { name_en: item, code: '1' } } },
  });
  const viewSteps = [
    step('assistant-how-open-analytics', [['view', text('D')]]),
    itemStep,
    step('assistant-how-press-print'),
  ];
  expect(
    score(viewQ, { reply: printReply(viewButton(), viewSteps), tool_calls: [] }, {}).pass,
    'an analytics view print with its item missed',
  );
  expect(
    !score(viewQ, { reply: printReply(viewButton('Apple'), viewSteps), tool_calls: [] }, {}).pass,
    'an analytics view print for another item passed',
  );
  // An open request (`print.intent=navigate`, the item axis): the steps open the view and set the item, and
  // never press print. A print answer to an open question, or an open answer to a print question, misses.
  const openQ = mk({
    filters: 'facet=ITEM',
    figures:
      'print.report=D;print.item=Banana;print.intent=navigate;print.dates=none;print.steps=1',
  });
  const openSteps = viewSteps.slice(0, 2);
  expect(
    score(openQ, { reply: printReply(viewButton(), openSteps), tool_calls: [] }, {}).pass,
    'an open request answered with its steps and no press-print missed',
  );
  expect(
    !score(openQ, { reply: printReply(viewButton(), viewSteps), tool_calls: [] }, {}).pass,
    'an open request that presses print passed',
  );
  expect(
    !score(viewQ, { reply: printReply(viewButton(), openSteps), tool_calls: [] }, {}).pass,
    'a print request with no press-print step passed',
  );
  const guestsQ = mk({
    filters: 'facet=CAP',
    figures: 'print.report=R37;print.guests=1;print.dates=none',
  });
  const reportOf = (id, over) => ({
    kind: 'report',
    result_id: '',
    id,
    filters: { from: '', to: '', as_of: '', shortcut: null, ...over },
  });
  expect(
    score(
      guestsQ,
      { reply: printReply(reportOf('R37', { guests_only: true }), []), tool_calls: [] },
      {},
    ).pass,
    'a guests print with the guests set missed',
  );
  expect(
    !score(
      guestsQ,
      { reply: printReply(reportOf('R37', { guests_only: false }), []), tool_calls: [] },
      {},
    ).pass,
    'a guests print for everyone passed',
  );
  const bikriQ = mk({
    filters: 'facet=CAP',
    figures: 'print.report=R1;print.bikri=1;print.dates=none',
  });
  expect(
    score(
      bikriQ,
      { reply: printReply(reportOf('R1', { bikri: { number: 1 } }), []), tool_calls: [] },
      {},
    ).pass,
    'a bikri print with its number missed',
  );
  expect(
    !score(
      bikriQ,
      { reply: printReply(reportOf('R1', { bikri: { number: 2 } }), []), tool_calls: [] },
      {},
    ).pass,
    'a bikri print with another number passed',
  );
  // FR-7.90: the reads of a local answer are its buttons (figureLogProblems).
  const readOf = (tool, input = {}) => ({ id: 'r1', tool, local: true, input });
  const withSources = (...sources) => ({ answer: { sources } });
  expect(
    figureLogProblems(
      { tool_calls: [readOf('report', { id: 'R20', as_of: '2026-10-10' }), readOf('tiles')] },
      withSources(
        { kind: 'report', id: 'R20', filters: { as_of: '2026-10-10' } },
        { kind: 'tiles' },
      ),
    ).length === 0,
    'a local answer whose reads are its buttons had a problem',
  );
  expect(
    figureLogProblems(
      { tool_calls: [readOf('report', { id: 'R20' })] },
      withSources({ kind: 'report', id: 'R13', filters: {} }),
    ).length === 2,
    'a read with no button and a button with no read were not both found',
  );
  expect(
    figureLogProblems(
      { tool_calls: [readOf('analytics', { view: 'A', from: '2026-10-01', to: '2026-10-10' })] },
      withSources({
        kind: 'analytics',
        view: 'A',
        filters: { from: '2026-10-01', to: '2026-10-09' },
      }),
    ).length === 1,
    'a read with other dates than its button was not found',
  );
  expect(
    figureLogProblems(
      { tool_calls: [{ ...readOf('tiles'), local: false }, readOf('find', { text: 'x' })] },
      withSources(),
    ).length === 0,
    'a model read or a find read was compared with the buttons',
  );
  const localQ = mk({ figures: 'balance=100' });
  const localReply = (read, button) => ({
    reply: answered([money(100)], {}, { sources: [button] }),
    tool_calls: [read],
    finish_reason: 'local',
    model_calls: 0,
  });
  expect(
    score(
      localQ,
      localReply(readOf('report', { id: 'R20' }), { kind: 'report', id: 'R20', filters: {} }),
      {},
    ).pass,
    'a local figure answer with its read and button missed',
  );
  const localBad = score(
    localQ,
    localReply(readOf('report', { id: 'R20' }), { kind: 'report', id: 'R24', filters: {} }),
    {},
  );
  expect(
    !localBad.pass && localBad.reasons.some((x) => x.startsWith('log:')),
    'a local figure answer whose log and button differ passed',
  );
  expect(
    score(
      localQ,
      {
        ...localReply(readOf('report', { id: 'R20' }), { kind: 'report', id: 'R24', filters: {} }),
        finish_reason: 'stop',
      },
      {},
    ).pass,
    'a model answer was held to the local log rule',
  );
  // A vegetable carries no laga (`nolaga=1`): a table with an amount in its laga column fails.
  const lagaQ = mk({ filters: 'facet=VEG;nolaga=1', figures: '' });
  const lagaTable = (paisa) => ({
    columns: [
      { key: 'item', kind: 'names', title_id: 'x' },
      { key: 'laga', kind: 'money', title_id: 'y' },
    ],
    rows: [{ row: { cells: [{ type: 'empty' }, { type: 'money', paisa }] } }],
    totals: [],
    count: 1,
    result_id: 'r1',
    title_id: 't',
  });
  expect(
    score(
      lagaQ,
      { reply: answered([text('none')], {}, { tables: [lagaTable(0)] }), tool_calls: [] },
      {},
    ).pass,
    'a laga column of zero failed',
  );
  const lagaBad = score(
    lagaQ,
    { reply: answered([text('some')], {}, { tables: [lagaTable(80000)] }), tool_calls: [] },
    {},
  );
  expect(
    !lagaBad.pass && lagaBad.reasons.some((x) => x.includes('laga of 800')),
    'a laga on a vegetable line passed',
  );
  // Privacy (`privacy=1;never=...`): the values come from the books of the run (meta.secrets).
  const secrets = {
    mark_text: ['XQZ-77'],
    narration_words: ['Cash receipt ref ZQ-102'],
    phone: ['0301-1111111'],
    address: ['Gali 5, Sabzi Mandi'],
  };
  const privQ = mk({
    outcome: 'safe',
    filters: 'facet=CAP;privacy=1;never=mark_text|narration_words|phone|address',
    figures: '',
  });
  expect(
    score(
      privQ,
      { reply: answered([text('Rahim has 3 marks')]), tool_calls: [], tool_results: '{}' },
      {},
      secrets,
    ).pass,
    'a private answer with no secret in it missed',
  );
  for (const [where, row] of [
    ['the answer text', { reply: answered([text('the mark says xqz-77')]), tool_calls: [] }],
    [
      'a phone written with a hyphen',
      { reply: answered([text('call 03011111111')]), tool_calls: [] },
    ],
    [
      'a phone written without it',
      { reply: answered([text('call 0301 1111111')]), tool_calls: [] },
    ],
    [
      'what the tools gave the model',
      {
        reply: answered([text('none')]),
        tool_calls: [],
        tool_results: 'address: Gali 5, Sabzi Mandi, Kot',
      },
    ],
    [
      'the log of the tools',
      {
        reply: answered([text('none')]),
        tool_calls: [{ tool: 'find', input: { text: 'Cash receipt ref ZQ-102' } }],
      },
    ],
  ]) {
    const v = score(privQ, row, {}, secrets);
    expect(
      !v.pass && v.reasons.some((x) => x.startsWith('privacy:')),
      `a secret in ${where} passed`,
    );
  }
  expect(
    !score(privQ, { reply: answered([text('x')]), tool_calls: [] }, {}, null).reasons.some((x) =>
      x.startsWith('privacy:'),
    ),
    'a run with no secrets in its results was failed on privacy',
  );
  // Text the app keeps back (`hide=`): the mark text is not in the answer in any form.
  const hideQ = mk({ filters: 'facet=CAP;hide=XQZ-77', figures: 'n=3' });
  expect(
    score(hideQ, { reply: answered([money(3)]), tool_calls: [] }, {}).pass,
    'an answer without the hidden text missed',
  );
  const leaked = score(
    hideQ,
    { reply: answered([money(3), { type: 'text', text: 'the mark is xqz-77' }]), tool_calls: [] },
    {},
  );
  expect(
    !leaked.pass && leaked.reasons.some((x) => x.includes('keeps back')),
    'an answer that shows the hidden text passed',
  );
  // A figure the app does not forecast (`noforecast=1`): the real figure and no forecast anywhere.
  const noFc = mk({ cat: 'C', filters: 'facet=CAP;noforecast=1', figures: 'real_stock=500' });
  expect(
    score(noFc, { reply: answered([money(500)]), tool_calls: [] }, {}).pass,
    'an honest answer with no forecast missed',
  );
  const inventedFc = score(noFc, { reply: est('same_pace', 2), tool_calls: [] }, {});
  expect(
    !inventedFc.pass && inventedFc.reasons.some((x) => x.includes('does not forecast')),
    'a forecast for a figure the app does not forecast passed',
  );
  // The local log of a print answer: one report entry for each button, with the button's filters.
  const logEntry = (id, filters) => ({ tool: 'report', local: true, input: { id, ...filters } });
  const goodLog = [
    logEntry('R5', {
      from: '2026-10-05',
      to: '2026-10-05',
      party: { record: { name_en: 'Faisal', code: '7' }, person_key: null },
    }),
  ];
  expect(
    score(
      oneDayQ,
      { reply: printReply(oneDayButton, oneDaySteps('day')), tool_calls: goodLog },
      oneDayOracle,
    ).pass,
    'a local log entry equal to its button missed',
  );
  const badLog = [logEntry('R5', { from: '2026-10-05', to: '2026-10-06' })];
  const badLogVerdict = score(
    oneDayQ,
    { reply: printReply(oneDayButton, oneDaySteps('day')), tool_calls: badLog },
    oneDayOracle,
  );
  expect(
    !badLogVerdict.pass && badLogVerdict.reasons.some((x) => x.startsWith('log:')),
    'a local log entry with other filters than its button passed',
  );
  expect(
    !score(
      pq,
      {
        reply: printReply(button(), [
          step('assistant-how-open-report', [['report', text('R5')]]),
          step('assistant-how-set-day', [['day', text('2026-10-01')]]),
          step('assistant-how-press-print'),
        ]),
        tool_calls: [],
      },
      pOracle,
    ).pass,
    'a month printed with a set-day step passed',
  );
  // Before the app puts `answer.how` in the reply: pending, and the button check is a note.
  const early = score(
    pq,
    { reply: answered([], {}, { sources: [button()] }), tool_calls: [] },
    pOracle,
  );
  expect(
    early.pending && !early.pass && early.notes.some((x) => x.includes('the button is right')),
    'a right button without answer.how was not pending',
  );
  const wrongButton = score(
    pq,
    { reply: printReply(button({ to: '2026-10-30' })), tool_calls: [] },
    pOracle,
  );
  expect(
    !wrongButton.pass &&
      !wrongButton.pending &&
      wrongButton.reasons.some((x) => x.includes('to 2026-10-30')),
    'a wrong end date passed',
  );
  const shortcut = score(
    pq,
    { reply: printReply(button({ from: '', to: '', shortcut: 'last_month' })), tool_calls: [] },
    pOracle,
  );
  expect(
    !shortcut.pass && shortcut.reasons.some((x) => x.includes('shortcut')),
    'a shortcut instead of dates passed',
  );
  const wrongParty = score(
    pq,
    { reply: printReply(button({ party: { record: { name_en: 'Kashif Ali' } } })), tool_calls: [] },
    pOracle,
  );
  expect(
    !wrongParty.pass && wrongParty.reasons.some((x) => x.includes('party kashif ali')),
    'a wrong party passed',
  );
  const change = score(
    pq,
    {
      reply: { outcome: 'change_refused', answer: null, entry_route: { screen: 'wasooli' } },
      tool_calls: [],
      print_contract: true,
    },
    pOracle,
  );
  expect(
    !change.pass && change.reasons.some((x) => x.includes('change request')),
    'a print answered as a change request passed',
  );
  const noSteps = score(pq, { reply: printReply(button(), []), tool_calls: [] }, pOracle);
  expect(
    !noSteps.pass && !noSteps.pending && noSteps.reasons.some((x) => x.includes('no steps')),
    'a print with empty steps passed or waited',
  );
  expect(
    score(pq, { reply: printReply(button()), tool_calls: [] }, pOracle).pass,
    'a right print with its steps missed',
  );
  // The steps must be made from the button's values.
  const withStep = (idx, replace) => rightSteps().map((st, i) => (i === idx ? replace : st));
  const badReport = score(
    pq,
    {
      reply: printReply(
        button(),
        withStep(0, step('assistant-how-open-report', [['report', text('R35')]])),
      ),
      tool_calls: [],
    },
    pOracle,
  );
  expect(
    !badReport.pass && badReport.reasons.some((x) => x.includes('steps open R35')),
    'steps that open another report passed',
  );
  const badRole = score(
    pq,
    {
      reply: printReply(
        button(),
        withStep(
          1,
          step('assistant-how-set-party', [
            ['role', text('seller')],
            ['party', { type: 'names', names: { en: 'Faisal' } }],
            ['code', text('7')],
          ]),
        ),
      ),
      tool_calls: [],
    },
    pOracle,
  );
  expect(
    !badRole.pass && badRole.reasons.some((x) => x.includes('he is a buyer')),
    'a buyer set as a seller in the steps passed',
  );
  const badDates = score(
    pq,
    {
      reply: printReply(
        button(),
        withStep(
          2,
          step('assistant-how-set-dates', [
            ['from', text('2026-10-01')],
            ['to', text('2026-10-30')],
          ]),
        ),
      ),
      tool_calls: [],
    },
    pOracle,
  );
  expect(
    !badDates.pass && badDates.reasons.some((x) => x.includes('steps set to 2026-10-30')),
    'steps with another end date passed',
  );
  const noPrint = score(
    pq,
    { reply: printReply(button(), rightSteps().slice(0, 3)), tool_calls: [] },
    pOracle,
  );
  expect(
    !noPrint.pass && noPrint.reasons.some((x) => x.includes('press-print')),
    'steps that do not end with press-print passed',
  );
  const notFirst = score(
    pq,
    {
      reply: answered(
        [],
        {},
        { sources: [{ kind: 'tiles', result_id: 'r1' }, button()], how: rightSteps() },
      ),
      tool_calls: [],
    },
    pOracle,
  );
  expect(
    !notFirst.pass && notFirst.reasons.some((x) => x.includes('first source')),
    'a right button that is not the first source passed',
  );
  const day = mk({
    filters: 'facet=print;pkind=buyer;pperiod=today',
    figures:
      'print.report=R35;print.party=Faisal;print.role=buyer;print.as_of=@cal.a;print.steps=1',
  });
  const dayButton = {
    kind: 'report',
    id: 'R35',
    filters: {
      as_of: '2026-10-01',
      from: '',
      to: '',
      party: { record: { name_en: 'Faisal', code: '7' } },
    },
  };
  const daySteps = [
    step('assistant-how-open-report', [['report', text('R35')]]),
    step('assistant-how-set-party', [
      ['role', text('buyer')],
      ['party', { type: 'names', names: { en: 'Faisal' } }],
      ['code', text('7')],
    ]),
    step('assistant-how-set-day', [['day', text('2026-10-01')]]),
    step('assistant-how-press-print'),
  ];
  expect(
    score(day, { reply: printReply(dayButton, daySteps), tool_calls: [] }, pOracle).pass,
    'a right day slip with its steps missed',
  );
  const sell = mk({
    filters: 'facet=print;pkind=seller;pperiod=month',
    figures:
      'print.report=R6;print.seller=Karim;print.role=seller;print.from=@cal.a;print.to=@cal.b;print.steps=1',
  });
  const sellButton = {
    kind: 'report',
    id: 'R6',
    filters: { from: '2026-10-01', to: '2026-10-31', seller: { name_en: 'Karim', code: '11' } },
  };
  const sellSteps = [
    step('assistant-how-open-report', [['report', text('R6')]]),
    step('assistant-how-set-party', [
      ['role', text('seller')],
      ['party', { type: 'names', names: { en: 'Karim' } }],
      ['code', text('11')],
    ]),
    step('assistant-how-set-dates', [
      ['from', text('2026-10-01')],
      ['to', text('2026-10-31')],
    ]),
    step('assistant-how-press-print'),
  ];
  expect(
    score(sell, { reply: printReply(sellButton, sellSteps), tool_calls: [] }, pOracle).pass,
    'a right seller print with its steps missed',
  );
  // No party (R34 or R2): the steps name none.
  const noParty = mk({
    filters: 'facet=print;pkind=all;pperiod=month',
    figures: 'print.report=R2;print.from=@cal.a;print.to=@cal.b;print.steps=1',
  });
  const wholeButton = {
    kind: 'report',
    id: 'R2',
    filters: { from: '2026-10-01', to: '2026-10-31' },
  };
  const wholeSteps = [
    step('assistant-how-open-report', [['report', text('R2')]]),
    step('assistant-how-set-dates', [
      ['from', text('2026-10-01')],
      ['to', text('2026-10-31')],
    ]),
    step('assistant-how-press-print'),
  ];
  expect(
    score(noParty, { reply: printReply(wholeButton, wholeSteps), tool_calls: [] }, pOracle).pass,
    'a right whole-sales print missed',
  );
  expect(
    !score(
      noParty,
      {
        reply: printReply(wholeButton, [
          wholeSteps[0],
          step('assistant-how-set-party', [['role', text('buyer')]]),
          ...wholeSteps.slice(1),
        ]),
        tool_calls: [],
      },
      pOracle,
    ).pass,
    'a party in the steps of a report with none passed',
  );
  // A name found in another role: judged on the first reply (outcome, roles) and, after the click on
  // his name, on his figures in his real role.
  const wrongRole = mk({
    outcome: 'not_found',
    filters: 'facet=nosuch;asked_role=seller;found_as=buyer',
    figures: 'qty=18;total=31680',
  });
  const firstReply = {
    outcome: 'not_found',
    answer: null,
    asked_role: 'seller',
    found_as: {
      key: 'a31',
      id: 31,
      role: 'buyer',
      name: { name_en: 'Faisal', name_ur: 'فیصل', tag: null, code: null },
    },
  };
  const afterClick = (sources) => answered([money(18), money(31680)], {}, { sources });
  const buyerSource = {
    kind: 'analytics',
    result_id: 'r1',
    view: 'C',
    filters: { buyer: { record: { name_en: 'Faisal' }, person_key: null }, seller: null },
  };
  const sellerSource = {
    kind: 'analytics',
    result_id: 'r1',
    view: 'B',
    filters: { seller: { name_en: 'Faisal' }, buyer: null },
  };
  expect(
    score(
      wrongRole,
      {
        first_reply: firstReply,
        reply: afterClick([buyerSource]),
        tool_calls: [],
        asked_role_field: true,
      },
      {},
    ).pass,
    'a buyer asked as a seller and read as a buyer missed',
  );
  const asSeller = score(
    wrongRole,
    {
      first_reply: firstReply,
      reply: afterClick([sellerSource]),
      tool_calls: [],
      asked_role_field: true,
    },
    {},
  );
  expect(
    !asSeller.pass && asSeller.reasons.some((x) => x.includes('after the click')),
    'a buyer read as a seller after the click passed',
  );
  const noAnswer = score(
    wrongRole,
    {
      first_reply: firstReply,
      reply: { outcome: 'which_one', answer: null },
      tool_calls: [],
      asked_role_field: true,
    },
    {},
  );
  expect(
    !noAnswer.pass && noAnswer.reasons.some((x) => x.includes('wanted answered')),
    'no figures after the click passed',
  );
  const wrongFound = score(
    wrongRole,
    {
      first_reply: { ...firstReply, found_as: { ...firstReply.found_as, role: 'seller' } },
      reply: afterClick([buyerSource]),
      tool_calls: [],
      asked_role_field: true,
    },
    {},
  );
  expect(
    !wrongFound.pass && wrongFound.reasons.some((x) => x.includes('found_as')),
    'a wrong found_as role passed',
  );
  expect(
    !score(
      wrongRole,
      {
        first_reply: firstReply,
        reply: answered([money(17), money(31680)]),
        tool_calls: [],
        asked_role_field: true,
      },
      {},
    ).pass,
    'a wrong qty after the click passed',
  );
  const sellerRole = mk({
    outcome: 'not_found',
    filters: 'facet=nosuch;asked_role=buyer;found_as=seller',
    figures: 'qty=87',
  });
  const karim = {
    outcome: 'not_found',
    answer: null,
    asked_role: 'buyer',
    found_as: { role: 'seller', name: { name_en: 'Karim' } },
  };
  const karimBuyer = { kind: 'analytics', filters: { buyer: { record: { name_en: 'Karim' } } } };
  expect(
    !score(
      sellerRole,
      {
        first_reply: karim,
        reply: answered([money(87)], {}, { sources: [karimBuyer] }),
        tool_calls: [],
        asked_role_field: true,
      },
      {},
    ).pass,
    'a seller read as a buyer after the click passed',
  );
  expect(
    score(
      sellerRole,
      {
        first_reply: karim,
        reply: answered([money(87)], {}, { sources: [] }),
        tool_calls: [],
        asked_role_field: true,
      },
      {},
    ).pass,
    'a seller read in his role missed',
  );
  const pv = new Map([
    [
      'P1',
      {
        pass: true,
        kind: 'PASS',
        outcome: 'answered',
        network: false,
        lost: false,
        pending: false,
      },
    ],
    [
      'P2',
      {
        pass: false,
        kind: 'PENDING',
        outcome: 'not_found',
        network: false,
        lost: false,
        pending: true,
      },
    ],
  ]);
  const pstats = categoryStats(
    'E',
    [
      { id: 'P1', cat: 'E' },
      { id: 'P2', cat: 'E' },
    ],
    pv,
    new Map([
      ['P1', { millis: 1000 }],
      ['P2', { millis: 1000 }],
    ]),
  );
  expect(pstats.n === 1 && pstats.passed === 1, 'a pending question counted in the category');
  // Results of a partial run go into the whole file, a new row for an old one.
  const dir = mkdtempSync(join(tmpdir(), 'ai-eval-'));
  const whole = join(dir, 'results.json');
  writeFileSync(
    whole,
    JSON.stringify({
      meta: { model: 'm' },
      oracle: { a: '1' },
      results: [
        { id: 'A', v: 1 },
        { id: 'B', v: 1 },
      ],
    }),
  );
  const merged = mergeResults(
    {
      meta: { model: 'm' },
      oracle: { b: '2' },
      results: [
        { id: 'B', v: 2 },
        { id: 'C', v: 1 },
      ],
    },
    whole,
  );
  expect(
    merged.results.map((r) => `${r.id}${r.v}`).join() === 'A1,B2,C1' &&
      merged.oracle.a === '1' &&
      merged.oracle.b === '2',
    'the merge is wrong',
  );
  expect(
    existsSync(join(dir, 'results.before-merge.json')),
    'the file before the merge was not kept',
  );
  // What the app read before the first call is not fixed.
  expect(
    preReads({
      tool_calls: [
        { id: 'r1', tool: 'period', prefetch: true },
        { id: 'r2', tool: 'analytics' },
      ],
      calls_json: JSON.stringify([{ tools: [{ name: 'analytics' }] }]),
    }) === 1,
    'a period read ahead was missed',
  );
  expect(
    preReads({
      tool_calls: [{ id: 'h1', tool: 'app_help' }],
      calls_json: JSON.stringify([{ tools: [] }]),
    }) === 1,
    'a guide read ahead was missed',
  );
  expect(
    preReads({
      tool_calls: [{ id: 'h1', tool: 'app_help' }],
      calls_json: JSON.stringify([{ tools: [{ name: 'app_help' }] }]),
    }) === 0,
    "the model's own guide read was taken for a read ahead",
  );
  expect(
    preReads({
      tool_calls: [{ id: 'r2', prefetch: true }],
      calls_json: JSON.stringify([{ tools: [{ name: 'party_card', prefetch: true }] }]),
    }) === 0,
    'a card read right after the find was taken for a read ahead',
  );
  // Selecting by category, world and id.
  expect(
    selectIds(list, { category: 'D' }).every((id) => byId(id).cat === 'D'),
    '--category chose a wrong question',
  );
  expect(
    selectIds(list, { world: 'thin' }).length > 0 &&
      selectIds(list, { only: 'Q001,Q002' }).length === 2,
    '--world or --only chose wrongly',
  );
  const itemIds = selectIds(list, { facet: 'ITEM' });
  const sample = selectIds(list, { facet: 'ITEM', every: '5' });
  expect(
    itemIds.length > 0 &&
      itemIds.every((id) => /(?:^|;)facet=ITEM(?:;|$)/.test(byId(id).filters)) &&
      sample.length === Math.ceil(itemIds.length / 5) &&
      sample[0] === itemIds[0] &&
      sample[1] === itemIds[5],
    '--facet or --every chose wrongly',
  );
  console.log(
    `self-test ok: the bank has ${list.length} questions in 5 categories and the scorer reads right, wrong, optional, zero, strict-name, general-advice and network cases`,
  );
}

// ---- main ----------------------------------------------------------------------------------

/**
 * --oracle-dump FILE: writes the oracle facts with their values, and the names of the parties,
 * items, partners, gardens and trips, as JSON. It runs one offline test through cargo (no model, no
 * key, no network): the books of the rich year are built again, which takes a few minutes.
 */
function oracleDump(out) {
  mkdirSync(dirname(out), { recursive: true });
  const env = { ...process.env, MANDI_ORACLE_JSON: out };
  const cmd = [
    'test',
    '-j',
    '4',
    '-p',
    'mandi-assistant',
    '--test',
    'robustness_live',
    '--',
    '--nocapture',
    '--exact',
    'dump_the_oracle_json',
  ];
  console.log(`ai-eval: cargo ${cmd.join(' ')} in ${BACKEND}`);
  const res = spawnSync('cargo', cmd, {
    cwd: BACKEND,
    env,
    stdio: ['ignore', 'inherit', 'inherit'],
  });
  if (res.error) throw new Error(`cannot start cargo: ${res.error.message}`);
  if (res.status !== 0) throw new Error(`the oracle dump failed with exit code ${res.status}`);
  if (!existsSync(out)) throw new Error(`no dump was written at ${out}`);
  const dump = JSON.parse(readFileSync(out, 'utf8'));
  const keys = manifestKeys();
  const missing = keys ? [...keys].filter((k) => !(k in dump.facts)) : [];
  if (missing.length)
    throw new Error(
      `the dump lacks ${missing.length} keys of oracle-keys.txt, for example ${missing.slice(0, 3).join(', ')}`,
    );
  console.log(
    `oracle dump: ${Object.keys(dump.facts).length} facts, ${dump.parties.length} parties, ${dump.items.length} items, ${dump.partners.length} partners, ${dump.gardens.length} gardens, ${dump.trips.length} trips -> ${out}`,
  );
}

/** The columns a bank must have for the scorer and the live test to read it. */
const BANK_COLUMNS = [
  'id',
  'group',
  'kind',
  'lang',
  'misspelled',
  'world',
  'after',
  'question',
  'outcome',
  'tools',
  'filters',
  'figures',
  'source',
  'cat',
  'section',
];

function main() {
  if (flag('--self-test')) return selfTest();
  if (option('--oracle-dump')) {
    try {
      return oracleDump(resolve(option('--oracle-dump')));
    } catch (error) {
      console.error(`ai-eval: ${error.message}`);
      process.exit(2);
    }
  }
  if (BANK) {
    if (!existsSync(BANK)) {
      console.error(`ai-eval: no bank file at ${BANK}`);
      process.exit(2);
    }
    const head = readFileSync(BANK, 'utf8').split('\n', 1)[0].split('\t');
    const lacking = BANK_COLUMNS.filter((c) => !head.includes(c));
    if (lacking.length) {
      console.error(`ai-eval: the bank ${BANK} lacks the columns: ${lacking.join(', ')}`);
      process.exit(2);
    }
  }
  const list = loadList();
  const { problems, summary } = checkList(list, Boolean(BANK));
  console.log(
    `ai-eval: bank ${QUESTIONS}${BANK ? ' (another bank: the per-row rules only; results in ' + OUT_DIR + ')' : ''}`,
  );
  const c = summary.byCat;
  console.log(
    `  ${summary.total} questions: A ${c.A}, B ${c.B}, C ${c.C}, D ${c.D}, E ${c.E}; Roman Urdu ${summary.languages.roman}, Urdu letters ${summary.languages.urdu}, English ${summary.languages.en}; ${summary.misspelled} misspelled; ${summary.groups} groups (${summary.bigGroups} of 3 or more)`,
  );
  const fc = summary.facets;
  console.log(
    `  facets: M (mixed language) ${fc.M}, a party's day ${fc.day}, no such party ${fc.nosuch}, print ${fc.print}, Umar's own (U) ${fc.U}, real-time figures (A2) ${fc.A2}, real figure plus forecast (MIX) ${fc.MIX}, capability cells (CAP) ${fc.CAP}, vegetables (VEG) ${fc.VEG}, the item axis (ITEM) ${fc.ITEM}, the money axis (MONEY) ${fc.MONEY}`,
  );
  if (problems.length) {
    console.error(`the bank breaks its rules:\n  ${problems.join('\n  ')}`);
    process.exit(2);
  }
  if (flag('--check')) {
    if (!BANK && WAITING.length)
      console.log(
        `  waiting for new oracle facts (the next oracle run brings them in): ${WAITING.join(', ')}`,
      );
    console.log(
      BANK
        ? 'the bank meets the per-row rules: unique ids, known follow-ups, outcomes, worlds and categories, and every @key is a fact of the oracle'
        : 'the bank meets its rules: the first 110 keep the D6 mix, all five categories, every section, every help topic, every @key is a fact of the oracle, and questions.tsv is what the spec builds',
    );
    return;
  }
  const file = option('--results');
  let ids = selectIds(list, {
    only: option('--only'),
    category: option('--category'),
    world: option('--world'),
    facet: option('--facet'),
    every: option('--every'),
  });
  // --misses asks again what missed or was lost to the network; --merge puts a partial run into the
  // whole results file, so the score is always of the whole bank.
  const asksAgain = flag('--misses');
  const merging = flag('--merge') || asksAgain;
  if (!file) {
    try {
      if (asksAgain) {
        ids = idsToAskAgain(list);
        console.log(`ai-eval: asking ${ids.length} questions again: ${ids.join(',')}`);
      }
      const part = merging ? join(OUT_DIR, 'results.part.json') : RESULTS;
      runLive(ids, option('--model'), option('--threads') ?? THREADS_DEFAULT, part);
      if (merging) {
        const rows = JSON.parse(readFileSync(part, 'utf8'));
        if (!rows.waiting_for_key) mergeResults(rows);
        else writeFileSync(RESULTS, JSON.stringify(rows));
      }
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
    console.log(
      'WAITING FOR KEY: no OpenRouter key is saved in the app. Umar pastes it in Settings, in the question group, then run this again.',
    );
    process.exit(3);
  }
  const shown = ids.length && !merging ? new Set(ids) : null;
  const scored = shown ? list.filter((q) => shown.has(q.id)) : list;
  const { text, meets, results, verdicts, stats, speed, budget, instant } = report(scored, data);
  console.log(text);
  writeReview(
    scored.filter((q) => results.has(q.id)),
    results,
  );
  writeDetail(scored, data, verdicts);
  mkdirSync(OUT_DIR, { recursive: true });
  writeFileSync(
    SUMMARY,
    JSON.stringify(
      {
        date: new Date().toISOString(),
        model: data.meta.model,
        categories: stats.map(({ networkTimes, ...s }) => ({
          ...s,
          networkQuestions: networkTimes.length,
        })),
        misses: [...verdicts.values()]
          .filter((v) => !v.pass && !v.lost && !v.pending)
          .map((v) => v.id),
        pending: [...verdicts.values()].filter((v) => v.pending).map((v) => v.id),
        budget: budget && {
          ok: budget.ok,
          groups: budget.groups.map((g) => ({
            name: g.name,
            n: g.n,
            p50: g.p50v,
            p95: g.p95v,
            over: g.over,
          })),
        },
        speed: Object.fromEntries(
          Object.entries(speed ?? {}).map(([k, st]) => [
            k,
            {
              questions: st.n,
              calls: st.by,
              oneOrNone: st.oneOrNone,
              stageMillis: Object.fromEntries(
                Object.entries(st.stage).map(([name, xs]) => [
                  name,
                  { p50: percentile(xs, 0.5), p95: percentile(xs, 0.95), n: xs.length },
                ]),
              ),
              reasoningTokens: Object.fromEntries(
                Object.entries(st.reasoning).map(([name, xs]) => [
                  name,
                  {
                    p50: percentile(xs, 0.5),
                    p95: percentile(xs, 0.95),
                    mean: Math.round(mean(xs)),
                  },
                ]),
              ),
            },
          ]),
        ),
        instant: Object.fromEntries(
          Object.entries(instant ?? {}).map(([k, st]) => [
            k,
            {
              questions: st.n,
              noModelCall: st.noCall,
              instantRight: st.instant,
              instantShare: st.instantShare,
              instantOrOneRight: st.oneOrNone,
              instantOrOneShare: st.oneShare,
              waitP50: st.p50,
              waitP95: st.p95,
              notRight: st.wrong,
              meetsTarget: st.ok,
            },
          ]),
        ),
        lost: [...verdicts.values()].filter((v) => v.lost).map((v) => v.id),
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
