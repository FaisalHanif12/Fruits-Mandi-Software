// Typed wrappers around the typing-help commands (plan 05 contract; module `translit`, Tech Stack
// 5.2). Components never call invoke() (Code Standards F4). Suggestions and learning are used only
// by Urdu text fields, never by pickers or search boxes (F41).

import type { LearnInput } from '../../bindings/LearnInput';
import type { PairsList } from '../../bindings/PairsList';
import type { RemovePairInput } from '../../bindings/RemovePairInput';
import type { SuggestInput } from '../../bindings/SuggestInput';
import type { Suggestions } from '../../bindings/Suggestions';
import { callCommand } from '../api';

/**
 * Up to three spellings of the text, best first, each with its source (learned, word list or
 * rules) and the same for each of its words. Empty text gives none.
 */
export function translitSuggest(input: SuggestInput): Promise<Suggestions> {
  return callCommand<Suggestions>('translit_suggest', { input });
}

/** Remembers one word pair: offered first from the next word on (PRD FR-7.26). */
export function translitLearn(input: LearnInput): Promise<null> {
  return callCommand<null>('translit_learn', { input });
}

/**
 * One page of the learned pairs, most used first, with the count and the totals of the whole list
 * (PRD FR-7.26, FR-19.11). Owner only.
 */
export function translitPairsList(offset: number, limit: number): Promise<PairsList> {
  return callCommand<PairsList>('translit_pairs_list', { input: { offset, limit } });
}

export function translitPairsRemove(input: RemovePairInput): Promise<null> {
  return callCommand<null>('translit_pairs_remove', { input });
}
