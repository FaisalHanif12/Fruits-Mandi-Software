// A label with a value that is shown, not typed (plan 13: a saved بکری opens read-only). The look is
// a Field without the box: the label above in the label style, the value below in the field text
// size. The value can be an amount (`Num`), a code, a name, or a few words. Several facts sit in a
// grid of the screen's own; nothing here works out anything (CLAUDE.md rule 1).

import type { ReactNode } from 'react';

interface FactProps {
  label: ReactNode;
  children: ReactNode;
}

export function Fact({ label, children }: FactProps) {
  return (
    <div className="flex min-w-0 flex-col gap-1">
      <dt className="type-label text-ink-muted">{label}</dt>
      <dd className="type-field min-w-0 break-words text-ink">{children}</dd>
    </div>
  );
}
