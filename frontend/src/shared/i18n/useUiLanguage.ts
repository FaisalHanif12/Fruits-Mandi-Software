// The language of the Fluent messages around a component. It is the app language everywhere
// except inside the development gallery, which shows Urdu and English side by side. Parts that
// render in a portal (dialogs, select lists, toasts) leave the page's `lang` and `dir`, so they
// set both again from this.

import { useLocalization } from '@fluent/react';
import { DEFAULT_LANGUAGE, LANGUAGES, type Language } from '../../i18n';
import { directionOf, type Direction } from './language';

export interface UiLanguage {
  language: Language;
  direction: Direction;
}

export function useUiLanguage(): UiLanguage {
  const { l10n } = useLocalization();
  let language: Language = DEFAULT_LANGUAGE;
  for (const bundle of l10n.bundles) {
    const locale = bundle.locales[0];
    language = LANGUAGES.find((candidate) => candidate === locale) ?? DEFAULT_LANGUAGE;
    break;
  }
  return { language, direction: directionOf(language) };
}
