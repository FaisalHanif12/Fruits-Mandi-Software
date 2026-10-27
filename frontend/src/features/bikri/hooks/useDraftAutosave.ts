// Keeps the unsaved بکری safe (PRD FR-12.10, NFR-REL-2): every 5 seconds, if something changed,
// the draft stores go to the backend as text. After a crash or a power cut at most the last 5
// seconds are lost. Nothing is sent for an empty draft, and a draft the user emptied by hand is
// thrown away, so an old one is not offered later. A save of the بکری stops the timer and throws
// away a draft that was still on its way (`stop`).

import { useEffect, useRef } from 'react';
import { DRAFT_SAVE_INTERVAL_MS, draftsDiscard, draftsSave } from '../../../shared/drafts';
import { buildDraft, isFilledByUser } from '../draft';
import type { BikriDraftStores } from '../draft-store';
import { BIKRI_DRAFT_SCREEN, serialiseDraft } from '../draft-snapshot';

interface Options {
  /** False on a screen that has no draft (a saved بکری). */
  enabled: boolean;
}

export function useDraftAutosave(stores: BikriDraftStores, { enabled }: Options) {
  const dirty = useRef(false);
  // True once a draft of this screen is on the backend, so emptying the screen can remove it.
  const onBackend = useRef(false);
  // The text last sent, so moving the cursor (which also marks the draft changed) sends nothing.
  const lastSent = useRef<string | null>(null);
  const stopped = useRef(false);
  // Read through a function: `stop` can be called while a draft is on its way, and TypeScript
  // cannot see that a ref changes during an `await`.
  const isStopped = () => stopped.current;

  useEffect(() => {
    if (!enabled) return undefined;
    stopped.current = false;
    const unsubscribe = stores.version.subscribe(() => {
      dirty.current = true;
    });
    const send = async () => {
      if (!dirty.current || isStopped()) return;
      dirty.current = false;
      const filled = isFilledByUser(stores, buildDraft(stores));
      try {
        if (!filled) {
          if (onBackend.current) {
            onBackend.current = false;
            lastSent.current = null;
            await draftsDiscard(BIKRI_DRAFT_SCREEN);
          }
          return;
        }
        const text = serialiseDraft(stores);
        if (text === lastSent.current) return;
        await draftsSave(BIKRI_DRAFT_SCREEN, text);
        lastSent.current = text;
        onBackend.current = true;
        // The بکری was saved while the draft was on its way: it must not stay.
        if (isStopped()) await draftsDiscard(BIKRI_DRAFT_SCREEN);
      } catch {
        // Saving a draft is a safety net. If it fails the next tick tries again.
        dirty.current = true;
      }
    };
    const timer = window.setInterval(() => {
      void send();
    }, DRAFT_SAVE_INTERVAL_MS);
    return () => {
      window.clearInterval(timer);
      unsubscribe();
    };
  }, [stores, enabled]);

  return {
    /** The بکری was saved or thrown away: no draft is sent from now on. */
    stop: () => {
      stopped.current = true;
      dirty.current = false;
      onBackend.current = false;
      lastSent.current = null;
    },
  };
}
