// The fields of the سرمایہ screen (PRD 17.4, FR-7.67): date, the partner, the amount and the
// narration. Both types (invested, taken back) have the same fields in the same places. Everything
// typed is text; the backend reads it (CLAUDE.md rule 1). A problem the backend found shows under
// the field it points at.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  DateInput,
  ErrorState,
  Field,
  FreeTextInput,
  MoneyInput,
  Select,
  fieldError,
  fieldMark,
  FormCard,
} from '../../../shared/ui';
import type { EntryMarks } from '../../../shared/edit';
import type { CapitalDraft } from '../capital-draft';
import { usePartnerList } from '../hooks';

interface CapitalFormProps {
  draft: CapitalDraft;
  onChange: (patch: Partial<CapitalDraft>) => void;
  error: UiError | null;
  /** The marks of the edits this entry went through ("Before ...", plan 44); none when new. */
  marks?: EntryMarks | undefined;
}

export function CapitalForm({ draft, onChange, error, marks }: CapitalFormProps) {
  const language = useLanguage();
  const partners = usePartnerList();
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
      {partners.isError ? (
        <ErrorState
          title={<Localized id="cashbook-partners-failed" />}
          error={uiErrorOf(partners.error)}
          onRetry={() => {
            void partners.refetch();
          }}
        />
      ) : (
        <Field
          label={<Localized id="cashbook-field-partner" />}
          mark={fieldMark(marks, 'partner_id')}
          error={fieldError(error, 'partner_id')}
        >
          {(props) => (
            <Select
              {...props}
              readOnly={props.readOnly === true || partners.isPending}
              value={draft.partnerId === '' ? undefined : draft.partnerId}
              onChange={(partnerId) => {
                onChange({ partnerId });
              }}
              options={(partners.data ?? [])
                .filter((partner) => partner.is_active)
                .map((partner) => ({
                  value: String(partner.id),
                  label: displayName(partner, language),
                }))}
            />
          )}
        </Field>
      )}
      <Field
        label={<Localized id="cashbook-field-amount" />}
        mark={fieldMark(marks, 'amount_paisa')}
        error={fieldError(error, 'amount')}
      >
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
      <Field
        label={<Localized id="cashbook-field-narration" />}
        mark={fieldMark(marks, 'narration')}
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
    </FormCard>
  );
}
