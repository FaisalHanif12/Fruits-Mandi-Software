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

/**
 * A matcher for `getByText` and `findByText` that finds a paragraph by ALL its text. A line of words
 * with a number in it holds the number in its own element (`<Num>`, Code Standards F25), so the
 * paragraph has no text of its own that Testing Library would compare; this reads the whole of it.
 */
export function paragraph(text: string) {
  return (_content: string, element: Element | null): boolean =>
    element?.tagName === 'P' && visible(element.textContent) === text;
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
  // A message that selects on a variable the caller did not give reads its default variant, the way the
  // app does; any other error is a real one. Two such selectors exist: the closed-day message selects
  // on `today_state`, and the out-of-range message on `reason` (the cloud address; the app sends an
  // empty `reason` when the backend sends none, so the plain range reads).
  const errors: Error[] = [];
  const text = bundle.formatPattern(pattern, args, errors);
  const SELECTORS = ['Unknown variable: $today_state', 'Unknown variable: $reason'];
  const real = errors.filter((error) => !SELECTORS.some((known) => error.message.includes(known)));
  if (real[0] !== undefined) throw real[0];
  return visible(text).replace(/<\/?(num|name|money|key)>/g, '');
}
