// What the trips list remembers for the Owner (PRD FR-7.45): the days, the status and the search he
// last used, kept as text under the screen name `list.trips`. A value that cannot be read any more is
// left out. Nothing about money is kept.

import { choiceOr, isRecord, textOr } from '../../shared/screen-state';

export const TRIPS_SCREEN = 'list.trips';

export type TripStatusChoice = 'all' | 'open' | 'closed';
const STATUS_CHOICES: readonly TripStatusChoice[] = ['all', 'open', 'closed'];

export interface TripChoices {
  from: string;
  to: string;
  status: TripStatusChoice;
  search: string;
}

export const NO_TRIP_CHOICES: TripChoices = { from: '', to: '', status: 'all', search: '' };

function short(value: unknown, most: number): string {
  const text = textOr(value, '');
  return text.length <= most ? text : '';
}

export function tripChoicesFrom(raw: unknown): TripChoices {
  if (!isRecord(raw)) return NO_TRIP_CHOICES;
  return {
    from: short(raw['from'], 20),
    to: short(raw['to'], 20),
    status: choiceOr(raw['status'], STATUS_CHOICES, 'all'),
    search: short(raw['search'], 100),
  };
}

export function tripChoicesText(choices: TripChoices): Record<string, unknown> | null {
  const kept: Record<string, unknown> = {};
  if (choices.from !== '') kept['from'] = choices.from;
  if (choices.to !== '') kept['to'] = choices.to;
  if (choices.status !== 'all') kept['status'] = choices.status;
  if (choices.search !== '') kept['search'] = choices.search;
  return Object.keys(kept).length === 0 ? null : kept;
}
