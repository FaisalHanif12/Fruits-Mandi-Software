// "This day is closed" (PRD FR-12.18, FR-12.22, FR-6.6): an edit or a cancel of a posting on a
// closed day is refused until the Owner reopens the day (FR-21.2). The note names the day and
// points the way: a button to the day screen, where the reopen is done (D2). Nothing is hidden:
// the screen still shows what it can, and the save stays off.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { isoToDisplayDate } from '../../../shared/format';
import { LinkButton, Notice } from '../../../shared/ui';

export function ClosedDayNotice({ date }: { date: string }) {
  const navigate = useNavigate();
  return (
    <Notice
      tone="error"
      action={
        <LinkButton
          onClick={() => {
            void navigate({ to: '/day' });
          }}
        >
          <Localized id="bikri-closed-day-open" />
        </LinkButton>
      }
    >
      <Localized id="bikri-closed-day" vars={{ date: isoToDisplayDate(date) }}>
        <span />
      </Localized>
    </Notice>
  );
}
