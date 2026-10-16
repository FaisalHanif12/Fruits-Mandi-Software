// Typed wrappers around the rules commands (plan 09; module `rules`, Tech Stack 5.2). Components
// never call invoke() (Code Standards F4). The only file that knows the wire shape.

import type { RuleTestInput } from '../../bindings/RuleTestInput';
import { callCommand } from '../../shared/api';
import type { RuleDraft, RuleFilter, RuleRow, RuleSaved, RuleTest } from './types';

export function rulesList(filter: RuleFilter): Promise<RuleRow[]> {
  return callCommand<RuleRow[]>('rules_list', { input: { filter } });
}

/** One rule by id, for the form that edits it. */
export function rulesGet(id: number): Promise<RuleRow> {
  return callCommand<RuleRow>('rules_get', { input: { id } });
}

/**
 * Creates (no id) or edits (with id and the version that was read) one rule. The value and the
 * dates go as typed; the backend checks every field and the combination of PRD 11.2 again. The
 * answer lists the rules that overlap this one at the same level (PRD FR-11.5).
 */
export function rulesSave(draft: RuleDraft): Promise<RuleSaved> {
  return callCommand<RuleSaved>('rules_save', {
    input: {
      id: draft.id,
      version: draft.version,
      side: draft.side,
      charge_type: draft.charge_type,
      account_id: draft.account?.id ?? null,
      item_id: draft.item?.id ?? null,
      item_group: draft.item_group === '' ? null : draft.item_group,
      method: draft.method,
      value: draft.value,
      from_date: draft.from_date,
      to_date: draft.to_date,
      is_active: draft.is_active,
      note: draft.note,
    },
  });
}

/** Switches a rule off or on. A rule is never deleted. */
export function rulesSetActive(id: number, isActive: boolean): Promise<RuleSaved> {
  return callCommand<RuleSaved>('rules_set_active', { input: { id, is_active: isActive } });
}

/** "Test a rule" (PRD FR-11.8): which rule wins for each charge, and the amount. Saves nothing. */
export function rulesTest(input: RuleTestInput): Promise<RuleTest> {
  return callCommand<RuleTest>('rules_test', { input });
}
