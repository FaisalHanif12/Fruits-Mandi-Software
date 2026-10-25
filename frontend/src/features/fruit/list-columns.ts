// The columns of the list under the پھل form (PRD FR-16.12, FR-19.11). Amounts and counts are the
// backend's and shown with `money` and `String`, never worked out. The list sits under the روکڑ side
// menu, so the room is the content area minus the main rail and the 200 px side menu: about 690 px
// at the narrowest window and 850 px at the normal step. Four columns fit the narrowest (who, item,
// quantity, value); the day comes at the normal step; the number, weight, rate and source at the wide
// step, so a list never scrolls sideways (NFR-USE-15, Code Standards F22). A cancelled entry says so
// in words in the first column, at every size (NFR-USE-12). Names go through `displayName` (F38).

import { formatGrams, isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { lotCode } from './lot-source';
import type { FruitRow } from './types';

type Say = (id: string, vars?: Record<string, string>) => string;

/** The number of an entry in its series: "پھل-3" or "FRT-3" (letters from Fluent). */
export function fruitNumberText(number: number, text: Say): string {
  return text('fruit-number', { number: String(number) });
}

export const FRUIT_COLUMNS: readonly ListColumn<FruitRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'fruit-col-number',
    from: 'wide',
    value: (row, { text }) => fruitNumberText(row.number, text),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'fruit-col-date',
    from: 'normal',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'for',
    type: 'name',
    headerId: 'fruit-col-for',
    grow: true,
    value: (row, { language, text }) => {
      const name =
        row.partner === null ? text('fruit-for-guests') : displayName(row.partner, language);
      return row.status === 'cancelled' ? text('fruit-for-cancelled', { name }) : name;
    },
  },
  {
    field: 'item',
    type: 'name',
    headerId: 'fruit-col-item',
    grow: true,
    value: (row, { language }) =>
      `${displayName(row.item, language)} ${displayName(row.packing, language)}`,
  },
  {
    field: 'qty',
    type: 'number',
    headerId: 'fruit-col-qty',
    value: (row) => String(row.qty),
  },
  {
    field: 'weight',
    type: 'number',
    headerId: 'fruit-col-weight',
    from: 'wide',
    value: (row) => (row.weight_grams === null ? '' : formatGrams(row.weight_grams)),
  },
  {
    field: 'rate',
    type: 'money',
    headerId: 'fruit-col-rate',
    from: 'wide',
    value: (row, { money }) => money(row.rate_paisa),
  },
  {
    field: 'value',
    type: 'money',
    headerId: 'fruit-col-value',
    value: (row, { money }) => money(row.value_paisa),
  },
  {
    field: 'from',
    type: 'name',
    headerId: 'fruit-col-from',
    from: 'wide',
    grow: true,
    // From our shop: the lot ("1-2"); from another shop: the agent who is credited.
    value: (row, { language, text }) =>
      row.lot === null
        ? displayName(row.credit_account, language)
        : text('fruit-from-lot', { lot: lotCode(row.lot) }),
  },
];
