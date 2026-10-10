// A code, number, phone or truck number inside text (Code Standards F25, NFR-URD-13).
// Isolated and always left to right, so "LES-1234" or "0.1.0" never flips inside Urdu.

import type { ReactNode } from 'react';

interface CodeProps {
  /** Optional: Fluent fills it when the component is used in `<Localized elems>`. */
  children?: ReactNode;
}

export function Code({ children }: CodeProps) {
  return (
    <bdi dir="ltr" className="type-number">
      {children}
    </bdi>
  );
}
