// Gives every picker of the app the forms of "add it here" (plan 41, PRD FR-7.69). A picker asks
// for a kind; this host shows the short form of that kind above whatever screen asked, and, from
// its link "Full form", the full Index form. The forms are loaded only when first needed. It sits
// in the root route, so every screen and every development page has it, and the screen underneath
// is never closed: a sale with its lines, a voucher or a grid keeps every value.
//
// The requests are kept as a stack. A form can ask for another one (the full account form has a
// city picker, which can add a city), and the last one asked is on top. When a form closes, its
// picker is told which record was saved, or that nothing was.
//
// An account (seller, buyer, any) has its short form in the picker itself (shared/ui), so the host
// shows only its full form.

import { Suspense, lazy, useCallback, useEffect, useRef, useState, type ReactNode } from 'react';
import {
  AddRecordContext,
  useWorkAreaStore,
  type AddedRecord,
  type AddRecord,
  type AddRecordKind,
  type AddRecordRequest,
} from '../shared/ui';
import type { MasterKind } from '../bindings/MasterKind';

const IndexAddDialog = lazy(() =>
  import('../features/masters/components/IndexAddDialog').then((module) => ({
    default: module.IndexAddDialog,
  })),
);
const QuickAddMaster = lazy(() =>
  import('../features/masters/components/QuickAddMaster').then((module) => ({
    default: module.QuickAddMaster,
  })),
);

/** One form that is open: the request, and whether it is the short form or the full one. */
interface Entry {
  id: number;
  request: AddRecordRequest;
  form: 'quick' | 'full';
}

/** The master list a kind belongs to, or null for an account (which has no short form here). */
function masterKindOf(kind: AddRecordKind): MasterKind | null {
  return kind === 'seller' || kind === 'buyer' || kind === 'any' ? null : kind;
}

export function AddRecordHost({ children }: { children: ReactNode }) {
  const [stack, setStackState] = useState<Entry[]>([]);
  // The same stack, readable from an event handler without waiting for the next draw.
  const stackRef = useRef<Entry[]>([]);
  const nextId = useRef(0);
  const setStack = useCallback((next: Entry[]) => {
    stackRef.current = next;
    setStackState(next);
  }, []);

  // The main menu marks انڈکس as the current area while the Index form is open (decision D1).
  const setIndexFormOpen = useWorkAreaStore((state) => state.setIndexFormOpen);
  const fullOpen = stack.some((entry) => entry.form === 'full');
  useEffect(() => {
    setIndexFormOpen(fullOpen);
    // The popup of a grid cell is drawn on the page, outside the work area, so while the form is
    // open it is hidden by this mark (grid.css) and shows again, as it was, when the form closes.
    document.body.toggleAttribute('data-index-form', fullOpen);
    return () => {
      setIndexFormOpen(false);
      document.body.removeAttribute('data-index-form');
    };
  }, [fullOpen, setIndexFormOpen]);

  const add = useCallback<AddRecord>(
    (request) => {
      nextId.current += 1;
      const quick = request.form !== 'full' && masterKindOf(request.kind) !== null;
      setStack([
        ...stackRef.current,
        { id: nextId.current, request, form: quick ? 'quick' : 'full' },
      ]);
    },
    [setStack],
  );

  /** A form closed. Its picker hears which record was saved, or null. A second call does nothing. */
  const close = useCallback(
    (id: number, added: AddedRecord | null) => {
      const entry = stackRef.current.find((candidate) => candidate.id === id);
      if (entry === undefined) return;
      setStack(stackRef.current.filter((candidate) => candidate.id !== id));
      entry.request.onClosed(added);
    },
    [setStack],
  );

  /** The short form asked for the full one: same request, with the name typed so far. */
  const toFull = useCallback(
    (id: number, text: string) => {
      setStack(
        stackRef.current.map((entry) =>
          entry.id === id ? { ...entry, form: 'full', request: { ...entry.request, text } } : entry,
        ),
      );
    },
    [setStack],
  );

  return (
    <AddRecordContext.Provider value={add}>
      {children}
      <Suspense fallback={null}>
        {stack.map((entry) => {
          const kind = masterKindOf(entry.request.kind);
          if (entry.form === 'full' || kind === null) {
            return (
              <IndexAddDialog
                key={entry.id}
                request={entry.request}
                onClose={(added) => {
                  close(entry.id, added);
                }}
              />
            );
          }
          return (
            <QuickAddMaster
              key={entry.id}
              open
              onOpenChange={(open) => {
                if (!open) close(entry.id, null);
              }}
              kind={kind}
              typed={entry.request.text}
              owner={entry.request.owner}
              onSaved={(record) => {
                close(entry.id, { id: record.id, code: record.code });
              }}
              // A full form inside a full form would be a maze; the link shows only outside one.
              onFullForm={
                stack.some((other) => other.form === 'full')
                  ? undefined
                  : (text) => {
                      toFull(entry.id, text);
                    }
              }
              onPickExisting={(hit) => {
                close(entry.id, { id: hit.id, code: hit.code });
              }}
            />
          );
        })}
      </Suspense>
    </AddRecordContext.Provider>
  );
}
