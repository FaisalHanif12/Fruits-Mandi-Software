// The first-run wizard (PRD NFR-INS-2, boards WizardUr and WizardEn): four steps. 1 the firm
// (names in both scripts, phone, address), 2 the Owner (name, user name, password twice), 3 the
// recovery code to print, 4 the partners and their shares (plan 11). Nothing is saved until step 2
// is finished: setup_first_run then makes the firm and the Owner in one action and returns the
// code. If the backend refuses a field of an earlier step, the wizard goes back to that step and
// shows the message at the field.
//
// The partners commands need a signed-in Owner, and the setup does not sign him in. So right
// after the setup the wizard signs him in with the name and password he just typed (a call that
// does not refresh the screen, so the wizard stays). If that fails, the wizard ends at the
// sign-in screen as before and the partners are set up later (the Partners area). The partners
// step can always be left with "do this later" (a بکری then asks for the shares first, E9).
//
// The firm and mandi names have the spelling check (PRD FR-7.66): a word made only by the spelling
// rules is marked, and going on from step 1 asks the Owner to confirm or fix it. The words he
// confirmed go with the setup, and the backend checks them again.

import { Localized, useLocalization } from '@fluent/react';
import { useEffect, useState, type ReactNode } from 'react';
import { LanguageSwitch } from '../../../app/layout/LanguageSwitch';
import { uiErrorOf } from '../../../shared/api';
import { PartnerSetupForm } from '../../partners';
import { useFocusFirstField, useSingleFlight, useSpellingFlow } from '../../../shared/hooks';
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
  type NamePairValue,
} from '../../../shared/ui';
import { authLogin } from '../api';
import { RecoveryCodeCard } from '../components/RecoveryCodeCard';
import { PASSWORD_MIN_LENGTH } from '../format';
import { useFirstRun, useRefreshAfterSetup } from '../hooks';
import { firstHintedField, hintFor } from '../wizard-hints';
import {
  STEP_CODE,
  STEP_FIELDS,
  STEP_PARTNERS,
  WIZARD_FIELDS,
  earliestStepWithError,
  useWizardStore,
  type WizardField,
  type WizardNameKey,
} from '../wizard-store';

export function WizardPage() {
  const { l10n } = useLocalization();
  const step = useWizardStore((state) => state.step);
  const values = useWizardStore((state) => state.values);
  const marks = useWizardStore((state) => state.marks);
  const recoveryCode = useWizardStore((state) => state.recoveryCode);
  const signedIn = useWizardStore((state) => state.signedIn);
  const setStep = useWizardStore((state) => state.setStep);
  const setValue = useWizardStore((state) => state.setValue);
  const setMarks = useWizardStore((state) => state.setMarks);
  const finishSetup = useWizardStore((state) => state.finishSetup);
  const toPartners = useWizardStore((state) => state.toPartners);
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

  // "Check the spelling" and he chose to fix it: the names are on step 1, so go there.
  const spelling = useSpellingFlow({
    onFix: () => {
      if (step !== 0) {
        setStep(0);
        return;
      }
      firstField.current?.querySelector<HTMLElement>('[role="combobox"], input')?.focus();
    },
  });

  const namesOf = (name: WizardNameKey): NamePairValue =>
    name === 'firm'
      ? { urdu: values.firm_name_ur, english: values.firm_name_en, spelling: marks.firm }
      : { urdu: values.mandi_ur, english: values.mandi_en, spelling: marks.mandi };
  const rememberMarks = (name: WizardNameKey) => (next: NamePairValue) => {
    setMarks(name, next.spelling);
  };

  /**
   * Runs `run` once the firm name and then the mandi name have passed the spelling check, with the
   * words he confirmed. Undefined when he went back to fix a spelling.
   */
  const checked = <T,>(run: (confirmedWords: string[]) => Promise<T>) =>
    spelling.guard(namesOf('firm'), rememberMarks('firm'), (firmWords) =>
      spelling.guard(namesOf('mandi'), rememberMarks('mandi'), (mandiWords) =>
        run([...new Set([...firmWords, ...mandiWords])]),
      ),
    );

  // One try at a time: a second Enter while the setup is on its way would make a second Owner.
  const flight = useSingleFlight();
  const create = () => {
    if (setup.isPending) return;
    const work = () =>
      checked((confirmedWords) =>
        setup.mutateAsync({
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
          confirmed_rules_words: confirmedWords,
        }),
      )
        .then(async (done) => {
          if (done === undefined) return;
          // Sign in with what he just typed, before the passwords are cleared (see the top).
          const asOwner = await authLogin({
            username: values.owner_username,
            password: values.password,
          }).then(
            () => true,
            () => false,
          );
          finishSetup(done.recovery_code, asOwner);
        })
        .catch((failure: unknown) => {
          // A field of an earlier step: go back to the earliest one, so each message is at its field.
          const back = earliestStepWithError(uiErrorOf(failure).errors.map((item) => item.field));
          if (back !== null && back < step) setStep(back);
        });
    void flight.run(work, undefined);
  };
  const goOn = () => {
    // An empty or wrong-script step is not sent on. The backend checks again at the end.
    if (firstHintedField(step, values) !== null) {
      setTried((current) => current.map((was, index) => was || index === step));
      return;
    }
    if (step === 0) {
      // The names are checked here, where they are typed (PRD FR-7.66).
      void flight.run(
        () =>
          checked(() => Promise.resolve(true)).then((passed) => {
            if (passed === true) setStep(1);
          }),
        undefined,
      );
    } else create();
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
    { id: 'partners', label: <Localized id="auth-wizard-step-partners" /> },
  ];
  // The last step of the wizard: the app (already signed in) or the sign-in screen.
  const finishWizard = () => {
    reset();
    void refresh();
  };

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
    name: WizardNameKey,
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
        value={namesOf(name)}
        onChange={(next) => {
          setValue(urduField, next.urdu);
          setValue(englishField, next.english);
          rememberMarks(name)(next);
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
      <div className={`mx-auto w-full ${step === STEP_PARTNERS ? 'max-w-form' : 'max-w-reading'}`}>
        {step === STEP_PARTNERS ? (
          <div className="flex flex-col gap-4">
            <Localized id="auth-wizard-partners-title">
              <h2 className="type-section" />
            </Localized>
            <Localized id="auth-wizard-partners-help">
              <p className="type-body text-ink-muted" />
            </Localized>
            <PartnerSetupForm
              labelId="auth-wizard-partners-save"
              onSaved={finishWizard}
              actionsStart={
                <Button onClick={finishWizard}>
                  <Localized id="auth-wizard-partners-later" />
                </Button>
              }
            />
          </div>
        ) : (
          <Card>
            {step === STEP_CODE && recoveryCode !== null ? (
              <RecoveryCodeCard
                code={recoveryCode}
                titleId="auth-wizard-code-title"
                helpId="auth-wizard-code-help"
                finishId={signedIn ? 'auth-wizard-next' : 'auth-wizard-finish'}
                onFinish={() => {
                  // Signed in: the partners step comes next. Not signed in: the wizard is over.
                  if (signedIn) toPartners();
                  else finishWizard();
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
                      'firm',
                      'firm_name_ur',
                      'firm_name_en',
                      'auth-wizard-firm-name-ur',
                      'auth-wizard-firm-name-en',
                    )}
                    {pair(
                      'mandi',
                      'mandi_ur',
                      'mandi_en',
                      'auth-wizard-mandi-ur',
                      'auth-wizard-mandi-en',
                    )}
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
                    busy={flight.busy || setup.isPending}
                    onClick={goOn}
                  >
                    <Localized id={step === 0 ? 'auth-wizard-next' : 'auth-wizard-create'} />
                  </Button>
                </div>
              </div>
            )}
          </Card>
        )}
      </div>
      {spelling.dialog}
    </div>
  );
}

interface FieldOptions {
  lang?: string;
  hint?: ReactNode;
  password?: boolean;
}
