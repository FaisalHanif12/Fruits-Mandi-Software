// The buttons that move a window of one month, under a list or a statement (PRD FR-9.3, FR-17.12,
// FR-19.11, plan 76): "previous month" on the start side, "next month" on the end side, the month
// between them and, when the window is not the current month, a button back to it. It is the
// `WindowNav` of the day windows with the words of months: the earlier button goes off at the first
// month with entries, the later one at the month of today, each saying why. Page Up and Page Down are
// the screen's own shortcut (`SHORTCUTS.pastEarlier` and `pastLater`, the same keys as the day
// windows). The months come from `shared/format/months.ts`; there is no money here.

import { Localized, useLocalization } from '@fluent/react';
import { addMonths, canMonthComeNearer, canMonthGoBack, monthParts, windowMonth } from '../format';
import { SHORTCUTS } from '../keyboard';
import { Button } from './Button';
import { RUNNING_NUMBER } from './Num';
import { WindowNav } from './WindowNav';

interface MonthNavProps {
  /** The month shown, `YYYY-MM`. */
  month: string;
  /** Today, `YYYY-MM-DD`, as the backend says it: the window never passes its month. */
  today: string;
  /** The first month with entries, `YYYY-MM`, or null while it is not known. */
  first: string | null;
  onMove: (month: string) => void;
}

/** The month as words, "October 2026": the month word from the report words, the year as digits. */
export function MonthTitle({ month }: { month: string }) {
  const { l10n } = useLocalization();
  const parts = monthParts(month);
  if (parts === null) return <span>{month}</span>;
  return (
    <Localized
      id="common-month-title"
      vars={{
        month: l10n.getString(`report-month-${String(parts.month)}`),
        year: String(parts.year),
      }}
      elems={RUNNING_NUMBER}
    >
      <span />
    </Localized>
  );
}

export function MonthNav({ month, today, first, onMove }: MonthNavProps) {
  const { l10n } = useLocalization();
  const canNearer = canMonthComeNearer(month, today);
  const canBack = canMonthGoBack(month, first);
  return (
    <WindowNav
      label={l10n.getString('common-month-nav-label')}
      earlier={<Localized id="common-month-earlier" />}
      earlierShortcut={SHORTCUTS.pastEarlier}
      earlierDetail={<MonthTitle month={windowMonth(addMonths(month, -1), today, first)} />}
      onEarlier={() => {
        onMove(windowMonth(addMonths(month, -1), today, first));
      }}
      {...(canBack ? {} : { earlierOffNote: <Localized id="common-month-first-note" /> })}
      later={<Localized id="common-month-later" />}
      laterShortcut={SHORTCUTS.pastLater}
      laterDetail={<MonthTitle month={windowMonth(addMonths(month, 1), today, first)} />}
      onLater={() => {
        onMove(windowMonth(addMonths(month, 1), today, first));
      }}
      {...(canNearer ? {} : { laterOffNote: <Localized id="common-month-latest-note" /> })}
      middle={
        <>
          <span className="type-body text-ink">
            <MonthTitle month={month} />
          </span>
          {canNearer ? (
            <Button
              variant="ghost"
              onClick={() => {
                onMove(windowMonth(null, today, first));
              }}
            >
              <Localized id="common-month-latest-button" />
            </Button>
          ) : null}
        </>
      }
    />
  );
}
