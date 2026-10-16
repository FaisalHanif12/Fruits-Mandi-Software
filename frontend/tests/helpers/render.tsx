// Test helper: render a component inside one language, the way the app does (Fluent bundle,
// Radix direction, and an element carrying lang and dir).

import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { render, type RenderResult } from '@testing-library/react';
import { Direction } from 'radix-ui';
import type { ReactNode } from 'react';
import { bundles, type Language } from '../../src/i18n';
import { directionOf } from '../../src/shared/i18n';

export function renderIn(language: Language, ui: ReactNode): RenderResult {
  const l10n = new ReactLocalization([bundles[language]]);
  const dir = directionOf(language);
  return render(
    <LocalizationProvider l10n={l10n}>
      <Direction.Provider dir={dir}>
        <div lang={language} dir={dir} data-testid="language-root">
          {ui}
        </div>
      </Direction.Provider>
    </LocalizationProvider>,
  );
}

/** Fluent wraps variables in bidi isolation marks; tests compare the visible text. */
export function visible(text: string | null | undefined): string {
  return (text ?? '').replace(/[⁨⁩]/g, '');
}

/** The text of one Fluent message in one language, without isolation marks. */
export function message(
  language: Language,
  id: string,
  args: Record<string, string | number> = {},
): string {
  const bundle = bundles[language];
  const pattern = bundle.getMessage(id)?.value;
  if (pattern == null) throw new Error(`${language}: no message ${id}`);
  // The <num> and <name> markup of a message is drawn as a number or a name in the page, so its
  // text is what is compared.
  return visible(bundle.formatPattern(pattern, args)).replace(/<\/?(num|name)>/g, '');
}
