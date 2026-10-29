// The card that holds the fields of an entry form (plan 38 G1, Code Standards F15): a bordered card
// with a `FormGrid` inside, so its columns follow the free width of the card, not the window. Two
// columns from 480 px, three from 640 px when it asks for three. A field that needs the whole row
// goes in `<FormSpan>`.

import type { ReactNode } from 'react';
import { FormGrid } from './FormGrid';

interface FormCardProps {
  /** How many columns the card reaches at most (default 2). */
  columns?: 2 | 3 | 4;
  children: ReactNode;
}

export function FormCard({ columns = 2, children }: FormCardProps) {
  return (
    <div className="rounded-3 border border-border bg-surface p-3">
      <FormGrid columns={columns}>{children}</FormGrid>
    </div>
  );
}
