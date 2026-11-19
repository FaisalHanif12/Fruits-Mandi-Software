// The filters of the kharch page (PRD FR-9.5, plan 77): the period (`PeriodFilterBox`: one date, a month, a year
// or a range of dates) and one partner for the partner part (all partners when none is chosen). They sit above the
// two parts in one card, like the filters of every list: arrows move between the boxes. Nothing here is money and
// nothing is worked out: the page keeps the filters in its address (`kharch-search.ts`).

import { useLocalization } from '@fluent/react';
import { formNavigationProps } from '../../../shared/keyboard';
import { FilterCard } from '../../../shared/ui';
import { ReportPartnerSelect } from '../../reports';
import type { KharchFilters } from '../kharch-search';
import { PeriodFilterBox } from './PeriodFilterBox';

interface KharchFilterBarProps {
  filters: KharchFilters;
  onChange: (change: Partial<KharchFilters>) => void;
  /** The backend's today, `YYYY-MM-DD`, or null until it is known. */
  today: string | null;
}

export function KharchFilterBar({ filters, onChange, today }: KharchFilterBarProps) {
  const { l10n } = useLocalization();
  return (
    <FilterCard
      role="group"
      aria-label={l10n.getString('kharch-filters-label')}
      {...formNavigationProps}
      className="filter-grid"
    >
      <PeriodFilterBox filters={filters} onChange={onChange} today={today} error={null} />
      <ReportPartnerSelect
        partner={filters.partner}
        required={false}
        error={undefined}
        onChange={(partner) => {
          onChange({ partner });
        }}
      />
    </FilterCard>
  );
}
