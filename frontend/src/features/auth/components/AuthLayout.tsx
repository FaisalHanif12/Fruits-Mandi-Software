// The frame of the sign-in and recovery screens (board LoginUr and LoginEn): a brand panel on the
// start side with the app name, the firm and the version, and the form on the other. It stacks
// in the compact layout. The language switch sits above the form and Alt+L works (PRD FR-7.30).

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import type { FirmDisplay } from '../../../bindings/FirmDisplay';
import { LanguageSwitch } from '../../../app/layout/LanguageSwitch';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Code } from '../../../shared/ui';

interface AuthLayoutProps {
  firm: FirmDisplay;
  version: string;
  children: ReactNode;
}

export function AuthLayout({ firm, version, children }: AuthLayoutProps) {
  const language = useLanguage();
  const firmName = displayName({ name_ur: firm.name_ur, name_en: firm.name_en }, language);
  const mandiName = displayName({ name_ur: firm.mandi_ur, name_en: firm.mandi_en }, language);
  return (
    <div className="flex min-h-screen flex-col bg-bg text-ink normal:flex-row">
      <aside className="flex flex-col justify-between gap-6 bg-nav p-6 text-on-primary normal:w-nav normal:shrink-0 wide:w-module-menu">
        <div className="flex flex-col gap-1">
          <Localized id="common-app-name">
            <p className="type-title" />
          </Localized>
          <p className="type-lead text-nav-ink">{firmName}</p>
          <p className="type-body text-nav-ink">{mandiName}</p>
        </div>
        <Localized id="common-version" vars={{ version }} elems={{ code: <Code /> }}>
          <p className="type-body text-nav-ink" />
        </Localized>
      </aside>
      <main className="flex min-w-0 flex-1 flex-col gap-5 p-6">
        <div className="flex justify-end">
          <LanguageSwitch />
        </div>
        <div className="flex flex-1 items-center justify-center">
          <div className="flex w-full max-w-dialog flex-col gap-4 rounded-3 border border-border bg-surface p-6">
            {children}
          </div>
        </div>
        <Localized id="common-language-hint" vars={{ key: SHORTCUTS.languageToggle.label }}>
          <p className="type-body text-center text-ink-muted" />
        </Localized>
      </main>
    </div>
  );
}
