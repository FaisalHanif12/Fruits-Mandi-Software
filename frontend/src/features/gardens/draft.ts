// The text side of the gardens module (Code Standards F39): the new garden form and the entry form
// of the garden screen as text in the screen's state, the input the backend takes for each, and the
// field paths its problems point at. Nothing here works out money (CLAUDE.md rule 1): a share, an
// amount or a quantity is the text the user typed, and the backend reads it.

import { formatBps, isoToDisplayDate } from '../../shared/format';
import { sourceOf } from '../../shared/ui';
import { confirmedSpelling } from '../../shared/urdu/spelling';
import type { ShareInput } from '../partners';
import type { EntryRow } from '../../shared/grid';
import { arrivalLineInputs, linesAreFilled, textOrNull } from '../purchases';
import type {
  ArrivalEntryInput,
  EntryDraft,
  EntryPartInput,
  EntryType,
  GardenDefaults,
  GardenDraft,
  GardenEntryInput,
  GardenInput,
  MoneySource,
  VenturePartnerRow,
} from './types';

/** The text a share box shows for a share in bps: empty when he is not in the garden. */
function shareText(shareBps: number): string {
  return shareBps === 0 ? '' : formatBps(shareBps);
}

/**
 * A new garden filled from the firm's shares or the last garden (PRD A56): every active partner
 * once, with a % and a source. The user checks and changes. `today` is `YYYY-MM-DD`, or empty
 * while the day is not known yet.
 */
export function newGardenDraft(defaults: GardenDefaults, today: string): GardenDraft {
  return {
    partners: defaults.partners.map((row) => ({
      partner: row.partner,
      share: shareText(row.share_bps),
      source: row.source,
    })),
    names: { urdu: '', english: '' },
    date: today === '' ? '' : isoToDisplayDate(today),
    item: null,
    rent: '',
    paidNow: '',
    paidNowReason: '',
    seasonEnd: '',
    note: '',
  };
}

/** What the screen sends to the running total: every partner with his typed text (empty is 0). */
export function gardenShareInputs(partners: GardenDraft['partners']): ShareInput[] {
  return partners.map((row) => ({
    partner_id: row.partner.id,
    share: row.share.trim() === '' ? '0' : row.share.trim(),
  }));
}

/**
 * What goes to the backend when the garden is made. The partners keep the order of the rows. The
 * first installment takes the sources of the partners' rows (`parts` stays empty, which the
 * backend reads as "the garden's default sources").
 */
export function buildGarden(draft: GardenDraft, confirmCash: boolean): GardenInput {
  return {
    name_ur: draft.names.urdu.trim(),
    name_en: draft.names.english.trim(),
    confirmed_rules_words: confirmedSpelling(draft.names),
    date: draft.date.trim(),
    item_id: draft.item?.id ?? null,
    rent: draft.rent.trim(),
    season_end: textOrNull(draft.seasonEnd),
    note: textOrNull(draft.note),
    partners: gardenShareInputs(draft.partners).map((row, index) => ({
      partner_id: row.partner_id,
      share: row.share,
      source: draft.partners[index]?.source ?? 'shop',
    })),
    paid_now: textOrNull(draft.paidNow),
    // The reason belongs to the first installment only; without an amount there is none.
    paid_now_reason: textOrNull(draft.paidNow) === null ? null : textOrNull(draft.paidNowReason),
    parts: [],
    confirm_cash_below_zero: confirmCash,
  };
}

/** The fields of the new garden form, by the field paths the backend's problems use. */
export const GARDEN_FIELD_PATHS = [
  'name_ur',
  'name_en',
  'date',
  'item_id',
  'rent',
  'season_end',
  'note',
  'paid_now',
  'paid_now_reason',
  'partners',
] as const;

/** True once anything but the filled-in defaults was typed, so leaving asks first. */
export function gardenDraftChanged(draft: GardenDraft, start: GardenDraft): boolean {
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
    reason: '',
    shop: '',
    qty: '',
    arrivalId: '',
    sellDate: '',
  };
}

/**
 * Changing the type (PRD FR-7.67): the day, the amount and the note stay; the rest starts again.
 * The sources stay between the two payments, an installment and a cost, because they are one
 * choice of who pays; a sale in another shop has none.
 */
export function changeType(draft: EntryDraft, type: EntryType): EntryDraft {
  const fresh = newEntryDraft(type, '');
  return {
    ...fresh,
    date: draft.date,
    amount: draft.amount,
    note: draft.note,
    sources: type === 'other_sale' || type === 'arrival' ? {} : draft.sources,
  };
}

/** The next entry of the same type after a save: the day stays, the rest starts again. */
export function nextEntryDraft(draft: EntryDraft): EntryDraft {
  const fresh = newEntryDraft(draft.type, '');
  return {
    ...fresh,
    date: draft.date,
    sources: draft.sources,
    // Several sales in one shop are likely in one sitting.
    shop: draft.shop,
  };
}

/**
 * True once anything beyond the day is typed or chosen on the entry form. `lines` are the rows of
 * the goods arrived lines grid (PRD FR-12.41); the other types have none.
 */
export function entryIsFilled(draft: EntryDraft, lines: readonly EntryRow[]): boolean {
  switch (draft.type) {
    case 'installment':
    case 'cost':
      // A payment has no note of its own: its reason says why (PRD FR-12.35).
      return draft.amount.trim() !== '' || draft.reason.trim() !== '';
    case 'other_sale':
      return (
        draft.amount.trim() !== '' ||
        draft.shop.trim() !== '' ||
        draft.qty.trim() !== '' ||
        draft.arrivalId !== '' ||
        draft.note.trim() !== ''
      );
    case 'arrival':
      return linesAreFilled(lines) || draft.sellDate.trim() !== '' || draft.note.trim() !== '';
  }
}

/** One row for every partner of the garden with this entry's source (the contract asks for all). */
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
 * What the goods arrived form sends (PRD FR-12.40, FR-12.41): the text typed and the ids of what
 * was chosen, with every filled line. One day and one day to sell for the whole entry; an empty
 * day to sell is sent as nothing, and the backend reads it as the day of the arrival.
 */
export function buildArrivalEntry(
  draft: EntryDraft,
  lines: readonly EntryRow[],
  gardenId: number,
): ArrivalEntryInput {
  return {
    garden_id: gardenId,
    date: draft.date.trim(),
    sell_date: textOrNull(draft.sellDate),
    lines: arrivalLineInputs(lines, draft.note),
  };
}

/** A whole number from text, or null (the id of a chosen arrival). */
function idOf(text: string): number | null {
  const id = Number(text);
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}

/** What the entry form sends, whichever type is open. The amount may be a simple sum. */
export function buildEntry(
  draft: EntryDraft,
  gardenId: number,
  partners: readonly VenturePartnerRow[],
  confirmCash: boolean,
): GardenEntryInput {
  if (draft.type === 'arrival') {
    // Goods arrived are saved by their own command and are never a money entry: the editor never
    // builds one from this type.
    throw new Error('goods arrived are not a money entry');
  }
  if (draft.type === 'other_sale') {
    return {
      type: 'other_sale',
      garden_id: gardenId,
      date: draft.date.trim(),
      shop: textOrNull(draft.shop),
      arrival_id: idOf(draft.arrivalId),
      qty: draft.qty.trim(),
      amount: draft.amount.trim(),
      note: textOrNull(draft.note),
    };
  }
  return {
    type: draft.type,
    garden_id: gardenId,
    date: draft.date.trim(),
    amount: draft.amount.trim(),
    reason: draft.reason.trim(),
    parts: entryParts(partners, draft.sources),
    confirm_cash_below_zero: confirmCash,
  };
}

/** The field paths of the problems of each type of entry. */
export const ENTRY_FIELD_PATHS: Record<EntryType, readonly string[]> = {
  installment: ['date', 'amount', 'reason'],
  cost: ['date', 'amount', 'reason'],
  other_sale: ['date', 'shop', 'arrival_id', 'qty', 'amount', 'note'],
  arrival: ['date', 'sell_date', 'note', 'garden_id'],
};
