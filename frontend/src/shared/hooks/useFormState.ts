import { useCallback, useState } from 'react';

/**
 * The text of a form's fields, by field name, with a setter per field. Everything is text, as
 * typed (CLAUDE.md rule 1); the backend checks it. The state sits in the screen, above any grid
 * or portal, so a language switch loses nothing (Code Standards F39).
 */
export function useFormState<T extends Record<string, string>>(initial: T) {
  const [values, setValues] = useState<T>(initial);
  const setField = useCallback(
    (field: keyof T) => (text: string) => {
      setValues((current) => ({ ...current, [field]: text }));
    },
    [],
  );
  const reset = useCallback(() => {
    setValues(initial);
  }, [initial]);
  return { values, setField, setValues, reset };
}
