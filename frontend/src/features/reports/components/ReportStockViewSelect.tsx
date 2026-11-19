// The layout box of the stock report (R10, PRD 12.8, 19.2): by lot (the default, one row for each
// lot) or by item (one row for each item with its quantities over all its lots, the biggest balance
// first). It changes the columns of the table, not the figures: both layouts add up to the same
// totals over the whole filter.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { ReportStockView } from '../../../bindings/ReportStockView';
import { Field, Select } from '../../../shared/ui';
import { STOCK_VIEW_CHOICES, stockViewMessageId } from '../filters';

interface ReportStockViewSelectProps {
  value: ReportStockView;
  onChange: (value: ReportStockView) => void;
  error?: ReactNode;
}

export function ReportStockViewSelect({ value, onChange, error }: ReportStockViewSelectProps) {
  const { l10n } = useLocalization();
  return (
    <Field label={<Localized id="report-filter-stock-view" />} error={error}>
      {(control) => (
        <Select<ReportStockView>
          {...control}
          value={value}
          onChange={onChange}
          options={STOCK_VIEW_CHOICES.map((choice) => ({
            value: choice,
            label: l10n.getString(stockViewMessageId(choice)),
          }))}
        />
      )}
    </Field>
  );
}
