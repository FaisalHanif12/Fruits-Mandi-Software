// The box that asks before an entry takes cash in hand below zero (PRD FR-6.7, FR-7.8). One piece
// for every save that can do it: a بکری with نقد (plan 13), a پیشگی, an ادائیگی, a خرچہ and the
// rest (plans 15 to 17). It says what cash will be and asks "is this entry correct?".
//
// - Two buttons: "ہاں، محفوظ کریں" (Yes, save) and "نہیں، واپس جائیں" (No, go back).
// - Focus starts on No, so pressing Enter out of habit never saves a mistake. Esc is No.
// - The screen decides when to open it (the backend says `needs_confirm`) and what saving means.
//   The amount is the cash after the entry from the backend, shown with formatPaisa; nothing is
//   worked out here (CLAUDE.md rule 1).
// - `onCloseFocus` lets the screen put focus on the amount field after No (PRD FR-6.7).

import { Localized } from '@fluent/react';
import { AlertDialog } from 'radix-ui';
import { useRef } from 'react';
import { formatPaisa, useGrouping, type Paisa } from '../format';
import { useUiLanguage } from '../i18n';
import { SHORTCUTS } from '../keyboard';
import { Button } from './Button';
import { Num } from './Num';

interface CashConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Cash in hand after the entry, from the backend (`cash_after_paisa`). */
  cashAfter: Paisa;
  /** The user said yes: save with the confirmation. */
  onConfirm: () => void;
  /** Called when the box has closed and focus is about to return; the screen may take it. */
  onCloseFocus?: ((event: Event) => void) | undefined;
}

export function CashConfirmDialog({
  open,
  onOpenChange,
  cashAfter,
  onConfirm,
  onCloseFocus,
}: CashConfirmDialogProps) {
  const { language, direction } = useUiLanguage();
  const grouping = useGrouping();
  const noRef = useRef<HTMLButtonElement>(null);
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
            noRef.current?.focus();
          }}
          onCloseAutoFocus={(event) => {
            onCloseFocus?.(event);
            if (event.defaultPrevented) return;
            const target = returnFocusRef.current;
            if (!target?.isConnected) return;
            event.preventDefault();
            target.focus();
          }}
          className="fixed inset-0 z-50 m-auto flex h-fit w-full max-w-dialog flex-col gap-3 rounded-3 bg-surface p-6 text-ink shadow-overlay"
        >
          <AlertDialog.Title className="type-section">
            <Localized id="common-cash-confirm-title" />
          </AlertDialog.Title>
          <AlertDialog.Description className="type-body text-ink">
            {/* The child element is needed: without one Fluent does not read the <num> markup. */}
            <Localized
              id="common-cash-confirm-text"
              vars={{ amount: formatPaisa(cashAfter, { grouping }) }}
              elems={{ num: <Num /> }}
            >
              <span />
            </Localized>
          </AlertDialog.Description>
          <div className="mt-2 flex flex-wrap justify-end gap-2">
            <Button
              ref={noRef}
              shortcut={SHORTCUTS.back}
              onClick={() => {
                onOpenChange(false);
              }}
            >
              <Localized id="common-cash-confirm-no" />
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                onConfirm();
                onOpenChange(false);
              }}
            >
              <Localized id="common-cash-confirm-yes" />
            </Button>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
