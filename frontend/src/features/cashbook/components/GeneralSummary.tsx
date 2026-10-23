// The totals of the جنرل entry (PRD 16.4): the بنام total, the جمع total, their difference and a
// word that says whether the two sides are equal. Every figure is the backend's from
// `general_preview` (CLAUDE.md rule 1): the screen adds nothing up. The entry can be saved only
// when the backend says so (`can_save`).

import { Localized, useLocalization } from '@fluent/react';
import type { GeneralPreview } from '../types';
import { Card, CashNote, Fact, Notice, Num, ShareAmounts } from '../../../shared/ui';
import { SignedNum } from './SignedNum';

interface GeneralSummaryProps {
  /** The backend's answer for the lines on screen now, or undefined while there is none. */
  preview: GeneralPreview | undefined;
}

export function GeneralSummary({ preview }: GeneralSummaryProps) {
  const { l10n } = useLocalization();
  if (preview === undefined) {
    return (
      <Card compact title={<Localized id="cashbook-general-totals-title" />}>
        <Localized id="cashbook-general-totals-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
      </Card>
    );
  }
  return (
    <div className="flex flex-col gap-3">
      <Card compact title={<Localized id="cashbook-general-totals-title" />}>
        <dl className="grid grid-cols-3 gap-3">
          <Fact label={<Localized id="cashbook-general-total-dr" />}>
            <Num paisa={preview.total_dr_paisa} />
          </Fact>
          <Fact label={<Localized id="cashbook-general-total-cr" />}>
            <Num paisa={preview.total_cr_paisa} />
          </Fact>
          <Fact label={<Localized id="cashbook-general-difference" />}>
            <SignedNum paisa={preview.difference_paisa} />
          </Fact>
        </dl>
        {preview.is_balanced ? (
          <Notice tone="info">
            <Localized id="cashbook-general-balanced" />
          </Notice>
        ) : (
          <Notice tone="warn">
            <Localized id="cashbook-general-not-balanced" />
          </Notice>
        )}
      </Card>
      {preview.split.length === 0 ? null : (
        <Card compact title={<Localized id="cashbook-general-split-title" />}>
          <Localized id="cashbook-general-split-help">
            <p className="type-body text-ink-muted" />
          </Localized>
          <ShareAmounts
            credits={preview.split}
            label={l10n.getString('cashbook-general-split-title')}
          />
        </Card>
      )}
      <CashNote cash={preview.cash} />
    </div>
  );
}
