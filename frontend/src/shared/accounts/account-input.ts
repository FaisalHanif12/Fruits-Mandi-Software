// The account form's text as the backend takes it (plan 07, plan 15). Kept apart from api.ts,
// which only wraps commands, because this only maps one shape to another.

import type { AccountInput } from '../../bindings/AccountInput';
import type { AccountDraft } from './types';

/**
 * What the backend takes for one account, from the account form's text. Money and the date go as
 * typed; the backend parses them and checks every field again. The quick add, the account form and
 * "make regular" (PRD FR-13.8) all send the account this way.
 */
export function accountInputOf(
  draft: AccountDraft,
  confirmedRulesWords: readonly string[],
): AccountInput {
  return {
    id: draft.id,
    version: draft.version,
    account_type: draft.account_type,
    control_id: draft.control_id,
    code: draft.code,
    name_ur: draft.names.urdu,
    name_en: draft.names.english,
    tag: draft.tag,
    father_or_firm_ur: draft.father.urdu,
    father_or_firm_en: draft.father.english,
    confirmed_rules_words: [...confirmedRulesWords],
    city_id: draft.city?.id ?? null,
    adda_id: draft.adda?.id ?? null,
    phone1: draft.phone1,
    phone2: draft.phone2,
    address: draft.address,
    linked_account_id: draft.linked_account?.id ?? null,
    opening_date: draft.opening_date,
    opening_dr: draft.opening_dr,
    opening_cr: draft.opening_cr,
    opening_is_advance: draft.opening_is_advance,
    credit_allowed: draft.credit_allowed,
    credit_limit: draft.credit_limit,
    default_marka_id: draft.default_marka?.id ?? null,
    collector_staff_id: draft.collector?.id ?? null,
    collection_weekday: draft.collection_weekday,
    is_active: draft.is_active,
    note: draft.note,
  };
}
