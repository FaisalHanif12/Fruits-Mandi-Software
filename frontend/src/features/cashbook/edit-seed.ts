// A saved voucher as the form of its screen holds it (plan 44, PRD FR-7.70): the typed entry the
// backend sends (`vouchers_open_for_edit`) becomes the draft of the خرچہ, ادھار رقم or الگ کھاتہ form,
// with the records its pickers hold looked up by id. The draft is text, like a typed one; nothing
// here works out money (CLAUDE.md rule 1).

import { useQuery } from '@tanstack/react-query';
import { accountsGet, pickedOf, type PickedAccount } from '../../shared/accounts';
import { findMaster } from '../../shared/search';
import type { PickerValue } from '../../shared/ui';
import type { ExpenseDraft } from './expense-draft';
import { newExpenseDraft } from './expense-draft';
import { GENERAL_CR, GENERAL_DR, GENERAL_NARRATION } from './general-draft';
import type { KhataDraft } from './khata-draft';
import { newKhataDraft } from './khata-draft';
import type { LentDraft } from './lent-draft';
import { newLentDraft } from './lent-draft';
import type { VoucherEditable } from './types';

/** The record of an expense head as its picker holds it; a head that was switched off is kept by id. */
async function headOf(id: number | null): Promise<PickerValue | null> {
  if (id === null) return null;
  const found = await findMaster('expense_head', id);
  return found === null
    ? { id, code: '', name_ur: String(id), name_en: String(id) }
    : pickedOf(found);
}

/** The khata or the person of the voucher: its account, as the picker holds it. */
function accountPicked(editable: VoucherEditable): PickerValue | null {
  return editable.row.account === null ? null : pickedOf(editable.row.account);
}

function textOf(value: string | null | undefined): string {
  return value ?? '';
}

export async function expenseDraftOf(editable: VoucherEditable): Promise<ExpenseDraft> {
  const entry = editable.entry;
  switch (entry.type) {
    case 'shop_expense':
      return {
        ...newExpenseDraft('shop_expense', ''),
        date: entry.date,
        amount: entry.amount,
        narration: textOf(entry.narration),
        head: await headOf(entry.expense_head_id),
        paidTo: textOf(entry.paid_to),
      };
    case 'personal_expense':
      return {
        ...newExpenseDraft('personal_expense', ''),
        date: entry.date,
        amount: entry.amount,
        narration: textOf(entry.narration),
        partnerId: entry.partner_id === null ? '' : String(entry.partner_id),
        reason: entry.reason,
        returnBy: textOf(entry.return_by),
        takenBy: textOf(entry.taken_by),
      };
    case 'personal_return':
      return {
        ...newExpenseDraft('personal_return', ''),
        date: entry.date,
        amount: entry.amount,
        narration: textOf(entry.narration),
        partnerId: entry.partner_id === null ? '' : String(entry.partner_id),
      };
    default:
      throw new Error('not an expense');
  }
}

export async function lentDraftOf(editable: VoucherEditable): Promise<LentDraft> {
  const entry = editable.entry;
  if (entry.type === 'money_lent') {
    return {
      ...newLentDraft('money_lent', ''),
      date: entry.date,
      person: accountPicked(editable),
      amount: entry.amount,
      reason: entry.reason,
      returnBy: textOf(entry.return_by),
      narration: textOf(entry.narration),
    };
  }
  if (entry.type === 'money_lent_return') {
    return {
      ...newLentDraft('money_lent_return', ''),
      date: entry.date,
      person: accountPicked(editable),
      amount: entry.amount,
      mode: entry.mode,
      salaryHead: await headOf(entry.salary_head_id),
      narration: textOf(entry.narration),
    };
  }
  throw new Error('not a loan');
}

export function khataDraftOf(editable: VoucherEditable): KhataDraft {
  const entry = editable.entry;
  if (entry.type === 'khata_out' || entry.type === 'khata_in') {
    return {
      ...newKhataDraft(entry.type, ''),
      date: entry.date,
      khata: accountPicked(editable),
      amount: entry.amount,
      reason: entry.reason,
      narration: textOf(entry.narration),
    };
  }
  throw new Error('not a khata entry');
}

/** One line of a saved جنرل entry as the grid's store holds it, with its account. */
export interface GeneralSeedLine {
  typed: Record<string, string>;
  account: PickedAccount | null;
}

/** A saved جنرل entry as the screen holds it. */
export interface GeneralSeed {
  /** `DD-MM-YYYY`. */
  date: string;
  narration: string;
  lines: GeneralSeedLine[];
}

/** The lines of a saved جنرل entry, with each account looked up by id. */
export async function generalSeedOf(editable: VoucherEditable): Promise<GeneralSeed> {
  const entry = editable.entry;
  if (entry.type !== 'general') throw new Error('not a journal entry');
  const ids = [
    ...new Set(entry.lines.flatMap((line) => (line.account_id === null ? [] : [line.account_id]))),
  ];
  const accounts = new Map(
    await Promise.all(ids.map(async (id) => [id, pickedOf(await accountsGet(id))] as const)),
  );
  const lines = entry.lines.map((line): GeneralSeedLine => {
    const account = line.account_id === null ? null : (accounts.get(line.account_id) ?? null);
    return {
      typed: {
        [GENERAL_DR]: line.dr,
        [GENERAL_CR]: line.cr,
        [GENERAL_NARRATION]: textOf(line.narration),
      },
      account,
    };
  });
  return { date: entry.date, narration: textOf(entry.narration), lines };
}

/**
 * The draft of an edit, read once for each version of the voucher. The screen waits for it; it is
 * never rebuilt while the user types.
 */
export function useEditDraft<D>(
  editable: VoucherEditable | undefined,
  convert: (editable: VoucherEditable) => D | Promise<D>,
) {
  return useQuery({
    queryKey: ['cashbook', 'edit-draft', editable?.row.id, editable?.row.version],
    queryFn: async () => {
      if (editable === undefined) throw new Error('no voucher');
      return convert(editable);
    },
    enabled: editable !== undefined,
    staleTime: Infinity,
    gcTime: 0,
  });
}
