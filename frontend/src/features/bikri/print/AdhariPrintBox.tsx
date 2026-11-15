// The second print box of the sale book (PRD FR-12.12, FR-19.15, plan 68, plan 76 point 3), opened with
// Shift+F9 or its button next to the seller-bill print. It asks two things: what to print, the buyer
// parchi (one short slip for each buyer, several to an A4 page; the default) or the overall sale print
// (the sellers' slips and the buyers' slips), and which lines, the whole day of the saved بکری (the default)
// or this بکری only. Then it is the slips' own print preview (language, paper, Print and Save as PDF), so
// what is previewed is what is printed (`DaySlipsPreview`). The backend works out every figure and writes
// every date; nothing here adds up.

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
import { DaySlipsPreview, type DaySlipsScope } from '../../reports';
import { useBikriDetail } from '../hooks/useBikriReads';
import { daySlipsScope, type AdhariScope, type SlipsKind } from './adhari-filters';
import { mixedSaveOf } from './mixed-save-ids';
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
  const [kind, setKind] = useState<SlipsKind>('parchi');
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
  // A save with carried lots made this بکری: "this بکری only" is the whole save (its new بکری and the old بکری of
  // each carried lot), so the print holds everything it sold.
  const save = mixedSaveOf(target.id);
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
      label:
        save === null ? (
          <Localized
            id="print-adhari-this-bikri"
            vars={{ number: String(target.number) }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        ) : (
          <Localized
            id="print-adhari-this-save"
            vars={{ numbers: save.numbers.join(', ') }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        ),
    },
  ];
  const kindOptions: readonly SegmentedOption<SlipsKind>[] = [
    { value: 'parchi', label: <Localized id="print-adhari-kind-parchi" /> },
    { value: 'sale', label: <Localized id="print-adhari-kind-sale" /> },
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
      <Field label={<Localized id="print-adhari-kind" />}>
        {() => (
          <div>
            <SegmentedControl
              options={kindOptions}
              value={kind}
              groupLabel={l10n.getString('print-adhari-kind')}
              onChange={setKind}
            />
          </div>
        )}
      </Field>
      {/* A new scope or kind is a new page: the preview starts again, with no old notice about the other page. */}
      <DaySlipsPreview
        key={`${kind}-${scope}`}
        variant={kind}
        scope={daySlipsScope(scope, day, target.id, save?.ids ?? null) satisfies DaySlipsScope}
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
