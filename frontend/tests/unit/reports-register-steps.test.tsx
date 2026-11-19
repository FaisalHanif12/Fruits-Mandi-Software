// The sales registers R2, R3 and R4 when the backend adds columns (plan 59, D7): the quality (PRD
// FR-12.42), the packing (when the filter has more than one, PRD FR-7.53) and the seller's rate (when
// a line has a rate difference, PRD FR-12.3). A table never scrolls sideways (PRD FR-7.2, NFR-USE-15),
// so at each layout step the columns that are shown must fit the room of the step, whatever mix of the
// three is there. The sums use the widths of the column types that the grid uses. The screen test
// draws the register at the compact step (1024 px) in the Urdu and in the English app; the pictures
// at 1024 x 600 in both languages are taken from the development page (`/dev/reports/R2?threecols`).

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportColumn } from '../../src/bindings/ReportColumn';
import type { ReportId } from '../../src/bindings/ReportId';
import type { ReportPage } from '../../src/bindings/ReportPage';
import { GRID_ROOM, reportColumns } from '../../src/features/reports/columns';
import { LANGUAGES, type Language } from '../../src/i18n';
import { LIST_COLUMN_WIDTHS, LIST_GROW_MIN_WIDTH } from '../../src/shared/grid';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { R2_THREE_COLUMNS } from '../helpers/register-three-columns';
import { fakeReports } from '../helpers/reports';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

const STEPS: readonly LayoutStep[] = ['compact', 'normal', 'wide'];
const REGISTERS: readonly ReportId[] = ['R2', 'R3', 'R4'];
const OPTIONAL = ['grade', 'packing', 'seller_rate'] as const;
/** What a register always shows: the buyer, the item, the quantity, the rate and the buyer's total. */
const ALWAYS = ['buyer', 'item', 'qty', 'rate', 'buyer_total'];

/** The register with only some of the three optional columns (the backend leaves the others out). */
function registerWith(keep: readonly string[]): readonly ReportColumn[] {
  return R2_THREE_COLUMNS.columns
    .filter((column) => !OPTIONAL.some((key) => key === column.key) || keep.includes(column.key))
    .map((column) =>
      // The rate has the heading "Rate, Buyer" only when the seller's rate is beside it.
      column.key === 'rate' && !keep.includes('seller_rate')
        ? { ...column, title_id: 'report-col-rate' }
        : column,
    );
}

/** Every mix of the three optional columns: none, each alone, each pair and all three. */
const MIXES: readonly (readonly string[])[] = [
  [],
  ['grade'],
  ['packing'],
  ['seller_rate'],
  ['grade', 'packing'],
  ['grade', 'seller_rate'],
  ['packing', 'seller_rate'],
  ['grade', 'packing', 'seller_rate'],
];

/** The columns drawn at a step, with the width the grid gives each at least. */
function shown(
  id: ReportId,
  columns: readonly ReportColumn[],
  step: LayoutStep,
): { field: string; width: number }[] {
  return reportColumns(id, columns, step).map((column) => ({
    field: column.field,
    width: column.grow === true ? LIST_GROW_MIN_WIDTH : LIST_COLUMN_WIDTHS[column.type],
  }));
}

describe.each(REGISTERS)('the columns of %s at each layout step (no sideways scroll)', (id) => {
  describe.each(MIXES.map((mix) => [mix.join(' + ') || 'no optional column', mix] as const))(
    'with %s',
    (_name, mix) => {
      const columns = registerWith(mix);

      it.each(STEPS)('fits the room of the %s step and keeps the always-shown columns', (step) => {
        const drawn = shown(id, columns, step);
        const total = drawn.reduce((sum, column) => sum + column.width, 0);
        expect(total).toBeLessThanOrEqual(GRID_ROOM[step]);
        const fields = drawn.map((column) => column.field);
        for (const key of ALWAYS) expect(fields).toContain(key);
      });

      it('never takes a column away at a wider step', () => {
        let before: string[] = [];
        for (const step of STEPS) {
          const fields = shown(id, columns, step).map((column) => column.field);
          for (const key of before) expect(fields).toContain(key);
          before = fields;
        }
      });
    },
  );

  it('with all three: the quality at the compact step, and the day and the value only at the wide one', () => {
    const columns = registerWith(OPTIONAL);
    const at = (step: LayoutStep) => shown(id, columns, step).map((column) => column.field);
    expect(at('compact')).toEqual(['buyer', 'item', 'grade', 'qty', 'rate', 'buyer_total']);
    expect(at('normal')).toEqual([
      'buyer',
      'item',
      'packing',
      'grade',
      'qty',
      'rate',
      'seller_rate',
      'buyer_total',
    ]);
    expect(at('wide')).toEqual([
      'date',
      'doc',
      'buyer',
      'item',
      'packing',
      'grade',
      'qty',
      'rate',
      'seller_rate',
      'value',
      'buyer_total',
    ]);
  });

  it('with none of them: it is the register as it was (the day at every step, the entry and the value from normal, the city at wide)', () => {
    const columns = registerWith([]);
    const at = (step: LayoutStep) => shown(id, columns, step).map((column) => column.field);
    expect(at('compact')).toEqual(['date', 'buyer', 'item', 'qty', 'rate', 'buyer_total']);
    expect(at('normal')).toEqual([
      'date',
      'doc',
      'buyer',
      'item',
      'qty',
      'rate',
      'value',
      'buyer_total',
    ]);
    expect(at('wide')).toContain('city');
  });
});

describe.each(LANGUAGES)(
  'R2 with the quality, the packing and the seller’s rate (%s)',
  (language: Language) => {
    beforeEach(() => {
      vi.resetAllMocks();
      vi.mocked(useLayoutStep).mockReturnValue('compact');
      healthyBackend();
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    afterEach(() => {
      leaveAppWindow();
    });

    const m = (id: string) => message(language, id);

    function drawnColumns(): string[] {
      return Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map(
        (header) => header.getAttribute('col-id') ?? '',
      );
    }

    async function openRegister(page: ReportPage) {
      fakeReports({ pages: { R2: page } });
      await renderApp('/reports/R2');
      await waitFor(
        () => {
          expect(drawnColumns().length).toBeGreaterThan(0);
        },
        { timeout: 8_000 },
      );
    }

    it('draws the compact window with the quality and no more columns than 890 px hold', async () => {
      await openRegister(R2_THREE_COLUMNS);
      expect(drawnColumns()).toEqual(['buyer', 'item', 'grade', 'qty', 'rate', 'buyer_total']);
      // The rate says whose it is, because the seller's rate is on the page (in the print and at the
      // normal step), and the quality has its own heading.
      const heading = (key: string) =>
        visible(document.querySelector(`.ag-header-cell[col-id="${key}"]`)?.textContent).trim();
      expect(heading('rate')).toBe(m('report-col-buyer-rate'));
      expect(heading('grade')).toBe(m('report-col-grade'));
      expect(screen.queryByRole('columnheader', { name: m('report-col-seller-rate') })).toBeNull();
    });

    it('draws the three extra columns at the normal step, still with no sideways scroll', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('normal');
      await openRegister(R2_THREE_COLUMNS);
      expect(drawnColumns()).toEqual([
        'buyer',
        'item',
        'packing',
        'grade',
        'qty',
        'rate',
        'seller_rate',
        'buyer_total',
      ]);
      const heading = (key: string) =>
        visible(document.querySelector(`.ag-header-cell[col-id="${key}"]`)?.textContent).trim();
      expect(heading('rate')).toBe(m('report-col-buyer-rate'));
      expect(heading('seller_rate')).toBe(m('report-col-seller-rate'));
    });
  },
);
