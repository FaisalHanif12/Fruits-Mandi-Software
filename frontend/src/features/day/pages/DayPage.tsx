// دن بند کریں: closing and reopening days (route /day; PRD FR-21.1, FR-21.2). Two cards: close the
// days up to a date, and reopen a date with a reason. The state of the day comes from the backend
// and the status bar follows it. Four states: loading, error with a retry, and the cards.

import { Localized } from '@fluent/react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { ErrorState, LoadingState, Num } from '../../../shared/ui';
import { CloseDayCard } from '../components/CloseDayCard';
import { ReopenDayCard } from '../components/ReopenDayCard';

export function DayPage() {
  const day = useDayStatus();
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="day-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  const { today, closed_up_to, today_open } = day.data;
  return (
    <div className="flex max-w-form flex-col gap-5">
      <div className="flex flex-col gap-1">
        <Localized id="day-title">
          <h1 className="type-title" />
        </Localized>
        <p className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body text-ink-muted">
          <span className="flex items-baseline gap-2">
            <Localized id="day-today" />
            <Num>{isoToDisplayDate(today)}</Num>
          </span>
          <span className="flex items-baseline gap-2">
            <Localized id="day-closed-up-to" />
            {closed_up_to === null ? (
              <Localized id="day-none-closed" />
            ) : (
              <Num>{isoToDisplayDate(closed_up_to)}</Num>
            )}
          </span>
        </p>
      </div>
      <CloseDayCard todayOpen={today_open} />
      <ReopenDayCard closedUpTo={closed_up_to} />
    </div>
  );
}
