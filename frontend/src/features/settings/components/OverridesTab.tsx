// Overrides and checks (PRD Section 21): whether a line may change its commission and لاگا by hand
// (the changed amount is saved on the line as typed by hand, FR-11.10), what happens when a sale
// takes a buyer over his credit limit: only a warning, or the sale is stopped (FR-12.4), and the
// rate check: a warning when a rate is far from the item's average of the last 7 days, and how far
// (FR-7.55, plan 12). The percent is typed as text; the backend checks it again (1 to 1,000).

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
  PercentInput,
  Select,
  fieldError,
  showToast,
} from '../../../shared/ui';
import type { UiError } from '../../../bindings/UiError';
import { useOverrideSettings, useSetOverrideSettings } from '../hooks';
import type { CreditLimitMode, OverridesSettings } from '../types';

const MODES: readonly CreditLimitMode[] = ['warn', 'block'];
/** The fields this form draws, so a problem on any of them shows next to its field. */
const FIELDS = [
  'allow_line_override',
  'credit_limit_mode',
  'rate_check_enabled',
  'rate_check_percent',
];
/** The rate check percent is a whole number of at most four digits (1 to 1,000). */
const WHOLE_PERCENT = /^\d{1,4}$/;
const PERCENT_MIN = 1;
const PERCENT_MAX = 1000;

/** The problem shown at once for a percent that is not a whole number: the same words as the backend. */
function percentProblem(): UiError {
  return {
    ok: false,
    errors: [
      {
        code: 'ERR_OUT_OF_RANGE',
        field: 'rate_check_percent',
        values: { min: PERCENT_MIN, max: PERCENT_MAX },
      },
    ],
    ref: '',
  };
}

function OverridesForm({ saved }: { saved: OverridesSettings }) {
  const { l10n } = useLocalization();
  const [values, setValues] = useState<OverridesSettings>(saved);
  // The percent is text while it is typed; it becomes a number only when it is sent.
  const [percentText, setPercentText] = useState(String(saved.rate_check_percent));
  const [localProblem, setLocalProblem] = useState<UiError | null>(null);
  const save = useSetOverrideSettings();
  const error = localProblem ?? (save.error === null ? null : uiErrorOf(save.error));
  const dirty =
    values.allow_line_override !== saved.allow_line_override ||
    values.credit_limit_mode !== saved.credit_limit_mode ||
    values.rate_check_enabled !== saved.rate_check_enabled ||
    percentText !== String(saved.rate_check_percent);

  const submitOnce = (): Promise<boolean> => {
    if (!WHOLE_PERCENT.test(percentText.trim())) {
      setLocalProblem(percentProblem());
      return Promise.resolve(false);
    }
    setLocalProblem(null);
    return save
      .mutateAsync({ ...values, rate_check_percent: Number.parseInt(percentText.trim(), 10) })
      .then(() => {
        showToast('success', l10n.getString('settings-overrides-saved'));
        return true;
      })
      .catch(() => false);
  };
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
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}
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
      <div className="flex flex-col gap-1">
        <Checkbox
          checked={values.rate_check_enabled}
          onChange={(checked) => {
            setValues((current) => ({ ...current, rate_check_enabled: checked }));
          }}
          label={<Localized id="settings-overrides-rate-check" />}
        />
        <Localized id="settings-overrides-rate-check-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <Field
        label={<Localized id="settings-overrides-rate-check-percent" />}
        hint={<Localized id="settings-overrides-rate-check-percent-hint" />}
        error={fieldError(error, 'rate_check_percent')}
      >
        {(control) => (
          <PercentInput
            {...control}
            readOnly={!values.rate_check_enabled}
            value={percentText}
            onChange={(text) => {
              setLocalProblem(null);
              setPercentText(text);
            }}
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
