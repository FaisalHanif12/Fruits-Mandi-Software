// The draft as the backend takes it (plan 12 contract). Everything the user typed goes as the text
// typed, and the pickers go as ids (CLAUDE.md rule 1). Nothing here adds, multiplies or rounds:
// a quantity or an amount is only passed on (Code Standards M12). The backend parses it, checks
// it and works out every number (PRD 22).

import type { CarriedLotInput } from '../../bindings/CarriedLotInput';
import type { ChargeInput } from '../../bindings/ChargeInput';
import type { ChuktaInput } from '../../bindings/ChuktaInput';
import type { HeaderInput } from '../../bindings/HeaderInput';
import type { LineChargeInput } from '../../bindings/LineChargeInput';
import type { LineChargeMethod } from '../../bindings/LineChargeMethod';
import type { LotChargeMethod } from '../../bindings/LotChargeMethod';
import type { LotInput } from '../../bindings/LotInput';
import type { SaleGrade } from '../../bindings/SaleGrade';
import type { SaleInput } from '../../bindings/SaleInput';
import { isChuktaPick, isGardenAccount } from '../../shared/accounts';
import { formatPaisa, type Paisa } from '../../shared/format';
import type { EntryRow } from '../../shared/grid';
import type { BikriDraftStores, HeaderState } from './draft-store';
import { LOT, ON, OWN_GOODS_CODE, SALE, type BikriDraft } from './types';

/** Text or null: an empty field is sent as nothing, so the backend treats it as not typed. */
function textOrNull(text: string | undefined): string | null {
  const trimmed = (text ?? '').trim();
  return trimmed === '' ? null : trimmed;
}

/**
 * The header of PRD 12.2: the date alone (plan 71 H1). The bill date is the entry date, and the
 * other old fields are not sent: the backend does not take them any more, and an old بکری keeps its
 * own in the database when it is edited.
 */
export function headerInput(header: HeaderState): HeaderInput {
  return { entry_date: header.entryDate.trim() };
}

/** Has nothing been typed on this lot row but what the app filled? */
export function isEmptyLotRow(row: EntryRow): boolean {
  if (Object.keys(row.picked ?? {}).length > 0) return false;
  return Object.entries(row.typed).every(
    ([field, text]) => text === '' || field === LOT.sellerFilled || field === LOT.memoryFilledFor,
  );
}

/**
 * Is this a lot carried over from an earlier بکری (PRD FR-12.17, FR-12.48)? It is a saved lot with goods
 * left: only the lines typed on it today are the day's work, and it is sent as `carried`, not as a new lot.
 */
export function isCarriedLot(row: EntryRow): boolean {
  return (row.typed[LOT.carried] ?? '') === ON;
}

/** Was this lot loaded by hand from a trip purchase or a garden's goods arrived (PRD FR-12.50)? */
export function isGoodsLot(row: EntryRow): boolean {
  return (row.typed[LOT.purchase] ?? '') !== '' || (row.typed[LOT.arrival] ?? '') !== '';
}

/** Is this a lot of the firm's own goods: the seller is the اپنا مال account (PRD FR-12.26)? */
export function isOwnLot(row: EntryRow): boolean {
  return row.picked?.[LOT.seller]?.code === OWN_GOODS_CODE;
}

/**
 * Is this a lot of a garden: the seller is a garden's account (PRD FR-12.36)? Like an own goods
 * lot it gives no نقد and has no advance tick to deduct, so those cells do not apply to it.
 */
export function isGardenLot(row: EntryRow): boolean {
  return isGardenAccount(row.picked?.[LOT.seller]);
}

/**
 * Is this a lot that takes its goods from a garden's goods arrived entry (PRD FR-12.40)? Its item
 * and packing follow the entry, like the goods of a purchase, and it must be sold in full.
 */
export function isArrivalLot(row: EntryRow): boolean {
  return isGardenLot(row) && (row.typed[LOT.arrival] ?? '') !== '';
}

/** Is this line's buyer a چکتا buyer, who pays the whole bill on the spot (PRD FR-13.1, FR-13.5)? */
export function isChuktaBuyer(row: EntryRow): boolean {
  return isChuktaPick(row.picked?.[SALE.buyer]);
}

/**
 * The چکتا buyer of a line, as the backend takes him (PRD FR-13.5): the names and phone of the
 * buyer chosen in the picker. The line's `buyer_account_id` is the account چکتا کھاتہ, which is the
 * id of the chosen record. Null for any other buyer, so a normal line is sent as before.
 */
function chuktaInput(row: EntryRow): ChuktaInput | null {
  const buyer = row.picked?.[SALE.buyer];
  if (buyer?.chukta === undefined) return null;
  return {
    name_ur: buyer.name_ur.trim(),
    name_en: buyer.name_en.trim(),
    phone: buyer.chukta.phone.trim(),
  };
}

/**
 * A lot charge typed in its cell. Empty means nothing was typed and the rules may suggest it. The
 * method is the one picked in the "more" panel; the amount itself (دستی) when none was picked.
 */
function chargeInput(value: string | undefined, method: string | undefined): ChargeInput | null {
  const text = textOrNull(value);
  if (text === null) return null;
  const chosen: LotChargeMethod = method === 'per_unit' || method === 'per_lot' ? method : 'manual';
  return { method: chosen, value: text };
}

/** A commission or لاگا typed on a line (PRD FR-11.10). Empty: the rule decides. */
function lineChargeInput(
  value: string | undefined,
  method: string | undefined,
  fallback: LineChargeMethod,
): LineChargeInput | null {
  const text = textOrNull(value);
  if (text === null) return null;
  return { method: method === 'percent' || method === 'per_unit' ? method : fallback, value: text };
}

/** The quality of a line (PRD FR-12.42): one of the three words, or none. */
function gradeOf(row: EntryRow): SaleGrade | null {
  const typed = row.typed[SALE.grade];
  return typed === 'first' || typed === 'second' || typed === 'third' ? typed : null;
}

/** The id of a saved row kept in a hidden cell, or nothing for a row that is new. */
function savedId(text: string | undefined): number | null {
  const id = Number(text ?? '');
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}

export function saleInput(row: EntryRow): SaleInput {
  const buyer = row.picked?.[SALE.buyer];
  const saleId = savedId(row.typed[SALE.saleId]);
  return {
    key: row.rowId,
    // A saved line carries its id, so the backend can tell it is unchanged (PRD FR-12.20).
    ...(saleId === null ? {} : { sale_id: saleId }),
    buyer_account_id: buyer?.id ?? null,
    chukta: chuktaInput(row),
    qty: (row.typed[SALE.qty] ?? '').trim(),
    weight: textOrNull(row.typed[SALE.weight]),
    seller_rate: (row.typed[SALE.rate] ?? '').trim(),
    rate_differs: row.typed[SALE.differs] === ON,
    buyer_rate: textOrNull(row.typed[SALE.buyerRate]),
    // The money paid on the spot, as typed (PRD FR-12.44). Empty is sent as nothing, which the backend
    // takes as 0; for a buyer who must pay in full it sets the amount itself and ignores this.
    paid: textOrNull(row.typed[SALE.paid]),
    // Left out when there is none, so a line with no quality is sent exactly as before (FR-12.42).
    ...(gradeOf(row) === null ? {} : { grade: gradeOf(row) }),
    seller_comm: lineChargeInput(
      row.typed[SALE.sellerCommValue],
      row.typed[SALE.sellerCommMethod],
      'percent',
    ),
    buyer_comm: lineChargeInput(
      row.typed[SALE.buyerCommValue],
      row.typed[SALE.buyerCommMethod],
      'percent',
    ),
    laga: lineChargeInput(row.typed[SALE.lagaValue], row.typed[SALE.lagaMethod], 'per_unit'),
    note: textOrNull(row.typed[SALE.note]),
  };
}

/** Does the seller's tick of the summary cover this lot: not carried, not own goods, not a garden's (plan 78 C3)? */
function takesTick(row: EntryRow): boolean {
  return !isCarriedLot(row) && !isOwnLot(row) && !isGardenLot(row);
}

/** Has the lot been saved before (an edit)? It keeps the tick it was saved with unless the munshi changes it. */
function isSavedLot(row: EntryRow): boolean {
  return (row.typed[LOT.lotId] ?? '') !== '';
}

/** The lots of the بکری that the tick of one seller covers, in order. */
export function tickLotsOf(rows: readonly EntryRow[], sellerId: number): EntryRow[] {
  return rows.filter((row) => takesTick(row) && row.picked?.[LOT.seller]?.id === sellerId);
}

/**
 * What a lot sends as its tick "پیشگی اس بکری سے کاٹیں" (PRD FR-12.23, plan 78 C3). The tick is the seller's, made
 * in his summary and off when a بکری opens, and the backend still takes one for each lot. A lot with no tick to
 * take (own goods, a garden) sends none. A saved lot sends the tick it carries. A new lot sends the seller's tick
 * as his summary shows it: on when any lot of his that the tick covers has it on, saved or new, so a lot added in
 * an edit is covered too (plan 80 R2-4, Q-R2-3: one tick covers every lot of that seller in the بکری).
 */
export function adjustAdvanceOf(row: EntryRow, rows: readonly EntryRow[]): boolean {
  if (!takesTick(row)) return false;
  if (isSavedLot(row)) return row.typed[LOT.adjustAdvance] === ON;
  const sellerId = row.picked?.[LOT.seller]?.id;
  if (sellerId === undefined) return row.typed[LOT.adjustAdvance] === ON;
  return tickLotsOf(rows, sellerId).some((lot) => lot.typed[LOT.adjustAdvance] === ON);
}

export function lotInput(
  row: EntryRow,
  sales: readonly EntryRow[],
  /** What the lot sends as its tick: the seller's tick from his summary (plan 78 C3, `advance-tick.ts`). */
  adjustAdvance = row.typed[LOT.adjustAdvance] === ON,
): LotInput {
  const typed = row.typed;
  const lotId = savedId(typed[LOT.lotId]);
  // A lot of own goods takes its goods from one purchase (PRD FR-12.26). There is no cash for the
  // firm to give itself and no advance to deduct, so a new lot sends no cash and no advance tick
  // whatever was typed on the row before the seller became اپنا مال. A saved lot sends back the
  // cash it came with ("0"), so an edit that changes nothing sends the draft as the backend gave
  // it. A freight typed here is sent as typed: the purchase's freight is already in its cost, so
  // the backend warns (ERR_OWN_LOT_FREIGHT_IN_COST) and the cell shows it.
  const own = isOwnLot(row);
  // A garden's lot is the same: no نقد and no advance tick (PRD FR-12.36). The backend refuses a
  // typed نقد on it, so a new lot sends none whatever was typed before the seller became a garden.
  const noCash = own || isGardenLot(row);
  const advanceText = noCash ? null : textOrNull(typed[LOT.advance]);
  const purchaseId = own ? savedId(typed[LOT.purchase]) : null;
  // A garden's lot may take its goods from a goods arrived entry (PRD FR-12.40). A saved lot says
  // it has none, as the backend's own draft of it does (an edit sends it back as it came); a new
  // lot leaves the field out.
  const arrivalId = isGardenLot(row) ? savedId(typed[LOT.arrival]) : null;
  return {
    key: row.rowId,
    ...(lotId === null ? {} : { lot_id: lotId }),
    // A saved lot says it has no purchase, as the backend's own draft of it does (an edit sends it
    // back as it came); a new lot leaves the field out.
    ...(purchaseId !== null
      ? { purchase_id: purchaseId }
      : lotId !== null
        ? { purchase_id: null }
        : {}),
    ...(arrivalId !== null
      ? { arrival_id: arrivalId }
      : lotId !== null
        ? { arrival_id: null }
        : {}),
    seller_account_id: row.picked?.[LOT.seller]?.id ?? null,
    item_id: row.picked?.[LOT.item]?.id ?? null,
    packing_id: row.picked?.[LOT.packing]?.id ?? null,
    marka_id: row.picked?.[LOT.marka]?.id ?? null,
    store_id: row.picked?.[LOT.store]?.id ?? null,
    qty_received: (typed[LOT.qty] ?? '').trim(),
    weight_received: textOrNull(typed[LOT.weight]),
    freight: textOrNull(typed[LOT.freight]),
    freight_paid_by: typed[LOT.freightPaidBy] === 'seller' ? 'seller' : 'firm',
    labour: chargeInput(typed[LOT.labour], typed[LOT.labourMethod]),
    munshiana: chargeInput(typed[LOT.munshiana], typed[LOT.munshianaMethod]),
    phone: chargeInput(typed[LOT.phone], typed[LOT.phoneMethod]),
    cash_paid: noCash && lotId === null ? null : textOrNull(typed[LOT.cash]),
    store_rent: textOrNull(typed[LOT.storeRent]),
    // پیشگی (plan 95 item 11): sent as typed, only for a lot that can take cash; the backend works it out and checks it.
    ...(advanceText === null ? {} : { advance: advanceText }),
    // The tick starts OFF and is made in the seller's summary (PRD FR-12.23, plan 78 C3). Trip goods have no
    // advance to deduct, and the backend stores and sends back false for them.
    adjust_advance: noCash ? false : adjustAdvance,
    sales: sales.map(saleInput),
  };
}

/**
 * The draft as text, for telling whether two drafts are the same to the backend. A blank row (no buyer
 * and no rate) is left out: the preview and the save ignore it (PRD FR-7.52), and the app fills its
 * quantity by itself a moment after the row appears, so that fill is not a change the user made.
 */
export function draftKey(draft: BikriDraft): string {
  const typedLine = (sale: { buyer_account_id: number | null; seller_rate: string }) =>
    sale.buyer_account_id !== null || sale.seller_rate !== '';
  return JSON.stringify({
    ...draft,
    lots: draft.lots.map((lot) => ({ ...lot, sales: lot.sales.filter(typedLine) })),
    ...(draft.carried === undefined
      ? {}
      : { carried: draft.carried.map((lot) => ({ ...lot, sales: lot.sales.filter(typedLine) })) }),
  });
}

/** Has a line been typed on a carried lot: a buyer or a rate? A carried lot alone is not typed work. */
function carriedFilled(lot: CarriedLotInput): boolean {
  return lot.sales.some((sale) => sale.buyer_account_id !== null || sale.seller_rate !== '');
}

/** Has anything been typed on this lot or its lines? */
function lotFilled(lot: BikriDraft['lots'][number]): boolean {
  return (
    lot.seller_account_id !== null ||
    lot.item_id !== null ||
    lot.packing_id !== null ||
    lot.qty_received !== '' ||
    lot.freight !== null ||
    lot.cash_paid !== null ||
    lot.labour !== null ||
    lot.munshiana !== null ||
    lot.sales.some((sale) => sale.buyer_account_id !== null || sale.seller_rate !== '')
  );
}

/** Has anything been typed yet? An untouched screen asks the backend nothing. */
export function isFilled(draft: BikriDraft): boolean {
  // The date alone is not typed work: the header has nothing else (plan 71 H1).
  return draft.lots.some(lotFilled) || (draft.carried ?? []).some(carriedFilled);
}

/**
 * Has the user put anything on the screen? A lot he loaded by hand (trip goods, garden goods, a lot carried over)
 * or typed is his work, so a form with one asks "save changes?" when he leaves it (PRD FR-7.9). An empty row that
 * only waits to be typed in is not.
 */
export function isFilledByUser(draft: BikriDraft): boolean {
  return draft.lots.some(lotFilled) || (draft.carried ?? []).some(carriedFilled);
}

/** The draft of the screen as the backend takes it. Reads the stores as they are now. */
export function buildDraft(stores: BikriDraftStores): BikriDraft {
  const rows = stores.lots.getState().rows;
  const sales = (row: EntryRow) => stores.salesFor(row.rowId).getState().rows;
  // A lot carried over is sent as its id and the lines typed on it; it is never a new lot (plan 76 point 1).
  // Its key is the row's, so the answer's lot preview (and the part of a mixed answer) finds its row again.
  const carried = rows.filter(isCarriedLot).map((row): CarriedLotInput => ({
    key: row.rowId,
    lot_id: savedId(row.typed[LOT.lotId]) ?? 0,
    sales: sales(row).map(saleInput),
  }));
  // With carried lots or goods loaded by hand on the screen, the one empty lot row that waits to be typed in is not
  // a new lot: a screen with only carried lots makes no new بکری (the backend takes an empty `new_lots` as that),
  // and a loaded lot is the lot of the sale, not one lot and a stray empty one (plan 78 C1).
  const hasWork = carried.length > 0 || rows.some(isGoodsLot);
  const waiting = (row: EntryRow) =>
    hasWork &&
    isEmptyLotRow(row) &&
    !sales(row).some(
      (line) => line.picked?.[SALE.buyer] !== undefined || (line.typed[SALE.rate] ?? '') !== '',
    );
  return {
    header: headerInput(stores.header.getState()),
    lots: rows
      .filter((row) => !isCarriedLot(row) && !waiting(row))
      .map((row) => lotInput(row, sales(row), adjustAdvanceOf(row, rows))),
    ...(carried.length === 0 ? {} : { carried }),
  };
}

/**
 * An amount from the backend as the plain whole-rupee text a user would type ("3000"), to put in
 * a cell the app fills, such as the freight of a lot after the split (PRD C17). The cell shows it
 * grouped; what is sent is these digits. It only writes text; it works out nothing.
 */
export function plainRupees(amount: Paisa): string {
  return formatPaisa(amount, { grouping: 'international' }).replaceAll(',', '');
}
