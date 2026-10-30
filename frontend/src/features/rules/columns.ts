// The columns of the rule list (PRD 11.1): side, charge, party, item or group, value with its
// method, state, then level and dates. The less used ones come with the wider layout steps, so the
// list never scrolls sideways (NFR-USE-15, Code Standards F22): the steps are by window width, and
// the content area is about 290 px narrower than the window (the main bar), so the columns of a
// step add up to less than that. Words are Fluent IDs.

import { isoToDisplayDate } from '../../shared/format';
import { displayName, levelMessageId } from '../../shared/i18n';
import type { ListColumn } from '../../shared/grid';
import type { RuleMethod, RuleRow } from './types';

const VALUE_ID: Record<RuleMethod, string> = {
  percent: 'rules-value-percent',
  per_unit: 'rules-value-per-unit',
  per_lot: 'rules-value-per-lot',
};

export const RULE_COLUMNS: readonly ListColumn<RuleRow>[] = [
  {
    field: 'side',
    type: 'status',
    headerId: 'rules-col-side',
    sortKey: 'side',
    value: (row, { text }) => text(`rules-side-${row.side}`),
  },
  {
    field: 'charge',
    type: 'status',
    headerId: 'rules-col-charge',
    sortKey: 'charge_type',
    value: (row, { text }) => text(`rules-charge-${row.charge_type}`),
  },
  {
    field: 'party',
    type: 'name',
    headerId: 'rules-col-party',
    sortKey: 'account',
    grow: true,
    value: (row, { language, text }) =>
      row.account === null ? text('rules-any-party') : displayName(row.account, language),
  },
  {
    field: 'item',
    type: 'text',
    headerId: 'rules-col-item',
    sortKey: 'item',
    value: (row, { language, text }) =>
      row.item !== null
        ? displayName(row.item, language)
        : row.item_group !== null
          ? text(`rules-group-${row.item_group}`)
          : text('rules-any-item'),
  },
  {
    field: 'value',
    type: 'status',
    headerId: 'rules-col-value',
    sortKey: 'method',
    value: (row, { text }) => text(VALUE_ID[row.method], { value: row.value_text }),
  },
  {
    field: 'level',
    type: 'text',
    headerId: 'rules-col-level',
    sortKey: 'level',
    from: 'wide',
    value: (row, { text }) => text(levelMessageId(row.level)),
  },
  {
    field: 'from_date',
    type: 'date',
    headerId: 'rules-col-from',
    sortKey: 'from_date',
    from: 'wide',
    value: (row) => isoToDisplayDate(row.from_date),
  },
  {
    field: 'to_date',
    type: 'date',
    headerId: 'rules-col-to',
    sortKey: 'to_date',
    from: 'wide',
    value: (row) => (row.to_date === null ? '' : isoToDisplayDate(row.to_date)),
  },
  {
    field: 'status',
    // A default that has ended says until when, so the old defaults can be told apart.
    type: 'text',
    headerId: 'rules-col-status',
    sortKey: 'is_active',
    from: 'normal',
    value: (row, { text }) =>
      row.is_default
        ? row.to_date === null
          ? text('rules-status-default')
          : text('rules-status-default-ended', { to: isoToDisplayDate(row.to_date) })
        : text(row.is_active ? 'rules-status-active' : 'rules-status-inactive'),
  },
];
