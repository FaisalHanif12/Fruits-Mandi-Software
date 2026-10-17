// A stand-in for the Tauri bridge, for the development page /dev/bikri only (plan 12). In a plain
// browser tab there is no backend, so no screen can be looked at. This answers the commands of the
// بکری screen with the same canned numbers the component tests use (Event 1, PRD 4.4), so the
// screen can be opened in a browser and photographed in both languages. It answers nothing else:
// any other command is refused with ERR_NOT_IMPLEMENTED, like a stub. Never part of a build: only
// the development route imports it.

import { EVENT1, IDS, SELLER_MEMORY, TRUCK_MEMORY, previewFor } from './bikri-data';
import { NASIR, SHAHID, listRow, registerRow, savedEvent1 } from './bikri-saved';
import { paisaForTest } from './paisa';
import { PARTIES as BOOK_PARTIES, collectionList, createBooks, voucherRow } from './voucher-books';
import { printDataFor, voucherPrintData } from './print-data';
import { printingSettings } from './print-settings';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { AdvanceInput } from '../../src/bindings/AdvanceInput';
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
  if (scope !== 'any' && party.type !== scope) return false;
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
        mode: 'cash',
        bank_account_id: null,
        cheque_no: '',
        cheque_date: '',
        reason: '',
        narration: '',
        confirm_cash_below_zero: false,
      }).account,
    }),
  );
}

const handlers: Record<string, Handler> = {
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
  translit_pairs_list: () => [],
  bikri_preview: (input) => {
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
    savedEvent1({
      status: devFlag('cancelled') ? 'cancelled' : 'saved',
      cancel_reason: devFlag('cancelled') ? 'wrong truck' : null,
    }),
  bikri_list: () =>
    devFlag('empty') ? [] : [listRow(), listRow({ id: 42, number: 2, status: 'cancelled' })],
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
  pukhta_register: () =>
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
        const handler = handlers[command];
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
