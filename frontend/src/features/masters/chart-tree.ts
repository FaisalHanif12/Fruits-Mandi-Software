// The chart of accounts as the rows the tree shows (PRD FR-10.1): heads, then their controls,
// then their accounts, only for what is expanded. Pure, so the keys can be tested without a page.

import type { Balance, ChartAccount, ChartControl, ChartHead } from '../../shared/accounts';

export type NodeKind = 'head' | 'control' | 'account';

/** One visible row of the tree. */
export interface TreeRow {
  /** Stable: `head:1`, `control:5`, `account:12`. */
  key: string;
  kind: NodeKind;
  /** 1 for a head, 2 for a control, 3 for an account (aria-level). */
  level: 1 | 2 | 3;
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
  balance: Balance;
  /** Heads and controls can be opened; an account cannot. */
  expandable: boolean;
  expanded: boolean;
  /** The key of the row above it in the chart (its head or control), or null for a head. */
  parent: string | null;
  inactive: boolean;
}

export const headKey = (id: number) => `head:${String(id)}`;
export const controlKey = (id: number) => `control:${String(id)}`;
export const accountKey = (id: number) => `account:${String(id)}`;

function accountRow(account: ChartAccount, parent: string): TreeRow {
  return {
    key: accountKey(account.id),
    kind: 'account',
    level: 3,
    id: account.id,
    code: account.code,
    name_ur: account.name_ur,
    name_en: account.name_en,
    balance: account.balance,
    expandable: false,
    expanded: false,
    parent,
    inactive: !account.is_active,
  };
}

/** The rows to show now: a head always, its controls when it is open, their accounts when open. */
export function visibleRows(heads: readonly ChartHead[], expanded: ReadonlySet<string>): TreeRow[] {
  const rows: TreeRow[] = [];
  for (const head of heads) {
    const hk = headKey(head.id);
    const headOpen = expanded.has(hk);
    rows.push({
      key: hk,
      kind: 'head',
      level: 1,
      id: head.id,
      code: head.code,
      name_ur: head.name_ur,
      name_en: head.name_en,
      balance: head.balance,
      expandable: head.controls.length > 0,
      expanded: headOpen,
      parent: null,
      inactive: false,
    });
    if (!headOpen) continue;
    for (const control of head.controls) {
      const ck = controlKey(control.id);
      const controlOpen = expanded.has(ck);
      rows.push(controlRow(control, hk, controlOpen));
      if (!controlOpen) continue;
      for (const account of control.accounts) rows.push(accountRow(account, ck));
    }
  }
  return rows;
}

function controlRow(control: ChartControl, parent: string, open: boolean): TreeRow {
  return {
    key: controlKey(control.id),
    kind: 'control',
    level: 2,
    id: control.id,
    code: control.code,
    name_ur: control.name_ur,
    name_en: control.name_en,
    balance: control.balance,
    expandable: control.accounts.length > 0,
    expanded: open,
    parent,
    inactive: false,
  };
}

/** The keys of every head, to start with the heads open and the controls closed. */
export function headKeys(heads: readonly ChartHead[]): Set<string> {
  return new Set(heads.map((head) => headKey(head.id)));
}

/** The keys of everything that can be opened, to open the whole chart. */
export function allKeys(heads: readonly ChartHead[]): Set<string> {
  const keys = new Set<string>();
  for (const head of heads) {
    keys.add(headKey(head.id));
    for (const control of head.controls) keys.add(controlKey(control.id));
  }
  return keys;
}

/** What a key does on a row: the side that opens it is the side the user sees the children on. */
export type TreeMove =
  | { to: 'index'; index: number }
  | { to: 'toggle'; key: string; open: boolean }
  | { to: 'open-account'; id: number }
  | { to: 'none' };

/**
 * What the arrow that points at the children (`opens`) or at the parent (`closes`) does on a row:
 * open it, or go to its first child; close it, or go to its parent. In left-to-right the right
 * arrow opens; in right-to-left the left arrow does (PRD FR-7.13, FR-7.34).
 */
export function horizontalMove(
  rows: readonly TreeRow[],
  index: number,
  action: 'opens' | 'closes',
): TreeMove {
  const row = rows[index];
  if (row === undefined) return { to: 'none' };
  if (action === 'opens') {
    if (row.expandable && !row.expanded) return { to: 'toggle', key: row.key, open: true };
    if (row.expandable && row.expanded) return { to: 'index', index: index + 1 };
    return { to: 'none' };
  }
  if (row.expandable && row.expanded) return { to: 'toggle', key: row.key, open: false };
  const parentIndex = rows.findIndex((candidate) => candidate.key === row.parent);
  return parentIndex >= 0 ? { to: 'index', index: parentIndex } : { to: 'none' };
}
