// The print box of ادھاری تفصیل (R5) on the sale book (PRD FR-12.12, plan 68, D8 Feature 8), opened with
// Shift+F9 or its button next to the seller-bill print. R5 is one slip for each buyer of the day: his
// goods, the amount after commission and لاگا, what he paid on the spot and what is left on his
// account. The box asks one thing, which lines: the whole day of the saved بکری (the default) or
// this بکری only. Then it is the report's own print preview (language, paper, preview, Print and
// Save PDF), so what is previewed is what is printed (`BuyerBillsPreview`). The backend works out
// every figure and writes every date; nothing here adds up.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import {
  ErrorState,
  Field,
  FormDialog,
  LoadingState,
  SegmentedControl,
  type SegmentedOption,
} from '../../../shared/ui';
import { Num } from '../../../shared/ui/Num';
import { BuyerBillsPreview } from '../../reports';
import { useBikriDetail } from '../hooks/useBikriReads';
import { adhariFilters, type AdhariScope } from './adhari-filters';
import type { PrintTarget } from './PrintBox';

interface AdhariPrintBoxProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: PrintTarget;
}

function AdhariBody({ target, onClose }: { target: PrintTarget; onClose: () => void }) {
  const { l10n } = useLocalization();
  const detail = useBikriDetail(target.id);
  const [scope, setScope] = useState<AdhariScope>('day');
  if (detail.isPending) return <LoadingState />;
  if (detail.isError) {
    return (
      <ErrorState
        title={<Localized id="print-preview-failed" />}
        error={uiErrorOf(detail.error)}
        onRetry={() => {
          void detail.refetch();
        }}
      />
    );
  }
  const day = detail.data.entry_date;
  const options: readonly SegmentedOption<AdhariScope>[] = [
    {
      value: 'day',
      label: (
        <Localized
          id="print-adhari-whole-day"
          vars={{ date: isoToDisplayDate(day) }}
          elems={{ num: <Num /> }}
        >
          <span />
        </Localized>
      ),
    },
    {
      value: 'bikri',
      label: (
        <Localized
          id="print-adhari-this-bikri"
          vars={{ number: String(target.number) }}
          elems={{ num: <Num /> }}
        >
          <span />
        </Localized>
      ),
    },
  ];
  return (
    <div className="flex flex-col gap-3">
      <Localized id="print-adhari-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <Field label={<Localized id="print-adhari-scope" />}>
        {() => (
          <div>
            <SegmentedControl
              options={options}
              value={scope}
              groupLabel={l10n.getString('print-adhari-scope')}
              onChange={setScope}
            />
          </div>
        )}
      </Field>
      {/* A new scope is a new page: the preview starts again, with no old notice about the other page. */}
      <BuyerBillsPreview
        key={scope}
        filters={adhariFilters(scope, day, target.id)}
        onBack={onClose}
      />
    </div>
  );
}

export function AdhariPrintBox({ open, onOpenChange, target }: AdhariPrintBoxProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="print-adhari-title" />}
    >
      {/* The body is made when the box opens, so every opening starts from the whole day. */}
      {open ? (
        <AdhariBody
          target={target}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
