// A row of two or more joined choices, one pressed (design system 6, the language switch).
// 40 px tall, one border around the group, the pressed choice in primary.

import type { ReactNode } from 'react';

export interface SegmentedOption<T extends string> {
  value: T;
  label: ReactNode;
  /** Language of the label when it differs from the page, for example "English" on Urdu. */
  lang?: string;
}

interface SegmentedControlProps<T extends string> {
  options: readonly SegmentedOption<T>[];
  value: T;
  onChange: (value: T) => void;
  /** What the group is for, read by screen readers. Comes from Fluent. */
  groupLabel: string;
}

export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  groupLabel,
}: SegmentedControlProps<T>) {
  return (
    <div
      role="group"
      aria-label={groupLabel}
      className="inline-flex h-control-compact overflow-hidden rounded-2 border border-border-control"
    >
      {options.map((option) => {
        const pressed = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            lang={option.lang}
            aria-pressed={pressed}
            onClick={() => {
              onChange(option.value);
            }}
            className={`px-3 type-control ${pressed ? 'bg-primary text-on-primary' : 'bg-surface text-ink'}`}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
