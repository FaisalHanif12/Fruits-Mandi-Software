// تبدیل کریں and منسوخ کریں for the row that has focus in a list of پچھلی بکری (PRD FR-7.70, FR-21.4, plan
// 72), the same two buttons in the same place as every other list (`EntryActions`): above the table, on
// the right of the heading of the part. What they do comes from the flags on the row, which the backend
// sets; the screen only follows them.
//
// - Edit opens the بکری on the sale book straight in edit mode, with the way back kept in the address
//   (`past`, `pastAll`), so after the save the Owner is on the same window and row. A row of a closed day
//   reads "reopen the day and fix": it asks the reason, reopens the days, opens the edit, and the days
//   close again after the save (FR-21.4). A row that cannot be changed says why, in words.
// - Cancel asks the reason in the shared box and cancels without leaving the list; the list is read
//   again and its totals with it.
// Enter on a row still opens the whole بکری to look at (P10). There is no key for Edit or Cancel in
// any list of the app (PRD 7.3): the two buttons are reached with Tab like the other fields.

import { Localized } from '@fluent/react';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate, useRouter } from '@tanstack/react-router';
import { useState } from 'react';
import type { BikriListRow } from '../../../bindings/BikriListRow';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { NO_ENTRY_FLAGS, useFixDayFlow } from '../../../shared/edit';
import { EntryActions, ErrorNotice, RUNNING_NUMBER } from '../../../shared/ui';
import { bikriGet } from '../api';
import { firstEditablePosting } from '../edit-target';
import { CancelBikriFlow } from './CancelBikriFlow';
import { keepPastReturn } from '../hooks/usePastReturn';
import type { PastReturn } from '../past-window';

interface PastRowActionsProps {
  /** The بکری that has focus in this part of the list, or null. */
  row: BikriListRow | null;
  /** The window and the switch the list is on: the way back after an edit. */
  back: PastReturn;
}

export function PastRowActions({ row, back }: PastRowActionsProps) {
  const navigate = useNavigate();
  const router = useRouter();
  const queryClient = useQueryClient();
  const fix = useFixDayFlow();
  const [opening, setOpening] = useState(false);
  const [failed, setFailed] = useState<UiError | null>(null);
  const [cancelling, setCancelling] = useState<{ id: number; number: number } | null>(null);

  // The entry that Edit opens is a posting of the بکری, not the بکری: it is read when Edit is pressed,
  // so what it opens is what the backend says now (a posting may have been undone since the list was read).
  const edit = () => {
    if (row === null) return;
    setOpening(true);
    setFailed(null);
    queryClient
      .query({
        queryKey: ['bikri', 'detail', row.id],
        queryFn: () => bikriGet(row.id),
        staleTime: 0,
      })
      .then((detail) => {
        const posting = firstEditablePosting(detail);
        if (posting === undefined) return;
        const search = { id: row.id, edit: posting.entry_id, ...keepPastReturn(back) };
        const open = () => {
          void navigate({ to: '/bikri/book', search });
        };
        if (row.needs_reopen) {
          fix.start({
            date: posting.entry_date,
            ownerHref: router.buildLocation({ to: '/bikri/book', search }).href,
            then: open,
          });
        } else {
          open();
        }
      })
      .catch((failure: unknown) => {
        setFailed(uiErrorOf(failure));
      })
      .finally(() => {
        setOpening(false);
      });
  };

  return (
    <div className="flex flex-col items-end gap-1" data-past-row-actions="">
      <div className="flex flex-wrap items-center justify-end gap-x-3 gap-y-2">
        <span className="type-body text-ink-muted">
          {row === null ? (
            <Localized id="bikri-past-row-none" />
          ) : (
            <Localized
              id="bikri-past-row-for"
              vars={{ number: String(row.number) }}
              elems={RUNNING_NUMBER}
            >
              <span />
            </Localized>
          )}
        </span>
        <EntryActions
          flags={row ?? NO_ENTRY_FLAGS}
          busy={opening || fix.busy}
          onEdit={edit}
          onCancel={() => {
            if (row !== null) setCancelling({ id: row.id, number: row.number });
          }}
        />
      </div>
      {failed === null ? null : <ErrorNotice error={failed} />}
      {fix.dialog}
      <CancelBikriFlow
        target={cancelling}
        onClose={() => {
          setCancelling(null);
        }}
        onCancelled={() => {
          setCancelling(null);
        }}
      />
    </div>
  );
}
