// What the audit log remembers for the Owner (PRD FR-7.45): the filters he last used, kept as one
// text under the screen name `audit.filters`. The filters stay in the store while the app runs; this
// puts them back once when the log first opens, and keeps each change. A value that cannot be read
// any more is left out and the box starts empty. Nothing about money is kept.

import { isRecord, textOr } from '../../shared/screen-state';
import { ANY, EMPTY_AUDIT_FILTERS, type AuditFilterState } from './filters';

/** The name under which the audit filters are kept. */
export const AUDIT_SCREEN = 'audit.filters';

function short(value: unknown, most: number, fallback: string): string {
  const text = textOr(value, fallback);
  return text.length <= most ? text : fallback;
}

/** The filters to keep, or null when nothing is chosen. */
export function rememberedAudit(state: AuditFilterState): Record<string, unknown> | null {
  const base = EMPTY_AUDIT_FILTERS;
  const kept: Record<string, unknown> = {};
  if (state.user !== base.user) kept['user'] = state.user;
  if (state.action !== base.action) kept['action'] = state.action;
  if (state.entity !== base.entity) kept['entity'] = state.entity;
  if (state.entityId !== '') kept['entityId'] = state.entityId;
  if (state.from !== '') kept['from'] = state.from;
  if (state.to !== '') kept['to'] = state.to;
  if (state.text !== '') kept['text'] = state.text;
  return Object.keys(kept).length === 0 ? null : kept;
}

/** The filters a text kept, or null when it holds nothing this screen can read. */
export function auditFromRemembered(raw: unknown): AuditFilterState | null {
  if (!isRecord(raw)) return null;
  return {
    user: short(raw['user'], 20, ANY),
    action: short(raw['action'], 60, ANY),
    entity: short(raw['entity'], 60, ANY),
    entityId: short(raw['entityId'], 15, ''),
    from: short(raw['from'], 20, ''),
    to: short(raw['to'], 20, ''),
    text: short(raw['text'], 200, ''),
  };
}
