// The print menu of the sale book (PRD FR-12.12, FR-19.15, plan 78 A1), opened with F9 or its button on a saved
// بکری. It asks two things: what to print, with one line saying what each print is for (the buyer slips, the
// seller bills, or the whole sale print), and which lines, this بکری only (or the whole save that made it) or the
// whole day. Then it is the print's own preview (language, paper, Print and Save as PDF), so what is previewed is
// what is printed (`DaySlipsPreview`). The backend works out every figure and writes every date; nothing here
// adds up. ادھاری تفصیل has its own box (AdhariSheetBox): it is not one of these three.

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
import type { PrintTarget } from './target';

interface PrintMenuBoxProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: PrintTarget;
}

/** What each print is for, the line under the choice. */
const PURPOSE_ID: Record<SlipsKind, string> = {
  parchi: 'print-menu-buyers-purpose',
  'seller-bills': 'print-menu-sellers-purpose',
  sale: 'print-menu-sale-purpose',
};

function MenuBody({ target, onClose }: { target: PrintTarget; onClose: () => void }) {
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
          id="print-menu-whole-day"
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
            id="print-menu-this-bikri"
            vars={{ number: String(target.number) }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        ) : (
          <Localized
            id="print-menu-this-save"
            vars={{ numbers: save.numbers.join(', ') }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        ),
    },
  ];
  const kindOptions: readonly SegmentedOption<SlipsKind>[] = [
    { value: 'parchi', label: <Localized id="print-menu-buyers" /> },
    { value: 'seller-bills', label: <Localized id="print-menu-sellers" /> },
    { value: 'sale', label: <Localized id="print-menu-sale" /> },
  ];
  return (
    <div className="flex flex-col gap-3">
      <Localized id="print-menu-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <Field label={<Localized id="print-menu-kind" />}>
        {() => (
          <div className="flex flex-col gap-2">
            <SegmentedControl
              options={kindOptions}
              value={kind}
              groupLabel={l10n.getString('print-menu-kind')}
              onChange={setKind}
            />
            {/* What the chosen print is for, one line (plan 78 A0). */}
            <Localized id={PURPOSE_ID[kind]}>
              <p className="type-body text-ink-muted" />
            </Localized>
          </div>
        )}
      </Field>
      <Field label={<Localized id="print-menu-scope" />}>
        {() => (
          <div>
            <SegmentedControl
              options={options}
              value={scope}
              groupLabel={l10n.getString('print-menu-scope')}
              onChange={setScope}
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

export function PrintMenuBox({ open, onOpenChange, target }: PrintMenuBoxProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="print-box-title" vars={{ number: String(target.number) }} />}
    >
      {/* The body is made when the box opens, so every opening starts from the whole day. */}
      {open ? (
        <MenuBody
          target={target}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
