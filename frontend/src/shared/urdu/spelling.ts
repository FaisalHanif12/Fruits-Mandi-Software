// The spelling check of names (PRD FR-7.66, E49). A word the app made only from its spelling
// rules, not from the word list or a learned pair, is marked "check the spelling". A name with
// such a word is not saved until the user confirms or fixes it. These helpers keep the marks next
// to the text of the name, so they sit in the screen's form state and a language switch loses
// nothing (Code Standards F39). They only decide what to ask; the backend checks again on save
// (ERR_SPELLING_UNCONFIRMED) and is the final word.

import type { Suggestion } from '../../bindings/Suggestion';
import { wordsOf } from './text';

/** The words of one name pair that the user has been told to check, and the ones he confirmed. */
export interface SpellingMarks {
  /** Words of the Urdu name made by rules only and not confirmed yet. */
  urdu: readonly string[];
  /** Words of the English name made by rules only and not confirmed yet. */
  english: readonly string[];
  /** Words he confirmed. They go with the save while they stay in a name. */
  confirmed: readonly string[];
}

export const NO_MARKS: SpellingMarks = { urdu: [], english: [], confirmed: [] };

/** A name pair with its marks. `NamePairValue` is this. */
export interface NameWithMarks {
  urdu: string;
  english: string;
  spelling?: SpellingMarks | undefined;
}

/**
 * A word of one letter is an initial ("M. Akram"). The backend never asks about it, so the screen
 * never marks it (services/spelling).
 */
export function isInitial(word: string): boolean {
  // Letters as the reader sees them, so a letter with a mark on it still counts as one.
  return Array.from(new Intl.Segmenter().segment(word)).length < 2;
}

/**
 * The words of a suggestion that the user must look at (FR-7.66): the ones the spelling rules
 * alone made, leaving out initials. A word the backend marks is exactly a word its save asks about.
 */
export function rulesOnlyWords(suggestion: Suggestion | undefined): string[] {
  if (suggestion === undefined) return [];
  return suggestion.words
    .filter((word) => word.source === 'rules' && !isInitial(word.text))
    .map((word) => word.text);
}

/** The words of `candidates` that are still in `text`, once each, in the order of the text. */
export function wordsStillIn(text: string, candidates: readonly string[]): string[] {
  const wanted = new Set(candidates);
  const found: string[] = [];
  for (const word of wordsOf(text)) {
    if (wanted.has(word) && !found.includes(word)) found.push(word);
  }
  return found;
}

/** `marks` with `word` added to the Urdu or the English list. */
export function withMark(
  marks: SpellingMarks | undefined,
  script: 'urdu' | 'english',
  word: string,
): SpellingMarks {
  const base = marks ?? NO_MARKS;
  if (base.confirmed.includes(word) || base[script].includes(word)) return base;
  return { ...base, [script]: [...base[script], word] };
}

/** The words still to be checked in each name: marked, not confirmed, still in the text. */
export function pendingSpelling(value: NameWithMarks): { urdu: string[]; english: string[] } {
  const marks = value.spelling ?? NO_MARKS;
  return {
    urdu: wordsStillIn(value.urdu, marks.urdu),
    english: wordsStillIn(value.english, marks.english),
  };
}

/** True when a name still has a word to check. */
export function hasPendingSpelling(value: NameWithMarks): boolean {
  const pending = pendingSpelling(value);
  return pending.urdu.length > 0 || pending.english.length > 0;
}

/** The confirmed words that are still in a name: the list a save sends as `confirmed_rules_words`. */
export function confirmedSpelling(value: NameWithMarks): string[] {
  const confirmed = value.spelling?.confirmed ?? [];
  return [
    ...new Set([...wordsStillIn(value.urdu, confirmed), ...wordsStillIn(value.english, confirmed)]),
  ];
}

/**
 * The value after the user confirmed `words`: they leave the "to check" lists and join the
 * confirmed list, so a second save does not ask again.
 */
export function confirmSpelling(value: NameWithMarks, words: readonly string[]): NameWithMarks {
  const marks = value.spelling ?? NO_MARKS;
  const confirmed = new Set([...marks.confirmed, ...words]);
  return {
    ...value,
    spelling: {
      urdu: marks.urdu.filter((word) => !confirmed.has(word)),
      english: marks.english.filter((word) => !confirmed.has(word)),
      confirmed: [...confirmed],
    },
  };
}
