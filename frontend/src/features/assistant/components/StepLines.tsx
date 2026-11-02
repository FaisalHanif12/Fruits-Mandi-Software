// A question being worked on (PRD FR-7.73, NFR-AI-3): the first line "سوال سمجھ رہا ہوں…" at once,
// then one line for each step the backend reports, in the app's own reviewed words ("اکرم ٹول ٹیکس کا
// کھاتہ دیکھ رہا ہوں…"). A step the build has no words for is told in the plain line "جواب تیار کر رہا
// ہوں…", never as a raw id. No animation: the lines are the progress.

import { useLocalization } from '@fluent/react';
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
};

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
    </ul>
  );
}
