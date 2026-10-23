// The words of the audit log (PRD FR-21.3). The backend names an action and an area with a short
// code; the ones the app knows have words in both languages (audit.ftl), and a code it does not know
// is shown as it is rather than hidden. A changed field is shown by its path, as the backend names
// it, because that is the exact name of the field.

import type { ListContext } from '../../shared/grid';
import type { AuditValue } from './types';

/** The actions that have words (audit.ftl `audit-action-<code>`). */
export const KNOWN_ACTIONS = [
  'create',
  'edit',
  'cancel',
  'login',
  'login_failed',
  'logout',
] as const;

/** The areas that have words (audit.ftl `audit-entity-<code>`). */
export const KNOWN_ENTITIES = [
  'settings',
  'backup_settings',
  'day_settings',
  'user',
  'account',
  'bikri',
  'voucher',
  'rule',
  'partner',
] as const;

/** The Fluent message ID of a code: `login_failed` is `login-failed`. */
function idPart(code: string): string {
  return code.replaceAll('_', '-');
}

/** The message ID for an action, or null when the app has no words for it. */
export function actionMessageId(code: string): string | null {
  return (KNOWN_ACTIONS as readonly string[]).includes(code)
    ? `audit-action-${idPart(code)}`
    : null;
}

/** The message ID for an area, or null when the app has no words for it. */
export function entityMessageId(code: string): string | null {
  return (KNOWN_ENTITIES as readonly string[]).includes(code)
    ? `audit-entity-${idPart(code)}`
    : null;
}

type Text = ListContext['text'];

export function actionWord(code: string, text: Text): string {
  const id = actionMessageId(code);
  return id === null ? code : text(id);
}

export function entityWord(code: string, text: Text): string {
  const id = entityMessageId(code);
  return id === null ? code : text(id);
}

/**
 * A value of a changed field as text. Money is the backend's amount through the list's money
 * formatter; a secret is never a value, only the word that it changed (the backend sends `hidden`).
 */
export function valueText(value: AuditValue, context: ListContext): string {
  switch (value.kind) {
    case 'null':
      return context.text('audit-value-none');
    case 'bool':
      return context.text(value.value ? 'audit-value-yes' : 'audit-value-no');
    case 'number':
      return String(value.value);
    case 'money':
      return context.money(value.paisa);
    case 'text':
      return value.text;
    case 'hidden':
      return context.text('audit-value-hidden');
  }
}
