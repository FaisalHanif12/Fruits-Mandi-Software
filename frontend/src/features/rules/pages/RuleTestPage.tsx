// "Test a rule" (route /index/rules/test; PRD FR-11.8): pick a seller, a buyer, an item, a date,
// a quantity and a rate, and see for each charge which rule wins, at which level, which levels
// were skipped, and the amount with its working. Nothing is saved. The backend works everything
// out (the same function the بکری uses); the screen only shows it. Keys: Enter on the last field
// runs the test, Esc goes back to the rules.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useFocusFirstField } from '../../../shared/hooks';
import { useEscape, useFormNavigation } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  CalcInfo,
  Card,
  DateInput,
  ErrorNotice,
  Field,
  fieldError,
  LinkButton,
  MoneyInput,
  Num,
  Picker,
  type PickerValue,
  QtyInput,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { formatBps, formatPaisa, isoToDisplayDate, useGrouping } from '../../../shared/format';
import { levelMessageId } from '../../../shared/i18n';
import { useTestRule } from '../hooks';
import type { ChargeTest, WinnerView } from '../types';

const FIELD_PATHS = ['seller_id', 'buyer_id', 'item_id', 'date', 'qty', 'rate'];

const VALUE_ID = {
  percent: 'rules-value-percent',
  per_unit: 'rules-value-per-unit',
  per_lot: 'rules-value-per-lot',
} as const;

/** The value of the winning rule as words: "6 percent", "8 per Nag". Written from what the backend sent. */
function WinnerValue({ winner }: { winner: WinnerView }) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const value =
    winner.method === 'percent'
      ? winner.value_bps === null
        ? ''
        : formatBps(winner.value_bps)
      : winner.value_paisa === null
        ? ''
        : formatPaisa(winner.value_paisa, { grouping });
  return <span>{l10n.getString(VALUE_ID[winner.method], { value })}</span>;
}

/** Which rule won, with a link to it. */
function WinnerCell({ charge }: { charge: ChargeTest }) {
  const navigate = useNavigate();
  const { winner } = charge;
  if (winner === null) return <Localized id="rules-test-no-rule" />;
  return (
    <span className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
      {winner.manual ? <Localized id="rules-test-manual" /> : <WinnerValue winner={winner} />}
      {winner.rule_id === null ? null : (
        <LinkButton
          onClick={() => {
            void navigate({
              to: '/index/rules/$id',
              params: { id: String(winner.rule_id) },
            });
          }}
        >
          <Localized id="rules-overlap-open" />
        </LinkButton>
      )}
    </span>
  );
}

const RESULT_COLUMNS: TableColumn<ChargeTest>[] = [
  {
    id: 'charge',
    header: <Localized id="rules-col-charge" />,
    cell: (charge) => (
      <>
        <Localized id={`rules-side-${charge.side}`} />{' '}
        <Localized id={`rules-charge-${charge.charge_type}`} />
      </>
    ),
  },
  {
    id: 'winner',
    header: <Localized id="rules-test-winner" />,
    cell: (charge) => <WinnerCell charge={charge} />,
  },
  {
    id: 'level',
    header: <Localized id="rules-test-level" />,
    cell: ({ winner }) =>
      winner === null ? null : <Localized id={levelMessageId(winner.level)} />,
  },
  {
    id: 'skipped',
    header: <Localized id="rules-test-skipped" />,
    cell: (charge) =>
      charge.skipped.length === 0 ? (
        <Localized id="rules-test-skipped-none" />
      ) : (
        <ul className="flex flex-wrap gap-1">
          {charge.skipped.map((skipped) => (
            <li key={skipped.level} className="rounded-1 bg-surface-sunken px-2 py-1 type-label">
              <Localized id={levelMessageId(skipped.level)} />
            </li>
          ))}
        </ul>
      ),
  },
  {
    id: 'amount',
    header: <Localized id="rules-test-amount" />,
    kind: 'number',
    // The ⓘ has a slot of its own at the start of the cell, so the icons line up and the number
    // sits on the physical right edge (NFR-USE-19).
    cell: (charge) => (
      <div className="flex items-center justify-between gap-1">
        <span className="inline-flex size-control-compact shrink-0 items-center justify-center">
          <CalcInfo working={charge.working} />
        </span>
        <Num paisa={charge.amount_paisa} />
      </div>
    ),
  },
];

export function RuleTestPage() {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const day = useDayStatus();
  const [seller, setSeller] = useState<PickerValue | null>(null);
  const [buyer, setBuyer] = useState<PickerValue | null>(null);
  const [item, setItem] = useState<PickerValue | null>(null);
  // The date starts as today, unless the user has typed one.
  const [typedDate, setTypedDate] = useState<string | null>(null);
  const [qty, setQty] = useState('');
  const [rate, setRate] = useState('');
  const test = useTestRule();
  const [missing, setMissing] = useState<string[]>([]);
  const containerRef = useFocusFirstField<HTMLFormElement>();
  const date = typedDate ?? (day.data === undefined ? '' : isoToDisplayDate(day.data.today));
  const backendError = test.error === null ? null : uiErrorOf(test.error);
  const error: UiError | null =
    missing.length > 0
      ? {
          ok: false,
          errors: missing.map((field) => ({ code: 'ERR_REQUIRED', field, values: {} })),
          ref: '',
        }
      : backendError;

  // The three records are required. A picker left empty is told at once, without asking the
  // backend (a hint only: the backend checks every value again).
  const run = () => {
    const empty = [
      ...(seller === null ? ['seller_id'] : []),
      ...(buyer === null ? ['buyer_id'] : []),
      ...(item === null ? ['item_id'] : []),
    ];
    setMissing(empty);
    if (seller === null || buyer === null || item === null) return;
    test.mutate({ seller_id: seller.id, buyer_id: buyer.id, item_id: item.id, date, qty, rate });
  };
  const navigation = useFormNavigation(run);
  useEscape('leave', () => {
    void navigate({ to: '/index/rules' });
  });
  const toValue = (hit: { id: number; code: string; name_ur: string; name_en: string } | null) =>
    hit === null
      ? null
      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en };

  return (
    <div className="flex max-w-form flex-col gap-5">
      <div className="flex flex-col gap-1">
        <Localized id="rules-test-title">
          <h1 className="type-title" />
        </Localized>
        <Localized id="rules-test-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <form
        ref={containerRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
        <Card>
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            <Field
              label={<Localized id="rules-test-seller" />}
              error={fieldError(error, 'seller_id')}
            >
              {(props) => (
                <AccountPicker
                  {...props}
                  scope="seller"
                  value={seller}
                  onChange={(hit) => {
                    setSeller(toValue(hit));
                    setMissing([]);
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="rules-test-buyer" />}
              error={fieldError(error, 'buyer_id')}
            >
              {(props) => (
                <AccountPicker
                  {...props}
                  scope="buyer"
                  value={buyer}
                  onChange={(hit) => {
                    setBuyer(toValue(hit));
                    setMissing([]);
                  }}
                />
              )}
            </Field>
            <Field label={<Localized id="rules-test-item" />} error={fieldError(error, 'item_id')}>
              {(props) => (
                <Picker
                  {...props}
                  scope="item"
                  value={item}
                  onChange={(hit) => {
                    setItem(toValue(hit));
                    setMissing([]);
                  }}
                />
              )}
            </Field>
            <Field label={<Localized id="rules-test-date" />} error={fieldError(error, 'date')}>
              {(props) => <DateInput {...props} value={date} onChange={setTypedDate} />}
            </Field>
            <Field label={<Localized id="rules-test-qty" />} error={fieldError(error, 'qty')}>
              {(props) => <QtyInput {...props} value={qty} onChange={setQty} />}
            </Field>
            <Field label={<Localized id="rules-test-rate" />} error={fieldError(error, 'rate')}>
              {(props) => <MoneyInput {...props} value={rate} onChange={setRate} />}
            </Field>
          </div>
          <div>
            <Button variant="primary" busy={test.isPending} onClick={run}>
              <Localized id="rules-test-run" />
            </Button>
          </div>
        </Card>
      </form>
      <Card title={<Localized id="rules-test-result" />}>
        {test.data === undefined ? (
          <Localized id={test.isPending ? 'common-loading' : 'rules-test-nothing'}>
            <p className="type-body text-ink-muted" />
          </Localized>
        ) : (
          <div className="flex flex-col gap-3">
            <p className="flex items-baseline gap-2 type-body">
              <Localized id="rules-test-line-value" />
              <Num paisa={test.data.line_value_paisa} />
            </p>
            <Table
              label={l10n.getString('rules-test-result')}
              columns={RESULT_COLUMNS}
              rows={test.data.charges}
              rowKey={(charge) => `${charge.side}-${charge.charge_type}`}
            />
          </div>
        )}
      </Card>
    </div>
  );
}
