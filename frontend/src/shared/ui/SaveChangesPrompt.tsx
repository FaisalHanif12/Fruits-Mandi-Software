// "Save changes?" when the user leaves a screen with unsaved work (PRD FR-7.9, design system 6).
// Three buttons: back (Esc), leave without saving (danger outline), and save (primary, focused).
// Built on Radix AlertDialog like ConfirmDialog: focus stays inside, and goes back to where it
// was when the prompt closes.

import { Localized } from '@fluent/react';
import { AlertDialog } from 'radix-ui';
import { useRef } from 'react';
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
}

export function SaveChangesPrompt({ open, onOpenChange, onSave, onLeave }: SaveChangesPromptProps) {
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
            <Localized id="layout-save-title" />
          </AlertDialog.Title>
          <AlertDialog.Description className="type-body text-ink-muted">
            <Localized id="layout-save-description" />
          </AlertDialog.Description>
          <div className="mt-2 flex flex-wrap justify-end gap-2">
            <Button
              shortcut={SHORTCUTS.back}
              onClick={() => {
                onOpenChange(false);
              }}
            >
              <Localized id="common-back" />
            </Button>
            <Button
              variant="danger"
              onClick={() => {
                onLeave();
                onOpenChange(false);
              }}
            >
              <Localized id="layout-save-leave" />
            </Button>
            <Button
              ref={saveRef}
              variant="primary"
              shortcut={SHORTCUTS.save}
              onClick={() => {
                onSave();
                onOpenChange(false);
              }}
            >
              <Localized id="layout-save-save" />
            </Button>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
