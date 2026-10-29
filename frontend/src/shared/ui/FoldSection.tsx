// A part of a long form that folds (plan 38, account form): a heading that is a button, and the
// fields under it. The fields typed every time are in sections that start open; the rest start shut.
// A shut section never hides a value or a problem in secret: the heading says how many of its fields
// hold something (`filled`), and a section with a problem (`forceOpen`) opens itself and cannot be
// shut. The button is a real button: Tab reaches it, Enter or Space opens and shuts, and
// `aria-expanded` says which it is.

import { ChevronDown, ChevronUp } from 'lucide-react';
import { useId, useState, type ReactNode } from 'react';
import { Num } from './Num';

interface FoldSectionProps {
  /** The heading, as drawn. */
  title: ReactNode;
  /** The section starts open. Default: shut. */
  defaultOpen?: boolean;
  /** How many of its fields hold a value now: shown beside the heading while it is shut. */
  filled?: number;
  /** A problem is inside: the section is open and stays open. */
  forceOpen?: boolean;
  children: ReactNode;
}

export function FoldSection({
  title,
  defaultOpen = false,
  filled = 0,
  forceOpen = false,
  children,
}: FoldSectionProps) {
  const [kept, setKept] = useState(defaultOpen);
  const bodyId = useId();
  const open = kept || forceOpen;
  return (
    <section className="flex flex-col gap-3 rounded-3 border border-border bg-surface p-3">
      <button
        type="button"
        aria-expanded={open}
        aria-controls={bodyId}
        onClick={() => {
          if (!forceOpen) setKept(!kept);
        }}
        className="flex min-h-target items-center justify-between gap-3 text-start"
      >
        <span className="type-section text-ink">{title}</span>
        <span className="flex items-center gap-2 text-ink-muted">
          {filled > 0 && !open ? (
            <span className="rounded-2 bg-primary-soft px-2 type-label text-primary">
              <Num>{String(filled)}</Num>
            </span>
          ) : null}
          {open ? (
            <ChevronUp aria-hidden="true" className="size-icon-md" />
          ) : (
            <ChevronDown aria-hidden="true" className="size-icon-md" />
          )}
        </span>
      </button>
      {open ? <div id={bodyId}>{children}</div> : null}
    </section>
  );
}
