// The filter boxes of a report (PRD FR-19.1, FR-19.2, FR-19.10): the period with its shortcuts, the
// two dates, the day of a balance list, and the party, seller, زمیندار, item, city and اڈا pickers.
// Each report shows only the boxes the catalogue says it takes. The boxes are plain fields of the
// shared kit: arrows move between them (shared/keyboard), a picker searches both scripts and never
// converts English letters (Code Standards F40, F41). The party card is not drawn here: these are
// filters, not an entry, and the table is the answer.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode, Ref } from 'react';
import type { ReportCatalogEntry } from '../../../bindings/ReportCatalogEntry';
import type { ReportFilterKind } from '../../../bindings/ReportFilterKind';
import type { UiError } from '../../../bindings/UiError';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  AccountPicker,
  Checkbox,
  DateInput,
  Field,
  Picker,
  Select,
  fieldError,
  type PickerValue,
} from '../../../shared/ui';
import {
  PERIOD_CHOICES,
  periodMessageId,
  type PeriodChoice,
  type ReportFilterState,
} from '../filters';

interface ReportFilterBarProps {
  entry: ReportCatalogEntry;
  state: ReportFilterState;
  onChange: (change: Partial<ReportFilterState>) => void;
  /** The backend's refusal of the last run, so a wrong date shows under its box. */
  error: UiError | null;
  containerRef?: Ref<HTMLDivElement>;
}

/** A record the picker chose, as the value the screen keeps (both names, so a language switch is free). */
function valueOf(
  hit: { id: number; name_ur: string; name_en: string; code?: string | undefined } | null,
): PickerValue | null {
  return hit === null
    ? null
    : { id: hit.id, name_ur: hit.name_ur, name_en: hit.name_en, code: hit.code };
}

export function ReportFilterBar({
  entry,
  state,
  onChange,
  error,
  containerRef,
}: ReportFilterBarProps) {
  const { l10n } = useLocalization();
  const takes = (kind: ReportFilterKind) => entry.filters.includes(kind);
  const custom = state.period === 'custom';

  const boxes: ReactNode[] = [];
  if (takes('date_range')) {
    boxes.push(
      <Field
        key="period"
        label={<Localized id="report-filter-period" />}
        error={fieldError(error, 'filters.shortcut')}
      >
        {(control) => (
          <Select<PeriodChoice>
            {...control}
            value={state.period}
            onChange={(period) => {
              onChange({ period });
            }}
            options={PERIOD_CHOICES.map((choice) => ({
              value: choice,
              label: l10n.getString(periodMessageId(choice)),
            }))}
          />
        )}
      </Field>,
    );
    if (custom) {
      boxes.push(
        <Field
          key="from"
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
        </Field>,
        <Field
          key="to"
          label={<Localized id="report-filter-to" />}
          error={fieldError(error, 'filters.to')}
        >
          {(control) => (
            <DateInput
              {...control}
              value={state.to}
              onChange={(to) => {
                onChange({ to });
              }}
            />
          )}
        </Field>,
      );
    }
  }
  if (takes('as_of')) {
    boxes.push(
      <Field
        key="as_of"
        label={<Localized id="report-filter-as-of" />}
        error={fieldError(error, 'filters.as_of')}
      >
        {(control) => (
          <DateInput
            {...control}
            value={state.asOf}
            onChange={(asOf) => {
              onChange({ asOf });
            }}
          />
        )}
      </Field>,
    );
  }
  if (takes('party')) {
    boxes.push(
      <Field
        key="party"
        label={<Localized id="report-filter-party" />}
        error={fieldError(error, 'filters.party') ?? fieldError(error, 'filters.party.account_id')}
      >
        {(control) => (
          <AccountPicker
            {...control}
            scope="any"
            showCard={false}
            value={state.party}
            onChange={(hit) => {
              onChange({ party: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  for (const kind of ['seller', 'zamindar'] as const) {
    if (!takes(kind)) continue;
    boxes.push(
      <Field
        key={kind}
        label={<Localized id={`report-filter-${kind}`} />}
        error={fieldError(error, `filters.${kind}_id`)}
      >
        {(control) => (
          <AccountPicker
            {...control}
            scope="seller"
            showCard={false}
            value={state[kind]}
            onChange={(hit) => {
              onChange({ [kind]: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  for (const kind of ['item', 'city', 'adda'] as const) {
    if (!takes(kind)) continue;
    boxes.push(
      <Field
        key={kind}
        label={<Localized id={`report-filter-${kind}`} />}
        error={fieldError(error, `filters.${kind}_id`)}
      >
        {(control) => (
          <Picker
            {...control}
            scope={kind}
            value={state[kind]}
            onChange={(hit) => {
              onChange({ [kind]: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  if (takes('detail')) {
    boxes.push(
      <div key="detail" className="flex items-end">
        <Checkbox
          checked={state.detail}
          onChange={(detail) => {
            onChange({ detail });
          }}
          label={<Localized id="report-filter-detail" />}
        />
      </div>,
    );
  }

  if (boxes.length === 0) return null;
  return (
    <div
      ref={containerRef}
      role="group"
      aria-label={l10n.getString('report-filters-label')}
      {...formNavigationProps}
      className="grid grid-cols-4 items-end gap-4"
    >
      {boxes}
    </div>
  );
}
