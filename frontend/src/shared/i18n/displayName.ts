// The one way a name is shown (Code Standards F38, PRD FR-7.24): the Urdu name in the Urdu app and
// the English name in the English app, with the tag (نشان) of an account after it ("عاشق ظفر K",
// "Ashiq Zafar K", PRD 10.2). The tag is added here and nowhere else, so every screen and every
// picker row shows it the same way. Components never read name_ur or name_en themselves.

import type { Language } from '../../i18n';

/** A record that has a name in both scripts. */
export interface Named {
  name_ur: string;
  name_en: string;
  /** The tag after the name. Absent, null or empty when the account has none. */
  tag?: string | null | undefined;
}

/** The name with a space and its tag after it, or the name alone when there is no tag. */
function withTag(name: string, tag: string | null | undefined): string {
  return tag === undefined || tag === null || tag === '' ? name : `${name} ${tag}`;
}

export function displayName(record: Named, language: Language): string {
  return withTag(language === 'ur' ? record.name_ur : record.name_en, record.tag);
}

/** The language of the script the app is not in, for a `lang` attribute on the other name. */
export function otherScriptLang(language: Language): Language {
  return language === 'ur' ? 'en' : 'ur';
}

/**
 * The name in the script the app is not in, for a dimmed second line that tells two records
 * apart (the English name on an Urdu screen, the Urdu name on an English one).
 */
export function otherScriptName(record: Named, language: Language): string {
  return withTag(language === 'ur' ? record.name_en : record.name_ur, record.tag);
}
