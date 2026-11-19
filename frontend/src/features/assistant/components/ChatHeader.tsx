// The header of the chat window (plan 51, PRD FR-7.73): the assistant's round icon with a dot for its
// state, its name, its state in words (ready, preparing an answer, no internet, switched off) with
// today's count, then "نیا سوال" (New question), the size button and close. The dot has a word beside
// it, so colour is never the only signal (NFR-USE-12).

import { Localized, useLocalization } from '@fluent/react';
import { MessageCircleQuestion, Maximize2, Minimize2, RotateCcw, X } from 'lucide-react';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Button, RUNNING_NUMBER } from '../../../shared/ui';
import { useAssistantStatus } from '../hooks';
import { useAssistantStore } from '../store';

export type ChatState = 'ready' | 'working' | 'offline' | 'off';

const STATE_MESSAGE: Record<ChatState, string> = {
  ready: 'assistant-state-ready',
  working: 'assistant-state-working',
  offline: 'assistant-state-offline',
  off: 'assistant-state-off',
};

/** The dot of each state. Ready is green, working amber; the other two are plain grey. */
const STATE_DOT: Record<ChatState, string> = {
  ready: 'bg-money-in',
  working: 'bg-warn',
  offline: 'bg-border-control',
  off: 'bg-border-control',
};

/** What the assistant is doing, from what the store and the status say. */
export function chatStateOf(input: {
  working: boolean;
  asking: boolean;
  lastOutcome: string | null;
}): ChatState {
  if (input.working) return 'working';
  if (!input.asking) return 'off';
  if (input.lastOutcome === 'offline') return 'offline';
  return 'ready';
}

interface ChatHeaderProps {
  state: ChatState;
  /** "نیا سوال": forgets the conversation. */
  onNew: () => void;
  onClose: () => void;
}

export function ChatHeader({ state, onNew, onClose }: ChatHeaderProps) {
  const { l10n } = useLocalization();
  /** The large size of the window; the store keeps it as `tall`. */
  const large = useAssistantStore((store) => store.tall);
  const empty = useAssistantStore((store) => store.entries.length === 0);
  const { setTall } = useAssistantStore.getState();
  const status = useAssistantStatus();
  const SizeIcon = large ? Minimize2 : Maximize2;
  const sizeLabel = l10n.getString(large ? 'assistant-smaller' : 'assistant-larger');
  return (
    <header className="flex shrink-0 items-center gap-3 border-b border-border bg-surface px-4 py-2">
      <span className="relative shrink-0">
        <span className="flex size-control items-center justify-center rounded-3 bg-primary text-on-primary">
          <MessageCircleQuestion aria-hidden="true" className="size-icon-lg" />
        </span>
        <span
          aria-hidden="true"
          className={`absolute -bottom-1 -end-1 size-4 rounded-full border-2 border-surface ${STATE_DOT[state]}`}
        />
      </span>
      <div className="min-w-0 flex-1">
        <h2 className="type-label truncate text-ink">
          <Localized id="assistant-dock-title" />
        </h2>
        <p className="flex flex-wrap items-center gap-x-2 type-body text-ink-muted">
          <span>{l10n.getString(STATE_MESSAGE[state])}</span>
          {status.data === undefined ? null : (
            <>
              <span aria-hidden="true" className="size-1 self-center rounded-full bg-ink-muted" />
              <Localized
                id="assistant-button-count"
                vars={{
                  used: String(status.data.count_today),
                  limit: String(status.data.limit),
                }}
                elems={RUNNING_NUMBER}
              >
                <span />
              </Localized>
            </>
          )}
        </p>
      </div>
      <Button round onClick={onNew} disabled={empty}>
        <RotateCcw aria-hidden="true" className="size-icon-sm" />
        <Localized id="assistant-new-conversation" />
      </Button>
      <Button
        round
        iconOnly
        variant="ghost"
        accessibleName={`${sizeLabel} (${(large ? SHORTCUTS.dockShorter : SHORTCUTS.dockTaller).label})`}
        onClick={() => {
          setTall(!large);
        }}
      >
        <SizeIcon aria-hidden="true" className="size-icon-md" />
      </Button>
      <Button
        round
        iconOnly
        variant="ghost"
        accessibleName={l10n.getString('assistant-close')}
        onClick={onClose}
      >
        <X aria-hidden="true" className="size-icon-md" />
      </Button>
    </header>
  );
}
