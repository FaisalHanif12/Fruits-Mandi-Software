// The three kinds of voucher that share one screen (PRD FR-7.67, FR-15.10), and the address of that
// screen. Kept apart from the screen so the router can read the address without loading the screen
// (Code Standards F36).

import type { VoucherType } from '../../bindings/VoucherType';

/** The types in the type list of the voucher screen, in the order of the list. */
export type ScreenType = 'receipt' | 'payment' | 'advance';

export const SCREEN_TYPES: readonly ScreenType[] = ['receipt', 'payment', 'advance'];

/** The type a value from an address or a search result means, or null for any other value. */
export function asScreenType(value: unknown): ScreenType | null {
  return SCREEN_TYPES.find((type) => type === value) ?? null;
}

/** The voucher screen's type of a voucher, or null when the voucher belongs to another screen. */
export function screenTypeOf(type: VoucherType): ScreenType | null {
  return asScreenType(type);
}

/**
 * The address of the voucher screen: `/bikri/voucher?type=receipt` starts a new voucher of a type;
 * `&id=12` opens a saved voucher (edit while its day is open, else read only).
 */
export interface VoucherSearch {
  type?: ScreenType;
  id?: number;
}

export function parseVoucherSearch(search: Record<string, unknown>): VoucherSearch {
  const type = asScreenType(search['type']);
  const id = Number(search['id']);
  return {
    ...(type === null ? {} : { type }),
    ...(Number.isSafeInteger(id) && id > 0 ? { id } : {}),
  };
}
