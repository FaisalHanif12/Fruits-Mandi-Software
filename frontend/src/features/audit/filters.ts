// What the filter boxes of the audit log hold, and how it becomes the filters the backend reads
// (PRD FR-21.3: the Owner can filter it). The boxes hold text as typed; the backend reads the dates
// and checks everything (CLAUDE.md rule 1). The record number is an id, typed as digits.

import type { AuditEventKind, AuditFilters } from './types';

/** The choice of a drop-down that means "any". Radix does not allow an empty value for a choice. */
export const ANY = '*';

/** The kinds of event the log can be filtered to (PRD FR-21.3 v1.61), in the order of the drop-down. */
export const AUDIT_KINDS: readonly AuditEventKind[] = [
  'edit',
  'cancel',
  'reopen',
  'wrong_code',
  'wrong_sign',
  'wrong_password',
  'lockout',
  'closed_day',
];

export interface AuditFilterState {
  /** A kind of event, or ANY (plan 100 D). */
  kind: string;
  /** Only the security events (wrong password, code or sign, a wait started, a closed day refused). */
  securityOnly: boolean;
  /** The user id as text, or ANY. */
  user: string;
  /** An action as the backend names it, or ANY. */
  action: string;
  /** An area as the backend names it, or ANY. */
  entity: string;
  /** The record number as digits, with an area. */
  entityId: string;
  /** `DD-MM-YYYY` as typed (the backend also reads `10` and `10-9`). */
  from: string;
  to: string;
  /** Words for the reason and the old and new values. */
  text: string;
}

export const EMPTY_AUDIT_FILTERS: AuditFilterState = {
  kind: ANY,
  securityOnly: false,
  user: ANY,
  action: ANY,
  entity: ANY,
  entityId: '',
  from: '',
  to: '',
  text: '',
};

/** A whole number typed as digits, or null when there is nothing (or nothing usable) typed. */
function idOf(text: string): number | null {
  const digits = text.trim();
  if (!/^\d{1,15}$/.test(digits)) return null;
  return Number.parseInt(digits, 10);
}

/** The kind chosen, or null for every kind. */
function kindToWire(state: AuditFilterState): AuditEventKind | null {
  return AUDIT_KINDS.find((kind) => kind === state.kind) ?? null;
}

/** The filters as the backend reads them. Every key is sent. */
export function filtersToWire(state: AuditFilterState): AuditFilters {
  return {
    user_id: state.user === ANY ? null : idOf(state.user),
    action: state.action === ANY ? '' : state.action,
    entity: state.entity === ANY ? '' : state.entity,
    entity_id: idOf(state.entityId),
    from: state.from.trim(),
    to: state.to.trim(),
    text: state.text.trim(),
    kind: kindToWire(state),
    security_only: state.securityOnly,
  };
}

/** True when no box is filled: nothing to clear. */
export function isUnfiltered(state: AuditFilterState): boolean {
  const wire = filtersToWire(state);
  return (
    wire.kind === null &&
    !wire.security_only &&
    wire.user_id === null &&
    wire.action === '' &&
    wire.entity === '' &&
    wire.entity_id === null &&
    wire.from === '' &&
    wire.to === '' &&
    wire.text === ''
  );
}
