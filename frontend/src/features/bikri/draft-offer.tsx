// "Finish the last بکری?" at start-up (PRD FR-12.10, NFR-REL-2, E2E-17). If the app was closed or
// lost power while a بکری was being typed, the draft the screen kept is offered back: restore opens
// the sale book with it, discard throws it away, and Esc ("not now") keeps it for the next start.
// A draft that cannot be read is thrown away quietly, because half of it must never be used.
//
// The decision is made once for each sign-in. Restore hands the draft to the sale book through
// `usePendingRestore`; the screen takes it when it opens (or when it is open and still empty).

import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { create } from 'zustand';
import { utcToLocalDisplay } from '../../shared/format';
import { draftsDiscard, draftsGet } from '../../shared/drafts';
import { SaveChangesPrompt, type PromptWords } from '../../shared/ui';
import { BIKRI_DRAFT_SCREEN, parseSnapshot, type DraftSnapshot } from './draft-snapshot';
import { Localized } from '@fluent/react';

interface RestoreState {
  /** A draft the user chose to restore, waiting for the sale book to open with it. */
  snapshot: DraftSnapshot | null;
  /**
   * Goes up with every draft offered. The sale book is made new for each value (its React key),
   * so it opens on the draft even when it was already open.
   */
  tick: number;
  offer: (snapshot: DraftSnapshot) => void;
  /** The sale book opened on the draft: forget it. */
  clear: () => void;
}

export const usePendingRestore = create<RestoreState>()((set) => ({
  snapshot: null,
  tick: 0,
  offer: (snapshot) => {
    set((state) => ({ snapshot, tick: state.tick + 1 }));
  },
  clear: () => {
    set({ snapshot: null });
  },
}));

const WORDS: PromptWords = {
  title: 'bikri-draft-title',
  description: 'bikri-draft-text',
  back: 'bikri-draft-later',
  leave: 'bikri-draft-discard',
  save: 'bikri-draft-restore',
};

const DRAFT_KEY = ['drafts', BIKRI_DRAFT_SCREEN] as const;

export function BikriDraftOffer() {
  const navigate = useNavigate();
  const [decided, setDecided] = useState(false);
  const query = useQuery({
    queryKey: DRAFT_KEY,
    queryFn: () => draftsGet(BIKRI_DRAFT_SCREEN),
    // Asked once for each sign-in: a draft saved while the screen is open is not "left over".
    staleTime: Infinity,
    retry: false,
    refetchOnWindowFocus: false,
  });
  const row = query.data ?? null;
  const snapshot = row === null ? null : parseSnapshot(row.payload_json);
  const unreadable = row !== null && snapshot === null;

  useEffect(() => {
    if (!unreadable) return;
    void draftsDiscard(BIKRI_DRAFT_SCREEN).catch(() => null);
  }, [unreadable]);

  if (row === null || snapshot === null) return null;
  return (
    <SaveChangesPrompt
      open={!decided}
      onOpenChange={(open) => {
        // Esc or "not now": the draft stays and is offered again at the next start.
        if (!open) setDecided(true);
      }}
      words={WORDS}
      showSaveKey={false}
      description={
        <Localized id="bikri-draft-text" vars={{ time: utcToLocalDisplay(row.saved_at) }}>
          <span />
        </Localized>
      }
      onSave={() => {
        setDecided(true);
        usePendingRestore.getState().offer(snapshot);
        void navigate({ to: '/bikri/book', search: {} });
      }}
      onLeave={() => {
        setDecided(true);
        void draftsDiscard(BIKRI_DRAFT_SCREEN).catch(() => null);
      }}
    />
  );
}
