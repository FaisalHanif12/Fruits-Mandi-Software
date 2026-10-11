// The chart of accounts as tree rows (plan 07, PRD FR-10.1): which rows show for what is open, and
// what each key does. Pure, so no page is needed.

import { describe, expect, it } from 'vitest';
import {
  allKeys,
  controlKey,
  headKey,
  headKeys,
  horizontalMove,
  moveActive,
  visibleRows,
} from '../../src/features/masters/chart-tree';
import { sampleTree } from '../helpers/accounts';

const heads = sampleTree();

describe('visibleRows', () => {
  it('shows only the heads when nothing is open', () => {
    const rows = visibleRows(heads, new Set());
    expect(rows.map((row) => row.key)).toEqual([headKey(1), headKey(3)]);
    expect(rows.every((row) => row.level === 1 && !row.expanded)).toBe(true);
  });

  it('shows the controls of an open head, and the accounts of an open control', () => {
    const rows = visibleRows(heads, new Set([headKey(3), controlKey(4)]));
    expect(rows.map((row) => row.key)).toEqual([
      headKey(1),
      headKey(3),
      controlKey(4),
      'account:21',
      'account:22',
      controlKey(11),
    ]);
    const akram = rows.find((row) => row.key === 'account:21');
    expect(akram).toMatchObject({ level: 3, expandable: false, parent: controlKey(4) });
    // A control with no account in it cannot be opened.
    expect(rows.find((row) => row.key === controlKey(11))?.expandable).toBe(false);
  });

  it('starts with the heads open and the controls closed, and can open everything', () => {
    expect(visibleRows(heads, headKeys(heads)).map((row) => row.level)).toEqual([1, 2, 1, 2, 2]);
    expect(allKeys(heads).size).toBe(5);
  });
});

describe('moveActive', () => {
  it('moves one row or a page, and stops at the ends', () => {
    expect(moveActive(0, 5, 'ArrowDown', 10)).toBe(1);
    expect(moveActive(0, 5, 'ArrowUp', 10)).toBe(0);
    expect(moveActive(2, 50, 'PageDown', 10)).toBe(12);
    expect(moveActive(2, 5, 'PageDown', 10)).toBe(4);
    expect(moveActive(3, 5, 'Home', 10)).toBe(0);
    expect(moveActive(0, 5, 'End', 10)).toBe(4);
    expect(moveActive(0, 5, 'x', 10)).toBeNull();
    expect(moveActive(0, 0, 'ArrowDown', 10)).toBeNull();
  });
});

describe('horizontalMove', () => {
  const rows = visibleRows(heads, new Set([headKey(3), controlKey(4)]));
  const index = (key: string) => rows.findIndex((row) => row.key === key);

  it('the arrow at the children opens a closed row, then goes to its first child', () => {
    expect(horizontalMove(rows, index(headKey(1)), 'opens')).toEqual({
      to: 'toggle',
      key: headKey(1),
      open: true,
    });
    expect(horizontalMove(rows, index(headKey(3)), 'opens')).toEqual({
      to: 'index',
      index: index(controlKey(4)),
    });
    expect(horizontalMove(rows, index('account:21'), 'opens')).toEqual({ to: 'none' });
  });

  it('the arrow at the parent closes an open row, else goes to the parent', () => {
    expect(horizontalMove(rows, index(controlKey(4)), 'closes')).toEqual({
      to: 'toggle',
      key: controlKey(4),
      open: false,
    });
    expect(horizontalMove(rows, index('account:22'), 'closes')).toEqual({
      to: 'index',
      index: index(controlKey(4)),
    });
    expect(horizontalMove(rows, index(headKey(1)), 'closes')).toEqual({ to: 'none' });
  });
});
