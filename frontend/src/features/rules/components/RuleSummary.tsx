// One rule in a line of words: the side and charge, the party, the item or group, and the value
// with its method, for example "Seller Commission, Rahim, Banana, 5 percent". Used where a rule is
// mentioned (an overlap warning, the winner of "Test a rule"). The value is the backend's text.

import { useLocalization } from '@fluent/react';
import { displayName, useLanguage } from '../../../shared/i18n';
import type { RuleMethod, RuleRow } from '../types';

const VALUE_ID: Record<RuleMethod, string> = {
  percent: 'rules-value-percent',
  per_unit: 'rules-value-per-unit',
  per_lot: 'rules-value-per-lot',
};

export function RuleSummary({ rule }: { rule: RuleRow }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  return (
    <span>
      {l10n.getString('rules-summary', {
        side: l10n.getString(`rules-side-${rule.side}`),
        charge: l10n.getString(`rules-charge-${rule.charge_type}`),
        party:
          rule.account === null
            ? l10n.getString('rules-any-party')
            : displayName(rule.account, language),
        item:
          rule.item !== null
            ? displayName(rule.item, language)
            : rule.item_group !== null
              ? l10n.getString(`rules-group-${rule.item_group}`)
              : l10n.getString('rules-any-item'),
        value: l10n.getString(VALUE_ID[rule.method], { value: rule.value_text }),
      })}
    </span>
  );
}
