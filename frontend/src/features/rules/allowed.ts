// Which charges and methods a rule may have (PRD 11.2). The rule form offers only these; the
// backend refuses any other (`ERR_RULE_NOT_ALLOWED`) and is the final word. This is a table of the
// business rules of the PRD, not money maths.

import type { ChargeType, RuleMethod, RuleSide } from './types';

export const SIDES: readonly RuleSide[] = ['seller', 'buyer'];

/** The charges of each side, in the order of the PRD table, with the methods each allows. */
export const ALLOWED: Record<
  RuleSide,
  readonly { charge: ChargeType; methods: readonly RuleMethod[] }[]
> = {
  seller: [
    { charge: 'commission', methods: ['percent', 'per_unit', 'per_lot'] },
    { charge: 'labour', methods: ['per_unit', 'per_lot'] },
    { charge: 'munshiana', methods: ['per_unit', 'per_lot'] },
    { charge: 'phone', methods: ['per_unit', 'per_lot'] },
  ],
  buyer: [
    { charge: 'commission', methods: ['percent', 'per_unit'] },
    { charge: 'laga', methods: ['per_unit', 'percent'] },
  ],
};

/** The charges a side may have. */
export function chargesOf(side: RuleSide): ChargeType[] {
  return ALLOWED[side].map((entry) => entry.charge);
}

/** The methods a charge may have on a side. Empty when the combination does not exist. */
export function methodsOf(side: RuleSide, charge: ChargeType): readonly RuleMethod[] {
  return ALLOWED[side].find((entry) => entry.charge === charge)?.methods ?? [];
}

/** Every charge that exists on any side, in a stable order, for a filter. */
export const ALL_CHARGES: readonly ChargeType[] = [
  'commission',
  'laga',
  'labour',
  'munshiana',
  'phone',
];
