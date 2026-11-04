// The numbered steps under the answer to "how do I print or find this?" (plan 54 S2). The backend
// sends each step as a message id and its values, and the panel writes them in its own reviewed
// words: "Open report R5", "Choose the buyer: Akram Toll Tax (03040001)", "Press F9 to print". It
// works nothing out and fills nothing in: the steps are what the Owner does, and the button under the
// answer opens the report with the party and the dates already set (PRD FR-7.74).

import { useLocalization } from '@fluent/react';
import type { ReportMessageArg } from '../../../bindings/ReportMessageArg';
import { isoToDisplayDate } from '../../../shared/format';
import type { ListContext } from '../../../shared/grid';
import { useListContext } from '../../analytics';
import { messageText } from '../../reports';

/** One step as the backend sends it. */
export interface HowStepView {
  id: string;
  args: readonly ReportMessageArg[];
}

/**
 * What each step needs to be written. A step the build has no words for, or one that arrives without
 * a value its words are built on, is left out: a step is never shown as a raw id or with a hole in it.
 */
const HOW_NEEDS: Readonly<Record<string, readonly string[]>> = {
  'assistant-how-open-report': ['report'],
  'assistant-how-set-party': ['role', 'party', 'code'],
  // A record with no code (a partner), and the row of a report the Owner must look at (a trip, a head).
  'assistant-how-set-record': ['role', 'name'],
  'assistant-how-find-row': ['name'],
  'assistant-how-set-day': ['day'],
  'assistant-how-set-dates': ['from', 'to'],
  'assistant-how-press-print': [],
};

/** The values the backend sends as ISO dates, which the steps show as DD-MM-YYYY. */
const DATE_ARGS: ReadonlySet<string> = new Set(['day', 'from', 'to']);

function withDisplayDates(args: readonly ReportMessageArg[]): ReportMessageArg[] {
  return args.map((arg) =>
    DATE_ARGS.has(arg.name) && arg.value.type === 'text'
      ? { name: arg.name, value: { type: 'text', text: isoToDisplayDate(arg.value.text) } }
      : arg,
  );
}

/** The words of each step the build can write, in the order the backend sent them. */
export function howLines(
  steps: readonly HowStepView[],
  context: ListContext,
  known: (id: string) => boolean,
): string[] {
  const lines: string[] = [];
  for (const step of steps) {
    const needs = HOW_NEEDS[step.id];
    if (needs === undefined || !known(step.id)) continue;
    if (!needs.every((name) => step.args.some((arg) => arg.name === name))) continue;
    lines.push(messageText(step.id, withDisplayDates(step.args), context));
  }
  return lines;
}

export function HowSteps({ steps }: { steps: readonly HowStepView[] }) {
  const { l10n } = useLocalization();
  const context = useListContext();
  const lines = howLines(steps, context, (id) => l10n.getBundle(id) !== null);
  if (lines.length === 0) return null;
  return (
    <ol
      aria-label={l10n.getString('assistant-how-label')}
      className="type-body flex list-decimal flex-col gap-1 ps-6 text-ink"
    >
      {lines.map((line, index) => (
        // The order of the steps is the order of the work, and a step has no id of its own.
        <li key={index}>{line}</li>
      ))}
    </ol>
  );
}
