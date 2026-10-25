// A stand-in for the Tauri bridge, for the development page /dev/bikri only (plan 12). In a plain
// browser tab there is no backend, so no screen can be looked at. This answers the commands of the
// بکری screen with the same canned numbers the component tests use (Event 1, PRD 4.4), so the
// screen can be opened in a browser and photographed in both languages. It answers nothing else:
// any other command is refused with ERR_NOT_IMPLEMENTED, like a stub. Never part of a build: only
// the development route imports it.

import { EVENT1, IDS, SELLER_MEMORY, TRUCK_MEMORY, previewFor } from './bikri-data';
import {
  cancelPreview,
  editable,
  laterSaleAnswer,
  stockList,
  stockRow,
  undoResult,
} from './bikri-plan16-data';
import { NASIR, SHAHID, listRow, registerRow, savedEvent1 } from './bikri-saved';
import { paisaForTest } from './paisa';
import { PARTIES as BOOK_PARTIES, collectionList, createBooks, voucherRow } from './voucher-books';
import { printDataFor, voucherPrintData } from './print-data';
import {
  createTripBooks,
  e34Trip,
  tripRow,
  TRIP_REF,
  purchaseEntry,
  costEntry,
} from './trips-data';
import { printingSettings } from './print-settings';
import { gardensHandlers, isGardensPage } from './dev-bridge-gardens';
import { indexHandlers, isIndexPage } from './dev-bridge-index';
import { isRokarPage, rokarHandlers } from './dev-bridge-rokar';
import * as ownReal from './bikri-own-real';
import * as tripsReal from './trips-real';
import { EMPTY_PAIRS, bikriListOf, pukhtaRegisterOf } from './paged';
import type { OtherSaleInput } from '../../src/bindings/OtherSaleInput';
import type { PurchaseInput } from '../../src/bindings/PurchaseInput';
import type { PurchasesListInput } from '../../src/bindings/PurchasesListInput';
import type { TripCostInput } from '../../src/bindings/TripCostInput';
import type { TripGetInput } from '../../src/bindings/TripGetInput';
import type { TripInput } from '../../src/bindings/TripInput';
import type { TripsListInput } from '../../src/bindings/TripsListInput';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { AdvanceInput } from '../../src/bindings/AdvanceInput';
import type { SaleInput } from '../../src/bindings/SaleInput';
import type { PaymentInput } from '../../src/bindings/PaymentInput';
import type { ReceiptInput } from '../../src/bindings/ReceiptInput';
import type { VoucherEntry } from '../../src/features/vouchers/types';
import { createBikriDraft, type BikriDraftStores } from '../../src/features/bikri/draft-store';
import { serialiseDraft } from '../../src/features/bikri/draft-snapshot';

interface Party {
  id: number;
  code: string;
  type: 'seller' | 'buyer';
  ur: string;
  en: string;
  /** Balance in paisa and its side, as the backend sends it. */
  paisa: number;
  side: 'dr' | 'cr' | 'zero';
}

const OWN_ID = 17;
const OWN_CODE = '030119998';

const BIKRI_PARTIES: readonly Party[] = [
  {
    id: IDS.rahim,
    code: '030110001',
    type: 'seller',
    ur: 'رحیم',
    en: 'Rahim',
    paisa: 431600,
    side: 'cr',
  },
  {
    id: IDS.karim,
    code: '030110002',
    type: 'seller',
    ur: 'کریم',
    en: 'Karim',
    paisa: 0,
    side: 'zero',
  },
  {
    id: IDS.pukhtaSeller,
    code: '030119999',
    type: 'seller',
    ur: 'پختہ کھاتہ (بیوپاری)',
    en: 'Pukhta Khata (Seller)',
    paisa: 0,
    side: 'zero',
  },
  // اپنا مال: offered only in the seller field of a lot (scope lot_seller, PRD FR-12.26).
  {
    id: OWN_ID,
    code: OWN_CODE,
    type: 'seller',
    ur: 'اپنا مال',
    en: 'Own Goods (Trips)',
    paisa: 0,
    side: 'zero',
  },
  {
    id: IDS.akram,
    code: '03040001',
    type: 'buyer',
    ur: 'اکرم ٹول ٹیکس',
    en: 'Akram Toll Tax',
    paisa: 288000,
    side: 'dr',
  },
  {
    id: IDS.arshad,
    code: '03040002',
    type: 'buyer',
    ur: 'ارشد ملتانی',
    en: 'Arshad Multani',
    paisa: 1868400,
    side: 'dr',
  },
  {
    id: IDS.pukhtaBuyer,
    code: '03049999',
    type: 'buyer',
    ur: 'پختہ کھاتہ (خریدار)',
    en: 'Pukhta Khata (Buyer)',
    paisa: 0,
    side: 'zero',
  },
];

// The parties of the voucher pages are the ones of the worked day's books (voucher-books.ts), with
// their own ids, so the pickers and the helper panels agree.
const VOUCHER_BALANCE: Record<number, { paisa: number; side: Party['side'] }> = {
  21: { paisa: 1288000, side: 'dr' },
  22: { paisa: 1868400, side: 'dr' },
  31: { paisa: 431600, side: 'cr' },
  32: { paisa: 1016000, side: 'cr' },
};
const VOUCHER_PARTIES: readonly Party[] = BOOK_PARTIES.filter(
  (party) => party.type !== 'other',
).map((party) => ({
  id: party.id,
  code: party.code,
  type: party.type === 'seller' ? 'seller' : 'buyer',
  ur: party.name_ur,
  en: party.name_en,
  paisa: VOUCHER_BALANCE[party.id]?.paisa ?? 0,
  side: VOUCHER_BALANCE[party.id]?.side ?? 'zero',
}));

/** The parties the pickers offer: the worked day's books on the voucher pages, else the sale book's. */
function parties(): readonly Party[] {
  return window.location.hash.startsWith('#/dev/vouchers') ? VOUCHER_PARTIES : BIKRI_PARTIES;
}

interface Master {
  id: number;
  scope: string;
  code: string;
  ur: string;
  en: string;
}

const MASTERS: readonly Master[] = [
  { id: IDS.banana, scope: 'item', code: '0001', ur: 'کیلا', en: 'Banana' },
  { id: IDS.apple, scope: 'item', code: '0002', ur: 'سیب', en: 'Apple' },
  { id: IDS.box, scope: 'packing', code: '0001', ur: 'پیٹی', en: 'Box' },
  { id: IDS.swat, scope: 'adda', code: '0001', ur: 'سوات اڈا', en: 'Swat Adda' },
];

type Input = Record<string, unknown>;
type Handler = (input: Input) => unknown;

const asText = (value: unknown): string => (typeof value === 'string' ? value : '');
const asNumber = (value: unknown): number => (typeof value === 'number' ? value : 0);

function accountHit(party: Party) {
  return {
    id: party.id,
    code: party.code,
    name_ur: party.ur,
    name_en: party.en,
    extra_ur: '',
    extra_en: '',
    account_type: party.type,
    control_id: party.type === 'seller' ? 11 : 4,
    is_active: true,
  };
}

function partyMatches(party: Party, scope: string, text: string): boolean {
  // The seller of a lot may be any seller and اپنا مال; every other seller field leaves it out.
  if (scope === 'lot_seller') {
    if (party.type !== 'seller') return false;
  } else if (scope === 'seller' && party.code === OWN_CODE) {
    return false;
  } else if (scope !== 'any' && party.type !== scope) {
    return false;
  }
  const wanted = text.trim().toLowerCase();
  if (wanted === '') return true;
  if (/^\d+$/.test(wanted)) {
    return party.code === wanted || Number(party.code.slice(-4)) === Number(wanted);
  }
  return [party.code, party.ur, party.en].some((part) => part.toLowerCase().includes(wanted));
}

/** The flags in the hash of the development page, for example `#/dev/bikri?cash`. */
export function devFlag(name: string): boolean {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '').has(name);
}

/** The value of a flag in the hash, for example `type` in `#/dev/vouchers?type=payment`. */
export function devValue(name: string): string | null {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '').get(name);
}

const CASH_BELOW_ZERO = {
  cash_before_paisa: paisaForTest(200_000),
  change_paisa: paisaForTest(-700_000),
  cash_after_paisa: paisaForTest(-500_000),
  below_zero: true,
  lowers_cash: true,
  needs_confirm: true,
};

// The books of the worked day for the voucher pages. A few saved vouchers are in the list so it is
// not empty (`?empty` starts with none).
const books = createBooks();
if (!new URLSearchParams(window.location.hash.split('?')[1] ?? '').has('empty')) {
  books.store.rows.push(
    voucherRow({ id: 501, number: 1 }),
    voucherRow({
      id: 503,
      number: 3,
      status: 'cancelled',
      editable: false,
      cancel_reason: 'wrong buyer',
    }),
    voucherRow({ id: 504, number: 4, editable: false }),
    voucherRow({
      id: 502,
      number: 2,
      voucher_type: 'payment',
      account: books.preview({
        type: 'payment',
        date: '10-10-2026',
        account_id: 31,
        person_key: null,
        amount: '1',
        reason: '',
        narration: '',
        confirm_cash_below_zero: false,
      }).account,
    }),
  );
}

// The trips of the development page /dev/trips: the worked trip of E2E-34 (a purchase of 20 for
// 10,400, 15 sold) with one cost; `&closed` shows it closed, `&empty` a list with no trip.
const tripBooks = createTripBooks({
  trips: devFlag('empty')
    ? []
    : [
        e34Trip({ id: TRIP_REF.id, status: devFlag('closed') ? 'closed' : 'open' }),
        tripRow({ id: 2, number: 2 }),
      ],
  entries: [costEntry(), purchaseEntry()],
  cashBefore: devFlag('cash') ? 5_000 : 50_000,
});
const tripsDefaults = tripBooks.defaults();

/** The share text of one row sent to partners_check_shares, or an empty text. */
function shareTextOf(row: unknown): string {
  if (typeof row !== 'object' || row === null) return '';
  const share: unknown = Reflect.get(row, 'share');
  return typeof share === 'string' ? share : '';
}

/** The running total of the shares (partners_check_shares), read from the typed texts. */
function sharesCheck(input: Input) {
  const rows: unknown[] = Array.isArray(input['shares']) ? input['shares'] : [];
  let total = 0;
  for (const row of rows) {
    total += Math.round(Number(shareTextOf(row).replace(',', '.')) * 100) || 0;
  }
  return { total_bps: total, is_100: total === 10_000, errors: [] };
}

const handlers: Record<string, Handler> = {
  partners_check_shares: (input) => sharesCheck(input),
  trips_defaults: () => tripsDefaults,
  trips_list: (input) => tripBooks.list(input as TripsListInput),
  trips_get: (input) => tripBooks.get(input as TripGetInput),
  trips_create: (input) => tripBooks.create(input as TripInput),
  trips_cost_preview: (input) => tripBooks.costPreview(input as TripCostInput),
  trips_cost_save: (input) => tripBooks.costSave(input as TripCostInput),
  trips_cost_cancel: () => tripBooks.costCancel(),
  trips_other_sale_preview: (input) => tripBooks.salePreview(input as OtherSaleInput),
  trips_other_sale_save: (input) => tripBooks.saleSave(input as OtherSaleInput),
  trips_other_sale_cancel: () => tripBooks.saleCancel(),
  purchases_preview: (input) => tripBooks.purchasePreview(input as PurchaseInput),
  purchases_save: (input) => tripBooks.purchaseSave(input as PurchaseInput),
  // `?own` answers with the backend's real purchase (id 1) instead of the canned one (id 7).
  purchases_get: (input) =>
    devFlag('own')
      ? tripsReal.PURCHASE_GET_AFTER_SALES
      : tripBooks.purchaseGet(asNumber(input['id'])),
  purchases_list: (input) =>
    devFlag('own')
      ? { ...tripsReal.PURCHASES_LIST_ALL, rows: [tripsReal.PURCHASE_GET_AFTER_SALES] }
      : tripBooks.purchaseList(input as PurchasesListInput),
  purchases_cancel: () => tripBooks.purchaseCancel(),
  purchases_write_off: () => tripBooks.writeOff(),
  purchases_write_off_undo: () => tripBooks.writeOffUndo(),
  day_status: () => ({ today: '2026-10-10', closed_up_to: null, today_open: true }),
  search_accounts: (input) =>
    parties()
      .filter((party) => partyMatches(party, asText(input['scope']), asText(input['text'])))
      .map(accountHit),
  search_accounts_recent: (input) =>
    parties()
      .filter((party) => partyMatches(party, asText(input['scope']), ''))
      .map(accountHit),
  search_accounts_touch: () => null,
  accounts_balances: (input) => {
    const ids = Array.isArray(input['ids']) ? input['ids'] : [];
    return parties()
      .filter((party) => ids.includes(party.id))
      .map((party) => ({
        id: party.id,
        balance: { amount_paisa: party.paisa, side: party.side },
      }));
  },
  accounts_resolve_code: (input) => {
    const found = parties().filter(
      (party) =>
        partyMatches(party, asText(input['scope']), asText(input['text'])) &&
        /^\d+$/.test(asText(input['text']).trim()),
    );
    const only = found[0];
    return only !== undefined && found.length === 1
      ? { status: 'found', account: accountHit(only), candidates: [], suggested_code: null }
      : { status: 'unknown', account: null, candidates: [], suggested_code: null };
  },
  accounts_get: (input) => {
    const party = parties().find((candidate) => candidate.id === asNumber(input['id']));
    return party === undefined
      ? null
      : { id: party.id, code: party.code, name_ur: party.ur, name_en: party.en };
  },
  assist_party_card: (input) => {
    const party = parties().find((candidate) => candidate.id === asNumber(input['account_id']));
    if (party === undefined) return null;
    const buyer = party.type === 'buyer';
    return {
      id: party.id,
      code: party.code,
      account_type: party.type,
      name_ur: party.ur,
      name_en: party.en,
      father_or_firm_ur: '',
      father_or_firm_en: '',
      city_ur: 'کوٹ رادھا کشن',
      city_en: 'Kot Radha Kishan',
      phone: '03001234567',
      is_active: true,
      balance: { amount_paisa: party.paisa, side: party.side },
      last_entry_date: '2026-10-09',
      credit_limit: buyer ? 5000000 : null,
      credit_used: buyer ? party.paisa : null,
      advance_open: buyer ? null : 1500000,
      warnings: [],
    };
  },
  search_masters: (input) =>
    MASTERS.filter(
      (master) =>
        master.scope === asText(input['scope']) &&
        [master.code, master.ur, master.en].some((part) =>
          part.toLowerCase().includes(asText(input['text']).trim().toLowerCase()),
        ),
    ).map(masterHit),
  search_recent: (input) =>
    MASTERS.filter((master) => master.scope === asText(input['scope'])).map(masterHit),
  search_touch: () => null,
  translit_suggest: () => ({ items: [] }),
  translit_pairs_list: () => EMPTY_PAIRS,
  bikri_preview: (input) => {
    // `?own` is the lot of trip goods of E2E-34, the backend's real answer.
    if (devFlag('own')) return fitOwnPreview(ownReal.OWN_PREVIEW_E34, draftOf(input));
    const preview = previewFor(draftOf(input));
    // `#/dev/bikri?cash` shows the cash note: the entry takes cash in hand to -5,000 (PRD FR-6.7).
    return devFlag('cash') ? { ...preview, cash: CASH_BELOW_ZERO } : preview;
  },
  bikri_split_freight: (input) => {
    const lots = Array.isArray(input['lots']) ? input['lots'] : [];
    const first: unknown = lots[0];
    const key =
      typeof first === 'object' && first !== null && 'key' in first ? asText(first.key) : '';
    return [{ key, freight_paisa: EVENT1.lots[0]?.freight ?? 0 }];
  },
  assist_truck_memory: (input) =>
    asText(input['truck_no']).toUpperCase().startsWith('LES')
      ? TRUCK_MEMORY
      : { last: null, same_day: null, suggestions: [] },
  assist_seller_memory: () => SELLER_MEMORY,
  // Plan 13: the saved بکری, the lists, drafts and the one-time people.
  bikri_save: (input) => ({
    bikri_id: 41,
    number: 1,
    entry_id: 7,
    preview: previewFor(draftOf(input)),
  }),
  bikri_get: () =>
    devFlag('own')
      ? devValue('own') === '41'
        ? ownReal.OWN_GET_E41
        : ownReal.OWN_GET_E34
      : savedEvent1({
          status: devFlag('cancelled') ? 'cancelled' : 'saved',
          cancel_reason: devFlag('cancelled') ? 'wrong truck' : null,
        }),
  bikri_list: () =>
    bikriListOf(
      devFlag('empty') ? [] : [listRow(), listRow({ id: 42, number: 2, status: 'cancelled' })],
      0,
      0,
      // The sums of the saved one: the cancelled posting is left out of them (PRD 12.9).
      devFlag('empty')
        ? {}
        : {
            qty_received: 50,
            qty_sold: 50,
            gross: paisaForTest(2_940_000),
            net: paisaForTest(1_931_600),
            buyer_total: paisaForTest(3_156_400),
          },
    ),
  bikri_find: () => [listRow(), listRow({ id: 42, number: 2, truck_no: 'LHR-0077' })],
  bikri_navigate: () => 41,
  // `#/dev/bikri?offer` shows the start-up offer of an unfinished بکری (PRD FR-12.10).
  drafts_get: () => {
    if (!devFlag('offer')) return null;
    const stores = createBikriDraft('10-10-2026');
    seedEvent1(stores);
    return { payload_json: serialiseDraft(stores), saved_at: '2026-10-10T07:30:00Z' };
  },
  drafts_save: () => ({ saved_at: '2026-10-10T07:30:00Z' }),
  drafts_discard: () => null,
  search_pukhta_people: () => [SHAHID, NASIR],
  // Plan 14: the print box, the preview and the Printing settings. `#/dev/bikri?id=41&windows`
  // pretends the computer is Windows (the print goes through); without it the answer is the one a
  // Mac gives, ERR_PRINT_WINDOWS_ONLY, so the preview offer can be photographed.
  settings_get_printing: () => printingSettings(),
  settings_set_printing: (input) => ({
    ...printingSettings(),
    ...input,
    copies: Number(input['copies']),
  }),
  reports_list_printers: () => (devFlag('noprinters') ? [] : ['HP LaserJet Pro', 'Epson TM-T20']),
  reports_bikri_print_data: (input) =>
    printDataFor(input['doc'] as 'seller_bill', {
      language: input['language'] === 'en' ? 'en' : 'ur',
    }),
  reports_print: (input) => {
    if (!devFlag('windows')) throw windowsOnly();
    const docs = Array.isArray(input['docs']) ? (input['docs'] as string[]) : [];
    return {
      printed: docs,
      pdf_paths: input['to'] === 'pdf' ? docs.map((doc) => `prints/${doc}-1.pdf`) : null,
    };
  },
  // Plan 16: goods left, later sale, edit, cancel and undo (bikri-plan16-data.ts). `#/dev/bikri?
  // closed` is a closed day (the edit and the cancel are refused); `empty` is nothing left.
  bikri_stock_list: () =>
    devFlag('own')
      ? ownReal.STOCK_OWN_ROWS
      : devFlag('empty')
        ? stockList([])
        : stockList([
            stockRow(),
            stockRow({
              lot_id: 41,
              bikri_id: 43,
              bikri_number: 3,
              received_date: '2026-10-08',
              qty_received: 30,
              qty_sold: 20,
              balance: 10,
              days_waiting: 2,
            }),
          ]),
  bikri_later_sale_preview: (input) =>
    laterSaleAnswer(Array.isArray(input['sales']) ? (input['sales'] as SaleInput[]) : [], 5),
  bikri_later_sale_save: (input) => ({
    bikri_id: 41,
    number: 1,
    lot_id: asNumber(input['lot_id']),
    entry_id: 9,
    preview: laterSaleAnswer([], 5),
  }),
  bikri_open_for_edit: () =>
    devFlag('own') ? ownReal.OWN_OPEN_FOR_EDIT : editable({ editable: !devFlag('closed') }),
  bikri_edit_preview: (input) =>
    devFlag('own')
      ? fitOwnPreview(ownReal.OWN_PREVIEW_E34, draftOf(input))
      : previewFor(draftOf(input)),
  bikri_edit_save: (input) => {
    // `#/dev/bikri?id=41&edit=7&rule=lot_item_locked` is the backend refusing the edit by that rule.
    const rule = devValue('rule');
    if (rule !== null) throw editNotAllowed(rule);
    return {
      bikri_id: 41,
      number: 1,
      version: 4,
      old_entry_id: 7,
      reversal_entry_id: 10,
      entry_id: 11,
      preview: previewFor(draftOf(input)),
    };
  },
  bikri_cancel_preview: () =>
    cancelPreview(devFlag('closed') ? { closed_dates: ['2026-10-10'], can_cancel: false } : {}),
  bikri_cancel: () => ({ bikri_id: 41, number: 1, reversal_entry_ids: [10] }),
  bikri_undo: () => undoResult(),
  bikri_write_off: (input) => ({
    lot_id: asNumber(input['lot_id']),
    move_id: 70,
    qty: 5,
    qty_written_off: 5,
    balance: 0,
  }),
  bikri_write_off_undo: () => ({
    lot_id: 40,
    reversal_move_id: 71,
    qty_written_off: 0,
    balance: 5,
  }),
  // Plan 15: the vouchers, answered from the books of the worked day (voucher-books.ts).
  vouchers_preview: (input) => books.preview(input['entry'] as VoucherEntry),
  vouchers_save_receipt: (input) => books.save({ type: 'receipt', input: input as ReceiptInput }),
  vouchers_save_payment: (input) => books.save({ type: 'payment', input: input as PaymentInput }),
  vouchers_save_advance: (input) => books.save({ type: 'advance', input: input as AdvanceInput }),
  vouchers_edit: (input) => books.edit(input as never),
  vouchers_cancel: (input) => books.cancel(input as never),
  vouchers_get: (input) => books.get(asNumber(input['id'])),
  vouchers_list: (input) => books.list(input as never),
  assist_open_bills: (input) => books.openBills({ account_id: asNumber(input['account_id']) }),
  assist_open_payables: (input) => books.openPayables({ seller_id: asNumber(input['seller_id']) }),
  assist_advances: (input) => books.advances(asNumber(input['seller_id'])),
  assist_collection_list: () => collectionList(devFlag('empty') ? 'none' : 'two'),
  vouchers_save_receipts_batch: (input) => {
    const rows = Array.isArray(input['rows']) ? (input['rows'] as ReceiptInput[]) : [];
    return {
      rows: rows.map((row) => ({
        saved: books.save({ type: 'receipt', input: row }),
        skipped: false,
        errors: [],
      })),
      saved_paisa: paisaForTest(0),
    };
  },
  reports_voucher_print_data: (input) =>
    voucherPrintData('receipt', { language: input['language'] === 'en' ? 'en' : 'ur' }),
  pukhta_register: (input) =>
    pukhtaRegisterOf(
      devFlag('empty')
        ? []
        : [
            registerRow(),
            registerRow({
              bikri_id: 43,
              bikri_number: 3,
              name_ur: NASIR.name_ur,
              name_en: NASIR.name_en,
              phone: NASIR.phone,
              amount: paisaForTest(200_000),
              paid: false,
              open: paisaForTest(200_000),
            }),
            registerRow({ bikri_id: 44, bikri_number: 4, cancelled: true, paid: false }),
          ],
      0,
      0,
      // As the backend: the sums are given only when a side is chosen (they are null otherwise).
      devFlag('empty') || input['side'] === null || input['side'] === undefined
        ? {}
        : {
            qty: 20,
            amount: paisaForTest(1_480_000),
            commission: paisaForTest(144_000),
            open: paisaForTest(200_000),
          },
    ),
};

function masterHit(master: Master) {
  return {
    id: master.id,
    kind: master.scope,
    code: master.code,
    name_ur: master.ur,
    name_en: master.en,
    extra_ur: '',
    extra_en: '',
    is_active: true,
  };
}

/**
 * The backend's real preview of the own lot, with the keys of the draft it is asked about (the real
 * preview echoes the key of each lot and line it was given).
 */
function fitOwnPreview(real: typeof ownReal.OWN_PREVIEW_E34, draft: BikriDraft) {
  const preview = structuredClone(real);
  preview.lots.forEach((lot, lotIndex) => {
    lot.key = draft.lots[lotIndex]?.key ?? lot.key;
    lot.lines.forEach((line, lineIndex) => {
      line.key = draft.lots[lotIndex]?.sales[lineIndex]?.key ?? line.key;
    });
  });
  return preview;
}

/** The draft inside the input of `bikri_preview`. The screen sends it in the shape of the contract. */
function draftOf(input: Input): BikriDraft {
  return input['draft'] as BikriDraft;
}

/** What the real `invoke` hands to the command: `{ input: {...} }` for the commands of this app. */
function inputOf(args: unknown): Input {
  if (typeof args !== 'object' || args === null) return {};
  const inner = (args as Input)['input'];
  return typeof inner === 'object' && inner !== null ? (inner as Input) : {};
}

/** What Tauri rejects with: an error that also carries the UiError fields the app reads. */
function refusal(): Error {
  return Object.assign(new Error('ERR_NOT_IMPLEMENTED'), {
    ok: false,
    errors: [{ code: 'ERR_NOT_IMPLEMENTED', field: null, values: {} }],
    ref: '',
  });
}

/** The field each edit rule points to, as the backend sends it (plan 16, ERR_EDIT_NOT_ALLOWED). */
const EDIT_RULE_FIELDS: Readonly<Record<string, string>> = {
  day_changed: 'header.entry_date',
  lot_removed: 'lots',
  lot_not_in_posting: 'lots[0].lot_id',
  lot_order_changed: 'lots[0].lot_id',
  lot_seller_locked: 'lots[0].seller_account_id',
  lot_item_locked: 'lots[0].item_id',
  line_not_in_posting: 'lots[0].sales[0].sale_id',
  posting_lots_locked: 'lots',
};

/** The backend's refusal of an edit by one rule. */
function editNotAllowed(rule: string): Error {
  return Object.assign(new Error('ERR_EDIT_NOT_ALLOWED'), {
    ok: false,
    errors: [
      { code: 'ERR_EDIT_NOT_ALLOWED', field: EDIT_RULE_FIELDS[rule] ?? null, values: { rule } },
    ],
    ref: '',
  });
}

/** What a computer that is not Windows answers to a print (plan 14 contract 8). */
function windowsOnly(): Error {
  return Object.assign(new Error('ERR_PRINT_WINDOWS_ONLY'), {
    ok: false,
    errors: [{ code: 'ERR_PRINT_WINDOWS_ONLY', field: null, values: {} }],
    ref: '',
  });
}

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      invoke: (command: string, args: unknown): Promise<unknown> => {
        // The pages of the روکڑ area and of the gardens answer their own commands first.
        const own = isRokarPage()
          ? rokarHandlers[command]
          : isGardensPage()
            ? gardensHandlers[command]
            : isIndexPage()
              ? indexHandlers[command]
              : undefined;
        const handler = own ?? handlers[command];
        // The delay is the answer time of a real command, so the screen's waiting looks real.
        return new Promise((resolve, reject) => {
          window.setTimeout(() => {
            if (handler === undefined) {
              reject(refusal());
              return;
            }
            try {
              resolve(handler(command === 'day_status' ? {} : inputOf(args)));
            } catch (failure: unknown) {
              reject(failure instanceof Error ? failure : refusal());
            }
          }, 20);
        });
      },
    },
  });
}

/** The draft of Event 1 (PRD 4.4) typed into the stores, for the sample page. */
export function seedEvent1(stores: BikriDraftStores): void {
  const { header, lots } = stores;
  const rahim = { id: IDS.rahim, code: '030110001', name_ur: 'رحیم', name_en: 'Rahim' };
  header.getState().patch({
    truckNo: 'LES-1234',
    zamindar: rahim,
    adda: { id: IDS.swat, code: '0001', name_ur: 'سوات اڈا', name_en: 'Swat Adda' },
    totalFreight: '3000',
  });
  const lot = lots.getState().rows[0];
  if (lot === undefined) return;
  lots.getState().patchRow(lot.rowId, {
    picked: {
      seller: rahim,
      item: { id: IDS.banana, code: '0001', name_ur: 'کیلا', name_en: 'Banana' },
      packing: { id: IDS.box, code: '0001', name_ur: 'پیٹی', name_en: 'Box' },
    },
    typed: {
      qty: '50',
      freight: '3000',
      labour: '5',
      labourMethod: 'per_unit',
      munshiana: '50',
      cash: '5000',
      sellerFilled: '1',
      memoryFilledFor: String(IDS.rahim),
    },
  });
  const sales = stores.salesFor(lot.rowId);
  const first = sales.getState().rows[0];
  if (first === undefined) return;
  const akram = {
    id: IDS.akram,
    code: '03040001',
    name_ur: 'اکرم ٹول ٹیکس',
    name_en: 'Akram Toll Tax',
  };
  const arshad = {
    id: IDS.arshad,
    code: '03040002',
    name_ur: 'ارشد ملتانی',
    name_en: 'Arshad Multani',
  };
  sales.getState().patchRow(first.rowId, {
    picked: { buyer: akram },
    typed: { qty: '20', rate: '600', qtyFilled: '1' },
  });
  sales.getState().addRow({ qty: '30', rate: '580', qtyFilled: '1' }, { buyer: arshad });
}

/** The own lot of E2E-34 typed into the stores, for the sample page `#/dev/bikri?own`. */
export function seedOwn(stores: BikriDraftStores): void {
  const { header, lots } = stores;
  header.getState().patch({ truckNo: 'OWN-1' });
  const lot = lots.getState().rows[0];
  if (lot === undefined) return;
  lots.getState().patchRow(lot.rowId, {
    picked: {
      seller: { id: OWN_ID, code: OWN_CODE, name_ur: 'اپنا مال', name_en: 'Own Goods (Trips)' },
      item: { id: 5, code: '5', name_ur: 'مالٹا', name_en: 'Orange' },
      packing: { id: 1, code: '1', name_ur: 'پیٹی', name_en: 'Box' },
    },
    typed: { qty: '15', purchase: '1', sellerFilled: '1' },
  });
  const sales = stores.salesFor(lot.rowId);
  const first = sales.getState().rows[0];
  if (first === undefined) return;
  sales.getState().patchRow(first.rowId, {
    picked: {
      buyer: {
        id: IDS.akram,
        code: '03040001',
        name_ur: 'اکرم ٹول ٹیکس',
        name_en: 'Akram Toll Tax',
      },
    },
    typed: { qty: '15', rate: '600', qtyFilled: '1' },
  });
}
