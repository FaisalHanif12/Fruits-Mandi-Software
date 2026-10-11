// The first-run wizard (PRD NFR-INS-2, boards WizardUr and WizardEn): three steps. 1 the firm
// (names in both scripts, phone, address), 2 the Owner (name, user name, password twice), 3 the
// recovery code to print. Nothing is saved until step 2 is finished: setup_first_run then makes
// the firm and the Owner in one action and returns the code. If the backend refuses a field of an
// earlier step, the wizard goes back to that step and shows the message at the field.

import { Localized, useLocalization } from '@fluent/react';
import { useEffect, useState, type ReactNode } from 'react';
import { LanguageSwitch } from '../../../app/layout/LanguageSwitch';
import { uiErrorOf } from '../../../shared/api';
import { useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  Card,
  ErrorNotice,
  Field,
  NamePair,
  PasswordInput,
  Stepper,
  TextInput,
  fieldError,
} from '../../../shared/ui';
import { RecoveryCodeCard } from '../components/RecoveryCodeCard';
import { useFirstRun, useRefreshAfterSetup } from '../hooks';
import { firstHintedField, hintFor } from '../wizard-hints';
import {
  STEP_FIELDS,
  WIZARD_FIELDS,
  earliestStepWithError,
  useWizardStore,
  type WizardField,
} from '../wizard-store';

/** The shortest password (NFR-SEC-1). The backend enforces it and is the final word. */
const PASSWORD_MIN_LENGTH = 6;

export function WizardPage() {
  const { l10n } = useLocalization();
  const step = useWizardStore((state) => state.step);
  const values = useWizardStore((state) => state.values);
  const recoveryCode = useWizardStore((state) => state.recoveryCode);
  const setStep = useWizardStore((state) => state.setStep);
  const setValue = useWizardStore((state) => state.setValue);
  const finishSetup = useWizardStore((state) => state.finishSetup);
  const reset = useWizardStore((state) => state.reset);
  const setup = useFirstRun();
  const refresh = useRefreshAfterSetup();
  const firstField = useFocusFirstField<HTMLDivElement>(step);
  const error = setup.error === null ? null : uiErrorOf(setup.error);
  // Per step: has the user tried to go on? Only then do empty required fields show a hint.
  const [tried, setTried] = useState<readonly boolean[]>([false, false]);

  // After a try that was held back, focus goes to the first field that needs a look.
  useEffect(() => {
    if (tried[step] === true) {
      firstField.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
    }
  }, [tried, step, firstField]);

  const create = () => {
    if (setup.isPending) return;
    setup.mutate(
      {
        firm_name_ur: values.firm_name_ur,
        firm_name_en: values.firm_name_en,
        mandi_ur: values.mandi_ur,
        mandi_en: values.mandi_en,
        address: values.address,
        phone: values.phone,
        owner_username: values.owner_username,
        owner_display_name: values.owner_display_name,
        password: values.password,
        password_repeat: values.password_repeat,
      },
      {
        onSuccess: (done) => {
          finishSetup(done.recovery_code);
        },
        onError: (failure) => {
          // A field of an earlier step: go back to the earliest one, so each message is at its field.
          const back = earliestStepWithError(uiErrorOf(failure).errors.map((item) => item.field));
          if (back !== null && back < step) setStep(back);
        },
      },
    );
  };
  const goOn = () => {
    // An empty or wrong-script step is not sent on. The backend checks again at the end.
    if (firstHintedField(step, values) !== null) {
      setTried((current) => current.map((was, index) => was || index === step));
      return;
    }
    if (step === 0) setStep(1);
    else create();
  };
  // Esc goes back one step (PRD FR-7.18).
  useEscape(
    'leave',
    () => {
      setStep(0);
    },
    step === 1,
  );
  const navigation = useFormNavigation(goOn);
  const steps = [
    { id: 'firm', label: <Localized id="auth-wizard-step-firm" /> },
    { id: 'owner', label: <Localized id="auth-wizard-step-owner" /> },
    { id: 'code', label: <Localized id="auth-wizard-step-code" /> },
  ];

  /** One text field of the wizard: its words, the hint or the backend's message, and its value. */
  const input = (field: WizardField, labelId: string, options: FieldOptions = {}) => {
    const stepOfField = STEP_FIELDS.findIndex((fields) => fields.includes(field));
    const hintId = hintFor(field, values[field], tried[stepOfField] === true);
    const shown =
      fieldError(error, field) ?? (hintId === null ? undefined : <Localized id={hintId} />);
    const onChange = (next: string) => {
      setValue(field, next);
    };
    return (
      <Field label={<Localized id={labelId} />} hint={options.hint} error={shown}>
        {(control) =>
          options.password === true ? (
            <PasswordInput
              {...control}
              value={values[field]}
              onChange={onChange}
              autoComplete="new-password"
            />
          ) : (
            <TextInput {...control} value={values[field]} onChange={onChange} lang={options.lang} />
          )
        }
      </Field>
    );
  };

  /** A name in both scripts: the one typed fills the other (PRD FR-7.24). */
  const pair = (
    urduField: WizardField,
    englishField: WizardField,
    urduLabelId: string,
    englishLabelId: string,
  ) => {
    // An empty required name shows its hint after a try. A wrong script is flagged by the pair.
    const required = (field: WizardField) =>
      hintFor(field, values[field], tried[0] === true) === 'err-required' ? (
        <Localized id="err-required" />
      ) : undefined;
    return (
      <NamePair
        value={{ urdu: values[urduField], english: values[englishField] }}
        onChange={(next) => {
          setValue(urduField, next.urdu);
          setValue(englishField, next.english);
        }}
        urduLabel={<Localized id={urduLabelId} />}
        englishLabel={<Localized id={englishLabelId} />}
        urduError={fieldError(error, urduField) ?? required(urduField)}
        englishError={fieldError(error, englishField) ?? required(englishField)}
      />
    );
  };

  return (
    <div className="flex min-h-screen flex-col gap-5 bg-bg p-6 text-ink">
      <header className="flex flex-wrap items-center justify-between gap-4 print:hidden">
        <Localized id="auth-wizard-title">
          <h1 className="type-title" />
        </Localized>
        <LanguageSwitch />
      </header>
      <div className="print:hidden">
        <Stepper steps={steps} current={step} label={l10n.getString('auth-wizard-steps')} />
      </div>
      <div className="mx-auto w-full max-w-reading">
        <Card>
          {step === 2 && recoveryCode !== null ? (
            <RecoveryCodeCard
              code={recoveryCode}
              titleId="auth-wizard-code-title"
              helpId="auth-wizard-code-help"
              finishId="auth-wizard-finish"
              onFinish={() => {
                reset();
                void refresh();
              }}
            />
          ) : (
            <div ref={firstField} {...navigation} className="flex flex-col gap-4">
              <Localized id={step === 0 ? 'auth-wizard-firm-title' : 'auth-wizard-owner-title'}>
                <h2 className="type-section" />
              </Localized>
              {error === null ? null : <ErrorNotice error={error} fieldsShown={WIZARD_FIELDS} />}
              {step === 0 ? (
                <>
                  {pair(
                    'firm_name_ur',
                    'firm_name_en',
                    'auth-wizard-firm-name-ur',
                    'auth-wizard-firm-name-en',
                  )}
                  {pair('mandi_ur', 'mandi_en', 'auth-wizard-mandi-ur', 'auth-wizard-mandi-en')}
                  {input('phone', 'auth-wizard-phone', { lang: 'en' })}
                  {input('address', 'auth-wizard-address', {
                    hint: <Localized id="auth-wizard-address-hint" />,
                  })}
                </>
              ) : (
                <>
                  {input('owner_display_name', 'auth-wizard-owner-name')}
                  {input('owner_username', 'auth-login-username', { lang: 'en' })}
                  {input('password', 'auth-wizard-password', {
                    password: true,
                    hint: (
                      <Localized
                        id="auth-wizard-password-hint"
                        vars={{ min: String(PASSWORD_MIN_LENGTH) }}
                      />
                    ),
                  })}
                  {input('password_repeat', 'auth-wizard-password-repeat', { password: true })}
                </>
              )}
              <div className="flex flex-wrap items-center justify-between gap-2">
                {step === 0 ? (
                  <Localized id="auth-wizard-later">
                    <span className="type-body text-ink-muted" />
                  </Localized>
                ) : (
                  <Button
                    shortcut={SHORTCUTS.back}
                    onClick={() => {
                      setStep(0);
                    }}
                  >
                    <Localized id="common-back" />
                  </Button>
                )}
                <Button
                  variant="primary"
                  shortcut={SHORTCUTS.confirm}
                  busy={setup.isPending}
                  onClick={goOn}
                >
                  <Localized id={step === 0 ? 'auth-wizard-next' : 'auth-wizard-create'} />
                </Button>
              </div>
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}

interface FieldOptions {
  lang?: string;
  hint?: ReactNode;
  password?: boolean;
}
