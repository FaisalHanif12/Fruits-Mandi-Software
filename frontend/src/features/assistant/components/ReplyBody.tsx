// The body of one reply (PRD FR-7.74, FR-7.76, FR-7.77, FR-7.79, FR-7.81, FR-7.82), one design for each
// outcome: an answer (also an estimate and advice), which one, not found, need dates, need party,
// locked, outside the books, change refused, not understood, checks failed, records differ, limit
// reached, offline and error. The words of every state that is not an answer are the panel's own, from
// Fluent (NFR-URD-4). The model's words only appear inside an answer, under the checks of FR-7.76.

import { Localized, useLocalization } from '@fluent/react';
import { useEffect, useRef } from 'react';
import type { AskReply } from '../../../bindings/AskReply';
import type { FailureKind } from '../../../bindings/FailureKind';
import type { PickChoice } from '../../../bindings/PickChoice';
import { uiErrorOf } from '../../../shared/api';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Button, ErrorNotice, Notice, PartnerLockedBox, RUNNING_NUMBER } from '../../../shared/ui';
import { usePartnerCodeStatus } from '../../partners';
import { useAssistantNavigation } from '../navigation';
import type { DockEntry } from '../store';
import { AnswerBlock } from './AnswerBlock';
import { AnswerTableView } from './AnswerTableView';
import { NameWithCode } from './AnswerText';
import { OptionsList } from './OptionsList';
import { SourceButtons } from './SourceFooter';
import { Figure } from './Figure';
import { StepLines } from './StepLines';
import { useListContext } from '../../analytics';

/** The reviewed words for a failed model call (D6 Section 2, `FailureKind`). */
export function failureMessageId(kind: FailureKind): string {
  return `assistant-failure-${kind.replaceAll('_', '-')}`;
}

interface ReplyBodyProps {
  entry: DockEntry;
  /** A suggested next question, or "ask again": sent as a new question. */
  onAsk: (text: string) => void;
  /** A choice for a question that waits. */
  onPick: (entryId: number, choice: PickChoice) => void;
  /** Any question is being worked on. */
  busy: boolean;
}

/** The partner section is behind its code: the box, and when the code is given the question goes on. */
function LockedView({
  entry,
  onPick,
}: {
  entry: DockEntry;
  onPick: (entryId: number, choice: PickChoice) => void;
}) {
  const status = usePartnerCodeStatus();
  const unlocked = status.data?.unlocked === true;
  const sent = useRef(false);
  const { id } = entry;
  // After the code the paused tool calls run and the model goes on, once, with no new count.
  useEffect(() => {
    if (unlocked && !sent.current) {
      sent.current = true;
      onPick(id, { type: 'partner_code' });
    }
  }, [unlocked, id, onPick]);
  // The box is the partner section's own: "Enter the code" (PRD FR-7.82, FR-17.11).
  return <PartnerLockedBox />;
}

function Failure({ reply, retry }: { reply: AskReply; retry: (() => void) | null }) {
  const { l10n } = useLocalization();
  const kind = reply.failure?.kind;
  const status = reply.failure?.http_status ?? null;
  return (
    <div className="flex flex-col gap-2">
      <Notice tone={reply.outcome === 'offline' ? 'info' : 'error'}>
        <Localized id={reply.outcome === 'offline' ? 'assistant-offline' : 'assistant-error'}>
          <span />
        </Localized>
      </Notice>
      {kind === undefined ? null : (
        <p className="type-body text-ink-muted">
          {l10n.getString(failureMessageId(kind))}
          {status === null ? null : ` (${String(status)})`}
        </p>
      )}
      {reply.outcome === 'error' && retry !== null ? (
        <div>
          <Button onClick={retry}>
            <Localized id="assistant-error-retry" />
          </Button>
        </div>
      ) : null}
    </div>
  );
}

/** A figure whose two places differ: what it is, whose it is, and both values with their places. */
function Differences({ reply }: { reply: AskReply }) {
  const { l10n } = useLocalization();
  const differences = reply.answer?.differences ?? [];
  return (
    <ul className="flex list-none flex-col gap-2">
      {differences.map((difference, index) => (
        // A difference has no id of its own; its place in the list is its key.
        <li key={index} className="flex flex-col gap-1 rounded-2 border border-border p-3">
          <p className="type-label text-ink">
            {l10n.getBundle(difference.figure_id) === null
              ? null
              : l10n.getString(difference.figure_id)}
            {difference.name === null ? null : (
              <>
                {' '}
                <NameWithCode name={difference.name} />
              </>
            )}
          </p>
          <ul className="flex list-none flex-col gap-1">
            {difference.places.map((place) => (
              <li key={place.place_id} className="type-body flex items-center gap-2">
                {l10n.getBundle(place.place_id) === null ? null : (
                  <span className="text-ink-muted">{l10n.getString(place.place_id)}</span>
                )}
                <Figure cell={place.cell} dir={null} checked={false} working={null} />
              </li>
            ))}
          </ul>
        </li>
      ))}
    </ul>
  );
}

export function ReplyBody({ entry, onAsk, onPick, busy }: ReplyBodyProps) {
  const language = useLanguage();
  const context = useListContext();
  const navigation = useAssistantNavigation();
  const reply = entry.reply;

  if (entry.working) {
    // A question that waited and goes on (after a choice or the code) says so.
    return <StepLines steps={entry.steps} />;
  }
  if (entry.error !== null) {
    const codes = entry.error.errors.map((item) => item.code);
    const settings =
      codes.includes('ERR_ASSISTANT_OFF') || codes.includes('ERR_ASSISTANT_KEY_MISSING');
    return (
      <div className="flex flex-col gap-2">
        <ErrorNotice error={uiErrorOf(entry.error)} />
        {settings && navigation !== null ? (
          <div>
            <Button onClick={navigation.openSettings}>
              <Localized id="assistant-open-settings" />
            </Button>
          </div>
        ) : null}
      </div>
    );
  }
  if (reply === null) return null;

  switch (reply.outcome) {
    case 'answered':
    case 'estimate':
    case 'advice':
      return reply.answer === null ? null : (
        <AnswerBlock answer={reply.answer} outcome={reply.outcome} onAsk={onAsk} busy={busy} />
      );
    case 'which_one':
      return (
        <div className="flex flex-col gap-2">
          <Localized id="assistant-which-one">
            <p className="type-lead text-ink" />
          </Localized>
          <OptionsList
            options={reply.options}
            onPick={(key) => {
              onPick(entry.id, { type: 'match', key });
            }}
          />
          {reply.more_options ? (
            <Localized id="assistant-which-one-more">
              <p className="type-body text-ink-muted" />
            </Localized>
          ) : null}
        </div>
      );
    case 'not_found':
      return (
        <div className="flex flex-col gap-2">
          <Localized id="assistant-not-found">
            <p className="type-lead text-ink" />
          </Localized>
          {reply.options.length === 0 ? null : (
            <>
              <Localized id="assistant-not-found-closest">
                <p className="type-body text-ink-muted" />
              </Localized>
              <OptionsList
                options={reply.options}
                onPick={(key) => {
                  onPick(entry.id, { type: 'match', key });
                }}
              />
            </>
          )}
        </div>
      );
    case 'need_dates':
      return (
        <Notice tone="info">
          <Localized id="assistant-need-dates">
            <span />
          </Localized>
        </Notice>
      );
    case 'need_party':
      return (
        <Notice tone="info">
          <Localized id="assistant-need-party">
            <span />
          </Localized>
        </Notice>
      );
    case 'locked':
      return <LockedView entry={entry} onPick={onPick} />;
    case 'outside_books':
      return (
        <div className="flex flex-col gap-2">
          <Notice tone="info">
            <Localized id="assistant-outside-books">
              <span />
            </Localized>
          </Notice>
          {reply.answer === null || reply.answer.sources.length === 0 ? null : (
            <>
              <Localized id="assistant-outside-books-nearest">
                <p className="type-body text-ink-muted" />
              </Localized>
              <SourceButtons sources={reply.answer.sources} />
            </>
          )}
        </div>
      );
    case 'change_refused': {
      const route = reply.entry_route;
      const party = route?.party ?? null;
      return (
        <div className="flex flex-col gap-2">
          <Notice tone="info">
            <Localized id="assistant-change-refused">
              <span />
            </Localized>
          </Notice>
          <Localized id="assistant-change-refused-help">
            <p className="type-body text-ink-muted" />
          </Localized>
          {route === null || navigation === null ? null : (
            <div>
              <Button
                onClick={() => {
                  navigation.openEntry(route);
                }}
              >
                {party === null
                  ? context.text('assistant-open-entry')
                  : context.text('assistant-open-entry-party', {
                      name: displayName(party, language),
                    })}
              </Button>
            </div>
          )}
        </div>
      );
    }
    case 'not_understood':
      return (
        <Notice tone="info">
          <Localized id="assistant-not-understood">
            <span />
          </Localized>
        </Notice>
      );
    case 'checks_failed':
      // The records the model read, as plain tables with their sources and no figure of the model's.
      return (
        <div className="flex flex-col gap-3">
          <Notice tone="warn">
            <Localized id="assistant-checks-failed">
              <span />
            </Localized>
          </Notice>
          {reply.answer?.tables.map((table) => (
            <AnswerTableView key={table.result_id} table={table} />
          ))}
          {reply.answer === null ? null : <SourceButtons sources={reply.answer.sources} />}
        </div>
      );
    case 'records_differ':
      return (
        <div className="flex flex-col gap-3">
          <Notice tone="error">
            <Localized id="assistant-records-differ">
              <span />
            </Localized>
          </Notice>
          <Differences reply={reply} />
          {reply.answer === null ? null : <SourceButtons sources={reply.answer.sources} />}
        </div>
      );
    case 'limit_reached':
      return (
        <Notice tone="warn">
          <Localized
            id="assistant-limit-reached"
            vars={{ limit: String(reply.limit) }}
            elems={RUNNING_NUMBER}
          >
            <span />
          </Localized>{' '}
          <Localized id="assistant-limit-chips">
            <span />
          </Localized>
        </Notice>
      );
    case 'offline':
    case 'error':
      return (
        <Failure
          reply={reply}
          retry={
            entry.asked.type === 'text'
              ? () => {
                  onAsk(entry.asked.type === 'text' ? entry.asked.text : '');
                }
              : null
          }
        />
      );
  }
}
