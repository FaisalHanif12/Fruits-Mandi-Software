// Development page /dev/dead-amount (plan 40): the screens of the dead amount inside the app shell,
// answered by a stand-in for the backend with the worked numbers of E2E-44
// (tests/helpers/dev-bridge-dead.ts), so they can be opened in a plain browser tab, used, and
// photographed in both languages and at every window size. The flag in the hash chooses the screen:
// none = R38; `?card=21` / `?card=22` = the party card of a buyer who owes / of one in the dead
// amount; `?move` opens the box "Move to dead amount" and `?undo` the box "Make active again" at
// once. Other flags choose the state (`both`, `none`, `down`, `linked`, `notowed`). Never shipped: the
// route exists only when import.meta.env.DEV is true (app/router.tsx).

import { useEffect, useState } from 'react';
import { installDeadDevBridge } from '../../../tests/helpers/dev-bridge-dead';
import { devFlag, devValue } from '../../../tests/helpers/dev-bridge';
import { ARSHAD_DEAD } from '../../../tests/helpers/dead';
import { DeadAmountPage } from '../../features/reports/pages/DeadAmountPage';
import { PartyCard, useDeadAmountActions } from '../../shared/ui';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';
const NAMES = { name_ur: 'ارشد ملتانی', name_en: 'Arshad Multani', tag: null };

/** Opens one of the two boxes straight away, to photograph it. */
function OpenAtOnce() {
  const actions = useDeadAmountActions();
  useEffect(() => {
    if (actions === null) return;
    if (devFlag('move')) actions.move({ accountId: 22, personKey: null, names: NAMES });
    else if (devFlag('undo')) actions.undo({ markId: ARSHAD_DEAD.mark_id, names: NAMES });
  }, [actions]);
  return null;
}

export function DeadAmountDevPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDeadDevBridge();
    return true;
  });
  const card = devValue('card');
  return (
    <AppShell
      nav={<MainNav current="reports" onSelect={() => undefined} />}
      moduleMenu={null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onGuide={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      <OpenAtOnce />
      {card === null ? <DeadAmountPage /> : <PartyCard accountId={Number(card)} />}
    </AppShell>
  );
}
