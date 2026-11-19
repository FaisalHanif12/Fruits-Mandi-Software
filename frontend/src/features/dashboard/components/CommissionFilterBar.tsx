// The filters of the commission page (PRD FR-9.3, plan 77): the period (`PeriodFilterBox`: one date, a month, a year
// or a range of dates) and the party (all, a buyer or a seller, then one of that side). They sit above the table
// in one card, like the filters of every list: arrows move between the boxes and a picker searches both scripts
// (Code Standards F40). The party picker lists only the side that is chosen: a name that is not one of that side
// lists nothing (PRD FR-9.3).
//
// Nothing here is money, and nothing is worked out: the page keeps the filters in its address
// (`commission-search.ts`) and the backend reads the days.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  AccountPicker,
  Field,
  FilterCard,
  SegmentedControl,
  Select,
  fieldError,
  type PickerValue,
} from '../../../shared/ui';
import {
  COMMISSION_GOODS,
  COMMISSION_SIDES,
  type CommissionFilters,
  type CommissionSide,
} from '../commission-search';
import { PeriodFilterBox } from './PeriodFilterBox';

interface CommissionFilterBarProps {
  filters: CommissionFilters;
  onChange: (change: Partial<CommissionFilters>) => void;
  /** The backend's today, `YYYY-MM-DD`, or null until it is known. */
  today: string | null;
  /** The backend's refusal of the last read, so a wrong date shows under its box. */
  error: UiError | null;
}

export function CommissionFilterBar({ filters, onChange, today, error }: CommissionFilterBarProps) {
  const { l10n } = useLocalization();
  const { side } = filters;

  const sideOptions: readonly { value: CommissionSide | 'all'; label: ReactNode }[] = [
    { value: 'all', label: <Localized id="commission-side-all" /> },
    ...COMMISSION_SIDES.map((value) => ({
      value,
      label: <Localized id={`commission-side-${value}`} />,
    })),
  ];

  return (
    <FilterCard
      role="group"
      aria-label={l10n.getString('commission-filters-label')}
      {...formNavigationProps}
      className="filter-grid"
    >
      <PeriodFilterBox filters={filters} onChange={onChange} today={today} error={error} />
      <Field label={<Localized id="commission-side-label" />} hideLabel>
        {() => (
          <div>
            <SegmentedControl<CommissionSide | 'all'>
              options={sideOptions}
              value={side ?? 'all'}
              groupLabel={l10n.getString('commission-side-label')}
              onChange={(next) => {
                // A new side starts with nobody picked: a buyer is not a seller.
                onChange(
                  next === 'all'
                    ? { side: null, party: null, person: null }
                    : { side: next, party: null, person: null },
                );
              }}
            />
          </div>
        )}
      </Field>
      <Field label={<Localized id="analytics-filter-goods" />} error={fieldError(error, 'goods')}>
        {(control) => (
          <Select
            {...control}
            value={filters.goods}
            onChange={(goods) => {
              onChange({ goods });
            }}
            options={COMMISSION_GOODS.map((choice) => ({
              value: choice,
              label: l10n.getString(`analytics-goods-${choice}`),
            }))}
          />
        )}
      </Field>
      {side === null ? null : (
        <Field
          key={side}
          label={<Localized id={`commission-side-${side}`} />}
          error={fieldError(error, 'party.account_id')}
        >
          {(control) => (
            <AccountPicker
              {...control}
              canAdd={false}
              allChoice
              scope={side}
              showCard={false}
              value={filters.party}
              onChange={(hit) => {
                const party: PickerValue | null =
                  hit === null
                    ? null
                    : { id: hit.id, name_ur: hit.name_ur, name_en: hit.name_en, code: hit.code };
                onChange({ party, person: null });
              }}
            />
          )}
        </Field>
      )}
    </FilterCard>
  );
}
