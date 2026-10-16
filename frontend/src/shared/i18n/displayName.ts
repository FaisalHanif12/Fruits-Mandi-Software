// The one way a name is shown (Code Standards F38, PRD FR-7.24): the Urdu name in the Urdu app and
// the English name in the English app. Components never read name_ur or name_en themselves.

import type { Language } from '../../i18n';

/** A record that has a name in both scripts. */
export interface Named {
  name_ur: string;
  name_en: string;
}

export function displayName(record: Named, language: Language): string {
  return language === 'ur' ? record.name_ur : record.name_en;
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
  return language === 'ur' ? record.name_en : record.name_ur;
}
