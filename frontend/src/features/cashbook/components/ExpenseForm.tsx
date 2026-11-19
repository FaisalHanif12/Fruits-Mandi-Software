// The fields of the خرچہ screen (PRD 16.2, FR-7.67): only the fields of the chosen type, always in
// the same places. Date first, then the head (دکان خرچہ) or the partner (ذاتی خرچہ and its واپسی),
// the amount, the fields of the type, and the narration last. The heads come with the most used
// first (FR-7.62). Everything typed is text; the backend reads it (CLAUDE.md rule 1). A problem the
// backend found shows under the field it points at.

import { Localized } from '@fluent/react';
import { pickedOf } from '../../../shared/accounts';
import { uiErrorOf } from '../../../shared/api';
import type { UiError } from '../../../bindings/UiError';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  DateInput,
  ErrorState,
  Field,
  FormCard,
  FormSpan,
  FreeTextInput,
  MoneyInput,
  Picker,
  Select,
  fieldError,
  fieldMark,
} from '../../../shared/ui';
import type { EntryMarks } from '../../../shared/edit';
import { usePartnerList } from '../../partners';
import type { ExpenseDraft } from '../expense-draft';

interface ExpenseFormProps {
  draft: ExpenseDraft;
  onChange: (patch: Partial<ExpenseDraft>) => void;
  /** What the backend refused, to show next to the field it points at. */
  error: UiError | null;
  /** The marks of the edits this entry went through ("Before ...", plan 44); none when new. */
  marks?: EntryMarks | undefined;
}

export function ExpenseForm({ draft, onChange, error, marks }: ExpenseFormProps) {
  const language = useLanguage();
  const partners = usePartnerList();
  const shop = draft.type === 'shop_expense';
  const personal = draft.type === 'personal_expense';

  // The head is chosen from the expense heads with the shared picker: a head that is missing is
  // added from its list (PRD FR-7.69), so the user never goes to the Index for it.
  const headField = shop ? (
    <Field
      label={<Localized id="cashbook-field-head" />}
      mark={fieldMark(marks, 'expense_head_id')}
      error={fieldError(error, 'expense_head_id')}
    >
      {(props) => (
        <Picker
          {...props}
          scope="expense_head"
          value={draft.head}
          onChange={(hit) => {
            onChange({ head: hit === null ? null : pickedOf(hit) });
          }}
        />
      )}
    </Field>
  ) : partners.isError ? (
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
  );

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
      {headField}
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
      {shop ? (
        <Field
          label={<Localized id="cashbook-field-paid-to" />}
          error={fieldError(error, 'paid_to')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.paidTo}
              onChange={(paidTo) => {
                onChange({ paidTo });
              }}
            />
          )}
        </Field>
      ) : null}
      {personal ? (
        <>
          <Field
            label={<Localized id="cashbook-field-reason" />}
            mark={fieldMark(marks, 'reason')}
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
            mark={fieldMark(marks, 'return_by_date')}
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
          <Field
            label={<Localized id="cashbook-field-taken-by" />}
            mark={fieldMark(marks, 'taken_by')}
            error={fieldError(error, 'taken_by')}
          >
            {(props) => (
              <FreeTextInput
                {...props}
                value={draft.takenBy}
                onChange={(takenBy) => {
                  onChange({ takenBy });
                }}
              />
            )}
          </Field>
        </>
      ) : null}
      <FormSpan>
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
      </FormSpan>
    </FormCard>
  );
}
