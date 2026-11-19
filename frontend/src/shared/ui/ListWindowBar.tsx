// The bar under a list that shows one window at a time (PRD FR-19.11, plan 76 point 8): a box to go to a day (or a
// month), and the buttons that move the window, with the days or the month of each. It is the one bar of every
// windowed list: the 7-day lists (the sales registers, the cash book, the voucher lists, the audit log) and the
// 1-month lists (the expense, fruit, خرچہ and جنرل lists and the party and partner statements). It works
// on a `ListWindow` (`useListWindow`) and adds nothing up; there is no money here. Page Up and Page Down are the
// list's own (`onPageKey` of `PagedList`, which the window gives).

import { Localized, useLocalization } from '@fluent/react';
import type { ListWindow } from '../hooks/useListWindow';
import { DAYS_WINDOW, daysWindowOf, isoToDisplayDate, stepDaysWindow } from '../format';
import { SHORTCUTS } from '../keyboard';
import { Button } from './Button';
import { GoToDate } from './GoToDate';
import { GoToMonth } from './GoToMonth';
import { MonthNav } from './MonthNav';
import { RUNNING_NUMBER } from './Num';
import { WindowNav } from './WindowNav';

/** The days of a window as words under a button: "07-11-2026 to 13-11-2026". */
function DaysRange({ end }: { end: string }) {
  const days = daysWindowOf(end);
  return (
    <Localized
      id="common-days-range"
      vars={{ from: isoToDisplayDate(days.from), to: isoToDisplayDate(days.to) }}
      elems={RUNNING_NUMBER}
    >
      <span />
    </Localized>
  );
}

interface ListWindowBarProps {
  window: ListWindow;
}

export function ListWindowBar({ window }: ListWindowBarProps) {
  const { l10n } = useLocalization();
  if (!window.ready || window.today === null) return null;
  const { today } = window;
  if (window.kind === 'month') {
    if (window.month === null) return null;
    return (
      <div className="flex flex-col gap-3">
        <GoToMonth today={today} first={window.firstMonth} onGo={window.goTo} />
        <MonthNav
          month={window.month}
          today={today}
          first={window.firstMonth}
          onMove={window.goTo}
        />
      </div>
    );
  }
  const end = window.to;
  return (
    <div className="flex flex-col gap-3">
      <GoToDate
        today={today}
        onGo={window.goTo}
        label={<Localized id="common-goto-day" />}
        info={<Localized id="common-goto-day-hint" />}
        placeholder={l10n.getString('common-goto-day-placeholder')}
      />
      <WindowNav
        label={l10n.getString('common-days-nav-label')}
        earlier={
          <Localized
            id="common-days-earlier"
            vars={{ days: String(DAYS_WINDOW) }}
            elems={RUNNING_NUMBER}
          >
            <span />
          </Localized>
        }
        earlierShortcut={SHORTCUTS.pastEarlier}
        earlierDetail={<DaysRange end={stepDaysWindow(end, -1, today, window.first)} />}
        onEarlier={() => {
          window.move(-1);
        }}
        {...(window.canGoBack ? {} : { earlierOffNote: <Localized id="common-days-first-note" /> })}
        later={
          <Localized
            id="common-days-later"
            vars={{ days: String(DAYS_WINDOW) }}
            elems={RUNNING_NUMBER}
          >
            <span />
          </Localized>
        }
        laterShortcut={SHORTCUTS.pastLater}
        laterDetail={<DaysRange end={stepDaysWindow(end, 1, today, window.first)} />}
        onLater={() => {
          window.move(1);
        }}
        {...(window.isLatest ? { laterOffNote: <Localized id="common-days-latest-note" /> } : {})}
        middle={
          <>
            <span className="type-body text-ink">
              <DaysRange end={end} />
            </span>
            {window.isLatest ? null : (
              <Button variant="ghost" onClick={window.latest}>
                <Localized id="common-days-latest-button" />
              </Button>
            )}
          </>
        }
      />
    </div>
  );
}
