// The nine masters of PRD 10.3 as configuration for one generic screen (plan 06). A master adds
// only what is special about it: which extra fields its form has and which extra columns its list
// has. The code, the names, the four states, the keys and the leave guard are the screen's. Words
// are Fluent message IDs (masters.ftl), never text. Names go through `displayName` (F38).

import {
  Apple,
  Boxes,
  Container,
  MapPin,
  MessageSquareText,
  Receipt,
  Tag,
  UserRound,
  Warehouse,
  type LucideIcon,
} from 'lucide-react';
import type { Language } from '../../i18n';
import { displayName, otherScriptName } from '../../shared/i18n';
import type { AccountScope } from '../../shared/accounts';
import type { ListColumn } from '../../shared/grid';
import type { MasterKind, MasterRecord } from './types';

/** A choice from a short fixed list. The value is what the backend stores. */
export interface ChoiceOption {
  value: string;
  labelId: string;
}

/** What a form field is. Everything is text, as typed (CLAUDE.md rule 1). */
export type FieldControl =
  | { type: 'text' }
  /** Latin letters and digits written left to right: a phone, a weight. */
  | { type: 'latin' }
  | { type: 'select'; options: readonly ChoiceOption[] }
  /** A record of another master, found by a Picker. */
  | { type: 'picker'; scope: MasterKind }
  /** An account (a seller), found by the AccountPicker, with his party card. */
  | { type: 'account'; scope: AccountScope }
  /** Several choices from a short fixed list, ticked. Nothing ticked means all (PRD 10.3). */
  | { type: 'multi'; options: readonly ChoiceOption[] };

export interface FieldSpec {
  /** The key in `MasterDraft.text` (or `refs` for a picker). */
  key: string;
  labelId: string;
  hintId?: string;
  control: FieldControl;
}

/**
 * A fact the backend works out and the form only shows, for example the code of the account a
 * staff member gets when he is first lent money (read-only, PRD 10.3).
 */
export interface ShownSpec {
  key: string;
  labelId: string;
  /** Said under the field while it is empty. */
  emptyHintId: string;
}

export interface MasterConfig {
  kind: MasterKind;
  /** The address segment: /index/<path>. */
  path: string;
  icon: LucideIcon;
  /** Menu card and page title (plural words). */
  titleId: string;
  helpId: string;
  /** The button and form title for a new one: "New city". */
  newId: string;
  /** The empty state: "No cities yet". */
  emptyId: string;
  /** Does this master have a code of its own? Narration and staff do not. */
  hasCode: boolean;
  /** The two names, or null for a master with none (mark). */
  names: { urduLabelId: string; englishLabelId: string; spelling: boolean } | null;
  fields: readonly FieldSpec[];
  /** Read-only facts under the fields. */
  shown: readonly ShownSpec[];
  /** Columns after the code and the names. */
  extraColumns: readonly ListColumn<MasterRecord>[];
}

const choice = (prefix: string, values: readonly string[]): ChoiceOption[] =>
  values.map((value) => ({ value, labelId: `${prefix}-${value}` }));

const NAMES = {
  urduLabelId: 'masters-field-name-ur',
  englishLabelId: 'masters-field-name-en',
  spelling: true,
} as const;

/** A column that shows the text of a field or the name of a record it points at. */
const textColumn = (key: string, headerId: string): ListColumn<MasterRecord> => ({
  field: key,
  type: 'text',
  headerId,
  value: (row) => row.text[key] ?? '',
});

const latinColumn = (key: string, headerId: string): ListColumn<MasterRecord> => ({
  field: key,
  type: 'number',
  headerId,
  value: (row) => row.text[key] ?? '',
});

/** A column that shows the label of a short choice (fruit, vegetable, sold by weight). */
const choiceColumn = (
  key: string,
  headerId: string,
  prefix: string,
  options: { from?: ListColumn<MasterRecord>['from']; short?: boolean } = {},
): ListColumn<MasterRecord> => ({
  field: key,
  // One short word (fruit, vegetable) fits the narrow column; a phrase needs the wider one.
  type: options.short === true ? 'status' : 'choice',
  headerId,
  from: options.from,
  value: (row, { text }) => {
    const value = row.text[key];
    return value === undefined || value === '' ? '' : text(`${prefix}-${value}`);
  },
});

const refColumn = (key: string, headerId: string): ListColumn<MasterRecord> => ({
  field: key,
  type: 'text',
  headerId,
  value: (row, { language }) => {
    const ref = row.refs[key];
    return ref === null || ref === undefined ? '' : displayName(ref, language);
  },
});

const shownColumn = (key: string, headerId: string): ListColumn<MasterRecord> => ({
  field: key,
  type: 'code',
  headerId,
  value: (row) => row.shown[key] ?? '',
});

const ITEM_GROUPS = choice('masters-item-group', ['fruit', 'vegetable']);
const SOLD_BY = choice('masters-sold-by', ['unit', 'weight']);
const STAFF_JOBS = choice('masters-staff-job', ['munshi', 'collector', 'labour']);

/** The voucher types a standard narration is offered in (Architecture 15.4 `used_in`). */
const VOUCHER_TYPES = choice('masters-used-in', [
  'receipt',
  'payment',
  'advance',
  'shop_expense',
  'personal_expense',
  'personal_return',
  'money_lent',
  'money_lent_return',
  'general',
]);

export const MASTER_CONFIGS: readonly MasterConfig[] = [
  {
    kind: 'city',
    path: 'cities',
    icon: MapPin,
    titleId: 'masters-city-title',
    helpId: 'masters-city-help',
    newId: 'masters-city-new',
    emptyId: 'masters-city-empty',
    hasCode: true,
    names: NAMES,
    fields: [],
    shown: [],
    extraColumns: [],
  },
  {
    kind: 'item',
    path: 'items',
    icon: Apple,
    titleId: 'masters-item-title',
    helpId: 'masters-item-help',
    newId: 'masters-item-new',
    emptyId: 'masters-item-empty',
    hasCode: true,
    names: NAMES,
    fields: [
      {
        key: 'item_group',
        labelId: 'masters-item-group',
        control: { type: 'select', options: ITEM_GROUPS },
      },
      {
        key: 'default_packing',
        labelId: 'masters-item-packing',
        hintId: 'masters-item-packing-hint',
        control: { type: 'picker', scope: 'packing' },
      },
      {
        key: 'sold_by',
        labelId: 'masters-item-sold-by',
        control: { type: 'select', options: SOLD_BY },
      },
    ],
    shown: [],
    extraColumns: [
      choiceColumn('item_group', 'masters-item-group', 'masters-item-group', { short: true }),
      refColumn('default_packing', 'masters-item-packing'),
      // Less used: it comes with the normal and wide layouts, so a list never scrolls sideways.
      choiceColumn('sold_by', 'masters-item-sold-by', 'masters-sold-by', { from: 'normal' }),
    ],
  },
  {
    kind: 'packing',
    path: 'packings',
    icon: Boxes,
    titleId: 'masters-packing-title',
    helpId: 'masters-packing-help',
    newId: 'masters-packing-new',
    emptyId: 'masters-packing-empty',
    hasCode: true,
    names: NAMES,
    fields: [
      {
        key: 'approx_kg',
        labelId: 'masters-packing-weight',
        hintId: 'masters-packing-weight-hint',
        control: { type: 'latin' },
      },
    ],
    shown: [],
    extraColumns: [latinColumn('approx_kg', 'masters-packing-weight')],
  },
  {
    kind: 'adda',
    path: 'addas',
    icon: Container,
    titleId: 'masters-adda-title',
    helpId: 'masters-adda-help',
    newId: 'masters-adda-new',
    emptyId: 'masters-adda-empty',
    hasCode: true,
    names: NAMES,
    fields: [
      { key: 'city', labelId: 'masters-adda-city', control: { type: 'picker', scope: 'city' } },
    ],
    shown: [],
    extraColumns: [refColumn('city', 'masters-adda-city')],
  },
  {
    kind: 'marka',
    path: 'markas',
    icon: Tag,
    titleId: 'masters-marka-title',
    helpId: 'masters-marka-help',
    newId: 'masters-marka-new',
    emptyId: 'masters-marka-empty',
    hasCode: true,
    names: null,
    fields: [
      {
        key: 'mark_text',
        labelId: 'masters-marka-text',
        hintId: 'masters-marka-text-hint',
        control: { type: 'text' },
      },
      {
        key: 'owner',
        labelId: 'masters-marka-owner',
        hintId: 'masters-marka-owner-hint',
        control: { type: 'account', scope: 'seller' },
      },
    ],
    shown: [],
    extraColumns: [
      textColumn('mark_text', 'masters-marka-text'),
      refColumn('owner', 'masters-marka-owner'),
    ],
  },
  {
    kind: 'store',
    path: 'stores',
    icon: Warehouse,
    titleId: 'masters-store-title',
    helpId: 'masters-store-help',
    newId: 'masters-store-new',
    emptyId: 'masters-store-empty',
    hasCode: true,
    names: NAMES,
    fields: [],
    shown: [],
    extraColumns: [],
  },
  {
    kind: 'narration',
    path: 'narrations',
    icon: MessageSquareText,
    titleId: 'masters-narration-title',
    helpId: 'masters-narration-help',
    newId: 'masters-narration-new',
    emptyId: 'masters-narration-empty',
    hasCode: false,
    // A narration is free text, not a name: it is not spell-checked (PRD FR-7.66).
    names: {
      urduLabelId: 'masters-narration-text-ur',
      englishLabelId: 'masters-narration-text-en',
      spelling: false,
    },
    fields: [
      {
        key: 'used_in',
        labelId: 'masters-narration-used-in',
        hintId: 'masters-narration-used-in-hint',
        control: { type: 'multi', options: VOUCHER_TYPES },
      },
    ],
    shown: [],
    extraColumns: [],
  },
  {
    kind: 'expense_head',
    path: 'expense-heads',
    icon: Receipt,
    titleId: 'masters-expense-head-title',
    helpId: 'masters-expense-head-help',
    newId: 'masters-expense-head-new',
    emptyId: 'masters-expense-head-empty',
    hasCode: true,
    names: NAMES,
    fields: [],
    shown: [
      {
        key: 'account_code',
        labelId: 'masters-account-code',
        emptyHintId: 'masters-account-code-hint',
      },
    ],
    extraColumns: [shownColumn('account_code', 'masters-account-code')],
  },
  {
    kind: 'staff',
    path: 'staff',
    icon: UserRound,
    titleId: 'masters-staff-title',
    helpId: 'masters-staff-help',
    newId: 'masters-staff-new',
    emptyId: 'masters-staff-empty',
    hasCode: false,
    names: NAMES,
    fields: [
      {
        key: 'job',
        labelId: 'masters-staff-job',
        control: { type: 'select', options: STAFF_JOBS },
      },
      { key: 'phone', labelId: 'masters-staff-phone', control: { type: 'latin' } },
    ],
    shown: [
      {
        key: 'account_code',
        labelId: 'masters-staff-account',
        emptyHintId: 'masters-staff-account-hint',
      },
    ],
    extraColumns: [
      choiceColumn('job', 'masters-staff-job', 'masters-staff-job'),
      latinColumn('phone', 'masters-staff-phone'),
      shownColumn('account_code', 'masters-staff-account'),
    ],
  },
];

export function configOfKind(kind: MasterKind): MasterConfig {
  const found = MASTER_CONFIGS.find((config) => config.kind === kind);
  if (found === undefined) throw new Error(`no master configuration for ${kind}`);
  return found;
}

/** The configuration for an address segment, or undefined for a segment that is not a master. */
export function configOfPath(path: string): MasterConfig | undefined {
  return MASTER_CONFIGS.find((config) => config.path === path);
}

/** The columns of a master's list: code, name, the other script's name, its own, then the state. */
export function columnsOf(config: MasterConfig): ListColumn<MasterRecord>[] {
  const columns: ListColumn<MasterRecord>[] = [];
  if (config.hasCode) {
    columns.push({
      field: 'code',
      type: 'code',
      headerId: 'masters-col-code',
      value: (row) => row.code,
    });
  }
  if (config.names !== null) {
    columns.push(
      {
        field: 'name',
        type: 'name',
        headerId: 'masters-col-name',
        grow: true,
        value: (row, { language }) => (row.names === null ? '' : displayName(row.names, language)),
      },
      {
        field: 'other_name',
        type: 'name',
        headerId: 'masters-col-other-name',
        grow: true,
        // The name in the other script only tells two records apart: the normal layout shows it.
        from: 'normal',
        script: (language: Language) => (language === 'ur' ? 'en' : 'ur'),
        value: (row, { language }) =>
          row.names === null ? '' : otherScriptName(row.names, language),
      },
    );
  }
  columns.push(...config.extraColumns, {
    field: 'status',
    type: 'status',
    headerId: 'masters-col-status',
    value: (row, { text }) =>
      text(row.is_active ? 'masters-status-active' : 'masters-status-inactive'),
  });
  return columns;
}
