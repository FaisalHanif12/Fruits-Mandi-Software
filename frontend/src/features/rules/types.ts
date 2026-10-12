// Types of the commission rules. What the backend owns comes from the generated bindings (Code
// Standards F2); the names below are the names the screens use. What the user types is text
// (CLAUDE.md rule 1): the rule form holds a `RuleDraft`, and "Test a rule" sends text.

import type { ChargeMethod } from '../../bindings/ChargeMethod';
import type { ChargeSide } from '../../bindings/ChargeSide';
import type { ChargeTest } from '../../bindings/ChargeTest';
import type { ChargeType } from '../../bindings/ChargeType';
import type { ItemGroup } from '../../bindings/ItemGroup';
import type { RecordRef } from '../../bindings/RecordRef';
import type { RuleLevel } from '../../bindings/RuleLevel';
import type { RuleOverlap } from '../../bindings/RuleOverlap';
import type { RuleRow } from '../../bindings/RuleRow';
import type { RuleSaved } from '../../bindings/RuleSaved';
import type { RulesFilter } from '../../bindings/RulesFilter';
import type { RuleTest } from '../../bindings/RuleTest';
import type { SkippedLevel } from '../../bindings/SkippedLevel';
import type { WinnerView } from '../../bindings/WinnerView';

export type {
  ChargeMethod,
  ChargeSide,
  ChargeTest,
  ChargeType,
  ItemGroup,
  RuleLevel,
  RuleOverlap,
  RuleRow,
  RuleSaved,
  RuleTest,
  SkippedLevel,
  WinnerView,
};

/** Who the charge is on (PRD 11.1): the seller's side is deducted, the buyer's is added to his bill. */
export type RuleSide = ChargeSide;
export type RuleMethod = ChargeMethod;
export type RuleFilter = RulesFilter;

/** What the rule form holds, all as text (CLAUDE.md rule 1). */
export interface RuleDraft {
  id: number | null;
  version: number | null;
  side: RuleSide;
  charge_type: ChargeType;
  account: RecordRef | null;
  item: RecordRef | null;
  item_group: ItemGroup | '';
  method: RuleMethod;
  value: string;
  /** `DD-MM-YYYY` as typed. */
  from_date: string;
  to_date: string;
  is_active: boolean;
  note: string;
}
