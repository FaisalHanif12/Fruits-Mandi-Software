// The screen and the backend agree on the month close (round 7, PRD FR-17.6, FR-21.2): each wrapper
// in features/month/api.ts sends the command name the backend registers and the input in the shape
// `{ input }` (the list sends only its name). The names are also read from the backend's own command
// file, so a command renamed there fails here first. The fixtures in tests/fixtures are the
// backend's own, copied (the same bytes), carry exactly the fields of the generated types, and every
// refusal the close can make has words in Urdu and in English, so no raw code is ever shown.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as monthApi from '../../src/features/month/api';
import { monthErrorId } from '../../src/features/month/text';
import { bundles, LANGUAGES } from '../../src/i18n';
import { errorMessageId } from '../../src/shared/api';
import * as invoke from '../../src/shared/api/invoke';
import {
  MONTH_LIST_CLOSED,
  MONTH_LIST_READY,
  MONTH_PREVIEW,
  MONTH_PREVIEW_BAD_MONTH,
  MONTH_PREVIEW_BAD_MONTH_REQUEST,
  MONTH_PREVIEW_NOT_FINISHED,
  MONTH_PREVIEW_NOT_FINISHED_REQUEST,
  MONTH_PREVIEW_REQUEST,
  MONTH_REVERSE,
  MONTH_REVERSE_NOT_IN_FORCE,
  MONTH_REVERSE_NOT_IN_FORCE_REQUEST,
  MONTH_REVERSE_NO_REASON,
  MONTH_REVERSE_NO_REASON_REQUEST,
  MONTH_REVERSE_REQUEST,
  MONTH_SAVE,
  MONTH_SAVE_DAYS_OPEN,
  MONTH_SAVE_DAYS_OPEN_REQUEST,
  MONTH_SAVE_NOTHING_TO_CLOSE,
  MONTH_SAVE_NOTHING_TO_CLOSE_REQUEST,
  MONTH_SAVE_REQUEST,
  MONTH_SAVE_TWICE,
  MONTH_SAVE_TWICE_REQUEST,
} from '../helpers/month-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const BACKEND_FILE = path.resolve(__dirname, '../../../backend/app/src/commands/month.rs');
const BACKEND_FIXTURES = path.resolve(__dirname, '../../../backend/crates/services/tests/fixtures');
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

const isMonthFixture = (name: string) => name.endsWith('.json') && name.startsWith('month_');

describe('the commands the screen calls are the ones the backend declares', () => {
  it('the four month close commands exist in the backend file', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    const declared = [...text.matchAll(/pub async fn (\w+)</g)].map((match) => match[1] ?? '');
    for (const name of [
      'month_list',
      'month_close_preview',
      'month_close_save',
      'month_close_reverse',
    ]) {
      expect(declared).toContain(name);
    }
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  it('the list sends only its name', async () => {
    call.mockResolvedValue(MONTH_LIST_READY);
    await expect(monthApi.monthList()).resolves.toBe(MONTH_LIST_READY);
    expect(call.mock.calls).toEqual([['month_list']]);
  });

  it('the preview, the close and the reversal send the exact request the backend wrote', async () => {
    await monthApi.monthClosePreview(MONTH_PREVIEW_REQUEST.month);
    await monthApi.monthClosePreview(MONTH_PREVIEW_NOT_FINISHED_REQUEST.month);
    await monthApi.monthClosePreview(MONTH_PREVIEW_BAD_MONTH_REQUEST.month);
    await monthApi.monthCloseSave(MONTH_SAVE_REQUEST.month);
    await monthApi.monthCloseSave(MONTH_SAVE_DAYS_OPEN_REQUEST.month);
    await monthApi.monthCloseSave(MONTH_SAVE_NOTHING_TO_CLOSE_REQUEST.month);
    await monthApi.monthCloseSave(MONTH_SAVE_TWICE_REQUEST.month);
    await monthApi.monthCloseReverse(MONTH_REVERSE_REQUEST.month, MONTH_REVERSE_REQUEST.reason);
    await monthApi.monthCloseReverse(
      MONTH_REVERSE_NO_REASON_REQUEST.month,
      MONTH_REVERSE_NO_REASON_REQUEST.reason,
    );
    await monthApi.monthCloseReverse(
      MONTH_REVERSE_NOT_IN_FORCE_REQUEST.month,
      MONTH_REVERSE_NOT_IN_FORCE_REQUEST.reason,
    );
    expect(call.mock.calls).toEqual([
      ['month_close_preview', { input: MONTH_PREVIEW_REQUEST }],
      ['month_close_preview', { input: MONTH_PREVIEW_NOT_FINISHED_REQUEST }],
      ['month_close_preview', { input: MONTH_PREVIEW_BAD_MONTH_REQUEST }],
      ['month_close_save', { input: MONTH_SAVE_REQUEST }],
      ['month_close_save', { input: MONTH_SAVE_DAYS_OPEN_REQUEST }],
      ['month_close_save', { input: MONTH_SAVE_NOTHING_TO_CLOSE_REQUEST }],
      ['month_close_save', { input: MONTH_SAVE_TWICE_REQUEST }],
      ['month_close_reverse', { input: MONTH_REVERSE_REQUEST }],
      ['month_close_reverse', { input: MONTH_REVERSE_NO_REASON_REQUEST }],
      ['month_close_reverse', { input: MONTH_REVERSE_NOT_IN_FORCE_REQUEST }],
    ]);
  });
});

describe('the fixtures are the backend’s own', () => {
  it('every month fixture is the same text as the backend’s file, and none is missing', () => {
    const backend = readdirSync(BACKEND_FIXTURES).filter(isMonthFixture).sort();
    const frontend = readdirSync(FRONTEND_FIXTURES).filter(isMonthFixture).sort();
    expect(frontend).toEqual(backend);
    expect(frontend.length).toBeGreaterThanOrEqual(24);
    for (const name of frontend) {
      expect(readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8'), name).toBe(
        readFileSync(path.join(BACKEND_FIXTURES, name), 'utf8'),
      );
    }
  });
});

const keysOf = (value: object) => Object.keys(value).sort();

describe('the answers carry exactly the fields of the generated types', () => {
  it('MonthList and MonthRow', () => {
    for (const list of [MONTH_LIST_READY, MONTH_LIST_CLOSED]) {
      expect(keysOf(list)).toEqual(['rows']);
      for (const row of list.rows) {
        expect(keysOf(row)).toEqual([
          'closed_entry_id',
          'closed_on',
          'month',
          'reversed_count',
          'state',
        ]);
      }
    }
    // Newest month first, as the screen shows them.
    const months = MONTH_LIST_READY.rows.map((row) => row.month);
    expect(months).toEqual([...months].sort().reverse());
    const states = new Set(
      [...MONTH_LIST_READY.rows, ...MONTH_LIST_CLOSED.rows].map((row) => row.state),
    );
    expect([...states].sort()).toEqual(['closed', 'not_finished', 'ready']);
  });

  it('MonthPreview and MonthLine', () => {
    expect(keysOf(MONTH_PREVIEW)).toEqual([
      'entry_date',
      'expense_paisa',
      'first_day',
      'income_paisa',
      'last_day',
      'lines',
      'month',
      'profit_paisa',
      'total_paisa',
    ]);
    for (const line of MONTH_PREVIEW.lines) {
      expect(keysOf(line)).toEqual(['account', 'cr_paisa', 'dr_paisa', 'kind', 'partner_id']);
      expect(keysOf(line.account)).toEqual(['code', 'id', 'name_en', 'name_ur']);
    }
    expect(new Set(MONTH_PREVIEW.lines.map((line) => line.kind))).toEqual(
      new Set(['income', 'expense', 'partner_capital']),
    );
  });

  it('MonthClosed and MonthReversed', () => {
    expect(keysOf(MONTH_SAVE)).toEqual(['entry_id', 'month', 'total_paisa']);
    expect(keysOf(MONTH_REVERSE)).toEqual(['month', 'reversal_entry_id']);
  });
});

describe('every refusal of the close has words in both languages, and none shows a raw code', () => {
  const refusals = [
    ['bad month', MONTH_PREVIEW_BAD_MONTH, 'ERR_FORMAT_INVALID', 'month'],
    ['not finished', MONTH_PREVIEW_NOT_FINISHED, 'ERR_DATE_IN_FUTURE', 'month'],
    ['days open', MONTH_SAVE_DAYS_OPEN, 'ERR_DAY_NOT_CLOSED', 'month'],
    ['nothing to close', MONTH_SAVE_NOTHING_TO_CLOSE, 'ERR_NOTHING_TO_CLOSE', 'month'],
    ['closed twice', MONTH_SAVE_TWICE, 'ERR_MONTH_CLOSED', 'month'],
    ['no reason', MONTH_REVERSE_NO_REASON, 'ERR_REASON_REQUIRED', 'reason'],
    ['not in force', MONTH_REVERSE_NOT_IN_FORCE, 'ERR_NOT_FOUND', 'month'],
  ] as const;

  it.each(refusals)(
    '%s: the code and the field are the backend’s',
    (_name, refusal, code, field) => {
      expect(refusal.ok).toBe(false);
      expect(refusal.errors).toHaveLength(1);
      expect(refusal.errors[0]).toMatchObject({ code, field });
    },
  );

  it.each(refusals)('%s: the screen has words for it', (_name, refusal) => {
    const [item] = refusal.errors;
    if (item === undefined) throw new Error('no error item');
    // The month screen's own words when it has them, else the general words of errors.ftl.
    const id = monthErrorId(item.code) ?? errorMessageId(item.code);
    for (const language of LANGUAGES) {
      expect(bundles[language].hasMessage(id), `${language} ${id}`).toBe(true);
    }
  });

  it('five refusals have the month screen’s own words, because the general words belong to the day screen', () => {
    expect(monthErrorId('ERR_DATE_IN_FUTURE')).toBe('month-error-not-finished');
    expect(monthErrorId('ERR_DAY_NOT_CLOSED')).toBe('month-error-days-open');
    expect(monthErrorId('ERR_MONTH_CLOSED')).toBe('month-error-already-closed');
    expect(monthErrorId('ERR_NOTHING_TO_CLOSE')).toBe('month-error-nothing-to-close');
    expect(monthErrorId('ERR_NOT_FOUND')).toBe('month-error-not-in-force');
    expect(monthErrorId('ERR_FORMAT_INVALID')).toBeNull();
    expect(monthErrorId('ERR_REASON_REQUIRED')).toBeNull();
    expect(monthErrorId('ERR_JOURNAL_UNBALANCED')).toBeNull();
  });

  it('the general words of the codes the month screen does not rename exist too', () => {
    for (const code of [
      'ERR_FORMAT_INVALID',
      'ERR_REASON_REQUIRED',
      'ERR_JOURNAL_UNBALANCED',
    ] as const) {
      for (const language of LANGUAGES) {
        expect(bundles[language].hasMessage(errorMessageId(code)), `${language} ${code}`).toBe(
          true,
        );
      }
    }
  });
});
