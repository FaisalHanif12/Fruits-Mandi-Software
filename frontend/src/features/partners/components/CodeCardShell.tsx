// The look of a partner code card (plan 100 K, PRD FR-17.11): a modern card with a large lock or key
// picture, its name and what to do in the middle, and the fields under them. The card is as wide as a
// dialog and no wider; where it stands (the middle of the page) is the gate's job. It only draws: the
// fields, the buttons and the keys are the card's own.

import { Localized } from '@fluent/react';
import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

interface CodeCardShellProps {
  icon: LucideIcon;
  /** The Fluent id of the card's name. */
  titleId: string;
  /** The Fluent id of the line under it that says what to do. */
  helpId: string;
  children: ReactNode;
}

export function CodeCardShell({ icon: Icon, titleId, helpId, children }: CodeCardShellProps) {
  return (
    <section className="flex w-full max-w-dialog min-w-0 flex-col gap-4 rounded-3 border border-border bg-surface p-6 text-ink">
      <div className="flex flex-col items-center gap-2 text-center">
        <span
          aria-hidden="true"
          className="flex items-center justify-center rounded-full bg-primary-soft p-4 text-primary"
        >
          <Icon aria-hidden="true" className="size-icon-lg" />
        </span>
        <h2 className="type-section">
          <Localized id={titleId} />
        </h2>
        <Localized id={helpId}>
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      {children}
    </section>
  );
}
