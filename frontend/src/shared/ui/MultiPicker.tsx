// A field for one or several records of a master list, for example the items of a garden (PRD 12.11, plan 78 D2,
// Umar 17-11-2026): the chosen ones are listed above the picker, each with a button that removes it, and the picker
// adds the next. The picker always starts empty after a choice, so it is the same field as every other picker
// (search by name or code, the add row, the keyboard of `Picker`). The records are a screen's own state (Code
// Standards F39); problems the backend names by the field path of each record (`item_ids[1]`) show under the field.

import { Localized, useLocalization } from '@fluent/react';
import { X } from 'lucide-react';
import { displayName, useLanguage } from '../i18n';
import type { SearchScope } from '../search';
import type { FieldControlProps } from './Field';
import { Picker, type PickerValue } from './Picker';

interface MultiPickerProps extends FieldControlProps {
  /** Which list to search: a master kind. */
  scope: SearchScope;
  values: readonly PickerValue[];
  onChange: (values: PickerValue[]) => void;
  /** Turns the record a picker gave back into a value of this list (the same one every form of the screen uses). */
  valueOf: (hit: { id: number; code: string; name_ur: string; name_en: string }) => PickerValue;
}

export function MultiPicker({ scope, values, onChange, valueOf, ...control }: MultiPickerProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  return (
    <div className="flex min-w-0 flex-col gap-2">
      {values.length === 0 ? null : (
        <ul className="flex flex-wrap gap-2">
          {values.map((value) => {
            const name = displayName(value, language);
            return (
              <li
                key={value.id}
                className="inline-flex items-center gap-1 rounded-2 border border-border bg-surface-sunken ps-3 pe-1 type-body text-ink"
              >
                <span lang={language}>{name}</span>
                <button
                  type="button"
                  aria-label={l10n.getString('ui-multi-remove', { name })}
                  title={l10n.getString('ui-multi-remove', { name })}
                  onClick={() => {
                    onChange(values.filter((candidate) => candidate.id !== value.id));
                  }}
                  className="inline-flex size-control-compact items-center justify-center rounded-1 text-ink-muted transition-calm hover:bg-border"
                >
                  <X aria-hidden="true" className="size-icon-sm" />
                </button>
              </li>
            );
          })}
        </ul>
      )}
      <Picker
        {...control}
        scope={scope}
        value={null}
        onChange={(hit) => {
          if (hit === null || values.some((candidate) => candidate.id === hit.id)) return;
          onChange([...values, valueOf(hit)]);
        }}
      />
      <Localized id="ui-multi-hint">
        <p className="type-body text-ink-muted" />
      </Localized>
    </div>
  );
}
