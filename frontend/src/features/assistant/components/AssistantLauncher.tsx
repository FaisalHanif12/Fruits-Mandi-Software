// The round launcher of Ask the app (plan 51, PRD FR-7.73): a button in the app's own navy that floats
// at the end of every screen, just above the status bar (bottom right in English, bottom left in Urdu,
// through the logical offsets). It opens the chat window above it, and the same button then closes it,
// showing a close icon. A small dot says the assistant is on and has a key; the header of the window
// says it in words too, so the dot is never the only signal (NFR-USE-12).
//
// It is drawn by the dock inside the shell's zero-height wrapper that sits on the status bar's top edge,
// so its edge offset counts from there (`--chat-edge`, smaller in a short window). It is not inside the
// chat window, so the window's focus trap and its closed, inert state never touch it. The shell keeps a
// band under every screen as tall as the launcher and its gaps (`--chat-corner`, plan 52), so the
// shut launcher never covers a control.

import { useLocalization } from '@fluent/react';
import { MessageCircleQuestion, X } from 'lucide-react';
import { useAssistantStatus } from '../hooks';
import { useAssistantStore } from '../store';

/** The id of the chat window, for `aria-controls`. */
export const CHAT_WINDOW_ID = 'assistant-chat-window';

export function AssistantLauncher() {
  const { l10n } = useLocalization();
  const open = useAssistantStore((state) => state.open);
  const { openDock, closeDock } = useAssistantStore.getState();
  const status = useAssistantStatus();
  const ready = status.data?.enabled === true && status.data.key_saved;
  const name = l10n.getString(open ? 'assistant-close' : 'assistant-button');
  return (
    <button
      type="button"
      aria-expanded={open}
      aria-controls={CHAT_WINDOW_ID}
      aria-label={name}
      title={name}
      onClick={() => {
        if (open) closeDock();
        else openDock();
      }}
      className="absolute bottom-chat-edge end-chat-edge flex size-launcher items-center justify-center rounded-full bg-primary text-on-primary shadow-overlay ring-4 ring-primary-soft transition-calm hover:bg-primary-chip"
    >
      {open ? (
        <X aria-hidden="true" className="size-icon-lg" />
      ) : (
        <MessageCircleQuestion aria-hidden="true" className="size-icon-lg" />
      )}
      {ready && !open ? (
        <span
          aria-hidden="true"
          className="absolute end-0 top-0 size-4 rounded-full border-2 border-surface bg-money-in"
        />
      ) : null}
    </button>
  );
}
