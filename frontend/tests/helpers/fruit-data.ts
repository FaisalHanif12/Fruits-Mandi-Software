// Test data: the books of fruit taken (plan 17d contract, PRD 16.7), in memory, with no test runner
// in it, so the development page can use it too. A test sets the command wrappers from it
// (tests/helpers/fruit.ts); the development page /dev/fruit answers from it. It answers the way a
// healthy backend answers, from the worked numbers of PRD 27.2 E2E-42 (after E2E-16, 5 پیٹی کیلا
// are left in رحیم's lot): 2 × 580 = 1,160 taken home by A, رحیم credited 1,160, the lot's balance 3;
// guest fruit 3,000 shared 1,000 / 1,000 / 1,000, and 1,000 over three partners as 334 / 333 / 333
// (C27). The sums are this test double's, written the way the plan has them; the screen only shows
// what comes back and never works out an amount (CLAUDE.md rule 1). Every shape is the generated
// binding, so a change of the contract fails to compile here first.

import type { ErrorCode } from '../../src/bindings/ErrorCode';
import type { FieldError } from '../../src/bindings/FieldError';
import type { FruitCancelInput } from '../../src/bindings/FruitCancelInput';
import type { FruitCancelled } from '../../src/bindings/FruitCancelled';
import type { FruitEditInput } from '../../src/bindings/FruitEditInput';
import type { FruitEdited } from '../../src/bindings/FruitEdited';
import type { FruitGetInput } from '../../src/bindings/FruitGetInput';
import type { FruitInput } from '../../src/bindings/FruitInput';
import type { FruitList } from '../../src/bindings/FruitList';
import type { FruitListInput } from '../../src/bindings/FruitListInput';
import type { FruitLotRow } from '../../src/bindings/FruitLotRow';
import type { FruitLots } from '../../src/bindings/FruitLots';
import type { FruitLotsInput } from '../../src/bindings/FruitLotsInput';
import type { FruitNumbers } from '../../src/bindings/FruitNumbers';
import type { FruitPart } from '../../src/bindings/FruitPart';
import type { FruitPreview } from '../../src/bindings/FruitPreview';
import type { FruitRow } from '../../src/bindings/FruitRow';
import type { FruitSaved } from '../../src/bindings/FruitSaved';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { Working } from '../../src/bindings/Working';
import type { WorkingArg } from '../../src/bindings/WorkingArg';
import { CommandError } from '../../src/shared/api';
import { paisaForTest } from './paisa';
import { refusal, type RefusalItem } from './ui-error';

const money = (rupees: number): ReturnType<typeof paisaForTest> => paisaForTest(rupees * 100);

const person = (id: number, ur: string, en: string, code = ''): RecordRef => ({
  id,
  code,
  name_ur: ur,
  name_en: en,
});
export const PARTNER_A = person(1, 'الف', 'Partner A');
export const PARTNER_B = person(2, 'ب', 'Partner B');
export const PARTNER_C = person(3, 'ج', 'Partner C');
export const RAHIM_REF = person(21, 'رحیم', 'Rahim', '0010');
export const HAJI_NAZIR = person(31, 'حاجی نذیر', 'Haji Nazir', '0012');
export const BANANA_REF = person(62, 'کیلا', 'Banana', '0062');
export const APPLE_REF = person(63, 'سیب', 'Apple', '0063');
export const BOX_REF = person(71, 'پیٹی', 'Box', '0071');

/** The partners in the order they were made, with the total of each before E2E-42 (C22). */
const PARTNERS: readonly { ref: RecordRef; total: number }[] = [
  { ref: PARTNER_A, total: 1_424 },
  { ref: PARTNER_B, total: 1_068 },
  { ref: PARTNER_C, total: 1_068 },
];

/** A working the way the backend names it: a message and its values. */
export function working(id: string, args: readonly WorkingArg[]): Working {
  return { message_id: id, args: [...args], rule_id: null };
}
const moneyArg = (name: string, rupees: number): WorkingArg => ({
  name,
  value: { kind: 'money', paisa: money(rupees) },
});
const countArg = (name: string, value: number): WorkingArg => ({
  name,
  value: { kind: 'count', value },
});

/** Lot 40 of بکری 1: رحیم's کیلا in پیٹی, 5 left (E2E-16). */
export function lotRow(over: Partial<FruitLotRow> = {}): FruitLotRow {
  return {
    lot_id: 40,
    lot_no: 1,
    bikri_id: 41,
    bikri_number: 1,
    received_date: '2026-10-09',
    seller: RAHIM_REF,
    pukhta: null,
    item: BANANA_REF,
    packing: BOX_REF,
    sold_by_weight: false,
    balance: 5,
    ...over,
  };
}

export interface FruitOptions {
  lots?: FruitLotRow[];
}

export interface FruitStore {
  lots: FruitLotRow[];
  rows: FruitRow[];
  saved: { entries: FruitInput[]; edits: FruitEditInput[]; cancels: FruitCancelInput[] };
  /** A refusal the next save, edit or cancel throws. */
  refuse: { save?: RefusalItem[]; cancel?: RefusalItem[] };
}

const problem = (
  code: ErrorCode,
  field: string,
  values: FieldError['values'] = {},
): FieldError => ({
  code,
  field,
  values,
});

function refuse(items: RefusalItem[]): never {
  throw new CommandError(refusal(items));
}

/** Digits of what was typed, commas away: the books of a test double work on whole rupees. */
const digits = (text: string): number => Number(text.replace(/,/g, '').trim());

/** C27 for the test double: equal parts, the rupees left over to the partner made first. */
function guestParts(value: number): number[] {
  const each = Math.floor(value / PARTNERS.length);
  const left = value - each * PARTNERS.length;
  return PARTNERS.map((_, index) => each + (index < left ? 1 : 0));
}

export function createFruitBooks(options: FruitOptions = {}) {
  const store: FruitStore = {
    lots: options.lots ?? [lotRow()],
    rows: [],
    saved: { entries: [], edits: [], cancels: [] },
    refuse: {},
  };
  // The value of each saved row in whole rupees, kept next to the rows so the totals of the list
  // never read a Paisa as a number (a test double works on rupees).
  const rupees = new Map<number, number>();
  const lotOf = (id: number | null) => store.lots.find((lot) => lot.lot_id === id) ?? null;

  const preview = (input: FruitInput): FruitPreview => {
    const errors: FieldError[] = [];
    const guests = input.for_guests;
    if (!guests && input.partner_id === null)
      errors.push(problem('ERR_PARTNER_REQUIRED', 'partner_id'));
    const lot = input.source === 'shop' ? lotOf(input.lot_id) : null;
    if (input.source === 'shop') {
      if (input.lot_id === null) errors.push(problem('ERR_REQUIRED', 'lot_id'));
      else if (lot === null) errors.push(problem('ERR_LOT_NOT_FOUND', 'lot_id'));
    } else {
      if (input.agent_account_id === null) errors.push(problem('ERR_REQUIRED', 'agent_account_id'));
      if (input.item_id === null) errors.push(problem('ERR_REQUIRED', 'item_id'));
      if (input.packing_id === null) errors.push(problem('ERR_REQUIRED', 'packing_id'));
    }
    const qty = digits(input.qty);
    // An entry that is being edited gives its quantity back to its lot first.
    const own = store.rows.find((row) => row.id === input.editing_id);
    const available =
      lot === null ? 0 : lot.balance + (own?.lot?.lot_id === lot.lot_id ? own.qty : 0);
    if (input.qty.trim() === '') errors.push(problem('ERR_REQUIRED', 'qty'));
    else if (lot !== null && qty > available) {
      errors.push(problem('ERR_QTY_EXCEEDS_LOT', 'qty', { available }));
    }
    const rate = digits(input.rate);
    if (input.rate.trim() === '') errors.push(problem('ERR_REQUIRED', 'rate'));
    else if (rate <= 0) errors.push(problem('ERR_MUST_BE_MORE_THAN_ZERO', 'rate'));
    // The last rate of the item, shown as grey text only (PRD FR-7.53).
    const known = lot !== null || (input.item_id !== null && input.packing_id !== null);
    const rate_hint: FruitPreview['rate_hint'] = known
      ? { rate_paisa: money(580), source: 'today', date: '2026-10-10' }
      : null;
    if (errors.length > 0 || lot === null) {
      // Fruit from another shop has no lot; the numbers need the item picked as typed.
      if (errors.length > 0 || input.source === 'shop') {
        return { numbers: null, errors, warnings: [], rate_hint };
      }
    }
    const value = qty * rate;
    const amounts = guests ? guestParts(value) : [value];
    // A part of 0 is left out of the parts (the backend does not save it).
    const parts: FruitPart[] = guests
      ? PARTNERS.flatMap((partner, index) => {
          const amount = amounts[index] ?? 0;
          return amount === 0
            ? []
            : [
                {
                  partner: partner.ref,
                  amount_paisa: money(amount),
                  total_before_paisa: money(partner.total),
                  total_after_paisa: money(partner.total - amount),
                },
              ];
        })
      : PARTNERS.filter((partner) => partner.ref.id === input.partner_id).map((partner) => ({
          partner: partner.ref,
          amount_paisa: money(value),
          total_before_paisa: money(partner.total),
          total_after_paisa: money(partner.total - value),
        }));
    const numbers: FruitNumbers = {
      item: lot?.item ?? BANANA_REF,
      packing: lot?.packing ?? BOX_REF,
      qty,
      weight_grams: null,
      rate_paisa: money(rate),
      value_paisa: money(value),
      value_working: working('working-line-value', [
        countArg('qty', qty),
        moneyArg('rate', rate),
        moneyArg('result', value),
      ]),
      parts,
      parts_working: guests
        ? working('working-guest-fruit-split', [
            moneyArg('value', value),
            countArg('partners', PARTNERS.length),
            moneyArg('result', amounts[0] ?? 0),
          ])
        : null,
      credit: {
        account: lot === null ? HAJI_NAZIR : lot.seller,
        pukhta: null,
      },
      lot:
        lot === null
          ? null
          : { lot_id: lot.lot_id, balance_before: available, balance_after: available - qty },
    };
    return { numbers, errors: [], warnings: [], rate_hint };
  };

  const rowFrom = (id: number, version: number, input: FruitInput): FruitRow => {
    const shown = preview(input);
    const numbers = shown.numbers;
    if (numbers === null)
      return refuse(shown.errors.map((item) => ({ code: item.code, field: item.field })));
    const lot = lotOf(input.lot_id);
    return {
      id,
      version,
      number: id,
      entry_date: '2026-10-10',
      partner: input.for_guests
        ? null
        : (PARTNERS.find((partner) => partner.ref.id === input.partner_id)?.ref ?? null),
      source: input.source,
      lot:
        lot === null
          ? null
          : {
              lot_id: lot.lot_id,
              lot_no: lot.lot_no,
              bikri_id: lot.bikri_id,
              bikri_number: lot.bikri_number,
            },
      credit_account: numbers.credit.account,
      pukhta: null,
      item: numbers.item,
      packing: numbers.packing,
      qty: numbers.qty,
      weight_grams: numbers.weight_grams,
      rate_paisa: numbers.rate_paisa,
      value_paisa: numbers.value_paisa,
      note: input.note,
      status: 'saved',
      cancel_reason: null,
      parts: numbers.parts.map((part) => ({
        partner: part.partner,
        amount_paisa: part.amount_paisa,
      })),
      journal_entry_id: 100 + id,
      can_change: true,
    };
  };

  const takeFromLot = (lotId: number | null, qty: number) => {
    const lot = lotOf(lotId);
    if (lot !== null) lot.balance -= qty;
  };

  return {
    store,
    preview,
    save: (input: FruitInput): FruitSaved => {
      if (store.refuse.save !== undefined) return refuse(store.refuse.save);
      const row = rowFrom(store.rows.length + 1, 1, input);
      takeFromLot(input.lot_id, row.qty);
      rupees.set(row.id, digits(input.qty) * digits(input.rate));
      store.saved.entries.push(input);
      store.rows.push(row);
      return {
        id: row.id,
        number: row.number,
        version: 1,
        journal_entry_id: 100 + row.id,
        preview: preview(input),
      };
    },
    edit: (input: FruitEditInput): FruitEdited => {
      if (store.refuse.save !== undefined) return refuse(store.refuse.save);
      const index = store.rows.findIndex((row) => row.id === input.id);
      const old = store.rows[index];
      if (old === undefined) return refuse([{ code: 'ERR_NOT_FOUND', field: 'id' }]);
      if (old.status === 'cancelled') return refuse([{ code: 'ERR_FRUIT_CANCELLED', field: 'id' }]);
      if (input.reason.trim() === '')
        return refuse([{ code: 'ERR_REASON_REQUIRED', field: 'reason' }]);
      // The new values are checked with the old quantity given back (the preview does that for
      // `editing_id`); then the old quantity goes back and the new one is taken: a reversal and a
      // new posting.
      const row = rowFrom(old.id, old.version + 1, input.entry);
      takeFromLot(old.lot?.lot_id ?? null, -old.qty);
      takeFromLot(input.entry.lot_id, row.qty);
      rupees.set(row.id, digits(input.entry.qty) * digits(input.entry.rate));
      store.rows[index] = row;
      store.saved.edits.push(input);
      return {
        id: row.id,
        number: row.number,
        version: row.version,
        journal_entry_id: 200 + row.id,
        reversal_entry_id: 300 + row.id,
        preview: preview(input.entry),
      };
    },
    cancel: (input: FruitCancelInput): FruitCancelled => {
      if (store.refuse.cancel !== undefined) return refuse(store.refuse.cancel);
      const row = store.rows.find((candidate) => candidate.id === input.id);
      if (row === undefined) return refuse([{ code: 'ERR_NOT_FOUND', field: 'id' }]);
      if (row.status === 'cancelled') return refuse([{ code: 'ERR_FRUIT_CANCELLED', field: 'id' }]);
      takeFromLot(row.lot?.lot_id ?? null, -row.qty);
      row.status = 'cancelled';
      row.cancel_reason = input.reason;
      row.can_change = false;
      row.journal_entry_id = null;
      store.saved.cancels.push(input);
      return { id: row.id, number: row.number, reversal_entry_id: 300 + row.id };
    },
    get: (input: FruitGetInput): FruitRow => {
      const row = store.rows.find((candidate) => candidate.id === input.id);
      if (row === undefined) return refuse([{ code: 'ERR_NOT_FOUND', field: 'id' }]);
      return row;
    },
    list: (input: FruitListInput): FruitList => {
      const rows = store.rows.filter(
        (row) =>
          (input.partner_id === null || row.partner?.id === input.partner_id) &&
          (!input.guests_only || row.partner === null) &&
          (input.item_id === null || row.item.id === input.item_id) &&
          (input.source === null || row.source === input.source) &&
          (input.status === null || row.status === input.status),
      );
      const saved = rows.filter((row) => row.status === 'saved');
      const page = rows.slice(input.offset, input.offset + (input.limit === 0 ? 100 : input.limit));
      return {
        rows: page,
        total_count: rows.length,
        totals: {
          qty: saved.reduce((sum, row) => sum + row.qty, 0),
          value_paisa: money(saved.reduce((sum, row) => sum + (rupees.get(row.id) ?? 0), 0)),
        },
      };
    },
    lots: (input: FruitLotsInput): FruitLots => {
      const text = (input.search ?? '').trim().toLowerCase();
      const rows = store.lots.filter(
        (lot) =>
          lot.balance > 0 &&
          (text === '' ||
            [
              lot.item.name_en,
              lot.item.name_ur,
              lot.seller.name_en,
              lot.seller.name_ur,
              String(lot.bikri_number),
            ]
              .join(' ')
              .toLowerCase()
              .includes(text)),
      );
      return {
        rows: rows.slice(0, input.limit === 0 ? 100 : input.limit),
        total_count: rows.length,
      };
    },
  };
}

export type FruitBooks = ReturnType<typeof createFruitBooks>;
