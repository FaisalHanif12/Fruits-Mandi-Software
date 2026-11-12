// The box of "اسٹیٹمنٹ پرنٹ" (plan 74 T3, PRD FR-12.47), opened from the top of a party's statement. It asks one
// thing, which period: the last 10 days, the last 30 days (the default), the last 6 months, or two dates the
// user types (FR-7.19 shortcuts: "10" is the 10th of this month, `+` and `-` move a day). Then it is the
// statement's own print preview (print language, paper, Print and Save PDF), so what is previewed is what is
// printed. The period is only dates as typed; the backend reads and checks them and builds every figure.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { useDebouncedValue } from '../../../shared/hooks';
import {
  DateInput,
  ErrorState,
  Field,
  FormDialog,
  LoadingState,
  SegmentedControl,
  type SegmentedOption,
} from '../../../shared/ui';
import { Num } from '../../../shared/ui/Num';
import { uiErrorOf } from '../../../shared/api';
import { StatementPreview } from './StatementPreview';
import {
  DEFAULT_STATEMENT_PERIOD,
  STATEMENT_PERIODS,
  STATEMENT_PERIOD_MESSAGE,
  statementRange,
  type StatementPeriod,
} from './statement-period';

/** A typed date is sent after the user stops typing for this long (G8). */
const DATE_DELAY_MS = 400;

interface StatementPrintBoxProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  accountId: number;
}

function BoxBody({ accountId, onClose }: { accountId: number; onClose: () => void }) {
  const { l10n } = useLocalization();
  const today = useDayStatus();
  const [period, setPeriod] = useState<StatementPeriod>(DEFAULT_STATEMENT_PERIOD);
  const [typedFrom, setTypedFrom] = useState('');
  const [typedTo, setTypedTo] = useState('');
  const from = useDebouncedValue(typedFrom.trim(), DATE_DELAY_MS);
  const to = useDebouncedValue(typedTo.trim(), DATE_DELAY_MS);

  if (today.isPending) return <LoadingState />;
  if (today.isError) {
    return (
      <ErrorState
        title={<Localized id="print-preview-failed" />}
        error={uiErrorOf(today.error)}
        onRetry={() => {
          void today.refetch();
        }}
      />
    );
  }
  const range = statementRange(period, today.data.today);
  const options: readonly SegmentedOption<StatementPeriod>[] = STATEMENT_PERIODS.map((kind) => ({
    value: kind,
    label: (
      <Localized id={STATEMENT_PERIOD_MESSAGE[kind]} elems={{ num: <Num inherit /> }}>
        <span />
      </Localized>
    ),
  }));
  // The dates the print is for: a preset's days, or what the user typed (once both are there).
  const shown = range ?? { from, to };
  const ready = shown.from !== '' && shown.to !== '';
  return (
    <div className="flex flex-col gap-3">
      <Localized id="statement-print-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <Field label={<Localized id="statement-print-period" />}>
        {() => (
          <div>
            <SegmentedControl
              options={options}
              value={period}
              groupLabel={l10n.getString('statement-print-period')}
              onChange={(next) => {
                // Own dates start from the last 30 days, so the user changes what is there.
                if (next === 'custom' && typedFrom === '' && typedTo === '') {
                  const start = statementRange(DEFAULT_STATEMENT_PERIOD, today.data.today);
                  setTypedFrom(start?.from ?? '');
                  setTypedTo(start?.to ?? '');
                }
                setPeriod(next);
              }}
            />
          </div>
        )}
      </Field>
      {period === 'custom' ? (
        <div className="grid grid-cols-2 gap-4">
          <Field label={<Localized id="statement-print-from" />}>
            {(control) => <DateInput {...control} value={typedFrom} onChange={setTypedFrom} />}
          </Field>
          <Field label={<Localized id="statement-print-to" />}>
            {(control) => <DateInput {...control} value={typedTo} onChange={setTypedTo} />}
          </Field>
        </div>
      ) : (
        <Localized
          id="statement-print-range"
          vars={{ from: shown.from, to: shown.to }}
          elems={{ num: <Num inherit /> }}
        >
          <p className="type-body text-ink" />
        </Localized>
      )}
      {/* A new period is a new page: the preview starts again, with no old notice about the other page. */}
      {ready ? (
        <StatementPreview
          key={`${shown.from}|${shown.to}`}
          accountId={accountId}
          from={shown.from}
          to={shown.to}
          onBack={onClose}
        />
      ) : null}
    </div>
  );
}

export function StatementPrintBox({ open, onOpenChange, accountId }: StatementPrintBoxProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="statement-print-title" />}
    >
      {/* The body is made when the box opens, so every opening starts from the last 30 days. */}
      {open ? (
        <BoxBody
          accountId={accountId}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
