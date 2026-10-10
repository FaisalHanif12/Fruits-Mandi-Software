// The first screen (plan 01 T3.5): app name and version, in Urdu by default, Alt+L for English.
// A calm placeholder until the login screen of plan 04. Four states: loading, error and data
// (there is no empty state: the version always exists).

import { Localized } from '@fluent/react';
import { Suspense, lazy } from 'react';
import { hasAppBridge, uiErrorOf } from '../../shared/api';
import { SHORTCUTS } from '../../shared/keyboard';
import { Button, Code, ErrorNotice } from '../../shared/ui';
import { LanguageSwitch } from '../layout/LanguageSwitch';
import { useAppInfo } from './useAppInfo';

// Development only. The helpers are loaded through import.meta.env.DEV, so a build has neither
// the code nor its words. In a build both constants are null and the branches below are removed.
const BrowserPreviewNote = import.meta.env.DEV
  ? lazy(() => import('../dev/StartDevTools').then((m) => ({ default: m.BrowserPreviewNote })))
  : null;
const GalleryLink = import.meta.env.DEV
  ? lazy(() => import('../dev/StartDevTools').then((m) => ({ default: m.GalleryLink })))
  : null;

function AppVersionLine() {
  const query = useAppInfo();
  if (query.isPending) {
    return (
      <Localized id="common-loading">
        <p className="type-body text-ink-muted" aria-live="polite" />
      </Localized>
    );
  }
  if (query.isError) {
    return (
      <div className="flex flex-col items-start gap-3">
        <ErrorNotice error={uiErrorOf(query.error)} />
        <Button
          onClick={() => {
            void query.refetch();
          }}
        >
          <Localized id="common-retry">
            <span />
          </Localized>
        </Button>
      </div>
    );
  }
  return (
    <Localized
      id="common-version"
      vars={{ version: query.data.version }}
      elems={{ code: <Code /> }}
    >
      <p className="type-body text-ink-muted" data-testid="app-version" />
    </Localized>
  );
}

function VersionLine() {
  // A plain browser tab has no app bridge, so app_info cannot answer. Say so, in development.
  if (BrowserPreviewNote !== null && !hasAppBridge()) {
    return (
      <Suspense fallback={null}>
        <BrowserPreviewNote />
      </Suspense>
    );
  }
  return <AppVersionLine />;
}

export function StartPage() {
  return (
    <div className="flex min-h-screen flex-col bg-bg text-ink">
      <header className="flex justify-end px-6 py-5">
        <LanguageSwitch />
      </header>
      <main className="flex flex-1 items-center px-6 pb-7">
        <section className="mx-auto flex w-full max-w-reading flex-col gap-4">
          <div className="flex flex-col gap-3">
            <Localized id="common-app-name">
              <h1 className="type-title text-primary" />
            </Localized>
            <div className="ledger-rule w-full" aria-hidden="true" />
            <Localized id="common-app-tagline">
              <p className="type-lead text-ink" />
            </Localized>
          </div>
          <VersionLine />
        </section>
      </main>
      <footer className="flex flex-wrap items-center justify-between gap-4 px-6 pb-5">
        <Localized id="common-language-hint" vars={{ key: SHORTCUTS.languageToggle.label }}>
          <p className="type-body text-ink-muted" />
        </Localized>
        {GalleryLink === null ? null : (
          <Suspense fallback={null}>
            <GalleryLink />
          </Suspense>
        )}
      </footer>
    </div>
  );
}
