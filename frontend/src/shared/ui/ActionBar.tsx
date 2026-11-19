// The bar of a long form's buttons (plan 38, account form): it sits under the scrolling part of the
// screen, never over the fields, so Save is always in view however far the form is scrolled. The
// screen makes its root `flex h-full min-h-0 flex-col`, the form `min-h-0 flex-1 overflow-auto`, and
// puts this bar last. Buttons are in the usual order, the primary one last (Code Standards F12).

import type { ReactNode } from 'react';

interface ActionBarProps {
  children: ReactNode;
}

export function ActionBar({ children }: ActionBarProps) {
  return (
    <div className="flex shrink-0 flex-wrap items-center justify-end gap-2 border-t border-border bg-bg pt-3">
      {children}
    </div>
  );
}
