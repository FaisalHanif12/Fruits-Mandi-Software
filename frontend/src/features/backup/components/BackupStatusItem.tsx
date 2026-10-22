// The last good cloud backup in the status bar (PRD FR-20.6, NFR-BAK-10, FR-5.5). Green with a tick
// while the last good cloud copy is under 24 hours old; red with a warning sign and the words when
// it is older, when there is none yet, or when a copy failed its check (FR-20.7). Colour always has
// an icon and words with it (NFR-USE-12). The whole item opens the backup screen. The state is
// the backend's: the screen works out nothing but which words to show.

import { Localized } from '@fluent/react';
import { Check, CircleAlert, CircleDashed } from 'lucide-react';
import { utcToLocalDisplay } from '../../../shared/format';
import { Code } from '../../../shared/ui';
import { useBackupStatus } from '../hooks';
import type { BackupStatus } from '../types';

interface BackupStatusItemProps {
  /** Opens the backup screen. */
  onOpen: () => void;
}

type Tone = 'good' | 'bad' | 'quiet';

interface Shown {
  tone: Tone;
  messageId: string;
  vars: Record<string, string>;
}

/** Which words the status bar shows for a state of the backups. */
export function statusBarWords(status: BackupStatus): Shown {
  const when = status.last_cloud_at === null ? '' : utcToLocalDisplay(status.last_cloud_at);
  // A copy that was made and then failed its check is the loudest thing to say (FR-20.7).
  if (status.errors.some((item) => item.check_failed)) {
    return { tone: 'bad', messageId: 'backup-bar-check-failed', vars: {} };
  }
  switch (status.cloud_level) {
    case 'off':
      return { tone: 'quiet', messageId: 'layout-status-backup-none', vars: {} };
    case 'green':
      return {
        tone: 'good',
        messageId: status.queue_len > 0 ? 'backup-bar-ok-queue' : 'backup-bar-ok',
        vars: { date: when, queued: String(status.queue_len) },
      };
    case 'red':
    case 'alert':
      return status.last_cloud_at === null
        ? { tone: 'bad', messageId: 'backup-bar-never', vars: {} }
        : { tone: 'bad', messageId: 'backup-bar-old', vars: { date: when } };
  }
}

export function BackupStatusItem({ onOpen }: BackupStatusItemProps) {
  const status = useBackupStatus();
  if (status.isPending) return null;
  const shown: Shown =
    status.data === undefined
      ? { tone: 'quiet', messageId: 'backup-bar-unknown', vars: {} }
      : statusBarWords(status.data);
  const Icon = shown.tone === 'good' ? Check : shown.tone === 'bad' ? CircleAlert : CircleDashed;
  const colour =
    shown.tone === 'good' ? 'text-money-in' : shown.tone === 'bad' ? 'text-due' : 'text-ink-muted';
  return (
    <button
      type="button"
      onClick={onOpen}
      data-backup-level={status.data?.cloud_level ?? 'unknown'}
      className={`flex min-h-target items-center gap-2 rounded-1 underline ${colour}`}
    >
      <Icon aria-hidden="true" className="size-icon-sm shrink-0" />
      <Localized id={shown.messageId} vars={shown.vars} elems={{ num: <Code /> }}>
        <span />
      </Localized>
    </button>
  );
}
