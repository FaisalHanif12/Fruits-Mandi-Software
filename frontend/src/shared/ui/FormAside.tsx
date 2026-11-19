// A form with its preview beside it (plan 38, forms): from 820 px of free width the form takes two
// thirds and the preview (what saving will do) one third, so the preview is on screen while the
// fields are typed; in a narrower box the preview goes under the form. The layout follows the free
// width of the box, not the window (a container query).
//
//   <FormAside form={<ExpenseForm .../>} aside={<ExpensePreviewCard .../>} />

import type { ReactNode } from 'react';

interface FormAsideProps {
  /** The fields. */
  form: ReactNode;
  /** The preview, the notes and what goes with them. */
  aside: ReactNode;
}

export function FormAside({ form, aside }: FormAsideProps) {
  return (
    <div className="@container min-w-0">
      <div className="grid grid-cols-1 gap-4 @beside:grid-cols-3">
        <div className="flex min-w-0 flex-col gap-4 @beside:col-span-2">{form}</div>
        <div className="flex min-w-0 flex-col gap-3">{aside}</div>
      </div>
    </div>
  );
}
