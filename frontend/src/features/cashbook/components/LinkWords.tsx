// A link in words (PRD FR-16.6, FR-16.13): "Nobody", "The shop" or the partner's name. The name goes through
// `displayName` and the partner list the app already reads (F38). Nothing is worked out here.

import { Localized } from '@fluent/react';
import type { Link } from '../../../bindings/Link';
import { displayName, useLanguage } from '../../../shared/i18n';
import { usePartnerList } from '../../partners';
import { linkWordId } from '../link-draft';

export function LinkWords({ link }: { link: Link }) {
  const language = useLanguage();
  const partners = usePartnerList();
  if (link.kind !== 'partner') return <Localized id={linkWordId(link.kind)} />;
  const partner = partners.data?.find((candidate) => candidate.id === link.partner_id);
  return partner === undefined ? (
    <Localized id={linkWordId('partner')} />
  ) : (
    <>{displayName(partner, language)}</>
  );
}
