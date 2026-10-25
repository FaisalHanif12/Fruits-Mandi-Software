// Typed wrappers around the پھل commands (plan 17d; module `fruit`, Tech Stack 5.2). Components
// never call invoke() (Code Standards F4). The only file that knows the wire shape. Every command
// takes one `{ input }` object. No cash moves on this screen, so there is no cash flag.

import { callCommand } from '../../shared/api';
import type {
  FruitCancelInput,
  FruitCancelled,
  FruitEditInput,
  FruitEdited,
  FruitGetInput,
  FruitInput,
  FruitList,
  FruitListInput,
  FruitLots,
  FruitLotsInput,
  FruitPreview,
  FruitRow,
  FruitSaved,
} from './types';

/** What saving would do: the value, each partner's part, who is credited, the lot's balance (C27). */
export function fruitPreview(input: FruitInput): Promise<FruitPreview> {
  return callCommand<FruitPreview>('fruit_preview', { input });
}

/** Saves the fruit taken: one transaction, the same function as the preview (PRD FR-16.10). */
export function fruitSave(input: FruitInput): Promise<FruitSaved> {
  return callCommand<FruitSaved>('fruit_save', { input });
}

/** Changes an entry while its day is open: the old posting is reversed, the new one posted. */
export function fruitEdit(input: FruitEditInput): Promise<FruitEdited> {
  return callCommand<FruitEdited>('fruit_edit', { input });
}

/** Cancels an entry with a reason, by a reversal, while its day is open (PRD FR-16.12). */
export function fruitCancel(input: FruitCancelInput): Promise<FruitCancelled> {
  return callCommand<FruitCancelled>('fruit_cancel', { input });
}

/** One entry with everything the edit form needs. */
export function fruitGet(input: FruitGetInput): Promise<FruitRow> {
  return callCommand<FruitRow>('fruit_get', { input });
}

/** One page of entries and the totals over the whole filter (PRD FR-19.11). */
export function fruitList(input: FruitListInput): Promise<FruitList> {
  return callCommand<FruitList>('fruit_list', { input });
}

/** The lots of our shop that have goods left, for the "from our shop" choice (PRD FR-16.11). */
export function fruitLots(input: FruitLotsInput): Promise<FruitLots> {
  return callCommand<FruitLots>('fruit_lots', { input });
}
