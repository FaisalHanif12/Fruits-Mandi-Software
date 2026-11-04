// The very first screen of a new install, before anything is set up (PRD FR-20.11, NFR-INS-2): two
// big choices. A new shop opens the first-run wizard as it always was. Bringing back the data opens
// the restore pages. The language switch is here too, so the Owner can read it in his own language.
// If a restore failed, the reason is shown above the choices and nothing was changed on this
// computer (plan 56 Section 3, step 7).

import { Localized } from '@fluent/react';
import { DatabaseBackup, Store } from 'lucide-react';
import { LanguageSwitch } from '../../../app/layout/LanguageSwitch';
import { useEffect, useRef } from 'react';
import { SHORTCUTS, formNavigationProps } from '../../../shared/keyboard';
import { ChoiceCard, ErrorNotice, Notice } from '../../../shared/ui';
import { useRestoreStore } from '../store';

export function StartPage() {
  const failure = useRestoreStore((state) => state.failure);
  const setChoice = useRestoreStore((state) => state.setChoice);
  // The first choice has focus, so Enter starts a new shop and the arrows reach the other one.
  const choices = useRef<HTMLDivElement>(null);
  useEffect(() => {
    choices.current?.querySelector<HTMLElement>('[data-nav-field]')?.focus();
  }, []);
  return (
    <div className="flex min-h-screen flex-col gap-5 bg-bg p-6 text-ink">
      <header className="flex flex-wrap items-center justify-between gap-4 print:hidden">
        <Localized id="common-app-name">
          <h1 className="type-title" />
        </Localized>
        <LanguageSwitch />
      </header>
      <div className="mx-auto flex w-full max-w-form flex-1 flex-col justify-center gap-5">
        <div className="flex flex-col gap-2">
          <Localized id="restore-start-title">
            <h2 className="type-section" />
          </Localized>
          <Localized id="restore-start-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        {failure === null ? null : (
          <Notice tone="error">
            <span className="flex flex-col gap-1">
              <Localized id="restore-start-failed-title">
                <strong className="type-body" />
              </Localized>
              <ErrorNotice error={failure} />
              <Localized id="restore-start-failed-help">
                <span className="type-body text-ink-muted" />
              </Localized>
            </span>
          </Notice>
        )}
        {/* Arrow keys move between the two cards in the direction the user sees (PRD FR-7.13). */}
        <div ref={choices} {...formNavigationProps} className="flex flex-col gap-4 normal:flex-row">
          <ChoiceCard
            icon={Store}
            title={<Localized id="restore-start-new-title" />}
            help={<Localized id="restore-start-new-help" />}
            onChoose={() => {
              setChoice('new');
            }}
          />
          <ChoiceCard
            icon={DatabaseBackup}
            title={<Localized id="restore-start-restore-title" />}
            help={<Localized id="restore-start-restore-help" />}
            onChoose={() => {
              setChoice('restore');
            }}
          />
        </div>
        <Localized id="common-language-hint" vars={{ key: SHORTCUTS.languageToggle.label }}>
          <p className="type-body text-center text-ink-muted" />
        </Localized>
      </div>
    </div>
  );
}
