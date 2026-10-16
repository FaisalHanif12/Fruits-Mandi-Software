// The Fluent message of a rule level (PRD 11.3). The six levels come from the backend by name.

import type { RuleLevel } from '../../bindings/RuleLevel';

/** `party_item` is `rules-level-party-item`: Fluent IDs are kebab-case. */
export function levelMessageId(level: RuleLevel): string {
  return `rules-level-${level.replaceAll('_', '-')}`;
}
