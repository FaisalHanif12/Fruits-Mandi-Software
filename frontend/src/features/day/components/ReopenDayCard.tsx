// Reopen a day (PRD FR-21.2): the date, the reason and the Owner's sign (FR-21.4, plan 80 Q-R2-2), both required. Reopening a date reopens
// every later date too, and the reason goes to the audit log. A confirm box says exactly that
// (FR-7.8). Nothing is reopened until the backend has checked the date and the reason.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { useSingleFlight } from '../../../shared/hooks';
import {
  Button,
  Card,
  ConfirmDialog,
  DateInput,
  EmptyState,
  ErrorNotice,
  Field,
  fieldError,
  FreeTextInput,
  showToast,
  TextInput,
} from '../../../shared/ui';
import { Lock, LockOpen } from 'lucide-react';
import { useReopenDay } from '../hooks';

const FIELD_PATHS = ['date', 'reason', 'sign'];

interface ReopenDayCardProps {
  /** The last closed day, `YYYY-MM-DD`, or null when no day is closed. */
  closedUpTo: string | null;
}

export function ReopenDayCard({ closedUpTo }: ReopenDayCardProps) {
  const { l10n } = useLocalization();
  // The date starts as the last closed day, unless the user has typed one.
  const [typedDate, setTypedDate] = useState<string | null>(null);
  const date = typedDate ?? (closedUpTo === null ? '' : isoToDisplayDate(closedUpTo));
  const [reason, setReason] = useState('');
  // The Owner's sign is raw text, compared as typed by the backend: never converted from Roman letters.
  const [sign, setSign] = useState('');
  const [confirming, setConfirming] = useState(false);
  const reopen = useReopenDay();
  // One reopen at a time: a second Enter on the confirm box does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const error = reopen.error === null ? null : uiErrorOf(reopen.error);

  if (closedUpTo === null) {
    return (
      <Card icon={LockOpen} title={<Localized id="day-reopen-title" />}>
        <EmptyState
          icon={Lock}
          title={<Localized id="day-reopen-none-title" />}
          help={<Localized id="day-reopen-none-help" />}
        />
      </Card>
    );
  }

  const confirm = () => {
    setConfirming(false);
    void flight.run(
      () =>
        reopen
          .mutateAsync({ date, reason, sign })
          .then(() => {
            showToast('success', l10n.getString('day-reopened', { date }));
            setReason('');
            setSign('');
          })
          // The refusal is shown on the card by its fields.
          .catch(() => undefined),
      undefined,
    );
  };

  return (
    <Card title={<Localized id="day-reopen-title" />}>
      <Localized id="day-reopen-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        <Field label={<Localized id="day-reopen-date" />} error={fieldError(error, 'date')}>
          {(props) => <DateInput {...props} value={date} onChange={setTypedDate} />}
        </Field>
        <Field label={<Localized id="day-reopen-reason" />} error={fieldError(error, 'reason')}>
          {(props) => <FreeTextInput {...props} value={reason} onChange={setReason} />}
        </Field>
        <Field
          label={<Localized id="day-reopen-sign" />}
          hint={<Localized id="day-reopen-sign-hint" />}
          error={fieldError(error, 'sign')}
        >
          {(props) => <TextInput {...props} value={sign} onChange={setSign} />}
        </Field>
      </div>
      <div>
        <Button
          variant="danger"
          busy={flight.busy || reopen.isPending}
          onClick={() => {
            setConfirming(true);
          }}
        >
          <Localized id="day-reopen-button" />
        </Button>
      </div>
      <ConfirmDialog
        open={confirming}
        onOpenChange={setConfirming}
        title={<Localized id="day-reopen-confirm-title" />}
        description={<Localized id="day-reopen-confirm-description" vars={{ date }} />}
        confirmLabel={<Localized id="day-reopen-button" />}
        tone="danger"
        onConfirm={confirm}
      />
    </Card>
  );
}
