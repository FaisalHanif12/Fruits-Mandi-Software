// Nothing to show yet: says why, and offers the next step with one button and its key
// (design system 6, PRD FR-7.46, Code Standards F8). Example: "ابھی کوئی گاہک نہیں" with
// "نیا گاہک F2".

import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import { StateBadge } from './StateBadge';

interface EmptyStateProps {
  icon: LucideIcon;
  /** One line: what is empty. */
  title: ReactNode;
  /** One line: what to do. */
  help: ReactNode;
  /** One button, usually primary with its key. */
  action?: ReactNode;
}

export function EmptyState({ icon, title, help, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 p-4 text-center">
      <StateBadge icon={icon} tone="primary" />
      <h2 className="type-section text-ink">{title}</h2>
      <p className="type-body text-ink-muted">{help}</p>
      {action}
    </div>
  );
}
