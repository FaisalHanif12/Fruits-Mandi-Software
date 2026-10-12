// One rule that overlaps the rule just saved (PRD FR-11.5), as a line: what it is (the same
// words as everywhere a rule is named), the dates it covers, and a link to open it. The backend
// sends the id and the dates; the rest is read with `rules_get`.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { isoToDisplayDate } from '../../../shared/format';
import { LinkButton } from '../../../shared/ui';
import { useRule } from '../hooks';
import type { RuleOverlap } from '../types';
import { RuleSummary } from './RuleSummary';

export function OverlapItem({ overlap }: { overlap: RuleOverlap }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const rule = useRule(overlap.id);
  return (
    <span className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
      {rule.data === undefined ? null : <RuleSummary rule={rule.data} />}
      <span className="text-ink-muted">
        {overlap.to_date === null
          ? l10n.getString('rules-overlap-dates-open', {
              from: isoToDisplayDate(overlap.from_date),
            })
          : l10n.getString('rules-overlap-dates', {
              from: isoToDisplayDate(overlap.from_date),
              to: isoToDisplayDate(overlap.to_date),
            })}
      </span>
      <LinkButton
        onClick={() => {
          void navigate({ to: '/index/rules/$id', params: { id: String(overlap.id) } });
        }}
      >
        <Localized id="rules-overlap-open" />
      </LinkButton>
    </span>
  );
}
