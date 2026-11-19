// A button that looks like a link, for a quiet action inside a form (design system 6): "Forgot the
// password?". It is a real button, so the keyboard and screen readers treat it as one. It is at
// least 32 px high, so it is easy to hit with the mouse (NFR-USE-23).

import type { ReactNode } from 'react';

interface LinkButtonProps {
  children: ReactNode;
  onClick: () => void;
}

export function LinkButton({ children, onClick }: LinkButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="type-body inline-flex min-h-target items-center self-start rounded-1 text-start text-primary underline transition-calm"
    >
      {children}
    </button>
  );
}
