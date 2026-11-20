// Puts the current language on the page: Fluent messages, <html dir lang>, and Radix direction.
// Switching changes only what is shown, never the data (PRD FR-7.32).

import { LocalizationProvider } from '@fluent/react';
import { Direction } from 'radix-ui';
import { useEffect, useMemo, type ReactNode } from 'react';
import { createLocalization } from '../../i18n/localization';
import { directionOf, useLanguage } from './language';

interface I18nProviderProps {
  children: ReactNode;
}

export function I18nProvider({ children }: I18nProviderProps) {
  const language = useLanguage();
  const direction = directionOf(language);
  const l10n = useMemo(() => createLocalization(language), [language]);

  useEffect(() => {
    document.documentElement.lang = language;
    document.documentElement.dir = direction;
    document.title = l10n.getString('common-app-name');
  }, [language, direction, l10n]);

  return (
    <LocalizationProvider l10n={l10n}>
      <Direction.Provider dir={direction}>{children}</Direction.Provider>
    </LocalizationProvider>
  );
}
