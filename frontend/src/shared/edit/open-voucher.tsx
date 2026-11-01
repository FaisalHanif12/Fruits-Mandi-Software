// A saved voucher opens on the screen of its kind (plan 44, PRD FR-7.67, FR-7.70, FR-19.4): an
// expense on the خرچہ screen, a loan on ادھار رقم, a khata entry on الگ کھاتہ, capital on سرمایہ, a
// journal on جنرل, and a receipt, payment or advance on the voucher screen. One place knows the
// addresses, so a row of a list, of a report or of the cash book never writes its own.
// `useOpenVoucher` is the call for a click; `OpenOnItsScreen` is the screen that moves a wrong
// address to the right one.

import { useNavigate, useRouter } from '@tanstack/react-router';
import { useCallback, useEffect } from 'react';
import type { VoucherType } from '../../bindings/VoucherType';
import { LoadingState } from '../ui/LoadingState';

/** What is needed to know where a voucher opens: its id and its type. */
interface Target {
  id: number;
  voucher_type: VoucherType;
}

/** Where a saved voucher is opened: the screen and the address it is asked with. */
type Place =
  | {
      to: '/rokar/expense';
      search: { type: 'shop_expense' | 'personal_expense' | 'personal_return'; id: number };
    }
  | { to: '/rokar/lent'; search: { type: 'money_lent' | 'money_lent_return'; id: number } }
  | { to: '/rokar/khata'; search: { type: 'khata_out' | 'khata_in'; id: number } }
  | { to: '/partners/capital'; search: { type: 'capital_in' | 'capital_out'; id: number } }
  | { to: '/rokar/general'; search: { id: number } }
  | { to: '/bikri/voucher'; search: { type: 'receipt' | 'payment' | 'advance'; id: number } };

/** The one place that knows which screen a voucher type belongs to. */
function placeOf({ id, voucher_type: type }: Target): Place {
  switch (type) {
    case 'shop_expense':
    case 'personal_expense':
    case 'personal_return':
      return { to: '/rokar/expense', search: { type, id } };
    case 'money_lent':
    case 'money_lent_return':
      return { to: '/rokar/lent', search: { type, id } };
    case 'khata_out':
    case 'khata_in':
      return { to: '/rokar/khata', search: { type, id } };
    case 'capital_in':
    case 'capital_out':
      return { to: '/partners/capital', search: { type, id } };
    case 'general':
      return { to: '/rokar/general', search: { id } };
    case 'receipt':
    case 'payment':
    case 'advance':
      return { to: '/bikri/voucher', search: { type, id } };
  }
}

export function useOpenVoucher(): (target: Target, replace?: boolean) => void {
  const navigate = useNavigate();
  return useCallback(
    (target, replace = false) => {
      void navigate({ ...placeOf(target), replace });
    },
    [navigate],
  );
}

/** The address of a saved voucher's screen, for the days a fix reopens (it closes them on leaving). */
export function useVoucherHref(): (target: Target) => string {
  const router = useRouter();
  return useCallback((target) => router.buildLocation(placeOf(target)).href, [router]);
}

export function OpenOnItsScreen({ row }: { row: Target }) {
  const open = useOpenVoucher();
  useEffect(() => {
    open({ id: row.id, voucher_type: row.voucher_type }, true);
  }, [open, row.id, row.voucher_type]);
  return <LoadingState />;
}
