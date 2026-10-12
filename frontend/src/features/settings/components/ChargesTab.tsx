// The global charge defaults (PRD Section 21 "Charges", 4.2): the seller commission, the buyer
// commission and the لاگا on fruit. Each is a rule that the backend holds, so a rule for one party
// or item (انڈکس, commission rules) still wins over it. مزدوری, منشیانہ and فون have no default.
// Each default has its method (only the methods PRD 11.2 allows for that charge) and its value as
// the text typed; the backend checks them (ERR_PERCENT_OVER_100, ERR_NOT_A_NUMBER ...) and is the
// final word. The version read goes back with the save, so a change made elsewhere is noticed.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  ErrorState,
  Field,
  LinkButton,
  LoadingState,
  Select,
  TextInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { methodsOf } from '../../rules';
import { useChargeDefaults, useSetChargeDefaults } from '../hooks';
import {
  DEFAULT_KEYS,
  type ChargesDraft,
  type ChargesInput,
  type ChargesSettings,
  type DefaultKey,
} from '../types';

/** Which side and charge each default is, to know the methods it may have (PRD 11.2). */
const DEFAULT_OF: Record<DefaultKey, { side: 'seller' | 'buyer'; charge: 'commission' | 'laga' }> =
  {
    seller_commission: { side: 'seller', charge: 'commission' },
    buyer_commission: { side: 'buyer', charge: 'commission' },
    laga_fruit: { side: 'buyer', charge: 'laga' },
  };

const LABEL_ID: Record<DefaultKey, string> = {
  seller_commission: 'settings-charges-seller',
  buyer_commission: 'settings-charges-buyer',
  laga_fruit: 'settings-charges-laga',
};

const FIELD_PATHS = DEFAULT_KEYS.flatMap((key) => [`${key}.value`, `${key}.method`]);

/** The form's starting point: each default as saved. */
function draftOf(saved: ChargesSettings): ChargesDraft {
  return {
    seller_commission: {
      method: saved.seller_commission.method,
      value: saved.seller_commission.value_text,
    },
    buyer_commission: {
      method: saved.buyer_commission.method,
      value: saved.buyer_commission.value_text,
    },
    laga_fruit: { method: saved.laga_fruit.method, value: saved.laga_fruit.value_text },
  };
}

/** What goes to the backend: the version read, the method and the value as typed. */
function inputOf(saved: ChargesSettings, draft: ChargesDraft): ChargesInput {
  return {
    seller_commission: { version: saved.seller_commission.version, ...draft.seller_commission },
    buyer_commission: { version: saved.buyer_commission.version, ...draft.buyer_commission },
    laga_fruit: { version: saved.laga_fruit.version, ...draft.laga_fruit },
  };
}

function ChargesForm({ saved }: { saved: ChargesSettings }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const [values, setValues] = useState<ChargesDraft>(() => draftOf(saved));
  const change = (key: DefaultKey, patch: Partial<ChargesDraft[DefaultKey]>) => {
    setValues((current) => ({ ...current, [key]: { ...current[key], ...patch } }));
  };
  const save = useSetChargeDefaults();
  const error = save.error === null ? null : uiErrorOf(save.error);
  const start = draftOf(saved);
  const dirty = DEFAULT_KEYS.some(
    (key) => values[key].method !== start[key].method || values[key].value !== start[key].value,
  );

  const submit = (): Promise<boolean> =>
    save
      .mutateAsync(inputOf(saved, values))
      .then(() => {
        showToast('success', l10n.getString('settings-charges-saved'));
        return true;
      })
      .catch(() => false);
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
      <Localized id="settings-charges-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      {DEFAULT_KEYS.map((key) => {
        const { side, charge } = DEFAULT_OF[key];
        const methods = methodsOf(side, charge);
        return (
          <section key={key} className="flex flex-col gap-3">
            <Localized id={LABEL_ID[key]}>
              <h3 className="type-section" />
            </Localized>
            <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
              <Field
                label={<Localized id="rules-field-method" />}
                error={fieldError(error, `${key}.method`)}
              >
                {(control) => (
                  <Select
                    {...control}
                    value={values[key].method}
                    onChange={(method) => {
                      change(key, { method });
                    }}
                    options={methods.map((method) => ({
                      value: method,
                      label: <Localized id={`rules-method-${method}`} />,
                    }))}
                  />
                )}
              </Field>
              <Field
                label={<Localized id="rules-field-value" />}
                hint={<Localized id={`rules-value-hint-${values[key].method}`} />}
                error={fieldError(error, `${key}.value`)}
              >
                {(control) => (
                  <TextInput
                    {...control}
                    value={values[key].value}
                    onChange={(value) => {
                      change(key, { value });
                    }}
                    lang="en"
                  />
                )}
              </Field>
            </div>
          </section>
        );
      })}
      <Localized id="settings-charges-note">
        <p className="type-body text-ink-muted" />
      </Localized>
      <div className="flex flex-wrap items-center gap-4">
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={save.isPending}
          disabled={!dirty}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
        <LinkButton
          onClick={() => {
            void navigate({ to: '/index/rules' });
          }}
        >
          <Localized id="settings-charges-open-rules" />
        </LinkButton>
      </div>
    </form>
  );
}

export function ChargesTab() {
  const charges = useChargeDefaults();
  if (charges.isPending) return <LoadingState />;
  if (charges.isError) {
    return (
      <ErrorState
        title={<Localized id="settings-load-charges-failed" />}
        error={uiErrorOf(charges.error)}
        onRetry={() => {
          void charges.refetch();
        }}
      />
    );
  }
  // The form starts again from the saved defaults whenever they change (a save, or a rule edit).
  const { seller_commission, buyer_commission, laga_fruit } = charges.data;
  const key = [seller_commission, buyer_commission, laga_fruit]
    .map((one) => `${String(one.version)}|${one.method}|${one.value_text}`)
    .join(';');
  return <ChargesForm key={key} saved={charges.data} />;
}
