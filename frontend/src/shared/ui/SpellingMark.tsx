// The amber "check the spelling" note under a name that holds a word made by the spelling rules
// alone (PRD FR-7.66, design system 2: amber is warning, with an icon and a word, never colour
// only). It lists the words to check. It shows nothing when there is no such word.

import { Localized } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';

interface SpellingMarkProps {
  /** The words to check, as they stand in the name. */
  words: readonly string[];
  /** The script of the words: Urdu words are written right to left. */
  lang: 'ur' | 'en';
}

export function SpellingMark({ words, lang }: SpellingMarkProps) {
  if (words.length === 0) return null;
  return (
    <p
      role="status"
      className="flex flex-wrap items-center gap-2 rounded-1 border border-warn-border bg-warn-soft px-3 py-1 type-body text-warn"
    >
      <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
      <Localized id="translit-spelling-check" />
      {words.map((word) => (
        <bdi key={word} lang={lang} dir={lang === 'ur' ? 'rtl' : 'ltr'} className="font-semibold">
          {word}
        </bdi>
      ))}
    </p>
  );
}
