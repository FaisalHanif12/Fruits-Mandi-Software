// The tag (نشان) of an account (PRD 10.2): 1 to 4 English capital letters and dots, for example
// K, BA, GM or D.P. It is stored once as typed, so it is never converted to Urdu and never goes
// through RomanUrduInput. Letters are made capital when the user leaves the field, so he sees
// what will be saved; the backend makes them capital and checks them again (`ERR_TAG_INVALID`).

import { controlClasses } from './control-classes';
import type { FieldControlProps } from './Field';

interface TagInputProps extends FieldControlProps {
  value: string;
  onChange: (text: string) => void;
  /** Grey text shown while empty. Comes from Fluent. */
  placeholder?: string | undefined;
}

export function TagInput({
  id,
  'aria-describedby': describedBy,
  invalid,
  readOnly,
  value,
  onChange,
  placeholder,
}: TagInputProps) {
  return (
    <input
      id={id}
      type="text"
      lang="en"
      dir="ltr"
      value={value}
      placeholder={placeholder}
      readOnly={readOnly}
      aria-invalid={invalid}
      aria-describedby={describedBy}
      autoComplete="off"
      spellCheck={false}
      onChange={(event) => {
        onChange(event.target.value);
      }}
      onBlur={(event) => {
        const capital = event.target.value.trim().toUpperCase();
        if (capital !== event.target.value) onChange(capital);
      }}
      className={`${controlClasses({ invalid, readOnly })} type-field text-start`}
    />
  );
}
