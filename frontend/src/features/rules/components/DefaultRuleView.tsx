// One of the three global default rules (seller commission, buyer commission, لاگا on fruit). It
// is changed only in Settings, "Charges", so the rule form never edits or switches it off: this
// view says what the rule is and takes the user to Settings.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { isoToDisplayDate } from '../../../shared/format';
import { SHORTCUTS, useEscape } from '../../../shared/keyboard';
import { Button, Card } from '../../../shared/ui';
import type { RuleRow } from '../types';
import { RuleSummary } from './RuleSummary';

export function DefaultRuleView({ rule }: { rule: RuleRow }) {
  const navigate = useNavigate();
  const goBack = () => {
    void navigate({ to: '/index/rules' });
  };
  useEscape('leave', goBack);
  return (
    <div className="flex max-w-form flex-col gap-5">
      <Localized id="rules-default-title">
        <h1 className="type-title" />
      </Localized>
      <Card>
        <p className="type-body font-semibold">
          <RuleSummary rule={rule} />
        </p>
        <p className="type-body text-ink-muted">{isoToDisplayDate(rule.from_date)}</p>
        <Localized id="rules-default-notice">
          <p className="type-body" />
        </Localized>
        <div className="flex flex-wrap gap-2">
          <Button shortcut={SHORTCUTS.back} onClick={goBack}>
            <Localized id="common-back" />
          </Button>
          <Button
            variant="primary"
            onClick={() => {
              void navigate({ to: '/settings', search: { tab: 'charges' } });
            }}
          >
            <Localized id="rules-default-open-settings" />
          </Button>
        </div>
      </Card>
    </div>
  );
}
