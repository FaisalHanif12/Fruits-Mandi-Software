// No list scrolls sideways (NFR-USE-15, FR-7.2, Code Standards F22; D1+D2 review, finding 8).
// A list is built from column types with one width each (F18). This adds up the least width of
// the columns shown at each layout step and checks it fits the room the content area has, for
// every master, the account list and the rule list. A column that grows can give up room down
// to LIST_GROW_MIN_WIDTH.

import { describe, expect, it } from 'vitest';
import { ACCOUNT_COLUMNS } from '../../src/features/masters/account-columns';
import { MASTER_CONFIGS, columnsOf } from '../../src/features/masters/configs';
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

  it.each(STEPS)('rule list at the %s step', (step) => {
    expect(leastWidth(RULE_COLUMNS, step)).toBeLessThanOrEqual(ROOM[step]);
  });
});
