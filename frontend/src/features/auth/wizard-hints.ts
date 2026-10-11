// The on-screen hints of the first-run wizard (Code Standards 4.1 layer 1). They stop an empty or
// wrong-script step from being sent on, and tell the user at once which field. The backend still
// checks everything when the setup is made and is the final word (plan 04 contract 4).

import { requiredHintId, scriptHintId, type Script } from '../../shared/text/scriptHint';
import { STEP_FIELDS, type WizardField, type WizardValues } from './wizard-store';

/** The fields the setup cannot do without. Phone and address may stay empty. */
const REQUIRED: readonly WizardField[] = [
  'firm_name_en',
  'firm_name_ur',
  'mandi_en',
  'mandi_ur',
  'owner_display_name',
  'owner_username',
  'password',
  'password_repeat',
];

/** The script a name field must be written in. */
export const FIELD_SCRIPT: Partial<Record<WizardField, Script>> = {
  firm_name_ur: 'ur',
  mandi_ur: 'ur',
  firm_name_en: 'en',
  mandi_en: 'en',
};

/**
 * The message ID to show on a field, or null. A wrong script shows at once, while typing. An empty
 * required field shows only after the user has tried to go on (`tried`), so a fresh form is calm.
 */
export function hintFor(field: WizardField, value: string, tried: boolean): string | null {
  const script = FIELD_SCRIPT[field];
  const wrongScript = script === undefined ? null : scriptHintId(value, script);
  if (wrongScript !== null) return wrongScript;
  if (tried && REQUIRED.includes(field)) return requiredHintId(value);
  return null;
}

/** The first field of a step that has a hint when the user tries to go on, or null if none. */
export function firstHintedField(step: number, values: WizardValues): WizardField | null {
  const fields = STEP_FIELDS[step] ?? [];
  return fields.find((field) => hintFor(field, values[field], true) !== null) ?? null;
}
