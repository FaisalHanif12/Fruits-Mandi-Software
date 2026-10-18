// Cancelling a whole بکری (PRD FR-12.11, FR-12.22, FR-7.8, FR-6.7). Nothing is deleted: every
// posting of the بکری is reversed on its own day, the money goes back, the receipts that were
// matched to its lines become the buyers' credit again and are matched to their next oldest bills,
// and the بکری stays in the list marked cancelled, with its number kept and never used again.
//
// 1. The box names what will be reversed (each posting, with its day and amount, from the backend's
//    `bikri_cancel_preview`), shows the cash line, and asks for the reason (required).
// 2. If a posting is on a closed day the cancel is refused until the Owner reopens it: the box says
//    which days and has a button to the day screen. Nothing is sent.
// 3. If the cancel would take cash in hand below zero, the box "is this entry correct?" opens after
//    the reason (focus on "No"); "No" goes back to the reason with what was typed (PRD FR-6.7).
// The screen sends text and ids and shows the backend's numbers (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { useRef, useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { formatPaisa, isoToDisplayDate, useGrouping } from '../../../shared/format';
import { CashConfirmDialog, Code, Num, ReasonDialog, showToast } from '../../../shared/ui';
import { useCancelPreview } from '../hooks/useBikriReads';
import { useCancelBikri } from '../hooks/useStockActions';
import { ClosedDayNotice } from './ClosedDayNotice';

interface CancelBikriFlowProps {
  /** The بکری to cancel, or null while the box is closed. */
  target: { id: number; number: number } | null;
  onClose: () => void;
  /** It was cancelled. */
  onCancelled: () => void;
}

const POSTING_LABEL: Record<string, string> = {
  bikri_first: 'bikri-posting-first',
  bikri_later: 'bikri-posting-later',
  reversal: 'bikri-posting-reversal',
};

export function CancelBikriFlow({ target, onClose, onCancelled }: CancelBikriFlowProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const preview = useCancelPreview(target?.id ?? null);
  const cancel = useCancelBikri();
  const [error, setError] = useState<UiError | null>(null);
  // Which box is open: the reason, the cash question, or none while the cancel is on its way.
  const [step, setStep] = useState<'reason' | 'cash' | 'sending'>('reason');
  // The reason waits here while the cash box is open, so "No" comes back to it intact.
  const [typedReason, setTypedReason] = useState('');
  // The cash box calls onConfirm and then onOpenChange(false): only a "no" is a decline.
  const confirmed = useRef(false);
  const info = preview.data;

  const reset = () => {
    setError(null);
    setStep('reason');
    setTypedReason('');
  };
  const close = () => {
    reset();
    onClose();
  };

  const send = (reason: string, confirm: boolean) => {
    if (target === null || info === undefined) return;
    setError(null);
    setTypedReason(reason);
    setStep('sending');
    cancel
      .mutateAsync({ id: target.id, version: info.version, reason, confirm })
      .then(() => {
        showToast(
          'success',
          <Localized id="bikri-cancelled-toast" vars={{ number: String(target.number) }}>
            <span />
          </Localized>,
        );
        reset();
        onCancelled();
      })
      .catch((failure: unknown) => {
        const refused = uiErrorOf(failure);
        if (refused.errors.some((item) => item.code === 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED')) {
          // The cash changed since the box opened: ask again with the numbers as they are now.
          void preview.refetch();
          setStep('cash');
          return;
        }
        setError(refused);
        setStep('reason');
      });
  };

  const onReason = (reason: string) => {
    if (info?.cash.needs_confirm === true) {
      setTypedReason(reason);
      setStep('cash');
      return;
    }
    send(reason, false);
  };

  const cashChanges =
    info !== undefined &&
    formatPaisa(info.cash.cash_before_paisa, { grouping }) !==
      formatPaisa(info.cash.cash_after_paisa, { grouping });

  return (
    <>
      <ReasonDialog
        open={target !== null && step === 'reason'}
        onOpenChange={(open) => {
          if (!open) close();
        }}
        title={<Localized id="bikri-cancel-title" />}
        description={
          target === null ? null : (
            <Localized id="bikri-cancel-text" vars={{ number: String(target.number) }}>
              <span />
            </Localized>
          )
        }
        reasonLabel={<Localized id="bikri-cancel-reason" />}
        confirmLabel={<Localized id="bikri-cancel-confirm" />}
        initialReason={typedReason}
        confirmDisabled={!(info?.can_cancel ?? false)}
        error={error}
        busy={cancel.isPending || preview.isPending}
        extra={
          <div className="flex flex-col gap-2">
            {info === undefined ? null : (
              <>
                <p className="type-label text-ink">
                  <Localized id="bikri-cancel-reverses" />
                </p>
                <ul className="flex flex-col gap-1 type-body text-ink">
                  {info.postings.map((posting) => (
                    <li key={posting.entry_id} className="flex flex-wrap items-center gap-x-3">
                      <span>
                        {l10n.getString(
                          POSTING_LABEL[posting.posting_type] ?? 'bikri-posting-other',
                        )}
                      </span>
                      <Code>{isoToDisplayDate(posting.entry_date)}</Code>
                      <Num paisa={posting.total} />
                    </li>
                  ))}
                </ul>
                {cashChanges ? (
                  <p className="type-body text-ink">
                    <Localized
                      id="bikri-cancel-cash"
                      vars={{
                        before: formatPaisa(info.cash.cash_before_paisa, { grouping }),
                        after: formatPaisa(info.cash.cash_after_paisa, { grouping }),
                      }}
                      elems={{ num: <Num /> }}
                    >
                      <span />
                    </Localized>
                  </p>
                ) : null}
                {info.closed_dates.length === 0 ? null : (
                  <ClosedDayNotice date={info.closed_dates[0] ?? ''} />
                )}
              </>
            )}
            {preview.isError ? (
              <p role="alert" className="type-body text-due">
                <Localized id="bikri-cancel-preview-failed" />
              </p>
            ) : null}
          </div>
        }
        onConfirm={onReason}
      />
      {info === undefined ? null : (
        <CashConfirmDialog
          open={target !== null && step === 'cash'}
          onOpenChange={(open) => {
            if (open) return;
            if (confirmed.current) {
              confirmed.current = false;
              return;
            }
            // Anything but a yes is a no: back to the reason, with what was typed.
            setStep('reason');
          }}
          cashAfter={info.cash.cash_after_paisa}
          onConfirm={() => {
            confirmed.current = true;
            send(typedReason, true);
          }}
        />
      )}
    </>
  );
}
