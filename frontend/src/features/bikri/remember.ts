// What the بکری list screens remember for the Owner (PRD FR-7.45): the filters he last used on
// today's بکری (آج کی بکری, seller, item and status; the day always starts as today) and on باقی مال
// (days received, seller, item and the tick for written-off lots). Kept as text under the screen
// names below. A value that cannot be read any more is left out. Nothing about money is kept.

import type { BikriStatus } from '../../bindings/BikriStatus';
import type { PickerValue } from '../../shared/ui';
import {
  choiceOr,
  flagOf,
  isRecord,
  pickedOrNull,
  pickedText,
  textOr,
} from '../../shared/screen-state';

export const TODAY_SCREEN = 'list.bikri-today';
export const STOCK_SCREEN = 'list.bikri-stock';

const STATUS_CHOICES: readonly (BikriStatus | 'all')[] = ['all', 'saved', 'cancelled'];

export interface TodayChoices {
  seller: PickerValue | null;
  item: PickerValue | null;
  status: BikriStatus | 'all';
}

export const NO_TODAY_CHOICES: TodayChoices = { seller: null, item: null, status: 'all' };

export function todayChoicesFrom(raw: unknown): TodayChoices {
  if (!isRecord(raw)) return NO_TODAY_CHOICES;
  return {
    seller: pickedOrNull(raw['seller']),
    item: pickedOrNull(raw['item']),
    status: choiceOr(raw['status'], STATUS_CHOICES, 'all'),
  };
}

/** What to keep of today's list, or null when nothing is chosen. */
export function todayChoicesText(choices: TodayChoices): Record<string, unknown> | null {
  const kept: Record<string, unknown> = {};
  if (choices.seller !== null) kept['seller'] = pickedText(choices.seller);
  if (choices.item !== null) kept['item'] = pickedText(choices.item);
  if (choices.status !== 'all') kept['status'] = choices.status;
  return Object.keys(kept).length === 0 ? null : kept;
}

export interface StockChoices {
  from: string;
  to: string;
  seller: PickerValue | null;
  item: PickerValue | null;
  includeWrittenOff: boolean;
}

export const NO_STOCK_CHOICES: StockChoices = {
  from: '',
  to: '',
  seller: null,
  item: null,
  includeWrittenOff: false,
};

function dateText(value: unknown): string {
  const text = textOr(value, '');
  return text.length <= 20 ? text : '';
}

export function stockChoicesFrom(raw: unknown): StockChoices {
  if (!isRecord(raw)) return NO_STOCK_CHOICES;
  return {
    from: dateText(raw['from']),
    to: dateText(raw['to']),
    seller: pickedOrNull(raw['seller']),
    item: pickedOrNull(raw['item']),
    includeWrittenOff: flagOf(raw['includeWrittenOff']),
  };
}

export function stockChoicesText(choices: StockChoices): Record<string, unknown> | null {
  const kept: Record<string, unknown> = {};
  if (choices.from !== '') kept['from'] = choices.from;
  if (choices.to !== '') kept['to'] = choices.to;
  if (choices.seller !== null) kept['seller'] = pickedText(choices.seller);
  if (choices.item !== null) kept['item'] = pickedText(choices.item);
  if (choices.includeWrittenOff) kept['includeWrittenOff'] = true;
  return Object.keys(kept).length === 0 ? null : kept;
}
