// What the entry on the خرچہ form will do, before it is saved (PRD FR-16.8): for a دکان خرچہ each
// partner's share (C11, E2E-20), for a ذاتی خرچہ or its واپسی the chosen partner's total now and
// after (C22) and a note that the other partners are not affected. Every figure and every working is
// the backend's (the same function as the save); the screen shows them and works nothing out.

import { Localized, useLocalization } from '@fluent/react';
import { Card, CalcInfo, Fact, Notice, Num, ShareAmounts } from '../../../shared/ui';
import type { ExpensePreview } from '../types';
import type { ExpenseType } from '../screen-type';
import { SignedNum } from './SignedNum';

interface ExpensePreviewCardProps {
  type: ExpenseType;
  /** The backend's answer for this type, or undefined while there is nothing to ask about. */
  preview: ExpensePreview | undefined;
}

export function ExpensePreviewCard({ type, preview }: ExpensePreviewCardProps) {
  const { l10n } = useLocalization();
  const shop = type === 'shop_expense';
  if (preview === undefined) {
    return (
      <Card compact>
        <Localized id={shop ? 'cashbook-preview-hint-shop' : 'cashbook-preview-hint-personal'}>
          <p className="type-body text-ink-muted" />
        </Localized>
      </Card>
    );
  }
  if (shop) {
    return (
      <Card compact title={<Localized id="cashbook-shares-title" />}>
        {preview.shares.length === 0 ? (
          <Localized id="cashbook-shares-none">
            <p className="type-body text-ink-muted" />
          </Localized>
        ) : (
          <ShareAmounts credits={preview.shares} label={l10n.getString('cashbook-shares-title')} />
        )}
      </Card>
    );
  }
  const effect = preview.partner_effect;
  return (
    <Card compact title={<Localized id="cashbook-effect-title" />}>
      {effect === null ? (
        <Localized id="cashbook-effect-none">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <dl className="grid grid-cols-2 gap-3">
          <Fact label={<Localized id="cashbook-effect-total-now" />}>
            <SignedNum paisa={effect.total_before_paisa} />
          </Fact>
          <Fact label={<Localized id="cashbook-effect-total-after" />}>
            <span className="flex items-center gap-1">
              <SignedNum paisa={effect.total_after_paisa} />
              <CalcInfo working={effect.working} />
            </span>
          </Fact>
          <Fact label={<Localized id="cashbook-effect-open-now" />}>
            <Num paisa={effect.open_before_paisa} />
          </Fact>
          <Fact label={<Localized id="cashbook-effect-open-after" />}>
            <Num paisa={effect.open_after_paisa} />
          </Fact>
        </dl>
      )}
      {preview.others_not_affected ? (
        <Notice tone="info">
          <Localized id="cashbook-others-not-affected" />
        </Notice>
      ) : null}
    </Card>
  );
}
