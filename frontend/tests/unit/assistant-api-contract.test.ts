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
  ['advice', real.REPLY_ADVICE],
  ['answered with a route', real.REPLY_ANSWERED_ROUTE],
  ['advice with a route', real.REPLY_ADVICE_ROUTE],
  ['which one', real.REPLY_WHICH_ONE],
  ['not found', real.REPLY_NOT_FOUND],
  ['need dates', real.REPLY_NEED_DATES],
  ['need party', real.REPLY_NEED_PARTY],
  ['not understood', real.REPLY_NOT_UNDERSTOOD],
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
        'next',
        'parts',
        'read_at',
        'sources',
        'tables',
      ]);
    }
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
        'outcome',
        'parent_id',
        'question',
        'source',
        'tokens_cached',
        'tokens_in',
        'tokens_out',
        'tokens_reasoning',
        'tools',
      ]);
    }
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
