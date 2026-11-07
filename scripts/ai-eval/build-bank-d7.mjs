#!/usr/bin/env node
// Builds the D7 bank (plan 59 task B1) from bank-spec-d7.mjs:
//
//   scripts/ai-eval/realtime-d7.tsv            the questions (the same columns as questions.tsv)
//   scripts/ai-eval/realtime-d7-coverage.md    every kind of record and figure the app keeps, with the question
//                                              ids that cover it, and the size of the bank by area and language
//
//   node scripts/ai-eval/build-bank-d7.mjs           write both files
//   node scripts/ai-eval/build-bank-d7.mjs --check   say whether both files are what the spec builds
//
// Nothing here calls a model. `run.mjs --bank scripts/ai-eval/realtime-d7.tsv --check` checks the rows (and that
// every @key is a fact of the oracle).

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { COVERAGE, D7 } from './bank-spec-d7.mjs';
import { COLUMNS } from './build-bank.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
export const BANK_D7 = join(HERE, 'realtime-d7.tsv');
export const COVERAGE_D7 = join(HERE, 'realtime-d7-coverage.md');

/** The rows of the bank and the id of each group's questions. */
export function buildRows() {
  const rows = [];
  const idsOf = new Map();
  const clean = (s) => {
    if (/[\t\r\n]/.test(s)) throw new Error(`a tab or a line break in "${s}"`);
    return s;
  };
  let next = 1;
  for (const entry of D7) {
    const wordings = (entry.use ?? entry.q.map((_, i) => i)).map((i) => entry.q[i]);
    if (wordings.some((w) => !w)) throw new Error(`${entry.g}: a wording in "use" is not there`);
    if (idsOf.has(entry.g)) throw new Error(`the group key ${entry.g} is used twice`);
    const ids = [];
    for (const [lang, text, flag] of wordings) {
      const id = `D${String(next).padStart(3, '0')}`;
      next += 1;
      ids.push(id);
      // `mix=short`: one figure asked (the common short question); `mix=picky`: several figures, or none (a refusal).
      const asked = (entry.figures ?? '')
        .split(';')
        .filter((piece) => piece && !/^[^=]*\?=/.test(piece)).length;
      const mix = asked === 1 ? 'short' : 'picky';
      const row = {
        id,
        group: entry.g,
        kind: entry.kind,
        lang,
        misspelled: flag === 'mis' ? '1' : '0',
        world: entry.world ?? 'year',
        after: '',
        question: text,
        outcome: entry.outcome ?? 'answered',
        tools: entry.tools ?? '',
        filters: `${entry.filters ?? ''};mix=${mix}`,
        figures: entry.figures ?? '',
        source: entry.source ?? 'plan 59 B1',
        cat: entry.cat ?? 'A',
        section: entry.section,
      };
      rows.push(COLUMNS.map((c) => clean(String(row[c]))).join('\t'));
    }
    idsOf.set(entry.g, ids);
  }
  return { rows, idsOf };
}

/** "D001-D003" for consecutive ids, "D001, D004" otherwise. */
function span(ids) {
  const nums = [...new Set(ids)].map((i) => Number(i.slice(1))).sort((a, b) => a - b);
  const parts = [];
  let start = nums[0];
  let prev = nums[0];
  for (const n of [...nums.slice(1), Infinity]) {
    if (n !== prev + 1) {
      const f = (x) => `D${String(x).padStart(3, '0')}`;
      parts.push(start === prev ? f(start) : `${f(start)}-${f(prev)}`);
      start = n;
    }
    prev = n;
  }
  return parts.join(', ');
}

export function buildText() {
  const { rows } = buildRows();
  return [COLUMNS.join('\t'), ...rows].join('\n') + '\n';
}

export function buildCoverage() {
  const { rows, idsOf } = buildRows();
  const cells = rows.map((r) => Object.fromEntries(r.split('\t').map((v, i) => [COLUMNS[i], v])));
  const lines = [];
  const say = (s = '') => lines.push(s);
  say('# D7 bank: every kind of record and figure the app keeps');
  say();
  say(
    'Written by `scripts/ai-eval/build-bank-d7.mjs` from `bank-spec-d7.mjs`. Do not edit by hand. The bank is `realtime-d7.tsv`.',
  );
  say(
    'The kinds come from the PRD (Sections 6, 9 to 22, the report catalogue R1 to R38, the analytics views A to K) and the tables of Architecture 15. Every kind has question ids, or a reason why it has none.',
  );
  say();
  say('| # | Kind of record or figure | Where it comes from | Questions (ids) |');
  say('|---|---|---|---|');
  let gaps = 0;
  COVERAGE.forEach((c, i) => {
    const ids = (c.groups ?? []).flatMap((g) => {
      const found = idsOf.get(g);
      if (!found)
        throw new Error(`coverage "${c.kind}" names the group ${g}, which is not in the bank`);
      return found;
    });
    let cell;
    if (ids.length) cell = span(ids) + (c.note ? ` (${c.note})` : '');
    else if (c.reason) cell = `none: ${c.reason}`;
    else {
      cell = 'NONE';
      gaps += 1;
    }
    say(`| ${i + 1} | ${c.kind} | ${c.from} | ${cell} |`);
  });
  say();
  if (gaps) throw new Error(`${gaps} kinds have neither a group nor a reason`);

  const by = (key) => {
    const out = new Map();
    for (const q of cells) out.set(q[key], (out.get(q[key]) ?? 0) + 1);
    return out;
  };
  say('## Common short questions (one figure asked), by area');
  say();
  say('| Area | Groups | Questions (ids) |');
  say('|---|---|---|');
  const shortBy = new Map();
  for (const entry of D7) {
    const ids = idsOf.get(entry.g);
    const first = cells.find((q) => q.id === ids[0]);
    if (!/(?:^|;)mix=short(?:;|$)/.test(first.filters)) continue;
    const bucket = shortBy.get(entry.section) ?? { groups: 0, ids: [] };
    bucket.groups += 1;
    bucket.ids.push(...ids);
    shortBy.set(entry.section, bucket);
  }
  for (const [section, bucket] of [...shortBy.entries()].sort()) {
    say(`| ${section} | ${bucket.groups} | ${span(bucket.ids)} |`);
  }
  say();
  const shortCount = cells.filter((q) => /(?:^|;)mix=short(?:;|$)/.test(q.filters)).length;
  say(
    `${shortCount} of ${cells.length} questions (${((100 * shortCount) / cells.length).toFixed(0)}%) ask one figure in a plain wording (mix=short); the rest ask several figures or several filters at once (mix=picky). Run one kind with \`--mix short\` or \`--mix picky\`.`,
  );
  say();
  say('## Size of the bank');
  say();
  say(`${cells.length} questions in ${D7.length} groups.`);
  say();
  const lang = by('lang');
  const mis = cells.filter((q) => q.misspelled === '1').length;
  say(
    `By language: Roman Urdu ${lang.get('roman') ?? 0}, Urdu letters ${lang.get('urdu') ?? 0}, English ${lang.get('en') ?? 0}. Misspelt on purpose: ${mis} (${((100 * mis) / cells.length).toFixed(0)}%).`,
  );
  say();
  say('| Area (section) | Questions | Roman Urdu | Urdu | English | Misspelt |');
  say('|---|---|---|---|---|---|');
  const sections = [...by('section').keys()].sort();
  for (const sec of sections) {
    const xs = cells.filter((q) => q.section === sec);
    const k = (l) => xs.filter((q) => q.lang === l).length;
    const m = xs.filter((q) => q.misspelled === '1').length;
    say(`| ${sec} | ${xs.length} | ${k('roman')} | ${k('urdu')} | ${k('en')} | ${m} |`);
  }
  say();
  const complex = cells.filter(
    (q) =>
      (
        q.filters.match(
          /(?:^|;)(buyer|seller|item|group|grade|pay|day|period|partner|head|person|garden|trip|truck)=/g,
        ) ?? []
      ).length >= 3,
  ).length;
  say(`Questions with three or more filters at once: ${complex}.`);
  say();
  say('## Groups');
  say();
  say('| Group | Area | Kind | Questions | What it asks (first wording) |');
  say('|---|---|---|---|---|');
  for (const entry of D7) {
    const ids = idsOf.get(entry.g);
    const first = cells.find((q) => q.id === ids[0]);
    say(`| ${entry.g} | ${entry.section} | ${entry.kind} | ${span(ids)} | ${first.question} |`);
  }
  say();
  return lines.join('\n') + '\n';
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const text = buildText();
  const coverage = buildCoverage();
  if (process.argv.includes('--check')) {
    let bad = false;
    for (const [file, want] of [
      [BANK_D7, text],
      [COVERAGE_D7, coverage],
    ]) {
      let now = '';
      try {
        now = readFileSync(file, 'utf8');
      } catch {
        /* a missing file is a difference */
      }
      if (now !== want) {
        console.error(`${file} is not what bank-spec-d7.mjs builds: run build-bank-d7.mjs`);
        bad = true;
      }
    }
    if (bad) process.exit(2);
    console.log('realtime-d7.tsv and realtime-d7-coverage.md are what the spec builds');
  } else {
    writeFileSync(BANK_D7, text, 'utf8');
    writeFileSync(COVERAGE_D7, coverage, 'utf8');
    console.log(`wrote ${BANK_D7}: ${text.split('\n').length - 2} questions; and ${COVERAGE_D7}`);
  }
}
