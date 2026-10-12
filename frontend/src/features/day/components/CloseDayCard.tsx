// Close the day (دن بند کریں; PRD FR-21.1): the date to close up to, a preview of what exists now
// (the days, the entries in them, whether the books balance, the cash in hand), and a confirm box
// that says exactly what will happen (FR-7.8). The numbers are the backend's. A day is closed in
// order, so the backend refuses a date that is closed already, a date after today and a trial
// balance that does not balance; each problem shows next to the date or as a notice.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import {
  Balance,
  Button,
  Card,
  ConfirmDialog,
  DateInput,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  Num,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useCloseDays, useDayClosePreview } from '../hooks';

const FIELD_PATHS = ['up_to'];

interface CloseDayCardProps {
  /** False when every day up to today is closed already: there is nothing to ask or close. */
  todayOpen: boolean;
}

export function CloseDayCard({ todayOpen }: CloseDayCardProps) {
  const { l10n } = useLocalization();
  // The date starts as the oldest open day, which the backend names; the user may pick a later one
  // to close several days in order (PRD FR-21.1). A field the user emptied closes nothing.
  const [typedUpTo, setTypedUpTo] = useState<string | null>(null);
  const [confirming, setConfirming] = useState(false);
  const preview = useDayClosePreview(typedUpTo ?? '', todayOpen);
  const close = useCloseDays();
  const closeError = close.error === null ? null : uiErrorOf(close.error);
  const previewError = preview.isError ? uiErrorOf(preview.error) : null;
  // A problem with the date (closed already, after today) shows under the date. Anything else the
  // preview could not do (a busy database) is a state with a retry, so it is not told twice.
  const previewIsAboutDate = previewError?.errors.some((item) => item.field === 'up_to') ?? false;
  const error = closeError ?? (previewIsAboutDate ? previewError : null);
  const data = preview.data;
  const upTo = typedUpTo ?? (data === undefined ? '' : isoToDisplayDate(data.up_to));
  const nothingToClose = !todayOpen || data?.days === 0;

  const confirm = () => {
    setConfirming(false);
    close.mutate(upTo, {
      onSuccess: () => {
        showToast('success', l10n.getString('day-closed', { date: upTo }));
        setTypedUpTo(null);
      },
    });
  };

  if (!todayOpen) {
    return (
      <Card title={<Localized id="day-close-title" />}>
        <Localized id="day-close-nothing">
          <p className="type-body text-ink-muted" />
        </Localized>
      </Card>
    );
  }

  return (
    <Card title={<Localized id="day-close-title" />}>
      <Localized id="day-close-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      <div className="max-w-field">
        <Field label={<Localized id="day-close-up-to" />} error={fieldError(error, 'up_to')}>
          {(props) => <DateInput {...props} value={upTo} onChange={setTypedUpTo} />}
        </Field>
      </div>
      {preview.isPending && preview.fetchStatus !== 'idle' ? <LoadingState /> : null}
      {preview.isError && !previewIsAboutDate ? (
        <ErrorState
          title={<Localized id="day-close-preview-failed" />}
          error={uiErrorOf(preview.error)}
          onRetry={() => {
            void preview.refetch();
          }}
        />
      ) : null}
      {data === undefined || preview.isError ? null : (
        <dl className="flex flex-col gap-2 type-body">
          <div className="flex flex-wrap items-baseline gap-x-3">
            <Localized id="day-close-from">
              <dt className="text-ink-muted" />
            </Localized>
            <dd>
              <Num>{isoToDisplayDate(data.from_date)}</Num>
            </dd>
          </div>
          <div className="flex flex-wrap items-baseline gap-x-3">
            <Localized id="day-close-days">
              <dt className="text-ink-muted" />
            </Localized>
            <dd>
              <Num>{String(data.days)}</Num>
            </dd>
          </div>
          <div className="flex flex-wrap items-baseline gap-x-3">
            <Localized id="day-close-entries">
              <dt className="text-ink-muted" />
            </Localized>
            <dd>
              <Num>{String(data.entries)}</Num>
            </dd>
          </div>
          <div className="flex flex-wrap items-baseline gap-x-3">
            <Localized id="day-close-trial-dr">
              <dt className="text-ink-muted" />
            </Localized>
            <dd>
              <Num paisa={data.trial_dr_paisa} />
            </dd>
          </div>
          <div className="flex flex-wrap items-baseline gap-x-3">
            <Localized id="day-close-trial-cr">
              <dt className="text-ink-muted" />
            </Localized>
            <dd>
              <Num paisa={data.trial_cr_paisa} />
            </dd>
          </div>
          <div className="flex flex-wrap items-baseline gap-x-3">
            <Localized id="day-close-trial">
              <dt className="text-ink-muted" />
            </Localized>
            <dd>
              <Balance balance={data.trial_difference} />
            </dd>
          </div>
          <div className="flex flex-wrap items-baseline gap-x-3">
            <Localized id="day-close-cash">
              <dt className="text-ink-muted" />
            </Localized>
            <dd>
              <Balance balance={data.cash_in_hand} />
            </dd>
          </div>
        </dl>
      )}
      {nothingToClose ? (
        <Localized id="day-close-nothing">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
      <div>
        <Button
          variant="primary"
          busy={close.isPending}
          disabled={data === undefined || preview.isError || upTo === '' || nothingToClose}
          onClick={() => {
            setConfirming(true);
          }}
        >
          <Localized id="day-close-button" />
        </Button>
      </div>
      <ConfirmDialog
        open={confirming}
        onOpenChange={setConfirming}
        title={<Localized id="day-close-confirm-title" />}
        description={
          <Localized
            id="day-close-confirm-description"
            vars={{ date: upTo, days: String(data?.days ?? 0) }}
          />
        }
        confirmLabel={<Localized id="day-close-button" />}
        onConfirm={confirm}
      />
    </Card>
  );
}
