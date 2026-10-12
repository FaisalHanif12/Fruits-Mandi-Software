// Reopen a day (PRD FR-21.2): the date and the reason, which is required. Reopening a date reopens
// every later date too, and the reason goes to the audit log. A confirm box says exactly that
// (FR-7.8). Nothing is reopened until the backend has checked the date and the reason.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
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
} from '../../../shared/ui';
import { Lock } from 'lucide-react';
import { useReopenDay } from '../hooks';

const FIELD_PATHS = ['date', 'reason'];

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
  const [confirming, setConfirming] = useState(false);
  const reopen = useReopenDay();
  const error = reopen.error === null ? null : uiErrorOf(reopen.error);

  if (closedUpTo === null) {
    return (
      <Card title={<Localized id="day-reopen-title" />}>
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
    reopen.mutate(
      { date, reason },
      {
        onSuccess: () => {
          showToast('success', l10n.getString('day-reopened', { date }));
          setReason('');
        },
      },
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
      </div>
      <div>
        <Button
          variant="danger"
          busy={reopen.isPending}
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
