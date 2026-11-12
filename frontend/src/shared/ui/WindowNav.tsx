// The two buttons that move a window of days, under a list (PRD 12.7 and FR-12.47, plans 72 and 74): the
// earlier window on the start side and the later one on the end side, each with its chevron (turned
// round in right-to-left, F26) and its key on the button (Page Up and Page Down, FR-7.4). In the middle
// goes what the screen says about the window (its days, a button to the latest one). When the window is
// already the latest, the later button is off and says why under its words. With focus on the bar the
// arrow keys move the window the way the text reads (FR-7.13). The words and the days are the screen's;
// there is no money and no date work here. Page Up and Page Down themselves are the screen's shortcut.

import { useId, type ReactNode } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { elementDirection, readingStep, type Shortcut } from '../keyboard';
import { Button } from './Button';

interface WindowNavProps {
  /** The name of the bar for a screen reader. */
  label: string;
  /** The words of the earlier button, and under them the days it goes to. */
  earlier: ReactNode;
  earlierDetail?: ReactNode;
  earlierShortcut?: Shortcut;
  onEarlier: () => void;
  /** The words of the later button, and under them the days it goes to. */
  later: ReactNode;
  laterDetail?: ReactNode;
  laterShortcut?: Shortcut;
  onLater: () => void;
  /** Set when the window is the latest: the later button is off and these words say why. */
  laterOffNote?: ReactNode;
  /** The days of the window and a way back to the latest one, one under the other, between the buttons. */
  middle?: ReactNode;
}

export function WindowNav({
  label,
  earlier,
  earlierDetail,
  earlierShortcut,
  onEarlier,
  later,
  laterDetail,
  laterShortcut,
  onLater,
  laterOffNote,
  middle,
}: WindowNavProps) {
  const off = laterOffNote !== undefined;
  const reasonId = useId();
  return (
    <div
      role="group"
      aria-label={label}
      onKeyDown={(event) => {
        if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return;
        const step = readingStep(event.key, elementDirection(event.currentTarget));
        if (step === null) return;
        event.preventDefault();
        if (step === 'toStart') onEarlier();
        else if (!off) onLater();
      }}
      className="flex shrink-0 flex-wrap items-center justify-between gap-x-4 gap-y-2 border-t border-border pt-3"
    >
      <Button
        {...(earlierShortcut === undefined ? {} : { shortcut: earlierShortcut })}
        {...(earlierDetail === undefined ? {} : { detail: earlierDetail })}
        onClick={onEarlier}
      >
        <ChevronLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
        {earlier}
      </Button>
      {/* At the compact step the two buttons fill the row; the days of the window go on a line of their own under them. */}
      {middle === undefined ? null : (
        <div className="order-last flex basis-full flex-col items-center justify-center gap-1 normal:order-none normal:basis-auto">
          {middle}
        </div>
      )}
      <div className="flex flex-col items-end gap-1">
        <Button
          {...(laterShortcut === undefined ? {} : { shortcut: laterShortcut })}
          disabled={off}
          {...(off
            ? { describedBy: reasonId }
            : laterDetail === undefined
              ? {}
              : { detail: laterDetail })}
          onClick={onLater}
        >
          {later}
          <ChevronRight aria-hidden="true" className="size-icon-md rtl:rotate-180" />
        </Button>
        {/* Why it is off, in plain text at full strength: the dimmed button must not dim its reason. */}
        {off ? (
          <p id={reasonId} className="type-chip text-ink-muted">
            {laterOffNote}
          </p>
        ) : null}
      </div>
    </div>
  );
}
