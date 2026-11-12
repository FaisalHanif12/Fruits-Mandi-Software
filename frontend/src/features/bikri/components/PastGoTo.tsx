// "تاریخ پر جائیں" of پچھلی بکری (PRD 12.7, plan 72 P6): the shared date box with this screen's words.
// The Owner types a day and presses Enter or leaves the box: the window of 7 days that ends on that day
// opens. A day that is not real, or after today, is refused in words next to the box (`GoToDate`).

import { Localized, useLocalization } from '@fluent/react';
import { GoToDate } from '../../../shared/ui';

interface PastGoToProps {
  /** The backend's today, `YYYY-MM-DD`, or null until it is known. */
  today: string | null;
  /** The day to go to, `YYYY-MM-DD`, never after today. */
  onGo: (day: string) => void;
}

export function PastGoTo({ today, onGo }: PastGoToProps) {
  const { l10n } = useLocalization();
  return (
    <GoToDate
      today={today}
      onGo={onGo}
      label={<Localized id="bikri-past-goto" />}
      info={<Localized id="bikri-past-goto-hint" />}
      placeholder={l10n.getString('bikri-past-goto-placeholder')}
    />
  );
}
