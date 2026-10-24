// What the پختہ کھاتہ register remembers for the Owner (PRD FR-7.45): the filters he last used (days,
// side, name, phone, paid or due), kept as text under the screen name `list.pukhta-register`. A value
// that cannot be read any more is left out. Nothing about money is kept.

import type { PaidState } from '../../bindings/PaidState';
import type { PukhtaSide } from '../../bindings/PukhtaSide';
import { choiceOr, isRecord, textOr } from '../../shared/screen-state';

export const PUKHTA_REGISTER_SCREEN = 'list.pukhta-register';

const SIDE_CHOICES: readonly (PukhtaSide | 'all')[] = ['all', 'buyer', 'seller'];
const STATE_CHOICES: readonly (PaidState | 'all')[] = ['all', 'paid', 'due'];

export interface RegisterChoices {
  from: string;
  to: string;
  side: PukhtaSide | 'all';
  name: string;
  phone: string;
  state: PaidState | 'all';
}

export const NO_REGISTER_CHOICES: RegisterChoices = {
  from: '',
  to: '',
  side: 'all',
  name: '',
  phone: '',
  state: 'all',
};

function short(value: unknown, most: number): string {
  const text = textOr(value, '');
  return text.length <= most ? text : '';
}

export function registerChoicesFrom(raw: unknown): RegisterChoices {
  if (!isRecord(raw)) return NO_REGISTER_CHOICES;
  return {
    from: short(raw['from'], 20),
    to: short(raw['to'], 20),
    side: choiceOr(raw['side'], SIDE_CHOICES, 'all'),
    name: short(raw['name'], 100),
    phone: short(raw['phone'], 30),
    state: choiceOr(raw['state'], STATE_CHOICES, 'all'),
  };
}

export function registerChoicesText(choices: RegisterChoices): Record<string, unknown> | null {
  const kept: Record<string, unknown> = {};
  if (choices.from !== '') kept['from'] = choices.from;
  if (choices.to !== '') kept['to'] = choices.to;
  if (choices.side !== 'all') kept['side'] = choices.side;
  if (choices.name !== '') kept['name'] = choices.name;
  if (choices.phone !== '') kept['phone'] = choices.phone;
  if (choices.state !== 'all') kept['state'] = choices.state;
  return Object.keys(kept).length === 0 ? null : kept;
}
