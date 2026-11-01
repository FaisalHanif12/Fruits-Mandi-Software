// "This day is closed" under the opening date of an account (plan 44, PRD FR-10.11). An account
// saves on any day; only an opening balance posts, and it needs an open date. When the date is
// closed the form says so and offers a way out, never a dead end:
// - today is open: "use today's date";
// - today is closed too (it is night and the days are closed): "save without the opening", or
//   "reopen the day and fix" (PRD FR-21.4).
// The backend's answer carries `today` and `today_open`; this reads them and offers.

import { Localized } from '@fluent/react';
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
  offer: { today: string; todayOpen: boolean };
  onUseToday: (today: string) => void;
  onSaveWithout: () => void;
  onReopen: () => void;
}

export function OpeningDateOffer({
  offer,
  onUseToday,
  onSaveWithout,
  onReopen,
}: OpeningDateOfferProps) {
  return (
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
          ) : (
            <>
              <Button onClick={onSaveWithout}>
                <Localized id="accounts-opening-closed-without" />
              </Button>
              <Button onClick={onReopen}>
                <Localized id="edit-action-fix-day" />
              </Button>
            </>
          )}
        </span>
      }
    >
      <Localized
        id={
          offer.todayOpen
            ? 'accounts-opening-closed-today-open'
            : 'accounts-opening-closed-today-closed'
        }
      />
    </Notice>
  );
}
