// Types of the charge and override settings (PRD 4.2, Section 21), from the generated bindings
// (Code Standards F2). The defaults are rules; what the Charges tab sends is text.

import type { ChargeMethod } from '../../bindings/ChargeMethod';
import type { ChargesInput } from '../../bindings/ChargesInput';
import type { ChargesSettings } from '../../bindings/ChargesSettings';
import type { CreditLimitMode } from '../../bindings/CreditLimitMode';
import type { DefaultCharge } from '../../bindings/DefaultCharge';
import type { OverridesSettings } from '../../bindings/OverridesSettings';

export type {
  ChargeMethod,
  ChargesInput,
  ChargesSettings,
  CreditLimitMode,
  DefaultCharge,
  OverridesSettings,
};

/** The three defaults of the Charges tab, in the order the tab shows them. */
export const DEFAULT_KEYS = ['seller_commission', 'buyer_commission', 'laga_fruit'] as const;
export type DefaultKey = (typeof DEFAULT_KEYS)[number];

/** One default as the form holds it: how it is worked out and the value as text. */
export interface DefaultDraft {
  method: ChargeMethod;
  value: string;
}

export type ChargesDraft = Record<DefaultKey, DefaultDraft>;
