// The fields of the ادھار رقم screen (PRD 16.5, FR-16.4, FR-7.67): date, the person, the amount, then
// the fields of the type (money lent: the reason and the return-by date; money returned: in cash
// or deducted from his salary, with the salary head), and the narration last. `F4` in the person
// field adds a new person. There is no mode other than cash or salary, no bank and no cheque
// (PRD FR-10.4). Everything typed is text; the backend reads it (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { pickedOf } from '../../../shared/accounts';
import {
  ControlAccountPicker,
  DateInput,
  Field,
  FreeTextInput,
  MoneyInput,
  Notice,
  Picker,
  Select,
  fieldError,
  FormCard,
  FormSpan,
} from '../../../shared/ui';
import type { LentDraft } from '../lent-draft';
import type { LentReturnMode } from '../types';
import { AddAccountDialog } from './AddAccountDialog';

/** The control of the persons who can borrow (PRD FR-10.4). */
const LENT_CONTROL = '0103';
const MODES: readonly LentReturnMode[] = ['cash', 'salary'];

interface LentFormProps {
  draft: LentDraft;
  onChange: (patch: Partial<LentDraft>) => void;
  error: UiError | null;
}

export function LentForm({ draft, onChange, error }: LentFormProps) {
  const { l10n } = useLocalization();
  const [adding, setAdding] = useState<string | null>(null);
  const lent = draft.type === 'money_lent';
  const salary = draft.type === 'money_lent_return' && draft.mode === 'salary';
  return (
    <FormCard>
      <Field label={<Localized id="cashbook-field-date" />} error={fieldError(error, 'date')}>
        {(props) => (
          <DateInput
            {...props}
            value={draft.date}
            onChange={(date) => {
              onChange({ date });
            }}
          />
        )}
      </Field>
      <Field
        label={<Localized id="cashbook-field-person" />}
        error={fieldError(error, 'person_account_id')}
      >
        {(props) => (
          <ControlAccountPicker
            {...props}
            controlCode={LENT_CONTROL}
            // A person in the dead amount can still give money back, tagged "ڈوبی" (PRD FR-10.10).
            includeDead={!lent}
            value={draft.person}
            onChange={(account) => {
              onChange({
                person: account === null ? null : pickedOf(account),
              });
            }}
            onAddNew={setAdding}
          />
        )}
      </Field>
      <Field label={<Localized id="cashbook-field-amount" />} error={fieldError(error, 'amount')}>
        {(props) => (
          <MoneyInput
            {...props}
            value={draft.amount}
            onChange={(amount) => {
              onChange({ amount });
            }}
          />
        )}
      </Field>
      {lent ? (
        <>
          <Field
            label={<Localized id="cashbook-field-reason" />}
            error={fieldError(error, 'reason')}
          >
            {(props) => (
              <FreeTextInput
                {...props}
                value={draft.reason}
                onChange={(reason) => {
                  onChange({ reason });
                }}
              />
            )}
          </Field>
          <Field
            label={<Localized id="cashbook-field-return-by" />}
            error={fieldError(error, 'return_by')}
          >
            {(props) => (
              <DateInput
                {...props}
                value={draft.returnBy}
                onChange={(returnBy) => {
                  onChange({ returnBy });
                }}
              />
            )}
          </Field>
        </>
      ) : (
        <>
          <Field label={<Localized id="cashbook-field-mode" />} error={fieldError(error, 'mode')}>
            {(props) => (
              <Select
                {...props}
                value={draft.mode}
                onChange={(mode) => {
                  onChange({ mode });
                }}
                options={MODES.map((mode) => ({
                  value: mode,
                  label: <Localized id={`cashbook-mode-${mode}`} />,
                }))}
              />
            )}
          </Field>
          {salary ? (
            <Field
              label={<Localized id="cashbook-field-salary-head" />}
              hint={<Localized id="cashbook-field-salary-head-hint" />}
              error={fieldError(error, 'salary_head_id')}
            >
              {(props) => (
                <Picker
                  {...props}
                  scope="expense_head"
                  value={draft.salaryHead}
                  placeholder={l10n.getString('cashbook-salary-head-default')}
                  onChange={(hit) => {
                    onChange({ salaryHead: hit === null ? null : pickedOf(hit) });
                  }}
                />
              )}
            </Field>
          ) : null}
          {salary ? (
            <FormSpan>
              <Notice tone="info">
                <Localized id="cashbook-salary-note" />
              </Notice>
            </FormSpan>
          ) : null}
        </>
      )}
      <FormSpan>
        <Field
          label={<Localized id="cashbook-field-narration" />}
          error={fieldError(error, 'narration')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.narration}
              onChange={(narration) => {
                onChange({ narration });
              }}
            />
          )}
        </Field>
      </FormSpan>
      <AddAccountDialog
        kind="person"
        open={adding !== null}
        onOpenChange={(open) => {
          if (!open) setAdding(null);
        }}
        typed={adding ?? ''}
        onCreated={(account) => {
          onChange({ person: account });
        }}
      />
    </FormCard>
  );
}
