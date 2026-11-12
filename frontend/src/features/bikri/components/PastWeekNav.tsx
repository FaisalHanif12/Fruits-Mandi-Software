// The buttons that move the window of پچھلی بکری by 7 days, at the END of the page, after the table and
// its page buttons (PRD 12.7, plan 72; Umar 12-11-2026): "پچھلے 7 دن" on the start side and "اگلے 7 دن" on
// the end side, each with its chevron (turned round in right-to-left, F26), its key on the button (Page Up
// and Page Down, FR-7.4) and, under its words, the days it goes to, for example "26-09-2026 تا
// 02-10-2026". When the window already ends yesterday the later button is off and says why under its
// words. In the middle, when the window is not the latest, a small button jumps back to the window that
// ends yesterday. With focus on the bar, the arrow keys move the window the way the text reads (FR-7.13).
// Page Up and Page Down are handled by the list (`shared/keyboard`), as before. The days of each button are
// worked out from the window by the date helpers of `past-window.ts`; there is no money here.

import { Localized, useLocalization } from '@fluent/react';
import { useId } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { isoToDisplayDate } from '../../../shared/format';
import { SHORTCUTS, elementDirection, readingStep } from '../../../shared/keyboard';
import { Button, RUNNING_NUMBER } from '../../../shared/ui';
import { WINDOW_DAYS, canComeNearer, stepWindowEnd, windowOf } from '../past-window';

interface PastWeekNavProps {
  /** The last day of the window shown, `YYYY-MM-DD`. */
  end: string;
  /** Today, `YYYY-MM-DD`, as the backend says it: the window never passes the day before. */
  today: string;
  onEarlier: () => void;
  onNearer: () => void;
  /** Jumps to the window that ends yesterday. */
  onLatest: () => void;
}

/** The days a button goes to, as the words under it. */
function Range({ end }: { end: string }) {
  const days = windowOf(end);
  return (
    <Localized
      id="bikri-past-range"
      vars={{ from: isoToDisplayDate(days.from), to: isoToDisplayDate(days.to) }}
      elems={RUNNING_NUMBER}
    >
      <span />
    </Localized>
  );
}

export function PastWeekNav({ end, today, onEarlier, onNearer, onLatest }: PastWeekNavProps) {
  const { l10n } = useLocalization();
  const latest = !canComeNearer(end, today);
  const reasonId = useId();
  return (
    <div
      role="group"
      aria-label={l10n.getString('bikri-past-nav-label')}
      onKeyDown={(event) => {
        if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return;
        const step = readingStep(event.key, elementDirection(event.currentTarget));
        if (step === null) return;
        event.preventDefault();
        if (step === 'toStart') onEarlier();
        else if (!latest) onNearer();
      }}
      className="flex shrink-0 flex-wrap items-center justify-between gap-x-4 gap-y-2 border-t border-border pt-3"
    >
      <Button
        shortcut={SHORTCUTS.pastEarlier}
        detail={<Range end={stepWindowEnd(end, -1, today)} />}
        onClick={onEarlier}
      >
        <ChevronLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
        <Localized id="bikri-past-back" vars={{ days: String(WINDOW_DAYS) }} elems={RUNNING_NUMBER}>
          <span />
        </Localized>
      </Button>
      {latest ? null : (
        <Button variant="ghost" onClick={onLatest}>
          <Localized id="bikri-past-nav-latest-button" />
        </Button>
      )}
      <div className="flex flex-col items-end gap-1">
        <Button
          shortcut={SHORTCUTS.pastLater}
          disabled={latest}
          {...(latest
            ? { describedBy: reasonId }
            : { detail: <Range end={stepWindowEnd(end, 1, today)} /> })}
          onClick={onNearer}
        >
          <Localized
            id="bikri-past-forward"
            vars={{ days: String(WINDOW_DAYS) }}
            elems={RUNNING_NUMBER}
          >
            <span />
          </Localized>
          <ChevronRight aria-hidden="true" className="size-icon-md rtl:rotate-180" />
        </Button>
        {/* Why it is off, in plain text at full strength: the dimmed button must not dim its reason. */}
        {latest ? (
          <p id={reasonId} className="type-chip text-ink-muted">
            <Localized id="bikri-past-nav-latest" />
          </p>
        ) : null}
      </div>
    </div>
  );
}
