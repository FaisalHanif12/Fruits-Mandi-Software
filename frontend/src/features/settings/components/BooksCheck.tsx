// The daily books check (NFR-OBS-5, NFR-REL-8): the result of the last run and a button to run it now.
// Three checks: the books (the trial balance difference is 0 and the two hash chains match), the data
// file (the check at start-up) and the backups. The backend sends states and numbers, no words: every
// word here is from Fluent. Each check says its state in words and with an icon, never by colour alone
// (NFR-USE-12). The difference, if there is one, is the backend's amount shown by formatPaisa.

import { Localized, useLocalization } from '@fluent/react';
import { CircleCheck, CircleHelp, CircleX, TriangleAlert, type LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import type { BackupHealth } from '../../../bindings/BackupHealth';
import type { BooksHealth } from '../../../bindings/BooksHealth';
import type { ChainHealth } from '../../../bindings/ChainHealth';
import type { DailyHealth } from '../../../bindings/DailyHealth';
import type { DataHealth } from '../../../bindings/DataHealth';
import { uiErrorOf } from '../../../shared/api';
import { formatPaisa, useGrouping, utcToLocalDisplay } from '../../../shared/format';
import { Button, Code, ErrorNotice, ErrorState, LoadingState, Num } from '../../../shared/ui';
import { useLastHealth, useRunHealth } from '../maintenance-hooks';

type Tone = 'ok' | 'warn' | 'bad' | 'unknown';

const TONE_ICON: Record<Tone, LucideIcon> = {
  ok: CircleCheck,
  warn: TriangleAlert,
  bad: CircleX,
  unknown: CircleHelp,
};

const TONE_TEXT: Record<Tone, string> = {
  ok: 'text-money-in',
  warn: 'text-warn',
  bad: 'text-due',
  unknown: 'text-ink-muted',
};

/** One line of a check: its icon and its words. */
function Line({ tone, children }: { tone: Tone; children: ReactNode }) {
  const Icon = TONE_ICON[tone];
  return (
    <li className="flex items-start gap-2">
      <Icon aria-hidden="true" className={`mt-1 size-icon-md shrink-0 ${TONE_TEXT[tone]}`} />
      <span className="type-body min-w-0">{children}</span>
    </li>
  );
}

function CheckCard({ title, children }: { title: ReactNode; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-2 rounded-3 border border-border bg-surface p-4">
      <h3 className="type-control">{title}</h3>
      <ul className="flex flex-col gap-2">{children}</ul>
    </div>
  );
}

/** One hash chain: ok, or the first entry that does not match its seal (or no entry named). */
function ChainLine({ chain, kind }: { chain: ChainHealth; kind: 'journal' | 'audit' }) {
  if (chain.ok) {
    return (
      <Line tone="ok">
        <Localized id={`settings-check-${kind}-ok`} />
      </Line>
    );
  }
  return (
    <Line tone="bad">
      {chain.at === null ? (
        <Localized id={`settings-check-${kind}-bad-unknown`} />
      ) : (
        <Localized id={`settings-check-${kind}-bad`} vars={{ entry: chain.at }} />
      )}
    </Line>
  );
}

function BooksCard({ books }: { books: BooksHealth }) {
  const grouping = useGrouping();
  return (
    <CheckCard title={<Localized id="settings-check-books" />}>
      {!books.ran ? (
        <Line tone="bad">
          <Localized id="settings-check-books-not-run" />
        </Line>
      ) : books.balanced ? (
        <Line tone="ok">
          <Localized
            id="settings-check-books-balanced"
            vars={{ entries: String(books.journal_entries) }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        </Line>
      ) : (
        <Line tone="bad">
          <Localized
            id="settings-check-books-difference"
            vars={{ difference: formatPaisa(books.difference_paisa, { grouping }) }}
          />
        </Line>
      )}
      {books.ran ? (
        <>
          <ChainLine chain={books.journal_chain} kind="journal" />
          <ChainLine chain={books.audit_chain} kind="audit" />
        </>
      ) : null}
    </CheckCard>
  );
}

function DataCard({ data }: { data: DataHealth }) {
  const tone: Tone = data.state === 'ok' ? 'ok' : data.state === 'failed' ? 'bad' : 'unknown';
  return (
    <CheckCard title={<Localized id="settings-check-data" />}>
      <Line tone={tone}>
        <Localized id={`settings-check-data-${data.state}`} />
      </Line>
      {data.state === 'failed' && data.reference !== null ? (
        <Line tone="unknown">
          <Localized
            id="settings-check-data-reference"
            vars={{ reference: data.reference }}
            elems={{ code: <Code /> }}
          >
            <span />
          </Localized>
        </Line>
      ) : null}
    </CheckCard>
  );
}

function BackupCard({ backup }: { backup: BackupHealth | null }) {
  if (backup === null) {
    return (
      <CheckCard title={<Localized id="settings-check-backup" />}>
        <Line tone="unknown">
          <Localized id="settings-check-backup-unknown" />
        </Line>
      </CheckCard>
    );
  }
  const hours = backup.cloud_age_hours;
  let tone: Tone = 'unknown';
  let words: ReactNode;
  if (backup.cloud_level === 'off') {
    words = <Localized id="settings-check-backup-off" />;
  } else if (hours === null) {
    // Cloud is on but no copy has ever gone up.
    tone = 'bad';
    words = <Localized id="settings-check-backup-never" />;
  } else {
    tone = backup.cloud_level === 'green' ? 'ok' : backup.cloud_level === 'red' ? 'warn' : 'bad';
    words = (
      <Localized
        id={`settings-check-backup-${backup.cloud_level}`}
        vars={{ hours: String(Math.trunc(hours)) }}
      />
    );
  }
  return (
    <CheckCard title={<Localized id="settings-check-backup" />}>
      <Line tone={tone}>{words}</Line>
      {backup.queue_len > 0 ? (
        <Line tone="warn">
          <Localized id="settings-check-backup-queue" vars={{ count: backup.queue_len }} />
        </Line>
      ) : null}
      {backup.errors > 0 ? (
        <Line tone="warn">
          <Localized id="settings-check-backup-errors" vars={{ count: backup.errors }} />
        </Line>
      ) : null}
    </CheckCard>
  );
}

function Result({ health }: { health: DailyHealth }) {
  const Icon = health.ok ? CircleCheck : TriangleAlert;
  return (
    <div className="flex flex-col gap-3">
      <p
        className={`type-control flex items-center gap-2 ${health.ok ? 'text-money-in' : 'text-due'}`}
      >
        <Icon aria-hidden="true" className="size-icon-md shrink-0" />
        <Localized id={health.ok ? 'settings-check-ok' : 'settings-check-bad'} />
      </p>
      <Localized id="settings-check-last" vars={{ time: utcToLocalDisplay(health.checked_at) }}>
        <p className="type-body text-ink-muted" />
      </Localized>
      <div className="grid gap-3">
        <BooksCard books={health.books} />
        <DataCard data={health.data} />
        <BackupCard backup={health.backup} />
      </div>
    </div>
  );
}

export function BooksCheck() {
  const { l10n } = useLocalization();
  const last = useLastHealth();
  const run = useRunHealth();
  const error = run.error === null ? null : uiErrorOf(run.error);

  let body: ReactNode;
  if (last.isPending) {
    body = <LoadingState rows={3} />;
  } else if (last.isError) {
    body = (
      <ErrorState
        title={<Localized id="settings-check-failed" />}
        error={uiErrorOf(last.error)}
        onRetry={() => {
          void last.refetch();
        }}
      />
    );
  } else if (last.data === null) {
    body = (
      <Localized id="settings-check-none">
        <p className="type-body text-ink-muted" />
      </Localized>
    );
  } else {
    body = <Result health={last.data} />;
  }

  return (
    <section className="flex flex-col gap-3" aria-label={l10n.getString('settings-check-title')}>
      <Localized id="settings-check-title">
        <h2 className="type-section" />
      </Localized>
      <Localized id="settings-check-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} />}
      <div>
        <Button
          variant="primary"
          busy={run.isPending}
          onClick={() => {
            run.mutate();
          }}
        >
          <Localized id={run.isPending ? 'settings-check-running' : 'settings-check-run'} />
        </Button>
      </div>
      <div aria-live="polite">{body}</div>
    </section>
  );
}
