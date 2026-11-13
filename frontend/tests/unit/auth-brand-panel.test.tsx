// The brand panel beside the sign-in form (plan 75 T2, PRD FR-5.6 area, Section 8.1), in Urdu and
// English: a clear order (mark, app name, firm, mandi, short lines, version), the app name on one
// line whatever the window, no hand-written side or text, and a picture that a reader skips.
// The width and the one-line check of a real window are made in the browser (plan 75 screenshots);
// here the class that keeps the name on one line is held to.

import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { AuthLayout } from '../../src/features/auth/components/AuthLayout';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { message, renderIn, visible } from '../helpers/render';

const FIRM = {
  name_ur: 'مہر عدنان اینڈ برادرز',
  name_en: 'Maher Adnan and Brothers',
  mandi_ur: 'نئی سبزی منڈی، کوٹ رادھا کشن',
  mandi_en: 'New Sabzi Mandi, Kot Radha Kishan',
};

/** A line by its whole visible text, without the isolation marks Fluent puts around a term. */
const line = (text: string) => (_content: string, element: Element | null) =>
  element !== null && element.children.length === 0 && visible(element.textContent) === text;

describe.each(LANGUAGES)('sign-in brand panel (%s)', (language) => {
  function renderPanel() {
    useLanguageStore.setState({ language });
    renderIn(
      language,
      <AuthLayout firm={FIRM} version="0.1.12">
        <div data-testid="form-slot" />
      </AuthLayout>,
    );
    return screen.getByRole('complementary');
  }

  it('shows the app name, the firm and the mandi in the language of the screen, in that order', () => {
    const panel = renderPanel();
    const firm = language === 'ur' ? FIRM.name_ur : FIRM.name_en;
    const mandi = language === 'ur' ? FIRM.mandi_ur : FIRM.mandi_en;
    const lines = Array.from(panel.querySelectorAll('p')).map((p) => visible(p.textContent));
    const at = (text: string) => lines.findIndex((p) => p.includes(text));
    expect(at(message(language, 'common-app-name'))).toBe(0);
    expect(at(message(language, 'common-app-tagline'))).toBe(1);
    expect(at(firm)).toBe(2);
    expect(at(mandi)).toBe(3);
    // The other language's names are not drawn.
    expect(panel.textContent).not.toContain(language === 'ur' ? FIRM.name_en : FIRM.name_ur);
  });

  it('keeps the app name on one line and never splits a word of it', () => {
    const panel = renderPanel();
    const name = within(panel).getByText(message(language, 'common-app-name'));
    expect(name).toHaveClass('whitespace-nowrap');
  });

  it('wraps the firm, the mandi and the tagline only between words, in balanced lines', () => {
    const panel = renderPanel();
    for (const text of [
      message(language, 'common-app-tagline'),
      language === 'ur' ? FIRM.name_ur : FIRM.name_en,
      language === 'ur' ? FIRM.mandi_ur : FIRM.mandi_en,
    ]) {
      expect(within(panel).getByText(line(text))).toHaveClass('text-balance');
    }
  });

  it('says three short things about the app, from Fluent, and the version with its number kept in order', () => {
    const panel = renderPanel();
    for (const id of ['auth-brand-books', 'auth-brand-offline', 'auth-brand-backup']) {
      expect(within(panel).getByText(message(language, id))).toBeInTheDocument();
    }
    const version = panel.querySelector('bdi[data-kind="code"]');
    // The number is its own element, isolated and written left to right like every code (F25).
    expect(visible(version?.textContent)).toBe('0.1.12');
    expect(version).toHaveAttribute('dir', 'ltr');
  });

  it('draws its picture as decoration that a screen reader skips, and uses no hand-written side', () => {
    const panel = renderPanel();
    const picture = panel.querySelector('svg.absolute');
    expect(picture).toHaveAttribute('aria-hidden', 'true');
    // The rings stay in the corner at the end side: placed with `end`, mirrored in the right-to-left screen.
    expect(picture).toHaveClass('end-0', 'rtl:-scale-x-100');
    expect(panel.outerHTML).not.toMatch(/\b(?:left|right|ml-|mr-|pl-|pr-)/);
  });
});
