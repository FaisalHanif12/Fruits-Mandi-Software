#!/usr/bin/env node
// Builds scripts/ai-eval/questions.tsv (plan 52 item 13): the frozen 110 questions of
// questions-110.tsv, then the new ones of bank-spec.mjs, one row for each wording.
//
//   node scripts/ai-eval/build-bank.mjs          write questions.tsv
//   node scripts/ai-eval/build-bank.mjs --check  say whether questions.tsv is what the spec builds
//
// `run.mjs --check` makes the same comparison, so the file cannot drift from the spec. Edit the
// spec (or questions-110.tsv), then run this once. Nothing here calls a model.

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import ENTRIES from './bank-spec.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
export const COLUMNS = [
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

/** The text of the whole file, as the spec builds it. */
export function buildText() {
  const frozen = readFileSync(join(HERE, 'questions-110.tsv'), 'utf8')
    .split('\n')
    .filter((l) => l.trim() !== '');
  const head = frozen[0].split('\t');
  if (head.join('\t') !== COLUMNS.join('\t'))
    throw new Error('questions-110.tsv has other columns than the bank');
  const lines = [COLUMNS.join('\t'), ...frozen.slice(1)];
  let next = frozen.length; // the frozen file has 110 rows after its header: the next id is Q111
  const firstId = new Map();
  const clean = (s) => {
    if (/[\t\r\n]/.test(s)) throw new Error(`a tab or a line break in "${s}"`);
    return s;
  };
  for (const entry of ENTRIES) {
    const ids = [];
    const wordings = (entry.use ?? entry.q.map((_, i) => i)).map((i) => entry.q[i]);
    if (wordings.some((w) => !w)) throw new Error(`${entry.g}: a wording in "use" is not there`);
    for (const [lang, text, flag] of wordings) {
      const id = `Q${String(next).padStart(3, '0')}`;
      next += 1;
      ids.push(id);
      const after = entry.after ? firstId.get(entry.after) : '';
      if (entry.after && !after)
        throw new Error(`${entry.g} follows ${entry.after}, not built yet`);
      const row = {
        id,
        group: wordings.length >= 2 ? entry.g : '',
        kind: entry.kind,
        lang,
        misspelled: flag === 'mis' ? '1' : '0',
        world: entry.world ?? 'year',
        after: after ?? '',
        question: text,
        outcome: entry.outcome ?? 'answered',
        tools: entry.tools ?? '',
        filters: entry.filters ?? '',
        figures: entry.figures ?? '',
        source: entry.source ?? 'oracle',
        cat: entry.cat,
        section: entry.section,
      };
      lines.push(COLUMNS.map((c) => clean(String(row[c]))).join('\t'));
    }
    if (firstId.has(entry.g)) throw new Error(`the group key ${entry.g} is used twice`);
    firstId.set(entry.g, ids[0] ?? '');
  }
  return lines.join('\n') + '\n';
}

export const BANK_FILE = join(HERE, 'questions.tsv');

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const text = buildText();
  if (process.argv.includes('--check')) {
    const now = readFileSync(BANK_FILE, 'utf8');
    if (now !== text) {
      console.error('questions.tsv is not what bank-spec.mjs builds: run build-bank.mjs');
      process.exit(2);
    }
    console.log('questions.tsv is what the spec builds');
  } else {
    writeFileSync(BANK_FILE, text, 'utf8');
    console.log(`wrote ${BANK_FILE}: ${text.split('\n').length - 2} questions`);
  }
}
