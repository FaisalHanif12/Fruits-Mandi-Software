// The voucher commands as hooks (Code Standards F6). A saved voucher refreshes everything that
// shows what it wrote (PRD 6.2): the party cards, the open bills and payables, the lists and the
// chart, so every screen shows the new balance.

import {
  keepPreviousData,
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query';
import { useDebouncedValue } from '../../shared/hooks';
import {
  assistAdvances,
  assistCollectionList,
  assistOpenBills,
  assistOpenPayables,
  vouchersCancel,
  vouchersEdit,
  vouchersGet,
  vouchersList,
  vouchersPreview,
  vouchersSave,
  vouchersSaveReceiptsBatch,
} from './api';
import { pukhtaMakeRegular } from '../pukhta_khata';
import type { BuiltVoucher } from './draft';
import type { CollectionListInput } from '../../bindings/CollectionListInput';
import type { MakeRegularInput } from '../../bindings/MakeRegularInput';
import type { VouchersListInput } from '../../bindings/VouchersListInput';
import type { ReceiptInput, VoucherEntry, VoucherPreview } from './types';

export const VOUCHERS_KEY = ['vouchers'] as const;
/** The preview waits this long after the last key press, and the one before it is dropped (F35). */
const PREVIEW_DELAY_MS = 50;

/** Everything a saved voucher changes, to read again (PRD 6.2). */
const READ_AGAIN: readonly (readonly string[])[] = [
  VOUCHERS_KEY,
  ['accounts'],
  ['coa'],
  ['partners'],
  ['search'],
  ['pukhta'],
  ['bikri'],
  ['day_status'],
];

/** A seller's open advances and last advance (PRD FR-7.60). Never kept: they change with a save. */
export function useAdvances(sellerId: number | null) {
  return useQuery({
    queryKey: [...VOUCHERS_KEY, 'advances', sellerId],
    queryFn: () => assistAdvances(sellerId ?? 0),
    enabled: sellerId !== null,
    staleTime: 0,
  });
}

/** The open bills of a buyer or of a one-time person, oldest first (PRD FR-7.58). */
export function useOpenBills(accountId: number | null, personKey: string | null) {
  return useQuery({
    queryKey: [...VOUCHERS_KEY, 'open-bills', accountId, personKey],
    queryFn: () => assistOpenBills({ account_id: accountId ?? 0, person_key: personKey }),
    enabled: accountId !== null,
    staleTime: 0,
  });
}

/** The payable postings of a seller or of a one-time seller, oldest first (PRD FR-7.59). */
export function useOpenPayables(sellerId: number | null, personKey: string | null) {
  return useQuery({
    queryKey: [...VOUCHERS_KEY, 'open-payables', sellerId, personKey],
    queryFn: () =>
      assistOpenPayables({
        seller_id: sellerId ?? 0,
        person_key: personKey,
        include_settled: false,
      }),
    enabled: sellerId !== null,
    staleTime: 0,
  });
}

/**
 * The preview of the entry on the form: the balance before and after, the bills it settles, the
 * warnings and the cash after. Asked 50 ms after the last key, only once there is an account and
 * an amount, so an empty form asks nothing. `replacing` is the voucher an edit takes back.
 */
export function useVoucherPreview(entry: VoucherEntry | null, replacing: number | null) {
  const settled = useDebouncedValue(entry, PREVIEW_DELAY_MS);
  const query = useQuery({
    queryKey: [...VOUCHERS_KEY, 'preview', settled, replacing],
    // The answer carries the type it was asked for, so an older answer kept on screen while a new
    // one is on its way is never taken for the answer of another type of voucher.
    queryFn: async (): Promise<TaggedPreview> => {
      if (settled === null) throw new Error('no entry');
      return { type: settled.type, preview: await vouchersPreview(settled, replacing) };
    },
    enabled: settled !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
  // The answer is for what is on screen only when the entry has stayed as it was asked about.
  const current = settled === entry && !query.isPlaceholderData && query.isSuccess;
  return { settled, query, current };
}

/**
 * The preview of each voucher waiting in the list (PRD FR-15.1): its amount and the balance after,
 * as the backend works them out now. A row is asked once, and again after any save, because a save
 * changes cash and balances (the same preview as the form's, Code Standards B5).
 */
export function useRowPreviews(
  rows: readonly { id: string; entry: VoucherEntry | null }[],
): ReadonlyMap<string, VoucherPreview> {
  const answers = useQueries({
    queries: rows.map(({ entry }) => ({
      queryKey: [...VOUCHERS_KEY, 'row-preview', entry],
      queryFn: () => {
        if (entry === null) throw new Error('no entry');
        return vouchersPreview(entry, null);
      },
      enabled: entry !== null,
      staleTime: 0,
      gcTime: 0,
    })),
  });
  const previews = new Map<string, VoucherPreview>();
  rows.forEach(({ id }, index) => {
    const data = answers[index]?.data;
    if (data !== undefined) previews.set(id, data);
  });
  return previews;
}

/** A preview and the type of voucher it is the answer for. */
interface TaggedPreview {
  type: VoucherEntry['type'];
  preview: VoucherPreview;
}

/** One saved voucher, with its version and whether it can still be edited. */
export function useVoucher(id: number | null) {
  return useQuery({
    queryKey: [...VOUCHERS_KEY, 'one', id],
    queryFn: () => vouchersGet(id ?? 0),
    enabled: id !== null,
    staleTime: 0,
    gcTime: 0,
  });
}

/** The vouchers of a kind in a date range (the list under the form). */
export function useVouchersList(input: VouchersListInput) {
  return useQuery({
    queryKey: [...VOUCHERS_KEY, 'list', input],
    queryFn: () => vouchersList(input),
    placeholderData: keepPreviousData,
    staleTime: 0,
  });
}

/** The اگراہی فہرست of a day (PRD FR-15.6). */
export function useCollectionList(input: CollectionListInput | null) {
  return useQuery({
    queryKey: [...VOUCHERS_KEY, 'collection', input],
    queryFn: () => {
      if (input === null) throw new Error('no list');
      return assistCollectionList(input);
    },
    enabled: input !== null,
    staleTime: 0,
    gcTime: 0,
  });
}

function useReadAgain() {
  const queryClient = useQueryClient();
  return () =>
    Promise.all(
      READ_AGAIN.map((queryKey) => queryClient.invalidateQueries({ queryKey: [...queryKey] })),
    );
}

export function useSaveVoucher() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (built: BuiltVoucher) => vouchersSave(built),
    onSuccess: readAgain,
  });
}

export function useEditVoucher() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (input: Parameters<typeof vouchersEdit>[0]) => vouchersEdit(input),
    onSuccess: readAgain,
  });
}

export function useCancelVoucher() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (input: Parameters<typeof vouchersCancel>[0]) => vouchersCancel(input),
    onSuccess: readAgain,
  });
}

export function useSaveReceiptsBatch() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (rows: readonly ReceiptInput[]) => vouchersSaveReceiptsBatch(rows),
    onSuccess: readAgain,
  });
}

/**
 * Makes a one-time person a regular account (PRD FR-13.8). The backend makes the account and moves
 * his balance in one action, so every list that shows accounts, balances, bills or the people of the
 * پختہ کھاتہ is read again.
 */
export function useMakeRegular() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (input: MakeRegularInput) => pukhtaMakeRegular(input),
    onSuccess: readAgain,
  });
}
