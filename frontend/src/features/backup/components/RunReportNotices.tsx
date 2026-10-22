// What "back up now" did (PRD FR-20.8): one line for each place tried. A copy that is made and
// checked is a plain line; a cloud copy that waits in the queue for the internet says so (it is not
// lost, FR-20.6); a copy that failed says why, in the app language. A failure of one place is a
// result, not an error, so the other places still show their own line.

import { Localized, useLocalization } from '@fluent/react';
import { ErrorMessage, Notice, Num } from '../../../shared/ui';
import { kindId, sizeText } from '../format';
import type { RunReport, TargetResult } from '../types';

function messageIdOf(result: TargetResult): string {
  if (result.ok) return 'backup-run-ok';
  return result.queued ? 'backup-run-queued' : 'backup-run-failed';
}

export function RunReportNotices({ report }: { report: RunReport }) {
  const { l10n } = useLocalization();
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  return (
    <div className="flex flex-col gap-2">
      {report.results.map((result) => (
        <Notice key={result.kind} tone={result.ok ? 'info' : result.queued ? 'warn' : 'error'}>
          <span className="flex flex-wrap items-baseline gap-x-2">
            <Localized id={kindId(result.kind)}>
              <span className="type-label" />
            </Localized>
            <Localized id={messageIdOf(result)}>
              <span />
            </Localized>
            {result.ok ? <Num>{sizeText(result.size_bytes, say)}</Num> : null}
            {/* The reason of a failure is not part of the answer, so the general words are used. */}
            {!result.ok && !result.queued && result.code !== null ? (
              <ErrorMessage item={{ code: result.code, values: { reason: 'other' } }}>
                <span className="text-ink-muted" />
              </ErrorMessage>
            ) : null}
          </span>
        </Notice>
      ))}
    </div>
  );
}
