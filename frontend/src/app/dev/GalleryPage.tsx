// Development gallery (/dev/gallery, plan 02 T3): every shared component, in Urdu and English
// side by side, to compare with the design boards. Registered only in development, so it never
// ships (app/router.tsx).

import { Localized } from '@fluent/react';
import { Link } from '@tanstack/react-router';
import type { Language } from '../../i18n';
import { directionOf, useLanguage } from '../../shared/i18n';
import { LanguageSwitch } from '../layout/LanguageSwitch';
import { GalleryLocalization } from './GalleryLocalization';
import {
  ButtonsSection,
  FieldsSection,
  MessagesSection,
  NumbersSection,
  StatesSection,
} from './GallerySections';

const PANES: readonly Language[] = ['ur', 'en'];

function Pane({ language }: { language: Language }) {
  return (
    <GalleryLocalization language={language}>
      <div
        lang={language}
        dir={directionOf(language)}
        data-testid={`gallery-${language}`}
        className="flex min-w-0 flex-col gap-5"
      >
        <Localized id={`common-language-${language}`}>
          <h2 className="type-section text-primary" />
        </Localized>
        <ButtonsSection />
        <FieldsSection />
        <NumbersSection />
        <StatesSection />
        <MessagesSection />
      </div>
    </GalleryLocalization>
  );
}

export function GalleryPage() {
  const language = useLanguage();
  return (
    <GalleryLocalization language={language}>
      <main className="mx-auto flex w-full max-w-page flex-col gap-5 p-5">
        <header className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex flex-col gap-1">
            <Localized id="gallery-back-to-start">
              <Link to="/" className="type-body text-primary underline" />
            </Localized>
            <Localized id="gallery-title">
              <h1 className="type-title" />
            </Localized>
            <Localized id="gallery-note">
              <p className="type-body text-ink-muted" />
            </Localized>
          </div>
          <LanguageSwitch />
        </header>
        <div className="grid grid-cols-1 gap-6 normal:grid-cols-2">
          {PANES.map((pane) => (
            <Pane key={pane} language={pane} />
          ))}
        </div>
      </main>
    </GalleryLocalization>
  );
}
