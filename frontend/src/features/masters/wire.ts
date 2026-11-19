// The masters commands' own types turned into the one shape the screens use, and back (plan 06 T4).
// The backend sends one row type per kind (generated in bindings/); the list, the form and the
// tests work on `MasterRecord` and `MasterDraft`. This is the only file that knows each kind's
// fields, so a change in the backend contract changes this file and nothing else.
//
// Everything the form edits travels back on a save: the owner of a mark (an account picker) and
// the "used in" list of a narration (ticked voucher types).

import type { MasterInput } from '../../bindings/MasterInput';
import type { MasterRow } from '../../bindings/MasterRow';
import type { PickerValue } from '../../shared/ui';
import type { MasterDraft, MasterRecord } from './types';

/** A record another one points at, as a picker shows it. The code is not part of the row. */
function refOf(
  id: number | null,
  nameUr: string | null,
  nameEn: string | null,
  code = '',
): PickerValue | null {
  return id === null ? null : { id, code, name_ur: nameUr ?? '', name_en: nameEn ?? '' };
}

/** One row from the backend as the screens use it. */
export function rowToRecord(row: MasterRow): MasterRecord {
  const base = {
    kind: row.kind,
    id: row.id,
    version: row.version,
    is_active: row.is_active,
    text: {},
    refs: {},
    shown: {},
    lists: {},
  } as const;
  switch (row.kind) {
    case 'city':
    case 'store':
      return {
        ...base,
        code: row.code,
        names: { name_ur: row.name_ur, name_en: row.name_en },
      };
    case 'item':
      return {
        ...base,
        code: row.code,
        names: { name_ur: row.name_ur, name_en: row.name_en },
        text: { item_group: row.item_group, sold_by: row.sold_by },
        refs: {
          default_packing: refOf(
            row.default_packing_id,
            row.default_packing_name_ur,
            row.default_packing_name_en,
          ),
        },
      };
    case 'packing':
      return {
        ...base,
        code: row.code,
        names: { name_ur: row.name_ur, name_en: row.name_en },
        text: { approx_kg: row.approx_kg ?? '' },
      };
    case 'adda':
      return {
        ...base,
        code: row.code,
        names: { name_ur: row.name_ur, name_en: row.name_en },
        refs: { city: refOf(row.city_id, row.city_name_ur, row.city_name_en) },
      };
    case 'marka':
      return {
        ...base,
        code: row.code,
        names: null,
        text: { mark_text: row.mark_text },
        // The owner is chosen with the account picker (a seller).
        refs: {
          owner: refOf(
            row.owner_account_id,
            row.owner_name_ur,
            row.owner_name_en,
            row.owner_account_code ?? '',
          ),
        },
      };
    case 'narration':
      return {
        ...base,
        code: '',
        names: { name_ur: row.text_ur, name_en: row.text_en },
        lists: { used_in: row.used_in },
      };
    case 'expense_head':
      return {
        ...base,
        code: row.code,
        names: { name_ur: row.name_ur, name_en: row.name_en },
        shown: { account_code: row.account_code },
      };
    case 'staff':
      return {
        ...base,
        code: row.code,
        names: { name_ur: row.name_ur, name_en: row.name_en },
        text: { job: row.job, phone: row.phone ?? '' },
        shown: { account_code: row.account_code ?? '' },
      };
  }
}

/** What the form holds as the input of `masters_save`. Everything is text, as typed. */
export function draftToInput(
  kind: MasterRecord['kind'],
  draft: MasterDraft,
  confirmedRulesWords: readonly string[],
): MasterInput {
  const common = {
    id: draft.id,
    version: draft.version,
    name_ur: draft.names.urdu,
    name_en: draft.names.english,
    confirmed_rules_words: [...confirmedRulesWords],
  };
  const text = (key: string) => draft.text[key] ?? '';
  const refId = (key: string) => draft.refs[key]?.id ?? null;
  switch (kind) {
    case 'city':
    case 'store':
    case 'expense_head':
      return { kind, ...common, code: draft.code };
    case 'item':
      return {
        kind,
        ...common,
        code: draft.code,
        item_group: text('item_group'),
        default_packing_id: refId('default_packing'),
        sold_by: text('sold_by'),
      };
    case 'packing':
      return { kind, ...common, code: draft.code, approx_kg: text('approx_kg') };
    case 'adda':
      return { kind, ...common, code: draft.code, city_id: refId('city') };
    case 'marka':
      return {
        kind,
        id: draft.id,
        version: draft.version,
        code: draft.code,
        mark_text: text('mark_text'),
        owner_account_id: refId('owner'),
      };
    case 'narration':
      return {
        kind,
        id: draft.id,
        version: draft.version,
        text_ur: draft.names.urdu,
        text_en: draft.names.english,
        used_in: [...(draft.lists['used_in'] ?? [])],
      };
    case 'staff':
      return { kind, ...common, code: draft.code, job: text('job'), phone: text('phone') };
  }
}
