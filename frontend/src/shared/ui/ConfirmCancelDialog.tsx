// The one box of every cancel (PRD FR-7.93, FR-12.22, FR-15.3, plan 100 C): a cancel reverses every
// figure an entry made, so in every section (بکری, vouchers, خرچہ, ادھار رقم, الگ کھاتہ, سرمایہ, جنرل,
// پھل, trips, gardens, write-off undo) منسوخ opens this box and it asks, all three required, on any day:
// the reason, the Owner's sign (FR-21.4) and the 4-digit partner code (FR-17.11). The undo straight
// after a save (FR-7.47) does not use it. The backend checks the three together with the cancel itself;
// a wrong one changes nothing and its words show under its field. When no partner code has been made yet
// the box offers the way to make one, in Settings, Security. The screen that opens the box sends the three
// as typed and keeps none of them (the sign and the code are secrets).

import { Localized } from '@fluent/react';
import { useRouter } from '@tanstack/react-router';
import { KeyRound } from 'lucide-react';
import type { ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { Button } from './Button';
import { Notice } from './Notice';
import { ReasonDialog } from './ReasonDialog';

/** What a cancel sends with it: the words are typed by the user and checked by the backend. */
export interface CancelSecrets {
  reason: string;
  sign: string;
  partner_code: string;
}

/** The code the backend sends when the partner code has not been made yet. */
const NOT_SET_CODE = 'ERR_PARTNER_CODE_NOT_SET';

interface ConfirmCancelDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  /** What will happen, in plain words. */
  description: ReactNode;
  /** The label of the reason field; "Reason for cancelling" when left out. */
  reasonLabel?: ReactNode;
  confirmLabel: ReactNode;
  /** What the backend refused, to show inside the box. */
  error: UiError | null;
  /** The action is running. */
  busy: boolean;
  /** Something the user must read before the action, for example the cash after it. */
  note?: ReactNode;
  /** Fields the action needs besides the three, shown above them (for example which write-off). */
  extra?: ReactNode;
  /** The field paths `extra` shows its own problems for, so they are not told twice. */
  extraFields?: readonly string[];
  /** The look of the action button. Default: danger. */
  confirmVariant?: 'danger' | 'primary';
  /** The action cannot be taken now: the button stays, dimmed. */
  confirmDisabled?: boolean;
  /** What the fields start with, for a box opened again after a step between (the cash question). */
  initial?: Partial<CancelSecrets> | undefined;
  onConfirm: (secrets: CancelSecrets) => void;
}

/**
 * The way to make the partner code, when the backend says there is none: Settings, Security. A box
 * around a test or a gallery page has no router; the button then does nothing and nothing fails.
 */
function MakeCodeNote({ onLeave }: { onLeave: () => void }) {
  const router = useRouter({ warn: false }) as ReturnType<typeof useRouter> | undefined;
  return (
    <Notice
      tone="warn"
      action={
        <Button
          onClick={() => {
            onLeave();
            void router?.navigate({ to: '/settings', search: { tab: 'lock' } });
          }}
        >
          <KeyRound aria-hidden="true" className="size-icon-md" />
          <Localized id="cancel-make-code" />
        </Button>
      }
    >
      <Localized id="cancel-no-code" />
    </Notice>
  );
}

export function ConfirmCancelDialog({
  open,
  onOpenChange,
  title,
  description,
  reasonLabel,
  confirmLabel,
  error,
  busy,
  note,
  extra,
  extraFields,
  confirmVariant = 'danger',
  confirmDisabled,
  initial,
  onConfirm,
}: ConfirmCancelDialogProps) {
  const noCode = error?.errors.some((item) => item.code === NOT_SET_CODE) === true;
  // "No code made yet" is told by its own note with the way to make one, not as a plain error too.
  const shownError =
    error === null || !noCode
      ? error
      : { ...error, errors: error.errors.filter((item) => item.code !== NOT_SET_CODE) };
  const notes =
    note === undefined && !noCode ? undefined : noCode ? (
      <>
        {note}
        <MakeCodeNote
          onLeave={() => {
            onOpenChange(false);
          }}
        />
      </>
    ) : (
      note
    );
  return (
    <ReasonDialog
      open={open}
      onOpenChange={onOpenChange}
      title={title}
      description={description}
      reasonLabel={reasonLabel ?? <Localized id="cancel-reason-label" />}
      signLabel={<Localized id="cancel-sign-label" />}
      signHint={<Localized id="cancel-sign-hint" />}
      codeLabel={<Localized id="cancel-code-label" />}
      codeHint={<Localized id="cancel-code-hint" />}
      confirmLabel={confirmLabel}
      confirmVariant={confirmVariant}
      error={shownError === null || shownError.errors.length === 0 ? null : shownError}
      busy={busy}
      note={notes}
      extra={extra}
      {...(extraFields === undefined ? {} : { extraFields })}
      {...(confirmDisabled === undefined ? {} : { confirmDisabled })}
      {...(initial?.reason === undefined ? {} : { initialReason: initial.reason })}
      {...(initial?.sign === undefined ? {} : { initialSign: initial.sign })}
      {...(initial?.partner_code === undefined ? {} : { initialCode: initial.partner_code })}
      onConfirm={(reason, sign, code) => {
        onConfirm({ reason, sign: sign ?? '', partner_code: code ?? '' });
      }}
    />
  );
}
