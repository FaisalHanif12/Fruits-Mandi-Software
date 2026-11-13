// A row of two or more joined choices, one pressed (design system 6, the language switch).
// 40 px tall, one border around the group, the pressed choice in primary.

import type { ReactNode } from 'react';
import { HoverDetails } from './HoverDetails';

export interface SegmentedOption<T extends string> {
  value: T;
  label: ReactNode;
  /** Language of the label when it differs from the page, for example "English" on Urdu. */
  lang?: string;
  /**
   * What this choice does, in a card that opens beside it when it is pointed at or reached with the keyboard
   * (plan 75 §7.6, `HoverDetails`). The buttons keep their order and their keys; the card only adds to them.
   * The card brings its own padding (`p-4`).
   */
  details?: ReactNode;
  /** The name of that card for a screen reader, for example "About the shop expense". Comes from Fluent. */
  detailsLabel?: string;
}

interface SegmentedControlProps<T extends string> {
  options: readonly SegmentedOption<T>[];
  value: T;
  onChange: (value: T) => void;
  /** What the group is for, read by screen readers. Comes from Fluent. */
  groupLabel: string;
  /** The choice cannot be changed now (the type of a voucher that is being edited). */
  disabled?: boolean | undefined;
}

export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  groupLabel,
  disabled = false,
}: SegmentedControlProps<T>) {
  return (
    <div
      role="group"
      aria-label={groupLabel}
      className="inline-flex h-control-compact overflow-hidden rounded-2 border border-border-control"
    >
      {options.map((option) => {
        const pressed = option.value === value;
        const button = (
          <button
            key={option.value}
            type="button"
            lang={option.lang}
            aria-pressed={pressed}
            disabled={disabled}
            onClick={() => {
              onChange(option.value);
            }}
            className={`px-3 type-control disabled:cursor-default ${pressed ? 'bg-primary text-on-primary' : 'bg-surface text-ink'} ${disabled && !pressed ? 'text-ink-muted' : ''}`}
          >
            {option.label}
          </button>
        );
        return option.details === undefined ? (
          button
        ) : (
          <HoverDetails key={option.value} label={option.detailsLabel ?? ''} card={option.details}>
            {button}
          </HoverDetails>
        );
      })}
    </div>
  );
}
