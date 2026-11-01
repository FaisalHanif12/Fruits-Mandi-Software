// "Edited: 31-10-26 4:35 pm, Owner" (plan 44, PRD FR-7.71): the line in the header of an entry that
// was changed, with the reason when one was given, how many times it was changed, and a button for
// the edit history. The time and the name are the backend's, read from the audit log; nothing is
// typed for the stamp and nothing here can change it.

import { Localized, useLocalization, type ReactLocalization } from '@fluent/react';
import { History } from 'lucide-react';
import { useState } from 'react';
import type { EditEntity } from '../../bindings/EditEntity';
import type { EditStamp } from '../../bindings/EditStamp';
import { clockPartsOf } from '../edit/time';
import { isoToDisplayDate } from '../format';
import { Button } from './Button';
import { EditHistoryDialog } from './EditHistoryDialog';

/** The clock of a stamp in words: "4:35 pm". The backend's local time is shown as it is on a bad shape. */
export function stampTimeText(localTime: string, l10n: ReactLocalization): string {
  const clock = clockPartsOf(localTime);
  return clock === null
    ? localTime
    : l10n.getString('edit-time', {
        hour: clock.hour,
        minute: clock.minute,
        part: clock.part,
        meridian: clock.meridian,
      });
}

/** The words of a stamp for a tooltip or a list mark: "Edited: 31-10-2026 4:35 pm, Owner". */
export function stampTipText(stamp: EditStamp, l10n: ReactLocalization): string {
  return l10n.getString('edit-mark-tip', {
    date: isoToDisplayDate(stamp.local_date),
    time: stampTimeText(stamp.local_time, l10n),
    by: stamp.by,
  });
}

interface EditStampLineProps {
  stamp: EditStamp;
  /** What the history is asked for. Without them there is no history button. */
  history?: { entity: EditEntity; id: number };
}

export function EditStampLine({ stamp, history }: EditStampLineProps) {
  const { l10n } = useLocalization();
  const [open, setOpen] = useState(false);
  return (
    <div
      className="flex flex-wrap items-center gap-x-3 gap-y-1 type-body text-ink"
      data-edit-stamp=""
    >
      <Localized
        id="edit-stamp"
        vars={{
          date: isoToDisplayDate(stamp.local_date),
          time: stampTimeText(stamp.local_time, l10n),
          by: stamp.by,
        }}
      >
        <span className="font-semibold" />
      </Localized>
      {stamp.count > 1 ? (
        <Localized id="edit-stamp-count" vars={{ count: String(stamp.count) }}>
          <span />
        </Localized>
      ) : null}
      {stamp.reason === null || stamp.reason === '' ? null : (
        <Localized id="edit-stamp-reason" vars={{ reason: stamp.reason }}>
          <span />
        </Localized>
      )}
      {history === undefined ? null : (
        <>
          <Button
            onClick={() => {
              setOpen(true);
            }}
          >
            <History aria-hidden="true" className="size-icon-md" />
            <Localized id="edit-action-history" />
          </Button>
          <EditHistoryDialog
            open={open}
            onOpenChange={setOpen}
            entity={history.entity}
            id={history.id}
          />
        </>
      )}
    </div>
  );
}
