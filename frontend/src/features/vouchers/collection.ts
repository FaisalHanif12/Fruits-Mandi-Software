// The collection round as rows (PRD FR-15.6, FR-7.61): one row for each buyer of the اگراہی فہرست,
// with what the backend knows (his dues, his oldest bill) and one cell to type the amount received
// in. These functions only move the backend's text and ids around and turn the amounts typed into
// the receipts the backend takes. Nothing here works out money (CLAUDE.md rule 1).

import { formatPaisa, type Grouping } from '../../shared/format';
import { displayName } from '../../shared/i18n';
import type { Language } from '../../i18n';
import type { CollectionGroup, CollectionList, ReceiptInput } from './types';

/** The field of the cell the user types the amount received in. */
export const RECEIVED_FIELD = 'received';

/** One buyer of the list, with the collector and the city of the group he is in. */
export interface CollectionRow {
  /** The buyer's account id: the row identity that survives a refresh of the list. */
  accountId: number;
  group: CollectionGroup;
  buyer: CollectionGroup['buyers'][number];
}

/** The buyers of every group, in the order the backend sent them. */
export function collectionRows(list: CollectionList): CollectionRow[] {
  return list.groups.flatMap((group) =>
    group.buyers.map((buyer) => ({ accountId: buyer.account.id, group, buyer })),
  );
}

/** The text of the read-only cells of a row, for the grid's `calc` (names in the app language). */
export function calcOfRow(
  row: CollectionRow,
  language: Language,
  grouping: Grouping,
  groupSeparator: string,
): Record<string, string> {
  const { buyer, group } = row;
  return {
    code: buyer.account.code,
    name: displayName(buyer.account, language),
    group: [group.collector, group.city]
      .filter((item) => item !== null)
      .map((item) => displayName(item, language))
      .join(groupSeparator),
    dues: formatPaisa(buyer.dues_paisa, { grouping }),
    days: buyer.oldest_days === null ? '' : String(buyer.oldest_days),
  };
}

/**
 * The receipts to save: one for each row with an amount typed, with the date of the round and the
 * collector of the buyer's group. A row left empty is not sent. Returns the rows in the order they
 * are sent, because the backend answers with one result for each, in the same order.
 */
export function receiptsOf(
  rows: readonly CollectionRow[],
  typed: ReadonlyMap<number, string>,
  date: string,
): { sent: CollectionRow[]; receipts: ReceiptInput[] } {
  const sent = rows.filter((row) => (typed.get(row.accountId) ?? '').trim() !== '');
  return {
    sent,
    receipts: sent.map((row) => ({
      date,
      account_id: row.accountId,
      person_key: null,
      amount: typed.get(row.accountId) ?? '',
      collector_staff_id: row.group.collector?.id ?? null,
      narration: '',
    })),
  };
}

/**
 * `rows[3].amount` as `{ index: 3, field: 'amount' }`, and `rows[3]` alone (an error of the row
 * with no field) as `{ index: 3, field: '' }`; null for any other path.
 */
export function rowPathOf(path: string): { index: number; field: string } | null {
  const match = /^rows\[(\d+)\](?:\.(.+))?$/.exec(path);
  if (match === null) return null;
  return { index: Number(match[1]), field: match[2] ?? '' };
}
