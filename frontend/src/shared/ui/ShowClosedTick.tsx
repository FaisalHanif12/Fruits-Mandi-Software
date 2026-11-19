// The tick "show closed and cancelled too" of a list (plan 38 R4, G6): off by default, so finished
// records step aside while nothing is deleted. One piece for every list, so the words and the place
// are the same everywhere (Code Standards F12).

import { Localized } from '@fluent/react';
import { Checkbox } from './Checkbox';

interface ShowClosedTickProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function ShowClosedTick({ checked, onChange }: ShowClosedTickProps) {
  return (
    <Checkbox checked={checked} onChange={onChange} label={<Localized id="common-show-closed" />} />
  );
}
