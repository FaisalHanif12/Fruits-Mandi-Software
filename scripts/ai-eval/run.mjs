#!/usr/bin/env node
// The live check of "ask the app" (D6 Section 3 / plan 50 R2, NFR-AI-2, D6-A16, D6-A17, D6-A19).
//
// It asks the 100 questions of scripts/ai-eval/questions.tsv through the app's own question path
// (run::ask, the function behind the assistant_ask command) on seeded databases, with the real
// model, then prints the score, every miss, the tool calls, the model calls, the tokens, the bad
// parallel inputs and the time, and writes scripts/ai-eval/urdu-review.tsv for the Urdu reviewer.
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
//   node scripts/ai-eval/run.mjs                 the full run (about 100 questions)
//   node scripts/ai-eval/run.mjs --only Q001,Q016   only these ids
//   node scripts/ai-eval/run.mjs --check         check the list (the D6 mix), no model, no cargo
//   node scripts/ai-eval/run.mjs --self-test     check the scorer on made-up replies, no model
//   node scripts/ai-eval/run.mjs --results FILE  score a results file again, no model
//   --model NAME   another model name (default: the app's default, openai/gpt-6-luna)
//   --review-out FILE   write the Urdu review sheet here instead of scripts/ai-eval/urdu-review.tsv
//
// Exit codes: 0 the run meets D6-A16; 1 it does not; 2 the list or the run is broken; 3 waiting
// for the key.
//
// The project folder has spaces in its name. Every path here is built with node:path and every
// process is started without a shell, so nothing is split.

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

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
const REVIEW = option('--review-out')
  ? resolve(option('--review-out'))
  : join(HERE, 'urdu-review.tsv');
const OUT_DIR = join(BACKEND, 'target', 'ai-eval');
const RESULTS = join(OUT_DIR, 'results.json');

const PASS_AT = 97; // D6-A16: at least 97 of 100.
const PREFIX_TOKENS = 9500; // D6-A19: the fixed part, at most 9,500 tokens (D6-system-prompt v1.2).
const ONE_ROUND_MS = 8000; // NFR-AI-3.
const MORE_ROUNDS_MS = 20000;
const SAFE_ENDS = new Set([
  'which_one',
  'need_dates',
  'need_party',
  'not_found',
  'not_understood',
  'checks_failed',
  'records_differ',
]);

// ---- the list ------------------------------------------------------------------------------

/** Reads questions.tsv into objects. */
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

const KINDS = {
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
  help: 10, // plan 51 B3, PRD FR-7.89: how to use the app, from the guide of `app_help`
};

/** Checks the list against the mix of D6 Section 3 and NFR-AI-2. Returns problems and a summary. */
function checkList(list) {
  const problems = [];
  const count = (key, value) => list.filter((q) => q[key] === value).length;
  const languages = {
    roman: count('lang', 'roman'),
    urdu: count('lang', 'urdu'),
    en: count('lang', 'en'),
  };
  const misspelled = list.filter((q) => q.misspelled).length;
  const groups = new Map();
  for (const q of list) if (q.group) groups.set(q.group, (groups.get(q.group) ?? 0) + 1);
  const fours = [...groups.values()].filter((n) => n === 4).length;
  if (list.length < 110) problems.push(`only ${list.length} questions (at least 110)`);
  for (const [kind, want] of Object.entries(KINDS)) {
    if (count('kind', kind) !== want)
      problems.push(`kind ${kind}: ${count('kind', kind)}, want ${want}`);
  }
  // About 40, 35 and 25 of every 100: the 10 help questions of plan 51 B3 keep that mix (4, 3, 3).
  if (languages.roman < 42 || languages.roman > 46)
    problems.push(`Roman Urdu ${languages.roman} (about 44)`);
  if (languages.urdu < 36 || languages.urdu > 40)
    problems.push(`Urdu letters ${languages.urdu} (about 38)`);
  if (languages.en < 26 || languages.en > 30) problems.push(`English ${languages.en} (about 28)`);
  if (misspelled < 15) problems.push(`only ${misspelled} misspelled (at least 15)`);
  if (fours < 8) problems.push(`only ${fours} groups of 4 (at least 8)`);
  const ids = new Set(list.map((q) => q.id));
  for (const q of list) {
    if (q.after && !ids.has(q.after))
      problems.push(`${q.id} follows ${q.after}, which is not in the list`);
    if (!q.question || !q.outcome) problems.push(`${q.id} has no question or outcome`);
  }
  return { problems, summary: { total: list.length, languages, misspelled, groupsOfFour: fours } };
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
  const out = { parts: [], all: [], months: new Set(), names: new Set(), order: [] };
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

/** Splits `a=1;b=2 Dr` into entries with the numbers, side and month the question expects. */
function expectedFigures(text) {
  return text
    .split(';')
    .filter(Boolean)
    .map((piece) => {
      const at = piece.indexOf('=');
      const label = piece.slice(0, at).trim();
      const value = piece.slice(at + 1).trim();
      let m;
      if ((m = value.match(/^(-?\d+(?:\.\d+)?)%$/)))
        return { label, kind: 'percent', n: Math.round(Number(m[1]) * 100) };
      if (/^\d{4}-\d{2}$/.test(label === 'date' ? '' : value))
        return { label, kind: 'month', month: value };
      if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return { label, kind: 'date', date: value };
      if ((m = value.match(/^(-?\d+)(?:\s+(Dr|Cr|up|down))?/))) {
        return { label, kind: 'amount', n: Number(m[1]), side: m[2]?.toLowerCase(), text: value };
      }
      // "Arshad Multani 18684 Dr 36 days": a name with its amount (advice).
      if ((m = value.match(/^([A-Za-z ]+?)\s+(-?\d+)(?:\s+(Dr|Cr))?/))) {
        return {
          label,
          kind: 'amount',
          n: Number(m[2]),
          side: m[3]?.toLowerCase(),
          text: value,
          name: m[1].toLowerCase(),
        };
      }
      return { label, kind: 'name', name: value.toLowerCase() };
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

function toolSeen(log, token) {
  const [tool, detail] = token.split(':');
  return (log ?? []).some(
    (c) => c.tool === tool && (!detail || c.input?.id === detail || c.input?.view === detail),
  );
}

/** The verdict of one question: pass or a miss with its reasons, and notes that are not misses. */
function score(question, result) {
  const reply = result.reply;
  const outcome = reply?.outcome ?? 'none';
  const found = collect(reply);
  const reasons = [];
  const notes = [];
  const expectedOutcome = question.outcome;
  if (expectedOutcome === 'safe') {
    const ok =
      outcome === 'answered' ||
      outcome === 'outside_books' ||
      outcome === 'change_refused' ||
      SAFE_ENDS.has(outcome);
    if (!ok) reasons.push(`outcome ${outcome}`);
    if (found.all.some((v) => v.n === 99999)) reasons.push('shows 99,999');
  } else if (outcome !== expectedOutcome) {
    reasons.push(`outcome ${outcome}, wanted ${expectedOutcome}`);
  }
  for (const want of expectedFigures(question.figures)) {
    if (want.kind === 'name') {
      if (
        !found.names.has(want.name) &&
        !JSON.stringify(reply?.answer?.sources ?? [])
          .toLowerCase()
          .includes(want.name)
      ) {
        notes.push(`name ${want.name} not seen`);
      }
    } else if (want.kind === 'month') {
      if (![...found.months].some((m) => m.endsWith(want.month)))
        reasons.push(`month ${want.month} (${want.label}) missing`);
    } else if (want.kind === 'date') {
      // A date is shown as a date part; the reply text is not checked further.
    } else if (!hasNumber(found, want)) {
      reasons.push(`figure ${want.label}=${want.text ?? want.n} missing`);
    }
  }
  // "Whom first": the first expected name comes before the second.
  const first = question.figures.match(/first=([A-Za-z ]+?)\s+\d/);
  const second = question.figures.match(/second=([A-Za-z ]+?)\s+\d/);
  if (first && second) {
    const a = found.order.indexOf(first[1].toLowerCase());
    const b = found.order.indexOf(second[1].toLowerCase());
    if (a < 0 || (b >= 0 && a > b))
      reasons.push(`order: ${first[1]} should come before ${second[1]}`);
  }
  // A help question must ask the guide for its topic (plan 51 B3).
  const topicWant = question.filters.match(/topic=([a-z_]+)/);
  if (
    topicWant &&
    !(result.tool_calls ?? []).some((c) => c.tool === 'app_help' && c.input?.topic === topicWant[1])
  )
    reasons.push(`app_help topic ${topicWant[1]} not asked`);
  if (question.outcome === 'change_refused' && question.id === 'Q096') {
    const route = reply?.entry_route;
    if (route?.screen !== 'wasooli') reasons.push('entry route is not wasooli');
  }
  if (question.outcome === 'outside_books' && question.filters.includes('nearest=party_card')) {
    if (!JSON.stringify(reply?.answer?.sources ?? []).includes('party_card'))
      reasons.push('no party card button');
  }
  // Tools are a note: another route to the same figure is not a miss.
  const groups = question.tools.split(',').filter(Boolean);
  for (const group of groups) {
    if (!group.split('/').some((t) => toolSeen(result.tool_calls, t)))
      notes.push(`tool ${group} not used`);
  }
  const shows = found.parts.length + found.all.length > 0;
  const kind =
    reasons.length === 0
      ? 'PASS'
      : SAFE_ENDS.has(outcome)
        ? 'MISS (safe end)'
        : shows || outcome === 'answered'
          ? 'MISS (REVIEW: figure, party, period or source)'
          : 'MISS';
  return { id: question.id, pass: reasons.length === 0, kind, reasons, notes, outcome };
}

/** Questions of one group must reach the same figures (NFR-AI-2). */
function groupAgreement(list, results) {
  const byGroup = new Map();
  for (const q of list) if (q.group) byGroup.set(q.group, [...(byGroup.get(q.group) ?? []), q]);
  const bad = [];
  for (const [group, members] of byGroup) {
    // A group is compared only when the run has all its questions.
    if (members.some((q) => !results.has(q.id))) continue;
    const signs = members.map((q) => {
      const r = results.get(q.id);
      if (q.kind === 'help') {
        // A help group agrees on the topic it asked for.
        const topics = (r?.tool_calls ?? [])
          .filter((c) => c.tool === 'app_help')
          .map((c) => c.input?.topic);
        return [...new Set(topics)].sort().join(',');
      }
      const numbers = collect(r?.reply).parts.map((v) => `${v.kind}:${v.n}:${v.side ?? ''}`);
      return [...new Set(numbers)].sort().join(',');
    });
    if (new Set(signs).size > 1) bad.push({ group, members: members.map((q) => q.id), signs });
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
  if (answer.advice) extra.push('[مشورہ، فیصلہ آپ کا]');
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
      q.kind === 'advice' ? '' : 'n/a',
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

function report(list, data) {
  const results = new Map(data.results.map((r) => [r.id, r]));
  const ran = list.filter((q) => results.has(q.id));
  const verdicts = ran.map((q) => score(q, results.get(q.id)));
  const passed = verdicts.filter((v) => v.pass).length;
  const lines = [];
  const say = (s = '') => lines.push(s);
  say(`AI EVAL  model ${data.meta.model}`);
  say(`endpoint ${data.meta.endpoint}`);
  say(
    `date ${new Date().toISOString().slice(0, 10)}   questions run ${ran.length} of ${list.length}`,
  );
  say();
  say(`SCORE ${passed} of ${ran.length}   (D6-A16: at least ${PASS_AT} of 100)`);
  const misses = verdicts.filter((v) => !v.pass);
  const review = misses.filter((v) => v.kind.includes('REVIEW'));
  say(
    `misses ${misses.length}: safe ends ${misses.filter((v) => v.kind.includes('safe')).length}, to review ${review.length}, other ${misses.length - review.length - misses.filter((v) => v.kind.includes('safe')).length}`,
  );
  for (const v of misses) {
    const q = list.find((x) => x.id === v.id);
    say(`  ${v.id} [${v.kind}] ${q.question}`);
    say(`      expected ${q.outcome}: ${q.figures || q.filters}`);
    say(`      got ${v.outcome}; ${v.reasons.join('; ')}`);
  }
  const noted = verdicts.filter((v) => v.notes.length > 0);
  say();
  say(`notes (not misses): ${noted.length} questions used another tool route or showed no name`);
  for (const v of noted.slice(0, 40)) say(`  ${v.id}: ${v.notes.join('; ')}`);
  const bad = groupAgreement(list, results);
  say();
  say(
    `GROUPS of the same question: ${bad.length === 0 ? 'every group reaches the same figures' : `${bad.length} DISAGREE`}`,
  );
  for (const g of bad) say(`  ${g.group} ${g.members.join(' ')}: ${g.signs.join('  |  ')}`);
  // Tools, model calls, tokens.
  const all = ran.map((q) => results.get(q.id));
  const toolCalls = all.map((r) => (r.tool_calls ?? []).filter((c) => !c.error).length);
  const modelCalls = all.map((r) => r.model_calls ?? 0);
  const t = (k) => sum(all.map((r) => r.tokens?.[k] ?? 0));
  const calls2 = all.map((r) => r.calls?.[1]?.cached).filter((x) => typeof x === 'number');
  const parallel = all.filter((r) => (r.calls ?? []).some((c) => c.tool_calls >= 2));
  const badParallel = sum(parallel.map((r) => (r.tool_calls ?? []).filter((c) => c.error).length));
  const limited = all.filter(
    (r) =>
      r.reply?.outcome === 'not_understood' &&
      ((r.model_calls ?? 0) >= 4 || (r.tokens?.in ?? 0) >= 60000 || (r.tokens?.out ?? 0) >= 3000),
  );
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
  const cachedMax = calls2.length ? Math.max(...calls2) : 0;
  say(
    `  cached tokens of each second call: ${calls2.length} calls, min ${calls2.length ? Math.min(...calls2) : 0}, max ${cachedMax} ${cachedMax <= PREFIX_TOKENS ? 'OK' : 'OVER'} (D6-A19: at most ${PREFIX_TOKENS}; it is the fixed part as the model counts it)`,
  );
  say(
    `PARALLEL tool calls that arrived with a bad input: ${badParallel} (in ${parallel.length} questions that asked for tools together)`,
  );
  say(
    `LIMITS  ${limited.length} questions stopped at a limit of NFR-AI-5 ${limited.length > 2 ? '(MORE THAN 2: ask Umar before raising a limit)' : ''}`,
  );
  // Time.
  const oneRound = all.filter((r) => (r.model_calls ?? 0) <= 2 && r.reply?.outcome !== 'estimate');
  const more = all.filter((r) => !oneRound.includes(r));
  const p = (rs) =>
    percentile(
      rs.map((r) => r.millis),
      0.95,
    );
  say(
    `TIME  total ${ms(sum(all.map((r) => r.millis)))}   one round: ${oneRound.length} questions, p95 ${ms(p(oneRound))} (target ${ms(ONE_ROUND_MS)})   more rounds or estimates: ${more.length}, p95 ${ms(p(more))} (target ${ms(MORE_ROUNDS_MS)})`,
  );
  const callTimes = all.flatMap((r) => (r.calls ?? []).map((c) => c.millis));
  say(
    `  one model call: p50 ${ms(percentile(callTimes, 0.5))}, p95 ${ms(percentile(callTimes, 0.95))}, max ${ms(callTimes.length ? Math.max(...callTimes) : 0)}`,
  );
  say();
  say(
    `URDU REVIEW SHEET ${REVIEW} (${ran.length} answers; the reviewer marks good or not, and for advice answers inside FR-7.88 or not; at least 95 of 100 must be good)`,
  );
  const advice = list.filter((q) => q.kind === 'advice' && results.has(q.id));
  say(`  advice answers to read for FR-7.88: ${advice.map((q) => q.id).join(' ')}`);
  const meets = passed >= PASS_AT && review.length === 0 && bad.length === 0 && ran.length >= 100;
  say();
  say(
    meets
      ? 'RESULT: meets the score, group and wrong-figure rules of D6-A16 (the Urdu review is still to do)'
      : 'RESULT: does NOT yet meet D6-A16',
  );
  return { text: lines.join('\n'), meets, results };
}

// ---- running -------------------------------------------------------------------------------

/** Runs the ignored live test through cargo. The key is not passed: the app reads its own store. */
function runLive(only, model) {
  mkdirSync(OUT_DIR, { recursive: true });
  const env = { ...process.env, MANDI_EVAL_QUESTIONS: QUESTIONS, MANDI_EVAL_OUT: RESULTS };
  if (only) env.MANDI_EVAL_ONLY = only;
  if (model) env.MANDI_EVAL_MODEL = model;
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
  const dr = (n) => ({
    type: 'figure',
    cell: { type: 'balance', paisa: n * 100, side: 'dr' },
    dir: null,
    checked: true,
  });
  const name = (en, ur) => ({
    type: 'name',
    name: { name_en: en, name_ur: ur, tag: null, code: '0020' },
  });
  const answered = (parts, extra = {}) => ({
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
    },
    ...extra,
  });
  const q1 = list.find((q) => q.id === 'Q001');
  const good = score(q1, {
    reply: answered([
      name('Akram Toll Tax', 'اکرم ٹول ٹیکس'),
      { type: 'text', text: ' کے ذمے ' },
      dr(2880),
    ]),
    tool_calls: [{ tool: 'find' }, { tool: 'party_card' }],
  });
  if (!good.pass) fail(`a right answer missed: ${good.reasons}`);
  const wrong = score(q1, { reply: answered([dr(2800)]), tool_calls: [] });
  if (wrong.pass || !wrong.kind.includes('REVIEW'))
    fail('a wrong figure was not flagged for review');
  const ask = score(q1, { reply: { outcome: 'which_one', answer: null }, tool_calls: [] });
  if (ask.pass || !ask.kind.includes('safe')) fail('which_one was not a safe end');
  const q96 = list.find((q) => q.id === 'Q096');
  const refused = score(q96, {
    reply: { outcome: 'change_refused', answer: null, entry_route: { screen: 'wasooli' } },
    tool_calls: [],
  });
  if (!refused.pass) fail(`a refusal missed: ${refused.reasons}`);
  const a = (o) => ({ id: 'Q001', reply: o });
  const sheet = answerAsShown(
    answered([
      name('Akram Toll Tax', 'اکرم ٹول ٹیکس'),
      { type: 'text', text: ' کے ذمے ' },
      dr(2880),
    ]),
  );
  if (!sheet.includes('2,880 بنام ✓')) fail(`the review text is "${sheet}"`);
  void a;
  const e = expectedFigures('same_pace=288000;pace_month=2027-04;share=72.10%;sum=21564 Dr');
  if (e.length !== 4 || e[1].kind !== 'month' || e[2].n !== 7210 || e[3].side !== 'dr')
    fail('expected figures were read wrongly');
  console.log(
    'self-test ok: the list matches the D6 mix and the scorer reads right, wrong and safe answers',
  );
}

// ---- main ----------------------------------------------------------------------------------

function main() {
  if (flag('--self-test')) return selfTest();
  const list = loadList();
  const { problems, summary } = checkList(list);
  console.log(`ai-eval: list ${QUESTIONS}`);
  console.log(
    `  ${summary.total} questions; Roman Urdu ${summary.languages.roman}, Urdu letters ${summary.languages.urdu}, English ${summary.languages.en}; ${summary.misspelled} misspelled; ${summary.groupsOfFour} groups of 4`,
  );
  if (problems.length) {
    console.error(`the list breaks the mix of D6 Section 3:\n  ${problems.join('\n  ')}`);
    process.exit(2);
  }
  if (flag('--check')) {
    console.log('the list meets the mix of D6 Section 3 and NFR-AI-2');
    return;
  }
  const file = option('--results');
  if (!file) {
    try {
      runLive(option('--only'), option('--model'));
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
  const { text, meets, results } = report(list, data);
  console.log(text);
  writeReview(
    list.filter((q) => results.has(q.id)),
    results,
  );
  process.exit(meets ? 0 : 1);
}

main();
