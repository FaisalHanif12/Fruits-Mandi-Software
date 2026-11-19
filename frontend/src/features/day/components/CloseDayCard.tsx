// Close the day (دن بند کریں; PRD FR-21.1): the date to close up to, a preview of what exists now
// (the days, the entries in them, whether the books balance, the cash in hand), and a confirm box
// that says exactly what will happen (FR-7.8). The numbers are the backend's. A day is closed in
// order, so the backend refuses a date that is closed already, a date after today and a trial
// balance that does not balance; each problem shows next to the date or as a notice. The full
// summary of the days (PRD FR-21.1, 4.9) is DaySummary.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { useSingleFlight } from '../../../shared/hooks';
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
import { DaySummary } from './DaySummary';

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
  // One close at a time: a second Enter on the confirm box does nothing (NFR-REL-1).
  const flight = useSingleFlight();
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
    void flight.run(
      () =>
        close
          .mutateAsync(upTo)
          .then(() => {
            showToast('success', l10n.getString('day-closed', { date: upTo }));
            setTypedUpTo(null);
          })
          // The refusal is shown on the card by its fields.
          .catch(() => undefined),
      undefined,
    );
  };

  if (!todayOpen) {
    return (
      <Card compact>
        <Localized id="day-close-nothing">
          <p className="type-body text-ink-muted" />
        </Localized>
      </Card>
    );
  }

  return (
    <Card compact>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      {/* The date and the button that closes up to it are side by side, so the button is always in
          view with the date (plan 38). The help of the card is behind the ⓘ of the date. */}
      <div className="flex flex-wrap items-end gap-x-4 gap-y-2">
        <div className="w-date shrink-0">
          <Field
            label={<Localized id="day-close-up-to" />}
            info={
              <Localized id="day-close-help">
                <p />
              </Localized>
            }
            error={fieldError(error, 'up_to')}
          >
            {(props) => <DateInput {...props} value={upTo} onChange={setTypedUpTo} />}
          </Field>
        </div>
        <Button
          variant="primary"
          busy={flight.busy || close.isPending}
          disabled={data === undefined || preview.isError || upTo === '' || nothingToClose}
          onClick={() => {
            setConfirming(true);
          }}
        >
          <Localized id="day-close-button" />
        </Button>
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
        <div className="@container min-w-0">
          <dl className="grid grid-cols-1 gap-x-6 gap-y-2 type-body @form2:grid-cols-2">
            <div className="grid grid-cols-2 items-baseline gap-x-4">
              <Localized id="day-close-from">
                <dt className="text-ink-muted" />
              </Localized>
              <dd dir="ltr" className="text-end">
                <Num>{isoToDisplayDate(data.from_date)}</Num>
              </dd>
            </div>
            <div className="grid grid-cols-2 items-baseline gap-x-4">
              <Localized id="day-close-days">
                <dt className="text-ink-muted" />
              </Localized>
              <dd dir="ltr" className="text-end">
                <Num>{String(data.days)}</Num>
              </dd>
            </div>
            <div className="grid grid-cols-2 items-baseline gap-x-4">
              <Localized id="day-close-entries">
                <dt className="text-ink-muted" />
              </Localized>
              <dd dir="ltr" className="text-end">
                <Num>{String(data.entries)}</Num>
              </dd>
            </div>
            <div className="grid grid-cols-2 items-baseline gap-x-4">
              <Localized id="day-close-trial-dr">
                <dt className="text-ink-muted" />
              </Localized>
              <dd dir="ltr" className="text-end">
                <Num paisa={data.trial_dr_paisa} />
              </dd>
            </div>
            <div className="grid grid-cols-2 items-baseline gap-x-4">
              <Localized id="day-close-trial-cr">
                <dt className="text-ink-muted" />
              </Localized>
              <dd dir="ltr" className="text-end">
                <Num paisa={data.trial_cr_paisa} />
              </dd>
            </div>
            <div className="grid grid-cols-2 items-baseline gap-x-4">
              <Localized id="day-close-trial">
                <dt className="text-ink-muted" />
              </Localized>
              <dd dir="ltr" className="text-end">
                <Balance balance={data.trial_difference} />
              </dd>
            </div>
            <div className="grid grid-cols-2 items-baseline gap-x-4">
              <Localized id="day-close-cash">
                <dt className="text-ink-muted" />
              </Localized>
              <dd dir="ltr" className="text-end">
                <Balance balance={data.cash_in_hand} />
              </dd>
            </div>
          </dl>
        </div>
      )}
      {data === undefined || preview.isError || nothingToClose ? null : (
        <DaySummary summary={data.summary} />
      )}
      {nothingToClose ? (
        <Localized id="day-close-nothing">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
      <ConfirmDialog
        open={confirming}
        onOpenChange={setConfirming}
        title={<Localized id="day-close-confirm-title" />}
        description={
          <Localized
            id="day-close-confirm-description"
            vars={{ date: upTo, days: data?.days ?? 0 }}
          />
        }
        confirmLabel={<Localized id="day-close-button" />}
        onConfirm={confirm}
      />
    </Card>
  );
}
