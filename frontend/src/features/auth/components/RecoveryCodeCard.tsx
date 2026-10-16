// Shows a recovery code to print and keep (PRD NFR-SEC-12, board WizardUr step 3). The code is
// shown once, in large Latin letters left to right. F9 prints. Finish stays off until the user
// ticks that he printed it and put it in a safe place. Used by the first-run wizard and by the
// recovery path (a used code is spent, so a new one is shown, plan 04 decision).

import { Localized } from '@fluent/react';
import { useState } from 'react';
import { useShortcut, SHORTCUTS } from '../../../shared/keyboard';
import { Button, Checkbox, Code } from '../../../shared/ui';
import { groupRecoveryCode } from '../format';

interface RecoveryCodeCardProps {
  code: string;
  /** What this code is for, the title and the help line, as Fluent message IDs. */
  titleId: string;
  helpId: string;
  finishId: string;
  onFinish: () => void;
}

export function RecoveryCodeCard({
  code,
  titleId,
  helpId,
  finishId,
  onFinish,
}: RecoveryCodeCardProps) {
  const [printed, setPrinted] = useState(false);
  const print = () => {
    window.print();
  };
  useShortcut(SHORTCUTS.print, print);
  return (
    <div className="flex flex-col gap-4">
      <Localized id={titleId}>
        <h2 className="type-section print:text-ink" />
      </Localized>
      <Localized id={helpId}>
        <p className="type-body text-ink-muted" />
      </Localized>
      <p
        aria-label={code}
        className="type-title rounded-2 border border-border bg-surface-sunken px-4 py-3 text-center tracking-code"
      >
        <Code>{groupRecoveryCode(code)}</Code>
      </p>
      <div className="print:hidden">
        <Checkbox
          checked={printed}
          onChange={setPrinted}
          label={<Localized id="auth-wizard-printed" />}
        />
      </div>
      <div className="flex flex-wrap justify-between gap-2 print:hidden">
        <Button shortcut={SHORTCUTS.print} onClick={print}>
          <Localized id="auth-wizard-print" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.confirm}
          disabled={!printed}
          onClick={onFinish}
        >
          <Localized id={finishId} />
        </Button>
      </div>
    </div>
  );
}
