// The partner setup (route /partners/setup; PRD 17.1, FR-17.1, FR-17.2): the partners, a share
// box each, the day the shares start, the running total and Save. It is the same form as the
// first-run wizard's partners step. Esc goes back to the overview (asking first when there are
// unsaved shares).

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { SHORTCUTS, useEscape } from '../../../shared/keyboard';
import { Button, InfoHint } from '../../../shared/ui';
import { PartnerSetupForm } from '../components/PartnerSetupForm';

export function PartnerSetupPage() {
  const navigate = useNavigate();
  const goBack = () => {
    void navigate({ to: '/partners' });
  };
  useEscape('leave', goBack);
  return (
    <div className="flex max-w-form flex-col gap-3 short:gap-2">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-1">
          <Localized id="partners-setup-title">
            <h1 className="type-title" />
          </Localized>
          <InfoHint>
            <Localized id="partners-setup-help">
              <p />
            </Localized>
          </InfoHint>
        </div>
        <Button shortcut={SHORTCUTS.back} onClick={goBack}>
          <Localized id="common-back" />
        </Button>
      </div>
      <PartnerSetupForm labelId="partners-save" leaveGuard onSaved={() => undefined} />
    </div>
  );
}
