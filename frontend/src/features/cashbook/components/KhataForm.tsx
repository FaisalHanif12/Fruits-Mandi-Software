// The fields of the الگ کھاتہ screen (PRD 16.6, FR-7.67): date, the khata, the amount, the reason
// and the narration. Both types (paid out, received) have the same fields in the same places. `F4`
// in the khata field adds a new khata. Everything typed is text; the backend reads it (CLAUDE.md
// rule 1).

import { Localized } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { pickedOf } from '../../../shared/accounts';
import {
  ControlAccountPicker,
  DateInput,
  Field,
  FreeTextInput,
  MoneyInput,
  fieldError,
} from '../../../shared/ui';
import type { KhataDraft } from '../khata-draft';
import { AddAccountDialog } from './AddAccountDialog';

/** The control of the separate khatas (PRD FR-10.4, 16.6). */
const KHATA_CONTROL = '0305';

interface KhataFormProps {
  draft: KhataDraft;
  onChange: (patch: Partial<KhataDraft>) => void;
  error: UiError | null;
}

export function KhataForm({ draft, onChange, error }: KhataFormProps) {
  const [adding, setAdding] = useState<string | null>(null);
  return (
    <div className="grid grid-cols-1 gap-4 rounded-3 border border-border bg-surface p-3 normal:grid-cols-2">
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
        label={<Localized id="cashbook-field-khata" />}
        error={fieldError(error, 'khata_account_id')}
      >
        {(props) => (
          <ControlAccountPicker
            {...props}
            controlCode={KHATA_CONTROL}
            value={draft.khata}
            showCard={false}
            onChange={(account) => {
              onChange({
                khata: account === null ? null : pickedOf(account),
              });
            }}
            onAddNew={setAdding}
            addLabelId="search-add-khata"
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
      <Field label={<Localized id="cashbook-field-reason" />} error={fieldError(error, 'reason')}>
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
      <div className="normal:col-span-2">
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
      </div>
      <AddAccountDialog
        kind="khata"
        open={adding !== null}
        onOpenChange={(open) => {
          if (!open) setAdding(null);
        }}
        typed={adding ?? ''}
        onCreated={(account) => {
          onChange({ khata: account });
        }}
      />
    </div>
  );
}
