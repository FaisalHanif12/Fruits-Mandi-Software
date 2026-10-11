// Test helper: a fake of `translit_suggest` in the real answer shape (plan 05 v1.2): up to three
// suggestions of the whole text, each with its source and the source of each of its words. A word
// listed in `rulesOnly` has only rule-made spellings (PRD FR-7.66); every other known word comes
// from the word list. A text of several words gives one suggestion, the best of each word.

import type { Source } from '../../src/bindings/Source';
import type { Suggestion } from '../../src/bindings/Suggestion';
import type { Suggestions } from '../../src/bindings/Suggestions';

type Direction = 'to_urdu' | 'to_english';

interface Fake {
  /** English word (lower case) to its Urdu spellings, best first. */
  toUrdu?: Record<string, string[]>;
  /** Urdu word to its English spelling. */
  toEnglish?: Record<string, string>;
  /** Words (either script) whose spellings are made by the rules alone. */
  rulesOnly?: readonly string[];
}

/** The answer the backend gives for `text`. A word it does not know gives no suggestion at all. */
export function suggestionsFor(fake: Fake, text: string, direction: Direction): Suggestions {
  const rules = new Set(fake.rulesOnly ?? []);
  const words = text.split(/\s+/).filter((word) => word !== '');
  const spellingsOf = (input: string): string[] =>
    direction === 'to_urdu'
      ? (fake.toUrdu?.[input.toLowerCase()] ?? [])
      : fake.toEnglish?.[input] === undefined
        ? []
        : [fake.toEnglish[input]];
  const sourceOf = (input: string): Source =>
    rules.has(input.toLowerCase()) || rules.has(input) ? 'rules' : 'word_list';

  const known = words.map((input) => ({ input, spellings: spellingsOf(input) }));
  if (known.length === 0 || known.some((word) => word.spellings.length === 0)) return { items: [] };
  const [only] = known;
  if (known.length === 1 && only !== undefined) {
    // One word: its spellings are the suggestions.
    const source = sourceOf(only.input);
    return {
      items: only.spellings.map((spelling) => ({
        text: spelling,
        source,
        words: [{ text: spelling, source }],
      })),
    };
  }
  // A phrase: one suggestion, the best spelling of each word.
  const best = known.map(({ input, spellings }) => ({
    text: spellings[0] ?? '',
    source: sourceOf(input),
  }));
  const weakest: Source = best.some((word) => word.source === 'rules') ? 'rules' : 'word_list';
  const suggestion: Suggestion = {
    text: best.map((word) => word.text).join(' '),
    source: weakest,
    words: best,
  };
  return { items: [suggestion] };
}

/** The answer of an empty text, or a text with nothing to suggest. */
export const NO_SUGGESTIONS: Suggestions = { items: [] };
