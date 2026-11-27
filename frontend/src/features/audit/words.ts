// The words of the audit log (PRD FR-21.3). The backend names an action and an area with a short
// code; the ones the app knows have words in both languages (audit.ftl), and a code it does not know
// is shown as it is rather than hidden. A changed field is shown by its path, as the backend names
// it, because that is the exact name of the field.

import type { ListContext } from '../../shared/grid';
import type { AuditEventKind, AuditValue } from './types';

/** The actions that have words (audit.ftl `audit-action-<code>`). */
export const KNOWN_ACTIONS = [
  'create',
  'edit',
  'cancel',
  'login',
  'login_failed',
  'logout',
  // The day close and the month close (PRD FR-21.1, FR-21.2, FR-17.6).
  'day_close',
  'day_auto_close',
  'day_reopen',
  'month_close',
  'month_close_reverse',
  // The rest of what the backend writes (backend-d5, 25-10-2026).
  'unlock',
  'lock',
  'recover',
  'change_password',
  'make_regular',
  'update',
  'learn',
  'remove',
  'set_shares',
  'later_sale',
  'print',
  'write_off',
  'write_off_undo',
  'confirm',
  'check_failed',
  'restore',
  // The dead amount (PRD FR-10.10, plan 40): a person moved to it, and made active again.
  'dead_mark',
  'dead_undo',
  // Plan 100 D: every other action the backend writes, and the security events (wrong password, code or
  // sign, a wait started, a save refused on a closed day).
  'change_split',
  'close',
  'reopen',
  'day_fix_open',
  'day_fix_close',
  'mixed_save',
  'mixed_undo',
  'opening_moved_to_cash',
  'opening_on_closed_day',
  'set_sell_date',
  'reveal',
  'reveal_refused',
  'partner_code_failed',
  'partner_code_set',
  'partner_code_change',
  'partner_code_reset',
  'partner_unlock',
  'assistant_key_save',
  'assistant_key_replace',
  'assistant_key_remove',
  'day_reopen_sign_wrong',
  'day_fix_sign_wrong',
  'voucher_edit_sign_wrong',
  'owner_sign_change_sign_wrong',
  'garden_rent_sign_wrong',
  'openings_to_cash_sign_wrong',
  'cancel_sign_wrong',
  'lockout_started',
  'closed_day_refused',
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
  'day',
  'month',
  'backup_cloud_keys',
  'backup_key',
  'backup',
  'health',
  'staff',
  'city',
  'item',
  'packing',
  'adda',
  'marka',
  'store',
  'narration',
  'expense_head',
  'bank_account',
  'charge_rule',
  'translit_pair',
  'bikri_lot',
  'trip',
  'purchase',
  'trip_cost',
  'trip_other_sale',
  'assistant_key',
  'chukta_buyer',
  'collection_list',
  'fruit_take',
  'garden',
  'garden_arrival',
  'garden_entry',
  'garden_installment',
  'partner_code',
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

/** The words of each kind of event (a new kind from the backend must be added here to build). */
const KIND_IDS: Record<AuditEventKind, string> = {
  edit: 'audit-kind-edit',
  cancel: 'audit-kind-cancel',
  reopen: 'audit-kind-reopen',
  wrong_code: 'audit-kind-wrong-code',
  wrong_sign: 'audit-kind-wrong-sign',
  wrong_password: 'audit-kind-wrong-password',
  lockout: 'audit-kind-lockout',
  closed_day: 'audit-kind-closed-day',
};

/** The message ID of a kind of event (`audit-kind-wrong-code`). */
export function kindMessageId(kind: AuditEventKind): string {
  return KIND_IDS[kind];
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
