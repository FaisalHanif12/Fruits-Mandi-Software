// The field for free text: notes, narrations, reasons, addresses (PRD FR-7.22, FR-7.25, Code
// Standards F41). It is the shared Urdu text field with the starting mode chosen by the app
// language: in the Urdu app it starts in رومن mode (English letters become Urdu); in the English
// app it starts in EN mode (no conversion), and is written left to right. Alt+U switches the mode.
// Its words are spelt as everyday words (`kind` word, plan 99); a free field that holds a person, a
// shop or a market says `name` or `place`, so names come first there.
//
// Free text is stored once, exactly as typed. It has no spelling check (FR-7.66), so a word that
// came from the spelling rules alone is never marked here.

import { useState } from 'react';
import type { Kind } from '../../bindings/Kind';
import { useLanguage } from '../i18n';
import type { TypingMode } from '../urdu/typing-mode';
import { RomanUrduInput } from './RomanUrduInput';
import type { FieldControlProps } from './Field';

interface FreeTextInputProps extends FieldControlProps {
  value: string;
  onChange: (text: string) => void;
  placeholder?: string | undefined;
  /** What the text holds. Default: `word` (everyday words); `name` or `place` for such a field. */
  kind?: Extract<Kind, 'word' | 'name' | 'place'> | undefined;
}

export function FreeTextInput({
  value,
  onChange,
  placeholder,
  kind = 'word',
  ...field
}: FreeTextInputProps) {
  const language = useLanguage();
  // The mode a user picks stays for the life of the field; a language switch does not change it.
  const [mode, setMode] = useState<TypingMode>(language === 'ur' ? 'roman' : 'english');
  return (
    <RomanUrduInput
      {...field}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      kind={kind}
      mode={mode}
      onModeChange={setMode}
      direction={mode === 'english' && language === 'en' ? 'ltr' : 'rtl'}
    />
  );
}
