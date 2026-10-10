// Fluent and direction for one language, with the gallery's own words added. The gallery shows
// Urdu and English side by side, so each half has its own language, whatever the app language.

import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { Direction } from 'radix-ui';
import { useMemo, type ReactNode } from 'react';
import { createBundle, type Language } from '../../i18n';
import { directionOf } from '../../shared/i18n';

const gallerySources: Record<Language, Record<string, string>> = {
  ur: import.meta.glob<string>('./i18n/ur/*.ftl', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
  en: import.meta.glob<string>('./i18n/en/*.ftl', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
};

const galleryBundles = {
  ur: createBundle('ur', gallerySources.ur),
  en: createBundle('en', gallerySources.en),
};

interface GalleryLocalizationProps {
  language: Language;
  children: ReactNode;
}

export function GalleryLocalization({ language, children }: GalleryLocalizationProps) {
  const l10n = useMemo(() => new ReactLocalization([galleryBundles[language]]), [language]);
  return (
    <LocalizationProvider l10n={l10n}>
      <Direction.Provider dir={directionOf(language)}>{children}</Direction.Provider>
    </LocalizationProvider>
  );
}
