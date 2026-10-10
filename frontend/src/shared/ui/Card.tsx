// A white card on the page ground (design system 6): 1 px border, 12 px radius, 24 px padding.
// No coloured side borders.

import type { ReactNode } from 'react';

interface CardProps {
  /** Optional card title, step 3 of the type scale. */
  title?: ReactNode;
  children: ReactNode;
}

export function Card({ title, children }: CardProps) {
  return (
    <section className="flex min-w-0 flex-col gap-4 rounded-3 border border-border bg-surface p-5">
      {title === undefined ? null : <h2 className="type-section text-ink">{title}</h2>}
      {children}
    </section>
  );
}
