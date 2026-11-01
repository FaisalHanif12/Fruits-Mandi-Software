// The shell's part of the partner code (plan 44, PRD FR-17.11). It does two things.
//
// 1. It shows the code box when a locked card or a refused report asks for it (one box for the app).
// 2. It closes the section again when the user moves to a screen of another group (the partner
//    section, the dashboard, a report and the analytics are groups, see partner-areas.ts). The idle
//    lock, a sign-out and closing the app close it in the backend.

import { Localized } from '@fluent/react';
import { useRouterState } from '@tanstack/react-router';
import { useEffect, useRef } from 'react';
import { FormDialog, usePartnerCodeDialog } from '../../../shared/ui';
import { useLockPartners, usePartnerCodeStatus } from '../code-hooks';
import { partnerAreaOf } from '../partner-areas';
import { PartnerCodeFlow } from './PartnerCodeFlow';

export function PartnerCodeHost() {
  const open = usePartnerCodeDialog((state) => state.open);
  const hide = usePartnerCodeDialog((state) => state.hide);
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const status = usePartnerCodeStatus();
  const { mutate: lockNow } = useLockPartners();
  const unlocked = status.data?.unlocked === true;
  // The group the section was opened in; a move to another group closes it.
  const openedIn = useRef<string | null>(null);

  useEffect(() => {
    const area = partnerAreaOf(pathname);
    if (!unlocked) {
      openedIn.current = null;
      return;
    }
    if (openedIn.current === null) {
      openedIn.current = area;
      return;
    }
    if (openedIn.current !== area) {
      openedIn.current = null;
      lockNow();
    }
  }, [pathname, unlocked, lockNow]);

  return (
    <FormDialog
      open={open}
      onOpenChange={(next) => {
        if (!next) hide();
      }}
      title={<Localized id="partners-code-title" />}
    >
      <PartnerCodeFlow onOpened={hide} />
    </FormDialog>
  );
}
