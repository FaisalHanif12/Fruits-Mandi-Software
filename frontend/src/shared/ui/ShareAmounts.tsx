// What each partner of a trip gets of money that comes in (PRD C25): his part of the seller credit
// of a trip lot, or of the money of a sale in another shop. Each part is the backend's, with the
// working of the part behind its ⓘ. The parts are in the order the partners were created.
// Nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import type { ShareAmount } from '../../bindings/ShareAmount';
import { formatBps } from '../format';
import { displayName, useLanguage } from '../i18n';
import { CalcInfo } from './CalcInfo';
import { Num } from './Num';

interface ShareAmountsProps {
  credits: readonly ShareAmount[];
  /** The name of the list for a screen reader. Comes from Fluent. */
  label: string;
}

export function ShareAmounts({ credits, label }: ShareAmountsProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  if (credits.length === 0) {
    return (
      <Localized id="ventures-credits-none">
        <p className="type-body text-ink-muted" />
      </Localized>
    );
  }
  return (
    <ul aria-label={label} className="flex flex-col">
      {credits.map((credit) => (
        <li
          key={credit.partner.id}
          className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 border-b border-border py-1 type-body text-ink"
        >
          <span className="flex flex-wrap items-baseline gap-x-2">
            <span lang={language}>{displayName(credit.partner, language)}</span>
            <span className="text-ink-muted">
              <Num>
                {l10n.getString('ventures-share-percent', { value: formatBps(credit.share_bps) })}
              </Num>
            </span>
          </span>
          <span className="flex items-center gap-1">
            <Num paisa={credit.amount_paisa} />
            <CalcInfo working={credit.working} inCell />
          </span>
        </li>
      ))}
    </ul>
  );
}
