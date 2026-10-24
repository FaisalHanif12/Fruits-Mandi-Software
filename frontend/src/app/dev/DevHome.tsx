// Development only: what a plain browser tab shows instead of the app. There is no Tauri bridge in
// a browser tab, so no command can answer. The page says so and lists the development pages that
// need no backend. It and its words never reach a build (AppFrame loads it through
// import.meta.env.DEV).

import { Localized } from '@fluent/react';
import { Link } from '@tanstack/react-router';
import { useLanguage } from '../../shared/i18n';
import { LanguageSwitch } from '../layout/LanguageSwitch';
import { GalleryLocalization } from './GalleryLocalization';
import { BrowserPreviewNote } from './StartDevTools';

export function DevHome() {
  const language = useLanguage();
  return (
    <GalleryLocalization language={language}>
      <main className="mx-auto flex min-h-screen w-full max-w-reading flex-col gap-5 p-6">
        <header className="flex flex-wrap items-center justify-between gap-4">
          <Localized id="common-app-name">
            <h1 className="type-title text-primary" />
          </Localized>
          <LanguageSwitch />
        </header>
        <BrowserPreviewNote />
        <nav className="type-body flex flex-col gap-2">
          <Localized id="gallery-title">
            <Link to="/dev/gallery" className="text-primary underline" />
          </Localized>
          <Localized id="gridpage-title">
            <Link to="/dev/grid" className="text-primary underline" />
          </Localized>
          <Localized id="shellpage-title">
            <Link to="/dev/shell" className="text-primary underline" />
          </Localized>
          <Localized id="bikripage-title">
            <Link to="/dev/bikri" className="text-primary underline" />
          </Localized>
          <Localized id="voucherspage-title">
            <Link to="/dev/vouchers" className="text-primary underline" />
          </Localized>
          <Localized id="gardenspage-title">
            <Link to="/dev/gardens" className="text-primary underline" />
          </Localized>
          <Localized id="trippage-title">
            <Link to="/dev/trips" className="text-primary underline" />
          </Localized>
          <Localized id="rokarpage-title">
            <Link to="/dev/rokar" className="text-primary underline" />
          </Localized>
        </nav>
      </main>
    </GalleryLocalization>
  );
}
