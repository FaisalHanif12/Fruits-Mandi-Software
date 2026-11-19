// A big choice with a picture, a name and a line of help (plan 56: the first screen of a new
// install). The whole card is one button, so the keyboard and a screen reader treat it as one. It is
// for a choice between a few large ways to go on, never for a list. `data-nav-field` lets the
// arrow keys of a form move between cards in the directions the user sees (PRD FR-7.13).

import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

interface ChoiceCardProps {
  icon: LucideIcon;
  /** The name of the choice, in step 3 of the type scale. */
  title: ReactNode;
  /** One or two lines saying what happens. */
  help: ReactNode;
  onChoose: () => void;
}

export function ChoiceCard({ icon: Icon, title, help, onChoose }: ChoiceCardProps) {
  return (
    <button
      type="button"
      data-nav-field
      onClick={onChoose}
      className="flex min-w-0 flex-1 flex-col items-start gap-3 rounded-3 border border-border-control bg-surface p-5 text-start transition-calm hover:border-primary hover:bg-primary-soft"
    >
      <span className="flex size-badge shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary">
        <Icon aria-hidden="true" className="size-icon-lg" />
      </span>
      <span className="type-section text-ink">{title}</span>
      <span className="type-body text-ink-muted">{help}</span>
    </button>
  );
}
