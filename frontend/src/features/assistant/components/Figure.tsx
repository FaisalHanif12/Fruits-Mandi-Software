// A figure inside an answer (PRD FR-7.74, FR-7.75, FR-7.76): the value the app put there, written
// large through `<Num>` so the eye goes to it first, with the word of its side (بنام or جمع), its
// direction word when it is a difference, a ✓ when its two places agree, and ⓘ with the working when
// the calculator or the estimator made it. The panel writes the figure; it never works one out: the
// cell, the side, the direction and the working are the backend's (CLAUDE.md rule 1, Code Standards
// F2, F42). A number the Owner typed in his own question, and a figure with no second place, have no ✓.

import { useLocalization } from '@fluent/react';
import { Check, Info } from 'lucide-react';
import { Popover } from 'radix-ui';
import type { AnswerWorking } from '../../../bindings/AnswerWorking';
import type { FigureDirection } from '../../../bindings/FigureDirection';
import type { ReportCell } from '../../../bindings/ReportCell';
import type { WorkingStep } from '../../../bindings/WorkingStep';
import type { ListContext } from '../../../shared/grid';
import { useUiLanguage } from '../../../shared/i18n';
import { SHORTCUTS, matchesShortcut } from '../../../shared/keyboard';
import { Num } from '../../../shared/ui';
import { useListContext } from '../../analytics';
import { cellText, sideText } from '../../reports';

/** The word that goes with a direction (put in by the app, never chosen by the model, FR-7.75). */
function directionMessageId(dir: FigureDirection): string {
  switch (dir) {
    case 'up':
      return 'assistant-direction-more';
    case 'down':
      return 'assistant-direction-less';
    case 'same':
      return 'assistant-direction-same';
  }
}

/** A cell as the figure and its side word, as text. */
export function figureParts(
  cell: ReportCell,
  context: ListContext,
): { value: string; side: string } {
  return { value: cellText(cell, context), side: sideText(cell, context) };
}

/** A term of a working: the figure and its side word, in one run that never flips. */
function Term({ cell, context }: { cell: ReportCell; context: ListContext }) {
  const { value, side } = figureParts(cell, context);
  return (
    <bdi>
      <Num inherit>{value}</Num>
      {side === '' ? null : <span> {side}</span>}
    </bdi>
  );
}

/** The sign between the terms of a step. */
const SIGN: Record<'sum' | 'difference' | 'times' | 'divided_by', string> = {
  sum: ' + ',
  difference: ' − ',
  times: ' × ',
  divided_by: ' ÷ ',
};

/** The terms of a step joined by a sign. */
function Joined({
  cells,
  sign,
  context,
}: {
  cells: readonly ReportCell[];
  sign: string;
  context: ListContext;
}) {
  return (
    <>
      {cells.map((cell, index) => (
        // The cells of a working have no id of their own; their place in the step is their key.
        <span key={index}>
          {index === 0 ? null : sign}
          <Term cell={cell} context={context} />
        </span>
      ))}
    </>
  );
}

/** One step: its inputs with the sign of its operation, then `=` and its result. Nothing is worked out. */
function StepLine({ step, context }: { step: WorkingStep; context: ListContext }) {
  const { inputs, result } = step;
  const last = inputs.at(-1);
  let body;
  switch (step.op) {
    case 'sum':
    case 'difference':
    case 'times':
    case 'divided_by':
      body = <Joined cells={inputs} sign={SIGN[step.op]} context={context} />;
      break;
    case 'average':
      // The count of the months is the last input when the backend sent it: (a + b + c) ÷ n.
      if (last?.type === 'number' && inputs.length > 1) {
        const terms = inputs.slice(0, -1);
        body = (
          <>
            {terms.length > 1 ? '(' : null}
            <Joined cells={terms} sign=" + " context={context} />
            {terms.length > 1 ? ')' : null}
            {' ÷ '}
            <Term cell={last} context={context} />
          </>
        );
      } else {
        body = (
          <>
            {context.text('assistant-working-average-of')}{' '}
            <Joined cells={inputs} sign=", " context={context} />
          </>
        );
      }
      break;
    case 'percent_of': {
      const [first, second] = inputs;
      body =
        first === undefined || second === undefined ? null : (
          <>
            <Term cell={first} context={context} />
            {' ÷ '}
            <Term cell={second} context={context} />
            {' × 100'}
          </>
        );
      break;
    }
    case 'change_percent': {
      const [first, second] = inputs;
      body =
        first === undefined || second === undefined ? null : (
          <>
            {'('}
            <Term cell={second} context={context} />
            {' − '}
            <Term cell={first} context={context} />
            {') ÷ '}
            <Term cell={first} context={context} />
            {' × 100'}
          </>
        );
      break;
    }
  }
  return (
    <p dir="ltr" className="type-body text-start">
      {body}
      {' = '}
      <Term cell={result} context={context} />
    </p>
  );
}

const WORKING_CLASSES =
  'z-30 flex max-w-reading flex-col gap-1 rounded-2 border border-border bg-surface p-3 text-ink shadow-overlay';

/** The ⓘ of a figure the calculator or the estimator made: its working, with the real numbers. */
export function WorkingInfo({
  working,
  note,
}: {
  working: AnswerWorking;
  /** A line after the steps that says what the last step's inputs are (a whole year: the books + the estimate). */
  note?: string | undefined;
}) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const context = useListContext();
  const label = l10n.getString('calc-info-label');
  // A working this build has no words for shows the plain title, never the raw id.
  const title =
    l10n.getBundle(working.title_id) === null
      ? l10n.getString('calc-info-title')
      : l10n.getString(working.title_id);
  return (
    <Popover.Root>
      <Popover.Trigger
        aria-label={label}
        title={label}
        onKeyDown={(event) => {
          // Ctrl+I opens it from the button as well (PRD 7.3).
          if (matchesShortcut(event.nativeEvent, SHORTCUTS.explain)) {
            event.preventDefault();
            event.currentTarget.click();
          }
        }}
        className="inline-flex size-control-compact shrink-0 items-center justify-center rounded-1 text-ink-muted transition-calm"
      >
        <Info aria-hidden="true" className="size-icon-sm" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content lang={language} dir={direction} sideOffset={4} className={WORKING_CLASSES}>
          <p className="type-label">{title}</p>
          {working.steps.map((step, index) => (
            <StepLine key={`${step.op}-${String(index)}`} step={step} context={context} />
          ))}
          {note === undefined ? null : <p className="type-body text-ink-muted">{note}</p>}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

interface FigureProps {
  cell: ReportCell;
  /** The direction word of a difference or a change, put in by the app. */
  dir: FigureDirection | null;
  /** True when the figure's two places agree (✓). */
  checked: boolean;
  working: AnswerWorking | null;
  /** A line under the working's last step, for a figure whose last step adds two named parts. */
  workingNote?: string | undefined;
}

export function Figure({ cell, dir, checked, working, workingNote }: FigureProps) {
  const { l10n } = useLocalization();
  const context = useListContext();
  const { value, side } = figureParts(cell, context);
  return (
    <span className="inline-flex items-center gap-1 whitespace-nowrap align-middle">
      <span className="type-figure text-ink">
        <Num inherit>{value}</Num>
      </span>
      {side === '' ? null : <span className="type-body">{side}</span>}
      {dir === null ? null : (
        <span className="type-body">{l10n.getString(directionMessageId(dir))}</span>
      )}
      {checked ? (
        <span className="inline-flex items-center text-money-in" data-checked="">
          <Check aria-hidden="true" className="size-icon-sm" />
          <span className="sr-only">{l10n.getString('assistant-checked')}</span>
        </span>
      ) : null}
      {working === null ? null : <WorkingInfo working={working} note={workingNote} />}
    </span>
  );
}
