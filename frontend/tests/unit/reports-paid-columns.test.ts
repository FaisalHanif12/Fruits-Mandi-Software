// The money paid on the spot (موقع پر جمع) and what is left on the account (کھاتے میں باقی) as the last two
// columns of R3, R4 and R5 (plan 65, PRD FR-12.44). A table never scrolls sideways (PRD FR-7.2,
// NFR-USE-15), so at each layout step the columns that are shown must fit the room of the step. The
// column lists are the ones the backend writes (backend/crates/reports/src/reports/sales/register.rs
// and buyer_bills.rs); the widths are the ones the grid gives each column type.

import { describe, expect, it } from 'vitest';
import type { Language } from '../../src/i18n';
import { cellText } from '../../src/features/reports/cells';
import { bundles } from '../../src/i18n';
import { message } from '../helpers/render';
import type { ReportColumn } from '../../src/bindings/ReportColumn';
import type { ReportId } from '../../src/bindings/ReportId';
import { GRID_ROOM, reportColumns } from '../../src/features/reports/columns';
import { LIST_COLUMN_WIDTHS, LIST_GROW_MIN_WIDTH } from '../../src/shared/grid';
import type { LayoutStep } from '../../src/shared/hooks/useLayoutStep';

const STEPS: readonly LayoutStep[] = ['compact', 'normal', 'wide'];

type Kind = ReportColumn['kind'];
const column = (key: string, kind: Kind): ReportColumn => ({
  key,
  title_id: `report-col-${key.replaceAll('_', '-')}`,
  kind,
});

/** R3 and R4: the register's columns, then the two new ones (`with_paid`, register.rs). */
const REGISTER: readonly ReportColumn[] = [
  column('date', 'date'),
  column('doc', 'doc'),
  column('buyer', 'party'),
  column('city', 'names'),
  column('item', 'names'),
  column('qty', 'number'),
  column('rate', 'money'),
  column('value', 'money'),
  column('buyer_total', 'money'),
  { key: 'paid', title_id: 'report-col-paid-on-spot', kind: 'money' },
  { key: 'left', title_id: 'report-col-left-on-account', kind: 'money' },
];

/** R5: the buyers' list of the day (buyer_bills.rs). */
const BUYERS_LIST: readonly ReportColumn[] = [
  column('serial', 'number'),
  column('buyer', 'party'),
  column('detail', 'detail'),
  column('value', 'money'),
  column('buyer_total', 'money'),
  { key: 'paid', title_id: 'report-col-paid-on-spot', kind: 'money' },
  { key: 'left', title_id: 'report-col-left-on-account', kind: 'money' },
];

function drawn(id: ReportId, columns: readonly ReportColumn[], step: LayoutStep) {
  return reportColumns(id, columns, step).map((shown) => ({
    field: shown.field,
    width: shown.grow === true ? LIST_GROW_MIN_WIDTH : LIST_COLUMN_WIDTHS[shown.type],
  }));
}

describe.each([
  ['R3', REGISTER],
  ['R4', REGISTER],
  ['R5', BUYERS_LIST],
] as const)('%s with the paid and the left columns', (id, columns) => {
  it.each(STEPS)('fits the room of the %s step, with no sideways scroll', (step) => {
    const total = drawn(id, columns, step).reduce((sum, shown) => sum + shown.width, 0);
    expect(total).toBeLessThanOrEqual(GRID_ROOM[step]);
  });

  it('shows both new columns from the normal step, and the buyer total at every step', () => {
    for (const step of ['normal', 'wide'] as const) {
      const fields = drawn(id, columns, step).map((shown) => shown.field);
      expect(fields).toContain('paid');
      expect(fields).toContain('left');
    }
    for (const step of STEPS) {
      expect(drawn(id, columns, step).map((shown) => shown.field)).toContain('buyer_total');
    }
  });

  it('never takes a column away at a wider step', () => {
    let before: string[] = [];
    for (const step of STEPS) {
      const fields = drawn(id, columns, step).map((shown) => shown.field);
      for (const key of before) expect(fields).toContain(key);
      before = fields;
    }
  });

  it('ends with the paid and the left columns, in the order of the backend', () => {
    const fields = drawn(id, columns, 'wide').map((shown) => shown.field);
    expect(fields.slice(-2)).toEqual(['paid', 'left']);
  });
});

describe('the column each report is about is in the narrowest window', () => {
  it('R3 (money paid on the spot) keeps the paid column at the compact step', () => {
    expect(drawn('R3', REGISTER, 'compact').map((shown) => shown.field)).toContain('paid');
  });

  it('R4 (something left on the account) keeps the left column at the compact step', () => {
    expect(drawn('R4', REGISTER, 'compact').map((shown) => shown.field)).toContain('left');
  });

  it('R5 keeps the paid and the left columns at the compact step; the value of the goods waits', () => {
    const fields = drawn('R5', BUYERS_LIST, 'compact').map((shown) => shown.field);
    expect(fields).toEqual(['serial', 'buyer', 'detail', 'buyer_total', 'paid', 'left']);
  });
});

/** The message IDs the backend sends for the paid amount (report columns, the ledger mark, the slip lines). */
const BACKEND_IDS = [
  'report-col-paid-on-spot',
  'report-col-left-on-account',
  'report-detail-paid-on-spot',
  'print-paid-on-spot',
  'print-left-on-account',
  'working-paid-typed',
  'working-paid-full',
  'working-paid-over',
  'working-paid-kept',
  'working-left',
];

describe('the words of the paid amount are in both languages', () => {
  it.each(BACKEND_IDS)('%s has Urdu and English words that differ', (id) => {
    for (const language of ['ur', 'en'] as const) {
      expect(bundles[language].hasMessage(id), `${language} has ${id}`).toBe(true);
    }
    expect(
      message('ur', id, { number: '1', total: '2', typed: '3', paid: '4', result: '5' }),
    ).not.toBe(message('en', id, { number: '1', total: '2', typed: '3', paid: '4', result: '5' }));
  });

  it('the refusal has both languages and names the most allowed', () => {
    for (const language of ['ur', 'en'] as const) {
      expect(message(language, 'err-paid-exceeds-total', { max: '21,360' })).toContain('21,360');
    }
  });

  it('the Urdu uses the exact glossary words of PRD FR-12.44', () => {
    expect(message('ur', 'glossary-mauqa-par-jama')).toBe('موقع پر جمع');
    expect(message('ur', 'glossary-khate-mein-baqi')).toBe('کھاتے میں باقی');
    expect(message('en', 'glossary-mauqa-par-jama')).toBe('Paid on the Spot');
    expect(message('en', 'glossary-khate-mein-baqi')).toBe('Left on Account');
  });
});

describe('the mark on the credit line of money paid on the spot (R13, R14)', () => {
  const context = (language: Language) => ({
    language,
    text: (id: string, vars?: Record<string, string>) => message(language, id, vars),
    money: () => '',
    signedMoney: () => '',
  });

  it.each(['ur', 'en'] as const)('says the sale and "paid on the spot" (%s)', (language) => {
    const text = cellText(
      { type: 'detail', parts: [{ kind: 'paid_on_spot', bikri_number: 7 }] },
      context(language),
    );
    expect(text).toContain('7');
    expect(text).toContain(message(language, 'glossary-mauqa-par-jama'));
  });
});
