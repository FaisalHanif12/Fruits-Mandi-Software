#!/usr/bin/env node
// Builds scripts/ai-eval/capability-items.tsv: the ITEM AXIS of the capability matrix (Umar, 06-11-2026), one line
// for each cell of measure x subject x period (see capability-items.mjs), and its bank question.
//
//   node scripts/ai-eval/build-capability-items.mjs                          write capability-items.tsv
//   node scripts/ai-eval/build-capability-items.mjs --check                  is it up to date, is every cell A, X, N or G
//   node scripts/ai-eval/build-capability-items.mjs --summary                counts, and the G and N lists with owners
//   node scripts/ai-eval/build-capability-items.mjs --sold-from <dump.json>  write item-axis-sold.json from an oracle dump
//
// A line of an X, N or G cell that is the same in every period of its measure is written once, with period `*`.

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  ITEMS,
  MEASURES,
  PARTY_ITEMS,
  PERIODS,
  SOLD_FILE,
  buildCells,
} from './capability-items.mjs';
import { MONEY_SUBJECTS, moneyCells } from './capability-money.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
export const ITEMS_FILE = join(HERE, 'capability-items.tsv');
export const MONEY_FILE = join(HERE, 'capability-money.tsv');
const HEAD = ['measure', 'subject', 'period', 'status', 'owner', 'note', 'group', 'lang', 'thing'];

/** The periods of the baselines too: the sold table holds them all. */
const ALL_PERIODS = [
  ...new Set([...PERIODS.map((p) => p.id), ...PERIODS.map((p) => p.prev).filter(Boolean)]),
];

function lines(cells) {
  const rows = [];
  const key = (c) => `${c.measure}|${c.subject}`;
  const byKey = new Map();
  for (const c of cells) {
    if (!byKey.has(key(c))) byKey.set(key(c), []);
    byKey.get(key(c)).push(c);
  }
  for (const group of byKey.values()) {
    const first = group[0];
    const same =
      first.status !== 'A' &&
      group.length > 1 &&
      group.every(
        (c) => c.status === first.status && c.owner === first.owner && c.note === first.note,
      );
    const list = same ? [{ ...first, period: '*' }] : group;
    for (const c of list)
      rows.push(
        [
          c.measure,
          c.subject,
          c.period,
          c.status,
          c.owner,
          c.note,
          c.status === 'A' ? c.id : '',
          c.lang ?? '',
          c.thing ?? '',
        ].join('\t'),
      );
  }
  return `${HEAD.join('\t')}\n${rows.join('\n')}\n`;
}

function summary(cells) {
  const out = [];
  const count = (f) => cells.filter(f).length;
  out.push(
    `item axis: ${cells.length} cells: A ${count((c) => c.status === 'A')}, X ${count((c) => c.status === 'X')}, N ${count((c) => c.status === 'N')}, G ${count((c) => c.status === 'G')}`,
  );
  const reasons = new Map();
  for (const c of cells.filter((c) => c.status === 'N' || c.status === 'G')) {
    const k = `${c.status} ${c.measure} x ${c.subject}${c.owner ? ` [${c.owner}]` : ''}: ${c.note}`;
    reasons.set(k, (reasons.get(k) ?? 0) + 1);
  }
  for (const [k, n] of reasons) out.push(`  ${k} (${n} cells)`);
  return out.join('\n');
}

function check(cells) {
  const problems = [];
  const ids = new Set();
  for (const c of cells) {
    if (ids.has(c.id)) problems.push(`${c.id}: used twice`);
    ids.add(c.id);
    if (!['A', 'X', 'N', 'G'].includes(c.status)) problems.push(`${c.id}: status ${c.status}`);
    if (c.status === 'G' && !c.owner) problems.push(`${c.id}: a gap with no owner`);
    if ((c.status === 'N' || c.status === 'X' || c.status === 'G') && !c.note)
      problems.push(`${c.id}: no reason`);
    if (c.status === 'A' && (!c.entry || c.entry.q.length !== 1))
      problems.push(`${c.id}: no question`);
  }
  // Every measure has a cell for every subject.
  for (const m of MEASURES)
    for (const s of [
      'fruit',
      'veg',
      'g_fruit',
      'g_veg',
      'all',
      'party',
      'quality',
      'unknown',
      'zero',
    ])
      if (!cells.some((c) => c.measure === m.id && c.subject === s))
        problems.push(`measure ${m.id} has no cell for subject ${s}`);
  // The wordings rotate: every language kind is used, and some are misspelled.
  const answerable = cells.filter((c) => c.status === 'A');
  for (const lang of ['roman', 'urdu', 'en'])
    if (!answerable.some((c) => c.lang === lang)) problems.push(`no ${lang} wording`);
  if (!answerable.some((c) => c.entry.q[0][2] === 'mis')) problems.push('no misspelled wording');
  return problems;
}

const MONEY_HEAD = [
  'subject',
  'qtype',
  'period',
  'status',
  'owner',
  'note',
  'group',
  'lang',
  'thing',
];

function moneyLines(cells) {
  const rows = cells.map((c) =>
    [
      c.subject,
      c.qtype,
      c.period,
      c.status,
      c.owner,
      c.note,
      c.status === 'A' ? c.id : '',
      c.lang ?? '',
      c.thing ?? '',
    ].join('\t'),
  );
  return `${MONEY_HEAD.join('\t')}\n${rows.join('\n')}\n`;
}

function moneySummary(cells) {
  const count = (f) => cells.filter(f).length;
  const out = [
    `money axis: ${cells.length} cells: A ${count((c) => c.status === 'A')}, X ${count((c) => c.status === 'X')}, N ${count((c) => c.status === 'N')}, G ${count((c) => c.status === 'G')}`,
  ];
  const reasons = new Map();
  for (const c of cells.filter((c) => c.status === 'N' || c.status === 'G')) {
    const k = `${c.status} ${c.subject} ${c.qtype}${c.owner ? ` [${c.owner}]` : ''}: ${c.note}`;
    reasons.set(k, (reasons.get(k) ?? 0) + 1);
  }
  for (const [k, n] of reasons) out.push(`  ${k} (${n} cells)`);
  return out.join('\n');
}

function moneyCheck(cells) {
  const problems = [];
  const ids = new Set();
  for (const c of cells) {
    if (ids.has(c.id)) problems.push(`${c.id}: used twice`);
    ids.add(c.id);
    if (!['A', 'X', 'N', 'G'].includes(c.status)) problems.push(`${c.id}: status ${c.status}`);
    if (c.status === 'G' && !c.owner) problems.push(`${c.id}: a gap with no owner`);
    if (c.status !== 'A' && !c.note) problems.push(`${c.id}: no reason`);
    if (c.status === 'A' && (!c.entry || c.entry.q.length !== 1))
      problems.push(`${c.id}: no question`);
  }
  for (const s of MONEY_SUBJECTS)
    for (const qt of ['F', 'L', 'C', 'T'])
      if (!cells.some((c) => c.subject === s.id && c.qtype === qt))
        problems.push(`subject ${s.id} has no cell for ${qt}`);
  const answerable = cells.filter((c) => c.status === 'A');
  for (const lang of ['roman', 'urdu', 'en'])
    if (!answerable.some((c) => c.lang === lang)) problems.push(`no ${lang} wording`);
  if (!answerable.some((c) => c.entry.q[0][2] === 'mis')) problems.push('no misspelled wording');
  return problems;
}

function soldFrom(dumpFile) {
  const dump = JSON.parse(readFileSync(dumpFile, 'utf8'));
  const facts = dump.facts ?? dump;
  const value = (k) => {
    const v = facts[k];
    return v && typeof v === 'object' ? v.value : v;
  };
  const periods = {};
  const names = ITEMS.map((i) => i.key);
  for (const p of ALL_PERIODS) {
    // The item axis facts hold every item; the older `a.` facts hold only the items with sales, and only for
    // the periods the bank asked before.
    if (names.some((n) => value(`ix.${p}.${n}.gross`) !== undefined)) {
      periods[p] = names.filter((n) => Number(value(`ix.${p}.${n}.gross`) ?? 0) > 0);
    } else if (Object.keys(facts).some((k) => k.startsWith(`a.${p}.`))) {
      periods[p] = names.filter((n) => Number(value(`a.${p}.${n}.gross`) ?? 0) > 0);
    } else {
      periods[p] = null;
    }
  }
  // The items Faisal bought in each period (his rows of the item axis).
  const party = {};
  for (const p of ALL_PERIODS)
    party[p] = PARTY_ITEMS.filter((n) => Number(value(`ix.${p}.${n}.faisal.gross`) ?? 0) > 0);
  // The qualities of the sale lines in each period, by item (plan 58 T4: the oracle's `ixq.*` facts); null when the
  // dump has no quality facts.
  const quality = {};
  for (const p of ALL_PERIODS) {
    const byItem = {};
    for (const n of names) {
      const list = ['first', 'second', 'third'].filter(
        (q) => Number(value(`ixq.${p}.${n}.${q}.lines`) ?? 0) > 0,
      );
      if (list.length) byItem[n] = list;
    }
    quality[p] = Object.keys(facts).some((k) => k.startsWith(`ixq.${p}.`)) ? byItem : null;
  }
  const stock0 = names.filter(
    (n) => Number(value(`ix.stock.${n}`) ?? value(`stock.${n}.qty`) ?? 0) === 0,
  );
  writeFileSync(
    SOLD_FILE,
    `${JSON.stringify({ periods, party, quality, stock0 }, null, 2)}\n`,
    'utf8',
  );
  const known = Object.values(periods).filter(Boolean).length;
  console.log(
    `wrote ${SOLD_FILE}: ${known} of ${ALL_PERIODS.length} periods known, ${stock0.length} items with no stock`,
  );
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const args = process.argv.slice(2);
  const at = args.indexOf('--sold-from');
  if (at >= 0) {
    soldFrom(resolve(args[at + 1]));
  } else {
    const cells = buildCells();
    const text = lines(cells);
    const money = moneyCells();
    const moneyText = moneyLines(money);
    if (args.includes('--summary')) {
      console.log(summary(cells));
      console.log(moneySummary(money));
    } else if (args.includes('--check')) {
      const problems = [...check(cells), ...moneyCheck(money)];
      let moneyNow = '';
      try {
        moneyNow = readFileSync(MONEY_FILE, 'utf8');
      } catch {
        moneyNow = '';
      }
      if (moneyNow !== moneyText)
        problems.push(
          'capability-money.tsv is not what the builder writes: run build-capability-items.mjs',
        );
      let now = '';
      try {
        now = readFileSync(ITEMS_FILE, 'utf8');
      } catch {
        now = '';
      }
      if (now !== text)
        problems.push(
          'capability-items.tsv is not what the builder writes: run build-capability-items.mjs',
        );
      if (problems.length) {
        console.error(problems.join('\n'));
        process.exit(2);
      }
      console.log(summary(cells).split('\n')[0]);
      console.log(moneySummary(money).split('\n')[0]);
    } else {
      writeFileSync(ITEMS_FILE, text, 'utf8');
      writeFileSync(MONEY_FILE, moneyText, 'utf8');
      console.log(`wrote ${ITEMS_FILE} and ${MONEY_FILE}`);
      console.log(summary(cells));
      console.log(moneySummary(money));
    }
  }
}
