// "Put the old opening balances into روکڑ" (پرانے ابتدائی بقایا روکڑ میں درج کریں, PRD FR-10.9 v1.60, plan 99): the
// one-time fix of the openings of sellers, buyers, people lent money and الگ کھاتے that were saved against the
// opening difference. It lists each one with what it does to cash, cash and the opening difference before and after,
// and what days reopen. The save asks the reason and the Owner's sign (PRD FR-21.4); the backend reopens and closes
// the days itself, and a cash below zero is asked first (FR-6.7). No party balance and no partner total changes.
// Every figure is the backend's (CLAUDE.md rule 1). It is shown in place of the book, with a way back (Esc).

import { Localized, useLocalization } from '@fluent/react';
import { ArrowRight, Landmark } from 'lucide-react';
import { useState } from 'react';
import type { OpeningsToCashPreview } from '../../../bindings/OpeningsToCashPreview';
import type { OpeningToCashRow } from '../../../bindings/OpeningToCashRow';
import { cashAfterOfRefusal, uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate, isZeroPaisa, type Paisa } from '../../../shared/format';
import { Pager } from '../../../shared/grid';
import { useClientPager } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape } from '../../../shared/keyboard';
import {
  Balance,
  Button,
  CashConfirmDialog,
  Code,
  ErrorState,
  Fact,
  ListPage,
  LoadingState,
  Notice,
  Num,
  ReasonDialog,
  showToast,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { useMoveOpeningsToCash, useOpeningsToCashPreview } from '../hooks';
import { SignedNum } from './SignedNum';

interface OpeningsToCashViewProps {
  onBack: () => void;
}

/** Cash before and after, and the opening difference before and after, from the preview. */
function Effects({ preview }: { preview: OpeningsToCashPreview }) {
  return (
    <dl className="grid grid-cols-2 gap-3 normal:grid-cols-4">
      <Fact stat label={<Localized id="cashbook-openings-cash-before" />}>
        <SignedNum inherit paisa={preview.cash_before_paisa} />
      </Fact>
      <Fact stat label={<Localized id="cashbook-openings-cash-after" />}>
        <SignedNum inherit paisa={preview.cash_after_paisa} />
      </Fact>
      <Fact stat label={<Localized id="cashbook-openings-difference-before" />}>
        <Balance inherit balance={preview.opening_difference_before} />
      </Fact>
      <Fact stat label={<Localized id="cashbook-openings-difference-after" />}>
        <Balance inherit balance={preview.opening_difference_after} />
      </Fact>
    </dl>
  );
}

/** An amount of a side: nothing when it is 0, so the other side is easy to see. */
function side(amount: Paisa) {
  return isZeroPaisa(amount) ? null : <Num paisa={amount} />;
}

/** The openings the fix moves, a page at a time, with the backend's totals under بنام and جمع. */
function OpeningsTable({ preview }: { preview: OpeningsToCashPreview }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const { pageRows, pager } = useClientPager('cashbook.openings_to_cash', '', preview.rows);
  const columns: readonly TableColumn<OpeningToCashRow>[] = [
    {
      id: 'code',
      header: <Localized id="cashbook-openings-col-code" />,
      kind: 'number',
      size: 'code',
      cell: (row) => <Code>{row.account.code}</Code>,
    },
    {
      id: 'account',
      header: <Localized id="cashbook-openings-col-account" />,
      cell: (row) => displayName(row.account, language),
    },
    {
      id: 'kind',
      header: <Localized id="cashbook-openings-col-kind" />,
      cell: (row) => <Localized id={`cashbook-openings-kind-${row.kind.replaceAll('_', '-')}`} />,
    },
    {
      id: 'date',
      header: <Localized id="cashbook-col-date" />,
      kind: 'number',
      size: 'date',
      cell: (row) => <Code>{isoToDisplayDate(row.date)}</Code>,
    },
    {
      id: 'dr',
      header: <Localized id="cashbook-openings-col-out" />,
      kind: 'number',
      size: 'money',
      cell: (row) => side(row.dr_paisa),
      total: <Num paisa={preview.total_dr_paisa} />,
    },
    {
      id: 'cr',
      header: <Localized id="cashbook-openings-col-in" />,
      kind: 'number',
      size: 'money',
      cell: (row) => side(row.cr_paisa),
      total: <Num paisa={preview.total_cr_paisa} />,
    },
  ];
  return (
    <div className="flex flex-col gap-2">
      <Table
        label={l10n.getString('cashbook-openings-table')}
        columns={columns}
        rows={pageRows ?? []}
        rowKey={(row) => String(row.account.id)}
      />
      <Pager pager={pager} />
    </div>
  );
}

/** The box of the reason and sign, then the cash question when cash ends below zero, then the save. */
function SaveBoxes({
  preview,
  asking,
  onAskingChange,
  onDone,
}: {
  preview: OpeningsToCashPreview;
  asking: boolean;
  onAskingChange: (open: boolean) => void;
  onDone: () => void;
}) {
  const move = useMoveOpeningsToCash();
  // The reason and sign already typed, while the cash question is asked; null when it is not asked.
  const [cashAsk, setCashAsk] = useState<{ reason: string; sign: string; after: Paisa } | null>(
    null,
  );

  const send = (reason: string, sign: string, confirmCash: boolean) => {
    move.mutate(
      { reason, sign, confirm_cash_below_zero: confirmCash },
      {
        onSuccess: () => {
          onAskingChange(false);
          showToast('success', <Localized id="cashbook-openings-done" />);
          onDone();
        },
        onError: (failure) => {
          // The cash changed since the preview: ask the question of FR-6.7 with the figure of the refusal.
          const after = cashAfterOfRefusal(failure);
          if (!confirmCash && after !== null) {
            onAskingChange(false);
            setCashAsk({ reason, sign, after });
          }
        },
      },
    );
  };
  const confirm = (reason: string, sign = '') => {
    if (preview.needs_cash_confirm) {
      onAskingChange(false);
      setCashAsk({ reason, sign, after: preview.cash_after_paisa });
      return;
    }
    send(reason, sign, false);
  };

  return (
    <>
      <ReasonDialog
        open={asking}
        onOpenChange={(open) => {
          onAskingChange(open);
          if (!open) move.reset();
        }}
        title={<Localized id="cashbook-openings-ask-title" />}
        description={<Localized id="cashbook-openings-ask-text" />}
        reasonLabel={<Localized id="cashbook-openings-reason" />}
        signLabel={<Localized id="edit-fix-sign-label" />}
        signHint={<Localized id="cashbook-openings-sign-hint" />}
        confirmLabel={<Localized id="cashbook-openings-confirm" />}
        confirmVariant="primary"
        error={move.isError ? uiErrorOf(move.error) : null}
        busy={move.isPending}
        onConfirm={confirm}
      />
      {cashAsk === null ? null : (
        <CashConfirmDialog
          open
          onOpenChange={(open) => {
            if (!open) setCashAsk(null);
          }}
          cashAfter={cashAsk.after}
          onConfirm={() => {
            const asked = cashAsk;
            setCashAsk(null);
            // A refusal of this save opens the box of the reason again, where it is told.
            onAskingChange(true);
            send(asked.reason, asked.sign, true);
          }}
        />
      )}
    </>
  );
}

export function OpeningsToCashView({ onBack }: OpeningsToCashViewProps) {
  const read = useOpeningsToCashPreview();
  const [asking, setAsking] = useState(false);
  useEscape('leave', onBack);
  const preview = read.data;
  const nothing = preview?.rows.length === 0;

  let body;
  if (read.isError) {
    body = (
      <ErrorState
        title={<Localized id="cashbook-openings-failed" />}
        error={uiErrorOf(read.error)}
        onRetry={() => {
          void read.refetch();
        }}
      />
    );
  } else if (preview === undefined) {
    body = <LoadingState />;
  } else if (nothing) {
    body = (
      <Notice tone="info">
        <Localized id="cashbook-openings-nothing" />
      </Notice>
    );
  } else {
    body = (
      <div className="flex flex-col gap-3">
        {/* The row of the روکڑ tabs stands in for the page's name, so the view says what it is here. */}
        <Localized id="cashbook-openings-title">
          <h2 className="type-section text-ink" />
        </Localized>
        <Localized id="cashbook-openings-explain">
          <p className="type-body text-ink-muted" />
        </Localized>
        <Effects preview={preview} />
        {preview.reopens_from === null ? null : (
          <Notice tone="info">
            <Localized
              id="cashbook-openings-reopens"
              vars={{ date: isoToDisplayDate(preview.reopens_from) }}
              elems={{ num: <Num /> }}
            >
              <span />
            </Localized>
          </Notice>
        )}
        {preview.needs_cash_confirm ? (
          <Notice tone="warn">
            <Localized id="cashbook-openings-below-zero" />
          </Notice>
        ) : null}
        <OpeningsTable preview={preview} />
        <SaveBoxes preview={preview} asking={asking} onAskingChange={setAsking} onDone={onBack} />
      </div>
    );
  }

  return (
    <ListPage
      icon={Landmark}
      titleId="cashbook-openings-title"
      actions={
        <>
          <Button shortcut={SHORTCUTS.back} onClick={onBack}>
            <Localized id="common-back" />
          </Button>
          {preview === undefined || nothing ? null : (
            <Button
              variant="primary"
              onClick={() => {
                setAsking(true);
              }}
            >
              <ArrowRight aria-hidden="true" className="size-icon-md rtl:rotate-180" />
              <Localized id="cashbook-openings-button" />
            </Button>
          )}
        </>
      }
    >
      <div className="min-h-0 flex-1 overflow-y-auto">{body}</div>
    </ListPage>
  );
}
