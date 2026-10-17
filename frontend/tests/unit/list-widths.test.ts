// No list scrolls sideways (NFR-USE-15, FR-7.2, Code Standards F22; D1+D2 review, finding 8).
// A list is built from column types with one width each (F18). This adds up the least width of
// the columns shown at each layout step and checks it fits the room the content area has, for
// every master, the account list and the rule list. A column that grows can give up room down
// to LIST_GROW_MIN_WIDTH.

import { describe, expect, it } from 'vitest';
import { ACCOUNT_COLUMNS, accountColumns } from '../../src/features/masters/account-columns';
import { MASTER_CONFIGS, columnsOf } from '../../src/features/masters/configs';
import { BIKRI_LIST_COLUMNS, bikriListColumns } from '../../src/features/bikri/list-columns';
import { PUKHTA_COLUMNS, pukhtaColumns } from '../../src/features/pukhta_khata/columns';
import { VOUCHER_COLUMNS } from '../../src/features/vouchers/list-columns';
import { RULE_COLUMNS } from '../../src/features/rules/columns';
import type { ListColumn } from '../../src/shared/grid';
import { LIST_COLUMN_WIDTHS, LIST_GROW_MIN_WIDTH } from '../../src/shared/grid/columnTypes';
import { isFromStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';

// What the content area has at the narrowest window of each step, measured on the screens
// (1024 wide: 892; 1366 wide: 1074; 1920 wide: the content stops at 1600).
const ROOM: Record<LayoutStep, number> = { compact: 892, normal: 1074, wide: 1532 };

function leastWidth<T>(columns: readonly ListColumn<T>[], step: LayoutStep): number {
  return columns
    .filter((column) => isFromStep(step, column.from ?? 'compact'))
    .reduce(
      (sum, column) =>
        sum + (column.grow === true ? LIST_GROW_MIN_WIDTH : LIST_COLUMN_WIDTHS[column.type]),
      0,
    );
}

const STEPS: LayoutStep[] = ['compact', 'normal', 'wide'];

describe('lists fit the content area', () => {
  for (const config of MASTER_CONFIGS) {
    it.each(STEPS)(`${config.kind} list at the %s step`, (step) => {
      expect(leastWidth(columnsOf(config), step)).toBeLessThanOrEqual(ROOM[step]);
    });
  }

  it.each(STEPS)('account list at the %s step', (step) => {
    expect(leastWidth(ACCOUNT_COLUMNS, step)).toBeLessThanOrEqual(ROOM[step]);
  });

  // "More columns" adds the opening debit and credit below the wide step (N3). They must fit too.
  it.each(STEPS)('account list with More columns at the %s step', (step) => {
    expect(leastWidth(accountColumns(step, true), step)).toBeLessThanOrEqual(ROOM[step]);
  });

  it('More columns shows the opening debit and credit at every step', () => {
    for (const step of STEPS) {
      const fields = accountColumns(step, true)
        .filter((column) => isFromStep(step, column.from ?? 'compact'))
        .map((column) => column.field);
      expect(fields).toContain('opening_dr');
      expect(fields).toContain('opening_cr');
    }
  });

  it.each(STEPS)('rule list at the %s step', (step) => {
    expect(leastWidth(RULE_COLUMNS, step)).toBeLessThanOrEqual(ROOM[step]);
  });

  // The بکری area has its 200 px side menu beside the screen (design system 6), which the lists of
  // انڈکس do not (plan 13): the lists of بکری and of the پختہ کھاتہ accounts have that much less.
  describe('lists beside the side menu of the بکری area', () => {
    const SIDE_MENU = 200;
    it.each(STEPS)('list of بکری at the %s step', (step) => {
      expect(leastWidth(BIKRI_LIST_COLUMNS, step)).toBeLessThanOrEqual(ROOM[step] - SIDE_MENU);
    });
    it.each(STEPS)('register of the پختہ کھاتہ accounts at the %s step', (step) => {
      expect(leastWidth(PUKHTA_COLUMNS, step)).toBeLessThanOrEqual(ROOM[step] - SIDE_MENU);
    });

    it.each(STEPS)('list of vouchers under the voucher form at the %s step', (step) => {
      expect(leastWidth(VOUCHER_COLUMNS, step)).toBeLessThanOrEqual(ROOM[step] - SIDE_MENU);
    });

    // "More columns" adds the نگ column of FR-12.7 and FR-13.6, which fits at no step beside the
    // others. A column steps aside where the room is short, so nothing scrolls sideways.
    it.each(STEPS)('list of بکری with More columns at the %s step', (step) => {
      expect(leastWidth(bikriListColumns(step, true), step)).toBeLessThanOrEqual(
        ROOM[step] - SIDE_MENU,
      );
    });
    it.each(STEPS)('register with More columns at the %s step', (step) => {
      expect(leastWidth(pukhtaColumns(step, true), step)).toBeLessThanOrEqual(
        ROOM[step] - SIDE_MENU,
      );
    });

    it('the نگ column is in neither list without the tick, at any step', () => {
      for (const step of STEPS) {
        expect(bikriListColumns(step, false).map((column) => column.field)).not.toContain('qty');
        expect(pukhtaColumns(step, false).map((column) => column.field)).not.toContain('qty');
      }
    });

    it('with the tick the نگ column is shown at every step, in both lists', () => {
      for (const step of STEPS) {
        for (const columns of [bikriListColumns(step, true), pukhtaColumns(step, true)]) {
          const shown = columns
            .filter((column) => isFromStep(step, column.from ?? 'compact'))
            .map((column) => column.field);
          expect(shown).toContain('qty');
        }
      }
    });

    it('with the tick the amounts and the status of a بکری stay at every step', () => {
      for (const step of STEPS) {
        const shown = bikriListColumns(step, true)
          .filter((column) => isFromStep(step, column.from ?? 'compact'))
          .map((column) => column.field);
        expect(shown).toEqual(expect.arrayContaining(['number', 'sellers', 'net', 'status']));
      }
    });
  });
});
