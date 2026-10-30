// A month that is not closed (PRD FR-17.6): what closing it would post, and the button that closes
// it. The preview is the backend's own plan, the same one the close posts, so the screen and the
// books cannot differ: the income and the shop expenses of the month, the profit between the
// partners, and every line of the entry with its debit and credit. When the month cannot be closed
// yet (not finished, days still open) the backend says so and the screen says it in words. A confirm
// box says exactly what will happen before anything is posted (FR-7.8).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { MonthLine } from '../../../bindings/MonthLine';
import type { MonthRow } from '../../../bindings/MonthRow';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate, isZeroPaisa } from '../../../shared/format';
import { useSingleFlight } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  Button,
  Card,
  Code,
  ConfirmDialog,
  Fact,
  InfoHint,
  LoadingState,
  Num,
  ScriptText,
  Table,
  showToast,
  type TableColumn,
} from '../../../shared/ui';
import { useCloseMonth, useMonthPreview } from '../hooks';
import { kindMessageId, monthName } from '../text';
import { MonthRefusal } from './MonthRefusal';

interface MonthPreviewCardProps {
  row: MonthRow;
  /** Called with the month after it was closed, so the screen keeps showing it (now closed). */
  onClosed: (month: string) => void;
}

export function MonthPreviewCard({ row, onClosed }: MonthPreviewCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const preview = useMonthPreview(row.month, true);
  const close = useCloseMonth();
  // One close at a time: a second Enter on the confirm box does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const [confirming, setConfirming] = useState(false);
  const name = monthName((id) => l10n.getString(id), row.month);
  const data = preview.data;
  // The close's own refusal (the state changed meanwhile) comes before the preview's.
  const refusal =
    close.error !== null
      ? uiErrorOf(close.error)
      : preview.isError
        ? uiErrorOf(preview.error)
        : null;

  const confirm = () => {
    setConfirming(false);
    void flight.run(
      () =>
        close
          .mutateAsync(row.month)
          .then(() => {
            showToast('success', l10n.getString('month-closed-toast', { month: name }));
            onClosed(row.month);
          })
          // The refusal is shown on the card, in the month screen's words.
          .catch(() => undefined),
      undefined,
    );
  };

  const columns: TableColumn<MonthLine>[] = [
    {
      id: 'account',
      header: <Localized id="month-col-account" />,
      cell: (line) => (
        <span className="flex flex-wrap items-baseline gap-x-2">
          <Code>{line.account.code}</Code>
          <ScriptText>{displayName(line.account, language)}</ScriptText>
        </span>
      ),
      total: <Localized id="month-total" />,
    },
    {
      id: 'kind',
      header: <Localized id="month-col-kind" />,
      cell: (line) => l10n.getString(kindMessageId(line.kind)),
    },
    {
      id: 'debit',
      kind: 'number',
      size: 'money',
      header: <Localized id="month-col-debit" />,
      cell: (line) => (isZeroPaisa(line.dr_paisa) ? null : <Num paisa={line.dr_paisa} />),
      total: data === undefined ? undefined : <Num paisa={data.total_paisa} />,
    },
    {
      id: 'credit',
      kind: 'number',
      size: 'money',
      header: <Localized id="month-col-credit" />,
      cell: (line) => (isZeroPaisa(line.cr_paisa) ? null : <Num paisa={line.cr_paisa} />),
      total: data === undefined ? undefined : <Num paisa={data.total_paisa} />,
    },
  ];

  return (
    <Card compact>
      {/* The button that closes the month is on the row of the card's title, so it is in view
          whatever the length of the entry's lines below it (plan 38). */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-1">
          <Localized id="month-preview-title" vars={{ month: name }}>
            <h2 className="type-section text-ink" />
          </Localized>
          <InfoHint>
            <Localized id="month-preview-help">
              <p />
            </Localized>
          </InfoHint>
        </div>
        {data === undefined ? null : (
          <Button
            variant="primary"
            busy={flight.busy || close.isPending}
            onClick={() => {
              setConfirming(true);
            }}
          >
            <Localized id="month-close-button" />
          </Button>
        )}
      </div>
      {refusal === null ? null : <MonthRefusal error={refusal} month={row.month} />}
      {preview.isPending ? <LoadingState /> : null}
      {data === undefined ? null : (
        <>
          <div className="@container min-w-0">
            <dl className="grid grid-cols-2 gap-4 @form3:grid-cols-4">
              <Fact label={<Localized id="month-fact-income" />}>
                <Num paisa={data.income_paisa} />
              </Fact>
              <Fact label={<Localized id="month-fact-expense" />}>
                <Num paisa={data.expense_paisa} />
              </Fact>
              <Fact label={<Localized id="month-fact-profit" />}>
                <Num paisa={data.profit_paisa} />
              </Fact>
              <Fact label={<Localized id="month-fact-date" />}>
                <Num>{isoToDisplayDate(data.entry_date)}</Num>
              </Fact>
            </dl>
          </div>
          <Table
            label={l10n.getString('month-lines-label')}
            columns={columns}
            rows={data.lines}
            rowKey={(line) => `${line.kind}-${String(line.account.id)}`}
          />
          <ConfirmDialog
            open={confirming}
            onOpenChange={setConfirming}
            title={<Localized id="month-close-confirm-title" vars={{ month: name }} />}
            description={
              <Localized
                id="month-close-confirm-description"
                vars={{ month: name, date: isoToDisplayDate(data.entry_date) }}
              />
            }
            confirmLabel={<Localized id="month-close-button" />}
            initialFocus="back"
            onConfirm={confirm}
          />
        </>
      )}
    </Card>
  );
}
