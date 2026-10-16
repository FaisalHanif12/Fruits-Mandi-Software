// One `language` value drives the whole app: html dir and lang, the Fluent bundle, the Radix
// direction and, later, grids and charts (Tech Stack 4.3, Architecture 18).

import { create } from 'zustand';
import { DEFAULT_LANGUAGE, type Language } from '../../i18n';

export type Direction = 'rtl' | 'ltr';

/** Urdu is right to left, English left to right (PRD FR-7.31). */
export function directionOf(language: Language): Direction {
  return language === 'ur' ? 'rtl' : 'ltr';
}

interface LanguageState {
  language: Language;
  setLanguage: (language: Language) => void;
  toggleLanguage: () => void;
}

/**
 * The app language. Saving the choice per user (settings_set_pref) comes with the shell in
 * plan 04; until then every start is in Urdu.
 */
export const useLanguageStore = create<LanguageState>()((set) => ({
  language: DEFAULT_LANGUAGE,
  setLanguage: (language) => {
    set({ language });
  },
  toggleLanguage: () => {
    set((state) => ({ language: state.language === 'ur' ? 'en' : 'ur' }));
  },
}));

/** The current app language. */
export function useLanguage(): Language {
  return useLanguageStore((state) => state.language);
}
