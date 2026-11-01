// The shell's part of "reopen and fix" (plan 44, PRD FR-21.4). It does two things.
//
// 1. When the user leaves the screen where he fixed an entry (without saving, or by any other way),
//    the days that were reopened for the fix close again. A save closes them itself, at once.
// 2. When a day does not close again (a check of FR-21.1 fails), it tells which day and why, and
//    the day stays open until it is put right. Nothing is hidden.

import { Localized, useLocalization } from '@fluent/react';
import { useRouterState } from '@tanstack/react-router';
import { useEffect } from 'react';
import { useFixStore } from '../shared/edit/fix-store';
import { finishFixSession } from '../shared/edit/useFixDay';
import { isoToDisplayDate } from '../shared/format';
import { Button, ErrorMessage, FormDialog } from '../shared/ui';

export function FixDayHost() {
  const { l10n } = useLocalization();
  const href = useRouterState({ select: (state) => state.location.href });
  const session = useFixStore((state) => state.session);
  const stayedOpen = useFixStore((state) => state.stayedOpen);
  const setStayedOpen = useFixStore((state) => state.setStayedOpen);

  useEffect(() => {
    if (session === null) return;
    if (href === session.ownerHref) {
      useFixStore.getState().markEntered();
      return;
    }
    // The user has been at the screen of the fix and is somewhere else now.
    if (session.entered) void finishFixSession(l10n);
  }, [href, session, l10n]);

  return (
    <FormDialog
      open={stayedOpen.length > 0}
      onOpenChange={(open) => {
        if (!open) setStayedOpen([]);
      }}
      title={<Localized id="edit-fix-stayed-title" />}
    >
      <div className="flex flex-col gap-3" role="alert">
        {stayedOpen.map((day) => (
          <section key={day.date} className="flex flex-col gap-1">
            <Localized id="edit-fix-stayed-text" vars={{ date: isoToDisplayDate(day.date) }}>
              <p className="type-body font-semibold text-ink" />
            </Localized>
            {day.errors.map((error) => (
              <ErrorMessage key={`${day.date}-${error.code}`} item={error}>
                <p className="type-body text-due" />
              </ErrorMessage>
            ))}
          </section>
        ))}
        <div className="flex justify-end">
          <Button
            variant="primary"
            onClick={() => {
              setStayedOpen([]);
            }}
          >
            <Localized id="edit-fix-stayed-ok" />
          </Button>
        </div>
      </div>
    </FormDialog>
  );
}
