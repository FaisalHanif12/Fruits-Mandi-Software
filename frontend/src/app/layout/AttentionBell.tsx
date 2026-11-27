// The attention bell in the top bar (PRD FR-7.44, FR-5.6, plan 100 J): a bell with the number of things
// to do, and a large panel that opens under it, about half the window wide and most of its height, with
// three sections (sellers to pay, buyers to collect from, general reminders; `AttentionPanelBody`). The
// number and the lines are the backend's (`assist_attention`); the bell asks again after every save and
// once a minute (NFR-SPD-22). A new version of the app is one more line at the top of the panel, only a
// note: installing stays in Settings. Four states: loading, an error with a retry, nothing to do, and the
// lines. The bell never blocks typing: it is one small read in the background.

import { Localized, useLocalization } from '@fluent/react';
import { Bell, BellRing, Download } from 'lucide-react';
import { Popover } from 'radix-ui';
import { useState } from 'react';
import type { AttentionPanel } from '../../bindings/AttentionPanel';
import {
  AttentionPanelBody,
  useAttention,
  useOpenDestination,
  type Destination,
} from '../../features/dashboard';
import { useStartupUpdateCheck } from '../../features/settings';
import { uiErrorOf } from '../../shared/api';
import { useUiLanguage } from '../../shared/i18n';
import { EmptyState, ErrorState, LoadingState, Num, radixCollisionPadding } from '../../shared/ui';

/** Nothing in any group, and nothing hidden for later: the panel then says there is nothing to do. */
function isNothing(panel: AttentionPanel): boolean {
  return (
    panel.sellers.rows.length === 0 &&
    panel.buyers.rows.length === 0 &&
    panel.payables.length === 0 &&
    panel.general.length === 0 &&
    panel.hidden_count === 0
  );
}

export function AttentionBell() {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const [open, setOpen] = useState(false);
  const attention = useAttention();
  const update = useStartupUpdateCheck(true);
  const openDestination = useOpenDestination();
  const count = attention.data?.bell_count ?? 0;
  const newVersion = update.data?.available === true ? (update.data.version ?? '') : null;
  const label = l10n.getString('attention-bell-label', { count: String(count) });
  const Icon = count > 0 ? BellRing : Bell;

  const go = (destination: Destination) => {
    setOpen(false);
    openDestination(destination);
  };

  let body;
  if (attention.isPending) {
    body = <LoadingState />;
  } else if (attention.isError) {
    body = (
      <ErrorState
        title={<Localized id="attention-load-failed" />}
        error={uiErrorOf(attention.error)}
        onRetry={() => {
          void attention.refetch();
        }}
      />
    );
  } else if (isNothing(attention.data) && newVersion === null) {
    body = (
      <EmptyState
        icon={Bell}
        title={<Localized id="attention-empty-title" />}
        help={<Localized id="attention-empty-help" />}
      />
    );
  } else {
    body = (
      <div className="flex flex-col gap-2">
        {newVersion === null ? null : (
          <button
            type="button"
            data-update-note
            onClick={() => {
              go({ kind: 'settings-app' });
            }}
            className="flex min-h-target items-center gap-3 rounded-2 bg-surface-sunken px-2 text-start"
          >
            <Download aria-hidden="true" className="size-icon-md shrink-0 text-primary" />
            <Localized
              id="attention-update"
              vars={{ version: newVersion }}
              elems={{ num: <Num /> }}
            >
              <span className="type-body min-w-0" />
            </Localized>
          </button>
        )}
        <AttentionPanelBody panel={attention.data} onOpen={go} />
      </div>
    );
  }

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger
        aria-label={label}
        title={label}
        data-attention-count={count}
        className="relative ms-auto flex size-control-compact items-center justify-center rounded-2 border border-transparent text-ink transition-calm hover:border-border hover:bg-surface-sunken"
      >
        <Icon aria-hidden="true" className="size-icon-md" />
        {count > 0 ? (
          <span
            aria-hidden="true"
            className="type-key absolute -top-2 -end-2 inline-flex h-chip min-w-chip items-center justify-center rounded-full bg-due px-1 text-on-primary"
          >
            {count}
          </span>
        ) : null}
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          lang={language}
          dir={direction}
          align="end"
          sideOffset={4}
          collisionPadding={radixCollisionPadding()}
          className="attention-panel z-30 flex flex-col gap-2 overflow-y-auto rounded-3 border border-border bg-surface p-4 text-ink shadow-overlay"
        >
          <Localized id="attention-title">
            <h2 className="type-section" />
          </Localized>
          {body}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
