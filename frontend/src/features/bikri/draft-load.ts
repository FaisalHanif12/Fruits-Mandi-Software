// A draft that came from the backend, put into the stores of the screen (plan 16): the draft of a
// posting opened for edit (`bikri_open_for_edit`) and the draft that an undo gives back
// (`bikri_undo`, PRD FR-7.47). It is the opposite of `buildDraft` (draft.ts): every value goes into
// its cell as the text the backend sent, like typed text, and the screen works out nothing.
//
// The backend sends ids only, so the records the pickers show (the seller, the buyer, the item ...)
// are looked up here, once, by id: the ones the saved بکری already carries come from there, the
// rest are asked for. A record that cannot be found (an item made inactive since) is shown by its
// id, never dropped, so saving it again cannot change it by accident (PRD FR-12.20).
//
// Keys: a lot row and a sale row take the `key` of the draft as their row id, so the preview's
// answer lands on the same row, and a saved row keeps the id of the saved lot or line in a hidden
// cell (`LOT.lotId`, `SALE.saleId`).

import type { BikriDraft } from '../../bindings/BikriDraft';
import type { LotInput } from '../../bindings/LotInput';
import type { RecordRef } from '../../bindings/RecordRef';
import type { SaleInput } from '../../bindings/SaleInput';
import type { SavedLot } from '../../bindings/SavedLot';
import { accountsGet } from '../../shared/accounts/api';
import { isoToDisplayDate } from '../../shared/format';
import type { EntryRow, PickedRecord } from '../../shared/grid';
import { findMaster, type SearchScope } from '../../shared/search';
import type { BikriDraftStores, HeaderState } from './draft-store';
import { LOT, ON, SALE } from './types';

/** The master lists a draft points at by id. */
type MasterScope = Extract<SearchScope, 'item' | 'packing' | 'marka' | 'adda' | 'store'>;

/** The records of a draft, found by id. */
export interface DraftRecords {
  account: (id: number | null) => PickedRecord | null;
  master: (scope: MasterScope, id: number | null) => PickedRecord | null;
}

function pickedOfRef(ref: RecordRef): PickedRecord {
  return { id: ref.id, code: ref.code, name_ur: ref.name_ur, name_en: ref.name_en };
}

/** A record that cannot be found: shown by its id, so it is still there and still saved as it was. */
function stub(id: number): PickedRecord {
  const text = String(id);
  return { id, code: text, name_ur: text, name_en: text };
}

/** The accounts and items a saved بکری already carries by name, so they need no asking. */
export function recordsOfSaved(
  lots: readonly SavedLot[],
  zamindar: RecordRef | null,
): { accounts: RecordRef[]; masters: Map<string, RecordRef> } {
  const accounts: RecordRef[] = zamindar === null ? [] : [zamindar];
  const masters = new Map<string, RecordRef>();
  for (const lot of lots) {
    accounts.push(lot.seller);
    masters.set(`item:${String(lot.item.id)}`, lot.item);
    masters.set(`packing:${String(lot.packing.id)}`, lot.packing);
    if (lot.marka_id !== null && lot.marka_text !== null) {
      masters.set(`marka:${String(lot.marka_id)}`, {
        id: lot.marka_id,
        code: '',
        name_ur: lot.marka_text,
        name_en: lot.marka_text,
      });
    }
    for (const sale of lot.sales) accounts.push(sale.buyer);
  }
  return { accounts, masters };
}

interface Wanted {
  accounts: Set<number>;
  masters: Map<string, { scope: MasterScope; id: number }>;
}

function wantedBy(draft: BikriDraft): Wanted {
  const accounts = new Set<number>();
  const masters = new Map<string, { scope: MasterScope; id: number }>();
  const account = (id: number | null) => {
    if (id !== null) accounts.add(id);
  };
  const master = (scope: MasterScope, id: number | null) => {
    if (id !== null) masters.set(`${scope}:${String(id)}`, { scope, id });
  };
  account(draft.header.zamindar_account_id);
  master('adda', draft.header.adda_id);
  master('store', draft.header.store_id);
  for (const lot of draft.lots) {
    account(lot.seller_account_id);
    master('item', lot.item_id);
    master('packing', lot.packing_id);
    master('marka', lot.marka_id);
    master('store', lot.store_id);
    for (const sale of lot.sales) account(sale.buyer_account_id);
  }
  return { accounts, masters };
}

/**
 * Finds every record the draft points at. `known` are the ones the saved بکری already carries;
 * the others are asked for (an account by id, a master from its list).
 */
export async function resolveDraftRecords(
  draft: BikriDraft,
  known: { accounts: readonly RecordRef[]; masters: ReadonlyMap<string, RecordRef> },
): Promise<DraftRecords> {
  const wanted = wantedBy(draft);
  const accounts = new Map<number, PickedRecord>();
  for (const ref of known.accounts) accounts.set(ref.id, pickedOfRef(ref));
  const masters = new Map<string, PickedRecord>();
  for (const [key, ref] of known.masters) masters.set(key, pickedOfRef(ref));

  await Promise.all([
    ...[...wanted.accounts]
      .filter((id) => !accounts.has(id))
      .map(async (id) => {
        try {
          const row = await accountsGet(id);
          accounts.set(id, {
            id: row.id,
            code: row.code,
            name_ur: row.name_ur,
            name_en: row.name_en,
          });
        } catch {
          accounts.set(id, stub(id));
        }
      }),
    ...[...wanted.masters]
      .filter(([key]) => !masters.has(key))
      .map(async ([key, { scope, id }]) => {
        try {
          const hit = await findMaster(scope, id);
          masters.set(
            key,
            hit === null
              ? stub(id)
              : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
          );
        } catch {
          masters.set(key, stub(id));
        }
      }),
  ]);

  return {
    account: (id) => (id === null ? null : (accounts.get(id) ?? stub(id))),
    master: (scope, id) =>
      id === null ? null : (masters.get(`${scope}:${String(id)}`) ?? stub(id)),
  };
}

/** Text or nothing, as the cell keeps it. */
function text(value: string | null | undefined): string {
  return value ?? '';
}

/** A date from the backend as the user types dates: DD-MM-YYYY. */
function dateText(value: string): string {
  return isoToDisplayDate(value.trim());
}

function headerState(draft: BikriDraft, records: DraftRecords): HeaderState {
  const header = draft.header;
  return {
    entryDate: dateText(header.entry_date),
    billDate: dateText(header.bill_date),
    book: text(header.book),
    truckNo: text(header.truck_no),
    zamindar: records.account(header.zamindar_account_id),
    adda: records.master('adda', header.adda_id),
    driverName: text(header.driver_name),
    driverPhone: text(header.driver_phone),
    totalFreight: text(header.total_freight),
    store: records.master('store', header.store_id),
    note: text(header.note),
  };
}

function put(into: Record<string, string>, field: string, value: string | null | undefined) {
  const shown = text(value);
  if (shown !== '') into[field] = shown;
}

function saleRow(sale: SaleInput, records: DraftRecords): EntryRow {
  const typed: Record<string, string> = {};
  put(typed, SALE.qty, sale.qty);
  put(typed, SALE.weight, sale.weight);
  put(typed, SALE.rate, sale.seller_rate);
  if (sale.rate_differs) typed[SALE.differs] = ON;
  put(typed, SALE.buyerRate, sale.buyer_rate);
  put(typed, SALE.payment, sale.payment_type);
  put(typed, SALE.note, sale.note);
  const charges: [string, string, SaleInput['seller_comm']][] = [
    [SALE.sellerCommValue, SALE.sellerCommMethod, sale.seller_comm],
    [SALE.buyerCommValue, SALE.buyerCommMethod, sale.buyer_comm],
    [SALE.lagaValue, SALE.lagaMethod, sale.laga],
  ];
  for (const [valueField, methodField, charge] of charges) {
    if (charge === null) continue;
    typed[valueField] = charge.value;
    typed[methodField] = charge.method;
  }
  if (sale.pukhta !== null) {
    put(typed, SALE.pukhtaUr, sale.pukhta.name_ur);
    put(typed, SALE.pukhtaEn, sale.pukhta.name_en);
    put(typed, SALE.pukhtaPhone, sale.pukhta.phone);
  }
  if (sale.sale_id !== undefined && sale.sale_id !== null) {
    typed[SALE.saleId] = String(sale.sale_id);
  }
  // The quantity is the loaded one: the app must not fill it again from what the lot has left.
  typed[SALE.qtyFilled] = ON;
  const buyer = records.account(sale.buyer_account_id);
  return buyer === null
    ? { rowId: sale.key, typed }
    : { rowId: sale.key, typed, picked: { [SALE.buyer]: buyer } };
}

function lotRow(lot: LotInput, header: HeaderState, records: DraftRecords): EntryRow {
  const typed: Record<string, string> = {};
  put(typed, LOT.qty, lot.qty_received);
  put(typed, LOT.weight, lot.weight_received);
  put(typed, LOT.freight, lot.freight);
  typed[LOT.freightPaidBy] = lot.freight_paid_by;
  const charges: [string, string, LotInput['labour']][] = [
    [LOT.labour, LOT.labourMethod, lot.labour],
    [LOT.munshiana, LOT.munshianaMethod, lot.munshiana],
    [LOT.phone, LOT.phoneMethod, lot.phone],
  ];
  for (const [valueField, methodField, charge] of charges) {
    if (charge === null) continue;
    typed[valueField] = charge.value;
    // A typed amount (دستی) has no method of its own in the "more" panel.
    if (charge.method !== 'manual') typed[methodField] = charge.method;
  }
  put(typed, LOT.cash, lot.cash_paid);
  put(typed, LOT.storeRent, lot.store_rent);
  typed[LOT.adjustAdvance] = lot.adjust_advance ? ON : '0';
  if (lot.pukhta !== null) {
    put(typed, LOT.pukhtaUr, lot.pukhta.name_ur);
    put(typed, LOT.pukhtaEn, lot.pukhta.name_en);
    put(typed, LOT.pukhtaPhone, lot.pukhta.phone);
  }
  if (lot.lot_id !== undefined && lot.lot_id !== null) typed[LOT.lotId] = String(lot.lot_id);

  const picked: Record<string, PickedRecord> = {};
  const seller = records.account(lot.seller_account_id);
  if (seller !== null) {
    picked[LOT.seller] = seller;
    // The loaded seller and what he sent are already in the cells: nothing is filled again.
    typed[LOT.sellerFilled] = ON;
    typed[LOT.memoryFilledFor] = String(seller.id);
  }
  const item = records.master('item', lot.item_id);
  if (item !== null) picked[LOT.item] = item;
  const packing = records.master('packing', lot.packing_id);
  if (packing !== null) picked[LOT.packing] = packing;
  const marka = records.master('marka', lot.marka_id);
  if (marka !== null) picked[LOT.marka] = marka;
  // The lot store is the header store unless it was chosen separately.
  if (lot.store_id !== null && lot.store_id !== header.store?.id) {
    const store = records.master('store', lot.store_id);
    if (store !== null) picked[LOT.store] = store;
  }
  return { rowId: lot.key, typed, picked };
}

/** The rows a draft makes: the lots, and the lines of each lot by the row id of the lot. */
export function rowsOfDraft(
  draft: BikriDraft,
  records: DraftRecords,
): { header: HeaderState; lots: EntryRow[]; sales: Record<string, EntryRow[]> } {
  const header = headerState(draft, records);
  const sales: Record<string, EntryRow[]> = {};
  const lots = draft.lots.map((lot) => {
    sales[lot.key] = lot.sales.map((sale) => saleRow(sale, records));
    return lotRow(lot, header, records);
  });
  return { header, lots, sales };
}

/** Puts a draft from the backend into the stores, so the screen shows it like a typed one. */
export function applyBackendDraft(
  stores: BikriDraftStores,
  draft: BikriDraft,
  records: DraftRecords,
): void {
  const { header, lots, sales } = rowsOfDraft(draft, records);
  stores.header.setState(header);
  // A grid always keeps one row: a lot with no lines gets a blank one.
  stores.lots.setState({ rows: lots.length === 0 ? stores.lots.getState().rows : lots });
  for (const lot of lots) {
    const lines = sales[lot.rowId] ?? [];
    if (lines.length > 0) stores.salesFor(lot.rowId).setState({ rows: lines });
  }
  stores.view.getState().select(lots[0]?.rowId ?? null);
}
