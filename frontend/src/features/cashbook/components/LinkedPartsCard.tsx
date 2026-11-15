// What an entry on a linked khata or on a loan that is linked does to each partner's کل رقم, before it is
// saved (PRD FR-16.6, FR-16.13, plan 76 points 5 and 10): the backend's `linked_parts`, one row for each
// partner in the order of his id, signed (minus when money goes out on his account, plus when it comes back).
// Nothing is worked out here: the amounts are those of the function the save uses (CLAUDE.md rule 1). The
// partners' names are not behind the partner code (FR-17.11); their totals are not shown here.

import { Localized, useLocalization } from '@fluent/react';
import { Users } from 'lucide-react';
import type { LinkedPart } from '../../../bindings/LinkedPart';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Card } from '../../../shared/ui';
import { usePartnerList } from '../../partners';
import { SignedNum } from './SignedNum';

interface LinkedPartsCardProps {
  /** The preview's `linked_parts`; none when the entry touches no linked account. */
  parts: readonly LinkedPart[];
}

export function LinkedPartsCard({ parts }: LinkedPartsCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const partners = usePartnerList();
  if (parts.length === 0) return null;
  return (
    <Card compact icon={Users} title={<Localized id="cashbook-linked-parts-title" />}>
      <ul
        className="flex flex-col gap-1"
        aria-label={l10n.getString('cashbook-linked-parts-title')}
      >
        {parts.map((part) => {
          const partner = partners.data?.find((one) => one.id === part.partner_id);
          return (
            <li key={part.partner_id} className="flex items-baseline justify-between gap-4">
              <span className="type-body">
                {partner === undefined ? String(part.partner_id) : displayName(partner, language)}
              </span>
              <SignedNum paisa={part.amount_paisa} />
            </li>
          );
        })}
      </ul>
    </Card>
  );
}
