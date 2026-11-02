// The conversation of the chat window (PRD FR-7.73, plan 51): a day label, then each question the Owner
// asked as a navy bubble at the end side, with its answer under it as a wide white card that can hold
// tables, charts and the bands. Oldest at the top, the newest at the bottom. It scrolls on its own, so
// the box below it never moves. With no question yet it greets the Owner and shows the quick questions
// as round chips under the greeting (PRD FR-7.46, FR-7.80, Code Standards F8).

import { Localized, useLocalization } from '@fluent/react';
import { useEffect, useRef, type ReactNode } from 'react';
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
  /** The quick questions, shown under the greeting while the conversation is empty. */
  chips: ReactNode;
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

/** "Today", between two fine rules, at the top of the conversation as in a chat widget. */
function DayLabel() {
  return (
    <p className="flex items-center gap-3 type-label text-ink-muted">
      <span aria-hidden="true" className="flex-1 border-t border-border" />
      <Localized id="common-period-today" />
      <span aria-hidden="true" className="flex-1 border-t border-border" />
    </p>
  );
}

/** What the assistant says, as a card: the corner by the speaker is square, the others are round. */
const ASSISTANT_BUBBLE = 'rounded-4 rounded-ss-1 border border-border bg-surface';

export function Conversation({ entries, onAsk, onPick, busy, chips }: ConversationProps) {
  const { l10n } = useLocalization();
  const end = useRef<HTMLLIElement>(null);
  // The newest question is brought into view when it is added (the answer grows under it).
  const count = entries.length;
  useEffect(() => {
    if (count > 0) end.current?.scrollIntoView({ block: 'start' });
  }, [count]);
  if (count === 0) {
    return (
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto bg-bg p-4">
        <DayLabel />
        <div className={`flex max-w-4/5 flex-col gap-1 px-4 py-3 ${ASSISTANT_BUBBLE}`}>
          <h3 className="type-lead text-ink">
            <Localized id="assistant-empty-title" />
          </h3>
          <p className="type-body text-ink-muted">
            <Localized id="assistant-empty-help" />
          </p>
        </div>
        {chips}
      </div>
    );
  }
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto bg-bg p-4">
      <DayLabel />
      <ol
        aria-label={l10n.getString('assistant-conversation-label')}
        className="flex list-none flex-col gap-5"
      >
        {entries.map((entry, index) => (
          <li
            key={entry.id}
            ref={index === count - 1 ? end : undefined}
            className="flex flex-col gap-3"
          >
            <p
              aria-label={l10n.getString('assistant-you-asked')}
              className="max-w-4/5 self-end rounded-4 rounded-ee-1 bg-primary px-4 py-2 type-body text-on-primary"
            >
              <QuestionLine asked={entry.asked} />
            </p>
            <div aria-live="polite" className={`flex flex-col gap-3 p-4 ${ASSISTANT_BUBBLE}`}>
              {entry.asked.type === 'log' && !entry.working ? (
                // A question read again shows plain tables on today's books; the model's old words are
                // not shown over new figures (PRD FR-7.84).
                <Localized id="assistant-rerun-note">
                  <p className="type-body text-ink-muted" />
                </Localized>
              ) : null}
              <ReplyBody entry={entry} onAsk={onAsk} onPick={onPick} busy={busy} />
            </div>
          </li>
        ))}
      </ol>
    </div>
  );
}
