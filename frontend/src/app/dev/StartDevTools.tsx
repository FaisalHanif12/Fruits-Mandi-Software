// Development helpers for the start page. They exist only in development: the start page loads
// this file through `import.meta.env.DEV`, so it and its words never reach a build (plan 02 T3).
// The words are in app/dev/i18n, not in the shipped Fluent files.

import { Localized } from '@fluent/react';
import { Link } from '@tanstack/react-router';
import { Info } from 'lucide-react';
import { useLanguage } from '../../shared/i18n';
import { GalleryLocalization } from './GalleryLocalization';

/**
 * Shown in place of the version when the page runs in a plain browser tab, where there is no
 * Tauri bridge and app_info cannot answer. It says so, instead of showing an error state.
 */
export function BrowserPreviewNote() {
  const language = useLanguage();
  return (
    <GalleryLocalization language={language}>
      <div
        role="note"
        data-testid="browser-preview-note"
        className="flex items-start gap-3 rounded-2 border border-warn-border bg-warn-soft p-4 text-warn"
      >
        <Info aria-hidden="true" className="mt-1 size-icon-lg shrink-0" />
        <div className="flex flex-col gap-1">
          <Localized id="gallery-browser-preview-title">
            <p className="type-label" />
          </Localized>
          <Localized id="gallery-browser-preview-note">
            <p className="type-body" />
          </Localized>
        </div>
      </div>
    </GalleryLocalization>
  );
}

/** The link from the start page to the component gallery. */
export function GalleryLink() {
  const language = useLanguage();
  return (
    <GalleryLocalization language={language}>
      <Localized id="gallery-title">
        <Link to="/dev/gallery" className="type-body text-primary underline" />
      </Localized>
    </GalleryLocalization>
  );
}
