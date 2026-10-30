// The commands of the cash book area as hooks (Code Standards F6). A saved entry refreshes
// everything that shows what it wrote (PRD 6.2): the cash book, the vouchers of the day, the party
// cards and balances, the partners' totals, the search recents and the day, so every screen shows
// the new numbers. Lists load page by page with the backend's totals over the whole filter
// (PRD FR-19.11, Code Standards F34, B31).

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { usePagedQuery, usePreviewQuery } from '../../shared/hooks';
import { VOUCHERS_KEY, vouchersPreview } from '../vouchers';
import {
  cashbookGet,
  expensePreview,
  generalPreview,
  generalSave,
  khataCreate,
  khataStatement,
  lentList,
  lentPersonCreate,
  partnerTakes,
  repeatLastMonth,
  saveKhataIn,
  saveKhataOut,
  saveMoneyLent,
  saveMoneyLentReturn,
  savePersonalExpense,
  savePersonalReturn,
  saveShopExpense,
} from './api';
import { inputOf } from './entry-support';
import type {
  ExpenseEntry,
  GeneralInput,
  GeneralPreview,
  GeneralPreviewInput,
  KhataCreateInput,
  LentPersonCreateInput,
  VoucherEntry,
  VoucherPreview,
  VoucherSaved,
} from './types';

export const CASHBOOK_KEY = ['cashbook'] as const;

/** Everything a saved entry changes, to read again (PRD 6.2). */
const READ_AGAIN: readonly (readonly string[])[] = [
  CASHBOOK_KEY,
  VOUCHERS_KEY,
  ['accounts'],
  ['coa'],
  ['partners'],
  ['search'],
  ['day_status'],
];

/** Reads again everything a saved entry changes (PRD 6.2). Also used by the capital screen. */
export function useReadAgain() {
  const queryClient = useQueryClient();
  return () =>
    Promise.all(
      READ_AGAIN.map((queryKey) => queryClient.invalidateQueries({ queryKey: [...queryKey] })),
    );
}

/** The cash book of a range, page by page, with the totals over the whole range (PRD 16.1). */
export function useCashbook(
  range: { from: string | null; to: string | null },
  groupByMonth = false,
) {
  return usePagedQuery({
    screen: 'cashbook.book',
    queryKey: [...CASHBOOK_KEY, 'book', range, groupByMonth],
    fetchPage: (offset, limit, sort) =>
      cashbookGet({ ...range, offset, limit, sort, group_by_month: groupByMonth && sort === null }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/**
 * The preview of the entry on the خرچہ form (PRD FR-16.8): each partner's share, or the partner's
 * total now and after. Asked 50 ms after the last key, only once there is an amount.
 */
export function useExpensePreview(entry: ExpenseEntry | null) {
  return usePreviewQuery([...CASHBOOK_KEY, 'expense-preview'], entry, (asked) =>
    expensePreview(asked, null),
  );
}

/**
 * The preview of an entry of the ادھار رقم or الگ کھاتہ screen: the generic voucher preview, the
 * same function as the save (Code Standards B5). The answer carries the type it was asked for, so
 * an older answer kept on screen while a new one is on its way is never taken for the answer of
 * another type.
 */
export function useEntryPreview(entry: VoucherEntry | null) {
  return usePreviewQuery(
    [...CASHBOOK_KEY, 'entry-preview'],
    entry,
    async (asked): Promise<TaggedPreview> => ({
      type: asked.type,
      preview: await vouchersPreview(asked, null),
    }),
  );
}

/** A preview and the type of entry it is the answer for. */
export interface TaggedPreview {
  type: VoucherEntry['type'];
  preview: VoucherPreview;
}

/** One partner's money taken, page by page (PRD FR-16.2). Never kept: a save changes it. */
export function usePartnerTakes(
  partnerId: number | null,
  groupByMonth = false,
  showFinished = false,
) {
  return usePagedQuery({
    screen: 'cashbook.takes',
    queryKey: [...CASHBOOK_KEY, 'takes', partnerId, groupByMonth, showFinished],
    enabled: partnerId !== null,
    fetchPage: (offset, limit, sort) =>
      partnerTakes({
        partner_id: partnerId ?? 0,
        offset,
        limit,
        sort,
        group_by_month: groupByMonth && sort === null,
        show_finished: showFinished,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
  });
}

/** This month's regular expenses proposed from last month (PRD FR-7.62). */
export function useRepeatLastMonth(month: string, enabled: boolean) {
  return useQuery({
    queryKey: [...CASHBOOK_KEY, 'repeat', month],
    queryFn: () => repeatLastMonth({ month }),
    enabled,
    staleTime: 0,
  });
}

/** One person's loans, page by page (PRD FR-16.5). */
export function useLentList(personId: number | null, groupByMonth = false, showFinished = false) {
  return usePagedQuery({
    screen: 'cashbook.lent',
    queryKey: [...CASHBOOK_KEY, 'lent', personId, groupByMonth, showFinished],
    enabled: personId !== null,
    fetchPage: (offset, limit, sort) =>
      lentList({
        person_id: personId ?? 0,
        offset,
        limit,
        sort,
        group_by_month: groupByMonth && sort === null,
        show_finished: showFinished,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
  });
}

/** A khata's entries with the running balance, page by page (PRD FR-16.6). */
export function useKhataStatement(khataId: number | null, groupByMonth = false) {
  return usePagedQuery({
    screen: 'cashbook.khata',
    queryKey: [...CASHBOOK_KEY, 'khata', khataId, groupByMonth],
    enabled: khataId !== null,
    fetchPage: (offset, limit, sort) =>
      khataStatement({
        khata_account_id: khataId ?? 0,
        from: null,
        to: null,
        offset,
        limit,
        sort,
        group_by_month: groupByMonth && sort === null,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
  });
}

/** Saves an entry of the خرچہ screen, by its type: its own command and posting (PRD FR-7.67). */
export function useSaveExpense() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (entry: ExpenseEntry): Promise<VoucherSaved> => {
      switch (entry.type) {
        case 'shop_expense':
          return saveShopExpense(inputOf(entry));
        case 'personal_expense':
          return savePersonalExpense(inputOf(entry));
        case 'personal_return':
          return savePersonalReturn(inputOf(entry));
      }
    },
    onSuccess: readAgain,
  });
}

/** Saves an entry of the ادھار رقم screen. */
export function useSaveLent() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (entry: Extract<VoucherEntry, { type: 'money_lent' | 'money_lent_return' }>) =>
      entry.type === 'money_lent'
        ? saveMoneyLent(inputOf(entry))
        : saveMoneyLentReturn(inputOf(entry)),
    onSuccess: readAgain,
  });
}

/** Saves an entry of the الگ کھاتہ screen. */
export function useSaveKhata() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (entry: Extract<VoucherEntry, { type: 'khata_out' | 'khata_in' }>) =>
      entry.type === 'khata_out' ? saveKhataOut(inputOf(entry)) : saveKhataIn(inputOf(entry)),
    onSuccess: readAgain,
  });
}

/** Makes a person who can borrow; the lists of accounts and the search are read again. */
export function useCreateLentPerson() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (input: LentPersonCreateInput) => lentPersonCreate(input),
    onSuccess: readAgain,
  });
}

/** Makes a new الگ کھاتہ. */
export function useCreateKhata() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (input: KhataCreateInput) => khataCreate(input),
    onSuccess: readAgain,
  });
}

/** What the جنرل preview is asked about, with the row each sent line came from. */
export interface GeneralAsk {
  input: GeneralPreviewInput;
  /** The row of each line of `input`, in the same order: the answer's `lines[i]` is `rowIds[i]`. */
  rowIds: readonly string[];
}

/** The answer of the preview with the rows it belongs to, so a problem finds its cell again. */
export interface GeneralAnswer {
  rowIds: readonly string[];
  preview: GeneralPreview;
}

/**
 * The preview of the جنرل entry (PRD 16.4): the totals, the line problems, the split and the cash.
 * Asked 50 ms after the last key. `ask` must keep its identity while the entry does not change.
 */
export function useGeneralPreview(ask: GeneralAsk | null) {
  return usePreviewQuery(
    [...CASHBOOK_KEY, 'general-preview'],
    ask,
    async (asked): Promise<GeneralAnswer> => ({
      rowIds: asked.rowIds,
      preview: await generalPreview(asked.input),
    }),
  );
}

/** Saves a جنرل entry. */
export function useSaveGeneral() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (input: GeneralInput) => generalSave(input),
    onSuccess: readAgain,
  });
}
