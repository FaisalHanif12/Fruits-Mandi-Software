// A month that is closed (PRD FR-17.6, FR-21.2): the day it was closed, how many times a close of
// it was reversed before, and the button that reverses the close in force. The reversal is a mirror
// entry dated today; it asks for a reason, which goes to the audit log, in the shared reason box. The
// month can be closed again after it. A reversal that the backend refuses is told inside the box.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { MonthRow } from '../../../bindings/MonthRow';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { useSingleFlight } from '../../../shared/hooks';
import { Button, Card, Fact, Num, ReasonDialog, showToast } from '../../../shared/ui';
import { useReverseMonth } from '../hooks';
import { monthName } from '../text';
import { MonthRefusal } from './MonthRefusal';

interface MonthClosedCardProps {
  row: MonthRow;
  /** Called with the month after its close was reversed, so the screen keeps showing it. */
  onReversed: (month: string) => void;
}

export function MonthClosedCard({ row, onReversed }: MonthClosedCardProps) {
  const { l10n } = useLocalization();
  const reverse = useReverseMonth();
  // One reversal at a time: a second Enter on the box does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const [open, setOpen] = useState(false);
  const name = monthName((id) => l10n.getString(id), row.month);
  const error = reverse.error === null ? null : uiErrorOf(reverse.error);

  const submit = (reason: string) => {
    void flight.run(
      () =>
        reverse
          .mutateAsync({ month: row.month, reason })
          .then(() => {
            setOpen(false);
            showToast('success', l10n.getString('month-reversed-toast', { month: name }));
            onReversed(row.month);
          })
          // The refusal stays in the box, with what was typed.
          .catch(() => undefined),
      undefined,
    );
  };

  return (
    <Card title={<Localized id="month-closed-title" vars={{ month: name }} />}>
      <Localized id="month-closed-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <dl className="grid grid-cols-2 gap-4 normal:grid-cols-4">
        <Fact label={<Localized id="month-closed-on" />}>
          <Num>{row.closed_on === null ? '' : isoToDisplayDate(row.closed_on)}</Num>
        </Fact>
        {row.reversed_count === 0 ? null : (
          <Fact label={<Localized id="month-col-reversed" />}>
            <Localized id="month-reversed-before" vars={{ count: row.reversed_count }}>
              <span />
            </Localized>
          </Fact>
        )}
      </dl>
      <div>
        <Button
          variant="danger"
          onClick={() => {
            reverse.reset();
            setOpen(true);
          }}
        >
          <Localized id="month-reverse-button" />
        </Button>
      </div>
      <ReasonDialog
        open={open}
        onOpenChange={setOpen}
        title={<Localized id="month-reverse-title" vars={{ month: name }} />}
        description={<Localized id="month-reverse-description" vars={{ month: name }} />}
        reasonLabel={<Localized id="month-reverse-reason" />}
        confirmLabel={<Localized id="month-reverse-confirm" />}
        error={error}
        busy={flight.busy || reverse.isPending}
        onConfirm={submit}
      />
      {/* A refusal that points at the month itself (nothing to reverse) is told on the card too:
          the box may already be closed when the backend answers. */}
      {error === null || open ? null : <MonthRefusal error={error} month={row.month} />}
    </Card>
  );
}
