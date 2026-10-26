// Gives every picker of the app the Index form for its "add" button (plan 32, R8). It holds the
// one request that is open and shows the Index form above whatever screen asked. The form is
// loaded only when it is first needed. It sits in the root route, so every screen and every
// development page has it, and the screen underneath is never closed.

import { Suspense, lazy, useCallback, useEffect, useRef, useState, type ReactNode } from 'react';
import {
  AddRecordContext,
  useWorkAreaStore,
  type AddRecord,
  type AddRecordRequest,
} from '../shared/ui';

const IndexAddDialog = lazy(() =>
  import('../features/masters/components/IndexAddDialog').then((module) => ({
    default: module.IndexAddDialog,
  })),
);

export function AddRecordHost({ children }: { children: ReactNode }) {
  const [request, setRequest] = useState<AddRecordRequest | null>(null);
  // The main menu marks انڈکس as the current area while the Index form is open (decision D1).
  const setIndexFormOpen = useWorkAreaStore((state) => state.setIndexFormOpen);
  useEffect(() => {
    setIndexFormOpen(request !== null);
    // The popup of a grid cell is drawn on the page, outside the work area, so while the form is
    // open it is hidden by this mark (grid.css) and shows again, as it was, when the form closes.
    document.body.toggleAttribute('data-index-form', request !== null);
    return () => {
      setIndexFormOpen(false);
      document.body.removeAttribute('data-index-form');
    };
  }, [request, setIndexFormOpen]);
  // The request that is open, read when the form closes (an event, never while drawing).
  const open = useRef<AddRecordRequest | null>(null);

  const add = useCallback<AddRecord>((next) => {
    // One Index form at a time: a second ask while one is open is turned away, and its picker is
    // told at once so it does not wait for a form that will not come.
    if (open.current !== null) {
      next.onClosed(false);
      return;
    }
    open.current = next;
    setRequest(next);
  }, []);

  const close = useCallback((saved: boolean) => {
    const done = open.current;
    open.current = null;
    setRequest(null);
    done?.onClosed(saved);
  }, []);

  return (
    <AddRecordContext.Provider value={add}>
      {children}
      {request === null ? null : (
        <Suspense fallback={null}>
          <IndexAddDialog request={request} onClose={close} />
        </Suspense>
      )}
    </AddRecordContext.Provider>
  );
}
