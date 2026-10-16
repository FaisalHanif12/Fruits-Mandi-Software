// The row above the sale book (PRD FR-12.13, FR-12.11): first, previous, next and last بکری, find,
// and at the end the actions of the screen (Save on a new بکری, New on a saved one). The same row
// is on the new sale book and on a saved one opened read-only, so the user never feels he is on
// another screen (PRD FR-7.67 in spirit, design system 8 item 14).
//
// - The four arrows go by بکری number (`bikri_navigate`). From the new, unsaved بکری "previous" is
//   the last saved one and "next" is the first. At the end of the list a short note says so.
// - Moving off a new بکری that has typed work asks first: the leave guard of the screen stops it
//   (PRD FR-7.9), so these buttons never lose anything.
// - The arrows that point a way are mirrored in Urdu (Code Standards F26).

import { useLocalization } from '@fluent/react';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Search } from 'lucide-react';
import { useEffect, useState, type ReactNode } from 'react';
import type { NavigateTo } from '../../../bindings/NavigateTo';
import { uiErrorOf } from '../../../shared/api';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Button, ErrorNotice } from '../../../shared/ui';
import { bikriNavigate } from '../api';
import { BikriFindDialog } from './BikriFindDialog';
import type { UiError } from '../../../bindings/UiError';

/** How long "there is no previous بکری" stays on screen. */
const NOTE_MS = 4_000;

interface BikriToolbarProps {
  /** The saved بکری on screen, or null on a new one. */
  currentId: number | null;
  /** Open this saved بکری. */
  onOpen: (id: number) => void;
  /** The find box is open (F3 opens it; the screen owns the key so a dialog can disable others). */
  findOpen: boolean;
  onFindOpenChange: (open: boolean) => void;
  /** The actions of the screen, at the end of the row. */
  children?: ReactNode;
  /** What was saved last, with a way to open it. */
  lastSaved?: ReactNode;
}

const STEPS: readonly {
  to: NavigateTo;
  labelId: string;
  noneId: string;
  icon: typeof ChevronLeft;
}[] = [
  { to: 'first', labelId: 'bikri-nav-first', noneId: 'bikri-nav-none', icon: ChevronsLeft },
  {
    to: 'previous',
    labelId: 'bikri-nav-previous',
    noneId: 'bikri-nav-no-previous',
    icon: ChevronLeft,
  },
  { to: 'next', labelId: 'bikri-nav-next', noneId: 'bikri-nav-no-next', icon: ChevronRight },
  { to: 'last', labelId: 'bikri-nav-last', noneId: 'bikri-nav-none', icon: ChevronsRight },
];

export function BikriToolbar({
  currentId,
  onOpen,
  findOpen,
  onFindOpenChange,
  children,
  lastSaved,
}: BikriToolbarProps) {
  const { l10n } = useLocalization();
  const [noteId, setNoteId] = useState<string | null>(null);
  const [failure, setFailure] = useState<UiError | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (noteId === null) return undefined;
    const timer = window.setTimeout(() => {
      setNoteId(null);
    }, NOTE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [noteId]);

  const go = async (to: NavigateTo, noneId: string) => {
    setNoteId(null);
    setFailure(null);
    setBusy(true);
    try {
      const id = await bikriNavigate(currentId, to);
      if (id === null) setNoteId(noneId);
      else onOpen(id);
    } catch (error: unknown) {
      setFailure(uiErrorOf(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-2">
        <div role="group" aria-label={l10n.getString('bikri-nav-label')} className="flex gap-2">
          {STEPS.map(({ to, labelId, noneId, icon: Icon }) => (
            <Button
              key={to}
              busy={busy}
              onClick={() => {
                void go(to, noneId);
              }}
            >
              <Icon aria-hidden="true" className="size-icon-md rtl:rotate-180" />
              <span className="sr-only">{l10n.getString(labelId)}</span>
            </Button>
          ))}
        </div>
        <Button
          shortcut={SHORTCUTS.find}
          onClick={() => {
            onFindOpenChange(true);
          }}
        >
          <Search aria-hidden="true" className="size-icon-md" />
          {l10n.getString('bikri-find-open')}
        </Button>
        {lastSaved}
        <div className="ms-auto flex flex-wrap items-center gap-2">{children}</div>
      </div>
      {noteId === null ? null : (
        <p role="status" className="type-body text-ink-muted">
          {l10n.getString(noteId)}
        </p>
      )}
      {failure === null ? null : <ErrorNotice error={failure} />}
      <BikriFindDialog open={findOpen} onOpenChange={onFindOpenChange} onOpen={onOpen} />
    </div>
  );
}
