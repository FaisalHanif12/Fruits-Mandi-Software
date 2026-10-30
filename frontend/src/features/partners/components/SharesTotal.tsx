// The running total of the shares (PRD FR-17.1): green at exactly 100, red otherwise. The number
// is the backend's total, written from its digits (partners_check_shares), never added here
// (CLAUDE.md rule 1). Colour is never the only signal: an icon and words go with it (NFR-USE-12).

import { Localized } from '@fluent/react';
import { Check, CircleAlert, LoaderCircle } from 'lucide-react';
import { Num } from '../../../shared/ui';

export type TotalState = 'ok' | 'bad' | 'checking';

interface SharesTotalProps {
  state: TotalState;
  /** One row: the words, the number and the verdict side by side (the strip under the title). */
  compact?: boolean;
  /** The total as percent text, for example "90", or null before the first answer. */
  total: string | null;
}

const BOX: Record<TotalState, string> = {
  ok: 'bg-money-in-soft text-money-in',
  bad: 'bg-due-soft text-due',
  checking: 'bg-surface-sunken text-ink-muted',
};

export function SharesTotal({ state, total, compact = false }: SharesTotalProps) {
  const Icon = state === 'ok' ? Check : state === 'bad' ? CircleAlert : LoaderCircle;
  const messageId =
    state === 'ok'
      ? 'partners-total-ok'
      : state === 'bad'
        ? 'partners-total-bad'
        : 'partners-total-checking';
  return (
    <div
      role="status"
      className={`flex min-w-0 gap-1 rounded-2 px-4 py-2 ${compact ? 'flex-row flex-wrap items-center gap-x-4' : 'flex-col'} ${BOX[state]}`}
    >
      <div className="flex items-center justify-between gap-3">
        <Localized id="partners-total-label">
          <span className="type-label" />
        </Localized>
        <span className="type-title">
          <Num>{total === null ? '—' : `${total}%`}</Num>
        </span>
      </div>
      <div className="flex items-center gap-2">
        <Icon aria-hidden="true" className="size-icon-md shrink-0" />
        <Localized id={messageId}>
          <span className="type-body" />
        </Localized>
      </div>
    </div>
  );
}
