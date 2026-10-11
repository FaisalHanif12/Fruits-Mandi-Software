// A tick box with its words beside it (design system 6). The whole row is the click target
// (NFR-USE-23), and the box is a real input, so the keyboard (Space) and screen readers work.

import type { ReactNode } from 'react';

interface CheckboxProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: ReactNode;
}

export function Checkbox({ checked, onChange, label }: CheckboxProps) {
  return (
    <label className="type-body flex min-h-control items-center gap-3">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => {
          onChange(event.target.checked);
        }}
        className="size-icon-md shrink-0"
      />
      <span>{label}</span>
    </label>
  );
}
