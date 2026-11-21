// "This day is closed" on a NEW entry (plan 85, PRD FR-21.4 v1.49, FR-12.6, FR-6.6). A new بکری, voucher,
// خرچہ, پھل, ادھار رقم, الگ کھاتہ, سرمایہ or garden or trip entry whose date is a closed day is refused by the
// backend (`ERR_DATE_CLOSED`), so it has no figures and no save. This is the ONE piece every such screen
// shows instead of a small red word under the date: a red notice that says the day is closed, with the button
// "Reopen the day and save". The button opens the box of the reopen-and-fix flow (the reason and the Owner's
// sign, `useFixDayFlow`); a wrong sign stays inside the box and nothing opens. When the days are open the
// screen asks its answer again, the figures fill, and the user saves as always. The days close again after
// the save (the save flows call `finishFixSession`) or when the user leaves the screen (`FixDayHost`).
//
// Nothing here decides whether a day is closed: the notice appears only when the backend's own answer holds
// `ERR_DATE_CLOSED` on the date of the entry (shared/edit/README). While the days are open for this entry it
// shows the amber line of the days that are open and close again after the save.

import { Localized, useLocalization } from '@fluent/react';
import { useQueryClient } from '@tanstack/react-query';
import { useRouter } from '@tanstack/react-router';
import { isoToDisplayDate } from '../format';
import { Button } from '../ui/Button';
import { Notice } from '../ui/Notice';
import { useFixStore } from './fix-store';
import { daysText } from './useFixDay';
import { useFixDayFlow } from './useFixDayFlow';

/** The date fields of the answers of the entry screens: vouchers `date`, a posting `entry_date`, a بکری `header.entry_date`. */
export const ENTRY_DATE_FIELDS: readonly string[] = ['date', 'entry_date', 'header.entry_date'];

/** What the notice needs of one problem: its code and the field it is about. */
export interface DateProblem {
  code: string;
  field: string | null;
}

/** Does the answer say that the date of the entry is a closed day? Only the backend's answer decides. */
export function dateIsClosed(
  errors: readonly DateProblem[] | null | undefined,
  fields: readonly string[] = ENTRY_DATE_FIELDS,
): boolean {
  return (
    errors?.some(
      (item) =>
        item.code === 'ERR_DATE_CLOSED' && (item.field === null || fields.includes(item.field)),
    ) === true
  );
}

interface ClosedDayEntryNoticeProps {
  /** What the backend refused or found for the entry: the preview's problems and the last save's refusal. */
  errors: readonly DateProblem[] | null | undefined;
  /** The date of the entry as the screen holds it, `DD-MM-YYYY` as typed or `YYYY-MM-DD`. */
  date: string;
  /** The field paths that name the date. Default: the date of every entry screen. */
  fields?: readonly string[];
  /** The days are open: the screen forgets the old refusal and asks its answer again. */
  onReopened?: (() => void) | undefined;
}

export function ClosedDayEntryNotice({
  errors,
  date,
  fields = ENTRY_DATE_FIELDS,
  onReopened,
}: ClosedDayEntryNoticeProps) {
  const { l10n } = useLocalization();
  const router = useRouter();
  const queryClient = useQueryClient();
  const fix = useFixDayFlow();
  const session = useFixStore((state) => state.session);
  const shown = isoToDisplayDate(date);
  // The days are open for this entry: the red notice is over, and the amber line says what happens next.
  const reopenedHere =
    session !== null && date !== '' && session.days.some((day) => isoToDisplayDate(day) === shown);
  const closed = dateIsClosed(errors, fields);

  let body = null;
  if (reopenedHere) {
    body = (
      <Notice tone="warn">
        <Localized id="edit-band-days" vars={{ days: daysText(session.days, l10n) }}>
          <span />
        </Localized>
      </Notice>
    );
  } else if (closed && date !== '') {
    body = (
      <Notice
        tone="error"
        action={
          <Button
            variant="primary"
            busy={fix.busy}
            onClick={() => {
              fix.start({
                date,
                // The screen stays where it is: leaving it closes the days again (FixDayHost).
                ownerHref: router.state.location.href,
                then: () => {
                  // The answers asked while the day was closed (the figures, the refusal) are old now.
                  void queryClient.invalidateQueries();
                  onReopened?.();
                },
              });
            }}
          >
            <Localized id="edit-action-fix-save" />
          </Button>
        }
      >
        <Localized id="edit-closed-entry" vars={{ date: shown }}>
          <span />
        </Localized>
      </Notice>
    );
  }
  return (
    <>
      {body}
      {fix.dialog}
    </>
  );
}
