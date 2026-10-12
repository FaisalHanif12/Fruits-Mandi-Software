// The opening balances grid as plain logic (plan 07, PRD FR-10.9): which rows are lines, what goes
// to the backend, and where each problem belongs. No page, no backend.

import { describe, expect, it } from 'vitest';
import type { EntryRow } from '../../src/shared/grid';
import {
  baselineOf,
  changedLines,
  openingTyped,
  problemsOfError,
  problemsOfPreview,
  type OpeningBaseline,
} from '../../src/features/masters/openings';
import { fail } from '../helpers/app';
import { at } from '../helpers/sample';
import { account, balance, money } from '../helpers/accounts';
import { paisaForTest } from '../helpers/paisa';
import type { UiError } from '../../src/bindings/UiError';
import { uiErrorOf } from '../../src/shared/api';

const COMMON = '10-10-2026';
const AKRAM = account(21, 'buyer', '03040001', 'اکرم', 'Akram');
const RAHIM = account(31, 'seller', '030110001', 'رحیم', 'Rahim', {
  opening_date: '2026-08-31',
  opening_dr_paisa: money(15000),
  opening_is_advance: true,
});
const NEW_ONE = account(41, 'buyer', '03040009', 'نیا', 'New One', { opening_date: null });

function rowsOf(...accounts: ReturnType<typeof account>[]) {
  const baselines = new Map<string, OpeningBaseline>();
  const rows: EntryRow[] = accounts.map((one, index) => {
    const typed = openingTyped(one, 'international', COMMON);
    baselines.set(String(one.id), baselineOf(one, typed));
    return { rowId: `row-${String(index)}`, typed };
  });
  return { rows, baselines };
}

const edit = (row: EntryRow | undefined, patch: Record<string, string>): EntryRow => {
  if (row === undefined) throw new Error('no row');
  return { ...row, typed: { ...row.typed, ...patch } };
};

describe('openingTyped', () => {
  it('shows a saved opening as the text of the cells, and nothing for no amount', () => {
    expect(openingTyped(RAHIM, 'international', COMMON)).toEqual({
      account_id: '31',
      opening_date: '31-08-2026',
      opening_dr: '15,000',
      opening_cr: '',
    });
  });

  it('gives an account with no opening the common date', () => {
    expect(openingTyped(NEW_ONE, 'international', COMMON)).toMatchObject({
      opening_date: COMMON,
      opening_dr: '',
      opening_cr: '',
    });
  });
});

describe('changedLines', () => {
  it('nothing changed gives no line, even though new accounts carry the common date', () => {
    const { rows, baselines } = rowsOf(AKRAM, RAHIM, NEW_ONE);
    expect(changedLines(rows, baselines)).toEqual([]);
  });

  it('an amount typed on an account with no opening makes a line with the text as typed', () => {
    const { rows, baselines } = rowsOf(AKRAM, NEW_ONE);
    const next = [at(rows, 0), edit(rows[1], { opening_dr: '5000' })];
    expect(changedLines(next, baselines)).toEqual([
      {
        rowId: 'row-1',
        line: {
          account_id: 41,
          version: 1,
          opening_date: COMMON,
          opening_dr: '5000',
          opening_cr: '',
          opening_is_advance: false,
        },
      },
    ]);
  });

  it('a changed date alone is a line only for an account that had an opening', () => {
    const { rows, baselines } = rowsOf(RAHIM, NEW_ONE);
    const next = [
      edit(rows[0], { opening_date: '01-09-2026' }),
      edit(rows[1], { opening_date: '01-09-2026' }),
    ];
    const lines = changedLines(next, baselines);
    expect(lines.map((entry) => entry.line.account_id)).toEqual([31]);
    expect(lines[0]?.line.opening_dr).toBe('15,000');
  });

  it('keeps the advance mark of a seller while he has a debit, and drops it without one', () => {
    const { rows, baselines } = rowsOf(RAHIM);
    const bigger = changedLines([edit(rows[0], { opening_dr: '20000' })], baselines);
    expect(bigger[0]?.line.opening_is_advance).toBe(true);
    const cleared = changedLines([edit(rows[0], { opening_dr: '', opening_cr: '100' })], baselines);
    expect(cleared[0]?.line.opening_is_advance).toBe(false);
  });

  it('a buyer is never an advance', () => {
    const { rows, baselines } = rowsOf(AKRAM);
    const lines = changedLines([edit(rows[0], { opening_dr: '1' })], baselines);
    expect(lines[0]?.line.opening_is_advance).toBe(false);
  });

  it('the lines keep the order of the grid', () => {
    const { rows, baselines } = rowsOf(AKRAM, RAHIM, NEW_ONE);
    const next = [
      edit(rows[0], { opening_cr: '10' }),
      at(rows, 1),
      edit(rows[2], { opening_dr: '20' }),
    ];
    expect(changedLines(next, baselines).map((entry) => entry.line.account_id)).toEqual([21, 41]);
  });
});

describe('problems', () => {
  const { rows, baselines } = rowsOf(AKRAM, NEW_ONE);
  const changed = changedLines(
    [edit(rows[0], { opening_dr: '1', opening_cr: '2' }), edit(rows[1], { opening_dr: '5' })],
    baselines,
  );

  it('a problem of the preview goes to the row and cell it belongs to', () => {
    const problems = problemsOfPreview(changed, {
      rows: [
        {
          account: null,
          errors: [{ code: 'ERR_OPENING_BOTH_SIDES', field: 'opening_dr', values: {} }],
          opening_dr_paisa: paisaForTest(0),
          opening_cr_paisa: paisaForTest(0),
        },
        {
          account: null,
          errors: [
            { code: 'ERR_ADVANCE_NOT_SELLER', field: 'opening_is_advance', values: { x: 'y' } },
          ],
          opening_dr_paisa: paisaForTest(0),
          opening_cr_paisa: paisaForTest(0),
        },
      ],
      total_dr_paisa: paisaForTest(0),
      total_cr_paisa: paisaForTest(0),
      opening_difference_after: balance(0, 'zero'),
    });
    expect(problems.map((problem) => [problem.rowId, problem.field, problem.item.code])).toEqual([
      ['row-0', 'opening_dr', 'ERR_OPENING_BOTH_SIDES'],
      // The advance mark is typed with the debit, so its problem shows on the debit cell.
      ['row-1', 'opening_dr', 'ERR_ADVANCE_NOT_SELLER'],
    ]);
    // The values of the message go with the problem (Code Standards 4.3).
    expect(problems.map((problem) => problem.item.values)).toEqual([{}, { x: 'y' }]);
  });

  it('a refused save puts lines[i].field on the i-th changed row, and the rest in a notice', () => {
    const error: UiError = uiErrorOf(
      fail(
        { code: 'ERR_OPENING_BOTH_SIDES', field: 'lines[0].opening_dr' },
        { code: 'ERR_DATE_CLOSED', field: 'lines[1].opening_date' },
        { code: 'ERR_DB_BUSY' },
        { code: 'ERR_DATE_CLOSED', field: 'lines[7].opening_date' },
      ),
    );
    const split = problemsOfError(error, changed);
    expect(split.problems.map((problem) => [problem.rowId, problem.field])).toEqual([
      ['row-0', 'opening_dr'],
      ['row-1', 'opening_date'],
    ]);
    // A line that is not in the grid, and a whole-form problem, stay in the notice.
    expect(split.rest?.errors.map((item) => item.code)).toEqual(['ERR_DB_BUSY', 'ERR_DATE_CLOSED']);
  });

  it('a refusal with only line problems leaves no notice', () => {
    const error = uiErrorOf(fail({ code: 'ERR_OPENING_BOTH_SIDES', field: 'lines[0].opening_dr' }));
    expect(problemsOfError(error, changed).rest).toBeNull();
  });
});
