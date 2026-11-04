// The restore pages of a new computer (PRD FR-20.11, NFR-INS-6, NFR-REL-4): where the copy is, the
// copy and its key, what is in it, and the restore itself. One page at a time, with the steps shown
// above. Everything typed lives in the restore store, so a language switch loses nothing (Code
// Standards F39). Esc goes back one step (PRD FR-7.18); while the books are being put in place it
// does nothing, so a stray key cannot leave a half-finished restore.

import { Localized, useLocalization } from '@fluent/react';
import { LanguageSwitch } from '../../../app/layout/LanguageSwitch';
import { useEscape } from '../../../shared/keyboard';
import { Stepper } from '../../../shared/ui';
import { CheckStep } from '../components/CheckStep';
import { CopyStep } from '../components/CopyStep';
import { RunStep } from '../components/RunStep';
import { WhereStep } from '../components/WhereStep';
import { STEP_CHECK, STEP_COPY, STEP_RUN, STEP_WHERE, useRestoreStore } from '../store';

const STEPS = [
  { id: 'where', label: <Localized id="restore-step-where" /> },
  { id: 'copy', label: <Localized id="restore-step-copy" /> },
  { id: 'check', label: <Localized id="restore-step-check" /> },
  { id: 'restore', label: <Localized id="restore-step-restore" /> },
];

export function RestorePage() {
  const { l10n } = useLocalization();
  const step = useRestoreStore((state) => state.step);
  const busy = useRestoreStore((state) => state.busy);
  const goTo = useRestoreStore((state) => state.goTo);
  const setChoice = useRestoreStore((state) => state.setChoice);

  useEscape(
    'leave',
    () => {
      if (step === STEP_WHERE) setChoice('start');
      else goTo(step - 1);
    },
    step !== STEP_RUN && !busy,
  );

  return (
    <div className="flex min-h-screen flex-col gap-5 bg-bg p-6 text-ink">
      <header className="flex flex-wrap items-center justify-between gap-4 print:hidden">
        <Localized id="restore-title">
          <h1 className="type-title" />
        </Localized>
        <LanguageSwitch />
      </header>
      <div className="print:hidden">
        <Stepper steps={STEPS} current={step} label={l10n.getString('restore-steps')} />
      </div>
      <div className="mx-auto w-full max-w-form">
        {step === STEP_WHERE ? (
          <WhereStep />
        ) : step === STEP_COPY ? (
          <CopyStep />
        ) : step === STEP_CHECK ? (
          <CheckStep />
        ) : (
          <RunStep />
        )}
      </div>
    </div>
  );
}
