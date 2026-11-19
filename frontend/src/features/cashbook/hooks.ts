// The commands of the cash book area as hooks (Code Standards F6). A saved entry refreshes
// everything that shows what it wrote (PRD 6.2): the cash book, the vouchers of the day, the party
// cards and balances, the partners' totals, the search recents and the day, so every screen shows
// the new numbers. Lists load page by page with the backend's totals over the whole filter
// (PRD FR-19.11, Code Standards F34, B31).

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useMemo } from 'react';
import { usePagedQuery, usePreviewQuery } from '../../shared/hooks';
import { VOUCHERS_KEY, vouchersPreview } from '../vouchers';
import {
  cashbookCapitalWhere,
  cashbookGet,
  expensePreview,
  generalPreview,
  generalSave,
  khataCreate,
  khataSetLink,
  khataStatement,
  lentDues,
  lentList,
  lentPersonCreate,
  lentPersonUpdate,
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
  KhataCreateRequest,
  LentDuesInput,
  Link,
  LentPersonCreateRequest,
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
 * Where the partners' money is and whose it is (PRD FR-16.14, plan 78 E). It is read only when the card is open, and
 * read again after every save (the key starts with `cashbook`). A locked partner section is an error the card shows
 * as the code box (`ErrorState`).
 */
export function useCapitalWhere(enabled: boolean) {
  return useQuery({
    queryKey: [...CASHBOOK_KEY, 'capital-where'],
    queryFn: cashbookCapitalWhere,
    enabled,
    staleTime: 0,
  });
}

/**
 * The preview of the entry on the خرچہ form (PRD FR-16.8): each partner's share, or the partner's
 * total now and after. Asked 50 ms after the last key, only once there is an amount.
 */
export function useExpensePreview(entry: ExpenseEntry | null, replacing: number | null = null) {
  return usePreviewQuery([...CASHBOOK_KEY, 'expense-preview', replacing], entry, (asked) =>
    expensePreview(asked, replacing),
  );
}

/**
 * The preview of an entry of the ادھار رقم or الگ کھاتہ screen: the generic voucher preview, the
 * same function as the save (Code Standards B5). The answer carries the type it was asked for, so
 * an older answer kept on screen while a new one is on its way is never taken for the answer of
 * another type. `replacing` is the saved entry being edited: the preview is then the effect of the
 * change, with that entry taken out first (plan 44).
 */
export function useEntryPreview(
  entry: VoucherEntry | null,
  replacing: number | null = null,
  link: Link | null = null,
) {
  // `link` is what a NEW loan would be linked to, as typed on its form (PRD FR-16.13): the preview then shows
  // what the save will post for each partner. The caller keeps it the same object until it changes.
  const asked = useMemo(() => (entry === null ? null : { entry, link }), [entry, link]);
  return usePreviewQuery(
    [...CASHBOOK_KEY, 'entry-preview', replacing],
    asked,
    async (wanted): Promise<TaggedPreview> => ({
      type: wanted.entry.type,
      preview: await vouchersPreview(wanted.entry, replacing, wanted.link),
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
  range: { from: string | null; to: string | null } = { from: null, to: null },
  ready = true,
) {
  return usePagedQuery({
    screen: 'cashbook.takes',
    queryKey: [...CASHBOOK_KEY, 'takes', partnerId, groupByMonth, showFinished, range],
    enabled: partnerId !== null && ready,
    fetchPage: (offset, limit, sort) =>
      partnerTakes({
        partner_id: partnerId ?? 0,
        from: range.from,
        to: range.to,
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

/** One person's loans, page by page (PRD FR-16.5). `range` is the month window of the list (PRD FR-19.11). */
export function useLentList(
  personId: number | null,
  groupByMonth = false,
  showFinished = false,
  range: { from: string | null; to: string | null } = { from: null, to: null },
  ready = true,
) {
  return usePagedQuery({
    screen: 'cashbook.lent',
    queryKey: [...CASHBOOK_KEY, 'lent', personId, groupByMonth, showFinished, range],
    enabled: personId !== null && ready,
    fetchPage: (offset, limit, sort) =>
      lentList({
        person_id: personId ?? 0,
        from: range.from,
        to: range.to,
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

/**
 * A khata's entries with the running balance, page by page (PRD FR-16.6). `range` is the month window of the
 * list (PRD FR-19.11); no range is the whole history.
 */
export function useKhataStatement(
  khataId: number | null,
  groupByMonth = false,
  range: { from: string | null; to: string | null } = { from: null, to: null },
  ready = true,
) {
  return usePagedQuery({
    screen: 'cashbook.khata',
    queryKey: [...CASHBOOK_KEY, 'khata', khataId, groupByMonth, range],
    enabled: khataId !== null && ready,
    fetchPage: (offset, limit, sort) =>
      khataStatement({
        khata_account_id: khataId ?? 0,
        from: range.from,
        to: range.to,
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

/** One entry of the ادھار رقم screen and, for money lent, the link the form asks for (null keeps his link). */
export interface LentSave {
  entry: Extract<VoucherEntry, { type: 'money_lent' | 'money_lent_return' }>;
  link: Link | null;
}

/** Saves an entry of the ادھار رقم screen. */
export function useSaveLent() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: ({ entry, link }: LentSave) =>
      entry.type === 'money_lent'
        ? saveMoneyLent({ ...inputOf(entry), link })
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
    mutationFn: (input: LentPersonCreateRequest) => lentPersonCreate(input),
    onSuccess: readAgain,
  });
}

/** Changes the type and link of a person who is lent money (plan 76 point 10). */
export function useUpdateLentPerson() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: lentPersonUpdate,
    onSuccess: readAgain,
  });
}

/** Makes a new الگ کھاتہ with its link. */
export function useCreateKhata() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (input: KhataCreateRequest) => khataCreate(input),
    onSuccess: readAgain,
  });
}

/** Changes the link of a khata (plan 76 point 5); its page and the lists are read again. */
export function useSetKhataLink() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: khataSetLink,
    onSuccess: readAgain,
  });
}

/**
 * One person's type, link and whether he owes anything, for the ادھار رقم form: the first row of his loans,
 * which carries them with his balance. Never kept: a save changes the balance.
 */
export function useLentPerson(personId: number | null) {
  return useQuery({
    queryKey: [...CASHBOOK_KEY, 'lent-person', personId],
    queryFn: () =>
      lentList({
        person_id: personId ?? 0,
        from: null,
        to: null,
        offset: 0,
        limit: 1,
        sort: null,
        group_by_month: false,
        show_finished: false,
      }),
    enabled: personId !== null,
    staleTime: 0,
  });
}

/**
 * The page behind the dashboard tile of garden owners and others, one month at a time, page by page
 * (PRD FR-16.13, plan 76 point 10). The people and the totals come with every page and are the backend's.
 */
/** What the dues page asks about: the filters and the month. The rows and the people have a page each. */
export type LentDuesFilter = Omit<
  LentDuesInput,
  'offset' | 'limit' | 'people_offset' | 'people_limit'
>;

/**
 * The entries of the month, page by page, with the totals over the whole filter. The people of the same answer
 * have their own pages (`useLentDuesPeople`), so one row of people is asked for here and the rest is left out.
 */
export function useLentDues(filter: LentDuesFilter, enabled = true) {
  return usePagedQuery({
    screen: 'cashbook.lent_dues',
    queryKey: [...CASHBOOK_KEY, 'lent-dues', filter],
    enabled,
    fetchPage: (offset, limit) =>
      lentDues({ ...filter, offset, limit, people_offset: 0, people_limit: 1 }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
  });
}

/**
 * The people with something open or an entry in the month, page by page (PRD FR-19.11, Code Standards B31): the
 * backend gives 100 of them a page and says how many there are in all, so none is cut off. One entry row is asked
 * for with each page, as the entries have their own pages.
 */
export function useLentDuesPeople(filter: LentDuesFilter, enabled = true) {
  return usePagedQuery({
    screen: 'cashbook.lent_dues_people',
    queryKey: [...CASHBOOK_KEY, 'lent-dues-people', filter],
    enabled,
    fetchPage: (offset, limit) =>
      lentDues({ ...filter, offset: 0, limit: 1, people_offset: offset, people_limit: limit }),
    rowsIn: (page) => page.people.length,
    totalOf: (page) => page.people_total_count,
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
