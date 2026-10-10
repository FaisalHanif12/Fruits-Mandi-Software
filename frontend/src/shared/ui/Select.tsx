// A choice from a short fixed list, for example ذریعہ: نقد، بینک، چیک (design system 6). Long
// lists of records use pickers (AccountPicker and the master pickers), never this.
// Built on Radix Select: keyboard, focus and screen readers come from it, and the list opens
// on the side the direction says. The list renders in a portal, so it sets lang and dir again.

import { useLocalization } from '@fluent/react';
import { Check, ChevronDown } from 'lucide-react';
import { Select as RadixSelect } from 'radix-ui';
import type { ReactNode } from 'react';
import { useUiLanguage } from '../i18n';
import { controlClasses } from './control-classes';
import type { FieldControlProps } from './Field';

/** Gap between the field and its open list, in CSS pixels (Radix takes a number). */
const LIST_GAP = 4;

export interface SelectOption<T extends string> {
  value: T;
  label: ReactNode;
}

interface SelectProps<T extends string> extends FieldControlProps {
  value: T | undefined;
  onChange: (value: T) => void;
  options: readonly SelectOption<T>[];
  /** Shown while nothing is chosen. Default: "چنیں" / "Choose". */
  placeholder?: ReactNode;
}

export function Select<T extends string>({
  id,
  'aria-describedby': describedBy,
  invalid,
  readOnly,
  value,
  onChange,
  options,
  placeholder,
}: SelectProps<T>) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  return (
    <RadixSelect.Root
      dir={direction}
      // Radix uses '' for "nothing chosen"; undefined would switch it to uncontrolled.
      value={value ?? ''}
      disabled={readOnly === true}
      onValueChange={(next) => {
        const option = options.find((candidate) => candidate.value === next);
        if (option !== undefined) onChange(option.value);
      }}
    >
      <RadixSelect.Trigger
        id={id}
        aria-invalid={invalid}
        aria-describedby={describedBy}
        className={`${controlClasses({ invalid, readOnly })} flex items-center justify-between gap-2 type-field text-start disabled:cursor-default data-placeholder:text-ink-placeholder`}
      >
        <RadixSelect.Value
          placeholder={placeholder ?? l10n.getString('common-select-placeholder')}
        />
        {readOnly === true ? null : (
          <RadixSelect.Icon>
            <ChevronDown aria-hidden="true" className="size-icon-md shrink-0 text-ink-muted" />
          </RadixSelect.Icon>
        )}
      </RadixSelect.Trigger>
      <RadixSelect.Portal>
        <RadixSelect.Content
          lang={language}
          dir={direction}
          position="popper"
          sideOffset={LIST_GAP}
          className="z-50 max-h-(--radix-select-content-available-height) min-w-(--radix-select-trigger-width) overflow-hidden rounded-2 border border-border bg-surface p-1 shadow-overlay"
        >
          <RadixSelect.Viewport>
            {options.map((option) => (
              <RadixSelect.Item
                key={option.value}
                value={option.value}
                className="flex h-control cursor-default items-center justify-between gap-3 rounded-1 px-3 type-field text-ink outline-none data-highlighted:bg-primary-soft data-highlighted:text-primary"
              >
                <RadixSelect.ItemText>{option.label}</RadixSelect.ItemText>
                <RadixSelect.ItemIndicator>
                  <Check aria-hidden="true" className="size-icon-sm text-primary" />
                </RadixSelect.ItemIndicator>
              </RadixSelect.Item>
            ))}
          </RadixSelect.Viewport>
        </RadixSelect.Content>
      </RadixSelect.Portal>
    </RadixSelect.Root>
  );
}
