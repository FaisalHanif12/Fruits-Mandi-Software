// The panel and the backend agree on plan 48 (ask the app, D6 Section 2): each wrapper in
// features/assistant/api.ts sends the command name the backend registers and the input in the shape
// `{ input }`. The names are also read from the backend's own command file, so a command renamed there
// fails here first. The fixtures in tests/fixtures are the backend's own, copied; here they must still
// be the same bytes as the backend's, must carry exactly the fields of the generated types, and every
// message ID the backend sends (a title, a measure, a figure, a place, a working) must have words in
// Urdu and in English. The step event is read from the same shape the backend declares.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as api from '../../src/features/assistant/api';
import { failureMessageId } from '../../src/features/assistant/components/ReplyBody';
import { bundles, LANGUAGES } from '../../src/i18n';
import * as invoke from '../../src/shared/api/invoke';
import * as real from '../helpers/assistant-real';
import type { Answer } from '../../src/bindings/Answer';
import type { AskReply } from '../../src/bindings/AskReply';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.clearAllMocks();
  call.mockResolvedValue(null);
});

const BACKEND_FILE = path.resolve(__dirname, '../../../backend/app/src/commands/assistant.rs');
const BACKEND_FIXTURES = path.resolve(
  __dirname,
  '../../../backend/crates/assistant/tests/fixtures',
);
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

const isAssistantFixture = (name: string) =>
  name.startsWith('assistant_') && name.endsWith('.json');

describe('the commands the panel calls are the ones the backend declares', () => {
  it('every assistant command used in api.ts exists in the backend file', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    const declared = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    for (const name of [
      'assistant_status',
      'assistant_ask',
      'assistant_quick',
      'assistant_pick',
      'assistant_rerun',
      'assistant_new_conversation',
      'assistant_settings_set',
      'assistant_key_save',
      'assistant_key_remove',
      'assistant_test',
      'assistant_log_list',
    ]) {
      expect(declared).toContain(name);
    }
  });

  it('the step event is the one the backend names', () => {
    const text = readFileSync(
      path.resolve(__dirname, '../../../backend/crates/assistant/src/step.rs'),
      'utf8',
    );
    expect(text).toContain(`"${api.STEP_EVENT}"`);
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  it('the status and the connection test send only their name', async () => {
    await api.assistantStatus();
    await api.assistantTest();
    expect(call.mock.calls).toEqual([['assistant_status'], ['assistant_test']]);
  });

  it('the others send the exact request the backend wrote', async () => {
    await api.assistantAsk(real.REPLY_ANSWERED_REQUEST);
    await api.assistantAsk(real.REPLY_ANSWERED_ROUTE_REQUEST);
    await api.assistantAsk(real.REPLY_ADVICE_ROUTE_REQUEST);
    await api.assistantAsk(real.REPLY_ESTIMATE_WHAT_IF_REQUEST);
    await api.assistantAsk(real.REPLY_ESTIMATE_YEAR_END_REQUEST);
    await api.assistantAsk(real.REPLY_ANSWERED_PERCENT_MORE_REQUEST);
    await api.assistantAsk(real.REPLY_GENERAL_ADVICE_REQUEST);
    await api.assistantAsk(real.REPLY_HELP_ROUTE_REQUEST);
    await api.assistantAsk(real.REPLY_NOT_UNDERSTOOD_CALLS_REQUEST);
    await api.assistantAsk(real.REPLY_NOT_UNDERSTOOD_TOKENS_TABLE_REQUEST);
    await api.assistantQuick(real.QUICK_CASH_IN_HAND_REQUEST);
    await api.assistantPick(real.PICK_MATCH_REQUEST);
    await api.assistantPick(real.PICK_PARTNER_CODE_REQUEST);
    await api.assistantRerun(real.RERUN_REQUEST);
    await api.assistantNewConversation(real.NEW_CONVERSATION_REQUEST);
    await api.assistantSettingsSet(real.SETTINGS_SET_REQUEST);
    await api.assistantKeySave(real.KEY_SAVE_REQUEST);
    await api.assistantKeyRemove(real.KEY_REMOVE_REQUEST);
    await api.assistantLogList(real.LOG_LIST_FIRST_PAGE_REQUEST);
    expect(call.mock.calls).toEqual([
      ['assistant_ask', { input: real.REPLY_ANSWERED_REQUEST }],
      ['assistant_ask', { input: real.REPLY_ANSWERED_ROUTE_REQUEST }],
      ['assistant_ask', { input: real.REPLY_ADVICE_ROUTE_REQUEST }],
      ['assistant_ask', { input: real.REPLY_ESTIMATE_WHAT_IF_REQUEST }],
      ['assistant_ask', { input: real.REPLY_ESTIMATE_YEAR_END_REQUEST }],
      ['assistant_ask', { input: real.REPLY_ANSWERED_PERCENT_MORE_REQUEST }],
      ['assistant_ask', { input: real.REPLY_GENERAL_ADVICE_REQUEST }],
      ['assistant_ask', { input: real.REPLY_HELP_ROUTE_REQUEST }],
      ['assistant_ask', { input: real.REPLY_NOT_UNDERSTOOD_CALLS_REQUEST }],
      ['assistant_ask', { input: real.REPLY_NOT_UNDERSTOOD_TOKENS_TABLE_REQUEST }],
      ['assistant_quick', { input: real.QUICK_CASH_IN_HAND_REQUEST }],
      ['assistant_pick', { input: real.PICK_MATCH_REQUEST }],
      ['assistant_pick', { input: real.PICK_PARTNER_CODE_REQUEST }],
      ['assistant_rerun', { input: real.RERUN_REQUEST }],
      ['assistant_new_conversation', { input: real.NEW_CONVERSATION_REQUEST }],
      ['assistant_settings_set', { input: real.SETTINGS_SET_REQUEST }],
      ['assistant_key_save', { input: real.KEY_SAVE_REQUEST }],
      ['assistant_key_remove', { input: real.KEY_REMOVE_REQUEST }],
      ['assistant_log_list', { input: real.LOG_LIST_FIRST_PAGE_REQUEST }],
    ]);
  });
});

describe('the fixtures are the backend’s own', () => {
  it('every assistant fixture is the same text as the backend’s file, and none is missing', () => {
    const names = (dir: string) => readdirSync(dir).filter(isAssistantFixture).sort();
    expect(names(FRONTEND_FIXTURES)).toEqual(names(BACKEND_FIXTURES));
    expect(names(FRONTEND_FIXTURES).length).toBeGreaterThan(40);
    for (const name of names(FRONTEND_FIXTURES)) {
      expect(readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8'), name).toBe(
        readFileSync(path.join(BACKEND_FIXTURES, name), 'utf8'),
      );
    }
  });
});

const keysOf = (value: object) => Object.keys(value).sort();

const REPLIES: readonly (readonly [string, AskReply])[] = [
  ['answered', real.REPLY_ANSWERED],
  ['answered sum', real.REPLY_ANSWERED_SUM],
  ['answered chart', real.REPLY_ANSWERED_CHART],
  ['estimate', real.REPLY_ESTIMATE],
  ['estimate target', real.REPLY_ESTIMATE_TARGET],
  ['estimate what if', real.REPLY_ESTIMATE_WHAT_IF],
  ['estimate year end', real.REPLY_ESTIMATE_YEAR_END],
  ['advice', real.REPLY_ADVICE],
  ['answered with a route', real.REPLY_ANSWERED_ROUTE],
  ['advice with a route', real.REPLY_ADVICE_ROUTE],
  ['answered percent more', real.REPLY_ANSWERED_PERCENT_MORE],
  ['general advice', real.REPLY_GENERAL_ADVICE],
  ['help with a main screen', real.REPLY_HELP_ROUTE],
  ['which one', real.REPLY_WHICH_ONE],
  ['not found', real.REPLY_NOT_FOUND],
  ['need dates', real.REPLY_NEED_DATES],
  ['need party', real.REPLY_NEED_PARTY],
  ['not understood', real.REPLY_NOT_UNDERSTOOD],
  ['not understood after the calls', real.REPLY_NOT_UNDERSTOOD_CALLS],
  ['not understood after the tokens, with a table', real.REPLY_NOT_UNDERSTOOD_TOKENS_TABLE],
  ['outside books', real.REPLY_OUTSIDE_BOOKS],
  ['change refused', real.REPLY_CHANGE_REFUSED],
  ['checks failed', real.REPLY_CHECKS_FAILED],
  ['records differ', real.REPLY_RECORDS_DIFFER],
  ['locked', real.REPLY_LOCKED],
  ['limit reached', real.REPLY_LIMIT_REACHED],
  ['offline', real.REPLY_OFFLINE],
  ['error 404', real.REPLY_ERROR_404],
  ['quick', real.QUICK_CASH_IN_HAND],
  ['pick match', real.PICK_MATCH],
  ['pick partner code', real.PICK_PARTNER_CODE],
  ['rerun', real.RERUN],
];

describe('the answers carry exactly the fields of the generated types', () => {
  it.each(REPLIES)('AskReply: %s', (_name, reply) => {
    expect(keysOf(reply)).toEqual([
      'answer',
      'conversation_id',
      'count_today',
      'entry_route',
      'failure',
      'limit',
      'locked',
      'more_options',
      'options',
      'outcome',
      'question_id',
      'stop_reason',
    ]);
  });

  it('an Answer: parts, tables, charts, the estimate, advice, sources, differences, next, read_at', () => {
    const answers: Answer[] = REPLIES.flatMap(([, reply]) =>
      reply.answer === null ? [] : [reply.answer],
    );
    expect(answers.length).toBeGreaterThan(8);
    for (const answer of answers) {
      expect(keysOf(answer)).toEqual([
        'advice',
        'charts',
        'differences',
        'estimate',
        'general_advice',
        'next',
        'parts',
        'read_at',
        'sources',
        'tables',
      ]);
    }
  });

  it('an estimate method carries the factor: a percent for a what-if, null for the others', () => {
    const methods = [
      real.REPLY_ESTIMATE,
      real.REPLY_ESTIMATE_TARGET,
      real.REPLY_ESTIMATE_WHAT_IF,
    ].flatMap((reply) => reply.answer?.estimate?.methods ?? []);
    expect(methods.length).toBeGreaterThan(4);
    for (const method of methods) {
      expect(Object.keys(method)).toContain('factor');
      if (method.kind === 'what_if_pace') {
        expect(method.factor).toEqual({ type: 'percent', hundredths: 20000 });
      } else {
        expect(method.factor).toBeNull();
      }
    }
  });

  it('a whole-year estimate carries the books so far and each method its year total; no other estimate does', () => {
    const estimates = [
      real.REPLY_ESTIMATE,
      real.REPLY_ESTIMATE_TARGET,
      real.REPLY_ESTIMATE_WHAT_IF,
      real.REPLY_ESTIMATE_YEAR_END,
    ].map((reply) => reply.answer?.estimate ?? null);
    for (const estimate of estimates) {
      if (estimate === null) throw new Error('every estimate fixture has an estimate');
      expect(Object.keys(estimate)).toContain('year');
      const yearEnd = estimate.year !== null;
      for (const method of estimate.methods) {
        expect(Object.keys(method)).toContain('year_total');
        expect(method.year_total !== null).toBe(yearEnd);
      }
    }
    const year = real.REPLY_ESTIMATE_YEAR_END.answer?.estimate?.year;
    expect(year?.months).toBe(6);
    expect(year?.so_far).toEqual({ type: 'money', paisa: 26_100_000 });
    expect(year?.from).toEqual({ year: 2026, month: 4 });
    expect(year?.to).toEqual({ year: 2026, month: 9 });
  });

  it('the status, the test result, the log and the step', () => {
    expect(keysOf(real.STATUS_ON)).toEqual([
      'count_today',
      'default_model',
      'enabled',
      'key_last4',
      'key_saved',
      'limit',
      'model',
    ]);
    expect(real.STATUS_OFF.enabled).toBe(false);
    expect(keysOf(real.TEST_NO_ENDPOINT)).toEqual([
      'count_today',
      'failure',
      'limit',
      'millis',
      'model',
      'worked',
    ]);
    expect(keysOf(real.LOG_LIST_FIRST_PAGE)).toEqual(['limit', 'offset', 'rows', 'total_count']);
    expect(keysOf(real.STEP_READING_REPORT)).toEqual(['args', 'id']);
    for (const row of real.LOG_LIST_FIRST_PAGE.rows) {
      expect(keysOf(row)).toEqual([
        'asked_at',
        'asked_day',
        'can_rerun',
        'counted',
        'id',
        'model',
        'model_calls',
        'model_millis',
        'outcome',
        'parent_id',
        'question',
        'source',
        'tokens_cached',
        'tokens_in',
        'tokens_out',
        'tokens_reasoning',
        'tool_millis',
        'tools',
        'total_millis',
      ]);
    }
  });

  it('general advice is only text, is advice too, and no other reply is general advice', () => {
    for (const [name, reply] of REPLIES) {
      const answer = reply.answer;
      if (answer === null) continue;
      expect(answer.general_advice, name).toBe(reply === real.REPLY_GENERAL_ADVICE);
    }
    const answer = real.REPLY_GENERAL_ADVICE.answer;
    expect(real.REPLY_GENERAL_ADVICE.outcome).toBe('advice');
    expect(answer?.advice).toBe(true);
    expect(answer?.parts.length).toBeGreaterThan(0);
    expect(answer?.parts.every((part) => part.type === 'text')).toBe(true);
    expect(answer?.tables).toEqual([]);
    expect(answer?.charts).toEqual([]);
    expect(answer?.estimate).toBeNull();
    expect(answer?.sources).toEqual([]);
  });

  it('stop_reason is null in every reply but a stopped one, and a stopped one is not_understood', () => {
    for (const [name, reply] of REPLIES) {
      const stopped = name.startsWith('not understood after');
      expect(reply.stop_reason !== null, name).toBe(stopped);
      if (stopped) expect(reply.outcome).toBe('not_understood');
    }
    expect(real.REPLY_NOT_UNDERSTOOD_CALLS.stop_reason).toBe('calls');
    expect(real.REPLY_NOT_UNDERSTOOD_TOKENS_TABLE.stop_reason).toBe('tokens');
  });

  it('the stored key never appears: the status holds only its last four characters', () => {
    for (const status of [real.STATUS_ON, real.KEY_SAVE, real.KEY_REMOVE, real.SETTINGS_SET]) {
      expect(JSON.stringify(status)).not.toMatch(/sk-/);
      if (status.key_last4 !== null) expect(status.key_last4).toHaveLength(4);
    }
  });
});

describe('the step event is read as the backend writes it', () => {
  it('a step with its arguments', () => {
    expect(api.stepOf(real.STEP_READING_REPORT)).toEqual(real.STEP_READING_REPORT);
  });

  it('anything else is not a step', () => {
    expect(api.stepOf(null)).toBeNull();
    expect(api.stepOf('text')).toBeNull();
    expect(api.stepOf({ id: '' })).toBeNull();
    expect(api.stepOf({ id: 'assistant-step-checking' })).toEqual({
      id: 'assistant-step-checking',
      args: [],
    });
  });
});

/** Every message ID a fixture points at, so a missing word is found here and not on the screen. */
function idsIn(reply: AskReply): string[] {
  const ids: string[] = [];
  const answer = reply.answer;
  if (answer !== null) {
    for (const part of answer.parts) {
      if (part.type === 'doc') ids.push(part.type_id);
      if (part.type === 'figure' && part.working !== null) ids.push(part.working.title_id);
    }
    for (const table of answer.tables) {
      ids.push(table.title_id, table.totals_label_id, ...table.columns.map((c) => c.title_id));
    }
    for (const chart of answer.charts) ids.push(chart.measure_id);
    if (answer.estimate !== null) {
      ids.push(answer.estimate.measure_id);
      for (const method of answer.estimate.methods) ids.push(method.working.title_id);
    }
    for (const source of answer.sources) {
      if (source.kind === 'report' || source.kind === 'analytics') ids.push(source.label_id);
    }
    for (const difference of answer.differences) {
      ids.push(difference.figure_id, ...difference.places.map((place) => place.place_id));
    }
  }
  if (reply.failure !== null) ids.push(failureMessageId(reply.failure.kind));
  return ids;
}

/** Every working title the backend's source names, read from the source, so a new one fails here first. */
function workingTitlesInBackend(): string[] {
  const dir = path.resolve(__dirname, '../../../backend/crates/assistant/src');
  const found = new Set<string>();
  const walk = (folder: string) => {
    for (const entry of readdirSync(folder, { withFileTypes: true })) {
      const full = path.join(folder, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.name.endsWith('.rs')) {
        for (const match of readFileSync(full, 'utf8').matchAll(/"(assistant-working-[a-z-]+)"/g)) {
          found.add(match[1] ?? '');
        }
      }
    }
  };
  walk(dir);
  return [...found].sort();
}

describe('every working title the backend can send has words in both languages', () => {
  it('reads the titles from the backend source, including the percent more and less', () => {
    const titles = workingTitlesInBackend();
    expect(titles).toContain('assistant-working-percent-more');
    expect(titles).toContain('assistant-working-percent-less');
    expect(titles.length).toBeGreaterThan(9);
  });

  it.each(LANGUAGES)('%s', (language) => {
    const missing = workingTitlesInBackend().filter((id) => !bundles[language].hasMessage(id));
    expect(missing).toEqual([]);
  });
});

describe('every message ID the backend sends has words in both languages', () => {
  it.each(LANGUAGES)('%s', (language) => {
    const missing = new Set<string>();
    for (const [, reply] of REPLIES) {
      for (const id of idsIn(reply)) {
        if (!bundles[language].hasMessage(id)) missing.add(id);
      }
    }
    for (const id of [real.STEP_READING_REPORT.id]) {
      if (!bundles[language].hasMessage(id)) missing.add(id);
    }
    expect([...missing]).toEqual([]);
  });
});
