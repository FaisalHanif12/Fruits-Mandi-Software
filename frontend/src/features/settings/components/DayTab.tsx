// The Day group of Settings (PRD Section 21, FR-21.1): the closing time and the automatic close, and
// under them the daily books check (NFR-OBS-5). The closing time is the same value as on the backup
// screen (the daily cloud and USB copy runs at it). The automatic close is only stored for now: the
// app does not close the day by itself yet, and the screen says so. The time is sent as typed; the
// backend checks it (ERR_OUT_OF_RANGE with 00:00 and 23:59) and is the final word.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { DaySettings } from '../../../bindings/DaySettings';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { useFormState, useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Checkbox,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  Notice,
  TextInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useDaySettings, useSetDaySettings } from '../maintenance-hooks';
import { BooksCheck } from './BooksCheck';

function DayForm({ saved }: { saved: DaySettings }) {
  const { l10n } = useLocalization();
  const { values, setField } = useFormState({ closing_time: saved.closing_time });
  const [autoClose, setAutoClose] = useState(saved.auto_day_close);
  const save = useSetDaySettings();
  const error = save.error === null ? null : uiErrorOf(save.error);
  const dirty = values.closing_time !== saved.closing_time || autoClose !== saved.auto_day_close;

  const submitOnce = (): Promise<boolean> =>
    save
      .mutateAsync({ closing_time: values.closing_time.trim(), auto_day_close: autoClose })
      .then(() => {
        showToast('success', l10n.getString('settings-saved'));
        return true;
      })
      .catch(() => false);
  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = (): Promise<boolean> => flight.run(submitOnce, false);
  const submitNow = () => {
    void submit();
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);

  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex max-w-dialog flex-col gap-4"
    >
      <LeaveGuard dirty={dirty} onSave={submit} />
      <Localized id="settings-day-title">
        <h2 className="type-section" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={['closing_time']} />}
      <Field
        label={<Localized id="settings-day-closing-time" />}
        hint={<Localized id="settings-day-closing-time-hint" />}
        error={fieldError(error, 'closing_time')}
      >
        {(control) => (
          <TextInput
            {...control}
            lang="en"
            value={values.closing_time}
            onChange={setField('closing_time')}
          />
        )}
      </Field>
      <Checkbox
        checked={autoClose}
        onChange={setAutoClose}
        label={<Localized id="settings-day-auto-close" />}
      />
      <Notice tone="info">
        <Localized id="settings-day-auto-close-note" />
      </Notice>
      <div>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || save.isPending}
          disabled={!dirty}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
    </form>
  );
}

export function DayTab() {
  const day = useDaySettings();
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="settings-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  return (
    <div className="flex max-w-reading flex-col gap-6">
      {/* A new saved value starts the form again, so what the backend kept replaces what was typed. */}
      <DayForm
        key={`${day.data.closing_time}|${String(day.data.auto_day_close)}`}
        saved={day.data}
      />
      <BooksCheck />
    </div>
  );
}
