// The money guard of the live check (plan 70 T3, D8 Feature 10: at most $0.50 for all of D8).
//
// `run.mjs --budget <usd>` asks the questions in small batches and, before each batch, checks that
// the money already spent plus the WORST that the batch could still cost stays inside the budget.
// After each batch it reads the real usage of every model call from the results (prompt, cached and
// completion tokens, the usage block of the reply), works out the cost and writes it to a ledger file
// in the run folder. The ledger lives across runs: several small runs together stay under the budget,
// because every run starts from what the ledger already holds.
//
// Money is counted in nano-dollars (n$, whole numbers), so no sum ever drifts: a token of input at
// $0.10 a million is 100 n$. The fee of OpenRouter (5.5%) is added to every cost, rounded up.
//
// What is real and what is an estimate (every entry says which):
//   - a call that has a usage block (prompt, cached, completion tokens): REAL, from those tokens;
//   - a request with no usage block (a call that timed out, or the copy that lost a hedge): an
//     ESTIMATE at the ceiling of one call (the fixed prompt plus the largest answer of one call),
//     because the provider may still bill it;
//   - a follow-up question re-asks the question it follows in the same conversation, and the calls of
//     that first question are not in the row of the follow-up: an ESTIMATE, charged at the follow-up's
//     own cost again;
//   - a batch that started and left no results (the process was stopped): ESTIMATE, charged at the
//     reserve of the whole batch.
// The true figure is always what OpenRouter shows for the key (`run.mjs --key-limit`).
//
// The key is not read here and nothing here touches the network.

import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

/** The budget of D8, in dollars (plan 70). */
export const DEFAULT_BUDGET_USD = 0.5;
/** OpenRouter's fee on top of the price of the model: 5.5% (NFR-AI-5), as thousandths. */
export const FEE_PERMILLE = 1055;
/** Dollars per million tokens (NFR-AI-5, price of 01-11-2026). Add a model here before it is used. */
export const MODEL_PRICES = {
  'openai/gpt-6-luna': { in: 0.1, cached: 0.01, out: 0.5 },
};
/** The limits of one question, when the app's own file cannot be read (backend/crates/assistant/src/limits.rs). */
export const FALLBACK_LIMITS = {
  MAX_INPUT_TOKENS: 90_000,
  MAX_OUTPUT_TOKENS: 4_000,
  MAX_CALL_OUTPUT_TOKENS: 1_500,
  MAX_MODEL_CALLS: 6,
  MAX_REWRITES: 1,
  MAX_PREFIX_TOKENS: 11_600,
};
/** Batches are never larger than this many questions, so the world is not rebuilt more often than needed. */
export const DEFAULT_MAX_BATCH = 40;

/** A dollar amount as nano-dollars. */
export function toNano(dollars) {
  return Math.round(dollars * 1e9);
}

/** Nano-dollars as "$0.0123". */
export function usd(nano, digits = 4) {
  return `$${(nano / 1e9).toFixed(digits)}`;
}

/** Price per token in nano-dollars, as whole numbers: $0.10 a million is 100 n$ a token. */
export function pricePerToken(price) {
  return {
    in: Math.round(price.in * 1000),
    cached: Math.round(price.cached * 1000),
    out: Math.round(price.out * 1000),
  };
}

/** The price of a model, or `undefined` when it is not in the table and no override is given. */
export function priceOf(model, override = null) {
  if (
    override &&
    override.in !== undefined &&
    override.cached !== undefined &&
    override.out !== undefined
  )
    return { in: override.in, cached: override.cached, out: override.out };
  return MODEL_PRICES[model];
}

/** The fee added to a cost, rounded up to a whole nano-dollar. */
export function withFee(nano) {
  return Math.ceil((nano * FEE_PERMILLE) / 1000);
}

/**
 * The cost of one call from its usage block, fee included. `prompt` counts the cached tokens too (the
 * usage block of the API: `prompt_tokens` and `prompt_tokens_details.cached_tokens`), so the fresh part is
 * the difference. `completion` already holds the reasoning tokens.
 *
 *   24,000 cached, 9,000 fresh (33,000 prompt), 1,200 out: 24,000 x 10 + 9,000 x 100 + 1,200 x 500 n$
 *   = 1,740,000 n$ = $0.00174 (NFR-AI-5), and $0.0018357 with the fee.
 */
export function callCost(price, usage) {
  const per = pricePerToken(price);
  const prompt = Math.max(0, Math.trunc(usage.prompt ?? 0));
  const cached = Math.min(prompt, Math.max(0, Math.trunc(usage.cached ?? 0)));
  const completion = Math.max(0, Math.trunc(usage.completion ?? 0));
  return withFee((prompt - cached) * per.in + cached * per.cached + completion * per.out);
}

/**
 * The cost of a request that has no usage block: the fixed prompt (and the question) as fresh input, and the
 * largest answer one call may give.
 */
export function unknownCallCost(price, limits, userTokens = 0) {
  return callCost(price, {
    prompt: limits.MAX_PREFIX_TOKENS + userTokens,
    cached: 0,
    completion: limits.MAX_CALL_OUTPUT_TOKENS,
  });
}

/**
 * The most that one question can cost, fee included. A question has at most MAX_INPUT_TOKENS of input and
 * MAX_OUTPUT_TOKENS of output over its accepted calls; every call may have been sent twice (the hedge copy)
 * and a call that failed or timed out has no usage block, so a request for each possible call is added at the
 * ceiling of one call. This is the reserve kept free for each question of a batch.
 */
export function worstCasePerQuestion(price, limits) {
  const accepted = callCost(price, {
    prompt: limits.MAX_INPUT_TOKENS,
    cached: 0,
    completion: limits.MAX_OUTPUT_TOKENS,
  });
  const calls = limits.MAX_MODEL_CALLS + limits.MAX_REWRITES;
  return 2 * accepted + calls * unknownCallCost(price, limits);
}

/** The rows of the calls of a result row: the client's notes (the usage block of each reply). */
function usageCalls(row) {
  if (!Array.isArray(row.calls)) return [];
  return row.calls
    .filter((c) => c && typeof c === 'object')
    .map((c) => ({
      prompt: Number(c.prompt ?? 0),
      cached: Number(c.cached ?? 0),
      completion: Number(c.completion ?? 0),
    }));
}

/** The entries of the question log of a result row (`calls_json` is a JSON text, or already a list). */
function logCalls(row) {
  let value = row.calls_json;
  if (typeof value === 'string') {
    try {
      value = JSON.parse(value);
    } catch {
      return [];
    }
  }
  return Array.isArray(value) ? value.filter((c) => c && typeof c === 'object') : [];
}

/**
 * What a result row cost, in nano-dollars, fee included.
 * Returns `{ nano, in, cached, out, calls, estimated, notes }`: `calls` the replies that had a usage block,
 * `estimated` the nano-dollars of it that are estimates (see the top of this file).
 */
export function rowCost(row, price, limits) {
  const notes = [];
  let nano = 0;
  let estimated = 0;
  let prompt = 0;
  let cached = 0;
  let out = 0;
  let calls = 0;
  // What the replies said they used. The client's notes are the usage blocks themselves; the question log
  // holds the same numbers for the calls it accepted.
  let counted = usageCalls(row);
  const log = logCalls(row);
  if (counted.length === 0) {
    counted = log
      .filter((c) => !c.failed && Number(c.tokens_in ?? 0) > 0)
      .map((c) => ({
        prompt: Number(c.tokens_in ?? 0),
        cached: Number(c.tokens_cached ?? 0),
        completion: Number(c.tokens_out ?? 0),
      }));
  }
  for (const u of counted) {
    nano += callCost(price, u);
    prompt += u.prompt;
    cached += u.cached;
    out += u.completion;
    calls += 1;
  }
  // Requests with no usage block that the provider may still have billed: a failed call (a timeout) and the
  // copy of a hedge that lost. A call that failed because there was no network or no model ("offline") sent
  // nothing.
  const user = Number(row.user_tokens ?? 0);
  const silent = log.filter((c) => c.failed && c.error !== 'offline');
  if (!row.offline && silent.length > 0) {
    const each = unknownCallCost(price, limits, user);
    nano += silent.length * each;
    estimated += silent.length * each;
    notes.push(`${silent.length} request(s) with no usage block, charged at the ceiling of a call`);
  }
  // A follow-up asks its base question again, live, and the notes of that first question are cleared: the base
  // is not in this row, so it is charged once more at the cost of the replies this row does have.
  if (row.followup && !row.offline && calls > 0) {
    const base = nano;
    nano += base;
    estimated += base;
    notes.push(
      'follow-up: the question it follows was asked again; charged once more at the same cost',
    );
  }
  return { nano, in: prompt, cached, out, calls, estimated, notes };
}

/** The ledger of a run folder: `{ version, entries: [...] }`. A missing or broken file is an empty ledger. */
export function readLedger(path) {
  if (!existsSync(path)) return { version: 1, entries: [] };
  try {
    const data = JSON.parse(readFileSync(path, 'utf8'));
    if (data && Array.isArray(data.entries)) return { version: 1, entries: data.entries };
  } catch {
    // Fall through: a broken ledger must not be trusted, and must not be silently emptied.
  }
  throw new Error(
    `the budget ledger ${path} cannot be read; fix or move it (it holds what was already spent)`,
  );
}

/** The total spent, always summed from the entries (a stored total is never trusted). */
export function ledgerTotal(ledger) {
  return ledger.entries.reduce((sum, e) => sum + (Number.isFinite(e.nano) ? e.nano : 0), 0);
}

/** Writes the ledger whole, through a temporary file, so a stop in the middle cannot leave half a file. */
export function writeLedger(path, ledger) {
  mkdirSync(dirname(path), { recursive: true });
  const temporary = `${path}.tmp`;
  writeFileSync(
    temporary,
    JSON.stringify({ ...ledger, total_nano: ledgerTotal(ledger) }, null, 2) + '\n',
    'utf8',
  );
  renameSync(temporary, path);
}

/**
 * How many questions (counting the questions that each follow-up asks again) a chain costs: the question and
 * every question it follows, one after the other.
 */
export function chainLength(id, afterOf) {
  let length = 1;
  let at = afterOf(id);
  const seen = new Set([id]);
  while (at && !seen.has(at)) {
    length += 1;
    seen.add(at);
    at = afterOf(at);
  }
  return length;
}

/**
 * Asks the questions in batches inside the budget.
 *
 * `runBatch(ids)` runs one batch and returns `{ status, started, data }`: the exit status of the process,
 * whether the test itself started (and so may have spent money), and the parsed results file (`null` when
 * there is none). `data` is `{ meta, oracle, results }`, or `{ waiting_for_key: true }`.
 *
 * Before each batch: spent + reserve of the batch <= budget, or the run stops with the rest not asked. After
 * each batch the real cost of its rows goes to the ledger, so the next check sees it.
 *
 * Returns `{ rows, meta, oracle, asked, notAsked, spent, total, waitingForKey, stopped, runId }`.
 */
export function runBudgeted({
  ids,
  afterOf = () => '',
  runBatch,
  ledgerPath,
  budget,
  reserve,
  price,
  limits,
  model,
  maxBatch = DEFAULT_MAX_BATCH,
  say = () => {},
  now = () => new Date().toISOString(),
}) {
  const ledger = readLedger(ledgerPath);
  const runId = now();
  const rows = new Map();
  let meta = null;
  let oracle = null;
  let spent = 0;
  const asked = [];
  let waitingForKey = false;
  let stopped = null;
  const each = (id) => chainLength(id, afterOf) * reserve;
  if (budget < each(ids[0] ?? '')) {
    // Not even one question fits: say so before anything is sent.
    return {
      rows: [],
      meta,
      oracle,
      asked,
      notAsked: [...ids],
      spent,
      total: ledgerTotal(ledger),
      waitingForKey,
      stopped: `the budget ${usd(budget)} is smaller than the reserve for one question ${usd(each(ids[0] ?? ''))}`,
      runId,
    };
  }
  let todo = [...ids];
  let batchNumber = 0;
  while (todo.length > 0) {
    const room = budget - ledgerTotal(ledger);
    const batch = [];
    let held = 0;
    for (const id of todo) {
      if (batch.length >= maxBatch) break;
      if (held + each(id) > room) break;
      held += each(id);
      batch.push(id);
    }
    if (batch.length === 0) {
      stopped = `the next question could cost up to ${usd(each(todo[0]))} and only ${usd(Math.max(0, room))} of the budget ${usd(budget)} is left`;
      break;
    }
    batchNumber += 1;
    say(
      `budget: batch ${batchNumber}: ${batch.length} question(s), reserve ${usd(held)}, spent so far ${usd(ledgerTotal(ledger))} of ${usd(budget)}`,
    );
    const result = runBatch(batch);
    todo = todo.filter((id) => !batch.includes(id));
    const data = result.data;
    if (data?.waiting_for_key) {
      waitingForKey = true;
      break;
    }
    if (!data || !Array.isArray(data.results)) {
      // No results. If the test never started, nothing was spent. If it started, it may have spent anything up
      // to its reserve: charge the reserve, so the budget stays true.
      if (result.started) {
        ledger.entries.push({
          run: runId,
          at: now(),
          model,
          batch,
          nano: held,
          estimated: held,
          note: `the batch started and left no results (exit ${result.status}); charged at its reserve`,
        });
        spent += held;
        writeLedger(ledgerPath, ledger);
      }
      stopped = `the batch ${batch.join(',')} left no results (exit ${result.status})${result.started ? '; its reserve was charged' : '; it never started, nothing was charged'}`;
      break;
    }
    meta ??= data.meta;
    oracle ??= data.oracle;
    let batchSpent = 0;
    for (const row of data.results) {
      const cost = rowCost(row, price, limits);
      ledger.entries.push({
        run: runId,
        at: now(),
        model,
        id: row.id,
        nano: cost.nano,
        estimated: cost.estimated,
        calls: cost.calls,
        in: cost.in,
        cached: cost.cached,
        out: cost.out,
        ...(cost.notes.length ? { note: cost.notes.join('; ') } : {}),
      });
      batchSpent += cost.nano;
      rows.set(row.id, row);
      asked.push(row.id);
    }
    // A question of the batch that has no row (the test dropped it) may still have been billed: reserve.
    const missing = batch.filter((id) => !rows.has(id));
    for (const id of missing) {
      const nano = each(id);
      ledger.entries.push({
        run: runId,
        at: now(),
        model,
        id,
        nano,
        estimated: nano,
        note: 'no row came back for this question; charged at its reserve',
      });
      batchSpent += nano;
    }
    spent += batchSpent;
    writeLedger(ledgerPath, ledger);
    say(
      `budget: batch ${batchNumber} cost ${usd(batchSpent)}; total ${usd(ledgerTotal(ledger))} of ${usd(budget)}`,
    );
    if (ledgerTotal(ledger) >= budget) {
      stopped = `the budget ${usd(budget)} is reached`;
      break;
    }
  }
  const notAsked = ids.filter((id) => !rows.has(id));
  return {
    rows: ids.filter((id) => rows.has(id)).map((id) => rows.get(id)),
    meta,
    oracle,
    asked: ids.filter((id) => rows.has(id)),
    notAsked,
    spent,
    total: ledgerTotal(ledger),
    waitingForKey,
    stopped: notAsked.length > 0 && !waitingForKey ? (stopped ?? 'not asked') : null,
    runId,
  };
}

/** The ledger as a few lines of text, for the end of a run. */
export function ledgerReport(ledger, budget) {
  const total = ledgerTotal(ledger);
  const estimated = ledger.entries.reduce(
    (sum, e) => sum + (Number.isFinite(e.estimated) ? e.estimated : 0),
    0,
  );
  const runs = new Set(ledger.entries.map((e) => e.run)).size;
  const tokens = ledger.entries.reduce(
    (t, e) => ({
      in: t.in + (e.in ?? 0),
      cached: t.cached + (e.cached ?? 0),
      out: t.out + (e.out ?? 0),
    }),
    { in: 0, cached: 0, out: 0 },
  );
  return [
    `BUDGET LEDGER: ${ledger.entries.length} entries in ${runs} run(s)`,
    `  tokens   prompt ${tokens.in} (cached ${tokens.cached}), completion ${tokens.out}`,
    `  spent    ${usd(total)} of ${usd(budget)}   (${usd(estimated)} of it is an estimate: failed requests, copies, follow-up base questions)`,
    `  left     ${usd(Math.max(0, budget - total))}`,
  ].join('\n');
}

// ---------------------------------------------------------------------------------------------
// The self-test: a fake usage stream, no model, no key, no cargo.
// `node scripts/ai-eval/budget.mjs --self-test`, and `run.mjs --self-test` runs it too.
// ---------------------------------------------------------------------------------------------

export function budgetSelfTest(tmp) {
  const failures = [];
  const check = (cond, name) => {
    if (!cond) failures.push(name);
  };
  const price = MODEL_PRICES['openai/gpt-6-luna'];
  const limits = FALLBACK_LIMITS;

  // 1. The arithmetic of NFR-AI-5: a normal question with the instructions cached (3 calls).
  const raw = 24_000 * 10 + 9_000 * 100 + 1_200 * 500;
  check(raw === 1_740_000, 'the NFR-AI-5 example is $0.00174 in nano-dollars');
  check(
    callCost(price, { prompt: 33_000, cached: 24_000, completion: 1_200 }) === withFee(1_740_000),
    'the cost of the NFR-AI-5 example is the raw cost with the fee',
  );
  check(withFee(1_740_000) === 1_835_700, 'the fee of 5.5% on $0.00174 is $0.0018357');
  check(withFee(1) === 2, 'the fee is rounded up');
  // The worst case of one question: 90,000 fresh input and 4,000 output is $0.011 before the fee.
  check(
    callCost(price, { prompt: 90_000, cached: 0, completion: 4_000 }) === withFee(11_000_000),
    'the ceiling of one question as the limits give it',
  );
  check(toNano(0.5) === 500_000_000 && usd(toNano(0.5)) === '$0.5000', 'dollars and nano-dollars');
  check(priceOf('openai/gpt-6-luna')?.in === 0.1, 'the price of the default model is known');
  check(priceOf('some/unknown-model') === undefined, 'a model with no price is refused');
  check(
    priceOf('some/unknown-model', { in: 1, cached: 1, out: 1 })?.out === 1,
    'a price given on the command line is used',
  );

  // 2. A row's cost from the usage blocks of its calls; a request with no usage block is an estimate.
  const row = (id, calls, extra = {}) => ({
    id,
    calls: calls.map(([prompt, cached, completion]) => ({
      prompt,
      cached,
      completion,
      reasoning: 0,
    })),
    calls_json: '[]',
    user_tokens: 10,
    ...extra,
  });
  const one = rowCost(row('Q1', [[10_000, 9_000, 100]]), price, limits);
  check(
    one.nano === callCost(price, { prompt: 10_000, cached: 9_000, completion: 100 }),
    'one call',
  );
  check(one.calls === 1 && one.estimated === 0, 'a call with a usage block is real');
  const two = rowCost(
    row('Q2', [
      [10_000, 9_000, 100],
      [11_000, 9_000, 300],
    ]),
    price,
    limits,
  );
  check(
    two.nano ===
      callCost(price, { prompt: 10_000, cached: 9_000, completion: 100 }) +
        callCost(price, { prompt: 11_000, cached: 9_000, completion: 300 }),
    'two calls are added',
  );
  const timedOut = rowCost(
    row('Q3', [[10_000, 9_000, 100]], {
      calls_json: JSON.stringify([
        { failed: true, error: 'timeout', tokens_in: 0 },
        { failed: true, error: 'hedge_lost', hedged: 'copy', hedge_extra: true, tokens_in: 0 },
        { failed: false, tokens_in: 10_000, tokens_cached: 9_000, tokens_out: 100 },
      ]),
    }),
    price,
    limits,
  );
  check(
    timedOut.nano === one.nano + 2 * unknownCallCost(price, limits, 10),
    'a timed-out request and a lost hedge copy are charged at the ceiling of a call',
  );
  check(
    timedOut.estimated === 2 * unknownCallCost(price, limits, 10),
    'and are marked as estimates',
  );
  const offline = rowCost(
    { id: 'Q4', offline: true, calls: [], calls_json: '[{"failed":true,"error":"offline"}]' },
    price,
    limits,
  );
  check(offline.nano === 0, 'an offline row costs nothing');
  const fromLog = rowCost(
    {
      id: 'Q5',
      calls: [],
      calls_json: [{ failed: false, tokens_in: 5_000, tokens_cached: 4_000, tokens_out: 50 }],
    },
    price,
    limits,
  );
  check(
    fromLog.nano === callCost(price, { prompt: 5_000, cached: 4_000, completion: 50 }),
    'without the client notes the question log gives the usage',
  );
  const follow = rowCost(row('Q6', [[10_000, 9_000, 100]], { followup: true }), price, limits);
  check(
    follow.nano === 2 * one.nano && follow.estimated === one.nano,
    'a follow-up counts its base question again, as an estimate',
  );

  // 3. The run: stops before passing the budget, with a fake usage stream. Each fake question costs the same.
  const reserve = 3_000_000; // $0.003 reserved for each question
  const real = (id) =>
    row(id, [
      [5_000, 4_000, 100],
      [5_500, 4_000, 200],
    ]);
  const realCost = rowCost(real('x'), price, limits).nano;
  check(realCost < reserve, 'the fake question really costs less than its reserve');
  const ledgerPath = join(tmp, 'budget-ledger.json');
  let calls = 0;
  const runner = (log) => (batch) => {
    calls += 1;
    log.push([...batch]);
    return {
      status: 0,
      started: true,
      data: { meta: { model: 'fake' }, oracle: {}, results: batch.map(real) },
    };
  };
  const budget = toNano(0.008); // room for 2 questions of reserve at a time
  const clock = () => {
    let n = 0;
    return () => `t${(n += 1)}`;
  };
  // 4. Run A: a few questions, well inside the budget. Run B in the same folder starts from A's total, asks a
  // lot more than is left, and stops before passing the budget: several small runs together stay under it.
  const batchesA = [];
  const runA = runBudgeted({
    ids: ['Q1', 'Q2', 'Q3', 'Q4'],
    runBatch: runner(batchesA),
    ledgerPath,
    budget,
    reserve,
    price,
    limits,
    model: 'fake',
    maxBatch: 2,
    now: clock(),
  });
  check(runA.asked.length === 4 && runA.stopped === null, 'run A asks all four questions');
  check(runA.total === 4 * realCost, 'run A is counted at the real cost of every question');
  const idsB = Array.from({ length: 30 }, (_, i) => `R${i + 1}`);
  const batchesB = [];
  const runB = runBudgeted({
    ids: idsB,
    runBatch: runner(batchesB),
    ledgerPath,
    budget,
    reserve,
    price,
    limits,
    model: 'fake',
    maxBatch: 3,
    now: clock(),
  });
  check(runB.total <= budget, 'two runs together never pass the budget');
  check(runB.total > runA.total, 'run B adds to the total of run A');
  check(runB.stopped !== null && runB.notAsked.length > 0, 'run B stops with questions not asked');
  check(
    runB.asked.length + runB.notAsked.length === idsB.length,
    'every question is asked or not asked',
  );
  check(
    runB.asked.every((id, i) => id === idsB[i]),
    'the questions are asked in order',
  );
  const entries = readLedger(ledgerPath).entries;
  check(
    entries.reduce((s, e) => s + e.nano, 0) === runB.total,
    'the total is the sum of the ledger entries',
  );
  check(
    runB.total === (4 + runB.asked.length) * realCost,
    'each question is counted at its real cost',
  );
  check(
    runB.total + reserve > budget,
    'it stopped because the next question might not fit, not earlier',
  );
  // Before every batch of run B, what was spent plus the reserve of the batch was inside the budget.
  let running = runA.total;
  for (const batch of batchesB) {
    check(running + batch.length * reserve <= budget, 'a batch only starts when its reserve fits');
    running += batch.length * realCost;
  }
  // A third run with the budget already used up sends nothing.
  const full = runBudgeted({
    ids: ['Q99'],
    runBatch: () => {
      throw new Error('a batch was started with no budget left');
    },
    ledgerPath,
    budget: runB.total,
    reserve,
    price,
    limits,
    model: 'fake',
  });
  check(full.asked.length === 0 && full.notAsked[0] === 'Q99', 'no budget left: nothing is asked');
  check(full.total === runB.total, 'and nothing is added');

  // 5. A budget smaller than the reserve of one question sends nothing.
  const tiny = runBudgeted({
    ids: ['Q1'],
    runBatch: () => {
      throw new Error('a batch was started below the reserve');
    },
    ledgerPath: join(tmp, 'tiny.json'),
    budget: reserve - 1,
    reserve,
    price,
    limits,
    model: 'fake',
  });
  check(
    tiny.asked.length === 0 && /smaller than the reserve/.test(tiny.stopped ?? ''),
    'a budget below one reserve',
  );

  // 6. A batch that started and left no results is charged at its reserve; one that never started costs nothing.
  const crash = runBudgeted({
    ids: ['Q1', 'Q2'],
    runBatch: () => ({ status: 101, started: true, data: null }),
    ledgerPath: join(tmp, 'crash.json'),
    budget: toNano(0.05),
    reserve,
    price,
    limits,
    model: 'fake',
  });
  check(
    crash.total === 2 * reserve,
    'a batch that started and left nothing is charged at its reserve',
  );
  const noStart = runBudgeted({
    ids: ['Q1', 'Q2'],
    runBatch: () => ({ status: 101, started: false, data: null }),
    ledgerPath: join(tmp, 'nostart.json'),
    budget: toNano(0.05),
    reserve,
    price,
    limits,
    model: 'fake',
  });
  check(
    noStart.total === 0 && noStart.notAsked.length === 2,
    'a batch that never started costs nothing',
  );

  // 7. A question with no row in a batch that gave rows is charged at its reserve.
  const dropped = runBudgeted({
    ids: ['Q1', 'Q2'],
    runBatch: (batch) => ({
      status: 0,
      started: true,
      data: { meta: {}, oracle: {}, results: [real(batch[0])] },
    }),
    ledgerPath: join(tmp, 'dropped.json'),
    budget: toNano(0.05),
    reserve,
    price,
    limits,
    model: 'fake',
  });
  check(dropped.total === realCost + reserve, 'a question with no row is charged at its reserve');

  // 8. A follow-up reserves for the question it follows too.
  const afterOf = (id) => (id === 'F2' ? 'F1' : id === 'F3' ? 'F2' : '');
  check(
    chainLength('F3', afterOf) === 3 && chainLength('F1', afterOf) === 1,
    'a chain of follow-ups',
  );
  const chain = runBudgeted({
    ids: ['F3'],
    afterOf,
    runBatch: () => {
      throw new Error('started with too little room for the chain');
    },
    ledgerPath: join(tmp, 'chain.json'),
    budget: 3 * reserve - 1,
    reserve,
    price,
    limits,
    model: 'fake',
  });
  check(chain.asked.length === 0, 'a follow-up chain needs the reserve of every question in it');

  // 9. The key missing: the run stops and says so; nothing is charged.
  const noKey = runBudgeted({
    ids: ['Q1'],
    runBatch: () => ({ status: 0, started: true, data: { waiting_for_key: true } }),
    ledgerPath: join(tmp, 'nokey.json'),
    budget: toNano(0.05),
    reserve,
    price,
    limits,
    model: 'fake',
  });
  check(noKey.waitingForKey && noKey.total === 0, 'no key: nothing charged');

  // 10. The worst case of a question is bigger than its input and output ceilings alone, and a broken ledger is refused.
  check(
    worstCasePerQuestion(price, limits) >
      2 * callCost(price, { prompt: 90_000, cached: 0, completion: 4_000 }),
    'the reserve covers the copies and the failed requests too',
  );
  const broken = join(tmp, 'broken.json');
  writeFileSync(broken, '{ not json', 'utf8');
  let refused = false;
  try {
    readLedger(broken);
  } catch {
    refused = true;
  }
  check(refused, 'a ledger that cannot be read is refused, not emptied');
  check(calls > 0, 'the fake runner was used');
  return failures;
}

if (
  process.argv[1] &&
  process.argv[1].endsWith('budget.mjs') &&
  process.argv.includes('--self-test')
) {
  const { mkdtempSync } = await import('node:fs');
  const { tmpdir } = await import('node:os');
  const failures = budgetSelfTest(mkdtempSync(join(tmpdir(), 'ai-eval-budget-')));
  if (failures.length) {
    console.error(`budget self-test FAILED:\n  ${failures.join('\n  ')}`);
    process.exit(1);
  }
  console.log(
    'budget self-test: ok (the cost of every call, the stop before the budget, the running total)',
  );
}
