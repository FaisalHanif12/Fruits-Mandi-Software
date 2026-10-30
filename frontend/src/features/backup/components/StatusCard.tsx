// The state of the backups on the backup screen (PRD FR-20.6, FR-20.7, NFR-BAK-10, NFR-BAK-11,
// NFR-BAK-13): the last good copy of each place, the cloud copies waiting for the internet, the
// USB drive, the latest problems and what is still to do. Every word and time comes from the
// backend's state; the screen only chooses which words to show. Colour always has an icon and words
// with it (NFR-USE-12).

import { Localized } from '@fluent/react';
import { Check, CircleAlert } from 'lucide-react';
import type { ReactNode } from 'react';
import { utcToLocalDisplay } from '../../../shared/format';
import { Button, ErrorMessage, Notice, Num } from '../../../shared/ui';
import { kindId } from '../format';
import type { BackupStatus } from '../types';

interface StatusCardProps {
  status: BackupStatus;
  /** The Owner asks for a test restore of the newest copy. */
  onTest: () => void;
}

/** One fact of the state as a chip: what it is, then its value (plan 38). */
function Chip({ labelId, children }: { labelId: string; children: ReactNode }) {
  return (
    <div className="flex items-center gap-2 rounded-2 border border-border bg-surface px-3 py-2">
      <Localized id={labelId}>
        <dt className="type-body text-ink-muted" />
      </Localized>
      <dd className="flex items-center gap-2 type-body">{children}</dd>
    </div>
  );
}

/** A time of the last copy, or "none yet". */
function When({ at }: { at: string | null }) {
  return at === null ? (
    <Localized id="backup-state-never">
      <span />
    </Localized>
  ) : (
    <span className="whitespace-nowrap">
      <Num>{utcToLocalDisplay(at)}</Num>
    </span>
  );
}

export function StatusCard({ status, onTest }: StatusCardProps) {
  const cloudBad = status.cloud_level === 'red' || status.cloud_level === 'alert';
  return (
    <section aria-labelledby="backup-state-heading" className="flex shrink-0 flex-col gap-2">
      <Localized id="backup-state-title">
        <h2 id="backup-state-heading" className="sr-only" />
      </Localized>
      {/* The four copies the Owner asks about, as one row of chips (plan 38). */}
      <dl className="flex flex-wrap gap-2">
        <Chip labelId="backup-state-local">
          <When at={status.last_local_at} />
        </Chip>
        <Chip labelId="backup-state-usb">
          {!status.usb_enabled ? (
            <Localized id="backup-state-off">
              <span className="text-ink-muted" />
            </Localized>
          ) : (
            <>
              <When at={status.last_usb_at} />
              {status.usb_present ? null : (
                <Localized id="backup-state-usb-missing">
                  <span className="text-warn" />
                </Localized>
              )}
            </>
          )}
        </Chip>
        <Chip labelId="backup-state-cloud">
          {!status.cloud_enabled ? (
            <Localized id="backup-state-off">
              <span className="text-ink-muted" />
            </Localized>
          ) : (
            <>
              {cloudBad ? (
                <CircleAlert aria-hidden="true" className="size-icon-sm shrink-0 text-due" />
              ) : (
                <Check aria-hidden="true" className="size-icon-sm shrink-0 text-money-in" />
              )}
              <When at={status.last_cloud_at} />
              <Localized id={cloudBad ? 'backup-state-cloud-old' : 'backup-state-cloud-fresh'}>
                <span className={cloudBad ? 'text-due' : 'text-money-in'} />
              </Localized>
            </>
          )}
        </Chip>
        {status.queue_len > 0 ? (
          <Chip labelId="backup-state-queue">
            <Num>{String(status.queue_len)}</Num>
          </Chip>
        ) : null}
        <Chip labelId="backup-state-last-test">
          <When at={status.last_test_restore_at} />
        </Chip>
      </dl>
      {status.running ? (
        <Notice tone="info">
          <Localized id="backup-state-running" />
        </Notice>
      ) : null}
      {status.errors.map((problem, index) => (
        <Notice key={`${problem.kind}-${problem.at}-${String(index)}`} tone="error">
          <span className="flex flex-wrap items-baseline gap-x-2">
            <Localized id={kindId(problem.kind)}>
              <span className="type-label" />
            </Localized>
            <ErrorMessage
              item={{ code: problem.code, values: { reason: problem.reason ?? 'other' } }}
            >
              <span />
            </ErrorMessage>
            <Num>{utcToLocalDisplay(problem.at)}</Num>
          </span>
        </Notice>
      ))}
      {status.test_restore_due ? (
        <Notice
          tone="warn"
          action={
            <Button onClick={onTest}>
              <Localized id="backup-test-now" />
            </Button>
          }
        >
          <Localized id="backup-state-test-due" />
        </Notice>
      ) : null}
      {status.pack_pending ? (
        <Localized id="backup-state-pack-pending">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </section>
  );
}
