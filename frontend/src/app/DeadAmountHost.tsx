// Gives the party card, the account form and the dues reports the two dialogs of the dead amount
// (PRD FR-10.10, plan 40): "ڈوبی رقم میں ڈالیں" and "دوبارہ فعال کریں". It holds the one request that
// is open and shows its dialog above whatever screen asked. The dialogs are loaded only when first
// needed. It sits in the root route, next to the Index form host, so every screen has it and the
// screen underneath is never closed.

import { Suspense, lazy, useCallback, useMemo, useRef, useState, type ReactNode } from 'react';
import {
  DeadAmountContext,
  type DeadAmountActions,
  type DeadMoveRequest,
  type DeadUndoRequest,
} from '../shared/ui';

const DeadMarkDialog = lazy(() =>
  import('../features/masters/components/DeadMarkDialog').then((module) => ({
    default: module.DeadMarkDialog,
  })),
);
const DeadUndoDialog = lazy(() =>
  import('../features/masters/components/DeadUndoDialog').then((module) => ({
    default: module.DeadUndoDialog,
  })),
);

type Open = { kind: 'move'; request: DeadMoveRequest } | { kind: 'undo'; request: DeadUndoRequest };

export function DeadAmountHost({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState<Open | null>(null);
  // The request that is open, read when the dialog closes (an event, never while drawing).
  const current = useRef<Open | null>(null);

  const show = useCallback((next: Open) => {
    // One dialog at a time: a second ask while one is open is turned away.
    if (current.current !== null) return;
    current.current = next;
    setOpen(next);
  }, []);

  const actions = useMemo<DeadAmountActions>(
    () => ({
      move: (request) => {
        show({ kind: 'move', request });
      },
      undo: (request) => {
        show({ kind: 'undo', request });
      },
    }),
    [show],
  );

  const close = useCallback((done: boolean) => {
    const was = current.current;
    current.current = null;
    setOpen(null);
    was?.request.onClosed?.(done);
  }, []);

  return (
    <DeadAmountContext.Provider value={actions}>
      {children}
      {open === null ? null : (
        <Suspense fallback={null}>
          {open.kind === 'move' ? (
            <DeadMarkDialog request={open.request} onClose={close} />
          ) : (
            <DeadUndoDialog request={open.request} onClose={close} />
          )}
        </Suspense>
      )}
    </DeadAmountContext.Provider>
  );
}
