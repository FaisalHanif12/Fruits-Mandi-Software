// Overrides and checks (PRD Section 21): whether a line may change its commission and لاگا by hand
// (the changed amount is saved on the line as typed by hand, FR-11.10), and what happens when a
// sale takes a buyer over his credit limit: only a warning, or the sale is stopped (FR-12.4).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Checkbox,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  Select,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useOverrideSettings, useSetOverrideSettings } from '../hooks';
import type { CreditLimitMode, OverridesSettings } from '../types';

const MODES: readonly CreditLimitMode[] = ['warn', 'block'];

function OverridesForm({ saved }: { saved: OverridesSettings }) {
  const { l10n } = useLocalization();
  const [values, setValues] = useState<OverridesSettings>(saved);
  const save = useSetOverrideSettings();
  const error = save.error === null ? null : uiErrorOf(save.error);
  const dirty =
    values.allow_line_override !== saved.allow_line_override ||
    values.credit_limit_mode !== saved.credit_limit_mode;

  const submitOnce = (): Promise<boolean> =>
    save
      .mutateAsync(values)
      .then(() => {
        showToast('success', l10n.getString('settings-overrides-saved'));
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
      className="flex max-w-form flex-col gap-4"
    >
      <LeaveGuard dirty={dirty} onSave={submit} />
      {error === null ? null : (
        <ErrorNotice error={error} fieldsShown={['allow_line_override', 'credit_limit_mode']} />
      )}
      <div className="flex flex-col gap-1">
        <Checkbox
          checked={values.allow_line_override}
          onChange={(checked) => {
            setValues((current) => ({ ...current, allow_line_override: checked }));
          }}
          label={<Localized id="settings-overrides-allow" />}
        />
        <Localized id="settings-overrides-allow-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <Field
        label={<Localized id="settings-overrides-credit" />}
        error={fieldError(error, 'credit_limit_mode')}
      >
        {(control) => (
          <Select
            {...control}
            value={values.credit_limit_mode}
            onChange={(mode) => {
              setValues((current) => ({ ...current, credit_limit_mode: mode }));
            }}
            options={MODES.map((mode) => ({
              value: mode,
              label: <Localized id={`settings-overrides-credit-${mode}`} />,
            }))}
          />
        )}
      </Field>
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

export function OverridesTab() {
  const overrides = useOverrideSettings();
  if (overrides.isPending) return <LoadingState />;
  if (overrides.isError) {
    return (
      <ErrorState
        title={<Localized id="settings-load-charges-failed" />}
        error={uiErrorOf(overrides.error)}
        onRetry={() => {
          void overrides.refetch();
        }}
      />
    );
  }
  return <OverridesForm saved={overrides.data} />;
}
