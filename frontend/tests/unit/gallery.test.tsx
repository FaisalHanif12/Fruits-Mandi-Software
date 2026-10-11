// The development gallery (plan 02 T3): every component in Urdu and English side by side, with
// its own words kept out of the shipped Fluent files.

import { fireEvent, screen, within } from '@testing-library/react';
import path from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { IDENTICAL_ALLOWED, checkI18n } from '../../../scripts/i18n-check-lib.mjs';
import { GalleryPage } from '../../src/app/dev/GalleryPage';
import { LANGUAGES } from '../../src/i18n';
import { directionOf, useLanguageStore } from '../../src/shared/i18n';
import { message, visible } from '../helpers/render';
import { renderRouted } from '../helpers/router';

const GALLERY_I18N = path.resolve(import.meta.dirname, '../../src/app/dev/i18n');

beforeEach(() => {
  useLanguageStore.setState({ language: 'ur' });
});

describe('/dev/gallery', () => {
  it('its Urdu and English words match, like the app files (NFR-URD-22)', () => {
    // "20 نگ @ 600" is the same markup in both files; only the glossary term differs.
    const allowed = new Set([...IDENTICAL_ALLOWED, 'gallery-numbers-line']);
    expect(checkI18n(GALLERY_I18N, { identicalAllowed: allowed })).toEqual([]);
  });

  it('shows every component once per language, each pane in its own language and direction', async () => {
    await renderRouted({ '/dev/gallery': GalleryPage }, '/dev/gallery');
    for (const language of LANGUAGES) {
      const pane = screen.getByTestId(`gallery-${language}`);
      expect(pane).toHaveAttribute('lang', language);
      expect(pane).toHaveAttribute('dir', directionOf(language));
      const inPane = within(pane);
      // Buttons, six fields and the backend-error cell, numbers, the states and the overlay triggers.
      expect(inPane.getAllByRole('textbox')).toHaveLength(7);
      expect(inPane.getByRole('combobox')).toBeInTheDocument();
      expect(inPane.getByRole('status')).toHaveAttribute('aria-busy', 'true');
      // The error state, the data-check state, and the notice of the backend-errors section.
      const alerts = inPane.getAllByRole('alert');
      expect(alerts).toHaveLength(3);
      expect(alerts[0]).toHaveTextContent(message(language, 'common-retry'));
      const dataCheck = visible(alerts[1]?.textContent);
      expect(dataCheck).toContain(message(language, 'common-data-check-title'));
      expect(dataCheck).toContain(message(language, 'err-data-check-failed'));
      expect(dataCheck).toContain(message(language, 'common-retry'));
      // The cell problem sits at its field; the notice keeps only the whole-form one.
      const notice = visible(alerts[2]?.textContent);
      expect(notice).toContain(
        message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }),
      );
      expect(notice).not.toContain(message(language, 'err-qty-too-large', { max: '1,000,000' }));
      expect(inPane.getAllByText('1,316,040').length).toBeGreaterThan(0);
      expect(inPane.getByRole('button', { name: /F10/ })).toBeInTheDocument();
    }
  });

  it.each([
    ['ur', 'شروع کے صفحے پر واپس'],
    ['en', 'Back to the start page'],
  ] as const)('has a link back to the start page in %s', async (language, name) => {
    useLanguageStore.setState({ language });
    await renderRouted(
      { '/dev/gallery': GalleryPage, '/': () => <p data-testid="start-stub" /> },
      '/dev/gallery',
    );
    const link = screen.getByRole('link', { name });
    expect(link).toHaveAttribute('href', '/');
    fireEvent.click(link);
    expect(await screen.findByTestId('start-stub')).toBeInTheDocument();
  });
});
