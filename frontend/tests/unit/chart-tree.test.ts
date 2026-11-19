// The chart of accounts as tree rows (plan 07, PRD FR-10.1): which rows show for what is open, and
// what each key does. Pure, so no page is needed.

import { describe, expect, it } from 'vitest';
import {
  allKeys,
  controlKey,
  headKey,
  headKeys,
  horizontalMove,
  pagerKey,
  visibleRows,
} from '../../src/features/masters/chart-tree';
import type { AccountNode } from '../../src/bindings/AccountNode';
import {
  SAMPLE_CONTROL_ACCOUNTS,
  balance,
  controlAccountsPage,
  sampleTree,
} from '../helpers/accounts';

const heads = sampleTree();
/** What the tree has read of each opened control: its first page of 25. */
const pages = new Map(
  [101, 4, 11].map((id) => [id, controlAccountsPage(heads, SAMPLE_CONTROL_ACCOUNTS, id, 0, 25)]),
);

describe('visibleRows', () => {
  it('shows only the heads when nothing is open', () => {
    const rows = visibleRows(heads, new Set());
    expect(rows.map((row) => row.key)).toEqual([headKey(1), headKey(3)]);
    expect(rows.every((row) => row.level === 1 && !row.expanded)).toBe(true);
  });

  it('shows the controls of an open head, and the accounts of an open control', () => {
    const rows = visibleRows(heads, new Set([headKey(3), controlKey(4)]), pages);
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

  it('shows nothing under an opened control until its page has been read', () => {
    const rows = visibleRows(heads, new Set([headKey(3), controlKey(4)]));
    expect(rows.map((row) => row.key)).toEqual([
      headKey(1),
      headKey(3),
      controlKey(4),
      controlKey(11),
    ]);
  });

  it('a control with more accounts than one page has a page row with its numbers', () => {
    const many: Record<number, AccountNode[]> = {
      4: Array.from({ length: 60 }, (_, index) => ({
        id: 100 + index,
        code: String(index + 1).padStart(4, '0'),
        name_ur: `خریدار ${String(index + 1)}`,
        name_en: `Buyer ${String(index + 1)}`,
        is_active: true,
        is_system: false,
        balance: balance(index, 'dr'),
      })),
    };
    const second = new Map([[4, controlAccountsPage(heads, many, 4, 25, 25)]]);
    const rows = visibleRows(heads, new Set([headKey(3), controlKey(4)]), second);
    const accounts = rows.filter((row) => row.kind === 'account');
    expect(accounts).toHaveLength(25);
    expect(accounts[0]?.code).toBe('0026');
    const pager = rows.find((row) => row.key === pagerKey(4));
    expect(pager).toMatchObject({ kind: 'pager', level: 3, parent: controlKey(4), balance: null });
    expect(pager?.pager).toEqual({
      controlId: 4,
      page: 2,
      pageCount: 3,
      from: 26,
      to: 50,
      total: 60,
    });
    // The page row comes right after the accounts of its control.
    expect(rows.at(-2)?.key).toBe(pagerKey(4));
  });

  it('starts with the heads open and the controls closed, and can open everything', () => {
    expect(visibleRows(heads, headKeys(heads)).map((row) => row.level)).toEqual([1, 2, 1, 2, 2]);
    expect(allKeys(heads).size).toBe(5);
  });
});

describe('horizontalMove', () => {
  const rows = visibleRows(heads, new Set([headKey(3), controlKey(4)]), pages);
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
