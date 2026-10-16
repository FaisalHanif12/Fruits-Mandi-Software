// The steps of a wizard (design system 6, WizardUr board): done steps in green with a tick, the
// current step in primary with its number, the next ones plain. Colour is never the only signal:
// a tick, a number, and aria-current="step" go with it (NFR-USE-12).

import { Check } from 'lucide-react';
import type { ReactNode } from 'react';
import { Num } from './Num';

export interface Step {
  id: string;
  label: ReactNode;
}

interface StepperProps {
  steps: readonly Step[];
  /** Index of the current step. Steps before it are done. */
  current: number;
  /** What the list is, read by screen readers. Comes from Fluent. */
  label: string;
}

function stepClasses(state: 'done' | 'current' | 'next'): string {
  if (state === 'done') return 'bg-money-in-soft text-money-in';
  if (state === 'current') return 'bg-primary text-on-primary';
  return 'bg-surface-sunken text-ink-muted';
}

export function Stepper({ steps, current, label }: StepperProps) {
  return (
    <ol aria-label={label} className="flex min-w-0 gap-2">
      {steps.map((step, index) => {
        const state = index < current ? 'done' : index === current ? 'current' : 'next';
        return (
          <li
            key={step.id}
            aria-current={state === 'current' ? 'step' : undefined}
            className={`flex min-w-0 flex-1 items-center gap-3 rounded-2 px-4 py-2 ${stepClasses(state)}`}
          >
            {state === 'done' ? (
              <Check aria-hidden="true" className="size-icon-md shrink-0" />
            ) : (
              <span
                className={`flex size-5 shrink-0 items-center justify-center rounded-full ${state === 'current' ? 'bg-surface text-primary' : 'bg-surface text-ink-muted'}`}
              >
                <Num>{String(index + 1)}</Num>
              </span>
            )}
            <span className="type-body min-w-0">{step.label}</span>
          </li>
        );
      })}
    </ol>
  );
}
