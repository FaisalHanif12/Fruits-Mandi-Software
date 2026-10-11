// The picker for the master lists: cities, items, packings and the rest of انڈکس (plan 06). All
// the behaviour is in PickerBase; this one only says where its records come from (shared/search).
// Accounts have their own, AccountPicker, built on the same base (plan 08).

import { useMemo } from 'react';
import { masterSource, type SearchHit, type SearchScope } from '../search';
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
}

export function Picker({ scope, ...rest }: PickerProps) {
  const source = useMemo(() => masterSource(scope), [scope]);
  return <PickerBase<SearchHit> source={source} {...rest} />;
}
