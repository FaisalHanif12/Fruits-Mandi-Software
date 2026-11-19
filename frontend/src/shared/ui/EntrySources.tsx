// Where each partner's part of one payment comes from (PRD FR-12.29, A54): the source of the
// trip, filled in and changeable on this entry. One row for every partner in the trip, with his
// share and a two-way choice: from the shop's cash (it comes off his total) or from his own
// pocket (recorded only, nothing posts).
//
// The choices are kept by the screen, as partner id to source (Code Standards F39). A partner
// with no choice yet shows the trip's source. The parts with their amounts are not worked out
// here: they come from the backend's preview (EntryPartsCard).

import { Localized, useLocalization } from '@fluent/react';
import type { MoneySource } from '../../bindings/MoneySource';
import type { VenturePartnerRow } from '../../bindings/VenturePartnerRow';
import { formatBps } from '../format';
import { displayName, useLanguage } from '../i18n';
import { Num } from './Num';
import { SegmentedControl } from './SegmentedControl';

interface EntrySourcesProps {
  /** The partners in the trip, with their shares and default sources. */
  partners: readonly VenturePartnerRow[];
  /** This entry's choices by partner id. A missing one means the trip's source. */
  value: Readonly<Record<number, MoneySource>>;
  onChange: (partnerId: number, source: MoneySource) => void;
}

/** The source shown for a partner: this entry's choice, else the trip's. */
export function sourceOf(
  partner: VenturePartnerRow,
  value: Readonly<Record<number, MoneySource>>,
): MoneySource {
  return value[partner.partner.id] ?? partner.source;
}

export function EntrySources({ partners, value, onChange }: EntrySourcesProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  return (
    <ul aria-label={l10n.getString('ventures-sources-label')} className="flex flex-col">
      {partners.map((partner) => {
        const name = displayName(partner.partner, language);
        const label = l10n.getString('ventures-source-of', { name });
        return (
          <li
            key={partner.partner.id}
            className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 border-b border-border py-1 type-body text-ink"
          >
            <span className="flex items-baseline gap-2">
              <span lang={language}>{name}</span>
              <span className="text-ink-muted">
                <Num>
                  {l10n.getString('ventures-share-percent', {
                    value: formatBps(partner.share_bps),
                  })}
                </Num>
              </span>
            </span>
            <SegmentedControl<MoneySource>
              groupLabel={label}
              value={sourceOf(partner, value)}
              onChange={(source) => {
                onChange(partner.partner.id, source);
              }}
              options={[
                { value: 'shop', label: <Localized id="ventures-source-shop" /> },
                { value: 'pocket', label: <Localized id="ventures-source-pocket" /> },
              ]}
            />
          </li>
        );
      })}
    </ul>
  );
}
