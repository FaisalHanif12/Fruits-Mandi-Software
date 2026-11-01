// A saved garden entry as the form of its type holds it (plan 44, PRD FR-7.70): the row of the
// garden's history becomes the draft of the installment, cost or sale-in-another-shop form, the
// same draft a new entry uses, so the same builder makes the input. The draft is text, like a typed
// one; nothing here works out money (CLAUDE.md rule 1): an amount of the row is only written out
// with the display formatter.

import { pickedOf, type PickedAccount } from '../../shared/accounts';
import {
  formatBps,
  formatGrams,
  formatPaisa,
  isoToDisplayDate,
  type Grouping,
} from '../../shared/format';
import { textOrNull } from '../purchases';
import { newEntryDraft } from './draft';
import type {
  ArrivalInput,
  ArrivalRow,
  EntryDraft,
  GardenDraft,
  GardenHistoryRow,
  GardenRow,
  MoneySource,
} from './types';

/** The source each partner had on this entry, by partner id. */
function sourcesOf(row: GardenHistoryRow): Record<number, MoneySource> {
  const sources: Record<number, MoneySource> = {};
  for (const part of row.parts) sources[part.partner.id] = part.source;
  return sources;
}

/**
 * The draft of an entry open for edit. A payment has its reason; a sale in another shop has the
 * shop, the qty, the goods arrived it took its نگ from and its note (a sale keeps its note where a
 * payment keeps its reason).
 */
export function draftOfGardenEntry(row: GardenHistoryRow, grouping: Grouping): EntryDraft {
  const kind = row.kind === 'installment' || row.kind === 'cost' ? row.kind : 'other_sale';
  const fresh = newEntryDraft(kind, '');
  const base = {
    ...fresh,
    date: isoToDisplayDate(row.entry_date),
    amount: formatPaisa(row.amount_paisa, { grouping }),
    sources: sourcesOf(row),
  };
  if (kind === 'other_sale') {
    return {
      ...base,
      note: row.note ?? '',
      shop: row.shop_name ?? '',
      qty: row.qty === null ? '' : String(row.qty),
      arrivalId: row.arrival_id === null ? '' : String(row.arrival_id),
    };
  }
  return { ...base, reason: row.reason ?? '' };
}

/** One garden arrival (goods arrived) as the edit form holds it. */
export interface ArrivalEditDraft {
  /** `DD-MM-YYYY`. The day never changes; it is shown and sent as it is. */
  date: string;
  /** `DD-MM-YYYY`, the day to sell. */
  sellDate: string;
  item: PickedAccount | null;
  packing: PickedAccount | null;
  qty: string;
  weight: string;
  note: string;
}

/** The draft of a goods arrived entry open for edit. */
export function arrivalEditDraftOf(row: ArrivalRow): ArrivalEditDraft {
  return {
    date: isoToDisplayDate(row.entry_date),
    sellDate: isoToDisplayDate(row.sell_date),
    item: pickedOf(row.item),
    packing: pickedOf(row.packing),
    qty: String(row.qty),
    weight: row.weight_grams === null ? '' : formatGrams(row.weight_grams),
    note: row.note ?? '',
  };
}

/** What goes to the backend for the review and the save: the text as typed and the ids chosen. */
export function buildArrivalEdit(draft: ArrivalEditDraft, gardenId: number): ArrivalInput {
  return {
    garden_id: gardenId,
    date: draft.date.trim(),
    sell_date: textOrNull(draft.sellDate),
    item_id: draft.item?.id ?? null,
    packing_id: draft.packing?.id ?? null,
    qty: draft.qty.trim(),
    weight: textOrNull(draft.weight),
    note: textOrNull(draft.note),
  };
}

/** The field paths of the problems of a goods arrived edit. */
export const ARRIVAL_EDIT_FIELD_PATHS = [
  'date',
  'sell_date',
  'item_id',
  'packing_id',
  'qty',
  'weight',
  'note',
] as const;

/**
 * The draft of the facts of a garden open for edit (plan 44): the names, the day, the main fruit, the
 * rent, the end of the season and the note, with the partners and their shares as they are (the
 * shares never change after the first entry, PRD FR-12.29). There is no first installment: it is an
 * entry of its own.
 */
export function gardenHeaderDraftOf(garden: GardenRow, grouping: Grouping): GardenDraft {
  return {
    partners: garden.partners.map((row) => ({
      partner: row.partner,
      share: formatBps(row.share_bps),
      source: row.source,
    })),
    names: { urdu: garden.name_ur, english: garden.name_en },
    date: isoToDisplayDate(garden.entry_date),
    item: garden.item === null ? null : pickedOf(garden.item),
    rent: formatPaisa(garden.rent_paisa, { grouping }),
    paidNow: '',
    paidNowReason: '',
    seasonEnd: garden.season_end_date === null ? '' : isoToDisplayDate(garden.season_end_date),
    note: garden.note ?? '',
  };
}
