// What a saved voucher needs besides its own row to be opened for an edit (PRD FR-15.3): the voucher
// holds its collector only by id, and the kind of its account (a seller or a buyer) is not on it.
// The form needs the name and the kind, so they are read once, together.
// Nothing here works out money.

import { useQuery } from '@tanstack/react-query';
import { accountsGet, pickedOf } from '../../shared/accounts';
import { findMaster, type SearchScope } from '../../shared/search';
import type { PickerValue } from '../../shared/ui';
import type { RowExtras } from './draft';
import { VOUCHERS_KEY } from './hooks';
import type { VoucherRow } from './types';

/** The record a voucher points at by id, as the picker holds it. */
async function pickerValueOf(scope: SearchScope, id: number | null): Promise<PickerValue | null> {
  if (id === null) return null;
  const found = await findMaster(scope, id);
  // A master that was switched off since is no longer offered, but the voucher still points at it.
  return found === null ? { id, name_ur: String(id), name_en: String(id) } : pickedOf(found);
}

export function useEditSeed(row: VoucherRow | null) {
  return useQuery({
    queryKey: [...VOUCHERS_KEY, 'seed', row?.id, row?.version],
    queryFn: async (): Promise<RowExtras> => {
      if (row === null) throw new Error('no voucher');
      const [account, collector] = await Promise.all([
        row.account === null ? Promise.resolve(null) : accountsGet(row.account.id),
        pickerValueOf('staff', row.collector_staff_id),
      ]);
      return { accountType: account?.account_type ?? null, collector };
    },
    enabled: row !== null,
    staleTime: 0,
    gcTime: 0,
  });
}
