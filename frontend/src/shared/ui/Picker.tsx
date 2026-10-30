// The picker for the master lists: cities, items, packings and the rest of انڈکس (plan 06). All
// the behaviour is in PickerBase; this one only says where its records come from (shared/search)
// and which short form its add row opens (plan 41, PRD FR-7.69).
// Accounts have their own, AccountPicker, built on the same base (plan 08).

import { useMemo } from 'react';
import { findMaster, masterSource, type SearchHit, type SearchScope } from '../search';
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
  /**
   * False where adding a record from this field would not make sense: a filter box (report,
   * analytics, list filter, find, the rule test, plan 41 R5). Default: the open list ends with the
   * add row when the app can open the short form.
   */
  canAdd?: boolean | undefined;
  /** For a مارکہ added from a lot: the lot's seller, who becomes its owner (plan 41, R3). */
  addOwner?: PickerValue | null | undefined;
  /** For a picker in a grid cell (see PickerBase). */
  initialText?: string | undefined;
  startOpen?: boolean | undefined;
  onEnterPicked?: (() => void) | undefined;
}

/**
 * The word of each list in the add row's and the empty list's Fluent messages, and the short form
 * it opens. A trip and a garden have none: they are made on their own screens (PRD 12.10, 12.11),
 * so their picker only finds.
 */
const ADD: Record<Exclude<SearchScope, VentureScope>, { noun: string; kind: AddRecordKind }> = {
  city: { noun: 'city', kind: 'city' },
  item: { noun: 'item', kind: 'item' },
  packing: { noun: 'packing', kind: 'packing' },
  adda: { noun: 'adda', kind: 'adda' },
  marka: { noun: 'marka', kind: 'marka' },
  store: { noun: 'store', kind: 'store' },
  narration: { noun: 'narration', kind: 'narration' },
  expense_head: { noun: 'expense-head', kind: 'expense_head' },
  staff: { noun: 'staff', kind: 'staff' },
  seller: { noun: 'seller', kind: 'seller' },
  buyer: { noun: 'buyer', kind: 'buyer' },
  any_account: { noun: 'account', kind: 'any' },
};

function isVentureScope(scope: SearchScope): scope is VentureScope {
  return scope === 'trip' || scope === 'garden' || scope === 'trip_all' || scope === 'garden_all';
}

export function Picker({ scope, canAdd = true, addOwner, ...rest }: PickerProps) {
  const source = useMemo(() => masterSource(scope), [scope]);
  const addRecord = useAddRecord();
  const add = isVentureScope(scope) ? null : ADD[scope];
  const adds = canAdd && add !== null;
  return (
    <PickerBase<SearchHit>
      source={source}
      {...(adds ? { addNoun: add.noun } : {})}
      onAddRecord={
        adds && addRecord !== null
          ? (text, done) => {
              addRecord({
                kind: add.kind,
                text,
                owner: addOwner,
                onClosed: (added) => {
                  if (added === null) {
                    done(null);
                    return;
                  }
                  // The list's own row for the new record: its names, second line and weight mark.
                  findMaster(scope, added.id).then(done, () => {
                    done(null);
                  });
                },
              });
            }
          : undefined
      }
      {...rest}
    />
  );
}
