// What the commission rule list remembers for the Owner (PRD FR-7.45): the side, the charge, the
// party, the item and whether inactive rules show, kept as text under the screen name
// `list.rules`. A value that cannot be read any more is left out. Nothing about money is kept.

import type { PickerValue } from '../../shared/ui';
import { choiceOr, flagOf, isRecord, pickedOrNull, pickedText } from '../../shared/screen-state';
import { ALL_CHARGES, SIDES } from './allowed';
import type { ChargeType, RuleSide } from './types';

export const RULES_SCREEN = 'list.rules';

const SIDE_CHOICES: readonly (RuleSide | 'all')[] = ['all', ...SIDES];
const CHARGE_CHOICES: readonly (ChargeType | 'all')[] = ['all', ...ALL_CHARGES];

export interface RuleChoices {
  side: RuleSide | 'all';
  charge: ChargeType | 'all';
  party: PickerValue | null;
  item: PickerValue | null;
  showInactive: boolean;
}

export const NO_RULE_CHOICES: RuleChoices = {
  side: 'all',
  charge: 'all',
  party: null,
  item: null,
  showInactive: false,
};

export function ruleChoicesFrom(raw: unknown): RuleChoices {
  if (!isRecord(raw)) return NO_RULE_CHOICES;
  return {
    side: choiceOr(raw['side'], SIDE_CHOICES, 'all'),
    charge: choiceOr(raw['charge'], CHARGE_CHOICES, 'all'),
    party: pickedOrNull(raw['party']),
    item: pickedOrNull(raw['item']),
    showInactive: flagOf(raw['showInactive']),
  };
}

export function ruleChoicesText(choices: RuleChoices): Record<string, unknown> | null {
  const kept: Record<string, unknown> = {};
  if (choices.side !== 'all') kept['side'] = choices.side;
  if (choices.charge !== 'all') kept['charge'] = choices.charge;
  if (choices.party !== null) kept['party'] = pickedText(choices.party);
  if (choices.item !== null) kept['item'] = pickedText(choices.item);
  if (choices.showInactive) kept['showInactive'] = true;
  return Object.keys(kept).length === 0 ? null : kept;
}
