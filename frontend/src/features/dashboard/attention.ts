// What each line of the attention list says and looks like (PRD FR-7.44). The backend sends the
// kind, the count and the figures it worked out; the words are Fluent, one message for each kind
// (`attention-<kind>`), and the figures go into them as they came (money through `formatPaisa`).
// Nothing is worked out here (CLAUDE.md rule 1).

import type { AttentionItem } from '../../bindings/AttentionItem';
import type { AttentionKind } from '../../bindings/AttentionKind';
import type { Paisa } from '../../shared/format';

/** The Fluent message of a kind: `attention-over-credit-limit` for `over_credit_limit`. */
export function messageOfKind(kind: AttentionKind): string {
  return `attention-${kind.replaceAll('_', '-')}`;
}

/**
 * The kinds that have a second message for when the backend also names an age in days. The age is
 * not always there (a payable with no dated bill), so the plain message must read well alone.
 */
const AGED_KINDS: ReadonlySet<AttentionKind> = new Set([
  'stock_waiting',
  'dues_old',
  'payables_old',
  'loans_overdue',
  'takes_overdue',
]);

/** The words of the backup line when there is no copy yet. */
export const BACKUP_NEVER_MESSAGE = 'attention-backup-never';

/** The message and the variables of one line. `money` writes an amount with the app's grouping. */
export function wordsOfItem(
  item: AttentionItem,
  money: (amount: Paisa) => string,
): { id: string; vars: Record<string, string | number> } {
  // A backup that was never made has no age to say.
  if (item.kind === 'backup_old' && item.days === null) {
    return { id: BACKUP_NEVER_MESSAGE, vars: {} };
  }
  const aged = item.days !== null && AGED_KINDS.has(item.kind);
  return {
    id: aged ? `${messageOfKind(item.kind)}-aged` : messageOfKind(item.kind),
    vars: {
      // A number, so the message can choose its singular or plural form.
      count: item.count,
      days: item.days ?? 0,
      qty: item.qty ?? 0,
      amount: item.amount_paisa === null ? '' : money(item.amount_paisa),
    },
  };
}
