// The sections of the development gallery. Each uses only shared/ui components and Fluent words.

import { Localized, useLocalization } from '@fluent/react';
import { Users } from 'lucide-react';
import { useState } from 'react';
import type { UiError } from '../../bindings/UiError';
import { formatPaisa, useGrouping } from '../../shared/format';
import { SHORTCUTS } from '../../shared/keyboard';
import {
  Button,
  Card,
  Code,
  CodeInput,
  ConfirmDialog,
  DateInput,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  MoneyInput,
  Num,
  QtyInput,
  Select,
  TextInput,
  showToast,
  type SelectOption,
} from '../../shared/ui';
import { SAMPLE_AMOUNTS, SAMPLE_TRUCK } from './sample';

type Mode = 'cash' | 'bank' | 'cheque';

const MODES: readonly SelectOption<Mode>[] = [
  { value: 'cash', label: <Localized id="gallery-mode-cash" /> },
  { value: 'bank', label: <Localized id="gallery-mode-bank" /> },
  { value: 'cheque', label: <Localized id="gallery-mode-cheque" /> },
];

const SAMPLE_ERROR: UiError = {
  ok: false,
  errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
  ref: 'E-20261010-0007',
};

const SAMPLE_BILL_NUMBER = '1';
const SAMPLE_QTY = '20';
const DECIMAL_POINT = '.';

function Label({ id }: { id: string }) {
  return <Localized id={id} />;
}

export function ButtonsSection() {
  return (
    <Card title={<Label id="gallery-section-buttons" />}>
      <div className="flex flex-wrap items-center gap-3">
        <Button shortcut={SHORTCUTS.newEntry} onClick={() => undefined}>
          <Label id="gallery-button-new" />
        </Button>
        <Button variant="danger" onClick={() => undefined}>
          <Label id="gallery-button-cancel-sale" />
        </Button>
        <Button disabled onClick={() => undefined}>
          <Label id="gallery-button-disabled" />
        </Button>
        <Button variant="primary" busy onClick={() => undefined}>
          <Label id="gallery-button-busy" />
        </Button>
        <Button variant="primary" shortcut={SHORTCUTS.save} onClick={() => undefined}>
          <Label id="gallery-button-save" />
        </Button>
      </div>
    </Card>
  );
}

export function FieldsSection() {
  const { l10n } = useLocalization();
  const [narration, setNarration] = useState('');
  const [amount, setAmount] = useState('15000.50');
  const [qty, setQty] = useState('50');
  const [date, setDate] = useState('10-10-2026');
  const [code, setCode] = useState('1');
  const [mode, setMode] = useState<Mode | undefined>('cash');
  // A layer-1 hint only (Code Standards 4.1): the backend says the same with
  // ERR_DECIMALS_NOT_ALLOWED and has the final word.
  const amountError = amount.includes(DECIMAL_POINT) ? (
    <Label id="err-decimals-not-allowed" />
  ) : undefined;
  return (
    <Card title={<Label id="gallery-section-fields" />}>
      <div className="grid grid-cols-2 gap-x-5 gap-y-4">
        <div className="col-span-2">
          <Field
            label={<Label id="gallery-field-narration" />}
            hint={<Label id="gallery-field-narration-hint" />}
          >
            {(control) => (
              <TextInput
                {...control}
                value={narration}
                onChange={setNarration}
                placeholder={l10n.getString('gallery-field-narration-hint')}
              />
            )}
          </Field>
        </div>
        <Field label={<Label id="gallery-field-amount" />} error={amountError}>
          {(control) => <MoneyInput {...control} value={amount} onChange={setAmount} />}
        </Field>
        <Field
          label={<Label id="gallery-field-qty" />}
          hint={<Label id="gallery-field-qty-hint" />}
        >
          {(control) => <QtyInput {...control} value={qty} onChange={setQty} />}
        </Field>
        <Field
          label={<Label id="gallery-field-date" />}
          hint={<Label id="gallery-field-date-hint" />}
        >
          {(control) => <DateInput {...control} value={date} onChange={setDate} />}
        </Field>
        <Field
          label={<Label id="gallery-field-code" />}
          hint={<Label id="gallery-field-code-hint" />}
        >
          {(control) => <CodeInput {...control} value={code} onChange={setCode} />}
        </Field>
        <Field label={<Label id="gallery-field-mode" />}>
          {(control) => <Select {...control} value={mode} onChange={setMode} options={MODES} />}
        </Field>
        <Field label={<Label id="gallery-field-total" />} readOnly>
          {(control) => <MoneyInput {...control} value="12880" onChange={() => undefined} />}
        </Field>
      </div>
    </Card>
  );
}

export function NumbersSection() {
  const grouping = useGrouping();
  const text = (index: number) => {
    const amount = SAMPLE_AMOUNTS[index];
    return amount === undefined ? '' : formatPaisa(amount, { grouping });
  };
  return (
    <Card title={<Label id="gallery-section-numbers" />}>
      <Localized id="gallery-numbers-gross" vars={{ amount: text(2) }} elems={{ num: <Num /> }}>
        <p className="type-body" />
      </Localized>
      <Localized
        id="gallery-numbers-line"
        vars={{ qty: SAMPLE_QTY, rate: text(3) }}
        elems={{ num: <Num /> }}
      >
        <p className="type-body" />
      </Localized>
      <Localized
        id="gallery-numbers-truck"
        vars={{ truck: SAMPLE_TRUCK }}
        elems={{ code: <Code /> }}
      >
        <p className="type-body" />
      </Localized>
      <div className="flex flex-col rounded-2 border border-border">
        <div dir="ltr" className="bg-surface-sunken px-4 py-2 text-end">
          <Localized id="gallery-numbers-column">
            <span className="type-label text-ink-muted" />
          </Localized>
        </div>
        {SAMPLE_AMOUNTS.slice(0, 3).map((amount, index) => (
          <div key={index} dir="ltr" className="border-t border-border px-4 py-2 text-end">
            <Num paisa={amount} />
          </div>
        ))}
      </div>
    </Card>
  );
}

export function StatesSection() {
  const [loadingKey, setLoadingKey] = useState(0);
  return (
    <Card title={<Label id="gallery-section-states" />}>
      <Localized id="gallery-state-loading">
        <h3 className="type-label text-ink-muted" />
      </Localized>
      <LoadingState key={loadingKey} />
      <div>
        <Button
          onClick={() => {
            setLoadingKey((key) => key + 1);
          }}
        >
          <Label id="gallery-loading-again" />
        </Button>
      </div>
      <Localized id="gallery-state-empty">
        <h3 className="type-label text-ink-muted" />
      </Localized>
      <EmptyState
        icon={Users}
        title={<Label id="gallery-empty-title" />}
        help={<Label id="gallery-empty-help" />}
        action={
          <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={() => undefined}>
            <Label id="gallery-empty-action" />
          </Button>
        }
      />
      <Localized id="gallery-state-error">
        <h3 className="type-label text-ink-muted" />
      </Localized>
      <ErrorState
        title={<Label id="gallery-error-title" />}
        error={SAMPLE_ERROR}
        onRetry={() => undefined}
      />
    </Card>
  );
}

export function MessagesSection() {
  const { l10n } = useLocalization();
  const [confirmOpen, setConfirmOpen] = useState(false);
  return (
    <Card title={<Label id="gallery-section-messages" />}>
      <div className="flex flex-wrap gap-3">
        <Button
          onClick={() => {
            showToast(
              'success',
              l10n.getString('gallery-toast-saved', { number: SAMPLE_BILL_NUMBER }),
            );
          }}
        >
          <Label id="gallery-toast-success-button" />
        </Button>
        <Button
          onClick={() => {
            showToast('error', l10n.getString('gallery-toast-failed'));
          }}
        >
          <Label id="gallery-toast-error-button" />
        </Button>
        <Button
          variant="danger"
          onClick={() => {
            setConfirmOpen(true);
          }}
        >
          <Label id="gallery-confirm-open" />
        </Button>
      </div>
      <ConfirmDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        tone="danger"
        title={<Localized id="gallery-confirm-title" vars={{ number: SAMPLE_BILL_NUMBER }} />}
        description={
          <Localized id="gallery-confirm-description" vars={{ number: SAMPLE_BILL_NUMBER }} />
        }
        confirmLabel={<Label id="gallery-button-cancel-sale" />}
        onConfirm={() => undefined}
      />
    </Card>
  );
}
