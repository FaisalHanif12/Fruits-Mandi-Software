// The conversation of the question panel (PRD FR-7.73): each question the Owner asked with its answer
// under it, oldest at the top and the newest at the bottom. It scrolls on its own, so the box below it
// never moves. With no question yet it says what the panel is for (PRD FR-7.46, Code Standards F8).

import { Localized, useLocalization } from '@fluent/react';
import { useEffect, useRef } from 'react';
import type { PickChoice } from '../../../bindings/PickChoice';
import type { Asked, DockEntry } from '../store';
import { NameWithCode } from './AnswerText';
import { QUICK_CHIPS } from './QuickChips';
import { ReplyBody } from './ReplyBody';

interface ConversationProps {
  entries: readonly DockEntry[];
  onAsk: (text: string) => void;
  onPick: (entryId: number, choice: PickChoice) => void;
  /** Any question is being worked on. */
  busy: boolean;
}

/** The question line: his own words as typed, or the name of the quick question he pressed. */
function QuestionLine({ asked }: { asked: Asked }) {
  if (asked.type === 'chip') {
    const chip = QUICK_CHIPS.find((found) => found.chip === asked.chip);
    return (
      <>
        {chip === undefined ? null : <Localized id={chip.messageId} />}
        {asked.party === null ? null : (
          <>
            {' '}
            <NameWithCode name={asked.party} />
          </>
        )}
      </>
    );
  }
  // The words as typed: their own script and direction, the way global search takes them.
  return <bdi dir="auto">{asked.text}</bdi>;
}

export function Conversation({ entries, onAsk, onPick, busy }: ConversationProps) {
  const { l10n } = useLocalization();
  const end = useRef<HTMLLIElement>(null);
  // The newest question is brought into view when it is added (the answer grows under it).
  const count = entries.length;
  useEffect(() => {
    if (count > 0) end.current?.scrollIntoView({ block: 'start' });
  }, [count]);
  if (count === 0) {
    return (
      <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 overflow-y-auto px-4 py-3 text-center">
        <h3 className="type-section text-ink">
          <Localized id="assistant-empty-title" />
        </h3>
        <p className="type-body max-w-reading text-ink-muted">
          <Localized id="assistant-empty-help" />
        </p>
      </div>
    );
  }
  return (
    <ol
      aria-label={l10n.getString('assistant-conversation-label')}
      className="flex min-h-0 flex-1 list-none flex-col gap-4 overflow-y-auto px-4 py-3"
    >
      {entries.map((entry, index) => (
        <li
          key={entry.id}
          ref={index === count - 1 ? end : undefined}
          className="flex flex-col gap-2"
        >
          <p
            aria-label={l10n.getString('assistant-you-asked')}
            className="type-label border-s-4 border-primary bg-primary-soft px-3 py-2 text-ink"
          >
            <QuestionLine asked={entry.asked} />
          </p>
          {entry.asked.type === 'log' && !entry.working ? (
            // A question read again shows plain tables on today's books; the model's old words are not
            // shown over new figures (PRD FR-7.84).
            <Localized id="assistant-rerun-note">
              <p className="type-body text-ink-muted" />
            </Localized>
          ) : null}
          <div aria-live="polite">
            <ReplyBody entry={entry} onAsk={onAsk} onPick={onPick} busy={busy} />
          </div>
        </li>
      ))}
    </ol>
  );
}
