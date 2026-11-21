// The filters of the sales page (PRD FR-9.9, plan 83): the period (`PeriodFilterBox`: one date, a month, a year or a
// range of dates), the party (all, a buyer or a seller, then one of that side), one item and the goods (all,
// consigned, trip or garden). They sit above the table in one card, like the filters of every list: arrows move
// between the boxes and a picker searches both scripts (Code Standards F40). The party picker lists only the side
// that is chosen, and every party box starts at All (PRD FR-19.1).
//
// Nothing here is money, and nothing is worked out: the page keeps the filters in its address
// (`sales-search.ts`) and the backend reads the days.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  AccountPicker,
  Field,
  FilterCard,
  Picker,
  SegmentedControl,
  Select,
  fieldError,
  type PickerValue,
} from '../../../shared/ui';
import { COMMISSION_SIDES } from '../commission-search';
import { SALES_GOODS, type SalesFilters, type SalesSide } from '../sales-search';
import { PeriodFilterBox } from './PeriodFilterBox';

interface SalesFilterBarProps {
  filters: SalesFilters;
  onChange: (change: Partial<SalesFilters>) => void;
  /** The backend's today, `YYYY-MM-DD`, or null until it is known. */
  today: string | null;
  /** The backend's refusal of the last read, so a wrong date shows under its box. */
  error: UiError | null;
}

/** A record the picker chose, as the value the page keeps (both names, so a language switch is free). */
function valueOf(
  hit: { id: number; name_ur: string; name_en: string; code?: string | undefined } | null,
): PickerValue | null {
  return hit === null
    ? null
    : { id: hit.id, name_ur: hit.name_ur, name_en: hit.name_en, code: hit.code };
}

export function SalesFilterBar({ filters, onChange, today, error }: SalesFilterBarProps) {
  const { l10n } = useLocalization();
  const { side } = filters;

  const sideOptions: readonly { value: SalesSide | 'all'; label: ReactNode }[] = [
    { value: 'all', label: <Localized id="sales-side-all" /> },
    ...COMMISSION_SIDES.map((value) => ({
      value,
      label: <Localized id={`sales-side-${value}`} />,
    })),
  ];

  return (
    <FilterCard
      role="group"
      aria-label={l10n.getString('sales-filters-label')}
      {...formNavigationProps}
      className="filter-grid"
    >
      <PeriodFilterBox filters={filters} onChange={onChange} today={today} error={error} />
      <Field label={<Localized id="sales-side-label" />} hideLabel>
        {() => (
          <div>
            <SegmentedControl<SalesSide | 'all'>
              options={sideOptions}
              value={side ?? 'all'}
              groupLabel={l10n.getString('sales-side-label')}
              onChange={(next) => {
                // A new side starts with nobody picked: a buyer is not a seller.
                onChange(
                  next === 'all' ? { side: null, party: null } : { side: next, party: null },
                );
              }}
            />
          </div>
        )}
      </Field>
      {side === null ? null : (
        <Field
          key={side}
          label={<Localized id={`sales-side-${side}`} />}
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
                onChange({ party: valueOf(hit) });
              }}
            />
          )}
        </Field>
      )}
      <Field label={<Localized id="report-filter-item" />} error={fieldError(error, 'item_id')}>
        {(control) => (
          <Picker
            {...control}
            canAdd={false}
            allChoice
            scope="item"
            value={filters.item}
            onChange={(hit) => {
              onChange({ item: valueOf(hit) });
            }}
          />
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
            options={SALES_GOODS.map((choice) => ({
              value: choice,
              label: l10n.getString(`analytics-goods-${choice}`),
            }))}
          />
        )}
      </Field>
    </FilterCard>
  );
}
