// The chart of accounts as the rows the tree shows (PRD FR-10.1): heads, then their controls,
// then the accounts of an opened control, one page at a time (plan 38 B1). A control with more
// accounts than one page has a page row under them. Pure, so the keys can be tested without a page.

import type {
  Balance,
  ChartAccount,
  ChartAccountsPage,
  ChartControl,
  ChartHead,
} from '../../shared/accounts';

export type NodeKind = 'head' | 'control' | 'account' | 'pager';

/** How many accounts of a control one page of the tree shows. */
export const CHART_PAGE_ROWS = 25;

/** Where a control's accounts are: the page row's numbers. Pages count from 1. */
export interface ControlPager {
  controlId: number;
  page: number;
  pageCount: number;
  from: number;
  to: number;
  total: number;
}

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
  /** The tag (نشان) after an account's name, shown by `displayName`. Absent when there is none. */
  tag?: string;
  /** The page row has none. */
  balance: Balance | null;
  /** Heads and controls can be opened; an account cannot. */
  expandable: boolean;
  expanded: boolean;
  /** The key of the row above it in the chart (its head or control), or null for a head. */
  parent: string | null;
  inactive: boolean;
  /** The page row of an opened control with more accounts than one page. */
  pager?: ControlPager;
}

export const headKey = (id: number) => `head:${String(id)}`;
export const controlKey = (id: number) => `control:${String(id)}`;
export const accountKey = (id: number) => `account:${String(id)}`;
export const pagerKey = (controlId: number) => `pager:${String(controlId)}`;

function accountRow(account: ChartAccount, parent: string): TreeRow {
  return {
    key: accountKey(account.id),
    kind: 'account',
    level: 3,
    id: account.id,
    code: account.code,
    name_ur: account.name_ur,
    name_en: account.name_en,
    ...(account.tag === undefined ? {} : { tag: account.tag }),
    balance: account.balance,
    expandable: false,
    expanded: false,
    parent,
    inactive: !account.is_active,
  };
}

/**
 * The rows to show now: a head always, its controls when it is open, and the accounts of an opened
 * control as far as the page that was read (`accounts`, by control id). A control with several pages
 * has a page row after its accounts.
 */
export function visibleRows(
  heads: readonly ChartHead[],
  expanded: ReadonlySet<string>,
  accounts: ReadonlyMap<number, ChartAccountsPage> = new Map(),
): TreeRow[] {
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
      const page = accounts.get(control.id);
      if (page === undefined) continue;
      for (const account of page.rows) rows.push(accountRow(account, ck));
      const pager = pagerOf(page);
      if (pager !== null) rows.push(pagerRow(pager, ck));
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
    expandable: control.account_count > 0,
    expanded: open,
    parent,
    inactive: false,
  };
}

/** The numbers of a control's page row, or null when its accounts fit one page. */
function pagerOf(page: ChartAccountsPage): ControlPager | null {
  if (page.total_count <= CHART_PAGE_ROWS) return null;
  const pageCount = Math.ceil(page.total_count / page.limit);
  const number = Math.floor(page.offset / page.limit) + 1;
  return {
    controlId: page.control_id,
    page: number,
    pageCount,
    from: page.offset + 1,
    to: Math.min(page.offset + page.limit, page.total_count),
    total: page.total_count,
  };
}

function pagerRow(pager: ControlPager, parent: string): TreeRow {
  return {
    key: pagerKey(pager.controlId),
    kind: 'pager',
    level: 3,
    id: pager.controlId,
    code: '',
    name_ur: '',
    name_en: '',
    balance: null,
    expandable: false,
    expanded: false,
    parent,
    inactive: false,
    pager,
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
