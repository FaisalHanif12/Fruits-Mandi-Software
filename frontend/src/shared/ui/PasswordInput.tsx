// A password field with a "show password" eye (PRD Section 8.1, design system 6). Same look as
// every field. The text is never kept anywhere but in the screen's form state.

import { useLocalization } from '@fluent/react';
import { Eye, EyeOff } from 'lucide-react';
import { useState } from 'react';
import { controlClasses } from './control-classes';
import type { FieldControlProps } from './Field';

interface PasswordInputProps extends FieldControlProps {
  value: string;
  onChange: (text: string) => void;
  /** For the browser's own helpers: 'current-password' or 'new-password'. */
  autoComplete?: 'current-password' | 'new-password';
}

export function PasswordInput({
  id,
  'aria-describedby': describedBy,
  invalid,
  readOnly,
  value,
  onChange,
  autoComplete = 'current-password',
}: PasswordInputProps) {
  const { l10n } = useLocalization();
  const [shown, setShown] = useState(false);
  return (
    <div className="relative">
      <input
        id={id}
        type={shown ? 'text' : 'password'}
        dir="ltr"
        lang="en"
        value={value}
        readOnly={readOnly}
        aria-invalid={invalid}
        aria-describedby={describedBy}
        autoComplete={autoComplete}
        spellCheck={false}
        onChange={(event) => {
          onChange(event.target.value);
        }}
        className={`${controlClasses({ invalid, readOnly })} type-number-field pe-12`}
      />
      <button
        type="button"
        tabIndex={-1}
        aria-label={l10n.getString(shown ? 'common-password-hide' : 'common-password-show')}
        aria-pressed={shown}
        onClick={() => {
          setShown((current) => !current);
        }}
        className="absolute inset-y-0 end-0 flex w-control items-center justify-center text-ink-muted"
      >
        {shown ? (
          <EyeOff aria-hidden="true" className="size-icon-md" />
        ) : (
          <Eye aria-hidden="true" className="size-icon-md" />
        )}
      </button>
    </div>
  );
}
