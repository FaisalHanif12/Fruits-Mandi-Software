// The words of a printed document come from the PRINT language, not the app language (PRD
// FR-7.36, Code Standards F37): an Urdu bill can be printed while the app is in English. This
// gives the children a Fluent bundle of the print language and leaves the app language alone.
// Both bundles are built once at start (i18n/index.ts), so nothing loads.

import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { Direction } from 'radix-ui';
import { useMemo, type ReactNode } from 'react';
import { bundles, type Language } from '../../i18n';
import { directionOf } from '../i18n';

interface PrintProviderProps {
  language: Language;
  children: ReactNode;
}

export function PrintProvider({ language, children }: PrintProviderProps) {
  const l10n = useMemo(() => new ReactLocalization([bundles[language]]), [language]);
  return (
    <LocalizationProvider l10n={l10n}>
      <Direction.Provider dir={directionOf(language)}>{children}</Direction.Provider>
    </LocalizationProvider>
  );
}
