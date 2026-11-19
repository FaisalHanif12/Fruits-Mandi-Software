// A saved trip purchase as the edit form holds it (plan 44, PRD FR-7.70, FR-12.19): the row of the
// purchase becomes the text of the form, the same fields as a new purchase, and the input the backend
// takes for the review and the save is built from that text. The draft is text, like a typed one;
// nothing here works out money (CLAUDE.md rule 1): an amount of the row is only written out with
// the display formatter, and the backend reads it back.

import type { EntryPart } from '../../bindings/EntryPart';
import type { MoneySource } from '../../bindings/MoneySource';
import type { VenturePartnerRow } from '../../bindings/VenturePartnerRow';
import { pickedOf, type PickedAccount } from '../../shared/accounts';
import { formatGrams, formatPaisa, isoToDisplayDate, type Grouping } from '../../shared/format';
import { sourceOf } from '../../shared/ui';
import { textOrNull } from './draft';
import type { PurchaseInput, PurchaseRow } from './types';

/** One purchase as the edit form holds it. */
export interface PurchaseEditDraft {
  /** `DD-MM-YYYY`. The day never changes; it is shown and sent as it is. */
  date: string;
  supplier: string;
  /** `DD-MM-YYYY`, the day to sell. */
  sellDate: string;
  item: PickedAccount | null;
  packing: PickedAccount | null;
  qty: string;
  weight: string;
  rate: string;
  freight: string;
  labour: string;
  otherCost: string;
  /** The نگ for another shop, or empty for none. */
  otherQty: string;
  shopName: string;
  note: string;
  /** This purchase's choice of source by partner id. */
  sources: Record<number, MoneySource>;
}

/** An amount that is 0 is an empty box, as in a new purchase. */
function amountText(paisa: PurchaseRow['freight_paisa'], grouping: Grouping): string {
  const text = formatPaisa(paisa, { grouping });
  return text === '0' ? '' : text;
}

/** The draft of a purchase open for edit. `parts` are the sources it was saved with. */
export function purchaseEditDraftOf(
  row: PurchaseRow,
  parts: readonly Pick<EntryPart, 'partner' | 'source'>[],
  grouping: Grouping,
): PurchaseEditDraft {
  const sources: Record<number, MoneySource> = {};
  for (const part of parts) sources[part.partner.id] = part.source;
  return {
    date: isoToDisplayDate(row.entry_date),
    supplier: row.supplier ?? '',
    sellDate: isoToDisplayDate(row.sell_date),
    item: pickedOf(row.item),
    packing: pickedOf(row.packing),
    qty: String(row.qty),
    weight: row.weight_grams === null ? '' : formatGrams(row.weight_grams),
    rate: formatPaisa(row.rate_paisa, { grouping }),
    freight: amountText(row.freight_paisa, grouping),
    labour: amountText(row.labour_paisa, grouping),
    otherCost: amountText(row.other_cost_paisa, grouping),
    otherQty: row.qty_other === 0 ? '' : String(row.qty_other),
    shopName: row.shop_name ?? '',
    note: row.note ?? '',
    sources,
  };
}

/** What goes to the backend for the review and the save: the text as typed and the ids chosen. */
export function buildPurchaseEdit(
  draft: PurchaseEditDraft,
  tripId: number,
  partners: readonly VenturePartnerRow[],
  confirmCash: boolean,
): PurchaseInput {
  return {
    trip_id: tripId,
    date: draft.date.trim(),
    supplier: textOrNull(draft.supplier),
    item_id: draft.item?.id ?? null,
    packing_id: draft.packing?.id ?? null,
    qty: draft.qty.trim(),
    weight: textOrNull(draft.weight),
    sell_date: textOrNull(draft.sellDate),
    rate: draft.rate.trim(),
    freight: textOrNull(draft.freight),
    labour: textOrNull(draft.labour),
    other_cost: textOrNull(draft.otherCost),
    note: textOrNull(draft.note),
    parts: partners.map((partner) => ({
      partner_id: partner.partner.id,
      source: sourceOf(partner, draft.sources),
    })),
    confirm_cash_below_zero: confirmCash,
    other_qty: textOrNull(draft.otherQty),
    shop_name: textOrNull(draft.shopName),
  };
}

/** The field paths of a purchase's problems. */
export const PURCHASE_EDIT_FIELD_PATHS = [
  'date',
  'supplier',
  'sell_date',
  'item_id',
  'packing_id',
  'qty',
  'weight',
  'rate',
  'freight',
  'labour',
  'other_cost',
  'note',
  'other_qty',
  'shop_name',
] as const;
