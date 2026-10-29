// The period box of a list (plan 38 R4, PRD FR-19.2): a choice of today, this week, this month, last
// month, this year, last year, from the first date, or two dates typed by hand. It gives the cells of
// a `FormGrid` (the choice, and the two dates only while "choose the dates" is on). The days come
// from `usePeriod`, which also remembers the choice for the screen.

import { Localized } from '@fluent/react';
import { PERIOD_KINDS, periodMessageId, type PeriodKind } from '../format';
import type { PeriodControl } from '../hooks';
import { DateInput } from './DateInput';
import { Field } from './Field';
import { Select, type SelectOption } from './Select';

interface PeriodBoxProps {
  period: PeriodControl;
  /** The kinds this list offers (default: all of them). */
  kinds?: readonly PeriodKind[];
}

export function PeriodBox({ period, kinds = PERIOD_KINDS }: PeriodBoxProps) {
  const { state, range, change } = period;
  const options: readonly SelectOption<PeriodKind>[] = kinds.map((kind) => ({
    value: kind,
    label: <Localized id={periodMessageId(kind)} />,
  }));
  return (
    <>
      <Field label={<Localized id="common-period-label" />}>
        {(control) => (
          <Select<PeriodKind>
            {...control}
            value={state.kind}
            options={options}
            onChange={(kind) => {
              // "Choose the dates" starts from the days the period had, so he edits them.
              change(
                kind === 'custom'
                  ? { kind, from: range.from, to: range.to }
                  : { kind, from: '', to: '' },
              );
            }}
          />
        )}
      </Field>
      {state.kind === 'custom' ? (
        <>
          <Field label={<Localized id="common-period-from" />}>
            {(control) => (
              <DateInput
                {...control}
                value={state.from}
                onChange={(from) => {
                  change({ ...state, from });
                }}
              />
            )}
          </Field>
          <Field label={<Localized id="common-period-to" />}>
            {(control) => (
              <DateInput
                {...control}
                value={state.to}
                onChange={(to) => {
                  change({ ...state, to });
                }}
              />
            )}
          </Field>
        </>
      ) : null}
    </>
  );
}
