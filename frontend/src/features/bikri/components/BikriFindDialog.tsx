// Find a saved بکری by number, truck, seller, day or item (PRD FR-12.13). A small dialog on the
// sale book: the five filters, and under them the بکری that match, newest first (at most 50).
// ↓ from the last filter goes into the results, ↑ ↓ move between them, Enter opens the one chosen.
// Every filter that is given must match (`bikri_find`); a truck number matches by its start, in any
// letter case. The words that tell the user nothing was asked, nothing matched or something failed
// are on the dialog (four states, F8).

import { Localized, useLocalization } from '@fluent/react';
import { SearchX } from 'lucide-react';
import { useState } from 'react';
import type { BikriFindInput } from '../../../bindings/BikriFindInput';
import type { BikriListRow } from '../../../bindings/BikriListRow';
import { uiErrorOf } from '../../../shared/api';
import { formNavigationProps } from '../../../shared/keyboard';
import { isoToDisplayDate, useGrouping, formatPaisa } from '../../../shared/format';
import { useLanguage } from '../../../shared/i18n';
import {
  AccountPicker,
  Code,
  CodeInput,
  DateInput,
  EmptyState,
  ErrorState,
  Field,
  FormDialog,
  LoadingState,
  Num,
  Picker,
  TextInput,
  type PickerValue,
} from '../../../shared/ui';
import { hasFindFilter, useBikriFind } from '../hooks/useBikriReads';
import { sellerNames } from '../list-columns';

interface BikriFindDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Open this saved بکری. */
  onOpen: (id: number) => void;
}

/** A number typed as digits, or null. A بکری number is a count of saves, not money. */
function numberOf(text: string): number | null {
  return /^\d{1,9}$/.test(text.trim()) ? Number(text.trim()) : null;
}

function Result({ row, onOpen }: { row: BikriListRow; onOpen: () => void }) {
  const language = useLanguage();
  const grouping = useGrouping();
  const { l10n } = useLocalization();
  return (
    <button
      type="button"
      data-nav-field
      onClick={onOpen}
      className="flex w-full flex-col gap-1 rounded-2 border border-border px-3 py-2 text-start"
    >
      <span className="flex flex-wrap items-baseline justify-between gap-2">
        <Localized id="bikri-hit-title" vars={{ number: String(row.number) }}>
          <span className="type-field" />
        </Localized>
        <span className="flex items-baseline gap-2">
          {row.status === 'cancelled' ? (
            <Localized id="bikri-status-cancelled">
              <span className="type-label text-due" />
            </Localized>
          ) : null}
          <Code>{isoToDisplayDate(row.entry_date)}</Code>
        </span>
      </span>
      <span className="flex flex-wrap items-baseline gap-x-3 type-body text-ink-muted">
        {row.truck_no === null ? null : <Code>{row.truck_no}</Code>}
        <span>{sellerNames(row, language, l10n.getString('bikri-list-separator'))}</span>
        <Num>{formatPaisa(row.gross, { grouping })}</Num>
      </span>
    </button>
  );
}

export function BikriFindDialog({ open, onOpenChange, onOpen }: BikriFindDialogProps) {
  const { l10n } = useLocalization();
  const resultsLabel = l10n.getString('bikri-find-results');
  const [number, setNumber] = useState('');
  const [truck, setTruck] = useState('');
  const [seller, setSeller] = useState<PickerValue | null>(null);
  const [date, setDate] = useState('');
  const [item, setItem] = useState<PickerValue | null>(null);
  const input: BikriFindInput = {
    number: numberOf(number),
    truck_no: truck.trim() === '' ? null : truck.trim(),
    seller_id: seller?.id ?? null,
    date: date.trim() === '' ? null : date.trim(),
    item_id: item?.id ?? null,
  };
  const asked = hasFindFilter(input);
  const found = useBikriFind(input, open);
  const rows = found.data ?? [];

  let results;
  if (!asked) {
    results = (
      <Localized id="bikri-find-hint">
        <p className="type-body text-ink-muted" />
      </Localized>
    );
  } else if (found.isError) {
    results = (
      <ErrorState
        title={<Localized id="bikri-find-failed" />}
        error={uiErrorOf(found.error)}
        onRetry={() => {
          void found.refetch();
        }}
      />
    );
  } else if (found.data === undefined) {
    results = <LoadingState rows={2} />;
  } else if (rows.length === 0) {
    results = (
      <EmptyState
        icon={SearchX}
        title={<Localized id="bikri-find-none" />}
        help={<Localized id="bikri-find-none-help" />}
      />
    );
  } else {
    results = (
      <ul aria-label={resultsLabel} className="flex flex-col gap-2">
        {rows.map((row) => (
          <li key={row.id}>
            <Result
              row={row}
              onOpen={() => {
                onOpen(row.id);
                onOpenChange(false);
              }}
            />
          </li>
        ))}
      </ul>
    );
  }

  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="bikri-find-title" />}
      description={<Localized id="bikri-find-help" />}
    >
      <div {...formNavigationProps} className="flex flex-col gap-3">
        <div className="grid grid-cols-2 gap-x-3 gap-y-2">
          <Field label={<Localized id="bikri-find-number" />}>
            {(control) => <CodeInput {...control} value={number} onChange={setNumber} />}
          </Field>
          <Field label={<Localized id="bikri-header-truck" />}>
            {(control) => <TextInput {...control} value={truck} onChange={setTruck} lang="en" />}
          </Field>
          <Field label={<Localized id="glossary-beopari" />}>
            {(control) => (
              <AccountPicker
                {...control}
                canAdd={false}
                scope="seller"
                showCard={false}
                value={seller}
                onChange={setSeller}
              />
            )}
          </Field>
          <Field label={<Localized id="bikri-header-date" />}>
            {(control) => <DateInput {...control} value={date} onChange={setDate} />}
          </Field>
          <Field label={<Localized id="glossary-qism-maal" />}>
            {(control) => (
              <Picker
                {...control}
                canAdd={false}
                scope="item"
                value={item}
                onChange={(hit) => {
                  setItem(hit);
                }}
              />
            )}
          </Field>
        </div>
        {results}
      </div>
    </FormDialog>
  );
}
