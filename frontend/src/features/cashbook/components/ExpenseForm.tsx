// The fields of the خرچہ screen (PRD 16.2, FR-7.67): only the fields of the chosen type, always in
// the same places. Date first, then the head (دکان خرچہ) or the partner (ذاتی خرچہ and its واپسی),
// the amount, the fields of the type, and the narration last. The heads come with the most used
// first (FR-7.62). Everything typed is text; the backend reads it (CLAUDE.md rule 1). A problem the
// backend found shows under the field it points at.

import { Localized } from '@fluent/react';
import { uiErrorOf } from '../../../shared/api';
import type { UiError } from '../../../bindings/UiError';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  DateInput,
  ErrorState,
  Field,
  FreeTextInput,
  MoneyInput,
  Select,
  fieldError,
} from '../../../shared/ui';
import { usePartnerList } from '../../partners';
import type { ExpenseDraft } from '../expense-draft';
import { useExpenseHeads } from '../hooks';

interface ExpenseFormProps {
  draft: ExpenseDraft;
  onChange: (patch: Partial<ExpenseDraft>) => void;
  /** What the backend refused, to show next to the field it points at. */
  error: UiError | null;
}

export function ExpenseForm({ draft, onChange, error }: ExpenseFormProps) {
  const language = useLanguage();
  const heads = useExpenseHeads();
  const partners = usePartnerList();
  const shop = draft.type === 'shop_expense';
  const personal = draft.type === 'personal_expense';

  const headField = shop ? (
    heads.isError ? (
      <ErrorState
        title={<Localized id="cashbook-heads-failed" />}
        error={uiErrorOf(heads.error)}
        onRetry={() => {
          void heads.refetch();
        }}
      />
    ) : (
      <Field
        label={<Localized id="cashbook-field-head" />}
        error={fieldError(error, 'expense_head_id')}
      >
        {(props) => (
          <Select
            {...props}
            readOnly={props.readOnly === true || heads.isPending}
            value={draft.headId === '' ? undefined : draft.headId}
            onChange={(headId) => {
              onChange({ headId });
            }}
            // The backend sends the most used heads first (PRD FR-7.62); the list keeps that order.
            options={(heads.data?.rows ?? []).map((choice) => ({
              value: String(choice.id),
              label: displayName(choice.head, language),
            }))}
          />
        )}
      </Field>
    )
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
      {headField}
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
          <Field
            label={<Localized id="cashbook-field-taken-by" />}
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
    </div>
  );
}
