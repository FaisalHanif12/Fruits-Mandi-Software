import { useEffect, useState } from 'react';
import { translitSuggest } from './api';
import type { Suggestion } from '../../bindings/Suggestion';

interface Answer {
  word: string;
  items: Suggestion[];
}

/**
 * The Urdu spellings of an English word being typed (PRD FR-7.22), up to three, best first, each
 * with its source so the field can mark a spelling made by rules only (FR-7.66). Each key press
 * asks again, and the answer to an older word is dropped when a newer one has been asked, so a
 * slow answer never shows under the wrong word. A failure (the command is not ready, or the
 * backend is busy) gives no suggestions and no error: the field keeps what was typed. Used only
 * by Urdu text fields (Code Standards F41).
 */
export function useSuggestions(word: string, enabled: boolean): Suggestion[] {
  const [answer, setAnswer] = useState<Answer>({ word: '', items: [] });
  useEffect(() => {
    if (!enabled || word === '') return undefined;
    let current = true;
    translitSuggest({ text: word, direction: 'to_urdu' })
      .then((result) => {
        if (current) setAnswer({ word, items: result.items });
      })
      .catch(() => {
        if (current) setAnswer({ word, items: [] });
      });
    return () => {
      current = false;
    };
  }, [word, enabled]);
  return enabled && answer.word === word ? answer.items : [];
}
