// The live partner split (PRD FR-17.4, 12.5): each partner's part of the income this بکری shares,
// by the shares of the posting day. The numbers are the backend's, and each has its working.
// The shared income itself is the line "مشترکہ آمدن" of the whole بکری: commission, لاگا and
// فون only. The seller credit of a lot of trip goods goes to the trip's partners, not here (PRD
// FR-12.26, E2E-34: shared income 1,200); that split is in the panel of the lot (OwnLotPanel).

import { Localized, useLocalization } from '@fluent/react';
import { formatBps } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { CalcInfo, Num } from '../../../shared/ui';
import type { BikriPreview } from '../types';

interface SplitPanelProps {
  preview: BikriPreview;
}

export function SplitPanel({ preview }: SplitPanelProps) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  return (
    <section
      aria-labelledby="bikri-split-heading"
      className="flex min-w-0 flex-col gap-2 rounded-3 border border-border bg-surface p-4"
    >
      <Localized id="bikri-split-title">
        <h2 id="bikri-split-heading" className="type-section text-ink" />
      </Localized>
      <dl className="flex flex-col">
        <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-label text-ink">
          <Localized id="bikri-split-shared">
            <dt />
          </Localized>
          <dd className="flex items-center gap-1">
            <Num paisa={preview.summary.shared_income} />
            <CalcInfo working={preview.summary.workings.shared_income} inCell />
          </dd>
        </div>
        {preview.split.map((part) => (
          <div
            key={part.partner_id}
            className="flex flex-wrap items-center justify-between gap-x-3 border-b border-border py-1 type-body text-ink"
          >
            <dt className="flex items-baseline gap-2">
              <span>{displayName(part, language)}</span>
              <span className="text-ink-muted">
                <Num>
                  {l10n.getString('bikri-tag-percent', { value: formatBps(part.share_bps) })}
                </Num>
              </span>
            </dt>
            <dd className="flex items-center gap-1">
              <Num paisa={part.income_paisa} />
              <CalcInfo working={part.working} inCell />
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
