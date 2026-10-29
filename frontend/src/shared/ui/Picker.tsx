// The picker for the master lists: cities, items, packings and the rest of انڈکس (plan 06). All
// the behaviour is in PickerBase; this one only says where its records come from (shared/search)
// and, when nothing is found, which Index form its "add" button opens (plan 32, R8).
// Accounts have their own, AccountPicker, built on the same base (plan 08).

import { useMemo } from 'react';
import { masterSource, type SearchHit, type SearchScope } from '../search';
import { useAddRecord, type AddRecordKind, type VentureScope } from './add-record';
import { PickerBase, type PickerValue } from './PickerBase';
import type { FieldControlProps } from './Field';

export type { PickerValue } from './PickerBase';

interface PickerProps extends FieldControlProps {
  /** Which list to search: a master kind. */
  scope: SearchScope;
  value: PickerValue | null;
  /** The record chosen, or null when the field was emptied. */
  onChange: (hit: SearchHit | null) => void;
  placeholder?: string | undefined;
  /** Called with the typed text on F4 and from the button of an empty result. Leave out to hide both. */
  onAddNew?: ((text: string) => void) | undefined;
  /**
   * False where adding a record from this field would not make sense, for example a filter.
   * Default: the empty list offers the button when the app can open the Index form.
   */
  canAdd?: boolean | undefined;
  /** For a picker in a grid cell (see PickerBase). */
  initialText?: string | undefined;
  startOpen?: boolean | undefined;
  onEnterPicked?: (() => void) | undefined;
}

/**
 * The Fluent message of the "add" button of each list, and the Index form it opens. A trip and a
 * garden have none: they are made on their own screens (PRD 12.10, 12.11), so their picker only finds.
 */
const ADD: Record<Exclude<SearchScope, VentureScope>, { label: string; kind: AddRecordKind }> = {
  city: { label: 'search-add-city', kind: 'city' },
  item: { label: 'search-add-item', kind: 'item' },
  packing: { label: 'search-add-packing', kind: 'packing' },
  adda: { label: 'search-add-adda', kind: 'adda' },
  marka: { label: 'search-add-marka', kind: 'marka' },
  store: { label: 'search-add-store', kind: 'store' },
  narration: { label: 'search-add-narration', kind: 'narration' },
  expense_head: { label: 'search-add-expense-head', kind: 'expense_head' },
  staff: { label: 'search-add-staff', kind: 'staff' },
  seller: { label: 'search-add-seller', kind: 'seller' },
  buyer: { label: 'search-add-buyer', kind: 'buyer' },
  any_account: { label: 'search-add-account', kind: 'any' },
};

function isVentureScope(scope: SearchScope): scope is VentureScope {
  return scope === 'trip' || scope === 'garden' || scope === 'trip_all' || scope === 'garden_all';
}

export function Picker({ scope, canAdd = true, ...rest }: PickerProps) {
  const source = useMemo(() => masterSource(scope), [scope]);
  const addRecord = useAddRecord();
  const add = isVentureScope(scope) ? null : ADD[scope];
  return (
    <PickerBase<SearchHit>
      source={source}
      addLabelId={add?.label}
      onAddRecord={
        addRecord !== null && add !== null
          ? (text, done) => {
              addRecord({ kind: add.kind, text, onClosed: done });
            }
          : undefined
      }
      showAdd={canAdd && add !== null}
      {...rest}
    />
  );
}
