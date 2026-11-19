// The status bar button "سوال پوچھیں F1" (PRD FR-5.5, FR-7.73): it opens and closes the question
// panel, with its key in a chip (FR-7.4), and beside it the day's count "آج 5 / 20" (FR-7.81). The
// count is the backend's; the button works before it is known and when it cannot be read.

import { Localized } from '@fluent/react';
import { MessageCircleQuestion } from 'lucide-react';
import { SHORTCUTS } from '../../../shared/keyboard';
import { KeyChip, RUNNING_NUMBER } from '../../../shared/ui';
import { useAssistantStatus } from '../hooks';
import { useAssistantStore } from '../store';

export function AssistantButton() {
  const open = useAssistantStore((state) => state.open);
  const { openDock, closeDock } = useAssistantStore.getState();
  const status = useAssistantStatus();
  return (
    <span className="flex items-center gap-3">
      {status.data === undefined ? null : (
        <Localized
          id="assistant-button-count"
          vars={{ used: String(status.data.count_today), limit: String(status.data.limit) }}
          elems={RUNNING_NUMBER}
        >
          <span />
        </Localized>
      )}
      <button
        type="button"
        aria-expanded={open}
        onClick={() => {
          if (open) closeDock();
          else openDock();
        }}
        className="flex min-h-target items-center gap-2 rounded-1 border border-border-control bg-surface px-3 text-ink"
      >
        <MessageCircleQuestion aria-hidden="true" className="size-icon-sm text-primary" />
        <Localized id="assistant-button" />
        <KeyChip shortcut={SHORTCUTS.askApp} tone="on-light" />
      </button>
    </span>
  );
}
