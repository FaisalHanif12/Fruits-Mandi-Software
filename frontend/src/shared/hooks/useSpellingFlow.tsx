// The spelling check around a save (PRD FR-7.66, E49). A screen that saves a named record wraps
// its save in `guard`:
//
//   const spelling = useSpellingFlow({ onFix: focusNameField });
//   const result = await spelling.guard(names, setNames, (confirmed) => save({ ..., confirmed_rules_words: confirmed }));
//   ... {spelling.dialog}
//
// - Before the save, a name that still holds a marked word shows the dialog: the user confirms
//   the spelling (the save goes on) or goes back to fix it (the save does not happen).
// - If the backend refuses the save with ERR_SPELLING_UNCONFIRMED (a word the screen did not
//   know about), the same dialog shows the words the backend named, and a confirm repeats the save.
// - Words he confirmed are kept in the names' value and go with every later save while they stay
//   in a name, so he is never asked twice for the same word.

import { useCallback, useRef, useState, type ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { uiErrorOf } from '../api';
import { SpellingConfirmDialog, type SpellingName } from '../ui/SpellingConfirmDialog';
import {
  confirmSpelling,
  confirmedSpelling,
  pendingSpelling,
  wordsStillIn,
  type NameWithMarks,
} from '../urdu/spelling';

/** The error code the backend sends for a name with an unconfirmed rules-only word. */
const SPELLING_UNCONFIRMED = 'ERR_SPELLING_UNCONFIRMED';

/** A letter of the Arabic script block, which Urdu is written in. */
const URDU_LETTER = /[؀-ۿ]/;

interface Asked {
  urdu: SpellingName | null;
  english: SpellingName | null;
}

/** The words the backend named in its refusal, split by the name they belong to. */
export function spellingWordsOf(error: UiError): { urdu: string[]; english: string[] } {
  const urdu: string[] = [];
  const english: string[] = [];
  for (const item of error.errors) {
    const code: string = item.code;
    if (code !== SPELLING_UNCONFIRMED) continue;
    const text = item.values['words'];
    const words = (typeof text === 'string' ? text : String(text ?? ''))
      .split(/[\s,،;|]+/)
      .filter((word) => word !== '');
    for (const word of words) {
      // The field says which name it was (name_ur, father_or_firm_ur ...); the script of the
      // word decides when it does not.
      const field = item.field ?? '';
      const toUrdu = field.endsWith('_ur')
        ? true
        : field.endsWith('_en')
          ? false
          : URDU_LETTER.test(word);
      (toUrdu ? urdu : english).push(word);
    }
  }
  return { urdu, english };
}

interface SpellingFlowOptions {
  /** Called when he chooses to fix the spelling: the screen puts the cursor in the name field. */
  onFix?: (() => void) | undefined;
}

export function useSpellingFlow({ onFix }: SpellingFlowOptions = {}) {
  const [asked, setAsked] = useState<Asked | null>(null);
  const resolver = useRef<((confirmed: boolean) => void) | null>(null);
  // Set when he chose to fix: when the dialog has closed, focus goes to the name, not the button.
  const fixing = useRef(false);

  const answer = useCallback((confirmed: boolean) => {
    resolver.current?.(confirmed);
    resolver.current = null;
    setAsked(null);
  }, []);

  /** Shows the dialog and waits: true when he confirms, false when he goes back to fix. */
  const ask = useCallback(
    (what: Asked) =>
      new Promise<boolean>((resolve) => {
        resolver.current = resolve;
        setAsked(what);
      }),
    [],
  );

  const guard = useCallback(
    async <T,>(
      names: NameWithMarks,
      onNames: (next: NameWithMarks) => void,
      run: (confirmedWords: string[]) => Promise<T>,
    ): Promise<T | undefined> => {
      let current = names;
      const review = async (urduWords: string[], englishWords: string[]) => {
        const confirmed = await ask({
          urdu: urduWords.length === 0 ? null : { text: current.urdu, words: urduWords },
          english:
            englishWords.length === 0 ? null : { text: current.english, words: englishWords },
        });
        if (!confirmed) {
          fixing.current = true;
          return false;
        }
        current = confirmSpelling(current, [...urduWords, ...englishWords]);
        onNames(current);
        return true;
      };

      const pending = pendingSpelling(current);
      if (pending.urdu.length > 0 || pending.english.length > 0) {
        if (!(await review(pending.urdu, pending.english))) return undefined;
      }
      // One refusal by the backend is answered with the dialog and one more try. A second
      // refusal for the same words is not asked again: it is shown as an error.
      for (let attempt = 0; ; attempt += 1) {
        try {
          return await run(confirmedSpelling(current));
        } catch (error) {
          const named = spellingWordsOf(uiErrorOf(error));
          const urduWords = wordsStillIn(current.urdu, named.urdu);
          const englishWords = wordsStillIn(current.english, named.english);
          const known = new Set(confirmedSpelling(current));
          const fresh = [...urduWords, ...englishWords].some((word) => !known.has(word));
          if (attempt > 0 || !fresh) throw error;
          if (!(await review(urduWords, englishWords))) return undefined;
        }
      }
    },
    [ask],
  );

  const dialog: ReactNode = (
    <SpellingConfirmDialog
      open={asked !== null}
      urdu={asked?.urdu ?? null}
      english={asked?.english ?? null}
      onConfirm={() => {
        answer(true);
      }}
      onBack={() => {
        answer(false);
      }}
      onCloseFocus={(event) => {
        if (!fixing.current) return;
        fixing.current = false;
        event.preventDefault();
        onFix?.();
      }}
    />
  );
  return { guard, dialog };
}
