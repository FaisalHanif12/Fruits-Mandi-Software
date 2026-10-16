// "Save changes?" when the user leaves a screen with unsaved work (PRD FR-7.9, design system 6).
// Three buttons: back (Esc), leave without saving (danger outline), and save (primary, focused).
// Built on Radix AlertDialog like ConfirmDialog: focus stays inside, and goes back to where it
// was when the prompt closes.
//
// The same three-way question is asked in one more place, with its own words: at start-up, "finish
// the last بکری?" with restore, discard and later (plan 13, PRD FR-12.10). The words are Fluent
// ids with the save-changes ones as the default; the layout, the focus and the keys are the same.

import { Localized } from '@fluent/react';
import { AlertDialog } from 'radix-ui';
import { useRef, type ReactNode } from 'react';
import { useUiLanguage } from '../i18n';
import { SHORTCUTS } from '../keyboard';
import { Button } from './Button';

interface SaveChangesPromptProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Save, then go on. */
  onSave: () => void;
  /** Go on without saving. */
  onLeave: () => void;
  /** Other words than "Save changes?", as Fluent ids. Each one has the usual words by default. */
  words?: PromptWords | undefined;
  /** Text under the title, when it needs values the Fluent id cannot get (a time, a number). */
  description?: ReactNode;
  /** Show F10 on the save button. Default: yes. */
  showSaveKey?: boolean;
}

/** The Fluent ids of the words of the prompt. */
export interface PromptWords {
  title: string;
  description: string;
  back: string;
  leave: string;
  save: string;
}

const SAVE_WORDS: PromptWords = {
  title: 'layout-save-title',
  description: 'layout-save-description',
  back: 'common-back',
  leave: 'layout-save-leave',
  save: 'layout-save-save',
};

export function SaveChangesPrompt({
  open,
  onOpenChange,
  onSave,
  onLeave,
  words = SAVE_WORDS,
  description,
  showSaveKey = true,
}: SaveChangesPromptProps) {
  const { language, direction } = useUiLanguage();
  const saveRef = useRef<HTMLButtonElement>(null);
  const returnFocusRef = useRef<HTMLElement | null>(null);
  return (
    <AlertDialog.Root open={open} onOpenChange={onOpenChange}>
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="fixed inset-0 z-40 bg-scrim" />
        <AlertDialog.Content
          lang={language}
          dir={direction}
          onOpenAutoFocus={(event) => {
            event.preventDefault();
            const before = document.activeElement;
            returnFocusRef.current = before instanceof HTMLElement ? before : null;
            saveRef.current?.focus();
          }}
          onCloseAutoFocus={(event) => {
            const target = returnFocusRef.current;
            if (!target?.isConnected) return;
            event.preventDefault();
            target.focus();
          }}
          className="fixed inset-0 z-50 m-auto flex h-fit w-full max-w-dialog flex-col gap-3 rounded-3 bg-surface p-6 text-ink shadow-overlay"
        >
          <AlertDialog.Title className="type-section">
            <Localized id={words.title} />
          </AlertDialog.Title>
          <AlertDialog.Description className="type-body text-ink-muted">
            {description ?? <Localized id={words.description} />}
          </AlertDialog.Description>
          <div className="mt-2 flex flex-wrap justify-end gap-2">
            <Button
              shortcut={SHORTCUTS.back}
              onClick={() => {
                onOpenChange(false);
              }}
            >
              <Localized id={words.back} />
            </Button>
            <Button
              variant="danger"
              onClick={() => {
                onLeave();
                onOpenChange(false);
              }}
            >
              <Localized id={words.leave} />
            </Button>
            <Button
              ref={saveRef}
              variant="primary"
              {...(showSaveKey ? { shortcut: SHORTCUTS.save } : {})}
              onClick={() => {
                onSave();
                onOpenChange(false);
              }}
            >
              <Localized id={words.save} />
            </Button>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
