// A saved trip entry as the form of its type holds it (plan 44, PRD FR-7.70): the row of the trip's
// history becomes the draft of the cost form or of the form of a sale in another shop, the same
// draft a new entry uses, so the same builders make the input. The draft is text, like a typed one;
// nothing here works out money (CLAUDE.md rule 1): an amount of the row is only written out with
// the display formatter.

import { formatPaisa, isoToDisplayDate, type Grouping } from '../../shared/format';
import { newEntryDraft } from './draft';
import type { EntryDraft, MoneySource, TripEntryRow } from './types';

/** The source each partner had on this entry, by partner id. */
function sourcesOf(entry: TripEntryRow): Record<number, MoneySource> {
  const sources: Record<number, MoneySource> = {};
  for (const part of entry.parts) sources[part.partner.id] = part.source;
  return sources;
}

/**
 * The draft of an entry open for edit. A cost has its reason; a sale in another shop has the shop,
 * the purchase, the qty and its note (a sale keeps its note where a cost keeps its reason).
 */
export function draftOfEntry(entry: TripEntryRow, grouping: Grouping): EntryDraft {
  const fresh = newEntryDraft(entry.kind, '');
  const base = {
    ...fresh,
    date: isoToDisplayDate(entry.entry_date),
    amount: formatPaisa(entry.amount_paisa, { grouping }),
    sources: sourcesOf(entry),
  };
  if (entry.kind === 'cost') return { ...base, cost: { reason: entry.reason ?? '' } };
  if (entry.kind === 'other_sale') {
    return {
      ...base,
      note: entry.reason ?? '',
      sale: {
        shop: entry.shop_name ?? '',
        purchaseId: entry.purchase_id === null ? '' : String(entry.purchase_id),
        qty: entry.qty === null ? '' : String(entry.qty),
      },
    };
  }
  return base;
}
