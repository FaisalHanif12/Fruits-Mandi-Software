#!/usr/bin/env node
// Builds scripts/ai-eval/capability.tsv (plan 55 Section 11): the capability matrix of the whole
// app. One line for each row (a report, a view, a tile, a record, a figure, a guide topic ...) and
// each question type, with what the assistant can do, and the bank questions that already reach it.
//
//   node scripts/ai-eval/build-capability.mjs          write capability.tsv
//   node scripts/ai-eval/build-capability.mjs --check  say whether capability.tsv is up to date and
//                                                      whether any cell is empty or any gap unknown
//   node scripts/ai-eval/build-capability.mjs --todo   list the answerable cells with no bank question
//
// The rows are in capability-rows.mjs, read from the code. The bank questions are matched to cells
// from their own fields (tools, kind, outcome, filters, figures), so the count is a guide, not a
// proof: a cell with a count of 0 is certainly not covered; a cell with a count is probably covered.
// Nothing here calls a model.

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { GAPS, QTYPES, ROWS } from './capability-rows.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
export const CAPABILITY_FILE = join(HERE, 'capability.tsv');
const BANK = join(HERE, 'questions.tsv');

// ---- the bank, tagged with the cells it reaches --------------------------------------------------------

function readBank() {
  const lines = readFileSync(BANK, 'utf8')
    .split('\n')
    .filter((l) => l.trim() !== '');
  const head = lines[0].split('\t');
  return lines.slice(1).map((l) => {
    const cols = l.split('\t');
    return Object.fromEntries(head.map((h, i) => [h, cols[i] ?? '']));
  });
}

const TILE_OF_KEY = [
  [/^tiles\.sales_today|^tiles\.count_today/, 'tile-today_sales'],
  [/^tiles\.commission_today/, 'tile-today_commission'],
  [/^tiles\.cash|^cash\b/, 'tile-cash_in_hand'],
  [/^tiles\.buyers_dues/, 'tile-buyers_dues'],
  [/^tiles\.total_dues/, 'tile-total_dues'],
  [/^tiles\.dead/, 'tile-dead_amount'],
  [/^tiles\.sellers_payable/, 'tile-sellers_payable'],
  [/^tiles\.sellers_owe/, 'tile-sellers_dues'],
  [/^tiles\.advances_open/, 'tile-advances_open'],
  [/^tiles\.stock/, 'tile-stock'],
  [/^tiles\.trips/, 'tile-trips'],
  [/^tiles\.gardens/, 'tile-gardens'],
  [/^tiles\.lent/, 'tile-money_lent_open'],
  [/^tiles\.expenses_today/, 'tile-today_expenses'],
];

const RE_COMPARE =
  /(zyada|zyaada|ziyada|kam hai|compare|\bvs\b|versus|farq|than last|more than|less than|difference|پچھلے .*سے|زیادہ|بمقابلہ|فرق)/i;
const RE_TOP =
  /(sab se|sabse|top\b|most\b|biggest|highest|lowest|ranking|best\b|worst\b|سب سے|پہلے کس|first\b|pehle kis)/i;
const RE_LIST =
  /(\bsab\b|saare|sare|\ball\b|\blist\b|kaun kaun|fehrist|tamam|فہرست|کون کون|سب |تمام|har ek|each)/i;

/** The cells (rowId|qtype) one bank question reaches. */
function cellsOf(q) {
  const out = new Set();
  // The generator of bank-spec-cap-rest.mjs asks which cells have no question OTHER than its own.
  if (process.env.CAP_IGNORE_CX && /#rest$/.test(q.source ?? '')) return out;
  const add = (row, qtype) => out.add(`${row}|${qtype}`);
  const f = q.filters;
  const figs = q.figures;
  const topic = (f.match(/topic=([a-z_]+)/) ?? [])[1];
  // A question written for a cell says which one in its `source` ("capability.tsv R20 / list").
  const named = (q.source ?? '').match(/^capability\.tsv (\S+) \/ (\w+)( #rest)?$/);
  if (named) {
    add(named[1], named[2]);
    return out;
  }
  // The groups of facet MIX (one question: a real figure and a forecast) fill the mixed cell of their rows.
  const MIX_CELLS = {
    MX01: ['record-expense_head'],
    MX02: ['view-A', 'record-item'],
    MX03: ['record-buyer', 'R20', 'tile-buyers_dues'],
    MX04: ['tile-today_sales'],
    MX05: ['view-G'],
    MX06: ['tile-today_commission', 'R28'],
    MX07: ['R12', 'tile-cash_in_hand'],
    MX08: ['view-B', 'record-seller'],
    MX09: ['R30'],
    MX10: ['R29', 'tile-today_expenses'],
  };
  if (MIX_CELLS[q.group]) {
    for (const row of MIX_CELLS[q.group]) add(row, 'mixed');
    return out;
  }
  if (q.cat === 'D' || q.kind === 'help') {
    // A question that asks to OPEN a screen has route= beside its topic (facet CAP).
    if (topic) add(`topic-${topic}`, /(?:^|;)route=/.test(f) ? 'print' : 'howto');
    // Opening the dashboard is the "print or open" of every tile.
    if (topic === 'dashboard' && /(?:^|;)route=dashboard/.test(f))
      for (const row of ROWS) if (row.kind === 'tile') add(row.id, 'print');
    return out;
  }
  const facet = (f.match(/(?:^|;)facet=([A-Za-z0-9]+)/) ?? [])[1];
  const printReport = (figs.match(/print\.report=([R0-9|]+)/) ?? [])[1];
  if (facet === 'print' || q.kind === 'print') {
    for (const r of (printReport ?? '').split('|').filter(Boolean)) add(`${r}`, 'print');
    const role = (figs.match(/print\.role=([a-z_]+)/) ?? [])[1];
    if (role) add(`record-${role}`, 'print');
    return out;
  }
  if (q.outcome === 'change_refused') {
    const route = (f.match(/route=([a-z_]+)/) ?? [])[1];
    const t = ROWS.find((r) => r.kind === 'topic' && r.name.includes(`screen ${route})`));
    if (t) add(t.id, 'change');
    if (route === 'kharcha') add('record-expense_head', 'change');
    if (route === 'wasooli') add('record-buyer', 'change');
    if (route === 'adaigi' || route === 'peshgi') add('record-seller', 'change');
    if (route === 'sarmaya' || route === 'phal') add('record-partner', 'change');
    return out;
  }
  if (q.outcome === 'outside_books' || q.outcome === 'safe') {
    const text = q.question.toLowerCase();
    const k = /mausam|موسم|weather|barish|rain/.test(text)
      ? 'weather'
      : /gst|tax|ٹیکس/.test(text)
        ? 'tax'
        : /bank|بینک|account balance/.test(text)
          ? 'bank'
          : /lahore|rate in|mandi ka rate|market rate|بازار|ریٹ/.test(text)
            ? 'market_rate'
            : /dollar|gold|petrol|sona|ڈالر|سونا|stock market/.test(text)
              ? 'prices'
              : 'other';
    add(`outside-${k}`, 'outside');
    return out;
  }
  if (q.outcome === 'not_found' || q.outcome === 'which_one' || facet === 'nosuch') {
    const role = (f.match(/asked_role=([a-z_]+)/) ?? [])[1];
    if (role) add(`record-${role}`, 'nosuch');
    else add('record-buyer', 'nosuch');
    return out;
  }
  if (q.cat === 'B') {
    const t = q.tools;
    for (const m of t.matchAll(/report:(R\d+)/g)) add(`${m[1]}`, 'advice');
    for (const m of t.matchAll(/analytics:([A-K])/g)) add(`view-${m[1]}`, 'advice');
    if (/tiles/.test(t)) add('tile-cash_in_hand', 'advice');
    if (!t) add('tile-total_dues', 'advice');
    return out;
  }
  // The qtype of a figure question.
  let qtype = 'figure';
  if (q.cat === 'C' || q.kind === 'estimates' || q.kind === 'whatif') qtype = 'forecast';
  else if (facet === 'day' || q.kind === 'partyday') qtype = 'day';
  else if (RE_COMPARE.test(q.question)) qtype = 'compare';
  else if (RE_TOP.test(q.question) || /(^|;)first=/.test(figs)) qtype = 'top';
  else if (
    RE_LIST.test(q.question) &&
    !/kitna|kitni|how much|کتنا|کتنی/.test(q.question.toLowerCase())
  )
    qtype = 'list';
  const rows = new Set();
  for (const m of q.tools.matchAll(/report:(R\d+)/g)) rows.add(`${m[1]}`);
  for (const m of q.tools.matchAll(/analytics:([A-K])/g)) rows.add(`view-${m[1]}`);
  for (const key of figs.matchAll(/@([A-Za-z0-9_.]+)/g)) {
    const k = key[1];
    for (const [re, id] of TILE_OF_KEY) if (re.test(k)) rows.add(id);
    const head = k.split('.')[0];
    const rid = /^r(\d+)$/.test(head) ? `R${head.slice(1)}` : null;
    if (rid) rows.add(rid);
    if (head === 'a') rows.add('view-A');
    if (head === 'b') rows.add('view-B');
    if (head === 'c') rows.add('view-C');
    if (head === 'f') rows.add('view-F');
    if (head === 'g') rows.add('view-G');
    if (head === 'i') rows.add('view-I');
    if (head === 'j') rows.add('view-J');
    if (head === 'k') rows.add('view-K');
    if (head === 'buyer') rows.add('record-buyer');
    if (head === 'seller') rows.add('record-seller');
    if (head === 'partner') (rows.add('record-partner'), rows.add('figure-partner_total'));
    if (head === 'stock') rows.add('record-item');
    if (head === 'top') rows.add('view-H');
    if (head === 'buyer_day') rows.add('record-buyer');
    if (head === 'seller_day') rows.add('record-seller');
    if (/^trip\d/.test(head)) (rows.add('record-trip'), rows.add('figure-trip_result'));
    if (head === 'est' || head === 'est_item') rows.add('view-G');
  }
  if (/item=/.test(f)) rows.add('record-item');
  if (/buyer=/.test(f)) rows.add('record-buyer');
  if (/seller=/.test(f)) rows.add('record-seller');
  if (/partner=/.test(f)) rows.add('record-partner');
  if (/garden=/.test(f)) rows.add('record-garden');
  if (/trip=/.test(f)) rows.add('record-trip');
  if (/fruit/.test(q.section)) rows.add('figure-fruit_partner');
  if (/capital/.test(q.section)) rows.add('figure-partner_capital');
  if (/expense/.test(q.section)) (rows.add('record-expense_head'), rows.add('R29'));
  if (/lent/.test(q.kind)) (rows.add('R32'), rows.add('record-money_lent'));
  for (const r of rows) add(r, qtype);
  return out;
}

// ---- the matrix ----------------------------------------------------------------------------------------

const parseCell = (text) => {
  const status = text[0];
  const rest = text.slice(2);
  if (status === 'G') {
    const [id, ...what] = rest.split(':');
    return { status, id, what: what.join(':') };
  }
  return { status, what: rest };
};

export function matrix() {
  const bank = readBank();
  const covered = new Map();
  for (const q of bank)
    for (const key of cellsOf(q)) covered.set(key, [...(covered.get(key) ?? []), q.id]);
  const lines = [];
  const problems = [];
  for (const row of ROWS) {
    for (const qtype of QTYPES) {
      let text = row.cells[qtype];
      if (!text) {
        problems.push(`${row.id}: the cell ${qtype} is empty`);
        continue;
      }
      // A mixed question (a real figure and a forecast in one) follows the forecast of its row.
      if (qtype === 'mixed' && text.startsWith('X:') && row.cells.forecast) {
        const fc = row.cells.forecast;
        if (fc.startsWith('A:')) text = `A:${fc.slice(2)} + the real figure in ONE question`;
        else if (fc.startsWith('G:')) text = fc;
        else if (fc.startsWith('N:')) text = `N:${fc.slice(2)}`;
      }
      const cell = parseCell(text);
      if (!'AGNX'.includes(cell.status))
        problems.push(`${row.id}: ${qtype} has the status ${cell.status}`);
      if (cell.status === 'G' && !GAPS[cell.id])
        problems.push(`${row.id}: ${qtype} names the unknown gap ${cell.id}`);
      const ids = covered.get(`${row.id}|${qtype}`) ?? [];
      const gap = cell.status === 'G' ? GAPS[cell.id] : null;
      lines.push(
        [
          row.id,
          row.kind,
          row.name,
          qtype,
          cell.status,
          cell.what,
          cell.status === 'G' ? cell.id : '',
          gap ? gap[1] : '',
          gap ? gap[0] : '',
          String(ids.length),
          ids.slice(0, 8).join(','),
          row.source,
        ].join('\t'),
      );
    }
  }
  const header = [
    'row',
    'kind',
    'name',
    'qtype',
    'status',
    'route_or_what',
    'gap',
    'owner',
    'severity',
    'bank_n',
    'bank_ids',
    'source',
  ];
  return { text: [header.join('\t'), ...lines].join('\n') + '\n', lines, problems };
}

function summary(lines) {
  const by = {};
  for (const l of lines) {
    const c = l.split('\t');
    const key = c[4];
    by[key] = (by[key] ?? 0) + 1;
  }
  return by;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const { text, lines, problems } = matrix();
  if (problems.length) {
    console.error(`the matrix is not whole:\n  ${problems.join('\n  ')}`);
    process.exit(2);
  }
  if (process.argv.includes('--check')) {
    let have = '';
    try {
      have = readFileSync(CAPABILITY_FILE, 'utf8');
    } catch {
      /* missing */
    }
    if (have !== text) {
      console.error(
        'capability.tsv is not what the rows and the bank build: run node scripts/ai-eval/build-capability.mjs',
      );
      process.exit(2);
    }
    console.log(
      `capability.tsv is up to date: ${lines.length} cells, ${JSON.stringify(summary(lines))}`,
    );
    process.exit(0);
  }
  if (process.argv.includes('--todo')) {
    const todo = lines.map((l) => l.split('\t')).filter((c) => c[4] === 'A' && c[9] === '0');
    for (const c of todo) console.log(`${c[0]}\t${c[3]}\t${c[5]}`);
    console.log(`${todo.length} answerable cells have no bank question`);
    process.exit(0);
  }
  const outAt = process.argv.indexOf('--out');
  const target = outAt >= 0 ? process.argv[outAt + 1] : CAPABILITY_FILE;
  writeFileSync(target, text, 'utf8');
  const todo = lines.map((l) => l.split('\t')).filter((c) => c[4] === 'A' && c[9] === '0').length;
  console.log(
    `wrote ${target}: ${lines.length} cells ${JSON.stringify(summary(lines))}; answerable with no bank question: ${todo}`,
  );
}
