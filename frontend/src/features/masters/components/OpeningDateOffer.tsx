// "This day is closed" under the opening date of an account (plan 44, plan 53, PRD FR-10.11). A NEW
// account saves its opening on any day, even a closed one; only a CHANGED opening of an existing account
// needs an open date. When that date is closed the form says so and offers one way out, never a dead end
// and never "enter a date after X" (X may be today, and no later date is allowed):
// - today is open: "use today's date", and also "reopen the day": using today's date does not help
//   when the OLD opening of the account is on the closed day (its entry must be reversed on that day), and
//   the form cannot tell which of the two dates is the closed one;
// - today is closed too: "reopen the day" (PRD FR-21.4). Clearing the opening needs the open day too,
//   so there is no "save without the opening".
// A new account whose opening goes through روکڑ on a closed day (PRD FR-10.9, FR-21.4) says so in its own
// words, with the "reopen the day and save" button of the other entry screens.
// The backend's answer carries `today` and `today_open`; this reads them and offers.
// The refusal comes from the save, and the opening date sits low in a long form, so the offer scrolls
// itself into view when it appears: the Owner never sees a red date with the reason out of sight.

import { Localized } from '@fluent/react';
import { useEffect, useRef } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { Button, Notice } from '../../../shared/ui';

/** The refusal of a closed opening date, or null when the error is something else. */
export function closedOpeningDate(
  error: UiError | null,
): { today: string; todayOpen: boolean } | null {
  const item = error?.errors.find(
    (candidate) => candidate.code === 'ERR_DATE_CLOSED' && candidate.field === 'opening_date',
  );
  if (item === undefined) return null;
  return {
    today: String(item.values['today'] ?? ''),
    todayOpen: Number(item.values['today_open']) === 1,
  };
}

interface OpeningDateOfferProps {
  /** A new account: its opening was never saved, so the words say it is to be written. */
  isNew?: boolean;
  offer: { today: string; todayOpen: boolean };
  onUseToday: (today: string) => void;
  onReopen: () => void;
}

/** The words of the offer, by whether today is open and whether the account is new. */
function offerMessageId(todayOpen: boolean, isNew: boolean): string {
  const base = isNew ? 'accounts-opening-closed-new' : 'accounts-opening-closed';
  return `${base}-${todayOpen ? 'today-open' : 'today-closed'}`;
}

export function OpeningDateOffer({
  offer,
  onUseToday,
  onReopen,
  isNew = false,
}: OpeningDateOfferProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const root = rootRef.current;
    if (root !== null && typeof root.scrollIntoView === 'function') {
      root.scrollIntoView({ block: 'nearest' });
    }
  }, []);
  return (
    <div ref={rootRef}>
      <Notice
        tone="warn"
        action={
          <span className="flex flex-wrap items-center gap-2">
            {offer.todayOpen ? (
              <Button
                onClick={() => {
                  onUseToday(offer.today);
                }}
              >
                <Localized id="accounts-opening-closed-use-today" vars={{ today: offer.today }} />
              </Button>
            ) : null}
            <Button onClick={onReopen}>
              <Localized id={isNew ? 'edit-action-fix-save' : 'edit-action-fix-day'} />
            </Button>
          </span>
        }
      >
        <Localized id={offerMessageId(offer.todayOpen, isNew)} />
      </Notice>
    </div>
  );
}
