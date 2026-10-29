// The text side of the trips module (Code Standards F39): the new trip form and the entry form of
// the trip screen as text in the screen's state, the input the backend takes for each, and the
// field paths its problems point at. Nothing here works out money (CLAUDE.md rule 1): a share, an
// amount or a quantity is the text the user typed, and the backend reads it.

import { formatBps, isoToDisplayDate } from '../../shared/format';
import { sourceOf } from '../../shared/ui';
import type { ShareInput } from '../partners';
import type { EntryRow } from '../../shared/grid';
import {
  buildPurchaseEntry,
  emptyPurchaseFields,
  purchaseIsFilled,
  textOrNull,
  type PurchaseCommon,
} from '../purchases';
import type {
  EntryDraft,
  EntryPartInput,
  EntryType,
  MoneySource,
  OtherSaleInput,
  TripCostInput,
  TripDefaults,
  TripDraft,
  TripInput,
  VenturePartnerRow,
} from './types';

/** The text a share box shows for a share in bps: empty when he is not in the trip. */
function shareText(shareBps: number): string {
  return shareBps === 0 ? '' : formatBps(shareBps);
}

/**
 * A new trip filled from the last one (PRD A56): every active partner once, with the % and the
 * source of the last trip, or the firm's shares and the shop for the first trip. The user checks
 * and changes. `today` is `YYYY-MM-DD`, or empty while the day is not known yet.
 */
export function newTripDraft(defaults: TripDefaults, today: string): TripDraft {
  return {
    partners: defaults.partners.map((row) => ({
      partner: row.partner,
      share: shareText(row.share_bps),
      source: row.source,
    })),
    date: today === '' ? '' : isoToDisplayDate(today),
    market: '',
    sentPerson: '',
    note: '',
  };
}

/** What the screen sends to the running total: every partner with his typed text (empty is 0). */
export function tripShareInputs(partners: TripDraft['partners']): ShareInput[] {
  return partners.map((row) => ({
    partner_id: row.partner.id,
    share: row.share.trim() === '' ? '0' : row.share.trim(),
  }));
}

/** What goes to the backend when the trip is made. The partners keep the order of the rows. */
export function buildTrip(draft: TripDraft): TripInput {
  return {
    date: draft.date.trim(),
    market: textOrNull(draft.market),
    sent_person: textOrNull(draft.sentPerson),
    note: textOrNull(draft.note),
    partners: tripShareInputs(draft.partners).map((row, index) => ({
      partner_id: row.partner_id,
      share: row.share,
      source: draft.partners[index]?.source ?? 'shop',
    })),
  };
}

/** The fields of the new trip form, by the field paths the backend's problems use. */
export const TRIP_FIELD_PATHS = ['date', 'market', 'sent_person', 'note', 'partners'] as const;

/** True once anything but the filled-in defaults was typed, so leaving asks first. */
export function tripDraftChanged(draft: TripDraft, start: TripDraft): boolean {
  return JSON.stringify({ ...draft, date: '' }) !== JSON.stringify({ ...start, date: '' });
}

/** A new entry form of one type on the day given (`YYYY-MM-DD`, or empty while unknown). */
export function newEntryDraft(type: EntryType, today: string): EntryDraft {
  return {
    type,
    date: today === '' ? '' : isoToDisplayDate(today),
    amount: '',
    note: '',
    sources: {},
    purchase: emptyPurchaseFields(),
    cost: { reason: '' },
    sale: { shop: '', purchaseId: '', qty: '' },
  };
}

/**
 * Changing the type (PRD FR-7.67): the day, the amount and the note stay; the rest starts again.
 * The sources stay between the two payments, a purchase and a cost, because they are one choice of
 * who pays; a sale in another shop has none.
 */
export function changeType(draft: EntryDraft, type: EntryType): EntryDraft {
  const fresh = newEntryDraft(type, '');
  return {
    ...fresh,
    date: draft.date,
    amount: draft.amount,
    note: draft.note,
    sources: type === 'other_sale' ? {} : draft.sources,
  };
}

/** The next entry of the same type after a save: the day stays, the rest starts again. */
export function nextEntryDraft(draft: EntryDraft): EntryDraft {
  const fresh = newEntryDraft(draft.type, '');
  return {
    ...fresh,
    date: draft.date,
    sources: draft.sources,
    // Several lots from one person are likely in one sitting.
    purchase: { ...fresh.purchase, supplier: draft.purchase.supplier },
    sale: { ...fresh.sale, shop: draft.sale.shop },
  };
}

/**
 * True once anything beyond the day is typed or chosen on the entry form. `lines` are the rows of
 * the purchase lines grid (PRD FR-12.41); the other types have none.
 */
export function entryIsFilled(draft: EntryDraft, lines: readonly EntryRow[]): boolean {
  switch (draft.type) {
    case 'purchase':
      return purchaseIsFilled(draft.purchase, lines) || draft.note.trim() !== '';
    case 'cost':
      return draft.amount.trim() !== '' || draft.cost.reason.trim() !== '';
    case 'other_sale':
      return (
        draft.amount.trim() !== '' ||
        draft.sale.shop.trim() !== '' ||
        draft.sale.purchaseId !== '' ||
        draft.sale.qty.trim() !== '' ||
        draft.note.trim() !== ''
      );
  }
}

/** One row for every partner of the trip with this entry's source (the contract asks for all). */
export function entryParts(
  partners: readonly VenturePartnerRow[],
  sources: Readonly<Record<number, MoneySource>>,
): EntryPartInput[] {
  return partners.map((partner) => ({
    partner_id: partner.partner.id,
    source: sourceOf(partner, sources),
  }));
}

/**
 * What the purchase form sends: its fields and every filled line (PRD FR-12.41) with the day, the
 * note and the parts of the screen.
 */
export function buildEntryPurchase(
  draft: EntryDraft,
  lines: readonly EntryRow[],
  tripId: number,
  partners: readonly VenturePartnerRow[],
  confirmCash: boolean,
) {
  const common: PurchaseCommon = {
    tripId,
    date: draft.date,
    note: draft.note,
    parts: entryParts(partners, draft.sources),
  };
  return buildPurchaseEntry(draft.purchase, lines, common, confirmCash);
}

/** What the trip cost form sends. The amount may be a simple sum; the backend reads it. */
export function buildEntryCost(
  draft: EntryDraft,
  tripId: number,
  partners: readonly VenturePartnerRow[],
  confirmCash: boolean,
): TripCostInput {
  return {
    trip_id: tripId,
    date: draft.date.trim(),
    amount: draft.amount.trim(),
    reason: draft.cost.reason.trim(),
    parts: entryParts(partners, draft.sources),
    confirm_cash_below_zero: confirmCash,
  };
}

/** A whole number from text, or null (the id of a chosen purchase). */
function idOf(text: string): number | null {
  const id = Number(text);
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}

/** What the sale in another shop form sends. */
export function buildEntrySale(draft: EntryDraft, tripId: number): OtherSaleInput {
  return {
    trip_id: tripId,
    date: draft.date.trim(),
    shop: textOrNull(draft.sale.shop),
    purchase_id: idOf(draft.sale.purchaseId),
    qty: draft.sale.qty.trim(),
    amount: draft.amount.trim(),
    note: textOrNull(draft.note),
  };
}

/** The field paths of a trip cost's problems. */
export const COST_FIELD_PATHS = ['date', 'amount', 'reason'] as const;

/** The field paths of a sale in another shop's problems. */
export const SALE_FIELD_PATHS = ['date', 'shop', 'purchase_id', 'qty', 'amount', 'note'] as const;
