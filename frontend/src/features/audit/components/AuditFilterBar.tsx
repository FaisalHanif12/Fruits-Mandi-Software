// The filter boxes of the audit log (PRD FR-21.3): who, the action, the area, one record of an area,
// the two dates and words. They combine freely. The lists in the drop-downs are the ones the backend
// found in the log (audit_options), so only values that exist are offered. A refusal from the backend
// shows under its own box (PRD FR-7.6): a record number with no area is refused on `filters.entity`.
// The words box is a search box, so it never converts English letters to Urdu (Code Standards F41).

import { Localized, useLocalization } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  CodeInput,
  DateInput,
  Field,
  Notice,
  Select,
  TextInput,
  fieldError,
} from '../../../shared/ui';
import { ANY, type AuditFilterState } from '../filters';
import { actionMessageId, entityMessageId } from '../words';
import type { AuditOptions } from '../types';

interface AuditFilterBarProps {
  state: AuditFilterState;
  onChange: (change: Partial<AuditFilterState>) => void;
  /** What the backend lists for the drop-downs; undefined while it loads or when it failed. */
  options: AuditOptions | undefined;
  optionsFailed: boolean;
  /** The backend's refusal of the last run, so a wrong date shows under its box. */
  error: UiError | null;
}

export function AuditFilterBar({
  state,
  onChange,
  options,
  optionsFailed,
  error,
}: AuditFilterBarProps) {
  const { l10n } = useLocalization();
  const any = l10n.getString('audit-filter-any');
  const word = (id: string | null, code: string) => (id === null ? code : l10n.getString(id));

  // A value already chosen stays in its list even if the list has not come yet (a language switch).
  const users = options?.users ?? [];
  const actions = options?.actions ?? [];
  const entities = options?.entities ?? [];

  return (
    <div className="flex flex-col gap-3">
      {optionsFailed ? (
        <Notice tone="warn">
          <Localized id="audit-options-failed" />
        </Notice>
      ) : null}
      <div
        role="group"
        aria-label={l10n.getString('audit-filters-label')}
        {...formNavigationProps}
        className="grid grid-cols-4 items-start gap-4"
      >
        <Field
          label={<Localized id="audit-filter-user" />}
          error={fieldError(error, 'filters.user_id')}
        >
          {(control) => (
            <Select<string>
              {...control}
              value={state.user}
              onChange={(user) => {
                onChange({ user });
              }}
              options={[
                { value: ANY, label: any },
                ...users.map((user) => ({ value: String(user.id), label: user.display_name })),
              ]}
            />
          )}
        </Field>
        <Field
          label={<Localized id="audit-filter-action" />}
          error={fieldError(error, 'filters.action')}
        >
          {(control) => (
            <Select<string>
              {...control}
              value={state.action}
              onChange={(action) => {
                onChange({ action });
              }}
              options={[
                { value: ANY, label: any },
                ...actions.map((code) => ({
                  value: code,
                  label: word(actionMessageId(code), code),
                })),
              ]}
            />
          )}
        </Field>
        <Field
          label={<Localized id="audit-filter-entity" />}
          error={fieldError(error, 'filters.entity')}
        >
          {(control) => (
            <Select<string>
              {...control}
              value={state.entity}
              onChange={(entity) => {
                // A record number belongs to one area: another area lets the old number go.
                onChange({ entity, entityId: entity === ANY ? '' : state.entityId });
              }}
              options={[
                { value: ANY, label: any },
                ...entities.map((code) => ({
                  value: code,
                  label: word(entityMessageId(code), code),
                })),
              ]}
            />
          )}
        </Field>
        <Field
          label={<Localized id="audit-filter-record" />}
          hint={<Localized id="audit-filter-record-hint" />}
          error={fieldError(error, 'filters.entity_id')}
        >
          {(control) => (
            <CodeInput
              {...control}
              value={state.entityId}
              onChange={(entityId) => {
                onChange({ entityId });
              }}
            />
          )}
        </Field>
        <Field
          label={<Localized id="report-filter-from" />}
          error={fieldError(error, 'filters.from')}
        >
          {(control) => (
            <DateInput
              {...control}
              value={state.from}
              onChange={(from) => {
                onChange({ from });
              }}
            />
          )}
        </Field>
        <Field label={<Localized id="report-filter-to" />} error={fieldError(error, 'filters.to')}>
          {(control) => (
            <DateInput
              {...control}
              value={state.to}
              onChange={(to) => {
                onChange({ to });
              }}
            />
          )}
        </Field>
        <div className="col-span-2">
          <Field
            label={<Localized id="audit-filter-text" />}
            hint={<Localized id="audit-filter-text-hint" />}
            error={fieldError(error, 'filters.text')}
          >
            {(control) => (
              <TextInput
                {...control}
                value={state.text}
                onChange={(text) => {
                  onChange({ text });
                }}
              />
            )}
          </Field>
        </div>
      </div>
    </div>
  );
}
