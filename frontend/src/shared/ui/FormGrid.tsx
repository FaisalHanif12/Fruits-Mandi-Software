// The columns of a form or a filter bar (plan 38 G1, Code Standards F15). The number of columns follows
// the free width of the box the form is in (a container query), not the width of the window: a form
// next to a side menu at 1024 px still gets 2 or 3 columns when there is room. Short fields sit side
// by side, so the form is not twice as tall as it needs to be.
//
//   <FormGrid columns={3}> <Field .../> <Field .../> <FormSpan>...</FormSpan> </FormGrid>
//
// A field that needs the whole row (a long note) goes in `<FormSpan>`.

import type { ReactNode } from 'react';

/** How many columns the grid reaches at most; it uses fewer when the box is narrow. */
type Columns = 2 | 3 | 4;

const COLUMN_CLASSES: Record<Columns, string> = {
  2: 'grid grid-cols-1 gap-x-4 gap-y-3 @form2:grid-cols-2',
  3: 'grid grid-cols-1 gap-x-4 gap-y-3 @form2:grid-cols-2 @form3:grid-cols-3',
  4: 'grid grid-cols-1 gap-x-4 gap-y-3 @form2:grid-cols-2 @form3:grid-cols-3 @form4:grid-cols-4',
};

interface FormGridProps {
  columns?: Columns;
  /** Fields that sit at the bottom of their row (a button beside fields with labels). */
  alignEnd?: boolean;
  children: ReactNode;
}

export function FormGrid({ columns = 2, alignEnd = false, children }: FormGridProps) {
  return (
    <div className="@container min-w-0">
      <div className={`${COLUMN_CLASSES[columns]} ${alignEnd ? 'items-end' : ''}`}>{children}</div>
    </div>
  );
}

interface FormSpanProps {
  /** `all`: the whole row. `two`: two columns when there are at least two. */
  span?: 'all' | 'two';
  children: ReactNode;
}

/** A cell of the grid that takes more than one column. */
export function FormSpan({ span = 'all', children }: FormSpanProps) {
  return (
    <div className={span === 'all' ? 'col-span-full min-w-0' : '@form2:col-span-2 min-w-0'}>
      {children}
    </div>
  );
}
