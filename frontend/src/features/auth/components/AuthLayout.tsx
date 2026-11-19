// The frame of the sign-in and recovery screens (board LoginUr and LoginEn): a brand panel on the
// start side and the form on the other. The panel (plan 75 T2) has a small mark and the app name on
// one line, then the firm and the mandi, then two short lines about the app and the version. It is
// wide enough for the name at every supported window size (1024 px and up, NFR-USE-15), so it never
// stacks. The language switch sits above the form and Alt+L works (PRD FR-7.30).

import { Localized } from '@fluent/react';
import { DatabaseBackup, Scale, WifiOff, type LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import type { FirmDisplay } from '../../../bindings/FirmDisplay';
import { LanguageSwitch } from '../../../app/layout/LanguageSwitch';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS } from '../../../shared/keyboard';
import { BrandMark, Code } from '../../../shared/ui';
import { BrandRings } from './BrandRings';

interface AuthLayoutProps {
  firm: FirmDisplay;
  version: string;
  children: ReactNode;
}

interface BrandPoint {
  id: string;
  icon: LucideIcon;
}

/** The short lines about the app (plan 75 §7.1): PRD G4, NFR-REL and NFR-BAK-2, said quietly. */
const BRAND_POINTS: readonly BrandPoint[] = [
  { id: 'auth-brand-books', icon: Scale },
  { id: 'auth-brand-offline', icon: WifiOff },
  { id: 'auth-brand-backup', icon: DatabaseBackup },
];

export function AuthLayout({ firm, version, children }: AuthLayoutProps) {
  const language = useLanguage();
  const firmName = displayName({ name_ur: firm.name_ur, name_en: firm.name_en }, language);
  const mandiName = displayName({ name_ur: firm.mandi_ur, name_en: firm.mandi_en }, language);
  return (
    <div className="flex min-h-screen bg-bg text-ink">
      <aside className="relative flex w-auth-panel shrink-0 flex-col justify-between gap-6 overflow-hidden bg-nav p-6 text-on-primary wide:w-auth-panel-wide wide:p-7">
        <BrandRings />
        <div className="relative flex flex-col gap-3">
          <BrandMark />
          <div className="flex flex-col">
            {/* One line at every width: the panel is as wide as the name needs, and the name never
                breaks inside (a word is never split, NFR-URD-10). */}
            <Localized id="common-app-name">
              <p className="type-title whitespace-nowrap" />
            </Localized>
            <Localized id="common-app-tagline">
              <p className="type-body text-balance text-nav-ink-muted" />
            </Localized>
          </div>
        </div>
        <div className="relative flex flex-col gap-1 border-t border-nav-active pt-5">
          <p className="type-section text-balance text-on-primary">{firmName}</p>
          <p className="type-body text-balance text-nav-ink">{mandiName}</p>
        </div>
        <div className="relative flex flex-col gap-4">
          <ul className="flex flex-col gap-2">
            {BRAND_POINTS.map(({ id, icon: Icon }) => (
              <li key={id} className="type-body flex items-start gap-3 text-nav-ink">
                {/* The icon stands in the first line of its text, whatever the line height of the script. */}
                <span className="flex h-lh shrink-0 items-center text-nav-ink-muted">
                  <Icon aria-hidden="true" className="size-icon-md" />
                </span>
                <Localized id={id}>
                  <span className="text-balance" />
                </Localized>
              </li>
            ))}
          </ul>
          <Localized id="common-version" vars={{ version }} elems={{ code: <Code /> }}>
            <p className="type-body text-nav-ink-muted" />
          </Localized>
        </div>
      </aside>
      <main className="flex min-w-0 flex-1 flex-col gap-5 p-6 shortest:gap-3 shortest:p-4">
        <div className="flex justify-end">
          <LanguageSwitch />
        </div>
        <div className="flex flex-1 items-center justify-center">
          <div className="flex w-full max-w-dialog flex-col gap-4 rounded-3 border border-border bg-surface p-6 shortest:p-5">
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
