// A question being worked on (PRD FR-7.73, NFR-AI-3): the first line "سوال سمجھ رہا ہوں…" at once,
// then one line for each step the backend reports, in the app's own reviewed words ("اکرم ٹول ٹیکس کا
// کھاتہ دیکھ رہا ہوں…"). A step the build has no words for is told in the plain line "جواب تیار کر رہا
// ہوں…", never as a raw id. No animation: the lines are the progress. After a few seconds the last line
// says how long the Owner has waited, from this screen's own clock (plan 55): the backend sends the
// steps ("trying again", "still slow") and this only counts the seconds.

import { useLocalization } from '@fluent/react';
import { useEffect, useState } from 'react';
import type { AssistantStep } from '../../../bindings/AssistantStep';
import type { ReportMessageArg } from '../../../bindings/ReportMessageArg';
import type { ListContext } from '../../../shared/grid';
import { useListContext } from '../../analytics';
import { messageText } from '../../reports';

/** A name may be called `party` or `name` in a step message: both are given, so either reads. */
function withAliases(args: readonly ReportMessageArg[]): ReportMessageArg[] {
  const has = (name: string) => args.some((arg) => arg.name === name);
  const party = args.find((arg) => arg.name === 'party');
  const name = args.find((arg) => arg.name === 'name');
  return [
    ...args,
    ...(party !== undefined && !has('name') ? [{ name: 'name', value: party.value }] : []),
    ...(name !== undefined && !has('party') ? [{ name: 'party', value: name.value }] : []),
  ];
}

/**
 * What each step line needs to be told: the argument its words are built on. A step that arrives
 * without it (a party that could not be found) is told in the plain line, never with a hole in it.
 */
const STEP_NEEDS: Readonly<Record<string, string>> = {
  'assistant-step-finding': 'text',
  'assistant-step-reading-card': 'name',
  'assistant-step-reading-ledger': 'name',
  'assistant-step-reading-report': 'report',
  'assistant-step-understood': 'kind',
};

const MILLIS_PER_SECOND = 1000;
/** Fewer seconds than this are not worth a line: most questions are answered by then. */
const SECONDS_BEFORE_SHOWN = 5;

/** The whole seconds since this question started waiting, counted on the screen's own clock. */
export function useWaitedSeconds(): number {
  const [seconds, setSeconds] = useState(0);
  useEffect(() => {
    const started = Date.now();
    const timer = setInterval(() => {
      setSeconds(Math.trunc((Date.now() - started) / MILLIS_PER_SECOND));
    }, MILLIS_PER_SECOND);
    return () => {
      clearInterval(timer);
    };
  }, []);
  return seconds;
}

/** The words of one step line. */
export function stepText(
  step: AssistantStep,
  context: ListContext,
  known: (id: string) => boolean,
): string {
  const args = withAliases(step.args);
  const needs = STEP_NEEDS[step.id];
  const complete = needs === undefined || args.some((arg) => arg.name === needs);
  return known(step.id) && complete
    ? messageText(step.id, args, context)
    : context.text('assistant-step-generic');
}

export function StepLines({ steps }: { steps: readonly AssistantStep[] }) {
  const { l10n } = useLocalization();
  const context = useListContext();
  const known = (id: string) => l10n.getBundle(id) !== null;
  const waited = useWaitedSeconds();
  return (
    <ul
      role="status"
      aria-label={l10n.getString('assistant-working-label')}
      className="flex list-none flex-col gap-1 text-ink-muted"
    >
      <li className="type-body">{l10n.getString('assistant-understanding')}</li>
      {steps.map((step, index) => (
        // A step line has no id of its own; the order of the steps is the order of the work.
        <li key={index} className="type-body">
          {stepText(step, context, known)}
        </li>
      ))}
      {waited < SECONDS_BEFORE_SHOWN ? null : (
        // The count changes every second: it is for the eye, so the live region does not read it out.
        <li aria-hidden="true" className="type-body">
          {context.text('assistant-waited', { seconds: String(waited) })}
        </li>
      )}
    </ul>
  );
}
