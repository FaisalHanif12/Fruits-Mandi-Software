import { useEffect, useState } from 'react';
import { translitSuggest } from './api';
import type { Kind } from '../../bindings/Kind';
import type { Suggestion } from '../../bindings/Suggestion';

interface Answer {
  asked: string;
  items: Suggestion[];
}

/** What the field says about the word being typed (plan 99, PRD FR-7.22). */
export interface TypingContext {
  /** `word` for free text, where everyday words come first; a name kind for a name field. */
  kind: Kind;
  /** The text of the field before the word being typed: the backend reads its end for context. */
  before: string;
}

/**
 * The Urdu spellings of an English word being typed (PRD FR-7.22), up to three, best first, each
 * with its source so the field can mark a spelling made by rules only (FR-7.66). The field says its
 * kind and the text before the word, so free text and names are ranked each their own way and a
 * word is read in its phrase (after کہا, `ke` is کہ). Only the word being typed is converted. Each
 * key press asks again, and the answer to an older word is dropped when a newer one has been asked,
 * so a slow answer never shows under the wrong word. A failure (the command is not ready, or the
 * backend is busy) gives no suggestions and no error: the field keeps what was typed. Used only
 * by Urdu text fields (Code Standards F41).
 */
export function useSuggestions(
  word: string,
  enabled: boolean,
  { kind, before }: TypingContext,
): Suggestion[] {
  const asked = JSON.stringify([word, kind, before]);
  const [answer, setAnswer] = useState<Answer>({ asked: '', items: [] });
  useEffect(() => {
    if (!enabled || word === '') return undefined;
    let current = true;
    translitSuggest({
      text: word,
      direction: 'to_urdu',
      kind,
      before: before === '' ? null : before,
    })
      .then((result) => {
        if (current) setAnswer({ asked, items: result.items });
      })
      .catch(() => {
        if (current) setAnswer({ asked, items: [] });
      });
    return () => {
      current = false;
    };
  }, [word, enabled, kind, before, asked]);
  return enabled && answer.asked === asked ? answer.items : [];
}
