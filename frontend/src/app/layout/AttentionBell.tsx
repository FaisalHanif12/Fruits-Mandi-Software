// The attention bell in the top bar (PRD FR-7.44, FR-5.6): a bell with the number of things to do,
// and a list that opens under it. Each line says what to do and opens the right screen. The number
// and the lines are the backend's (`assist_attention`); the bell asks again after every save and
// once a minute (NFR-SPD-22). A new version of the app is one more line at the top of the list, only
// a note: installing stays in Settings. Four states: loading, an error with a retry, nothing to do,
// and the lines. The bell never blocks typing: it is one small read in the background.

import { Localized, useLocalization } from '@fluent/react';
import { Bell, BellRing, Download } from 'lucide-react';
import { Popover } from 'radix-ui';
import { useState } from 'react';
import {
  AttentionLines,
  useAttention,
  useOpenDestination,
  type Destination,
} from '../../features/dashboard';
import { useStartupUpdateCheck } from '../../features/settings';
import { uiErrorOf } from '../../shared/api';
import { useUiLanguage } from '../../shared/i18n';
import { EmptyState, ErrorState, LoadingState, Num } from '../../shared/ui';

export function AttentionBell() {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const [open, setOpen] = useState(false);
  const attention = useAttention();
  const update = useStartupUpdateCheck(true);
  const openDestination = useOpenDestination();
  const count = attention.data?.total_count ?? 0;
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
  } else if (attention.data.items.length === 0 && newVersion === null) {
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
        <AttentionLines items={attention.data.items} onOpen={go} />
      </div>
    );
  }

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger
        aria-label={label}
        title={label}
        data-attention-count={count}
        className="relative ms-auto flex size-control-compact items-center justify-center rounded-2 border border-border bg-surface text-ink"
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
          className="z-30 flex max-h-screen w-full max-w-reading flex-col gap-2 overflow-y-auto rounded-2 border border-border bg-surface p-3 text-ink shadow-overlay"
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
