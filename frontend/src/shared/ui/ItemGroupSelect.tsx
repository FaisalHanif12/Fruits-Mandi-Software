// The item group box (plan 55, PRD v1.29): the sales reports and the analytics views A, D and G can look
// at fruit, at vegetables, or at both. One box for every screen that has it, so the words are the same:
// the two groups are the ones of the item list (`masters-item-group-*`). Choosing "fruit and vegetables"
// sends no filter.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { ItemGroup } from '../../bindings/ItemGroup';
import { Field } from './Field';
import { Select } from './Select';

/** A group of items, or `all` for both (no filter). */
export type ItemGroupChoice = ItemGroup | 'all';

export const ITEM_GROUP_CHOICES: readonly ItemGroupChoice[] = ['all', 'fruit', 'vegetable'];

interface ItemGroupSelectProps {
  value: ItemGroupChoice;
  onChange: (value: ItemGroupChoice) => void;
  /** The backend's refusal for this box, shown under it. */
  error?: ReactNode;
}

export function ItemGroupSelect({ value, onChange, error }: ItemGroupSelectProps) {
  const { l10n } = useLocalization();
  return (
    <Field label={<Localized id="common-item-group" />} error={error}>
      {(control) => (
        <Select<ItemGroupChoice>
          {...control}
          value={value}
          onChange={onChange}
          options={ITEM_GROUP_CHOICES.map((choice) => ({
            value: choice,
            label: l10n.getString(
              choice === 'all' ? 'common-item-group-all' : `masters-item-group-${choice}`,
            ),
          }))}
        />
      )}
    </Field>
  );
}
