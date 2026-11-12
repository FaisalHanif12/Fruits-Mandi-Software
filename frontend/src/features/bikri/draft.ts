// The draft as the backend takes it (plan 12 contract). Everything the user typed goes as the text
// typed, and the pickers go as ids (CLAUDE.md rule 1). Nothing here adds, multiplies or rounds:
// a quantity or an amount is only passed on (Code Standards M12). The backend parses it, checks
// it and works out every number (PRD 22).

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

export function lotInput(row: EntryRow, sales: readonly EntryRow[]): LotInput {
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
    // The tick starts on (PRD FR-12.23); only "0" is an untick. Trip goods have no advance to
    // deduct, and the backend stores and sends back false for them.
    adjust_advance: noCash ? false : typed[LOT.adjustAdvance] !== '0',
    sales: sales.map(saleInput),
  };
}

/**
 * The draft as text, for telling whether two drafts are the same to the backend. A blank row (no buyer
 * and no rate) is left out: the preview and the save ignore it (PRD FR-7.52), and the app fills its
 * quantity by itself a moment after the row appears, so that fill is not a change the user made.
 */
export function draftKey(draft: BikriDraft): string {
  return JSON.stringify({
    ...draft,
    lots: draft.lots.map((lot) => ({
      ...lot,
      sales: lot.sales.filter((sale) => sale.buyer_account_id !== null || sale.seller_rate !== ''),
    })),
  });
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
  return draft.lots.some(lotFilled);
}

/**
 * Has the user put anything on the screen? The lots that came in by themselves from the ready stock
 * (plan 33, PRD FR-12.39, FR-12.40) and that he has not worked on (no buyer and no rate on any of
 * their lines) are not his work yet: a form with only those asks no "save changes?", keeps no draft
 * and asks no preview. As soon as he types a line on any lot, the screen counts as filled and the
 * whole draft goes to the preview, those lots too, so a ready lot he leaves alone is still checked
 * and must be sold or removed (FR-12.26).
 */
export function isFilledByUser(stores: BikriDraftStores, draft: BikriDraft): boolean {
  const { readyLots } = stores.view.getState();
  const cameByItself = (lot: BikriDraft['lots'][number]) =>
    lot.key in readyLots &&
    lot.sales.every((sale) => sale.buyer_account_id === null && sale.seller_rate === '');
  return draft.lots.some((lot) => !cameByItself(lot) && lotFilled(lot));
}

/** The draft of the screen as the backend takes it. Reads the stores as they are now. */
export function buildDraft(stores: BikriDraftStores): BikriDraft {
  return {
    header: headerInput(stores.header.getState()),
    lots: stores.lots
      .getState()
      .rows.map((row) => lotInput(row, stores.salesFor(row.rowId).getState().rows)),
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
