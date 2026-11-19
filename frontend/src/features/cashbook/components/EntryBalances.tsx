// The balance of the account an entry is about, now and after the entry, as the backend works them
// out (PRD FR-7.58 for vouchers; the same preview command serves the ادھار رقم and الگ کھاتہ
// screens). The side is written as a word, never by colour alone (NFR-USE-12). Shown only once the
// backend has answered for an account.

import { Localized } from '@fluent/react';
import { Balance } from '../../../shared/ui';
import type { VoucherPreview } from '../types';

export function EntryBalances({ preview }: { preview: VoucherPreview | undefined }) {
  if (preview?.account === null || preview === undefined) return null;
  return (
    <p className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body">
      <span className="flex items-baseline gap-2">
        <Localized id="cashbook-balance-now" />
        <Balance balance={preview.current_balance} />
      </span>
      <span className="flex items-baseline gap-2">
        <Localized id="cashbook-balance-after" />
        <Balance balance={preview.new_balance} />
      </span>
    </p>
  );
}
