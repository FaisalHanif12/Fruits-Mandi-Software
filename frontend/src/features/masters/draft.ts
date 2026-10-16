// The form of a master as text in the screen's state (Code Standards F39): a new draft, a draft
// from a saved record, and whether anything has changed since. Nothing here works out money.

import type { Language } from '../../i18n';
import { displayName } from '../../shared/i18n';
import type { NamePairValue } from '../../shared/ui';
import type { MasterConfig } from './configs';
import type { MasterDraft, MasterRecord } from './types';

/** The names of a draft with the spelling marks the name pair keeps next to them. */
export interface FormValues extends Omit<MasterDraft, 'names'> {
  names: NamePairValue;
}

/** A new master: the suggested code, empty names, and the first choice of every select. */
export function newValues(config: MasterConfig, code: string): FormValues {
  const text: Record<string, string> = {};
  const lists: Record<string, readonly string[]> = {};
  for (const field of config.fields) {
    const { type } = field.control;
    if (type === 'select') text[field.key] = field.control.options[0]?.value ?? '';
    else if (type === 'multi') lists[field.key] = [];
    else if (type !== 'picker' && type !== 'account') text[field.key] = '';
  }
  return {
    id: null,
    version: null,
    code,
    names: { urdu: '', english: '' },
    text,
    refs: Object.fromEntries(
      config.fields
        .filter((f) => f.control.type === 'picker' || f.control.type === 'account')
        .map((f) => [f.key, null]),
    ),
    lists,
  };
}

/** The form of a saved master, exactly as it is saved. */
export function valuesOf(config: MasterConfig, record: MasterRecord): FormValues {
  const empty = newValues(config, record.code);
  return {
    id: record.id,
    version: record.version,
    code: record.code,
    names:
      record.names === null
        ? { urdu: '', english: '' }
        : { urdu: record.names.name_ur, english: record.names.name_en },
    text: { ...empty.text, ...record.text },
    refs: { ...empty.refs, ...record.refs },
    lists: { ...empty.lists, ...record.lists },
  };
}

/** The same ticked choices, whatever the order they were ticked in. */
function sameChoices(a: readonly string[] = [], b: readonly string[] = []): boolean {
  return a.length === b.length && a.every((choice) => b.includes(choice));
}

/** True when nothing differs: the same code, names, texts, ticked choices and chosen records. */
export function sameValues(a: FormValues, b: FormValues): boolean {
  const keys = new Set([...Object.keys(a.text), ...Object.keys(b.text)]);
  const refKeys = new Set([...Object.keys(a.refs), ...Object.keys(b.refs)]);
  const listKeys = new Set([...Object.keys(a.lists), ...Object.keys(b.lists)]);
  return (
    a.code === b.code &&
    [...listKeys].every((key) => sameChoices(a.lists[key], b.lists[key])) &&
    a.names.urdu === b.names.urdu &&
    a.names.english === b.names.english &&
    [...keys].every((key) => (a.text[key] ?? '') === (b.text[key] ?? '')) &&
    [...refKeys].every((key) => (a.refs[key]?.id ?? null) === (b.refs[key]?.id ?? null))
  );
}

/** What to call a master in a heading or a question: its name, else its mark text, else its code. */
export function labelOf(record: MasterRecord, language: Language): string {
  if (record.names !== null) return displayName(record.names, language);
  return record.text['mark_text'] ?? record.code;
}
