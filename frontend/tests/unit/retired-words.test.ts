// Words the glossary has retired must not come back in a Fluent file (PRD Section 3 glossary v1.43, NFR-URD-2,
// NFR-URD-7). سابقہ رقم is gone: پچھلا بقایا replaces it on every slip, bill and help text, and it is written
// through the glossary term `-sabiqa-raqam`, so one word is spelt in one place. In English it is "Previous
// Balance", not "previous due". Plan 80 R5-5.

import { describe, expect, it } from 'vitest';
import { LANGUAGES, bundles } from '../../src/i18n';

const sources = {
  ur: import.meta.glob<string>('../../src/i18n/ur/*.ftl', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
  en: import.meta.glob<string>('../../src/i18n/en/*.ftl', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
} as const;

/** A retired word, the language it was written in, and the glossary word that replaces it. */
const RETIRED: readonly (readonly ['ur' | 'en', string, string])[] = [
  ['ur', 'سابقہ رقم', 'پچھلا بقایا'],
  ['en', 'previous due', 'Previous Balance'],
];

function format(language: 'ur' | 'en', id: string): string {
  const bundle = bundles[language];
  const message = bundle.getMessage(id);
  if (message?.value === null || message?.value === undefined) {
    throw new Error(`no ${language} ${id}`);
  }
  return bundle.formatPattern(message.value, {}, []);
}

describe('retired glossary words', () => {
  it.each(RETIRED)('%s: "%s" is in no Fluent file (the word is %s)', (language, retired) => {
    const found = Object.entries(sources[language])
      .filter(([, source]) => source.toLowerCase().includes(retired.toLowerCase()))
      .map(([file]) => file.split('/').at(-1));
    expect(found, `"${retired}" is still written in: ${found.join(', ')}`).toEqual([]);
  });

  it('the glossary term is the new word in both languages', () => {
    for (const language of LANGUAGES) {
      const text = format(language, 'glossary-sabiqa-raqam');
      expect(text).toBe(language === 'ur' ? 'پچھلا بقایا' : 'Previous Balance');
    }
  });
});
