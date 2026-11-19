// A short note inside a screen: a warning that still lets the user go on (amber), something worth
// knowing (neutral) or a problem (red). The tone is always written as well as coloured, because the
// icon and the words go with it (NFR-USE-12). Used for "cash will go below zero", "more than is
// payable", and the right-type notes (PRD FR-6.7, FR-15.7, FR-7.68). The text and any button are
// the screen's own, from Fluent.

import { CircleAlert, Info, TriangleAlert, type LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

export type NoticeTone = 'warn' | 'info' | 'error';

interface NoticeProps {
  tone?: NoticeTone;
  children: ReactNode;
  /** A button or link that answers the note, for example "make it an advance". */
  action?: ReactNode;
}

const STYLE: Record<NoticeTone, string> = {
  warn: 'border-warn-border bg-warn-soft text-warn',
  info: 'border-border bg-surface-sunken text-ink',
  error: 'border-due bg-due-soft text-due',
};

const ICON: Record<NoticeTone, LucideIcon> = {
  warn: TriangleAlert,
  info: Info,
  error: CircleAlert,
};

export function Notice({ tone = 'warn', children, action }: NoticeProps) {
  const Icon = ICON[tone];
  return (
    <div
      role={tone === 'error' ? 'alert' : 'status'}
      className={`flex flex-wrap items-center gap-x-3 gap-y-2 rounded-2 border p-3 ${STYLE[tone]}`}
    >
      <Icon aria-hidden="true" className="size-icon-md shrink-0" />
      <span className="type-body min-w-0 flex-1">{children}</span>
      {action}
    </div>
  );
}
