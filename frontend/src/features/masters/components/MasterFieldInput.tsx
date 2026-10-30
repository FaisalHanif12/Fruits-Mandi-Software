// One field of a master's form: a text, a Latin text (a phone, a weight), a choice, a record of
// another list or an account (plan 06, plan 41). The full Index form and the short form of "add it
// here" draw their fields with this one piece, so a field looks and works the same in both.
// The ticked choices of a narration (`multi`) are the full form's own.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import type { SearchHit } from '../../../shared/search';
import { AccountPicker, Field, Picker, Select, TextInput, fieldError } from '../../../shared/ui';
import type { FieldSpec } from '../configs';
import type { FormValues } from '../draft';

/** The path the backend names a field by when it refuses it. */
export function fieldPath(spec: FieldSpec): string {
  switch (spec.control.type) {
    case 'picker':
      return `${spec.key}_id`;
    case 'account':
      return `${spec.key}_account_id`;
    default:
      return spec.key;
  }
}

type PickedHit = Pick<SearchHit, 'id' | 'code' | 'name_ur' | 'name_en'>;

interface MasterFieldInputProps {
  spec: FieldSpec;
  values: FormValues;
  /** What the backend refused, to show under the field it points at. */
  error: UiError | null;
  onText: (key: string, text: string) => void;
  onRef: (key: string, hit: PickedHit | null) => void;
}

export function MasterFieldInput({ spec, values, error, onText, onRef }: MasterFieldInputProps) {
  const kind = spec.control;
  if (kind.type === 'multi') return null;
  return (
    <Field
      label={<Localized id={spec.labelId} />}
      hint={spec.hintId === undefined ? undefined : <Localized id={spec.hintId} />}
      error={fieldError(error, fieldPath(spec))}
    >
      {(props) => {
        switch (kind.type) {
          case 'text':
            return (
              <TextInput
                {...props}
                value={values.text[spec.key] ?? ''}
                onChange={(text) => {
                  onText(spec.key, text);
                }}
              />
            );
          case 'latin':
            return (
              <TextInput
                {...props}
                value={values.text[spec.key] ?? ''}
                onChange={(text) => {
                  onText(spec.key, text);
                }}
                lang="en"
              />
            );
          case 'select':
            return (
              <Select
                {...props}
                value={values.text[spec.key]}
                onChange={(text) => {
                  onText(spec.key, text);
                }}
                options={kind.options.map((option) => ({
                  value: option.value,
                  label: <Localized id={option.labelId} />,
                }))}
              />
            );
          case 'picker':
            return (
              <Picker
                {...props}
                scope={kind.scope}
                value={values.refs[spec.key] ?? null}
                onChange={(hit) => {
                  onRef(spec.key, hit);
                }}
              />
            );
          case 'account':
            return (
              <AccountPicker
                {...props}
                scope={kind.scope}
                value={values.refs[spec.key] ?? null}
                onChange={(hit) => {
                  onRef(spec.key, hit);
                }}
              />
            );
        }
      }}
    </Field>
  );
}
