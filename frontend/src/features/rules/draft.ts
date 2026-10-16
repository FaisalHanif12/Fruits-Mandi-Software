// The rule form as text in the screen's state (Code Standards F39): a new draft, a draft from a
// saved rule, and whether anything changed. Nothing here works out money or a rate: the value is
// the text the backend sent, shown as written, and typed back as text.

import { isoToDisplayDate } from '../../shared/format';
import { chargesOf, methodsOf } from './allowed';
import type { RuleDraft, RuleRow } from './types';

/** A new rule: the first charge of the side, its first method, today as the start date. */
export function newRuleDraft(today: string): RuleDraft {
  return {
    id: null,
    version: null,
    side: 'seller',
    charge_type: 'commission',
    account: null,
    item: null,
    item_group: '',
    method: 'percent',
    value: '',
    from_date: today === '' ? '' : isoToDisplayDate(today),
    to_date: '',
    is_active: true,
    note: '',
  };
}

export function ruleDraftOf(row: RuleRow): RuleDraft {
  return {
    id: row.id,
    version: row.version,
    side: row.side,
    charge_type: row.charge_type,
    account: row.account,
    item: row.item,
    item_group: row.item_group ?? '',
    method: row.method,
    value: row.value_text,
    from_date: isoToDisplayDate(row.from_date),
    to_date: row.to_date === null ? '' : isoToDisplayDate(row.to_date),
    is_active: row.is_active,
    note: row.note,
  };
}

/** The draft after the side changed: a charge the new side does not have, or a method the charge
 * does not allow, moves to the first one that is allowed (PRD 11.2). */
export function withSide(draft: RuleDraft, side: RuleDraft['side']): RuleDraft {
  const charge = chargesOf(side).includes(draft.charge_type)
    ? draft.charge_type
    : (chargesOf(side)[0] ?? draft.charge_type);
  return withCharge({ ...draft, side, account: null }, charge);
}

export function withCharge(draft: RuleDraft, charge: RuleDraft['charge_type']): RuleDraft {
  const methods = methodsOf(draft.side, charge);
  const method = methods.includes(draft.method) ? draft.method : (methods[0] ?? draft.method);
  return { ...draft, charge_type: charge, method };
}

const TEXT_FIELDS = ['value', 'from_date', 'to_date', 'note', 'item_group'] as const;

export function sameRuleDraft(a: RuleDraft, b: RuleDraft): boolean {
  return (
    a.side === b.side &&
    a.charge_type === b.charge_type &&
    a.method === b.method &&
    a.is_active === b.is_active &&
    (a.account?.id ?? null) === (b.account?.id ?? null) &&
    (a.item?.id ?? null) === (b.item?.id ?? null) &&
    TEXT_FIELDS.every((field) => a[field] === b[field])
  );
}
