// Plain text: notes, free text, user names (design system 6). Urdu name fields will use
// RomanUrduInput (plan 04 onwards), which is built on this look.

import { controlClasses } from './control-classes';
import type { FieldControlProps } from './Field';

interface TextInputProps extends FieldControlProps {
  value: string;
  onChange: (text: string) => void;
  /** Grey text shown while empty. Comes from Fluent. */
  placeholder?: string;
  /** Set when the text is in the other language, for example an English name on an Urdu screen. */
  lang?: string | undefined;
}

export function TextInput({
  id,
  'aria-describedby': describedBy,
  invalid,
  readOnly,
  value,
  onChange,
  placeholder,
  lang,
}: TextInputProps) {
  return (
    <input
      id={id}
      type="text"
      lang={lang}
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
      className={`${controlClasses({ invalid, readOnly })} type-field`}
    />
  );
}
