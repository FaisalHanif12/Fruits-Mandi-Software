// What each partner pays of one payment, before it is saved (PRD FR-12.30, C23): his part, the
// source of it in words (from the shop's cash, or from his own pocket), and the working of the
// part with its ⓘ. A shop part comes off his total at once; a pocket part is only recorded. The
// two sums at the bottom are the backend's: what leaves روکڑ, and what is recorded only.
// Nothing is added up here (CLAUDE.md rule 1). The source is told in words and an icon, never by
// colour alone (NFR-USE-12).
//
// Four states: nothing to show yet (the entry cannot be worked out), or the parts. The loading
// and error states belong to the preview the parts come from, which the screen shows.

import { Localized, useLocalization } from '@fluent/react';
import { Banknote, HandCoins } from 'lucide-react';
import type { EntryPart } from '../../bindings/EntryPart';
import type { EntryParts } from '../../bindings/EntryParts';
import { formatBps, isZeroPaisa } from '../format';
import { displayName, useLanguage } from '../i18n';
import { CalcInfo } from './CalcInfo';
import { Card } from './Card';
import { Num } from './Num';

interface EntryPartsListProps {
  parts: readonly EntryPart[];
  /** The name of the list for a screen reader. Comes from Fluent. */
  label: string;
}

/** The parts of one payment, one line for each partner. Also used on the card of a saved entry. */
export function EntryPartsList({ parts, label }: EntryPartsListProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  return (
    <ul className="flex flex-col" aria-label={label}>
      {parts.map((part) => {
        const shop = part.source === 'shop';
        const Icon = shop ? Banknote : HandCoins;
        return (
          <li
            key={part.partner.id}
            className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 border-b border-border py-1 type-body text-ink"
          >
            <span className="flex flex-wrap items-baseline gap-x-2">
              <span lang={language}>{displayName(part.partner, language)}</span>
              <span className="text-ink-muted">
                <Num>
                  {l10n.getString('ventures-share-percent', { value: formatBps(part.share_bps) })}
                </Num>
              </span>
            </span>
            <span className="value-col">
              <span className="inline-flex items-center gap-1 text-ink-muted">
                <Icon aria-hidden="true" className="size-icon-sm shrink-0" />
                <Localized id={shop ? 'ventures-part-shop' : 'ventures-part-pocket'} />
              </span>
              <Num paisa={part.amount_paisa} />
              <CalcInfo working={part.working} inCell />
            </span>
          </li>
        );
      })}
    </ul>
  );
}

interface EntryPartsCardProps {
  /** The parts from the preview, or null while the entry cannot be worked out. */
  parts: EntryParts | null;
  /** The title: what the card is about. Comes from Fluent. */
  title: string;
  /** The words shown while there is nothing to show yet. */
  emptyId?: string;
}

export function EntryPartsCard({
  parts,
  title,
  emptyId = 'ventures-parts-empty',
}: EntryPartsCardProps) {
  return (
    <Card compact title={title}>
      {parts === null ? (
        <Localized id={emptyId}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <>
          <EntryPartsList parts={parts.parts} label={title} />
          <dl className="flex flex-col">
            <div className="flex items-center justify-between gap-3 py-1 type-label text-ink">
              <Localized id="ventures-parts-shop-total">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num paisa={parts.shop_total_paisa} />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 py-1 type-label text-ink">
              <Localized id="ventures-parts-pocket-total">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num paisa={parts.pocket_total_paisa} />
              </dd>
            </div>
          </dl>
          {isZeroPaisa(parts.shop_total_paisa) ? (
            <Localized id="ventures-parts-no-posting">
              <p className="type-body text-ink-muted" />
            </Localized>
          ) : null}
        </>
      )}
    </Card>
  );
}
